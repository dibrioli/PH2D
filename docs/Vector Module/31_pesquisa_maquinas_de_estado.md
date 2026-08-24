# 31 — PESQUISA: as máquinas de estado mais amadas, e o desenho da **máquina do Morph**

> Pedido do Enio em **2026-08-24**: *"A maquina de estados do morph deve funcionar no próprio canvas
> 2d onde criaremos setar de uma forma para outra e nas setas colocaremos condições. antes de
> programar faça pesquisa e estudo profundos e descubra as maquinas de estado mais amadas pelos
> usuários nos mais diversos apps e vamos usar os melhores como referência."*
>
> ⛔ **Isto é a PESQUISA e a direcção de desenho — não é o plano de execução.** O plano nasce quando
> a wave começar, e ela começa **depois** do [Input Map (doc 30)](30_plano_input_map.md).

---

## §1 — As cinco referências, e o que cada uma resolveu

### 1.1 ⭐⭐⭐ **Rive — *State Machine*** (a mais próxima, e este módulo já é *Rive-referenced*)

É, literalmente, o produto que o Enio descreveu: **um grafo visual onde se arrastam setas entre
estados e se põem condições nas setas.**

**Como uma seta nasce** — *"posicione o cursor perto do estado de origem até aparecer um círculo;
clique e arraste desse círculo até o estado de destino"*. A seta é **direccional**: A→B e B→A são
duas setas.

**As TRÊS entradas** (e note como elas casam com o doc 30):

| Tipo | O que é | Exemplos da própria Rive |
|---|---|---|
| **Boolean** | verdadeiro/falso **persistente** | `IsLoading`, `IsHovered`, `IsDarkTheme` |
| **Trigger** | sinal **transiente**, de uma vez | `OnClick`, `Fire`, `ErrorOccurred` |
| **Number** | `f32` contínuo | `PercentDownloaded`, `ScrollPosition`, `HealthPoints` |

**As propriedades de uma seta** — e esta lista é o padrão-ouro a copiar:

- **Duration** — quanto dura a transição (`0` = corte seco).
- **Exit Time** — que fracção do estado de origem tem de tocar antes de sair (tempo ou %).
- **Pause Source When Exiting** — congela a origem enquanto a transição corre.
- **Allow Exit During Transition** — deixa sair de uma transição a meio, se a condição mudar.
- **Interpolation** — a easing da transição.
- **Actions** — a seta pode **executar** algo ao disparar (definir propriedade, emitir evento),
  no **início** ou no **fim**.

**A lógica booleana, e a decisão é elegante:**
> várias condições **na mesma seta** = **E** · várias **setas** entre os mesmos dois estados = **OU**

⇒ **não há operador `OR` na UI.** O desenho **é** a expressão. É a razão pela qual a Rive não
precisa de um editor de expressões, e é a coisa mais copiável de todo este documento.

**Camadas:** em conflito, a camada mais à direita/baixo **ganha** — é o que deixa um *"Hit
Reaction"* sobrepor-se temporariamente ao *"Walk Cycle"*.

### 1.2 ⛔ **Unity Animator / Mecanim** — o caso de ÓDIO, e é o que NÃO fazer

A crítica é consistente e específica:

- *"as setas de transição estão em todo o lado"*, com parâmetros que ninguém *"tem coragem de
  apagar porque não sei o que os usa"*.
- **passados ~30 estados vira esparguete.** Sub-state machines *"na maior parte só escondem a
  confusão um nível mais fundo"*.
- *"não estás a desenhar 50 setas para exprimir lógica que na verdade pertence a código"*.
- A saída da comunidade foi **abandonar o editor**: mover a lógica para a *Playables API*, ou
  substituir o Animator por FSMs em C# puro.

⭐⭐ **As DUAS lições operacionais** (as duas entram no nosso desenho):

1. ⛔ **Um parâmetro tem de saber quem o usa.** O medo de apagar é um defeito de **rastreabilidade**,
   não de estética — e é barato de curar: *quem lê este input?* é uma pergunta que a máquina pode
   responder por construção.
2. ⛔ **Um grafo que cresce sem hierarquia morre.** Não basta ter sub-máquinas — elas têm de
   **reduzir** o que se vê, não aninhar a mesma parede.

### 1.3 **Unreal — *State Tree*** (a resposta da Epic ao esparguete)

*"uma máquina de estados hierárquica de propósito geral que combina os **Selectors** das behavior
trees com os **States e Transitions** das máquinas de estado."*

⭐ **A decisão de desenho que importa:** o State Tree **só avalia as transições do estado ACTUAL** —
*"mais barato e mais previsível, mas obriga a pensar as transições explicitamente"*.

⇒ é o oposto exacto do `Any State` do Unity, que é justamente de onde nasce metade das setas do
esparguete. **Herdamos a regra: avaliar só o que sai do estado corrente.**

### 1.4 **Statecharts (Harel, 1987) e XState / Stately** — a resposta formal, e a mais amada fora dos jogos

O que o formalismo dá, e que nenhum dos editores de jogo dá inteiro:

- **hierarquia** (estados dentro de estados — uma transição de saída do pai serve todos os filhos:
  **N setas viram uma**);
- **estados paralelos** (regiões ortogonais activas ao mesmo tempo — é a *"camada"* da Rive, com
  nome formal);
- **guardas** (a condição na seta);
- **acções de entrada/saída**.

⭐ O que a comunidade do XState repete: *"statecharts valem mais quando são tratados como
**comportamento executável**, não como documentação"* — e o Stately Studio existe exactamente para
os **editar visualmente** em vez de os desenhar à parte. É a mesma tese deste app.

### 1.5 **Godot — `AnimationTree` / `AnimationNodeStateMachine`**

A referência que o Enio conhece: estados são animações, transições têm **modo de troca**
(*imediato · sincronizado · no fim*) e **advance conditions**. ⚠️ É mais pobre que a Rive em
condições (não tem o vocabulário de três tipos), e é onde a Rive claramente ganha.

---

## §2 — A escolha, derivada dos cinco princípios

> **Base: a State Machine da Rive.** Com **duas** correcções vindas das outras referências.

| Princípio | O que decide |
|---|---|
| **Estado da arte / padrão-ouro** | A Rive é o produto que resolveu **exactamente** este problema para **formas vectoriais**, e este módulo já é *Rive-referenced* por ADR-0108. Copiar a semântica dela é copiar a melhor que existe |
| **Intuitivo para artistas** | ⭐ *"várias condições na seta = E; várias setas = OU"* — o **desenho é a expressão**, e o artista nunca vê um operador booleano |
| **Poderoso** | os **três tipos** de entrada (bool/trigger/number) exprimem toggle, evento e valor contínuo sem tipo novo |
| **Fácil de usar** | as propriedades da seta são as da Rive (duration · exit time · interpolation), que são o vocabulário que animadores já têm |
| ⛔ **correcção 1 (Unreal State Tree)** | **avaliar só as transições que saem do estado corrente.** Sem `Any State` na v1 — é a fábrica de esparguete do Unity |
| ⛔ **correcção 2 (Unity, a lição do medo)** | **todo input sabe quem o lê.** Seleccionar um input acende as setas que dependem dele; apagá-lo diz **quantas** partirá. Barato, e mata o defeito nº 1 da referência odiada |

### 2.1 ⚠️ Onde o pedido do Enio DIVERGE da Rive — e por que ele pode estar certo

A Rive põe a máquina num **grafo separado** (uma vista própria, com caixas abstractas). O Enio pediu
**no próprio canvas 2D, com setas de forma para forma**.

- ⭐ **A favor:** aqui os estados **são objectos visíveis**. Uma caixa abstracta chamada `"Pose B"`
  obriga o artista a manter um mapa mental entre o nome e o desenho; uma seta que sai da **forma
  desenhada** não tem esse salto. É mais directo que a referência.
- ⛔ **Contra, e é o risco medido do Unity:** setas por cima da arte **competem com a arte**. Aos 30
  estados, o canvas fica ilegível — e ao contrário do Animator, aqui não há uma vista limpa para
  onde fugir.
- ⇒ **A mitigação a desenhar (não decidida):** as setas são uma **camada de overlay comutável**,
  visível no *modo Morph* e apagada fora dele — o precedente existe neste app (*"Show sheet on
  canvas"* do Sprite, e o realce de proveniência que esta linha construiu em 23/08). ⚠️ A pergunta
  a **medir** na wave é: *a partir de quantos estados o overlay deixa de ser legível?*

---

## §3 — O que JÁ está construído aqui (medido 2026-08-24, com endereço)

| Peça | Onde | O que serve |
|---|---|---|
| **O morph vivo** | [`ph2d-ecs/src/vec_morph.rs`](../../crates/ph2d-ecs/src/vec_morph.rs) | ⭐ **já não-destrutivo**: guarda a **relação**, a aparência é função pura re-cozida por quadro; undo e save de graça. ⛔ **entre DUAS formas** — é esta a parede |
| **Uma máquina de estados** | [`ph2d-ui-state/`](../../crates/ph2d-ui-state/) — `machine.rs` · `transition.rs` · `pose.rs` · `role.rs` · `sets.rs` · `binding.rs` | estados de UI + Smart Animate. Em 23/08 aprendeu a carregar **morfos por-objecto** (`BoolMorph`, `Transition::bool_morphs(t)`) ⇒ **o padrão "a transição carrega um morfo" já foi construído e smokado** |
| **Um editor de grafo no canvas** | [`ph2d-panel-motion-graph/`](../../crates/ph2d-panel-motion-graph/) — `flow.rs` · `geom.rs` · `hits.rs` · `interact_*.rs` | ⭐ **o gesto de arrastar um fio entre dois nós já existe neste app**, com hit-test e roteamento. ⛔ **Meça-o antes de escrever um segundo** |
| **A saída de sinais** | [`ph2d-runtime`](../Runtime/) (R0) | `Signal`/`SignalOutbox`/`SignalReader`. ⭐ **É onde os `Trigger` da Rive aterram**: o produtor publica, o consumidor lê com cursor próprio |
| **O Input Map** | [doc 30](30_plano_input_map.md) — **por construir, e vem primeiro** | ⭐ **é a fonte dos inputs da máquina.** `pressed("jump")` é um **Boolean**; `just_pressed` é um **Trigger**; `strength`/`axis` é um **Number**. As três da Rive, sem inventar nada |

⭐⭐ **É por isto que o Enio mandou fazer o input map primeiro, e a pesquisa confirma-o:** sem ele, as
condições das setas não teriam de onde vir, e a wave inventaria uma fonte falsa para as testar.

---

## §4 — As perguntas que o PLANO terá de responder (aqui não respondidas)

1. ⭐ **A pergunta-mãe: esta máquina é uma SEGUNDA, ou a `ph2d-ui-state` a servir outro dono?**
   Duas máquinas sobre o mesmo tipo de facto é [[feedback_two_engines_one_state_is_worse_than_a_slow_engine]].
   ⚠️ O Enio pediu *"um tipo **específico** para o tool Morph"* — o plano tem de dizer **qual contém
   a outra**, com medição, e não escolher por conforto.
2. **"Entre múltiplas formas"** — um estado é *uma forma-alvo* (A→B→C **passa por B**) ou *uma pose
   no espaço de N formas* (mistura A+B+C, **não passa por lado nenhum**)? São produtos diferentes.
   ⚠️ A Rive responde **a primeira**, e o pedido (*"setas de uma forma para outra"*) também.
3. ⛔ **"funcional no runtime do game"** — hoje quem cozinha o morph é a **shell do editor**, e o
   `shells/game` (**R1**) está **adiado por decisão do Enio**. ⇒ a lei desce a uma **crate-folha**
   que as duas shells consomem, ou o R1 sai do gelo. **É o item que muda o preço.**
   ⭐ O doc 30 §7 já manda o `ph2d-input` nascer folha **por esta razão**.
4. **Correspondência entre formas com contagens de nós diferentes** — o Morph de duas já a resolve.
   Com N, é **transitiva**? ⚠️ O Flip resolveu o irmão disto por **atribuição óptima + espiral
   logarítmica** ([`docs/Flip/`](../Flip/), tween v2) — **código nosso, medido**. Leia antes de inventar.
5. **Determinismo** — se a máquina corre no runtime, ela entra no replay. ⚠️ Ver a lei nº 1 do
   [doc 30 §2.1](30_plano_input_map.md): **a fita grava a acção resolvida**, e o `physics_ecs_c9`
   é o controlo.
6. **Ordem de avaliação quando duas setas podem disparar** — ⚠️ **a documentação da Rive NÃO
   especifica** a resolução de prioridade (verificado). ⇒ é uma decisão **nossa**, e tem de ser
   **determinística e visível** (a ordem de criação é o candidato óbvio, mostrada na UI).

---

## §5 — Onde encosta (a re-verificar no dia)

- **`PROJECT_SCHEMA`** (hoje **95**) — **move**; conta-se nos **três** sítios.
- **Registro de componentes** (`ph2d-ecs` 65, dois espelhos 66) — provável **+1**, número que **soma**.
- **Contratos congelados** (§6) — não deve encostar. **Prove por grep.**
- **`VEC_SCENE_SCHEMA_VERSION`** (hoje **14**) — só se a forma cozida mudar.

> ⚠️ **Meça cada linha antes de a honrar.** Escrito em 2026-08-24; *quem move o número reconfere a
> nota* ([estudo §6.6.1](Estudos/ESTUDO_UI_viva_o_que_falta_para_encantar_2026-08-12.md)).

---

## Fontes

- **Rive** — [State Machines (guia)](https://rive.app/docs/editor/state-machine/transitions) ·
  [Inputs](https://help.rive.app/editor/state-machine/inputs) ·
  [A beginner's guide to the Rive State Machine](https://rive.app/blog/how-state-machines-work-in-rive) ·
  [State Machines em runtime](https://help.rive.app/runtimes/state-machines)
- **Unity Animator** — [o esparguete aos ~30 estados](https://moonjump.com/forum/animating/unity-s-animator-controller-turns-into-spaghetti-past-30-states-how-are-you-actually-managing-this-15de87) ·
  [Don't Re-invent Finite State Machines](https://medium.com/the-unity-developers-handbook/dont-re-invent-finite-state-machines-how-to-repurpose-unity-s-animator-7c6c421e5785)
- **Unreal State Tree** — [Overview of State Tree](https://dev.epicgames.com/documentation/en-us/unreal-engine/overview-of-state-tree-in-unreal-engine) ·
  [State Tree vs Behavior Tree (2026)](https://www.strayspark.studio/blog/state-tree-vs-behavior-tree-ue5-7-migration-2026)
- **Statecharts / XState** — [xstate](https://github.com/statelyai/xstate) ·
  [statecharts.dev](https://statecharts.dev/resources.html) ·
  [Harel, *A Visual Formalism for Complex Systems* (1987)](https://news.ycombinator.com/item?id=25454175)
