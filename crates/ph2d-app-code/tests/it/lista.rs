//! ⭐ **E11 — a lista de scripts do projeto**, pelo rato real da aba: filtrar, abrir, criar, renomear,
//! apagar — e apagar um script EM USO é recusado com a frase no ecrã (na própria lista).

use crate::{Mesa, centro};
use ph2d_editor_core::doc_surface::Key;
use ph2d_host::Modifiers;
use ph2d_panel_code::{ListField, ListHit};
use ph2d_script::{ProjectScripts, ScriptId};

fn nomes(m: &Mesa) -> Vec<String> {
    m.sim
        .world()
        .resource::<ProjectScripts>()
        .iter()
        .map(|(_, d)| d.name.clone())
        .collect()
}

/// Clica no alvo `h` da lista (pintada no último quadro) com a aba `id` activa.
fn clica(m: &mut Mesa, id: ScriptId, h: ListHit) {
    m.pinta(id);
    let r = m
        .docs()
        .list_target(h)
        .unwrap_or_else(|| panic!("{h:?} não foi pintado"));
    let (x, y) = centro(r);
    m.clica(id, x, y, Modifiers::default());
    m.quadro();
    m.pinta(id);
}

#[test]
fn a_lista_filtra_abre_cria_renomeia_e_apaga_o_que_nao_esta_em_uso() {
    let (m, ids) = Mesa::new(&[("bob.luau", "-- bob"), ("ana.luau", "-- ana")]);
    let mut m = m.com_cena(ids[0], 2);
    m.pede(ids[0]);
    m.pinta(ids[0]);
    assert_eq!(m.docs().list_names(), ["bob.luau", "ana.luau"]);

    // O filtro (sem maiúsculas).
    clica(&mut m, ids[0], ListHit::Field(ListField::Filter, 0));
    for c in "AN".chars() {
        m.tecla(ids[0], Key::Char(c), Modifiers::default(), Some(&c.to_string()));
    }
    assert_eq!(m.docs().list_names(), ["ana.luau"]);
    // Enter abre o primeiro que passa.
    m.tecla(ids[0], Key::Enter, Modifiers::default(), None);
    m.quadro();
    assert_eq!(m.docs().open_scripts(), [ids[0], ids[1]]);
    assert_eq!(m.hero.documents.surface_active().map(|t| t.id), Some(ids[1].0));
    // Esvazia o filtro: as duas voltam.
    clica(&mut m, ids[1], ListHit::Field(ListField::Filter, 2));
    for _ in 0..2 {
        m.tecla(ids[1], Key::Backspace, Modifiers::default(), None);
    }
    m.pinta(ids[1]);
    assert_eq!(m.docs().list_names().len(), 2);

    // Clicar numa linha abre (activa) esse script.
    clica(&mut m, ids[1], ListHit::Row(ids[0].0));
    assert_eq!(m.hero.documents.surface_active().map(|t| t.id), Some(ids[0].0));

    // «New script»: nome único, aberto na hora.
    clica(&mut m, ids[0], ListHit::New);
    assert_eq!(nomes(&m), ["bob.luau", "ana.luau", "script.luau"]);
    clica(&mut m, ids[0], ListHit::New);
    assert_eq!(nomes(&m)[3], "script 2.luau", "o segundo ganha nome único");
    let novo = m.docs().open_scripts()[3];
    assert_eq!(m.hero.documents.surface_active().map(|t| t.id), Some(novo.0));

    // Renomear: duplo clique na linha, escrever, Enter. A aba muda de título.
    let (x, y) = centro(m.docs().list_target(ListHit::Row(novo.0)).expect("linha"));
    m.clica(novo, x, y, Modifiers::default());
    m.clica(novo, x, y, Modifiers::default());
    m.pinta(novo);
    for c in "luz.luau".chars() {
        m.tecla(novo, Key::Char(c), Modifiers::default(), Some(&c.to_string()));
    }
    m.tecla(novo, Key::Enter, Modifiers::default(), None);
    m.quadro();
    assert_eq!(nomes(&m)[3], "luz.luau");
    use ph2d_editor_core::doc_surface::DocumentSurface;
    let titulo = m.docs().tabs().into_iter().find(|t| t.id == novo.0).map(|t| t.title);
    assert_eq!(titulo.as_deref(), Some("luz.luau"));

    // Apagar sem uso: sai da biblioteca e a aba fecha.
    m.pinta(novo);
    clica(&mut m, novo, ListHit::Delete(novo.0));
    assert_eq!(nomes(&m).len(), 3);
    assert!(!m.docs().open_scripts().contains(&novo));
    assert_eq!(m.docs().list_notice(), None);
}

/// ⭐⭐ **Apagar um script EM USO é recusado com a frase NA LISTA** (nunca só no terminal), e o script
/// fica. Controlo: o mesmo gesto num script sem uso apaga.
#[test]
fn apagar_um_script_em_uso_e_recusado_com_frase_no_ecra() {
    let (m, ids) = Mesa::new(&[("bob.luau", "-- bob"), ("solto.luau", "-- solto")]);
    let mut m = m.com_cena(ids[0], 3);
    m.pede(ids[0]);
    m.pinta(ids[0]);
    clica(&mut m, ids[0], ListHit::Delete(ids[0].0));
    assert_eq!(nomes(&m), ["bob.luau", "solto.luau"], "em uso: fica");
    let frase = m.docs().list_notice().expect("a recusa diz-se no ecrã").to_owned();
    assert!(frase.contains("bob.luau") && frase.contains('3'), "{frase}");
    assert_ne!(ph2d_i18n::tr("code.list.in_use"), "code.list.in_use");
    // Controlo: sem uso, apaga.
    m.pede(ids[1]);
    m.pinta(ids[1]);
    clica(&mut m, ids[1], ListHit::Delete(ids[1].0));
    assert_eq!(nomes(&m), ["bob.luau"]);
}

/// A lista recolhe e volta: recolhida, o texto ganha a largura dela.
#[test]
fn a_lista_recolhe_e_volta() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", "-- bob")]);
    m.pede(ids[0]);
    m.pinta(ids[0]);
    assert!(m.docs().list_open());
    assert!(m.docs().list_target(ListHit::New).is_some());
    clica(&mut m, ids[0], ListHit::Toggle);
    assert!(!m.docs().list_open());
    assert!(m.docs().list_target(ListHit::New).is_none(), "recolhida só tem o botão de voltar");
    clica(&mut m, ids[0], ListHit::Toggle);
    assert!(m.docs().list_open());
}
