//! **O Solid com uma ponta de IMAGEM enche-se de CARIMBOS** (smoke do dono 12/10): com uma imagem no
//! Shape (folhas, pétalas, pedras) a mancha lisa na cor do Color apaga exactamente o que o pincel é; o
//! miolo cercado tem de se encher da própria ponta, com o jitter dela (escala, rotação, posição, cor).
//!
//! ⭐ **A lei:** uma grade ALTERNADA (linhas ímpares deslocadas meio passo) ancorada no CANVAS, com o
//! passo do pincel; cada ponto cuja cobertura da região é `≥ ½` vira um [`Dab`] com o SEU sorteio,
//! semeado pela semente do traço e pelas coordenadas inteiras do ponto. Por isso a nuvem é ESTÁVEL
//! enquanto a forma cresce: o Solid re-carimba a mancha a cada evento, e um ponto que já estava dentro
//! sai igual — só os novos aparecem. O empilhamento segue uma chave sorteada do ponto (folhas por cima
//! umas das outras sem a cara de telhado de uma ordem de varrimento).
//!
//! ⚠️ **O passo de ÁREA tem piso de meio diâmetro** ([`PASSO_MINIMO_EM_DIAMETROS`]): o Spacing do pincel
//! responde a *«quão liso é o traço ao longo da linha»*, que uma área não tem. Abaixo de meio diâmetro
//! cada ponto já é coberto por ~4 carimbos; descer daí multiplica o trabalho por `1/s²` sem mudar a
//! imagem de uma ponta opaca (Spacing 0,05 daria 100× os carimbos).

use crate::Dab;
use crate::jitter::{disc_sample, next_f32};
use crate::spec::{BrushSpec, MAX_BRUSH_RADIUS_PX};
use crate::stroke_method::JitterUnit;

/// O piso do passo de área, em diâmetros — ver o doc do módulo.
pub const PASSO_MINIMO_EM_DIAMETROS: f32 = 0.5;

/// A cobertura mínima (`0..=255`) para um ponto da grade estar DENTRO da região.
const DENTRO: u8 = 128;

/// Os carimbos que enchem a região `loops` (já espelhada pela Symmetry) dentro de `w × h`, na ordem de
/// empilhamento. Vazio quando a região não toca a tela. `seed` = a semente do traço.
#[must_use]
pub fn fill_dabs(spec: &BrushSpec, loops: &[Vec<[f32; 2]>], w: u32, h: u32, seed: u64) -> Vec<Dab> {
    let Some([bx, by, bw, bh]) = crate::solid::loops_bbox(loops, w as usize, h as usize) else {
        return Vec::new();
    };
    #[allow(clippy::cast_precision_loss)]
    let origin = [bx as f32, by as f32];
    let cov = crate::solid::fill_coverage(loops, bw, bh, origin);
    let r = spec.clamped_radius();
    let passo = (2.0 * r * spec.spacing.max(PASSO_MINIMO_EM_DIAMETROS)).max(1.0);
    let jitter = spec.stroke_method.allows_jitter();
    let cor_do_traco = crate::stroke::Stroke::stroke_colour_draw(spec, seed);
    #[allow(clippy::cast_possible_truncation)]
    let (j0, j1) = (
        (by as f32 / passo).floor() as i64,
        ((by + bh) as f32 / passo).ceil() as i64,
    );
    let mut out: Vec<(u64, Dab)> = Vec::new();
    for j in j0..=j1 {
        #[allow(clippy::cast_precision_loss)]
        let y = (j as f32 + 0.5) * passo;
        let meio = if j.rem_euclid(2) == 1 { 0.5 } else { 0.0 };
        #[allow(clippy::cast_possible_truncation)]
        let (i0, i1) = (
            (bx as f32 / passo).floor() as i64 - 1,
            ((bx + bw) as f32 / passo).ceil() as i64,
        );
        for i in i0..=i1 {
            #[allow(clippy::cast_precision_loss)]
            let x = (i as f32 + 0.5 + meio) * passo;
            let (lx, ly) = (x - origin[0], y - origin[1]);
            if lx < 0.0 || ly < 0.0 {
                continue;
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let (cx, cy) = (lx as usize, ly as usize);
            if cx >= bw || cy >= bh || cov[cy * bw + cx] < DENTRO {
                continue;
            }
            let mut rng = semente_do_ponto(seed, i, j);
            let chave = rng;
            out.push((
                chave,
                carimbo(spec, [x, y], r, jitter, cor_do_traco, &mut rng),
            ));
        }
    }
    out.sort_by_key(|(k, _)| *k);
    out.into_iter().map(|(_, d)| d).collect()
}

/// O sorteio de UM ponto: escala → rotação → cor → posição, a ordem do `dab_from` do traço.
fn carimbo(
    spec: &BrushSpec,
    p: [f32; 2],
    r: f32,
    jitter: bool,
    cor_do_traco: Option<([f32; 3], [f32; 3])>,
    rng: &mut u64,
) -> Dab {
    let (scale, rotation, mut color, mut color_shift) = if jitter {
        crate::jitter::per_dab(rng, spec)
    } else {
        (1.0, [1.0, 0.0], spec.color, [0.0; 3])
    };
    if let (Some(c), true) = (cor_do_traco, jitter) {
        (color, color_shift) = c;
    }
    let radius = (r * scale).clamp(0.5, MAX_BRUSH_RADIUS_PX);
    let alcance = match spec.jitter_unit {
        JitterUnit::Brush => spec.jitter.clamp(0.0, 1.0) * (2.0 * radius),
        JitterUnit::View => 2.0 * spec.jitter_absolute_px.max(0.0),
    };
    let center = if jitter && alcance > 0.0 {
        let (dx, dy) = disc_sample(rng);
        [p[0] + dx * alcance, p[1] + dy * alcance]
    } else {
        p
    };
    Dab {
        center,
        radius_px: radius,
        coverage: spec.strength.clamp(0.0, 1.0),
        color,
        color_shift,
        rotation,
        dir: [1.0, 0.0],
        arc_len: 0.0,
        stroke_radius_px: r,
        step: [0.0, 0.0],
    }
}

/// O estado do RNG de um ponto da grade: a semente do traço misturada com as coordenadas inteiras,
/// passada uma vez pelo `splitmix` para dois vizinhos não começarem correlacionados.
fn semente_do_ponto(seed: u64, i: i64, j: i64) -> u64 {
    #[allow(clippy::cast_sign_loss)]
    let mut s = seed
        ^ (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (j as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    let _ = next_f32(&mut s);
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::texture::TextureKind;

    fn folha() -> BrushSpec {
        let mut s = BrushSpec::default();
        s.shape.kind = TextureKind::Image;
        s.radius_px = 6.0;
        s.jitter = 0.5;
        s.jitter_scale = 0.3;
        s.jitter_rotate = 1.0;
        s
    }

    fn quadrado(c: f32, m: f32) -> Vec<Vec<[f32; 2]>> {
        vec![vec![
            [c - m, c - m],
            [c + m, c - m],
            [c + m, c + m],
            [c - m, c + m],
        ]]
    }

    /// **A nuvem é ESTÁVEL quando a forma cresce** — o Solid re-carimba a mancha a cada evento; se os
    /// carimbos que já estavam dentro mudassem, as folhas dançariam sob o cursor. Todo carimbo da forma
    /// pequena reaparece AO BIT na grande.
    #[test]
    fn a_nuvem_e_estavel_quando_a_forma_cresce() {
        let s = folha();
        let pequena = fill_dabs(&s, &quadrado(128.0, 30.0), 256, 256, 7);
        let grande = fill_dabs(&s, &quadrado(128.0, 60.0), 256, 256, 7);
        assert!(
            pequena.len() > 20,
            "CONTROLO: a forma pequena tem carimbos ({})",
            pequena.len()
        );
        assert!(grande.len() > pequena.len() * 3);
        for d in &pequena {
            assert!(
                grande.iter().any(|g| g.center == d.center
                    && g.radius_px == d.radius_px
                    && g.rotation == d.rotation),
                "o carimbo em {:?} mudou quando a forma cresceu",
                d.center
            );
        }
    }

    /// O passo de área tem PISO de meio diâmetro: o Spacing 0,05 do traço não multiplica os carimbos.
    #[test]
    fn o_spacing_fino_nao_multiplica_os_carimbos() {
        let mut fino = folha();
        fino.spacing = 0.05;
        let mut piso = folha();
        piso.spacing = PASSO_MINIMO_EM_DIAMETROS;
        let a = fill_dabs(&fino, &quadrado(128.0, 60.0), 256, 256, 7).len();
        let b = fill_dabs(&piso, &quadrado(128.0, 60.0), 256, 256, 7).len();
        assert_eq!(a, b);
    }

    /// Fora da tela não há carimbos; sem região, nenhum.
    #[test]
    fn sem_regiao_nao_ha_carimbos() {
        assert!(fill_dabs(&folha(), &[], 256, 256, 7).is_empty());
        assert!(fill_dabs(&folha(), &quadrado(-500.0, 20.0), 256, 256, 7).is_empty());
    }
}
