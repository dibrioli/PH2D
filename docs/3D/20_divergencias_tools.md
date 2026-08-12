# As divergências dos TOOLS — nosso app × SculptGL × Blender

> **Estudo. Nada foi mudado.** Cada achado traz o mecanismo e, onde há número, a
> medição. O que não foi medido está marcado **[NÃO MEDIDO]** em vez de estimado.

## §0 — Método, e o que cada perna pode afirmar

| perna | fonte | o que ela pode afirmar |
|---|---|---|
| **nós** | a árvore desta linha | tudo — código lido **e** medido pela porta do produto |
| **SculptGL** | `/home/enio/Documentos/Recursos/SculptGL`, MIT | tudo — código lido **e** executado (o oráculo de 14 casos) |
| **Blender** | `/home/enio/Documentos/Recursos/BlenderSculpt`, GPL | **só código lido.** O clone é um TRIM de escultura: `blenkernel/intern/brush.cc` **não existe nele**, então as fórmulas das curvas-preset (`BRUSH_CURVE_*`) **não são verificáveis aqui** e não são afirmadas abaixo |

⚠️ **GPL: comportamento, nunca código.** Tudo o que este doc tira do Blender é
*descrição de mecanismo*, que é o que o [ADR-0150](architecture/decisions/0150-3d-sculpt-is-a-mesh-that-donates-shading-sculptgl-referenced.md)
permite.

⚠️ **E há uma assimetria de ALVO que decide como ler a lista:** a paridade
bit-idêntica que o Enio pediu é **com o SculptGL** ([doc 19](19_paridade_sculptgl.md)).
O Blender entra aqui como **terceira opinião**, não como alvo — quando os dois
discordam, o doc diz qual dos dois nós seguimos e por quê.

## §1 — Os três catálogos

**Nós: 16 verbos.** **SculptGL: 12 tools.** **Blender: 33 tipos de pincel**
(`SCULPT_BRUSH_TYPE_*` em `sculpt.cc`).

| nosso verbo | SculptGL | Blender |
|---|---|---|
| Draw | `Brush` com `_clay = false` | `DRAW` |
| Clay | `Brush` com `_clay = true` (**o default de fábrica**) | `CLAY` |
| Flatten · Fill · Scrape | `Flatten` (um só, com `_negative`) | `PLANE` (um só, com `height`/`depth`) |
| Inflate | `Inflate` | `INFLATE` |
| Crease | `Crease` | `CREASE` |
| Pinch · Magnify | `Pinch` (com `_negative`) | `PINCH` |
| Smooth | `Smooth` | `SMOOTH` |
| Sharpen | — (nosso) | — |
| Mask | `Masking` | `MASK` |
| Move | `Move` | `GRAB` |
| SnakeHook | `Drag` | `SNAKE_HOOK` |
| Twist | `Twist` | `ROTATE` |
| LocalScale | `LocalScale` | — (o `THUMB`/`SCALE` do gizmo) |
| — | `Paint` (cor por-vértice) | `PAINT` · `SMEAR` · `BLUR` |
| — | `Transform` (gizmo) | `TRANSFORM` (operador) |

**O que o Blender tem e nenhum dos dois tem** (19 tipos): `CLAY_STRIPS` ·
`CLAY_THUMB` · `BLOB` · `LAYER` · `NUDGE` · `THUMB` · `ELASTIC_DEFORM` · `POSE` ·
`BOUNDARY` · `CLOTH` · `SLIDE_RELAX` · `MULTIPLANE_SCRAPE` · `SIMPLIFY` ·
`DRAW_SHARP` · `DRAW_FACE_SETS` · `DISPLACEMENT_ERASER` · `DISPLACEMENT_SMEAR` ·
`SCENE_PROJECT` · `SURFACE_SMOOTH`.

---

## §2 — Os eixos: o que faz uma ferramenta ser ela

Um verbo é decidido por **sete** coisas, e é por eixo que as divergências abaixo
estão organizadas — não por verbo.

1. **A CURVA** — o peso a uma distância normalizada.
2. **A DIREÇÃO** — normal de área · normal do vértice · normal do PONTO de
   impacto · tangencial · vista.
3. **A MAGNITUDE** — a constante do verbo × intensidade × raio.
4. **O ALVO** — posição absoluta (plano, média) × incremento.
5. **A ACUMULAÇÃO** — de onde a distância é medida, e o que o dab `k+1` vê do `k`.
6. **A AMOSTRA** — quem entra no plano, com que peso, em que raio.
7. **OS DEFAULTS** — porque é neles que o artista encontra a ferramenta.

---

## §3 — OS ACHADOS

### A — Defaults: o que o artista pega na mão (o grupo mais caro)

#### D1 — ⚠️ O falloff default é `Smooth`, e **nenhuma tool da referência usa essa curva**

As **dez** tools de geometria do SculptGL multiplicam pela quártica
`3t⁴ − 4t³ + 1` — o nosso [`Falloff::Plateau`]. O `Brush::default()` ship
`falloff: Falloff::Smooth`, que é `(1 − t²)²`, e **o seletor de verbo nunca arma
o falloff** (o `event.rs:120-126` e o `sculpt3d_keys.rs:448-452` armam
`default_strength` e `default_accumulate` — **o falloff não está lá**).

**Medido** (sonda `what_separates_our_kernels_from_the_reference`, tabela *A CURVA*):

| t | referência (`Plateau`) | nosso default (`Smooth`) | razão |
|---|---|---|---|
| 0,250 | 0,949219 | 0,878906 | **1,080×** |
| 0,500 | 0,687500 | 0,562500 | **1,222×** |
| 0,750 | 0,261719 | 0,191406 | **1,367×** |
| 0,875 | 0,078857 | 0,054932 | **1,436×** |

⚠️ **Isto não é diferença numérica: é o pincel INTEIRO mais magro**, com o ombro
morrendo cedo. É a primeira coisa que um artista sente e a última que ele
associaria a um enum. E é literalmente o *"cada tool deve ter seu falloff
apropriado"* do pedido — **a resposta estava na referência e a fábrica escolheu
outra**.

**Classificação:** defeito de produto. **Decisão:** de quem é o default — do
artista (produto) ou da referência (paridade)?

#### D2 — ⚠️ `default_strength` é **0,5 chapado**; a referência tem intensidade POR TOOL

`Verb::default_strength()` devolve `1.0` para `Mask` e **`0.5` para os outros
quinze**. O SculptGL declara a intensidade no construtor de cada tool:

| tool | referência | nós | razão |
|---|---|---|---|
| `Brush` (Draw/Clay) | 0,50 | 0,50 | 1,00× |
| `Flatten` | **0,75** | 0,50 | 0,67× |
| `Inflate` | **0,30** | 0,50 | **1,67×** |
| `Crease` | **0,75** | 0,50 | 0,67× |
| `Pinch` | **0,75** | 0,50 | 0,67× |
| `Smooth` | **0,75** | 0,50 | 0,67× |
| `Masking` | 1,00 | 1,00 | 1,00× |
| `Move` | 1,00 | 0,50 | 0,50× |

⚠️ **A intensidade é LINEAR em todos os kernels** (ela multiplica o `deform`),
então a razão da tabela **é** a razão do deslocamento: o Inflate de fábrica
empurra **2/3 a mais** que o do original e o Flatten metade do que devia.

⚠️ **E o oráculo JÁ SABE disso.** O `sculptgl_oracle.mjs` grava cada caso com a
intensidade da tool (`0.5` no brush, `0.75` no flatten/crease/pinch, `0.3` no
inflate) — *o harness carrega a tabela que o produto não tem*.

**Classificação:** defeito de produto. **Decisão:** nenhuma — é a referência.

#### D3 — O RAIO default é um só; a referência tem um por tool

`Brush::default()` ship `radius: 0.25` para os dezesseis. A referência:
`Crease` **25** (metade do padrão) · `Twist` **75** · `Move`/`Drag` **150**
(triplo) · o resto **50**.

⚠️ **O Crease é o caso que dói:** um vinco com o raio de um Draw é um sulco
largo — a queixa clássica de *"o crease não afia"*. **[NÃO MEDIDO]** em ângulo
de sulco; o mecanismo é aritmético (a largura do V é o raio).

#### D4 — O `_negative` default é por tool, e o nosso `invert` nasce `false` para todos

Referência: `Flatten._negative = true` · `Crease._negative = true` ·
`Masking._negative = true`. ⚠️ **Consequência de leitura:** o Crease de fábrica
do original **CAVA**, o nosso **LEVANTA** — a mesma tecla, o sinal oposto.

---

### B — Algoritmo: onde divergimos da referência

#### D5 — ⚠️ Draw e Crease: a direção é a normal do **PONTO DE IMPACTO** lá, e a de **ÁREA** aqui — e o ORÁCULO passa a de área

`Brush.js:44` e `Crease.js:33` chamam o kernel com **`picking.getPickedNormal()`**
— a normal interpolada no ponto em que o raio bateu. Nós passamos `plane.normal`
(a normal de área).

⚠️ **O gate de paridade é VERDE sobre isto, e o motivo é a fixture:** o
`sculptgl_oracle.mjs:505` computa `areaNormal.call(self, front)` e o entrega ao
`kernels.brush` — *o harness mede o KERNEL, que é agnóstico a qual normal você
lhe dá, e não a TOOL, que escolhe uma*. O comentário do gerador chega a declarar
a escolha (*"empurra pela normal de area"*), o que a torna deliberada e não
acidental — mas ela é uma afirmação sobre o kernel, não sobre o produto.

⚠️ **O Blender concorda CONOSCO:** `draw.cc:194-197` usa
`tilt_effective_normal_get`, que resolve para `cache->sculpt_normal_symm` — a
normal de ÁREA — e `crease.cc:205` idem.

**[NÃO MEDIDO]** quanto pesa. Numa esfera as duas coincidem por simetria (é a
fixture do atlas); a diferença vive em **superfície curva assimétrica** (uma
crista, um vinco já cavado). **Decisão:** seguimos o Blender (área) ou o
SculptGL (impacto)? Hoje seguimos o Blender **sem que isso esteja escrito**.

#### D6 — Pinch/Crease: nós PROJETAMOS a tangente, a referência puxa em 3D

`Pinch.js:56-59` soma `(c − v) · fallOff` **direto**, sem projetar. Nós usamos
`tangential(live, dab.center, n_area)`. O `ref_kernels.rs:375` já nomeia isto.

**Medido** (atlas de um dab): `|diferença|` **5,776e-4** no Pinch/Magnify e
**8,087e-4** no Crease — os **dois maiores da tabela** junto ao Flatten, contra
o piso de `5,96e-8` dos outros nove.

**Classificação:** divergência conhecida, já aberta no doc 19 como decisão do Enio.

#### D7 — Flatten: nós somos BILATERAL, a referência é unilateral

`Flatten.js:64` faz `if (distToPlane * comp > 0.0) continue` — o `Flatten` deles
**é** o nosso `Fill` **ou** o nosso `Scrape`, nunca os dois. **Medido:** o
`|diferença|` do Flatten no atlas é **1,717e-3**, e ele **é a soma do lado
Fill** — a divergência tem nome e tamanho.

**Classificação:** decisão de produto já aberta no doc 19.

#### D8 — Inflate: nós lemos a normal CONGELADA, a referência lê a viva

`Inflate.js:64-66` lê `nAr` (as normais VIVAS, que o `updateGeometry` recomputa a
cada dab). Nós lemos `self.base_nrm[s]`, congelada no pen-down. Está documentado
em `stroke_target.rs:197-200` como deliberado (*"a normal viva sobe junto com a
tinta, e um traço parado passaria a inflar numa direção que gira sozinha"*).

⚠️ **O atlas não vê:** ele mede **um** dab, onde congelado e vivo são o mesmo
valor. A divergência só existe a partir do 2º.

#### D9 — Smooth: a referência **não tem falloff**, nós temos

`Smooth.js:44-64` aplica `intensity × mask × alpha` **uniforme** sobre a pegada.
Nós multiplicamos pela curva. Documentado como superset deliberado
(`Falloff::Constant` reproduz a referência).

#### D10 — ⚠️ Um doc-comment nosso nomeia a lei do BLENDER sobre um corpo que implementa a do SculptGL

`stroke_target.rs:27-30` diz que o estimador do plano é *"a média ponderada pelo
**falloff** das posições e das normais da pegada, que é o
`calc_area_normal_and_center` do Blender"*. O corpo, sessenta linhas abaixo,
pondera pela **MÁSCARA** — e o comentário da linha 91 diz isso explicitamente
(*"A ponderação é a MÁSCARA, não o falloff"*).

Os dois comentários estão no mesmo arquivo e se contradizem; o segundo é o certo.
⚠️ *Um comentário que descreve a lei de outro produto é como a próxima pessoa
"conserta" a nossa para uma que ela nunca teve.*

---

### C — O que a REFERÊNCIA tem e nós não

| # | falta | mecanismo | consequência |
|---|---|---|---|
| **D11** | `_culling` por tool | `SculptBase.getFrontVertices` filtra o conjunto DEFORMADO | sem ele, um dab sobre uma casca fina move os dois lados |
| **D12** | `_lockPosition` | `updateSculptLock` — a pegada trava e o arrasto vira DIREÇÃO do alpha | é o gesto de carimbo direcional |
| **D13** | `smoothTangent` | `Smooth.js:67-105` projeta o deslocamento no plano da normal | **é a cura do ENCOLHIMENTO** — ver abaixo |
| **D14** | `smoothAlongNormals` | `Smooth.js:107-134` — só a componente normal | alisa relevo sem mexer no contorno |
| ~~**D15**~~ | ~~dyntopo DENTRO do traço~~ | — | ⚠️ **RETIRADO — ele EXISTE.** Ver abaixo |

⚠️ **D15 estava ERRADO e a correção é minha.** Eu greppei `dyntopo` no
`stroke.rs`, não achei, e escrevi que a capacidade faltava. Ela existe e roda
**por dab**: `sculpt3d_input.rs:128` abre a sessão no pen-down
(`open_dyntopo_stroke`) e a linha `:355` chama **`refine_for_dab(hit.point,
brush.radius)` ANTES de carimbar**, com o comentário ao lado dizendo exatamente
isso (*"REFINA E DEPOIS CARIMBA"*). O motor é o `ph2d_mesh::refine_in_sphere`, e
há toggle + slider de Detail na seção TOPOLOGY do painel.

*Ausência procurada no arquivo errado vira capacidade faltando num doc.* O
mecanismo mora no SHELL (é decisão do editor, não do kernel de traço), que é
onde eu não olhei.

⚠️ **O D13 é o mais caro, e ele já está MEDIDO no nosso próprio repo.** O
`verb_border_tests.rs:75-93` mede a boca de um tubo aberto encolhendo em
**progressão geométrica exata** — raio `1,0000 → 0,5000 → 0,2500 → … → 0,0156`
em seis passes, fator `cos(60°)`. O doc dele diz, corretamente, que *"a
referência encolhe igual, e por escolha"* — mas a escolha dela é ter o
`smoothTangent` **desligado**, e nós **não temos o interruptor**. O Blender tem
`SURFACE_SMOOTH` e `SLIDE_RELAX` para exatamente isto.

*Um Smooth que suga a forma para dentro é a queixa nº 1 de "esculpe mal", e nós
temos o número dela há mais tempo do que temos a cura.*

---

### D — O que o BLENDER tem e nenhum dos dois tem

| # | mecanismo | onde | por que importa |
|---|---|---|---|
| **D16** | `normal_radius_factor` — o plano/normal é amostrado num raio **DIFERENTE** do dab | `sculpt.cc:1437-1463` | é o que estabiliza o Clay/Flatten: o plano deixa de tremer com o que o próprio pincel acabou de mover |
| **D17** | o peso do área-normal é **smoothstep da distância** (`3p²−2p³`) | `sculpt.cc:1466-1470` | nós e a referência pesamos pela MÁSCARA, uniforme em distância ⇒ o vértice da borda da pegada vota tanto quanto o do centro |
| **D18** | o front-face é **CONTÍNUO** (`factors[i] *= max(dot, 0)`) | `sculpt.cc:7283-7295` | o nosso e o da referência são **binários** ⇒ degrau no terminador |
| **D19** | a amostra é partida em **DOIS baldes** (frente/verso) e vence o maior | `sculpt.cc:1568`, `flip_index` | trata casca fina e superfície dobrada, onde a média das normais se cancela |
| **D20** | `hardness` por pincel, que **reshapeia a DISTÂNCIA** antes da curva | `sculpt.cc:7549-7575` | nós temos hardness **só** no canal de máscara |
| **D21** | `alpha = strength²` (*"square it to make lower values more sensitive"*) | `sculpt.cc:2337-2339` | ergonomia do slider; nós e a referência somos lineares |
| **D22** | magnitude por-tool com **curva de pressão própria** — Clay `pressure⁴`, Clay Strips `pressure^1,5 × 0,3`, Clay Thumb `pressure² × 1,3`, Inflate **0,25 inflando × 0,125 murchando** | `sculpt.cc:2352-2419` | é o eixo 3 do §2, e o Blender o resolve por tabela; nós temos as constantes (0,1 · 0,07 · 0,05) e **nenhuma assimetria de sinal** |
| **D23** | `overlap_factor` — a força é normalizada pelo **espaçamento** | `sculpt.cc:2343` | o nosso espaçamento é fixo em `0,15·raio` ⇒ hoje é moot, e deixa de ser no dia em que o espaçamento virar knob |
| **D24** | `sculpt_plane` (Area/View/X/Y/Z) · `falloff_shape` (esfera/**tubo**) · `plane_trim` · `area_radius_factor` | `sculpt.cc:2703-2717`, `3070-3095` | temos só `plane_offset` |
| **D25** | `BRUSH_ORIGINAL_NORMAL` / `BRUSH_ORIGINAL_PLANE` — congelar o plano é **escolha do artista** | `sculpt.cc:3060-3065` | ver a nota abaixo |
| **D26** | auto-masking (topologia · face set · cavidade · normal de vista) | `sculpt_automasking.cc` | é o que impede um pincel de atravessar para a outra perna do modelo |
| **D27** | o `PLANE` unificado com `height`/`depth` e falloff **CILÍNDRICO** no eixo do plano | `plane.cc:80-120` | Blender 5.x **fundiu** Flatten/Fill/Scrape num verbo com dois números; o nosso split em três verbos é o desenho anterior |

⚠️ **O D25 tem uma metade que nos dá razão, e vale registrar:** o
`calc_brush_plane` do Blender recomputa o plano **por dab** a menos que o artista
marque as duas flags — e o brush `PLANE` é **proibido** de congelá-las
(`brush.sculpt_brush_type != SCULPT_BRUSH_TYPE_PLANE` nas duas linhas). Mais: o
`calc_area_normal_and_center` lê posições **ORIGINAIS quando `!cache->accum`** e
**VIVAS quando accum** (`sculpt.cc:1544`) — *o Blender amarra a vivacidade do
plano ao mesmo interruptor que nós acabamos de amarrar*. A correção de 12/08
(plano vivo) tem as duas referências do lado dela.

---

## §4 — O que **NÃO** diverge (negativos, e eles valem)

| # | conferido | resultado |
|---|---|---|
| **N1** | as constantes de magnitude | `REACH_FRACTION 0,1` · `CREASE_FRACTION 0,07` · `PINCH_GAIN 0,05` · `CLAY_PLANE_FRACTION 0,1` — **idênticas** ao `Brush.js`/`Crease.js`/`Pinch.js` |
| **N2** | o espaçamento | `MIN_SPACING_FRACTION 0,15` = o `0.15 * this._radius` do `SculptBase.sculptStroke` |
| **N3** | a máscara DENTRO do expoente 5 do Crease | correto: `shape = fall·keep` carrega a máscara, e `w · shape⁴` **é** `(curva·alpha·máscara)⁵ · intensidade` |
| **N4** | a cobertura de máscara do oráculo | **144 de 503 vértices mascarados em TODOS os 14 casos** (311 de 1089 no `smooth`) ⇒ o termo de máscara de cada kernel **está** sob teste |
| **N5** | a família do GRAB | Move · SnakeHook · Twist · LocalScale a **1 ULP** (`2,98e-8` / `5,96e-8`) |
| **N6** | o `areaCenter` divide pela SOMA DAS MÁSCARAS | portado fielmente, **com o buraco do original tampado**: pegada 100% travada devolve `None` em vez do `NaN` que envenena a malha |
| **N7** | o plano, hoje | `cos 1,000000` na normal e **0,0%** do raio no centro contra o `area_*` da referência |

---

## §5 — A TABELA DE ESCOLHAS, e o que cada uma muda no olho

⚠️ **Só entram aqui as divergências que são ESCOLHA.** As que são defeito puro
(D2 — o Inflate a 1,67×) não pedem gosto, pedem a tabela da referência.

| # | a escolha | nós hoje | SculptGL | Blender | testável **HOJE**? | o que o olho vê |
|---|---|---|---|---|---|---|
| **E1** | a **CURVA** do pincel | `Smooth` `(1−t²)²` | quártica `3t⁴−4t³+1` | curva editável, 9 presets | ✅ painel BRUSH → Falloff → `Plateau` | ombro do pincel: magro e morrendo cedo × cheio com pouso plano |
| **E2** | a **FORÇA** de fábrica por tool | 0,5 em tudo | 0,75 (Flatten/Crease/Pinch/Smooth) · 0,30 (Inflate) | tabela por tool + `pressão^k` | ✅ slider Strength | quanto um toque entrega |
| **E3** | o **RAIO** de fábrica por tool | um só | Crease **½** · Move/Drag **3×** | por tool | ✅ `[` e `]` | largura do sulco, alcance do puxão |
| **E4** | o **SINAL** de fábrica | tudo levanta | Flatten/Crease/Mask **cavam** | por tool | ✅ Ctrl | o Crease de fábrica cava ou levanta |
| **E5** | o **Flatten** é bilateral? | bilateral | unilateral (é o nosso Fill **ou** Scrape) | `height`/`depth`, dois números | ✅ compare Flatten × Fill × Scrape | achatar corta a crista **e** enche o vale, ou só um |
| **E6** | o **Smooth** tem falloff? | tem | **não tem** (uniforme) | tem | ✅ Falloff → `Constant` | borda do alisado: degrau × transição |
| **E7** | o Smooth **encolhe**? | encolhe (laplaciano cru) | encolhe (`smoothTangent` **existe e está OFF**) | tem `SURFACE_SMOOTH` e `SLIDE_RELAX` | ⛔ build | o volume some sob o dedo × a forma fica e só o ruído sai |
| **E8** | a **direção** do Draw/Crease | normal de **ÁREA** | normal do **PONTO de impacto** | normal de área | ⛔ build | numa crista: o dab sobe pelo *lugar* ou pela *região* |
| **E9** | o **Pinch** projeta na tangente? | projeta | puxa em **3D** | projeta (`x_disp` + `z_disp`) | ⛔ build | apertar afunda junto, ou só junta |
| **E10** | a **normal do Inflate** | congelada no pen-down | **viva** | viva | ⛔ build | traço parado: infla reto × a direção gira sozinha |
| **E11** | o plano é amostrado no raio do dab? | sim | sim | **não** (`normal_radius_factor`) | ⛔ build | Clay/Flatten tremendo × plano firme |
| **E12** | o front-face é binário? | binário | binário | **contínuo** (`max(dot,0)`) | ⛔ build | degrau na silhueta × pouso macio |
| **E13** | o slider de força é linear? | linear | linear | **`strength²`** | ⛔ build | sensibilidade da faixa baixa |
| **E14** | `hardness` que reshapeia a distância | só na máscara | não tem | **tem, por pincel** | ⛔ build | um platô duro no miolo do dab |

## §6 — O ROTEIRO: o que testar, em que ordem, e o que perguntar

**As seis primeiras não custam build nenhum** — são controles que já estão na
tela. Cena: `env PH2D_SCULPT3D_SMOKE=1 cargo run -p ph2d-host-desktop --release`
(esfera limpa), painel na CRASE (`` ` ``).

⚠️ **A regra do A/B:** faça **o mesmo traço** nos dois lados da esfera, mude
**uma** coisa, e compare **lado a lado na mesma tela**. Trocar e re-desenhar por
cima mede a soma, não a diferença.

| # | teste | gesto | a pergunta que ele responde |
|---|---|---|---|
| **T1** | **A CURVA** — o maior número do estudo | Draw, Strength 0,5. Um traço com Falloff `Smooth`; troque para `Plateau`; o mesmo traço ao lado | *o pincel de fábrica é magro demais?* Se `Plateau` parecer "o pincel certo", ele vira o default e a paridade ganha de graça |
| **T2** | **A FORÇA por tool** | Inflate a 0,5 (fábrica) × a **0,30** (referência). Depois Flatten a 0,5 × **0,75** | *o Inflate estoura? o Flatten não chega?* — é a tabela que o oráculo já tem |
| **T3** | **O CREASE inteiro** (três divergências de uma vez) | Crease de fábrica × Crease com raio **pela metade** (`[`), força **0,75** e **Ctrl** | *é assim que um vinco se faz?* Se sim, E2+E3+E4 fecham juntos |
| **T4** | **ACHATAR** | uma crista. Flatten × Fill × Scrape, no mesmo relevo | *"achatar" para você corta a crista, enche o vale, ou os dois?* Decide E5 |
| **T5** | **O SMOOTH** | Shift-arraste sobre um relevo com Falloff `Smooth` × `Constant` | *a borda do alisado precisa de transição?* Decide E6 |
| **T6** | **O SMOOTH que ENCOLHE** (só medir, ainda não escolher) | alise um dedo/ponta fina por uns segundos, sem soltar | *quanto de forma some?* É o E7, e **o número já existe**: raio `1,0 → 0,5 → 0,25 → … → 0,0156` em seis passes |

**Depois destes seis, o que você sentir decide qual build vale.** Os candidatos,
por ordem do que eu esperaria render:

1. **E7 (o Smooth que não encolhe)** — é a única da lista que muda o que o app
   *consegue* fazer. O `smoothTangent` já existe na referência, desligado.
2. **E11 (`normal_radius_factor`)** — se o T4 mostrar o plano tremendo.
3. **E8 (a normal do Draw)** — se o T1 deixar o Draw "escorregando" numa crista.
4. **E12/E14** (silhueta e hardness) — refinamento, depois dos três acima.

⚠️ **E há um par que eu recomendo NÃO testar por gosto:** o **E9** e o **E10**
têm resposta técnica (o Blender concorda com o nosso Pinch projetado; a normal
congelada do Inflate evita uma direção que gira sozinha num traço parado). Se o
olho não reclamar deles no T2, deixe-os quietos.

## §7 — O placar, e o que ele diz

**Onze** divergências de comportamento (D1-D10 + D15), **cinco** capacidades que
a referência tem e nós não (D11-D15), **doze** que só o Blender tem (D16-D27).

⚠️ **A leitura que a lista sugere — e ela é minha, não uma medição:** os quatro
primeiros achados (**D1-D4**, os defaults) são de longe os mais baratos e os mais
visíveis. Nenhum deles é um algoritmo: são **quatro tabelas** que a referência
declara e nós não copiamos, e três delas já vivem no nosso próprio oráculo. O
falloff sozinho (**D1**) muda o pincel inteiro em **1,08× a 1,44×** ao longo do
raio, e é o único achado deste doc que o artista encontra **sem tocar em nada**.

Depois deles, **D13/D15** (o Smooth que encolhe · dyntopo no traço) são os dois
que mudam o que o app *consegue fazer*, não o quanto ele erra.

O resto é decisão de produto (D5-D9) ou catálogo (§1, D16-D27).

---

# PARTE II — O CATÁLOGO COMPLETO, ferramenta a ferramenta

⚠️ **O que cada linha desta parte pode afirmar.** As colunas *nós* e *SculptGL*
são código **lido e executado**. A coluna *Blender* é código **lido**, e onde eu
li só o cabeçalho do arquivo a linha diz **[cabeçalho]** — a descrição é a que o
próprio comentário dá, não uma leitura minha do kernel.

## §8 — Por FAMÍLIA, que é como um escultor pensa

### A — DEPOSITAR massa

| ferramenta | nós | SculptGL | Blender |
|---|---|---|---|
| **Draw** | ✅ `Verb::Draw` — `pos += n_área · (falloff·força·máscara) · raio·0,1` | ✅ `Brush` com `_clay=false` — idem, mas pela normal do **ponto de impacto** | ✅ `DRAW` — `offset = normal_área · raio · escala · força`, com `tilt` de caneta |
| **Draw Sharp** | ❌ | ❌ | ✅ `DRAW_SHARP` — **o mesmo offset, computado das posições e normais ORIGINAIS** (`orig_data`), congeladas no pen-down. É o que dá o vinco duro em vez do domo |
| **Clay** | ✅ `Verb::Clay` — `Fill` contra o plano deslocado por `raio·0,1` | ✅ `Brush` com `_clay=true`, **o default de fábrica** | ✅ `CLAY` — projeta no plano mais próximo; força `0,25·alpha·pressão⁴` |
| **Clay Strips** | ❌ | ❌ | ✅ `CLAY_STRIPS` — plano + falloff **CÚBICO/quadrado** (dab retangular) e um segundo falloff **parabólico no eixo Z do plano**; força `alpha·pressão^1,5·0,3` |
| **Clay Thumb** | ❌ | ❌ | ✅ `CLAY_THUMB` — dois planos, um deles **INCLINADO**, e o ângulo **cresce com a acumulação do traço** (*"simulate the clay accumulation by increasing the plane angle"*) |
| **Blob** | ❌ | ❌ | ✅ `BLOB` — o Crease com o sinal do pinch invertido (empurra para fora em vez de juntar) |
| **Layer** | ❌ | ❌ | ✅ `LAYER` — carrega um **deslocamento PERSISTENTE por vértice** que satura (`d += f·s·(1,05 − \|d\|)`, clamp ±1) e sabe **APAGAR** camadas anteriores (`reset_displacement_factors`). É uma demão de altura constante |
| **Crease** | ✅ `Verb::Crease` — tangencial linear + normal **quíntica**, ganho `0,07` | ✅ `Crease` — idem, pela normal do **ponto de impacto**; `raio 25`, `intensidade 0,75`, `_negative=true` | ✅ `CREASE` — pinch com `crease_pinch_factor` + offset pela normal de área |
| **Draw Vector Displacement** | ❌ | ❌ | ✅ — a **cor RGB de uma textura** vira o vetor de deslocamento (mapa de displacement vetorial) |

### B — NIVELAR contra um plano

| ferramenta | nós | SculptGL | Blender |
|---|---|---|---|
| **Flatten** | ✅ **bilateral** | ✅ `Flatten` **unilateral** (`_negative`) | ✅ dentro do `PLANE` |
| **Fill** | ✅ (o lado que sobe) | = o `Flatten` com `_negative=false` | ✅ dentro do `PLANE` |
| **Scrape** | ✅ (o lado que desce) | = o `Flatten` com `_negative=true` | ✅ dentro do `PLANE` |
| **Plane** (unificado) | ❌ | ❌ | ✅ `PLANE` — **os três num só**, com `height` e `depth`: a distância local é dividida por eles, o que faz a pegada virar um **cilindro achatado** em vez de uma esfera. `height=0` desliga o lado de cima; `depth=0` o de baixo |
| **Multiplane Scrape** | ❌ | ❌ | ✅ `MULTIPLANE_SCRAPE` — **DOIS planos** com um ângulo entre eles, amostrados dos dois lados do eixo X local do pincel. É o que faz uma quina em V numa passada |
| **Scene Project** | ❌ | ❌ | ✅ **[cabeçalho]** — projeta os vértices contra a superfície de **OUTROS objetos da cena** por raycast, com direção (vista ou normal do plano) e modo bidirecional |

### C — INFLAR e APERTAR

| ferramenta | nós | SculptGL | Blender |
|---|---|---|---|
| **Inflate** | ✅ normal **congelada** no pen-down | ✅ normal **viva**, `intensidade 0,3` | ✅ força **assimétrica**: `0,250` inflando × `0,125` murchando |
| **Pinch** | ✅ tangencial **projetada** | ✅ puxa em **3D**, sem projetar | ✅ decompõe em `x_disp` + `z_disp` (projeta no eixo do traço e na normal) |
| **Magnify** | ✅ (o Pinch com o sinal trocado) | = `Pinch` com `_negative` | = `PINCH` invertido |

### D — ALISAR

| ferramenta | nós | SculptGL | Blender |
|---|---|---|---|
| **Smooth** | ✅ laplaciano + falloff | ✅ laplaciano **sem falloff** | ✅ `SMOOTH` — laplaciano com regra de borda (`interior`) e caso `loose` |
| **Smooth tangencial** | ❌ | ✅ `smoothTangent` — **existe e está DESLIGADA** (`_tangent=false`) | ✅ `SURFACE_SMOOTH` — laplaciano **de dois passes** (HC), guardando o `laplacian_disp` por vértice |
| **Smooth ao longo da normal** | ❌ | ✅ `smoothAlongNormals` — existe, e **nenhuma UI a alcança** | — |
| **Relax / Slide Relax** | ❌ | ❌ | ✅ `SLIDE_RELAX` — redistribui os vértices **sobre a superfície** sem mudar a forma; sabe **respeitar face sets** (`relax_face_sets`) e traz um *"strength tweak"* próprio |
| **Topology Slide** | ❌ | ❌ | ✅ — desliza os vértices ao longo das arestas, com `slide_deform_type` escolhendo a direção |
| **Enhance Details** | ❌ | ❌ | ✅ — **o Smooth com o sinal trocado**, aplicado ao deslocamento em relação à média: é o nosso `Sharpen`, mas o Blender o computa como *"o que o smooth removeria, devolvido"* |
| **Sharpen** | ✅ `Verb::Sharpen` (nosso) | ❌ | = `ENHANCE_DETAILS` |
| **Topology Rake** | ❌ | ❌ | ✅ `bmesh_topology_rake` — alinha o **fluxo das arestas** com a direção do traço (só dyntopo) |

### E — PUXAR e AGARRAR

| ferramenta | nós | SculptGL | Blender |
|---|---|---|---|
| **Grab / Move** | ✅ `Verb::Move` — pegada congelada, `pos = pre + gesto·falloff` | ✅ `Move`, `raio 150`, `intensidade 1,0` | ✅ `GRAB` — lê `orig_data`, com **silhueta opcional** (`calc_silhouette_factors`) e a normal do traço **congelada** de propósito |
| **Snake Hook / Drag** | ✅ `Verb::SnakeHook` | ✅ `Drag`, `raio 150` | ✅ `SNAKE_HOOK` — mais **pinch** ao longo do caminho e **rake rotation** (a ponta gira com a curva do traço) |
| **Nudge** | ❌ | ❌ | ✅ `NUDGE` — empurra **tangencialmente** na direção do traço (`cross(cross(normal, delta), normal)`) |
| **Thumb** | ❌ | ❌ | ✅ `THUMB` — o Grab **projetado no plano da vista**; a normal não é recomputada |
| **Rotate / Twist** | ✅ `Verb::Twist` | ✅ `Twist`, `raio 75` | ✅ `ROTATE` — gira os `orig_data` em torno do eixo da vista |
| **Local Scale** | ✅ `Verb::LocalScale` | ✅ `LocalScale` | ❌ (é o gizmo de transform, não um pincel) |
| **Elastic Deform** | ❌ | ❌ | ✅ — **KELVINLETS** (`BKE_kelvinlet_*`): grab · grab bi/tri-escala · scale · twist. É deformação elástica com solução analítica, e é o que dá o puxão que "puxa o modelo inteiro" de forma contínua |
| **Pose** | ❌ | ❌ | ✅ **[cabeçalho]** — pose por cadeia de segmentos, com IK |
| **Boundary** | ❌ | ❌ | ✅ **[cabeçalho]** — deforma a partir de um **contorno aberto**, propagando pela topologia |
| **Cloth** | ❌ | ❌ | ✅ — simulação de tecido local, com modos `GRAB`/`SNAKE_HOOK`/`EXPAND` e força própria (`10×` por default) |

### F — PINTAR CANAIS (não movem geometria)

| ferramenta | nós | SculptGL | Blender |
|---|---|---|---|
| **Mask** | ✅ `Verb::Mask` — aditivo saturante, curva `(1−d)^{2(1−hardness)}` | ✅ `Masking`, `hardness 0,25`, `_negative=true` | ✅ `MASK` — com `mask_tool` = `DRAW` ou `SMOOTH` |
| **Mask Smooth** | ⚠️ temos botões `Blur`/`Sharpen` de máscara no painel | ✅ (dentro do `Masking`) | ✅ `smooth_mask.cc`, um brush-mode do Mask |
| **Paint (cor por-vértice)** | ❌ | ✅ `Paint` — o kernel dele **já está portado de carona** (o `Masking` o chama) | ✅ `PAINT` |
| **Smear (cor)** | ❌ | ❌ | ✅ `SMEAR` — arrasta a COR pela superfície |
| **Blur (cor)** | ❌ | ❌ | ✅ `BLUR` |
| **Draw Face Sets** | ❌ | ❌ | ✅ — pinta **conjuntos de faces**, que outras ferramentas leem (relax, automask) |

### G — TOPOLOGIA e MULTIRES

| ferramenta | nós | SculptGL | Blender |
|---|---|---|---|
| **Dyntopo no traço** | ✅ `refine_for_dab`, **antes** de carimbar | ✅ `dynamicTopology` (subdivide **e decima**) | ✅ `sculpt_dyntopo.cc` |
| **Decimar no traço** | ❌ (só refinamos) | ✅ `decimate` com `decFactor` | ✅ (colapso de aresta) |
| **Simplify** | ❌ | ❌ | ✅ `SIMPLIFY` — um pincel cujo único efeito é **rodar o dyntopo**, sem deformar |
| **Subdividir / níveis** | ✅ (`K`, `,`/`.`, `J`) | ✅ (multires) | ✅ (multires) |
| **Remesh por voxel** | ✅ (`V`) | ✅ | ✅ |
| **Displacement Eraser** | ❌ | ❌ | ✅ — **apaga o displacement de multires** de volta à base |
| **Displacement Smear** | ❌ | ❌ | ✅ — arrasta o displacement de multires pela superfície |

---

## §9 — ⚠️ A LISTA PEDIDA: o que o Blender tem e o SculptGL **NÃO** tem

**Vinte** ferramentas. Marquei com **★** as que nós também não temos (todas menos
uma) e com **◆** as que eu leria primeiro se fôssemos escolher.

### Deposição
1. ★◆ **Draw Sharp** — o Draw que lê as posições/normais **ORIGINAIS**. Vinco duro em vez de domo.
2. ★◆ **Clay Strips** — dab **retangular** com falloff parabólico no eixo do plano. É a ferramenta de blocagem por excelência do Blender.
3. ★ **Clay Thumb** — plano inclinado cujo ângulo **cresce ao longo do traço**.
4. ★ **Blob** — o Crease com o pinch para fora.
5. ★◆ **Layer** — demão de **altura constante** com deslocamento persistente por vértice, saturante e **apagável**.
6. ★ **Draw Vector Displacement** — a cor RGB de uma textura vira o vetor de deslocamento.

### Plano
7. ★◆ **Multiplane Scrape** — **dois planos** com um ângulo: faz a quina em V numa passada.
8. ★ **Scene Project** — projeta contra **outros objetos** da cena por raycast.

### Alisamento
9. ★◆ **Surface Smooth** — laplaciano de **dois passes** (HC) que **não encolhe**. É a resposta do Blender ao E7.
10. ★◆ **Slide Relax** — redistribui vértices **sobre** a superfície sem mudar a forma; respeita face sets.
11. ★ **Topology Slide** — desliza ao longo das arestas com direção autorada.
12. **Enhance Details** — ⚠️ **nós TEMOS o equivalente** (`Verb::Sharpen`); o SculptGL não.
13. ★ **Topology Rake** — alinha o fluxo das arestas com o traço (dyntopo).

### Puxar
14. ★ **Nudge** — empurra tangencialmente na direção do traço.
15. ★ **Thumb** — o Grab projetado no plano da vista.
16. ★◆ **Elastic Deform** — **Kelvinlets** (grab · bi/tri-escala · scale · twist): deformação elástica analítica.
17. ★ **Pose** — pose por cadeia com IK.
18. ★ **Boundary** — deforma a partir de um contorno aberto.
19. ★ **Cloth** — simulação de tecido local.

### Canais e topologia
20. ★ **Smear** e **Blur** de cor · **Draw Face Sets** · **Simplify** · **Displacement Eraser/Smear** — a cauda de multires e de face sets, que só existe porque o Blender tem multires e face sets.

### E na direção INVERSA — o que o SculptGL tem e o Blender não

Só **um**, e é pequeno: o **Local Scale** como *pincel* (no Blender inflar/encolher
uma região é o gizmo de transform ou o Elastic Deform em modo `scale`). ⚠️ E há
duas capacidades que o SculptGL **tem e não expõe**: o `smoothTangent` e o
`smoothAlongNormals` — código vivo que nenhuma UI dele alcança.

---

## §10 — O placar do catálogo

| | ferramentas | delas, que os outros dois não têm |
|---|---|---|
| **nós** | **16 verbos** | 1 (`Sharpen`, e o Blender tem o equivalente) |
| **SculptGL** | **13 tools** | 1 (`LocalScale` como pincel) |
| **Blender** | **33 tipos** | **20** |

⚠️ **A leitura honesta do placar:** nós cobrimos o SculptGL quase inteiro — falta
o `Paint` (cor por-vértice, cujo kernel **já está portado**), o `Transform`
(gizmo) e o **decimar** do dyntopo. Contra o Blender falta uma classe inteira de
ferramentas, e ela **não é uma lista de 20 features**: são **quatro ideias** que
geram quase todas —

1. **ler o estado ORIGINAL** (Draw Sharp, Grab, Rotate, Thumb, e o `!accum` do plano),
2. **um dab que não é um disco** (Clay Strips retangular, Multiplane com dois planos),
3. **estado PERSISTENTE por vértice** (Layer, Displacement Eraser),
4. **um modelo físico em vez de um deslocamento** (Elastic Deform/Kelvinlets, Cloth, Pose, Boundary).

*Escolher uma das quatro rende mais do que escolher cinco ferramentas da lista.*

---

# PARTE III — O CLAY, nos três, linha a linha

*(pergunta do Enio: nosso Clay é idêntico ao do SculptGL? o SculptGL e o Blender
são idênticos? os resultados são idênticos?)*

## §11 — As quatro perguntas

### 11.1 — Nosso Clay × SculptGL: **o kernel é idêntico ao ULP; o produto não**

O Clay do original **não é um verbo**: o `Brush.js:46-53` computa a normal e o
centro de área, empurra o centro por `sqrt(r2)·0,1` ao longo da normal, e chama
**o `Flatten.prototype.flatten`**. O nosso `Verb::Clay` faz a mesma coisa por
outra álgebra — em vez de mover o ponto do plano, **subtrai o mesmo comprimento
da distância assinada** (`d = signed_distance − sign·raio·0,1`), o que é a mesma
translação de plano.

| item | nós | SculptGL | igual? |
|---|---|---|---|
| deslocamento do plano | `raio · 0,1` | `sqrt(r2) · 0,1` | ✅ (o `sqrt(r2)` **é** o raio) |
| lateralidade | `if d·sign < 0 { move } else { fica }` | `if (distToPlane·comp > 0) continue` | ✅ o mesmo lado |
| a lei | `p += n · (−d · w)` | `vAr -= aNormal · (falloff·distToPlane·intensidade·máscara·alpha)` | ✅ termo a termo |
| ponto do plano | `area_center` ponderado pela máscara | `areaCenter` idem | ✅ |
| normal | `area_normal` sobre o conjunto **frontal** | idem | ✅ |
| origem da distância | viva / `pre` conforme o Accumulate | `vAr` / `vProxy` idem | ✅ |
| Ctrl | desce o plano **e** inverte o lado | `_negative ? -off : off` + o `comp` | ✅ |

**Medido** (atlas, um dab, 272 vértices, `Falloff::Plateau`, intensidade 0,5):
nosso **0,006490** · referência **0,006490** · razão **1,00×** · `|diferença|`
**5,960e-8** = **um ULP do `f32`**.

⚠️ **Mas o atlas mede com a curva da REFERÊNCIA, e o produto ship outra** (D1).
E aqui a divergência é **exatamente calculável, sem sonda**: o deslocamento é
`d · w` com `w = curva · alpha · intensidade · máscara`, ou seja **LINEAR na
curva** — então a razão por-vértice **é** a tabela do D1: `1,000×` no centro,
`1,222×` a meio raio, `1,436×` a `7/8`. *Com `Falloff::Plateau` no painel o Clay
do produto é o do original ao ULP; com o default de fábrica ele é o mesmo Clay
com um pincel mais magro.*

⚠️ **UMA divergência de comportamento, e ela é nossa:** com a pegada
**inteiramente de costas** (um dab que só pegou o outro lado da silhueta) o
`areaNormal` do original devolve `undefined` e o `stroke()` faz `return` — *ele
não esculpe*. Nós caímos num **fallback** que ajusta o plano sobre a pegada
inteira e esculpimos. Está documentado no `fit_plane`, e é deliberado.

⚠️ **E o multi-dab não é linear:** o plano é re-ajustado a cada dab sobre a malha
VIVA, então o barro que o dab `k` deposita levanta o plano do `k+1`. A razão
acima vale **por dab**; ao longo de um traço ela compõe.

### 11.2 — SculptGL × Blender: **não, e a diferença é estrutural**

Não é afinação de constante: são **cinco** decisões diferentes.

| | SculptGL (`Brush` + `Flatten.flatten`) | Blender (`clay.cc::do_clay_brush`) |
|---|---|---|
| **o PONTO do plano** | `areaCenter` — o **centroide** ponderado da pegada frontal | ⚠️ **`cache->location_symm`, o ponto do CURSOR.** O `area_co` é computado (`clay.cc:167`) e **nunca lido** |
| **lateralidade** | **um lado** (`distToPlane·comp > 0 → continue`) — o Clay dele só **ENCHE** | **os dois** — `calc_closest_to_plane` não filtra: o Clay dele **corta a crista E enche o vale** |
| **o deslocamento** | `sqrt(r2)·0,1`, constante embutida, sobre o raio **VIVO** | `\|initial_radius · plane_offset\|` — knob do usuário, sobre o raio do **PEN-DOWN**, opcionalmente × pressão |
| **a força** | `intensity`, **linear** | `0,25 · alpha² · flip · pressão⁴ · overlap · feather` |
| **os fatores** | curva × máscara × alpha | curva × máscara × **auto-máscara** × front-face **contínuo** × hardness × textura |

⚠️ **E há uma divergência DENTRO do próprio Blender que quase escrevi errado:**
o **Clay** ancora no cursor, e o **`PLANE`** (o Flatten/Fill/Scrape unificado)
ancora no **centroide** (`plane.cc:502` + `:383`) — e re-varre os nós em volta do
centro do plano com o comentário *"its effective center often deviates from the
cursor location"*. Os dois pincéis de plano do Blender **não usam a mesma
âncora**. (E o `PLANE` tem outra: `if (is_zero(grab_delta_symm)) return;` — ele
**não faz nada no primeiro dab**, porque o frame local dele é orientado pela
DIREÇÃO do traço.)

### 11.3 — Os resultados são idênticos?

| par | veredito | número |
|---|---|---|
| **nós × SculptGL**, kernel, fixture da paridade | ✅ **sim, ao ULP** | `\|diff\| 5,960e-8` |
| **nós × SculptGL**, produto de fábrica | ❌ não — a **curva** | razão por-vértice `1,000×` → `1,436×` (exata, por linearidade) |
| **nós × SculptGL**, pegada 100% de costas | ❌ não — eles não esculpem, nós sim | — |
| **SculptGL × Blender** | ❌ **não, nem estruturalmente** | 5 decisões diferentes; três delas mudam o desenho, não o número: **onde o plano ancora**, **se ele corta ou só enche**, e **a lei da força** |

⚠️ **A leitura que importa:** *"Clay"* nomeia **três ferramentas diferentes**. A
do SculptGL é *encher até um plano levantado sobre o centroide*. A do Blender é
*achatar contra um plano levantado sobre o cursor, com pressão à quarta*. A nossa
é a do SculptGL — de propósito, e ao ULP.
