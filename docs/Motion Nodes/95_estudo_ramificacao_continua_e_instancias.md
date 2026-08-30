# 95 — Estudo: ramificação CONTÍNUA e instâncias por letra/fase no `source.lsystem`

> **Dois reports do Enio (2026-08-30):**
> 1. *"as formas crescem sempre separadas e não crescem como um objeto só. Exemplo: o tronco de
>    árvore. O tronco deve ter uma estrutura única e não vários retângulos soltos sobrepostos."*
> 2. *"não há modo de escolher objetos para as fases de crescimento. Exemplo: as folhas.
>    Deveríamos ter um modo de escolher o objeto que será exposto (instanciado) em cada fase."*
>
> **Estudo, não plano.** O que se constrói e em que ordem é decisão dele. Aqui está o que o
> nosso código faz hoje (medido), o que as referências fazem (pesquisado, com fonte), onde
> exactamente está o buraco de cada report, e o que tem de ser MEDIDO antes de desenhar.

---

## §1 — O que o nosso nó já faz (MEDIDO, 2026-08-30)

⭐ **O `source.lsystem` já emite um ESQUELETO, não uma nuvem de pontos.** Por elemento:

| coluna | o quê |
|---|---|
| `P` | posição |
| **`parent`** | o índice do elemento anterior no ramo (`-1` = raiz) |
| `len` · `rot` · `wrot` | comprimento, ângulo local, ângulo de mundo |
| **`size`** | a ESPESSURA da tartaruga (grossa no tronco, fina no galho — o `!` da gramática) |
| `depth` | quão fundo no aninhamento de `[ ]` |
| **`gen`** | **em que geração este elemento nasceu** |
| **`sym`** | **que letra o desenhou** (`F` tronco · `J`/`K`/`M` folha/flor/instância) |
| `Index` · `Count` | os de sempre |

⚠️ **É exactamente o contrato de colunas do `rig.skeleton`** — o doc do `turtle.rs` diz-se assim,
e há gate a provar que `source.lsystem → rig.fk` é a identidade ao bit.

E as letras de instância **já existem**: o alfabeto declara `J K M` = *"pousa um elemento **sem**
segmento (folha, flor, instância)"*, e o interpretador emite-as com o `sym` preenchido.

⛔ **O que NÃO existe, e é o buraco dos dois reports:**

| medição | resultado |
|---|---|
| Há um passo que transforma a cadeia `parent` numa forma contínua? | ⛔ **Não.** Cada elemento é carimbado como um quad próprio ⇒ **N retângulos sobrepostos** por ramo. É o report nº 1, literalmente |
| O `source.lsystem` tem portas de entrada? | ⛔ **`inputs: &[]`** — zero. Não há onde ligar a folha |
| O `motion.cull` filtra por um atributo qualquer? | ⛔ **Não** — só *Fraction* e *Falloff*. Separar `sym == 'J'` pede `value.attribute` + limiar + `motion.falloff` + `cull` + `duplicator` + `source.object` + `combine`, **e saber que `J` é 74 em ASCII** |

⇒ O report nº 2 é o caso canónico de ***exprimível não é alcançável*** (a mesma frase que abriu o
`value.number`, citada no [doc 92](92_o_que_o_mini_cavalry_tem_e_nos_nao.md)).

---

## §2 — O que as referências fazem: **a resposta é unânime**

> **Um ramo não é uma sequência de segmentos desenhados. Um ramo é uma CURVA (o eixo) com uma
> FUNÇÃO DE RAIO ao longo dela, e a geometria é esse perfil VARRIDO pelo eixo — uma superfície
> contínua.** Quatro referências independentes, quatro vocabulários, uma só lei.

| referência | o esqueleto | a superfície | a lei do raio | a JUNÇÃO |
|---|---|---|---|---|
| **cpfg** — o interpretador dos autores do ABOP, que é a **nossa** referência | pontos de controlo postos pela tartaruga: `@Gs` (primeiro) · `@Gc(n)` (do meio) · `@Ge(n)` (último). O eixo é uma **curva de Hermite**, e a **tangente de cada ponto é o heading da tartaruga** | *generalized cylinder* | **`@Gr(ângulo1, comprimento1, ângulo2, comprimento2)`** — o perfil de raio entre dois pontos de controlo, dado pela inclinação e comprimento das tangentes ⇒ afinamento **não-linear** | `[` guarda *"o último ponto de controlo antes do ramo"*, e `]` repõe-no: **o filho LIGA-SE a esse ponto** |
| **Houdini** *L-System SOP* | emite **polilinha** (skeleton) com atributo de largura (`width` / `pscale`) | **um nó SEPARADO**: *PolyWire* (rápido) ou *Sweep* (com curva de secção) | o atributo por-ponto | herdada do esqueleto |
| **SpeedTree** *Spine Generator* | **spine** = spline, com segmentação **adaptativa pela curvatura** | secção radial varrida; **tampas** nas pontas abertas | *radius profile*, com o número de segmentos da secção configurável | ⭐ **o raio do filho nunca excede o do pai no ponto onde nasceu** (restrição explícita) |
| **Blender** *Sapling Tree Gen* | curvas de Bézier | **bevel** (a secção) + **taper** | `Branch Radius Ratio` · `Split Radius Ratio` · `Auto Taper` · `Minimum Radius`, **por nível** | a razão de raio por nível |

### ⭐⭐ As duas coisas que TODAS têm e nós não

1. **O VARRIMENTO é uma etapa própria.** Nenhuma delas desenha no interpretador. Houdini é o
   caso mais explícito: o L-System SOP devolve *linhas*, e quem faz o tubo é outro nó.
   *O interpretador produz o esqueleto; a superfície é uma segunda pergunta.*
2. **Uma LEI DE RAIO NA JUNÇÃO.** SpeedTree crava-a (o filho nunca mais grosso que o pai ali),
   Blender dá-lhe um knob por nível, cpfg fá-la contínua por construção (o filho começa no
   ponto de controlo do pai). Sem ela, um galho pode nascer mais grosso que o tronco que o
   segura — que é meio caminho para *"retângulos soltos"* mesmo com varrimento.

### ⚠️ E a resposta ao report nº 2 é a MESMA em todas: **a letra escolhe o objeto**

O *L-System SOP* do Houdini tem **três entradas de geometria** — e elas chamam-se, literalmente,
**`J`, `K` e `M`**: *"any geometry connected to these inputs is created in the sequence by these
letters"*. O exemplo da documentação é exactamente o do Enio: uma regra de tronco que chama um
`J` para a folha.

⭐⭐⭐ **É o nosso alfabeto, letra por letra** — o `J K M` já está implementado aqui, com o mesmo
significado, e vindo da mesma fonte (o ABOP). *A metade que falta é só a porta que diz **qual**
objecto cada letra pousa.*

⚠️ **E «por fase de crescimento» resolve-se pela GRAMÁTICA, não por uma tabela de fases.** É por
isso que nenhuma das quatro tem um controlo *«objeto da fase N»*: a regra que dispara na geração
`N` é que emite a letra, e a letra escolhe o objecto. O Houdini expõe a geração como `@gens`
para quem quiser filtrar — e **nós já emitimos `gen` por elemento**. Uma folha *«só na geração
mais nova»* é um filtro sobre `gen`; uma folha *«onde a regra mandou»* é a letra. As duas
perguntas têm resposta, e são perguntas diferentes.

---

## §3 — Os dois buracos, nomeados

### Buraco A — falta o VARRIMENTO (report nº 1)

O esqueleto está certo e é o mesmo das referências. Falta a etapa que o torna superfície.

⚠️ **É o MESMO buraco que o [doc 92](92_o_que_o_mini_cavalry_tem_e_nos_nao.md) já tinha achado
pelo outro lado** — o item 8 da §2, `skeletonRender`: *"sem ele um esqueleto é invisível, o que
torna os cinco `rig.*` difíceis de autorar"*. O L-System e o rig partilham o contrato de colunas,
logo **partilham a cura**: um varrimento que leia `parent`/`P`/`size` serve os dois.

⭐ **Em 2D o varrimento é MUITO mais barato que em 3D:** uma fita afinada é um **traço com perfil
de largura**, e o módulo vetorial já tem *live width profile*
([ADR-0148](../architecture/decisions/0148-vector-live-width-profile-is-an-ecs-component-and-one-baker-serves-preview-and-apply.md))
e o motor de traço com junções e pontas.

⏳ **O que tem de ser MEDIDO antes de desenhar** (⛔ nenhuma destas se escolhe):
1. **Onde vive a fita** — um `VecPath` (ganha junções, pontas e o perfil de largura de graça,
   mas atravessa a fronteira Motion↔Vector) ou uma tira de triângulos no próprio stream (barato
   e residente, mas as junções passam a ser nossas). *A resposta muda o tamanho da wave.*
2. **Quantos ramos** uma planta típica tem (a cadeia `parent` parte-se em quantos caminhos?) — é
   o número que decide se uma fita por ramo é barata.
3. **A lei do raio na junção**: adoptar a de SpeedTree (o filho nunca excede o pai ali) muda o
   desenho de plantas já autoradas? Se mudar, é decisão de produto.
4. **A curvatura**: cpfg interpola o eixo com **Hermite** e o heading da tartaruga como tangente
   — um tronco fica CURVO em vez de poligonal. Medir o custo disso contra a polilinha crua.

### Buraco B — falta dizer QUAL objecto cada letra pousa (report nº 2)

⭐ **A forma da cura já está escolhida pela referência**, e a nossa casa tem o idioma:
o `fx.glow` guarda o **NOME** de um objecto da cena num text param e é a **shell** que o resolve
(a mesma porta do `motion.path`), e o painel já sabe desenhar essa linha — a `SourceRow`, com
chips dos nomes que a app publicou.

⇒ Três text params (*Leaf J* · *Leaf K* · *Leaf M*), cada um nomeando um objecto.
⛔ **Não** portas de entrada: `inputs: &[]` hoje, e o Houdini usa portas porque é um grafo de
geometria; aqui o nome já é a porta de toda a casa, e ela **é alcançável** (os chips existem).

⏳ **O que tem de ser MEDIDO antes de desenhar:**
1. **Quem escreve a aparência por elemento.** O `source.object` emite as colunas de aparência
   (`texture_id`, `uv_rect`, `size`, `tint`) para UM elemento. Aqui é preciso que **elementos
   diferentes do mesmo stream** tenham aparências diferentes — medir se o lowering suporta isso
   ou se hoje a aparência é uniforme por stream. *É esta medição que decide se o buraco B é uma
   wave pequena ou uma mudança de substrato.*
2. Se não suportar, a saída conhecida é a que a composição já faz: **separar por letra e voltar a
   juntar** (`cull` → `duplicator` → `combine`), com o nó a fazê-lo por dentro em vez de o
   artista o montar. Medir o custo dessa separação numa planta cheia.

---

## §4 — O que NÃO fazer

- ⛔ **Não desenhar a fita dentro do interpretador.** As quatro referências separam esqueleto de
  superfície, e a nossa também tem de o fazer: o mesmo varrimento serve o `rig.*`, e um
  interpretador que desenha não é reutilizável.
- ⛔ **Não inventar uma tabela «objeto por fase».** Nenhuma referência tem uma, e a razão é
  estrutural: a fase é a gramática, e o objecto é a letra. Uma tabela paralela daria duas
  respostas à mesma pergunta e a segunda envelheceria.
- ⛔ **Não resolver o nº 2 por composição de sete nós.** É exprimível hoje e ninguém lá chega —
  incluindo saber que `J` vale 74.
- ⛔ **Não escolher a lei do raio na junção.** Ela muda plantas já autoradas; é medição + decisão
  do dono.

---

## §5 — O smoke de 30/08: *"só apareceu em seu exemplo, ao trocar o tipo de árvore não aparece mais"*

A letra plantava o objecto e o campo funcionava — mas **só o molde `Sprig` emitia `J`**, e a
cena `=108` tinha-o na samambaia dela. Trocar o molde no selector trocava a gramática inteira,
levando a âncora com ela: o campo *Leaf (J)* ficava cheio e nada nascia.

⇒ **duas metades, e as duas são precisas.**

### 5.1 — Os moldes de PLANTA passam a trazer a âncora

`Tree`, `Fern` e `Wild`. Medições que decidiram a forma:

| pergunta | número |
|---|---|
| a âncora muda o desenho? | **não** — `16 → 46` elementos em `Segments` e **`0` posições novas** (ela nasce na posição e largura do pai) |
| o que ela custa | ~**3×** a contagem (`32 → 94` a `g = 5`; `256 → 766` a `g = 8`) — um `J` não é reescrito, logo acumula |
| a forma que NÃO acumula (`A(s) : s <= k -> F(s)J`) | **muda a planta**: termina a recursão no limiar e a `g = 8` dá `64` elementos em vez de `256`. *Não é a mesma planta com folhas; é outra planta* |

⚠️ **O `DEFAULT_RULES` fica intocado** — ele é o oráculo do modo guiado (há gate a comparar os
dois ao bit) e o default de fábrica; o molde `Tree` ganhou texto próprio.

⛔ **Os refinadores (`Bush`, `Weed`) e as curvas (`Koch`, `Dragon`) ficam de fora** — decisão de
produto: num refinador toda a silhueta renasce a cada geração e os `J` acumulados espalhavam
folhas pelo tronco em vez das pontas; numa curva não há ponta.

### 5.2 — E o painel DIZ quando a letra não existe

A metade geral, para a gramática que o artista escreve: com um nome posto e nenhuma âncora
daquela letra, sai uma linha a nomear **a cura** (*acrescente um `[J]` no fim de um ramo*), uma
vez por `(conteúdo da gramática, slot)`. *Um controlo com valor lá dentro e efeito nenhum
parece ligado — é a pior espécie de morto.*

⚠️ **A lei vive numa função pura** (`unanswered_slots`) porque a outra metade escreve no
`stderr`, e um gate não lê o `stderr`: o que se gateia é a DECISÃO, não o canal.

### 5.3 — ⛔⛔ E a wave revelou um defeito que já lá estava: a pergunta *«o que é um osso?»* era LARGA

A família de crescimento (`grows_by_refining`) pergunta *«sobrou algum módulo VELHO que
desenha?»* e lia `F | G | f | g | J | K | M`. Uma marca de instância **não desenha** — e como
nunca é reescrita, **acumula** ⇒ pendurar uma folha num molde de refinamento reclassificava-o
como da ponta, trocando-lhe a lei de comprimento **e** a remapagem do `Growth`.

*A planta era nova, as folhas eram velhas, e a velhice das folhas decidia por ela.*

A pergunta estreitou-se para `F | G` (`turtle::draws`), com gate
(`hanging_a_leaf_on_a_grammar_does_not_change_its_growth_family`, que mede **as duas**
famílias: uma cura que respondesse sempre «da ponta» passaria com metade do oráculo) e prova
de mutação. ⭐ E o clippy fechou o ciclo: `draws_or_marks` ficou **sem nenhum leitor** ⇒ era
lixo, apagado.

---

## §6 — O 2.º smoke de 30/08: três queixas, três causas distintas

> *"as folhas não crescem, elas aparecem e sem rotação [relativa] ao galho. Elas não nascem e
> crescem na ponta dos galhos, elas aparecem em cada segmento. O Alpha usado escurece as bordas
> da pintura (diferente da sprite)."*

⚠️ **Nenhuma das três era a que eu teria adivinhado**, e as três saíram de uma sonda só
(`P`, `rot`, `size`, `gen` de cada âncora, ao longo de `g = 4 → 5`).

### 6.1 — A rotação: uma coluna com o NOME errado

A membrana publicava o ângulo numa coluna chamada **`rotation`**. A convenção de instâncias do
Motion (`ph2d-eval-motion`) chama-lhe **`rot`**, em GRAUS.

⛔ *Um nome de coluna errado não dá erro nenhum:* a coluna é ignorada, o default é a identidade,
e a folha desenha-se a direito. ⇒ **o gate tem de perguntar ao CONSUMIDOR** — ele baixa a
corrente com `lower_to_instances_onto` e mede a `basis` da instância. Uma leitura da coluna
publicada passaria com o bug lá dentro, e passou durante um bloco inteiro.

⚠️ **E a fonte mudou de `wrot` para `rot`**, que é a coluna que honra o param `Orient` do
artista e a que o modo `Segments` publica. ⛔ **A escolha só é observável em `Orient = Local`**
— no default as duas colunas trazem o mesmo número para uma marca, e a mutação SOBREVIVE; o
gate `the_orient_param_reaches_the_leaf` é o que a torna load-bearing.

### 6.2 — «não crescem» e «em cada segmento» eram DUAS coisas, e a 1.ª cura errou o preço

A 1.ª leitura foi *«é uma grandeza em falta»*: uma marca nunca é reescrita, logo **acumula**, e
desenhá-las todas do mesmo tamanho seria «uma folha em cada segmento». A cura foi um
**cruza-fade** — peso `f` à geração mais nova, `1 − f` à anterior — para que uma planta parada
mostrasse só as pontas.

⛔⛔ **O smoke seguinte matou-a em duas palavras** (Enio, mesmo dia): *"a cada segmento a folha
cresce e diminui. bem bizarro"*. E ele tem razão: com aquela lei **cada folha é um pulso** —
nasce, cresce, e encolhe até sumir quando o ramo seguinte brota dela.

⚠️ **As duas coisas não cabem na mesma lei:** *«só as pontas»* e *«uma folha não encolhe»* são
incompatíveis, porque **uma ponta VIRA interior quando a planta cresce**.

⇒ a lei passa a ser a **IDADE, monótona**: a colheita nova abre com a fracção da geração, e
toda a mais velha fica **cheia** (`turtle::mark_grow`). Ela sobe e nunca desce.

| `Generations` | o que se desenha (molde `Tree`) |
|---|---|
| `4,0` | 15 folhas, todas cheias |
| `4,5` | as 15 velhas cheias + 16 novas a meio |
| `5,0` | 31 folhas, todas cheias |

⭐⭐ **E a outra metade da queixa era do MOLDE, não da lei** — medida, e eu não a tinha medido:
a gramática de fábrica era `A(s) -> F(s)![+A(s*0.7)J][-A(s*0.7)J]`, com o `J` **depois** da
sub-árvore inteira. Ao sair dela a tartaruga está de volta ao fim do `F`, onde as marcas de
todas as gerações que a envolvem também caem ⇒ **62 marcas em 30 sítios** (`2,07×`), folhas
idênticas empilhadas.

⇒ o `J` passa a vir **logo a seguir ao segmento** (`F(s)[J]!…`): `31` marcas em `31` sítios,
`1,00×`. ⚠️ **Uma contagem de marcas não vê um empilhamento** — o que o vê é contar os
**SÍTIOS**, e nenhuma régua desta linha o fazia.

⚠️ **ONDE a folha vive é da GRAMÁTICA**, que é o desenho do nó desde que ele existe. Um `J` a
seguir ao segmento dá uma folha por segmento (os moldes desta casa); um `J -> ` (sucessor
vazio) apaga as velhas e dá **só as pontas** — ao preço de elas sumirem de repente, que é a
mesma dor de cima. O `Segments` continua a publicar o esqueleto **CRU** (contrato do `rig.*`).

### 6.3 — A alfa: o lowering do Motion cravava `premultiplied: 0.0`

Para **toda** instância que o Motion emite — não só a folha. Um documento pintado sobe **já
premultiplicado** (há assert a dizê-lo em `project_painter.rs`), e o fragmento pré-multiplicava
outra vez ⇒ `RGB·α²`: invisível no interior opaco, **escuro na borda anti-aliased**.

A bandeira passa a viajar da `Sprite` até à instância (`appearance_tile` ganhou o parâmetro,
coluna `premultiplied`, ausente ⇒ `0` ⇒ byte-idêntico). ⏳ **Os dois bakes de Flip ficam a
`false` como hoje, e a pergunta fica NOMEADA**: o `FlipTile` não carrega bandeira de alfa, e
adivinhar mudaria os pixels de todo objecto Flip com base num palpite.

---

## ⛔ Recusas MEDIDAS (deste estudo)

| Item | Motivo |
|---|---|
| Emitir a fita como quads no próprio interpretador | Separa-se esqueleto de superfície nas quatro referências; e o varrimento serve também os cinco `rig.*` (doc 92 §2 item 8) |
| Portas de geometria `J`/`K`/`M` à la Houdini | O nó tem `inputs: &[]` e a casa nomeia objectos por **nome** (`fx.glow`, `motion.path`), com chips já alcançáveis no painel. Portas seriam um segundo idioma |
| Tabela de «objeto por fase de crescimento» | A fase é a regra que dispara; a letra é o objecto. Nenhuma das quatro referências tem tabela de fase — e nós já emitimos `gen` para quem quiser filtrar |
| Âncora de folha nos moldes que REFINAM (`Bush`, `Weed`) e nas curvas (`Koch`, `Dragon`) | Um `J` acumula e num refinador toda a silhueta renasce ⇒ folhas pelo tronco, não nas pontas; numa curva não há ponta. O sítio de a pedir é a gramática do artista, e o painel **diz** quando o nome está posto e a letra falta |
| Âncora no `DEFAULT_RULES` | Ele é o oráculo do modo guiado (gate compara os dois ao bit) e o default de fábrica — obrigaria a pôr a âncora na derivação guiada e a pagar ~3× a contagem em toda planta que nunca terá folha |
| Terminar a recursão para não acumular (`A(s) : s <= k -> F(s)J`) | Medido: **muda a planta** — `64` elementos em vez de `256` a `g = 8`. Não é a mesma planta com folhas |
| O cruza-fade entre duas gerações (peso `f` / `1 − f`) | Comprava «só as pontas» numa planta parada e fazia **cada folha encolher até sumir** durante o crescimento — *«só as pontas» e «uma folha não encolhe» não cabem na mesma lei, porque uma ponta vira interior*. Veredito do Enio: *«bem bizarro»* |
| Filtrar `sym` com `motion.cull` | Ele só faz *Fraction* e *Falloff*; a rota por atributo pede 6-7 nós e o código ASCII da letra |

---

## Fontes

- [L-System geometry node — SideFX Houdini docs](https://www.sidefx.com/docs/houdini/nodes/sop/lsystem.html)
- [PolyWire geometry node — SideFX](https://www.sidefx.com/docs/houdini/nodes/sop/polywire.html)
- [Generalized Cylinders — cpfg 3.0 tutorial, Algorithmic Botany](https://algorithmicbotany.org/cpfg3.0-tutorial/cylinders.html)
- [Spine generator — SpeedTree documentation](https://docs.speedtree.com/doku.php?id=spine_generator)
- [Sapling Tree Gen — Blender manual](https://docs.blender.org/manual/en/4.1/addons/add_curve/sapling.html)
- [How to create L-Systems — Houdini Kitchen](https://www.houdinikitchen.net/2019/12/21/how-to-create-l-systems/)
