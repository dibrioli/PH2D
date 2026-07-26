//! **The film** — where a body of paint ENDS, and how thick it is at that edge.
//!
//! One curve ([`body_profile`]) on one pair of thresholds ([`W_TAIL`] / [`W_SOLID`]), read by three
//! things that must never disagree about what counts as paint:
//!
//! - the **relief** ([`crate::height::derive_height`]) — the body it stands up,
//! - the **light** (`impasto_light::paint_body`) — the shading it is allowed to weigh,
//! - the **pigment** ([`film_coverage`]) — the colour the brush is allowed to lay.
//!
//! The third one is new (Enio, 2026-07-12). Until it existed the first two agreed and the pigment did
//! not: the colour ran out to the dab's geometric rim, so every impasto stroke wore a skirt of paint the
//! light was RIGHT to refuse to model — a haze around the ridge, whose width was a pure function of the
//! falloff. Sibling module of [`crate::height`], for the file-LOC cap and because it is its own idea.

/// Coverage below which the paint carries NO body: everything thinner is the STAIN — pigment rubbed
/// into the paper, not paint you could stand a wall on. It is deliberately high: the wall must rise
/// INSIDE the pigmented part of the stroke (Photoshop's bevel runs from the matte's edge *inward*,
/// over solid pixels, for the same reason) — a wall standing on the translucent rim gets its strong
/// lighting multiplied into pixels that are mostly PAPER, which is the white-canvas halo all over
/// again (`impasto_light_shades_the_paint_not_the_paper_showing_through_it` refuses it). // CLAMP-OK
pub const W_TAIL: f32 = 0.35;

/// Coverage at which the paint is a SOLID film: full thickness from here inward. Shared with the
/// light pass (its coverage weighting rides the same [`body_profile`]), so "solid paint" is one
/// concept with one pair of numbers on both sides of the pipeline. // CLAMP-OK
pub const W_SOLID: f32 = 0.75;

/// The **body curve**: how the dab's silhouette becomes the paint's *body*.
///
/// `h = depth × coverage × w` copies the colour's own soft profile into the relief — and for the
/// default brush (hardness 0) that is a dome the full width of the stroke, which reads as a blur, not
/// as paint (measured: shading peak 7.3 levels at 31% of the half-width; 1 level at the visible
/// edge). Nobody in the state of the art does that: Photoshop's bevel is a distance profile from the
/// coverage EDGE (Chisel), Blender's Layer brush caps at a fixed height ("creates the appearance of a
/// flat layer"), Hertzmann (NPAR 2002) gives height its own texture, and Painter documents Uniform as
/// "even depth". See `docs/Painter/17_impasto_deposito_pesquisa2.md` §3.
///
/// So the height gets its own profile: a **plateau** wherever the paint is solid (`w ≥ W_SOLID`),
/// **nothing** over the stain (`w ≤ W_TAIL` — the translucent rim keeps its pigment and stays flat),
/// and the **shoulder** — the wall the light lives on — in between, standing on pigment-backed
/// pixels a little inside the stain's edge, the way a real paint film ends inside its own smear.
/// Because a falloff is monotone in distance, remapping `w` IS a profile-in-distance (the bevel), it
/// commutes with the stroke envelope's `max`, and rule 1 still holds: the height consumes exactly
/// the silhouette the colour consumes. The shoulder's width follows the brush's own softness — a
/// soft brush lays a softer body edge — which is the coupling that should exist; the dome was the
/// one that shouldn't.
#[inline]
#[must_use]
pub fn body_profile(w: f32) -> f32 {
    let t = ((w - W_TAIL) / (W_SOLID - W_TAIL)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The normalised dab distance `t` at which a bare-falloff silhouette's BODY ends — where its
/// coverage first drops to [`W_TAIL`], the SAME threshold [`film_of`] uses to decide a texel carries
/// paint at all. Inside this `t` the paint has a body; beyond it there is only the translucent stain,
/// then bare canvas.
///
/// It is the anchor the displaced paint must bank from: **a brush shoves paint to where the paint
/// STOPS**, which is the body's edge — not the dab's geometric rim (`t = 1`). Anchoring the rim at
/// `t = 1` on a soft falloff stood a hard, perfectly circular collar of banked paint a whole
/// silhouette's-width of BARE CANVAS out from the soft paint it was meant to be shoved by — Enio's
/// smoke, 2026-07-15: *"é usada a circunferência do gizmo do brush para empurrar a massa e não o
/// alpha do falloff"*. Under `Smooth`, `W_TAIL` sits at `t ≈ 0.61` (§14.1 of doc 16): on a 40 px
/// brush the visible paint ends at ~24 px and the old rim was born at 40 px — 16 px of naked canvas
/// between them. This closes that gap; the ridge's foot now rises from the body's edge.
///
/// A **hard-edged** falloff (Constant, or hardness ≥ 1) carries its body right out to the geometric
/// rim, so its edge IS `t = 1` and the rim anchors exactly where it always did — **byte-identical**,
/// and there was no soft skirt to leave a gap in the first place. The [`RIM_PROBE`] guard returns
/// exactly `1.0` for those, so the bisection never approximates a value a hair below it.
///
/// The silhouette is monotone in `t`, so this is a bisection on [`crate::BrushSpec::falloff_weight`]
/// (which folds in the brush's hardness) — cheap, and meant to be hoisted **once per stroke** (the
/// falloff and hardness do not change mid-stroke). The **Shape** slot has no radial body edge (an
/// Image tip is a stamp, a procedural one is masked per-pixel); the caller keeps `t = 1` there — see
/// [`crate::height_push::rim_t0`].
#[must_use]
pub fn body_edge_t(spec: &crate::BrushSpec) -> f32 {
    /// A hair inside the geometric rim: if the silhouette is still solid here, the body reaches the
    /// edge (Constant / hardness ≥ 1) and the anchor is exactly `t = 1`. No soft falloff comes within
    /// `W_TAIL` of the rim (Sphere, the flattest, is ~0.045 here), so this fires only for hard tips.
    const RIM_PROBE: f32 = 0.999;
    if spec.falloff_weight(RIM_PROBE) >= W_TAIL {
        return 1.0;
    }
    // Bisection: `falloff_weight` decreases monotonically from 1 at the centre to 0 at the rim, so the
    // invariant `weight(lo) ≥ W_TAIL > weight(hi)` holds from `lo = 0` / `hi = 1` and is preserved.
    // 24 halvings ⇒ ~6e-8 px of `t`, far under a texel; `lo` is the last distance that still has body.
    let mut lo = 0.0f32;
    let mut hi = 1.0f32;
    for _ in 0..24 {
        let mid = 0.5 * (lo + hi);
        if spec.falloff_weight(mid) >= W_TAIL {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// The **film** the brush actually lays: a dab's coverage, cut to the paint that carries a BODY.
///
/// ## The bug this closes
///
/// Enio, 2026-07-12, two screenshots of the same crossing strokes: *"o efeito leva em consideração os
/// limites do pincel e não o peso do relevo. Este falloff (smooth) pinta tinta fora do relevo. Usando o
/// falloff Sphere fica mais preciso e a tinta corresponde ao relevo."*
///
/// He had found a real seam. Two things already agreed on where paint stops carrying a body — the relief
/// ([`body_profile`], zero below [`W_TAIL`]) and the light (it weighs its shading by the same curve, so
/// it will not bleach the paper showing through a translucent rim). The **pigment** knew nothing about
/// it: the colour kernel deposited all the way out to the dab's geometric rim. So every impasto stroke
/// carried a skirt of paint that the light was *right* to refuse to model, and it read as a haze around
/// the ridge.
///
/// Its width is a pure function of the falloff, which is exactly why the falloff looked like the culprit:
/// coverage `W_TAIL` sits at `t = 0.61` under `Smooth` (39% of the radius — 16 px on a 40 px brush) and
/// at `t = 0.94` under `Sphere` (6%). Sphere was not more *precise*; it simply has almost no skirt.
///
/// ## The rule
///
/// **A brush that lays no body lays no paint.** One curve, one threshold, one definition of "paint":
/// wherever the light gives a pixel no shading, the brush gives it no pigment either.
///
/// It closes exactly because the light's weight is `body_profile(cover)` and `cover` is the RAW paint
/// (`silhouette × dynamics`), which this does not touch: the film is that same curve on that same
/// quantity, so the pigment's support and the lit region are the *same set*, at every falloff and every
/// pressure. The stroke does not come out narrower — the lit ridge was already only `t < 0.61` wide; what
/// goes is the haze around it.
///
/// It also leaves the ingredients alone, so the whole Body card stays live: Depth, Body, Depth Source,
/// Smoothing and Push still re-derive the last stroke. `Body` shapes the *relief's profile* (dome ↔ slab)
/// over a film whose edge is fixed — which is the right split, because a film's edge is a fact of the
/// paint that is already down, and its cross-section is not.
///
/// ## The cut belongs to the SILHOUETTE — not to the grain, and not to the dynamics
///
/// `sil` is the dab's bare silhouette (falloff, or a Shape image) — **before** the Grain multiplies it and
/// **before** the dab's pressure × Flow × Strength scale it. Both exclusions were paid for:
///
/// - **Not the dynamics.** Cutting the dab's *full* coverage silently kills the brush: at Strength 0.5 the
///   peak is 0.25, under [`W_TAIL`], so the curve returns zero for every texel and the stroke deposits
///   **nothing at all** (`the_film_never_starves_the_brush_at_low_strength`). The physics agrees: a film's
///   edge is a property of the tip, not of how hard you press. A light touch lays a THINNER film, not a
///   film with a different outline.
/// - **Not the grain.** The relief the light weighs (`cover`) is the silhouette × dynamics — the Grain is
///   deliberately *not* in it, because a Grain textures the pigment and does not carve the body
///   ([`DepthSource::Uniform`]). Cut the film through the grain and its valleys lose their pigment while
///   keeping their full body: the light then shines, at full strength, on bare paper. Measured at 124
///   levels over 1694 px before this was moved (`impasto_light_does_not_shade_paint_that_is_not_there`).
///
/// So the film reshapes the **silhouette**, once, and everything downstream — grain, dynamics, the
/// Accumulate-OFF cap, the ramps, the per-layer colour — consumes the reshaped one and needs no
/// arithmetic of its own. The relief consumes the RAW silhouette (`cover = sil × dynamics`), so the
/// light's weight `body_profile(cover)` is the film's own alpha: the pigment's support and the lit region
/// are the same set, which is the property the gate states.
#[inline]
#[must_use]
pub fn film_coverage(lays_body: bool, sil: f32) -> f32 {
    if lays_body { film_of(sil) } else { sil }
}

/// The film a bare silhouette leaves: its body ([`body_profile`]) seen as an OPACITY
/// ([`film_opacity`]). The two ends are constants, and skipping them is not a micro-optimisation — the
/// tail is most of a dab's bounding box, and this runs per texel per dab in the height kernel.
#[inline]
#[must_use]
pub fn film_of(sil: f32) -> f32 {
    if sil <= W_TAIL {
        return 0.0; // the stain: no body, so no paint
    }
    if sil >= W_SOLID {
        return 1.0; // solid paint, and opaque long before this
    }
    film_opacity(body_profile(sil))
}

/// How **opaque** a film of paint of this thickness is.
///
/// ## Opacity is not thickness — and getting that wrong left the haze standing
///
/// Enio, 2026-07-12, third smoke: *"regrediu ao deixar a tinta extravasar o relevo e não resolveu a
/// distância da tinta levantada."* The film was cutting the pigment at the body's edge, and the support
/// of the two matched exactly — the gate said so and the gate was right. It was also **too weak to see
/// what he saw**: the alpha *ramped* with the body over ~8 px, so the stroke ended in a soft gradient of
/// pale red carrying no 3D form at all. A soft gradient with no form IS a haze. Measured, on the smoke's
/// own brush: ink 227 → 0 across `t = 0.38 … 0.57`, with the shading fading in lockstep — the two agree
/// perfectly and the edge still reads as fog.
///
/// The physics I had wrong: a film's **opacity saturates long before its thickness does**
/// (Beer–Lambert). Oil paint at a tenth of full thickness is already all but opaque — that is why a
/// palette knife leaves an EDGE and not a gradient. Modelling alpha as proportional to body was
/// modelling paint as glass.
///
/// So the deposit's alpha is the film's opacity: `1 − (1 − d)⁸`, saturating fast, transcendental-free
/// (HR-5 — three squarings, no `exp`). The paint is opaque right up to where the body ends and then it
/// **stops**, over the ~1 px the silhouette needs to fall the rest of the way. That is also why Enio's
/// `Sphere` looked right: its silhouette is nearly flat to the rim, so its film reached full body over a
/// pixel or two — it was already doing this, by accident of shape. Now every falloff does.
///
/// It closes the halo from the other side too, and that is not a coincidence: the pass may not bleach the
/// **paper seen through translucent paint**, and this is the function that says there is barely any
/// translucent paint to see through. // CLAMP-OK
#[inline]
#[must_use]
pub fn film_opacity(d: f32) -> f32 {
    let q = (1.0 - d.clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let q2 = q * q;
    let q4 = q2 * q2;
    1.0 - q4 * q4
}

/// How much **solid paint** one dab lays at a texel — the quantity the light weighs its shading by.
///
/// `sil` is the bare silhouette, `dynamics` the dab's folded pressure × Flow × Strength. It is exactly
/// [`film_coverage`] scaled by the dynamics — the film's own alpha — and the factorisation is the point.
///
/// ## The hole it closes (Enio's bug, at partial pressure)
///
/// The light used to weigh by `body_profile(cover)` where `cover` was the raw paint, `sil × dynamics`.
/// The dynamics were INSIDE the curve, so they could starve it: at Flow × Strength × pressure ≈ 0.35 the
/// argument falls under [`W_TAIL`] for every texel and the light models **nothing anywhere on the
/// stroke** — while the pigment (cut on the silhouette, [`film_coverage`]) is still there. That is the
/// haze all over again, and it is the exact species of bug the film was written to kill; it merely hid
/// behind the mouse, which always presses at 1.0.
///
/// It is the same theorem as the film's, applied to the other side: **the threshold belongs to the
/// silhouette; the dynamics multiply afterwards.** A light touch lays a THINNER film — less pigment, less
/// body, less light — not a film the light refuses to look at.
///
/// At full dynamics `dynamics × body_profile(sil)` and `body_profile(sil × dynamics)` are the same
/// number, so the light is unchanged for every stroke drawn with a mouse — and every gate that pins its
/// look was drawn with one.
#[inline]
#[must_use]
pub fn solid_paint(sil: f32, dynamics: f32) -> f32 {
    (dynamics * film_of(sil)).clamp(0.0, 1.0)
}

// ── Screen-space AA of the film silhouette (BUGS #16, impasto half — Enio 2026-07-20) ──────────────

/// Sub-texel offsets of the AA reconstruction grid (3×3, ±0.667 texel) — the same grid the
/// watercolor AA settled on (`watercolor_field::AA_SS`): wider than the unit box so a footprint of
/// ~1.3 texels reconstructs a sub-texel film step as a soft ramp. `LITERAL-PX-OK`: AA geometry.
const AA_SS: [f32; 3] = [-0.667, 0.0, 0.667]; // LITERAL-PX-OK

/// Screen-space anti-aliasing of the film's silhouette, hoisted **once per dab**.
///
/// The film's transition occupies a FIXED interval of `t = distance/radius` (a property of the
/// falloff shape), so its screen width shrinks with the radius: a thin impasto stroke's edge crosses
/// it in well under a texel and snaps to a binary stair-step — the impasto half of the watercolor's
/// "borda dura pixelada" (BUGS #16; measured: radius 10 jumps `255 → 0` in one texel). Unlike the
/// watercolor, the impasto pigment's alpha **composites linearly** — so the film's fractional
/// texel-area coverage can replace `film_of(sil)` directly, in BOTH consumers (the pigment funnel in
/// `dab.rs` and the light's coverage envelope in `height.rs`), and the invariant the film exists for
/// — pigment support == lit region — holds texel-for-texel by construction.
///
/// `None` (checkbox off / no body laid / a Shape image tip) = the callers keep the single-sample
/// `film_of`, byte-for-byte. A Shape image is a STAMP and its edge is hard by design (the same
/// precedent as `body_edge_t` / `rim_t0`: an image tip has no radial body edge).
pub struct FilmAa {
    /// `t` below which the film is solidly 1 for every subsample (band start − sub-texel reach).
    t_lo: f32,
    /// `t` above which the film is solidly 0 for every subsample (band end + sub-texel reach).
    t_hi: f32,
}

impl FilmAa {
    /// Build the per-dab AA plan, or `None` when the single-sample path applies (see type docs).
    #[must_use]
    pub fn for_dab(spec: &crate::BrushSpec, shape_active: bool, radius: f32) -> Option<Self> {
        if !spec.film_aa_wanted(shape_active) || radius <= 0.0 {
            return None;
        }
        // Sub-texel reach in `t` units: the grid's diagonal extent (0.667·√2 texels) over the radius.
        let reach_t = 0.943 / radius; // LITERAL-PX-OK: 0.667·√2, the AA grid's diagonal reach
        // The band's inner edge is where the film SATURATES, not where it reaches `W_SOLID`:
        // `film_of` is sub-quantum from 1.0 (< 1/512 in u8) by sil ≈ 0.58, well before the body is
        // solid at 0.75 — supersampling the stretch between the two averages a constant, and at a
        // big radius that stretch is most of the ring (Sphere r=100: 28 → 12.5 texels, measured
        // ~+2 ms/move of pure waste in the perf kill gate).
        const FILM_SATURATED_SIL: f32 = 0.58; // LITERAL-PX-OK: film_of ≥ 1 − 5.2e-4 here (u8-exact)
        let (t_solid, t_tail) = if matches!(spec.falloff, crate::Falloff::Custom) {
            // A Custom curve need not be monotone, so a bisection could find A crossing instead of
            // THE band — take the whole dab as the band (correct, just unhoisted).
            (0.0, 1.0)
        } else {
            (cross_t(spec, FILM_SATURATED_SIL), cross_t(spec, W_TAIL))
        };
        Some(Self {
            t_lo: t_solid - reach_t,
            t_hi: t_tail + reach_t,
        })
    }

    /// The film at a texel: outside the rim band, exactly `film_of(sil_centre)` — the old single
    /// sample, byte-identical for the whole interior and exterior; inside, the **mean of `film_of`
    /// over the sub-texel grid** — the texel's fractional area under the film. `sil_at(ox, oy)` is
    /// the caller's own silhouette chain evaluated at the sub-texel offset, so the fraction follows
    /// whatever geometry the caller stamps (disc or swept capsule) with no second formula.
    ///
    /// # ⛔ A redução de amostras foi CONSTRUÍDA e REJEITADA por medição (plano 26 §9.5)
    ///
    /// O AA é **54% de um traço de impasto** (68,7 de 127,1 ms a raio 100, medido em
    /// `ph2d-tool-painter`'s `measure_impasto_cost`), porque os DOIS consumidores o pagam sobre a banda
    /// do aro de CADA dab, e a 0,1 de spacing um texel do aro cai dentro de ~10 bandas. Cortar as nove
    /// amostras valia ~30% do traço, e o caro é o **`sil_at`** (a cadeia inteira do chamador), não o
    /// `film_of` (um punhado de multiplicações).
    ///
    /// A tentativa: amostrar de verdade só a **cruz** (centro + os quatro vizinhos axiais, 5 chamadas)
    /// e estimar as quatro **QUINAS** pela extensão separável
    /// `s(ox,oy) ≈ s00 + [s(ox,0) − s00] + [s(0,oy) − s00]` — a MESMA média de `film_of` sobre a MESMA
    /// grade, exata para qualquer silhueta separável, com o erro sendo só a derivada segunda MISTA.
    ///
    /// A varredura de paridade (`height_film_aa_tests`) a matou em duas frentes:
    ///
    /// 1. **`Constant` erra `2/9` EXATO — 56,67 níveis de u8 — em todo raio e em TODOS os texels da
    ///    banda** (788 de 788 a r=100). Com borda dura o `film_of` é um DEGRAU e a extensão separável
    ///    erra dois dos nove termos por construção. É o caso pelo qual o AA existe.
    /// 2. **O erro dos falloffs suaves NÃO é monotônico no raio:** `Sharp` erra 0,13 nível a r=40,
    ///    **2,02 a r=50**, 0,03 a r=60, **2,01 a r=70**; `Sphere` 0,13 a r=80, **2,01 a r=90**, 0,07 a
    ///    r=100. Picos isolados em raios arbitrários ⇒ **nenhum limiar de raio limita o erro**, e um
    ///    limiar tirado da tabela seria o *"limite que só diz por segurança"* que o §0 proíbe.
    ///
    /// ⚠️ **E não há troca de quadratura que salve:** com menos amostras a cobertura de um DEGRAU fica
    /// quantizada mais grosso (4 pontos ⇒ quartos, contra nonos), então uma grade rotacionada de 4 é
    /// *pior* justamente no caso 1. **O custo do AA é a contagem de amostras, e a qualidade dele
    /// também é** — não há almoço grátis nesse eixo.
    ///
    /// ⛔ E o `clamp(0,5 − f/|∇f|)` dos livros (o AA de SDF) foi rejeitado ANTES, no papel: ele troca a
    /// *curva* junto com as amostras, assumindo que a cobertura é a área de um lado de uma reta — e
    /// `film_of` não é isso (é uma S saturando em `W_SOLID`).
    ///
    /// A avenida que sobra é OUTRA: `film_of ∘ falloff_weight` é uma função 1-D **fixa** de `t`, então a
    /// média sobre o texel pode sair de uma **LUT pré-convoluída em `(t, |∇t|)`** hoisteada por dab —
    /// zero amostras extras, em vez de cinco de nove. Precisa de aceitação própria (o `|∇t|` da cápsula
    /// varrida não é constante perto das calotas).
    ///
    /// [`Self::film_at_exact`] nasceu dessa tentativa e **FICA**: é o oráculo que a varredura de
    /// paridade mede, e é o que torna a próxima tentativa mensurável em vez de opinável.
    #[inline]
    #[must_use]
    pub fn film_at(&self, t: f32, sil_centre: f32, sil_at: impl Fn(f32, f32) -> f32) -> f32 {
        self.film_at_exact(t, sil_centre, sil_at)
    }

    /// **A LUT pré-convoluída (plano 26 §9.6): a grade sem um único `sqrt`.**
    ///
    /// O que a medição (`measure_impasto_cost::is_the_silhouette_chain_the_aa_cost`) disse: **~91% do
    /// custo do `film_at` é a CADEIA de silhueta** — a closure real custa **10,3× a 12,3×** uma closure
    /// que devolve constante, com as nove chamadas de `film_of` e o laço intactos nas duas. E lida a
    /// cadeia, o que ela tem por amostra é **`sqrt`**: um em [`crate::Footprint::falloff_t`] (a norma do
    /// ponto deformado) e, em `Sphere`/`Root`, outro em `Falloff::weight` ⇒ **até 18 `sqrt` dependentes
    /// por texel da banda**. Não é a curva que é caro; é a RAIZ.
    ///
    /// # A derivação, e por que ela vale para TODA geometria que os dois kernels usam
    ///
    /// Os dois consumidores computam `t = |A·(r/radius)|`, onde `A` é o afim do footprint (rotação +
    /// flatten) e `r` é o resíduo — o offset ao centro no pigmento, e o resíduo da **CÁPSULA** na altura.
    /// Escrevendo `w = A·(r/radius)`, temos `t = |w|`, e **no espaço deformado tudo é euclidiano**:
    ///
    /// ```text
    ///   t(o) = |w + e|  ≈  t + (ŵ·e) + [|e|² − (ŵ·e)²] / (2t),   com e = B·o
    /// ```
    ///
    /// onde `B` leva um offset de texel ao espaço deformado. É isso que torna a fórmula geral em vez de
    /// um caso do disco redondo — a versão anterior desta função assumia `t = |d|/r` euclidiano e
    /// **errava sob Flatten & Rotate**, silenciosamente.
    ///
    /// `B` é linear em CADA região, e é o chamador que sabe qual:
    ///
    /// * **pigmento (disco):** `r = d` ⇒ `B = A/radius`;
    /// * **altura, CALOTAS da cápsula** (`d·u ∉ (0, back)`): `r = d − s·u` com `s` constante ⇒ o mesmo
    ///   `B = A/radius`, porque uma calota **é** um disco centrado no extremo;
    /// * **altura, BANDA da cápsula** (`0 < d·u < back`): `r = P·d` com `P = I − uuᵀ` ⇒ `B = A·P/radius`.
    ///
    /// ⚠️ **E na BANDA o termo de 2ª ordem é EXATAMENTE ZERO** — não aproximadamente. `P` projeta no
    /// complemento de `u`, que em 2D é **1-D**, então `P·o` e `P·d` são múltiplos do mesmo vetor; `A`
    /// preserva o paralelismo ⇒ `e ∥ w` ⇒ `|e|² = (ŵ·e)²`. É a afirmação geométrica *"distância a uma
    /// reta é afim"*, saindo da álgebra em vez de ser assumida. A cápsula, que era a incógnita desta
    /// wave, é o caso **mais fácil** dela.
    ///
    /// A única fonte de erro que sobra, além da 3ª ordem: um texel cujas sub-amostras **cruzam** a
    /// fronteira calota↔banda, onde o `B` correto muda no meio da grade. O gate mede esse conjunto.
    ///
    /// ⚠️ **Por que este erro NÃO é o da estimativa separável rejeitada na §9.5:** lá as quinas erravam
    /// o VALOR por `2/9` sempre que o `film_of` era um degrau — estrutural, em todo texel, em todo raio.
    /// Aqui os nove valores continuam vindo da mesma função `F`; o que desloca é a **POSIÇÃO** em que ela
    /// é lida, por um termo de 3ª ordem.
    ///
    /// `bx`/`by` são `B·(1,0)` e `B·(0,1)` — **constantes por dab**, e é por isso que a conta por texel é
    /// só duas produtos-internos e uma recíproca.
    #[must_use]
    pub fn film_at_lut(
        &self,
        lut: &FilmLut,
        t: f32,
        w: [f32; 2],
        bx: [f32; 2],
        by: [f32; 2],
    ) -> f32 {
        if t <= self.t_lo || t >= self.t_hi {
            // Fora da banda o valor é o single-sample, e a LUT o serve exatamente como a curva serviria.
            return lut.at(t);
        }
        // Por TEXEL: a normal do campo no espaço deformado, e as projeções da base nela.
        let inv_t = 1.0 / t.max(1e-6);
        let (nx, ny) = (w[0] * inv_t, w[1] * inv_t); // ŵ, unitário porque t = |w|
        let ax = nx * bx[0] + ny * bx[1]; // ŵ·bx
        let ay = nx * by[0] + ny * by[1]; // ŵ·by
        // Por DAB (o compilador as hoista do laço de texels; ficam aqui para a fórmula ser legível):
        let bxx = bx[0] * bx[0] + bx[1] * bx[1];
        let byy = by[0] * by[0] + by[1] * by[1];
        let bxy = bx[0] * by[0] + bx[1] * by[1];
        let half_inv_t = 0.5 * inv_t;
        let mut acc = 0.0;
        for &oy in &AA_SS {
            for &ox in &AA_SS {
                let along = ox * ax + oy * ay; // ŵ·e
                // |e|² = ox²·|bx|² + 2·ox·oy·(bx·by) + oy²·|by|²
                let e2 = ox * ox * bxx + 2.0 * ox * oy * bxy + oy * oy * byy;
                acc += lut.at(t + along + (e2 - along * along) * half_inv_t);
            }
        }
        Self::toe(acc * (1.0 / (AA_SS.len() * AA_SS.len()) as f32))
    }

    /// The **reference**: the film averaged over the grid with all nine silhouette values SAMPLED.
    ///
    /// This is what [`Self::film_at`] approximated until 2026-07-26, kept as the oracle the parity
    /// gates measure the épsilon against — the same template the impasto light's CPU×GPU parity uses
    /// (a canonical kernel plus a declared budget, never *"bit-a-bit"*, which is not this project's
    /// policy).
    #[must_use]
    pub fn film_at_exact(&self, t: f32, sil_centre: f32, sil_at: impl Fn(f32, f32) -> f32) -> f32 {
        if t <= self.t_lo || t >= self.t_hi {
            return film_of(sil_centre);
        }
        let mut acc = 0.0;
        for &oy in &AA_SS {
            for &ox in &AA_SS {
                acc += film_of(sil_at(ox, oy));
            }
        }
        Self::toe(acc * (1.0 / (AA_SS.len() * AA_SS.len()) as f32))
    }

    /// Os limites da banda, para o gate de paridade construir um `t` no MEIO dela (o único lugar onde
    /// a estimativa das quinas é exercitada).
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn t_lo_for_test(&self) -> f32 {
        self.t_lo
    }

    /// Irmão do [`Self::t_lo_for_test`].
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn t_hi_for_test(&self) -> f32 {
        self.t_hi
    }

    /// The toe cut, at the DOOR and shared by both kernels: below two u8 quanta the pigment blend and
    /// the film byte quantize INDEPENDENTLY (one rounds to nothing, the other to 1) and the light
    /// shades bare canvas — the support equality the film exists for, broken by rounding. Both
    /// consumers inherit the same cut, so the supports stay one set.
    #[inline]
    fn toe(frac: f32) -> f32 {
        if frac < 2.0 / 255.0 { 0.0 } else { frac }
    }

    /// Whether the AA is active for this dab — the bbox pads by one texel so the outermost
    /// fractional ring is not clipped at small radii (a texel whose CENTRE lies past the geometric
    /// rim can still be partially covered).
    #[must_use]
    pub fn pad_px(aa: &Option<Self>) -> f32 {
        if aa.is_some() { 1.0 } else { 0.0 }
    }
}

/// **A tabela de `F(t) = film_of(falloff_weight(t))`** — a curva do filme, pré-computada.
///
/// Construída **uma vez por traço** (o falloff e a hardness não mudam no meio de um) e emprestada por
/// referência aos dois kernels: uma alocação por traço, nunca por dab, senão o `hot_path_no_alloc` do
/// depósito acende — e com razão.
///
/// ⚠️ **A tabela cobre `[0, 1]` inteiro, não só a banda**, e isso é decisão: o `film_at_lut` precisa
/// dela também fora da banda (o early-out do single-sample) e nas amostras que o gradiente joga PARA
/// fora dela, e uma tabela de banda com clamp nas pontas mentiria exactamente nos texels de borda que o
/// AA existe para desenhar. A resolução vem do tamanho, não do recorte.
pub struct FilmLut {
    /// `N + 1` amostras de `F` em `t = i/N`, para a interpolação linear ter o vizinho da direita sem
    /// caso especial no último índice.
    table: Vec<f32>,
    /// `N` como `f32`, o fator de índice — guardado para o `at` não converter por chamada.
    n: f32,
}

impl FilmLut {
    /// Amostras da tabela. **16384** é o mesmo tamanho que a tabulação da transferência sRGB do doc 24
    /// escolheu, e pelo mesmo motivo: a resolução em `t` (6,1e-5) fica **abaixo** do erro que a
    /// linearização da métrica já introduz (~4,4e-5 no aro de um raio 100), então a tabela não é o termo
    /// dominante do épsilon — o que a torna um detalhe de implementação em vez de um segundo knob.
    pub const N: usize = 16_384;

    /// Tabela para este pincel. `F` é avaliada com as MESMAS funções do produto (`falloff_weight` e
    /// `film_of`), então a tabela não pode discordar da curva: ela **é** a curva, amostrada.
    #[must_use]
    pub fn new(spec: &crate::BrushSpec) -> Self {
        let n = Self::N;
        #[expect(
            clippy::cast_precision_loss,
            reason = "N = 16384 é exato em f32; o índice é um contador pequeno"
        )]
        let inv = 1.0 / (n as f32);
        let mut table = Vec::with_capacity(n + 1);
        for i in 0..=n {
            #[expect(clippy::cast_precision_loss, reason = "i <= 16384, exato em f32")]
            let t = (i as f32) * inv;
            table.push(film_of(spec.falloff_weight(t)));
        }
        #[expect(clippy::cast_precision_loss, reason = "N = 16384 é exato em f32")]
        let nf = n as f32;
        Self { table, n: nf }
    }

    /// **`raio × minor` mínimo para a LUT ser oferecida** — medido, não escolhido.
    ///
    /// O erro da expansão é o resto de 3ª ordem, logo escala com a **CURVATURA** da silhueta, e a
    /// curvatura é governada pelo **menor raio local** ([`crate::FootprintDeform::minor_fraction`]).
    /// Abaixo deste produto o pior texel passa de meio nível de u8 e o chamador fica no caminho exato.
    ///
    /// ⚠️ **E a coincidência que não é coincidência:** é a partir daqui que o AA custa caro (68,7 ms a
    /// raio 100 contra ~9 a raio 20) — os dois escalam com o tamanho da pegada, então a LUT rende
    /// exactamente onde o custo está e é recusada exactamente onde erraria.
    pub const MIN_EFFECTIVE_RADIUS: f32 = 40.0;

    /// **A porta única da admissibilidade** — o chamador pergunta aqui em vez de re-derivar a regra.
    ///
    /// Três cláusulas, e a terceira é do CHAMADOR porque é por-texel:
    ///
    /// 1. a família de falloff **SUAVE** — `Constant` se exclui por DOIS motivos ao mesmo tempo (é
    ///    errático, porque um degrau interage com a grade de texels, **e é mais LENTO**, 0,46×, porque a
    ///    curva dele é a constante 1 e não há raiz a economizar); `Custom` porque a `for_dab` toma o dab
    ///    inteiro como banda para ele e a tabela seria indexada por uma curva do documento;
    /// 2. **`raio × minor ≥ `[`Self::MIN_EFFECTIVE_RADIUS`];
    /// 3. ⚠️ **o texel não pode STRADDLEAR a fronteira calota↔banda da cápsula** — ali o `B` correto muda
    ///    no meio da grade 3×3 e nenhuma base única serve (medido: 0,77 nível a raio 40, contra 0,06 nas
    ///    outras regiões). É uma faixa de ~2 linhas de texel por calota, então o caminho exato ali custa
    ///    quase nada — e é por isso que a cláusula é do chamador, que é quem tem a projeção em mãos.
    #[must_use]
    pub fn admissible(spec: &crate::BrushSpec, radius: f32) -> bool {
        let smooth = matches!(
            spec.falloff,
            crate::Falloff::Smooth
                | crate::Falloff::Smoother
                | crate::Falloff::Sphere
                | crate::Falloff::Sharp
                | crate::Falloff::Pow4
                | crate::Falloff::Root
                | crate::Falloff::Linear
                | crate::Falloff::InvSquare
        );
        // Hardness ≥ 1 torna QUALQUER falloff um degrau (`falloff_weight` devolve 1 ou 0), então ela
        // recai no caso `Constant` e sai pela mesma porta.
        let hard = spec.hardness >= 1.0;
        let minor = spec.dab_footprint([1.0, 0.0]).minor_fraction();
        smooth && !hard && radius * minor >= Self::MIN_EFFECTIVE_RADIUS
    }

    /// `F(t)` por interpolação linear. Fora de `[0, 1]` clampa — que é o que a curva faz de qualquer
    /// forma (`falloff_weight` clampa o remap, `weight` devolve 0 em `t >= 1`).
    #[inline]
    #[must_use]
    pub fn at(&self, t: f32) -> f32 {
        let x = (t * self.n).clamp(0.0, self.n);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "x está clampado em [0, N] logo acima"
        )]
        let i = x as usize;
        let f = x - {
            #[expect(clippy::cast_precision_loss, reason = "i <= 16384, exato em f32")]
            let fi = i as f32;
            fi
        };
        // `i + 1` existe: a tabela tem N+1 entradas e `i <= N`. Em `i == N` o `f` é 0, então o vizinho
        // não é lido — mas o índice tem de ser válido, e é por isso que a tabela leva a entrada extra.
        let a = self.table[i];
        let b = self.table[(i + 1).min(self.table.len() - 1)];
        a + (b - a) * f
    }
}

/// The `t` where [`crate::BrushSpec::falloff_weight`] crosses `v` — the [`body_edge_t`] bisection,
/// generalised to any threshold (same hard-tip probe, same convergence: ~6e-8 of `t`).
fn cross_t(spec: &crate::BrushSpec, v: f32) -> f32 {
    const RIM_PROBE: f32 = 0.999;
    if spec.falloff_weight(RIM_PROBE) >= v {
        return 1.0; // hard tip: the body reaches the geometric rim, the crossing IS t = 1
    }
    let mut lo = 0.0f32;
    let mut hi = 1.0f32;
    for _ in 0..24 {
        let mid = 0.5 * (lo + hi);
        if spec.falloff_weight(mid) >= v {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}
