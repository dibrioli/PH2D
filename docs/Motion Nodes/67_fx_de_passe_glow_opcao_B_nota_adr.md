# 67 — FX de passe: o **glow** é do módulo (Opção B), nó `fx.glow` (nota-ADR)

> **Status:** implementado (linha `line/motion-value`, 2026-07-14). O Enio decidiu
> **Opção B** do [doc 66](66_fx_de_passe_a_premissa_do_plano_e_FALSA.md). Contrato
> congelado **intocado** (8/2/1). **92 crates-nó** (+`fx.glow`).

## 1. O que o doc 66 deixou pra decidir, e o que o Enio escolheu

O doc 66 provou que a premissa do plano era falsa (o compositor do Painter é
8-bit e **destrói** o HDR de que o bloom vive) e ofereceu duas formas:

- **A** — pós-processo do frame inteiro (muda **tudo**; é feature de outro plano).
- **B** — o Motion desenha num RT HDR **próprio**, o FX roda ali, compõe de volta.
  Blast radius **zero** fora do Motion.

**Enio escolheu B.** Esta fatia entrega o efeito que **justifica** o B — o **glow**.

## 2. A forma que o B tomou: **glow ADITIVO, byte-idêntico no ponto neutro**

O doc 66 esboçou B como "compõe de volta (premult-over)". Compor a camada inteira
por cima **quebraria o z** (o Motion pularia pra frente de todo sprite quando o FX
ligasse, e sumiria o entrelaçamento com o ECS). O ganho que o próprio doc 66 nomeia
é *"as faíscas brilham, o fundo não"* — e isso é **luz aditiva**, não uma camada por
cima. Então:

- **O core do Motion continua FUNDIDO no `game_rt`** (o passe sprites+Motion de hoje,
  `render_with_extra`, **intocado**). O Motion fica z-ordenado como sempre.
- **Quando há glow autorado**, um passe extra: re-renderiza as instâncias do Motion
  **em isolamento** num `Rgba16Float` só dele → bright-pass + blur → **SOMA** o halo
  sobre o `game_rt` (aditivo, antes do tonemap). Aditivo = luz emitida: sangra sobre
  o que está na frente (correto e **z-agnóstico**).
- **O tonemap fica INTOCADO** (o clamp dele vira o core estourado em branco = a cara
  do bloom). **Não** flipei o `BYPASS_LUT` — isso mudaria o frame inteiro (blast
  radius). O HDR do bloom vive dentro do passe do Motion e é somado já resolvido.

**O ponto neutro é byte-idêntico:** sem nó de glow (ou intensidade 0) o bloco inteiro
é **pulado** — nenhum passe novo, frame idêntico ao de antes da feature. É a mesma
disciplina da rack de áudio (todo efeito é no-op byte-idêntico no ponto neutro) e do
`cooked()` do vetor (`Cow::Borrowed` quando não há raio).

O custo (~2× o trabalho de sprite quando o glow está ligado) é o que o doc 66 previu,
e só se paga com a tool ativa **e** o nó presente.

## 3. A decisão que evitou ~200 linhas de UI cheia de armadilhas: **o glow é um NÓ**

O plano dizia *"o documento declara `layer_fx`"*. **O grafo É o documento** — então o
glow é um **nó** `fx.glow`, e "declarar no documento" fica mais literal, não menos.
Cheguei a construir o caminho alternativo (um `PassFx` no `MotionDoc` + seção `[fx]`
serializada) e o **descartei**: ele exigia uma **seção nova no painel de params**, e é
exatamente aí que este código sangra tempo (o [drift de 1px do clique](../../project-memory/feedback_a_click_is_a_press_that_drifted.md),
[pintado≠populado](../../project-memory/feedback_painted_is_not_populated_paint_gate.md),
um slider que [nenhum teste clica](../../project-memory/feedback_widget_is_done_when_a_test_clicks_it.md)).

O nó compra tudo de graça:

| precisa de | vem de |
|---|---|
| **autoria** | o painel de params **existente** — selecionou o `fx.glow`, arrastou o slider. Zero UI nova. |
| **persistência / undo / param dirigido** | a infra do grafo onde ele já vive (`set_param` = passo de undo normal; o formato textual já serializa). |
| **fonte única dos defaults** | o `MANIFEST` do nó — o leitor do shell lê `override-ou-default`, nunca duplica os números. |

Duas portas pro mesmo estado **divergem** — então **não** deixei o `PassFx` no doc como
2ª fonte. Uma fonte: o nó. (E `fx.glow` é a razão de o `MotionDoc` **não** ter ganhado
`layer_fx`, contra o que o doc 66 §1 antecipava — a §3 dele estava certa: era feature
de fora do doc-model.)

**O nó configura o passe, não o escopa.** É um **passthrough** (`out == in`,
`Effect::Pure`, gate byte-idêntico no cook) — largar na cadeia não muda o stream, e
deixar solto também vale. O shell o acha com `from_graph` e lê os params; o glow aplica
na imagem inteira do Motion **onde quer que o nó esteja**. Posição é legibilidade, nunca
limita o efeito. (É o mesmo espírito do badge de portal ⊙: um nó que documenta um laço
que o grafo não desenha.)

## 4. A costura de isolamento no `ph2d-render` (foundational, projetada pra isolamento)

O Motion é **fundido** no passe de sprites sem tag de origem
([`sprite_collect`](../../crates/ph2d-render/src/sprite_collect.rs)), então "pós-processar
só o Motion" é impossível a jusante. A costura:

- **`SpriteRenderer::render_instances_only`** — renderiza SÓ um slice de instâncias num
  alvo que você possui, **sem** drenar o `PresentWorld`. Não dupliquei o laço de draw:
  extraí o corpo compartilhado (`draw_scratch`) e os **dois** caminhos (cena+extra e
  isolado) passam por ele — **um laço só**, então não podem divergir
  ([[feedback_two_doors_to_the_same_question_diverge]]). O `render_with_extra` só delega;
  os 152 testes do `ph2d-render` seguem verdes (é um *move* mecânico).
- **Módulo novo `motion_fx.rs`** (`MotionFx` + `BloomParams`) — append-only, isolado de
  propósito: um arquivo novo + um método novo + dois exports, **nenhuma** linha de outra
  linha muda. Molde: o `Tonemap` (owns pipeline+bgl+sampler+bind group, triângulo
  fullscreen, WGSL por `include_str!`).

**A cadeia** (`shaders/bloom.wgsl`, sem transcendentais — HR-5):
`prefilter (bright-pass soft-knee, COD/Karis) + downsample p/ ½-res` → **4 iterações
Kawase** ping-pong `a→b→a→b→a` (Gaussiana larga barata) → **composite aditivo** (color
One/One) sobre o `game_rt`. Tudo `Rgba16Float`: o `tint` da instância é `[f32;4]` **sem
clamp** no lowering, então uma faísca `(6,4,2)` some 6× mais luz que uma branca — o excesso
> 1.0 sobrevive ao blur (era exatamente isso que o round-trip 8-bit mataria).

## 5. A demo: `PH2D_MOTION_FX_SMOKE=1`

Uma fileira de faíscas cuja cor rampa **branco → laranja quente** (`motion.tint` em
Gradient). Branco é `1.0` (LDR, mal encosta no threshold); a ponta quente é HDR e brilha
forte. **O halo cresce com o brilho da esquerda pra direita** — a propriedade que *define*
o bloom HDR e a razão de isto ser o RT próprio do Motion, não o compositor de 8 bits.

```text
grid(1×9) → tint(Gradient branco→quente) → scale → fx.glow(intensity 1.3, radius 1.6) → output
```

Selecione o `fx.glow` no painel de params pra arrastar o glow ao vivo; delete o nó e as
faíscas ficam com o **mesmo** brilho, só sem o halo — o ponto neutro é byte-idêntico.

## 6. Superfície

- **Drop-crate nova:** `ph2d-node-fx-glow` (`fx.glow`, passthrough Pure, params
  threshold/knee/intensity/radius + `ParamUiHint`s). **`registry-init` regenerado por
  `cargo run -p ph2d-node-sync` — 92 crates-nó.** Conflito esperado no rebase:
  **gere de novo pelo `node-sync`, nunca resolva à mão.**
- **Foundational `ph2d-render`:** `render_instances_only` + `draw_scratch` (extração,
  um laço só) + módulo `motion_fx.rs` (`MotionFx`/`BloomParams`) + `shaders/bloom.wgsl`.
- **Shell:** `AppGfx.motion_fx` (constrói no `init`, resize ao lado do `game_rt`), o
  **Passe 1c** no `present.rs` (entre o Flip e o tonemap, guardado por
  `fx.glow` presente + `intensity > 0` + tool ativa + instâncias > 0), dep direta na
  crate do nó (o leitor `from_graph`), e a cena `motion_fx_smoke.rs`.
- **Contrato congelado:** intocado (8/2/1 verde). O `NodeManifest` do glow é um nó
  normal — nada de campo novo.
- **Gates:** passthrough byte-idêntico no cook · `from_graph` neutro (sem nó → sem passe)
  e override-sobre-default · `BloomParams` (curva soft-knee, knee 0 não divide por zero) ·
  **pipeline válido em device real** (o guard da tela-preta: shader compila, layouts
  batem, bind groups válidos).
- **Aberto (Opção A, dono de outro plano):** vignette / levels / hue **não** entram aqui —
  são grades do **frame inteiro** (aplicá-las "só na camada Motion" não é operação real, e
  forçá-las quebraria o z). São a Opção A do doc 66 — um *post stack* do app, que merece o
  ADR próprio (*"o PH2D tem um post stack"*). O **blur/vignette** que o plano listava junto
  do glow segue essa mesma porta. Também aberto: um **readback** headless que prove o glow
  *dispara* por valor (hoje o "dispara" é o smoke; o gate GPU prova a *validade* do pipeline).
