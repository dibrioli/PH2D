//! ⭐⭐⭐ **A luz num ponto do mundo não muda com a VISTA** (smoke do dono, 10/10: *«ao dar zoom e pan o
//! aspecto da iluminação muda»* — duas fotos da cena 4 com a mesma área à direita da parede pequena,
//! preta numa vista e cinza na outra). A grade de luz cobre a vista e uma margem; a lâmpada DIRETA
//! ilumina de qualquer distância, mas um opaco fora da grade não a tapa.
//!
//! A régua: pontos fixos do mundo na sala da lâmpada pequena, lidos pelo caminho do PRODUTO em várias
//! vistas (pan e zoom), contra o gabarito.

use bevy_ecs::world::World;
use ph2d_app_light2d::{LuzDoQuadro, smoke};
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_render::Camera2d;

use crate::comum::fluencia_lida;

const FHD: (u32, u32) = (1920, 1080);

/// A cena 4 sem a ambiente (só as lâmpadas contam).
fn cena() -> World {
    crate::a_escada_da_sombra::cena4_com_e_sem().0
}

/// As vistas: a de fábrica e as que aproximam a sala pequena (o muro do meio e a lâmpada grande
/// saem da tela, e da grade).
const VISTAS: [([f32; 2], f32); 4] = [
    ([0.0, 0.0], 10.0),
    ([-3.0, -1.5], 4.0),
    ([-3.5, -1.5], 2.5),
    ([-2.6, -2.0], 1.5),
];

/// Pontos à direita da parede pequena, na sombra da lâmpada pequena (só a grande, do outro lado do
/// muro, poderia acendê-los — e o muro tapa-a).
const PONTOS: [[f64; 2]; 3] = [[-2.8, -1.0], [-2.5, -1.8], [-2.2, -2.3]];

/// O teto (provisório, a medir com a cura): o pior erro de um ponto, do máximo da cena.
const TETO: f64 = 0.02;

/// ⚠️ VERMELHO visto (10/10, W5a): a vista de 1,5 m dá 0 · 2,45 · 2,38 nos três pontos (o gabarito 0 · 0
/// · 0; 0,176 do máximo) — o muro do meio cruza o raio FORA da grade. É o F4 da fila
/// (`docs/Luz2D/01_plano.md` §7.1): a W5 tira o `ignore` com a cura.
#[test]
#[ignore = "F4 da fila do dono: vermelho até a cura (W5)"]
fn a_luz_num_ponto_nao_muda_com_a_vista() {
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let w = cena();
    let c = cena_de_luz(&w);
    let r = crate::comum::renderer(&gpu);
    let gab: Vec<f64> = PONTOS
        .iter()
        .map(|&p| ph2d_light2d_ref::fluencia(&c, p)[1])
        .collect();
    let max = ph2d_light2d_ref::fluencia(&c, [-4.0, 1.0])[1];
    let mut pior: f64 = 0.0;
    for (centro, altura) in VISTAS {
        let afim =
            Camera2d::new(centro, altura).world_to_screen_affine(WindowSize::new(FHD.0, FHD.1));
        let mut l = LuzDoQuadro::com_degrau(ph2d_app_light2d::DEGRAU_PADRAO);
        assert!(l.preparar(&w, afim, FHD));
        l.gravar(&gpu, &mut ph2d_ecs::PresentWorld::new(), &r);
        let g = l.grade().expect("grade");
        let f = fluencia_lida(&gpu, l.solver().expect("solver"));
        let placa: Vec<f64> = PONTOS.iter().map(|&p| f.amostra(p)).collect();
        let fora_do_muro = g.origem_m[0] + f64::from(g.largura) * g.texel_m < -0.1;
        eprintln!(
            "vista {centro:?} × {altura} m · grade x ∈ [{:.2}, {:.2}] (o muro {}) · placa {placa:.4?} · gabarito {gab:.4?}",
            g.origem_m[0],
            g.origem_m[0] + f64::from(g.largura) * g.texel_m,
            if fora_do_muro { "FORA" } else { "dentro" }
        );
        for (p, q) in placa.iter().zip(&gab) {
            pior = pior.max((p - q).abs() / max);
        }
    }
    eprintln!("pior erro {pior:.4} do máximo (teto {TETO})");
    assert!(
        pior <= TETO,
        "a luz muda com a vista: {pior:.4} do máximo (teto {TETO})"
    );
}
