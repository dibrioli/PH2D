//! **O sprite vira PADRÃO por um buffer STRAIGHT, e o pedido chega ao frame.**
//!
//! ⚠️ **Nenhum teste de unidade alcança isto.** O cumprimento mora dentro do
//! laço de frame — é o único ponto em que a cena 3D, o mundo 2D, o renderizador
//! e o mapa de atlas estão os quatro em escopo —, e aquele laço precisa de
//! janela e de GPU. Sobra ler o fonte, e o que se lê é a PROPRIEDADE, nunca um
//! endereço.

use std::fs;

const FRAME: &str = "src/render_loop/mod.rs";
const PANEL: &str = "src/sculpt3d_panel.rs";

/// O corpo do bloco que cumpre o pedido, do `mem::replace` do flag até o fim.
fn fulfilment(src: &str) -> String {
    let at = src
        .find("self.sculpt3d_alpha_request, false")
        .expect("o laço de frame cumpre o pedido do alpha por imagem");
    // ⚠️ **O fim é o FIM DO BLOCO, não um número de caracteres.** A 1ª versão
    // pegava 1800 e a 2ª 3600, e as duas reprovaram produto correto assim que o
    // bloco cresceu (a consulta às camadas vivas, depois o readout da escala) —
    // uma janela por contagem mede quanto o autor escreveu, não onde o trabalho
    // acaba. O último ato do cumprimento é anunciar o resultado.
    let rest = &src[at..];
    let end = rest
        .find("toasts.push(Toast::success(line));")
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
    let src = fs::read_to_string(FRAME).expect("o laço de frame existe");
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
    let src = fs::read_to_string(PANEL).expect("o módulo do painel existe");
    let at = src
        .find("pub(crate) fn set_alpha_image")
        .expect("a porta do alpha por imagem existe");
    // ⚠️ **A janela cresceu com o doc-comment do PRODUTO, e isso é o esperado:**
    // ela mede o CORPO (o `end` abaixo o recorta no primeiro `\n    }`), e 600
    // caracteres deixaram de o alcançar quando a porta ganhou o sentinela e a
    // semeadura do eixo. Uma janela curta demais falha por VÁCUO — ela deixa de
    // ver a linha que afirma, e o gate reprova produto correto.
    let body: String = src[at..].chars().take(1600).collect();
    let end = body.find("\n    }").unwrap_or(body.len());
    let body = &body[..end];

    assert!(
        body.contains("recommended_scale"),
        "a porta arma a imagem sem semear a escala: `{}`",
        body.trim()
    );
    assert!(
        body.contains("Alpha::Image"),
        "a porta não arma a imagem: `{}`",
        body.trim()
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
    let src = fs::read_to_string(FRAME).expect("o laço de frame existe");
    let body = fulfilment(&src);

    let at = body
        .find("set_alpha_image(")
        .expect("o cumprimento chama a porta do alpha por imagem");
    // O retorno da porta É a escala em vigor; um `scene.set_alpha_image(a);` solto descarta o
    // único número que o readout tem para dizer.
    let call_line = body[..at].rsplit('\n').next().unwrap_or("");
    assert!(
        call_line.contains("let "),
        "o cumprimento descarta o retorno de `set_alpha_image`: `{}`",
        call_line.trim()
    );
    assert!(
        body.contains("escala {scale:"),
        "o readout nao reporta a escala que a porta devolveu"
    );
    assert!(
        !body.contains("({w}x{h})"),
        "o readout ainda anuncia os pixels da fonte — o unico numero medido como INERTE para a \
         escala do padrao no modelo"
    );
}
