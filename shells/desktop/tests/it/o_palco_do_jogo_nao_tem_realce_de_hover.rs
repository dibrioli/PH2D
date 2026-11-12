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
        linha.contains("pick_hovered_object") && linha.contains("(!palco"),
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

/// **Com uma aba de documento aberta (Input Map, Kanban, scripts…), o rato não fura até ao canvas**
/// (smoke do dono 11/11: o hover sobre a aba Input Map acendia objectos da cena na Hierarchy). A aba
/// cobre o canvas, então conta como painel: a resposta é a linha apontada, nunca o objecto por baixo.
/// **Mutação que deve sangrar:** tirar `|| hero.documents.surface_active().is_some()`.
#[test]
fn uma_aba_de_documento_aberta_tapa_o_hover_do_canvas() {
    let src = so_codigo(include_str!("../../src/hover_highlight.rs"));
    let corpo = &src[src
        .find("fn pick_hovered_object")
        .expect("a porta única do hover")..];
    let decide = &corpo[..corpo.find("if over_panel").expect("a decisão do painel")];
    assert!(
        decide.contains("chrome_panel_at(")
            && decide.contains("documents.surface_active().is_some()"),
        "o painel da porta do hover não conta a aba de documento aberta — o rato fura até ao canvas:\n{decide}"
    );
    let ramo = &corpo[corpo.find("if over_panel").unwrap_or(0)..];
    assert!(
        ramo[..ramo.find("PickWorld").expect("o canvas depois do painel")].contains("return"),
        "o ramo do painel tem de sair antes do canvas"
    );
}
