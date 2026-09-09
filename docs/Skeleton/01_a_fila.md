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
| F6 | **A segunda mídia** (raster/Flip) | ⛔ **bloqueado**: precisa de uma malha sobre a imagem, que não existe — meça o preço antes de prometer |
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
([`flip_fit_cache_tests.rs`](../../shells/desktop/src/flip_fit_cache_tests.rs)). ⚠️ O ficheiro é
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


| O quê | Por quê | Onde |
|---|---|---|
| Guardar os **pesos** numa tabela por ordem de varredura | é o *vector paralelo* que o `corner_radius` proíbe por escrito; derivar custa **0,146 %** de um quadro | [`ph2d-skeleton-ecs`](../../crates/ph2d-skeleton-ecs/src/lib.rs) |
| Referenciar um osso por `Entity::to_bits()` | **medido `0 de 2`**: o undo re-spawna e os bits são ids de alocação — a pele morria em silêncio; e o `from_bits` **aborta o processo** com bits de outra sessão | degrau 122 da escada |
| Misturar FK↔IK pelas **posições** das juntas | encurta os ossos (a corda é mais curta que o arco); a mistura é sobre o **ângulo** | `ph2d_skeleton::blend_angle` |
| Um `Driver` novo no `preview_drive` para a âncora | dois memos sobre o **mesmo componente** repõem `Transform`s diferentes na mesma fotografia, e quem ganha é a ordem do `BTreeMap` | `skeleton_goal` (cabeçalho) |
| `chain = 0` (*até à raiz*) como valor de nascimento | é o default do Blender e a queixa nº 1 documentada da feature dele: ao primeiro arrasto o esqueleto inteiro dobra | `DEFAULT_CHAIN = 2` |
