//! The dab's **raster core** (child of [`super`], split for the workspace
//! file-LOC cap): the parallel band splitter, the shared per-band pixel kernel
//! ([`stamp_band`] over a [`DabCtx`]) and the byte encode/composite helpers.
//! The public stamp API and the silhouette/grain doors stay in the parent —
//! this file is how a dab becomes pixels, not what a dab is.

use super::*;
use crate::ramp_alpha::{RampAlphaMode, RampIndex};
use crate::spec::BrushSpec;
use crate::texture::{ImageMask, TexDabBasis};
use ph2d_det_math::DetMath as _;
use rayon::prelude::*;

/// **Quantas visitas de texel uma thread precisa RECEBER para se pagar** — a razão medida entre o
/// custo de ABRIR uma thread e o custo de VISITAR um texel.
///
/// Medido na RTX em 2026-08-04, máquina calma, pela sonda
/// `measure_route_cost::what_the_banded_batch_buys_when_the_cap_is_on`: abrir uma banda custa
/// **~10,5 µs** e uma visita de texel custa **~13 ns** na rota serial ⇒ `10,5 µs / 13 ns ≈ 808`.
///
/// ⚠️ **O 10,5 µs saiu do estado ANTES desta correção, e a correção o apagou da tabela** — o piso
/// chato de 0,33-0,39 ms que TODA linha pequena mostrava era `32 × 10,5 µs`, o custo de abrir 32
/// threads para um trabalho de 3 µs cada. Depois que a contagem passou a sair do trabalho aquele
/// piso não existe mais (as mesmas linhas medem 0,10-0,20 ms), então *quem quiser re-derivar o
/// número tem de ablacionar a contagem, não reler a tabela de hoje*: a evidência foi consumida pela
/// própria cura.
///
/// ⚠️ **É uma RAZÃO entre dois custos, não um teto.** É disso que ela é feita, e é por isso que ela
/// sobrevive a uma máquina diferente: numa com menos núcleos o `cores` cai e o `clamp` resolve; numa
/// com thread mais cara o número real sobe, e o ótimo é **plano** em volta dele (medido: para
/// 110 224 visitas, `T = 8` custa 263 µs, `T = 16` custa 258 e `T = 24` custa 312 — errar por 2× no
/// constante custa ~2%).
pub(crate) const SPAWN_EQUIV_VISITS: usize = 808;

/// A menor área que pede DUAS bandas — **derivada**, nunca escolhida (é onde o `band_count_with`
/// deixa de devolver 1).
///
/// ⚠️ **Ela era `1 << 17 = 131 072` e o número estava errado por 40×, com um mecanismo que a
/// medição nomeou.** O texto antigo dizia *"abaixo dela a rota serial vence (abrir thread não vale
/// ~1 ms de trabalho)"* — a frase certa sobre a coisa errada: o que não se pagava não era a DIVISÃO,
/// era abrir **32 threads** para um trabalho que rende 3 µs por thread.
///
/// Medido **com o piso antigo e as 32 threads**, um dab de pegada 33 489 já pagava `1,33×` e um de
/// 67 081 pagava `2,64×` — os dois **abaixo** daquele piso, os dois mandados para a rota serial. Com
/// a contagem derivada os mesmos dois pagam **`2,95×` e `4,74×`**.
///
/// ⚠️ E o `1 << 17` não era arbitrário: `131 072 = 32 × 4096`, ou seja **a área em que as 32 threads
/// desta máquina recebem 4 k visitas cada**. O número foi escolhido para UM ponto de operação (todos
/// os núcleos, este hardware) e aplicado como se fosse uma propriedade do trabalho.
pub const PARALLEL_MIN_AREA: usize = SPAWN_EQUIV_VISITS * 4;

/// **Em quantas BANDAS dividir um trabalho de `area` visitas sobre `rows` linhas** — a porta ÚNICA,
/// e `1` significa *serial*.
///
/// # O ótimo é uma RAIZ, não uma divisão, e é isso que o `1 << 17` não sabia
///
/// Dividir custa `T · c_spawn` e rende `area · c_visita / T`, então o tempo é
/// `T·c_spawn + area·c_visita/T` — mínimo em **`T* = √(area / SPAWN_EQUIV_VISITS)`**. O modelo foi
/// conferido contra o produto antes de virar código: para 110 224 visitas em 32 threads ele prevê
/// **381 µs** e a sonda mediu **382**.
///
/// ⚠️ **O que estava quebrado era o `T` ser CONSTANTE.** `available_parallelism()` devolve o mesmo
/// número para um dab de 7 k visitas e para um de 500 k, então o pequeno pagava 32 spawns por 3 µs de
/// trabalho cada — e a única defesa era um piso que o mandava inteiro para a rota serial. *Um cliff é
/// o que se constrói quando o knob certo não existe.*
///
/// ⚠️ **Isto NÃO move um byte, e é a espinha da wave inteira:** bandas são linhas disjuntas e cada uma
/// percorre todos os dabs na mesma ordem, então o resultado é bit-idêntico ao serial **para qualquer
/// contagem de bandas** (HR-5, a invariante que o doc do [`band_split`] já declarava). Mudar o número
/// de bandas é mudar quem AVALIA a linha, nunca o que ela diz.
pub fn band_count(area: usize, rows: usize, min_area: usize) -> usize {
    // ⚠️ **A recusa vem ANTES de perguntar à máquina, e o preço disso está medido.** A primeira versão
    // desta função consultava o `available_parallelism()` no topo, e a sonda mediu a rota em banda de
    // um lote de 128 dabs indo de **0,99 para 9,79 ms** — 10× — com a MESMA contagem de bandas: o
    // `band_split` roda uma vez por dab POR BANDA (128 × 32 = 4096 chamadas), e ali um syscall por
    // chamada é o custo inteiro. *Nenhum gate de identidade podia ver isso; quem viu foi o relógio.*
    if area < min_area || rows <= 1 {
        return 1;
    }
    band_count_with(area, rows, min_area, cores())
}

/// Quantos núcleos esta máquina tem — perguntado **uma vez por processo**.
///
/// ⚠️ `std::thread::available_parallelism()` é um **syscall** (`sched_getaffinity` no Linux), e este
/// caminho é o mais quente do depósito: um dab por banda, por lote. A resposta não muda enquanto o
/// processo vive, então guardá-la não é cache — é parar de fazer a mesma pergunta ao kernel do SO
/// milhares de vezes por quadro. Isto **já** era custo antes desta wave, para todo dab acima do piso.
fn cores() -> usize {
    static CORES: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *CORES.get_or_init(|| {
        std::thread::available_parallelism()
            .map(std::num::NonZero::get)
            .unwrap_or(1)
    })
}

/// [`band_count`] com o número de núcleos **injetado** — a face pura, que é a que um gate consegue
/// interrogar (a outra pergunta à máquina, e uma máquina não é uma fixture).
/// As faixas de um carimbo no POOL do `rayon` (ADR-0200): a tarefa custa ~1–3 µs contra os 10,5 µs de abrir uma
/// thread ([`SPAWN_EQUIV_VISITS`]), logo a faixa paga-se com [`VISITAS_POR_FAIXA_NO_POOL`] visitas e o carimbo
/// grande usa os núcleos todos. O véu do Wet Wash Mix (~38 mil visitas, ~107 ns cada) abria 6 threads; abre até 18.
/// Cada faixa tem linhas disjuntas: o byte é o mesmo com qualquer contagem.
pub(crate) fn faixas_no_pool(area: usize, rows: usize, min_area: usize) -> usize {
    if area < min_area || rows <= 1 {
        return 1;
    }
    (area / VISITAS_POR_FAIXA_NO_POOL).clamp(1, cores().min(rows))
}

/// Visitas de texel que pagam uma tarefa do pool: 2 048 × ~13 ns (a visita mais barata) ≈ 27 µs, ≥ 10× o custo da
/// tarefa.
pub(crate) const VISITAS_POR_FAIXA_NO_POOL: usize = 2048;

pub(crate) fn band_count_with(area: usize, rows: usize, min_area: usize, cores: usize) -> usize {
    if area < min_area || rows <= 1 {
        return 1;
    }
    (area / SPAWN_EQUIV_VISITS)
        .isqrt()
        .clamp(1, cores.max(1).min(rows))
}

/// Run `stamp` over the dab's row span `[y0, y1)` — serial for small footprints, split into disjoint
/// row bands across the cores for large ones (≥ [`PARALLEL_MIN_AREA`]). `stamp(dst, band_y0)` writes
/// the full-width band slice `dst` whose first row is `band_y0`, returning whether it touched a
/// pixel. Disjoint bands ⇒ the result is bit-identical to serial regardless of band count (HR-5).
/// Shared by the per-pixel [`stamp_dab_textured`] and the cached-mask [`crate::stamp::blit_stamp`].
///
/// A face **sem máscara** de [`band_split`]; a com máscara é [`parallel_band_stamp_masked`].
pub(crate) fn parallel_band_stamp<F>(
    buf: &mut [u8],
    y0: i64,
    y1: i64,
    x0: i64,
    x1: i64,
    stride: usize,
    stamp: F,
) -> bool
where
    F: Fn(&mut [u8], i64) -> bool + Sync,
{
    band_split::<u8, _>(
        buf,
        None,
        None,
        None,
        y0,
        y1,
        x0,
        x1,
        stride,
        PARALLEL_MIN_AREA,
        |dst, _l, _m, _f, by0| (stamp(dst, by0), false),
    )
    .0
}

/// [`parallel_band_stamp`] carregando a **máscara por-traço** (o cap de Accumulate) fatiada nas MESMAS
/// linhas.
///
/// # Por que ela existe
///
/// Até 2026-08-04 o caminho do cap rodava **numa banda só**, com o motivo escrito no `dab.rs`:
/// *"reads+writes the shared per-stroke mask, so it runs SERIALLY — small soft-brush dabs anyway, where
/// the cap is observable"*. ⚠️ **A premissa era verdadeira e deixou de ser:** o cap disparava só em
/// `strength < 1`, e o **AA do filme do impasto** (`film_aa_wanted`) passou a ligá-lo para TODO pincel
/// de impasto — inclusive os maiores do app, que são exatamente os que cruzam o piso. Medido: com o dab
/// **abaixo** do piso o cap custa `0,99×` (grátis), **acima** dele custa `4,15×` — o vão inteiro é o
/// paralelismo perdido, zero é a aritmética do cap.
///
/// # Por que continua bit-idêntico
///
/// A máscara é **row-major com `stride / 4` bytes por linha** (o `stamp_band` já a indexa em coordenada
/// de BANDA), então fatiá-la pelas MESMAS linhas do canvas dá a cada banda texels que só ela toca. O
/// kernel lê e escreve **o próprio texel** nos dois buffers e não olha vizinho, então um texel qualquer
/// é visitado pelo mesmo dab, na mesma ordem, com a mesma aritmética — muda quem AVALIA a linha, nunca
/// o que a linha diz (ADR-0109, o mesmo argumento que o canvas já usava).
///
/// ⚠️ **Uma régua, dois buffers:** `stride` decide as duas fatias (`stride` e `stride / 4`), então elas
/// não podem discordar sobre onde uma banda começa.
#[allow(clippy::too_many_arguments)]
pub(crate) fn parallel_band_stamp_masked<F>(
    buf: &mut [u8],
    mask: Option<&mut [u8]>,
    forca: Option<&mut [u16]>,
    y0: i64,
    y1: i64,
    x0: i64,
    x1: i64,
    stride: usize,
    min_area: usize,
    stamp: F,
) -> bool
where
    F: Fn(&mut [u8], Option<&mut [u8]>, Option<&mut [u16]>, i64) -> bool + Sync,
{
    band_split(
        buf,
        None,
        mask,
        forca,
        y0,
        y1,
        x0,
        x1,
        stride,
        min_area,
        |dst, _l, m, f, by0| (stamp(dst, m, f, by0), false),
    )
    .0
}

/// A divisão por linhas — **uma** resposta a *"vale dividir, e onde cada banda começa?"*.
#[allow(clippy::too_many_arguments)]
pub(super) fn band_split<M: Send, F>(
    buf: &mut [u8],
    liso: Option<&mut [u8]>,
    mask: Option<&mut [M]>,
    forca: Option<&mut [u16]>,
    y0: i64,
    y1: i64,
    x0: i64,
    x1: i64,
    stride: usize,
    min_area: usize,
    stamp: F,
) -> (bool, bool)
where
    F: Fn(&mut [u8], Option<&mut [u8]>, Option<&mut [M]>, Option<&mut [u16]>, i64) -> (bool, bool)
        + Sync,
{
    let mrow = stride / 4;
    let region = &mut buf[(y0 as usize) * stride..(y1 as usize) * stride];
    let liso_region = liso.map(|l| &mut l[(y0 as usize) * stride..(y1 as usize) * stride]);
    let mask_region = mask.map(|m| &mut m[(y0 as usize) * mrow..(y1 as usize) * mrow]);
    let forca_region = forca.map(|f| &mut f[(y0 as usize) * mrow..(y1 as usize) * mrow]);
    let rows = (y1 - y0) as usize;
    let area = rows * ((x1 - x0).max(0) as usize);
    let threads = faixas_no_pool(area, rows, min_area);
    if threads <= 1 {
        return stamp(region, liso_region, mask_region, forca_region, y0);
    }
    let rows_per_band = rows.div_ceil(threads);
    // As duas fatias saem da MESMA contagem de linhas; um `zip` as manteria alinhadas só enquanto os
    // dois iteradores tivessem o mesmo comprimento, e é justamente isso que um erro de stride quebra.
    let mut mask_bands = mask_region.map(|m| m.chunks_mut(rows_per_band * mrow));
    let mut forca_bands = forca_region.map(|f| f.chunks_mut(rows_per_band * mrow));
    let mut liso_bands = liso_region.map(|l| l.chunks_mut(rows_per_band * stride));
    let bands: Vec<_> = region
        .chunks_mut(rows_per_band * stride)
        .enumerate()
        .map(|(bi, chunk)| {
            let lchunk = liso_bands.as_mut().and_then(Iterator::next);
            let mchunk = mask_bands.as_mut().and_then(Iterator::next);
            let fchunk = forca_bands.as_mut().and_then(Iterator::next);
            (
                chunk,
                lchunk,
                mchunk,
                fchunk,
                y0 + (bi * rows_per_band) as i64,
            )
        })
        .collect();
    // Todas as faixas correm (`map` + `reduce`, sem curto-circuito).
    bands
        .into_par_iter()
        .map(|(chunk, lchunk, mchunk, fchunk, band_y0)| {
            stamp(chunk, lchunk, mchunk, fchunk, band_y0)
        })
        .reduce(|| (false, false), |a, b| (a.0 | b.0, a.1 | b.1))
}

/// Like [`parallel_band_stamp`] but splits THREE row-aligned buffers across the bands: the RGBA
/// `canvas` (stride `width*4`) plus the canvas-space texture cache `tex` and its `ready` flags (both
/// stride `width`). Each band owns disjoint rows of all three, so the lazy cache fill + the blend run
/// race-free and bit-identical to serial (HR-5). Used by [`crate::stamp::blit_canvas_cached`].
#[allow(clippy::too_many_arguments)]
pub(crate) fn parallel_band_cached<F>(
    canvas: &mut [u8],
    tex: &mut [u8],
    ready: &mut [u8],
    forca: Option<&mut [u16]>,
    width: u32,
    y0: i64,
    y1: i64,
    x0: i64,
    x1: i64,
    stamp: F,
) -> bool
where
    F: Fn(&mut [u8], &mut [u8], &mut [u8], Option<&mut [u16]>, i64) -> bool + Sync,
{
    let (cstride, mstride) = ((width as usize) * 4, width as usize);
    let canvas = &mut canvas[(y0 as usize) * cstride..(y1 as usize) * cstride];
    let tex = &mut tex[(y0 as usize) * mstride..(y1 as usize) * mstride];
    let ready = &mut ready[(y0 as usize) * mstride..(y1 as usize) * mstride];
    let forca = forca.map(|f| &mut f[(y0 as usize) * mstride..(y1 as usize) * mstride]);
    let rows = (y1 - y0) as usize;
    let area = rows * ((x1 - x0).max(0) as usize);
    // A MESMA porta do [`band_split`]: *"vale dividir, e em quantas?"* é uma pergunta só, e uma
    // segunda cópia da regra é como este arquivo passou a ter dois pisos escritos à mão.
    let threads = faixas_no_pool(area, rows, PARALLEL_MIN_AREA);
    if threads <= 1 {
        return stamp(canvas, tex, ready, forca, y0);
    }
    let rpb = rows.div_ceil(threads);
    let mut forca_bands = forca.map(|f| f.chunks_mut(rpb * mstride));
    let bands: Vec<_> = canvas
        .chunks_mut(rpb * cstride)
        .zip(tex.chunks_mut(rpb * mstride))
        .zip(ready.chunks_mut(rpb * mstride))
        .enumerate()
        .map(|(bi, ((cb, tb), rb))| {
            let fb = forca_bands.as_mut().and_then(Iterator::next);
            (cb, tb, rb, fb, y0 + (bi * rpb) as i64)
        })
        .collect();
    bands
        .into_par_iter()
        .map(|(cb, tb, rb, fb, band_y0)| stamp(cb, tb, rb, fb, band_y0))
        .reduce(|| false, |a, b| a | b)
}

/// Read-only per-dab context shared by every row band of a [`stamp_dab_textured`] stamp. `Sync`
/// (refs to `Sync` data + `Copy` scalars), so `&DabCtx` is safely shared across the band threads.
pub(super) struct DabCtx<'a> {
    pub(super) spec: &'a BrushSpec,
    pub(super) tex: Option<&'a TexDabBasis>,
    pub(super) image: Option<ImageMask<'a>>,
    /// The Shape slot's silhouette inputs (frame + optional image). `Some` ⇒ the silhouette is the
    /// Shape, replacing the falloff; `None` ⇒ the falloff is the silhouette. See [`ShapeInput`].
    pub(super) shape: Option<ShapeInput<'a>>,
    /// The Shape whose VALUE the Texture Color reads ([`crate::RampPaint::reads_shape_value`]).
    pub(super) shape_cor: Option<ShapeInput<'a>>,
    /// Baked Color Ramp: its LUT, what its alpha does, and who indexes it ([`crate::RampIndex`]).
    pub(super) ramp: Option<crate::RampPaint<'a>>,
    /// Dab flatten/rotate for the falloff `t` (Shape/Grain carry it in their bases); baked once per dab.
    pub(super) footprint: crate::footprint::FootprintDeform,
    pub(super) center: [f32; 2],
    pub(super) cx: f32,
    pub(super) cy: f32,
    pub(super) inv_radius: f32,
    pub(super) radius: f32,
    pub(super) coverage: f32,
    /// Sob o tecto do traço (Accumulate off): o tecto (`coverage` sem o Flow) e a taxa por dab (o Flow).
    pub(super) tecto: f32,
    pub(super) flow: f32,
    pub(super) preserve_alpha: bool,
    pub(super) x0: i64,
    pub(super) x1: i64,
    pub(super) stride: usize,
    /// A coluna da tela que é a 1.ª do buffer (`0` na tela; a caixa do dab numa [`crate::PegadaLocal`]).
    pub(super) ox: i64,
    /// O buffer é uma [`crate::PegadaLocal`] nova: cada texel nasce `[0; 4]` e o dab escreve-o uma vez só.
    pub(super) vazio_local: bool,
    /// Screen-space AA of the film silhouette (BUGS #16) — `None` = single-sample `film_of`,
    /// byte-identical. Hoisted per dab in [`stamp_dab_pixels`].
    pub(super) film_aa: Option<crate::height_film::FilmAa>,
    /// The AA's pre-convolved table + deformed basis (plano 26 §9.6), when the brush admits it.
    /// `None` ⇒ the AA samples the silhouette chain nine times, as it always did.
    pub(super) lut: Option<crate::height_film::FilmLutPlan<'a>>,
    /// The dab-capsule ([`crate::capsula`]) and its law, read once per dab; `None` ⇒ the plain dab.
    pub(super) capsula: Option<crate::capsula::Capsula>,
    pub(super) lei: crate::capsula::Lei,
    /// O pó pousado antes do traço: o Height lê o papel com o dente cheio ([`crate::GrainLaw::papel_com_po`]).
    pub(super) po: Option<crate::texture::PoDoPapel<'a>>,
    /// A memória do grão Tiled ([`crate::texture::grao_cache`]): o mesmo `sample`, lido uma vez por píxel do traço.
    pub(super) grao: Option<std::sync::Arc<crate::texture::grao_cache::GraoCache>>,
    /// A memória do `t` do dente do papel ([`crate::texture::grao_cache::para_o_dente`], B26.A2).
    pub(super) dente: Option<std::sync::Arc<crate::texture::grao_cache::GraoCache>>,
}

mod deposito;
mod silhueta;
mod tinta;
use deposito::Onde;

/// O que um dab decide UMA vez e o laço por texel só lê (PX-17, B26.1): antes cada texel recalculava a taxa da
/// cápsula (`det_powf`), a orla em área, o Grain Depth/Granulation e o modo da rampa. Funções puras do dab e do
/// buffer (`mask` presente ou não), logo os mesmos bits (`oraculo_tests`).
pub(in crate::dab) struct Invariantes {
    /// A capsule composes its sub-dabs at the pass opacity: the dab's own, or — under the Accumulate cap, where
    /// `w` is a RATE toward the ceiling — the Flow of ONE sub-dab, `1 − (1 − flow)^(1/n)` (the step's rate, shared
    /// by its `n` sub-dabs): the rates multiply across capsules exactly as inside one, so the seam between two
    /// steps gets no extra pass (dono 25/10: Continuous + Flow < 1 em contas). Flow 1 ⇒ `1`, the law that shipped.
    /// `n` is the STEP's: the opening dab the 1.st capsule carries ([`crate::Ante::fecho`]) is one more sub-dab.
    pub(in crate::dab) c_cap: f32,
    /// A orla em PÍXEIS (J3, `BrushSpec::borda_px`) sob o tecto do traço é ÁREA: o texel pára na fracção que o
    /// dab cobre (o tecto da Smooth Edges, BUGS #16) e, na cápsula, no MAIOR sub-dab — a união deles enchia a
    /// orla de volta ao duro (`basicos::a_orla_em_pixeis_e_a_mesma_a_40_e_a_200`).
    pub(in crate::dab) orla_area: bool,
    /// A capsule under a Shape ramp composes its COLOUR over the same sub-dabs ([`super::cor_capsula`], lista 26 A9).
    pub(in crate::dab) cor_cap: bool,
    pub(in crate::dab) depth: f32,
    pub(in crate::dab) gran: f32,
    pub(in crate::dab) texture_alpha: bool,
    /// A taxa do tecto: a união da cápsula já leva a dos sub-dabs (`w·c_cap = 1 − Π(1 − c_cap·x_k)`).
    pub(in crate::dab) flow_do_tecto: f32,
    /// O tecto lê a cobertura como ÁREA (película com AA, ou a orla em píxeis).
    pub(in crate::dab) area: crate::stroke_cover::Area,
    /// B29.1: o texel largo recompõe a tela do pré-traço só sob a source-over pura (Mix, sem pigmento, sem rampa nem
    /// cor do Shape, sem Preserve Alpha) — a cor constante ao longo do traço é de quem dá o [`crate::TexelLargo`].
    pub(in crate::dab) recompoe: bool,
    /// A queda do dab com a dureza lida uma vez ([`crate::spec::QuedaDoDab`]).
    pub(in crate::dab) queda: crate::spec::QuedaDoDab,
    /// O branco em `Mix` sobre o `[0; 4]` da [`crate::PegadaLocal`] (sem pigmento espectral, sem Preserve Alpha): a
    /// `blend_over` dá `[1, 1, 1, a]` exactos (`a + 0 = a`, `a/a = 1`; o K–M pesa pelo alfa `0`) — [`branco_no_vazio`].
    pub(in crate::dab) branco_no_vazio: bool,
}

/// O que a [`crate::blend_vazio::blend_over_pigment_modo`] devolve com [`Invariantes::branco_no_vazio`].
#[inline(always)]
pub(in crate::dab) fn branco_no_vazio(a: f32) -> [f32; 4] {
    [1.0, 1.0, 1.0, a.clamp(0.0, 1.0)]
}

impl Invariantes {
    pub(in crate::dab) fn do_dab(ctx: &DabCtx, com_mascara: bool) -> Self {
        let c_cap = match (com_mascara, ctx.capsula) {
            (true, Some(cap)) if ctx.flow < 1.0 => {
                1.0 - (1.0 - ctx.flow.max(0.0)).det_powf(1.0 / cap.n_do_passo() as f32)
            }
            (true, _) => 1.0,
            (false, _) => ctx.coverage,
        };
        let orla_area = com_mascara && ctx.spec.borda_px > 0.0 && ctx.spec.leis_do_digital();
        Self {
            branco_no_vazio: ctx.vazio_local
                && !com_mascara
                && ctx.spec.blend == crate::BrushBlend::Mix
                && ctx.spec.color == [1.0; 3]
                && !ctx.spec.pigmento().espectral
                && !ctx.preserve_alpha
                && ctx.ramp.is_none(),
            queda: ctx.spec.queda_do_dab(),
            c_cap,
            orla_area,
            cor_cap: ctx.capsula.is_some() && ctx.ramp.is_some_and(|r| r.index == RampIndex::Shape),
            depth: ctx.spec.grain_depth(),
            gran: ctx.spec.effective_granulation(),
            texture_alpha: ctx
                .ramp
                .is_some_and(|r| matches!(r.alpha_mode, RampAlphaMode::TextureAlpha)),
            flow_do_tecto: if ctx.capsula.is_some() && !orla_area {
                c_cap
            } else {
                ctx.flow
            },
            area: crate::stroke_cover::Area::do_pincel(ctx.spec, ctx.film_aa.is_some(), orla_area),
            recompoe: com_mascara
                && ctx.spec.blend == crate::BrushBlend::Mix
                && ctx.spec.pigmento().mix <= 0.0
                && !ctx.spec.pigmento().espectral
                && ctx.ramp.is_none()
                && ctx.shape_cor.is_none()
                && !ctx.preserve_alpha,
        }
    }
}

/// Stamp the dab's pixels for the full-width row band `dst` whose first row is `band_y0` (so global
/// row `py` lives at local offset `(py - band_y0) * stride`). Returns whether any pixel was written.
/// Pure over disjoint bands — no shared mutable state — so bands run in parallel deterministically.
/// Três fases por texel: [`silhueta`] (o peso `w`), [`tinta`] (cor e `g`) e [`deposito`] (tecto, blend, encode).
///
/// `liso` (o mesmo `stride` do `dst`): a pegada SEM o grão, escrita da silhueta `w` antes do Grain, do dente e do
/// stencil — ao bit o que o MESMO kernel escreveria com a textura desligada e sem dente (cobertura `w·coverage`,
/// cor do pincel, o blend dele). Só sem `mask`/`forca`/rampa/Shape ([`crate::stamp_dab_textured_masked_po_com_liso`]).
/// Devolve `(tocou o dst, tocou o liso)`.
pub(super) fn stamp_band<M: crate::Tecto>(
    ctx: &DabCtx,
    dst: &mut [u8],
    mut liso: Option<&mut [u8]>,
    mut mask: Option<&mut [M]>,
    mut forca: Option<&mut [u16]>,
    band_y0: i64,
) -> (bool, bool) {
    let inv = Invariantes::do_dab(ctx, mask.is_some());
    let mrow = ctx.stride / 4;
    let mut touched = false;
    let mut liso_tocou = false;
    for r in 0..dst.len() / ctx.stride {
        let py = band_y0 + r as i64;
        let dy = (py as f32 + 0.5) - ctx.cy;
        let row = r * ctx.stride;
        for px in ctx.x0..ctx.x1 {
            let dx = (px as f32 + 0.5) - ctx.cx;
            let s = silhueta::em(ctx, &inv, px, py, dx, dy);
            let (w, maior_sub) = (s.w, s.maior_sub);
            // Skip pixels the silhouette already zeroes BEFORE the grain sample — the grain only
            // modulates where the dab paints, so sampling it there is pure waste (large Anchored).
            if w <= 0.0 {
                continue;
            }
            let col = (px - ctx.ox) as usize;
            if let Some(l) = liso.as_deref_mut() {
                liso_tocou |= super::liso::escreve(
                    ctx,
                    l,
                    row + col * 4,
                    w,
                    ctx.spec.blend,
                    inv.branco_no_vazio,
                );
            }
            let Some(t) = tinta::em(ctx, &inv, s, px, py) else {
                continue;
            };
            let onde = Onde {
                i: row + col * 4,
                mi: r * mrow + col,
            };
            touched |= deposito::pousa(
                ctx,
                &inv,
                dst,
                mask.as_deref_mut(),
                forca.as_deref_mut(),
                onde,
                w,
                maior_sub,
                t,
            );
        }
    }
    (touched, liso_tocou)
}

#[cfg(test)]
mod dente_tests;
#[cfg(test)]
mod oraculo_tests;

#[inline]
pub(crate) fn encode(v: f32) -> u8 {
    // Round-to-nearest, clamped. (No gamma here — the buffer is already in its native space.)
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

/// Stamp source `(color, sa)` (straight RGBA) onto destination `dst` within coverage `m ∈ [0,1]`:
/// a premultiplied lerp `out = dst·(1−m) + (color,sa)·m`. Unlike a colour blend this can LOWER the
/// destination alpha (where `sa < dst.a` and `m` is high) — that's how mode [`RampAlphaMode::TextureAlpha`]
/// makes parts of the sprite transparent. With `sa = 1` everywhere it reduces to ordinary opaque paint.
pub(crate) fn stamp_rgba(dst: [f32; 4], color: [f32; 3], sa: f32, m: f32) -> [f32; 4] {
    let sa = sa.clamp(0.0, 1.0);
    let m = m.clamp(0.0, 1.0);
    let da = dst[3];
    let out_a = da * (1.0 - m) + sa * m;
    if out_a <= 0.0 {
        return [0.0, 0.0, 0.0, 0.0];
    }
    // Un-premultiply: each channel is the premultiplied lerp divided by the out alpha.
    let mix = |b: f32, s: f32| (b * da * (1.0 - m) + s * sa * m) / out_a;
    [
        mix(dst[0], color[0]),
        mix(dst[1], color[1]),
        mix(dst[2], color[2]),
        out_a,
    ]
}

/// Sample a baked Color Ramp LUT at `s ∈ [0, 1]` (nearest entry — the 256-step LUT is already fine).
pub(crate) fn ramp_sample(lut: &[[f32; 4]], s: f32) -> [f32; 4] {
    if lut.is_empty() {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let n = lut.len();
    let idx = (s.clamp(0.0, 1.0) * (n as f32 - 1.0) + 0.5) as usize;
    lut[idx.min(n - 1)]
}
