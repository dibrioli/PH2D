# 20 — Nota-ADR: Bend + Look At (M3.3 — dois deformers)

**Data:** 2026-07-11 · **Linha:** `line/motion-value` · **Status:** fatia M3.3 implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates. **Contratos congelados intocados** (gate
`architecture_contract_surface` verde: `NodeOp`=2 / `OpResolver`=1 / `NodeManifest`=8). Terceira fatia do
M3 (doc 01 §3): dois **deformers** clássicos — o arc-wrap (Bend) e o orient-toward (Look At) — de famílias
diferentes do `twist` (rotação) e do `morph` (interpolação).

---

## 1. O problema (doc 01 §3 — M3)

M3.1 (fibonacci+twist) e M3.2 (scatter+morph) deram duas distribuições e dois deformers (rotacional,
interpolação). Faltavam dois deformers que todo pacote de motion-graphics tem e que os anteriores não
cobrem: **dobrar** um layout num arco (Bend) e **orientar** cada elemento pra um ponto (Look At — talvez
o nó mais usado de todos: setas apontando pro cursor, flores pro sol).

## 2. A pesquisa do padrão-ouro (antes de codar — DIRETIVA §1)

**`motion.bend` (o arc-wrap):** o **Bend** clássico (Maya/Blender/Cinema4D). Veredito:

- **A extensão X mapeia num arco circular** de ângulo total `angle`: a distância along-axis de cada
  elemento vira ângulo de arco, o offset perpendicular vira distância radial. `θ = angle·(dx/x_extent)`,
  `R = x_extent/angle`, ponto → `((R−dy)·sinθ, R·(1−cosθ) + dy·cosθ)`. O centro (dx=0) fica; a borda
  encurva. **Preserva o comprimento de arco** (um elemento a distância `d` do pivô anda um arco de
  comprimento `d`) — a propriedade que o teste falsifica (um bend de 180° fecha as duas pontas no mesmo
  ponto acima do pivô). Normalizar pelo `x_extent` do frame o deixa scale-independente. HR-5: `cos/sin`
  pelo `cos_sin_cycles` parabólico + a constante `π` (literal, não chamada) + `√` determinístico.
- **A força é um input de valor** (`amount`, animável por `value.lfo`; desconectado → 1.0), como o twist.
  Positivo encurva pra cima, negativo pra baixo. Falloff-masked. `Pure`.

**`motion.look_at` (o orient-toward):** o "orient toward point" (AE/Cavalry), Houdini `lookat`. Veredito:

- **Escreve o canal `rot`** de modo que o +x local de cada elemento aponte pro alvo. O **alvo é um input
  de valor** (`target_x`/`target_y`, animável; desconectado → origem `(0,0)`) — um `value.lfo` desliza o
  alvo e o campo inteiro **vira pra seguir**. Per-element (um campo de alvo dá a cada elemento a sua
  mira). `offset` (graus) gira o resultado (180 = aponta pro lado oposto). `Pure`.
- **`atan2` transcendental-free (HR-5):** o heading precisa de `atan2` (proibido). Portei a **aproximação
  racional de Rajan** (`atan(a) ≈ ¼π·a − a·(a−1)·(0.2447 + 0.0663·a)` pra `a∈[0,1]`, dobrada pelos oito
  octantes), **~0.0015 rad (0.09°)** de erro usando só multiply/add/compare — muito abaixo de um pixel de
  orientação. Testada contra os valores VERDADEIROS conhecidos (sem chamar `std::atan2`, pra o teste ficar
  transcendental-free também).

## 3. O que foi adicionado (fatia M3.3)

**`ph2d-node-motion-bend` (drop-crate, o ARC-WRAP):** `(stream, amount?) → stream`. Dobra a extensão X num
arco de `angle·amount`, preservando o comprimento de arco; `amount` value input (desconectado → 1.0),
falloff-masked. `Pure`. `NodeUiCategory::Transform`. Display "Bend".

**`ph2d-node-motion-look-at` (drop-crate, o ORIENT-TOWARD):** `(stream, target_x?, target_y?) → stream`.
Escreve `rot = atan2(target − pos) + offset` (Rajan approx). Alvo value inputs (desconectado → origem).
`Pure`. `NodeUiCategory::Transform`. Display "Look At".

**Cena boot — uma GRADE QUE ENCURVA seguindo um ALVO** (`motion_demo_strobe.rs`, ~7 nós):

```
grid → bend → look_at → tint → output
       │(amount)  │(target_x)
       lfo_bend   lfo_target
```

- `grid` (4×5 quadrados) → `bend` encurva a grade num arco; o `amount` é um `value.lfo` (±1) → a grade
  **encurva pra cima e pra baixo** como uma onda.
- `look_at` aponta o `rot` de cada quadrado pro alvo; o `target_x` é outro `value.lfo` que desliza o alvo
  esquerda↔direita → todos os quadrados **giram pra segui-lo**.

Dois deformers do M3, cada um animado pelo domínio de valor, numa grade legível. Cena playhead-pura.

**Testes (13 unit + 2 integração):** bend (7: linha reta → arco [centro fica, borda encurva/espelha],
comprimento de arco preservado [180° fecha as pontas], amount escala/sinaliza, falloff mascara, + trig —
falsificados); look_at (6: **atan2 approx bate o atan2 verdadeiro** [<0.003 rad], alvo coincidente ≠ NaN,
cada elemento aponta pro alvo [±x opostos], offset gira, alvo em movimento vira a mira, através do cook —
falsificados). Integração no shell: `the_bend_curls_the_grid_over_time` (a quina da borda varre um caminho
amplo; dead-bend a prende) · `the_look_at_aims_each_square_at_the_moving_target` (spread de rotações pelo
grid [posições diferentes → ângulos diferentes] + a mira de um quadrado gira no tempo; dead-look_at deixa
tudo em 0). Determinismo (playhead-puro, atan2/trig determinísticos).

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-motion-bend`, tipo `motion.bend` | nova | nome novo |
| crate `ph2d-node-motion-look-at`, tipo `motion.look_at` | nova | nome novo |
| `trig.rs` (copiado de `motion.orbit`) em bend; `atan2_approx` (inline) em look_at | local | nenhum (leaf) |
| `ph2d-node-registry-init` regenerado (46 crates) | codegen | **conflito provável** → `cargo run -p ph2d-node-sync` |
| cena boot `motion_demo_strobe.rs` (reescrita p/ bend+look_at, ~7 nós) | shell | módulo Motion |
| `motion_state.rs` + `motion_state_tests.rs` | shell | idem |

Nenhum contrato congelado, nenhum `NodeId`/token/dep novo. As crates novas só dependem de
`ph2d-nodegraph` + `ph2d-node-registry` (machete verde).

## 5. O que fica (M3, doc 01 §3)

- **Distribuições:** `motion.distribute-voronoi` (Lloyd) · `-path` (integra vector.*) · `motion.lattice`.
- **Deformers:** `motion.four-point-warp` · `-slit-scan`.
- **Sim/agentes:** `motion.boids` (spatial hash) · `-verlet-rope` · `-soft-body` (XPBD) · `-pin-constraint`.
- Straggler do M2: `motion.delay` (eco/time-shift puro).

> M3.1–M3.3 estabelecem o vocabulário geométrico do M3: distribuições (spiral, blue-noise) + deformers
> (rotação, interpolação, arc-wrap, orient-toward), todos dirigidos por valor. O que resta são as
> distribuições avançadas (voronoi/path) e a família de SIMULAÇÃO (boids/ropes/soft-body) — a próxima
> classe de capacidade (dinâmica emergente, sequencial).
