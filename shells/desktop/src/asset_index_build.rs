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
// ⚠️ **A memória do que um cartão DESENHA** mudou-se para o irmão `asset_card_art` — ver o
// cabeçalho de lá. Re-exportada para os gates e os chamadores a nomearem como sempre.
pub(crate) use crate::asset_card_art::{CardArt, dimensions, swatch_for, thumb_for};
use ph2d_ecs::{Children, Entity, MasterRoot, Name, SimWorld, SpritePixels, StableId};
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
pub(crate) fn publish_for_frame(
    sim: &mut SimWorld,
    db: &AssetDb,
    atlas_assets: &BTreeMap<u32, AssetId>,
    catalogs: &ph2d_asset_index::CatalogTree,
    visible: bool,
) {
    if !visible {
        return;
    }
    let index = SWATCHES.with(|sw| {
        LIBRARY.with(|lib| {
            build(
                sim,
                db,
                atlas_assets,
                catalogs,
                &mut sw.borrow_mut(),
                &mut lib.borrow_mut(),
            )
        })
    });
    ph2d_panel_asset_browser::set_current_index(index);
}

/// Reconstrói o índice a partir do mundo + do `AssetDb`.
///
/// ⚠️ **Recebe `&mut SimWorld`** porque `World::query` o exige (o `QueryState` é construído no
/// mundo). Ele **não escreve nada** — e há gate a afirmá-lo.
pub(crate) fn build(
    sim: &mut SimWorld,
    db: &AssetDb,
    // ⭐⭐ `célula do atlas → AssetId`, preenchido pelo import / canvas novo / load — **nunca pelo
    // arranque**. É o que separa *«o artista trouxe isto»* de *«o boot pôs isto no `AssetDb`»*.
    atlas_assets: &BTreeMap<u32, AssetId>,
    // ⭐⭐ **A TAXONOMIA** (wave A3) — é ela que dá a cada entrada o catálogo a que pertence.
    //
    // ⛔⛔ **Sem isto o filtro da coluna não tinha efeito nenhum, e a suíte ficava verde**: a
    // árvore guardava a atribuição, o painel expandia o escopo certo, e as entradas do índice
    // chegavam todas com `catalog: None` — a consulta não casava nenhuma. *O fio estava completo
    // dos dois lados e não se tocava no meio.* Quem o achou foi o roteiro de ponteiro, ao contar
    // os cartões que a grade de facto desenha.
    catalogs: &ph2d_asset_index::CatalogTree,
    swatches: &mut CardArt,
    remembered: &mut TextureLibrary,
) -> AssetIndex {
    let mut index = AssetIndex::new();
    // O orçamento é POR QUADRO — ver [`THUMB_BUDGET_PX`].
    let mut budget = THUMB_BUDGET_PX;

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
        // ⭐⭐ **A cor e a miniatura de um Prefab saem da MESMA peça** (auditoria de 2026-08-30,
        // achado nº 2).
        //
        // ⛔ A 1.ª versão tirava a cor de `deps.first()` — o **menor digest blake3**, que não tem
        // relação nenhuma com o que o artista vê — e a miniatura da peça maior. Num prefab de duas
        // peças com texturas diferentes, o fundo que transparece pela alfa da miniatura era a cor
        // média de **outra** textura, e o doc do pintor afirmava que era a mesma.
        //
        // ⚠️ Uma receita sem pixels nenhuns fica com a cor neutra do construtor, e isso é honesto:
        // *ela não tem cor*.
        let face = largest_piece_texture(sim, &pieces);
        if let Some(id) = face
            && let Some(rgba) = swatch_for(db, id, swatches)
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
        entry.thumb = face.and_then(|id| thumb_for(db, id, swatches, &mut budget));
        entry.deps = deps;
        entry.catalog = catalogs.catalog_of(&entry.key);
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
    // ⇒ **Uma textura é um asset quando uma ENTIDADE a referencia.**
    //
    // ⛔⛔⛔ **E «referencia» tem DUAS formas, não uma — a 1.ª versão só conhecia a segunda, e o
    // resultado foi o report do Enio de 2026-08-30: *«as imagens não aparecem no painel nem
    // importando nem criando as imagens no app»*.**
    //
    // | forma | quem a tem | o carimbo |
    // |---|---|---|
    // | **atlas** | o caminho NORMAL de todo import e de todo canvas novo | `Sprite { source: Atlas { key } }` + o `atlas_asset_map[key]` |
    // | **individual** | só o que não coube no atlas (16 bits, sobredimensionado) ou foi promovido | `SpritePixels(AssetId)` |
    //
    // ⚠️ **O `SpritePixels` é o carimbo da MINORIA**, e o `spawn_sprite` di-lo por escrito: ele só o
    // insere no ramo `PackedSource::Individual`, porque *«uma sprite de atlas grava-se pelo
    // `atlas_asset_map`; um `SpritePixels` a mais fá-la-ia ser gravada duas vezes»*. Eu li aquele
    // carimbo como *«o artista trouxe isto»* quando ele significa *«esta sprite tem textura
    // própria»* — e o caso comum não tem.
    //
    // ⭐⭐ **E o `atlas_asset_map` é a resposta EXACTA à pergunta que a 1.ª cura fazia**, sem o
    // efeito colateral: ele é preenchido pelo **import**, pelo **canvas novo** e pelo **load do
    // projecto**, e ⛔ **nunca pelo arranque** — as 16 do átlas de demonstração entram no `AssetDb`
    // pelo `atlas_loader` e **não** neste mapa. *A cura anterior acertava no sintoma pelo
    // predicado errado; esta acerta pela proveniência, que é o que a pergunta sempre foi.*
    let mut loose: Vec<(u64, Entity, AssetId)> = {
        let mut q = sim
            .world_mut()
            .query::<(Entity, Option<&SpritePixels>, Option<&ph2d_render::Sprite>)>();
        let mut v: Vec<(u64, Entity, AssetId)> = q
            .iter(sim.world())
            .filter_map(|(e, px, spr)| {
                let id = texture_of(px, spr, atlas_assets)?;
                let order = sim.world().get::<StableId>(e).map_or(u64::MAX, |s| s.0);
                Some((order, e, id))
            })
            .collect();
        v.sort_unstable_by_key(|(order, _, _)| *order);
        v
    };
    loose.dedup_by_key(|(_, _, id)| *id);

    // ⭐⭐ **Quem o mundo usa AGORA** — a lista que faz uma lápide ceder sem ninguém editar o
    // documento (ver [`TextureLibrary::entries`]).
    let live: std::collections::BTreeSet<AssetId> = loose.iter().map(|(_, _, id)| *id).collect();

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
        remembered.remember(id, texture_entry(db, id, name, swatches, &mut budget));
    }

    // ⭐⭐ **E a BIBLIOTECA LEMBRA.** Report do Enio, 2026-08-30: *«ao deletar o objeto do canvas, o
    // do painel assets foi deletado»*.
    //
    // ⚠️ **Uma textura não é um objecto da cena — ela é CONTEÚDO** (bytes endereçados por blake3).
    // Derivá-la do mundo a cada quadro fazia da biblioteca um espelho da cena: apagar a sprite que
    // a usava apagava o asset. *Uma biblioteca que perde o que o artista trouxe não é uma
    // biblioteca.*
    //
    // ⛔ E a memória é **união, nunca subtracção AUTOMÁTICA**: o que entrou fica até alguém o mandar
    // sair. ⭐ A porta de o mandar sair é o [`forget_texture`], e ela nasceu do report seguinte do
    // Enio — *«uma sprite que foi deletada do canvas não consegui deletar do painel»*.
    for entry in remembered.entries(&live) {
        let mut e = entry.clone();
        // ⚠️ **O catálogo NÃO é lembrado com a entrada, e é de propósito:** a `TextureLibrary` é
        // memória de SESSÃO e a taxonomia é do PROJECTO. Guardá-lo no clone faria a atribuição
        // sobreviver a um `Open Project` que a não tem.
        e.catalog = catalogs.catalog_of(&e.key);
        index.push(e);
    }

    index
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
pub(crate) fn forget_texture(id: AssetId) {
    LIBRARY.with(|lib| lib.borrow_mut().forget(id));
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
pub(crate) fn forgotten_textures() -> Vec<[u8; 32]> {
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
pub(crate) fn set_forgotten_textures(ids: &[[u8; 32]]) {
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
    fn forget(&mut self, id: AssetId) {
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
    fn entries<'a>(
        &'a self,
        live: &'a std::collections::BTreeSet<AssetId>,
    ) -> impl Iterator<Item = &'a AssetEntry> {
        self.entries
            .iter()
            .filter(|(id, _)| !self.forgotten.contains(id) || live.contains(id))
            .map(|(_, e)| e)
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
        "1 piece".to_string()
    } else {
        format!("{n} pieces")
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

/// ⭐⭐⭐ **QUE TEXTURA esta entidade referencia** — e a resposta tem DUAS formas.
///
/// ⛔⛔ **É a porta que faltava, e a ausência dela custou o report do Enio de 2026-08-30**
/// (*«as imagens não aparecem no painel nem importando nem criando as imagens no app»*): eu
/// escrevi *«uma textura é um asset quando uma entidade a referencia»* e implementei só metade
/// disso — o `SpritePixels`, que é o carimbo da MINORIA.
///
/// | forma | quem a tem |
/// |---|---|
/// | `SpritePixels(id)` | só o que não coube no átlas (16 bits, sobredimensionado) ou foi promovido |
/// | célula de átlas + `atlas_asset_map` | o caminho NORMAL de todo import e de todo canvas novo |
///
/// ⚠️ **O `atlas_assets` é o que separa proveniência de presença:** ele é preenchido pelo import,
/// pelo canvas novo e pelo load do projecto, e ⛔ **nunca pelo arranque** — as 16 do átlas de
/// demonstração ficam de fora sem um caso especial.
///
/// ⚠️ **UMA porta, dois leitores** — o índice (*«que assets existem?»*) e o
/// [`crate::asset_card_verbs::users_of`] (*«quem usa isto?»*). A 1.ª versão respondia a segunda
/// pergunta com a metade errada, e o *Select users* numa imagem importada devolvia **zero**.
pub(crate) fn texture_of(
    pixels: Option<&SpritePixels>,
    sprite: Option<&ph2d_render::Sprite>,
    atlas_assets: &BTreeMap<u32, AssetId>,
) -> Option<AssetId> {
    if let Some(p) = pixels {
        return Some(p.0);
    }
    match sprite?.source {
        ph2d_render::SpriteSource::Atlas { key } => atlas_assets.get(&key).copied(),
        _ => None,
    }
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
fn largest_piece_texture(sim: &SimWorld, pieces: &[Entity]) -> Option<AssetId> {
    // `(área em micro-unidades, id de desempate, textura)`. ⚠️ O cast `f64 -> u64` **satura** em
    // Rust, então duas peças acima de ~1,8e13 empatam no topo — e o desempate resolve-o. E
    // `f64::max(NaN, 0.0) == 0.0`, então uma `size` inválida pontua zero em vez de estourar.
    let mut best: Option<(u64, u64, AssetId)> = None;
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

#[cfg(test)]
#[path = "asset_index_build_tests.rs"]
mod tests;

/// Os gates da LÁPIDE — irmão pelo tecto de LOC (HR-18); o corte é por responsabilidade.
#[cfg(test)]
#[path = "asset_index_build_library_tests.rs"]
mod library_tests;
