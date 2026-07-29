//! **A cor escolhida no picker pousa na PONTA que o artista abriu** (plano 24 W9).
//!
//! Arch-gate, e ele existe porque a decisão mora dentro do `render_frame` — a função que exige
//! janela e dispositivo, e que **nenhum teste de unidade alcança**. A metade testável já tem gate
//! (`fx_live::colour_target`, que traduz o id do alvo em *(linha, é_a_segunda)*); o que falta provar
//! é que a shell **usa** a resposta.
//!
//! ⚠️ **O modo de falha é MUDO:** com o Duotone, abrir a swatch clara e escolher uma cor escreveria
//! na escura — o picker mostra a cor certa, o card mostra a cor certa no lugar errado, e nada
//! parece quebrado. É exactamente a classe de defeito que a lista de `FilterHit` não consegue
//! impedir, porque as duas pontas são o MESMO tipo de controle para todo o resto do sistema.
//!
//! ⚠️ **A afirmação é sobre a PROPRIEDADE, não sobre um endereço** — esta linha já teve dois
//! arch-gates apodrecerem por afirmarem distância em bytes no fonte, e o conserto foi este.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// A janela do fonte que decide para onde a cor do picker vai.
fn readback_block() -> &'static str {
    let start = SRC
        .find("crate::fx_live::colour_target(target)")
        .expect("o readback do picker de filtro sumiu — ou o nome da porta unica mudou");
    // Até ao fim da edição que ele dispara. O `edit(` seguinte é o dela.
    let rest = &SRC[start..];
    let end = rest
        .find("});")
        .expect("o bloco do readback nao fecha — o gate leria o arquivo inteiro");
    &rest[..end]
}

/// **As duas pontas são escritas, e a escolha vem do ALVO do picker.**
#[test]
fn the_picker_writes_the_ramp_end_it_was_opened_from() {
    let block = readback_block();
    assert!(
        block.contains("op.color_b = col;"),
        "o readback do picker nunca escreve a SEGUNDA ponta — a swatch clara seria decorativa"
    );
    assert!(
        block.contains("op.color = col;"),
        "o readback do picker deixou de escrever a primeira ponta"
    );
    assert!(
        block.contains("if second {"),
        "o readback nao ramifica no ALVO do picker — se ele derivasse a ponta do `kind`, um tipo \
         futuro com rampa escreveria sempre na mesma swatch"
    );
}

/// **Controle POSITIVO do scanner.** Sem ele, um `find` que passasse a devolver a janela errada
/// deixaria as três asserções acima verdes sobre um bloco que não é o do readback.
#[test]
fn the_scanner_reads_the_block_it_thinks_it_reads() {
    let block = readback_block();
    assert!(
        block.contains("blender_picker"),
        "a janela do gate nao contem o picker — ele esta a ler outra coisa"
    );
    assert!(
        block.len() < 2000,
        "a janela do gate cresceu demais ({} bytes): ela deixou de descrever UM bloco",
        block.len()
    );
}
