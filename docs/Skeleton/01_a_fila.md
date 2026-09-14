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

⚠️⚠️ **E em 2026-09-09 aquela cura VOLTOU À ÁRVORE, por outra pergunta** (`still_driving`, F3-l): o
report *«Remove Smart Bone não devolve o objeto»* mediu que um condutor **persistente** era largado
pela `settle` um quadro depois de o output ficar constante. ⛔ **Isto NÃO reabre este item:** a
refutação acima continua de pé e tem gate (`posing_a_governed_bone_by_hand_is_not_swallowed_by_the_ledger`,
verde depois da cura). *Uma recusa medida responde UMA pergunta — e a de 07/09 era «a pose da outra
mão é engolida?», não «o autorado ainda lá está quando o motor é DESLIGADO?».*

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
| F3 | **Smart Bones** (Moho) | ✅ **FECHADO** (2026-09-08) — ver abaixo |
| F4 | **Limites de ângulo por junta** | ✅ **FECHADO** (2026-09-07) — ver abaixo |
| F5 | ~~**Pole target**~~ → **O LADO DA DOBRA** | ✅ **FECHADO** (2026-09-07) — ver abaixo |
| F6 | **A segunda mídia** (raster/Flip) | ✅ **FECHADA para o RASTER** (2026-09-09) — ver F6 abaixo. ⛔ A nota antiga dizia *«bloqueado: precisa de uma malha sobre a imagem, que não existe»*: estava certa sobre o facto e errada sobre o preço — **duas das quatro peças já existiam**, e o doc de uma delas dizia-o por escrito. O **Flip** continua por fazer |
| F7 | **O painel próprio do módulo** | ✅ **FECHADO** (2026-09-09, por escolha do dono) — ver F3-m abaixo. A nota antiga: ⏸️ **a condição CAIU e a medição era falsa por ~3×** — ela dizia *«adiado até F3–F5 lhe darem conteúdo (hoje são 3 botões e 5 campos)»*, e as três estão ✅ nesta mesma tabela enquanto a secção tem **10 verbos** e **9 campos** (`VECTOR_BONE_VERBS`/`_FIELDS`, comprimento verificado pelo compilador), mais uma fileira segmentada e dois selectores. ⇒ decisão do dono, não mais um adiamento medido |

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


---

### F4-b — ✅ **OS LIMITES VÊEM-SE E PEGAM-SE NO CANVAS** (report do dono, 2026-09-07)

> *«os limites devem ser visíveis e manipuláveis no canvas através de gizmos»*

Um limite que só existe como dois números num painel é um limite **invisível**: o artista não vê
onde a parede está, não a pode empurrar, e tem de traduzir graus de cabeça.

⭐⭐⭐ **O raio do setor é o COMPRIMENTO DO OSSO, e isso elimina toda a arbitrariedade:** o arco é
literalmente o caminho que a ponta percorre, e arrastar uma alça é *«leve a ponta até aqui e
trave»*. ⛔ Um raio escolhido em píxeis seria um número sem dono, e num osso curto cobriria o
esqueleto inteiro.

**O que ficou:** um setor translúcido a partir da junta, as duas paredes traçadas, e duas alças
**triangulares** nas pontas — a junta é um círculo, a força é um quadrado, o limite é um triângulo:
*três alças do mesmo osso, três formas.* `BonePart` ganha `LimitMin`/`LimitMax`.

⚠️ **O leque vem AMOSTRADO em mundo**, e não como *(centro, raio, dois ângulos)*: sob um afim
não-conforme (o pai escalado só em X) um arco de circunferência é uma **elipse**, e reconstruí-lo de
um raio só desenharia a coisa errada exactamente onde o artista mais precisa de confiar no que vê.

⛔⛔ **E o gate apanhou uma COLISÃO DE ALÇAS que a 1.ª redacção tinha.** Ela testava as três alças do
osso em foco por **ORDEM** (força, depois paredes); com `strength ≈ 1` e uma parede perto de 90° elas
caem a menos de um dedo uma da outra e a parede ficava **inalcançável**. ⚠️ Reordenar não cura — só
troca quem fica inalcançável. ⇒ **ganha a mais PERTO do ponteiro**, que é a única regra que não
escolhe uma vítima.

⭐ **E a colisão é REAL mas condicionada ao ZOOM, com o número ao lado:** a tolerância do dedo são
`12 px`, logo em mundo ela vale `12 × px_to_world`. No zoom de trabalho (o osso a ~100 px, medido
`107,52`) isso são `1,2` unidades e as alças da fixtura distam `5,0` — não colidem. Com a câmera
afastada (o osso a ~20 px) a tolerância passa a `6,0` e elas colidem. *O defeito só aparecia quando
o artista olhava o rig inteiro.*

⭐⭐ **E a wave endireitou o DESPACHO do gesto:** o `pose` despachava por `if part == …` com o corpo
do osso a apanhar tudo o que sobrasse, então uma alça nova caía **no braço do `Body`** e girava o
osso em silêncio — a família do *dreno de um braço só* (`CLAUDE.md` §5), que nenhuma sonda deste repo
vê. Hoje é um `match` **exaustivo**: uma alça sem verbo é **erro de compilação**.

⚠️ Três cortes por responsabilidade nesta wave (nenhuma isenção): `bone_limit.rs` (a lei do limite),
`bone_pose.rs` (*o que a mão faz* × *o que o dedo aponta*) e `ph2d-skeleton-render/limit.rs` (irmão
do `goal.rs`).


---

### F4-c — ✅ *«os gizmos de limite mudam de posição sozinho após mover a cadeia»* (report, 2026-09-07)

⭐⭐⭐ **O mecanismo, MEDIDO em três chamadas:** como o raio do arco é o comprimento do osso, quando
ele **encosta na parede** a ponta e a borda do setor ocupam o mesmo ponto — `distância ponta→parede
= 0,000000` — e o dedo devolvia `LimitMax` onde o artista queria a ponta. Ele movia a cadeia até ao
limite, agarrava para continuar, e **arrastava a parede**. O gizmo mexia-se sem ele o ter pedido.

⛔⛔ **Priorizar a ponta sobre a parede NÃO cura — troca a vítima** (a parede ficaria inalcançável
exactamente quando o osso está nela). É a mesma lição que a colisão força↔parede já tinha dado
horas antes, e a segunda vez que ela apareceu nesta wave.

⇒ **A cura é geométrica:** a alça sai para **fora** do raio que o osso alcança, a uma folga
derivada — um dedo da casa (`BONE_HIT_PX`) mais o raio do próprio triângulo
(`LIMIT_HANDLE_R_PX`), para a alça **inteira** ficar fora e não só o centro dela. ⚠️ É por isto que
o `arc` passou a precisar do ZOOM: a folga é uma grandeza de **tela** sobre geometria de **mundo**.
O setor continua a ir até ao comprimento do osso — ele é o caminho da ponta, e isso não mudou.

⚠️ **TRÊS hipóteses minhas caíram por medição antes de eu achar esta**, e as três estavam ilibadas:
o `arc()` acompanha o pai rigidamente (`−0,4 → +0,3` com `+0,7` aplicado) e não se move quando o
próprio osso gira · ele é estável sob a IK em quadros sucessivos (`−0,9 → −0,9`) · e o round-trip
`radianos → graus → radianos` do painel é **exacto** e não deriva (600 quadros: `0`).

⭐ **E a cura curou também a colisão força↔parede**, que deixou de acontecer por acaso (as duas
passaram a distar `9,86`). ⚠️ O gate daquela regra ficou **vácuo** e teve de ser reescrito para
**construir** o encontro em vez de torcer por ele — a força é ajustada por um valor **derivado da
posição da parede**. *Uma fixtura que espera uma coincidência morre quando a coincidência é curada.*


---

### F4-d — ✅ *«gizmo não mantém ângulo fixo em relação ao osso»* (2.º report, 2026-09-08)

⭐⭐⭐ **O DEDO E O DESENHO PERGUNTAVAM POR OSSOS DIFERENTES.** O dedo lê o osso da selecção
**INTEIRA** ([`bone_gesture::selected_bone`], cujo doc explica porquê: prender uma forma a um
esqueleto entre vários faz-se escolhendo os dois, e aí **o primário é a forma**); o desenho lia só o
**primário** do gizmo. Com uma forma seleccionada ao lado do osso — que é o gesto do *Bind* — as
alças respondiam num osso e o arco era pintado noutro, ou em sítio nenhum.

⚠️⚠️ **E um doc AFIRMAVA que as duas eram a mesma pergunta** (*«o foco é a SELECÇÃO, e é a mesma
pergunta que o dedo faz»*, no `draw_influence`): *uma afirmação de igualdade sem um gate é um
comentário.* ⭐ O doc do `selected_bone_bits` já prescrevia a cura — *«no laço de desenho chama-se a
função livre acima»* — e ninguém a chamava.

⛔ **A `draw_influence` tinha o mesmo defeito** e foi curada no mesmo passe: a alça da força sofria
disto desde que existe.

⚠️ **DOIS gates, porque são duas perguntas:** um mede a PORTA (o `selected_bone` acha o osso quando
o primário é a forma) e outro mede o CHAMADOR (o laço de desenho recebe mesmo `osso_focado`). *Um
gate sobre a porta não cobre quem a ignora* — provado por mutação: repor `gizmo.selection` deixa o
primeiro verde e reprova o segundo.

⚠️⚠️ **TRÊS hipóteses minhas foram medidas e ILIBADAS antes desta**, e as três viraram gates: o arco
acompanha o pai rigidamente · é estável sob a IK em quadros sucessivos · e a parede no ângulo que o
osso TEM cai na ponta dele a `1e-7` (sobre a cadeia montada pela porta REAL, não por uma fixtura à
mão — foi por medir a errada que elas saíram todas verdes).

⏳ **ABERTO, medido e nomeado:** com **escala NÃO-UNIFORME** na cadeia o ângulo entre o osso e a
parede distorce-se (`DIF` de `+0,500000` para `+0,135736` com a raiz a `2 × 0,5`). É geometricamente
correcto — o arco é a imagem do caminho da ponta, que sob um afim não-conforme é uma **elipse** — e
⛔ **não é o que o dono viu** (o `bone_gesture::create` usa `Transform::IDENTITY`, logo a cena não
tem escala). Fica aqui porque um rig escalado num eixo é coisa que um artista faz.


---

### F4-e — ✅ *«não consigo mover os gizmos dos ângulos»* (regressão MINHA, 2026-09-08)

⛔⛔ **A cura do F4-c causou-a.** Ao empurrar a alça para **fora** do alcance do osso, o único alvo
ficou a `17 px` ALÉM da borda do setor — e é a **borda** que se lê como *«a parede»*. O artista
mirava no que via e não havia alvo nenhum ali: um triângulo de `5 px` a `17 px` do sítio para onde a
mão vai. *Um alvo que não está onde a coisa PARECE estar é um alvo ausente.*

⇒ **o alvo passa a ser o SEGMENTO inteiro** do vértice até a alça, que é exactamente o traço
desenhado: grande, debaixo do que o artista vê, e passa pelo triângulo por construção.

⚠️ **E o OSSO entra na mesma competição de proximidade**, senão a cura devolvia o F4-c ao contrário
— as paredes **cruzam** o osso sempre que ele se aproxima de uma delas, e sem isso a parede roubaria
o gesto de girar em toda a faixa. *Ganha o que está mais perto do dedo*, agora sobre **todos** os
alvos do osso em foco (a força, as duas paredes e o próprio osso). É a terceira vez nesta wave que a
resposta é a proximidade e não a ordem.

⚠️ **A sonda do dedo foi RETIRADA depois do smoke aprovado** (ordem do dono, 2026-09-08): ela
imprimia a cada mudança de realce, que num rato a mexer são dezenas de linhas por segundo. ⭐ O que
ela mediu está aqui: as três hipóteses que a lógica não distingue — *não vejo* · *não alcanço* ·
*agarro outra coisa* — separam-se pela **distância do ponteiro a cada alça** contra a tolerância do
dedo (`12 px × px_to_world`), e é essa a linha a reconstruir se o report voltar. ⛔ O `PH2D_BONE_LOG`
em si **fica**: ele é anterior a esta wave e serve o diagnóstico da pele e o do laço da âncora.

⚠️ **A lei do limite esteve CERTA nos três reports** — o que falhou foi sempre o gizmo. Por isso os
gates estão agora cortados em dois ficheiros: `skeleton_limit_tests` (a lei) e
`skeleton_limit_gizmo_tests` (o que o dedo apanha). *As duas perguntas falham de maneiras
diferentes.*


---

### F3 (fila antiga) — ✅ **OS OSSOS INTELIGENTES** (2026-09-08)

Girar um osso **percorre uma acção inteira**. É o *Smart Bone* do Moho e o *Action Constraint* do
Blender: o artista grava uma acção na timeline e diz *«quando este osso vai de A a B, ela vai do
princípio ao fim»*. Um osso passa a ser um **controlo**.

⭐ **Para que serve, e por que não se resolve com pesos:** a **correcção** (um cotovelo a 120° amassa
a manga, e a deformação certa naquele ângulo é uma pose AUTORADA, não uma interpolação) e o
**controlo composto** (um osso solto que abre uma boca, fecha uma mão, vira uma cabeça de perfil).

⚠️ **A nossa forma é a do Blender e a do Moho** (um ângulo → o **TEMPO** de uma acção), e **não** a do
Godot (`AnimationNodeBlendSpace1D`: um valor → a **MISTURA** de N animações). Misturar duas poses
exige que elas existam e sejam compatíveis; percorrer um clip exige **um** clip — e é o que o artista
já sabe fazer aqui, porque ele grava na timeline que já existe.

⭐⭐ **A porta que faltava era pequena, e o desmonte do «precisa de Y» valeu a pena:** a timeline já
sabia amostrar (`Clip::sample`) e escrever (`write_prop`); faltava **tocar UM clip num instante
derivado, sem mexer no transporte** (`ph2d_timeline::apply_one_clip`, 40 linhas, `doc` por referência
**partilhada** — trocar o clip activo e repor seria uma janela de um quadro em que o documento mente
sobre si próprio, e o painel lê-o no mesmo quadro).

⭐ **O gesto de ligar é de DUAS MÃOS** (o do *Bind*): a acção é a que está **aberta** na timeline. ⛔
Digitar o nome seria a quarta superfície a poder discordar dela. E o clip é nomeado pelo **NOME**,
nunca pelo índice — reordenar clips faria o osso percorrer a animação do vizinho, em silêncio.

⚠️ **O que a acção escreve é PRÉ-VISUALIZAÇÃO**, e o ledger é o **da timeline**
([`timeline_preview`]), não um novo: o que ela escreve é literalmente o que a timeline escreveria, e
aquela porta já cobre os **quatro** factos (pose · alfa · `t` do morph · params de junta). ⛔ Um
ledger próprio poria dois memos sobre o mesmo componente.

⚠️ **A ORDEM no quadro:** o passe corre **antes** da âncora de IK — o controlo escreve a pose de
BASE e a IK é a restrição que persegue um alvo, logo tem de ver a pose já corrigida.

⛔⛔ **E um defeito que só a construção revelou:** um clip acabado de criar nasce com
`duration = 0`, e o artista grava as chaves sem lhe tocar. Multiplicar por esse zero deixa a acção
presa no instante `0` para todo ângulo — **medido**: `dur = 0.0`, `feitas = 1`, `x = 0`. *O controlo
pinta, o gate de escrita conta `1`, e nada se move.* ⭐ A porta certa já existia e responde à pergunta
inteira (`clip_end_seconds`: o override, senão a extensão das CHAVES, e o clip só-de-expressão). *A
grandeza crua e a pergunta têm nomes parecidos e respostas diferentes.*

⏳ **DÍVIDA NOMEADA:** o painel mostra os dois ângulos e **não diz qual acção está ligada** — falta a
este painel uma **linha de texto de leitura**, e construí-la é wave própria. Quem responde é o log do
gesto. *Uma dívida nomeada e um controlo mudo leem-se igual na tela; a diferença é esta linha
existir.*


### F3-b — ✅ *«não há meios de selecionar nem o objeto alvo nem a animação»* (report, 2026-09-08)

⛔⛔ **A dívida nomeada acima NÃO era uma dívida: era o defeito.** O dono correu o smoke e leu a
feature como avariada — e tinha razão nas três metades, cada uma invisível sozinha:

| o que faltava | como se lia na tela | medido em |
|---|---|---|
| o painel não NOMEIA a acção ligada | dois campos de graus **sem sujeito** | `paint_bone::smart_rows` — `Remove` + 2 números, nada mais |
| o gesto adoptava o clip **ABERTO** | todo osso casava com a animação principal, calado | `TimelineDoc::new()` tem **um** clip, `"Main"` |
| a timeline nasce **FECHADA** | não havia onde gravar a acção | `TimelinePanel::DEFAULT_VISIBLE = false` |

⇒ *um controlo cujo sujeito é invisível lê-se exactamente como um controlo morto*, e a única
resposta que a casa dava era um `eprintln!` que o artista nunca vê.

⚠️⚠️ **LEIA A F3-c ANTES DE ACREDITAR NAS TRÊS PORTAS ABAIXO** — duas delas foram **removidas** no
mesmo dia, por ordem do dono, e a terceira mudou de comportamento: o selector passou a **filtrar**
pelo alvo em vez de listar o documento inteiro. Os nomes `fresh_action_name`,
`TimelineIntent::AddNamedClip` e a porta `a_clip_is_born` **já não existem no código** — sobrevivem
só neste parágrafo, como história.

⭐⭐⭐ **A cura são três portas, e as referências dão as duas metades:**

1. **O gesto CRIA a acção** com o nome do osso (`fresh_action_name` — *«Bone 7 Action»*, *«… 2»* se
   estiver tomado), torna-a activa e **abre a timeline**. É o *Create Smart Bone Action* do Moho.
   ⛔ Nunca adoptar a aberta.
2. **O selector `Action`** na secção Skeleton lista os clips do documento e troca o ligado — o
   botão **New** + o *datablock* do *Action Constraint* do Blender. Ele é o **readout e o gesto**:
   o rótulo do chip é o nome da acção, então *«qual é?»* responde-se sem abrir nada.
3. **`TimelineIntent::AddNamedClip`** — o clip nasce pela MESMA porta que o `+` (`a_clip_is_born`:
   4 s de duração e `set_active`), só o nome difere. ⛔ `AddClip` + `RenameClip` obrigaria o
   chamador a adivinhar `clips().len()`, que é errado à primeira intenção enfileirada à frente.

⭐⭐ **E a construção revelou um quarto defeito, que torna a feature inutilizável e não é uma
nicety: um controlo NÃO pode percorrer a acção que está ABERTA.** Ali o artista está a gravá-la; os
dois escrevem o mesmo objecto no mesmo quadro e o passe do controlo corre **depois** do da timeline
⇒ arrastar o playhead não move nada (o ângulo repõe sempre o mesmo instante) e a pose acabada de pôr
é reposta antes de ser vista. ⚠️ E o **autokey corre ainda mais tarde** (`autokey_pass` na linha
~12 862, contra ~9 780 do controlo): com o objecto seleccionado ele leria a saída do próprio
controlo como *«o artista mexeu»* e cunharia chaves a partir dela — um laço fechado. ⇒ a lei é a do
Moho: **dentro de uma acção o relógio é o do editor**.

⚠️ **A cena de smoke passou a trazer uma acção PRONTA** (`"Leaf Rises"`, a folha roxa sobe `3` em
`2 s`, semeada por `vec_bone_smoke::seed_demo_action` e deixada **fechada**). Sem ela o único caminho
para provar um osso inteligente era gravar uma animação primeiro, e o smoke passava a testar a
timeline em vez do osso — *uma cena que só produz o fenómeno depois de o artista acertar OUTRO gesto
não prova nada quando esse gesto falha*.

⛔ **Uma linha nova na catraca do `the_painted_control_reaches_a_consumer`**, com o motivo medido: o
chip `VECTOR_BONE_SMART_CLIP` é vivo pelo despacho **genérico** de `Dropdown`, que não nomeia id
nenhum. ⚠️ **É a TERCEIRA ocorrência dessa família e a mais instrutiva** — os chips irmãos (mistura
de filtro, tecla de forma do Morph) escapam à régua **por acidente de FORMA** (os ids deles são
gerados por função, e ela lê `ids::LITERAL`), não por estarem ligados. ⛔ Por isso a cura não é
dar-lhe um id gerado: seria esconder o controlo em vez de o ligar.

⚠️ **E o `cargo fmt` re-expandiu o `reach_tests.rs` para 735 linhas** (teto 700) — curado por
**corte por responsabilidade** em `reach_limit_tests.rs` (a parede de uma junta) e
`reach_action_tests.rs` (o instante que um ângulo pede), ⛔ nunca por isenção.


### F3-c — ✅ *«melhor não criar nada»* + *«um botão de picker … só as animações do objeto»* (2026-09-08)

O dono correu o smoke da F3-b (*«Funciona!»*) e devolveu **duas ordens**, que juntas apagam metade
do desenho da manhã:

1. *«porque criar Bone Action no inspector e na timeline? **Melhor não criar nada**»* ⇒ o gesto
   **anexa o controlo VAZIO** e mais nada. A `TimelineIntent::AddNamedClip`, a abertura automática
   da timeline e o `fresh_action_name` foram **removidos** — não desligados: nenhum deles tem
   consumidor, e um ponto de extensão sem chamador é dívida com cara de feature.
2. *«é necessário um botão de picker para selecionar o objeto seja no canvas ou seja na hierarquia
   … só deve aparecer as animações relacionadas ao objeto selecionado»* ⇒ **`SmartBone::target`**
   (o NOME do objecto) + a linha ***Pick Object*** + o **filtro** da lista.

⭐⭐ **As DUAS superfícies que ele pediu saem de GRAÇA, e é o desenho:** o pick resolve por *«a
selecção passou a ser outra coisa»*, e o canvas e a Hierarquia escrevem a **mesma** selecção. ⛔ Um
segundo caminho de acerto (um hit-test próprio) seria a segunda resposta à mesma pergunta, e
divergiria do primeiro no dia em que um deles mudasse. ⚠️ O **osso é capturado no arm** (a lei do
`PathPick`: o clique seguinte MUDA a selecção, logo lê-la então leria o alvo), e a selecção **volta
ao osso** depois de acertar — ao contrário dos irmãos `PathPick`, porque aqui o contexto do artista
é o painel do osso, e sem isso a secção desaparecia debaixo dele no instante do acerto. `Escape`
desiste; ⛔ clique no vazio **não** desarma.

⚠️⚠️ **METADE DA PREMISSA DELE ESTÁ MEDIDA E É OUTRA:** a lista **não cresce com os objectos** — ela
lista **clips**, e o documento recusa mais que `MAX_CLIPS` = **16** (`add_clip`, e o pool de ids do
selector é gateado contra esse número). ⇒ o que o filtro compra não é **tamanho**, é **relevância**:
*quais destas 16 tocam este objecto* é a única pergunta que as separa. *A ordem estava certa e a
razão escrita ao lado dela não* — e vale registá-lo, porque a próxima leitura desta linha vai herdar
a razão, não a ordem.

⛔⛔ **E há uma LEI no filtro: um filtro que esvaziaria a lista NÃO se aplica.** Os três casos são
reais — o alvo apagado, o alvo renomeado, e um objecto que nunca foi animado —, e nos três filtrar
daria um selector com **zero** opções. *Um controlo que só sabe recusar é pior que um ausente*, e
ali o artista não teria gesto nenhum que o curasse: o alvo escolhe-se no canvas, não na lista.

⚠️ **`PROJECT_SCHEMA` 126 → 127** — um CAMPO novo no componente. ⚠️ **O degrau é obrigatório e a
razão é o postcard, não o campo:** ele é **posicional**, logo um ficheiro de três campos seria lido
com quatro **em silêncio** (os bytes do `from` entrariam no `target`); com o degrau, o load recusa em
voz alta.

⚠️ **O descritor passou de `intrinsic` a `authored`**, e a razão MUDOU com o desenho: ele era
intrínseco porque o gesto lhe dava a acção e a paleta não tinha caminho para o artista o completar —
hoje as duas linhas do painel completam-no, e nascer vazio é um no-op exacto.

⚠️ E o **`Add Smart Bone` deixou de ter um caminho de recusa**: ele já não lê a timeline, logo já não
pode dizer *«não há acção aberta»* nem *«já há 16»*.


### F3-d — ✅ os DOIS defeitos que o smoke da F3-c devolveu (2026-09-08)

**(a) *«Pick object deve inibir a criação de bones. Ao tentar fazer o pick no canvas criou um osso
indesejado»***

⛔⛔ **A causa é uma lei que este app já tinha escrita e que o meu pick não seguia: *um pick armado é
MODAL — ele CONSOME o press*.** O *Pick Object* arma-se a partir da secção Skeleton, logo o artista
está na ferramenta **Bone** — a única do app em que um `Down` no canvas **CRIA** alguma coisa. A 1.ª
versão não consumia o press: ela esperava que a **SELECÇÃO** mudasse, e no modo *Criar* o clique não
selecciona, **desenha**.

⇒ *um pick modal que não consome o press herda o gesto da ferramenta em que foi armado* — e esta é a
única que cria. A guarda entra ao lado do irmão **independente de ferramenta** (o conta-gotas de
corpo de junta), e ⛔ **não** ao lado do `vec_path_pick`, que só é modal no modo *Select*: uma guarda
que exigisse um modo teria de nomear exactamente o modo em que o defeito acontece, e um modo novo
nasceria fora dela.

⚠️ **E o defeito tinha um segundo andar, silencioso:** o osso criado por engano **mudava a
selecção**, e a resolução por selecção lia isso como *«o artista escolheu este objecto»* ⇒ o alvo
ficava a apontar para um osso acabado de nascer. A guarda modal cura os dois de uma vez.

⭐ Gates: `the_smart_bone_target_pick_precedes_the_bone_gesture` (a âncora é a **CHAMADA**, nunca a
declaração — a lição, escrita, do gate irmão do `the_node_ops_are_wired`) e
`the_target_pick_guard_asks_no_tool_and_no_mode`. Mutação: apagar a guarda ⇒ os dois VERMELHOS.

**(b) *«ao selecionar na lista de actions … não consegue selecionar o clip desejado»***

⛔⛔ **Defeito meu, da mesma jornada, e do pior tipo que há.** O clique numa opção devolve uma
**POSIÇÃO** no pool de ids, e a shell resolvia-a contra `doc.clips()` — a lista **INTEIRA**. Isso
estava certo enquanto o painel mostrava todas, e deixou de estar **no instante em que o alvo passou a
FILTRAR** (a F3-c, três horas antes). Com o filtro activo, carregar na 1.ª linha escrevia o 1.º clip
do DOCUMENTO.

⚠️ **A leitura errada compila, devolve um nome VÁLIDO, e o osso passa a percorrer uma animação que o
artista nunca escolheu.** ⇒ *uma posição só significa alguma coisa ao lado da lista que a produziu*,
e por isso quem indexa é a MESMA porta que constrói (`skeleton_smart::action_at` → `actions_for`).

⚠️ **O PAINEL foi ilibado por medição antes de eu tocar na shell:** o gate novo
`the_option_you_press_is_the_one_you_see` faz o gesto REAL sobre a **2.ª** linha (⛔ não a 1.ª: um
erro de deslocamento de um passa despercebido no índice `0`) e prova que ela devolve o 2.º id, na
ordem de cima para baixo. ⭐ E a mutação do lado da shell **reproduz o report à letra**: pôr
`doc.clips().get(i)` de volta deixa `the_chosen_position_resolves_against_the_list_the_panel_painted`
vermelho com *«a posição 0 devolveu o 1.º clip do DOCUMENTO em vez da 1.ª linha que o artista viu»*.

⭐ **A lição que fica é de ARQUITECTURA, não deste selector:** *acrescentar um filtro a uma lista
transforma todo índice que alguém guardou noutra coisa* — e nenhum tipo, nenhum compilador e nenhum
gate de registo vê isso, porque as duas listas têm o mesmo tipo e o mesmo tamanho no caso comum.


### F3-e — ✅ *«tudo configurado e a animação não rodou ao rotacionar o bone»* (2026-09-08)

⭐⭐⭐ **MEDIDO no app a correr, e o motor está ILIBADO.** A sonda nova `PH2D_BONE_SMART_PROBE=1`
(sobre a cena dos ossos) liga o controlo à acção da cena, gira o osso `+4,58°` por quadro e imprime
a linha que separa as causas:

| osso | rotação ao longo dos quadros | a folha roxa |
|---|---|---|
| **livre** (a ponta do tentáculo) | `+21,8° → +58,4° → +95,1°` | `y +0,573 → +1,795 → +3,000` |
| **governado** (do braço, sob a âncora de IK) | **PRESA em `+30,4°`** | `y` parado |

⇒ **a causa é QUAL osso.** Um osso na corrente de uma âncora tem a pose **reescrita pelo solver
depois** do passe do controlo (a ordem é lei e está documentada: o controlo escreve a pose de BASE e
a IK tem de a ver corrigida) ⇒ girar não muda o ângulo, e a acção congela no mesmo instante.

⭐ *Não é uma proibição que alguém escreveu: **o ângulo de um osso governado não é uma coisa que o
artista escreve**.* É a mesma lei da pose de repouso de um corpo dinâmico — o dono do `Transform` é o
solver, sempre. ⚠️ E a cena convida ao erro por construção: **o braço nasce com âncora**, então ele é
o osso natural para experimentar.

⇒ o app **avisa** (toast, no *Add Smart Bone*) e **anexa na mesma**: ⛔ recusar deixaria o artista
sem caminho, porque tirar a âncora é um gesto que existe (*Remove IK*). O que não pode é ficar
**calado** sobre um controlo que ele sabe que vai nascer mudo. Porta:
`skeleton_goal::is_governed` · gate `a_bone_under_an_anchor_cannot_be_a_control` (com a metade que
impede a régua de acusar a cena inteira).

⚠️⚠️ **E a SONDA pagou a mesma lei DUAS vezes na mesma hora**, o que vale mais que o achado:

1. A 1.ª versão armava **no quadro 30** e nunca disparou — *uma janela em segundo plano redesenha
   poucas vezes* (medido: ~11 quadros em 25 s). ⇒ **não se contam quadros, pergunta-se o FATO**, que
   é a lei que a cena ao lado desta já tinha escrito no cabeçalho dela.
2. A 2.ª armava em `ossos.last()` e caiu num osso do **braço** — o governado. A leitura ingénua
   disso teria sido *«o osso inteligente não funciona»*, e teria mandado reescrever um motor são.
   ⇒ *uma sonda que arma no sujeito errado mede o passe do vizinho.*

⚠️ E o log do passe (`PH2D_BONE_LOG=1`) passou a dizer a história inteira numa linha — ângulo, faixa,
duração, instante derivado, e **quantas propriedades escreveu de quantas a acção TEM**. *Sem a
segunda metade dessa razão, `0 escritas` não distingue «o clip está vazio» de «o objecto que ele
anima já não existe».*

⚠️ **`PROJECT_SCHEMA` 125 → 126** (componente registado novo; o registo do esqueleto vai de **5 para
6** e o catálogo idem — conte o delta). ⚠️ E o gate `every_registered_component_has_a_descriptor`
apanhou-me a registar sem descrever: *o Inspector e o registo são duas listas, e o gate é o que as
ata.*


### F3-f — ⭐⭐⭐ A AUDITORIA DE CINCO LENTES (2026-09-08), e o que ela mudou

O dono mandou auditar antes de seguir. Cinco lentes independentes sobre os 12 commits: **correcção ·
costura de UI · as réguas · estado/quadro/persistência · cercas e isenções.**

⭐ **O veredito de conjunto foi bom nas duas metades que costumam falhar:** a linha **não silenciou
um único diagnóstico** (zero `#[allow]`, zero `#[ignore]` novo, zero folga de LOC — cinco cortes por
responsabilidade onde um ficheiro estourou), e a costura dos **29** controlos novos estava completa
nas quatro perguntas (pintado · registado · o clique atravessa · alguém DECIDE com o valor).

⛔⛔ **E mesmo assim havia SEIS defeitos de produto que o smoke não alcança:**

| defeito | mecanismo | cura |
|---|---|---|
| a lista de acções oferece a **ABERTA**, e o motor recusa-a | um documento novo tem **uma** acção e ela **está aberta** ⇒ a única opção era a única que não corre, **calada** | a lei fica; o app **diz** (toast), pelo mesmo instrumento do aviso da âncora |
| o *Pick Object* armado **sobrevive** a load / `Ctrl+Z` / apagar o osso | ele consome o `Down` primário em TODA ferramenta ⇒ canvas morto ao botão esquerdo, e o botão *Picking…* já não é pintado ⇒ **invisível** | pergunta-se o **FACTO** por quadro (o osso ainda é um controlo?), ⛔ não uma lista de sítios a limpar |
| apagar o objecto animado **destrói a acção de todos os controlos** | a purga da timeline reseta o documento inteiro na última binding; a recuperação fica em **duas** pilhas de undo | o reset passa a perguntar **quem depende**: um clip que um controlo NOMEIA não é estado rançoso |
| o **AutoKey** cunha chaves a partir da saída do controlo | a invariante do `autokey_pass` exige que ninguém escreva pose entre o apply e ele, e os passes do esqueleto escrevem exactamente aí | o autokey salta quem o **ledger de pré-visualização** está a conduzir (`PreviewDrive::drives`) — e isto cura também a âncora, que era **pré-existente** |
| `Limit Min > Max` **congela a junta**, e o desenho continua bonito | o guarda cruzado existia no **arrasto** e não nos **campos**; e o `arc()` normaliza os dois extremos | as duas superfícies passam pela mesma porta (`bone_limit::set_edge`) |
| um `from_bits` cru na `ph2d-timeline` | a crate escreve **duas vezes** *«`try_from_bits`, NUNCA `from_bits`»*, e este era o único dos 15 sítios a violá-la — `from_bits` **aborta o processo** com bits de outra sessão | uma linha |

⚠️⚠️ **E QUATRO dos reports do dono não tinham prova que os apanhasse de volta** — em cada um
existia um gate com o nome certo, a medir a metade que **não** era o defeito:

| o report | o que o gate media | o que passou a medir |
|---|---|---|
| *«o gizmo não mantém ângulo fixo»* | o **NOME** da variável, e uma `contains` satisfeita por **3** sítios do ficheiro | a **ATRIBUIÇÃO mais próxima antes** de cada `draw_*` vem da porta do dedo |
| *«o pick criou um osso»* | a **ORDEM** do despacho | a **CONSUMPÇÃO**: a linha a seguir à chamada tem de ser `return;` |
| *«não consegue selecionar o clip desejado»* | a **porta** (`action_at`) | o dreno chama UMA porta (`choose_action`), e é ela que é gateada |
| *Add Angle Limit* | **nada** — o verbo shipou com zero gates, com o doc a afirmar *«é um no-op exacto»* sobre nada | o no-op **e** o centro **e** a largura da faixa, em três poses |

⚠️ **A cerca das constantes da cena ganhou a metade que faltava**: ela afirmava o piso
(`TENTACLE_LIMITED_BONE >= 1`) e não o tecto, então pôr o número acima da contagem da cadeia fazia a
cena nascer **sem limite nenhum** — muda, e sem nada a acusar.

⛔⛔ **E treze textos descreviam desenhos que já não existem** — nove porque *a wave seguinte mudou o
facto e não voltou à frase*. Os dois caros diziam **«e NÃO X»** sobre coisas que o código passou a
fazer: cinco sítios afirmavam que o pick *«resolve pela selecção e nunca por um hit-test próprio»*
(o commit seguinte construiu o hit-test — que é a cura do report), e o doc do `two_bone` dizia que
**derivar o lado da pose é a CURA** do salto do cotovelo, quando é o defeito medido.

⭐ *A lição instrumental: quando um facto é medido e escrito num sítio, um gate tem de o ler dos
DOIS. Hoje nenhuma sonda deste repo pergunta se duas prosas sobre o mesmo facto concordam.*

⚠️ E uma que a própria auditoria cobrou no mesmo dia: o método novo do ledger nasceu **entre um
`#[cfg(test)]` e o item dele** e herdou-o — *um item novo colado a um atributo rouba-o ao dono*, que
é exactamente a família (quatro ocorrências) que a lente da correcção acabara de nomear.

⏳ **FICA ABERTO desta auditoria** (com mecanismo, sem cura): *Remove Smart Bone* não devolve a pose
autorada (o *Remove IK* devolve, e o preço é a lista de N entidades × 4 drivers em vez de uma
corrente) · dois clips podem partilhar o NOME e o controlo percorre o primeiro, calado · o ledger
cobre 4 das 5 escritas do `write_prop` (falta o `VecDrivenStyle`, que é desregistado) · o gizmo do
limite é pintado e ACENDE em todo modo de vector e só é agarrável no modo Osso · o aviso do osso
governado só dispara na ordem *IK → Smart* · e quatro verbos da secção morrem em silêncio na janela
*«nenhum osso em foco»* (o braço que fala cobre só os dois da âncora).


### F3-g — ✅ *«selecionar o bone nem sempre abre a seção de skeleton no painel»* (report, 2026-09-08)

⛔⛔ **O *«nem sempre»* era o painel já estar rolado até lá — a secção NUNCA cabe na tela por si.**
Medido no arnês do painel (`MockPanelHost`, viewport `1600 × 900`), o `y` do cabeçalho da secção
SKELETON contra a faixa visível do painel (`774 px`, de `114` a `888`):

| o que está seleccionado | `y` do cabeçalho | altura do conteúdo |
|---|---|---|
| só um osso, modo **Osso** | **1394 px** | 1542 |
| só um osso, modo **Select** | 1394 px | 1542 |
| osso + forma presa | 1394 px | 1542 |
| \+ forma com traço | **~1978 px** | — |

⇒ ela está **sempre pelo menos `506 px` abaixo da dobra**. Ver a secção só acontecia quando o painel
tinha ficado rolado por outro motivo, e é isso — e só isso — o *«nem sempre»*.

⚠️ **As TRÊS leituras possíveis do report leem-se iguais na tela** — *nada seleccionado* · *algo
seleccionado que não é osso* · *é osso, e a secção está fora da dobra*. A linha de diagnóstico do
`PH2D_BONE_LOG` foi escrita para separar as duas primeiras, e a **ausência** dela na corrida que o
dono colou é que apontou para a terceira: a secção tinha sujeito o tempo todo.

⭐ **A cura é REVELAR-AO-FOCAR, e é a lei que a timeline já segue** — *«seleccionar um objecto NOVO
leva a timeline à aba Keys»* (Enio, 2026-07-22). *Uma superfície que só existe fora da dobra é uma
superfície que o artista descobre por acaso.*

O caminho tem três peças, e o corte entre elas é **quem sabe o quê**:

| peça | pergunta que ela responde | porquê ali |
|---|---|---|
| [`skeleton_reveal::on_focus`](../../shells/desktop/src/skeleton_reveal.rs) | *entrou um osso NOVO em foco?* | é a **ARESTA**; a shell é quem vê a selecção |
| `state_bone::set_reveal_bone_section` | o pedido atravessa shell → painel | a mesma porta única dos outros factos do osso |
| [`paint_bone::reveal_section`](../../crates/ph2d-panel-vector/src/paint_bone.rs) | *o cabeçalho está fora da faixa? então rola* | é o **único** sítio com a banda visível e o `y` do cabeçalho ao mesmo tempo |

⚠️ **A aresta, nunca o estado.** Pedir a revelação em todo quadro com um osso em foco prenderia o
painel: o artista rolava e o quadro seguinte puxava-o de volta, para sempre. E largar o osso
**esquece-o**, para que re-escolher o mesmo osso revele outra vez — que é literalmente o gesto do
report.

⚠️ **Um cabeçalho JÁ à vista fica onde está**, senão a cura seria um salto por clique.

⚠️ **Escreve-se o ALVO (`set_panel_scroll`), nunca o vivo** — o substrato de rolagem suave interpola
até lá, e a secção **desliza** para dentro em vez de saltar.

⚠️ **O `y` que atravessa é de ECRÃ** (o corpo é pintado deslocado por `−scroll`), e o alvo é
`rolagem + (y − topo)`. ⛔ Não se converte para espaço de conteúdo em sítio nenhum: *duas conversões
da mesma grandeza é como um número passa a significar outra coisa sem ninguém dar por isso.*

**Gates** (`the_skeleton_section_comes_into_view.rs` + os dois do `skeleton_reveal`), os quatro
mortos por mutação:

| gate | mutação que ele mata |
|---|---|
| `a_bone_in_focus_brings_the_skeleton_section_into_view` | o pedido deixa de guardar o `y` ⇒ o cabeçalho fica em `1394` |
| `a_section_already_in_view_is_left_where_it_is` | tirar o guarda «já está à vista» ⇒ o cabeçalho salta `154 px` |
| `only_a_new_bone_asks_for_the_section` | tirar a memória ⇒ o mesmo osso pede em todos os 600 quadros |
| `letting_go_asks_for_nothing_and_arms_the_same_bone_again` | a memória não se limpa ⇒ o gesto do report não repete |

⚠️⚠️ **A 1.ª redacção do 2.º gate era VÁCUA e passava**: ela punha o cabeçalho no **fundo** do
documento, onde o `clamp` engole a correcção que o gate proíbe e ela vale exactamente zero. A
fixtura tem de estar longe das **duas** bordas do que a rolagem alcança — *uma fixtura que não
produz o fenómeno deixa o gate verde sobre a mutação que o devia matar.*

⛔ **A alternativa foi pesada e não escolhida:** reordenar as secções para a SKELETON subir quando
tem sujeito. Ela cura o mesmo report e muda a ordem do painel para **toda** ferramenta e todo
objecto — é decisão de produto, não de correcção, e a revelação é a metade pequena e já
precedentada.


### F3-h — ✅ As alças do osso **pegam onde são pintadas** (achado da auditoria, 2026-09-08)

⛔⛔ **O defeito:** o arco de limite — e a alça da força, e a ponta da corrente — é **pintado e
ACENDE sob o rato nos 14 modos da ferramenta de vector**, e o `Down` só era lido dentro do
`DrawMode::Bone`. O artista via o triângulo acender, arrastava, e não acontecia nada.

⚠️ **E a assimetria era DELIBERADA dos dois lados, em sítios diferentes:** o
`refresh_bone_hover` **não** se gateia pelo modo (com a razão escrita: *«os ossos desenham-se em
TODO modo da ferramenta de vetor, então o realce deles tem de existir onde eles existem»*) e o arm
vivia dentro do bloco do modo Osso. *Duas decisões certas, cada uma no seu ficheiro, somam um
controlo morto.*

⭐ **A cura é uma LINHA, e ela é o VERBO, não a alça**
([`bone_gesture::grabbable_outside_bone_mode`](../../shells/desktop/src/bone_gesture.rs)):

| alça | pega fora do modo Osso? | porquê |
|---|---|---|
| **Influence** (a força) | ✅ | nenhuma outra ferramenta exprime este verbo |
| **LimitMin / LimitMax** (as paredes) | ✅ | idem |
| **Tip** (a ponta da corrente, IK) | ✅ | idem |
| **Body** (girar) | ⛔ | o gizmo de sprite já GIRA |
| **Joint** (deslocar) | ⛔ | o gizmo de sprite já DESLOCA |

⛔ Roubar `Body`/`Joint` aqui trocaria a lei do arrasto da seta **em silêncio**. E é um `match`
exaustivo, não uma lista: uma parte nova é **erro de compilação** exactamente no sítio onde alguém
tem de responder *«este verbo existe noutra ferramenta?»*.

⛔ **Dentro do modo Osso o arm novo NÃO corre** — lá a `bone_gesture::press` já decide, e ela
distingue *Criar* de *Transformar*: em *Criar*, pousar sobre uma alça só ACENDE o osso, que é o
desenho e não um esquecimento.

⚠️⚠️ **E a cura quase entrou com um defeito PIOR que o que curava:** o `Up` que liberta a alça vivia
**dentro** do bloco `vector_tool_active() && modo != Select`. Com o arm a correr em todo modo, no
**Select** o slot era agarrado e **nunca largado** — *o osso seguiria o rato para sempre, sem botão
nenhum apertado*. ⇒ o `Up` mudou-se para a família dos irmãos independentes de modo (o da alça de
ligação, o da ficha do padrão, o da âncora do motion path). **Lei:** *um slot de arrasto é largado
onde quer que possa ser agarrado.*

**Gates** (4, todos mortos por mutação):

| gate | mutação que ele mata |
|---|---|
| `only_the_verbs_no_other_tool_can_express_are_grabbed_outside_bone_mode` | a linha aceita `Body`/`Joint` ⇒ a seta perde o arrasto |
| `the_bone_handles_are_grabbed_before_the_tool_takes_the_canvas` | o arm volta para dentro do bloco ⇒ o Select volta a acender e não pegar |
| `the_bone_handle_arm_consumes_the_press` | tirar o `return;` ⇒ o Select abre um marquee por cima do arrasto |
| `the_bone_handle_is_released_in_every_mode_it_can_be_grabbed_in` | o `Up` volta para dentro do bloco ⇒ **o osso segue o rato para sempre** |


### F3-i — ✅ *Remove Smart Bone* **devolve a pose autorada** (achado da auditoria, 2026-09-08)

⛔⛔ **O defeito:** o verbo só tirava o componente, e o objecto ficava **assado no instante da acção**
em que o controlo o tinha deixado. É letra por letra o report de 2026-09-07 sobre o *Remove IK* —
*«não funciona plenamente»* — noutro verbo, e contradiz a lei que este módulo escreveu: *o que um
motor escreve vê-se, não se guarda*.

⚠️ **A `settle` NÃO serve** (a mesma nota do `skeleton_goal::remove`): ela é para um motor que
**largou**, e aí o vivo *é* o documento; aqui o motor foi **DESLIGADO**, e o vivo é dele.

⭐ **A cura é a porta [`skeleton_smart::remove`]**, irmã da do *Remove IK* — e o preço é a diferença
entre as duas: a âncora larga uma **corrente** que ela sabe nomear; um controlo larga o que a
**ACÇÃO** dele anima, que é `N` objectos × os quatro factos que uma curva escreve.

⛔ **Largar «tudo o que o ledger tem» apagaria a reprodução da própria timeline**, que partilha
aqueles motores — por isso a lista sai do **clip deste controlo**, e de mais nada.

⚠️⚠️ **E isso obrigou a NOMEAR uma lista que até aqui só existia como código:**
`timeline_preview::DRIVERS`, os quatro factos que uma curva conduz. O `declare_timeline_writes`
**não a pode iterar** (cada facto tem o seu tipo de payload), então quem os ata é um gate que corre
os dois e compara — e é **ali** que o quinto facto em falta (o `VecDrivenStyle`, item aberto da
auditoria) vai aparecer, em vez de num report em que tirar um controlo deixa a cor assada.

**Gates** (3, todos mortos por mutação):

| gate | mutação que ele mata |
|---|---|
| `removing_the_control_gives_the_authored_pose_back` | o verbo volta a só tirar o componente ⇒ a pose fica assada |
| `removing_one_control_does_not_release_what_another_engine_drives` | ele larga tudo ⇒ a reprodução da timeline morre num clique |
| `the_timeline_drivers_list_is_what_the_declaration_writes` | a lista perde um facto ⇒ esse fica assado, calado |

⚠️ **O `skeleton_smart_tests.rs` estourou o teto de 600 LOC e foi CORTADO por responsabilidade**
(nunca isentado): a **LISTA que o painel pinta** — quais acções o controlo oferece, e o que uma
POSIÇÃO nela significa — mudou-se para `skeleton_smart_list_tests.rs`; o que fica mede *o controlo
PERCORRE a acção*. A fixtura é a **mesma** de propósito: duas cenas para o mesmo módulo divergiriam.

⚠️ **FLAKE DE CARGA, membro NOVO da família do §5.0** — apanhada no fecho desta wave e a promover no
handoff: `the_cache_makes_a_preview_frame_cost_the_tail_not_the_stroke`
([`flip_fit_cache_tests.rs`](../../crates/ph2d-app-flip/src/fit_cache_tests.rs)). ⚠️ O ficheiro é
**novo** para a lista — os três membros já catalogados vivem no `flip_fit_budget_tests.rs`. As três
assinaturas: gate de RAZÃO de custo · **zero** linhas do diff naquela crate · **5 de 5 verde
sozinho**, com `loadavg 42,6–43,5` impresso ao lado de cada corrida (outra linha a compilar nesta
máquina).


### F3-j — ✅ O app **não fica calado** quando um controlo não tem sujeito (auditoria, 2026-09-08)

Dois achados da mesma família — *um controlo que morre em silêncio dá o mesmo sintoma que uma rota
cortada* —, e os dois curados pela mesma ideia: **pendurar a pergunta no FACTO, nunca no verbo.**

**(a) «nenhum osso em foco» cobria `8` de `30` controlos.** O braço que diz isso nasceu com **dois**
verbos (os da âncora), foi a **oito** na wave dos ossos inteligentes, e os **nove campos** e as
**duas fileiras de chips** nunca lá entraram. ⇒ hoje a condição é **DERIVADA das tabelas de ids**
([`ids::needs_focused_bone`](../../crates/ph2d-editor-core/src/ids/chrome/vector_bone.rs)):
acrescentar um controlo põe-no do lado certo sem ninguém se lembrar do braço, e quem age sobre as
**formas** declara-se na única lista à mão que resta (`VECTOR_BONE_ON_SELECTION`, com `3` entradas).

⚠️ **E ela é alimentada pelos DOIS caminhos de evento** — um clique e um campo numérico chegam por
portas diferentes, e a prova de mutação mostrou que alimentar só uma compila limpo e deixa metade da
seção calada outra vez.

**(b) o aviso do osso governado só disparava numa das TRÊS ordens.** Ele vivia dentro do *Add Smart
Bone*, logo respondia por **IK → Smart** e ficava calado nas outras duas: pôr a âncora **depois** do
controlo, e **alargar o `Chain`** até ele. ⇒ a pergunta passou a ser o facto
([`skeleton_smart::governed_controls`](../../shells/desktop/src/skeleton_smart.rs)), perguntado
**depois** dos verbos do quadro, e o aviso sai quando o conjunto **cresce**.

⭐ *Um aviso pendurado num VERBO responde por uma ordem; pendurado no FACTO, responde por todas —
incluindo as que ninguém enumerou* (a terceira ordem não tem verbo de criação nenhum).

⚠️ **Avisa e FAZ na mesma**, ⛔ nunca recusa: tirar a âncora depois é um gesto que existe.

**Gates** (6 — 5 mortos por mutação, 1 com o ponto cego nomeado):

| gate | mutação que ele mata |
|---|---|
| `the_derived_question_covers_the_whole_section` | a derivação perde uma tabela ⇒ aqueles controlos morrem calados |
| `the_selection_exceptions_are_all_real_controls` | a excepção nomeia um controlo que já não existe |
| `a_control_from_another_section_is_not_claimed` | a pergunta reclama o painel inteiro |
| `the_arm_that_speaks_reads_the_derived_question` | o braço volta a uma disjunção à mão |
| `both_event_paths_feed_it` | o caminho dos CAMPOS deixa de alimentar ⇒ metade da seção cala |
| `a_control_governed_by_an_anchor_is_found_in_any_order` | a porta do facto deixa de perguntar |

⚠️⚠️ **UMA MUTAÇÃO SOBREVIVEU, e o ponto cego está escrito no gate:** tirar o *Bind* da lista de
excepções passa. Ele fica a contar como *«precisa de osso em foco»*, o que é falso, e **nada em
`ph2d-editor-core` o pode saber** — um id é um hash, e *de quem é o sujeito* só se lê no DRENO, que
vive na shell. ⇒ o gate mede **cobertura**, que era a falha real (`8` de `30`), e diz isso de si
mesmo. *Um gate que se diz partição e mede cobertura mente sobre a metade que ele não vê.*

⛔⛔ **E o portão apanhou um vermelho que TRÊS corridas desta linha não viam:** o `clamp` cru da cura
F3-g reprovou o `clamp_calls_in_ui_are_safe_or_justified` — que vive na `ph2d-editor-core` e varre a
`ph2d-panel-vector`. *Correr a suíte da crate EDITADA é cego aos gates de arquitectura que moram
noutra.* Curado pela porta da casa (`math::safe_clamp`), nunca por `// CLAMP-OK`.


### F3-k — ✅ Dois clips com o MESMO NOME, e o quinto motor do desfazer (auditoria, 2026-09-08)

Os dois últimos itens abertos da auditoria. **Um era um defeito e curou-se na PORTA; o outro
DISSOLVEU-SE com a medição.**

**(a) dois clips podiam partilhar um NOME, e o controlo percorria o primeiro, calado.**
⚠️ **A lei já estava escrita no ficheiro, duas vezes** — *«two clips sharing a label make the
dropdown unreadable and the rename ambiguous»* — e era honrada nas duas portas que **INVENTAM** um
nome (`fresh_clip_name`, `fresh_copy_name`); as duas que **RECEBEM** um do chamador (`add_clip`,
`rename_clip`) aceitavam qualquer coisa.

⭐⭐ **E então chegou um terceiro leitor que torna a ambiguidade SILENCIOSA em vez de apenas feia:**
um osso inteligente guarda o **NOME** do clip (a referência durável desta casa), então dois chamados
`"Wave"` fazem o controlo percorrer **o primeiro** — o artista configura o segundo e lê o app como
avariado. *Uma ambiguidade que só ficava feia num dropdown vira uma resposta errada no instante em
que alguém referencia por nome.*

⇒ **uma porta só** (`TimelineDoc::unique_clip_name`), com as quatro a passarem por ela; renomear um
clip para o nome que ele **já tem** é um no-op, nunca `"Wave 2"`. ⛔ Avisar em vez de coagir deixaria
o artista com um documento que ele não consegue reparar renomeando (os dois nomes são igualmente
válidos), que é a forma de uma recusa com passos extra.

**(b) *«o ledger cobre 4 das 5 escritas do `write_prop`»* — ⛔ RECUSA MEDIDA, não trabalho.**
A quinta é a opacidade de um caminho vectorial (`VecDrivenStyle`), e ela tem **duas** protecções que
juntas são mais fortes que o ledger:

| protecção | o que ela compra |
|---|---|
| **desregistada, e não por esquecimento** — não deriva `Serialize`, e o `register_default` exige-o ⇒ *uma linha de registo não compila* | nunca entra no snapshot, no save, nem num passo de undo — que é exactamente o que o ledger compra para os outros quatro, que **são** registados |
| **volta ao autorado TODO QUADRO** (`settle_to_authored`) | resíduo de **um** quadro no máximo, contra o `release_to_authored`, que só corre quando um motor é DESLIGADO |

⛔ Acrescentá-la ao ledger seria pôr no memo um facto que nunca esteve na fotografia.

⚠️⚠️ **O que PODE regredir em silêncio é a ORDEM DO QUADRO, e ela não tinha gate nenhum:** repor o
autorado **antes** de a projecção ser lida apaga a curva **deste** quadro, e a forma deixa de
desvanecer com a suíte inteira verde.

**Gates** (4, todos mortos por mutação):

| gate | mutação que ele mata |
|---|---|
| `two_clips_never_share_a_name` | a porta deixa de coagir · o rename colide consigo mesmo |
| `a_copy_keeps_its_own_idiom_through_the_same_door` | a cópia perde o idioma `"Walk copy"` |
| `the_driven_style_settles_after_the_frame_has_read_it` | o `settle` sobe acima do `resolve` ⇒ o fade morre, calado |

⇒ **os SEIS itens da auditoria de 2026-09-08 estão fechados.**


### F3-l — ✅ *«Remove Smart Bone não devolve o objeto animado à posição inicial»* (report, 2026-09-09)

⛔⛔ **O report do dono é a ponta de um defeito MAIOR, e ele vale para a âncora de IK também.**

**A medição, antes de qualquer dedução** (o mesmo controlo, o osso PARADO, um quadro de cada vez com
a `settle` que a shell corre em todos):

| quadro | `x` do objecto | o ledger conduz? |
|---|---|---|
| 0 | 10,000 | **sim** |
| 1 | 10,000 | **não** |
| 2..4 | 10,000 | **não** |
| *depois do `Remove`* | **10,000** | — (autorado: `0,000`) |

⇒ **o ledger largava o objecto no quadro 1**, e o verbo já não tinha o autorado para devolver.

⭐⭐⭐ **A causa é uma lei CERTA aplicada a um motor de outra espécie.** Quem declara só o que
**MUDOU** está certo para a **timeline**: uma reprodução que pára tem de deixar o valor virar
documento, e é isso que faz *«desfazer a corrida»* ser **um** passo. Mas um **CONDUTOR PERSISTENTE**
— uma âncora de IK, um osso inteligente — nunca «pára»: ele escreve todo quadro, e o output dele é
**constante na maior parte do tempo**. A `settle` lê essa constância como *«o motor largou»*.

⇒ *para um condutor persistente, **«não mudou»** e **«acabou»** são factos diferentes com a mesma
forma* — e a porta que os separa é [`PreviewDrive::still_driving`](../../shells/desktop/src/preview_drive.rs).

⚠️⚠️ **E o defeito é MAIOR que o verbo:** com a pose promovida a documento, ela entra no **undo** e
no **save** — que é exactamente o que o `preview_drive` existe para impedir. O *Remove* era só onde
ele se via.

⛔⛔ **A ÂNCORA DE IK tinha o mesmo buraco** (`solve_one` saltava a declaração quando a rotação não
mudava, e uma corrente assente é o estado normal dela) — curado na mesma wave, com gate próprio.

⚠️⚠️⚠️ **E esta cura foi CONSTRUÍDA E REVERTIDA em 2026-09-07, com razão.** O doc do
`posing_a_governed_bone_by_hand_is_not_swallowed_by_the_ledger` regista-o: *«construí a cura
(declarar a condução todo quadro) e ela ficou verde… e o desenho ORIGINAL também ⇒ a cura era
redundante e foi revertida»*. **Ela era mesmo redundante — para AQUELA pergunta** (*a pose que a
outra mão faz é engolida?*), que a regra da outra mão já resolvia. *Uma recusa medida responde UMA
pergunta*, e a de agora é outra: *o autorado ainda está lá quando o motor é DESLIGADO?*

⛔⛔ **E as cinco fixturas destes gates mediam outro programa: nenhuma chamava a `settle`.** O gate
`removing_the_control_gives_the_authored_pose_back` corria `drive` **uma vez** e declarava a cura de
ontem verde. *Uma fixtura que não corre o quadro do artista mede outro programa* — a **5.ª** vez
nesta linha, e a primeira em que o dono a apanhou antes do gate.

**Gates** (4, todos mortos por mutação):

| gate | mutação que ele mata |
|---|---|
| `removing_the_control_gives_the_authored_pose_back` (agora com 5 quadros + `settle`) | o controlo deixa de dizer que ainda conduz |
| `what_the_action_writes_is_preview_not_document` (idem) | a mesma — e ali o sintoma é o **undo**, não o verbo |
| `a_settled_chain_is_still_driven_and_remove_still_has_the_authored_pose` | a âncora deixa de dizer que ainda conduz |
| `still_driving_keeps_what_exists_and_invents_nothing` | a porta **INVENTA** uma entrada ⇒ o `release` passa a carimbar a pré-visualização como documento |

⚠️ **O `skeleton_goal_tests.rs` estourou o teto de 600 LOC e foi CORTADO por responsabilidade**
(nunca isentado): o que a âncora faz ao **DOCUMENTO** mudou-se para `skeleton_goal_ledger_tests.rs`;
o que fica mede a **LEI** que ela resolve por quadro.

⚠️ **Flake de carga confirmada no fecho:** `the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke`
(família `orcamento`, já catalogada no §5.0) — **5 de 5 verde sozinha** a `loadavg 17,0–17,5`, com
zero linhas do diff naquele ficheiro.


### F3-m — ✅ **O ESQUELETO TEM PAINEL PRÓPRIO** (escolha do dono, 2026-09-09)

⭐⭐⭐ **A medição que pôs a escolha na mesa.** Com **só um osso** seleccionado, o painel de vector
pinta, acima da secção Skeleton:

| secção acima | espaço | um osso tem isso? |
|---|---|---|
| Traço | 422 px | não |
| Encaixe | 246 px | é do desenho |
| Mistura | 188 px | não |
| Preenchimento | 88 px | não |
| Morph | 87 px | não |

⇒ **785 px de coisas que um osso não tem**, empurrando o cabeçalho para `y = 1394` sobre uma faixa
visível de `774`. *Era esta a raiz do report de 08/09* — o revelar-ao-focar tratou o sintoma.

⚠️ **As três saídas foram postas ao dono com o preço de cada uma** (esconder o que não serve ·
painel próprio · deixar como está) — a primeira estava perto de uma recusa dele (*a cura «esconder»
das faces do balde foi construída e revertida por ordem dele*). **Ele escolheu o painel próprio.**

**O que a wave fez, e a ordem importa:**

| passo | porquê ele veio primeiro |
|---|---|
| `ph2d_editor_core::panel::RowCtx` | o vocabulário de linhas passou a ter **dois** hospedeiros; duplicar ~200 linhas duplicaria o **RITMO**, e duas cópias divergem no primeiro vão que alguém mexe |
| `ph2d-panel-skeleton` | crate nova: `state` · `section` · `paint` · `seam`, com os controlos movidos |
| a shell | publica os factos para o painel novo, **decide a visibilidade**, e traz a aba à frente |

⚠️ **Os ids NÃO foram renomeados.** Um `NodeId` é o hash de uma STRING: `vector.bone.*` continua a
ser o endereço de cada controlo, e o que mudou é **quem os pinta**. Renomeá-los quebraria o registo,
o encaminhamento e os gates — e o ganho seria estético.

⚠️ **Os 147 chamadores do painel de vector não mudaram**: o `BodyCtx` manteve os métodos e passou a
**delegar** por uma porta (`with_rows`). *Uma extracção que obriga 147 sítios a mudar de nome no
mesmo commit é uma extracção que colide com toda linha viva.*

⚠️⚠️ **DUAS leis mudaram de DONO, e as duas levaram o gate atrás:**

| lei | dono antigo | dono novo |
|---|---|---|
| *a secção some numa cena sem ossos* | a própria secção | a **shell** (`panel_visible`), gate `the_shell_opens_it_from_the_scene_and_from_the_tool` |
| *um osso novo REVELA* | rolagem da secção | a **aba** (`bump_panel_z`), gate `a_new_bone_in_focus_brings_the_tab_forward` |

⛔⛔ **E o «revelar-ao-focar» DISSOLVEU-SE como rolagem** — não foi apagado por gosto: neste painel o
cabeçalho é a **primeira** linha e o corpo mede ~620 px numa coluna de 836; *não há dobra abaixo da
qual esconder-se*. A **lei da aresta sobreviveu inteira** (`skeleton_reveal::on_focus` e os dois
gates dela) e passou a comandar a aba. *A pergunta era boa; o que ela comandava é que era do desenho
antigo.*

⚠️ **O pill do modo Osso ficou no painel de VECTOR** — ele é o selector de **ferramenta**, e o que
se mudou foram os controlos do OSSO, não o modo que os cria.

⛔⛔ **TRÊS vermelhos que só a árvore inteira tinha**, e nenhum deles no que a wave editava:

| gate | o que ele apanhou |
|---|---|
| `every_registered_panel_is_reachable_by_the_z_order_walk` | o painel nasceu **registado, visível e nunca pintado** — a guarda que este repo construiu exactamente para isto |
| `cargo_*_in_sync_with_folder` | eu editei **à mão** blocos que o `ph2d-panel-sync` **gera** |
| `workspace_src_files_under_loc_cap` | o `doc.rs` da timeline passou 700 com a porta dos nomes de ontem |

⚠️⚠️ **E a 1.ª redacção dos dois gates novos era VÁCUA: as três mutações SOBREVIVERAM.** Eles
procuravam os nomes numa **janela de bytes** à volta da chamada — e as linhas que os DECLARAM caem
dentro da janela, então tirar um do ARGUMENTO não movia nada. ⇒ a âncora passou a ser a **linha do
argumento**. *Um gate que mede a vizinhança de um nome mede o nome, não a origem* — a mesma família
que mordeu o gizmo do limite em 08/09.


### F3-n — ✅ **O pill saiu, a fileira virou PORTA, e o menu Window ganhou *Bones*** (2026-09-09)

Quatro pedidos do dono no mesmo report, e o terceiro é o que muda quem manda:

| pedido dele | o que foi feito |
|---|---|
| *«vc deixou o botão Bones no Painel Vector — melhor tirar de lá»* | o pill saiu da fileira de modos |
| *«mudar o modo de criar e transformar bones»* | os dois segmentos passaram a **trocar o modo E o verbo** |
| *«ao selecionar o osso o painel é aberto e o botão Transform é selecionado»* | as **três** metades saem da mesma aresta |
| *«se não há ossos … nenhum botão fica selecionado até apertar Create»* | nada acende sem estar armado |
| *«o Menu Windows deve receber a opção de Bones»* | *Window → Bones* |

⭐⭐⭐ **O pill e o menu são o mesmo pedido de dois lados.** Tirar o pill tira a **única** porta para o
`DrawMode::Bone` ⇒ os segmentos *Create* / *Transform* tornaram-se a entrada. E a visibilidade
derivada da cena (*«há ossos?»*) não tinha porta nenhuma numa cena **sem** ossos — que é exactamente
onde o artista quer carregar em *Create*. ⇒ *uma feature cuja única porta é já ter o que ela produz
não tem porta.*

⛔⛔ **E foi por isso que a visibilidade DEIXOU de ser escrita em todo quadro.** Com
`tem_esqueleto || ferramenta_osso` a correr sempre, a linha do menu seria um interruptor **morto** —
o quadro seguinte repunha a decisão da shell por cima da do artista. *Duas fontes de verdade para o
mesmo bool, e a que o artista toca é a que perde.* ⇒ ficam **duas portas, as duas de ARESTA**: a
linha do menu e a selecção de um osso.

⚠️ **O «nenhum aceso» não é um terceiro estado do enum:** «armado» já é o modo Osso estar na mão, e
o `bone_tool()` que atravessa é `Option<usize>`. Um terceiro valor diria a mesma coisa duas vezes.

⚠️ **A ferramenta arma-se no quadro SEGUINTE** (`bone_arm_pending`): na aresta do foco o `gfx` já
está emprestado a `sim`/`hero`. O **espelho** da shell escreve-se já, para o mesmo quadro rotear
certo — e o consumo corre **antes** de a ferramenta republicar o espelho, senão ela reverteria a
escrita da aresta.

⚠️ **O painel perdeu o cabeçalho de secção**: com painel próprio, o título dele **é** o cabeçalho, e
o gate `the_tab_and_the_menu_call_a_panel_the_same_thing` obrigou a escolher **um** nome — *Bones*,
o do dono. A conta das secções do painel de vector **DESCEU** pela primeira vez (`42 → 41`).

**Gates** (8; 7 mortos por mutação, 1 é censo):

| gate | mutação que ele mata |
|---|---|
| `nothing_is_lit_until_the_artist_arms_it` | um default aceso ⇒ o painel afirma um verbo que a ferramenta não tem |
| `the_create_transform_group_is_the_door_and_starts_with_nothing_lit` | a fileira volta a só existir dentro do modo ⇒ não há porta |
| `the_two_bone_segments_are_the_door_to_the_mode` | os segmentos armam o verbo e não entram no modo ⇒ **meio gesto** |
| `every_mode_button_reaches_the_tool` (o censo `SEM_PILL`) | um modo sem pill e sem o endereço da porta que o alcança |
| `nothing_writes_the_visibility_every_frame` | a visibilidade volta a ser escrita sempre ⇒ o menu morre |
| `the_focus_edge_opens_raises_and_arms` | falta **uma** das três metades da aresta |
| `every_painted_menu_row_is_registered_and_therefore_clickable` | a linha do menu nasce muda |
| `every_topbar_verb_has_a_door_that_is_not_the_legacy_key` | a linha sai do menu |

⚠️⚠️ **E a 1.ª redacção do gate da aresta reprovou sobre produto CORRECTO**: ele procurava nas `12`
linhas seguintes, e a terceira metade estava lá — atrás de um bloco de nota de quatro linhas. *Num
ficheiro em que a nota é metade do texto, contar LINHAS é contar prosa.* ⇒ a janela passou a contar
**linhas de código**.


### F3-o — ✅ **O PARENTESCO É A PONTA, e um osso RECÉM-NASCIDO não arma *Transform*** (report, 2026-09-09)

> *«Cada vez que se cria um osso o modo Transform é selecionado. Corrija.»*
> *«Atualmente ao criar ossos quando se clica no canvas um osso é criado como filho do osso
> selecionado. Desse modo não se pode criar ossos fora da cadeia de ossos. Vamos mudar: Para criar
> um osso como filho de outro o clique deve acontecer na ponta do osso pai e o usuário arrasta o
> mouse definindo tamanho e direção do novo osso.»*

**São DOIS defeitos numa família só: *«o foco mudou»* tem duas causas que se leem iguais.**

#### (a) O recém-nascido acordava a aresta

A aresta do F3-n faz **três** coisas ao ver um osso NOVO em foco (abrir · trazer a aba · armar
*Transform*), e o osso que acaba de nascer **fica aceso** — é assim que o artista vê qual é. ⇒ o
quadro seguinte lia isso como *«o artista escolheu um osso»* e trocava-lhe o verbo debaixo da mão.

⚠️ **A aresta nunca teve defeito.** *«O foco mudou»* tem duas causas — *o artista apontou* e *o gesto
produziu* — e **só quem produziu as distingue**. ⇒ quem cria o osso **alimenta a memória**
([`skeleton_reveal::on_birth`](../../shells/desktop/src/skeleton_reveal.rs)), e a aresta seguinte não
tem nada a relatar. ⛔ **Não é uma isenção nem um sinalizador**: a memória continua a ser o único
estado da lei, e a porta escreve exactamente o que o `on_focus` escreveria.

#### (b) O parentesco era a SELECÇÃO, e isso tornava uma RAIZ inexprimível

A lei antiga: `origin = tip_of(osso aceso).unwrap_or(world)` — a ponta do osso aceso, **sempre**.
Com um esqueleto na cena não havia press nenhum que fizesse uma raiz nova, que é o report à letra.

⇒ a pergunta passa a ser **geométrica e local**, por uma porta única
([`bone_gesture::tip_at`](../../shells/desktop/src/bone_gesture.rs)):

| onde o press cai | o que nasce |
|---|---|
| na **ponta** de um osso (raio = a bolinha DESENHADA, `joint_radius_px`) | **filho** dele, com a origem encaixada nela |
| em qualquer outro sítio — vazio, ou **em cima do corpo** de outro osso | **raiz**, na origem apontada |

⭐ **Ramificar do meio de uma corrente passa a ser o mesmo gesto que continuá-la** — a espinha que dá
dois braços era, na lei antiga, *escolher o osso do meio na Hierarquia e voltar ao canvas*.

⛔ **A `BonePress::Select` MORREU.** Ela existia só para servir a lei antiga (o doc dela dizia-o: *«é
assim que se escolhe onde ramificar»*), e mantê-la seria pior que redundante: em *Criar*, trocar a
selecção acorda a aresta do F3-n e arranca o artista do verbo em que ele está — o defeito (a) por
outra porta.

⭐⭐ **E o `hover` passou a ser MODAL, porque o clique é:** em *Criar* existe **um** alvo, a ponta.
⛔ Sem isso o realce acenderia o osso **errado** exactamente no ponto que decide o parentesco — numa
corrente contínua a ponta do osso `k` é a raiz do `k+1`, e o realce de *Transformar* responde ali
*«a JUNTA do `k+1`»*. E no canvas, em *Criar*, o anel passa a ser desenhado na ponta de **todo** osso
(em *Transformar* ele é o *end effector*, logo só em quem fecha a corrente e não tem âncora):
*a mesma alça a dizer o que o clique faz AGORA*.

⚠️ **O pai viaja no `BoneBirth` do press até ao release.** Guardar só a origem e ir buscar o pai à
selecção no `Up` — a 1.ª redacção — é a lei que o dono mandou tirar **sobrevivendo no outro extremo
do gesto**; há gate de AUSÊNCIA sobre isso.

**Gates** (6, todos mortos por mutação):

| gate | mutação que ele mata |
|---|---|
| `a_newborn_bone_asks_for_nothing` | o `on_birth` não escreve ⇒ criar um osso arma *Transform* |
| `the_bone_creation_site_absorbs_the_focus_edge` | a lei existe e **não está no caminho** de quem cria |
| `only_a_press_on_the_tip_makes_the_new_bone_a_child` | o `press` volta a ler a selecção ⇒ nenhuma raiz nova |
| `the_release_never_asks_the_selection_who_the_parent_is` | o pai re-derivado no `Up` ⇒ a lei antiga volta por trás |
| `in_create_the_hover_lights_exactly_the_bone_the_click_would_branch_from` | o realce acende o filho e o press ramifica do pai |
| `when_two_tips_are_under_the_finger_the_nearest_one_wins` | a ponta escolhida por ORDEM ⇒ a do osso curto fica inalcançável |

#### O tecto de LOC mordeu, e a cura foi um CORTE por responsabilidade

`bone_gesture.rs` chegou a `669` e o ficheiro de gates a `693`, contra o tecto de `600` da shell.
⛔ Nenhuma isenção: o módulo partiu-se em **[`bone_gesture`]** (*o gesto que FAZ um osso* — `create`,
`BonePress`/`BoneBirth`, `press`, `aim_rotation`, `reach_chain`, `drag_makes_a_bone`) e
**[`bone_pick`]** (*o que o PONTEIRO significa* — `BONE_HIT_PX`, `hit`, `tip_of`, `tip_at`,
`grabbed_the_joint`, `hover`, `grabbable_outside_bone_mode`, e as duas portas do `App`), com os
gates a acompanhar a lei que medem (`bone_pick_tests.rs`). Ficheiros: `335` · `353` · `377` · `347`.

⚠️ **As duas metades continuam a ser UMA lei** — o `press` pergunta ao `hover`, e o gate que os
compara ponto a ponto atravessa o corte de propósito.

⚠️⚠️ **A 1.ª redacção do último deixou a mutação SOBREVIVER, e a razão é a terceira leitura: a
fixtura não produzia o fenómeno.** Medido: o `Entity::to_bits` **DESCE** a cada entidade nova
(`4294967294`, depois `4294967293`) e o `bone_segments` ordena por bits **crescentes** ⇒ *o último
osso criado vem primeiro na lista*. Com o osso curto criado por último, «o primeiro dentro do raio»
e «o mais perto» davam a mesma resposta. ⇒ o gate corre agora **as duas ordens de criação**.


### F3-p — ✅ **DUAS CORRENTES SEPARADAS VIRAM UMA** (ordem do dono, 2026-09-09)

> *«Torne possível criar uma cadeia de ossos a partir de dois ossos separados ligando a ponta de um
> com o fundo de outro ao criar um osso intermediário.»*

**O gesto:** press na **ponta** de `A` · arrasta · solta na **base** de `B`. Nasce o osso do meio,
encaixado nos dois pontos, e a corrente de `B` passa a pendurar-se nele. `A → novo → B`.

⭐⭐ **É o ESPELHO exacto da lei do F3-o, e é isso que o torna barato:** ali *a ponta decide de quem o
osso novo é FILHO* (o press); aqui *a base decide quem vira filho DELE* (o release). Duas metades
independentes do mesmo gesto, cada uma um sítio na tela.

| porta | o que responde |
|---|---|
| [`bone_pick::tip_at`] | de quem o osso novo é filho (press) |
| [`bone_pick::free_root_at`] | quem o osso novo adopta (release) |
| [`bone_gesture::splice_target`] | a emenda **legítima** — a base solta, sem laço |
| [`bone_gesture::drag_now`] | o que se DESENHA e o que o release vai FAZER |
| [`bone_gesture::connect`] | pendura sem mover |

⛔ **Só bases SEM PAI-OSSO se oferecem.** A base de um osso do meio **É**, no mesmo pixel, a ponta do
pai dele — e ali a lei da ponta já fala. *Uma base que já tem dono não está livre para ser adoptada,
e dois verbos num pixel é o defeito que o F3-o veio curar.*

⚠️ **A pose de MUNDO do adoptado NÃO muda** — ele é uma corrente que o artista já posicionou, e um
`ChildOf` cru somaria a pose do osso novo (o esqueleto inteiro saltaria). A porta que devolve a pose
local certa já existia (`vec_transform::reparent_keeping_world`, a mesma da Hierarquia) e o
`RootOrder` sai com o adoptado, que deixou de ser raiz.

⛔ **A recusa do LAÇO vive no `splice_target`, não no `connect`**, e a diferença é observável: ali o
alvo simplesmente **não existe**, logo a pré-visualização não encaixa; no `connect` o desenho
prometeria a emenda e o release entregaria um osso sem ela. (Uma travessia de `Transform` sobre uma
árvore cíclica **não devolve** — não é defeito cosmético.)

**Gates** (10 nesta wave, todos mortos por mutação):

| gate | mutação que ele mata |
|---|---|
| `only_a_chain_that_starts_free_offers_its_base_for_a_splice` | a base do meio da corrente oferece-se ⇒ dois verbos num pixel |
| `two_separate_chains_become_one_and_the_adopted_one_does_not_move` | `ChildOf` cru ⇒ o esqueleto adoptado salta |
| `the_adopted_root_stops_being_a_root` | o `RootOrder` fica num osso que já não é raiz |
| `splicing_refuses_the_loop_that_would_hang_the_app` | o laço passa ⇒ a travessia de pose não devolve |
| `what_is_drawn_is_the_bone_that_will_be_born` | a ponta não encaixa, ou o limiar mede o ponteiro |
| `the_release_splices_through_the_same_door_the_preview_asked` | o desenho e o release resolvem por si |

⚠️⚠️ **UMA MUTAÇÃO SOBREVIVEU e foi ela que ditou o desenho final.** A 1.ª redacção resolvia as três
coisas — a emenda, a ponta encaixada e o limiar — **duas vezes**: uma na pré-visualização, outra no
release. Apagar o encaixe do lado do DESENHO deixava a suíte inteira verde, e o produto ficava com o
defeito mais caro desta família: *o osso acaba no cursor na tela e nasce na bolinha.* ⇒ nasceu o
`BoneDragNow`: **uma leitura, dois consumidores**, com gate de costura a provar que os dois a leem.

⚠️ **E a fixtura do gate novo não produzia o fenómeno à primeira**: a bolinha da base encolhe com o
comprimento (`joint_radius_px = min(12, comp/4)`), e um osso de `10` oferece um alvo de `2,5` — o
ponteiro estava a `3,6`. *O raio do alvo é do DESENHO, e uma fixtura que não o calcula mede outra
coisa.*


### F6 — ✅ **A SEGUNDA MÍDIA: uma IMAGEM obedece ao esqueleto** (ordem do dono, 2026-09-09)

> *«Prender desenhos/imagens aos ossos»* · *«faça pesquisa para criar o modo mais intuitivo e
> eficaz de criar e fazer o bind da malha»* · *«quero o estado da arte»* · *«vamos tentar como vc
> recomenda»*

⛔⛔ **A entrada anterior desta linha dizia «bloqueado: precisa de uma malha sobre a imagem, que
não existe — meça o preço antes de prometer».** Ela estava certa sobre o facto e o preço era outro:
duas das quatro peças **já existiam**, e o doc de uma delas dizia-o por escrito.

Pesquisa completa, com as sete ferramentas e as recusas:
[`02_pesquisa_a_malha_sobre_a_imagem.md`](02_pesquisa_a_malha_sobre_a_imagem.md).
Página de leitura do dono: <https://claude.ai/code/artifact/0efd5bdd-c103-4f64-a965-7b863c9a8f5d>

#### O que a pesquisa decidiu

⭐⭐⭐ **O campo inteiro deixou de mandar o artista construir a malha.** Spine (*Create Hull*),
Live2D (*Automatic Mesh Generation*), OpenToonz (*Plastic*) e o **Puppet** do After Effects — que
não tem interface de malha nenhuma — derivam-na do recorte da própria tinta; e os pesos são
automáticos por distância em cinco das sete, com o pincel como **correcção**.

⛔⛔ **E a porta aberta que temos é o contra-exemplo, medido no binário dele:** o Godot (MIT) faz
`Create Internal Vertex` um a um e `Paint Bone Weights` à mão, **sem nenhum automático**. Portá-lo
seria portar o trabalho.

#### O balanço, medido antes de uma linha de código

| | |
|---|---|
| ✅ | **peso automático** — `weights_at`, derivado por distância, sem tabela guardada (`0,146 %` de um quadro) |
| ✅ | **a pele já é agnóstica de mídia** — o doc do `SkinBind::source` dizia *«serve um `VecPath` hoje e uma malha raster amanhã sem uma variante nova nem um schema por mídia»* |
| ⏳ | a malha sobre a imagem |
| ⏳ | desenhar imagem entortada (`draw_image_rgba_transformed` é **um afim por imagem**) |

#### O que foi construído

| porta | pergunta |
|---|---|
| [`ph2d_poly2d`](../../crates/ph2d-poly2d/) (crate nova, zero deps obrigatórias) | onde a tinta acaba · o orçamento do artista · em que triângulos isso se divide |
| `Xform::from_triangle` | o afim que faz a imagem entortar |
| `skeleton_live::bind_image` + `tendons_for` | prender, pela **mesma** lei de tendões das formas |
| `skeleton_skin_image::{pixel_to_local, posed_local, draw_skinned_images}` | a régua da imagem, a pose, e o desenho |
| `sim_extract::skinned_image` | a sprite original **sai do passe**, por facto derivado |

⭐⭐ **Zero capacidade nova de render.** O `push_clip(forma)` já existia, o
`draw_image_rgba_transformed` já existia, e o compositor põe o Vello **por cima** do passe de
sprites: cada triângulo é um recorte mais um afim. Dois triângulos vizinhos concordam nos dois
vértices que partilham ⇒ **a continuidade é consequência, não tolerância**.

⛔ **Não se escreve `Visibility`** para esconder a original: é o olho da Hierarquia, e escrevê-lo
poria a shell a discutir com o artista — *duas fontes de verdade para o mesmo bool, e a que o
artista toca é a que perde*, a lei que o menu *Window* desta mesma linha pagou horas antes.

#### ⚠️ DOIS defeitos que só a fixtura CONTADA achou, ambos no mesmo algoritmo

1. **O critério de paragem do rastreio comparava com a entrada ARTIFICIAL do arranque**, que o laço
   nunca reproduz ⇒ ele **nunca disparava**. Medido: um quadrado de `10×10` deu **3201** pontos em
   vez de 36 — 89 voltas até bater no tecto de segurança. *Um critério de paragem que compara com
   um valor que o laço nunca produz é um laço infinito com cara de algoritmo.*
2. **O `backtrack` do passo seguinte** tem de ser o vizinho que PRECEDE o achado na ordem de Moore,
   e não *«o último vazio que se viu»*. Sintoma: três testes acima de 60 s.

⚠️⚠️ **Os dois só apareceram porque a fixtura pedia o perímetro CONTADO (`36`).** Uma que só
exigisse *«um anel não vazio»* teria ficado verde nos dois casos.

**Gates:** 9 na malha · 2 no afim · 5 na 2.ª mídia · 3 na costura = **19**, com **11** mutações
mortas.

⚠️⚠️ **E a prova de mutação expôs o limite dos gates de TEXTO:** pôr `if false &&` à frente do
*Bind* de imagens **SOBREVIVEU** — o texto continua lá. ⛔ A cura não é apertar a varredura (seria
uma corrida contra o próximo idioma que a desliga): a metade que falta mede-se do outro lado, nos
gates de unidade, e a nota está escrita no cabeçalho do ficheiro. *Um gate de texto responde «isto
está escrito», nunca «isto corre».*

⏳ **ABERTO, e nomeado:** a malha é só o **contorno** (o *ear-clipping* não põe vértices no miolo),
logo um membro grosso dobra pela borda — ⭐ **o dono viu isso na primeira olhada e a F6-b curou-o**;
um **buraco** no meio de uma forma não é traçado, e a malha cobre-o; e o número de triângulos por
quadro **não foi medido sob cena cheia** — a rota por pipeline de triângulos texturados é a
optimização, com razão medida, se a de hoje não couber.


### F6-b — ✅ **A MALHA É UMA GRELHA GRADUADA PELAS ARTICULAÇÕES** (report do dono, 2026-09-10)

> *«a malha criada automaticamente é de péssima qualidade. deveria ser um quadmesh inteligente com
> maior densidade nas áreas das articulações»* (três fotos)

⭐⭐⭐ **Ele tem razão e o número diz quanto — e o número que o explica não é nenhum dos que a F6
mediu.** Medido sobre a MESMA cápsula do smoke, com a mesma régua nos dois lados:

| | contorno (a F6) | grelha (hoje) | grelha **sem** articulações |
|---|---:|---:|---:|
| vértices | `18` | `154` | `69` |
| **no MIOLO** | **`0`** | **`114`** | `48` |
| triângulos | `16` | `252` | `102` |
| aspecto p50 | `17,42` | **`2,30`** | **`2,00`** |
| pior aspecto | `53,10` | `8,25` | `3,56` |

⛔⛔ **O `0` da coluna do miolo é a causa inteira.** Um *ear-clipping* triangula o **contorno**: toda
a deformação tinha de passar pela borda, e as lascas do leque cisalhavam a arte — é literalmente o
que as fotos mostram. ⚠️ **Nenhum dos 9 gates da malha o via**, porque todos perguntavam pela
*silhueta* (o anel fecha · o perímetro conta `36` · a concavidade sobrevive) e **nenhum perguntava
pela DISPOSIÇÃO dos vértices** — *uma malha certa por fora pode não ter nada por dentro.*

⚠️⚠️ **A barra do aspecto NÃO foi escolhida: `2` é o CHÃO.** Um quadrado partido em dois dá dois
triângulos rectângulos isósceles, cuja razão maior-lado/menor-altura é exactamente `2` — e a coluna
da direita, uma grelha uniforme, **lê `2,00`**. Pedir menos seria pedir o impossível a uma grelha.

#### ⚠️ O que «quadmesh» quer dizer aqui, e o que ele NÃO muda

O que a qualidade da deformação pede é a **DISPOSIÇÃO DOS VÉRTICES**. ⛔ O *primitivo guardado* não
pode ser um quadrilátero: o desenho é **um afim por triângulo**, e um afim não leva um quadrilátero
qualquer a outro qualquer (quatro pontos são **oito equações para seis incógnitas**) ⇒ cada célula é
guardada como **dois triângulos**, sempre com a diagonal `a–c` escolhida no **repouso**. ⛔ Escolhê-la
pela célula **deformada** (a mais curta das duas, que é a resposta clássica) faria a malha trocar de
diagonal a meio de um gesto — *o desenho piscaria exactamente enquanto o artista dobra.*

#### ⭐⭐⭐ Porque é uma GRELHA-PRODUTO e não uma quadtree

Os cortes escolhem-se **eixo a eixo** ([`ph2d_poly2d::axis_samples`](../../crates/ph2d-poly2d/src/grid.rs)):
uma lista de `x` e uma de `y`, densas perto das articulações e largas longe. A malha é o produto das
duas, e **ela CONFORMA por construção** — dois vizinhos partilham a aresta inteira, sempre.

⛔ Uma *quadtree* graduada (a resposta «óbvia») deixa **nós pendurados** na transição entre níveis, e
um nó pendurado abre **FENDA** numa deformação: ele move-se pelos pesos dele enquanto a aresta do
vizinho grosso se move linearmente entre as pontas. Curá-los pede a tabela de moldes de transição
(5 casos a menos de rotação) — e a grelha-produto entrega o mesmo adensamento sem nenhum deles.

⚠️ **A fronteira DECLARADA:** a densidade é o produto de dois campos de UMA dimensão, então uma
articulação adensa a **coluna** e a **linha** inteiras dela. Para um membro — que é o caso deste
módulo — é o que se quer: as dobras ao longo de um braço dão colunas finas em cada uma, e as linhas
ficam largas porque o membro é fino de través.

#### ⭐⭐ A malha COBRE a tinta, não segue a silhueta

`expand: 2.0` px — o *Expansion* do *Puppet* do After Effects. ⚠️ O recorte fino é do **alfa da
própria arte**, que já o faz de graça e ao sub-pixel; obrigar a grelha a seguir o contorno traria de
volta as células deformadas da borda, *que é exactamente o defeito que esta wave cura*.

#### As articulações são REAIS, não um palpite

[`skeleton_skin_image::joints_in_image`](../../shells/desktop/src/skeleton_skin_image.rs) leva cada
osso de mundo → local → **pixel da imagem** e entrega a lista ao leaf. ⛔ A `ph2d-poly2d` continua
sem saber o que é um osso — ela recebe pontos. Lista vazia ⇒ grelha **uniforme**, que é a leitura
certa de *«não há dobra nenhuma para adensar»*.

**Gates:** 5 novos (`the_mesh_has_a_middle_and_the_cells_are_square` · `a_joint_makes_the_grid_denser_around_it`
· `the_march_never_steps_over_a_joint` · `a_coarse_smaller_than_fine_is_coerced_not_obeyed` ·
`cells_without_paint_are_dropped`), **5 de 5 mortos por mutação**, com o controlo da árvore limpa.

⚠️⚠️ **E o primeiro deles reprovou sobre produto CORRECTO.** A 1.ª redacção de
`the_march_never_steps_over_a_joint` exigia um corte **em cima** da articulação; a lei real é *«a
menos de um quarto do passo fino»*, e a diferença não é folga — uma dobra a `1 px` de um corte que já
existe **está** naquele corte, e forçar um segundo ali produziria uma tira de `1 px`, isto é, a
célula de aspecto enorme que esta wave inteira existe para apagar. *Um gate que exige igualdade onde
a lei tem tolerância mede o defeito que a tolerância evita.*

⏳ **ABERTO:** os três números (`fine 10` · `coarse 26` · `radius 40` px) são de **PRODUTO, não
tectos de recurso** — o que está medido é a FORMA da resposta, e quem os julga é o smoke do dono.


### F6-c — ✅ **`SMOOTH`: A MALHA REFINA-SE NA HORA DE DESENHAR** (report do dono, 2026-09-10)

> *«malha bem desenhada. Contudo não é a solução perfeita em termos de deformação pois ao dobrar a
> articulação temos arestas retas na imagem. Estude um algoritmo com opção de um tipo de smooth na
> imagem e coloque como alternativa»* (foto com três setas)

#### ⭐⭐⭐ A lei que a wave achou

**A malha não é a deformação — ela é uma AMOSTRAGEM dela.** O campo `Φ(p) = Σ wᵢ(p)·Mᵢ·p` está
definido em **todo** ponto da imagem, porque os pesos são **derivados** e não guardados. A aresta
reta não vem do motor: vem do DESENHO, em que cada triângulo é pintado com **um afim**, que é a
aproximação de 1.ª ordem de um campo curvo.

⇒ o desvio da silhueta desenhada em relação ao campo verdadeiro é **`O(h²)`**, medido sobre a
cápsula do smoke com uma cadeia de 3 ossos e a silhueta **analítica** amostrada em 2000 pontos:

| | triângulos | desvio p99 a 60° | a 105° | a 150° |
|---|---:|---:|---:|---:|
| a grelha guardada | `216` | `3,85 px` | `6,85 px` | **`9,84 px`** |
| refinada `2×` | `768` | `1,09` | `1,98` | `2,79` |
| refinada `4×` | `3 086` | `0,29` | `0,54` | **`0,78`** |
| refinada `8×` | `11 472` | `0,07` | `0,13` | `0,19` |

**Com o `Smooth` ligado:** `9,84 px → 0,41 px` numa dobra de `150°`, com o `k` **derivado** da
deformação do próprio quadro.

#### ⛔⛔ E o erro NÃO mora nas articulações — a premissa da F6-b não vale para o DESENHO

Com as juntas em `x = 0 · 107 · 213 · 320`, o pior desvio cai em **`x = 119 · 190 · 266`**. Apertar
a banda fina à volta das juntas leva `3,99 px` a `3,39 px` e mais nada. ⇒ *a densidade que a DOBRA
precisa e a densidade que o DESENHO precisa não estão no mesmo sítio*: a primeira está na junta, a
segunda está onde os **pesos** variam depressa — e com `raio = comprimento do osso` isso é o membro
inteiro. **As duas leis são verdadeiras e nenhuma substitui a outra.**

#### ⚠️ Porque a DESENHAR e não a PRENDER

No instante do *bind* a pose é a de repouso: **não há dobra nenhuma**, logo não há erro para medir.
A densidade necessária é função da POSE, que só existe no quadro. *Uma malha escolhida no bind é
escolhida antes de a pergunta ser feita.* ⭐ E a malha guardada fica pequena — o save e o undo
continuam a fotografar a mesma coisa, e `PROJECT_SCHEMA` e `VEC_SCENE_SCHEMA` **não se mexem**.

#### ⚠️⚠️ A tolerância é em pixels de ECRÃ

Meia unidade local é meio pixel a zoom `1` e **quatro** a zoom `8`. *A suavidade que o olho vê é um
facto de espaço de ecrã* — por isso a tolerância passa pela escala da câmara antes de entrar no
leaf, e não é um número de unidades do documento.

#### ⭐⭐⭐ A conformidade é EXACTA, e não uma tolerância

O refinamento é **uniforme, com o mesmo `k` para toda a malha**, e cada ponto novo é nomeado pela
**aresta canónica** que o gera (o vértice de índice menor primeiro) ⇒ os dois triângulos que
partilham uma aresta calculam o ponto dela pela **mesma expressão, na mesma ordem**, e obtêm os
**mesmos bits**. ⛔ Um `k` por triângulo abriria nós pendurados — e aqui uma fenda é pior que numa
malha normal, porque cada peça é um **recorte independente** e a fenda vira um fio de fundo a
atravessar a arte.

#### ⛔⛔ O candidato CLÁSSICO foi construído, medido e REFUTADO

Misturar no **logaritmo do movimento rígido** (`se(2)` — o *dual quaternion skinning* do 2D, a
resposta de manual para o colapso do LBS) **PIORA**: área no pior ponto `−0,129 → −0,280`, e a
fracção da imagem **dobrada sobre si mesma** vai de `2,52 %` a `4,47 %` numa dobra de `150°`.
⇒ *o colapso não vem de a mistura das matrizes não ser uma rotação; vem do GRADIENTE DOS PESOS*, e
o `log` não toca nesse termo.

#### ⛔⛔ E a régua mentiu DUAS vezes antes de dizer a verdade

1. **O ângulo de quina sobre o contorno TRAÇADO** media o artefacto dela própria: o traçado de Moore
   devolve uma escada de pixels cujos degraus **já viram 90°** no repouso. O campo verdadeiro lia
   `86°`–`129°` de quina — tão «mau» como o produto.
2. **O ângulo de quina sobre a silhueta analítica** degenera: onde a silhueta raspa a esquina de um
   triângulo os dois segmentos ficam minúsculos e o ângulo entre eles é ruído.

⇒ a régua que ficou é o **desvio em PIXELS** entre a silhueta desenhada e a mesma silhueta levada
pelo campo verdadeiro. Ela é monótona, converge limpa (`O(h²)`) e **zero pontos caem fora da malha**,
que é o controlo dela.

#### ⚠️ Três gates reprovaram sobre produto CORRECTO, e cada um mudou o que a lei diz

| o gate dizia | o que a medição disse |
|---|---|
| *«o refinamento melhora ESTRITAMENTE a cada aperto»* | a correcção de um passo pode **passar** da tolerância pedida, e aí o degrau seguinte já está satisfeito. A lei é **não-crescente** |
| *«o campo é perguntado uma vez por VÉRTICE»* | a conferência anda pela malha **refinada** (`3` por triângulo) ⇒ o tecto do mecanismo é `~8 V`, não `4 V` |
| *«a escada de tolerâncias `4 / 2 / 1 / 0,5`»* | o desvio cru da fixtura era `3,82 px`, logo os dois primeiros degraus **não mordiam**. A escada passou a sair do **próprio desvio medido** |

#### ⭐⭐ E o estimador passou a CONFERIR o que entregou

Um gate vermelho exigiu-o: com a tolerância a pedir `0,954 px` o `k` de um só passo entregava
`0,980`. *Um número que se chama tolerância e não é honrado é um número que mente ao artista.* ⇒
uma correcção **única**, que parte da medição já feita **na malha refinada** (`k·√(d/tol)`) —
⛔ nunca um laço até convergir, que seria trabalho por quadro sem tecto.

#### O controlo

Fileira **Deform** no painel *Bones*, com dois segmentos — **Fast** (o de sempre, byte-idêntico) e
**Smooth**. ⚠️ Ela só é pintada quando há algo **preso**, e ⛔ **não** troca o modo da ferramenta: a
pergunta é de qualidade de desenho, não do que o arrasto faz.

⚠️ **O gate de costura apanhou-a PINTADA E NÃO REGISTADA** — morta sob o dedo, sem nada na tela que
o dissesse. É a terceira vez que esta linha paga a mesma lei: *um controlo nunca pintado e um morto
sob o dedo dão o MESMO report.*

#### ⛔⛔⛔ E a fileira ia nascer VIVA E INALCANÇÁVEL — achado ANTES do smoke

O portão óbvio era o `state::skinned()`, o mesmo dos botões *Expand* e *Release*. ⚠️ **Só que aquela
pergunta é *«a SELECÇÃO é uma forma presa?»*, e a shell responde-a varrendo `selected_paths()` —
CAMINHOS VECTORIAIS.** Uma imagem presa é uma **sprite**: ela nunca aparece naquela lista. A fileira
teria sido pintada em código e **nunca na tela**, com o gate de costura VERDE (ele arma o estado à
mão).

⇒ *VIVO e ALCANÇÁVEL são duas perguntas, e a segunda quase não tem instrumento neste repo.*

⚠️ E a pergunta certa é sobre a **CENA** (*«há alguma imagem presa?»*) e não sobre a selecção, porque
a escolha é **global**: ela vive na ferramenta e vale para toda imagem presa. Uma fileira que só
aparecesse com a imagem escolhida prometeria uma propriedade por-objecto que não existe.

O gate tem **três** metades e a do meio é a que mata: nada · uma **FORMA** presa (não basta) · uma
**IMAGEM** presa. Provado por mutação — repor o portão na pergunta da selecção dá RED com
*«uma FORMA presa acendeu a fileira do desenho da IMAGEM»*.

**Gates:** 7 no refinamento (`7 de 7` mortos por mutação, com o controlo da árvore limpa) + 3 na
costura do painel (o da alcançabilidade também morto por mutação).

⏳ **ABERTO, e nomeado:**
- **O custo de GPU não foi medido.** O que está medido é a CPU (encoding + deformação); cada
  triângulo é um `push_clip` do Vello, e o preço de `3 456` camadas por quadro contra `216` só o
  smoke o diz. É por isso que o `Smooth` **nasce desligado**.
- ⛔⛔ **O MAPA DOBRA SOBRE SI MESMO em dobras fortes, e isso NÃO é o que esta wave curou.** Medido:
  a `60°` por junta a área no pior ponto é **`−0,129`** (negativa ⇒ inversão) e `0,22 %` da imagem
  está dobrada; a `150°` são **`−1,017`** e `2,52 %`. O `Smooth` desenha o campo com fidelidade —
  **inclusive a dobra**. A causa é o gradiente dos pesos com `raio = comprimento do osso`, e a
  família de curas (pesos mais apertados · *centers of rotation* · o alcance por osso) não foi
  medida.
- O `max_split = 6` tem tecto de **CPU**; a tolerância de `0,5 px` é de PRODUTO, e quem a julga é o
  dono.

### F6-d — ⛔⛔⛔ *«Smooth bugado quebrando a forma»* — **o TECTO estava na grandeza errada** (report, 2026-09-10)

> Foto: o braço quase **recto**, a silhueta com **degraus** e a ponta direita comida.

#### A malha está PROVADAMENTE certa — e é isso que aponta o dedo ao consumidor

Medido no caminho exacto do produto (cápsula do smoke, cadeia de 3 ossos, `GridOptions` de omissão):

| | valor |
|---|---|
| área de repouso | `30 720,00` refinada contra `30 720,00` guardada — **ao cêntimo** |
| triângulos que o desenho SALTARIA (`from_triangle` a devolver `None`) | **`0`** |
| arestas com mais de dois donos | **`0`** |
| bordo | `288` = `48 × 6`, exactamente o que `k = 6` uniforme dá |

⇒ *sem peças perdidas, sem sobreposição, sem nó pendurado.* O que quebra está **a jusante**.

#### ⭐⭐⭐ A experiência que o report correu sem querer

Com o braço a **`2°`** de dobra o desvio já é **`0,499 px`** — a tolerância de omissão. Logo o `k`
saltava para o tecto e a malha ia a **`7 776` peças para desenhar a MESMA coisa que o `Fast`
desenha em `216`** (o desvio dele ali é `0,5 px`). *Mesma geometria, `36×` as peças, partida.*

⇒ **o recurso é a CAMADA DE RECORTE do renderer.** Cada triângulo é um `push_clip` do Vello, que
dimensiona os buffers dele por heurística e **degrada em SILÊNCIO** quando eles estouram: geometria
certa, imagem partida — que é exactamente o retrato da foto.

#### ⛔⛔ E o meu tecto era do `k`, que é a grandeza ERRADA

`max_split` limitava as **partes por aresta**; o renderer paga a **contagem**. As duas não são a
mesma coisa: o `k` é **quadrático** na contagem, e a malha de partida pode ter qualquer tamanho —
*o mesmo `k = 6` custa `7 776` peças numa malha de 216 e `36` numa de 1.* ⇒ §0.0: um tecto legítimo
diz de que recurso ele é, e o meu dizia de outro.

O tecto passa a ser **`max_pieces`**, e o `k` sai de uma **divisão** (`peças · k² ≤ orçamento`).

#### ⚠️ O número NÃO está medido, e isso está dito em voz alta

O limite é de **GPU** e não há aqui como o medir sem ecrã: o que se mediu foi a **malha**. O
intervalo conhecido vem do smoke do dono — **`216` desenha, `7 776` parte** — e o valor de omissão
(`1 024`) fica do lado seguro dele.

⭐⭐ **E o smoke deixa de PERGUNTAR e passa a MEDIR:** `PH2D_SKIN_PIECES=<n>` fecha o intervalo
**numa corrida só**, em vez de custar uma volta de report por tentativa; `PH2D_BONE_LOG=1` imprime
`peças (k=…, orçamento …)` para o report dele carregar o número.

#### ⚠️ Uma mutação SOBREVIVEU e obrigou a uma fixtura patológica

O tecto **dentro da correcção de um passo** não era alcançado por nenhuma fixtura: com um campo
suave a lei `O(h²)` acerta, a correcção pede `k + 1` e o `clamp` de cima nunca morde. A fixtura que
o alcança é uma onda cujo **período é da ordem da célula**: o estimador **aliasa** (os meios das
arestas caem perto dos zeros dela), lê `k = 5`, e a conferência — que já vê a onda — pede **`19`**.
*Uma guarda que só o caso patológico alcança precisa do caso patológico escrito.*

#### O preço da correcção, dito sem maquiagem

Com o orçamento de omissão a malha do smoke vai a `k = 2` (`864` peças) em vez de `k = 6`
(`7 776`). O desvio numa dobra de `150°` passa de `13,99 px` (`Fast`) para **`3,5 px`**, e não para
os `0,41 px` que a F6-c anunciava — *aquele número era real e foi medido sobre uma contagem de peças
que o renderer não desenha.* Subir o orçamento é do dono, e o botão existe para isso.

### F6-e — ✅ *«Smooth bugado quebrando a forma»* **era o ATLAS, não a camada de recorte** — e o `Fast` também partia (2026-09-13)

⛔⛔⛔ **A F6-d atribuiu o limite à camada de recorte do Vello e escreveu *«não há como o medir sem
ecrã»*. As duas afirmações estavam erradas.** A decisão que partia a imagem é da **CPU**, dentro
do `vello_encoding::Resolver`, e mede-se sem adaptador.

**O mecanismo:** cada triângulo desenhava a imagem pela porta CRUA
(`VectorScene::draw_image_rgba_transformed`), que cunha uma `Blob` — logo um **id do atlas** — **por
chamada**, mesmo com o `Arc` partilhado. O atlas da `vello` 0.10 é por id e pára em `8192²`: N peças
eram **N cópias inteiras** da imagem, e o que não cabe **não é desenhado, em silêncio**. Medido
(`ph2d-vector::atlas_probe_pieces_tests`, 4 quadros, o último lido):

| imagem | peças | porta crua: peças que NÃO aparecem | estável |
|---|---:|---:|---:|
| `320×96` (o smoke) | `216` | `0` — e `25,3 MB` reenviados por quadro | `0` |
| `320×96` | `2 048` | `0` | `0` |
| `320×96` | `7 776` (o report) | **`5 651`** | `0` |
| `1024×1024` | `216` (**o `Fast`**) | **`152`** | `0` |

⇒ *«216 desenha, 7 776 parte»* reproduz-se à letra, e a última linha diz que **o modo de omissão
partia com arte de tamanho comum** — defeito que ninguém tinha reportado porque o smoke usa uma
imagem pequena.

**A cura:** a `SkinImageCache` guarda um `StableImage` (clone = mesmo id) e cada peça desenha pela
porta nova `VectorScene::draw_stable_image_transformed` (aditiva; `draw_stable_image` delega nela).
**Gates:** `a_skinned_image_is_one_atlas_resident_however_many_pieces_and_frames` (o que a cena
EMITE, pelo `draw_skinned_images` real; visto RED com `8 224` ids para uma imagem) + o controlo da
porta crua e a lei com a corrida gémea como oráculo.

### F6-f — ✅ **Um quadro sem recurso tardio apagava o atlas do Vello** — achado pela cura da F6-e (2026-09-13)

⛔⛔⛔ **A cura da F6-e pôs a pele num caminho onde um defeito do `vello` 0.10 é alcançável.** Medido
na GPU, pelo `VelloPass` (alfa no centro):

| sequência | resultado |
|---|---|
| imagem → imagem | `255 → 255` |
| imagem → **quadro vazio** → imagem → imagem | `255 → 0 → 0 → 0` — **nunca mais volta** |
| imagem → imagem CRUA → imagem | `255 → 255 → 255` |

**O mecanismo:** sem patch nenhum (nenhuma imagem, gradiente ou texto) o `Resolver::resolve` sai por
`resolve_solid_paths_only` com um atlas de largura `0`; o `render.rs` troca a textura do atlas por
uma de `1×1` e, no quadro seguinte, por uma NOVA em branco — e o `ImageCache` da CPU continua a dar a
imagem estável por enviada. ⚠️ Pela porta crua nunca se via (reenvio por quadro). **Vale para todo
utilizador de `StableImage` da casa.**

**A cura, na porta única** ([`ph2d-render::vello_keepalive`](../../crates/ph2d-render/src/vello_keepalive.rs)):
uma cena sem recurso tardio é composta com uma imagem de `1×1` fora do alvo (id fixo, cobertura
zero); com recurso, passa **sem cópia**. **Gates:** o de GPU (visto RED) · a decisão sem GPU · o
**censo das duas entregas** ao Vello (`render_to_intermediate` e `render`), porque o gate de GPU só
passa pela primeira. ⚠️ No editor o risco era baixo (todo quadro tem texto); um runtime sem texto
cairia nele.

### F6-g — ⏳ **O que sobra do orçamento, MEDIDO: o tecto é um buffer do QUADRO, e as costuras existem em todo modo** (2026-09-13)

**1. O tecto duro.** O Vello guarda a informação de todo desenho num buffer FIXO
(`bin_data = 1 << 18`, *«hand picked»* no `vello_encoding::BufferSizes::new`), e `binning_size =
bin_data − layout.bin_data_start` dá a volta a um `u32` quando passa: **pânico em debug, quadro em
branco em release — painéis incluídos**. Uma peça custa **11 palavras**, linear (gate
`a_skin_piece_costs_eleven_vello_bin_info_words_and_the_cost_is_linear`; sonda do produto
`VectorScene::probe_bin_info_words`).

**2. A GPU**, sonda `ph2d-render::skin_pieces_gpu_cost` (arte opaca `320×96`, alvo limpo por
grelha; zoom 8, carga `2,2`→`9,5`, as últimas linhas de relógio valem pouco):

| peças | quadro | BURACOS | px de costura | alfa mín |
|---:|---:|---:|---:|---:|
| sem recorte | `1,25 ms` | – | – | – |
| `216` (`Fast`) | `1,58 ms` | `0` | `33 476` | `182` |
| `3 456` | `2,46 ms` | `0` | `132 164` | `182` |
| `7 776` | `5,02 ms` | `0` | `259 231` | `171` |
| `17 496` | `10,2 ms`* | `0` | `408 886` | `170` |
| `21 600` | — | **todo o miolo** | — | `0` |
| `≥ 31 104` | pânico no Vello (debug) | | | |

⇒ numa cena **só com a pele** o quadro fica em branco entre `17 496` e `21 600` peças (as 11 palavras
mais a distribuição por bins, que usa o resto do mesmo buffer).

**3. ⛔⛔ As COSTURAS existem em todo modo, o `Fast` incluído:** dois recortes vizinhos com AA
analítico compõem `1 − a·b` na aresta partilhada, e o fundo espreita até **~29 %** (alfa `182`) numa
linha por aresta — `16 580` px a zoom 4 com as `216` peças do smoke.

**4. ✅ A guarda por QUADRO** (o tecto de `1 024` por IMAGEM passava por cima da tolerância — a
`k = 2` o `Smooth` entregava `3,5 px` numa dobra forte contra `0,5 px` pedidos — e não protegia o
quadro: N imagens presas multiplicavam-no). Medido o outro consumidor do buffer, o chrome do editor
pintado sem ecrã pelo registo real (sonda `ph2d-editor-core::vello_bin_budget_of_an_editor_frame`,
texto incluído): **`109`** palavras com os painéis de omissão, **`~1 190`** com todos abertos. ⇒
`SKIN_FRAME_PIECES = (1 << 18) ÷ 2 ÷ (11 + 4) = 8 738`, repartido **proporcionalmente** pelas
imagens presas (o mesmo `k` para todas), e dentro dele **a tolerância decide**. ⚠️ A metade que
sobra é da arte do canvas, que **não foi medida**. Gate
`the_smooth_pieces_of_all_skinned_images_share_one_frame_budget` (visto RED com o tecto por imagem:
`18 t` contra `9 t`). Malhas `Fast` que sozinhas passam do orçamento não têm o que cortar: aviso
único no stderr.

⏳ **ABERTO, e nomeado:**
- ✅ **As costuras — CURADAS pela F6-i.** Sobrepor cada recorte `~0,5 px` foi medido e recusado
  (dobra a composição em arte translúcida ao longo da costura); ficou o **pipeline de triângulos
  texturados** que a F6 nomeou *«com razão medida»* — e ele não pediu pipeline nova: é a malha
  dentro do passe de sprites.
- ✅ **Medido por leitura, e é pior do que a pergunta:** ver a F6-h.

### F6-h — ⛔⛔⛔ **A IMAGEM PRESA NÃO É UMA SPRITE DO QUADRO: é uma camada por cima dele** — quatro defeitos, uma causa (2026-09-13)

Lido no código (com um mapa da composição do quadro a apontar os sítios):

1. **A ORDEM DE PROFUNDIDADE perde-se inteira.** Sprites e formas vectoriais do documento
   intercalam-se por UMA classificação partilhada (`sim_extract.rs` → `compute_sort_ranks_into` →
   `FrameOrder` → bandas). A sprite presa passa pela guarda `drawn && !skinned_image(…)` e **não
   emite** — e o `emit::sprite` é o único sítio onde uma sprite empurra o seu `SortInput`; a
   `vector_participant` devolve `None` para toda entidade com `Sprite`. ⇒ **sem rank**. O desenho vai
   para a cena do CHROME (`fase_vector_edit_overlay.rs`), que se compõe por cima de tudo: **acima de
   todas as sprites, de toda a arte do documento, e do vidro do prefab aberto**; entre várias imagens
   presas, a ordem é a de arquétipo (`iter_entities`).
   ⛔ **E o doc da `skinned_image` afirma o contrário** (*«ocupa o lugar dele na ordem mas não emite
   instância»*), com um gate que só compara posições de texto.
2. **A VISIBILIDADE não é perguntada.** A porta `off_canvas::draws_this_frame` (olho da Hierarquia,
   peça de receita fora do canvas, camadas) só corre na extracção; o `draw_skinned_images` não a
   chama ⇒ **uma imagem presa escondida continua a ser desenhada**.
3. **As PROPRIEDADES da sprite perdem-se:** o desenho da pele não lê tinta, opacidade, espelhamento,
   modo de mistura nem filtro de amostragem (zero ocorrências no `skin_image.rs`).
4. **As COSTURAS** (F6-g), e o tecto de peças do Vello.

⇒ **A cura de padrão-ouro é uma só: a sprite presa é desenhada como MALHA dentro do passe de
sprites, no lugar dela na ordem** — a extracção emite-a (com rank, com a porta de visibilidade, com
as propriedades), e o passe troca o quad pela malha posada. As costuras somem (triângulos sem AA nas
arestas internas) e o orçamento do Vello deixa de se aplicar. ✅ **Curada pela F6-i**
([plano 03](03_plano_a_pele_no_passe_de_sprites.md), W1+W2) — e o plano achou um quinto defeito: a
régua da imagem lia a âncora CRUA.

### F6-i — ✅ **A IMAGEM PRESA É UMA SPRITE DO QUADRO, desenhada como MALHA** (plano 03, W1+W2, 2026-09-13)

A cura da F6-h, em duas waves ([plano 03](03_plano_a_pele_no_passe_de_sprites.md) §5).

**W1 — o primitivo** (`ph2d-render`, aditivo, `335fe893a`). Um `SpriteMesh { local, uv, tris }` na
entidade de presente de uma instância faz o passe de sprites trocar o quad pela malha.
⭐ **Sem pipeline nova:** o `vs_main` já calcula tudo de `quad_pos`/`quad_uv` e da instância, logo
um vértice leva a UV de repouso e o `quad_pos` que devolve a posição posada, e a malha herda tinta,
opacidade, mistura, repetição, espelho e recorte. `N` triângulos entram como UMA tira com
degenerados (`5N − 2` vértices) nas 10 pipelines `TriangleStrip` de sempre; a marca da malha vive
nos bits `8..31` do `flip_uv` (só CPU), e os três passes (normal, recorte, máscara) desenham pela
mesma porta (`sprite_mesh::draw_run`). Gates de GPU: a malha em repouso É o quad (tinta, opacidade,
espelho, âncora deslocada) e ⭐ **uma imagem TRANSLÚCIDA numa malha `8×8` não tem costura** — a regra
de canto dá cada centro de pixel a UM triângulo.

**W2 — a extracção** (shell + `ph2d-skeleton-live`):

- A guarda `!skinned_image` saiu: a sprite presa passa pelo `emit::sprite` (rank, porta de
  visibilidade, propriedades), e **depois** do extract o `attach_skin_meshes` põe o `SpriteMesh`
  posado na instância BASE (`Without<SlicePatchMirror>`) — só se ela for o quad da própria sprite.
  Uma sprite escondida não tem instância ⇒ não recebe malha: *a visibilidade não é perguntada duas
  vezes*.
- ⛔⛔ **O quinto defeito:** o `pixel_to_local` lia a âncora CRUA. Hoje lê a `resolve_anchor(ppm)` e
  o ESPELHO entra na POSIÇÃO; a UV é a do quad no ponto de repouso (`SpriteMesh::uv_at`, na crate do
  shader), e o shader espelha-a. O `ppm` do projecto entra no `bind_image`, `joints_in_image` e
  `deform_field`.
- O `Smooth` mede a tolerância com a base que a instância leva à GPU × a escala da câmera da CENA
  (`scene_camera_window`).
- Saíram: `draw_skinned_images`, `SkinImageCache`, `stable_image`, `triangle_xform`, o campo
  `skin_image_cache` do `SkeletonState`, a chamada no overlay do Vello e a `skinned_image` do
  extract — que virou `skin_image::is_skinned_image` (o painel do esqueleto pergunta-a, e o
  `attach_skin_meshes` também).
- Gates: `at_rest_each_pixel_of_a_bound_image_is_read_where_the_quad_reads_it` (controlo · não
  centrada + offset · espelho X + offset · não centrada + espelho Y) ·
  `only_the_base_instance_of_the_sprites_own_quad_gets_the_mesh` · o orçamento por quadro reescrito
  sobre a malha · `uv_at_gives_every_quad_corner_the_uv_the_quad_gives_it` · os três de texto da
  shell reescritos contra a lei nova (o braço que emite não pergunta pela pele · a malha é posta
  depois do extract e a camada do Vello não voltou · o *Bind* alcança as duas mídias).
- **Oito mutações, oito RED na asserção certa** (cada controlo com `1 failed`): âncora crua (o pixel
  cai `21 px` ao lado) · sem espelho (o pixel `0` lê o `40`) · sem `Without<SlicePatchMirror>` (a
  fantasma rouba a malha à base) · sem a comparação do quad (`2` malhas) · `v` do `uv_at` invertido ·
  tecto por imagem (`288` peças contra `144`) · a guarda de volta ao extract · a chamada apagada.

**W3 — os outros consumidores** (`ph2d-render` + shell). Uma malha vive num componente AO LADO da
instância, então quem COPIAVA a instância ou lia o QUAD de repouso desenhava ou apontava o sítio
errado:

- **Quem copia leva a malha.** O vidro do prefab (`present_frost::lift`) e o emissivo
  (`sprite_emissive::collect`) passam por `LiftedInstances::collect_from` — a instância COM a
  `SpriteMesh` — e desenham por `render_lifted_instances`, que marca cada cópia com a malha dela
  antes da ordenação. ⚠️ As malhas copiam-se por `clone_from` para buffers reusados, e o `Clone` da
  `SpriteMesh` passou a ser à mão por isso. O glow do Motion continua pelo `render_instances_only`.
- **Quem aponta lê o que é desenhado.** Os pickings, a caixa do gizmo, o laço e a UV do pintor
  perguntam a MESMA regra do desenho (`sprite_mesh::drawn_mesh`) e testam os triângulos POSADOS;
  numa dobra que sobrepõe a malha ganha o triângulo desenhado por último, e fora da malha o
  `sprite_world_to_uv` devolve `None`.
- ⭐ **O *View All* já estava errado antes da malha:** a shell reconstruía à mão um quad centrado no
  PIVÔ, sem âncora nem base. Hoje lê `scene_sprites_bbox_world`, a caixa do que é desenhado.
- Os testes do picking saíram para `picking_tests.rs` (corte mecânico, `13` testes contados antes e
  depois) para o ficheiro caber no tecto de `700` com a malha.
- **Oito mutações, oito RED:** o `push` a ignorar a malha · o `clear` sem repor as malhas (a
  instância sem malha herdava a do quadro anterior) · o `tag_lifted` sem o laço · a UV do 1.º
  vértice em vez da interpolada · o `drawn_mesh` sem comparar comprimentos (apanhado pelo gate da
  W1) · e o picking, a caixa e a UV a ignorarem a malha.

⏳ **ABERTO, e nomeado:**
- ✅ **Os fantasmas do onion desenhavam o quad de repouso — CURADO pela W7** (abaixo). A redacção
  deste item previa *três peças* e o preço estava certo; o que ela não previa era que, num rig
  normal, o onion **não produzia fantasma nenhum**.
- O `sprite_world_to_uv_unclamped` fora da malha responde pela lei do quad de repouso: um traço de
  pincel que sai da silhueta posada é mapeado como se a imagem repousasse.
- **9-slice e folha desdobrada:** a malha só conhece o quad da sprite; essas desenham-se sem
  deformar, com aviso único no stderr.

**W4 — o orçamento, RE-MEDIDO.** O `SKIN_FRAME_PIECES` era `8 738`, derivado do buffer fixo do Vello
— um recurso que este caminho já não gasta. Medido o que sobra (sondas
`measure_the_cpu_cost_of_a_skinned_frame` e `measure_the_frame_cost_of_a_mesh_sprite`, `load 3,7`–`3,9`,
o **mínimo** de 40/60 corridas — a carga de FUNDO desta workstation é `~7` sem ninguém compilar, e
uma média sob contenção mede o vizinho):

| o que o quadro faz por peça | µs |
|---|---:|
| descodificar a malha guardada (postcard, **por quadro**) | `0,134` |
| `Fast`: descodificar + deformar + montar o `SpriteMesh` | `0,200` |
| recolher + costurar a tira + enviar + DESENHAR (GPU esperada) | `0,039` |
| **`Smooth`: o quadro inteiro, por peça ENTREGUE** | **`1,08`** |

⇒ as `8 738` peças do número velho custariam **`9,4 ms`** — mais de metade de um quadro de 60 fps. O
tecto novo é **`1 543`**, derivado de `16,667 ms ÷ 10 ÷ 1,08 µs`; ⚠️ **a FATIA (`1/10`) é a única
escolha** e está nomeada, o resto é medição. Gate nas duas pontas (cabe na fatia · ainda refina um
`k = 2`), e a `ph2d-vector` saiu das dependências da `ph2d-skeleton-live` com o número que ela
sustentava.

**W5 — a cena que ENSINA.** A cena do osso (`PH2D_VEC_BONE_SMOKE=1`) ganha uma **barra azul** que
atravessa o braço pintado e nasce DEPOIS dele ⇒ desenha-se à frente: é a prova, à vista, de que a
imagem presa está na ORDEM do quadro. ⚠️ **A barra é DERIVADA do rectângulo da imagem** (uma porta,
`overlap_bar`), com gate — escrita como dois literais, mover a imagem deixaria a barra ao lado e a
cena passaria a ensinar nada, em silêncio. E a linha *«Painted arm»* na Hierarquia tem o **olho**:
fechá-lo esconde a imagem presa, que a camada do Vello continuava a desenhar.

**W6 — a CAIXA DO GIZMO ENGOLIA O RIG** (report do dono no smoke da W5: *«selecionar o osso não é
mais possível»*). ⭐⭐⭐ **O sexto consumidor da W3 não copiava a instância nem lia o quad: ele passou
a EXISTIR.** A `snapshots::gizmo::sprite_view` pede um espelho no presente
(`query::<(&SimRef, &GlobalTransform)>`), então enquanto a imagem presa não emitia instância ela
**não tinha caixa de gizmo** — e ninguém o notou, porque a ausência era a do objecto inteiro. Com a
W2 ela passou a emitir: seleccionada, o `paint_sprite_gizmo` regista `ids::GIZMO_BBOX_INTERIOR`
sobre a arte inteira, o `on_canvas` do despacho pede o `hit_index` **VAZIO**, fica falso em cima
dela, o `ramo_ferramenta_vetorial` **nem corre** — e o ramo do modo Osso, que é onde
`bone_gesture::press` vive, nunca chega a ser perguntado. *Nenhum osso por cima da arte que ele
deforma podia ser apontado nem posado.*

⚠️⚠️ **A lei que cura já estava escrita, e a SPRITE ficava de fora dela.** O ADR-0112 diz *«o gizmo
de objecto só existe fora da ferramenta vectorial, ou no modo Select dela»*, com a razão ao lado —
*as alças registam hit-rects, e uma caixa sobre o canvas de um modo de autoria é um ladrão de
cliques*. Ela estava escrita **por família**, num `if` dentro de cada ramo: a forma vectorial tinha,
o envelope tinha, o Flip tinha o gémeo dele — **a sprite e o grupo não tinham**. ⇒ hoje é **UMA
porta** no topo do `build_view` (`object_gizmo_on`, renomeado de `vec_gizmo_on`, que era um nome a
mentir), e os dois `if` por família saíram. *Uma lei escrita em dois sítios ainda não é uma lei.*

⚠️ **A selecção fica ARMADA** — só a caixa e as alças somem —, e é isso que mantém o *Bind to
Skeleton* (que age sobre a selecção de formas) com sujeito dentro do modo Osso. ⛔ E o gizmo não
perde gesto nenhum: dentro daqueles modos o `ramo_ferramenta_vetorial` consome **todo** press de
canvas, logo a caixa só era alcançável exactamente onde ela bloqueava.

Gate `snapshots_object_gizmo_tests` (duas metades, cada uma com o CONTROLO `object_gizmo_on = true`
ao lado — sem ele o gate ficava verde sobre uma view que nunca nasce): o primário e as **extras** de
uma multi-selecção. Mutação (`if false`): **2 de 2 RED**, na asserção certa.

⏳ **NOMEADO e não curado:** o gémeo do Flip — um objecto de OUTRA família continua a publicar caixa
enquanto a ferramenta Flip desenha (o `flip_gizmo_on` gateia só a arte do Flip). Mesmo mecanismo,
outra ferramenta, e sem report.

⚠️ **E há DUAS caixas de sprite neste repo, que esta wave não unificou:** a do gizmo sai do
`sheet_grid_overlay::gizmo_box(sprite, …)` (o quad da sprite) e a do `ph2d_editor_core::gizmo` sai do
`ph2d_render::selection_bbox_world` (que a W3 tornou ciente da malha). Numa imagem presa e dobrada
elas **discordam**, e hoje só a segunda é lida pelo *View All* e pelo contorno do realce.

**W7 — O ONION VÊ A PELE** (2026-09-13). ⛔⛔ **Medido primeiro, e a medição mudou a forma do
trabalho:** a redacção anterior deste item dizia *«os fantasmas desenham o quad de repouso»*, o que é
verdade **se houver fantasma**. Num rig normal não há — e por duas guardas que se excluem uma à
outra: o `collect_onion_ghosts` exige que o seleccionado esteja em `animated_entities` **e** tenha
`RenderInstance`, e numa personagem riggada quem leva keys são os **ossos** (que não desenham) e quem
desenha é a **imagem** (que não leva keys). *Um recurso cujas duas guardas se excluem está
desligado, não configurado.*

As quatro peças:

1. **A pose de MUNDO num instante** — [`ph2d_ecs::world_transform_with`] / `parent_world_transform_with`
   (a travessia de sempre com a **fonte da pose local injectada**) e
   [`ph2d_timeline::world_pose_at`], que a alimenta com o `pose_at` de **cada elo**. ⛔ Escrever o
   laço noutra crate seria a segunda resposta a *«onde está esta entidade?»*, que é o defeito de que
   o cabeçalho do `transform_inverse` avisa. Gate: a fonte VIVA devolve a travessia de sempre **ao
   bit** (com o controlo de uma fonte que mente, nas DUAS pontas — ⚠️ a 1.ª redacção só tinha a do
   PAI e uma mutação na folha **sobreviveu**).
2. **A pele resolvida NOUTRO instante** — `skin_live::skin_of_in` / `skin_image::deform_field_with`
   com a fonte de poses injectada; a crate continua sem conhecer a timeline (*aqui só existe «onde
   está esta entidade»*). O `bone_index` passou a ser **público e emprestado**, porque o consumidor é
   um laço (`N` artes × `M` instantes) — a lei que o doc do `skin_of` já escrevia.
3. **A malha posada numa PORTA** (`skin_image::posed_sprite_mesh`), com dois consumidores: o quadro
   vivo e o fantasma. ⛔ Copiada, as duas divergiriam no primeiro ajuste da UV.
4. **A fatia `extra` do passe leva MALHAS** — ela virou uma `LiftedInstances` (o par instância+malha
   que a W3 criou). ⛔ Um vector paralelo de malhas ao lado de uma fatia crua é o padrão que o
   `corner_radius` proíbe por escrito. O stream do Motion entra sem malha e desenha byte a byte.

⛔⛔ **E a mesma pergunta tinha de ser feita DUAS vezes, uma em cada eixo — achado ANTES do smoke:**
os *instantes* do modo `Keys` (o de **OMISSÃO**) saíam das keyframes do alvo **DESENHADO**, e a
imagem de um rig não tem nenhuma. Com o escopo curado e os instantes não, o recurso continuava mudo
**na configuração de fábrica**. ⇒ um alvo do onion passou a ser `{ o que se DESENHA, quem tem as
KEYS }` (`GhostTarget::relogios`), e para um rig os relógios são os ossos **animados** do esqueleto —
a UNIÃO deles, porque a pose do braço muda quando QUALQUER osso tem uma key. *Nas duas metades a
pergunta certa é a mesma: «quem MOVE isto?».*

E o **ESCOPO**: `ghost_targets` responde *«o que o seleccionado faz mover?»* — a arte animada (o
escopo do ADR-0142) **ou**, com um OSSO na mão, as imagens presas ao esqueleto dele
(`skin_live::skinned_images_of_skeleton`), se alguma coisa naquele esqueleto estiver animada.
⚠️ **A condição é do ESQUELETO, não do osso na mão:** o animador escolhe o osso que vai posar, que
pode ainda não ter key nenhuma. ⛔ **Só IMAGENS** — uma forma vectorial presa é desenhada pelo Vello
e não tem instância, logo o passe que desenha fantasmas não a alcança (limite NOMEADO).

⭐ **E uma nota FECHOU de graça:** o `ghost_instance` lia o `pose_at` LOCAL, com *«para um objeto RAIZ
o Transform É o GlobalTransform; rigs parenteados são wave futura»* escrito ao lado — hoje lê o
`world_pose_at`, e para uma raiz as duas respostas são as mesmas (os nove gates do onion passam sem
uma linha mudada).

⏱️ **MEDIDO** (`min` de 40 corridas, `load 9,6`–`19,0`; a estabilidade entre as duas cargas é a
assinatura da instrumentação da W4): `4` fantasmas × `528` peças custam **`0,333 ms`** — **`2,0 %`**
de um quadro —, logo uma peça de fantasma vale `0,158 µs`. ⚠️ **No extremo dos dois sliders**
(`MAX_GHOSTS = 8` de cada lado) sobre uma pele no tecto dela (`SKIN_FRAME_PIECES = 1 543`) isso é
**`~3,9 ms`, `23 %` de um quadro**. ⛔ **Não se corta nada:** o artista pediu `n` fantasmas, e deitar
fora os mais distantes é decisão de PRODUTO — o que fica é o NÚMERO, no log da família
(`PH2D_BONE_LOG=1`) quando as peças de fantasma passam o orçamento que a pele viva declara para si.

⛔ **O fantasma usa SEMPRE a malha guardada, nunca o `Smooth`:** uma silhueta chapada não tem detalhe
que um quarto de pixel de tolerância salve, e o orçamento de refinamento foi derivado para a arte
VIVA. *O fantasma é uma leitura, não a obra.*

**Oito mutações, oito RED** — o ramo do osso no escopo · o fantasma levar a malha · a pele resolvida
em `t` · a fatia de fora levar malhas · a fonte injectada na ENTIDADE e nos ANCESTRAIS · a pele ler a
fonte · os relógios serem os ossos e não a arte. ⚠️ **A da ENTIDADE sobreviveu à primeira**, e nomeou o buraco: numa FOLHA a cadeia de
ancestrais já basta para a resposta mudar, então o controlo tem de ser numa **raiz**.

⚠️ **E a fixtura mordeu antes do produto** (2×): a pose injectada nasceu `Transform::IDENTITY` e o
gate leu `2e0` de desvio — exactamente a translação da raiz do osso. *Uma fixtura que perde a pose de
base mede outro esqueleto.*

**A cena ENSINA:** o osso da ponta do braço pintado nasce com animação **no clip ABERTO** (⚠️ ao
contrário da acção do osso inteligente, que tem de estar fechada — o onion lê o clip **activo**), e a
cena diz o NOME da linha na Hierarquia.

**W8 — OS DOIS RELATOS DO SMOKE DA W7** (2026-09-14). O dono correu a cena e devolveu duas frases:
*«só aparece a silhueta do futuro»* e *«com a timeline aberta não é possível transformar os ossos e
criar key frames com AutoKey»*. São **dois defeitos de FIO**, nenhum de motor — e cada um tem a mesma
forma: *o quadro já respondia àquela pergunta noutro sítio, e este sítio respondia sozinho.*

**(a) O RELÓGIO.** Um fantasma é a pose de [`ph2d_timeline::pose_at`], que fala o tempo do **clip
activo** — o espelho exacto do `apply_active_clip`. Mas o relógio que a vista dirige não é sempre o
mesmo objecto: a aba **Keys** (a de OMISSÃO) move o `clip_playhead`, um contêiner aberto move o
`container_playhead`, e só o Arrange move o `playhead` da cena. O quadro escolhe entre os três em
**dois** sítios (o dreno da timeline e o passe de AutoKey), e a chamada do onion passava um **quarto**
palpite escrito à mão, `self.playhead.time()`. ⇒ na configuração de fábrica arrastar o cursor **não
movia** esse número: ele ficava em `0`, não havia key ANTES dele, e **só o futuro tinha vizinhos**.
Cura: ler o que a vista **já publica** uma vez por quadro, `TimelineViewSnapshot::clip_time` — que é
`None` quando o clip não toca ali (ou toca duas vezes numa pilha), e aí a resposta honesta é
**nenhum fantasma**. ⚠️ **A decisão não é alcançável de um teste de unidade** (vive numa fase do
`render_frame`), então a lei é um **arch-gate** que varre o `src/` da shell — o irmão do
`the_motion_path_is_offered_only_on_the_keys_tab`, e pela mesma razão. ⚠️ **Ele reprovou primeiro
sobre o COMENTÁRIO que explica a cura**, que nomeia o relógio errado para dizer que ele saiu: *um
censo textual que não separa prosa de código mente nos DOIS sentidos.*

**(b) A MÃO.** O apply da timeline escreve a pose de toda entidade keyada, **menos** a que a mão
segura — e a única lista que ele conhecia era a do **gizmo de sprite** (`hero.gizmo.drag`). Posar um
osso é um gesto PRÓPRIO (o gizmo não serve: a caixa de um osso é `0×0`, e o `bone_pose::pose` já o
escrevia), e ele não publicava nada. Medido com sonda: a mão punha `0,77` e o apply devolvia `0,45`
**no quadro seguinte** — o osso voltava debaixo do dedo, e o AutoKey, que corre DEPOIS do apply, lia
`mundo == curva` e não tinha o que cunhar. *Duas metades do mesmo defeito, um relato só.*
⇒ `timeline_bridge::maos_do_quadro` (o gizmo ∪ o esqueleto que a ferramenta Bone pousa), e o
`live_entity: Option<u64>` do bridge vira `maos: &[u64]`.
⚠️ **De um osso vai o ESQUELETO INTEIRO**, não o agarrado: puxar a PONTA faz cinemática inversa e
dobra a corrente toda, logo uma mão de um elemento deixaria metade dela a brigar com o dedo — e qual
metade depende da alça. Congelá-lo durante o arrasto não perde animação (posar é um gesto de pausa: o
AutoKey é inerte a tocar). ⛔ E o **guarda** do sujeito que não é osso é load-bearing: sem ele o
`skeleton_of` devolve TODOS os ossos da cena, que é a leitura certa dele para *«ninguém apontou»* e a
errada aqui.
⚠️ **E a cura criava um defeito novo sem a terceira metade:** o `drag_now` do AutoKey também só
conhecia o gizmo, então cada quadro do arrasto seria uma «edição discreta» com passo de undo próprio
— quarenta passos para dobrar um braço. Hoje ele soma a mão no osso, e o gesto é **UM** passo.
⚠️ **As duas portas recebem os ESTADOS, não `Option`s já resolvidos** — é isso que fecha o fio: a
chamada vive numa fase que nenhum teste alcança, e resolver a mão fora delas repetiria o defeito.

**Cinco mutações, cinco RED:** o relógio da cena de volta · a porta a esquecer o OSSO · a porta a
esquecer o GIZMO (⚠️ *a mão antiga não se perde ao ganhar a nova* — é a forma de defeito que este
repo já pagou) · o AutoKey a esquecer a mão no osso · o onion a aceitar um instante ausente.

⏳ **ABERTO e nomeado:** no Arrange **com pilha** o `clip_time` responde `None` quando o clip toca
zero ou duas vezes — nenhum fantasma, que é honesto e **não** foi smokado · e o AutoKey keya só o
osso **seleccionado**, logo quem a IK moveu na corrente não recebe chave (é o modelo do Blender, e
não foi medido contra ele).

**W9 — O GÉMEO DO FLIP DA W6, fechado SEM report** (2026-09-14). A W6 deixou-o nomeado: *«um
objecto de outra família continua a publicar caixa enquanto a ferramenta Flip desenha»*. Medido
antes de tocar em código, e as três metades da medição:

1. **A condição.** `object_gizmo_on` era *«não estou na ferramenta vectorial, ou estou no Select
   dela»* — com a ferramenta **Flip** na mão ela é **verdadeira**, logo uma sprite, um grupo ou uma
   forma seleccionados publicam caixa.
2. **O consumidor.** O `ramo_flip_premidos` exige o **MESMO** `on_canvas` que o ramo vectorial
   (= *nenhum painel e nenhum widget sob o cursor*), em **9** sítios. ⇒ a caixa mata o traço.
3. **O alcance.** O `flip_wants_canvas()` não olha a selecção (ele lê `flip_state.active` e o modo),
   e o objecto Flip activo é **o 1.º do documento**, não o escolhido ⇒ o artista pode ter qualquer
   coisa seleccionada enquanto desenha. *O caso normal é uma sprite de REFERÊNCIA.*

⭐⭐ **E a medição achou a resposta já escrita uma família ao lado, com a forma OPOSTA:** o Painter
não usa o `on_canvas` — ele tem porta própria (`chrome_hit::pointer_over_chrome`) que **isenta** os
ids do gizmo, e por isso pinta por cima da caixa. ⛔ **Essa saída está RECUSADA aqui**: ali a caixa
continua PINTADA e deixa de pegar, que é a alça morta que o `CLAUDE.md` §5.0 nomeia. O que tem de
não existir é a CAIXA — que é o que o ADR-0112 já dizia.

⇒ a condição da porta única passou a ser *«nenhuma ferramenta AUTORA no canvas»* (a vectorial fora
do Select dela **e** a do Flip fora do Select dela), e o **terceiro** `if` por família
(`if !flip_gizmo_on`) morreu com ela — a W6 tinha matado os dois primeiros.
⚠️ **Para o objecto FLIP nada muda, e isso é álgebra, não promessa:** antes a caixa dele era
`object_gizmo_on_antigo ∧ flip_ok`, hoje é `object_gizmo_on_novo` = `vec_ok ∧ flip_ok ∧ ¬preview` —
a mesma expressão. O que muda é a caixa das **outras** famílias.

**Dois gates novos, e os dois são ARCH** (a condição vive numa fase do `render_frame` que nenhum
teste de unidade alcança — a porta que a consome já tem o gate da W6): a condição **nomeia as duas
ferramentas**, e o **censo dos ramos** que recebem o `on_canvas` — um QUARTO acorda a lei, porque é
uma ferramenta nova a partilhar o canvas. **Duas mutações, duas RED.**
⚠️ **E o `cargo check -p` do laço interno voltou a ler VERDE com um gate partido** (o 3.º hoje): ele
não compila os testes, e a chamada da W6 ao `publish_gizmo` ficou com um argumento a mais.

⛔⛔ **E a W9 EXPIROU um gate que estava certo, sobre um quadro certo:** o
`the_gizmo_is_not_published_while_the_preview_runs` citava a condição **INTEIRA** achatada — as duas
cláusulas da ferramenta vectorial *seguidas* do termo da preview — e a cláusula nova do Flip entrou
**no meio delas**. *Uma agulha ancorada na ADJACÊNCIA é um proxy que expira na primeira linha que
alguém acrescenta.* Hoje ela é o **termo** (`&&!self.ui_preview.is_on(),`, com a vírgula a provar que
ele fecha o argumento) e exige **unicidade** — mais forte do que era, e imune à cláusula seguinte.
Mutação (apagar o termo): RED.

⏳ **ABERTO:** a condição continua a viver no FIO (uma expressão numa fase), e o gate que a protege é
textual — ele apanha a regressão (uma cláusula apagada) e o crescimento da população, **não** a
ferramenta nova que ninguém ligou. A cura de fundo seria a ferramenta DECLARAR se autora no canvas,
e isso é o `Tool`, que é contrato **congelado** (§6).

**W10 — O PINCEL SEGUE A ARTE DOBRADA** (2026-09-14). O item da fila dizia *«a UV do pintor fora da
malha responde pela lei do quad de repouso»*. **Re-medido, o endereço estava errado e o defeito é
maior:**

- as duas portas que conhecem a malha desde a W3 — `sprite_world_to_uv` e o `_unclamped` — têm
  **ZERO chamadores de produto** (só testes e o `pub use`);
- o Painter mapeia o ponteiro por **outra** porta, o afim do **quad de repouso**
  (`ph2d_sprite_screen::sprite_image_to_screen_affine`), que não sabe o que é uma malha;
- ⇒ numa arte presa e **dobrada** a pincelada cai deslocada **exactamente pela deformação, em TODA a
  arte** — não só fora dela, que era o que a fila dizia.

*Duas portas com a lei certa, e o consumidor a usar uma terceira.*

⇒ [`ph2d_render::mesh_uv`] passa a ser **a porta de canvas**, com três estados porque o chamador tem
três coisas a fazer: **`Quad`** (não é malha ⇒ a lei do chamador fica **intocada** — e no Painter ela
carrega a grelha da folha, o *Repeat Image* e a margem do gizmo de deformação, que esta porta não
conhece), **`Use`** (a UV de repouso do texel sob o ponto) e **`Refuse`** (fora da arte, num gesto que
COMEÇA — um traço não nasce sobre um quad que não se desenha; com o traço já aberto ela devolve
`Use` com a UV do quad, que é a lei que o `_unclamped` já escrevia e é o que deixa a pincelada sair
da silhueta sem se partir).

**Gates:** os três estados + o **controlo da sprite SEM malha** (`ph2d-render`, atrás de uma fixtura
que já existia) · e um **arch-gate** para o fio (a porta de canvas exige janela e GPU: nenhum teste a
alcança). **Quatro mutações, quatro RED** — ⚠️ **uma sobreviveu à primeira e nomeou a metade que
faltava:** um `let malha = MeshUv::Quad;` ao lado de um `let _ = mesh_uv(..)` deixava o gate verde
sobre o defeito inteiro. *Citar uma porta não é consultá-la* ⇒ ele passou a exigir a **ligação**
(`let malha = ph2d_render::mesh_uv(`).

⏳ **ABERTO e NOMEADO:** o **chrome** do Painter continua no quad de repouso (o anel do pincel segue o
ponteiro e só o TAMANHO dele sai do afim; a curva, a linha, o gizmo de deformação, os gizmos de
selecção, os crachás e a humidade desenham-se em posições de IMAGEM) — numa arte dobrada eles ficam
no sítio de repouso. Não piorou com esta wave: antes a tinta estava errada **com** eles. E o
conta-gotas do *BgRemoval* usa uma caixa alinhada aos eixos que ignora rotação **e** malha.

⛔⛔ **E o TECTO DA SHELL ficou em `16` linhas de folga** (`196 974` de `196 990`): as três waves de
hoje são quase todas GATES, e o tecto é a grandeza que soma entre linhas sem ninguém a contar
(`CLAUDE.md` §5.0). ⭐ **A cura medida está identificada e não foi feita:** o `timeline_onion.rs` e os
dois ficheiros de teste dele são **~800 linhas** que não são composição — o motor do onion só depende
de crates (`ph2d-ecs`, `-render`, `-timeline`, `-skeleton-live`, `-poly2d`, `-vec-entities`) e só a
CHAMADA é da shell. Tirá-lo daria ao integrador ~800 linhas de folga e é o molde do HOWTO.

**W11 — O PINCEL PAGA A DEFORMAÇÃO** (report do dono com foto, 2026-09-14: *«o local é correto mas o
pincel não considera o resultante das deformações do mesh … o pincel é redondo mas pinta como se os
polígonos não estivessem deformados»*). A W10 pôs a tinta no **texel certo**; a **forma** continuava
a ser a da textura. Onde o leque comprime a arte, um disco de textura chega ao ecrã como uma
**lasca** — é o que a foto mostra dentro do anel redondo do cursor.

**A lei:** o artista pede um disco de raio `R` **no ecrã**. Com `W` a deformação local (ecrã por
textura, adimensional, **identidade em repouso**), o que tem de ser pintado na textura é `W⁻¹`
aplicado a esse disco — uma **elipse**. ⭐⭐ E uma elipse é exactamente o que o dab já sabe ser: o par
*Flatten + Angle* do gizmo de Shape mais o raio. ⇒ **nenhum tipo novo chega ao kernel**.

As três peças:

1. **A deformação sai do TRIÂNGULO** (`ph2d_render::sprite_mesh::warp_under`): `d(local)/d(uv)` do
   triângulo, **dividido pelo do quad de repouso** — é a divisão que a torna adimensional, e sem ela
   o pincel mudaria de forma ao redimensionar a sprite. Ela viaja no `MeshUv::Use`.
2. **A decomposição** (`ph2d_painter_brush::canvas_warp::warped_dab`): `W⁻¹ · E` (com `E` = a elipse
   que o artista autorou) nos três números que o motor consome. ⚠️ **Sem transcendentais (HR-5):** os
   semi-eixos saem dos autovalores de `A·Aᵀ` (só `sqrt`) e o ângulo por **procura na tabela cozida**
   de graus — um `atan2` podia arredondar para graus diferentes noutra plataforma, e o
   `dab_angle_deg` é um **inteiro** que escolhe tinta.
3. **A composição entra no `stroke_spec`**, que é a porta que o *Grid Stamp* já usava pela mesma
   razão — *os DOIS leitores têm de concordar*: o motor emite cada dab com aquele raio e o carimbo
   estica a silhueta com aquele achatamento.

⛔⛔ **E o gate apanhou a minha promessa a ser falsa:** sem um atalho explícito para a identidade, a
decomposição devolvia `radius_scale = 1,0000006` e `flatten = 0,39999998` sobre uma arte em
**repouso** — números plausíveis e **outra tinta**, em toda pincelada do app. *«Byte a byte» não é
uma promessa que uma raiz quadrada cumpra: é um `if`.*

**Gates:** o no-op ao bit (com o controlo de uma deformação real) · o eixo comprimido a pedir o dobro
do raio · **a elipse pintada a voltar REDONDA ao ecrã** (a régua é o produto, não os três números) ·
o triângulo colapsado recusado · a deformação adimensional (com o controlo do `posed_arm`, que só
translada) · o `stroke_spec` a pagá-la (com o controlo da identidade) · e o fio.
**Cinco mutações, cinco RED** — ⚠️ **uma sobreviveu à primeira, pela segunda vez no mesmo dia:** o
arch-gate exigia a MENÇÃO da porta e não a LIGAÇÃO à resposta dela.

⏳ **ABERTO e NOMEADO:** a deformação é a do **pen-down**, para o traço inteiro (é o `stroke_spec`
que o motor captura ao abrir, e é a mesma fotografia que o pincel de tecido tira da lista de
obstáculos). Um traço LONGO que atravesse regiões de compressão diferentes usa a do princípio. Para
seguir por dab, o caminho está medido: a deformação tem de viajar no `StrokePoint`, como a pressão.

**W11b — «SEM MELHORIAS»: a matriz nascia numa BASE MISTA, e as duas metades passavam** (2.º report
com foto, 2026-09-14). A W11 tinha as duas leis certas e o **join** errado.

⛔⛔ **O defeito:** o `warp_under` devolvia `j · diag(1/sw, −1/sh)` — as **linhas** em coordenadas
LOCAIS (`y` para CIMA) e as **colunas** em coordenadas de imagem (`v` para BAIXO). Numa base mista a
conjugação pelo espelho do `y` **nega os termos fora da diagonal**: `E = D·T·D`. Para uma
deformação diagonal (comprimir num eixo) `E = T` e nada se vê; para uma **rodada** — que é
exactamente o leque da foto — a elipse sai **espelhada**, esticada na diagonal errada. A tinta
continuava uma lasca, e o dono leu o que havia para ler: *«sem melhorias»*.

⚠️⚠️ **E as duas metades tinham gate:** a da malha (`the_canvas_port_reports_the_local_deformation`)
e a do pincel (`the_painted_ellipse_comes_back_round_on_screen`). **As fixturas das duas eram
ALINHADAS AOS EIXOS** — `posed_arm` só translada, o outro comprime em `x` — e ali os termos fora da
diagonal são zero. *Duas metades verdes não fazem uma junção verde, e uma fixtura alinhada aos eixos
não mede uma base.*

⇒ o gate novo é a **JUNÇÃO**, e a fixtura nasce da resposta: escolhe-se a deformação de ECRÃ
(`rodar 40° + comprimir 3×`), **constrói-se o triângulo que a produz**, e mede-se a ida (a matriz
publicada é a escolhida) e a volta (os dois semi-eixos da elipse pintada chegam ao ecrã com o mesmo
comprimento). Ele vive na `ph2d-render`, que ganhou um dev-dep para a **lei canónica** do pincel em
vez de a re-implementar — a forma dos dois dev-deps que aquela crate já tinha. **Duas mutações, duas
RED**, a primeira sendo o próprio erro do report.

⚠️ E o raio composto passou a ter cerca: a que o **motor já aceita do artista**
(`BRUSH_SIZE_MAX_PX`), com o recurso nomeado — o custo de um dab cresce com o raio ao QUADRADO.

**W12 — «QUASE BOM»: A DEFORMAÇÃO DEBAIXO DE UM DAB MEDE-SE AO TAMANHO DO DAB** (3.º report com
foto, 2026-09-14: *«melhor. quase bom. Talvez artefato inevitável devido à natureza das deformações
do mesh»*). ⭐⭐⭐ **O dono tinha meia razão, e a medição diz qual metade.**

**O mecanismo.** Uma malha é **afim por TRIÂNGULO**. Dentro de um triângulo a deformação é constante
e a elipse que o pincel pinta volta ao ecrã como um disco **exacto**. Um dab que se estende por
vários triângulos era corrigido pela deformação do triângulo debaixo do **CENTRO**, e as partes dele
que caem nos vizinhos recebiam a correcção errada. ⇒ a grandeza certa não é a deformação **no
ponto**: é o melhor afim (mínimos quadrados) do mapa da malha **sobre o disco que o dab ocupa**.

**A medição** (sonda sobre dois leques × 4 raios × 3 pontos; redondeza da marca no ECRÃ, `1` = disco):

| regime | sem correcção | facete (W11b) | **ao tamanho do dab** |
|---|---|---|---|
| dab DENTRO de um triângulo | `1,86`–`2,31` | `1,006` | `1,006` (o mesmo, **ao bit**) |
| dab sobre `~2` triângulos | `2,20` | `1,21` | **`1,11`** |
| dab sobre `~4` triângulos, leque forte | `1,19` | `1,38` | **`1,18`** |

⛔⛔ **A linha do fundo é a que obrigou esta wave:** com um pincel GRANDE sobre uma malha grossa a
correcção da W11b deixava a marca **MENOS redonda do que não corrigir nada** (`1,383` contra
`1,188`). Nas 24 células a resposta nova é **sempre melhor ou igual** à da facete.

⭐ **Degenera no de sempre, e é um `if`:** se todas as amostras do bordo caem no MESMO triângulo do
centro, a porta devolve a facete **sem tocar num float** — *«byte a byte» não é uma promessa que uns
mínimos quadrados cumpram*, que é a lição da W11 repetida um nível acima.

**As peças:** [`ph2d_render::sprite_mesh_warp::warp_over`] (crate nova de módulo, com a álgebra do
triângulo extraída para uma `warp_of` **única** — a `warp_under` MORREU, porque uma porta sem
chamador e uma lei ausente produzem o mesmo app) · `mesh_uv` ganha `footprint_uv` (o raio do dab em
UV de repouso; `[0, 0]` = *«não vou pintar»*, que é o que o picking e as caixas passam) ·
`PainterTool::dab_footprint_px` (o raio **ANTES** da composição da deformação — realimentá-lo com a
saída dela seria um laço).

**Dois números MEDIDOS, não escolhidos:** `AMOSTRAS = 8` (concorda com `32` a `±0,001` nas 24
células; `4` desvia até `0,014`) e o raio de amostragem = o **raio do dab** (a metade dele deixa a
coluna do raio pequeno em `1,162` contra `1,162` da facete — **ganho zero**).

**O custo, medido** (`--release`, uma chamada por evento de ponteiro): `0,7 µs` a `128` triângulos ·
`10,9 µs` a `2 048` · **`40,7 µs`** ao tecto do `Smooth` (`7 688`) = `0,24 %` de um quadro. É por
isso que a varredura é **UMA** passagem com rejeito por caixa em UV, e não oito.

**Cinco mutações, cinco RED** — ⚠️ **TRÊS sobreviveram à primeira**, e as três nomearam buracos:
1. **um dab REDONDO não consegue medir a BASE do ajuste.** Trocar a base por um espelho multiplica a
   matriz por um factor **ORTOGONAL**, que tem os mesmos valores singulares e os mesmos eixos ⇒ com
   `flatten = 0` a resposta é a mesma **ao bit**. É a W11b uma volta mais fundo: lá a fixtura
   alinhada aos eixos não media a base, aqui é o **pincel** que não a mede. ⇒ gate novo com a elipse
   **AUTORADA** (`flatten 0,4`, `angle 30°`): o eixo maior chega a `29,8°`–`38,3°` com a lei certa e
   a `152,6°`–`164,2°` com a base trocada.
2. **nomear um argumento não é alimentá-lo** — `let footprint_uv = [0.0, 0.0];` com a chamada
   intacta passava no arch-gate. Ele exige agora a **DERIVAÇÃO** do raio do pincel.
3. **um gate só com o raio grande não pina o raio de amostragem** — a lei só se lê com a coluna do
   raio PEQUENO ao lado.

⏳ **ABERTO e NOMEADO — e é aqui que o dono tem razão:** o que sobra (`≈1,1`–`1,2` no pior regime) é
inerente a **UMA elipse por dab** — sobre um footprint em que a deformação varia, nenhum afim único
a descreve. ⛔ **Mas não é uma parede: é uma troca de RESOLUÇÃO** — os dois diminuidores medidos são
a malha mais fina (o `Smooth`) e o pincel menor.

⛔⛔ **E o item que a fila nomeava como o próximo — a deformação por DAB — foi MEDIDO e vale ~zero:**
com o pen-down do outro lado do braço a redondeza muda `≤ 0,05` e **em sinal ambíguo**, contra os
`0,10`–`0,23` que o footprint compra. A razão é que o `stamp_dabs_inner` já relê o `stroke_spec`
**ao vivo**, logo só o RAIO fica congelado no pen-down. *Um item de fila escrito por quem acabou de
curar a porta ao lado descreve o resíduo daquela porta, não o maior defeito.* Fica aberto como
**inconsistência declarada** (dois leitores do mesmo spec em instantes diferentes), não como cura.

**W12b — O ONION SAIU DA SHELL** (a cura NOMEADA do tecto, 2026-09-14). As waves do pincel tinham
deixado `the_shell_only_shrinks` com **UMA** linha de folga (`196 989` de `196 990`), e a lei do
`CLAUDE.md` §2 é clara: *quando ela reprovar, MOVA — nunca suba o número*. O motor do onion
(`timeline_onion.rs` + os dois ficheiros de teste, **1 014 linhas**) não é composição — ele só
depende de crates irmãs —, e vive agora em [`ph2d-timeline-onion`](../../crates/ph2d-timeline-onion/);
a shell fica com a **CHAMADA**, que é a fase do quadro onde ela pertence. Shell: **195 974**
(`1 016` de folga). ⚠️ **Prova exacta:** `15` testes antes, `15` depois (14 + 1 `#[ignore]`).
⛔ **E um gate apanhou a armadilha do HOWTO à primeira:** a crate nova não herdava os lints da
workspace (`unsafe` PERMITIDO nela, em todo alvo) — `[lints] workspace = true`.

**W13 — «SÓ FICA REDONDO ONDE NÃO TEMOS DEFORMAÇÃO»: o diagnóstico COMPLETO, e a cura MEDIDA e não
construída** (4.º report com foto, 2026-09-14). ⛔⛔ **A W12 melhorou e não chegou**, e a razão é que
a régua dela ainda estava errada.

⚠️⚠️ **A régua da W12 media `maior/menor` da marca — e isso é CEGO a uma marca AMASSADA.** Uma forma
lobada pode ter exactamente os mesmos dois extremos que um círculo. A régua certa é o **PERFIL visto
do ECRÃ**: para cada direcção, a que distância a cobertura acaba — e dela sai a **ondulação** (o
desvio quadrático médio do raio). *Pela terceira vez nesta linha, o que estava errado era a régua.*

**E com a régua certa a fixtura também estava errada:** o leque suave que a W12 usou lê `1,1`–`1,2`,
e a foto do dono é um leque **forte sobre uma malha grossa**. Reproduzido (`6×6`, `2,5 rad`):

| lei | `maior/menor` | ondulação |
|---|---|---|
| sem correcção nenhuma | `2,0`–`2,7` | `20`–`29 %` |
| a facete (W11b) | `2,2`–`3,2` ⛔ **pior que não corrigir** | até `40 %` |
| ao tamanho do dab (W12) | `1,3`–`2,6` | `8`–`28 %` |

⇒ **A causa é o MODELO do dab, e não a malha:** o pincel pinta **UMA elipse** na textura, e a malha
leva-a ao ecrã por um mapa **afim POR TRIÂNGULO** — com **dobras**, não com curvatura. Nenhuma
elipse única é a pré-imagem de um disco quando o dab atravessa facetes com deformações diferentes.

⛔ **Uma lei QUADRÁTICA foi construída, medida e REFUTADA:** ela ajuda nalgumas células
(`1,41 → 1,16`) e **piora noutras** (`1,33 → 1,60`), porque um polinómio liso não representa uma
**dobra** — pode até ultrapassá-la.

⭐⭐⭐ **A CURA ESTÁ MEDIDA: a lei do dab passa a ser o mapa AMOSTRADO** — o deslocamento de ECRÃ numa
grelha sobre a caixa do dab, lido por interpolação bilinear:

| lei | `maior/menor` | ondulação |
|---|---|---|
| hoje (uma elipse) | `1,33`–`2,56` | `5`–`28 %` |
| grelha `8×8` | **`1,07`–`1,14`** | `1,3`–`1,9 %` |
| grelha `24×24` | **`1,02`–`1,05`** | `0,4`–`0,7 %` |
| grelha `64×64` | `1,008`–`1,017` | `0,2`–`0,5 %` |

⭐ **E ela degenera no de hoje por CONSTRUÇÃO: uma bilinear reproduz um mapa AFIM exactamente** ⇒ em
repouso, e dentro de uma facete, a tinta é a de sempre **ao bit**.

⛔⛔ **O PREÇO, que é o que a torna decisão do dono:** a lei do dab vive na
[`FootprintDeform::apply`](../../crates/ph2d-painter-brush/src/footprint.rs), que é `Copy` de 12
bytes e o **choke point de todo pincel do app** (os quatro meios partilham-no). Uma grelha não cabe
lá dentro, e levá-la por fora toca **~12 sítios de cobertura** na crate mais quente do repo
(`ph2d-tool-painter`/`ph2d-painter-brush`, 136 k LOC) — *o raio de explosão é todo pincel do app,
para curar uma coisa que só acontece em arte presa a um esqueleto.*

**AS DUAS ALAVANCAS QUE O DONO JÁ TEM HOJE, medidas** (leque `2,5 rad`, a lei que shipa; pior de 3
pontos):

| malha | dab `0,25` | `0,125` | `0,06` | `0,03` |
|---|---|---|---|---|
| `6×6` (grossa) | `2,16` | `1,56` | `1,82` | `1,75` |
| `12×12` | `2,17` | `1,36` | `1,20` | `1,23` |
| `24×24` (o `Smooth`) | `2,15` | `1,33` | **`1,15`** | **`1,11`** |
| `48×48` | `2,15` | `1,32` | `1,15` | `1,11` |

⚠️ **Três leituras que só a tabela dá:** (a) a malha fina **satura** em `24×24` — mais peças não
compram nada; (b) **o pincel pequeno só ajuda com malha fina** (numa malha grossa um dab pequeno cai
em cima de uma aresta de facete e fica pior); (c) ⛔ **um pincel MUITO grande (`0,25`) fica em `2,15`
em TODA malha** — ali o que varia é a deformação, não a facete, e nenhuma densidade a salva.

⏳ **ESTADO: diagnóstico fechado, cura medida, construção NÃO autorizada.** A pergunta ao dono é se
vale gastar uma wave no núcleo partilhado de todo pincel para arte presa a esqueleto, ou se as duas
alavancas chegam.

## ⛔ Recusas MEDIDAS deste módulo — não as reconstrua

> ⚠️ **As seis de 2026-09-07/08 entraram aqui na auditoria de 08/09** — elas viviam só em prosa e em
> doc-comments, e o §5.0 é explícito: *arquivar sem indexar as recusas seria apagá-las.*

| recusa | o mecanismo MEDIDO |
|---|---|
| **Pôr o `VecDrivenStyle` no ledger de pré-visualização** (o «quinto de cinco» da auditoria de 08/09) | Ele é **desregistado, e não por esquecimento** — não deriva `Serialize`, e o `register_default` exige-o, logo *uma linha de registo não compila*: ele nunca entra no snapshot, no save, nem num passo de undo, que é exactamente o que o ledger compra para os outros quatro. E ele **volta ao autorado TODO QUADRO** (`settle_to_authored`), contra o `release_to_authored`, que só corre quando um motor é desligado. ⇒ acrescentá-lo poria no memo um facto que nunca esteve na fotografia. |
| **Avisar em vez de COAGIR** o nome de um clip a ser único | Os dois nomes são igualmente válidos, então o artista fica com um documento que ele não consegue reparar renomeando — a forma de uma recusa com passos extra. A lei já estava escrita no `doc.rs` e honrada nas duas portas que INVENTAM um nome; faltavam as duas que o RECEBEM. |
| **Roubar `Body`/`Joint` do gizmo de sprite** ao alargar as alças de osso a todo modo de vector | Os dois verbos (girar · deslocar) **já existem** na seta, e agarrá-los aqui trocaria a lei do arrasto dela **em silêncio**: o artista escolhe um osso com a seta e o arrasto passa a fazer outra coisa. A linha é o VERBO — entram só os quatro que nenhuma outra ferramenta sabe exprimir. |
| **Reordenar as secções do painel** para a SKELETON subir quando tem sujeito | Cura o mesmo report que o *revelar-ao-focar* (o cabeçalho a `1316 px` sobre uma faixa de `900`) e muda a ordem do painel para **toda** ferramenta e todo objecto — é decisão de produto, não de correcção, e a revelação é a metade pequena e já precedentada pela timeline. |
| **O *pole target*** para escolher o lado do joelho | Em 3D o triângulo raiz–cotovelo–ponta roda em torno do eixo raiz→ponta — um **grau de liberdade contínuo**, que um objecto no espaço fixa. No plano sobra **UM BIT**. Godot (`flip_bend_direction: bool`) e Spine (`bendDirection ±1`) escolheram o interruptor, cada um por si. ⇒ o alvo de pólo resolveria com um objecto o que um booleano resolve. |
| **Priorizar a ordem no hit-test** para resolver a colisão alça↔ponta | *Não cura: só troca a vítima.* Medido: com o osso na parede a distância ponta→alça é `0,000000`, logo quem quer que ganhe a ordem, o outro fica inalcançável. A cura foi **afastar** a alça (folga derivada do dedo da casa). |
| **Adoptar o clip ABERTO** no *Add Smart Bone* | `TimelineDoc::new()` tem **um** clip, `"Main"` ⇒ todo controlo casava com a animação principal da cena, em silêncio. |
| **Criar uma acção com o nome do osso** no *Add Smart Bone* | Veredito do dono (*«porque criar Bone Action no inspector e na timeline? Melhor não criar nada»*): duas coisas fabricadas por um clique, nenhuma pedida. |
| **Herdar o encaminhamento** pendurando os ids da fileira do lado da dobra na `VECTOR_BONE_VERBS` | Reprovado pelo `table_driven_chips_are_registered_too`: ele exige que o `populate` itere a MESMA tabela que o `paint`, e sem esse laço a fileira seguinte nasce **morta sob o dedo**. |
| **Registar o chip do selector como `Button`** | Mutação medida: o clique **acende e nunca abre lista nenhuma** (`the_action_picker_lists_the_document…` fica vermelho em *«com a lista ABERTA a acção tem de ser pintada»*). É a cicatriz da swatch dos tokens e dos dois números do Input Map. |
| **A *quadtree* graduada** como malha da imagem (F6-b) | Ela deixa **nós pendurados** na transição entre níveis, e um nó pendurado abre **FENDA** numa deformação: ele move-se pelos pesos dele enquanto a aresta do vizinho grosso se move linearmente entre as pontas. Curá-los pede a tabela de moldes de transição (5 casos a menos de rotação). A **grelha-produto** entrega o mesmo adensamento e **CONFORMA por construção** — dois vizinhos partilham a aresta inteira, sempre. |
| **Guardar quadriláteros** em vez de dois triângulos (F6-b) | Um afim não leva um quadrilátero qualquer a outro qualquer: quatro pontos são **oito equações para seis incógnitas**. «Quadmesh» aqui é a DISPOSIÇÃO dos vértices, nunca o primitivo guardado. |
| **Escolher a diagonal da célula pela forma DEFORMADA** (a mais curta das duas — a resposta clássica) | A malha trocaria de diagonal a meio de um gesto ⇒ *o desenho pisca exactamente enquanto o artista dobra.* A diagonal `a–c` fixa-se no **repouso**. |
| **Dilatar cada recorte** para fechar as costuras da pele de imagem (F6-g, 2026-09-13) | Dilatar `0,5 px` (homotetia pelo incentro) fecha a costura em arte OPACA (`16 580 → 0` px com `216` peças) e, em arte TRANSLÚCIDA (alfa `128`), compõe a faixa sobreposta DUAS vezes: `10 580 → 40 050` px com alfa errado e o pior erro `20 → 111` (`3 456` peças: `41 732 → 158 046`). Sombras suaves, bordas anti-aliased e brilhos são translúcidos ⇒ a cura estraga mais do que conserta. Sonda `ph2d-render::skin_pieces_gpu_cost::measure_seams_against_clip_dilation`. A cura que resta é rasterizar a malha SEM AA nas arestas internas. |
| **Fazer a malha SEGUIR a silhueta** em vez de a cobrir (F6-b) | Traz de volta as células deformadas da borda, que são o defeito que a wave cura. O recorte fino é do **alfa da própria arte**, de graça e ao sub-pixel — o *Expansion* do *Puppet* do AE. |


| O quê | Por quê | Onde |
|---|---|---|
| Guardar os **pesos** numa tabela por ordem de varredura | é o *vector paralelo* que o `corner_radius` proíbe por escrito; derivar custa **0,146 %** de um quadro | [`ph2d-skeleton-ecs`](../../crates/ph2d-skeleton-ecs/src/lib.rs) |
| Referenciar um osso por `Entity::to_bits()` | **medido `0 de 2`**: o undo re-spawna e os bits são ids de alocação — a pele morria em silêncio; e o `from_bits` **aborta o processo** com bits de outra sessão | degrau 122 da escada |
| Misturar FK↔IK pelas **posições** das juntas | encurta os ossos (a corda é mais curta que o arco); a mistura é sobre o **ângulo** | `ph2d_skeleton::blend_angle` |
| Um `Driver` novo no `preview_drive` para a âncora | dois memos sobre o **mesmo componente** repõem `Transform`s diferentes na mesma fotografia, e quem ganha é a ordem do `BTreeMap` | `skeleton_goal` (cabeçalho) |
| `chain = 0` (*até à raiz*) como valor de nascimento | é o default do Blender e a queixa nº 1 documentada da feature dele: ao primeiro arrasto o esqueleto inteiro dobra | `DEFAULT_CHAIN = 2` |
