//! J2 — **o verso liga o cartão ao projecto** pela costura do hero: «Scene object» lista o catálogo
//! (com pesquisa), escolher liga, a secção «Project links» abre a ligação pelo dono dela e tira-a
//! (um passo de desfazer cada); e «Convert to sticky note» com o Kanban num Quadro.

use crate::Mesa;
use crate::duas_janelas::Duas;
use crate::j2_auto::toca_v;
use crate::verso::{abre, toca};
use ph2d_editor_core::kanban_links::{self, Family};
use ph2d_kanban_links::Catalog;
use ph2d_kanban_model::{CardId, KanbanId, Target};

fn catalogo() -> Catalog {
    let mut c = Catalog::default();
    c.objects.insert(7, "Hero".into());
    c.scripts.insert(3, "jump.luau".into());
    c.macros.insert("Walk cycle".into());
    c
}

fn ligacoes(m: &Mesa, id: KanbanId, c: CardId) -> Vec<(Target, String)> {
    m.kanban(id)
        .links_of(c)
        .into_iter()
        .map(|l| (l.target.clone(), l.name.clone()))
        .collect()
}

#[test]
fn j2_scene_object_liga_abre_e_tira_uma_ligacao_ao_projecto() {
    kanban_links::publish_scene(catalogo());
    let (mut m, id, c) = abre("Write the logline");
    m.quadro();
    toca_v(&mut m, id, "Side(SceneObject)");
    assert_eq!(m.docs().probe_back_state(id.0).0.as_deref(), Some("Scene"));
    // A pesquisa estreita o catálogo: «jump» deixa só o script.
    m.escreve("jump");
    m.quadro();
    toca(&mut m, id, "Pop(Scene(0))");
    assert_eq!(ligacoes(&m, id, c), [(Target::Script(3), "jump.luau".to_owned())]);
    assert_eq!(m.docs().probe_back_state(id.0).0, None, "escolher fecha o popover");
    // O objecto, pelo popover sem pesquisa (a 1.ª entrada é o objecto da cena).
    toca_v(&mut m, id, "Side(SceneObject)");
    toca(&mut m, id, "Pop(Scene(0))");
    assert_eq!(ligacoes(&m, id, c).len(), 2);
    assert_eq!(ligacoes(&m, id, c)[1], (Target::Object(7), "Hero".to_owned()));
    // Clicar na ligação pede «abrir» ao dono dela (a cena seleciona o objecto).
    let _ = kanban_links::take(Family::Scene);
    let open: Vec<String> = m
        .docs()
        .probe_back_hits(id.0)
        .into_iter()
        .filter(|(n, _)| n.starts_with("LinkOpen("))
        .map(|x| x.0)
        .collect();
    assert_eq!(open.len(), 2, "a secção mostra as duas");
    toca_v(&mut m, id, &open[1]);
    assert_eq!(kanban_links::take(Family::Scene), [Target::Object(7)]);
    // Tirar a ligação; o desfazer devolve-a; três desfazer voltam ao cartão sem ligações.
    toca_v(&mut m, id, "LinkRemove(");
    assert_eq!(ligacoes(&m, id, c).len(), 1);
    assert!(m.desfaz().is_some());
    assert_eq!(ligacoes(&m, id, c).len(), 2, "o tirar desfeito");
    assert!(m.desfaz().is_some());
    assert!(m.desfaz().is_some());
    assert!(ligacoes(&m, id, c).is_empty(), "um passo de desfazer por ligação");
}

#[test]
fn j2_convert_to_sticky_note_com_o_kanban_num_quadro() {
    let (mut d, id, _) = Duas::new();
    let c = d.hero.documents.kanbans().get(id).unwrap().cards.values().find(|c| c.title == "Write the logline").unwrap().id;
    d.docs().open(id);
    d.quadros(2);
    d.docs().open_back(id.0, c);
    d.quadros(2);
    let hits = d.docs().probe_back_hits(id.0);
    let r = hits
        .iter()
        .rev()
        .find(|(n, _)| n == "Side(ToNote)")
        .map(|x| x.1)
        .expect("«Convert to sticky note» com o Kanban posto num Quadro");
    let _ = r;
    // Sem Quadro o verso não o oferece.
    let (mut m, id2, _) = abre("Write the logline");
    assert!(
        !m.docs()
            .probe_back_hits(id2.0)
            .iter()
            .any(|(n, _)| n == "Side(ToNote)")
    );
}
