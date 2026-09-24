//! ⭐⭐⭐ **A ORDEM DAS SECÇÕES DO INSPECTOR É A DA PALETA QUE AS ACRESCENTA.**
//!
//! ⛔⛔⛔ **Report do dono, 2026-09-21:** *«várias seções muito confusas e desorganizadas»*.
//! Medido pelo `y` PINTADO (a sonda `diag_em_que_ordem_as_seccoes_aparecem`): o Inspector tem
//! **38** secções, e as **22** opcionais estavam na ordem em que foram CONSTRUÍDAS — `Tags`, que
//! todo objecto pode ter, era a **38.ª e última**, a `14 785 px` do topo.
//!
//! ⭐⭐⭐ **A ordem não foi escolhida: ela é DERIVADA.** O catálogo dos componentes
//! ([`ph2d_component_desc::ComponentCategory::ALL`]) já declara **16 famílias, por ordem**, e é
//! por elas que a paleta do *Add Component* agrupa. *Uma tabela com dois consumidores e só um a
//! lê-la é a forma que esta casa já pagou meia dúzia de vezes* — o artista acrescentava um
//! `Timer` da prateleira **Logic** e ia procurá-lo entre as Âncoras e o Áudio.
//!
//! ⚠️⚠️ **A `ids::LIVE_SECTIONS` NÃO é a fonte desta ordem, e não pode ser:** ela é um ÍNDICE DE
//! ARMAZENAMENTO — as notas por secção indexam-na por POSIÇÃO, e o cabeçalho dela di-lo por
//! escrito (*«acrescentar a meio renumeraria a lista posicional das notas»*). ⇒ *a ordem de
//! ARMAZENAMENTO e a ordem de LEITURA são duas coisas, e confundi-las é o que pôs as Tags no fim.*
//!
//! ⚠️ **Dentro de uma família a ordem RELATIVA não mudou** (ordenação estável): a família é
//! derivada, o que vem dentro dela não é uma decisão que esta wave tenha de tomar.
//!
//! ⚠️ **Os cortes entre `…_logica` e `…_logica_cont` são do TECTO DE FUNÇÃO e não significam
//! nada** — a família LÓGICA tem doze secções e doze chamadas não cabem em `200` linhas.

use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

use crate::paint_optional_top20::Top20;

/// ⭐ **FÍSICA** (família `9` de 16) — o mover de vista de cima, o projéctil e o raio.
///
/// ⚠️ **O seguidor de caminho SAIU daqui** e foi para a LÓGICA, que é a família que o catálogo lhe
/// dá (`ph2d::ecs::PathFollow`): ele vivia neste grupo por ter chegado na mesma wave que os dois
/// movers, que é exactamente a ordem que esta wave existe para desfazer.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_familia_fisica(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    section_tops_y: &mut Vec<f32>,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    mut y: f32,
    header_h: f32,
    snaps: &crate::paint_frame::LiveSnapshots,
) -> f32 {
    y = crate::paint_optional_movers::paint_topdown_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        snaps.topdown_info.as_ref(),
    );
    y = crate::paint_optional_movers::paint_projectile_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        snaps.projectile_info.as_ref(),
    );
    y = crate::paint_optional_suplentes::paint_ray_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        snaps.ray_info.as_ref(),
    );
    y
}

/// ⭐ **LÓGICA** (família `11` de 16), primeira metade — *o que faz um jogo acontecer sem uma
/// linha de script*.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_familia_logica(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    section_tops_y: &mut Vec<f32>,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    mut y: f32,
    header_h: f32,
    snaps: &crate::paint_frame::LiveSnapshots,
    timer_selected: &mut usize,
    action_selected: &mut usize,
) -> f32 {
    y = crate::paint_optional::paint_timer_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        snaps.timer_info.as_ref(),
        timer_selected,
    );
    y = crate::paint_optional::paint_action_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        snaps.action_info.as_ref(),
        action_selected,
    );
    y = crate::paint_optional_factory::paint_factory_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        snaps.factory_info.as_ref(),
    );
    y = crate::paint_optional_factory::paint_lifecycle_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        snaps.factory_info.as_ref(),
    );
    y = crate::paint_optional_movers::paint_path_follow_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        snaps.path_follow_info.as_ref(),
    );
    y = crate::paint_optional_suplentes::paint_weapon_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        snaps.weapon_info.as_ref(),
    );
    y
}

/// ⭐ **LÓGICA**, segunda metade. ⚠️ A fronteira é o TECTO DE FUNÇÃO e não significa nada.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_familia_logica_cont(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    section_tops_y: &mut Vec<f32>,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    mut y: f32,
    header_h: f32,
    snaps: &crate::paint_frame::LiveSnapshots,
    infos: &Top20,
) -> f32 {
    y = crate::paint_optional_factory::paint_statemachine_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        infos.statemachine,
        infos.sm_state_selected,
        infos.sm_trans_selected,
    );
    y = crate::paint_optional_top20::paint_hud_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        infos.hud,
    );
    y = crate::paint_optional_top20::paint_sequence_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        infos.sequence,
    );
    y = crate::paint_optional_top20::paint_counter_watch_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        infos.watch,
        infos.watch_selected,
    );
    y = crate::paint_optional_top20::paint_action_trigger_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        infos.trigger,
        infos.trigger_selected,
    );
    y = crate::paint_optional_top20_tail::paint_tween_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        infos.tween,
        infos.tween_selected,
    );
    // ⭐⭐⭐ A VIDA e o DANO (plano 28, W3) — família LÓGICA pelo catálogo
    // (`ph2d::physics::Health` · `ph2d::physics::Damage`), as últimas a chegar a ela; entraram
    // depois da ordem por família (integração de 2026-09-25).
    y = crate::paint_optional_vida::paint_vida_sections(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        snaps.vida_info.as_ref(),
    );
    y
}

/// ⭐ **ÁUDIO** (`12`) · **CÂMERA** (`13`) · **SCRIPT** (`14`) — as três últimas famílias que o
/// Inspector pinta, pela ordem do catálogo.
///
/// ⚠️ O abanão e o emissor de abanão são da família CÂMERA pelo catálogo
/// (`ph2d::ecs::CameraShake` · `ph2d::ecs::ShakeEmitter`), e é por isso que vêm com ela e não no
/// fim, onde chegaram.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_familia_saida(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: ph2d_tokens::Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    section_tops_y: &mut Vec<f32>,
    inner_x: f32,
    inner_w: f32,
    body_top_y: f32,
    mut y: f32,
    header_h: f32,
    snaps: &crate::paint_frame::LiveSnapshots,
    infos: &Top20,
) -> f32 {
    y = crate::paint_optional_factory::paint_audio_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        snaps.audio_info.as_ref(),
    );
    y = crate::paint_optional_factory::paint_camera_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        snaps.camera_info.as_ref(),
    );
    y = crate::paint_optional_shake::paint_shake_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        infos.shake,
    );
    y = crate::paint_optional_shake::paint_shake_emitter_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        infos.emitter,
        infos.emitter_selected,
    );
    // ⭐⭐⭐ A PARALAXE (plano 24) — família CÂMERA pelo catálogo (`ph2d::ecs::ScrollFactor`…), logo
    // a seguir às irmãs dela; chegou depois da ordem por família (integração de 2026-09-25).
    y = crate::paint_optional_suplentes::paint_parallax_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        snaps.parallax_info.as_ref(),
    );
    y = crate::paint_optional_factory::paint_script_section(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        section_tops_y,
        inner_x,
        inner_w,
        body_top_y,
        y,
        header_h,
        infos.script,
    );
    y
}
