//! A pirâmide (o mipmap) das máscaras do carimbo: caixa 2×2 por nível e a escolha do nível pelo tamanho do dab.

/// ⭐ A pirâmide de uma máscara quadrada de `canais` bytes por texel: `size/2`, `size/4`… por caixa 2×2 (a média
/// arredondada), enquanto o lado for par. Premultiplicada, a caixa do RGBA é a média certa da cor.
pub(crate) fn piramide(data: &[u8], size: u32, canais: usize) -> Vec<(Vec<u8>, u32)> {
    let mut niveis: Vec<(Vec<u8>, u32)> = Vec::new();
    let mut size = size;
    while size >= 2 && size.is_multiple_of(2) {
        let src = niveis.last().map_or(data, |(d, _)| d.as_slice());
        let (s, h) = (size as usize, (size / 2) as usize);
        let mut meio = vec![0u8; h * h * canais];
        for j in 0..h {
            for i in 0..h {
                for c in 0..canais {
                    let at = |x: usize, y: usize| u32::from(src[(y * s + x) * canais + c]);
                    let soma = at(2 * i, 2 * j)
                        + at(2 * i + 1, 2 * j)
                        + at(2 * i, 2 * j + 1)
                        + at(2 * i + 1, 2 * j + 1);
                    meio[(j * h + i) * canais + c] = ((soma + 2) / 4) as u8;
                }
            }
        }
        size /= 2;
        niveis.push((meio, size));
    }
    niveis
}

/// O índice do nível de uma pirâmide de lados `topo, topo/2, …` (`0` = o topo) para um dab que percorre
/// `lado_px` píxeis: o MENOR com lado `≥ lado_px` (a redução bilinear ≤ 2:1); `n` = quantos níveis há abaixo do topo.
pub(crate) fn indice_do_nivel(topo: u32, n: usize, lado_px: f32) -> usize {
    let mut k = 0;
    while k < n && (topo >> (k + 1)) as f32 >= lado_px {
        k += 1;
    }
    k
}
