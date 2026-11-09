use super::*;
use crate::widget::command_palette::top_match;

const A: &[AddEntry] = &[
    AddEntry::new("object_add.flip", AddGroup::Drawing),
    AddEntry::new("object_add.vector.drawing", AddGroup::Drawing),
];
const B: &[AddEntry] = &[AddEntry::new("object_add.game.camera", AddGroup::Game)];

/// ⭐ **Cada entrada compilada tem UM item, e o id volta à mesma entrada.**
///
/// (Mutação: `entry_of_pick` a devolver a primeira entrada ⇒ RED.)
#[test]
fn every_entry_has_one_item_and_its_id_maps_back() {
    let fams: &[&[AddEntry]] = &[CORE, A, B];
    let model = build(fams, &|_| None, None);
    let all: Vec<AddEntry> = fams.iter().flat_map(|f| f.iter().copied()).collect();
    assert_eq!(model.item_count(), all.len());
    for e in &all {
        assert!(model.is_item(e.id()), "{} sem item", e.key.key());
        assert_eq!(entry_of_pick(fams, e.id()), Some(*e));
    }
    let mut ids: Vec<_> = all.iter().map(|e| e.id()).collect();
    ids.sort_by_key(|id| id.0);
    ids.dedup();
    assert_eq!(ids.len(), all.len(), "dois itens com o mesmo id");
}

/// **Uma família que não está compilada não deixa grupo** — o grupo vazio não é pintado.
#[test]
fn a_group_without_entries_is_not_shown() {
    let model = build(&[CORE], &|_| None, None);
    let titles: Vec<&str> = model.groups.iter().map(|g| g.title.as_str()).collect();
    assert_eq!(
        titles,
        [AddGroup::Pictures.title(), AddGroup::Empty.title()],
        "sem a família de Jogo, o grupo dela não pode aparecer"
    );
    assert_eq!(entry_of_pick(&[CORE], A[0].id()), None);
}

/// ⭐ **A busca acha cada tipo pelo NOME do tipo** — o rótulo do item leva-o, porque a busca da
/// paleta só lê o rótulo.
#[test]
fn the_search_finds_every_family_by_its_type_name() {
    let model = build(&[CORE, A, B], &|_| None, None);
    assert_eq!(top_match(&model, "flip"), Some(A[0].id()));
    assert_eq!(top_match(&model, "image"), Some(IMAGE.id()));
    for e in [CORE, A, B].iter().flat_map(|f| f.iter()) {
        assert_eq!(
            top_match(&model, e.key.tr()),
            Some(e.id()),
            "{}",
            e.key.key()
        );
    }
}

/// **Cada rótulo existe na tabela** — um `tr` que devolve a chave pintaria `object_add.…` no ecrã.
#[test]
fn every_label_is_translated() {
    for e in CORE.iter().chain(A).chain(B) {
        assert_ne!(e.key.tr(), e.key.key());
    }
    for g in AddGroup::ALL {
        assert!(!g.title().starts_with("object_add."));
    }
}

/// ⭐ **O que não pode nascer agora APARECE, depois do que pode, com a razão no rótulo** — e o id
/// continua a ser o da entrada, para o clique responder.
///
/// (Mutação: largar o `blocked` ⇒ RED; pô-lo antes do `ready` ⇒ RED.)
#[test]
fn a_blocked_entry_shows_after_the_ready_ones_with_its_reason() {
    let reason = "already one";
    let model = build(&[A], &|e| (e == A[0]).then_some(reason), None);
    let subs = &model.groups[0].subs;
    assert_eq!(subs.len(), 2);
    assert_eq!(subs[0].items[0].id, A[1].id(), "o que pode vem primeiro");
    assert_eq!(subs[1].items[0].id, A[0].id());
    assert!(subs[1].items[0].label.ends_with(reason));
    assert!(subs[1].items[0].label.starts_with(A[0].key.tr()));
}

const PICTURE: &[crate::widget::illustration::Mark] = &[crate::widget::illustration::Mark::Disc {
    c: [0.5, 0.5],
    r: 0.2,
    fill: ph2d_tokens::ColorToken::Accent,
}];

/// ⭐ **A dica de uma entrada chega ao modelo, pela id do item** (GameEngineFuncional N1): o título é o
/// rótulo, o texto é a chave da explicação traduzida, o desenho é o da entrada — e uma entrada sem dica
/// não deixa nenhuma.
///
/// (Mutação: o `build` a deixar `tips` vazio ⇒ RED.)
#[test]
fn an_entry_with_a_tip_puts_it_in_the_model() {
    const WITH: AddEntry = AddEntry::new("object_add.game.camera", AddGroup::Game)
        .with_tip("object_add.title", PICTURE);
    let fams: &[&[AddEntry]] = &[A, &[WITH]];
    let model = build(fams, &|_| None, None);
    assert_eq!(model.tips.len(), 1, "só a entrada com dica deixa dica");
    let tip = &model.tips[0];
    assert_eq!(tip.id, WITH.id());
    assert_eq!(tip.title, WITH.key.tr());
    assert_eq!(tip.body, ph2d_i18n::tr("object_add.title"));
    assert_eq!(tip.picture, PICTURE);
    assert_ne!(WITH, B[0], "a dica faz parte da entrada");
    assert_eq!(WITH.id(), B[0].id(), "mas não da identidade do item");
}

/// ⭐ **O Recent guarda três também nas pontas que não passam pelo `push`**: o modelo que recebe uma lista maior só
/// pinta três, e o ficheiro de preferências com cinco ids volta com três.
///
/// (Mutação: tirar o `take(ADD_RECENT_MAX)` do `build` ⇒ RED; tirar o `truncate` do `le_recentes` ⇒ RED.)
#[test]
fn o_recent_so_mostra_e_so_le_tres_mesmo_vindo_com_mais() {
    let fams: &[&[AddEntry]] = &[CORE, A, B];
    let cinco: Vec<_> = fams.iter().flat_map(|f| f.iter()).map(|e| e.id()).collect();
    assert_eq!(cinco.len(), 5);
    let select = AddSelect {
        initial: IMAGE,
        teach: &[],
        recent: &cinco,
    };
    let model = build(fams, &|_| None, Some(select));
    let recent: Vec<_> = model.groups[0].subs[0].items.iter().map(|i| i.id).collect();
    assert_eq!(recent, cinco[..3], "o modelo pintou mais de três recentes");

    let linha: Vec<String> = cinco.iter().map(|n| n.0.to_string()).collect();
    let mut store = crate::interaction::WidgetStore::with_capacity(4);
    let text = format!("add_recent={}\n", linha.join(","));
    crate::interaction::restore_section_prefs_text(&mut store, &text);
    assert_eq!(
        store.add_recent(),
        &cinco[..3],
        "o ficheiro deixou passar mais de três"
    );
}
