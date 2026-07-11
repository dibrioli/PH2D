# 28 — Nota-ADR: Distribute Curve + Spline Wrap (M3 — a família CURVA, self-contained)

**Data:** 2026-07-11 · **Linha:** `line/motion-value` · **Status:** fatia implementada, gates verdes.
**Escopo:** **fan-out aditivo (caminho A)** — 2 drop-crates. **Contratos congelados intocados** (gate
`architecture_contract_surface` verde: 2/1/8). Adiciona a **distribuição por curva** (`motion.distribute_
curve`) e o **deformer por curva** (`motion.spline_wrap`) — a última grande família que faltava, **mantida
self-contained** (a curva é authored nos params do próprio nó, sem tocar o módulo vetor).

---

## 1. O problema

Tínhamos distribuições retangular/polar/espiral/blue-noise/hex/CVT, mas **nenhuma por CURVA** — colocar
elementos ao longo de um caminho é o pão-com-manteiga do motion graphics (texto num arco, dots fluindo numa
trilha). E tínhamos deformers de rotação/arc/corner-pin/lente, mas nenhum que **enverga um layout sobre uma
curva arbitrária** (só o `bend`, que é um arco *circular*). O `distribute-path` estava deferido "porque integra
o módulo vetor" — mas isso conflava duas coisas: **ler uma curva do documento vetorial** (cross-module, ainda
deferido) vs **uma curva authored no nó** (self-contained). Esta fatia faz a segunda, preservando a história
"linha auto-contida, integra como bloco único".

## 2. A pesquisa do padrão-ouro (antes de codar — DIRETIVA §1)

**`motion.distribute_curve` (colocar ao longo da curva):** o padrão-ouro é o **"Curve to Points" do Blender** —
distribui N pontos **uniformemente por comprimento de arco** ao longo da spline, e emite a **tangente
normalizada** por ponto. Veredito:

- Uma **Bézier cúbica** authored por 4 control points (params). A amostragem-chave é a **reparametrização por
  arc-length**: amostrar em `t` uniforme *amontoa* nas curvas apertadas; então construímos uma **LUT de
  comprimento acumulado** sobre `t` e a invertemos pra achar o `t` numa fração de arco `s`. `count` pontos
  cell-centered (`(i+0.5)/count`), um `offset` value input que desliza (wrap) → um `value.lfo` faz um marquee
  fluir. HR-5: curva/derivada polinomiais + `sqrt` só nas cordas da LUT, **sem trig**. `Effect::Pure`, Source.
  Testado por: **espaçamento uniforme numa reta** (chords iguais — falsifica amostrar em `t`) · pontos ON the
  curve · offset desliza · count exato.

**`motion.spline_wrap` (envergar sobre a curva):** o padrão-ouro é o **"Spline Wrap" do Cinema 4D** — deforma
E move um objeto ao longo de uma spline (offset/size/twist). Veredito:

- O `x` de cada elemento é normalizado sobre a bbox do layout → `u ∈ [0,1]`, deslizado por `offset` (clamp — o
  layout cobre a curva, o endpoint fica no fim), e o **frame** `(B, tangente, normal)` naquele arco coloca o
  elemento em `B + normal · (y · height_scale)` — as linhas seguem a curva, as colunas empilham na normal. Um
  `amount` value input mistura flat → wrapped (desconectado → totalmente wrapped) → um `value.lfo` acha/desfaz.
  Falloff-masked. Mais geral que o `bend` (qualquer curva, não só arco circular). HR-5: polinômios + `sqrt` na
  normalização. `Effect::Pure`, Transform. Testado por: **amount 0 = identidade** · linha reta na curva reta
  fica reta · **linha reta numa curva enverga** (cross-product ≠ 0; a S antissimétrica falsificava no meio, uso
  um ARCH simétrico) · falloff mascara · cook copia colunas + enverga P.

**A máquina compartilhada** (`curve.rs`, copiado por-crate — convenção leaf): `eval`/`tangent` (Bernstein),
`arc_lut` (LUT de comprimento), `t_at_arclen` (inversão), `frame_at` (ponto+tangente+normal).

## 3. O que foi adicionado (fatia)

**`ph2d-node-motion-distribute-curve` (drop-crate, COLOCAR na curva):** `(offset?) → out`. `count` pontos por
arc-length numa Bézier de 4 params, `offset` desliza (wrap). `Pure`, Source, `curve.rs`. Display "Curve Points".

**`ph2d-node-motion-spline-wrap` (drop-crate, ENVERGAR na curva):** `(in, amount?) → out`. Mapeia bbox-X → arco,
Y → normal; `amount` mistura flat↔wrapped, `height_scale`/`offset` params. `Pure`, Transform, `curve.rs`,
falloff-masked. Display "Spline Wrap".

**Cena boot — DUAS cenas** (`motion_demo_strobe.rs`, 11 nós):

```
ESQUERDA (marquee): distribute_curve → tint(âmbar) → move(−6) → output   lfo(saw)  → offset
DIREITA  (ribbon):  grid → spline_wrap → tint(ciano) → move(+6) → output  lfo(sine) → amount
```

- **marquee** (x≈−6): 24 dots por arc-length na S-curve default; o `offset` (um **saw** `value.lfo`, ramp 0→1)
  faz o marquee **fluir** pela trilha (o saw = `2f−1`, com amp 0.5 + offset 0.5 vira `frac` = ramp limpo).
- **ribbon** (x≈+6): um grid 3×12 (36 dots) **envergado** na S-curve; o `amount` (um **sine** `value.lfo`)
  mistura flat↔wrapped → a fita se enverga na curva e volta.

**Testes (15 unit + 3 integração):** curve.rs (3: endpoints, arc-length uniforme numa reta, frame axis-aligned);
distribute_curve (4: uniforme-na-reta, on-curve, offset-desliza, count-exato/cook); spline_wrap (5: amount-0-
identidade, reta-em-curva-reta-fica-reta, reta-em-curva-enverga, falloff-mascara, cook). Integração no shell:
`the_curve_marquee_flows` (24 dots + viaja >1 + esquerda) · `the_grid_wraps_onto_the_spline` (36 dots + viaja +
**y-extent wrapped ≫ flat** + direita) · `the_default_document_replays_deterministically`.

## 4. Superfície nova (para o handoff de integração)

| Símbolo | Onde | Risco de colisão |
|---|---|---|
| crate `ph2d-node-motion-distribute-curve`, tipo `motion.distribute_curve` | nova | nome novo |
| crate `ph2d-node-motion-spline-wrap`, tipo `motion.spline_wrap` | nova | nome novo |
| `curve.rs` (Bézier + arc-length LUT, copiado idêntico nas 2 crates) | local | nenhum (leaf) |
| `ph2d-node-registry-init` regenerado (62 crates) | codegen | **conflito provável** → `cargo run -p ph2d-node-sync` |
| cena boot `motion_demo_strobe.rs` (reescrita, 2 cenas, 11 nós) | shell | módulo Motion |
| `motion_state.rs` + `motion_state_tests.rs` | shell | idem |

Nenhum contrato congelado, **nenhum acoplamento ao módulo vetor** (curva nos params), nenhum `NodeId`/token/dep
novo (só path crates). Machete verde. **Nota:** `no_tofu_glyphs` pegou 1 seta `→` nova em string literal de
teste (spline_wrap) — trocada por `->`. `curve.rs` marca `tangent`/`frame_at` com `#[allow(dead_code)]` (o
consumidor distribute só usa `eval`/`arc_lut`/`t_at_arclen`; o wrap usa tudo — leaf partial-use).

## 5. O que fica

A família **curva self-contained** fecha. O poço de nós que dependem só de `ph2d-nodegraph` está **esgotado** —
tudo que resta é cross-module ou a fronteira GPU:
- **Distribuição:** `motion.distribute-path` — a versão que lê a curva do **documento vetorial** (`vector.*`),
  distinta desta (authored no nó). DEFERIDA (cross-module; crate satélite que só LÊ o contrato vetor, [memória
  brush-bridge]).
- **Deformer:** `motion.slit-scan` (amostragem temporal; DEFERIDO).
- **Simulação:** `pin_constraint` (port de constraints) · spatial hash · colisão contra bordas · **motor GPU**
  (`docs/plans/2026-07-gpu-resident-node-pipeline.md`).
- Straggler do M2: `motion.delay` (time-scope do editor; DEFERIDO).

> Com a curva, o vocabulário M3 auto-contido está **completo de verdade**: distribuições (retangular, polar,
> espiral, blue-noise, hex, CVT, **curva**), deformers (rotação, arc, corner-pin, lente, **spline-wrap**),
> simetria, empacotamento, estruturais. O próximo passo é **integrar** as 13 fatias, ou cruzar pra outro módulo
> / abrir a linha GPU.
