//! Snapshot publication phase — once per frame, before paint.
//!
// Partido por assunto na OBRA 3 da `line/render-bodies` (2026-09-13): aqui ficam a `publish` (o orquestrador, pela ordem
// de sempre), a hierarquia e o passe do gizmo — o que os gates leem pelo caminho deste ficheiro —; o HUD, a caixa de sprite
// e a vista global do gizmo, e o Inspector moram nos filhos `hud`, `gizmo`, `inspector` e `inspector_sprite`.
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
use ph2d_editor_core::HeroScreen;
use ph2d_flip::FlipDoc;
use ph2d_host::WindowSize;
use ph2d_i18n::tr;
use ph2d_render::{Camera2d, Sprite};
use std::collections::BTreeMap;

/// A caixa de uma sprite e a vista global do gizmo — filho por assunto.
#[path = "snapshots_gizmo.rs"]
mod gizmo;
/// A grelha, a barra do Edit Prefab e o HUD — filho por assunto, num ficheiro irmão.
#[path = "snapshots_hud.rs"]
mod hud;
/// O Inspector da seleção — filho por assunto.
#[path = "snapshots_inspector.rs"]
mod inspector;
/// A secção Sprite do Inspector e os ajudantes dela — filho por assunto.
#[path = "snapshots_inspector_sprite.rs"]
mod inspector_sprite;

/// O player de um objecto preso por um cartão solto do Inspector: a vista e o veredito da lei (T61).
pub(super) struct Preso {
    pub(super) bits: u64,
    pub(super) live: Option<ph2d_physics_ecs::PlayerView>,
    pub(super) law: ph2d_physics_ecs::PlayerLiveness,
}

/// Walks PresentWorld + SimWorld to build the per-frame snapshots
/// and writes them onto the `HeroScreen`. Caller (orchestrator)
/// already holds the destructured `AppGfx` refs and the per-frame
/// EWMA stats; this is purely the publication logic.
#[allow(clippy::too_many_arguments)]
pub(super) fn publish(
    hero: &mut HeroScreen,
    hero_live: &mut Option<HeroLive>,
    // **O objecto sob o ponteiro**, já resolvido pela porta única (`App::hovered_object`).
    //
    // ⚠️ Ele chega RESOLVIDO e não como um ponteiro a picar aqui: o pick precisa do `App` (o mapa
    // vivo fundido, a câmara, o `view_state`), e esta função só tem os pedaços destruturados do
    // `AppGfx`. Passar a resposta em vez dos ingredientes é o que impede um segundo pick — com
    // outras entradas — de nascer neste ficheiro.
    hovered: Option<u64>,
    sim: &mut SimWorld,
    present: &mut PresentWorld,
    camera: &Camera2d,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    // ⭐ A taxonomia da biblioteca (wave A3) — publicada ao painel.
    catalogs: &ph2d_asset_index::CatalogTree,
    // As FOLHAS hand-packed da sessao, para a linha Storage nomear a regiao em vez de
    // mostrar dois indices crus (plano `docs/Sprite_projeto/17` §8).
    sheets: &BTreeMap<u32, ph2d_sprite_sheet::AuthoredSheet>,
    renderer: &ph2d_render::SpriteRenderer,
    window_size: WindowSize,
    // ⭐ **A vista está a ser conduzida pela câmera da cena?** (TOP-20 #7)
    //
    // ⚠️ **Estado da SHELL, e por isso viaja como argumento** — o painel não tem como o saber, e
    // derivá-lo do mundo seria impossível: ele não está no mundo.
    game_camera_preview: bool,
    preview_drive: &ph2d_preview_drive::PreviewDrive,
    last_pointer: (f32, f32),
    frame_ms_ewma: f32,
    frame_cpu_ms_ewma: f32,
    input_events: u32,
    paint_stamps: u32,
    paint_ms: f32,
    suppress_sprite_gizmo: bool,
    // **QUEM ESTÁ SOB PRÉ-VISUALIZAÇÃO DE FERRAMENTA** — a mesma lista de `run_render_frame` que o
    // tique da §11, o extract e o overlay da grelha leem. Aqui ela decide **em que disposição** a
    // folha está, para a caixa do gizmo a envolver inteira.
    tool_preview_bits: &[Option<u64>],
    // ADR-0111: uma forma vetorial também publica `GizmoView` — ela é um objeto com
    // `Transform`, e o gizmo que a manipula é o de sprite.
    vec_scene: &ph2d_vec_scene::VecScene,
    // ADR-0112: …mas NÃO enquanto uma ferramenta AUTORA no canvas. As alças do gizmo registam
    // hit-rects e comeriam o clique do gesto.
    //
    // ⚠️ **Ele vale para TODA família de objeto e para TODA ferramenta de autoria** — a vectorial
    // fora do Select dela, e a do Flip fora do Select dela. Ver a porta única em
    // `publish_gizmo::build_view`; o nome diz `object` por isso.
    object_gizmo_on: bool,
    // Os fatos DERIVADOS por frame (as poses do auto layout): sem eles a caixa do gizmo de um
    // filho colocado aparece onde a forma foi AUTORADA, e não onde a moldura a pôs.
    vec_view: &ph2d_vec_scene::VecViewState,
    // ADR-0114/ADR-0111: um objeto Flip TAMBÉM publica `GizmoView` (mesma caixa/pivô/rotação, da
    // bbox local da arte + `Transform`). ⚠️ O gate dele **não** vive aqui desde 2026-09-14: é o
    // `object_gizmo_on`, que é de toda família.
    flip: &FlipDoc,
    // W4: the `(start, end)` window in seconds the §11 Bake button would cover.
    // Resolved by the caller (which owns the clock) and shown ON the button —
    // see `physics_bake::bake_range`. Start is honoured now (W-BakeRange), so a
    // `[2s, 5s]` loop bakes exactly that.
    // ⭐ O relógio anda? — a secção FACTORY di-lo (TOP-20 #11).
    clock_playing: bool,
    bake_range: (f32, f32),
    // Which pose channels the §11 Bake selector shows as chosen (the shell's
    // transient `bake_channels`, a global bake option).
    bake_channels_tag: u8,
    // The kind the §11 join-kind selector shows as chosen (the shell's transient
    // `App.physics.join_kind`, the pending TYPE for the next *Join Selected Bodies*).
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
    // O que a LEI publicou sobre o player selecionado e o veredito do `pose_owner` (o que ela lê
    // dele), pela porta única da ponte (`publish` não a recebe) — e o mesmo de cada objecto preso
    // por um cartão solto do Inspector (T61).
    player_live: Option<ph2d_physics_ecs::PlayerView>,
    player_law: ph2d_physics_ecs::PlayerLiveness,
    presos: &[Preso],
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
    joint_anchor_handles: Vec<ph2d_editor_core::gizmo::PointHandle>,
    joint_anchor_snap: Option<[f32; 2]>,
    // **O SELO de cada linha da hierarquia** por bits de entidade (2026-08-22): o papel
    // que aquela forma tem dentro da booleana viva que a consome. Resolvido pelo caller,
    // que tem o `bool_live` em maos, e stampado aqui porque e' aqui que as linhas ainda
    // sao mutaveis -- o mesmo sitio, e o mesmo motivo, do `entry.selected`.
    bool_badges: &std::collections::BTreeMap<u64, &'static str>,
    // ⭐ **O registo de componentes** (ADR-0164 / F5) — a seção COMPONENT nomeia os componentes
    // overridados, e só o registo traduz `type_id → nome de exibição`. ⚠️ Sem ele a seção teria
    // de guardar uma tabela de nomes: uma SEGUNDA resposta a *«como se chama este
    // componente?»*, que divergiria do rótulo do botão que o anexa.
    component_registry: &ph2d_ecs::scene::ComponentRegistry,
    // ⭐⭐⭐ **A ÁRVORE DE TAGS do projecto** (TOP-20 #9) — a lista que a secção *Tags* oferece.
    //
    // ⚠️ **Ela viaja como o `vec_scene` e o `flip`, e pela MESMA razão: não está no mundo.** É um
    // documento irmão, e o painel não tem como lá chegar — derivar a lista dos ids que os objectos
    // carregam daria só as tags JÁ usadas, e a caixa de escolha deixaria de saber oferecer as
    // outras (nem de existir num projecto ainda sem nenhum objecto marcado).
    tags: &ph2d_tags::TagTree,
    // ⭐ Ver o parâmetro homónimo do [`inspector::publish`] (TOP-20 #14).
    projectile_over: &[u64],
    // ⭐⭐⭐ O que cada RAIO vê (suplente #21) — ver o parâmetro homónimo do publicador.
    ray_hits: &[(u64, u64, f32)],
    // ⭐ A Animação por objecto — quem tem clips (o 🎞) e quais (o conteúdo da linha).
    timeline: &ph2d_timeline::TimelineState,
) {
    #[cfg(feature = "panel-hierarchy")]
    publish_hierarchy(
        hero,
        hero_live,
        hovered,
        sim,
        bool_badges,
        component_registry,
        timeline,
    );
    hud::publish(
        hero,
        sim,
        present,
        camera,
        window_size,
        frame_ms_ewma,
        frame_cpu_ms_ewma,
        input_events,
        paint_stamps,
        paint_ms,
    );
    publish_gizmo(
        hero,
        sim,
        present,
        camera,
        window_size,
        last_pointer,
        tool_preview_bits,
        vec_scene,
        object_gizmo_on,
        vec_view,
        flip,
        joint_anchor_handles,
        joint_anchor_snap,
        join_draw_armed,
    );
    gizmo::global_view(hero, sim, last_pointer, suppress_sprite_gizmo);
    // §14 Platform Player: `Some` para todo corpo Dynamic (o botão dele faz o comportamento
    // existir). A corrida gravada é um facto do documento, em segundos pelo MESMO passo fixo.
    let recorded_run_seconds = (player_tape_ticks as f64 * fixed_dt) as f32;
    let discarded_run_seconds = (discarded_run_ticks as f64 * fixed_dt) as f32;
    inspector::publish_global(hero, sim, asset_db, atlas_asset_map, catalogs, tags);
    let publica = |sim: &mut SimWorld, alvo: inspector::Alvo, live, law| {
        let player = alvo.bits.and_then(|b| {
            let (rec, disc) = (recorded_run_seconds, discarded_run_seconds);
            ph2d_app_physics::inspector::player::build_player_info(sim, b, rec, disc, live, law)
        });
        inspector::publish(
            hero,
            &alvo,
            sim,
            asset_db,
            atlas_asset_map,
            sheets,
            renderer,
            game_camera_preview,
            preview_drive,
            clock_playing,
            bake_range,
            bake_channels_tag,
            join_kind_tag,
            joint_body_pick,
            joint_paste_targets,
            wheel_body_pick,
            wheel_rope_pick,
            join_draw_armed,
            component_registry,
            player,
            tags,
            projectile_over,
            ray_hits,
        );
    };
    // T61: cada objecto preso por um cartão solto, pelas mesmas portas; a seleção fica por último.
    let pins: Vec<u64> = presos.iter().map(|p| p.bits).collect();
    ph2d_panel_inspector::com_cada_preso(&pins, |b| {
        let p = presos.iter().find(|p| p.bits == b);
        let (live, law) = p.map_or((None, ph2d_physics_ecs::PlayerLiveness::INERT), |p| {
            (p.live, p.law)
        });
        publica(sim, inspector::Alvo::preso(b), live, law);
    });
    publica(
        sim,
        inspector::Alvo::da_selecao(hero),
        player_live,
        player_law,
    );
}

// A regra da autoria de folha mora no filho `inspector_sprite`; os testes dela seguem neste módulo, pelo nome.
#[cfg(test)]
use inspector_sprite::sheet_authorship;

#[cfg(test)]
#[path = "snapshots_sheet_authorship_tests.rs"]
mod sheet_authorship_tests;

/// A porta única da caixa de objecto (`publish_gizmo::build_view`) — o gate mora ao lado da lei.
#[cfg(test)]
#[path = "snapshots_object_gizmo_tests.rs"]
mod object_gizmo_tests;

/// A caixa de uma sprite com SKEW é o paralelogramo desenhado (onda 5, item a).
#[cfg(test)]
#[path = "snapshots_skew_gizmo_tests.rs"]
mod skew_gizmo_tests;

/// O passe do gizmo: a poda dos mortos, a view da seleção primária e das extras (o `build_view`), o número do
/// arrasto e o gizmo de ponto das juntas. Os gates que leem este passe pelo caminho do ficheiro leem-no aqui.
///
/// M14.7 B: a seleção que ainda resolve para uma entidade presente (pode ter sido apagada entre
/// quadros) ganha uma `GizmoView` da bbox no mundo + câmera; seleção vazia limpa a view.
/// M14.7 (parent-fix): o gizmo lê o `GlobalTransform` do `PresentWorld`, não o `Transform` local do
/// `SimWorld` — depois de um reparent a sprite desenha-se em `parent.world ∘ local` (o extract lê o
/// global) e o gizmo fugiria dela pelo deslocamento do pai. O `size` local da `Sprite` continua a vir
/// do `SimWorld` (o rect de autor), multiplicado pela escala do mundo como no `RenderInstance`.
#[allow(clippy::too_many_arguments)]
fn publish_gizmo(
    hero: &mut HeroScreen,
    sim: &SimWorld,
    present: &mut PresentWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    last_pointer: (f32, f32),
    tool_preview_bits: &[Option<u64>],
    vec_scene: &ph2d_vec_scene::VecScene,
    object_gizmo_on: bool,
    vec_view: &ph2d_vec_scene::VecViewState,
    flip: &FlipDoc,
    joint_anchor_handles: Vec<ph2d_editor_core::gizmo::PointHandle>,
    joint_anchor_snap: Option<[f32; 2]>,
    join_draw_armed: bool,
) {
    // Whether the Pivot transform tool is the active radio selection —
    // captured as a Copy bool so the gizmo-view closure (which can't
    // re-borrow `hero`) can emphasize the pivot dot.
    let pivot_tool_active = hero.store.button_state(ph2d_editor_core::ids::TOOL_PIVOT)
        == Some(ph2d_editor_core::widget::ButtonState::Pressed);
    // Captured Copy so the closure (which can't re-borrow `hero`) can
    // resolve the same effective anchor the extract stamps — keeping the
    // selection box aligned with the rendered quad under centered/offset.
    let gizmo_ppm = hero.project.pixels_per_meter;
    // Uma só conta mundo→vista para a primária, as extras e a união; sem `Sprite`, a forma vetorial
    // (ADR-0111). O bit da folha lê-se ANTES: o closure não pode emprestar `hero`.
    let sheet_gizmo_bits: Option<u64> =
        crate::render_loop::sim_extract_sheet::previewed(hero).map(|e| e.to_bits());
    let build_view = |bits: u64,
                      sim: &SimWorld,
                      present: &mut PresentWorld|
     -> Option<ph2d_editor_core::GizmoView> {
        // ⭐ ADR-0112: nenhuma família publica caixa de objecto com uma ferramenta AUTORA no canvas —
        // as alças roubariam o clique (a seleção fica armada). O caso do esqueleto e do Flip: o ADR.
        if !object_gizmo_on {
            return None;
        }
        let sim_entity = ph2d_ecs::Entity::from_bits(bits);
        if sim.world().get::<Sprite>(sim_entity).is_none() {
            // Não é sprite: uma forma vetorial ou um objeto Flip — cada um lê o
            // gizmo de sprite da sua bbox local + `Transform`.
            if sim
                .world()
                .get::<ph2d_ecs::VecPathRef>(sim_entity)
                .is_some()
            {
                // (O SPINE de um Blend não publica gizmo — o `vec_gizmo_view::view` o pula, como
                // faz com o conector. ADR-0128.)
                return ph2d_app_vec::vec_gizmo_view::view(
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
                // ⛔ Aqui morava o TERCEIRO `if` por família (`if !flip_gizmo_on`) — a porta única
                // acima já o diz, e para TODA família (2026-09-14).
                return ph2d_app_flip::gizmo_view::view(
                    sim,
                    flip,
                    sim_entity,
                    camera,
                    window_size,
                    last_pointer,
                    pivot_tool_active,
                );
            }
            // O Pixel Sprite (ADR-0196): a caixa da grelha à régua do projecto, com o `Transform`.
            if let Some((centro, half)) = ph2d_app_pixel::draw::caixa_local(bits, gizmo_ppm) {
                return Some(ph2d_app_vec::vec_gizmo_view::gizmo_view_from(
                    centro,
                    half,
                    ph2d_vec_entities::transform::world_transform(sim, sim_entity),
                    camera,
                    window_size,
                    last_pointer,
                    pivot_tool_active,
                ));
            }
            // ADR-0129 Fatia 3: o contentor de um Envelope tem a caixa-união dos filhos (a porta acima
            // gateia o Envelope: no Node aparece a gaiola).
            let union = ph2d_app_vec::vec_gizmo_view::container_view(
                sim,
                vec_scene,
                sim_entity,
                camera,
                window_size,
                last_pointer,
                pivot_tool_active,
            );
            if union.is_some() {
                return union;
            }
            // ⭐ **O GRUPO e o VAZIO** (Enio, 2026-08-26) — até aqui isto era `None`, e um
            // objeto sem gizmo não é agarrável por gesto nenhum: o objeto que o botão `Add` da
            // Hierarquia acabou de criar era o único do app que o artista não podia pegar. A caixa
            // é a união dos filhos VISÍVEIS (ou o marcador do vazio), e a lei mora em
            // `group_gizmo_view`, que é onde ela tem gate.
            return crate::group_gizmo_view::view(
                sim,
                sim_entity,
                camera,
                window_size,
                last_pointer,
                pivot_tool_active,
                gizmo_ppm,
            );
        }
        gizmo::sprite_view(
            bits,
            sim_entity,
            sim,
            present,
            camera,
            window_size,
            last_pointer,
            pivot_tool_active,
            gizmo_ppm,
            sheet_gizmo_bits,
            tool_preview_bits,
        )
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
    // O gizmo de PONTO: as âncoras de cada junta, sem depender da seleção (a regra vive no
    // `point_gizmo`, testada sem `HeroScreen`); o `join_draw_armed` torna-as inertes (W-J2b, W-J4).
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
}

#[path = "snapshots_hierarchy.rs"]
mod hierarquia;
use hierarquia::publish_hierarchy;
