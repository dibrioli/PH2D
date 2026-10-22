//! Live tuning registry (port of `tuning.js`, SPEC §16): one module owning
//! every calibration knob the engine reads at runtime.
//!
//! The engine reads knobs through the typed [`Knob`] enum (an array index —
//! cheaper than the JS Map, same semantics); the panel reads the [`KNOB_DEFS`]
//! table for labels/ranges/defaults and writes through [`Tuning::set_by_key`]
//! (NaN falls back to the default, exactly like the JS free numeric input).
//!
//! The `Hidden` group holds the gated extensions of SPEC §17: implemented,
//! default-neutral, no visible sliders.

/// Every calibration knob. Discriminants are indices into [`KNOB_DEFS`] and
/// the value array — the order below IS the table order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Knob {
    PigmentPerDab,
    PaperGate,
    Felt,
    BristleCount,
    Drag,
    Pickup,
    Intensity,
    BristleStrength,
    BristleSize,
    Spacing,
    TipClean,
    BlendForce,
    GateSaturation,
    WaterPerDab,
    WaterCap,
    Evaporation,
    Rewet,
    Retention,
    EdgeDarkening,
    BaseEvaporation,
    Leveling,
    Capillary,
    Brake,
    Gravity,
    LevelClamp,
    Viscosity,
    MaxVelocity,
    Projection,
    BrakeReach,
    CapillaryGate,
    Eraser,
    Dryer,
    Blow,
    Smear,
    PaperContrast,
    PaperFibres,
    PaperGrooves,
    VisualGrain,
    Emboss,
    PaperVisibility,
    ExtDiffusion,
    ExtBackrun,
    ExtFingering,
    ExtRivulets,
    ExtGranulation,
    ExtStaining,
    ExtDryBrush,
    ExtWetSoften,
    ExtRakeLight,
    ExtValleyGran,
    ExtWetSheen,
    ExtGrainDither,
    ExtEdgeTint,
    /// Doc 23 P1: active re-wet lift — how strongly a wet-tool stamp
    /// dissolves SETTLED pigment back into suspension (watercolor is
    /// re-soluble; Rebelle's "Re-wet" is stroke-driven). Appended LAST so
    /// every earlier discriminant — and the registry order — is untouched.
    WetLift,
    /// **O Glaze do PINCEL** (prompt 42): a fracção da tinta deste pincel que FILTRA a de baixo (`0` = a lei
    /// da obra, ao byte; `1` = multiplica — o marcador do Rebelle). Pousa com a tinta no plano
    /// [`crate::grid::Grid::vela`]. Appended LAST, like `WetLift`.
    Glaze,
    /// **O PÓ do pincel** (prompt 43): cada dab pousa a tinta em PARTÍCULAS — uma fracção das células recebe
    /// a dose das outras (`0` = o carimbo contínuo, ao byte; `1` = uma célula em 20). O aerógrafo.
    Dust,
    /// **A pressão → o TAMANHO, do pincel** (prompt 44): quanto o raio cai com a pressão da caneta
    /// (`0,9` = a lei de sempre, o raio a 10 % na pressão 0; `0` = não muda). Lido pelo HOSPEDEIRO
    /// (`ph2d_painter_brush::Dynamics`); o motor não o vê.
    PressureSize,
    /// **A pressão → a OPACIDADE, do pincel** (prompt 44): quanto a tinta do dab cai com a pressão (`0` = a
    /// lei de sempre; `1` = nada na pressão 0 — o Flat do Rebelle). Lido pelo hospedeiro.
    PressureOpacity,
    /// **A CURVA da pressão → opacidade, do pincel** (prompt 45): a pressão sobe a ela antes da rampa (`1` =
    /// linear; `> 1` esmaece mais depressa no toque leve — o Flat do Rebelle ~1,75). Lido pelo hospedeiro.
    PressureOpacityCurve,
    /// **A caneta no pincel** (W6, `ph2d_painter_brush::pen`; doc 47_medicao) — todas lidas pelo hospedeiro,
    /// cada uma com o neutro = a lei de antes ao bit. A CURVA da pressão → tamanho: a MESMA lei da
    /// [`Knob::PressureOpacityCurve`], no raio (`1` = linear).
    PressureSizeCurve,
    /// O PISO da mão: a pressão que o pincel lê como `0` (`0` = nenhum).
    PressureFloor,
    /// O TECTO da mão: a pressão que o pincel já lê como `1` (`1` = nenhum; os lápis do Rebelle: 0,6).
    PressureCeiling,
    /// A SUAVIZAÇÃO da pressão ao longo do traço (`0` = nenhuma; a fracção que fica da anterior por amostra).
    PressureSmoothing,
    /// O «Pulled String»: o comprimento da corda em px (`0` = sem corda; toma o lugar do estabilizador).
    PulledString,
    /// A ROTAÇÃO do carimbo: `0` nenhuma · `1` segue o traço · `2` a inclinação da caneta.
    DabRotation,
    /// O ângulo à vertical (graus) onde a inclinação começa a contar.
    TiltStart,
    /// O ângulo à vertical (graus) onde a inclinação conta inteira.
    TiltFull,
    /// O raio ATRAVÉS do azimute com a caneta deitada, em raios (`1` = nenhum; o 8B do Rebelle: 1,2).
    TiltWidth,
    /// O raio AO LONGO do azimute com a caneta deitada, em raios (`1` = nenhum; o 8B do Rebelle: 3,3).
    TiltLength,
    /// O grão que o lado da ponta alisa, deitada e arrastada ao comprido (`0` = nada).
    TiltSmooth,
    /// O atraso da inclinação lida, em px de caminho (`0` = nenhum; o Rebelle: ~170).
    TiltLag,
    /// **Os fios com física própria** (W2, [`crate::bristles_fios::Fios`]), lidos pelo hospedeiro quando o Shape é
    /// Bristles; sem painel (W8). A amplitude da pressão de cada fio (`0` = a lei de antes).
    BristlePressure,
    /// O comprimento de caminho (px) em que a pressão de um fio e a oscilação mudam.
    BristleNoiseLength,
    /// A largura de um fio (px).
    BristleLaneWidth,
    /// A amplitude da onda da largura dos fios (`0` = fixa).
    BristleWave,
    /// O comprimento da onda da largura (px de caminho).
    BristleWaveLength,
    /// A oscilação do pincel na normal do traço (px; `0` = nenhuma).
    BristleOscillation,
    /// O «Random Span»: o desvio (px) de cada fio da sua faixa (`0` = nenhum).
    BristleSpan,
    /// O comprimento de caminho (px) em que o desvio de um fio muda.
    BristleSpanLength,
    /// O ruído de cor de cada fio em HSLuv: o matiz (voltas) — `0` = nenhum.
    BristleNoiseHue,
    /// … a saturação (fracção de 100).
    BristleNoiseSat,
    /// … a luminosidade (fracção de 100).
    BristleNoiseLight,
    /// Quanto a mão leve levanta os fios fracos (`0` = nada).
    BristleLift,
    /// O RELEVO de cada fio (W3): a altura (px) que um fio de ganho 1 deixa na tinta (`0` = nenhum).
    BristleHeight,
    /// … a fracção dos fios que ESCAVAM em vez de construir (o Remove impasto do Rebelle).
    BristleRemove,
    /// **O pigmento é por COMPRIMENTO de traço** (prompt 51): o Spacing (fracção do diâmetro ao longo do caminho) a
    /// que o [`Knob::PigmentPerDab`] foi afinado — cada dab pousa `passo ÷ o passo deste Spacing` do seu pigmento, e o
    /// traço não muda com o Spacing do pincel. `0` = a lei de antes (cada dab o pigmento inteiro).
    PigmentSpacing,
    /// O sub-passo da cápsula do Continuous (fracção do diâmetro; `capsula::SUB_SPACING` por omissão): o pincel cuja
    /// textura de cerdas anda com o dab pede-o à densidade a que foi afinado (a Acrylic Round, doc 51 §3).
    CapsuleSpacing,
    /// O tecto do sub-passo da cápsula em px (`0` = nenhum): os fios do Oil Bristle têm largura em px, e a 103 px a
    /// escada lê 2,24 a 2 px contra 4,71 a 3 (doc 51 §3).
    CapsuleMaxPx,
    /// O sub-passo da cápsula pelo inteiro de células mais PERTO em vez de para BAIXO (`1`;
    /// `capsula::sub_passo_mais_perto`): os pincéis secos de grão, onde o de baixo dobra o custo (doc 51 §4).
    CapsuleRoundNearest,
    /// ⭐ **O relevo pelo DIÂMETRO** (J2, 03_MOTORES §7 c): o expoente `γ` do relevo que a célula guarda,
    /// `Emboss · (ref ÷ D)^γ` com `D` o diâmetro do traço — o relevo das cerdas do Rebelle não cresce com o
    /// pincel (o Bristle Flat lê 15,6 a 41 px e 11,3 a 202). `0` = o de antes, ao bit.
    EmbossSizeExp,
    /// … o diâmetro (px) a que o `Emboss` do preset foi medido (o factor `1`).
    EmbossSizeRef,
    /// ⭐ **A ponta DESFIADA das cerdas** (J2, 03_MOTORES §7 c): a amplitude do desvio de cada fio ao longo do traço
    /// (a pegada dele deslocada; o desfiado das duas pontas) — `0` = a pegada de sempre, ao bit.
    BristleTipRagged,
    /// ⭐ **O nivelamento não atravessa para OUTRO traço** (J2, 03_MOTORES §7 d): o filme de um traço não escoa
    /// para a tinta molhada de outro (`1`); `0` = o nivelamento de sempre, ao bit.
    LevelingOwnStroke,
    /// ⭐ **A ponta de tinta LÍQUIDA é UM reservatório** (J3 tintas, `docs/Painter/Pinceis/exames/tintas.md` «2.ª
    /// onda»): a fracção `r` da tinta debaixo da ponta entra nela a cada transfer, mistura-se por SUBTRACÇÃO (o
    /// produto das duas tintas, nunca a média) e espalha-se pela pegada inteira — o azul escurece um raio ANTES da
    /// faixa e arrasta depois. `0` = a ponta de sempre (cada texel guarda o que tocou, pelo `Pickup`), ao bit.
    TipReservoir,
    /// ⭐ **A pressão leve desfaz o traço em GRÃO** (J4 pastel_carvao, `docs/Painter/Pinceis/exames/pastel_carvao.md`):
    /// a fracção `k` da opacidade da pressão ([`Knob::PressureOpacity`]) que entra ANTES do portão do papel — com pouca
    /// tinta só os picos do dente a vencem (os pontos soltos do pastel e do carvão do Rebelle a pressão leve); o resto
    /// multiplica depois, como sempre. `0` = a lei de sempre (a opacidade toda depois do portão), ao bit.
    PressureGrain,
    /// ⭐ **O MIOLO LISO da lavagem** (J4 aguarela, `docs/Painter/Pinceis/exames/aguarela.md` «Motor J4»): o traço deixa
    /// UM filme de tinta — cada célula fica com o MÁXIMO que um dab do traço lhe trouxe (a espessura do miolo da soma de
    /// sempre, `2r ÷ passo` dabs), nunca a soma dos que a cruzaram — e o tapete das cerdas não o texturiza (o Grain armado
    /// continua). Medido: as listras do miolo são as linhas do tapete arrastadas pelo traço, o domo é a soma das cordas.
    /// `0` = a soma de sempre, ao bit.
    StrokeFilm,
    /// ⭐ **A GOTA do pó por preset** (J4 pastel_carvao, `docs/Painter/Pinceis/exames/pastel_carvao.md`): o lado da gota
    /// do [`Knob::Dust`] em células — o pó grosso do carvão em pó e do aerógrafo do Rebelle (a gota fixa de 2 células dava
    /// o pó fino a todo o tamanho). `2` = a gota de sempre, ao bit (o grão do aerógrafo do Rebelle mede 1,28 px a 20, 40
    /// e 80 px; a gota de 2 dá 1,40 e a de 1,5 dá 1,17 — prompt 43, `43_medicao` §3).
    DustDrop,
    /// ⭐ **O VEIO de cada fio** (J4 oleo_acrilica, `docs/Painter/Pinceis/exames/oleo_acrilica.md` «2.ª ronda»): a carga de
    /// um fio que, a troços, pesa (o fio quase preto) ou falta (o claro) ao longo do traço — a luminosidade HSLuv do fio
    /// desce desta fracção de 100 nos troços escuros ([`crate::bristles_fios::Fios::veio`]). `0` (e o claro a `0`) = os
    /// fios de sempre, ao bit.
    BristleStreak,
    /// … a fracção de 100 que a luminosidade SOBE nos troços claros.
    BristleStreakLight,
    /// … o comprimento de caminho (px) em que o veio de um fio muda (o comprimento dos troços).
    BristleStreakLength,
    /// … o limiar do ruído do fio acima do qual um troço pesa ou falta (`0..1`: quanto maior, mais raros).
    BristleStreakCut,
    /// ⭐ **O pincel de cerdas FINO esmaece** (J4 oleo_acrilica, «2.ª ronda»): abaixo de [`Knob::ThinStrokeRef`] px de
    /// diâmetro a carga do dab cai como `(D ÷ ref)^γ` — as poucas cerdas de um chato pequeno não se sobrepõem (o `Flat`
    /// do Rebelle: o recto de 5 px cobre 0,22 do de 40). `0` = a carga de sempre, ao bit.
    ThinStrokeFade,
    /// … o diâmetro (px) a partir do qual a carga é a inteira.
    ThinStrokeRef,
    /// ⭐ **A ÁGUA EMPURRA o pigmento** (J4 mistura, L-6 Water Brush, `docs/Painter/Pinceis/exames/mistura.md` «Motor
    /// J4»): a fracção do pigmento SUSPENSO que o traço da ferramenta Wet leva para fora da pegada, para os dois lados,
    /// pela tinta molhada, até à frente da água ([`crate::tools::apply_wet_push`]) — o Water Soft do Rebelle abre branco
    /// na faixa molhada (0,13) e deixa a orla escura onde a água pára. `0` = a água de sempre, ao bit.
    WaterPush,
    /// ⭐ **O secador ASSENTA e deixa ORLA** (J4 mistura, L-7 Dry/Lift, `docs/Painter/Pinceis/exames/mistura.md` «Motor
    /// J4»): a fracção, por diâmetro de caminho, com que o dab da ferramenta Dry nivela a tinta suspensa da pegada e puxa
    /// para dentro da borda a tinta molhada ao lado ([`crate::tools::apply_dry_edge`]) — a mancha seca do Rebelle tem a
    /// linha escura por dentro. `0` = o secador de sempre, ao bit.
    DryerEdge,
}

pub const KNOB_COUNT: usize = 105;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnobGroup {
    Paint,
    Water,
    Physics,
    Tools,
    Paper,
    /// SPEC §17 gated extensions: no visible sliders, neutral defaults.
    Hidden,
}

/// What a knob write invalidates (the facade acts on it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rebuild {
    /// Rebuild the bristle stamp texture.
    Brush,
    /// Re-bake the paper tile (on slider release).
    Paper,
    /// Appearance only: repaint.
    Render,
}

mod defs;

pub use defs::{KNOB_DEFS, KnobDef};

/// The boot value of every knob, in [`Knob`] discriminant order — `const` so
/// a host's authored-state `DEFAULT` (a `const` item) can carry the exact
/// engine boot (f64 equality gates depend on it).
#[must_use]
pub const fn knob_defaults() -> [f64; KNOB_COUNT] {
    let mut values = [0.0; KNOB_COUNT];
    let mut i = 0;
    while i < KNOB_COUNT {
        values[i] = KNOB_DEFS[i].default;
        i += 1;
    }
    values
}

/// Live knob values. Writes report what they invalidated; the facade reacts
/// (lazy brush rebuild, paper re-bake on release, repaint).
#[derive(Clone)]
pub struct Tuning {
    values: [f64; KNOB_COUNT],
}

impl Default for Tuning {
    fn default() -> Self {
        Tuning {
            values: knob_defaults(),
        }
    }
}

impl Tuning {
    #[inline]
    pub fn get(&self, knob: Knob) -> f64 {
        self.values[knob as usize]
    }

    /// Copy of the whole value array (the per-step params snapshot).
    #[inline]
    pub fn snapshot(&self) -> [f64; KNOB_COUNT] {
        self.values
    }

    pub fn def_of_key(key: &str) -> Option<&'static KnobDef> {
        KNOB_DEFS.iter().find(|d| d.key == key)
    }

    /// Set a knob. NaN (a garbled numeric input) falls back to the default.
    /// Returns the def when the stored value actually changed.
    pub fn set(&mut self, knob: Knob, v: f64) -> Option<&'static KnobDef> {
        let d = &KNOB_DEFS[knob as usize];
        let nv = if v.is_nan() { d.default } else { v };
        if self.values[knob as usize] == nv {
            return None;
        }
        self.values[knob as usize] = nv;
        Some(d)
    }

    /// Panel-side write by registry key. Unknown keys are ignored.
    pub fn set_by_key(&mut self, key: &str, v: f64) -> Option<&'static KnobDef> {
        let d = Self::def_of_key(key)?;
        self.set(d.knob, v)
    }

    pub fn reset(&mut self, knob: Knob) -> Option<&'static KnobDef> {
        self.set(knob, KNOB_DEFS[knob as usize].default)
    }

    /// Reset every knob of a group. Returns the defs that actually changed —
    /// the caller MUST act on their `rebuild` kinds exactly as it would for
    /// single `set` calls (the JS fires onChange per knob; swallowing these
    /// left stale brush textures / unbaked paper — port-verify finding).
    #[must_use = "act on the rebuild kinds of the changed knobs"]
    pub fn reset_group(&mut self, group: KnobGroup) -> Vec<&'static KnobDef> {
        let mut changed = Vec::new();
        for d in KNOB_DEFS.iter() {
            if d.group == group
                && let Some(def) = self.set(d.knob, d.default)
            {
                changed.push(def);
            }
        }
        changed
    }

    pub fn is_default(&self, knob: Knob) -> bool {
        self.values[knob as usize] == KNOB_DEFS[knob as usize].default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_order_is_the_enum_order() {
        for (i, d) in KNOB_DEFS.iter().enumerate() {
            assert_eq!(d.knob as usize, i, "KNOB_DEFS[{i}] holds {:?}", d.knob);
        }
    }

    #[test]
    fn keys_are_unique() {
        for (i, a) in KNOB_DEFS.iter().enumerate() {
            for b in KNOB_DEFS.iter().skip(i + 1) {
                assert_ne!(a.key, b.key);
            }
        }
    }
}
