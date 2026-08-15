//! Drawing **symmetry** — replicate every dab across a mirror axis or as N radial copies.
//!
//! When the artist paints a dab at `P`, symmetry stamps extra dabs at the geometric images of `P`:
//! one reflection across a line (Mirror X / Y / a custom line) or `N − 1` rotations about a centre
//! (radial / "circular", 3..=12 segments). It is the drawing analogue of seamless Tiling — Tiling
//! wraps the texture coordinate; symmetry replicates the dab *footprint* itself.
//!
//! The transform operates on a finished [`Dab`] (canvas-pixel `center`, plus the `rotation` and `dir`
//! unit vectors), so it sits at the dab-emission chokepoint in [`crate::stroke`] and is oblivious to
//! the stroke method, pressure, jitter and dash that already shaped the base dab.
//!
//! HR-5 (transcendental-free): reflection and rotation are dot-products and complex multiplies only.
//! The per-segment radial rotor for each `N` is a **baked unit-vector table** ([`radial_rotor`]) — the
//! cosine/sine literals are authored, never evaluated at runtime — because `360 / N` is not a whole
//! number of degrees for `N ∈ {7, 11}`, so stepping a 1° rotor would fail to close the ring. `sqrt`
//! (for normalising a custom axis direction) is the one allowed root, as elsewhere in the engine.

use crate::stroke::Dab;

/// Fewest radial copies the UI slider reaches (a 3-fold rosette).
pub const SYMMETRY_MIN_SEGMENTS: u32 = 3;
/// Most radial copies the UI slider reaches (a 12-fold rosette).
pub const SYMMETRY_MAX_SEGMENTS: u32 = 12;

/// Which mirror line a non-circular symmetry reflects across.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MirrorAxis {
    /// Reflect left↔right across a **vertical** line through [`SymmetrySettings::center`]
    /// (the "X" symmetry artists expect: it mirrors the X coordinate).
    #[default]
    X,
    /// Reflect top↔bottom across a **horizontal** line through [`SymmetrySettings::center`].
    Y,
    /// Reflect across the arbitrary line through [`SymmetrySettings::center`] with direction
    /// [`SymmetrySettings::custom_dir`] (the line the artist draws on the canvas).
    Custom,
}

impl MirrorAxis {
    /// Wire discriminant for the panel snapshot (`0` = X, `1` = Y, `2` = Custom).
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            MirrorAxis::X => 0,
            MirrorAxis::Y => 1,
            MirrorAxis::Custom => 2,
        }
    }

    /// Decode a wire discriminant (out-of-range → `X`).
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => MirrorAxis::Y,
            2 => MirrorAxis::Custom,
            _ => MirrorAxis::X,
        }
    }
}

/// Drawing-symmetry parameters, read by the stroke engine once per emitted dab. `Copy` so it rides
/// inside [`crate::BrushSpec`] without breaking its alloc-free `Copy` contract. All geometry is in
/// **canvas pixels** and is resolved by the tool (the centre defaults to the canvas centre; the
/// canvas "draw line" / "pick centre" modes overwrite it).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SymmetrySettings {
    /// Master toggle. When `false`, [`push_symmetric`] emits only the base dab, so painting is
    /// **byte-identical** to having no symmetry at all.
    pub enabled: bool,
    /// `false` = mirror across [`Self::axis`]; `true` = radial / circular ([`Self::radial_segments`]
    /// rotational copies).
    pub circular: bool,
    /// The mirror line, used when `!circular`.
    pub axis: MirrorAxis,
    /// Number of rotational copies when `circular`, clamped to
    /// `[SYMMETRY_MIN_SEGMENTS, SYMMETRY_MAX_SEGMENTS]`. The total dab count is this value (the base
    /// dab is copy `0`).
    pub radial_segments: u32,
    /// A point ON the mirror line / the radial centre, in canvas pixels.
    pub center: [f32; 2],
    /// Unit direction of the custom mirror line (used only when `axis == Custom`). Normalised
    /// defensively in [`push_symmetric`]; a degenerate `[0, 0]` falls back to a vertical line.
    pub custom_dir: [f32; 2],
}

impl Default for SymmetrySettings {
    /// Disabled — so a default brush paints exactly as before symmetry existed.
    fn default() -> Self {
        Self {
            enabled: false,
            circular: false,
            axis: MirrorAxis::X,
            radial_segments: 6,
            center: [0.0, 0.0],
            custom_dir: [0.0, 1.0],
        }
    }
}

impl SymmetrySettings {
    /// The radial segment count clamped to the UI range.
    #[must_use]
    pub fn segments(&self) -> u32 {
        self.radial_segments
            .clamp(SYMMETRY_MIN_SEGMENTS, SYMMETRY_MAX_SEGMENTS)
    }

    /// **Quantos dabs [`push_symmetric`] emite por dab de ORIGEM** — `1` desligada, `2` no espelho,
    /// [`Self::segments`] no radial.
    ///
    /// ⚠️ **Ela existe para se poder VOLTAR do lote à raia primária**, e é isso que o `Style: Solid`
    /// consome: o preenchimento é o polígono do caminho da TINTA, e o lote que chega ao carimbo já
    /// traz as cópias interleavadas. Como [`push_symmetric`] empurra a ORIGEM primeiro e as cópias
    /// depois, o elemento `i · copies()` de um lote é sempre um primário — a propriedade que o gate
    /// `the_source_dab_is_pushed_first_and_the_copies_follow` pina, para ninguém a inferir de novo do
    /// corpo da função.
    #[must_use]
    pub fn copies(&self) -> usize {
        if !self.enabled {
            return 1;
        }
        if self.circular {
            self.segments() as usize
        } else {
            2
        }
    }

    /// Unit direction of the mirror line for the current [`Self::axis`]: vertical for `X`, horizontal
    /// for `Y`, the (normalised) [`Self::custom_dir`] for `Custom` (falling back to vertical if it is
    /// degenerate). Used by [`push_symmetric`] and by the overlay to draw the axis.
    #[must_use]
    pub fn mirror_dir(&self) -> [f32; 2] {
        match self.axis {
            MirrorAxis::X => [0.0, 1.0],
            MirrorAxis::Y => [1.0, 0.0],
            MirrorAxis::Custom => normalize_or(self.custom_dir, [0.0, 1.0]),
        }
    }
}

/// Push the base `dab` into `out`, then — when symmetry is enabled — its mirror/radial copies.
///
/// The single chokepoint the stroke engine routes every primary dab emission through. Disabled (or a
/// 1-segment radial) ⇒ only the base dab, preserving the no-symmetry pixel output exactly.
///
/// ⚠️ **`pub` porque o TOOL também emite dabs que o motor não viu** — a corda de fechamento do
/// `Style: Solid`, que liga o fim do caminho ao começo dele. Ela tem de cair nas mesmas cópias que os
/// dabs do gesto, e re-derivar o espelho lá seria a **segunda resposta** a *"onde cai a cópia deste
/// dab?"*, divergindo desta no dia em que um dos dois ganhasse um eixo novo.
pub fn push_symmetric(out: &mut Vec<Dab>, dab: Dab, sym: &SymmetrySettings) {
    out.push(dab);
    if !sym.enabled {
        return;
    }
    if sym.circular {
        let n = sym.segments();
        // Per-segment rotor; compose it onto itself to walk copies 1..n around the centre.
        let step = radial_rotor(n);
        let mut rotor = step;
        for _ in 1..n {
            out.push(rotate_dab_about(dab, sym.center, rotor));
            rotor = crate::heading::rotate(rotor, step);
        }
    } else {
        out.push(reflect_dab(dab, sym.center, sym.mirror_dir()));
    }
}

/// Push the base `seg` into `out`, then — quando a simetria está ligada — as cópias dele.
///
/// ⚠️ **Irmã de [`push_symmetric`], no MESMO módulo e sobre os MESMOS primitivos** ([`reflect_vec`] /
/// [`radial_rotor`]), porque um fio do Sketchy e o dab que o gerou têm de acabar no mesmo lugar em
/// toda cópia. Um segmento não é um dab (ele não tem raio, cobertura nem orientação de carimbo), mas
/// a LEI de onde as cópias caem é uma só — e é por isso que as duas funções são vizinhas e há um gate
/// exigindo que o fio espelhado ligue os centros dos dabs espelhados.
pub(crate) fn push_symmetric_segment(
    out: &mut Vec<[f32; 4]>,
    seg: [f32; 4],
    sym: &SymmetrySettings,
) {
    out.push(seg);
    if !sym.enabled {
        return;
    }
    let map = |f: &dyn Fn([f32; 2]) -> [f32; 2]| {
        let a = f([seg[0], seg[1]]);
        let b = f([seg[2], seg[3]]);
        [a[0], a[1], b[0], b[1]]
    };
    if sym.circular {
        let n = sym.segments();
        let step = radial_rotor(n);
        let mut rotor = step;
        for _ in 1..n {
            out.push(map(&|p| {
                let v = [p[0] - sym.center[0], p[1] - sym.center[1]];
                let rv = crate::heading::rotate(v, rotor);
                [sym.center[0] + rv[0], sym.center[1] + rv[1]]
            }));
            rotor = crate::heading::rotate(rotor, step);
        }
    } else {
        let d = sym.mirror_dir();
        out.push(map(&|p| {
            let v = [p[0] - sym.center[0], p[1] - sym.center[1]];
            let r = reflect_vec(v, d);
            [sym.center[0] + r[0], sym.center[1] + r[1]]
        }));
    }
}

/// As cópias de um conjunto de **LAÇOS FECHADOS** — a terceira irmã de [`push_symmetric`] (dabs) e
/// [`push_symmetric_segment`] (fios), sobre os MESMOS primitivos ([`reflect_vec`] / [`radial_rotor`]).
///
/// ⚠️ **O WINDING é REPOSTO, e isto é correção e não higiene.** O preenchimento sólido é `nonzero`
/// (`|soma corrida|.min(1)`), então dois contornos de sentidos OPOSTOS se furam um ao outro — é
/// exatamente assim que o `solid_shapes` desenha um `Remove`. Uma **reflexão inverte o sinal da
/// área**: sem repor o sentido, um espelho cujo eixo cruza a figura devolveria uma cópia que
/// **CANCELA** a original na sobreposição e abriria um buraco onde tem de haver tinta. Uma
/// **rotação preserva** o sinal, então só o braço do espelho reverte a lista de pontos.
///
/// Desligada ⇒ devolve os laços de entrada **na ordem e nos valores de entrada**, o que mantém o
/// mundo sem simetria byte-idêntico.
#[must_use]
pub fn symmetric_loops(loops: &[Vec<[f32; 2]>], sym: &SymmetrySettings) -> Vec<Vec<[f32; 2]>> {
    if !sym.enabled || loops.is_empty() {
        return loops.to_vec();
    }
    let mut out = Vec::with_capacity(loops.len() * 2);
    out.extend_from_slice(loops);
    if sym.circular {
        let n = sym.segments();
        let step = radial_rotor(n);
        let mut rotor = step;
        for _ in 1..n {
            for lp in loops {
                out.push(
                    lp.iter()
                        .map(|p| {
                            let v = [p[0] - sym.center[0], p[1] - sym.center[1]];
                            let rv = crate::heading::rotate(v, rotor);
                            [sym.center[0] + rv[0], sym.center[1] + rv[1]]
                        })
                        .collect(),
                );
            }
            rotor = crate::heading::rotate(rotor, step);
        }
    } else {
        let d = sym.mirror_dir();
        for lp in loops {
            // Espelha os pontos **e** inverte a ordem: a reflexão trocou o sinal da área, e a ordem
            // invertida o traz de volta ao da fonte (ver o ⚠️ do doc).
            let mut m: Vec<[f32; 2]> = lp
                .iter()
                .rev()
                .map(|p| {
                    let v = [p[0] - sym.center[0], p[1] - sym.center[1]];
                    let r = reflect_vec(v, d);
                    [sym.center[0] + r[0], sym.center[1] + r[1]]
                })
                .collect();
            m.shrink_to_fit();
            out.push(m);
        }
    }
    out
}

/// Reflect a dab across the line through `a` with unit direction `d`: the centre is mirrored, and the
/// orientation vectors (`rotation`, `dir`) are reflected too so a mirror image has the flipped
/// handedness a real reflection would. Radius / coverage / colour are untouched.
fn reflect_dab(dab: Dab, a: [f32; 2], d: [f32; 2]) -> Dab {
    let v = [dab.center[0] - a[0], dab.center[1] - a[1]];
    let r = reflect_vec(v, d);
    Dab {
        center: [a[0] + r[0], a[1] + r[1]],
        rotation: reflect_vec(dab.rotation, d),
        dir: reflect_vec(dab.dir, d),
        ..dab
    }
}

/// Rotate a dab about `c` by the unit rotor `r`: the centre orbits `c`, and the orientation vectors
/// spin by the same rotor so the stamped tip follows the rotated copy.
fn rotate_dab_about(dab: Dab, c: [f32; 2], r: [f32; 2]) -> Dab {
    let v = [dab.center[0] - c[0], dab.center[1] - c[1]];
    let rv = crate::heading::rotate(v, r);
    Dab {
        center: [c[0] + rv[0], c[1] + rv[1]],
        rotation: crate::heading::rotate(dab.rotation, r),
        dir: crate::heading::rotate(dab.dir, r),
        ..dab
    }
}

/// Reflect vector `v` across the line spanned by unit vector `d`: `2·(v·d)·d − v`. Transcendental-free.
#[inline]
fn reflect_vec(v: [f32; 2], d: [f32; 2]) -> [f32; 2] {
    let dot = v[0] * d[0] + v[1] * d[1];
    [2.0 * dot * d[0] - v[0], 2.0 * dot * d[1] - v[1]]
}

/// Normalise `v`, or return `fallback` when `v` is ~zero (so a degenerate custom axis is well-defined).
#[inline]
fn normalize_or(v: [f32; 2], fallback: [f32; 2]) -> [f32; 2] {
    let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if len < 1e-6 {
        fallback
    } else {
        [v[0] / len, v[1] / len]
    }
}

/// The unit rotor for one radial segment of an `n`-fold rosette — `[cos(2π/n), sin(2π/n)]` as a
/// **baked literal** (HR-5: never evaluated at runtime). `n` outside `3..=12` is clamped first, so
/// every arm of the match is reachable only for an in-range count.
fn radial_rotor(n: u32) -> [f32; 2] {
    // `FRAC_1_SQRT_2` is a compile-time constant (still HR-5 transcendental-free at runtime); the rest
    // are authored cos/sin literals at f32 precision.
    use std::f32::consts::FRAC_1_SQRT_2;
    match n.clamp(SYMMETRY_MIN_SEGMENTS, SYMMETRY_MAX_SEGMENTS) {
        3 => [-0.5, 0.866_025_4],            // 120°
        4 => [0.0, 1.0],                     // 90°
        5 => [0.309_017, 0.951_056_5],       // 72°
        6 => [0.5, 0.866_025_4],             // 60°
        7 => [0.623_489_8, 0.781_831_5],     // 360/7 ≈ 51.43°
        8 => [FRAC_1_SQRT_2, FRAC_1_SQRT_2], // 45° (cos = sin = 1/√2)
        9 => [0.766_044_4, 0.642_787_6],     // 40°
        10 => [0.809_017, 0.587_785_2],      // 36°
        11 => [0.841_253_5, 0.540_640_8],    // 360/11 ≈ 32.73°
        _ => [0.866_025_4, 0.5],             // n == 12: 30°
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_dab() -> Dab {
        Dab {
            center: [60.0, 20.0],
            radius_px: 10.0,
            coverage: 0.5,
            color: [0.1, 0.2, 0.3],
            rotation: [1.0, 0.0],
            dir: [1.0, 0.0],
            arc_len: 0.0,
            stroke_radius_px: 10.0,
        }
    }

    #[test]
    fn disabled_is_byte_identical_single_dab() {
        let mut out = Vec::new();
        let sym = SymmetrySettings::default();
        assert!(!sym.enabled);
        push_symmetric(&mut out, base_dab(), &sym);
        assert_eq!(out.len(), 1, "disabled symmetry emits only the base dab");
        assert_eq!(out[0], base_dab());
    }

    #[test]
    fn mirror_x_reflects_across_vertical_line_through_centre() {
        let mut out = Vec::new();
        let sym = SymmetrySettings {
            enabled: true,
            axis: MirrorAxis::X,
            center: [50.0, 0.0],
            ..Default::default()
        };
        push_symmetric(&mut out, base_dab(), &sym);
        assert_eq!(out.len(), 2);
        // base.center.x = 60, centre.x = 50 → mirror at 40; y unchanged.
        assert_eq!(out[1].center, [40.0, 20.0]);
        // Orientation flips handedness: a +x rotor reflected across a vertical line stays +x; the
        // dir reflected across vertical line [0,1] flips x. Check the y-axis mirror below for the flip.
        assert_eq!(out[1].radius_px, base_dab().radius_px);
        assert_eq!(out[1].coverage, base_dab().coverage);
    }

    #[test]
    fn mirror_y_reflects_across_horizontal_line() {
        let mut out = Vec::new();
        let sym = SymmetrySettings {
            enabled: true,
            axis: MirrorAxis::Y,
            center: [0.0, 50.0],
            ..Default::default()
        };
        push_symmetric(&mut out, base_dab(), &sym);
        // base.center.y = 20, centre.y = 50 → mirror at 80; x unchanged.
        assert_eq!(out[1].center, [60.0, 80.0]);
        // dir [1,0] reflected across horizontal line [1,0] is unchanged; rotation likewise.
        assert_eq!(out[1].dir, [1.0, 0.0]);
    }

    #[test]
    fn mirror_custom_reflects_across_drawn_line() {
        let mut out = Vec::new();
        // 45° line through origin: reflection swaps x and y.
        let sym = SymmetrySettings {
            enabled: true,
            axis: MirrorAxis::Custom,
            center: [0.0, 0.0],
            custom_dir: [1.0, 1.0], // not unit — must be normalised internally
            ..Default::default()
        };
        push_symmetric(&mut out, base_dab(), &sym);
        let c = out[1].center;
        assert!(
            (c[0] - 20.0).abs() < 1e-3 && (c[1] - 60.0).abs() < 1e-3,
            "swapped across 45°: {c:?}"
        );
    }

    #[test]
    fn radial_emits_n_dabs_around_centre_and_closes() {
        for n in SYMMETRY_MIN_SEGMENTS..=SYMMETRY_MAX_SEGMENTS {
            let mut out = Vec::new();
            let sym = SymmetrySettings {
                enabled: true,
                circular: true,
                radial_segments: n,
                center: [0.0, 0.0],
                ..Default::default()
            };
            // A dab on the +x axis at radius 100.
            let d = Dab {
                center: [100.0, 0.0],
                ..base_dab()
            };
            push_symmetric(&mut out, d, &sym);
            assert_eq!(out.len() as u32, n, "radial {n} emits n dabs");
            // Every copy is the same distance from the centre (rotation preserves radius).
            for dab in &out {
                let r = (dab.center[0] * dab.center[0] + dab.center[1] * dab.center[1]).sqrt();
                assert!(
                    (r - 100.0).abs() < 1e-2,
                    "n={n}: copy off the orbit: {:?}",
                    dab.center
                );
            }
        }
    }

    #[test]
    fn radial_four_lands_on_the_axes() {
        let mut out = Vec::new();
        let sym = SymmetrySettings {
            enabled: true,
            circular: true,
            radial_segments: 4,
            center: [0.0, 0.0],
            ..Default::default()
        };
        let d = Dab {
            center: [10.0, 0.0],
            ..base_dab()
        };
        push_symmetric(&mut out, d, &sym);
        assert_eq!(out.len(), 4);
        // 0°, 90°, 180°, 270° → (10,0), (0,10), (-10,0), (0,-10).
        let near =
            |a: [f32; 2], b: [f32; 2]| (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3;
        assert!(near(out[0].center, [10.0, 0.0]));
        assert!(near(out[1].center, [0.0, 10.0]));
        assert!(near(out[2].center, [-10.0, 0.0]));
        assert!(near(out[3].center, [0.0, -10.0]));
    }

    /// Área com sinal (sapateiro). Só o SINAL é lido, então a falta do fator ½ não importa.
    fn signed_area(pts: &[[f32; 2]]) -> f32 {
        let n = pts.len();
        let mut acc = 0.0;
        for i in 0..n {
            let (a, b) = (pts[i], pts[(i + 1) % n]);
            acc += a[0] * b[1] - b[0] * a[1];
        }
        acc
    }

    /// Um quadrado anti-horário longe do eixo.
    fn square() -> Vec<[f32; 2]> {
        vec![[10.0, 10.0], [30.0, 10.0], [30.0, 30.0], [10.0, 30.0]]
    }

    #[test]
    fn loops_disabled_are_the_input_verbatim() {
        let src = vec![square()];
        let out = symmetric_loops(&src, &SymmetrySettings::default());
        assert_eq!(out, src, "sem simetria o conjunto tem de sair como entrou");
    }

    /// **UMA CÓPIA ESPELHADA MANTÉM O SENTIDO DA FONTE** — a propriedade que impede o buraco.
    ///
    /// ⚠️ O preenchimento é `nonzero`: sentidos opostos se cancelam na sobreposição. Uma reflexão
    /// inverte o sinal da área, então a cópia só é somável se a ordem dos pontos for revertida.
    ///
    /// **Mutação que sangra:** tirar o `.rev()` do braço do espelho (o sinal da cópia inverte).
    #[test]
    fn a_mirrored_loop_keeps_the_winding_of_its_source() {
        let src = vec![square()];
        let sym = SymmetrySettings {
            enabled: true,
            axis: MirrorAxis::X,
            center: [0.0, 0.0],
            ..Default::default()
        };
        let out = symmetric_loops(&src, &sym);
        assert_eq!(out.len(), 2, "espelho = fonte + uma cópia");
        let (a, b) = (signed_area(&out[0]), signed_area(&out[1]));
        assert!(
            a * b > 0.0,
            "a cópia espelhada trocou de sentido (fonte {a:.1}, cópia {b:.1}) -- o `nonzero` vai \
             abrir um buraco onde ela cruzar a fonte"
        );
        // E ela de facto foi para o outro lado do eixo (senão o gate acima é vácuo).
        assert!(
            out[1].iter().all(|p| p[0] < 0.0),
            "não espelhou: {:?}",
            out[1]
        );
    }

    /// **E O BURACO É MEDIDO, não deduzido** — um espelho cujo eixo CRUZA a figura tem de a deixar
    /// cheia. É o oráculo de APARÊNCIA da propriedade acima, sobre o rasterizador que a consome.
    #[test]
    fn a_mirror_through_the_shape_does_not_punch_a_hole_in_it() {
        let src = vec![square()];
        let sym = SymmetrySettings {
            enabled: true,
            axis: MirrorAxis::X,
            center: [20.0, 0.0], // o eixo passa pelo MEIO do quadrado 10..30
            ..Default::default()
        };
        let out = symmetric_loops(&src, &sym);
        let cov = crate::solid::fill_coverage(&out, 40, 40, [0.0, 0.0]);
        // O miolo, onde a fonte e a cópia se sobrepõem, tem de estar CHEIO.
        for y in 12..28 {
            for x in 12..28 {
                assert_eq!(
                    cov[y * 40 + x],
                    255,
                    "buraco em ({x},{y}): a cópia espelhada cancelou a fonte"
                );
            }
        }
    }

    /// **A cópia RADIAL não reverte, e isso também é medido** — uma rotação preserva o sinal, então
    /// reverter ali seria a inversão que o espelho existe para desfazer.
    #[test]
    fn radial_loops_keep_the_winding_without_reversing() {
        let src = vec![square()];
        let sym = SymmetrySettings {
            enabled: true,
            circular: true,
            radial_segments: 4,
            center: [0.0, 0.0],
            ..Default::default()
        };
        let out = symmetric_loops(&src, &sym);
        assert_eq!(out.len(), 4, "radial 4 = quatro laços");
        let a0 = signed_area(&out[0]);
        for (i, lp) in out.iter().enumerate() {
            assert!(
                signed_area(lp) * a0 > 0.0,
                "a cópia radial {i} trocou de sentido"
            );
        }
    }

    #[test]
    fn segment_count_is_clamped() {
        let lo = SymmetrySettings {
            radial_segments: 1,
            ..Default::default()
        };
        assert_eq!(lo.segments(), SYMMETRY_MIN_SEGMENTS);
        let hi = SymmetrySettings {
            radial_segments: 99,
            ..Default::default()
        };
        assert_eq!(hi.segments(), SYMMETRY_MAX_SEGMENTS);
    }
}
