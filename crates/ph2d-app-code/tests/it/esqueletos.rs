//! I12 (Onda 2, `docs/CodeEditor/03_onda2.md`): os esqueletos com buracos — a lei medida no VS Code
//! 1.142.0 (`ferramentas/medicoes_onda2/I9_I12_I14.md` §I12): o 1.º buraco seleccionado, Tab/Shift+Tab,
//! a ponta que estica, o fim no `$0`, Esc e sair acabam, o preenchimento é UM passo de undo, a lista
//! ganha ao Tab. Os ganchos saem da `api_table` — sem assistente também.

use crate::assistente::{Plano, com_falso};
use crate::completar::{T0, abre, escreve, quadro_em, responde};
use crate::{Mesa, ctrl};
use ph2d_editor_core::doc_surface::Key;
use ph2d_host::Modifiers;
use ph2d_script::ScriptId;
use ph2d_script::api_table::HOOKS;

const PAUSA: u128 = 300_000_000;
const UPDATE: &str = "function update(self: ScriptSelf, dt: number)\n  \nend";

fn shift() -> Modifiers {
    Modifiers {
        shift: true,
        ..Modifiers::default()
    }
}

fn tecla(m: &mut Mesa, id: ScriptId, k: Key, mods: Modifiers) {
    let text = match k {
        Key::Char(c) => Some(c.to_string()),
        _ => None,
    };
    m.tecla(id, k, mods, text.as_deref());
}

/// Sem assistente: escreve `s` e a lista abre na pausa só com os esqueletos.
fn abre_sem(m: &mut Mesa, id: ScriptId, s: &str) -> Vec<String> {
    quadro_em(m, T0);
    escreve(m, id, s);
    quadro_em(m, T0 + PAUSA);
    m.docs()
        .completion_of(id)
        .map(|(l, _)| l)
        .unwrap_or_default()
}

fn sel(m: &mut Mesa, id: ScriptId) -> Vec<std::ops::Range<usize>> {
    m.docs().selection_of(id)
}

#[test]
fn o_gancho_entra_com_o_1o_buraco_seleccionado_e_o_tab_anda() {
    let (mut m, ids) = Mesa::new(&[("a.luau", "")]);
    let id = ids[0];
    m.pede(id);
    assert_eq!(abre_sem(&mut m, id, "upd"), ["update"]);
    tecla(&mut m, id, Key::Tab, Modifiers::default());
    assert_eq!(m.docs().text_of(id).as_deref(), Some(UPDATE));
    assert_eq!(sel(&mut m, id), [16..20], "`self` seleccionado");
    assert_eq!(m.docs().snippet_of(id), Some(vec![34..36]), "o `dt` marcado");
    escreve(&mut m, id, "me");
    tecla(&mut m, id, Key::Tab, Modifiers::default());
    assert_eq!(sel(&mut m, id), [32..34], "Tab: o `dt`");
    tecla(&mut m, id, Key::Tab, shift());
    assert_eq!(sel(&mut m, id), [16..18], "Shift+Tab: `me` inteiro");
    tecla(&mut m, id, Key::Tab, Modifiers::default());
    escreve(&mut m, id, "t");
    tecla(&mut m, id, Key::Tab, Modifiers::default());
    assert_eq!(sel(&mut m, id), [45..45], "o `$0`: o corpo, indentado");
    assert_eq!(m.docs().snippet_of(id), None, "a sessão acabou");
    tecla(&mut m, id, Key::Tab, Modifiers::default());
    assert_eq!(
        m.docs().text_of(id).as_deref(),
        Some("function update(me: ScriptSelf, t: number)\n    \nend"),
        "depois do fim, o Tab indenta"
    );
    // Desfazer: a indentação; o preenchimento todo de uma vez (com `self` seleccionado); o modelo.
    tecla(&mut m, id, Key::Char('z'), ctrl());
    tecla(&mut m, id, Key::Char('z'), ctrl());
    assert_eq!(m.docs().text_of(id).as_deref(), Some(UPDATE));
    assert_eq!(sel(&mut m, id), [16..20]);
    tecla(&mut m, id, Key::Char('z'), ctrl());
    assert_eq!(m.docs().text_of(id).as_deref(), Some("upd"));
}

#[test]
fn escrever_na_ponta_do_buraco_fica_dentro_dele() {
    let (mut m, ids) = Mesa::new(&[("a.luau", "")]);
    let id = ids[0];
    m.pede(id);
    abre_sem(&mut m, id, "upd");
    tecla(&mut m, id, Key::Tab, Modifiers::default());
    tecla(&mut m, id, Key::Right, Modifiers::default());
    escreve(&mut m, id, "Q");
    tecla(&mut m, id, Key::Tab, Modifiers::default());
    tecla(&mut m, id, Key::Tab, shift());
    assert_eq!(sel(&mut m, id), [16..21], "`selfQ` inteiro");
    // Apagado, o buraco continua paragem.
    tecla(&mut m, id, Key::Backspace, Modifiers::default());
    tecla(&mut m, id, Key::Tab, Modifiers::default());
    tecla(&mut m, id, Key::Tab, shift());
    assert_eq!(sel(&mut m, id), [16..16]);
}

#[test]
fn esc_e_sair_do_modelo_acabam_a_sessao() {
    let (mut m, ids) = Mesa::new(&[("a.luau", ""), ("b.luau", "local x = 1\n")]);
    let id = ids[0];
    m.pede(id);
    abre_sem(&mut m, id, "upd");
    tecla(&mut m, id, Key::Tab, Modifiers::default());
    tecla(&mut m, id, Key::Escape, Modifiers::default());
    assert_eq!(m.docs().snippet_of(id), None, "Esc");
    let antes = m.docs().text_of(id);
    tecla(&mut m, id, Key::Tab, Modifiers::default());
    assert_ne!(m.docs().text_of(id), antes, "o Tab a seguir escreve");

    // Sair com as setas: no fim do texto de outro script, a linha de cima fica fora do modelo.
    let id = ids[1];
    m.pede(id);
    tecla(&mut m, id, Key::End, ctrl());
    abre_sem(&mut m, id, "upd");
    tecla(&mut m, id, Key::Tab, Modifiers::default());
    assert!(m.docs().snippet_of(id).is_some());
    tecla(&mut m, id, Key::Up, Modifiers::default());
    assert_eq!(m.docs().snippet_of(id), None, "subiu para fora do modelo");
}

#[test]
fn so_os_ganchos_por_declarar_e_depois_de_function_so_o_resto() {
    let (mut m, ids) = Mesa::new(&[("a.luau", "function init(self)\nend\n")]);
    let id = ids[0];
    m.pede(id);
    tecla(&mut m, id, Key::End, ctrl());
    let l = abre_sem(&mut m, id, "in");
    assert!(!l.contains(&"init".to_owned()), "o `init` já existe: {l:?}");
    for _ in 0..2 {
        tecla(&mut m, id, Key::Backspace, Modifiers::default());
    }
    assert_eq!(abre_sem(&mut m, id, "function up"), ["update"]);
    tecla(&mut m, id, Key::Tab, Modifiers::default());
    assert_eq!(
        m.docs().text_of(id).as_deref(),
        Some(&*format!("function init(self)\nend\n{UPDATE}")),
        "o `function ` escrito fica, sem repetir"
    );
    // Dentro do corpo de uma função não há esqueletos (os ganchos são do topo).
    tecla(&mut m, id, Key::Escape, Modifiers::default());
    tecla(&mut m, id, Key::Home, ctrl());
    tecla(&mut m, id, Key::End, Modifiers::default());
    tecla(&mut m, id, Key::Enter, Modifiers::default());
    assert_eq!(abre_sem(&mut m, id, "on_"), Vec::<String>::new());
}

/// ⛔ Gate da tabela: cada gancho de `api_table::HOOKS` tem esqueleto (um gancho novo = um esqueleto
/// novo, sem código à mão), com um buraco por parâmetro e o tipo escrito.
#[test]
fn cada_gancho_da_tabela_tem_esqueleto() {
    for h in HOOKS {
        let (mut m, ids) = Mesa::new(&[("a.luau", "")]);
        let id = ids[0];
        m.pede(id);
        let l = abre_sem(&mut m, id, &h.ident[..3]);
        assert!(l.contains(&h.ident.to_owned()), "{}: {l:?}", h.ident);
        let i = l.iter().position(|x| x == h.ident).unwrap();
        for _ in 0..i {
            tecla(&mut m, id, Key::Down, Modifiers::default());
        }
        tecla(&mut m, id, Key::Tab, Modifiers::default());
        let t = m.docs().text_of(id).unwrap();
        let sig = h.luau_type.split(" -> ").next().unwrap();
        assert_eq!(t, format!("function {}{sig}\n  \nend", h.ident), "{}", h.ident);
        let primeiro = sig.trim_start_matches('(').split(':').next().unwrap();
        let s = sel(&mut m, id)[0].clone();
        assert_eq!(&t[s], primeiro, "{}: o 1.º parâmetro seleccionado", h.ident);
    }
}

#[test]
fn a_propriedade_do_topo_e_a_chamada_da_api_poem_o_texto_entre_aspas() {
    let (mut m, ids) = Mesa::new(&[("a.luau", "")]);
    let id = ids[0];
    m.pede(id);
    assert_eq!(abre_sem(&mut m, id, "prop"), ["ph2d.property"]);
    tecla(&mut m, id, Key::Tab, Modifiers::default());
    assert_eq!(
        m.docs().text_of(id).as_deref(),
        Some("ph2d.property(\"name\", default)")
    );
    assert_eq!(sel(&mut m, id), [15..19], "o nome, dentro das aspas");
    escreve(&mut m, id, "altura");
    tecla(&mut m, id, Key::Tab, Modifiers::default());
    escreve(&mut m, id, "1");
    assert_eq!(
        m.docs().text_of(id).as_deref(),
        Some("ph2d.property(\"altura\", 1)")
    );

    // Pelo servidor: a chamada `ph2d.set` leva o esqueleto da tabela (o campo entre aspas).
    let (mut m, ids, _) = com_falso(
        &[("b.luau", "")],
        Plano {
            completar: Some(responde),
            ..Plano::default()
        },
    );
    let id = ids[0];
    m.pede(id);
    abre(&mut m, id, "ph2d.se");
    tecla(&mut m, id, Key::Tab, Modifiers::default());
    assert_eq!(
        m.docs().text_of(id).as_deref(),
        Some("ph2d.set(entity, \"x\", value)")
    );
    assert_eq!(sel(&mut m, id), [9..15]);
}

#[test]
fn com_a_lista_aberta_o_tab_e_da_lista() {
    let (mut m, ids, _) = com_falso(
        &[("a.luau", "")],
        Plano {
            completar: Some(responde),
            ..Plano::default()
        },
    );
    let id = ids[0];
    m.pede(id);
    abre(&mut m, id, "upd");
    tecla(&mut m, id, Key::Tab, Modifiers::default());
    tecla(&mut m, id, Key::Tab, Modifiers::default());
    assert_eq!(sel(&mut m, id), [34..36], "no `dt`");
    // Escrito no buraco, a lista abre na pausa: o Tab aceita a sugestão em vez de saltar.
    let l = abre(&mut m, id, "pai");
    assert_eq!(l[0], "pairs");
    tecla(&mut m, id, Key::Tab, Modifiers::default());
    assert_eq!(
        m.docs().text_of(id).as_deref(),
        Some("function update(self: ScriptSelf, pairs: number)\n  \nend")
    );
}
