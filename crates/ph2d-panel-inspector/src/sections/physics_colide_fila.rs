//! ⭐ **A fila de quadros de camada** — «Collides with» da forma e «Sees» do Ray Sensor. Na forma (GEF02 B9, dono
//! 08/11) os quadros mostram a linha EFETIVA dela (a matriz + a exceção), misto quando a seleção diverge, e o clique
//! edita a exceção da forma (`CollidesWith`), nunca a matriz. O «Sees» continua a editar a matriz.

use super::physics_cartao::Tinta;
use super::physics_colide::{DO_COLLIDER, Quem, da_fila};
use crate::camadas_fisica as camadas;
use crate::ids;
use ph2d_editor_core::PhysicsFieldEdit;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::paint::{
    fill_rounded_rect, folga_do_texto, frame_radius, paint_layer_outline, paint_text_elided,
    resolve,
};
use ph2d_editor_core::panel::{PanelHostInternal, seam_reset_button};
use ph2d_editor_core::screens::hero::InspectorPhysicsInfo;
use ph2d_editor_core::widget::CheckboxValue;
use ph2d_editor_core::widget::card_ink::tinta;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, TypeToken};

/// O estado do quadro `j` numa linha com bits `misto` divergentes.
fn valor(linha: u32, misto: u32, j: usize) -> CheckboxValue {
    if misto & (1 << j) != 0 {
        CheckboxValue::Indeterminate
    } else if linha & (1 << j) != 0 {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    }
}

/// A linha efetiva da forma: a da matriz viva com a exceção dela (`ph2d_physics::world::layer_override::effective_row`).
#[must_use]
pub(super) fn linha(info: &InspectorPhysicsInfo) -> u32 {
    let [include, exclude] = info.collides_override;
    (camadas::linha(usize::from(info.layer)) | include) & !exclude
}

/// «Collides with» da forma: a linha efetiva, os mistos e, com exceção, «Differs from layer X» + «Use layer default».
pub(super) fn colide_com(t: &mut Tinta<'_>, y: f32, info: &InspectorPhysicsInfo) -> f32 {
    let rotulo = tr("card.insp.phys.collides_with");
    let camada = usize::from(info.layer);
    let chave = if info.collides_differs {
        "card.insp.phys.collides_differs"
    } else {
        "card.insp.phys.collides_applies"
    };
    let frase = tr_with(chave, &[("layer", &camadas::rotulo(camada))]);
    let estado = |j| valor(linha(info), info.collides_mixed, j);
    // Dono 09/11: a frase e o nome dos quadros fixados pela exceção vão em destaque (a mesma cor).
    let destaque = Destaque {
        fixos: info.collides_pinned,
        frase: info.collides_differs,
    };
    let y = fila(
        t,
        y,
        (&DO_COLLIDER, camada),
        rotulo,
        &estado,
        (&frase, destaque),
    );
    if !info.collides_differs {
        return y;
    }
    let (r, y) = t.botao(
        ids::INSP_PHYS_LAYER_DEFAULT,
        tr("card.insp.phys.use_layer_default"),
        y,
    );
    t.hit.register(ids::INSP_PHYS_LAYER_DEFAULT, r);
    y
}

/// «Sees» do Ray Sensor: a linha `camada` da matriz do mundo.
pub(super) fn fila_da_matriz(
    t: &mut Tinta<'_>,
    y: f32,
    quem: &Quem,
    camada: usize,
    rotulo: &str,
) -> f32 {
    let frase = tr_with(
        "card.insp.phys.collides_applies",
        &[("layer", &camadas::rotulo(camada))],
    );
    let estado = |j| valor(camadas::linha(camada), 0, j);
    fila(
        t,
        y,
        (quem, camada),
        rotulo,
        &estado,
        (&frase, Destaque::default()),
    )
}

/// O que a fila pinta em destaque: os quadros `fixos` (bits) e a `frase`.
#[derive(Clone, Copy, Default)]
struct Destaque {
    fixos: u32,
    frase: bool,
}

/// O clique num quadro da forma: o bit vira o contrário do que se vê (misto ⇒ aceso para todas), para a seleção
/// inteira (a shell espalha). `true` = era dela.
pub(crate) fn clique_da_forma(
    host: &mut dyn PanelHostInternal,
    id: ph2d_a11y::NodeId,
    info: &InspectorPhysicsInfo,
) -> bool {
    let edit = if let Some(j) = DO_COLLIDER.quadros.iter().position(|&o| o == id) {
        let on = valor(linha(info), info.collides_mixed, j) != CheckboxValue::Checked;
        PhysicsFieldEdit::CollidesWith { layer: j as u8, on }
    } else if id == ids::INSP_PHYS_LAYER_DEFAULT {
        PhysicsFieldEdit::UseLayerDefault
    } else {
        return false;
    };
    seam_reset_button(host, id);
    if !super::physics_colide::solta(info) {
        host.bus_mut().push(EditorAction::InspectorPhysicsEdit {
            entity_bits: info.entity_bits,
            edit,
        });
    }
    true
}

/// A caixa de um quadro em `x`; a largura sai do nome.
fn largura(t: &mut Tinta<'_>, texto: &str, com_caixa: bool) -> f32 {
    let (fonte, folga) = (TypeToken::Sm.px(), Spacing::Xs.px());
    let caixa = if com_caixa { fonte + folga } else { 0.0 };
    let medida = t.text.prefix_width(texto, fonte).ceil();
    folga + caixa + medida + folga
}

/// A caixinha do quadro: cheia (acesa), só o contorno (apagada) ou cheia com um traço (mista).
fn caixinha(t: &mut Tinta<'_>, caixa: Rect, v: CheckboxValue) {
    let raio = frame_radius(t.theme, Radius::Sm.px());
    if v == CheckboxValue::Unchecked {
        // FRAME-RAW-OK: o quadro apagado é só o contorno; a forma é o que distingue os dois estados
        ph2d_editor_core::paint::stroke_rounded_rect(
            t.scene,
            caixa,
            raio,
            StrokeToken::Default.px(),
            tinta(ColorToken::CardLabel, t.theme),
        );
        return;
    }
    fill_rounded_rect(t.scene, caixa, raio, resolve(ColorToken::Accent, t.theme));
    if v == CheckboxValue::Indeterminate {
        let (traco, lado) = (StrokeToken::Default.px(), caixa.w * 0.5);
        let barra = Rect::new(
            caixa.x + (caixa.w - lado) * 0.5,
            caixa.y + (caixa.h - traco) * 0.5,
            lado,
            traco,
        );
        fill_rounded_rect(
            t.scene,
            barra,
            traco * 0.5,
            tinta(ColorToken::KanbanBtnOnFg, t.theme),
        );
    }
}

/// O fundo com a caixinha (ou sem ela, no «+») e o nome.
fn pinta(
    t: &mut Tinta<'_>,
    r: Rect,
    texto: &str,
    (id, quem): (ph2d_a11y::NodeId, &Quem),
    (aceso, destaque): (Option<CheckboxValue>, bool),
) {
    let (fonte, folga) = (TypeToken::Sm.px(), Spacing::Xs.px());
    let raio = frame_radius(t.theme, Radius::Sm.px());
    let fundo = if t.store.hover_live(id) > 0.0 {
        ColorToken::CardFieldHover
    } else if destaque {
        ColorToken::WarnSoft
    } else {
        ColorToken::CardField
    };
    fill_rounded_rect(t.scene, r, raio, tinta(fundo, t.theme));
    let mut x = r.x + folga;
    if let Some(v) = aceso {
        caixinha(t, Rect::new(x, r.y + (r.h - fonte) * 0.5, fonte, fonte), v);
        x += fonte + folga;
    }
    let apagado = aceso == Some(CheckboxValue::Unchecked);
    let cor = if destaque {
        ColorToken::Warn
    } else if apagado {
        ColorToken::CardLabel
    } else {
        ColorToken::KanbanCardFg
    };
    let (topo, cabe) = (
        r.y + folga_do_texto(r.h, fonte),
        (r.x + r.w - folga - x).max(0.0),
    );
    paint_text_elided(
        t.text,
        t.scene,
        texto,
        x,
        topo,
        fonte,
        cabe,
        tinta(cor, t.theme),
    );
    // O contorno da cor da camada na moldura do quadro; desligado, mais apagado (onda 9f; o «+» não tem camada).
    if let Some(j) = quem.quadros.iter().position(|&o| o == id) {
        paint_layer_outline(t.scene, t.theme, id, j, r, raio, !apagado);
    }
    ph2d_editor_core::panel::cards::censo::controlo_maior_que_a_pega(id, r);
    t.hit.register(id, r);
}

/// O rótulo com a marca, os quadros (quebrando pelo tamanho do nome) no `estado` de cada um, o «+» e a `frase`.
fn fila(
    t: &mut Tinta<'_>,
    y: f32,
    (quem, camada): (&Quem, usize),
    rotulo: &str,
    estado: &dyn Fn(usize) -> CheckboxValue,
    (frase, destaque): (&str, Destaque),
) -> f32 {
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
        quem.quadros,
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
        let fixo = destaque.fixos & (1 << j) != 0;
        pinta(
            t,
            r,
            &nome,
            (quem.quadros[j], quem),
            (Some(estado(j)), fixo),
        );
    }
    // Nas 32 não há mais camada para mostrar: o «+» sai.
    if camadas::proxima_livre().is_some() {
        let mais = tr("card.insp.phys.collides_add");
        let cw = largura(t, mais, false);
        // A coluna estreita (a face clássica ao lado do nome) cortaria o «+»: ele desce a fila e usa a largura toda.
        let (r, ultima) = if cw > c.w {
            let abaixo = cy + c.h + gap;
            (Rect::new(t.x, abaixo, cw.min(t.w), c.h), abaixo)
        } else {
            assenta(cw)
        };
        fundo = ultima;
        pinta(t, r, mais, (quem.mais, quem), (None, false));
    }
    // O «+» é o fim da mesma fila: um composto, uma capacidade.
    let fila = quem.quadros.iter().copied().chain([quem.mais]);
    ph2d_editor_core::widget::composto::grupo(fila);
    let y = linha.fim(ph2d_tokens::row_pitch_px()) + (fundo - c.y);
    let tom = if destaque.frase {
        ColorToken::Warn
    } else {
        ColorToken::Text3
    };
    let fim = t.frase(y, frase, tom);
    t.hit.register(quem.frase, Rect::new(t.x, y, t.w, fim - y));
    fim
}
