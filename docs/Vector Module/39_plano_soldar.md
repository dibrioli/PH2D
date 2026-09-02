# Plano 39 — **SOLDAR** (a rede que o balde precisa)

> Ideia do Enio (2026-08-31): *"e se pudéssemos soldar linhas cruzadas? Ou seja: linhas cruzadas
> compartilham o mesmo nó de modo que criem várias áreas fechadas interligadas?"*
> Decisão dele, no mesmo dia: **soldar CONSOME os traços originais.**

## §1 — A pesquisa

| onde | o modelo | o preço |
|---|---|---|
| **Figma** (*vector networks*, 2016) | o objecto é um **multigrafo não-dirigido com identidade de aresta**: um nó tem 3+ segmentos e sobrevive à edição | refazer o tipo *caminho*. Eles contam *"becos sem saída"* e que quase desistiram |
| **CAD** (Fusion, SolidWorks) | um esboço **já é** uma rede; as regiões fechadas (*profiles*) formam-se sozinhas | é por isso que o Trim de lá parece natural |
| **Illustrator** (*Pathfinder > Divide*) | corta tudo nos cruzamentos, uma vez | ⛔ **perde as partes de caminhos abertos que ficam de fora** — a queixa documentada |

⭐ **A nossa é a metade barata da Figma com o defeito do Illustrator curado.**

## §2 — A lei

> **Cada contorno parte-se em ARCOS nos pontos onde encontra os outros**, e as pontas dos arcos
> vizinhos caem exactamente no mesmo sítio — porque saem do mesmo cruzamento.

⭐ **O grafo não é uma estrutura de dados: é implícito nas coordenadas coincidentes.** Nenhum tipo
novo, nenhum contrato mexido.

| o contorno | os cruzamentos | o que sai |
|---|---|---|
| qualquer | nenhum | ele próprio, **intacto** (os mesmos vértices, não uma reconstrução) |
| aberto | `n` | `n + 1` arcos |
| fechado | `n` | `n` arcos abertos (o último dá a volta pela emenda) |
| fechado | `1` | **um** arco aberto — um anel cortado num ponto, não um degenerado |

⛔ **E não é automático.** Se cruzar duas linhas as colasse sozinho, seria impossível apenas
**sobrepor** dois traços. É um verbo sobre a selecção (o botão **Weld**, colado no *Join*: aquele
solda duas pontas, este solda os cruzamentos).

⭐⭐⭐ **E o nó SOBREVIVE ao dedo** — report do Enio, com foto: *"weld dividiu e não soldou (eu que
afastei os pontos)"*. Ele estava certo, e a falta eram **duas** coisas:

1. **Cortar não é soldar.** As duas metades de um cruzamento nascem de contornos DIFERENTES: cada um
   converte a mesma travessia para a SUA fracção e avalia a SUA cúbica ali — os pontos ficam perto e
   **não iguais**. *Dois pontos perto não são um nó, são dois nós.* ⇒ `weld::fuse_endpoints` funde
   cada aglomerado numa coordenada só (o **centroide**, para a solda não depender da ordem da
   selecção), e a alça acompanha a âncora.
   ⚠️ **A folga é DUAS vezes a flecha da amostragem**, não uma: cada lado erra a sua, em direcções
   opostas. Medido em dois círculos de raio 100 — as pontas a `0,1376` com flecha de `0,12`, e com
   uma flecha só a solda **não pegava**.
2. **Arrastar tem de levar todos.** `PenTool::welded_with` responde *"quem mais partilha esta
   ponta"* (comparação **exacta**, `WELD_TOL`), e o arrasto de âncora move a união *selecção ∪
   juntas*. É a lei do esboço de CAD: **duas pontas no mesmo sítio são UM nó**.

⛔ **O que a solda ainda NÃO promete:** *acrescentar* uma linha nova ao nó depois não a solda — a
junta é a coincidência, e quem a cria é o Weld, o Join ou o encaixe. E o modelo completo da Figma (a
aresta com identidade, que sobrevive a qualquer edição) continua fora.

## §2-bis — ⭐⭐⭐ **CRUZAR NÃO É A ÚNICA FORMA DE SE ENCONTRAR** (report de 2026-09-01)

> Enio: *"ainda não consegue conectar as duas curvas. 2 curvas geram outras duas linhas com
> aparência igual ou diferente. mas as linhas não compartilham o mesmo nó"*.

⚠️ **Medido antes de mexer, e a sonda percorreu o CAMINHO REAL DO FRAME** (entidades ECS,
`settle_origins`, `vec_transform::build`, `PenTool::set_xforms`): duas curvas **cruzadas** dão 4
arcos, com o nó partilhado bit a bit e `welded_with` a devolver 3 juntas em cada ponta do meio — *o
caminho do cruzamento estava certo de ponta a ponta*. ⛔ Uma varredura de 8 configurações
encontrou o buraco noutro sítio: **duas curvas ponta-com-ponta a `0,36` de distância faziam o
comando recusar-se** (*"nada se cruza"*).

⇒ A lei ganha a segunda metade:

> **Duas pontas que se encontram são um nó, tanto quanto duas curvas que se atravessam.**

- **A folga é o ÍMÃ DO ENCAIXE** (`vec_snap::vec_weld_tolerance` = `SNAP_PX` × zoom), com piso na
  flecha dobrada do cruzamento. ⚠️ *O app já decide, a cada traço desenhado, que duas coisas a menos
  de 10 px de tela são para ficar no mesmo sítio* — soldar reusa esse veredito em vez de inventar um
  segundo número. ⛔ **Não é o `WELD_TOL`** (`1e-6`, exacto): um mede **intenção**, o outro mede um
  **facto** já consumado.
- **Um caminho que só empresta a ponta NÃO se dissolve** — mantém id, estilo, pose e efeitos, e só
  a âncora se muda (descendo ao espaço local dele). *Dissolver um traço que ninguém cortou seria
  cobrar o preço do corte por uma ligação.*
- ⛔ **Quem tem EFEITOS fica de fora**: o que se vê nele é geometria cozida e os vértices autorados
  já não são as pontas que a medição encontrou.
- ⚠️ **As pontas vêm de DOIS substratos** (um arco recém-cortado num vector · um caminho da cena com
  pose e id) ⇒ quem as agrupa é **uma porta só**, `weld::cluster_endpoints`; cada chamador escreve o
  resultado no seu substrato. O `fuse_endpoints` da wave anterior **morreu** ao ficar sem consumidor.

### ⛔⛔ E a régua da flecha estava a medir uma curvatura que não existe

O gate da cerca reprovou imprimindo `folga 0,5493` sobre **duas retas**. A causa: `sampling_error`
media a distância do ponto do meio ao **ponto médio da corda**, o que conta o deslize **tangencial**
como se fosse desvio — e uma reta autorada com as alças em cima das âncoras é uma cúbica
**degenerada**, que percorre o segmento exacto com velocidade `3t² − 2t³`.

⇒ Hoje mede a distância ao **SEGMENTO** (`arc_cut::dist_to_segment`). ⚠️ **É uma régua PARTILHADA**:
ela responde *"este ponto está SOBRE a curva?"* ao Trim (`touches`) e *"estas duas pontas são o
mesmo nó?"* ao Soldar — as duas eram generosas **em proporção ao tamanho do traço**. ⛔ Num círculo
de raio 100 ela **não muda** (`0,119`), que é o número com que o gate dos dois círculos foi
calibrado.

### ⭐⭐⭐ E o nó agora VÊ-SE

⚠️ **A metade que faltava do report não era código, era leitura**: uma coordenada partilhada não se
desenha — duas pontas coincidentes e duas pontas a um pixel pintam **o mesmo quadradinho**, e o
único instrumento era arrastar, que é um teste destrutivo para responder a uma pergunta de leitura.

⇒ `ph2d_vec_render::draw_weld_marks`: um **anel verde** em cada nó partilhado.

- **FIGURA, não cor** — o vocabulário que a crate já usa nas guias de encaixe (*afirmações
  diferentes recebem figuras diferentes*). As âncoras são quadrados; o anel é a única forma que não
  colide com nenhuma, e é a marca de coincidência do desenho de CAD.
- ⚠️ **A LEI é a do arrasto** (`PenTool::welded_nodes`, mesmo predicado e mesma `WELD_TOL` do
  `welded_with`), com gate nos **dois** sentidos: *"todo nó marcado tem juntas"* passaria com uma
  lista vazia. ⛔ Uma segunda régua acenderia o anel onde o dedo não arrasta junto, e o instrumento
  que existe para ensinar a lei ensinaria a errada.
- ⚠️ **A projecção é a MESMA do overlay** (`overlay_transform`), senão o anel pousa ao lado da
  âncora que ele afirma abraçar — e justamente sob auto layout, onde conferir a olho é mais difícil.
- ⚠️ **Ordenado por `x` antes de agrupar**: o passe corre por quadro, e `n²` sobre as pontas de um
  documento grande não é aceitável.

### ⛔⛔ E o 1.º smoke era INEXECUTÁVEL — o motor estava certo e o GESTO que eu pedi não existia

> Enio, 2026-09-01: *"o smoke não tinha nada do que vc falou e não funcionou ainda o Weld"*.

A cena armava a **seta branca** (`DrawMode::Node`) e mandava `Shift`+clicar para somar a 2.ª curva.
⚠️ **No modo Node esse gesto é tentado PRIMEIRO como *"alterna este PONTO na multi-selecção de
pontos"*** (`input_dispatch.rs`, raio de 10 px, decisão do Enio de 2026-07-15) — e num par de curvas
que se encontram **pelas pontas**, o sítio natural de clicar *é* um ponto. ⇒ o segundo traço nunca
entrava na selecção, o Weld via **um** caminho, não achava cruzamento e não fazia nada.

⇒ A cena arma a **seta PRETA** (`Select`), onde `Shift`+clique **soma uma forma**, e o 1.º par
**nasce seleccionado** — sem selecção a seção Path não é pintada, logo o botão que a cena manda
carregar não estaria sequer na tela. Ela também **activa a ferramenta de vetor** (`set_mode` escolhe
o modo *dentro* dela e **não** a activa).

⚠️ **E o motor foi ILIBADO pelo instrumento que faltava:** `seam_weld.rs` faz o gesto REAL sobre o
botão (Down+Up no rectângulo que o painel pintou) e prova que ele vira `Click` e chega ao
barramento — *nenhum gate desta wave atravessava o painel, e um verbo cujo botão não fala com
ninguém lê-se exactamente como um motor partido*. É a lição que a fileira de chips da booleana já
tinha custado (`seam_bool.rs`).

**Cena de smoke: `PH2D_BUILD_SMOKE=81`** — três pares (pontas que se encontram · cruzadas · longe uma
da outra), com gate a provar que a cena faz o que a mensagem dela promete.

## §3 — O que reusou

Tudo o que o **Trim** (plano 38) pagou: `trim_tool::crossings_against` (cruzamentos + **toques**),
`trim_tool::piece_geometry` (o arco entre duas fracções), `arc_cut::Geom`. O módulo novo
([`weld.rs`](../../crates/ph2d-vec-scene/src/weld.rs)) tem **um** algoritmo: as fronteiras ordenadas
e um arco entre cada par consecutivo.

⚠️ **A pose entra na conta** (`vec_weld.rs`): dois traços só se cruzam depois de o `Transform` os
pôr no lugar, e cada operando é assado no MUNDO — a mesma convenção do `apply_vec_boolean`.

## §4 — ⛔⛔ O DEFEITO DO TRIM que este plano encontrou

O gate de soldar reprovou com *"dois cortes num anel dão DOIS arcos: left 1, right 2"* — e a causa
estava **no Trim**, três horas mais velha:

- o corte vai por `strands_of`, que **normaliza** uma faixa que atravessa a costura de um anel;
- o realce ia por `piece_geometry`, que **não** normalizava ⇒ um pedaço que passa pela emenda
  **não acendia** e ainda assim era comido.

⚠️ **É exactamente a divergência «acende uma coisa e apaga outra» que o desenho do Trim proíbe**, e
o gate de «a mesma porta» **não a apanhou**: as duas portas só discordam sobre a emenda, e nenhuma
fixtura tinha um pedaço que passasse por lá. *Uma porta única prova que a resposta é a mesma; ela
não escolhe as perguntas que se fazem.* Dois gates novos no `trim_tool`.

## §5 — ⏳ Aberto

- ⏳ **O balde** — é o consumidor desta rede, e a razão de ela existir.
- ⏳ **Uma linha DESENHADA depois não entra no nó** — a junta é a coincidência, e desenhar não a
  cria (o encaixe cria, o Join cria, o Weld cria). Soldar outra vez resolve.
- ⏳ **Um composto soldado perde o buraco**: depois da solda os contornos são arcos, e um arco não
  tem dentro. É o preço declarado de *"consome os originais"*.
- ⏳ **A régua do anel não conhece o `layout_pose`** — `welded_with`/`welded_nodes` agrupam pelo
  `xform` do pen, e o overlay projecta com `layout_pose ∘ xform`. Sob auto layout as duas divergem;
  é pré-existente (o arrasto já divergia) e não foi tocado aqui.
- ⏳ **Custo não medido** — `O(contornos²)` em arestas de amostragem, sobre a selecção. É um verbo
  de clique, não um laço de quadro; sem número, a acusação seria palpite.

## §6 — ⭐⭐⭐ **A REDE É UM OBJECTO** (report de 2026-09-02)

> *"O weld cria uma grande quantidade de path na hierarquia quando na verdade deveria criar apenas
> 1. No canvas também deve ser transformado como se fosse um único objeto e o seu gizmo deve ser
> apenas 1."* — Enio

### §6.1 — O mecanismo, medido

Cada arco era um `VecPath`, e **um `VecPath` é uma entidade ECS** (ADR-0110). A cadeia é
determinística e foi seguida ponta a ponta:

| o que o emit fazia | o que isso produz |
|---|---|
| `insert_path` por arco | 1 entidade por arco (`vec_entities::sync`, *"um path ⟺ uma entidade"*) |
| 1 entidade por arco | 1 linha na Hierarquia por arco, baptizada `Path N` pela fábrica |
| 1 entidade por arco | 1 `Transform` por arco ⇒ **a rede pode ser rasgada** por um arrasto |
| N entidades seleccionadas | **N gizmos + 1 gizmo de união** (`snapshots.rs`, `if painted_views > 1`) |

Medido nas fixturas: duas linhas cruzadas davam **4** caminhos; um X num quadrado, **5**; um
asterisco de três linhas, **6**.

⇒ *soldar prometia uma rede e entregava um monte de pedaços que só por acaso estavam encostados.*

### §6.2 — A cura: um caminho COMPOSTO, escrito NO LUGAR

A rede passa a ser **um** `VecPath` cujos contornos são os arcos (`verts`/`closed` = o primeiro,
`subpaths` = os outros). ⭐ **O substrato já estava pago e já shipa:** `trim_tool::sever` emite
contornos **ABERTOS** desde o plano 38, `build.rs`/`path_tess` desenham-nos (gate
`an_open_contour_never_punches_a_hole_in_the_fill`), o índice **plano** de vértice (`compound.rs`)
já é o que o hit-test, o overlay e o arrasto falam, e `path_bbox`/`for_each_vert_mut` já percorrem
contornos.

⚠️⚠️ **Escreve-se no lugar do participante mais ao FUNDO** (`path_mut`), como o Trim: *o id, a fatia
de z e a entidade ECS têm de sobreviver à operação* — um `insert_path` daria um objecto novo, com
nome de fábrica, e o artista perderia o que baptizou. Os arcos descem ao espaço **local** do
anfitrião pelo inverso da pose dele (regra-mãe: a rede fala MUNDO, o documento guarda LOCAL).

⭐⭐ **E é isso que dá o gizmo único de graça.** O gizmo sai da selecção **ECS**
(`hero.gizmo.selection` + `extra_selection`), e `gizmo_prune::prune_dead` tira dela quem morreu. Ao
consumir *todos* os participantes, a solda matava o primário e os extras e o artista ficava **sem
gizmo nenhum**; com o anfitrião vivo, sobra exactamente **uma** entidade seleccionada ⇒ **um**
gizmo. ⚠️ E vale nos dois sentidos de escolha: se o artista clicou primeiro no traço de cima, esse
é o primário, morre, e a poda **promove** o anfitrião — que é o extra que restou.

⛔ **O preço, declarado:** a rede tem **um** estilo e **uma** pose — os arcos que vieram de outros
traços perdem cor, largura e pose próprias. É o que *"um objecto"* significa, e é a mesma conta que
o `make_compound` já cobra. A pilha de efeitos do anfitrião **sai**: os arcos já saíram da geometria
cozida por ela, e mantê-la aplicá-la-ia duas vezes.

### §6.3 — ⛔⛔ O que a mudança PARTIU, e teve de ser curado junto

- **A marca e o arrasto do nó ficavam CEGOS.** `PenTool::is_endpoint` era
  `!closed && subpaths.is_empty() && (vert == 0 || vert == len-1)`: a rede recém-criada **não tinha
  ponta nenhuma**, o anel verde apagava-se e o primeiro dedo rasgava-a. ⚠️ **E o gate
  `the_mark_and_the_drag_answer_the_same_question` continuava VERDE** — os dois lados cegam-se
  juntos, e um gate que cose duas respostas não vê as duas irem a zero ao mesmo tempo. Cura:
  `endpoints_flat`, uma ponta por extremo de cada contorno ABERTO, em índice plano.
- **A porta de entrada recusava a própria saída.** `editavel_no_sitio` recusava compostos, então
  pendurar uma linha nova na ponta de uma rede existente não pegava — o fluxo normal a partir da
  segunda solda. Cura: a recusa passa a ser só a dos **efeitos** (geometria cozida ≠ autorada), e a
  enumeração de pontas é a `pontas_planas`, irmã da de cima.

### §6.4 — ⛔⛔⛔ E um defeito ANTERIOR que a mudança tornou alcançável: **JUNTAR EVAPORAVA CONTORNOS**

`VecScene::merge_path_into` e `PenTool::join_open_path` costuram o contorno **primário** da origem
no destino e depois **apagam a origem**. Uma origem composta perdia todos os `subpaths` **sem erro
nenhum** — o pior modo de falha que há.

⚠️ **O defeito é anterior a este plano** (valia para o resultado de uma booleana, para uma
rosquinha) e nenhum gate o via, porque **a régua era a contagem de CAMINHOS**: ela cai de 2 para 1
nos dois mundos. A régua que o vê é a contagem de **CONTORNOS**. ⚠️ E eram **duas portas para a
mesma pergunta, divergindo para o mesmo lado** — as duas foram curadas, cada uma com o seu gate.

### §6.5 — ⏳ Aberto (nomeado, com o mecanismo)

- ⏳ **A lâmina (Cut) recusa a rede.** `ph2d_vec_boolean::cut_open` recusa `is_compound()` — uma
  recusa **pré-existente e declarada** (vale para toda booleana e toda rosquinha), em que a saída da
  solda passou a cair. O verbo irmão que faz o trabalho **existe e é contour-indexed**: o **Trim**
  (`trim_tool::sever`). Ensinar o `cut_open` o índice de contorno é wave própria.
- ⏳ **A caneta só continua a partir do ARCO 0.** O cabeçote de desenho é `verts.last()` e
  `reopen_endpoint` procura em `verts` — reabrir a rede por outro arco pede um cabeçote que
  enderece **contorno**, que toca `pen_drag`/`lib.rs` em vários sítios. Benigno hoje (reverter o
  primário não perde nada e renderiza igual), mas é uma capacidade que cada arco solto tinha.
- ⏳ **Marcadores e alças de largura vivem no arco 0** (`marker::end_tangent`,
  `width_handles`): setas e perfil de largura aplicam-se a um arco da rede, não à rede.
- ⏳ **Cada área do balde continua a ser o seu próprio item na Hierarquia** — e o gizmo dela é
  **inerte** (o re-cozimento divide a pose fora). Ou o gizmo sai, ou as áreas passam a filhas da
  rede; é decisão de produto, e a linha da Hierarquia é hoje o **único** manípulo que o artista tem
  sobre a cor de uma área.
