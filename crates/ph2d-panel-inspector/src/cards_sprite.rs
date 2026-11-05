//! ⭐ **Os cartões da Sprite**, versão B da revisão de 05/11
//! ([ficha](../../../docs/GameEngineFuncional/componentes/01_sprite.md)): Image · Look · Transform dobrado ·
//! Ordering · Advanced. Os ids são os de sempre; `PH2D_CARDS=0` pinta o painel de antes.

use crate::cards_componentes::Presente;
use crate::cards_objeto::com_corpo;
use crate::ids;
use crate::sections;
use crate::slot_image::{self, SlotImage};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, HeadAction, Seg, Widget};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::screens::hero::{
    InspectorSpriteInfo, InspectorSpriteSource, InspectorTransformInfo,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{IconId, ImageUseEdit, InspectorImageUse, SpritePivot};
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, Spacing, TypeToken};

/// A miniatura mede três campos: a imagem é o que se lê primeiro no cartão (Godot, GameMaker).
const LINHAS_DA_MINIATURA: f32 = 3.0;
/// O convite do cartão vazio mede cinco campos: é o único alvo do cartão.
const LINHAS_DO_CONVITE: f32 = 5.0;

fn botao(store: &mut WidgetStore, id: NodeId, dica: &str) {
    store.register_if_absent(
        id,
        InteractiveState::Button {
            state: ph2d_editor_core::widget::ButtonState::Normal,
        },
    );
    if store.tooltip_for(id).is_none() {
        store.set_tooltip(id, tr(dica));
    }
}

/// Os botões e as dobras dos cartões da Sprite: uma vez por quadro, antes da pintura.
pub(crate) fn prepara(store: &mut WidgetStore) {
    botao(store, ids::INSP_IMAGE_REPLACE, "card.insp.image_replace_tip");
    botao(store, ids::INSP_IMAGE_KEEP_SIZE, "card.insp.keep_old_size_tip");
    botao(store, ids::INSP_IMAGE_EDIT_PIXEL, "card.insp.edit_in_pixel_tip");
    botao(store, ids::INSP_IMAGE_EDIT_PAINTER, "card.insp.edit_in_painter_tip");
    botao(store, ids::INSP_IMAGE_PAINT_NEW, "card.insp.paint_new_tip");
    for id in ids::INSP_IMAGE_RECENT {
        botao(store, id, "card.insp.recent_tip");
    }
    let selecionada = crate::state::current_inspector_sprite().map(|s| s.entity_bits);
    if slot_image::take_pintar_no_lugar_de().is_some_and(|b| Some(b) == selecionada) {
        slot_image::set_pintar_no_lugar_de(selecionada);
    }
    if let Some(r) = slot_image::recibo()
        && crate::state::current_inspector_sprite().map(|s| s.entity_bits) != Some(r.entity_bits)
    {
        slot_image::set_recibo(None);
    }
    botao(
        store,
        ids::INSP_IMAGE_SHEET_ANIMATED,
        "card.insp.sheet_make_animated_tip",
    );
    botao(
        store,
        ids::INSP_IMAGE_SHEET_REGION,
        "card.insp.sheet_use_region_tip",
    );
    store.set_collapsed_if_unchosen(crate::cards::CARD_TRANSFORM_IMAGE, true);
    botao(store, ids::INSP_SPRITE_PIVOT_NOME, "card.insp.pivot_nome_tip");
    for (id, nome) in ids::INSP_SPRITE_PIVOT.into_iter().zip(NOMES_DO_PIVOT) {
        botao(store, id, nome);
    }
    prepara_campos_do_pivot(store);
    if store.tooltip_for(ids::INSP_RENDER_TEXTURE_SLOT).is_none() {
        store.set_tooltip(
            ids::INSP_RENDER_TEXTURE_SLOT,
            tr("card.insp.image_slot_tip"),
        );
    }
    crate::sections::ordering::prepara_opcoes(store);
    if let Some(info) = crate::state::current_inspector_sprite() {
        let valor = if sections::look::cantos_ligados(&info) {
            ph2d_editor_core::widget::CheckboxValue::Checked
        } else {
            ph2d_editor_core::widget::CheckboxValue::Unchecked
        };
        if let Some(InteractiveState::Checkbox { value, .. }) =
            store.get_mut(ids::INSP_SPRITE_CORNERS_ON)
        {
            *value = valor;
        }
    }
}

/// O cartão Image: a imagem, o seletor Use e os campos do uso.
pub(crate) fn imagem(info: InspectorSpriteInfo) -> CardDef<'static> {
    CardDef {
        section: crate::cards_objeto::CARD_IMAGE,
        title: tr("panel.inspector.image"),
        actions: Vec::new(),
        add: None,
        summary: slot_image::current_slot_image(info.entity_bits).map(|i| i.name),
        body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
            if slot_image::sem_imagem(info.entity_bits) {
                return convite(c, y);
            }
            let y = cabeca(c, y, &info);
            let ink = c.ink;
            let y = com_corpo(c, y, |t, theme, store, x, w, y| {
                let y = sections::image_use_rows::paint_image_use(
                    t.scene, t.text, theme, t.hit, store, x, w, y, &info, None,
                );
                if info.image_use != InspectorImageUse::Repeated {
                    return y;
                }
                let (rotulo, texto) = crate::cards_objeto::tamanho(&info);
                ph2d_editor_core::panel::cards::compacta::fact(
                    t.scene,
                    t.text,
                    store,
                    t.hit,
                    ink,
                    (x, w, y),
                    rotulo,
                    &texto,
                )
            });
            let y = recibo_do_replace(c, y, &info);
            aviso_de_folha(c, y, info.entity_bits)
        })),
    }
}

/// Depois de trocar por uma imagem de outro tamanho: o que mudou, e a saída. Quem repete não precisa: o bloco manda.
fn recibo_do_replace(c: &mut CardCtx<'_>, y: f32, info: &InspectorSpriteInfo) -> f32 {
    let Some(r) = slot_image::recibo().filter(|r| {
        r.entity_bits == info.entity_bits && info.image_use != InspectorImageUse::Repeated
    }) else {
        return y;
    };
    let texto = tr_with(
        "card.insp.replace_receipt",
        &[
            ("aw", &r.de[0]),
            ("ah", &r.de[1]),
            ("bw", &r.para[0]),
            ("bh", &r.para[1]),
        ],
    );
    c.row(
        &[
            Cell::new(Widget::Readout { text: &texto }),
            Cell::new(Widget::Button {
                id: ids::INSP_IMAGE_KEEP_SIZE,
                text: tr("card.insp.keep_old_size"),
                primary: false,
            }),
        ],
        y,
    )
}

/// Em Still, uma folha mostra-se inteira (regra do dono, 27/10): o aviso explica o salto de tamanho e dá as saídas.
fn aviso_de_folha(c: &mut CardCtx<'_>, y: f32, bits: u64) -> f32 {
    let Some([colunas, linhas]) = slot_image::factos()
        .filter(|f| f.entity_bits == bits)
        .and_then(|f| f.folha)
    else {
        return y;
    };
    let frase = tr_with(
        "card.insp.sheet_warning",
        &[("c", &colunas), ("l", &linhas)],
    );
    let y = com_corpo(c, y, |t, theme, _, x, w, y| {
        sections::rows::aviso(t.scene, t.text, theme, x, w, y, &frase, ColorToken::Warn)
    });
    c.row(
        &[
            Cell::new(Widget::Button {
                id: ids::INSP_IMAGE_SHEET_ANIMATED,
                text: tr("card.insp.sheet_make_animated"),
                primary: false,
            }),
            Cell::new(Widget::Button {
                id: ids::INSP_IMAGE_SHEET_REGION,
                text: tr("card.insp.sheet_use_region"),
                primary: false,
            }),
        ],
        y,
    )
}

/// O objecto sem imagem: o cartão inteiro é o alvo. Um só convite (GDevelop).
fn convite(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let caixa = Rect::new(c.x, y, c.w, m::FIELD_H * LINHAS_DO_CONVITE);
    let theme = c.ink.theme;
    ph2d_editor_core::paint::stroke_rounded_rect_dashed(
        c.scene,
        caixa,
        ph2d_tokens::Radius::Md.px(),
        ph2d_tokens::StrokeToken::Default.px(),
        m::LOOSE_DASH,
        resolve(ColorToken::Border, theme),
    );
    let linha = TypeToken::Md.px() + Spacing::Xs.px();
    let meio = caixa.y + (caixa.h - linha * 2.0) * 0.5;
    ph2d_editor_core::paint::paint_text_centered(
        c.text_system,
        c.scene,
        tr("card.insp.image_drop"),
        Rect::new(caixa.x, meio, caixa.w, linha),
        TypeToken::Md.px(),
        resolve(ColorToken::Text1, theme),
    );
    ph2d_editor_core::paint::paint_text_centered(
        c.text_system,
        c.scene,
        tr("card.insp.image_drop_hint"),
        Rect::new(caixa.x, meio + linha, caixa.w, linha),
        TypeToken::Sm.px(),
        resolve(ColorToken::Text2, theme),
    );
    c.hit_index.register(ids::INSP_RENDER_TEXTURE_SLOT, caixa);
    let mut y = caixa.y + caixa.h + m::FIELD_ROW_GAP;
    let recentes = slot_image::recentes();
    if !recentes.is_empty() {
        let celulas: Vec<Cell<'_>> = recentes
            .iter()
            .zip(ids::INSP_IMAGE_RECENT)
            .map(|((nome, _), id)| {
                Cell::new(Widget::Button {
                    id,
                    text: nome,
                    primary: false,
                })
            })
            .collect();
        y = c.compact(&celulas, y);
    }
    c.row(
        &[Cell::new(Widget::Button {
            id: ids::INSP_IMAGE_PAINT_NEW,
            text: tr("object_add.image"),
            primary: false,
        })],
        y,
    )
}

/// A miniatura grande, o nome, o tamanho e os verbos da imagem.
fn cabeca(c: &mut CardCtx<'_>, y: f32, info: &InspectorSpriteInfo) -> f32 {
    let lado = m::FIELD_H * LINHAS_DA_MINIATURA;
    let caixa = Rect::new(c.x, y, c.w, lado);
    let theme = c.ink.theme;
    let img = slot_image::current_slot_image(info.entity_bits);
    miniatura(c, Rect::new(c.x, y, lado, lado), img.as_ref());
    let tx = c.x + lado + Spacing::Sm.px();
    let tw = (c.w - lado - Spacing::Sm.px()).max(0.0);
    let nome = img.as_ref().map_or_else(
        || tr("panel.inspector.image.none").to_owned(),
        |i| {
            if i.name.is_empty() {
                tr("card.insp.image_unnamed").to_owned()
            } else {
                i.name.clone()
            }
        },
    );
    paint_text(
        c.text_system,
        c.scene,
        &nome,
        tx,
        y + Spacing::Xs.px(),
        TypeToken::Md.px(),
        tw,
        resolve(ColorToken::Text1, theme),
    );
    if let Some((pw, ph)) = info.source_pixels {
        paint_text(
            c.text_system,
            c.scene,
            &tr_with(
                "panel.inspector.render_source.size_px",
                &[("pw", &pw), ("ph", &ph)],
            ),
            tx,
            y + Spacing::Xs.px() + TypeToken::Md.px() + Spacing::Xs.px(),
            TypeToken::Sm.px(),
            tw,
            resolve(ColorToken::Text2, theme),
        );
    }
    c.hit_index.register(ids::INSP_RENDER_TEXTURE_SLOT, caixa);
    c.row(
        &[
            Cell::new(Widget::Button {
                id: ids::INSP_IMAGE_REPLACE,
                text: tr("card.insp.image_replace"),
                primary: false,
            }),
            Cell::new(Widget::Button {
                id: ids::INSP_IMAGE_EDIT_PIXEL,
                text: tr("card.insp.edit_in_pixel"),
                primary: false,
            }),
            Cell::new(Widget::Button {
                id: ids::INSP_IMAGE_EDIT_PAINTER,
                text: tr("card.insp.edit_in_painter"),
                primary: false,
            }),
        ],
        y + lado + m::FIELD_ROW_GAP,
    )
}

/// A imagem sobre o xadrez da transparência, com o aspecto dela.
fn miniatura(c: &mut CardCtx<'_>, r: Rect, img: Option<&SlotImage>) {
    // Sem raio: o xadrez encolhe pelo raio que recebe, e a imagem por cima não encolheria com ele.
    ph2d_editor_core::widget::paint_checker(c.scene, r, 0.0);
    let Some(img) = img else {
        return;
    };
    let Some(s) = slot_image::stable(img) else {
        return;
    };
    let (tw, th) = (img.w.max(1) as f32, img.h.max(1) as f32);
    let k = (r.w / tw).min(r.h / th);
    let (dw, dh) = (tw * k, th * k);
    let x0 = f64::from(r.x + (r.w - dw) * 0.5);
    let y0 = f64::from(r.y + (r.h - dh) * 0.5);
    c.scene.draw_stable_image(
        &s,
        (x0, y0, x0 + f64::from(dw), y0 + f64::from(dh)),
        ph2d_vector::ImageQuality::Medium,
    );
}

/// O resumo do Transform dobrado: `850, 200 · 0° · 1 × 1`.
pub(crate) fn resumo_do_transform(t: &InspectorTransformInfo) -> String {
    let unit = crate::state::current_display_unit();
    let ppm = crate::state::current_pixels_per_meter();
    let (px, py) = (
        unit.from_meters(t.translation[0], ppm),
        unit.from_meters(t.translation[1], ppm),
    );
    let graus = t.rotation_rad.to_degrees();
    tr_with(
        "card.insp.transform_summary",
        &[
            ("x", &format!("{px:.0}")),
            ("y", &format!("{py:.0}")),
            ("r", &format!("{graus:.0}")),
            ("sx", &curto(t.scale[0])),
            ("sy", &curto(t.scale[1])),
        ],
    )
}

/// Um fator sem zeros à direita: `1`, `1.5`.
fn curto(v: f32) -> String {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// O resumo do Advanced dobrado: a estratégia e o formato.
fn resumo_do_advanced(info: &InspectorSpriteInfo) -> String {
    let estrategia = match info.source_kind {
        InspectorSpriteSource::Atlas { .. } => tr("panel.inspector.render_source.atlas"),
        InspectorSpriteSource::Individual { .. } => tr("panel.inspector.render_source.individual"),
        InspectorSpriteSource::HandPacked { .. } => tr("panel.inspector.render_source.hand_packed"),
        InspectorSpriteSource::CookedTexture => tr("panel.inspector.render_source.cooked_texture"),
    };
    tr_with(
        "card.insp.advanced_summary",
        &[
            ("strategy", &estrategia),
            (
                "format",
                &info.source_kind.pixel_format(info.source_precision),
            ),
        ],
    )
}

/// Look e Advanced têm corpo próprio; os outros cartões são o corpo de sempre.
pub(crate) fn proprio<'a>(p: &'a Presente, actions: Vec<HeadAction>) -> Option<CardDef<'a>> {
    let info = crate::state::current_inspector_sprite()?;
    if p.id == core_ids::INSP_LIVE_COLOR_SECTION {
        return Some(CardDef {
            section: p.id,
            title: tr("card.insp.look"),
            actions,
            add: None,
            summary: None,
            body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
                let y = com_corpo(c, y, |t, theme, store, x, w, y| {
                    sections::look::paint_cores(t.scene, t.text, theme, t.hit, store, x, w, y, &info)
                });
                flip_e_pivot(c, y)
            })),
        });
    }
    if p.id == core_ids::INSP_LIVE_RENDER_SECTION {
        return Some(CardDef {
            section: p.id,
            title: tr("card.insp.advanced"),
            actions,
            add: None,
            summary: Some(resumo_do_advanced(&info)),
            body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
                com_corpo(c, y, |t, theme, store, x, w, y| {
                    sections::look::paint_advanced(
                        t.scene, t.text, theme, t.hit, store, x, w, y, &info,
                    )
                })
            })),
        });
    }
    None
}

/// Os nomes dos nove pontos, por linhas.
const NOMES_DO_PIVOT: [&str; 9] = [
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
    /// Os campos X e Y abertos à mão, com o pivô num dos nove pontos.
    static PIVOT_ABERTO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Os campos X e Y estão à vista? Sempre em Custom; nos nove pontos, só se a mão os abriu.
fn pivot_campos_abertos(f: &slot_image::SpriteFacts) -> bool {
    f.pivot_ponto.is_none() || PIVOT_ABERTO.with(std::cell::Cell::get)
}

fn prepara_campos_do_pivot(store: &mut WidgetStore) {
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
    let lado = casa * 3.0;
    let caixa = Rect::new(c.x, y, lado, lado);
    ph2d_editor_core::paint::fill_rounded_rect(
        c.scene,
        caixa,
        ph2d_tokens::Radius::Sm.px(),
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
    let nome = aceso.map_or(tr("card.insp.pivot.custom"), |i| tr(NOMES_DO_PIVOT[i]));
    let tx = c.x + lado + Spacing::Xs.px();
    let tw = (c.w - lado - Spacing::Xs.px()).max(0.0);
    paint_text(
        c.text_system,
        c.scene,
        nome,
        tx,
        y + (lado - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        tw,
        resolve(ColorToken::Text1, theme),
    );
    c.hit_index
        .register(ids::INSP_SPRITE_PIVOT_NOME, Rect::new(tx, y, tw, lado));
    y + lado + m::FIELD_ROW_GAP
}

/// Flip e Pivot lado a lado. Os ids do Flip continuam caixas de marcar: o clique emite o mesmo `Toggled`.
fn flip_e_pivot(c: &mut CardCtx<'_>, y: f32) -> f32 {
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
        &[Cell::new(Widget::Icons(&segs)).label(tr("card.insp.flip"))],
        y,
    );
    let Some(f) = crate::state::current_inspector_sprite()
        .and_then(|i| slot_image::factos().filter(|f| f.entity_bits == i.entity_bits))
    else {
        (c.x, c.w) = (x0, w0);
        return fim_flip;
    };
    (c.x, c.w) = (xb, wb);
    c.row(
        &[Cell::new(Widget::Readout { text: "" }).label(tr("card.insp.pivot"))],
        y,
    );
    // A grade começa onde a caixa do Flip começa: logo abaixo dos rótulos.
    let topo = fim_flip - m::FIELD_H;
    let fim_grade = grade_do_pivot(c, topo, f);
    (c.x, c.w) = (x0, w0);
    let fim = fim_flip.max(fim_grade);
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
fn evento_do_pivot(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let pivot = match ev {
        WidgetEvent::Click(id) if id == ids::INSP_SPRITE_PIVOT_NOME => {
            PIVOT_ABERTO.with(|a| a.set(!a.get()));
            return true;
        }
        WidgetEvent::Click(id) => {
            let Some(i) = ids::INSP_SPRITE_PIVOT.iter().position(|o| *o == id) else {
                return false;
            };
            SpritePivot::Ponto([(i % 3) as u8, (i / 3) as u8])
        }
        WidgetEvent::ValueChanged(id) if ids::INSP_SPRITE_PIVOT_XY.contains(&id) => {
            let [a, b] = ids::INSP_SPRITE_PIVOT_XY.map(|n| host.store().number_value(n));
            SpritePivot::Px([a.unwrap_or(0.0) as f32, b.unwrap_or(0.0) as f32])
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

/// O clique em Replace é o mesmo pedido do clique na miniatura; as opções do Ordering revelam a linha.
pub(crate) fn evento(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    if crate::sections::ordering::evento_das_opcoes(ev)
        || sections::look::evento_dos_cantos(host, ev)
        || evento_do_pivot(host, ev)
    {
        return true;
    }
    let WidgetEvent::Click(id) = ev else {
        return false;
    };
    if id == ids::INSP_IMAGE_EDIT_PIXEL || id == ids::INSP_IMAGE_EDIT_PAINTER {
        let Some(sprite) = crate::state::current_inspector_sprite() else {
            return true;
        };
        let pintor = id == ids::INSP_IMAGE_EDIT_PAINTER;
        host.bus_mut().push(EditorAction::ImageUseEdit {
            entity_bits: sprite.entity_bits,
            edit: if pintor {
                ImageUseEdit::EditInPainter
            } else {
                ImageUseEdit::EditInPixel
            },
        });
        if pintor {
            use ph2d_editor_core::object_mode::{ModeRequest, ObjectMode};
            host.bus_mut()
                .push(EditorAction::ObjectMode(ModeRequest::Enter(ObjectMode::Draw)));
        }
        return true;
    }
    if id == ids::INSP_IMAGE_KEEP_SIZE {
        let Some(r) = slot_image::recibo() else {
            return true;
        };
        host.bus_mut().push(EditorAction::ImageUseEdit {
            entity_bits: r.entity_bits,
            edit: ImageUseEdit::KeepSize(r.tamanho),
        });
        slot_image::set_recibo(None);
        return true;
    }
    if id == ids::INSP_IMAGE_PAINT_NEW {
        slot_image::set_pintar_no_lugar_de(
            crate::state::current_inspector_sprite().map(|s| s.entity_bits),
        );
        host.store_mut().open_new_image_dialog();
        return true;
    }
    if let Some(i) = ids::INSP_IMAGE_RECENT.iter().position(|o| *o == id) {
        let (Some(sprite), Some((_, asset))) = (
            crate::state::current_inspector_sprite(),
            slot_image::recentes().get(i).cloned(),
        ) else {
            return true;
        };
        host.bus_mut().push(EditorAction::ChooseImage {
            entity_bits: sprite.entity_bits,
            asset,
        });
        return true;
    }
    let uso = if id == ids::INSP_IMAGE_SHEET_ANIMATED {
        Some(InspectorImageUse::Animated)
    } else if id == ids::INSP_IMAGE_SHEET_REGION {
        Some(InspectorImageUse::Region)
    } else if id == ids::INSP_IMAGE_REPLACE {
        None
    } else {
        return false;
    };
    let Some(sprite) = crate::state::current_inspector_sprite() else {
        return false;
    };
    let entity_bits = sprite.entity_bits;
    host.bus_mut().push(match uso {
        // Uma Region nova abre o editor: um clique no quadro escolhe a área.
        Some(u) => EditorAction::ImageUseEdit {
            entity_bits,
            edit: ImageUseEdit::Use(u),
        },
        None => EditorAction::OpenImagePicker { entity_bits },
    });
    true
}
