//! ⭐ **O Pivot de 9 pontos** ([ficha da Sprite](../../../docs/GameEngineFuncional/componentes/01_sprite.md)): os pontos
//! contam sobre o que está VISÍVEL (a caixa opaca do desenho), não sobre a borda da imagem. «Bottom» é o pé do boneco,
//! mesmo com sobra transparente em volta. Mudar o pivô nunca move o desenho: a posição compensa, como na ferramenta.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::{
    Entity, RepeatMode, ScrollRepeat, SpriteGrid, SpritePixels, SpriteRegion, TextureRepeat,
    TiledDraw, Transform, World,
};
use ph2d_editor_core::SpritePivot;
use ph2d_render::{Sprite, texture_of};
use std::cell::RefCell;
use std::collections::BTreeMap;

/// Menos do que isto, em metros locais, é o mesmo ponto (o arredondamento de `f32`, não geometria).
const MESMO_PONTO_M: f32 = 1.0e-3;

/// A janela de píxeis mostrada e como ela se corta: a chave do que já foi medido.
type Chave = (AssetId, [u32; 2], [u32; 4]);

thread_local! {
    /// A imagem é endereçada pelo conteúdo: a mesma chave dá sempre a mesma caixa, e varrer custa a imagem inteira.
    static CAIXAS: RefCell<BTreeMap<Chave, Option<[f32; 4]>>> = const { RefCell::new(BTreeMap::new()) };
}

/// A caixa opaca da janela `[x, y, w, h]` de `rgba` (RGBA8, `largura` px por linha): `[x0, y0, x1, y1]`, relativa
/// à janela, com o fim exclusivo. `None` se a janela é toda transparente.
#[must_use]
pub fn caixa_opaca(rgba: &[u8], largura: u32, janela: [u32; 4]) -> Option<[u32; 4]> {
    let [jx, jy, jw, jh] = janela;
    let mut caixa: Option<[u32; 4]> = None;
    for y in 0..jh {
        for x in 0..jw {
            let i = ((jy + y) as usize * largura as usize + (jx + x) as usize) * 4 + 3;
            if rgba.get(i).copied().unwrap_or(0) == 0 {
                continue;
            }
            caixa = Some(caixa.map_or([x, y, x + 1, y + 1], |c| {
                [c[0].min(x), c[1].min(y), c[2].max(x + 1), c[3].max(y + 1)]
            }));
        }
    }
    caixa
}

/// A fração do quad que tem desenho: `[u0, v0, u1, v1]`, `v` para baixo. Numa folha, a união das células: o pivô não
/// pode saltar quando o quadro troca.
#[must_use]
pub fn fracao_opaca(
    rgba: &[u8],
    largura: u32,
    area: [u32; 4],
    grade: [u32; 2],
) -> Option<[f32; 4]> {
    let cw = (area[2] / grade[0].max(1)).max(1);
    let ch = (area[3] / grade[1].max(1)).max(1);
    let mut uniao: Option<[u32; 4]> = None;
    for linha in 0..grade[1].max(1) {
        for coluna in 0..grade[0].max(1) {
            let janela = [area[0] + coluna * cw, area[1] + linha * ch, cw, ch];
            let Some(c) = caixa_opaca(rgba, largura, janela) else {
                continue;
            };
            uniao = Some(uniao.map_or(c, |u| {
                [
                    u[0].min(c[0]),
                    u[1].min(c[1]),
                    u[2].max(c[2]),
                    u[3].max(c[3]),
                ]
            }));
        }
    }
    uniao.map(|u| {
        [
            u[0] as f32 / cw as f32,
            u[1] as f32 / ch as f32,
            u[2] as f32 / cw as f32,
            u[3] as f32 / ch as f32,
        ]
    })
}

/// Quem repete enche o quad: o ponto conta sobre o quad inteiro.
fn repete(world: &World, e: Entity) -> bool {
    world.get::<TiledDraw>(e).is_some()
        || world.get::<ScrollRepeat>(e).is_some()
        || matches!(
            world.get::<TextureRepeat>(e),
            Some(TextureRepeat(RepeatMode::Enabled | RepeatMode::Mirror))
        )
}

/// A fração visível do quad de `e`; o quad inteiro se ela não tem píxeis conhecidos.
fn fracao_de(world: &World, e: Entity, db: &AssetDb, atlas: &BTreeMap<u32, AssetId>) -> [f32; 4] {
    const INTEIRO: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
    if repete(world, e) {
        return INTEIRO;
    }
    let Some(id) = texture_of(world.get::<SpritePixels>(e), world.get::<Sprite>(e), atlas) else {
        return INTEIRO;
    };
    let g = world
        .get::<SpriteGrid>(e)
        .copied()
        .unwrap_or(SpriteGrid::SINGLE);
    let grade = [g.hframes.max(1), g.vframes.max(1)];
    let regiao = world.get::<SpriteRegion>(e).map(|r| r.rect);
    let Some(asset) = db.get(&id) else {
        return INTEIRO;
    };
    let Some((w, h, px)) = asset.image_rgba8() else {
        return INTEIRO;
    };
    let area = regiao.map_or([0, 0, w, h], |r| {
        let x = (r[0].max(0.0) as u32).min(w.saturating_sub(1));
        let y = (r[1].max(0.0) as u32).min(h.saturating_sub(1));
        [
            x,
            y,
            (r[2] as u32).clamp(1, w - x),
            (r[3] as u32).clamp(1, h - y),
        ]
    });
    let chave = (id, grade, area);
    let medida = CAIXAS.with(|c| {
        *c.borrow_mut()
            .entry(chave)
            .or_insert_with(|| fracao_opaca(&px, w, area, grade))
    });
    medida.unwrap_or(INTEIRO)
}

/// O ponto `[coluna, linha]` da grade (`0..=2`, a linha de cima é a 0), em metros locais a partir do centro do quad.
fn ponto_local(s: &Sprite, f: [f32; 4], ponto: [u8; 2]) -> [f32; 2] {
    let meio = |a: f32, b: f32, i: u8| a + (b - a) * f32::from(i.min(2)) * 0.5;
    let u = meio(f[0], f[2], ponto[0]);
    let v = meio(f[1], f[3], ponto[1]);
    let u = if s.flip_x { 1.0 - u } else { u };
    let v = if s.flip_y { 1.0 - v } else { v };
    [(u - 0.5) * s.size[0], (0.5 - v) * s.size[1]]
}

/// Onde `local` cai no espaço do pai de `t`: a composição da casa, com rotação, escala e skew.
fn no_pai(t: Transform, local: [f32; 2]) -> Vec2 {
    let filho = Transform {
        translation: Vec2::new(local[0], local[1]),
        ..Default::default()
    };
    Transform::compose(t, filho).translation
}

/// Põe o pivô de `e` em `alvo` (metros locais a partir do centro do quad), sem mover o desenho. `false` se nada mudou.
fn pousa(world: &mut World, e: Entity, ppm: f32, alvo: [f32; 2]) -> bool {
    let Some(s) = world.get::<Sprite>(e).copied() else {
        return false;
    };
    let Some(t) = world.get::<Transform>(e).copied() else {
        return false;
    };
    let antes = s.resolve_anchor(ppm);
    let depois = [-alvo[0], -alvo[1]];
    if (antes[0] - depois[0]).abs() < MESMO_PONTO_M && (antes[1] - depois[1]).abs() < MESMO_PONTO_M
    {
        return false;
    }
    let delta = no_pai(t, antes) - no_pai(t, depois);
    // Os filhos também são desenho: ficam onde estão.
    let filhos: Vec<Entity> = world
        .get::<ph2d_ecs::Children>(e)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();
    for f in filhos {
        if let Some(mut tf) = world.get_mut::<Transform>(f) {
            tf.translation.x -= antes[0] - depois[0];
            tf.translation.y -= antes[1] - depois[1];
        }
    }
    let mut ent = world.entity_mut(e);
    if let Some(mut tr) = ent.get_mut::<Transform>() {
        tr.translation += delta;
    }
    if let Some(mut sp) = ent.get_mut::<Sprite>() {
        sp.anchor = depois;
        sp.centered = true;
        sp.offset = [0.0, 0.0];
    }
    true
}

/// ⭐ O campo Pivot do cartão Look. `false` se nada mudou.
pub fn set_pivot(
    world: &mut World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
    pivot: SpritePivot,
) -> bool {
    let Some(s) = world.get::<Sprite>(e).copied() else {
        return false;
    };
    let alvo = match pivot {
        SpritePivot::Ponto(p) => ponto_local(&s, fracao_de(world, e, db, atlas), p),
        SpritePivot::Px(px) => [px[0] / ppm, px[1] / ppm],
    };
    pousa(world, e, ppm, alvo)
}

/// O pivô de `e`: o ponto da grade em que ele cai (`None` = Custom) e a posição em px a partir do centro do quad, `y`
/// para cima. O centro do quad conta como Center: é onde toda sprite nasce.
#[must_use]
pub fn pivot_de(
    world: &World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
) -> Option<(Option<[u8; 2]>, [f32; 2])> {
    let s = world.get::<Sprite>(e)?;
    let a = s.resolve_anchor(ppm);
    let p = [-a[0], -a[1]];
    let px = [p[0] * ppm, p[1] * ppm];
    if p[0].abs() < MESMO_PONTO_M && p[1].abs() < MESMO_PONTO_M {
        return Some((Some([1, 1]), px));
    }
    let f = fracao_de(world, e, db, atlas);
    let casa = (0..3_u8)
        .flat_map(|l| (0..3_u8).map(move |c| [c, l]))
        .find(|q| {
            let v = ponto_local(s, f, *q);
            (v[0] - p[0]).abs() < MESMO_PONTO_M && (v[1] - p[1]).abs() < MESMO_PONTO_M
        });
    Some((casa, px))
}

#[cfg(test)]
#[path = "pivot_tests.rs"]
mod tests;
