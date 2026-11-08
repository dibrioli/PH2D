//! Item 16 (CodeEditor 13 §4, Onda C): «Make it a property» — o Alt+Enter sobre um número no corpo de
//! uma função com `self` declara a `ph2d.property` no topo, troca o número por `self.<nome>` num passo de
//! undo e abre o F2 sobre o nome. Nunca no topo nem dentro de uma `ph2d.property(…)`.

use crate::acoes::{alt_enter, sem_assistente, texto};
use crate::assistente::tecla;
use crate::completar::{T0, quadro_em};
use crate::definicao::clica_em;
use ph2d_editor_core::doc_surface::Key;
use ph2d_host::Modifiers;

const FIX: &str = "Make it a property (Inspector field)";

const TEXTO: &str = "-- door\n\
ph2d.property(\"amplitude\", 1.0)\n\
function update(self, dt)\n  \
local x = self.amplitude\n  \
ph2d.set(self.id, \"x\", x + 220 * dt)\n\
end\n";

fn titulos_em(src: &str, agulha: &str) -> Vec<String> {
    let (mut m, id) = sem_assistente(src);
    clica_em(
        &mut m,
        id,
        src.find(agulha).unwrap() + 1,
        Modifiers::default(),
    );
    alt_enter(&mut m, id);
    quadro_em(&mut m, T0);
    m.docs().actions_of(id).expect("abriu").0
}

#[test]
fn alt_enter_no_220_faz_dele_uma_propriedade_num_passo_e_abre_o_f2() {
    let (mut m, id) = sem_assistente(TEXTO);
    clica_em(
        &mut m,
        id,
        TEXTO.find("220").unwrap() + 1,
        Modifiers::default(),
    );
    alt_enter(&mut m, id);
    quadro_em(&mut m, T0);
    let (titulos, _, _) = m.docs().actions_of(id).expect("abriu");
    let i = titulos.iter().position(|t| t == FIX).expect("oferecida");
    for _ in 0..i {
        tecla(&mut m, id, Key::Down, Modifiers::default(), None, "");
    }
    tecla(&mut m, id, Key::Enter, Modifiers::default(), None, "");
    let depois = TEXTO
        .replace("1.0)\n", "1.0)\nph2d.property(\"value\", 220)\n")
        .replace("x + 220", "x + self.value");
    assert_eq!(texto(&mut m, id), depois);
    let (campo, _, razao) = m.docs().rename_of(id).expect("o F2 abriu sobre o nome");
    assert_eq!(campo.as_deref(), Some("value"));
    assert_eq!(razao, None);
    tecla(&mut m, id, Key::Escape, Modifiers::default(), None, "");
    assert_eq!(m.docs().rename_of(id), None);
    tecla(&mut m, id, Key::Char('z'), crate::ctrl(), None, "");
    assert_eq!(
        texto(&mut m, id),
        TEXTO,
        "um Ctrl+Z desfaz o conserto inteiro"
    );
}

#[test]
fn nao_se_oferece_no_topo_nem_dentro_de_ph2d_property() {
    let src = "local base = 220\nph2d.property(\"speed\", 330)\nfunction update(self, dt)\nend\n";
    assert!(!titulos_em(src, "220").iter().any(|t| t == FIX));
    assert!(!titulos_em(src, "330").iter().any(|t| t == FIX));
    let sem_self = "function f(a)\n  return a + 220\nend\n";
    assert!(!titulos_em(sem_self, "220").iter().any(|t| t == FIX));
}

/// A prévia do «Make it a property» mostra só as duas linhas que mudam (a declaração nova e a do
/// número), com um «…» no lugar das que ficam iguais entre elas.
#[test]
fn a_previa_mostra_so_as_duas_linhas_que_mudam() {
    let (mut m, id) = sem_assistente(TEXTO);
    clica_em(
        &mut m,
        id,
        TEXTO.find("220").unwrap() + 1,
        Modifiers::default(),
    );
    alt_enter(&mut m, id);
    quadro_em(&mut m, T0);
    let (titulos, _, _) = m.docs().actions_of(id).expect("abriu");
    let i = titulos.iter().position(|t| t == FIX).expect("oferecida");
    for _ in 0..i {
        tecla(&mut m, id, Key::Down, Modifiers::default(), None, "");
    }
    let (_, sel, previa) = m.docs().actions_of(id).expect("aberta");
    assert_eq!(sel, i);
    assert_eq!(
        previa,
        [
            "ph2d.property(\"value\", 220)",
            "\u{2026}",
            "  ph2d.set(self.id, \"x\", x + self.value * dt)",
        ]
    );
}
