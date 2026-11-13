//! Watercolor **ground** (real backdrop) + water **soak** (dwell) — the per-stroke state the optical
//! composite reads its Beer–Lambert base / rewet reference / lift target from, and the "the longer
//! the water sits, the more it dissolves" field. Split from `watercolor_render.rs` for the LOC cap.
//!
//! The ground REPLACES the old virtual-cream constant (`PAPER`): where the active layer is
//! transparent, the optics now see the **composite of the layers below it** over the document
//! [`paper colour`](super::PaintState::paper_color) — painting over a white layer no longer pulls
//! toward beige, and the rewet's "what is paint" question is answered against the true local
//! ground (Enio 2026-07-06, doc `11_aquarela_avaliacao_padrao_ouro.md` §3).

use super::*;
use crate::compositor::composite_below;
use rayon::prelude::*;

/// Molhar a camada (EDGE-1) e as leituras da humidade para o véu da shell — cortado pelo tecto de 700 linhas.
mod molhar;
pub(crate) mod veu;

/// How fast the parked brush pours dwell into [`PaintState::wet_soak`]: a held nib saturates its
/// disc in ~2 s (`255 / 2 s`). Moving strokes soak too (each tick pours at the current dab), but the
/// dt spreads across positions, so a sweep stays light — only LINGERING builds a deep soak.
const SOAK_RATE_PER_S: f32 = 127.5;

/// A dureza que o [`PainterTool::pour_canvas_wet`] despeja, **tabelada** — `cobertura u8 -> umidade u8`.
///
/// O despejo roda sobre a união CUMULATIVA do traço em todo quadro que compõe, e a expressão que ele
/// avaliava por texel (`smoothstep(SS0, SS1, c/255) * 255`) é **função pura de um byte**: 256 respostas
/// possíveis. Preguiçosa, porque uma sessão que nunca pinta aquarela não paga a construção.
///
/// ⚠️ **A tabela É a expressão, não uma aproximação dela** — cada entrada é computada pela MESMA
/// `smoothstep` com as MESMAS constantes, e o gate `the_pour_table_is_the_expression_it_replaces`
/// compara as 256 contra o cálculo direto. Um erro aqui não seria arredondamento: seria a umidade
/// discordando da silhueta que o composite desenha.
pub(super) fn pour_hardening_lut() -> &'static [u8; 256] {
    static LUT: std::sync::OnceLock<[u8; 256]> = std::sync::OnceLock::new();
    LUT.get_or_init(|| {
        let mut t = [0u8; 256];
        for (i, v) in t.iter_mut().enumerate() {
            *v = pour_hardening(i as u8);
        }
        t
    })
}

/// A expressão que a tabela substitui, isolada para o gate poder compará-las — e **usada** por ela,
/// então não há duas respostas para a mesma pergunta (a lição do `warp_axis`: um oráculo congelado que
/// o produto não chama é uma segunda resposta esperando um chamador).
pub(super) fn pour_hardening(coverage: u8) -> u8 {
    let cov = f32::from(coverage) / 255.0;
    let cw = super::watercolor_field::smoothstep(
        super::watercolor_render::SS0,
        super::watercolor_render::SS1,
        cov,
    );
    (cw * 255.0) as u8
}

/// Piso do pool em TEXELS para os dois passes por-quadro da umidade (o DESPEJO aqui, a SECAGEM no
/// irmão `watercolor_dry`): abaixo disto eles rodam serial, porque o fork do rayon custa mais que o
/// trabalho. Mora com o MAPA, que os dois compartilham. Molde: o limiar em bytes do [`crate::plane_copy`].
pub(super) const WET_PAR_MIN: usize = 64 << 10;

/// Tudo o que [`composite_below`] lê para compor o fundo de baixo (ver [`PainterTool::fundo_de_baixo`]).
#[derive(Clone)]
pub(crate) struct ChaveDoFundo {
    pilha: crate::layers::LayerStack,
    imagens: Vec<(crate::layers::LayerId, std::sync::Weak<Vec<u8>>, u32, u32)>,
    papel: [u8; 3],
    tamanho: (u32, u32),
}

impl ChaveDoFundo {
    pub(super) fn casa(&self, o: &Self) -> bool {
        self.papel == o.papel
            && self.tamanho == o.tamanho
            && self.imagens.len() == o.imagens.len()
            && self.imagens.iter().zip(&o.imagens).all(|(a, b)| {
                a.0 == b.0 && a.2 == b.2 && a.3 == b.3 && std::sync::Weak::ptr_eq(&a.1, &b.1)
            })
            && self.pilha == o.pilha
    }
}

thread_local! {
    /// A×B do fundo de baixo em cache (`false` = compor a cada pen-down, o de antes). Só a sonda e o gate o desligam.
    #[doc(hidden)]
    pub static FUNDO_EM_CACHE: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

impl PainterTool {
    /// Freeze the watercolor ground for a beginning stroke: the pre-stroke `canvas_rgba` as the
    /// optical base (shared `Arc`, O(1)) + the real **backdrop** under the active layer. The soak
    /// (dwell) resets only on a FRESH session — a CONTINUING wet session keeps the paper's dwell,
    /// so the union re-renders reproduce the bakes byte-exact (the soak boosts rim/settle per
    /// pixel; zeroing it per stroke re-rendered the neighbour wash weaker, Enio 2026-07-09).
    /// No-op (clears both) when the watercolor render-path is off.
    pub(super) fn freeze_watercolor_ground(&mut self, wet_session: bool) {
        // Envelope de diagnóstico — o corpo é o `_inner` (ver `apply_watercolor`).
        let t0 = std::time::Instant::now();
        self.freeze_watercolor_ground_inner(wet_session);
        crate::wash_diag::note_pendown(t0.elapsed().as_secs_f32() * 1e3);
    }

    fn freeze_watercolor_ground_inner(&mut self, wet_session: bool) {
        if !self.watercolor_render_active() {
            self.paint.watercolor_base = None;
            self.paint.wet_backdrop = None;
            return;
        }
        self.paint.watercolor_base = Some(Arc::clone(&self.canvas_rgba));
        self.paint.wet_backdrop = Some(self.fundo_de_baixo());
        // A new wet render supersedes any live-editable wash — the previous one is now permanent (it was
        // baked into `canvas_rgba` at its pen-up, which THIS base freeze captured). Drop its snapshot.
        self.clear_wet_editable();
        if !wet_session {
            // B16.9: pelos fios como o `NaN` abaixo (cada byte recebe o mesmo valor ⇒ o mesmo byte).
            self.paint
                .wet_soak
                .par_chunks_mut(WET_PAR_MIN)
                .for_each(|c| c.fill(0));
            self.paint.wet_soak_active = false;
        }
        self.paint.wet_soak_pos = None;
        // Manual Shape tip normaliser (see `PaintState::wet_shape_norm`): 1 / max luminance of the
        // Shape image, computed once per stroke — the watercolor coverage must SATURATE in the tip's
        // core (wetness geometry), so grey tips scale up to full while keeping their relative texture.
        self.paint.wet_shape_norm = 1.0;
        if !self.paint.brush.watercolor_shape_auto
            && let Some(img) = self.paint.shape_image.as_ref()
        {
            let (lum, _, _) = img.parts();
            let max = lum.iter().copied().max().unwrap_or(0);
            if max > 0 {
                self.paint.wet_shape_norm = 255.0 / f32::from(max);
            }
        }
        // Substrate cache: invalidate for the new stroke (the paper is constant WITHIN a stroke, so a
        // full reset at pen-down is the only invalidation needed — no in-stroke staleness). Filled
        // lazily by the composite ([`Self::apply_watercolor`]); `NaN` marks an untouched pixel.
        self.repoe_o_substrato();
    }

    /// Repõe o memo do substrato a `NaN` (B16.9): só na caixa que o composite preencheu desde o
    /// último reset, porque fora dela já é `NaN` ⇒ o mesmo estado que a tela inteira. Tamanho ou
    /// largura novos ⇒ a tela inteira. Pelos fios, por linha: cada elemento recebe o mesmo valor.
    pub(super) fn repoe_o_substrato(&mut self) {
        let (fw, fh) = (self.source_size.0 as usize, self.source_size.1 as usize);
        let n = fw * fh;
        let sub = &mut self.paint.wet_substrate;
        let caixa = self.paint.wet_substrate_caixa.take();
        let nova = sub.len() != n;
        if nova {
            *sub = vec![0.0; n];
        }
        match caixa {
            _ if nova => sub
                .par_chunks_mut(WET_PAR_MIN)
                .for_each(|c| c.fill(f32::NAN)),
            None => {}
            Some((c, w)) if w as usize == fw && c.x + c.w <= w && ((c.y + c.h) as usize) <= fh => {
                let (x0, x1) = (c.x as usize, (c.x + c.w) as usize);
                sub[c.y as usize * fw..(c.y + c.h) as usize * fw]
                    .par_chunks_mut(fw)
                    .for_each(|l| l[x0..x1].fill(f32::NAN));
            }
            Some(_) => sub
                .par_chunks_mut(WET_PAR_MIN) // rayon, like the soak pour below (ADR-0173)
                .for_each(|c| c.fill(f32::NAN)),
        }
    }

    /// Composite the layers strictly BELOW the anchor over the document paper colour → the opaque
    /// RGBA8 ground. The anchor is the active layer; painting a MASK anchors at the mask's PARENT
    /// (the mask shows through its parent, so the ground under the parent is what the eye sees).
    /// Runs once per stroke (pen-down); `composite_below` handles an unknown anchor (plain paper).
    fn build_wet_backdrop(&self) -> Vec<u8> {
        self.build_wet_backdrop_sobre(self.cor_do_chao())
    }

    /// **O fundo de baixo, da cache quando nada do que o compõe mudou** (n.º 6): a pilha, a imagem de cada camada (pela
    /// identidade da alocação: com o `Weak` vivo nenhuma escrita segura muda o plano no lugar — o `make_mut` copia ou
    /// MOVE), o papel e o tamanho. Medido a 4K: 10–14 ms por pen-down de aguada (e o chão preto do vidro, que se
    /// guarda pelo mesmo `Arc`, mais 8 ms). Com uma máscara activa a fonte troca a âncora: compõe sempre.
    pub(super) fn fundo_de_baixo(&mut self) -> Arc<Vec<u8>> {
        let mascara = self
            .layers
            .active()
            .is_some_and(|id| self.layers.is_mask(id));
        if mascara || !FUNDO_EM_CACHE.get() {
            return Arc::new(self.build_wet_backdrop());
        }
        let chave = self.chave_do_fundo();
        self.colhe_o_aquecimento(&chave);
        if let Some((c, fundo)) = &self.paint.fundo_em_cache
            && c.casa(&chave)
        {
            return Arc::clone(fundo);
        }
        let fundo = Arc::new(self.build_wet_backdrop());
        self.paint.fundo_em_cache = Some((chave, Arc::clone(&fundo)));
        fundo
    }

    /// Tudo o que o fundo de baixo lê agora, pela identidade (a chave da cache e do aquecimento em fundo).
    pub(super) fn chave_do_fundo(&self) -> ChaveDoFundo {
        ChaveDoFundo {
            pilha: self.layers.clone(),
            imagens: self
                .images
                .iter()
                .map(|(&id, img)| (id, Arc::downgrade(&img.rgba8), img.width, img.height))
                .collect(),
            papel: self.cor_do_chao(),
            tamanho: self.source_size,
        }
    }

    /// O mesmo chão sobre um papel `paper` qualquer — o vidro pede-o sobre o PRETO
    /// ([`super::watercolor_vidro`]).
    pub(super) fn build_wet_backdrop_sobre(&self, paper: [u8; 3]) -> Vec<u8> {
        let (w, h) = self.source_size;
        let anchor = self.layers.active().and_then(|id| {
            if self.layers.is_mask(id) {
                // The mask isn't in the z-order — resolve the raster that owns it.
                self.layers
                    .all_ids()
                    .find(|&pid| self.layers.get(pid).is_some_and(|l| l.mask == Some(id)))
            } else {
                Some(id)
            }
        });
        let Some(anchor) = anchor else {
            let mut ground = vec![0u8; (w as usize) * (h as usize) * 4];
            for px in ground.as_chunks_mut::<4>().0 {
                px.copy_from_slice(&[paper[0], paper[1], paper[2], 255]);
            }
            return ground;
        };
        let src = ToolPixelSource {
            active_id: anchor,
            active_rgba: &self.canvas_rgba,
            images: &self.images,
        };
        composite_below(&self.layers, &src, w, h, anchor, paper)
    }

    /// The document paper colour as straight sRGB8 bytes.
    pub(crate) fn paper_color_rgb8(&self) -> [u8; 3] {
        let c = self.paint.paper_color;
        [
            (c[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (c[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            (c[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        ]
    }

    /// Set one paper-colour channel (`0..1` straight sRGB); the shared-picker read-back drives all
    /// three per frame via `SelectOption(PAINTER_WATERCOLOR_PAPER_COLOR_THUMB, "r,g,b")`.
    pub(crate) fn set_paper_color_rgb8(&mut self, r: u8, g: u8, b: u8) {
        self.paint.paper_color = [
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
        ];
    }

    /// Pour `dt_s` of water dwell into the soak disc at the last dab position (the tick heartbeat,
    /// moving or parked). Returns the poured disc's region when the soak actually grew AND the
    /// rewet reads it (`wet_rewet > 0`), so the caller can fold it into the frame dirty rect and
    /// recomposite — a parked wet brush visibly deepens its bleed while held.
    pub(super) fn grow_wet_soak(&mut self, dt_s: f32) -> Option<Region> {
        let wet = self.paint.brush.wet_rewet;
        let (center, radius) = self.paint.wet_soak_pos?;
        if wet <= 0.0 || dt_s <= 0.0 || radius <= 0.0 {
            return None;
        }
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        if fw == 0 || fh == 0 {
            return None;
        }
        if self.paint.wet_soak.len() != fw * fh {
            self.paint.wet_soak = vec![0u8; fw * fh];
        }
        // Byte units per tick; at least 1 step so slow frames still soak.
        let add = (SOAK_RATE_PER_S * dt_s).clamp(1.0, 255.0) as u16;
        let (cx, cy) = (center[0], center[1]);
        // A sitting puddle SPREADS: the pour disc grows with the centre's own saturation (up to
        // 2× the nib) — holding the brush pushes the dwell past the disc edge, so the widened
        // dissolve visibly creeps outward instead of stopping at the footprint.
        let ci = (cy.clamp(0.0, (fh - 1) as f32) as usize) * fw
            + (cx.clamp(0.0, (fw - 1) as f32) as usize);
        let center_soak = if self.paint.wet_soak.len() == fw * fh {
            f32::from(self.paint.wet_soak[ci]) / 255.0
        } else {
            0.0
        };
        let r = radius * (1.0 + center_soak);
        let inv_r = 1.0 / r;
        let x0 = (cx - r).floor().max(0.0) as usize;
        let y0 = (cy - r).floor().max(0.0) as usize;
        let x1 = ((cx + r).ceil() as i64).clamp(0, fw as i64) as usize;
        let y1 = ((cy + r).ceil() as i64).clamp(0, fh as i64) as usize;
        if x0 >= x1 || y0 >= y1 {
            return None;
        }
        // Selection + protection gates: water doesn't pool on gated-out texels either (the soak
        // deepens lift/dissolve — letting it accumulate outside the selection would deepen the
        // dissolve just inside the boundary from paint the user masked off). See `splat_keep`.
        let (sel, prot, alock) = self.wet_splat_gates();
        let gated = sel.is_some() || prot.is_some() || alock.is_some();
        // ⭐ Row-parallel (ADR-0109 class, ADR-0173): every texel reads only ITSELF (its soak, its
        // gates) and writes only itself, no RNG, and the one reduction is the `grew` OR — the kind
        // the fence exempts. The disc reaches `2 × 250` px at the owner's brush (~785 k texels, a
        // `sqrt` each) on EVERY tick with Rewet on, and ran serial on the main thread: ~9 % of the
        // frame's wall in the product profile. ⚠️ `reduce`, never `any`: `any` short-circuits and
        // would SKIP the writes of the rows after the first that grew.
        use rayon::prelude::*;
        let (sel, prot, alock) = (
            sel.as_deref().map(Vec::as_slice),
            prot.as_deref().map(Vec::as_slice),
            alock.as_deref().map(Vec::as_slice),
        );
        let grew = self.paint.wet_soak[y0 * fw..y1 * fw]
            .par_chunks_mut(fw)
            .with_min_len(8)
            .enumerate()
            .map(|(j, row)| {
                let y = y0 + j;
                let dy = (y as f32 + 0.5) - cy;
                let base = y * fw;
                let mut grew = false;
                for (x, cell) in row.iter_mut().enumerate().take(x1).skip(x0) {
                    let dx = (x as f32 + 0.5) - cx;
                    let dn = (dx * dx + dy * dy).sqrt() * inv_r;
                    if dn >= 1.0 {
                        continue;
                    }
                    let keep = if gated {
                        super::watercolor_accum::splat_keep(sel, prot, alock, base + x)
                    } else {
                        1.0
                    };
                    if keep <= 0.0 {
                        continue;
                    }
                    // Full pour inside the core, fading to the rim (the water pools under the nib).
                    let w = (1.0 - dn).min(0.6) / 0.6;
                    let cur = *cell;
                    let next = (u16::from(cur) + (f32::from(add) * w * keep) as u16).min(255) as u8;
                    if next != cur {
                        *cell = next;
                        grew = true;
                    }
                }
                grew
            })
            .reduce(|| false, |a, b| a | b);
        if grew {
            self.paint.wet_soak_active = true;
        }
        grew.then(|| Region {
            x: x0 as u32,
            y: y0 as u32,
            w: (x1 - x0) as u32,
            h: (y1 - y0) as u32,
        })
    }

    /// B16.11 (RC-66): a janela que o pour caminha, dentro do retângulo do traço. Depois do pour
    /// anterior do MESMO traço, `canvas_wet ≥ dureza(cobertura)` em todo texel dele; sem passo do
    /// secador (ele derruba a marca), cobertura e dono só mudaram nos quadros consumidos desde então
    /// (`pendente`) e no sujo ainda por consumir — fora deles o max-blend não muda um byte.
    fn janela_do_pour(&mut self, rect: Rect4, cur_o: Dono, has_owner: bool) -> Rect4 {
        let marca = self.paint.wet_pour_marca.take();
        #[cfg(test)]
        if POUR_CUMULATIVO.get() {
            return rect;
        }
        let Some(m) = marca.filter(|m| has_owner && m.dono == cur_o && self.paint.tracos_fora == 0)
        else {
            return rect;
        };
        let (x0, y0, x1, y1) = rect;
        let novo = match (m.pendente, self.paint.wet_frame_dirty) {
            (Some(p), Some(f)) => union_region(p, f),
            (p, f) => match p.or(f) {
                Some(r) => r,
                None => return (x0, y0, x0, y0),
            },
        };
        let nx0 = (novo.x as usize).clamp(x0, x1);
        let ny0 = (novo.y as usize).clamp(y0, y1);
        let nx1 = ((novo.x + novo.w) as usize).clamp(nx0, x1);
        let ny1 = ((novo.y + novo.h) as usize).clamp(ny0, y1);
        (nx0, ny0, nx1, ny1)
    }
}

type Rect4 = (usize, usize, usize, usize);
use super::watercolor_field::Dono;

/// B16.11: a marca do último pour do traço ([`PaintState::wet_pour_marca`]) — o dono que ele despejou
/// e a união dos quadros que os composites consumiram depois dele (`watercolor_render/window.rs`).
#[derive(Clone, Copy, Debug)]
pub(super) struct MarcaDoPour {
    dono: Dono,
    pub(super) pendente: Option<Region>,
}

#[cfg(test)]
thread_local! {
    /// O oráculo do B16.11: o pour pelo retângulo CUMULATIVO do traço (o caminho antigo).
    pub(super) static POUR_CUMULATIVO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// Texels que os pours caminharam (o gate conta em vez de cronometrar).
    pub(super) static POUR_PX: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// EDGE-1 — DEFAULT paper drying rate in wetness-bytes per second (~10 s window, Enio 2026-07-10:
/// 60 s ficou "extremamente alto"; histórico: 30 ≈ 8,5 s DiVerdi/Adobe TVCG 2013 → 4.25 ≈ 60 s). The
/// live rate is now `PaintState::dry_rate_per_s` (the Drying-Time slider, doc 13 #11); this is only
/// its default. `255 / seconds`: 10 s → 25.5.
pub(super) const CANVAS_WET_DRY_DEFAULT: f32 = 25.5;

/// #12a (doc 14) — DEFAULT wetness-preview strength: the max veil alpha the shell paints over the wet
/// paper (a discreet damp darkening). `0` = no preview. The live value is `PaintState::wet_preview_intensity`
/// (the Wetness card's slider); this is only its default.
pub(super) const WET_PREVIEW_DEFAULT: f32 = 0.3;

/// #3 (doc 14) — the **Wet the layer** forced rewet level: how strongly strokes lift/blend the EXISTING
/// paint after the Wet button re-wets the canvas (Rebelle-style), independent of the brush's own Rewet.
/// Strong so the reactivation is clearly visible; the artist still tempers it with the brush's Rewet up.
pub(super) const WET_CANVAS_REWET: f32 = 0.8;

impl PainterTool {
    /// EDGE-1: whether the stroke beginning NOW continues the live **wet session** (one wash):
    /// watercolor mode, some paper still wet, the session base is sized, the union buffers exist,
    /// and — the load-bearing guard — `canvas_rgba` is EXACTLY the Arc our last bake produced
    /// (`Arc::ptr_eq`). Every foreign mutation path (undo restore, layer switch/bind, fill,
    /// resize, document switch) assigns a fresh Arc, so no per-site invalidation is needed.
    pub(crate) fn wet_session_continues(&self) -> bool {
        let (fw, fh) = self.source_size;
        let n = (fw as usize) * (fh as usize);
        self.watercolor_render_active()
            && n > 0
            && self.paint.canvas_wet_rect.is_some()
            && self.paint.stroke_coverage.len() == n
            && self.paint.stroke_color.len() == n * 4
            && self
                .paint
                .wet_session_base
                .as_ref()
                .is_some_and(|b| b.len() == n * 4)
            && self.paint.wet_session_canvas.as_ref().is_some_and(|g| {
                Arc::ptr_eq(&g.tela, &self.canvas_rgba) && g.camada == self.layers.active()
            })
    }

    /// Arma a guarda da sessão sobre a tela de agora e a camada activa.
    pub(crate) fn arma_a_guarda_da_sessao(&mut self) {
        self.paint.wet_session_canvas = Some(super::state::GuardaDaSessao {
            tela: Arc::clone(&self.canvas_rgba),
            camada: self.layers.active(),
        });
    }

    /// EDGE-1: pour the stroke's coverage-so-far into the persistent canvas MOISTURE map (max-blend
    /// over THIS STROKE's OWN footprint).
    ///
    /// ⚠️ **This doc used to say "Called at the bake, AFTER the commit composite", and that has not
    /// been true for a long time** — the moisture is laid LIVE (`#2`), so the damp shows *during* the
    /// stroke. Three live call sites plus the bake: `paint_begin`, `paint_tick`, the legacy per-event
    /// route in `paint_extend`, and `paint_end`. A function that names itself a once-per-stroke cost is
    /// a function an auditor walks past, and this one was the LARGER half of the per-event footprint
    /// bill this module carried until 2026-08-02: it is not gated on a dab landing, and it walks
    /// `wet_stroke_dirty`, the **cumulative** union since pen-down. Cheap only because the max-blend
    /// makes re-pouring idempotent — never because it is rare.
    ///
    /// The stroke's own rim still renders against dry paper (the bake is what the rim reads); only
    /// strokes that follow within the drying window merge into it. Restricted to the pixels THIS stroke
    /// OWNS (`wet_styles.owner == cur_o`,
    /// recency): `stroke_coverage` is the whole session UNION, so pouring it over any RECT re-wet the
    /// earlier washes' pixels that fell inside the rect — a rectangular re-wet patch over the neighbour
    /// (Enio 2026-07-11 "retângulo gigante na união"; the per-stroke RECT of #4 only shrank it). The owner
    /// map is exactly this stroke's footprint (its overlap with a prior wash included, by recency).
    pub(super) fn pour_canvas_wet(&mut self) {
        // Envelope de diagnóstico — o corpo é o `_inner` (ver `apply_watercolor`).
        let t0 = std::time::Instant::now();
        self.pour_canvas_wet_inner();
        crate::wash_diag::note_pour(t0.elapsed().as_secs_f32() * 1e3);
    }

    fn pour_canvas_wet_inner(&mut self) {
        let (fw, fh) = self.source_size;
        let (fw, fh) = (fw as usize, fh as usize);
        let n = fw * fh;
        if n == 0 || self.paint.stroke_coverage.len() != n {
            self.paint.wet_pour_marca = None;
            return;
        }
        let Some(r) = self.paint.wet_stroke_dirty else {
            return;
        };
        let (x0, y0) = ((r.x as usize).min(fw), (r.y as usize).min(fh));
        let (x1, y1) = (
            ((r.x + r.w) as usize).min(fw),
            ((r.y + r.h) as usize).min(fh),
        );
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        if self.paint.canvas_wet.len() != n {
            self.paint.canvas_wet = vec![0u8; n];
            self.paint.canvas_wet_rect = None;
            self.paint.wet_pour_marca = None;
        }
        // Only THIS stroke's OWN footprint re-wets (not the union coverage over the rect) — else a stroke
        // whose bbox merely CONTAINS an earlier wash re-wet that wash into a rectangle. `has_owner` false
        // (no style map) ⇒ pour the whole rect, the pre-owner-map behaviour.
        let cur_o = self.paint.wet_styles.current_owner();
        let has_owner = self.paint.wet_styles.owner.len() == n;
        // Pour the HARDENED coverage (the composite's own `cw` smoothstep): moisture is "the wash is
        // here", so the wash INTERIOR is fully wet even at low dab flow — pouring the raw coverage
        // left the rim only half-suppressed at the junction.
        //
        // ⚠️ **A dureza é função PURA de um `u8`**, então ela é uma TABELA de 256 entradas e não cinco
        // operações de ponto flutuante por texel (a mesma leitura que a §5.43 fez do `alpha_of_mass`).
        //
        // ⚠️ **E o walk é row-parallel** — irmão do decaimento, sob a MESMA emenda do [ADR-0109], e o
        // caso mais simples dos dois: aqui **não há redução nenhuma**. Cada linha lê `owner` e
        // `stroke_coverage` (imutáveis) e escreve só a própria fatia de `canvas_wet`; o rect é unido
        // depois, fora do laço, a partir de números que o laço não produz. Medido pelo log do produto
        // em 9,6-13,6 ms/quadro, o segundo maior item do quadro do artista.
        //
        // [ADR-0109]: ../../../../docs/architecture/decisions/0109-rayon-exception-watercolor-composite.md
        let rect = (x0, y0, x1, y1);
        let (wx0, wy0, wx1, wy1) = self.janela_do_pour(rect, cur_o, has_owner);
        self.paint.wet_pour_marca = has_owner.then_some(MarcaDoPour {
            dono: cur_o,
            pendente: None,
        });
        #[cfg(test)]
        POUR_PX.set(POUR_PX.get() + ((wx1 - wx0) * (wy1 - wy0)) as u64);
        let lut = pour_hardening_lut();
        let cov = &self.paint.stroke_coverage;
        let owner = &self.paint.wet_styles.owner;
        let band = &mut self.paint.canvas_wet[wy0 * fw..wy1 * fw];
        let fold = |y: usize, row: &mut [u8]| {
            let base = (wy0 + y) * fw;
            for x in wx0..wx1 {
                if has_owner && owner[base + x] != cur_o {
                    continue;
                }
                let c = lut[usize::from(cov[base + x])];
                if c > row[x] {
                    row[x] = c;
                }
            }
        };
        if (wx1 - wx0) * (wy1 - wy0) >= WET_PAR_MIN {
            use rayon::prelude::*;
            band.par_chunks_mut(fw).enumerate().for_each(|(y, row)| {
                fold(y, row);
            });
        } else {
            band.chunks_mut(fw).enumerate().for_each(|(y, row)| {
                fold(y, row);
            });
        }
        self.paint.canvas_wet_rect = Some(match self.paint.canvas_wet_rect {
            Some((rx0, ry0, rx1, ry1)) => (rx0.min(x0), ry0.min(y0), rx1.max(x1), ry1.max(y1)),
            None => (x0, y0, x1, y1),
        });
        self.paint.canvas_wet_rev = veu::nova_revisao();
    }

    /// EDGE-1 (doc 13 #9): END the wet session NOW — the atomic teardown (drop the moisture map +
    /// every session/union buffer + the frozen base + soak). The baked washes STAY in `canvas_rgba`
    /// (this doesn't touch pixels); only the FUSION with future strokes ends — Rebelle "Dry the
    /// layer". Shared by the drying-deadline path above, the **Dry** button, and undo/redo (the canvas
    /// identity changed, so the moisture map is stale — clear it, Enio 2026-07-11). Idempotent.
    pub(crate) fn dry_session_now(&mut self) {
        self.paint.canvas_wet = Vec::new();
        self.paint.canvas_wet_rect = None;
        self.paint.canvas_wet_carry = 0.0;
        self.paint.canvas_wet_rev = veu::nova_revisao();
        self.paint.wet_session_base = None;
        self.paint.wet_session_canvas = None;
        self.paint.stroke_coverage = Vec::new();
        self.paint.stroke_color = Vec::new();
        self.paint.stroke_density = Vec::new();
        self.paint.stroke_deplete = Vec::new();
        self.paint.stroke_deplete_prox = Vec::new();
        self.paint.wet_reserve_cache = None;
        self.paint.wet_rewet_cache = None; // B16.5: idem
        self.paint.wet_styles.clear();
        self.paint.stroke_water = Vec::new();
        self.paint.wet_soak = Vec::new();
        self.paint.wet_soak_active = false;
        self.paint.wet_cum_dirty = None;
        self.paint.wet_stroke_dirty = None;
        self.paint.wet_pour_marca = None;
        self.paint.wet_session_wetness = 0.0; // the forced "Wet the layer" rewet ends with the session (#3)
        self.clear_wet_editable(); // the wash is now permanent — no more live texture edits
    }

    /// Drop the live-editable wash snapshot (Enio 2026-07-11): the last wash stops re-rendering on a
    /// texture-param change. Called when the wash becomes permanent — the session dries or a NEW wet render
    /// starts (a fresh ground is frozen). The baked pixels in `canvas_rgba` are untouched.
    pub(super) fn clear_wet_editable(&mut self) {
        self.paint.wet_editable_base = None;
        self.paint.wet_editable_backdrop = None;
        self.paint.wet_editable_region = None;
        self.paint.wet_editable_tex = None;
    }

    /// Re-render the **live-editable wash** (Enio 2026-07-11): reconstruct the LAST committed wash from its
    /// kept pre-wash base + frozen ground with the CURRENT brush texture, over the committed footprint
    /// (already full-axis on a tiled axis ⇒ central AND every Tiling copy). Driven by the paint tick when a
    /// Grain/Paper param moved while the paper is still wet, so the wash reflects the new texture live — not
    /// only the next stroke. No editable wash / mis-sized buffers ⇒ no-op (byte-identical).
    pub(super) fn rerender_editable_wash(&mut self) {
        let (Some(base), Some(region)) = (
            self.paint.wet_editable_base.clone(),
            self.paint.wet_editable_region,
        ) else {
            return;
        };
        let (fw, fh) = self.source_size;
        let n = (fw as usize) * (fh as usize);
        if n == 0
            || base.len() != n * 4
            || self.paint.stroke_coverage.len() != n
            || self.paint.stroke_color.len() != n * 4
        {
            return;
        }
        // The paper-tooth memo is keyed by canvas pixel and NOTHING else — its only invalidation is the
        // NaN-reset at pen-down, on the premise (written into the field's doc) that "the paper cannot
        // change mid-stroke". THIS function is what made that premise false: it exists precisely to
        // re-render when a Paper param moved. `fill_substrate_cache` fills only the NaN misses, so without
        // this reset every already-memoised pixel keeps the tooth of the OLD paper — the pool re-renders
        // and the paper does not change, which defeats this feature's whole purpose for the Paper slot.
        // (The Grain is sampled live, which is why the Grain half worked and the smoke passed.)
        // Sweep 2026-07-12, `BUGS_painter.md` #14. Same root as Bug #12: a reuse guard that never asks
        // whether the cached value still belongs to the inputs that produced it.
        self.repoe_o_substrato();
        // `close_stroke` dropped the live base + ground; point the render at the kept snapshot and force the
        // committed footprint dirty, then reconstruct the wash with the CURRENT texture (`apply_watercolor`
        // reads `brush.texture`/`paper`). `commit=false` keeps the base for the next edit.
        self.paint.watercolor_base = Some(Arc::clone(&base));
        self.paint.wet_backdrop = self.paint.wet_editable_backdrop.clone();
        self.paint.wet_frame_dirty = Some(region);
        self.apply_watercolor(false);
        // Keep ONLY the snapshot live (mirror the post-`close_stroke` state) and re-arm the session guard —
        // `apply_watercolor`'s `Arc::make_mut` forked `canvas_rgba`, so the old guard Arc no longer matches.
        self.paint.watercolor_base = None;
        self.paint.wet_backdrop = None;
        self.arma_a_guarda_da_sessao();
        self.paint.wet_editable_tex = Some(self.wet_editable_sig());
    }
}

/// The on-canvas wetness overlay's read (#12a): `(moisture bytes, w, h, rect [x0, y0, x1, y1])`.
pub type CanvasWetView<'a> = (&'a [u8], u32, u32, [u32; 4]);

#[cfg(test)]
#[path = "watercolor_backdrop_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "watercolor_backdrop_pour_tests.rs"]
mod pour_tests;
