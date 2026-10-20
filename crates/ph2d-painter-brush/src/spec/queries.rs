//! The spec's **derived questions** (child of [`super`], split for the
//! workspace file-LOC cap): every `BrushSpec` method that ANSWERS something
//! about the authored knobs — clamps, effective values, capability predicates,
//! spacing and falloff weights. The knobs themselves (the struct, its consts,
//! the material bundle) stay in the parent; nothing here stores state.

use super::*;

impl BrushSpec {
    /// Effective dab radius after clamping to the allocation cap.
    #[must_use]
    pub fn clamped_radius(&self) -> f32 {
        self.radius_px.clamp(0.5, MAX_BRUSH_RADIUS_PX)
    }

    /// The baked dab flatten + rotate ([`crate::footprint::FootprintDeform`]) — applied to the
    /// footprint of the falloff, the Shape silhouette and the View-mapped Grain so they deform together.
    #[must_use]
    /// ⭐ **E a CURVATURA da arte entra AQUI, numa porta só** — é isso que faz o falloff, a
    /// silhueta do Shape, o Grão e as bandas herdarem a dobra sem uma linha em cada um deles.
    pub fn footprint_deform(&self) -> crate::footprint::FootprintDeform {
        crate::footprint::FootprintDeform::new(self.dab_flatten, self.dab_angle_deg)
            .with_curve(self.dab_curve)
            .with_quina(self.dab_quina)
    }

    /// ⭐ **A pegada NÃO é invariante à rotação** — achatada ou com a ponta reta ([`Self::dab_quina`]).
    /// A pergunta de quem decide se o ângulo do dab se VÊ (o Jitter Rotate, o Follow, as rotas que
    /// servem a máscara de orientação constante).
    #[must_use]
    pub fn pegada_anisotropica(&self) -> bool {
        self.dab_flatten > 0.0 || self.dab_quina > 0.0
    }

    /// O alcance da PEGADA em raios: o maior raio da ponta reta em qualquer ângulo
    /// (`1 / (1 − q + q/√2)`, o canto) — `1` sem quina, ao bit.
    #[must_use]
    pub fn alcance_da_quina(&self) -> f32 {
        if self.dab_quina <= 0.0 {
            return 1.0;
        }
        let q = self.dab_quina.min(1.0);
        1.0 / (1.0 - q * (1.0 - std::f32::consts::FRAC_1_SQRT_2))
    }

    /// A caixa da pegada deste pincel em raios (`≥ 1` por eixo): `[1, 1]` sem quina, ao bit; com ela a
    /// [`crate::footprint::FootprintDeform::extent`] — o quadrado rodado sai do disco.
    #[must_use]
    pub fn caixa_da_pegada(&self) -> [f32; 2] {
        if self.dab_quina <= 0.0 {
            return [1.0, 1.0];
        }
        let e = self.footprint_deform().extent();
        [e[0].max(1.0), e[1].max(1.0)]
    }

    /// Whether **Randomize Color** has any non-zero amount (so enabling it would actually change a
    /// dab). Used to gate the RNG draw in [`crate::jitter::per_dab`].
    #[must_use]
    pub fn has_colour_jitter_amount(&self) -> bool {
        self.color_jitter_hue > 0.0 || self.color_jitter_sat > 0.0 || self.color_jitter_val > 0.0
    }

    /// Whether this brush rotates each dab's texture frame independently (Jitter Rotate), so the
    /// constant-orientation stamp caches must be bypassed (each dab needs its own basis). Only
    /// meaningful with a texture that uses per-dab rotation (i.e. not Stencil); Drag Dot / Anchored
    /// opt out of all per-dab jitter, so they never trigger it.
    #[must_use]
    pub fn has_per_dab_rotation(&self) -> bool {
        // Jitter Rotate spins the WHOLE dab footprint (falloff + Shape + View-Grain) about the centre. It
        // only *shows* when the footprint is not rotation-invariant — i.e. an anisotropic footprint
        // (`dab_flatten > 0`) OR a texture that rides the dab frame. This used to require
        // `texture.is_active()`, so a FLATTENED, untextured dab with Jitter Rotate looked constant to the
        // guard and Smear/Blur/Clone served it the cached (constant-orientation) mask: every dab came out
        // with the SAME ellipse angle and the knob did nothing (sweep 2026-07-12). The paint path never had
        // the bug — with both slots off it has no cache to serve and resolves `d.rotation` per pixel.
        self.jitter_rotate > 0.0
            && self.stroke_method.allows_jitter()
            && (self.pegada_anisotropica()
                || (self.texture.is_active() && self.texture.mapping.uses_dab_rotation()))
    }

    /// Whether the **Grain** slot samples a texture — the twin of [`Self::shape_silhouette_active`]: an
    /// `Image` grain with no pixels loaded samples `1.0` everywhere. Where the Grain MULTIPLIES that is
    /// inert by itself; where an armed Grain REPLACES another texture (the Wet Paint bristle) it must
    /// not count as armed, or choosing *Image* before loading one swaps the bristle for a flat sheet.
    #[must_use]
    pub fn grain_samples(&self, has_grain_image: bool) -> bool {
        self.texture.is_active()
            && (self.texture.kind != crate::texture::TextureKind::Image || has_grain_image)
    }

    /// ⭐ **O alcance da ponta em raios, invariante à rotação** (Bug #52): o raio do disco que contém a
    /// ponta de imagem do Shape em QUALQUER ângulo (a da Shape e a sorteada de cada dab) — `1` sem ponta
    /// de imagem. As regiões do tool (salvar/restaurar, sujar, desfazer, a guarda) multiplicam o raio
    /// por ele: inflar a mais é inócuo, a menos deixa rasto (Bug #24). O achatamento só encolhe a ponta.
    #[must_use]
    pub fn tip_reach(&self) -> f32 {
        let quina = self.alcance_da_quina();
        if !self.shape.is_active() {
            return quina;
        }
        crate::texture::image_tip_radius(&self.shape).map_or(quina, |r| r.max(quina))
    }

    /// O alcance de UM dab em raios (`≥ 1`): a ponta de imagem com a base dele (Angle + a rotação
    /// sorteada, já na pegada de `basis`), ou `1`. Para os núcleos que medem o dab por um raio escalar
    /// (o relevo, a escultura, o motor da tinta molhada); os por-eixo usam [`crate::tip`].
    #[must_use]
    pub fn dab_tip_reach(&self, basis: Option<&crate::texture::TexDabBasis>) -> f32 {
        let e = crate::tip::dab_extent(
            self.caixa_da_pegada(),
            &self.shape,
            basis.filter(|_| self.shape.is_active()),
        );
        e[0].max(e[1])
    }

    /// O DOMÍNIO do carimbo assado em raios (`≥ 1`, [`crate::tip`]): o quadrado que contém a ponta
    /// estática deste pincel. Quem escolhe a resolução do assado multiplica o raio por ele, para a ponta
    /// alargada não perder nitidez. `1` sem ponta de imagem ⇒ o assado de sempre, ao bit.
    #[must_use]
    pub fn stamp_domain(&self, has_shape_image: bool) -> f32 {
        let pegada = self.caixa_da_pegada();
        let pegada = pegada[0].max(pegada[1]);
        if !self.shape_silhouette_active(has_shape_image) {
            return pegada;
        }
        let basis = crate::texture::shape_basis(
            &self.shape,
            &mut 0u64,
            [1.0, 1.0],
            self.footprint_deform(),
            crate::texture::ShapeFrame::Static,
        );
        crate::tip::stamp_domain(&self.shape, Some(&basis)).max(pegada)
    }

    /// ⭐ **O Solid enche-se de CARIMBOS** ([`crate::solid_carimbos`]) em vez da mancha lisa: a ponta é
    /// uma IMAGEM com pixels (a forma do pincel É o desenho — folhas, pétalas). Uma ponta redonda ou
    /// procedural continua com a mancha na cor do Color.
    #[must_use]
    pub fn solid_fills_with_stamps(&self, has_shape_image: bool) -> bool {
        self.shape_silhouette_active(has_shape_image)
            && self.shape.kind == crate::texture::TextureKind::Image
    }

    /// Whether the **Shape** slot supplies the silhouette (else the [`Self::falloff`] does). True when
    /// a shape kind is assigned AND — for the `Image` kind — the pixels are present: an Image shape with
    /// no image falls back to the falloff, so the brush never paints a blank silhouette. `has_shape_image`
    /// is whether the tool currently holds Shape pixels (the heavy buffer lives in `PaintState`).
    #[must_use]
    pub fn shape_silhouette_active(&self, has_shape_image: bool) -> bool {
        self.shape.is_active()
            && (self.shape.kind != crate::texture::TextureKind::Image || has_shape_image)
    }

    /// Grain Depth clamped to `[0, 1]`. The per-texel grain value `g` becomes `1 + (g − 1)·depth`, so
    /// `depth = 1` is the historical full-bite behaviour and `depth = 0` disables the grain.
    #[must_use]
    pub fn grain_depth(&self) -> f32 {
        self.grain_depth.clamp(0.0, 1.0)
    }

    /// Effective **Granulation** `[0, 1]` (the watercolor deposit gate on the Grain, [`crate::texture::grain_coverage`]).
    /// Zero unless the Watercolor section is on, so a non-watercolor brush keeps the plain Grain multiply
    /// (byte-identical). Pair with a canvas-anchored Grain (`Tiled` mapping + `Grain` kind) for paper granulation.
    #[must_use]
    pub fn effective_granulation(&self) -> f32 {
        if self.watercolor {
            self.granulation.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Effective **Pigment mix** `[0, 1]` (how much the dab composites subtractively — Kubelka–Munk,
    /// [`ph2d_pigment::mix_unit`] — vs the plain blend; [`crate::blend::blend_over_pigment`]). Zero
    /// unless the Pigment knob is on, so a brush that never touched it blends exactly as before
    /// (byte-identical). Non-zero ⇒ a dab over paint mixes like real paint (blue + yellow → green).
    ///
    /// ⭐⭐ **A cerca `self.watercolor &&` SAIU por ordem do dono (2026-09-20: *«ligue o digital»*),
    /// e o que ela escondia não era uma lei — era um MEIO.** A pergunta que este predicado responde
    /// é *«como é que a cor deste dab encontra a tinta que já lá está?»*, e **todo meio que componha
    /// um dab pela porta [`crate::blend::blend_over_pigment`] a faz**. Amarrá-la à aguada tornava a
    /// lei inalcançável no Digital, que é o meio de omissão do app.
    ///
    /// ⚠️ **O ALCANCE foi MEDIDO antes de a cerca sair**, e não é «os quatro meios» —
    /// `diag_pigmento_por_meio` (em `ph2d-tool-painter`) pinta azul sobre amarelo a meia força em
    /// cada meio e compara o pixel com o knob a `0` e a `1`. Quem lê: **Digital** e **Impasto**
    /// (a mesma porta de composite) e a **Watercolor** (pela cadeia óptica dela). Quem NÃO lê: o
    /// **Wet Paint**, cujo depósito é do solver de fluido e que tem o K–M próprio dele.
    ///
    /// ⛔ **É por isso que o painel oferece a fileira nesses três e só nesses** — um knob que o motor
    /// lê e o painel esconde é um INALCANÇÁVEL, e um que o painel pinta e o motor não lê é um MORTO;
    /// a porta que decide isso é a `PaintMedia::offers_pigment_mixing` (em `ph2d-tool-painter`, que
    /// é onde os meios vivem — esta crate não os conhece), lida pelo painel.
    #[must_use]
    pub fn effective_pigment_mix(&self) -> f32 {
        if self.pigment {
            self.pigment_mix.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Whether this brush's dabs deposit **height** (the impasto relief). The master switch must be on,
    /// the brush must be set to write depth, and the depth must be non-zero — a zero-depth dab is a
    /// no-op *by definition* (it neither lifts nor carves), so the height pass skips it entirely rather
    /// than writing a flat zero over relief that is already there.
    #[must_use]
    pub fn deposits_height(&self) -> bool {
        self.impasto && self.impasto_draw_to.writes_depth() && self.impasto_depth != 0.0
    }

    /// O **FILME** de pigmento em cargas ([`Self::film_depth`]), clampado — e deliberadamente **sem** o
    /// gate do `impasto`: um filme é do substrato, não do corpo da tinta.
    #[must_use]
    pub fn effective_film_depth(&self) -> f32 {
        self.film_depth.clamp(0.0, 1.0)
    }

    /// Whether this brush's dabs touch the height field **at all** — it lays thickness down (body or
    /// the substrate's film), or it shoves existing body around, or both.
    ///
    /// Not the same question as [`Self::deposits_height`], and the difference is a real brush: at
    /// **Depth 0 with Push up** the brush carries no paint and still moves the paint it finds. That is a
    /// dry brush, and it is a palette knife — the tool that does nothing BUT displace. Gating the height
    /// pass on the deposit alone made that brush a no-op, and it is the most physical use of Push there is.
    ///
    /// ⚠️ **A terceira razão é o FILME, e ela é DELIBERADAMENTE separada do [`Self::deposits_height`]**
    /// — a distinção foi medida, não escolhida. Aquele predicado não significa *"escreve no plano de
    /// altura"*: ele significa **"este pincel deixa um CORPO de tinta"**, e quatro sítios do caminho de
    /// COR o leem para cortar o pigmento na borda do corpo (`height::film_coverage`). Pôr o filme lá
    /// dentro mudava a SILHUETA do pigmento no instante em que o artista subia o Paint — medido, *pior
    /// 61 níveis* de diferença que não tinham nada a ver com relevo. Um filme é tinta fina sobre papel;
    /// ele não converte o pincel num pincel de corpo.
    #[must_use]
    pub fn touches_height(&self) -> bool {
        self.deposits_height()
            || (self.impasto && self.effective_impasto_push() > 0.0)
            || self.effective_film_depth() != 0.0
    }

    /// Whether this brush's dabs deposit **pigment**. Only [`DrawTo::Depth`] — *with the master switch
    /// on* — suppresses it. With impasto off, [`Self::impasto_draw_to`] is not read at all, so a brush
    /// left on "Depth" in a previous session paints normally again the moment impasto is unticked; the
    /// master switch is the single gate, which is what makes the off state byte-identical.
    #[must_use]
    pub fn deposits_color(&self) -> bool {
        !self.impasto || self.impasto_draw_to.writes_color()
    }

    /// Effective impasto **Depth**, clamped to `[-1, 1]`. Zero unless the master switch is on, so every
    /// caller can read this one number without re-checking the gate.
    #[must_use]
    pub fn effective_impasto_depth(&self) -> f32 {
        if self.impasto {
            self.impasto_depth.clamp(-1.0, 1.0)
        } else {
            0.0
        }
    }

    /// Effective impasto **Smoothing** `[0, 1]`. Zero unless the master switch is on.
    #[must_use]
    pub fn effective_impasto_smoothing(&self) -> f32 {
        if self.impasto {
            self.impasto_smoothing.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Whether this brush's Smear should drag relief: a non-zero **Plow**. Independent of the `impasto`
    /// master switch — the knife moves paint that is already down, whoever laid it.
    #[must_use]
    pub fn impasto_plow_active(&self) -> bool {
        self.impasto_plow > 0.0
    }

    /// Effective impasto **Plow** `[0, 1]`.
    #[must_use]
    pub fn effective_impasto_plow(&self) -> f32 {
        self.impasto_plow.clamp(0.0, 1.0)
    }

    /// Effective impasto **Body** `[0, 1]` — how far the deposit is pushed through the body curve
    /// (`1` = plateau + wall; `0` = the silhouette's own cross-section). No master-switch gate: with
    /// impasto off the kernel is never reached, and the value has no other reader.
    #[must_use]
    pub fn effective_impasto_body(&self) -> f32 {
        self.impasto_body.clamp(0.0, 1.0)
    }

    /// [`Self::impasto_push`], clamped — how much of the ground this brush shoves aside.
    #[must_use]
    pub fn effective_impasto_push(&self) -> f32 {
        self.impasto_push.clamp(0.0, 1.0)
    }

    /// Compose the dab silhouette from a Shape sample `shape_val` and the round `falloff` envelope. The
    /// **Image** kind REPLACES the falloff (a crisp finite tip stays uneroded); any **procedural** kind is
    /// MASKED BY it (`falloff × pattern`, so the soft round envelope shapes the texture — Enio 2026-06-25).
    /// `None` never reaches here (the caller uses the bare falloff when the Shape is inactive). The single
    /// source for all three stamp paths (per-pixel, scale-invariant bake, canvas-cached) so they agree.
    #[must_use]
    pub fn compose_shape_silhouette(&self, shape_val: f32, falloff: f32) -> f32 {
        crate::texture::compose_shape_silhouette_kind(self.shape.kind, shape_val, falloff)
    }

    /// Whether the **Shape** slot rotates its silhouette frame per dab — Rake (follows the stroke) OR the
    /// Stroke **Jitter Rotate** (which spins the WHOLE dab stamp, Shape + Grain together) — so the
    /// constant-orientation caches can't apply (each dab needs its own Shape basis). Only meaningful when
    /// the Shape is the active silhouette. Mirrors [`Self::has_per_dab_rotation`].
    #[must_use]
    pub fn shape_has_per_dab_rotation(&self, has_shape_image: bool) -> bool {
        self.shape_silhouette_active(has_shape_image)
            && self.shape.mapping.uses_dab_rotation()
            && (self.shape.rake
                || self.shape.flow
                || (self.jitter_rotate > 0.0 && self.stroke_method.allows_jitter()))
    }

    /// Whether the **Grain** slot rotates its texture frame per dab (Rake follows the stroke) — each dab
    /// needs its own Grain basis. Mirrors [`Self::shape_has_per_dab_rotation`] for the Grain; complements
    /// [`Self::has_per_dab_rotation`] (which is the Grain **Jitter Rotate**).
    #[must_use]
    pub fn grain_has_per_dab_rotation(&self) -> bool {
        self.texture.is_active() && self.texture.rake && self.texture.mapping.uses_dab_rotation()
    }

    /// Whether the dab is eligible for the **scale-invariant cached stamp** ([`crate::stamp`]) given
    /// both slots: the silhouette must be dab-relative-constant (the falloff always is; a Shape image is
    /// when View-static) AND the Grain must be cacheable (None or static View). `has_shape_image` gates
    /// whether an `Image` shape counts as active. Mirrors [`crate::TextureSettings::is_cacheable`] for
    /// the Grain, extended with the Shape. (Per-dab rotation / Accumulate gating stays at the call site.)
    #[must_use]
    pub fn dab_mask_cacheable(&self, has_shape_image: bool) -> bool {
        let shape_static = !self.shape_silhouette_active(has_shape_image)
            || (matches!(
                self.shape.mapping,
                crate::texture::TextureMapping::ViewPlane
            ) && !self.shape.rake
                && !self.shape.flow
                && !self.shape.kind.samples_in_pixels());
        // Film AA (BUGS #16) is measured in CANVAS texels, and the cached mask is radius-independent —
        // a baked AA rim would be wrong at every other radius, so an AA'd film routes per-pixel.
        shape_static
            && self.texture.is_cacheable()
            && !self.film_aa_wanted(self.shape_silhouette_active(has_shape_image))
    }

    /// Whether the film's screen-space AA ([`crate::height_film::FilmAa`], BUGS #16) applies to this
    /// brush's dabs — the ONE door every consumer asks (the dab/height kernels build the plan from
    /// it, the stamp route refuses the radius-independent cached mask, and the Accumulate-OFF cap
    /// arms so overlapping dabs can't build a rim texel past its area fraction). A Shape silhouette
    /// is a STAMP and keeps its hard edge by design (the `body_edge_t` precedent).
    #[must_use]
    pub fn film_aa_wanted(&self, shape_active: bool) -> bool {
        self.impasto_smooth_edges && self.deposits_height() && !shape_active
    }

    /// Distance between dab centres in pixels, derived from spacing × diameter.
    /// At least one pixel so a stroke always advances.
    #[must_use]
    pub fn dab_spacing_px(&self) -> f32 {
        self.passo_de(self.spacing)
    }

    /// [`Self::dab_spacing_px`] a outra fracção `spacing` do diâmetro.
    #[must_use]
    pub fn passo_de(&self, spacing: f32) -> f32 {
        (spacing.max(0.01) * 2.0 * self.clamped_radius()).max(1.0)
    }

    /// O passo do traço a `spacing` pela direcção `dir` (unitária): a extensão da pegada AO LONGO do caminho
    /// (`stroke/walk.rs`, o pincel achatado anda pelo lado fino), nunca abaixo de 1 px.
    #[must_use]
    pub fn passo_ao_longo_px(&self, spacing: f32, dir: [f32; 2]) -> f32 {
        let v = self.footprint_deform().apply(dir);
        let ao_longo = v[0].hypot(v[1]);
        if ao_longo.is_finite() && ao_longo > 1e-4 {
            (self.passo_de(spacing) / ao_longo).max(1.0)
        } else {
            self.passo_de(spacing)
        }
    }

    /// **Continuous Spacing** is armed: the option is on, the method spaces dabs along a path, and the
    /// step is wider than the capsule's sub-step — every dab then lays a CAPSULE ([`crate::capsula`]),
    /// with ANY tip. The method question is the panel's own ([`StrokeMethod::uses_spacing`], the rows the
    /// box lives in): Dots and Airbrush have no Spacing — the box is hidden there and must not act (prompt
    /// 40 #5) —, the single marks lay no path, and a Spacing already as fine as
    /// [`crate::capsula::SUB_SPACING`] has nothing to fill.
    #[must_use]
    pub fn lays_capsules(&self) -> bool {
        self.continuous_spacing
            && self.stroke_method.uses_spacing()
            && self.dab_spacing_px() > crate::capsula::sub_passo_px(self.clamped_radius())
    }

    /// Whether the dab at dash-slot `slot` is painted, given the dash pattern.
    ///
    /// Behavioural reference — Blender's per-step dash gate: `dash = (slot % dash_samples) /
    /// dash_samples`, skip when `dash > dash_ratio`. `dash_samples == 0` ⇒ no dash (always on).
    /// With the default `dash_ratio = 1.0` every slot is on (solid).
    #[must_use]
    pub fn dash_on(&self, slot: u32) -> bool {
        if self.dash_samples == 0 {
            return true;
        }
        let dash = (slot % self.dash_samples) as f32 / self.dash_samples as f32;
        dash <= self.dash_ratio.clamp(0.0, 1.0)
    }

    /// **O Space Attenuation chega ao carimbo?** — só com o Accumulate ligado e o espaçamento abaixo
    /// de um diâmetro (a condição do [`Self::space_overlap_factor`], que a pergunta a ESTA porta). O
    /// painel esmaece a linha quando ela responde `false` (doc 46 §2-3).
    #[must_use]
    pub fn space_attenuation_reaches(&self) -> bool {
        self.accumulate && self.spacing * 100.0 < 100.0
    }

    /// **A unidade do Jitter muda alguma coisa?** — só se o método deixa o jitter agir e algum dos
    /// dois raios (`jitter × diâmetro` · `2 × jitter_absolute_px`) é positivo: com os dois a zero, as
    /// duas unidades dão o MESMO deslocamento nulo (`stroke::dab_build::apply_jitter`).
    #[must_use]
    pub fn jitter_unit_matters(&self) -> bool {
        self.stroke_method.allows_jitter() && (self.jitter > 0.0 || self.jitter_absolute_px > 0.0)
    }

    /// O raio do disco do Jitter para um dab de raio `radius` (`jitter × diâmetro` · `2 × jitter_absolute_px`): a
    /// ÚNICA conta — o `stroke::dab_build::apply_jitter` e os carimbos sólidos leem esta.
    #[must_use]
    pub fn alcance_do_jitter(&self, radius: f32) -> f32 {
        match self.jitter_unit {
            JitterUnit::Brush => self.jitter.clamp(0.0, 1.0) * (2.0 * radius),
            JitterUnit::View => 2.0 * self.jitter_absolute_px.max(0.0),
        }
    }

    /// **O comprimento do traço-ponto muda alguma coisa?** — só com o Dash Ratio abaixo de `1`: com
    /// ele a `1`, o [`Self::dash_on`] liga todo carimbo, qualquer que seja o comprimento.
    #[must_use]
    pub fn dash_length_matters(&self) -> bool {
        self.dash_ratio < 1.0
    }

    /// The space-attenuation factor THIS stroke method puts in each dab's coverage: the spaced fills
    /// carry [`Self::space_overlap_factor`], the per-event methods `1` (`Stroke::method_overlap`).
    #[must_use]
    pub fn method_overlap_factor(&self) -> f32 {
        if self.stroke_method.is_spaced_fill() {
            self.space_overlap_factor()
        } else {
            1.0
        }
    }

    /// "Adjust Strength for Spacing" multiplier applied to each dab's coverage, in `(0, 1]`.
    ///
    /// Behavioural reference (clean-room): Blender's overlap integral over the falloff curve.
    /// Models how many neighbouring dab falloff kernels stack at a
    /// given phase and returns `1 / max_phase(Σ kernels)` so a densely-spaced stroke is normalised to
    /// unit opacity instead of piling up. Returns `1.0` (no attenuation) when the flag is off or
    /// spacing ≥ 100% (Blender's exact gate). Uses this brush's own falloff, like Blender.
    ///
    /// ## ⚠️ Ele só vale com **Accumulate LIGADO**, e isso é o mecanismo, não uma preferência
    ///
    /// *"normaliza em vez de EMPILHAR"* nomeia a lei que ele compensa: **empilhar é o que o modo
    /// Accumulate faz** (cada dab compõe por conta, `1 − A = Π(1 − a_k)`, então mais dabs = mais
    /// opacidade). Com Accumulate DESLIGADO o traço tem um **TETO por texel**
    /// ([`crate::stroke_cover::cover_add`]: `m += w·(cap − m)`) e nada empilha — mais dabs só
    /// aproximam o mesmo teto.
    ///
    /// E o fator entra em `coverage = strength × pressão × overlap` ([`crate::Stroke::dab_at`]),
    /// que **É o teto**. Sob a lei capada ele portanto não normaliza coisa alguma: só **abaixa o
    /// teto** por um número que depende do espaçamento — o inverso exato do que o nome promete.
    ///
    /// **Medido** (doc 35 §3.3; mesmo caminho, `strength 0.5`, só muda quantos dabs o motor emite;
    /// a razão é entre o espaçamento 5 % e o 40 %):
    ///
    /// ```text
    ///   space_atten off  accumulate off   1,02x   <- ja era independente do espacamento
    ///   space_atten off  accumulate ON    2,95x
    ///   space_atten ON   accumulate off   8,17x   <- o knob PIORAVA o que promete consertar
    ///   space_atten ON   accumulate ON    1,25x   <- e aqui ele faz o trabalho dele
    /// ```
    ///
    /// ⚠️ **Byte-idêntico no default de hoje** (`space_attenuation: false` ⇒ este ramo já devolvia
    /// `1.0`); a cláusula só muda a combinação que estava errada.
    #[must_use]
    pub fn space_overlap_factor(&self) -> f32 {
        // Blender stores spacing as an integer percent; this engine stores a 0..1 fraction.
        let spacing_pct = (self.spacing * 100.0).max(0.0);
        if !(self.space_attenuation && self.space_attenuation_reaches()) {
            return 1.0;
        }
        // Sample the overlap sum at M phases across one period; the factor cancels the peak.
        const M: usize = 10;
        let g = 1.0 / M as f32;
        let mut max = 0.0_f32;
        for i in 0..M {
            let o = self.overlapped_curve(i as f32 * g, spacing_pct).abs();
            if o > max {
                max = o;
            }
        }
        if max == 0.0 { 1.0 } else { 1.0 / max }
    }

    /// Sum of overlapping dab falloff kernels at phase `x` for a given `spacing_pct`
    /// (`paint_stroke_overlapped_curve`): `n = floor(100 / spacing_pct)` kernels spaced
    /// `h = spacing_pct / 50` apart, each evaluated through this brush's falloff.
    fn overlapped_curve(&self, x: f32, spacing_pct: f32) -> f32 {
        let clamped = spacing_pct.max(0.1);
        let n = (100.0 / clamped) as i32;
        let h = clamped / 50.0;
        let x0 = x - 1.0;
        let mut sum = 0.0;
        for i in 0..n {
            let xx = (x0 + i as f32 * h).abs();
            if xx < 1.0 {
                sum += self.falloff_weight(xx);
            }
        }
        sum
    }

    /// Falloff weight remapped by [`Self::hardness`]. `t = distance / radius`.
    ///
    /// Hardness pushes the falloff outward: for `t < hardness` the weight is full; the curve then
    /// runs over `[hardness, 1]`. `hardness >= 1` yields a hard disk. [`Falloff::Custom`] reads the
    /// editable [`Self::custom_falloff`] profile; every other preset uses its formula.
    #[must_use]
    pub fn falloff_weight(&self, t: f32) -> f32 {
        let h = self.hardness.clamp(0.0, 1.0);
        if h >= 1.0 {
            return if t < 1.0 { 1.0 } else { 0.0 };
        }
        let remapped = ((t - h) / (1.0 - h)).clamp(0.0, 1.0);
        match self.falloff {
            Falloff::Custom => self.custom_falloff.weight(remapped),
            preset => preset.weight(remapped),
        }
    }
}
