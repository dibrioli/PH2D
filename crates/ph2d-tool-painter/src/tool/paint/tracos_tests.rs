//! O censo dos campos do traço (W16): cada campo do `PaintState` é do traço ou partilhado — um novo, sem
//! classificação, reprova aqui (seria escrito por duas pinceladas sobrepostas, calado).

use super::{DO_TRACO, PARTILHADO};

/// Mutações que sangram: tirar um nome de qualquer das listas · pô-lo nas duas.
#[test]
fn cada_campo_do_paint_state_e_do_traco_ou_partilhado() {
    let src = include_str!("state.rs");
    let ini = src
        .find("pub(crate) struct PaintState {")
        .expect("a struct existe");
    let fim = ini + src[ini..].find("\n}\n").expect("e fecha");
    let campos: Vec<&str> = src[ini..fim]
        .lines()
        .filter_map(|l| l.strip_prefix("    pub(super) "))
        .filter_map(|l| l.split(':').next())
        .collect();
    assert!(campos.len() > 100, "a varredura viu a struct: {}", campos.len());
    for c in &campos {
        let (t, p) = (DO_TRACO.contains(c), PARTILHADO.contains(c));
        assert!(t ^ p, "`{c}`: do traço {t}, partilhado {p} — classifique-o em `tracos.rs`");
    }
    for c in DO_TRACO.iter().chain(PARTILHADO) {
        assert!(campos.contains(c), "`{c}` já não é campo do `PaintState`");
    }
}
