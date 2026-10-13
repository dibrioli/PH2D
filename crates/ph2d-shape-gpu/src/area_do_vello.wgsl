// A ÁREA DE UM PIXEL À LEI DO VELLO — a função partilhada pelo passe de formas (`shape.wgsl`,
// `contorno.wgsl`) e pelo das IMAGENS (`imagem.wgsl`): o quad de uma imagem é um caminho de quatro
// rectas, e a cobertura dele é a MESMA conta. Portada de `vello_shaders/shader/fine.wgsl`
// (`fill_path`, `AaConfig::Area`): Copyright 2022 the Vello Authors — Apache-2.0 OR MIT OR Unlicense.

// A contribuição de UM segmento (em pixels) para a área do pixel cujo canto é `xy` — portada do
// `fill_path` do Vello (ver o cabeçalho).
fn contribuicao(p0: vec2<f32>, p1: vec2<f32>, xy: vec2<f32>) -> f32 {
    let y = p0.y - xy.y;
    let delta = p1 - p0;
    let y0 = clamp(y, 0.0, 1.0);
    let y1 = clamp(y + delta.y, 0.0, 1.0);
    let dy = y0 - y1;
    if dy == 0.0 {
        return 0.0;
    }
    // ⭐ doc 121 §9.5 — **os dois lados sem divisão**: com os DOIS pontos à esquerda do pixel o
    // troço recortado também está (o `x` dele fica entre os dos pontos), e a contribuição é a faixa
    // inteira — o mesmo `dy` que o ramo de baixo devolve, sem o recíproco; com os dois à DIREITA é
    // zero (a fórmula de baixo daria `~1e-6/(xmax − 1)`, o resto do `−1e-6` do Vello).
    let px0 = p0.x - xy.x;
    let px1 = p1.x - xy.x;
    if max(px0, px1) <= 0.0 {
        return dy;
    }
    if min(px0, px1) >= 1.0 {
        return 0.0;
    }
    let vec_y_recip = 1.0 / delta.y;
    let t0 = (y0 - y) * vec_y_recip;
    let t1 = (y1 - y) * vec_y_recip;
    let startx = p0.x - xy.x;
    let x0 = startx + t0 * delta.x;
    let x1 = startx + t1 * delta.x;
    let xmin0 = min(x0, x1);
    let xmax0 = max(x0, x1);
    // ⛔ doc 121 W4 (a linha da `=127`): o segmento todo à ESQUERDA do pixel conta a faixa inteira.
    // A fórmula abaixo dá `1` aí só enquanto `xmax − xmin ≠ 0` — e o `−1e-6` que a garante é do
    // Vello, cujas coordenadas são relativas a um ladrilho de 16 px. Aqui elas são relativas ao
    // PIXEL e chegam a centenas: numa aresta VERTICAL longe à esquerda o `−1e-6` perde-se no `f32`,
    // a conta vira `0/0 = NaN`, e o `min(abs(NaN), 1)` pinta a fileira inteira.
    if xmax0 <= 0.0 {
        return dy;
    }
    let xmin = min(xmin0, 1.0) - 1.0e-6;
    let xmax = xmax0;
    let b = min(xmax, 1.0);
    let c = max(b, 0.0);
    let d = max(xmin, 0.0);
    let a = (b + 0.5 * (d * d - c * c) - xmin) / (xmax - xmin);
    return a * dy;
}
