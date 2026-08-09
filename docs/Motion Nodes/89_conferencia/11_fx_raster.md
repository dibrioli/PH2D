# 89 · Família 11 — **FX (raster)** · 3 nós

**Nós:** `fx.drop_shadow` · `fx.glow` · `fx.rgb_split` · **20 params** (6 · 9 · 5)
**Data:** 2026-08-09 · **Agente:** conferência família 11 · **Plano:** [89](../89_plano_conferencia_dos_nos.md) §3/§4
**Referência autoritativa:** [`referencia_pesquisa_cavalry.md`](../referencia_pesquisa_cavalry.md) §A.4 (os 54 filtros) · AE/Photoshop/Unity/Unreal · primeira-parte: `ph2d-painter-effects::AdjustmentKind` (24) e `ph2d-ecs::vec_filter_kinds::SPECS` (15)

---

## §0 — A NAVALHA que organiza a família inteira (ler antes da tabela)

Esta família tem **três nós e DUAS arquiteturas**, e confundi-las é o erro que uma varredura
ingênua comete:

| | `fx.drop_shadow` · `fx.rgb_split` | `fx.glow` |
|---|---|---|
| O que é | **FX de STREAM** — duplica linhas, desloca, recolore, emite `3n`/`2n` instâncias | **FX de PASSE** — `passthrough` byte-idêntico (`out == in`) que **configura** um passe de render |
| Onde o efeito acontece | no `Stream`, no cook | em `ph2d_render::MotionFx` (RT `Rgba16Float` próprio, `render_instances_only`) |
| Params são | geometria+cor de cópias-fantasma | os knobs de um bloom HDR |
| `lowerings` | `Cpu` (os dois) | `Cpu` (o nó não computa nada) |

**E há uma LEI, derivada de três documentos + um revert, que decide o que pode ou não virar
`fx.*` no grafo — ela vale mais que qualquer linha da tabela:**

> **O passe do Motion compõe ADITIVAMENTE** (`motion_fx.rs:265-275`: `src_factor: One`,
> `dst_factor: One`, `BlendOperation::Add`). Isso não é detalhe de implementação — é a **razão**
> de o glow poder ser um nó: o core do Motion continua FUNDIDO no `game_rt` junto com os sprites
> do ECS, sem tag de origem ([doc 67 §2](../67_fx_de_passe_glow_opcao_B_nota_adr.md)), então a
> única coisa que se pode fazer "só com o Motion" a jusante é **SOMAR luz**. Um FX que
> **escurece, substitui ou remapeia** os pixels que já estão lá exigiria compor a camada Motion
> por cima — e isso **quebra o z** (doc 67 §2, textual).

⇒ **A regra de triagem desta família, em uma linha:**
**aditivo ⇒ pode ser nó `fx.*` hoje** (a maquinaria da Opção B já existe e está shipada) ·
**subtrativo/remapeador ⇒ é o módulo de PÓS-PRODUÇÃO que o Enio anunciou** (cerca C4 abaixo).

Isto **não é opinião minha**: é a mesma frase que o commit do post-stack usou para justificar por
que o glow foi Opção B e a vinheta seria Opção A (`9a36d4a27`: *"o glow pode ser um no porque
bloom e luz ADITIVA z-agnostica; a vinheta e ancorada na moldura e subtrativa"*), e é ela que
explica por que a vinheta foi **construída e removida** dias depois.

---

## §1 — A TABELA, nó a nó

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `fx.drop_shadow` | 6 (`direction`·`distance`·`r`·`g`·`b`·`a`) | **Softness / Size** — o borrão da sombra. AE *Drop Shadow*: `Softness`; Photoshop layer style: `Size`; **primeira-parte: o nosso próprio `ph2d-ecs::vec_filter_kinds` "Drop Shadow" tem `Radius`** (o módulo Vector shipou a sombra MACIA) | **PARCIAL, e degradado — tentado:** encadear `fx.drop_shadow → fx.drop_shadow` funciona estruturalmente (a 2ª passada sombreia as linhas da 1ª, e todas as sombras continuam ATRÁS: `[s(s₁)]++[s(e)]++[s₁]++[e]`), dando **4 taps** direcionais; com 3 encadeados, 8 taps. ⚠️ **Mas:** (a) é um *smear* discreto ao longo de UMA direção, não um borrão — não alarga perpendicular; (b) o alfa **compõe** (`a = color.a · base.a`, então a 2ª ordem é `0,35² = 0,1225`), o que não é o perfil de uma gaussiana; (c) a contagem é `2ⁿ` e o `MAX_INSTANCES = 65_536` corta em **n ≤ 16.384** com 2 encadeados e **n ≤ 8.192** com 3 | **omissão**, com **fence** (C1) | **P1** | `softness = 0` ⇒ a sombra hard-edged de hoje, ao bit |
| `fx.drop_shadow` | idem | **Blend Mode da sombra** — Photoshop layer style abre com **Multiply** (o default dele); AE *Glow Operation* é o irmão disso do outro lado | **NÃO** — não existe coluna nem canal de blend por-instância; todo instance de Motion desenha com o *over* padrão (`premultiplied = 0`, doc 38 §2). Duas sombras sobrepostas hoje **não escurecem** uma à outra | **omissão** (custo: uma coluna de convenção de stream, o padrão do `texture_id`) | **P2** | modo `Over` ⇒ o de hoje |
| `fx.drop_shadow` | idem | **Spread / Choke** (Photoshop) — estrangula a matte borrada | **NÃO** — é operação de MATTE (raster), e sem `Softness` não há matte a estrangular: depende do item acima | **natureza** (raster puro) enquanto o FX for de stream | ⛔ | — |
| `fx.drop_shadow` | idem | **Shadow Only** (AE, `Shadow Only` checkbox) | **SIM, e a cadeia FUNCIONA:** as sombras saem **PRIMEIRO** no bloco (doc 38 §1) e `motion.cull` em modo **Fraction** *"keeps the first `amount·n` elements"* ⇒ **`fx.drop_shadow → motion.cull(Fraction, amount = 0.5)`** é exatamente *Shadow Only* | — | ⛔ **refutado** | — |
| `fx.drop_shadow` | idem | **Use Global Light** (Photoshop) — um ângulo de luz do documento que todas as sombras herdam | **SIM** — um param é uma ARESTA (doc 58): um `debug.const` (ou `value.*`) dirigindo o `direction` de todos os `fx.drop_shadow` **é** a luz global, e mais geral que a do Photoshop (pode ser animada/dirigida) | — | ⛔ **refutado** | — |
| `fx.drop_shadow` | idem | **`ParamUnit` ausente:** `distance` é comprimento de MUNDO e não declara `ParamUnit::Length`; `direction` é `Angle` no widget mas não na unidade — doc 88 lei #1 (*a unidade é o que o número É*) | **N/A** (não é gap de capacidade, é de metadado) | **omissão** — a família `fx.*` tem **ZERO** dos quatro canais de side-metadata (`register_param_units` · `_hard_max` · `_hard_min` · `_sections`); confirmado por grep nas 3 crates | **P1** (barato, e a fronteira de display já existe) | declarar a unidade **não muda um número** — só como ele é mostrado |
| `fx.glow` | 9 (`threshold`·`knee`·`intensity`·`radius`·`saturation`·`tint_rgba`) | **Glow Operation** (blend mode do halo: Add/Screen/Multiply) — AE *Glow* §*Glow Operation*; Unreal expõe o mesmo pelo método | **NÃO** — o `BlendState` é do pipeline (`motion_fx.rs:265`), não do stream; nenhum nó o alcança. ⚠️ E pela navalha do §0, **só o aditivo é z-seguro** — Multiply "só no Motion" é a operação que quebra o z | **natureza** (o aditivo é o que torna a Opção B possível) para Multiply · **omissão** para **Screen** (que é aditivo-compatível: `a+b−ab`, monotônico e nunca escurece) | **P2** (só o Screen) | modo `Add` ⇒ o de hoje |
| `fx.glow` | idem | **Glow Dimensions H/V** (AE) — bloom **anisotrópico**, que é o *streak* anamórfico que Unity (*Lens Flare/Streak*) e Unreal (*Bloom Convolution*) shipam e é O look cinematográfico | **NÃO** — a cadeia é mips isotrópicos com tent redondo (`fr` escalado por aspecto *para ficar redondo em pixels*, `motion_fx.rs:387-389`). Tentado: `fx.glow` duas vezes ⇒ o `from_graph` lê **o PRIMEIRO nó e ignora o resto** (ver a linha abaixo) ⇒ nem empilhar dois glows é possível | **omissão** (2 params: `stretch` + `angle`, na mesma cadeia) | **P1** | `stretch = 1` (isotrópico) ⇒ o halo de hoje ao bit |
| `fx.glow` | idem | **Um SEGUNDO `fx.glow` é silenciosamente INERTE** — `from_graph` faz `.find(...)` e devolve o **primeiro** (`lib.rs:149`). Não é gap de referência: é a lei anti-knob-morto deste repo (doc 88; *"botão que não faz nada é pior que botão que falta"*) | **N/A** — o defeito é a AUSÊNCIA de resposta: o 2º nó pinta, aceita clique, entra no undo e **não faz nada** | **omissão** (defeito de produto, não de catálogo) | **P1** | qualquer cura (badge ⚠ do `ph2d-motion-diagnose` · somar os dois passes · recusar) preserva o caso de UM nó, que é o único que existe hoje |
| `fx.glow` | idem | **Clamp** do bright-pass (Unity URP Bloom §*Clamp*) — teto no valor que entra na cadeia, o antídoto dos *fireflies* | **NÃO** — o `tint` chega **sem clamp** ao lowering (doc 67 §4, textual: *"o `tint` da instância é `[f32;4]` sem clamp"*), então um único valor enorme lava a tela. ⚠️ É exatamente a classe do §0 do CLAUDE.md: **o teto não está medido** — não sei em que valor a cadeia satura, e o slider `intensity` para em 4.0 sem `ParamHardMax` | **omissão** | **P1** | `clamp = ∞` ⇒ o de hoje |
| `fx.glow` | idem | **A COR do halo por RAMPA** — AE *Glow Colors: A & B* + *Color Looping*/*Color Phase*; Unreal tem 5 tints por tamanho de bloom | **PARCIAL** — hoje há `saturation` + um `tint` **constante**. ⚠️ **E nós temos a peça que a referência não tem:** `ParamWidget::Curve`/o gradiente como text param (doc 85) e o **canal de LUT** que leva a curva ao DEVICE (`KernelResolver::luts()`, ADR da família `field.*`) — a rampa por raio do halo é **barata aqui e cara lá** | **omissão** | **P2** | rampa de 1 stop = o `tint` de hoje |
| `fx.glow` | idem | **Glow Based On: Alpha Channels** (AE) — o bright-pass lê alfa em vez de luma | **NÃO** — o `prefilter` lê `max(r,g,b)` premult | **omissão** (1 param de modo) | **P2** | modo `Luminance` ⇒ o de hoje |
| `fx.glow` | idem | **Dirt texture** (Unity `Dirt Texture`/`Dirt Intensity`; Unreal `Bloom Dirt Mask`) | **NÃO** — mas a convenção `texture_id` do doc 86 já leva imagem ao device | **omissão**, **cara** (asset + fiação de shell) | **P2** | intensidade `0` ⇒ o de hoje |
| `fx.rgb_split` | 5 (`mode`·`x`·`y`·`strength`·`opacity`) | **O EIXO da aberração não é autorável** — Cavalry *Falloff* tem `center`; o post-process de jogo usa o centro da TELA. O nosso é o **centroide do layout**, calculado e não escolhido (`offsets()`, `lib.rs`) | **NÃO** — não há como deslocar o eixo sem mover a arte. ⚠️ **O centroide é uma DECISÃO documentada e superior** (doc 38 §2: o efeito segue o layout), então o gap não é "usar o centro da tela": é **não haver um offset do eixo** | **omissão** (2 params + 1 modo) | **P2** | offset `(0,0)` ⇒ o centroide de hoje |
| `fx.rgb_split` | idem | **Start Offset / raio interno** — Unreal *Chromatic Aberration §Start Offset* (a franja começa só além de `r₀`; o centro fica perfeitamente limpo) | **PARCIAL — tentado e MEDIDO no código:** o nó multiplica o alfa de cada fantasma por `falloff_at(input, i)` (`lib.rs`), e a família `field.*` escreve exatamente essa coluna ⇒ **`field.box`/radial → `field.remap`(curva) → `fx.rgb_split`** faz a franja *aparecer* de dentro para fora. ⚠️ **Mas modula a OPACIDADE, não o DESLOCAMENTO**: dentro de `r₀` a franja fica transparente em vez de inexistente, e o deslocamento segue linear desde o centroide. Perto o bastante para muitos usos, **não** o que o Unreal faz | **omissão** | **P2** | `start = 0` ⇒ o de hoje |
| `fx.rgb_split` | idem | **Deslocamento por canal INDEPENDENTE** — Photoshop *Lens Correction* tem TRÊS eixos (Red/Cyan · Green/Magenta · Blue/Yellow); ours é o par complementar simétrico R ↔ G+B | **NÃO** — a estrutura de 2 fantasmas complementares é o que torna o efeito correto sob alpha-blend (doc 38 §2, com mutante provado). Um 3º eixo exigiria 3 fantasmas e **o miolo sairia errado** pelo motivo que o doc 38 já mede | **natureza** — a restrição vem do *over* do renderer, não de preguiça | ⛔ **recusado com motivo** | — |
| `fx.rgb_split` | idem | `ParamUnit` ausente em `x`/`y` (comprimento de mundo) e `strength` (`Ratio`); sem `ParamHardMax` | **N/A** | **omissão** (idem `fx.drop_shadow`) | **P1** | metadado não move número |

**Contagem da família:** 3 nós · 16 linhas · **P0 = 0** · **P1 = 6** · **P2 = 7** · **⛔ = 3**
(dois refutados por cadeia que FUNCIONA, um recusado por natureza com mutante já provado).

⚠️ **Por que ZERO P0:** a régua do §7 do plano exige *"inexprimível **E** o artista vê na primeira
cena **E** todas as referências têm"*. Nenhum item acima passa nas três — o que a família tem de
mais grave (a sombra macia) tem **fence explícita** e a cura verdadeira é a §2, não um param.
**O P0 desta família não é um param: é a AUSÊNCIA de catálogo**, e ela vive na seção seguinte.

---

## §2 — `EFEITOS QUE FALTAM:` (a pergunta principal desta família)

⚠️ **Sobre o número 54.** O cabeçalho da §A.4 do dump declara **"Filters (54)"**; a linha
enumera **~50 nomes** (contando *Linear/Radial Wipe* como dois). A diferença de ~4 **não está no
dump** e não a inventei — trato a lista nomeada como o universo, e o *delta* fica registrado.

**O que o grafo tem, dos ~50 nomeados:** Drop Shadow · Glow · RGB Split · Chromatic Aberration
(o 2º modo do mesmo nó) · Slit Scan (`motion.slit_scan`) · Spherise (`motion.spherize`) ·
Bulge (≈ o mesmo `spherize`) · Mirror (`motion.mirror`) · Fill Color (`motion.tint` Solid)
= **9**. ⚠️ **Cinco deles NÃO estão na família `fx.*`** — são deformers/transform/cor —, e isso
não é desarrumação: os nossos operam sobre **GEOMETRIA e INSTÂNCIAS**, os deles sobre **PIXELS**.

⇒ **~41 dos ~50 filtros da Cavalry não existem no grafo.**

| Efeito ausente | **já existe noutro módulo?** | vale como nó `fx.*`? (a navalha do §0) |
|---|---|---|
| **Gaussian Blur** | **SIM** — Painter `AdjustmentKind::GaussianBlur` · Vector `FxOp::BLUR` (radius de mundo) | ⛔ subtrativo/remapeador ⇒ pós-produção |
| **Directional / Motion Blur** | **SIM** — Painter `MotionBlur` | ⚠️ **exceção interessante:** um motion blur do MOTION é aditivo-compatível se feito como *streaks* de luz; como borrão de matte, não |
| Fast Blur · Box Blur | ~ (mesma família de algoritmo no Painter) | ⛔ |
| Zoom Blur · Luminance Blur · Bilateral Blur · Background Blur | **NÃO** | ⛔ (raster de frame) |
| **Inner Shadow** | **SIM** — Vector `FxOp` "Inner Shadow" (+ modos Proximity/Contour) | ⛔ **natureza**: exige o INTERIOR da silhueta; uma instância de Motion é um quad texturado — não há silhueta no stream |
| **Inner Glow** | **SIM** — Vector "Inner Glow" | ⛔ idem |
| **Outline / Stroke** | **SIM** — Vector "Outline" (`Width`, com o corte duro medido) | ⛔ idem (mas ver SUPERAR #3) |
| **Halftone** | **SIM** — Painter `Halftone` | ⛔ raster de frame · ⚠️ *mas* halftone **por-instância** (a grade É a arte) já é `motion.grid + field + motion.scale` |
| **Posterize · Threshold · Levels · HSV · Brightness&Contrast · Gamma · B&W · Invert · Sharpen · Grain** | **SIM, TODOS** — Painter: `Posterize`·`Threshold`·`Levels`·`HueSaturationBrightness`·`BrightnessContrast`·`Exposure`·`BlackAndWhite`·`Invert`·`Sharpen`·`Noise` (+ `Curves`·`ColorBalance`·`Vibrance`·`ColorLookupLut`·`PhotoFilter`·`SelectiveColor`·`ChannelMixer`·`ShadowsHighlights`) | ⛔ **é literalmente a grade de tela inteira que o Enio mandou retirar** (cerca C4) |
| **Gradient Map** | **SIM** — Painter `GradientMap` · Vector "Gradient Map" | ⛔ **EXPRESSÍVEL no grafo, cadeia verificada:** `motion.luminance` (Rec.709 do `tint` → campo de valor) **→** `motion.color_ramp` (porta `t`) — os dois nós existem e as portas casam |
| **Tritone / Duotone** | **SIM** — Vector "Duotone" | ⛔ caso particular do acima (rampa de 2 stops) |
| **Erosion / Dilate** | **SIM** — Vector "Grow / Shrink" | ⛔ morfologia de matte |
| **Distortion / Distort Edges** | ~ — Vector "Turbulence" (deslocamento por ruído) | ⛔ no raster · ✅ **já temos no eixo certo**: `motion.noise`/`wiggle`/`bend`/`twist` deformam a GEOMETRIA |
| **Shift Channels** | ~ — Painter `ChannelMixer` | ⛔ (o `fx.rgb_split` já isola canais internamente, doc 38 §2) |
| **Vignette** | **NÃO** (foi construída e **REMOVIDA**, cerca C4) | ⛔ **duplamente:** recusada por ordem do Enio **e** já expressível por-instância — `field.box`(radial) → `motion.tint(Solid preto)`, porque o `motion.tint` faz `lerp(existing, target, falloff)` |
| **Linear / Radial Wipe** | **NÃO** | ⛔ **EXPRESSÍVEL:** `field.box`(Linear) ou `field.radial_sweep` → `motion.cull(Falloff)` — e o doc-header do `motion.cull` já nomeia esta receita (*"sort radial + cull … wipes a layout in from the centre"*) |
| **Venetian Blinds · Stripes · Scan Lines** | **NÃO** | ⚠️ **PARCIAL por-instância** (`value.pattern`/`value.step` sobre o índice → `motion.cull`/`tint`); como raster de frame ⇒ pós-produção |
| **Pixelate · Pixel Sorting · Dithering · Chroma Key · Edge Detection · Polar Coordinates · Scrape · SkSL Filter** | **NÃO — em módulo nenhum do app** | ⛔ raster de frame, todos ⇒ pós-produção |
| **Light Sweep** | **NÃO** | ✅ **ADITIVO ⇒ cabe como nó `fx.*` HOJE** — uma banda brilhante que varre é luz somada; a maquinaria da Opção B a comporta sem tocar o z (ver SUPERAR #2) |

### Os TRÊS efeitos mais importantes que faltam (o ranking, com o motivo)

1. **A SOMBRA MACIA** — é a única linha da tabela §1 que o artista vê na PRIMEIRA cena, o app já
   a tem no módulo Vector (`FxOp::DROP_SHADOW` com `Radius`), e a nossa é *hard-edged* por uma
   fence de 2026-07-12 que **hoje tem uma resposta que ela não tinha** (SUPERAR #1).
2. **O STREAK ANAMÓRFICO** (`fx.glow` anisotrópico) — 2 params na cadeia que já existe, blast
   radius zero, e é o que separa "tem bloom" de "parece cinema". Unity e Unreal shipam; nós não.
3. **O SEGUNDO `fx.glow` INERTE** — não é um efeito que falta, é um **controle morto que já
   shipou**; pela lei deste repo isso vale mais que qualquer item novo.

⚠️ **E o achado que o brief pediu, dito sem rodeio:** dos ~41 filtros ausentes do grafo,
**~21 já existem noutro módulo deste app** (15 com nome idêntico no Painter/Vector, ~6 como
parente próximo). **Isso NÃO é um gap de catálogo — é uma pergunta de ONDE o efeito mora**, e ela
já tem resposta oficial: o **módulo de pós-produção** que o Enio anunciou ao mandar remover o
post-stack. Construir `fx.levels`/`fx.vignette`/`fx.posterize` como nós de Motion seria a
**terceira** cópia da mesma matemática (Painter → Vector → Motion) num lugar onde ela quebra o z.

---

## `SUPERAR:` (o que nenhuma referência tem, derivado do que só nós temos)

**1. A sombra que vem de uma LUZ, não de um ângulo.**
Em toda referência (`AE Direction`, `Photoshop Angle`, Cavalry, Vector `Offset X/Y`) a direção da
sombra é **um número para a camada inteira** — no nosso código, literalmente: `offset()` é
calculado **uma vez fora do laço** (`fx-drop-shadow/lib.rs`), e um param dirigido é *um número por
TICK*, nunca por instância (o mesmo teto que o gabarito do plano §10 mediu no `emitter`).
Nós temos a família **`field.*` composável** e a coluna `falloff`, que os dois FX de stream **já
consomem**. Fazer `direction`/`distance` serem **derivados por-instância** (de uma posição de luz,
ou de um campo composto) dá o que raster nenhum consegue: **cada elemento projeta a sua sombra
para longe da MESMA luz** — as sombras divergem, e a cena lê como um ponto de luz de verdade em
vez de um deslocamento uniforme. Custo: a mesma leitura que o `falloff_at` já faz.

**2. O `LIGHT SWEEP` como luz aditiva, bit-exato sob scrub.**
Nas referências o *Light Sweep* é um filtro raster com estado de tempo. Aqui ele é **aditivo**
(cabe na Opção B sem tocar no z, §0) e o Motion é **função pura do playhead** — então a varredura
é reprodutível ao bit para trás e para a frente, o que o filtro deles não é.

**3. O contorno que é GEOMETRIA e não matte.**
`Outline` no Vector é raster (uma banda ao redor da matte). No grafo, um contorno de instância é
`motion.clone`/`mirror` + `motion.scale` — **geometria viva**, que sobrevive a zoom sem
re-rasterizar e é deformável a jusante. Nenhum filtro raster tem isso.

**4. Uma franja cromática que segue a ARTE.**
Já é verdade e merece ficar escrito: o `fx.rgb_split` em Aberration ancora no **centroide do
layout**, não no centro do frame (doc 38 §2, mutante #2 provado). Todo post-process de jogo ancora
na tela — mova a arte e o efeito escorrega dela. O nosso não.

---

## `CERCAS:` (Chesterton — grepadas ANTES de propor, cada uma com o sítio)

- **C1 — o blur da sombra é raster e foi deixado de fora DE PROPÓSITO.**
  [doc 38 §3](../38_fx_ghost_copias_rgb_split_drop_shadow_nota_adr.md) + o doc-header de
  `fx-drop-shadow/lib.rs` (textual): *"What is deliberately NOT here: blur … belong to the HDR
  compositor pass FX … **Não** fabriquei maciez falsa com uma pilha de fantasmas."*
  ⚠️ **A fence continua válida — e a razão MUDOU de lugar.** Em 2026-07-12 ela dizia *"cross-module,
  PARE e reporte"*; hoje a maquinaria de passe do Motion **existe** (Opção B, doc 67). O que a
  mantém de pé é a navalha do §0: **um halo escuro não pode ser somado.** A rota que a supera é
  nomeada e não é um param — é um **passe ANTES** do passe de sprites (limpar → halo da sombra →
  cena fundida → glow aditivo → tonemap), que é *decisão de renderer*, não de nó.
- **C2 — `fx.mirror` foi CANCELADO** (doc 38 §0): o `motion.mirror` já faz exatamente aquilo.
  *"Antes de criar o nó que o plano pediu, procure o nó que já o faz."*
- **C3 — o `glow` é um NÓ e não um campo do documento** (doc 67 §3): o caminho alternativo
  (`PassFx` no `MotionDoc` + seção `[fx]`) **foi construído e descartado** — duas portas para o
  mesmo estado divergem. Qualquer FX de passe novo entra pela mesma porta: um nó passthrough.
- **C4 — ⛔ A GRADE DE TELA INTEIRA FOI CONSTRUÍDA E REMOVIDA POR ORDEM DIRETA DO ENIO.**
  Commit `9a36d4a27` (build, ADR-0145 provisório) → `f2daa787a` (revert integral), citando-o:
  > *"vamos abandonar esses efeitos de tela inteira dentro do motion. Pois teremos um modulo de
  > pos producao no futuro. Retire esse efeito e limpe o codigo."*
  Saíram `post_stack.rs`, `post_stack.wgsl`, o `Pass 1d`, o `App.grade` e o ADR. **Isto recusa,
  de uma vez, a maior fatia dos 54 filtros** (exposição · tint · contraste · saturação · vinheta ·
  levels · gamma · posterize · threshold · invert · B&W · grain · sharpen…). Propor qualquer um
  deles como nó de Motion é reabrir uma decisão do Enio.
- **C5 — a premissa "reusar o compositor do Painter" é FALSA** (doc 66 §1, com `file:line`): o
  compositor é **8-bit** e o round-trip 16F→sRGB8→16F **destrói** o que o bloom vive. Quem propuser
  "é só chamar o `ph2d-painter-effects`" está repetindo uma instrução que produz um bloom errado.
- **C6 — os dois FX de stream duplicam `id`** e por isso vão **depois** de tudo que pareia estado
  por id (`motion.integrate`, `motion.spring`) — doc 38 §4 + doc-header dos dois. Um param novo que
  dependa de identidade estável herda esta restrição.
- **C7 — `MAX_INSTANCES = 65_536` e o FX se DESLIGA inteiro ao estourar** (doc 38 §4): *"uma cena
  sem um terço das franjas lê como bug; uma cena sem aberração lê como o efeito está desligado."*
  Qualquer param que multiplique a contagem (Softness por N taps) mora sob este teto.
- **C8 — `fx.glow` params 9 / hints 6 NÃO é defeito** (doc 88 §9.3, com gate
  `every_declared_param_is_drawn_by_some_widget`): `ParamWidget::Color` declara-se num hint e
  desenha quatro canais. Idem `fx.drop_shadow` 6/3.

---

## `O DOC 63 ERROU EM:`

1. **A família `fx.*` NÃO TEM UMA LINHA na §3.2** — a tabela *"gap por nó existente"* cobre 22
   nós e **nenhum** deles é `fx.*`. Não é discordância: é **buraco**, e o doc 88 §9.3 já o
   nomeou (*"os outros 64 — `field.*` · `force.*` · **`fx.*`** · … nunca tiveram linha"*).
2. **A §8 declarou os pós-FX raster "fora desta linha" e o mundo ANDOU depois disso.** Ela diz
   *"o que falta é um estágio de render por-instância/grupo no pipeline — decisão de renderer,
   linha própria, ADR próprio"*. Isso era verdade em 2026-07-24 e **deixou de ser em 2026-07-14
   → 07-30**: o estágio **existe** (`MotionFx` + `SpriteRenderer::render_instances_only` +
   RT `Rgba16Float` do módulo), o `fx.glow` shipou por ele, e a Opção A foi **decidida pelo Enio
   e removida**. ⚠️ Ler a §8 hoje faz a próxima varredura pedir um ADR que já foi escrito, usado
   e revertido.
3. **A §8 diz *"os ALGORITMOS já existem em `ph2d-painter-effects`"* — meia verdade, e a metade
   errada é a cara.** O doc 66 §1 mediu: a `-effects` é **dados + kernels CPU**; o compositor
   está em `ph2d-render`, é **8-bit**, e reusá-lo para bloom **produz o efeito errado**. Duas
   frases do repo em contradição direta; a que tem `file:line` é a do doc 66.
4. **A §1 (*"a régua honesta"*) não conta a família:** ela lista 7 vãos estruturais e o pós-FX
   não é um deles, embora `fx.*` fosse 3 dos 87 nós à data.
5. **Contagem envelhecida (esperado, registro para a consolidação):** §1 diz *"87 nós / 318
   params"*; o censo de 2026-08-09 diz **118 nós / 411-420 params**. A §2.7 do doc 63 (`Aparência
   / estilo`) segue **válida e não conferida por mim** — ela é da família 8/9, não desta.
6. **O que o doc 63 ACERTOU e vale repetir:** *"54 filtros do Cavalry"* como fronteira, e a
   intuição de que a resposta é *"linha própria"*. O que ele não podia saber é que a linha própria
   ganhou nome: **o módulo de pós-produção**.

---

## Nota de método (o que EU não fiz, para o consolidador não herdar como feito)

- Os params exatos dos filtros **da Cavalry** não estão no repo — a §A.4 dá **só os nomes**. Toda
  citação de param acima é de **AE / Photoshop / Unity / Unreal**, ou **primeira-parte** (o nosso
  `AdjustmentKind` e `vec_filter_kinds::SPECS`, que são os mais fortes: mesmo app, mesmo artista).
- **Nenhum número foi medido por mim nesta conferência.** As duas grandezas que a §1 marca como
  *não medidas* — o teto real do `radius` do bloom e o valor de `intensity`/`tint` em que a cadeia
  satura — precisam de sonda antes de virar `ParamHardMax` (§0 do CLAUDE.md).
- As cadeias marcadas **SIM/PARCIAL** foram derivadas **lendo os manifests e os kernels** (portas,
  colunas e ordem de emissão conferem); **não** as executei num grafo. A §5 do plano manda o
  coordenador tentá-las — as três que mais importam são *Shadow Only* (`cull` Fraction), *Gradient
  Map* (`luminance → color_ramp`) e *Wipe* (`field → cull`).
