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
use ph2d_asset_index::{AssetEntry, AssetIndex, AssetRef, Thumb};
use ph2d_ecs::{Children, Entity, MasterRoot, Name, SimWorld, SpritePixels, StableId};
use std::cell::RefCell;
use std::collections::BTreeMap;

/// Quantos pixels a média amostra, no máximo.
///
/// ⚠️ **É um teto de RELÓGIO, e a conta está aqui:** a média percorre a imagem com passo, então
/// uma textura de 4096² custa o mesmo que uma de 64² — `4096` amostras, ~4 µs. Sem o passo, a
/// mesma textura custaria 16,7 M amostras (~14 ms, um quadro inteiro) **por textura**.
/// ⚠️ E o resultado é guardado por `AssetId` na [`CardArt`], então a conta corre **uma vez por
/// conteúdo**, não uma vez por quadro.
const SWATCH_SAMPLES: usize = 4096;

/// ⭐ **A memória do que um cartão DESENHA**, chaveada por CONTEÚDO — o que a torna reutilizável
/// entre quadros, entre entidades e depois de um undo.
///
/// ⚠️ **Ela é obrigatória, não uma optimização.** A `TextureLibrary` reescreve a entrada de cada
/// textura **a cada quadro** (é assim que um nome novo chega lá), então sem esta memória a média
/// de cor e a redução da miniatura correriam 60×/s por textura. A cor tem tecto de amostras; a
/// miniatura **não pode ter** — ela lê a imagem inteira, e é exactamente por isso que a resposta
/// se guarda.
#[derive(Default)]
pub(crate) struct CardArt {
    /// A cor dominante já calculada.
    swatches: BTreeMap<AssetId, [u8; 4]>,
    /// ⭐⭐ A miniatura já reduzida (wave A6). ⚠️ O `Arc` guardado aqui é o que faz a igualdade
    /// `O(1)` do [`Thumb`] funcionar a jusante: enquanto o conteúdo não muda, o painel recebe **o
    /// mesmo ponteiro** e não reconstrói a textura de GPU.
    thumbs: BTreeMap<AssetId, Thumb>,
}

impl CardArt {
    /// Uma memória vazia.
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

thread_local! {
    /// A cache viva da sessão. Ela é `thread_local` e não um campo do `App` porque a chave é o
    /// **conteúdo** — ela não pertence a um projecto, a uma cena nem a um quadro, e sobrevive
    /// correctamente a um `Open Project` (os bytes iguais dão a mesma cor).
    static SWATCHES: std::cell::RefCell<CardArt> = const {
        std::cell::RefCell::new(CardArt {
            swatches: BTreeMap::new(),
            thumbs: BTreeMap::new(),
        })
    };
    /// A biblioteca de texturas da sessão — ver [`TextureLibrary`].
    static LIBRARY: RefCell<TextureLibrary> = const {
        RefCell::new(TextureLibrary {
            entries: BTreeMap::new(),
        })
    };
}

/// ⭐ **A publicação do quadro.** Chamada uma vez por quadro pelo `snapshots::publish`.
///
/// ⚠️ **`visible == false` não publica nada, e é a decisão:** o índice é uma travessia do mundo,
/// e pagá-la com o painel fechado seria trabalho que ninguém lê. ⛔ O preço de o publicar não é
/// zero e está medido no handoff — é por isso que a guarda existe em vez de «é barato».
pub(crate) fn publish_for_frame(sim: &mut SimWorld, db: &AssetDb, visible: bool) {
    if !visible {
        return;
    }
    let index = SWATCHES
        .with(|sw| LIBRARY.with(|lib| build(sim, db, &mut sw.borrow_mut(), &mut lib.borrow_mut())));
    ph2d_panel_asset_browser::set_current_index(index);
}

/// Reconstrói o índice a partir do mundo + do `AssetDb`.
///
/// ⚠️ **Recebe `&mut SimWorld`** porque `World::query` o exige (o `QueryState` é construído no
/// mundo). Ele **não escreve nada** — e há gate a afirmá-lo.
pub(crate) fn build(
    sim: &mut SimWorld,
    db: &AssetDb,
    swatches: &mut CardArt,
    remembered: &mut TextureLibrary,
) -> AssetIndex {
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
            .map(|id| AssetRef::Texture {
                asset: *id.as_bytes(),
            })
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
        // ⭐⭐ **A miniatura de um Prefab é a da PEÇA MAIOR dele** (wave A6).
        //
        // ⚠️ **Isto não é o retrato do prefab, e a diferença está declarada.** O retrato a sério é
        // um render offscreen da sub-árvore, e ele está BLOQUEADO por uma medição: esta função
        // corre sem `gpu`, sem `renderer` e sem `vello_pass` em mãos (o índice é construído no
        // `snapshots::publish`), então um retrato teria de nascer noutra fase e ser **consultado**
        // daqui — o molde é o `ObjectBake::thumbnail_for`, e é wave própria.
        //
        // ⭐ O que isto compra hoje: **no caso comum um prefab é UMA peça**, e aí a peça maior *é*
        // o prefab — a miniatura fica exacta. Num prefab de várias peças ela é parcial, e o que a
        // torna honesta é a linha de detalhe ao lado dizer *«N pieces»*.
        //
        // ⚠️ **Maior por ÁREA do `Sprite`, com desempate pelo `StableId`** — sem o desempate, duas
        // peças do mesmo tamanho fariam o cartão trocar de imagem entre quadros ao sabor da ordem
        // de arquétipo.
        entry.thumb =
            largest_piece_texture(sim, &pieces).and_then(|id| thumb_for(db, id, swatches));
        entry.deps = deps;
        index.push(entry);
    }

    // ── Fonte 2: as TEXTURAS ───────────────────────────────────────────────────────────────────
    //
    // ⛔⛔ **O `AssetDb` NÃO é a lista de assets do artista, e a 1.ª versão tratava-o como se
    // fosse.** Report do Enio, 2026-08-30: *«o painel de assets apareceu e está com várias sprites
    // que ninguém colocou lá»* — eram as 16 do átlas de demonstração que o ARRANQUE carrega de
    // `./assets/sprites`. Elas estão no `AssetDb` porque o boot as pôs lá, não porque alguém as
    // trouxe. ⇒ o `tracked_paths()` deixou de ser fonte de ENTRADAS e passou a ser só fonte de
    // NOMES para as que qualificam.
    //
    // ⇒ **Uma textura é um asset quando uma ENTIDADE a referencia** — isto é, quando o artista a
    // importou, pintou ou editou (o `SpritePixels` é o carimbo dessas oito portas).
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
        // ⚠️ O nome de FICHEIRO ganha ao da entidade quando ele existe — é o que o artista
        // reconhece. O `AssetDb` continua a ser consultado; o que mudou é que ele já não decide
        // **quem** está na lista.
        let name = db
            .tracked_paths()
            .into_iter()
            .find(|p| db.id_for_path(p) == Some(id))
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .or_else(|| sim.world().get::<Name>(entity).map(|n| n.0.clone()))
            .unwrap_or_else(|| id.to_hex()[..12].to_string());
        remembered.remember(id, texture_entry(db, id, name, swatches));
    }

    // ⭐⭐ **E a BIBLIOTECA LEMBRA.** Report do Enio, 2026-08-30: *«ao deletar o objeto do canvas, o
    // do painel assets foi deletado»*.
    //
    // ⚠️ **Uma textura não é um objecto da cena — ela é CONTEÚDO** (bytes endereçados por blake3).
    // Derivá-la do mundo a cada quadro fazia da biblioteca um espelho da cena: apagar a sprite que
    // a usava apagava o asset. *Uma biblioteca que perde o que o artista trouxe não é uma
    // biblioteca.*
    //
    // ⛔ E a memória é **união, nunca subtracção**: o que entrou fica até alguém o mandar sair. A
    // porta de remover é wave própria — hoje não existe, e uma remoção automática é exactamente o
    // que este report recusa.
    for entry in remembered.entries() {
        index.push(entry.clone());
    }

    index
}

/// ⭐ **A memória da biblioteca de texturas** — o que o artista trouxe, por CONTEÚDO.
///
/// ⚠️ Ela é da SESSÃO e não do projecto, e o que a torna correcta é a chave ser o blake3 dos
/// bytes: reabrir um projecto reencontra as mesmas entradas pelas mesmas sprites. ⛔ Persisti-la
/// seria conteúdo derivado dentro do arquivo, e ela envelheceria contra o `AssetDb`.
#[derive(Default)]
pub(crate) struct TextureLibrary {
    entries: BTreeMap<AssetId, AssetEntry>,
}

impl TextureLibrary {
    /// Regista (ou actualiza) uma textura. ⚠️ **Nunca remove** — ver o bloco acima.
    fn remember(&mut self, id: AssetId, entry: AssetEntry) {
        self.entries.insert(id, entry);
    }

    fn entries(&self) -> impl Iterator<Item = &AssetEntry> {
        self.entries.values()
    }

    /// Quantas texturas a biblioteca conhece — para os gates.
    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

/// `"3 pieces"` / `"1 piece"` — o detalhe de um componente.
fn piece_count_label(n: usize) -> String {
    if n == 1 {
        "1 piece".to_string()
    } else {
        format!("{n} pieces")
    }
}

fn texture_entry(db: &AssetDb, id: AssetId, name: String, swatches: &mut CardArt) -> AssetEntry {
    let mut entry = AssetEntry::new(
        AssetRef::Texture {
            asset: *id.as_bytes(),
        },
        name,
    );
    if let Some((w, h)) = dimensions(db, id) {
        entry.detail = format!("{w}x{h}");
    }
    if let Some(rgba) = swatch_for(db, id, swatches) {
        entry.swatch = rgba;
    }
    entry.thumb = thumb_for(db, id, swatches);
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
fn swatch_for(db: &AssetDb, id: AssetId, cache: &mut CardArt) -> Option<[u8; 4]> {
    if let Some(hit) = cache.swatches.get(&id) {
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
    cache.swatches.insert(id, rgba);
    Some(rgba)
}

/// A textura da **maior** peça de uma receita — ver o bloco que a chama para o porquê.
///
/// ⚠️ Uma peça sem `Sprite` mas com `SpritePixels` conta com área `0`: ela ainda é a única
/// candidata num prefab que só tenha essa, e devolver `None` ali daria um cartão cinzento sobre um
/// asset que tem imagem.
fn largest_piece_texture(sim: &SimWorld, pieces: &[Entity]) -> Option<AssetId> {
    let mut best: Option<(u64, u64, AssetId)> = None; // (área em ulps, id de desempate, textura)
    for &p in pieces {
        let Some(px) = sim.world().get::<SpritePixels>(p) else {
            continue;
        };
        let area = sim.world().get::<ph2d_render::Sprite>(p).map_or(0.0, |s| {
            f64::from(s.size[0].abs()) * f64::from(s.size[1].abs())
        });
        // A ordem é sobre `f64`, que não é `Ord`; a chave inteira mantém a comparação total **e**
        // determinística — um `partial_cmp` com `NaN` devolveria `None` e o `max_by` escolheria ao
        // acaso.
        let key = (area.max(0.0) * 1e6) as u64;
        let tie = sim.world().get::<StableId>(p).map_or(u64::MAX, |s| s.0);
        let cand = (key, tie, px.0);
        if best.is_none_or(|b| (cand.0, std::cmp::Reverse(cand.1)) > (b.0, std::cmp::Reverse(b.1)))
        {
            best = Some(cand);
        }
    }
    best.map(|(_, _, id)| id)
}

/// ⭐⭐ **A miniatura do cartão, com memória por conteúdo** (wave A6).
///
/// ⚠️ **Sem tecto de amostras, ao contrário da cor — e isso é a razão da cache, não um descuido.**
/// Uma média de cor pode saltar pixels porque a resposta é UM número; uma miniatura é a forma, e
/// saltar pixels apaga exactamente o que se quer ver. ⇒ ela lê a imagem inteira **uma vez por
/// conteúdo** e a resposta fica guardada. A `TextureLibrary` reescreve a entrada a cada quadro, e
/// sem esta memória seria uma passagem completa por textura, 60×/s.
///
/// ⚠️ **Devolve sempre o MESMO `Arc` para o mesmo conteúdo** — é isso que deixa o painel decidir
/// em `O(1)` que a imagem não mudou e não reconstruir a textura de GPU. ⛔ Um `Arc` novo por quadro
/// faria o `vello` reenviar cada cartão ao atlas **todo o quadro**.
fn thumb_for(db: &AssetDb, id: AssetId, cache: &mut CardArt) -> Option<Thumb> {
    if let Some(hit) = cache.thumbs.get(&id) {
        return Some(hit.clone());
    }
    let asset = db.get(&id)?;
    // ⚠️ **Aqui a porta é a `image_rgba8`, e não o `match` que a cor usa** — ela cobre as DUAS
    // variantes (a de 16 bits sai convertida), e é isso que fecha a dívida que o `swatch_for`
    // declara no `_`: *«uma imagem de 16 bits fica com a cor neutra até a A6 desenhar a miniatura
    // a sério»*. A conversão custa uma descodificação inteira, que aqui já se paga na mesma — e
    // **uma vez só**, por conteúdo.
    let (w, h, pixels) = asset.image_rgba8()?;
    if w == 0 || h == 0 || pixels.len() < (w as usize) * (h as usize) * 4 {
        return None;
    }
    let (rgba, tw, th) = crate::thumbnail::reduce(&pixels, w, h);
    let thumb = Thumb { rgba, w: tw, h: th };
    cache.thumbs.insert(id, thumb.clone());
    Some(thumb)
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
            .spawn((
                Transform::IDENTITY,
                Name::new("Ragdoll"),
                MasterRoot,
                StableId(1),
            ))
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
        let mut cache = CardArt::new();
        let mut lib = TextureLibrary::default();
        let index = build(&mut sim, &db, &mut cache, &mut lib);
        let counts = index.counts();
        assert_eq!(counts.get(&AssetKind::Component), Some(&1));
        assert_eq!(counts.get(&AssetKind::Texture), Some(&1));
    }

    /// A dependência é guardada **só num sentido**, e o índice inverte-a.
    #[test]
    fn the_component_declares_the_texture_and_the_texture_names_its_owner() {
        let db = AssetDb::new();
        let (mut sim, id) = world_with_one_component(&db);
        let mut cache = CardArt::new();
        let mut lib = TextureLibrary::default();
        let index = build(&mut sim, &db, &mut cache, &mut lib);
        let tex = AssetRef::Texture {
            asset: *id.as_bytes(),
        };
        let owners: Vec<&str> = index.owners(&tex).iter().map(|e| e.name.as_str()).collect();
        assert_eq!(owners, vec!["Ragdoll"]);
    }

    /// ⛔⛔ **A lente 1 da auditoria, executável:** apagar a receita do mundo tem de a tirar do
    /// índice. É isto que a reconstrução compra e que a mutação por evento perderia.
    #[test]
    fn deleting_the_master_removes_it_from_the_next_build() {
        let db = AssetDb::new();
        let (mut sim, _) = world_with_one_component(&db);
        let mut cache = CardArt::new();
        let mut lib = TextureLibrary::default();
        assert_eq!(
            build(&mut sim, &db, &mut cache, &mut lib)
                .counts()
                .get(&AssetKind::Component),
            Some(&1)
        );
        let root = {
            let mut q = sim
                .world_mut()
                .query_filtered::<Entity, bevy_ecs::prelude::With<MasterRoot>>();
            q.iter(sim.world()).next().unwrap()
        };
        sim.world_mut().entity_mut(root).remove::<MasterRoot>();
        let after = build(&mut sim, &db, &mut cache, &mut lib);
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
        let mut cache = CardArt::new();
        let rgba = swatch_for(&db, id, &mut cache).expect("uma imagem rgba8 tem cor");
        assert!(rgba[0] > 200, "vermelho esperado, veio {rgba:?}");
        assert!(
            rgba[1] < 40 && rgba[2] < 40,
            "sem outros canais, veio {rgba:?}"
        );
    }

    /// A cor calcula-se **uma vez por conteúdo** — a cache é chaveada pelo `AssetId`, e é isso que
    /// a torna reutilizável entre quadros e entre entidades.
    #[test]
    fn the_swatch_is_computed_once_per_content() {
        let db = AssetDb::new();
        let id = db.insert_image_rgba8(2, 2, vec![9u8; 16]);
        let mut cache = CardArt::new();
        let _ = swatch_for(&db, id, &mut cache);
        assert_eq!(cache.swatches.len(), 1);
        let _ = swatch_for(&db, id, &mut cache);
        assert_eq!(cache.swatches.len(), 1, "a segunda leitura nao recalcula");
    }

    /// ⭐⭐ **E a MINIATURA também** — a metade que o A6 acrescentou, e a que de facto obriga a
    /// memória: a cor tem tecto de amostras, a miniatura lê a imagem inteira.
    ///
    /// ⚠️ **A barra não é «a cache tem 1 entrada», é «o `Arc` é o MESMO»** — é isso que o painel
    /// compara em `O(1)` para não reconstruir a textura de GPU. Um cache que devolvesse bytes
    /// iguais num `Arc` novo passaria na contagem e faria o `vello` reenviar cada cartão ao atlas
    /// todo o quadro, sem um único gate vermelho.
    #[test]
    fn the_thumbnail_is_reduced_once_and_hands_back_the_same_arc() {
        let db = AssetDb::new();
        let id = db.insert_image_rgba8(2, 2, vec![9u8; 16]);
        let mut cache = CardArt::new();
        let a = thumb_for(&db, id, &mut cache).expect("a miniatura sai de 2x2");
        assert_eq!(cache.thumbs.len(), 1);
        let b = thumb_for(&db, id, &mut cache).expect("a segunda leitura acerta na memória");
        assert_eq!(cache.thumbs.len(), 1, "a segunda leitura nao recalcula");
        assert!(
            std::sync::Arc::ptr_eq(&a.rgba, &b.rgba),
            "o mesmo conteúdo tem de devolver o MESMO ponteiro"
        );
    }

    /// ⚠️ **A ordem NÃO é a de iteração do ECS.** Ela sai do `StableId`, e por isso é a mesma em
    /// dois builds do mesmo mundo — o cartão debaixo do dedo não muda entre quadros.
    #[test]
    fn two_builds_of_the_same_world_agree_entry_for_entry() {
        let db = AssetDb::new();
        let (mut sim, _) = world_with_one_component(&db);
        let mut cache = CardArt::new();
        let mut lib = TextureLibrary::default();
        let a: Vec<AssetRef> = build(&mut sim, &db, &mut cache, &mut lib)
            .entries()
            .iter()
            .map(|e| e.key)
            .collect();
        let b: Vec<AssetRef> = build(&mut sim, &db, &mut cache, &mut lib)
            .entries()
            .iter()
            .map(|e| e.key)
            .collect();
        assert_eq!(a, b);
    }

    /// ⛔⛔ **O átlas do ARRANQUE não é a biblioteca do artista.** Report do Enio, 2026-08-30:
    /// *«o painel de assets apareceu e está com várias sprites que ninguém colocou lá»*.
    ///
    /// **Mutação que deve sangrar:** voltar a percorrer `db.tracked_paths()` como fonte de
    /// entradas — que é literalmente o estado em que o painel estava.
    #[test]
    fn textures_the_boot_loaded_but_nobody_placed_are_not_assets() {
        let db = AssetDb::new();
        // O boot carrega 16 texturas para o `AssetDb`; nenhuma entidade as referencia.
        for i in 0..16u8 {
            let _ = db.insert_image_rgba8(4, 4, vec![i; 64]);
        }
        let mut sim = SimWorld::new();
        let mut cache = CardArt::new();
        let mut lib = TextureLibrary::default();
        let index = build(&mut sim, &db, &mut cache, &mut lib);
        assert!(
            index.is_empty(),
            "o painel mostrou {} assets que ninguem colocou la'",
            index.len()
        );
    }

    /// ⭐⭐ **A biblioteca LEMBRA.** Report do Enio: *«ao deletar o objeto do canvas, o do painel
    /// assets foi deletado»*.
    ///
    /// **Mutação que deve sangrar:** reconstruir as texturas do mundo a cada quadro, sem memória.
    #[test]
    fn deleting_the_sprite_does_not_delete_the_texture_from_the_library() {
        let db = AssetDb::new();
        let (mut sim, id) = world_with_one_component(&db);
        let mut cache = CardArt::new();
        let mut lib = TextureLibrary::default();
        assert_eq!(
            build(&mut sim, &db, &mut cache, &mut lib)
                .counts()
                .get(&AssetKind::Texture),
            Some(&1)
        );

        // O artista apaga a sprite da cena.
        let victim = {
            let mut q = sim.world_mut().query::<(Entity, &SpritePixels)>();
            q.iter(sim.world()).map(|(e, _)| e).next().unwrap()
        };
        sim.world_mut().despawn(victim);

        let after = build(&mut sim, &db, &mut cache, &mut lib);
        assert_eq!(
            after.counts().get(&AssetKind::Texture),
            Some(&1),
            "a textura saiu da biblioteca quando o objecto foi apagado"
        );
        assert!(
            after
                .get(&AssetRef::Texture {
                    asset: *id.as_bytes()
                })
                .is_some(),
            "e' a MESMA textura que tem de ficar, pelo endereco de conteudo"
        );
        assert_eq!(lib.len(), 1);
    }

    /// ⛔ **A visibilidade NÃO alcança a biblioteca.** Report do Enio: *«mudei o hide no objeto da
    /// cena e o objeto do painel foi modificado»*. Esconder é vista; um asset é conteúdo.
    #[test]
    fn hiding_an_object_changes_nothing_in_the_library() {
        let db = AssetDb::new();
        let (mut sim, _) = world_with_one_component(&db);
        let mut cache = CardArt::new();
        let mut lib = TextureLibrary::default();
        let before: Vec<(String, String, [u8; 4])> = build(&mut sim, &db, &mut cache, &mut lib)
            .entries()
            .iter()
            .map(|e| (e.name.clone(), e.detail.clone(), e.swatch))
            .collect();

        // Esconde a raiz — a mesma marca que o olho da Hierarquia escreve.
        let root = {
            let mut q = sim
                .world_mut()
                .query_filtered::<Entity, bevy_ecs::prelude::With<MasterRoot>>();
            q.iter(sim.world()).next().unwrap()
        };
        sim.world_mut()
            .entity_mut(root)
            .insert(ph2d_ecs::Visibility { hidden: true });

        let after: Vec<(String, String, [u8; 4])> = build(&mut sim, &db, &mut cache, &mut lib)
            .entries()
            .iter()
            .map(|e| (e.name.clone(), e.detail.clone(), e.swatch))
            .collect();
        assert_eq!(before, after, "esconder mudou o que o painel mostra");
    }

    /// ⭐⭐ **Apagar a CÓPIA não apaga a RECEITA.** Report do Enio, 2026-08-30: *«o objeto foi até o
    /// painel corretamente mas ao deletar o objeto do canvas, o do painel assets foi deletado»*.
    ///
    /// ⚠️ Este gate mede o ÍNDICE, que é a metade que eu possuo: dada uma receita viva no mundo, o
    /// painel mostra-a. Se ele ficar verde e o report persistir, o defeito está no **verbo de
    /// apagar** (ele leva a receita junto), e não aqui — e é essa a próxima pergunta.
    #[test]
    fn deleting_the_copy_leaves_the_recipe_in_the_panel() {
        let db = AssetDb::new();
        let (mut sim, _) = world_with_one_component(&db);
        // A cópia que o *Make Component* deixa no lugar: uma raiz própria, SEM `MasterRoot`.
        let copy = sim
            .world_mut()
            .spawn((
                ph2d_ecs::Transform::IDENTITY,
                Name::new("Ragdoll"),
                StableId(50),
            ))
            .id();
        let mut cache = CardArt::new();
        let mut lib = TextureLibrary::default();
        assert_eq!(
            build(&mut sim, &db, &mut cache, &mut lib)
                .counts()
                .get(&AssetKind::Component),
            Some(&1)
        );
        sim.world_mut().despawn(copy);
        assert_eq!(
            build(&mut sim, &db, &mut cache, &mut lib)
                .counts()
                .get(&AssetKind::Component),
            Some(&1),
            "apagar a copia tirou a receita do painel"
        );
    }
}
