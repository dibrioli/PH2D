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

**`motion.soft_body` (o corpo mole):** o padrão-ouro é o **shape-matching de Müller et al. — *Meshless
Deformations Based on Shape Matching* (SIGGRAPH 2005)**. **Verifiquei as equações canônicas contra o material
dos próprios autores** (regra "no industrial claims"). Veredito:

- **Shape-matching constraint (2005, verificado idêntico):** acha o **melhor frame** `(M, c)` da forma de
  repouso na nuvem deformada; o **goal** de cada partícula é `gᵢ = M·qᵢ + c` (`qᵢ = xᵢ⁰ − c₀`). `M` é a
  rotação **rígida** `R` = fator polar de `A_pq = Σ pᵢ qᵢᵀ` (`R = A_pq S⁻¹`, `S = √(A_pqᵀA_pq)`), que em 2D
  tem **forma fechada SEM trig**: `(cos,sin) ∝ (A₀₀+A₁₁, A₁₀−A₀₁)`, normalizado — um `sqrt` só (HR-5). Massa
  **uniforme** (`wᵢ=mᵢ=1`, exato p/ a grade par → os pesos do paper somem). Testado por **invariância rígida**
  (o falsificador do sinal do polar).
- **Modo linear (β, 2005 — ADICIONADO nesta auditoria):** o paper também blenda o mapa linear de
  mínimos-quadrados `A = A_pq A_qq⁻¹`, **com preservação de área** `A / √det(A)` (o `A/det(A)^{1/d}`, d=2),
  dando `M = β·A + (1−β)·R` — **squash & stretch** de gelatina (o modo rígido puro não deforma). Param
  `stretch` (β∈[0,1], default 0 = byte-idêntico ao rígido; a cena boot usa 0.3). Testado por
  **segue-o-stretch** (β=1 acompanha um cisalhamento área-preservante; β=0 volta ao repouso) + **preserva
  área** (uma escala uniforme ×3 é normalizada de volta).
- **Integração — divergência HONESTA do paper de 2005:** eu uso a formulação **PBD (Müller et al. 2007 /
  "Ten Minute Physics")** — *predict → project ao goal → derivar velocidade* `v=(x_new−x_old)/dt` — e **NÃO**
  o velocity-blend `v += α(g−x)/h` da eq. 12 do paper de 2005 (o goal aqui vem da nuvem PREVISTA, não da
  atual). Mesma linhagem de autores, é o esquema **moderno-padrão** e incondicionalmente estável — mas o doc
  foi **corrigido** pra atribuir certo (antes dizia "Müller 2005" pra a integração que é 2007). Geometria em
  módulo irmão `shape.rs`.
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
`rows×cols` shape-matching (geometria em `shape.rs`); fileira de cima fixada na âncora animável (`pin`),
gravidade + goal-pull; params `stiffness`/`stretch`(β)/`damping`. Estado `P`+`sb_vel` no `pre`. `Temporal`,
Source, `sqrt` só na polar 2D + preservação de área.

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
  desliza o pino → **balança como gelatina** (`stretch`=0.3 → squash & stretch) e volta à forma.
- **wave** (centro x≈+6): o `drive` (`value.lfo`) oscila a célula central → **ripples concêntricos** incham
  os pontos pra fora.

Duas sims de mídia contínua (um corpo deformável, um campo que propaga), cada uma dirigida pelo domínio de
valor, em regiões separadas do canvas. Pequena e legível (regra do doc 12 / feedback "simplifique"). A
contraparte discreta é o doc 21 (rope+boids).

**Testes (18 unit + 3 integração):** soft_body (11: em `shape.rs` — **invariância rígida** [pose rígida = seu
próprio goal, falsifica o sinal do polar], **recupera a forma** rígida, **modo linear segue o stretch** [β=1
acompanha, β=0 volta ao repouso], **preserva área** [escala ×3 normalizada]; em `lib.rs` — seed da malha na
âncora, gravidade pendura abaixo da fileira fixada, âncora móvel arrasta o corpo, replay determinístico,
sem-loop = malha de repouso, NaN recupera, cook com pre-loop — falsificados); wave (7: seed plano, **fonte
propaga outward** [vs speed 0], **velocidade
finita** [vizinho move no tick 2, célula distante ainda 0], **damping limita e silencia**, replay, sem-loop =
plano, cook emitindo `size` — falsificados). Integração no shell:
`the_soft_body_hangs_and_wobbles_from_the_moving_anchor` (base afunda abaixo do topo fixado + a âncora varre
o corpo) · `the_wave_ripples_outward_from_the_driven_center` (a coluna `size` ganha spread — anéis de pontos
grandes/pequenos — e fica limitada) · `the_default_document_replays_deterministically` (drena o pump
sequencial completo).

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-motion-soft-body`, tipo `motion.soft_body` (+ módulo irmão `shape.rs`) | nova | nome novo |
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
