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
| F3 | **Smart Bones** (Moho) | ✅ **FECHADO** (2026-09-08) — ver abaixo |
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

⚠️ **`PROJECT_SCHEMA` 125 → 126** (componente registado novo; o registo do esqueleto vai de **5 para
6** e o catálogo idem — conte o delta). ⚠️ E o gate `every_registered_component_has_a_descriptor`
apanhou-me a registar sem descrever: *o Inspector e o registo são duas listas, e o gate é o que as
ata.*


## ⛔ Recusas MEDIDAS deste módulo — não as reconstrua

| O quê | Por quê | Onde |
|---|---|---|
| Guardar os **pesos** numa tabela por ordem de varredura | é o *vector paralelo* que o `corner_radius` proíbe por escrito; derivar custa **0,146 %** de um quadro | [`ph2d-skeleton-ecs`](../../crates/ph2d-skeleton-ecs/src/lib.rs) |
| Referenciar um osso por `Entity::to_bits()` | **medido `0 de 2`**: o undo re-spawna e os bits são ids de alocação — a pele morria em silêncio; e o `from_bits` **aborta o processo** com bits de outra sessão | degrau 122 da escada |
| Misturar FK↔IK pelas **posições** das juntas | encurta os ossos (a corda é mais curta que o arco); a mistura é sobre o **ângulo** | `ph2d_skeleton::blend_angle` |
| Um `Driver` novo no `preview_drive` para a âncora | dois memos sobre o **mesmo componente** repõem `Transform`s diferentes na mesma fotografia, e quem ganha é a ordem do `BTreeMap` | `skeleton_goal` (cabeçalho) |
| `chain = 0` (*até à raiz*) como valor de nascimento | é o default do Blender e a queixa nº 1 documentada da feature dele: ao primeiro arrasto o esqueleto inteiro dobra | `DEFAULT_CHAIN = 2` |
