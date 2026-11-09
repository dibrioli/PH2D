//! **P14 — o Sprite do motor com normal map liga o momento `M`** (`docs/PixelArt/P14_plano.md` §4 «L»):
//! sem `CenaDeLuz.com_normal` o solver não grava `M` e o `fs_main_n` do sprite lê a 1×1 neutra (a luz
//! de frente). A shell passa `com_sprites_de_normal(&renderer.caixas_com_normal(present))` antes do
//! `recolher`; só uma caixa que cruze a GRADE da luz (a vista com a margem) liga `M`.

use bevy_ecs::world::World;
use ph2d_app_light2d::LuzDoQuadro;
use ph2d_ecs::{GlobalTransform, PresentWorld, Transform};
use ph2d_light2d_ecs::{Luz, TipoDeLuz};

use crate::comum::{H, W, afim, gpu, renderer};

/// A vista é `x ∈ [−2, 2]`, `y ∈ [−1, 1]`: um sprite de 1 m no meio, e outro a 100 m.
const NA_VISTA: [f64; 4] = [-0.5, -0.5, 0.5, 0.5];
const LONGE: [f64; 4] = [99.5, 99.5, 100.5, 100.5];

fn com_lampada() -> World {
    let mut w = World::new();
    let t = Transform::IDENTITY;
    w.spawn((
        Luz {
            cor: [1.0; 3],
            intensidade: 1.0,
            tamanho_m: 0.25,
            alcance_m: f32::INFINITY,
            tipo: TipoDeLuz::Ponto,
        },
        t,
        GlobalTransform::from_transform(t),
    ));
    w
}

#[test]
fn um_sprite_com_normal_liga_o_momento_da_cena() {
    let world = com_lampada();
    let mut luz = LuzDoQuadro::new();
    luz.preparar(&world, afim(), (W, H));
    assert!(
        !luz.cena().com_normal,
        "sem normal à vista a cena pediu o momento"
    );
    luz.com_sprites_de_normal(&[NA_VISTA]);
    luz.preparar(&world, afim(), (W, H));
    assert!(
        luz.cena().com_normal,
        "o sprite com normal não ligou o momento (preparar)"
    );
    luz.recolher(&world);
    assert!(
        luz.cena().com_normal,
        "o sprite com normal não ligou o momento (recolher)"
    );
    luz.com_sprites_de_normal(&[]);
    luz.recolher(&world);
    assert!(
        !luz.cena().com_normal,
        "tirar a normal não desligou o momento"
    );
}

/// ⭐ «À vista», não «registada»: um sprite com normal fora da grade não paga `M` (0 bytes).
#[test]
fn uma_normal_fora_da_vista_nao_liga_o_momento() {
    let world = com_lampada();
    let mut luz = LuzDoQuadro::new();
    luz.com_sprites_de_normal(&[LONGE]);
    luz.preparar(&world, afim(), (W, H));
    luz.recolher(&world);
    assert!(
        !luz.cena().com_normal,
        "uma normal a 100 m da vista ligou o momento"
    );
    let Some(gpu) = gpu() else {
        eprintln!("normal fora da vista: sem placa — saltado o lado dos bytes");
        return;
    };
    let r = renderer(&gpu);
    let bytes = |caixas: &[[f64; 4]]| {
        let mut luz = LuzDoQuadro::new();
        luz.com_sprites_de_normal(caixas);
        luz.preparar(&world, afim(), (W, H));
        luz.recolher(&world);
        luz.gravar(&gpu, &mut PresentWorld::new(), &r);
        luz.solver().map_or(0, |s| s.bytes_do_momento())
    };
    let (longe, perto) = (bytes(&[LONGE]), bytes(&[LONGE, NA_VISTA]));
    eprintln!("normal fora da vista: momento {longe} B longe · {perto} B com uma à vista");
    assert_eq!(longe, 0, "a normal fora da vista alocou {longe} B de M");
    assert!(perto > 0, "o controlo: a normal à vista não alocou M");
}

/// O lado da textura do sprite (1 m ⇒ 16 texels a 16 px/m).
const LADO: u32 = 16;

/// O hemisfério (y para cima): `n_xy` = a posição no disco, `n_z` o resto; fora do disco, o equador.
fn hemisferio() -> Vec<u8> {
    let mut d = Vec::new();
    for y in 0..LADO {
        for x in 0..LADO {
            let c = f64::from(LADO) / 2.0;
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

/// ⭐ **O solver REAL** (não `F`/`M` sintéticos): uma lâmpada à esquerda, o sprite do motor com o
/// hemisfério no meio. A razão claro/escuro (luma a ±0,3 m do centro) com a normal é maior que a do
/// MESMO sprite sem ela (só a distância à lâmpada); sem normal, 0 B de `M`.
#[test]
fn o_sprite_com_normal_acende_com_o_solver_real() {
    use crate::comum::{Cadeia, quad};
    let Some(gpu) = gpu() else {
        eprintln!("solver real: sem placa — saltado");
        return;
    };
    let mut w = World::new();
    let mut t = Transform::IDENTITY;
    t.translation.x = -1.5;
    w.spawn((Luz::default(), t, GlobalTransform::from_transform(t)));
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
    let medir = |com_normal: bool| {
        let mut c = Cadeia::nova(&gpu);
        let cor: Vec<u8> = (0..LADO * LADO)
            .flat_map(|_| [160, 160, 160, 255])
            .collect();
        let tex = c
            .renderer
            .acquire_individual(LADO, LADO, &cor)
            .expect("cor");
        if com_normal {
            c.renderer
                .definir_normal(tex, Some((LADO, LADO, &hemisferio())))
                .expect("normal");
        }
        let mut present = PresentWorld::new();
        present.world_mut().spawn(quad(tex, [0.0, 0.0]));
        let mut l = LuzDoQuadro::com_degrau(ph2d_app_light2d::DEGRAU_PADRAO);
        for _ in 0..30 {
            l.com_sprites_de_normal(&c.renderer.caixas_com_normal(&mut present));
            assert!(l.preparar(&w, afim(), (W, H)));
            l.recolher(&w);
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
        (luma(&b, px([-0.3, 0.0])), luma(&b, px([0.3, 0.0])), m)
    };
    let (c0, e0, m0) = medir(false);
    let (c1, e1, m1) = medir(true);
    let (r0, r1) = (c0 / e0.max(1.0), c1 / e1.max(1.0));
    eprintln!(
        "solver real: sem normal {c0:.1}/{e0:.1} = {r0:.3} (M {m0} B) · com o hemisfério {c1:.1}/{e1:.1} = {r1:.3} (M {m1} B)"
    );
    assert_eq!(m0, 0, "sem normal o momento não é alocado (G6)");
    assert!(m1 > 0, "o sprite com normal à vista liga o momento");
    assert!(
        r1 > 1.5 * r0,
        "a normal do sprite tem de sombrear com o solver real: {r1:.3} contra {r0:.3}"
    );
}
