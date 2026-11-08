//! ⭐ **A imagem aberta no Painter: UMA cópia inteira** (dono 06/11), sem ladrilhos nem corte. Ao largar, volta sozinha
//! ao que estava: nada a repor, a mesma lei de regresso de `ph2d_app_painter::skin_suspend`.
//!
//! ⚠️ **É uma vista, não uma edição:** publica-se no recurso [`ImagemAberta`], que os componentes autorados nunca veem.
//! O desenho lê-o no extract e em `ph2d_ecs::resolve_uv_xform`; o ponteiro e o chrome, por [`crate::sprite_mostrada`].

use ph2d_ecs::{Entity, ImagemAberta, ScrollRepeat, SpriteRegion, TiledDraw, UvTransform, World};
use ph2d_render::Sprite;

/// **O quad de UMA cópia da imagem inteira de `e`**, à escala a que o objecto a mostra (`canvas` = o tamanho dela em
/// px). Numa grelha é uma célula: quem desenha desdobra-a. O centro fica onde o bloco tem o dele: a âncora não muda.
#[must_use]
pub fn quad_da_imagem_inteira(world: &World, e: Entity, canvas: (u32, u32)) -> Option<[f32; 2]> {
    let size = world.get::<Sprite>(e)?.size;
    // A folha extra visível (plano GEF 26 F13): a imagem inteira dela, dita no quad que a grelha da folha `0` desdobra.
    let k = ph2d_ecs::shown_sheet(world, e);
    if let Some(s) = world.get::<ph2d_ecs::SpriteSheets>(e).and_then(|x| x.sheet(k)) {
        let g = world.get::<ph2d_ecs::SpriteGrid>(e).copied().unwrap_or(ph2d_ecs::SpriteGrid::SINGLE);
        let (n, n0) = ([s.hframes, s.vframes], [g.hframes, g.vframes]);
        return Some(std::array::from_fn(|i| s.size[i] * n[i] as f32 / n0[i].max(1) as f32));
    }
    if let Some(t) = world.get::<TiledDraw>(e) {
        return Some(t.tile);
    }
    let fundo = world.get::<ScrollRepeat>(e).map(|r| r.tile);
    let uv = world.get::<UvTransform>(e).map(|u| u.scale);
    let area = world.get::<SpriteRegion>(e).map(|r| r.rect);
    let px = [canvas.0, canvas.1];
    Some(std::array::from_fn(|i| {
        if let Some(t) = fundo.map(|t| t[i]).filter(|t| *t > 0.0) {
            t
        } else if let Some(k) = uv.map(|k| k[i]).filter(|k| *k > 0.0) {
            size[i] / k
        } else if let Some(w) = area.map(|r| r[2 + i]).filter(|w| *w > 0.0 && px[i] > 0) {
            size[i] * px[i] as f32 / w
        } else {
            size[i]
        }
    }))
}

/// Publica a vista de `aberta` (a imagem e o tamanho dela); `None` tira-a.
pub fn publica(world: &mut World, aberta: Option<(Entity, (u32, u32))>) {
    let agora = aberta.and_then(|(entity, canvas)| {
        let quad = quad_da_imagem_inteira(world, entity, canvas)?;
        Some(ImagemAberta { entity, quad })
    });
    match agora {
        Some(a) => {
            if world.get_resource::<ImagemAberta>() != Some(&a) {
                world.insert_resource(a);
            }
        }
        None => {
            world.remove_resource::<ImagemAberta>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{sprite_image_to_screen_affine, sprite_mostrada};
    use ph2d_ecs::{SimWorld, Transform};
    use ph2d_render::Camera2d;
    use ph2d_vector::Point;

    const JANELA: ph2d_host::WindowSize = ph2d_host::WindowSize {
        width: 800,
        height: 800,
    };
    /// A imagem da cena do dono: 256 × 128.
    const IMAGEM: (u32, u32) = (256, 128);

    /// ⭐⭐ O traço cai onde o cursor está: o píxel debaixo de um ponto da cópia desenhada é o desse ponto na imagem, pela
    /// mesma porta que a entrega do ponteiro usa. *Mutação: `sprite_mostrada` sem o quad ⇒ o pincel estica 3×.*
    #[test]
    fn o_traco_cai_onde_o_cursor_esta_na_copia() {
        let mut sim = SimWorld::default();
        let e = sim
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                Sprite::atlas(0, [6.0, 1.0], [1.0; 4]),
                TiledDraw { tile: [2.0, 1.0] },
            ))
            .id();
        publica(sim.world_mut(), Some((e, IMAGEM)));
        let w = sim.world();
        let camera = Camera2d::new([0.0, 0.0], 8.0);
        let afim = sprite_image_to_screen_affine(
            IMAGEM.0,
            IMAGEM.1,
            ph2d_ecs::world_transform(w, e).expect("a pose"),
            sprite_mostrada(w, e).expect("a sprite"),
            None,
            &camera,
            JANELA,
        );
        // 0,5 m à direita do centro da cópia de 2 m: 3/4 da largura da imagem, no meio da altura.
        let (sx, sy) = camera.world_to_screen([0.5, 0.0], JANELA);
        let px = afim.inverse() * Point::new(f64::from(sx), f64::from(sy));
        assert!((px.x - 192.0).abs() < 1e-2, "x do traço: {}", px.x);
        assert!((px.y - 64.0).abs() < 1e-2, "y do traço: {}", px.y);
    }

    /// ⭐ A Region: a imagem inteira, à escala a que a área se vê (era a imagem toda esmagada no quad da área).
    #[test]
    fn a_region_abre_a_imagem_inteira_a_mesma_escala() {
        let mut w = World::new();
        let e = w
            .spawn((
                Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
                SpriteRegion {
                    rect: [0.0, 0.0, 64.0, 64.0],
                    ..Default::default()
                },
            ))
            .id();
        assert_eq!(quad_da_imagem_inteira(&w, e, IMAGEM), Some([4.0, 2.0]));
    }

    /// ⭐ A folha extra visível abre inteira, no quad que a grelha da folha `0` desdobra ao tamanho DELA (plano GEF 26
    /// F13): 6 × 2 células de 1,5 m sobre uma grelha 4 × 1 ⇒ 9 m × 3 m ⇒ quad 2,25 × 3. *Mutação: ignorar a folha ⇒
    /// o quad de uma célula da folha 0.*
    #[test]
    fn a_folha_extra_visivel_abre_inteira_ao_tamanho_dela() {
        let mut w = World::new();
        let run = ph2d_ecs::ExtraSheet {
            asset: ph2d_asset_id::AssetId::from_digest([2; 32]),
            size: [1.5, 1.5],
            anchor: [0.0, 0.0],
            hframes: 6,
            vframes: 2,
        };
        let e = w
            .spawn((
                Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
                ph2d_ecs::SpriteGrid::new(4, 1),
                ph2d_ecs::SpriteSheets([run].into_iter().collect()),
                ph2d_ecs::ShownSheet(1),
            ))
            .id();
        assert_eq!(quad_da_imagem_inteira(&w, e, (288, 96)), Some([2.25, 3.0]));
    }

    /// CONTROLO: uma imagem parada não muda de quad.
    #[test]
    fn a_imagem_parada_abre_no_seu_quad() {
        let mut w = World::new();
        let e = w.spawn(Sprite::atlas(0, [4.0, 2.0], [1.0; 4])).id();
        assert_eq!(quad_da_imagem_inteira(&w, e, IMAGEM), Some([4.0, 2.0]));
    }
}
