//! ⭐⭐⭐ **A face ACESA da parede não tem linha escura** (F3 da fila do dono, `docs/Luz2D/01_plano.md`
//! §7.1: «uma linha escura de 1 px junto à face iluminada das paredes, em todo nível»). O interior de
//! um opaco recebia 0 e a bilinear do receptor misturava-o na face. A régua (a sonda D da
//! `medida_luz2d_w4b`): a razão placa ÷ gabarito em faixas de ¼ px a partir da face da parede da
//! cena 1 (`x = −0,95`), média ao longo dela, pelo caminho do PRODUTO em cada nível.

use bevy_ecs::world::World;
use ph2d_app_light2d::{LuzDoQuadro, smoke};
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_render::Camera2d;
use vello::kurbo::Affine;

use crate::comum::fluencia_lida;

const FHD: (u32, u32) = (1920, 1080);
const FACE: f64 = -0.95;

fn afim() -> Affine {
    Camera2d::default().world_to_screen_affine(WindowSize::new(FHD.0, FHD.1))
}

/// O MEDIDO − folga (rodada `medida_luz2d_w5a`): o mínimo, ao longo das faixas de ¼ a 20 px da face,
/// da razão média placa ÷ gabarito — Low 0,983 · Medium e High 0,992 (a 1,25 px) com o preenchimento;
/// sem ele 0,446 · 0,456 · 0,446 a 0,25 px (a linha escura de 1 px).
const PISO_DA_RAZAO: f64 = 0.97;

/// ⭐ A régua: o gabarito nas faixas de ¼ a 20 px da face, uma vez.
pub(crate) struct Regua {
    ys: Vec<f64>,
    faixas: Vec<f64>,
    gab: Vec<Vec<f64>>,
    ppm: f64,
}

impl Regua {
    pub(crate) fn nova(cena: &ph2d_light2d_ecs::CenaDeLuz) -> Self {
        let [a, b, ..] = afim().as_coeffs();
        let ppm = a.hypot(b);
        let ys: Vec<f64> = (0..)
            .map(|k| -0.1 + f64::from(k) / ppm)
            .take_while(|&y| y <= 1.3)
            .collect();
        let faixas: Vec<f64> = (0..80).map(|k| 0.25 + 0.25 * f64::from(k)).collect();
        let gab = faixas
            .iter()
            .map(|d| {
                let x = FACE - d / ppm;
                ys.iter()
                    .map(|&y| ph2d_light2d_ref::fluencia(cena, [x, y])[1])
                    .collect()
            })
            .collect();
        Self {
            ys,
            faixas,
            gab,
            ppm,
        }
    }

    /// `(a pior razão média placa ÷ gabarito, a distância dela à face em px)`.
    pub(crate) fn pior(&self, f: &crate::comum::Fluencia) -> (f64, f64) {
        let (mut pior, mut onde) = (f64::INFINITY, 0.0);
        for (d, g) in self.faixas.iter().zip(&self.gab) {
            let x = FACE - d / self.ppm;
            let m = self
                .ys
                .iter()
                .zip(g)
                .map(|(&y, &gy)| f.amostra([x, y]) / gy.max(1e-9))
                .sum::<f64>()
                / self.ys.len() as f64;
            if m < pior {
                (pior, onde) = (m, *d);
            }
        }
        (pior, onde)
    }
}

/// A cena 1 do smoke.
pub(crate) fn cena_1() -> World {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 1));
    w
}

/// ⭐⭐⭐ **GATE F3 — junto à face acesa, a placa é o gabarito** em todo nível.
#[test]
fn a_face_acesa_da_parede_nao_tem_linha_escura() {
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let w = cena_1();
    let regua = Regua::nova(&cena_de_luz(&w));
    let r = crate::comum::renderer(&gpu);
    for (nome, degrau) in crate::a_escada_da_sombra::degraus() {
        let mut l = LuzDoQuadro::com_degrau(degrau);
        assert!(l.preparar(&w, afim(), FHD));
        l.gravar(&gpu, &mut ph2d_ecs::PresentWorld::new(), &r);
        let (pior, onde) = regua.pior(&fluencia_lida(&gpu, l.solver().expect("solver")));
        eprintln!("F3 · {nome}: a pior faixa {pior:.3} a {onde:.2} px da face");
        assert!(
            pior >= PISO_DA_RAZAO,
            "F3 · {nome}: a face acesa tem linha escura — placa ÷ gabarito {pior:.3} a {onde:.2} px (piso {PISO_DA_RAZAO})"
        );
    }
}
