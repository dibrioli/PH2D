//! **O sprite vira PADRÃO por um buffer STRAIGHT, e o pedido chega ao frame.**
//!
//! ⚠️ **Nenhum teste de unidade alcança isto.** O cumprimento mora dentro do
//! laço de frame — é o único ponto em que a cena 3D, o mundo 2D, o renderizador
//! e o mapa de atlas estão os quatro em escopo —, e aquele laço precisa de
//! janela e de GPU. Sobra ler o fonte, e o que se lê é a PROPRIEDADE, nunca um
//! endereço.

use std::fs;

// ⚠️ O laço lê-se no texto EMENDADO (`frame_text::render_frame`) desde a OBRA 2 da `line/render-loop`
// (2026-09-12): o cumprimento do pedido mudou-se para a fase `fase_sculpt3d_bake.rs`.
const PANEL: &str = "../../crates/ph2d-app-sculpt3d/src/panel.rs";

/// O corpo do bloco que cumpre o pedido, do `mem::replace` do flag até o fim.
fn fulfilment(src: &str) -> String {
    let at = src
        .find("req.alpha_request, false")
        .expect("o laço de frame cumpre o pedido do alpha por imagem");
    // ⚠️ **O fim é o FIM DO BLOCO, não um número de caracteres.** A 1ª versão
    // pegava 1800 e a 2ª 3600, e as duas reprovaram produto correto assim que o
    // bloco cresceu (a consulta às camadas vivas, depois o readout da escala) —
    // uma janela por contagem mede quanto o autor escreveu, não onde o trabalho
    // acaba. O último ato do cumprimento é anunciar o resultado.
    let rest = &src[at..];
    // ⛔⛔ **E a âncora do fim é `toasts.push(` e não a chamada INTEIRA** — 3.ª reparação desta
    // mesma linha, e a 1.ª que não é sobre o tamanho: em 21/09 o aviso passou a escolher a CARA
    // pelo veredito (`if deu { success } else { error }`, a foto do ✓ verde numa recusa) e a
    // âncora literal deixou de casar. *A propriedade é «o cumprimento acaba ao ANUNCIAR»; qual
    // construtor de aviso ele usa é um detalhe que este gate não tem de saber.*
    let end = rest
        .find("toasts.push(")
        .expect("o cumprimento termina anunciando o que fez");
    rest[..end].to_string()
}

/// **A conversão para STRAIGHT precede a construção da imagem.**
///
/// ⚠️ **É correção, não higiene.** A lei do `AlphaImage` é `luminância × alfa`, e
/// num buffer PREMULTIPLICADO a luminância já traz o alfa dentro: o peso sairia
/// com o alfa ao **QUADRADO**, e toda borda macia ficaria mais fina do que o
/// desenho é. Um sprite `Individual` volta premultiplicado do readback, então
/// este não é um caso de canto — é o caminho de quem acabou de pintar.
#[test]
fn the_pixels_are_straightened_before_the_law_reads_them() {
    let src = crate::frame_text::render_frame();
    let body = fulfilment(&src);

    // ⚠️ **A asserção é sobre o caminho da imagem GUARDADA, e o recorte é o
    // conserto de um proxy que envelheceu.** A 1ª versão procurava a primeira
    // `from_rgba` do bloco inteiro; quando o padrão passou a consultar as CAMADAS
    // vivas antes (`composite_to_lum`), a primeira `from_rgba` virou a do
    // composite — e ela é ISENTA por construção: a luminância entra como cinza
    // OPACO, onde premultiplicar é a identidade. Quem pode chegar premultiplicado
    // é o readback do sprite, então é dele que esta ordem fala.
    let stored = body
        .split_once("read_sprite_source(")
        .expect("o cumprimento lê os pixels do sprite")
        .1;
    let straight = stored
        .find("into_straight()")
        .expect("o cumprimento converte para straight");
    let build = stored
        .find("AlphaImage::from_rgba")
        .expect("o cumprimento constrói a imagem");
    assert!(
        straight < build,
        "a imagem é construída ANTES de o buffer ser endireitado — o peso sai \
         com o alfa ao quadrado"
    );
    // **Controle positivo:** se o bloco deixar de ler o sprite, isto falha alto
    // em vez de varrer o vazio.
    assert!(
        body.contains("read_sprite_source"),
        "o cumprimento deixou de ler os pixels do sprite"
    );
}

/// **Armar um padrão SEMEIA a escala**, e a porta é uma só.
///
/// ⚠️ **Sem isto o smoke reprova, e já reprovou:** *"os poros são gigantescos"*.
/// O chip semeia pelo número que o retrato publica; a imagem semeia aqui, onde
/// a cena tem a MALHA na mão — e as duas metades perguntam à mesma
/// `recommended_scale`, senão um padrão nasceria num tamanho e o outro noutro.
#[test]
fn arming_an_image_seeds_the_scale_this_model_can_hold() {
    // ⚠️ **O MÓDULO, e não o corpo de UMA função — a terceira vez desta cicatriz
    // na mesma jornada.** As versões anteriores recortavam o corpo do
    // `set_alpha_image` por contagem de caracteres (600, depois 1600) e
    // reprovaram produto correto duas vezes: primeiro quando o doc-comment
    // cresceu, depois quando a LEI se mudou para o `seed_alpha_placement` — a
    // porta passou a DELEGAR, que é exatamente o que se quer de uma lei com dois
    // pedintes, e o gate leu a delegação como ausência.
    //
    // O que ele afirma agora é a propriedade que importa: *a semeadura existe
    // neste módulo, UMA vez, e a porta que arma a imagem existe*. Onde ela mora
    // dentro do arquivo é endereço.
    let src = fs::read_to_string(PANEL).expect("o módulo do painel existe");

    assert!(
        src.contains("pub fn set_alpha_image"),
        "a porta do alpha por imagem sumiu do módulo do painel"
    );
    assert!(
        src.contains("Alpha::Image"),
        "o módulo do painel não arma a imagem em lugar nenhum"
    );
    // ⚠️ **Toda pergunta vai ao MOTOR, e a contagem NÃO é a propriedade** — a 1ª
    // versão desta asserção exigia *exatamente uma* menção e reprovou produto
    // correto: há duas chamadas legítimas (o retrato publica o seed que o CHIP
    // usa, a porta semeia o que o BOTÃO arma) e duas menções em prosa. As duas
    // chamadas serem a mesma função é o ponto; contá-las é ruído.
    assert!(
        src.contains("ph2d_sculpt3d::recommended_scale("),
        "o painel deixou de perguntar ao motor que tamanho este modelo comporta \
         — um número escolhido aqui seria a segunda resposta, e ela diverge no \
         primeiro modelo de outra densidade"
    );
}

/// **O READOUT diz a ESCALA, não os pixels.**
///
/// ⚠️ **Porque a resolução da fonte tem efeito ZERO sobre o tamanho do padrão no modelo** — o
/// `AlphaImage::sample` mapeia em unidades de LADRILHO, então a MESMA imagem a 64² e a 4096² dá o
/// mesmo número de transições ao longo das mesmas unidades de objeto. O número que de fato governa
/// o que o artista vê é o `Alpha Scale`, e era justamente ele que mudava sem aparecer em lugar
/// nenhum: o toast anunciava `WxH` e ficava calado sobre o que tinha acabado de escrever.
#[test]
fn the_readout_reports_the_scale_the_door_returned() {
    let src = crate::frame_text::render_frame();
    let body = fulfilment(&src);

    // ⛔⛔ **A conta da escala mudou-se para a família em 21/09** (uma catraca da shell trouxe a
    // lei do padrão para lá), logo o `set_alpha_image` já não é chamado neste corpo. ⭐ O que fica
    // aqui é a FIAÇÃO — *a fase chama a lei* —, e o retorno dela é afirmado onde ela vive.
    assert!(
        body.contains("alpha_pedido::drain("),
        "o cumprimento deixou de chamar a lei do padrão"
    );

    // ⚠️⚠️ **E esta metade MUDOU DE CASA com o texto.** Ela media *«o readout não anuncia os
    // pixels da fonte»* no corpo da fase — e com a frase na tabela ela passava a ser trivialmente
    // verdadeira ali, verde a afirmar nada. *Migrar texto para uma tabela move, em silêncio, o
    // alvo de todo gate que o media.*
    let tabela = fs::read_to_string("../../crates/ph2d-i18n/src/app_sculpt3d.rs")
        .expect("a tabela de texto da familia existe");
    let frase = tabela
        .split_once("\"app.sculpt3d.alpha.padrao_definido\"")
        .expect("a frase do padrao vive na tabela da familia")
        .1
        .lines()
        .nth(1)
        .unwrap_or_default();
    assert!(
        frase.contains("{scale}"),
        "a frase do padrao deixou de dizer a ESCALA: {frase}"
    );
    assert!(
        !frase.contains("{w}") && !frase.contains("{h}"),
        "o readout voltou a anunciar os pixels da fonte — o unico numero medido como INERTE para \
         a escala do padrao no modelo: {frase}"
    );
}
