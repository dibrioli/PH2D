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

fn medir(gpu: &ph2d_gpu::GpuContext, opacidades: &[f32]) -> Vec<Leitura> {
    varrer(gpu, opacidades).iter().map(ler).collect()
}

/// A fluência a cada opacidade, na ORDEM dada, com um renderizador e uma luz só: montar o solver
/// (os shaders) custa ~14 s em `dev` e os 40 quadros ~0,5 s — a 2.ª opacidade parte da 1.ª e os 40
/// quadros reassentam o rebote.
fn varrer(gpu: &ph2d_gpu::GpuContext, opacidades: &[f32]) -> Vec<Fluencia> {
    let mut r = crate::comum::renderer(gpu);
    let disco: Vec<u8> = disc_tile_pixels(DISC_TILE_PX)
        .into_iter()
        .flat_map(|p| p.0)
        .collect();
    let disco = r
        .acquire_individual(DISC_TILE_PX, DISC_TILE_PX, &disco)
        .expect("textura do disco");
    let afim = ph2d_render::Camera2d::default()
        .world_to_screen_affine(ph2d_host::WindowSize::new(1920, 1080));
    let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
    let mut saida = Vec::new();
    for &opacidade in opacidades {
        let (w, mut present) = cena(opacidade, |s: &Sprite, t| {
            let mut i = crate::comum::quad(disco, t);
            i.size = s.size;
            i.anchor = s.anchor;
            i.tint = s.tint;
            i
        });
        for _ in 0..40 {
            assert!(l.preparar(&w, afim, (1920, 1080)));
            l.gravar(gpu, &mut present, &r);
        }
        saida.push(crate::comum::fluencia_lida(gpu, l.solver().expect("solver")));
    }
    saida
}

/// A cena 7 com o laranja junto à sombra do azul e o azul a `opacidade`, e o mundo da apresentação
/// (cada disco que bloqueia, pelo `quad` com a base do `GlobalTransform`).
fn cena(
    opacidade: f32,
    quad: impl Fn(&Sprite, [f32; 2]) -> ph2d_render::RenderInstance,
) -> (World, PresentWorld) {
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
    let mut present = PresentWorld::new();
    let mut q = w.query_filtered::<(bevy_ecs::entity::Entity, &Sprite, &GlobalTransform), bevy_ecs::query::With<BloqueiaLuz>>();
    for (e, s, g) in q.iter(&w) {
        let [ax, ay, bx, by, tx, ty] = g.affine();
        let mut i = quad(s, [tx, ty]);
        i.basis = [ax, ay, bx, by];
        present.world_mut().spawn((i, SimRef(e)));
    }
    (w, present)
}

fn ler(f: &Fluencia) -> Leitura {
    let n = AZUL[0].hypot(AZUL[1]);
    let u = [-AZUL[0] / n, -AZUL[1] / n];
    let face = [AZUL[0] + 0.6 * R_AZUL * u[0], AZUL[1] + 0.6 * R_AZUL * u[1]];
    // Na sombra do azul, do lado do laranja.
    let sombra = [-2.2, -3.0];
    let ponto = |p: [f64; 2]| [0, 1, 2].map(|c| f.amostra_canal(p, c));
    [
        ponto(face),
        anel(f, AZUL, R_AZUL),
        anel(f, LARANJA, R_LARANJA),
        ponto(sombra),
    ]
}

/// ⭐ **A luz é CONTÍNUA na opacidade** (a lei da W5k: mostrado = `mix(transmitida, exterior, α)`,
/// albedo `α·a`): a face acesa do azul, o anel dele e a sombra a `α` ficam a `tol` de `α = 1`, canal a
/// canal. Antes da W5k, a 0,999 a face caía a 0,0003 de 2,70 (o corpo PRETO) e o rebote do azul sumia.
#[test]
fn a_luz_e_continua_na_opacidade_do_oclusor() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    let [base, l999, l99] = medir(&gpu, &[1.0, 0.999, 0.99]).try_into().expect("três");
    for (a, tol, l) in [(0.999_f32, 0.01, l999), (0.99, 0.02, l99)] {
        for (k, nome) in [(0, "face acesa do azul"), (1, "anel do azul"), (3, "sombra do azul")] {
            for c in 0..3 {
                let (b, v) = (base[k][c], l[k][c]);
                assert!(
                    (v - b).abs() <= tol * b.abs(),
                    "{nome}, canal {c}: {v:.4} a α {a} contra {b:.4} a α 1 (tolerância {tol})"
                );
            }
        }
    }
}

/// Instrumento (`--ignored`): grava a fluência a `α = 1` em `$PH2D_ALFA1_SAIDA` — a prova de que a
/// lei nova é a de antes BIT A BIT no opaco (`cmp` entre a corrida da base e a da W5k).
#[test]
#[ignore = "instrumento: grava um ficheiro"]
fn sonda_grava_a_fluencia_a_alfa_1() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    let Ok(saida) = std::env::var("PH2D_ALFA1_SAIDA") else {
        return;
    };
    let f = varrer(&gpu, &[1.0]).remove(0);
    let bytes: Vec<u8> = f.v.iter().flatten().flat_map(|x| x.to_le_bytes()).collect();
    std::fs::write(saida, bytes).expect("gravar");
}

/// Sonda (`--ignored --nocapture`): a varredura da opacidade.
#[test]
#[ignore = "sonda: imprime"]
fn sonda_a_opacidade_do_oclusor() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    let rgb = |v: [f64; 3]| format!("{:.4} {:.4} {:.4}", v[0], v[1], v[2]);
    let alfas = [1.0_f32, 0.999, 0.99, 0.9, 0.5];
    for (a, [face, anel_azul, anel_laranja, sombra]) in alfas.into_iter().zip(medir(&gpu, &alfas)) {
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
