//! **Arch-gate da costura da ferramenta TRIM** (plano 38) — o gesto inteiro é dela.
//!
//! ## O que este gate protege
//!
//! Quatro maneiras de partir a ferramenta deixam **todos os unit tests verdes**, porque nenhum
//! deles alcança o corpo do `input_dispatch` nem o dreno do quadro:
//!
//! 1. **o press cai na cadeia de baixo** — sem o `return`, um clique no vazio com o Trim na mão
//!    começa a desenhar uma forma (é o defeito que o Lápis e o Width já pagaram, cada um uma vez);
//! 2. **o realce não é limpo ao trocar de ferramenta** — o vermelho fica a arder a prometer um
//!    corte que nenhum clique faz;
//! 3. **o clique RECALCULA em vez de usar o pedaço do quadro** — o cursor pode ter andado um pixel
//!    entre o desenho e o gesto, e numa ferramenta destrutiva isso é apagar outra coisa;
//! 4. **a forma VIVA não congela a receita** — o corte é comido pelo `recook_into` no quadro
//!    seguinte e a ferramenta lê como *"não funciona"* (a queixa exacta do Fillet/Chamfer).
//!
//! As asserções afirmam RELAÇÃO, nunca distância no fonte — esta linha já perdeu arch-gates duas
//! vezes por medir bytes.

const DISPATCH: &str = include_str!("../src/input_dispatch.rs");
const LOOP: &str = include_str!("../src/render_loop/mod.rs");
const TRIM: &str = include_str!("../src/vec_trim.rs");

fn at(src: &str, needle: &str, onde: &str) -> usize {
    src.find(needle).unwrap_or_else(|| {
        panic!(
            "{onde} nao contem `{needle}` — se foi renomeado, actualize este \
             gate (e confira que o Trim ainda funciona: `PH2D_BUILD_SMOKE=80`)"
        )
    })
}

/// **Controle positivo:** as âncoras existem. Um scanner que não acha nada passaria em silêncio.
#[test]
fn the_scanner_finds_what_it_scans_for() {
    at(DISPATCH, "ph2d_tool_vector::DrawMode::Trim", "o dispatch");
    at(DISPATCH, "crate::vec_trim::apply(", "o dispatch");
    at(LOOP, "self.refresh_trim_hover(pointer);", "o render_loop");
    at(LOOP, "ph2d_vec_render::draw_trim_piece(", "o render_loop");
    at(TRIM, "fn trim_hit_at(", "o vec_trim");
}

/// **O press é CONSUMIDO** — o `return` vem depois do corte e antes da cadeia que desenha.
#[test]
fn the_press_never_falls_through_to_the_drawing_chain() {
    let arm = at(
        DISPATCH,
        "self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::Trim",
        "o dispatch",
    );
    let apply = at(DISPATCH, "crate::vec_trim::apply(", "o dispatch");
    let corner = at(
        DISPATCH,
        "if self.vec_draw_config.mode.is_corner_tool() {",
        "o dispatch",
    );
    assert!(
        arm < apply,
        "o corte corre antes de o modo ser reconhecido?"
    );
    assert!(
        apply < corner,
        "o braco do Trim tem de fechar ANTES da cadeia das quinas — sem o `return` dele, um \
         clique no vazio comeca a desenhar"
    );
    let entre = &DISPATCH[apply..corner];
    assert!(
        entre.contains("return;"),
        "o press do Trim nao e' consumido: falta o `return`"
    );
}

/// ⭐⭐ **O CLIQUE USA O PEDAÇO DO QUADRO, e não um recálculo.** É a diferença entre apagar o que
/// se vê e apagar o que estava lá há um pixel.
#[test]
fn the_click_consumes_the_highlighted_piece_and_does_not_recompute_it() {
    let arm = at(
        DISPATCH,
        "self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::Trim",
        "o dispatch",
    );
    let corner = at(
        DISPATCH,
        "if self.vec_draw_config.mode.is_corner_tool() {",
        "o dispatch",
    );
    let bloco = &DISPATCH[arm..corner];
    assert!(
        bloco.contains("self.vec_trim_hit"),
        "o clique tem de LER o pedaco do quadro"
    );
    assert!(
        !bloco.contains("trim_hit_at("),
        "o clique RECALCULOU o pedaco — o que o artista viu a vermelho deixa de ser o que some"
    );
}

/// **A forma VIVA congela a receita**, senão o corte volta no quadro seguinte.
#[test]
fn a_live_shape_freezes_its_recipe_before_the_cut() {
    let arm = at(
        DISPATCH,
        "self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::Trim",
        "o dispatch",
    );
    let apply = at(DISPATCH, "crate::vec_trim::apply(", "o dispatch");
    assert!(
        DISPATCH[arm..apply].contains("freeze_shape_recipe("),
        "sem congelar a receita, o `recook_into` come o corte no quadro seguinte"
    );
}

/// **UM passo de undo**, e ele é CANCELADO quando nada mudou — um clique que erra não pode deixar
/// um passo vazio na fila.
#[test]
fn the_cut_is_one_undo_step_and_a_miss_cancels_it() {
    let arm = at(
        DISPATCH,
        "self.vec_draw_config.mode == ph2d_tool_vector::DrawMode::Trim",
        "o dispatch",
    );
    let corner = at(
        DISPATCH,
        "if self.vec_draw_config.mode.is_corner_tool() {",
        "o dispatch",
    );
    let bloco = &DISPATCH[arm..corner];
    assert!(bloco.contains("self.vec_history.begin("), "falta o begin");
    assert!(
        bloco.contains("commit_if_changed("),
        "falta o commit — o corte nao entra na fila de undo"
    );
    assert!(
        bloco.contains("self.vec_history.cancel();"),
        "um clique que nao corta tem de CANCELAR o passo"
    );
}

/// ⚠️ **O realce é LIMPO fora do modo** — um vermelho a arder promete um corte que nenhum clique
/// faria.
#[test]
fn the_highlight_is_cleared_outside_the_tool() {
    let f = at(TRIM, "pub(crate) fn refresh_trim_hover(", "o vec_trim");
    let corpo = &TRIM[f..f + 900];
    assert!(
        corpo.contains("!= ph2d_tool_vector::DrawMode::Trim"),
        "o refresh nao pergunta pelo modo"
    );
    assert!(
        corpo.contains("self.vec_trim_piece.clear();"),
        "fora do modo o realce tem de ser LIMPO, nao apenas nao-actualizado"
    );
}
