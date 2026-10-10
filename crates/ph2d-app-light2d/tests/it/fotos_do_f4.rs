//! **As fotos do F4** (W5b) — a LUZ que o chão da cena 4 recebe, como o editor a pinta (o chão cinza
//! 0,85 × a luz recebida, o tonemap `x/(1+x)` e o gama 2,2 — uma aproximação para OLHAR, não a régua:
//! a régua é `a_luz_nao_muda_com_a_vista`), na vista de fábrica e na do dono (1,5 m de altura,
//! centrada à direita da parede pequena), ANTES (a luz direta sem o mundo fora da grade, a W5a) ×
//! DEPOIS. O app não abre noutra câmera (o `PH2D_LIGHT2D_SMOKE` usa a de fábrica), logo a foto do
//! pan vem daqui: a fluência do MESMO solver do quadro, lida de volta. À mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     fotos_do_f4 -- --ignored --nocapture
//! ```
//!
//! Saída: `target/fotos_w5b/*.ppm` (o `magick` converte e monta).

use std::io::Write;

use ph2d_app_light2d::{
    DEGRAU_PADRAO, captura_do_quadro, fina_do_quadro, grade_do_quadro, regiao_da_janela,
};
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_gpu::{Config, Solver};
use ph2d_render::Camera2d;

use crate::comum::{fluencia_lida, gpu};

const FHD: (u32, u32) = (1920, 1080);
/// A foto: metade da janela.
const FOTO: (u32, u32) = (960, 540);

fn dentro(v: &[[f32; 2]], p: [f64; 2]) -> bool {
    let mut d = false;
    let mut j = v.len() - 1;
    for i in 0..v.len() {
        let (a, b) = (v[i], v[j]);
        let (ay, by) = (f64::from(a[1]), f64::from(b[1]));
        if (ay > p[1]) != (by > p[1])
            && p[0]
                < (f64::from(b[0]) - f64::from(a[0])) * (p[1] - ay) / (by - ay) + f64::from(a[0])
        {
            d = !d;
        }
        j = i;
    }
    d
}

#[test]
#[ignore = "as fotos do F4: corre-se à mão (cabeçalho)"]
fn fotos_do_f4() {
    let Some(gpu) = gpu() else { return };
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/fotos_w5b");
    std::fs::create_dir_all(&dir).expect("pasta");
    let w = crate::a_escada_da_sombra::cena4_com_e_sem().0;
    let cena = cena_de_luz(&w);
    for (vista, camera) in [
        ("fabrica", Camera2d::default()),
        ("dono", Camera2d::new([-2.6, -2.0], 1.5)),
    ] {
        let afim = camera.world_to_screen_affine(WindowSize::new(FHD.0, FHD.1));
        let inv = afim.inverse();
        for (quando, distantes) in [("antes", false), ("depois", true)] {
            let d = DEGRAU_PADRAO;
            let cfg = Config {
                fina: fina_do_quadro(afim, d),
                captura: captura_do_quadro(afim, d),
                distantes,
                ..Config::padrao_para(&gpu.device, grade_do_quadro(afim, FHD, d).expect("grade"))
            };
            let mut s = Solver::novo(&gpu, cfg).expect("solver");
            let (a, b) = regiao_da_janela(s.grade_da_fluencia(), afim, FHD);
            s.regiao_fina(a, b);
            s.quadro(&gpu, &cena);
            let f = fluencia_lida(&gpu, &s);
            let mut px = Vec::with_capacity((FOTO.0 * FOTO.1 * 3) as usize);
            for j in 0..FOTO.1 {
                for i in 0..FOTO.0 {
                    let p = inv
                        * vello::kurbo::Point::new(f64::from(2 * i) + 1.0, f64::from(2 * j) + 1.0);
                    let p = [p.x, p.y];
                    let opaco = cena.oclusores.iter().any(|o| dentro(&o.vertices, p));
                    let v = if opaco {
                        0.12
                    } else {
                        let l = 0.85 * f.amostra(p) / std::f64::consts::TAU;
                        (l / (1.0 + l)).powf(1.0 / 2.2)
                    };
                    let c = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                    px.extend([c, c, c]);
                }
            }
            let nome = dir.join(format!("cena4_{vista}_{quando}.ppm"));
            let mut ficheiro = std::fs::File::create(&nome).expect("foto");
            write!(ficheiro, "P6\n{} {}\n255\n", FOTO.0, FOTO.1).expect("cabeçalho");
            ficheiro.write_all(&px).expect("píxeis");
            eprintln!("foto: {}", nome.display());
        }
    }
}
