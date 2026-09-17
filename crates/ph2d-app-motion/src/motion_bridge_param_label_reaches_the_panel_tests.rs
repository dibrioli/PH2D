//! ⭐⭐⭐ **A PALAVRA CHEGA A QUEM PINTA — as duas superfícies, medidas pela porta do produto.**
//!
//! ⛔⛔ **Este gate existe porque os dois irmãos dele não podem vê-lo.** O
//! `every_param_label_is_a_key_derived_from_its_type_and_param` lê o REGISTO e afirma que o
//! rótulo é uma chave bem derivada; o `every_param_label_key_resolves_to_a_word` afirma que
//! essa chave tem palavra na tabela. **Nenhum dos dois pergunta se a palavra chega ao artista**
//! — um consumidor que se esqueça do `ph2d_i18n::tr` pinta `node.motion.wiggle.param.amount`
//! na linha e os dois ficam verdes. É o ponto cego que o `CLAUDE.md` §5.0 nomeia sobre si
//! mesmo: *nenhum instrumento do repo pergunta se o VALOR chega a um consumidor.*
//!
//! ⚠️ **São DUAS superfícies e elas têm consumidores DIFERENTES** — o painel lateral
//! (`build_params_snapshot`, que copia o rótulo para cada variante de `ParamRow`) e o CARTÃO
//! (`snapshot_from` + `stamp_card_params`, que o passa ao pintor do cartão). Medir só uma
//! deixaria a outra a pintar identificadores, e foi por isso que a migração tocou em treze
//! sítios e não em três.
//!
//! ⚠️ A régua é *«nenhum rótulo começa por `node.`»* e não *«o rótulo é igual ao esperado»*:
//! o que o artista nunca pode ler é o **identificador**, e a segunda metade já é o trabalho
//! dos dois gates do registo. ⭐ Ela reprova por ausência de tradução **e** por uma chave nova
//! que ninguém pôs na tabela, que são os dois modos de falha desta wave.

use super::stamp_card_params;
use crate::motion_bridge::params::build_params_snapshot;
use crate::motion_state::MotionState;
use ph2d_panel_motion_params::ParamRow;

/// O rótulo de uma row, qualquer que seja a variante — a struct não expõe um acessor.
fn rotulo(r: &ParamRow) -> &str {
    match r {
        ParamRow::Scalar(x) => &x.label,
        ParamRow::Color(x) => &x.label,
        ParamRow::Toggle(x) => &x.label,
        ParamRow::Enum(x) => &x.label,
        ParamRow::Angle(x) => &x.label,
        ParamRow::Seed(x) => &x.label,
        ParamRow::Text(x) => &x.label,
        ParamRow::Curve(x) => &x.label,
        ParamRow::Gradient(x) => &x.label,
        ParamRow::Palette(x) => &x.label,
        ParamRow::Channels(x) => &x.label,
        ParamRow::Source(x) => &x.label,
        ParamRow::File(x) => &x.label,
    }
}

#[test]
fn no_param_row_of_any_node_paints_a_raw_key() {
    let tipos: Vec<String> = {
        let base = MotionState::new();
        base.registry
            .manifests()
            .map(|m| m.name.to_string())
            .collect()
    };
    // ⛔ Piso de população: com o catálogo vazio a varredura não vê nada e passa.
    assert!(tipos.len() >= 130, "só {} tipos — encolheu?", tipos.len());

    let mut cruas = Vec::new();
    let mut linhas = 0usize;
    for nome in &tipos {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node(nome.clone());
        ph2d_panel_motion_graph::set_graph_selection(vec![id.0]);
        let Some(snap) = build_params_snapshot(&m, ph2d_editor_core::ProjectSettings::default())
        else {
            continue;
        };
        for r in &snap.rows {
            linhas += 1;
            let l = rotulo(r);
            if l.starts_with("node.") {
                cruas.push(format!("painel · {nome}: {l:?}"));
            }
        }
    }
    assert!(linhas >= 700, "só {linhas} linhas de painel — encolheu?");
    assert!(
        cruas.is_empty(),
        "estas {} linhas do painel pintam o IDENTIFICADOR e não a palavra — falta o \
         `ph2d_i18n::tr` no consumidor:\n  {}",
        cruas.len(),
        cruas.join("\n  ")
    );
}

/// ⭐⭐ **E a MESMA lei no CARTÃO**, que é o outro consumidor do mesmo rótulo.
///
/// ⚠️ Ele não passa pelo `build_params_snapshot`: o cartão lê o `hint` directamente
/// (`snapshot_from` + `stamp_card_params`), logo uma cura feita só no painel deixa-o a pintar
/// identificadores — *duas superfícies sobre o mesmo dado são dois sítios onde o `tr` pode
/// faltar, e uma régua sobre uma delas não afirma nada sobre a outra.*
#[test]
fn no_card_param_of_any_node_paints_a_raw_key() {
    let tipos: Vec<String> = {
        let base = MotionState::new();
        base.registry
            .manifests()
            .map(|m| m.name.to_string())
            .collect()
    };
    assert!(tipos.len() >= 130, "só {} tipos — encolheu?", tipos.len());

    let mut cruas = Vec::new();
    let mut vistos = 0usize;
    for nome in &tipos {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node(nome.clone());
        ph2d_panel_motion_graph::set_graph_selection(vec![id.0]);
        let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
        stamp_card_params(&m, ph2d_editor_core::ProjectSettings::default(), &mut snap);
        let Some(v) = snap.nodes.iter().find(|v| v.id == id.0) else {
            continue;
        };
        for c in &v.params {
            vistos += 1;
            if c.hint.label.starts_with("node.") {
                cruas.push(format!("cartão · {nome}::{}", c.hint.param));
            }
        }
    }
    assert!(
        vistos >= 1,
        "o cartão não mostrou um único param — encolheu?"
    );
    assert!(
        cruas.is_empty(),
        "estes {} params do CARTÃO carregam o identificador cru:\n  {}",
        cruas.len(),
        cruas.join("\n  ")
    );
}

/// ⭐⭐⭐ **E A LISTA DE CANAIS ABRE COM PALAVRAS** — a quarta superfície, e a que sobrevive.
///
/// ⚠️ O picker de canal é o único selector do catálogo cujas opções **não** são um
/// `ParamWidget::Enum`: são `ReadChannel`s, e a lista que o artista abre é montada por
/// `channel_labels` — que mistura canais CURADOS (que têm nome) com colunas vindas da corrente de
/// cima (que **são** o próprio nome). ⛔ Um `tr` em falta ali põe `node.channel.p.length` no menu,
/// e nenhum dos gates do registo o vê: eles afirmam que a chave deriva e que resolve, não que
/// alguém a resolveu.
#[test]
fn no_channel_list_of_any_node_paints_a_raw_key() {
    use ph2d_node_registry::ParamWidget;
    let tipos: Vec<String> = {
        let base = MotionState::new();
        base.registry
            .manifests()
            .map(|m| m.name.to_string())
            .collect()
    };
    assert!(tipos.len() >= 130, "só {} tipos — encolheu?", tipos.len());

    let mut cruas = Vec::new();
    let mut vistas = 0usize;
    for nome in &tipos {
        let mut m = MotionState::new();
        let id = m.doc.graph.add_node(nome.clone());
        let Some(hints) = m
            .registry
            .manifests()
            .find(|man| man.name == nome.as_str())
            .and_then(|man| m.registry.param_ui(man.id))
        else {
            continue;
        };
        let pickers: Vec<_> = hints
            .iter()
            .filter_map(|h| match h.widget {
                ParamWidget::Channels {
                    mode_param,
                    channels,
                } => Some((h.param, mode_param, channels)),
                _ => None,
            })
            .collect();
        for (text_param, mode_param, channels) in pickers {
            let (rotulos, _) = crate::motion_bridge::params::channel_labels_for_tests(
                &m, id, text_param, mode_param, channels,
            );
            for r in &rotulos {
                vistas += 1;
                if r.starts_with("node.") {
                    cruas.push(format!("{nome}::{text_param}: {r:?}"));
                }
            }
        }
    }
    // ⛔ Piso de população: sem um picker aberto a varredura mede nada.
    assert!(vistas >= 24, "só {vistas} rótulos de canal — encolheu?");
    assert!(
        cruas.is_empty(),
        "estes {} rótulos da lista de CANAIS carregam o identificador cru:\n  {}",
        cruas.len(),
        cruas.join("\n  ")
    );
}
