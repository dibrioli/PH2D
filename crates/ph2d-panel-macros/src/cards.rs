//! ⭐⭐ **O painel Macros em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`): os MESMOS ids e
//! a mesma tabela de pedidos; o despacho (`seam.rs`) não muda. As linhas de lista (etapa, macro) e as
//! frases são as do corpo de antes ([`Corpo`]), na tinta do cartão. Um verbo que a tabela não pode
//! pedir agora pinta-se apagado e sem clique, como antes.

use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::macro_edits::{
    FamiliaDaMacro, MacroPanelInfo, PropsDaEtapa, TipoDaEtapaView,
};
use ph2d_editor_core::panel::cards::corpo::letra_da_grandeza;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget};
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::ColorToken;

use crate::biblioteca::{aviso_e_erro, frase};
use crate::corpo::{Corpo, escolhida, frase_de_gravar};
use crate::ids;
use crate::nomes;
use crate::seam::{CAMPOS, PASSO_FINO, pedido_da_biblioteca};
use crate::state::MacrosPanelState;

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

/// Força o modo NESTA thread (a variável de ambiente seria partilhada entre testes paralelos).
#[doc(hidden)]
pub fn force_cards_on_this_thread(on: Option<bool>) {
    FORCADO.with(|f| f.set(on));
}

const SECCOES: [NodeId; 4] = [
    ids::MACROS_SEC_RECORD,
    ids::MACROS_SEC_STEPS,
    ids::MACROS_SEC_STEP,
    ids::MACROS_SEC_LIBRARY,
];

/// Antes de pintar: fecho, secções, pegas e os campos marcados (os valores vêm do `semeia`).
pub(crate) fn prepare(store: &mut WidgetStore) {
    use ph2d_editor_core::interaction::InteractiveState;
    store.mark_card_list_close(ph2d_editor_core::ids::MACROS_PANEL, ids::MACROS_CLOSE);
    for id in SECCOES {
        store.mark_collapsible_section(id);
        let grip = ph2d_editor_core::ids::grip_de(id);
        let _ = store.register_if_absent(grip, InteractiveState::Plain);
        if store.tooltip_for(grip).is_none() {
            store.set_tooltip(grip, tr("chrome.section.grip_hint"));
        }
    }
    for (id, ritmo) in CAMPOS {
        let step = if id == ids::MACROS_SIZE {
            1.0
        } else {
            PASSO_FINO
        };
        store.mark_card_field(
            id,
            CardFieldSpec {
                step,
                scrub: ritmo,
                default: None,
            },
        );
    }
}

fn corpo<'a>(c: &'a mut CardCtx<'_>) -> Corpo<'a> {
    Corpo {
        scene: &mut *c.scene,
        text_system: &mut *c.text_system,
        store: c.store,
        hit_index: &mut *c.hit_index,
        theme: c.ink.theme,
        x: c.x,
        w: c.w,
        ink: Some(c.ink),
    }
}

/// Gravar ▸ Parar e o que ele vai fazer; sem quem grave, a frase diz porquê.
fn corpo_de_gravar(c: &mut CardCtx<'_>, info: &MacroPanelInfo, y: f32) -> f32 {
    let pode = info.pode_gravar.is_some() || info.objeto_macro.is_some();
    let botao = match (info.armado || info.gravando.is_some(), pode) {
        (true, _) => Some((ids::MACROS_STOP, tr("panel.macros.stop"), false)),
        (false, true) => Some((ids::MACROS_RECORD, tr("panel.macros.record"), true)),
        (false, false) => None,
    };
    let Some((id, text, primary)) = botao else {
        let k = info
            .recusa
            .unwrap_or("panel.macros.refusal.nothing_to_record");
        return corpo(c).bloco(tr(k), ColorToken::Text3, y);
    };
    let y = c.full(None, Widget::Button { id, text, primary }, y);
    let y = corpo(c).bloco(&frase_de_gravar(info), ColorToken::Text3, y);
    match info.recusa {
        Some(k) => corpo(c).bloco(tr(k), ColorToken::Danger, y),
        None => y,
    }
}

fn corpo_das_etapas(
    c: &mut CardCtx<'_>,
    info: &MacroPanelInfo,
    st: &MacrosPanelState,
    y: f32,
) -> f32 {
    let Some(o) = &info.objeto else {
        return corpo(c).bloco(tr("panel.macros.empty"), ColorToken::Text3, y);
    };
    let mut p = corpo(c);
    let resumo = tr_with(
        "panel.macros.summary",
        &[
            ("steps", &o.etapas.len()),
            ("seconds", &nomes::segundos(o.duracao_s)),
        ],
    );
    let mut y = p.linha(&resumo, ColorToken::Text2, y);
    if let Some(n) = o.editada_desde {
        y = p.linha(
            &tr_with("panel.macros.edited_since", &[("n", &(n + 1))]),
            ColorToken::Text3,
            y,
        );
    }
    if let Some(e) = &o.erro {
        y = p.bloco(&frase(e), ColorToken::Danger, y);
    }
    if !o.etapas.is_empty() {
        y += ph2d_card_stack::metrics::FIELD_ROW_GAP;
    }
    for (i, e) in o.etapas.iter().enumerate() {
        let esta = st
            .escolhida
            .is_some_and(|s| s.entity_bits == o.entity_bits && s.trilha == e.trilha && s.k == e.k);
        y = p.etapa(i, e, esta, esta && st.renomeando, y);
    }
    y
}

fn botao(id: NodeId, text: &'static str) -> Cell<'static> {
    Cell::new(Widget::Button {
        id,
        text,
        primary: false,
    })
}

fn campo(id: NodeId, label: &'static str) -> Cell<'static> {
    Cell::new(Widget::Field {
        id,
        glyph: Glyph::Text(letra_da_grandeza(label)),
    })
    .label(label)
}

/// As propriedades da etapa escolhida: renomear, apagar (a do mundo não se apaga), o balão, a espera e
/// as da família.
fn corpo_da_etapa(
    c: &mut CardCtx<'_>,
    info: &MacroPanelInfo,
    st: &MacrosPanelState,
    y: f32,
) -> f32 {
    let Some(e) = info
        .objeto
        .as_ref()
        .and_then(|o| st.escolhida.and_then(|s| escolhida(o, s)))
    else {
        return y;
    };
    let renomear = botao(ids::MACROS_RENAME, tr("panel.macros.rename"));
    let mut y = if e.familia == FamiliaDaMacro::Mundo {
        c.row(&[renomear], y)
    } else {
        let apagar = botao(ids::MACROS_DELETE, tr("panel.macros.delete_step"));
        c.row(&[renomear, apagar], y)
    };
    let balao = botao(ids::MACROS_HINT, tr("panel.macros.hint"));
    let espera = campo(ids::MACROS_WAIT, tr("card.macros.wait"));
    match e.props {
        PropsDaEtapa::Pincelada { cor, .. } => {
            y = c.row(&[balao, espera], y);
            let rgba = c
                .store
                .widget_color(ids::MACROS_COLOR)
                .unwrap_or([cor[0], cor[1], cor[2], 255]);
            let codigo = format!("{:02X}{:02X}{:02X}", rgba[0], rgba[1], rgba[2]);
            let amostra = Cell::new(Widget::Swatch {
                id: ids::MACROS_COLOR,
                rgba: Some(rgba),
                text: &codigo,
                edit: None,
                detach: None,
            })
            .label(tr("panel.macros.field.color"));
            // O ColorDrop inunda pela cor: o tamanho do pincel não lhe chega.
            if matches!(e.tipo, TipoDaEtapaView::Preenchimento(_)) {
                c.row(&[amostra], y)
            } else {
                c.row(
                    &[campo(ids::MACROS_SIZE, tr("card.macros.size")), amostra],
                    y,
                )
            }
        }
        PropsDaEtapa::Param { .. } if e.familia == FamiliaDaMacro::Motion => {
            y = c.row(&[balao], y);
            let valor = campo(ids::MACROS_VALUE, tr("panel.macros.field.value"));
            c.row(&[espera, valor], y)
        }
        _ => c.row(&[balao, espera], y),
    }
}

/// Um verbo da tabela: activo só se a tabela tem pedido para ele.
fn verbo<'a>(
    info: &MacroPanelInfo,
    nome: Option<&str>,
    id: NodeId,
    text: &'a str,
    primary: bool,
) -> Cell<'a> {
    Cell::new(if pedido_da_biblioteca(id, info, nome).is_some() {
        Widget::Button { id, text, primary }
    } else {
        Widget::ButtonOff { text }
    })
}

fn corpo_da_biblioteca(
    c: &mut CardCtx<'_>,
    info: &MacroPanelInfo,
    st: &MacrosPanelState,
    y: f32,
) -> f32 {
    let nome = st.lib_escolhida.as_deref();
    let v = |id, k| verbo(info, nome, id, tr(k), false);
    let mut y = c.row(
        &[
            v(ids::MACROS_LIB_SAVE, "panel.macros.lib.save"),
            v(ids::MACROS_LIB_SAVE_ASSET, "panel.macros.lib.save_asset"),
            v(ids::MACROS_LIB_IMPORT, "panel.macros.lib.import"),
        ],
        y,
    );
    let mut p = corpo(c);
    if let Some(a) = &info.aviso {
        let cor = if aviso_e_erro(a.chave) {
            ColorToken::Danger
        } else {
            ColorToken::Text2
        };
        y = p.bloco(&frase(a), cor, y);
    }
    if info.biblioteca_ilegiveis > 0 {
        y = p.bloco(
            &tr_with(
                "panel.macros.lib.unreadable",
                &[("n", &info.biblioteca_ilegiveis)],
            ),
            ColorToken::Danger,
            y,
        );
    }
    if info.biblioteca.is_empty() {
        return p.bloco(tr("panel.macros.lib.empty"), ColorToken::Text3, y);
    }
    y += m::FIELD_ROW_GAP;
    for (i, macro_) in info.biblioteca.iter().enumerate() {
        if i > 0 {
            y += ph2d_tokens::list_row_gap_px();
        }
        let esta = nome == Some(macro_.nome.as_str());
        y = p.entrada(macro_, esta, esta && st.lib_renomeando, y);
    }
    if nome.is_none() {
        return y;
    }
    let mut y = c.row(
        &[verbo(
            info,
            nome,
            ids::MACROS_LIB_APPLY_NEW,
            tr("panel.macros.lib.apply_new"),
            true,
        )],
        y,
    );
    match (&info.por_cima_de, info.por_cima_alvo) {
        (Some(n), Some(_)) => {
            let rotulo = tr_with("panel.macros.lib.apply_over", &[("name", n)]);
            y = c.row(
                &[verbo(
                    info,
                    nome,
                    ids::MACROS_LIB_APPLY_OVER,
                    &rotulo,
                    false,
                )],
                y,
            );
        }
        _ => {
            let k = info
                .por_cima_recusa
                .unwrap_or("panel.macros.lib.apply_over_needs_target");
            y = corpo(c).bloco(tr(k), ColorToken::Text3, y);
        }
    }
    let v = |id, k| verbo(info, nome, id, tr(k), false);
    c.row(
        &[
            v(ids::MACROS_LIB_EXPORT, "panel.macros.lib.export"),
            v(ids::MACROS_LIB_DELETE, "panel.macros.lib.delete"),
        ],
        y,
    )
}

/// ⭐ Os cartões: gravar, as etapas, a etapa escolhida (só com uma) e a biblioteca.
pub(crate) fn cards<'a>(info: &'a MacroPanelInfo, st: &'a MacrosPanelState) -> Vec<CardDef<'a>> {
    let cartao =
        |section, title, summary, body: Box<dyn Fn(&mut CardCtx<'_>, f32) -> f32 + 'a>| CardDef {
            section,
            title,
            actions: Vec::new(),
            add: None,
            summary,
            body: Some(body),
        };
    let resumo = info.objeto.as_ref().map(|o| o.etapas.len().to_string());
    let mut out = vec![
        cartao(
            ids::MACROS_SEC_RECORD,
            tr("card.macros.record"),
            None,
            Box::new(move |c, y| corpo_de_gravar(c, info, y)),
        ),
        cartao(
            ids::MACROS_SEC_STEPS,
            tr("card.macros.steps"),
            resumo,
            Box::new(move |c, y| corpo_das_etapas(c, info, st, y)),
        ),
    ];
    let com_etapa = info
        .objeto
        .as_ref()
        .is_some_and(|o| st.escolhida.and_then(|s| escolhida(o, s)).is_some());
    if com_etapa {
        out.push(cartao(
            ids::MACROS_SEC_STEP,
            tr("card.macros.step"),
            None,
            Box::new(move |c, y| corpo_da_etapa(c, info, st, y)),
        ));
    }
    out.push(cartao(
        ids::MACROS_SEC_LIBRARY,
        tr("panel.macros.lib.title"),
        Some(info.biblioteca.len().to_string()),
        Box::new(move |c, y| corpo_da_biblioteca(c, info, st, y)),
    ));
    out
}
