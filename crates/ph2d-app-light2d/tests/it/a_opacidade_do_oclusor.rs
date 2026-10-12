//! ⭐ **A opacidade do «Light Occluder» é CONTÍNUA na luz** (smoke do dono, 12/10: «ao reduzir o Opacity
//! do objeto azul, o laranja da sombra desapareceu — nem sempre»). A cena 7 com as duas IMAGENS juntas
//! (o disco laranja logo abaixo e à direita do azul, na sombra dele): a opacidade do azul varre
//! 1 → 0,999 → 0,99 → 0,9 → 0,5 e lê-se, por canal, a face acesa do azul, o anel do rebote de cada um e
//! um ponto da sombra do azul onde chega o rebote do laranja.

use bevy_ecs::world::World;
use ph2d_app_light2d::{DEGRAU_PADRAO, LuzDoQuadro, smoke};
use ph2d_ecs::{GlobalTransform, Name, PresentWorld, SimRef, Transform};
use ph2d_light2d_ecs::BloqueiaLuz;
use ph2d_render::{DISC_TILE_PX, Sprite, disc_tile_pixels};

use crate::comum::Fluencia;

const AZUL: [f64; 2] = [-2.6, -1.6];
const R_AZUL: f64 = 0.9;
/// O laranja junto à sombra do azul (a foto do dono).
const LARANJA: [f64; 2] = [-1.4, -2.9];
const R_LARANJA: f64 = 0.5;

/// As leituras por canal: `[face acesa do azul, anel do azul, anel do laranja, sombra do azul]`.
type Leitura = [[f64; 3]; 4];

fn anel(f: &Fluencia, c: [f64; 2], r: f64) -> [f64; 3] {
    let mut s = [0.0; 3];
    for k in 0..360 {
        let (sn, cs) = (std::f64::consts::TAU * f64::from(k) / 360.0).sin_cos();
        for (ch, v) in s.iter_mut().enumerate() {
            *v += f.amostra_canal([c[0] + 1.25 * r * cs, c[1] + 1.25 * r * sn], ch) / 360.0;
        }
    }
    s
}

fn medir(gpu: &ph2d_gpu::GpuContext, opacidade: f32) -> Leitura {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 7));
    let mut por_nome = |nome: &str| {
        let mut q = w.query::<(bevy_ecs::entity::Entity, &Name)>();
        q.iter(&w)
            .find(|(_, n)| n.0 == nome)
            .map(|(e, _)| e)
            .expect(nome)
    };
    let (azul, laranja) = (
        por_nome("Disco grande (imagem)"),
        por_nome("Disco pequeno (imagem)"),
    );
    #[expect(clippy::cast_possible_truncation, reason = "o mundo é f32")]
    let pose = |c: [f64; 2]| {
        let mut t = Transform::IDENTITY;
        t.translation.x = c[0] as f32;
        t.translation.y = c[1] as f32;
        (t, GlobalTransform::from_transform(t))
    };
    w.entity_mut(laranja).insert(pose(LARANJA));
    w.entity_mut(azul).insert(BloqueiaLuz { opacidade });
    let mut r = crate::comum::renderer(gpu);
    let disco: Vec<u8> = disc_tile_pixels(DISC_TILE_PX)
        .into_iter()
        .flat_map(|p| p.0)
        .collect();
    let disco = r
        .acquire_individual(DISC_TILE_PX, DISC_TILE_PX, &disco)
        .expect("textura do disco");
    let mut present = PresentWorld::new();
    let mut q = w.query_filtered::<(bevy_ecs::entity::Entity, &Sprite, &GlobalTransform), bevy_ecs::query::With<BloqueiaLuz>>();
    for (e, s, g) in q.iter(&w) {
        let [ax, ay, bx, by, tx, ty] = g.affine();
        let mut i = crate::comum::quad(disco, [tx, ty]);
        i.size = s.size;
        i.anchor = s.anchor;
        i.basis = [ax, ay, bx, by];
        i.tint = s.tint;
        present.world_mut().spawn((i, SimRef(e)));
    }
    let afim = ph2d_render::Camera2d::default()
        .world_to_screen_affine(ph2d_host::WindowSize::new(1920, 1080));
    let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
    for _ in 0..40 {
        assert!(l.preparar(&w, afim, (1920, 1080)));
        l.gravar(gpu, &mut present, &r);
    }
    let f = crate::comum::fluencia_lida(gpu, l.solver().expect("solver"));
    let n = AZUL[0].hypot(AZUL[1]);
    let u = [-AZUL[0] / n, -AZUL[1] / n];
    let face = [AZUL[0] + 0.6 * R_AZUL * u[0], AZUL[1] + 0.6 * R_AZUL * u[1]];
    // Na sombra do azul, do lado do laranja.
    let sombra = [-2.2, -3.0];
    let ponto = |p: [f64; 2]| [0, 1, 2].map(|c| f.amostra_canal(p, c));
    [
        ponto(face),
        anel(&f, AZUL, R_AZUL),
        anel(&f, LARANJA, R_LARANJA),
        ponto(sombra),
    ]
}

/// Sonda (`--ignored --nocapture`): a varredura da opacidade.
#[test]
#[ignore = "sonda: imprime"]
fn sonda_a_opacidade_do_oclusor() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    let rgb = |v: [f64; 3]| format!("{:.4} {:.4} {:.4}", v[0], v[1], v[2]);
    for a in [1.0_f32, 0.999, 0.99, 0.9, 0.5] {
        let [face, anel_azul, anel_laranja, sombra] = medir(&gpu, a);
        eprintln!(
            "azul a {a}: face {} · anel azul {} · anel laranja {} · sombra {} (R/B {:.3})",
            rgb(face),
            rgb(anel_azul),
            rgb(anel_laranja),
            rgb(sombra),
            sombra[0] / sombra[2]
        );
    }
}
