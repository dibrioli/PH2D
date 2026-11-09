//! **P14 — o Sprite do motor com normal map liga o momento `M`** (`docs/PixelArt/P14_plano.md` §4 «L»):
//! sem `CenaDeLuz.com_normal` o solver não grava `M` e o `fs_main_n` do sprite lê a 1×1 neutra (a luz
//! de frente). A shell diz `com_sprites_de_normal(renderer.normais_alocadas() > 0)` antes do `recolher`.

use bevy_ecs::world::World;
use ph2d_app_light2d::LuzDoQuadro;

use crate::comum::afim;

#[test]
fn um_sprite_com_normal_liga_o_momento_da_cena() {
    let world = World::new();
    let mut luz = LuzDoQuadro::new();
    luz.preparar(&world, afim(), (8, 8));
    assert!(
        !luz.cena().com_normal,
        "sem normal à vista a cena pediu o momento"
    );
    luz.com_sprites_de_normal(true);
    luz.preparar(&world, afim(), (8, 8));
    assert!(
        luz.cena().com_normal,
        "o sprite com normal não ligou o momento (preparar)"
    );
    luz.com_sprites_de_normal(false);
    luz.preparar(&world, afim(), (8, 8));
    assert!(
        !luz.cena().com_normal,
        "tirar a normal não desligou o momento"
    );
}
