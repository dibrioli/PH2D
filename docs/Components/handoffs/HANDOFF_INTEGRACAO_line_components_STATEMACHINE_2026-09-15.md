# HANDOFF DE INTEGRAÇÃO — `line/components` · TOP-20 **#15 `StateMachine`** · 2026-09-15

> Plano e medições: [`12_plano_state_machine.md`](../12_plano_state_machine.md).
> ⚠️ Leia o **§6** (o que o diff inverte) e o **§7** (as premissas que a medição derrubou) antes de
> ler o diff.

---

## §1 — O que o ARTISTA consegue fazer agora

Escrever o cérebro de um objecto **sem uma linha de script**: *«esta porta está `Fechada`; quando
ouvir `botao` vai para `A abrir`; quando ouvir `fim` vai para `Aberta`»* — e cada estado
**anuncia-se**, de modo que a tabela de acções do #5 faça o resto (mostrar, esconder, tocar um som,
arrancar um relógio). O painel diz **`Now: …`** enquanto a cena corre.

---

## §2 — Os contadores, como DELTA (⛔ nunca o literal)

| contador | delta | nota |
|---|---|---|
| `PROJECT_SCHEMA` | **+1** (132 → 133 nesta árvore) | degrau na escada **e** a tripla actualizada — *a tripla não vê este degrau, é a 17.ª vez* |
| registo do `ph2d-ecs` | **+1** (89 → 90) | `ph2d::ecs::StateMachine` |
| espelho `ph2d-render` | **+1** (90 → 91) | ele conta `ecs + render` |
| espelho `ph2d-script` | **+1** (90 → 91) | ele conta `ecs + script` |
| `LIVE_SECTIONS` | **+1** (25 → 26) | a secção do Inspector, no MESMO commit |
| `SignalOrigin` | **+1 variante** | `StateMachine { source }`, **append-only** |

⛔ **O `StateMachineRuntime` NÃO se regista**, e a cerca é o **TIPO**: ele não deriva `Serialize`,
logo a linha do registo **nem compila** (o precedente do `TimerRuntime`).

---

## §3 — ⛔⛔ O que esta wave ACHOU no substrato, e que não era dela (W0)

**Medido pela SUPERFÍCIE DA API, não por grep:** o molde da casa separa CONFIG (registado) de VIVO
(sem `Serialize`), e isso tem uma consequência que ninguém tinha escrito — **o que o undo não
fotografa, o undo também não REPÕE.** O `ph2d-ecs` exportava `advance`/`reconcile`/`start`/`stop`
para o timer e **nenhuma porta de reposição**.

⇒ **um `Timer` que correu continuava corrido depois de um Reset.** É a mesma família do report do
dono sobre o rewind dos projécteis, um nível acima.

⭐⭐⭐ **E o CENSO achou mais dois na primeira corrida dele:**

| vivo | o que sobrevivia a um Reset | o que isso fazia |
|---|---|---|
| `TimerRuntime` | `elapsed` e `running` | um relógio corrido ficava corrido |
| `FactoryRuntime` | `total` e **`rng`** | ⭐ uma fábrica com `Max Total` gasto **recusava-se a produzir na 2.ª corrida**, e uma fábrica **aleatória dava outra corrida a cada rebobinar** |
| `CameraRuntime` | o amortecimento e a velocidade suavizada | a câmera continuava de onde a corrida anterior a deixou |
| `LifetimeRuntime` | `elapsed` | hoje inerte (a varredura da fábrica já as despeja) — a redundância está **dita**, não suposta |

A cura é **UMA porta** com um «nascer» **próprio por membro** — e eles não são o mesmo:
`timer::born` por slot (um `autostart` nasce **a correr**; o `Default` é parado) · `Default` para a
vida e a fábrica (⭐ `rng: 0` = *por semear* ⇒ a corrida seguinte **repete** a primeira) · e
**APAGAR** o da câmera, para o `ensure_runtime` da shell o recriar **da pose autorada** (um
`Default` poria toda câmera na **origem**).

---

## §4 — O ORÁCULO (§0.9), e as sete leis que ele deu

**Triagem primeiro:** Godot 4.7.2, `/usr/bin/godot`, **MIT** — porta ABERTA. Corrido sem interface
por [`godot_statemachine_probe.gd`](../ferramentas/godot_statemachine_probe.gd).

1. prioridade: o número **menor** ganha · 2. empate: ganha a **última** declarada · 3. uma condição
num `advance_mode = ENABLED` é **inerte** · 4. no `AUTO`, transita no **mesmo** avanço · 5. uma
cadeia acíclica atravessa-se **inteira** num avanço · 6. um ciclo **não pendura** (um estado por
avanço) · 7. `A→A` é **recusada**.

⚠️ **E a sonda mentiu DUAS vezes antes de dizer a verdade, as duas por FIXTURA:** sem arranque
explícito as cinco perguntas liam `Start` (que se teria lido como *«o Godot não transita»*), e a
condição foi escrita no modo onde ela é **inerte**. *Uma fixtura que o oráculo recusa nunca é
comparada.*

---

## §5 — As provas

[`mutacao_statemachine_2026-09-15.sh`](../ferramentas/mutacao_statemachine_2026-09-15.sh) — **16
mutações, todas a sangrar** (as **duas** últimas são do report do §9-bis: o cérebro sem corpo, e a
faixa por cima do corpo), e **DUAS sobreviveram primeiro**:

1. ⚠️ **A fixtura da ORDEM era fraca:** com **duas** máquinas semeadas ao contrário, `reverse()`
   devolve **exactamente** a ordem certa — ela não separava *«ordenado»* de *«ao contrário da
   criação»*. Corrigida para **três** em ordem arbitrária. *Uma fixtura que não separa a lei da sua
   coincidência não testa a lei.*
2. ⚠️ **Uma mutação minha não mutava nada:** `has_exit: true && X` é `X`. *Uma prova de mutação que
   não muda o programa mede o compilador.*

---

## §6 — As SEIS leituras que o diff inverte

1. **O `rewind_runtime` não é «pôr a zero»** — cada membro tem o SEU nascer, e o da câmera é
   **apagar o componente**.
2. **A lei NÃO está numa crate-folha** — a família `Logic` inteira vive no `ph2d-ecs`, e uma crate
   nova seria o primeiro membro fora da casa dela (§7.1).
3. **O conjunto de visitados do oráculo NÃO foi portado** — o que ficou é o **consumo do sinal**, e
   a lei observável dele fica intacta (§7.2).
4. **Um `+ Add Transition` nasce a apontar para o ÚLTIMO estado**, nunca para o `0`: uma seta `0→0`
   é uma auto-transição, que a lei recusa — ela leria-se como um controlo partido ao nascer.
5. **A cena de smoke usa TRÊS placas empilhadas e não um objecto que muda de cor** — a tabela de
   acções sabe mostrar e esconder, e **não sabe pintar**. Inventar um verbo de cor só para o smoke
   seria medir um app que não existe. ⚠️ **E desde o §9-bis as placas são uma FAIXA sobre um CORPO**,
   que é quem carrega os componentes e quem o dedo apanha — as duas peças **encostam sem se
   sobrepor**, de propósito.
6. **O `montar` da cena não tem `match`** (uma cena só; o `clippy` recusa um braço único) — o que
   mantém o `CENAS` honesto é um **gate** que varre níveis, não a forma do código.

---

## §7 — As premissas DESTE plano que a medição derrubou

1. ⛔ **«W1: uma crate-folha `ph2d-statemachine`»** — refutado pelo precedente unânime da família.
2. ⛔⛔ **«a travessia pára no conjunto de VISITADOS»** — refutado por um **gate vermelho sobre o
   meu próprio desenho: uma porta `Aberta` ia a `Fechada` e logo a `A abrir` com UM toque.** A
   regra do oráculo é certa para uma entrada que é **NÍVEL**; a nossa é um **EVENTO**, e um evento
   é **gasto**. ⭐ Com o consumo, o orçamento de profundidade que o `SignalActions` prescreveu passa
   a ser um recurso **real** — quantos sinais soaram neste tique — em vez de um `MAX_DEPTH`.

---

## §8 — O SMOKE (o que o dono corre)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_STATEMACHINE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

⚠️ **O passo que escolhe o objecto é *«carregue no CORPO da porta da esquerda»* — o batente escuro
por baixo da faixa de cor**, e não na faixa: a faixa são as três placas que a máquina acende, e elas
não têm o componente. Ver o §9-bis.

---

## §9-bis — ⛔⛔⛔ O SMOKE do dono: *«não apareceu no painel a seção state machine»*

**Report de 2026-09-15, depois do fecho.** A fiação do painel estava **inteira** — o defeito era a
**CENA**, e ela estava certa como DADOS e era impossível como GESTO.

### O que era

A entidade que carregava o `StateMachine` chamava-se `Door` e **não tinha `Sprite` nenhum**. O que
se via no ecrã eram as três placas coloridas (`Left Closed`/`Opening`/`Open`), e **nenhuma delas tem
cérebro**. ⇒ um objecto sem sprite não emite `RenderInstance`, logo o
[`ph2d_render::pick_sprite_at_world`] **nunca o devolve**; o clique na porta escolhia uma placa, e o
Inspector mostrava — correctamente — a verdade sobre o objecto escolhido.

⚠️ **O roteiro impresso pela própria cena dizia *«Escolha a da esquerda»*, e esse passo era
impossível.** É o irmão exacto da lei
[`feedback_a_smoke_step_that_names_a_panel_row_must_prove_the_row_is_in_the_list`](../../../project-memory/feedback_a_smoke_step_that_names_a_panel_row_must_prove_the_row_is_in_the_list.md),
um nível acima: *um passo que manda CLICAR numa coisa afirma que ela é clicável.*

### ⛔ Porque nenhum dos seis gates da cena o via, e a cegueira é ESTRUTURAL

Os seis liam a cena como **dados** — *«a porta tem `StateMachine`?»*, *«as duas tabelas têm o mesmo
tamanho?»*, *«as três setas ouvem o nome que o relógio publica?»* — e **nenhum** perguntava o que o
roteiro promete: ***o artista consegue CHEGAR a ela?*** Uma cena de smoke tem duas metades, e esta
linha só gateava uma.

### A cura

Cada porta passa a ser um **CORPO** (o batente escuro, sempre visível, que carrega `StateMachine` +
`SignalActions`) com uma **FAIXA** de cor por cima — as três placas empilhadas, uma visível. O corpo
é a peça grande e é onde o dedo cai.

⛔⛔ **E elas ENCOSTAM sem se sobrepor, por causa do desempate:** o `pick_sprite_at_world` devolve
*«o último da ordem de iteração»*, que **entre arquétipos diferentes é indefinido por escrito** (o
corpo carrega `StateMachine`, a placa carrega `Visibility` ⇒ arquétipos diferentes). Sobrepostas, a
resposta dependeria da ordem em que o `bevy_ecs` visita os arquétipos; encostadas, não há desempate
nenhum a fazer. A geometria vive num módulo só (`porta::{corpo, faixa}`), com `JUNTA_Y` a ser ao
mesmo tempo o topo de um e o fundo do outro — *duas caixas escritas à mão seriam a segunda resposta
à mesma pergunta, e o ponto que o gate carrega deixaria de ser o ponto que o dono carrega.*

⚠️ **O CONTROLO ganhou corpo pelo mesmo motivo, e é isso que o torna um controlo:** o dono escolhe
as duas portas e o painel diz, de uma, que ela tem cérebro, e da outra que não. Sem corpo, *«a que
não tem»* era inalcançável e a comparação não existia.

### Os dois gates que faltavam

- **`o_que_tem_cerebro_tem_corpo`** — o portador do componente tem `Sprite`, de área não-nula e não
  escondido. Sozinho, este teria apanhado o defeito.
- **`o_dedo_do_dono_apanha_a_porta_e_nao_uma_placa`** — no ponto que o roteiro nomeia
  (`ONDE_O_DONO_TOCA`, derivado da geometria), os únicos sprites visíveis que o contêm são a porta e
  o chão.

⚠️ **O segundo é uma REPRODUÇÃO declarada da lei do `pick_sprite_at_world`**, não ela: aquela corre
sobre o mundo de PRESENTE (`RenderInstance` + `GlobalTransform`), que esta crate não monta. A
divergência fecha-se por **asserção**: o gate afirma, de cada sprite da cena, que não tem rotação,
escala, skew, âncora nem deslocamento — e só então aplica a caixa alinhada aos eixos. ⛔ E a
visibilidade lê-se por `world.get` entidade a entidade, **nunca** por um `Option<&Visibility>` dentro
da consulta: o `try_query` do `bevy_ecs` devolve `None` quando *qualquer* componente dela é
desconhecido do mundo — a armadilha que a wave das TAGS já pagou.

### O CENSO: as outras cenas desta linha

Varridas as cenas de `ph2d-app-components` e `ph2d-app-physics` à procura de um `world.spawn` que
carregue um componente do TOP-20 **sem** `Sprite`, sobram **dois** acertos e **nenhum é defeito**:
as duas `GameCamera` (`camera_2d_smoke` e `factory_smoke`). Uma câmera **não tem corpo** por
natureza, e o roteiro impresso das duas cenas não manda clicar nela. ⚠️ **E o censo é um proxy
TEXTUAL que dá falso positivo na própria cura** — ele acusa `Door` porque a palavra `Sprite` mora
dentro do `corpo_da_porta()` e não no bloco do `spawn`. *Um censo por texto sobre uma cena mede
como ela está escrita, não o que ela monta;* quem o repetir num gate tem de montar o mundo.

### ⏳ O que este report deixa ABERTO e NÃO foi curado

**Nenhuma secção opcional do Inspector — as 26 — tem gate a provar que ela chega a PIXEL.** A
presença é decidida por `build_*_info` (gateado) e a tabela `LIVE_SECTIONS` é gateada por censo,
mas não existe arnês de pintura na `ph2d-panel-inspector`, e construí-lo é wave própria. Esta
rodada **não** o construiu, e o defeito de hoje não estava aí — mas a lente continua a faltar.

---

## §9 — O que fica ABERTO

Ver [`12_plano_state_machine.md` §8.2](../12_plano_state_machine.md): estados aninhados (recusado
por desenho) · o verbo `EmitSignal` na tabela de acções (a recusa dissolveu **para esta máquina**,
não para ela) · o custo a N máquinas não varrido · e a tabela que **não se reordena** no painel — o
arrasto de linha (`PanelRowFamily`) **existe** desde a wave das tags, e ligá-lo é UI, não mecanismo.
