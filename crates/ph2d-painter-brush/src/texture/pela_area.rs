//! **A borda pela ÁREA do píxel**: o que um padrão ou uma curva desenha mais fino que um píxel, lido num ponto, acende e
//! apaga em degraus (o Bokeh, o Halftone: smoke 04/11). Aqui mora a largura do píxel em cada espaço de amostragem, a
//! média do grão de imagem, e a chave que desliga a cura para a medir no mesmo processo.

use super::*;
use std::sync::atomic::{AtomicBool, Ordering};

static SEM_AREA: AtomicBool = AtomicBool::new(false);

/// Desliga a leitura pela área (Dots e falloff Custom). Só para gate e medição A×B
/// ([MEDIR_VELOCIDADE](../../../../docs/DevOps/MEDIR_VELOCIDADE.md)). ⚠️ O global é do processo: um teste que o vira
/// muda o que os vizinhos pintam, e por isso corre num processo seu.
#[doc(hidden)]
pub fn mede_sem_a_area(sem: bool) {
    SEM_AREA.store(sem, Ordering::Relaxed);
}

pub(crate) fn ligada() -> bool {
    !SEM_AREA.load(Ordering::Relaxed)
}

fn maior_size(s: &TextureSettings) -> f32 {
    s.size[0]
        .clamp(TEX_SIZE_MIN, TEX_SIZE_MAX)
        .max(s.size[1].clamp(TEX_SIZE_MIN, TEX_SIZE_MAX))
}

/// Quanto a pegada estica uma unidade do dab (o maior dos dois eixos): `1` num dab redondo.
fn esticao(b: &TexDabBasis) -> f32 {
    let [x, y] = [b.footprint.apply([1.0, 0.0]), b.footprint.apply([0.0, 1.0])];
    x[0].det_hypot(x[1]).max(y[0].det_hypot(y[1]))
}

/// A largura de um píxel em células do padrão preso à tela ([`sample_tiled_rot_wrapped`]). `0` = lê-se num ponto.
pub(super) fn largura_tiled(s: &TextureSettings) -> f32 {
    if s.kind == TextureKind::Dots && ligada() {
        maior_size(s) / TEX_TILE_BASE_PX
    } else {
        0.0
    }
}

/// A largura de um píxel em células na amostra de um dab ([`sample`]), por mapeamento.
pub(super) fn largura_no_dab(s: &TextureSettings, b: &TexDabBasis, radius: f32) -> f32 {
    if s.kind != TextureKind::Dots || !ligada() {
        return 0.0;
    }
    if s.mapping.is_stencil() {
        let meio = b.stencil_half[0].min(b.stencil_half[1]).max(1e-3);
        return maior_size(s) * 0.5 * stencil::STENCIL_TILES / meio;
    }
    if s.flow {
        return maior_size(s) * esticao(b) / b.frame.parts().1;
    }
    if matches!(s.mapping, TextureMapping::Tiled) {
        return largura_tiled(s);
    }
    maior_size(s) * esticao(b) / radius.max(1e-3)
}

/// A largura de um píxel em células na máscara assada: `passo` = o lado do texel em unidades do dab.
pub(super) fn largura_unit(s: &TextureSettings, b: &TexDabBasis, passo: f32) -> f32 {
    if s.kind == TextureKind::Dots && ligada() {
        maior_size(s) * esticao(b) * passo
    } else {
        0.0
    }
}

/// ⭐ **O GRÃO de imagem pela ÁREA do píxel** (o Stars pequeno, 23/10 — a mesma lei da ponta de imagem,
/// [`shape::sample_shape`]): reduzida, a imagem lida num ponto por píxel acende e apaga os traços finos em
/// degraus (erro 0,55 contra a área a raio 5). A média de `nx × ny` amostras pelo píxel (`em(desvio)` = a
/// coordenada da textura), `n` = metade dos texels que um passo de um píxel atravessa (≤ 8); ampliada é a amostra
/// no centro ao bit.
pub(super) fn grao_de_imagem(
    s: &TextureSettings,
    img: &ImageMask,
    em: impl Fn([f32; 2]) -> [f32; 2],
) -> f32 {
    let ler = |t: [f32; 2]| patterns::sample_kind(s.kind, t, s.params, Some(img)).clamp(0.0, 1.0);
    let t0 = em([0.0, 0.0]);
    let (hw, hh) = (
        img.width.max(1) as f32 * 0.5,
        img.height.max(1) as f32 * 0.5,
    );
    let texels = |d: [f32; 2]| ((d[0] - t0[0]) * hw).det_hypot((d[1] - t0[1]) * hh);
    let lado = |t: f32| {
        if t.is_finite() {
            (0.5 * t).ceil().clamp(1.0, shape::AREA_MAX)
        } else {
            1.0
        }
    };
    let (nx, ny) = (lado(texels(em([1.0, 0.0]))), lado(texels(em([0.0, 1.0]))));
    if nx <= 1.0 && ny <= 1.0 {
        return ler(t0);
    }
    let mut soma = 0.0;
    for j in 0..ny as u32 {
        let oy = (j as f32 + 0.5) / ny - 0.5;
        for i in 0..nx as u32 {
            let ox = (i as f32 + 0.5) / nx - 0.5;
            soma += ler(em([ox, oy]));
        }
    }
    soma / (nx * ny)
}
