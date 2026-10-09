use super::*;

/// Uma fila de undo de brinquedo: `open` desfaz o topo se ele for o passo da cova.
fn fila_desfaz(
    fila: &mut Vec<&'static str>,
    doc: &mut &'static str,
) -> impl FnMut(&'static str) -> bool {
    move |antes| {
        if fila.last() == Some(&antes) {
            *doc = fila.pop().unwrap();
            true
        } else {
            false
        }
    }
}

/// Apagar A, apagar B, desfazer B, desfazer A: cada cova abre na sua vez, pela ordem do undo.
#[test]
fn as_covas_abrem_pela_ordem_do_undo() {
    let (mut fila, mut doc) = (vec!["A+B", "B"], "");
    let mut g = Graveyard::default();
    g.bury(vec![1], "A+B");
    g.bury(vec![2], "B");
    assert!(g.rise(|id| id == 2, fila_desfaz(&mut fila, &mut doc)));
    assert_eq!(doc, "B");
    assert!(
        !g.rise(|id| id == 2, fila_desfaz(&mut fila, &mut doc)),
        "a de B já abriu"
    );
    assert!(g.rise(|_| true, fila_desfaz(&mut fila, &mut doc)));
    assert_eq!((doc, fila.len()), ("A+B", 0));
    assert!(g.is_empty());
}

/// ⛔ Editado depois da purga (outro passo no topo) ⇒ a cova fecha-se sem desfazer nada.
#[test]
fn um_passo_por_cima_fecha_a_cova() {
    let (mut fila, mut doc) = (vec!["com A", "sem A"], "sem A, editado");
    let mut g = Graveyard::default();
    g.bury(vec![1], "com A");
    assert!(!g.rise(|_| true, fila_desfaz(&mut fila, &mut doc)));
    assert_eq!(doc, "sem A, editado");
    assert!(g.is_empty());
}

/// Os dois donos de uma purga voltam juntos; enquanto falta um, a cova fica fechada.
#[test]
fn a_cova_espera_por_todos_os_donos() {
    let (mut fila, mut doc) = (vec!["com A e B"], "");
    let mut g = Graveyard::default();
    g.bury(vec![1, 2], "com A e B");
    assert!(!g.rise(|id| id == 1, fila_desfaz(&mut fila, &mut doc)));
    assert!(!g.is_empty());
    assert!(g.rise(|_| true, fila_desfaz(&mut fila, &mut doc)));
}

/// Um único undo global que devolve dois objectos apagados em passos seguidos abre as duas covas.
#[test]
fn duas_covas_abrem_no_mesmo_quadro() {
    let (mut fila, mut doc) = (vec!["AB", "B"], "-");
    let mut g = Graveyard::default();
    g.bury(vec![1], "AB");
    g.bury(vec![2], "B");
    assert!(g.rise(|_| true, fila_desfaz(&mut fila, &mut doc)));
    assert_eq!(doc, "AB");
}

/// Sem identidade não há cova; a profundidade é a do undo global.
#[test]
fn sem_id_nao_ha_cova_e_a_fundura_tem_tecto() {
    let mut g = Graveyard::default();
    g.bury(Vec::new(), 1);
    assert!(g.is_empty());
    for i in 0..=GRAVEYARD_DEPTH as u64 {
        g.bury(vec![i + 1], i);
    }
    assert_eq!(g.graves.len(), GRAVEYARD_DEPTH);
}
