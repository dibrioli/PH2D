//! **O VÉU DE UMIDADE É RECORTADO À JANELA ANTES DE SER CONSTRUÍDO.**
//!
//! O véu cobre o rect CUMULATIVO de umidade — ele só cresce enquanto o artista pinta —, e o build
//! custa ~8-12 ns/texel. O log do produto de 2026-08-02 (`PH2D_PAINT_PERF`, sessão do Enio) o mediu
//! subindo **2,13 → 9,67 → 42,64 ms/quadro** (`CHROME wet`): a essa altura, **60% do quadro inteiro**,
//! com `frame p50 = 72,3 ms` (~14 fps). Recortar à viewport é **livre de mudança de aparência por
//! construção** — o que está fora da janela ninguém vê — e troca um custo que cresce com a PINTURA
//! por um limitado pela TELA.
//!
//! ⚠️ **Este gate existe porque os testes de unidade do `clip_to_viewport` são CEGOS à fiação:**
//! removida a CHAMADA, os três passam verdes sobre um produto que reconstrói a pintura inteira todo
//! quadro. É a lição que a `line/anim` já pagou (*um gate de unidade é cego ao shell*), e a mutação
//! que a prova é justamente apagar a linha da chamada.

const SRC: &str = include_str!("../src/painter_bridge_wetness.rs");

#[test]
fn the_build_only_sees_the_visible_region() {
    // ⚠️ **A âncora é a ATRIBUIÇÃO inteira, não a chamada.** A 1ª versão procurava a chamada e depois
    // exigia a atribuição na fatia `call..build` — mas o `let` vem ANTES da chamada, na mesma linha,
    // então a fatia nunca podia contê-lo e o gate reprovava o código correto. Uma âncora só é oráculo
    // se descrever a forma que o produto de fato tem. O espaço em branco normaliza-se: o `rustfmt`
    // parte a atribuição em linhas (integ 13/11), e a quebra não é a lei.
    let src: String = SRC.split_whitespace().collect::<Vec<_>>().join(" ");
    let call = src
        .find(
            "let (rx0, ry0, rx1, ry1) = clip_to_viewport(base, window_size, (rx0, ry0, rx1, ry1),",
        )
        .expect(
            "`draw_wetness_overlay` recorta o rect à janela E escreve o resultado de volta — um \
             recorte cujo resultado ninguém lê é um recorte que não recorta",
        );
    // ⚠️ Âncora no build que CONSOME o rect recortado: desde B16.10 o véu é construído (e guardado)
    // pela tool, `painter.wetness_veil([rx0, ry0, rx1, ry1], …)`; o `build_veil` mora em
    // `ph2d-tool-painter/…/watercolor_backdrop/veu.rs`.
    let build = src
        .find("= painter.wetness_veil([rx0, ry0, rx1, ry1],")
        .expect("`draw_wetness_overlay` constrói o véu sobre o rect recortado");
    assert!(
        call < build,
        "o recorte à janela acontece DEPOIS do build — o véu volta a ser construído sobre a pintura \
         inteira, que é o custo que o log mediu em 42,64 ms/quadro"
    );
}
