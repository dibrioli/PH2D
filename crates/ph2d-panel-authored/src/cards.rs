//! ⭐ **O painel desenhado pelo artista, em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`):
//! converte-se só o cromo. Cada `SectionHeader` autorado é um cartão (o id da row é o da secção: a dobra é
//! a mesma), e as rows continuam pintadas pela MESMA pele que a prévia do canvas percorre: a cara delas é
//! o que o artista desenhou, não é nossa. Nenhuma palavra do programa entra: as rows antes do 1.º cabeçalho
//! vão num cartão com o nome da própria moldura.

use crate::paint::{OpenList, paint_open_list};
use crate::rows::{self, Row};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::linhas_extra::{FloatHandles, paint_floating_list};
use ph2d_editor_core::panel::cards::{CardCtx, CardDef};
use ph2d_editor_core::widget::{SkinParam, icon_glyph, inline_option_rect, paint_widget_skin_with};
use ph2d_editor_core::zones::Rect;
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    static FORCADO: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// O painel pinta-se em cartões? `PH2D_CARDS=0` = o corpo de antes.
#[must_use]
pub fn cards_on() -> bool {
    FORCADO
        .with(std::cell::Cell::get)
        .unwrap_or_else(|| std::env::var("PH2D_CARDS").as_deref() != Ok("0"))
}

/// Força o modo NESTA thread (os testes de um processo correm em paralelo).
#[doc(hidden)]
pub fn force_cards_on_this_thread(on: Option<bool>) {
    FORCADO.with(|f| f.set(on));
}

/// Um cartão: o id da secção, o título autorado e os índices das rows dele na tabela viva.
struct Grupo {
    section: NodeId,
    titulo: String,
    linhas: Vec<usize>,
}

fn grupos(table: &[Row]) -> Vec<Grupo> {
    let mut out: Vec<Grupo> = Vec::new();
    for (i, row) in table.iter().enumerate() {
        if row.folds_a_section() {
            out.push(Grupo {
                section: row.id,
                titulo: row.label.clone(),
                linhas: Vec::new(),
            });
            continue;
        }
        if out.is_empty() {
            out.push(Grupo {
                section: crate::ids::AUTHORED_SEC_TOP,
                titulo: crate::generated::PANEL_TITLE.to_string(),
                linhas: Vec::new(),
            });
        }
        if let Some(g) = out.last_mut() {
            g.linhas.push(i);
        }
    }
    out
}

/// Uma row, pela porta única da pele: hit do controlo, um rect por opção e a lista aberta para o fim.
fn linha(c: &mut CardCtx<'_>, row: &Row, y: f32, abertas: &RefCell<Vec<OpenList>>) {
    let r = Rect::new(
        c.x,
        y,
        c.w,
        ph2d_editor_core::panel::cards::banda::metrics::FIELD_H,
    );
    let store = c.store;
    if row.wants_pointer() {
        c.hit_index.register(row.id, r);
    }
    for i in 0..row.options.len() {
        if let Some(or) = inline_option_rect(row.kind, r, i, row.options.len()) {
            c.hit_index
                .register(crate::ids::authored_option_id(&row.key, i), or);
        }
    }
    if row.kind.defers_a_popover()
        && matches!(
            store.get(row.id),
            Some(InteractiveState::Dropdown { open: true, .. })
        )
    {
        // O corpo corre mais de uma vez por quadro (voo, solto): vale o 1.º passe, o da lista.
        let mut abertas = abertas.borrow_mut();
        if !abertas.iter().any(|o| o.key == row.key) {
            abertas.push(OpenList {
                key: row.key.clone(),
                options: row.options.clone(),
                selected: rows::selected_of(store.get(row.id)).unwrap_or(0),
                chip: r,
            });
        }
    }
    paint_widget_skin_with(
        row.kind,
        &row.label,
        row.id,
        store.skin_live(row.id),
        SkinParam {
            rgba: row.rgba,
            body: row.body.map(|c| ph2d_tokens::skin_body::Body::Flat {
                colour: ph2d_tokens::Color {
                    r: c[0],
                    g: c[1],
                    b: c[2],
                    a: c[3],
                },
            }),
            options: &row.options,
            selected: rows::selected_of(store.get(row.id)).unwrap_or(0),
            icon: icon_glyph(row.icon_id, row.icon.as_ref()),
        },
        r,
        c.scene,
        c.text_system,
        c.ink.theme,
    );
}

/// ⭐ A lista flutua no rect de sempre; as listas abertas pintam-se por cima, depois das alças.
pub(crate) fn paint(ctx: &mut PaintCtx, rect: Rect) {
    ctx.host
        .store_mut()
        .mark_card_list_close(ids::AUTHORED_PANEL, crate::ids::AUTHORED_CLOSE);
    let grupos = rows::with_rows(|table| {
        for row in table {
            crate::populate::adopt(ctx.host.store_mut(), row);
        }
        grupos(table)
    });
    let abertas: Rc<RefCell<Vec<OpenList>>> = Rc::new(RefCell::new(Vec::new()));
    let cartoes: Vec<CardDef<'_>> = grupos
        .iter()
        .map(|g| {
            let linhas = g.linhas.clone();
            let abertas = Rc::clone(&abertas);
            CardDef {
                section: g.section,
                title: &g.titulo,
                actions: Vec::new(),
                add: None,
                summary: None,
                body: Some(Box::new(move |c, top| {
                    use ph2d_editor_core::panel::cards::banda::metrics as m;
                    let passo = m::FIELD_H + m::FIELD_ROW_GAP;
                    rows::with_rows(|table| {
                        let mut y = top;
                        for &i in &linhas {
                            if let Some(row) = table.get(i) {
                                linha(c, row, y, &abertas);
                                y += passo;
                            }
                        }
                        if y > top { y - m::FIELD_ROW_GAP } else { y }
                    })
                })),
            }
        })
        .collect();
    paint_floating_list(
        ctx,
        ids::AUTHORED_PANEL,
        crate::generated::PANEL_TITLE,
        &cartoes,
        rect,
        FloatHandles {
            drag: crate::ids::AUTHORED_DRAG_HANDLE,
            resize_br: Some(crate::ids::AUTHORED_RESIZE_HANDLE),
            resize_bl: crate::ids::AUTHORED_RESIZE_HANDLE_BL,
        },
    );
    let tema = ctx.host.theme();
    for aberta in abertas.borrow().iter() {
        paint_open_list(ctx, aberta, tema);
    }
}
