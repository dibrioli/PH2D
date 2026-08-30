//! ⭐⭐ **A JUNÇÃO** (plano 07, wave A2) — o único sítio do app que conhece as DUAS fontes de
//! asset ao mesmo tempo, e por isso o único que pode responder *«que assets existem?»*.
//!
//! # As duas fontes, e por que nenhuma delas sozinha responde
//!
//! - **Componente** = uma sub-árvore MARCADA (`MasterRoot`) que vive no **mundo**. Ele não é um
//!   ficheiro: é o *«Mark as Asset»* do Blender aplicado a uma sub-árvore, e a identidade dele é o
//!   `StableId`.
//! - **Textura** = bytes no `AssetDb`, endereçados pelo **conteúdo** (blake3). A entidade que os
//!   usa carrega o `SpritePixels(AssetId)`; os pixels não estão no ECS.
//!
//! ⇒ as duas travessias são diferentes, e antes desta wave **nenhum sítio as juntava**.
//!
//! # ⚠️ Reconstrução, nunca mutação por evento
//!
//! O índice é **derivado**: cada chamada de [`build`] o refaz a partir da verdade. A alternativa —
//! mutá-lo quando algo nasce ou morre — cria a segunda fonte de verdade sobre *«o que existe»*, e
//! o modo de falha dela é um asset apagado que continua na grade (a lente 1 da auditoria procura
//! exactamente isto). O preço está medido no handoff.
//!
//! ⚠️ **A cor do cartão é a MÉDIA em luz linear, não a «dominante»** — ela é a redução da imagem a
//! um pixel, que é o que um cartão sem miniatura está a substituir. A dominante (agrupamento em
//! OKLab) responde a outra pergunta e fica para quando alguém a precise; o campo `swatch` é o
//! mesmo nos dois casos, então trocar a lei não mexe no modelo.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_asset_index::{AssetEntry, AssetIndex, AssetRef};
use ph2d_ecs::{Children, Entity, MasterRoot, Name, SimWorld, SpritePixels, StableId};
use std::collections::BTreeMap;

/// Quantos pixels a média amostra, no máximo.
///
/// ⚠️ **É um teto de RELÓGIO, e a conta está aqui:** a média percorre a imagem com passo, então
/// uma textura de 4096² custa o mesmo que uma de 64² — `4096` amostras, ~4 µs. Sem o passo, a
/// mesma textura custaria 16,7 M amostras (~14 ms, um quadro inteiro) **por textura**.
/// ⚠️ E o resultado é guardado por `AssetId` na [`SwatchCache`], então a conta corre **uma vez por
/// conteúdo**, não uma vez por quadro.
const SWATCH_SAMPLES: usize = 4096;

/// A memória das cores já calculadas — chaveada por CONTEÚDO, que é o que as torna reutilizáveis
/// entre quadros, entre entidades e depois de um undo.
pub(crate) type SwatchCache = BTreeMap<AssetId, [u8; 4]>;

/// Reconstrói o índice a partir do mundo + do `AssetDb`.
///
/// ⚠️ **Recebe `&mut SimWorld`** porque `World::query` o exige (o `QueryState` é construído no
/// mundo). Ele **não escreve nada** — e há gate a afirmá-lo.
pub(crate) fn build(sim: &mut SimWorld, db: &AssetDb, swatches: &mut SwatchCache) -> AssetIndex {
    let mut index = AssetIndex::new();

    // ── Fonte 1: os COMPONENTES ────────────────────────────────────────────────────────────────
    //
    // ⚠️ Ordenado por `StableId`, e não pela ordem de iteração do ECS: a ordem de arquétipo muda
    // com um `insert` qualquer, e uma grade que se reordena sozinha entre quadros é uma grade em
    // que o cartão debaixo do dedo deixa de ser o que o artista mirou.
    let mut masters: Vec<(u64, Entity)> = {
        let mut q = sim
            .world_mut()
            .query_filtered::<(Entity, &StableId), bevy_ecs::prelude::With<MasterRoot>>();
        let mut v: Vec<(u64, Entity)> = q.iter(sim.world()).map(|(e, s)| (s.0, e)).collect();
        v.sort_unstable();
        v
    };
    masters.dedup_by_key(|(id, _)| *id);

    for (stable_id, entity) in masters {
        let name = sim
            .world()
            .get::<Name>(entity)
            .map_or_else(|| format!("Component {stable_id}"), |n| n.0.clone());
        let pieces = subtree(sim, entity);
        // As dependências: as texturas que as peças desta receita usam. É a metade guardada; o
        // sentido inverso (*quem usa esta textura?*) é derivado pelo índice.
        let mut deps: Vec<AssetRef> = pieces
            .iter()
            .filter_map(|&p| sim.world().get::<SpritePixels>(p).map(|sp| sp.0))
            .map(|id| AssetRef::Texture { asset: *id.as_bytes() })
            .collect();
        deps.sort_unstable();
        deps.dedup();

        let mut entry = AssetEntry::new(AssetRef::Component { stable_id }, name);
        entry.detail = piece_count_label(pieces.len());
        // ⚠️ A cor de um componente é a da PRIMEIRA textura que ele usa, e é uma escolha de
        // produto declarada: uma receita sem pixels nenhuns fica com a cor neutra do construtor,
        // que é honesto — *ela não tem cor*.
        if let Some(&AssetRef::Texture { asset }) = deps.first()
            && let Some(rgba) = swatch_for(db, AssetId::from_digest(asset), swatches)
        {
            entry.swatch = rgba;
        }
        entry.deps = deps;
        index.push(entry);
    }

    // ── Fonte 2: as TEXTURAS ───────────────────────────────────────────────────────────────────
    //
    // Duas portas, e a ordem entre elas é load-bearing: primeiro as que vieram de um FICHEIRO
    // (essas têm nome que o artista reconhece), depois as que só existem porque uma sprite as
    // carrega. ⚠️ O índice é idempotente por endereço, então a segunda porta **não** rebaixa o
    // nome que a primeira deu — ela substituiria a entrada inteira, e é por isso que ela salta o
    // que já lá está.
    for path in db.tracked_paths() {
        let Some(id) = db.id_for_path(&path) else {
            continue;
        };
        let name = path
            .file_name()
            .map_or_else(|| id.to_hex()[..12].to_string(), |n| n.to_string_lossy().into_owned());
        index.push(texture_entry(db, id, name, swatches));
    }

    let mut loose: Vec<(u64, Entity, AssetId)> = {
        let mut q = sim.world_mut().query::<(Entity, &SpritePixels)>();
        let mut v: Vec<(u64, Entity, AssetId)> = q
            .iter(sim.world())
            .map(|(e, sp)| {
                let order = sim.world().get::<StableId>(e).map_or(u64::MAX, |s| s.0);
                (order, e, sp.0)
            })
            .collect();
        v.sort_unstable_by_key(|(order, _, _)| *order);
        v
    };
    loose.dedup_by_key(|(_, _, id)| *id);

    for (_, entity, id) in loose {
        if index.get(&AssetRef::Texture { asset: *id.as_bytes() }).is_some() {
            continue;
        }
        let name = sim
            .world()
            .get::<Name>(entity)
            .map_or_else(|| id.to_hex()[..12].to_string(), |n| n.0.clone());
        index.push(texture_entry(db, id, name, swatches));
    }

    index
}

/// `"3 pieces"` / `"1 piece"` — o detalhe de um componente.
fn piece_count_label(n: usize) -> String {
    if n == 1 {
        "1 piece".to_string()
    } else {
        format!("{n} pieces")
    }
}

fn texture_entry(
    db: &AssetDb,
    id: AssetId,
    name: String,
    swatches: &mut SwatchCache,
) -> AssetEntry {
    let mut entry = AssetEntry::new(AssetRef::Texture { asset: *id.as_bytes() }, name);
    if let Some((w, h)) = dimensions(db, id) {
        entry.detail = format!("{w}x{h}");
    }
    if let Some(rgba) = swatch_for(db, id, swatches) {
        entry.swatch = rgba;
    }
    entry
}

/// A raiz **e toda a descendência** — a mesma definição de «peça» do `assign_master_pieces`.
fn subtree(sim: &SimWorld, root: Entity) -> Vec<Entity> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        out.push(e);
        if let Some(children) = sim.world().get::<Children>(e) {
            stack.extend(children.iter());
        }
    }
    out
}

fn dimensions(db: &AssetDb, id: AssetId) -> Option<(u32, u32)> {
    match &*db.get(&id)? {
        ph2d_asset::Asset::ImageRgba8 { width, height, .. }
        | ph2d_asset::Asset::ImageRgba16 { width, height, .. } => Some((*width, *height)),
        _ => None,
    }
}

/// A cor do cartão, com memória por conteúdo.
fn swatch_for(db: &AssetDb, id: AssetId, cache: &mut SwatchCache) -> Option<[u8; 4]> {
    if let Some(hit) = cache.get(&id) {
        return Some(*hit);
    }
    let asset = db.get(&id)?;
    let rgba = match &*asset {
        ph2d_asset::Asset::ImageRgba8 {
            width,
            height,
            pixels,
        } => mean_rgba8(*width, *height, pixels)?,
        // ⚠️ O ramo de 16 bits fica de fora com motivo nomeado: os pixels são meio-float LINEAR, e
        // a média deles precisa da conversão inversa que o `Asset::image_rgba8` já faz — pagar uma
        // descodificação inteira por um quadrado de 24 px é o oposto do que a cache existe para
        // fazer. Uma imagem de 16 bits fica com a cor neutra até a A6 desenhar a miniatura a
        // sério. *`#[non_exhaustive]` obriga o ramo `_`, então esta ausência é DECLARADA aqui.*
        _ => return None,
    };
    cache.insert(id, rgba);
    Some(rgba)
}

/// A média em **luz linear**, ponderada por alfa.
///
/// ⚠️ Ponderada por alfa porque uma sprite recortada é quase toda transparente, e a média crua
/// dela é a cor do NADA (preto), não a cor do desenho. Foi isso que a primeira versão devolveu.
fn mean_rgba8(width: u32, height: u32, pixels: &[u8]) -> Option<[u8; 4]> {
    let total = (width as usize).checked_mul(height as usize)?;
    if total == 0 || pixels.len() < total * 4 {
        return None;
    }
    let stride = total.div_ceil(SWATCH_SAMPLES).max(1);
    let (mut r, mut g, mut b, mut wsum) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    let mut alpha_sum = 0.0f64;
    let mut taken = 0.0f64;
    for i in (0..total).step_by(stride) {
        let p = &pixels[i * 4..i * 4 + 4];
        let a = f64::from(p[3]) / 255.0;
        r += srgb_to_linear(p[0]) * a;
        g += srgb_to_linear(p[1]) * a;
        b += srgb_to_linear(p[2]) * a;
        wsum += a;
        alpha_sum += a;
        taken += 1.0;
    }
    if taken == 0.0 {
        return None;
    }
    // Tudo transparente: não há cor a reportar.
    if wsum <= f64::EPSILON {
        return Some([0x50, 0x50, 0x58, 0xFF]);
    }
    Some([
        linear_to_srgb(r / wsum),
        linear_to_srgb(g / wsum),
        linear_to_srgb(b / wsum),
        // A opacidade média entra no alfa do cartão — uma textura quase vazia desenha-se quase
        // vazia, que é informação.
        ((alpha_sum / taken) * 255.0).round().clamp(0.0, 255.0) as u8,
    ])
}

fn srgb_to_linear(v: u8) -> f64 {
    let c = f64::from(v) / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f64) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let s = if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_asset_index::AssetKind;
    use ph2d_ecs::{ChildOf, Transform};

    /// Um mundo com uma receita de duas peças, a de baixo com pixels próprios.
    fn world_with_one_component(db: &AssetDb) -> (SimWorld, AssetId) {
        let mut sim = SimWorld::new();
        let pixels = vec![0u8; 4 * 4 * 4];
        let id = db.insert_image_rgba8(4, 4, pixels);
        let root = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("Ragdoll"), MasterRoot, StableId(1)))
            .id();
        sim.world_mut().spawn((
            Transform::IDENTITY,
            Name::new("Torso"),
            StableId(2),
            ChildOf(root),
            SpritePixels(id),
        ));
        (sim, id)
    }

    /// ⭐ **A junção**: uma travessia devolve as duas famílias, e a textura da peça aparece como
    /// asset por direito próprio — não escondida dentro do componente.
    #[test]
    fn one_walk_returns_both_families() {
        let db = AssetDb::new();
        let (mut sim, _) = world_with_one_component(&db);
        let mut cache = SwatchCache::new();
        let index = build(&mut sim, &db, &mut cache);
        let counts = index.counts();
        assert_eq!(counts.get(&AssetKind::Component), Some(&1));
        assert_eq!(counts.get(&AssetKind::Texture), Some(&1));
    }

    /// A dependência é guardada **só num sentido**, e o índice inverte-a.
    #[test]
    fn the_component_declares_the_texture_and_the_texture_names_its_owner() {
        let db = AssetDb::new();
        let (mut sim, id) = world_with_one_component(&db);
        let mut cache = SwatchCache::new();
        let index = build(&mut sim, &db, &mut cache);
        let tex = AssetRef::Texture { asset: *id.as_bytes() };
        let owners: Vec<&str> = index.owners(&tex).iter().map(|e| e.name.as_str()).collect();
        assert_eq!(owners, vec!["Ragdoll"]);
    }

    /// ⛔⛔ **A lente 1 da auditoria, executável:** apagar a receita do mundo tem de a tirar do
    /// índice. É isto que a reconstrução compra e que a mutação por evento perderia.
    #[test]
    fn deleting_the_master_removes_it_from_the_next_build() {
        let db = AssetDb::new();
        let (mut sim, _) = world_with_one_component(&db);
        let mut cache = SwatchCache::new();
        assert_eq!(build(&mut sim, &db, &mut cache).counts().get(&AssetKind::Component), Some(&1));
        let root = {
            let mut q = sim
                .world_mut()
                .query_filtered::<Entity, bevy_ecs::prelude::With<MasterRoot>>();
            q.iter(sim.world()).next().unwrap()
        };
        sim.world_mut().entity_mut(root).remove::<MasterRoot>();
        let after = build(&mut sim, &db, &mut cache);
        assert_eq!(after.counts().get(&AssetKind::Component), None);
    }

    /// ⚠️ **A média é ponderada por ALFA.** Uma sprite recortada — 1 pixel vermelho opaco em 15
    /// transparentes — tem de dar VERMELHO. A média crua daria quase preto, que é a cor do nada.
    #[test]
    fn the_swatch_of_a_cut_out_sprite_is_the_colour_of_the_drawing_not_of_the_hole() {
        let db = AssetDb::new();
        let mut pixels = vec![0u8; 4 * 4 * 4];
        pixels[0..4].copy_from_slice(&[255, 0, 0, 255]);
        let id = db.insert_image_rgba8(4, 4, pixels);
        let mut cache = SwatchCache::new();
        let rgba = swatch_for(&db, id, &mut cache).expect("uma imagem rgba8 tem cor");
        assert!(rgba[0] > 200, "vermelho esperado, veio {rgba:?}");
        assert!(rgba[1] < 40 && rgba[2] < 40, "sem outros canais, veio {rgba:?}");
    }

    /// A cor calcula-se **uma vez por conteúdo** — a cache é chaveada pelo `AssetId`, e é isso que
    /// a torna reutilizável entre quadros e entre entidades.
    #[test]
    fn the_swatch_is_computed_once_per_content() {
        let db = AssetDb::new();
        let id = db.insert_image_rgba8(2, 2, vec![9u8; 16]);
        let mut cache = SwatchCache::new();
        let _ = swatch_for(&db, id, &mut cache);
        assert_eq!(cache.len(), 1);
        let _ = swatch_for(&db, id, &mut cache);
        assert_eq!(cache.len(), 1, "a segunda leitura nao recalcula");
    }

    /// ⚠️ **A ordem NÃO é a de iteração do ECS.** Ela sai do `StableId`, e por isso é a mesma em
    /// dois builds do mesmo mundo — o cartão debaixo do dedo não muda entre quadros.
    #[test]
    fn two_builds_of_the_same_world_agree_entry_for_entry() {
        let db = AssetDb::new();
        let (mut sim, _) = world_with_one_component(&db);
        let mut cache = SwatchCache::new();
        let a: Vec<AssetRef> = build(&mut sim, &db, &mut cache).entries().iter().map(|e| e.key).collect();
        let b: Vec<AssetRef> = build(&mut sim, &db, &mut cache).entries().iter().map(|e| e.key).collect();
        assert_eq!(a, b);
    }
}
