# Família 9 — COR / APARÊNCIA (4 nós)

**Conferência do [plano 89](../89_plano_conferencia_dos_nos.md)** · **Data:** 2026-08-09 · **Linha:** `line/motion-value`
**Nós:** `motion.tint` · `motion.color_ramp` · `motion.color_array` · `motion.luminance`
**Status:** ⬛ **W3 ABERTA E PARCIALMENTE FECHADA** (2026-08-09) — ver §3 abaixo. O resto da
tabela segue como claims.

---

## §0 — O que este documento mediu antes de escrever uma linha

Os quatro `MANIFEST` foram LIDOS (não o doc), os widgets contados no registry/painel, e **toda
cadeia de composição foi TENTADA contra o código que a executaria**. Três fatos estruturais
saíram dessa varredura e decidem quase toda a tabela:

1. **O loop de cor é one-way e LOSSY.** A única leitura de cor do catálogo inteiro é o
   `motion.luminance`, que colapsa RGB num escalar. O `value.attribute` **não alcança o `tint`**:
   o `field()` dele só trata `Column::Scalar` e `Column::Vec2` e um `Vec4` cai no
   `_ => vec![0.0; n]` — **zeros**, em silêncio (`ph2d-node-value-attribute/src/lib.rs:74-85`).
2. **Nada escreve R/G/B a partir de um valor.** O `motion.drive` é o único escritor value→coluna
   e o canal de cor dele é `CH_OPACITY`, que escreve **`ti[3]`** — o ALFA
   (`ph2d-node-motion-drive/src/channel.rs:66,74,152-165`).
3. **O `blend` do `motion.mixer` é um ESCALAR GLOBAL** (`v.first()`,
   `ph2d-node-motion-mixer/src/lib.rs:214`), não um campo. É isso que derruba a cadeia óbvia de
   "mascarar uma rampa por um campo".

*Sem (1) e (2), toda referência da forma "ajuste a cor que já está lá" (hue/sat, blend modes,
color info) é inexprimível por construção — não por falta de um knob.*

---

## §1 — Tabela (formato fixo da §3 do plano 89)

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `motion.tint` | **9 `ParamSpec` → 3 CONTROLES**: `mode` (`Enum` Solid/Gradient) + `r,g,b,a` (**1** `ParamWidget::Color` → swatch OKLCH) + `r2,g2,b2,a2` (**1** swatch). Sem text param. Lê a coluna `falloff` (máscara). GPU sim | **blending mode da cor** (Mix · Add · Subtract · Multiply · Divide) — C4D MoGraph Effector, *"Color group: Color Mode (Off · Effector Color · Fields Color · Custom Color) · **Blending Mode (Mix · Add · Subtract · Multiply · Divide)** · Color · Use Alpha/Strength"* ([`referencia_pesquisa_c4d_fields.md` §linha 116](../referencia_pesquisa_c4d_fields.md)); AE aplica o efeito Fill/Tint sob um blend de camada | **NÃO.** (a) `motion.mixer(Add)` soma **todas** as colunas comuns — `P` inclusive ⇒ dobra as posições; (b) o `blend` do mixer é `v.first()`, escalar global (`mixer/lib.rs:214`); (c) `motion.drive` só alcança o alfa (`channel.rs:74`). Não há nó que combine dois `tint` por instância | **omissão** — o kernel tem UMA lei (`mixed_tint` = lerp), e o enum de modo nunca foi escrito | **P1** | `blend = Mix (0)` ⇒ o `lerp(existing, target, falloff)` de hoje, bit a bit |
| `motion.tint` | idem | **hue / saturation / lightness sobre a cor EXISTENTE** — Cavalry *"Color/Alpha/**HSV Material Override**, Swap Color, Material Sampler"* ([cavalry A.2 §73](../referencia_pesquisa_cavalry.md)); Blender **Hue Saturation Value** (Utilities▸Color, [blender GN §13/§24](../referencia_pesquisa_blender_gn.md)); AE **Hue/Saturation** (Master Hue/Saturation/Lightness) | **NÃO — e a razão é a §0.** Tentado: (a) `value.attribute` custom `"tint"` ⇒ **zeros** (Vec4 não é lido); (b) `motion.luminance → value.math → color_ramp` recupera só o BRILHO (o matiz já foi perdido); (c) `motion.drive` escreve o alfa. **Nenhuma cadeia lê R/G/B de volta** | **omissão** — falta a metade de LEITURA do loop, não um param | **P0** | `hue = 0 · sat = 1 · val = 1` ⇒ identidade sobre qualquer cor |
| `motion.tint` | idem | **colorir por LUMINÂNCIA** — AE **Tint** (*Map Black To / Map White To / Amount to Tint*); Cavalry Filter **Gradient Map** ([cavalry A.4 §113](../referencia_pesquisa_cavalry.md)) | ⛔ **SIM, e já é uma cena demo:** `… → motion.luminance → motion.color_ramp(t)`. O [doc 31 §cena](../31_make_point_luminance_nota_adr.md) monta exatamente `grid → color_ramp(Rainbow) → luminance → color_ramp(t, Heat)` | — | ⛔ **recusado com motivo** | — |
| `motion.tint` | idem | **cor ALEATÓRIA por instância** — C4D Cloner Transform tab, *"**Color** (por clone, gradiente/**aleatória**)"* ([c4d §22/§101](../referencia_pesquisa_c4d_fields.md)) | ⛔ **SIM:** `value.instance_field(mode = Random)` → `motion.color_ramp.t` (o `Random` é hash `(seed, index)`, `value-instance-field/src/lib.rs:117`) ⇒ cor aleatória ao longo de uma rampa autorada | — | ⛔ **recusado com motivo** | — |
| `motion.tint` | idem | *(adjacência, não gap)* o modo **Gradient** é uma rampa de **2 stops keyed no índice** — o `motion.color_ramp` faz o mesmo com N stops | **SIM**, com uma diferença que é o valor real do modo: `color_ramp` **SUBSTITUI** o `tint`; `tint` **LERPA sobre o existente pelo `falloff`**. O que o modo Gradient entrega e o `color_ramp` não é a MÁSCARA | natureza (duas leis de escrita diferentes) | **P2** | — |
| `motion.color_ramp` | **0 `ParamSpec`** + **1 TEXT param `ramp`** → o editor da [doc 85](../85_gradient_editor_nota_adr.md): barra + **N marcadores arrastáveis** + **1 swatch OKLCH por stop** + botão de **interp** (cicla os 5: Linear/Ease/Constant/Cardinal/B-Spline) + `+`/`−` + **4 chips de preset**. Porta `t` (ladder 0/1/n com broadcast). GPU por **3 LUTs**. **Nó RICO, não magro** | **espaço de interpolação** (RGB / HSV / HSL + caminho de matiz Near/Far/CW/CCW) — Blender **Color Ramp** (`ColorRamp`/`ColorBand`), e o modelo **já os implementa**: `RampColorMode::{Rgb,Hsv,Hsl}` + `RampHue::{Near,Far,Cw,Ccw}` (`ph2d-color/src/color_ramp.rs:22-63`) | **NÃO — o espaço não é autorável em lugar nenhum.** `parse_gradient` **fixa** `RampColorMode::Rgb` (`color_ramp_text.rs:115`) e o editor só cicla o `interp`. Um azul→amarelo passa pelo cinza morto e não há como pedir outra coisa | **omissão** — o motor tem, o FORMATO e a UI não | ✅ **FEITO** (era P0) | `space = Rgb`, `hue = Near` ⇒ toda string `g1`/`g2` existente rende byte a byte o que rende hoje |
| `motion.color_ramp` | idem | **interpolação POR STOP** — o *ramp parameter* do **Houdini** (cada ponto carrega a própria interpolação); o próprio [doc 63 §206](../63_pesquisa_industria_2026_e_plano_estado_da_arte.md) já lista *"interpolações por stop"* | **NÃO** — o `interp` é UM `u8` global na string (`color_ramp_text.rs:59,87`) e o botão do painel cicla o global | omissão | **P2** | stop sem interp própria ⇒ o interp global (a string de hoje é exatamente isso) |
| `motion.color_ramp` | idem | ⚠️ **NÃO é gap de referência — é DIVERGÊNCIA CPU×GPU:** a **alfa por stop** existe no formato (`g2`, 2026-08-08) e no `ColorRamp::eval` (devolve `[f32;4]`), e o kernel escreve **`1.0` literal** (`color-ramp/src/lib.rs:151`) sobre **3 LUTs** (`cr_grad_r/g/b`, §185-204). O smoke que originou o `g2` foi *"a transparência das cores não está sendo respeitada no motion"* | — (é defeito). ⚠️ **E os dois gates de paridade são VERDES sobre ele**: usam presets e uma string **`g1`** opaca (`gpu_cpu_parity.rs:1290,1386`) ⇒ a fixture não contém o fenômeno | **omissão** (a wave do `g2` moveu formato + CPU e não o device) | ✅ **FEITO** (era P0 defeito) | a 4ª LUT (`cr_grad_a`) com fill `eval(t)[3]`: ramp opaco ⇒ 1.0 em toda entrada ⇒ byte-idêntico |
| `motion.color_ramp` | idem | **máscara por falloff / campo** — C4D: o Color group do effector tem *"Use Alpha/**Strength**"* e a pilha de Fields tem **canal Color** (*"3 canais amostrados: Value + Color + Direction"*, [c4d §57/§116/§197](../referencia_pesquisa_c4d_fields.md)) | **NÃO.** O nó faz `out.set("tint", …)` incondicional (`lib.rs:252`) — nunca lê `falloff`. Tentado: fork do stream + `motion.mixer(Blend)` com `value.attribute("falloff")` no `blend` ⇒ **falha**, o `blend` do mixer é `v.first()`, um escalar global (`mixer/lib.rs:214`). Nada blenda dois `tint` por instância | **omissão** | ✅ **FEITO** (era P0) | `mask = 1` (ou `falloff` ausente ⇒ 1) ⇒ a substituição de hoje |
| `motion.color_ramp` | idem | **teto de stops:** Blender permite **32** (`MAXCOLORBAND`; o nosso `ph2d_color::MAX_RAMP_STOPS` **é 32**), o editor corta em **8** (`snapshot_ids.rs:50`) | **N/A** — é cap de UI. ⚠️ O motivo escrito é *"o painel é estreito e a faixa tem de ficar legível"*, **sem medição** (§0 do CLAUDE.md: um teto legítimo diz de que RECURSO é). A row de **paleta** provou o padrão oposto no mesmo painel (envolve, altura em função da contagem) | omissão (cap de conforto) | **P2** | `MAX_GRADIENT_STOPS` medido ⇒ ramp de ≤8 stops idêntico |
| `motion.color_ramp` | idem | **Cycle Repetitions + Phase Shift** do gradiente — AE **Colorama** | ⛔ **SIM:** `value.instance_field(Ramp)` → `value.gain(×N)` → `value.wrap` → `color_ramp.t` (repetição) e `value.math(+φ)` antes do wrap (fase) | — | ⛔ **recusado com motivo** (P2 de ergonomia, não de capacidade) | — |
| `motion.color_ramp` | idem | **gradiente ESPACIAL** (linear/radial ancorado no canvas) — AE **Gradient Ramp**; Cavalry Shaders *Gradient / Multi-Point Gradient* ([cavalry A.4 §114](../referencia_pesquisa_cavalry.md)) | ⛔ **SIM, e mais forte que a referência:** `field.box` / `field.radial_sweep` escrevem a coluna `falloff` (`field-box/src/lib.rs:3,152,196`) → `value.attribute(Custom "falloff")` → `color_ramp.t`. Os `field.*` COMPÕEM (combine/remap) e têm gizmo de canvas; o Gradient Ramp do AE é fechado | — | ⛔ **recusado com motivo** (o que falta é ergonomia: ver `SUPERAR:` 2) | — |
| `motion.color_array` | **0 `ParamSpec`** + **1 TEXT param `palette`** → o editor: **strip de swatches OKLCH que ENVOLVE, sem cap de comprimento** + `+`/`−`. Porta `offset`. **Sem lowering de GPU** | **o `offset` / índice tem de ser um CAMPO por instância** — Cavalry: *"Color Array … arrays indexáveis de cada tipo (**cor por índice do clone**)"* + os **Index Context / Velocity Context** que alimentam esse índice ([cavalry A.3 §90/§99](../referencia_pesquisa_cavalry.md)). Hoje `scalar_first(...).first()` (`lib.rs:79-84,104`) ⇒ **um campo por-instância é silenciosamente DESCARTADO** | **PARCIAL, a um custo que ninguém paga.** Tentado: `motion.sort(key = Random)` → `color_array` — funciona (a permutação carrega TODAS as colunas, os pontos não se movem) mas **destrói a ordem de índice** para tudo a jusante (`stagger`, `cull`, `trail`, o `t` posicional do próprio `color_ramp`). Pelo caminho direto: **NÃO** | **omissão** — o ladder 0/1/n existe no catálogo (`colorize`, `drive`, `mixer` de posição) e este nó ficou no `.first()` | **P1** | campo de comprimento 1 continua lido como o escalar de hoje ⇒ byte-idêntico; o par per-elemento é caminho novo |
| `motion.color_array` | idem | **máscara por falloff** — mesma citação C4D da linha do `color_ramp` | **NÃO** — mesmo mecanismo (`out.set("tint", …)` incondicional, `lib.rs:116`; e o mixer não blenda por instância) | omissão | ✅ **FEITO** (era P1) | `mask = 1` ⇒ hoje |
| `motion.color_array` | idem | ⚠️ **sem GPU** (0 `register_gpu_kernel`, contra 1 em cada um dos outros três da família) ⇒ um grafo que o usa perde a aceleração inteira | **N/A.** Mecanismo: a paleta é uma **lista de comprimento variável** e o device só tem uniforme fixo (`params`) e **LUT escalar**. ⚠️ Mas *"a i-ésima cor de uma lista"* **É** uma rampa `Constant` de stops equiespaçados ⇒ o canal de LUT que o `color_ramp` já usa serve, sem infra nova (ver `SUPERAR:` 4) | omissão | **P1** | o kernel é aditivo (side-metadata no registry); a rota CPU segue oráculo |
| `motion.color_array` | idem | *(não-gap)* interpolar entre slots | ⛔ **recusado por natureza, com mecanismo:** este é o nó **DISCRETO** de propósito (listras duras), e o contínuo já existe — `color_ramp` com `RampInterp::Constant` sobre stops equiespaçados dá o mesmo, e com `Linear` dá a versão interpolada. Um slider "blend" aqui seria a 2ª resposta à pergunta do vizinho ([`palette_text.rs` §"por que uma paleta não é um gradiente"](../../../crates/ph2d-color/src/palette_text.rs)) | natureza | ⛔ | — |
| `motion.luminance` | **0 `ParamSpec`, 0 text params.** Adapter puro `(in) → out(VALUE)`, Rec.709. GPU sim | **qual canal extrair** — AE **Colorama** *"Get Phase From"* (Lightness · **Hue** · **Saturation** · Red · Green · Blue · Alpha); Cavalry **Color Info** (*"amostra pixels de imagem→valores/cores"*, [cavalry A.3 §107](../referencia_pesquisa_cavalry.md) — marcado *TEMOS (motion.luminance) / **PARCIAL***); Blender **Separate Color** (RGB/HSV/HSL, [blender GN §13](../referencia_pesquisa_blender_gn.md)) | **NÃO — este nó É a única porta de leitura de cor do sistema** (§0). `value.attribute` devolve zeros num `Vec4`; nenhuma outra rota existe. Hoje o sistema sabe responder *"quão claro?"* e **nada mais** sobre a cor de uma instância | **omissão** — um nó de 0 params aqui é **magro por OMISSÃO**, não por natureza: a referência dá 7 canais e nós damos 1 | ✅ **FEITO** (era P0) | `channel = Luma` ⇒ `0.2126·R + 0.7152·G + 0.0722·B`, bit a bit o de hoje |
| `motion.luminance` | idem | ⚠️ **DUAS PORTAS latente (correção, não feature):** o `motion.drive` escreve opacidade em **`tint[3]`** (`channel.rs:152-165`) e o picker do `value.attribute` oferece **"Opacity" → coluna `"opacity"`** (`value-attribute/src/lib.rs:117-120`) — **colunas diferentes**. Ler de volta a opacidade que o `drive` escreveu é inexprimível | ⚠️ **A leitura estava ERRADA e a medição a corrigiu:** não eram duas colunas divergindo — **ninguém escreve `"opacity"`**, e o renderer lê `tint[3]`. Era uma porta e um FANTASMA (ver §3) | omissão | ✅ **FEITO** (era P0-correção) | o picker aponta a lane (`tint`+`MODE_COMPONENT_BASE+3`); e o `luminance` ganhou o canal `Alpha` |
| `motion.luminance` | idem | *(não-gap)* coeficientes alternativos (Rec.601/2020) | ⛔ **recusado com mecanismo:** a coluna `tint` é **RGB linear** e os pesos Rec.709 são os de **luminância relativa** nesse espaço — trocar por Rec.601 seria pedir a luma de um espaço que não é o nosso. É o único nó da família cujo "0 params" é **natureza** | natureza | ⛔ | — |

---

## `SUPERAR:`

**1. O gradiente PERCEPTUAL sai de graça, porque a rampa vira LUT.**
A objeção padrão a interpolar em OKLab é o custo (`cbrt`/`pow` por amostra, HR-5) — e é exatamente
ela que o doc-comment do `motion.tint` registra: *"OKLab is a future, **cbrt-gated** refinement"*.
⚠️ **Para o `motion.color_ramp` essa objeção não existe, e o mecanismo é nosso:** o device **não
avalia a rampa** — ele amostra 3 LUTs de 256 entradas que a CPU enche **uma vez por cook**
(`LutSpec::fill`). Logo o `cbrt` roda **768 vezes por cook e ZERO vezes por instância**, num grafo
de 490k instâncias. E o motor já existe e é testado: `OklabColor::{from_linear,lerp,to_linear}`
(`ph2d-color/src/oklab.rs:109,142,170`). Nenhuma referência entrega gradiente perceptual num
sistema de partículas GPU-resident — Blender interpola em RGB/HSV, Cavalry em RGB, AE em RGB — e
o "cinza morto no meio" de todo azul→amarelo é o defeito que **todas** carregam. Aqui ele custa
**um variant no `RampColorMode` e um token `g3` no formato**.

**2. O `t` de uma rampa pode ser um CAMPO COMPOSTO — as referências têm as duas metades separadas.**
O AE tem gradiente-por-posição (Gradient Ramp, fechado: linear ou radial, dois pontos) e o C4D tem
cor-por-campo (Color Remap, uma aba **dentro** de cada field). Nós temos a família `field.*`
**composável** (`box` ∪ `radial_sweep` ∩ `index_range`, com `remap` por curva) escrevendo uma
coluna escalar, e um leitor genérico dela. Fechar a costura é **UMA row**: um seletor
`t from: Index | Value | Field` no `color_ramp` — e o que se ganha não é o gradiente espacial (esse
já é exprimível, ver a tabela), é **um gradiente cuja régua é um campo animado, composto e com
gizmo de canvas**, que nenhuma das duas referências consegue montar.

**3. A cor pode dirigir a SIMULAÇÃO, e ninguém faz isso.**
O `motion.luminance` já é `cor → valor`; com o canal escolhível (P0 da tabela) a **saturação** ou o
**matiz** de uma instância passam a alimentar `sim.spawn`, `motion.cull`, `force.*` — o loop
*aparência → simulação*. A Cavalry só tem o sentido inverso (Collision Events → Color), o C4D
mistura cor na pilha de fields mas não a devolve ao solver, e o Niagara lê cor de textura, não da
partícula. É a consequência direta de a cor ser **uma coluna do mesmo stream**, não um material
pendurado no fim do pipeline.

**4. A LUT já é a resposta do `color_array` no device — e ela traz o teto MEDIDO que falta.**
Uma paleta é uma rampa `Constant` de stops equiespaçados ⇒ 3 (ou 4) LUTs escalares, **zero infra
nova**. E ela dá a régua honesta para os dois caps que hoje discordam (paleta *"sem limite"* ×
gradiente *"8, porque o painel é estreito"*): a LUT tem **256 entradas**, então **uma paleta de até
256 cores cabe exatamente e sem perda** — um teto **de recurso**, medido, no lugar de dois números
escolhidos.

---

## `CERCAS:` (decisões já registradas — grepadas antes de propor)

- **`motion.tint` doc-comment:** *"The gradient lerp is in the wire's linear space
  (transcendental-free; **OKLab is a future, cbrt-gated refinement**)"* — DIFERIDO pelo custo
  transcendental. ⚠️ A premissa **não vale no caminho da LUT** do `color_ramp` (`SUPERAR:` 1); vale
  no `tint`, que avalia por instância no kernel.
- **`color_ramp_text.rs:40-42`:** *"Colour mode is always `RampColorMode::Rgb` and hue
  `RampHue::Near` in v1; an HSV/HSL custom ramp is a **future token**, not a reinterpretation of an
  old string."* — DIFERIDO **com o mecanismo já escolhido** (um header `g3`, append-only). Não
  reinterprete `g1`/`g2`.
- **[doc 85](../85_gradient_editor_nota_adr.md) §alternativas rejeitadas — "Per-stop alpha:
  diferido"** ⚠️ **SUPERSEDIDA**: o `g2` (2026-08-08) a entregou no formato e na CPU. A nota do doc
  85 **mente hoje**, e o que sobrou dela é o buraco do device (linha P0 da tabela).
- **doc 85: "estender o `LutSpec` para carregar vec4 — rejeitado"** (*3 LUTs escalares dão a mesma
  coisa reusando o canal existente*) ⇒ a 4ª LUT escalar (alfa) é **exatamente o caminho já
  sancionado**, não uma exceção.
- **doc 85: "NÃO há modo preset vs custom"** — os presets são **SEMENTES** que o editor carrega nos
  stops (report do Enio: *"as cores dos presets deveriam aparecer no colorramp e ser editáveis e
  arrastáveis"*). Não reintroduza um enum `preset`.
- **`motion.color_array` doc + `palette_text.rs`:** *"⚠️ isto é um DEFAULT, não um cap … não há
  máximo"* (Enio: *"color array poderia ter quantas cores o usuário quisesse, **tire os limites**"*)
  e *"a strip WRAPS, e é isso que torna 'sem limite' verdade na tela"*. Não re-imponha cap.
- **`palette_text.rs` §"Why a palette is not a gradient":** posições e interpolação seriam
  **controles mortos** numa paleta. Não funda os dois nós nem acrescente `pos`/`interp` ao palette.
- **[doc 31](../31_make_point_luminance_nota_adr.md) §43-46:** o `luminance` emite **VALUE** e
  **não** passa a geometria — a 1ª versão emitia `INST_VEC2` e o `t` do `color_ramp` **não
  validava**. Não "conserte" devolvendo a geometria.
- **`motion.tint` gate `default_params_are_opaque_white_and_no_op_on_white`:** o Start branco opaco
  é a IDENTIDADE, e existe por causa de um **cast reportado**. Não troque o default.
- **`color_ramp`: o fallback Rainbow é compartilhado pelas TRÊS rotas** (eval da CPU, fill da LUT,
  painel) — mexer numa delas sozinha reabre o "o render e o editor discordam".
- **`colorize` (`color-ramp/src/lib.rs:88-105`):** o ladder **0/1/n com BROADCAST em length-1** foi
  um bug real (*"um `value.lfo` coloria exatamente uma faísca"*), achado no port da GPU. Não volte
  ao braço `_`.
- **`motion.tint` GPU:** o `HAS_Index`/`HAS_Count` existe porque `ColumnBinding.identity` é
  CONSTANTE e a chave posicional não cabe nele. Qualquer chave nova de rampa herda esse cuidado.

---

## `O DOC 63 ERROU EM:`

- **§3 linha 153 — `motion.color_ramp` v2, "gradiente com N stops via `ParamWidget::Gradient` (hoje:
  2 cores + preset)", P0 (upgrade): FEITO.** A [doc 85](../85_gradient_editor_nota_adr.md)
  (2026-07-29) entregou o editor multi-stop, o text param, os presets-semente e as 3 LUTs. A linha
  **manda construir o que está construído** — o custo exato que a §1 do plano 89 nomeia.
- **§3 linha 154 — `motion.number_to_color`, P1: REFUTADO POR COMPOSIÇÃO.** A porta `t` do
  `motion.color_ramp` (com o ladder 0/1/n) **é** o *Number to Color*; um nó separado seria a 2ª
  resposta à mesma pergunta. O que sobra da linha é o adjetivo *"separado do ramp por posição"* —
  e isso é o item de ergonomia da `SUPERAR:` 2, não um nó.
- **§ linha 206 — "`motion.color_ramp` (2 stops) | N stops via Gradient (D2) · **interpolações por
  stop**": METADE envelheceu.** N stops está feito; *interpolações por stop* segue **válido** (e
  entra na tabela como P2, com a citação do Houdini).
- **§ linha 249 — "Heatmap por instância do campo selecionado … FALTA | **Baixo (1 tint por
  coluna)**": a ESTIMATIVA supõe uma capacidade que não existe.** Hoje não há como escrever o
  `tint` a partir de uma coluna arbitrária: o `motion.drive` só alcança o alfa e o `value.attribute`
  devolve zeros em `Vec4`. O item continua barato, mas pelo caminho
  `field → value.attribute(escalar) → color_ramp.t` — não pelo que a linha imagina.
- **[`referencia_pesquisa_c4d_fields.md` §57] — "3 canais amostrados: Value + Color + Direction |
  **FALTA** — nosso falloff só produz escalar": SEGUE VERDADEIRO**, e esta conferência mede o preço
  concreto: sem o canal Color (ou sem um `mask` nos nós de cor), **`color_ramp` e `color_array` não
  podem ser mascarados por campo nenhum** — dois P0/P1 da tabela são a mesma ausência.
- **[`referencia_pesquisa_cavalry.md` §73/§107] — "PARCIAL (tint, luminance, color_ramp)" /
  "TEMOS (motion.luminance) / PARCIAL": a coluna `status vs PH2D` está CERTA e o "PARCIAL" nunca foi
  desdobrado.** Esta tabela desdobra: o que falta em Cavalry §73 é **ler e ajustar** a cor
  (hue/sat/swap), e em §107 é **escolher o canal** do Color Info.

---

## §3 — O que a W3 fechou, e o que ela mediu no caminho (2026-08-09)

**Dois commits, dois P0.**

**(A) A alfa chega ao device.** A 4ª LUT (`cr_grad_a`), exatamente o *default que reduz* da
tabela. ⚠️ **E a prova de que os dois gates de paridade eram verdes sobre o defeito é
executável:** no estado que shipava (3 LUTs + o `1.0` literal) o gate novo sangra com
**|Δ| = 1e0** e os dois irmãos **passam** — a fixture deles não continha o fenômeno.
⚠️ O gate nasceu com um número que MENTIA: comparação fail-fast reporta o primeiro instance
que o scan alcança, e perto da ponta opaca isso é ~um épsilon (0,006), que se lê como
*"a tolerância está um fio apertada"* e esconde que o device escreve uma CONSTANTE.

**(B) A máscara por campo**, nos DOIS nós de cor, com a lei do `motion.tint`
(`existing·(1−f) + target·f`). O neutro não é promessa, é aritmética: `falloff` ausente lê
`1.0` e a forma é endpoint-exata, então um grafo sem campo é **byte-idêntico** — os 8 gates
que já existiam passaram sem edição de expectativa, porque usam stream vazio.

**Medido no caminho, e vale mais que os dois itens:**

- ⚠️ **O orçamento de bindings do cook contava só as COLUNAS.** O módulo também declara os
  dois arrays da grade, um buffer por redução e um por LUT ⇒ o check concedia um orçamento
  que o dispatch estourava. `motion.four_point_warp` declara **8** onde as colunas dizem 4.
  Corrigido com porta única (`codegen::storage_buffers`) + gate cujo oráculo é o TEXTO do
  módulo gerado. O pior kernel declara **13**, e é por isso que a 4ª LUT (6 no total) não
  chega perto de teto nenhum — **medido, não suposto**.
- ⚠️ **O helper `falloff_at` já estava copiado em NOVE+ crates-nó** antes desta wave, todos
  idênticos. A porta única já foi decidida ao contrário por acréscimo; colapsá-las toca nove
  crates e nenhuma é de cor ⇒ **wave própria**, nomeada em vez de contrabandeada.
- ⚠️ **Dois vermelhos-latentes pré-existentes**, provados no HEAD com o diff fora. Um
  CORRIGIDO (`the_bare_emitters…` semeava o `color_ramp` com os params `a_*`/`b_*` do
  manifesto de dois stops — a família exata da integração de 30/07); o outro **NOMEADO**
  (`value_slope…` falha por 1,05e-4 contra barra 1e-4: barra **absoluta** sobre coordenada
  cuja magnitude a fixture escolhe — outra família, outro oráculo).

### O terceiro P0 — **e ele era um FANTASMA, não duas portas** (2026-08-09)

A tabela catalogava *"DUAS PORTAS latente: o `drive` escreve `tint[3]`, o picker oferece
`"opacity"`"*. Medido, é pior e mais simples: **nenhum nó da biblioteca escreve uma coluna
de stream chamada `opacity`** (o `"opacity"` do `fx.rgb_split` é PARAM), e o
`lower_to_instances` lê o alfa de `tint` lane 3 — o `RenderInstance.opacity` é **cravado em
1.0**. Não eram duas convenções divergindo: era **uma porta e um nome que não existe**, e a
entrada caía no MISS ORDINÁRIO do módulo — zeros no comprimento cheio, indistinguíveis de
um nome digitado errado. A cura é a **lane** (`column: "tint", mode: MODE_COMPONENT_BASE +
3`), dizível só porque o W0-A a destravou.

### O quarto P0 — **o canal do `motion.luminance`** (2026-08-09)

O único leitor de cor do catálogo passou de **um** canal a **oito**: `Luma` (o default,
byte-idêntico) · `Hue` · `Saturation` · `Value` · `Red` · `Green` · `Blue` · `Alpha`. É o
*Get Phase From* do Colorama e o *Separate Color* do Blender, e é o que a `SUPERAR:` 3
desta família chamava de pré-requisito do loop **aparência → simulação**.

- ⚠️ **HSL ficou de FORA e o motivo NÃO é espaço** (o teto de opções virou 48 na mesma
  jornada): a *lightness* do HSL é `(max+min)/2`, a MESMA pergunta que o `Luma` responde
  com os pesos perceptuais — e a saturação do HSL só é definida em termos dela. Duas
  respostas para *"quão clara é esta cor?"* dentro do mesmo picker, com a nova sendo a
  pior.
- ⚠️ **Há UMA definição de matiz:** o canal delega ao `ph2d_color::rgb_to_hsv` (exposto
  `pub` nesta wave), o MESMO que o `RampColorMode::Hsv` interpola — senão um grafo que
  rampeia em HSV e lê o matiz de volta encontraria dois matizes.
- ⚠️ **O kernel é UM corpo com um `if`, não `variant_by_param`:** as variantes existem
  quando as BINDINGS diferem, e aqui a leitura é sempre `tint` e a escrita sempre `v`.
- ⚠️ **E uma afirmação minha caiu na medição:** escrevi que a paridade CPU×GPU seria
  bit-exata (*"sem transcendental, sem FMA a contrair"*) e o gate reprovou **no canal
  `Luma`, o que esta wave nem tocou**, por **1 ulp** — o WGSL PODE fundir
  multiplicação-e-soma. A barra virou o `1e-5` da casa, e a mutação mostra que uma
  diferença de LEI vale **~0,79**: quatro ordens de grandeza acima do arredondamento.

### O quinto P0 — **o ESPAÇO de interpolação da rampa** (2026-08-09)

O motor **sempre soube** interpolar em HSV/HSL (`mix2`/`cubic` ramificam em `color_mode`
desde que a crate nasceu, e o `unwrap_hues` do caminho cúbico existe só para isso) — o que
faltava era **onde guardar a escolha**. `parse_gradient` cravava `RampColorMode::Rgb` e
`ColorRamp::new` crava `RampHue::Near`, então um azul→amarelo passava pelo **cinza morto**
e não havia como pedir outra coisa. **A capacidade estava construída e era inexprimível no
formato** — a forma exata do defeito que o `g2` fechou para a alfa.

- **O token `g3`**, o que a cerca do próprio módulo prescrevia (*"an HSV/HSL custom ramp is
  a future token, not a reinterpretation of an old string"*):
  `g3 <interp> <mode> <hue> <pos>:<r>,<g>,<b>,<a> …`. ⚠️ **A versão continua escolhida pelo
  CONTEÚDO** — `g3` só sai quando `mode != Rgb || hue != Near`, então todo gradiente que
  ninguém levou para fora do RGB serializa `g1`/`g2` **byte a byte** como antes.
- ⚠️ **A régua é o que a rampa precisa EXPRIMIR (os campos), não o que hoje pinta um pixel:**
  o matiz é inerte em RGB, e guardá-lo assim mesmo é o que faz a escolha do artista
  **sobreviver a um desvio** por RGB e voltar (gate próprio). Uma regra por *liveness*
  dropava o `Ccw` no caminho de volta.
- ⚠️ **E o DISPOSITIVO herda de graça, sem uma linha de WGSL:** o LUT da GPU é assado **na
  CPU** por `bake_into` → `eval`, pela MESMA `parse_gradient` — então não há segunda
  expressão da lei para divergir. A afirmação virou gate (`the_baked_lut_takes_the_space_the_string_asked_for`):
  as mesmas paradas em RGB e em HSV assam LUTs **diferentes**, e o de HSV concorda com o
  `eval` ponto a ponto.
- **Na UI:** um botão de **espaço** (RGB → HSV → HSL) ao lado do de interp, e um de
  **matiz** (Near → Far → CW → CCW) que ⚠️ **só é pintado fora do RGB** — o braço `Rgb` do
  `mix2` nunca chama `lerp_hue`, então ali ele seria um controle que gira e não muda um
  pixel. Gate de presença **e ausência**; **3 mutações, 3 sangram**.
- ⚠️ **E a header do editor virou uma LISTA:** a largura do rótulo era um literal com um
  comentário contando os vãos (`- INTERP_W - BTN_W*2 - gap*3`), que envelhece no dia em que
  um botão entra — e foi exatamente o que este espaço fez. Agora os botões são colocados da
  direita para a esquerda e o rótulo fica com a **sobra**.

O oráculo que decide a wave é de **APARÊNCIA, não de flag**: a saturação no meio de um
azul→amarelo, medida `< 0,05` em RGB e `> 0,9` em HSV — a régua é `max−min` dos canais, que
é a definição de saturação do HSV e vem de fora do nosso código. Um gate que comparasse
`back.color_mode` ficaria verde com o `eval` ignorando o campo.

**Segue P0 nesta família:** hue/sat **sobre a cor existente**, que continua precisando da
metade de ESCRITA do loop (o W0-B-genérico) — o `luminance` fechou a metade de LEITURA.

---

## §2 — Nota de escopo

Todo gap acima é **claim**. A verificação do fato decisivo (§5 do plano 89) é do coordenador: os
três que mais mudam a prioridade se conferem em minutos —
(a) `grep -n "cr_grad" crates/ -r` deve mostrar **3** LUTs e um `1.0` literal no `write_tint`;
(b) `value.attribute` sobre uma coluna `Vec4` devolve zeros (`field()`, o braço `_`);
(c) `motion.mixer` lê o `blend` com `.first()`.
