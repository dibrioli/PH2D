# ADR-0199 — A forma de colisão é um objeto filho do corpo

- **Status:** Accepted — produto do dono (08/11, smoke da W3-B); desenho técnico delegado (padrão-ouro).
- **Data:** 2026-11-08
- **Linha:** `line/GameEngineFuncional02`
- **Estende:** [ADR-0110](0110-vector-nodes-are-ecs-entities-one-hierarchy.md) (uma Hierarquia só) e o ADR-0131 (física runtime-truth); não revoga nenhum.
- **Amparo:** [plano f2, «REVISÃO 08/11 … opção B»](../../GameEngineFuncional/componentes/f2_contorno_e_formas_plano.md) · [continuação](../../GameEngineFuncional/handoffs/HANDOFF_CONTINUACAO_2026-11-08_OPCAO_B.md)

> ⚠️ Número por recontar na integração (§5.0): escrito como `0199` com o `main` em `0198`.

## Contexto

Até a schema 219 a forma de colisão de um corpo era um componente `Collider` na própria entidade do corpo, e as formas extra
eram filhas (ADR-0110, `bridge/parts.rs`). Duas maneiras de dizer «esta é uma forma» levaram a duas UIs: o cartão Shape do
corpo e, na W3-B, uma lista «Shapes» que era vista dos filhos. No smoke de 08/11 o dono revogou a lista e pediu: *«tirar
collision shapes dos componentes e deixar apenas como Objeto aparecendo como filhos na hierarchy como objetos independentes
(invisíveis)»*.

## Decisão

1. **Motor, uma lei.** As formas de um corpo são o seu `Collider` (se ainda tiver um) e, depois dele, as formas descendentes pela
   ordem da árvore, parando num corpo filho e atravessando grupos (`ph2d_physics_ecs::shapes_in_order`). A primeira é a **forma
   principal** (`main_shape`): viaja no `BodyDesc` como antes (Crouch, linha d'água, força de zona, `rest`, nav e gatilhos leem-na),
   com a pose relativa em `BodyDesc.shape_pose: [f32; 3]` (zero = o caso antigo). As outras são partes. O lado-forma do catálogo
   (material, one-way, superfície, zonas, sinais, `LADO_FORMA`) é lido da entidade da FORMA.
2. **Equivalência provada.** Um corpo com a forma em si e o mesmo corpo com a forma numa filha dão o mesmo hash em 300 ticks
   (`crates/ph2d-physics-ecs/tests/it/forma_filha.rs`).
3. **Editor, uma lei.** Nenhuma porta do editor põe `Collider` num corpo (gate `nenhuma_porta_do_editor_poe_a_forma_no_corpo`):
   todo corpo nasce com a filha «Collision Shape» (`nasce_com_forma`); *Add Body*, *Make Independent Body* e o *Rig* passam por
   `torna_corpo`; um campo de forma editado no corpo vai para `main_shape` (`dono_do_campo`). «Collision Shape» sai do Add
   Component e fica no Add ▸ objeto (sem seleção → «Static Body» + filha; imagem sem corpo → a imagem vira Static + filha; corpo
   → filha dele).
4. **Formato.** `PROJECT_SCHEMA` 219 → 220: um projeto `<= 219` abre com `migra_formas_para_filhos` (por `StableId`, a filha
   primeira entre irmãs, Transform identidade, leva o `Collider` e o lado-forma; trilhas da Timeline do lado-forma re-alvejadas;
   instâncias materializam do mestre). O motor continua a aceitar `Collider` no corpo: é o que a migração lê e o que os testes antigos montam.
5. **Inspector e canvas.** O corpo mostra só o cartão Body; as definições da forma vivem só no Inspector da filha. As alças da
   forma só aparecem com a PRÓPRIA forma selecionada; com o corpo selecionado o canvas mostra só o contorno.

## Alternativas recusadas

| Alternativa | Por que não |
|---|---|
| Lista «Shapes» no cartão do corpo (W3-B) | Revogada pelo dono no smoke: a forma passa a aparecer em dois sítios |
| `Collider.shapes: Vec<…>` | Segunda verdade ao lado dos filhos; tira `Copy` de `Collider`/`ShapeDesc`/`BodyDesc`; degrau de schema sem ganho |
| Corpo sem collider próprio, todas as formas como partes | Parte os cinco leitores da forma principal e a ordem dos handles do rapier |
| O motor recusa `Collider` no corpo | 234 ficheiros de teste a reescrever sem nenhum comportamento novo; a migração precisa de o ler |
| Sistema por quadro que converte a forma do corpo em filha | Muta o documento fora de um gesto: o undo vê um passo que ninguém fez |

## Preço

- Cada corpo do editor custa uma entidade a mais (a filha) e uma linha a mais na Hierarchy.
- `main_shape` percorre os filhos: o leitor por quadro não pode alocar (`ordered_children`), cura na B6.
- Força de zona e rotação em formas filhas precisam da pose da forma, não só a do corpo (`effector.rs`, B6).
- O Outline (`ColliderOutline`) mora na FILHA e é a schema 221.
