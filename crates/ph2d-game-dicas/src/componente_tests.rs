//! Os gates dos dados do Add Component (plano 25 §4).

use super::*;
use ph2d_component_desc::Attach;
use ph2d_editor_core::widget::command_palette::{PaletteItem, item_matches};

fn authored() -> impl Iterator<Item = &'static ph2d_component_desc::ComponentDesc> {
    ph2d_component_desc::all().filter(|d| matches!(d.attach, Attach::Authored { .. }))
}

fn resolve(k: TextKey) -> bool {
    k.tr() != k.key()
}

#[test]
fn todo_componente_oferecido_tem_grupo_posicao_e_dica_com_desenho() {
    for d in authored() {
        assert!(
            entrada(d.canonical_name).is_some(),
            "{} sem entrada",
            d.canonical_name
        );
    }
    for (i, e) in ENTRADAS.iter().enumerate() {
        assert!(
            e.name == ANIMATION || authored().any(|d| d.canonical_name == e.name),
            "{} não é componente oferecível",
            e.name
        );
        assert!(
            !ENTRADAS[..i].iter().any(|o| o.name == e.name),
            "{} em dobro",
            e.name
        );
        assert!(!e.picture.is_empty(), "{} sem desenho", e.name);
        assert!(resolve(e.tip), "{} sem frase", e.name);
    }
    assert_eq!(ENTRADAS.len(), authored().count() + 1);
}

#[test]
fn nenhum_assunto_tem_uma_entrada_so() {
    for a in Assunto::ALL {
        let n = ENTRADAS.iter().filter(|e| e.assunto == a).count();
        assert!(n >= 2, "{a:?} tem {n}");
        assert!(
            a.title() != "" && !a.title().starts_with("component_add."),
            "{a:?}"
        );
    }
    // Contíguos e na ordem de `Assunto::ALL`.
    let ordem: Vec<usize> = ENTRADAS
        .iter()
        .map(|e| {
            Assunto::ALL
                .iter()
                .position(|a| *a == e.assunto)
                .unwrap_or(usize::MAX)
        })
        .collect();
    assert!(ordem.windows(2).all(|w| w[0] <= w[1]), "{ordem:?}");
}

#[test]
fn toda_dependencia_tem_o_seu_porque() {
    let mut arestas = 0;
    for d in ph2d_component_desc::all() {
        for r in d.requires {
            if matches!(d.attach, Attach::Authored { .. }) {
                arestas += 1;
                let k = porque(d.canonical_name, r);
                assert!(
                    k.is_some_and(resolve),
                    "{} → {r} sem porquê",
                    d.canonical_name
                );
            }
        }
    }
    for (a, b, _) in PORQUE {
        assert!(
            ph2d_component_desc::desc_for(a).is_some_and(|d| d.requires.contains(b)),
            "{a} → {b} não é aresta"
        );
    }
    assert_eq!(PORQUE.len(), arestas);
}

#[test]
fn cada_ensino_do_add_component_so_cita_o_que_existe_e_so_aparece_sem_resultado() {
    let items: Vec<PaletteItem> = authored()
        .map(|d| {
            let kw = d.keywords_key();
            let words = ph2d_i18n::tr(&kw);
            PaletteItem {
                badge: None,
                label: ph2d_i18n::tr(d.display_key).to_string(),
                id: ph2d_editor_core::NodeId(0),
                keywords: if words == kw { "" } else { words },
            }
        })
        .collect();
    for (text, words, start) in TEACH {
        assert!(resolve(*text) && resolve(*words), "{}", text.key());
        assert!(entrada(start).is_some(), "{start}");
        for w in words.tr().split(',') {
            let w = w.trim();
            assert!(!w.is_empty());
            let hit = items.iter().find(|it| item_matches(w, it));
            assert!(hit.is_none(), "«{w}» já acha {:?}", hit.map(|h| &h.label));
        }
    }
}
