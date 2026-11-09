# Os cabeçalhos das cenas de smoke da física (verbatim)

Movidos dos `//!` de `crates/ph2d-app-physics/src/` (09/11, tecto de LOC da `ph2d-app-physics`: a cura é cortar,
o porquê vive no doc). Cada bloco é o texto original, sem os `//! `; o sha256 é o do bloco original.

## `physics_smoke_stone.rs`

sha256 `6897f8715db520f8763cf3f2164cfc04d685133a2f177305e530a84e9ee0345c`

**A cena 107 — A PEDRA ESTREITA** (`W-ShapeCast`), irmã de
`physics_smoke_player_crouch.rs`.

Uma fileira de pedras de **8 cm** penduradas à altura da cabeça, e uma laje
larga no fim como CONTROLE. O gesto que a cena existe para julgar é
**soltar o agachar debaixo de uma pedra estreita** — e a resposta tem de ser
a mesma que debaixo da laje: ele fica agachado.

# ⚠️ Por que ESTREITA, e por que várias

O sensor do agachar lia o teto com **três raios** nascidos em
`−0,20 · 0,00 · +0,20` do centro do corpo — a caixa envolvente mede
exactamente 0,20 de meia-largura. Uma pedra de 8 cm cabe **inteira** no vão
entre duas dessas amostras, e o preço estava medido: a cabeça subia a
**1,267** contra uma face de pedra em **1,25**, com o solver a segurá-la lá
dentro.

São várias e em posições diferentes porque a grade acompanhava o CORPO: o vão
cego movia-se com o personagem, então uma pedra só teria uma janela de `x` em
que o defeito aparecia. Com a varredura de corpo **nenhuma** delas deixa
passar, e é isso que a fileira torna visível de uma vez.

# ⚠️ As alturas são ARITMÉTICA da cápsula

| pose | centro | topo |
|---|---|---|
| de pé | 0,90 | **1,40** |
| agachado | 0,55 | **1,05** |

A face de baixo das pedras fica em **1,20** — entre os dois, os mesmos
0,15 m de folga para cada lado que o corredor da cena 94 usa. Mais alta e ele
passaria de pé (a cena mostraria uma capacidade que não é esta); mais baixa e
nem agachado caberia.

# ⚠️ O que esta cena NÃO mostra, e porquê

A varredura devolveu uma segunda coisa: a **quina da caixa** deixou de ser um
teto (a cápsula ali alcança menos que a caixa dela), o que vale **18 cm** de
altura de teto medidos. Isso **não é julgável de olho** — não há
contrafactual na tela, o artista veria apenas um personagem a levantar-se
normalmente. Fica **gateado** (`the_capsule_corner_is_no_longer_a_ceiling`) em
vez de encenado, porque uma estação que não se pode julgar é uma que se
aprova por cansaço.

## `physics_smoke_ledge.rs`

sha256 `807ab24474e5dfb977bca72e813040b09fe1b9af89ac00bd466c7753253b2a45`

**A cena 111 — O PARAPEITO** (`W-Ledge`), a beirada.

Duas raias IDÊNTICAS, dois personagens IDÊNTICOS, e a única diferença entre
eles é um número: o alcance do braço. O da esquerda tem **zero** (o mundo de
antes desta wave), o da direita tem **0,60 m**.

# ⚠️ O teclado dirige os DOIS ao mesmo tempo, e é isso que faz a cena

`hand_input_to_players` entrega a entrada a **todo** `PlatformPlayer` da
cena, então um único gesto do artista move os dois lado a lado — o controle
está **dentro do quadro**, e não numa segunda corrida.

# ⚠️ As alturas são MEDIDAS, e a do meio é a que faz a cena

`measure_the_reachable_height` (a sonda da wave anterior), aperto SEGURADO:
um pulo alcança **1,903 m**. Daí:

| patamar | topo | quem chega |
|---|---|---|
| o BAIXO | **1,0 m** | os dois, a pular — é o controlo |
| o ALTO | **2,4 m** | ninguém, a pular; só quem agarra a beirada |

⚠️ **E o número que de facto decide a cena é OUTRO, medido aqui:** pular
**colado à parede** — que é o gesto do passo 3 — alcança **0,745 m**, contra
os 1,903 do ar livre. O atrito contra a face come **61%** da subida
(`measure_what_a_jump_against_the_wall_reaches`), e o topo do corpo pica em
**2,145 m**. A primeira versão desta cena pôs o patamar alto em 2,60 usando o
número do ar livre, e o corpo **nunca chegava à janela**.

Com o lábio em 2,40 e o braço em 0,60 a janela é `[1,80 · 2,40)`, e o corpo
atravessa-a de 1,80 a 2,145 — perto do ápice, onde ele é lento. E 2,40 fica
acima de **2,303**, que é onde os PÉS chegam num pulo de ar livre perfeito:
ninguém pousa nele sem agarrar.

# ⚠️ E os dois patamares têm PAREDE, porque uma beirada é uma quina

Um bloco que nasce do chão dá as duas metades de uma vez: a face vertical por
onde se sobe e o topo em que se pousa. Um patamar a pairar teria um lábio de
cada lado e o gesto deixaria de ser o que a cena quer mostrar.

## `physics_smoke_player_flank.rs`

sha256 `d92cfcf6e3403d7f84efaae0ed343841feeb58d0e8ba0a53ba6d3c12589b695c`

**A cena 98 — A PAREDE COM JANELAS** (W13, o flanco), irmã de
`physics_smoke_player_wall.rs`.

O poço da cena 92 provou o ziguezague contra paredes **lisas**. Esta troca uma
delas por uma parede com **janelas** — e é exatamente essa geometria que o
sensor lateral não enxergava.

# ⚠️ A janela mede 0,8 m, e o número não é decorativo

O corpo do personagem mede **1,0 m** de caixa envolvente (cápsula
`half_height 0,3` + `radius 0,2`). Uma janela de 0,8 m deixa **10 cm de pé E
10 cm de ombro** encostados na pedra — o jogador vê um personagem colado à
parede. O sensor lia só a **cintura**, que é justamente o que atravessa o
vazio, então a lei respondia *"não há parede aqui"*.

**Medido** (`ph2d-physics-ecs/tests/it/measure_wall_flank.rs`), com a janela na
altura da cintura:

| janela | pulo de parede |
|---|---|
| parede lisa | **2,162 m** |
| 0,60 m | 1,997 m (o *buffer* do pulo mascarava) |
| 0,70 m | 2,292 m (idem) |
| **0,75 m** | **0,000 m** — recusado por inteiro |
| **0,80 m** | **0,000 m** |
| 0,90 m | 0,000 m |

⚠️ **Abaixo de ~0,70 m o defeito era INVISÍVEL**, e não por acaso: o buffer do
pulo guarda o aperto até o bloco de baixo reaparecer. Uma cena com janelas
pequenas mostraria tudo a funcionar sobre um sensor cego — por isso as desta
aqui têm 0,8.

# ⚠️ E o ESCORREGAMENTO quase não denuncia — o oráculo é o PULO

Passar pela janela custa uns centímetros de descida a mais (0,0500 →
0,0632 m/tique medidos), porque a *cola* que o `platform_wall` documenta
segura o personagem de qualquer jeito. O pulo não tem cola: ou a lei vê
parede naquele tique, ou o botão não faz nada.
