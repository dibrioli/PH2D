# ADR-0078 — Watercolor Gold Standard: GPU-Resident, Tiled-Sparse, Three-Layer Shallow-Water

**Status:** Proposed (2026-06-08) — North Star ratificado pelo Enio ("chegue ao padrão ouro, vá até onde ninguém foi. Seremos os melhores"). Execução em estágios (S0 feito; S1+ em andamento).
**Decisor(es):** Enio (dono/decisor) + Claude.
**Supersede/estende:** [ADR-0049 — Fluid Brushes](0049-fluid-brushes.md) §2.x — especificamente a decisão de **aproximar** a aquarela por *gated diffusion-advection* (Curtis simplificado). O modelo de solver passa a ser o **Curtis 1997 completo de três camadas**; a crate, o contrato opt-in, o graceful-degrade e o det-fallback de ADR-0049 permanecem.
**Pré-requisitos:** ADR-0048 (GPU layer compositor), ADR-0044 (brush engine GPU), [ADR-0049](0049-fluid-brushes.md).
**Tags:** painter, fluid-sim, shallow-water, kubelka-munk, tiled-sparse, gpu-resident, compositor-node, 4k, multi-layer, gold-standard
**Referências (estado-da-arte):**
- Curtis, Anderson, Seims, Fleischer, Salesin — *Computer-Generated Watercolor*, SIGGRAPH 1997 ([grail.cs.washington.edu](https://grail.cs.washington.edu/projects/watercolor/paper_small.pdf)) — o modelo físico de 3 camadas + Kubelka–Munk.
- Chu & Tai — *MoXi: real-time ink dispersion in absorbent paper*, ACM TOG 24(3), 2005 — percolação por Lattice-Boltzmann (refinamento da camada capilar).

---

## 1. Contexto — onde paramos e onde ninguém chegou

ADR-0049 entregou aquarela wet-on-wet **GPU** com uma **aproximação**: *gated diffusion-advection* (uma camada, sem campo de velocidade). Foi ratificada visualmente e roda; mas é uma sombra do modelo físico. Pós-rewrite residente (S0, 2026-06-08, [`HANDOFF_painter_fluid_gpu_composite.md`](../../HANDOFF_painter_fluid_gpu_composite.md) §4), medimos em Metal `--release`: traço típico **4K real-time já** (13ms), mas (a) o passo do solver é `O(grid)` (não `O(frente molhada)`), (b) o composite ainda faz readback por-frame, (c) o grid CPU ainda é alocado `O(grid)`, e (d) **a física é a aproximação, não a aquarela real**.

Os concorrentes:
- **Procreate / Fresco / Photoshop:** aproximação visual (smudge + blur + wet-edges fake). Sem física de fluxo.
- **Rebelle (Escape Motions):** simulação de fluido real (a melhor do mercado) — mas não a 4K, multi-camada, 60–120Hz simultâneos.

**A fronteira aberta** (o "onde ninguém foi"): **a física de aquarela completa de três camadas — backruns/cauliflower, edge-darkening, granulação, mistura multi-pigmento Kubelka–Munk — a 4K, em muitas camadas, a 60–120Hz**, viável porque o custo é `O(frente molhada)` (tiled-sparse) e GPU-residente integrado ao compositor (zero round-trip CPU por frame). Nenhum produto faz isso. Nós faremos.

## 2. Decisão — os 5 pilares do padrão-ouro

### P1 — Física: Curtis 1997 completo (3 camadas) + Kubelka–Munk multi-pigmento
Substitui a difusão-gateada de ADR-0049 por:
1. **Camada shallow-water** — altura `h`, velocidade `(u,v)`, pressão `p`. Passo `MoveWater` (Euler shallow-water: `(u,v) ← (u,v) − Δt·∇p − μ·∇²(u,v) − κ·(u,v)` com relaxamento de borda na fronteira molhada) → **fluxo real**.
2. **Camada de pigmento (multi-pigmento)** — por pigmento `k`: concentração **fluindo** `g_k` (advectada por `(u,v)`) + **depositada** `d_k` (no papel). `TransferPigment`: deposição `γ_k` + lifting + **granulação** (dependente da altura do papel) + staining → granulação, lifting, mistura.
3. **Camada capilar** — capacidade `c` + saturação `s` do papel; `CapillaryFlow` espalha água para o papel seco além da área molhada (a franja capilar) e a fronteira em recuo/avanço **acumula pigmento** → **edge-darkening + backruns (cauliflower)**. Refinamento futuro: percolação LBM (MoXi) para a camada capilar.
4. **Cor:** Kubelka–Munk K/S por-pigmento (já temos o composite K–M de ADR-0049) compondo `d_k`+`g_k` sobre o backdrop.

> Estritamente **mais rico** que a difusão-gateada: adiciona fluxo dirigido por velocidade, backruns, edge-darkening, granulação e mistura multi-pigmento — os efeitos que **definem** aquarela e que nenhum app de consumo simula em tempo real.

### P2 — Engine: GPU-residente + tiled-sparse adaptativo (`O(frente molhada)`)
- **Todo** estado (`h,u,v,p,g_k,d_k,c,s,paper`) em texturas GPU. **Zero** cópia CPU. **Papel/tooth gerado na GPU** (grain noise em compute) — elimina o grid CPU `O(grid)` e o alloc por-traço.
- **Active-tile set:** canvas em tiles (ex. 16×16). Lista de tiles ativos mantida **na GPU**; só tiles ativos (com água, + apron de 1) são simulados via **indirect dispatch**. Tile ativa quando água flui pra ela, desativa ao secar. **Custo = `O(tiles molhados)`, independente do tamanho do canvas** → 4K/8K, canvas enormes e muitas camadas, tudo em tempo real.
- **Advecção de alta ordem** (BFECC/MacCormack) → transporte de pigmento nítido sem borrão de difusão numérica.

### P3 — Integração: composite como **nó do compositor GPU**; zero sync CPU por-frame
- O composite K–M do campo molhado vira um **passo no grafo do layer-compositor GPU** (ADR-0048). O campo vivo desenha como parte do pipeline multi-camada normal → **multi-camada de graça**, sem readback, sem re-upload de canvas.
- **Readback CPU só no pen-up / flatten:** assa o pigmento depositado → a camada RGBA8 canônica (undo/Apply/persist/replay HR-5).
- A textura de preview do painter vira um **alvo GPU-escrito** que o renderer amostra (mudança foundational em `ph2d-render` — ver §4).

### P4 — Determinismo + paridade (HR-5)
Solver de **referência CPU** (o mesmo modelo de 3 camadas) permanece como **det-fallback + gate de paridade por-passo**, na resolução validada. GPU é o caminho real-time. Replay usa o caminho CPU (ou um schedule GPU de seed-fixa determinístico).

### P5 — Precisão/qualidade
fp16 para campos advectados bandwidth-bound onde a precisão permite; fp32 para pressão/acumulação. Composite mantém supersampling 2×2 (→ adaptativo).

## 3. Plano em estágios (cada um valida sozinho — visual + perf; commit local, push só após Enio OK)

| Estágio | Entrega | Estado |
|---|---|---|
| **S0** | Núcleo GPU-residente (dab-list `cs_splat` + diffuse/advect/evaporate residentes + redução GPU max-water/bbox) — ponte do caminho antigo | ✅ FEITO + smoke OK (commits 693b6f3, 1d31dc5, 8772132) |
| **S1** | **Tiled-sparse** no modelo atual (active-tile indirect dispatch + passes region-scoped) → `O(frente)`; **dropar grid CPU**; **paper-gen GPU**. Totalmente GPU-residente. | ▶ próximo |
| **S2** | Composite como **nó do compositor** + **zero readback por-frame** (E4 real + mudança foundational `ph2d-render`); bake no pen-up | |
| **S3** | Física **shallow-water de 3 camadas** (velocidade + pressão + capilar) → fluxo, backruns, edge-darkening. Referência CPU + paridade por-passo | |
| **S4** | **Multi-pigmento K–M** (granulação, staining, mistura real) + encadeamento **multi-camada** a 4K | |
| **S5** | Advecção **BFECC** + supersampling adaptativo + refinamento capilar **LBM** (MoXi); tune a 120Hz | |

Cada estágio é independentemente shippável + validado visualmente. S1–S2 são engine (escala + integração); S3–S5 são física (a alma).

## 4. Impacto em contratos congelados (mexer = ADR, por §6 do CLAUDE.md)

- **ADR-0049 caps** (`FluidSim≤12`, `GravitySource≤6`): o **descritor** `FluidSim` continua pequeno e serializável; o estado pesado de 3 camadas vive no **solver (não capado)** — por design do próprio ADR-0049 §2.4. Sem violar caps.
- **`ph2d-render` (foundational, ADR-0023/0048):** a textura de preview do painter (`IndividualTextureStore`, hoje `Rgba8UnormSrgb`, sem `STORAGE_BINDING`) precisa virar **alvo escrito pelo compositor fluido** (storage/render-target) amostrado pelo renderer — **ou** o composite fluido vira um nó do `layer_compositor`. Mudança de contrato de renderização → **decidida aqui** (S2), implementada com gate de paridade.
- **`Tool=11` / `Brush` caps:** fluid é opt-in via brush flag; sem novas variantes de `Tool`. Sem impacto.
- **Det-painter / HR-5:** mantido via referência CPU (P4).

## 5. Riscos + mitigação

- **Estabilidade shallow-water em GPUs heterogêneas** (o risco nº1 de ADR-0049): CFL clamp + relaxamento de pressão limitado + fp32 na pressão; gate de paridade vs referência CPU por-passo; graceful-degrade pro caminho de difusão (S0) em GPU incapaz.
- **Tiled-sparse correctness (classe do bug §2 do handoff):** invariante **região-do-solver ⊇ região-do-composite**; apron de tile + ativação de vizinhos cobrem a frente que avança; testes headless de "frente nunca congela".
- **Custo de contexto:** reescrita foundational → executar em **contextos frescos e focados por estágio**, cada um com validação visual do Enio.

## 6. Consequências

Aquarela fisicamente correta (backruns, granulação, edge-darkening, mistura multi-pigmento) a **4K, multi-camada, 60–120Hz** — uma capacidade que **nenhum produto de consumo entrega**. O custo por-frame vira `O(frente molhada)` + passes GPU; o `O(grid)` da CPU e o readback por-frame **desaparecem**. O caminho de difusão de ADR-0049 vira o **graceful-degrade**; a referência CPU vira o **det-fallback**. Esta ADR é o norte; cada estágio amenda o handoff vivo com o medido.
