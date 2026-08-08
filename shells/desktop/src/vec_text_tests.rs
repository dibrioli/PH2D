//! Gates da SESSÃO de texto ([`crate::vec_text`]) — irmão pelo teto de 600 LOC (HR-18),
//! e **FILHO** por `#[path]`: o `use super::*` alcança o que é privado na sessão (o
//! `layout_of`, o `placement_of`), que é onde o caret mora.

use super::*;
use ph2d_vec_scene::Rgba8;

fn black() -> Paint {
    Paint::solid(Rgba8::new(0, 0, 0, 255))
}

/// O cursor de texto fica à direita da origem depois de digitar, e é vertical.
#[test]
fn the_caret_advances_as_text_is_typed() {
    let edit = VecTextEdit {
        origin: [5.0, 2.0],
        size: 1.0,
        weight: 400.0,
        line_height: 1.2,
        tracking: 0.0,
        align: TextAlign::Left,
        extra_axes: Vec::new(),
        family: None,
        fill: Some(black()),
        stroke: None,
        text: "Hi".to_string(),
        id: None,
        center: [0.0, 0.0],
        wrap_width: None,
    };
    let (a, b) = caret_of(Some(&edit)).expect("caret com edição ativa");
    assert!(a[0] > 5.0, "o cursor avançou à direita da origem");
    assert!((a[0] - b[0]).abs() < 1e-9, "cursor vertical");
    assert!(b[1] > a[1], "topo acima da base");
    assert!(caret_of(None).is_none(), "sem edição, sem cursor");
}

/// Sair do modo Text COMMITA a sessão: `sync_active_text_style` com um modo != Text
/// zera a sessão (os glyphs ficam na cena). Sem isto, uma sessão viva no Select
/// regeneraria o texto inteiro a cada mudança de Style — pegando letras não
/// selecionadas e sumindo com o gizmo (Enio 2026-07-11).
#[test]
fn leaving_text_mode_commits_the_session() {
    use ph2d_tool_vector::DrawMode;
    let mut scene = ph2d_vec_scene::VecScene::new();
    let mut edit = Some(VecTextEdit {
        origin: [0.0, 0.0],
        size: 1.0,
        weight: 400.0,
        line_height: 1.2,
        tracking: 0.0,
        align: TextAlign::Left,
        extra_axes: Vec::new(),
        family: None,
        fill: Some(black()),
        stroke: None,
        text: "A".to_string(),
        id: None,
        center: [0.0, 0.0],
        wrap_width: None,
    });
    let pen = ph2d_vec_edit::PenTool::default();
    sync_active_text_style(&mut edit, DrawMode::Select, &pen, 0.01, &mut scene);
    assert!(edit.is_none(), "modo != Text termina a sessão (commit)");
}

/// No modo Text, mudar o Style do painel regenera os glyphs vivos com o novo Paint
/// (herança em tempo real). O glyph troca de fill sem sair da sessão.
#[test]
fn active_text_restyles_live_when_the_panel_style_changes() {
    use ph2d_tool_vector::DrawMode;
    use ph2d_vec_edit::{PenStyle, PenTool};
    let mut scene = ph2d_vec_scene::VecScene::new();
    let mut edit = Some(VecTextEdit {
        origin: [0.0, 0.0],
        size: 1.0,
        weight: 400.0,
        line_height: 1.2,
        tracking: 0.0,
        align: TextAlign::Left,
        extra_axes: Vec::new(),
        family: None,
        fill: Some(black()),
        stroke: None,
        text: "A".to_string(),
        id: None,
        center: [0.0, 0.0],
        wrap_width: None,
    });
    regen_into(&mut scene, edit.as_mut().unwrap());
    assert_eq!(scene.paths().len(), 1, "o glyph foi para a cena");
    let mut pen = PenTool::default();
    pen.set_style(PenStyle {
        fill: Rgba8::new(10, 200, 30, 255),
        ..PenStyle::default()
    });
    sync_active_text_style(&mut edit, DrawMode::Text, &pen, 0.0, &mut scene);
    let gid = edit.as_ref().unwrap().id.expect("o compound do texto");
    let fill = scene
        .paths()
        .iter()
        .find(|p| p.id == gid)
        .and_then(|p| p.fill.clone());
    assert!(
        matches!(fill, Some(Paint::Solid(c)) if c.g == 200),
        "o texto adotou o novo fill do painel ao vivo"
    );
    assert_eq!(
        scene.paths().len(),
        1,
        "regen atualiza o compound in-place, não acumula"
    );
}

/// Uma sessão pronta a digitar, com a caixa que o gate quiser.
fn session(text: &str, wrap: Option<f64>) -> VecTextEdit {
    VecTextEdit {
        origin: [0.0, 0.0],
        size: 0.3,
        weight: 400.0,
        line_height: 1.2,
        tracking: 0.0,
        align: TextAlign::Left,
        extra_axes: Vec::new(),
        family: None,
        fill: Some(black()),
        stroke: None,
        text: text.to_string(),
        wrap_width: wrap,
        id: None,
        center: [0.0, 0.0],
    }
}

/// A caixa das fixtures do caret — a mesma dos gates do motor, e larga o bastante para o
/// texto de teste quebrar mais de uma vez.
const CARET_BOX: f64 = 3.0;

/// Um texto sem `\n` nenhum que a caixa parte em várias linhas. É a fixture que separa
/// *contar quebras escritas* de *contar linhas desenhadas* — num texto com `\n` as duas
/// respostas coincidem, e o defeito passaria.
const FLOWING: &str = "the quick brown fox jumps over the lazy dog again and again";

/// ⭐ **O gate da wave: o cursor pousa na última linha DESENHADA, não na última digitada.**
///
/// Sem refluxo as duas perguntas dão o mesmo número e nada disto se vê; com uma caixa, contar
/// `\n` responde *quantas quebras o artista escreveu* (zero, aqui) e o caret fica pendurado na
/// primeira linha enquanto a tinta já desceu três — a lei
/// [[feedback_derived_coordinate_seed_must_match_sample]], aplicada ao cursor.
///
/// ⚠️ **A mutação que o prova:** devolver `edit.text.rsplit('\n')` + `matches('\n').count()` ao
/// `caret_of`. Ela deixa os outros dois gates de caret VERDES (os dois usam texto de uma linha)
/// e este VERMELHO, com o caret a 0.0 de altura sobre um bloco de três linhas.
#[test]
fn the_caret_sits_on_the_last_drawn_line_not_the_last_typed_one() {
    let boxed = session(FLOWING, Some(CARET_BOX));
    let loose = session(FLOWING, None);
    let (a_boxed, _) = caret_of(Some(&boxed)).expect("caret");
    let (a_loose, _) = caret_of(Some(&loose)).expect("caret");

    // Quantas linhas a caixa de facto produziu — o gate não pode afirmar "desceu" sem saber
    // que havia para onde descer.
    let f = crate::vec_font::resolve(None);
    let lines = crate::vec_glyph::wrapped_lines(
        &f,
        FLOWING,
        &layout_of(&boxed),
        &axes_of(&boxed),
        &placement_of(&boxed),
    );
    assert!(
        lines.len() >= 3,
        "a fixture tem de conter o fenomeno: {} linha(s)",
        lines.len()
    );

    // O cursor DESCEU exactamente a entrelinha vezes o número de linhas refluídas.
    // ⚠️ O ponto devolvido é a BASE do cursor, não o `pen`: o `caret_of` desce `0.2·size` (o
    // descendente do traço). O oráculo tem de carregar isso — foi ele que estava errado na 1ª
    // corrida, não o produto.
    let step = boxed.size * boxed.line_height;
    let want_y = -((lines.len() - 1) as f64) * step - 0.2 * boxed.size;
    assert!(
        (a_boxed[1] - want_y).abs() < 1e-9,
        "o caret devia estar na linha {} (y {want_y:.4}), esta' em {:.4}",
        lines.len() - 1,
        a_boxed[1]
    );
    // E o controle no MESMO texto: sem caixa ele fica na primeira linha, onde a tinta está.
    assert!(
        (a_loose[1] + 0.2 * loose.size).abs() < 1e-9,
        "sem caixa o texto e' uma linha so' - o caret nao pode descer ({:.4})",
        a_loose[1]
    );
    // A metade horizontal: com a caixa o cursor recua para dentro dela; sem ela vai até ao fim
    // de uma linha muito mais longa.
    assert!(
        a_boxed[0] < a_loose[0],
        "o caret refluido ({:.4}) tem de estar a' esquerda do solto ({:.4})",
        a_boxed[0],
        a_loose[0]
    );
    assert!(
        a_boxed[0] <= CARET_BOX,
        "o caret ({:.4}) saiu da caixa ({CARET_BOX:.4})",
        a_boxed[0]
    );
}

/// **Sem caixa o caret é byte-idêntico ao que já shipava** — o controle que torna o bump e a
/// porta nova seguros para todo texto já autorado.
#[test]
fn without_a_box_the_caret_is_where_it_always_was() {
    for text in ["Hi", "one\ntwo", "a\n\nb"] {
        let edit = session(text, None);
        let (a, _) = caret_of(Some(&edit)).expect("caret");
        // A régua antiga, escrita à mão aqui de propósito: é o oráculo EXTERNO, e não uma
        // segunda chamada à porta que o produto agora usa.
        let f = crate::vec_font::resolve(None);
        let last = text.rsplit('\n').next().unwrap_or("");
        let idx = text.matches('\n').count();
        let want_x = crate::vec_glyph::caret_x_offset(&f, last, &layout_of(&edit), &axes_of(&edit));
        let want_y = -(idx as f64) * edit.size * edit.line_height - 0.2 * edit.size;
        assert!(
            (a[0] - want_x).abs() < 1e-9 && (a[1] - want_y).abs() < 1e-9,
            "sem caixa o caret mudou de sitio em {text:?}: {a:?} vs [{want_x}, {want_y}]"
        );
    }
}

/// **A caixa viaja: sessão → componente → sessão.** Sem isto o refluxo seria um ajuste que o
/// texto esquece ao ser fechado e reaberto — e o gate de round-trip é o único sítio onde
/// *"apendei o campo aos DOIS lados"* deixa de ser promessa.
#[test]
fn the_box_survives_the_round_trip_through_the_component() {
    let edit = session("a b c", Some(4.25));
    let params = crate::vec_text_object::text_params_for_test(&edit);
    assert_eq!(
        params.wrap_width,
        Some(4.25),
        "a caixa tem de chegar ao componente"
    );
    assert_eq!(
        crate::vec_text_object::layout_of_params(&params).wrap_width,
        Some(4.25),
        "e voltar ao layout que o cozedor consome"
    );
}
