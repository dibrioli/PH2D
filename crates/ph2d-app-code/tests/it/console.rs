//! ⭐ **E10 — o console da aba**: o `print` chega à aba do script que o disse (e não à do vizinho); um
//! erro de runtime abre o console sozinho e a linha dele leva o cursor ao sítio.

use crate::{Mesa, centro};
use ph2d_host::Modifiers;

const OI: &str = "function update(self, dt)\n  print(\"oi\")\nend\n";
const TCHAU: &str = "function update(self, dt)\n  print(\"tchau\")\nend\n";
const PARTE: &str =
    "-- parte no 1.º tique\n\nfunction update(self, dt)\n  local t = nil\n  t.x = 1\nend\n";

#[test]
fn print_oi_chega_ao_console_da_aba_certa() {
    let (m, ids) = Mesa::new(&[("oi.luau", OI), ("tchau.luau", TCHAU)]);
    let mut m = m.com_cena(ids[0], 1).com_cena(ids[1], 1);
    m.pede(ids[0]);
    m.pede(ids[1]);
    m.cena();
    m.quadro();
    assert_eq!(m.docs().console_texts(ids[0]), ["oi"]);
    assert_eq!(
        m.docs().console_texts(ids[1]),
        ["tchau"],
        "cada aba filtra pelo seu script"
    );
    assert!(
        !m.docs().console_open(ids[0]),
        "um print não abre o console"
    );
}

#[test]
fn um_erro_de_runtime_abre_o_console_e_a_linha_leva_o_cursor() {
    let (m, ids) = Mesa::new(&[("parte.luau", PARTE)]);
    let mut m = m.com_cena(ids[0], 1);
    m.pede(ids[0]);
    m.cena();
    m.quadro();
    assert!(
        m.docs().console_open(ids[0]),
        "o erro de runtime abre o console sozinho"
    );
    let linhas = m.docs().console_texts(ids[0]);
    assert_eq!(linhas.len(), 1, "{linhas:?}");
    assert!(
        linhas[0].contains('5'),
        "a linha do erro (5) vem na frase: {linhas:?}"
    );
    m.pinta(ids[0]);
    let r = m
        .docs()
        .console_row_rect(ids[0], 0)
        .expect("a linha foi pintada");
    let (x, y) = centro(r);
    m.clica(ids[0], x, y, Modifiers::default());
    assert_eq!(
        m.docs().cursor_of(ids[0]).map(|c| c.0),
        Some(4),
        "o clique leva o cursor à linha 5"
    );
}

/// ⛔ O registo por aba tem o MESMO tecto em bytes que o do motor (item 8 da auditoria): sem ele,
/// o console de UMA aba guardava tudo o que o motor já tinha deitado fora por quadro.
#[test]
fn o_console_da_aba_tem_tecto_em_bytes() {
    use ph2d_script::{CONSOLE_BYTES_MAX, CONSOLE_LINE_BYTES_MAX};
    let longo = "function update(self, dt)\n  print(string.rep(\"x\", 9000))\nend\n";
    let (m, ids) = Mesa::new(&[("longo.luau", longo)]);
    let mut m = m.com_cena(ids[0], 1_000);
    m.pede(ids[0]);
    for _ in 0..5 {
        m.cena();
        m.quadro();
    }
    let linhas = m.docs().console_texts(ids[0]);
    let total: usize = linhas.iter().map(String::len).sum();
    assert!(
        linhas
            .iter()
            .all(|l| l.len() <= CONSOLE_LINE_BYTES_MAX + '…'.len_utf8())
    );
    assert!(total <= CONSOLE_BYTES_MAX, "a aba guardou {total} bytes");
    assert!(
        linhas.len() < 5_000,
        "controlo: o tecto em bytes deitou linhas fora"
    );
}
