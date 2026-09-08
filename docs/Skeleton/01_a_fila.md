# 01 — A FILA do módulo do esqueleto

> **O que está ABERTO, na ordem em que faz sentido pegar.** Cada item traz o **mecanismo** (ou o
> instrumento que o nomeia numa corrida) e o que as referências fazem — não uma promessa.
>
> ⚠️ **Uma nota de diferido não é uma spec.** O que torna um item desta fila pegável não é ele estar
> escrito: é ele dizer *por onde começar a medir*. Um item sem isso é trabalho a redescobrir.
>
> ⚠️ **A ordem é uma recomendação, não uma decisão.** Quem escolhe é o dono.

---

## Aberto por REPORT do dono (2026-09-07, no smoke da âncora)

> *«Undo não funciona para add IK. Múltiplos IKs numa cadeia de bones tem resultado ruim. Mas não
> precisa fazer isso agora. Coloque na fila de implementação para o melhor momento possível.»*

### F1 — ✅ *«undo … não funciona plenamente»* — **CURADO** (2026-09-07, 2.ª volta)

⭐⭐⭐ **A palavra era «plenamente», e ela nomeava o defeito:** o `Ctrl+Z` **desfazia** a criação da
âncora — isso foi medido e está certo — e **perdia a SELECÇÃO**. Com o osso desescolhido, a secção
SKELETON fica sem sujeito e `Length`, `Strength`, *Add IK* e os três números da âncora
**desaparecem do painel**. O undo faz o trabalho certo e o artista vê o app partir-se.

**A causa:** [`undo_selection::keeps_its_selection`] filtrava **uma família só** — *«só quem é nó do
MODELADOR»* — e um osso não é. ⚠️ **A nota que ficou lá previu-o e não o impediu** (*«o resto da
selecção segue as leis de quem a possui»*): nada obriga uma família nova a vir a uma lista escrita à
mão. É a **terceira** desta linha a morder pelo mesmo mecanismo (as outras: o
`publishes_its_own_handles` e a allowlist de cliques do painel).

⛔ **A generalização — «tudo o que tem identidade durável sobrevive» — NÃO foi feita:** é uma cerca
de Chesterton, e a decisão é de quem possui os outros módulos. O que se fez sem os acordar foi
**nomear** as duas famílias do esqueleto, uma linha cada.

**Medido no app:** `Ctrl+Z` passa de `sel=[]` para `sel=[<bits novos>]` — a mesma coisa, re-achada
pelo `StableId`.

<details><summary>a 1.ª volta, que ilibou a máquina do undo</summary>

### F1 (1.ª volta) — o `Add IK` não é desfazível — **NÃO REPRODUZIA**

**Sintoma** (verbatim): *«Undo não funciona para add IK»*. Carregar em *Add IK* cria o alvo e a
restrição; o `Ctrl+Z` seguinte não os leva embora.

**⚠️ A causa NÃO está medida, e este parágrafo existe para não a fabricar.** Há **cinco** motivos
pelos quais o `App::post_frame_undo` suprime um passo, e eles suprimem **igual** — o
[`undo_app.rs`](../../shells/desktop/src/undo_app.rs) nomeia-os por escrito, na ordem do
diagnóstico: *botão do rato em baixo* · *arrasto do gizmo 3D em curso* · *colorize a recalcular* ·
*transição de estado de UI ao vivo* · *sem entrada neste quadro*.

⭐ **O PRIMEIRO passo é uma corrida, não uma hipótese:**

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector \
  && env PH2D_VEC_BONE_SMOKE=1 PH2D_UNDO_LOG=1 cargo run -p ph2d-host-desktop --release
```

…carregar em *Add IK* e ler a linha `[undo] ⛔ o documento MUDOU em … e o passo foi SUPRIMIDO —
motivo: …`. Ela responde a pergunta inteira. ⛔ **Se ela NÃO aparecer**, o passo foi registado e o
defeito está no outro lado (o `restore`), que é uma investigação diferente — e a ausência dessa
linha é o que as separa.

⚠️ **Duas coisas que este report NÃO distingue, e que a medição tem de separar:**

1. **Um passo SUPRIMIDO e um passo AUSENTE leem-se iguais de fora**, e as causas são opostas — o
   doc do `post_frame_undo` diz-o à letra. Um suprimido **funde-se no passo seguinte**, então o
   sintoma pode ser *«o Ctrl+Z desfaz demais»* uma acção depois.
2. ⚠️ **Criar a âncora é um no-op VISUAL por construção** (gate
   `adding_an_anchor_moves_nothing`): o alvo nasce exactamente na ponta. ⇒ o único sinal na tela é
   o **losango** aparecer e desaparecer. Um `Ctrl+Z` que funcionasse e um que não funcionasse
   diferem só nisso, e é por isso que o log importa mais aqui do que noutro sítio.

## ⭐⭐⭐ MEDIDO, e as TRÊS metades estão ilibadas

A sonda existe: **`PH2D_BONE_UNDO_PROBE=1`**
([`bone_undo_probe.rs`](../../shells/desktop/src/bone_undo_probe.rs)). Ela percorre o caminho
**completo do artista** — carrega no pill *Bone*, **rola o painel** (a secção SKELETON fica abaixo da
dobra, e o índice de acerto é recortado), escolhe um osso sem âncora, faz `Down` num quadro e `Up`
**três quadros depois** (como uma mão humana) e manda `Ctrl+Z` — tudo pelo roteamento do `winit`.

```text
f=49 undo=1 ancoras=1        <- antes
Down em Add IK em (696, 717)
f=50..52  held=Some(Primary) <- o botao segurado por TRES quadros
f=53 Up
f=54 undo=2 ancoras=2        <- o passo NASCEU e a ancora foi criada
f=70 Ctrl+Z
f=71 undo=1 redo=1 ancoras=1 <- a ancora do botao FOI EMBORA
```

| metade | medida | resultado |
|---|---|---|
| a fotografia VÊ a âncora? | `parts_that_differ` | **sim** (`["world"]`) |
| o restauro leva-a embora? | despawn + `IkGoal` fora | **sim** |
| o PASSO nasce no clique real? | `undo` `1 → 2` | **sim** |
| o `Ctrl+Z` desfaz? | `ancoras` `2 → 1` | **sim** |

⇒ **o defeito não está no caminho que a sonda percorre.** O que falta é a **sequência exacta do
dono**, e há duas hipóteses baratas de eliminar — as duas com o mesmo sintoma (*«o Ctrl+Z não fez
nada»*):

1. **Ele arrastou a âncora depois de a criar.** O arrasto é um passo **próprio**; desfazê-lo move o
   losango de volta uns píxeis, o que se lê como *nada aconteceu*. O segundo `Ctrl+Z` é que apaga a
   âncora.
2. **O `Ctrl+Z` foi roteado para outro dono** — o `undo_or_redo` escolhe entre o Áudio, o Painter, o
   global e o image-edit.

⛔⛔ **E há uma armadilha ESTRUTURAL desta cena, medida aqui e que não é da âncora:** a cena de smoke
monta-se **sem entrada nenhuma**, então o **primeiro clique** do dono — seja ele qual for — regista
um passo cujo *antes* é a **cena vazia**. Um `Ctrl+Z` a mais apaga o desenho inteiro. (Medido: `913`
supressões em 15 s, todas a mesma diferença constante repetida — e **`0`** sem smoke nenhum. ⚠️ A
primeira leitura disto foi *«há uma deriva por quadro»*, e era falsa: duas capturas no MESMO quadro
são idênticas. *Um contador de supressões conta a mesma diferença N vezes, não N diferenças.*)

**Suspeito nomeado, e não era o primeiro a verificar:** o passe da âncora escreve a pose dos ossos
governados **todo quadro** através do `preview_drive`, e essa condução **nunca larga**. ⭐ Ele não
era a causa do undo — mas era a do **F3**, o irmão que o mesmo report trouxe.

</details>

---

### F3 — ✅ *«Remove IK não funciona plenamente»* — **CURADO** (2026-09-07)

**O que faltava era a POSE.** O verbo tirava a âncora e o alvo — e deixava a corrente **dobrada onde
a âncora a tinha posto**, sem caminho de volta.

⛔⛔ **E isso contradizia a lei que este módulo escreveu:** o que a restrição escreve é
**pré-visualização** — *vê-se, não se guarda*. Deixá-la ficar promovia-a a documento no `settle()` do
quadro seguinte, que é exactamente o que o `preview_drive` existe para impedir. O Blender faz o que
se faz agora: remover a *constraint* devolve o osso à pose de FK.

⚠️ **A `settle` NÃO servia**, e a distinção é o achado: ela trata de um motor que **largou** (e aí o
vivo *é* o documento); aqui o motor foi **desligado**, e o vivo é dele. Dois factos com a mesma
forma ⇒ porta nova, `PreviewDrive::release_to_authored`.

⚠️ **E o que volta é o AUTORADO, não o repouso** — se o artista girou o ombro à mão antes de criar a
âncora, é essa pose que volta. Um gate que só medisse *«voltou ao que estava antes de arrastar»*
ficaria verde sobre uma implementação que endireitasse a corrente, e o artista perderia trabalho.

---

### F2 — ✅ Duas âncoras na mesma corrente brigam — **CURADO** (2026-09-07)

**A cura: as correntes são DISJUNTAS e resolvem-se da RAIZ para a ponta** (`skeleton_goal::schedule`).
A âncora mais **rasa** reclama primeiro e a mais funda fica com o que sobra **abaixo** dela ⇒ cada
osso obedece a **uma** âncora e **as duas alcançam o próprio alvo** — uma IK na coluna e outra na
mão trabalham ao mesmo tempo. A ordem é **derivada da hierarquia** (não autorada, sem UI, sem estado
a gravar) e o desempate é o `StableId`, nunca o `to_bits`.

⚠️ **A ordem de posse inversa foi construída e MEDIDA como errada:** com a funda a reclamar primeiro
ela leva a corrente inteira e a de cima fica **inerte** — o artista põe duas âncoras e uma não faz
nada.

⛔⛔ **E TRÊS fixturas não discriminaram antes de a quarta o fazer.** *Convergir* e *alcançar o
alvo* ficam verdes **mesmo sem agenda**, porque o FABRIK parte da pose que encontra e tende a
**preservar** o trabalho da âncora anterior. ⇒ o defeito não é *«a cena não assenta»*: é *«ela
assenta numa pose que depende da ordem dos ARQUÉTIPOS»*. O gate que separa é
`the_plan_does_not_depend_on_the_order_the_anchors_are_found` (as duas permutações, plano idêntico).

⚠️ E um segundo `sort_by` ficou **redundante** quando a posse mudou de sentido — apagado, com a
razão escrita: *uma linha que sobrevive a uma mudança de desenho ao lado dela costuma ter deixado de
fazer alguma coisa.*

<details><summary>o diagnóstico original, para quem quiser o histórico</summary>

### O mecanismo, como foi medido antes da cura

**Sintoma** (verbatim): *«Múltiplos IKs numa cadeia de bones tem resultado ruim»*.

**⭐ O mecanismo está MEDIDO por leitura, e são três defeitos, não um:**

1. ⛔ **A ordem de resolução é a dos ARQUÉTIPOS.** O
   [`skeleton_goal::solve`](../../shells/desktop/src/skeleton_goal.rs) recolhe as âncoras com
   `iter_entities()` e **não ordena**. Com correntes independentes isso não se nota; com correntes
   que se sobrepõem, **a ordem É a resposta** — e ela não é sequer estável entre sessões.
2. ⛔ **Ninguém impõe RAIZ PRIMEIRO.** Se a âncora de baixo resolve antes da de cima, a de cima
   move os pais e **arrasta** a solução da de baixo — o resultado é sempre a última a correr, e a
   outra não vale nada.
3. ⛔ **Nada proíbe duas âncoras sobre o MESMO osso.** Elas escrevem a mesma rotação em sequência,
   todo quadro: a corrente vibra entre duas poses. É a família do defeito que a wave de 07/09 já
   curou uma vez (*«uma corrente parada não escreve»*), com outra origem.

**O que as referências fazem** — e as três dão a mesma resposta, que é *a ordem é AUTORADA*:

| Referência | como resolve |
|---|---|
| **Spine** | as restrições têm uma **ordem explícita na árvore**, que o artista reordena. É feature, não detalhe. |
| **Blender** | avalia por **ordem da hierarquia de ossos** (raiz primeiro) e, dentro de um osso, pela ordem da pilha de constraints. |
| **Rive** | ordem de dependência derivada do grafo, com o ciclo **recusado**. |

⇒ **O desenho provável** (não implementado, não decidido): a ordem sai da **profundidade na
hierarquia** (raiz primeiro, que é derivada e não precisa de UI), e duas âncoras sobre o mesmo osso
é **estado inválido** — a segunda é recusada em voz alta, como o `add()` já recusa uma segunda
âncora no mesmo osso hoje.

⚠️ **A cerca que JÁ EXISTE e não cobre isto:** o `add()` recusa uma segunda âncora **no mesmo
osso**, e o `feeds_back` recusa um alvo **dentro da própria corrente**. Nenhuma das duas vê duas
âncoras em ossos **diferentes** cujas correntes se **cruzam** — que é exactamente o caso do report.

**A régua que falta é a que apanharia isto sozinha:** resolver a mesma cena **duas vezes** e exigir
a mesma pose (o `solving_twice_from_its_own_output_gives_the_same_pose` existe para UMA corrente, em
`ph2d-skeleton`; o irmão para N âncoras não existe). ⭐ *Um gate de ponto fixo é o que separa «ordem
arbitrária» de «ordem errada».*

⚠️ **Esta última frase estava ERRADA, e a implementação refutou-a:** o ponto fixo **não** separa —
a cena assenta nas duas leis. Quem separa é a **independência da ordem**.

</details>

---

### F4 — ⏳ *«undo tem poucos passos»* — **ABERTO, e o meu diagnóstico foi REFUTADO**

**Sintoma** (verbatim, 2026-09-07): *«undo tem poucos passos»* — o `Ctrl+Z` tem menos etapas do que
o artista fez.

**O que está MEDIDO, e ilibado:**

| medida | resultado |
|---|---|
| 5 arrastos separados no canvas | **5** passos (`PH2D_BONE_UNDO_PROBE=2`) |
| *Add IK* pelo botão real | **1** passo, e o `Ctrl+Z` desfá-lo |
| posar à mão um osso governado | **não** é engolido pela fotografia |

⛔⛔ **A minha explicação era esta, e a medição derrubou-a:** *«com a corrente assente a restrição
não escreve, logo não declara condução, logo o memo fica com o autorado velho e a fotografia repõe-o
por cima da pose do artista»*. Construí a cura (declarar a condução **todo quadro**) — e o desenho
**original** passou o mesmo gate. ⇒ **revertida.** A razão é a regra da **outra mão** que o
`preview_drive` já tem: perturbar um osso governado muda a solução, então a restrição **volta a
escrever** no quadro seguinte, e nessa escrita o `before` é a pose do artista.

⚠️ **E TRÊS fixturas não produziram o fenómeno antes de a quarta o fazer**, cada uma por uma metade
diferente: a régua era `solve() == 0` (que conta escritas de **1 ULP**, não movimento) · a cadeia do
`braco()` **nunca assenta** · e sem um arrasto ANTES não há entrada no memo para ficar velha.
*A primeira vermelha que vi era a régua errada, não o produto.*

⇒ **O que falta é a sequência do dono.** O instrumento existe (`PH2D_BONE_UNDO_PROBE=2` conta
passos; `PH2D_UNDO_LOG=1` nomeia qual dos cinco motivos suprimiu). ⚠️ E há **uma armadilha da cena**
já medida que produz este sintoma sem ser um defeito: o smoke monta-se **sem entrada nenhuma**, então
o **primeiro clique** regista um passo cujo *antes* é a **cena vazia** — as acções feitas antes dele
**fundem-se** todas nesse passo.

---

### F5 — ✅ O losango e a bolinha ficavam SOBREPOSTOS — **CURADO** (2026-09-07)

**Report** (verbatim): *«quando colocamos um IK num bone no meio dos ossos, o losango do IK e o
círculo do outro osso ficam sobrepostos. Sugiro que o losango seja maior e que seu gizmo tenha
espessura maior. Então ao clicar no gizmo do losango, move-se o IK, se clicar no círculo interior
move-se o outro bone.»*

⭐ **Feito como ele desenhou, e é a única cura que serve para alvos concêntricos: eles diferem em
TAMANHO.** `goal_radius_px = (joint_radius_px + BONE_JOINT_R_PX) × 1,25` — o **piso** do anel é
**derivado**, não escolhido: ele é exactamente a tolerância do dedo desta casa, então há sempre *pelo
menos* um dedo inteiro de anel para agarrar a âncora, em qualquer zoom. O traço é mais grosso
(`GOAL_LINE_PX`), e o losango **deixou de se encher** quando seleccionado — enchê-lo tapava a
bolinha que vive por dentro dele.

⚠️ **O `× 1,25` é veredito do DONO sobre a tela** (2.º report, depois de ver a 1.ª versão: *«o
losango deve ser 25% maior»*) — ⛔ não é medição nem teto de recurso. Medido: num osso longo o
losango passa de `24` para **`30` px** de meia-diagonal, e o anel exclusivo de `12` para **`18`**.

⚠️ **O dedo tem um FURO no meio**, e é ele que deixa o clique de dentro chegar ao osso. ⛔ Mas só
quando há mesmo um osso lá: com a âncora longe de tudo o disco inteiro é dela, senão o centro do
losango seria um alvo morto.

---

## Aberto de waves anteriores (as opções que o dono ainda não escolheu)

| # | O quê | Estado |
|---|---|---|
| F3 | **Smart Bones** (Moho) — girar um osso toca uma animação inteira | nunca começado |
| F4 | **Limites de ângulo por junta** | ✅ **FECHADO** (2026-09-07) — ver abaixo |
| F5 | ~~**Pole target**~~ → **O LADO DA DOBRA** | ✅ **FECHADO** (2026-09-07) — ver abaixo |
| F6 | **A segunda mídia** (raster/Flip) | ⛔ **bloqueado**: precisa de uma malha sobre a imagem, que não existe — meça o preço antes de prometer |
| F7 | **O painel próprio do módulo** | ⏸️ adiado até F3–F5 lhe darem conteúdo (medido: hoje são 3 botões e 5 campos, que cabem na seção do vetor) |

---

---

### F5 (fila antiga) — ✅ **O LADO DA DOBRA É AUTORADO** (2026-09-07)

⭐⭐⭐ **O defeito estava MEDIDO antes de uma linha de cura ser escrita**, e é determinístico — não
é ruído: o artista dobra o cotovelo para um lado (`side_of = −99,498744`), estica o membro até ele
ficar direito (`0,000000` — *uma recta não tem lado*) e traz a mão de volta **ao mesmo alvo**; o
cotovelo aparece do outro lado (`+99,498744`). Mesma magnitude, sinal trocado. Quem decide ali é o
**desempate da corrente recta**, e ele é sempre o mesmo lado.

⛔⛔ **E o *pole target* do Blender era a resposta ERRADA para este app.** A triagem de licença (§0.9)
parou na primeira porta aberta — o **Godot é MIT** — e a API dele, corrida (`godot --headless
--doctool`), diz o que as ferramentas 2D fazem:

| ferramenta | dimensão | o que ela oferece |
|---|---|---|
| **Godot** `SkeletonModification2DTwoBoneIK` | 2D | **`flip_bend_direction: bool`** |
| **Spine** `IkConstraint` | 2D | `bendDirection` `+1`/`−1`, animável |
| **Blender** *Inverse Kinematics* | 3D | *Pole Target* (um objecto) + *Pole Angle* |
| **Maya** `ikRPsolver` | 3D | `poleVector` + twist |

⭐⭐ **O *pole* responde à pergunta do 3D, que aqui não existe.** Em três dimensões o triângulo
raiz–cotovelo–ponta **roda em torno** do eixo raiz→ponta, e é esse grau de liberdade contínuo que um
objecto no espaço fixa. No plano sobra **um bit**. Um alvo arrastável que codifica um bit dá a
ilusão de controlo contínuo e depois **salta** quando o artista cruza a recta — e é por isso que as
duas referências 2D, independentes uma da outra, escolheram a mesma forma.

**O que ficou:** `IkGoal::bend` (`Auto` · `CCW` · `CW`), pintado como uma fileira de três segmentos
depois do `Chain`. ⭐ **O valor de nascimento é CAPTURADO** — o `add` mede de que lado a corrente já
está e grava-o —, e é isso que faz a âncora continuar a nascer sem mover um pixel **e** curar o
defeito no mesmo gesto. ⚠️ Uma corrente que nasce **recta** recebe `Auto`, porque ali o desvio é
ruído de `f32` amplificado e escolher um lado seria fabricar uma decisão do artista.

⚠️ **`Auto` é o comportamento de sempre, ao bit** (gate
`keep_is_the_default_and_it_is_bit_identical_to_having_no_side_at_all`), e continua a ser o que o
**gesto** de arrastar a ponta usa: *um gesto preserva o que se vê, uma restrição defende o que se
autorou.*

⭐ **E a wave devolveu uma porta que faltava:** as duas leis desta crate mediam o lado com grandezas
de **sinal oposto** (`v1 × v2` no ramo de dois ossos, o desvio perpendicular no de 3+), e nada
escrito o dizia. Hoje há uma régua só, `ph2d_skeleton::side_of` — ⚠️ e a 1.ª redacção dos gates
novos usou a errada e **acusou a implementação certa de inverter**: escrevi a porta para não cair
nisto e caí no mesmo turno.

⛔ **Fronteira NOMEADA:** numa corrente de 3+ ossos o lado é imposto **espelhando** a pose sobre a
recta raiz→alvo antes de iterar (uma isometria, logo nenhum osso estica). Isso põe o desvio
**dominante** do lado pedido; uma corrente em S continua a ter desvios dos dois sinais, e não há
uma resposta única para «o lado» dela. O Godot responde a isso com um ímã **por junta**
(`magnet_position`), que é outra feature.

⚠️ **DUAS premissas minhas caíram por medição nesta wave:** a 1.ª fixtura não produzia a inversão
(partia do lado que o desempate já escolhe, e lia `+99,498744` nas duas pontas), e a hipótese de que
*«as duas leis desempatam para lados opostos»* — que eu ia registar como defeito — está **refutada**:
`n=2` `−7,416198` · `n=3` `−4,472136` · `n=5` `−4,486331`, o mesmo lado nas três.


---

### F4 (fila antiga) — ✅ **O LIMITE DE ÂNGULO POR JUNTA** (2026-09-07)

Sem ele o cotovelo dobra para trás e o joelho hiperextende: a corrente alcança o alvo por um
caminho que um corpo não faz, e o rig lê-se como um barbante.

⭐⭐ **Ele mora no OSSO, e o Godot (MIT, corrido por `--doctool`) põe-no na RESTRIÇÃO.** O
`SkeletonModification2DCCDIK` guarda `constraint_angle_min`/`_max` **por junta da modificação**,
logo o limite existe enquanto existir aquela IK. Aqui é um componente do **osso** — o modelo do
Blender e do Moho —, e a razão é que *«este cotovelo não dobra para trás»* é uma afirmação sobre a
**anatomia**: ela vale quando o artista gira o osso à mão, vale sem IK nenhuma, e não pode evaporar
quando ele carrega em *Remove IK*.

⭐ **Uma porta, duas mãos:** [`bone_gesture::limited`] é chamada pelo gesto que gira o osso **e**
pelo solver. Um limite que obedecesse só a um deles seria pior que limite nenhum — o artista não
conseguiria formar um modelo do que a ferramenta faz.

**O que ficou:** `BoneLimit { min, max }` em radianos locais (o mesmo espaço do
`Transform::rotation`, que numa hierarquia já é *«quanto este osso está virado em relação ao pai»*);
*Add / Remove Angle Limit* e dois campos em **graus** — a conversão vive na shell, que é a porta
onde as duas unidades se encontram. A faixa nasce **em volta da pose actual** (o quarto de volta),
que é no-op exacto e ainda assim diz ao artista para onde ele pode ir.

⛔⛔ **E o gate apanhou um defeito CARO na 1.ª redacção da lei:** ela reconstruía sempre
`centro + wrap_pi(rot − centro)`, o que **normaliza** o ângulo — medido, `−6,3` saía como
`−0,016815`. Os dois são o **mesmo ângulo** (diferem por uma volta) e **não os mesmos bytes**, e o
undo desta casa regista **por diferença de bytes**: uma junta parada dentro do próprio limite
escreveria um valor novo a cada quadro, e cada clique empilharia um passo cujo conteúdo é *«o limite
normalizou um ângulo»*. ⇒ dentro do limite devolve-se `rot` **ao bit**. É a mesma família do defeito
que o `two_bone` já pagou (*«uma corrente parada não escreve»*), com outra origem: ali era ruído de
`f32`, aqui uma volta inteira.

⚠️ **O risco desta wave era a OSCILAÇÃO**, e ele está medido: o solver resolve, o limite apara, e o
quadro seguinte parte da pose **aparada**. `a_limited_chain_settles_instead_of_oscillating` mede o
movimento por quadro a **decrescer** — ⛔ e não a ser zero, que foi a correcção que a régua do lado
da dobra também precisou: *convergir e oscilar são coisas diferentes.*

⚠️ **A ponta deixa de alcançar o alvo quando o limite morde, e isso é o CERTO** (é o que o Blender
faz com *IK limits*) — há gate a fixá-lo, para ninguém o ler como a IK partida.

⛔ **A faixa INVERTIDA (`max < min`) trava a junta no centro em vez de entrar em pânico:** um
`f64::clamp` com os limites trocados **aborta o processo**, e um `.ph2dproj` editado à mão chega lá.
A diferença entre *«a junta ficou presa»* e *«o app fechou»* é a diferença entre um defeito e uma
perda de trabalho.

⚠️ **`PROJECT_SCHEMA` 124 → 125** (componente registado novo; o registo do esqueleto vai de **4 para
5** e o catálogo do Inspector idem — conte o delta, nunca o literal).


## ⛔ Recusas MEDIDAS deste módulo — não as reconstrua

| O quê | Por quê | Onde |
|---|---|---|
| Guardar os **pesos** numa tabela por ordem de varredura | é o *vector paralelo* que o `corner_radius` proíbe por escrito; derivar custa **0,146 %** de um quadro | [`ph2d-skeleton-ecs`](../../crates/ph2d-skeleton-ecs/src/lib.rs) |
| Referenciar um osso por `Entity::to_bits()` | **medido `0 de 2`**: o undo re-spawna e os bits são ids de alocação — a pele morria em silêncio; e o `from_bits` **aborta o processo** com bits de outra sessão | degrau 122 da escada |
| Misturar FK↔IK pelas **posições** das juntas | encurta os ossos (a corda é mais curta que o arco); a mistura é sobre o **ângulo** | `ph2d_skeleton::blend_angle` |
| Um `Driver` novo no `preview_drive` para a âncora | dois memos sobre o **mesmo componente** repõem `Transform`s diferentes na mesma fotografia, e quem ganha é a ordem do `BTreeMap` | `skeleton_goal` (cabeçalho) |
| `chain = 0` (*até à raiz*) como valor de nascimento | é o default do Blender e a queixa nº 1 documentada da feature dele: ao primeiro arrasto o esqueleto inteiro dobra | `DEFAULT_CHAIN = 2` |
