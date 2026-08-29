# 92 — O que o Mini Cavalry tem e nós não

> **Varredura pedida pelo Enio em 2026-08-28:** *"faça varredura e descubra os nós do app
> `/home/enio/Documentos/Recursos/Nodes/MiniCavalryV2` que ainda não temos e que vale a pena ter."*
>
> ⚠️ **Isto é uma LISTA DE ACHADOS, não um plano.** Cada item diz o que falta e o que ele compra;
> o que se constrói e em que ordem é decisão do Enio.

## O método (e o que ele não prova)

Duas fontes, as duas **derivadas**, nunca de memória:

- **Deles:** `DOCS/_NODES.md` — o catálogo curado do próprio autor, **88 nós** com uma linha de
  função cada. (A pasta `src/nodes/` tem ~134 arquivos; a diferença são nós que o autor não pôs
  no catálogo — e o catálogo *é* o filtro de «vale a pena» já aplicado por quem o escreveu.)
- **Nosso:** todo `NodeTypeId::of("…")` da árvore, menos as fixturas de teste — **~130 tipos**.

⚠️ **Nome igual não é capacidade igual, e nome diferente não é capacidade diferente.** Duas
verificações desta varredura mudaram de lado ao serem medidas em vez de adivinhadas:

- **`pivot` parecia faltar e NÃO falta** — o `motion.rotate` tem `Pivot X`/`Pivot Y` e o
  `motion.scale` tem `Scale About`. Uma leitura por nome de arquivo teria publicado um item falso.
- **`bloom`/`blur`/`levels` pareciam faltar e a CAPACIDADE existe** — só não pelo grafo (§1).

---

## §1 ⭐⭐⭐ O buraco maior não é um nó: é uma PONTE

**Medido:** o app tem **24 ajustes de imagem** (`AdjustmentKind` em `ph2d-painter-effects`) —
`Bloom` · `GaussianBlur` · `MotionBlur` · `Levels` · `Curves` · `BrightnessContrast` ·
`HueSaturationBrightness` · `Invert` · `Threshold` · `Posterize` · `Halftone` · `GradientMap` ·
`ChromaticAberration` · `ColorBalance` · `SelectiveColor` · `ChannelMixer` · `ColorLookupLut` ·
`PhotoFilter` · `BlackAndWhite` · `Vibrance` · `Exposure` · `ShadowsHighlights` · `Sharpen` ·
`Noise`.

**E o grafo alcança TRÊS:** `fx.glow`, `fx.drop_shadow`, `fx.rgb_split`.

O Mini Cavalry tem 15 nós de filtro. A leitura errada desta linha é *"faltam 15 nós"*. A leitura
certa é: **o motor já existe do outro lado do app, e o que falta é ele ser alcançável de um nó** —
um `fx.adjust` que aplica um `AdjustmentKind` autorado é **um** nó que abre os 24.

> É a mesma forma do report que abriu o `value.number`: *exprimível não é alcançável*. Aqui é
> ainda mais barato, porque nem a expressão precisa de ser construída.

⚠️ **A cerca a medir antes de construir:** os três `fx.*` que existem são **por-instância** e
compõem no passe do device; um ajuste do Painter opera sobre um **raster**. Antes de desenhar,
meça em qual dos dois substratos o `fx.adjust` vive — o `fx.glow` já paga um assador de tiles por
quadro, e a resposta pode ser que a ponte é para *aquele* assador, não para o `AdjustmentKind`.

**Único filtro deles que não existe em lado nenhum:** `vignette` (escurecer por distância radial).

---

## §2 ⭐⭐ Os que faltam de verdade, e o que cada um compra

| # | Nó deles | O que ele faz | Por que vale |
|---|---|---|---|
| 1 | **`lsystem`** | Reescreve um axioma N vezes por regras e interpreta como *turtle* 2D | Nada no nosso catálogo gera **estrutura recursiva**. Árvores, samambaias, raios, corais, ramificação — hoje só à mão |
| 2 | **`dataSource`** | Parseia CSV/JSON e emite instâncias com `x, y, scale, rotation, color` | **Zero entrada de dado externo** no grafo. É a porta de gráficos, dashboards, data-viz e motion dirigido por planilha |
| 3 | **`celAnimation`** | Alterna entre 2–8 entradas como quadros a um FPS | Temos o **Flip** (um módulo inteiro de animação 2D) e **nenhum nó** que alterne streams. É o idioma de grafo do que o Flip já faz |
| 4 | **`proceduralWalker`** | Personagem que anda até um alvo, com pernas por IK e *gait* alternado | Temos **todas** as peças (`rig.skeleton`, `rig.fabrik`, `rig.ik_2bone`, `rig.rubber_hose`, `rig.skin_deformer`) e **não** a composição que as faz um personagem |
| 5 | **`loopSequencer`** | *Step sequencer* de 4/8/16 passos, emitindo `pulse` + `value` | Temos `pulse.beat` (um metrônomo) mas nenhum **padrão autorado** de batidas. Casa com o módulo de áudio |
| 6 | **`stateMachine`** | FSM de N estados que avança a cada `pulse` (cycle/bounce/random) | Existe uma máquina de estados no **Vector** (morph), que é outro subsistema. No grafo não há nenhuma |
| 7 | **`autoAnimate`** | Detecta mudança e faz *tween* automático até o valor novo | Parcial: o `motion.spring` já suaviza um canal. A diferença é *duração + easing* contra *mola* — meça se o spring já basta antes de construir |
| 8 | **`skeletonRender`** | Bones viram retângulos, joints viram círculos | Pequeno, e sem ele **um esqueleto é invisível** — o que torna os cinco `rig.*` difíceis de autorar |
| 9 | **`snapGrid`** | Força `x`/`y`/`rotation` a múltiplos de uma grade | Existe *snap* no editor; **não** como nó, então não entra numa composição |
| 10 | **`shapeEdges`** | Distribui N pontos no **perímetro** de cada forma de entrada | ⏳ **Confira antes:** o `motion.distribute_curve` distribui ao longo de uma curva e o `field.shape` usa a geometria como campo — pode já estar coberto por composição |
| 11 | **`vignette`** | Escurece a cor por distância radial ao centro | O único filtro deles ausente **dos dois lados** do nosso app (§1) |

---

## §3 ✅ O que parece faltar e NÃO falta

⛔ **Não construa nada desta lista sem medir de novo** — cada linha foi verificada nesta varredura.

| Deles | Nosso |
|---|---|
| `pivot` | `motion.rotate::Pivot X/Y` + `motion.scale::Scale About` |
| `modulate` (i mod N) | `value.pattern` (slots que ciclam por índice) |
| `constant` | `value.number` (nasceu em 2026-08-27) |
| `boolean` | `value.number` no modo **Boolean** |
| `hsvColor` | o *color picker* + `motion.tint` |
| `combineStreams` | `motion.combine` |
| `collisionEvent` | `sim.collide` + os sinais do `ph2d-runtime` |
| `attributeRemap` · `clamp` | `field.remap` · `value.map_range` |
| `sampleGradient` · `numberRangeToColor` | `motion.color_ramp` |
| `vectorField` · os 6 `*Field` | `force.attractor/buoyancy/curl/drag/vortex/wind` + a família `field.*` |
| `skeleton` · `ik2bone` · `ikFabrik` · `rubberHose` · `skinDeformer` | `rig.*` (os cinco) |
| `followTarget` (Tentacle) | `rig.fabrik` + `motion.verlet_rope` por composição |
| `group` | os *subgraph cards* (doc 57) |
| `render` | `motion.output` |
| todo o resto dos 88 | ✅ |

---

## §4 O outro lado da comparação

⚠️ **Uma varredura que só lista o que falta mede metade.** O catálogo deles tem 88; o nosso tem
~130, e a diferença não é gordura — é o que este módulo construiu e eles não têm:

`motion.bezier_warp` (fronteira curva por patch de Coons) · a pilha `sim.*` inteira residente em
**GPU** · `motion.trail` que **re-cozinha** o passado em vez de o lembrar · `motion.emitter` com
`Emitter Motion` · a família `field.*` (box, combine, index_range, radial_sweep, remap, shape) ·
`motion.kaleidoscope` · `motion.spherize` · `motion.sub_uv` · `motion.strobe` · `motion.sort` ·
`motion.cull` · `motion.proximity` · `motion.velocity` · `motion.luminance` · `motion.spline_wrap` ·
`motion.orbit` · `pulse.adsr` · `value.percentile/median/smooth/quantize/normalize/reduce` ·
`motion.integrate` com sub-passos declarados · o `motion.expression`.

---

## ⛔ Recusas MEDIDAS (nesta varredura)

| Item | Motivo |
|---|---|
| `pivot` como nó novo | **Já existe** em `motion.rotate` e `motion.scale` — verificado nos hints, não no nome |
| `modulate` como nó novo | **Já existe** como `value.pattern` |
| `bloom`/`blur`/`levels`/… como 15 nós | A capacidade existe (24 `AdjustmentKind`); o que falta é **uma ponte**, não quinze nós (§1) |
