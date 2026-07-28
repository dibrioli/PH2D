// **AS 22 LEIS DE MISTURA — W3C Compositing and Blending Level 1.**
//
// Extraído do `layer_composite.wgsl` quando ganhou o SEGUNDO consumidor (a pilha de FX raster do
// módulo vetorial, plano 24 W6). O bloco sempre foi auto-contido — só builtins do WGSL, nenhum
// binding, nenhum global — e é por isso que a extração é movimento de código PURO.
//
// ⚠️ **Uma resposta, dois consumidores.** *Como duas cores se combinam* é uma pergunta que este
// repositório já respondeu, e a resposta está pinada bit a bit contra o Rust
// (`shader_blend_modes_bit_identical_with_rust`, que lê a CONCATENAÇÃO). Uma segunda cópia no
// shader do FX divergiria no único lugar onde ninguém lê um número: uma captura de tela.
//
// **A convenção é RETA e LINEAR** (`cb`/`cs` em `[0,1]`), nunca premultiplicada — quem chama de um
// pipeline premultiplicado divide pelo alfa na entrada e multiplica na saída. Os dois chamadores
// fazem exatamente isso, cada um na própria fronteira.
//
// ⚠️ Este arquivo **não parseia sozinho** de propósito: ele é o PREFIXO de um módulo, e quem o
// compõe é a porta de cada consumidor. Um `include_str!` que o usasse solto seria um terceiro
// caminho, e é o terceiro caminho que diverge.

const F32_EPSILON: f32 = 1.1920929e-7; // f32::EPSILON (2^-23), mirror of Rust

// ── Separable blend functions — W3C Compositing L1 §9 ────────────────────
fn screen(cb: f32, cs: f32) -> f32 {
    return cb + cs - cb * cs;
}

fn hard_light(cb: f32, cs: f32) -> f32 {
    if cs <= 0.5 {
        return 2.0 * cb * cs;
    }
    return 1.0 - 2.0 * (1.0 - cb) * (1.0 - cs);
}

fn color_dodge(cb: f32, cs: f32) -> f32 {
    if cb <= 0.0 {
        return 0.0;
    }
    if cs >= 1.0 {
        return 1.0;
    }
    return min(cb / (1.0 - cs), 1.0);
}

fn color_burn(cb: f32, cs: f32) -> f32 {
    if cb >= 1.0 {
        return 1.0;
    }
    if cs <= 0.0 {
        return 0.0;
    }
    return 1.0 - min((1.0 - cb) / cs, 1.0);
}

fn soft_light(cb: f32, cs: f32) -> f32 {
    if cs <= 0.5 {
        return cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb);
    }
    var d: f32;
    if cb <= 0.25 {
        d = ((16.0 * cb - 12.0) * cb + 4.0) * cb;
    } else {
        d = sqrt(cb);
    }
    return cb + (2.0 * cs - 1.0) * (d - cb);
}

fn blend_sep(mode: u32, cb: f32, cs: f32) -> f32 {
    switch mode {
        case 0u: { return cs; }                                  // Normal
        case 1u: { return cb * cs; }                             // Multiply
        case 2u: { return min(cb, cs); }                         // Darken
        case 3u: { return color_burn(cb, cs); }                  // ColorBurn
        case 4u: { return clamp(cb + cs - 1.0, 0.0, 1.0); }      // LinearBurn
        case 5u: { return max(cb, cs); }                         // Lighten
        case 6u: { return screen(cb, cs); }                      // Screen
        case 7u: { return color_dodge(cb, cs); }                 // ColorDodge
        case 8u: { return min(cb + cs, 1.0); }                   // Add (LinearDodge)
        case 9u: { return hard_light(cs, cb); }                  // Overlay
        case 10u: { return soft_light(cb, cs); }                 // SoftLight
        case 11u: { return hard_light(cb, cs); }                 // HardLight
        case 12u: {                                              // VividLight
            if cs <= 0.5 {
                return color_burn(cb, clamp(2.0 * cs, 0.0, 1.0));
            }
            return color_dodge(cb, clamp(2.0 * cs - 1.0, 0.0, 1.0));
        }
        case 13u: { return clamp(cb + 2.0 * cs - 1.0, 0.0, 1.0); } // LinearLight
        case 14u: { return abs(cb - cs); }                       // Difference
        case 15u: { return cb + cs - 2.0 * cb * cs; }            // Exclusion
        default: { return cs; }
    }
}

// ── Non-separable (HSL) blend functions — W3C Compositing L1 §10 ──────────
fn lum(c: vec3<f32>) -> f32 {
    return 0.3 * c.r + 0.59 * c.g + 0.11 * c.b;
}

fn clip_color(c: vec3<f32>) -> vec3<f32> {
    let l = lum(c);
    let n = min(c.r, min(c.g, c.b));
    let x = max(c.r, max(c.g, c.b));
    var out = c;
    if n < 0.0 {
        let denom = l - n;
        if abs(denom) > F32_EPSILON {
            out = l + (out - l) * l / denom;
        }
    }
    if x > 1.0 {
        let denom = x - l;
        if abs(denom) > F32_EPSILON {
            out = l + (out - l) * (1.0 - l) / denom;
        }
    }
    return out;
}

fn set_lum(c: vec3<f32>, l: f32) -> vec3<f32> {
    let d = l - lum(c);
    return clip_color(c + vec3<f32>(d));
}

fn sat(c: vec3<f32>) -> f32 {
    return max(c.r, max(c.g, c.b)) - min(c.r, min(c.g, c.b));
}

// W3C `SetSat`: stretch the mid/max channels to saturation `s`, preserving
// the original min→max channel ORDER. Mirrors the Rust `set_sat`, which sorts
// the three channels by value with a *stable* comparator (ties keep original
// R<G<B order). Implemented with SCALARS (no local arrays) so it adds no
// register/stack pressure to the kernel — keeping occupancy (and thus memory-
// latency hiding) high on the hot path. `rank_*` is each channel's position in
// the stable ascending sort (0 = min, 1 = mid, 2 = max); ties break R<G<B,
// matching Rust's stable sort, via the `<` / `<=` asymmetry below.
fn set_sat(c: vec3<f32>, s: f32) -> vec3<f32> {
    // Stable ascending rank of each channel. For a tie, the earlier channel
    // (R<G<B) ranks lower — encoded by using `<` against earlier channels and
    // `<=`... we instead count, for each channel, how many channels are
    // strictly below it, plus earlier channels that are equal.
    let r = c.r;
    let g = c.g;
    let b = c.b;
    let rank_r = u32(g < r) + u32(b < r);
    let rank_g = u32(r <= g) + u32(b < g);
    let rank_b = u32(r <= b) + u32(g <= b);
    let cmin = min(r, min(g, b));
    let cmax = max(r, max(g, b));
    // mid = the value whose rank is 1.
    var cmid = r;
    if rank_g == 1u { cmid = g; }
    if rank_b == 1u { cmid = b; }

    var out_min = 0.0;
    var out_mid = 0.0;
    var out_max = 0.0;
    if cmax > cmin {
        out_mid = (cmid - cmin) * s / (cmax - cmin);
        out_max = s;
    }
    // Scatter the three rank-slots back to the original channels.
    let or = select(select(out_min, out_mid, rank_r == 1u), out_max, rank_r == 2u);
    let og = select(select(out_min, out_mid, rank_g == 1u), out_max, rank_g == 2u);
    let ob = select(select(out_min, out_mid, rank_b == 1u), out_max, rank_b == 2u);
    return vec3<f32>(or, og, ob);
}

fn blend_hsl(mode: u32, cb: vec3<f32>, cs: vec3<f32>) -> vec3<f32> {
    switch mode {
        case 16u: { return set_lum(set_sat(cs, sat(cb)), lum(cb)); } // Hue
        case 17u: { return set_lum(set_sat(cb, sat(cs)), lum(cb)); } // Saturation
        case 18u: { return set_lum(cs, lum(cb)); }                   // Color
        case 19u: { return set_lum(cb, lum(cs)); }                   // Luminosity
        default: { return cs; }
    }
}

fn is_hsl(mode: u32) -> bool {
    return mode == 16u || mode == 17u || mode == 18u || mode == 19u;
}
