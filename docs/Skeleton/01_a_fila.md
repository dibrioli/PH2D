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
| **F8** | ✅ **BENDY BONES (B-Bones) — FECHADO em 2026-09-15**, da lei ao painel ([handoff](handoffs/HANDOFF_O_OSSO_QUE_DOBRA_2026-09-15.md)) | Um osso ganha `segments` + duas alças e **arqueia**: ele parte-se em `N` sub-ossos ao longo de uma Bézier, o desenho e o dedo seguem a curva, e o painel oferece os dois controlos. ⭐⭐⭐ **A LEI DA PELE NÃO MUDOU UMA LINHA** — o `Skin` já misturava `N` poses RÍGIDAS por peso, que é exactamente o que um B-Bone é; o que mudou foi **quem produz**, e era **um** sítio (`resolve_with`). ⛔⛔ **E esta célula dizia que o B-Bone «ataca na ORIGEM» a queixa das *«arestas retas ao dobrar»* — REFUTADO** pela recusa medida um bloco abaixo (subdividir com a população de amostras constante **piora**: `2,61 % → 4,94 %` a `24` sub-ossos): *o B-Bone é uma feature de AUTORIA — um rabo em S, um membro flexível —, não a cura da dobra.* ⭐⭐ **O ponto neutro é exacto POR CONSTRUÇÃO** (a fábrica colapsa num osso só quando a curva é recta, e mesmo sem colapsar o frame seria a identidade ao bit) ⇒ todo rig já autorado desenha-se e deforma-se **ao bit** como antes. ⚠️ `PROJECT_SCHEMA` **+1** — conte o DELTA. **Tecto MEDIDO: `MAX_SEGMENTS = 32`** (`17,9 %` de um quadro com um osso curvo sobre 20 000 pontos; a `64` um par come o quadro) — a tabela vive no doc da const. ✅ **OS TRÊS ABERTOS FECHARAM EM 2026-09-16.** **(1)** O esticão deixou de VARIAR ao longo do osso — os nós saem agora da **CORDA** e não do parâmetro (`12,63 % → 0,000 %` com as alças a `0,2 L`; `82,01 % → 0,000 %` a `0,6 L`; `1 051,95 % → 0,000 %` com as alças cruzadas no eixo). ⛔⛔ **E a cura publicada — equalizar o ARCO — NÃO chegava**, o que só a varredura da densidade disse: ela deixa um piso que **não desce com a tabela** (`1,22 %` a `0,6 L`, igual de `16` a `32` amostras), porque *arcos iguais dão cordas desiguais* e a grandeza que o artista vê é a corda. ⚠️ **E a objecção registada na recusa era verdadeira e não mordia** (*«um somatório de cordas não devolve `L` ao bit»*): o somatório **nunca corre** no ponto neutro — *uma recusa que nomeia um custo tem de dizer em que CAMINHO ele é pago*. **(2)** As alças **pegam-se no canvas** (duas alças de Bézier, com as hastes até à raiz e à ponta) — ⛔ e a armadilha foi que no ponto NEUTRO a alça está **em cima do eixo**, logo a competição por proximidade de sempre torná-la-ia inalcançável no único estado em que todo osso nasce: ela é a única que ignora o corpo, e paga um raio apertado cujo recurso é o comprimento que sobra para o verbo de girar. **(3)** As **tangentes dos vizinhos** existem (`Curve Handles: Manual | From Chain`), e o ponto neutro é **exacto** porque elas saem da transformação RELATIVA e não de uma volta pelo mundo. ⚠️ `PROJECT_SCHEMA` **+1** — conte o DELTA. Cena **`PH2D_VEC_BONE_SMOKE=1`**. |
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

✅ **(curada pela F3-b, logo abaixo — conferido em 2026-09-16)** ~~DÍVIDA NOMEADA~~: o painel mostra os dois ângulos e **não diz qual acção está ligada** — falta a
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

✅ **FECHADOS, os seis — e esta linha dizia «FICA ABERTO» até à auditoria de 2026-09-16**, que a
conferiu contra as secções abaixo: o gizmo do limite (**F3-h**), o *Remove Smart Bone* (**F3-i**), o
silêncio e o aviso nas três ordens (**F3-j**), os nomes repetidos e a 5.ª escrita (**F3-k**, a
segunda como recusa medida). A redacção de então fica, como contraste:
*«FICA ABERTO desta auditoria (com mecanismo, sem cura): Remove Smart Bone não devolve a pose
autorada (o *Remove IK* devolve, e o preço é a lista de N entidades × 4 drivers em vez de uma
corrente) · dois clips podem partilhar o NOME e o controlo percorre o primeiro, calado · o ledger
cobre 4 das 5 escritas do `write_prop` (falta o `VecDrivenStyle`, que é desregistado) · o gizmo do
limite é pintado e ACENDE em todo modo de vector e só é agarrável no modo Osso · o aviso do osso
governado só dispara na ordem *IK → Smart* · e quatro verbos da secção morrem em silêncio na janela
*«nenhum osso em foco»* (o braço que fala cobre só os dois da âncora).»*


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

✅ **FECHADOS os dois (conferido em 2026-09-16).** *«Um **buraco** no meio de uma forma não é traçado,
e a malha cobre-o»* era verdade para o CONTORNO triangulado, que o produto deixou de usar na F6-b: a
grelha decide célula a célula, e um furo maior que uma célula fica de fora — hoje com gate
(`a_hole_in_the_art_is_a_hole_in_the_mesh`, morto por mutação). E *«o número de triângulos por
quadro não foi medido sob cena cheia»* está medido desde 13/09 (a tabela do `SKIN_FRAME_PIECES`), e a
rota por triângulos texturados é a de hoje (F6-i). ⚠️ O que fica, e é da GRELHA: duas partes
separadas por menos de uma célula (mais a folga) ficam ligadas.

✅ **CURADO e por isso RETIRADO desta lista (auditoria de 2026-09-16):** *«a malha é só o contorno, e
um membro grosso dobra pela borda»*. A própria linha acima já dizia *«o dono viu isso na primeira
olhada e a F6-b curou-o»* — ela ficou aqui com o ⏳ à frente durante cinco dias, e um item marcado
como aberto ao lado da prova de que fechou é exactamente o que faz alguém reconstruir trabalho pago
(`CLAUDE.md` §5.0).


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

✅ **JULGADOS** (conferido em 2026-09-16): os três números (`fine 10` · `coarse 26` · `radius 40` px)
são de **PRODUTO, não tectos de recurso**, e o smoke do dono julgou-os — a F6-c abre com *«malha bem
desenhada»*. Desde 2026-09-15 a contagem é um ORÇAMENTO (`target_tris`) e estes três são a FORMA da
graduação.


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
- ✅ **O custo FOI medido, e esta nota envelheceu duas vezes** (re-conferida contra o código em
  2026-09-14). Ela dizia *«cada triângulo é um `push_clip` do Vello»* — desde a **F6-i** a pele é uma
  **malha no passe de sprites** e aquele buffer já não é gasto por ela; e dizia *«não foi medido»* —
  a tabela do [`SKIN_FRAME_PIECES`](../../crates/ph2d-skeleton-live/src/skin_image.rs) mede-o desde
  13/09: **`1,08 µs` por peça entregue**, com o envio e o desenho na conta (`0,039 µs`). O orçamento
  do QUADRO é `1 543` peças (`1/10` de um quadro de 60 fps), com cerca em tempo de compilação nas
  duas pontas.
  ⇒ ✅ ~~O que fica ABERTO é outra coisa, e é uma DECISÃO DO DONO: o `Smooth` continua a nascer
  desligado~~ — **decidido**: o `Smooth` é o de fábrica desde 2026-09-14, e o dono manteve os dois em
  2026-09-16 (F6-p).
- ⛔⛔ **O MAPA DOBRA SOBRE SI MESMO em dobras fortes, e isso NÃO é o que esta wave curou.** Medido:
  a `60°` por junta a área no pior ponto é **`−0,129`** (negativa ⇒ inversão) e `0,22 %` da imagem
  está dobrada; a `150°` são **`−1,017`** e `2,52 %`. O `Smooth` desenha o campo com fidelidade —
  **inclusive a dobra**. A causa é o gradiente dos pesos com `raio = comprimento do osso`, e a
  família de curas (pesos mais apertados · *centers of rotation* · o alcance por osso) não foi
  medida.
- ✅ ~~O `max_split = 6` tem tecto de CPU~~ — **obsoleto desde 2026-09-16**: a lei do produto é a
  adaptativa, cujo tecto é o orçamento do QUADRO (`SKIN_FRAME_PIECES`); o `max_split` só serve a porta
  de bissecção uniforme. A tolerância de `0,5 px` é de PRODUTO.

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

✅ **FECHADO pela W9, logo abaixo (conferido em 2026-09-16)** — ~~NOMEADO e não curado~~: o gémeo do Flip — um objecto de OUTRA família continua a publicar caixa
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

✅ **DISSOLVIDO pela ordem do dono de 2026-09-15** (sob o Painter a sprite pintada é desenhada
ACHATADA — `skin_suspend::sprite_achatada` —, logo o quad de repouso É o ecrã): ~~o **chrome** do Painter continua no quad de repouso~~ (o anel do pincel segue o
ponteiro e só o TAMANHO dele sai do afim; a curva, a linha, o gizmo de deformação, os gizmos de
selecção, os crachás e a humidade desenham-se em posições de IMAGEM) — numa arte dobrada eles ficam
no sítio de repouso. Não piorou com esta wave: antes a tinta estava errada **com** eles.

⚠️⚠️ **E esta linha dizia também que *«o conta-gotas do BgRemoval usa uma caixa alinhada aos eixos
que ignora rotação e malha»* — CURADO na F6-m, que está QUATRO SECÇÕES ABAIXO neste mesmo ficheiro**
(`ph2d_sprite_screen::uv_sob_o_ponteiro`, com as três entradas da Remoção de fundo a passarem por
uma porta só). ⛔ Ela sobreviveu à auditoria de 2026-09-16 e foi relatada ao dono como aberta.
*Uma nota obsoleta ao lado da secção que a fecha é pior que uma nota ausente: a ausente não é
acreditada.*

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

✅ **DISSOLVIDO pela mesma ordem (a arte pintada está achatada):** ~~a deformação é a do **pen-down**, para o traço inteiro~~ (é o `stroke_spec`
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

✅ **CURADO pela F6-l (`1,002`) e depois DISSOLVIDO pelo achatamento** — a redacção de então: o que sobra (`≈1,1`–`1,2` no pior regime) é
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

⛔⛔ **VEREDITO DO DONO (2026-09-14): *«Deixe como está.»*** ⇒ a cura amostrada **NÃO se constrói**, e
o caminho recomendado ao artista é **`Deform = Smooth`** com pincel médio ou pequeno (`~1,15`, quase
redondo). ⚠️ **Isto é uma DECISÃO registada, não trabalho pendente** — *um ⛔ «recusado com motivo» e
um ⏳ «ninguém fez» leem igual numa tabela* (`CLAUDE.md` §5). Quem reabrir isto tem de trazer ou um
preço menor (uma forma de a lei viver DENTRO do `FootprintDeform` sem tocar os ~12 sítios) ou um
report novo que a tabela das alavancas não explique.

⛔ **E a alavanca tem um limite MEDIDO que faz parte do veredito:** um pincel muito grande (`0,25` de
UV) fica em `2,15` em **toda** densidade de malha — ali o que varia ao longo do dab é a própria
dobra, e a malha fina não a salva.

**W14 — O AUTOKEY GRAVA A POSE QUE A MÃO FEZ, e não só o osso debaixo do dedo** (2026-09-14,
escolhido como fase seguinte depois de o dono recusar a W13).

⛔⛔ **O defeito, medido no passe antes de tocar em código:** a população que o `autokey_pass`
amostrava era a **SELECÇÃO** (`hero.gizmo.iter_selected()`). Puxar a PONTA de uma corrente é
cinemática inversa — a corrente **inteira** dobra —, e agarrar um osso **não** o selecciona
([`despacho_clique_select`](../../shells/desktop/src/input_dispatch/despacho_clique_select.rs):
o gesto escreve `bone_pose` e devolve). ⇒ a pose que o artista acabou de fazer ia para a animação
**por um osso só**, e o resto dela perdia-se no primeiro instante em que o apply voltasse a escrever
pelas curvas. **Gate red-first, e o número foi `(2, 3)`**: numa corrente de dois ossos dobrada
inteira, a raiz ficou com as suas duas chaves de sempre e só a ponta recebeu a nova.

⭐⭐ **A população certa já era CALCULADA:** a `timeline_bridge::maos_do_quadro` nasceu na **W8** para
o apply *não* escrever por cima da mão, e é exactamente a mesma pergunta — *uma porta com um
consumidor só estava a metade do trabalho que sabia fazer*. A cura é a população passar a ser
**selecção ∪ mão**, sem repetidos, com a selecção à frente.

⚠️⚠️ **E ela NÃO espalha chaves pelo esqueleto — quem filtra é o DIFF**, e isso tem gate próprio
(uma corrente de TRÊS com só dois movidos: `(2, 3, 3)`). *A mão diz «olha também para estes», nunca
«cunha estes».* Sem esse gate a cura seria indistinguível do defeito OPOSTO — vinte ossos, vinte
faixas novas por arrasto.

⭐ **A decisão de quem o passe olha saiu para uma PORTA** (`autokey_pass::populacao`), e não por
gosto: com a lista construída em linha, **duas mutações sobreviveram** (apagar o filtro de repetidos
e perder a mão do GIZMO — que a selecção cobre **por acaso**, e por acaso não é uma lei). Com a porta
extraída e gateada, **cinco de cinco RED**.

⭐⭐ **E a quinta apanhou uma lei que vivia SEM gate desde 2026-09-08:** *pré-visualização não é
autoria* — um osso que um motor conduz (o osso inteligente, a âncora de IK) escreve pose **entre** o
apply e este passe, e cunhar dali faria uma chave da saída do motor. O filtro existia; nada o
protegia. Hoje o gate exige que ele salte o conduzido **mesmo estando na mão**.

⚠️ **O `run` passou a receber `&SimWorld`** (era `&World`): a mão precisa de perguntar pela cadeia de
ossos. É uma mudança de assinatura interna da shell.

⚠️⚠️ **E uma fixtura quase FABRICOU um defeito que não existe:** um gate de UM quadro sobre um rig
**novo** (sem curva nenhuma) lê *«não grava nada»* — e o passe está são. Sem curva não há de que a
pose esteja «fora», então quem mede a mudança é a **BASELINE**: o 1.º quadro do gesto estabelece-a e
os seguintes cunham a diferença. *Um arrasto são DOIS quadros, e um teste que só corre um mede outro
programa.* O gate ficou, escrito como o gesto de facto é — e ele é o **primeiro gesto de toda
animação** deste app.

**W14b — *«AINDA NÃO FUNCIONA PARA IK»*: com uma restrição VIVA o sujeito da autoria é OUTRO**
(report do dono, 2026-09-14, logo a seguir a aprovar o smoke da W14).

⭐⭐⭐ **E o módulo já o dizia por escrito.** O cabeçalho do [`goal`](../../crates/ph2d-app-skeleton/src/goal.rs):
*«o que o artista autora é a pose da **ÂNCORA** (e os três números da restrição); a rotação dos ossos
governados é **derivada** dela»*. E o `bone_pose::pose` faz exactamente isso: com um `IkGoal` vivo,
arrastar a ponta **não posa osso nenhum** — chama o `goal::drag_anchor`, que escreve o `Transform`
do **ALVO**.

⛔⛔ **A população da W14 não podia funcionar aqui, e por DUAS razões ao mesmo tempo:**
1. o alvo **não é osso** — ele não entra pelo `skeleton_of`;
2. os ossos que a IK dobrou estão todos **sob condução do solver** (`goal::solve` regista
   `preview.driven` em cada um, todo quadro), e o filtro de pré-visualização salta-os — **e está
   certo**: cunhar dali faria chaves da saída do motor.

*A pose inteira era derivada, e a única coisa autorada do gesto não estava na lista.*

⇒ **A cura é uma linha na `maos_do_quadro`:** o esqueleto que a mão segura passa a trazer também o
**ALVO de cada âncora** dele, por uma porta nova (`goal::target_of`, extraída do `drag_anchor` — *a
mesma procura, dois consumidores*). ⭐ E ela serve as **duas** metades daquela porta: o apply também
não pode escrever por cima do alvo enquanto o dedo o move.

**Duas mutações, duas RED** (o alvo fora da mão · o `target_of` a devolver o alvo errado).

⚠️⚠️ **E a fixtura quase fabricou um defeito pela SEGUNDA vez nesta wave:** com `keys_mode = false`
(o default do `TimelineState::new()`) o gate falha **com a cura aplicada** — uma âncora de
trajectória é geometria do CLIP e o documento **recusa-a** fora da aba *Keys* (Enio, 2026-07-31), que
é a aba de omissão do app. *Um gate que herda um default de construtor mede outro programa;* o do
motion path já trazia esta nota, e foi preciso pagá-la outra vez.

**W14c — *«AO ACRESCENTAR O IK O OSSO PERDE INFLUÊNCIA SOBRE A PONTA DA MALHA»*: a ORDEM DO QUADRO,
e o próprio código já escrevia a lei violada** (report do dono com duas fotos, 2026-09-14).

Nas fotos o **gizmo** do osso está na pose resolvida pela IK e a **arte** não o acompanha — duas
leituras da mesma pose, e a que o artista vê é a errada.

⭐⭐⭐ **A lei estava escrita, para a OUTRA mídia.** O comentário que punha a âncora imediatamente
antes do `skeleton_live::recook` (a pele **vectorial**) diz à letra: *«ela escreve a pose dos ossos,
e o recook é quem transforma a pose em geometria. Ao contrário, a pele mostraria a pose do quadro
anterior.»*

⛔⛔ **A SEGUNDA MÍDIA chegou depois e não herdou a arrumação.** A malha de uma imagem presa é posta
no `attach_skin_meshes`, dentro da `fase_sim_extract` — que corre na metade da **SIMULAÇÃO**, antes
da fase de canvas inteira onde os dois motores viviam. ⇒ a malha era construída da pose de **ANTES**
do solver.

⚠️⚠️ **E não é um atraso de um quadro, é PERMANENTE:** o apply da timeline (`fase_timeline_drain`)
corre na mesma metade e reescreve todo osso **keyado** pela curva, mesmo a tempo de a malha a ler.
Cada quadro: apply repõe a curva → a malha lê a curva → o solver escreve a IK → o gizmo mostra-a.
*É por isso que o sintoma só aparece com o IK: sem ele ninguém escreve pose depois do extract.*

⇒ **A cura é a ORDEM:** os dois motores (`skeleton_smart::drive` e `skeleton_goal::solve`) mudaram-se
para uma fase própria (`fase_skeleton_drives`) **entre o apply e o extract**. A ordem interna entre
eles continua a de sempre e continua load-bearing (o osso inteligente primeiro — ele é a pose de
BASE; a âncora depois — ela persegue um alvo e tem de ver a pose já corrigida), e a pele vectorial
continua a correr depois dos dois, agora por uma margem maior.

**Dois gates, três mutações RED:** a ordem medida **dentro de UM ficheiro só** (⛔ concatenar dois
para medir uma ordem é fraude — tudo o que está no segundo vem depois de tudo o que está no
primeiro; a lição que o corte do teclado do sculpt pagou em 2026-09-04), e **a outra porta** — a fase
de canvas não pode voltar a chamá-los, senão eles correm **duas** vezes por quadro.

⚠️ **A generalização NÃO foi feita, e é nomeada:** este quadro tem outros motores que escrevem pose
(a física, as curvas, os nós de Motion) e nada os obriga a correr antes de quem lê. *Nenhuma sonda
deste repo pergunta «quem escreve pose depois de ela ser lida?»* — o que existe agora é este gate,
sobre estes dois.

**W15 — OS TRÊS ITENS DO REPORT DO IK** (2026-09-14: *«IK chain mostra 0 ao inserir IK. Não temos
uma linha indicativa do IK Chain. IK Bend só funciona se o IK Chain for 2»*). ⭐ **Dois eram o mesmo
defeito, e o terceiro não existia.**

**(a) O painel MENTIA sobre os cinco números, não só sobre o `Chain`.** O `populate` regista os
campos com `value: 0.0`, o publicador (`state::set_current_bone_ik`) existe e a shell chama-o todo
quadro — e os valores morriam no `state`, porque quem pinta a fileira (`labeled_number_field`) tira o
valor do **WidgetStore** e ninguém lá escrevia. *Um publicador sem quem o leia e uma lei ausente
produzem o mesmo painel.* ⚠️ **O gate red-first acusou `Length` primeiro**: os cinco mostravam `0`, e
o dono só notou o `Chain` porque ali `0` significa **«até à raiz»** — o default do Blender, que é a
queixa nº 1 documentada da feature. ⇒ **UMA tabela, DOIS consumidores** (`campos_do_osso` /
`campos_da_ancora`): quem pinta e quem semeia percorrem a mesma lista, e um campo novo traz o valor
no tuplo ou **não compila**. A semeadura é a `set_number_value`, que **preserva a edição em curso**
(o campo com foco e o arrasto) — semear por cima do cursor seria trocar um defeito por outro.

**(b) A LINHA DO `IK Chain` EXISTE** — uma **faixa** por baixo dos ossos governados, da raiz da
corrente à ponta, com um **X** na raiz (é ali que o `Chain` pára) e a cor a seguir a selecção. ⚠️ A
população é a **MESMA** do solver (`goal::governed`): uma lista derivada por outro caminho seria a
segunda resposta à mesma pergunta, e no dia em que divergissem o desenho estaria a mentir sobre quem
dobra. ⚠️ Desenhada **antes** dos ossos e a `35 %` de opacidade — ela é um realce, não um desenho
novo: a cheio e por cima esconderia o corpo do osso, que é o que a mão agarra.

**(c) ⛔⛔ *«IK Bend só funciona se o Chain for 2»* — MEDIDO, e a premissa é FALSA.** O lado é honrado
de **2 a 6** ossos, em três distâncias de alvo, com a ponta a chegar (`erro < 1e-2`). Os dois
caminhos aplicam-no por mecanismos diferentes — o fechado escolhe o sinal da lei dos cossenos, o
FABRIK **arqueia e espelha** a corrente antes de iterar — e ambos respondem. ⇒ o que o dono viu foi
**(a)**: o painel dizia `Chain = 0` e o número que ele lia não era o que a restrição usava. *Um gate
que defende uma premissa refutada vale mais que a refutação sozinha: ele impede que ela volte.*

**Cinco mutações, cinco RED** — o painel sem semeadura · a faixa a ignorar o `Chain` · a faixa fora
do quadro · o espelho do FABRIK apagado · o valor do `Chain` trocado na tabela.

⚠️ **E o tecto de LOC apanhou o gate novo:** o `reach.rs` foi a `731` de `700` e partiu-se por
RESPONSABILIDADE (`reach_chain_side_tests.rs`) — *para que lado dobra uma corrente de N* não é a
mesma pergunta que *onde ela chega*. ⛔ Nunca subir o número.

✅ **FECHADO na W16b, e esta nota envelheceu no mesmo dia em que foi escrita** (re-conferida contra o
código em 2026-09-14): ela dizia que re-capturar ao mudar o `Chain` *«é decisão de produto»* — e o
dono decidiu-a horas depois, pelo nome (*«o lado da dobra é capturado no momento em que carrega Add
IK e sempre que IK Chain for mudado»*). A porta é a `goal::side_for_chain`, com os dois chamadores.
⚠️ E o modo **MISTO** é a excepção, também já fechada: ali a re-captura APAGARIA a escolha do
artista, e o misto não precisa dela.

**W16 — *«IK BEND NÃO ESTÁ CONSISTENTE PARA MAIOR QUE 2. MUDA O ÂNGULO DE LADO»*** (report do dono,
2026-09-14, depois de a W15 refutar a premissa anterior). ⭐⭐⭐ **Ele tinha razão, e a minha régua é
que era grossa.**

⛔⛔ **A W15 mediu o LADO (um bit) e ele estava certo; o que muda é o ÂNGULO.** Medido agora: a MESMA
restrição — mesmo alvo, mesmo lado — resolvida a partir de **quatro poses de partida diferentes** dá
**quatro poses finais diferentes**:

| ossos | pior desvio entre as quatro | em fracção do alcance |
|---|---|---|
| **2** | `0,0000` | **`0 %`** |
| 3 | `0,52` | `17 %` |
| 4 | `1,15` | `29 %` |
| 5 | `1,58` | **`32 %`** |

⇒ **o FABRIK é sensível à pose inicial, e cada quadro partia do resultado do anterior.** A dois ossos
isto nunca aconteceu porque ali a lei é **fechada** e não olha para a pose — que é exactamente o
*«só funciona se o Chain for 2»* dos dois reports, visto pelo lado certo.

⇒ **Com um lado AUTORADO a corrente parte sempre de uma pose canónica** — um arco de seno da raiz ao
alvo, de amplitude igual à **folga** (a altura do triângulo isósceles de lados `total/2` sobre a
base `d`: zero no limite do alcance, máxima com a corrente dobrada em dois). O resultado passa a ser
uma **função de `(raiz, comprimentos, alvo, lado)`**.

⚠️⚠️ **E o arco tem de ser de VERDADE:** a 1.ª cura deitou a corrente RECTA e deixou o arqueamento de
`1e-3` dar-lhe o lado — e a **8 ossos ela caiu para o lado errado**. Aquele arqueamento existe para
dar ao FABRIK *por onde cair*, não para escolher a pose: uma perturbação de um milésimo do alcance
não sobrevive a quarenta passagens.

⚠️ **Só com lado autorado.** Com `Keep` — o gesto de arrastar a ponta — partir da pose que lá está é
o DESENHO (*um gesto preserva o que se vê, uma restrição defende o que se autorou*), e aquele
caminho fica byte a byte o que era.

⛔⛔ **E a cura DISSOLVEU a premissa de um gate que estava certo:** o
`a_locked_side_is_stable_not_a_flip_flop` exigia `primeiro > 1e-12` acima de dois ossos — *«a fixtura
tem de produzir movimento, senão mede o nada»* — e **esse movimento era o defeito**. Hoje ele afirma
o contrário e mais forte (*toda* corrente com lado autorado é ponto fixo), e a anti-vacuidade mudou
de sítio: ela vive no caminho do GESTO, que é o único que ainda refina.

⭐⭐ **E uma mutação SOBREVIVENTE deu uma lei mais forte:** o bojo do arco multiplicava pelo lado
pedido, e apagar esse factor não reprovava nada — o espelho a seguir já corrige. ⇒ *o lado tem UM
dono*, e daí sai a lei nova: **`Ccw` e `Cw` são espelhos EXACTOS** um do outro sobre a recta
`raiz → alvo`, o que um bojo com sinal próprio não garantiria.

**W16b — ORDEM DO DONO: o lado é capturado no `Add IK` E sempre que o `Chain` mudar.** A razão é
geométrica: o lado descreve **uma corrente**, e subir o número troca a corrente por outra — o bit
guardado passaria a falar de uma geometria que já não é a que está debaixo do artista. ⇒ uma porta
(`goal::side_for_chain`) com os **dois** chamadores. ⚠️ Lida com a corrente **NOVA** (com a velha
devolveria o lado que já lá está) e **antes** de escrever (o `captured_side` precisa de `&sim` e o
`IkGoal` é um empréstimo mutável do mesmo mundo).

**Cinco mutações, cinco RED.** ⚠️ E o `reach.rs` voltou ao tecto (`734` de `700`): partiu-se por
RESPONSABILIDADE em `reach_side.rs` — *onde a corrente chega* e *para que lado ela dobra* são duas
perguntas, e os três passos do lado (semear · arquear · espelhar) são um assunto só.

### F5-c — ✅ **O MODO MISTO: ossos para os dois lados, cada um a guardar o seu** (ordem do dono, 2026-09-14)

> *«ótimo. Funcionou. Mas além de CCw e CW precisamos de um modo misto onde temos ossos com ângulos
> para os dois lados. e cada osso mantêm sua direção inicial»*

O `Ccw` e o `Cw` põem **todas** as juntas do mesmo lado — é o que um bit por corrente pode dizer. O
misto é a quarta variante do [`BendSide`], e o que ela guarda não é um bit: é **o lado de cada
junta**, lido da pose que o artista desenhou.

**⛔⛔ E ele NÃO é uma variante do FABRIK — isso foi construído, medido e REFUTADO.** Pôr o sinal de
cada junta como restrição dentro das varreduras (a forma clássica, *FABRIK with constraints*)
**oscila**: a ida prega a ponta no alvo e re-resolve a corrente inteira sem olhar aos sinais, a
correcção desfaz isso, e as duas leis andam à roda. Medido no corpus do zig-zag, o erro da ponta
**CRESCE** passagem a passagem em 2 dos 12 alvos:

| alvo | passagem 1 | … | passagem 12 |
|---|---|---|---|
| `n=3`, alvo a `0,7` do alcance | `0,5156` | ↗ | `1,5797` |
| `n=5`, alvo a `0,7` do alcance | `1,8707` | ↗ | `1,9912` |

⚠️ **E os dois TÊM pose exacta** — uma busca cega sobre os ângulos, com os sinais como restrição,
acha erro `0,0` nos dois. ⇒ *não era a restrição a ser impossível, era o laço.*

⛔ **Três curas foram medidas e nenhuma resolve:** deitar a junta violada na **fronteira** (a
projecção que menos a move — e a que mais custa à corrente, porque desfaz a DOBRA: `2,43` de erro
num alvo a `1,52` de uma corrente de alcance `4`), **espelhá-la** (conserva o ângulo e troca o lado
— melhor, e ainda deixa dois alvos por resolver), e o **amortecimento** entre as duas, varrido em
`0,0 · 0,2 · 0,4 · 0,5 · 0,6 · 0,8 · 1,0`: nenhum valor resolve os dois, e os intermédios são
**piores** que qualquer um dos extremos. *Não é afinação.*

**⭐⭐⭐ A lei que fica é a DESCIDA JUNTA A JUNTA com a parede de cada lado** — e ela ganha por uma
PROPRIEDADE, não por um número. Rodar a cauda em torno de uma junta muda **essa junta e mais
nenhuma** (as de jusante viajam rigidamente, a de montante não se mexe), logo o lado pedido vira um
**intervalo fechado** naquele ângulo; e a pose de partida, sendo a autorada, já está dentro dele. ⇒
**cada passo nunca piora a ponta, e um laço que nunca piora não pode entrar em ciclo.** Corpus
inteiro: **12 de 12** no alvo (erro `≤ 0,0004`), **12 de 12** conjuntos de sinais intactos.

**⭐⭐ A parede tem margem nas DUAS pontas, e as duas são load-bearing.** Uma junta exactamente
**recta** não tem lado legível — e uma dobrada a **`π`** também não (ali o produto vectorial é
zero). Parar em qualquer das duas perderia a feature **no quadro seguinte**, em silêncio: o lado é
re-lido da pose a cada resolução. A margem é `8 × STRAIGHT` (`0,46°`), e as duas mutações que a
apagam são RED.

**⭐⭐⭐ E ele parte da POSE AUTORADA, que é o que dá sentido a «inicial».** Lido da pose viva,
«inicial» seria *o que o solver deixou no quadro anterior* — estável enquanto o alvo está ao alcance
(o modo é ponto fixo, e há gate), e **apagado para sempre** no primeiro arrasto que o leve para fora
dele, porque fora do alcance a resposta certa é a RECTA e uma recta não tem lado nenhum para ler.
⇒ uma porta nova no ledger (`PreviewDrive::authored`, que responde por **uma** entidade sem deslocar
o mundo como a `substitute_authored` faz) e a reconstrução da corrente desenhada
(`goal_authored::authored_joints`). ⭐ **O truque é que a junta É a rotação local do osso de baixo** —
num encadeamento pai→filho o ângulo de mundo acumula, logo a diferença dos dois ângulos de mundo
**é** o `local` do filho: não é preciso pose de mundo autorada nenhuma. Gate:
`a_mixed_chain_survives_a_drag_out_of_reach` (RED sem a cura).

⚠️ **Os outros três modos ficam byte a byte** — o lado deles vive num campo do documento, não na
pose.

**Quatro mutações RED de cinco.** ⚠️ **A quinta SOBREVIVE e fica registada no fonte**: trocar a
escolha do passo (mínimo exacto sobre o intervalo) por um `clamp` linear não reprova gate nenhum —
ela muda o passo em **165** varreduras do corpus (o `clamp` escolhe o extremo errado: `−0,027` onde
o mínimo está em `+3,099`) e a descida absorve isso, com a pior deterioração na pose FINAL a ser
`0,0003` num alcance de `4`. ⛔ Fica assim mesmo assim: é esse mínimo exacto que compra a
propriedade pela qual este solver substituiu o FABRIK.

⚠️⚠️ **E uma fixtura calibrada no caminho do PRODUTO não discrimina o mutante** — a 1.ª redacção do
gate da parede passou com a margem da recta apagada, porque o mutante anda por outro caminho e a
junta acabava noutro sítio. ⇒ a varredura de `1 176` células **corre-se sobre os dois**, e o gate
carrega uma célula de cada (no produto o menor seno final é `0,008000`, a margem encostada; sem ela
é `0,000000`, noutra célula).

⚠️ **E `f64::signum` devolve `±1` para o ZERO** — a régua do gate lia uma junta exactamente recta
como tendo lado, e um instrumento construído sobre isso não vê a junta que ficou sem nenhum.

**Cortes por responsabilidade** (o `reach.rs` voltou a passar o tecto de 700): `reach_fabrik.rs` (a
varredura), `reach_mixed.rs` (o solver do misto), `goal_authored.rs` (a pose do documento).

### F5-d — ✅ **A linha do `IK Chain` é uma RECTA AO LADO** (2.º report do dono, 2026-09-14)

> *«a linha do IK Chain deve ser uma linha reta e não passar por dentro dos ossos»*

A 1.ª redacção (F5-a, W15) traçava a **polilinha das juntas**: numa corrente quase esticada ela caía
**exactamente** sobre os corpos dos ossos e lia-se como parte deles; numa dobrada, serpenteava. O
que ela tem de dizer é *até onde o `Chain` chega* — uma **extensão**, que é o que uma cota de
desenho técnico diz com uma recta deslocada.

⭐ **A geometria saiu para uma PORTA** (`chain_bar`), que é o que os gates medem — desenhar e medir
a mesma conta em dois sítios seria a resposta que envelhece.

⭐⭐ **O afastamento é DERIVADO, não escolhido:** o que está desenhado sobre cada osso é o **corpo**
(`bone_half_width_px`) ou a **bolinha da junta** (`joint_radius_px`), as duas já portas desta crate e
as duas função do comprimento **na tela**. Mais meia faixa, mais a folga — e a folga é `2 × LINE_PX`,
o mesmo recurso que o `BONE_HALF_MIN_PX` já nomeia por escrito: *abaixo de duas larguras de contorno
as duas bordas fundem-se numa risca só*, que é literalmente o defeito reportado.

⛔⛔ **E um afastamento CONSTANTE não chega — quem o disse foi o gate.** Uma corrente que se enrola
mais de meia volta (8 ossos a `0,9 rad` por junta) tem bojo dos **dois** lados e vem por trás da
recta: ela passava a `18,39 px` de um osso que ocupa `18,75`. ⇒ o afastamento passa por fora da
**excursão** do lado escolhido, e o lado é o **mais livre** dos dois.

⚠️ **Cinco mutações, cinco RED — e a 5.ª só morreu depois de um gate NOVO.** Com o lado FIXO a recta
continua a limpar todos os ossos (o afastamento já passa por fora daquela excursão): ela só fica
**longe**, do outro lado do arco. *O lado não é correcção, é LEITURA* — e um indicador atirado para
fora do desenho não diz até onde a corrente vai, diz que há uma risca algures.

⚠️⚠️ **E a 1.ª redacção do gate reprovou o PRODUTO por `1e-15`:** exigir exactamente o que o produto
entrega faz a asserção passar por **igualdade em `f64`**, e uma igualdade amostrada é um gate a
morrer de pé. A barra pede **metade** da folga; o produto entrega-a inteira.

### F5-e — ✅ **E a recta ENCOSTA nas duas pontas, e é FINA** (3.º report do dono, com foto, 2026-09-14)

> *«linha muito grossa e deslocada das pontas dos ossos»*

⛔⛔ **O deslocamento da F5-d está RECUSADO por veredito de produto.** Ele comprava folga contra os
ossos em toda pose e pagava-a com o que a linha existe para dizer: ela deixava de **tocar** a junta
onde o `Chain` pára (a cruz) e a ponta da corrente (o losango do alvo). *Um indicador de extensão
que não encosta nas pontas não diz qual extensão é.* ⇒ ela é a **corda** entre as duas pontas, e a
folga em pose dobrada vem de graça (uma corda passa por fora do arco); em pose esticada paga-se
sendo **fina** em vez de uma faixa.

⭐ **A largura deixou de ser um número próprio** — é a do losango (`GOAL_LINE_PX`), por **alias**: um
`const` só dela seria a segunda resposta à mesma pergunta. E a **meia opacidade morreu com a faixa**
(ela existia porque `7 px` por cima da arte escondiam o que o artista posa; a meio tom uma linha fina
desaparece).

⭐ **E a cruz da raiz é INSCRITA na bolinha da junta que marca** (`raio / √2`), em vez de herdar a
largura da faixa — aquele raio já encolhe com o comprimento do osso, e um número próprio
desalinhava a marca da alça.

⚠️⚠️ **DOIS gates dissolveram-se com a premissa** (`never_touches_a_bone`, `takes_the_free_side`) e
foram substituídos pelos da lei nova: `ends_on_the_chain_ends` (igualdade **exacta** — qualquer
deslocamento é o defeito reportado) e `is_a_chord_not_the_joint_polyline` (a metade que impede a
volta ao 1.º defeito). E um terceiro foi **apagado por ter virado tautologia**: com os extremos
iguais às pontas, o paralelismo é por construção.

⭐⭐ **A leitura das três voltas é uma só:** *as duas queixas dele eram sobre a mesma linha e puxavam
em sentidos opostos* — «não passes por dentro dos ossos» e «não te afastes das pontas». A resposta
não estava em nenhum dos extremos que eu construí (polilinha · barra deslocada), estava na **corda
fina**, que é o que as duas frases dele, juntas, descrevem.

### F6-j — ⭐⭐⭐ **A RÉGUA DA DOBRA existe, e ela REFUTOU a cura que eu ia construir** (2026-09-14)

O dono mandou seguir o artefacto que sobra depois do `Smooth`: **a arte dobra sobre si mesma** em
dobras fortes. ⛔⛔ **O instrumento NÃO EXISTIA** — os números que esta fila citava (`−0,129` a `60°`,
`−1,017` a `150°`) eram de uma medição avulsa de outra janela. *Uma lei sem instrumento é uma nota
que envelhece*, e sem ele nenhuma cura pode ser comparada com a doença.

**A grandeza é o determinante do jacobiano** ([`ph2d_skeleton::fold`](../../crates/ph2d-skeleton/src/fold.rs)):
onde ele passa por zero, a vizinhança é desenhada **do avesso**. ⚠️ **Nenhuma régua deste módulo a
via** — o desvio em píxeis da silhueta (a régua do `Smooth`) mede a **aproximação** do campo, e o
campo dobrado é aproximado com fidelidade: *quanto melhor o `Smooth`, mais nítida a dobra.*

#### ⛔⛔ A 1.ª fixtura leu o produto como SÃO

Com arte **fina** (meia-altura `0,15 ×` o arco) a corrente de dois ossos a `150°` não inverte **um
único ponto**. A dobra vive **longe do eixo**, onde o gradiente dos pesos é grande — uma caixa
estreita não tem lá pontos. ⇒ **a variável não é a dobra, é o RAIO contra a ESPESSURA DA ARTE.**

#### A tabela, e ela vai ao contrário da intuição

Corrente de 2 ossos, arco `4`, arte de meia-altura `2,4` — `% da arte invertida`:

| alcance | `60°` | `90°` | `120°` | `150°` |
|---|---:|---:|---:|---:|
| `0,83 ×` a meia-altura da arte (**o que o produto entrega hoje**) | `0,17 %` | `0,49 %` | `1,22 %` | `2,61 %` |
| `1,25 ×` | `0` | `0` | `0,51 %` | `2,88 %` |
| `1,67 ×` | `0` | `0` | `0` | `0,60 %` |
| **`2,08 ×`** | **`0`** | **`0`** | **`0`** | **`0`** |

⚠️ **E APERTAR os pesos PIORA** (`0,25 ×` do osso ⇒ `0,64 %` de inversão): o que dobra a arte é o
**gradiente** dos pesos, e apertá-los torna-o mais íngreme. *A resposta intuitiva — «influência mais
local» — é a errada.*

#### ⛔⛔ E o B-BONE não cura sozinho — mede-se PIOR

Subdividir o osso (o mecanismo que o dono acabou de pedir na F8) com a população de amostras
constante: `2,61 % → 4,94 %` a `24` sub-ossos. **Com o alcance já certo** ele não cura nada e compra
**margem**: `det_min` `0,013 → 0,367` (`28 ×` mais folga antes de inverter), com zero inversão em
toda a escada. ⇒ **a ordem é o ALCANCE primeiro, o B-Bone depois.**

#### ⛔⛔⛔ E a cura que parecia a melhor era a RÉGUA A NÃO MEDIR NADA

Subdividir encolhendo o raio com o osso lê **`0,00 %` invertido** a `24` sub-ossos — com **`94,5 %`
das amostras fora da conta por serem ÓRFÃS. O ponto órfão é um salto **descontínuo** (fora do raio de
todo osso a pele salta para o mais próximo), e uma diferença finita que o atravesse devolve um
determinante enorme e falso: medi **`−57`** num mapa de escala `1`. ⇒ excluí-las é obrigatório, e a
**fracção excluída sai no relatório** — sem ela, uma «cura» que deixasse a arte inteira órfã
leria-se **perfeita**.

⚠️ **Quatro gates, três mutações RED** (incluir as órfãs · o determinante sem sinal · o passo da
diferença em unidades do documento). ⭐ **A terceira sobreviveu aos três primeiros gates** e só morreu
com o gate da **ESCALA** — a invariância estava afirmada no doc e não tinha quem a medisse.

⛔⛔ **MORTO — a pergunta deixou de existir (auditoria de 2026-09-16).** Esta entrada pedia uma
DECISÃO DO DONO entre três rotas para *«o alcance de um osso não sabe nada da arte»* (derivá-lo da
arte no bind · por quadro no `recook` · guardá-lo por tendão, com schema).

⭐⭐⭐ **O padrão-ouro apagou a pergunta inteira.** Desde que os pesos são *Bounded Biharmonic* o
alcance do osso é **inerte**: a energia é resolvida sobre o domínio da própria arte, logo ela já
"sabe" o que a arte é — e a bancada mediu as duas linhas do padrão-ouro (com `strength` diferentes)
a saírem **idênticas coluna a coluna**. O hand-tuning do alcance foi **apagado** do produto na mesma
jornada.

⇒ ⛔ **Não reconstrua nenhuma das três rotas.** Quem as ler aqui estaria a pagar de novo um problema
que a troca de lei dissolveu — que é a forma nº 1 pela qual esta lista custa dinheiro.

### F6-t — ⭐⭐⭐ **O `Smooth` COM A CENA CHEIA: pagava inerte, e custava o dobro do que o orçamento prometia** (2026-09-16)

**UMA LINHA:** o item aberto *«o custo da adaptativa não foi medido sob cena cheia»* foi medido
(sonda nova `measure_the_smooth_under_a_full_scene`: `n` imagens do tamanho do smoke, malha de bind do
produto, `25°`/`60°`, zoom `1`/`4`/`8`), e devolveu **dois defeitos** e uma nota que mentia.

1. ⛔⛔ **Com as malhas guardadas acima do orçamento, o `Smooth` pagava a lei inteira para não
   partir nada.** O orçamento é do QUADRO e repartido na proporção das peças guardadas; com
   `2` imagens do smoke (`4 860` > `4 721`) cada uma recebe o que já guarda, a saída é a do `Fast`
   ao bit — e o quadro custava `28×` o `Fast`: **`8` imagens, `5,9 ms` (`35 %` de um quadro)**
   contra `0,21 ms`. ⭐ Cura: sem espaço no orçamento a imagem segue o caminho do `Fast`
   (`attach_skin_meshes`); `8` imagens passam a `0,47 ms` = o `Fast`.
2. ⛔⛔ **O orçamento prometia `10 %` de um quadro e o refinamento real custava `19 %`.** A sonda
   que dava o `0,340 µs` por peça **não refinava** (a arte dela media `200 × 100` px de ecrã —
   nota aberta desde a manhã). Com o refinamento a trabalhar, o custo tem DUAS partes: avaliar uma
   peça guardada `0,36 µs` e cada peça NOVA `~1,0 µs` ⇒ `60°` a zoom `8×` custava `3,17 ms`.
   ⭐⭐ **A causa era o livro das arestas da lei adaptativa** — um `BTreeMap` percorrido `~9` vezes
   por triângulo e **nunca iterado**. Um índice pela ponta menor (`ph2d_poly2d::refine_adaptive`)
   dá as mesmas respostas: **impressão digital de `48` casos do produto igual ao bit** (`22` saídas
   distintas), e a avaliação cai para `0,156 µs`, a peça nova para `0,311`–`0,324`.
3. ⇒ **o orçamento foi remedido:** `324 ns` por peça nova (o maior dos dois custos, logo um tecto
   para qualquer mistura) ⇒ **`5 144` peças** (era `4 721`). O orçamento cheio custa agora
   `1,09`–`1,13 ms` = **`6,6 %`** de um quadro (perfil `smoke`, `load 2,7`–`3,2`, mínimo de 30, três
   corridas). E **duas** imagens do smoke passam a caber (`4 860`).

| cena | antes | depois |
|---|---:|---:|
| 1 imagem, `25°`, zoom `1` (avalia, não parte) | `0,90 ms` | `0,39 ms` |
| 1 imagem, `60°`, zoom `8` (orçamento cheio) | `3,17 ms` | `1,10 ms` |
| 8 imagens (acima do orçamento) | `5,9 ms` | `0,47 ms` (= `Fast`) |

- **A sonda antiga** (`measure_the_cpu_cost_of_a_skinned_frame`) passou a medir a zoom `8×` e marca
  `NAO PARTIU` nas linhas em que a lei não trabalhou — *uma sonda cujo sujeito deixou de fazer a
  coisa medida mede outra coisa com o mesmo nome.*
- **Gate:** `without_room_in_the_budget_the_smooth_pays_nothing_and_draws_the_fast_mesh` (contador
  de refinamentos por thread — uma contagem, nunca um relógio — + a saída do `Fast` ao bit + o
  controlo com espaço, que tem de partir). **Mutações (3, todas RED):** o curto-circuito apagado · o
  curto-circuito com `>=` · o índice das arestas a procurar pela ponta errada.
- ⏳ **ABERTO e nomeado:** uma cena com a arte presa **muito acima** do orçamento (um personagem de
  muitas peças) fica com o `Smooth` igual ao `Fast`, com um aviso único no terminal — agora de
  graça, mas sem alisar. O caminho que o alisaria em qualquer cena é deformar na GPU (a malha
  densa assada no bind, deformada no *vertex shader*), e é obra de plano, não de afinação.

---

### F6-s — ⭐⭐⭐⭐ **REGRA DO DONO: editar PIXELS acontece na imagem PLANA** (ordem de 2026-09-16)

> *«nenhuma ferramente de edição de imagem deve trabalhar com arte dobrada. Aqui o mesmo que fizemos
> para painter: ao usar Background removal, a imagem fica sem deformação até o fim da operação. Ao
> finalizar a operação ela se dobra para obedecer aos ossos. Isso serve para todas as tools de
> pintura e remoção e deformação de pixels com exceção do Liquify que deverá ser capaz de fazer
> ajustes na imagem dobrada. Exceção também para filtros (como contraste, blur, etc) e shaders como
> shadows e outros que não pintam ou apagam a imagem. Escreva isso para não esquecer.»*

Ela estende a ordem de 2026-09-15 (só o Painter achatava) a **toda ferramenta da mesma espécie**, e
mora na mesma porta: [`ph2d_app_painter::skin_suspend`](../../crates/ph2d-app-painter/src/skin_suspend.rs).
⛔ **Uma ferramenta nova desta espécie entra na TABELA dessa porta** — nunca numa cerca própria, nunca
num «o pincel X segue a dobra».

| ferramenta | achata? | porquê |
|---|---|---|
| **Painter** — Paint · Erase · Smear · Blur · Clone · Mask · Inpaint · Fill · Selection · Sculpt · Knife · Wet Paint | ✅ **sim** | pinta, apaga ou mexe em pixels (a ordem de 15/09) |
| **Painter — Transform** (o gizmo que deforma pixels) | ✅ **sim** | deforma pixels — ⚠️ e partilha o `PaintMode::Deform` com o Liquify, por isso a chave da exceção é o id do modo e não o `PaintMode` |
| **Painter — Liquify** | ⛔ **não** | a exceção nomeada: *«deverá ser capaz de fazer ajustes na imagem dobrada»* |
| **Background Removal** | ✅ **sim** | remove pixels (a ferramenta nomeada) |
| filtros (contraste, blur, …) e shaders/efeitos (sombras, …) | ⛔ não | a exceção nomeada: não pintam nem apagam |
| **Color Equalization** e toda ferramenta que só trata CORES | ⛔ **não** *(decisão do dono)* | *«pode ser aplicada dobrada»* |
| **Padding** · **Upscale** · **Equalize Sizes** | ✅ **sim, a SELECÇÃO inteira** *(decisão do dono)* | mudam o TAMANHO ou a MARGEM — e o **Apply SOLTA a imagem dos ossos** |
| **Trim Transparency** · **Make Square** · **Rasterize** · **Real Size** (um clique) | — (não têm «enquanto») | mudam o tamanho ou a margem — e **SOLTAM a imagem dos ossos** |

⭐ **As leituras da linha foram RESPONDIDAS pelo dono no mesmo dia** (as três linhas de baixo da
tabela): *«Color Equalization e qualquer outra do tipo que trata apenas cores, não endireita a
imagem, pode ser aplicada dobrada. As que mudam tamanho ou padding devem endireitar e se aplicadas
quebrar o binding com os ossos.»*

✅ ~~**ABERTO e nomeado, achado ao escrever a regra:** o Padding (e os botões de imagem que cortam
ou acrescentam margem) mudam ONDE o conteúdo está na textura, e a malha do bind guarda a arte em px
da textura de ANTES — numa imagem presa, depois do Apply, a pele leria os texels errados.~~
**DISSOLVIDO pela decisão do dono:** o Apply dessas ferramentas solta a imagem dos ossos (ver
*«a moldura»* abaixo).

#### ✅ IMPLEMENTADA no mesmo dia (2026-09-16)

- **A porta:** `sprite_achatada` lê a TABELA (`FERRAMENTAS_QUE_ACHATAM = ["painter", "bgremoval"]`)
  e a EXCEÇÃO (`MODOS_SOBRE_A_DOBRA = ["liquify"]`, chave `PainterTool::active_paint_mode_id`).
  ⚠️ Ela passou a pedir `&mut ToolRegistry`: o contrato `Tool` (congelado, §6) só chega ao Painter
  concreto pelo `as_any_mut`. O aviso mudou para *«Editing pixels flattens this image…»*.
- ⭐⭐ **O LIQUIFY sobre a dobra tinha UM consumidor errado: o ANEL.** O kernel (`warp_dab_at`) já
  trabalhava certo por construção (px de textura, no texel que o ponteiro resolve pela malha, e o
  arrasto é a diferença de dois pontos resolvidos por ela); o anel era um disco com a escala do quad
  de repouso. Na fixtura (faixa `4 × 2` m dobrada em arco de `4` m, fora da origem e rodada) o anel
  de antes errava o raio do kernel em **`13,2 %` / `1,2 %` / `18,0 %`** (aresta que estica · meio ·
  aresta que encolhe); o de agora em **`≤ 2,6e-6`**. A lei é a do anel da F6-q, pela metade de
  LEITURA do mapa (`DrawnMesh::uv_at_world`, a mesma álgebra do `mesh_uv` sem `&mut World`) —
  `ph2d_sprite_screen::anel_na_malha` ← `painter_bridge_brush_ring::anel_do_liquify`.
- ⏸️ **As notas DORMENTE** do `CanvasMap` e dos 9 desenhadores dizem agora *«só no Liquify»*; e as
  duas portas da Remoção de fundo (`uv_sob_o_ponteiro`, `anel_do_pincel`) ganharam a delas — o ramo
  da malha ali não tem sujeito enquanto ela achatar.
- **Gates (8):** `every_pixel_mode_flattens_and_liquify_works_on_the_bend` · `the_table_is_the_rule`
  (a porta) · `the_read_only_ring_is_the_same_ring` (a metade de leitura, ponto a ponto) ·
  `the_liquify_ring_lies_where_the_kernel_deforms_on_bent_art` (cada ponto do anel devolvido pela
  porta do ponteiro do Painter, com o anel de antes como controlo) ·
  `only_the_deform_ring_asks_the_mesh_and_only_when_there_is_one` ·
  `the_liquify_ring_is_drawn_through_the_art_mesh` (costura) ·
  `the_background_remover_notes_its_mesh_branch_is_dormant_while_it_flattens` (a nota nasce e morre
  com a entrada na tabela) · e a fixtura dos 4 gates do anel da F6-q passou a estar **fora da origem
  e rodada** — ⚠️ com a pose identidade a mutação *«o `uv_at_world` ignora a base e a posição»*
  **sobrevivia**.
- **Mutações (8, todas RED):** o Transform sem a chave do modo · o Liquify a achatar · a Remoção de
  fundo fora da tabela (as três na porta) · o `uv_at_world` sem a pose · o anel com o raio do pincel
  de PINTURA · o anel sem a guarda do Deform · o desenho sem a chamada · a tabela sem `"bgremoval"`
  com as notas no sítio.


#### ✅ A MOLDURA — a resposta do dono, implementada no mesmo dia (2026-09-16)

- **Duas tabelas na mesma porta** (`skin_suspend`): `FERRAMENTAS_QUE_ACHATAM` ganhou o **alcance**
  (`Principal` para o Painter e a Remoção de fundo; `Seleccao` para o Padding, o Upscale e o Equalize
  Sizes, que aplicam a TODAS as selecionadas) e `FERRAMENTAS_QUE_MUDAM_A_MOLDURA` lista as sete que
  mudam o tamanho ou a margem. ⚠️ **A suspensão passou a ser um CONJUNTO** (`attach_skin_meshes(…,
  suspensas: &[u64])`): com o alcance de antes, uma selecção de três imagens presas deixava duas
  dobradas debaixo do Padding.
- **O Apply solta:** todo Apply de ferramenta de imagem grava pela porta `commit_edit` da shell, e a
  transacção NASCE com o id da ferramenta (`Edicao::new("padding")`, onde estava `Vec::new()`) — a
  tabela decide, e soltar é tirar a pele (`ph2d_skeleton_live::skin_image::release_image`). O *Real
  Size* não tem transacção e solta pela metade sem ela. ⚠️ A gravação antiga ficou **privada**: um
  Apply novo não consegue gravar sem passar pela decisão.
- ⭐ **O Ctrl+Z devolve as duas coisas:** a pele é um componente registado, e soltar no MESMO quadro
  do Apply põe a imagem nova e a ligação perdida no mesmo passo de desfazer. Os avisos dizem-no: ao
  abrir uma ferramenta de moldura (*«… Apply unbinds it from the bones»*) e ao aplicar (*«N image(s)
  unbound … Ctrl+Z brings the binding back»*).
- ⚠️ **A shell ENCOLHE `7` linhas** apesar da porta nova: os dois predicados do grupo Image Tools
  (`is_image_edit_tool`, `palette_visible_tool_indices`), que só usam tipos do núcleo, mudaram-se
  para `ph2d_editor_core::tool` (igual ao original tirando formatação e caminhos; a shell
  reexporta-os com o mesmo nome). ⛔ A 1.ª forma (o id como 6.º argumento da gravação) deixava a
  shell em **`+65`** (medido) — o `fn_call_width` do `rustfmt` é `60`, e cada chamada com um id
  longo partia-se em oito linhas.
- **Gates (7 novos/endurecidos):** `every_suspended_image_is_flat_and_the_rest_stays_bent` ·
  `releasing_an_image_takes_its_skin_and_nothing_else` · `the_frame_tools_flatten_the_whole_selection_and_the_colour_tools_nothing`
  · `the_two_tables_agree` · `a_frame_apply_releases_the_bound_images_it_changed_and_a_colour_apply_does_not`
  · `every_image_apply_commits_through_the_door_with_its_own_tool_id` (o id de cada transacção contra
  o DRENO do mesmo bloco, e cada id das tabelas contra os MANIFESTOS reais) · e o
  `painting_flattens_the_art_and_the_frame_passes_it_through` passou a exigir a SELECÇÃO inteira.
- **Mutações (14, todas RED):** suspender só o 1.º · soltar sem a guarda da imagem · soltar sem
  remover · o alcance ignorado · o aviso de moldura para todas · o Color Equalization na tabela · o
  Apply sem a guarda da tabela · contar as que não tinham pele · o aviso calado · o Padding fora do
  achatamento · o id do vizinho numa transacção · o Real Size sem soltar · um id inventado na tabela
  · o quadro a passar só a principal.
- ✅ **O Separate Islands** da Remoção de fundo cria sprites novas a partir das ilhas de uma imagem
  presa, e elas nascem **sem** ligação aos ossos — **decisão do dono** (2026-09-16): *«nasce sem
  ossos mesmo»*. A original não muda de moldura e fica fora da tabela.
- ✅ ~~O desfazer de UM nível das ferramentas de imagem repõe a textura e não a pele~~ — **não é
  alcançável depois de um Apply**, e a nota anterior não o tinha conferido: o Ctrl+Z só vai a esse
  desfazer quando a fila GERAL está vazia (`undo_route::undo_owner`, `global_has`), e o próprio Apply
  (com a soltura, que é um componente registado) põe um passo nela. Quando o desfazer das
  ferramentas responde, o passo do Apply já foi desfeito — com a pele devolvida.
- ✅ **Smoke do dono aprovado** (2026-09-16).
---

### F6-r — ✅ **As alças do gizmo de uma imagem presa cercam a arte DOBRADA** (2026-09-16)

**UMA LINHA:** a caixa do gizmo de uma sprite era sempre o **quad de repouso** (ou a folha aberta),
enquanto o realce e o *View All* já liam a malha — *duas caixas neste repo, que a F6-i nomeou e não
unificou*. Medido na cena do smoke (`sonda_a_caixa_do_gizmo_contra_a_malha`): a `25°` por junta a arte
dobrada sai **`221` px acima** e **`69` px à direita** da caixa; a `60°`, `324` px acima. ⭐ A caixa
passa a ser a do que se DESENHA quando a sprite é malha
([`sheet_lattice::gizmo_box`](../../crates/ph2d-sprite-screen/src/sheet_lattice.rs), parâmetro
`desenhada`), e ganha à folha aberta (uma sprite em malha não desenha a folha).

⭐ **O código saiu da shell primeiro, num commit de movimento PROVADO** (`scripts/moved-proof.py`,
quatro blocos `OK`; `8` gates antes, `7 + 1` depois com os mesmos nomes — fica na shell o que cruza
com o `sim_extract_sheet::cell`). Gates: 1 na folha + 1 de costura; **duas mutações, duas RED**.

---

### F6-q — ✅ **O anel do pincel da Remoção de fundo mostra onde o pincel PINTA, na arte dobrada** (2026-09-16)

**UMA LINHA:** o anel era um círculo com a escala do **quad de repouso**; o pincel pinta pela
**malha** (a F6-m pôs-lhe a porta do ponteiro). Medido na cena do smoke, cada ponto do anel devolvido
à imagem por essa mesma porta: erro do raio **`14,4 %`** na mediana, **`33,2 %`** no p90 e
**`46,0 %`** no pior (junto de uma junta) — e **`0,0 %`** sem dobra, que é o controlo. ⭐ A lei nova
é o disco do pincel **na imagem**, levado ao ecrã pela malha desenhada
([`ph2d_sprite_screen::anel_do_pincel`](../../crates/ph2d-sprite-screen/src/anel_do_pincel.rs)):
erro `≤ 2,5e-5`. ⚠️ Fora da arte o anel parte-se em ARCOS (o texel existe e não é desenhado), e
com o ponteiro fora dela não há anel (a porta do ponteiro recusa, e o pincel não pinta).

⚠️ **A nota que o pôs na fila dizia `8,8 %`** (doze pontos, lei de pesos de antes do padrão-ouro).
⭐ **O código saiu da shell**: o desenho do anel encolheu (`−13` linhas) e a forma vive na folha.
Gates: 3 na folha (um com o anel de ANTES como controlo, `23,6 %` na fixtura) + 1 de costura no
censo do ponteiro; **duas mutações, duas RED**. Sonda na cena real:
`sonda_o_anel_da_remocao_de_fundo`.

⏸️ **No MESMO dia a regra F6-s pôs a Remoção de fundo a ACHATAR**, e o ramo da malha deste anel
ficou sem sujeito para ela (o `8,8 %`/`46 %` descreve um caminho que o produto de hoje não corre
nesta ferramenta). ⭐ **A lei não morreu:** é ela que desenha o anel do **Liquify**, a exceção da
regra (F6-s, *implementada*). A fixtura destes gates passou a estar fora da origem e rodada, e o
controlo lá lê `16,0 %` / `1,3 %` / `23,6 %`.

---

### F6-p — ⭐⭐⭐ **O `Smooth` deixou de desenhar os VINCOS dos pesos** (smoke do dono, foto, 2026-09-16)

**UMA LINHA:** *«micro irregularidades»* era o refinamento a seguir **fielmente** um campo com um
vinco em cada aresta do bind — os pesos eram lidos em linha recta. A cura é a lei dos atributos
([`AttrLaw::Hermite`](../../crates/ph2d-poly2d/src/attr_law.rs), porta de produto
[`skin_refine`](../../crates/ph2d-skeleton-live/src/skin_refine.rs)); mecanismo, tabela e gates no
[Bug #33](../Vector%20Module/BUGS_vector.md).

| lado de cima, zoom `8×` | nós | vai-e-volta | trocas de sinal |
|---|---:|---:|---:|
| `Fast` | `46` | `26,60°` | `2` |
| `Smooth` antes | `66` | `47,00°` | `24` |
| `Smooth` antes, orçamento `×16` | `74` | `57,95°` | `36` |
| **`Smooth` agora** | `57` | **`26,62°`** | **`2`** |
| **`Smooth` agora, orçamento `×16`** | `73` | **`26,71°`** | **`2`** |

⚠️ **Duas portas de bissecção, independentes:** `PH2D_SKIN_REFINE=uniforme` (a lei de refinamento
de antes) e `PH2D_SKIN_WEIGHTS=linear` (a lei dos pesos de antes).

⏳ **ABERTO e nomeado:**
- **o campo de Hermite amostrado DENSO ainda vai e volta `39,67°`** no lado de cima (`54` trocas, o
  maior canto a `0,23°`, `0,137 px` do P1 a zoom `1`): são ondulações abaixo da tolerância, que o
  refinamento não resolve de propósito — com orçamento de sobra ele converge a `26,71°`. Um campo
  C¹ **de verdade** pede a recuperação de gradientes de ordem mais alta (ou um *patch* por
  triângulo); não foi pedido.
- ✅ **DECIDIDO (ver o veredito no fim deste item): tirar o `Smooth`?** Depois da cura ele comparou os dois na cena
  do smoke e viu *«ambos iguais»* — e está certo **para aquela cena**: a `25°` por junta o `Smooth`
  nem refina a zoom `1` (`2 430` peças nos dois). Medido nas dobras FORTES (sonda
  `sonda_a_faceta_do_fast_na_dobra_forte`): o `Fast` erra `0,96`/`1,35`/`1,66`/`1,85 px` a
  `60°`/`90°`/`120°`/`150°` (zoom `1`; `×4` a zoom `4×`) e o `Smooth` `0,50`; o maior canto da
  silhueta a `150°` vai de `17°` para `10°`. ⭐ **E a outra razão de o `Smooth` ser o de fábrica
  caiu:** com a malha de bind de hoje o `Fast` já liga a cura do pincel (pior redondeza `1,05`
  contra `1,42` sem ela; `sonda_o_pincel_precisa_do_smooth`). ⛔ **Nada foi apagado** até ele ver
  uma dobra forte.
  ✅ **VEREDITO (dono, 2026-09-16, depois de dobrar forte):** *«não se percebe diferença, mas por
  enquanto deixe os dois»*. ⇒ **ficam os dois, o `Smooth` continua de fábrica.** ⚠️ Para quem
  reabrir: a diferença MEDIDA existe (a tabela acima) e o olho do dono não a separou — *uma
  diferença que a régua vê e o dono não vê é o número que decide se o custo compensa*, e ele está
  aqui: `+19 %` (`60°`, `2 898` peças) a `+91 %` (`150°`, `4 652`) sobre as `2 430` do `Fast`, a
  zoom `1`.
- **a sonda de custo desta crate deixou de refinar** (a arte dela mede `200 × 100` px de ecrã) e
  não reproduz a tabela do [`skin_budget`](../../crates/ph2d-skeleton-live/src/skin_budget.rs) hoje
  — o preço da lei nova foi medido na cena do smoke, as duas leis intercaladas.

---

### F6-o — ⭐⭐⭐ **O OSSO QUE DOBRA FECHOU: o esticão deixou de variar, e as alças pegam-se no canvas** (2026-09-16)

**UMA LINHA:** os três abertos da **F8** fecharam, e dois deles corrigiram uma recusa escrita.

**(1) O esticão VARIAVA ao longo do osso** (`12,63 %` com as alças a `0,2 L`, `82,01 %` a `0,6 L`,
`1 051,95 %` com as alças cruzadas no eixo) porque os nós saíam do PARÂMETRO. ⛔⛔ **E a cura
publicada — equalizar o ARCO — NÃO chegava, o que só a varredura da densidade disse:** ela deixa um
piso que **não desce com a tabela** (`1,22 %` a `0,6 L`, igual de `16` a `32` amostras). *Um número
que não se move quando se afina a discretização não é erro de discretização.* A causa é geometria:
o esticão de um sub-osso é `corda / (L/n)`, e a corda de um pedaço mais curvo é mais curta que o
arco dele ⇒ **arcos iguais dão cordas desiguais**. A lei que fica parte da equalização por arco e
corrige-a para a **CORDA**, em rondas de Gauss-Seidel: `0,000 %` nos três casos.

⚠️⚠️ **E a objecção registada na recusa era VERDADEIRA e não mordia:** *«um somatório de cordas não
devolve `L` ao bit»* — ele **nunca corre** no ponto neutro, porque a primeira linha da lei é um `if`
sobre `Bend::is_straight`. *Uma recusa que nomeia um custo tem de dizer em que CAMINHO ele é pago.*

**(2) As alças ganharam GESTO DE CANVAS** — duas alças de Bézier, com as hastes até à raiz e à
ponta, porque elas **são** os pontos de controlo da cúbica. ⛔⛔ **E a armadilha quase as matou à
nascença:** no ponto NEUTRO a alça está **em cima do eixo do osso**, e o corpo é um alvo com outro
verbo — a competição por proximidade de sempre tornava-a **inalcançável no único estado em que todo
osso nasce**. *Uma alça que só se agarra depois de já ter sido movida não se agarra nunca.* ⇒ ela é
a única que ignora o corpo, e paga um raio apertado cujo recurso tem nome: **o comprimento do osso
que sobra para o verbo de girar** (o meio continua a girar a partir de `48 px` de osso na tela).

**(3) As TANGENTES DOS VIZINHOS existem** — `Curve Handles: Manual | From Chain`, o *Handle Type:
Auto* do Blender, que esta fila tinha como *«lei da HIERARQUIA e não existe»*. ⭐⭐ **E o ponto neutro
é EXACTO**, por uma decisão de implementação: as tangentes saem da transformação **RELATIVA** (a pose
de um filho em relação ao pai **é** o `Transform` dele), nunca de uma volta pelo mundo — *uma volta
pelo mundo teria deixado `y ≈ 1e-17`, e ligar o modo num rig recto arquearia tudo um bocadinho.*

⛔ **Em `From Chain` não há alça para agarrar e os quatro números do painel somem:** elas são
derivadas, e arrastar uma seria escrever num valor que o quadro seguinte recalcula. E o `curve`
autorado fica **intocado** — *um modo que sobrescreve o valor autorado é um modo que não se desliga.*

⚠️ `PROJECT_SCHEMA` **+1** — conte o DELTA. ⚠️ E **duas afirmações de exactidão minhas** foram
corrigidas pela medição (o par alça⇄curvatura fecha ao bit **no neutro** e a `~4e-17` fora dele; a
ida-e-volta do GESTO perde mais porque atravessa o inverso da pose do osso): *uma afirmação de
exactidão tem de nomear o caminho em que ela vale*.

⏳ **ABERTO e nomeado:** um osso com **dois filhos-osso** não tem «o seguinte» — ali a corrente
ramifica e aquele lado fica recto, que é a leitura honesta de *«não há tangente única»*. A rota
publicada (o *custom handle* do Blender, que nomeia OUTRO osso como alça) move schema outra vez e
não foi pedida.

---

### F6-n — ⛔⛔⛔⛔ **O botão `Smooth` era um CONTROLO MORTO, e o orçamento vinha de um custo que o produto nunca pagava** (auditoria de 2026-09-16)

**UMA LINHA:** o par `Fast`/`Smooth` do painel entregava a **MESMA malha, ao bit** — e não por um
fio cortado, mas por **aritmética**: a lei de refinamento era o `k` GLOBAL, cujo tecto é
`⌊√(orçamento/peças)⌋`, logo toda malha acima de `orçamento/4` peças só admite `k = 1`. Medido na
cena do produto: `2 268` peças com orçamento `1 543` ⇒ inerte em TODO zoom.

⛔⛔ **É a terceira espécie de controlo morto do `CLAUDE.md` §5.0, e nenhuma sonda deste repo a
vê:** o id existe, é pintado, é registado, o clique chega à ferramenta, o valor chega ao consumidor
— *e o consumidor devolve a entrada*. O censo de registo mede focalizabilidade; os `seam_*` provam
que a escrita chega ao consumidor. **Nenhum pergunta se a SAÍDA muda.**

⇒ a lei nova parte uma **ARESTA** de cada vez, e os dois donos dela ao mesmo tempo — logo a malha é
conforme depois de **cada passo**, sem nó pendurado. ⛔⛔ **Isto refuta por construção a frase que o
cabeçalho do `refine.rs` carregava** (*«um `k` por triângulo abriria nós pendurados»*): ela estava
certa sobre **uma** construção — a grelha baricêntrica por triângulo — e foi lida como se fosse
sobre a pergunta inteira. *Antes de dizer que uma pergunta é inexprimível, procure a outra operação
elementar.*

⚠️⚠️ **E o GANHO é função da ARTE, não uma constante — a primeira medição mediu a pergunta errada.**
Sobre um campo que curva por igual em todo o lado a lei uniforme está quase óptima (`1,9×`); o ganho
grande é o caso do PRODUTO, a curvatura na **articulação** (`11,3×` a `0,50 px`, e ali a uniforme
**nunca chega**: satura o orçamento em `k = 8` e fica em `1,121 px`). *A vantagem dela não é «partir
menos»; é partir **onde**.*

⚠️⚠️ **E na cena do dono enquadrada inteira o `Fast` já entrega `0,34 px`** — abaixo da promessa de
meio pixel, e não refinar é a resposta CERTA. A alavanca é o **ZOOM**:

| zoom | `Fast` | `Smooth` uniforme | `Smooth` adaptativo |
|---:|---:|---:|---:|
| `1×` | `0,34 px` | `0,34` (`2 268`) | `0,34` (`2 268`) — nem precisa |
| `2×` | `0,68 px` | `0,68` (`2 268`) | **`0,50`** (`2 364`) |
| `4×` | `1,37 px` | `1,37` (`2 268`) | **`0,50`** (`3 416`) |
| `8×` | `2,74 px` | `2,74` (`2 268`) | **`0,54`** (`4 720`) |

⛔⛔ **E O ORÇAMENTO DO QUADRO ESTAVA DERIVADO DE UM CUSTO QUE O PRODUTO NUNCA PAGOU:** o
`1 080 ns/peça` foi medido em 13/09 sobre um `Smooth` que **refinava**, e desde então o que ele de
facto pagava era o `Fast` mais o custo de decidir. Remedido com a lei que shipa: `0,340 µs` de CPU +
`0,013` marginal de GPU ⇒ **`353 ns`**, e o orçamento passa de `1 543` para **`4 721`** peças —
com o que a malha de bind de `2 268` deixa de disparar o aviso em toda execução. *Um custo medido
sobre um caminho que não corre é um orçamento que mente nos dois sentidos, e este mentia para baixo.*

⚠️ **A lei nova é `3,9×` mais cara POR PEÇA** (`0,340` contra `0,087`) — é o preço de ela decidir; a
antiga era barata porque não fazia nada. O livro de contas já foi cortado uma vez: dois mapas e um
`Vec` por aresta liam `0,52 µs`, e com **um** mapa e os donos num par fixo desceu a `0,34` (`−35 %`).

⚠️⚠️ **E a sonda de custo media ZERO:** a tabela de pesos dela era **uniforme**, e com pesos iguais
em todo o lado a mistura das poses é a mesma em todo o ponto ⇒ o campo é um **AFIM**, que um
triângulo reproduz exactamente ⇒ desvio zero e nenhuma das leis refina. *Uma fixtura no ponto neutro
de um knob não testa esse knob*, e a coluna do `Smooth` mediu *«o custo de decidir não fazer nada»*
durante três dias.

⏳ **ABERTO e nomeado:** a `ph2d-poly2d` guarda as **duas** leis (`PH2D_SKIN_REFINE=uniforme` volta à
antiga, para bissecar), e o custo da adaptativa **não foi medido sob cena cheia** — o número de cima
é de uma imagem presa.

---

### F6-m — ⭐⭐⭐ **O resto do app ainda achava que a arte é PLANA** (censo + as três ferramentas, 2026-09-15)

**UMA LINHA:** a wave anterior curou **o pincel**; o censo do dia seguinte achou o **conta-gotas**, o
editor de **curva/linha** e as **três** entradas da Remoção de fundo a resolver o ponteiro pelo afim
do quad de repouso — e as três entradas do removedor faziam algo **pior que o afim**: uma caixa
alinhada aos eixos, tirada da pose LOCAL, cega à rotação, ao pai e à malha.

⛔⛔ **A lei que esta wave pagou:** *um controlo DESENHADO por um mapa e AGARRADO por outro é um
controlo morto sob o dedo* — e curar só a metade do ponteiro **fabricava** três deles, porque antes
as duas metades concordavam **por acidente**. ⇒ cada cura tem um PAR: a porta inversa (quem aponta) e
a directa (quem desenha), que passou a existir (`ph2d_render::drawn_mesh_of` / `drawn_instance_of`).

⛔ **E a TINTA da máscara saiu do Vello para o passe de sprites**, porque o caminho por um recorte
por triângulo está **medido e refutado**: numa arte translúcida ele deixa `10 580` px de costura
(`41 732` a `3 456` peças) e a cura barata — dilatar os recortes — **piora** exactamente o caso
translúcido (`55 978`, pior desvio `124`); acima disso os buffers do Vello são FIXOS e degradam em
**silêncio**, que foi o *«Smooth bugado»* deste módulo.

⚠️ **E uma afirmação minha ao dono estava meia errada:** a *prévia* da Remoção de fundo já seguia a
dobra (ela viaja no `PreviewOverride`, que só troca a textura da MESMA instância); o que estava plano
era a **tinta** que a anota.

⛔⛔⛔ **E o smoke do dono desmentiu METADE desta wave, no mesmo dia** (*«o color picker não
funciona de maneira nenhuma e em nenhum lugar… sempre `#00000000`»*): **eu curei ONDE o conta-gotas
amostra e nunca medi SE ele amostra.** A leitura de reserva dele pedia só a camada do Vello, que
sobre o canvas é transparente **por construção** — e o censo desta wave não o via, porque a pergunta
dele era *«quem resolve o ponteiro pelo quad de repouso?»* e um consumidor que lê a **camada
errada** responde a essa pergunta correctamente. ⇒ `ph2d_render::screen_pick` compõe as **duas**
metades do quadro, e a lei da composição — que vivia dentro de `#[cfg(test)]` — passou a existir
para o produto.

⇒ **[HANDOFF_O_RESTO_DO_APP_ACHAVA_A_ARTE_PLANA_2026-09-15](handoffs/HANDOFF_O_RESTO_DO_APP_ACHAVA_A_ARTE_PLANA_2026-09-15.md)**
— a tabela das três portas, as seis leituras que o diff inverte, as três premissas minhas que a
medição derrubou e a recusa medida com os números.

### F6-l — ⭐⭐⭐ **O *«quase bom»* tinha número: era a CURVATURA dentro do dab, e a malha fina não a alcança** (4.º report, 2026-09-14)

**UMA LINHA:** a marca do pincel sobre arte dobrada fecha em **`1,002`** (era `1,107`), e o que
sobrava **não era facetagem** — refinar a malha de `32` para `8 192` triângulos deixava-a onde
estava. A deformação sob o dab passou de uma matriz a um **polinómio de grau 3**, com duas cercas
medidas que degradam para a elipse de ontem.

⚠️⚠️ **E o 5.º report — *«sem melhorias!»* — estava CERTO:** a cura vive no `Smooth` e o app estava
no `Fast`, onde ela é **inteiramente inerte** com pincel pequeno (`1,127 → 1,127`). ⇒ o `Smooth` é o
de fábrica desde 2026-09-14, e a decisão que ficou pendente duas vezes fechou com a **segunda** razão
medida. *Um número medido numa densidade que o artista nunca alcança é um número sobre outro
programa.*

⭐⭐⭐ **E o 6.º report (*«mais redondo do que nunca… mas pinta com diâmetro menor onde é mais
estreito»*) eram DOIS defeitos, e nenhum era o dab:** a DENSIDADE do traço seguia a dobra (o passo
saía do eixo MAIOR do dab: `44` → `22` → `11` → `5` dabs no mesmo caminho de ecrã) e o TECTO do raio
era a faixa do slider. ⛔ **O que escondia os dois era a minha régua:** `maior/menor` é invariante à
escala e não vê uma marca certa na forma e errada no tamanho.

⭐⭐⭐⭐ **E o 7.º report achou o defeito POR BAIXO dos outros dois: o traço lia a dobra UMA VEZ, no
pen-down.** Um clique solto ficava certo (*«mais redondo do que nunca»*) e um traço pintava tudo com
a dobra do sítio onde começou. ⛔ **Nenhum gate a exercitava ao longo de um CAMINHO** — todos mediam
a lei num PONTO, e ali ela estava certa.

⇒ **[HANDOFF_A_CURVATURA_DEBAIXO_DO_DAB_2026-09-14](handoffs/HANDOFF_A_CURVATURA_DEBAIXO_DO_DAB_2026-09-14.md)**
— as tabelas, as duas cercas, as **seis** leituras que o diff inverte, as **quatro** premissas minhas
que a medição derrubou (entre elas a que este módulo tinha escrita no cabeçalho) e a régua minha que
media o espaçamento das amostras em vez da lei.

### F6-k — ⛔⛔ **A TRIAGEM DA PORTA ABERTA: as duas permissivas estão FECHADAS, e o oráculo que se corre PERDE para a nossa cura** (2026-09-14)

Ordem do dono: antes de trocar a lei dos pesos, correr o oráculo sobre a **nossa** arte. Protocolo
[`_ComoInvestigarApps`](../_ComoInvestigarApps/00_o_metodo.md) §3, com a pergunta escrita antes:
*«o Godot CALCULA os pesos da pele 2D, ou só os APLICA?»*

⛔ **Godot (MIT): não calcula nenhum.** O `Polygon2D` só tem `get/set_bone_weights`, o `Bone2D` só
auto-calcula comprimento e ângulo, e a única acção do binário é **`Paint Bone Weights`** — o artista
pinta-os. ⇒ *ele está onde nós estamos, menos o automatismo.* **Não há nada para portar.**

⚠️ **OpenToonz (BSD-3): tem a coisa e não tem porta.** A `libtnzext.so` traz
`PlasticSkeletonDeformation` sobre uma malha, com `tcg::Vertex<RigidPoint>` — o vocabulário da
família *rigid/ARAP*. Mas o app **não tem consola**, logo não se corre sobre a nossa arte. O fonte
continua aberto — isso é **portar**, não **medir**, e é outra wave.

⭐⭐⭐ **E o único que se CORRE — o Blender — PERDE.** Ele calcula pesos automáticos por difusão de
calor; corrido sobre a **nossa** grelha e o **nosso** esqueleto, com a **nossa** lei de mistura e o
**mesmo código** nos dois lados:

| dobra | nosso (alcance = osso) | **Blender (calor automático)** | nosso (alcance `2,08 ×` a arte) |
|---:|---:|---:|---:|
| `60°` | `0,434 %` | **`0,000 %`** | **`0,000 %`** |
| `90°` | `0,651 %` | `5,273 %` | **`0,000 %`** |
| `120°` | `1,107 %` | `9,397 %` | **`0,000 %`** |
| `150°` | `2,300 %` | `8,876 %` | **`0,000 %`** |

⭐ **A cura que a nossa própria régua deu bate o oráculo em toda a escada.**

⚠️⚠️ **E o PASSO A PASSO diz porquê — a afirmação é sobre O NOSSO MEIO, não sobre o método deles.**
Os pesos do oráculo saem uma **função escada** nesta fixtura (`1` até à junta, `0` depois): a difusão
de calor foi desenhada para uma superfície 3D que **envolve** o osso, e aqui a arte é uma folha plana
com os ossos **dentro do plano dela** — cada osso «vê» só a metade mais perto e a partição sai dura.
*O método não é mau; ele não é do nosso meio.*

⇒ **Nenhuma das três portas leva a um sítio melhor que o que já medimos.**

### E a obra seguinte foi MEDIDA ANTES de ser escrita — e o veredito é NÃO CONSTRUIR

⛔⛔⛔ **A rota que o dono aprovou (derivar o alcance da arte) está REFUTADA**, e por uma régua que não
existia: a da dobra é **cega a uma pele que deixou de deformar**. Alargar o alcance dá `0 %` de dobra
e **cobra `27 %` da rotação** (`110,3°` de `150°` mandados).

⭐⭐⭐ **E as SEIS leis medidas pelas DUAS réguas dizem que a que SHIPA ganha o par:**

| lei | dobra `150°` | segue |
|---|---:|---:|
| **o que shipa (bump `raio = osso` + mistura linear)** | **`3,1 %`** | **`150,0°`** |
| alcance `2,08 ×` a arte | `0,0 %` | `110,3°` |
| pesos automáticos do Blender | `8,9 %` | `150,0°` |
| pesos harmónicos | `11,3 %` | `123,5°` |
| bump + centros de rotação | `8,8 %` | `150,0°` |
| harmónicos + centros de rotação | `16,8 %` | `134,3°` |

⚠️ **A recusa é sobre ESTA fixtura — uma corrente de DOIS ossos** —, e a fronteira está nomeada: com
dois ossos o vector de pesos é um escalar, e a semelhança que define um centro de rotação quase não
tem padrão para distinguir regiões. *Uma corrente longa é outra medição, e é barata.*

⛔⛔ **E um bug meu quase virou conclusão:** a 1.ª versão do solver harmónico sobre-relaxava uma
iteração de **Jacobi**, que diverge para `ω > 1` — resíduo preso em `1,0`, campo binário, e a tabela
dizia `45 %` de dobra. *Um protótipo que contradiz uma publicação é suspeito do protótipo primeiro.*

A corrida, os cinco scripts e a saída com proveniência: [`docs/Skeleton/oraculo/`](oraculo/README.md).

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
| **O sinal de cada junta como restrição DENTRO das varreduras do FABRIK** (a forma clássica; F5-c, 2026-09-14) | **Oscila.** A ida prega a ponta no alvo e re-resolve a corrente inteira sem olhar aos sinais; a correcção desfaz isso. Medido: o erro da ponta **cresce** passagem a passagem em 2 dos 12 alvos do zig-zag (`0,52 → 1,58` num alcance de `3`; `1,87 → 2,00` num de `5`) — e os dois **têm pose exacta**, achada por busca cega sobre os ângulos com os sinais como restrição. A cura é outro solver (descida junta a junta), não outra projecção. |
| **Deitar a junta violada na FRONTEIRA** (a projecção de norma mínima) | Ela move aquela junta o mínimo e custa à CORRENTE o máximo: desfaz a **dobra**, e uma ponta que só se alcança dobrando deixa de se alcançar — `2,43` de erro num alvo a `1,52` de uma corrente de alcance `4`, que tem pose exacta com aqueles sinais. |
| **Amortecer entre a fronteira e o espelho** (`λ · ângulo`, varrido em `0,0 · 0,2 · 0,4 · 0,5 · 0,6 · 0,8 · 1,0`) | Nenhum valor resolve os dois alvos teimosos, e os intermédios são **piores que qualquer um dos extremos** (a `λ = 0,5` três alvos que o `λ = 1` resolve ao bit passam a errar `0,60`–`1,34`). *Não é afinação — é o laço.* |
| **`livre ± 2π` entre os candidatos** do passo do misto | Código defensivo **sem consumidor**: nunca venceu em `900` fixturas, e não pode vencer — a pose de partida é feita dos sinais que dela se leram, logo cada ângulo já está dentro da sua parede e o intervalo vive inteiro dentro de `(−π, π)`, onde o candidato do interior também vive. |
| **Traçar a linha do `IK Chain` pela POLILINHA das juntas** (F5-d, 2026-09-14) | Numa corrente quase esticada ela cai **exactamente** sobre os corpos dos ossos e lê-se como parte deles; numa dobrada, serpenteia. O que o controlo tem de dizer é uma EXTENSÃO, e uma extensão desenha-se como cota: recta e deslocada. |
| **DESLOCAR a recta da corrente para o lado livre** (F5-e, veredito do dono) | A folga contra os ossos em toda pose custa as PONTAS: ela deixa de tocar a junta onde o `Chain` pára e o losango do alvo, e um indicador de extensão que não encosta nas pontas não diz qual extensão é. A corda passa por fora do arco sozinha; em pose esticada a folga vem de a linha ser **fina**. |
| **Deslocar a recta da corrente por uma CONSTANTE** | Não limpa uma corrente que se enrola mais de meia volta: ela tem bojo dos DOIS lados e vem por trás da recta (medido: `18,39 px` de um osso que ocupa `18,75`). O afastamento tem de passar por fora da **excursão** do lado escolhido. |
| **Portar os pesos do Godot** (F6-k, 2026-09-14) | Não há nada para portar: ele **não os calcula**. Só `get/set_bone_weights` e uma acção de painel — *Paint Bone Weights*. |
| **Adoptar os pesos automáticos do Blender** (difusão de calor) | Na nossa fixtura eles dobram `5`–`9 %` da arte acima de `60°`, contra `0,65`–`2,3 %` dos nossos e `0 %` do alcance curado. No nosso meio (folha plana, ossos no plano dela) eles degeneram numa **partição dura** — o método é de outro meio. |
| ⭐⭐⭐ **Os CENTROS DE ROTAÇÃO optimizados** (Le & Hodgins 2016), **re-medidos em 2026-09-15 sobre pesos NÃO degenerados** | ⚠️ **A recusa de 14/09 era *«não dá para julgar por cima de pesos degenerados»* — e essa premissa DISSOLVEU-SE** (arte rígida `37,6 % → 6,6 %`). Re-medido, ele perde na mesma: `11,70 %` de dobra a `90°` contra `8,27 %` do LBS. ⭐ **A causa é OUTRA e é do MEIO:** a nossa arte é uma **folha plana com os ossos a correr pelo meio**, logo o campo de pesos é **simétrico em `y`** — medido, `1 176` pares espelhados com diferença de peso `6,7e-4`. A semelhança do artigo é função **dos pesos e de mais nada** ⇒ não distingue os dois lados, e o centro de ambos cai **no eixo** (`\|y\|` médio `0,0003` numa arte que vai de `−2,4` a `+2,4`; `2 074` de `2 243` vértices a mais de `0,5` do próprio centro). É a limitação que o próprio artigo nomeia — aqui ela **é a forma normal da nossa arte**. ⛔ **Nenhum `σ` cura**: dois pontos com o mesmo vector de peso são indistinguíveis para qualquer função deles. Bancada: `ph2d-skin-weights/src/bancada_centros.rs`. |
| ⭐⭐ **A LEI DO MEIO que as duas recusas acima partilham** | ⛔ *Duas técnicas de topo do campo (difusão de calor · centros de rotação) foram recusadas pela MESMA propriedade da nossa geometria:* uma folha plana com os ossos no plano dela. Qualquer candidata que dependa de **distinguir pontos pelos PESOS** falha aqui, porque os dois lados da folha têm o mesmo vector de peso. ⇒ *sabe-se antes de construir*, e é isso que esta linha vale. |
| **Apertar os pesos** para curar a dobra da pele (F6-j, 2026-09-14) | **Piora, e é a resposta intuitiva:** `0,25 ×` do osso dá `0,64 %` de arte invertida contra `0,17 %` do alcance de hoje. O que dobra a arte é o **gradiente** dos pesos — apertá-los torna-o mais íngreme. Quem cura é ALARGAR: a `2,08 ×` a meia-altura da arte são **zero** pontos invertidos até `150°`. |
| **SUBDIVIDIR o osso (o mecanismo do B-Bone) como cura da dobra** | Com a população de amostras constante ele **piora**: `2,61 % → 4,94 %` a `24` sub-ossos. Com o alcance já certo não cura nada — compra **margem** (`det_min` `0,013 → 0,367`). ⇒ a ordem é o alcance primeiro. ⛔ E a variante «raio encolhe com o sub-osso» lê `0 %` invertido a **`94,5 %` de amostras órfãs**: é a régua a não medir nada. |
| **Ler os lados do modo MISTO da pose VIVA** | Estável enquanto o alvo está ao alcance (o modo é ponto fixo, e há gate) e **apagado para sempre** no primeiro arrasto que o leve para fora dele: fora do alcance a resposta certa é a RECTA, e uma recta não tem lado nenhum para ler. «Inicial» tem de ser o DOCUMENTO. |
| **Fazer a malha SEGUIR a silhueta** em vez de a cobrir (F6-b) | Traz de volta as células deformadas da borda, que são o defeito que a wave cura. O recorte fino é do **alfa da própria arte**, de graça e ao sub-pixel — o *Expansion* do *Puppet* do AE. |


| O quê | Por quê | Onde |
|---|---|---|
| Guardar os **pesos** numa tabela por ordem de varredura | é o *vector paralelo* que o `corner_radius` proíbe por escrito; derivar custa **0,146 %** de um quadro | [`ph2d-skeleton-ecs`](../../crates/ph2d-skeleton-ecs/src/lib.rs) |
| Referenciar um osso por `Entity::to_bits()` | **medido `0 de 2`**: o undo re-spawna e os bits são ids de alocação — a pele morria em silêncio; e o `from_bits` **aborta o processo** com bits de outra sessão | degrau 122 da escada |
| Misturar FK↔IK pelas **posições** das juntas | encurta os ossos (a corda é mais curta que o arco); a mistura é sobre o **ângulo** | `ph2d_skeleton::blend_angle` |
| Um `Driver` novo no `preview_drive` para a âncora | dois memos sobre o **mesmo componente** repõem `Transform`s diferentes na mesma fotografia, e quem ganha é a ordem do `BTreeMap` | `skeleton_goal` (cabeçalho) |
| `chain = 0` (*até à raiz*) como valor de nascimento | é o default do Blender e a queixa nº 1 documentada da feature dele: ao primeiro arrasto o esqueleto inteiro dobra | `DEFAULT_CHAIN = 2` |
