# 29 — FILA: a *state machine* do Morph · e o *Texture Pattern* para formas vetoriais

> ⛔ **ISTO NÃO É UM PLANO — é a FILA.** Pedido do Enio em **2026-08-24**, com a instrução
> explícita: *"Apenas insira na fila de implementação. não começaremos hoje. Amanhã iniciaremos"*.
> Nenhuma linha de código foi escrita, nenhuma decisão de desenho foi tomada.
>
> ⭐ **O plano de cada uma nasce quando ela começar**, pelo `/pd-feature` (pesquisa do estado da
> arte · a porta única · onde encosta em contrato/schema · as 4 condições de UI · os gates
> red-first · a cena de smoke com números medidos). Este doc existe só para que esse plano **não
> gaste a primeira hora a redescobrir o que já está construído**.

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

### Onde encosta (a conferir no plano, não decidido)

- **Schema:** um componente novo, ou campos novos no `VecMorph`, movem o **`PROJECT_SCHEMA`**
  (hoje **95**) — e o número **conta-se** contra o `main` do dia, nos **três** sítios
  ([`CLAUDE.md §5.0`](../../CLAUDE.md)).
- **Contrato congelado (§6):** `VectorOp`/`Vertex`/`Segment`/`AnimValue` em `ph2d-vector-doc` estão
  **congelados**. O motor novo (`ph2d-vec-*`) **não** está. ⚠️ Confirme por grep antes de assumir.
- **Registro de componentes:** `ph2d-ecs` está em **65**, com **dois espelhos** em 66
  (`ph2d-render`, `ph2d-script`) — número que **soma entre linhas**.

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
