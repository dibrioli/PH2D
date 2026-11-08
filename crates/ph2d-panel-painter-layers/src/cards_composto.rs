//! O Composite Brush em linhas de cartão (fatia H8, `docs/CartoesUI/13_PAINTER_PLANO.md`). Cada camada: nome com as
//! setas, Strength + Size, Hardness + cor (a borracha troca a cor pelo escopo). Mesmos ids, mesmos eventos; a quota
//! e a contagem continuam a ser do motor.

use crate::cards_linhas::{Item, aberta, arma_barra, fila, filete, registra_lista};
use crate::paint_composite::{apaga, deposita_cor, escopo_name, n_camadas, op_name};
use crate::{composite_picker, paint_brush, state};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::{paint_icon, paint_text};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget, censo};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_tool_painter::BrushSettings;
use ph2d_tool_painter::ids as pids;

fn enc(c: f32) -> u8 {
    (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8 // LITERAL-PX-OK: sRGB 8-bit normalize
}

fn presente(b: &BrushSettings) -> bool {
    b.composite_enabled
}

/// A fracção da barra do tamanho: o chip mostra px de diâmetro, e a projecção é a de `populate_composite_chips`.
fn tamanho(b: &BrushSettings, pos: usize) -> f32 {
    crate::populate_composite_chips::pista_do_px(b, pos)
}

/// Com o store e o barramento, antes de pintar.
pub(crate) fn prepare(ctx: &mut PaintCtx, b: &BrushSettings) {
    registra_lista(
        ctx.host.store_mut(),
        pids::PAINTER_BRUSH_COMPOSITE_ADD_KIND,
        usize::from(b.composite_add_op),
    );
    crate::paint::register_button(ctx.host.store_mut(), pids::PAINTER_BRUSH_COMPOSITE_ADD);
    if !presente(b) {
        return;
    }
    let f = paint_brush::FALLBACK_BRUSH;
    for pos in 0..n_camadas(b) {
        arma_camada(ctx.host.store_mut(), b, &f, pos);
        if deposita_cor(b.composite_ops[pos]) {
            let id = pids::PAINTER_BRUSH_COMPOSITE_COLOR[pos];
            crate::paint::register_button(ctx.host.store_mut(), id);
            composite_picker::readback(ctx, id, b.composite_color[pos]);
        }
    }
}

fn arma_camada(store: &mut WidgetStore, b: &BrushSettings, f: &BrushSettings, pos: usize) {
    arma_barra(
        store,
        [
            pids::PAINTER_BRUSH_COMPOSITE_STRENGTH[pos],
            pids::PAINTER_BRUSH_COMPOSITE_STRENGTH_CHIP[pos],
        ],
        b.composite_strength[pos].clamp(0.0, 1.0),
        f.composite_strength[pos].clamp(0.0, 1.0),
        crate::cards_topo::CASAS_DO_PINCEL,
    );
    crate::populate_composite_chips::liga_tamanho(store, b, pos);
    arma_barra(
        store,
        [
            pids::PAINTER_BRUSH_COMPOSITE_SIZE[pos],
            pids::PAINTER_BRUSH_COMPOSITE_SIZE_CHIP[pos],
        ],
        tamanho(b, pos),
        tamanho(f, pos),
        crate::populate_composite_chips::CASAS_DO_PX,
    );
    arma_barra(
        store,
        [
            pids::PAINTER_BRUSH_COMPOSITE_HARDNESS[pos],
            pids::PAINTER_BRUSH_COMPOSITE_HARDNESS_CHIP[pos],
        ],
        b.composite_hardness[pos].clamp(0.0, 1.0),
        f.composite_hardness[pos].clamp(0.0, 1.0),
        crate::cards_topo::CASAS_DO_PINCEL,
    );
    for id in [
        pids::PAINTER_BRUSH_COMPOSITE_UP[pos],
        pids::PAINTER_BRUSH_COMPOSITE_DOWN[pos],
        pids::PAINTER_BRUSH_COMPOSITE_REMOVE[pos],
        pids::PAINTER_BRUSH_COMPOSITE_COLOR_CLEAR[pos],
        pids::PAINTER_BRUSH_COMPOSITE_HARDNESS_CLEAR[pos],
        pids::PAINTER_BRUSH_COMPOSITE_ERASE_SCOPE[pos],
    ] {
        crate::paint::register_button(store, id);
    }
}

/// Onde a caixa aberta ficou é onde o menu do `+` abre.
pub(crate) fn listas_abertas(ctx: &mut PaintCtx, b: &BrushSettings) {
    let id = pids::PAINTER_BRUSH_COMPOSITE_ADD_KIND;
    if presente(b)
        && aberta(ctx.host.store(), id)
        && let Some(r) = ctx.host.hit_index_mut().rect_for(id)
    {
        state::set_pending_composite_add_menu(Some(r));
    }
}

/// Uma seta ou o `x`: apagada e sem alvo nas pontas da lista, como as linhas das camadas.
fn icone(c: &mut CardCtx<'_>, id: NodeId, r: Rect, vivo: bool, icon: IconId) {
    if vivo {
        c.bare(
            r,
            &[Seg {
                id,
                icon: Some(icon),
                text: "",
                on: false,
            }],
        );
        return;
    }
    let g = m::ICON_GLYPH;
    paint_icon(
        c.scene,
        icon,
        Rect::new(r.x + (r.w - g) * 0.5, r.y + (r.h - g) * 0.5, g, g),
        c.ink.color(ColorToken::TextDisabled),
        StrokeToken::Default.px(),
    );
}

/// O nome da camada, à esquerda, e as setas na ponta: nada do que é uma linha de propriedade.
fn cabeca(c: &mut CardCtx<'_>, b: &BrushSettings, pos: usize, y: f32) -> f32 {
    let topo = filete(c, y) + m::FIELD_ROW_GAP;
    let lado = m::FIELD_H;
    let n = n_camadas(b);
    let setas = [
        (
            pids::PAINTER_BRUSH_COMPOSITE_UP[pos],
            pos > 0,
            IconId::ChevronUp,
        ),
        (
            pids::PAINTER_BRUSH_COMPOSITE_DOWN[pos],
            pos + 1 < n,
            IconId::ChevronDown,
        ),
        (
            pids::PAINTER_BRUSH_COMPOSITE_REMOVE[pos],
            true,
            IconId::Close,
        ),
    ];
    let inicio = c.x + c.w - lado * setas.len() as f32;
    for (i, (id, vivo, icon)) in setas.into_iter().enumerate() {
        let r = Rect::new(inicio + lado * i as f32, topo, lado, lado);
        icone(c, id, r, vivo, icon);
    }
    let nome = format!("{} {}", pos + 1, op_name(b.composite_ops[pos]));
    let cor = c.ink.color(ColorToken::CardLabel);
    paint_text(
        c.text_system,
        c.scene,
        &nome,
        c.x,
        topo + ph2d_editor_core::paint::folga_do_texto(lado, m::LABEL_PX),
        m::LABEL_PX,
        (inicio - c.x).max(0.0),
        cor,
    );
    censo::frase((c.x, c.w), topo, topo + lado, &nome);
    topo + lado
}

/// Hardness com o botão que a devolve ao pincel; ao lado, a cor (ou o que a borracha apaga).
fn dureza_e_cor(c: &mut CardCtx<'_>, b: &BrushSettings, pos: usize, y: f32) -> f32 {
    let dureza = Item::Barra {
        chave: "panel.painter_layers.composite.hardness",
        ids: [
            pids::PAINTER_BRUSH_COMPOSITE_HARDNESS[pos],
            pids::PAINTER_BRUSH_COMPOSITE_HARDNESS_CHIP[pos],
        ],
    };
    let limpa = |id, texto| {
        [Seg {
            id,
            icon: Some(IconId::Close),
            text: texto,
            on: false,
        }]
    };
    let limpa_dureza = limpa(pids::PAINTER_BRUSH_COMPOSITE_HARDNESS_CLEAR[pos], "");
    let limpa_cor = limpa(pids::PAINTER_BRUSH_COMPOSITE_COLOR_CLEAR[pos], "");
    let cor = b.composite_color[pos];
    let rgba = [enc(cor[0]), enc(cor[1]), enc(cor[2]), u8::MAX];
    let codigo = format!("{:02X}{:02X}{:02X}", rgba[0], rgba[1], rgba[2]);
    let mut cells: Vec<Cell<'_>> = vec![dureza.cell()];
    if b.composite_hardness_authored[pos] {
        cells.push(Cell::new(Widget::Bare(&limpa_dureza)));
    }
    if deposita_cor(b.composite_ops[pos]) {
        cells.push(
            Cell::new(Widget::Swatch {
                id: pids::PAINTER_BRUSH_COMPOSITE_COLOR[pos],
                rgba: Some(rgba),
                text: &codigo,
                edit: None,
                detach: None,
            })
            .label(tr("panel.painter_layers.brush.color")),
        );
        if b.composite_color_authored[pos] {
            cells.push(Cell::new(Widget::Bare(&limpa_cor)));
        }
    }
    if apaga(b.composite_ops[pos]) {
        cells.push(
            Cell::new(Widget::Button {
                id: pids::PAINTER_BRUSH_COMPOSITE_ERASE_SCOPE[pos],
                text: escopo_name(b.composite_erase_scope[pos]),
                primary: false,
            })
            .label(tr("card.painter.composite.scope")),
        );
    }
    c.row(&cells, y)
}

fn camada(c: &mut CardCtx<'_>, b: &BrushSettings, pos: usize, y: f32) -> f32 {
    let y = cabeca(c, b, pos, y);
    let forca = [
        Item::Barra {
            chave: "panel.painter_layers.composite.strength",
            ids: [
                pids::PAINTER_BRUSH_COMPOSITE_STRENGTH[pos],
                pids::PAINTER_BRUSH_COMPOSITE_STRENGTH_CHIP[pos],
            ],
        },
        Item::Barra {
            chave: "panel.painter_layers.composite.size",
            ids: [
                pids::PAINTER_BRUSH_COMPOSITE_SIZE[pos],
                pids::PAINTER_BRUSH_COMPOSITE_SIZE_CHIP[pos],
            ],
        },
    ];
    let y = fila(c, &forca, y);
    dureza_e_cor(c, b, pos, y)
}

/// O `+` e o menu das operações com quota; com a quota gasta, os dois apagados e sem alvo.
fn adiciona(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let mais = [Seg {
        id: pids::PAINTER_BRUSH_COMPOSITE_ADD,
        icon: Some(IconId::Add),
        text: "",
        on: false,
    }];
    let cells = [
        Cell::new(Widget::Bare(&mais)),
        Cell::new(Widget::Dropdown {
            id: pids::PAINTER_BRUSH_COMPOSITE_ADD_KIND,
            text: op_name(b.composite_add_op),
            options: &[],
        }),
    ];
    if b.composite_add_available.iter().any(|&a| a) {
        return c.row(&cells, y);
    }
    let mut fora = HitIndex::new();
    std::mem::swap(c.hit_index, &mut fora);
    let fim = crate::cards_linhas::esmaecida(c, y, |c, y| c.row(&cells, y));
    std::mem::swap(c.hit_index, &mut fora);
    fim
}

/// As camadas e o `+`, que existe mesmo com a pilha vazia: a caixa de marcar é irmã da linha do traço.
pub(crate) fn corpo(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    if !presente(b) {
        return y;
    }
    let mut y = y;
    for pos in 0..n_camadas(b) {
        y = camada(c, b, pos, y);
    }
    adiciona(c, b, y)
}
