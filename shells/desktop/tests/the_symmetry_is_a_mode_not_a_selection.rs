//! **Arch-gate: a simetria é um MODO de desenho, não um efeito da selecção.**
//!
//! As três exigências do Enio (2026-08-01) que só a fiação da `render_loop` pode cumprir:
//!
//! > *"A linha deve aparecer logo que se aperta o botão e não quando se inicia o desenho. A
//! > simetria funciona apenas para formas que serão desenhadas com a tool ligada e não deve fazer
//! > simetria de formas que já existem previamente. Com o botão checado pode-se fazer quantos
//! > desenhos desejar que a linha permanece no lugar."*
//!
//! Os gates de unidade (`symmetry_live_tests.rs`) provam que o KERNEL honra isso quando lhe dão a
//! lista certa. **Eles são cegos a quem monta a lista** — e é aí que o modelo anterior errava: ele
//! passava `selected_paths()`, e cada uma das três exigências caía em silêncio, com a suíte verde.
//!
//! Nenhum teste de unidade alcança a `render_loop` (ela precisa de janela e GPU), então a prova é
//! sobre o FONTE — o mesmo recurso que a `line/physics` usou para pinar que o Join não faz fan-out.
//!
//! ⚠️ **Controle positivo:** um scanner que deixe de encontrar o bloco passa a guardar NADA, e um
//! gate que não vê nada passa sempre. Por isso ele primeiro exige achar o bloco e a chamada.

use std::fs;

fn render_loop_src() -> String {
    fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/render_loop/mod.rs"
    ))
    .expect("render_loop/mod.rs")
}

/// O bloco que ADOPTA — do cabeçalho da wave até o cozimento que o consome.
fn adoption_block(src: &str) -> &str {
    let start = src
        .find("A SIMETRIA de DESENHO")
        .expect("o bloco de adopção da simetria sumiu do `render_loop` — o gate perdeu o alvo");
    let rest = &src[start..];
    let end = rest
        .find("self.symmetry_live.recook(")
        .expect("o cozimento da simetria deixou de seguir a adopção — a ordem é a lei do modelo");
    &rest[..end]
}

/// **A adopção olha para quem está EM GESTO, nunca para a selecção.**
///
/// É esta linha que cumpre *"não deve fazer simetria de formas que já existem previamente"*: uma
/// forma pré-existente pode ser seleccionada a qualquer momento, e nunca está em gesto.
#[test]
fn the_adoption_reads_the_gesture_never_the_selection() {
    let src = render_loop_src();
    let block = adoption_block(&src);

    assert!(
        block.contains("self.symmetry_live.adopt("),
        "o bloco existe mas não chama a porta de adopção — o gate está a olhar para o sítio errado"
    );
    assert!(
        block.contains("&drawing"),
        "a adopção tem de receber a lista de GESTO (`drawing`, a mesma do `settle_origins`); sem \
         ela não há como saber o que o artista acabou de desenhar"
    );
    assert!(
        !block.contains("selected_paths"),
        "a adopção NÃO pode ler a selecção: é exactamente assim que uma forma pré-existente ganha \
         simetria sem ninguém a ter desenhado — o modelo que o Enio recusou"
    );
}

/// **A semeadura do eixo é gateada SÓ no interruptor.**
///
/// *"A linha deve aparecer logo que se aperta o botão."* Se a semeadura dependesse de haver forma
/// desenhada ou seleccionada, a linha nasceria tarde — que é a queixa literal.
#[test]
fn the_axis_is_seeded_by_the_switch_alone() {
    let src = render_loop_src();
    let block = adoption_block(&src);

    assert!(
        block.contains("if style.on"),
        "a semeadura tem de pender do interruptor e de mais nada"
    );
    assert!(
        block.contains("get_or_insert_with"),
        "e tem de acontecer UMA vez: re-semear por frame faria a linha seguir a câmera, e panhar o \
         canvas arrastaria o eixo junto — o oposto de *permanece no lugar*"
    );
    assert!(
        block.contains("screen_to_world"),
        "o eixo nasce no centro da TELA (*'a tela é a referência para a posição inicial'*), e só a \
         shell tem câmera para o saber"
    );
}

/// **O overlay desenha a linha de SESSÃO, e não só os eixos das formas.**
///
/// Com a cena vazia não há forma nenhuma a produzir eixo, então sem esta metade o botão ligaria e
/// nada apareceria até o primeiro traço — que é o defeito reportado.
#[test]
fn the_overlay_draws_the_session_line_even_with_an_empty_scene() {
    let src = render_loop_src();
    let start = src
        .find("As linhas da SIMETRIA")
        .expect("o bloco de overlay da simetria sumiu — o gate perdeu o alvo");
    let rest = &src[start..];
    // ⚠️ O fim é a SEÇÃO seguinte, nunca uma distância em bytes: um proxy de janela expira no dia
    // em que alguém acrescenta duas linhas no meio, e esta linha já pagou isso duas vezes.
    let end = rest
        .find("O realce do Shape Builder")
        .expect("a vizinha do overlay de simetria mudou de nome — o gate perdeu o fim do bloco");
    let block = &rest[..end];

    assert!(
        block.contains("symmetry_live::session_axis("),
        "o overlay tem de empurrar o eixo de SESSÃO; os eixos das formas só existem depois de \
         alguém desenhar, e a linha aparece ANTES disso"
    );
    assert!(
        block.contains("self.vec_symmetry_origin"),
        "e a fonte dele é o eixo de sessão da shell, o único que existe com a cena vazia"
    );
    assert!(
        block.contains("draw_symmetry_axes("),
        "e o conjunto tem de chegar ao desenhador"
    );
}
