//! O Pivot de 9 pontos do cartão Look ([ficha](../../../docs/GameEngineFuncional/componentes/01_sprite.md)), ao lado do Flip.

use crate::ids;
use crate::slot_image;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget};
use ph2d_editor_core::widget::CheckboxValue;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{IconId, ImageUseEdit, SpritePivot};
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Spacing};

const CASAS_POR_LADO: f32 = 3.0; // LITERAL-PX-OK: a grade é 3 × 3, conta e não geometria
/// A marca do Flip grava os dois eixos; a do Pivot, o pivô e a posição (`marcas_inspector::mascara`).
const FLIP_KEY: [NodeId; 2] = [ids::INSP_SPRITE_FLIP_X, ids::INSP_SPRITE_FLIP_Y];

/// Os nomes dos nove pontos, por linhas.
pub(crate) const NOMES_DO_PIVOT: [&str; 9] = [
    "card.insp.pivot.top_left",
    "card.insp.pivot.top",
    "card.insp.pivot.top_right",
    "card.insp.pivot.left",
    "card.insp.pivot.center",
    "card.insp.pivot.right",
    "card.insp.pivot.bottom_left",
    "card.insp.pivot.bottom",
    "card.insp.pivot.bottom_right",
];

thread_local! {
    /// **Keep in place** está marcado? Uma preferência da sessão, não do objecto.
    static MIRROR_NO_LUGAR: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// Os objectos com os campos X e Y abertos à mão, com o pivô num dos nove pontos.
    static PIVOT_ABERTO: std::cell::RefCell<std::collections::BTreeSet<u64>> =
        const { std::cell::RefCell::new(std::collections::BTreeSet::new()) };
}

/// O Mirror object do Look mantém a imagem no lugar?
pub(crate) fn mirror_no_lugar() -> bool {
    MIRROR_NO_LUGAR.with(std::cell::Cell::get)
}

/// Os campos X e Y estão à vista? Sempre em Custom; nos nove pontos, só se a mão os abriu.
fn pivot_campos_abertos(f: &slot_image::SpriteFacts) -> bool {
    f.pivot_ponto.is_none() || PIVOT_ABERTO.with(|a| a.borrow().contains(&f.entity_bits))
}

pub(crate) fn prepara_campos_do_pivot(store: &mut WidgetStore) {
    // A diferença entre os dois espelhos só se explica aqui, e na Sprite o Mirror não diz o mesmo que noutro objecto.
    for (id, chave) in [
        (ids::INSP_SPRITE_FLIP_X, "card.insp.flip_tip"),
        (ids::INSP_SPRITE_FLIP_Y, "card.insp.flip_tip"),
        (ids::INSP_TRANSFORM_FLIP_H, "card.insp.mirror_object_tip"),
        (ids::INSP_TRANSFORM_FLIP_V, "card.insp.mirror_object_tip"),
    ] {
        store.set_tooltip(id, tr(chave));
    }
    store.register_if_absent(
        ids::INSP_SPRITE_MIRROR_KEEP,
        ph2d_editor_core::interaction::InteractiveState::Checkbox {
            state: ph2d_editor_core::widget::CheckboxState::Normal,
            value: CheckboxValue::Unchecked,
        },
    );
    store.set_tooltip(
        ids::INSP_SPRITE_MIRROR_KEEP,
        tr("card.insp.mirror_keep_tip"),
    );
    let f = slot_image::factos();
    let foco = store.focus_id();
    let mao = store.number_input_drag().map(|d| d.id);
    for (i, id) in ids::INSP_SPRITE_PIVOT_XY.into_iter().enumerate() {
        store.mark_card_field(
            id,
            ph2d_editor_core::widget::card_field::CardFieldSpec {
                step: 1.0,
                scrub: 1.0,
                default: Some(0.0),
            },
        );
        if let Some(f) = f.filter(|_| foco != Some(id) && mao != Some(id)) {
            let v = f64::from(f.pivot_px[i].round());
            if store.number_value(id) != Some(v) {
                store.set_number_value(id, v);
            }
        }
    }
}

/// A grade 3 × 3 do Pivot, o nome do ponto ao lado; devolve o fundo.
fn grade_do_pivot(c: &mut CardCtx<'_>, y: f32, f: slot_image::SpriteFacts) -> f32 {
    let theme = c.ink.theme;
    let casa = m::FIELD_H * 0.5;
    let lado = casa * CASAS_POR_LADO;
    let caixa = Rect::new(c.x, y, lado, lado);
    ph2d_editor_core::paint::fill_rounded_rect(
        c.scene,
        caixa,
        ph2d_editor_core::paint::frame_radius(theme, ph2d_tokens::Radius::Sm.px()),
        resolve(ColorToken::Bg2, theme),
    );
    let aceso = f
        .pivot_ponto
        .map(|p| usize::from(p[1]) * 3 + usize::from(p[0]));
    for (i, id) in ids::INSP_SPRITE_PIVOT.into_iter().enumerate() {
        let r = Rect::new(
            c.x + (i % 3) as f32 * casa,
            y + (i / 3) as f32 * casa,
            casa,
            casa,
        );
        let on = aceso == Some(i);
        let d = if on {
            Spacing::Sm.px()
        } else {
            Spacing::Xxs.px()
        };
        let cor = if on {
            ColorToken::Accent
        } else {
            ColorToken::Text3
        };
        ph2d_editor_core::paint::fill_rounded_rect(
            c.scene,
            Rect::new(r.x + (r.w - d) * 0.5, r.y + (r.h - d) * 0.5, d, d),
            d * 0.5,
            resolve(cor, theme),
        );
        c.hit_index.register(id, r);
    }
    let botao = [Seg {
        id: ids::INSP_SPRITE_PIVOT_NOME,
        icon: Some(IconId::More),
        text: "",
        on: pivot_campos_abertos(&f),
    }];
    let (x0, w0) = (c.x, c.w);
    (c.x, c.w) = (c.x + lado + Spacing::Xs.px(), m::FIELD_H);
    c.row(&[Cell::new(Widget::Bare(&botao))], y);
    (c.x, c.w) = (x0, w0);
    y + lado + m::FIELD_ROW_GAP
}

/// A caixa `Keep in place`, na largura inteira do cartão: o texto não cabe ao lado do Pivot no painel mínimo.
fn caixa_do_mirror(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let value = if mirror_no_lugar() {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    };
    let id = ids::INSP_SPRITE_MIRROR_KEEP;
    let label = tr("card.insp.mirror_keep").into();
    c.row(
        &[Cell::new(Widget::Check {
            id,
            label,
            value,
            off: false,
        })],
        y,
    )
}

/// Os dois espelhos e o Pivot lado a lado. Os ids do Flip continuam caixas de marcar: o clique emite o mesmo
/// `Toggled`; o Mirror é o verbo do Transform (`event_transform::giro_click`), aceso se a escala é negativa.
pub(crate) fn flip_e_pivot(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let aceso = |store: &WidgetStore, id: NodeId| {
        matches!(
            store.checkbox(id).map(|(_, v)| v),
            Some(ph2d_editor_core::widget::CheckboxValue::Checked)
        )
    };
    let segs = [
        Seg {
            id: ids::INSP_SPRITE_FLIP_X,
            icon: Some(IconId::FlipH),
            text: "",
            on: aceso(c.store, ids::INSP_SPRITE_FLIP_X),
        },
        Seg {
            id: ids::INSP_SPRITE_FLIP_Y,
            icon: Some(IconId::FlipV),
            text: "",
            on: aceso(c.store, ids::INSP_SPRITE_FLIP_Y),
        },
    ];
    let [(xa, wa), (xb, wb)] = ph2d_editor_core::panel::cards::corpo::colunas_iguais(c.x, c.w);
    let (x0, w0) = (c.x, c.w);
    (c.x, c.w) = (xa, wa);
    let fim_flip = c.row(
        &[Cell::new(Widget::Choice(&segs))
            .label(tr("card.insp.flip"))
            .key(&FLIP_KEY)],
        y,
    );
    let escala = crate::state::current_inspector_transform().map_or([1.0; 2], |t| t.scale);
    let espelhos = [
        Seg {
            id: ids::INSP_TRANSFORM_FLIP_H,
            icon: Some(IconId::FlipH),
            text: "",
            on: escala[0] < 0.0,
        },
        Seg {
            id: ids::INSP_TRANSFORM_FLIP_V,
            icon: Some(IconId::FlipV),
            text: "",
            on: escala[1] < 0.0,
        },
    ];
    let fim_espelhos = c.row(
        &[Cell::new(Widget::Choice(&espelhos)).label(tr("card.insp.mirror_object"))],
        fim_flip,
    );
    let Some(f) = crate::state::current_inspector_sprite()
        .and_then(|i| slot_image::factos().filter(|f| f.entity_bits == i.entity_bits))
    else {
        (c.x, c.w) = (x0, w0);
        return caixa_do_mirror(c, fim_espelhos);
    };
    (c.x, c.w) = (xb, wb);
    c.row(
        &[Cell::new(Widget::Readout { text: "" })
            .label(tr("card.insp.pivot"))
            .key(&ids::INSP_SPRITE_PIVOT_XY)],
        y,
    );
    // A grade começa onde a caixa do Flip começa: logo abaixo dos rótulos.
    let topo = fim_flip - m::FIELD_H;
    let fim_grade = grade_do_pivot(c, topo, f);
    (c.x, c.w) = (x0, w0);
    let fim = caixa_do_mirror(c, fim_espelhos.max(fim_grade));
    if !pivot_campos_abertos(&f) {
        return fim;
    }
    let (gx, gy) = (tr("card.glyph.x"), tr("card.glyph.y"));
    c.row(
        &[
            Cell::new(Widget::Field {
                id: ids::INSP_SPRITE_PIVOT_XY[0],
                glyph: ph2d_editor_core::widget::card_field::Glyph::Text(gx),
            })
            .label(tr("card.insp.pivot_px")),
            Cell::new(Widget::Field {
                id: ids::INSP_SPRITE_PIVOT_XY[1],
                glyph: ph2d_editor_core::widget::card_field::Glyph::Text(gy),
            }),
        ],
        fim,
    )
}

/// Os cliques e os números do Pivot. `true` = consumido.
pub(crate) fn evento_do_pivot(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let pivot = match ev {
        WidgetEvent::Toggled(id) if id == ids::INSP_SPRITE_MIRROR_KEEP => {
            MIRROR_NO_LUGAR.with(|m| m.set(!m.get()));
            return true;
        }
        WidgetEvent::Click(id) if id == ids::INSP_SPRITE_PIVOT_NOME => {
            if let Some(s) = crate::state::current_inspector_sprite() {
                PIVOT_ABERTO.with(|a| {
                    let mut a = a.borrow_mut();
                    if !a.remove(&s.entity_bits) {
                        a.insert(s.entity_bits);
                    }
                });
            }
            return true;
        }
        WidgetEvent::Click(id) => {
            let Some(i) = ids::INSP_SPRITE_PIVOT.iter().position(|o| *o == id) else {
                return false;
            };
            SpritePivot::Ponto([(i % 3) as u8, (i / 3) as u8])
        }
        WidgetEvent::ValueChanged(id) if ids::INSP_SPRITE_PIVOT_XY.contains(&id) => {
            // Só o eixo mexido muda: a caixa do outro está arredondada, e o exacto é o do objecto.
            let mut px = slot_image::factos().map_or([0.0; 2], |f| f.pivot_px);
            for (i, n) in ids::INSP_SPRITE_PIVOT_XY.into_iter().enumerate() {
                if n == id
                    && let Some(v) = host.store().number_value(n)
                {
                    px[i] = v as f32;
                }
            }
            SpritePivot::Px(px)
        }
        _ => return false,
    };
    let Some(sprite) = crate::state::current_inspector_sprite() else {
        return true;
    };
    host.bus_mut().push(EditorAction::ImageUseEdit {
        entity_bits: sprite.entity_bits,
        edit: ImageUseEdit::Pivot(pivot),
    });
    true
}
