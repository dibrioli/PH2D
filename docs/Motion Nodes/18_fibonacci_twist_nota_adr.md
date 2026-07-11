# 18 — Nota-ADR: Fibonacci + Twist (abertura do M3 — distribuições + deformers)

**Data:** 2026-07-11 · **Linha:** `line/motion-value` · **Status:** fatia M3.1 implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates. **Contratos congelados intocados** (gate
`architecture_contract_surface` verde: `NodeOp`=2 / `OpResolver`=1 / `NodeManifest`=8). Com o
vocabulário de VALOR completo (docs 12–17), esta fatia abre o **M3** (doc 01 §3): a primeira
**distribuição** e o primeiro **deformer**, provando o pipeline *gerar → deformar*.

---

## 1. O problema (doc 01 §3 — M3)

O M0–M2 deu geradores simples (`motion.grid`), behaviours, forças e o domínio de valor inteiro. O M3
é o salto de **riqueza geométrica**: distribuições generativas (phyllotaxis, blue-noise, Voronoi) e
**deformers** (bend/twist/morph). Faltava abrir esse capítulo — um gerador de layout rico e um deformer
que o reesculpe, para estabelecer o pipeline e a estética.

## 2. A pesquisa do padrão-ouro (antes de codar — DIRETIVA §1)

**`motion.fibonacci` (a distribuição):** o **Vogel model** (1979) é o padrão-ouro do layout
"regular-mas-orgânico": semente `i` em ângulo `i·θ` e raio `c·√i`, com `θ` = **golden angle** (~137.5°).
Vereditos:

- **`√i` no raio** dá **área constante por semente** (empacotamento uniforme) — a razão de o girassol
  não ter buracos nem aglomerados. O **golden angle** é o ângulo "mais irracional" (`360/φ²`): nenhuma
  fração racional dele fecha, então as sementes nunca se alinham em raios/spokes (o motivo profundo do
  padrão). Expus `angle` como param (default golden) — **nudge e a espiral re-empacota em spokes
  visíveis**, um knob didático.
- **Source node** (sem input, minta `P`), como `motion.grid`. Count capado (`param_as_count`) como o
  grid. Trig transcendental-free: reusei o `cos_sin_cycles` (parábola corrigida Capens/devmaster) do
  `motion.orbit`/`oscillator` (copiado por-crate, leaf); `√i` é IEEE-determinístico (HR-5).

**`motion.twist` (o deformer):** o **Twist** clássico (Maya/Blender/Houdini/Cinema4D). Vereditos:

- **Rotação FUNÇÃO da posição** (não rígida): o centro fica parado, a borda gira o `angle` cheio —
  `θ_i = angle · (r_i / r_max)`. Normalizar pelo **raio da borda do frame** deixa o twist
  scale-independente (a borda sempre recebe `angle`). É o 1º transform NÃO-rígido do módulo (distinto do
  `motion.rotate`, uma volta uniforme). **Preserva o raio** (é uma rotação em torno do pivô) — a
  propriedade definidora, e o que o teste falsifica contra "é só um rotate".
- **A força é um INPUT de valor** (`amount`), não um param — assim um `value.lfo` a ANIMA (o domínio de
  valor dos docs 12–17 dirigindo um deformer do M3). `amount` desconectado → **1.0** (twist estático
  cheio, pra um Twist pelado ser visível); length-1 broadcast; length-N per-element. Falloff-masked
  como toda a família. `Pure` (a animação vem do `amount`, não do playhead). Trig via `cos_sin_cycles`.

## 3. O que foi adicionado (fatia M3.1)

**`ph2d-node-motion-fibonacci` (drop-crate, o GERADOR):** `() → stream`. Minta `count` sementes na
espiral de Vogel (`r = spacing·√i`, ângulo `i·angle`). Params: `count`/`spacing`/`angle` (default
golden). `Effect::Pure`. `NodeUiCategory::Source`. Display "Fibonacci Spiral".

**`ph2d-node-motion-twist` (drop-crate, o DEFORMER):** `(stream, amount?) → stream`. Rotaciona cada
elemento em torno do pivô por `angle · amount_i · (r_i/r_max)` — borda gira, centro fica; preserva raio;
falloff-masked. `amount` desconectado → 1.0. `Effect::Pure`. `NodeUiCategory::Transform`. Display "Twist".

**Cena boot — um GIRASSOL QUE TORCE** (`motion_demo_strobe.rs`, ~8 nós):

```
fibonacci → twist → tint → drive_size → output
            lfo ─────┘ (amount)
fibonacci → instance_field(Ramp) → size_range → drive_size.value
```

- `fibonacci` (180 sementes, golden angle) → a espiral de girassol.
- `twist` (angle 200°) reesculpe a espiral; o `amount` é um `value.lfo` lento (0..1) → a espiral
  **enrola e desenrola** no tempo (o domínio de valor animando o deformer).
- `instance_field(Ramp) → size_range → drive_size` dimensiona as sementes por índice (pequenas no
  centro, grandes na borda — um girassol de verdade).

O pipeline **gerar → deformar** do M3 na tela, com o domínio de valor dirigindo a animação. A cena é uma
**função pura do playhead** (o lfo é Temporal; nada guarda estado `pre`) → sem self-loops.

**Testes (13 unit + 3 integração):** fibonacci (6: raio ∝ √i, golden angle entre sementes, count 0
vazio, através do cook, + trig anchors/unit-circle — falsificados); twist (7: preserva raio, borda
gira mais que o interior [vs rotate], amount escala/zero=identidade, falloff mascara, + trig — falsif.).
Integração no shell: `the_fibonacci_lays_out_a_phyllotaxis_spiral` (**raio 0 no centro, `c·√(N-1)` na
borda, cresce por quartis** — grid/uniforme não cresceria) · `the_twist_coils_the_spiral_over_time`
(**a borda varre um arco amplo E preserva o raio** — dead-twist não varre, scale não preservaria).
**Reescrevi o loop-replay do doc 11** pra construir seu PRÓPRIO doc sequencial (`pulse.beat` →
`motion.strobe`) em vez de depender da cena boot — a cena atual é playhead-pura (sem estado `pre`), então
o teste ficava vacuoso; agora é robusto a qualquer cena boot futura.

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-motion-fibonacci`, tipo `motion.fibonacci` | nova | nome novo |
| crate `ph2d-node-motion-twist`, tipo `motion.twist` | nova | nome novo |
| `trig.rs` (copiado de `motion.orbit`, `pub(crate) cos_sin_cycles`) em cada crate | local | nenhum (leaf) |
| `ph2d-node-registry-init` regenerado (42 crates) | codegen | **conflito provável** → `cargo run -p ph2d-node-sync` |
| cena boot `motion_demo_strobe.rs` (reescrita p/ o girassol, ~8 nós) | shell | módulo Motion |
| `motion_state.rs` + `motion_state_tests.rs` + `render_loop/motion_bridge_tests.rs` (loop-replay agora com doc próprio) | shell | idem |

Nenhum contrato congelado, nenhum `NodeId`/token/dep novo. As crates novas só dependem de
`ph2d-nodegraph` + `ph2d-node-registry` (machete verde).

## 5. O que fica (M3, doc 01 §3)

- **Distribuições:** `motion.distribute-poisson` (Bridson blue-noise) · `-voronoi` (Lloyd) ·
  `-path` (integra vector.*) · `motion.lattice`.
- **Deformers:** `motion.bend` · `-four-point-warp` · `motion.morph` (vertex/switch/crossfade) ·
  `motion.look-at` · `-slit-scan`.
- **Sim/agentes:** `motion.boids` (spatial hash) · `-verlet-rope` · `-soft-body` (XPBD) ·
  `-pin-constraint`.
- Straggler do M2: `motion.delay` (eco/time-shift puro).

> Com Fibonacci + Twist o **M3 está aberto**: a primeira distribuição rica e o primeiro deformer, com o
> domínio de valor (docs 12–17) já pronto pra animá-los. O padrão *gerar → deformar (dirigido por valor)*
> se repete pra todo o resto do M3.
