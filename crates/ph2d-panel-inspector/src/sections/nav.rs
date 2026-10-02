//! ⭐⭐⭐ **O que o Inspector mostra da NAVEGAÇÃO** (plano 30, W4) — a REGIÃO onde se anda e o
//! AGENTE que acha o caminho sozinho.
//!
//! # ⭐⭐ A QUEIXA vem antes dos números
//!
//! O agente **PEDE** e o mover **ANDA**: a ponte salta quem não tem um mover que a ouça, e *«pus o
//! agente e ele não anda»* tem oito causas com oito curas. A ordem delas NÃO vive aqui — é a porta
//! [`InspectorNavAgent::queixa`], cujo gate corre sem um device; este ficheiro só lhe dá a língua
//! ([`chave_do_agente`]) e a cor.
//!
//! # ⭐⭐ E a leitura VIVA
//!
//! *«Moving · 3,20 m to go»* e o raio DERIVADO do colisor (o caso comum, que o `0` do campo não
//! diz) saem do `NavNow` que a ponte publica — não de campo nenhum.

use super::*;
use ph2d_editor_core::nav_edits::{
    AgentQueixa, InspectorNavAgent, InspectorNavRegion, NavAlvoModo, NavEstado, RegionQueixa,
};
use ph2d_editor_core::property_row::{Seccao, paint_check_row, paint_choice_row};
use ph2d_editor_core::widget::{SectionFold, Unit};
use ph2d_i18n::{tr, tr_with};

/// **A CHAVE e a COR de cada queixa do agente** — a porta entre o enum da lei e a língua.
///
/// ⚠️ **As três primeiras são `Danger`**: com elas a ponte NÃO conduz o agente, e nenhum outro
/// número da secção faz diferença enquanto não forem curadas.
#[must_use]
const fn chave_do_agente(q: AgentQueixa) -> (&'static str, ColorToken) {
    match q {
        AgentQueixa::SemCorpo => ("panel.inspector.nav.no_body", ColorToken::Danger),
        AgentQueixa::SemMover => ("panel.inspector.nav.no_mover", ColorToken::Danger),
        AgentQueixa::MoverLeTeclado => ("panel.inspector.nav.mover_reads_keys", ColorToken::Danger),
        AgentQueixa::ComPlataforma => ("panel.inspector.nav.platformer_wins", ColorToken::Warn),
        AgentQueixa::Desligado => ("panel.inspector.nav.switched_off", ColorToken::Text3),
        AgentQueixa::SemAlvo => ("panel.inspector.nav.no_target", ColorToken::Text3),
        AgentQueixa::AlvoPerdido => ("panel.inspector.nav.target_lost", ColorToken::Warn),
        AgentQueixa::ForaDaRegiao => ("panel.inspector.nav.outside_regions", ColorToken::Warn),
    }
}

#[must_use]
const fn chave_da_regiao(q: RegionQueixa) -> &'static str {
    match q {
        RegionQueixa::SemTamanho => "panel.inspector.nav.region_has_no_size",
        RegionQueixa::SemParedes => "panel.inspector.nav.no_layer_blocks",
    }
}

/// O cabeçalho dobrável comum às duas secções. `None` = dobrada (devolve o `y` por baixo dele).
#[allow(clippy::too_many_arguments)]
fn cabecalho(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    id: NodeId,
    titulo: &str,
) -> Result<(SectionFold, f32), f32> {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(store, id, titulo);
    paint_section_header(
        &header,
        Rect::new(x, y, w, header_h),
        scene,
        text_system,
        theme,
    );
    match SectionFold::begin(store, id, x, w, y + header_h, scene, hit_index) {
        Some(fold) => Ok((fold, y + header_h)),
        None => Err(y + header_h),
    }
}

/// Pinta a secção NAV REGION. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_nav_region_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    r: &InspectorNavRegion,
) -> f32 {
    let (fold, mut cur_y) = match cabecalho(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ph2d_editor_core::ids::INSP_LIVE_NAV_REGION_SECTION,
        tr("panel.inspector.nav.nav_region"),
    ) {
        Ok(v) => v,
        Err(y) => return y,
    };
    if let Some(q) = r.queixa() {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr(chave_da_regiao(q)),
            ColorToken::Warn,
        );
    }
    let seccao = Seccao::medida(
        text_system,
        2,
        &[
            tr("panel.inspector.nav.half_size"),
            tr("panel.inspector.nav.blocking_layers"),
        ],
    );
    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.nav.half_size"),
        &[crate::ids::INSP_NAV_HALF_W, crate::ids::INSP_NAV_HALF_H],
        0.1, // LITERAL-PX-OK: passo de arrasto em METROS
        Some(Unit::Meters),
        seccao,
    );
    // ⚠️ **Escolha MÚLTIPLA** — cada camada acesa é um bit da máscara; os rótulos são os números
    // nus da secção de física (a camada não tem nome próprio: o nome é a linha dela na matriz).
    let segmentos: Vec<(&str, bool, NodeId)> = crate::ids::INSP_NAV_LAYERS
        .iter()
        .enumerate()
        .map(|(i, &id)| {
            (
                super::physics_rows::LAYER_LABELS[i],
                r.obstacle_layers & (1 << i) != 0,
                id,
            )
        })
        .collect();
    cur_y = paint_choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.nav.blocking_layers"),
        &segmentos,
        seccao,
    );
    fold.finish(store, scene, hit_index, cur_y)
}

/// **O que o agente está a fazer AGORA** — a leitura viva, e o raio derivado.
#[allow(clippy::too_many_arguments)]
fn leitura(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    a: &InspectorNavAgent,
) -> f32 {
    let Some(agora) = a.agora else {
        return y;
    };
    let dist = format!("{:.2}", agora.restante);
    let texto = match agora.estado {
        NavEstado::Parado => tr("panel.inspector.nav.idle").to_string(),
        NavEstado::AAndar => tr_with("panel.inspector.nav.moving_x_m_to_go", &[("dist", &dist)]),
        NavEstado::Parcial => tr_with("panel.inspector.nav.cant_reach_x_m", &[("dist", &dist)]),
        NavEstado::Chegou => tr("panel.inspector.nav.arrived").to_string(),
        NavEstado::SemCaminho => tr("panel.inspector.nav.no_way_there").to_string(),
    };
    let mut cur_y = super::rows::aviso(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        &texto,
        ColorToken::Text1,
    );
    if a.radius <= 0.0 {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &tr_with(
                "panel.inspector.nav.radius_x_m_from_the_collider",
                &[("r", &format!("{:.2}", agora.raio))],
            ),
            ColorToken::Text3,
        );
    }
    cur_y
}

/// Os avisos do agente: a queixa, a leitura viva e o relógio.
#[allow(clippy::too_many_arguments)]
fn avisos(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    a: &InspectorNavAgent,
    clock_playing: bool,
) -> f32 {
    let mut cur_y = y;
    if let Some(q) = a.queixa() {
        let (chave, cor) = chave_do_agente(q);
        cur_y = super::rows::aviso(scene, text_system, theme, x, w, cur_y, tr(chave), cor);
    }
    cur_y = leitura(scene, text_system, theme, x, w, cur_y, a);
    if !clock_playing {
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.nav.the_clock_is_stopped"),
            ColorToken::Text3,
        );
    }
    cur_y
}

/// Pinta a secção NAV AGENT. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(crate) fn paint_nav_agent_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    a: &InspectorNavAgent,
    clock_playing: bool,
) -> f32 {
    let (fold, mut cur_y) = match cabecalho(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        ph2d_editor_core::ids::INSP_LIVE_NAV_AGENT_SECTION,
        tr("panel.inspector.nav.nav_agent"),
    ) {
        Ok(v) => v,
        Err(y) => return y,
    };
    cur_y = avisos(scene, text_system, theme, x, w, cur_y, a, clock_playing);
    // ⭐⭐ **A coluna do nome é da SECÇÃO** — os nomes da secção INTEIRA, senão ela salta quando o
    //    modo do alvo troca a linha de baixo.
    let seccao = Seccao::medida(
        text_system,
        2,
        &[
            tr("panel.inspector.nav.target"),
            tr("panel.inspector.nav.radius"),
            tr("panel.inspector.nav.arrive_at"),
            tr("panel.inspector.nav.repath_after"),
            tr("panel.inspector.nav.stuck_after"),
            tr("panel.inspector.nav.on_arrive"),
            tr("panel.inspector.nav.on_no_path"),
            tr("panel.inspector.nav.on_stuck"),
        ],
    );
    let modos = [
        tr("panel.inspector.nav.target_none"),
        tr("panel.inspector.nav.target_object"),
        tr("panel.inspector.nav.target_point"),
    ];
    let segmentos: Vec<(&str, bool, NodeId)> = NavAlvoModo::ALL
        .iter()
        .enumerate()
        .map(|(i, &m)| {
            (
                modos[i],
                m == a.alvo_modo,
                crate::ids::INSP_NAV_TARGET_MODE[i],
            )
        })
        .collect();
    cur_y = paint_choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.nav.target"),
        &segmentos,
        seccao,
    );
    // ⚠️ **A linha de baixo depende do MODO** — a lei do `SignalVerb::uses_arg`: mostrar as duas
    // entregava um controlo morto em cada modo.
    match a.alvo_modo {
        NavAlvoModo::Nenhum => {}
        NavAlvoModo::Objecto => {
            cur_y = super::anim_rows::text_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                cur_y,
                tr("panel.inspector.nav.target_object"),
                crate::ids::INSP_NAV_TARGET_NAME,
                TextInput::new(crate::ids::INSP_NAV_TARGET_NAME, "")
                    .placeholder(tr("panel.inspector.nav.object_name_u")),
                seccao,
            );
        }
        NavAlvoModo::Ponto => {
            cur_y = super::rows::fields_row(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                x,
                w,
                cur_y,
                tr("panel.inspector.nav.target_point"),
                &[crate::ids::INSP_NAV_TARGET_X, crate::ids::INSP_NAV_TARGET_Y],
                0.1, // LITERAL-PX-OK: passo de arrasto em METROS, mundo
                Some(Unit::Meters),
                seccao,
            );
        }
    }
    for (label, id, unidade) in [
        (
            tr("panel.inspector.nav.radius"),
            crate::ids::INSP_NAV_RADIUS,
            Unit::Meters,
        ),
        (
            tr("panel.inspector.nav.arrive_at"),
            crate::ids::INSP_NAV_ARRIVE,
            Unit::Meters,
        ),
        (
            tr("panel.inspector.nav.repath_after"),
            crate::ids::INSP_NAV_REPATH,
            Unit::Meters,
        ),
        (
            tr("panel.inspector.nav.stuck_after"),
            crate::ids::INSP_NAV_STUCK,
            Unit::Seconds,
        ),
    ] {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            label,
            &[id],
            0.05, // LITERAL-PX-OK: passo de arrasto
            Some(unidade),
            seccao,
        );
    }
    cur_y = paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        (
            crate::ids::INSP_NAV_ACTIVE,
            tr("panel.inspector.nav.active"),
            a.active,
        ),
        seccao,
    );
    cur_y = paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        (
            crate::ids::INSP_NAV_AVOIDANCE,
            tr("panel.inspector.nav.avoidance"),
            a.avoidance,
        ),
        seccao,
    );
    cur_y = sinais(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        seccao,
    );
    fold.finish(store, scene, hit_index, cur_y)
}

/// Os três NOMES que o agente publica — vazio = calado, a regra do `SignalOnHit`.
///
/// ⛔ **Pela porta que já existe** (`anim_rows::text_row`), e cada `.placeholder("…")` escrito
/// por extenso: o gate do HR-15 conta-os no código de widget, e uma string passada por argumento
/// num laço sai da vista dele (a isenção silenciosa que o #15 pagou).
#[allow(clippy::too_many_arguments)]
fn sinais(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    seccao: Seccao,
) -> f32 {
    let mut cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        tr("panel.inspector.nav.on_arrive"),
        crate::ids::INSP_NAV_ON_ARRIVED,
        TextInput::new(crate::ids::INSP_NAV_ON_ARRIVED, "")
            .placeholder(tr("panel.inspector.nav.signal_when_it_arrives_u")),
        seccao,
    );
    cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.nav.on_no_path"),
        crate::ids::INSP_NAV_ON_NO_PATH,
        TextInput::new(crate::ids::INSP_NAV_ON_NO_PATH, "")
            .placeholder(tr("panel.inspector.nav.signal_when_there_is_no_way_u")),
        seccao,
    );
    super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.nav.on_stuck"),
        crate::ids::INSP_NAV_ON_STUCK,
        TextInput::new(crate::ids::INSP_NAV_ON_STUCK, "")
            .placeholder(tr("panel.inspector.nav.signal_when_it_gets_stuck_u")),
        seccao,
    )
}
