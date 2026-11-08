//! O quadro e os gestos da janela: um cartão de cima (busca, captura, pergunta, Import/Export) e um
//! cartão por secção com as acções que casam a busca. Cada mudança grava o delta e vai à família.

use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::panel::cards::ferramenta::cartao;
use ph2d_editor_core::panel::cards::{CardDef, Cell, Widget, borda, paint_card_list};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};
use ph2d_editor_core::widget::panel_chrome::panel_drag_handle_rect;
use ph2d_editor_core::widget::{ButtonState, TextInputState};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_pixel_keys::{Chord, Context, Keymap, PixelAction, Section};
use ph2d_tokens::Spacing;

use crate::lei::{Gesto, Janela, Modo, frase_tomada};
use crate::{PixelKeysPanel, PixelKeysPanelState, canal, ids};

fn botao(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

pub(crate) fn populate(store: &mut WidgetStore) {
    for id in ids::BOTOES {
        botao(store, id);
    }
    store.register(
        ids::SEARCH,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    store.register(
        ids::DRAG,
        InteractiveState::BlenderHit {
            parent: PixelKeysPanel::NODE_ID,
            kind: BlenderHitKind::DragHandle,
        },
    );
}

/// Uma linha da lista: a acção, as fichas das teclas e os ids dos botões dela.
struct Linha {
    acao: PixelAction,
    /// As fichas: o id, o texto e o contexto (`None` = `Normal`, na fileira compacta).
    fichas: Vec<(ph2d_a11y::NodeId, String, Option<&'static str>)>,
}

/// As acções que casam a busca, por secção, na ordem do diálogo quando a busca é vazia.
fn grupos(query: &str) -> Vec<(Section, Vec<PixelAction>)> {
    let achadas: Vec<PixelAction> = if query.trim().is_empty() {
        PixelAction::ALL.to_vec()
    } else {
        ph2d_pixel_keys::search(query)
            .into_iter()
            .map(|(a, _)| a)
            .collect()
    };
    let mut out: Vec<(Section, Vec<PixelAction>)> = Vec::new();
    for s in [
        Section::Menus,
        Section::Commands,
        Section::Tools,
        Section::Modifiers,
    ] {
        let v: Vec<_> = achadas
            .iter()
            .copied()
            .filter(|a| a.section() == s)
            .collect();
        if !v.is_empty() {
            out.push((s, v));
        }
    }
    out
}

/// O nome da tecla na tela: as teclas com nome de palavra passam pela tabela de strings.
fn texto_da_ficha(k: Chord, hover: bool) -> String {
    let mut t = k.to_string();
    if let Some(nome) = ph2d_pixel_keys::key_name(k.key) {
        let chave = format!("panel.pixel_keys.key.{}", nome.to_ascii_lowercase());
        let dito = tr(&chave);
        if dito != chave && t.ends_with(nome) {
            t.truncate(t.len() - nome.len());
            t.push_str(dito);
        }
    }
    if hover {
        t.push_str(" ×");
    }
    t
}

fn hovered(store: &WidgetStore, id: ph2d_a11y::NodeId) -> bool {
    matches!(
        store.get(id),
        Some(InteractiveState::Button {
            state: ButtonState::Hovered
        })
    )
}

fn linhas(store: &mut WidgetStore, keys: &Keymap, acoes: &[PixelAction]) -> Vec<Linha> {
    acoes
        .iter()
        .map(|&acao| {
            botao(store, ids::add(acao));
            botao(store, ids::reset(acao));
            let fichas = keys
                .keys_of(acao)
                .into_iter()
                .map(|(k, c)| {
                    let id = ids::ficha(acao, k);
                    botao(store, id);
                    store.set_tooltip(id, tr("panel.pixel_keys.remove_tip"));
                    let ctx = (c != Context::Normal).then(|| tr(c.name_key()));
                    (id, texto_da_ficha(k, hovered(store, id)), ctx)
                })
                .collect();
            Linha { acao, fichas }
        })
        .collect()
}

fn b(id: ph2d_a11y::NodeId, chave: &'static str, primary: bool) -> Cell<'static> {
    Cell::new(Widget::Button {
        id,
        text: tr(chave),
        primary,
    })
}

fn ficha(id: ph2d_a11y::NodeId, t: &str) -> Cell<'_> {
    Cell::new(Widget::Button {
        id,
        text: t,
        primary: false,
    })
}

fn cartao_secao(s: Section, linhas: Vec<Linha>) -> CardDef<'static> {
    cartao(ids::secao(s), tr(s.name_key()), None, move |c, mut y| {
        for l in &linhas {
            y = c.compact(
                &[
                    Cell::new(Widget::Readout {
                        text: l.acao.name(),
                    }),
                    b(ids::add(l.acao), "panel.pixel_keys.add", false),
                    b(ids::reset(l.acao), "panel.pixel_keys.reset", false),
                ],
                y,
            );
            if l.fichas.is_empty() {
                y = c.hint(tr("panel.pixel_keys.no_keys"), y);
            } else {
                let normais: Vec<Cell<'_>> = l
                    .fichas
                    .iter()
                    .filter(|f| f.2.is_none())
                    .map(|(id, t, _)| ficha(*id, t))
                    .collect();
                if !normais.is_empty() {
                    y = c.compact(&normais, y);
                }
                // As teclas de outro contexto: o nome do contexto numa linha, as fichas por baixo.
                let mut vistos: Vec<&str> = Vec::new();
                for onde in l.fichas.iter().filter_map(|f| f.2) {
                    if vistos.contains(&onde) {
                        continue;
                    }
                    vistos.push(onde);
                    y = c.hint(onde, y);
                    let dele: Vec<Cell<'_>> = l
                        .fichas
                        .iter()
                        .filter(|f| f.2 == Some(onde))
                        .map(|(id, t, _)| ficha(*id, t))
                        .collect();
                    y = c.compact(&dele, y);
                }
            }
        }
        y
    })
}

/// O cartão de cima: a busca e o que o modo pede.
fn cartao_topo(modo: Modo, aviso: Option<String>, vazio: bool) -> CardDef<'static> {
    let frase = match modo {
        Modo::Captura { acao, .. } => {
            Some(tr("panel.pixel_keys.press").replace("{action}", acao.name()))
        }
        Modo::Pergunta { tecla, dono, .. } => Some(frase_tomada(tecla, dono)),
        Modo::ResetAll => Some(tr("panel.pixel_keys.reset_all_ask").to_owned()),
        Modo::Lista => None,
    };
    let lida = match modo {
        Modo::Captura { tecla: Some(t), .. } => t.to_string(),
        _ => tr("panel.pixel_keys.waiting").to_owned(),
    };
    cartao(
        ids::SEC_TOP,
        tr("panel.pixel_keys.sec_top"),
        None,
        move |c, y| {
            let mut y = c.row(
                &[Cell::new(Widget::Text {
                    id: ids::SEARCH,
                    placeholder: tr("panel.pixel_keys.search"),
                })],
                y,
            );
            if let Some(f) = &frase {
                y = c.hint(f, y);
            }
            match modo {
                Modo::Captura { tecla, .. } => {
                    y = c.row(&[Cell::new(Widget::Readout { text: &lida })], y);
                    let confirma = if tecla.is_some() {
                        b(ids::CONFIRM, "panel.pixel_keys.confirm", true)
                    } else {
                        Cell::new(Widget::ButtonOff {
                            text: tr("panel.pixel_keys.confirm"),
                        })
                    };
                    y = c.row(
                        &[confirma, b(ids::CANCEL, "panel.pixel_keys.cancel", false)],
                        y,
                    );
                }
                Modo::Pergunta { .. } => {
                    y = c.row(
                        &[
                            b(ids::REPLACE, "panel.pixel_keys.replace", true),
                            b(ids::CANCEL, "panel.pixel_keys.cancel", false),
                        ],
                        y,
                    );
                }
                Modo::ResetAll => {
                    y = c.row(
                        &[
                            b(ids::CONFIRM, "panel.pixel_keys.reset_all", true),
                            b(ids::CANCEL, "panel.pixel_keys.cancel", false),
                        ],
                        y,
                    );
                }
                Modo::Lista => {
                    y = c.row(
                        &[
                            b(ids::IMPORT, "panel.pixel_keys.import", false),
                            b(ids::EXPORT, "panel.pixel_keys.export", false),
                            b(ids::RESET_ALL, "panel.pixel_keys.reset_all", false),
                            b(ids::CLOSE, "panel.pixel_keys.close", false),
                        ],
                        y,
                    );
                }
            }
            if let Some(a) = &aviso {
                y = c.hint(a, y);
            }
            if vazio {
                y = c.hint(tr("panel.pixel_keys.no_match"), y);
            }
            y
        },
    )
}

/// O rect da janela: à esquerda da coluna, como a SLICES; o arrasto do título desloca-a.
fn solta(ctx: &mut PaintCtx) -> Rect {
    let area = ctx.layout.draw_area;
    let livre = ctx.layout.clear_of_left_list(area);
    let teto = {
        let r = ph2d_editor_core::ruler::top_band(area);
        r.y + r.h + Spacing::Sm.px()
    };
    let fundo = area.y + area.h - Spacing::Sm.px();
    let coluna = ctx.slot.x;
    let store = ctx.host.store_mut();
    let mut w = borda::largura(store, PixelKeysPanel::NODE_ID, area, coluna);
    let esquerda = livre.x.max(area.x);
    w = w.min((coluna - Spacing::Lg.px() - esquerda).max(0.0));
    let base = (coluna - Spacing::Lg.px() - w).max(esquerda);
    let (dx, dy) = store.blender_picker_offset(PixelKeysPanel::NODE_ID);
    let x = (base + dx).clamp(esquerda, (coluna - Spacing::Lg.px() - w).max(esquerda));
    let y = (teto + dy).clamp(teto, (fundo - m::LIST_HEAD_H).max(teto));
    if (x - base, y - teto) != (dx, dy) {
        store.set_blender_picker_offset(PixelKeysPanel::NODE_ID, x - base, y - teto);
    }
    Rect::new(x, y, w, (fundo - y).max(0.0))
}

fn janela(st: &mut PixelKeysPanelState) -> &mut Janela {
    st.janela
        .get_or_insert_with(|| Janela::new(canal::carrega()))
}

pub(crate) fn paint(st: &mut PixelKeysPanelState, ctx: &mut PaintCtx) {
    let node = PixelKeysPanel::NODE_ID;
    if canal::take_abrir() {
        st.janela = None;
        ctx.host.set_panel_visible(PixelKeysPanel::ID, true);
    }
    if !ctx.host.panel_visible(PixelKeysPanel::ID) {
        // Só quem capturava larga a captura: outra tela sem a janela não mexe no canal.
        if let Some(j) = st.janela.as_mut().filter(|j| j.a_capturar()) {
            j.modo = Modo::Lista;
            canal::publica_captura(false);
        }
        ctx.host.store_mut().clear_panel_rect(node);
        return;
    }
    if janela(st).a_capturar()
        && let Some(t) = canal::take_tecla()
    {
        aplica(st, Gesto::Tecla(t));
    }
    let query = ctx
        .host
        .store()
        .text(ids::SEARCH)
        .unwrap_or_default()
        .to_owned();
    let j = janela(st);
    if j.a_capturar() {
        canal::publica_captura(true);
    }
    let (modo, aviso, keys) = (j.modo, j.aviso.clone(), j.keys.clone());
    let store = ctx.host.store_mut();
    let grupos = grupos(&query);
    let mut cartoes = vec![cartao_topo(modo, aviso, grupos.is_empty())];
    for (s, acoes) in grupos {
        let l = linhas(store, &keys, &acoes);
        cartoes.push(cartao_secao(s, l));
    }
    let slot = solta(ctx);
    let encaixe = ctx.slot;
    ctx.slot = slot;
    let lista = paint_card_list(ctx, node, PixelKeysPanel::TITLE.tr(), &cartoes);
    ctx.slot = encaixe;
    ctx.host.store_mut().set_panel_rect(node, lista);
    let reserva = (m::HEAD_ICON_BOX + m::LIST_HEAD_PAD_X) * 2.0;
    ctx.host.hit_index_mut().register(
        ids::DRAG,
        panel_drag_handle_rect(lista, m::LIST_HEAD_H, reserva),
    );
    borda::regista(ctx.host.store_mut(), ids::EDGE, node);
    borda::pega(ctx, ids::EDGE, lista);
}

/// Aplica o gesto; se o mapa mudou, grava o delta e entrega-o à família.
fn aplica(st: &mut PixelKeysPanelState, g: Gesto) {
    let j = janela(st);
    if j.aplica(g) {
        if let Err(e) = canal::grava(&j.keys) {
            j.aviso = Some(tr("panel.pixel_keys.save_failed").replace("{error}", &e.to_string()));
        }
        canal::publica_keymap(j.keys.clone());
    }
}

fn exporta(st: &mut PixelKeysPanelState) {
    let j = janela(st);
    let r = canal::ficheiro_de_troca()
        .ok_or_else(|| std::io::Error::other("no home"))
        .and_then(|p| std::fs::write(&p, j.keys.to_text()).map(|()| p));
    j.aviso = Some(match r {
        Ok(p) => tr("panel.pixel_keys.exported").replace("{path}", &p.display().to_string()),
        Err(e) => tr("panel.pixel_keys.save_failed").replace("{error}", &e.to_string()),
    });
}

fn importa(st: &mut PixelKeysPanelState) {
    let lido =
        canal::ficheiro_de_troca().and_then(|p| std::fs::read_to_string(&p).ok().map(|t| (p, t)));
    match lido {
        Some((_, t)) => aplica(st, Gesto::Importa(t)),
        None => {
            let p = canal::ficheiro_de_troca().map(|p| p.display().to_string());
            janela(st).aviso = Some(
                tr("panel.pixel_keys.import_missing").replace("{path}", &p.unwrap_or_default()),
            );
        }
    }
}

/// O gesto de um clique: os botões fixos e os de cada acção.
fn gesto_de(st: &mut PixelKeysPanelState, id: ph2d_a11y::NodeId) -> Option<Gesto> {
    Some(match id {
        i if i == ids::CONFIRM => Gesto::Confirma,
        i if i == ids::CANCEL => Gesto::Cancela,
        i if i == ids::REPLACE => Gesto::Replace,
        i if i == ids::RESET_ALL => Gesto::ResetAll,
        _ => {
            let keys = &janela(st).keys;
            PixelAction::ALL.iter().find_map(|&a| {
                if ids::add(a) == id {
                    return Some(Gesto::Add(a));
                }
                if ids::reset(a) == id {
                    return Some(Gesto::Reset(a));
                }
                keys.keys_of(a)
                    .into_iter()
                    .find(|(k, _)| ids::ficha(a, *k) == id)
                    .map(|(k, _)| Gesto::Remove(a, k))
            })?
        }
    })
}

pub(crate) fn apply_event(
    st: &mut PixelKeysPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let id = match ev {
        WidgetEvent::Click(id) | WidgetEvent::DoubleClick(id) => id,
        _ => return EventOutcome::Ignored,
    };
    if id == ids::CLOSE {
        canal::publica_captura(false);
        host.set_panel_visible(PixelKeysPanel::ID, false);
        return EventOutcome::Consumed;
    }
    if id == ids::EXPORT {
        exporta(st);
        return EventOutcome::Consumed;
    }
    if id == ids::IMPORT {
        importa(st);
        return EventOutcome::Consumed;
    }
    match gesto_de(st, id) {
        Some(g) => {
            let antes = janela(st).a_capturar();
            aplica(st, g);
            let agora = janela(st).a_capturar();
            if antes != agora {
                canal::publica_captura(agora);
            }
            EventOutcome::Consumed
        }
        None => EventOutcome::Ignored,
    }
}
