//! **Com o palco do jogo aberto, o editor não realça o que está debaixo do rato** (plano 11 do
//! GameEngineFuncional): a foto do nível 2 mostrou um fio azul de 2 px a atravessar o jogo — o
//! contorno de hover da camada «Hills», a única aresta dela dentro da vista. O rato é do jogo
//! (`posse_do_jogo`), e os outros auxiliares do editor já perguntavam `no_palco()`; a escolha do
//! objecto apontado não perguntava.
//!
//! ⚠️ A lente é o TEXTO EMENDADO do quadro, sem as linhas de comentário (a lição do gate da paralaxe).

fn so_codigo(texto: &str) -> String {
    texto
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// **Mutação que deve sangrar:** `self.hovered_object = self.pick_hovered_object(pointer);`.
#[test]
fn o_palco_do_jogo_nao_tem_realce_de_hover() {
    let src = so_codigo(&crate::frame_text::render_frame());
    let at = src
        .find("self.hovered_object =")
        .expect("o quadro deixou de escolher o objecto apontado");
    let linha = &src[at..at + src[at..].find(';').expect("a atribuição fecha")];
    assert!(
        linha.contains("pick_hovered_object") && linha.contains("(!palco)"),
        "o objecto apontado escolhe-se também com o palco aberto — o contorno de hover desenha-se \
         por cima do jogo:\n{linha}"
    );
    let antes = &src[..at];
    let decl = antes
        .rfind("let palco =")
        .expect("o `palco` não está declarado antes da escolha");
    assert!(
        antes[decl..].starts_with("let palco = self.no_palco();"),
        "o `palco` da escolha não é o `no_palco()`:\n{}",
        &antes[decl..]
    );
}
