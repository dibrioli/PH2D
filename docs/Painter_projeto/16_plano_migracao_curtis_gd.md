# Plano de migração — Wash → topologia Curtis `g`/`d` (ADR-0095)

> **Motivação.** As 3 tentativas de wash/fluid divergiram porque nenhuma implementou o modelo
> canônico — todas fizeram reduções ad-hoc com constantes inventadas. A auditoria multiagêntica
> (2026-06-15, 34 achados confirmados, 0 refutados) provou que o erro-raiz é estrutural: **falta a
> separação pigmento suspenso `g` ↔ depositado `d` com `TransferPigment(ρ,ω,γ)`** — o coração do
> Curtis 1997. Sem ela, diluição e "seca-escurece" são impossíveis (viram hacks) e nada é
> reprodutível. Este plano adota Curtis de verdade, com constantes NOMEADAS e PUBLICADAS.
>
> Fonte canônica de parâmetros: [`wash_parametros_canonicos.md`](wash_parametros_canonicos.md).
> Catálogo de bugs a não repetir: [`wash_solucao_de_erros.md`](wash_solucao_de_erros.md) (B1-B9).
> Protocolo: cada fase fecha com **gate executável** (prova comportamento PUBLICADO, não constante
> inventada) + **checkpoint visual** (Enio dirige fase a fase). GPU-first, tempo-real, zero fallback CPU.

---

## 1. O que MUDA, o que FICA, o que SAI

**FICA (provado correto pela auditoria):**
- **Cor = Mixbox residual** (ADR-0091): encode `[unmix(rgb); rgb−mix(c)]`, decode `mix(c)+r`. Cor sozinha exata, mistura espectral. A ausência de scattering `S` **não é bug** — granulação/staining entram pela DEPOSIÇÃO, não pela cor.
- **Topologia GPU-first** (ADR-0093/0094): texturas residentes, single-submit/frame, composite no slot de preview, sessão persistente (ADR-0088 — campo vive entre traços, mistura wet-on-wet).
- **Proteções B1-B9** que sobreviveram (saturação suave, AA gaussiano, etc.).

**MUDA (o núcleo):**
- Campo de cor único → **par `g_k` (suspenso) + `d_k` (depositado)** por pigmento. Espessura óptica = `g+d`.
- Difusão gated ad-hoc → **transporte Curtis** (MovePigment advecta `g`; FlowOutward Eq.3 faz edge-darkening real).
- Constantes inventadas → **constantes nomeadas publicadas** (Curtis §1, MoXi §3).

**SAI (band-aids que colidem com a física nova):**
- `pigment_load=1−water_add` (diluição forçada no input) → diluição passa a ser **emergente** (mais água ⇒ `g` espalha ⇒ menos `d`/área ⇒ transparente; ao secar `g→d` ⇒ escurece).
- `EDGE_EVAP_FLOOR` → substituído por FlowOutward `η`.
- `D_MAX`/`V_MAX`/`FIELD_CAP`/`WATER_HALO`/`dry_drive`/`MIN_PIGMENT_LOAD` → removidos ou substituídos por equivalentes Curtis/MoXi.

---

## 2. Fases (cada uma com gate executável + checkpoint visual)

### C0 — ADR-0095 (Coord-only)
Supersede a **deposição implícita** da ADR-0094 (mantém a cor Mixbox e a topologia GPU). Declara: campos `g_k`/`d_k`, `TransferPigment(ρ,ω,γ)`, tabela de 11 pigmentos §1.7, constantes nomeadas, e que diluição/seca-escurece são EMERGENTES (proíbe band-aids de input). Lista os contratos ABI a rebump (WashParams, Dab).
**Gate:** ADR revisado + aprovado pelo Enio. **Sem código.**

### C1 — Campo depositado `d` + TransferPigment (o fix estrutural)
O passo que falta. Adiciona campo `d` (rgba32f, espelha `pig`), e um pass `cs_transfer` entre o transporte e a evaporação:
```
Δdown = g · (1 − h·γ) · ρ          # adsorção  (suspenso → depositado)
Δup   = d · (1 + (h−1)·γ) · ρ / ω  # dessorção (depositado → suspenso)
clamp d≤cap, g≤cap ;  d += Δdown−Δup ;  g += Δup−Δdown
```
- `pig`/`dye`/`res` viram o campo SUSPENSO `g` (já existem). `d` é novo (mesma estrutura).
- Composite passa a ler espessura `x = g + d` (cor) e cobertura de `d` quando seco, `g+d` quando molhado.
- **Remove `pigment_load`**: o dab injeta `g` cheio + água; a diluição emerge.
- `ρ`/`ω`/`γ` entram por pigmento da tabela §1.7 (ou globais no C1, por-pigmento no C4).
**Gate executável (headless Metal):**
  - `dilution_emerges_from_water`: mesma massa de pigmento, água↑ ⇒ cobertura final↓ (diluição SEM tocar o input).
  - `drying_darkens`: evaporar a água ⇒ `g→d` ⇒ cobertura sobe (seca-escurece).
  - `mass_conserved_g_plus_d`: `Σ(g+d)` constante sob TransferPigment puro (sem evaporação).
**Checkpoint visual:** água pura dilui de verdade (transparente); deixar secar escurece; mistura azul+amarelo→verde intacta.

### C2 — Shallow-water completo (substitui o gather explícito) — ESCOLHA: Curtis completo
Troca o gather difusivo ad-hoc pelo **MoveWater canônico inteiro** (Enio 2026-06-15: shallow-water completo):
- **Campo de velocidade `u,v` (grid staggered/MAC) + pressão `p`.**
- **UpdateVelocities**: Navier-Stokes shallow-water com `μ=0.1` (viscosidade), `κ=0.01` (arrasto), perturbação por `∇h` (declive do papel), `EnforceBoundaryConditions` (v=0 fora da máscara wet).
- **RelaxDivergence**: incompressibilidade, `N=50` iter, `τ=0.01` tol, `ξ=0.1` fração.
- **FlowOutward (Eq.3)**: kernel `K=10`, `η∈[0.01,0.05]` — o edge-darkening FÍSICO (remove `EDGE_EVAP_FLOOR`).
- **MovePigment**: advecção de `g` por out-flux conservativo, bounded pelo Δt adaptativo (`Δt=1/max|u,v|`); semi-Lagrangiano de Stam (§2) como fallback se aparecer instabilidade.
- Remove `D_MAX`/`V_MAX` (caps de CFL explícito) a favor do Δt adaptativo.
**Gate:** `relax_divergence_incompressible` (∇·v→0 em ≤50 iter); `flow_outward_darkens_rim` (Eq.3 concentra `d` na borda); `no_checkerboard_extreme_flow` (B2); `advection_conserves_mass`.
**Checkpoint visual:** fluxo dirigido/tilt fiel; borda escura natural; espalhamento estável.

### C3 — Camada capilar / backruns (EM ESCOPO — natural após o shallow-water)
Saturação `s`, capacidade `c=h·(c_max−c_min)+c_min`, difusão `≈0.25` (patente), expansão de máscara → backruns/blooms. Constantes `α/ε/δ/σ` são art-directable (§1.6 — não publicadas).
**Gate:** `backrun_expands_into_dry`; `capillary_capacity_caps_saturation`. **Checkpoint:** bloom/backrun aparece ao pingar água em molhado.

### C4 — Parâmetros nomeados + pigmentos reais + UI
- Substitui TODA constante inventada por nomeada: `μ=0.1`,`κ=0.01`,`N=50`,`τ=0.01`,`ξ=0.1`,`K=10`,`η∈[0.01,0.05]`; remove `COVER_K`/`FIELD_CAP`/`WATER_HALO`/`dry_drive`.
- **Tabela §1.7**: 11 pigmentos reais com K/S/ρ/ω/γ (ou mapear a cor escolhida ao pigmento mais próximo p/ herdar ρ/ω/γ).
- **5 sliders universais** (§5): Water · **Load** · **Dilution** · Diffusion · **Mixing** (faltavam 3) + tier-2 (Edge darkening, Granulation, estado Wet/Dry). Faixas Rebelle 0–10 onde existir.
**Gate:** `no_unfounded_constants` (grep dos nomes proibidos = 0); slider round-trip; `params_count_capped` (rebump contrato). **Checkpoint:** sliders batem com comportamento físico esperado.

### C5 — Lifecycle: undo de campo (FieldSnap) + bake-on-switch + perf
- **FieldSnap** (ADR-0094 §7): `WashSolver::snapshot()/restore()` do estado (`g`,`d`,`water`,`res`) → integra no histórico transacional → **conserta o undo morto (CRÍTICO)** + remove entries fantasmas.
- **Bake-on-switch**: ao trocar de alvo, bakear no DONO antes de descartar (conserta perda de dados).
- **Perf**: readback assíncrono no pen-up (sem `wait_indefinitely` na UI); bake/step/composite por **wet-bbox** (envelope com cota); `upload_seed` escreve AMBOS gêmeos (mata o B7 latente).
**Gate:** `undo_wash_restores_field`; `switch_target_bakes_owner`; `field_snapshot_roundtrip`. **Checkpoint:** Cmd+Z desfaz traço wash; trocar de sprite preserva o trabalho.

---

## 3. Contratos / gates afetados (rebump documentado, Coord)
- `Dab=64B`, `WashParams=64B` (`abi_sizes_are_frozen`, solver.rs) — mudam ao adicionar `d`/ρ/ω/γ. Rebump com justificativa na ADR-0095.
- `architecture_painter_contract_surface` (PainterParams field cap) — +sliders Load/Dilution/Mixing pode estourar o cap de 12; avaliar sub-struct.
- Downcast allowlist (já cobre `painter_wash_gpu.rs`).

## 4. Riscos (e mitigação)
1. **Diluição dupla na transição** (band-aid antigo + emergente): remover `pigment_load` no MESMO commit que liga g/d (C1). Gate `dilution_emerges_from_water` prova que a fonte é o espalhamento, não o input.
2. **Custo GPU** (campos g+d + velocidade): canvas-res ok (MoXi fez 512²@44fps em HW 2005). Wet-bbox (C5) é o orçamento real p/ 4K.
3. **Instabilidade do transporte** (C2): Δt adaptativo Curtis primeiro; se aparecer, cair p/ semi-Lagrangiano de Stam (§2, incondicional).
4. **Regressão B1-B9**: rodar o gate de regressão da auditoria a cada fase.
5. **ABI churn**: concentrar os rebumps na ADR-0095 (C0), não pingados por fase.

## 5. Ordem (decidida Enio 2026-06-15)
C0 (ADR) → **C1 (g/d — o fix que importa)** → checkpoint → **C2 (shallow-water completo)** → C4 (params/UI, p/ você tunar) → C5 (undo/lifecycle: os 2 bugs vivos consertados aqui, FieldSnap sobre o campo settled) → C3 (backruns).
**Escopo físico:** Curtis completo (velocidade u,v + pressão + RelaxDivergence + backruns), não a versão reduzida.
**Bugs vivos:** consertados no C5 (junto), não puxados p/ frente — undo definitivo é melhor sobre o campo já estabilizado pós-C1/C2.
