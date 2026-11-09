//! **Smoke do dono 1 da cena 26 (09/11): «segurar → dá uns pulinhos estranhos e nada mais».** A
//! cena montada pelo roteador, com a animação (a lei `advance_frames_marked` + `segura_regra`), a
//! forma derivada do quadro MOSTRADO (`deriva_formas`) e a ponte do rapier no MESMO laço, como a
//! shell os corre a cada quadro (`fase_frame_open` ⇒ `fase_physics_step`). `docs/PixelArt/P14_plano.md`
//! «Smoke do dono 1 — pulinhos».

use ph2d_ecs::{ChildOf, FrameRef, SpriteAnimations, SpriteAnimator, SpriteGrid, Transform, World};
use ph2d_physics_ecs::{AnimRules, Collider, ColliderShape, PhysicsBridge};
use ph2d_pixel_cena::SegueSlice;
use ph2d_pixel_cenas::heroi as h;

use crate::cena26::{Cena, montada, sprite_de, tique};

/// Um passo da física medido.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Passo {
    pub t_ms: u32,
    pub celula: usize,
    /// A origem do corpo (m).
    pub y: f32,
    pub x: f32,
    /// A velocidade vertical (m/s) pela diferença de posição.
    pub vy: f32,
    /// Meia-altura e offset Y do collider (px).
    pub meia_h_px: f32,
    pub off_y_px: f32,
    /// O canto superior esquerdo da arte desenhada, em px de mundo (Y para baixo).
    pub arte: [f64; 2],
    /// O fundo e o topo da caixa do colisor NO RAPIER (m).
    pub rapier_y: [f32; 2],
}

fn celula(w: &World, e: ph2d_ecs::Entity) -> usize {
    match w.get::<SpriteGrid>(e).expect("grelha").shown() {
        FrameRef::Cell { cell, .. } => cell as usize,
        FrameRef::Empty => usize::MAX,
    }
}

fn forma(w: &mut World, pai: ph2d_ecs::Entity) -> Collider {
    let mut q = w.query::<(&SegueSlice, &ChildOf, &Collider)>();
    q.iter(w)
        .find(|(_, c, _)| c.parent() == pai)
        .map(|(_, _, c)| c.clone())
        .expect("a forma da slice")
}

/// Segura `move_right` de `prime` a `solta` ms; devolve um [`Passo`] por passo da física.
pub(crate) fn corre(c: &mut Cena, prime: u32, solta: u32, ate: u32) -> Vec<Passo> {
    let s = sprite_de(&c.app, c.sim.world(), c.heroi);
    let e = c.heroi;
    let docs: Vec<(u32, ph2d_pixel_core::Sprite)> = {
        let w = c.sim.world_mut();
        let mut q = w.query::<&ph2d_ecs::PixelSprite>();
        let ks: Vec<u32> = q.iter(w).map(|p| p.0).collect();
        ks.into_iter()
            .map(|k| (k, c.app.doc(k).expect("doc").sprite.clone()))
            .collect()
    };
    let q0 = s.frames()[0].id;
    {
        let w = c.sim.world_mut();
        ph2d_pixel_cena::deriva(w, |_| Some((&s, 1)));
        let clips = w.get::<SpriteAnimations>(e).expect("clips").clone();
        let mut a = w.get_mut::<SpriteAnimator>(e).expect("animador");
        a.current = clips.0.first().expect("1.º clip").name.clone();
        a.playing = true;
        let r = clips.0.first().cloned();
        a.rewind(r.as_ref());
    }
    let regra = AnimRules::of_entity(c.sim.world(), e).expect("regras");
    let run = regra.0[0].play.clone();
    let entrada = ph2d_physics_ecs::clip_de_entrada(c.sim.world(), e);
    let mut antes = None;
    let mut ponte = PhysicsBridge::new();
    let mut fita = Vec::new();
    let mut ultimo_tique = 0_u64;
    let mut y_antes = None;
    for t in 0..=ate {
        let w = c.sim.world_mut();
        if t > prime {
            let manda = (t < solta).then_some(0);
            let mut a = w.get_mut::<SpriteAnimator>(e).unwrap();
            antes =
                ph2d_physics_ecs::segura_regra(&regra.0, manda, antes, &mut a, entrada.as_deref());
        }
        if t == prime {
            let mut a = w.get_mut::<SpriteAnimator>(e).unwrap();
            a.current.clone_from(&run);
            a.elapsed_ticks = 0;
            a.pingpong_reverse = false;
            a.repeat_count = 0;
            a.playing = true;
        }
        tique(w, e, if t == 0 { 0 } else { 1000 });
        ph2d_pixel_cena::deriva_formas(
            w,
            |k| {
                docs.iter()
                    .find(|(d, _)| *d == k)
                    .map(|(_, d)| (d, d.frames()[0].id))
            },
            1.0 / h::PPM,
        );
        let alvo = u64::from(t) * 60 / 1000;
        ponte.dispatch(&mut c.sim, true, alvo);
        if alvo != ultimo_tique {
            ultimo_tique = alvo;
            let caixa = ponte
                .body_aabb(e)
                .map_or([f32::NAN; 2], |(a, b)| [a[1], b[1]]);
            let w = c.sim.world_mut();
            let tr = *w.get::<Transform>(e).expect("t");
            let col = forma(w, e);
            let meia_h = match col.shape {
                ColliderShape::Cuboid { half_y, .. } => half_y,
                _ => f32::NAN,
            };
            let cel = celula(w, e);
            let quadro = ph2d_pixel_cena::quadro_mostrado(w, e, &s, q0);
            let (px, py) = ph2d_pixel_cena::pivo_px(&s, quadro);
            let ppm = h::PPM as f32;
            let vy = y_antes.map_or(0.0, |ya: f32| (tr.translation.y - ya) * 60.0);
            y_antes = Some(tr.translation.y);
            fita.push(Passo {
                t_ms: t,
                celula: cel,
                y: tr.translation.y,
                x: tr.translation.x,
                vy,
                meia_h_px: meia_h * ppm,
                off_y_px: col.offset[1] * ppm,
                arte: [
                    f64::from(tr.translation.x) * h::PPM - px,
                    -f64::from(tr.translation.y) * h::PPM - py,
                ],
                rapier_y: caixa,
            });
        }
    }
    fita
}

#[test]
fn a_cena_26_mede_os_pulinhos() {
    let mut c = montada();
    let w = c.sim.world();
    eprintln!(
        "H4: o herói lê o teclado? {}",
        ph2d_physics_ecs::reads_the_keyboard(w, c.heroi)
    );
    let fita = corre(&mut c, 1000, 2600, 2800);
    let topo = h::topo_do_chao();
    eprintln!(
        "t_ms cel   y(px acima do topo)  vy(m/s)  meia_h  off_y   x(m)    arte(x,y px)  rapier fundo/topo (px acima do chão)"
    );
    for p in fita.iter().filter(|p| p.t_ms >= 900) {
        eprintln!(
            "{:5} {:3} {:8.3} {:8.3} {:6.2} {:6.2} {:7.4} ({:6.2},{:6.2}) {:7.3} {:7.3}",
            p.t_ms,
            p.celula,
            (f64::from(p.y) - topo) * h::PPM,
            p.vy,
            p.meia_h_px,
            p.off_y_px,
            p.x,
            p.arte[0],
            p.arte[1],
            (f64::from(p.rapier_y[0]) - topo) * h::PPM,
            (f64::from(p.rapier_y[1]) - topo) * h::PPM
        );
    }
    assert!(
        !ph2d_physics_ecs::reads_the_keyboard(c.sim.world(), c.heroi),
        "nenhum controlador move o herói: a cena ensina a regra de animação"
    );
    let correr: Vec<_> = fita
        .iter()
        .filter(|p| (1000..2600).contains(&p.t_ms))
        .collect();
    let acima = |p: &Passo| (f64::from(p.y) - topo) * h::PPM;
    let pior = correr
        .iter()
        .map(|p| (acima(p) - 1.0).abs())
        .fold(0.0, f64::max);
    let vy = correr.iter().map(|p| p.vy.abs()).fold(0.0, f32::max);
    let artes: std::collections::BTreeSet<_> = correr
        .iter()
        .map(|p| (p.arte[0].to_bits(), p.arte[1].to_bits()))
        .collect();
    eprintln!(
        "segurando →: pior desvio da altura {pior:.3} px · |vy| máx {vy:.3} m/s · {} posições da arte",
        artes.len()
    );
    assert!(pior < 0.05, "o herói não pula a correr: {pior:.3} px");
    assert!(
        vy < 0.01,
        "nenhuma velocidade vertical a correr: {vy:.3} m/s"
    );
    assert_eq!(artes.len(), 1, "o desenho não salta: o pivô é o mesmo");
    // Os quadros do run têm píxeis diferentes (a animação vê-se).
    let s = sprite_de(&c.app, c.sim.world(), c.heroi);
    let px = |i: usize| {
        let mut v = Vec::new();
        ph2d_pixel_core::compose_rgba(&s, s.frames()[i].id, None, &mut v);
        v
    };
    let difere = |a: usize, b: usize| {
        px(a)
            .chunks(4)
            .zip(px(b).chunks(4))
            .filter(|(x, y)| x != y)
            .count()
    };
    let d = [difere(0, 1), difere(2, 3), difere(3, 4), difere(2, 4)];
    eprintln!(
        "píxeis diferentes: idle 0×1 {} · run 2×3 {} · 3×4 {} · 2×4 {}",
        d[0], d[1], d[2], d[3]
    );
    assert!(
        d.iter().all(|&n| n > 0),
        "cada quadro mostra-se diferente: {d:?}"
    );
    let celulas: std::collections::BTreeSet<_> = correr.iter().map(|p| p.celula).collect();
    assert_eq!(
        celulas.into_iter().collect::<Vec<_>>(),
        vec![2, 3, 4],
        "o run toca"
    );
}
