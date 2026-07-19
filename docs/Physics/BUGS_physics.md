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
| [1](#bug-1--a-simulação-rodava-junto-com-a-animação-e-uma-nota-minha-dizia-que-o-interruptor-seria-o-desenho-errado) | **A sim rodava junto com a animação** — e uma nota minha dizia que o interruptor seria o desenho errado | `ph2d-timeline` (transporte) + `ph2d-physics-ecs` (ponte) | ✅ Resolvido (W4b; pendente smoke) | 2026-07-18 |

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

**Estado:** ✅ resolvido em 2026-07-18 (W4b, commit `25bf851e`; **pendente smoke**).
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
