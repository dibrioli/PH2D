//! ⭐⭐ **O painel Vector em CARTÕES** (`docs/CartoesUI/12_VECTOR_PLANO.md`): a lista inteira, com as
//! abas no título. Mesmos ids de secção e de controlo; `populate` e `event` não mudam.
//!
//! Toda secção tem corpo nativo ([`nativo`]); os pintores antigos só correm com `PH2D_CARDS=0`. Quais
//! estão à vista e com que título responde a tabela ([`crate::cards_presenca`]).

use crate::cards_presenca::Presente;
use crate::paint_sections::BodyCtx;
use crate::{ids, state};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::TextKey;
use ph2d_editor_core::panel::cards::{CardBody, CardCtx, CardDef, Cell, HeadAction, Seg, Widget};
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_editor_core::widget::{number_input_min_w_px, section_cards};
use ph2d_i18n::tr;
use ph2d_tokens::{ROW_H_PX, Spacing, TypeToken};
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::frames::DEVICE_PRESETS;
use ph2d_tool_vector::ids as vids;

thread_local! {
    static FORCADO: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// O painel pinta-se em cartões? `PH2D_CARDS=0` = o painel de antes.
#[must_use]
pub fn cards_on() -> bool {
    FORCADO
        .with(std::cell::Cell::get)
        .unwrap_or_else(|| std::env::var("PH2D_CARDS").as_deref() != Ok("0"))
}

/// Força o modo NESTA thread: os testes de um processo correm em paralelo. `None` volta ao ambiente.
#[doc(hidden)]
pub fn force_cards_on_this_thread(on: Option<bool>) {
    FORCADO.with(|f| f.set(on));
}

fn comprimento_em_px() -> bool {
    state::length_suffix() == "px"
}

/// Passo dos números de comprimento: 1 px, ou 1 cm no projecto em metros.
fn passo_do_comprimento() -> f64 {
    if comprimento_em_px() { 1.0 } else { 0.01 } // LITERAL-PX-OK: passo em metros
}

/// Com o store MUTÁVEL, antes de pintar. As sementes dos campos são as de sempre (`paint.rs`).
pub(crate) fn prepare(store: &mut WidgetStore) {
    store.mark_card_list_close(core_ids::VECTOR_PANEL, core_ids::VECTOR_CLOSE);
    crate::cards_tinta::prepare(store);
    crate::cards_texto::prepare(store);
    crate::cards_ponto::prepare(store);
    crate::cards_padrao::prepare(store);
    crate::cards_avancado::prepare(store);
    crate::cards_interface::prepare(store);
    crate::cards_estados::prepare(store);
    let p = passo_do_comprimento();
    for id in [vids::VECTOR_TRANSFORM_W, vids::VECTOR_TRANSFORM_H] {
        let spec = CardFieldSpec {
            step: p,
            scrub: p,
            default: None,
        };
        store.mark_card_field(id, spec);
    }
}

/// ⭐ Os cartões do painel, pela ordem natural.
pub(crate) fn cards<'a>(
    snap: &'a VectorStyleSnapshot,
    presentes: &'a [Presente],
) -> Vec<CardDef<'a>> {
    presentes
        .iter()
        .filter_map(|p| {
            let id = p.id;
            let body: CardBody<'a> = match nativo(id) {
                Some(f) => Box::new(f),
                None if id == ids::VECTOR_SECTION_ADVANCED => {
                    Box::new(move |c: &mut CardCtx<'_>, y| avancado(c, snap, y))
                }
                None if id == vids::VECTOR_SECTION_FILL => {
                    Box::new(move |c: &mut CardCtx<'_>, y| fill_stroke(c, snap, y))
                }
                None => return None,
            };
            Some(CardDef {
                section: id,
                title: p.titulo,
                // O «+» das listas do Figma (F43/F44) mora no cabeçalho do cartão, com o MESMO id.
                actions: p
                    .acao
                    .filter(|a| a.3)
                    .map(|(id, icon, on, _)| HeadAction { id, icon, on })
                    .into_iter()
                    .collect(),
                add: None,
                summary: None,
                body: Some(body),
            })
        })
        .collect()
}

type CorpoNativo = fn(&mut CardCtx<'_>, f32) -> f32;

fn nativo(id: NodeId) -> Option<CorpoNativo> {
    let f: CorpoNativo = if id == vids::VECTOR_SECTION_TRANSFORM {
        shape
    } else if id == vids::VECTOR_SECTION_SNAP {
        snap_card
    } else if id == ids::VECTOR_SECTION_ANCHORS {
        anchors
    } else if id == vids::VECTOR_SECTION_FRAME {
        frame
    } else if id == vids::VECTOR_SECTION_CLIP {
        clip
    } else if id == ids::VECTOR_SECTION_APPEARANCE {
        crate::cards_tinta::appearance
    } else if id == vids::VECTOR_SECTION_EFFECTS {
        crate::cards_fx::effects
    } else if id == ids::VECTOR_SECTION_FILTERS {
        crate::cards_fx::filters
    } else if id == vids::VECTOR_SECTION_TEXT {
        crate::cards_texto::text
    } else if id == vids::VECTOR_SECTION_FONT {
        crate::cards_texto::font
    } else if id == vids::VECTOR_SECTION_PARAGRAPH {
        crate::cards_texto::paragraph
    } else if id == vids::VECTOR_SECTION_AXES {
        crate::cards_texto::axes
    } else if id == ids::VECTOR_SECTION_TEXTPATH {
        crate::cards_texto::textpath
    } else if id == vids::VECTOR_SECTION_VERTEX {
        crate::cards_ponto::vertex
    } else if id == vids::VECTOR_SECTION_CONNECTOR {
        crate::cards_ponto::connector_card
    } else if id == ids::VECTOR_SECTION_BRUSH {
        crate::cards_ponto::brush
    } else if id == ids::VECTOR_SECTION_TEXPAT {
        crate::cards_padrao::pattern_fill
    } else if id == ids::VECTOR_SECTION_TEXPAT_STROKE {
        crate::cards_padrao::pattern_stroke
    } else if id == ids::VECTOR_SECTION_COMPONENT {
        crate::cards_interface::component
    } else if id == ids::VECTOR_SECTION_LAYOUT {
        crate::cards_interface::layout
    } else if id == ids::VECTOR_SECTION_WIDGET {
        crate::cards_interface::widget
    } else if id == ids::VECTOR_SECTION_STATES {
        crate::cards_estados::states
    } else if id == ids::VECTOR_SECTION_MORPH_STATES {
        crate::cards_estados::morph_states
    } else {
        return None;
    };
    Some(f)
}

/// ⭐ **As linhas de Fill & Stroke** (F44/F45): as MESMAS linhas, o mesmo arrasto pelo ⚙ e a mesma
/// conta do vão; os primitivos da linha pintam-se com a cara do cartão sob a tinta dele.
fn fill_stroke(c: &mut CardCtx<'_>, snap: &VectorStyleSnapshot, y: f32) -> f32 {
    let mut b = linhas_do_cartao(c);
    let fim = b.fill_stroke_rows(snap, y);
    // A última linha não leva o vão de baixo: o cartão tem a folga dele.
    if fim > y { fim - m::FIELD_ROW_GAP } else { y }
}

/// As linhas que se pintam pelo `BodyCtx` e perguntam `card_body()`, no passo e na largura do cartão.
pub(crate) fn linhas_do_cartao<'a>(c: &'a mut CardCtx<'_>) -> BodyCtx<'a> {
    BodyCtx {
        scene: &mut *c.scene,
        text_system: &mut *c.text_system,
        store: c.store,
        hit_index: &mut *c.hit_index,
        theme: c.ink.theme,
        inner_x: c.x,
        inner_w: c.w,
        row_h: m::FIELD_H,
        row_gap: m::FIELD_ROW_GAP,
        chip_w: number_input_min_w_px(),
        font: m::VALUE_PX,
        open_fold: None,
    }
}

/// O grupo Advanced é UM cartão: os membros lá dentro, com a cara de grupo, a dobra e as notas pelas
/// portas partilhadas, e o corpo nativo. Um arrasto não tira um membro do grupo (F17).
fn avancado(c: &mut CardCtx<'_>, snap: &VectorStyleSnapshot, y: f32) -> f32 {
    let membros = crate::cards_avancado::membros(snap, c.ink);
    let (theme, x, w, store) = (c.ink.theme, c.x, c.w, c.store);
    let (scene, text_system, hit_index) = (&mut *c.scene, &mut *c.text_system, &mut *c.hit_index);
    section_cards::with_section_cards(scene, theme, y, |scene| {
        BodyCtx {
            scene,
            text_system,
            store,
            hit_index,
            theme,
            inner_x: x,
            inner_w: w,
            row_h: ROW_H_PX,
            row_gap: Spacing::Xs.px(),
            chip_w: number_input_min_w_px(),
            font: TypeToken::Base.px(),
            open_fold: None,
        }
        .corre(membros, y)
    })
}

fn campo<'a>(id: NodeId, letra: &'a str) -> Widget<'a> {
    Widget::Field {
        id,
        glyph: Glyph::Text(letra),
    }
}

/// ⭐ O cartão **Shape**: o que só a forma tem. A pose é o Transform do Inspector, primeiro cartão da coluna
/// (`WidgetStore::invite_card_guest`); `VECTOR_TRANSFORM_X/Y/R` só se pintam com `PH2D_CARDS=0`.
fn shape(c: &mut CardCtx<'_>, y: f32) -> f32 {
    const TAM: &[NodeId] = &[vids::VECTOR_TRANSFORM_W, vids::VECTOR_TRANSFORM_H];
    let tamanho = if comprimento_em_px() {
        tr("card.vector.size_px")
    } else {
        tr("card.vector.size_m")
    };
    let y = c.row(
        &[
            Cell::new(campo(TAM[0], tr("card.vector.glyph.w"))).label(tamanho),
            Cell::new(campo(TAM[1], tr("card.vector.glyph.h"))).key(TAM),
        ],
        y,
    );
    let y = match state::resize_box() {
        Some(on) => c.row(
            &[crate::cards_tinta::caixa(
                crate::ids::VECTOR_TRANSFORM_RESIZE_BOX,
                Rotulo::vista(TextKey::new("panel.vector.transform.resize_box")),
                on,
            )],
            y,
        ),
        None => y,
    };
    let centro = if state::pivot_edit_armed() {
        tr("panel.vector.transform.click_canvas_to_set_center")
    } else {
        tr("panel.vector.transform.set_center")
    };
    c.full(
        None,
        Widget::Button {
            id: ids::VECTOR_PIVOT_EDIT,
            text: centro,
            primary: false,
        },
        y,
    )
}

fn off_on<'a>(off: NodeId, on: NodeId, ligado: bool) -> [Seg<'a>; 2] {
    let seg = |id, chave, aceso| Seg {
        id,
        icon: None,
        text: tr(chave),
        on: aceso,
    };
    [
        seg(off, "panel.vector.arrange.off", !ligado),
        seg(on, "panel.vector.arrange.on", ligado),
    ]
}

/// Os quatro ímãs são independentes, e nenhum é canal: ajuste da ferramenta (D17).
fn snap_card(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let formas = off_on(
        ids::VECTOR_SNAP_OFF,
        core_ids::VECTOR_SNAP_ON,
        state::current_snap(),
    );
    let linha = off_on(
        ids::VECTOR_SNAP_PATH_OFF,
        ids::VECTOR_SNAP_PATH_ON,
        state::current_snap_path(),
    );
    let cruzes = off_on(
        ids::VECTOR_SNAP_CROSS_OFF,
        ids::VECTOR_SNAP_CROSS_ON,
        state::current_snap_crossings(),
    );
    let guias = off_on(
        ids::VECTOR_SNAP_GUIDES_OFF,
        ids::VECTOR_SNAP_GUIDES_ON,
        state::current_snap_guides(),
    );
    let y = c.row(
        &[
            Cell::new(Widget::Choice(&formas)).label(tr("panel.vector.arrange.shapes")),
            Cell::new(Widget::Choice(&linha)).label(tr("panel.vector.arrange.path")),
        ],
        y,
    );
    c.row(
        &[
            Cell::new(Widget::Choice(&cruzes)).label(tr("panel.vector.arrange.cross")),
            Cell::new(Widget::Choice(&guias)).label(tr("panel.vector.arrange.guides")),
        ],
        y,
    )
}

fn anchors(c: &mut CardCtx<'_>, y: f32) -> f32 {
    const H: [NodeId; 4] = [
        ids::VECTOR_ANCHOR_H_START,
        ids::VECTOR_ANCHOR_H_CENTER,
        ids::VECTOR_ANCHOR_H_END,
        ids::VECTOR_ANCHOR_H_STRETCH,
    ];
    const V: [NodeId; 4] = [
        ids::VECTOR_ANCHOR_V_START,
        ids::VECTOR_ANCHOR_V_CENTER,
        ids::VECTOR_ANCHOR_V_END,
        ids::VECTOR_ANCHOR_V_STRETCH,
    ];
    let Some(a) = state::anchor_state() else {
        return y;
    };
    let opcoes = |grupo: [NodeId; 4], chaves: [&'static str; 4], aceso: Option<NodeId>| {
        [0, 1, 2, 3].map(|i| Seg {
            id: grupo[i],
            icon: None,
            text: tr(chaves[i]),
            on: aceso == Some(grupo[i]),
        })
    };
    let h = opcoes(
        H,
        [
            "panel.vector.anchors.left",
            "panel.vector.anchors.center",
            "panel.vector.anchors.right",
            "panel.vector.anchors.stretch",
        ],
        a.h,
    );
    let v = opcoes(
        V,
        [
            "panel.vector.anchors.top",
            "panel.vector.anchors.center",
            "panel.vector.anchors.bottom",
            "panel.vector.anchors.stretch",
        ],
        a.v,
    );
    let y = c.row(
        &[Cell::new(Widget::Choice(&h))
            .label(tr("panel.vector.anchors.h"))
            .key(&H)],
        y,
    );
    c.row(
        &[Cell::new(Widget::Choice(&v))
            .label(tr("panel.vector.anchors.v"))
            .key(&V)],
        y,
    )
}

fn frame(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let aberto = state::frame_panel_open();
    let painel = [
        Seg {
            id: ids::VECTOR_FRAME_PANEL_OFF,
            icon: None,
            text: tr("panel.vector.frame.panel.off"),
            on: !aberto,
        },
        Seg {
            id: ids::VECTOR_FRAME_PANEL_ON,
            icon: None,
            text: tr("panel.vector.frame.panel.on"),
            on: aberto,
        },
    ];
    let y = c.full(
        Some(tr("panel.vector.frame.panel")),
        Widget::Choice(&painel),
        y,
    );
    // Da TABELA: um aparelho novo ganha o botão de graça. Nenhum fica aceso: é um gesto, não um modo.
    DEVICE_PRESETS.chunks(2).fold(y, |y, par| {
        let cells: Vec<Cell<'_>> = par
            .iter()
            .map(|p| {
                Cell::new(Widget::Button {
                    id: p.id,
                    text: crate::nomes_do_motor::moldura(p.label),
                    primary: false,
                })
            })
            .collect();
        c.row(&cells, y)
    })
}

fn clip(c: &mut CardCtx<'_>, y: f32) -> f32 {
    const CANAL: [NodeId; 2] = [ids::VECTOR_FRAME_CLIP_OFF, ids::VECTOR_FRAME_CLIP_ON];
    let ligado = state::frame_clip().unwrap_or(false);
    let segs = [
        Seg {
            id: CANAL[0],
            icon: None,
            text: tr("panel.vector.frame.clip.off"),
            on: !ligado,
        },
        Seg {
            id: CANAL[1],
            icon: None,
            text: tr("panel.vector.frame.clip.on"),
            on: ligado,
        },
    ];
    c.row(
        &[Cell::new(Widget::Choice(&segs))
            .label(tr("panel.vector.frame.clip"))
            .key(&CANAL)],
        y,
    )
}
