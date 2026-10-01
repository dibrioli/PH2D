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

use ph2d_editor_core::interaction::WidgetStore;

use crate::paint_optional_top20::Top20;

/// ⭐ **FÍSICA** (família `9` de 16) — o mover de vista de cima, o projéctil e o raio.
///
/// ⚠️ **O seguidor de caminho SAIU daqui** e foi para a LÓGICA, que é a família que o catálogo lhe
/// dá (`ph2d::ecs::PathFollow`): ele vivia neste grupo por ter chegado na mesma wave que os dois
/// movers, que é exactamente a ordem que esta wave existe para desfazer.
#[allow(clippy::too_many_arguments)]
pub(crate) fn push_familia_fisica<'a>(
    plano: &mut crate::plano::Plano<'a>,
    store: &'a WidgetStore,
    inner_x: f32,
    inner_w: f32,
    header_h: f32,
    snaps: &'a crate::paint_frame::LiveSnapshots,
) {
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_TOPDOWN_SECTION,
        move |c, t, y| {
            crate::paint_optional_movers::paint_topdown_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                snaps.topdown_info.as_ref(),
            )
        },
    );
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_PROJECTILE_SECTION,
        move |c, t, y| {
            crate::paint_optional_movers::paint_projectile_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                snaps.projectile_info.as_ref(),
            )
        },
    );
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_RAY_SECTION,
        move |c, t, y| {
            crate::paint_optional_suplentes::paint_ray_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                snaps.ray_info.as_ref(),
            )
        },
    );
    // ⭐⭐⭐ A NAVEGAÇÃO (plano 30, W4) — família FÍSICA pelo catálogo, a última a chegar a ela.
    crate::paint_optional_nav::push_nav_sections(
        plano,
        store,
        inner_x,
        inner_w,
        header_h,
        snaps.nav_info.as_ref(),
    );
}

/// ⭐ **LÓGICA** (família `11` de 16), primeira metade — *o que faz um jogo acontecer sem uma
/// linha de script*.
#[allow(clippy::too_many_arguments)]
pub(crate) fn push_familia_logica<'a>(
    plano: &mut crate::plano::Plano<'a>,
    store: &'a WidgetStore,
    inner_x: f32,
    inner_w: f32,
    header_h: f32,
    snaps: &'a crate::paint_frame::LiveSnapshots,
    timer_selected: &'a mut usize,
    action_selected: &'a mut usize,
) {
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_TIMER_SECTION,
        move |c, t, y| {
            crate::paint_optional::paint_timer_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                snaps.timer_info.as_ref(),
                timer_selected,
            )
        },
    );
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_ACTION_SECTION,
        move |c, t, y| {
            crate::paint_optional::paint_action_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                snaps.action_info.as_ref(),
                action_selected,
            )
        },
    );
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_FACTORY_SECTION,
        move |c, t, y| {
            crate::paint_optional_factory::paint_factory_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                snaps.factory_info.as_ref(),
            )
        },
    );
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_LIFECYCLE_SECTION,
        move |c, t, y| {
            crate::paint_optional_factory::paint_lifecycle_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                snaps.factory_info.as_ref(),
            )
        },
    );
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_PATHFOLLOW_SECTION,
        move |c, t, y| {
            crate::paint_optional_movers::paint_path_follow_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                snaps.path_follow_info.as_ref(),
            )
        },
    );
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_WEAPON_SECTION,
        move |c, t, y| {
            crate::paint_optional_suplentes::paint_weapon_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                snaps.weapon_info.as_ref(),
            )
        },
    );
}

/// ⭐ **LÓGICA**, segunda metade. ⚠️ A fronteira é o TECTO DE FUNÇÃO e não significa nada.
#[allow(clippy::too_many_arguments)]
pub(crate) fn push_familia_logica_cont<'a>(
    plano: &mut crate::plano::Plano<'a>,
    store: &'a WidgetStore,
    inner_x: f32,
    inner_w: f32,
    header_h: f32,
    snaps: &'a crate::paint_frame::LiveSnapshots,
    infos: Top20<'a>,
) {
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_SM_SECTION,
        move |c, t, y| {
            crate::paint_optional_factory::paint_statemachine_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                infos.statemachine,
                infos.sm_state_selected,
                infos.sm_trans_selected,
            )
        },
    );
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_HUD_SECTION,
        move |c, t, y| {
            crate::paint_optional_top20::paint_hud_section(
                c.scene, c.text, t, c.hit, store, inner_x, inner_w, y, header_h, infos.hud,
            )
        },
    );
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_SEQ_SECTION,
        move |c, t, y| {
            crate::paint_optional_top20::paint_sequence_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                infos.sequence,
            )
        },
    );
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_WATCH_SECTION,
        move |c, t, y| {
            crate::paint_optional_top20::paint_counter_watch_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                infos.watch,
                infos.watch_selected,
            )
        },
    );
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_TRIGGER_SECTION,
        move |c, t, y| {
            crate::paint_optional_top20::paint_action_trigger_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                infos.trigger,
                infos.trigger_selected,
            )
        },
    );
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_TWEEN_SECTION,
        move |c, t, y| {
            crate::paint_optional_top20_tail::paint_tween_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                infos.tween,
                infos.tween_selected,
            )
        },
    );
    // ⭐⭐⭐ A VIDA e o DANO (plano 28, W3) — família LÓGICA pelo catálogo
    // (`ph2d::physics::Health` · `ph2d::physics::Damage`), as últimas a chegar a ela; entraram
    // depois da ordem por família (integração de 2026-09-25).
    crate::paint_optional_vida::push_vida_sections(
        plano,
        store,
        inner_x,
        inner_w,
        header_h,
        snaps.vida_info.as_ref(),
        infos.resist_selected,
    );
}

/// ⭐ **ÁUDIO** (`12`) · **CÂMERA** (`13`) · **SCRIPT** (`14`) — as três últimas famílias que o
/// Inspector pinta, pela ordem do catálogo.
///
/// ⚠️ O abanão e o emissor de abanão são da família CÂMERA pelo catálogo
/// (`ph2d::ecs::CameraShake` · `ph2d::ecs::ShakeEmitter`), e é por isso que vêm com ela e não no
/// fim, onde chegaram.
#[allow(clippy::too_many_arguments)]
pub(crate) fn push_familia_saida<'a>(
    plano: &mut crate::plano::Plano<'a>,
    store: &'a WidgetStore,
    inner_x: f32,
    inner_w: f32,
    header_h: f32,
    snaps: &'a crate::paint_frame::LiveSnapshots,
    infos: Top20<'a>,
) {
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_AUDIO_SECTION,
        move |c, t, y| {
            crate::paint_optional_factory::paint_audio_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                snaps.audio_info.as_ref(),
            )
        },
    );
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_CAMERA_SECTION,
        move |c, t, y| {
            crate::paint_optional_factory::paint_camera_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                snaps.camera_info.as_ref(),
            )
        },
    );
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_SHAKE_SECTION,
        move |c, t, y| {
            crate::paint_optional_shake::paint_shake_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                infos.shake,
            )
        },
    );
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_EMITTER_SECTION,
        move |c, t, y| {
            crate::paint_optional_shake::paint_shake_emitter_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                infos.emitter,
                infos.emitter_selected,
            )
        },
    );
    // ⭐⭐⭐ A PARALAXE (plano 24) — família CÂMERA pelo catálogo (`ph2d::ecs::ScrollFactor`…), logo
    // a seguir às irmãs dela; chegou depois da ordem por família (integração de 2026-09-25).
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_PARALLAX_SECTION,
        move |c, t, y| {
            crate::paint_optional_suplentes::paint_parallax_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                snaps.parallax_info.as_ref(),
            )
        },
    );
    plano.push(
        ph2d_editor_core::ids::INSP_LIVE_SCRIPT_SECTION,
        move |c, t, y| {
            crate::paint_optional_factory::paint_script_section(
                c.scene,
                c.text,
                t,
                c.hit,
                store,
                inner_x,
                inner_w,
                y,
                header_h,
                infos.script,
            )
        },
    );
}
