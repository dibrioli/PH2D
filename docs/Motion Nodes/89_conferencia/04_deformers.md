# 89 · Conferência — Família 4: DEFORMERS (7 nós)

**Data:** 2026-08-09 · **Linha:** `line/motion-value` · **Briefing:** [doc 89 §3](../89_plano_conferencia_dos_nos.md) · **Leis:** §4
**Params lidos do `MANIFEST`** (não do doc), **em 2026-08-09**: `bend` 3 · `twist` 3 · `spherize` 3 · `four_point_warp` 8 · `kaleidoscope` 4 · `slit_scan` 1 · `spline_wrap` 10.

> ⚠️ **DOIS já estavam conferidos e NÃO foram refeitos:** `motion.spherize` ([doc 88 §9.1](../88_plano_parametros_nos_unidades_e_slider.md)) e `motion.slit_scan` (§9.2). Eles aparecem na tabela **só com o resíduo** que esta conferência achou depois do fato.
> ⚠️ **`motion.lattice` NÃO é desta família** (é `NodeUiCategory::Source`, um gerador de grade) — doc 88 §9 já o tirou daqui, e ele **não** foi conferido nesta linha.

---

## §1 — O achado ESTRUTURAL da família (ele decide metade da coluna "exprimível?")

O contrato de campo desta casa é: todo `field.*` **multiplica** a coluna `falloff`, e o deformer a lê para
misturar *deformado ↔ original* por elemento — `p_out = p + (p_def − p) · f`. Verificado: `field.box`,
`field.combine`, `field.index_range`, `field.radial_sweep` e `field.remap` **todos** escrevem `falloff`.

**Mas a mistura por LERP não é a mesma coisa que atenuar o PARÂMETRO, e a diferença é geométrica:**

| Classe do deformer | O que `lerp(p, p_def, f)` faz | Vale como "perfil"? |
|---|---|---|
| **RADIAL** (`spherize`) | `p` e `p_def` estão **na mesma reta** que sai do centro ⇒ `c + d·(1 + f·amount·(1−t²))` | ✅ **SIM** — o `f` multiplica o `amount` exatamente |
| **ROTACIONAL** (`twist`, `kaleidoscope`, e o arco do `bend`) | `p` e `p_def` estão num **arco**; o lerp corta pela **CORDA** ⇒ o raio encolhe para `r·cos(θ/2)` | ❌ **NÃO** — mascarar uma rotação **encolhe o layout**, não a suaviza |

⇒ **Para o `spherize`, "perfil/taper/falloff" JÁ é exprimível** (`motion.falloff` Circle + `curve`, ou
`field.remap` com **Curve**, → `falloff` → spherize) e não vira gap.
⇒ **Para o `twist` e o `kaleidoscope`, NÃO é** — um perfil de ângulo pedido pela referência não tem
composição no catálogo, porque a única porta (o `falloff`) responde a outra pergunta.

*É a mesma doença que a `line/Painter` chama de "duas coisas que devem concordar sobre um fato,
discordando": aqui o `falloff` promete "quanto deste deformer", e para os rotacionais ele entrega
"quanto do deslocamento", que só coincide em deformação infinitesimal.*

---

## §2 — A tabela

| nó | params hoje | falta (referência CITADA) | exprimível? (a cadeia tentada) | natureza/omissão | P | default que reduz |
|---|---|---|---|---|---|---|
| `motion.bend` | 3 (`angle`·`pivot_x`·`pivot_y`) + input `amount` | **DIREÇÃO da dobra** — C4D Bend/Twist Object Properties: *"**Angle** defines the direction of deformation. **0° is the deformer's local X axis**"* ([help.maxon.net OTWIST-ID_OBJECTPROPERTIES](https://help.maxon.net/c4d/r21/us/html/OTWIST-ID_OBJECTPROPERTIES.html)); Blender *Simple Deform ▸ Bend* tem **Axis**. Nós dobramos **só no X** (o doc-header diz *"along the X axis"* como FATO) | **SIM, a 3 nós — e a cadeia ÓBVIA é uma armadilha.** `motion.rotate` **NÃO** serve: ele soma no atributo `rot` (a base do sprite), **não move `P`** — verificado no header dele. Quem move posição em torno de um pivô é **`motion.orbit`** ⇒ `orbit(pivot,+θ,speed=0)` → `bend(pivot,angle)` → `orbit(pivot,−θ,speed=0)`. ⚠️ **Preço:** 3 nós por um knob **e** `motion.orbit` é `Effect::Temporal` (lê o playhead mesmo com `speed=0`) ⇒ a sub-árvore inteira troca de classe de efeito | **omissão** | **P1** | `direction` (deg) `= 0`. ⚠️ **Reduz LITERALMENTE, e conferi a aritmética do `trig.rs`:** `sin_cycles(0)=0` e `sin_cycles(0.25)=1` **exatos** (f=0 ⇒ p=0; f=0,25 ⇒ u=0,5, p=4·0,5·0,5=1, `0.225·(1−1)+1=1`) ⇒ a rotação por 0 é `x·1−y·0` = identidade bit a bit |
| `motion.bend` | idem | **Mode: Limited / Within Box / Unlimited** (C4D, ibid.) e **Limits lower/upper** (Blender Simple Deform) — *qual FATIA da extensão dobra* | **PARCIAL.** `field.box` (que tem `rotation`) → `falloff` → bend **é exatamente o "Within Box"** (fora ⇒ identidade). O **"Limited"** (fora **acompanha rigidamente** a ponta dobrada) **NÃO** é exprimível: `falloff=0` deixa o elemento **onde estava**, não o CARREGA. ⚠️ Num stream de instâncias não há conectividade para rasgar, então a diferença é *carona rígida*, não *rasgo* — visível, menos grave | omissão (parcial) | **P2** | n/a (o "Within Box" já existe pela composição) |
| `motion.bend` | idem | **Keep Y-Axis Length** como TOGGLE — `BENDOBJECT_KEEPYAXIS` ([Maxon SDK `obend`](https://developers.maxon.net/docs/Cinema4DPythonSDK/classic_resource/object/obend.html), o par com `BENDOBJECT_STRENGTH`). Nosso comprimento de arco é **preservado por construção** (header: *"the bend never stretches the layout"*) ⇒ o modo *esticar* é inalcançável | **NÃO.** `motion.scale` escala a coluna `size`, não `P`; `motion.transform` escala `P` **uniformemente em torno da origem**, que não é "esticar ao longo do arco" | omissão de **baixo valor** (é o modo feio; o default de C4D é preservar) | **P2** | `keep_length = 1` ⇒ o caminho de hoje |
| `motion.twist` | 3 (`angle`·`pivot_x`·`pivot_y`) + input `amount` | **Size / a EXTENSÃO do twist** — C4D Twist: **Size [XYZ]** + **Mode** + **Fit to Parent** ([ibid.](https://help.maxon.net/c4d/r21/us/html/OTWIST-ID_OBJECTPROPERTIES.html)); Blender Simple Deform ▸ Twist: **Limits lower/upper**. Nós **derivamos o aro por redução** (`r_max`, `ReduceOp::Max`, todo frame) e **não há knob** | **NÃO — e nada no grafo pode sobrepor uma REDUÇÃO.** Ela é computada dentro do kernel, sobre `P` da porta 0; nenhum nó escreve `r_max`. A única saída é uma **isca** no raio desejado — que **muda a CONTAGEM e a arte**, o mesmo argumento que a §9.1 usou para matar a saída análoga do `spherize` | **omissão** | **P1** | `radius = 0` = *"auto (o aro medido)"* ⇒ ramo para o `r_max` de hoje, byte-idêntico (o sentinela-em-float, o idioma que o `mode` do `motion.cull` já usa) |
| `motion.twist` | idem | **Falloff tab** (C4D) = o **perfil radial do ÂNGULO** | **NÃO — e a razão é a §1.** `field.remap` (Curve) → `falloff` → twist é um **lerp posicional**: mascarar uma rotação por `f` põe o ponto na **CORDA**, com raio `r·cos(θ/2)` ⇒ o layout **encolhe** em vez de destorcer. A referência pede um perfil sobre o `deg`, e nenhuma porta o alcança | **omissão** (contrato de campo cego a rotação) | **P1** | perfil `Linear` ⇒ `r/r_max` de hoje |
| `motion.spherize` ✅ *(conferido §9.1)* | 3 (`radius`·`offset_x`·`offset_y`) + input `amount` | **LENTE ELÍPTICA** — AE *Bulge*: **Horizontal Radius** *e* **Vertical Radius** ([helpx.adobe.com ▸ Distort effects](https://helpx.adobe.com/after-effects/desktop/apply-effects-and-animation-presets/list-of-effects/distort-effects.html)); Photoshop *Spherize*: **Mode Normal / Horizontal only / Vertical only**. Nós temos **um** `radius` ⇒ a lente é **sempre um círculo**. ⚠️ **A §9.1 deu à lente um LUGAR e não uma FORMA** | **NÃO.** `field.box` tem largura/altura/rotação, mas mascara a **mistura**; o deslocamento continua **radial em torno do centro**. Uma lente elíptica desloca **anisotropicamente** — nenhuma máscara produz isso | **omissão** (resíduo da §9.1) | **P1** | `radius_y = 0` = *"siga o `radius`"* ⇒ círculo de hoje |
| `motion.spherize` | idem | **Taper Radius** (AE Bulge: *"controls the shallowness of the sides"*) — o perfil, hoje soldado no quadrático `1−t²` | **SIM, a 1–2 nós — e é a §1 a favor.** `motion.falloff` (**shape Circle**, `curve` Linear/Quad/Smooth/Smoother, `center`, `radius`) → `falloff` → spherize; ou `field.remap` com **Curve** para um perfil arbitrário. Como o spherize é **radial**, `p` e `p_def` ficam **na mesma reta** e o lerp **multiplica o `amount` exatamente** | exprimível | **P2** | n/a |
| `motion.spherize` | idem | **Pinning** (AE Bulge: *"prevents the edges of the layer from bulging"*) | **JÁ EXISTE** — `r ≥ radius ⇒ identidade` é o comportamento do kernel (`if (sp_r >= 1e-6 && sp_r < sp_rmax)`) | ⛔ **não é gap** — escrito aqui para ninguém o "acrescentar" | — | — |
| `motion.four_point_warp` | 8 (`tl_dx`…`bl_dy`) + input `warp` | **ARESTAS CURVAS** — AE *Bezier Warp*: *"a closed Bezier curve along the boundary… four segments. Each segment has three points (a vertex and two tangents)"* ⇒ **4 vértices + 8 tangentes**. Nosso Corner Pin mantém **retas retas** por construção (homografia de Heckbert) | **NÃO.** `motion.lattice` é **gerador de grade** (`Source`), não gaiola sobre um stream existente (doc 88 §9 o reclassificou); um `field.*` não entorta uma aresta; `spline_wrap` dobra ao longo de **UMA** curva, não da fronteira de um quad | omissão | **P1** | ⚠️ **NENHUM — e é isto que decide o desenho.** Tangentes em ⅓/⅔ degeneram a Bézier na reta, mas o patch de **Coons** com arestas retas é **BILINEAR**, e o próprio header diz que o bilinear *"bows them"* ⇒ **não reduz à homografia**. ⇒ **é NÓ NOVO (`motion.bezier_warp`), não param deste** |
| `motion.kaleidoscope` | 4 (`segments`·`reflect`·`pivot_x`·`pivot_y`) + input `spin` | ⚠️ **O `falloff` é IGNORADO** — grep: **zero** ocorrências de `falloff` na crate inteira. É o **único** deformer da família que não lê o contrato de campo ⇒ um `field.*` a montante é **silenciosamente inerte** sobre a transformação dele. *(A referência do contrato é a própria casa: os outros 6 o leem.)* | **NÃO** (não há o que compor: o canal existe e o nó não escuta). ⚠️ **Mas o desenho NÃO é óbvio, e a §1 é o motivo:** um lerp posicional **encolheria as cópias** para o centro; o `f` teria de escalar o **ÂNGULO** (`s·(1/k)·f`). E o neutro é **ambíguo** — `f=0` põe as `k·n` cópias **coincidentes**, o que não é a identidade em **CONTAGEM** | **inconsistência de contrato** (não é "param faltando") | **P2** — *decisão de design, com o mecanismo escrito* | `falloff` ausente ⇒ `1.0` (a identidade que as bindings dos irmãos já declaram) |
| `motion.kaleidoscope` | idem | **A WEDGE de origem** — AE *CC Kaleida* (Center · **Size** · Mirroring · Rotation): um caleidoscópio real **dobra a fonte numa cunha**; nós replicamos a fonte **INTEIRA** `k` vezes, então fonte larga faz cópias invadirem as vizinhas | **SIM, a 2 nós — verificado peça por peça.** `field.radial_sweep` (cunha angular → escreve `falloff`) → `motion.cull` (**`mode = 1` Falloff**, mantém `falloff ≥ amount`) → `kaleidoscope`. Os dois existem e casam | por natureza (a fatoração é o idioma do grafo) | **P2** — *e deve ser ESCRITO no doc-comment*, o padrão §9.2 | n/a |
| `motion.kaleidoscope` | idem | **Setor (Start/End Angle)** — C4D Cloner *Radial* / Cavalry Duplicator: as cópias num leque em vez do giro cheio | **NÃO.** `spin` gira o padrão inteiro; `cull` a jusante **APAGA** cópias (deixa buracos), não as **comprime** num setor | omissão — ⚠️ **mas é fronteira de família**: leque é trabalho do *Cloner/duplicator*, não do caleidoscópio | **P2** | `start=0, end=360` ⇒ `s·(1/k)` exato |
| `motion.kaleidoscope` | idem | **Mirroring com N modos** (CC Kaleida: Unfold / Starfish / Flower / …) contra o nosso `reflect` de 2 estados | ⛔ **RECUSADO COM MOTIVO** — no domínio de **instâncias** o eixo com significado é **Cₙ vs Dₙ** (girar × girar-e-espelhar), que é exatamente o `reflect`; os demais modos do CC Kaleida são **dobras de coordenada de RASTER**, sem referente num stream | por natureza | ⛔ | — |
| `motion.slit_scan` ✅ *(conferido §9.2)* | 1 (`lag`) | **Time Displacement** (AE) desloca o **PIXEL inteiro**, cor incluída; nós atrasamos **só `P`** | ⛔ **RECUSADO — e a cerca JÁ ESTAVA ESCRITA**, o que a §9.2 não citou: *"What is delayed is POSITION. The appearance columns stay live: a slit-scan is a geometric shear of time, and echoing whole rows — colour and all — is what `motion.trail` is for."* ⇒ a capacidade tem **dono**, e é o `trail` (7 knobs desde 2026-08-08) | por natureza | ⛔ | — |
| `motion.slit_scan` | idem | *(eixo · direção · forma da rampa · Time Resolution)* | ⛔ **Os quatro já foram fechados pela §9.2** com mecanismo (`motion.sort` a montante · o `descending` dele · `field.*`+`field.remap` Curve sobre o `falloff`, que o nó **consome** — `Coupling::Consumes("falloff")` registrado · Time Resolution é amostragem de raster) | por natureza | ⛔ | — |
| `motion.spline_wrap` | 10 (`height_scale`·`offset`·`p0x`…`p3y`) + input `amount` | **From / To** — C4D Spline Wrap: *"Define the spline region where deformations occur, expressed as percentages"* ([help.maxon.net OMOGRAPH_SPLINEWRAP-ID_OBJECTPROPERTIES](https://help.maxon.net/c4d/s22/us/html/OMOGRAPH_SPLINEWRAP-ID_OBJECTPROPERTIES.html)). Temos `offset` (desliza) e **nenhuma extensão** | **NÃO.** Nada no catálogo re-escala o mapeamento `x → u ∈ [0,1]`; `field.*` mascara a mistura, não a parametrização | **omissão** | **P0** — *é a animação de REVELAÇÃO (write-on) pela qual este nó existe, e o knob mais usado do Spline Wrap* | `from = 0, to = 1` ⇒ `u` intacto ⇒ identidade |
| `motion.spline_wrap` | idem | **A ORIENTAÇÃO não segue a curva** — em C4D a geometria acompanha o frame; o nosso `out.set("P", …)` é a **única** escrita (verificado) ⇒ um sprite embrulhado num S **mantém a rotação original** | **NÃO — e não há precedente no catálogo.** Verificado: **`motion.distribute_curve` também não escreve `rot`**, e `motion.look_at` orienta para um **ponto-alvo**, não para uma tangente carregada pelo stream. Nenhum nó converte tangente→`rot` | ✅ **FECHADO** — `follow_rotation`. ⚠️ **A refutação desta célula já era falsa quando alguém foi construir:** o `motion.distribute_curve` ganhou `align` em `dfd7bf895`, **no mesmo dia desta folha**, então o precedente existe e a implementação o ESPELHA (o `trig.rs` é cópia verbatim). ⚠️ E não há geometria nova: o `frame_at` já devolvia a tangente e o wrap ligava-a a `_t`. ⚠️ **SOMA** (o irmão faz `set` porque é FONTE; este é modificador sobre um layout que já pode estar orientado) e honra a **MESMA máscara** que a posição — meio-embrulhado é meio-virado | ✅ | `follow_rotation = 0` ⇒ `rot` **COPIADO** pelo laço de sempre, não escrito com o valor de antes ⇒ byte-idêntico por ESTRUTURA |
| `motion.spline_wrap` | idem | **Mode: Fit Spline / Keep Length** (ibid.) — o nosso **sempre** normaliza a bbox sobre `u ∈ [0,1]` (*Fit*) | **NÃO** (mesma razão do From/To: a parametrização não é alcançável de fora) | omissão | **P1** | `mode = Fit` ⇒ hoje |
| `motion.spline_wrap` | idem | **Size / Spline Size** (graph) — o *taper* ao longo da curva; e **Size Strength** | **PARCIAL.** `motion.scale` + um `field.*` rampa o `size` — mas a rampa é em **espaço de MUNDO**, e o pedido é ao longo do **COMPRIMENTO DE ARCO**; num S os dois divergem | omissão (parcial) | **P2** | perfil plano ⇒ hoje |
| `motion.spline_wrap` | idem | **Banking** (*"Rotates the object around its longitudinal axis"*) · **Rail** · **Up Vector** · **Rotation from Rail** · **Scale from Rail** (ibid.) | ⛔ **RECUSADO COM MOTIVO — é GEOMETRIA, não escopo:** *banking* é rotação **em torno da tangente**, um eixo que **não existe** num stream planar; Rail/Up Vector são a construção de frame 3D que ele serve | por natureza | ⛔ | — |
| `motion.spline_wrap` | idem | **A curva como ENTRADA** (o documento vetorial), em vez de 8 params | ⛔ **CERCA DE CHESTERTON — [doc 28](../28_distribute_curve_spline_wrap_nota_adr.md):** *"a curva do documento vetorial (deferido) vs uma curva authored no nó (self-contained). Esta fatia faz a segunda"*, e o deferimento tem motivo escrito (*"cross-module; crate satélite que só LÊ o contrato vetor"*) | decisão registrada | ⛔ *(não re-propor)* | — |
| `motion.spline_wrap` | idem | **Axis** (±X/±Y/±Z — qual eixo do layout mapeia na curva) | **SIM**, pela mesma conjugação do `bend` (`motion.orbit` ±θ), com o mesmo preço (3 nós + `Temporal`) | omissão | **P2** | `axis = X` ⇒ hoje |

**Contagem:** **P0 2** · **P1 6** · **P2 8** · **⛔ 6** (5 recusados com referência/cerca + 1 "não é gap").
**Refutados por composição:** **4** — o *taper* do spherize · a *wedge* do kaleidoscope · a *direção* do bend · o *axis* do spline_wrap (os dois últimos exprimíveis **mas caros**, P1/P2 pela régua da §7).

---

## `SUPERAR:`

O que **só nós** temos aqui é que **os deformers desta família cozinham 100% no device** com o canal
`reduces()` (`reduce → broadcast → map`), e que a redução é **declarada como metadado**, não escondida
no kernel. Nenhuma referência 2D tem isso (Cavalry e C4D são CPU). Três coisas ficam baratas para nós e
caras para todos:

1. **O deformer que mede o ASSUNTO, não o mundo — e diz QUAL medida usa.** C4D resolve *"que tamanho tem
   a deformação?"* com **Fit to Parent**: um botão que **congela** a bbox do pai no momento do clique. O
   nosso `r_max`/bbox é re-medido **todo frame, no device, de graça**. ⇒ um `fit` de **três** estados —
   `Live` (a redução, hoje) · `Frozen` (mede uma vez e pina) · `Manual` (o `radius` do P1) — é **um `if`
   sobre um número que a GPU já produziu**, e entrega ao mesmo tempo o *Fit to Parent* deles **e** o
   comportamento que eles não têm (um twist que acompanha uma nuvem viva sem afrouxar). *A capacidade
   inteira é a redução que já roda, mais um enum.*

2. **O perfil radial/angular como LUT no device** — a `line/motion-nodes` já levou a **curva ao device**
   por `luts()` (o `field.remap`, 2026-07-25). ⇒ o *Falloff tab* do C4D e o *Taper Radius* do AE (que são
   **curvas por-deformer**) não custam um passe novo: são **a mesma LUT** que o `field.remap` já sobe,
   lida por-elemento dentro do deformer — e isso é justamente o que resolve o buraco da §1 **sem**
   inventar um segundo canal de campo. *A referência tem a curva; nós temos a curva **e** o device.*

3. **A composição de deformers é EXATA e reversível, porque é um GRAFO.** Um `bend` conjugado por dois
   `orbit` é, hoje, uma cadeia de três nós que **funciona** — em C4D isso é hierarquia de objetos e em AE
   é impossível. O que falta não é a capacidade: é o **custo**. ⇒ o alvo honesto de "superar" aqui não é
   um param novo, é o **`motion.orbit` puro** (o `Effect::Temporal` dele é o que envenena a cadeia: com
   `speed = 0` ele **não lê o playhead** e mesmo assim carimba a sub-árvore inteira como temporal).
   *Corrigir isso torna a conjugação barata e devolve o eixo de TODO deformer de uma vez — um conserto,
   sete nós.*

---

## `CERCAS:`

1. **`motion.spline_wrap` — a curva é AUTHORED no nó, de propósito** ([doc 28](../28_distribute_curve_spline_wrap_nota_adr.md)): *"a curva do documento vetorial (deferido) vs uma curva authored no nó (self-contained). Esta fatia faz a segunda"*. **Não re-propor** uma porta de curva sem tratar o deferimento cross-module.
2. **`motion.slit_scan` — atrasa `P` e SÓ `P`**, com o dono da alternativa nomeado no próprio header: *"echoing whole rows — colour and all — is what `motion.trail` is for."*
3. **`motion.spherize` — o offset é RELATIVO ao centroide, não absoluto** (§9.1, três razões escritas). Um `center_x/center_y` absoluto **arrancaria a lente do assunto em todo documento já autorado**.
4. **`motion.bend`/`twist`/`kaleidoscope` — `pivot_x`/`pivot_y` significam coordenada ABSOLUTA de mundo**; a §9.1 recusou reusar o nome no spherize por isso. Qualquer param novo de posicionamento nesta família herda essa distinção.
5. **A trigonometria é o polinômio da CPU, portado operação por operação** (`bend`, `twist`, `kaleidoscope`): *"calling the device's real `sin` here would not be a tighter ε, it would be a **different curve**"*. Um param de ângulo novo não pode chamar `sin` nativo no WGSL.
6. **O piso `MIN_RADIUS`/`MIN_ANGLE_RAD` pertence ao CONSUMIDOR, não à redução** (`twist`: *"The MIN_RADIUS floor belongs to the CONSUMER"*) — é guarda sobre *o uso* (uma divisão), não sobre *o medido*.
7. ⚠️ **Uma cerca que NÃO existe, e devia:** o teto de `lag` do `slit_scan` é **32** (só `ParamUiHint.max`, **sem `ParamHardMax`**) e **não há medição ao lado** — o §0 do CLAUDE.md exige que todo teto diga *de que recurso ele é* (aqui: o comprimento do anel de histórico) com a tabela. **Não é gap de param; é dívida de medição**, e cai na varredura de tetos do doc 88 §B2.

---

## `O DOC 63 ERROU EM:`

1. ⚠️ **A §3.2 (o gap nó-a-nó dos 87 existentes) NÃO TEM UMA ÚNICA LINHA DE DEFORMER.** As 22 linhas dela cobrem forças, noise, oscillator, stagger, wiggle, spring, emitter, falloff, clone, trail, map_range, grid, look_at, sort, mixer, color_ramp, spawn e poisson — **nenhum dos sete**. O doc 63 **não errou sobre esta família: ele não a conferiu**, e a §3.2 anuncia-se como *"o gap nó-a-nó"*. É exatamente o buraco que o doc 89 §0 descreve, um nível abaixo.
2. **§A.2 do dump Cavalry: `Travel Deformer` → "PARCIAL (spline_wrap)"** — a etiqueta está certa e o **conteúdo** dela nunca foi escrito. Esta conferência nomeia o que a torna parcial: **From/To**, **Mode Fit/Keep Length** e a **orientação que não segue a tangente** (P0/P0/P1).
3. **§A.2: `Four Point Warp` → "TEMOS"** — verdade para o *Corner Pin*, **e a linha esconde que a família tem DOIS membros na referência**: o Bezier Warp (arestas curvas) não é um param a mais no que temos, é outro algoritmo cujo neutro **não reduz** ao nosso (Coons-com-arestas-retas é bilinear, não homografia). *"TEMOS" sobre um par é meia resposta.*
4. **§A.2: `Skew / Pinch` → "FALTA"** — o **Pinch** **NÃO falta**: é o `motion.spherize` com `amount < 0` (o header dele diz *"`amount < 0` compresses it (**pinch**)"*). ⚠️ Item marcado FALTA que **já existe** — precisamente o erro que o doc 89 §1 avisa custar caro, *"porque manda construir o que está construído"*. **O `Skew` continua faltando** (e é afim: caberia no `motion.transform`, não aqui).
5. **§A.2: `Squash and Stretch / Motion Stretch` → "FALTA (clássico de animação!)"** — confirmado, e esta conferência acrescenta o **porquê é desta família**: é um deformer dirigido pela **velocidade**, e nenhum dos sete lê a coluna de velocidade. Fica **nomeado, fora do escopo** (nó novo, não param).
6. **A §0.1/D13 manda conferir "Coordinates (position/**rotation**/scale)" em todo nó** — e a família **falha na rotação em quatro dos sete** (`bend` sem direção, `spline_wrap` sem axis, `four_point_warp` e `spherize` sem frame rotacionado). *É literalmente o "buraco da rotação" que a D13 foi escrita para impedir, um ano depois, noutra família.*
7. **A §3.1 lista um cluster `STRENGTH/weight`** — *"todo modificador multiplicável por campo de peso"* — e é justamente esse cluster que a **§1 desta conferência mostra estar QUEBRADO para os rotacionais**: o peso existe, mas mascara deslocamento em vez de escalar o parâmetro.
