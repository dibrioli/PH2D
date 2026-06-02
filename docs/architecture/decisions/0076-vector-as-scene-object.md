# ADR-0076 — Vector como objeto de cena (Rank 10): entidade ECS + offset no render, sem desgelar o schema

**Status:** Accepted (2026-06-02)

**Contexto de origem:** `docs/HANDOFF_vector_w2_audit_fixes_coord.md` §4.1 (Rank 10 — "o
maior gap restante"). Decisão + ADR são Coord-only (CLAUDE.md §0.2 + §6: mexer no
modelo de cena = ADR). O Implementador Vector recomendou os dois caminhos abaixo e
pediu o ADR ("posso escrever o caminho B se você quiser — só não executo sozinho").

---

## 1. Contexto

Hoje um vetor "commitado" (`Ph2dVectorAsset`: `network` + `styles`) vive num **side-channel**
`App::committed_vector_pen_paths: Vec<Ph2dVectorAsset>` e é desenhado por
`vector_pen_bridge.rs` com **um único `world_to_screen` global** (vértices já estão em
world-coords). Consequências do "vetor não é entidade":

1. **Invisível na hierarquia** — `build_hierarchy_snapshot` (`crates/ph2d-ecs/src/scene/
   snapshot.rs`) só enxerga entidades `With<Transform>`; um `Vec` paralelo nunca aparece.
2. **Não pega no gizmo** — o pick do gizmo é `ph2d_render::pick_sprite_at_world`
   (`crates/ph2d-render/src/picking.rs`), que testa só sprites; não há caminho de pick
   para vetor, então o transform-gizmo nunca seleciona/move um vetor.
3. **Sem placement** — não há onde guardar "este vetor está em (x,y), rotacionado θ":
   os vértices SÃO o dado, em world-coords rest-pose.

**Restrição dura:** `Ph2dVectorAsset` é **contrato CONGELADO** (gate
`architecture_vector_contract_surface`, CLAUDE.md §6). Não podemos adicionar `transform`
ao asset sem amendment de contrato congelado + re-lock de cook-hash/persist.

**Invariantes que já jogam a favor (mapeadas na exploração 2026-06-02):**

| Peça | Arquivo | Fato que destrava |
|---|---|---|
| Gizmo write | `shells/desktop/src/input_dispatch/gizmo_drag.rs:~300` | `gfx.sim.world_mut().get_mut::<Transform>(entity)` — **genérico**, escreve `Transform` de QUALQUER entidade. Sprites não têm código especial no advance. |
| Hierarquia | `crates/ph2d-ecs/src/scene/snapshot.rs` (`roots: QueryState<Entity, (With<Transform>, Without<ChildOf>)>`) | Entidade com `Transform` (+ `Name` opcional) **aparece de graça** na árvore. |
| Hit-test vetor | `crates/ph2d-vector-doc/src/hit_test.rs` `region_contains_point(&self, region, p: Vec2) -> bool` | Point-in-polygon pronto, em world-coords. |
| Render | `shells/desktop/src/render_loop/vector_pen_bridge.rs` `draw_vector_network(scene, net, styles, world_to_screen: Affine)` | Aceita um `Affine` por chamada — dá pra compor um offset por-asset. |

---

## 2. Decisão

**Caminho (B): cada `Ph2dVectorAsset` commitado ganha UMA entidade ECS `(Transform, Name,
VectorSceneRef)` na SimWorld. O `Transform` é um overlay de PLACEMENT — os vértices ficam
em rest-pose world-coords; o render compõe `entity.Transform ∘ world_to_screen`, e o pick
inverte o `Transform` antes do `region_contains_point`. NÃO se desgela o schema.**

Rejeitamos (A) "adicionar `transform` ao `Ph2dVectorAsset`" — ver §4.

### 2.1 Refinamento-chave vs. a recomendação original (SimWorld-direto, sem tocar o boundary sim/present)

A recomendação inicial mirrorava a entidade para a **PresentWorld** (como sprites:
`SimRef`+`GlobalTransform`) e fazia pick/render lendo de lá. **Não é necessário no MVP** e
seria uma mudança no boundary de extract (ADR-0021). Porque:

- O gizmo escreve `Transform` na **SimWorld** (`gizmo_drag.rs`), que é a fonte autoritativa.
- A hierarquia lê a **SimWorld**.
- O render e o pick rodam **shell-side** e já têm `gfx.sim` em mão.

Logo, render + pick leem o `Transform` da entidade pareada **direto da SimWorld**. Para um
vetor **raiz** (sem pai), `Transform` local == world — não precisamos de propagação de
`GlobalTransform`. Isso elimina a mudança de extract e reduz o blast-radius a **shell + 1
componente**. Reparenting/agrupar vetor (que exigiria `GlobalTransform` propagado) é
incremento futuro (§2.7).

### 2.2 Novo componente — `VectorSceneRef` (contrato deste ADR)

```rust
/// Liga uma entidade de cena ao `Ph2dVectorAsset` commitado correspondente.
/// `asset` é a chave estável do asset (id do asset, NÃO o índice no Vec — índices
/// rotacionam em remoção). A entidade carrega também `Transform` (placement) + `Name`.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct VectorSceneRef {
    pub asset: u64,
}
```

- **Local v1:** shell (`shells/desktop`) — é cola editor↔cena; a SimWorld (bevy_ecs) aceita
  qualquer `#[derive(Component)]`. Promover para `ph2d-ecs` (p/ ícone de vetor na
  hierarquia) é trivial e fica para quando a hierarquia ganhar ícone por-tipo.
- **Chave `asset: u64`:** o `Ph2dVectorAsset` precisa de um id estável. Se ainda não houver
  um, a v1 usa o **id da entidade** como chave canônica e mantém um `BiMap entity↔índice`
  no shell; preferir um `asset.id` real assim que o schema expuser um (aditivo, não-congela).

### 2.3 Ciclo de vida (spawn/despawn) — a única lógica nova de estado

Um mapa shell-side `vector_scene: Vec<(Entity)>` paralelo a `committed_vector_pen_paths`
(mesmo índice), mantido em DOIS pontos:

- **Commit** (`vector_pen_bridge` drena `pen.take_committed_asset()`): `push` no Vec **e**
  `gfx.sim.world_mut().spawn((Transform::IDENTITY, Name::new("Vector N"), VectorSceneRef{..}))`;
  guarda a `Entity`.
- **Remoção** de um asset (undo de commit / delete): `despawn` da `Entity` pareada.

Invariante: `committed_vector_pen_paths.len() == vector_scene.len()` e índices alinhados.
Um teste de unidade trava esse invariante no ponto de spawn/despawn.

### 2.4 Render — compor o placement (`vector_pen_bridge.rs`)

```rust
let world_to_screen = camera.world_to_screen_affine(window_size);
for (i, asset) in committed_paths.iter().enumerate() {
    let placement = sim.world().get::<Transform>(vector_scene[i]).copied()
        .unwrap_or(Transform::IDENTITY);
    let composed = world_to_screen * placement.affine(); // placement aplicado ANTES de world→screen
    draw_vector_network(scene, &asset.network, &asset.styles, composed);
}
```

`Transform::IDENTITY` ⇒ bit-idêntico ao comportamento atual (zero regressão para vetores não
movidos).

### 2.5 Pick — `pick_vector_at_world` (shell helper, espelha o de sprite)

```rust
/// Topmost vetor sob `world_pos`, ou None. Inverte o placement de cada entidade antes
/// do hit-test (os vértices estão em rest-pose; o `Transform` deslocou o visual).
fn pick_vector_at_world(sim, scene: &[Entity], assets: &[Ph2dVectorAsset], world_pos: Vec2)
    -> Option<u64> {
    for (entity, asset) in scene.iter().zip(assets).rev() {       // rev = topo primeiro
        let placement = sim.world().get::<Transform>(*entity).copied().unwrap_or(IDENTITY);
        let local = placement.inverse().transform_point(world_pos); // world→rest-pose
        if asset.network.regions.iter()
            .any(|r| asset.network.region_contains_point(r, local)) {
            return Some(entity.to_bits());
        }
    }
    None
}
```

Roteamento no MouseDown (`input_dispatch.rs`, ao lado do `pick_sprite_at_world`):
`let bits = pick_sprite_at_world(...).or_else(|| pick_vector_at_world(...));
 if let Some(b) = bits { hero.gizmo.replace_selection(Some(b)); }`.
Ambos retornam **sim entity bits** ⇒ seleção unificada; o gizmo segue daí sem código novo.

### 2.6 O que NÃO muda (de propósito)

- Schema `Ph2dVectorAsset` — intocado (gate congelado verde).
- Boundary sim/present (ADR-0021) — intocado (render/pick leem SimWorld).
- Gizmo advance/write — intocado (já genérico).
- `region_contains_point` — intocado (consumido como está).

### 2.7 Fora de escopo (incrementos futuros, sequenciados)

1. **Reparent/agrupar vetor** → exige `GlobalTransform` propagado (ler world, não local) no
   render/pick. Aí sim talvez valha o mirror para PresentWorld.
2. **Ícone "vetor" na hierarquia** → promover `VectorSceneRef` para `ph2d-ecs` + caso no
   snapshot.
3. **Persistência do placement** → serializar `Transform` por-asset na cena (v2 da
   persistência de cena, não do asset).
4. **`asset.id` estável** no schema (aditivo) para a chave de `VectorSceneRef`.

---

## 3. Consequências

### 3.1 Positivas
- **Vetor vira cidadão de primeira classe da cena** (hierarquia + gizmo) reusando 100% da
  infra de sprite — zero código novo no gizmo, na hierarquia ou no hit-test.
- **Schema congelado intacto** — sem amendment, sem re-lock de cook-hash, gate verde.
- **Blast-radius mínimo** — shell + 1 componente; o boundary sim/present não se mexe.
- **Regressão zero** para vetores não movidos (`IDENTITY` ⇒ render bit-idêntico).

### 3.2 Negativas
- **Estado duplicado** — o `Vec` de assets e a entidade de cena precisam ficar em sincronia
  (spawn/despawn). Mitigado por invariante testada (§2.3).
- **Pick em mundo diferente do sprite** — sprite pega na PresentWorld, vetor na SimWorld.
  Aceitável (ambos devolvem sim-bits); a divergência some se/quando o mirror chegar (§2.7).
- **Placement não persiste na v1** — mover um vetor e salvar não preserva a posição até o
  incremento §2.7.3. Documentar na UI/handoff.

### 3.3 Neutras
- `VectorSceneRef` nasce shell-side; promovê-lo a foundational é um move trivial e aditivo.

---

## 4. Alternativas consideradas

- **(A) `transform` no `Ph2dVectorAsset`** (vértices viram local-coords). Mais "correto"
  conceitualmente, mas **desgela contrato congelado** (amendment + re-lock cook-hash +
  migração de todos os assets/persist + toca o gate). Custo desproporcional para o ganho;
  e a v1 de placement nem precisa que o dado do asset mude. **Rejeitado** (reabrir só se
  reparenting/persistência exigir vértices locais — improvável; o `Transform` da cena cobre).
- **(B') Path B com mirror para PresentWorld** (recomendação original). Correto, mas adiciona
  uma regra de extract (boundary ADR-0021) sem necessidade para o MVP raiz-only. **Adiado**
  para o incremento de reparenting (§2.7.1), onde o `GlobalTransform` propagado se paga.
- **(C) Overlay puramente visual sem entidade** (lista de gizmos sintéticos). Não aparece na
  hierarquia, não reusa seleção/gizmo, vira código paralelo a manter. **Rejeitado.**

---

## 5. Plano de execução (sequenciado — coordenar com o Impl Vector p/ evitar colisão em `vector_pen_bridge`/`input_dispatch`)

1. `VectorSceneRef` + `vector_scene: Vec<Entity>` no shell; spawn/despawn no commit/remoção
   (+ teste do invariante de sincronia).
2. Render compose em `vector_pen_bridge` (`IDENTITY` ⇒ no-op verificável).
3. `pick_vector_at_world` + roteamento no MouseDown (`.or_else` após o pick de sprite).
4. Smoke do Enio: vetor aparece na hierarquia, clica, gizmo move, solta.

**Colisão:** `vector_pen_bridge.rs` + `input_dispatch.rs` são tocados pelo Impl Vector
(W2). Executar numa janela em que o Impl esteja parado nesses arquivos, ou delegar a
execução ao Impl SOB este ADR. **A decisão (este ADR) está fechada; a execução é
coordenada.**
