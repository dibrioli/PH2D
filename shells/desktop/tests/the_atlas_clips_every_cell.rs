//! **Nenhuma forma pinta na célula da vizinha** (plano 24 W10 — o atlas de raster).
//!
//! Arch-gate, e ele existe pelo motivo de sempre nesta shell: a decisão mora no `cook_batch`, que
//! precisa de `GpuContext` + `VelloPass` + um renderer do Vello, e **nenhum teste de unidade a
//! alcança**. As metades testáveis já têm gate — o empacotador prova que as células são disjuntas
//! (`fx_atlas`), e a paridade de GPU prova que uma célula filtra igual à forma sozinha
//! (`ph2d-render/tests/fx_stack_atlas_gpu.rs`). O que falta é a terceira: que o RASTER respeite a
//! célula que o empacotador deu.
//!
//! ⚠️ **Isto não é uma otimização, é a reposição de um limite que já existia.** Com um render por
//! forma, a textura tinha exactamente o tamanho do scratch — arte que passasse da caixa calculada
//! (um traço mais largo, uma junta miter comprida) era descartada pelo rasterizador, na borda.
//! Partilhando a textura, essa mesma arte cai **dentro da vizinha**.
//!
//! ⚠️ **E o modo de falha é MUDO**: o que aparece é arte a mais numa forma que não a pediu, no z
//! dela, com a cor dela — nada estoura, nada avisa, e a paridade de GPU continua verde porque ela
//! mede UMA célula por vez.

const SRC: &str = include_str!("../src/fx_live.rs");

/// A janela do fonte que monta a cena partilhada do lote.
fn batch_scene_block() -> &'static str {
    let start = SRC
        .find("let mut scratch_scene = VectorScene::new();")
        .expect("a cena do lote sumiu — ou o `cook_batch` foi reescrito");
    let rest = &SRC[start..];
    let end = rest
        .find("render_to_intermediate")
        .expect("o bloco nao chega ao render — o gate leria o arquivo inteiro");
    &rest[..end]
}

/// **Cada forma é desenhada DENTRO de um clip, e o clip é fechado.**
#[test]
fn the_atlas_clips_every_cell() {
    let block = batch_scene_block();
    assert!(
        block.contains("push_clip"),
        "as formas do lote entram na textura partilhada SEM recorte — arte que passe da caixa \
         calculada vai pintar dentro da célula da vizinha"
    );
    assert!(
        block.contains("pop_layer"),
        "o clip da célula nunca é fechado — as formas seguintes herdariam o recorte da primeira, \
         e todas menos uma desapareceriam"
    );
    // O recorte tem de ser da CÉLULA (a origem que o empacotador deu mais o tamanho da forma), não
    // de um retângulo qualquer: um clip na origem do atlas não recorta nada.
    assert!(
        block.contains("cell.org[0]") && block.contains("job.w"),
        "o recorte não é a célula desta forma — ele tem de sair da origem do empacotador e do \
         tamanho do scratch dela"
    );
    // Controle positivo: sem isto, um `find` que falhasse devolveria um bloco vazio e as três
    // afirmações acima passariam a ser sobre nada.
    assert!(
        block.contains("draw_path_isolated"),
        "o gate não está a ler o laço que desenha as formas — o âncora do `find` apodreceu"
    );
}

/// **O lote é UM render, não um por forma** — a propriedade que a wave inteira compra.
///
/// ⚠️ Afirmada sobre a ESTRUTURA (o `render_to_intermediate` está FORA do laço das células), não
/// sobre um endereço no arquivo: esta linha já teve dois arch-gates apodrecerem por medirem
/// distância em bytes no fonte.
#[test]
fn the_batch_rasterises_in_a_single_pass() {
    let block = batch_scene_block();
    assert!(
        !block.contains("render_to_intermediate"),
        "há um render do Vello DENTRO do laço que monta a cena do lote — o custo fixo (~0,12 ms) \
         voltaria a multiplicar por forma, que é exactamente o que esta wave removeu"
    );
    // E o `cook_batch` chama exactamente um render.
    let body = {
        let start = SRC.find("fn cook_batch").expect("o `cook_batch` sumiu");
        &SRC[start..]
    };
    let end = body
        .find("fn ensure_output")
        .expect("o `cook_batch` nao fecha");
    assert_eq!(
        body[..end].matches("render_to_intermediate").count(),
        1,
        "o `cook_batch` faz mais de um render do Vello"
    );
}
