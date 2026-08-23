# ADR-0072-amendment-1 — Montar numa âncora é um QUADRO na hierarquia (o consumidor do §2.6)

**Status:** Accepted (2026-08-22, `line/Sprite`) — implementado, com gate de paridade entre as duas travessias de mundo.
**Amends:** [ADR-0072 — Named Anchor unification](0072-named-anchor-unification.md) §2.6 (Runtime API) e §5 (critérios de fecho da W5).
**Spec:** [`docs/Sprite_projeto/07_named_anchors.md`](../../Sprite_projeto/07_named_anchors.md).
**Referência:** [`crates/ph2d-ecs/src/anchor_mount.rs`](../../../crates/ph2d-ecs/src/anchor_mount.rs) · gate [`anchor_mount_hierarchy.rs`](../../../crates/ph2d-ecs/tests/anchor_mount_hierarchy.rs).
**Tags:** sprite, anchor, socket, runtime, hierarchy

---

## 1. Contexto — o ADR-0072 não tinha consumidor

O ADR-0072 está `Accepted` desde 2026-05-28. A §12 do Inspector, o gizmo de canvas e a persistência
foram construídos em 2026-08-21/22 — e, medido em 2026-08-22, **nada no app LIA uma âncora**. O
artista marcava a boca de uma arma, via a cruz no canvas, e não havia forma de prender coisa
nenhuma ali.

O §2.6 do ADR-mãe pede três superfícies (Rust, Luau, MCP), e nenhuma delas é o que faltava: as três
são formas de **perguntar** onde uma âncora está. O que faltava é o que **usa** a resposta.

## 2. Decisão

### 2.1 `AnchorMount` — o quadro do pai

```rust
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AnchorMount { pub anchor: String }
```

Uma entidade que monta numa âncora **já é filha** da entidade que a possui. O componente diz apenas
*qual* quadro do pai serve de origem: em vez da pose do pai, `compose(pose_do_pai, pose_da_âncora)`.
O `Transform` local do filho continua a ser dele, relativo a esse quadro — é o socket do Paper 2D.

Isto compra três coisas **de graça**, e são elas que escolhem o desenho:

1. **Ordem.** A propagação já visita o pai antes do filho ⇒ a âncora está resolvida quando o filho a
   pede. Sem ordenação topológica, sem segundo passe.
2. **Ciclos.** Impossíveis por construção: a hierarquia já os proíbe.
3. **Netos.** Quem monta pode ter filhos próprios, e eles herdam sem uma linha de código.

### 2.2 `MountState` tem TRÊS valores

`Free` · `Mounted(Transform)` · `Dangling`.

`Dangling` (o nome não está na lista do pai) comporta-se **geometricamente** como `Free` — o filho
fica no pai, não salta para a origem do mundo. Ele existe para poder ser **mostrado**: um `None`
confundir-se-ia com «esta entidade não monta em nada», e o artista veria a espada saltar sem ter o
que ler. O painel mostra o nome perdido e oferece a linha que o desfaz.

### 2.3 ⚠️ A lei entra nas DUAS travessias, pela mesma função

Este repositório responde «onde está esta entidade?» por dois caminhos, de propósito:
[`propagate_transforms`](../../../crates/ph2d-ecs/src/transform.rs) (DFS de cima para baixo, por
quadro, para o renderer) e
[`world_transform`](../../../crates/ph2d-ecs/src/transform_inverse.rs) (subida pela cadeia, sob
demanda, para gizmos, pick e física).

Um quadro de âncora injetado **só numa** delas faria a espada **desenhar** na mão e todo gesto —
clicar, arrastar, colidir — lê-la na origem do pai. O doc de `transform_inverse` já regista a
família (`docs/Physics/BUGS_physics.md` #2, medida a um offset de pai inteiro). Por isso a lei mora
numa função só (`mount_state`) e as duas travessias chamam-na, com o gate
`the_two_walks_agree_about_a_mounted_child` a prendê-las.

### 2.4 O nome, nunca o índice, nunca os bits

O componente guarda o **nome**. Um índice tornaria o vínculo dependente da ORDEM da lista — apagar a
âncora `0` faria toda a gente descer uma casa em silêncio. Os bits da entidade seriam pior: *o undo
respawna tudo com bits novos*, e bits dentro dos bytes de um componente envenenam o próprio undo
(lei do `stable_name_id`).

### 2.5 A API de runtime em Rust

```rust
pub fn anchor_world_pose(world: &World, entity: Entity, name: &str) -> Option<Transform>;
pub fn anchor_names(world: &World, entity: Entity) -> Vec<String>;
pub fn anchor_pose_under(owner_world: Transform, anchor: &NamedAnchor) -> Transform; // a lei pura
```

`anchor_pose_under` é a **única** lei de «onde está esta âncora». A montagem, o desenho da cruz no
canvas, as alças do gizmo e a API de runtime chamam-na todas. É uma linha de álgebra — e é
exatamente por ser uma linha que se reimplementa sem ninguém reparar.

### 2.6 `PROJECT_SCHEMA` 89 → 90

Quem obriga o bump é o **REGISTRO**, não um campo: um componente fora do `ComponentRegistry` é
descartado **em silêncio** pelo snapshot, e reabrir o projeto devolveria a espada como filha comum
do personagem — no sítio certo, parada. Ausência do componente é «não montar», que é o que toda
entidade fazia até v89, por isso todo arquivo ≤ v89 desenha byte-idêntico.

---

## 3. ⛔ As outras duas superfícies do §2.6 estão BLOQUEADAS, e não é escopo desta linha

Medido em 2026-08-22, antes de as construir:

| superfície | estado medido | o que falta |
|---|---|---|
| **Rust** | ✅ construída | — |
| **Luau** | ⛔ bloqueada | O `ScriptHost` do desktop arranca com um **script placeholder**; `provide_read` **nunca é chamado** na shell, não há UI para anexar um script a uma entidade, e o `ph2d.get` lê um `ReadSnapshot` que ninguém popula. Uma `ph2d.anchor` hoje seria API sobre um runtime que não corre. |
| **MCP** | ⛔ bloqueada | O `McpHost` é um **`MemoryHost`** de referência — `HashMap<u64, HashMap<String, Value>>`, com «Real backends (S2/S3) implementarão sobre bevy_ecs World direto» escrito no próprio doc. Um `sprite_anchor_get` leria JSON de um mapa, não uma âncora. |

⚠️ **Construir qualquer uma delas hoje seria repetir, um nível acima, o defeito que este amendment
cura**: autoria sem consumidor. As duas acordam quando a ponte respetiva existir — o `ScriptHost`
ligado à cena real, e o backend MCP sobre o `World`. O critério de fecho da W5 (ADR-0072 §5)
lê-se, para estes dois itens, como *bloqueado por outro subsistema*, não como *feito*.

---

## 4. Alternativas medidas e recusadas

### 4.1 Empurrar a pose (`RemoteTransform2D` do Godot) — rejeitada

Um sistema que escrevesse o `Transform` do filho a cada quadro. **Por que rejeitada:** registaria um
passo de undo **por quadro** — é exatamente a lei que o
[ADR-0153](0153-vector-auto-layout-is-taffy-behind-one-leaf-crate-and-the-pose-is-derived.md) pagou
no auto layout (*o passe publica onde as coisas ficam; ele não escreve onde elas estão*). E criaria
a segunda porta para «onde está o filho», que diverge da primeira no dia em que uma metade compõe e
a outra atribui cru.

### 4.2 Vínculo entre árvores, por nome — rejeitada

`AnchorMount { host: stable_name_id, anchor: String }`, resolvido num passe próprio.
**Por que rejeitada:** exigiria ordenação topológica sobre o grafo de vínculos, deteção de ciclos, e
um segundo passe de propagação para os sub-arbores dos vinculados. A hierarquia entrega as três
coisas de graça, e um socket que **não** é filho do que o possui é um desenho que nenhum dos três
motores de referência (Paper 2D, Aseprite, Construct) tem.

### 4.3 Entidade por âncora — rejeitada (já era recusa medida)

É a alternativa 4.2 do ADR-mãe, e continua rejeitada pela mesma razão: 50-100 âncoras × 100 sprites
seriam 5-10 k entidades, e a hierarquia — que é o que o artista lê — encher-se-ia de linhas que não
são objetos.

### 4.4 Guardar o índice da âncora — rejeitada

Ver §2.4.

---

## 5. Consequências

- **Positiva:** uma âncora passa a ser uma coisa que **move** outras. Toda a autoria da §12 (o
  gizmo, a lista, a persistência) ganha consumidor no mesmo dia.
- **Positiva:** o gizmo de âncora vira, de graça, uma ferramenta de posicionamento: arrastar a cruz
  move tudo o que monta nela, ao vivo, num passo de undo.
- **Negativa:** um filho que monta **tem** de ser filho. Prender a espada a uma mão de outro objeto
  na cena exige reparentar primeiro. Aceite: é o que os três motores de referência fazem.
- **Negativa:** `propagate_transforms` ganha uma leitura de componente por filho. Ela é `Option`
  sobre um archetype que a esmagadora maioria das entidades não tem.
- **Neutra:** `AnchorMount { anchor: "" }` é «não monta», e um sprite com ele desenha byte-idêntico
  a um que nunca o teve — a mesma decisão que o «Remove» de uma âncora toma.

---

## 6. Referências

- ADR-mãe: [ADR-0072](0072-named-anchor-unification.md).
- A lei das duas travessias: [`transform_inverse.rs`](../../../crates/ph2d-ecs/src/transform_inverse.rs) (doc de módulo) + `docs/Physics/BUGS_physics.md` #2.
- A lei do pose-derivada: [ADR-0153](0153-vector-auto-layout-is-taffy-behind-one-leaf-crate-and-the-pose-is-derived.md).
- Smoke: `PH2D_MOUNT_SMOKE=1` ([`mount_smoke.rs`](../../../shells/desktop/src/mount_smoke.rs)).
