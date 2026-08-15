//! Os gates do **GESTO** da rolagem.
//!
//! ⚠️ Eles são ARCH-GATES sobre o fonte, e não testes de comportamento, pela razão de sempre nesta
//! shell: a decisão mora dentro do `on_mouse_wheel`, que precisa de `gfx` (janela + GPU) para
//! sequer converter o cursor para mundo. Nenhum teste de unidade a alcança — e um gate que
//! afirmasse a mesma coisa chamando a função por dentro estaria a testar a função, não a ORDEM em
//! que ela é perguntada, que é o que decide se a roda dá zoom ou rola.

/// O fonte do despachante de entrada.
fn dispatch_src() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/input_dispatch.rs"
    ))
    .expect("o despachante existe")
}

/// O fonte **sem os comentários de linha** — para um gate poder afirmar coisas sobre o que o
/// código FAZ sem casar com o que a prosa ao lado DIZ.
///
/// ⚠️ **Aproximação declarada:** um `//` dentro de um literal de string também seria cortado.
/// Neste ficheiro os gates procuram chamadas de função, então o falso-corte só poderia tornar o
/// gate mais permissivo num sítio onde ele já não afirmava nada — e nunca mais severo.
fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|l| l.split_once("//").map_or(l, |(code, _)| code))
        .collect::<Vec<_>>()
        .join("\n")
}

/// **A moldura é perguntada ANTES do zoom.**
///
/// ⚠️ Depois dele a câmera já se mexeu quando a moldura for consultada, e o artista veria a lista
/// rolar *e* a cena aproximar no mesmo tique — o defeito que nenhum gate de unidade pode ver,
/// porque as duas metades funcionam isoladas.
#[test]
fn the_frame_is_asked_before_the_camera_zooms() {
    let s = dispatch_src();
    let ask = s
        .find("wheel_scrolls_a_frame")
        .expect("a roda pergunta pela moldura");
    // ⚠️ **O `expect` é o controle POSITIVO**, e não cerimónia: no dia em que o zoom mudar de
    // porta outra vez, este gate tem de falhar ALTO — nunca varrer um ficheiro onde a coisa que
    // ele julga deixou de existir e passar por vácuo. (A cicatriz é o gate do `keyboard.rs`, que
    // ficou verde sobre um dono que se tinha mudado.)
    let zoom = s
        .find("canvas_zoom.wheel(")
        .expect("a roda escreve o destino do zoom");
    assert!(
        ask < zoom,
        "a moldura tem de ser perguntada ANTES do zoom (pergunta em {ask}, zoom em {zoom})"
    );
}

/// **A roda NÃO move a câmera; ela escreve um destino.**
///
/// ⚠️ Sem esta metade, reverter para o `camera.zoom(factor)` de antes passaria no gate de ORDEM
/// acima — a moldura continuaria a ser perguntada primeiro, e o zoom voltaria a saltar. É a
/// diferença entre *quando* se pergunta e *o que* se faz com a resposta.
///
/// ⚠️ **E o oráculo olha para o CÓDIGO, não para a prosa** — a primeira versão nasceu VERMELHA
/// sobre um produto correto, porque casou com o comentário que esta mesma wave escreveu a explicar
/// o que tinha saído dali. *Um gate que casa com a documentação de si mesmo não está a olhar para
/// o produto* (a cicatriz é o `stamps: media` do Painter).
///
/// *Mutação que deve sangrar:* `gfx.camera.zoom(factor)` de volta no `on_mouse_wheel`.
#[test]
fn the_wheel_never_writes_the_camera_itself() {
    let s = strip_line_comments(&dispatch_src());
    assert!(
        !s.contains("camera.zoom("),
        "o despachante da roda nao pode mover a camera na hora — quem a move e' o quadro \
         (`canvas_zoom::tick`), senao o gesto mais repetido do app volta a saltar"
    );
}

/// **Um painel continua a ganhar da moldura.**
///
/// ⚠️ Controle POSITIVO da ordem acima: o `over_panel` é decidido no topo do handler e as duas
/// perguntas seguintes (escultura e moldura) são guardadas por ele. Sem esta afirmação, mover a
/// pergunta da moldura para o topo passaria no gate anterior e faria a roda sobre um painel rolar
/// a arte por baixo dele.
#[test]
fn a_panel_still_wins_the_wheel() {
    let s = dispatch_src();
    let i = s
        .find("wheel_scrolls_a_frame")
        .expect("a roda pergunta pela moldura");
    // A linha da pergunta tem de trazer a guarda do painel consigo.
    let line_start = s[..i].rfind('\n').map_or(0, |n| n + 1);
    let line = &s[line_start..i];
    assert!(
        line.contains("!over_panel"),
        "a pergunta da moldura tem de ser guardada por `!over_panel`, e a linha diz: {line:?}"
    );
}
