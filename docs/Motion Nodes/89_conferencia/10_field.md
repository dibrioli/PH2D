# 10 — FIELD (5 nós) · conferência do plano 89

**Data:** 2026-08-09 · **Família:** `field.box` · `field.combine` · `field.index_range` ·
`field.radial_sweep` · `field.remap` · **Referência autoritativa:**
[`referencia_pesquisa_c4d_fields.md`](../referencia_pesquisa_c4d_fields.md) (a referência de
ORIGEM) · [`referencia_pesquisa_cavalry.md`](../referencia_pesquisa_cavalry.md) §Falloff ·
[`referencia_pesquisa_houdini_mops.md`](../referencia_pesquisa_houdini_mops.md) §Falloffs.

> Params lidos do `MANIFEST` de cada crate (não do doc). Contrato congelado **não é tocado** por
> nenhuma proposta: tudo o que segue é `ParamSpec` do próprio nó, side-metadata no registry
> (`param_hard_max`/`param_units`/`param_groups`/`luts` são o precedente) ou nó novo.

---

## §1 — O achado que reordena a família (leia antes da tabela)

**O canal `falloff` é FECHADO À ESCRITA pelo domínio de VALOR — e isso é verificável, não
opinião.** Exatamente **7** nós do repo escrevem a coluna `falloff` (`field.box` ·
`field.combine` · `field.index_range` · `field.radial_sweep` · `field.remap` ·
`motion.falloff` · `motion.slit_scan`) e **nenhum deles tem porta de entrada no domínio
`Values`**:

```
$ grep -l 'set("falloff"' crates/ph2d-node-*/src/lib.rs | xargs grep -l 'Domain::Values'
NENHUM
```

E a porta que existiria para isso — `motion.drive`, *"o value-domain CONSUMER: route a value
field onto a channel"* — tem `labels: &["X", "Y", "Rotation", "Size", "Opacity"]`. **`falloff`
não está lá**; o `drive` LÊ a máscara e nunca a escreve.

⚠️ **Quatro das cinco espécies de campo que faltam colapsam nesse único mecanismo ausente**
(noise · textura/shader · fórmula · áudio), e uma quinta (atributo→campo) junto. O nó
`motion.luminance` **já devolve** a luma Rec.709 do `tint` como VALOR — o *Shader field* do C4D
está a **uma porta de escrita** de existir, e a mesma porta destrava as outras quatro.

---

## §2 — A tabela

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `field.box` | 9 (`width·height·soft·center_x·center_y·rotation·curve·strength·invert`) | **`strength` por-campo, COM SINAL** — C4D Remapping tab §B6 tem `Strength [-∞..+∞%]` **e** `Multiplier` em TODO field object; Cavalry §Falloff: *"Strength · multiplicador do resultado do Graph"*; MOPs Combine: *"Blend Strength"* | ✅ **FECHADO em 2026-08-19** — param `strength`, default `1`, faixa `−1..2`. ⚠️ **A forma é a de DOIS TERMOS, `f·s + (1 − s)`, e não `1 + (f − 1)·s`** — a mesma álgebra e **não** o mesmo número: em `s = 1` esta dá `f·1 + 0`, que é `f` **ao bit**, enquanto a outra passa por `f − 1` e perde bits para `f < 0,5`. O gate que prova isto usa uma máscara **SUAVE** de propósito, com um controle de fixture a exigir ≥4 amostras na rampa — o gate de arestas duras que escrevi primeiro tem `f ∈ {0,1}`, onde as duas formas concordam, e teria passado sobre a errada (a mutação B1 comprova). ⚠️ **Negativo NÃO é o `invert`:** em `s = −1` a máscara vira `2 − f`, o cheio fica em `1` e o vazio vai a `2` — o campo passa a **empurrar** em vez de mascarar, e sair de `[0,1]` é exactamente o que a linha do `clamp` abaixo passou a permitir. O `invert` trocaria os dois DENTRO da faixa. ⚠️ A faixa do slider é o **curso útil**, não um teto: o param não é limitado, e um param dirigido passa dela. **5 gates, 2 mutações, 2 sangram.** Cena `=66` | **omissão** | ✅ | `strength = 1` ⇒ a máscara de hoje, provada byte-a-byte contra a máscara crua |
| `field.box` | idem | **`Enabled`** — Cavalry §Falloff: *"Enabled \| desligado ⇒ **constante 1**"* | **SIM, exato** — o **BYPASS/MUTE (`H`)** do grafo (integrado 2026-07-30, record `y`) passa `input[0] → output[0]`, e para um campo isso **é** a constante 1 | natureza | ⛔ | — (um checkbox seria a 2ª porta do mesmo fato) |
| `field.box` | idem | **o neutro D12 do próprio doc-comment é INALCANÇÁVEL pela UI** — ele diz *"a box larger than the scene with `soft = 0`"* e o teste usa `width = 100`; o hint é `max: 40.0` e **não há `ParamHardMax` para `width`/`height`** (só para `soft`), e `ui.rs:206` é explícito: *"A param with no entry here **types to its soft `max`**"* | **NÃO** — nem arrastando nem digitando se chega a 100 | **omissão** (defeito de UI, não de kernel) | ~~P2~~ ✅ **FECHADO 2026-08-23** | `ParamHardMax` não move nada: ele só ALARGA a digitação ✅ **`ParamHardMax` em `width`/`height`, DERIVADO** (bloco Z, [doc 91](../91_os_tetos_que_ninguem_mediu.md) §2). ⚠️ **O recurso é a PRECISÃO**: nada nesta lei satura — uma caixa maior que a cena É o neutro —, então o que acaba é o `f32`: acima de `2²¹` somar o `step` do slider (0,1) não move o número. `2 097 151,875`, re-derivado a cada corrida pelo gate `every_precision_bound_param_types_to_the_measured_ceiling`. |
| `field.box` | idem | **Color Remap · Direction** (os canais 2 e 3) — C4D §C1: *"Um Field é `sample(p) -> {value, color, direction}` — **três canais, não um**"*; §C3: o canal Direction *"conversa direto com a coluna `accel`/forças"* | **NÃO** (a coluna `falloff` é escalar) | **omissão DECLARADA** | ⛔ | ⚠️ **CERCA** — doc 63 §0.1: *"Color Remap / Direction = **diferido** (§6.3; v1 = só o canal value/`falloff`)"* |
| `field.index_range` | 5 (`start·end·soft·curve·invert`) | **curva LIVRE de decaimento** — Cavalry §Falloff: *"Falloff Graph \| curva custom de decaimento"*; C4D Step effector: *"rampa 0→1 pelo índice do clone, **com spline de forma**"*; C4D §B6 Contour: *"Curve (freely definable)"* | **SIM, exato** — `field.index_range(curve = Linear) → field.remap(contour = Curve)`; o `ParamWidget::Curve` existe, o `LutSpec` o leva ao **device** e as 5 contours cozinham lá (*"No fallback… every mode is device-resident"*). ⚠️ tem de pôr `curve = Linear` no campo, senão as duas formas **compõem** em vez de a segunda substituir | natureza (a fatoração é o D1) | ⛔ | — |
| `field.index_range` | idem | **rank por ATRIBUTO, não por ordem de stream** — MOPs *"Falloff From Attribute \| remapeia atributo existente→falloff (min/max + **Auto Range**)"*; C4D Random effector modo **Indexed** | **PARCIAL, com efeito colateral NOMEADO** — `motion.sort(key) → field.index_range` dá o rank por qualquer chave que o `sort` aceita (é literalmente o mecanismo que a §4 do plano 89 cita no `motion.slit_scan`: *"o eixo é um `motion.sort` a montante"*). ⚠️ **mas o `sort` REORDENA o stream para sempre a jusante** (z-order, pareamento por índice, `id`), e nas referências o falloff-por-atributo **não reordena nada** | omissão (do *não-destrutivo*, não do rank) | ✅ **FEITO** (era P1) — `key = Index/Attribute` + a porta `attr`: o **posto** do elemento no campo, `s = rank/(n−1)`, e o stream **não se mexe**. Empates desempatam pelo ÍNDICE (ordem total, estável entre plataformas). ⚠️ **O `Auto Range` da mesma citação NÃO entrou, e isso é MEDIÇÃO:** `value.attribute → value.normalize(Range) → motion.drive(Falloff, Set)` já o exprime, e sem cair para a CPU. ⚠️ Ligado, RECUSA o device: um posto é ordenação global, e o contorno por-elemento seria `O(n²)` = 6,9·10¹⁰ comparações a 262 k | — |
| `field.index_range` | idem | **`probability` + seed** — Cavalry §Falloff: *"Probability (+Seed) \| converte o valor em binário 0/1 com probabilidade %"* | **SIM, exato** — `→ field.remap(probability, seed)`, que já o tem e com hash inteiro **bit-idêntico CPU↔GPU** | natureza (D1) | ⛔ | — |
| `field.radial_sweep` | 11 (`radius·start_angle·end_angle·repetitions·inner_radius·soft·center_x·center_y·rotation·curve·invert`) | **raio INTERNO (o anel)** — C4D §B4 field **Torus**; MOPs Shape Falloff: *"**inner/outer** (zona cheia→zero)"* | ✅ **FECHADO em 2026-08-19** — param `inner_radius`, default `0`. ⚠️ **`0` é o disco de hoje AO BIT, e não «quase»**: a rampa interna devolve `1.0` **exacto** para todo `r` quando o buraco é zero, e `min(rad, 1.0)` é `rad` para toda a imagem da rampa externa — nenhum caminho novo é tomado no default, e há gate sobre a FUNÇÃO (não sobre uma cena). ⚠️ **A banda macia do buraco come para DENTRO do anel**, como a de fora já fazia: a cura ingénua (`1 − edge_ramp`) põe-na **fora**, dá o mesmo valor nos extremos e desenha outro anel — o gate escolhe a amostra a meio passo de cada lado, que é onde as duas leis discordam. ⚠️ Um buraco maior que o raio externo **esvazia o campo**, e isso é uma resposta e não um erro: é o que deixa o slider partilhar o teto do `radius` em vez de precisar de um clamp que esconderia metade dos anéis legítimos. **5 gates, 2 mutações, 2 sangram.** Cena `=66` | omissão | ✅ | `inner_radius = 0` ⇒ o disco de hoje |
| `field.radial_sweep` | idem | **softness SEPARADA para a borda angular e a radial** — C4D dá `Remapping` por field e os shape-params por eixo; Cavalry Sweep separa Angles de Size | **NÃO** — é um número só (`soft: fraction`), então uma cunha **fina** com borda radial macia é inexprimível: `soft = 0.9` amacia as DUAS | ⚠️ **CERCA + omissão** — o doc-comment declara: *"uma knob adimensional, porque as duas bordas vivem em unidades diferentes, graus vs mundo"*. O motivo é bom; a consequência (o caso da cunha fina) não estava medida | **P2** | `soft_angular = soft` ⇒ hoje |
| `field.radial_sweep` | idem | `radius` digitável para além de **40** (mesmo mecanismo do `field.box`: sem `ParamHardMax`) — e o doc-comment promete o neutro *"`radius` larger than the scene"* | **NÃO** pela UI | omissão | ~~P2~~ ✅ **FECHADO 2026-08-23** | idem ✅ **Idem, nos DOIS raios** (o anel só existe enquanto `inner < radius`, então um teto menor no interno esconderia metade dos anéis que o externo alcança). |
| `field.combine` | 2 (`mode·strength`) | **`Average`** — MOPs Combine Falloffs (fetchado): *"blend modes de compositor (add/sub/mult/screen/max/min/**average**~) + Blend Strength"* | ✅ **FECHADO em 2026-08-19 — e o diagnóstico da célula estava CERTO**: era o clamp. ⚠️ **Ela ficou exprimível a DOIS nós no instante em que o toggle da linha abaixo entrou** (`Add(clamp off) → field.remap(multiplier = 0,5)` dá `0,85`), o que a põe **abaixo** do critério de `P1` da §7 — e há gate que corre a cadeia e a compara ao modo, número a número, porque *"ficou exprimível"* sobre uma composição que ninguém correu é uma alegação e não uma medição. Entra como **modo 8** na mesma, por duas razões escritas: a referência lista-a ao lado dos outros oito que já temos, e o custo é **um braço de `match`** dos dois lados. ⚠️ **Não trunca**, de propósito — truncar num modo novo reintroduziria o que o toggle acabou de curar | **omissão** (custa um braço de `match`) | ✅ | modo novo ⇒ nada muda nos 8 |
| `field.combine` | idem | **os modos NÃO deviam saturar** — C4D §C3, verbatim: *"**Min/Max SEM clamp por default** em vários pontos — valores <0 e >100% fluem de propósito (um Add de dois campos passa de 1; **o consumidor decide truncar**). O sistema é linear-space até o consumidor"* — o nosso `Add`/`Subtract` clampam INLINE | ✅ **FECHADO em 2026-08-19** — param `clamp`, **Toggle**, nasce `1` ⇒ todo grafo autorado byte-idêntico. ⚠️ **É a CAUSA-RAIZ das duas linhas**, e é por isso que vieram juntas. ⚠️ **Ele governa exactamente os modos 1 e 2**, os únicos que truncavam — e o gate disso é um **CENSO** sobre os nove modos, não um caso: se um modo novo nascer com um clamp inline, a contagem de sensíveis passa de dois e ele acusa. ⚠️ **O consumidor de facto decide, e nesta casa ele já sabia:** `motion.scale` lê a máscara como `1 + (amount − 1)·falloff` e `motion.tint` como `lerp(existing, target, falloff)` — os dois **extrapolam** além de `1` em vez de partir, que é a mesma figura que o `motion.mixer` documenta como *"um overshoot que tem figura"*. **7 gates, 3 mutações, 3 sangram.** Cena `=66` | **omissão** — é a MESMA causa da linha acima | ✅ | o toggle `clamp` nasce `1` ⇒ byte-idêntico |
| `field.combine` | idem | **`Clip`** (o 9º modo do C4D) — FLBASE verbatim: *"Clip \| functions as a **mask across all channels** using layer volume"* | **SIM para o canal value** — o próprio FLBASE diz *"Multiply \| effective for masking via null values"*, e `Multiply` nós temos. O `Clip` só difere **nos canais color/direction**, que estão diferidos | natureza (enquanto os 3 canais forem 1) | ⛔ | — |
| `field.combine` | idem | **pilha de N camadas** — C4D §C2: *"uma Field list é uma pilha ordenada de camadas, avaliada de baixo para cima"* | **SIM, exato** — encadear `combine → combine → combine`; num grafo a pilha É a cadeia, e a ordem fica VISÍVEL (o anti-padrão que o C4D odeia: §D, *"a ordem-importa da lista de effectors ser invisível para iniciantes"*) | natureza | ⛔ | — |
| `field.remap` | 12 + 1 text (`inner_offset·contour·curvature·steps·min·max·multiplier·clamp·invert·strength·probability·seed` + `curve`) | **animar a CURVA** — C4D §B6, o único item do dump com `!`: *"Spline \| curva livre \| + **Spline Animation Speed** (\"animates curve along X-axis\"!), **Spline Offset**, **Spline Range**"* | **NÃO** — a curva é um **text param** (uma string serializada `ph2d-curve`); animá-la exigiria reescrever a string por frame, e não há param que a desloque | **omissão** | ✅ **FEITO** (era P1) — `curve_offset` (o *Spline Offset*; o *Animation Speed* é uma expressão a somar tempo a ele), com WRAP, CPU **e** kernel. ⚠️ **A guarda `offset == 0 ⇒ devolve t` é load-bearing:** `x − floor(x)` leva `1.0` a `0.0`, e `t = 1` é o que TODA peça a máscara cheia entrega — sem ela, ligar o nó trocaria `curve(1)` por `curve(0)` em metade da cena. ⚠️ A costura é da CURVA (some numa cujos extremos concordem), e recortá-la seria decidir pelo artista. ⚠️ **Um SMOKE do Enio (21/08) achou o knob MORTO no estado em que o nó nasce:** `curve.map_or(t, |c| c.eval(shifted(t, off)))` não corre o `shifted` quando não há curva autorada — e o device fazia o contrário (a LUT assa a identidade e o WGSL amostra-a já deslocada), então os dois caminhos **divergiam**. Curado: o deslocamento corre ANTES de consultar a curva. E os três números de contorno (`curvature`/`steps`/`curve_offset`) passaram a ser gateados pelo modo que os lê — ele tinha dois knobs vivos e inertes ao lado | `curve_offset = 0` ⇒ a curva de hoje, ao bit |
| `field.remap` | idem | **`Clamp Min` e `Clamp Max` SEPARADOS** — C4D §B6 verbatim: *"Min / Max \| limites de saída + **Clamp Min** / **Clamp Max** separados"* | **NÃO** — um `bool` não exprime *"segure o piso, deixe o teto voar"*, e é exatamente o que a lei linear-space da linha do `combine` pede | omissão | **P2** | dois bools nascendo `1` ⇒ o `clamp` de hoje |
| `field.remap` | idem | **camada `Solid`** (o "Plain" da pilha) — C4D §B5: *"**Solid** (valor constante — o 'Plain' da pilha)"* | **SIM, exato** — `min = max = k, contour = None, strength = 1` ⇒ `mapped = k·multiplier` para todo `t`, constante | natureza | ⛔ | — |
| `field.remap` | idem | **`Mirror Graph`** — C4D §B6: *"bool \| display; disabled for Linear Fields, used for symmetric fields like Spherical"* | é **display** do editor de curva, não transferência | natureza | ⛔ | — |
| `field.remap` | idem | `steps` chega a **32**; C4D §B6: *"Steps \| [1..2³¹]"* | **NÃO** (sem `ParamHardMax`, o digitado para em 32) | omissão | ~~P2~~ ✅ **FECHADO 2026-08-23** | `ParamHardMax` só alarga ✅ **`16 777 215` (`2²⁴ − 1`), MEDIDO.** ⚠️ **E a entrada bate no mesmo muro pelo outro lado:** `t` é um `f32` em `[0,1]`, com ~`2²⁴` valores distintos, então nem o degrau nem quem o pisa resolvem mais que isso. *Dois recursos independentes a dar o mesmo número é o sinal de que o número é do problema, e não do instrumento.* |

---

## ESPÉCIES QUE FALTAM:

> A pergunta que o briefing manda fazer primeiro. O C4D tem um **catálogo** de *field objects*
> (§B4: Linear · Spherical · Box · Cylinder · Cone · **Torus** · **Capsule** · **Radial** ·
> **Random** · **Shader** · **Sound** · **Formula** · Python · **Group** · *objeto arrastado →
> Proximal*) mais **14 modifier layers** (§B5: Solid · Curve · Step · Quantize · Clamp ·
> Colorize · **Decay** · **Delay** · **Freeze** · Invert · Rangemap · Time · Formula · Proximal).
> Descontando o que é 3D-only (Cylinder/Cone) e o Python (n/a), sobram **~12 espécies de FONTE
> relevantes em 2D**. Nós temos **3 fontes** (`box` · `radial_sweep` · `index_range`) + o
> composer + o remap.
>
> ⚠️ **E o remap não é uma fonte a menos — ele DOBRA sete das 14 modifier layers num nó só**
> (Solid · Curve · Step · Quantize · Clamp · Invert · Rangemap). Isso é força da família, não
> lacuna, e vale escrever antes de contar buracos.
>
> ⚠️ **`field.index_range` não tem par no C4D** — é a *Range Falloff* da Cavalry, e nós a temos
> GPU-resident bit-exata. Uma espécie a mais que a referência de origem.

| espécie | referência CITADA | exprimível? (a cadeia tentada) | P |
|---|---|---|---|
| **`field.noise`** — o campo procedural | C4D §B4 field **Random**: *"generates random values for each clone or object point… Noise types editáveis"* (FRANDOM s22) · MOPs **Noise Falloff**: *"noise→falloff, animável, loopável, transform handle"* · doc 63 §2.1 marca **P0** | **NÃO — dois bloqueios independentes.** (1) `value.noise → motion.drive(?)`: o `drive` não tem canal `falloff` (`["X","Y","Rotation","Size","Opacity"]`), e **nenhum** nó do repo lê `Domain::Values` e escreve `falloff` (§1). (2) mesmo com a porta, `value.noise` é indexado — *"`frequency` scales the INSTANCE axis"* —, **não** tem `P`, logo não é um campo ESPACIAL | ✅ **FECHADO** (2026-08-12) — ⚠️ **o bloqueio (1) já estava MORTO e a célula não sabia:** o canal `Falloff` do `motion.drive` existe e escreve a coluna. O (2) era real e fechou com o param **`space`** (Index · World) no `value.noise`: em World ele amostra em `P` em vez de no ordinal, e o tempo continua a andar no mesmo eixo `x` (a referência pede *"noise→falloff, **animável**"* no mesmo fôlego). ⇒ **`value.noise(World) → motion.drive(Falloff)` É o campo de ruído** — composição, que é o desenho desta biblioteca, e não um nó novo. ⚠️ **Sem coluna `P` o modo World cai no ÍNDICE, não em zero** — a `identity` de um binding ausente É zero, e zero como POSIÇÃO daria a todo elemento o mesmo ponto: um campo que devolve **um valor só** (medido na mutação: `[-1, -1, -1, -1, -1, -1]`). O kernel ramifica no **`HAS_P`**, com paridade CPU×GPU sobre o ramo espacial. ⚠️ **Fica de fora, e é honesto:** o *transform handle* que a referência (MOPs) cita é o gizmo de canvas dos `field.*`, e uma composição não o herda ~~**P0**~~ |
| **`field.linear`** — a rampa num ângulo QUALQUER | C4D §B4 **Linear**: *"Length, **Direction (X±/Y±/Z±)**, Clip to Shape"* · Cavalry §Falloff Shape Type inclui **Linear** · doc 63 §2.1 **P0** | **PARCIAL, e só na HORIZONTAL** — `motion.falloff(shape = Linear)` **existe** e é a única forma linear da casa, mas os params dele são `shape·curve·center_x·center_y·radius·invert`: **não há `rotation`**, e o kernel ramifica em `dx` cru. Tentei `field.box(rotation) + invert` → dá platô com rampa **simétrica**, não gradiente monótono; tentei um `field.radial_sweep` gigante e distante (a aproximação clássica) → o `radius` **para em 40** (§2). ⚠️ **A cura mais barata não é nó novo: é `motion.falloff` ganhar o `rotation` que o `field.box` já tem** | ✅ **FECHADO** (2026-08-12) — param `rotation` (graus), gateado às formas que **têm direção** (Rect e Linear): um Círculo é isotrópico, girá-lo não move um texel, e o knob seria morto ali. `rotation = 0` reduz LITERALMENTE ao nó de sempre (`cos_sin_cycles(0)` é `(1, 0)` **exato**, então `ox·1 + oy·0` é `ox` em IEEE-754 — identidade estrutural, não tolerância). ⚠️ **O `trig.rs` é cópia VERBATIM do `field.box`**, com gate ESTRUTURAL a compará-los: os dois são campos espaciais que o artista gira, e um `30°` que significasse ângulos diferentes seria a falha de duas portas na forma mais quieta — nada na tela diria qual está certo. ⚠️ **O repo tem 21 cópias desse arquivo**; medido, os CORPOS são idênticos (só testes e docs diferem) ⇒ não há divergência hoje, há vinte e uma chances de uma. **Unificá-las é wave própria e está NOMEADA, não contrabandeada** ~~**P0**~~ |
| **`field.from_value`** (ou o canal `Falloff` no `motion.drive`) — **o destravador** | C4D §B4 **Shader** (*"qualquer shader/textura amostrado"*) + **Formula** (*"expressão f(x,y,z,t)"*) · MOPs **Texture Falloff** (*"textura projetada→falloff por luminância"*) + **Falloff From Attribute** | **NÃO — é o mecanismo do §1.** E o custo/benefício é assimétrico: `motion.luminance` **já** devolve a luma do `tint` como valor; `motion.expression`, `value.noise`, `value.curve`, `value.wave`, `value.attribute` e os outros ~23 `value.*` já existem. **Uma porta de escrita converte todos eles em fontes de campo de uma vez** — e fecha noise, textura, fórmula, áudio e atributo com um item só | ✅ **JÁ FECHADO** — reconferido contra o código em 2026-08-12: **o canal `Falloff` do `motion.drive` EXISTE**, inteiro (a row no seletor, o `ColumnBinding` de `falloff` em `ReadWrite`, o WGSL). Ele shipou na wave do `pulse.level` e a folha 08 já o tinha usado para REFUTAR o P0 do `motion.cull`; esta célula envelheceu sem que ninguém voltasse aqui. `value.* → motion.drive(channel = Falloff)` **é** a porta de escrita que ela pede ~~**P0**~~ |
| **`field.shape` / `field.spline`** — a GEOMETRIA como campo | Cavalry §Falloff: *"Shape Type … **Shape** (usa Input Shapes)"* + *"Path Mode \| **Filled Path** (força cheia dentro) · **Path Edges** (decai da borda, com Distance)"* · MOPs **Spline Falloff** / **Object Falloff** (*"Area of Influence"*) · C4D §B4 *"objeto arrastado \| mesh vira campo de DISTÂNCIA (proximal)"* · doc 63: `field.spline` **P0**, `field.shape` **P1** | **NÃO, mas a fiação está PROVADA** — o `field.combine` já tem duas portas, e o `motion.kaleidoscope` estreou o `StreamOp::SourceRows` / `ColumnAccess::SourceRead` (ler uma porta-template de comprimento desacoplado, ADR-0136). O que falta é o kernel de distância-com-sinal a um caminho; nenhum campo tem porta-template hoje | **P1** |
| **`field.delay` · `field.decay` · `field.freeze`** — os modificadores com ESTADO | C4D §B5, a joia da pilha: *"**a pilha de campo tem ESTADO entre frames**: um modificador de pilha pode ser uma equação diferencial, não só uma função"* (FLDELAY modos **Spring**/**Smooth**; FLDECAY meia-vida) · doc 63 §2.1 **P1** | **NÃO** — `motion.spring` age num CANAL de transform (está entre os *leitores* de `falloff`, não os escritores) e `pulse.sample_hold` segura um VALOR, não uma máscara. Mesmo bloqueio do §1 | **P1** |
| **`field.audio`** — as bandas como campo | MOPs **Audio Falloff**: *"bandas de frequência→falloff, gain por banda, Auto Distribute"* · C4D **Sound field** (*"o motor do Sound Effector como field reutilizável"*) · Cavalry §Sound (*"**Use Index Context (bandas→duplicates!)**"*) | **NÃO** — o mesmo bloqueio, **mais** a ponte que não existe: o C4D dump é explícito sobre nós — *"temos módulo de áudio inteiro e **nenhuma ponte áudio→motion**"*. Cross-line (`ph2d-audio-spectral` já tem a FFT) | **P1** |
| **`field.spread`** — a "infecção" | MOPs **Spread Falloff** (fetchado): *"**Spread (ANIMÁVEL — frente de crescimento)** · Falloff Width (fade da frente) · métrica: conectividade ou raio · pontos-semente por grupo"* · doc 63 §2.1 **P1**, *"assinatura mograph"* | **NÃO** — precisa de vizinhança por raio, que nenhum `field.*` tem. ⚠️ **Mas o substrato está pronto e é nosso diferencial:** a **grade espacial na GPU** (ADR-0140, `scan → grade → boids → collide`) já responde "quem está perto de quem" no device | **P1** |
| **`field.torus` (anel) e `field.capsule` (stadium)** | C4D §B4: *"Cylinder / Cone / **Torus** / **Capsule** \| dimensões da primitiva + Direction"* | **Anel: SIM a 3 nós** (ver a linha `inner_radius` do `radial_sweep`). **Stadium: NÃO** — nem box nem disco dão as duas calotas, e não há `min`/`max` de dois campos que produza um retângulo de pontas redondas com UMA rampa coerente. ⚠️ mas a `ph2d-physics-ecs` **já tem** `capsule_vertices`/`Stadium` como precedente de forma | **P2** |
| **`field.group`** (a sub-pilha) | C4D §B4/§C2: *"Group \| contêiner com pilha filha (compose e reuse)"* + *"Sub-fields: parâmetros de uma camada podem ter a própria mini-pilha"* | **SIM, exato** — o grafo tem **subgrafos** (doc 57) e **grupos com bypass como unidade** (`H` num card, integrado 2026-08-02). É o `Group` do C4D com um nome melhor | ⛔ |
| **camada = REFERÊNCIA reusável** (o mesmo field em N listas) | C4D §C3: *"Camada = referência, não cópia: o mesmo Field object em N listas; animar a esfera move todos os efeitos juntos"* | **SIM, exato** — a saída de um `field.*` é um socket; **ligá-la a N consumidores é literalmente a mesma instância**. É o item que a Maxon precisou de uma QUEBRA (R20) para conseguir, e que o idioma de grafo dá de graça | ⛔ |

---

## SUPERAR:

Derivado do que **só nós temos** (lei 8 do plano 89):

1. **A porta `value → falloff` é estritamente MAIOR que o catálogo do C4D, e não é uma
   espécie: é uma classe.** Lá um *field object* é um TIPO FECHADO — para inventar um campo
   novo você escreve **Python** (§B4). Aqui o campo é uma **COLUNA**, então uma única porta de
   escrita transforma os ~23 `value.*` + `motion.expression` + `motion.luminance` em fontes de
   campo **de uma vez**, cada uma já GPU-resident e já dirigível por param (doc 58). O C4D tem
   *Shader field*, *Formula field*, *Sound field* como três objetos distintos; nós teríamos
   **um** nó e o resto do catálogo já construído.
2. **O campo com ESTADO que REBOBINA exato.** O Delay/Decay/Freeze é a joia da pilha do C4D e
   é também o que a comunidade odeia nela (§D: *"performance de pilhas de fields profundas com
   Delay (estado por ponto)"*) — porque lá é um acumulador que não sabe voltar. Nós temos
   `Cook::checkpoint`/`CheckpointRing` (GGPO save/load/advance, scrub bit-exato, M2.N2), então
   **um `field.decay` nosso scrubba para trás bit-a-bit** — o mesmo argumento que o gabarito do
   plano 89 usa para o `inherit_velocity` do emissor (forma fechada onde os outros integram).
3. **A curva de transferência DIRIGIDA pelo próprio grafo.** O C4D anima a curva do Remapping
   deslizando-a no eixo X (*Spline Animation Speed*) — um número interno, na CPU. Aqui o
   `LutSpec` já leva a curva ao **device** com a `fill` como fn-pointer no crate do nó, e **um
   param é uma aresta** (doc 58): um `curve_offset` de uma linha faz um `value.lfo` /
   `pulse.beat` **animar a função de transferência de um campo**, no device, bit-exato sob
   scrub. Nenhuma das três referências tem "a curva do remap dirigida por um sinal do grafo".
4. **A infecção na GRADE.** O `field.spread` do MOPs é a assinatura mograph que ninguém copia
   porque a vizinhança é cara. Nós já portamos `scan → grade espacial → vizinhos` para a GPU
   (ADR-0140) por causa de boids/collide: **a peça cara do `field.spread` já está paga.**
5. **A pilha é VISÍVEL.** O §D do dump nomeia como odiado *"a ordem-importa da lista de
   effectors ser **invisível** para iniciantes"* — no nosso idioma a ordem É o fio na tela.
   Já superamos; vale não perder isso empurrando a pilha para dentro de um nó.

---

## CERCAS:

Decisões já registradas que encontrei (grepadas antes de propor):

1. **`field.remap` REESCREVE o `falloff`, não multiplica** — doc-comment: *"it does NOT multiply
   into it; a remap is not a field, it is a transfer function"*. Toda proposta de "strength
   por-campo via remap" tem de honrar isto (é por isso que a cadeia só é exata como 1º campo).
2. **Color Remap / Direction DIFERIDOS** — doc 63 §0.1: *"v1 = só o canal value/`falloff`"*
   (§6.3). Não é gap; é escopo declarado.
3. **`field.index_range` NÃO tem Coordinates** — doc 63 §0.1: *"mascara por RANK, não por
   posição; rotação/posição não têm sentido num posto. É **categoria**, não gap"*.
4. **O Remapping é fatorado a JUSANTE (D1)** — e cada field o documenta no próprio doc-comment
   (*"Remapping is a DOWNSTREAM node (D1)… senão parece gap"*, doc 63 §0.1).
5. **`field.radial_sweep`: UM `soft` adimensional para as duas bordas** — doc-comment: *"uma
   knob adimensional, porque as duas bordas vivem em unidades diferentes, degrees vs world"*.
6. **A TRANSFERÊNCIA fica FORA de grupo de propósito** no `field.remap` — e há gate
   (`selected_field_remap_yields_an_interactive_curve_row`) que já corrigiu uma 1ª versão que a
   sepultava numa seção.
7. **HR-5 transcendental-free é estrutural nesta família** — o `pseudo_angle` (diamond angle)
   existe para não chamar `atan2` por instância; `cos_sin_cycles` é uma parábola corrigida
   compartilhada CPU/WGSL. Espécie nova respeita isso ou não é bit-exata.
8. **As crates-nó entram em `[dev-dependencies]` da `ph2d-gpu-cook`**, nunca em
   `[dependencies]` (paridade CPU×GPU; machete-safe).
9. **O bypass/mute (`H`) já É o `Enabled` da Cavalry** — record `y`, semântico, no fingerprint.

---

## O DOC 63 ERROU EM:

Envelheceu **nos dois sentidos** (o §1 do plano 89 avisa que item marcado FALTA que já existe
custa tanto quanto o inverso):

1. **§2.1 marca os CINCO como P0 a construir** (`field.box`/`radial_sweep`/`index_range`/
   `combine`/`remap`) — **os cinco EXISTEM** desde 2026-07-25, GPU-resident bit-exatos. A
   tabela manda construir o construído.
2. **§46 é histórico** (*"Estado (2026-07-24): 3 nós landaram… Próximo: o gizmo · radial_sweep ·
   remap + Curve"*) — os três "próximos" landaram.
3. **§1.4: *"Zero widgets de curva/gradiente"*** — **falso hoje**: `ParamWidget::Curve` existe,
   o `field.remap` o usa, a curva vai ao **device** por LUT, e o editor de gradiente landou
   (doc 85).
4. **§1.1: *"Falloff raso — 1 nó, 1 forma, 6 params, 1 canal"*** — a primeira metade envelheceu
   (6 nós, 4 formas, 37 params); **a última palavra continua verdadeira**: *1 canal*.
5. **§1: *"87 nós / 318 params"*** — o censo do doc 88 diz **118 nós / 411-420 params**.
6. **§2.1 especifica `field.combine` com OITO modos** (*"add/sub/mul/screen/min/max/overlay/
   normal"*) — e foi exatamente o que se construiu. **Mas a referência tem mais**: o C4D tem
   **nove** (falta o `Clip`) e o MOPs tem **`Average`**. O doc-63 já era um subconjunto do
   dump, e o subconjunto virou o nó — é o modo de falha que a D13 existe para impedir,
   acontecendo dentro do próprio documento que a enuncia.
7. **§198: *"`motion.falloff` → dissolve na família `field.*` (D1); o nó atual vira
   alias/compat"*** — **não aconteceu, e é bom que não tenha**: ele segue vivo, é o **ÚNICO nó
   com a forma Linear** (que a família `field.*` não tem), e é **o único campo espacial sem
   gizmo** (o `field_gizmo::spec_for` só reconhece `field.box` e `field.radial_sweep`). A nota
   descreve como *legado* o nó que hoje carrega a espécie P0 que falta.

---

## Contagem

**Contagem (DERIVADA, reconciliada em 2026-08-19):** 19 linhas — **P0 = 0** · **P1 = 0** · **P2 = 2** · ✅ fechadas **9** · ⛔ recusadas/refutadas **8**. ⭐ **Re-medida em 2026-08-23 (bloco Z, [doc 91](../91_os_tetos_que_ninguem_mediu.md)): TRÊS fecharam de uma vez, e o que as juntava era a espécie do teto** — nenhuma das leis satura, então o que as limita é o `f32`, e o número é DERIVADO do `step` do slider a cada corrida. ⚠️ Entre elas o **NEUTRO** que dois doc-comments desta folha prometiam e a UI recusava (*"a box larger than the scene"* num campo que digitava até 40). ⚠️ **As quatro que fecharam vieram numa wave só porque DUAS partilhavam uma causa** — o `Average` era inexprimível *por causa* do clamp inline, e o clamp era o defeito. ✅ **E os 2 `P1` estruturais FECHARAM em 2026-08-21**, na mesma wave: o `field.index_range` ganhou `key = Attribute` + a porta `attr` (o posto **sem reordenar**), e o `field.remap` ganhou `curve_offset` (deslocar a curva, com wrap). ⚠️ Das duas, uma trouxe uma **medição que ENCOLHEU o item**: o *Auto Range* citado ao lado do rank já é exprimível (`value.attribute → value.normalize → motion.drive(Falloff)`), então só o posto era gap. **A folha está a ZERO `P1`.** Antes: 19 linhas · P1 = 6 · ✅ 0, reconciliadas no grupo M em 2026-08-16.

Re-medir: `python3 "docs/Motion Nodes/ferramentas/placar_conferencia.py"` — ⚠️ **esta linha é DERIVADA da coluna `P` da tabela acima; não a edite à mão** (a contagem desta conferência envelheceu SEIS vezes, e a folha 13 chegou a contradizer a própria prosa três parágrafos abaixo).

⚠️ **Esta linha dizia `P0 = 3` até 2026-08-13, com os três já RISCADOS na tabela acima** —
`field.noise` · `field.linear` · e a porta `value → falloff`, que era o destravador e virou o canal
`Falloff` do `motion.drive`. Uma contagem que sobrevive ao próprio fechamento faz a próxima
varredura propor o que já existe, que é o que a §0 do CLAUDE.md manda reconferir em quem move o
número.

⚠️ **A leitura honesta da contagem:** os três P0 e três dos P1 (`shape/spline`,
`delay/decay/freeze`, `audio`) **não são seis itens** — são **um mecanismo** (§1) mais dois nós.
Priorizar por linha da tabela dispersaria uma wave que cabe num commit de fundação.
