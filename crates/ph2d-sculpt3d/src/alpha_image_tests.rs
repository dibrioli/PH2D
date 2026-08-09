//! Os gates do [`super::AlphaImage`] e do décimo padrão.

use super::AlphaImage;
use crate::{Alpha, Brush};

/// RGBA de `w × h` a partir de um gerador `(x, y) -> [r, g, b, a]`.
fn rgba(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Vec<u8> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            v.extend_from_slice(&f(x, y));
        }
    }
    v
}

/// **A LEI: luminância × alfa** — e as duas metades são afirmadas separadas,
/// porque elas falham por motivos diferentes.
///
/// ⚠️ **O oráculo não chama a função sob teste.** Os coeficientes do Rec. 709
/// estão escritos aqui em literal; usar a `LUM` do módulo tornaria este gate um
/// espelho, e trocar os três números passaria em silêncio.
#[test]
fn the_law_is_luminance_times_alpha() {
    // Uma coluna de cores conhecidas, opacas.
    let px = rgba(4, 1, |x, _| match x {
        0 => [255, 255, 255, 255], // branco
        1 => [0, 0, 0, 255],       // preto
        2 => [255, 0, 0, 255],     // vermelho puro
        _ => [0, 255, 0, 255],     // verde puro
    });
    let img = AlphaImage::from_rgba(4, 1, &px).expect("a fixture descreve o buffer");

    // Amostra no CENTRO de cada texel: `u = (i + 0.5) / w`.
    let at = |i: u32| img.sample((f32::from(i as u16) + 0.5) / 4.0, 0.5);
    assert!((at(0) - 1.0).abs() < 2e-3, "branco não é cheio: {}", at(0));
    assert!(at(1) < 2e-3, "preto não é vazio: {}", at(1));
    // 0,2126 e 0,7152 — os coeficientes, não o que o módulo diz que eles são.
    assert!(
        (at(2) - 0.2126).abs() < 4e-3,
        "o vermelho não pesa o Rec. 709: {}",
        at(2)
    );
    assert!(
        (at(3) - 0.7152).abs() < 4e-3,
        "o verde não pesa o Rec. 709: {}",
        at(3)
    );

    // ⚠️ **E a metade do ALFA:** branco TRANSPARENTE não tem tinta nenhuma.
    // Sem o `× a` ele leria 1,0 — cheio — onde não há nada.
    let ghost = rgba(2, 1, |x, _| {
        if x == 0 {
            [255, 255, 255, 0]
        } else {
            [255, 255, 255, 255]
        }
    });
    let g = AlphaImage::from_rgba(2, 1, &ghost).expect("descreve");
    assert!(
        g.sample(0.25, 0.5) < 2e-3,
        "um texel transparente pesou {}",
        g.sample(0.25, 0.5)
    );
    assert!((g.sample(0.75, 0.5) - 1.0).abs() < 2e-3);
}

/// **A imagem LADRILHA — ela não é uma ponta finita.**
///
/// ⚠️ **É a diferença que a separa do slot Shape do Painter**, onde uma imagem É
/// um carimbo e fora dele a cobertura é zero. Aqui ela é um PADRÃO, irmão dos
/// nove que cobrem a superfície inteira; recortá-la faria o pincel pintar um
/// retângulo, e o artista descobriria isso pela borda.
#[test]
fn the_image_tiles_instead_of_being_clipped() {
    let px = rgba(4, 4, |x, y| {
        let v = u8::try_from((x * 60 + y * 15) % 256).unwrap_or(255);
        [v, v, v, 255]
    });
    let img = AlphaImage::from_rgba(4, 4, &px).expect("descreve");

    for (u, v) in [(0.3f32, 0.7f32), (0.05, 0.95), (0.5, 0.5)] {
        let home = img.sample(u, v);
        for (du, dv) in [(1.0f32, 0.0f32), (0.0, 3.0), (-2.0, -5.0)] {
            let far = img.sample(u + du, v + dv);
            assert!(
                (home - far).abs() < 1e-6,
                "o ladrilho ({du}, {dv}) leu {far} onde a origem lê {home}"
            );
        }
    }
}

/// **A emenda do ladrilho é CONTÍNUA** — o vizinho à direita do último texel é o
/// primeiro.
///
/// ⚠️ **Sem o wrap nos DOIS vizinhos a última coluna interpolaria contra ela
/// mesma**, e cada emenda ganharia uma costura de um texel — visível exatamente
/// onde o olho procura repetição. A fixture tem um DEGRAU forte na fronteira
/// (preto no fim, branco no começo) para a costura ter o que mostrar.
#[test]
fn the_tile_seam_is_continuous() {
    let px = rgba(4, 1, |x, _| {
        let v = if x == 3 { 0 } else { 255 };
        [v, v, v, 255]
    });
    let img = AlphaImage::from_rgba(4, 1, &px).expect("descreve");

    // ⚠️ **O oráculo é a DESCONTINUIDADE, e a primeira versão deste gate estava
    // errada:** ela comparava duas amostras a `0,002` de distância contra um
    // épsilon de `5e-3`, e a fixture tem contraste MÁXIMO — um passo desses muda
    // `0,008` legitimamente. *A tolerância era mais apertada que a inclinação*, e
    // ela reprovou código correto. Agora a varredura é densa e o que se afirma é
    // que NENHUM passo é grande: um salto de emenda vale ~1,0, um passo liso
    // vale ~0,02, e a barra fica dez vezes abaixo do primeiro e cinco acima do
    // segundo.
    const N: usize = 201;
    let mut worst = 0.0f32;
    let mut prev = img.sample(0.5, 0.5);
    for i in 1..N {
        // `0,5 .. 1,5` atravessa a emenda do ladrilho E a fronteira que um
        // `clamp` no lugar do wrap produziria — as duas ficam nessa faixa.
        let u = 0.005f32.mul_add(f32::from(i as u16), 0.5);
        let now = img.sample(u, 0.5);
        worst = worst.max((now - prev).abs());
        prev = now;
    }
    assert!(
        worst < 0.1,
        "a emenda salta {worst:.3} entre amostras vizinhas — o vizinho do último \
         texel não é o primeiro, e cada ladrilho ganha uma costura"
    );
}

/// **Coordenada NEGATIVA lê o ladrilho certo**, e isto acontece o tempo todo: o
/// padrão é lido em espaço de OBJETO, cuja origem fica no meio da peça, então
/// metade da malha tem `u < 0`.
#[test]
fn a_negative_coordinate_wraps_instead_of_folding() {
    let px = rgba(4, 4, |x, y| {
        let v = u8::try_from((x * 37 + y * 91) % 256).unwrap_or(255);
        [v, v, v, 255]
    });
    let img = AlphaImage::from_rgba(4, 4, &px).expect("descreve");
    for (u, v) in [(0.1f32, 0.2f32), (0.6, 0.9), (0.45, 0.05)] {
        assert!(
            (img.sample(u, v) - img.sample(u - 1.0, v - 1.0)).abs() < 1e-6,
            "o ladrilho negativo não casa em ({u}, {v})"
        );
        assert!(img.sample(-3.7, -9.2).is_finite());
    }
}

/// **A porta RECUSA um buffer que não descreve as dimensões** — uma imagem
/// meio-lida é um padrão que desenha lixo.
#[test]
fn the_door_refuses_a_buffer_that_does_not_describe_the_image() {
    assert!(AlphaImage::from_rgba(4, 4, &[0u8; 60]).is_none(), "curto");
    assert!(AlphaImage::from_rgba(0, 4, &[0u8; 64]).is_none(), "vazia");
    assert!(AlphaImage::from_rgba(4, 0, &[0u8; 64]).is_none(), "vazia");
    // O caso exato passa, e um buffer MAIOR também (o excedente é ignorado).
    assert!(AlphaImage::from_rgba(4, 4, &[7u8; 64]).is_some());
    assert!(AlphaImage::from_rgba(4, 4, &[7u8; 999]).is_some());
}

/// Uma imagem de teste com estrutura ANISOTRÓPICA — faixas, para o giro do eixo
/// ter o que mover.
fn striped() -> Alpha {
    let px = rgba(8, 8, |x, _| {
        let v = if x < 4 { 255 } else { 0 };
        [v, v, v, 255]
    });
    Alpha::Image(std::sync::Arc::new(
        AlphaImage::from_rgba(8, 8, &px).expect("descreve"),
    ))
}

/// **A imagem é DIRECIONAL, e o eixo de fato a move.**
///
/// ⚠️ **A metade que importa é a SEGUNDA:** `is_directional` devolver `true` é
/// uma afirmação sobre um `match`; o que o artista compra é o padrão GIRAR
/// quando ele arrasta a pista. Um gate só do predicado ficaria verde com o
/// `weight_at` ignorando o frame.
#[test]
fn the_image_turns_with_the_axis() {
    let img = striped();
    assert!(img.is_directional(), "uma imagem precisa de um frame");

    let brush = |az: u16| Brush {
        alpha: Some(img.clone()),
        alpha_scale: 0.2,
        alpha_az_deg: az,
        alpha_elev_deg: 0,
        ..Brush::default()
    };
    let a = brush(0);
    let b = brush(90);
    let (fa, fb) = (a.alpha_frame(), b.alpha_frame());

    let mut moved = 0;
    for i in 0..64 {
        // Uma varredura no plano XY, longe da origem para o giro morder.
        let p = [
            0.05f32.mul_add(f32::from(i as u16), -1.6),
            0.37,
            0.11 * f32::from(i as u16),
        ];
        if (a.alpha_weight(p, &fa) - b.alpha_weight(p, &fb)).abs() > 1e-3 {
            moved += 1;
        }
    }
    assert!(
        moved > 16,
        "girar o eixo 90° moveu {moved} de 64 amostras — o frame não chega à imagem"
    );
}

/// **Duas imagens são o MESMO padrão quando são a MESMA imagem**, e o custo é a
/// razão.
///
/// ⚠️ **A derivada compararia PIXEL A PIXEL**, e a chave do cache do swatch do
/// painel é comparada a cada quadro: um megabyte de `memcmp` por frame para
/// responder *"o artista mexeu?"*. A segunda metade é a que prova que a
/// comparação é por IDENTIDADE — dois buffers de conteúdo idêntico e `Arc`s
/// diferentes têm de sair DIFERENTES.
#[test]
fn two_alphas_are_the_same_when_they_are_the_same_image() {
    let a = striped();
    assert_eq!(a, a.clone(), "clonar o Arc mudou o padrão");

    let b = striped();
    assert_ne!(
        a, b,
        "duas imagens de mesmo conteúdo compararam iguais — a comparação está \
         lendo os pixels, e ela roda por quadro"
    );

    // E os nove seguem comparando por NOME, não por endereço.
    assert_eq!(Alpha::Noise, Alpha::Noise);
    assert_ne!(Alpha::Noise, Alpha::Grain);
    assert_ne!(Alpha::Noise, a);
}

/// **A imagem fica FORA do [`Alpha::ALL`]**, e isso é a fileira de chips dizendo
/// a verdade.
///
/// ⚠️ **Aquela lista é o que a UI oferece como NOMES.** Uma imagem não é um nome
/// — é uma coisa para a qual se aponta —, e um chip "Image" nela criaria
/// exatamente o estado que esta wave torna inexprimível: a escolha sem os
/// pixels.
#[test]
fn the_image_is_not_a_chip_in_the_row_of_names() {
    assert_eq!(Alpha::ALL.len(), 9, "a fileira de chips mudou de tamanho");
    assert!(
        !Alpha::ALL.iter().any(|a| matches!(a, Alpha::Image(_))),
        "uma imagem entrou na lista de nomes"
    );
    // Controle: a lista não está vazia, e cada um dela tem rótulo próprio.
    let mut seen: Vec<&str> = Alpha::ALL.iter().map(Alpha::label).collect();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), 9, "dois padrões dividem um rótulo");
}

/// **Um pincel LISO continua devolvendo `1.0` EXATO** — a byte-identidade que
/// toda wave de alpha desta linha se apoia.
#[test]
fn a_smooth_brush_is_still_byte_identical() {
    let b = Brush::default();
    assert!(b.alpha.is_none(), "o default ganhou um padrão");
    let f = b.alpha_frame();
    for p in [[0.0f32, 0.0, 0.0], [0.3, -0.7, 1.1], [-9.0, 4.0, 0.5]] {
        assert!(
            (b.alpha_weight(p, &f) - 1.0).abs() == 0.0,
            "o pincel liso deixou de ser 1.0 ao bit"
        );
    }
}
