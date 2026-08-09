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
    // Uma janela generosa: o bloco tem comentário longo, e o que importa é que
    // as duas chamadas apareçam nele — não a que distância.
    src[at..].chars().take(3600).collect()
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
    let body: String = src[at..].chars().take(600).collect();
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
