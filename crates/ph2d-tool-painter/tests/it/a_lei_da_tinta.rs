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

/// `(preset, caixas da obra ligadas, fnv1a da cena)` — pinados em `f46fcffb2a`.
#[rustfmt::skip]
const CENAS: [(&str, bool, u64); 14] = [
    ("Wet Round", false, 0xfce64c9627dc5bc3),
    ("Wet Wash", false, 0x782a0a8a8ed9c439),
    ("Wet Dry Brush", false, 0x1a361614679dde9a),
    ("Wet Drip", false, 0x8159963aa484f688),
    ("Wet Liner", false, 0x27defb305741fea9),
    ("Oil Bristle", false, 0x53cf6eb66d3857e1),
    ("Acrylic Round", false, 0xfea6bfd5e31bf152),
    ("Express Oil", false, 0x6e348caa0df4a42e),
    ("Graphite 8B", false, 0x55368fbf0685ad46),
    ("Charcoal", false, 0x3b6500d86d772fd6),
    ("Soft Pastel", false, 0xf99dd413f5f0218e),
    ("Oil Pastel", false, 0x235fe3c132334588),
    ("Wet Round", true, 0x44016d14e50f2119),
    ("Wet Liner", true, 0x7a276b4a53ec4531),
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
        let [r, g, b] = s.map(|v| lin(((v / n) as f64).round().clamp(0.0, 255.0) as u8));
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
