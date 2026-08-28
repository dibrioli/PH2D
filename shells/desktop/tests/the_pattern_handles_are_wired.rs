//! **AS TRÊS ALÇAS DO PADRÃO ESTÃO FIADAS NOS QUATRO SÍTIOS** (plano 33, W6).
//!
//! ⚠️ **A shell não é alcançável de um teste de unidade** — o `App` segura uma surface de janela
//! real. É a mesma razão pela qual o undo do filtro do sculpt3d, o desenho do offset vivo e o pick
//! do mapa desenhado têm todos um gate que lê o FONTE. Aqui o risco é concreto e já mordeu duas
//! vezes nesta casa: uma alça **pintada e morta sob o ponteiro** dá exactamente o mesmo report que
//! uma alça que nunca foi pintada.
//!
//! Os quatro sítios são independentes:
//!
//! 1. **o PRESS** agarra (sem ele, o desenho é um enfeite);
//! 2. **o MOVE** arrasta (sem ele, agarra e nada acontece);
//! 3. **o RELEASE** fecha o passo de undo (sem ele, um gesto vira N passos ou nenhum);
//! 4. **o DESENHO** mostra (sem ele, o artista tem de adivinhar onde agarrar).

use std::fs;
use std::path::Path;

fn src(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// ⚠️ **Comentários FORA**, e não é higiene: a prosa que explica a lei contém, por construção,
/// exactamente as agulhas que o gate procura — *um gate que lê o comentário sobre a lei em vez do
/// código que a obedece aprova quem a documenta*.
fn code(rel: &str) -> String {
    src(rel)
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_pattern_handles_are_wired_at_all_four_sites() {
    let dispatch = code("input_dispatch.rs");
    for (needle, what) in [
        (
            "fn vec_pattern_hit(",
            "o hit-test (qual alça o cursor acerta)",
        ),
        (
            "self.vec_pattern_drag = Some(i);",
            "o PRESS que agarra a alça",
        ),
        ("if self.vec_pattern_drag_move(", "o MOVE que a arrasta"),
        (
            "if self.vec_pattern_drag.take().is_some() {",
            "o RELEASE que fecha o passo de undo",
        ),
    ] {
        assert!(
            dispatch.contains(needle),
            "{what} saiu do `input_dispatch.rs` - a alca fica pintada e morta sob o ponteiro"
        );
    }
    assert!(
        code("render_loop/mod.rs").contains("pattern_handle::draw_pattern_handles("),
        "o DESENHO das alcas saiu do quadro - o artista tem de adivinhar onde agarrar"
    );
}

/// ⚠️ **UM GESTO É UM PASSO DE UNDO**, e isso exige os DOIS lados: o `begin` no press e o
/// `commit_if_changed` no release. Só o segundo, e o passo nasce do nada; só o primeiro, e o gesto
/// nunca fecha.
#[test]
fn a_handle_drag_opens_and_closes_exactly_one_undo_step() {
    let d = code("input_dispatch.rs");
    let press = d
        .find("self.vec_pattern_drag = Some(i);")
        .expect("o press existe");
    let depois = &d[press..press + 400];
    assert!(
        depois.contains("self.vec_history.begin("),
        "o press da alca de padrao nao ABRE o passo de undo"
    );
    let rel = d
        .find("if self.vec_pattern_drag.take().is_some() {")
        .expect("o release existe");
    assert!(
        d[rel..rel + 400].contains("commit_if_changed("),
        "o release da alca de padrao nao FECHA o passo de undo"
    );
}
