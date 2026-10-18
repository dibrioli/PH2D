//! ⭐ **G7 do rebote — o desenho DEVOLVE a luz como a imagem da mesma cor** (W5e; o ABERTO 8 da W5d:
//! a cena 6 media a paridade da luz RECEBIDA, não a do rebote que o desenho devolve). Na cena 6, a luz
//! num anel logo FORA de cada forma desenhada (onde chega o rebote dela, com a cor dela), por canal,
//! contra o mesmo anel no espelho da imagem gêmea.
//!
//! Controlo na MESMA corrida: o desenho com «não recebe luz» (não reflete) — o anel do desenho perde o
//! rebote e afasta-se da imagem.

use bevy_ecs::world::World;
use ph2d_app_light2d::{DEGRAU_PADRAO, LuzDoQuadro, smoke};
use ph2d_ecs::{GlobalTransform, Name, PresentWorld, SimRef, Transform, VecPathRef};
use ph2d_light2d_ecs::{BloqueiaLuz, NaoRecebeLuz};
use ph2d_render::{DISC_TILE_PX, Sprite, disc_tile_pixels};
use ph2d_vec_scene::{VecScene, VecXforms, Xform};

use crate::comum::Fluencia;

/// O anel lido, em múltiplos dos semi-eixos (logo fora da borda).
const ANEL: f64 = 1.25;

/// A luz do anel `ANEL` em volta de `centro` (semi-eixos `semi`), por canal: `[desenho, imagem]`.
fn anel(f: &Fluencia, centro: [f64; 2], semi: [f64; 2]) -> [[f64; 3]; 2] {
    let mut soma = [[0.0; 3]; 2];
    for k in 0..360 {
        let t = std::f64::consts::TAU * f64::from(k) / 360.0;
        let (sn, cs) = t.sin_cos();
        let p = [
            centro[0] + ANEL * semi[0] * cs,
            centro[1] + ANEL * semi[1] * sn,
        ];
        let [d, im] = &mut soma;
        for (c, (sd, si)) in d.iter_mut().zip(im.iter_mut()).enumerate() {
            *sd += f.amostra_canal(p, c);
            *si += f.amostra_canal([p[0], -p[1]], c);
        }
    }
    soma
}

fn medir(gpu: &ph2d_gpu::GpuContext, reflete: bool) -> Vec<[[f64; 3]; 2]> {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 6));
    let mut doc = VecScene::new();
    let semeadas = smoke::semear_documento(&mut doc, 6);
    let mut xforms = VecXforms::new();
    for (id, _) in &semeadas {
        let e = w
            .spawn((
                Transform::IDENTITY,
                Name::new(format!("Path {id}")),
                VecPathRef(*id),
            ))
            .id();
        if !reflete {
            w.entity_mut(e).insert(NaoRecebeLuz);
        }
        xforms.insert(*id, Xform([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]));
    }
    assert!(smoke::marcar_semeadas(&mut w, &doc, semeadas).is_empty());
    let cena = ph2d_light2d_ecs::coleta::cena_de_luz(&w);
    assert!(cena.vetores.iter().all(|v| v.reflete == reflete));
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
        present.world_mut().spawn((i, SimRef(e)));
    }
    let afim = ph2d_render::Camera2d::default()
        .world_to_screen_affine(ph2d_host::WindowSize::new(1920, 1080));
    let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
    for _ in 0..40 {
        assert!(l.preparar(&w, afim, (1920, 1080)));
        l.desenhar_vetores(gpu, &doc, &xforms);
        l.gravar(gpu, &mut present, &r);
    }
    let f = crate::comum::fluencia_lida(gpu, l.solver().expect("solver"));
    smoke::REDONDAS
        .iter()
        .map(|forma| anel(&f, forma.centro, forma.semi))
        .collect()
}

/// O anel do desenho contra o da imagem, por canal (relativo) — MEDIDO (RTX 5060 Ti, `DEGRAU_PADRAO`, 40
/// quadros): o círculo −0,37 · −0,25 · −0,20 %, a elipse −0,12 · −0,19 · −0,21 %; sem reflexo (o
/// controlo) +1,8 · −6,7 · −8,5 % e −9,0 · −3,1 · −0,9 % — o rebote leva a cor de cada corpo.
const TOL_REBOTE: f64 = 0.01;

#[test]
fn o_desenho_devolve_a_luz_como_a_imagem() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    let (com, sem) = (medir(&gpu, true), medir(&gpu, false));
    let mut falhas = Vec::new();
    for (i, forma) in smoke::REDONDAS.iter().enumerate() {
        let rel = |a: [[f64; 3]; 2]| [0, 1, 2].map(|c| (a[0][c] - a[1][c]) / a[1][c]);
        let (r_com, r_sem) = (rel(com[i]), rel(sem[i]));
        eprintln!(
            "G7 rebote · {}: desenho × imagem RGB {:+.2} {:+.2} {:+.2} % · sem reflexo {:+.2} {:+.2} {:+.2} %",
            forma.nome,
            100.0 * r_com[0],
            100.0 * r_com[1],
            100.0 * r_com[2],
            100.0 * r_sem[0],
            100.0 * r_sem[1],
            100.0 * r_sem[2]
        );
        let pior = |r: [f64; 3]| r.iter().fold(0.0_f64, |m, x| m.max(x.abs()));
        assert!(
            pior(r_sem) > 2.0 * TOL_REBOTE,
            "controlo: sem reflexo o anel do {} devia afastar-se da imagem ({:.4})",
            forma.nome,
            pior(r_sem)
        );
        if pior(r_com) > TOL_REBOTE {
            falhas.push(format!("{}: {:.4}", forma.nome, pior(r_com)));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join(" · "));
}

/// O anel da camada dos OUTROS contra o da imagem — MEDIDO + 10 % (RTX 5060 Ti, 40 quadros), remedido
/// com a borda da W6d (o campo pelas retas próprias; handoff W6d §24):
///
/// | o pior canal | Disco grande | Disco pequeno |
/// |---|---|---|
/// | o produto | −0,26 % | −0,30 % |
/// | a camada lida NÃO premultiplicada (a mutação M12 da W5e) | −0,37 % | −0,31 % |
/// | sem reflexo (o controlo) | −8,6 % | −14,3 % |
///
/// (Na W5e, com a reta por texel: −0,27 % · M12 −0,39 %. A M12 separa-se agora por 0,04 no disco grande.)
const TOL_REBOTE_OUTROS: f64 = 0.0033;

/// ⭐ **G7 do rebote para os OUTROS** (W5e): o traço do Flip e as formas do Motion devolvem a luz pela
/// cor que a família desenha na camada (premultiplicada). Na cena 7, com a família de discos de ÁREA
/// EXATA coloridos, o anel logo fora de cada disco contra o da imagem gêmea, por canal. Controlo: o
/// mesmo objeto com «não recebe luz».
#[test]
fn os_outros_devolvem_a_luz_como_a_imagem() {
    use crate::o_traco_do_flip::DiscosExatos;
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    let medir = |reflete: bool| {
        let mut w = World::new();
        assert!(smoke::encenar(&mut w, 7));
        let e = w
            .spawn((Transform::IDENTITY, BloqueiaLuz { opacidade: 1.0 }))
            .id();
        if !reflete {
            w.entity_mut(e).insert(NaoRecebeLuz);
        }
        let cena = ph2d_light2d_ecs::coleta::cena_de_luz(&w);
        assert_eq!(cena.outros.len(), 1);
        assert_eq!(cena.outros[0].reflete, reflete);
        let mut exatos = DiscosExatos {
            entidade: e.to_bits(),
            discos: smoke::DISCOS_DE_TINTA
                .iter()
                .map(|d| (d.centro, d.raio, d.cor))
                .collect(),
        };
        let mut r = crate::comum::renderer(&gpu);
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
            present.world_mut().spawn((i, SimRef(e)));
        }
        let afim = ph2d_render::Camera2d::default()
            .world_to_screen_affine(ph2d_host::WindowSize::new(1920, 1080));
        let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
        for _ in 0..40 {
            assert!(l.preparar(&w, afim, (1920, 1080)));
            l.gravar_com(&gpu, &mut present, &r, &mut [&mut exatos]);
        }
        let f = crate::comum::fluencia_lida(&gpu, l.solver().expect("solver"));
        smoke::DISCOS_DE_TINTA
            .iter()
            .map(|d| anel(&f, d.centro, [d.raio; 2]))
            .collect::<Vec<_>>()
    };
    let (com, sem) = (medir(true), medir(false));
    let mut falhas = Vec::new();
    for (i, d) in smoke::DISCOS_DE_TINTA.iter().enumerate() {
        let rel = |a: [[f64; 3]; 2]| [0, 1, 2].map(|c| (a[0][c] - a[1][c]) / a[1][c]);
        let (r_com, r_sem) = (rel(com[i]), rel(sem[i]));
        let pior = |r: [f64; 3]| r.iter().fold(0.0_f64, |m, x| m.max(x.abs()));
        eprintln!(
            "G7 rebote dos outros · {}: RGB {:+.2} {:+.2} {:+.2} % · sem reflexo {:+.2} {:+.2} {:+.2} %",
            d.nome,
            100.0 * r_com[0],
            100.0 * r_com[1],
            100.0 * r_com[2],
            100.0 * r_sem[0],
            100.0 * r_sem[1],
            100.0 * r_sem[2]
        );
        assert!(
            pior(r_sem) > 10.0 * TOL_REBOTE_OUTROS,
            "controlo: sem reflexo o anel do {} devia afastar-se da imagem",
            d.nome
        );
        if pior(r_com) > TOL_REBOTE_OUTROS {
            falhas.push(format!("{}: {:.4}", d.nome, pior(r_com)));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join(" · "));
}
