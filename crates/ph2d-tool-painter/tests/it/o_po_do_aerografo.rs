//! **O pó do Airbrush** pela porta do produto (prompt 43, doc `43_medicao_o_po_fino.md`): as gotas mudam a
//! cada dab (dois traços no mesmo sítio enchem), o pincel gravado leva o pó, e o grão é do PÍXEL — a textura
//! não muda com o tamanho, como no Rebelle (22,7 · 23,4 % a 40 e 80 px).

use super::midias_de_fabrica::de_fabrica;
use super::regua_midia::{RECTO, mede};
use super::sonda_gestos::{AZUL, gesto, pinta_cena_ate, tela};
use ph2d_editor_core::tool::RasterEditTool;

/// A cobertura (`max |px − 255| ÷ 255`) de cada píxel do miolo do recto (y 114..126, x 150..450).
fn miolo(px: &[u8], w: u32) -> Vec<f64> {
    (114..126)
        .flat_map(|y| (150..450).map(move |x| (x, y)))
        .map(|(x, y)| {
            let i = (y * w as usize + x) * 4;
            f64::from(px[i..i + 3].iter().map(|&v| 255 - v).max().unwrap()) / 255.0
        })
        .collect()
}

fn recto(vezes: usize) -> (Vec<u8>, u32) {
    let mut t = tela();
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&de_fabrica("Airbrush")).is_empty());
    t.set_brush_size_px(15.0);
    t.set_brush_color_srgb8(AZUL);
    for _ in 0..vezes {
        gesto(&mut t, &[[60.0, 120.0], [560.0, 120.0]], 3.0, |_| 1.0);
    }
    let (px, w, _) = t.run_full();
    (px, w)
}

/// ⭐ **O segundo traço no MESMO sítio cai noutras gotas**: as gotas sorteiam-se por dab no fluxo do pincel —
/// com a mesma semente o que o segundo traço acrescenta seria o desenho do primeiro (correlação ~1).
#[test]
fn o_segundo_traco_cai_noutras_gotas() {
    let [(um, w), (dois, _)] = std::thread::scope(|s| {
        [1, 2]
            .map(|n| s.spawn(move || recto(n)))
            .map(|h| h.join().unwrap())
    });
    let (a, b) = (miolo(&um, w), miolo(&dois, w));
    let d: Vec<f64> = b.iter().zip(&a).map(|(b, a)| b - a).collect();
    let n = a.len() as f64;
    let (ma, md) = (a.iter().sum::<f64>() / n, d.iter().sum::<f64>() / n);
    let cov = a
        .iter()
        .zip(&d)
        .map(|(x, y)| (x - ma) * (y - md))
        .sum::<f64>()
        / n;
    let va = a.iter().map(|x| (x - ma).powi(2)).sum::<f64>() / n;
    let vd = d.iter().map(|y| (y - md).powi(2)).sum::<f64>() / n;
    let r = cov / (va * vd).sqrt();
    assert!(md > 0.05, "o segundo traço acrescenta tinta: {md:.3}");
    assert!(
        r < 0.3,
        "o segundo traço repete as gotas do primeiro: correlação {r:.3}"
    );
}

/// O pincel gravado leva o seu pó (o mapa `engine`, por nome) e pinta o mesmo traço ao byte.
#[test]
fn um_pincel_gravado_leva_o_seu_po() {
    let mut t = tela();
    let fabrica = de_fabrica("Airbrush");
    assert!(t.wet_preset_aplicar(&fabrica).is_empty());
    let mut base = fabrica.parametros.clone();
    base.engine.remove("dust");
    let (capturado, _) = t.wet_preset_capturar(&base);
    let po = capturado.engine.get("dust").copied();
    assert!(
        po.is_some_and(|d| (d - 0.8).abs() < 1e-9),
        "o pincel gravado tem o pó {po:?}"
    );
}

/// ⭐ **O grão é do píxel**: a textura do Airbrush seco à largura das fotos de 40 e 80 px (26 · 54 px) fica na
/// do Rebelle ± 3,5 % — uma gota que crescesse com o pincel teria o dobro do lado a 80. A foto de 20 px (13 px
/// de largura) lê 28,7 contra 22,5: aberto (handoff P43 §6), fora deste portão.
#[test]
fn o_po_e_do_pixel() {
    const TAMANHOS: [(f32, f64); 2] = [(15.0, 22.70), (31.0, 23.45)];
    let lidas: Vec<f64> = std::thread::scope(|s| {
        TAMANHOS
            .map(|(raio, _)| {
                s.spawn(move || {
                    let mut t = tela();
                    t.set_wet_relogio_fixo(true);
                    assert!(t.wet_preset_aplicar(&de_fabrica("Airbrush")).is_empty());
                    t.set_brush_size_px(raio);
                    let (px, w, _) = pinta_cena_ate(&mut t, 1800);
                    mede(&px, w, RECTO).textura
                })
            })
            .map(|h| h.join().unwrap())
            .to_vec()
    });
    for ((raio, rebelle), nossa) in TAMANHOS.iter().zip(&lidas) {
        assert!(
            (nossa - rebelle).abs() < 3.5,
            "raio {raio}: textura {nossa:.2} contra a do Rebelle {rebelle} (lidas {lidas:.2?})"
        );
    }
}

/// O pó só anda no fluxo do pincel quando está LIGADO: um pincel sem pó cujo grão salta por dab (`Random`, o que
/// lê esse fluxo) pinta a cena de antes ao byte — o ramo do pó não é tomado com `dust` a 0.
#[test]
fn sem_po_o_fluxo_do_pincel_nao_anda() {
    // J2 (03_MOTORES §7 b, a): o traço fino repõe o pigmento por área abaixo de ~29 px (a cena a 20 px pesa 1,05×) e
    // a pressão do Soft Pastel é a da origem no Rebelle. J4 (receita, nenhum motor): `3ea7f0ff32` (a família curada,
    // PressureGrain) e `9965b60b59` (o pó SOLTO, outro knob — `dust` continua ausente); o ficheiro de `77da300cb1`
    // pinta no HEAD o `0xc6d7748f78a789fe` de antes, ao byte (medido 22/10).
    const ANTES: u64 = 0x9b486226bdbd2ada;
    let mut p = de_fabrica("Soft Pastel");
    let tex = p.parametros.brush.get_mut("texture").unwrap();
    tex["mapping"] = serde_json::json!("Random");
    let mut t = tela();
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&p).is_empty());
    t.set_brush_size_px(10.0);
    let (px, _, _) = pinta_cena_ate(&mut t, 60);
    let agora = super::sonda_gestos::fnv1a(&px);
    assert_eq!(agora, ANTES, "a cena mudou: {agora:#018x}");
}
