//! **A lei da tinta por pincel** (prompt 42, doc [42_medicao](../../../../docs/Painter/NewFeaturesWetPaint/42_medicao_a_lei_da_tinta.md)).
//!
//! O portão de IDENTIDADE nasceu antes da lei: a cena da bancada (`sonda_gestos::pinta_cena_ate`, o
//! cruzamento incluído) de cada pincel de fábrica, ao byte, como o motor a pintava em `f46fcffb2a` — e duas
//! com as caixas da OBRA ligadas. Um pincel sem Glaze pinta a lei da obra, ao byte.

use super::midias_de_fabrica::de_fabrica;
use super::sonda_gestos::{fnv1a, pinta_cena_ate, tela};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_tool_painter::ids;

/// O raio das cenas de identidade: o do recto da bancada a meia escala dos presets.
const RAIO_DA_CENA: f32 = 18.0;

/// `(preset, caixas da obra ligadas, fnv1a da cena)` — pinados em `f46fcffb2a`; os 5 de tinta e marcador
/// (prompt 42) em `ddc782e9f0`, antes do pó (prompt 43); o Airbrush em `4fa48c25df`, antes das formas (prompt 44), e
/// refeito por ORDEM DO DONO no smoke do 44 (a pressão do Spray 2: esmaece, não estreita); o Wet Flat e a Gouache
/// Flat pinados com a lei de antes do prompt 45 (a dos 20 de cima, ao bit, `a5134220e5`); os três com a pressão →
/// opacidade refeitos pela lei do 45 por ORDEM DO DONO (o Airbrush «como no Rebelle», a rampa dos Flats).
#[rustfmt::skip]
const CENAS: [(&str, bool, u64); 22] = [
    ("Wet Round", false, 0x8e37a39e8ee2ba79),
    ("Wet Wash", false, 0xb08be15433107c85),
    ("Wet Dry Brush", false, 0xf45f7dcaa32eb88e),
    ("Wet Drip", false, 0x6ad975ec85398727),
    ("Wet Liner", false, 0x859efc54fe6e2c43),
    // Os dois com os FIOS da W2 (prompt 48, doc 48_medicao): repinados pela lei nova; os outros 20 ao bit.
    // O Oil Bristle repintado pelo relevo sem escada (W3b, prompt 50, doc 50_medicao): os outros 21 ao bit.
    // Os 22 repintados pela regra do prompt 51 (Spacing 0,1 + Continuous em todos, doc 51_medicao); o Airbrush também
    // pela curva da pressão no tamanho que casa as pontas do Rebelle (ordem do dono 19/10).
    // J2 (03_MOTORES §7 a): a pressão de cada preset passou a ser a da sua origem no Rebelle (a cena tem a rampa).
    ("Oil Bristle", false, 0xcd20d9a3853d6fb1), // J2 §7 c/d: o relevo pelo diâmetro, a parede, a ponta (era 0x939fc3665324de0d)
    ("Acrylic Round", false, 0xb6475854432e61f0),
    ("Express Oil", false, 0x6ae408041d1e042f),
    ("Graphite 8B", false, 0x4b28ef0af7f2eb9f), // J3 3.ª ronda: a 2.ª ponta à frente (no arco), a orla 0,8 × 1, os tufos, a carga 280 e o papel 0,4 (era 0x0aed0cb29386b748)
    ("Charcoal", false, 0x319febeb25876cd8), // J2 §7.2: o grão fino (era 0xd747450876d66163)
    ("Soft Pastel", false, 0x6d2510034bf7bea2),
    ("Oil Pastel", false, 0x53088992b4918139),
    ("Ink Bullet", false, 0xd3ea3f5b4f847521),
    ("Dip Pen", false, 0x2785d41ac04f93a1),
    ("Ink Brush", false, 0x5856e27ccfe8f621),
    ("Marker Bullet", false, 0xb30ddf5eaa30b6b8),
    ("Marker Liner", false, 0xb7568999a13fb6ca),
    ("Airbrush", false, 0xc9c61bd3db4a244e),
    ("Wet Flat", false, 0x270ceb734f4d9aea),
    ("Gouache Flat", false, 0x6111ea627d99cff5),
    ("Wet Round", true, 0xab51e31b5a6e8cb0),
    ("Wet Liner", true, 0xc2fd88e64e7098c4),
];

fn cena(preset: &str, obra: bool) -> u64 {
    let mut t = tela();
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&de_fabrica(preset)).is_empty());
    t.set_brush_size_px(RAIO_DA_CENA);
    if obra {
        for id in [ids::WET_TUNING_KM_MIXING, ids::WET_TUNING_KM_GLAZE] {
            t.handle_panel_event(PanelEvent::Click(id));
        }
    }
    let (px, _, _) = pinta_cena_ate(&mut t, 480);
    fnv1a(&px)
}

#[test]
fn sem_glaze_cada_preset_de_fabrica_pinta_a_cena_de_antes() {
    let hoje: Vec<u64> = std::thread::scope(|s| {
        let h: Vec<_> = CENAS
            .iter()
            .map(|&(p, obra, _)| s.spawn(move || cena(p, obra)))
            .collect();
        h.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mudaram: Vec<String> = CENAS
        .iter()
        .zip(&hoje)
        .filter(|((_, _, antes), agora)| antes != *agora)
        .map(|((p, obra, antes), agora)| {
            format!("(\"{p}\", {obra}, {agora:#018x}), // era {antes:#018x}")
        })
        .collect();
    assert!(
        mudaram.is_empty(),
        "a cena mudou — a lei da obra deixou de ser a de antes:\n{}",
        mudaram.join("\n")
    );
}

/// A régua do cruzamento do 18 §2 (`ferramentas/midias_regua.py::cruzamento`): a luminância (luz linear) do
/// miolo do cruzamento ÷ a menor das duas tintas fora dele — `< 1` escurece, `~1` o de cima tapa.
pub fn cruzamento(px: &[u8], w: u32) -> f64 {
    let lin = |v: u8| {
        let c = f64::from(v) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let lum = |x0: usize, x1: usize| {
        let mut s = [0.0; 3];
        let mut n = 0.0;
        for y in 474..487 {
            for x in x0..x1 {
                let i = (y * w as usize + x) * 4;
                for (c, v) in s.iter_mut().enumerate() {
                    *v += f64::from(px[i + c]);
                }
                n += 1.0;
            }
        }
        let [r, g, b] = s.map(|v| lin((v / n).round().clamp(0.0, 255.0) as u8));
        0.2126 * r + 0.7152 * g + 0.0722 * b
    };
    lum(294, 307) / lum(800, 860).min(lum(120, 200)).max(1e-6)
}

/// O Wet Liner (duro, cheio) com o Glaze do pincel `g` — a mesma receita, só a lei muda.
fn liner_com_glaze(g: f64) -> ph2d_brush_presets::Preset {
    let mut p = de_fabrica("Wet Liner");
    p.parametros.engine.insert("glaze".to_owned(), g);
    p
}

fn cena_com(p: &ph2d_brush_presets::Preset, quadros: u32) -> (Vec<u8>, u32) {
    let mut t = tela();
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.set_brush_size_px(RAIO_DA_CENA);
    let (px, w, _) = pinta_cena_ate(&mut t, quadros);
    (px, w)
}

/// ⭐ **O Glaze do pincel escurece o cruzamento, graduado** (doc 42 §3: Rebelle 0,12–0,30 a tinta, 0,016 o
/// marcador; o nosso tapava — 1,00). Vermelho antes: sem a lei (`vela` = 0 no pouso) os três dão o de Glaze 0.
#[test]
fn o_glaze_do_pincel_escurece_o_cruzamento() {
    let [c0, c5, c1] = std::thread::scope(|s| {
        [0.0, 0.5, 1.0]
            .map(|g| {
                s.spawn(move || {
                    let (px, w) = cena_com(&liner_com_glaze(g), 1800);
                    cruzamento(&px, w)
                })
            })
            .map(|h| h.join().unwrap())
    });
    assert!(
        c0 > 0.7,
        "sem Glaze, a lei da obra (o Wet Liner é aquarela): {c0:.3}"
    );
    assert!(
        c1 < 0.35,
        "Glaze 1 multiplica as camadas: {c1:.3} (medido 0,221 no Wet Liner, aquarela; os pincéis de tinta \
         e marcador, mais carregados, chegam a 0,04–0,06 — Rebelle 0,016)"
    );
    assert!(c1 < c5 && c5 < c0, "graduado: {c0:.3} > {c5:.3} > {c1:.3}");
}

/// **O traço sozinho não muda com o Glaze** — sobre o papel limpo a tinta não filtra nada, e a parte do próprio
/// traço que seca não é outra camada (medido: com o Glaze em toda a tinta o recto escurecia, cobertura 0,90 →
/// 0,93). Os traços AZUIS da bancada (sem o vermelho que os cruza), a água corrida: Glaze 1 = Glaze 0 ao byte.
/// (Com o cruzamento, a linha 3 inteira muda até 24 níveis: o azul que assenta debaixo do vermelho muda a água
/// dela — é o cruzamento, não o traço.)
#[test]
fn o_traco_sozinho_nao_muda_com_o_glaze() {
    use super::sonda_gestos::{AZUL, gesto};
    let [a, b] = std::thread::scope(|s| {
        [0.0, 1.0]
            .map(|g| {
                s.spawn(move || {
                    let mut t = tela();
                    t.set_wet_relogio_fixo(true);
                    assert!(t.wet_preset_aplicar(&liner_com_glaze(g)).is_empty());
                    t.set_brush_size_px(RAIO_DA_CENA);
                    t.set_brush_color_srgb8(AZUL);
                    for y in [120.0, 300.0, 480.0] {
                        gesto(&mut t, &[[60.0, y], [560.0, y]], 3.0, |_| 1.0);
                    }
                    for _ in 0..1800 {
                        t.on_tick(1000.0 / 60.0);
                    }
                    t.run_full().0
                })
            })
            .map(|h| h.join().unwrap())
    });
    let n = a.chunks(4).zip(b.chunks(4)).filter(|(p, q)| p != q).count();
    assert_eq!(n, 0, "{n} px de um traço sozinho mudaram com o Glaze");
}

/// **Sonda da prancha (doc 42)**: os 5 pincéis de tinta e marcador SEM o Glaze (a mesma receita com a chave
/// `glaze` a 0), na cena da bancada seca ao raio do portão das mídias — `sem_glaze_<preset>.ppm` em
/// `PH2D_SONDA_OUT`, ao lado dos `midia_<preset>.ppm` de `cada_preset_parece_a_sua_midia`
/// (`ferramentas/prancha_lei_da_tinta.py` monta a prancha).
#[test]
#[ignore = "sonda: a prancha da lei da tinta (prompt 42); corre-se à mão"]
fn sonda_prancha_sem_glaze() {
    let dir = std::env::var("PH2D_SONDA_OUT").expect("PH2D_SONDA_OUT");
    std::fs::create_dir_all(&dir).unwrap();
    const RAIOS: [(&str, f32); 5] = [
        ("Ink Bullet", 21.0),
        ("Dip Pen", 19.0),
        ("Ink Brush", 20.0),
        ("Marker Bullet", 20.0),
        ("Marker Liner", 9.0),
    ];
    std::thread::scope(|s| {
        for (nome, raio) in RAIOS {
            let dir = &dir;
            s.spawn(move || {
                let mut p = de_fabrica(nome);
                assert!(
                    p.parametros
                        .engine
                        .insert("glaze".to_owned(), 0.0)
                        .is_some(),
                    "{nome}: o preset tem a chave glaze"
                );
                let mut t = tela();
                t.set_wet_relogio_fixo(true);
                assert!(t.wet_preset_aplicar(&p).is_empty());
                t.set_brush_size_px(raio);
                let (px, w, h) = pinta_cena_ate(&mut t, 1800);
                let mut f = format!("P6\n{w} {h}\n255\n").into_bytes();
                px.as_chunks::<4>()
                    .0
                    .iter()
                    .for_each(|q| f.extend_from_slice(&q[..3]));
                std::fs::write(format!("{dir}/sem_glaze_{nome}.ppm"), &f).unwrap();
                println!(
                    "[prancha] {nome:<14} sem Glaze: cruzamento {:.3}",
                    cruzamento(&px, w)
                );
            });
        }
    });
}

/// ⭐ **Trocar de pincel não muda a tinta que está na tela** (a lei VIAJA com a tinta, como o relevo do Bug #48):
/// o cruzamento de Glaze 1 e, antes de a água correr, o MESMO pincel com Glaze 0 — a tela que seca é a de quem
/// não trocou, ao byte. Uma lei da sim inteira (o knob lido no ecrã) mudaria o cruzamento molhado.
#[test]
fn trocar_de_pincel_nao_muda_a_tinta_da_tela() {
    let [a, b] = std::thread::scope(|s| {
        [false, true]
            .map(|troca| {
                s.spawn(move || {
                    let mut t = tela();
                    t.set_wet_relogio_fixo(true);
                    assert!(t.wet_preset_aplicar(&liner_com_glaze(1.0)).is_empty());
                    t.set_brush_size_px(RAIO_DA_CENA);
                    let (px, _, _) = pinta_cena_ate(&mut t, 0);
                    drop(px);
                    if troca {
                        assert!(t.wet_preset_aplicar(&liner_com_glaze(0.0)).is_empty());
                    }
                    for _ in 0..1800 {
                        t.on_tick(1000.0 / 60.0);
                    }
                    t.run_full().0
                })
            })
            .map(|h| h.join().unwrap())
    });
    let n = a.chunks(4).zip(b.chunks(4)).filter(|(p, q)| p != q).count();
    assert_eq!(
        n, 0,
        "{n} px mudaram ao trocar o Glaze do pincel depois de pintar"
    );
}

/// ⭐ **Um pincel gravado leva o seu Glaze**: o Wet Liner de Glaze 0,6 capturado sobre uma base SEM a chave
/// grava `glaze` 0,6 — e o capturado, aplicado, pinta a cena da bancada do original ao byte.
#[test]
fn um_pincel_gravado_leva_o_seu_glaze() {
    let mut t = tela();
    assert!(t.wet_preset_aplicar(&liner_com_glaze(0.6)).is_empty());
    let mut base = de_fabrica("Wet Liner").parametros;
    base.engine.remove("glaze");
    let (capturado, texturas) = t.wet_preset_capturar(&base);
    let g = capturado.engine.get("glaze").copied();
    assert!(
        g.is_some_and(|g| (g - 0.6).abs() < 1e-6),
        "o pincel gravado tem o Glaze {g:?}"
    );
    let mut de_volta = liner_com_glaze(0.6);
    de_volta.parametros = capturado;
    de_volta.texturas = texturas;
    let original = liner_com_glaze(0.6);
    let [a, b] = std::thread::scope(|s| {
        [&original, &de_volta]
            .map(|p| s.spawn(move || cena_com(p, 480)))
            .map(|h| h.join().unwrap())
    });
    let n =
        a.0.chunks(4)
            .zip(b.0.chunks(4))
            .filter(|(p, q)| p != q)
            .count();
    assert_eq!(
        n,
        0,
        "{n} px mudaram ao re-aplicar o pincel gravado (cruzamento {:.3} contra {:.3})",
        cruzamento(&a.0, a.1),
        cruzamento(&b.0, b.1)
    );
}

/// O vermelho do smoke do dono (16/10, as duas capturas: o traço sozinho (190, 55, 55), duas camadas (166, 34, 34)).
const VERMELHO_DO_DONO: [u8; 3] = [190, 55, 55];

/// Um traço recto da cena das camadas.
type Traco = [[f32; 2]; 2];

/// A luminância (luz linear) de um pixel RGBA — a do [`cruzamento`].
fn luz(px: &[u8], i: usize) -> f64 {
    let lin = |v: u8| {
        let c = f64::from(v) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(px[i]) + 0.7152 * lin(px[i + 1]) + 0.0722 * lin(px[i + 2])
}

fn dist_ao_traco(p: [f32; 2], [a, b]: Traco) -> f32 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let f = (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
    ((p[0] - a[0] - f * dx).powi(2) + (p[1] - a[1] - f * dy).powi(2)).sqrt()
}

/// A meia largura que se VÊ de um traço horizontal do pincel `p` ao raio `raio` (metade do contraste ao papel).
fn meia_largura(p: &ph2d_brush_presets::Preset, raio: f32) -> f32 {
    use super::sonda_gestos::{H, gesto};
    let mut t = tela();
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.set_brush_size_px(raio);
    t.set_brush_color_srgb8(VERMELHO_DO_DONO);
    gesto(&mut t, &[[200.0, 320.0], [800.0, 320.0]], 3.0, |_| 1.0);
    let (px, w, _) = t.run_full();
    let col: Vec<f64> = (0..H as usize)
        .map(|y| luz(&px, (y * w as usize + 500) * 4))
        .collect();
    let miolo = col[320];
    let n = col.iter().filter(|&&l| l < (1.0 + miolo) * 0.5).count();
    n as f32 * 0.5
}

/// As luminâncias por NÚMERO de camadas: cada pixel conta os traços em cujo miolo está (a menos de `0,75·hw`);
/// fica de fora quem está na orla de algum (entre `0,75·hw` e `1,25·hw`). Devolve, por camadas `k`, as luzes.
fn luzes_por_camadas(px: &[u8], w: u32, h: u32, tracos: &[Traco], hw: f32) -> Vec<Vec<f64>> {
    let mut por = vec![Vec::new(); tracos.len() + 1];
    for y in 0..h as usize {
        for x in 0..w as usize {
            let p = [x as f32, y as f32];
            let mut k = 0;
            let mut orla = false;
            for &t in tracos {
                let d = dist_ao_traco(p, t);
                if d < 0.75 * hw {
                    k += 1;
                } else if d < 1.25 * hw {
                    orla = true;
                }
            }
            if !orla && k > 0 {
                por[k].push(luz(px, (y * w as usize + x) * 4));
            }
        }
    }
    por
}

fn quantil(v: &[f64], q: f64) -> f64 {
    let mut v = v.to_vec();
    v.sort_by(f64::total_cmp);
    v.get(((v.len() as f64 - 1.0) * q).round() as usize)
        .copied()
        .unwrap_or(f64::NAN)
}

/// A cena das camadas da MESMA tinta: `tracos` um a um, com `esperas[k − 1]` quadros de 60 Hz antes do traço `k`
/// (a última repete-se) e `depois`
/// quadros no fim; `le(t, k)` corre depois do traço `k` (e `k = n` no fim).
fn pinta_camadas(
    p: &ph2d_brush_presets::Preset,
    raio: f32,
    tracos: &[Traco],
    esperas: &[u32],
    depois: u32,
    mut le: impl FnMut(&mut ph2d_tool_painter::PainterTool, usize),
) -> (Vec<u8>, u32, u32) {
    use super::sonda_gestos::gesto;
    let mut t = tela();
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.set_brush_size_px(raio);
    t.set_brush_color_srgb8(VERMELHO_DO_DONO);
    for (k, tr) in tracos.iter().enumerate() {
        if k > 0 {
            for _ in 0..esperas[(k - 1).min(esperas.len() - 1)] {
                t.on_tick(1000.0 / 60.0);
            }
        }
        gesto(&mut t, tr, 3.0, |_| 1.0);
        le(&mut t, k);
    }
    for _ in 0..depois {
        t.on_tick(1000.0 / 60.0);
    }
    le(&mut t, tracos.len());
    t.run_full()
}

/// As duas cenas: `camadas` (dois horizontais meio sobrepostos, um vertical, uma diagonal — 1, 2, 3 e 4
/// camadas em regiões limpas) e `estrela` (três traços por um ponto, a 1.ª captura do dono).
fn cenas_das_camadas(hw: f32) -> [(&'static str, Vec<Traco>); 2] {
    let mut c = cenas_das_camadas_(hw);
    // `PH2D_SONDA_INVERTE=1`: cada traço no sentido contrário.
    if std::env::var("PH2D_SONDA_INVERTE").is_ok_and(|v| v == "1") {
        for (_, t) in &mut c {
            t.iter_mut().for_each(|t| t.reverse());
        }
    }
    c
}

fn cenas_das_camadas_(hw: f32) -> [(&'static str, Vec<Traco>); 2] {
    [
        (
            "camadas",
            vec![
                [[150.0, 300.0], [850.0, 300.0]],
                [[150.0, 300.0 + 0.5 * hw], [850.0, 300.0 + 0.5 * hw]],
                [[500.0, 120.0], [500.0, 560.0]],
                [[330.0, 520.0], [700.0, 100.0]],
            ],
        ),
        (
            "estrela",
            vec![
                [[150.0, 320.0], [850.0, 320.0]],
                [[380.0, 80.0], [620.0, 560.0]],
                [[260.0, 500.0], [740.0, 140.0]],
            ],
        ),
    ]
}

/// Os pincéis de Glaze de fábrica e o raio do portão das mídias.
const PINCEIS_DE_GLAZE: [(&str, f32); 5] = [
    ("Ink Bullet", 21.0),
    ("Dip Pen", 19.0),
    ("Ink Brush", 20.0),
    ("Marker Bullet", 20.0),
    ("Marker Liner", 9.0),
];

/// **SONDA (smoke do dono 16/10): a 3.ª camada da MESMA tinta clareia.** Para cada pincel de Glaze × espera entre
/// traços (`PH2D_SONDA_ESPERAS`, quadros; 0 = molhado sobre molhado) × cena: por número de camadas, a luz mediana e
/// a mais clara (p95), e a fracção dos pixels de `k` camadas mais claros que a mediana de `k − 1`. Com
/// `PH2D_SONDA_CELULAS=1`, as células ao longo de `y` 300 + hw/4 (a faixa de 2 camadas da cena `camadas`) através
/// do vertical, depois de cada traço.
#[test]
#[ignore = "sonda: a terceira camada da mesma tinta (smoke 16/10); corre-se à mão"]
fn sonda_a_terceira_camada() {
    let so = std::env::var("PH2D_SONDA_SO").ok();
    let esperas: Vec<Vec<u32>> = std::env::var("PH2D_SONDA_ESPERAS").map_or_else(
        |_| vec![vec![0], vec![120], vec![1800]],
        |v| {
            v.split(',')
                .map(|x| x.split('/').map(|y| y.parse().unwrap()).collect())
                .collect()
        },
    );
    let raio_x: f32 = std::env::var("PH2D_SONDA_RAIO_X").map_or(1.0, |v| v.parse().unwrap());
    let celulas = std::env::var("PH2D_SONDA_CELULAS").is_ok();
    // `PH2D_SONDA_KS="nome:knob=v,knob=v|…"`: variantes do pincel (as chaves do mapa `engine` do preset).
    let variantes: Vec<(String, Vec<(String, f64)>)> = std::env::var("PH2D_SONDA_KS").map_or_else(
        |_| vec![(String::new(), Vec::new())],
        |v| {
            v.split('|')
                .map(|x| {
                    let (n, kv) = x.split_once(':').unwrap_or((x, ""));
                    let kv = kv
                        .split(',')
                        .filter(|y| !y.is_empty())
                        .map(|y| {
                            let (k, v) = y.split_once('=').unwrap();
                            (k.to_owned(), v.parse().unwrap())
                        })
                        .collect();
                    (n.to_owned(), kv)
                })
                .collect()
        },
    );
    std::thread::scope(|s| {
        // `PH2D_SONDA_SO`: os pincéis (qualquer um de fábrica; raio 20 fora dos de Glaze).
        let pinceis: Vec<(&str, f32)> = so.as_deref().map_or_else(
            || PINCEIS_DE_GLAZE.to_vec(),
            |q| {
                q.split(',')
                    .map(|n| {
                        let r = PINCEIS_DE_GLAZE
                            .iter()
                            .find(|(m, _)| *m == n)
                            .map_or(20.0, |x| x.1);
                        (n, r)
                    })
                    .collect()
            },
        );
        for (nome, raio) in pinceis {
            for (vnome, knobs) in &variantes {
                let esperas = &esperas;
                s.spawn(move || {
                let raio = raio * raio_x;
                let mut p = de_fabrica(nome);
                for (k, x) in knobs {
                    p.parametros.engine.insert(k.clone(), *x);
                }
                let hw = meia_largura(&p, raio);
                let mut linhas = vec![format!("[camadas] {nome}@{vnome} raio {raio} hw {hw:.1}")];
                for espera in esperas {
                    let ne = format!("{espera:?}");
                    for (cena, tracos) in cenas_das_camadas(hw) {
                        let mut cel = Vec::new();
                        let yc = 300.0 + hw * 0.25;
                        let (px, w, h) = pinta_camadas(&p, raio, &tracos, espera, 1800, |t, k| {
                            if !celulas || cena != "camadas" {
                                return;
                            }
                            t.wet_sonda_motor(|e| {
                                let g = e.active_grid();
                                let razao = 1000.0 / g.w as f64;
                                let ecra0 = e.sim.gather_params(&e.tuning).ecra;
                                cel.push(format!("    depois do traço {k} (y {yc:.0}):"));
                                for dx in (-36..=36).step_by(6) {
                                    let x = 500.0 + f64::from(dx) * f64::from(hw) / 24.0;
                                    let (cx, cy) =
                                        ((x / razao) as usize + 1, (f64::from(yc) / razao) as usize + 1);
                                    let i = cx + cy * g.s;
                                    let a = |m: f32| {
                                        ph2d_wet_paint::opacity::alpha_of_mass(f64::from(m))
                                    };
                                    let ec = ph2d_wet_paint::assenta::sobre_branco(
                                        ecra0.com_vela(g.vela[i]).seco(),
                                        g.sett_rgb[i].map(f64::from),
                                        g.susp_rgb[i].map(f64::from),
                                        a(g.sett[i]),
                                        a(g.susp[i]),
                                    );
                                    cel.push(format!(
                                        "      x {x:5.0} sett {:7.1} {:?} susp {:7.1} {:?} film {:.3} vela {:.3} → {:?}",
                                        g.sett[i],
                                        g.sett_rgb[i].map(|v| v.round() as i32),
                                        g.susp[i],
                                        g.susp_rgb[i].map(|v| v.round() as i32),
                                        g.film[i],
                                        g.vela[i],
                                        ec.map(|v| v.round() as i32)
                                    ));
                                }
                            });
                        });
                        let por = luzes_por_camadas(&px, w, h, &tracos, hw);
                        let mut l = format!("  espera {ne:>11} {cena:<8}");
                        for k in 1..por.len() {
                            let claros = if k > 1 {
                                let m = quantil(&por[k - 1], 0.5);
                                por[k].iter().filter(|&&v| v > m).count() as f64
                                    / por[k].len().max(1) as f64
                            } else {
                                0.0
                            };
                            l += &format!(
                                " | {k}: n {:5} med {:.4} p95 {:.4} >med{} {:.2}",
                                por[k].len(),
                                quantil(&por[k], 0.5),
                                quantil(&por[k], 0.95),
                                k - 1,
                                claros
                            );
                        }
                        linhas.push(l);
                        linhas.extend(cel);
                        if let Ok(dir) = std::env::var("PH2D_SONDA_OUT") {
                            let mut f = format!("P6\n{w} {h}\n255\n").into_bytes();
                            px.as_chunks::<4>().0.iter().for_each(|q| f.extend_from_slice(&q[..3]));
                            std::fs::write(format!("{dir}/camadas_{nome}{vnome}_{cena}_{}.ppm", ne.replace(", ", "-")), &f)
                                .unwrap();
                        }
                    }
                }
                println!("{}", linhas.join("\n"));
            });
            }
        }
    });
}

/// O estado de uma célula e o ecrã seco dela.
#[derive(Clone, Copy, Debug)]
struct Celula {
    sett: f32,
    sett_rgb: [f32; 3],
    susp: f32,
    susp_rgb: [f32; 3],
    film: f32,
    vela: f32,
    ecra: [f64; 3],
}

impl std::fmt::Display for Celula {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "sett {:7.1} {:?} susp {:7.1} {:?} film {:.3} vela {:.3} → {:?}",
            self.sett,
            self.sett_rgb.map(|v| v.round() as i32),
            self.susp,
            self.susp_rgb.map(|v| v.round() as i32),
            self.film,
            self.vela,
            self.ecra.map(|v| v.round() as i32)
        )
    }
}

/// Todas as células do motor e a razão pixel/célula.
fn celulas(t: &mut ph2d_tool_painter::PainterTool) -> (Vec<Celula>, f64, usize) {
    let mut out = (Vec::new(), 0.0, 0);
    t.wet_sonda_motor(|e| {
        let ecra0 = e.sim.gather_params(&e.tuning).ecra;
        let g = e.active_grid();
        let a = |m: f32| ph2d_wet_paint::opacity::alpha_of_mass(f64::from(m));
        out.1 = 1000.0 / g.w as f64;
        out.2 = g.s;
        out.0 = (0..g.sett.len())
            .map(|i| Celula {
                sett: g.sett[i],
                sett_rgb: g.sett_rgb[i],
                susp: g.susp[i],
                susp_rgb: g.susp_rgb[i],
                film: g.film[i],
                vela: g.vela[i],
                ecra: ph2d_wet_paint::assenta::sobre_branco(
                    ecra0.com_vela(g.vela[i]).seco(),
                    g.sett_rgb[i].map(f64::from),
                    g.susp_rgb[i].map(f64::from),
                    a(g.sett[i]),
                    a(g.susp[i]),
                ),
            })
            .collect();
    });
    out
}

/// **SONDA: a CÉLULA que clareia** — a cena `camadas` (os 3 primeiros), `PH2D_SONDA_ESPERA` quadros entre traços (omissão 120):
/// o estado de cada célula do miolo das 3 camadas ANTES do 3.º traço (A), logo DEPOIS (B), +30 quadros (C) e seca
/// (D). Lista as que acabam mais claras que em A (as piores) e conta as que escurecem.
#[test]
#[ignore = "sonda: a célula que clareia na 3.ª camada (smoke 16/10); corre-se à mão"]
fn sonda_a_celula_que_clareia() {
    use super::sonda_gestos::gesto;
    let nome = std::env::var("PH2D_SONDA_SO").unwrap_or_else(|_| "Dip Pen".to_owned());
    let espera: Vec<u32> = std::env::var("PH2D_SONDA_ESPERA").map_or_else(
        |_| vec![120, 120],
        |v| v.split('/').map(|x| x.parse().unwrap()).collect(),
    );
    let mut p = de_fabrica(&nome);
    // `PH2D_SONDA_K="knob=v,…"`: o pincel com outros knobs.
    for kv in std::env::var("PH2D_SONDA_K")
        .unwrap_or_default()
        .split(',')
        .filter(|x| !x.is_empty())
    {
        let (k, v) = kv.split_once('=').unwrap();
        p.parametros.engine.insert(k.to_owned(), v.parse().unwrap());
    }
    let raio = PINCEIS_DE_GLAZE.iter().find(|(n, _)| *n == nome).unwrap().1;
    let hw = meia_largura(&p, raio);
    let [(_, mut tracos), _] = cenas_das_camadas(hw);
    tracos.truncate(3);
    let mut t = tela();
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&p).is_empty());
    t.set_brush_size_px(raio);
    t.set_brush_color_srgb8(VERMELHO_DO_DONO);
    let espera_ = |t: &mut ph2d_tool_painter::PainterTool, n: u32| {
        for _ in 0..n {
            t.on_tick(1000.0 / 60.0);
        }
    };
    gesto(&mut t, &tracos[0], 3.0, |_| 1.0);
    espera_(&mut t, espera[0]);
    gesto(&mut t, &tracos[1], 3.0, |_| 1.0);
    espera_(&mut t, espera[espera.len() - 1]);
    let (a, razao, s) = celulas(&mut t);
    gesto(&mut t, &tracos[2], 3.0, |_| 1.0);
    let (b, _, _) = celulas(&mut t);
    espera_(&mut t, 30);
    let (c, _, _) = celulas(&mut t);
    espera_(&mut t, 1800);
    let (d, _, _) = celulas(&mut t);
    let lum = |e: [f64; 3]| 0.2126 * e[0] + 0.7152 * e[1] + 0.0722 * e[2];
    let mut claras = Vec::new();
    let (mut n, mut escurece) = (0, 0);
    for i in 0..a.len() {
        let (cx, cy) = (i % s, i / s);
        let px = [
            ((cx as f64 - 0.5) * razao) as f32,
            ((cy as f64 - 0.5) * razao) as f32,
        ];
        if !tracos.iter().all(|&tr| dist_ao_traco(px, tr) < 0.6 * hw) {
            continue;
        }
        n += 1;
        let dl = lum(d[i].ecra) - lum(a[i].ecra);
        if dl > 0.5 {
            claras.push((dl, i, px));
        } else if dl < -0.5 {
            escurece += 1;
        }
    }
    claras.sort_by(|x, y| y.0.total_cmp(&x.0));
    // As células cuja cor ASSENTADA é mais clara que a tinta pura no fim (a lei nunca as dá; a cobertura das orlas
    // não conta): onde estão e a história delas.
    let pura = lum(VERMELHO_DO_DONO.map(f64::from));
    let mut palidas: Vec<(f64, usize, [f32; 2], usize)> = (0..a.len())
        .filter_map(|i| {
            let (cx, cy) = (i % s, i / s);
            let px = [
                ((cx as f64 - 1.0) * razao) as f32,
                ((cy as f64 - 1.0) * razao) as f32,
            ];
            let k = tracos
                .iter()
                .filter(|&&tr| dist_ao_traco(px, tr) < 0.75 * hw)
                .count();
            let dl = lum(d[i].sett_rgb.map(f64::from)) - pura;
            (k > 0 && d[i].sett > 0.0 && dl > 1.0).then_some((dl, i, px, k))
        })
        .collect();
    palidas.sort_by(|x, y| y.0.total_cmp(&x.0));
    println!(
        "  {} células com o assentado mais claro que a tinta (> 1 nível)",
        palidas.len()
    );
    for &(dl, i, px, k) in palidas.iter().take(4) {
        println!(
            "  ({:.0}, {:.0}) {k} camadas, +{dl:.1} sobre a tinta",
            px[0], px[1]
        );
        for (q, c) in [("A", &a), ("B", &b), ("C", &c), ("D", &d)] {
            println!("     {q} {}", c[i]);
        }
    }
    // A média do ecrã e do estado nas células de 2 camadas (1.º e 2.º, longe do 3.º) e de 3, em A · B · C · D.
    for (rotulo, quer) in [("2 camadas", 2usize), ("3 camadas", 3)] {
        let sel: Vec<usize> = (0..a.len())
            .filter(|&i| {
                let (cx, cy) = (i % s, i / s);
                let px = [
                    ((cx as f64 - 1.0) * razao) as f32,
                    ((cy as f64 - 1.0) * razao) as f32,
                ];
                let dentro = |k: usize| dist_ao_traco(px, tracos[k]) < 0.6 * hw;
                let fora = |k: usize| dist_ao_traco(px, tracos[k]) > 1.4 * hw;
                dentro(0) && dentro(1) && if quer == 2 { fora(2) } else { dentro(2) }
            })
            .collect();
        let n = sel.len().max(1) as f64;
        for (q, c) in [("A", &a), ("B", &b), ("C", &c), ("D", &d)] {
            let m = |f: &dyn Fn(&Celula) -> f64| sel.iter().map(|&i| f(&c[i])).sum::<f64>() / n;
            println!(
                "  {rotulo} ({}) {q}: ecrã [{:.1}, {:.1}, {:.1}] sett {:.0} [{:.0}, {:.0}] susp {:.0} film {:.3} vela {:.3}",
                sel.len(),
                m(&|c| c.ecra[0]),
                m(&|c| c.ecra[1]),
                m(&|c| c.ecra[2]),
                m(&|c| f64::from(c.sett)),
                m(&|c| f64::from(c.sett_rgb[0])),
                m(&|c| f64::from(c.sett_rgb[1])),
                m(&|c| f64::from(c.susp)),
                m(&|c| f64::from(c.film)),
                m(&|c| f64::from(c.vela)),
            );
        }
    }
    println!(
        "[célula] {nome} espera {espera:?} hw {hw:.1}: {n} células de 3 camadas · {} clareiam > 0,5 · {escurece} escurecem",
        claras.len()
    );
    for &(dl, i, px) in claras.iter().take(10) {
        println!("  ({:.0}, {:.0}) Δlum +{dl:.1}", px[0], px[1]);
        for (q, c) in [("A", &a), ("B", &b), ("C", &c), ("D", &d)] {
            println!("     {q} {}", c[i]);
        }
    }
}

/// ⭐ **A 3.ª camada da mesma tinta não clareia** (smoke do dono 16/10, doc 42 §7). A cena `camadas` com o 1.º traço
/// seco e o 2.º e o 3.º MOLHADOS sobre o anterior (o caso do dono): cada uma das três primeiras camadas escurece a
/// mediana (luz linear) e a 4.ª não clareia — a lei de camadas que o Rebelle mede igual molhada e seca (§3, §3b) —, e
/// nenhum miolo de uma camada só fica mais claro que a tinta (o reflexo claro das capturas).
#[test]
fn a_terceira_camada_da_mesma_tinta_nao_clareia() {
    let medidos = std::thread::scope(|s| {
        [("Marker Bullet", 30.0), ("Dip Pen", 28.5)]
            .map(|(nome, raio)| {
                s.spawn(move || {
                    let p = de_fabrica(nome);
                    let hw = meia_largura(&p, raio);
                    let [(_, tracos), _] = cenas_das_camadas_(hw);
                    let (px, w, h) =
                        pinta_camadas(&p, raio, &tracos, &[1800, 0, 0], 1800, |_, _| {});
                    (nome, luzes_por_camadas(&px, w, h, &tracos, hw))
                })
            })
            .map(|h| h.join().unwrap())
    });
    let mut q = VERMELHO_DO_DONO.to_vec();
    q.push(255);
    let pura = luz(&q, 0);
    for (nome, por) in &medidos {
        let med: Vec<f64> = por[1..].iter().map(|v| quantil(v, 0.5)).collect();
        let claro = quantil(&por[1], 0.95);
        assert!(
            med[1] < med[0] - 0.002 && med[2] < med[1] - 0.002 && med[3] <= med[2] + 1e-9,
            "{nome}: cada camada da mesma tinta escurece — medianas 1..4 {med:.4?}"
        );
        assert!(
            claro < pura * 1.04,
            "{nome}: uma camada só mais clara que a tinta — p95 {claro:.4} contra {pura:.4}"
        );
    }
}

/// SONDA (handoff P51 §4.2): as medianas de [`a_terceira_camada_da_mesma_tinta_nao_clareia`] por variante
/// (`PH2D_SONDA_VARIANTES`, a régua de `sonda_spacing_continuo`), todas no MESMO processo.
#[test]
#[ignore = "sonda: a 3.ª camada por variante (prompt 51); corre-se à mão"]
fn sonda_a_terceira_camada_por_variante() {
    use super::sonda_spacing_continuo::{com, com_sub, variantes};
    let vs = variantes();
    let linhas: Vec<String> = std::thread::scope(|s| {
        let hs: Vec<_> = [("Marker Bullet", 30.0), ("Dip Pen", 28.5)]
            .into_iter()
            .flat_map(|(nome, raio)| {
                vs.iter().map(move |(vn, kv)| {
                    s.spawn(move || {
                        com_sub(kv, || {
                            let p = com(de_fabrica(nome), kv);
                            let hw = meia_largura(&p, raio);
                            let [(_, tracos), _] = cenas_das_camadas_(hw);
                            let (px, w, h) =
                                pinta_camadas(&p, raio, &tracos, &[1800, 0, 0], 1800, |_, _| {});
                            let por = luzes_por_camadas(&px, w, h, &tracos, hw);
                            let med: Vec<f64> = por[1..].iter().map(|v| quantil(v, 0.5)).collect();
                            format!(
                                "[camadas] {nome}@{vn}: medianas {med:.4?} · p95 {:.4}",
                                quantil(&por[1], 0.95)
                            )
                        })
                    })
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for l in linhas {
        println!("{l}");
    }
}

/// **O primeiro toque só assenta a tinta de OUTRO traço** (doc 42 §7): com a Live Water a água corre com o traço aberto
/// e a tinta do próprio traço chega a células que ele ainda não tocou — tomadas pelo toque, a orla do traço velava-se a
/// si mesma. Os três azuis de [`o_traco_sozinho_nao_muda_com_o_glaze`] com a Live Water, Dip Pen de Glaze 1 contra 0:
/// o pior canal muda ≤ 25 níveis (medido 13; com as células TOCADAS em vez das molhadas antes do traço, 119; o motor de
/// `2e43def369`, 39 — a Live Water já movia o assentado do traço sob o Glaze).
#[test]
fn o_primeiro_toque_nao_assenta_o_proprio_traco_com_agua_viva() {
    use super::sonda_gestos::{AZUL, gesto};
    let [a, b] = std::thread::scope(|s| {
        [0.0, 1.0]
            .map(|g| {
                s.spawn(move || {
                    let mut p = de_fabrica("Dip Pen");
                    p.parametros.engine.insert("glaze".to_owned(), g);
                    let mut t = tela();
                    t.set_wet_relogio_fixo(true);
                    assert!(t.wet_preset_aplicar(&p).is_empty());
                    t.set_wet_agua_viva(true);
                    t.set_brush_size_px(RAIO_DA_CENA);
                    t.set_brush_color_srgb8(AZUL);
                    for y in [120.0, 300.0, 480.0] {
                        gesto(&mut t, &[[60.0, y], [560.0, y]], 3.0, |_| 1.0);
                    }
                    for _ in 0..1800 {
                        t.on_tick(1000.0 / 60.0);
                    }
                    t.run_full().0
                })
            })
            .map(|h| h.join().unwrap())
    });
    let pior = a
        .chunks(4)
        .zip(b.chunks(4))
        .flat_map(|(p, q)| (0..3).map(move |c| (i32::from(p[c]) - i32::from(q[c])).abs()))
        .max()
        .unwrap_or(0);
    assert!(
        pior <= 25,
        "a Live Water: o traço sozinho muda {pior} níveis com o Glaze"
    );
}

/// A cena dos MUITOS cruzamentos (smoke do dono 17/10): seis traços do mesmo vermelho em direcções várias.
const MUITOS: [Traco; 6] = [
    [[100.0, 200.0], [900.0, 260.0]],
    [[150.0, 520.0], [850.0, 120.0]],
    [[300.0, 80.0], [700.0, 580.0]],
    [[100.0, 380.0], [900.0, 330.0]],
    [[600.0, 60.0], [350.0, 600.0]],
    [[120.0, 120.0], [880.0, 500.0]],
];

/// A luz linear de uma cor sRGB 0..255.
fn luz_rgb(c: [f64; 3]) -> f64 {
    let lin = |v: f64| {
        let c = (v / 255.0).clamp(0.0, 1.0);
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2])
}

/// As LINHAS CLARAS dentro da tinta escura, no ecrã das CÉLULAS (`l`, `w` × `h`, luz linear): uma célula mais clara
/// por mais de `limiar` (razão) que as DUAS do outro lado, a 2 ou 3 células numa das quatro direcções, ambas já de
/// várias camadas (abaixo de 85 % da tinta pura) — uma crista fina, não a orla entre uma zona clara e uma escura.
/// Devolve (quantas, a pior razão).
fn linhas_claras(l: &[f64], w: usize, h: usize, limiar: f64) -> (usize, f64) {
    let pura = luz_rgb(VERMELHO_DO_DONO.map(f64::from));
    let (mut n, mut pior) = (0, 1.0f64);
    for y in 3..h - 3 {
        for x in 3..w - 3 {
            let c = l[y * w + x];
            let mut r = 0.0f64;
            for (dx, dy) in [(1i64, 0i64), (0, 1), (1, 1), (1, -1)] {
                for k in [2i64, 3] {
                    let at = |sx: i64| {
                        l[((y as i64 + sx * k * dy) as usize) * w
                            + (x as i64 + sx * k * dx) as usize]
                    };
                    let lado = at(1).max(at(-1));
                    if lado < 0.85 * pura && lado > 0.0 {
                        r = r.max(c / lado);
                    }
                }
            }
            if r > 1.0 + limiar {
                n += 1;
            }
            pior = pior.max(r);
        }
    }
    (n, pior)
}

/// O ecrã das células (luz linear) e a largura da grade (sem o anel).
fn luz_das_celulas(t: &mut ph2d_tool_painter::PainterTool) -> (Vec<f64>, usize, usize) {
    let (c, _, s) = celulas(t);
    let h = c.len() / s;
    let l = c.iter().map(|c| luz_rgb(c.ecra)).collect();
    (l, s, h)
}

/// **SONDA (smoke do dono 17/10): as bordas que esbranquiçam depois de vários cruzamentos.** Os pincéis de
/// `PH2D_SONDA_SO` (omissão: os 5 de Glaze) × `PH2D_SONDA_ESPERAS` (quadros entre traços) × o Glaze do preset e 0 (o
/// controlo): as linhas claras ([`linhas_claras`], > 4 %) 1 quadro depois do último traço e a 240 e 1 800.
#[test]
#[ignore = "sonda: as bordas que esbranquiçam (smoke 17/10); corre-se à mão"]
fn sonda_as_bordas() {
    use super::sonda_gestos::gesto;
    let so = std::env::var("PH2D_SONDA_SO").ok();
    let esperas: Vec<u32> = std::env::var("PH2D_SONDA_ESPERAS").map_or_else(
        |_| vec![0, 60, 300],
        |v| v.split(',').map(|x| x.parse().unwrap()).collect(),
    );
    let raio_x: f32 = std::env::var("PH2D_SONDA_RAIO_X").map_or(1.5, |v| v.parse().unwrap());
    let pinceis: Vec<(&str, f32)> = so.as_deref().map_or_else(
        || PINCEIS_DE_GLAZE.to_vec(),
        |q| {
            q.split(',')
                .map(|n| {
                    let r = PINCEIS_DE_GLAZE
                        .iter()
                        .find(|(m, _)| *m == n)
                        .map_or(20.0, |x| x.1);
                    (n, r)
                })
                .collect()
        },
    );
    std::thread::scope(|s| {
        for (nome, raio) in pinceis {
            for &espera in &esperas {
                for controlo in [false, true] {
                    s.spawn(move || {
                        let mut p = de_fabrica(nome);
                        if controlo {
                            p.parametros.engine.insert("glaze".to_owned(), 0.0);
                        }
                        let mut t = tela();
                        t.set_wet_relogio_fixo(true);
                        assert!(t.wet_preset_aplicar(&p).is_empty());
                        t.set_brush_size_px(raio * raio_x);
                        t.set_brush_color_srgb8(VERMELHO_DO_DONO);
                        for (k, tr) in MUITOS.iter().enumerate() {
                            if k > 0 {
                                for _ in 0..espera {
                                    t.on_tick(1000.0 / 60.0);
                                }
                            }
                            gesto(&mut t, tr, 3.0, |_| 1.0);
                        }
                        let mut l = format!(
                            "[bordas] {nome:<13} {} espera {espera:4}:",
                            if controlo { "Glaze 0" } else { "Glaze  " }
                        );
                        let mut feito = 0;
                        let mut antes: Vec<f64> = Vec::new();
                        // `PH2D_SONDA_FIM=q`: a foto (e o fim) em `q` quadros em vez de 1 800.
                        let fim: u32 = std::env::var("PH2D_SONDA_FIM").map_or(1800, |v| v.parse().unwrap());
                        for q in [1u32, 240, 1800].into_iter().filter(|&q| q <= fim).chain((fim < 1800).then_some(fim)) {
                            for _ in feito..q {
                                t.on_tick(1000.0 / 60.0);
                            }
                            feito = q;
                            let (lc, wc, hc) = luz_das_celulas(&mut t);
                            let (n, pior) = linhas_claras(&lc, wc, hc, 0.04);
                            // As que CLAREIAM com a água: > 4 % mais claras que logo depois do traço, na tinta.
                            let pura = luz_rgb(VERMELHO_DO_DONO.map(f64::from));
                            let conta = |teto: f64| {
                                lc.iter()
                                    .zip(&antes)
                                    .filter(|&(&d, &a)| a < teto && a > 0.0 && d > a * 1.04)
                                    .count()
                            };
                            let (escuras, toda) = if antes.is_empty() {
                                antes = lc.clone();
                                (0, 0)
                            } else {
                                (conta(0.85 * pura), conta(0.6))
                            };
                            l += &format!(
                                " | +{q}: {n:5} cristas, pior {pior:.3}; clareiam {escuras:5} escuras · {toda:5} na tinta"
                            );
                            if q == fim
                                && let Ok(dir) = std::env::var("PH2D_SONDA_OUT")
                            {
                                let (px, w, h) = t.run_full();
                                let mut f = format!("P6\n{w} {h}\n255\n").into_bytes();
                                px.as_chunks::<4>().0.iter().for_each(|c| f.extend_from_slice(&c[..3]));
                                std::fs::write(
                                    format!("{dir}/bordas_{nome}_{espera}_{}_{fim}.ppm", u8::from(controlo)),
                                    &f,
                                )
                                .unwrap();
                            }
                        }
                        println!("{l}");
                    });
                }
            }
        }
    });
}

/// **SONDA: a CÉLULA da linha clara** — a cena [`MUITOS`] com `PH2D_SONDA_SO` (omissão Marker Bullet) e
/// `PH2D_SONDA_ESPERA` (omissão 0): as 3 células mais claras que a vizinhança aos 240 quadros, com o estado delas e dos
/// vizinhos a 1, 60 e 240 quadros depois do último traço — e o estado delas DEPOIS DE CADA TRAÇO.
#[test]
#[ignore = "sonda: a célula da linha clara (smoke 17/10); corre-se à mão"]
fn sonda_a_celula_da_borda() {
    use super::sonda_gestos::gesto;
    let nome = std::env::var("PH2D_SONDA_SO").unwrap_or_else(|_| "Marker Bullet".to_owned());
    let espera: u32 = std::env::var("PH2D_SONDA_ESPERA").map_or(0, |v| v.parse().unwrap());
    let raio = PINCEIS_DE_GLAZE
        .iter()
        .find(|(n, _)| *n == nome)
        .map_or(20.0, |x| x.1)
        * 1.5;
    let p = de_fabrica(&nome);
    let mut t = tela();
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&p).is_empty());
    t.set_brush_size_px(raio);
    t.set_brush_color_srgb8(VERMELHO_DO_DONO);
    t.wet_sonda_motor(|e| {
        println!(
            "[borda] a lei do ecrã: {:?}",
            e.sim.gather_params(&e.tuning).ecra
        )
    });
    let mut por_traco = Vec::new();
    for (k, tr) in MUITOS.iter().enumerate() {
        if k > 0 {
            for _ in 0..espera {
                t.on_tick(1000.0 / 60.0);
            }
        }
        gesto(&mut t, tr, 3.0, |_| 1.0);
        por_traco.push(celulas(&mut t).0);
    }
    // `PH2D_SONDA_QUADRO="x,y"`: essa célula (pixels da tela) a cada quadro, até 60, e as 4 vizinhas.
    if let Ok(v) = std::env::var("PH2D_SONDA_QUADRO") {
        let (x, y) = v.split_once(',').unwrap();
        let (x, y): (f64, f64) = (x.parse().unwrap(), y.parse().unwrap());
        for q in 0..60 {
            let (c, razao, s) = celulas(&mut t);
            let i = (y / razao) as usize * s + s + (x / razao) as usize + 1;
            println!("[quadro {q:2}] {}", c[i]);
            for (nm, d) in [
                ("x−1", i - 1),
                ("x+1", i + 1),
                ("y−1", i - s),
                ("y+1", i + s),
            ] {
                println!("      {nm} {}", c[d]);
            }
            t.on_tick(1000.0 / 60.0);
        }
        return;
    }
    let mut fotos = Vec::new();
    let mut feito = 0;
    for q in [1u32, 60, 240] {
        for _ in feito..q {
            t.on_tick(1000.0 / 60.0);
        }
        feito = q;
        fotos.push(celulas(&mut t));
    }
    let (c, razao, s) = &fotos[2];
    let h = c.len() / s;
    let l: Vec<f64> = c.iter().map(|c| luz_rgb(c.ecra)).collect();
    let mut piores = Vec::new();
    for y in 3..h - 3 {
        for x in 3..s - 3 {
            let (n, r) = {
                let mut janela = vec![0.0; 49];
                for dy in 0..7 {
                    for dx in 0..7 {
                        janela[dy * 7 + dx] = l[(y + dy - 3) * s + x + dx - 3];
                    }
                }
                linhas_claras(&janela, 7, 7, 0.04)
            };
            if n > 0 {
                piores.push((r, x, y));
            }
        }
    }
    piores.sort_by(|a, b| b.0.total_cmp(&a.0));
    // `PH2D_SONDA_PX="x,y;x,y"`: estas posições (pixels da tela) em vez das piores.
    if let Ok(v) = std::env::var("PH2D_SONDA_PX") {
        piores = v
            .split(';')
            .map(|q| {
                let (x, y) = q.split_once(',').unwrap();
                let (x, y): (f64, f64) = (x.parse().unwrap(), y.parse().unwrap());
                (0.0, (x / razao) as usize + 1, (y / razao) as usize + 1)
            })
            .collect();
    }
    println!(
        "[borda] {nome} espera {espera}: {} células claras",
        piores.len()
    );
    let mut vistos: Vec<(usize, usize)> = Vec::new();
    for &(r, x, y) in &piores {
        if std::env::var("PH2D_SONDA_PX").is_err()
            && vistos
                .iter()
                .any(|&(a, b)| a.abs_diff(x) < 10 && b.abs_diff(y) < 10)
        {
            continue;
        }
        vistos.push((x, y));
        if vistos.len() > 3 && std::env::var("PH2D_SONDA_PX").is_err() {
            break;
        }
        println!(
            "  célula ({x}, {y}) = px ({:.0}, {:.0}), razão {r:.3}",
            (x as f64 - 1.0) * razao,
            (y as f64 - 1.0) * razao
        );
        for (k, pt) in por_traco.iter().enumerate() {
            println!("     traço {k}: {}", pt[y * s + x]);
        }
        for (q, (f, _, _)) in [1, 60, 240].iter().zip(&fotos) {
            println!("     +{q:3}: {}", f[y * s + x]);
        }
        for (nm, d) in [
            ("x−2", y * s + x - 2),
            ("x+2", y * s + x + 2),
            ("y−2", (y - 2) * s + x),
            ("y+2", (y + 2) * s + x),
        ] {
            println!("     {nm} +240: {}", c[d]);
        }
    }
}

/// **SONDA: a ORLA de um traço sobre duas camadas** — dois horizontais iguais (y 300) e um vertical (x 500) cujas
/// orlas cruzam a faixa de 2 camadas; `PH2D_SONDA_ESPERAS="a/b"` quadros antes do 2.º e do 3.º. O ecrã das células ao
/// longo de y 300 (x 440..560) logo depois do vertical, a +60 e +1 800, e o estado da célula mais clara da orla.
#[test]
#[ignore = "sonda: a orla de um traço sobre duas camadas (smoke 17/10); corre-se à mão"]
fn sonda_a_orla() {
    let so = std::env::var("PH2D_SONDA_SO").unwrap_or_else(|_| "Marker Bullet".to_owned());
    let esperas: Vec<Vec<u32>> = std::env::var("PH2D_SONDA_ESPERAS").map_or_else(
        |_| vec![vec![1800, 0], vec![1800, 60], vec![1800, 1800], vec![0, 0]],
        |v| {
            v.split(',')
                .map(|x| x.split('/').map(|y| y.parse().unwrap()).collect())
                .collect()
        },
    );
    for nome in so.split(',') {
        let raio = PINCEIS_DE_GLAZE
            .iter()
            .find(|(n, _)| *n == nome)
            .map_or(20.0, |x| x.1)
            * 1.5;
        for g in [None, Some(0.0)] {
            for espera in &esperas {
                let mut p = de_fabrica(nome);
                if let Some(g) = g {
                    p.parametros.engine.insert("glaze".to_owned(), g);
                }
                let tracos: [Traco; 3] = [
                    [[150.0, 300.0], [850.0, 300.0]],
                    [[850.0, 300.0], [150.0, 300.0]],
                    // `PH2D_SONDA_DIAG=1`: o 3.º em diagonal (cruza y 300 em x ≈ 500).
                    if std::env::var("PH2D_SONDA_DIAG").is_ok() {
                        [[370.0, 100.0], [630.0, 500.0]]
                    } else {
                        [[500.0, 100.0], [500.0, 560.0]]
                    },
                ];
                let mut t = tela();
                t.set_wet_relogio_fixo(true);
                assert!(t.wet_preset_aplicar(&p).is_empty());
                t.set_brush_size_px(raio);
                t.set_brush_color_srgb8(VERMELHO_DO_DONO);
                let mut linha = format!(
                    "[orla] {nome} {} espera {espera:?}:",
                    if g.is_some() { "Glaze 0" } else { "Glaze  " }
                );
                let mut fotos = Vec::new();
                for (k, tr) in tracos.iter().enumerate() {
                    if k > 0 {
                        for _ in 0..espera[(k - 1).min(espera.len() - 1)] {
                            t.on_tick(1000.0 / 60.0);
                        }
                    }
                    super::sonda_gestos::gesto(&mut t, tr, 3.0, |_| 1.0);
                    if k == 1 {
                        fotos.push(celulas(&mut t));
                    }
                }
                let mut feito = 0;
                for q in [0u32, 60, 1800] {
                    for _ in feito..q {
                        t.on_tick(1000.0 / 60.0);
                    }
                    feito = q;
                    fotos.push(celulas(&mut t));
                }
                for (rot, (c, razao, s)) in ["antes", "+0", "+60", "+1800"].iter().zip(&fotos) {
                    let lum: Vec<f64> = (440..=560)
                        .step_by(4)
                        .map(|x| {
                            let i = (300.0 / razao) as usize * s
                                + s
                                + (f64::from(x) / razao) as usize
                                + 1;
                            luz_rgb(c[i].ecra)
                        })
                        .collect();
                    let fora = (lum[0] + lum[1] + lum[lum.len() - 1] + lum[lum.len() - 2]) / 4.0;
                    let pico = lum.iter().copied().fold(0.0, f64::max);
                    linha += &format!(" | {rot}: fora {fora:.4} pico {pico:.4}");
                    if std::env::var("PH2D_SONDA_PERFIL").is_ok() {
                        linha += &format!(
                            "\n      {:?}",
                            lum.iter()
                                .map(|v| (v * 1e4).round() / 1e4)
                                .collect::<Vec<_>>()
                        );
                    }
                }
                println!("{linha}");
            }
        }
    }
}

/// As células da tinta de várias camadas (abaixo de 85 % da tinta pura) que CLAREIAM mais de 4 % enquanto a água corre
/// `quadros` depois do último traço da cena [`MUITOS`].
fn clareiam_nas_escuras(nome: &str, espera: u32, quadros: u32) -> usize {
    use super::sonda_gestos::gesto;
    let raio = PINCEIS_DE_GLAZE
        .iter()
        .find(|(n, _)| *n == nome)
        .map_or(20.0, |x| x.1)
        * 1.5;
    let mut t = tela();
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&de_fabrica(nome)).is_empty());
    t.set_brush_size_px(raio);
    t.set_brush_color_srgb8(VERMELHO_DO_DONO);
    for (k, tr) in MUITOS.iter().enumerate() {
        if k > 0 {
            for _ in 0..espera {
                t.on_tick(1000.0 / 60.0);
            }
        }
        gesto(&mut t, tr, 3.0, |_| 1.0);
    }
    t.on_tick(1000.0 / 60.0);
    let (antes, _, _) = luz_das_celulas(&mut t);
    for _ in 1..quadros {
        t.on_tick(1000.0 / 60.0);
    }
    let (depois, _, _) = luz_das_celulas(&mut t);
    let teto = 0.85 * luz_rgb(VERMELHO_DO_DONO.map(f64::from));
    depois
        .iter()
        .zip(&antes)
        .filter(|&(&d, &a)| a < teto && a > 0.0 && d > a * 1.04)
        .count()
}

/// ⭐ **As bordas não esbranquiçam depois de vários cruzamentos** (smoke do dono 17/10, doc 42 §8): os seis traços de
/// [`MUITOS`] e a água a correr — a tinta de várias camadas não clareia (as linhas finas que seguiam a orla de cada
/// traço). Medido: Marker Bullet molhado sobre molhado 975 células → 60; Ink Bullet com 5 s entre traços 923 → 299.
#[test]
fn as_bordas_nao_esbranquicam_depois_de_varios_cruzamentos() {
    let [marcador, tinta] = std::thread::scope(|s| {
        [("Marker Bullet", 0, 240), ("Ink Bullet", 300, 1800)]
            .map(|(n, e, q)| s.spawn(move || clareiam_nas_escuras(n, e, q)))
            .map(|h| h.join().unwrap())
    });
    assert!(
        marcador <= 150 && tinta <= 500,
        "a tinta de várias camadas clareia com a água: Marker Bullet {marcador}, Ink Bullet {tinta} células"
    );
}
