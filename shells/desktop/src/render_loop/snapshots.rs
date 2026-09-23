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
) {
    #[cfg(feature = "panel-hierarchy")]
    publish_hierarchy(hero, hero_live, hovered, sim, bool_badges);
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
    // §14 Platform Player (W5) — a quarta da família. Ao contrário da §12/§13,
    // ela TEM face vazia: `Some` para todo corpo Dynamic, com ou sem o
    // componente, porque o botão dela é o que faz o comportamento existir.
    // ⚠️ **A corrida gravada entra por FORA do mundo** (W17): ela é um fato do
    // documento, não desta entidade, e é o único número da §14 que não sai do
    // componente. Segundos, medidos com o MESMO passo fixo que gravou os tiques.
    let recorded_run_seconds = (player_tape_ticks as f64 * fixed_dt) as f32;
    let discarded_run_seconds = (discarded_run_ticks as f64 * fixed_dt) as f32;
    let inspector_player = hero.gizmo.selection.and_then(|b| {
        ph2d_app_physics::inspector::player::build_player_info(
            sim,
            b,
            recorded_run_seconds,
            discarded_run_seconds,
            player_live,
            player_law,
        )
    });
    inspector::publish(
        hero,
        sim,
        asset_db,
        atlas_asset_map,
        catalogs,
        sheets,
        renderer,
        window_size,
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
        inspector_player,
        tags,
        projectile_over,
        ray_hits,
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

/// O passe do gizmo: a poda dos mortos, a view da seleção primária e das extras (o `build_view`), o número do
/// arrasto e o gizmo de ponto das juntas. Os gates que leem este passe pelo caminho do ficheiro leem-no aqui.
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
    let pivot_tool_active = hero.store.button_state(ph2d_editor_core::ids::TOOL_PIVOT)
        == Some(ph2d_editor_core::widget::ButtonState::Pressed);
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
    // ⚠️ Lido ANTES do closure: ele captura um `u64` simples, e não `&hero` — que seria emprestado
    // outra vez, mutavelmente, quando a view é escrita logo abaixo.
    let sheet_gizmo_bits: Option<u64> =
        crate::render_loop::sim_extract_sheet::previewed(hero).map(|e| e.to_bits());
    let build_view = |bits: u64,
                      sim: &SimWorld,
                      present: &mut PresentWorld|
     -> Option<ph2d_editor_core::GizmoView> {
        // ⭐⭐⭐ **NENHUMA FAMÍLIA PUBLICA CAIXA DE OBJECTO FORA DO SELECT DA FERRAMENTA VECTORIAL**
        // (ADR-0112) — UMA porta, e não um `if` por família. A razão é a que aquele ADR já escreve:
        // *as alças registam hit-rects, e uma caixa sobre o canvas de um modo de autoria é um
        // ladrão de cliques*. ⚠️ A selecção fica ARMADA — só a caixa e as alças somem —, e é isso
        // que mantém o *Bind to Skeleton* com sujeito dentro do modo Osso.
        //
        // ⛔⛔ **A lei estava escrita para a forma vectorial e para o envelope, e a SPRITE e o
        // GRUPO ficavam de fora — o buraco era invisível porque a sprite que vive debaixo de um
        // gesto de autoria não existia.** Em 2026-09-13 a imagem presa ao esqueleto passou a emitir
        // instância (plano `docs/Skeleton/03`, W2) e a `gizmo::sprite_view` passou a devolver
        // caixa: seleccionada, o `ids::GIZMO_BBOX_INTERIOR` que o gizmo regista cobre o braço
        // inteiro, o `on_canvas` do despacho fica **falso** em cima dele, o
        // `ramo_ferramenta_vetorial` nem corre e **nenhum osso por cima da arte que ele deforma
        // podia ser apontado ou posado** (report do dono: *«selecionar o osso não é mais
        // possível»*). *Uma caixa que nasce onde o rig vive engole o rig.*
        //
        // ⭐⭐ **E o GÉMEO DO FLIP fechou no dia seguinte (2026-09-14), sem report — medido:** o
        // `flip_gizmo_on` gateava só a arte do Flip, logo com a ferramenta Flip a desenhar um
        // objecto de OUTRA família (uma sprite de referência, um grupo, uma forma) continuava a
        // publicar caixa — e o `ramo_flip_premidos` exige o MESMO `on_canvas` que o ramo vectorial.
        // ⇒ a condição desta porta passou a ser *«nenhuma ferramenta AUTORA no canvas»*, e o
        // terceiro `if` por família morreu com ela. ⛔ **A saída não é isentar o gizmo no hit-test**
        // (o que o Painter faz, por ter porta própria): ali a caixa continua PINTADA, e uma alça
        // pintada que não pega é o controlo morto que este repo nomeia.
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
            // ADR-0129 Fatia 3: o container de um Envelope é um grupo SEM path próprio, mas TEM
            // gizmo — a caixa-união dos filhos, para o gizmo de sprite mover/girar/escalar o
            // envelope inteiro (Fatia 2). A porta acima já o gateia (Select; no Node aparece a
            // gaiola, não a caixa).
            if sim
                .world()
                .get::<ph2d_ecs::VecEnvelope>(sim_entity)
                .is_some()
            {
                return ph2d_app_vec::vec_gizmo_view::container_view(
                    sim,
                    vec_scene,
                    sim_entity,
                    camera,
                    window_size,
                    last_pointer,
                    pivot_tool_active,
                );
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
}

#[path = "snapshots_hierarchy.rs"]
mod hierarquia;
use hierarquia::publish_hierarchy;
