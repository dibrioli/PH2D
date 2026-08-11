//! The **Impasto relief's per-stroke state** — split out of the `PaintState` god-struct for the
//! workspace LOC cap, and because it is one coherent thing: the planes a stroke deposits its body into,
//! the ingredients the Body card re-derives that body FROM, and the window all of them are indexed
//! against. Everything here is born at pen-down and dies at the next one; nothing survives the document.

use super::Region;

/// The tip a Symmetry copy's bow wave is standing at: exactly what [`ph2d_painter_brush::height_push::
/// wave_lobe`] needs to recompute — and therefore exactly negate — the lobe it painted there.
#[derive(Clone, Copy)]
pub(super) struct WaveTip {
    pub(super) center: [f32; 2],
    pub(super) radius: f32,
    pub(super) rotation: [f32; 2],
    pub(super) prev_center: Option<[f32; 2]>,
}

/// See the module docs. Fields are `pub(super)` so the whole `paint` module tree reaches them exactly as
/// it did when they lived on `PaintState` directly.
#[derive(Default)]
pub(super) struct ReliefState {
    /// **Impasto** per-stroke relief (f32, `w*h`) — the deposit ([`ph2d_painter_brush::height::
    /// derive_height`] of the ingredient planes below) plus the displacement the brush banked. Merged
    /// into the layer at `close_stroke`. Empty ⇒ no impasto this stroke (zero cost).
    pub(super) stroke_height: Vec<f32>,
    /// **Impasto** per-stroke PAINT envelope (f32, `0..1`, by `max` — the heaviest dab owns the pixel,
    /// so one pass leaves one thickness). Both the stroke's coverage (what the light weighs its shading
    /// by; merged into the layer's `covers` at stroke end) and the first ingredient of the relief.
    pub(super) stroke_paint: Vec<f32>,
    /// **Impasto** per-stroke DISPLACEMENT at `Push = 1` (f32, `w*h`; `ph2d_painter_brush::height_push`)
    /// — negative where the brush took paint, positive where it banked it, summing to zero. Linear in
    /// Push, so it is an INGREDIENT: the ridge appears under the moving brush *and* the knob stays live.
    pub(super) stroke_push: Vec<f32>,
    /// **Impasto** per-stroke GRAIN (1 byte/px, `255` = none): the grain sample of the dab that won each
    /// pixel. The second ingredient — what lets `Depth Source` be flipped AFTER the stroke and re-carve
    /// the very grooves that dab would have left.
    pub(super) stroke_grain: Vec<u8>,
    /// The **solid paint** this stroke laid (`ph2d_painter_brush::height::solid_paint`) — the film's own
    /// alpha, and the coverage the LIGHT weighs its shading by. Separate from [`Self::stroke_paint`] (the
    /// relief's ingredient, the raw silhouette × dynamics) because the two are different functions of the
    /// dab: the body curve runs on the silhouette and the dynamics scale it, which is what keeps the light
    /// alive under a light touch. Merged into the layer's `covers` at stroke end.
    pub(super) stroke_film: Vec<u8>,
    /// The winning dab's radius per texel — the THIRD ingredient (see `HeightFields::radius`): the
    /// height scales with the dab's size, and the drag-sized methods make that size a per-stroke
    /// fact, not the panel knob. Same envelope winner as `stroke_paint`.
    pub(super) stroke_radius: Vec<f32>,
    /// **Impasto**: where each Symmetry copy of the stroke was when the LAST pointer batch ended — and
    /// how BIG that dab was — so the first dab of the next batch can sweep back to it. Without it the
    /// relief would bead at every pointer event — a beading chosen by the artist's mouse polling rate,
    /// not by their hand. One slot per copy; cleared on pen-down. Mirrors `last_smear_pos` (which is
    /// the same idea, for the smear chain), but per-copy, because Symmetry paints several strokes at
    /// once. The radius rides along because the sweep is CONDITIONAL on overlap (`(center, radius)` —
    /// see the capsule law in `stamp_dabs_height`): the batch-boundary sweep must ask the same
    /// question the in-batch sweep asks, or the phantom bar would come back once per pointer event.
    pub(super) last_height_center: Vec<Option<([f32; 2], f32)>>,
    /// **The bow wave, one per Symmetry copy**: the cargo each copy's tip is carrying (loads·px²)
    /// and the exact lobe last painted for it — enough to UN-paint it bit-for-bit before the next
    /// dab of that copy repaints it further along (`height_push::wave_lobe`; the removal runs
    /// BEFORE the dab's own deposit touches `stroke_paint`, so the `(1−paint)` weights are the
    /// same numbers that laid it). Cleared with the stroke: the wave is a fact about the dab list.
    pub(super) stroke_wave: Vec<(f32, Option<WaveTip>)>,
    /// **Impasto live-edit** — the LAST stroke's ingredients, so the whole Body card re-derives that
    /// stroke after the fact instead of only affecting the next one (Enio 2026-07-12). Storing the
    /// HEIGHT instead bakes Body and Depth Source into it, leaving nothing to re-derive them from — the
    /// first cut's bug. Empty ⇒ nothing live.
    pub(super) live_paint: Vec<f32>,
    /// That stroke's grain plane — the second ingredient (see [`Self::stroke_grain`]).
    pub(super) live_grain: Vec<u8>,
    /// That stroke's per-texel dab radius — the third ingredient (see [`Self::stroke_radius`]).
    pub(super) live_radius: Vec<f32>,
    pub(super) live_push: Vec<f32>, // that stroke's displacement at `Push = 1` — see [`Self::stroke_push`]
    pub(super) push_scratch: Vec<f32>, // per-dab rim weights, reused (so `bank_dab_push` allocates nothing)
    /// The active layer's committed relief BEFORE that stroke — the ground the re-derived stroke is
    /// added back onto. **A PATCH over [`Self::live_relief_rect`], not the canvas**: outside that rect
    /// the stroke contributes nothing, so the layer's relief there is its own and never re-derived.
    /// Empty ⇒ the layer had none (the common case: a first stroke).
    pub(super) live_relief_base: Vec<f32>,
    /// The last stroke's **film** — its solid-paint alpha, as a PATCH over [`Self::live_relief_rect`].
    /// Kept for exactly one reason: it is the weight the MATERIAL merges with, so re-baking the material
    /// after the fact (the artist turning Roughness while looking at the stroke) needs it. Without it the
    /// four material knobs would only affect the NEXT stroke — a knob that does nothing to what you are
    /// looking at, which is the failure mode this section keeps producing (§17).
    pub(super) live_film: Vec<u8>,
    /// The active layer's MATERIAL from BEFORE that stroke, as a patch over the same rect — the ground
    /// the re-baked material is merged back onto. The `over` merge is NOT idempotent, so re-baking has to
    /// start from what was there, exactly like [`Self::live_relief_base`] does for the height.
    pub(super) live_mat_base: Vec<ph2d_painter_brush::material::MaterialBytes>,
    /// Which layer the live stroke belongs to. `None` ⇒ nothing live.
    pub(super) live_relief_layer: Option<crate::tool::RtLayerId>,
    /// The union of this stroke's dab footprints, in canvas texels — accumulated as the relief is
    /// deposited, cleared with the stroke.
    ///
    /// The commit used to re-derive, settle, diff and re-base over the **whole canvas** for a stroke that
    /// touched a corner of it — a one-SECOND freeze at every pen-up at 4096² (§11). This is the window
    /// that makes that work `O(stroke)`.
    pub(super) stroke_relief_bbox: Option<Region>,
    /// [`Self::stroke_relief_bbox`] grown by the settle's reach and clipped to the canvas — the window
    /// the live re-derive owns. Every buffer above that is "per-stroke" is indexed against THIS.
    pub(super) live_relief_rect: Option<Region>,
    /// Whether the layer already carried a height entry before the live stroke — so a re-derive that
    /// zeroes the stroke out (Depth 0) knows whether the entry is now empty or merely untouched here.
    pub(super) live_relief_had_entry: bool,
    /// **A massa como a BORRACHA a encontrou** — o `pre` dela, e a janela que ela já mordeu.
    ///
    /// O eraser não tem envelope próprio a descascar: ele esfrega o plano que já está COMMITADO na camada
    /// (`super::impasto`, a metade `erasing`). Isso é a mesma forma do sculpt, e o
    /// `stamp_preview::stamp_drag_preview` já explica por que o sculpt precisa restaurar — *sem isso uma
    /// Curve cavaria mais fundo a cada movimento do ponteiro enquanto o artista apenas OLHAVA*. O eraser
    /// não tinha restauração nenhuma, e a mordida de toda figura por onde a mão passou ficava para sempre.
    pub(super) erase_pre: Option<ErasePre>,
    /// Os cinco planos que o traço anterior devolveu — ver [`SparePlanes`].
    pub(super) spare: SparePlanes,
    /// **Desliga a rota do pool**, e ela existe para o gate de identidade poder fazê-lo.
    ///
    /// ⚠️ **O sentido é NEGATIVO de propósito:** o `Default` desta struct é derivado, e um
    /// `pool: bool` nasceria `false` — a cura desligada em todo caminho de produto, com a suíte verde.
    /// Um flag cujo valor neutro é o `Default::default()` do tipo não pode ser esquecido; é o molde do
    /// `ph2d_painter_brush::ablate`, cuja máscara neutra é `0`.
    ///
    /// É ablação de **ROTA**, não de peça: as duas rotas TÊM de escrever o mesmo byte, e provar isso é
    /// a razão de ela existir.
    pub(super) planes_pool_off: bool,
    /// **Quantas vezes o pool de facto SERVIU.** É a métrica do ADR-0120 (*o gate que conta quantas
    /// vezes o caminho rápido dispara*): sem ela, um pool que nunca acerta deixa o gate de identidade
    /// mais verde do que nunca — as duas rotas seriam a mesma — enquanto o produto segue a alocar.
    pub(super) planes_pooled: u32,
}

/// A camada como a borracha do impasto a encontrou, mais a janela que ela mordeu desde então.
///
/// ⚠️ Os dois planos são `Arc` da entrada que estava no mapa — **refcount, não cópia**. O
/// `stamp_dabs_height` ARRANCA o `Arc` do mapa e insere outro objeto no fim, então guardar o antigo
/// congela o estado de partida sem uma segunda alocação canvas-sized por batch.
pub(super) struct ErasePre {
    /// De quem é este `pre`. Trocar de camada com a borracha em mãos encerra a sessão: restaurar um plano
    /// numa camada que não é a dele escreveria a massa de outro desenho.
    pub(super) layer: crate::tool::RtLayerId,
    pub(super) heights: std::sync::Arc<Vec<f32>>,
    pub(super) covers: std::sync::Arc<Vec<u8>>,
    /// A união do que a borracha esfregou desde o congelamento — a janela que a restauração percorre.
    /// `None` ⇒ nada a devolver, e a restauração é um no-op sem tocar num byte.
    pub(super) bbox: Option<Region>,
}

/// **Os cinco planos que um traço terminado DEVOLVE** — e a janela em que ficaram sujos.
///
/// Um traço do relevo pede **83 MB a 2048²** (`tests/measure_pendown_alloc.rs`, contados pelo `dhat`),
/// dos quais **56 são estes cinco planos**: o `reset_stroke_height` faz `clear()` — que preserva a
/// capacidade — e o primeiro dab do traço seguinte a joga fora numa linha (`h.len() != n ⇒ h =
/// vec![0.0; n]`). Duas linhas discordando sobre o mesmo buffer.
///
/// ⚠️ **E o preço disso NÃO é o de um memset:** medidas em sequência, as mesmas cinco alocações deram
/// **0,008 · 0,028 · 7,586 ms** — três ordens de grandeza, porque `alloc_zeroed` custa o que o alocador
/// tiver de fazer para arranjar páginas zeradas. É por isso que o pen-down do filme media *plano na
/// tela e plano no raio*: o que ele paga não é trabalho por texel, é o alocador.
///
/// A cura é fazê-los **CIRCULAR**, e o que a torna byte-idêntica é a janela: um traço só escreve dentro
/// do próprio `stroke_relief_bbox` (o walk devolve a janela em que escreve, e é dela que o bbox cresce),
/// então zerar essa janela devolve exactamente o plano que o `vec![0.0; n]` devolveria. O custo deixa de
/// ser função do DOCUMENTO e passa a ser função do TRAÇO — o mesmo movimento que o histórico de undo
/// fez na U1, um plano acima.
///
/// ⚠️ **A janela guardada é a UNIÃO, e ela tem de ser** — os cinco não vêm todos do mesmo traço: o
/// `height` e o `film` são deste (o commit os consome), e o `paint`/`grain`/`radius` são os do traço
/// ANTERIOR, que o commit acabou de substituir nos `live_*`. Uma janela por buffer seria mais apertada
/// e mais fácil de errar; a união é um superconjunto, e um superconjunto zera de mais, nunca de menos.
#[derive(Default)]
pub(super) struct SparePlanes {
    pub(super) planes: StrokePlanes,
    /// Onde os cinco estão sujos. `None` ⇒ não há nada guardado.
    pub(super) dirty: Option<Region>,
}

/// **Os cinco planos, NOMEADOS** — e o nome é o que impede a troca silenciosa.
///
/// ⚠️ Três deles são `Vec<f32>` (`height`, `paint`, `radius`), então uma tupla posicional aceita a
/// ordem errada **sem o compilador dizer nada**: trocar dois deles compilaria e corromperia o relevo.
/// O compilador só pegou a minha primeira versão porque a troca que fiz calhou de ser `f32` com `u8`.
#[derive(Default)]
pub(super) struct StrokePlanes {
    pub(super) height: Vec<f32>,
    pub(super) paint: Vec<f32>,
    pub(super) grain: Vec<u8>,
    pub(super) film: Vec<u8>,
    pub(super) radius: Vec<f32>,
}

/// Zera `rect` num plano canvas-shaped de largura `w` — uma `fill` por linha, que é um memset.
fn zero_rows<T: Copy + Default>(buf: &mut [T], rect: Region, w: u32) {
    for y in rect.y..rect.y + rect.h {
        let row = (y as usize) * (w as usize);
        let (a, b) = (row + rect.x as usize, row + (rect.x + rect.w) as usize);
        if b <= buf.len() {
            buf[a..b].fill(T::default());
        }
    }
}

impl ReliefState {
    /// **Toma os cinco planos do pool, já zerados na janela suja** — ou `None`, e o chamador aloca.
    ///
    /// Tudo-ou-nada: cinco buffers do tamanho certo ou nenhum. Uma retomada parcial teria de decidir o
    /// que fazer com os outros, e a decisão certa é a que o chamador já tem escrita (`vec![0.0; n]`).
    pub(super) fn take_planes(&mut self, n: usize, w: u32) -> Option<StrokePlanes> {
        if self.planes_pool_off {
            return None;
        }
        let dirty = self.spare.dirty?;
        let p = &self.spare.planes;
        if p.height.len() != n
            || p.paint.len() != n
            || p.grain.len() != n
            || p.film.len() != n
            || p.radius.len() != n
        {
            return None;
        }
        let mut p = std::mem::take(&mut self.spare.planes);
        self.spare.dirty = None;
        self.planes_pooled += 1;
        zero_rows(&mut p.height, dirty, w);
        zero_rows(&mut p.paint, dirty, w);
        zero_rows(&mut p.grain, dirty, w);
        zero_rows(&mut p.film, dirty, w);
        zero_rows(&mut p.radius, dirty, w);
        Some(p)
    }

    /// **Guarda os cinco planos que o commit acabou de aposentar**, com a janela em que estão sujos.
    pub(super) fn retire_planes(&mut self, planes: StrokePlanes, dirty: Option<Region>) {
        let Some(dirty) = dirty else { return };
        self.spare = SparePlanes {
            planes,
            dirty: Some(dirty),
        };
    }
}
