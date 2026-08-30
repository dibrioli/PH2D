//! **O ASSADO dos padrões de textura da cena** (plano 33, W4) — memoizado, irmão do
//! [`crate::fx_live`].
//!
//! ⚠️⚠️ **O NOME É `texture_pattern_live`, e o `pattern_live` ao lado é OUTRA COISA.** Aquele é o
//! *Pattern Along Path* ([plano 23](../../../docs/Vector%20Module/23_plano_pattern_along_path.md)):
//! um MOTIVO copiado ao longo de uma guia, com alças e picker. Este é a TINTA de uma forma. A
//! primeira redacção desta wave chamou-se `pattern_live` e o módulo passou a estar **declarado
//! duas vezes** — *o nome de um módulo carrega um contrato, e este já tinha dono*.
//!
//! O documento guarda a RELAÇÃO (qual arte, que reticulado, que tamanho, onde); isto é o desenho
//! derivado dela, e vive só em runtime.
//!
//! # Porque há um memo, e o que está na chave
//!
//! ⚠️ **Não é o quadro que custa — é o ASSADO.** Desenhar um padrão é uma `fill()` (a repetição é
//! do amostrador do Vello); assar é compor a arte num reticulado. Medido na W1: `1,047 ms` para um
//! ladrilho de `536x1072` em colmeia. Fazê-lo por quadro seria 6% de um quadro de 60 fps por forma
//! com padrão, por nada.
//!
//! A chave é **`(fonte, lei assada, dimensões da arte)`** — exactamente o que muda os pixels do
//! ladrilho, e nada mais. ⛔ **A `quality` NÃO entra nela**, e a ausência é a decisão: ela escolhe o
//! filtro de amostragem na GPU e não toca um byte do assado, então metê-la na chave faria alternar
//! o modo de imagem do projecto re-assar toda a cena para produzir os MESMOS pixels. Ela é
//! actualizada em cada quadro sobre a entrada que já existe.
//!
//! ⚠️ E o [`StableImage`] é construído **UMA vez** e clonado por quadro, pela razão que o
//! `FxImage` já documenta: o Vello indexa o cache de imagem pelo id do `Blob`, então um handle novo
//! por quadro faz a textura ser **re-enviada ao atlas** todo quadro.

use ph2d_asset::AssetDb;
use ph2d_vec_render::{PatternSlot, PatternTile, PatternTiles};
use ph2d_vec_scene::{Paint, PatternSource, VecPath, VecPathId, VecScene, Xform};
use ph2d_vector::{ImageQuality, StableImage};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Com o que estes pixels foram assados. Ver o cabeçalho para o que NÃO está aqui.
///
/// ⚠️⚠️ **Ela é feita SEM tocar na arte, e isso é o assunto.** A 1.ª versão punha aqui as dimensões
/// em pixels e a lei já convertida — e para as saber tinha de **resolver a arte primeiro**, o que
/// com uma fonte-FORMA significa um render + readback de GPU **por quadro**. Um gate apanhou-o
/// (`editing_the_source_shape_rebakes_the_tile`, a metade do *"não re-assou o que não mudou"*).
///
/// ⇒ a chave guarda o que o **artista autorou** mais a **identidade/conteúdo da fonte**, e as
/// dimensões em pixels são derivadas disso. Uma imagem é endereçada por conteúdo (HR-6), então o
/// mesmo `AssetId` são os mesmos pixels; uma forma entra pelo conteúdo dela.
#[derive(Clone, PartialEq)]
struct Key {
    source: PatternSource,
    kind: ph2d_vec_pattern::TileKind,
    offset_denom: u8,
    /// ⚠️ O `size` entra porque o vão em PIXELS deriva de `gap/size` — mexer no tamanho muda o
    /// assado. Já o `origin` e o `angle` são COLOCAÇÃO: eles não tocam um byte do ladrilho, e
    /// metê-los aqui faria arrastar a alça de mover re-assar a cena.
    size: [f64; 2],
    gap: [f64; 2],
    /// ⭐⭐ **A FORMA-FONTE, quando a arte vem do documento** (W7) — e é ela que torna o padrão
    /// **VIVO**: editar a forma re-assa o ladrilho em toda forma que a usa, que é o *"pattern fills
    /// are dynamic"* do Figma.
    ///
    /// ⚠️ Sem ela na chave, a `PatternSource::Shape(id)` seria estável e mexer nos nós da fonte não
    /// mudaria a tela — o defeito EXACTO que o `FxKey` da crate irmã documenta (*"a chave era
    /// `(pilha, w, h)` e a forma não entrava nela, então mudar a cor do fill de uma forma filtrada
    /// não mudava a tela"*).
    shape: Vec<VecPath>,
    /// ⭐⭐⭐ **A POSE DOS MEMBROS, e ela NÃO está no [`VecPath`]** (report do Enio, 2026-08-30:
    /// *"ao mover os objetos do grupo que serve como shape, a pattern não atualiza em tempo real"*).
    ///
    /// Desde o [ADR-0110](../../../docs/architecture/decisions/0110-vector-nodes-are-ecs-entities-one-hierarchy.md)
    /// a geometria guardada num `VecPath` é **local**, e quem a põe no mundo é o [`Xform`] que a
    /// shell publica por quadro. ⇒ uma chave feita só de `VecPath`s é **cega ao gesto de mover**: o
    /// membro anda na tela e o ladrilho fica com o desenho de antes, **para sempre** — a chave nunca
    /// mais difere. *Um memo cuja chave não contém tudo o que o assado lê não é um memo: é um
    /// congelamento.*
    ///
    /// ⚠️ **NORMALIZADA pela translação COMUM**, e isso é a lei do assado, não uma optimização: o
    /// [`crate::motion_object_bake::bake_rgba_many`] põe a caixa da UNIÃO na origem do ladrilho, e
    /// por isso arrastar o conjunto inteiro devolve o mesmo desenho — o que muda o ladrilho é um
    /// membro mexer-se **em relação aos outros**. ⛔ Sem a normalização, arrastar o grupo re-assaria
    /// (render + readback de GPU) a **cada quadro**, que é exactamente a razão pela qual o `origin`
    /// e o `angle` também ficam de fora desta chave.
    pose: Vec<[f64; 6]>,
}

/// A ARTE que uma fonte nomeia, no que ela tem de identidade — a metade PURA da resolução.
///
/// ⚠️ Extraída da que assa porque assar precisa de GPU e **isto não**: é aqui que vive a recusa do
/// ciclo, e ela tem gate.
#[must_use]
fn source_shape(
    scene: &VecScene,
    host: VecPathId,
    source: &PatternSource,
    object_of: &dyn Fn(VecPathId) -> Vec<VecPathId>,
) -> Vec<VecPath> {
    let PatternSource::Shape(id) = source else {
        return Vec::new();
    };
    // ⭐⭐⭐ **UM GRUPO PODE SER A ARTE** (Enio, 2026-08-30: *"assim o grupo poderia ser usado como
    // pattern"*).
    //
    // O documento endereça a arte por um `VecPathId` — e um grupo **não tem um**: ele nasce
    // `(Transform, Name, RootOrder)` e mais nada. ⇒ o id continua a ser o de um CAMINHO, e o que
    // muda é a **resolução**: ele passa a nomear o OBJECTO a que aquele caminho pertence, que é
    // exactamente a lei de selecção que o app já tem (*"um grupo entra e sai da selecção INTEIRO"*).
    //
    // ⭐ E é por isso que **o schema não se mexe**: nenhuma variante nova, nenhum id de entidade
    // gravado (que o undo respawna com bits novos), nenhum degrau de migração.
    let membros = object_of(*id);
    // ⛔⛔ **A RECUSA DO CICLO passou a ser sobre PERTENÇA, não sobre igualdade.** Antes bastava
    // `id == host`; com um grupo, o anfitrião pode ser **um membro** da arte — e aí assá-la exigiria
    // desenhá-lo, desenhá-lo exigiria o ladrilho, e o ladrilho exigiria assá-la. ⚠️ O sintoma não
    // seria um erro: seria o app a parar.
    if membros.contains(&host) {
        return Vec::new();
    }
    // ⚠️ **A ORDEM é a do documento** (o `object_of` devolve-a por z), e ela vira a ordem de
    // desenho do assado: uma travessia em profundidade poria o membro errado por cima.
    membros
        .iter()
        .filter_map(|m| scene.paths().iter().find(|p| p.id == *m).cloned())
        .collect()
}

/// **ONDE cada membro da arte se senta**, na forma em que isso é identidade do ladrilho.
///
/// A metade que falta à [`source_shape`]: ela responde *quais* caminhos, esta responde *onde*. As
/// duas juntas são a arte de um grupo, e é isso que a [`Key`] guarda — ver o campo [`Key::pose`]
/// para o mecanismo e para o porquê da normalização.
///
/// ⚠️ **A âncora é o PRIMEIRO membro, e a ordem dele é a do documento** (a [`source_shape`] devolve
/// por z). Uma âncora tirada do centroide mudaria ao entrar ou sair um membro, e faria toda a lista
/// diferir de uma vez por uma coisa que não é sobre pose nenhuma.
///
/// ⚠️ Só a **translação** se normaliza. A parte linear (`a,b,c,d`) entra crua de propósito: rodar ou
/// escalar o grupo **muda** o desenho do ladrilho, e tem de re-assar.
#[must_use]
fn art_pose(art: &[VecPath], pose_of: &dyn Fn(VecPathId) -> Xform) -> Vec<[f64; 6]> {
    let ancora = art.first().map_or([0.0, 0.0], |p| {
        let Xform([_, _, _, _, e, f]) = pose_of(p.id);
        [e, f]
    });
    art.iter()
        .map(|p| {
            let Xform([a, b, c, d, e, f]) = pose_of(p.id);
            [a, b, c, d, e - ancora[0], f - ancora[1]]
        })
        .collect()
}

/// ⭐⭐⭐ **A FORMA que este padrão nomeia como arte deixou de existir?** (plano 33, W11.)
///
/// ⚠️ **Pergunta pela MESMA porta que assa** ([`source_shape`]), e não por um `scene.path(id)`
/// escrito à mão ao lado. Duas respostas à mesma pergunta divergem no primeiro ajuste — e esta já
/// divergiria hoje: a porta recusa também a **auto-referência** (`id == host`), que é igualmente um
/// padrão sem arte utilizável, e que uma consulta directa daria como presente.
///
/// ⚠️ **Só a fonte-FORMA responde `true`.** Uma fonte-IMAGEM que não resolve pode estar apenas a
/// carregar — os pixels dela viajam no ficheiro desde a W8, e a ausência é transitória por
/// construção. *Um aviso permanente sobre um estado transitório ensina o artista a ignorar avisos.*
#[must_use]
pub(crate) fn art_is_missing(
    scene: &VecScene,
    host: VecPathId,
    source: &PatternSource,
    object_of: &dyn Fn(VecPathId) -> Vec<VecPathId>,
) -> bool {
    matches!(source, PatternSource::Shape(_))
        && source_shape(scene, host, source, object_of).is_empty()
}

/// Os ladrilhos de padrão da cena, assados e memoizados.
#[derive(Default)]
pub(crate) struct TexturePatternLive {
    tiles: PatternTiles,
    keys: BTreeMap<(VecPathId, PatternSlot), Key>,
}

impl TexturePatternLive {
    /// Os ladrilhos deste quadro — o que o [`ph2d_vec_render::dispatch`] injecta no z das formas.
    ///
    /// Vazio = nenhum padrão resolvido, e toda forma com `Paint::Pattern` pinta a `fallback` dela.
    pub(crate) fn tiles(&self) -> &PatternTiles {
        &self.tiles
    }

    /// Re-assa o que mudou. Uma passagem pela cena; as formas sem padrão não pagam nada.
    ///
    /// ⭐⭐ **O assador de FORMA vem INJECTADO** (`bake_shape`), e não cablado.
    ///
    /// Assar uma forma em pixels é render + readback — GPU. Cablá-lo aqui poria uma `GpuContext` na
    /// assinatura e tornaria **todo** gate deste memo dependente de uma placa; com a injecção, o
    /// quadro passa a porta única [`crate::motion_object_bake::bake_rgba`] e os gates passam um
    /// bitmap sintético. *Um memo que só se pode medir com GPU é um memo que não se mede.*
    pub(crate) fn recook(
        &mut self,
        scene: &VecScene,
        assets: &AssetDb,
        quality: ImageQuality,
        bake_shape: &mut dyn FnMut(VecPathId) -> Option<(u32, u32, Vec<u8>)>,
        object_of: &dyn Fn(VecPathId) -> Vec<VecPathId>,
        pose_of: &dyn Fn(VecPathId) -> Xform,
    ) {
        let mut seen = BTreeSet::new();
        for path in scene.paths() {
            // ⭐⭐ **AS DUAS TINTAS de uma forma** (plano 35, wave C): o preenchimento e o traço
            // podem ter padrões INDEPENDENTES, então cada um é uma entrada própria no memo.
            // ⚠️ Uma chave só pela forma entregaria o ladrilho do preenchimento ao traço, e o
            // desenho ficaria certo **por acidente** enquanto os dois fossem iguais.
            let da_forma = match path.fill.as_ref() {
                Some(Paint::Pattern(pat)) => Some((PatternSlot::Fill, pat.as_ref())),
                _ => None,
            };
            let do_traco = path
                .stroke
                .as_ref()
                .and_then(ph2d_vec_scene::StrokeSpec::pattern)
                .map(|pat| (PatternSlot::Stroke, pat));
            for (slot, pat) in da_forma.into_iter().chain(do_traco) {
                self.bake_one(
                    scene, assets, quality, bake_shape, object_of, pose_of, &mut seen, path.id,
                    slot, pat,
                );
            }
        }
        // ⚠️ **A varredura tem as DUAS metades.** Marcar sem desmarcar deixaria o ladrilho de uma
        // tinta que deixou de ter padrão (ou de uma forma apagada) a ser desenhado para sempre, e a
        // memória dele viva — é a mesma lei das duas metades do passe do `MasterPiece`.
        self.tiles.retain(|k, _| seen.contains(k));
        self.keys.retain(|k, _| seen.contains(k));
    }

    /// Assa (ou reaproveita) o ladrilho de **uma** tinta. Extraída do [`Self::recook`] quando o
    /// traço passou a poder ter padrão — o corpo é o mesmo, o sujeito é que passou a ser dois.
    #[allow(clippy::too_many_arguments)] // um facto por argumento; agrupá-los esconderia o slot
    fn bake_one(
        &mut self,
        scene: &VecScene,
        assets: &AssetDb,
        quality: ImageQuality,
        bake_shape: &mut dyn FnMut(VecPathId) -> Option<(u32, u32, Vec<u8>)>,
        object_of: &dyn Fn(VecPathId) -> Vec<VecPathId>,
        pose_of: &dyn Fn(VecPathId) -> Xform,
        seen: &mut BTreeSet<(VecPathId, PatternSlot)>,
        id: VecPathId,
        slot: PatternSlot,
        pat: &ph2d_vec_scene::PatternFill,
    ) {
        let shape = source_shape(scene, id, &pat.source, object_of);
        // ⚠️ Uma fonte-FORMA que não resolve (inexistente, ou a própria forma) nunca chega ao
        // assador: a recusa é PURA e mora na `source_shape`.
        if matches!(pat.source, PatternSource::Shape(_)) && shape.is_empty() {
            return;
        }
        let key = Key {
            source: pat.source,
            kind: pat.kind,
            offset_denom: pat.offset_denom,
            size: pat.size,
            gap: pat.gap,
            pose: art_pose(&shape, pose_of),
            shape,
        };
        let slot_key = (id, slot);
        seen.insert(slot_key);
        if self.keys.get(&slot_key) == Some(&key) {
            // ⭐ Só o filtro se actualiza: ele não muda um byte do assado. E — mais importante —
            // **não se resolve a arte**: com uma fonte-FORMA isso seria um readback de GPU por
            // quadro.
            if let Some(t) = self.tiles.get_mut(&slot_key) {
                t.quality = quality;
            }
            return;
        }
        let Some((aw, ah, px)) = art_of(&pat.source, assets, !key.shape.is_empty(), bake_shape)
        else {
            // A arte ainda não carregou: a entrada fica de fora e a tinta pinta a `fallback` —
            // desenho certo, não desistência. ⚠️ E a chave NÃO se grava, senão o próximo quadro
            // acharia que já estava assado.
            self.tiles.remove(&slot_key);
            self.keys.remove(&slot_key);
            return;
        };
        let law = pat.law([aw, ah]);
        let tile = match ph2d_vec_pattern::bake(&px, aw, ah, &law) {
            Ok(t) => t,
            Err(e) => {
                eprintln!(
                    "[pattern] o assado da forma {id} ({slot:?}) recusou ({e:?}) - ela vai pintar a \
                     cor de recurso"
                );
                self.tiles.remove(&slot_key);
                self.keys.remove(&slot_key);
                return;
            }
        };
        // ⭐⭐ **O salto na volta mede-se AQUI, uma vez** (plano 33 W10) — no assado, antes de os
        // bytes irem para o `StableImage`. É o único sítio do quadro em que eles existem em CPU, e
        // é memoizado com o resto: recalcular por quadro seria varrer o perímetro de todo ladrilho
        // da cena para responder sempre a mesma coisa.
        let wrap_seam = ph2d_vec_pattern::wrap_seam(&tile);
        let Some(image) = StableImage::from_rgba(Arc::new(tile.rgba), tile.width, tile.height)
        else {
            self.tiles.remove(&slot_key);
            self.keys.remove(&slot_key);
            return;
        };
        self.tiles.insert(
            slot_key,
            PatternTile {
                image,
                cells: tile.cells,
                tile_px: [tile.width, tile.height],
                quality,
                wrap_seam,
            },
        );
        self.keys.insert(slot_key, key);
    }
}

/// Os pixels da ARTE de um padrão, RGBA reto.
///
/// ⚠️ Passa pela porta [`ph2d_asset::Asset::image_rgba8`] em vez de casar com `ImageRgba8`: ela
/// converte o caso de 16 bits, e um `match` directo aceitaria a variante de 16 bits **em silêncio**
/// pelo braço `_` (o `Asset` é `#[non_exhaustive]`, e o doc dele avisa exactamente disto).
fn art_of(
    source: &PatternSource,
    assets: &AssetDb,
    shape_ok: bool,
    bake_shape: &mut dyn FnMut(VecPathId) -> Option<(u32, u32, Vec<u8>)>,
) -> Option<(u32, u32, Vec<u8>)> {
    match source {
        PatternSource::Image(id) => {
            let asset = assets.get(id)?;
            let (w, h, px) = asset.image_rgba8()?;
            Some((w, h, px.into_owned()))
        }
        // ⭐⭐ **W7 — uma FORMA do documento como arte** (o modelo do Figma). O `shape_ok` é o
        // veredito da [`source_shape`]: ela é que diz se a fonte existe e **não é a própria forma**,
        // e é lá que a recusa do ciclo vive — pura, e com gate.
        PatternSource::Shape(id) => {
            if !shape_ok {
                return None;
            }
            bake_shape(*id)
        }
    }
}

#[cfg(test)]
#[path = "texture_pattern_live_tests.rs"]
mod tests;
// ⚠️ **Irmão por RESPONSABILIDADE, não por tamanho** — o de cima mede o MEMO, este mede o que é
// próprio de a arte vir do DOCUMENTO (ciclo, re-assado ao editar, pose dos membros de um grupo). O
// corte foi imposto pelo tecto de LOC, mas a linha dele é o assunto: um gate novo sabe onde nasce.
#[cfg(test)]
#[path = "texture_pattern_live_shape_tests.rs"]
mod shape_tests;

/// ⭐ **Os gates do vínculo MORTO** (plano 33, W11), num irmão — o corte é por responsabilidade:
/// o [`tests`] mede o memo do assado, este mede o que acontece quando a arte deixa de existir.
#[cfg(test)]
#[path = "texture_pattern_art_missing_tests.rs"]
mod art_missing_tests;
