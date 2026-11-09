//! ⭐ **O Pivot de 9 pontos** ([ficha da Sprite](../../../docs/GameEngineFuncional/componentes/01_sprite.md)): os pontos
//! contam sobre o que está VISÍVEL (a caixa opaca do desenho), não sobre a borda da imagem. «Bottom» é o pé do boneco,
//! mesmo com sobra transparente em volta. Mudar o pivô nunca move o desenho nem a colisão: a posição compensa.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::{
    Entity, RepeatMode, ScrollRepeat, SpriteGrid, SpritePixels, SpriteRegion, SpriteSheets,
    TextureRepeat, TiledDraw, Transform, World,
};
use ph2d_editor_core::SpritePivot;
use ph2d_physics_ecs::Collider;
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

/// ⭐ **A folha cujo Pivot se edita** (plano GEF 26 decisão 8): a visível no painel, como um `Sprite` com o tamanho e o
/// Pivot dela. `0` = o `Sprite` de sempre.
#[must_use]
pub fn sprite_do_pivot(world: &World, e: Entity) -> Option<(u8, Sprite)> {
    let k = ph2d_ecs::shown_sheet(world, e);
    ph2d_render::sheet_geometry(world, e, k, 0).map(|(s, _)| (k, s))
}

/// A fração visível do quad da folha `k` de `e`; o quad inteiro se ela não tem píxeis conhecidos.
fn fracao_de(
    world: &World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    k: u8,
) -> [f32; 4] {
    const INTEIRO: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
    if repete(world, e) {
        return INTEIRO;
    }
    let extra = world
        .get::<SpriteSheets>(e)
        .and_then(|s| s.sheet(k))
        .copied();
    let id = match extra {
        Some(s) => Some(s.asset),
        None => texture_of(world.get::<SpritePixels>(e), world.get::<Sprite>(e), atlas),
    };
    let Some(id) = id else {
        return INTEIRO;
    };
    let g = extra.map_or_else(
        || {
            world
                .get::<SpriteGrid>(e)
                .copied()
                .unwrap_or(SpriteGrid::SINGLE)
        },
        |s| s.grid(0),
    );
    let grade = [g.hframes.max(1), g.vframes.max(1)];
    let regiao = extra
        .is_none()
        .then(|| world.get::<SpriteRegion>(e).map(|r| r.rect))
        .flatten();
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
    // A caixa vira com a imagem; «Left» continua a ser o lado esquerdo do que se vê.
    let (u0, u1) = if s.flip_x {
        (1.0 - f[2], 1.0 - f[0])
    } else {
        (f[0], f[2])
    };
    let (v0, v1) = if s.flip_y {
        (1.0 - f[3], 1.0 - f[1])
    } else {
        (f[1], f[3])
    };
    let u = meio(u0, u1, ponto[0]);
    let v = meio(v0, v1, ponto[1]);
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

/// ⭐ O pivô andou `d` (metros locais, a âncora de antes menos a de agora) e a origem de `e` foi atrás: os filhos e a
/// colisão ficam onde estão no mundo (dono 05/11: o Pivot não separa desenho e colisão). O collider não sofre skew
/// (`ph2d_physics_ecs`: `offset ⊙ escala`, rodado), logo o skew de `t` entra no passo dele e não no dos filhos.
pub fn o_resto_fica(world: &mut World, e: Entity, t: Transform, d: [f32; 2]) {
    let filhos: Vec<Entity> = world
        .get::<ph2d_ecs::Children>(e)
        .map(|c| c.iter().copied().collect())
        .unwrap_or_default();
    for f in filhos {
        if let Some(mut tf) = world.get_mut::<Transform>(f) {
            tf.translation.x -= d[0];
            tf.translation.y -= d[1];
        }
    }
    let mut sem_rotacao = t;
    sem_rotacao.rotation = 0.0;
    let passo = no_pai(sem_rotacao, d) - no_pai(sem_rotacao, [0.0, 0.0]);
    let eixo = |p: f32, escala: f32, cru: f32| {
        if escala.abs() < MESMO_PONTO_M {
            cru
        } else {
            p / escala
        }
    };
    if let Some(mut c) = world.get_mut::<Collider>(e) {
        c.offset[0] -= eixo(passo.x, t.scale.x, d[0]);
        c.offset[1] -= eixo(passo.y, t.scale.y, d[1]);
    }
}

/// ⭐ **Uma lei para todas as folhas** (smoke do dono 09/11): o Pivot da folha `k` vai para `alvo` (metros locais a
/// partir do centro do quad dela) e o desenho DESSA folha, os filhos e a colisão ficam no mundo; a origem anda. As
/// outras folhas guardam o Pivot delas e seguem a origem: os pés de cada uma continuam no pivô. `false` se nada mudou.
fn pousa(world: &mut World, e: Entity, ppm: f32, k: u8, alvo: [f32; 2]) -> bool {
    let Some((s, _)) = ph2d_render::sheet_geometry(world, e, k, 0) else {
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
    o_resto_fica(world, e, t, [antes[0] - depois[0], antes[1] - depois[1]]);
    if let Some(mut tr) = world.get_mut::<Transform>(e) {
        tr.translation += delta;
    }
    poe_ancora(world, e, k, depois);
    true
}

/// O Pivot da folha `k` passa a `anchor`: o do `Sprite` na `0` (centrado, sem `offset`), o da folha extra nas outras.
fn poe_ancora(world: &mut World, e: Entity, k: u8, anchor: [f32; 2]) {
    if k == 0 {
        if let Some(mut sp) = world.get_mut::<Sprite>(e) {
            sp.anchor = anchor;
            sp.centered = true;
            sp.offset = [0.0, 0.0];
        }
        return;
    }
    if let Some(mut sheets) = world.get_mut::<SpriteSheets>(e)
        && let Some(s) = usize::from(k)
            .checked_sub(1)
            .and_then(|i| sheets.0.get_mut(i))
    {
        s.anchor = anchor;
    }
}

/// A ferramenta Pivot do canvas: a lei do [`pousa`], com a origem e a âncora que o arrasto calculou sobre o quad da
/// folha visível ([`sprite_do_pivot`]).
pub fn arrasta(world: &mut World, e: Entity, origem: [f32; 2], anchor: [f32; 2]) {
    let k = ph2d_ecs::shown_sheet(world, e);
    let Some(t) = world.get::<Transform>(e).copied() else {
        return;
    };
    if let Some((s, _)) = ph2d_render::sheet_geometry(world, e, k, 0) {
        o_resto_fica(
            world,
            e,
            t,
            [s.anchor[0] - anchor[0], s.anchor[1] - anchor[1]],
        );
    }
    if let Some(mut tr) = world.get_mut::<Transform>(e) {
        tr.translation = Vec2::new(origem[0], origem[1]);
    }
    if k == 0 {
        if let Some(mut s) = world.get_mut::<Sprite>(e) {
            s.anchor = anchor;
        }
    } else {
        poe_ancora(world, e, k, anchor);
    }
}

/// ⭐ **Mirror com a imagem no lugar** (dono 06/11): espelha o objeto inteiro, filhos e colisão, e a posição compensa o
/// pivô, que foi para o outro lado. O desenho parece ter feito só um Flip. `eixo`: 0 = horizontal, 1 = vertical.
pub fn espelha_no_lugar(world: &mut World, e: Entity, ppm: f32, eixo: usize) -> bool {
    let Some(a) = world.get::<Sprite>(e).map(|s| s.resolve_anchor(ppm)) else {
        return false;
    };
    let Some(t) = world.get::<Transform>(e).copied() else {
        return false;
    };
    let mut novo = t;
    if eixo == 0 {
        novo.scale.x = -t.scale.x;
    } else {
        novo.scale.y = -t.scale.y;
    }
    novo.translation += no_pai(t, a) - no_pai(novo, a);
    world.entity_mut(e).insert(novo);
    true
}

/// ⭐ **O centro do desenho para o encaixe do Pivot** (CTRL no arrasto), em metros locais desde o centro do quad da
/// folha cujo Pivot se edita (a visível, [`sprite_do_pivot`]). `img` = a imagem INTEIRA dessa folha. Folha `k ≥ 1`: a
/// união das células sobre o quad de UMA célula, como o Pivot de 9 pontos. Folha `0`: a lei de sempre. `None` = toda
/// transparente.
#[must_use]
pub fn centro_do_desenho(
    world: &World,
    e: Entity,
    img: &ph2d_render::SpriteImage,
) -> Option<[f32; 2]> {
    let (k, s) = sprite_do_pivot(world, e)?;
    let janela = [0, 0, img.width, img.height];
    // A mesma lei nas duas: a união das células sobre o quad de UMA, com o flip (auditoria do fecho 08/11, J).
    let extra = world
        .get::<SpriteSheets>(e)
        .and_then(|x| x.sheet(k))
        .filter(|_| k > 0);
    let grade = match extra {
        Some(x) => [x.hframes.max(1), x.vframes.max(1)],
        None => world
            .get::<SpriteGrid>(e)
            .map_or([1, 1], |g| [g.hframes.max(1), g.vframes.max(1)]),
    };
    let f = fracao_opaca(&img.pixels, img.width, janela, grade)?;
    Some(ponto_local(&s, f, [1, 1]))
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
    let Some((k, s)) = sprite_do_pivot(world, e) else {
        return false;
    };
    let alvo = match pivot {
        SpritePivot::Ponto(p) => ponto_local(&s, fracao_de(world, e, db, atlas, k), p),
        SpritePivot::Px(px) => [px[0] / ppm, px[1] / ppm],
    };
    pousa(world, e, ppm, k, alvo)
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
    let (k, s) = sprite_do_pivot(world, e)?;
    let a = s.resolve_anchor(ppm);
    let p = [-a[0], -a[1]];
    let px = [p[0] * ppm, p[1] * ppm];
    if p[0].abs() < MESMO_PONTO_M && p[1].abs() < MESMO_PONTO_M {
        return Some((Some([1, 1]), px));
    }
    let f = fracao_de(world, e, db, atlas, k);
    let casa = (0..3_u8)
        .flat_map(|l| (0..3_u8).map(move |c| [c, l]))
        .find(|q| {
            let v = ponto_local(&s, f, *q);
            (v[0] - p[0]).abs() < MESMO_PONTO_M && (v[1] - p[1]).abs() < MESMO_PONTO_M
        });
    Some((casa, px))
}

#[cfg(test)]
#[path = "pivot_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "pivot_espelho_tests.rs"]
mod espelho_tests;

#[cfg(test)]
#[path = "pivot_folhas_tests.rs"]
mod folhas_tests;
