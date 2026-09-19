# 11 — A camada de ESTILO: os botões para mentir de propósito

> **Ordem do dono, 2026-09-19:** *«8 e depois do smoke o 7»* — o ingrediente **`8`** do
> [`01`](01_o_alvo_decomposto.md) antes do **`7`**, que é o pós.

A [`W8`](03_o_plano.md) do plano. O [`01`](01_o_alvo_decomposto.md) §1 mede a tese do alvo e ela é
esta: *«o que faz aquele jogo ser bonito não é um shader — é o pipeline ser fisicamente honesto para
a direcção de arte poder depois mentir de propósito»*. As cinco waves anteriores construíram a
honestidade; esta constrói os botões para a trair.

```text
material + luz  →  [ESTILO]  →  olhar (exposição + vista)  →  sRGB
 (a física)        (a mentira)    (a ph2d-view-transform)
```

## §1 — Os quatro botões, e porque são estes quatro

| botão | o que ele mente | o que o olho lê |
|---|---|---|
| **Rim** (contorno) | acrescenta luz onde a superfície **foge** do olhar, `(1 − \|N·V\|)^largura` | a peça **descola do fundo** — é o que separa um personagem de um cenário |
| **Curvature** (aresta / cova) | tinge pelo que a peça **É**: aresta contra vinco | o aspecto pintado à mão, **sem ninguém pintar um mapa** |
| **Zones** (a grade) | uma tinta para o que é escuro, outra para o que é claro | a grade de cor de um filme — sombras frias, luzes quentes |
| **Indirect Saturation** | satura (ou lava) **só** a luz que ricocheteia | o sangramento de cor da [`W5`](08_a_luz_indirecta.md) deixa de ser tímido, sem tocar nas lâmpadas |

⛔ **E um quinto ficou de fora com o mecanismo escrito: o CONTORNO desenhado** (a tinta na
silhueta). Ele **não é uma lei por ponto** — precisa dos vizinhos no ecrã, logo é um **passe**, e
pô-lo na crate da lei obrigaria-a a receber um G-buffer. *Ela deixaria de ser a lei que os dois
motores partilham, que é a única razão de ela existir.* Fica nomeado na [`W8`](03_o_plano.md).

## §2 — ⭐⭐⭐ A omissão é a identidade, e por CONSTRUÇÃO

Toda tinta é uma **cor** cujo valor de fábrica é o **branco**; toda soma tem um peso cujo valor de
fábrica é **zero**. ⇒ um quadro que não pediu estilo nenhum sai **byte a byte** o de antes, **sem um
ramo pelo caminho** — e é isso que faz as paridades já pagas ([`08` §12](08_a_luz_indirecta.md), a
`100,000 %`) ficarem de pé sem serem re-medidas.

Medido nos dois motores: `0` píxeis fora, pior `0`.

### ⛔⛔ E uma premissa minha caiu aqui, por DUAS mutações SOBREVIVENTES

O cabeçalho da crate dizia que a forma ingénua `a·(1−w) + b·w` **não serve**, porque `(1−w) + w` não
seria `1` para todo `w`. Duas mutações que a instalavam nas tintas passaram os gates todos, e a
varredura explicou porquê:

> `(1−w) + w` dá **`1,0` exactamente** em `2 044 824` amostras de `f32` em `[0, 1]` e em `200 000`
> acima — como tem de dar, porque o erro daquela soma é no máximo **meia ULP** de `1` e o desempate
> é para o par, que é o próprio `1,0`.

⭐⭐⭐ **O perigo real é o que a mutação que SANGROU nomeia: reconstruir `b` a partir de `a + (b − a)`
quando `a ≠ b`.** Nas tintas os dois extremos são iguais no ponto de fábrica (as duas brancas), logo
o parêntesis é **zero** e as duas redacções são exactas; na saturação eles são a **luminância** e o
**rgb**, que são diferentes, e ali a reconstrução perde bits por cancelamento — `s = 1` deixaria de
devolver a entrada.

⇒ a lei que fica: **multiplicar o valor que se quer de volta, nunca reconstruí-lo por diferença.**

## §3 — ⭐⭐⭐ O SINAL da curvatura já existia, e era deitado fora uma linha antes

`H = ∇²f / 2` é **positivo numa bossa e negativo numa cova**. O consumidor que a estreou — a
subsuperfície MACIÇA da [`10`](10_a_luz_que_atravessa_a_peca.md) — pede um **comprimento** (a
referência do MaterialX estima-o por `length(fwidth(N))`, que é `≥ 0` por construção), e por isso o
porte escrevia `abs(…)` **dentro** da porta.

⇒ *ali era a resposta certa, e a perda ficava dentro da função em vez de no chamador.* Hoje a
subsuperfície toma o módulo do lado dela — a imagem que ela pinta é **byte a byte a mesma**, porque
tomar o módulo antes ou depois de guardar dá o mesmo `f32` — e a tinta por curvatura lê o sinal, que
é **a diferença entre uma aresta e um vinco**: entre um contorno e uma sujidade.

**Medido na fixtura da wave** (`128×96`, câmera de omissão): `3 631` píxeis acertados, `584` deles
com curvatura **negativa** (`16 %`), `min = −3,361` — que é `−1/0,30`, o raio da cratera **ao
terceiro decimal** —, `p50 = +1,819` (a bola, `1/0,55`) e `max = +11,7` (o lábio com filete).

### ⚠️ E a curvatura é adimensional antes de chegar ao estilo

Ela mede-se em `1/comprimento`. Um botão calibrado nisso **mudaria de sentido ao escalar a peça** —
a mesma quina daria outra tinta numa peça de `0,3` e numa de `3`. Multiplicada pelo raio da bola que
envolve a peça, o que o estilo lê é *«quantas vezes esta zona é mais curva do que a peça inteira»*.
⭐ Numa esfera ela vale **`1`** em todo o lado; num filete de `1/10` do raio, **`10`**; numa face
plana, **`0`**.

## §4 — ⭐⭐⭐ A lição do §24, aplicada ANTES de escrever a primeira linha

O [§24 da `10`](10_a_luz_que_atravessa_a_peca.md) é a auditoria mais cara que este módulo pagou:
duas waves seguidas foram entregues ao dono com tabela, gates e prova de mutação, e viviam **num ramo
que o produto não corre**. Esta wave foi escrita contra isso:

1. **O estilo entra na ASSINATURA.** `ph2d_field_render::Presentation { look, style, piece_radius }`
   substitui o `Look` solto no `shade_render` **e** no `gpu_frame::paint` — os dois motores recebem o
   **mesmo tipo**, e esquecê-lo é **erro de compilação**. ⛔ Uma função-irmã *«com estilo»* seria a
   segunda porta pela qual o defeito volta.
2. **Uma apresentação, montada UMA vez e ANTES do ramo do dispositivo.** Com gate **estrutural**, que
   corre sem placa e sem cena.
3. **A curvatura é medida quando ALGUÉM a lê, e são DUAS portas somadas** — o material ou o estilo.
   ⛔ Perguntar só ao material faria o artista mexer na tinta de aresta e a peça não mudar um pixel:
   *a grandeza que o botão escolhe nunca teria sido medida.*

## §5 — ⛔⛔⛔ A paridade achou uma divergência PRÉ-EXISTENTE que só um consumidor sensível revela

O gate de placa reprovou à primeira, com `3`–`5` bytes em oito píxeis. A **atribuição botão a botão**
diz que a lei do estilo não tem culpa nenhuma:

| o que se liga | píxeis acima de `1` | pior |
|---|---:|---:|
| a fábrica | `0` | **`0`** (byte-idêntico) |
| só o contorno | `0` | `1` |
| só as zonas e a saturação | `0` | `1` |
| a tinta por curvatura, `nitidez 0,2` | `0` | `1` |
| a tinta por curvatura, `nitidez 1` | `163` | `7` |
| a tinta por curvatura, `nitidez 2` | `168` | `13` |
| a tinta por curvatura, `nitidez 8` | `168` | **`45`** |

⭐⭐⭐ **A contagem SATURA em ~`165` e a magnitude cresce LINEARMENTE com a nitidez** — a assinatura de
*uma diferença pequena na CURVATURA, amplificada pelo ganho*. Os dois motores medem-na por caminhos
diferentes (a fita achatada da `ph2d-field-eval` contra o `field()` do WGSL), e **a divergência já lá
estava**: o único consumidor que ela tinha passa a curvatura por uma **tabela pré-integrada com
piso** (`max(κ, 0,01)`), que a **satura**. *Esta tinta é o primeiro consumidor LINEAR nela, e por
isso o primeiro instrumento que a vê.*

⏳ **Dívida NOMEADA, e não é desta wave:** falta o instrumento que mede a **curvatura** nos dois
motores, e não o pixel. ⛔ **A barra não foi afrouxada para engolir os `13` bytes** — a tinta é medida
onde ela não amplifica, e quem fechar aquela dívida sobe a nitidez desta linha.

### §5-bis — ⛔⛔⛔ A dívida FECHOU, e a leitura acima estava ERRADA em dois pontos

A auditoria de 2026-09-19 construiu o instrumento que falta (`curvatura_parity_tests.rs`, que mede
`H` nos **três** avaliadores no mesmo ponto: `cpu` · `gpu` · um árbitro `f64` que nenhum motor
pinta). Duas frases do parágrafo acima não sobrevivem à medição:

**(1) ⛔ «a fita achatada da `ph2d-field-eval` contra o `field()` do WGSL» — a premissa do `f64` é
FALSA neste caminho.** O `f64` é o `Field::at` (ponto a ponto, serve sondas); quem a
[`curvatura::curvaturas`] chama é `Hybrid::eval`, que é **`f32` em lote**. ⇒ os dois motores são
**os dois `f32`**, e o que diverge são **dois avaliadores `f32` do mesmo campo**.

**(2) ⛔⛔ «a divergência chega ao pixel» — ela chega à BORDA, e o clamp absorve o resto.**
Classificados pelo **alfa**, dos `86` píxeis divergentes **`84` são de borda anti-serrilhada e `2`
são de miolo** — e a contagem é a **MESMA** a `nitidez 2` e a `8`; só a magnitude cresce. O
mecanismo: `ΔH ≈ 9,8e-4`, a tinta é `clamp(H·R·nitidez, ±1)`, e a `nitidez ≥ 1` **zero** píxeis de
cobertura cheia desta peça estão dentro da banda. *A tabela lia-se como «a grandeza chega ao corpo»
e o que ela mede é a orla.*

⭐⭐⭐ **E a causa fecha em forma fechada.** A divergência é **UM ULP** da avaliação de campo,
amplificado por `1/(4ε²) = 16 403×` pelo cancelamento da segunda diferença:

```text
ULP(0,55)/(4ε²) = 9,7769e-4   ← medido na bola      9,778e-4
ULP(0,30)/(4ε²) = 4,8884e-4   ← medido na cratera   4,890e-4
```

⇒ **não é afinável**: é o que custa medir `∇²f` com `ε = 0,0064 · raio` em `f32`. ⭐ E a amplificação
ser `1/(4ε²)` tem uma consequência de desenho: **um `ε` maior divide a divergência pelo quadrado** —
a mesma alavanca que suaviza a borda da tinta (a §10) cura esta dívida de graça.

⭐ **A lei do estilo está ILIBADA com número:** a tinta por curvatura mede **`0` ULP na lei** a
`nitidez 2` e a `8` sobre `3 456` amostras com a curvatura **entregue** ⇒ `100 %` do que ela move no
pixel é a GRANDEZA. *Mexer na lei seria afinar o inocente.*

⛔⛔ **E o instrumento apanhou um gate VERDE sobre uma promessa falsa:** a placa **contrai `a*b + c`
num `fma`** — o `st_luma` bate com a forma **fundida** em `1 680/1 680` amostras e com a **solta** em
`1 463`. O cabeçalho da `ph2d-style` promete *«nenhuma conta desta crate usa `mul_add`»*, e isso é
honrado no **FONTE** e violado pelo **COMPILADOR**; o `nenhuma_conta_desta_crate_e_fundida` está
verde sobre um produto que corre fundido. ⛔ A contracção **não é exprimível em WGSL hoje** ⇒ a cura
honesta é o tecto **derivado** com a sonda de atribuição ao lado. Mais duas propriedades do
controlador, medidas nesta placa: o `pow` do WGSL **não** é o `powf` do Rust (`exp2(y·log2 x)`,
`870/1 680` iguais, pior `44` ULP, o erro a escalar com o expoente) e os **subnormais são esvaziados
a zero**.

## §6 — ⛔ O MATCAP fica de fora, por decisão

Ele é *a luz do OLHO* — um auxiliar de modelação que lê **forma** —, e o estilo é direcção de arte
sobre um pipeline fisicamente honesto. **Tingir o viewport de modelagem com a grade do filme faria o
artista medir a peça através de uma mentira.**

⇒ *o olhar governa os dois* (ele é a gestão de cor da cena, [`05` §1](05_o_modo_render_do_modelador.md));
*o estilo governa o Render.* Há um **censo** com a excepção nomeada — e ela apareceu porque o gate a
acusou: a 1.ª redacção exigia zero passagens do olhar por fora da apresentação e reprovou sobre
produto correcto.

## §7 — Onde o estilo VIVE, e a dívida que ele partilha

Ele é **estado de VISTA**, ao lado do olhar (`field3d_view::View::style`): sobrevive a fechar o
painel, **não** entra no undo nem no arquivo. ⚠️ O gate do `View::of` disparou ao vê-lo nascer, que é
o que o `CLAUDE.md` §5.1 promete, e a resposta está escrita no campo.

⏳ **Ele passa a DOCUMENTO no mesmo dia que o olhar** — quando houver uma saída de render que os
grave ([`04` §4](04_a_remedicao_contra_a_arvore.md)). *A dívida é uma e é partilhada; separá-las agora
poria metade da apresentação no ficheiro e metade fora.*

## §8 — O que o artista vê

Dez fileiras na secção **Style**, no fim do painel do modelador, **só no modo Render**: cinco cores
(contorno · aresta · cova · sombras · luzes) e cinco números (força e largura do contorno · nitidez
da curvatura · pivô das zonas · saturação da indirecta).

⭐ **A tabela do painel é DERIVADA da arrumação do uniforme** (`ph2d_style::wgsl::pack`): o
`Param::Style(n)` carrega a **posição**, e uma escrita é *desempacota, escreve a posição, empacota*.
⛔ Uma numeração própria seria a segunda resposta à mesma tabela.

⚠️ **Cada tecto nomeia o recurso** (`CLAUDE.md` §0.0), e os três que ainda não têm tabela ficam
**nomeados como dívida da [`W9`](03_o_plano.md)**, que é a wave da medição.

**Smoke:**

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling && env PH2D_FIELD_SMOKE=35 cargo run -p ph2d-host-desktop --profile smoke
```

⚠️ **O enquadramento da cena foi FOTOGRAFADO** (`docs/Components/ferramentas/fotografa_cena.sh`) e a
foto apanhou três coisas que a suíte não vê — as covas viradas de lado, a barra a sair do ecrã, e a
armadilha de fotografar com o `$HOME` do dono (que fotografa a **bancada** dele e não a cena). Ver o
doc-comment da `cena_35`.

## §9 — ⛔⛔⛔ O smoke reprovou: **cinco cores, UM controlo**

Report do dono (2026-09-19): *«se modifico qualquer cor em style, todas mudam ao mesmo tempo»*.
Reproduzido no caminho do produto, com o gate a dizer o par:

```text
panel.model3d.style.rim_color  e  panel.model3d.style.convex
sao o MESMO controlo (id 2905579657539185078)
```

### O mecanismo

O selector de cor da casa é **UM** e flutua sobre o canvas; um painel entra nele **registando o
`NodeId` da amostra**, e lê de volta `picker_target() == Some(id)`. Esse id era derivado **dentro**
do `paint_swatch` por um `match` cujo braço final dizia, por escrito:

> *«uma amostra sobre um param sem índice não existe hoje; `0` é a resposta estável»*

⭐ **Era verdade no dia em que foi escrita.** Ficou falsa quando esta wave trouxe **cinco** fileiras
de cor cujo `entity` é `0` **por desenho** — o estilo é da CENA e não de entidade nenhuma (§7). As
cinco caíam no braço final, recebiam `campo = 0` e partilhavam
`hash("model3d.color.swatch.0.0")` ⇒ com o selector aberto numa delas, **as cinco** liam *«aberto em
mim»*, **as cinco** comparavam a cor escolhida com a sua e **as cinco** pediam a edição.

### ⚠️⚠️ Porque os seis gates desta wave ficaram VERDES

Eles medem a **LEI** (`with_colour`, a arrumação, os tectos) e o **DRENO**
(`apply_intents_for_test`, com a âncora **já certa**). O defeito vive **ENTRE os dois** — na
IDENTIDADE com que a fileira é pintada —, logo o gate de costura entra **abaixo** da rotura. É o
ponto cego que o `CLAUDE.md` §5.0 nomeia sobre si mesmo, aqui noutra forma:

> *nenhum instrumento perguntava se duas fileiras são o **MESMO** controlo.*

### ⛔ E a segunda metade estava na outra ponta

A lista que fecha um selector órfão (`close_a_stranded_picker`) derivava o id por um **SEGUNDO
`match`**, que só conhecia `Param::Material`. Duas respostas à mesma pergunta — *«qual é o id desta
amostra?»* — e elas **já divergiam para a LUZ** desde a wave dela: um selector aberto sobre a cor de
uma lâmpada **nunca era fechado**, e ficava a flutuar sobre uma amostra que ninguém pinta — que é
exactamente o controlo morto que aquela função existe para impedir.

### A cura: uma porta, dois leitores

`ph2d_panel_model3d::swatch_id(&ParamRow) -> Option<NodeId>`:

| família | id | porquê |
|---|---|---|
| `Material(k)` · `Light(k)` | `model3d_color_swatch(entity, k)` | **inalterado** ⇒ as amostras que já shipavam mantêm o id ao bit |
| `Style(slot)` | `model3d_style_swatch(slot)` | espaço de nomes **próprio**: o sujeito é a cena. ⛔ Sem ele a não-colisão dependeria do acidente de `Entity::to_bits()` nunca valer `0` |
| qualquer outra | `None` | a fileira cai para o controlo normal — *visível e diferente lê-se como uma falta; um id partilhado em silêncio foi o que este report custou* |

⭐ De graça, a lista do selector órfão passa a conhecer **a luz e o estilo**.

### As quatro réguas novas

| gate | o que ele prende |
|---|---|
| `cada_cor_do_estilo_tem_uma_amostra_so_sua` | as cinco cores publicadas pelo **produto** dão cinco ids distintos, com piso de população |
| `a_amostra_do_estilo_nao_colide_com_a_de_um_objecto` | nos dois sentidos |
| `o_id_de_uma_amostra_e_derivado_num_sitio_so` | censo **derivado** sobre os ficheiros de produção: um terceiro leitor a cunhar o próprio id reprova com o endereço |
| `uma_familia_sem_id_devolve_none` | com **controlo positivo** — senão uma porta que devolvesse `None` a tudo passaria, apagando as amostras que funcionam |

Prova de mutação **4 de 4**, com o controlo (trocar a ordem de duas famílias equivalentes) a **não**
sangrar.

⚠️ **E o arnês mentiu duas vezes antes de dizer a verdade**, as duas formas já escritas nesta casa:
um filtro que casou **zero** testes imprimiu `ok` (lê-se como *«sobreviveu»*), e o parser contava
`running N tests` quando com **um** teste o libtest escreve `running 1 test`, no **singular**.

## §10 — ⛔⛔⛔ A AUDITORIA DE 2026-09-19: *«bordas muito duras sem ajustes finos»*

Ordem do dono, com foto: *«Edge tint e Cavity tint com bordas muito duras sem ajustes finos, não me
parece certo. Zone pivot não sei para que serve mas parece morto. Auditoria completa com agentes de
todo Style. Se for necessário compare com um app que faz bem feito.»* Quatro frentes em paralelo: a
LEI · o ORÁCULO · o PAINEL · os DOIS MOTORES.

### §10.1 — O campo de curvatura não é contínuo: é um punhado de PLATÔS

Na `=35` (`piece_radius = 0,676`), `H·R` lê-se assim, e **cada platô é uma peça da cena**:

| população | `H·R` | de onde |
|---|---:|---|
| faces planas da caixa (`4,5 %`) | `0` | — |
| a bola (`p50`) | `1,501` | `0,676/0,45` ao 3.º decimal |
| as três crateras (`12,5 %`) | `−3,38` · `−4,22` · `−5,20` | `−R/0,20`, `−R/0,16`, `−R/0,13` |
| os filetes (`19,0 %`) | `11,3` · `33,8` | o filete `0,06` e o `round` `0,02` |

**A prova de que os saltos são DESCONTINUIDADES e não um campo suave mal amostrado** — o salto de
`H·R` entre píxeis vizinhos, em três resoluções:

| resolução | p50 | p99 | **max** |
|---|---:|---:|---:|
| 320×240 | `0,113` | `8,56` | **`19,30`** |
| 640×480 | `0,025` | `5,77` | **`17,61`** |
| 1280×960 | `0,006` | `2,98` | **`17,09`** |

⇒ o `p99` **encolhe** ao dobrar a resolução (campo suave) e o **max NÃO** (`19,3 → 17,1` sobre `4×`
de píxeis). *Um campo suave amostrado com metade do pixel tem metade do salto; um degrau tem o
mesmo.*

⭐⭐⭐ **A consequência é a lei inteira numa frase:** *uma função POR PONTO de um campo constante por
troço é constante por troço* ⇒ **nenhum botão aplicado a `H` pode produzir um gradiente**; só um
operador que olhe à VIZINHANÇA pode. E o único operador de vizinhança do caminho é o **`ε` do
estêncil**, que é escolhido por PRECISÃO NUMÉRICA (`~ulp^{1/4}`, `0,64 %` da peça) — *um acidente de
diferenciação, não um controlo.*

### §10.2 — O número que dá razão ao olho do dono

Degrau de **byte** entre píxeis vizinhos, com o CONTROLO ao lado (a mesma imagem sem estilo):

**CONTROLO (sem estilo):** p50 `1` · **p99 `16`** · max `73`.

| `Curvature Sharpness` | saturados | degrau p99 | **vs controlo** | p99 **sem** os px de borda |
|---:|---:|---:|---:|---:|
| `0,0625` | `1,9 %` | `67` | **`4,19×`** | `41` |
| `0,25` | `17,4 %` | `135` | `8,44×` | `131` |
| **`1,00` (fábrica)** | **`86,8 %`** | **`169`** | **`10,56×`** | `161` |
| `8,00` | `95,3 %` | `205` | `12,81×` | `203` |

⇒ **a tinta põe um penhasco de `169` bytes numa imagem cujo próprio sombreamento nunca passa de
`16`.**

### §10.3 — ⛔ TRÊS explicações plausíveis, construídas e REFUTADAS

| hipótese | discriminador | veredito |
|---|---|---|
| «é o `clamp` a saturar» | a `nitidez 0,0625` só `1,9 %` satura | ⛔ **refutada** — o penhasco já é `4,19×` o controlo |
| «é o anti-serrilhado, que re-sombreia com 4 normais e UMA curvatura» (ele **faz** isso) | excluir os `1 825` píxeis de borda (`2,9 %`) | ⛔ **refutada** — p99 `169 → 161` |
| «falta um JOELHO SUAVE no `clamp`» *(a minha própria proposta)* | `smoothstep` no lugar do `clamp`, mesma curvatura | ⛔ **refutada, e PIORA**: `168` contra `162` a `nitidez 1`. *Um joelho actua no domínio do VALOR e a dureza vive no domínio do ESPAÇO* |

### §10.4 — ✅ A escala espacial CONFIRMADA, com a autoridade medida

| `ε/raio` | degrau de byte p99 | **vs controlo** | erro na esfera |
|---:|---:|---:|---:|
| **`0,0064` (hoje)** | **`169`** | `10,56×` | `0,16 %` |
| `0,0256` | `129` | `8,06×` | `0,54 %` |
| **`0,0512`** | **`103`** | **`6,44×`** | `1,09 %` |
| **`0,1024`** | **`66`** | **`4,12×`** | `2,15 %` |
| `0,2048` | `26` | `1,62×` | `4,11 %` |

⭐⭐ **`ε` tem `6,5×` de autoridade sobre a dureza (`169 → 26`); o `Curvature Sharpness` tem `1,5×`
(`135 → 205` no curso inteiro).** *O botão que existe não é o botão da grandeza de que o dono se
queixa.*

⛔ **E há uma PAREDE:** a `ε/raio = 0,2048` o `p05` fica **positivo** — as crateras deixam de ser
côncavas e a `Cavity Tint` **morre**. Janela útil medida: **`ε/raio ∈ [0,03 ; 0,10]`**.

### §10.5 — ⭐⭐⭐ A fábrica está EXACTAMENTE no ponto de saturação

`Point::curvature` é `H · raio_da_peça`, **que numa esfera vale exactamente `1`** ⇒ a nitidez de
fábrica (`1,0`) põe uma peça arredondada **precisamente** onde o clamp satura. Medido: a `nitidez 1`
há `2` de `4 593` amostras dentro da banda de transição; a `0,2` há `4 231`.

⇒ **o valor de fábrica escolhe o lado duro da lei**, e nenhum dos dois motores discorda (os `lados
trocados` são `0` em toda a nitidez).

### §10.6 — O ORÁCULO, e o que ele tem que nós não temos

⚠️ **Triagem de licença primeiro, e ela PARTE A MEIO** (`docs/3DModeling/cleanroom/fixtures/`):
OpenVDB (**MPL-2.0**) e VTK (**BSD-3**) são portas ABERTAS e respondem só à metade do **estimador** —
que o nosso já **bate**. A metade que interessa (*que controlos o artista tem*) só existe num
artefacto **walled**, logo ele foi **CORRIDO**, nunca lido.

⭐⭐ **Achado para o arsenal:** o alvo alcança o toolkit de level-set do OpenVDB **com ZERO GL**, por
nós de geometria avaliados no depsgraph ⇒ este repo passa a ter um **oráculo de SDF sem interface**.

**Medido, sobre uma peça NOSSA** (caixa `b = 1,0`, 12 arestas filetadas `r = 0,2`, verdade em forma
fechada `H = 1/(2r) = 2,500`):

| | **nosso** | oráculo SDF (grade) | oráculo malha |
|---|---:|---:|---:|
| erro no pico | **`0,22 %`** | `1,2`–`5,6 %` | — |
| oscilação sobre curvatura CONSTANTE | **`0,1 %`** | `0,8`–`7,4 %` | **`40 %`** (fábrica) · **`205 %`** (sem borrão) |

⇒ **`7`–`74×` mais limpo que a grade dele e `~400×` mais limpo que a rota de malha.** ⭐⭐ E isso tem
consequência de desenho: **boa parte do borrão que ele oferece serve para esconder o ruído do
estimador dele** — nós não precisamos dele por essa razão, só pela artística, logo **um raio menor
chega-nos do que a ele.**

**A largura da rampa dele, em % do arco do filete:** `10 %` (sem escala) → **`62 %`** (escala
máxima), com a amplitude **parada** (`H·R` pico `4,57 → 4,28`) ⇒ **`6,0×` de faixa de suavidade**.
A NOSSA, na mesma peça: `4,1 %` abaixo da saturação e **`1,2 %`** na fábrica.

⇒ **a borda que o dono fotografou é `8×` mais dura que a mais dura que o alvo consegue produzir, e
`50×` mais dura que a mais suave.**

**Controlos que ele tem e nós não:**

| | ele | nós |
|---|---|---|
| **raio / distância da leitura** | `matcap_ssao_distance` (`unit = LENGTH`) · `blur_iterations` (`0..40`) | ⛔ **nenhum** |
| intensidade de aresta e de cova | **quatro** factores (`ridge`/`valley` × ecrã/mundo) | duas cores e **UMA** nitidez partilhada |
| duas escalas em simultâneo | `cavity_type = BOTH` — *ele julgou que uma não chega* | — |

⛔ **O que é INEXPRIMÍVEL aqui, com o mecanismo:** o `WORLD` dele é **SSAO sobre profundidade** — um
integral de VISIBILIDADE, não de curvatura; portá-lo é uma **segunda lei**, não «acrescentar um
raio». E o `SCREEN` dele deriva de derivadas de ecrã ⇒ **muda ao rodar a câmera**, divergência que o
`curvatura.rs` já declara a nosso favor.

⚠️⚠️ **E se um raio entrar, ele pertence à MEDIDA e não ao CAMPO, com número:** filtrar o SDF **move
a superfície** (`dshift` até `0,84` voxel); filtrar a curvatura medida não move nada — **e os dois
dão exactamente as mesmas larguras de rampa**. Não há razão de precisão para pagar o deslocamento.

### §10.7 — ⛔⛔ QUATRO das dez fileiras são INERTES no estado em que o painel ABRE

Censo pelas portas do produto, população **derivada** de `estilo::rows()`:

| fileira | Δ do estado **FÁBRICA** | Δ do estado **ARMADO** | porquê |
|---|---:|---:|---|
| **Rim Color** | **`0 · 0`** | `25 756 · 184` | `rim.strength = 0` ⇒ `× 0` |
| **Rim Width** | **`0 · 0`** | `63 347 · 247` | a mesma causa |
| **Curvature Sharpness** | **`0 · 0`** | `60 751 · 246` | as duas tintas brancas ⇒ multiplica por `1` |
| **Zone Pivot** | **`0 · 0`** | **`63 347 · 160`** | `shadow == highlight` ⇒ o parêntesis é **exactamente zero** |

⭐⭐ **A resposta ao *«Zone pivot parece morto»* é literal: na configuração de fábrica ele é
matematicamente um no-op** — e é a **mesma lei** que faz a omissão ser a identidade ao bit (§2).
⭐ Mas ele **não é fraco**: armado, é o botão **MAIS FORTE da camada** (de ponta a ponta do curso,
`63 347` px = `100 %` da peça, pior byte **`233` de `255`**).

⛔⛔ **E o cabeçalho do `estilo.rs` cita a lei que o próprio ficheiro não implementa:** *«uma
affordance que não pode ser honrada é pior do que nenhuma, que é a lei que o `ParamRow::inert` já
escreve para as fileiras do material»*. O `ParamRow::inert` existe, tem **seis** frases prontas em
`ph2d-i18n/src/model3d_inert.rs`, e o doc dele carrega a **decisão do dono de 2026-09-18** — um dia
antes deste report: *«um controlo travado e sem razão à vista lê-se exactamente como um controlo
morto»*. **Zero fileiras de estilo a usam** (as dez passam `inert: None` incondicionalmente).

⚠️ **E há uma SEGUNDA rota para o mesmo estado, que esteve viva até às `09:51` de hoje:** o defeito
da §9 (cinco cores, um controlo) escrevia a MESMA cor nas cinco amostras ⇒ pôr a `Cavity Tint` a
azul punha `shadow = highlight = azul` e **matava o pivô ao bit**. *Se o report veio antes daquela
cura, os dois relatos do dono são UM defeito* — o discriminador é a hora da foto contra `61e9d7e8c`.

### §10.8 — Os TECTOS, e três deles são palpites

| knob | tecto | veredito | o número |
|---|---:|---|---|
| **Rim Width** | `64` | ✅ **MEDIDO e correcto — o único que sobrevive** | espessura do fio a `1920×1080`: `63,2 px` (`w=3`) · `6,1` (`16`) · **`0,535`** (`64`); o cruzamento de 1 px fica em `w ≈ 45–48`. E `w = 128` dá a imagem de `64` **ao bit** (o `sanitized()` corta) |
| **Rim Strength** | `4` | ⚠️ defensável como PRODUTO | o **pico** satura por volta de `1,5`–`2`; a **ÁREA** não satura (`18 270 → 47 626` px de `1` a `64`). Tecto de gosto, **com tabela** |
| **Curvature Sharpness** | `8` | ⛔ **PALPITE — a medição diz `2`** | `s = 1` entrega **`99,0 %`** do que `s = 8` entrega ⇒ **`87,5 %` do curso compra `1 %` do efeito**. *É isto o «sem ajustes finos»* |
| **Zone Pivot** | `4` | ⛔ **PALPITE, e a faixa útil é OUTRA** | o contraste da grade **pica em `0,5`** (amplitude de `h` p05–p95: `0,204` · `0,395` · **`0,438`** · `0,371` · `0,164` a `0,05 · 0,18 · 0,5 · 1 · 4`) e já está a **cair** no tecto |
| **Indirect Saturation** | `4` | ⛔ **PALPITE — o efeito cresce até `≥ 16`** | vs `sat = 1`, pior byte: `13` (`s=2`) · `36` (`4`) · `66` (`8`) · **`110`** (`16`) · `119` (`32`) |

⚠️ **Três destes são multiplicativos numa pista LINEAR** (`sharpness`, `pivot`, `rim strength`) ⇒ a
resolução do dedo está toda no primeiro oitavo: metade do efeito do `sharpness` vive nos primeiros
**`7 %`** do curso (**`3,35` passos de arrasto**) e o do `Rim Width` em **`3,0 %`** (`3` passos).
⭐ A casa **já tem a porta**: `ph2d_editor_core::…::link_slider_number_curved`, shipada em 16/09 pela
`line/sculpt3d` **pela mesma razão**. Expoentes que põem o meio-efeito a meio do slider:
`sharpness 3,84` · `rim width 5,06` · `pivot 2,18`.

### §10.9 — ⛔ O painel pode ENGOLIR a secção inteira

`publish_snapshot` apenda o estilo **no fim** e `paint` faz `.take(MAX_ROWS)` (`85`). Medido
(`param_rows = 2n + 31`):

| vértices do polígono | `param_rows` | + estilo | fileiras de ESTILO visíveis |
|---:|---:|---:|---|
| `22` | `75` | `85` | 10 de 10 |
| `23` | `77` | `87` | **8 de 10** |
| **`27`** (`MAX_POLYGON_VERTICES`) | `85` | `95` | **0 de 10 — a secção INTEIRA desaparece** |

⛔ O gate que defende o tecto (`every_row_of_the_biggest_polygon_fits_the_registered_family`) chama
`param_rows` **directamente** e **nunca vê** as dez fileiras que o `publish_snapshot` apenda.

### §10.10 — Outros achados do painel

- ⛔ **`Rim Width` tem o nome ao CONTRÁRIO.** A lei é `(1 − |N·V|)^width`: `w = 1` acende `50 %` da
  silhueta, `w = 3` acende `79 %`, `w = 64` acende `99 %` ⇒ **subir «Width» ESTREITA o contorno**. O
  comentário do i18n justifica o nome por *«o artista lê a LARGURA»*, e o valor **é** o expoente, sem
  remapeamento; o doc da própria lei diz o contrário. Ou o valor se inverte, ou o rótulo passa a
  `Rim Falloff`/`Rim Tightness`.
- ⛔ **`paint_fact` (as fileiras travadas) tem DOIS defeitos de aritmética**, invariantes na largura
  do dock (`220` → `720`): o rótulo é encostado à **direita** e acaba **exactamente** onde o valor
  começa ⇒ **vão `0,00 px` por construção**; e a coluna dela é `w − Lc` enquanto a da fileira viva é
  `Lc` ⇒ **`16 px` = `2 × Spacing::Md`** de desalinhamento. ⇒ o painel tem **TRÊS** alinhamentos de
  rótulo ao mesmo tempo (viva · amostra · travada), que é o «embolado» da foto.
- ⛔ **Atravessar a trava muda a PRECISÃO do número**: `paint_fact` formata com
  `decimals_for_step(1.0)` = **1 casa**, seja qual for a faixa ⇒ `0.375` vivo lê-se **`0.4`** travado
  (`Coat IOR 1.6` na foto é a prova).
- ⛔ **`Subsurface Radius Scale`** mede `142,79 px` a `TypeToken::Sm` contra uma coluna de
  `138,45 px` na largura real do dono (`~/.ph2d/layout.txt`, `dock_w_right = 296,89`) ⇒ **corta por
  `4,34 px`**. A cura desenhada para isto (`property_label_col_w_for`, empréstimo por secção) já
  existe desde o report de 14/09.
- ⭐ **O BALÃO já alcança este painel — medido, não lido:** um `Move` real sobre o slider do
  `Zone Pivot` põe `hot_id` correcto e o `store` aceita um `set_tooltip`; as chamadas a `set_tooltip`
  em `ph2d-panel-model3d` são **`0`**. *A ausência é de duas linhas, não de mecanismo.*
- ⭐ **A cura da §9 está a segurar:** as dez fileiras chegam ao pixel e ao dreno, e cada cor abre o
  SEU selector e escreve o SEU slot (gesto real, `10/10`).
- ⚠️ `ph2d-panel-model3d` é um dos **10 de 29** painéis **sem** o gate
  `every_word_this_panel_shows_comes_from_the_string_table` (o censo de literais lê `0`, mas não está
  gateado), e `tests/it/seam.rs` tem **zero** ocorrências de `Param::Style`.

## §11 — ⭐⭐⭐ A CURA: a curvatura ganha uma ESCALA, e a nitidez parte-se em duas

Ordem do dono a seguir à auditoria: *«siga como achar melhor mas coloque no estado da arte»*.

### §11.1 — O que mudou na LEI

| antes | depois |
|---|---|
| `sharpness` — **um** limiar partilhado | **`edge_sharpness`** e **`cavity_sharpness`** — um por lado |
| o `ε` da curvatura era o **óptimo de precisão**, para todos | **`softness`** — a escala ARTÍSTICA, só para o estilo |
| `clamp(k·s, ±1)` com `wc = max(c,0)`, `wv = max(−c,0)` | `wc = clamp(+k·gᵃ, 0, 1)`, `wv = clamp(−k·gᶜ, 0, 1)` |

⭐ **A justificação dos dois limiares é NOSSA e medida**, não uma cópia do alvo: numa peça real os
filetes leem `H·R ≈ 11`–`34` e as covas `≈ −3`–`−5` ⇒ *não existe um limiar partilhado que sirva os
dois* — o que acende as covas satura o filete `3×`–`9×`.

⛔ **O corte DURO fica**, e a recusa está medida (§10.3): um joelho (`smoothstep`) **piora**.

### §11.2 — ⭐⭐ A escala é da MEDIDA, e são DUAS assaduras porque são DUAS perguntas

| consumidor | `ε` | porquê |
|---|---|---|
| a subsuperfície MACIÇA | `eps_para(raio)` = `0,0064 · raio` | o **vale do erro** da segunda diferença, medido no mesmo sítio em três raios |
| a tinta por curvatura | `softness · raio` (fábrica `0,064`) | a **escolha do artista**: a que distância a peça é palpada |

⭐⭐⭐ **E a decisão inteira vive numa PORTA** ([`ph2d_field_render::curvatura::assar_canais`]) com
**dois consumidores** — o quadro do produto e o arnês dos gates. ⛔ *Escrita em linha em cada
chamador, ela divergiu no dia em que nasceu*: o produto assava dois canais, o arnês assava **um**, e
o gate da tinta de aresta acusou um botão VIVO de não chegar ao pixel. *Um arnês que monta o estado
à mão mede outro programa* — e foi o gate a apanhá-lo, que é o modo de falha bom.

⚠️ **O preço é zero no caminho de omissão:** cada canal só é assado se o consumidor **dele** estiver
vivo. Com um só — a omissão, e a cena do artista — não há segunda assadura nenhuma.

⭐⭐ **E a escala maior cura a DÍVIDA da §5-bis de graça:** a divergência CPU↔GPU é um ULP amplificado
por `1/(4ε²)` ⇒ **um `ε` `10×` maior divide a divergência por `100`**.

### §11.3 — A cura, MEDIDA no caminho do produto

`cena =35`, `320×240`, pelo `shade_render`, com o **CONTROLO** ao lado (a mesma imagem sem estilo):

| | degrau de byte p99 entre píxeis vizinhos | vs o controlo |
|---|---:|---:|
| **controlo** (sem estilo) | `30` | — |
| suavidade no **piso** (`0,0064`, a lei de ontem) | `107` | **`3,57×`** |
| suavidade de **fábrica** (`0,064`) | **`42`** | **`1,40×`** |

⇒ o penhasco cai **`2,5×`**, e as covas continuam côncavas (`p05` de `H·R`: `−3,358 → −2,256`).

⛔ **A cerca que impede o botão de matar a tinta** está no gate: acima de
[`ph2d_style::Curvature::MAX_SOFTNESS`] as covas deixam de ser côncavas e a `Cavity Tint` morre.

### §11.4 — ⭐⭐⭐ E o painel DIZ porque uma fileira está apagada

A resposta ao *«Zone pivot não sei para que serve mas parece morto»*: ele **é** um no-op no estado em
que o painel abre, e **não está sozinho**. A `Linha::apagada` declara a condição e a frase, e o
`ParamRow::inert` leva-as à tela — o mecanismo que existia desde 2026-09-18 e que esta secção citava
no cabeçalho sem cumprir.

| fileira | apagada quando | o gesto que a destranca |
|---|---|---|
| `Rim Color` · `Rim Width` | `rim.strength == 0` | subir a força do contorno |
| `Edge Sharpness` | a tinta de aresta é branca | dar cor à `Edge Tint` |
| `Cavity Sharpness` | a tinta de cova é branca | dar cor à `Cavity Tint` |
| `Curvature Softness` | `!reads_curvature()` | dar cor a uma das duas |
| **`Zone Pivot`** | `shadow == highlight` | dar cor a uma das tintas de zona |

⚠️ **O gate tem as DUAS metades**: de fábrica há pelo menos quatro apagadas **e** com os gestos
feitos não sobra nenhuma — senão «apagar» viraria licença para apagar tudo.

### §11.5 — Os tectos que a auditoria mediu, aplicados

`Edge`/`Cavity Sharpness` descem de **`8` para `2`** (`s = 1` entrega `99,0 %` do que `s = 8`
entrega ⇒ `87,5 %` do curso comprava `1 %` do efeito). Os outros três ficam **com a tabela ao lado**,
declarados como tectos de PRODUTO e já não como dívida por medir.

### §11.6 — A arrumação cresceu, e a RESERVA é declarada

`PACKED` vai de `20` para `24`: cinco cores e **sete** escalares são `22` floats, que o alinhamento
de `vec4` arredonda a `24`. As duas posições que sobram são [`ph2d_style::wgsl::RESERVADAS`],
**gateadas a zero e proibidas de serem reclamadas por uma fileira** — ⛔ *uma posição sem dono e sem
régua é onde o campo seguinte aterra por engano*.

### §11.7 — ⛔ A premissa de uma recusa MORREU, e a nota é reescrita com a morte à vista

O doc da [`ph2d_field_gpu::paint_wgsl::CURVATURA`] recusava por escrito passar o `ε` por argumento:
*«mudaria o texto do produto para servir o instrumento»*. **Era verdade enquanto o PRODUTO tivesse um
`ε` só.** Hoje ele tem dois, e o argumento serve o produto — o instrumento passa a ser o segundo
beneficiário em vez do único. *Quem move o número que tornava algo inalcançável tem de reconferir a
nota* (`CLAUDE.md` §0.0).

**Prova de mutação: 5 de 5 sangram**, com o controlo (reordenar duas fileiras equivalentes) a **não**
sangrar.

## ⛔ Recusas MEDIDAS

| o que foi recusado | porquê, com o número |
|---|---|
| a forma ingénua `a·(1−w) + b·w` ser «errada» | **refutada**: ela é exacta em `2 044 824` amostras. O perigo é `a + (b − a)` com `a ≠ b` |
| o contorno desenhado (a tinta na silhueta) nesta crate | ele lê os VIZINHOS no ecrã ⇒ é um passe, e a crate deixaria de ser a lei partilhada |
| afrouxar a barra da paridade para os `13` bytes | a divergência é da CURVATURA e é pré-existente; a tinta mede-se onde não amplifica |
| o estilo no MATCAP | ele é a luz do OLHO; tingi-lo faria o artista medir a peça através de uma mentira |
| um `shade_render_com_estilo` ao lado do outro | é a segunda porta pela qual o defeito do §24 volta |
| pendurar a amostra do estilo no `model3d_color_swatch(entity, slot)` | a não-colisão passaria a depender do acidente de `Entity::to_bits()` nunca valer `0` (§9) |
| um braço final que devolva um id «estável» a toda família nova | **é o defeito do §9 escrito outra vez**: `None` cai no controlo normal, que se lê como uma falta |
| um **joelho suave** (`smoothstep`) no lugar do `clamp` da curvatura | **PIORA** (`168` contra `162`): um joelho actua no domínio do VALOR e a dureza vive no do ESPAÇO (§10.3) |
| baixar o `Curvature Sharpness` como alavanca de dureza | `1,5×` de autoridade contra `6,5×` do `ε` (§10.4) |
| culpar o **anti-serrilhado** pela borda dura | p99 `169 → 161` ao excluir os píxeis de borda (§10.3) |
| culpar a **saturação do `clamp`** | `4,19×` o controlo já com a saturação em `1,9 %` (§10.3) |
| **borrar o canal de curvatura em espaço de ECRÃ** | ele lê os VIZINHOS ⇒ é um **passe**, e a crate deixaria de ser a lei que os dois motores partilham. Daria uma imagem na referência que o dispositivo não sabe reproduzir |
| filtrar o **SDF** (em vez da curvatura medida) para ganhar o raio | **move a superfície** (`dshift` até `0,84` voxel) e dá **as mesmas** larguras de rampa (§10.6) |
| ler a curvatura **de longe demais** (`ε/raio ≥ 0,2`) | o `p05` fica positivo: as crateras deixam de ser côncavas e a **`Cavity Tint` morre** (§10.4) |
| afinar a **lei** do estilo para curar a divergência CPU↔GPU | a tinta por curvatura mede **`0` ULP na lei**: `100 %` do que ela move é a GRANDEZA (§5-bis) |
