# Pesquisa 09 — Expressões para ARTISTAS: o estado da arte

> Pedido do Enio (2026-07-28): *"Artistas (alvo dessa engine) não sabem muito sobre
> expressões. Em vez de abrir apenas um input de texto, deveríamos ter um super modal
> com expressões comumente usadas … configuradas na UI … algo parecido com uma
> planilha? … um quadro de previsualização animada … apropriada para artistas e mesmo
> crianças."* · 2ª rodada: *"pode ser Expression mesmo. e precisamos de um catálogo
> muito maior."*

O **plano** que sai desta pesquisa é o [10_plano_editor_de_expressoes.md](10_plano_editor_de_expressoes.md).

---

## §1 — O achado central, e ele é NEGATIVO

> **Ninguém tornou um editor de TEXTO amigável melhorando o editor de texto.**

O After Effects tentou por vinte anos. Na 16.1 (2019) ele ganhou realce de sintaxe,
números de linha, *code folding*, autocomplete, destaque de chaves e erro inline — o
pacote completo de IDE, com cores customizáveis. E a resposta do MERCADO continuou
sendo uma **economia de plugins** (Motion 4, Motion Tools Pro, Excite, Ease and Wizz,
Expressionist, expressCode) cujo produto é literalmente **botões que escrevem a
expressão por você**.

Um editor melhor não converteu ninguém. Um **catálogo com knobs** converteu todo mundo.

E quatro produtos chegaram nisso de forma **independente**:

| produto | a resposta | ano | forma |
|---|---|---|---|
| Apple **Motion** | **Behaviors** | 2005 | ~45 comportamentos em 4 famílias |
| **Cavalry** | **Behaviours** | 2020 | 78 nós, empilháveis, drag-and-drop |
| **Blender** | **F-Curve Modifiers** + **Drivers** | — | 7 modificadores EM PILHA sobre a curva |
| **Rive** | **Data Binding converters** | 2025 | conversores encadeáveis (`Formula`, `Range Mapper`) |

⚠️ **O Blender é o mais próximo do que precisamos** e quase ninguém o cita nessa
conversa: os *F-Curve Modifiers* são **uma pilha de modificadores sobre uma faixa de
animação** — exatamente a nossa situação. Não é um sistema de nós, não é um editor de
texto: é uma **lista de linhas sobre a curva**.

---

## §2 — Quatro coisas TENTADAS e abandonadas → quatro leis

1. **AE *Expression Presets***. Existem há duas décadas e quase ninguém usa. O motivo é
   estrutural: um preset **larga texto opaco** que não se reconfigura sem lê-lo.
   ⇒ **Lei 1 — preset sem KNOB é beco sem saída.** O artista não quer "wiggle"; quer
   "wiggle *mais devagar*".

2. **Notion Formulas 1.0 → 2.0**: eles **reescreveram** o editor porque texto de uma
   linha com `prop("X")` era ilegível. A 2.0 trouxe **tokens no lugar de sintaxe**,
   *live preview* do resultado, navegador de funções com **exemplos interativos**,
   multi-linha e comentários. ⇒ **Lei 2 — mostre o VALOR agora; referência é ficha,
   não string.**

3. **Blender DEPRECOU referência direta em Python** nos drivers em favor de
   **Variables**, porque referência dentro do texto quebrava o rastreamento de
   dependência. ⇒ **Lei 3 — referência a outro objeto é LINHA de primeira classe,
   nunca substring.** ⚠️ Isto morde a gente: hoje `Name.prop` é *exatamente* uma
   substring, resolvida por `rsplit_once('.')`.

4. **Toon Boom não tem editor de expressão de fábrica** (a comunidade escreveu o
   `PS_ExpressionEditor`); o **Cavalry põe código como UM nó do catálogo**
   (`JavaScript Deformer`); o **C4D** idem (`Formula`/`Python` effector); o **Rive**
   idem (`Formula` converter). ⇒ **Lei 4 — o texto é a saída de emergência, e ela é um
   ITEM DO CATÁLOGO**, não um modo escondido.

**E uma quinta, do lado de fora:** o **Lottie** só suporta expressões *parcialmente* e
avisa que "uma teia complexa de expressões não sobrevive à exportação".
⇒ **Lei 5 — uma expressão é um passivo de portabilidade.** Um catálogo de receitas
canônicas é, além de UX, uma **superfície fechada e exportável** — o oposto de texto
arbitrário.

---

## §3 — O que o Blender acerta e ninguém copiou

O *Drivers Editor* plota a **CURVA que mapeia entrada→saída**: eixo X = valor do
driver, eixo Y = valor da propriedade. **Ver o MAPEAMENTO** — não o resultado final, a
*relação* — é o que torna um driver compreensível.

Isso vira requisito da nossa preview: **duas vistas**, o quadro animado (*como vai
ficar*) e a tira de curva com a entrada tracejada sob a saída (*o que isto está
fazendo*).

---

## §4 — Os quatro catálogos, lado a lado

### Apple Motion — 45 *behaviors* em 4 famílias
- **Basic Motion** (9): Align To · Fade In/Fade Out · Grow/Shrink · Motion Path · Move
  · Point At · Snap Alignment to Motion · Spin · Throw
- **Parameter** (19): Audio · Average · Clamp · Custom · Exponential · Link ·
  Logarithmic · MIDI · Negate · Oscillate · **Overshoot** · Quantize · Ramp ·
  Randomize · Rate · Reverse · Stop · Track · Wriggle
- **Retiming** (11): Flash Frame · Hold Frame · Loop · Ping Pong · Replay · Reverse ·
  Reverse Loop · Scrub · Set Speed · Strobe · Stutter
- **Simulation** (16): Align to Motion · Attracted To · Attractor · Drag · Drift
  Attracted To · Drift Attractor · Edge Collision · Gravity · Orbit Around · Random
  Motion · Repel · Repel From · Rotational Drag · Spring · Vortex · Wind

### Blender — *F-Curve Modifiers* (a pilha sobre a faixa)
Generator (polinômio) · Built-in Function (sin/cos/tan/sqrt/ln/sinc) · Envelope ·
**Cycles** · Noise · Limits · Stepped Interpolation

### Houdini — CHOPs de movimento
noise · wave · **spring** · jiggle · **lag** · limit · filter · math · function ·
blend · shift · stretch · trim · cycle · speed · constraint lookat · constraint path

### Cinema 4D — MoGraph *Effectors* (16)
Plain · Random · Shader · **Delay** · **Formula** · **Inheritance** · Push Apart ·
ReEffector · **Sound** · Spline · **Step** · **Target** · Time · Volume · Group · Python

### Rive — *converters*
Formula · **Range Mapper** (com clamp / wrap por módulo / reverse / interpolação) ·
Degs↔Rads · ToString · **Converter Group** (encadeamento)

### After Effects — o cânone da linguagem
`wiggle` · `loopIn`/`loopOut` · `valueAtTime` · `velocity` · `linear`/`ease` ·
`random`/`seedRandom` · `posterizeTime` · `length` · `toComp`/`fromComp` ·
`sourceRectAtTime` · `index` · `time` · `value` · `thisComp.layer()` · `effect()` ·
`marker` · **inertial bounce** (idioma, não função)

E os **Expression Controls** — Slider · Angle · Checkbox · Color · Point · Layer ·
**Dropdown Menu** — que são o mecanismo pelo qual um rig do AE ganha *knobs próprios*.

### Restrições (constraints) — a família que o 2D também usa
Blender: Copy Location/Rotation/Scale · Limit Location/Rotation/Scale/**Distance** ·
**Track To** · Damped Track · Locked Track · Stretch To · Floor · Follow Path ·
Child Of · Transformation · Clamp To.
Unity Animation Rigging: **Damped Transform** · Multi-Aim · Multi-Parent ·
Multi-Position · Multi-Rotation · Override Transform · Twist Correction · Two-Bone IK.

---

## §5 — A CONVERGÊNCIA (o que aparece em ≥3 sistemas)

Esta tabela é o esqueleto do nosso catálogo. Uma família que quatro produtos
inventaram separadamente não é gosto — é a forma do problema.

| família | Motion | Blender | Houdini | C4D | Rive | AE |
|---|---|---|---|---|---|---|
| **Oscilar / onda** | Oscillate | Built-in Fn | wave | — | — | `sin` |
| **Ruído / randomizar** | Randomize, Wriggle | Noise | noise | Random | — | `wiggle` |
| **Ligar / copiar** | Link, Track | (constraints) | — | Inheritance | binding | pick whip |
| **Limitar / clamp** | Clamp | Limits | limit | — | clamp | `clamp` |
| **Remapear faixa** | Ramp | Generator | math | Formula | Range Mapper | `linear()` |
| **Quantizar / passo** | Quantize | Stepped | — | Step | — | `posterizeTime` |
| **Mola / overshoot** | Spring, Overshoot | — | spring | Delay | — | inertial bounce |
| **Atraso / arrasto** | Track | — | lag | Delay | — | `valueAtTime` |
| **Apontar / look at** | Point At | Track To | constraintlookat | Target | — | `atan2` idiom |
| **Negar / inverter** | Negate | — | math | — | reverse | `-value` |
| **Média / suavizar** | Average | — | filter | — | — | — |
| **Expo / log** | Exponential, Logarithmic | Built-in Fn | function | — | — | `Math.exp` |
| **Áudio** | Audio | — | — | Sound | — | Convert to Keyframes |
| **Loop / ciclo** | Loop, Ping Pong | Cycles | cycle | — | wrap | `loopOut` |
| **Estrobo / piscar** | Strobe, Flash Frame | — | — | — | — | idiom |
| **Fórmula crua** | Custom | — | — | Formula | Formula | a expressão |

**Leitura:** dezesseis famílias, e **quinze delas cabem na nossa gramática hoje**
(a exceção é mola/overshoot, que quer `exp`). O catálogo grande não é ambição — é
o que a convergência já autoriza.

---

## §6 — "Planilha": a metáfora do Enio, corrigida

A planilha **não** presta pelas células. Presta por três coisas, e as três têm
análogo exato:

| o que a planilha dá | o análogo aqui | quem já faz |
|---|---|---|
| linhas lidas de cima pra baixo | a **pilha** de linhas | Blender F-Modifiers, Cavalry, Motion |
| a *formula bar* mostrando o que a célula É | a **barra de fórmula** | Notion 2.0 |
| a célula mostrando **quanto dá AGORA** | a **coluna de resultado** | Excel `Formula result = 42` |

O `Insert Function` do Excel é o modelo exato de UMA linha: argumentos nomeados,
descrição por argumento, e o resultado vivo embaixo — **validando antes de executar**.

> **A planilha que queremos é uma PILHA de linhas com coluna de resultado.**
> Não células, não grade 2D. Linhas que compõem.

---

## §7 — "Mesmo crianças": o que a literatura diz

Block-based (Scratch/Blockly) ganha das linguagens de texto para iniciantes por três
razões medidas, e nenhuma delas exige blocos:

1. **Elimina sintaxe e digitação** — as duas barreiras nomeadas na pesquisa.
2. **Todos os comandos ficam VISÍVEIS** — descoberta por navegação, não por memória.
3. **As peças só encaixam de forma que faz sentido** — erro por construção impossível.

E há **transferência**: quem começa em blocos vai melhor para texto depois.

⇒ Nosso alvo não é reimplementar blocos. É **entregar as três propriedades** — catálogo
visível, zero sintaxe, e combinações válidas por construção — **com o texto à vista**,
que é o canal de transferência.

---

## §8 — O que NÓS temos, conferido no código (não suposto)

**A gramática** (`ph2d-expr-parse`):
- 1 arg: `sin` `cos` `abs` `sqrt` `floor` `fract` `noise`
- 2 args: `min` `max` · 3 args: `mix`
- `select(cond, a, b)` · `wiggle(freq, amp [, octaves] [, amp_mult])`
  (⚠️ `octaves`/`amp_mult` **têm de ser literais** — dimensionam a árvore desenrolada)
- operadores: `+ - * /` · `< > ==` · `&& ||` · menos unário
- ⚠️ **não existe** `%`, `exp`, `pow`, `ln`, `atan2`, `clamp`, `smoothstep`, `tan`

**Os identificadores** (`ExprBindings`): `time` (o relógio CORTADO da composição) ·
`value` (o valor PRÉ-expressão) · `Name.prop`.

**Os aliases de link** (`PropKind::from_expr_name`) — o vocabulário que o artista de
fato digita:
`x|translationx|tx` · `y|translationy|ty` · `rotation|rot|r` · `scalex|sx` ·
`scaley|sy` · `opacity|alpha|a` · `position|pos|p` · `morph|m`
⚠️ `TimeRemap` **não** é linkável de propósito (é meta-relógio, não valor de cena).

**Onde a expressão MORA:** `NamedClip.expr: BTreeMap<AnimTarget, String>` (por clip,
fonte de lane que FADEIA) e `TargetBinding.expr: Option<String>` (global, transformação
de canal que NÃO fadeia) — ADR-0144/0145/0146.

---

## §9 — Recursos NOSSOS que a indústria não tem e que o catálogo pode explorar

⚠️ Cada um destes é um diferencial real, e nenhum deles existe pronto na concorrência
como *fonte de expressão*:

- **Áudio com análise espectral** (`ph2d-audio-spectral`): o AE exige *Convert Audio
  to Keyframes* (um passe destrutivo que gera uma key por quadro); o C4D tem o *Sound*
  effector. Nós temos FFT, bandas e true-peak **no produto**. Um canal
  `Audio.level` / `Audio.band(n)` seria nativo e contínuo.
- **Física rígida com rapier** (ADR-0131): `Body.vx`, contatos, `impact`.
- **Motion Nodes** (67 nós): `i`/`n` por instância, forças, campos.
- **Sinais de marker** (ADR-0143): eventos desacoplados na régua.
- **Morph vetorial** (`VecMorph`, `ph2d-vec-blend`).

---

## §10 — As dez coisas que o catálogo tem de RECUSAR

⚠️ Cada uma tem **dono no nosso produto**. Oferecê-las no modal seria a **segunda
porta** para um fato que já tem resposta — a doença que este repo mais pagou.

| o artista pede | quem já responde |
|---|---|
| Loop / Cycle / Ping-Pong / Hold de uma **track** | `Track.extrap` (ADR-0143) |
| Easing / suavizar entrada-saída | graph editor · `Interp::BezierW` |
| Retimar / congelar / reverter o tempo de um objeto | `PropKind::TimeRemap` |
| Seguir um caminho | `PropKind::Position` (motion path, ADR-0141) |
| Escalonar N duplicatas (stagger) | Motion Nodes (`i`/`n`) |
| Gravidade / colisão / mola de um CORPO | módulo de física (ADR-0131) |
| Atrair / repelir / vórtice / vento / arrasto | Motion Nodes `force.*` |
| Campo de influência espacial | `field.*` (Vector / Motion) |
| Interpolar entre duas FORMAS | `ph2d-vec-blend` / `VecMorph` |
| Segurar/estrobar um desenho | exposição do Flip (`set_exposure`) |

⚠️ **Mas a busca do modal TEM de responder por elas** — ver o plano §5.4: digitar
"loop" devolve um cartão que diz onde a coisa mora e leva até lá. É a ideia de
descoberta mais forte do desenho inteiro, e ninguém no mercado faz isso.

---

## Fontes

Cavalry: [behaviours](https://cavalry.studio/docs/nodes/behaviours/) ·
[o Houdini do 2D](https://schoolofmotion.com/blog/cavalry-houdini-of-2d-after-effects) ·
[Oscillator](https://docs.cavalry.scenegroup.co/nodes/behaviours/oscillator/).
Apple Motion: [tipos de behavior](https://support.apple.com/guide/motion/intro-to-behavior-types-motn95a79a3e/mac) ·
[Basic](https://support.apple.com/guide/motion/intro-to-basic-motion-behaviors-motnb0826a23/mac) ·
[Parameter](https://support.apple.com/guide/motion/intro-to-parameter-behaviors-motn49eb56eb/mac) ·
[Retiming](https://support.apple.com/guide/motion/intro-to-retiming-behaviors-motn1374420f/mac) ·
[Simulation](https://support.apple.com/guide/motion/intro-to-simulation-behaviors-motn13745a0c/mac) ·
[listagem](https://www.cryan.com/blog/20220126.jsp).
Blender: [Drivers Panel](https://docs.blender.org/manual/en/latest/animation/drivers/drivers_panel.html) ·
[Drivers Editor](https://docs.blender.org/manual/en/2.81/editors/drivers_editor.html) ·
[F-Curve Modifiers](https://docs.blender.org/manual/en/2.91/editors/graph_editor/fcurves/modifiers.html) ·
[Constraints](https://docs.blender.org/manual/en/latest/animation/constraints/index.html).
Houdini: [CHOPs](https://www.tokeru.com/cgwiki/HoudiniChops.html) ·
[Lag](https://www.sidefx.com/docs/houdini/nodes/chop/lag.html) ·
[Noise](https://www.sidefx.com/docs/houdini/nodes/chop/noise.html).
C4D: [Effectors](https://help.maxon.net/c4d/en-us/Content/html/7443.html) ·
[guia MoGraph](https://www.schoolofmotion.com/blog/a-guide-to-cinema-4d-menus-mograph).
Rive: [data binding](https://rive.app/blog/data-binding-in-rive-a-shared-language-for-designers-and-developers) ·
[converters](https://deepwiki.com/rive-app/rive-runtime/7.3-data-converters).
After Effects: [expression language](https://helpx.adobe.com/after-effects/using/expression-language-reference.html) ·
[exemplos](https://helpx.adobe.com/uk/after-effects/using/expression-examples.html) ·
[biblioteca](https://aereference.com/expressions) ·
[editor 16.1](https://ukramedia.com/12-new-features-for-expression-editor-in-after-effects-16-1/) ·
[Expression Controls](https://www.webdew.com/blog/important-expression-controllers-in-after-effects) ·
[áudio](https://www.premiumbeat.com/blog/animate-via-audio-frequencies-adobe-after-effects/) ·
[Dan Ebberts](http://motionscript.com/design-guide/basic-audio.html).
Unity: [Animation Rigging](https://docs.unity3d.com/Packages/com.unity.animation.rigging@1.1/manual/ConstraintComponents.html) ·
[Damped Transform](https://docs.unity3d.com/Packages/com.unity.animation.rigging@1.1/manual/constraints/DampedTransform.html).
Notion: [Formulas 2.0](https://notionmastery.com/everything-new-in-notion-formulas-2-0/) ·
[o editor](https://bensomething.com/notion/formulas/editor).
Excel: [Insert Function](https://support.microsoft.com/en-us/office/insert-function-in-excel-74474114-7c7f-43f5-bec3-096c56e2fb13).
Lottie: [limitações](https://www.elpassion.com/blog/lottie-who-common-mistakes-in-after-effects-animation-for-lottie).
Blocos: [por que ensinar com blocos](https://kb.vex.com/hc/en-us/articles/360048862911-Why-Teach-Programming-with-Blocks) ·
[block-based para crianças](https://www.codingal.com/coding-for-kids/blog/block-based-coding-for-kids/).
UX: [progressive disclosure, NN/g](https://www.nngroup.com/articles/progressive-disclosure/).
Easing: [easings.net](https://easings.net/).
Princípios: [follow-through & overlap](https://blog.cg-wire.com/follow-through-overlapping-action/).
