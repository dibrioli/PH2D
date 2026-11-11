//! **Família `oleo_wash`** — missão Painter 4K B36.1 (decisão 5 do dono, 10/11; PA-45, AP-38): o óleo DILUÍDO que
//! escorre, pela lei única de 01/11 (Paint type Oil, o diluente é líquido). Antes dele o óleo mais diluído da fábrica era
//! o Metallic Flat (Dilution 0,042). Composição só com o que o motor já tem: a pasta `Oil/` dá Paint type Oil e o Real
//! Oil; água 1 com pigmento 234 dá Dilution 0,6 (`receita::migra_para_controlos`: 350·a²·wpd ÷ (p + 350·a²·wpd)); a tela
//! inclinada do Wet Drip faz a tinta correr. Prancha: `docs/Painter/missao_4k/pranchas/B36/`.

use super::oleo_acrilica::FILBERT;
use super::prelude::*;

/// B36.1 (11/11): na fábrica (o ficheiro grava-o `pranchas/B36/regera_oil_wash.sh`, na fila pesada). O óleo diluído ESCORRE pela lei E10g que já
/// existe (o Bingham cai com a água, a mobilidade é `ETA_OLEO^−m` = 1/8): devagar, por isso o gate mede 15 s, não 3.
/// Sonda `diag_o_que_trava_o_oil_wash` (480×270, queda px; Filbert 15): Dilution 0,5 → 22 (10 s) · 28 (15 s); 0,6 → 25 ·
/// 33; 0,9 → 41 · 52; a mesma tinta com Paint type Water 71 · 86. Pigmento 234 ⇒ Dilution 0,599 (o topo da faixa).
const OIL_WASH: Receita = Receita {
    nome: "Oil Wash",
    water: 1.0,
    pigment: 234.0,
    size_px: Some(64.0),
    // A tela inclinada do Wet Drip (o dial para baixo): sem ela o diluente não tem para onde correr.
    tilt: Tilt {
        on: true,
        ring: 8,
        spoke: 3,
    },
    extras: &[
        ("smoothEdges", 0.3),
        // Tinta rala: o relevo e o brilho do Real Oil baixam (a película fina não faz crista).
        ("oilDepth", 0.1),
        ("oilGloss", 0.5),
        ("felt", 0.05),
        ("bristleStrength", 1.2),
        ("bristlePressure", 1.2),
        ("bristleLaneWidth", 4.0),
        ("bristleLift", 0.0),
        ("bristleNoiseSat", 0.3),
        ("bristleNoiseLight", 0.0),
        ("bristleHeight", 0.3),
        ("pigmentSpacing", 0.0),
        ("capsuleMaxPx", 4.0),
        ("pressureSize", 0.2),
        ("pressureSizeCurve", 0.25),
        ("pressureOpacity", 1.0),
        ("pressureOpacityCurve", 2.0),
        ("emboss", 0.1),
        ("embossSizeExp", 0.4),
        ("embossSizeRef", 41.0),
        ("bristleTipRagged", 1.6),
        ("glaze", 0.0),
    ],
    nasce_de: Some("Filbert"),
    ..FILBERT
};

pub const WET: &[Receita] = &[OIL_WASH];

pub const MEIOS: &[ReceitaMeio] = &[];

pub const MIDIAS: &[Midia] = &[];

/// O preset do Oil Wash pela RECEITA (não pelo ficheiro: os gates valem antes e depois da fila pesada o gravar).
fn oil_wash() -> ph2d_brush_presets::Preset {
    ph2d_brush_presets::Preset {
        parametros: crate::presets_de_fabrica::parametros(&OIL_WASH),
        origem: ph2d_brush_presets::Origem::Fabrica,
        miniatura: ph2d_brush_presets::Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![255; 4],
        },
        texturas: Default::default(),
    }
}

/// ⭐ **O Oil Wash é o óleo diluído da fábrica** (B36.1): Paint type Oil e Dilution 0,3–0,6; os outros óleos de
/// fábrica ficam todos abaixo (antes dele o máximo era o Metallic Flat, 0,042).
#[test]
fn o_oil_wash_e_o_oleo_diluido_da_fabrica() {
    let [d, _, m, _] = crate::cerda_e_marca::controlos_migrados(&oil_wash());
    println!("[B36] Oil Wash: Dilution {d:.3} · Paint type {m}");
    assert!((0.3..=0.6).contains(&d), "Dilution {d}");
    assert!((m - 1.0).abs() < 1e-9, "Paint type {m} (1 = Oil)");
    for r in
        crate::receitas_de_fabrica::receitas().filter(|r| r.meio == "Oil" && r.nome != "Oil Wash")
    {
        let p = crate::midias_de_fabrica::de_fabrica(r.nome);
        let [dr, _, mr, _] = crate::cerda_e_marca::controlos_migrados(&p);
        println!("[B36] {}: Dilution {dr:.3} · Paint type {mr}", r.nome);
        assert!(dr < d, "{} mais diluído que o Oil Wash", r.nome);
    }
}

/// A tela `w`×`h` com o fundo `qual` (0 branco · 1 blocos · 2 papel Textured), o traço horizontal de `p` e a
/// tela inclinada (`quadros` de 60 Hz depois do traço): a imagem e a linha do traço.
fn cena_ate(
    p: &ph2d_brush_presets::Preset,
    w: u32,
    h: u32,
    qual: u8,
    quadros: u32,
) -> (Vec<u8>, u32) {
    use ph2d_editor_core::tool::RasterEditTool as _;
    let mut t = ph2d_tool_painter::PainterTool::default();
    let mut fundo = vec![255u8; (w * h * 4) as usize];
    if qual == 1 {
        let cores = [
            [200u8, 60, 50],
            [60, 150, 210],
            [240, 200, 60],
            [90, 170, 90],
        ];
        for (i, px) in fundo.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let (x, y) = (i as u32 % w, i as u32 / w);
            px[..3].copy_from_slice(&cores[usize::from(x * 2 >= w) + 2 * usize::from(y * 2 >= h)]);
        }
    }
    if qual == 2 {
        t.agenda_papel_texturizado(7);
        t.bind_document(7, fundo, w, h);
    } else {
        t.set_source(fundo, w, h);
    }
    t.set_wet_relogio_fixo(true);
    t.set_paint_media(ph2d_tool_painter::PaintMedia::WetPaint);
    assert!(t.wet_preset_aplicar(p).is_empty());
    let s = h as f32 / 1080.0;
    t.set_brush_size_px(64.0 * s);
    t.set_brush_color_srgb8([40, 90, 220]);
    let y = h / 3;
    crate::sonda_gestos::gesto(
        &mut t,
        &[[0.25 * w as f32, y as f32], [0.75 * w as f32, y as f32]],
        3.0 * s,
        |_| 1.0,
    );
    crate::sonda_gestos::passa_o_tempo(&mut t, quadros, crate::sonda_gestos::Relogio::PorQuadro);
    (t.run_full().0, y)
}

/// O tempo do escorrer: 15 s de tela inclinada (o óleo corre a 1/8 da água; a sonda E10g mede também 15 s).
const QUADROS_DO_ESCORRER: u32 = 900;

/// Quantos px abaixo da linha do traço a tinta chega (a coluna do meio; tinta = azul que se afasta do fundo).
fn queda(img: &[u8], w: u32, h: u32, y0: u32) -> u32 {
    let x = w / 2;
    (y0..h)
        .filter(|&y| {
            let i = ((y * w + x) * 4) as usize;
            i32::from(img[i + 2]) - i32::from(img[i]) > 60
        })
        .max()
        .map_or(0, |y| y - y0)
}

/// ⭐ **O Oil Wash escorre** (B36.1, decisão 5): na tela inclinada a tinta desce além do traço; o Filbert (óleo de
/// corpo, a mesma ponta) fica onde se pôs. Red: Dilution 0,5 aos 3 s, 16 × 15 px.
#[test]
#[ignore = "gate lento B36.1 (2 cenas × 15 s de tela): corra por nome com --ignored"]
fn o_oil_wash_escorre_e_o_filbert_nao() {
    let (w, h) = (480, 270);
    let fil = crate::midias_de_fabrica::de_fabrica("Filbert");
    let (a, y) = cena_ate(&fil, w, h, 0, QUADROS_DO_ESCORRER);
    let (b, _) = cena_ate(&oil_wash(), w, h, 0, QUADROS_DO_ESCORRER);
    let (qa, qb) = (queda(&a, w, h, y), queda(&b, w, h, y));
    println!("[B36] queda abaixo do traço (px, 480×270, 15 s): Filbert {qa} · Oil Wash {qb}");
    assert!(
        qb > qa + 8,
        "o Oil Wash não escorre: {qb} contra o Filbert {qa}"
    );
}

/// Prancha B36: Filbert (esquerda) × Oil Wash (direita) nos 3 fundos, 1080p e 4K (recorte do centro).
#[test]
#[ignore = "prancha B36: escreve PNGs em docs/Painter/missao_4k/pranchas/B36/"]
fn prancha_b36_oil_wash() {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/Painter/missao_4k/pranchas/B36"
    );
    let fil = crate::midias_de_fabrica::de_fabrica("Filbert");
    let ow = oil_wash();
    for (w, h, res) in [(1920u32, 1080u32, "1080p"), (3840, 2160, "4k")] {
        for (qual, nome) in [(0u8, "branco"), (1, "blocos"), (2, "papel")] {
            let (a, y) = cena_ate(&fil, w, h, qual, QUADROS_DO_ESCORRER);
            let (b, _) = cena_ate(&ow, w, h, qual, QUADROS_DO_ESCORRER);
            println!(
                "PRANCHA {res} {nome}: queda Filbert {} · Oil Wash {}",
                queda(&a, w, h, y),
                queda(&b, w, h, y)
            );
            let (x0, lw, lh) = (w / 5, w * 3 / 5, h * 2 / 3);
            let gap = 8;
            let mut par =
                image::RgbaImage::from_pixel(lw * 2 + gap, lh, image::Rgba([40, 40, 40, 255]));
            for yy in 0..lh {
                for xx in 0..lw {
                    let i = (((yy + h / 6) * w + x0 + xx) * 4) as usize;
                    let px = |v: &[u8]| image::Rgba([v[i], v[i + 1], v[i + 2], 255]);
                    par.put_pixel(xx, yy, px(&a));
                    par.put_pixel(xx + lw + gap, yy, px(&b));
                }
            }
            par.save(format!("{dir}/{res}_{nome}_filbert_oil_wash.png"))
                .expect("png");
        }
    }
}

/// SONDA B36.1 — o que trava o Oil Wash: cada variante muda UMA coisa (motor `engine` do preset), 3 s e 10 s a 480×270.
#[test]
#[ignore = "sonda B36.1"]
fn diag_o_que_trava_o_oil_wash() {
    let (w, h) = (480, 270);
    let drip = crate::midias_de_fabrica::de_fabrica("Wet Drip");
    let ow = oil_wash();
    let [d, v, _, b] = crate::cerda_e_marca::controlos_migrados(&ow);
    let e10 =
        |dil: f64, meio: f64| crate::cerda_e_marca::com_meio_e_corpo([dil, v, 0.0, 0.0], meio, b);
    let como = |kv: Vec<(String, f64)>| kv.iter().map(|(k, v)| (k.clone(), *v)).collect::<Vec<_>>();
    let variantes: Vec<(String, ph2d_brush_presets::Preset)> = vec![
        ("Oil Wash".into(), ow.clone()),
        ("Oil Wash · Paint type Water".into(), {
            let mut p = ow.clone();
            for (k, x) in como(e10(d, 0.0)) {
                p.parametros.engine.insert(k, x);
            }
            p
        }),
        ("Oil Wash · Dilution 0,6".into(), {
            let mut p = ow.clone();
            for (k, x) in como(e10(0.6, 1.0)) {
                p.parametros.engine.insert(k, x);
            }
            p
        }),
        ("Oil Wash · Dilution 0,9".into(), {
            let mut p = ow.clone();
            for (k, x) in como(e10(0.9, 1.0)) {
                p.parametros.engine.insert(k, x);
            }
            p
        }),
        ("Wet Drip · Paint type Oil".into(), {
            let [dd, vd, _, bd] = crate::cerda_e_marca::controlos_migrados(&drip);
            let mut p = drip.clone();
            for (k, x) in crate::cerda_e_marca::com_meio_e_corpo([dd, vd, 0.0, 0.0], 1.0, bd) {
                p.parametros.engine.insert(k, x);
            }
            p
        }),
    ];
    let fil = crate::midias_de_fabrica::de_fabrica("Filbert");
    for q in [600u32, 900] {
        let (a, y) = cena_ate(&fil, w, h, 0, q);
        println!(
            "[B36 sonda] {q} quadros · Filbert: queda {}",
            queda(&a, w, h, y)
        );
        for (nome, p) in &variantes {
            let (img, y) = cena_ate(p, w, h, 0, q);
            let c = crate::cerda_e_marca::controlos_migrados(p);
            println!(
                "[B36 sonda] {q} quadros · {nome} (Dilution {:.3} · Paint type {}): queda {}",
                c[0],
                c[2],
                queda(&img, w, h, y)
            );
        }
    }
}
