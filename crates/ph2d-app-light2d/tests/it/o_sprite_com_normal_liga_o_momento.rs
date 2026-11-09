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
    let mut bytes = |caixas: &[[f64; 4]]| {
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
