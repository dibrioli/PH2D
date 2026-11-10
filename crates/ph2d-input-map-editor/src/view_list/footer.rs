//! As frases do pé: o resumo do mapa (com a tecla repetida explicada ou avisada), a busca por tecla e a escuta.

use ph2d_i18n::tr;
use ph2d_input::readers::{Reader, shared_bindings};
use ph2d_input::{ActionId, Binding, InputMap, Key};

use crate::snapshot::Snapshot;

/// Uma frase do pé: a parte de destaque, o resto, e se é um aviso (âmbar).
pub(crate) struct Phrase {
    pub lead: String,
    pub rest: String,
    pub warn: bool,
}

/// O nome que o artista lê de uma ligação.
#[must_use]
pub fn binding_label(b: Binding) -> String {
    match b {
        Binding::Key(k) => k.label(),
        Binding::PadButton(p) => tr(p.label_key()).to_owned(),
        Binding::PadAxis { axis, .. } => tr(axis.label_key()).to_owned(),
    }
}

/// «A, B and C».
fn and_list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [head @ .., last] => format!(
            "{}{}{last}",
            head.join(tr("input_map_list.comma")),
            tr("input_map_list.and")
        ),
    }
}

fn readers_of<'a>(snap: &'a Snapshot, name: &str) -> &'a [Reader] {
    snap.readers.get(name).map_or(&[], Vec::as_slice)
}

fn reader_names(rs: &[Reader]) -> String {
    let names: Vec<String> = rs
        .iter()
        .map(|r| match r {
            Reader::Trigger { object } => object.clone(),
            Reader::Script { name } => name.clone(),
            other => other
                .label_key()
                .map_or_else(String::new, |k| tr(k).to_owned()),
        })
        .collect();
    and_list(&names)
}

fn name_of(map: &InputMap, id: ActionId) -> String {
    map.get(id).map_or_else(String::new, |a| a.name.clone())
}

/// O resumo do mapa e, por baixo, um aviso por tecla repetida por engano.
pub(crate) fn summary(snap: &Snapshot) -> Vec<Phrase> {
    let map = &snap.map;
    let keys = map
        .actions()
        .iter()
        .flat_map(|a| &a.bindings)
        .filter(|b| matches!(b, Binding::Key(_)))
        .count();
    let lead = tr("input_map_list.summary")
        .replace("{n}", &map.len().to_string())
        .replace("{k}", &keys.to_string());
    let shared = shared_bindings(map, &snap.readers);
    let fine: Vec<String> = shared
        .iter()
        .filter(|s| s.on_purpose)
        .map(|s| binding_label(s.binding))
        .collect();
    let rest = match fine.len() {
        0 => String::new(),
        1 => tr("input_map_list.shared_one").replace("{keys}", &fine[0]),
        _ => tr("input_map_list.shared_many").replace("{keys}", &and_list(&fine)),
    };
    let mut out = vec![Phrase {
        lead,
        rest,
        warn: false,
    }];
    for s in shared.iter().filter(|s| !s.on_purpose) {
        let names: Vec<String> = s.actions.iter().map(|a| name_of(map, *a)).collect();
        out.push(Phrase {
            lead: tr("input_map_list.shared_warn_lead")
                .replace("{key}", &binding_label(s.binding))
                .replace("{actions}", &and_list(&names)),
            rest: tr("input_map_list.shared_warn_rest").to_owned(),
            warn: true,
        });
    }
    out
}

/// «Up Arrow is in 2 of 11 actions. jump is read by Platform Player, move_up by Top-Down Player.»
pub(crate) fn search(snap: &Snapshot, key: Key, shown: &[ActionId]) -> Phrase {
    let lead = tr("input_map_list.search_count")
        .replace("{key}", &key.label())
        .replace("{n}", &shown.len().to_string())
        .replace("{total}", &snap.map.len().to_string());
    let parts: Vec<String> = shown
        .iter()
        .filter_map(|id| {
            let name = name_of(&snap.map, *id);
            let rs = readers_of(snap, &name);
            (!rs.is_empty()).then(|| (name, reader_names(rs)))
        })
        .enumerate()
        .map(|(i, (name, who))| {
            let k = if i == 0 {
                "input_map_list.read_by_first"
            } else {
                "input_map_list.read_by_next"
            };
            tr(k).replace("{action}", &name).replace("{who}", &who)
        })
        .collect();
    let rest = if parts.is_empty() {
        String::new()
    } else {
        format!(
            "{}{}",
            parts.join(tr("input_map_list.comma")),
            tr("input_map_list.period")
        )
    };
    Phrase {
        lead,
        rest,
        warn: false,
    }
}

/// «Listening for reload. The next key goes to this row.»
pub(crate) fn listening(snap: &Snapshot, id: ActionId) -> Phrase {
    Phrase {
        lead: tr("input_map_list.listening").replace("{action}", &name_of(&snap.map, id)),
        rest: tr("input_map_list.listening_rest").to_owned(),
        warn: false,
    }
}
