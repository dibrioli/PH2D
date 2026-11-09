//! ⭐ P14 — **a costura inteira do Normal Map, na placa**: o pedido do cartão (`ImageUseEdit::SetNormalMap`,
//! pela porta `normal_map::aplica_edicao`) ⇒ `normal_map::sincronizar` com o `SpriteRenderer` REAL ⇒ o quadro com
//! o solver real e uma lâmpada lateral. A cadeia do quadro é a da família da luz (`comum_luz`, o arnês de
//! `ph2d-app-light2d`), o molde é `o_sprite_com_normal_acende_com_o_solver_real` — lá a normal entra direto
//! no renderer; aqui só pela verdade da imagem.

use crate::comum_luz::{Cadeia, H, W, afim, gpu, quad};
use ph2d_app_light2d::LuzDoQuadro;
use ph2d_app_sprite_anim::normal_map as nm;
use ph2d_asset::AssetDb;
use ph2d_ecs::{GlobalTransform, PresentWorld, SpritePixels, Transform, World};
use ph2d_light2d_ecs::Luz;
use ph2d_render::Sprite;
use std::collections::BTreeMap;

/// O lado da textura (1 m ⇒ 16 texels a 16 px/m).
const LADO: u32 = 16;

/// O hemisfério (y para cima), o mesmo do molde.
fn hemisferio() -> Vec<u8> {
    let c = f64::from(LADO) / 2.0;
    let mut d = Vec::new();
    for y in 0..LADO {
        for x in 0..LADO {
            let (mut nx, mut ny) = ((f64::from(x) + 0.5 - c) / c, -(f64::from(y) + 0.5 - c) / c);
            let r = nx.hypot(ny);
            if r > 1.0 {
                (nx, ny) = (nx / r, ny / r);
            }
            let nz = (1.0 - nx * nx - ny * ny).max(0.0).sqrt();
            for v in [nx, ny, nz] {
                d.push(((v * 0.5 + 0.5) * 255.0).round() as u8);
            }
            d.push(255);
        }
    }
    d
}

/// ⭐ **Escolher a normal no cartão acende o sprite com o solver real**: claro/escuro (luma a ±0,3 m) > 1,8 com a
/// normal escolhida; sem escolha, o MESMO quadro dá só a queda com a distância (~1,2) e 0 B de `M`.
///
/// **Mutação que sangra:** o quadro sem o `sincronizar` (a placa nunca recebe a normal da verdade).
#[test]
fn escolher_a_normal_no_cartao_acende_com_o_solver_real() {
    let Some(gpu) = gpu() else {
        eprintln!("costura do normal map: sem placa — saltado");
        return;
    };
    let mut luzes = World::new();
    let mut t = Transform::IDENTITY;
    t.translation.x = -1.5;
    luzes.spawn((Luz::default(), t, GlobalTransform::from_transform(t)));
    let px = |p: [f64; 2]| {
        (
            (f64::from(W) / 2.0 + p[0] * 16.0) as u32,
            (f64::from(H) / 2.0 - p[1] * 16.0) as u32,
        )
    };
    let luma = |b: &[u8], (x, y): (u32, u32)| {
        let i = ((y * W + x) * 4) as usize;
        (f64::from(b[i]) + f64::from(b[i + 1]) + f64::from(b[i + 2])) / 3.0
    };
    let medir = |escolhe: bool| {
        nm::set_normais(&[]);
        let mut c = Cadeia::nova(&gpu);
        let db = AssetDb::new();
        let cor_px: Vec<u8> = (0..LADO * LADO)
            .flat_map(|_| [160, 160, 160, 255])
            .collect();
        let cor = db.insert_image_rgba8(LADO, LADO, cor_px.clone());
        let normal = db.insert_image_rgba8(LADO, LADO, hemisferio());
        let tex = c
            .renderer
            .acquire_individual(LADO, LADO, &cor_px)
            .expect("cor");
        let atlas = BTreeMap::new();
        let mut sim = World::new();
        let bits = sim
            .spawn((
                Sprite::individual(tex, [1.0; 2], [1.0; 4]),
                SpritePixels(cor),
            ))
            .id()
            .to_bits();
        if escolhe {
            let pedido = ph2d_editor_core::ImageUseEdit::SetNormalMap(Some(*normal.as_bytes()));
            assert_eq!(
                nm::aplica_edicao(&sim, &db, &atlas, bits, pedido),
                Some(Ok(false))
            );
        }
        let mut present = PresentWorld::new();
        present.world_mut().spawn(quad(tex, [0.0, 0.0]));
        let mut l = LuzDoQuadro::com_degrau(ph2d_app_light2d::DEGRAU_PADRAO);
        for _ in 0..30 {
            assert!(nm::sincronizar(&mut sim, &db, &atlas, &mut c.renderer).is_empty());
            l.com_sprites_de_normal(&c.renderer.caixas_com_normal(&mut present));
            assert!(l.preparar(&luzes, afim(), (W, H)));
            l.recolher(&luzes);
            l.gravar(&gpu, &mut present, &c.renderer);
        }
        let m = l.solver().map_or(0, |s| s.bytes_do_momento());
        l.ligar(&gpu, &mut c.renderer, &mut c.colagem, &c.tonemap);
        let b = c.quadro(
            &gpu,
            &mut present,
            &ph2d_render::LiftedInstances::default(),
            &vello::Scene::new(),
        );
        nm::set_normais(&[]);
        let _ = nm::sincronizar(&mut sim, &db, &atlas, &mut c.renderer);
        (luma(&b, px([-0.3, 0.0])), luma(&b, px([0.3, 0.0])), m)
    };
    let (c0, e0, m0) = medir(false);
    let (c1, e1, m1) = medir(true);
    let (r0, r1) = (c0 / e0.max(1.0), c1 / e1.max(1.0));
    eprintln!(
        "costura: sem escolha {c0:.1}/{e0:.1} = {r0:.3} (M {m0} B) · escolhida no cartão {c1:.1}/{e1:.1} = {r1:.3} (M {m1} B)"
    );
    assert_eq!(m0, 0, "sem normal escolhida o momento não é alocado");
    assert!(m1 > 0, "a normal escolhida liga o momento");
    assert!(
        r1 > 1.8,
        "a normal escolhida no cartão tem de sombrear: {r1:.3} (sem ela {r0:.3})"
    );
}
