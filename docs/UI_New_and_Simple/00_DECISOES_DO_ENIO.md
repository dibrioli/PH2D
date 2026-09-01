# As decisões do Enio — FECHADAS (2026-08-30)

> ⛔ **Não re-litigar.** Estas três foram postas ao Enio com as alternativas, os preços e o
> mecanismo de cada uma, e ele decidiu. A spec descende daqui.
>
> Quem quiser reabrir uma delas precisa de um **facto novo medido**, não de uma preferência.

---

## D1 — Painéis: **ancorados, com flutuação DECLARADA**

> *"Cada painel diz se PODE flutuar."*

O modelo do Godot (`editor_dock.h:91`):

```cpp
BitField<DockLayout> available_layouts = DOCK_LAYOUT_VERTICAL | DOCK_LAYOUT_FLOATING;
```

**O que isto fixa:**
- Painéis de propriedade declaram que **não flutuam** ⇒ nunca chegam perto de uma viewport nem
  de uma régua. É a cura da foto 2, e é um `Constraint` (o gesto errado torna-se inexprimível),
  não uma verificação.
- O que faz sentido solto (paleta de cor, selector, popover) continua solto — **por declaração**.
- A posição passa a ser **enumerada**, não contínua.

**O que já temos e encaixa:**
- `PanelLayout::Sidebar` existe, funciona e tem teste (`theme.rs:54`).
- ⛔ **Mas está preso ao TEMA.** A primeira obra desta decisão é **separar layout de paleta** —
  hoje só se têm painéis ancorados aceitando o azul claro do `blueprint`.

⚠️ **Consequência obrigatória, no MESMO trabalho:** a **fuga do gizmo de navegação**
(`panel_ops::panel_rects`, o gizmo que se desloca para escapar à moldura — `CLAUDE.md` §5,
3D Modeling) **tem de ser removida**. Ela é o remédio do sintoma; com os painéis fora da vista
ela passaria a fugir de uma moldura que já não a alcança — remédio duplo.

⚠️ **E ancorar NÃO reduz por si só os 51 % de canvas coberto**
([`medicoes/02`](medicoes/02_a_area_tapada.md)): um dock ocupa o mesmo que um flutuante. O que
reduz é colapsar, empilhar por abas, ou ter menos conteúdo lá dentro — que é a **D2**.

---

## D2 — Comandos: **os dois** — barra global **e** cabeçalho por área

> Barra global para o que é do aplicativo inteiro (Arquivo, Editar, Ajuda);
> cabeçalho por área para o que é da ferramenta.

É o que o Godot faz na prática, e é o desenho mais completo dos três oferecidos.

**O que isto fixa:**
- Existe **um** sítio canónico para cada comando ⇒ o painel deixa de ser o depósito por omissão.
  É a cura da foto 3.
- O corte é **por âmbito**: se o comando vale em todo o app, vai à barra; se vale só naquele
  editor, vai ao cabeçalho dele.

**A tabela de destino, aplicada ao painel medido (`3D Model`, 74 entradas):**

| hoje no painel | nº | vai para |
|---|---:|---|
| `export.*` (Export Draft/Fine/Max) | 3 | **barra global → Arquivo** |
| `view.*` + `camera.*` + `frame.*` | 11 | **cabeçalho da área do canvas 3D** |
| `add.*` (paleta de formas) | 20 | **cabeçalho da área → menu Adicionar** |
| `kind/act/verb/op/mode` | 17 | **cabeçalho da área → pulldowns** |
| `mod.*` (modificadores) | 8 | ✅ **fica no painel** — é propriedade do objecto |
| leituras/estado/título | 15 | fica |

⇒ **8 de 74 entradas pertencem a um painel de propriedades.** As outras 66 têm outro dono.

**O que já temos e encaixa:** **148 itens `CTX_MENU_*`** e **40 handlers de chrome**, incluindo
um ficheiro cujo doc-comment é *"os itens do menu Ficheiro"* (`chrome/io_menu.rs`).
⭐ **O trabalho é de realojamento, não de construção.**

⚠️ **O preço que o Enio aceitou:** a barra global come uma faixa de altura permanente. No alvo
iPad (1024 pontos de altura) isso é caro, e soma-se aos 51 % de largura já medidos.

### ⛔⛔ CORRECÇÃO (2026-08-31) — o preço vale para UMA faixa, e a segunda foi RECUSADA

> Enio: *«Lembre-se que esse app tem tablets e iPad como alvo. Não podemos ir perdendo espaço.
> Desfaça isso.»*

O **cabeçalho por área** foi construído (entrega 30) e **revertido no mesmo dia**. Ele custava
**28 px** de altura permanente — `−1,5` ponto percentual de área de desenho no alvo declarado —
para dar casa a dois interruptores.

⚠️ **A decisão sobre o ÂMBITO fica de pé**: um comando do editor não pertence a um menu do app. O
que foi recusado é a **faixa própria**. ⇒ se a metade 2 voltar, ela tem de caber **onde já se paga
altura** (a fila de ferramentas, ou um popover à direita dela), nunca numa banda nova.

### ✅ E ela VOLTOU assim (2026-08-31, entrega 33) — um **pulldown** no fim da fila

O módulo que tem o canvas publica os comandos dele, e a fila mostra-os **num chip só** cuja face é a
**leitura** do estado (`Front`, `User`, …). Custo de altura: **zero** — a faixa já existia e continua
a ser uma linha.

⛔⛔ **UM chip, e o número é MEDIDO, não estimado:** com os nove comandos crus na fila ela precisa de
**2 linhas até no iPad 12,9"**, o maior dos três alvos, e ainda transborda `2` chips para o `⋯`
(mutação 6 do gate `the_area_costs_one_chip_and_the_bar_is_still_one_line`).
*Poupar altura gastando largura não poupa nada.*

**Primeiro inquilino — o painel `3D Model` perde as 9 primeiras das 74 entradas:** as seis vistas
nomeadas e os três gestos de câmera. Elas nunca foram propriedades do objecto — são sobre *olhar*.
⚠️ **Os ids são os MESMOS**, então o clique continua a chegar ao mesmo braço do painel: *um comando
com dois ids tem dois sítios a apodrecer em separado.*

⭐ E o preço deixou de ser uma nota: ele é um **gate** com a área medida nos três tablets —
[`medicoes/06`](medicoes/06_o_orcamento_de_ecra_em_tablet.md).

### ✅ E as DUAS metades fecharam para o `3D Model` (2026-09-01, entregas 35 + 36)

**O que sobrou do painel mudou-se, e para TRÊS sítios, porque o corte da D2 é por âmbito:**

| fileira | nº | destino | porquê |
|---|---:|---|---|
| verbos do gizmo + referencial | 5 | os chips **`MOVE`/`ROT`/`SCALE`/`SPACE` que já existiam** | é sobre **mover com a mão** |
| níveis de exportação | 3 | **menu global → File** | escrever um arquivo vale em **todo o app** |

⇒ o painel perdeu **17 das 74** entradas, e o que lá fica são os **números do objecto escolhido** e
as leituras — que é o que um painel de propriedades é.

### ⛔⛔⛔ CORRECÇÃO (2026-09-01) — o gizmo NÃO precisava de controlo novo: os dele estavam MORTOS

> Enio, com foto: *«esses botões de mover, rot e scale já existiam. só não estavam ligados a cada
> modo.»*

A 1.ª entrega desta metade construiu um **2.º pulldown de área** (*Gizmo*) — e os chips
`MOVE`/`ROT`/`SCALE`/`PIVOT` e `SPACE` estavam no trilho desde sempre, pintados, clicáveis e a
acender-se. **Medido:** fora o próprio pintor, `TOOL_TRANSLATE`/`ROTATE`/`SCALE` e o
`tool_space_local` do `SPACE` **não tinham um único leitor na árvore** — eles eram a 2.ª espécie de
controlo morto do `CLAUDE.md` §5.0 (*o clique chega, a luz acende, e o valor não alcança
consumidor*).

⭐⭐⭐ **A regra que fica:** *antes de dar casa nova a um comando, procure o controlo que já a tem.*
Um controlo **morto** e um controlo **ausente** produzem o mesmo report — e as curas são opostas:
para o ausente constrói-se, para o morto **liga-se**. Construir por cima de um morto deixa o app com
**dois** sítios para o mesmo verbo, e o que apodrece é o que ninguém relê.

⇒ o pulldown *Gizmo* foi **apagado** no mesmo dia, e com ele as famílias de id
`model3d_mode_button` / `model3d_frame_button`: *ligar os que existem e apagar os duplicados é UMA
obra, não duas.* O gate `the_area_offers_no_second_door_for_the_gizmo` impede a reconstrução.

⛔ **O `PIVOT` fica de fora, e é uma ausência declarada:** o gizmo deste módulo tem três verbos e
nenhum é *mover o pivô*. Ele continua a ser o que era.

⭐ **E o orçamento de chips de área é MEDIDO: `3`** (sonda de 2026-09-01 sobre `bar_split` +
`horizontal_lines`, com o módulo armado):

| alvo | largura da área | 1 chip | 2 | 3 | 4 |
|---|---:|---|---|---|---|
| iPad 12,9" | `754,0` | 1 linha | 1 | 1 | 1 |
| iPad 11 | `582,0` | 1 linha | 1 | 1 | **2** |
| iPad mini | `521,0` | 1 linha | 1 | 1 | **2** |

Usa-se **1** (a vista). ⛔ Ultrapassar não parte nada (o `⋯` absorve) — custa a 2.ª linha nos dois
alvos pequenos, que é exactamente o que a entrega 32 existe para não pagar.

⚠️ **As linhas do *File* são CONTRIBUÍDAS, não fixas:** o módulo publica-as enquanto tem o canvas, e
com ele fechado o menu volta byte a byte ao que era. *Uma linha `Export Draft` permanente seria um
alvo que consome o clique e não faz nada.* ⚠️ **E a mesma bandeira decide quem é dono dos chips do
trilho** — sem ela um módulo 3D fechado roubaria o `MOVE` ao editor 2D.

⚠️ **A célula `add.*` (20) desta tabela já estava FECHADA e a tabela não sabia:** a paleta de formas
(W100) reduziu-a a **um** chip que abre o catálogo genérico da casa — com busca, categorias e
rolagem. ⛔ Quem for pegar nela pelo número `20` reconstrói trabalho já pago.

⏳ **O que sobra do `kind/act/verb/op`:** o **verbo** e o **carácter** da mistura são **propriedades
do objecto** pelo critério desta própria decisão (o mesmo que manda o `mod.*` ficar) — a tabela
acima mandava-os para um pulldown e **contradizia-se**. Ficam no painel. Sobram as **operações
booleanas** e as **acções** (duplicar/apagar/isolar), que são gestos sobre a selecção e ainda não
têm destino decidido.

---

## D3 — **LAYOUTS por tarefa** · e MODOS são per-objecto (⚠️ **CORRIGIDA**)

> **Correcção do Enio, 2026-08-30:** *"Aí como no Blender há duas coisas: Layout e Mode. Alguns
> objetos têm modo de edição próprio como vector, cujas tools são completamente específicas e onde
> toda a tela vai mudar. Já Editor 2D, Editor de texto, Runtime, são layouts."*

⛔ **A minha pergunta original tratava Modo e Layout como a mesma coisa, e estava errada.** Os
manuais confirmam o Enio, e há um **terceiro** eixo que os dois motores separam e nós não. O
estudo completo, com as citações, está em
[`pesquisa/04_modo_layout_e_ferramenta.md`](pesquisa/04_modo_layout_e_ferramenta.md).

### Os três eixos

| eixo | quem decide | onde vive | o que muda |
|---|---|---|---|
| **Layout** | o **utilizador** | barra de cima (abas) | que **áreas** existem e que editor está em cada |
| **Modo** | ⭐ o **TIPO DO OBJECTO** seleccionado | cabeçalho da **área** | ferramentas, atalhos, aspecto da vista, e **que outros editores funcionam** |
| **Ferramenta** | o utilizador, dentro do modo | **toolbar** da área | o **gesto** do ponteiro |

**O que a decisão fixa:**
- **A escolha «por tarefa» vale, e é sobre LAYOUTS** — poucos e largos, escolhidos pelo
  utilizador, abas na barra de cima. *Editor 2D · Editor de Texto · Runtime · …* (exemplos dele).
- ⛔ **Modos NÃO são uma lista global e não se escolhem.** Cada **tipo de objecto** declara os
  seus. Blender: *"Which modes are available depends on the object's type."* Só o modo **Object**
  é universal.
- ⭐⭐ **A costura entre os dois é UM CAMPO OPCIONAL** — o Workspace do Blender tem
  `Mode: "switch to this Mode when activating the workspace"`. Ortogonais, com um atalho
  declarado; **não acoplados**.

⚠️ **E a mesma confusão está no nosso código, medida:** o `DrawMode` do vetor tem 14 variantes que
são, lidas pelos nossos próprios doc-comments, **2 modos (`Select`=Object, `Node`=Edit) + 12
ferramentas** — todas as doze justificam-se por *"o gesto é outro"*, que é a definição de
ferramenta, não de modo. ⇒ hoje **não se consegue exprimir «Edit + ferramenta Fillet»**.

⚠️ Os **29 pills** ordenam-se assim: ~19 **ferramentas** · 2 **layout** (mostrar Hierarquia /
Inspetor → menu *Ver*) · 3 ⛔ **uma preferência** (tamanho do botão → Preferências).

⚠️ E os **9 toggles de módulo** são a coisa mais próxima de Layouts que temos — mas são
interruptores **independentes** (2⁹ = 512 combinações), não um selector *um-de-N*.

⏳ **Fica por decidir:** a lista de Layouts · que modos declara cada tipo de objecto nosso · se
adoptamos o campo `Mode` do Workspace · e **como partir o `DrawMode` nos dois eixos sem partir o
que funciona** (14 variantes vivas, com gates — não desenhado).

---

## D4 — Áreas: **encaixes FIXOS**, como o Godot

> *"Lugares pré-definidos. O artista escolhe QUAL painel vai em cada lugar, e arrasta a
> divisória — mas não inventa lugares novos."*

**A alternativa recusada:** a divisão livre do Blender (qualquer área corta-se em duas, sem
limite). ⛔ **Recusada com motivo, não por preguiça:** é muito mais código, o artista consegue
produzir uma tela que não sabe desfazer, e no iPad arrastar divisórias finas com o dedo é mau.

**O que isto fixa:**
- A posição de um painel é **um valor de um conjunto finito**. É a forma mais forte do
  `Constraint`: o erro não é detectado, é **inexprimível**.
- Vários painéis no mesmo encaixe viram **abas** — que é como um encaixe absorve crescimento sem
  crescer.
- ⭐ E torna o layout **serializável de forma trivial**: um layout é `{encaixe → [painéis], posição
  das divisórias}`. O Godot chama à chave `layout_key` (`editor_dock.h:77`).

⚠️ **Isto resolve a tensão que o §6 do diagnóstico nomeava** («extrema simplicidade» contra
«altíssima capacidade de ajustes»): o ajuste que fica é **o que**, não **onde**.

---

## D5 — A régua entra na **ÁREA DE DESENHO**

> *"A régua deixa de ser da janela e passa a ser da área do canvas — começa depois do trilho, não
> por baixo dele."*

É o modelo do Blender: a régua é uma **region** do editor, não do ecrã.

**O que isto cura, e é medido** ([`medicoes/02`](medicoes/02_a_area_tapada.md)): hoje
`left_band = (canvas.x, canvas.y, 20, canvas.h)` com `canvas = (0,0,w,h)`, e o rail também
começa em `x = 0` ⇒ **86,8 % da régua da esquerda por baixo do rail**. Com a régua dentro da
área, ela começa **depois** do rail e a sobreposição é **estruturalmente zero**.

**As alternativas recusadas:**
- *Empurrar o rail 20 px* — ⛔ custa mais 20 px de largura numa tela onde 51 % já é moldura, e
  **não cura a régua de cima**, que continua sob a barra superior.
- *Régua ligável/desligável* — ⛔ quem desenha com medida deixa-a ligada sempre, e para essa
  pessoa nada muda.

⭐⭐ **E D5 generaliza para o RAIL:** se a régua é uma região da área, o trilho de ferramentas
também é (Blender: *Toolbar*, região esquerda do editor; Godot: barra da própria viewport). ⇒ os
dois passam a ser **irmãos numa fila**, não **camadas empilhadas** — e irmãos não se tapam. *A
ordem de pintura deixa de ser a resposta, porque deixa de haver sobreposição.*

---

## D6 — A tabela de MODOS por tipo de objecto

> Proposta pela linha, corrigida pelo Enio em 2026-08-30 (duas correcções, as duas aplicadas).

| o objecto é | modos que ele declara |
|---|---|
| **qualquer coisa** | **Object** (mover / rodar / escalar) |
| forma vetorial | Object · **Edit** (nós e alças) |
| malha 3D | Object · Edit · **Sculpt** · **Paint** |
| peça sólida (Model / SDF) | Object · **Edit** |
| imagem / sprite | Object · **Paint** · **Mask** |
| **desenho Flip** | Object · **Draw** · Edit |
| corpo de física | Object |

### As duas correcções do Enio

1. ⭐ **«O Flip merece modo próprio.»** ⇒ `Draw` é do Flip e **não** é o `Paint` da imagem. É a
   mesma escolha que o Blender faz — o Grease Pencil tem `Draw Mode` **próprio**, separado dos
   modos de pintura da malha, e pela mesma razão: *o que se edita é um TRAÇO, não um pixel.*

2. ⏳ **«Uma forma vetorial deveria ter um modo Pintar também, com o módulo painter atuando sobre
   o vector. Contudo esse feature ainda não existe.»**
   ⛔ **Por isso ele NÃO entra na tabela acima** — um modo declarado que não pinta é um **controlo
   morto**, e o `CLAUDE.md` §5.0 tem duas espécies dele catalogadas com o custo medido. O Blender
   é igual por construção: um modo indisponível **não aparece**, não fica cinzento.
   ⇒ Fica no mapa da estrada, com endereço: [`pesquisa/05_pintar_sobre_vetor.md`](pesquisa/05_pintar_sobre_vetor.md).

   ⭐⭐ **E a medição achou que METADE já existe:** `Paint::Pattern` com
   `PatternSource::Image(AssetId)` e `PatternMode::Clamp` (*"uma cópia só"*) já mostra **uma imagem
   mapeada dentro de uma forma**, hoje; e o `PaintedDoc` — a ponte do Painter — **não exige um
   `Sprite`** no alvo.
   ⛔ **O que falta e decide o preço é o mapa ser da FORMA e não do MUNDO:** o `PatternFill`
   posiciona-se em world-space, logo **editar a forma não leva a tinta com ela**. É o UV do
   Blender, e é o item caro.
   ⛔ **E NÃO é o mesmo trabalho do *Flip sai do Flip*** — hipótese minha, **refutada**: aquele é um
   assado de **sentido único** (para exportar); este tem de ser **durável e reversível**, senão
   assar o vetor mata o vetor. *Substrato partilhado não implica trabalho partilhado.*

---

## D7 — A lista de LAYOUTS: **oito**

> Proposta da linha com seis, corrigida pelo Enio em 2026-08-30: *"Desenho 2D e Vetor são dois"* e
> *"o Flip merece layout próprio"*. ⇒ **8**.

| Layout | a tela vem arrumada para |
|---|---|
| **Desenho 2D** | pintar — canvas grande, camadas, pincéis |
| **Vetor** | ⭐ desenho vetorial — **separado do raster, por decisão dele** |
| **Flip** | ⭐ animação quadro-a-quadro — **layout próprio, além do modo `Draw`** |
| **Modelagem 3D** | modelar e esculpir — vista 3D, hierarquia, propriedades |
| **Animação** | ⚠️ ver a nota abaixo |
| **Nós** | o grafo no centro, com pré-visualização |
| **Código** | o editor de texto, com saída e erros |
| **Runtime** | correr o jogo, sem chrome de edição |

⚠️ **A D8 muda o que «Animação» significa.** Se as timelines funcionam em **todos** os layouts,
então este não é o layout onde se pode animar — é o layout onde a **ênfase** é o tempo (timeline
grande, canvas pequeno). ⛔ *É uma distinção de proporção, não de capacidade,* e tem de ser escrita
assim ou alguém a implementa como um modo exclusivo.

⚠️ **Oito abas cabem na barra** — o Blender ship-a 10 por omissão e mais 6 opcionais.

---

## D8 — **As timelines funcionam em TODOS os modos de criação, 2D e 3D**

> **Enio, 2026-08-30:** *"As timelines existentes devem funcionar com todos os modos de criação 2d
> e 3d."*

⭐ **No modelo de áreas isto é uma linha:** a Timeline é uma **área** que pode ocupar o encaixe
`BOTTOM` em **qualquer** Layout, e liga-se ao que está **seleccionado**. *A área é do Layout; o
conteúdo é da selecção.* ⛔ Uma timeline que só existisse no Layout *Animação* repetiria o erro dos
9 toggles de módulo — um sistema alcançável a partir de um sítio só.

⚠️⚠️ **Mas a medição diz que o pedido são TRÊS trabalhos, com preços muito diferentes**
([`medicoes/04_o_alcance_das_timelines.md`](medicoes/04_o_alcance_das_timelines.md)):

| # | alvo | estado |
|---|---|---|
| 1 | **2D** (sprite · vetor · Flip · física) | ✅ **funciona hoje** — 13 propriedades vivas |
| 2 | **3D Modeling** (SDF) | ⏳ o objecto **está** na hierarquia, com pose própria; falta a Timeline aprender o **segundo vocabulário** |
| 3 | **3D / Sculpt** | ⛔⛔ **não é uma entidade** — vive num campo do app. Nada na cena a alcança |

⭐⭐⭐ **E a causa raiz não é da Timeline: o `Transform` da cena é 2D** (`Vec2` + um `f32` de
rotação). Ela anima exactamente o que existe para animar. Os 3D guardam a pose noutro sítio —
o Model num componente próprio (`FieldPose { Xform }`, com `[f32;3]` e quaternião), e o Sculpt
**em lado nenhum do mundo** (`grep -rn 'Sculpt' crates/ph2d-ecs/` devolve **zero**).

⛔ **O item 3 é pré-requisito, não uma parte do item 3:** enquanto a escultura for um campo do
estado do app, ela é inalcançável por **tudo** — undo por-componente, persistência, instâncias, e
qualquer pergunta do tipo *«que objectos existem?»*. ⭐ O molde da cura já existe: o
`PaintedDoc(u32)` é a ponte do Painter e carrega **só a identidade estável**.

✅ **E a decisão que eu devolvia aqui foi RESOLVIDA pela D9 — a resposta é «nenhuma das duas».**
Ver abaixo.

---

## D9 — **A engine é 2.5D**: canvas 2D em pixels, objectos 3D desenhados sobre ele

> **Enio, 2026-08-30:** *"Essa engine será uma engine 2.5d mais 2d que 3d. O canvas no runtime será
> 2d e a unidade principal é o pixel. Mas objetos 3d serão desenhados sobre o canvas 2d e esses
> objetos existirão e serão animados em 3d… a riqueza do volume, da luz e da textura 3d se movendo,
> contudo, em canvas 2d. Então para objetos 3d teremos todas as coordenadas 3d (pos, rot e scale,
> além de deformações e animações)."*

⭐⭐⭐ **Isto corrige a moldura que eu tinha posto na D8.** Eu apresentei «duas noções de pose» como
um preço a pagar pela saída barata. Com a engine 2.5D **por desenho**, duas noções de pose **é a
arquitectura**:

| | pose | unidade |
|---|---|---|
| objecto **2D** | `Vec2` + um ângulo + `Vec2` de escala | **pixel** |
| objecto **3D** | três eixos de posição · rotação nos três · escala · **deformação** | a sua |

⇒ ⛔ **O `Transform` NÃO sobe para 3D.** Ele descreve o canvas, e o canvas é 2D em pixels — que é o
que esta decisão fixa. ⭐ **E a medição que eu ofereci (contar os leitores do `Transform`) fica
CANCELADA**: ela era a régua de uma opção que deixou de existir. *Uma medição só vale enquanto a
pergunta que ela responde continua a ser a pergunta.*

⭐⭐ **Não é rumo novo — é a articulação do que o Sculpt já faz:** *"as mesmas quatro lâmpadas que
iluminam a tinta do Painter, resolvidas pela mesma função (`ph2d-light`)"*, e a razão de existir do
módulo é **a malha DOAR a normal** para a tinta 2D chapada sair acesa pela forma. A sobreposição
já existe e já compõe no mesmo alvo *"sem tocar no compositor 2D"*.

⛔ **O que falta é o OBJECTO.** Medido: os dois módulos 3D tomam a **janela inteira**
(`viewport_of(size)` = largura × altura; `Rect::new(0, 0, win_w, win_h)`) — hoje o 3D é um **modo de
edição em ecrã cheio**, não uma coisa com lugar e tamanho no canvas.

⭐⭐ **A consequência de desenho: um objecto 3D carrega DUAS poses**, e não é conflito — são duas
perguntas. *«Onde na página?»* (2D, pixels — o `Transform` continua a servir) e *«como está
virado?»* (3D). É o modelo de uma camada 3D numa composição 2D.

⚠️ **E isto arruma a Timeline sem contradição:** o `PropKind` ganha **canais 3D gateados ao tipo do
objecto** — exactamente a lei da D6 (*os canais dependem do tipo*, como os modos). Os 13 canais 2D
ficam onde estão.

### D9.1 — As quatro que a D9 abriu, **respondidas no mesmo dia**

Detalhe e mecanismo em [`pesquisa/06 §6`](pesquisa/06_a_engine_e_2_5d.md).

| | resposta do Enio | o que ela força |
|---|---|---|
| **z-index** | *"terá um z-index como o 2d, logo ficará **entre camadas**"* | ⭐ o `ZIndexOverride(i32)` **já existe** (modelado no Godot). ⛔⛔ Mas o 3D **deixa de poder ser um passe final**: tem de desenhar para textura própria e entrar na pilha **como camada** |
| **unidades** | *"em 3d usaremos **metros**. Para rot **graus**"* | ⭐ **nada a inventar — a cena já é métrica.** Graus é unidade de **autoria**; guarda-se radianos. ⛔ Lacuna: o `Xform` tem escala **uniforme** (`f32`) e precisa de três |
| **deformações** | *"as **3**"* — esqueleto · poses-alvo · gaiola | três obras independentes, ⛔ **nomeadas e não desenhadas** |
| **câmera** | *"não sei dizer. Mas será a que dá **mais possibilidades**"* | ⇒ **por objecto**, com uma da cena por omissão — o critério dele decide sozinho (ver abaixo) |

⭐⭐⭐ **O achado do z-index: um primitivo, TRÊS consumidores.** Pôr o 3D entre camadas exige
renderizá-lo para uma textura que entra na pilha 2D — que é **exactamente** o primitivo que o
*W-Saída* do Flip já pede (*"é UM buraco, não três"*), e que os 16 exportadores de imagem também
esperam. ⚠️ E ⛔ **não contradiz a recusa do [`pesquisa/05`](pesquisa/05_pintar_sobre_vetor.md)**:
lá seria um **assado guardado** (mata a editabilidade), aqui é um **render por quadro** (o objecto
continua 3D). *Assar e renderizar lêem-se igual e não são a mesma coisa.*

⭐ **A câmera é a única resposta que esta linha DERIVOU** em vez de receber, e o critério do Enio
decide-a porque a relação não é simétrica: com câmera **por objecto**, «todos partilham a mesma» é
exprimível (todos apontam para uma); com **uma da cena**, «este tem a sua» **não é exprimível de
todo**. *O geral contém o particular; o particular não contém o geral.* ⚠️ Marcada como **derivada
do critério**, e ⛔ reversível se o critério mudar.

### D9.2 — **As duas réguas ficam AMBAS no app**: px *e* metros, graus *e* radianos

> **Enio, 2026-08-30:** *"Devemos ter ambas as opções no app (px e metros, graus e radianos)."*

⚠️ Isto **emenda** o que eu escrevi na D9.1, onde tratei «graus» como *a* unidade de autoria. Não é
*a*: é **uma das duas**, escolhível. A lei que fica de pé é a outra metade do que eu disse —
**unidade de LEITURA ≠ unidade de ARMAZENAMENTO**: guarda-se metros e radianos, sempre.

⭐⭐ **Metade já ship-a, com a arquitectura certa**
([`medicoes/05_as_duas_reguas.md`](medicoes/05_as_duas_reguas.md)):
`DisplayUnit { Meters, Pixels }` tem enum, menu (*Settings → Display unit*), ponte
(`pixels_per_meter`), persistência no ficheiro e **62 consumidores**. E traz três decisões já
tomadas **com o motivo escrito**, que a metade em falta herda: fica **fora do `ProjectState`**
(⚠️ trocar a unidade **não entra no undo**, preço declarado) · **viaja no ficheiro** (*"são knobs
que ESQUECEM"*) · e o espelho do ficheiro tem **gate de round-trip por `PartialEq` inteiro** —
⭐ que **já defende o campo que ainda não existe**.

⏳ **E a outra metade está a um campo de distância:** `Unit { Px, Meters, Degrees, Radians,
Percent }` já existe no widget, com sufixo e parser (há teste: `parse("2.25rad")`). ⛔ **Mas
`Unit::Radians` tem 5 usos e os cinco estão dentro do próprio ficheiro** — nada no app alguma vez
**mostra** um ângulo em radianos.

⚠️ **Não é um id órfão nem um knob morto — é uma terceira forma**, e vale distingui-la porque as
curas diferem: a **entrada já aceita** `rad`, e é a **saída** que nunca o produz. *Meio caminho
ligado: o app lê radianos e nunca os escreve.*

⇒ Falta **um campo** (`DisplayAngle`), **um menu** (irmão de `settings_unit.rs`, 34 linhas), **o
campo no espelho** — e o trabalho real: os sítios que hoje fixam `Unit::Degrees` passarem a
**perguntar**, como os 62 fazem com o comprimento.

⛔ **Armadilha nomeada:** nem todo ângulo é do artista. O `skew_x`/`skew_y`, a **fase** de um
oscilador e o ângulo de um gradiente também são radianos. A troca vale para os ângulos **autorados**
e a lista tem de ser explícita — senão a unidade escorrega para leituras onde não significa nada.

---

### ⛔ E uma correcção minha, no ponto que decidia o desenho

A primeira redacção da D9 e do `pesquisa/06` dizia que o objecto 2D se posiciona **em pixels**.
**Está errado:** `crates/ph2d-ecs/src/transform.rs:55` diz *"translation (**meters**), rotation
(**radians**)"*, e o `CLAUDE.md` §5 já o afirmava no módulo de Física (*"o `Transform` já é
metros"*, ADR-0131).

⭐ A correcção **simplifica**: não há ponte de unidade a inventar entre 2D e 3D — a cena é métrica
por inteiro, e o pixel é a unidade da **arte**, com o `pixels_per_meter` a ligá-los. As duas frases
do Enio (*"a unidade principal é o pixel"* e *"em 3d usaremos metros"*) descrevem **dois níveis**,
não um conflito.

---

## O que estas seis decisões implicam, junto

⭐⭐ **As três convergem na MESMA peça em falta: um modelo de REGIÃO / ÁREA.**

- **D1** precisa de slots onde um dock possa estar (Godot: 12 slots enumerados).
- **D2** precisa de um cabeçalho **por área** — logo, precisa que áreas existam.
- **D3** precisa que um modo possa dizer *"esta área tem este editor"*.

⇒ **A primeira obra não é nenhuma das três: é o modelo de áreas.** Sem ele, D1 vira painéis
ancorados sem sítio, D2 vira cabeçalhos sem dono e D3 vira um selector que não sabe o que
arrumar.

⭐ **E a D4 + D5 dizem-lhe a forma:** áreas com **encaixes enumerados** (D4), e dentro de cada
área **regiões em fila** — cabeçalho, ferramentas, régua, conteúdo (D5). O rascunho está em
[`spec/01_modelo_de_areas.md`](spec/01_modelo_de_areas.md).

⛔ **E a ordem tem uma trava dura**
([`medicoes/03 §5`](medicoes/03_o_censo_de_cor.md)): reduzir os temas de 4 para 2 **antes** de
separar layout de paleta **apaga o único modo ancorado que o app tem** (o `blueprint`). A paleta
mexe-se **depois**.
