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

## §7 — O 3.º smoke de 30/08: a folha ganha CONTROLOS, e um report era do artista

> *"ainda nascem folhas no fim de cada segmento mesmo se o segmento é a raiz ou o caule. Outra
> coisa: não temos a opção de escolher quantas folhas são desenhadas na frente ou atrás dos
> galhos, nem as rotações das folhas."* · *"uma opção para livrar as folhas, os frutos do tint
> que pinta tudo na árvore"* · *"LFO não funciona animando Tropism Angle"*

### 7.1 — Cinco controlos na secção *Leaves*

| controlo | o que faz | default |
|---|---|---|
| **First Level** | o 1.º nível de ramo que ganha folha | **3** |
| **Leaf Angle** | soma-se à direcção do ramo | `0` (a direcção do ramo, **ao bit**) |
| **Leaf Spread** | abre as folhas umas em relação às outras (sorteio determinístico) | `0` |
| **Leaves In Front** | a fracção desenhada à frente dos galhos | `0` |
| **Effects Reach Leaves** | `Keep Own Colour` · `Reached` | **Keep Own Colour** |

⭐ **O `3` não é escolhido, é medido:** as marcas da árvore de fábrica vivem nas profundidades
`1..5` com contagens `1 · 2 · 4 · 8 · 16`, e as duas setas da foto apontam para os níveis `1`
(a raiz) e `2` (a primeira forquilha). Começar em `3` deixa `28` folhas de `31` e nenhuma no caule.

⚠️ **Contado da RAIZ, e não da ponta:** a ponta MOVE-SE quando o `Generations` sobe, então *«as
últimas N camadas»* mudaria de sujeito a cada geração; o tronco é o nível `1` para sempre.

### 7.2 — «À frente» é a ORDEM DAS LINHAS, e só uma FORMA lá chega

⛔⛔ A casa desenha **os sprites antes do vector** (declarado em `render_loop/mod.rs`: *«Fase 1:
vector over sprite»*), então uma folha que é uma **imagem** fica sempre atrás dos galhos e
nenhuma ordem de linhas a move. Uma folha que é uma **forma desenhada** vive na mesma passagem
que a planta, e ali quem manda é a ordem: as de trás antes da linha da planta, as da frente
depois.

⭐ **Aceitar uma forma como folha era um buraco por si** — o `named_appearance` exigia `uv_rect`,
que só uma sprite publica, então nomear uma forma do documento não plantava nada e não dizia
porquê. ⚠️ E o `Leaves In Front` **diz**, uma vez por planta, quando o objecto nomeado é uma
imagem: sem isso ele seria um knob morto no caso comum.

### 7.3 — A folha fora do tint: a máscara que a casa já fala

O `motion.tint` faz `lerp(existente, alvo, falloff)` ⇒ **`falloff = 0` mantém a cor**. A membrana
publica `0` nas linhas de folha e `1` na planta. ⚠️ Com `Reached` a coluna **não nasce** — uma
coluna de uns apagaria um `falloff` que um nó a montante tivesse escrito.

### 7.4 — ⛔ O LFO no *Tropism Angle*: a maquinaria está ILIBADA, com número

Medido: com `Tropism = 30` um `value.lfo` no `Tropism Angle` **move a planta** (altura
`0,541 → 0,528 → 0,578` em três instantes); com `Tropism = 0` não move nada. Duas causas, as
duas do lado do artista e nenhuma visível na tela:

1. **`Tropism` nasce em `0`**, e o ângulo é a DIRECÇÃO de uma força de intensidade zero;
2. **o `value.lfo` nasce com `amplitude = 1`**, e o param é em GRAUS: **±1°**.

⚠️ **Um `ParamGate` não exprime isto** (ele compara com uma lista de INTEIROS, e a condição é
*«diferente de zero»* num slider contínuo), e esconder a linha seria pior — ela desapareceria no
estado de fábrica, que é onde ele estava. ⇒ o app **diz**, e só quando há FIO: um `Tropism Angle`
parado no default é o estado de toda planta, e avisar sobre ele seria ruído por quadro.

⚠️ **E três sondas minhas mediram a coisa errada antes de eu chegar aqui:** a coluna `P` da
corrente publicada tem a origem e as folhas — **a planta vive na geometria** —, o `key_of` do
arnês resolve sempre em `t = 0`, e o primeiro condutor que liguei (`motion.oscillator`) não
produzia número nenhum. *Uma sonda que não move o número não prova que o produto não move.*

---

## §8 — O 4.º smoke: *"as folhas não aparecem"* — e o `falloff` era o canal ERRADO

> *"Keep own color não funciona, as folhas não aparecem"* · *"Leaves in front não funciona, nada
> muda"*

⛔⛔ **A minha cura do tint estava errada de ESPÉCIE.** Eu tinha escrito `falloff = 0` nas linhas
de folha porque o `motion.tint` faz `lerp(existente, alvo, falloff)`. Mas o `falloff` é a máscara
de **todos** os modificadores desta casa — e o `motion.move` faz `P' = P + (dx, dy) · falloff`.

⇒ na cena `=108`, que move cada coluna com um `motion.move`, **as folhas ficavam paradas
enquanto a árvore andava**. Elas não desapareciam: ficavam noutro sítio, longe da planta a que
pertenciam. *O canal que escolhi era muito mais largo do que a pergunta que fiz.*

⭐ A cura é uma coluna **própria** — `attr::TINT_MASK_COLUMN`, multiplicativa com o `falloff` no
`motion.tint`, ausente ⇒ `1` ⇒ byte-idêntica. É o molde do `falloff_y` (que só o
`motion.falloff(Mask Channel)` escreve e só o `motion.scale(Separate Y Mask)` lê), mas declarada
**numa porta só**: dois lados a escrever a mesma string à mão são duas leis à espera de divergir.

⚠️⚠️ **E o gate que eu tinha NÃO O VIU, porque media a COLUNA e não a CONSEQUÊNCIA.** Ele
afirmava *«a máscara nasce com `0` nas folhas»* — o que era verdade — e nada sobre o que um nó a
jusante faz com ela. O gate novo coze um **`motion.move` a sério** e mede que as folhas andam
com a planta; a mutação que repõe o `falloff` mata-o.

### 8.1 — E o `First Level` esvaziava um molde inteiro

Medido: as `10` marcas do `Sprig` estão **todas na profundidade `1`** — ali o `J` vive num ramo
lateral de primeiro nível (`[+F(s*0.35)J]`) enquanto no `Tree` ele vive no eixo. Com o default
único de `3`, aquele molde ficava com **zero** folhas.

⇒ **o molde carrega o SEU número** (`Preset::leaf_first_level`), como já carregava ângulo,
gerações, passo e espessura. ⚠️ *A profundidade de encaixe significa coisas diferentes em
gramáticas diferentes, então um número só não a atravessa.*

⛔⛔ **E a mutação que repunha o `3` no `Sprig` SOBREVIVEU a toda a suíte** — não havia nada a
dizer que um molde com marcas tem de mostrar pelo menos uma. É o gate
`no_preset_silences_its_own_leaves`, com o controlo que impede o laço de passar vazio.

### 8.2 — O silêncio era metade da causa dos DOIS reports

Uma gramática cujas marcas vivem todas abaixo do nível mínimo fica sem folha nenhuma, e nada na
tela o diz — e sem folhas, o `Leaves In Front` também *"não muda nada"*. ⇒ o app **diz**
(`say_if_the_level_hid_every_leaf`), e cala nos três casos vizinhos para não virar ruído por
quadro.

---

## §9 — *"Leaves in front não funciona"*: o padrão-ouro, e o que o BLOQUEIA (re-medido)

Enio, 2026-08-30: *"Leaves in front não funciona"* e, depois de eu lhe dar as três saídas com o
preço: *"faça o que for o padrão ouro, o estado da arte"*.

### 9.1 — A implementação está certa; a CENA é que não a alcança

O `draw_shared_instances` desenha as instâncias vectoriais **por ordem de linha** (o cache dele
é por `geometry_id`, não uma reordenação), e a membrana põe as folhas «à frente» depois da linha
da planta. ⇒ com uma folha que é uma **forma desenhada**, o knob funciona, e há gate.

⛔ **Mas a cena `=108` não tem forma desenhada nenhuma** — ali só há sprites, e para um sprite
isto é impossível (ver abaixo). *Construí uma feature que o smoke onde ele testa não pode
exercitar*, e o único sinal disso é uma linha no terminal.

### 9.2 — ⛔⛔ Por que um SPRITE nunca fica à frente: re-medido na versão de HOJE

A ordem de composição de um quadro é:

| passe | alvo |
|---|---|
| 1 — sprites | `game_rt`, **`Rgba16Float` HDR** |
| 2 — tonemap | LDR |
| 3 — Vello (chrome **+ o vector do documento**) | intermediário `Rgba8Unorm`, α=0 |
| 4 — compositor | os dois, para a swap chain |

⇒ **tudo o que é vector fica por cima de tudo o que é sprite**, por construção.

⚠️ **A nota que dizia isto citava o `vello` 0.8, e o stack subiu para o 0.10 em 2026-08-29** —
o §0.0 exige reconferir uma impossibilidade quando alguém mexe no número que a sustentava.
Reconferido no fonte da versão instalada: `Renderer::render_to_texture` continua a exigir
`TextureFormat::Rgba8Unorm` + `STORAGE_BINDING`/`COPY_SRC`. **O alvo HDR continua fora do
alcance dele.** A separação não é uma escolha nossa: é a biblioteca.

### 9.3 — O padrão-ouro é FOLHA COMO CARTA, e ele tem endereço

O que SpeedTree, o *Sapling* do Blender e todo gerador de vegetação sério fazem: **uma folha é
geometria com textura** (um *card*), não um sprite separado — e é exactamente por isso que ela
se intercala com os ramos sem uma segunda passagem.

Aqui isso quer dizer: a folha vira um quad `VecPath` com `Paint::Pattern` +
`PatternSource::Image(AssetId)` (as duas peças **já existem**, do plano 33 da `line/Vector`),
internado no mesmo store da planta e desenhado na mesma passagem. ⇒ o `Leaves In Front` passa a
valer para **qualquer** folha, com um só aspecto.

⛔ **E é uma wave com espec própria, não um remate.** Medido em 2026-08-30, depois de o dono
pedir *"faça o que for o padrão ouro"* — as DUAS rotas param na MESMA peça em falta:

| rota | o que falta |
|---|---|
| **carta com padrão** (`Paint::Pattern` + `PatternSource::Image`) | (a) o `AssetId` da folha na membrana; (b) o passe vectorial do Motion **não tem o mapa de ladrilhos do quadro** — e isso é uma cerca DECLARADA e GATEADA por outra linha (`a_motion_instance_of_a_patterned_shape_paints_the_fallback`, `ph2d-vec-render/src/instance.rs`) |
| **imagem na cena Vello** (`draw_image_rgba_premultiplied_transformed`) | os **pixels em CPU** da folha |

⇒ **a peça em falta é a mesma:** neste app um sprite é identificado por um handle de GPU
(`SpriteSource::{Atlas, Individual, Ktx2}`), e **nada leva os pixels dele de volta à CPU**, que é
onde o passe vectorial é codificado. O único mapa que existe é `AssetId → texture_id`, construído
no CARREGAMENTO (`project_sprite_pixels.rs`), não um mapa vivo ao contrário.

⇒ a wave é *«um objecto nomeado carrega a identidade de CONTEÚDO da arte dele»*, e ela mora no
oleoduto de objectos/assets — não no L-System. ⛔ **E a rota do TILE tem um segundo dono:** mexer
naquela cerca é acto de quem a escreveu.

⛔⛔ **E uma terceira saída foi considerada e REJEITADA:** assar a PLANTA numa tile e desenhá-la
no passe dos sprites (aí a ordem das linhas resolve tudo). Ela funciona e **desfoca a árvore** —
um knob que silenciosamente troca a nitidez da planta por uma folha à frente não é uma escolha
que o artista possa fazer sem ver o preço.

⛔⛔ **A saída INTERMÉDIA foi medida e REJEITADA:** desenhar só as folhas da frente como imagens
na cena Vello (o `draw_image_rgba_premultiplied_transformed` existe) poria **metade da copa
depois do tonemap e metade antes** — as duas metades da mesma folhagem com cores diferentes. Uma
cura que introduz uma inconsistência visível não é a cura.

---

## §10 — *"algumas aparecem já grandes"*: a IDENTIDADE de uma folha não é o índice dela

> Enio, 2026-08-30: *"ficou muito bom, mas nem todas as folhas crescem, algumas aparecem já
> grandes"*.

⭐ **A lei do crescimento estava certa** — medida: varrendo `g` de `1` a `6`, **zero** folhas
recebem um primeiro peso acima de `0,5`. Todas nascem pequenas.

⛔⛔ **O defeito era o SORTEIO, e a causa é a identidade.** O tamanho (e o lado frente/trás)
saíam do **índice da âncora na lista** — e ao crescer, a planta **insere marcas no MEIO** (a
travessia é em profundidade). ⇒ o índice de uma folha que já existia MUDA, ela recebe outro
número, e **salta de tamanho**. O mesmo valia para o `Leaves In Front`: as folhas trocavam de
lado sozinhas enquanto a planta crescia.

⭐ A cura é uma identidade **estável**: o par `(geração, ordinal dentro dela)`. E ele é estável
pela razão que faz a planta crescer — *as gerações velhas não se reescrevem, logo a ordem
relativa das marcas de uma geração não muda quando outra nasce*.

⚠️⚠️ **E a primeira sonda desta caça inventou o defeito ao medi-lo:** ela identificava a folha
pela POSIÇÃO e acusou **209 saltos de 420** — mas a marca da geração mais nova **move-se**
enquanto o ramo dela estica, então cada passo do varrimento contava «uma folha nova». Com a
identidade certa, `0` de `42`. *A régua que se usa para procurar um defeito pode fabricá-lo* —
e é a quarta vez nesta jornada que a régua mordeu antes do produto.

⚠️ O gate compara os retratos de `g = 4` e `g = 5` **só nas folhas cujas gerações já pararam de
crescer** (as que estão no mesmo sítio nos dois), com o controlo que impede a comparação de
passar vazia.

---

## §11 — *"vários dos presets não produzem folhas"*: a minha recusa estava errada

Enio, 2026-08-30. Eram **quatro de oito** — `Bush`, `Weed`, `Koch`, `Dragon` — por uma decisão
minha, registada neste doc como recusa medida: *«num refinador toda a silhueta renasce e as
folhas espalham-se pelo tronco; numa curva não há ponta»*.

⛔ **A medição desmentiu-a**, e o que a dissolveu foi trabalho meu de duas horas antes: o
`First Level` e a lei monótona da idade não existiam quando escrevi aquela recusa.

| molde | com o `[J]` no sítio certo |
|---|---|
| `Bush` · `Weed` | **121** marcas, profundidades `1..5`; a `First Level = 3` sobram `96` e nenhuma no caule |
| `Koch` | `156` marcas, **todas na profundidade 1** |
| `Dragon` | `512` marcas, todas na profundidade 1 |

⇒ os oito moldes passam a emitir. ⚠️ Nas curvas o `First Level` **não tem por onde
discriminar** (uma curva não tem tronco), então elas trazem `1` e mostram todas: *quem escreve
um nome numa curva quer decoração, e a resposta honesta a «não produz folhas» não é explicar
porquê — é produzir.*

### 11.1 — ⚠️ E a régua do empilhamento acusava a FIGURA, não a colocação

A curva do **Dragão toca-se a si própria** (é o que uma curva que ladrilha o plano faz): `2048`
marcas em `1324` sítios, sem nada empilhado por culpa da colocação. O gate reprovou.

⇒ a chave passa a incluir o **PAI**: o que ele acusa é o empilhamento que a COLOCAÇÃO faz —
duas marcas do mesmo pai no mesmo sítio, que foi o defeito de `F(s)![+A J][-A J]` — e não dois
ramos diferentes que a figura leva ao mesmo ponto. *Uma régua que não separa a geometria da
figura da colocação acusa a figura.* Provado: com o `J` antigo de volta, ela ainda diz
`62 marcas em 30 sítios`.

⚠️ **E não havia nada a afirmar que um molde produz folhas** — `no_preset_silences_its_own_leaves`
media só que o molde não se apagava a si próprio, e um molde SEM marcas passava por `continue`.
Hoje ele exige que os oito emitam, e o controlo conta os oito.

---

## ⛔ Recusas MEDIDAS (deste estudo)

| Item | Motivo |
|---|---|
| Emitir a fita como quads no próprio interpretador | Separa-se esqueleto de superfície nas quatro referências; e o varrimento serve também os cinco `rig.*` (doc 92 §2 item 8) |
| Portas de geometria `J`/`K`/`M` à la Houdini | O nó tem `inputs: &[]` e a casa nomeia objectos por **nome** (`fx.glow`, `motion.path`), com chips já alcançáveis no painel. Portas seriam um segundo idioma |
| Tabela de «objeto por fase de crescimento» | A fase é a regra que dispara; a letra é o objecto. Nenhuma das quatro referências tem tabela de fase — e nós já emitimos `gen` para quem quiser filtrar |
| Âncora no `DEFAULT_RULES` | Ele é o oráculo do modo guiado (gate compara os dois ao bit) e o default de fábrica — obrigaria a pôr a âncora na derivação guiada e a pagar ~3× a contagem em toda planta que nunca terá folha |
| Terminar a recursão para não acumular (`A(s) : s <= k -> F(s)J`) | Medido: **muda a planta** — `64` elementos em vez de `256` a `g = 8`. Não é a mesma planta com folhas |
| O cruza-fade entre duas gerações (peso `f` / `1 − f`) | Comprava «só as pontas» numa planta parada e fazia **cada folha encolher até sumir** durante o crescimento — *«só as pontas» e «uma folha não encolhe» não cabem na mesma lei, porque uma ponta vira interior*. Veredito do Enio: *«bem bizarro»* |
| Esconder o `Tropism Angle` quando o `Tropism` é `0` | O `ParamGate` da casa compara com uma lista de INTEIROS e a condição é *«diferente de zero»* num slider contínuo; e a linha desapareceria no estado de FÁBRICA, que é onde o artista estava quando reportou. A cura é o app DIZER, e só quando há fio |
| Usar o `falloff` para livrar as folhas do tint | ⛔ **PARTIU a planta**: o `falloff` é a máscara de TODOS os modificadores, e o `motion.move` faz `P' = P + (dx, dy) · falloff` ⇒ as folhas ficavam paradas enquanto a árvore andava. A cura é uma coluna própria (`attr::TINT_MASK_COLUMN`) |
| Um `First Level` único para todos os moldes | ⛔ **Esvaziava o `Sprig`** (10 marcas, todas na profundidade 1). O molde carrega o seu, como os outros quatro números de enquadramento |
| Folhas da frente como imagem na cena Vello | ⛔ Poria metade da copa DEPOIS do tonemap e metade antes — as duas metades da mesma folhagem com cores diferentes |
| Desenhar o vector do documento na passagem HDR das sprites | ⛔ **Re-medido no `vello` 0.10** (a nota citava o 0.8): o `render_to_texture` exige `Rgba8Unorm`; o alvo HDR está fora do alcance da biblioteca |
| Filtrar `sym` com `motion.cull` | Ele só faz *Fraction* e *Falloff*; a rota por atributo pede 6-7 nós e o código ASCII da letra |

---

## Fontes

- [L-System geometry node — SideFX Houdini docs](https://www.sidefx.com/docs/houdini/nodes/sop/lsystem.html)
- [PolyWire geometry node — SideFX](https://www.sidefx.com/docs/houdini/nodes/sop/polywire.html)
- [Generalized Cylinders — cpfg 3.0 tutorial, Algorithmic Botany](https://algorithmicbotany.org/cpfg3.0-tutorial/cylinders.html)
- [Spine generator — SpeedTree documentation](https://docs.speedtree.com/doku.php?id=spine_generator)
- [Sapling Tree Gen — Blender manual](https://docs.blender.org/manual/en/4.1/addons/add_curve/sapling.html)
- [How to create L-Systems — Houdini Kitchen](https://www.houdinikitchen.net/2019/12/21/how-to-create-l-systems/)
