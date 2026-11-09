//! `gera_rampa` × coloraide 8.14.1 (MIT), e a monotonia em L com o corte de gama só no croma.

use ph2d_color::perceptual::oklch_to_oklab;
use ph2d_pixel_rampa::{
    ConfigRampa, gera_rampa, gera_rampa_oklch, oklab_de, oklch_de, op_rampa_nova, planeada,
};
use serde_json::Value;

fn fixture() -> Value {
    let raw = include_str!("../fixtures/coloraide_rampas.json");
    serde_json::from_str(raw).expect("fixture")
}

fn f(v: &Value) -> f32 {
    v.as_f64().expect("número") as f32
}

fn trio(v: &Value) -> [f32; 3] {
    [f(&v[0]), f(&v[1]), f(&v[2])]
}

fn caso(c: &Value) -> ([u8; 4], ConfigRampa) {
    let b = &c["base_srgb8"];
    let px = |i: usize| b[i].as_u64().expect("byte") as u8;
    let k = &c["config"];
    let cfg = ConfigRampa {
        passos: k["passos"].as_u64().expect("passos") as usize,
        desvio_matiz_graus: f(&k["desvio_matiz_graus"]),
        curva_croma: f(&k["curva_croma"]),
        faixa_l: [f(&k["faixa_l"][0]), f(&k["faixa_l"][1])],
    };
    ([px(0), px(1), px(2), 255], cfg)
}

fn de(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// ΔE OKLab < 1e-3 por degrau, comparado DEPOIS do corte de gama e ANTES dos 8 bits (o
/// arredondamento a 8 bits sozinho chega a ~2e-3 de ΔE). Os 8 bits comparam-se a ±1 por canal.
#[test]
fn a_rampa_gerada_e_a_do_coloraide() {
    let fx = fixture();
    let casos = fx["casos"].as_array().expect("casos");
    assert_eq!(casos.len(), 6, "3 bases × 2 configurações");
    let mut pior = 0.0_f32;
    let mut cortados = 0;
    for c in casos {
        let nome = c["nome"].as_str().expect("nome");
        let (base, cfg) = caso(c);
        let ours = gera_rampa_oklch(oklch_de(base), cfg);
        let bytes = gera_rampa(base, cfg);
        let degraus = c["degraus"].as_array().expect("degraus");
        assert_eq!(ours.len(), degraus.len(), "{nome}");
        for (i, (p, d)) in ours.iter().zip(degraus).enumerate() {
            let lab = oklch_to_oklab([p.l, p.c, p.h / 360.0]);
            let e = de(lab, trio(&d["oklab_na_gama"]));
            pior = pior.max(e);
            assert!(e < 1e-3, "{nome}[{i}]: ΔE OKLab {e:.2e}");
            if f(&d["oklch_na_gama"][1]) < f(&d["oklch_planeado"][1]) - 1e-4 {
                cortados += 1;
            }
            for k in 0..3 {
                let o = d["srgb8"][k].as_i64().expect("byte");
                let diff = (i64::from(bytes[i][k]) - o).abs();
                assert!(diff <= 1, "{nome}[{i}] canal {k}: {} × {o}", bytes[i][k]);
            }
        }
    }
    assert!(
        cortados >= 5,
        "a fixture contém o corte de gama ({cortados} degraus)"
    );
    eprintln!("pior ΔE OKLab {pior:.2e}; {cortados} degraus cortados pelo croma");
}

#[test]
fn a_rampa_gerada_e_monotona_em_l() {
    for c in fixture()["casos"].as_array().expect("casos") {
        let nome = c["nome"].as_str().expect("nome");
        let (base, cfg) = caso(c);
        let plano = planeada(oklch_de(base), cfg);
        let feita = gera_rampa_oklch(oklch_de(base), cfg);
        for (i, (p, q)) in plano.iter().zip(&feita).enumerate() {
            assert_eq!(
                p.l.to_bits(),
                q.l.to_bits(),
                "{nome}[{i}]: o corte mexeu no L"
            );
            assert_eq!(
                p.h.to_bits(),
                q.h.to_bits(),
                "{nome}[{i}]: o corte mexeu na matiz"
            );
            assert!(q.c <= p.c, "{nome}[{i}]: o corte subiu o croma");
        }
        for w in feita.windows(2) {
            assert!(w[1].l > w[0].l, "{nome}: L planeado não sobe");
        }
        // E a rampa gravada (8 bits) continua a subir em L.
        let ls: Vec<f32> = gera_rampa(base, cfg)
            .into_iter()
            .map(|b| oklab_de(b)[0])
            .collect();
        assert!(
            ls.windows(2).all(|w| w[1] > w[0]),
            "{nome}: L dos 8 bits {ls:?}"
        );
    }
}

#[test]
fn a_rampa_nova_entra_na_paleta_num_passo() {
    use ph2d_pixel_core::{Background, ColorMode, NewSprite, Sprite, apply, rampa_de};
    let mut s = Sprite::new(NewSprite {
        width: 8,
        height: 8,
        mode: ColorMode::Indexed,
        background: Background::Transparent,
    })
    .expect("sprite");
    let cfg = ConfigRampa {
        passos: 5,
        desvio_matiz_graus: 20.0,
        curva_croma: 0.6,
        faixa_l: [0.25, 0.92],
    };
    let cores = gera_rampa([0xd9, 0xa0, 0x66, 255], cfg);
    let antes = s.clone();
    let op = op_rampa_nova(&s, "skin", &cores).expect("cabe");
    let inv = apply(&mut s, op).expect("aplica");
    assert_eq!(&s.palette()[32..], &cores[..]);
    assert_eq!(rampa_de(s.ramps(), 34).map(|r| r.1), Some(2));
    apply(&mut s, inv).expect("desfaz");
    assert_eq!(s.palette(), antes.palette());
    assert!(s.ramps().ramps.is_empty());
}
