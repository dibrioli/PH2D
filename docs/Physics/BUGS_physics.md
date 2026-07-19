# Bugs do módulo Physics — registro + soluções

> Log vivo dos bugs **não-triviais** da física (sintoma → causa-raiz → tentativas que falharam →
> solução → lições). O objetivo não é listar todo fix (o git já faz isso), mas registrar os bugs
> cuja **causa enganava** — aqueles em que a aparência, ou uma nota escrita antes, levou o
> diagnóstico pra pista errada. Cada entrada termina em **lições generalizáveis**, para o próximo
> agente não repetir o erro.
>
> Estado por-wave: [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md).
> O *porquê* da arquitetura: [ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md).

| # | Bug | Área | Estado | Data |
|---|---|---|---|---|
| [1](#bug-1--a-simulação-rodava-junto-com-a-animação-e-uma-nota-minha-dizia-que-o-interruptor-seria-o-desenho-errado) | **A sim rodava junto com a animação** — e uma nota minha dizia que o interruptor seria o desenho errado | `ph2d-timeline` (transporte) + `ph2d-physics-ecs` (ponte) | ✅ Resolvido (W4b, smoke aprovado) | 2026-07-18 |
| [2](#bug-2--o-collider-não-estava-onde-o-sprite-estava-em-todo-corpo-filho) | **O collider não estava onde o sprite estava**, em todo corpo FILHO | `ph2d-physics-ecs` (5 sítios) + `ph2d-ecs` (o inverso) | ✅ Resolvido (W5; pendente smoke) | 2026-07-18 |
| [3](#bug-3--a-escala-não-alcançava-o-collider) | **A escala não alcançava o collider** — o sprite crescia, o corpo não | `ph2d-physics-ecs` (`body_desc`) + `ph2d-physics` (`ShapeDesc`) + overlay | ✅ Resolvido (W6; smokada pelos gates) | 2026-07-19 |

**Anteriores, catalogados no tracker** (mesma classe — *a causa enganava* — mas escritos por wave,
lá, e não repetidos aqui):

| O quê | Onde |
|---|---|
| A interpenetração **não era falha do solver**: é `v × dt`. Damping, CCD e iterações extras deixaram o número *exatamente* igual | tracker §W2a |
| **Air Drag não era ar**: o `linear_damping` do rapier é decaimento uniforme (25× de massa, todas a 4,8925 m/s) — o erro estava no RÓTULO | tracker §W2b |
| O painel de mundo **não existia no build** (feature ligada na lista `default` errada); tudo a jusante "funcionou" sobre um painel inexistente, sem erro nem warning | tracker §W2b |
| Três gates **nasceram VERDES sobre o bug que existiam pra pegar** — oráculo que computa o esperado *com a função sob teste* | tracker §W2b |
| A âncora do joint **ANDAVA pelo corpo** (1,611 m ao vivo vs 0,642 m pós-Reset) porque `rest` guardava mundo e o spawn re-derivava contra a pose VIVA | tracker §W3 |
| `Kinematic` fez a sim depender de um **fluxo de entrada por-tick** e três lugares supunham o contrário → `SceneAtTick` | tracker §W4 |

---

## Bug #1 — A simulação rodava junto com a animação, e uma nota minha dizia que o interruptor seria o desenho errado

**Estado:** ✅ resolvido em 2026-07-18 (W4b, commit `25bf851e`). **Smoke aprovado pelo Enio** no
mesmo dia — *"smoke OK. Funciona muito bem"*.
O bug de código era pequeno. O caro foi a **nota**: ela teria feito o próximo agente recusar o
pedido do Enio como já-decidido.

### Sintoma

Relato do Enio, logo depois de aprovar o smoke do W4:

> *"os controles de simulação e de animação parecer ser os mesmos e na timeline o play ativa a
> simulação física. Sendo assim temos um conflito: a simulação roda junto com a animação."*

Concreto: com qualquer corpo dinâmico na cena, dar Play ou arrastar a régua para **revisar uma
animação** também **derrubava mais um pouco** cada corpo. A cena que o artista julga nunca era a
cena que ele autorou, e não havia gesto nenhum para separar as duas coisas.

### Causa-raiz

O transporte é **UM relógio com DOIS consumidores** — as curvas da timeline e o mundo rapier — e
isso nunca tinha sido dito em lugar nenhum, muito menos oferecido ao artista. Cada consumidor foi
ligado ao relógio na wave em que nasceu (Motion no W4.T7, física no ADR-0131 W1), cada um
corretamente, e ninguém perguntou *"quando o artista aperta Play, ele está pedindo os dois?"*.

### ⚠️ A parte que enganava: a nota que eu tinha escrito no W4

O `00_plano_waves.md` §W4 dizia, com minhas palavras:

> *"não existe interruptor para isso … e o desligamento manual seria o desenho errado de qualquer
> jeito: o apply da timeline escreve o `Transform` e o readback da física escreve **depois**, então
> um corpo dinâmico recém-assado é sobrescrito pelo solver todo frame."*

O **mecanismo** está certo, e é por isso que o Bake vira `BodyKind::Kinematic`. O erro é de
**escopo**: aquilo respondia *"o Bake deve desligar a física no corpo assado?"* — não, ele
**entrega a pose** — e eu enunciei a conclusão como uma lei sobre **qualquer** interruptor. São
duas perguntas, e elas nem se encostam:

| pergunta | quem responde | resposta |
|---|---|---|
| o solver **roda** neste take? | o **TRANSPORTE** (`simulate_physics`) | o artista escolhe; **off** por padrão |
| quem **escreve a pose** quando ele roda? | o **CORPO** (`BodyKind`) | `Kinematic` depois do bake |

Uma nota assim não fica só errada: ela **fecha a porta**. O próximo agente que lesse o plano
encontraria o pedido do Enio já respondido com "isso é o desenho errado", com um mecanismo real do
lado para dar credibilidade — e teria discutido em vez de construir. Foi corrigida **no lugar, com
data**, e não apagada: a versão antiga explica por que o Bake é do jeito que é.

### A tentativa errada (que é a primeira que ocorre)

**"Off" = pule o `dispatch`.** Compila, e o corpo para de cair — parece pronto. É errado de
**quatro** maneiras independentes, e três delas são invisíveis até muito depois:

| o que se perde | o sintoma, quando aparece |
|---|---|
| `reconcile` | corpo autorado com o toggle off **não existe** — sem contorno de collider, e o mundo tem de ser construído no instante em que se quer ver movimento |
| `settle` | o mundo rapier fica na pose de quando o toggle foi desligado ⇒ **armar teleporta tudo de volta** para onde o artista já não está |
| `last_stepped = target` | 10 s desarmado e a ponte **deve 600 ticks**: um frame simula todos, o app trava e a cena chega onde ninguém pediu |
| `ring.clear()` | o scrub semeia de um checkpoint de **antes** de o artista desarmar e mexer na cena — e só nos ticks que estavam em cache, então o mesmo scrub **discorda de si mesmo** conforme onde cai |

### Solução

Toggle **Physics** na barra de transporte (ao lado de Loop/PingPong — o cluster do *que o relógio
FAZ*, não o de autoria AutoKey/Record), **desmarcado por padrão**, e `PhysicsBridge::hold` do outro
lado: reconcilia, assenta, o relógio segue o transporte, o ring é descartado — e **nada disso
escreve `Transform`**, que é exatamente o que o "off" promete.

`TimelineFlags::simulate_physics`, **não serializado** (a classe do Record) ⇒ `DOC_VERSION` e
`PROJECT_SCHEMA` intactos, e o default sai de graça.

**Preço documentado:** scrubbar para trás sobre um trecho que nunca foi simulado o replaya como se
tivesse sido. Não há resposta melhor a dar — a trajetória daqueles ticks não existe, porque os
ticks não rodaram.

### O default quase quebrou todos os smokes

Com `false` no default, as **7** cenas `PH2D_PHYSICS_SMOKE` abririam **paradas** e leriam como *"a
física quebrou"*. O prólogo do `physics_smoke.rs` arma o flag — são demos de física. E a **cena 7
pede que o artista o DESARME**, que virou a demonstração inteira do Bake: assar converte simulação
em **animação**, e animação é precisamente o que toca com o solver off.

### Lições

1. **Uma nota que responde a pergunta A não pode ser escrita como lei sobre a pergunta B.** O
   mecanismo que eu citei era verdadeiro e específico (a ordem de escrita dentro do frame); a frase
   que o embrulhou (*"de qualquer jeito"*) generalizou para um espaço de decisão que ele não
   cobria. Nota fechando porta é mais cara que código errado: código errado alguém mede, nota
   errada alguém **obedece**.
   → memórias `feedback_stale_comment_and_dead_code_lie`, `feedback_documented_decision_chesterton_fence`.
2. **"Desligado", num subsistema com estado, é um ESTADO — não a ausência de uma chamada.** Pular o
   trabalho deixa quatro invariantes sem dono (existência, sincronia com a cena, o relógio, o
   cache). A pergunta certa não é *"o que eu não faço?"* e sim *"o que continua verdadeiro
   enquanto está off?"*.
3. **Um default que muda o que o produto faz ao abrir tem de ser conferido contra as CENAS DE
   DEMO.** `simulate_physics: false` está certo para um projeto e teria feito as 7 cenas de smoke
   mentirem. Feature opt-in + cena que existe para demonstrar a feature = a cena arma o opt-in.
   → memória `feedback_ready_to_smoke_example`.
4. **Uma busca negativa precisa de controle positivo — inclusive dentro do harness de mutação.**
   Um "sobrevivente" desta wave era o meu filtro: `cargo test --bins timeline_bridge_tests` casa
   com **zero** testes (o módulo é `render_loop::timeline_bridge::tests`), então o verde significava
   *nada rodou* e eu quase registrei um gate perfeito como cego.
   → memória `feedback_a_negative_search_needs_a_positive_control`.
5. **Um gate de painel que constrói o próprio snapshot não testa o publicador.** Apagar
   `self.simulate_physics = state.flags.simulate_physics` do `rebuild` ficava **verde**, porque o
   gate montava o `TimelineViewSnapshot` à mão. O fixture não continha o fenômeno; o gate novo
   percorre `intent → flag → snapshot` dentro da `ph2d-timeline`.
   → memórias `reference_topic_fixture_discipline`, `feedback_a_green_gate_may_be_green_by_accident`.

### Gates que fecham este bug

`the_transport_toggle_decides_whether_play_steps_the_solver` ·
`arming_mid_take_resumes_it_does_not_replay_what_was_skipped` ·
`the_simulation_is_disarmed_by_default` · `a_baked_take_plays_with_the_simulation_disarmed` ·
os 5 de `ph2d-physics-ecs/tests/hold.rs` · os 2 de `transport_physics_seam.rs` ·
`arming_physics_reaches_the_snapshot_the_panel_paints`.
**12 gates, 13 mutações, 13 sangram.**

### Smoke

`PH2D_PHYSICS_SMOKE=7` — assar, **desmarcar Physics**, dar Play: o movimento assado continua
tocando (virou animação) e a caixa não-assada para de cair.

---

## Bug #2 — O collider não estava onde o sprite estava, em todo corpo FILHO

**Estado:** ✅ resolvido em 2026-07-18 (W5). **Pendente de smoke** (`PH2D_PHYSICS_SMOKE=8`).

### Sintoma

Nenhum — e é isso que o torna caro. Parentear um objeto físico na Hierarquia (um
gesto que o app suporta inteiro) fazia o corpo **simular num lugar e desenhar
noutro**, sem erro, sem warning, sem nada na tela dizendo que algo estava
errado. O que o artista via era a arte cruzando paredes que não estão ali e
atravessando paredes que estão.

Medido antes do fix, com uma sonda de 20 linhas:

```
bola autorada em LOCAL (0, 4), sob um pai em (5, 0)  ⇒  desenhada em (5, 4)
  solver simula em      x = 0        ← leu o Transform LOCAL como se fosse mundo
  renderizado em        x = 5        ← pai ∘ local
```

E a divergência **não é constante**: ela muda se o pai se mexe, e cresce com a
profundidade da árvore.

### Causa-raiz

`Transform` é **LOCAL** e compõe com o pai (`Transform::compose`). O solver não
tem hierarquia nenhuma: ele fala **MUNDO**. A ponte lia `&Transform` cru na
entrada e escrevia a pose de mundo crua na saída.

**Para um corpo-raiz os dois coincidem** — e é exatamente por isso que isto
sobreviveu a quatro waves com 190 gates verdes: todo gate, toda cena de smoke e
toda demo usavam corpos-raiz. A premissa "local == mundo" nunca foi escrita em
lugar nenhum; ela era só verdade por acidente da fixture.

### ⚠️ O comentário prometia a wave que nunca veio

O `readback` dizia, em código, desde o W1:

> *"Only touches root-level bodies' local Transform == world for W1 (**child
> bodies land in W2**)."*

O W2 shipou em três pedaços (W2a Inspector, W2b painel de mundo, W2c camadas) e
**nenhum deles tocou nisto**. A nota ficou lá quatro waves, descrevendo um plano
que ninguém executou, e lida de passagem ela *tranquiliza*: parece que alguém já
sabe e já agendou. É a mesma classe do #1 — texto que a próxima LLM obedece — só
que aqui o texto prometia em vez de proibir.

### A armadilha do escopo: **cinco** sítios, não um

O instinto é consertar o `readback` (é onde o número errado aparece). São cinco,
e meia correção é pior que nenhuma — uma cena que compõe na entrada e atribui
cru na saída fica **estável e errada**, derivando um offset-de-pai por frame:

| sítio | direção | o que quebra sozinho |
|---|---|---|
| `reconcile_structure` → `body_desc` | entrada | o corpo NASCE (e repousa, e colide) nas coordenadas locais |
| `settle` | entrada | compara pose local com corpo em mundo ⇒ todo filho parece "movido à mão" **todo frame pausado**, é teleportado e tem a velocidade zerada |
| `drive_kinematic` | entrada | plataforma parenteada anda por um caminho que ninguém autorou — e, sendo kinematic, leva a carga junto |
| `reconcile_joints` | entrada | a âncora do joint é um ponto do MUNDO; lida local, o pino prende onde o artista não marcou |
| `readback` | saída | o renderer compõe o pai **de novo** ⇒ desenha em pai∘mundo |

### Solução

Uma lei, duas direções, **adjacentes**: `ph2d-ecs` ganhou
`Transform::inverse_compose` — o inverso exato de `compose`, ao lado dela — e
`parent_world_transform_into` (o caminhador de ancestrais sem alocar, com o
existente delegando). A ponte tem **duas portas** (`bridge/space.rs`) e os cinco
sítios passam por elas.

**A álgebra do repo é invertível e isso decidiu o desenho.** `compose` soma
rotações, multiplica escalas e cisalha com `[[1, tan sx], [tan sy, 1]]` — então
existe inverso exato, e eu **não** precisei do compromisso "só pai rígido" que
eu ia propor. Erro de round-trip **medido**: `6,08e-6` sobre uma varredura de
rotações, escalas não-uniformes, escala negativa e skew nos dois eixos.

⚠️ **A guarda não é um limiar, e a primeira versão estava errada.** Eu escrevi
`if det == 0.0 { return None }` e o gate da recusa nasceu **vermelho** — em
`f32`, um shear construído para ser singular dá `det ≈ 1e-8`, não `0.0`.
Trocado por **"todo campo do resultado é finito?"**, que é a pergunta que o
chamador de fato tem (*posso guardar isto?*), não precisa de número mágico, e
ainda recusa um `NaN` que chegou na **entrada**. Um pai mal-condicionado (det
minúsculo mas não-nulo) passa **de propósito**: as coordenadas locais saem
enormes mas `compose` as leva de volta à pose de mundo certa, então o par é
auto-consistente e o objeto desenha onde deve.

Por que recusar importa: escrever a alternativa põe `±inf`/`NaN` num
`Transform`, e o próprio `debug_assert` do `compose` diz o preço — **um ângulo
corrompido envenena o `GlobalTransform` da SUBÁRVORE inteira**, com padrões de
NaN signaling-vs-quiet que derivam entre OSes. Um corpo sob um pai escalado a
zero corromperia a cena toda.

### O que NÃO entrou, e por quê

A **escala não alcança o collider** — um sprite escalado 2× tem collider do
tamanho autorado. Isso é **pré-existente e vale para corpo-raiz também**
(`body_desc` lê `col.shape` verbatim), então é ortogonal a esta wave: consertar
aqui misturaria duas correções e a de escala pertence igualmente aos dois casos.
Nomeada no tracker, não contrabandeada.

### Lições

1. **Uma premissa que a fixture satisfaz por acidente não é um invariante — é
   uma coincidência com 190 gates verdes em cima.** "Local == mundo" era
   verdade em todo teste do módulo porque todo teste usava corpo-raiz. Ao
   escrever uma fixture, pergunte que premissa ela está *estabelecendo de
   graça*, e faça uma que não estabeleça.
2. **Comentário que promete uma wave futura apodrece pior que comentário
   errado.** *"child bodies land in W2"* sobreviveu ao W2 inteiro; lido de
   passagem, ele tranquiliza. Se a wave não pegou, a nota tem de virar item de
   backlog com dono — ou sair.
   → memória `feedback_stale_comment_and_dead_code_lie`.
3. **Uma conversão de espaço tem exatamente duas direções, e as duas têm de
   viver juntas.** Metade do fix produz um sistema *estável* e errado, que é
   mais difícil de diagnosticar que um que explode.
   → memória `feedback_two_doors_to_the_same_question_diverge`.
4. **Antes de aceitar um compromisso, leia a álgebra que você já tem.** Eu ia
   propor "só pai rígido, escala não suportada"; `compose` compõe aditivamente
   e é invertível, então o compromisso era desnecessário.
   → memória `feedback_the_representation_can_delete_the_special_case`.
5. **Uma guarda de degeneração pergunta pelo RESULTADO, não pela entrada.**
   `det == 0.0` erra por `1e-8` e é cego a `NaN` que já chegou. *"Todo campo é
   finito?"* é a propriedade que o chamador precisa e não tem número mágico.
   → memória `feedback_a_threshold_must_live_where_the_domain_is_empty`.

### Gates que fecham este bug

`a_parented_body_falls_onto_the_floor_it_is_drawn_above` ·
`the_drawn_pose_and_the_simulated_pose_never_diverge` ·
`a_paused_child_body_is_not_dragged_to_its_local_coordinates` ·
`a_parented_kinematic_platform_carries_its_cargo` ·
`a_parented_joint_anchors_where_it_is_drawn` ·
`a_degenerate_parent_never_poisons_the_transform` ·
`a_root_body_is_unchanged_by_the_conversion` (regressão) — mais os 5 de
`ph2d-ecs/tests/transform_inverse.rs` e o `hot_path_no_alloc` estendido com uma
hierarquia (sem ela, o "não aloca por frame" do caminho novo era afirmado sobre
código que a fixture nunca entrava).
**12 gates, 10 mutações, 10 sangram** — uma por sítio religado, mais a guarda,
o `clear()` do scratch e a subtração de rotação do inverso.

### ⚠️ E a cena de smoke nasceu com a fixture INVERTIDA (2026-07-19)

O primeiro corte da cena 8 tinha dois defeitos, achados pelo smoke do Enio:

**(a) Os rigs eram INVISÍVEIS.** Um rig só carregava `Transform` + `Name`, e o
publicador do gizmo lê `sprite.size`/`resolve_anchor` — uma entidade sem `Sprite`
**não publica `GizmoView` nenhum**. Então os três rigs não desenhavam nada, não
eram selecionáveis no viewport, e a própria instrução da cena mandava *"arraste
um RIG"*: um gesto que a cena tornava impossível. Fix: cada rig ganhou um
quadradinho azul. **Um ator que a demo manda manipular tem de estar na tela.**

**(b) A fixture do rig ROTACIONADO premiava o bug.** A bola estava autorada em
local `(0, 3)`; sob um rig girado 0,45 rad a composição correta a leva para
`x = 1,695` — **1,3 m fora do próprio pedestal**, então ela cai pra sempre e a
cena acusa uma regressão que não existe. Pior: uma implementação que *ignorasse*
a rotação do pai a poria em `x = 3,0`, **bem acima do pedestal, e passaria**.
A cena falhava o conserto e aprovava o defeito.

Fix: a bola é autorada em `R(−rot) · (0, DROP)`, que compõe para exatamente
acima do pedestal **qualquer que seja a rotação** — e agora é *dropar a rotação*
que erra o alvo. Medido antes/depois: `BallTilted` `x = 1,695` (cai pra sempre)
→ `x = 3,000` (pousa em `y = −0,551`, como as outras duas).

⚠️ **O mesmo cegamento estava nos GATES:** todos os 7 usavam `from_translation`,
ou seja **nenhum tinha ancestral rotacionado**. `parented_scene_rot` agora
parametriza a rotação e o gate de aterrissagem varre `[0, 0,45]`. Mutação nova
(compor só a translação do pai) sangra — e sangra **só ali**, o que é correto:
o gate de *"desenhado == simulado"* testa a direção de SAÍDA, que continua certa
mesmo com a entrada errada. Cada camada no seu gate.

### ⚠️ E o primeiro conserto NÃO CHEGOU AO DISCO — e eu o "confirmei" mesmo assim

Vale mais que os dois defeitos acima, porque é sobre método.

O script que aplicava os dois fixes tinha quatro hunks e um `write_text` **no
fim**. O quarto hunk (um texto de instrução) não casou, o `assert` disparou —
corretamente — e o script morreu **antes de escrever**. Os três primeiros hunks
foram perdidos. Eu li o `AssertionError`, consertei só o quarto hunk num script
seguinte, vi o `ok`, e **tratei o `ok` do segundo script como se o primeiro
tivesse aplicado**.

Aí veio o pior: rodei uma sonda que "confirmou o fix" — e a sonda era um
**arquivo de teste separado onde eu mesmo aplicava o offset à mão**. Ela provou
que `R(−rot)·(0,DROP)` pousa, coisa que nunca esteve em dúvida, em vez de provar
que **a CENA faz isso**. Mediu o mecanismo, não o produto. O Enio rodou o smoke
e viu, na tela, exatamente a geometria antiga (bola em `x = 1,695`, rigs sem
sprite): *"nada mudou aqui"*.

**Lições:**

6. **Um script de edição com N hunks e um `write` no fim é atômico no sentido
   errado:** um `assert` tardio descarta silenciosamente os hunks que já
   passaram. Escreva incrementalmente, ou **verifique no disco depois de
   escrever** (`grep` no arquivo, não no buffer).
7. **Verificar uma correção num arquivo que não é o corrigido não é
   verificação.** Se a sonda contém a mudança em vez de importá-la do produto,
   ela só confirma a sua própria aritmética.
   → memória `feedback_harness_reproduces_mechanism_not_context`.
8. **Onde a cena não é dirigível headless, extraia a aritmética.** A cena precisa
   de `gfx` (GPU + janela), então tudo nela que pode ser *conta errada* saiu para
   `RIGS` + `ball_local_offset`, e o gate
   `the_scene_hangs_every_ball_over_its_own_pedestal` lê **essas**. Mutação para
   o `(0, DROP)` que shipou: sangra com `BallTilted starts -1.305 m from its
   pedestal`.

### ⚠️ E havia um SEXTO leitor — o overlay de contorno (2026-07-19)

*"os colliders estão deslocados de suas sprites"* (Enio). Estavam.

Eu enumerei cinco sítios **dentro da ponte** e converti os cinco. O
`render_loop/physics_overlay.rs` é da SHELL e pergunta *"onde está este corpo?"*
por conta própria — `query::<(&RigidBody, &Collider, &Transform)>()` e depois
`t.translation.x/y` cru. Ele desenhava cada contorno na pose **LOCAL** do corpo,
enquanto o sprite é desenhado da cadeia composta.

**E isto reinterpreta o primeiro relato do Enio.** O que ele descreveu como
*"todos os 3 rigs numa mesma posição central e afastados dos seus filhos"* não
eram os rigs: eram os **contornos**, todos desenhados perto de `x = 0` (as
coordenadas locais das três bolas), longe das artes em `x = −3, 0, 3`. Ele
apontou o bug certo na primeira mensagem e eu fui atrás de visibilidade e gizmo.

Fix: uma **porta única de verdade** — `ph2d_ecs::world_transform{,_into}` (a
composição da cadeia + o local, ao lado do inverso). O `bridge::space` passou a
**delegar** a ela, e o overlay a chama. Mutação de volta para o `Transform` cru:
sangra com *"the outline is centred at x = 501.5 px but its sprite is drawn at
201.5 px"* — 300 px = as 3 unidades do rig. ⚠️ **Os 12 gates existentes do
overlay ficaram VERDES**, porque todos usam corpo-raiz; o gate novo
(`a_parented_bodys_outline_sits_on_its_sprite_not_its_local_pose`) é o único que
tem um pai.

**Lição 9 — enumerar os leitores DENTRO de um módulo não enumera os leitores.**
Cinco sítios na ponte pareciam a lista completa porque eu procurei onde a
conversão *deveria* morar. Quem responde *"onde está este corpo?"* é a pergunta
a fazer ao repo inteiro (`grep` por quem lê `Transform` **e** por quem lê o
solver), e a resposta certa é que ninguém deveria responder duas vezes: por isso
a porta subiu para a `ph2d-ecs`.
→ memória `feedback_a_condition_that_enumerates_its_readers_rots`.

### Smoke

`PH2D_PHYSICS_SMOKE=8` — três rigs, cada um com uma bola física parenteada,
cada um sobre um pedestal **estreito**. A regressão é inconfundível por
construção: um corpo que volte a ler a pose local como mundo cai pela linha
`x = 0`, erra o pedestal sobre o qual foi desenhado, e some de quadro.

---

## Bug #3 — A escala não alcançava o collider

**Estado:** ✅ resolvido em 2026-07-19 (W6). **Smokada pelos gates** (2 behavioral
via sim + 2 no overlay), pendente de smoke visual do Enio. Item escolhido pelo
Enio do cardápio pós-integração — *a única CORREÇÃO da lista, não capacidade*.

### Sintoma

Um sprite escalado 2× desenhava 2× (o quad multiplica pela `Transform.scale`),
mas o collider ficava do tamanho **autorado** — a bola de física de um objeto
esticado colidia com o que o artista **não** via.

### Causa-raiz

`body_desc` (a única porta ECS→rapier) lia `col.shape` **verbatim**:
`translation` e `rotation` do `Transform`, nunca `scale`. O overlay fazia o
mesmo. Então collider e wireframe **concordavam entre si** (ambos autorados) e
**os dois discordavam do sprite**.

### ⚠️ A parte que enganava: por que atravessou a linha inteira com 190 gates verdes

**Para um corpo-RAIZ, escala 1:1 é a identidade e o bug não aparece.** Toda
fixture, cena de smoke e demo da linha usava raiz — a premissa "escala é (1,1)"
nunca foi escrita, era **verdade por acidente do fixture**. Exatamente a doença
do Bug #2 (child bodies), um nível adiante: *o gate que pega a classe é ter um
**pai** na fixture*, e nenhum tinha até esta wave. → memória
`feedback_a_condition_that_enumerates_its_readers_rots`.

### A bifurcação que era decisão de PRODUTO, não de código

Escala é **per-eixo** (`Transform.scale` é `Vec2`). Um **Cuboid** toma isso
nativamente. Um **Ball** não: sob escala não-uniforme um círculo é uma **elipse**
na tela, e o rapier não tem elipse nativa. As saídas — *colapsar num círculo*
(Unity/Godot, com aviso) vs *construir a elipse* — mudam o que o produto FAZ, e
o Enio escolheu a **elipse**: o collider casa com o sprite desenhado, o mesmo
princípio do overlay (*"o collider parece redondo mas o desenho é box"*, Bug do
W2a). Colapsar num círculo seria o collider discordando do visível — a própria
classe de bug que a linha combate.

### Solução

- **`ph2d_physics_ecs::scaled_shape(ColliderShape, scale) -> ShapeDesc`** é a
  **porta única**: a ponte (→ rapier) E o overlay (→ o wireframe) resolvem por
  ela, então não podem divergir sobre "que tamanho/forma tem este collider".
- O `t` do `body_desc` já é o WORLD transform (W5) ⇒ `t.scale` é a escala de
  **mundo** ⇒ um corpo sob pai escalado herda a escala do pai (Unity/Godot).
- **Cuboid** → `half·|s|` per-eixo. **Ball uniforme** (`|sx|==|sy|`, limiar
  EXATO ⇒ `(1,1)` byte-idêntico a antes) → **círculo**. **Ball não-uniforme** →
  variant novo **`ShapeDesc::Ellipse{rx,ry}`** (append-only, só na plain-data —
  o `ColliderShape` AUTORADO **não muda**, então **zero bump de schema**; a
  escala já vive no `Transform` persistido), realizada como **polígono convexo**
  (`ellipse_vertices` → `convex_polyline`), tesselação por **`libm::sincosf`**
  (determinismo cross-OS — `physics_ecs_c9` ganhou uma bola escalada).
- O overlay traça o MESMO polígono (`ellipse_vertices`) ⇒ o wireframe senta no
  casco que o solver de fato vê, não numa curva mais lisa por fora.

### Lições

**Lição 1 — um fixture só de raízes esconde toda premissa sobre a hierarquia.**
A 2ª vez nesta linha (Bug #2 foi a 1ª). O gate desta wave inclui de propósito um
**pai escalado** e mede a pose de **repouso** da bola parenteada.

**Lição 2 — "o rapier não tem X" é uma afirmação sobre o rapier, não sobre o
produto** (§0 do CLAUDE.md: não deixe o fallback definir o produto). O rapier
não tem elipse; nós construímos uma. A decisão de *quando* (só não-uniforme) e
*como* (polígono, não colapso) foi do Enio porque muda a sensação do produto.

**Lição 3 — a porta única vale para as DUAS respostas ao mesmo fato.** Collider
e wireframe são saídas diferentes (rapier plain-data vs `BezPath`), mas
respondem à MESMA pergunta ("que forma, deste tamanho?"). `scaled_shape` +
`ellipse_vertices` são as portas; duas cópias divergiriam num screenshot.

### Gates que fecham este bug

`crates/ph2d-physics-ecs/tests/scale_reaches_the_collider.rs` (4 pure + 2
behavioral via sim: a bola 2× repousa mais alto, a **parenteada** repousa como
raiz 2×) · `crates/ph2d-physics/tests/ellipse_collider.rs` (AABB da elipse no
sim + determinismo da tesselação) · `render_loop::physics_overlay` (elipse
desenhada como elipse · o contorno cresce com a escala do PAI). **7 mutações,
todas sangram.**

**Smoke:** `PH2D_PHYSICS_SMOKE=9` — 4 bolas caem, cada uma um `Ball` escalado
diferente: círculo de referência · 2× uniforme (círculo maior, repousa mais
alto) · não-uniforme (ELIPSE, cai deitada e balança) · **parenteada** sob um rig
2× (herda a escala do pai). O oráculo é o contorno (tecla `B`): ele desenha a
forma RESOLVIDA, então um scale→collider morto traçaria o raio autorado dentro
de cada sprite escalado.
