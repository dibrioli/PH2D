//! ⭐ **Os cartões da Sprite**, versão B da revisão de 05/11
//! ([ficha](../../../docs/GameEngineFuncional/componentes/01_sprite.md)): Image · Look · Transform dobrado ·
//! Ordering · Advanced. Os ids são os de sempre; `PH2D_CARDS=0` pinta o painel de antes.

use crate::cards_componentes::Presente;
use crate::cards_objeto::com_corpo;
use crate::cards_sprite_cabeca::{cabeca, cabeca_do_pixel};
use crate::cards_sprite_pivot::{NOMES_DO_PIVOT, evento_do_pivot, flip_e_pivot};
use crate::ids;
use crate::sections;
use crate::slot_image;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, HeadAction, Widget};
use ph2d_editor_core::pixel_panel::PixelSpriteCard;
use ph2d_editor_core::screens::hero::{
    InspectorSpriteInfo, InspectorSpriteSource, InspectorTransformInfo,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{ImageUseEdit, InspectorImageUse};
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, Spacing, TypeToken};

/// O Look em Animated: a mesma coisa, com a dobra própria (como o Transform de uma imagem).
pub const CARD_LOOK_ANIMADA: NodeId = ph2d_tool_registry::hash_node_id("cards.insp.look.animated");

/// O convite do cartão vazio mede cinco campos: é o único alvo do cartão.
const LINHAS_DO_CONVITE: f32 = 5.0; // LITERAL-PX-OK: conta de campos, não geometria

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
    botao(
        store,
        ids::INSP_IMAGE_KEEP_SIZE,
        "card.insp.keep_old_size_tip",
    );
    botao(
        store,
        ids::INSP_IMAGE_EDIT_PAINTER,
        "card.insp.edit_in_painter_tip",
    );
    botao(
        store,
        ids::INSP_IMAGE_EDIT_PIXEL,
        "card.insp.edit_in_pixel_tip",
    );
    botao(store, ids::INSP_IMAGE_PAINT_NEW, "card.insp.paint_new_tip");
    botao(
        store,
        ids::INSP_IMAGE_SAMPLE_HERO,
        "card.insp.sample_hero_tip",
    );
    botao(store, ids::INSP_IMAGE_CUT_AGAIN, "card.insp.cut_again_tip");
    botao(store, ids::INSP_IMAGE_KEEP_CUT, "card.insp.keep_cut_tip");
    for id in ids::INSP_IMAGE_RECENT {
        botao(store, id, "card.insp.recent_tip");
    }
    // O convite e o recibo valem enquanto o objecto deles está à vista: selecionado ou preso num cartão.
    let selecionada = crate::state::current_inspector_sprite().map(|s| s.entity_bits);
    let presos = store.loose_pins_of(core_ids::INSP_PANEL);
    let a_vista = |b: u64| Some(b) == selecionada || presos.contains(&b);
    let vazio = slot_image::take_pintar_no_lugar_de().filter(|b| a_vista(*b));
    slot_image::set_pintar_no_lugar_de(vazio);
    if slot_image::recibo().is_some_and(|r| !a_vista(r.entity_bits)) {
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
    // O Look de uma folha animada nasce dobrado (ficha 02, decisão 6), com a dobra dele: a Still não muda, e
    // quem já escolheu a do Look fica com a escolha que fez.
    let escolha = store.collapsed_choice(core_ids::INSP_LIVE_COLOR_SECTION);
    store.set_collapsed_if_unchosen(CARD_LOOK_ANIMADA, escolha.unwrap_or(true));
    botao(
        store,
        ids::INSP_SPRITE_PIVOT_NOME,
        "card.insp.pivot_nome_tip",
    );
    for (id, nome) in ids::INSP_SPRITE_PIVOT.into_iter().zip(NOMES_DO_PIVOT) {
        botao(store, id, nome);
    }
    // Nos cartões a miniatura tem hover (borda e ícone de troca): o clique continua o mesmo.
    if matches!(
        store.get(ids::INSP_RENDER_TEXTURE_SLOT),
        Some(InteractiveState::Plain)
    ) {
        store.register(
            ids::INSP_RENDER_TEXTURE_SLOT,
            InteractiveState::Button {
                state: ph2d_editor_core::widget::ButtonState::Normal,
            },
        );
    }
    if store.tooltip_for(ids::INSP_RENDER_TEXTURE_SLOT).is_none() {
        store.set_tooltip(
            ids::INSP_RENDER_TEXTURE_SLOT,
            tr("card.insp.image_slot_tip"),
        );
    }
    crate::sections::ordering::prepara_opcoes(store);
    crate::cards_anim::prepara();
    semeia_do_objeto(store);
}

/// As caixas da Sprite que leem o objecto publicado: a seleção aqui, e o preso no ensaio de `cards_presos`.
pub(crate) fn semeia_do_objeto(store: &mut WidgetStore) {
    crate::cards_sprite_pivot::prepara_campos_do_pivot(store);
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
                return convite(c, y, info.image_use == InspectorImageUse::Animated);
            }
            let y = cabeca(c, y, &info);
            let y = com_corpo(c, y, |t, theme, store, x, w, y| {
                sections::image_use_rows::paint_image_use(
                    t.scene, t.text, theme, t.hit, store, x, w, y, &info, None,
                )
            });
            let y = recibo_do_replace(c, y, &info);
            let y = aviso_de_corte(c, y, info.entity_bits);
            aviso_de_folha(c, y, info.entity_bits)
        })),
    }
}

/// O cartão Image de um Pixel Sprite: a mesma moldura; não há imagem a trocar nem Use nem Look.
pub(crate) fn imagem_do_pixel(card: PixelSpriteCard) -> CardDef<'static> {
    let nome = crate::state::current_inspector_name().map_or_else(String::new, |n| n.name);
    CardDef {
        section: crate::cards_objeto::CARD_IMAGE,
        title: tr("panel.inspector.image"),
        actions: Vec::new(),
        add: None,
        summary: (!nome.is_empty()).then(|| nome.clone()),
        body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
            cabeca_do_pixel(c, y, &card, &nome)
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
    let y = com_corpo(c, y, |t, theme, _, x, w, y| {
        sections::rows::aviso(t.scene, t.text, theme, x, w, y, &texto, ColorToken::Text2)
    });
    c.row(
        &[Cell::new(Widget::Button {
            id: ids::INSP_IMAGE_KEEP_SIZE,
            text: tr("card.insp.keep_old_size"),
            primary: false,
        })],
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

/// A imagem trocada deixou o corte velho (ficha 02): o problema, com os números de verdade, e as duas saídas.
fn aviso_de_corte(c: &mut CardCtx<'_>, y: f32, bits: u64) -> f32 {
    let Some(a) = slot_image::factos()
        .filter(|f| f.entity_bits == bits)
        .and_then(|f| f.corte_velho)
    else {
        return y;
    };
    let frase = tr_with(
        "card.insp.new_image_old_cut",
        &[
            ("c", &a.grid[0]),
            ("r", &a.grid[1]),
            ("w", &a.cell_px[0]),
            ("h", &a.cell_px[1]),
            ("n", &a.animations),
        ],
    );
    let y = com_corpo(c, y, |t, theme, _, x, w, y| {
        sections::rows::aviso(t.scene, t.text, theme, x, w, y, &frase, ColorToken::Warn)
    });
    c.row(
        &[
            Cell::new(Widget::Button {
                id: ids::INSP_IMAGE_CUT_AGAIN,
                text: tr("card.insp.cut_again"),
                primary: false,
            }),
            Cell::new(Widget::Button {
                id: ids::INSP_IMAGE_KEEP_CUT,
                text: tr("card.insp.keep_cut"),
                primary: false,
            }),
        ],
        y,
    )
}

/// O objecto sem imagem: o cartão inteiro é o alvo. Um só convite (GDevelop). O vazio do Animated pede uma folha
/// e, no lugar de pintar uma imagem, oferece o herói de fábrica.
fn convite(c: &mut CardCtx<'_>, y: f32, animada: bool) -> f32 {
    let caixa = Rect::new(c.x, y, c.w, m::FIELD_H * LINHAS_DO_CONVITE);
    let theme = c.ink.theme;
    ph2d_editor_core::paint::stroke_rounded_rect_dashed(
        c.scene,
        caixa,
        ph2d_editor_core::paint::frame_radius(theme, ph2d_tokens::Radius::Md.px()),
        ph2d_tokens::StrokeToken::Default.px(),
        m::LOOSE_DASH,
        resolve(ColorToken::Border, theme),
    );
    let linha = TypeToken::Md.px() + Spacing::Xs.px();
    let meio = caixa.y + (caixa.h - linha * 2.0) * 0.5;
    ph2d_editor_core::paint::paint_text_centered(
        c.text_system,
        c.scene,
        tr(if animada {
            "card.insp.sheet_drop"
        } else {
            "card.insp.image_drop"
        }),
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
    let (id, text) = if animada {
        (ids::INSP_IMAGE_SAMPLE_HERO, tr("card.insp.sample_hero"))
    } else {
        (ids::INSP_IMAGE_PAINT_NEW, tr("object_add.image"))
    };
    c.row(
        &[Cell::new(Widget::Button {
            id,
            text,
            primary: false,
        })],
        y,
    )
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

/// O resumo do Look dobrado: `Opacity 100%`.
fn resumo_do_look(info: &InspectorSpriteInfo) -> String {
    let pct = format!("{:.0}", info.opacity * 100.0); // LITERAL-PX-OK: fração → %
    tr_with("card.insp.look_summary", &[("opacity", &pct)])
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
        let animada = info.image_use == InspectorImageUse::Animated;
        return Some(CardDef {
            section: if animada { CARD_LOOK_ANIMADA } else { p.id },
            title: tr("card.insp.look"),
            actions,
            add: None,
            summary: animada.then(|| resumo_do_look(&info)),
            body: Some(Box::new(move |c: &mut CardCtx<'_>, y| {
                let y = com_corpo(c, y, |t, theme, store, x, w, y| {
                    sections::look::paint_cores(
                        t.scene, t.text, theme, t.hit, store, x, w, y, &info,
                    )
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

/// Os verbos da imagem; as opções do Ordering revelam a linha. Clicar na miniatura é `event::texture_slot_click`.
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
    if id == ids::INSP_IMAGE_EDIT_PAINTER {
        let Some(sprite) = crate::state::current_inspector_sprite() else {
            return true;
        };
        host.bus_mut().push(EditorAction::ImageUseEdit {
            entity_bits: sprite.entity_bits,
            edit: ImageUseEdit::EditInPainter,
        });
        use ph2d_editor_core::object_mode::{ModeRequest, ObjectMode};
        host.bus_mut()
            .push(EditorAction::ObjectMode(ModeRequest::Enter(
                ObjectMode::Draw,
            )));
        return true;
    }
    if id == ids::INSP_IMAGE_EDIT_PIXEL {
        use ph2d_editor_core::object_mode::{ModeRequest, ObjectMode};
        let mode = ObjectMode::Draw;
        // O Enter é o do objecto activo; o do cartão preso selecciona o dono primeiro.
        let pedido = match host.store().card_event_subject(ev) {
            Some(entity) => ModeRequest::OpenContent {
                entity,
                mode,
                key: 0,
            },
            None => ModeRequest::Enter(mode),
        };
        host.bus_mut().push(EditorAction::ObjectMode(pedido));
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
    let verbo = if id == ids::INSP_IMAGE_SAMPLE_HERO {
        Some(ImageUseEdit::SampleHero)
    } else if id == ids::INSP_IMAGE_CUT_AGAIN {
        Some(ImageUseEdit::CutAgain)
    } else if id == ids::INSP_IMAGE_KEEP_CUT {
        Some(ImageUseEdit::KeepCut)
    } else {
        None
    };
    if let Some(edit) = verbo {
        if let Some(sprite) = crate::state::current_inspector_sprite() {
            host.bus_mut().push(EditorAction::ImageUseEdit {
                entity_bits: sprite.entity_bits,
                edit,
            });
        }
        return true;
    }
    let uso = if id == ids::INSP_IMAGE_SHEET_ANIMATED {
        InspectorImageUse::Animated
    } else if id == ids::INSP_IMAGE_SHEET_REGION {
        InspectorImageUse::Region
    } else {
        return false;
    };
    let Some(sprite) = crate::state::current_inspector_sprite() else {
        return false;
    };
    // Uma Region nova abre o editor: um clique no quadro escolhe a área.
    host.bus_mut().push(EditorAction::ImageUseEdit {
        entity_bits: sprite.entity_bits,
        edit: ImageUseEdit::Use(uso),
    });
    true
}
