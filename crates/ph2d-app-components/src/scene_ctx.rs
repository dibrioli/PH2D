//! **O que uma CENA desta família precisa da shell** — escrito em tipos, uma vez.
//!
//! # Por que ele existe, e por que não é um sexto método do `AppHost`
//!
//! As treze cenas de smoke desta família viviam em `impl crate::App`, e a régua do fecho diz o que
//! elas de facto tocam: `.gfx` (24 acessos), `.vec_entities` (10), `.playhead`, `.audio` e os
//! latches de arme. ⛔ **Nada disso é uma pergunta nova à shell** — é o mundo, a cena vectorial, o
//! registo de componentes e o relógio, que são **tipos de crates irmãs**. Escrito em tipos, o
//! pedido cabe numa struct de empréstimos e o trait de host fica onde está.
//!
//! *É a regra 4 da Fase D à letra: **antes de pedir porta, escreva em TIPOS o que a função
//! precisa.*** O molde é o `MotionSceneCtx` da `line/app-motion`, que tirou `crate::App` de 44
//! cenas com zero portas novas.
//!
//! # ⚠️ O que NÃO está aqui: o prólogo
//!
//! Os guardas de arme — *«já corri?»*, *«a env está posta?»*, *«o mundo já subiu?»* — **ficam na
//! shell**, no invólucro que constrói este contexto. É a lei que a `line/app-physics` pagou na
//! Fase C: *o que sai são os CORPOS; o que decide a ordem do quadro fica.* Um latch aqui dentro
//! seria esta crate a ter opinião sobre **quando** o quadro a chama.

use ph2d_ecs::{SimWorld, scene::ComponentRegistry};

/// Os empréstimos que uma cena desta família recebe do quadro.
///
/// ⚠️ **Os quatro primeiros vêm todos do `AppGfx`** e são campos disjuntos dele, e é por isso que
/// o invólucro os pode emprestar ao mesmo tempo. O `vec_entities` e o `playhead` são da `App`.
pub struct SceneCtx<'a> {
    /// O mundo ECS que as cenas povoam.
    pub sim: &'a mut SimWorld,
    /// A cena vectorial — as cópias com geometria própria escrevem-lhe.
    pub vec_scene: &'a mut ph2d_vec_scene::VecScene,
    /// O mapa documento-vectorial ⟷ entidade (folha partilhada desde 12/09).
    pub vec_entities: &'a mut ph2d_vec_entities::entities::VecEntityMap,
    /// ⚠️ **Só LEITURA:** uma cena povoa o mundo, nunca regista um tipo de componente.
    pub registry: &'a ComponentRegistry,
    /// ⭐⭐⭐ **A ÁRVORE DE TAGS do projecto** (TOP-20 #9, W4b) — a taxonomia que as cenas de tag
    /// escrevem antes de marcar os objectos.
    ///
    /// ⚠️ **`&mut`, e é a única coisa aqui que não é o mundo:** ela vive ao lado dele no `AppGfx`
    /// (é DOCUMENTO, como a `VecScene`), e uma cena que marque um objecto sem criar a tag deixaria
    /// ids órfãos — o painel abriria vazio sobre objectos marcados.
    pub tags: &'a mut ph2d_tags::TagTree,
    /// O ecrã, para as cenas que deixam uma peça **escolhida** — sem isso a receita que elas abrem
    /// não aparece na Hierarquia, porque a marca `MasterEditing` é derivada da selecção.
    ///
    /// ⚠️ `Option` porque no arranque ele ainda não existe, e uma cena que precise dele di-lo com
    /// um `if let` em vez de entrar em pânico.
    pub hero_screen: Option<&'a mut ph2d_editor_core::HeroScreen>,
    /// O relógio, que as cenas de instância rebobinam e põem a andar.
    ///
    /// ⚠️ **`&mut`, e de propósito:** *uma cena de smoke não é um observador — ela ENCENA, e
    /// encenar inclui carregar no play.* Sem isto os três pêndulos da `=1` ficam pendurados no ar
    /// e o smoke lê-se como *«a física está morta»*.
    pub playhead: &'a mut ph2d_core::Playhead,
    /// **A máquina abriu a saída de som?** — a cena do áudio 2D só o usa para dizer ao dono qual
    /// dos dois mundos ele está a ver.
    ///
    /// ⛔ Um `&AudioSystem` aqui seria a família a poder tocar som a partir de uma cena; o que ela
    /// precisa é de **um facto**, e o facto é um `bool`.
    pub audio_ready: bool,
    /// **A pré-visualização pela câmera da cena**, que a cena da câmera LIGA.
    ///
    /// ⚠️ Ela fica na `App` e não aqui dentro porque o `render_loop` a lê para decidir a vista do
    /// quadro — é estado da SHELL que esta cena escreve, e não estado da família.
    pub camera_preview: &'a mut bool,
}
