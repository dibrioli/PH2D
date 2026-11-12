//! ⭐⭐⭐ **O que o Inspector mostra do MOVER DE VISTA DE CIMA** — a versão A da ficha
//! `docs/GameEngineFuncional/componentes/05_topdown.md` («três perguntas», dono 12/11).
//!
//! À vista: os avisos, Speed, Directions, Facing e Turn (só quando ele gira). O resto mora na dobra
//! «More», fechada, cujo cabeçalho resume a rampa e o Viewpoint (`No easing · Top-Down view`).
//!
//! # ⭐⭐ Os avisos dizem o que os campos sozinhos não diriam
//!
//! | aviso | o que se passa |
//! |---|---|
//! | `No body` | sem `RigidBody` não há o que mover |
//! | `The body must be Kinematic` | um corpo dinâmico é do **solver** |
//! | `A Platform Player on this object wins` | dois movers, um `Transform` |
//! | `The clock is stopped` | a ponte corre no passo fixo |
//! | `Default Controls is off` | nada o move até um script ou um nó; o botão «Turn on» liga-o |
//!
//! # ⭐ Duas linhas SOMEM conforme o modo
//!
//! O *Board Angle* só existe em `Custom` e o *Turn* só quando ele roda — a lei do
//! `SignalVerb::uses_arg`: mostrar sempre entrega um controlo morto em cada modo.

use super::grupo_dobravel::{Pincel, botao, dobra};
use super::*;
use ph2d_editor_core::property_row::Seccao;
use ph2d_editor_core::topdown_edits::{
    InspectorFacing, InspectorMoveDirections, InspectorTopDownInfo, InspectorViewpoint,
};
use ph2d_editor_core::widget::SectionFold;
use ph2d_editor_core::widget::Unit;
use ph2d_i18n::{TextKey, tr};

/// Os rótulos da secção INTEIRA (dobrada ou não): a coluna do nome é uma só.
const ROTULOS: [&str; 12] = [
    "component.field.top_down_player.0",
    "component.field.top_down_player.3",
    "component.field.top_down_player.6",
    "component.field.top_down_player.7",
    "component.field.top_down_player.1",
    "component.field.top_down_player.2",
    "component.field.top_down_player.4",
    "component.field.top_down_player.5",
    "component.field.top_down_player.11",
    "component.field.top_down_player.8",
    "component.field.top_down_player.9",
    "component.field.top_down_player.10",
];

/// O pincel da secção.
struct Td<'a> {
    scene: &'a mut VectorScene,
    text: &'a mut TextSystem,
    theme: Theme,
    hit: &'a mut HitIndex,
    store: &'a WidgetStore,
    x: f32,
    w: f32,
    info: &'a InspectorTopDownInfo,
    seccao: Seccao,
}

impl Pincel for Td<'_> {
    #[allow(clippy::type_complexity)]
    fn partes(
        &mut self,
    ) -> (
        &mut VectorScene,
        &mut TextSystem,
        Theme,
        &mut HitIndex,
        &WidgetStore,
        f32,
        f32,
    ) {
        (
            self.scene, self.text, self.theme, self.hit, self.store, self.x, self.w,
        )
    }
}

impl Td<'_> {
    fn aviso(&mut self, y: f32, chave: &str, token: ColorToken) -> f32 {
        super::rows::aviso(
            self.scene,
            self.text,
            self.theme,
            self.x,
            self.w,
            y,
            tr(chave),
            token,
        )
    }

    /// Um número pela porta das unidades do mundo (`rows::fields_row` → px/m).
    fn numero(
        &mut self,
        y: f32,
        campo: &'static str,
        id: NodeId,
        passo: f64,
        u: Option<Unit>,
    ) -> f32 {
        super::rows::fields_row(
            self.scene,
            self.text,
            self.theme,
            self.hit,
            self.store,
            self.x,
            self.w,
            y,
            TextKey::new(campo),
            &[id],
            passo,
            u,
            self.seccao,
        )
    }

    /// Um segmentado. ⚠️ **A selecção vem do SNAPSHOT, nunca do store** — ler o store faria o
    /// primeiro clique depois de trocar de objecto mandar o valor do objecto anterior.
    fn escolha(
        &mut self,
        y: f32,
        campo: &'static str,
        ids: &[NodeId],
        rotulos: &[&str],
        sel: u8,
    ) -> f32 {
        let segmentos: Vec<(&str, bool, NodeId)> = ids
            .iter()
            .zip(rotulos)
            .enumerate()
            .map(|(i, (&id, &r))| (r, i == usize::from(sel), id))
            .collect();
        ph2d_editor_core::property_row::paint_choice_row(
            self.scene,
            self.text,
            self.theme,
            self.hit,
            self.store,
            self.x,
            self.w,
            y,
            TextKey::new(campo),
            &segmentos,
            self.seccao,
        )
    }
}

/// **Os AVISOS** — vêm antes dos números: quem não vê nada mexer não quer afinar uma rampa.
fn avisos(p: &mut Td<'_>, y: f32) -> f32 {
    let i = p.info;
    let mut y = y;
    if !i.has_body {
        y = p.aviso(
            y,
            "panel.inspector.topdown.no_body_u_add_a_rigid_body_for_this_to_move_anything",
            ColorToken::Danger,
        );
    } else if !i.body_is_kinematic {
        y = p.aviso(
            y,
            "panel.inspector.topdown.the_body_must_be_kinematic_u_a_dynamic_body_belongs_to_the_solver",
            ColorToken::Danger,
        );
    }
    if i.conflicts_with_platformer {
        y = p.aviso(
            y,
            "panel.inspector.topdown.a_platform_player_on_this_object_wins_u_remove_one_of_the_two",
            ColorToken::Warn,
        );
    } else if !i.clock_playing {
        y = p.aviso(
            y,
            "panel.inspector.topdown.the_clock_is_stopped_u_it_moves_while_the_clock_plays",
            ColorToken::Text3,
        );
    }
    // ⭐ A caixa mora no More (fechado): o aviso e a saída dele ficam à vista.
    if !i.default_controls {
        y = p.aviso(
            y,
            "panel.inspector.topdown.default_controls_is_off",
            ColorToken::Text3,
        );
        y = botao(
            p,
            y,
            ids::INSP_TD_TURN_ON_CONTROLS,
            tr("panel.inspector.topdown.turn_on"),
        );
    }
    y
}

/// À vista: Speed · Directions · Facing · Turn (só se gira).
fn a_vista(p: &mut Td<'_>, y: f32) -> f32 {
    let i = p.info;
    let y = p.numero(
        y,
        "component.field.top_down_player.0",
        ids::INSP_TD_SPEED,
        0.1, // LITERAL-PX-OK: passo de scrub em m/s
        Some(Unit::MetersPerSecond),
    );
    let rotulos = InspectorMoveDirections::ALL.map(InspectorMoveDirections::label);
    let y = p.escolha(
        y,
        "component.field.top_down_player.3",
        &ids::INSP_TD_DIRECTIONS,
        &rotulos,
        i.directions.tag(),
    );
    let rotulos = InspectorFacing::ALL.map(InspectorFacing::label);
    let y = p.escolha(
        y,
        "component.field.top_down_player.6",
        &ids::INSP_TD_FACING,
        &rotulos,
        i.facing.tag(),
    );
    if !i.facing.uses_turn_speed() {
        return y;
    }
    p.numero(
        y,
        "component.field.top_down_player.7",
        ids::INSP_TD_TURN_SPEED,
        10.0, // LITERAL-PX-OK: graus/s
        Some(Unit::DegreesPerSecond),
    )
}

/// ⭐ **O resumo do cabeçalho do More**: a rampa (`No easing` só com as duas a zero) · a vista.
pub(crate) fn resumo_do_mais(i: &InspectorTopDownInfo) -> String {
    let rampa = if i.acceleration == 0.0 && i.deceleration == 0.0 {
        tr("panel.inspector.topdown.sum_no_easing")
    } else {
        tr("panel.inspector.topdown.sum_eased")
    };
    let vista = ph2d_i18n::tr_with(
        "panel.inspector.topdown.sum_view",
        &[("name", &i.viewpoint.label())],
    );
    format!("{rampa} · {vista}")
}

/// O corpo do More, pela ordem da ficha.
fn mais(p: &mut Td<'_>, y: f32) -> f32 {
    let i = p.info;
    let ramp = Some(Unit::MetersPerSecondSquared);
    let y = p.numero(
        y,
        "component.field.top_down_player.1",
        ids::INSP_TD_ACCEL,
        0.5,
        ramp,
    );
    let y = p.numero(
        y,
        "component.field.top_down_player.2",
        ids::INSP_TD_DECEL,
        0.5,
        ramp,
    );
    let rotulos = InspectorViewpoint::ALL.map(InspectorViewpoint::label);
    let mut y = p.escolha(
        y,
        "component.field.top_down_player.4",
        &ids::INSP_TD_VIEWPOINT,
        &rotulos,
        i.viewpoint.tag(),
    );
    if i.viewpoint.uses_angle() {
        y = p.numero(
            y,
            "component.field.top_down_player.5",
            ids::INSP_TD_VIEW_ANGLE,
            0.5, // LITERAL-PX-OK: graus
            Some(Unit::Degrees),
        );
    }
    let campos = [
        (
            "component.field.top_down_player.11",
            ids::INSP_TD_KNOCKBACK_RECOVERY,
            0.5,
            ramp,
        ),
        (
            "component.field.top_down_player.8",
            ids::INSP_TD_MIN_SLIDE,
            1.0,
            Some(Unit::Degrees),
        ),
        (
            "component.field.top_down_player.9",
            ids::INSP_TD_MAX_SLIDES,
            1.0,
            None,
        ),
    ];
    for (campo, id, passo, u) in campos {
        y = p.numero(y, campo, id, passo, u);
    }
    ph2d_editor_core::property_row::paint_check_row(
        p.scene,
        p.text,
        p.theme,
        p.hit,
        p.store,
        p.x,
        p.w,
        y,
        (
            ids::INSP_TD_DEFAULT_CONTROLS,
            TextKey::new("component.field.top_down_player.10"),
            i.default_controls,
        ),
        p.seccao,
    )
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_topdown_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorTopDownInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        core_ids::INSP_LIVE_TOPDOWN_SECTION,
        tr("component.top_down_player.name"),
    );
    paint_section_header(
        &header,
        Rect::new(x, y, w, header_h),
        scene,
        text_system,
        theme,
    );
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_TOPDOWN_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let nomes = ROTULOS.map(tr);
    let seccao = Seccao::medida(text_system, 1, &nomes);
    let mut p = Td {
        scene,
        text: text_system,
        theme,
        hit: hit_index,
        store,
        x,
        w,
        info,
        seccao,
    };
    let mut cur_y = avisos(&mut p, y + header_h);
    cur_y = a_vista(&mut p, cur_y);
    let resumo = resumo_do_mais(info);
    cur_y = dobra(
        &mut p,
        cur_y,
        ids::INSP_TD_FOLD_MORE,
        tr("panel.inspector.topdown.more"),
        &resumo,
        mais,
    );
    fold.finish(store, p.scene, p.hit, cur_y)
}
