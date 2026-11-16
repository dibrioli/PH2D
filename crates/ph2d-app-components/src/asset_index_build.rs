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

pub(crate) use crate::asset_card_art::{CardArt, dimensions, swatch_for, thumb_for};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_asset_index::{AssetEntry, AssetIndex, AssetRef};
use ph2d_ecs::{Children, Entity, MasterRoot, Name, SimWorld, SpritePixels, StableId};
use ph2d_i18n::{tr, tr_with};
use ph2d_render::texture_of;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// ⭐⭐⭐ **Quantos pixels de FONTE a redução de miniaturas pode consumir num quadro.**
///
/// ⛔⛔ **A auditoria de 2026-08-30 apanhou uma tabela sem tecto** (§0.0 ao contrário): o custo da
/// redução estava medido — `12,079 ms` para uma textura de 4096² — e o laço que a chama não tinha
/// orçamento nenhum. A afirmação que dispensava o tecto (*«custa isso uma vez, no quadro em que a
/// imagem entra — o mesmo em que ela já foi descodificada»*) é verdade para uma **importação
/// nova** e **falsa** para a abertura do painel, onde a biblioteca INTEIRA é reduzida de uma vez.
/// Dez texturas grandes ⇒ ~120 ms de congelamento ao abrir *Assets*.
///
/// # O número
///
/// A tabela dá `16,8 M` px em `12,079 ms` ⇒ **~0,72 ns/px**. `4 M` px ⇒ **~2,9 ms**, que é 17% de
/// um quadro de 60 fps — o mesmo tipo de fatia que o resto do quadro tolera.
///
/// ⚠️ **O recurso é o RELÓGIO, e a unidade é o pixel da fonte** porque é dele que o custo depende
/// (não do número de texturas: uma de 4096² custa 44× uma de 512²).
///
/// ⚠️ **A primeira redução de cada quadro corre SEMPRE**, mesmo que sozinha estoure o orçamento —
/// senão uma textura acima do tecto nunca teria miniatura, e o cartão dela ficaria colorido para
/// sempre sem ninguém saber porquê.
const THUMB_BUDGET_PX: u64 = 4_000_000;

thread_local! {
    /// A cache viva da sessão. Ela é `thread_local` e não um campo do `App` porque a chave é o
    /// **conteúdo** — ela não pertence a um projecto, a uma cena nem a um quadro, e sobrevive
    /// correctamente a um `Open Project` (os bytes iguais dão a mesma cor).
    /// O nome da última imagem da ranhura do Inspector ([`image_card`]).
    static CARD_NAME: RefCell<Option<(AssetId, String)>> = const { RefCell::new(None) };
    static SWATCHES: std::cell::RefCell<CardArt> = const {
        std::cell::RefCell::new(CardArt::EMPTY)
    };
    /// A biblioteca de texturas da sessão — ver [`TextureLibrary`].
    static LIBRARY: RefCell<TextureLibrary> = const {
        RefCell::new(TextureLibrary {
            entries: BTreeMap::new(),
            forgotten: std::collections::BTreeSet::new(),
        })
    };
}

/// ⭐ **A publicação do quadro.** Chamada uma vez por quadro pelo `snapshots::publish`.
///
/// ⚠️ **`visible == false` não publica nada, e é a decisão:** o índice é uma travessia do mundo,
/// e pagá-la com o painel fechado seria trabalho que ninguém lê. ⛔ O preço de o publicar não é
/// zero e está medido no handoff — é por isso que a guarda existe em vez de «é barato».
pub fn publish_for_frame(
    sim: &mut SimWorld,
    db: &AssetDb,
    atlas_assets: &BTreeMap<u32, AssetId>,
    catalogs: &ph2d_asset_index::CatalogTree,
    ppm: f32,
    visible: bool,
    documents: &[(
        ph2d_editor_core::interaction::drag_payload::DragPayload,
        String,
    )],
) {
    if !visible {
        return;
    }
    let mut index = SWATCHES.with(|sw| {
        LIBRARY.with(|lib| {
            build(
                sim,
                db,
                atlas_assets,
                catalogs,
                &mut sw.borrow_mut(),
                &mut lib.borrow_mut(),
                ppm,
            )
        })
    });
    push_documents(&mut index, documents, catalogs);
    push_sounds(&mut index, sim.world());
    ph2d_panel_asset_browser::set_current_index(index);
}

/// ⭐ GEF 21 E2 — **os sons do projecto entram nos Assets por referência**, como as imagens: um cartão
/// por som da [`ph2d_ecs::ProjectSounds`], com o ícone do som (sem pixels).
pub(crate) fn push_sounds(index: &mut AssetIndex, world: &ph2d_ecs::World) {
    let Some(lib) = world.get_resource::<ph2d_ecs::ProjectSounds>() else {
        return;
    };
    for (name, som) in lib.iter() {
        let key = AssetRef::Sound { asset: som.id };
        let mut e = AssetEntry::new(key, name.to_owned());
        e.detail = tr(key.kind().label_key()).to_string();
        index.push(e);
    }
}

/// ⭐ D17 (dono 30/10) — **os documentos do projecto entram nos Assets por referência**, como as
/// imagens: um Kanban VAZIO também (existe até ser apagado). Sem pixels: o cartão pinta o ícone.
pub(crate) fn push_documents(
    index: &mut AssetIndex,
    documents: &[(
        ph2d_editor_core::interaction::drag_payload::DragPayload,
        String,
    )],
    catalogs: &ph2d_asset_index::CatalogTree,
) {
    use ph2d_editor_core::interaction::drag_payload::DragPayload;
    for (doc, name) in documents {
        let key = match *doc {
            DragPayload::Kanban { id } => AssetRef::Kanban { id },
            DragPayload::Board { id } => AssetRef::Board { id },
            _ => continue,
        };
        let mut e = AssetEntry::new(key, name.clone());
        e.detail = tr(key.kind().label_key()).to_string();
        e.catalog = catalogs.catalog_of(&key);
        index.push(e);
    }
}

/// Reconstrói o índice a partir do mundo + do `AssetDb`.
///
/// ⚠️ **Recebe `&mut SimWorld`** porque `World::query` o exige (o `QueryState` é construído no
/// mundo). Ele **não escreve nada** — e há gate a afirmá-lo.
pub(crate) fn build(
    sim: &mut SimWorld,
    db: &AssetDb,
    atlas_assets: &BTreeMap<u32, AssetId>,
    catalogs: &ph2d_asset_index::CatalogTree,
    swatches: &mut CardArt,
    remembered: &mut TextureLibrary,
    ppm: f32,
) -> AssetIndex {
    let mut index = AssetIndex::new();
    let mut budget = THUMB_BUDGET_PX;

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
        let name = master_label(sim, entity, stable_id);
        let pieces = subtree(sim, entity);
        let mut deps: Vec<AssetRef> = pieces
            .iter()
            .flat_map(|&p| ph2d_render::images_of(sim.world(), p, atlas_assets))
            .map(|id| AssetRef::Texture {
                asset: *id.as_bytes(),
            })
            .collect();
        deps.sort_unstable();
        deps.dedup();

        let mut entry = AssetEntry::new(AssetRef::Component { stable_id }, name);
        entry.detail = piece_count_label(pieces.len());
        entry.receita_de_macro = sim.world().get::<ph2d_ecs::Macro>(entity).is_some();
        let face = largest_piece_texture(sim, &pieces, atlas_assets);
        if let Some(id) = face
            && let Some(rgba) = swatch_for(db, id, swatches)
        {
            entry.swatch = rgba;
        }
        entry.thumb = crate::asset_card_portrait::compose(sim, entity, &pieces, ppm, |p| {
            let id = ph2d_render::drawn_texture(sim.world(), p, atlas_assets)?;
            thumb_for(db, id, swatches, &mut budget)
        })
        .or_else(|| face.and_then(|id| thumb_for(db, id, swatches, &mut budget)));
        entry.deps = deps;
        entry.catalog = catalogs.catalog_of(&entry.key);
        index.push(entry);
    }

    let mut loose: Vec<(u64, Entity, AssetId)> = {
        let (mut q, w) = (sim.world_mut().query::<Entity>(), sim.world());
        let mut v: Vec<(u64, Entity, AssetId)> = q
            .iter(w)
            .flat_map(|e| {
                let order = w.get::<StableId>(e).map_or(u64::MAX, |s| s.0);
                ph2d_render::images_of(w, e, atlas_assets)
                    .into_iter()
                    .map(move |id| (order, e, id))
            })
            .collect();
        v.sort_unstable_by_key(|(order, _, _)| *order);
        v
    };
    loose.dedup_by_key(|(_, _, id)| *id);

    let live: std::collections::BTreeSet<AssetId> = loose.iter().map(|(_, _, id)| *id).collect();

    let paths = crate::library_names::paths_of(db);
    for (_, entity, id) in loose {
        let name = paths
            .get(&id)
            .and_then(|p| crate::library_names::stem(p))
            .or_else(|| sim.world().get::<Name>(entity).map(|n| n.0.clone()))
            .unwrap_or_else(|| id.to_hex()[..12].to_string());
        remembered.remember(id, texture_entry(db, id, name, swatches, &mut budget));
    }

    let normais = ph2d_sprite_edit::normais::em_uso();
    for id in atlas_assets.values().chain(&normais) {
        if !live.contains(id) && !remembered.is_forgotten(id) {
            remembered.remember(
                *id,
                texture_entry(db, *id, String::new(), swatches, &mut budget),
            );
        }
    }
    let names = crate::library_names::library(sim, db, atlas_assets, remembered);
    for entry in remembered.entries(&live) {
        let mut e = entry.clone();
        if let AssetRef::Texture { asset } = e.key
            && let Some(n) = names.get(&AssetId::from_digest(asset))
        {
            e.name.clone_from(n);
        }
        e.catalog = catalogs.catalog_of(&e.key);
        index.push(e);
    }

    index
}

/// ⭐ **A imagem da biblioteca que `e` mostra: o nome e a miniatura** — a ranhura do Inspector (plano GEF 12).
/// Pelas mesmas portas do cartão do navegador ([`texture_of`], o nome único, a miniatura com cache).
pub fn image_card(
    sim: &mut SimWorld,
    db: &AssetDb,
    atlas_assets: &BTreeMap<u32, AssetId>,
    e: Entity,
) -> Option<(String, ph2d_asset_index::Thumb)> {
    let w = sim.world();
    let id = texture_of(
        w.get::<SpritePixels>(e),
        w.get::<ph2d_render::Sprite>(e),
        atlas_assets,
    )?;
    let mut budget = THUMB_BUDGET_PX;
    let thumb = SWATCHES.with(|sw| thumb_for(db, id, &mut sw.borrow_mut(), &mut budget))?;
    let name = CARD_NAME.with(|c| {
        if let Some((cached, n)) = c.borrow().as_ref()
            && *cached == id
        {
            return n.clone();
        }
        let n = crate::library_names::session_library(sim, db, atlas_assets)
            .remove(&id)
            .unwrap_or_default();
        *c.borrow_mut() = Some((id, n.clone()));
        n
    });
    Some((name, thumb))
}

/// A miniatura da imagem `id` inteira (as folhas de um Animated no cartão Image, plano GEF 26 §3.3).
pub fn image_thumb(db: &AssetDb, id: AssetId) -> Option<ph2d_asset_index::Thumb> {
    SWATCHES.with(|sw| thumb_for(db, id, &mut sw.borrow_mut(), &mut { THUMB_BUDGET_PX }))
}

/// ⭐⭐⭐ **TIRAR UMA TEXTURA DA BIBLIOTECA** — a porta que a memória declarava não existir.
///
/// ⛔⛔ Report do Enio (2026-08-30, 2.ª ronda): *«uma sprite que foi deletada do canvas não
/// consegui deletar do painel»*. A biblioteca lembra por CONTEÚDO e nunca subtrai sozinha — que é
/// a cura de um report anterior e continua certa —, e o único gesto de saída **recusava**: com a
/// sprite apagada o `Select users` conta `0`, e a recusa dizia literalmente *«esta imagem está na
/// biblioteca porque 0 objecto(s) a usam — mude esses para a tirar»*. ⇒ **beco sem saída**: uma
/// frase que manda mudar um conjunto vazio.
///
/// ⚠️ **A recusa continua CERTA quando há utilizadores** — tirar a imagem deixaria aqueles
/// objectos sem pixels, e não há saída sem perda. O que estava errado era aplicá-la ao caso em que
/// **não há ninguém a perder nada**.
///
/// ⚠️ **Ela é estável por construção:** o `build` reconstrói o índice a partir da verdade, então
/// uma textura que ainda tenha entidade a referenciá-la volta no quadro seguinte — e é isso que
/// impede este gesto de mentir. *Esquecer é dizer «ninguém a usa», e o mundo é quem confirma.*
pub fn forget_texture(id: AssetId) {
    LIBRARY.with(|lib| lib.borrow_mut().forget(id));
}

/// A biblioteca da sessão, emprestada (o nome das imagens para o script, [`crate::library_names`]).
pub(crate) fn with_library<R>(f: impl FnOnce(&TextureLibrary) -> R) -> R {
    LIBRARY.with(|lib| f(&lib.borrow()))
}

/// ⭐⭐⭐ **O que o artista mandou SAIR** — e é isto que o undo desfaz (Enio, 2026-08-30:
/// *«deveria ter undo/redo no painel inclusive em del»*).
///
/// ⚠️ **A lápide é AUTORIA, a biblioteca é memória.** A `TextureLibrary` é reconstruída do mundo a
/// cada quadro, então ela não pode guardar uma DECISÃO — o quadro seguinte apagá-la-ia. O que se
/// guarda é *«esta imagem foi mandada sair»*, e essa frase viaja no `ProjectState`, que é a unidade
/// que o Ctrl+Z restaura.
///
/// ⛔ **Sem isto o gesto era irreversível**: a imagem sem utilizadores não tem quem a re-lembre no
/// quadro seguinte (o laço só vê entidades vivas), então esquecê-la era para sempre.
#[must_use]
pub fn forgotten_textures() -> Vec<[u8; 32]> {
    LIBRARY.with(|lib| {
        lib.borrow()
            .forgotten
            .iter()
            .map(|a| *a.as_bytes())
            .collect()
    })
}

/// A porta de volta: o undo repõe o conjunto inteiro.
///
/// ⚠️ **Conjunto inteiro e não «acrescenta»** — desfazer uma remoção tem de tirar a lápide, e um
/// `insert` só saberia pôr.
pub fn set_forgotten_textures(ids: &[[u8; 32]]) {
    LIBRARY.with(|lib| {
        lib.borrow_mut().forgotten = ids.iter().map(|d| AssetId::from_digest(*d)).collect();
    });
}

/// ⭐ **A memória da biblioteca de texturas** — o que o artista trouxe, por CONTEÚDO.
///
/// ⚠️ Ela é da SESSÃO e não do projecto, e o que a torna correcta é a chave ser o blake3 dos
/// bytes: reabrir um projecto reencontra as mesmas entradas pelas mesmas sprites. ⛔ Persisti-la
/// seria conteúdo derivado dentro do arquivo, e ela envelheceria contra o `AssetDb`.
#[derive(Default)]
pub(crate) struct TextureLibrary {
    entries: BTreeMap<AssetId, AssetEntry>,
    /// ⭐⭐ **As LÁPIDES** — o que o artista mandou sair. Ver [`forgotten_textures`].
    ///
    /// ⚠️ **Uma lápide, e não um `remove`**: a entrada fica, e é o `build` que a filtra. É isso que
    /// torna o gesto reversível — desfazer é tirar a lápide, e a entrada está lá para voltar.
    forgotten: std::collections::BTreeSet<AssetId>,
}

impl TextureLibrary {
    /// Regista (ou actualiza) uma textura. ⚠️ **Nunca remove** — ver o bloco acima.
    fn remember(&mut self, id: AssetId, entry: AssetEntry) {
        self.entries.insert(id, entry);
    }

    /// Esquece uma textura — ver [`forget_texture`], que é a porta.
    ///
    /// ⚠️ **Marca, não apaga** (2026-08-30): a entrada tem de sobreviver para o undo a poder
    /// devolver. Quem a esconde é o [`TextureLibrary::entries`].
    pub(crate) fn forget(&mut self, id: AssetId) {
        self.forgotten.insert(id);
    }

    /// ⭐⭐⭐ **As entradas VIVAS** — as com lápide não saem daqui, **excepto as que o mundo usa
    /// AGORA**.
    ///
    /// ⛔⛔ A excepção é o que impede o laço de render de editar o documento (auditoria de
    /// 2026-08-30). A 1.ª versão levantava a lápide dentro do `remember`, que corre **por quadro**
    /// — e a sequência era alcançável: tirar a imagem da biblioteca, **fechar** o painel,
    /// re-importar os mesmos bytes, reabrir ⇒ a lápide caía **sem gesto nenhum**, o quadro seguinte
    /// registava um passo espúrio, e um Ctrl+Z a repô-la era **desfeito no quadro a seguir**.
    /// *O Ctrl+Z não pegava e queimava um passo.*
    ///
    /// ⚠️ **A regra passa a ser a mesma que a recusa do verbo já usa**: uma imagem que alguém usa
    /// nunca está escondida — o `Remove from Library` recusa-a, e aqui ela ganha à lápide sem
    /// ninguém escrever no documento. `live` são os ids que o mundo referenciou **neste quadro**.
    pub(crate) fn entries<'a>(
        &'a self,
        live: &'a std::collections::BTreeSet<AssetId>,
    ) -> impl Iterator<Item = &'a AssetEntry> {
        self.entries
            .iter()
            .filter(|(id, _)| !self.forgotten.contains(id) || live.contains(id))
            .map(|(_, e)| e)
    }

    /// O artista mandou-a sair (a lápide).
    pub(crate) fn is_forgotten(&self, id: &AssetId) -> bool {
        self.forgotten.contains(id)
    }

    /// Quantas texturas a biblioteca MOSTRA — para os gates.
    ///
    /// ⚠️ **As com lápide não contam, e a distinção nasceu com elas** (2026-08-30): desde que
    /// esquecer é marcar em vez de apagar, `entries.len()` passou a responder *«quantas guardo»* e
    /// a pergunta dos gates é *«quantas o artista vê»*. As duas leem-se igual e divergem
    /// exactamente no caso que interessa. ⇒ ela conta pelo mesmo iterador que o `build` consome.
    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries(&std::collections::BTreeSet::new()).count()
    }

    /// **Só para os gates:** quantas entradas ela GUARDA, lápides incluídas.
    ///
    /// ⚠️ Ela existe porque a lei-título desta wave — *«esquecer MARCA, não apaga»* — não tinha
    /// instrumento (auditoria de 2026-08-30): um `forget` que marcasse **e** apagasse passava na
    /// suíte inteira, e o produto voltava a ser irreversível. `len()` conta o que se VÊ; esta conta
    /// o que se pode DEVOLVER, e é a diferença entre as duas que prova a lápide.
    #[cfg(test)]
    fn stored_len(&self) -> usize {
        self.entries.len()
    }
}

/// `"3 pieces"` / `"1 piece"` — o detalhe de um componente.
fn piece_count_label(n: usize) -> String {
    if n == 1 {
        tr("shell.asset_index_build.n1_piece").to_string()
    } else {
        tr_with("shell.asset_index_build.pieces", &[("n", &n)])
    }
}

fn texture_entry(
    db: &AssetDb,
    id: AssetId,
    name: String,
    swatches: &mut CardArt,
    budget: &mut u64,
) -> AssetEntry {
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
    entry.thumb = thumb_for(db, id, swatches, budget);
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

/// A textura da **maior** peça de uma receita — ver o bloco que a chama para o porquê.
///
/// ⚠️ Uma peça sem `Sprite` mas com `SpritePixels` conta com área `0`: ela ainda é a única
/// candidata num prefab que só tenha essa, e devolver `None` ali daria um cartão cinzento sobre um
/// asset que tem imagem.
fn largest_piece_texture(
    sim: &SimWorld,
    pieces: &[Entity],
    atlas_assets: &BTreeMap<u32, AssetId>,
) -> Option<AssetId> {
    let mut best: Option<(u64, u64, AssetId)> = None;
    for &p in pieces {
        let Some(id) = texture_of(
            sim.world().get::<SpritePixels>(p),
            sim.world().get::<ph2d_render::Sprite>(p),
            atlas_assets,
        ) else {
            continue;
        };
        let area = sim.world().get::<ph2d_render::Sprite>(p).map_or(0.0, |s| {
            f64::from(s.size[0].abs()) * f64::from(s.size[1].abs())
        });
        let key = (area.max(0.0) * 1e6) as u64;
        let tie = sim.world().get::<StableId>(p).map_or(u64::MAX, |s| s.0);
        let cand = (key, tie, id);
        if best.is_none_or(|b| (cand.0, std::cmp::Reverse(cand.1)) > (b.0, std::cmp::Reverse(b.1)))
        {
            best = Some(cand);
        }
    }
    best.map(|(_, _, id)| id)
}

#[cfg(test)]
#[path = "asset_index_build_tests.rs"]
mod tests;

/// Os gates da LÁPIDE — irmão pelo tecto de LOC (HR-18); o corte é por responsabilidade.
#[cfg(test)]
#[path = "asset_index_build_library_tests.rs"]
mod library_tests;

/// O nome que a biblioteca mostra para uma receita — o `Name` dela, ou o de recurso da tabela.
fn master_label(sim: &SimWorld, entity: Entity, stable_id: u64) -> String {
    sim.world().get::<Name>(entity).map_or_else(
        || {
            tr_with(
                "shell.asset_index_build.component",
                &[("stable_id", &stable_id)],
            )
        },
        |n| n.0.clone(),
    )
}
