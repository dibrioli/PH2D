// colour_adjust.wgsl — **O AJUSTE DE COR do app**, num arquivo só.
//
// Extraído do `layer_composite.wgsl` quando ganhou o SEGUNDO consumidor (o degrau *Color Adjust*
// da pilha de FX raster do módulo vetorial, plano 24 W8) — **exactamente o movimento que o
// `blend_modes.wgsl` já fez** pelo mesmo motivo, e pela mesma razão: quando duas partes do app
// respondem *"o que a matiz/saturação/brilho fazem a uma cor?"*, elas têm de responder com a
// MESMA função. Duas cópias divergem no único lugar onde ninguém lê um número — uma cor.
//
// ⚠️ **Prefixo de módulo: não parseia sozinho, de propósito.** Quem o compila é a `composite_source()`
// do compositor e o `module_sources()` da pilha de FX.
//
// ⚠️ **Os coeficientes são bit-idênticos aos literais do Rust** (`ph2d_color::oklab::OklabColor`),
// e NÃO os valores de espec em precisão cheia — a paridade GPU↔CPU deriva com eles. Pinados por
// `shader_adjustment_coefficients_bit_identical_with_rust`, que lê a fonte MONTADA.

// Linear sRGB → OKLab. Coefficients bit-identical to
// `ph2d_color::oklab::OklabColor::from_linear` (the rounded f32 literals the
// Rust source uses — NOT the full-precision spec values, or the GPU↔CPU parity
// drifts). Pinned by `shader_adjustment_coefficients_bit_identical_with_rust`.
fn oklab_from_linear(c: vec3<f32>) -> vec3<f32> {
    let l = 0.41222147 * c.r + 0.5363325 * c.g + 0.051445993 * c.b;
    let m = 0.2119035 * c.r + 0.6806995 * c.g + 0.10739696 * c.b;
    let s = 0.08830246 * c.r + 0.28171884 * c.g + 0.6299787 * c.b;
    let l_ = pow(max(l, 0.0), 0.3333333333);
    let m_ = pow(max(m, 0.0), 0.3333333333);
    let s_ = pow(max(s, 0.0), 0.3333333333);
    return vec3<f32>(
        0.21045426 * l_ + 0.7936178 * m_ - 0.004072047 * s_,
        1.9779985 * l_ - 2.4285922 * m_ + 0.4505937 * s_,
        0.025904037 * l_ + 0.78277177 * m_ - 0.80867577 * s_,
    );
}

// OKLab → linear sRGB. Coefficients bit-identical to `OklabColor::to_linear`.
fn oklab_to_linear(lab: vec3<f32>) -> vec3<f32> {
    let l_ = lab.x + 0.39633778 * lab.y + 0.21580376 * lab.z;
    let m_ = lab.x - 0.105561346 * lab.y - 0.06385417 * lab.z;
    let s_ = lab.x - 0.08948418 * lab.y - 1.2914855 * lab.z;
    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;
    return vec3<f32>(
        4.0767417 * l3 - 3.3077116 * m3 + 0.23096994 * s3,
        -1.268438 * l3 + 2.6097574 * m3 - 0.34131938 * s3,
        -0.0041960863 * l3 - 0.7034186 * m3 + 1.7076147 * s3,
    );
}

// **A LEI**, sobre um triplo linear RETO (não premultiplicado). Espelho exacto do
// `ph2d_painter_effects::adjustments::compute::apply_hsb`:
//
// - **Matiz** (`h`, em VOLTAS): rotação RÍGIDA do vetor de croma **em OKLab**, `L` intacto.
//   ⚠️ Não é a matriz YIQ do `hueRotate` do SVG nem a matiz do HSL, e o porquê está na história
//   deste repo: a matiz do HSL é numericamente instável em pixel quase-cinza (croma minúsculo ⇒
//   matiz mal definida), e rodá-la espalhava cor incoerente — o *salpico colorido* que o Enio
//   viu num fundo cinzento. Em OKLab a instabilidade não existe.
// - **Saturação** (`s`, `-1..1`): escala o croma (`-1` = cinza, `+1` = 2×).
// - **Brilho** (`b`, `-1..1`): lerp para preto (`-1`) / branco (`+1`) em luz LINEAR, então os
//   extremos são preto e branco EXACTOS.
//
// ⚠️ **O neutro `{0,0,0}` devolve a entrada AO BIT — em FLOAT, pelo ramo; em 8 bits, de graça.**
// A frase precisa importa porque eu escrevi a errada primeiro (*"sem ele um degrau parado
// repintaria toda a arte já autorada"*) e a mutação a desmentiu: tirando o early-out, uma rampa
// sRGB COMPLETA (256 níveis × 3 canais) sai com **0 de 4096 bytes diferentes, pior delta 0** — o
// erro do ida-e-volta OKLab em `f32` fica muito abaixo de meio nível, e a quantização o come.
// O ramo é então **exactidão no float** (que é o que compõe numa pilha longa) **e custo** (dois
// triplos de raiz cúbica por texel que não se paga). É a mesma frase do `apply_hsb` do Painter,
// que diz *"EXACT identity … also the hot-path win while dragging"*.
fn adjust_hsb(rgb: vec3<f32>, h: f32, s: f32, b: f32) -> vec3<f32> {
    if h == 0.0 && s == 0.0 && b == 0.0 {
        return rgb;
    }
    let hue_rad = h * 6.2831853072;
    let hc = cos(hue_rad);
    let hs = sin(hue_rad);
    let chroma_scale = max(1.0 + s, 0.0);
    let lab = oklab_from_linear(rgb);
    let ca = (lab.y * hc - lab.z * hs) * chroma_scale;
    let cb = (lab.y * hs + lab.z * hc) * chroma_scale;
    var out = oklab_to_linear(vec3<f32>(lab.x, ca, cb));
    if b > 0.0 {
        out = out + (vec3<f32>(1.0) - out) * b;
    } else if b < 0.0 {
        out = out * (1.0 + b);
    }
    return out;
}
