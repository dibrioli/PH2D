//! Snapshot publication phase — once per frame, before paint.
//!
// ph2d-loc-cap: accreted one producer per W3 Inspector section
// (sprite/transform/visibility/ordering/sampling/name + §8 visibility-
// section). Was already AT the 600-LOC ceiling before §8; +7 LOC for the
// §8 producer tips it. Follow-up: lift the per-section producers into
// their sibling `inspector_*` modules (build_* already live there) and
// leave this file as the thin publish orchestrator.
//!
//! Wave 3.2 stage A — extracted from `render_loop::mod.rs` as a free
//! function taking explicit refs to the destructured `AppGfx` fields
//! it needs. Behavior-preserving lift.
//!
//! Publishes the live hierarchy snapshot, grid view, telemetry stats,
//! gizmo projection, and 4 inspector snapshots
//! (sprite/transform/visibility/name) onto the `HeroScreen` so the
//! subsequent paint pass reads them via the HR-8 / ADR-0021 boundary
//! (Inspector never reads SimWorld directly).

use crate::HeroLive;
use ph2d_asset::AssetDb;
use ph2d_asset::AssetId;
use ph2d_ecs::{Name, PresentWorld, SimRef, SimWorld, Transform, Visibility};
use ph2d_editor::HeroScreen;
use ph2d_flip::FlipDoc;
use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, Sprite};
use std::collections::BTreeMap;

/// BulkSelect (T2.0): compute which editable `Sprite` fields diverge
/// across the `selected` entities, relative to `primary`. Exact equality
/// is intentional — "Mixed" means the stored values literally differ, so
/// editing the field would stomp the divergence. `selected` includes the
/// primary (a no-op self-compare); unknown / non-sprite entities are
/// skipped. Returns all-`false` for a single selection.
#[allow(clippy::float_cmp)] // exact compare: same stored value = not mixed
fn compute_sprite_mixed(
    world: &ph2d_ecs::World,
    selected: &[u64],
    primary: &Sprite,
) -> ph2d_editor::InspectorSpriteMixed {
    let mut m = ph2d_editor::InspectorSpriteMixed::default();
    for &bits in selected {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let Some(s) = world.get::<Sprite>(entity) else {
            continue;
        };
        m.flip_x |= s.flip_x != primary.flip_x;
        m.flip_y |= s.flip_y != primary.flip_y;
        m.tint_fill |= s.tint_fill != primary.tint_fill;
        m.centered |= s.centered != primary.centered;
        m.region_enabled |= s.region_enabled != primary.region_enabled;
        m.region_filter_clip |= s.region_filter_clip != primary.region_filter_clip;
        m.opacity |= s.opacity != primary.opacity;
        m.hframes |= s.hframes != primary.hframes;
        m.vframes |= s.vframes != primary.vframes;
        m.frame |= s.frame != primary.frame;
        m.offset_x |= s.offset[0] != primary.offset[0];
        m.offset_y |= s.offset[1] != primary.offset[1];
        m.region_x |= s.region_rect[0] != primary.region_rect[0];
        m.region_y |= s.region_rect[1] != primary.region_rect[1];
        m.region_w |= s.region_rect[2] != primary.region_rect[2];
        m.region_h |= s.region_rect[3] != primary.region_rect[3];
        m.tint |= s.tint != primary.tint;
        m.self_tint |= s.self_tint != primary.self_tint;
        m.per_corner |= s.per_corner_tint != primary.per_corner_tint;
    }
    m
}

/// **A divergência de EMISSÃO** — comparada à parte porque ela não vive no `Sprite`.
///
/// ⚠️ `SpriteEmissive` é um componente OPCIONAL, e a sua ausência **é** `EMISSIVE_OFF`: uma sprite
/// sem o componente e outra com `0.0` concordam. Comparar `Option<&SpriteEmissive>` diretamente
/// diria que divergem, e o chip branquearia sobre duas sprites que emitem exatamente o mesmo nada.
fn emissive_of(world: &ph2d_ecs::World, entity: ph2d_ecs::Entity) -> f32 {
    world
        .get::<ph2d_ecs::SpriteEmissive>(entity)
        .map_or(ph2d_ecs::EMISSIVE_OFF, |e| e.clamped())
}

fn compute_emissive_mixed(world: &ph2d_ecs::World, selected: &[u64], primary: f32) -> bool {
    selected
        .iter()
        .any(|&bits| emissive_of(world, ph2d_ecs::Entity::from_bits(bits)) != primary)
}

/// Walks PresentWorld + SimWorld to build the per-frame snapshots
/// and writes them onto the `HeroScreen`. Caller (orchestrator)
/// already holds the destructured `AppGfx` refs and the per-frame
/// EWMA stats; this is purely the publication logic.
#[allow(clippy::too_many_arguments)]
pub(super) fn publish(
    hero: &mut HeroScreen,
    hero_live: &mut Option<HeroLive>,
    sim: &mut SimWorld,
    present: &mut PresentWorld,
    camera: &Camera2d,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    // As FOLHAS hand-packed da sessao, para a linha Storage nomear a regiao em vez de
    // mostrar dois indices crus (plano `docs/Sprite_projeto/17` §8).
    sheets: &BTreeMap<u32, ph2d_sprite_sheet::AuthoredSheet>,
    renderer: &ph2d_render::SpriteRenderer,
    window_size: WindowSize,
    last_pointer: (f32, f32),
    frame_ms_ewma: f32,
    frame_cpu_ms_ewma: f32,
    input_events: u32,
    paint_stamps: u32,
    paint_ms: f32,
    suppress_sprite_gizmo: bool,
    // ADR-0111: uma forma vetorial também publica `GizmoView` — ela é um objeto com
    // `Transform`, e o gizmo que a manipula é o de sprite.
    vec_scene: &ph2d_vec_scene::VecScene,
    // ADR-0112: …mas NÃO nos modos de desenho/edição de nós. As alças do gizmo
    // registram hit-rects e comeriam o clique da âncora.
    vec_gizmo_on: bool,
    // Os fatos DERIVADOS por frame (as poses do auto layout): sem eles a caixa do gizmo de um
    // filho colocado aparece onde a forma foi AUTORADA, e não onde a moldura a pôs.
    vec_view: &ph2d_vec_scene::VecViewState,
    // ADR-0114/ADR-0111: um objeto Flip TAMBÉM publica `GizmoView` (mesma caixa/pivô/
    // rotação, da bbox local da arte + `Transform`), fora dos modos Draw/Erase da
    // tool Flip (senão o gizmo comeria o clique do canvas).
    flip: &FlipDoc,
    flip_gizmo_on: bool,
    // W4: the `(start, end)` window in seconds the §11 Bake button would cover.
    // Resolved by the caller (which owns the clock) and shown ON the button —
    // see `physics_bake::bake_range`. Start is honoured now (W-BakeRange), so a
    // `[2s, 5s]` loop bakes exactly that.
    bake_range: (f32, f32),
    // Which pose channels the §11 Bake selector shows as chosen (the shell's
    // transient `bake_channels`, a global bake option).
    bake_channels_tag: u8,
    // The kind the §11 join-kind selector shows as chosen (the shell's transient
    // `App.join_kind`, the pending TYPE for the next *Join Selected Bodies*).
    join_kind_tag: u8,
    // The armed §12 joint-body eyedropper `(joint_bits, slot_b)`, so the waiting
    // slot's picker paints pressed. Owned by the shell (`App.joint_body_pick`).
    joint_body_pick: Option<(u64, bool)>,
    // W-JointCopy: quantos joints um Paste atingiria agora — `0` quando a área
    // de transferência está vazia, e é isso que tira o botão da tela. Resolvido
    // pelo chamador, que é dono da área de transferência E da seleção.
    joint_paste_targets: usize,
    // W17: quantos tiques de CORRIDA GRAVADA o documento carrega — `0` quando
    // ninguém correu, e é isso que tira o botão *Clear Recorded Run* da tela. A
    // fita é do shell (`App.player_tape`), como a área de transferência acima; o
    // passo fixo que a converte em segundos é o `fixed_dt` do parâmetro seguinte.
    player_tape_ticks: usize,
    discarded_run_ticks: usize,
    // O passo fixo do relógio, para o número acima virar SEGUNDOS pela mesma
    // régua com que os tiques foram gravados.
    fixed_dt: f64,
    // `W-PlayerOut` A3: o que a LEI publicou sobre o player selecionado no
    // último tique. Resolvido pelo chamador pelo mesmo motivo das âncoras de
    // joint — `publish` não recebe a ponte —, e vindo da porta ÚNICA
    // (`PhysicsBridge::player_view`): uma segunda derivação aqui descreveria um
    // personagem que a simulação não simulou.
    player_live: Option<ph2d_physics_ecs::PlayerView>,
    // **O veredito do `pose_owner` sobre o player selecionado** — o que a lei de
    // facto lê dele. Resolvido pelo chamador pelo mesmo motivo do `player_live`
    // acima, e pela MESMA porta que decide quem escreve a pose: uma segunda
    // derivação aqui (do `PlayerMode`) é o que fazia a §14 oferecer controles
    // que ninguém consome.
    player_law: ph2d_physics_ecs::PlayerLiveness,
    // W-Pulley W3: a §13 tem a mesma máquina, uma família adiante — o eyedropper
    // de montagem da ROLDANA armado, para que ele pinte pressed enquanto espera o
    // clique no corpo. Dono: `App.wheel_body_pick`.
    wheel_body_pick: Option<u64>,
    // W1: o pick de CORDA armado (o eyedropper da row Rope), e para qual roldana.
    // Dono: `App.wheel_rope_pick`.
    wheel_rope_pick: Option<u64>,
    // W-J4: o gesto de desenhar um joint está ARMADO (o botão pinta Pressed).
    join_draw_armed: bool,
    // W-J2/W-J2b: every grabbable joint anchor this frame, resolved through the
    // bridge's anchor door (`PhysicsBridge::joint_anchor_world`) — the SAME door
    // the A pivot is synced from, so no two dots can describe different frames.
    // Built in `point_gizmo::joint_anchor_handles` (which owns the rest-only
    // rule) because `publish` does not take the bridge. `joint_anchor_snap` is
    // the candidate a live drag has caught, for the crosshair.
    joint_anchor_handles: Vec<ph2d_editor::gizmo::PointHandle>,
    joint_anchor_snap: Option<[f32; 2]>,
    // **O SELO de cada linha da hierarquia** por bits de entidade (2026-08-22): o papel
    // que aquela forma tem dentro da booleana viva que a consome. Resolvido pelo caller,
    // que tem o `bool_live` em maos, e stampado aqui porque e' aqui que as linhas ainda
    // sao mutaveis -- o mesmo sitio, e o mesmo motivo, do `entry.selected`.
    bool_badges: &std::collections::BTreeMap<u64, &'static str>,
) {
    // M14.4a: if live-bridge enabled, rebuild HierarchySnapshot
    // from SimWorld + push into HeroScreen BEFORE paint. The
    // snapshot's DFS visit order = hierarchy panel display
    // order. ADR-0029 Phase C.2: the typed Hierarchy panel owns the
    // live-entries thread-local; we call into the panel crate
    // directly here (the shell already gates `panel-hierarchy` via
    // feature).
    #[cfg(feature = "panel-hierarchy")]
    if let Some(live) = hero_live.as_mut() {
        crate::build_hierarchy_snapshot(
            sim.world(),
            &mut live.walk_state,
            &mut live.walk_scratch,
            &mut live.snapshot,
        );
        let (ordered, mut entries) = live.bridge.sync_from_snapshot(&live.snapshot);
        // Fase 0 hotfix: mark every multi-selection row's
        // `HierarchyEntity.selected` BEFORE the panel paints, so
        // the row painter highlights N rows instead of just the
        // primary (paint.rs falls back to label match only when
        // `selected` is still false — fixture/demo path).
        for bits in hero.gizmo.iter_selected() {
            if let Some(node_id) = live.bridge.node_for(bits)
                && let Some(entry) = entries.get_mut(&node_id)
            {
                entry.selected = true;
            }
        }
        // Onda 1 hotfix: centralise the header label sync to the
        // multi-selection primary. Input handlers (canvas pick,
        // Hierarchy panel click, modifier override) used to stamp
        // hero.selection themselves and could race — e.g. Hierarchy
        // Cmd+click on row A stamped label="A" BEFORE the bus drain
        // toggled A out of the selection, leaving paint's label-match
        // fallback to re-highlight A. Snapshotting it once here
        // post-drain, against the post-toggle primary, removes the
        // race entirely.
        let primary_label = hero
            .gizmo
            .selection
            .and_then(|bits| live.bridge.node_for(bits))
            .and_then(|node| {
                entries
                    .get(&node)
                    .map(|e| (e.name.clone(), e.badge.clone()))
            });
        // **O SELO DO PAPEL BOOLEANO**, stampado DEPOIS do `primary_label` de propósito: o
        // cabeçalho usa o badge como *tipo* da seleção, e sobrescrevê-lo antes faria a
        // barra de cima dizer `SUB` onde sempre disse `ENT`. São dois consumidores do
        // mesmo campo, e só um deles pediu esta informação.
        if !bool_badges.is_empty() {
            for (&bits, &badge) in bool_badges {
                if let Some(node_id) = live.bridge.node_for(bits)
                    && let Some(entry) = entries.get_mut(&node_id)
                {
                    entry.badge = Some(badge.to_string());
                }
            }
        }
        ph2d_panel_hierarchy::sync_from_hierarchy(&mut hero.store, &ordered, entries);
        if let Some((label, badge)) = primary_label {
            hero.selection = Some(ph2d_editor::HeroSelection {
                label,
                kind: badge.unwrap_or_else(|| "ENT".to_string()),
                world_pos: (0.0, 0.0),
            });
        } else if hero.gizmo.selection.is_none() {
            hero.selection = None;
        }
    }
    // M14.4b: publish the demo camera + window dims so the
    // hero paints its world grid overlay. `canvas` is a
    // placeholder — `paint_hero_screen` overrides it with
    // the layout-computed canvas rect.
    // Motion Nodes drift fix (2026-07-25): sob o split da tool Motion a CENA renderiza num
    // sub-retângulo (present.rs, via `CenterSplit::scene_viewport`), mas a grade do mundo
    // projetava a janela CHEIA — as linhas não pousavam sobre os sprites/instâncias do
    // Motion. A grade usa as MESMAS dims da cena (a porta única); fora do split é a janela
    // cheia, byte-idêntico.
    let (grid_w, grid_h) = crate::field_gizmo::scene_window_wh(hero.view.center_split, window_size);
    hero.set_grid_view(Some(ph2d_editor::GridView {
        camera_center: camera.center,
        camera_height_world: camera.height_world,
        window_w: grid_w,
        window_h: grid_h,
        canvas: ph2d_editor::zones::Rect::new(0.0, 0.0, 0.0, 0.0),
    }));
    // M14.4g Telemetry Phase A: publish real stats. Sprite
    // and entity counts come from PresentWorld (the source of
    // truth for "what we shipped to the GPU this frame"); fps
    // is derived from the EWMA frame_ms.
    let sprite_count = present
        .world_mut()
        .query::<&ph2d_render::RenderInstance>()
        .iter(present.world_mut())
        .count() as u32;
    // ⚠️ **Sem os quads de 9-slice.** Os nove quads de um sprite fatiado partilham o `SimRef`
    // da entidade (é o que faz o carimbo de `z_order` servir os nove), por isso contá-los aqui
    // faria UMA caixa de diálogo aparecer no HUD como NOVE entidades — um número que passaria a
    // mentir exatamente quando a cena fica interessante. A contagem de INSTÂNCIAS acima sobe de
    // propósito: nove quads são nove quads, e isso é o que um contador de desenho deve dizer.
    let entity_count = present
        .world_mut()
        .query_filtered::<&SimRef, bevy_ecs::query::Without<ph2d_render::nine_slice::SlicePatchMirror>>()
        .iter(present.world_mut())
        .count() as u32;
    let fps = if frame_ms_ewma > 0.001 {
        1000.0 / frame_ms_ewma
    } else {
        0.0
    };
    // M14.7 polish (10.1): raw fps = inverse of pure
    // CPU/command-encode time. Floored at 1 ms (1000 fps) so
    // a startup-edge measurement of 0 doesn't blow up to
    // `inf`; real workloads stabilize within a few frames.
    let raw_fps = 1000.0 / frame_cpu_ms_ewma.max(0.001);
    // Diagnostics: wall-clock NOT in the CPU-encode window = present/vsync acquire stall PLUS any
    // between-frames input work — the gap that makes "Raw" rise while FPS falls (HANDOFF §1.R).
    let present_stall_ms = (frame_ms_ewma - frame_cpu_ms_ewma).max(0.0);
    hero.stats = ph2d_editor::BottomHudStats {
        fps,
        frame_ms: frame_ms_ewma,
        draws: 1,
        sprite_count,
        entity_count,
        raw_fps,
        present_stall_ms,
        paint_ms,
        input_events,
        paint_stamps,
    };
    // Hierarchy counts use PresentWorld's archetype components
    // (Transform + Sprite + Visibility + ChildOf + Children).
    // It's a proxy — exactly the components the editor's
    // snapshot pipeline observes per entity. Multiplying by
    // entity count is a rough estimate; counting via archetype
    // walk is cheap enough at editor scales.
    let component_count = {
        let world = sim.world();
        let mut total = 0u32;
        for archetype in world.archetypes().iter() {
            let len = archetype.len();
            let comps = archetype.components().len() as u32;
            total = total.saturating_add(len.saturating_mul(comps));
        }
        total
    };
    #[cfg(feature = "panel-hierarchy")]
    ph2d_panel_hierarchy::set_live_component_count(component_count);
    // M14.7 B: publish the gizmo's per-frame projection. When
    // the selection still resolves to a present entity (it can
    // vanish if the user deleted it between frames) we build a
    // `GizmoView` from the world-space bbox + camera. Empty
    // selection → clear the view so the painter skips.
    //
    // M14.7 polish (parent-fix): the gizmo MUST read
    // `GlobalTransform` from PresentWorld — not the entity's
    // local `Transform` in SimWorld. After a hierarchy reparent
    // the child's local Transform stays the same but its world
    // position is now parent.world ∘ local; the sprite renders
    // at the new world position via the extract path (which
    // reads GlobalTransform), so the gizmo has to do the same
    // or it drifts away from the sprite by exactly the parent's
    // world offset. The Sprite's local `size` is still pulled
    // from SimWorld — it's the import-time author rect,
    // multiplied here by the world scale extracted from the
    // matrix to match the renderer's RenderInstance build.
    // Whether the Pivot transform tool is the active radio selection —
    // captured as a Copy bool so the gizmo-view closure (which can't
    // re-borrow `hero`) can emphasize the pivot dot.
    let pivot_tool_active = hero.store.button_state(ph2d_editor::ids::TOOL_PIVOT)
        == Some(ph2d_editor::widget::ButtonState::Pressed);
    // Captured Copy so the closure (which can't re-borrow `hero`) can
    // resolve the same effective anchor the extract stamps — keeping the
    // selection box aligned with the rendered quad under centered/offset.
    let gizmo_ppm = hero.project.pixels_per_meter;
    // Onda 2: factor the per-sprite GizmoView build into a closure so
    // the primary, each extra, and the global union all share the
    // exact same world→view math. Single source of truth for the
    // affine decomposition + anchor compensation; any future render-
    // path tweak only touches this closure.
    // ADR-0111: sem `Sprite`, tenta a forma vetorial — mesma caixa, mesmo pivô,
    // mesma rotação, derivados da bbox local da curva e do `Transform` da entidade.
    let build_view =
        |bits: u64, sim: &SimWorld, present: &mut PresentWorld| -> Option<ph2d_editor::GizmoView> {
            let sim_entity = ph2d_ecs::Entity::from_bits(bits);
            if sim.world().get::<Sprite>(sim_entity).is_none() {
                // Não é sprite: uma forma vetorial ou um objeto Flip — cada um lê o
                // gizmo de sprite da sua bbox local + `Transform`.
                if sim
                    .world()
                    .get::<ph2d_ecs::VecPathRef>(sim_entity)
                    .is_some()
                {
                    if !vec_gizmo_on {
                        return None;
                    }
                    // (O SPINE de um Blend não publica gizmo — o `vec_gizmo_view::view` o pula, como
                    // faz com o conector. ADR-0128.)
                    return crate::vec_gizmo_view::view(
                        sim,
                        vec_scene,
                        vec_view,
                        sim_entity,
                        camera,
                        window_size,
                        last_pointer,
                        pivot_tool_active,
                    );
                }
                if sim
                    .world()
                    .get::<ph2d_ecs::FlipObjectRef>(sim_entity)
                    .is_some()
                {
                    if !flip_gizmo_on {
                        return None;
                    }
                    return crate::flip_gizmo_view::view(
                        sim,
                        flip,
                        sim_entity,
                        camera,
                        window_size,
                        last_pointer,
                        pivot_tool_active,
                    );
                }
                // ADR-0129 Fatia 3: o container de um Envelope é um grupo SEM path próprio, mas TEM
                // gizmo — a caixa-união dos filhos, para o gizmo de sprite mover/girar/escalar o
                // envelope inteiro (Fatia 2). Gate no mesmo `vec_gizmo_on` (Select; no Node aparece a
                // gaiola, não a caixa).
                if sim
                    .world()
                    .get::<ph2d_ecs::VecEnvelope>(sim_entity)
                    .is_some()
                {
                    if !vec_gizmo_on {
                        return None;
                    }
                    return crate::vec_gizmo_view::container_view(
                        sim,
                        vec_scene,
                        sim_entity,
                        camera,
                        window_size,
                        last_pointer,
                        pivot_tool_active,
                    );
                }
                return None; // grupo/outro: sem gizmo próprio
            }
            let sprite = sim.world().get::<Sprite>(sim_entity)?;
            let mut q = present
                .world_mut()
                .query::<(&SimRef, &ph2d_ecs::GlobalTransform)>();
            let gt = q.iter(present.world()).find_map(|(sref, gt)| {
                if sref.0 == sim_entity {
                    Some(*gt)
                } else {
                    None
                }
            })?;
            let affine = gt.affine();
            let col0_x = affine[0];
            let col0_y = affine[1];
            let col1_x = affine[2];
            let col1_y = affine[3];
            let scale_x = (col0_x * col0_x + col0_y * col0_y).sqrt();
            let scale_y = (col1_x * col1_x + col1_y * col1_y).sqrt();
            let rotation = col0_y.atan2(col0_x);
            let p = gt.translation();
            let half_w = sprite.size[0] * scale_x * 0.5;
            let half_h = sprite.size[1] * scale_y * 0.5;
            // Effective anchor (folds centered/offset) so the box tracks
            // the rendered quad, not just the raw tool pivot.
            let eff_anchor = sprite.resolve_anchor(gizmo_ppm);
            let ax = eff_anchor[0] * scale_x;
            let ay = eff_anchor[1] * scale_y;
            // T1.3.5 cross-OS bit-identical.
            let (sin_r, cos_r) = libm::sincosf(rotation);
            let cx = p.x + ax * cos_r - ay * sin_r;
            let cy = p.y + ax * sin_r + ay * cos_r;
            Some(ph2d_editor::GizmoView {
                bbox_min_world: [cx - half_w, cy - half_h],
                bbox_max_world: [cx + half_w, cy + half_h],
                pivot_world: [p.x, p.y],
                pivot_tool_active,
                rotation,
                camera_center: camera.center,
                camera_height_world: camera.height_world,
                window_w: window_size.width as f32,
                window_h: window_size.height as f32,
                canvas: ph2d_editor::zones::Rect::new(
                    0.0,
                    0.0,
                    window_size.width as f32,
                    window_size.height as f32,
                ),
                cursor_screen: Some(last_pointer),
            })
        };
    // Poda ANTES de construir as views: só a morte de uma entidade tira alguém da
    // seleção (ver `gizmo_prune` — o atalho "sem view = morreu" expulsava as
    // entidades vetoriais, que não têm `Sprite`).
    super::gizmo_prune::prune_dead(&mut hero.gizmo, sim);
    // Onda 2: rebuild the views every frame, from the pruned selection. An entity
    // with no `Sprite` simply has no view — it stays selected and paints no gizmo.
    hero.gizmo.view = hero
        .gizmo
        .selection
        .and_then(|bits| build_view(bits, sim, present));
    // **O NÚMERO do arrasto** (C3) — ao lado da view, e pelo mesmo motivo: os dois descrevem o
    // gesto em curso e são reconstruídos do mundo a cada quadro. Ver `gizmo_readout`.
    super::gizmo_readout::publish(hero, sim, camera, window_size);
    // The POINT gizmo — every joint's anchors. A joint has a `Transform` but no
    // box (so `build_view` returns None for it); these are the handles it gets,
    // and they are NOT selection-gated: a joint has no sprite to pick, so a
    // selection-gated handle is reachable only by finding the joint in the
    // Hierarchy first (W-J2b). The "which anchors get one" rule lives in
    // `point_gizmo` so it is gated headless (the publish here needs a live
    // HeroScreen the test cannot build).
    // ⚠️ `join_draw_armed` chega aqui pela SEGUNDA vez de propósito: ele pinta o
    // botão Pressed no §11 e torna estas alças inertes, e é o MESMO fato — durante
    // o gesto de desenhar, as âncoras já postas ficam à vista e fora de alcance.
    hero.gizmo.point_view = super::point_gizmo::build_point_view(
        joint_anchor_handles,
        camera,
        window_size,
        joint_anchor_snap,
        join_draw_armed,
    );
    hero.gizmo.extra_views.clear();
    for bits in hero.gizmo.extra_selection.clone() {
        if let Some(v) = build_view(bits, sim, present) {
            // Cada par carrega os próprios bits, então uma alça nunca é registrada
            // sob a identidade de outro sprite (Enio 2026-06-08: "a 2ª e 3ª sprites
            // não giram") — e `extra_views` pode ser um subconjunto de
            // `extra_selection` sem desalinhar nada.
            hero.gizmo.extra_views.push((bits, v));
        }
    }
    // Onda 2 polish: while a Global gizmo drag is alive, derive the
    // global view from the cached `global_view_start` snapshot +
    // primary's transform deltas. This is what makes the global gizmo
    // **rotate visually** during a Global Rotate (and scale rigidly
    // during a Global Scale) instead of being the axis-aligned union
    // of rotated sprites — that union grows under rotation, which
    // would make the gizmo "balloon" rather than rotate.
    let global_from_drag = if let (Some(start), Some(drag)) = (
        hero.gizmo.global_view_start.as_ref().copied(),
        hero.gizmo.drag.as_ref().copied(),
    ) && matches!(drag.target, ph2d_editor::GizmoTarget::Global)
    {
        let primary_entity = ph2d_ecs::Entity::from_bits(drag.entity_bits);
        let world = sim.world();
        let (delta_rot, factor_x, factor_y) =
            if let Some(t) = world.get::<Transform>(primary_entity) {
                let dr = t.rotation - drag.start_transform.rotation;
                let fx = if drag.start_transform.scale[0].abs() > f32::EPSILON {
                    t.scale.x / drag.start_transform.scale[0]
                } else {
                    1.0
                };
                let fy = if drag.start_transform.scale[1].abs() > f32::EPSILON {
                    t.scale.y / drag.start_transform.scale[1]
                } else {
                    1.0
                };
                (dr, fx, fy)
            } else {
                (0.0, 1.0, 1.0)
            };
        let cx_s = (start.bbox_min_world[0] + start.bbox_max_world[0]) * 0.5;
        let cy_s = (start.bbox_min_world[1] + start.bbox_max_world[1]) * 0.5;
        let hw_s = (start.bbox_max_world[0] - start.bbox_min_world[0]) * 0.5;
        let hh_s = (start.bbox_max_world[1] - start.bbox_min_world[1]) * 0.5;
        // Onda 2 hotfix: global drags (Scale + Rotate) PIVOT around the
        // start centre. The primary's translation shifts as a side
        // effect of the rotation/scale, but the gizmo's centre stays
        // at the original pivot — using the primary's delta_translation
        // here was making the gizmo drift away from the sprites it
        // covers (smoke: "o desenho do gizmo não rotaciona corretamente
        // em seu centro causando um drift entre as sprites e o
        // desenho do gizmo"). Global has no Translate handle (we
        // dropped BBOX_INTERIOR for keyed gizmos), so this branch only
        // sees Scale + Rotate.
        let new_cx = cx_s;
        let new_cy = cy_s;
        let new_hw = hw_s * factor_x.abs();
        let new_hh = hh_s * factor_y.abs();
        Some(ph2d_editor::GizmoView {
            bbox_min_world: [new_cx - new_hw, new_cy - new_hh],
            bbox_max_world: [new_cx + new_hw, new_cy + new_hh],
            pivot_world: [new_cx, new_cy],
            pivot_tool_active: false,
            rotation: delta_rot,
            camera_center: start.camera_center,
            camera_height_world: start.camera_height_world,
            window_w: start.window_w,
            window_h: start.window_h,
            canvas: start.canvas,
            cursor_screen: Some(last_pointer),
        })
    } else {
        None
    };
    // Onda 2: global view = union of every selected sprite's bbox,
    // EXPANDED by a fixed screen offset so the global gizmo's handles
    // sit clear of the individual gizmos' handles (Enio: "o gizmo da
    // multiseleção com offset em relação aos gizmos individuais para
    // não conflitar as alças de manipulação"). 32 px in screen space,
    // converted to world units at the current zoom so the offset
    // tracks the zoom level — handles stay one handle-size + a gap
    // outside the individuals at any scale.
    // Conta VIEWS, não bits selecionados: uma seleção de 1 sprite + 1 path
    // vetorial (ADR-0110) tem `selected_len() == 2` mas uma view só, e o gizmo
    // global desenharia — deslocado 32 px — em volta de um sprite sozinho.
    let painted_views = usize::from(hero.gizmo.view.is_some()) + hero.gizmo.extra_views.len();
    hero.gizmo.global_view = if let Some(v) = global_from_drag {
        Some(v)
    } else if painted_views > 1 {
        let primary = hero.gizmo.view.as_ref();
        let mut iter = primary
            .into_iter()
            .chain(hero.gizmo.extra_views.iter().map(|(_, v)| v));
        iter.next().map(|first| {
            let mut min_x = first.bbox_min_world[0];
            let mut min_y = first.bbox_min_world[1];
            let mut max_x = first.bbox_max_world[0];
            let mut max_y = first.bbox_max_world[1];
            for v in iter {
                min_x = min_x.min(v.bbox_min_world[0]);
                min_y = min_y.min(v.bbox_min_world[1]);
                max_x = max_x.max(v.bbox_max_world[0]);
                max_y = max_y.max(v.bbox_max_world[1]);
            }
            let pixel_to_world = first.camera_height_world / first.window_h.max(1.0);
            let offset_world = 32.0 * pixel_to_world;
            ph2d_editor::GizmoView {
                bbox_min_world: [min_x - offset_world, min_y - offset_world],
                bbox_max_world: [max_x + offset_world, max_y + offset_world],
                pivot_world: [(min_x + max_x) * 0.5, (min_y + max_y) * 0.5],
                pivot_tool_active: false,
                rotation: 0.0,
                camera_center: first.camera_center,
                camera_height_world: first.camera_height_world,
                window_w: first.window_w,
                window_h: first.window_h,
                canvas: first.canvas,
                cursor_screen: first.cursor_screen,
            }
        })
    } else {
        None
    };
    // While the Painter's Deform **Transform** gizmo is live (Uniform / Free / Distort / Warp), the
    // SPRITE gizmo is fully suppressed — view, extras and the global union. On a whole-image transform
    // both gizmos put their corner squares on the SAME screen corners, and a near-corner Down grabbed
    // the sprite's scale handle instead of the deform's (Enio 2026-07-04: "inative o gizmo da sprite
    // para as quatro ferramentas de Transform"). No view ⇒ nothing painted ⇒ no handle registered in
    // the hit index ⇒ every corner click reaches the deform gizmo.
    if suppress_sprite_gizmo {
        hero.gizmo.view = None;
        hero.gizmo.extra_views.clear();
        hero.gizmo.global_view = None;
    }
    // M14.5 inspector phase (6.4/§9): publish a per-frame
    // snapshot of the selected sprite so `paint_inspector` can
    // surface the Render Source section + Reimport button
    // without crossing the ADR-0021 boundary into SimWorld.
    // BulkSelect (T2.0): the full selection (primary + extras). Only
    // collected (one alloc) for a MULTI-selection — single-select (the
    // common case) takes the empty path and skips the Mixed compare.
    let selected_count = hero.gizmo.selected_len();
    let inspector_selection: Vec<u64> = if selected_count > 1 {
        hero.gizmo.iter_selected().collect()
    } else {
        Vec::new()
    };
    let inspector_sprite = hero.gizmo.selection.and_then(|bits| {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let world = sim.world();
        let sprite = world.get::<Sprite>(entity)?;
        let transform = world.get::<Transform>(entity)?;
        // ⚠️ A emissão é lida pela mesma porta que a comparação usa (`emissive_of`), senão as duas
        // metades — o valor mostrado e a decisão de o mostrar — poderiam discordar sobre o que
        // «ausente» significa.
        let emissive = emissive_of(world, entity);
        let mut mixed = if inspector_selection.len() > 1 {
            compute_sprite_mixed(world, &inspector_selection, sprite)
        } else {
            ph2d_editor::InspectorSpriteMixed::default()
        };
        if inspector_selection.len() > 1 {
            mixed.emissive = compute_emissive_mixed(world, &inspector_selection, emissive);
        }
        let (source_kind, source_pixels, can_reimport) = match sprite.source {
            ph2d_render::SpriteSource::Atlas { key } => {
                // ⚠️ `image_dimensions` e não um `match` na variante — ver o irmão em
                // `inspector_commits.rs`. Aqui a falha silenciosa era o tamanho da sprite sumir do
                // Inspector (plano `docs/Sprite_projeto/18`, auditoria da W2).
                let dims = atlas_asset_map
                    .get(&key)
                    .and_then(|aid| asset_db.get(aid).and_then(|a| a.image_dimensions()));
                (
                    ph2d_editor::InspectorSpriteSource::Atlas { key },
                    dims,
                    dims.is_some(),
                )
            }
            ph2d_render::SpriteSource::Individual { texture_id } => {
                // Source dims come from the renderer's individual-texture
                // store (the bake's own size) so the Region UI can show
                // "Source W×H" and seed `region_rect` to the full source —
                // the extract already supports Individual region sampling.
                let dims = renderer.individual().dims(texture_id);
                // ⚠️ **A ESTRATÉGIA é uma pergunta de AUTORIA, não de armazenamento.** No
                // armazenamento um sprite hand-packed É uma textura individual com um retângulo
                // — é essa composição que faz o extract não precisar de saber que ele existe
                // (plano `docs/Sprite_projeto/17` §2.1). Quem sabe de que FOLHA ele é, é o
                // `SpriteSheetRef`, e por isso o painel pergunta ao componente, não ao `source`.
                match world.get::<ph2d_ecs::SpriteSheetRef>(entity) {
                    Some(r) => (
                        ph2d_editor::InspectorSpriteSource::HandPacked {
                            sheet: r.sheet,
                            region: r.region,
                        },
                        dims,
                        false,
                    ),
                    None => (
                        ph2d_editor::InspectorSpriteSource::Individual { texture_id },
                        dims,
                        // Reimport recomputes world size from an Atlas asset's
                        // px/m; Individual bakes have no atlas asset to re-decode.
                        false,
                    ),
                }
            }
            // W2.T2: a cooked KTX2 source — read-only display marker. Dims
            // come from the W2.T4 loader (logical_id → tier asset); unknown
            // here, so the Region UI shows no "Source W×H" and no reimport.
            ph2d_render::SpriteSource::CookedTexture { .. } => (
                ph2d_editor::InspectorSpriteSource::CookedTexture,
                None,
                false,
            ),
        };
        // **ESTAR NUMA FOLHA JÁ É SER HAND-PACKED** (Enio, 2026-08-19: *"ao colocar uma imagem
        // numa sheet, no inspector ainda diz que ela usa a estratégia Individual"*).
        //
        // ⚠️ **O modelo já concordava com ele e o código não seguia** — a nota logo acima diz que
        // *"a estratégia é uma pergunta de AUTORIA, não de armazenamento"*, e a autoria estava a
        // ser lida só do `SpriteSheetRef`, que **nasce no bake**. Entre pôr a peça na folha e
        // assá-la, o painel dizia `Individual` — que é verdade sobre os PIXELS e mentira sobre o
        // que o artista acabou de fazer. *Uma resposta correta à pergunta errada lê-se como um
        // bug, e é.*
        //
        // A autoria passa a ter duas fontes, na ordem em que se tornam verdadeiras: o
        // `SpriteSheetRef` (assado — sabe a região) e, na falta dele, **ser filho de uma folha**
        // (arranjado — ainda não sabe). O rótulo diz qual das duas é.
        // **A precisão MEDIDA, não derivada** (plano `docs/Sprite_projeto/18` W5). O store de
        // texturas é quem sabe: o Inspector recebe o facto pronto, como já recebe o `sheet_label`.
        //
        // ⚠️ Uma célula de atlas é `Rgba8UnormSrgb` por construção; uma individual pode ser
        // qualquer das duas, e é por isso que se **pergunta** em vez de assumir.
        let source_precision = match sprite.source {
            ph2d_render::SpriteSource::Atlas { .. } => Some(ph2d_color::Precision::Rgba8),
            ph2d_render::SpriteSource::Individual { texture_id } => {
                renderer.individual_format(texture_id).map(|f| {
                    if f == ph2d_render::IndividualTextureStore::FORMAT_16 {
                        ph2d_color::Precision::Rgba16
                    } else {
                        ph2d_color::Precision::Rgba8
                    }
                })
            }
            // Cozida: BC/ASTC/ETC2, e o formato concreto depende do tier resolvido.
            ph2d_render::SpriteSource::CookedTexture { .. } => None,
        };
        let unbaked_sheet = if matches!(
            source_kind,
            ph2d_editor::InspectorSpriteSource::Individual { .. }
                | ph2d_editor::InspectorSpriteSource::Atlas { .. }
        ) {
            world
                .get::<ph2d_ecs::ChildOf>(entity)
                .map(|c| c.parent())
                .filter(|p| world.get::<ph2d_ecs::SpriteSheetFrame>(*p).is_some())
                .map(|p| {
                    world
                        .get::<ph2d_ecs::Name>(p)
                        .map(|n| n.0.clone())
                        .unwrap_or_else(|| "Sprite Sheet".to_string())
                })
        } else {
            None
        };
        // O rótulo legível de uma origem hand-packed. Derivado AQUI (e não no painel) porque o
        // painel é chrome e não pode depender do documento de folhas sem inverter a seta.
        let baked_label = match source_kind {
            ph2d_editor::InspectorSpriteSource::HandPacked { sheet, region } => {
                sheets.get(&sheet).and_then(|s| {
                    s.region(region)
                        .map(|r| format!("{} \u{00b7} {}", s.name, r.name))
                })
            }
            _ => None,
        };
        let (source_kind, sheet_label) =
            sheet_authorship(source_kind, unbaked_sheet.as_deref(), baked_label);
        let world_size = [
            sprite.size[0] * transform.scale.x,
            sprite.size[1] * transform.scale.y,
        ];
        Some(ph2d_editor::InspectorSpriteInfo {
            sheet_label,
            entity_bits: bits,
            world_size,
            source_kind,
            source_precision,
            // **Quanto esta sprite emite** (plano `docs/Sprite_projeto/18` W8). Ausente = `0.0`:
            // para o painel, «sem componente» e «componente a zero» são a mesma coisa, e é isso que
            // deixa o slider voltar a zero remover a linha em vez de a deixar morta no ficheiro.
            emissive,
            source_pixels,
            can_reimport,
            flip_x: sprite.flip_x,
            flip_y: sprite.flip_y,
            opacity: sprite.opacity,
            tint_fill: sprite.tint_fill,
            hframes: sprite.hframes,
            vframes: sprite.vframes,
            frame: sprite.frame,
            tint: sprite.tint,
            self_tint: sprite.self_tint,
            per_corner_tint: sprite.per_corner_tint,
            region_enabled: sprite.region_enabled,
            region_rect: sprite.region_rect,
            region_filter_clip: sprite.region_filter_clip,
            centered: sprite.centered,
            offset: sprite.offset,
            selected_count,
            mixed,
        })
    });
    // M14.A: live Transform snapshot for the inspector. Same
    // ADR-0021 / HR-8 boundary as sprite snapshot — Inspector
    // never reads SimWorld; the host bridges. Lands on every
    // entity that has a `Transform` component, not just sprites
    // (so non-renderable entities still show their pose).
    let inspector_transform = hero.gizmo.selection.and_then(|bits| {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let t = sim.world().get::<Transform>(entity)?;
        Some(ph2d_editor::InspectorTransformInfo {
            entity_bits: bits,
            translation: [t.translation.x, t.translation.y],
            rotation_rad: t.rotation,
            scale: [t.scale.x, t.scale.y],
            skew_rad: [t.skew_x, t.skew_y],
        })
    });
    // M14.D: live Visibility snapshot. Absence-equals-visible
    // is the canonical invariant — entities without a
    // `Visibility` component render normally, so `None` from
    // `world.get::<Visibility>` maps to `visible = true`.
    // Only published when the selection has a `Transform`
    // (i.e. it's an Inspector-worthy entity); without a
    // Transform the Inspector hides the whole panel content.
    let inspector_visibility = hero.gizmo.selection.and_then(|bits| {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        sim.world().get::<Transform>(entity)?;
        let visible = sim
            .world()
            .get::<Visibility>(entity)
            .map(|v| !v.hidden)
            .unwrap_or(true);
        // ⚠️ A ausência de `Visibility` É visível — a mesma invariante que a leitura acima usa, e
        // por isso a comparação passa pela mesma expressão. Compará-las como `Option` diria que
        // uma sprite sem componente diverge de outra com `hidden: false`, e as duas estão visíveis.
        let mixed = inspector_selection.iter().any(|&other| {
            let e = ph2d_ecs::Entity::from_bits(other);
            sim.world()
                .get::<Visibility>(e)
                .map(|v| !v.hidden)
                .unwrap_or(true)
                != visible
        });
        Some(ph2d_editor::InspectorVisibilityInfo {
            entity_bits: bits,
            visible,
            mixed,
        })
    });
    // M14.E: live `Name` snapshot. Falls back to
    // `Entity_{hex}` when the entity has no Name component
    // yet — matches the existing `InspectorSpriteInfo::name`
    // shape. Same Transform-presence gate.
    let inspector_name = hero.gizmo.selection.and_then(|bits| {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        sim.world().get::<Transform>(entity)?;
        let name = sim
            .world()
            .get::<Name>(entity)
            .map(|n| n.0.clone())
            .unwrap_or_else(|| format!("Entity_{bits:x}"));
        Some(ph2d_editor::InspectorNameInfo {
            entity_bits: bits,
            name,
        })
    });
    let sel = &inspector_selection; // W3 §7/§9 snapshots (§7 sibling module)
    let inspector_ordering = hero.gizmo.selection.and_then(|b| {
        super::inspector_ordering::build_ordering_info(sim.world(), b, sel, selected_count)
    });
    let inspector_sampling = hero.gizmo.selection.and_then(|b| {
        super::inspector_ordering::build_sampling_info(sim.world(), b, sel, selected_count)
    });
    let inspector_blend = hero.gizmo.selection.and_then(|b| {
        super::inspector_ordering::build_blend_info(sim.world(), b, sel, selected_count)
    });
    // §5 9-Slice. ⚠️ Publicado para TODA entidade digna de Inspector, com ou sem o componente:
    // é o snapshot que diz `present: false`, e é isso que faz a seção mostrar o «+ Add 9-Slice».
    // Publicar só quando o componente existe faria a seção aparecer depois de a feature estar
    // ligada — ou seja, nunca, porque não haveria por onde ligá-la.
    let inspector_slice = hero.gizmo.selection.and_then(|b| {
        super::inspector_slice::build_slice_info(sim.world(), b, sel, selected_count)
    });
    // §12 Sockets / Named Anchors (ADR-0072). Publicado para toda entidade digna de Inspector —
    // é o snapshot que diz `present: false`, e é isso que faz a seção mostrar o «+ Add Anchor».
    let inspector_anchor = hero.gizmo.selection.and_then(|b| {
        super::inspector_anchor::build_anchor_info(
            sim.world(),
            b,
            sel,
            selected_count,
            hero.project.pixels_per_meter,
        )
    });
    // W3: the Join gesture needs exactly TWO bodies, and only the shell can
    // see the selection — the panel is handed one entity at a time. Asked once
    // here, so the painter (which offers the button) and the event handler
    // (which honours the click) read the same fact.
    // ⚠️ `sel.len() == 2`, not just `selected_count == 2`: `all()` on an empty
    // slice is TRUE, so a count that disagreed with the slice would offer the
    // button over nothing at all.
    // ⚠️ **`>= 2`, not `== 2`** (W-J4): three or more selected bodies make a
    // CHAIN of `n − 1` joints, and the count travels to the panel so the button
    // can SAY so. A label that says "Join Selected Bodies" over a five-body
    // selection is how an artist discovers a chain by accident.
    let join_count = if sel.len() >= 2
        && selected_count == sel.len()
        && sel.iter().all(|&b| {
            let e = ph2d_ecs::Entity::from_bits(b);
            sim.world().get::<ph2d_physics_ecs::RigidBody>(e).is_some()
                && sim.world().get::<ph2d_physics_ecs::Collider>(e).is_some()
        }) {
        u8::try_from(sel.len()).unwrap_or(u8::MAX)
    } else {
        0
    };

    // W-Rig: quantas PARTES um clique em *Rig* tocaria — 0 quando não há aresta
    // pai→filho a ligar, e é esse zero que tira o botão da tela.
    //
    // ⚠️ **Da `iter_selected()`, NÃO do `inspector_selection`** — aquele vetor é
    // colhido só numa MULTI-seleção (`selected_count > 1`) e fica vazio no caso
    // único, que é precisamente o gesto do rig: marcar a raiz do personagem e
    // clicar. Lido dali, o botão só apareceria com dois objetos marcados, isto é
    // em toda situação menos a que ele existe para servir.
    let rig_parts = {
        let roots: Vec<u64> = hero.gizmo.iter_selected().collect();
        let plan = crate::joint_rig::plan(sim, &roots);
        if plan.is_offered() {
            u8::try_from(plan.parts.len()).unwrap_or(u8::MAX)
        } else {
            0
        }
    };

    // W-PartFace: quantas PEÇAS estão penduradas no objeto selecionado — filhos
    // que carregam `Collider` e não `RigidBody`.
    //
    // ⚠️ **Só a shell pode contar**, e é por isso que o número atravessa a
    // fronteira em vez de o painel o derivar: `ChildOf` é a única aresta do ECS,
    // então não há como DESCER a árvore — cada candidato tem de SUBIR até achar
    // um corpo, e a lista de candidatos vem de uma query sobre o mundo inteiro.
    // Mesma classe do `rig_parts` logo acima, e o mesmo custo por frame.
    let part_count = hero.gizmo.selection.map_or(0, |b| {
        let owner = ph2d_ecs::Entity::from_bits(b);
        let mut q = sim.world_mut().query_filtered::<ph2d_ecs::Entity, (
            bevy_ecs::query::With<ph2d_physics_ecs::Collider>,
            bevy_ecs::query::Without<ph2d_physics_ecs::RigidBody>,
        )>();
        let candidates: Vec<ph2d_ecs::Entity> = q.iter(sim.world()).collect();
        u8::try_from(ph2d_physics_ecs::count_parts(
            sim.world(),
            owner,
            candidates,
        ))
        .unwrap_or(u8::MAX)
    });

    let inspector_physics = hero.gizmo.selection.and_then(|b| {
        super::inspector_physics::build_physics_info(
            sim.world(),
            b,
            join_count,
            rig_parts,
            part_count,
            join_draw_armed,
            join_kind_tag,
            bake_range,
            bake_channels_tag,
        )
    });
    let inspector_joint = hero.gizmo.selection.and_then(|b| {
        // The eyedropper of the slot with an armed pick FOR THIS joint paints
        // pressed; 0 otherwise.
        let pick_armed = match joint_body_pick {
            Some((j, slot_b)) if j == b => {
                if slot_b {
                    2
                } else {
                    1
                }
            }
            _ => 0,
        };
        super::inspector_joint::build_joint_info(sim, b, pick_armed, joint_paste_targets)
    });
    // §13 Pulley Wheel (W-Pulley W1) — a irmã da §12, e a seleção é a MESMA
    // pergunta: uma roldana é uma entidade, então ela é o objeto selecionado.
    let inspector_wheel = hero.gizmo.selection.and_then(|b| {
        super::inspector_joint_wheel::build_wheel_info(
            sim,
            b,
            wheel_body_pick == Some(b),
            wheel_rope_pick == Some(b),
        )
    });
    // §14 Platform Player (W5) — a quarta da família. Ao contrário da §12/§13,
    // ela TEM face vazia: `Some` para todo corpo Dynamic, com ou sem o
    // componente, porque o botão dela é o que faz o comportamento existir.
    // ⚠️ **A corrida gravada entra por FORA do mundo** (W17): ela é um fato do
    // documento, não desta entidade, e é o único número da §14 que não sai do
    // componente. Segundos, medidos com o MESMO passo fixo que gravou os tiques.
    let recorded_run_seconds = (player_tape_ticks as f64 * fixed_dt) as f32;
    let discarded_run_seconds = (discarded_run_ticks as f64 * fixed_dt) as f32;
    let inspector_player = hero.gizmo.selection.and_then(|b| {
        super::inspector_player::build_player_info(
            sim,
            b,
            recorded_run_seconds,
            discarded_run_seconds,
            player_live,
            player_law,
        )
    });
    let inspector_visibility_section = hero.gizmo.selection.and_then(|b| {
        super::inspector_visibility::build_visibility_section_info(
            sim.world(),
            b,
            sel,
            selected_count,
        )
    });
    // ADR-0029 Phase C.1: publish snapshots to the panel crate's
    // thread-locals (replaces the pre-C.1 `hero.inspector.<field>`
    // writes — the field no longer exists; the panel-owned state +
    // its thread-local snapshot setters do).
    #[cfg(feature = "panel-inspector")]
    {
        ph2d_panel_inspector::set_current_inspector_sprite(inspector_sprite);
        ph2d_panel_inspector::set_current_inspector_ordering(inspector_ordering);
        ph2d_panel_inspector::set_current_inspector_sampling(inspector_sampling);
        ph2d_panel_inspector::set_current_inspector_blend(inspector_blend);
        ph2d_panel_inspector::set_current_inspector_slice(inspector_slice);
        ph2d_panel_inspector::set_current_inspector_anchor(inspector_anchor);
        ph2d_panel_inspector::set_current_inspector_physics(inspector_physics);
        ph2d_panel_inspector::set_current_inspector_joint(inspector_joint);
        ph2d_panel_inspector::set_current_inspector_wheel(inspector_wheel);
        ph2d_panel_inspector::set_current_inspector_player(inspector_player);
        ph2d_panel_inspector::set_current_inspector_visibility_section(
            inspector_visibility_section,
        );
        ph2d_panel_inspector::set_current_inspector_transform(inspector_transform);
        ph2d_panel_inspector::set_current_inspector_visibility(inspector_visibility);
        ph2d_panel_inspector::set_current_inspector_name(inspector_name);
        ph2d_panel_inspector::set_current_display_unit(
            hero.project.display_unit,
            hero.project.pixels_per_meter,
        );
    }
    #[cfg(not(feature = "panel-inspector"))]
    {
        let _ = (
            inspector_physics,
            inspector_sprite,
            inspector_transform,
            inspector_visibility,
            inspector_visibility_section,
            inspector_name,
        );
    }
}

/// **A autoria de folha que o painel MOSTRA** — a regra, isolada do mundo para poder ser testada.
///
/// Duas fontes, na ordem em que se tornam verdadeiras:
///
/// 1. **assado** (`SpriteSheetRef`) — o `storage` já chega como `HandPacked` e há uma região
///    nomeada; o rótulo é `folha · região`;
/// 2. **arranjado** (filho de uma folha, ainda sem `SpriteSheetRef`) — o armazenamento é mesmo
///    `Individual`/`Atlas`, mas a AUTORIA já é da folha. Mostra `HandPacked` com o rótulo a dizer
///    que ainda não foi assado.
///
/// ⚠️ **Os ids `0/0` do caso 2 não significam nada**, e é o rótulo (sempre presente aí) que
/// impede que alguém os leia: a linha `Storage` prefere-o, e só cai nos números quando ele falta —
/// o que neste caso não pode acontecer. *Um número sem significado é aceitável enquanto for
/// inalcançável; deixar de o ser é a regressão a vigiar.*
fn sheet_authorship(
    storage: ph2d_editor::InspectorSpriteSource,
    unbaked_sheet: Option<&str>,
    baked_label: Option<String>,
) -> (ph2d_editor::InspectorSpriteSource, Option<String>) {
    match unbaked_sheet {
        Some(name) => (
            ph2d_editor::InspectorSpriteSource::HandPacked {
                sheet: 0,
                region: 0,
            },
            Some(format!("{name} \u{00b7} not baked yet")),
        ),
        None => (storage, baked_label),
    }
}

#[cfg(test)]
mod sheet_authorship_tests {
    use super::sheet_authorship;
    use ph2d_editor::InspectorSpriteSource as S;

    /// ⚠️ **O caso que o Enio relatou.** A peça está na folha e ainda não foi assada: o
    /// armazenamento é mesmo `Individual`, mas a AUTORIA já é da folha — e é a autoria que a linha
    /// `Strategy` responde (a nota que já estava no `snapshots.rs` di-lo desde sempre; o código é
    /// que não a seguia).
    #[test]
    fn a_piece_dropped_into_a_sheet_reads_as_hand_packed() {
        let (kind, label) = sheet_authorship(S::Individual { texture_id: 7 }, Some("Fruits"), None);
        assert!(matches!(kind, S::HandPacked { .. }));
        assert_eq!(label.as_deref(), Some("Fruits \u{00b7} not baked yet"));
    }

    /// O mesmo para uma peça que ainda vive no atlas — pô-la na folha é o mesmo gesto.
    #[test]
    fn an_atlas_piece_in_a_sheet_reads_the_same() {
        let (kind, _) = sheet_authorship(S::Atlas { key: 3 }, Some("Fruits"), None);
        assert!(matches!(kind, S::HandPacked { .. }));
    }

    /// **Assado ganha do arranjado**, e não o contrário: quando há região nomeada, é ela que o
    /// artista quer ler (é por ela que ele reencontra o desenho no Aseprite).
    #[test]
    fn a_baked_piece_keeps_its_region_name() {
        let (kind, label) = sheet_authorship(
            S::HandPacked {
                sheet: 4,
                region: 2,
            },
            None,
            Some("hero \u{00b7} idle_0".into()),
        );
        assert!(matches!(
            kind,
            S::HandPacked {
                sheet: 4,
                region: 2
            }
        ));
        assert_eq!(label.as_deref(), Some("hero \u{00b7} idle_0"));
    }

    /// **Controle positivo:** fora de uma folha nada muda. Sem isto, a regra podia estar a
    /// devolver `HandPacked` para toda a gente e os testes acima passariam na mesma.
    #[test]
    fn a_sprite_outside_any_sheet_is_untouched() {
        let (kind, label) = sheet_authorship(S::Individual { texture_id: 7 }, None, None);
        assert!(matches!(kind, S::Individual { texture_id: 7 }));
        assert!(label.is_none());
        let (kind, _) = sheet_authorship(S::Atlas { key: 1 }, None, None);
        assert!(matches!(kind, S::Atlas { key: 1 }));
    }
}
