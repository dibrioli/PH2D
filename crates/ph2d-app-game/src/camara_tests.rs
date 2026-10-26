//! Os gates do J4 (`docs/GameEngineFuncional/11_camara_e_parallax_plano.md`): a câmara do nível 2 não mostra
//! fora dos limites, e o fundo anda a fracção autorada dela sem deixar buraco.

use super::*;
use ph2d_app_components::camera_2d;
use ph2d_preview_drive::PreviewDrive;

const DT: f64 = 1.0 / 60.0;
/// As janelas medidas: 16:9, 4:3 e 21:9, em pixéis.
const JANELAS: [[f32; 2]; 3] = [[1600.0, 900.0], [1200.0, 900.0], [2100.0, 900.0]];

fn por_nome(sim: &mut SimWorld, nome: &str) -> Entity {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w)
        .find(|(_, n)| n.as_str() == nome)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("sem {nome}"))
}

fn x_y(sim: &SimWorld, e: Entity) -> [f32; 2] {
    let t = sim.world().get::<Transform>(e).expect("pose").translation;
    [t.x, t.y]
}

/// O percurso do jogador: o nível todo ao rés do chão, a plataforma alta, e um ponto acima de tudo (o tecto
/// dos limites), ida e volta.
fn percurso() -> Vec<[f32; 2]> {
    let mut p = Vec::new();
    for y in [1.0, 2.5, 12.0] {
        for i in 0..=120 {
            p.push([-14.0 + 33.0 * i as f32 / 120.0, y]);
        }
        for i in (0..=120).rev() {
            p.push([-14.0 + 33.0 * i as f32 / 120.0, y]);
        }
    }
    p.push([3.5, -0.9]);
    p
}

/// Leva o jogador a cada ponto e a câmara atrás dele; devolve, por janela, a pior fuga (m) para fora dos limites
/// de um pixel da janela (o canto de cada pixel das bordas).
fn pior_fuga(com_limites: bool) -> f32 {
    let mut pior = f32::MIN;
    for janela in JANELAS {
        let mut sim = SimWorld::new();
        let jogador = montar(&mut sim, &crate::registo_de_teste(), 2);
        let camara = por_nome(&mut sim, "Camera");
        let l = sim
            .world()
            .get::<CameraLimits>(camara)
            .cloned()
            .expect("limites");
        if !com_limites {
            sim.world_mut().entity_mut(camara).remove::<CameraLimits>();
        }
        let aspecto = janela[0] / janela[1];
        for p in percurso() {
            sim.world_mut()
                .get_mut::<Transform>(jogador)
                .expect("pose")
                .translation = Vec2::new(p[0], p[1]);
            let (v, _) = camera_2d::update(&mut sim, aspecto, 30, DT, [0.0, 0.0]);
            let v = v.expect("câmara activa");
            let s = v.height_world / janela[1];
            // Os quatro cantos da janela, em mundo (y do ecrã para baixo).
            for (px, py) in [
                (0.0, 0.0),
                (janela[0], 0.0),
                (0.0, janela[1]),
                (janela[0], janela[1]),
            ] {
                let x = v.center[0] + (px - janela[0] * 0.5) * s;
                let y = v.center[1] - (py - janela[1] * 0.5) * s;
                let fuga = [l.min[0] - x, x - l.max[0], l.min[1] - y, y - l.max[1]]
                    .into_iter()
                    .fold(f32::MIN, f32::max);
                // Meio pixel de tolerância.
                pior = pior.max(fuga - 0.5 * s);
            }
        }
    }
    pior
}

/// ⭐ **A câmara não mostra fora dos limites:** em 16:9, 4:3 e 21:9, com o jogador em cada ponta do nível,
/// na plataforma alta, acima de tudo e no fundo do buraco, nenhum pixel da janela cai fora de `CameraLimits`.
/// CONTROLO na mesma régua: sem os limites, o mesmo percurso mostra fora.
#[test]
fn a_camara_do_nivel_2_nao_mostra_fora_dos_limites() {
    let com = pior_fuga(true);
    assert!(com <= 0.0, "um pixel fora dos limites: {com:.4} m");
    let sem = pior_fuga(false);
    assert!(
        sem > 1.0,
        "o controlo não fugiu ({sem:.3} m): a régua não mede"
    );
}

/// ⭐ **O fundo anda a fracção autorada da câmara, e cobre a vista:** com a câmara a seguir o jogador pelo nível
/// inteiro, entre dois quadros cada camada anda `(1 − k)·Δc` (a menos de um inteiro de ladrilhos, a repetição),
/// o céu anda com a câmara, o chão não anda; e em toda a faixa da câmara, numa janela 21:9, as nuvens e os
/// montes cobrem a largura da vista e os montes descem abaixo da borda de baixo dela.
#[test]
fn o_fundo_anda_a_fraccao_autorada_e_cobre_a_vista() {
    let mut sim = SimWorld::new();
    let jogador = montar(&mut sim, &crate::registo_de_teste(), 2);
    let camadas: Vec<(Entity, f32, Option<f32>, [f32; 2])> = ["Sky", "Clouds", "Hills"]
        .into_iter()
        .map(|n| {
            let e = por_nome(&mut sim, n);
            let w = sim.world();
            let k = w.get::<ScrollFactor>(e).expect(n).k;
            assert_eq!(k[0], k[1], "{n}");
            let tile = w.get::<ph2d_ecs::ScrollRepeat>(e).map(|r| r.tile[0]);
            let tamanho = w.get::<Sprite>(e).expect(n).size;
            let escala = w.get::<Transform>(e).expect(n).scale;
            (
                e,
                k[0],
                tile,
                [tamanho[0] * escala.x, tamanho[1] * escala.y],
            )
        })
        .collect();
    let chao = por_nome(&mut sim, "Ground Left");
    let chao_antes = x_y(&sim, chao);
    let mut drive = PreviewDrive::default();
    let aspecto = 21.0 / 9.0;
    let mut antes: Option<([f32; 2], Vec<[f32; 2]>)> = None;
    let mut passos = 0;
    for p in percurso() {
        sim.world_mut()
            .get_mut::<Transform>(jogador)
            .expect("pose")
            .translation = Vec2::new(p[0], p[1]);
        let (v, _) = camera_2d::update(&mut sim, aspecto, 2, DT, [0.0, 0.0]);
        let v = v.expect("câmara activa");
        let meia = ph2d_ecs::camera_2d::half_extent(v.height_world, aspecto);
        ph2d_app_components::parallax_bridge::drive_parallax(
            &mut sim,
            Some((v.center, meia)),
            0.0,
            &mut drive,
        );
        let agora: Vec<[f32; 2]> = camadas.iter().map(|c| x_y(&sim, c.0)).collect();
        for ((_, k, tile, tam), at) in camadas.iter().zip(&agora) {
            assert!(
                (at[0] - v.center[0]).abs() + meia[0] <= tam[0] * 0.5,
                "buraco ao lado: camada k={k} em {at:?}, vista {:?}",
                v.center
            );
            assert!(
                tile.is_some() || (at[1] - v.center[1]).abs() + meia[1] <= tam[1] * 0.5,
                "o céu não cobre a vista"
            );
            assert!(
                at[1] - tam[1] * 0.5 <= v.center[1] - meia[1] || *k < CAMADA_MONTES,
                "os montes não chegam à borda de baixo da vista"
            );
        }
        if let Some((c0, ref a0)) = antes {
            let dc = [v.center[0] - c0[0], v.center[1] - c0[1]];
            for (((_, k, tile, _), at), a) in camadas.iter().zip(&agora).zip(a0) {
                for i in 0..2 {
                    let mut resto = (at[i] - a[i]) - (1.0 - k) * dc[i];
                    if let (0, Some(t)) = (i, tile) {
                        resto -= (resto / t).round() * t;
                    }
                    assert!(resto.abs() < 1e-3, "camada k={k} eixo {i}: sobra {resto}");
                }
            }
            if dc != [0.0, 0.0] {
                passos += 1;
            }
        }
        antes = Some((v.center, agora));
    }
    assert!(passos > 200, "a câmara mal andou ({passos} passos)");
    assert_eq!(x_y(&sim, chao), chao_antes, "o chão é o mundo: não anda");
}

/// Os montes são a única camada que tem de tapar a borda de baixo da vista (o céu e as nuvens ficam acima).
const CAMADA_MONTES: f32 = crate::object_add::CAMADA_K;

/// A Background Layer do menu: os montes, a fracção e a repetição do ladrilho, atrás do mundo.
#[test]
fn a_camada_de_fundo_do_menu_nasce_pronta() {
    let mut sim = SimWorld::new();
    let reg = crate::registo_de_teste();
    let bits = crate::object_add::add(
        crate::object_add::BACKGROUND_LAYER,
        &mut sim,
        &reg,
        ph2d_app_physics::physics_seed::COMPONENT_SEEDS,
    )
    .expect("da família")
    .expect("nasce");
    let e = Entity::from_bits(bits);
    let w = sim.world();
    let lado = crate::object_add::ladrilho_m(HILLS_TILE);
    assert_eq!(lado, [8.0, 6.0]);
    assert_eq!(
        w.get::<ScrollFactor>(e).map(|f| f.k),
        Some([CAMADA_MONTES; 2])
    );
    assert_eq!(
        w.get::<ph2d_ecs::ScrollRepeat>(e).map(|r| r.tile),
        Some([lado[0], 0.0])
    );
    let s = w.get::<Sprite>(e).expect("sprite");
    assert_eq!(
        s.source,
        ph2d_render::SpriteSource::Atlas {
            key: HILLS_TILE.key
        }
    );
    assert_eq!(
        s.size,
        [lado[0] * crate::object_add::CAMADA_LADRILHOS, lado[1]]
    );
    assert_eq!(
        w.get::<UvTransform>(e).map(|u| u.scale),
        Some([crate::object_add::CAMADA_LADRILHOS, 1.0])
    );
    assert_eq!(
        w.get::<SpriteGrid>(e).map(|g| g.frame),
        Some(HILLS_TILE.middle_frame())
    );
    assert_eq!(w.get::<ZIndexOverride>(e).map(|z| z.0), Some(-1));
    assert!(
        crate::object_add::BACKGROUND_LAYER
            .tip
            .is_some_and(|t| !t.picture.is_empty()),
        "a dica tem desenho"
    );
}
