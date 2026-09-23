//! **Smear as a displacement FIELD** — the transport that carries mass instead of a filament.
//!
//! The lift-and-blend kernels next door ([`crate::smear_dab`] and friends) are correct for ONE dab and
//! wrong for a stroke. Each dab re-reads the *result of the previous dab* and lerps toward it:
//!
//! ```text
//! dst ← dst + (src_one_step_back − dst)·w
//! ```
//!
//! so what survives `n` steps is `h·wⁿ` — a **product** over the dab list. The Smear route's spacing is
//! ~1 px, so a 170 px drag is ~170 steps. Exactly on the drag axis `t = 0`, `w = 1` and nothing decays;
//! six pixels off it `w ≈ 0.8` and `0.8¹⁵⁰ ≈ 0`. The stroke therefore delivers a **one-texel needle** and
//! no body at all — measured on the product (`push_look_probe` scene 13: across the trail at x=250,
//! `y194 h0.00 · y200 h3.73 · y206 h0.00`, with a brush of radius 10).
//!
//! This is the third time this line has met the same disease — the bow wave's bite
//! (`the_trench_is_a_fact_of_the_path_not_of_the_dab_spacing`) and the relief capsule were the first two.
//! The cure is always the same shape, and `warp/apply.rs` already states it: **a sequential accumulation
//! is sampling-dependent; an accumulated displacement applied ONCE to a frozen source is not.**
//!
//! So a dab here writes no pixels. It advances a per-stroke **backward map** — for each destination
//! texel, where in the frozen source its content came from — and the render resolves
//! `out[p] = pre[p − disp[p]]` exactly once, from the pixels frozen at pen-down. The same map moves the
//! colour and the impasto planes through one door (`warp/relief.rs`), which is what makes the pigment and
//! the body physically unable to disagree about where the paint went.
//!
//! ## Why the map is COMPOSED and not summed — the bucket brigade
//!
//! The obvious accumulation, `disp[i] += step · w(i)`, is wrong, and wrong in a way that is worth
//! recording because it looks right and it *measures* right on a short drag. A texel only accumulates
//! while the brush is over it, so the total displacement it can reach is bounded by roughly
//! *brush diameter × mean weight* — about 20 px for a 32 px knife, **no matter how far you drag**. Past
//! that the render samples a frozen source at a point that was never painted, and the trail simply stops.
//! Measured: colour and relief both fell to zero ~35 px past the ridge, on a 160 px drag.
//!
//! But a smear is a **relay**. Dab *k* hands its content to dab *k+1*, which hands it on again; content
//! near the axis stays under the moving brush and therefore travels the *whole stroke*, while content off
//! the axis is passed over once and left behind. That is a **composition of maps**, not a sum of offsets:
//!
//! ```text
//! φ_new(p) = φ_old(p − v(p))          v(p) = step · w(p)
//! ```
//!
//! *"what is at `p` now came from wherever the thing at `p − v` came from"* — semi-Lagrangian
//! backtracking, the standard formulation. In displacement form (`φ(p) = p − disp(p)`) that is
//!
//! ```text
//! disp_new(p) = v(p) + disp_old(p − v(p))
//! ```
//!
//! which relays without bound on the axis and settles to a finite partial offset off it — the trail with
//! mass, instead of either a needle or a stub.
//!
//! **This is still "accumulate, then apply ONCE to a frozen source", and that is the point.** What gets
//! resampled repeatedly is the *map* — a smooth, locally near-affine coordinate field, which bilinear
//! interpolation reproduces almost exactly. The IMAGE is resampled a single time, at the end. The kernel
//! this replaces resampled the image itself on every one of ~170 steps, which is why its off-axis mass
//! died as `wⁿ`. Resampling a coordinate field and resampling a picture are not the same operation, and
//! the difference between them is this whole module.
//!
//! ## Why this rides `walk_dab` and not a footprint of its own
//!
//! Because the Smear gets Tiling, Symmetry, the shape editors, pressure, Jitter, the **Shape**
//! silhouette and the **Grain** for free by hanging off the one dab list, and a warp session with its own
//! geometry inherits none of it. `walk_dab` is documented as *"the ONE place the swept body, the
//! silhouette, the Grain and the Selection are resolved"*; this module is simply its third rider, beside
//! the sculpt intensity and the plane fit. A dab shaped differently for the smear than for everything
//! else is how "Tiling doesn't work in Smear" gets born.

use crate::dab::DirtyRect;
use crate::height::HeightDab;
use crate::spec::BrushSpec;

/// ⛔⛔⛔ **O TECTO DO TRANSPORTE — CONSTRUÍDO, MEDIDO e NÃO ADOPTADO.** Ele fica porque é o
/// **instrumento da recusa**: sem ele, a medição que a rejeitou não é repetível.
///
/// ⚠️⚠️ **Ele cura a curva e MATA o que o dono exigiu.** Os dois gates que defendem o transporte
/// longo (`the_knife_carries_the_body_across_the_frontier_as_mass_not_a_filament` e
/// `the_smear_trail_is_a_fact_of_the_path_not_the_dab_spacing`) ficam VERMELHOS com o tecto
/// ligado: o rasto da faca morre a `2 R` da origem, que é exactamente a lei que o dono reprovou
/// **duas vezes** (*«as fronteiras não são vencidas. o relevo não é levado além. nada
/// resolvido»*). ⇒ *a conservação de tinta numa curva e o transporte que atravessa a tela são o
/// MESMO mecanismo, e sob esta lei não há um sem o outro.*
///
/// ## O que ele era, e porque parecia certo
///
/// O cabeçalho deste esfregão afirma uma lei que o código não cumpre:
///
/// ⛔⛔ **O número NÃO é escolhido: ele é o ALCANCE do próprio dab.** Um texel só é tocado por
/// dabs cuja pegada o cobre, e enquanto ele está coberto o cursor atravessa-o por, no máximo, um
/// **DIÂMETRO** — logo a tinta debaixo dele não pode ser levada mais do que `2 R` pelos dabs que
/// de facto lhe tocam. *É exactamente o que o doc do módulo diz: «o `disp` de um texel só cresce
/// enquanto o cursor está a menos de um raio dele».*
///
/// ⛔⛔⛔ **A composição violava-o, e por muito.** `D(p) = v + D(p − v)` deixa um texel **herdar**
/// o mapa de um vizinho atrás dele, que por sua vez herdou do vizinho atrás desse: a corrente
/// alcança arbitrariamente longe, e não só os dabs que tocaram o texel. Medido pela porta do
/// produto, num traço RECTO de `580 px`: `|disp|` máximo de **`568,58 px`**, que são **`9,5`
/// raios** contra o raio da lei.
///
/// ## Porque isso só se VÊ numa curva
///
/// Num traço recto e uniforme, levar tinta `568 px` ao longo do eixo é invisível — o traço é
/// igual a si mesmo ao longo dele. Numa CURVA não é: o traçado para trás deixa de acompanhar o
/// caminho e aterra **fora** do traço, onde não há tinta nenhuma, e o re-amostrar traz o vazio.
/// Medido num anel (`r = 215`, pincel `30`), a tinta que sobrevive ao esfregão:
///
/// | tecto | anel (curva) | recta (controlo) | deriva radial | raio de `p − D` (anel: `185..245`) |
/// |---|---|---|---|---|
/// | `∞` (o de ontem) | `84,4 %` | `99,9 %` | `43,72 px` | `205,9` |
/// | **`2` (a lei)** | **`99,7 %`** | `99,9 %` | **`6,38 px`** | **`212,6`** |
///
/// ⚠️ **E a recta fica onde estava, que é o que faz disto uma correcção e não uma mudança de
/// produto:** mesma contagem de texels (`36 084`), e a pior coluna do traço move-se `8` de
/// `10 990` — **`0,07 %`**.
///
/// ⚠️ **Com a dureza a `1` o defeito era MUITO maior** (`|disp| 383 px`, `51,2 %` de tinta), e é
/// por isso que ele não se vê numa fixtura de dureza `0`: ali a atenuação da orla já limitava a
/// corrente por acidente.
/// ## A varredura, que é de onde o número sai
///
/// | tecto (R) | anel guardado | `\|disp\|` máx | recta: pior coluna contra o de ontem |
/// |---|---|---|---|
/// | `0,5` | `100,0 %` | `15` | **`1,097 %`** ← já corta o transporte aprovado |
/// | `1,0` | `100,0 %` | `30` | `0,073 %` |
/// | `1,5` | `99,9 %` | `45` | `0,036 %` |
/// | **`2,0`** | **`99,7 %`** | `60` | `0,073 %` |
/// | `3,0` | `96,0 %` | `90` | `0,018 %` |
/// | `4,0` | `86,7 %` | `120` | `0,000 %` |
/// | `∞` | `84,4 %` | `320` | `0,000 %` |
///
/// ⭐ **O valor DERIVADO da geometria é também o JOELHO da curva medida**: `2` é o maior tecto
/// que deixa a curva acima de `99,5 %`, e descer abaixo dele começa a cortar a recta que o dono
/// aprovou. *Uma derivação e uma medição independentes a darem o mesmo número é a única forma
/// honesta de escrever um limite.*
pub const TECTO_MEDIDO_E_RECUSADO_EM_RAIOS: f32 = 2.0;

/// **O que o produto passa: SEM TECTO.** A corrente alcança o que alcançar.
///
/// ⛔⛔⛔ **E isso é uma ESCOLHA medida, não um esquecimento** — ver a recusa em
/// [`TECTO_MEDIDO_E_RECUSADO_EM_RAIOS`]. *A conservação de tinta numa curva e o transporte que
/// atravessa a tela são o MESMO mecanismo: não há um sem o outro sob esta lei.*
pub const SEM_TECTO: f32 = f32::INFINITY;

/// Accumulate ONE smear dab into the session's cumulative displacement map.
///
/// `disp` is canvas-sized (`width · height`), in **pixels**, and is the source of truth for the whole
/// stroke: the renderer resolves each texel as `bilinear(pre, p − disp[p])`. `step` is this dab's motion
/// (`to − from`) in canvas pixels — **not** rounded to whole texels the way the lift-and-blend kernel
/// rounds it, because a displacement is resampled bilinearly and has no reason to quantise. `mask` is the
/// Selection's per-texel coverage, folded per dab as it lands (never onto the running total — see
/// [`crate::sculpt::accumulate_dab_sculpt`], which explains why that compounds under a Feather).
///
/// Returns the touched rect, or `None` if the dab wrote nothing.
///
/// ## The fold is the SMEAR's fold, and that is deliberate
///
/// [`crate::sculpt::walk_dab`] folds `coverage × flow × strength`, because that is what the deposit and
/// the colour routes do. The Smear route has never folded **Flow** — its weight is `coverage × strength`
/// (`stamp_dabs_smear`: `amount = strength · d.coverage`). Whether Flow *ought* to mean something on a
/// knife is a real question, and it is not this fix's to answer: turning an inert slider live would
/// change every existing smear drawing for a reason nobody asked for. So the caller hands us a spec whose
/// `flow` is `1.0` and the fold reduces to exactly the one the route has always applied.
#[must_use]
pub fn accumulate_dab_smear(
    out: SmearOut<'_>,
    mov: Transporte,
    mask: Option<&[u8]>,
    width: u32,
    height: u32,
    spec: &BrushSpec,
    dab: &HeightDab<'_>,
) -> Option<DirtyRect> {
    let SmearOut { disp, scratch } = out;
    let Transporte {
        step,
        tecto_em_raios,
        arco,
    } = mov;
    let n = (width as usize) * (height as usize);
    if disp.len() < n {
        return None;
    }
    // A dab that does not move transports nothing. (The lift-and-blend kernel early-outs on the same
    // fact; here it also keeps a zero-length step from marking the rect dirty for no reason.)
    if step[0] == 0.0 && step[1] == 0.0 {
        return None;
    }
    // Pass 1 — resolve the footprint ONCE (the expensive part: falloff, Shape image, Grain, Selection)
    // and park `(index, weight)`. The composition needs the dab's rect before it can snapshot, and
    // walking the silhouette twice would double the dab's cost.
    scratch.pairs.clear();
    let pairs = &mut scratch.pairs;
    let rect = crate::sculpt::walk_dab(mask, width, height, spec, dab, |i, _dx, _dy, add| {
        pairs.push((i as u32, add));
    })?;

    // Snapshot the OLD map over the rect grown by the furthest this dab can backtrack. The update reads
    // `disp_old(p − v)` for neighbours of `p`, so reading and writing the same buffer in place would let
    // a texel updated earlier in the scan pollute one updated later — the very sequential dependence
    // this kernel exists to remove.
    // O alcance do retro-traçado. Num arco um texel na borda de fora da pegada roda por um braço
    // maior que o do centro, logo ele anda MAIS do que o passo — e a janela tem de o conter.
    let reach = match arco {
        None => step[0].abs().max(step[1].abs()),
        Some(a) => {
            let braco = (dab.center[0] - a.centro[0]).hypot(dab.center[1] - a.centro[1]);
            (braco + dab.radius) * a.dtheta.abs()
        }
    };
    let win = MapWindow::snapshot(
        &mut scratch.win,
        disp,
        width,
        height,
        (rect.x, rect.y, rect.w, rect.h),
        reach,
    );

    // Pass 2 — compose. `disp_new(p) = v(p) + disp_old(p − v(p))`.
    for &(i, add) in &scratch.pairs {
        let i = i as usize;
        let px = (i % width as usize) as f32;
        let py = (i / width as usize) as f32;
        // O passo de volta: recto quando o caminho não virou, e uma rotação em torno do centro
        // de curvatura quando virou. O peso `add` gradua os dois do mesmo modo — um texel na
        // orla do dab acompanha uma fracção do movimento, e a `add = 0` ele não se mexe.
        let atras = match arco {
            None => [px - step[0] * add, py - step[1] * add],
            Some(a) => {
                let (sn, cs) = (-a.dtheta * add).sin_cos();
                let (dx, dy) = (px - a.centro[0], py - a.centro[1]);
                [
                    a.centro[0] + cs * dx - sn * dy,
                    a.centro[1] + sn * dx + cs * dy,
                ]
            }
        };
        let v = [px - atras[0], py - atras[1]];
        let back = win.sample(atras[0], atras[1]);
        let mut d = [v[0] + back[0], v[1] + back[1]];
        // ⛔⛔ **O TECTO DO TRANSPORTE** — sem ele a herança `D(p) = v + D(p − v)` alcança muito
        // além dos dabs que tocaram o texel, e numa CURVA o traçado para trás sai do traço e o
        // re-amostrar traz o vazio. A derivação, a tabela e o que ele custa numa recta estão em
        // [`TECTO_MEDIDO_E_RECUSADO_EM_RAIOS`].
        //
        // ⭐ **Sem tecto não há conta** (2026-09-23): o produto corre com [`SEM_TECTO`], ou seja
        // `tecto = ∞`, e `m > ∞` é falso para todo `m` — finito, infinito ou `NaN`. O `hypot` era
        // pago POR TEXEL para uma comparação que nunca podia ser verdade: medido por amostragem no
        // build do produto (a pilha do dono no Composite Brush), `4,7 %` da thread principal. A
        // guarda é EXACTA pela mesma razão, logo a saída é byte a byte a de antes — e ela pergunta
        // `!= ∞` e não `is_finite()` de propósito: com `−∞` ou `NaN` o ramo corre como corria.
        let tecto = spec.radius_px * tecto_em_raios;
        if tecto != f32::INFINITY {
            let m = d[0].hypot(d[1]);
            if m > tecto {
                let k = tecto / m;
                d = [d[0] * k, d[1] * k];
            }
        }
        disp[i] = d;
    }
    Some(rect)
}

/// **O movimento deste dab: o passo e o TECTO do transporte** — o par que descreve para onde a
/// tinta vai e até onde ela pode ir, agrupado do mesmo modo que [`SmearOut`] agrupa as saídas.
///
/// ⚠️ **Eles viajam juntos de propósito.** O tecto só tem sentido em relação ao passo que o
/// alimenta, e separá-los deixaria um chamador a passar um sem o outro. O valor do produto é
/// [`SEM_TECTO`]; o [`TECTO_MEDIDO_E_RECUSADO_EM_RAIOS`] é o instrumento da recusa.
#[derive(Clone, Copy)]
pub struct Transporte {
    /// Quanto o cursor andou desde o dab anterior, em píxeis de tela.
    pub step: [f32; 2],
    /// O tecto do deslocamento acumulado, em RAIOS de pincel.
    pub tecto_em_raios: f32,
    /// **Por onde o caminho VIROU** — `None` numa recta, e aí o passo de volta é o de sempre.
    pub arco: Option<Arco>,
}

/// **A curva osculadora do caminho neste dab** — o círculo que os três últimos centros definem.
///
/// ⛔⛔⛔ **É isto que faz a tinta seguir a curva em vez de sair dela.** O traçado para trás do
/// esfregão anda um passo de cada vez, e um passo recto (`p − v`) é uma CORDA: ele sai do arco
/// por `|v|²/2r` a cada elo. Isso é invisível num elo e a corrente tem **centenas** deles — o
/// caminho de volta afasta-se do traço, o re-amostrar aterra onde não há tinta, e a tinta morre.
/// Medido num anel de `r = 215`: o ponto de origem aterra a **`205,9`** do centro em vez de
/// `215`, e a figura perde **`15,6 %`** da tinta.
///
/// Com o arco, o passo de volta é uma **ROTAÇÃO** em torno do centro de curvatura: ele segue o
/// arco por construção, e o erro por elo passa a ser o do modelo (o caminho ser localmente um
/// círculo), não o da corda.
///
/// ⚠️ **Numa recta ele é `None` e o caminho é BYTE-IDÊNTICO ao de sempre** — é isso que preserva
/// o transporte longo que o dono exigiu duas vezes: a corrente continua a andar tão longe
/// quanto os dabs andaram (`568 px` medidos num traço de `580`), só que agora pelo caminho
/// certo.
#[derive(Clone, Copy)]
pub struct Arco {
    /// O centro de curvatura, em píxeis de tela.
    pub centro: [f32; 2],
    /// O ângulo que o caminho virou entre o dab anterior e este, com sinal.
    pub dtheta: f32,
}

/// The map being advanced, plus its caller-owned scratch — bundled the way [`crate::sculpt::PlaneOut`]
/// bundles the plane family's outputs, so the kernel keeps one output parameter instead of two.
pub struct SmearOut<'a> {
    /// The session's cumulative backward map, canvas-sized.
    pub disp: &'a mut [[f32; 2]],
    /// Reused per-dab scratch (see [`SmearScratch`]).
    pub scratch: &'a mut SmearScratch,
}

/// Caller-owned scratch so a hot stroke allocates nothing: the resolved `(index, weight)` pairs of one
/// dab, and the snapshot of the old map over that dab's window.
#[derive(Default)]
pub struct SmearScratch {
    pairs: Vec<(u32, f32)>,
    win: Vec<[f32; 2]>,
}

/// **A janela congelada do mapa antigo** — o pedaço de `disp` que uma composição pode ler, e a
/// amostragem bilinear dele.
///
/// ⚠️ **Por que ela é `pub`:** a composição semi-lagrangiana `disp_new(p) = v(p) + disp_old(p − v(p))`
/// tem DOIS consumidores nesta casa — o Smear (aqui, onde `v` é o passo escalado pelo peso do dab) e o
/// **Reshape** (`ph2d-tool-painter::warp::apply`, onde `v` é o campo do dab, [ADR-0157]). O que difere é
/// quem PRODUZ `v`; o que reamostra o mapa é o mesmo, e uma segunda cópia divergiria em silêncio no
/// único lugar onde ninguém lê um número.
///
/// [ADR-0157]: ../../../../docs/architecture/decisions/0157-liquify-is-an-authored-dab-list-cooked-on-the-device-never-a-stored-dense-field.md
pub struct MapWindow<'a> {
    win: &'a [[f32; 2]],
    ww: usize,
    wh: usize,
    ox: f32,
    oy: f32,
}

impl<'a> MapWindow<'a> {
    /// Copia `rect` de `disp` crescido por `reach` (o quanto esta atualização consegue retro-traçar) para
    /// o `buf` do chamador — que é de quem chama para que um traço quente não aloque por dab.
    ///
    /// ⚠️ `reach` entra em px CONTÍNUOS e é arredondado para cima aqui: a decisão *"quanto de margem uma
    /// leitura bilinear precisa"* é uma só, e deixá-la no chamador é como o segundo chamador nasce com
    /// uma margem diferente.
    pub fn snapshot(
        buf: &'a mut Vec<[f32; 2]>,
        disp: &[[f32; 2]],
        width: u32,
        height: u32,
        rect: (u32, u32, u32, u32),
        reach: f32,
    ) -> Self {
        let grow = i64::from(reach.max(0.0).ceil() as i32) + 2;
        let (wi, hi) = (i64::from(width), i64::from(height));
        let (rx, ry, rw, rh) = rect;
        let wx0 = (i64::from(rx) - grow).max(0);
        let wy0 = (i64::from(ry) - grow).max(0);
        let wx1 = (i64::from(rx) + i64::from(rw) + grow).min(wi);
        let wy1 = (i64::from(ry) + i64::from(rh) + grow).min(hi);
        let ww = (wx1 - wx0).max(0) as usize;
        let wh = (wy1 - wy0).max(0) as usize;
        buf.clear();
        buf.reserve(ww * wh);
        for y in wy0..wy1 {
            let row = (y * wi) as usize;
            buf.extend_from_slice(&disp[row + wx0 as usize..row + wx1 as usize]);
        }
        Self {
            win: buf,
            ww,
            wh,
            ox: wx0 as f32,
            oy: wy0 as f32,
        }
    }

    /// Bilinear-sample the windowed map snapshot at canvas coords `(x, y)`, clamping to the window's edge.
    ///
    /// Clamping is safe because the window is grown by the dab's maximum backtrack, so a clamp can only
    /// bite at the true canvas edge — where extending the map is exactly the policy the pixel and relief
    /// samplers already use (`bilinear_clamped`: extend, never wrap).
    #[inline]
    #[must_use]
    pub fn sample(&self, x: f32, y: f32) -> [f32; 2] {
        sample_window(self.win, self.ww, self.wh, self.ox, self.oy, x, y)
    }
}

#[inline]
fn sample_window(
    win: &[[f32; 2]],
    ww: usize,
    wh: usize,
    ox: f32,
    oy: f32,
    x: f32,
    y: f32,
) -> [f32; 2] {
    if ww == 0 || wh == 0 {
        return [0.0, 0.0];
    }
    let lx = x - ox;
    let ly = y - oy;
    let x0f = lx.floor();
    let y0f = ly.floor();
    let fx = lx - x0f;
    let fy = ly - y0f;
    let x0 = (x0f as i64).clamp(0, ww as i64 - 1) as usize;
    let y0 = (y0f as i64).clamp(0, wh as i64 - 1) as usize;
    let x1 = (x0 + 1).min(ww - 1);
    let y1 = (y0 + 1).min(wh - 1);
    let at = |xi: usize, yi: usize| win[yi * ww + xi];
    let (a, b, c, d) = (at(x0, y0), at(x1, y0), at(x0, y1), at(x1, y1));
    let top = [a[0] + (b[0] - a[0]) * fx, a[1] + (b[1] - a[1]) * fx];
    let bot = [c[0] + (d[0] - c[0]) * fx, c[1] + (d[1] - c[1]) * fx];
    [
        top[0] + (bot[0] - top[0]) * fy,
        top[1] + (bot[1] - top[1]) * fy,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::height::HeightDab;

    fn dab_at(center: [f32; 2], radius: f32) -> HeightDab<'static> {
        HeightDab {
            center,
            radius,
            coverage: 1.0,
            footprint: crate::footprint::FootprintDeform::identity(),
            prev_center: None,
            shape: None,
            grain: None,
            grain_image: None,
        }
    }

    fn spec(radius: f32) -> BrushSpec {
        BrushSpec {
            radius_px: radius,
            flow: 1.0,
            strength: 1.0,
            ..Default::default()
        }
    }

    /// ⛔⛔ **O INSTRUMENTO DA RECUSA GRAMPEIA, E O PRODUTO NÃO** — as duas metades, porque
    /// cada uma sozinha mente.
    ///
    /// O cabeçalho deste módulo afirma que *«o `disp` de um texel só cresce enquanto o cursor
    /// está a menos de um raio dele»*, e a composição `D(p) = v + D(p − v)` **não o cumpre**: um
    /// texel herda o mapa de quem está atrás, e a corrente alcança arbitrariamente longe. O tecto
    /// que honra a frase existe e foi **medido e recusado** — ver
    /// [`TECTO_MEDIDO_E_RECUSADO_EM_RAIOS`], porque ele mata o transporte longo que o dono exigiu.
    ///
    /// ⚠️ Este gate não escolhe: ele afirma que o instrumento **funciona** (com o tecto, o
    /// transporte pára no alcance do dab) e que o produto **não o usa** (sem ele, a corrente
    /// alcança muito mais). *Sem a segunda metade, alguém leria a recusa como já aplicada.*
    ///
    /// **Mutações que sangram:** apagar o grampo · `let k = 1.0` · trocar o `>` por `<`.
    #[test]
    fn o_instrumento_da_recusa_grampeia_e_o_produto_nao() {
        let (w, h) = (64u32, 64u32);
        let n = (w * h) as usize;
        let raio = 8.0f32;
        let s = spec(raio);
        let anda = |tecto: f32| -> f32 {
            let mut sc = SmearScratch::default();
            let mut disp = vec![[0.0f32; 2]; n];
            // Um traço LONGO de propósito: 48 passos de 1 px, seis vezes o diâmetro do dab.
            for k in 0..48u32 {
                let x = 8.0 + k as f32;
                let _ = accumulate_dab_smear(
                    SmearOut {
                        disp: &mut disp,
                        scratch: &mut sc,
                    },
                    Transporte {
                        step: [1.0, 0.0],
                        tecto_em_raios: tecto,
                        arco: None,
                    },
                    None,
                    w,
                    h,
                    &s,
                    &dab_at([x, 32.0], raio),
                );
            }
            disp.iter().map(|d| d[0].hypot(d[1])).fold(0.0, f32::max)
        };
        let com = anda(TECTO_MEDIDO_E_RECUSADO_EM_RAIOS);
        let sem = anda(SEM_TECTO);
        let alcance = 2.0 * raio;
        assert!(
            com <= alcance + 1e-3,
            "o transporte passou o alcance do dab: {com:.2} px contra {alcance:.2}"
        );
        // A metade do PRODUTO: ele passa `SEM_TECTO`, logo a corrente tem de disparar. Isto é
        // ao mesmo tempo o controlo positivo da fixtura e a afirmação de que a recusa não foi
        // aplicada às escondidas.
        assert!(
            sem > 2.0 * alcance,
            "o produto passou a grampear (ou a fixtura não contém o fenómeno): sem tecto a \
             corrente só chegou a {sem:.2} px, contra o alcance de {alcance:.2}"
        );
    }

    /// **The law the whole fix rests on: transport is a SUM, so it does not depend on how finely the
    /// motion was sampled.** Walk the same 32 px with 32 one-pixel steps and with 8 four-pixel steps —
    /// the displacement that lands on the axis must be the same distance, not a different one.
    ///
    /// The kernel this replaces fails exactly here: `h·wⁿ` depends on `n`.
    ///
    /// **Mutation that must bleed:** make the sink `disp[i] = step·add` (assign, not add) — the coarse
    /// walk then reports one step's worth and the two disagree by 4×.
    #[test]
    fn transport_is_a_sum_so_the_dab_spacing_cannot_change_it() {
        let (w, h) = (64u32, 64u32);
        let n = (w * h) as usize;
        let s = spec(8.0);
        let walk = |stride: f32| {
            let mut sc = SmearScratch::default();
            let mut disp = vec![[0.0f32; 2]; n];
            let steps = (32.0 / stride) as u32;
            for k in 0..steps {
                let x = 16.0 + stride * (k as f32 + 1.0);
                let _ = accumulate_dab_smear(
                    SmearOut {
                        disp: &mut disp,
                        scratch: &mut sc,
                    },
                    Transporte {
                        step: [stride, 0.0],
                        tecto_em_raios: TECTO_MEDIDO_E_RECUSADO_EM_RAIOS,
                        arco: None,
                    },
                    None,
                    w,
                    h,
                    &s,
                    &dab_at([x, 32.0], 8.0),
                );
            }
            disp
        };
        let fine = walk(1.0);
        let coarse = walk(4.0);
        // On the axis, mid-trail: the brush centre passed over this texel, so the weight is ~1 and the
        // displacement should be ~the distance travelled while it was under the brush.
        let i = (32 * w + 32) as usize;
        assert!(
            fine[i][0] > 8.0,
            "the fine walk must actually transport (got {})",
            fine[i][0]
        );
        let ratio = coarse[i][0] / fine[i][0];
        assert!(
            (0.8..1.25).contains(&ratio),
            "same 32 px of motion, 4× the sampling: fine displaced {} px, coarse {} px (ratio {ratio}). \
             Transport that depends on the spacing is the product law this kernel exists to replace",
            fine[i][0],
            coarse[i][0]
        );
    }

    /// **The trail has the brush's WIDTH.** Off the drag axis the falloff makes the displacement
    /// smaller, but it must not make it *zero* — that is the filament. Across the trail, the band of
    /// texels that moved at all should span roughly the brush's diameter.
    ///
    /// **Mutation that must bleed:** restore a per-step lerp toward the previous result — the off-axis
    /// column collapses to the centre texel.
    #[test]
    fn the_displaced_band_is_as_wide_as_the_brush() {
        let (w, h) = (96u32, 96u32);
        let n = (w * h) as usize;
        let s = spec(10.0);
        let mut sc = SmearScratch::default();
        let mut disp = vec![[0.0f32; 2]; n];
        for k in 0..48u32 {
            let x = 20.0 + k as f32;
            let _ = accumulate_dab_smear(
                SmearOut {
                    disp: &mut disp,
                    scratch: &mut sc,
                },
                Transporte {
                    step: [1.0, 0.0],
                    tecto_em_raios: TECTO_MEDIDO_E_RECUSADO_EM_RAIOS,
                    arco: None,
                },
                None,
                w,
                h,
                &s,
                &dab_at([x, 48.0], 10.0),
            );
        }
        // Cut across the trail at mid-drag and count what moved by a visible amount.
        let moved = (0..h)
            .filter(|&y| disp[(y * w + 44) as usize][0] > 0.5)
            .count();
        assert!(
            moved >= 14,
            "the knife is 20 px across but only {moved} px of the cross-section moved — the transport \
             narrowed to a filament"
        );
    }

    /// A dab that does not move writes nothing at all — and, in particular, does not dirty a rect.
    #[test]
    fn a_still_dab_transports_nothing() {
        let (w, h) = (32u32, 32u32);
        let mut disp = vec![[0.0f32; 2]; (w * h) as usize];
        assert!(
            accumulate_dab_smear(
                SmearOut {
                    disp: &mut disp,
                    scratch: &mut SmearScratch::default()
                },
                Transporte {
                    step: [0.0, 0.0],
                    tecto_em_raios: TECTO_MEDIDO_E_RECUSADO_EM_RAIOS,
                    arco: None,
                },
                None,
                w,
                h,
                &spec(6.0),
                &dab_at([16.0, 16.0], 6.0)
            )
            .is_none()
        );
        assert!(disp.iter().all(|d| *d == [0.0, 0.0]));
    }

    /// The Selection attenuates the transport where it is partial — the knife cannot drag paint out of a
    /// region the artist masked off.
    #[test]
    fn the_selection_attenuates_the_transport() {
        let (w, h) = (48u32, 48u32);
        let n = (w * h) as usize;
        let s = spec(8.0);
        // Left half fully selected, right half not at all.
        let mut mask = vec![0u8; n];
        for y in 0..h {
            for x in 0..(w / 2) {
                mask[(y * w + x) as usize] = 255;
            }
        }
        let mut sc = SmearScratch::default();
        let mut disp = vec![[0.0f32; 2]; n];
        for k in 0..24u32 {
            let _ = accumulate_dab_smear(
                SmearOut {
                    disp: &mut disp,
                    scratch: &mut sc,
                },
                Transporte {
                    step: [1.0, 0.0],
                    tecto_em_raios: TECTO_MEDIDO_E_RECUSADO_EM_RAIOS,
                    arco: None,
                },
                Some(&mask),
                w,
                h,
                &s,
                &dab_at([12.0 + k as f32, 24.0], 8.0),
            );
        }
        let inside = disp[(24 * w + 12) as usize][0];
        let outside = disp[(24 * w + 40) as usize][0];
        assert!(
            inside > 0.5,
            "inside the selection the knife drags ({inside})"
        );
        assert_eq!(
            outside, 0.0,
            "outside the selection nothing moves (got {outside})"
        );
    }
}
