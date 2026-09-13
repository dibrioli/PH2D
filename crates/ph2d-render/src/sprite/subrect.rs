//! **QUE PEDAÇO DA TEXTURA UMA SPRITE AMOSTRA** — a célula da folha e a região em pixels, as duas
//! leis que estreitam um rectângulo UV.
//!
//! # ⛔ Por que isto vive no MOTOR, e não na shell
//!
//! Elas nasceram no extract da shell (`render_loop::sim_extract`) e ganharam um **segundo leitor
//! noutra casa**: o retrato de um prefab (`ph2d_app_components::asset_card_portrait`) compõe as
//! peças de uma receita e tem de mostrar **a mesma célula** que a tela mostra. Com essa família fora
//! da shell, uma crate nunca pode chamar o `bin` ⇒ a lei desce para o motor que a executa
//! (`HOWTO_partir_uma_familia_da_shell.md` §2.20, a 2.ª espécie: *uma LEI que duas famílias usam
//! desce para o motor*), ao lado do [`Sprite`](super::Sprite) cujos campos ela lê.
//!
//! ⚠️ **Os corpos mudaram-se VERBATIM** (2026-09-13, `line/components`), com os testes deles; só a
//! visibilidade mudou, de `pub(crate)` para `pub`, porque o segundo leitor está do outro lado de uma
//! fronteira de crate. ⛔ *Uma segunda cópia no retrato faria o cartão e a tela discordarem sobre a
//! célula* — é por isso que há UMA lei com dois leitores, e não duas.

/// Select the sprite-sheet cell `frame` from a base UV rect
/// `[u_min, v_min, u_max, v_max]`, dividing it into an `hframes × vframes`
/// grid (anatomia §03 §3.4). Frame 0 = top-left, `col = frame % hframes`,
/// `row = frame / hframes` (row increases downward, matching V=0 = top).
/// `hframes`/`vframes` floor at 1 and `frame` is clamped into the grid,
/// so the default 1×1 sheet returns the input rect unchanged (no-op for
/// every legacy sprite). Render-only (PresentWorld), HR-5 exempt.
/// ⚠️ **Dois leitores desde 2026-09-01: o RETRATO de um prefab é o segundo.** Ele compõe as
/// peças de uma receita e tem de mostrar **a mesma célula** que a tela mostra — sem isto, uma
/// sprite de folha aparecia no cartão com a grelha inteira espremida na célula.
pub fn sprite_sheet_subrect(uv: [f32; 4], hframes: u32, vframes: u32, frame: u32) -> [f32; 4] {
    let hf = hframes.max(1);
    let vf = vframes.max(1);
    if hf == 1 && vf == 1 {
        return uv;
    }
    let cells = hf.saturating_mul(vf).max(1);
    let frame = frame.min(cells - 1);
    let col = frame % hf;
    let row = frame / hf;
    let [u0, v0, u1, v1] = uv;
    let cw = (u1 - u0) / hf as f32;
    let ch = (v1 - v0) / vf as f32;
    let nu0 = u0 + col as f32 * cw;
    let nv0 = v0 + row as f32 * ch;
    [nu0, nv0, nu0 + cw, nv0 + ch]
}

/// Narrow a UV rect to a sprite's pixel-space `region_rect` (anatomia
/// §03 §3.5). `region` is `[x, y, w, h]` in SOURCE pixels; `(src_w,
/// src_h)` are the source image's pixel dimensions, so the rect maps to
/// the fraction `region / src` of the base `uv`. A zero/negative region
/// or unknown source dims leaves `uv` untouched (region = no-op). When
/// `filter_clip_half_texel` is `Some((htu, htv))`, the result is inset
/// by half a texel per side (Godot `region_filter_clip`) so bilinear
/// sampling can't bleed past the region edge into neighbouring atlas
/// content. The `htu`/`htv` are in the SAMPLED texture's UV space.
/// ⚠️ **Os mesmos dois leitores da irmã acima** — o retrato é o segundo.
pub fn region_subrect(
    uv: [f32; 4],
    region: [f32; 4],
    src_w: f32,
    src_h: f32,
    filter_clip_half_texel: Option<(f32, f32)>,
) -> [f32; 4] {
    let [u0, v0, u1, v1] = uv;
    let [rx, ry, rw, rh] = region;
    if rw <= 0.0 || rh <= 0.0 || src_w <= 0.0 || src_h <= 0.0 {
        return uv;
    }
    let du = u1 - u0;
    let dv = v1 - v0;
    // Region beyond the source edges is clamped to the base rect.
    let mut nu0 = (u0 + du * (rx / src_w)).clamp(u0, u1);
    let mut nv0 = (v0 + dv * (ry / src_h)).clamp(v0, v1);
    let mut nu1 = (u0 + du * ((rx + rw) / src_w)).clamp(u0, u1);
    let mut nv1 = (v0 + dv * ((ry + rh) / src_h)).clamp(v0, v1);
    if let Some((htu, htv)) = filter_clip_half_texel {
        nu0 += htu;
        nv0 += htv;
        nu1 -= htu;
        nv1 -= htv;
        // A region thinner than one texel would invert under the inset;
        // collapse it to its center so the sample stays inside.
        if nu1 < nu0 {
            let m = 0.5 * (nu0 + nu1);
            nu0 = m;
            nu1 = m;
        }
        if nv1 < nv0 {
            let m = 0.5 * (nv0 + nv1);
            nv0 = m;
            nv1 = m;
        }
    }
    [nu0, nv0, nu1, nv1]
}

#[cfg(test)]
#[path = "subrect_sprite_sheet_tests.rs"]
mod sprite_sheet_tests;

#[cfg(test)]
#[path = "subrect_region_tests.rs"]
mod region_subrect_tests;
