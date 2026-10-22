//! **O grão do lápis** (JANELA 2, `docs/Painter/Pinceis/03_MOTORES.md` §7.2): o grão `Tiled` é o PAPEL, preso à tela —
//! o mesmo a todo tamanho de ponta, o Size do artista age —, e o dos lápis de fábrica é FINO (o Rebelle: ~1–2 px a 5 ·
//! 40 · 200 px). A régua do «colar de contas» (`leis_escala.py::recto`, o `cv`) e a sonda da grelha.

use super::as_leis_da_escala::{com, mediana, rebelle, recto, tamanhos, tela_com, traco};
use super::exame_bancada::{Pincel, pinceis};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::RasterEditTool;

/// O «colar de contas» AO LONGO do recto de `y` (`leis_escala.py::recto`, o `cv`, ao bit): a cobertura média do miolo
/// coluna a coluna (o miolo = a ±0,35 da largura mediana em volta do centro de cada coluna), desvio ÷ média — e a
/// autocorrelação dessa série a 1 px (o tamanho do grão AO LONGO: o Rebelle ~0,5 nos lápis macios, ~0,24 nos duros).
pub(super) fn cv_do_recto(px: &[u8], y: usize, h: usize) -> (f64, f64, f64) {
    let c = |x: usize, yy: usize| {
        let i = (yy * 1000 + x) * 4;
        (0..3)
            .map(|k| 255.0 - f64::from(px[i + k]))
            .fold(0.0, f64::max)
            / 255.0
    };
    let cols: Vec<Vec<f64>> = (148..852)
        .map(|x| (y - h..=y + h).map(|yy| c(x, yy)).collect())
        .collect();
    let (mut larg, mut cen) = (Vec::new(), Vec::new());
    for col in &cols {
        let m = col.iter().copied().fold(0.0, f64::max);
        if m < 0.08 {
            larg.push(0.0);
            cen.push(0.0);
            continue;
        }
        let a = col.iter().position(|&v| v >= 0.25 * m).unwrap();
        let b = col.iter().rposition(|&v| v >= 0.25 * m).unwrap();
        larg.push((b - a + 1) as f64);
        let s: f64 = col.iter().sum();
        let o: f64 = col
            .iter()
            .enumerate()
            .map(|(i, v)| (i as f64 - h as f64) * v)
            .sum();
        cen.push(o / s.max(1e-9));
    }
    let banda = (0.35 * mediana(larg)).max(0.5);
    let serie: Vec<f64> = cols
        .iter()
        .zip(&cen)
        .map(|(col, &ce)| {
            let v: Vec<f64> = col
                .iter()
                .enumerate()
                .filter(|&(i, _)| (i as f64 - h as f64 - ce).abs() <= banda)
                .map(|(_, &v)| v)
                .collect();
            if v.is_empty() {
                0.0
            } else {
                v.iter().sum::<f64>() / v.len() as f64
            }
        })
        .collect();
    let n = serie.len() as f64;
    let m = serie.iter().sum::<f64>() / n;
    let var = serie.iter().map(|v| (v - m).powi(2)).sum::<f64>() / n;
    let ac1 = serie
        .windows(2)
        .map(|w| (w[0] - m) * (w[1] - m))
        .sum::<f64>()
        / (n - 1.0)
        / var.max(1e-12);
    (var.sqrt() / m.max(1e-6), ac1, m)
}

/// Os 7 lápis/carvão de fábrica e a origem de cada um no Rebelle (`fixtures/leis_escala_rebelle.json`).
const LAPIS: [(&str, &str); 7] = [
    ("Graphite 8B", "PENCIL/graphite_pencil_8b"),
    ("Graphite HB", "PENCIL/graphite_pencil_hb"),
    ("Graphite 2H", "PENCIL/graphite_pencil_2h"),
    ("Graphite 2B", "PENCIL/graphite_pencil_2b"),
    ("Colored 4B", "PENCIL/colored_pencil_4b_color"),
    ("Charcoal", "PENCIL/charcoal_charcoal"),
    ("Scratch", "PENCIL/scratches_scratch"),
];

/// O pincel `nome` de fábrica com as variantes `kv` ([`com`]) e o Size do grão posto a `s` (`None` = o de fábrica).
/// Duas chaves da sonda além das de [`com`]: `rough=x` (a Roughness do grão, `params[3]`) e `gx=x` (o `paperGate`
/// do preset × `x`).
fn com_grao(nome: &str, s: Option<f64>, kv: &str) -> Pincel {
    let (meus, resto): (Vec<&str>, Vec<&str>) = kv
        .split(',')
        .filter(|x| !x.is_empty())
        .partition(|x| x.starts_with("rough=") || x.starts_with("gx="));
    let mut p = com(nome, &resto.join(","));
    if let super::exame_bancada::Fonte::Preset { preset, .. } = &mut p.fonte {
        let t = preset.parametros.brush.get_mut("texture").unwrap();
        if let Some(s) = s {
            t["size"] = serde_json::json!([s, s]);
        }
        for m in meus {
            let (k, v) = m.split_once('=').unwrap();
            let v: f64 = v.parse().unwrap();
            if k == "rough" {
                t["params"][3] = serde_json::json!(v); // os knobs do Noise começam em 2 (0/1 = contraste/brilho)
            } else {
                *preset.parametros.engine.get_mut("paperGate").unwrap() *= v;
            }
        }
    }
    p
}

/// SONDA (03_MOTORES §7.2): o cv e o maior buraco do recto de 5 px dos 7 lápis com o Size do grão de
/// `PH2D_SONDA_SIZES` (`,`; `-` = o de fábrica), e a autocorrelação do grão amostrado píxel a píxel (lag 1 · 2 · 3).
#[test]
#[ignore = "sonda: o grão fino dos lápis por Size; corre-se à mão"]
fn sonda_o_grao_fino_por_size() {
    use ph2d_painter_brush::texture::{
        TextureKind, TextureMapping, TextureSettings, sample_tiled_rot,
    };
    let sizes = std::env::var("PH2D_SONDA_SIZES").unwrap_or_else(|_| "-".to_owned());
    let sizes: Vec<(Option<f64>, String)> = sizes
        .split(',')
        .map(|s| {
            let (s, kv) = s.split_once(':').unwrap_or((s, ""));
            (s.parse().ok(), kv.replace(';', ","))
        })
        .collect();
    for s in sizes.iter().filter_map(|v| v.0) {
        let mut t = TextureSettings {
            kind: TextureKind::Noise,
            mapping: TextureMapping::Tiled,
            params: [0.5; ph2d_painter_brush::texture::MAX_TEX_PARAMS],
            ..TextureSettings::default()
        };
        t.size = [s as f32; 2];
        let v: Vec<f64> = (0..4000)
            .map(|x| f64::from(sample_tiled_rot(&t, x, 7, None, [1.0, 0.0])))
            .collect();
        let m = v.iter().sum::<f64>() / v.len() as f64;
        let var = v.iter().map(|a| (a - m).powi(2)).sum::<f64>() / v.len() as f64;
        let ac = |k: usize| {
            v.windows(k + 1)
                .map(|w| (w[0] - m) * (w[k] - m))
                .sum::<f64>()
                / (v.len() - k) as f64
                / var
        };
        println!(
            "[campo] Size {s}: período {:.2} px, ac 1·2·3 {:.2} {:.2} {:.2}",
            256.0 / s,
            ac(1),
            ac(2),
            ac(3)
        );
    }
    let casos: Vec<(&str, &str, Option<f64>, &str)> = LAPIS
        .iter()
        .flat_map(|&(n, o)| sizes.iter().map(move |(s, kv)| (n, o, *s, kv.as_str())))
        .collect();
    let linhas: Vec<String> = std::thread::scope(|sc| {
        let hs: Vec<_> = casos
            .iter()
            .map(|&(n, o, s, kv)| {
                sc.spawn(move || {
                    let px = tamanhos(&com_grao(n, s, kv));
                    let (_, buraco) = recto(&px, 70, 40);
                    let [a, b, c] = [(70, 40), (190, 70), (440, 130)].map(|(y, h)| cv_do_recto(&px, y, h));
                    format!(
                        "[fino] {n:<12} Size {:<5} {kv:<14} cv·ac·cob 5 {:.3}·{:.2}·{:.2} 40 {:.3}·{:.2}·{:.2} 200 {:.3}·{:.2}·{:.2} | Rebelle cv {:.3} {:.3} {:.3} cob {:.2} {:.2} | buraco {buraco}",
                        s.map_or("-".to_owned(), |s| s.to_string()),
                        a.0, a.1, a.2, b.0, b.1, b.2, c.0, c.1, c.2,
                        rebelle(o, "recto_5", "cv"),
                        rebelle(o, "recto_40", "cv"),
                        rebelle(o, "recto_200", "cv"),
                        rebelle(o, "recto_5", "cob"),
                        rebelle(o, "recto_40", "cob"),
                    )
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for l in linhas {
        println!("{l}");
    }
}

/// ⭐⭐ **O traço do lápis tem o grão do Rebelle** (03_MOTORES §7.2): nos 6 lápis/carvão de fábrica com origem no
/// Rebelle, o colar de contas a 40 px a ±35 % do da origem, a 5 px até 2× (o 8B 1,1×; o resto dos duros é o vale da
/// Textura da mídia, que pede o grão de 4 px) e o tamanho do grão a 5 px (a autocorrelação a 1 px do colar) até 0,6
/// acima da origem. Antes (o Noise 16–64 e a lei pelo diâmetro): a 40 px 1,4–2,4× o do Rebelle.
#[test]
fn o_traco_fino_do_lapis_tem_o_grao_do_rebelle() {
    type Medida<'a> = (&'a str, [(f64, f64, f64); 2], [f64; 3]);
    let r: Vec<Medida> = std::thread::scope(|s| {
        let hs: Vec<_> = LAPIS[..6]
            .iter()
            .map(|&(n, o)| {
                s.spawn(move || {
                    let px = tamanhos(&pinceis(n).remove(0));
                    let reb = [
                        rebelle(o, "recto_5", "cv"),
                        rebelle(o, "recto_40", "cv"),
                        AC5_DO_REBELLE.iter().find(|(k, _)| *k == o).unwrap().1,
                    ];
                    (
                        n,
                        [cv_do_recto(&px, 70, 40), cv_do_recto(&px, 190, 70)],
                        reb,
                    )
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut falhas = Vec::new();
    for (n, [a, b], [r5, r40, ac]) in &r {
        println!(
            "[grão] {n:<12} cv 5 {:.3}/{r5:.3} · 40 {:.3}/{r40:.3} · ac 5 {:.2}/{ac:.2}",
            a.0, b.0, a.1
        );
        if a.1 > ac + 0.6 || (b.0 / r40 - 1.0).abs() > 0.35 || a.0 > 2.0 * r5 {
            falhas.push(*n);
        }
    }
    assert!(falhas.is_empty(), "o grão não é o do Rebelle: {falhas:?}");
}

/// A autocorrelação a 1 px do colar de contas do recto de 5 px de cada origem (a foto A da bateria; a mesma régua,
/// `leis_escala.py::recto` + o lag 1 da série — medida a 20/10).
const AC5_DO_REBELLE: [(&str, f64); 6] = [
    ("PENCIL/graphite_pencil_8b", 0.50),
    ("PENCIL/graphite_pencil_hb", 0.24),
    ("PENCIL/graphite_pencil_2h", 0.24),
    ("PENCIL/graphite_pencil_2b", 0.52),
    ("PENCIL/colored_pencil_4b_color", 0.57),
    ("PENCIL/charcoal_charcoal", 0.51),
];

/// O perfil da cobertura ao longo da linha `y` (colunas 150..850) do recto `recto_40` da bateria pintado por `p` com
/// o diâmetro `d`.
fn perfil(p: &Pincel, d: f32) -> Vec<f64> {
    let mut t = tela_com(p, d);
    traco(
        &mut t,
        &super::as_leis_da_escala::pontos("tamanhos", "recto_40"),
    );
    for _ in 0..120 {
        t.on_tick(1000.0 / 60.0);
    }
    let px = t.run_full().0;
    (150..850)
        .map(|x| {
            let i = (190 * 1000 + x) * 4;
            (0..3)
                .map(|k| 255.0 - f64::from(px[i + k]))
                .fold(0.0, f64::max)
                / 255.0
        })
        .collect()
}

fn correlacao(a: &[f64], b: &[f64]) -> f64 {
    let m = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let (ma, mb) = (m(a), m(b));
    let cov: f64 = a.iter().zip(b).map(|(x, y)| (x - ma) * (y - mb)).sum();
    let va: f64 = a.iter().map(|x| (x - ma).powi(2)).sum();
    let vb: f64 = b.iter().map(|y| (y - mb).powi(2)).sum();
    cov / (va * vb).sqrt().max(1e-12)
}

/// ⭐⭐ **O grão `Tiled` é o PAPEL, preso à tela** (03_MOTORES §7.2; a revisão da J2): dois traços do 8B com um grão
/// GROSSO (Size 4, período 64 px) a 8 e a 40 px de diâmetro, na mesma linha, vêem os mesmos vales (o perfil ao longo
/// do eixo correlaciona ≥ 0,6) — e o Size do artista AGE (o Size 8 a 8 px muda o perfil: ≤ 0,6 contra o Size 4).
/// A lei de antes subia o Size até o período caber 3× no diâmetro (`3·256/D`): a 8 px os dois Sizes viravam 96 — o
/// mesmo papel entre eles e outro que o do traço de 40 px.
#[test]
fn o_grao_e_o_papel_preso_a_tela() {
    let [fino, largo, outro] = std::thread::scope(|s| {
        [(4.0, 8.0), (4.0, 40.0), (8.0, 8.0)]
            .map(|(size, d)| s.spawn(move || perfil(&com_grao("Graphite 8B", Some(size), ""), d)))
            .map(|h| h.join().unwrap())
    });
    let (mesmo, slider) = (correlacao(&fino, &largo), correlacao(&fino, &outro));
    println!("[papel] Size 4: 8 px × 40 px {mesmo:.2} · a 8 px Size 4 × Size 8 {slider:.2}");
    assert!(
        mesmo >= 0.6,
        "o papel muda com o tamanho da ponta: {mesmo:.2}"
    );
    assert!(
        slider <= 0.6,
        "o Size do grão não age na ponta fina: {slider:.2}"
    );
}
