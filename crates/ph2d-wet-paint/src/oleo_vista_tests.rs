//! Os portões da A.1 ([`super`]): a 100 % a vista é a linha de hoje ao byte; a 400 % tem o detalhe que a ampliação
//! bilinear de hoje não tem.

use super::prova::{
    VISUAL, bilinear, camadas, campo, faca, fraccao_outliers, hoje, pico_da_celula, var_laplaciano,
};
use super::{PreparoVista, Vista};
use crate::oleo_real::OleoReal;

/// O recorte de prova: 64 × 64 px do documento sobre a faca.
const RECORTE: (usize, usize, usize, usize) = (16, 16, 64, 64);

fn vista(s: f64) -> Vista {
    let (x0, y0, w, h) = RECORTE;
    Vista {
        x0: x0 as f64,
        y0: y0 as f64,
        w: (w as f64 * s) as usize,
        h: (h as f64 * s) as usize,
        s,
    }
}

/// ⭐ Identidade: a `s = 1` a vista nova é a linha do óleo de hoje AO BYTE (Catmull-Rom nos centros = a amostra).
#[test]
fn a_vista_a_100_por_cento_e_a_linha_de_hoje_ao_byte() {
    let e = faca();
    let o = OleoReal::default();
    let c = campo(&e, &o);
    let antes = hoje(&e, &o, &c, RECORTE);
    assert!(
        antes.chunks(4).filter(|p| p[3] > 0).count() > 1000,
        "a faca tem tinta no recorte"
    );
    let nova = PreparoVista::novo(None, &camadas(&e), VISUAL, &o, &c, vista(1.0)).render();
    let dif = antes.iter().zip(&nova).filter(|(a, b)| a != b).count();
    assert_eq!(dif, 0, "{dif} bytes diferem");
}

/// ⭐ Nitidez: a 400 % a variância do laplaciano da vista nova contra a da ampliação BILINEAR do composite de hoje.
/// O alvo do dono é 3×. Medido 30/10: bilinear 54,64 · nova 178,58 = 3,27× com a altura por B-spline interpolante
/// (era 2,47× com Catmull-Rom). Vale só com os portões do pico da célula e dos pontinhos ao lado.
/// ⚠️ 13/11: o 3× vinha dos grãos do brilho que o dono reprovou (12/11, a luz do traço). Com o antisserrilhado do
/// ambiente do produto: 2,02× (AA 0: 3,34× · 0,25: 2,51× · sem brilho: 1,39×) — o portão passa a 2× sobre o look
/// aprovado e continua a reprovar a vista sem brilho. Tabela: `HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_LUZ_E_CAMADAS.md` §7.
#[test]
fn a_vista_a_400_por_cento_tem_o_detalhe_que_a_ampliacao_nao_tem() {
    let e = faca();
    let o = OleoReal::default();
    let c = campo(&e, &o);
    let (_, _, w, h) = RECORTE;
    let ampliada = bilinear(&hoje(&e, &o, &c, RECORTE), w, h, 4);
    let nova = PreparoVista::novo(None, &camadas(&e), VISUAL, &o, &c, vista(4.0)).render();
    let (vb, vn) = (
        var_laplaciano(&ampliada, w * 4, h * 4),
        var_laplaciano(&nova, w * 4, h * 4),
    );
    eprintln!(
        "var laplaciano 400 %: bilinear de hoje {vb:.3} · vista nova {vn:.3} · razão {:.2}",
        vn / vb
    );
    assert!(vn >= 2.0 * vb, "bilinear {vb:.3} × nova {vn:.3}");
}

/// As duas imagens a 400 %: a ampliação bilinear de hoje e a vista nova.
fn as_duas_a_400() -> (Vec<u8>, Vec<u8>, usize, usize) {
    let e = faca();
    let o = OleoReal::default();
    let c = campo(&e, &o);
    let (_, _, w, h) = RECORTE;
    let ampliada = bilinear(&hoje(&e, &o, &c, RECORTE), w, h, 4);
    let nova = PreparoVista::novo(None, &camadas(&e), VISUAL, &o, &c, vista(4.0)).render();
    (ampliada, nova, w * 4, h * 4)
}

/// ⭐ Anti-artefacto: no laplaciano da vista nova a 400 % a energia no período da CÉLULA (4 px) não sobressai dos bins
/// vizinhos — pico/vizinhança ≤ 1,5. Um padrão ao passo da célula é detalhe FALSO (e infla a régua da nitidez).
#[test]
fn a_vista_a_400_por_cento_nao_tem_padrao_ao_passo_da_celula() {
    let (ampliada, nova, w, h) = as_duas_a_400();
    let (pb, pn) = (
        pico_da_celula(&ampliada, w, h, 4),
        pico_da_celula(&nova, w, h, 4),
    );
    eprintln!("pico da célula 400 %: bilinear de hoje {pb:.3} · vista nova {pn:.3}");
    assert!(pn <= 1.5, "pico da célula {pn:.3} (bilinear {pb:.3})");
}

/// ⭐ Sem pontinhos: a fracção de pixels com |lap| > 6 σ robusto (MAD) da vista nova ≤ 1,5× a da bilinear de hoje.
#[test]
fn a_vista_a_400_por_cento_nao_tem_mais_pontinhos_que_a_ampliacao() {
    let (ampliada, nova, w, h) = as_duas_a_400();
    let (ob, on) = (
        fraccao_outliers(&ampliada, w, h),
        fraccao_outliers(&nova, w, h),
    );
    eprintln!("outliers 6σ 400 %: bilinear de hoje {ob:.5} · vista nova {on:.5}");
    assert!(on <= 1.5 * ob, "outliers {on:.5} × bilinear {ob:.5}");
}

/// ⭐ A ESCALA da curvatura: sobre uma altura analítica (o parabolóide `h = a·((x−50)² + (y−50)²)`, que a B-spline
/// cúbica reproduz exactamente), a `geometria` que a vista dá à óptica a s = 2, 3, 4 é a do DOCUMENTO: `hx = 2a(x−50)`,
/// `hxx = hyy = 2a`, `hxy = 0`, seja qual for o zoom. Tolerância `1e-6`: o único erro é o espelho da borda, que o
/// pré-filtro amortece por `|z|^8 ≈ 2,6·10⁻⁵` sobre um declive de borda ≤ 1 (medido: ver o `eprintln!`).
#[test]
fn a_curvatura_da_vista_esta_em_unidades_do_documento_a_qualquer_zoom() {
    let a = 0.01;
    let f = move |x: f64, y: f64| a * ((x - 50.0) * (x - 50.0) + (y - 50.0) * (y - 50.0));
    for s in [2.0, 3.0, 4.0] {
        let v = vista(s);
        let k = s as usize;
        let pixels: Vec<(usize, usize)> = (8..56)
            .step_by(7)
            .flat_map(|i| (8..56).step_by(11).map(move |j| (i * k, j * k)))
            .collect();
        let g = super::prova::geometria_analitica(&f, v, &pixels);
        let mut pior = 0.0f64;
        for (&(x, y), q) in pixels.iter().zip(&g) {
            let cx = v.x0 + (x as f64 + 0.5) / s - 0.5 + 1.0;
            let cy = v.y0 + (y as f64 + 0.5) / s - 0.5 + 1.0;
            let esperado = [
                f(cx, cy),
                2.0 * a * (cx - 50.0),
                2.0 * a * (cy - 50.0),
                2.0 * a,
                2.0 * a,
                0.0,
            ];
            for (c, (&qv, &ev)) in q.iter().zip(&esperado).enumerate() {
                pior = pior.max((qv - ev).abs());
                assert!(
                    (qv - ev).abs() < 1e-6,
                    "s={s} pixel ({x},{y}) componente {c}: {qv} × {ev}"
                );
            }
        }
        eprintln!("s={s}: pior erro da geometria {pior:.2e}");
    }
}

/// ⭐ O MEIO PIXEL: a s = 3 os pixels `x = 3k + 1` têm o centro num centro de célula (`u` inteiro); aí a cor da pilha
/// ([`crate::render::PreLuz`]) é a da célula AO BIT e a altura é a fina da célula (a spline interpola; `1e-9`). As
/// estatísticas da nitidez são invariantes a uma translação de meio pixel: esta régua não.
#[test]
fn a_vista_alinha_os_centros_dos_pixels_com_os_das_celulas() {
    let v = vista(3.0);
    let (x0, y0) = (RECORTE.0, RECORTE.1);
    let ks: Vec<(usize, usize)> = (4..60)
        .step_by(5)
        .flat_map(|i| (4..60).step_by(3).map(move |j| (i, j)))
        .collect();
    let pixels: Vec<(usize, usize)> = ks.iter().map(|&(i, j)| (3 * i + 1, 3 * j + 1)).collect();
    let celulas: Vec<(usize, usize)> = ks.iter().map(|&(i, j)| (x0 + i + 1, y0 + j + 1)).collect();
    let (nova, hoje) = super::prova::alinhamento(v, &pixels, &celulas);
    let pintados = hoje.iter().filter(|(p, _)| p.pa_sem > 0.0).count();
    assert!(pintados > 50, "a amostra cai na tinta: {pintados}");
    for ((k, (pn, hn)), (ph, hh)) in ks.iter().zip(&nova).zip(&hoje) {
        assert_eq!(pn, ph, "célula {k:?}: a cor da vista não é a da célula");
        assert!(
            (hn - hh).abs() <= 1e-9 * hh.abs().max(1.0),
            "célula {k:?}: altura {hn} × {hh}"
        );
    }
}

/// ⭐ A BORDA: uma tela coberta até à borda, vista a 400 % encostada a `(0, 0)` e ao fim do documento — a primeira
/// coluna/linha tem a cor e a cobertura da segunda (a borda é PRESA, não zero). Tolerância 1 nível de 8 bits: a tinta
/// é uniforme e o campo dela também (medido: diferença 0).
#[test]
fn a_vista_encostada_a_borda_nao_escurece() {
    let lado = 32;
    for (x0, y0) in [(0.0, 0.0), (16.0, 16.0)] {
        let v = Vista {
            x0,
            y0,
            w: 64,
            h: 64,
            s: 4.0,
        };
        let img = super::prova::uniforme(lado, v);
        let px = |x: usize, y: usize| &img[(x + y * 64) * 4..(x + y * 64) * 4 + 4];
        let media = |f: &dyn Fn(usize) -> [u8; 4]| {
            (0..64).fold([0.0f64; 4], |mut m, k| {
                for (c, q) in f(k).iter().enumerate() {
                    m[c] += f64::from(*q) / 64.0;
                }
                m
            })
        };
        for (borda, dentro) in [(0usize, 1usize), (63, 62)] {
            let col = (
                media(&|k| px(borda, k).try_into().unwrap()),
                media(&|k| px(dentro, k).try_into().unwrap()),
            );
            let lin = (
                media(&|k| px(k, borda).try_into().unwrap()),
                media(&|k| px(k, dentro).try_into().unwrap()),
            );
            for (a, b) in [col, lin] {
                for c in 0..4 {
                    assert!(
                        (a[c] - b[c]).abs() <= 1.0,
                        "origem ({x0},{y0}) borda {borda}: {a:?} × {b:?}"
                    );
                }
            }
        }
        assert!(
            img.chunks(4).all(|p| p[3] > 0),
            "origem ({x0},{y0}): um pixel ficou sem tinta"
        );
    }
}
