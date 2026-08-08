//! Deform **displacement fields** — the per-mode `D(p)` fed to the single inverse-warp kernel
//! ([`super::apply`]). Every sub-mode is the SAME backward-gather kernel (`out[dst] = sample(dst − D)`);
//! only `D` changes here. Reconstruct has no field (it resamples the pre-deform buffer, [`super::reconstruct`]).
//!
//! **HR-5 (transcendental-free).** The Twist rotor is read from a per-dab baked **1° table** (iterated
//! complex-multiply of the compile-time `cos 1° / sin 1°` step), exactly like `ph2d_painter_brush`'s
//! `texture::rotate_by_degrees` — never a runtime `sin`/`cos`. Everything else is `+ − × ÷ floor`.

/// `cos 1°` / `sin 1°` — the baked rotation step (compile-time constants; the runtime never calls a
/// transcendental, mirroring `ph2d_painter_brush`'s `DEG_STEP`).
const COS_1DEG: f32 = 0.999_847_7;
const SIN_1DEG: f32 = 0.017_452_406;

/// **Per-dab** Twist angle at full pressure, in degrees (a UI-feel "reach" scalar). Dabs accumulate over
/// the session, so this is a small increment. Tuned 1/4 down from the first playtest (Enio 2026-07-04).
const TWIST_MAX_DEG: f32 = 5.0;
/// **Per-dab** radial gain at full pressure, per mode (Pinch/Wrinkle = 1/4, Fold = 1/2 of the first
/// playtest's `0.10` — Enio 2026-07-04). Accumulated over the session.
const PINCH_GAIN: f32 = 0.025;
const WRINKLE_GAIN: f32 = 0.025;
const FOLD_GAIN: f32 = 0.05;
/// Coherent-noise feature size in px — the lattice cell for [`value_noise`]. Big enough that the
/// turbulence reads as smooth warble, not per-pixel grain (that grain was the reported "scatter").
const NOISE_CELL_PX: f32 = 18.0;
/// Peak Push turbulence as a fraction of the brush **radius** at full Distortion (radius-scaled, so it's
/// stable at slow drags — decoupled from stroke speed).
const PUSH_TURB: f32 = 0.25;
/// Pinch/Fold gain-noise depth at full Distortion (fraction of the radial gain).
const PINCH_TURB: f32 = 0.5;
/// **Wrinkle**'s intrinsic crinkle depth — always on (its defining feature; NOT gated by Distortion).
const WRINKLE_NOISE: f32 = 0.7;

/// The Deform sub-mode (wire discriminant = the panel segmented index). `Reconstruct` carries no field.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum DeformMode {
    Push,
    Twist,
    Pinch,
    Wrinkle,
    Fold,
    Reconstruct,
}

impl DeformMode {
    pub(super) fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Twist,
            2 => Self::Pinch,
            3 => Self::Wrinkle,
            4 => Self::Fold,
            5 => Self::Reconstruct,
            _ => Self::Push,
        }
    }
}

/// Smooth radial falloff `f` for a squared normalised distance `t2 = (dist/radius)²` — a C¹ bell,
/// `1` at the centre → `0` at the edge (`(1 − t²)²`). Takes `t2` so the kernel avoids a per-pixel `sqrt`.
#[inline]
pub(super) fn falloff(t2: f32) -> f32 {
    if t2 >= 1.0 {
        0.0
    } else {
        let s = 1.0 - t2;
        s * s
    }
}

/// splitmix64-hashed value in `[0, 1)` at the integer lattice cell `(cx, cy)` for `seed`.
#[inline]
fn hash01(seed: u64, cx: i64, cy: i64) -> f32 {
    let mut h = seed
        ^ (cx as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (cy as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h ^= h >> 33;
    (h >> 40) as u32 as f32 / (1u32 << 24) as f32 // top 24 bits → [0, 1)
}

/// **Coherent** value noise in `[-1, 1)` — bilinear blend of `smoothstep`-faded lattice samples on a
/// coarse [`NOISE_CELL_PX`] grid (Perlin-1985-style value noise; smoothstep `t²(3−2t)` is a cubic
/// polynomial → transcendental-free, HR-5-safe). Unlike per-pixel white noise it varies SMOOTHLY across
/// neighbouring texels, so it warps as warble/wrinkle instead of the salt-and-pepper "scatter".
#[inline]
fn value_noise(seed: u64, x: f32, y: f32) -> f32 {
    let (gx, gy) = (x / NOISE_CELL_PX, y / NOISE_CELL_PX);
    let (x0, y0) = (gx.floor(), gy.floor());
    let (fx, fy) = (gx - x0, gy - y0);
    let (ix, iy) = (x0 as i64, y0 as i64);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let n00 = hash01(seed, ix, iy);
    let n10 = hash01(seed, ix + 1, iy);
    let n01 = hash01(seed, ix, iy + 1);
    let n11 = hash01(seed, ix + 1, iy + 1);
    let a = n00 + (n10 - n00) * sx;
    let b = n01 + (n11 - n01) * sx;
    (a + (b - a) * sy) * 2.0 - 1.0
}

/// A per-dab warp field: fixed dab geometry + knobs, sampled per pixel by [`Self::at`]. Built once per
/// dab in [`super::apply`]; for Twist it bakes the rotor table up front so `at` is O(1) per pixel.
pub(super) struct DabField {
    pub(super) mode: DeformMode,
    center: [f32; 2],
    inv_r2: f32,
    /// Dab radius in px (turbulence amplitude is a fraction of this, so it's zoom/size-stable).
    radius: f32,
    /// Effective push vector (mv·pressure + carried·momentum), in image px.
    mv: [f32; 2],
    /// Left-perpendicular unit of the push direction (for Push turbulence + Fold); `[0,0]` if no motion.
    perp: [f32; 2],
    /// Bipolar strength in `[-1, 1]` (Pinch −suck/+bulge · Twist CW/CCW). Push/Wrinkle/Fold use `|·|` for
    /// magnitude and the sign for direction where meaningful.
    signed: f32,
    pressure: f32,
    distortion: f32,
    seed: u64,
    /// Baked rotor table for Twist: `rotors[k]` = rotation by `k` whole degrees (`k ≤ twist_deg_max`).
    twist_rotors: Vec<[f32; 2]>,
    twist_deg_max: i32,
}

impl DabField {
    /// Build the field for one dab. `mv` is the raw drag delta (this sample − previous); `momentum_vec`
    /// is the carried velocity. `signed` is the bipolar strength (`0.5`-centred slider already mapped to
    /// `[-1,1]`). `pressure`/`distortion`/`momentum` are `0..1`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        mode: DeformMode,
        center: [f32; 2],
        radius: f32,
        mv: [f32; 2],
        momentum_vec: [f32; 2],
        signed: f32,
        pressure: f32,
        distortion: f32,
        momentum: f32,
        seed: u64,
    ) -> Self {
        let r = radius.max(1.0);
        // Effective push = this sample's motion plus the carried momentum (inertia).
        let eff = [
            mv[0] * pressure + momentum_vec[0] * momentum,
            mv[1] * pressure + momentum_vec[1] * momentum,
        ];
        let len2 = eff[0] * eff[0] + eff[1] * eff[1];
        let perp = if len2 > 1e-12 {
            let inv = 1.0 / len2.sqrt();
            [-eff[1] * inv, eff[0] * inv]
        } else {
            [0.0, 0.0]
        };
        // Twist: bake the rotor table once. Magnitude = pressure (so a drag twists at DEFAULT strength);
        // Strength adds intensity (`1 + |bias|`) and its SIGN picks the direction (applied per-pixel in `at`).
        let (twist_rotors, twist_deg_max) = if matches!(mode, DeformMode::Twist) {
            let deg_max = (TWIST_MAX_DEG * pressure * (1.0 + signed.abs())).round() as i32;
            (build_rotor_table(deg_max), deg_max)
        } else {
            (Vec::new(), 0)
        };
        Self {
            mode,
            center,
            inv_r2: 1.0 / (r * r),
            radius: r,
            mv: eff,
            perp,
            signed,
            pressure,
            distortion,
            seed,
            twist_rotors,
            twist_deg_max,
        }
    }

    /// Signed twist rotor `[cos, sin]` for a CONTINUOUS `deg_f` — linearly interpolates between the two
    /// bracketing baked integer-degree entries and renormalises. This is what kills the concentric
    /// "sawtooth" banding: quantising `deg` to whole degrees made the rotation a staircase across radius,
    /// visible as jagged rings (Enio 2026-07-04). Negative `deg_f` → conjugate.
    #[inline]
    fn twist_rotor(&self, deg_f: f32) -> [f32; 2] {
        let a = deg_f.abs();
        let k = a.floor() as usize;
        let frac = a - a.floor();
        let r0 = self.twist_rotors.get(k).copied().unwrap_or([1.0, 0.0]);
        let r1 = self.twist_rotors.get(k + 1).copied().unwrap_or(r0);
        let mut c = r0[0] + (r1[0] - r0[0]) * frac;
        let mut s = r0[1] + (r1[1] - r0[1]) * frac;
        let len = (c * c + s * s).sqrt();
        if len > 1e-6 {
            c /= len;
            s /= len; // the lerp shrinks the chord; renormalise back onto the unit circle
        }
        if deg_f < 0.0 { [c, -s] } else { [c, s] }
    }

    /// The displacement `D(p)` at image-pixel `p` (grid coordinate). `[0,0]` outside the dab radius, so an
    /// identity field (no motion / zero strength) leaves the gather at `dst` → byte-identical (DoD parity).
    ///
    /// ⚠️ **This is one DAB.** A whole deformation is a LIST of them, and the way the list is folded is
    /// the subject of [ADR-0156](../../../../../../docs/architecture/decisions/0156-liquify-is-an-authored-dab-list-cooked-on-the-device-never-a-stored-dense-field.md):
    /// `compose_at` is the law it decided, `apply.rs`'s running `d += a` is the defect it replaces, and
    /// the crossing between the two is W1 — not done here.
    pub(super) fn at(&self, p: [f32; 2]) -> [f32; 2] {
        let rel = [p[0] - self.center[0], p[1] - self.center[1]];
        let t2 = (rel[0] * rel[0] + rel[1] * rel[1]) * self.inv_r2;
        if t2 >= 1.0 {
            return [0.0, 0.0];
        }
        let f = falloff(t2);
        match self.mode {
            DeformMode::Push => {
                let mut d = [self.mv[0] * f, self.mv[1] * f];
                if self.distortion > 0.0 {
                    // Coherent, ISOTROPIC turbulence (two decorrelated noise channels), radius-scaled so it
                    // is stable at slow drags — a smooth warble, not the old per-pixel scatter.
                    let amp = self.radius * PUSH_TURB * self.distortion * f;
                    d[0] += value_noise(self.seed, p[0], p[1]) * amp;
                    d[1] += value_noise(self.seed ^ 0xA5A5_5A5A_5A5A_A5A5, p[0], p[1]) * amp;
                }
                d
            }
            DeformMode::Twist => {
                // Rotate content by +θ(r) about the centre: out at ψ samples in at ψ−θ, so
                // src = c + R(−θ)·rel  ⇒  D = rel − R(−θ)·rel.  (c,s) = (cosθ, sinθ).
                let deg_f = self.twist_deg_max as f32 * f;
                let deg_f = if self.signed < 0.0 { -deg_f } else { deg_f };
                let [c, s] = self.twist_rotor(deg_f);
                let rm = [rel[0] * c + rel[1] * s, -rel[0] * s + rel[1] * c]; // R(−θ)·rel
                [rel[0] - rm[0], rel[1] - rm[1]]
            }
            DeformMode::Pinch => {
                let g = self.radial_gain(f, p, PINCH_GAIN, self.distortion * PINCH_TURB);
                [rel[0] * g, rel[1] * g]
            }
            DeformMode::Wrinkle => {
                // Pinch/Punch whose gain is perturbed by COHERENT noise → smooth crinkle (not scatter). The
                // crinkle is intrinsic (its defining feature), so it is on regardless of Distortion.
                let g = self.radial_gain(f, p, WRINKLE_GAIN, WRINKLE_NOISE);
                [rel[0] * g, rel[1] * g]
            }
            DeformMode::Fold => {
                // Pinch confined to the axis PERPENDICULAR to the stroke → content folds toward the line.
                let g = self.radial_gain(f, p, FOLD_GAIN, self.distortion * PINCH_TURB);
                let proj = rel[0] * self.perp[0] + rel[1] * self.perp[1];
                [self.perp[0] * proj * g, self.perp[1] * proj * g]
            }
            DeformMode::Reconstruct => [0.0, 0.0], // resampled from the pre-deform buffer, not a field
        }
    }

    /// Signed radial gain, optionally modulated by COHERENT noise of depth `noise_amt` (Pinch/Wrinkle/Fold).
    /// Magnitude comes from **pressure** (so a drag pinches at DEFAULT strength — never a dead no-op);
    /// Strength adds intensity (`1 + |bias|`) and its SIGN picks the direction: `> 0` bulges out (punch),
    /// `≤ 0` sucks in (pinch — the centred default). Per-dab increment; accumulated over the stroke.
    #[inline]
    fn radial_gain(&self, f: f32, p: [f32; 2], max_gain: f32, noise_amt: f32) -> f32 {
        let dir = if self.signed > 0.0 { 1.0 } else { -1.0 };
        let base = dir * max_gain * self.pressure * (1.0 + self.signed.abs()) * f;
        if noise_amt > 0.0 {
            base * (1.0 + value_noise(self.seed, p[0], p[1]) * noise_amt)
        } else {
            base
        }
    }

    /// Os campos que o device precisa para avaliar [`Self::at`], **construídos por quem os possui**.
    ///
    /// ⚠️ Uma sonda que re-derivasse `eff`/`perp`/`inv_r2`/`twist_deg_max` a partir dos argumentos do
    /// construtor seria uma segunda cópia da derivação — e uma segunda cópia diverge no dia em que
    /// [`Self::new`] ganhar um termo. O payload sai daqui de dentro, ao lado dos campos.
    #[cfg(test)]
    pub(super) fn device_fields(&self) -> DeviceFields {
        DeviceFields {
            center: self.center,
            mv: self.mv,
            perp: self.perp,
            inv_r2: self.inv_r2,
            radius: self.radius,
            signed: self.signed,
            pressure: self.pressure,
            distortion: self.distortion,
            twist_deg_max: self.twist_deg_max as f32,
            // ⚠️ `as u32` sobre um enum sem payload dá a posição de DECLARAÇÃO, e é a mesma tabela que
            // `from_u8` assume — `deform_mode_index_is_the_wire_index` pina o par para que reordenar os
            // variants não vire um modo silenciosamente trocado no device.
            mode: self.mode as u32,
        }
    }
}

/// O retrato plano de um [`DabField`] para o kernel de [`super::cook_gpu`] — só os números que a lei lê.
#[cfg(test)]
pub(super) struct DeviceFields {
    pub(super) center: [f32; 2],
    pub(super) mv: [f32; 2],
    pub(super) perp: [f32; 2],
    pub(super) inv_r2: f32,
    pub(super) radius: f32,
    pub(super) signed: f32,
    pub(super) pressure: f32,
    pub(super) distortion: f32,
    pub(super) twist_deg_max: f32,
    pub(super) mode: u32,
}

/// ⚠️ **O ruído não atravessa** — e a recusa mora do lado que pode ser lida, não num shader mudo.
///
/// [`value_noise`] é splitmix64: `u64`, que o WGSL do core não tem. Enquanto a W1 não decidir entre uma
/// textura de ruído e um hash de 32 bits (as duas MUDAM os bytes), um dab que carrega ruído não tem
/// resposta no device — então o payload o RECUSA em vez de o deixar responder outra coisa.
#[cfg(test)]
pub(super) fn crosses_to_the_device(f: &DabField) -> bool {
    !matches!(f.mode, DeformMode::Wrinkle) && f.distortion <= 0.0
}

/// The displacement of a whole **dab list** at `p` — the fold that [ADR-0156] decided, and the single
/// door for it.
///
/// ⚠️ **A stroke is a RELAY, not a sum.** Dab `k` hands its result to dab `k+1`, so the list composes:
/// `D_k(p) = v_k(p) + D_{k−1}(p − v_k(p))` — the semi-Lagrangian law that
/// `ph2d_painter_brush::smear_field` already carries, and whose doc-comment says in as many words that
/// the obvious `disp += v` *"is WRONG, and wrong in a way worth recording because it LOOKS right"*.
/// `apply.rs` does the sum today, and the price is measured: a Twist that may displace a point at radius
/// `r` by at most `2r` (a rotation cannot do more — that is the diameter, reached at 180°) walks it
/// **158,55 px at r=30** after 60 dabs, because summing chords `R(θ)v − v` N times gives `N·chord`, a
/// straight tangent, while composing gives `R(Nθ)`, bounded. Summing IS exact composition — for
/// translation, and for nothing else, which is exactly why only **Push** ever looked right.
///
/// Unrolling that recursion is a **backwards walk**: start at `p` with the LAST dab and back-trace.
///
/// ⚠️ And the shape of this walk is the whole reason the cook can live on the device: it reads `p`, the
/// dab list, and **nothing else** — no neighbour, no previous frame, no accumulator shared between
/// pixels. Two nodes never talk. That is the condition [ADR-0109] asks of every parallel kernel here, and
/// it holds by construction rather than by discipline.
///
/// ⚠️ **`cfg(test)` é o estado HONESTO de hoje, não um esquecimento.** O produto (`apply.rs`) ainda
/// SOMA, e a travessia é a W1 do ADR — o cook, que substitui o acumulador incremental inteiro em vez de
/// remendá-lo. Enquanto o produto não a chama, deixá-la `pub` seria exatamente o que esta linha já
/// nomeou duas vezes (`warp_axis`, `serial_side`): **uma segunda resposta esperando alguém chamá-la**.
/// Os gates de [`super::compose_tests`] provam que ela cura o defeito reportado; o que falta é a
/// travessia, e ela tem smoke próprio porque muda o desenho de uma ferramenta que já shipa.
///
/// [ADR-0156]: ../../../../../../docs/architecture/decisions/0156-liquify-is-an-authored-dab-list-cooked-on-the-device-never-a-stored-dense-field.md
/// [ADR-0109]: ../../../../../../docs/architecture/decisions/0109-rayon-exception-watercolor-composite.md
#[cfg(test)]
pub(super) fn compose_at(dabs: &[DabField], p: [f32; 2]) -> [f32; 2] {
    let mut q = p;
    let mut d = [0.0_f32, 0.0];
    for f in dabs.iter().rev() {
        let v = f.at(q);
        d[0] += v[0];
        d[1] += v[1];
        q[0] -= v[0];
        q[1] -= v[1];
    }
    d
}

/// Build `[rotor_0 .. rotor_deg_max]` where `rotor_k` = the unit vector `(1,0)` rotated by `k` whole
/// degrees, via iterated 1° complex-multiply (transcendental-free — HR-5). `deg_max` is clamped to 360.
pub(super) fn build_rotor_table(deg_max: i32) -> Vec<[f32; 2]> {
    let n = deg_max.clamp(0, 360) as usize;
    let mut out = Vec::with_capacity(n + 1);
    let (mut x, mut y) = (1.0_f32, 0.0_f32);
    out.push([x, y]);
    for _ in 0..n {
        let nx = x * COS_1DEG - y * SIN_1DEG;
        let ny = x * SIN_1DEG + y * COS_1DEG;
        x = nx;
        y = ny;
        out.push([x, y]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_field_is_zero_everywhere() {
        // Zero PRESSURE + no motion ⇒ D = 0 at every pixel (⇒ the kernel is byte-identical). (Strength no
        // longer gates identity — magnitude comes from pressure, so every mode deforms at default strength.)
        for mode in [
            DeformMode::Push,
            DeformMode::Twist,
            DeformMode::Pinch,
            DeformMode::Wrinkle,
            DeformMode::Fold,
        ] {
            let f = DabField::new(
                mode,
                [50.0, 50.0],
                20.0,
                [0.0, 0.0],
                [0.0, 0.0],
                0.0,
                0.0, // pressure = 0 → no deformation
                0.0,
                0.0,
                1,
            );
            for &p in &[[50.0, 50.0], [55.0, 52.0], [40.0, 60.0]] {
                let d = f.at(p);
                assert_eq!(d, [0.0, 0.0], "{mode:?} at {p:?} must be identity");
            }
        }
    }

    #[test]
    fn modes_deform_at_default_strength() {
        // Regression for "only Push works": Twist / Pinch / Wrinkle / Fold take magnitude from PRESSURE,
        // so they deform even at the neutral (0.5-centred → signed 0) Strength default.
        for mode in [
            DeformMode::Twist,
            DeformMode::Pinch,
            DeformMode::Wrinkle,
            DeformMode::Fold,
        ] {
            let f = DabField::new(
                mode,
                [50.0, 50.0],
                20.0,
                [4.0, 0.0], // a drag (Fold needs a stroke direction, like Push)
                [0.0, 0.0],
                0.0, // signed = neutral (default Strength 0.5)
                0.8, // default pressure
                0.0,
                0.0,
                1,
            );
            let d = f.at([56.0, 54.0]); // off both the centre AND the drag axis, inside the radius
            assert!(
                d[0].abs() + d[1].abs() > 0.01,
                "{mode:?} must deform at default (neutral-strength) settings"
            );
        }
    }

    #[test]
    fn push_moves_content_by_the_drag_at_the_centre() {
        // At the centre (falloff = 1) with no distortion, D equals the drag vector, so the gather at the
        // centre samples `centre − mv` → content shifted by +mv. Falls to 0 at the edge.
        let mv = [6.0, -2.0];
        let f = DabField::new(
            DeformMode::Push,
            [30.0, 30.0],
            10.0,
            mv,
            [0.0, 0.0],
            0.0,
            1.0,
            0.0,
            0.0,
            7,
        );
        let d = f.at([30.0, 30.0]);
        assert!((d[0] - mv[0]).abs() < 1e-4 && (d[1] - mv[1]).abs() < 1e-4);
        // Outside the radius → no displacement.
        assert_eq!(f.at([100.0, 100.0]), [0.0, 0.0]);
    }

    #[test]
    fn pinch_sign_pulls_in_and_pushes_out() {
        // signed < 0 (pinch) ⇒ gather samples farther out (D points inward → negative along rel);
        // signed > 0 (punch) ⇒ inward sample (D points outward). Check the sign of the radial component.
        let rel_pt = [40.0, 30.0]; // +10 px along x from centre (30,30)
        let pinch = DabField::new(
            DeformMode::Pinch,
            [30.0, 30.0],
            20.0,
            [0.0, 0.0],
            [0.0, 0.0],
            -1.0,
            1.0,
            0.0,
            0.0,
            1,
        );
        let punch = DabField::new(
            DeformMode::Pinch,
            [30.0, 30.0],
            20.0,
            [0.0, 0.0],
            [0.0, 0.0],
            1.0,
            1.0,
            0.0,
            0.0,
            1,
        );
        assert!(pinch.at(rel_pt)[0] < 0.0, "pinch D points inward (−x)");
        assert!(punch.at(rel_pt)[0] > 0.0, "punch D points outward (+x)");
    }

    #[test]
    fn deform_mode_index_is_the_wire_index() {
        // `DeviceFields::mode` usa `self.mode as u32`, ou seja a posição de DECLARAÇÃO — e `from_u8`
        // assume a mesma tabela. Enquanto ninguém prova o par, reordenar os variants troca o modo no
        // device em silêncio (e o gate de paridade não pegaria: ele compara a MESMA lista dos dois lados).
        for v in 0u8..=5 {
            assert_eq!(
                DeformMode::from_u8(v) as u32,
                u32::from(v),
                "a ordem de declaração É a ordem do wire"
            );
        }
    }

    #[test]
    fn twist_rotor_is_unit_length_and_identity_at_zero() {
        let table = build_rotor_table(90);
        assert_eq!(table[0], [1.0, 0.0]);
        for r in &table {
            let len2 = r[0] * r[0] + r[1] * r[1];
            assert!((len2 - 1.0).abs() < 1e-3, "rotor stays unit-length");
        }
        // 90° rotor ≈ (0, 1).
        let r90 = table[90];
        assert!(r90[0].abs() < 1e-2 && (r90[1] - 1.0).abs() < 1e-2);
    }
}
