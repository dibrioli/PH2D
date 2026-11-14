//! **E9 — o portão-mãe** (`docs/Painter/Arquitetura/E9_portao_mae.md`): os limiares finais da F0 §4 (F0b),
//! lidos no render PLANO, sobre uma célula da sonda. A mesma tabela vive em
//! `docs/Painter/Arquitetura/fotos/f0_varredura/resumo_f0.py` (a grelha inteira).
use super::*;
use ph2d_det_math::DetMath as _;
use ph2d_wet_paint::sim::LeiDaTinta;

/// sRGB `0..255` → Oklab (Ottosson 2020; as matrizes de `ph2d_paint_tinta::oklab`).
fn oklab(c: [f64; 3]) -> [f64; 3] {
    let l = c.map(|v| srgb_to_linear(v / 255.0));
    let lms = [
        0.412_221_470_8 * l[0] + 0.536_332_536_3 * l[1] + 0.051_445_992_9 * l[2],
        0.211_903_498_2 * l[0] + 0.680_699_545_1 * l[1] + 0.107_396_956_6 * l[2],
        0.088_302_461_9 * l[0] + 0.281_718_837_6 * l[1] + 0.629_978_700_5 * l[2],
    ]
    .map(ph2d_det_math::DetMath::det_cbrt);
    [
        0.210_454_255_3 * lms[0] + 0.793_617_785 * lms[1] - 0.004_072_046_8 * lms[2],
        1.977_998_495_1 * lms[0] - 2.428_592_205 * lms[1] + 0.450_593_709_9 * lms[2],
        0.025_904_037_1 * lms[0] + 0.782_771_766_2 * lms[1] - 0.808_675_766 * lms[2],
    ]
}

/// Oklab → sRGB `0..255` (sem corte: o chamador compõe e mede).
fn srgb_de_oklab(o: [f64; 3]) -> [f64; 3] {
    let l = o[0] + 0.396_337_777_4 * o[1] + 0.215_803_757_3 * o[2];
    let m = o[0] - 0.105_561_345_8 * o[1] - 0.063_854_172_8 * o[2];
    let s = o[0] - 0.089_484_177_5 * o[1] - 1.291_485_548 * o[2];
    let (l, m, s) = (l * l * l, m * m * m, s * s * s);
    [
        4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s,
        -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s,
        -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701 * s,
    ]
    .map(|v| 255.0 * linear_to_srgb(v.clamp(0.0, 1.0)))
}

fn srgb_to_linear(v: f64) -> f64 {
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).det_powf(2.4)
    }
}

fn linear_to_srgb(v: f64) -> f64 {
    if v <= 0.003_130_8 {
        12.92 * v
    } else {
        1.055 * v.det_powf(1.0 / 2.4) - 0.055
    }
}

/// O segmento da mistura da lei `Pigmento` (Oklab pesado pelo volume, F2b §4): 201 pontos sRGB de `a` a `b`.
pub(super) fn segmento_oklab(a: [f64; 3], b: [f64; 3]) -> Vec<[f64; 3]> {
    let (oa, ob) = (oklab(a), oklab(b));
    (0..=200)
        .map(|i| {
            let c = f64::from(i) / 200.0;
            srgb_de_oklab([0, 1, 2].map(|k| (1.0 - c) * oa[k] + c * ob[k]))
        })
        .collect()
}

/// A distância (níveis sRGB) de `x` (já sobre o creme) ao segmento `seg` composto sobre o creme com o alfa `a`
/// do píxel — o `mix_*` da E9 §5.3 (a lei activa da Pigmento).
pub(super) fn fora_do_segmento(x: [f64; 3], seg: &[[f64; 3]], a: f64) -> f64 {
    seg.iter()
        .map(|m| {
            (0..3)
                .map(|k| (x[k] - (m[k] * a + CREME[k] * (1.0 - a))).det_powi(2))
                .sum::<f64>()
                .sqrt()
        })
        .fold(f64::INFINITY, f64::min)
}

pub(super) const MANCHAS_AZUL_MAX: f64 = 1.0;
pub(super) const KM_P99_MAX: f64 = 8.0;
pub(super) const KM_MAX_MAX: f64 = 20.0;
pub(super) const FERRUGEM_MAX: f64 = 0.05;
/// `furos = 0` só onde a mesma célula a pressão 1 cobre (furos < 1 %): abaixo disso é tinta fina, não furo.
pub(super) const FUROS_PR1_COBRE: f64 = 1.0;
pub(super) const DEGRAUS_X_MAX: usize = 2;

/// As métricas que reprovam (vazio = passa). `furos_pr1` = os furos da mesma célula a pressão 1. Um `NaN`
/// (nada com alfa ≥ 0,9) não reprova: é a régua que não mede, e o chamador diz se a célula TINHA de medir.
/// A régua da mistura é a da LEI: `km_*` (K-M) na `Velha`, `mix_*` (segmento Oklab) na `Pigmento` (E9 §5.3).
pub(super) fn reprova(c: &Celula, n: usize, furos_pr1: f64, lei: LeiDaTinta) -> Vec<String> {
    let q = c.plano;
    let mut r = vec![];
    if q.manchas_azul > MANCHAS_AZUL_MAX {
        r.push(format!("manchas_azul_plano {:.2}", q.manchas_azul));
    }
    let (nome, p99, max) = match lei {
        LeiDaTinta::Velha => ("km", q.km_p99, q.km_max),
        LeiDaTinta::Pigmento => ("mix", q.mix_p99, q.mix_max),
    };
    if p99 > KM_P99_MAX {
        r.push(format!("{nome}_p99_plano {p99:.1}"));
    }
    if max > KM_MAX_MAX {
        r.push(format!("{nome}_max_plano {max:.1}"));
    }
    if q.ferrugem > FERRUGEM_MAX {
        r.push(format!("ferrugem_plano {:.2}", q.ferrugem));
    }
    if furos_pr1 < FUROS_PR1_COBRE && c.furos > 0.0 {
        r.push(format!("furos {:.2}", c.furos));
    }
    if q.degraus_y > n.saturating_sub(1) {
        r.push(format!("degraus_y_plano {}", q.degraus_y));
    }
    if q.degraus_x > DEGRAUS_X_MAX {
        r.push(format!("degraus_x_plano {}", q.degraus_x));
    }
    r
}

/// Os pincéis do portão-mãe, um irmão de teste cada (integração 02/11c: as quatro células corriam em série,
/// ~36 s cada em debug, e passavam do prazo do nextest sob carga).
const PINCEIS_DO_PORTAO: [&str; 2] = ["Oil/Flat/Thick Impasto", "Watercolor/Fluid/Wet Round"];

/// O portão-mãe de UM pincel: as duas leis (a `Velha` só imprime, a `Pigmento` reprova), em paralelo — cada célula
/// é uma ferramenta nova e independente, por isso o paralelo não muda nenhuma medida.
fn portao_mae_f0_do(pincel: &str) {
    assert!(PINCEIS_DO_PORTAO.contains(&pincel));
    let b = Biblioteca::carregar(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
        None,
    );
    let leis = [LeiDaTinta::Velha, LeiDaTinta::Pigmento];
    let celulas: Vec<Celula> = std::thread::scope(|s| {
        let fios: Vec<_> = leis
            .map(|lei| {
                let b = &b;
                s.spawn(move || {
                    let cena = Cena {
                        lei,
                        fundo: Papel::Padrao,
                        ..Cena::default()
                    };
                    celula(b, pincel, 600.0, 0.5, 1.0, 3, 300, cena, None)
                })
            })
            .into_iter()
            .collect();
        fios.into_iter().map(|f| f.join().unwrap()).collect()
    });
    let mut falhas = vec![];
    for (lei, c) in leis.into_iter().zip(&celulas) {
        // pressão 1: a célula É o seu par de cobertura.
        let r = reprova(c, 3, c.furos, lei);
        let q = c.plano;
        println!(
            "[E9] {pincel} {}: manchas_azul {:.2} km_p99 {:.1} km_max {:.1} mix_p99 {:.1} mix_max {:.1} ferrugem {:.2} furos {:.2} \
             degraus {}/{} → {r:?}",
            lei.nome(),
            q.manchas_azul,
            q.km_p99,
            q.km_max,
            q.mix_p99,
            q.mix_max,
            q.ferrugem,
            c.furos,
            q.degraus_y,
            q.degraus_x
        );
        if lei == LeiDaTinta::Pigmento {
            assert!(
                q.mix_p99.is_finite(),
                "{pincel}: P600/pr1 não cobre (alfa < 0,9) — o portão não mediu nada"
            );
            if !r.is_empty() {
                falhas.push(format!("{pincel}: {r:?}"));
            }
        }
    }
    assert!(
        falhas.is_empty(),
        "lei Pigmento reprova o portão-mãe: {falhas:?}"
    );
}

#[test]
fn portao_mae_f0_lei_pigmento_minimo_thick_impasto() {
    portao_mae_f0_do(PINCEIS_DO_PORTAO[0]);
}

#[test]
fn portao_mae_f0_lei_pigmento_minimo_wet_round() {
    portao_mae_f0_do(PINCEIS_DO_PORTAO[1]);
}

/// Diagnóstico da 1.ª corrida (E9 §5): por ponto do eixo central, o pixel composto, as somas da célula e o pixel
/// depois de tirar o relevo (`relevo`/`altura` a 0) e depois de tirar o Óleo Real (`oleo` vazio).
#[test]
#[ignore = "diagnóstico E9 — corre à mão"]
fn diagnostico_e9_pigmento_por_ponto() {
    let b = Biblioteca::carregar(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
        None,
    );
    for fundo in [Papel::Padrao, Papel::Liso] {
        for lei in [LeiDaTinta::Velha, LeiDaTinta::Pigmento] {
            let cena = Cena {
                lei,
                fundo,
                ..Cena::default()
            };
            let mut t = pinta(
                &b,
                "Oil/Flat/Thick Impasto",
                600.0,
                0.5,
                1.0,
                3,
                300,
                cena,
                &mut |_| {},
            );
            t.wet_recomposite_full();
            let ys = [100usize, 112, 124, 128, 132, 140, 154, 170];
            let x = 180usize;
            let px = |t: &PainterTool, y: usize| {
                let q = &t.canvas_rgba[(x + y * W as usize) * 4..][..4];
                [q[0], q[1], q[2], q[3]]
            };
            let luz: Vec<_> = ys.iter().map(|&y| px(&t, y)).collect();
            let celulas: Vec<String> = {
                let s = t.paint.wetpaint.session.as_mut().expect("sessão");
                s.bring_home();
                let r = usize::from(s.ratio);
                let g = &s.engine.layers[0].grid;
                ys.iter()
                    .map(|&y| {
                        let i = (x / r + 1) + (y / r + 1) * g.s;
                        let c = g.tinta.get(i).copied();
                        let lab = |v: f32, l: [f32; 3]| if v > 0.0 { l.map(|x| x / v) } else { [0.0; 3] };
                        format!(
                            "susp {:.0} sett {:.0} film {:.3} relevo {:.2} | {}",
                            g.susp[i],
                            g.sett[i],
                            g.film[i],
                            g.relevo.get(i).copied().unwrap_or(0.0),
                            c.map_or("sem tinta".into(), |c| format!(
                                "filme v {:.3} lab {:.3?} t {:.2} · aberto v {:.3} lab {:.3?} · r_g {:.3?} r_g0 {:.3?} m {:.2} o {:.2}",
                                c.filme.v, lab(c.filme.v, c.filme.lab), c.filme.t, c.aberto.v, lab(c.aberto.v, c.aberto.lab), c.r_g, c.r_g0, c.meio(), c.o
                            ))
                        )
                    })
                    .collect()
            };
            {
                let s = t.paint.wetpaint.session.as_mut().expect("sessão");
                for l in &mut s.engine.layers {
                    l.grid.relevo.iter_mut().for_each(|v| *v = 0.0);
                    l.grid.altura.iter_mut().for_each(|v| *v = 0.0);
                }
            }
            t.wet_recomposite_full();
            let sem_relevo: Vec<_> = ys.iter().map(|&y| px(&t, y)).collect();
            {
                let s = t.paint.wetpaint.session.as_mut().expect("sessão");
                for l in &mut s.engine.layers {
                    l.grid.oleo.clear();
                }
            }
            t.set_wet_oleo_real(None);
            t.wet_recomposite_full();
            let sem_oleo: Vec<_> = ys.iter().map(|&y| px(&t, y)).collect();
            println!("[diag] {} {}", fundo.nome(), lei.nome());
            for (k, y) in ys.iter().enumerate() {
                println!(
                    "  y {y}: luz {:?} · sem relevo {:?} · sem óleo {:?}\n      {}",
                    luz[k], sem_relevo[k], sem_oleo[k], celulas[k]
                );
            }
        }
    }
}

/// Uma sessão nova (desfazer que a mata, guarda, grid ratio) herda a lei da tinta da anterior.
#[test]
fn a_sessao_nova_herda_a_lei_da_tinta() {
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(crate::PaintMedia::WetPaint);
    t.set_wet_lei_da_tinta(LeiDaTinta::Pigmento);
    assert_eq!(t.wet_lei_da_tinta(), Some(LeiDaTinta::Pigmento));
    t.paint.wetpaint.session = Default::default();
    traco(&mut t, [50.0, 50.0], [80.0, 50.0], 1.0, 0); // o traço cria a sessão nova
    assert_eq!(t.wet_lei_da_tinta(), Some(LeiDaTinta::Pigmento));
    t.set_wet_lei_da_tinta(LeiDaTinta::Velha);
    t.paint.wetpaint.session = Default::default();
    traco(&mut t, [50.0, 50.0], [80.0, 50.0], 1.0, 0); // o traço cria a sessão nova
    assert_eq!(t.wet_lei_da_tinta(), Some(LeiDaTinta::Velha));
}

/// J7 (`docs/Painter/Arquitetura/J7_pasta_do_oleo.md`): o óleo pouco acima da pasta (P600 × Water 0,5, Dilution ≈ 0,127)
/// cruza o amarelo sobre o azul em VERDE, como o óleo do tubo — o pigmento que o óleo prende pousa como pasta e o
/// líquido livre leva a mistura. Antes da cura: bege (o filme molhado tapava), ferrugem 3,48 e 21 degraus no Filbert Dry.
#[test]
fn o_oleo_pouco_diluido_cruza_o_amarelo_no_azul_em_verde() {
    let b = Biblioteca::carregar(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
        None,
    );
    let cena = Cena {
        lei: LeiDaTinta::Velha,
        fundo: Papel::Padrao,
        ..Cena::default()
    };
    let mut falhas = vec![];
    // O tecto de degraus por pincel. Filbert Dry 25: a mistura espectral (B32.1, `00f3ca1751`) dá 25 degraus de matiz
    // (> 3° entre linhas) com o verde presente e sem ferrugem — lei aprovada na integração de 13/11 (BUGS_painter #115;
    // handoff 12/11 DESEMPENHO_TOTAL §7). Catraca: o número medido, nunca acima; verde e ferrugem 0 valem para todos.
    for (pincel, pr, degraus_max) in [
        ("Oil/Flat/Flat Bristle", 0.2, 10),
        ("Oil/Flat/Tilted Shade", 0.2, 10),
        ("Oil/Flat/Filbert", 0.2, 10),
        ("Oil/Flat/Filbert Dry", 0.6, 25),
    ] {
        let mut t = pinta(&b, pincel, 600.0, 0.5, pr, 1, 300, cena, &mut |_| {});
        t.wet_recomposite_full();
        let mut m = [0.0f64; 3];
        let mut n = 0.0;
        for y in 132..148 {
            for x in 100..260 {
                let px = &t.canvas_rgba[(x + y * W as usize) * 4..][..3];
                for k in 0..3 {
                    m[k] += f64::from(px[k]);
                }
                n += 1.0;
            }
        }
        let m = m.map(|c| c / n);
        let c = celula(&b, pincel, 600.0, 0.5, pr, 1, 300, cena, None);
        println!(
            "[J7] {pincel} pr{pr}: cruzamento {m:.1?} ferrugem {:.2} degraus {}",
            c.plano.ferrugem, c.plano.degraus_y
        );
        if m[1] <= m[0] || c.plano.ferrugem > 0.0 || c.plano.degraus_y > degraus_max {
            falhas.push(format!(
                "{pincel}: {m:.0?} ferrugem {:.2} degraus {}",
                c.plano.ferrugem, c.plano.degraus_y
            ));
        }
    }
    assert!(
        falhas.is_empty(),
        "o cruzamento tapa em vez de misturar: {falhas:?}"
    );
}
