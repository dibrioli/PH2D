//! O quadro e os gestos da janela: um cartão de cima (busca, captura, pergunta, Import/Export) e um
//! cartão por secção com as acções que casam a busca. Cada mudança grava o delta e vai à família.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::panel::cards::ferramenta::cartao;
use ph2d_editor_core::panel::cards::{CardDef, Cell, Seg, Widget, borda, paint_card_list};
use ph2d_editor_core::panel::{EventOutcome, PaintCtx, Panel, PanelHostInternal};
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::widget::panel_chrome::panel_drag_handle_rect;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_pixel_keys::{Chord, Context, Keymap, PixelAction, Section};
use ph2d_tokens::Spacing;

use crate::fila::{self, Campo, Linha};
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
    store.set_tooltip(ids::IMPORT, tr("panel.pixel_keys.import_tip"));
    store.set_tooltip(ids::EXPORT, tr("panel.pixel_keys.export_tip"));
    ph2d_editor_core::panel::cards::busca::regista(store, ids::SEARCH);
    store.register(
        ids::DRAG,
        InteractiveState::BlenderHit {
            parent: PixelKeysPanel::NODE_ID,
            kind: BlenderHitKind::DragHandle,
        },
    );
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
fn texto_da_ficha(k: Chord) -> String {
    let mut t = k.to_string();
    if let Some(nome) = ph2d_pixel_keys::key_name(k.key) {
        let chave = format!("panel.pixel_keys.key.{}", nome.to_ascii_lowercase());
        let dito = tr(&chave);
        if dito != chave && t.ends_with(nome) {
            t.truncate(t.len() - nome.len());
            t.push_str(dito);
        }
    }
    t
}

/// O texto do campo em captura: os modificadores seguros («Ctrl+…») ou o convite.
fn texto_em_captura() -> String {
    let espera = tr("panel.pixel_keys.press");
    let Some(m) = canal::seguros() else {
        return espera.to_owned();
    };
    use winit::keyboard::KeyCode as K;
    let k = m.key;
    let mut t = String::new();
    for (on, nome) in [
        (
            m.ctrl || matches!(k, K::ControlLeft | K::ControlRight),
            "Ctrl+",
        ),
        (m.alt || matches!(k, K::AltLeft | K::AltRight), "Alt+"),
        (
            m.shift || matches!(k, K::ShiftLeft | K::ShiftRight),
            "Shift+",
        ),
        (m.space || k == K::Space, "Space+"),
    ] {
        if on {
            t.push_str(nome);
        }
    }
    if t.is_empty() {
        return espera.to_owned();
    }
    t.push('\u{2026}');
    t
}

/// As fileiras: cada tecla um campo; o campo vazio quando não há tecla ou é ele o capturado (o
/// clique no NOME da acção abre-o: a 2.ª tecla sem `+`); a recusa e a pergunta na fileira.
fn linhas(store: &mut WidgetStore, j: &Janela, acoes: &[PixelAction]) -> Vec<Linha> {
    let fabrica = Keymap::factory();
    let (keys, modo, campo) = (&j.keys, j.modo, j.campo());
    acoes
        .iter()
        .map(|&acao| {
            for (id, dica) in [
                (ids::add(acao), "panel.pixel_keys.add_tip"),
                (ids::reset(acao), "panel.pixel_keys.reset_tip"),
            ] {
                botao(store, id);
                store.set_tooltip(id, tr(dica));
            }
            // O nome não leva dica: ela taparia o campo vazio que o pairar mostra.
            botao(store, ids::nome(acao));
            let teclas = keys.keys_of(acao);
            let aceso_em = |alvo: Option<(Chord, Context)>| campo == Some((acao, alvo));
            let texto_aceso = || match modo {
                Modo::Pergunta { tecla, .. } => texto_da_ficha(tecla),
                _ => texto_em_captura(),
            };
            let mut campos: Vec<Campo> = Vec::new();
            for &(k, c) in &teclas {
                let id = ids::ficha(acao, k);
                botao(store, id);
                store.set_tooltip(id, tr("panel.pixel_keys.remove_tip"));
                let aceso = aceso_em(Some((k, c)));
                let texto = if aceso {
                    texto_aceso()
                } else if c == Context::Normal {
                    texto_da_ficha(k)
                } else {
                    format!("{} · {}", texto_da_ficha(k), tr(c.name_key()))
                };
                campos.push(Campo { id, texto, aceso });
            }
            let vazio_aceso = aceso_em(None);
            if teclas.is_empty() || vazio_aceso {
                let texto = if vazio_aceso {
                    texto_aceso()
                } else {
                    tr("panel.pixel_keys.empty").to_owned()
                };
                campos.insert(
                    0,
                    Campo {
                        id: ids::add(acao),
                        texto,
                        aceso: vazio_aceso,
                    },
                );
            }
            // A recusa (tecla do app) fica na fileira do campo, que continua à espera.
            let recusa = match modo {
                Modo::Captura { campo: (a, _) } if a == acao => j.aviso.clone(),
                _ => None,
            };
            let pergunta = match modo {
                Modo::Pergunta {
                    campo: (a, _),
                    tecla,
                    dono,
                } if a == acao => Some(frase_tomada(tecla, dono)),
                _ => None,
            };
            let mut agora = teclas;
            let mut antes = fabrica.keys_of(acao);
            agora.sort_by_key(|(k, c)| (k.to_string(), *c));
            antes.sort_by_key(|(k, c)| (k.to_string(), *c));
            Linha {
                acao,
                campos,
                mudada: agora != antes,
                recusa,
                pergunta,
            }
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

fn cartao_secao(s: Section, linhas: Vec<Linha>) -> CardDef<'static> {
    cartao(ids::secao(s), tr(s.name_key()), None, move |c, mut y| {
        for l in &linhas {
            y = fila::pinta(c, l, y);
        }
        y
    })
}

/// O cartão de cima: a busca, Reset All (com a confirmação) e a troca; a captura vive na fileira.
fn cartao_topo(modo: Modo, aviso: Option<String>, vazio: bool) -> CardDef<'static> {
    cartao(
        ids::SEC_TOP,
        tr("panel.pixel_keys.sec_top"),
        None,
        move |c, y| {
            let mut y = c.row(
                &[Cell::new(Widget::Search {
                    id: ids::SEARCH,
                    placeholder: tr("panel.pixel_keys.search"),
                })],
                y,
            );
            if modo == Modo::ResetAll {
                y = c.hint(tr("panel.pixel_keys.reset_all_ask"), y);
                y = c.row(
                    &[
                        b(ids::CONFIRM, "panel.pixel_keys.reset_all", true),
                        b(ids::CANCEL, "panel.pixel_keys.cancel", false),
                    ],
                    y,
                );
            } else {
                // Uma fileira: Reset All e os ícones da troca.
                let troca = [
                    Seg {
                        id: ids::IMPORT,
                        icon: Some(IconId::Open),
                        text: "",
                        on: false,
                    },
                    Seg {
                        id: ids::EXPORT,
                        icon: Some(IconId::Export),
                        text: "",
                        on: false,
                    },
                ];
                y = c.row(
                    &[
                        b(ids::RESET_ALL, "panel.pixel_keys.reset_all", false),
                        Cell::new(Widget::Bare(&troca)),
                    ],
                    y,
                );
            }
            // Em captura o aviso (a recusa) está DENTRO do campo.
            if let (Some(a), false) = (&aviso, matches!(modo, Modo::Captura { .. })) {
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
    let fundo = borda::fundo(ctx.layout);
    let coluna = ctx.slot.x;
    if ctx
        .host
        .store()
        .accessory_width(PixelKeysPanel::NODE_ID)
        .is_none()
    {
        let w = fila::largura_de_fabrica(ctx.text_system);
        ctx.host
            .store_mut()
            .set_accessory_width(PixelKeysPanel::NODE_ID, w);
    }
    let store = ctx.host.store_mut();
    let mut w = borda::largura(store, PixelKeysPanel::NODE_ID, area, coluna);
    let esquerda = livre.x.max(area.x);
    w = w.min((coluna - Spacing::Lg.px() - esquerda).max(0.0));
    let base = (coluna - Spacing::Lg.px() - w).max(esquerda);
    borda::coloca(
        store,
        PixelKeysPanel::NODE_ID,
        w,
        (esquerda, base),
        (teto, fundo),
    )
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
    match canal::take_escolhido() {
        Some((canal::Troca::Export, p)) => exporta(st, &p),
        Some((canal::Troca::Import, p)) => importa(st, &p),
        None => {}
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
    canal::publica_captura(j.a_capturar());
    if !j.a_capturar() {
        canal::larga_seguros();
    }
    let j = j.clone();
    canal::esquece_pintados();
    let store = ctx.host.store_mut();
    let grupos = grupos(&query);
    let mut cartoes = vec![cartao_topo(j.modo, j.aviso.clone(), grupos.is_empty())];
    for (s, acoes) in grupos {
        let l = linhas(store, &j, &acoes);
        cartoes.push(cartao_secao(s, l));
    }
    let slot = solta(ctx);
    let encaixe = ctx.slot;
    ctx.slot = slot;
    borda::declara_fecho(ctx.host.store_mut(), node);
    let lista = paint_card_list(ctx, node, PixelKeysPanel::TITLE.tr(), &cartoes);
    ctx.slot = encaixe;
    ctx.host.store_mut().set_panel_rect(node, lista);
    let reserva = borda::reserva_do_titulo();
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

fn exporta(st: &mut PixelKeysPanelState, p: &std::path::Path) {
    let j = janela(st);
    j.aviso = Some(match ph2d_pixel_keys::user_file::save_to(p, &j.keys) {
        Ok(()) => tr("panel.pixel_keys.exported").replace("{path}", &p.display().to_string()),
        Err(e) => tr("panel.pixel_keys.save_failed").replace("{error}", &e.to_string()),
    });
}

/// O Import SOMA (Aseprite R6).
fn importa(st: &mut PixelKeysPanelState, p: &std::path::Path) {
    match std::fs::read_to_string(p) {
        Ok(t) => aplica(st, Gesto::Importa(t)),
        Err(_) => {
            janela(st).aviso = Some(
                tr("panel.pixel_keys.import_missing").replace("{path}", &p.display().to_string()),
            );
        }
    }
}

/// O gesto de um clique: os botões fixos e os campos de cada acção.
fn gesto_de(st: &mut PixelKeysPanelState, id: ph2d_a11y::NodeId) -> Option<Gesto> {
    Some(match id {
        i if i == ids::CONFIRM => Gesto::Confirma,
        i if i == ids::CANCEL => Gesto::Cancela,
        i if i == ids::REPLACE => Gesto::Replace,
        i if i == ids::RESET_ALL => Gesto::ResetAll,
        _ => {
            let keys = &janela(st).keys;
            PixelAction::ALL.iter().find_map(|&a| {
                if ids::add(a) == id || ids::nome(a) == id {
                    return Some(Gesto::Campo((a, None)));
                }
                if ids::reset(a) == id {
                    return Some(Gesto::Reset(a));
                }
                keys.keys_of(a)
                    .into_iter()
                    .find(|(k, _)| ids::ficha(a, *k) == id)
                    .map(|t| Gesto::Campo((a, Some(t))))
            })?
        }
    })
}

pub(crate) fn apply_event(
    st: &mut PixelKeysPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    if ph2d_editor_core::panel::cards::busca::limpou(host.store_mut(), &ev, ids::SEARCH) {
        return EventOutcome::Consumed;
    }
    let id = match ev {
        WidgetEvent::Click(id) | WidgetEvent::DoubleClick(id) => id,
        _ => return EventOutcome::Ignored,
    };
    // Clicar fora do campo em captura cancela-a (o clique segue para o seu alvo).
    if janela(st).a_capturar() && !matches!(gesto_de(st, id), Some(Gesto::Campo(_))) {
        janela(st).aplica(Gesto::Cancela);
        canal::publica_captura(false);
    }
    if id == ids::EXPORT || id == ids::IMPORT {
        let t = if id == ids::EXPORT {
            canal::Troca::Export
        } else {
            canal::Troca::Import
        };
        canal::pede_troca(t);
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
