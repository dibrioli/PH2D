//! Sonda (08/11, auditoria): os 13 pincéis do Wet blend × Paper / Grain / Shape Color / Grain Colors, a mistura
//! ligada e desligada, numa cena 600×300 (vazio · branco · listras). Corre à mão:
//! `PH2D_AUD_OUT=<pasta> cargo test --profile smoke -p ph2d-tool-painter --test it sonda_auditoria_papel_grao -- --ignored`.
//! Grava `<pasta>/<pincel>__<variante>.rgba` e `<pasta>/medidas.tsv`.

use crate::familias::FAMILIAS;
use crate::presets_de_fabrica::preset_do_meio;
use crate::receitas_de_fabrica::ReceitaMeio;
use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::PainterTool;
use std::fmt::Write as _;
use std::path::PathBuf;

const W: usize = 600;
const H: usize = 300;
const YA: f32 = 80.0; // traço A: listras → branco → vazio (direita → esquerda)
const YB: f32 = 220.0; // traço B: só no vazio (esquerda → direita)
const PINCEIS: [&str; 13] = [
    "Round Mixing",
    "Gouache",
    "Dry Gouache",
    "Thin Gouache",
    "Gouache Blender",
    "Oil Paint",
    "Paint And Apply",
    "Thick Oil Paint",
    "Crayon",
    "Pastel",
    "Chalk",
    "Finger Tip",
    "Pointillism",
];
const PAPEL: &str = r#"{"angle_deg": 0, "flow": false, "kind": "PaperCold", "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [128.0, 128.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

fn receita(nome: &str) -> &'static ReceitaMeio {
    FAMILIAS
        .iter()
        .flat_map(|f| f.meios.iter())
        .find(|r| r.nome == nome)
        .unwrap_or_else(|| panic!("receita {nome} não existe"))
}

fn base() -> Vec<u8> {
    let mut b = vec![0u8; W * H * 4];
    for y in 0..H {
        for x in 200..W {
            let c: [u8; 4] = if x < 400 {
                [255, 255, 255, 255]
            } else {
                match ((x - 400) / 20) % 3 {
                    0 => [220, 30, 30, 255],
                    1 => [30, 60, 220, 255],
                    _ => [240, 220, 30, 255],
                }
            };
            b[(y * W + x) * 4..][..4].copy_from_slice(&c);
        }
    }
    b
}

fn traca(t: &mut PainterTool, x0: f32, x1: f32, y: f32) {
    let cp = |pos, phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let n = ((x1 - x0).abs() / 4.0) as usize;
    t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
    for k in 1..=n {
        let x = x0 + (x1 - x0) * k as f32 / n as f32;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp([x1, y], PointerPhase::Up));
    t.on_tick(8.0);
}

fn rampa() -> ColorRamp {
    ColorRamp::new(
        vec![
            RampStop::new(0.0, [1.0, 0.0, 0.0, 1.0]),
            RampStop::new(1.0, [0.0, 0.0, 1.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    )
}

#[derive(Default, Clone, Copy)]
struct V {
    nome: &'static str,
    off: bool,
    grao: bool,
    sem_grao: bool,
    papel: bool,
    cor_grao: bool,
    cor_shape: bool,
    amount: Option<f32>,
    raio: Option<f32>,
}

const VARIANTES: [V; 26] = {
    const Z: V = V {
        nome: "",
        off: false,
        grao: false,
        sem_grao: false,
        papel: false,
        cor_grao: false,
        cor_shape: false,
        amount: None,
        raio: None,
    };
    [
        V { nome: "base", ..Z },
        V { nome: "base2", ..Z },
        V {
            nome: "off",
            off: true,
            ..Z
        },
        V {
            nome: "liso",
            sem_grao: true,
            ..Z
        },
        V {
            nome: "liso_off",
            sem_grao: true,
            off: true,
            ..Z
        },
        V {
            nome: "grao",
            grao: true,
            ..Z
        },
        V {
            nome: "grao_off",
            grao: true,
            off: true,
            ..Z
        },
        V {
            nome: "grao_cor",
            grao: true,
            cor_grao: true,
            ..Z
        },
        V {
            nome: "grao_cor_off",
            grao: true,
            cor_grao: true,
            off: true,
            ..Z
        },
        V {
            nome: "shape_cor",
            cor_shape: true,
            ..Z
        },
        V {
            nome: "shape_cor_off",
            cor_shape: true,
            off: true,
            ..Z
        },
        V {
            nome: "papel",
            papel: true,
            sem_grao: true,
            ..Z
        },
        V {
            nome: "papel_off",
            papel: true,
            sem_grao: true,
            off: true,
            ..Z
        },
        V {
            nome: "grao_a0",
            grao: true,
            amount: Some(0.0),
            ..Z
        },
        V {
            nome: "grao_a010",
            grao: true,
            amount: Some(0.10),
            ..Z
        },
        V {
            nome: "grao_a025",
            grao: true,
            amount: Some(0.25),
            ..Z
        },
        V {
            nome: "grao_a050",
            grao: true,
            amount: Some(0.50),
            ..Z
        },
        V {
            nome: "r6",
            raio: Some(6.0),
            ..Z
        },
        V {
            nome: "r40",
            raio: Some(40.0),
            ..Z
        },
        V {
            nome: "papel_grao",
            papel: true,
            grao: true,
            ..Z
        },
        V {
            nome: "papel_grao_off",
            papel: true,
            grao: true,
            off: true,
            ..Z
        },
        V {
            nome: "papel_grao_cor",
            papel: true,
            grao: true,
            cor_grao: true,
            ..Z
        },
        V {
            nome: "papel_grao_cor_off",
            papel: true,
            grao: true,
            cor_grao: true,
            off: true,
            ..Z
        },
        V {
            nome: "papel_preset",
            papel: true,
            ..Z
        },
        V {
            nome: "papel_preset_off",
            papel: true,
            off: true,
            ..Z
        },
        V {
            nome: "grao_cor_a010",
            grao: true,
            cor_grao: true,
            amount: Some(0.10),
            ..Z
        },
    ]
};

fn pinta(nome: &str, v: V, base: &[u8]) -> (Vec<u8>, bool, f32) {
    let r = receita(nome);
    let mut p = preset_do_meio(r);
    let tinha_mistura = p.parametros.brush.contains_key("mistura_opaca");
    if v.off {
        p.parametros.brush.remove("mistura_opaca");
    }
    if let Some(a) = v.amount
        && let Some(o) = p.parametros.brush.get_mut("mistura_opaca")
    {
        let mut obj: serde_json::Value = match &*o {
            serde_json::Value::String(s) => serde_json::from_str(s).expect("mistura"),
            x => x.clone(),
        };
        obj["paint_amount"] = serde_json::json!(a);
        *o = obj;
    }
    if v.sem_grao {
        p.parametros.brush.remove("texture");
    }
    if v.grao {
        p.parametros.brush.insert(
            "texture".into(),
            serde_json::from_str(PAPEL).expect("papel"),
        );
        p.parametros
            .brush
            .insert("grain_depth".into(), serde_json::json!(0.7));
    }
    let mut t = PainterTool::default();
    t.set_source(base.to_vec(), W as u32, H as u32);
    assert_eq!(t.preset_aplicar(&p), Ok(Vec::new()), "{nome}");
    let raio = v.raio.unwrap_or(15.0);
    t.sonda_raio(raio);
    t.set_brush_color_srgb8([0, 160, 60]);
    if v.papel {
        t.set_substrate_depth(0.8);
        t.set_brush_paper_depth(1.6);
    }
    if v.cor_grao {
        t.set_texture_ramp(rampa());
        t.set_texture_ramp_enabled(true);
    }
    if v.cor_shape {
        t.set_shape_color_ramp(rampa());
        t.set_shape_ramp_enabled(true);
    }
    traca(&mut t, 580.0, 20.0, YA);
    traca(&mut t, 20.0, 180.0, YB);
    (t.run_full().0, tinha_mistura, raio)
}

struct Reg {
    a_media: f64,
    a_std: f64,
    lum_std: f64,
    rgb: [f64; 3],
    nao_verde: usize, // texels onde R ou B passam G por 40 (a cor da rampa)
    n: usize,
}

fn regiao(px: &[u8], x0: usize, x1: usize, yc: f32, r: f32) -> Reg {
    let h = (r * 0.5).max(1.0);
    let (y0, y1) = ((yc - h) as usize, (yc + h) as usize);
    let mut v = Vec::new();
    for y in y0..y1 {
        for x in x0..x1 {
            v.push(&px[(y * W + x) * 4..][..4]);
        }
    }
    let n = v.len() as f64;
    let a: Vec<f64> = v.iter().map(|p| f64::from(p[3]) / 255.0).collect();
    let lum: Vec<f64> = v
        .iter()
        .map(|p| {
            let al = f64::from(p[3]) / 255.0;
            (0.3 * f64::from(p[0]) + 0.59 * f64::from(p[1]) + 0.11 * f64::from(p[2])) * al
        })
        .collect();
    let media = |s: &[f64]| s.iter().sum::<f64>() / n;
    let std = |s: &[f64]| {
        let m = media(s);
        (s.iter().map(|x| (x - m).powi(2)).sum::<f64>() / n).sqrt()
    };
    let mut rgb = [0.0; 3];
    let mut wa = 0.0;
    for p in &v {
        let al = f64::from(p[3]);
        wa += al;
        for k in 0..3 {
            rgb[k] += f64::from(p[k]) * al;
        }
    }
    if wa > 0.0 {
        rgb = rgb.map(|c| c / wa);
    }
    let nao_verde = v
        .iter()
        .filter(|p| p[3] > 40)
        .filter(|p| {
            let g = i32::from(p[1]);
            i32::from(p[0]) > g + 40 || i32::from(p[2]) > g + 40
        })
        .count();
    Reg {
        a_media: media(&a),
        a_std: std(&a),
        lum_std: std(&lum),
        rgb,
        nao_verde,
        n: v.len(),
    }
}

#[test]
#[ignore = "sonda"]
fn sonda_auditoria_papel_grao() {
    let _ = ph2d_tool_painter::raio_da_sonda::perfil_da_sonda();
    let o = PathBuf::from(std::env::var("PH2D_AUD_OUT").expect("PH2D_AUD_OUT"));
    std::fs::create_dir_all(&o).expect("pasta");
    let b = base();
    let so = std::env::var("PH2D_AUD_SO").ok();
    let mut tsv = String::from(
        "pincel\tvariante\tmistura\traio\thash\tnan_ou_vazio\t\
         A_listras_rgb\tA_branco_a\tA_branco_lumstd\tA_branco_rgb\tA_branco_naoverde\tA_vazio_a\tA_vazio_astd\tA_vazio_rgb\t\
         B_ini_a\tB_ini_rgb\tB_fim_a\tB_fim_rgb\tB_astd\tB_naoverde\tB_n\n",
    );
    let f = |c: [f64; 3]| format!("{:.0},{:.0},{:.0}", c[0], c[1], c[2]);
    for nome in PINCEIS {
        if so
            .as_deref()
            .is_some_and(|s| !s.split(',').any(|n| n == nome))
        {
            continue;
        }
        let so_var = std::env::var("PH2D_AUD_VAR").ok();
        for v in VARIANTES {
            if so_var
                .as_deref()
                .is_some_and(|s| !s.split(',').any(|n| n == v.nome))
            {
                continue;
            }
            let t0 = std::time::Instant::now();
            let (px, mist, r) = pinta(nome, v, &b);
            let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
            for &x in &px {
                hash = (hash ^ u64::from(x)).wrapping_mul(0x100_0000_01b3);
            }
            let fora = px
                .chunks(4)
                .filter(|p| p[3] == 0 && (p[0] | p[1] | p[2]) != 0)
                .count();
            let al = regiao(&px, 410, 580, YA, r);
            let ab = regiao(&px, 220, 380, YA, r);
            let av = regiao(&px, 30, 180, YA, r);
            let bi = regiao(&px, 25, 55, YB, r);
            let bf = regiao(&px, 145, 175, YB, r);
            let bt = regiao(&px, 25, 175, YB, r);
            let _ = writeln!(
                tsv,
                "{nome}\t{}\t{mist}\t{r}\t{hash:016x}\t{fora}\t{}\t{:.3}\t{:.1}\t{}\t{}\t{:.3}\t{:.3}\t{}\t{:.3}\t{}\t{:.3}\t{}\t{:.3}\t{}\t{}",
                v.nome,
                f(al.rgb),
                ab.a_media,
                ab.lum_std,
                f(ab.rgb),
                ab.nao_verde,
                av.a_media,
                av.a_std,
                f(av.rgb),
                bi.a_media,
                f(bi.rgb),
                bf.a_media,
                f(bf.rgb),
                bt.a_std,
                bt.nao_verde,
                bt.n,
            );
            std::fs::write(
                o.join(format!("{}__{}.rgba", nome.replace(' ', "_"), v.nome)),
                &px,
            )
            .expect("grava");
            eprintln!("[aud] {nome} {} {:.1}s", v.nome, t0.elapsed().as_secs_f64());
        }
        std::fs::write(o.join("medidas.tsv"), &tsv).expect("tsv");
    }
    std::fs::write(o.join("medidas.tsv"), &tsv).expect("tsv");
}
