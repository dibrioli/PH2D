//! `StampScheduler` — converte pointer events em sequências de [`Stamp`]s
//! prontas para o `StampPipeline` compute dispatch.
//!
//! Spec [`docs/Painter_projeto/01_brush_engine.md`](
//! ../../../docs/Painter_projeto/01_brush_engine.md) §1.2 (stroke pipeline) +
//! §1.3.1 (StrokePath: spacing, jitter, lateral, falloff).
//!
//! ## Responsabilidade (T1.5 scope)
//!
//! 1. Manter **estado de stroke** (último ponto carimbado, distância residual
//!    não-consumida, seed do PRNG, contador monotônico de stamps).
//! 2. Para cada novo pointer sample, emitir N [`Stamp`]s distribuídos ao longo
//!    do segmento `last_point → point` conforme `Brush::stroke_path.spacing`.
//! 3. Aplicar `spacing_jitter` (variação aleatória do espaçamento) e
//!    `jitter_lateral` (deslocamento perpendicular ao stroke direction).
//!
//! ## Fora de escopo (W1+ subsystems)
//!
//! - **Curves per-device** (Pressure/Tilt/Barrel) — T-input (ADR-0050).
//! - **Color Dynamics** stamp-level jitter — T-color full (ADR-0051) +
//!   integração em T1.6.
//! - **Shape scatter / count / rotation_follow** — T1.6 (shape atlas wired).
//! - **Stabilization / Streamline / Motion Filtering** — T1.7.
//! - **Taper** size/opacity curves — T1.7.
//! - **Falloff** ao longo do stroke — T1.7.
//!
//! T1.5 carrega o esqueleto canônico (estado + spacing/jitter) — extensões
//! acima entram em sub-fields adicionais SEM quebrar a API pública
//! `advance()` (audit 2026-05-26 — extensão via builder, não refactor).
//!
//! ## HR-3 zero-alloc invariant
//!
//! Pool pré-alocado de [`MAX_STAMPS_PER_DISPATCH`] = 4096 [`Stamp`]s
//! (= 384 KB). [`StampScheduler::advance`] limpa o pool no início e
//! preenche via `push` (sem realloc pois `capacity >= MAX`). Gate
//! [`tests::advance_does_not_realloc`] prova zero realloc após `begin_stroke`.
//!
//! ## Determinismo (HR-5)
//!
//! O PRNG interno é determinístico: dado `stroke_seed` + posição do stamp no
//! stroke (counter monotônico), `spacing_jitter` e `jitter_lateral` emitem
//! offsets bit-identicos cross-OS. Usa `wyhash`-style hash de
//! `(seed, stamp_index, axis_tag)`; sem dependência de `rand` crate (que
//! tem variabilidade de seeding cross-platform).

use crate::brush::Brush;
use crate::stamp::{MAX_STAMP_SIZE_PX, MAX_STAMPS_PER_DISPATCH, Stamp};

/// Pointer sample input — uma amostra do dispositivo (mouse / Pencil / tablet).
///
/// Para T1.5 MVP esses 4 campos bastam. Curves (pressure_curve / tilt_curve /
/// palm rejection) entram via T-input (ADR-0050) ANTES do scheduler — quando
/// `ph2d-painter-input::PointerSource` ship, este struct passa a receber
/// valores já curvados.
#[derive(Copy, Clone, Debug, Default)]
pub struct PointerSample {
    /// Coordenadas em canvas-world pixels (mesmo espaço de `Stamp.position_world`).
    pub position: [f32; 2],
    /// Pressão normalizada `[0.0, 1.0]`. Mouse = 1.0 constante.
    pub pressure: f32,
    /// Tilt em radianos `[0, π/2]`. Mouse/touch sem tilt = 0.
    pub tilt: f32,
}

/// Estado de stroke + pool de stamps reutilizável. Owned pelo
/// [`PainterTool`](../../ph2d-tool-painter/index.html); um instance por tool,
/// reset via [`Self::begin_stroke`] no pointer-down e drained via
/// [`Self::advance`] em cada pointer-move.
pub struct StampScheduler {
    /// Buffer pré-alocado. `Vec` ao invés de array fixo porque o consumidor
    /// (`StampPipeline::encode`) recebe `&[Stamp]` — Vec é o canal natural.
    /// Capacity reservada no construtor; `clear()` mantém capacity.
    pool: Vec<Stamp>,
    /// Último ponto carimbado (canvas-world px). `None` antes do primeiro
    /// stamp do stroke ou após [`Self::end_stroke`].
    last_point: Option<[f32; 2]>,
    /// Distância "comida" do espaçamento que sobrou do segmento anterior.
    /// Garante continuidade do passo entre dois pointer samples consecutivos
    /// (último gap intra-segmento + primeiro gap do próximo segmento somam
    /// um spacing inteiro, nunca duplicam um stamp).
    residual_dist: f32,
    /// Seed do stroke. Determinístico — derive de pointer-down time + entity
    /// + brush hash no caller. PRNG interno mistura com `stamp_index`.
    stroke_seed: u64,
    /// Contador monotônico de stamps emitidos NESTE stroke. Reset em
    /// [`Self::begin_stroke`]. Usado como entrada do hash determinístico.
    stamp_index: u64,
}

impl Default for StampScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl StampScheduler {
    /// Construct com pool reservado para [`MAX_STAMPS_PER_DISPATCH`]
    /// (384 KB). One-shot alloc no construtor; HR-3 garantido por
    /// [`Self::advance`] daí em diante (`push` em Vec com headroom não
    /// re-aloca; [`Vec::clear`] preserva capacity).
    #[must_use]
    pub fn new() -> Self {
        Self {
            pool: Vec::with_capacity(MAX_STAMPS_PER_DISPATCH),
            last_point: None,
            residual_dist: 0.0,
            stroke_seed: 0,
            stamp_index: 0,
        }
    }

    /// Inicia um novo stroke. Reseta o estado interno mantendo a capacity do
    /// pool. `seed` deve ser derivado de inputs determinísticos pelo caller
    /// (tempo do pointer-down + entity bits + brush hash).
    pub fn begin_stroke(&mut self, seed: u64) {
        self.pool.clear();
        self.last_point = None;
        self.residual_dist = 0.0;
        self.stroke_seed = seed;
        self.stamp_index = 0;
    }

    /// Finaliza o stroke atual. Limpa estado de continuação mas mantém
    /// `stroke_seed` para auditoria. Idempotente.
    pub fn end_stroke(&mut self) {
        self.pool.clear();
        self.last_point = None;
        self.residual_dist = 0.0;
    }

    /// "Brush lifted" — interrompe o segmento atual SEM encerrar o stroke
    /// (mantém `stroke_seed` e o stamp_index counter). O próximo `advance`
    /// trata o sample como NOVO ponto inicial, igual ao primeiro stamp do
    /// stroke — sem interpolar uma linha reta entre o último ponto antes do
    /// "lift" e o novo ponto após o "drop".
    ///
    /// Caso de uso: cursor sai do footprint do sprite mid-drag e re-entra
    /// noutro local. Sem `break_segment`, `advance` interpola stamps ao
    /// longo do gap (smear visual). Audit T1.5 round 3 R3-LE-1.
    pub fn break_segment(&mut self) {
        self.pool.clear();
        self.last_point = None;
        // `residual_dist` deve voltar a 0 também — o gap não consome
        // espaçamento; a próxima chamada começa fresca como se fosse a
        // primeira do stroke.
        self.residual_dist = 0.0;
    }

    /// Verdadeiro se um stroke já tem PELO MENOS UM stamp emitido (i.e. já
    /// houve uma chamada `advance` que produziu um stamp). Após `begin_stroke`
    /// mas antes do primeiro `advance` retorna `false` — "stroke armado mas
    /// ainda vazio". Após `end_stroke` retorna `false`.
    ///
    /// Usado pelo bridge pra decidir se um pointer-up encerra um stroke
    /// válido (que vira commit) ou se foi click vazio (no-op).
    #[must_use]
    pub fn is_in_stroke(&self) -> bool {
        self.last_point.is_some()
    }

    /// Avança o stroke até `sample`, emitindo todos os stamps que cabem no
    /// segmento `last_point → sample.position` conforme `brush.stroke_path`
    /// e `size_px` (diameter efetivo do stamp em pixels — caller computa a
    /// partir de pressure curve etc.; T-input integrado em T1.6).
    ///
    /// Retorna um slice estável até a próxima chamada (`Vec::clear` na
    /// próxima invocação invalida o slice — caller deve consumir antes).
    ///
    /// `color_oklab` é a cor STRAIGHT-alpha (L, a, b, α) que vai dentro de
    /// cada [`Stamp`]; T-color full (ADR-0051) integra Color Dynamics aqui em
    /// T1.6 (jitter per-stamp baseado em `color_dynamics`).
    ///
    /// ## Estabilidade do slice
    ///
    /// O caller é responsável por consumir o slice ANTES de chamar `advance`
    /// novamente — a próxima invocação faz `pool.clear()` e regrava do zero.
    /// Caller que precisa persistir os stamps copia o slice (Stamp é `Copy`).
    pub fn advance(
        &mut self,
        brush: &Brush,
        sample: PointerSample,
        size_px: f32,
        color_oklab: [f32; 4],
    ) -> &[Stamp] {
        self.pool.clear();

        // Saneamento de entrada — pontos com componente não-finito viram no-op.
        // Defesa antes do GPU pipeline (que também filtra) para evitar
        // poluir o ring buffer com lixo.
        if !sample.position[0].is_finite()
            || !sample.position[1].is_finite()
            || !sample.pressure.is_finite()
        {
            return &self.pool;
        }

        // Diameter efetivo clampado ao limite ABI do Stamp. Caller passou
        // tamanho derivado de slider+pressure; aqui só impomos o teto.
        let diameter = size_px.clamp(1.0, MAX_STAMP_SIZE_PX as f32);
        let spacing_frac = brush.stroke_path.spacing.clamp(0.01, 1.0);
        // `spacing_px` = `spacing_frac * diameter`. Lower bound 1.0 evita
        // divisão por zero em strokes de stamp tamanho mínimo + spacing < 1
        // (audit-edge: spacing 0.01 * diameter 1.0 = 0.01 → infinite loop
        // without lower bound).
        let spacing_px = (spacing_frac * diameter).max(1.0);

        // **R4-LG-5 fix:** hoist brush-param clamps ABOVE the while-loop.
        // `brush: &Brush` is read-only for the duration of this advance;
        // these values are constant per stroke — re-evaluating them per
        // stamp wastes ~7 cycles each iteration (LLVM can't prove
        // invariance through the &Brush reference).
        let jitter_amp = brush.stroke_path.spacing_jitter.clamp(0.0, 1.0);
        let lat_amp = brush.stroke_path.jitter_lateral.clamp(0.0, 1.0);

        match self.last_point {
            None => {
                // Primeiro pointer do stroke: deposita 1 stamp na posição.
                self.push_stamp(brush, sample, diameter, color_oklab, [0.0, 0.0]);
                self.last_point = Some(sample.position);
                self.residual_dist = 0.0;
                return &self.pool;
            }
            Some(last) => {
                let delta = [sample.position[0] - last[0], sample.position[1] - last[1]];
                let segment_len = (delta[0] * delta[0] + delta[1] * delta[1]).sqrt();
                if segment_len < f32::EPSILON {
                    // Pointer ficou parado (jitter de driver / sample
                    // duplicado). Não emite stamp novo nem move last_point.
                    return &self.pool;
                }
                // Unit vector + perpendicular para jitter_lateral.
                let inv_len = 1.0 / segment_len;
                let ux = delta[0] * inv_len;
                let uy = delta[1] * inv_len;
                let perp = [-uy, ux];

                // Walk: `cursor` é a distância acumulada ao longo do segmento
                // a partir de `last`. Começa em `(spacing_px - residual_dist)`:
                // a contribuição residual do segmento anterior + a nova
                // distância forma o primeiro spacing inteiro neste segmento.
                //
                // Audit T1.5 round 1 A-H3: usar `rem_euclid` no residual
                // final garante invariante `residual ∈ [0, spacing_px)`
                // independente de quantos segmentos curtos foram
                // descartados em sequência.
                let cursor_initial = spacing_px - self.residual_dist;
                let mut cursor = cursor_initial;
                while cursor <= segment_len && self.pool.len() < MAX_STAMPS_PER_DISPATCH {
                    // Spacing jitter — variação aleatória multiplicativa do
                    // intervalo até o próximo stamp. `spacing_jitter` em
                    // [0, 1]; jitter em [-J, +J] frações do spacing.
                    let j_offset = if jitter_amp > 0.0 {
                        let j = self.det_random(self.stamp_index, 0xA1) * 2.0 - 1.0;
                        j * jitter_amp * spacing_px
                    } else {
                        0.0
                    };
                    let t_along = (cursor + j_offset).clamp(0.0, segment_len);

                    // Jitter lateral — deslocamento perpendicular ao stroke
                    // direction. `jitter_lateral` em [0, 1]; offset em
                    // [-L, +L] frações do diameter.
                    let lat_offset = if lat_amp > 0.0 {
                        let l = self.det_random(self.stamp_index, 0xB2) * 2.0 - 1.0;
                        l * lat_amp * diameter
                    } else {
                        0.0
                    };

                    let pos = [
                        last[0] + ux * t_along + perp[0] * lat_offset,
                        last[1] + uy * t_along + perp[1] * lat_offset,
                    ];
                    // Reusa o pointer sample do CALLER (pressure/tilt) — sem
                    // interp p/ MVP. T-input adicionará pressure curve
                    // ao longo do segmento (linear-interpolate last_press →
                    // sample.press conforme t_along/segment_len).
                    let interp_sample = PointerSample {
                        position: pos,
                        pressure: sample.pressure,
                        tilt: sample.tilt,
                    };
                    self.push_stamp(brush, interp_sample, diameter, color_oklab, perp);

                    cursor += spacing_px;
                }
                // Residual = posição efetiva mod spacing_px. Trata
                // uniformemente os 3 cenários:
                // 1. Stamps emitted → `consumed = last_cursor_that_fired
                //    = cursor - spacing_px` cai em `[0, segment_len)`;
                //    residual = `segment_len - consumed` cai em
                //    `(0, spacing_px]`, e o `rem_euclid` normaliza
                //    para `[0, spacing_px)`.
                // 2. Zero stamps (segment curto) → `consumed = -
                //    residual_dist_prev`, então
                //    `segment_len - consumed = segment_len +
                //    residual_dist_prev`, que pode ser > spacing_px;
                //    `rem_euclid` traz de volta para `[0, spacing_px)`.
                // 3. Stamp count cap atingido → cursor parou antes de
                //    consumir todo segmento; mesma fórmula vale.
                let consumed = cursor - spacing_px;
                let raw_residual = segment_len - consumed;
                self.residual_dist = raw_residual.rem_euclid(spacing_px);
                self.last_point = Some(sample.position);
            }
        }
        &self.pool
    }

    fn push_stamp(
        &mut self,
        brush: &Brush,
        sample: PointerSample,
        diameter: f32,
        color_oklab: [f32; 4],
        _perp: [f32; 2],
    ) {
        // Audit T1.5 round 1 A-M4: increment stamp_index iff a stamp is
        // actually pushed. Currently 1:1 — but a future refactor that
        // adds an early-out branch (e.g., skipping stamps that fall off
        // canvas) would silently desync the jitter sequence from the
        // stamp count. Asserting in debug catches accidental drift.
        let len_before = self.pool.len();
        // Branch HR-3 — `Vec::push` em capacity disponível não realoca.
        let mut s = Stamp::zeroed();
        s.position_world = sample.position;
        s.size_px = diameter;
        s.rotation_rad = 0.0; // T1.6 — shape_rotation_follow / scatter
        s.pressure = sample.pressure.clamp(0.0, 1.0);
        s.tilt = sample.tilt.clamp(0.0, std::f32::consts::FRAC_PI_2);
        s.azimuth = 0.0; // T1.6
        s.barrel_roll = 0.0; // T1.6
        s.color_oklab = color_oklab; // STRAIGHT alpha (shader premultiplies)
        s.opacity = 1.0; // T1.7 — taper opacity + stroke-level opacity
        s.flow = brush.rendering.flow.clamp(0.0, 1.0);
        s.wet_amount = 0.0; // T-wet-mix W7+
        s.shape_layer = 0; // round_hard slot
        s.grain_layer = u32::MAX; // sem grain (round_hard library default)
        s.grain_offset_uv = [0.0, 0.0];
        s.grain_scale = 1.0;
        s.flags = 0; // sem flip/grain procedural/etc. — T1.6+
        s.rendering_mode = brush.rendering.rendering_mode as u32;
        s.pigment_mode = brush.rendering.pigment_mode as u32;
        self.pool.push(s);
        debug_assert_eq!(
            self.pool.len(),
            len_before + 1,
            "push_stamp must add exactly one stamp before incrementing stamp_index (HR-5)"
        );
        self.stamp_index = self.stamp_index.wrapping_add(1);
    }

    /// PRNG determinístico — wyhash mixer com `(stroke_seed, stamp_index,
    /// axis_tag)` como entrada. Retorna `[0.0, 1.0)`.
    ///
    /// Não usa `rand`: `SmallRng` etc. tem seeding variável cross-platform.
    /// Mixer manual = bit-identico Mac/Linux/Windows. HR-5 cumprido.
    #[inline]
    fn det_random(&self, stamp_index: u64, axis_tag: u64) -> f32 {
        // wyhash-style: 3-fold xor-shift + multiply. Boa avalanche para
        // uso de jitter sem dep externa.
        let mut h = self
            .stroke_seed
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(stamp_index);
        h ^= h >> 32;
        h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h ^= axis_tag;
        h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
        h ^= h >> 31;
        // Top 24 bits → [0, 1) preserva precisão de f32 mantissa.
        ((h >> 40) as f32) / ((1u64 << 24) as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::round_hard;

    fn p(x: f32, y: f32) -> PointerSample {
        PointerSample {
            position: [x, y],
            pressure: 0.5,
            tilt: 0.0,
        }
    }

    #[test]
    fn new_pool_has_max_capacity() {
        let s = StampScheduler::new();
        assert_eq!(s.pool.capacity(), MAX_STAMPS_PER_DISPATCH);
        assert!(s.pool.is_empty());
        assert!(!s.is_in_stroke());
    }

    #[test]
    fn begin_stroke_resets_state() {
        let mut s = StampScheduler::new();
        s.begin_stroke(42);
        let brush = round_hard();
        let _ = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
        assert!(s.is_in_stroke());
        s.begin_stroke(99);
        assert!(s.pool.is_empty());
        assert!(!s.is_in_stroke());
        assert_eq!(s.stroke_seed, 99);
    }

    #[test]
    fn first_pointer_emits_one_stamp() {
        let mut s = StampScheduler::new();
        s.begin_stroke(1);
        let brush = round_hard();
        let stamps = s.advance(&brush, p(10.0, 20.0), 32.0, [0.7, 0.0, 0.0, 1.0]);
        assert_eq!(stamps.len(), 1);
        assert_eq!(stamps[0].position_world, [10.0, 20.0]);
        assert_eq!(stamps[0].size_px, 32.0);
        assert_eq!(stamps[0].color_oklab, [0.7, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn segment_emits_stamps_at_spacing() {
        let mut s = StampScheduler::new();
        s.begin_stroke(7);
        let brush = round_hard(); // spacing 0.10
        let diameter = 100.0;
        // spacing_px = 0.10 * 100.0 = 10 px. First pointer emits 1.
        let _ = s.advance(&brush, p(0.0, 0.0), diameter, [0.0; 4]);
        // Move to x=50; segment_len 50; expect 5 stamps at x = 10, 20, 30, 40, 50.
        let stamps = s.advance(&brush, p(50.0, 0.0), diameter, [0.0; 4]);
        assert_eq!(stamps.len(), 5, "expected 5 stamps from (0,0)→(50,0)");
        assert_eq!(stamps[0].position_world, [10.0, 0.0]);
        assert_eq!(stamps[4].position_world, [50.0, 0.0]);
    }

    #[test]
    fn determinism_same_seed_same_jitter() {
        let mut brush = round_hard();
        brush.stroke_path.spacing_jitter = 0.5;
        brush.stroke_path.jitter_lateral = 0.3;

        let mut s1 = StampScheduler::new();
        s1.begin_stroke(12345);
        let _ = s1.advance(&brush, p(0.0, 0.0), 50.0, [0.5, 0.0, 0.0, 1.0]);
        let a: Vec<[f32; 2]> = s1
            .advance(&brush, p(100.0, 0.0), 50.0, [0.5, 0.0, 0.0, 1.0])
            .iter()
            .map(|st| st.position_world)
            .collect();

        let mut s2 = StampScheduler::new();
        s2.begin_stroke(12345);
        let _ = s2.advance(&brush, p(0.0, 0.0), 50.0, [0.5, 0.0, 0.0, 1.0]);
        let b: Vec<[f32; 2]> = s2
            .advance(&brush, p(100.0, 0.0), 50.0, [0.5, 0.0, 0.0, 1.0])
            .iter()
            .map(|st| st.position_world)
            .collect();

        // HR-5 cross-OS implicit: same input → bit-identical output (no
        // floating-point lifestyle from `rand` etc.).
        assert_eq!(a, b, "same seed must produce bit-identical positions");
    }

    #[test]
    fn advance_does_not_realloc_pool() {
        // HR-3: pool capacity is reserved at construct; subsequent advances
        // re-clear and re-push without realloc. We sample capacity before
        // and after a high-density burst.
        let mut s = StampScheduler::new();
        s.begin_stroke(1);
        let mut brush = round_hard();
        brush.stroke_path.spacing = 0.01; // very tight (10x default)
        let cap_before = s.pool.capacity();
        let _ = s.advance(&brush, p(0.0, 0.0), 200.0, [0.0; 4]);
        // segment of 1000 px with spacing 0.01 * 200 = 2 px → 500 stamps; well
        // under cap. capacity should be unchanged.
        let _ = s.advance(&brush, p(1000.0, 0.0), 200.0, [0.0; 4]);
        let cap_after = s.pool.capacity();
        assert_eq!(
            cap_before, cap_after,
            "pool capacity must not grow across advances"
        );
        assert!(
            cap_after >= MAX_STAMPS_PER_DISPATCH,
            "pool must hold at least MAX_STAMPS_PER_DISPATCH"
        );
    }

    #[test]
    fn pool_is_capped_at_max() {
        let mut s = StampScheduler::new();
        s.begin_stroke(1);
        let mut brush = round_hard();
        brush.stroke_path.spacing = 0.01; // tight
        // First pointer + a huge segment that would otherwise overflow:
        // 1_000_000 px / 2 px spacing = 500k stamps — cap at 4096.
        let _ = s.advance(&brush, p(0.0, 0.0), 200.0, [0.0; 4]);
        let stamps = s.advance(&brush, p(1_000_000.0, 0.0), 200.0, [0.0; 4]);
        assert!(
            stamps.len() <= MAX_STAMPS_PER_DISPATCH,
            "scheduler must cap at MAX_STAMPS_PER_DISPATCH (got {})",
            stamps.len()
        );
    }

    #[test]
    fn nan_input_emits_nothing() {
        let mut s = StampScheduler::new();
        s.begin_stroke(1);
        let brush = round_hard();
        let bad = PointerSample {
            position: [f32::NAN, 0.0],
            pressure: 0.5,
            tilt: 0.0,
        };
        let stamps = s.advance(&brush, bad, 32.0, [0.0; 4]);
        assert!(
            stamps.is_empty(),
            "NaN input must produce zero stamps (HR-3 defense-in-depth)"
        );
        assert!(!s.is_in_stroke(), "state must remain pristine after NaN");
    }

    #[test]
    fn duplicate_pointer_emits_nothing_after_first() {
        let mut s = StampScheduler::new();
        s.begin_stroke(1);
        let brush = round_hard();
        let _ = s.advance(&brush, p(5.0, 5.0), 32.0, [0.0; 4]);
        let stamps = s.advance(&brush, p(5.0, 5.0), 32.0, [0.0; 4]);
        assert!(
            stamps.is_empty(),
            "duplicate pointer (segment_len < eps) must emit nothing"
        );
    }

    #[test]
    fn end_stroke_clears_state() {
        let mut s = StampScheduler::new();
        s.begin_stroke(1);
        let brush = round_hard();
        let _ = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
        s.end_stroke();
        assert!(!s.is_in_stroke());
        assert!(s.last_point.is_none());
        assert_eq!(s.residual_dist, 0.0);
    }

    #[test]
    fn stamp_index_monotonic_across_advances() {
        let mut s = StampScheduler::new();
        s.begin_stroke(1);
        let brush = round_hard();
        let _ = s.advance(&brush, p(0.0, 0.0), 50.0, [0.0; 4]);
        let idx_after_one = s.stamp_index;
        // (0,0)→(25,0) at spacing 5 px → 5 stamps.
        let _ = s.advance(&brush, p(25.0, 0.0), 50.0, [0.0; 4]);
        assert!(s.stamp_index > idx_after_one, "stamp_index must grow");
    }

    #[test]
    fn rendering_mode_propagates_to_stamps() {
        let mut s = StampScheduler::new();
        s.begin_stroke(1);
        let mut brush = round_hard();
        brush.rendering.rendering_mode = crate::RenderingMode::IntenseBlending;
        let stamps = s.advance(&brush, p(0.0, 0.0), 32.0, [0.0; 4]);
        assert_eq!(stamps.len(), 1);
        assert_eq!(
            stamps[0].rendering_mode,
            crate::RenderingMode::IntenseBlending as u32
        );
    }

    #[test]
    fn residual_dist_chain_short_segments() {
        // Audit T1.5 round 1 A-H3 regression + round 2 F7 strengthening:
        // 3 consecutive sub-spacing segments. The contract is:
        // (a) `residual_dist ∈ [0, spacing_px)` at all times;
        // (b) cumulative stamps sit on the spacing grid (positions
        //     `0, spacing_px, 2·spacing_px, …`) — NOT clustered at
        //     segment boundaries.
        let mut s = StampScheduler::new();
        s.begin_stroke(1);
        let brush = round_hard(); // spacing 0.10
        let diameter = 100.0;
        let spacing_px = 10.0;
        // 1st advance: emits stamp at (0,0). residual stays 0.
        let stamps_a = s
            .advance(&brush, p(0.0, 0.0), diameter, [0.0; 4])
            .iter()
            .map(|st| st.position_world[0])
            .collect::<Vec<_>>();
        assert_eq!(stamps_a, vec![0.0_f32], "first advance emits at origin");

        // Three sub-spacing segments (cumulative 9 < spacing_px=10).
        let _ = s.advance(&brush, p(3.0, 0.0), diameter, [0.0; 4]);
        assert!(
            s.residual_dist >= 0.0 && s.residual_dist < spacing_px,
            "residual must stay in [0, {}) after segment 1; got {}",
            spacing_px,
            s.residual_dist
        );
        let _ = s.advance(&brush, p(6.0, 0.0), diameter, [0.0; 4]);
        let _ = s.advance(&brush, p(9.0, 0.0), diameter, [0.0; 4]);
        assert!(
            s.residual_dist >= 0.0 && s.residual_dist < spacing_px,
            "residual must stay in [0, {}) after 3 short segments; got {}",
            spacing_px,
            s.residual_dist
        );

        // 5th advance crosses the next grid line.
        let stamps_d = s
            .advance(&brush, p(15.0, 0.0), diameter, [0.0; 4])
            .iter()
            .map(|st| st.position_world[0])
            .collect::<Vec<_>>();
        // At least one stamp landed on or near the next grid position.
        // The spacing grid expectation: stamps at x ∈ {0, 10, 20, …}.
        // After cumulative 9 px (3 short segments emitting zero stamps)
        // + 6 px crossing to x=15, the gridline at x=10 was crossed
        // once → exactly 1 stamp at x ≈ 10.
        assert_eq!(stamps_d.len(), 1, "exactly 1 stamp crossing x=10");
        assert!(
            (stamps_d[0] - spacing_px).abs() < 1.5,
            "grid stamp must be at x ≈ {} (got {})",
            spacing_px,
            stamps_d[0]
        );
        assert!(
            s.residual_dist >= 0.0 && s.residual_dist < spacing_px,
            "residual ∈ [0, {}) at end; got {}",
            spacing_px,
            s.residual_dist
        );
    }

    #[test]
    fn break_segment_resets_last_point_without_ending_stroke() {
        // Audit T1.5 round 3 R3-LE-1: cursor exit/re-enter footprint
        // should not paint a smear across the gap. `break_segment`
        // wipes `last_point` so next advance starts fresh (single stamp
        // at new position, no interpolation).
        let mut s = StampScheduler::new();
        s.begin_stroke(1);
        let brush = round_hard();
        let diameter = 100.0;
        // Deposit one stamp at (0,0).
        let _ = s.advance(&brush, p(0.0, 0.0), diameter, [0.0; 4]);
        assert!(s.is_in_stroke());
        let initial_stamp_index = s.stamp_index;

        // Cursor exits footprint → break_segment.
        s.break_segment();
        assert!(
            !s.is_in_stroke(),
            "break_segment clears last_point (is_in_stroke becomes false)"
        );
        assert_eq!(s.residual_dist, 0.0);
        // BUT: stroke_seed + stamp_index counter survive — same stroke continues.
        assert_eq!(
            s.stamp_index, initial_stamp_index,
            "stamp_index must NOT reset (stroke continues)"
        );
        assert_eq!(s.stroke_seed, 1, "stroke_seed survives break_segment");

        // Re-enter at (500, 500) — should emit exactly 1 stamp at that
        // position (no smear interpolation from (0,0)).
        let stamps = s
            .advance(&brush, p(500.0, 500.0), diameter, [0.0; 4])
            .iter()
            .map(|st| st.position_world)
            .collect::<Vec<_>>();
        assert_eq!(
            stamps,
            vec![[500.0_f32, 500.0]],
            "re-entry after break_segment must emit ONE stamp at new pos \
             (no smear across the gap)"
        );
    }

    #[test]
    fn stamp_index_does_not_exceed_pool_cap() {
        // Audit T1.5 round 2 F6 (MISSING-GATE-STAMP-INDEX-CAP). The
        // A-M4 desync risk fires exactly when `push_stamp` rejects
        // (cap saturated). Verify stamp_index advances ≤ pool cap.
        let mut s = StampScheduler::new();
        s.begin_stroke(1);
        let mut brush = round_hard();
        brush.stroke_path.spacing = 0.01; // tight spacing
        // (0,0) → (1_000_000, 0) at spacing 0.01·200=2 px would emit
        // 500k stamps; pool caps at 4096.
        let _ = s.advance(&brush, p(0.0, 0.0), 200.0, [0.0; 4]);
        let stamps = s.advance(&brush, p(1_000_000.0, 0.0), 200.0, [0.0; 4]);
        let emitted = stamps.len();
        assert!(
            emitted <= MAX_STAMPS_PER_DISPATCH,
            "scheduler must cap at MAX_STAMPS_PER_DISPATCH (got {})",
            emitted
        );
        // stamp_index = first advance (1) + second advance (emitted).
        // Critical: it must NOT exceed the actual stamps pushed +
        // the prior advance's 1 stamp.
        assert_eq!(
            s.stamp_index,
            1 + emitted as u64,
            "stamp_index = 1 (initial) + actually-emitted; got {}",
            s.stamp_index
        );
    }

    #[test]
    fn stamp_index_increments_one_per_pushed_stamp() {
        // Audit T1.5 round 1 A-M4: stamp_index must advance by exactly
        // the count of stamps actually emitted, never more.
        let mut s = StampScheduler::new();
        s.begin_stroke(1);
        let brush = round_hard();
        let stamps = s.advance(&brush, p(0.0, 0.0), 50.0, [0.0; 4]).len() as u64;
        assert_eq!(s.stamp_index, stamps);
        let n2 = s.advance(&brush, p(25.0, 0.0), 50.0, [0.0; 4]).len() as u64;
        assert_eq!(s.stamp_index, stamps + n2);
    }

    #[test]
    fn det_random_in_unit_interval() {
        let mut s = StampScheduler::new();
        s.begin_stroke(42);
        for i in 0..1024 {
            for axis in [0xA1, 0xB2, 0xC3] {
                let v = s.det_random(i, axis);
                assert!((0.0..1.0).contains(&v), "det_random out of [0,1): {v}");
            }
        }
    }
}
