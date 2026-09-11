//! **A âncora de escala é do FRAME, não do pen-down** — arch-gate sobre a costura que nenhum
//! unit test alcança (irmão de `a_frames_handle_resizes_it_and_does_not_scale_it`).
//!
//! A LEI é gateada onde ela mora: `ph2d_editor::live_anchor` é pura e tem os seus gates de
//! kernel (o centro é a translação de mundo da partida · soltar devolve o canto · premir e largar
//! não acumula · Translate/Rotate/MovePivot passam · Global troca a regra e não o ponto). O que
//! eles **não podem tocar** é a fiação: `advance_gizmo_drag` e o abridor de arrasto precisam de
//! `App` + `HeroScreen` + janela. E é lá que vivem as três metades que decidem se o modificador é
//! de facto vivo:
//!
//! 1. **A porta é chamada por movimento.** Sem ela a âncora volta a ser decidida no pen-down e
//!    congelada — o defeito: o `Shift` vivo e o Ctrl morto, no mesmo gizmo.
//! 2. **O resultado é LOCAL — o estado guardado continua a ser o CANTO.** Esta é a metade cara.
//!    O canto é a única coisa que não se re-deriva depois do pen-down (só ali se sabe qual alça
//!    foi pega); guardar o centro por cima dele apaga-o no primeiro frame com a tecla premida, e
//!    soltá-la deixa de devolver o que quer que seja. Um gate que só afirmasse (1) ficaria VERDE
//!    sobre isso — **e ficou**: foi o gate de kernel do retorno que apanhou esta versão minha.
//! 3. **O pen-down guarda o canto, sempre.** Se ele guardar o centro quando a tecla já está
//!    premida, (2) fica sem o que devolver: o gesto nasce sem canto.
//!
//! ⚠️ Nada aqui afirma distância em bytes nem vizinhança de linhas — a lição de
//! `the_dispatch_is_handed_the_live_geometry` (2026-07-23) é que um proxy posicional expira na
//! wave seguinte. O que se afirma é *qual pergunta é feita* e *onde a resposta pousa*.

use std::fs;

fn drag_src() -> String {
    fs::read_to_string("src/input_dispatch/gizmo_drag.rs").expect("gizmo_drag.rs")
}

fn open_src() -> String {
    fs::read_to_string("src/input_dispatch.rs").expect("input_dispatch.rs")
}

/// **(1) O avanço do arrasto pergunta a âncora deste frame.**
#[test]
fn the_drag_asks_for_this_frames_anchor() {
    let src = drag_src();
    assert!(
        src.contains("ph2d_editor::live_anchor("),
        "o avanco do arrasto nao chama `live_anchor` — a ancora volta a ser decidida no pen-down \
         e congelada, e o Ctrl deixa de ser vivo enquanto o Shift continua a ser"
    );
}

/// **(2) O estado guardado é o AUTORADO; a âncora derivada é local.**
///
/// A ordem das duas linhas é o invariante: `hero.gizmo.drag = Some(drag)` **antes** da derivação.
/// Depois dela, o que seria guardado é o centro — e o canto morre.
#[test]
fn the_derived_anchor_is_never_written_back_over_the_authored_one() {
    let src = drag_src();
    let store = src
        .find("hero.gizmo.drag = Some(drag);")
        .expect("o avanco do arrasto nao guarda o estado do gesto");
    let derive = src
        .find("ph2d_editor::live_anchor(")
        .expect("o avanco do arrasto nao deriva a ancora deste frame");
    assert!(
        store < derive,
        "a ancora DERIVADA e' guardada por cima do estado autorado: o canto — a unica coisa que \
         nao se re-deriva depois do pen-down — morre no primeiro frame com a tecla premida, e \
         solta-la deixa de devolver coisa nenhuma"
    );
    // E a derivação pousa num binding local, não no campo do arrasto.
    let tail = &src[derive.saturating_sub(64)..derive];
    assert!(
        tail.contains("let drag ="),
        "a saida de `live_anchor` nao pousa num binding local:\n...{tail}"
    );
}

/// **(3) O pen-down guarda o CANTO, com a tecla premida ou não.**
///
/// ⚠️ O oráculo é o argumento que o abridor passa ao `anchor_pivot_world` — `false` literal —,
/// e não a ausência de um nome: uma variável chamada `use_center_anchor` podia lá estar valendo
/// `false`, e o gate ficaria a medir vocabulário em vez de comportamento.
#[test]
fn the_pen_down_stores_the_corner_so_the_key_has_something_to_give_back() {
    let src = open_src();
    let i = src
        .find("ph2d_editor::anchor_pivot_world(")
        .expect("o abridor de arrasto nao calcula o pivo");
    let call_end = src[i..].find(')').map_or(src.len(), |e| i + e);
    let call = &src[i..call_end];
    assert!(
        call.contains("false"),
        "o pen-down pede o pivo do CENTRO quando a tecla ja' esta' premida — entao soltar a tecla \
         no meio do arrasto nao tem canto a que voltar (so' o pen-down sabe qual alca foi \
         pega):\n{call}"
    );
}
