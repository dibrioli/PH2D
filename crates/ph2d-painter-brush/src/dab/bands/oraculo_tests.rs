//! O ORÁCULO do corte B52.R25: a `stamp_band` de antes do corte, VERBATIM (só o nome e os caminhos `super::`),
//! e a matriz que a compara ao byte com a de agora (silhueta · tinta · depósito, invariantes içadas).

use super::*;

pub(super) fn stamp_band_antiga(
    ctx: &DabCtx,
    dst: &mut [u8],
    mut liso: Option<&mut [u8]>,
    mut mask: Option<&mut [u8]>,
    mut forca: Option<&mut [u16]>,
    band_y0: i64,
) -> (bool, bool) {
    let blend = ctx.spec.blend;
    let mut touched = false;
    let mut liso_tocou = false;
    for r in 0..dst.len() / ctx.stride {
        let py = band_y0 + r as i64;
        let dy = (py as f32 + 0.5) - ctx.cy;
        let row = r * ctx.stride;
        for px in ctx.x0..ctx.x1 {
            // SILHOUETTE — via the shared [`silhouette_at`], which the Impasto height kernel also
            // calls: one definition of a dab's shape, so relief and pigment cannot drift apart.
            let dx = (px as f32 + 0.5) - ctx.cx;
            // `w` is the DEFORMED point and `t` its length — byte-identical to `falloff_t`, which is
            // exactly `apply` then the same `sqrt`. The vector itself is what the LUT's expansion
            // needs (the metric is euclidean in that space), so it is kept instead of thrown away.
            let wv = ctx
                .footprint
                .apply([dx * ctx.inv_radius, dy * ctx.inv_radius]);
            let t = (wv[0] * wv[0] + wv[1] * wv[1]).sqrt();
            // A ponta RETA (`dab_quina`, J3 A-4): a norma do `falloff_t`, o quadrado no referencial — sem quina
            // o `sqrt` de cima, ao bit (`basicos::a_ponta_reta_do_digital_pinta_um_rectangulo`).
            let t = if ctx.footprint.quina() > 0.0 {
                ctx.footprint
                    .falloff_t(dx * ctx.inv_radius, dy * ctx.inv_radius)
            } else {
                t
            };
            // The FILM ([`crate::height::film_coverage`]): a brush laying body lays no pigment where it
            // lays no body. Applied to the SILHOUETTE — before the Grain, before the dynamics — so every
            // funnel below (grain, Accumulate-OFF cap, ramps) inherits the cut with no arithmetic.
            // With Smooth Edges ([`crate::height_film::FilmAa`], BUGS #16) the film at a rim texel is
            // its fractional AREA coverage — every funnel inherits the anti-aliased cut the same way.
            // A capsule composes its sub-dabs at the pass opacity: the dab's own, or — under the
            // Accumulate cap, where `w` is a RATE toward the ceiling — the Flow of ONE sub-dab,
            // `1 − (1 − flow)^(1/n)` (the step's rate, shared by its `n` sub-dabs): the rates multiply
            // across capsules exactly as inside one, so the seam between two steps gets no extra pass
            // (dono 25/10: Continuous + Flow < 1 em contas). Flow 1 ⇒ `1`, the law that shipped. `n` is the STEP's:
            // the opening dab the 1.st capsule carries ([`crate::Ante::fecho`]) is one more sub-dab at that rate.
            // A capsule under a Shape ramp composes its COLOUR over the same sub-dabs
            // ([`super::cor_capsula`], lista 26 A9); `None` ⇒ the colour is read at this pixel below.
            let c_cap = match (mask.is_some(), ctx.capsula) {
                (true, Some(cap)) if ctx.flow < 1.0 => {
                    1.0 - (1.0 - ctx.flow.max(0.0)).det_powf(1.0 / cap.n_do_passo() as f32)
                }
                (true, _) => 1.0,
                (false, _) => ctx.coverage,
            };
            // A orla em PÍXEIS (J3, `BrushSpec::borda_px`) sob o tecto do traço é ÁREA: o texel pára na fracção que o
            // dab cobre (o tecto da Smooth Edges, BUGS #16) e, na cápsula, no MAIOR sub-dab — a união deles enchia a
            // orla de volta ao duro (`basicos::a_orla_em_pixeis_e_a_mesma_a_40_e_a_200`).
            let orla_area = mask.is_some() && ctx.spec.borda_px > 0.0 && ctx.spec.leis_do_digital();
            let mut maior_sub = 0.0f32;
            let mut cor_cap = ctx
                .capsula
                .filter(|_| ctx.ramp.is_some_and(|r| r.index == RampIndex::Shape))
                .map(|_| super::super::cor_capsula::CorCapsula::new(c_cap));
            let w = match (ctx.capsula, &ctx.film_aa) {
                (Some(cap), _) => {
                    let uniao = crate::dab::silhouette_capsula_com(
                        ctx.spec,
                        ctx.spec.queda_do_dab(),
                        ctx.shape,
                        ctx.footprint,
                        cap,
                        ctx.lei,
                        c_cap,
                        px,
                        py,
                        ctx.center,
                        ctx.radius,
                        |s, r, x| {
                            maior_sub = maior_sub.max(x);
                            let Some(cc) = cor_cap.as_mut() else { return };
                            cc.mais(x, || {
                                let centro = [ctx.center[0] + s.off[0], ctx.center[1] + s.off[1]];
                                let forma = ctx.shape_cor.map(|sh| {
                                    let base = sh.basis.sub(s.rotor, s.arco);
                                    let sh = crate::dab::ShapeInput { basis: &base, ..sh };
                                    crate::dab::shape_value_at(ctx.spec, sh, px, py, centro, r)
                                });
                                crate::ramp_alpha::ramp_texel(
                                    ctx.ramp.as_ref(),
                                    x,
                                    forma,
                                    None,
                                    0.0,
                                    0.0,
                                    &crate::texture::GrainLaw::NEUTRA,
                                )
                            });
                        },
                    );
                    if orla_area { maior_sub } else { uniao }
                }
                (None, Some(aa)) => aa.film_at_planned(
                    ctx.lut.as_ref(),
                    t,
                    wv,
                    [dx, dy],
                    || silhouette_at(ctx.spec, ctx.shape, t, px, py, ctx.center, ctx.radius),
                    |ox, oy| {
                        ctx.spec.falloff_weight(
                            ctx.footprint
                                .falloff_t((dx + ox) * ctx.inv_radius, (dy + oy) * ctx.inv_radius),
                        )
                    },
                ),
                (None, None) => crate::height::film_coverage(
                    ctx.spec.deposits_height(),
                    silhouette_at(ctx.spec, ctx.shape, t, px, py, ctx.center, ctx.radius),
                ),
            };
            // Skip pixels the silhouette already zeroes BEFORE the grain sample — the grain only
            // modulates where the dab paints, so sampling it there is pure waste (large Anchored).
            if w <= 0.0 {
                continue;
            }
            if let Some(l) = liso.as_deref_mut() {
                liso_tocou |= super::super::liso::escreve(
                    ctx,
                    l,
                    row + ((px - ctx.ox) as usize) * 4,
                    w,
                    blend,
                    false,
                );
            }
            // `w` = dab PROFILE (falloff × silhouette, the build factor); `g` = texture COVERAGE factor
            // (Grain / Strength-ramp) kept SEPARATE so it CAPS the pixel — a 0.3 grain texel tops at
            // 0.3·Strength under re-passing, not climbing to full (the "fills in" bug). `g=1` ⇒ identical.
            // The colour and the Grain's bite come from ONE door ([`crate::ramp_alpha::ramp_texel`]):
            // the ramp is indexed by its OWNER's value (the Grain, or the Shape's layers: its texture
            // value and the coverage `w`), and the Grain bites unless it is the one indexing the ramp.
            let grain = ctx.tex.map(|b| {
                let amostra = || {
                    crate::texture::sample(
                        &ctx.spec.texture,
                        b,
                        px,
                        py,
                        ctx.center,
                        ctx.radius,
                        ctx.image.as_ref(),
                    )
                };
                match ctx.grao.as_deref() {
                    Some(c) => c.em(px, py, amostra),
                    None => amostra(),
                }
            });
            let lei = &ctx.spec.texture.lei;
            let grain = grain.map(|t| lei.papel_com_po(t, ctx.po.as_ref(), px, py));
            let (depth, gran) = (ctx.spec.grain_depth(), ctx.spec.effective_granulation());
            let tinta = match cor_cap {
                Some(cc) => {
                    let mordida =
                        crate::ramp_alpha::ramp_texel(None, w, None, grain, depth, gran, lei).g;
                    let Some(t) = cc.fim(mordida) else { continue };
                    t
                }
                None => {
                    let forma = ctx.shape_cor.map(|sh| {
                        crate::dab::shape_value_at(ctx.spec, sh, px, py, ctx.center, ctx.radius)
                    });
                    crate::ramp_alpha::ramp_texel(
                        ctx.ramp.as_ref(),
                        w,
                        forma,
                        grain,
                        depth,
                        gran,
                        lei,
                    )
                }
            };
            let color = tinta.color.unwrap_or(ctx.spec.color);
            let stamp_alpha = tinta.stamp_alpha; // mode TextureAlpha: the source pixel's own alpha
            let mut g = tinta.g * crate::dente::dente_at(ctx.spec, w, px, py);
            if let Some(b) = ctx.tex {
                g *= crate::texture::stencil_gate(&ctx.spec.texture, b, px, py); // rect mask, ramp-safe
                if w * g <= 0.0 {
                    continue;
                }
            }
            let i = row + ((px - ctx.ox) as usize) * 4;
            let prev = [
                f32::from(dst[i]) / 255.0,
                f32::from(dst[i + 1]) / 255.0,
                f32::from(dst[i + 2]) / 255.0,
                f32::from(dst[i + 3]) / 255.0,
            ];
            // Mode TextureAlpha stamps the ramp as an RGBA image so translucent areas LOWER the
            // sprite's own alpha (punch it transparent) — alpha-lock can't apply (it edits alpha).
            // The other modes gate coverage by the dest alpha when alpha-locked + blend with the brush
            // mode (colour blend modes keep the dest alpha; you only paint where there's coverage).
            let out = if ctx
                .ramp
                .is_some_and(|r| matches!(r.alpha_mode, RampAlphaMode::TextureAlpha))
            {
                let m = w * g * ctx.coverage;
                if m <= 0.0 {
                    continue;
                }
                stamp_rgba(prev, color, stamp_alpha, m)
            } else {
                let a = match mask.as_deref_mut() {
                    // The per-stroke coverage buffer answers "how much of THIS stroke's paint is here?",
                    // and its law lives in ONE place ([`crate::stroke_cover::cover_add`]): the
                    // Accumulate-OFF cap, where the dab profile is a RATE toward the ceiling. Whatever
                    // the numbers, `a = add/(1 − m)` makes the canvas hold exactly
                    // `pre_stroke·(1 − m) + colour·m` — the deposit lands on the state the stroke
                    // started from, by telescoping and with no copy of the canvas (see the module).
                    // At `cap = 1` this reduces to the EXACT maskless alpha sequence, so a full-strength
                    // interior is byte-identical whether the buffer is threaded or not.
                    Some(m_buf) => {
                        let mi = r * (ctx.stride / 4) + (px - ctx.ox) as usize;
                        let m = f32::from(m_buf[mi]) / 255.0;
                        // The capsule's union already carries its sub-dabs' rate (`c_cap` above):
                        // `w·c_cap = 1 − Π(1 − c_cap·x_k)`.
                        let flow = if ctx.capsula.is_some() && !orla_area {
                            c_cap
                        } else {
                            ctx.flow
                        };
                        // O Wash pára no MAIOR dab: na cápsula, o maior sub-dab.
                        let add = if ctx.spec.wash {
                            let w = if ctx.capsula.is_some() { maior_sub } else { w };
                            crate::stroke_cover::cover_max(m, w, g, ctx.tecto, ctx.flow)
                        } else {
                            crate::stroke_cover::cover_add(
                                m,
                                w,
                                g,
                                ctx.tecto,
                                flow,
                                crate::stroke_cover::Area::do_pincel(
                                    ctx.spec,
                                    ctx.film_aa.is_some(),
                                    orla_area,
                                ),
                            )
                        };
                        let Some(add) = add else {
                            continue; // this dab adds nothing here
                        };
                        m_buf[mi] = ((m + add) * 255.0 + 0.5) as u8;
                        let a = add / (1.0 - m).max(1e-4);
                        if ctx.preserve_alpha { a * prev[3] } else { a }
                    }
                    None if ctx.preserve_alpha => w * g * ctx.coverage * prev[3],
                    None => w * g * ctx.coverage,
                };
                if a <= 0.0 {
                    continue;
                }
                if let Some(f) = forca.as_deref_mut() {
                    crate::stroke_cover::compoe_forca(
                        &mut f[r * (ctx.stride / 4) + (px - ctx.ox) as usize],
                        a,
                    );
                }
                crate::blend_vazio::blend_over_pigment_modo(
                    blend,
                    prev,
                    color,
                    a,
                    ctx.spec.pigmento(),
                    ctx.spec.blend_sobre_vazio,
                )
            };
            dst[i] = encode(out[0]);
            dst[i + 1] = encode(out[1]);
            dst[i + 2] = encode(out[2]);
            dst[i + 3] = encode(out[3]);
            touched = true;
        }
    }
    (touched, liso_tocou)
}

const LADO: usize = 64;

fn padrao(n: usize, mul: usize, modulo: usize) -> Vec<u8> {
    (0..n).map(|i| ((i * mul + 7) % modulo) as u8).collect()
}

/// O passe de uma banda sob prova: `(ctx, dst, liso, máscara, força, y0)`.
type PasseDaBanda = fn(
    &DabCtx,
    &mut [u8],
    Option<&mut [u8]>,
    Option<&mut [u8]>,
    Option<&mut [u16]>,
    i64,
) -> (bool, bool);

/// Um dab pela banda de agora e pela antiga, sobre os mesmos buffers; devolve se algum texel foi pintado.
#[allow(clippy::too_many_arguments)]
fn compara(
    spec: &BrushSpec,
    capsula: Option<crate::capsula::Capsula>,
    flow: f32,
    preserve_alpha: bool,
    com_mascara: bool,
    com_forca: bool,
    com_liso: bool,
    caso: &str,
) -> bool {
    compara_entre(
        spec,
        capsula,
        flow,
        preserve_alpha,
        com_mascara,
        com_forca,
        com_liso,
        caso,
        stamp_band,
        stamp_band_antiga,
    )
    .0
}

/// [`compara`] entre dois passes quaisquer; devolve também quantas silhuetas cada um avaliou.
#[allow(clippy::too_many_arguments)]
fn compara_entre(
    spec: &BrushSpec,
    capsula: Option<crate::capsula::Capsula>,
    flow: f32,
    preserve_alpha: bool,
    com_mascara: bool,
    com_forca: bool,
    com_liso: bool,
    caso: &str,
    passe_novo: PasseDaBanda,
    passe_antigo: PasseDaBanda,
) -> (bool, usize, usize) {
    let radius = spec.radius_px;
    let center = [30.3f32, 33.7];
    let footprint = spec.dab_footprint([0.8, 0.6]);
    let film_aa = crate::height_film::FilmAa::for_dab(spec, false, radius);
    let ctx = DabCtx {
        spec,
        tex: None,
        image: None,
        shape: None,
        shape_cor: None,
        ramp: None,
        footprint,
        center,
        cx: center[0],
        cy: center[1],
        inv_radius: 1.0 / radius,
        radius,
        coverage: 0.8,
        tecto: 0.9,
        flow,
        preserve_alpha,
        x0: 3,
        x1: 61,
        stride: LADO * 4,
        ox: 0,
        vazio_local: false,
        film_aa,
        lut: None,
        capsula,
        lei: crate::capsula::lei(),
        po: None,
        grao: None,
        dente: None,
    };
    let n = LADO * LADO;
    let dst0 = padrao(n * 4, 37, 251);
    let mask0 = padrao(n, 13, 200);
    let forca0: Vec<u16> = (0..n).map(|i| ((i * 911) % 60_000) as u16).collect();
    let corre = |f: PasseDaBanda| {
        let (mut d, mut m, mut fo, mut l) = (
            dst0.clone(),
            mask0.clone(),
            forca0.clone(),
            vec![0u8; n * 4],
        );
        let mut tocou = (false, false);
        // Duas bandas (linhas 0..20 e 20..64) e dois dabs seguidos: a máscara lida é a que o 1.º escreveu.
        for _ in 0..2 {
            for (a, b) in [(0usize, 20usize), (20, LADO)] {
                let r = f(
                    &ctx,
                    &mut d[a * LADO * 4..b * LADO * 4],
                    com_liso.then(|| &mut l[a * LADO * 4..b * LADO * 4]),
                    com_mascara.then(|| &mut m[a * LADO..b * LADO]),
                    com_forca.then(|| &mut fo[a * LADO..b * LADO]),
                    a as i64,
                );
                tocou = (tocou.0 | r.0, tocou.1 | r.1);
            }
        }
        (d, m, fo, l, tocou)
    };
    let conta = |f: PasseDaBanda| {
        super::corte::AVALIADOS.with(|c| c.set(0));
        let r = corre(f);
        (r, super::corte::AVALIADOS.with(std::cell::Cell::get))
    };
    let (novo, n_novo) = conta(passe_novo);
    let (antigo, n_antigo) = conta(passe_antigo);
    assert!(novo.0 == antigo.0, "{caso}: o dst diverge");
    assert!(novo.1 == antigo.1, "{caso}: a máscara diverge");
    assert!(novo.2 == antigo.2, "{caso}: a força diverge");
    assert!(novo.3 == antigo.3, "{caso}: o liso diverge");
    assert_eq!(novo.4, antigo.4, "{caso}: o que tocou diverge");
    (novo.0 != dst0, n_novo, n_antigo)
}

/// **O corte de `stamp_band` em silhueta · tinta · depósito, com as invariantes do dab içadas, pinta os MESMOS
/// bytes que a de antes** (dst, máscara do tecto, força u16, liso e o «tocou»), em cada rota do kernel sem textura:
/// película com AA, orla em píxeis, Wash, ponta reta, cápsula com Flow < 1 e = 1, alpha lock, com e sem tecto.
#[test]
fn o_corte_do_carimbo_pinta_os_mesmos_bytes_que_a_banda_antiga() {
    let base = BrushSpec {
        radius_px: 20.0,
        ..Default::default()
    };
    let specs = [
        ("liso", base),
        (
            "pelicula_aa",
            BrushSpec {
                impasto: true,
                impasto_depth: 0.5,
                impasto_smooth_edges: true,
                ..base
            },
        ),
        (
            "orla_px",
            BrushSpec {
                borda_px: 2.0,
                ..base
            },
        ),
        ("wash", BrushSpec { wash: true, ..base }),
        (
            "quina",
            BrushSpec {
                dab_quina: 0.6,
                ..base
            },
        ),
    ];
    let capsulas = [
        None,
        crate::capsula::Capsula::new([9.0, 4.0], 2.0),
        crate::capsula::Capsula::new([9.0, 4.0], 2.0).map(|c| c.com_fecho(true)),
    ];
    let (mut casos, mut pintados) = (0usize, 0usize);
    for (nome, spec) in &specs {
        for (ci, cap) in capsulas.iter().enumerate() {
            for flow in [1.0f32, 0.35] {
                for pa in [false, true] {
                    for (mascara, forca, liso) in [
                        (false, false, false),
                        (false, false, true),
                        (true, false, false),
                        (true, true, false),
                    ] {
                        let caso =
                            format!("{nome} cap{ci} flow{flow} pa{pa} m{mascara} f{forca} l{liso}");
                        casos += 1;
                        pintados +=
                            usize::from(compara(spec, *cap, flow, pa, mascara, forca, liso, &caso));
                    }
                }
            }
        }
    }
    assert!(capsulas[1].is_some(), "o fixture tem de ter cápsula");
    assert_eq!(
        pintados, casos,
        "todo caso tem de pintar algo ({pintados}/{casos})"
    );
}

/// Sonda B26.1 (`--profile smoke -- --ignored`): a banda de agora × a antiga no MESMO processo, intercaladas em
/// blocos, o MÍNIMO de cada — Continuous + Flow < 1 sob o tecto (cápsula: o `det_powf` por texel de antes).
#[test]
#[ignore = "sonda de relógio: --profile smoke"]
fn sonda_o_carimbo_com_invariantes_icadas_custa_menos() {
    type Banda = fn(
        &DabCtx,
        &mut [u8],
        Option<&mut [u8]>,
        Option<&mut [u8]>,
        Option<&mut [u16]>,
        i64,
    ) -> (bool, bool);
    const L: usize = 160;
    for (nome, cap) in [
        (
            "capsula flow 0,35",
            crate::capsula::Capsula::new([6.0, 2.0], 1.5),
        ),
        ("dab simples flow 0,35", None),
    ] {
        let spec = BrushSpec {
            radius_px: 60.0,
            ..Default::default()
        };
        let center = [80.0f32, 80.0];
        let ctx = DabCtx {
            spec: &spec,
            tex: None,
            image: None,
            shape: None,
            shape_cor: None,
            ramp: None,
            footprint: spec.dab_footprint([1.0, 0.0]),
            center,
            cx: center[0],
            cy: center[1],
            inv_radius: 1.0 / 60.0,
            radius: 60.0,
            coverage: 0.8,
            tecto: 0.9,
            flow: 0.35,
            preserve_alpha: false,
            x0: 0,
            x1: L as i64,
            stride: L * 4,
            ox: 0,
            vazio_local: false,
            film_aa: None,
            lut: None,
            capsula: cap,
            lei: crate::capsula::lei(),
            po: None,
            grao: None,
            dente: None,
        };
        let mede = |f: Banda| {
            let (mut d, mut m) = (vec![0u8; L * L * 4], vec![0u8; L * L]);
            let t = std::time::Instant::now();
            for _ in 0..8 {
                d.fill(0);
                m.fill(0);
                let _ = f(&ctx, &mut d, None, Some(&mut m), None, 0);
            }
            t.elapsed().as_secs_f64() * 1e3 / 8.0
        };
        let (mut novo, mut antigo, mut sem) = (f64::MAX, f64::MAX, f64::MAX);
        for bloco in 0..12 {
            if bloco % 2 == 0 {
                novo = novo.min(mede(com_corte));
                sem = sem.min(mede(sem_corte));
                antigo = antigo.min(mede(stamp_band_antiga));
            } else {
                antigo = antigo.min(mede(stamp_band_antiga));
                sem = sem.min(mede(sem_corte));
                novo = novo.min(mede(com_corte));
            }
        }
        println!(
            "[B26.1] {nome}: antiga {antigo:.3} · sem corte {sem:.3} · com corte {novo:.3} ms/dab · corte {:.2}× · total {:.2}×",
            sem / novo,
            antigo / novo
        );
    }
}

fn com_corte(
    ctx: &DabCtx,
    dst: &mut [u8],
    liso: Option<&mut [u8]>,
    mask: Option<&mut [u8]>,
    forca: Option<&mut [u16]>,
    y0: i64,
) -> (bool, bool) {
    stamp_band_com(ctx, dst, liso, mask, forca, y0, true)
}

fn sem_corte(
    ctx: &DabCtx,
    dst: &mut [u8],
    liso: Option<&mut [u8]>,
    mask: Option<&mut [u8]>,
    forca: Option<&mut [u16]>,
    y0: i64,
) -> (bool, bool) {
    stamp_band_com(ctx, dst, liso, mask, forca, y0, false)
}

/// **B26.1: o corte por linha salta texéis, e só os que não pintam.** Na pegada inclinada e na ponta reta (raio 20 numa
/// caixa 58 × 64) a banda com o corte avalia a silhueta em menos de 60 % dos texéis da caixa e pinta os mesmos bytes
/// que sem ele (dst, máscara, força, liso); a película com AA (que pode pintar além de `t = 1`) não corta.
#[test]
fn o_corte_da_linha_salta_so_o_que_nao_pinta() {
    let base = BrushSpec {
        radius_px: 20.0,
        ..Default::default()
    };
    for (nome, spec, corta) in [
        ("liso", base, true),
        (
            "quina",
            BrushSpec {
                dab_quina: 0.6,
                ..base
            },
            true,
        ),
        (
            "pelicula_aa",
            BrushSpec {
                impasto: true,
                impasto_depth: 0.5,
                impasto_smooth_edges: true,
                ..base
            },
            false,
        ),
    ] {
        let (pintou, com, sem) = compara_entre(
            &spec, None, 0.35, false, true, true, false, nome, com_corte, sem_corte,
        );
        assert!(pintou, "{nome}: o fixture tem de pintar");
        let caixa = 58 * LADO * 2; // 58 colunas × 64 linhas × 2 dabs
        assert_eq!(
            sem, caixa,
            "{nome}: sem o corte o laço percorre a caixa inteira"
        );
        if corta {
            assert!(
                com * 10 < caixa * 6,
                "{nome}: o corte avaliou {com} de {caixa} texéis (≥ 60 %)"
            );
        } else {
            assert_eq!(com, caixa, "{nome}: a película com AA não pode cortar");
        }
    }
}

/// Missão 4K onda 3: o atalho do branco sobre o `[0; 4]` da pegada local é a `blend_over_pigment_modo` AO BIT, para
/// todo `a` de 2²⁴ passos em `(0, 1,5]` e toda mistura de pigmento K–M (o espectral não entra no atalho).
#[test]
fn o_branco_no_vazio_e_a_blend_de_sempre_ao_bit() {
    for mix in [0.0f32, 0.3, 1.0] {
        for vazio in [false, true] {
            let p = crate::blend::Pigmento {
                mix,
                espectral: false,
            };
            for k in 1..=(1u32 << 24) {
                let a = k as f32 * (1.5 / (1u32 << 24) as f32);
                let de_sempre = crate::blend_vazio::blend_over_pigment_modo(
                    crate::BrushBlend::Mix,
                    [0.0; 4],
                    [1.0; 3],
                    a,
                    p,
                    vazio,
                );
                assert_eq!(
                    super::branco_no_vazio(a).map(f32::to_bits),
                    de_sempre.map(f32::to_bits),
                    "a = {a}, mix {mix}, vazio {vazio}"
                );
            }
        }
    }
}
