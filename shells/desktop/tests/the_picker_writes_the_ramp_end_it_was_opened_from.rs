//! **A cor escolhida no picker pousa no SLOT que o artista abriu** (plano 24 W9, estendido na W11).
//!
//! Arch-gate, e ele existe porque a decisão mora dentro do `render_frame` — a função que exige
//! janela e dispositivo, e que **nenhum teste de unidade alcança**. A metade testável já tem gate
//! (`fx_live::colour_target`, que traduz o id do alvo em *(linha, slot)*); o que falta provar é que
//! a shell **usa** a resposta.
//!
//! ⚠️ **O modo de falha é MUDO:** abrir a swatch clara do Duotone e escolher uma cor escreveria na
//! escura — o picker mostra a cor certa, o card mostra a cor certa no lugar errado, e nada parece
//! quebrado. É exactamente a classe de defeito que a lista de `FilterHit` não consegue impedir,
//! porque as swatches são o MESMO tipo de controle para todo o resto do sistema.
//!
//! ⚠️ **A afirmação é sobre a PROPRIEDADE, não sobre um endereço** — esta linha já teve dois
//! arch-gates apodrecerem por afirmarem distância em bytes no fonte, e o conserto foi este.
//!
//! ⚠️ **A W11 provou o gate certo pela via difícil, DUAS vezes.** (1) Ele QUEBROU quando o terceiro
//! slot chegou — o readback ramificava num `bool` (*"é a segunda?"*) e o comentário dele já previa
//! esta wave (*"derivar a ponta do `kind` faria a segunda escrever na primeira em qualquer tipo que
//! ganhasse uma rampa depois"*). (2) E a mutação que dobrava o stop na ponta escura **passou verde**,
//! porque um arch-gate vê FORMA: o nome `ColourSlot::SelectedStop` sobrevivia num braço inalcançável.
//!
//! ⚠️ **Por isso a rota saiu daqui.** Ela mora em `fx_live::apply_picked_colour` — pura, e portanto
//! observável por um gate de unidade (`each_colour_slot_gets_the_picked_colour_and_only_it`). Este
//! arquivo ficou com o que só o fonte mostra: **que a shell ENTREGA à porta única, com o slot que o
//! alvo do picker nomeou** — e com as duas recusas que impedem a decisão de voltar para dentro da
//! função window-gated.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// O ORÇAMENTO da janela do scanner, em bytes.
///
/// ⚠️ **O número é MEDIDO** (1267 bytes com a entrega à porta única), não afrouxado por
/// conveniência — e ficou MENOR que os 2000 do gate original, porque extrair a rota encurtou o
/// bloco. O que o orçamento existe para pegar é o `find` que passa a devolver o arquivo inteiro, e
/// para isso a ordem de grandeza basta; um valor colado no exacto falharia no próximo comentário que
/// alguém escrevesse dentro do bloco.
const MAX_WINDOW: usize = 2000;

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

/// **A shell ENTREGA à porta única, com o slot que o alvo do picker nomeou.**
///
/// ⚠️ **É tudo o que um arch-gate pode provar, e a wave descobriu isso pela via difícil.** A versão
/// anterior afirmava *"o bloco contém `ColourSlot::SelectedStop`"*, e a mutação que dobrava o stop na
/// ponta escura **manteve esse nome num braço inalcançável**: verde sobre a regressão exacta que o
/// arquivo existe para pegar. A rota mudou-se para `fx_live::apply_picked_colour`, onde ela é
/// OBSERVÁVEL (gate `each_colour_slot_gets_the_picked_colour_and_only_it`, mutação-provado); aqui
/// fica só a COSTURA, que é o que só o fonte mostra.
#[test]
fn the_picker_writes_the_ramp_end_it_was_opened_from() {
    let block = readback_block();
    assert!(
        block.contains("apply_picked_colour(op, slot,"),
        "o readback nao entrega a porta unica com o SLOT — se ele voltasse a ramificar aqui, a rota \
         deixaria de ser observavel por um teste de unidade, e a regressao volta a ser muda"
    );
    assert!(
        !block.contains("op.color = col"),
        "o readback voltou a escrever a cor DIRETO — a decisao esta outra vez dentro da funcao que \
         exige janela, onde nenhum gate de unidade a alcanca"
    );
    assert!(
        !block.contains("op.kind =="),
        "o readback passou a olhar o `kind` do degrau — o alvo do picker e a unica resposta certa \
         para *qual cor o artista abriu*"
    );
}

/// **Controle POSITIVO do scanner.** Sem ele, um `find` que passasse a devolver a janela errada
/// deixaria as asserções acima verdes sobre um bloco que não é o do readback.
#[test]
fn the_scanner_reads_the_block_it_thinks_it_reads() {
    let block = readback_block();
    assert!(
        block.contains("blender_picker"),
        "a janela do gate nao contem o picker — ele esta a ler outra coisa"
    );
    assert!(
        block.len() < MAX_WINDOW,
        "a janela do gate cresceu demais ({} bytes, orcamento {MAX_WINDOW}): ela deixou de \
         descrever UM bloco",
        block.len()
    );
}
