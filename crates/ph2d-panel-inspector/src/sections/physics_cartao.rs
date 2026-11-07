//! **Os cartões Body e Shape da física** (ficha `docs/GameEngineFuncional/componentes/f1_physics_static_shape.md`,
//! opção B): «como se move» e «em que bate» em dois cartões, o resto em dobras com nome. Os ids são os de
//! sempre, e o destino de cada campo é a tabela `M` da maquete. `PH2D_CARDS=0` continua em `physics_body.rs`.

use super::physics_body::{
    BAKE_CH_LABELS, CCD_LABELS, FREEZE_POS_LABELS, KIND_DYNAMIC, KIND_LABELS, LOCKROT_LABELS,
};
use super::physics_rows::{SENSOR_LABELS, paint_damping_rows, paint_mass_source};
use super::rows::{Campo, Escolha, num_row_unit, par_row, seg_par_row, seg_row};
use super::*;
use ph2d_editor_core::interaction::format_number;
use ph2d_editor_core::screens::hero::InspectorPhysicsInfo;
use ph2d_editor_core::widget::SectionFold;
use ph2d_editor_core::widget::key_mark::Eixos;
use ph2d_i18n::{TextKey, tr, tr_with};

pub(crate) use crate::ids::CARD_PHYS_SHAPE;

/// Onde um cartão de física pinta.
pub(super) struct Tinta<'a> {
    pub scene: &'a mut VectorScene,
    pub text: &'a mut TextSystem,
    pub theme: Theme,
    pub hit: &'a mut HitIndex,
    pub store: &'a WidgetStore,
    pub x: f32,
    pub w: f32,
}

impl Tinta<'_> {
    pub(super) fn frase(&mut self, y: f32, texto: &str, token: ColorToken) -> f32 {
        rows::aviso(
            self.scene, self.text, self.theme, self.x, self.w, y, texto, token,
        )
    }

    /// Pinta o botão e devolve a caixa e o `y` de baixo; o chamador regista o id literal.
    pub(super) fn botao(&mut self, id: NodeId, texto: &str, y: f32) -> (Rect, f32) {
        let rect =
            ph2d_editor_core::property_row::caixa_do_botao(self.text, self.x, self.w, y, texto);
        let btn = Button::new(id, texto)
            .kind(ButtonKind::Default)
            .visual(self.store.button_visual(id));
        paint_button(&btn, rect, self.scene, self.text, self.theme);
        (rect, ph2d_editor_core::property_row::abaixo_do_botao(rect))
    }

    /// Uma dobra com nome e, fechada, o resumo à direita.
    pub(super) fn dobra(
        &mut self,
        y: f32,
        id: NodeId,
        titulo: &str,
        resumo: &str,
        corpo: impl FnOnce(&mut Self, f32) -> f32,
    ) -> f32 {
        let g = rows::grupo_dobravel(
            self.scene, self.text, self.theme, self.hit, self.store, self.x, self.w, y, id, titulo,
        );
        let fim = if g.aberto() {
            corpo(self, g.y)
        } else {
            rows::resumo_do_grupo(
                self.scene, self.text, self.theme, self.x, self.w, g.y, titulo, resumo,
            );
            g.y
        };
        g.fecha(self.store, self.scene, self.hit, fim)
    }

    pub(super) fn numero(&self, id: NodeId) -> String {
        format_number(self.store.number_value(id).unwrap_or(0.0))
    }
}

/// As dobras nascem fechadas, só aqui: `PH2D_CARDS=0` não as tem. `PH2D_FISICA_DOBRAS=1` abre-as para a foto.
pub(crate) fn prepara(store: &mut WidgetStore) {
    static ABERTAS: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let abertas = *ABERTAS.get_or_init(|| std::env::var_os("PH2D_FISICA_DOBRAS").is_some());
    for id in ids::INSP_PHYS_FOLDS {
        store.set_collapsed_if_unchosen(id, !abertas);
    }
    // O cartão Body nasce aberto como o Shape (maquete: travar a rotação é um clique), uma vez: depois
    // manda o que o artista dobrou. As secções de componente nascem dobradas no `populate`, daí a marca.
    if store.collapsed_choice(ids::INSP_PHYS_BODY_OPENED).is_none() {
        store.set_collapsed(core_ids::INSP_LIVE_PHYSICS_SECTION, false);
        store.set_collapsed(ids::INSP_PHYS_BODY_OPENED, false);
    }
}

/// O cartão **Body** de um corpo.
pub(super) fn corpo(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let dinamico = info.kind_tag == KIND_DYNAMIC;
    let mut y = seg_row(
        t.scene,
        t.text,
        t.theme,
        t.hit,
        t.store,
        t.x,
        t.w,
        y,
        TextKey::new("component.field.rigid_body.0"),
        core_ids::INSP_LIVE_PHYSICS_SECTION,
        &ids::INSP_PHYS_KIND,
        &KIND_LABELS.map(TextKey::tr),
        info.kind_tag,
    );
    if dinamico {
        y = num_row_unit(
            t.scene,
            t.text,
            t.theme,
            t.hit,
            t.store,
            t.x,
            t.w,
            y,
            TextKey::new("component.field.gravity_scale.0"),
            ids::INSP_PHYS_GRAVITY_SCALE,
            None,
            None,
        );
        y = seg_row(
            t.scene,
            t.text,
            t.theme,
            t.hit,
            t.store,
            t.x,
            t.w,
            y,
            tr("panel.inspector.physics.rotation"),
            ids::INSP_LIVE_PHYSICS_LOCKROT,
            &ids::INSP_PHYS_LOCKROT,
            &LOCKROT_LABELS.map(TextKey::tr),
            u8::from(info.lock_rotation),
        );
    }
    // Density some de quem a massa ninguém lê; o valor fica no `Collider` e volta com Dynamic.
    if info.mass_is_read {
        y = paint_mass_source(
            t.scene,
            t.text,
            t.theme,
            t.hit,
            t.store,
            t.x,
            t.w,
            y,
            info.mass_is_read,
            info.mass_manual,
        );
    }
    if dinamico {
        y = dobra_start(t, y, info);
        y = dobra_advanced(t, y, info);
    }
    y = dobra_joint(t, y, info);
    if dinamico {
        y = dobra_bake(t, y, info);
    }
    let (r, y) = t.botao(
        ids::INSP_PHYS_REMOVE,
        tr("panel.inspector.physics.remove_physics_body"),
        y,
    );
    t.hit.register(ids::INSP_PHYS_REMOVE, r);
    y - Spacing::Sm.px() + SECTION_BOTTOM_PAD_PX
}

fn dobra_start(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let parado = info.linvel == [0.0, 0.0] && info.angvel == 0.0;
    let resumo = tr(if parado {
        "card.insp.phys.still"
    } else {
        "card.insp.phys.moving"
    });
    let titulo = tr("card.insp.phys.start_moving");
    t.dobra(y, ids::INSP_PHYS_FOLD_START, titulo, resumo, |t, y| {
        let v = Some(ph2d_editor_core::widget::Unit::MetersPerSecond);
        let rotulo = tr("component.field.initial_velocity.0");
        let (vx, vy) = (
            tr_with("panel.inspector.label_x", &[("label", &rotulo)]),
            tr_with("panel.inspector.label_y", &[("label", &rotulo)]),
        );
        let y = par_row(
            t.scene,
            t.text,
            t.theme,
            t.hit,
            t.store,
            t.x,
            t.w,
            y,
            Some((tr("panel.inspector.physics.init_velocity"), Eixos::Xyzw)),
            [
                Campo::new(&vx, ids::INSP_PHYS_LINVEL_X, v),
                Campo::new(&vy, ids::INSP_PHYS_LINVEL_Y, v),
            ],
        );
        num_row_unit(
            t.scene,
            t.text,
            t.theme,
            t.hit,
            t.store,
            t.x,
            t.w,
            y,
            TextKey::new("component.field.initial_velocity.1"),
            ids::INSP_PHYS_ANGVEL,
            Some(ph2d_editor_core::widget::Unit::DegreesPerSecond),
            None,
        )
    })
}

fn dobra_advanced(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let colisao = CCD_LABELS[usize::from(info.ccd)].tr();
    let dominancia = tr_with("card.insp.phys.dominance", &[("n", &info.dominance)]);
    let resumo = format!("{colisao} \u{b7} {dominancia}");
    let titulo = tr("card.insp.phys.advanced");
    t.dobra(y, ids::INSP_PHYS_FOLD_ADVANCED, titulo, &resumo, |t, y| {
        let y = seg_row(
            t.scene,
            t.text,
            t.theme,
            t.hit,
            t.store,
            t.x,
            t.w,
            y,
            tr("panel.inspector.physics.collision"),
            ids::INSP_LIVE_PHYSICS_CCD,
            &ids::INSP_PHYS_CCD,
            &CCD_LABELS.map(TextKey::tr),
            u8::from(info.ccd),
        );
        let y = num_row_unit(
            t.scene,
            t.text,
            t.theme,
            t.hit,
            t.store,
            t.x,
            t.w,
            y,
            TextKey::new("component.field.dominance.0"),
            ids::INSP_PHYS_DOMINANCE,
            None,
            None,
        );
        let freeze = FREEZE_POS_LABELS.map(TextKey::tr);
        let y = seg_par_row(
            t.scene,
            t.text,
            t.theme,
            t.hit,
            t.store,
            t.x,
            t.w,
            y,
            Escolha {
                rotulo: tr("panel.inspector.physics.freeze_x").into(),
                group: ids::INSP_LIVE_PHYSICS_LOCKX,
                ids: &ids::INSP_PHYS_LOCKX,
                labels: &freeze,
                selected: u8::from(info.lock_x),
            },
            Escolha {
                rotulo: tr("panel.inspector.physics.freeze_y").into(),
                group: ids::INSP_LIVE_PHYSICS_LOCKY,
                ids: &ids::INSP_PHYS_LOCKY,
                labels: &freeze,
                selected: u8::from(info.lock_y),
            },
        );
        paint_damping_rows(
            t.scene,
            t.text,
            t.theme,
            t.hit,
            t.store,
            t.x,
            t.w,
            y,
            info.damp_mode_tag,
        )
    })
}

fn dobra_joint(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let resumo = super::physics_join_rows::JOIN_KIND_LABELS
        .get(usize::from(info.join_kind_tag))
        .map_or("", |k| k.tr());
    let titulo = tr("card.insp.phys.joint");
    t.dobra(y, ids::INSP_PHYS_FOLD_JOINT, titulo, resumo, |t, y| {
        super::physics_join_rows::paint_join_gesture(
            t.scene,
            t.text,
            t.theme,
            t.hit,
            t.store,
            t.x,
            t.w,
            y,
            info.join_kind_tag,
            info.join_count,
            info.join_draw_armed,
            info.rig_parts,
        )
    })
}

fn dobra_bake(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let resumo = BAKE_CH_LABELS
        .get(usize::from(info.bake_channels_tag))
        .map_or("", |k| k.tr());
    let titulo = tr("card.insp.phys.bake");
    t.dobra(y, ids::INSP_PHYS_FOLD_BAKE, titulo, resumo, |t, y| {
        let y = seg_row(
            t.scene,
            t.text,
            t.theme,
            t.hit,
            t.store,
            t.x,
            t.w,
            y,
            tr("panel.inspector.physics.bake"),
            ids::INSP_PHYS_BAKE_CH_GROUP,
            &ids::INSP_PHYS_BAKE_CH,
            &BAKE_CH_LABELS.map(TextKey::tr),
            info.bake_channels_tag,
        );
        let rotulo = super::physics::bake_label(info.bake_start_seconds, info.bake_seconds);
        let (r, y) = t.botao(ids::INSP_PHYS_BAKE, &rotulo, y);
        t.hit.register(ids::INSP_PHYS_BAKE, r);
        y
    })
}

/// O recibo do Remove, se houver: a frase e o «Undo».
pub(super) fn recibo(t: &mut Tinta<'_>, y: f32) -> f32 {
    if !crate::cards::active() {
        return y;
    }
    let Some(forma) = crate::recibo_fisica::a_vista() else {
        return y;
    };
    let frase = tr(if forma {
        "card.insp.phys.removed_shape"
    } else {
        "card.insp.phys.removed_body"
    });
    let y = t.frase(y, frase, ColorToken::Text2);
    let (r, y) = t.botao(ids::INSP_PHYS_REMOVED_UNDO, tr("card.insp.phys.undo"), y);
    t.hit.register(ids::INSP_PHYS_REMOVED_UNDO, r);
    y
}

/// A secção que só segura o recibo: o objecto ficou sem física nenhuma.
#[allow(clippy::too_many_arguments)]
pub(crate) fn secao_do_recibo(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    hit: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let id = core_ids::INSP_LIVE_PHYSICS_SECTION;
    let header = section_header(store, id, tr("component.rigid_body.name"));
    paint_section_header(&header, Rect::new(x, y, w, header_h), scene, text, theme);
    let top = y + header_h;
    let Some(fold) = SectionFold::begin(store, id, x, w, top, scene, hit) else {
        return top;
    };
    let mut t = Tinta {
        scene: &mut *scene,
        text,
        theme,
        hit: &mut *hit,
        store,
        x,
        w,
    };
    let fim = recibo(&mut t, top) + SECTION_BOTTOM_PAD_PX;
    fold.finish(store, scene, hit, fim)
}

/// O que o título dos cartões de física diz quando dobrado: o tipo; a forma e o toque.
pub(crate) fn resumo_do_cartao(id: NodeId) -> Option<String> {
    let info = crate::state::current_inspector_physics()?;
    let forma = || {
        let f = super::physics::SHAPE_LABELS.get(usize::from(info.shape_tag))?;
        let toque = SENSOR_LABELS[usize::from(info.is_sensor)].tr();
        Some(format!("{} \u{b7} {toque}", f.tr()))
    };
    if id == CARD_PHYS_SHAPE {
        forma()
    } else if id == core_ids::INSP_LIVE_PHYSICS_SECTION && info.has_body {
        KIND_LABELS
            .get(usize::from(info.kind_tag))
            .map(|k| k.tr().to_string())
    } else if id == core_ids::INSP_LIVE_PHYSICS_SECTION && info.has_collider {
        forma()
    } else {
        None
    }
}
