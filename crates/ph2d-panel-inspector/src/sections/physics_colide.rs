//! ⭐ **«Collides with»** (ficha f1, onda 6): a linha da matriz global, vista e editada a partir do
//! objecto. Um quadro por camada visível; clicar é a mesma edição da célula da matriz.

use super::physics_cartao::Tinta;
use crate::camadas_fisica as camadas;
use crate::ids;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::paint::{
    fill_rounded_rect, folga_do_texto, frame_radius, paint_text_elided, resolve,
};
use ph2d_editor_core::panel::{PanelHostInternal, seam_reset_button};
use ph2d_editor_core::screens::hero::InspectorPhysicsInfo;
use ph2d_editor_core::widget::card_ink::tinta;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, TypeToken};

/// Sem corpo acima nada bate na forma: os quadros desmentiriam a frase de cima.
#[must_use]
pub(super) fn solta(info: &InspectorPhysicsInfo) -> bool {
    !info.has_body && info.part_owner.is_empty()
}

/// As camadas da fila: as visíveis e, mesmo fora delas, a do objecto.
fn da_fila(camada: usize) -> impl Iterator<Item = usize> {
    (0..ids::INSP_PHYS_LAYERS).filter(move |&j| camadas::visivel(j) || j == camada)
}

/// A caixa de um quadro em `x`; a largura sai do nome.
fn largura(t: &mut Tinta<'_>, texto: &str, com_caixa: bool) -> f32 {
    let (fonte, folga) = (TypeToken::Sm.px(), Spacing::Xs.px());
    let caixa = if com_caixa { fonte + folga } else { 0.0 };
    let medida = t.text.prefix_width(texto, fonte).ceil();
    folga + caixa + medida + folga
}

/// O fundo com a caixinha acesa ou apagada (ou sem ela, no «+») e o nome.
fn pinta(t: &mut Tinta<'_>, r: Rect, texto: &str, id: ph2d_a11y::NodeId, aceso: Option<bool>) {
    let (fonte, folga) = (TypeToken::Sm.px(), Spacing::Xs.px());
    let raio = frame_radius(t.theme, Radius::Sm.px());
    let fundo = if t.store.hover_live(id) > 0.0 {
        ColorToken::CardFieldHover
    } else {
        ColorToken::CardField
    };
    fill_rounded_rect(t.scene, r, raio, tinta(fundo, t.theme));
    let mut x = r.x + folga;
    if let Some(on) = aceso {
        let caixa = Rect::new(x, r.y + (r.h - fonte) * 0.5, fonte, fonte);
        if on {
            fill_rounded_rect(t.scene, caixa, raio, resolve(ColorToken::Accent, t.theme));
        } else {
            // FRAME-RAW-OK: o quadro apagado é só o contorno; a forma é o que distingue os dois estados
            ph2d_editor_core::paint::stroke_rounded_rect(
                t.scene,
                caixa,
                raio,
                StrokeToken::Default.px(),
                tinta(ColorToken::CardLabel, t.theme),
            );
        }
        x += fonte + folga;
    }
    let cor = if aceso == Some(false) {
        ColorToken::CardLabel
    } else {
        ColorToken::KanbanCardFg
    };
    paint_text_elided(
        t.text,
        t.scene,
        texto,
        x,
        r.y + folga_do_texto(r.h, fonte),
        fonte,
        (r.x + r.w - folga - x).max(0.0),
        tinta(cor, t.theme),
    );
    ph2d_editor_core::panel::cards::censo::controlo_maior_que_a_pega(id, r);
    t.hit.register(id, r);
}

/// O rótulo com a marca, os quadros quebrando pelo tamanho do nome, a frase e o «Edit matrix…».
pub(super) fn colide_com(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let camada = usize::from(info.layer);
    let rotulo = tr("card.insp.phys.collides_with");
    let seccao = ph2d_editor_core::property_row::Seccao::medida(t.text, 1, &[rotulo]);
    let linha = ph2d_editor_core::property_row::paint_label_row(
        t.scene,
        t.text,
        t.theme,
        t.x,
        t.w,
        y,
        ph2d_tokens::ROW_H_PX,
        rotulo,
        seccao,
    );
    if let Some(alvo) = ph2d_editor_core::widget::key_mark::paint_decorator_for(
        t.scene,
        t.theme,
        linha.dot,
        &ids::INSP_PHYS_COLLIDES,
    ) {
        t.hit.register(alvo, linha.dot);
    }
    let c = linha.control;
    let gap = Spacing::Xs.px();
    let (mut cx, mut cy) = (c.x, c.y);
    let mut assenta = |cw: f32| {
        if cx > c.x && cx + cw > c.x + c.w {
            cx = c.x;
            cy += c.h + gap;
        }
        let r = Rect::new(cx, cy, cw.min(c.w), c.h);
        cx += r.w + gap;
        (r, cy)
    };
    let mut fundo = c.y;
    for j in da_fila(camada) {
        let nome = camadas::rotulo(j);
        let cw = largura(t, &nome, true);
        let (r, ultima) = assenta(cw);
        fundo = ultima;
        pinta(
            t,
            r,
            &nome,
            ids::INSP_PHYS_COLLIDES[j],
            Some(camadas::colide(camada, j)),
        );
    }
    // Nas 32 não há mais camada para mostrar: o «+» sai.
    if camadas::proxima_livre().is_some() {
        let mais = tr("card.insp.phys.collides_add");
        let cw = largura(t, mais, false);
        let (r, ultima) = assenta(cw);
        fundo = ultima;
        pinta(t, r, mais, ids::INSP_PHYS_COLLIDES_ADD, None);
    }
    ph2d_editor_core::widget::composto::grupo(ids::INSP_PHYS_COLLIDES.iter().copied());
    let y = linha.fim(ph2d_tokens::row_pitch_px()) + (fundo - c.y);
    let frase = tr_with(
        "card.insp.phys.collides_applies",
        &[("layer", &camadas::rotulo(camada))],
    );
    t.frase(y, &frase, ColorToken::Text3)
}

/// As dicas dos quadros: dizem, com os nomes de hoje, o que o próximo clique faz.
pub(crate) fn prepara_as_dicas(store: &mut WidgetStore, camada: usize) {
    let este = camadas::rotulo(camada);
    for j in da_fila(camada) {
        let aceso = camadas::colide(camada, j);
        let chave = match (j == camada, aceso) {
            (true, true) => "card.insp.phys.collides_self_on",
            (true, false) => "card.insp.phys.collides_self_off",
            (false, true) => "card.insp.phys.collides_on",
            (false, false) => "card.insp.phys.collides_off",
        };
        let outra = camadas::rotulo(j);
        let dica = tr_with(chave, &[("this", &este), ("other", &outra)]);
        let id = ids::INSP_PHYS_COLLIDES[j];
        if store.tooltip_for(id) != Some(dica.as_str()) {
            store.set_tooltip(id, dica);
        }
    }
}

/// Os cliques da fila. `true` = era dela. Uma forma solta não pinta quadros, e o clique não chega aqui.
pub(crate) fn clique(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = ev else {
        return false;
    };
    let Some(info) = crate::state::current_inspector_physics() else {
        return false;
    };
    let a = info.layer;
    if let Some(b) = ids::INSP_PHYS_COLLIDES.iter().position(|&o| o == id) {
        seam_reset_button(host, id);
        if !solta(&info) {
            host.bus_mut()
                .push(EditorAction::LayerPairToggle { a, b: b as u8 });
        }
        return true;
    }
    if id == ids::INSP_PHYS_COLLIDES_ADD {
        seam_reset_button(host, id);
        if let Some(layer) = camadas::proxima_livre() {
            host.bus_mut().push(EditorAction::LayerShow { layer });
        }
        return true;
    }
    if id == ids::INSP_PHYS_EDIT_MATRIX {
        seam_reset_button(host, id);
        host.bus_mut()
            .push(EditorAction::LayerMatrixOpen { layer: a });
        return true;
    }
    false
}
