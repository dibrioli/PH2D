//! **A âncora de escala é do FRAME, não do pen-down**: arch-gate sobre a costura que nenhum unit test
//! alcança. A lei é `ph2d_editor_core::live_anchor`, com gates de kernel; aqui, a fiação, que pede `App`,
//! `HeroScreen` e janela: (1) a porta é chamada por movimento; (2) o resultado é LOCAL e o estado
//! guardado continua a ser o CANTO, que não se re-deriva depois do pen-down; (3) o pen-down guarda o canto.
//! Afirma-se qual pergunta é feita e onde a resposta pousa, nunca distância em bytes.

fn drag_src() -> String {
    crate::input_text::gizmo_drag_file()
}

fn open_src() -> String {
    crate::input_text::dispatch()
}

/// **(1) O avanço do arrasto pergunta a âncora deste frame.**
#[test]
fn the_drag_asks_for_this_frames_anchor() {
    let src = drag_src();
    assert!(
        src.contains("ph2d_editor_core::live_anchor("),
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
        .find("ph2d_editor_core::live_anchor(")
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
/// ⚠️ O oráculo é a PORTA que o abridor chama: `grip_pivot_world` não tem o argumento do centro (pede
/// o canto ao `anchor_pivot_world` com `false`, prova ao bit em `gizmo/escala_tests.rs`), e o lado sai
/// do cursor do pen-down, para o canto oposto de um objecto espelhado ser o oposto da mão.
#[test]
fn the_pen_down_stores_the_corner_so_the_key_has_something_to_give_back() {
    let src = open_src();
    // A porta antiga tem o argumento do centro: nenhum abridor a pode chamar, nem num ramo ao lado.
    assert!(
        !src.contains("ph2d_editor_core::anchor_pivot_world("),
        "o despacho voltou a chamar `anchor_pivot_world`, que deixa o abridor pedir o CENTRO"
    );
    let i = src
        .find("ph2d_editor_core::grip_pivot_world(")
        .expect("o abridor de arrasto nao calcula o pivo");
    let call_end = src[i..].find(')').map_or(src.len(), |e| i + e);
    let call = &src[i..call_end];
    assert!(
        call.contains("start_world"),
        "o pen-down pede o pivo do CENTRO quando a tecla ja' esta' premida — entao soltar a tecla \
         no meio do arrasto nao tem canto a que voltar (so' o pen-down sabe qual alca foi \
         pega):\n{call}"
    );
}

/// **(4) Gizmo global com o primário a zero: a conta e o fator usam a partida do grupo.**
/// A lei é pura, em `gizmo/escala_tests.rs`; aqui, as duas pontas da fiação.
#[test]
fn the_group_math_and_its_factor_read_the_same_start() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/input_dispatch/gizmo_drag_calculo.rs"
    ))
    .expect("a conta do arrasto mudou de ficheiro: reaponte este gate");
    assert_eq!(
        src.matches("gizmo::partida_do_grupo(").count(),
        2,
        "a conta e o fator"
    );
    assert!(src.contains("gizmo::escala_do_primario(&drag, new_t.scale)"));
}
