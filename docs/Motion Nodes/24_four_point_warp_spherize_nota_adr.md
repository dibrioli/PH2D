# 24 — Nota-ADR: Four Point Warp + Spherize (M3 — os deformers que faltavam)

**Data:** 2026-07-11 · **Linha:** `line/motion-value` · **Status:** fatia implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates. **Contratos congelados intocados** (gate
`architecture_contract_surface` verde: `NodeOp`=2 / `OpResolver`=1 / `NodeManifest`=8). Fecha os dois
deformers restantes do M3 (doc 23 §5): o **corner-pin projetivo** e a **distorção radial** — duas famílias
distintas de warp ao lado de twist/morph/bend/look_at.

---

## 1. O problema (doc 01 §3 / doc 23 §5 — deformers)

O M3 tinha rotação (twist), interpolação (morph), arc (bend) e orientação (look_at). Faltavam os dois warps
"de layout" que todo pacote tem: o **corner-pin / keystone de perspectiva** (fixar 4 cantos e o campo entra em
perspectiva — retas continuam retas) e a **distorção de lente** (bulge/pinch radial — a lupa/fisheye).

## 2. A pesquisa do padrão-ouro (antes de codar — DIRETIVA §1)

**`motion.four_point_warp` (o corner-pin):** o padrão-ouro é a **homografia projetiva** do quadrado unitário
pra um quadrilátero (**Heckbert, *Projective Mappings for Image Warping*, 1989** — o "Corner Pin" do AE, o
keystone de projetor). Veredito:

- Cada elemento é normalizado pra `(u,v) ∈ [0,1]²` na bounding box do layout, e mapeado pela **matriz 3×3 `H`**
  cuja imagem dos cantos do quadrado unitário são os 4 cantos-alvo; a **divisão de perspectiva** `(X/W, Y/W)`
  é o que **mantém retas retas** (um patch bilinear entortaria). `H` tem a **forma fechada de Heckbert** (ramo
  afim se o quad é paralelogramo, senão os termos projetivos `g,h`). HR-5: aritmética + divisão, **sem trig,
  sem `sqrt`**. Testado por **os 4 cantos mapeiam EXATO** (a definição — o falsificador do sinal da fórmula) +
  **retas continuam retas** (3 pontos colineares → colineares; falsifica o bilinear).
- Cantos animáveis: cada um tem um param de offset (`tl_dx`…`br_dy`) da esquina da bbox; um **`warp` value
  input** escala tudo `0→1` → um `value.lfo` faz o layout **inflar em perspectiva e achatar**. Falloff-masked.
  `Effect::Pure`.

**`motion.spherize` (a lente):** o padrão-ouro da distorção radial (Photoshop Spherize/Pinch, fisheye).
Veredito:

- Cada elemento desloca ao longo do raio do centroide: `p' = c + (p−c)·scale(r)`, com
  `scale(r) = 1 + amount·(1 − (r/R)²)` dentro do raio `R`, `1` fora. `amount>0` amplia o centro (**bulge**,
  identidade na borda); `amount<0` comprime (**pinch**). O falloff quadrático faz a borda sem costura. HR-5:
  **um `sqrt`** (o raio) + polinômio, **sem trig**. Testado por **bulge empurra pra fora / pinch puxa pra
  dentro**, centro estável (sem NaN), fora do raio intocado, falloff mascara.
- `amount` animável (value input; desconectado → 0.5); `radius` (world) centrado no centroide. Falloff-masked.
  `Effect::Pure`.

## 3. O que foi adicionado (fatia)

**`ph2d-node-motion-four-point-warp` (drop-crate, o CORNER-PIN):** `(in, warp?) → out`. Homografia de Heckbert
(bbox → quad de 4 cantos-offset escalados por `warp`), divisão de perspectiva, falloff-masked. `Effect::Pure`,
`NodeUiCategory::Transform`. Display "Four Point Warp".

**`ph2d-node-motion-spherize` (drop-crate, a LENTE):** `(in, amount?) → out`. Magnificação radial
`(p−c)·(1+amount·(1−(r/R)²))` no raio `R` do centroide, falloff-masked. `Effect::Pure`, Transform. Display
"Spherize".

**Cena boot — DUAS cenas pequenas** (`motion_demo_strobe.rs`, 12 nós = 2 chains lineares):

```
ESQUERDA (perspectiva): grid → four_point_warp → move(−6) → tint(âmbar) → output   lfo → warp
DIREITA  (lente):       grid → spherize        → move(+6) → tint(ciano) → output   lfo → amount
```

- **four_point_warp** (x≈−6): os cantos de cima fixados pra dentro (keystone) + `warp` (`value.lfo` ∈ [0,1])
  → a grade **infla em perspectiva e achata** (retas retas).
- **spherize** (x≈+6): `amount` (`value.lfo` ∈ [−0.6, 0.6]) → a grade **incha (bulge) e afunda (pinch)** como
  lente.

Dois deformers de famílias distintas (warp afim/projetivo vs. radial não-linear), cada um dirigido pelo
domínio de valor, em dois chains lineares legíveis. `motion.move` desloca cada cena (grid nasce na origem).

**Testes (11 unit + 3 integração):** four_point_warp (6: **cantos mapeiam exato** [falsifica a homografia],
**retas continuam retas** [falsifica bilinear], offsets-zero/warp-0 = identidade, canto-segue-o-pin, falloff
mascara, cook); spherize (5: identidade, **bulge empurra / pinch puxa**, centro estável + fora-do-raio
intocado, falloff mascara, cook). Integração no shell: `the_four_point_warp_billows_the_grid` (a grade viaja
no tempo + na esquerda) · `the_spherize_bulges_and_pinches_the_grid` (spread cresce/encolhe + na direita) ·
`the_default_document_replays_deterministically`.

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-motion-four-point-warp`, tipo `motion.four_point_warp` | nova | nome novo |
| crate `ph2d-node-motion-spherize`, tipo `motion.spherize` | nova | nome novo |
| `ph2d-node-registry-init` regenerado (54 crates) | codegen | **conflito provável** → `cargo run -p ph2d-node-sync` |
| cena boot `motion_demo_strobe.rs` (reescrita, 2 cenas, 12 nós) | shell | módulo Motion |
| `motion_state.rs` + `motion_state_tests.rs` | shell | idem |

Nenhum contrato congelado, nenhum `NodeId`/token/dep novo, nenhum helper compartilhado (ambas 100%
self-contained — só aritmética + um `sqrt` no spherize). Machete verde.

## 5. O que fica (doc 23 §5)

Os deformers M3 auto-contidos estão COMPLETOS (twist/morph/bend/look_at/four_point_warp/spherize). O que resta:
- **Distribuição:** `motion.distribute-path` (curva — integra vector.*, cross-module; DEFERIDO).
- **Deformer:** `motion.slit-scan` (amostragem temporal do stream — mais pesado; DEFERIDO).
- **Simulação:** `motion.pin_constraint` (port de cadeia de constraints; DEFERIDO) · spatial hash no boids ·
  colisão · (e o motor GPU — plano em `docs/plans/2026-07-gpu-resident-node-pipeline.md`).
- Straggler do M2: `motion.delay` (eco/time-shift puro).

> Com four_point_warp + spherize, o vocabulário de deformers do M3 fecha nas famílias auto-contidas: rotação,
> interpolação, arc, orientação, corner-pin projetivo e lente radial. O que sobra depende de outros módulos
> (path/vector, slit-scan temporal, pin/constraints) ou é o motor GPU.
