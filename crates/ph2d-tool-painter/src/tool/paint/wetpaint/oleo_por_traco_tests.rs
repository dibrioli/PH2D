//! **Depth, Gloss e a caixa Real Oil são do TRAÇO** (dono 28/10): valem para os traços seguintes, a tinta pousada não
//! muda; os números que ficam no cartão (luz, sombra, fenda, ambiente) são da TELA e aplicam-se à tinta com óleo.
//! Doc: `docs/Painter/49_realshader/W3b_painel_so_no_futuro.md` §Depth, Gloss e Real Oil por traço.

use super::*;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase};
use ph2d_wet_paint::tuning::Knob;

const W: usize = 200;
const H: usize = 190;
const ANTIGOS: std::ops::Range<usize> = 0..95;
const SEGUINTE: std::ops::Range<usize> = 140..H;

pub(super) fn pinta(t: &mut PainterTool, y: f32) {
    let p = |x: f32, phase| CanvasPointer {
        pos: [x, y],
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(30.0, PointerPhase::Down));
    for k in 1..=40 {
        t.on_canvas_pointer(p(30.0 + 3.5 * k as f32, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(p(170.0, PointerPhase::Up));
    for _ in 0..20 {
        t.on_tick(1000.0 / 60.0);
    }
}

/// A tela de prova: papel Cold, relevo, dois traços (um seco por baixo de um molhado) com a caixa `oleo`.
pub(super) fn tela(oleo: bool) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(14.0);
    t.set_brush_paper_kind(ph2d_painter_brush::TextureKind::PaperCold.to_u8());
    t.paint.wetpaint.knobs.set(Knob::Emboss, 1.5);
    t.paint.wetpaint.knobs.set(Knob::PigmentPerDab, 1200.0);
    t.set_wet_oleo_real_ligado(oleo);
    t.set_brush_color_srgb8([180, 30, 30]);
    pinta(&mut t, 60.0);
    t.wetpaint_dry_canvas();
    t.set_brush_color_srgb8([30, 60, 200]);
    pinta(&mut t, 70.0);
    t
}

fn recompoe(t: &mut PainterTool) -> Vec<u8> {
    t.wet_recomposite_full();
    t.canvas_rgba.to_vec()
}

fn linhas(c: &[u8], r: std::ops::Range<usize>) -> &[u8] {
    &c[r.start * W * 4..r.end * W * 4]
}

fn grades(t: &mut PainterTool) -> &mut Vec<ph2d_wet_paint::painter::Layer> {
    t.wet_bring_home();
    &mut t
        .paint
        .wetpaint
        .session
        .as_mut()
        .expect("a sessão")
        .engine
        .layers
}

/// ⭐ …e o traço SEGUINTE usa o valor novo: Depth, Gloss e a caixa (desligar sobre uma tela com óleo; ligar sobre uma
/// sem) mudam o traço que vem depois, e a tinta de antes fica ao byte (a mesma história com e sem a mudança). O
/// seguinte pinta-se a mais do que o alcance da óptica (a fenda, 24 células) da tinta de antes: o PAPEL ao pé de um
/// traço mais alto escurece pela fenda dele (a física), a tinta de antes não.
#[test]
fn o_traco_seguinte_usa_o_depth_o_gloss_e_a_caixa_novos() {
    type Muda = fn(&mut PainterTool);
    // `(nome, a tela tem óleo, Smooth Edges da tela, a mudança)`: o Smooth Edges da TELA lê a orla — numa tela que
    // passa a ter óleo, a tinta sem óleo guarda a orla de sempre.
    let controlos: [(&str, bool, f32, Muda); 5] = [
        ("oilDepth", true, 0.0, |t| {
            t.paint.wetpaint.knobs.set(Knob::OilDepth, 2.6)
        }),
        ("oilGloss", true, 0.0, |t| {
            t.paint.wetpaint.knobs.set(Knob::OilGloss, 0.1)
        }),
        ("realOil desligado", true, 0.0, |t| {
            t.set_wet_oleo_real_ligado(false)
        }),
        ("realOil ligado", false, 0.0, |t| {
            t.set_wet_oleo_real_ligado(true)
        }),
        ("realOil ligado com Smooth Edges", false, 0.5, |t| {
            t.set_wet_oleo_real_ligado(true)
        }),
    ];
    let seguinte = |t: &mut PainterTool| {
        t.wetpaint_dry_canvas();
        t.set_brush_color_srgb8([200, 180, 20]);
        pinta(t, 165.0);
        recompoe(t)
    };
    for (nome, oleo, suave, muda) in controlos {
        let com_suave = |mut t: PainterTool| {
            t.set_wet_smooth_edges(suave);
            t
        };
        let sem = seguinte(&mut com_suave(tela(oleo)));
        let mut t = com_suave(tela(oleo));
        // A tinta de antes (e a sombra dela no papel): os píxeis que a tela já mostrava.
        let pousada: Vec<bool> = recompoe(&mut t).chunks(4).map(|p| p != [255; 4]).collect();
        muda(&mut t);
        let com = seguinte(&mut t);
        let mudam: Vec<(usize, usize)> = ANTIGOS
            .filter_map(|y| {
                let n = (0..W)
                    .filter(|&x| {
                        pousada[y * W + x]
                            && com[(y * W + x) * 4..][..4] != sem[(y * W + x) * 4..][..4]
                    })
                    .count();
                (n > 0).then_some((y, n))
            })
            .collect();
        assert!(
            pousada.iter().filter(|p| **p).count() > 3000,
            "{nome}: controlo: há tinta pousada"
        );
        assert!(
            mudam.is_empty(),
            "{nome}: a tinta de antes mudou (linha, píxeis): {mudam:?}"
        );
        assert_ne!(
            linhas(&com, SEGUINTE),
            linhas(&sem, SEGUINTE),
            "{nome}: o traço seguinte não usou o valor novo"
        );
    }
}

/// ⭐ Neutro: pintar a tela TODA com o Real Oil é a imagem de antes (a óptica pela caixa, sobre a tela toda) ao byte;
/// pintar toda sem ele não aloca o campo e o composite é o de sempre (sem óleo).
#[test]
fn pintar_tudo_com_o_real_oil_e_a_imagem_de_antes_ao_byte() {
    super::oleo_plano::sem_cache_do_campo::set(true);
    let mut t = tela(true);
    let nova = recompoe(&mut t);
    assert!(
        grades(&mut t).iter().any(|l| !l.grid.oleo.is_empty()),
        "controlo: o campo existe"
    );
    for l in grades(&mut t).iter_mut() {
        l.grid.oleo = Vec::new();
    }
    super::oleo_plano::lei_da_caixa::set(true);
    let antes = recompoe(&mut t);
    super::oleo_plano::lei_da_caixa::set(false);
    super::oleo_plano::sem_cache_do_campo::set(false);
    let difs = nova
        .chunks(4)
        .zip(antes.chunks(4))
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        nova.chunks(4).any(|p| p != [255, 255, 255, 255]),
        "controlo: há tinta"
    );
    assert_eq!(
        difs, 0,
        "a tela toda com óleo difere da lei de antes em {difs} píxeis"
    );
    let mut t = tela(false);
    let _ = recompoe(&mut t);
    assert!(
        grades(&mut t).iter().all(|l| l.grid.oleo.is_empty()),
        "sem óleo, o campo nasceu"
    );
    assert!(!t.brush_settings().wet_oleo_na_tela);
    assert!(
        t.paint
            .wetpaint
            .session
            .as_ref()
            .is_some_and(|s| s.campo_cache.is_none()),
        "sem óleo na tela, o cache do campo ficou"
    );
}

/// ⭐ O cartão segue a TINTA com óleo: com a caixa do pincel desligada, a tela com óleo mostra os números (que são da
/// tela) e mudar um deles recompõe a tinta com óleo; a memória do campo só se larga sem óleo em célula nenhuma.
#[test]
fn o_cartao_segue_a_tinta_com_oleo_na_tela() {
    let mut t = tela(true);
    t.set_wet_oleo_real_ligado(false);
    let antes = recompoe(&mut t);
    assert!(
        t.brush_settings().wet_oleo_na_tela,
        "a tela tem óleo e o cartão não sabe"
    );
    assert!(
        t.paint
            .wetpaint
            .session
            .as_ref()
            .is_some_and(|s| s.campo_cache.is_some()),
        "a caixa desligada largou o campo da tinta com óleo"
    );
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WETPAINT_OLEO_SOMBRA,
        0.2,
    ));
    for _ in 0..400 {
        t.on_tick(0.0);
        if !t.oleo_por_compor() {
            break;
        }
    }
    assert_ne!(
        t.canvas_rgba.to_vec(),
        antes,
        "a Shadow do cartão não chegou à tinta com óleo"
    );
}

/// O Real Oil viaja com o PRESET (gravado só ligado: os ficheiros de fábrica ficam ao byte) e com a FERRAMENTA.
#[test]
fn o_real_oil_viaja_no_preset_e_por_ferramenta() {
    use super::super::wetpaint_preset::CHAVE_OLEO;
    let mut t = PainterTool::default();
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_oleo_real_ligado(true);
    let base = ph2d_brush_presets::Parametros::novo("WetPaint", "p", "p");
    let com = t.wet_preset_capturar(&base).0;
    assert_eq!(com.engine.get(CHAVE_OLEO), Some(&1.0));
    t.set_wet_oleo_real_ligado(false);
    let sem = t.wet_preset_capturar(&base).0;
    assert!(
        !sem.engine.contains_key(CHAVE_OLEO),
        "desligado gravou a chave"
    );
    let preset = |p| ph2d_brush_presets::Preset {
        parametros: p,
        origem: Default::default(),
        miniatura: ph2d_brush_presets::Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![0; 4],
        },
        texturas: Default::default(),
    };
    t.wet_preset_aplicar(&preset(com));
    assert!(
        t.wet_oleo_real().is_some(),
        "o preset com o Real Oil não o ligou"
    );
    t.wet_preset_aplicar(&preset(sem));
    assert!(
        t.wet_oleo_real().is_none(),
        "o preset sem o Real Oil não o desligou"
    );
    // Por ferramenta (`tool_link`, o idioma do Glaze): ligado no pincel, a Máscara não o herda, voltar devolve-o.
    t.set_wet_oleo_real_ligado(true);
    t.set_paint_tool_mode("mask");
    assert!(t.wet_oleo_real().is_none(), "a outra ferramenta tem o seu");
    t.set_paint_tool_mode("wetpaint");
    assert!(t.wet_oleo_real().is_some(), "volta o do pincel");
}

/// A macro de ANTES (o Depth e o Gloss no cartão, o preset sem `oilDepth`/`oilGloss`): o valor do cartão vira o do
/// PINCEL; com a chave do knob no preset, o preset manda.
#[test]
fn a_macro_com_o_depth_e_o_gloss_no_cartao_passa_os_ao_pincel() {
    let mut t = PainterTool::default();
    t.set_paint_tool_mode("wetpaint");
    let (texto, tex) = t.macro_estado();
    let mut v: serde_json::Value = serde_json::from_str(&texto).expect("json");
    v["wet"]["oleo_real_valores"]["profundidade"] = serde_json::json!(2.2);
    v["wet"]["oleo_real_valores"]["brilho"] = serde_json::json!(0.3);
    let mut antigo = v.clone();
    let eng = antigo["preset"]["engine"].as_object_mut().expect("engine");
    eng.remove("oilDepth");
    eng.remove("oilGloss");
    let mut a = PainterTool::default();
    a.macro_estado_aplica(&antigo.to_string(), tex.clone())
        .expect("aplica");
    assert_eq!(
        (
            a.paint.wetpaint.knobs.get(Knob::OilDepth),
            a.paint.wetpaint.knobs.get(Knob::OilGloss)
        ),
        (2.2, 0.3),
        "o Depth/Gloss do cartão antigo não chegou ao pincel"
    );
    assert_eq!(
        a.wet_oleo_real_valores().profundidade,
        1.0,
        "o ganho global do motor ficou com o cartão"
    );
    let mut b = PainterTool::default();
    b.macro_estado_aplica(&v.to_string(), tex).expect("aplica");
    assert_eq!(
        b.paint.wetpaint.knobs.get(Knob::OilDepth),
        1.0,
        "com a chave no preset, o preset manda"
    );
}

/// A FOLHA (`PH2D_OLEO_PNG=<pasta>`, `--ignored`): `depth_gloss.ppm` — dois traços sobrepostos com o Real Oil, o de
/// baixo Depth 1 · Gloss 1, o de cima Depth 2,6 · Gloss 0,1; `com_sem.ppm` — um traço com e um sem o óleo, encostados.
#[test]
#[ignore = "folha: PH2D_OLEO_PNG=<pasta>"]
fn folha_depth_gloss_e_real_oil_por_traco() {
    let pasta =
        std::path::PathBuf::from(std::env::var("PH2D_OLEO_PNG").expect("PH2D_OLEO_PNG=<pasta>"));
    std::fs::create_dir_all(&pasta).expect("pasta");
    let grava = |nome: &str, c: &[u8]| {
        let mut ppm = format!("P6\n{W} {H}\n255\n").into_bytes();
        for p in c.chunks(4) {
            ppm.extend_from_slice(&p[..3]);
        }
        std::fs::write(pasta.join(nome), ppm).expect("grava");
    };
    let nova = || {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
        t.set_paint_tool_mode("wetpaint");
        t.set_wet_relogio_fixo(true);
        t.set_brush_size_px(22.0);
        t.set_brush_paper_kind(ph2d_painter_brush::TextureKind::PaperCold.to_u8());
        t.paint.wetpaint.knobs.set(Knob::PigmentPerDab, 1800.0);
        t
    };
    let mut t = nova();
    t.set_wet_oleo_real_ligado(true);
    t.set_brush_color_srgb8([180, 40, 30]);
    pinta(&mut t, 62.0);
    t.wetpaint_dry_canvas();
    t.paint.wetpaint.knobs.set(Knob::OilDepth, 2.6);
    t.paint.wetpaint.knobs.set(Knob::OilGloss, 0.1);
    t.set_brush_color_srgb8([40, 80, 190]);
    pinta(&mut t, 80.0);
    grava("depth_gloss.ppm", &recompoe(&mut t));
    let mut t = nova();
    t.set_wet_oleo_real_ligado(true);
    t.set_brush_color_srgb8([180, 40, 30]);
    pinta(&mut t, 58.0);
    t.wetpaint_dry_canvas();
    t.set_wet_oleo_real_ligado(false);
    pinta(&mut t, 84.0);
    grava("com_sem.ppm", &recompoe(&mut t));
}
