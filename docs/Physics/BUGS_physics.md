# Bugs do módulo Physics — registro + soluções

> Log vivo dos bugs **não-triviais** da física (sintoma → causa-raiz → tentativas que falharam →
> solução → lições). O objetivo não é listar todo fix (o git já faz isso), mas registrar os bugs
> cuja **causa enganava** — aqueles em que a aparência, ou uma nota escrita antes, levou o
> diagnóstico pra pista errada. Cada entrada termina em **lições generalizáveis**, para o próximo
> agente não repetir o erro.
>
> Estado por-wave: [`HANDOFF_line_physics.md`](handoffs/HANDOFF_line_physics.md).
> O *porquê* da arquitetura: [ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md).

| # | Bug | Área | Estado | Data |
|---|---|---|---|---|
| [1](#bug-1--a-simulação-rodava-junto-com-a-animação-e-uma-nota-minha-dizia-que-o-interruptor-seria-o-desenho-errado) | **A sim rodava junto com a animação** — e uma nota minha dizia que o interruptor seria o desenho errado | `ph2d-timeline` (transporte) + `ph2d-physics-ecs` (ponte) | ✅ Resolvido (W4b, smoke aprovado) | 2026-07-18 |
| [2](#bug-2--o-collider-não-estava-onde-o-sprite-estava-em-todo-corpo-filho) | **O collider não estava onde o sprite estava**, em todo corpo FILHO | `ph2d-physics-ecs` (5 sítios) + `ph2d-ecs` (o inverso) | ✅ Resolvido (W5; pendente smoke) | 2026-07-18 |
| [3](#bug-3--a-escala-não-alcançava-o-collider) | **A escala não alcançava o collider** — o sprite crescia, o corpo não | `ph2d-physics-ecs` (`body_desc`) + `ph2d-physics` (`ShapeDesc`) + overlay | ✅ Resolvido (W6; smokada pelos gates) | 2026-07-19 |
| [8](#bug-8--os-players-pulavam-e-andavam-sozinhos-e-a-física-estava-imóvel) | **Os players pulavam e andavam sozinhos** — e a física estava IMÓVEL | `shells/desktop` (o caminho da ENTRADA) | ✅ Resolvido (smoke aprovado) | 2026-08-12 |
| [9](#bug-9--o-leque-de-sensores-driftava-do-corpo-e-a-geometria-estava-certa) | **O leque de sensores driftava do corpo** — e a geometria estava certa | `ph2d-physics-ecs` (a ÂNCORA da leitura) | ✅ Resolvido (smoke aprovado) | 2026-08-12 |
| [10](#bug-10--na-2ª-vida-o-herói-andava-e-rodava-sozinho-e-nada-o-empurrava) | **Na 2.ª vida o herói andava e rodava sozinho** — e nada o empurrava | `ph2d-app-physics` (a GRAVAÇÃO da fita) | ✅ Resolvido (pendente smoke) | 2026-10-05 |
| [11](#bug-11--o-wall-jump-com-altura-19-20-atira-o-controle-ao-topo-da-parede) | **O wall jump com altura ~1,9–2,0 atira o controle ao topo da parede** | `ph2d-physics-ecs` (a ponte) | 🔴 ABERTO — linha GEF02 | 2026-11-12 |

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
os 5 de `ph2d-physics-ecs/tests/it/hold.rs` · os 2 de `transport_physics_seam.rs` ·
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
`ph2d-ecs/tests/it/transform_inverse.rs` e o `hot_path_no_alloc` estendido com uma
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

`crates/ph2d-physics-ecs/tests/it/scale_reaches_the_collider.rs` (4 pure + 2
behavioral via sim: a bola 2× repousa mais alto, a **parenteada** repousa como
raiz 2×) · `crates/ph2d-physics/tests/it/ellipse_collider.rs` (AABB da elipse no
sim + determinismo da tesselação) · `render_loop::physics_overlay` (elipse
desenhada como elipse · o contorno cresce com a escala do PAI). **7 mutações,
todas sangram.**

**Smoke:** `PH2D_PHYSICS_SMOKE=9` — 4 bolas caem, cada uma um `Ball` escalado
diferente: círculo de referência · 2× uniforme (círculo maior, repousa mais
alto) · não-uniforme (ELIPSE, cai deitada e balança) · **parenteada** sob um rig
2× (herda a escala do pai). O oráculo é o contorno (tecla `B`): ele desenha a
forma RESOLVIDA, então um scale→collider morto traçaria o raio autorado dentro
de cada sprite escalado.

---

## Bug #4 — A âncora de um joint entre corpos PARENTEADOS nascia em espaço LOCAL

**Achado pela W-Rig (2026-07-31), pré-existente desde o W3.** Consertado no mesmo
commit; a cena `=67` é o repro visual.

### Sintoma

Um boneco rigado a partir da Hierarquia **esparramava** ao dar Play: cada membro
pendia de um ponto flutuando no ar longe dele, e o par tronco↔cabeça separava
**1,09 m em 3 s** — mais do que a geometria do Pin permite por rotação (o máximo
seria a soma dos dois braços, 0,7 m).

### Causa-raiz

`create_joint_at` computava o ponto médio a partir de `Transform.translation`
**cru**:

```rust
let pa = sim.world().get::<Transform>(a)?.translation;
let pb = sim.world().get::<Transform>(b)?.translation;
let origin = (pa + pb) * 0.5;
```

E `Transform` é **LOCAL**: ele compõe com o pai (a lei que o W5 escreveu). Para um
corpo-RAIZ local e mundo coincidem; para um corpo-FILHO, `pb` é o **offset dentro do
pai**. O "meio entre os dois" saía entre a pose de MUNDO de um e o OFFSET do outro
— um lugar que não é nem um nem outro.

Medido na cena 67: o tronco em `(0, 3,0)` e a cabeça em `(0, 3,7)` de mundo
(local `(0, 0,7)`) davam âncora em **`(0, 1,85)`** — o meio entre `3,0` e `0,7` —
quando a emenda está em `y = 3,5`. **1,65 m abaixo.**

O mesmo erro estava no `pose` do lado da conversão mundo→local, então a rota do
**canvas** plantava a mesma âncora errada.

### ⚠️ A parte que enganava: por que atravessou o W3 inteiro

**Toda** fixture, cena de smoke e demo desta linha usava corpos-**RAIZ** — onde local
e mundo são o mesmo número, e o bug é literalmente invisível. É a mesma frase que o
W5 escreveu sobre si mesmo (*"a premissa nunca foi escrita, era verdade por acidente
do fixture"*), um arquivo ao lado, e ela reincidiu porque a nota do W5 enumerou os
**cinco sítios da PONTE** — e a porta de CRIAÇÃO não estava entre eles.

E o sintoma não acusa a causa: o joint EXISTE, o overlay o desenha, o solver o honra.
O que se vê é um ragdoll ruim, que se lê como *"a física está frouxa"* — nunca como
*"a âncora está no lugar errado"*.

### Solução

As duas leituras passam por `ph2d_ecs::world_transform` (`inspector_joint_create.rs`).
Uma linha cada, na PORTA — as três rotas de criação (Join, Draw, Rig) a compartilham,
então nenhuma pode divergir.

### Lições

- **Enumerar sítios apodrece; o W5 enumerou cinco e o sexto nasceu fora.** O que
  fecha a classe é a pergunta *"esta função lê `Transform` para compará-lo com outro
  espaço?"*, feita de toda função nova.
- **O gate red-first é o que prova a causa.** Ele nasceu vermelho em `y = 1,850` — o
  número EXATO que a sonda da cena tinha medido. Um gate que nasce vermelho no valor
  previsto é a diferença entre corrigir a causa e corrigir um sintoma.
- **A fixture tem de conter o fenômeno**, e aqui o fenômeno é *ter um pai*. O gate
  novo (`a_joint_between_parented_bodies_anchors_in_world_space`) é o primeiro desta
  linha a parentear um dos dois corpos.

### Gates que fecham este bug

- `render_loop::inspector_joint_tests::a_joint_between_parented_bodies_anchors_in_world_space`
  — o pivô cai em `y = 3,35` (o meio das poses de MUNDO), e a mensagem nomeia o
  `1,85` como a assinatura do defeito.
- `physics_smoke_rig::tests::the_message_of_scene_67_quotes_measured_numbers` — a
  separação do pescoço tem de ficar em milímetros; ela media 1,09 m.

---

## Bug #5 — O Reset não devolvia nenhum corpo PARENTEADO

**Reportado pelo Enio no smoke da cena 67** (*"o reset não consegue devolver o
conjunto à posição original"*, 2026-07-31). **Pré-existente desde o W5**, e nem o
rig nem os joints tinham parte nisso.

### Sintoma

Depois de simular e apertar Reset, a RAIZ do boneco voltava exata (0,000 m) e
**toda parte filha aterrissava 2,3–4,2 m fora**. O screenshot mostra o boneco
partido em dois grupos com os joints esticados através da tela.

### Isolamento

Um par pai+filho de corpos dinâmicos, **sem joint nenhum**:

| corpo | autorado | pós-Reset | erro |
|---|---|---|---|
| P (raiz) | `(0, 3)` | `(0, 3)` | **0,000 m** |
| C (filho) | `(1, 3)` | `(1, 7,91)` | **4,910 m** |

4,905 m é exatamente `½·g·t²` para o segundo simulado — **a queda do PAI, aparecendo
dentro do local do filho**.

### Causa-raiz

`Transform` é LOCAL e compõe com o pai, então o `readback` converte a pose de MUNDO
que o solver deu para o local que o **pai vigente** exige. O mapa de corpos é um
`BTreeMap<Entity>` — e a ordem de `Entity` **não é a de spawn**. Instrumentado:

```
[rb] e=1v0 solver=(1.000,-1.910) parent=(0.000,-1.748)
[rb] e=0v0 solver=(0.000,-1.910) parent=(0.000, 0.000)
```

O **filho é escrito primeiro**, contra o pai do frame ANTERIOR.

### ⚠️ A parte que enganava: durante o play o erro é de 3 mm

Escrito contra o pai de um frame atrás, o filho fica um frame atrás — 3,2 mm num
par caindo, invisível a olho e indistinguível do movimento. **Foi assim que
atravessou o W5 inteiro.** Num REWIND o "frame anterior" é o mundo inteiro antes do
salto, e o mesmo defeito vale 4,91 m.

### Solução

O readback escreve em ordem de **PROFUNDIDADE na hierarquia**: todo ancestral tem
profundidade estritamente menor, logo já foi escrito. Ordenar apenas *"raízes
primeiro"* consertaria um nível e deixaria o neto com o mesmo defeito — daí o gate
de três níveis.

### Lições

- **Um erro pequeno no regime comum é um erro grande em algum regime**, e o regime
  que o expõe pode não ser aquele em que ele foi introduzido. O play escondeu por
  um ano o que o Reset gritou.
- **A ordem de iteração de um `BTreeMap<Entity>` não é a de spawn.** Qualquer passe
  em que a ordem importa tem de a ESTABELECER, não herdá-la.

### Gates que fecham este bug

`ph2d-physics-ecs/tests/it/readback_order.rs` — o Reset (nasceu vermelho em 4,9101 m),
o play (0,0032 m) e a cadeia de três níveis. Mutação: tirar a profundidade da chave
de ordenação derruba os três.

---

## Bug #6 — A âncora nascia no meio dos centros, não na emenda

Não é um defeito reportado: é a **aproximação** que a regra antiga assumia, tornada
exata quando o Enio mandou buscar o padrão-ouro (2026-07-31).

O doc da regra dizia o que ela aproximava — *"entre dois corpos que se TOCAM (um elo
de corrente, o caso comum) o meio É o pivô certo"*. É verdade, e **só** entre formas
do mesmo tamanho. Uma cabeça de 0,4 sobre um tronco de 1,0 encostados: a junta está
em `y = 3,5` e o meio dos centros em `3,35` — o pivô **dentro do peito**, e a cabeça
girando em torno dele.

A emenda (`ph2d_physics_ecs::seam_point`) é o meio entre os dois pontos em que as
SILHUETAS cruzam a linha dos centros, e **reduz ao ponto médio no caso que o desenho
antigo acertava**. Sem geometria nova: `ShapeDesc::radial_fraction` — a régua do
falloff de área — é uma função-calibre, então o alcance da silhueta numa direção é
`1 / f(d)`, em forma fechada, já com a escala do W6 e o offset do collider dentro.

⚠️ **E ela expôs uma afirmação minha que virou falsa no mesmo commit:** o doc do
gerador de rig dizia que a ordem entre *dar corpo* e *ligar* não importava porque os
dois tocam componentes disjuntos. Era verdade até a emenda passar a medir o
`Collider` — que ainda estava na FILA. O sintoma não seria um erro: seria a emenda
caindo em silêncio no fallback do ponto médio. Medido: o pescoço nascendo em 3,35 de
novo. O flush entre as duas metades é load-bearing e agora mora dentro do gerador.

---

## Bug #7 — O personagem parado SUBIA a rampa sozinho, e o freio da caminhada não tinha o que ver

Relato do Enio no smoke da W10 (2026-08-04): *"nas rampas, se parado, a depender do
Float Height ele pode subir a rampa sozinho bem devagar"*. Medido antes de uma linha
de código: numa rampa de 30° ele subia a **3,3 cm/s, para sempre** — regime
permanente, não transiente (varrido em janelas de 2 s por 30 s: a taxa não muda).

### O que NÃO era, e cada refutação custou uma medição

- ⛔ **Não era oscilação.** A primeira sonda reportou "amplitude" de 3,8 cm e eu quase
  escrevi *ciclo-limite*: a métrica era `ymax − ymin` numa janela, e numa rampa isso
  **mede a própria subida**. Tique a tique a velocidade é **constante** e a folga
  perpendicular não varia um dígito. *Uma amplitude medida sobre uma rampa mede a
  rampa.*
- ⛔ **Não era o erro de repouso da mola.** A perna paira 5,75 mm acima do pedido
  (linear no amortecimento), e a de-polarização de meio tique que o corrige para
  0,38 mm **piorou** a deriva (0,164 → 0,343). Construída, medida, revertida.
- ⛔ **Não era a rigidez.** `k` de 40 a 6400: a deriva não se move.
- ⛔ **Não era o pouso, nem a plataforma móvel, nem o relógio irregular.** Cinco
  configurações de repouso medidas (plano estático · vagão dinâmico a 3 m/s · vagão
  kinematic · `float_height` abaixo do mínimo geométrico · dispatch devendo 0/1/2
  tiques): **nenhuma** produz movimento. O plano fica parado ao sexto decimal.

### O que era

**A ablação pela gravidade fecha a conta**: a deriva é **linear em `g`** e vale
**exatamente zero** num mundo sem gravidade (−9,81 → 0,164 · −4,00 → 0,067 ·
−1,00 → 0,017 · 0,00 → **−0,000**).

A perna cancela a gravidade com um **impulso no topo do tique**, e o `rapier` integra
a gravidade **ao longo** dele. Os dois somam zero em velocidade e **não** em
deslocamento: sobra meio tique de velocidade para cima, que numa rampa tem componente
**tangente**. O freio da caminhada removeria isso — mas ele é um controlador de
VELOCIDADE amostrado no fim do tique, e ali a sobra já foi devolvida. Medido no
estado assentado: `v = (0,0332, −0,0575)`, **perpendicular à tangente ao quarto
decimal**, com o freio a calcular um empurrão de `−8,7e−8`. Ele não está falhando:
não há nada que ele consiga ver.

E o que sobra ninguém remove, porque o **eixo do amortecedor da perna era o `up`**.
Numa rampa `{tangente, up}` não é ortogonal: a componente ao longo da NORMAL tem
projeção zero na tangente (a caminhada é cega) e a parte vertical dela é o que a mola
precisa para segurar a altura (o amortecedor não pode removê-la sem largar o
personagem). **Modo marginal**, e o personagem desliza nele para sempre.

### A correção

`ph2d_platformer::damping_axis` — o amortecedor passa a agir ao longo da **NORMAL do
chão**, com recuo para `up` numa normal degenerada. A premissa antiga não era errada
quando nasceu: **no plano a normal É o `up`**, e o plano era a única fixture. Com a
normal, `{tangente, normal}` é ortonormal e as duas leis cobrem o plano inteiro.

⚠️ **No plano é byte-idêntico e não por aproximação:** a normal de uma face horizontal
é `[0, 1]` exata, `sqrt(1.0)` é `1.0`, e dividir por um é a identidade. O
`physics_ecs_c9` saiu **`b3dbe792…`, o mesmo do W10** — a lane do player é em chão
plano, e é o oráculo mais forte disponível para essa afirmação.

**Deriva a 30°, 10 s, `float_height = 0,9`:**

| `spring_damping` | eixo no `up` (antes) | eixo na NORMAL (agora) |
|---|---|---|
| 0,25 | 0,33 m | 0,2476 m |
| **0,50** (o que shipa) | **0,3295 m** | **0,1644 m** |
| 0,75 | 0,33 m | 0,0819 m |
| 1,00 | **0,3276 m** | **0,0000 m** |

⚠️ **A coluna da esquerda é o achado:** com o eixo no `up`, **nenhum** valor do knob
removia a deriva. O eixo é o que transforma `spring_damping` num controle que de fato
governa isto.

### ⚠️ O que sobra, e por que o default NÃO subiu para o teto

No teto a deriva é zero — e o personagem passa a pesar **53%** do que devia. A perna
segura-o em parte com um **boost**, e um boost não é força: o `react` não o devolve
ao chão, por uma cerca **medida** daquele módulo (devolvê-lo fazia a jangada disparar
para −0,946 m e derivar sozinha). Quanto mais o knob amortece, mais do peso é
segurado por escrita de velocidade e menos chega ao chão — a jangada da cena `=85`
afunda 17 cm em vez de 27.

⚠️ **E subir a RIGIDEZ não recupera o peso** (medido, `k` de 400 a 6400): o erro de
repouso cai `∝ 1/k` mas o produto `k · erro` — a força que falta — fica **constante
em 4,60 m/s²**. Não há knob que pague as duas coisas.

⚠️ **Veredito do Enio no smoke (2026-08-04):** *"No setup como está o player sobe
sozinho bem devagar. Mas faremos os ajustes amanhã."* — o resíduo do default É
visível no produto, exatamente como a medição diz, e **o ajuste está DIFERIDO por
ordem dele**, com as quatro colunas e a cura já medidas.

⇒ O default fica em `0,5` com o resíduo medido, o knob passa a oferecer a troca de
verdade, e a cura que compra as duas está nomeada no plano (**a perna SUBSTITUI a
gravidade em vez de a cancelar**: sem impulso de cancelamento não há assimetria a
retificar, e o `react` devolve o peso explicitamente em vez de o ler da mola).

### ⚠️ E o "pulinho" do mesmo relato NÃO reproduziu

O relato irmão — *"de tempos em tempos enquanto está parado o player dá pulinhos
involuntários"* — foi procurado nas cinco configurações listadas acima e em nenhuma
há salto: a folga fica constante ao sexto decimal e nenhum tique move mais de 2 mm.
A única motilidade de repouso que o produto tem é a subida de rampa deste bug. Se ele
sobreviver ao próximo smoke, a sonda `measure_idle` é onde a próxima hipótese entra —
e ela já traz o instrumento (folga por tique, saltos por tique, partição por chão).

**Gates:** `ph2d-platformer::ride::tests` (o eixo: byte-identidade no plano · andar ao
longo da rampa não acorda o amortecedor · o teto zera a aproximação · o default é uma
decisão) + `ph2d-physics-ecs/tests/it/platform_idle.rs` (o produto: com o amortecedor no
teto o personagem fica parado em 10°/20°/30°/40° e por um minuto · o resíduo do
default é medido dos dois lados · o plano é o controle · a perna segura a altura).
**4 mutações, 4 sangram** — o eixo de volta ao `up` sangra a lei **e** o produto.

---

## Bug #7b — E a subida que sobrou era o impulso da perna a cair todo no topo do tique

**A segunda metade do #7**, aberta pelo veredito do Enio no smoke de 04/08 (*"no
setup como está o player sobe sozinho bem devagar; mas faremos os ajustes
amanhã"*). O eixo do amortecedor tinha cortado a deriva ao meio e deixado
**0,1644 m em 10 s a 30°** com o default que shipa — visível, e nomeado.

### ⚠️ A cura que este doc prescrevia estava certa no MECANISMO e errada no REMÉDIO

A §7 escreveu, e continua verdade: *a perna cancela a gravidade com um impulso no
topo do tique, e o `rapier` integra a gravidade ao longo dele*. O que ela não fez
foi perguntar **quem é "ao longo"** — e a resposta estava a três linhas de
distância, no `world/step.rs`: um tique é um laço de `substeps` sub-passos, e
`drag`, `effector`, `blast` e as polias são **todos** aplicados dentro dele, com
o comentário que diz por quê (*"per SUBSTEP, not per tick: a force applied once
per tick would be wrong by the substep count"*). O motor do player era o único
aplicado uma vez, antes do laço.

⇒ O remédio prescrito (*"a perna SUBSTITUI a gravidade"*, com uma escrita
por-tique de `gravity_scale`) atacava o sintoma por fora. O remédio verdadeiro é
uma linha de integração: **o que cancela a gravidade é integrado como ela.**

### As três medições que decidiram, e as duas hipóteses que morreram

**(1) O discriminador de sub-passos** (`measure_substep.rs`). Se a causa fosse só
o descasamento de integração, a deriva teria de **sumir com `substeps = 1`** (aí
o impulso e a gravidade caem na mesma integração). Medido:

| substeps | d = 0,00 | d = 0,50 | d = 1,00 |
|---|---|---|---|
| 1 | 0,3065 | 0,1533 | 0,0000 |
| 2 | 0,3576 | 0,1788 | 0,0000 |
| **4** (o produto) | 0,3832 | **0,1916** | 0,0000 |
| 8 | 0,3960 | 0,1980 | 0,0000 |

⚠️ **Não zerou** — logo a hipótese ingénua está REFUTADA. Mas a tabela ajusta
`deriva = A·(1 − 1/(4n))·(1 − d)` com `A = 0,409`, e **`A` é exactamente
`a·t · dt²/2` por tique**: o limite `n → ∞` não é zero, e o que sobra ali é o
impulso da perna a ser pago de uma vez em vez de espalhado. A gravidade era
metade da história; a outra metade é o próprio bloco.

> ### ⚠️ A TABELA ACIMA É **PRÉ-`gravity_hold`**, E O SINAL DELA INVERTEU (W26)
>
> Ela é o que **decidiu** a wave, e por isso fica — mas ela descreve o mundo de
> **antes** de o cancelamento da gravidade passar a ser integrado como a
> gravidade. Re-medida no produto de hoje (2026-08-07, mesmo rig, mesmas
> colunas), a série **cai pela metade a cada dobra**, com o `n = 1` idêntico:
>
> | substeps | d = 0,25 | d = 0,50 | d = 1,00 |
> |---|---|---|---|
> | 1 | 0,2299 | 0,1533 | 0,0000 |
> | 2 | 0,1150 | 0,0766 | 0,0000 |
> | **4** (o produto) | **0,0575** | **0,0383** | 0,0000 |
> | 8 | 0,0287 | 0,0193 | 0,0000 |
> | 12 (o teto do knob) | 0,0194 | 0,0130 | 0,0000 |
>
> ⇒ **a deriva é `∝ 1/n`**, e não `A·(1 − 1/(4n))`. Quem ler a tabela velha hoje
> ajusta uma lei cujo limite `n → ∞` é `A` e conclui que subir sub-passos **não**
> ajuda — o contrário do que o produto faz.
>
> ⚠️ **E isto reconcilia o item (3) abaixo:** fatiar o motor *"corta a deriva
> 4×"* é exactamente **uma potência de `n`** no default `n = 4`. Fatiar não
> removia o defeito; ele deslocava a série de um degrau — e o degrau já é
> comprável pelo knob de sub-passos que o artista tem.
>
> ⚠️ **A consequência de PRODUTO** (e a razão de a W26 existir): o **quique do
> pouso NÃO segue os sub-passos** — medido, 32,7 mm a `n = 4` contra 32,4 mm a
> `n = 12`, com `d = 0,25`. A deriva e o quique deixaram de estar soldados: o
> knob move os dois, os sub-passos movem só a deriva. Gate:
> `platform_idle.rs::the_bounce_is_a_fact_of_the_knob_and_the_drift_is_a_fact_of_the_substeps`
> (a mutação que reverte a `gravity_hold` devolve a razão **1,033** — ou seja
> reproduz a tabela velha, que é a prova de quando ela foi medida).

**(2) A componente horizontal veio dos PIXELS, não de um raciocínio.** A `d = 0`
todas as forças são verticais (a mola é no `up`, a gravidade também) e o freio da
caminhada mede `−8,7e−8`: não há ator horizontal. E mesmo assim o personagem
anda **exactamente ao longo da rampa** — `dy/dx = 0,5774 = tan 30°` a quatro
casas, com deslocamento por tique **constante**. É a assinatura de *a velocidade
dentro do tique não é a velocidade amostrada na fronteira dele*: o freio é um
controlador de velocidade, o que sobra é deslocamento, e ele é cego a isso.

**(3) ⛔ Fatiar o motor INTEIRO foi construído, medido e REJEITADO.** Ele corta a
deriva 4× — e faz um `spring_damping = 0` **LARGAR o personagem** (erro de
repouso 0,030 → **1430 mm**). O porquê é livro-texto: uma mola integrada por
força amostrada na posição do início do tique é **explícita**, e uma mola
explícita sem amortecimento ganha energia; o agrupamento no topo era um
amortecimento numérico acidental que a escondia. O piso medido não é sequer um
número — ele **anda com a rigidez**, que é autorável:

| `spring_strength` | piso de `spring_damping` |
|---|---|
| 100 | 0,01 |
| 400 (o default) | 0,04 |
| 1600 | 0,14 |
| 3200 | 0,25 |

⇒ um knob cujo valor mínimo é função de outro knob, que é a forma que este repo
chama de bug de design ([[feedback_ergonomics_verdict_is_a_design_bug]]). **Não
refaça.**

### A correção

**Fatiar SÓ o que cancela a gravidade**, e deixar a mola no ordenamento agrupado
que a mantém estável. A lei DECLARA qual metade é qual
(`PlayerStep::gravity_hold`), a ponte separa as duas, e o wrapper paga uma por
sub-passo (`PhysicsWorld::queue_player_hold`) e a outra no topo
(`apply_player_motor`).

⚠️ **A lei declara em vez de a ponte deduzir**, e a diferença é um tique: no
instante da decolagem o raio ainda vê o chão e a mola já está calada. Uma ponte
que subtraísse `− gravity` sempre que houvesse amostra de chão adivinharia — e
adivinharia errado exactamente ali.

⚠️ **A reação da 3ª lei NÃO é tocada.** A força que o pé faz no chão é um facto
físico (`m·(up·k − g)`) e não muda com o modo como o impulso é distribuído dentro
do tique: o `react` continua a ler o `Motor` inteiro, e é por isso que esta wave
não move um único gate daquele módulo.

### O resultado, nas duas colunas que a §7 disse não poder pagar juntas

| `spring_damping` | deriva (30°, 10 s) | erro de repouso | peso transmitido |
|---|---|---|---|
| 0,25 | 0,2476 → **0,0498** | 2,874 → **0,574 mm** | 88% → **98%** |
| **0,50** (o que shipa) | 0,1644 → **0,0331** | 5,748 → **1,150 mm** | 77% → **95%** |
| 0,75 | 0,0819 → **0,0165** | 8,622 → **1,723 mm** | 65% → **93%** |
| 1,00 (o teto) | **0,0000** | 11,496 → **2,297 mm** | 53% → **91%** |

**Cinco vezes menos deriva em TODO valor do knob**, e o peso de volta. ⚠️ E o
**quique do pouso não se moveu** (196 → 199 mm a 0,25; 20 → 24 a 0,50): a
correção é de integração, não de lei, e é essa coluna que o prova — o que o
artista aprovou no smoke da W6/W9 continua igual.

⚠️ **O que isto muda de PRODUTO não é o default, é o preço do teto.** O
`spring_damping = 1,0` sempre deu deriva exactamente zero; o que o impedia de ser
escolhido era custar metade do peso do personagem. Hoje custa 9%. O default fica
em 0,50 porque o teto também zera o quique do pouso, e um pouso sem quique é
outra sensação — **quem decide isso é o smoke, não a medição**.

### ⚠️ E o que sobra, com o mecanismo em FORMA FECHADA

**Veredito do Enio no smoke (2026-08-05):** *"Quase perfeito na rampa! Sobe
muitíssimo devagar, quase imperceptível. Jangadas Smoke OK."* — as duas metades
confirmadas no produto: a deriva quase fechou e **o peso voltou** (a jangada era
a cena que a media).

O que sobra é `0,0331 m em 10 s` no default, e a sonda o pôs em forma fechada.
Medido no referencial da rampa, assentado: o deslocamento **NORMAL é exactamente
zero** e o **tangencial é constante** — o personagem desliza pela superfície sem
subir nem afundar nela.

⚠️ **E há uma razão estrutural para ele não se auto-corrigir:** um raio VERTICAL
sobre um plano inclinado mede a mesma distância em qualquer ponto da rampa, e a
mola empurra ao longo do `up`. ⇒ **o sistema não tem rigidez TANGENTE nenhuma**.
Nada restaura a posição ao longo da ladeira; qualquer impulso tangente vira
deriva permanente, e a única coisa que a remove é o freio da caminhada — que é um
controlador de VELOCIDADE e não vê deslocamento.

A aritmética fecha ao quarto decimal:

```text
  ds/tique = a_t · dt²        com  a_t = D + k·(up·t)
  D  =  g·sen θ · (n−1)/(2n²)  =  4,905 · 3/32  =  0,4598   (medido: 0,4595)
```

⇒ o que sobra é **meio sub-passo de descasamento entre o impulso fatiado e a
gravidade do `rapier`**, e o termo tangente da própria mola é quem o cancela — é
por isso que o resíduo é exactamente linear em `(1 − d)` e **zero no teto**.

⛔ **Duas tentativas de fechar os 9% que sobram, medidas e REJEITADAS:**

1. **Fatiar também o termo da CAMINHADA** — **zero mudança**, tabela idêntica ao
   dígito. O freio calcula `−8,7e−8` no repouso: não há o que fatiar.
2. **Pagar a fatia DEPOIS do `physics_pipeline.step`** em vez de antes — **pior**:
   deriva `0,0383 → 0,0639` e o erro de repouso **inverte o sinal** (+1,150 →
   −1,915 mm, o personagem passa a pairar ABAIXO do pedido). ⚠️ E a inversão é o
   achado: o instante que zeraria as duas está **dentro** do pipeline do
   `rapier`, entre as duas portas que temos. Um fator afinado entre elas seria um
   número mágico dependente das entranhas de uma dep — exactamente o que este
   repo recusa.

⇒ **O ponto de parada da INTEGRAÇÃO é aqui**, e ele tinha uma saída exacta: o
**Spring Damping no teto** zera a deriva por completo e hoje custa **9% do peso
em vez de 47%** — a troca que a wave tornou pagável. ⚠️ **Ela foi TOMADA no
mesmo dia** (o §7c, abaixo). Fechar a deriva *sem* mexer no knob pediria uma mola
**semi-implícita** (re-amostrar o raio por sub-passo), que é outra wave e cobra
um raio por sub-passo.

**c9:** `b3dbe792…` → **`2278035e…`**, 108 corpos, debug ≡ release. Ele **tem** de
se mover: a altura de repouso do player mudou em toda cena que tenha um.

**Gates:** `ph2d-platformer` (a lei declara o canal · e ele é zero no ar E no
tique da decolagem) + `ph2d-physics::world::player` (o canal fatiado entrega o
MESMO impulso total · a fila é drenada · zero não entra nela) +
`ph2d-physics-ecs/tests/it/platform_idle.rs` (o resíduo do default, pinado dos dois
lados · o teto custa um décimo do peso e não metade). **A mutação — agrupar o
`gravity_hold` de volta no motor — sangra os dois gates de produto com os números
pré-wave exactos (0,1916 m e 11,496 mm) e deixa os outros quatro verdes**, que é
o par certo: eles medem coisas que a wave não mudou.

---

## Bug #7c — E a subida que restava era uma ESCOLHA de default, não um defeito

**Reportado:** Enio, 2026-08-05, no smoke seguinte ao §7b —
*"continua subindo um pouco mais rápido que antes, ainda bem discreto"*.

### O que a percepção media, e por que ela estava certa

Duas leituras cabiam na frase, e elas pedem curas opostas: *o mesmo resíduo, e o
artista o notou melhor* × *o resíduo REGREDIU*. Antes de qualquer hipótese, a
varredura que faltava (`measure_the_drift_against_the_slope`, 10 s parado, 4
sub-passos):

```text
  rampa     d=0.25     d=0.50     d=0.75     d=1.00
    20      0.0393     0.0262     0.0131     0.0000
    30      0.0575     0.0383     0.0192     0.0000
    35      0.0659     0.0439     0.0220     0.0000
    40      0.0739     0.0493     0.0246     0.0000
    45      0.0813     0.0542     0.0271     0.0000
    50   587.0293   587.0293   587.0293   587.0293
```

Três leituras, e a ordem importa:

1. **A percepção estava certa e é PREVISÍVEL.** A deriva escala com `sen θ`:
   a 40° ela é **1,29×** a de 30° (0,0493 contra 0,0383 em 10 s). A cena `=88`
   é o par 40°/50°, então *"um pouco mais rápido"* é exactamente o quanto a
   rampa é mais íngreme — **não há regressão a procurar**.
2. **A HIPÓTESE QUE MOTIVOU A VARREDURA MORREU NELA.** Eu suspeitava que o
   `0,0000` do teto fosse um **cruzamento** calibrado a 30°: o resíduo tem dois
   termos (`D ∝ g·sen θ` e a queda da mola `∝ k·erro·sen θ`, com o `erro` função
   da carga NORMAL, isto é de `cos θ`), e se o segundo carregasse um `cos θ` que
   o primeiro não tem, o cancelamento seria função da rampa. **É zero exacto de
   20° a 45°** — o teto é um piso de verdade.
3. **Os 587 m a 50° não são bug:** é o limite de rampa autorado (45°) a
   funcionar. A cena `=88` documenta *"40° sobe / 50° escorrega"*.

E as outras tres colunas colapsam numa lei unica, que reproduz os 24 valores ao
quarto decimal:

```text
  deriva(10 s) = 0,153 · sen θ · (1 − d)   metros
```

### A decisao, e por que ela inverteu de lado

Enquanto o teto custava **metade do peso** do personagem (§7b, tabela antiga), o
default tinha de ficar no meio e conviver com a deriva. A wave `gravity_hold`
mudou a coluna do preço — **95% → 91%, quatro pontos** —, e aí a conta inverte:

| | manter `0,50` | subir para `1,00` |
|---|---|---|
| deriva parado (30°, 10 s) | 0,0383 m | **0,0000 m** |
| quique do pouso | 24 mm | 0 mm |
| peso transmitido | 95% | **91%** |

⚠️ **O quique de verdade vive no FUNDO do knob, não aqui:** ele vale 199 mm em
`0,25`, 24 mm em `0,50` e **já é zero em `0,75`**. Trocar 24 mm de quique por um
personagem que não anda sozinho é o lado certo — e o knob do painel devolve o
quique a quem o quiser.

⇒ **`RideConfig::STARTING_POINT.spring_damping` = `MAX_DAMPING`.**

### O gate que MUDOU DE LADO com a decisão

O `the_shipped_default_leaves_a_measured_residue` existia para pinar um resíduo
*nomeado* dos dois lados. Com o default no teto ele não pode continuar a afirmar
isso — mas **apagar o conhecimento seria pior**, porque o zero de cima passaria a
poder ser satisfeito por vácuo (uma lei que congelasse o personagem, ou alguém
que "simplificasse" a deriva removendo o knob).

Ele virou `the_shipped_default_leaves_nothing_and_lowering_the_knob_is_what_costs`,
com **duas metades que medem coisas diferentes**: o default não viaja **E** com o
knob em meio curso o resíduo VOLTA na ordem que a lei prevê. É a segunda que
prova que o zero foi **comprado** pelo amortecimento.

⚠️ **A mutação é reverter o default**, e ela sangra **os dois** — o gate de
DECISÃO na `ph2d-platformer` (que agora compara contra `MAX_DAMPING`, não contra
um literal) e o de COMPORTAMENTO na `ph2d-physics-ecs`.

**c9:** `2278035e…` → **`74d4ea5d…`**, 108 corpos, debug ≡ release. Ele **tem**
de se mover: a altura de repouso do player é função do amortecimento.


---

## Bug #8 — Os players pulavam e andavam sozinhos, e a física estava IMÓVEL

**Sintoma** (Enio, com foto, cena `=112`): *"os players ficam dando pulinhos
discretos sozinhos (sem input)"* — e, na segunda rodada, *"o players pular e se
movem sozinhos"*.

### A física foi EXONERADA por MEDIÇÃO, não por argumento

Cinco sondas headless, todas negativas, e a decisiva rodou **pelo binário do
produto**: pose **bit-constante ao longo de 1190 tiques**, com a cadência real do
app (`time += 1.0/144.0; t = (time * 60.0).round()`), e o relógio **nunca** andou
para trás. ⚠️ **O resultado honesto de uma sonda é às vezes *não existe bug
aqui*** — e é ele que move a caçada para o sistema seguinte em vez de a deixar a
afinar constantes de mola.

⚠️ **DUAS hipóteses minhas morreram antes de virar código** (a oscilação da mola
da perna; a cadência de tiques do produto), as duas por medição.

### A causa: um ACORDE era lido como entrada de jogo

`player_keys.key()` observava a tecla **FÍSICA**, sem guarda de modificador.
Então:

| acorde | o que o player fazia |
|---|---|
| `Ctrl+Z` | **pulava** |
| `Ctrl+A` | andava para a esquerda |
| `Ctrl+D` | andava para a direita |
| `Ctrl+S` | agachava |

⇒ desfazer, salvar ou selecionar tudo **dirigia o personagem**. Os *"pulinhos
discretos"* eram os Ctrl+Z do artista.

⚠️ **E o doc do `player_input.rs` afirmava o oposto como impossível.** A frase é
que estava errada, não o report — e foi ela que fez a suspeita cair na física.

### A cura, e a assimetria que ela precisa

Um acorde **não arma**; a **SOLTURA passa sempre**:

```rust
let chord = mods.control_key() || mods.alt_key() || mods.super_key();
if !pressed || !chord { self.player_keys.key(code, pressed); }
```

⚠️ **Uma guarda simétrica trocaria um pulo espúrio por um personagem que ANDA
PARA SEMPRE** — tecla premida nua e solta com o Ctrl em baixo, e nada mais volta
a desarmá-la. *A metade que não pode ser esquecida é a que desliga.*

### Lições

1. **Um relato de *"a física está a agir sozinha"* começa por MEDIR a física.**
   Ela estava imóvel, e provar isso é o que dá licença para procurar noutro
   sistema.
2. **Um doc que declara algo impossível é uma pista, não uma prova** — este
   declarava exactamente o defeito como inalcançável.
3. **Um gate que pina TEXTO verbatim reprova no primeiro reflow.** O antigo
   cravava três linhas; passou a afirmar a **propriedade** (o bloco de observação
   não retorna cedo) e ganhou um irmão que exige as três perguntas de modificador.

---

## Bug #9 — O leque de sensores driftava do corpo, e a geometria estava certa

**Sintoma** (Enio, com foto, logo após o smoke dos sensores): *"temos um drift
dos gizmos dos sensores do platformer ao se mover o Player"*.

### O número veio ANTES da hipótese

Sonda `measure_probe_lag`, pela porta do produto: o leque fica **0,1000 m atrás
em regime** — a distância **exacta** que o personagem percorre num tique a
6 m/s —, constante e para sempre. **Parado é zero**, e é por isso que nenhuma
cena de repouso o mostrava; num arranque a 18 m/s são 0,3 m.

### A causa não é geometria, é QUANDO

A leitura é gravada **antes** do `step` — tem de ser, é ela que a lei consome
para decidir o tique — e o `readback` publica a pose **depois**. O overlay
desenhava a cápsula no fim do tique e o leque no começo dele.

⚠️ **É a mesma classe do defeito de ORDEM que o `readback` já documenta** (o
filho convertido contra a pose que o mesmo passe vai substituir): *duas coisas
publicadas de estados do mundo diferentes*.

### A cura é a ÂNCORA, não um re-cast

Re-perguntar ao mundo depois do passo seria a **segunda resposta** a *"o que este
sensor viu neste tique?"*, e ela discordaria da que a lei usou — a única que
produziu movimento. O leque é **rígido** em relação ao corpo, então o `readback`
o desloca pelo que o corpo andou, mantendo `hit`, `reach` e `skin` exactamente
como foram medidos. **0,1000 → 0,0000 m.**

⚠️ **O `Sweep` não é tocado, e a assimetria é a PROVA do diagnóstico:** ele já
viaja como `(corpo, deslocamento)` e o overlay o resolve contra a pose viva — por
isso ele **nunca** driftou. Deslocá-lo somaria o movimento a um número que já é
relativo a ele.

### Lições

1. **Um desenho que "erra a posição" pode não ter erro de posição nenhum** — a
   pergunta seguinte é *de que INSTANTE cada metade fala?*.
2. **O oráculo é a RELAÇÃO, não a coordenada.** O gate afirma que a distância do
   leque ao corpo não muda por o corpo estar em movimento; um gate que cravasse a
   origem em mundo teria de saber onde a perna nasce.
3. **Parado é o CONTROLE**, e cada amostra em movimento tem de afirmar primeiro
   que o corpo ANDOU — uma fixture parada passa sem conter o fenômeno.
4. **Um corte de LOC pode reprovar um arch-gate sobre produto CORRETO.** Dois
   gates ancorados no ENDEREÇO `player.rs` caíram quando duas funções se mudaram
   para o irmão; a cura é ler a **família** por uma porta única — *afirme a
   PROPRIEDADE, nunca o endereço*.

---

## Bug #10 — Na 2.ª vida o herói andava e rodava sozinho, e nada o empurrava

**Sintoma** (Enio, 05/10, a arena `PH2D_VIDA_SMOKE=4`): *«o player rotaciona e se move solinho algumas
vezes»*.

### O que foi EXONERADO por medição

A arena corrida pelo laço INTEIRO do jogo (relógios, fábricas, tabela de acções, a ponte, mortes e o
recomeço — o arnês `Jogo` dos gates dela), `2 400` quadros sem tecla nenhuma, com `8` mordidas de morcego e
um recomeço: o herói **nunca** mudou de sítio nem de rumo. Os morcegos e as balas têm empurrão `0`. ⇒ não era
um empurrão, nem a física: era algo que só o app tem — e o arnês não tinha uma coisa que o app tem: a FITA,
com o dedo de uma vida anterior.

### A causa: a fita regravava só o tique ALVO

`ph2d_app_physics::bridge::dispatch` gravava `tape.record(target, input)` — UM tique por quadro — e a ponte
joga a fita em TODO tique devido (`take_taped_input`). Na 1.ª corrida isso é inofensivo: a fita estende, e o
`record` preenche o vão com a última entrada. Mas o recomeço volta o relógio a zero e a corrida continua: a
2.ª vida é gravada **por cima** da 1.ª, e um quadro que deve DOIS tiques (um quadro mais lento) regravava só o
alvo — o tique do meio ficava com o dedo da vida anterior, e a ponte jogava-o. ⇒ um passo da vida anterior, e,
com `RotationMode::ToMovement`, o herói a virar-se para ele. Esporádico porque só morde os quadros que devem
mais de um tique; o mesmo vale para um loop ou um scrub para trás seguido de play.

⚠️ **E o comentário ao lado afirmava a cura que não existia:** *«o `record` preenche o vão com a última
entrada exatamente por isso»* — verdade só quando a fita CRESCE. O comentário era parte do bug (corrigido com
ele).

### A cura

Gravar o dedo de agora em TODO tique devido (`last_stepped + 1 ..= target`) — é o que o dispatch aplica a
todos eles (segurar uma tecla), e a fita passa a dizer o mesmo.

### Gate que fecha este bug

`tape_second_life_tests::na_segunda_vida_sem_tecla_o_heroi_nao_anda_nem_roda` (`ph2d-app-physics`, pela porta
do produto): a 1.ª vida anda 2 s, o relógio volta ao zero como no recomeço, e a 2.ª vida, sem tecla e com um
quadro em cada três a dever dois tiques, fica PARADA. Vermelho antes da cura (`(0,057; 0,034) m` e `0,21 rad`
num tique); CONTROLOS: um tique por quadro, e a fita esquecida no recomeço.

### Lições

1. **Irmão do Bug #8** (os players que andavam sozinhos por um acorde lido como tecla): *«anda sozinho» é,
   duas vezes em dois, uma ENTRADA que ninguém deu* — procure quem escreve o dedo antes de suspeitar da física.
2. **Uma estrutura que se REGRAVA tem de ser escrita por inteiro**: o que é verdade ao estender (o vão
   preenchido) não é ao sobrescrever. O gate tem de ter as DUAS corridas.
3. **O arnês sem a fita não podia ver o defeito** — *um arnês que não corre o que o quadro corre não afirma
   nada sobre o quadro*; a sonda negativa foi o que mandou procurar o que faltava nele.

---

## Bug #11 — O wall jump com altura ~1,9–2,0 atira o controle ao topo da parede — ✅ FECHADO (12/11)

Achado em 12/11 por `f3a92736ab` (a lei do pulo passou a pôr o ápice real = Jump Height); curado na linha GEF em `937f4ce952`.

**Sintoma:** `platform_wall`, `wall_jump_rise(0.0)` com Jump Height 2,0: o pulo de parede sai da fresta
(y=0,988, t=86), afasta-se, volta à parede a 6 m/s perto do ápice (t=132, y=3,048) e no tique seguinte sobe a
**27,02 m/s** até o topo da parede (y≈13,1 → pousa em 12,9; subida ≈ 12 m), em vez de ~2,06 m. A fixtura
contornava com `OLD_LAW_WALL_JUMP_HEIGHT = 2.1056`.

**Mecanismo (medido tique a tique):** não é o wall jump, nem buffer, nem coyote, nem o rapier.
1. Em t=132 o corpo chega à parede a 6 m/s e penetra **2 cm** (x=0,570; face 0,75; raio 0,2).
2. O pé de FORA da perna (leque `ride.spread` 0,9 × meia-largura 0,2 = 0,18 m) nasce em x=0,750 — DENTRO
   da parede. O raio é sólido (`cast_ray_for_body`, `ph2d-physics/src/world/cast.rs`), logo devolve
   `distance = 0` e normal nula.
3. `cast_leg` (`ph2d-physics-ecs/src/bridge/player_leg.rs`) aceitava-o: a lei do degrau só corre quando o pé
   do MEIO achou chão, e aqui ele não acha nada (o chão está 23 m abaixo).
4. `footing_verdict` (`ph2d-platformer/src/slope.rs`) lê a normal nula como chão plano (é a lei de propósito,
   para o pé do meio afundado numa prancha) → `Landed` a 0 m → a mola fecha os 0,9 m de `float_height` num
   tique: vy = 27,02 m/s (t=133), depois `Bonked` e voo balístico até o topo.

**Porquê «dependia de v0»:** o chão fantasma acende em TODAS as alturas (a 2,1056 também: `Landed` em t=133,
raio = 0,0). O que muda é a velocidade com que ele chega: a 2,1056 chega ainda a SUBIR (vy 0,13), e o pulo
mantém a mola calada; a 2,0 chega no ápice (0,04), e a mola actua. **Janela medida** (varredura de 0,05 com o
defeito reposto): parede sólida reprova de **1,50 a 2,05 m** (subida 11,9–12,1 m, vy 27,02), passa a 2,10+.
Os pulos do chão contra a parede (a 1,0 e a 1,6 m dela) nunca entram na janela.

**Cura (na causa, sem afrouxar nada):** em `cast_leg`, um pé de FORA nascido dentro da matéria
(`distance <= 0` ou normal nula/não finita, porta única `ph2d_platformer::slope::normal_is_known`) não entra
na redução (o overlay continua a vê-lo). O pé do MEIO mantém a lei: afundado numa prancha, a mola tira-o.
⛔ **Recusado, MEDIDO:** ler a normal nula como «sem informação» (`Airborne`) no `footing_verdict`, ou
recusá-la também no pé do meio — os dois reprovam `platform_drop::the_platform_is_solid_again_once_he_is_through`
e `platform_drop_ladder::a_wide_gap_ladder_retires_the_drop_as_it_should` (o corpo a atravessar a prancha
deixa de ser empurrado para fora).

**O gate que estava verde:** `the_wall_jump_survives_a_gap_the_waist_falls_into` mede UM ponto (Jump Height
2,0 da lei velha = `v0` 6,264). Com a lei nova esse ponto mudou de trajectória e caiu dentro da janela; a
fixtura passou para 2,1056 — 0,05 m acima da borda — e voltou a ficar verde sobre o defeito. Um gate de
um ponto não vê uma janela.

**Gate novo:** `platform_wall::no_jump_leaves_faster_than_its_launch_or_above_its_apex` — Jump Height
1,50–2,50 (passo 0,05), pulo da parede e do chão (a 1,0 e 1,6 m dela): vy ≤ `launch_speed` + um tique de g
+ 0,05, subida ≤ altura + 0,25 e ≥ 0,6 × altura. A fixtura voltou a 2,0 e `OLD_LAW_WALL_JUMP_HEIGHT` saiu.
**Mutação:** tirar a recusa do `cast_leg` → a varredura reprova (h=1,50…2,05, 27,02 m/s) e
`the_wall_jump_survives_…` também; restaurado, verde.

## Bug #12 — Teto baixo batido sem `ceiling` nem `Bonked` nos primeiros ~0,3 m da subida — ✅ FECHADO (12/11)

Achado em 12/11 pela varredura da linha GEF (sondas sobre `tests/it/player_ceiling.rs`).

**Sintoma:** em TODAS as alturas de pulo (1,0–2,6), um teto cujo fundo fica 0,10–0,30 m acima da cabeça
(cabeça parada = 1,40; fundo 1,50–1,70) é batido — a subida morre contra ele — e a vista não acende
`ceiling` nem publica `Bonked` (nem `player.bonked` no barramento).

**Mecanismo:** nos primeiros ~0,3 m da subida a perna (`cling_distance`) ainda vê o chão, logo `stand` é
`Some`. A ponte (`ph2d-physics-ecs/src/bridge/player.rs`) passava `stand.is_some()` como «no chão» a
`ceiling_fact_wanted` e a `corner_probe_wanted`, e a lei (`ph2d-platformer/src/lib.rs`, a quina) passava
`footing.is_some()`: as duas portas diziam «de pé sob uma marquise», e o sensor do teto não corria. A lei
já tinha a resposta certa: `JumpState::on_ground` (sensor E não `airborne`), que separa *«estou de pé»* de
*«acabei de sair daqui»* — e `lib.rs` já a calculava (`let grounded`) para o pulo, o arranque e a beirada.

**Cura:** a ponte usa `was.jump.on_ground(stand)` nas duas portas; a lei usa o `grounded` que já tinha na
porta da quina. `bridge/player.rs` ficou nos 700 (tecto) — a linha nova saiu de um comentário.
Efeito colateral medido: a ajuda de quina passa a agir também nesses tetos baixos; os três testes de
`platform_corner` ficam verdes com e sem a cura (nenhum mudou).

**O gate que estava verde:** os gates de `player_ceiling` mediam UM teto (1,9, com 1,3146 m e o pulo
solto no tique 8, topo livre 2,17) — muito acima da janela —, e o «esvanecer» uma tabela de tetos fixos
presa à lei velha.

**Gate novo:** `player_ceiling::every_ceiling_in_reach_is_bonked_once_at_every_jump_height` — Jump Height
1,0–2,6 (passo 0,05) × fundo do teto de cabeça + 0,10 até o topo livre (passo 0,05), quina 0,12 e 0:
exactamente UM `Bonked` e UMA borda do bit; nada a topo livre + 0,05 e + 0,10; vy ≤ `launch_speed`;
velocidade publicada nunca cresce com o teto. A fixtura `jumper` voltou a 1,2 m, o pulo é segurado até
depois do ápice (t ≤ 60; soltar no tique 8 cortava a subida), e o topo livre mede-se = cabeça + H (2,60 a
1,2). `the_bonk_fades_…` passou a tetos RELATIVOS ao topo livre (−0,60: 3,123 m/s · −0,01: 0,349 · +0,01:
nada). Verde também sobre `62c9ff8e9c` (tique dividido na faixa).
**Mutação:** repor `stand.is_some()` na ponte e `footing.is_some()` na lei → a varredura reprova com 266
falhas (tetos 1,50–1,70); restaurado, verde.

### Bug #12b — teto DENTRO do tique da decolagem (cabeça + ≤ ~0,05) — ✅ FECHADO (12/11)

**Sintoma:** fundo do teto a cabeça + 0,05 (1,45): o pulo bate e não acende `ceiling` nem publica
`Bonked`, nas 33 alturas, com a quina armada ou não.

**Mecanismo (medido tique a tique, h=1,2):** no tique do pulo a ponte sonda o teto ANTES de a lei lançar
(`rel_up` = −0,082, logo `ceiling_fact_wanted` = falso); o solver leva a cabeça a 1,4554 (contacto) no
MESMO tique; no seguinte `rel_up` já é −0,031 — a subida morreu sem ninguém perguntar. O facto só era
medido antes do passo, com a velocidade do tique anterior.

**Cura (o `is_on_ceiling` do Godot / os contactos do `CharacterBody`):** `bridge/ceiling_fact.rs`. O
`apply_player_results` arma, depois dos motores, a subida PRETENDIDA de cada player (a velocidade do
corpo com o lançamento aplicado), o bit da sonda deste tique e o da vista anterior. Depois do `step`
(`depois_do_passo`, os dois laços) `measure_player_ceilings` procura em `tick_contacts` (união dos
sub-passos) um par do corpo com `impact > 0` e normal (para o corpo) `n·up ≤ −cos(walk.max_slope_deg)`;
com subida pretendida > 0 é batida: acende o bit na vista e publica `Bonked { speed: subida pretendida }`.
**Sem duplicar:** nada se a sonda deste tique já acendeu; sem evento se a vista anterior estava acesa
(contacto contínuo); o `events_between` do tique seguinte vê `before.ceiling` aceso e não repete. Sob
Snap (cinemático, sem contacto no solver) o facto é o do movimento: subida pedida, cortada, contra uma
normal de teto (`player_kinmove`). A sonda de antes do passo fica: é ela que decide a ajuda de quina.

**Gates:** `player_ceiling::every_ceiling_in_reach_is_bonked_once_at_every_jump_height` passa a começar
em cabeça + 0,02 e + 0,05 (depois + 0,10 em passos de 0,05) — exactamente uma batida e uma borda do bit
em toda a grelha; `a_ceiling_inside_the_takeoff_tick_is_bonked_with_the_launch_speed` (teto a + 0,05,
quina 0,12 e 0: um `Bonked`, no tique do `Jumped` ou no seguinte, a ±25 % do `launch_speed`).
`standing_under_an_overhang_is_not_a_ceiling` e os gates do Bug #12 continuam verdes.

**Mutação:** desligar a medição pós-passo (`if true || !hit`) → a varredura e o teste do arranque
reprovam (`[]`); publicar por cima da sonda (sem a guarda `lit_now`) → a varredura reprova («exactamente
uma»); sem a guarda da vista anterior → a varredura reprova; restaurado, verde. O censo
`no_std_transcendental_reaches_the_deterministic_hash` apanhou um `f32::cos` na primeira versão (agora
`libm::cosf`). **Gate Snap (12/11):** `under_snap_every_ceiling_in_reach_is_bonked_once` (H 1,0/1,2/1,8/2,6 × quina 0,12 e 0, teto a cabeça + 0,02, + 0,05 e + 0,10..topo livre: um `Bonked` e uma borda do bit; nada a livre + 0,10; marquise sob Snap não bate) — desligar o corte em `player_kinmove` reprova 26 casos. `Bonked.speed` é UM contrato: a subida PRETENDIDA do tique do choque nos dois caminhos (a ponte recarimba o da sonda em `arm_ceiling_fact`); gate `the_bonk_speed_is_the_intended_rise_on_both_paths` (sem o recarimbo, Snap publica 3,286 contra 3,123). Nenhum consumidor de produção lê `speed` (o sinal `player.bonked` só leva o nome).

### Bug #13 — o player Snap não assenta onde a lei o vê de pé, e o nascido fora da perna cai 16 tiques — ✅ FECHADO (13/11)

**Sintoma:** sob Snap (cinemático), H = 1,0 premido no tique 20 subia 0,357 m; nascido a 0,9 (fora da
perna Snap 0,75, dentro da autorada `float_height + cling`) só assentava no tique 16.

**Mecanismo (três camadas, medidas):** (1) na faixa entre a perna da lei (`footing`) e o toque do
controlador o corpo caía em queda livre com a lei a vê-lo de pé — o pulo somava `v0` a uma velocidade
que ainda descia; (2) `world_steps` = 16 sob Snap fazia o `launch_speed` modelar outro integrador
(ápice 0,956 a H = 1,0); (3) **o índice das consultas (BVH do broad phase do rapier) só é actualizado
dentro do `step`**: no tique 1 o encaixe do nascimento pergunta pelo chão ANTES do primeiro passo e o
cast não acha nada (a broad phase está vazia) — o corpo caía até entrar na faixa da perna no tique 16.

**Cura:** `world_steps` = 1 sob Snap; `KinematicState::stood` (a lei de pé no tique anterior) separa o
POUSO da elevação do degrau; `kin_in` e o encaixe (`bridge/player_seat.rs`, o `apply_floor_snap` do
Godot) só no tique do pouso/nascimento; `seat::rest` cancela a velocidade contra o chão que o encaixe
achou e a lei não via. Para (3): `world/query_index.rs` — flag `query_index_stale` (marcada por inserir,
pose, reshape, teleporte; limpa no `step`; viaja no checkpoint) e `PhysicsWorld::refresh_query_index`
(pose de cada collider = corpo · pose no corpo, e `BroadPhaseBvh::set_aabb`, em ordem de arena; um
`bool` quando nada mudou). Chamado SÓ à cabeça de `drive_controllers` e só se há controladores — a cena
sem controlador fica byte-idêntica (mexer no BVH antes do passo muda a ordem dos pares). É a forma do
Box2D (a forma entra na árvore ao ser criada; `b2Body_SetTransform` move a folha).

**O vermelho que enganava — `the_ground_snap_is_load_bearing_again`:** com o refresh o salto da descida
da rampa de 25° caiu a 0 em QUALQUER `cling` (0,02: 0,4558 → 0; 0,04: 0,0380 → 0). Medido tique a tique
(lado a lado, mesmo binário, refresh ligado × desligado por variável): não há estado que persista — o
`born`/`seat` age só no tique 1, `stood` = verdadeiro do tique 2 em diante, `rest` só no tique 1, e a
velocidade horizontal à entrada da rampa é a mesma (1,359 / 2,626 m/s nos tiques 181–182). A diferença é
o REPOUSO: com refresh o tique 1 encaixa 0,191 m e o corpo repousa EM CONTACTO (centro a 0,4819 m da
superfície, perpendicular = 0,2 + 0,3·cos 25° + pele); sem refresh ele caía e parava a **0,5275**, 4,6 cm
acima do contacto (o repouso cinemático não reproduzível de 1,0–4,5 cm do item 5 da §7), com
`got.grounded` = falso no primeiro tique da caminhada. O «salto» era o `cling` a não alcançar esse vão
— a velha tabela (< 0,10 salta, ≥ 0,10 cola) é exactamente o vão. A 25° a partir do contacto o tique da
descida abre ~5 mm e nenhum `cling` salta; o knob morde onde o tique abre mais que ele:

```text
  rampa \ cling  0.02   0.04   0.06   0.08   0.10 .. 0.50     (arranque em contacto, 6 m/s)
  25°            0      0      0      0      0
  35°            0.760  0.004  0.004  0      0
  40°            0.732  0      0      0      0
  44°            0.803  0      0      0      0
  (queda, sem refresh, 25°: 0.456  0.038  0  0  0)
```

O gate passou a medir a 35° (`DESCENT_DEG`: 0,02 salta 0,760, 0,25 cola) e acrescenta o braço da
chegada: a 25° com `cling` 0,02, nascido assente, salto < 0,05. As hipóteses (i) `born` mais de um
tique, (ii) `stood` não escrito, (iv) velocidade de entrada diferente CAÍRAM; (iii) o `snap_to_ground`
do rapier vê o chão desde o contacto — é a geometria do repouso, não um regime preso.

**Gates:** `cast::tests::a_collider_inserted_before_any_step_is_seen_after_the_refresh` (raio, varredura
e teleporte antes do 1.º passo); `player_snap_settle` (3; nascido a 0,9 assenta em ≤ 2 tiques);
`measure_snap_and_step::the_ground_snap_is_load_bearing_again` (35° + braço dos 25°).

**Mutações (13/11, 8 ficheiros de teste da ponte, 41 testes):**

| mutação | resultado |
|---|---|
| M1 `world_steps` do mundo sob Snap | ✗ `the_first_jump_on_the_ground_rises_the_jump_height` (H = 1: 0,9559) |
| M2 `kin_in` desligado | ✗ (13/11, 2.ª ronda) `a_buffered_jump_fired_on_the_landing_tick_rises_the_jump_height` — o pulo BUFERIZADO que dispara no tique do pouso é engolido (largado de 2 m: sobe −0,146; de 4 m: −0,216; com `kin_in`: 1,0000 a partir da pose de decolagem). Na 1.ª ronda sobrevivia porque nenhum gate disparava o pulo NO tique do pouso |
| M3 encaixe da banda desligado | ✗ `the_kinematic_landing_does_not_slide_along_the_ramp_normal`, `a_snap_player_settles_where_it_lands` |
| M4 encaixe do nascimento desligado | ✗ `the_ground_snap_is_load_bearing_again`, `a_player_born_in_reach_of_its_leg_settles`, `a_snap_player_settles_where_it_lands` |
| M5 `stood` ignorado | ✗ `the_step_it_climbs_is_the_number_the_artist_authored` |
| M6 refresh das consultas desligado | ✗ `the_ground_snap_is_load_bearing_again`, `a_snap_player_settles_where_it_lands` |
| M7 `rest` desligado | ✗ `the_ground_snap_is_load_bearing_again`, `the_first_jump_on_the_ground_rises_the_jump_height` (H = 1: 0,9657) |

**Cenas (13/11, por nome):** `ph2d-physics-smoke-scenes` (`kin_`, `swim`, `blast`, `zone_force`) 23/24 —
⚠️ `physics_smoke_kin_water::…::the_scene_delivers_the_numbers_its_message_prints` reprova (Dynamic Player
afunda 1,1712 contra 1,0855 prometido) **já em `eb729a2253`, sem este diff** (e com o refresh desligado):
o player DINÂMICO, fora do Snap — aberto, por bissectar entre `62c9ff8e9c` e `eb729a2253`.
`ph2d-host-game-scenes` 80/80 · `ph2d-physics-scene` 16/16 · `ph2d-app-game` 1/1.

**2.ª ronda (13/11) — as três pontas:**

1. **Refresh INCREMENTAL.** A flag `query_index_stale` passou a ser `query_index_dirty: Vec<ColliderHandle>`
   (o `modified_colliders` do rapier): inserir, `retune_part`, reshape empurram o collider; `set_body_pose`,
   `nudge_body` e o empurrão do reshape empurram os colliders do corpo. O refresh ordena pelo índice da arena
   (`into_raw_parts`, determinista), tira repetições e só trata esses; o `step` limpa; o checkpoint leva o
   vetor. Um collider não marcado já está na sua caixa gorda (o `set_aabb` dele seria `Unchanged`), então o
   mundo é o mesmo da passada completa. Antes (sonda `query_index::tests::measure_the_refresh_cost`, profile
   smoke, 1 corpo movido): 100 colliders 1,8 µs · 1000 17,1 · 5000 91,7 (passo 97,8). Gate novo
   `cast::tests::a_platform_moved_by_set_body_pose_is_seen_where_it_is_now` (plataforma movida a cada tique
   entre 50 colliders parados, raio antes do passo; o checkpoint leva as marcas).
2. **`kin_in` NÃO é morto** — é o pulo buferizado no tique do pouso (M2 acima). Fica, com gate.
3. **A água (`physics_smoke_kin_water`) — culpado `62c9ff8e9c`** (o tique dividido no limiar de faixa).
   Bissecção: `50f6a8f6fa` verde; `62c9ff8e9c`, `a9430052cd`, `e13112fe92`, `a7ea182ff4`, `eb729a2253`
   vermelhos com o MESMO 1,1712. Mecanismo, medido no mesmo binário em `62c9ff8e9c` com
   `tick_gravity_scale` a devolver a faixa do INÍCIO do tique por variável: volta a 1,0855 / 1,0860 / bob
   1,4357 / 1,4394 exactos. Não é a água (a lei cala `waterborne`): o player SAI da água a cada bobeio (até
   ~1,5 m) e no ar a gravidade por faixa age — a média do tique dividido muda a fase do arco. O número da
   cena é uma amostra única em t = 4 s de uma oscilação de 1,4 m, por isso 8 cm. **A física nova está certa**
   (é a lei contínua por trechos); a cena foi re-medida: `SANK` 1,1070 / 1,1712 / 1,0079, `BOB` 1,4803 /
   1,4123, e a frase «concordam na quarta decimal» (acaso de fase da faixa antiga) passa a ser a diferença
   calculada dos `BOB` (~5 %). O cinemático também reprovava (1,0079 contra 1,0860), escondido atrás do
   `assert` do dinâmico.
4. **A prateleira alta (`physics_smoke_multi_jump`, cena 110) — culpado `f3a92736ab`** (o ápice real É a
   Jump Height). Bissecção: `4f46a3573c` verde (centro final 3,10); `f3a92736ab`, `48afab6f40`, `eb83c15600`,
   `62c9ff8e9c`, `eb729a2253`, `f820043427` vermelhos (2,70–2,71). Mecanismo: o v0 velho subia ~2,1 m para
   2,0 pedidos; com o v0 exacto o corpo bate mais baixo na parede da alta. Mas a cena **nunca pousou**: o
   gesto do gate começava encostado na parede da BAIXA, e o `assert` comparava o CENTRO com o topo — na
   base o corpo escorregava pela parede com os pés 0,4 m abaixo do topo. Além disso, com 3,5 m de pista, um
   pulo SIMPLES da beira da baixa chegava à alta. **A física nova está certa**; a cena foi refeita pela lei:
   baixa 1,0 m, alta 3,6 m (acima de um pulo dado da baixa, 3,40; abaixo de dois do chão, 4,40), pista 6 m
   (`HIGH_X0` 10), `FEET_GAP = FLOAT − 0,5`; gates pelos PÉS e parados, de três pontos da pista, e o novo
   `a_single_jump_from_the_low_shelf_does_not_reach_the_high_one`.

### Bug #14 — pular encostado na parede, com a seta contra ela, perde ~63 % do pulo — ✅ FECHADO (13/11)

**Sintoma.** Cena 110 (player dinâmico, Jump Height 2,0, segurado): parado sobe 1,969; encostado com
`drive = 1` sobe **0,726** (0,766 já em `4f46a3573c`).

**Mecanismo (medido, fixture `wall_jump_tests.rs`, sobe do repouso):** atrito de Coulomb da cápsula
contra a face, com a normal que o andar (seta premida) sustenta. Player μ 0,5 / parede μ 0,5
`Average`: **0,774**; player μ 0 sozinho **1,105** (a `Average` deixa 0,25); parede μ 0 sozinha
**1,105**; os dois 0 **2,000**; player 0 + `Min` **2,000**; o Snap (cinemático) já subia **2,000**. O chão
não depende do atrito: o Dynamic flutua na mola (a cápsula não toca o chão) e a correia, o lift, o
andar e a travagem são da lei (`ground_velocity`, walk/brake) — o andar no chão é idêntico com e
sem atrito (`platform_wall_sem_atrito`, 4,330 m × 4,330 m).

**Cura.** Estado da arte dos character controllers (Unity `CharacterController`, Godot
`CharacterBody2D`): o corpo do player não raspa em nada — `bridge::bodies::lei_da_forma_do_player`
impõe atrito 0 + regra `Min` à forma de todo corpo com `PlatformPlayer` (e às peças dele), na porta
única `desc_do_corpo` (o agachar herda pelo `rest`). O material autorado no Collider do player deixa
de valer; o Inspector deixou de mostrar Friction e Friction Combine na forma do player (corpo, filha e
peças, cartões e `PH2D_CARDS=0`), pela mesma pergunta da ponte (`ph2d_physics_ecs::forma_do_player`).

**One-Way na forma do próprio player** (irmão, 13/11): a auditoria da Surface
(`superficie_do_player_tests`) mediu que o `OneWayPlatform` na forma do player só fazia uma coisa — o
par de contacto com o chão deixava-o atravessar tudo (y final −16,671 × 0,900). Feature pior que não
existir: a mesma `lei_da_forma_do_player` dá `one_way: false` (corpo, filha, peças), o Inspector
esconde a linha One-Way (e o resumo da dobra Surface) pela mesma `forma_do_player`
(`InspectorPhysicsInfo::player_shape`), e o contorno do Play deixa de a tingir de uma via. Num corpo
que não é player nada muda. Gates: `a_superficie_na_forma_do_player_sozinho_nenhum_decide` (bit a
bit igual sem o componente) e `a_forma_do_player_nao_mostra_atrito` (One-Way ausente com player,
clique real sem).

**O que o atrito escondia, curado pela lei (não reposto):** o freio da parede (`wall_slide`)
somava-se ao boost do pulo de parede no tique da saída — dois donos do eixo — e o pulo saía 0,24 m/s
acima do arranque (`no_jump_leaves_faster_than_its_launch_or_above_its_apex`, vermelho 21/21 com o
atrito a 0). Cura: o freio cala-se com `jumped` (`ph2d-platformer` `player_motor`).

**Gates que estavam VERDES sobre o defeito, e porquê:** os da parede tinham o atrito como CONTROLO
(`the_slide_descends_at_the_authored_speed` pedia a «cola» < 0,3 m; `platform_wall_sem_atrito` pedia
que o padrão colasse); a cena 111 (`physics_smoke_ledge`) foi calibrada sobre o pulo encostado comido
(lábio 2,4 m para um ápice de 2,145) e a 110 pedia o pulo na pista. Re-derivados: controlo = queda
livre; lábio da 111 em **3,7 m** (topo no ápice 3,4 + `GRAB`/2), `REACH_ONE` = `jump_height` (sem
literal); a 110 ganha o gesto natural (`the_charged_one_lands_on_the_high_shelf_jumping_against_its_wall`).
`zero_reach_is_the_world_before_this_wave`: sem atrito a calota da cápsula escorrega ~1 cm da quina
do lábio e pára (geometria, não a lei); o gate passou a `< 2 cm` e parado.

**Gate novo:** `a_jump_against_a_wall_rises_the_full_jump_height` — alturas 1/1,5/2/2,5 × Dynamic/Snap
× paredes desligadas/slide/grab: encostado = livre ±1 mm. **Mutações:** tirar o
`lei_da_forma_do_player` de `desc_do_corpo` ⇒ o gate novo reprova; tirar o `&& !jumped` do freio ⇒
`no_jump_leaves_faster…` reprova; restauradas, verdes.
