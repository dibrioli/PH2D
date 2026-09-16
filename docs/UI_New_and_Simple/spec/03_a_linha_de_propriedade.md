# Spec — a LINHA DE PROPRIEDADE (2026-09-14)

> ⛔⛔ **Esta página é o «modelo pronto e bem estabelecido» que o Enio pediu** em 2026-09-14:
> *«falta para nós um modelo pronto e bem estabelecido com todas as regras para todos os widgets …
> o inspector está assustador de horrível»*. Ela cobre **uma** superfície — a linha de propriedade,
> que é a que mais se repete no app — e é o referente da ordem permanente dele: *«se temos o manual
> precisamos converter o APP todo a ele»*.
>
> ⚠️ **Cada lei tem PORTA e GATE, e a tabela do §9 é VERIFICADA por um teste**
> ([`the_property_row_manual_names_only_doors_that_exist`](../../../crates/ph2d-editor-core/tests/it/the_property_row_manual_names_only_doors_that_exist.rs)):
> um nome que deixe de existir reprova. *Um doc que enuncia a lei que o código não implementa lê-se
> como auditado* — e esta casa já pagou essa forma.
>
> ⛔ **Nenhum número desta página é escolhido.** Cada um traz a medição ou o dono ao lado.

---

## §1 — O que é uma linha de propriedade

Uma faixa de altura [`ROW_H_PX`] que reparte a largura entre **um nome** e **um controlo** que edita
uma propriedade de um objecto. É a unidade que o Inspector, o painel de vetor, o de física, o de
modelação e outros dez repetem dezenas de vezes por ecrã.

⛔ **Não é** uma linha de lista (a Hierarquia), nem uma faixa de faixa (a timeline), nem um cabeçalho
de secção. Esses têm leis próprias e **não** devem ser convertidos a esta.

---

## §2 — HÁ DUAS FORMAS, e a regra que escolhe é uma pergunta sobre o VALOR

| forma | quando | como se lê |
|---|---|---|
| **Caixa única** | o valor tem uma **fracção** para mostrar (um intervalo: `0..1`, `0..360`, um slider) | `[ Nome ················ 62% ]` — rótulo **DENTRO**, à esquerda; valor dentro, à direita; o **preenchimento** diz a fracção |
| **Linha de propriedade** | o valor **não** tem fracção (um número livre, um texto, uma caixa de verificação, um selector) | `Nome ····· [ 2 m ]` — rótulo **FORA**, à direita; controlo a começar no meio |

⚠️ **A partição é principiada, não histórica.** A caixa única existe porque o preenchimento **é**
informação; onde não há fracção, uma caixa larga só diz *«o número é o assunto»*, que foi a queixa
do dono em 2026-09-14 (*«as caixas numéricas são muito grandes. Maiores que as labels»*).

⛔⛔ **E as duas NÃO são convertíveis uma na outra na largura que o artista usa.** Medido: pôr o
rótulo fora numa linha com trilho deixaria, no mínimo do dock, **`72 px` para o trilho E o número
juntos** — que é exactamente o cromo fixo (`rótulo 70 | trilho | caixa 72` = `154 px`) que a caixa
única nasceu para matar em 2026-09-02. ⇒ *quem uniformizar as duas tem de trazer a medição que
desfaz esta.*

**Censo (2026-09-14):** caixa única em **15** painéis / 53 chamadas · linha de propriedade em **5**
painéis / 22 chamadas · **seis** painéis usam as duas, e está certo.

---

## §3 — A COLUNA: o controlo começa no MEIO da linha

⛔ **Ordem do dono, 2026-09-14:** *«Melhor alinhar no meio do painel e as labels alinhadas todas à
direita (no centro do painel)»*.

- `LABEL_COL_FRAC = 0,5` — a fracção é da **LINHA**, não do utilizável: a coluna de animação sai do
  lado do controlo, e *«o meio do painel» é o meio do que o artista vê*.
- O rótulo acaba **um vão [`Spacing::Md`] antes** do meio, para o controlo começar exactamente nele.

⚠️ **Ela já foi MEDIDA e deixou de o ser, e a troca é honesta.** Nasceu a reproduzir o literal que a
casa mais escrevia (`96/276 = 0,348`) — arqueologia correcta do produto de então. *Uma medição diz o
que o produto FAZ; ela nunca disse o que ele DEVIA fazer* (`CLAUDE.md` §0.8).

⛔ **Uma largura FIXA está errada por construção:** a coluna docada é arrastável
([`PANEL_MIN_W_PX`]`..720`). Antes desta lei a mesma pergunta tinha **onze** respostas no app
(`96` · `78` · `84` · `76` · `72` · `64` · `150` · `176`), cada uma um literal com dispensa.

---

## §4 — O RÓTULO: à direita, elidido, medido no peso em que pinta

1. **Alinhado à direita** da coluna — encostado ao controlo. Com o controlo a começar no meio, um
   rótulo à esquerda deixa um rio de espaço variável: *quanto mais curto o nome, mais longe do valor
   que ele nomeia*.
2. **Elidido** quando não cabe, **nunca** transbordado nem quebrado em duas linhas.
3. **Medido no peso em que é pintado** (`Medium`). ⛔ Medir em `Medium` e pintar em `SemiBold` corta
   `0,74 %`–`1,69 %` curto — `1,3 %` de `110 px` é um caractere, exactamente na fronteira em que o
   corte existe.
4. **A reticência nunca fica pendurada num espaço**: o prefixo é aparado antes de a receber
   (`Air Jump …` era o defeito).
5. **Um rótulo que CABE nunca é pintado com reticências.** ⛔ A largura que o pintor recebe é a
   **medida** do texto que coube, nunca `x + col_w − recuo`: em `f32` essa diferença cancela um ULP
   abaixo em **8,6 %** das posições e o pintor corta um nome com `40`–`77 px` de folga.
6. Quando nem a reticência cabe, o rótulo **degrada para a esquerda** e o pintor **recusa** em vez de
   invadir o controlo.

---

## §5 — O PISO DO CONTROLO é o que o CONTROLO declara

⛔⛔ **Ordem do dono, 2026-05-24**, escrita no doc de [`NUMBER_INPUT_MIN_W_PX`]: *«não permita que a
caixa seja redimencionada para menor que isso»* ⇒ **`72 px`**.

⚠️ **Um piso que NOMEIA o recurso ainda pode estar errado sobre ele.** A 1.ª redacção desta lei disse
*«o piso é [`ICON_BTN_SIZE_PX`] (36) — a largura da coluna do stepper — mais um dígito»* ⇒ `48`. A
coluna do stepper é [`stepper_width`] = `clamp(0,6 × altura, 16, 22)`, **nunca 36**. ⇒ *quando o
recurso já tem dono, o piso é o dele*.

⭐⭐⭐ **E a lei do §3 sobrevive exactamente até ao mínimo do dock, sem ninguém a escolher:** o rótulo
só pode ser a metade enquanto `metade ≤ tecto`, o que dá `linha ≥ 2 × (piso + DECORATOR_W)` =
**`172`**; a linha de um cartão do Inspector no mínimo do dock mede
`220 − 2×18 − 2×6` = **`172`**. Os dois números encontram-se ao píxel, e há gate a afirmá-lo.

---

## §6 — O EMPRÉSTIMO: o rótulo toma a folga que o controlo não usa

`coluna = clamp(o rótulo mais largo da SECÇÃO, a metade, o que o controlo pode ceder)`.

- **O piso é a metade** — a ordem do §3 continua a valer, e numa coluna larga nada muda.
- **O tecto é o controlo** — ele nunca desce do piso do §5.
- **A granularidade é a SECÇÃO**, nunca a linha nem o painel. Uma coluna por linha põe cada controlo
  num `x` diferente e a coluna sai esfarrapada; uma por painel faz a secção de nomes curtos herdar a
  largura do nome mais comprido do painel inteiro.

**Medido** (§14 do Inspector, 52 rótulos, ao longo do curso do dock):

| painel | coluna | campo | rótulos elididos |
|---|---|---|---|
| `220` (mínimo do dock) | `78,00` | `72,00` | 16 |
| `245` | `103,00` | `72,00` | 3 |
| `273,3` | `113,88` | `89,42` | **0** |
| `304` (omissão) | `120,00` | `114,00` | 0 |
| `720` (máximo do dock) | `328,00` | `322,00` | 0 |

### ⭐⭐⭐ A granularidade DEIXOU DE SER UMA NOTA e passou a ser um TIPO (2026-09-15)

⛔⛔ **Report do dono, com foto do painel da Grelha e uma seta na linha *Major every (px)*:** *«a
caixa recua quando na verdade o nome deveria criar as colunas»*.

A lei acima estava escrita e **a fiação não a cumpria**: o pintor media o rótulo **DESTA LINHA**.
Medido a `220` de painel:

| linha | o nome quer | a coluna que ela recebia | a caixa |
|---|---|---|---|
| `Cell size (px)` | `70,1` | `90,0` (a metade) | `x = 98,0` · `w = 84,0` |
| **`Major every (px)`** | **`92,2`** | **`92,2`** | **`x = 100,2` · `w = 81,8`** ⛔ |
| `Origin X (px)` | `70,7` | `90,0` | `x = 98,0` · `w = 84,0` |

⚠️⚠️ **E o nome entra na conta DUAS vezes** — como o que a coluna pede emprestado (§6) e como o
**piso da cedência** (§6-ter). Na secção *Transform* do Inspector a `273,3` os dois papéis produziam
`104,6` para `Position X / Y` e **`56,3`** para `Rotation`: `48 px` de desalinhamento **dentro da
mesma secção**, a mesma doença num regime mais largo.

⇒ **as duas grandezas da conta passam a viajar juntas, num tipo:**

```rust
Seccao::medida(text_system, campos, &nomes_da_seccao)
```

- ela mede o rótulo mais largo **na fonte e no peso em que ele pinta** — a medição é da porta, nunca
  do painel;
- a secção declara-a **uma vez** e entrega-a a **todas** as linhas dela — as de campo e as cujo
  controlo o painel constrói (`paint_label_row`), senão metade da secção mede e a outra metade não;
- `Seccao::apenas_campos(n)` é o «não sei que nomes vou pintar»: a coluna fica na metade e não há
  cedência.

⏳ **A excepção que fica, NOMEADA:** a `number_cell` do painel de vetor continua a medir o nome da
linha. Ela é uma **célula de uma grade de duas**, chamada de ~33 sítios em 39 secções, e metade
delas escolhe entre meia largura e a linha inteira **por linha** — *declarar ali a secção seria
declarar uma que não existe*. A conversão pede que as secções daquele painel sejam definidas
primeiro.

---

## §6-bis — A linha de VÁRIAS COMPONENTES: o controlo REFLUI, a coluna do rótulo não

⛔⛔ **Duas ordens do dono contrariam-se numa row de N campos, e a spec tem de escolher:** o §3 põe o
nome ao lado (*«Label acima do campo numérico! Muito ruim!»*, 2026-09-14), o que entrega ao controlo
**metade** da linha; o §5 põe um piso de `72 px` em cada caixa (*«não permita que a caixa seja
redimencionada para menor que isso»*, 2026-05-24). À largura de omissão do Inspector a coluna do
controlo mede `128 px` e **dois** campos ao piso pedem `148`.

⭐ **A saída NÃO é encolher a coluna do rótulo para esta linha** — a granularidade dela é a SECÇÃO
(§6), e uma coluna por linha devolve a coluna esfarrapada que o §6 recusa.

⇒ **as componentes que não cabem ao piso DESCEM, dentro da coluna do controlo** — é a lei que a
`seg_row` já praticava para um segmentado (*«o controlo reflui e a coluna do rótulo não»*), e é como
o Blender desenha um vector num painel estreito.

| interior | coluna do controlo | 2 campos | 4 campos |
|---|---|---|---|
| `200` (mínimo do dock) | `86,0` | 1×2 linhas | 1×4 linhas |
| `253,3` (o dock do dono) | `112,7` | 1×2 linhas | 1×4 linhas |
| `284` (omissão) | `128,0` | 1×2 linhas | 1×4 linhas |
| `380` | `176,0` | **2×1** | 2×2 linhas |
| `700` (máximo do dock) | `336,0` | **2×1** | **4×1** |

⚠️ **A largura é a MESMA em todas as linhas** — a última fica curta em vez de esticar o campo que
sobra; um `Bounds` de quatro componentes acabaria com o `H` ao dobro da largura do `X`.

⚠️ **Quando nem UM campo cabe ao piso, o campo recebe a coluna inteira.** Ali o recurso que falta é
o painel, e encolher mais só apagaria o número.

⭐⭐ **UM ponto de animação por LINHA, nunca por campo** — um par `X`/`Y` é *uma* propriedade com duas
componentes, e dois pontos diriam que são duas.

### O `lead`: quando a componente tem decoração própria

⛔⛔ **Ordem do dono, 2026-09-15:** *«a mesma formatação do Position X/Y que fez para Anchor vou
querer para todo o Transform»*. As linhas do Transform trazem uma **letra de eixo colorida**
(`X` / `Y`) antes de cada caixa — e ela é parte da **componente**, não do rótulo da linha.

⇒ a porta leva um `lead`: a largura que a decoração consome antes da caixa. **O piso continua a ser
o da CAIXA**; a célula precisa de `lead + piso` e a caixa mede `célula − lead`. ⛔ Somar o `lead` ao
piso pareceria igual e mentiria sobre o recurso — quem tem dono é a caixa (§0.0).

⛔⛔ **E o Transform DEIXOU de o usar no dia seguinte, por report do dono** (*«A disposição ficou
diferente. VC tinha colocado x e y na mesma linha. Position X/Y Caixa Caixa»*): com `lead = 14` as
duas componentes só ficavam lado a lado acima de um painel de **`~400`**, e sem decoração ficam
acima de **`~348`**. O dock dele está em **`369,74`** — *a letra de eixo custava-lhe exactamente a
disposição que ele tinha aprovado nas Âncoras.* ⇒ o `X`/`Y` viaja no NOME (`"Position X / Y"`), e
hoje **nenhuma linha do app usa `lead ≠ 0`**.

⚠️ **O parâmetro FICA, e o gate varre os dois regimes** (`lead ∈ {0, 14}`): a decoração por
componente é uma pergunta que volta, e a lei dela está medida. ⛔ Mas *ela não é grátis, e o número é
`~52 px` de largura de painel* — quem a reintroduzir paga isso.

⭐⭐⭐ **E o alinhamento que o dono pediu em 2026-05-24 — *«a caixa única de Rotation deve se alinhar
à caixa de X à esquerda e à direita»* — passa a sair de GRAÇA.** Era uma fórmula escrita à mão
(`2 × two_chip_w + col_gap + axis_col_w + tag_box_gap`), e o comentário dela registava que já
estivera **errada**. Com `n = 1` a célula **é** a coluna do controlo: ela começa onde o `X` começa e
acaba onde o `Y` acaba, nos dois regimes. *Uma propriedade obtida por construção não pode ficar
errada* — e tem gate mesmo assim, porque a refactoração seguinte pode perdê-la em silêncio.

⛔ **DUAS excepções, e as duas com a mesma medição:** a grelha 3×3 do 9-slice e as quatro amostras de
canto do *Color/Tint* ficam com o nome POR CIMA, porque o controlo delas não é o bloco — é o bloco
**mais um companheiro à direita dele** (os dois atalhos · a prévia do gradiente), e o par mede
`~144 px`. A coluna do controlo vale `0,5 × interior − 14`, logo só lá chega acima de um painel de
**`336`**; o dock do dono está em `273,3`. Elas são **blocos com legenda**, não linhas de
propriedade — e há gate com a lista, com a metade da obsolescência.

---

## §6-ter — A CEDÊNCIA: a metade encolhe antes de deixar a linha quebrar

⛔⛔ **Report do dono, 2026-09-15, com foto do Transform:** *«O painel ainda largo com espaço à
esquerda e as linhas já se quebram (caixa y passa para baixo. Isso não pode acontecer. Encontre a
solução»*.

Na foto, `Position X / Y` mede **`~88 px`** numa coluna de **`~154`** — **`~66 px` de vazio** à
esquerda do nome — e o `Y` desce porque ao controlo faltavam **`7 px`**. *A coluna do nome estava a
guardar espaço que não usava enquanto a do lado passava fome.*

⇒ **a metade (§3) deixa de ser um PISO e passa a ser um ALVO**, com três degraus:

1. **empréstimo** (§6): um nome mais largo que a metade passa dela;
2. **cedência** (esta): a metade encolhe até o controlo ter o que precisa;
3. **o piso da cedência é o que o NOME precisa** — medido, no peso em que pinta.

⚠️⚠️ **Só se cede quando a cedência RESOLVE.** Se nem com o nome no mínimo o controlo coubesse,
encolher a coluna troca *uma linha quebrada* por *um nome cortado* — e a linha continua quebrada.
Aí não se cede nada e a coluna fica onde o dono a pôs.

⚠️ **O `control_need` é da SECÇÃO, não da linha.** Se cada linha cedesse pelo que ELA precisa, a
linha de um campo (*Rotation*) não cederia e a de dois cederia — e a coluna saía esfarrapada, que é
o que *«as labels alinhadas todas à direita»* proíbe. ⇒ cada secção declara **a linha que ela não
quer ver quebrar**, e todas cedem o mesmo.

**Medido** (um par `X`/`Y`, nome de `88 px`):

| painel | coluna do nome | controlo | resultado |
|---|---|---|---|
| `220` | `92,0` | `86,0` | empilha (não há folga: `88 + 8 + 147 > 206`) |
| `273,3` | `118,7` | `112,7` | empilha (idem) |
| `304` | **`115,0`** (cedeu de `134`) | **`147,0`** | ⭐ **lado a lado** |
| `343` (a foto) | `153,5` | `147,5` | ⭐ **lado a lado** |
| `369,7` (o dock dele) | `166,9` | `160,9` | lado a lado |
| `720` | `342,0` | `336,0` | lado a lado |

⭐ A quebra passa a acontecer **só** onde `nome + vão + controlo > utilizável` — isto é, onde de
facto não há folga nenhuma.

---

## §6-quater — EMPARELHAR é uma escolha; CABER é uma medição

⛔⛔ **Report do dono, 2026-09-15, com duas fotos (o mesmo painel largo e estreito):** *«Em Grain:
Voronoi : Metric e Edges os nomes somem ao estreitar o painel. Melhor seria quebrar a linha»*.

Aqueles dois vivem **emparelhados**, cada um numa METADE da largura. Ao estreitar, a coluna do nome
de uma metade fica menor do que a própria reticência e o `paint_property_label` devolve **string
vazia** — que é a resposta **certa** dele para uma coluna degenerada (§4: *«a caixa fica só com o
número, que é o degrau seguinte da escada do estreito»*) e a **errada** para quem escolheu
emparelhar.

⇒ *o degrau a seguir a «não cabe o nome» não é apagar o nome: é **deixar de emparelhar**.*

⚠️ **Duas perguntas diferentes, e só uma é de produto:**

| pergunta | quem responde |
|---|---|
| *que propriedades PODEM partilhar uma fileira?* | o painel — é desenho (nomes curtos, assuntos irmãos) |
| *elas CABEM aqui?* | a porta, `property_row_fits(w, largura_do_nome)` |

⛔ **E a segunda pergunta vai à PORTA, nunca a uma segunda aritmética:** o veredito sai da mesma
`property_row_columns_for` que vai desenhar. *Duas contas para «isto cabe?» divergem no dia em que
uma das leis muda.*

**Medido** (`Metric`, `36,8 px` a `Sm`): a fileira emparelhada parte abaixo de um painel de
**`~300 px`** — exactamente a faixa entre as duas fotos dele.

---

## §6-quinquies — A LINHA DE MARCAR é uma linha de propriedade, e o nome dela vai à coluna do nome

⛔⛔ **Medido em 2026-09-15:** o widget mais usado do app (**81** sítios fora do `ph2d-editor-core`)
pintava o nome **encostado à esquerda da faixa**, a correr até à marca, com um orçamento próprio.
Num formulário que alterna linhas de número com linhas de marcar isso dá **duas colunas de nome**,
alternando linha sim linha não — *a mesma queixa do rótulo por cima do campo (§6-bis), meia volta
adiante*.

⇒ a linha de marcar passa pela mesma porta: **nome na coluna do nome, alinhado à direita, elidido,
na coluna da SECÇÃO** (§6). A [`Seccao`] viaja no próprio widget (`Checkbox::seccao`), e o default é
«sou uma linha de formulário» — são 27 sítios contra um.

### ⭐⭐⭐ E a MARCA vive dentro de uma CAIXA — a aparência do inspector do Godot

⛔⛔ **Ordem do dono, 2026-09-15, com duas fotos (Godot e Blender):** *«tanto o Blender como o Godot
têm checkbox mais sofisticada que a nossa. Na Godot coloca um box em todo o lado direito da linha e
dentro do box o checkbox alinhado à esquerda. Vamos adotar essa aparência»*.

```
.........Nome    [ ☑                        ] ·
         ^ a coluna do nome    ^ a caixa = a COLUNA DO CONTROLO      ^ a coluna de animação
```

- a **caixa** ocupa a coluna do controlo — começa no meio da linha e acaba na margem direita,
  **exactamente como a caixa de um número**;
- a **marca** encosta à ESQUERDA dentro dela, recuada **um degrau** ([`Spacing::Xs`]) de cada lado;
- a **palavra do valor** (*On*) fica à direita da marca, dentro da caixa;
- o alvo do clique é a linha inteira, como sempre.

⭐⭐ **Com isto a linha de marcar deixa de ser a excepção do §3.** A redacção anterior desta secção
dizia que a marca ficava na coluna do VALOR *«e é uma divergência deliberada»*, porque era ela que
dava ao formulário **uma** margem direita. Hoje essa margem sai da **própria caixa**, e o *«o
controlo começa no meio da linha»* passa a valer também aqui. *A cura do dono desfez uma excepção
que eu tinha escrito como permanente.*

⚠️ **A superfície é a PORTA do campo** ([`paint_field_surface`]) — o raio pelo tema, o fundo pelo
`field_fill`, a moldura com o eixo do hover. ⛔ Nunca uma cópia das quatro linhas que o
`paint_number_input_with_buffer` escreve: *duas superfícies de campo que hoje concordam são duas que
amanhã divergem*, e este pintor já pagou isso ao escrever `Bg1` (a cor de um CARTÃO) e deixar a
caixa invisível.

### ⭐⭐ A PALAVRA do valor, e as duas correcções que o dono pediu a seguir

⛔⛔ **Report do dono, 2026-09-15, com foto:** *«checkbox ficou maior que a caixa e não foi bem
alinhado à esquerda. Godot melhor. Outra coisa: Godot define a checkbox como uma palavra (On) à
direita. Isso me parece bom»*.

**(a) A marca enchia a caixa** — e a causa era aritmética em dois sítios:

| | antes | agora |
|---|---|---|
| altura da linha | `18`, o **mesmo literal em treze secções** do Inspector | [`ROW_H_PX`] = `22`, a altura de toda linha de propriedade |
| lado da marca | `min(18, 18) = 18` — a caixa TODA | `min(18, 22 − 2×`[`Spacing::Xs`]`) = 14` |
| recuo à esquerda | [`field_pad_x`] = `12` | [`Spacing::Xs`] = `4` |

⚠️ **O recuo do TEXTO não serve para a marca:** o valor de um campo precisa de folga para o caret e
para a selecção; *uma marca não tem caret*, e `12 px` liam-se como «não está à esquerda».

**(b) A palavra ENTRA** (`chrome.checkbox.on`). ⚠️ Ela **não muda com o valor**, e é assim no alvo:
na foto do dono três linhas dizem *On* e só uma está marcada — a palavra nomeia o que a marca LIGA,
como o rótulo de um interruptor de parede, e quem diz se está ligado é a marca.

⛔ **Eu tinha-a deixado de fora**, por ela parecer um rótulo constante ao lado de um indicador que
varia (a família que o `CLAUDE.md` §5.0 caça). Devolvi-lhe a decisão com o motivo; **ele decidiu, e
a decisão é dele**.

⚠️ **O INTERRUPTOR segue a mesma lei** (a linha *Show grid* do painel da Grelha): ele era um
comprimido de `40 × 20` encostado à margem direita, com dois literais próprios — *o único controlo
daquele painel que não acabava onde os campos acabam*.

⛔ **Há DUAS excepções, as duas com nome** (`Checkbox::fora_do_formulario`), e as duas pela mesma
razão — *não há secção nenhuma de que a coluna possa ser*:

| quem | o que a linha é |
|---|---|
| a **pele de canvas** | a moldura é o que o **artista** desenhou, não a linha de um painel |
| uma **célula do `BitmaskGrid32`** | um QUARTO de linha, com o número por etiqueta |

⚠️ É um campo **próprio** — derivá-lo do `box_px` ou do `decorator` seria o erro que o doc do
`box_px` já regista e que um gate já apanhou uma vez.

### ⛔⛔⛔ E a CAIXA NÃO CABE EM MEIA LINHA — o que emparelhava, PARTE

**Medido em 2026-09-15, um dia depois de a caixa entrar.** A caixa ocupa a coluna do controlo e o
piso dela é o do campo (`NUMBER_INPUT_MIN_W_PX` = `72`, ordem do dono de 2026-05-24). Numa METADE
de linha os dois não cabem, e **quem paga é o nome**:

| painel | METADE da linha | coluna que sobra ao nome | os dez nomes medem |
|---|---|---|---|
| `220` (mínimo do dock) | `83,0` | **`0,0`** ⛔ *nome nenhum é pintado* | `28,5`–`79,8` |
| `273,3` (a do dono) | `109,6` | **`15,6`** ⛔ | idem |
| `304` (omissão) | `125,0` | **`31,0`** ⛔ | idem |
| `420` | `183,0` | `83,5` ✅ | idem |

⇒ as **cinco** fileiras emparelhadas de booleanos do Inspector (*Animation* · *Timers* · *Audio* ·
*Camera* · o *Flip H/V* da folha) ficaram com os **dez** nomes cortados em todo o curso útil do dock.
⚠️ **Elas tinham sido deixadas de fora da wave anterior de propósito** — *«ali a secção é o PAR»* —
e essa decisão foi tomada **sem medir o que a caixa faz a meia linha**.

⭐ **A lei que responde já estava escrita: o §6-quater.** *Emparelhar é uma escolha do painel; caber
é uma medição da porta.* A porta é `paint_check_rows`, que pergunta à `property_row_fits` e
**empilha** quando a resposta é não — e aí cada linha entra na coluna da **secção**, que é o que a
alinha com os números ao lado. Acima de `~400 px` ela volta a emparelhar sozinha.

⚠️ **E a mesma aritmética apagou os 32 números do `BitmaskGrid32`** (a *Cull Mask* da câmera e a
*Layer* da visibilidade): num quarto de linha (`43`–`64 px`) a coluna do nome dá **`0,00`** em todo
o curso do dock. *Uma célula de grelha não é uma linha de propriedade* — é a segunda excepção da
tabela acima.

⭐ **Custo da cura:** cinco secções do Inspector ficam **uma linha mais altas** (`22 + 3 px` cada) no
regime estreito, e voltam a emparelhar quando o dono alarga o dock.

### ⛔⛔⛔ E a MARCA DESMARCADA ficou invisível — a mesma queixa, um degrau mais fundo

**Report do dono, 2026-09-15, com foto:** *«O Checkbox desmarcado é invisível»* — a linha *Autoplay*
mostrava a palavra *On* e mais nada.

Em **14/09** ele tinha reportado *«Checkbox invisível»*: a marca enchia `Bg1`, a cor do **cartão** em
que assentava, e num tema moderno não há moldura de repouso. A cura foi a porta `body_fill` — *um
degrau abaixo da superfície mais funda*, que é o `Chrome::field_fill`.

Em **15/09 a marca mudou-se para dentro da CAIXA DE CAMPO** (a secção acima), e o fundo dessa caixa é
exactamente `field_fill`. Medido:

| tema | a caixa | a marca desmarcada | antes | depois |
|---|---|---|---|---|
| Dark | `#090909` | `#292929` | **`0`/255** ⛔ | `32` |
| Gray | `#121212` | `#3D3D3D` | **`0`/255** ⛔ | `43` |
| Light | `#DBDBDB` | `#E6E6E6` | **`0`/255** ⛔ | `11` |
| Oled | `#000000` | — | `0` | `0`, e ali **a moldura lê-a** (*Draw Extra Borders*) |

⇒ *o mesmo pixel duas vezes*. A marcada continuava a ler-se porque é `Accent` — foi por isso que o
report é sobre **metade** das caixas, exactamente como no dia anterior.

⚠️⚠️ **A lei que isto escreve:** *uma cura de contraste é calibrada contra uma SUPERFÍCIE, e quem
move a peça para outra superfície tem de reconferir a nota* (`CLAUDE.md` §0.0). A de 14/09 estava
escrita como se fosse absoluta, e quem moveu o degrau foi a wave da véspera.

⇒ **duas portas, e a superfície escolhe:** `body_fill` para um corpo que assenta num **cartão**;
`on_field_fill` para um que assenta num **campo** — e esta fala a família de estados de um widget
(`inactive → hovered`), que é o vocabulário certo para uma peça interactiva pousada noutra
superfície, e cuja ponta quente a primeira já usava. ⛔ A escolha sai da mesma `caixa` que decidiu a
geometria, nunca de um segundo `if`.

⚠️ **A barra é a mesma do gate irmão do `ph2d-tokens`** (`SURFACE_STEP` = `10`/255,
`a_field_is_never_the_colour_of_what_it_sits_on`), com a **mesma** cláusula de moldura — e ali as
superfícies eram três, porque **até 15/09 nenhum controlo assentava num campo**.

**Custo medido** (26 rótulos booleanos reais do app, `Sm`, quantos passam da metade da linha):

| painel | metade | não cabem |
|---|---|---|
| `220` (mínimo do dock) | `78,0` | 9 de 26 |
| `273,3` | `104,6` | 7 de 26 |
| `304` (omissão) | `120,0` | 4 de 26 |
| `333,1` | `134,6` | **1** de 26 |
| `369,7` | `152,9` | **0** de 26 |

⭐ E quase nenhum chega a cortar: os nomes das linhas de marcar **entram na medida da secção**, logo
a coluna cresce para os acomodar até ao tecto (§6). Na ponta estreita o tecto e a metade coincidem e
ali eles cortam — que é a troca que o dono escolheu em 2026-05-24.

---

## §7 — O que a linha leva SEMPRE

| peça | lei |
|---|---|
| **coluna de animação** | [`DECORATOR_W`] = `14 px`, em **todas** as linhas, sempre — decisão do dono (*«vou querer animar tudo»*). É um **indicador**, não um controlo: não regista clique nenhum |
| **a unidade** | dentro do campo, **colada ao número**, em `Text2`, e **some enquanto se escreve**. ⛔ Nunca no rótulo: com a unidade lá, **20 de 39** rótulos do Inspector eram cortados; sem ela, **1**. ⭐ Em 2026-09-15 o censo levou os restantes **28** (`Break Torque (N.m)`, `Init Vel X (m/s)`, `Non-Spatialized Radius (m)`, …) e o vocabulário ganhou `N`, `N.m` e `deg/s` |
| **o que o campo pinta, o campo lê** | o sufixo que se MOSTRA pode ter maiúscula (o `N` do SI) e o que o artista escreve não tem de a ter ⇒ a leitura é **insensível à caixa**. ⛔ Não há uma segunda string por unidade: duas strings para a mesma coisa divergem |
| **a superfície** | o preenchimento sai da porta do TEMA ([`body_fill`]), nunca do token do cartão. Num tema moderno o campo é um degrau **abaixo** da superfície em que assenta, e **não** leva moldura em repouso |
| **o valor** | alinhado à esquerda dentro do campo, com as setinhas na coluna da direita |

---

## §8 — O que NÃO está decidido

1. **A reticência acaba ~2 px antes do fim da caixa dela** (o *side-bearing* do glifo `…`). Todas as
   reticências acabam na mesma posição; o que difere é onde a tinta pára. Curar pede métricas de
   glifo.
2. **Três nomes não cabem** em nenhuma coluna utilizável no mínimo do dock
   (`Corner Look-ahead`, `Weight on Ground`, `Swim Line (weights)`) — encurtar é decisão do dono.
3. **A coluna de animação na ponta estreita:** ela é `18 %` do que sobra para o nome a `220 px`.
   Escondê-la abaixo de uma largura levaria os 16 cortados para `~9` — ⛔ e um indicador que aparece
   e desaparece ensina a desconfiar dele. Decisão do dono.
4. **A unidade de EXIBIÇÃO** (`m` ⇄ `px`) ainda não conduz o sufixo do campo: mostrar `px` sobre um
   número em metros trocaria um rótulo comprido por um rótulo **mentiroso**. Precisa da conversão do
   valor, não de um sufixo.
5. ⏳ **DUAS portas ainda não levam unidade**, e a catraca do
   `no_row_label_carries_its_own_unit` nomeia-as linha a linha: a `transform_row::paint_row` (o par
   de chips `X`/`Y` do Transform, onde o rótulo carrega ainda por cima a RÉGUA activa) e o
   `field_row` do 9-slice. ⚠️ **O `field_row` das âncoras JÁ leva** (2026-09-15) — a mesma unidade em
   todos os campos da row, porque uma row de `X`/`Y` mede a mesma grandeza nos dois.
   ⛔ E a razão de cada entrada da catraca é a **PORTA**, não *«é multi-campo»*: o `field_row` pinta
   uma row de UM campo (`Rotation (deg)`) e mesmo assim não levava sufixo. *O que separa não é a
   contagem de campos — é a porta ter, ou não, por onde a unidade entrar.*
5. **A coluna de nome de FAIXA da timeline** (`tracks.rs`) responde a outra pergunta e fica de fora
   **de propósito**.

---

## §9 — A tabela VERIFICADA: lei → porta → gate

> ⚠️ Esta tabela é lida por
> [`the_property_row_manual_names_only_doors_that_exist`](../../../crates/ph2d-editor-core/tests/it/the_property_row_manual_names_only_doors_that_exist.rs):
> cada nome da coluna **porta** e da coluna **gate** tem de existir no código. ⛔ Não acrescente uma
> linha sem o nome real.

| § | lei | porta | gate |
|---|---|---|---|
| §3 | o controlo começa no meio da linha | `property_row_columns` | `a_property_row_never_starves_its_control` |
| §3 | a coluna é uma resposta só, nunca um literal no sítio de pintura | `property_label_col_w` | `the_label_column_is_never_chosen_at_the_painting_site` |
| §4 | o rótulo é alinhado à direita e encostado ao controlo | `property_label_origin` | `a_property_label_is_flush_against_its_control` |
| §4 | um rótulo que cabe nunca leva reticências | `paint_property_label` | `a_label_that_fits_is_never_painted_with_dots` |
| §4 | a reticência nunca fica pendurada num espaço | `corte` | `the_ellipsis_never_hangs_off_a_space` |
| §5 | o campo nunca é pintado abaixo do que o dono declarou | `NUMBER_INPUT_MIN_W_PX` | `a_field_is_never_narrower_than_its_owner_declared` |
| §6 | o pintor pede emprestado a folga que o controlo não usa | `property_label_col_w_for` | `the_painter_borrows_the_slack_the_control_does_not_need` |
| §6 | numa secção, todas as caixas começam no mesmo `x` | `Seccao` | `a_seccao_poe_todas_as_caixas_na_mesma_coluna` |
| §6-quinquies | o nome de uma linha de marcar vive na coluna do nome | `Checkbox` | `a_linha_de_marcar_poe_o_nome_na_coluna_do_nome` |
| §6-quinquies | a pele de canvas não segue a coluna de uma secção | `fora_do_formulario` | `fora_do_formulario_o_nome_fica_onde_o_artista_o_pos` |
| §6-quinquies | a marca vive dentro de uma caixa que ocupa a coluna do controlo | `paint_field_surface` | `a_marca_vive_dentro_de_uma_caixa_que_ocupa_a_coluna_do_controlo` |
| §6-quinquies | a marca nunca toca a caixa, e encosta mais que o texto | `paint_boolean_mark` | `a_marca_nunca_toca_a_caixa_e_encosta_mais_que_o_texto` |
| §6-quinquies | a palavra do valor vive dentro da caixa | `MarcaPintada` | `a_palavra_do_valor_vive_dentro_da_caixa` |
| §6-quinquies | o interruptor pinta a MESMA marca que a caixa de verificação | `paint_boolean_mark` | `the_switch_paints_the_very_same_mark_as_the_checkbox` |
| §6 | quantos rótulos elidem, por largura do dock | `property_row_columns_for` | `the_elision_ladder_only_shrinks` |
| §6-bis | as componentes que não cabem ao piso descem, dentro da coluna do controlo | `property_fields_layout` | `a_row_of_many_fields_never_starves_them` |
| §6-bis | alargar o painel nunca faz caber menos campos por linha | `property_fields_layout` | `a_wider_panel_never_fits_fewer_fields` |
| §6-bis | a caixa sozinha cobre exactamente o que o par cobre | `property_row_columns` | `the_lone_field_of_a_row_spans_what_the_pair_spans` |
| §6-bis | dispor N campos numa linha tem UM chamador em todo o painel | `property_fields_layout` | `only_one_door_lays_out_a_row_of_fields` |
| §6-ter | uma linha nunca quebra enquanto a coluna do nome tem folga | `property_label_col_w_for` | `a_row_never_wraps_while_the_name_column_has_slack` |
| §6-quater | quem emparelha duas propriedades pergunta antes se o nome cabe | `property_row_fits` | `a_paired_row_breaks_before_its_name_disappears` |
| §6-quinquies | uma linha de marcar ocupa a linha inteira quando o par não cabe | `paint_check_rows` | `uma_linha_de_marcar_ocupa_a_linha_inteira` |
| §6-quinquies | a altura de uma linha de marcar é a de toda linha de propriedade | `paint_check_row` | `a_altura_de_uma_linha_de_marcar_e_a_do_app` |
| §6-quinquies | uma célula de mapa de bits não é uma linha de formulário | `cell_checkbox` | `uma_celula_nao_e_uma_linha_de_formulario` |
| §6-quinquies | uma marca nunca pinta a cor da caixa em que assenta | `on_field_fill` | `a_marca_nunca_e_a_cor_da_caixa_em_que_assenta` |
| §6-quinquies | e o pintor da linha de marcar usa essa porta | `paint_boolean_mark` | `e_o_pintor_da_linha_de_marcar_usa_essa_porta` |
| §6-bis | nenhuma linha do Inspector põe o nome por cima do controlo | `fields_row` | `no_row_paints_its_name_above_its_control` |
| §7 | toda linha reserva a coluna de animação | `form_row_columns` | `every_form_row_reserves_the_animation_column` |
| §7 | as portas que reservam a coluna DERIVAM-SE, nunca se enumeram | `property_label_row` | `the_door_census_derives_the_second_order_doors` |
| §7 | o preenchimento do campo sai do tema | `body_fill` | `every_field_painter_asks_the_theme_for_its_fill` |
| §7 | um campo nunca tem a cor daquilo em que assenta | `SURFACE_STEP` | `a_field_is_never_the_colour_of_what_it_sits_on` |
| §7 | um sufixo mais longo nunca é ensombrado por um mais curto | `Unit` | `a_longer_suffix_is_never_shadowed_by_a_shorter_one` |
| §7 | o campo lê de volta a unidade que ele próprio pinta, em qualquer caixa | `parse_suffix` | `every_unit_reads_back_what_it_paints` |
| §7 | nenhum rótulo do app carrega a unidade no texto | `num_row_unit` | `no_row_label_carries_its_own_unit` |
| §7 | a unidade é tinta dentro da caixa, nunca uma segunda caixa | `paint_number_input_with_buffer` | `the_unit_is_ink_inside_the_box_and_never_a_second_box` |
| §3 | um painel inteiro tem UMA coluna por secção, não uma por família de linha | `seccao_da_chave` | `cada_nome_deste_painel_cabe_na_coluna_da_seccao` |
| §3 | e a coluna cega cortaria mais — o controlo que prova que a declaração não é decoração | `Declaracao` | `e_a_coluna_cega_cortaria_mais` |
| §3 | declarar a secção nunca deixa o controlo abaixo do piso do dono | `colunas_da_linha` | `e_o_controlo_nunca_fica_abaixo_do_piso_do_dono` |
| §3 | a declaração de uma secção e o que o painel pinta dizem o mesmo | `TODAS` | `a_declaracao_das_seccoes_e_o_painel_dizem_o_mesmo` |
| §6-quinquies | toda caixa de marcar de um painel declara a secção dela | `paint_checkbox_row` | `toda_caixa_deste_painel_declara_a_seccao_dela` |
| §3 | nenhuma coluna de rótulo é escolhida no sítio de pintura, em nenhuma GRAFIA | `linha_da_chave` | `nenhuma_coluna_de_rotulo_e_escolhida_no_sitio_de_pintura` |
| §2 | a barra do cartão Line é caixa única, e o número dela grava o que diz | `registar` | `o_chip_de_cada_barra_projecta_o_que_a_ferramenta_grava` |
| §2 | cada barra de uma camada de ajuste é caixa única, com número editável quando é afim | `paint_barra` | `cada_barra_de_ajuste_e_uma_caixa_unica` |
| §2 | o número de uma barra de ajuste grava o que diz, lido por um oráculo na unidade do artista | `adjustment_slider_numbers` | `o_numero_de_cada_barra_de_ajuste_escreve_o_que_diz` |
| §4 | um nome pintado que nasce noutra crate passa pela tabela de strings | `chave_da_barra` | `cada_rotulo_de_ajuste_tem_nome_na_tabela` |
| §4 | e o pintor pinta o nome da tabela, não o rótulo cru | `pintar` | `o_pintor_pinta_os_nomes_da_tabela` |
| §6-quinquies | o nome de um interruptor cabe na coluna da secção dele | `seccao_de` | `cada_nome_de_interruptor_cabe_na_coluna` |

---

## §10 — Como converter uma linha a esta spec

1. `let row = property_row_columns(x, w, y, ROW_H_PX);` — ou `…_for(…, Some(mais_largo))` se a
   secção souber o rótulo mais largo que vai pintar (§6).
2. `paint_property_label(ts, scene, label, row.label.x, row.label.y + (row.label.h − font) * 0.5,
   font, row.label.w, resolve(ColorToken::Text2, theme));` — a assinatura é a do `paint_text_elided`
   **argumento a argumento**, de propósito: converter é trocar o nome da função.
3. `hit_index.register(id, row.control);` e pinte o controlo em `row.control`.
4. `paint_decorator_dot(scene, theme, row.dot);`
5. Se o campo tem unidade física, `input.suffix(Some(unit.suffix()))` — ⛔ **não** a ponha no rótulo.
6. **Se a linha tem N campos, não faça nada disto:** chame `rows::fields_row(…, &[ids], passo,
   unidade, None)` — ela é a porta, e o reflúxo do §6-bis vem de dentro. ⛔ Uma aritmética de
   `(w − gap × (n−1)) / n` escrita no sítio de pintura é a quarta resposta à mesma pergunta (havia
   **três** em 2026-09-15, divergentes na altura da caixa e na existência do ponto de animação).
7. **Se o controlo é construído à mão** (um segmentado com «nenhum aceso», uma grelha), chame
   `rows::property_label_row(…)`, pinte em `row.control` e termine com o ponto.

⛔ **O que NÃO fazer:** escrever a largura da coluna no sítio de pintura (há censo com catraca) ·
pôr a unidade no rótulo · pintar o rótulo com `paint_text` (perde a elisão e o alinhamento) · dar ao
pintor um orçamento derivado por subtracção (§4.5).
