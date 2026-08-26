# 29 — FILA: a *state machine* do Morph · e o *Texture Pattern* para formas vetoriais

> ⛔ **ISTO NÃO É UM PLANO — é a FILA.** Pedido do Enio em **2026-08-24**, com a instrução
> explícita: *"Apenas insira na fila de implementação. não começaremos hoje. Amanhã iniciaremos"*.
>
> ⭐⭐ **ACTUALIZADO no mesmo dia** — o Enio deu a direcção e mudou a ORDEM:
>
> > *"A maquina de estados do morph deve funcionar no próprio canvas 2d onde criaremos setar de uma
> > forma para outra e nas setas colocaremos condições. (…) **Antes da máquina de estados,
> > criaremos o sistema de Inputs** (como o input Map do Godot). (…) **Primeiro o input map.**"*

## A ORDEM (e o porquê dela, que a pesquisa confirmou)

| # | O quê | Documento | Estado |
|---|---|---|---|
| **1** | ⭐ **O Input Map** — entradas nomeadas, à la Godot | **[30 — PLANO](30_plano_input_map.md)** | ✅ **FECHADO** e integrado ao `main` em 2026-08-24 |
| **2** | **A máquina de estados do Morph** | **[32 — PLANO](32_plano_maquina_de_estados_do_morph.md)** (pesquisa: [31](31_pesquisa_maquinas_de_estado.md)) | ✅ **W1–W8 feitas.** ⚠️ **O desenho MUDOU em 25/08 e o §5 do plano 32 é a fonte:** um botão faz o conjunto e gera o **grafo completo dirigido**; as setas são **virtuais** (⛔ o desenho no canvas e o gesto de arrasto foram **retirados**); seção **própria** *Morph States* |
| **3** | **Texture pattern** no preenchimento vectorial | §F2 **deste** doc | **é a próxima**, sem plano |

⭐⭐ **Por que o input map vem primeiro, e não é só ordem preferida:** a pesquisa de máquinas de
estado (doc 31) mostrou que as entradas da State Machine da **Rive** são exactamente **três** —
*boolean · trigger · number* — e que um input map produz as três **de graça**: `pressed()` é o
boolean, `just_pressed()` é o trigger, `strength()`/`axis()` é o number. **Construir a máquina
antes teria obrigado a inventar uma fonte falsa para as condições dela.**

> ⛔ **O que resta abaixo é a FILA original**, mantida porque a F2 continua sem plano. A F1 foi
> **substituída** pelos docs 30 e 31 — leia-os, não a esta secção.

---

## O pedido, verbatim

> **Próxima feature:** um tipo de state machine específico para o tool Morph
> Esse nova feature possibilita que o morph seja criado entre múltiplas formas de forma não
> destrutiva e funcional no runtime do game.
>
> **Mais uma feature:** Texture patttern para Formas Vetorias.

---

## F1 — A *state machine* do Morph

### O que já existe (medido 2026-08-24, com endereço)

| Peça | Onde | O que ela é hoje |
|---|---|---|
| **O morph vivo** | [`ph2d-ecs/src/vec_morph.rs`](../../crates/ph2d-ecs/src/vec_morph.rs) | ⭐ **já é não-destrutivo**: o componente guarda a **relação** (quais formas, e onde no caminho), e a aparência é **função pura** dela, re-cozida a cada quadro. Ninguém desenha um morph — mexe-se numa fonte ou no `t` e a forma refaz-se. Undo e save cobrem-no **sem uma linha a mais** |
| ⛔ **e é entre DUAS formas** | idem | *"a forma única que É o caminho entre **duas** outras, parada num `t`"* — **é exactamente esta a parede que o pedido derruba** |
| **O irmão de ilustração** | `VecBlend` ([ADR-0128](../architecture/decisions/0128-vector-blend-object-live-virtual-steps-editable-spine.md)) | mostra os **N passos de uma vez** (o Blend do Illustrator). *Blend é ilustração; Morph é animação* |
| **Uma máquina de estados que JÁ EXISTE** | [`ph2d-ui-state/`](../../crates/ph2d-ui-state/) — `machine.rs` · `transition.rs` · `pose.rs` · `role.rs` · `sets.rs` · `binding.rs` | os **estados de UI + Smart Animate**. Em 23/08 esta linha ensinou-a a carregar **morfos de booleana** (`BoolMorph`, `Transition::bool_morphs(t)`) — ⇒ **o padrão de "a transição carrega um morfo por-objecto" já foi construído e smokado uma vez** |
| **O nó de Motion** | `crates/ph2d-node-motion-morph/` | outro domínio (grafo de nós), **não** confundir |
| **A saída de sinais do runtime** | [`ph2d-runtime`](../Runtime/) (R0) | `Signal`/`SignalOutbox`/`SignalReader` — o produtor **publica**, cada consumidor lê com o próprio cursor |

### As perguntas que o plano de amanhã tem de responder (não respondidas aqui)

1. ⭐ **A pergunta-mãe: esta máquina é uma SEGUNDA, ou é a `ph2d-ui-state` a servir outro dono?**
   Duas máquinas de estados sobre o mesmo tipo de facto é a assinatura de
   [[feedback_two_engines_one_state_is_worse_than_a_slow_engine]]. ⚠️ Mas o Enio pediu *"um tipo
   **específico** para o tool Morph"* — o plano tem de dizer **qual contém a outra**, com a
   medição, e não escolher por conforto.
2. **"Entre múltiplas formas"** — o que é um estado: *uma forma-alvo*, ou *uma pose no espaço de
   N formas*? Um morph A→B→C **passa por B**, um morph que mistura A+B+C **não passa por lado
   nenhum**. São produtos diferentes e a UI de cada um é diferente.
3. ⛔ **"funcional no runtime do game"** — é o item que **muda o preço da feature**. Hoje quem
   cozinha o morph é a **shell do editor** (`shells/desktop`), e o `shells/game` (**R1**) está
   **adiado por decisão do Enio** ([`CLAUDE.md §5`](../../CLAUDE.md), Runtime). O plano tem de
   dizer: a lei do morph desce para uma **crate-folha** que as duas shells consomem, ou o R1 sai
   do gelo? *Uma lei que só existe dentro do editor não é "funcional no runtime".*
4. **Correspondência entre formas com contagens de nós diferentes** — o Morph de duas já a resolve;
   com N, a escolha de correspondência é **transitiva ou não?** (o Flip resolveu o irmão disto por
   **atribuição óptima + espiral logarítmica** — [`docs/Flip/`](../Flip/), tween v2. ⚠️ **Leia antes
   de inventar**: é código nosso, medido, noutro módulo.)
5. **Onde a máquina é AUTORADA** — painel do Vector, timeline, ou a árvore? A booleana-nos-estados
   de 23/08 escolheu o painel `states`; o precedente existe e tem gates de costura.

### ⭐ RECONFERÊNCIA de 2026-08-25 — **três afirmações desta folha estavam erradas no ponto que decide o preço**

> ⚠️ *Quem move o número que tornava algo inalcançável tem de reconferir a nota* (`CLAUDE.md` §0.0).
> Entre a escrita desta folha e hoje, o Input Map fechou e **oito outras linhas integraram**. Medido
> na reabertura, com endereço:

1. ⛔⛔ **A parede NÃO é «o `VecMorph` é entre duas formas».** O `vec_morph.rs` tem **64 linhas** e é
   um morph **keyado pela timeline** entre dois `VecPathId` — outro subsistema. Quem já interpola
   **N objectos** entre poses, com correspondência por id, geometria **cozida** e morfo de booleana,
   é a [`ph2d-ui-state`](../../crates/ph2d-ui-state/) — **3 112 LOC**, viva, e a dirigir o canvas
   pela ponte [`ui_state_bridge.rs`](../../shells/desktop/src/render_loop/ui_state_bridge.rs).
   ⇒ **a pergunta 1 já tem resposta medida: a máquina que contém a outra é a `ph2d-ui-state`.**
2. ⛔ **A parede REAL é o CATÁLOGO.** A máquina existente move-se entre **quatro papéis FIXOS**
   (`StateRole::{Default,Hover,Pressed,Disabled}` — um `enum`, `ALL: [_; 4]`), e a
   [`Transition`](../../crates/ph2d-ui-state/src/transition.rs) é **calculada** entre duas listas de
   pose, **nunca autorada**. Não há estado com **nome**, não há **aresta** e não há **condição**.
   ⇒ a obra é *generalizar de papel-fixo para estado-nomeado + arestas autoradas*, não construir uma
   máquina.
3. ⭐⭐ **O «modo preview» que o Enio pediu JÁ EXISTE** —
   [`ui_preview.rs`](../../shells/desktop/src/render_loop/ui_preview.rs), **323 LOC**, e o
   doc-comment dele chama-se a si próprio *"a metade de RUNTIME"*. Ele já resolve as duas coisas
   difíceis: enquanto corre, **o gesto de edição não existe e o undo não regista**; ao sair, **o
   mundo volta ao que era** (⚠️ restaura a pose CAPTURADA, ⛔ nunca «vai para o Default», que
   moveria o desenho do artista). ⇒ a pergunta 3 encolhe: **a lei desce a uma crate-folha** (já está
   numa) e corre na preview **hoje**; o `shells/game` continua a não existir e continua adiado — é o
   MESMO bloqueio dos contextos do Input Map, e não um preço novo desta feature.

⚠️ **E o vocabulário das CONDIÇÕES nasceu no dia 24:** as acções nomeadas do Input Map
([plano 30](30_plano_input_map.md)) são exactamente o que uma aresta lê. Uma condição sobre uma
tecla crua teria de ser reescrita no dia seguinte.

⚠️ **Arestas no canvas 2D — o que existe e o que não existe:** há um editor de grafo completo
(`ph2d-panel-motion-graph`, **11 381 LOC**: fios, sockets, hit-test, zoom, subgrafos) — mas ele é um
**painel**, não o canvas. No canvas há precedente de **vínculo autorado desenhado e arrastável**
(as juntas da física, o gizmo de âncoras). ⇒ o plano tem de dizer **qual dos dois** é o chão, e a
resposta muda o tamanho da wave.

### Onde encosta (a conferir no plano, não decidido)

- **Schema:** um componente novo, ou campos novos, movem o **`PROJECT_SCHEMA`** — medido **97** em
  2026-08-25 (era 95 quando esta folha nasceu, e a integração de 24/08 **RECONTOU** 96→97 porque
  duas linhas escreveram o mesmo literal). O número **conta-se** contra o `main` do dia, nos **três**
  sítios ([`CLAUDE.md §5.0`](../../CLAUDE.md)).
- **Contrato congelado (§6):** `VectorOp`/`Vertex`/`Segment`/`AnimValue` em `ph2d-vector-doc` estão
  **congelados**. O motor novo (`ph2d-vec-*`) **não** está. ⚠️ Confirme por grep antes de assumir.
- **Registro de componentes:** os **dois espelhos** estão em **71** (`ph2d-render`, `ph2d-script`) —
  eram 66 quando esta folha nasceu. Número que **soma entre linhas**; ⛔ conte-o, nunca o copie
  daqui.

---

## F2 — *Texture pattern* para formas vetoriais

### O que já existe (medido 2026-08-24)

O preenchimento de hoje é o `enum Paint` em
[`ph2d-vec-scene/src/paint.rs`](../../crates/ph2d-vec-scene/src/paint.rs), com **quatro** variantes
e **nenhuma** de imagem:

| Variante | O que é |
|---|---|
| `Solid(Rgba8)` | cor chapada |
| `Linear { stops, start, end }` | rampa linear, **world-space** |
| `Radial { stops, center, radius }` | rampa radial, **world-space** |
| `MultiPoint { points }` | freeform IDW (o do Cavalry), rasterizado num **image-brush** |

⭐⭐ **A LEI que este módulo já pagou, e que a F2 herda inteira** (está escrita no cabeçalho do
`paint.rs`): a geometria do preenchimento é guardada em **WORLD-space** e **transforma junto com o
path** — rodar a forma roda o preenchimento **rigidamente**, sem *"respirar"*. Isso foi a cura de
um bug: o gradiente relativo à bbox respirava a cada edição.

⇒ **A primeira pergunta da F2 não é "que formato de imagem", é "o padrão anda com a forma?"** — e a
resposta que o módulo já deu para os gradientes é **sim, rigidamente**.

⭐ **E há um vizinho a ler ANTES de desenhar:**
[`23_plano_pattern_along_path.md`](23_plano_pattern_along_path.md) — *pattern* **ao longo de um
caminho** já foi planeado neste módulo. ⚠️ **Meça o que dele está construído** antes de assumir que
é outra coisa (é literalmente o modo de falha que o [estudo §6.6](Estudos/ESTUDO_UI_viva_o_que_falta_para_encantar_2026-08-12.md)
documenta **nove** vezes).

### As perguntas que o plano de amanhã tem de responder

1. **Uma variante nova no `Paint`, ou um componente ao lado?** O `Paint` é construído em muitos
   sítios; [[feedback_widely_constructed_type_favors_optional_component_over_appended_field]] diz
   para medir antes de apender.
2. **De onde vem a textura** — um asset do projeto, uma camada raster do Painter, ou uma forma
   vetorial cozida? Cada resposta tem um dono de ciclo de vida diferente.
3. **Repetição:** tile / mirror / clamp · escala e rotação **próprias** do padrão · e o que
   acontece ao padrão quando a forma é escalada **não-uniformemente** (⚠️ é a **mesma família** do
   bug **#27** que esta linha fechou em 23/08: a caneta virava elipse porque o transform multiplica
   o **pincel**, não só a geometria — [`stroke_uniform.rs`](../../crates/ph2d-vec-render/src/stroke_uniform.rs)).
4. **Vello:** o `MultiPoint` já rasteriza num **image-brush** — ⇒ **a rota de imagem já existe**
   nesta crate. Meça-a antes de abrir outra.
5. **Schema:** variante nova no `Paint` move o **`VEC_SCENE_SCHEMA_VERSION`** (hoje **14**) e,
   por arrasto, o `PROJECT_SCHEMA`. ⚠️ Confirme a regra posicional do postcard na escada de
   [`project_schema.rs`](../../shells/desktop/src/project_schema.rs) — *apender variante* e
   *apender campo* não custam a mesma coisa.

---

## Como amanhã começa

1. `bash scripts/hw-profile.sh` e confirme a worktree (`pwd` + `git branch --show-current`) — a
   janela abre no primário e o mesmo caminho relativo existe nas duas árvores.
2. **`/pd-feature`** para a F1. Plano **antes** de código; produto final, não MVP.
3. ⚠️ **Meça cada linha deste doc antes de a honrar.** Ele foi escrito em 2026-08-24 e vale o que
   valia nesse dia — *quem move o número reconfere a nota*
   ([§6.6.1](Estudos/ESTUDO_UI_viva_o_que_falta_para_encantar_2026-08-12.md)).
