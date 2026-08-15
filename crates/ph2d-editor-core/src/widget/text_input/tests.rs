//! Gates do [`super`]. Ficheiro irmao DENTRO do directorio, e nao um `text_input_tests.rs`
//! solto em `src/widget/`: o `ph2d-widget-sync` varre os `*.rs` do topo e um ficheiro de
//! testes ali viraria um "widget" sem showcase nem a11y (a cicatriz do `command_palette`).

use super::*;

/// **A TINTA, e não o token:** a borda de um campo aceso não é a de um campo em repouso.
///
/// ⚠️ Um gate de token ficaria verde no dia em que a paleta pusesse `Border` e `BorderEmph` na
/// mesma cor; este compara o que o pintor produz.
#[test]
fn the_lit_border_is_not_the_resting_border() {
    use ph2d_tokens::Theme;
    let t = Theme::Forge;
    let rest = border_color(TextInputState::Normal, crate::motion::SETTLED, t);
    let hot = border_color(TextInputState::Hovered, crate::motion::SETTLED, t);
    assert_ne!(rest, hot, "acender um campo nao muda um pixel");
}

/// **E o meio-caminho fica ENTRE as duas pontas** — a prova de que o eixo é o escalar, não o
/// estado.
///
/// *Mutação que deve sangrar:* `border_color` a ignorar o `hover_t`.
#[test]
fn halfway_the_border_is_between_the_two_ends() {
    use ph2d_tokens::Theme;
    let t = Theme::Forge;
    let rest = border_color(TextInputState::Normal, crate::motion::SETTLED, t);
    let hot = border_color(TextInputState::Hovered, crate::motion::SETTLED, t);
    let mid = border_color(TextInputState::Hovered, 0.5, t);
    let eps = 1e-4; // LITERAL-PX-OK: tolerancia de f32 num canal normalizado (nao e' geometria)
    let between = |a: f32, b: f32, m: f32| m >= a.min(b) - eps && m <= a.max(b) + eps;
    for c in 0..3 {
        assert!(
            between(rest.components[c], hot.components[c], mid.components[c]),
            "canal {c}: meio caminho {} fora de [{}, {}]",
            mid.components[c],
            rest.components[c],
            hot.components[c]
        );
    }
    assert_ne!(mid, rest, "o meio caminho nao saiu do repouso");
    assert_ne!(mid, hot, "o meio caminho ja esta no fim");
}

fn fixture() -> TextInput {
    TextInput::new(NodeId(1), "Project name")
}

#[test]
fn defaults_match_spec() {
    let t = fixture();
    assert_eq!(t.value, "");
    assert_eq!(t.placeholder, "");
    assert_eq!(t.state, TextInputState::Normal);
    assert_eq!(t.caret_byte, 0);
}

#[test]
fn value_seed_moves_caret_to_end() {
    let t = fixture().value("hello");
    assert_eq!(t.value, "hello");
    assert_eq!(t.caret_byte, 5);
}

#[test]
fn a11y_role_is_text_input() {
    let node = fixture().build_a11y(0.0, 0.0, 200.0, 32.0);
    assert_eq!(node.role(), Role::TextInput);
}

fn smoke(t: TextInput, theme: Theme) {
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_text_input(
        &t,
        Rect::new(0.0, 0.0, 240.0, 32.0),
        &mut scene,
        &mut text,
        theme,
    );
}

/// Enio, 2026-07-16: renaming a clip to a long name drew a SECOND line that ran
/// out of the box and over the buttons below it. A single-line field clips —
/// and it must CLOSE what it opens, because an unbalanced layer would corrupt
/// everything painted after the field, not just the field.
#[test]
fn a_long_name_is_clipped_to_the_field_instead_of_spilling_out_of_it() {
    let long = "L2 ldldll ldllld dhdhdhhhhd jjdjjjd jdjfjjd";
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let t = fixture().state(TextInputState::Focused);
    paint_text_input_with_buffer(
        &t,
        Some(long),
        Some(long.len()),
        None,
        Rect::new(0.0, 0.0, 140.0, 32.0),
        &mut scene,
        &mut text,
        Theme::Forge,
    );
    let enc = scene.inner().encoding();
    assert!(enc.n_clips >= 1, "the field must clip its text to its box");
    assert_eq!(enc.n_open_clips, 0, "the clip must be closed");
}

#[test]
fn the_line_scrolls_only_far_enough_to_keep_the_caret_in_view() {
    // A caret inside the box leaves the line where it is.
    assert!((caret_scroll(true, 100.0, 40.0)).abs() < f32::EPSILON);
    // Past the right edge it slides exactly as far as it overhangs, plus the
    // caret's own width.
    assert!((caret_scroll(true, 100.0, 140.0) - 41.0).abs() < f32::EPSILON);
    // Unfocused: no caret to chase, so the name reads from its start.
    assert!((caret_scroll(false, 100.0, 140.0)).abs() < f32::EPSILON);
}

#[test]
fn paint_smoke_empty_with_placeholder() {
    smoke(fixture().placeholder("Untitled"), Theme::Forge);
}

#[test]
fn paint_smoke_filled_focused() {
    smoke(
        fixture()
            .value("hello world")
            .state(TextInputState::Focused),
        Theme::Forge,
    );
}

#[test]
fn paint_smoke_hovered() {
    smoke(fixture().state(TextInputState::Hovered), Theme::Sunstone);
}

#[test]
fn paint_smoke_error() {
    smoke(
        fixture().value("oops").state(TextInputState::Error),
        Theme::Blueprint,
    );
}

#[test]
fn paint_smoke_disabled() {
    smoke(
        fixture().value("locked").state(TextInputState::Disabled),
        Theme::Workshop,
    );
}

#[test]
fn paint_with_buffer_overrides_value() {
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let t = fixture().value("stale").state(TextInputState::Focused);
    // Pretend the WidgetStore has a freshly typed buffer.
    paint_text_input_with_buffer(
        &t,
        Some("live edit"),
        Some(4),
        None,
        Rect::new(0.0, 0.0, 240.0, 32.0),
        &mut scene,
        &mut text,
        Theme::Forge,
    );
}

#[test]
fn paint_with_buffer_handles_empty_caret_oob() {
    // Caret beyond buffer length should still paint without
    // panic (clamped at draw time).
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let t = fixture().state(TextInputState::Focused);
    paint_text_input_with_buffer(
        &t,
        Some(""),
        Some(99),
        None,
        Rect::new(0.0, 0.0, 240.0, 32.0),
        &mut scene,
        &mut text,
        Theme::Sunstone,
    );
}
