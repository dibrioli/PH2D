//! ⭐ **O papel morde por cima do Grain do pincel** (Digital, dono 08/11: *«a pintura parecia chapada mesmo com
//! paper com relevo»*). Antes, com um Grain escolhido ou o Grain Colors, o papel saía da tinta (sonda
//! `sonda_auditoria_papel_grao`: papel + grão = o grão sozinho; papel + Grain Colors = alfa chapado, std 0).

use crate::familias::FAMILIAS;
use crate::presets_de_fabrica::preset_do_meio;
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::PainterTool;

const W: usize = 220;
const H: usize = 80;
const Y: f32 = 40.0;
/// O Grain do pincel: outro papel, noutra escala, para o dente não ser o mesmo desenho.
const GRAO: &str = r#"{"angle_deg": 0, "flow": false, "kind": "PaperCold", "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [64.0, 64.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

#[derive(Clone, Copy)]
struct V {
    papel: bool,
    /// `false` = o Relief do documento desligado (`substrate_depth` 0), com o papel e o Tooth armados.
    relevo: bool,
    cor_grao: bool,
}

fn pinta(v: V) -> Vec<u8> {
    pinta_com(v, true)
}

fn pinta_sem_grao(v: V) -> Vec<u8> {
    pinta_com(v, false)
}

fn pinta_com(v: V, com_grao: bool) -> Vec<u8> {
    pinta_cfg(v, com_grao, false, false)
}

/// `sem_mistura` tira a mistura do preset; `vista` põe o Grain em View (o que a máscara em cache servia).
fn pinta_cfg(v: V, com_grao: bool, sem_mistura: bool, vista: bool) -> Vec<u8> {
    pinta_rt(v, com_grao, sem_mistura, vista, (0.8, 1.6))
}

/// [`pinta_cfg`] com o `(Relief, Tooth)` do papel.
fn pinta_rt(
    v: V,
    com_grao: bool,
    sem_mistura: bool,
    vista: bool,
    (relief, tooth): (f32, f32),
) -> Vec<u8> {
    let r = FAMILIAS
        .iter()
        .flat_map(|f| f.meios.iter())
        .find(|r| r.nome == "Round Mixing")
        .expect("Round Mixing");
    let mut p = preset_do_meio(r);
    if sem_mistura {
        p.parametros.brush.remove("mistura_opaca");
    }
    if com_grao {
        let mut g: serde_json::Value = serde_json::from_str(GRAO).expect("grão");
        if vista {
            g["mapping"] = serde_json::json!("ViewPlane");
        }
        p.parametros.brush.insert("texture".into(), g);
    } else {
        p.parametros.brush.remove("texture");
    }
    p.parametros
        .brush
        .insert("grain_depth".into(), serde_json::json!(0.7));
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; W * H * 4], W as u32, H as u32);
    assert_eq!(t.preset_aplicar(&p), Ok(Vec::new()));
    t.set_brush_size_px(15.0);
    t.set_brush_color_srgb8([0, 160, 60]);
    if v.papel {
        // O Relief arma o papel; desligá-lo depois deixa o papel ATIVO e só o Relief a 0.
        t.set_substrate_depth(relief);
        if !v.relevo {
            t.set_substrate_depth(0.0);
        }
        t.set_brush_paper_depth(tooth);
    }
    if v.cor_grao {
        t.set_texture_ramp(ColorRamp::new(
            vec![
                RampStop::new(0.0, [1.0, 0.0, 0.0, 1.0]),
                RampStop::new(1.0, [0.0, 0.0, 1.0, 1.0]),
            ],
            RampColorMode::Rgb,
            RampInterp::Linear,
        ));
        t.set_texture_ramp_enabled(true);
    }
    let cp = |pos, phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp([20.0, Y], PointerPhase::Down));
    for k in 1..=40 {
        let x = 20.0 + 160.0 * k as f32 / 40.0;
        t.on_canvas_pointer(cp([x, Y], PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp([180.0, Y], PointerPhase::Up));
    t.on_tick(8.0);
    t.run_full().0
}

/// Alfa médio, desvio do alfa e quantos texels levam a cor da rampa (R ou B acima do G), no miolo do traço.
fn miolo(px: &[u8]) -> (f64, f64, usize) {
    let a: Vec<f64> = (Y as usize - 7..Y as usize + 7)
        .flat_map(|y| (40..160).map(move |x| (y, x)))
        .map(|(y, x)| f64::from(px[(y * W + x) * 4 + 3]) / 255.0)
        .collect();
    let n = a.len() as f64;
    let m = a.iter().sum::<f64>() / n;
    let s = (a.iter().map(|x| (x - m).powi(2)).sum::<f64>() / n).sqrt();
    let rampa = (Y as usize - 7..Y as usize + 7)
        .flat_map(|y| (40..160).map(move |x| (y, x)))
        .map(|(y, x)| &px[(y * W + x) * 4..][..4])
        .filter(|p| {
            p[3] > 40
                && (i32::from(p[0]) > i32::from(p[1]) + 40
                    || i32::from(p[2]) > i32::from(p[1]) + 40)
        })
        .count();
    (m, s, rampa)
}

fn hash(px: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    px.hash(&mut h);
    h.finish()
}

/// O alfa do miolo, texel a texel.
fn alfas(px: &[u8]) -> Vec<f64> {
    (Y as usize - 7..Y as usize + 7)
        .flat_map(|y| (40..160).map(move |x| (y, x)))
        .map(|(y, x)| f64::from(px[(y * W + x) * 4 + 3]) / 255.0)
        .collect()
}

/// A correlação de Pearson entre dois alfas — «este traço segue o desenho daquele».
fn correlacao(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len() as f64;
    let (ma, mb) = (a.iter().sum::<f64>() / n, b.iter().sum::<f64>() / n);
    let cov: f64 = a.iter().zip(b).map(|(x, y)| (x - ma) * (y - mb)).sum();
    let va: f64 = a.iter().map(|x| (x - ma).powi(2)).sum();
    let vb: f64 = b.iter().map(|y| (y - mb).powi(2)).sum();
    cov / (va * vb).sqrt().max(1e-12)
}

#[test]
fn o_papel_morde_por_cima_do_grain_do_pincel() {
    let so_grao = pinta(V {
        papel: false,
        relevo: true,
        cor_grao: false,
    });
    let com_papel = pinta(V {
        papel: true,
        relevo: true,
        cor_grao: false,
    });
    // O desenho do papel sozinho: o mesmo pincel sem Grain, onde o papel faz de Grain (o caminho de antes).
    let so_papel = pinta_sem_grao(V {
        papel: true,
        relevo: true,
        cor_grao: false,
    });
    let (m_g, s_g, _) = miolo(&so_grao);
    let (m_pg, s_pg, _) = miolo(&com_papel);
    eprintln!(
        "[dente] grão: alfa {m_g:.3} std {s_g:.3} · papel + grão: alfa {m_pg:.3} std {s_pg:.3}"
    );
    assert!(m_g > 0.2, "o traço do grão sozinho pinta (alfa {m_g:.3})");
    assert!(
        m_pg < m_g - 0.03,
        "o dente tira tinta dos vales: alfa {m_pg:.3} (std {s_pg:.3}) contra {m_g:.3} (std {s_g:.3}) do grão sozinho"
    );
    let (c_g, c_pg) = (
        correlacao(&alfas(&so_grao), &alfas(&so_papel)),
        correlacao(&alfas(&com_papel), &alfas(&so_papel)),
    );
    eprintln!("[dente] correlação com o papel: grão {c_g:.3} · papel + grão {c_pg:.3}");
    assert!(
        c_pg > c_g + 0.2,
        "o traço passa a seguir o desenho do papel: correlação {c_pg:.3} contra {c_g:.3} do grão sozinho"
    );
}

#[test]
fn o_grain_colors_com_papel_nao_sai_chapado() {
    let px = pinta(V {
        papel: true,
        relevo: true,
        cor_grao: true,
    });
    let (m, s, rampa) = miolo(&px);
    eprintln!("[dente] papel + Grain Colors: alfa {m:.3} std {s:.3} rampa {rampa}");
    assert!(m > 0.2, "o traço pinta (alfa {m:.3})");
    assert!(
        s > 0.05,
        "o papel morde o alfa do Grain Colors: std {s:.3} (antes 0, chapado)"
    );
    assert!(
        rampa > 100,
        "o grão continua a dar a COR da rampa: {rampa} texels"
    );
}

#[test]
fn sem_relevo_o_grain_e_o_grain_colors_ficam_ao_byte() {
    for cor_grao in [false, true] {
        let v = V {
            papel: true,
            relevo: false,
            cor_grao,
        };
        ph2d_painter_brush::dente::sonda_sem_dente(true);
        let antes = hash(&pinta(v));
        ph2d_painter_brush::dente::sonda_sem_dente(false);
        let agora = hash(&pinta(v));
        assert_eq!(
            agora, antes,
            "Relief desligado (Grain Colors {cor_grao}): o traço de antes, ao byte"
        );
    }
}

/// ⭐ **O efeito do papel na tinta cresce com o Relief, e o Tooth multiplica-o** (dono 08/11: *«independente da
/// intensidade do relevo de paper, o efeito sobre a tinta é o mesmo»* · o multiplicador é o Tooth). Nos DOIS
/// caminhos do Digital: o papel como Grain (pincel sem Grain) e o dente por cima do Grain do pincel. A régua é o
/// alfa médio do miolo (a tinta que o papel tira dos vales; o desvio não serve: com Grain, mais papel ALISA a soma
/// das duas texturas). Antes: Relief 0,2 = 0,5 = 0,8 = 1, ao byte.
#[test]
fn o_efeito_do_papel_cresce_com_o_relief_e_o_tooth() {
    let v = V {
        papel: true,
        relevo: true,
        cor_grao: false,
    };
    for com_grao in [false, true] {
        let alfa_de = |rt| miolo(&pinta_rt(v, com_grao, false, false, rt)).0;
        let por_relief: Vec<f64> = [0.2, 0.5, 0.8, 1.0].map(|r| alfa_de((r, 1.0))).to_vec();
        let (fraco, forte) = (alfa_de((0.5, 0.5)), alfa_de((0.5, 2.0)));
        eprintln!(
            "[relief] Grain {com_grao}: alfa por Relief 0,2/0,5/0,8/1 (Tooth 1) {por_relief:.3?} · Relief 0,5 Tooth 0,5 {fraco:.3} · Tooth 2 {forte:.3}"
        );
        for par in por_relief.windows(2) {
            assert!(
                par[1] < par[0] - 0.005,
                "Grain {com_grao}: mais Relief, mais papel na tinta: {por_relief:.3?}"
            );
        }
        assert!(
            forte < fraco - 0.01,
            "Grain {com_grao}: o Tooth multiplica o efeito: {fraco:.3} (0,5) contra {forte:.3} (2)"
        );
    }
}

/// O custo do dente (A × B no MESMO processo, intercalado, o MÍNIMO de 9): um traço de grão com o papel,
/// com e sem o dente. Corre à mão: `cargo test --profile smoke -p ph2d-tool-painter --test it
/// sonda_custo_do_dente -- --ignored --nocapture`.
#[test]
#[ignore = "sonda de custo"]
fn sonda_custo_do_dente() {
    let v = V {
        papel: true,
        relevo: true,
        cor_grao: false,
    };
    for (nome, sem_mistura, vista) in [
        ("Round Mixing, Grain Tiled", false, false),
        ("Round Mixing sem mistura, Grain Tiled", true, false),
        ("Round Mixing sem mistura, Grain View", true, true),
    ] {
        let mut min = [f64::MAX; 2];
        for k in 0..18 {
            let sem = k % 2 == (k / 2) % 2;
            ph2d_painter_brush::dente::sonda_sem_dente(sem);
            let t0 = std::time::Instant::now();
            std::hint::black_box(pinta_cfg(v, true, sem_mistura, vista));
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            let i = usize::from(sem);
            min[i] = min[i].min(ms);
        }
        ph2d_painter_brush::dente::sonda_sem_dente(false);
        eprintln!(
            "[custo] {nome} + papel: com dente {:.2} ms · sem {:.2} ms (mín. de 9)",
            min[0], min[1]
        );
    }
}

/// A foto do Relief × Tooth para o dono: grava `<PH2D_AUD_OUT>/relief_<grão>_<relief>_<tooth>.rgba` (220×80). Corre à
/// mão: `PH2D_AUD_OUT=<pasta> cargo test --profile smoke -p ph2d-tool-painter --test it sonda_foto_relief_tooth -- --ignored`.
#[test]
#[ignore = "sonda de foto"]
fn sonda_foto_relief_tooth() {
    let o = std::path::PathBuf::from(std::env::var("PH2D_AUD_OUT").expect("PH2D_AUD_OUT"));
    std::fs::create_dir_all(&o).expect("pasta");
    let v = V {
        papel: true,
        relevo: true,
        cor_grao: false,
    };
    for com_grao in [false, true] {
        for rt in [
            (0.2, 1.0),
            (0.5, 1.0),
            (0.8, 1.0),
            (1.0, 1.0),
            (0.5, 0.5),
            (0.5, 2.0),
        ] {
            let px = pinta_rt(v, com_grao, false, false, rt);
            let nome = format!("relief_{}_{:.1}_{:.1}.rgba", u8::from(com_grao), rt.0, rt.1);
            std::fs::write(o.join(nome), px).expect("grava");
        }
    }
}
