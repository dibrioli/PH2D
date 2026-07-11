# 22 — Nota-ADR: Soft-Body + Wave (M4.2 — mídia contínua na família de simulação)

**Data:** 2026-07-11 · **Linha:** `line/motion-value` · **Status:** fatia M4.2 implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates. **Contratos congelados intocados** (gate
`architecture_contract_surface` verde: `NodeOp`=2 / `OpResolver`=1 / `NodeManifest`=8). Segunda fatia da
família de SIMULAÇÃO (doc 21 §5): onde M4.1 (rope+boids) eram **agentes/partículas discretas**, esta é
**mídia contínua/deformável** — um **corpo mole** e um **campo de onda**. Mesmo substrato sequencial (o `pre`
self-loop com `sim_t`/`dt`), caráter oposto.

---

## 1. O problema (doc 21 §5 — continuar a simulação)

M4.1 firmou a dinâmica sequencial com dois sistemas de **partículas discretas** (uma corda de pontos, um
enxame de agentes). Faltava a outra metade do vocabulário de simulação: **mídia contínua** — corpos que
deformam como um TODO e campos que propagam. Os dois clássicos que todo motor tem: um **soft body** (jelly
que amassa e volta à forma) e uma **onda** (ripple que se propaga a velocidade finita — algo que nenhum
oscilador per-element consegue, porque exige acoplamento espacial + memória temporal).

## 2. A pesquisa do padrão-ouro (antes de codar — DIRETIVA §1)

**`motion.soft_body` (o corpo mole):** o padrão-ouro de soft body ESTÁVEL é o **shape-matching de Müller et
al. — *Meshless Deformations Based on Shape Matching* (SIGGRAPH 2005)**. Veredito:

- Cada passo: prevê as partículas sob gravidade+inércia, acha o **melhor frame rígido** `(R, c)` da forma de
  repouso na nuvem deformada, puxa cada partícula uma fração `stiffness` rumo ao seu **goal** `R·qᵢ + c`, e
  lê a velocidade de volta da mudança de posição. **Incondicionalmente estável** (o goal é sempre uma pose
  rígida válida — nada explode), sem lista de constraints por-aresta. É o motor de blob/jelly gooey.
- A rotação `R` é a **decomposição polar de `Apq = Σ (xᵢ−c)(qᵢ)ᵀ`**, que em 2D tem **forma fechada SEM trig**:
  `(cos, sin) ∝ (Apq₀₀+Apq₁₁, Apq₁₀−Apq₀₁)`, normalizado. Um `sqrt` só (HR-5). Testado por **invariância
  rígida** (uma pose rígida da forma de repouso shape-match para si mesma — o falsificador do sinal do polar).
- A **âncora fixa a fileira de cima** (`anchor_x`/`anchor_y` value inputs): um `value.lfo` desliza o pino e o
  corpo **balança como gelatina**. `Temporal`.

**`motion.wave` (o campo de onda):** o padrão-ouro é a **equação de onda por diferenças finitas, leapfrog no
tempo**. Veredito:

- `∂²h/∂t² = c²∇²h` discretiza p/ `h_next = 2h − h_prev + C·∇²h` (leapfrog/Verlet no tempo), Laplaciano de
  5 pontos, bordas refletoras (Neumann). `C = (c·dt)²` **clampado abaixo do limite CFL 0.5** → nunca explode.
  `damping` sangra energia. O **centro é dirigido** ao value input `drive` (fonte de Dirichlet — um `value.lfo`
  emite ripples contínuos). **Aritmética pura** (HR-5: nem `sqrt`, nem trig). Testado por **propagação de
  velocidade FINITA** (1 tick após o impulso só os vizinhos imediatos mexem; uma célula longe ainda é 0 — o
  falsificador do acoplamento instantâneo) e **propagação outward** (vs `speed=0` que não propaga).
- A altura mapeia no `size` do ponto → a grade lê como **anéis concêntricos** de pontos maiores/menores.

## 3. O que foi adicionado (fatia M4.2)

**`ph2d-node-motion-soft-body` (drop-crate, o CORPO MOLE):** `(anchor_x?, anchor_y?, state) → out`. Malha
`rows×cols` shape-matching; fileira de cima fixada na âncora animável (`pin`), gravidade + goal-pull. Estado
`P`+`sb_vel` no `pre`. `Temporal`, Source, `sqrt` só na decomposição polar 2D.

**`ph2d-node-motion-wave` (drop-crate, o CAMPO DE ONDA):** `(drive?, state) → out`. Grade `rows×cols`, equação
de onda leapfrog + Laplaciano de 5 pontos (Neumann), centro dirigido por `drive`. Emite `P` (grade fixa) +
`size` (de |altura|). Estado `wave_h`+`wave_prev` no `pre`. `Temporal`, Source, aritmética pura. `center_x`/
`center_y` posicionam a grade (várias lado a lado).

**Cena boot — DUAS cenas pequenas** (`motion_demo_strobe.rs`, 8 nós, 2 sinks):

```
ESQUERDA (jelly):  soft_body → tint(magenta) → output      lfo_anchor → soft_body.anchor_x
                   soft_body --pre--> soft_body.state
DIREITA  (ripple): wave      → tint(ciano)   → output       lfo_drive  → wave.drive
                   wave --pre--> wave.state
```

- **soft_body** (x≈−6): a gravidade pendura o jelly da fileira fixada, o `anchor_x` (`value.lfo` ±1.2)
  desliza o pino → **balança como gelatina** e volta à forma.
- **wave** (centro x≈+6): o `drive` (`value.lfo`) oscila a célula central → **ripples concêntricos** incham
  os pontos pra fora.

Duas sims de mídia contínua (um corpo deformável, um campo que propaga), cada uma dirigida pelo domínio de
valor, em regiões separadas do canvas. Pequena e legível (regra do doc 12 / feedback "simplifique"). A
contraparte discreta é o doc 21 (rope+boids).

**Testes (16 unit + 3 integração):** soft_body (9: seed da malha de repouso na âncora, **invariância rígida**
[pose rígida = seu próprio goal], **recupera a forma** após deformar, gravidade pendura abaixo da fileira
fixada, âncora móvel arrasta o corpo, replay determinístico, sem-loop = malha de repouso, NaN recupera, cook
com pre-loop — falsificados); wave (7: seed plano, **fonte propaga outward** [vs speed 0], **velocidade
finita** [vizinho move no tick 2, célula distante ainda 0], **damping limita e silencia**, replay, sem-loop =
plano, cook emitindo `size` — falsificados). Integração no shell:
`the_soft_body_hangs_and_wobbles_from_the_moving_anchor` (base afunda abaixo do topo fixado + a âncora varre
o corpo) · `the_wave_ripples_outward_from_the_driven_center` (a coluna `size` ganha spread — anéis de pontos
grandes/pequenos — e fica limitada) · `the_default_document_replays_deterministically` (drena o pump
sequencial completo).

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-motion-soft-body`, tipo `motion.soft_body` | nova | nome novo |
| crate `ph2d-node-motion-wave`, tipo `motion.wave` | nova | nome novo |
| `ph2d-node-registry-init` regenerado (50 crates) | codegen | **conflito provável** → `cargo run -p ph2d-node-sync` |
| cena boot `motion_demo_strobe.rs` (reescrita p/ soft_body+wave, 2 cenas, 8 nós) | shell | módulo Motion |
| `motion_state.rs` + `motion_state_tests.rs` | shell | idem |

Nenhum contrato congelado, nenhum `NodeId`/token/dep novo, nenhum helper compartilhado (ambas as crates são
100% self-contained — nem `hash.rs`/`trig.rs`, só aritmética + `sqrt`). Machete verde. Colunas de estado de
stream novas (`sb_vel`, `wave_h`/`wave_prev`) são **locais, não símbolos**.

## 5. O que fica (doc 21 §5) — simulação + M3 em andamento

- **Simulação (continuação):** `motion.pin_constraint` (fixa instâncias arbitrárias de um stream simulado —
  **precisa de um port de cadeia de constraints no sim**, à la `motion.integrate.forces`; DEFERIDO até esse
  design) · **spatial hash** no boids (só-perf, O(N²)→O(N)) · colisão entre corpos.
- **Distribuições (M3):** `motion.distribute-voronoi` (Lloyd) · `-path` (vector.*) · `motion.lattice`.
- **Deformers (M3):** `motion.four-point-warp` · `-slit-scan`.
- Straggler do M2: `motion.delay` (eco/time-shift puro).

> M4.1 (rope+boids) + M4.2 (soft-body+wave) cobrem as duas metades da simulação: **partículas discretas** e
> **mídia contínua**, todas no substrato `pre`/`sim_t`. O que resta é constraint composável (pin), colisão,
> a otimização de vizinhança, e as distribuições/deformers avançados do M3.
