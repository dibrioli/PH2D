//! ⭐⭐⭐ **O que o Inspector mostra do PROJÉCTIL** — a versão B da ficha
//! `docs/GameEngineFuncional/componentes/06_projectile.md` («voo, ricochete, perseguição», dono 13/11).
//!
//! À vista: os avisos, Speed, Gravity, Range e Faces path; as caixas **Bounce** (abre Max Bounces e Bounciness) e
//! **Homing** (abre Target, com conta-gotas, e Pull); a dobra «Speed change» (Acceleration, Max Speed), fechada,
//! com o resumo no cabeçalho.
//!
//! # ⭐⭐ Os avisos dizem o que os campos sozinhos não diriam
//!
//! | aviso | o que se passa |
//! |---|---|
//! | `No body` | sem `RigidBody` não há o que mover nem em que bater |
//! | `The body must be Kinematic` | um corpo dinâmico é do **solver** |
//! | `The flight is over` | o alcance ou os ricochetes acabaram: parado no ar com os números certos |
//! | `The clock is stopped` | a ponte corre no passo fixo |
//!
//! # ⭐ As caixas não guardam nada: leem e escrevem os números
//!
//! Bounce = `max_bounces > 0`, Homing = `homing_accel > 0` — o componente não muda (ficha GEF 06). As linhas de
//! dentro só existem com a caixa ligada: a lei do `SignalVerb::uses_arg`.

use super::grupo_dobravel::{Pincel, dobra};
use super::*;
use ph2d_editor_core::projectile_edits::InspectorProjectileInfo;
use ph2d_editor_core::property_row::Seccao;
use ph2d_editor_core::widget::SectionFold;
use ph2d_editor_core::widget::Unit;
use ph2d_i18n::{TextKey, tr};

/// Os rótulos da secção INTEIRA (caixas e dobra incluídas): a coluna do nome é uma só.
const ROTULOS: [&str; 12] = [
    "component.field.projectile_motion.0",
    "component.field.projectile_motion.3",
    "component.field.projectile_motion.6",
    "component.field.projectile_motion.7",
    "panel.inspector.projectile.bounce",
    "component.field.projectile_motion.5",
    "component.field.projectile_motion.4",
    "panel.inspector.projectile.homing",
    "component.field.projectile_motion.9",
    "component.field.projectile_motion.8",
    "component.field.projectile_motion.1",
    "component.field.projectile_motion.2",
];

/// O pincel da secção.
struct Pj<'a> {
    scene: &'a mut VectorScene,
    text: &'a mut TextSystem,
    theme: Theme,
    hit: &'a mut HitIndex,
    store: &'a WidgetStore,
    x: f32,
    w: f32,
    info: &'a InspectorProjectileInfo,
    seccao: Seccao,
}

impl Pincel for Pj<'_> {
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

impl Pj<'_> {
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

    /// Uma caixa. ⚠️ O estado vem do SNAPSHOT, nunca do store.
    fn caixa(&mut self, y: f32, rotulo: &'static str, id: NodeId, ligada: bool) -> f32 {
        ph2d_editor_core::property_row::paint_check_row(
            self.scene,
            self.text,
            self.theme,
            self.hit,
            self.store,
            self.x,
            self.w,
            y,
            (id, TextKey::new(rotulo), ligada),
            self.seccao,
        )
    }
}

/// **Os AVISOS** — vêm antes dos números: quem não vê nada mexer não quer afinar um ricochete.
fn avisos(p: &mut Pj<'_>, y: f32) -> f32 {
    let i = p.info;
    let mut y = y;
    if !i.has_body {
        y = p.aviso(
            y,
            "panel.inspector.projectile.no_body_u_add_a_rigid_body_for_this_to_move_anything",
            ColorToken::Danger,
        );
    } else if !i.body_is_kinematic {
        y = p.aviso(
            y,
            "panel.inspector.projectile.the_body_must_be_kinematic_u_a_dynamic_body_belongs_to_the_solver",
            ColorToken::Danger,
        );
    }
    if i.flight_over {
        y = p.aviso(
            y,
            "panel.inspector.projectile.the_flight_is_over_u_rewind_to_launch_it_again",
            ColorToken::Text3,
        );
    } else if !i.clock_playing {
        y = p.aviso(
            y,
            "panel.inspector.projectile.the_clock_is_stopped_u_it_flies_while_the_clock_plays",
            ColorToken::Text3,
        );
    }
    y
}

/// O voo: Speed · Gravity · Range · Faces path.
fn voo(p: &mut Pj<'_>, y: f32) -> f32 {
    let campos = [
        (
            "component.field.projectile_motion.0",
            ids::INSP_PJ_SPEED,
            0.5, // LITERAL-PX-OK: m/s
            Unit::MetersPerSecond,
        ),
        (
            "component.field.projectile_motion.3",
            ids::INSP_PJ_GRAVITY,
            0.5, // LITERAL-PX-OK: m/s²
            Unit::MetersPerSecondSquared,
        ),
        (
            "component.field.projectile_motion.6",
            ids::INSP_PJ_RANGE,
            1.0, // LITERAL-PX-OK: metros
            Unit::Meters,
        ),
    ];
    let mut y = y;
    for (campo, id, passo, u) in campos {
        y = p.numero(y, campo, id, passo, Some(u));
    }
    let ligada = p.info.face_velocity;
    p.caixa(
        y,
        "component.field.projectile_motion.7",
        ids::INSP_PJ_FACE_VELOCITY,
        ligada,
    )
}

/// A caixa Bounce e, ligada, Max Bounces e Bounciness.
fn ricochete(p: &mut Pj<'_>, y: f32) -> f32 {
    let ligada = p.info.max_bounces > 0;
    let y = p.caixa(
        y,
        "panel.inspector.projectile.bounce",
        ids::INSP_PJ_BOUNCE,
        ligada,
    );
    if !ligada {
        return y;
    }
    let y = p.numero(
        y,
        "component.field.projectile_motion.5",
        ids::INSP_PJ_MAX_BOUNCES,
        1.0, // LITERAL-PX-OK: contagem
        None,
    );
    p.numero(
        y,
        "component.field.projectile_motion.4",
        ids::INSP_PJ_BOUNCINESS,
        0.05, // LITERAL-PX-OK: fracção
        None,
    )
}

/// O campo Target com o conta-gotas DENTRO da ponta direita da caixa (o molde do Target da câmera, GEF 20 C2).
fn alvo(p: &mut Pj<'_>, y: f32) -> f32 {
    let armado = p.info.target_pick_armed;
    let fim = super::anim_rows::text_row(
        p.scene,
        p.text,
        p.theme,
        p.hit,
        p.store,
        p.x,
        p.w,
        y,
        TextKey::new("component.field.projectile_motion.9"),
        ids::INSP_PJ_HOMING_TARGET,
        TextInput::new(ids::INSP_PJ_HOMING_TARGET, "")
            .placeholder(tr("panel.inspector.projectile.target_object_name_u")),
        p.seccao,
    );
    let caixa = ph2d_editor_core::property_row::caixa_da_linha_pintada(p.x, p.w, y, fim, p.seccao);
    let lado = if ph2d_editor_core::widget::card_ink::card_body().is_some() {
        ph2d_card_stack::metrics::GRID_CELL
    } else {
        (ROW_H_PX * 0.82).min(caixa.h) // LITERAL-PX-OK: o quadrado compacto da linha (o mesmo da câmera)
    };
    let inset = (caixa.h - lado) * 0.5;
    let gotas = Rect::new(
        caixa.x + caixa.w - lado - inset,
        caixa.y + inset,
        lado,
        lado,
    );
    paint_icon_button(
        gotas,
        IconGlyph::Builtin(IconId::Eyedropper),
        IconButtonStyle::Compact,
        if armado {
            (ButtonState::Pressed, ph2d_editor_core::motion::SETTLED)
        } else {
            p.store.button_visual(ids::INSP_PJ_TARGET_PICK)
        },
        p.scene,
        p.theme,
    );
    p.hit.register(ids::INSP_PJ_TARGET_PICK, gotas);
    fim
}

/// A caixa Homing e, ligada, Target (com o aviso do nome que ninguém tem) e Pull.
fn perseguicao(p: &mut Pj<'_>, y: f32) -> f32 {
    let ligada = p.info.homing_accel > 0.0;
    let y = p.caixa(
        y,
        "panel.inspector.projectile.homing",
        ids::INSP_PJ_HOMING,
        ligada,
    );
    if !ligada {
        return y;
    }
    let mut y = alvo(p, y);
    // ⚠️ Um nome escrito que ninguém tem não é o mesmo que nenhum nome: senão um alvo apagado lê-se como uma
    // perseguição partida.
    if p.info.homing_target_missing {
        y = p.aviso(
            y,
            "panel.inspector.projectile.no_object_in_the_scene_has_that_name",
            ColorToken::Warn,
        );
    }
    p.numero(
        y,
        "component.field.projectile_motion.8",
        ids::INSP_PJ_HOMING_ACCEL,
        1.0, // LITERAL-PX-OK: m/s²
        Some(Unit::MetersPerSecondSquared),
    )
}

/// ⭐ **O resumo do cabeçalho do Speed change**: None · Speeds up · Slows down · Capped, e «· capped» com tecto.
pub(crate) fn resumo_da_rapidez(i: &InspectorProjectileInfo) -> String {
    let muda = if i.acceleration > 0.0 {
        Some("panel.inspector.projectile.sum_speeds_up")
    } else if i.acceleration < 0.0 {
        Some("panel.inspector.projectile.sum_slows_down")
    } else {
        None
    };
    match (muda, i.max_speed > 0.0) {
        (None, false) => tr("panel.inspector.projectile.sum_none").to_string(),
        (None, true) => tr("panel.inspector.projectile.sum_capped").to_string(),
        (Some(m), false) => tr(m).to_string(),
        (Some(m), true) => ph2d_i18n::tr_with(
            "panel.inspector.projectile.sum_and_capped",
            &[("change", &tr(m))],
        ),
    }
}

/// O corpo do Speed change: Acceleration · Max Speed.
fn rapidez(p: &mut Pj<'_>, y: f32) -> f32 {
    let y = p.numero(
        y,
        "component.field.projectile_motion.1",
        ids::INSP_PJ_ACCEL,
        0.5, // LITERAL-PX-OK: m/s²
        Some(Unit::MetersPerSecondSquared),
    );
    p.numero(
        y,
        "component.field.projectile_motion.2",
        ids::INSP_PJ_MAX_SPEED,
        0.5, // LITERAL-PX-OK: m/s
        Some(Unit::MetersPerSecond),
    )
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_projectile_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorProjectileInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        core_ids::INSP_LIVE_PROJECTILE_SECTION,
        tr("component.projectile_motion.name"),
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
        core_ids::INSP_LIVE_PROJECTILE_SECTION,
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
    let mut p = Pj {
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
    cur_y = voo(&mut p, cur_y);
    cur_y = ricochete(&mut p, cur_y);
    cur_y = perseguicao(&mut p, cur_y);
    let resumo = resumo_da_rapidez(info);
    cur_y = dobra(
        &mut p,
        cur_y,
        ids::INSP_PJ_FOLD_SPEED_CHANGE,
        tr("panel.inspector.projectile.speed_change"),
        &resumo,
        rapidez,
    );
    fold.finish(store, p.scene, p.hit, cur_y)
}
