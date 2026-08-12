//! **Como um dab de altura vira TEXELS** — o laço, as bandas e o contexto que elas compartilham
//! (filho do [`super::height`], cortado pelo teto de LOC do workspace).
//!
//! O irmão diz **o que a altura É** (`derive_height`, os ingredientes, os planos); este diz **quem
//! percorre a pegada e escreve neles**. É o mesmo corte que o [`crate::dab::bands`] fez do lado da
//! cor — *"this file is how a dab becomes pixels, not what a dab is"* — e ele nasceu pela mesma
//! razão: as bandas.

use super::{DepositGate, HeightDab, HeightFields, NO_GRAIN, derive_height, sweep_residual};

/// Tudo o que o laço de texel precisa saber sobre ESTE dab, resolvido uma vez — e o que uma banda
/// recebe por referência compartilhada.
///
/// `Sync` pela mesma razão que o [`crate::dab::DabCtx`] do depósito de cor é: só referências a dados
/// imutáveis e escalares `Copy`.
pub(super) struct Walk<'a, 'b> {
    pub(super) spec: &'a crate::BrushSpec,
    pub(super) dab: &'a HeightDab<'b>,
    pub(super) coverage: f32,
    pub(super) radius: f32,
    pub(super) inv_radius: f32,
    pub(super) cx: f32,
    pub(super) cy: f32,
    pub(super) sweep: Option<([f32; 2], f32)>,
    /// **O passo do arco** deste dab, já dividido pelo normalizador — `Some` só no modo acumulativo
    /// ([`crate::height::accum_step`]). `None` é o envelope `max` que sempre shipou.
    pub(super) step: Option<f32>,
    pub(super) film_aa: Option<crate::height_film::FilmAa>,
    pub(super) lut: Option<crate::height_film::FilmLutPlan<'a>>,
    pub(super) ablate: u32,
    pub(super) grain_active: bool,
    pub(super) x0: i64,
    pub(super) x1: i64,
    pub(super) gate: Option<DepositGate<'a>>,
    pub(super) width: usize,
}

/// Percorre as linhas `[y0, y1)` do dab — **serial** numa pegada pequena, dividido em **bandas de
/// linhas disjuntas** numa grande, pela MESMA porta que o depósito de cor usa
/// ([`crate::dab::band_count`]).
///
/// ## Por que isto não move um byte
///
/// As bandas são linhas **disjuntas** e cada texel é escrito por exactamente uma delas; os cinco
/// planos são indexados por `i` e nenhum texel lê o vizinho. Mudar o número de bandas muda **quem
/// avalia a linha, nunca o que ela diz** — a mesma invariante que o [`crate::dab::band_count`]
/// declara para o depósito, e a razão de o número de bandas não precisar de gate próprio.
///
/// ## ⚠️ Com a MORDIDA do bow wave a rota é serial, e é uma decisão
///
/// A mordida acumula `PushBite::displaced`, um **escalar em `f32`** somado texel a texel. Somas
/// parciais por banda dariam outra ordem de adição, logo outros bits — e o desenho da onda de proa é
/// coisa que o Enio aprovou olhando. O regime que o report de 2026-08-10 nomeia (Digital + Relief da
/// Shape) tem `push == 0`, logo `bite == None`: **a cura cobre exactamente o caso reportado e deixa o
/// impasto com mordida byte-idêntico**, sem uma redução para ninguém auditar.
///
/// ## ⚠️ E o PISO é o do depósito de COR, o que é conservador — de propósito
///
/// O [`crate::dab::PARALLEL_MIN_AREA`] é derivado de uma visita de **13 ns** (a do kernel de cor).
/// Uma visita deste laço custa **26-58 ns** (medido em `measure_relief_systems`), então uma thread se
/// paga aqui com ~3× MENOS visitas: usar o piso do vizinho manda para a rota serial um trabalho que
/// já pagaria. É o lado seguro do erro, e o número que o corrigiria tem de sair de uma medição deste
/// kernel, não de uma segunda cópia do raciocínio.
pub(super) fn walk_dab_rows(
    w: &Walk<'_, '_>,
    fields: &mut HeightFields<'_>,
    y0: i64,
    y1: i64,
    bite: Option<&mut crate::height_push::PushBite<'_>>,
) -> bool {
    let (lo, hi) = ((y0 as usize) * w.width, (y1 as usize) * w.width);
    let rows = (y1 - y0) as usize;
    // ⚠️ **A mordida decide a rota ANTES de qualquer conta, e o `if let` é o que torna o descarte
    // INEXPRIMÍVEL:** ela é movida aqui, então nenhuma linha abaixo consegue alcançá-la. Escrito como
    // uma condição no `threads` (a 1ª forma), tirar a condição **compilava** e a rota em banda passava
    // `None` a cada banda — a onda de proa desaparecia em silêncio, com a suíte inteira verde menos
    // um gate. O compilador é a barreira mais barata que existe para isso.
    if let Some(b) = bite {
        return walk_band(w, whole(fields, lo, hi), y0, Some(b));
    }
    let area = rows * ((w.x1 - w.x0).max(0) as usize);
    let threads = if w.ablate & crate::ablate::SERIAL != 0 {
        1
    } else {
        crate::dab::band_count(area, rows, crate::dab::PARALLEL_MIN_AREA)
    };
    if threads <= 1 {
        return walk_band(w, whole(fields, lo, hi), y0, None);
    }
    let span = rows.div_ceil(threads) * w.width;
    let bands: Vec<_> = fields.height[lo..hi]
        .chunks_mut(span)
        .zip(fields.paint[lo..hi].chunks_mut(span))
        .zip(fields.grain[lo..hi].chunks_mut(span))
        .zip(fields.film[lo..hi].chunks_mut(span))
        .zip(fields.radius[lo..hi].chunks_mut(span))
        .enumerate()
        .map(|(bi, ((((h, p), g), f), r))| {
            (
                Planes {
                    height: h,
                    paint: p,
                    grain: g,
                    film: f,
                    radius: r,
                },
                y0 + (bi * (span / w.width)) as i64,
            )
        })
        .collect();
    std::thread::scope(|s| {
        bands
            .into_iter()
            .map(|(planes, band_y0)| s.spawn(move || band_out(walk_band(w, planes, band_y0, None))))
            // Coletado ANTES do fold para que TODA banda seja joinada (um `fold` preguiçoso com
            // curto-circuito deixaria thread pendurada) — a mesma nota do `parallel_band_stamp`.
            .collect::<Vec<_>>()
            .into_iter()
            .fold(false, |acc, h| band_join(acc, h.join().unwrap_or_default()))
    })
}

/// O que UMA banda devolve. Em produção é só *"tocou?"*; sob teste ela carrega junto os contadores da
/// LUT que acumulou na PRÓPRIA thread, porque um contador por-thread não atravessa uma thread — ver
/// [`crate::height_film_lut::add_lut_counts`], que é onde a premissa está escrita.
#[cfg(test)]
type BandOut = (bool, (usize, usize));
/// Ver [`BandOut`] sob `cfg(test)`.
#[cfg(not(test))]
type BandOut = bool;

#[cfg(test)]
fn band_out(touched: bool) -> BandOut {
    (touched, crate::height_film_lut::take_lut_counts())
}
#[cfg(not(test))]
fn band_out(touched: bool) -> BandOut {
    touched
}

#[cfg(test)]
fn band_join(acc: bool, out: BandOut) -> bool {
    crate::height_film_lut::add_lut_counts(out.1);
    acc | out.0
}
#[cfg(not(test))]
fn band_join(acc: bool, out: BandOut) -> bool {
    acc | out
}

/// Os cinco planos inteiros como UMA banda que cobre `[lo, hi)` — a rota serial.
fn whole<'a>(fields: &'a mut HeightFields<'_>, lo: usize, hi: usize) -> Planes<'a> {
    Planes {
        height: &mut fields.height[lo..hi],
        paint: &mut fields.paint[lo..hi],
        grain: &mut fields.grain[lo..hi],
        film: &mut fields.film[lo..hi],
        radius: &mut fields.radius[lo..hi],
    }
}

/// As cinco fatias que UMA banda possui — as linhas `[band_y0, band_y0 + len/width)` de cada plano.
struct Planes<'a> {
    height: &'a mut [f32],
    paint: &'a mut [f32],
    grain: &'a mut [u8],
    film: &'a mut [u8],
    radius: &'a mut [f32],
}

/// O laço de texel de UMA banda. `planes` são as linhas a partir de `band_y0`, então o índice dos
/// planos é **local** (`(py − band_y0)·width + px`) enquanto o gate — que é só leitura e pode ser mais
/// curto que a tela — segue lido no índice **global**, como sempre foi.
fn walk_band(
    wk: &Walk<'_, '_>,
    planes: Planes<'_>,
    band_y0: i64,
    mut bite: Option<&mut crate::height_push::PushBite<'_>>,
) -> bool {
    let (spec, dab) = (wk.spec, wk.dab);
    let (coverage, radius, inv_radius) = (wk.coverage, wk.radius, wk.inv_radius);
    let (cx, cy, sweep, ablate) = (wk.cx, wk.cy, wk.sweep, wk.ablate);
    let y1 = band_y0 + (planes.height.len() / wk.width) as i64;
    let mut touched = false;
    for py in band_y0..y1 {
        let dy = (py as f32 + 0.5) - cy;
        for px in wk.x0..wk.x1 {
            let dx = (px as f32 + 0.5) - cx;
            let (rx, ry) = sweep_residual(dx, dy, sweep);
            // `wv` is the DEFORMED residual and `t` its length — byte-identical to `falloff_t`, which
            // is exactly `apply` then the same `sqrt`. The vector is kept because the LUT's expansion
            // lives in that space (see [`crate::height_film::FilmAa::film_at_lut`]).
            let wv = dab.footprint.apply([rx * inv_radius, ry * inv_radius]);
            let t = (wv[0] * wv[0] + wv[1] * wv[1]).sqrt();
            let w = if ablate & crate::ablate::SILHOUETTE != 0 {
                f32::from(t < 1.0) // MESMO suporte, sem falloff/Shape/mascara -- so a sonda arma isto
            } else {
                crate::dab::silhouette_at(spec, dab.shape, t, px, py, dab.center, radius)
            };
            // The film at this texel: single-sample `film_of` (byte-identical old path), or the
            // fractional area coverage under Smooth Edges — the SAME fraction `dab.rs` gives the
            // pigment (same door, same grid, the caller's own swept-silhouette chain).
            let film = if ablate & crate::ablate::FILM_AA != 0 {
                crate::height_film::film_of(w) // o ramo `None` do proprio kernel
            } else {
                match &wk.film_aa {
                    Some(aa) => aa.film_at_planned(
                        wk.lut.as_ref(),
                        t,
                        wv,
                        [dx, dy],
                        || w,
                        |ox, oy| {
                            let (rx2, ry2) = sweep_residual(dx + ox, dy + oy, sweep);
                            spec.falloff_weight(
                                dab.footprint.falloff_t(rx2 * inv_radius, ry2 * inv_radius),
                            )
                        },
                    ),
                    None => crate::height_film::film_of(w),
                }
            };
            // A texel wholly outside silhouette AND film lays nothing (with AA a rim texel can carry
            // fractional film while its CENTRE silhouette is already 0 — it must not be skipped).
            if w <= 0.0 && film <= 0.0 {
                continue;
            }
            // ⚠️ **DOIS índices, e a distinção é o que torna a banda correta:** os cinco planos que
            // esta banda POSSUI começam na linha `band_y0`, então o índice deles é local; o gate e a
            // mordida são planos da TELA inteira (só-leitura o primeiro, serial o segundo), então o
            // índice deles é global. Na rota serial `band_y0 == y0` e os dois descrevem o mesmo texel
            // do mesmo jeito — é por isso que a rota antiga podia ter um só.
            let gi = (py as usize) * wk.width + px as usize;
            let i = gi - (band_y0 as usize) * wk.width;
            // **O gate do depósito** — quanto deste dab pousa AQUI. Sem gate é `1.0` exato, e
            // `x * 1.0 == x`, então o kernel de um documento sem máscara é byte-idêntico.
            let k = wk.gate.map_or(1.0, |g| g.factor_at(gi));
            if k <= 0.0 {
                continue; // texel congelado: nem filme, nem carga, nem mordida do Push
            }
            // The **film's** envelope, taken FIRST and on its own: the light's coverage is a different
            // function of the dab than the relief's ingredient is, so it cannot ride the same winner.
            // `coverage · film` is exactly the old `solid_paint(w, coverage)` when `film = film_of(w)`.
            let fq = ((coverage * film * k).clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            if fq > planes.film[i] {
                planes.film[i] = fq;
                touched = true;
            }
            // The **stroke envelope, taken on the PAINT** — the dab that laid the most paint at this
            // pixel owns it. One pass of a loaded brush leaves one thickness (a second pass over the
            // same line does not stack a staircase); separate strokes DO add, at stroke end.
            //
            // Enveloping the paint rather than the height is what makes every knob live: the winner is
            // then chosen by a quantity that no setting can change, so re-deriving the relief at a new
            // Body / Source / Depth cannot silently re-shuffle which dab shaped which pixel.
            if ablate & crate::ablate::TAIL != 0 {
                continue; // sem grain, sem mordida, sem as quatro escritas, sem derive_height
            }
            // **As DUAS leis de depósito**, e a de cima é a que sempre shipou.
            //
            // * `None` — o **ENVELOPE**: o dab que pousou mais tinta é dono do texel. Uma passada de
            //   um pincel carregado deixa UMA espessura; passar de novo no mesmo traço não empilha.
            // * `Some(step)` — a **INTEGRAL DE LINHA** do Accumulate (doc 35 §6/D3): a carga é
            //   `Σ perfil·Δs / NORM`, então esfregar constrói. Ela é função do **CAMINHO**, não de
            //   quão fino o motor amostrou o caminho — dobrar a densidade de dabs dobra a contagem e
            //   divide `Δs` por dois, e a soma converge para a mesma integral (o invariante I1 que
            //   esta linha pagou três vezes). E, por ser função do caminho, um re-carimbo do shape
            //   editor devolve o mesmo número (I2).
            let m = match wk.step {
                None => {
                    let m = (w * coverage * k).clamp(0.0, 1.0);
                    if m <= planes.paint[i] {
                        continue;
                    }
                    m
                }
                Some(step) => {
                    let add = (w * coverage * k).max(0.0) * step;
                    if add <= 0.0 {
                        continue;
                    }
                    // Sem clamp: é a carga passar de 1 que dá a espessura, e o único leitor que a vê
                    // crua é o `derive_height`, que a estende linearmente (varredura de consumidores
                    // no doc 35 §6/D3).
                    planes.paint[i] + add
                }
            };
            let g = if let Some(b) = dab.grain {
                crate::dab::grain_at(spec, b, dab.grain_image, px, py, dab.center, radius)
                    .clamp(0.0, 1.0)
            } else {
                1.0
            };
            let gq = if wk.grain_active {
                (g * 255.0 + 0.5) as u8
            } else {
                NO_GRAIN
            };
            // **Volume conservation, riding along** (`crate::height_push`): the ground this dab's advance
            // covers is ground it SHOVES, and it is taken here — inside the walk that already knows `m`,
            // `paint[i]` and the silhouette. Doing it in a kernel of its own meant evaluating
            // `silhouette_at` twice per texel, and that alone put the impasto cost at 5.0 ms/move, over
            // budget, on every stroke. Three operations, folded into a loop that was already running.
            if let Some(b) = bite.as_deref_mut() {
                // The bite takes from the ground AND from the stroke's own accumulated plane — the
                // bow wave the previous dab banked ahead is picked up here and shoved on (see
                // `forward_weight`). `(g + p)` is what actually stands at the texel right now, and
                // `.max(0)` guards float fuzz.
                //
                // **The share is the increment over the REMAINING HEADROOM, not the raw increment** —
                // and that is what makes the trench a fact of the PATH instead of a fact of the dab
                // spacing. With the raw `Δm`, `q = g + p` evolves as `q ← q·(1 − Δm)`, so the total
                // bite is `g·(1 − Π(1 − Δm_k))`: a PRODUCT over the increments, which depends on how
                // many steps the envelope was reached in and on each texel's phase against the dab
                // grid. A soft falloff hides it (its `Δm` are small and even); `Sphere`'s silhouette
                // has a VERTICAL tangent at the rim, so `Δm` jumps hard, the phase term explodes, and
                // the trench floor comes out RIPPLED at exactly the dab period — the coil Enio's smoke
                // caught (2026-07-15). Normalising by `(1 − paint)` telescopes the product exactly:
                // `Π (1 − Δm/(1 − m_{k−1})) = Π (1 − m_k)/(1 − m_{k−1}) = (1 − m_final)`, so the bite
                // lands on `g·m_final` — a pure function of the envelope, at ANY spacing, in ANY
                // order. It is also the honest law: the brush shoves the ground in proportion to how
                // much it ended up covering the texel, and at full coverage it takes all of it and
                // never more (the self-limiting guarantee the raw form gave, now exact).
                let head = 1.0 - planes.paint[i];
                if head > 1e-6 {
                    let share = ((m - planes.paint[i]) / head).clamp(0.0, 1.0);
                    let take = (b.ground[gi] + b.plane[gi]).max(0.0) * share;
                    if take != 0.0 {
                        b.plane[gi] -= take;
                        b.displaced += take;
                    }
                }
            }
            planes.paint[i] = m;
            planes.grain[i] = gq;
            planes.radius[i] = spec.radius_px;
            // Derived from the STORED (quantised) grain, so the buffer and the re-derivation always
            // agree to the last bit — a live edit can never make the relief jump.
            planes.height[i] = derive_height(spec, m, f32::from(gq) / 255.0);
            touched = true;
        }
    }
    touched
}

#[cfg(test)]
#[path = "height_walk_tests.rs"]
mod tests;
