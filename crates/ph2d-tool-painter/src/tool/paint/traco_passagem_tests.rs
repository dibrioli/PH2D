//! ⭐ **O nível de uma passagem não depende de onde o grão cai** (bug #58, 6.ª volta): com um grão grosso preso à
//! tela, o mesmo traço com o começo num PICO e num VALE do grão dá o mesmo nível `L` (±2 %), e o miolo do Wet é o do
//! seco nos dois (7.ª volta: o Wet é o aro da aguada, o miolo não mexe; a 5.ª/6.ª exigiam o patamar `0,92` do
//! Procreate, que saiu). A sonda ao lado mede os candidatos (o máximo do 1.º diâmetro, um percentil num
//! troço mais longo, o nível sem grão, o nível no grão médio) sobre os pincéis do dono.
//!
//! Tudo na tela TRANSPARENTE: o alfa da tela é a cobertura composta.

use super::traco_orla_bordas_tests::{H, W};
use super::traco_orla_lei_tests::{caminho, cp, pinta};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};

type Cfg = dyn Fn(&mut PainterTool);

/// O raio (px) do pincel das medições.
const R: f32 = 20.0;
/// O comprimento (px) do recto.
const COMPRIMENTO: f32 = 200.0;

fn monta(m: f32, glaze: bool, f: &dyn Fn(&mut PainterTool)) -> PainterTool {
    let mut t = if glaze {
        super::traco_orla_bordas_tests::glaze_round(|_| {})
    } else {
        let mut t = PainterTool::default();
        t.set_paint_media(crate::PaintMedia::Digital);
        t.set_brush_strength(0.6);
        t.set_traco_tecto(1.0);
        t
    };
    t.set_source(vec![0u8; W * H * 4], W as u32, H as u32);
    t.set_brush_size_px(2.0 * R);
    t.set_brush_color_srgb8([230, 90, 180]);
    t.set_traco_molhada(m);
    f(&mut t);
    t
}

/// Um recto horizontal de `x0` a `x0 + COMPRIMENTO` na altura `y` (`pressao`: a rampa 0,3 → 1); devolve o nível.
fn recto(t: &mut PainterTool, x0: f32, y: f32, pressao: bool) -> f32 {
    let pts = caminho(&[[x0, y], [x0 + COMPRIMENTO, y]]);
    if pressao {
        let n = pts.len();
        t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
        for (k, p) in pts[1..].iter().enumerate() {
            let mut e = cp(*p, PointerPhase::Move);
            e.pressure = 0.3 + 0.7 * (k as f32 / n as f32);
            t.on_canvas_pointer(e);
            if k % 4 == 3 {
                t.paint_tick(1.0 / 60.0);
            }
        }
        t.paint_tick(1.0 / 60.0);
    } else {
        pinta(t, &pts, |_| 4);
    }
    let l = 2.0
        * t.paint
            .plano_do_traco
            .as_ref()
            .map_or(0.0, |s| s.fronteira());
    t.on_canvas_pointer(cp(*pts.last().unwrap(), PointerPhase::Up));
    l
}

/// O que um recto mede.
struct Medida {
    /// O nível do motor.
    l: f32,
    /// O máximo da cobertura seca no 1.º diâmetro (a lei velha).
    velho: f32,
    /// O percentil 90 da cobertura seca na faixa do eixo dos 3 primeiros diâmetros.
    p90: f32,
    /// A cobertura seca (o alfa da tela) e a faixa do miolo.
    seco: Vec<f32>,
    faixa: Vec<usize>,
    /// O Wet 1 do motor sobre o seco, na faixa.
    razao: f32,
}

fn mede(
    glaze: bool,
    f: &dyn Fn(&mut PainterTool),
    x0: f32,
    y: f32,
    pressao: bool,
    meia_faixa: f32,
) -> Medida {
    let corre = |m: f32| {
        let mut t = monta(m, glaze, f);
        let l = recto(&mut t, x0, y, pressao);
        let a: Vec<f32> = t
            .canvas_rgba
            .chunks(4)
            .map(|p| f32::from(p[3]) / 255.0)
            .collect();
        (l, a)
    };
    let (_, seco) = corre(0.0);
    let (l, molhado) = corre(1.0);
    let r = monta(0.0, glaze, f).paint.brush.clamped_radius();
    let faixa: Vec<usize> = (0..W * H)
        .filter(|&i| {
            let (px, py) = ((i % W) as f32 + 0.5, (i / W) as f32 + 0.5);
            (py - y).abs() <= meia_faixa && px >= x0 + 2.0 * r && px <= x0 + COMPRIMENTO - 2.0 * r
        })
        .collect();
    let velho = (0..W * H)
        .filter(|&i| ((i % W) as f32) < x0 + 3.0 * r)
        .map(|i| seco[i])
        .fold(0.0f32, f32::max);
    let mut eixo: Vec<f32> = (0..W * H)
        .filter(|&i| {
            let (px, py) = ((i % W) as f32 + 0.5, (i / W) as f32 + 0.5);
            (py - y).abs() <= meia_faixa && px <= x0 + 6.0 * r
        })
        .map(|i| seco[i])
        .collect();
    eixo.sort_by(f32::total_cmp);
    let p90 = eixo[eixo.len() * 9 / 10];
    let soma = |a: &[f32]| faixa.iter().map(|&i| a[i]).sum::<f32>();
    let razao = soma(&molhado) / soma(&seco);
    Medida {
        l,
        velho,
        p90,
        seco,
        faixa,
        razao,
    }
}

/// O grão Noise de Size `tam` (0,25 ≈ 64 px), Depth 0,35, no mapa `mapa` (0 View, 1 Tiled).
fn grao(tam: f32, mapa: u8) -> impl Fn(&mut PainterTool) {
    move |t: &mut PainterTool| {
        t.new_brush_texture();
        t.set_brush_texture_mapping(mapa);
        t.set_brush_texture_size_norm(0, tam);
        t.set_brush_texture_size_norm(1, tam);
        t.set_brush_grain_depth(0.35);
    }
}

/// O maior grão sob o 1.º diâmetro de um recto que começa em `(x0, y)` (o que o máximo velho lia).
fn grao_no_comeco(t: &PainterTool, x0: f32, y: f32) -> f32 {
    let r = t.paint.brush.clamped_radius();
    let b = &t.paint.brush;
    let base = ph2d_painter_brush::texture::dab_basis(
        &b.texture,
        &mut 0u64,
        [W as f32, H as f32],
        b.footprint_deform(),
    );
    let mut m = 0.0f32;
    for yy in (y - 0.5 * r) as i64..(y + 0.5 * r) as i64 {
        for xx in x0 as i64..(x0 + 2.0 * r) as i64 {
            m = m.max(ph2d_painter_brush::dab::grain_at(
                b,
                &base,
                None,
                xx,
                yy,
                [x0, y],
                r,
            ));
        }
    }
    m
}

/// O grão Tiled de Size `tam` e Depth `fundo`, dureza 0,9; o começo no PICO e no VALE do grão (o grão médio sob o
/// 1.º diâmetro, `x0` em 20..100): `(grão no pico, no vale, a medida no pico, no vale)`.
fn pico_e_vale(tipo: u8, tam: f32, fundo: f32, diametro: f32) -> (f32, f32, Medida, Medida) {
    let f = move |t: &mut PainterTool| {
        t.set_brush_size_px(diametro);
        grao(tam, 1)(t);
        t.set_brush_texture_kind(tipo);
        t.set_brush_texture_mapping(1);
        t.set_brush_texture_size_norm(0, tam);
        t.set_brush_texture_size_norm(1, tam);
        t.set_brush_grain_depth(fundo);
        t.paint.brush.hardness = 0.9;
    };
    let sonda = monta(1.0, false, &f);
    let y = H as f32 / 2.0;
    let comecos: Vec<(f32, f32)> = (0..=40)
        .map(|k| {
            let x0 = 20.0 + k as f32 * 2.0;
            (x0, grao_no_comeco(&sonda, x0, y))
        })
        .collect();
    let por_grao = |a: &(f32, f32), b: &(f32, f32)| a.1.total_cmp(&b.1);
    let pico = comecos.iter().copied().max_by(por_grao).unwrap();
    let vale = comecos.iter().copied().min_by(por_grao).unwrap();
    (
        pico.1,
        vale.1,
        mede(false, &f, pico.0, y, false, 0.3 * diametro),
        mede(false, &f, vale.0, y, false, 0.3 * diametro),
    )
}

/// ⭐ **O gate**: Clouds Tiled (Size 0,1, Depth 0,35) sob um pincel de 12 px — o grão de baixa frequência que o
/// 1.º diâmetro lê como pico ou vale —, o começo no pico e no vale: `L` a ±2 % e o miolo do Wet 1 o do seco (±1 %).
#[test]
fn o_nivel_de_uma_passagem_nao_depende_de_onde_o_grao_cai() {
    let (gp, gv, a, b) = pico_e_vale(6, 0.1, 0.35, 12.0);
    eprintln!(
        "pico (grão {gp:.3}): L {:.3} velho {:.3} Wet/seco {:.3} | vale (grão {gv:.3}): L {:.3} velho {:.3} Wet/seco {:.3}",
        a.l, a.velho, a.razao, b.l, b.velho, b.razao
    );
    assert!(
        gp - gv > 0.05,
        "a fixtura não tem pico nem vale sob o começo: {gp} {gv}"
    );
    assert!(
        (a.l - b.l).abs() <= 0.02 * a.l.max(b.l),
        "o nível depende do começo: pico {} × vale {}",
        a.l,
        b.l
    );
    for (nome, m) in [("pico", &a), ("vale", &b)] {
        assert!(
            (m.razao - 1.0).abs() <= 0.01,
            "{nome}: o miolo do Wet é {} do seco",
            m.razao
        );
    }
}

/// Sonda: a fixtura do gate por Size × Depth.
#[test]
#[ignore = "sonda"]
fn sonda_pico_e_vale() {
    for (tipo, tam, d) in [
        (1u8, 0.05f32, 12.0f32),
        (1, 0.25, 12.0),
        (6, 0.1, 12.0),
        (6, 0.25, 12.0),
        (6, 0.25, 40.0),
    ] {
        for fundo in [0.35f32, 0.7] {
            let (gp, gv, a, b) = pico_e_vale(tipo, tam, fundo, d);
            eprintln!(
                "tipo {tipo} Size {tam} ⌀ {d} Depth {fundo}: grão {gp:.3}/{gv:.3} · L {:.3}/{:.3} · velho {:.3}/{:.3} · Wet/seco {:.3}/{:.3}",
                a.l, b.l, a.velho, b.velho, a.razao, b.razao
            );
        }
    }
}

/// Sonda: os candidatos sobre os pincéis do dono, em quatro sítios do recto.
#[test]
#[ignore = "sonda"]
fn sonda_nivel_contra_o_grao() {
    let tiled = || grao(0.25, 1);
    let casos: Vec<(&str, bool, Box<Cfg>, bool)> = vec![
        ("sem grão", false, Box::new(|_: &mut PainterTool| {}), false),
        ("Noise fino View", false, Box::new(grao(0.05, 0)), false),
        ("Noise grosso View", false, Box::new(grao(0.5, 0)), false),
        ("Noise fino Tiled", false, Box::new(grao(0.05, 1)), false),
        ("Noise grosso Tiled", false, Box::new(tiled()), false),
        (
            "Glaze Round",
            true,
            Box::new(|_: &mut PainterTool| {}),
            false,
        ),
        ("Glaze Round + grosso Tiled", true, Box::new(tiled()), false),
        (
            "Flow 0,3 Acc off + grão",
            false,
            Box::new(move |t: &mut PainterTool| {
                tiled()(t);
                t.set_brush_flow(0.3);
            }),
            false,
        ),
        (
            "Flow 0,3 Acc on + grão",
            false,
            Box::new(move |t: &mut PainterTool| {
                tiled()(t);
                t.set_brush_flow(0.3);
                t.paint.brush.accumulate = true;
            }),
            false,
        ),
        (
            "S 0,4 + grão",
            false,
            Box::new(move |t: &mut PainterTool| {
                tiled()(t);
                t.set_brush_strength(0.4);
            }),
            false,
        ),
        (
            "S 1 + grão",
            false,
            Box::new(move |t: &mut PainterTool| {
                tiled()(t);
                t.set_brush_strength(1.0);
            }),
            false,
        ),
        ("rampa de pressão + grão", false, Box::new(tiled()), true),
    ];
    for (nome, glaze, f, pressao) in &casos {
        for (x0, y) in [
            (40.0f32, 120.0f32),
            (60.0, 90.0),
            (80.0, 150.0),
            (100.0, 105.0),
        ] {
            let m = mede(*glaze, f.as_ref(), x0, y, *pressao, 6.0);
            let livre = {
                let g = |t: &mut PainterTool| {
                    f(t);
                    t.paint.brush.texture.kind = Default::default();
                };
                let mut t = monta(1.0, *glaze, &g);
                recto(&mut t, x0, y, *pressao)
            };
            let media = m.faixa.iter().map(|&i| m.seco[i]).sum::<f32>() / m.faixa.len() as f32;
            eprintln!(
                "{nome} @({x0},{y}) miolo {media:.3} | velho L {:.3} | p90 L {:.3} | sem grão L {livre:.3} | novo L {:.3} (Wet/seco {:.3})",
                m.velho, m.p90, m.l, m.razao
            );
        }
    }
}
