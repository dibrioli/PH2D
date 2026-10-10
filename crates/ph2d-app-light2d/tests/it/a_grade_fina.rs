//! ⭐ **A grade FINA da luz direta** (W4b), do lado da família: a célula fina não depende do nível,
//! e a REGIÃO que se calcula cobre tudo o que o receptor lê na janela.

use ph2d_app_light2d::{fina_do_quadro, grade_do_quadro, qualidade::degrau_do, regiao_da_janela};
use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_host::WindowSize;
use ph2d_light2d_gpu::Config;
use ph2d_render::Camera2d;
use vello::kurbo::{Affine, Point};

const FHD: (u32, u32) = (1920, 1080);

/// As vistas: a de fábrica, zooms dentro e entre oitavas, e pans que não caem na rede.
fn vistas() -> Vec<Affine> {
    let mut out = Vec::new();
    for zoom in [1.0_f32, 1.37, 0.61, 2.9, 0.23] {
        for centro in [[0.0_f32, 0.0], [3.17, -1.41], [-12.05, 7.3]] {
            let mut c = Camera2d::default();
            c.center = centro;
            c.zoom(zoom);
            out.push(c.world_to_screen_affine(WindowSize::new(FHD.0, FHD.1)));
        }
    }
    out
}

/// ⭐⭐ **A célula fina tem 1–2 px de tela em TODO nível e zoom** (a escada some em Low como em
/// High): a fina e a das cascatas são potências de 2 em metros, e a razão entre elas é exata.
#[test]
fn a_celula_fina_e_de_um_a_dois_px_em_todo_nivel() {
    for afim in vistas() {
        let ppm = afim.determinant().abs().sqrt();
        for nivel in NivelDaLuz::ALL {
            let d = degrau_do(nivel);
            let g = grade_do_quadro(afim, FHD, d).expect("grade");
            let fina = fina_do_quadro(afim, d);
            let px = g.texel_m / f64::from(fina) * ppm;
            assert!(fina.is_power_of_two(), "{nivel:?}: fina {fina}");
            assert!(
                (1.0..2.0 + 1e-9).contains(&px),
                "{nivel:?} a {ppm:.1} px/m: o texel fino tem {px:.3} px de tela"
            );
        }
    }
}

/// ⭐⭐⭐ **A região cobre a janela**: para todo píxel, os DOIS texels de cada eixo que a bilinear do
/// receptor lê estão dentro de `[início, fim)` — fora dela a textura guarda um quadro velho.
#[test]
fn a_regiao_cobre_tudo_o_que_o_receptor_le_na_janela() {
    let mut pior_folga = f64::INFINITY;
    for afim in vistas() {
        let inv = afim.inverse();
        for nivel in NivelDaLuz::ALL {
            let d = degrau_do(nivel);
            let cfg = Config {
                fina: fina_do_quadro(afim, d),
                ..Config::padrao(grade_do_quadro(afim, FHD, d).expect("grade"))
            };
            let g = cfg.grade_da_fluencia();
            let (ini, fim) = regiao_da_janela(g, afim, FHD);
            assert!(
                fim[0] < g.largura || fim[1] < g.altura || ini != [0, 0],
                "controlo: a região tem de ser menor que a grade ({nivel:?})"
            );
            let passo = 7;
            let xs = (0..=FHD.0).step_by(passo).chain([FHD.0]);
            for x in xs {
                for y in (0..=FHD.1).step_by(passo).chain([FHD.1]) {
                    let p = inv * Point::new(f64::from(x), f64::from(y));
                    let t = [
                        (p.x - g.origem_m[0]) / g.texel_m - 0.5,
                        (p.y - g.origem_m[1]) / g.texel_m - 0.5,
                    ];
                    for k in 0..2 {
                        let (a, b) = (t[k].floor(), t[k].floor() + 1.0);
                        assert!(
                            a >= f64::from(ini[k]) && b < f64::from(fim[k]),
                            "{nivel:?}: o píxel ({x}, {y}) lê os texels {a}..={b} no eixo {k}, fora de [{}, {})",
                            ini[k],
                            fim[k]
                        );
                        pior_folga = pior_folga.min(a - f64::from(ini[k]));
                        pior_folga = pior_folga.min(f64::from(fim[k]) - 1.0 - b);
                    }
                }
            }
        }
    }
    eprintln!(
        "região: a menor folga entre o que o receptor lê e a borda da região é {pior_folga} texel"
    );
}
