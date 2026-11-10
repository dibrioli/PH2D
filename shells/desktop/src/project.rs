//! Save/load de PROJETO em disco (Ctrl+S / Ctrl+O globais).
//!
//! O projeto é a MESMA captura do undo — `ProjectState = {WorldSnapshot + VecScene}`
//! — mais os **bytes das imagens** dos sprites (`SavedAsset`), que o undo não guarda
//! (são estáveis, não mudam a cada ação). Um arquivo é `(PROJECT_SCHEMA, ProjectFile)`
//! em postcard.
//!
//! Fase 2a (esta): estado + geometria. Formas vetoriais voltam 100%; sprites voltam
//! com pose/estrutura, e a imagem se o `AssetDb` ainda a tiver (mesma sessão).
//! Fase 2b: `collect_assets`/`materialize_assets` embutem e re-materializam os pixels,
//! fechando o cross-sessão.

use crate::project_schema::PROJECT_SCHEMA;
use crate::undo::{ProjectState, ProjectUndo};

/// O conteúdo de um arquivo de projeto.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct ProjectFile {
    /// Mundo (ECS) + geometria vetorial — a unidade do undo.
    pub(crate) state: ProjectState,
    /// Pixels dos sprites, para re-materializar o atlas noutra sessão (Fase 2b).
    /// Vazio na Fase 2a.
    pub(crate) assets: Vec<SavedAsset>,
    /// Os **documentos do Painter** (camadas + pixels + relevo), por identidade estável
    /// (`ph2d_ecs::PaintedDoc`). Vazio quando nada foi pintado. Ver [`crate::project_painter`].
    pub(crate) painted: Vec<ph2d_tool_painter::PaintedDocument>,
    /// O documento de **Motion Nodes**, na forma textual canônica do `ph2d-motion-doc`
    /// (linha-a-linha, com `[layout]` e `[backdrop]` — ADR-0032 §6).
    ///
    /// Campo do ARQUIVO, deliberadamente **fora do `ProjectState`**: o `ProjectState` é a
    /// unidade do undo GLOBAL, e o Motion tem undo próprio (`MotionHistory`) — o Enio já
    /// separou os dois escopos. Enfiar o grafo ali dentro faria cada Ctrl+Z do canvas
    /// rebobinar o grafo junto, e vice-versa.
    ///
    /// É **texto**, não postcard, porque esse já é o formato canônico do documento: é
    /// diffável e mergeável por linha (o requisito multiagente que descartou JSON/RON).
    /// Um projeto sem grafo carrega `""`.
    pub(crate) motion: String,
    /// O **`TimelineDoc`** (clips, faixas, tracks, keys) em postcard — a animação inteira.
    ///
    /// Fora do `ProjectState` pelo mesmo motivo do `motion`: o `ProjectState` é a unidade do
    /// undo GLOBAL, e a timeline tem undo próprio. Enfiá-la ali faria cada Ctrl+Z do canvas
    /// rebobinar a animação junto.
    ///
    /// As bindings viajam com o **`wire_id`** (hash do `Name` do objeto) carimbado no save, e
    /// NÃO com os bits de entidade — que o load recicla. Quem as recola é o `upkeep` do frame,
    /// a mesma função que cura delete+undo (ver [`ph2d_timeline_persist`]). Um projeto sem
    /// animação carrega `vec![]`.
    pub(crate) timeline: Vec<u8>,
    /// As **settings de MUNDO** da física (ADR-0131 D8 / W2b).
    ///
    /// Fora do `ProjectState` de propósito: o `ProjectState` é a unidade do undo
    /// GLOBAL, e um Ctrl+Z do canvas não deve rebobinar a gravidade da cena —
    /// o mesmo motivo que mantém `motion` e `timeline` aqui fora.
    ///
    /// O mundo rapier em si **não** viaja (D2: ele é derivado); o que viaja é o
    /// que o artista autorou.
    pub(crate) physics: ph2d_physics_ecs::PhysicsSettings,
    /// **A tabela de COR autorada pelo artista** (plano UI/UX W6, degrau 1).
    ///
    /// ⚠️ **Esparsa e FORA do `ProjectState`**, pelas duas razões de sempre: só o que difere da
    /// fábrica viaja (um projeto que nunca abriu o painel guarda um vetor vazio), e um Ctrl+Z do
    /// canvas não deve rebobinar a cara do editor — o mesmo motivo que mantém `physics`,
    /// `motion` e `timeline` aqui fora.
    ///
    /// ⚠️ O que viaja é o par `(modo, chave-do-token)` e a cor. A **CHAVE**, nunca o índice do
    /// variant: guardar o índice amarraria todo projeto salvo à ORDEM da lista, e acrescentar um
    /// token no meio da tabela re-pintaria o app com as cores trocadas. É a mesma lei do `W4a`.
    pub(crate) tokens: Vec<crate::project_tokens::SavedToken>,
    /// **AS SETTINGS DO PROJETO** (doc 88, D3) — a escala do mundo
    /// (`pixels_per_meter`), a unidade que o artista LÊ (`display_unit`), os dois
    /// snaps do gizmo e o modo de filtragem.
    ///
    /// Fora do `ProjectState` pelo mesmo motivo de `physics`/`motion`/`timeline`: o
    /// `ProjectState` é a unidade do undo GLOBAL, e um Ctrl+Z do canvas não deve
    /// rebobinar a escala do mundo.
    ///
    /// ⚠️ Tipo PRÓPRIO do arquivo, e não o `ProjectSettings` de runtime — a mesma
    /// razão do `tokens` logo acima (a `ph2d-editor-core` não fala serde, e herdar o
    /// layout de um tipo de runtime torna um refactor interno numa quebra de save).
    /// Ver [`crate::project_settings`].
    pub(crate) settings: crate::project_settings::SavedSettings,
    /// **A CORRIDA GRAVADA** (ADR-0131 W17) — o que o dedo do jogador fez, tique
    /// a tique, na forma de arquivo da fita (`ph2d_physics_ecs::TapeWire`).
    ///
    /// ⚠️ **Ela é AUTORIA, e é o bake da W16 que o prova:** a fita é a entrada que
    /// o bake replaya para escrever as curvas, então perdê-la ao fechar o app é
    /// perder a corrida que o artista jogou — reabrir e apertar Bake devolvê-la é
    /// a razão inteira deste campo.
    ///
    /// ⚠️ **Fora do `ProjectState`**, pelo mesmo motivo de `motion`/`timeline`/
    /// `physics`: aquele é a unidade do undo GLOBAL, e um Ctrl+Z do canvas não
    /// deve rebobinar a gravação.
    ///
    /// Vazia num projeto onde ninguém correu — e ⚠️ **é a correção da W17 que
    /// torna essa frase verdadeira**: antes dela a fita gravava todo tique que o
    /// relógio andasse, então TODO projeto do app carregaria uma corrida de
    /// ninguém. Ver `render_loop::physics_bridge::dispatch`.
    pub(crate) player_tape: ph2d_physics_ecs::TapeWire,
    /// **OS PIXELS PRÓPRIOS** (plano [`docs/Sprite_projeto/17`] §3) — os bytes de todo sprite
    /// `SpriteSource::Individual`, nomeados pelo `ph2d_ecs::SpritePixels` que ele carrega.
    /// Ver [`crate::project_sprite_pixels`].
    ///
    /// ⚠️ **Ele fecha uma perda de dados que já acontecia**, e não é uma capacidade nova: o
    /// `texture_id` do `Individual` é um id de alocação da GPU, e o store recomeça em `1` a cada
    /// processo — um sprite tocado por qualquer ferramenta de imagem reabria **invisível**, ou a
    /// exibir os pixels de outro sprite. O `painted` (v3) resolveu isto para o produtor rico (e o
    /// `baked_forms` para o outro, até o 3D sair — ADR-0179); este campo é o chão que faltava
    /// debaixo deles, e cobre o funil que todas as ferramentas atravessam
    /// (`commit_edited_texture`).
    ///
    /// ⚠️ **`Vec<u8>` opaco, e carrega a própria versão lá dentro** (`SHEET_DOC_VERSION`) — o
    /// precedente literal do `timeline`. É o que faz as REGIÕES do hand-packed,
    /// que entram neste mesmo documento, não voltarem a bumpar o `PROJECT_SCHEMA`.
    ///
    /// Vazio num projeto sem sprites individuais.
    pub(crate) sprite_pixels: Vec<u8>,
    /// **O contador de [`ph2d_ecs::StableId`]** — o próximo id livre do documento
    /// (ADR-0164 F1).
    ///
    /// ⚠️ **Campo do ARQUIVO, deliberadamente FORA do `ProjectState`** — e não pela razão
    /// dos irmãos acima (o escopo do undo), mas por uma mais dura: um undo que o
    /// rebobinasse faria um **redo** entregar um id que ainda está vivo na pilha, e duas
    /// entidades com o mesmo `StableId` corrompem tudo o que aponta para um objeto (uma
    /// junta, um binding, um override). Ele só sobe.
    pub(crate) stable_id_counter: u64,
    // ⚠️ O INPUT MAP viaja no `state` desde a v232 (undo e ficheiro, um só dono); o `game_id` é o
    // último campo.
    /// **A ARTE DOS PADRÕES de textura** (plano 33, W4) — os pixels que cada `Paint::Pattern` da
    /// cena vectorial nomeia por `AssetId`.
    ///
    /// ⚠️ Sem ele, reabrir o projecto encontraria o `AssetDb` vazio, a fonte não resolveria, e toda
    /// forma com padrão pintaria a cor de recurso **para sempre e sem erro nenhum a que agarrar** —
    /// literalmente o defeito que o `sprite_pixels` acima curou para as sprites.
    ///
    /// ⚠️ **`Vec<u8>` opaco, e carrega a própria versão lá dentro** (`PATTERN_ART_DOC_VERSION`) — o
    /// precedente do `timeline` e do `sprite_pixels`. É o que faz um campo novo lá
    /// dentro não voltar a bumpar o `PROJECT_SCHEMA`.
    ///
    /// ⚠️⚠️ **A POSIÇÃO É O FORMATO, e este campo é o ÚLTIMO** — o postcard é posicional. Ver o
    /// aviso do `stable_id_counter`: trocar dois campos de ordem não dá erro nenhum, dá dois campos a
    /// ler os bytes um do outro.
    ///
    /// Vazio num projecto sem padrão nenhum. Ver [`crate::project_texture_pattern`].
    pub(crate) pattern_art: Vec<u8>,
    /// **Os QUADROS** (MiroClone, v185) — as abas de documento, cada uma com o seu conteúdo e vista
    /// (`ph2d_board_model::BoardSet::to_bytes`). `Vec<u8>` opaco com versão PRÓPRIA lá dentro, o
    /// precedente do `timeline`. ⚠️ A posição é o formato.
    pub(crate) boards: Vec<u8>,
    /// **Os SCRIPTS do projeto** (v190, ADR-0188 §2) — o blob `PH2DSCRIPTS` da `ScriptLibrary`
    /// (`ph2d_app_components::script_project`), com versão PRÓPRIA. Fora do `ProjectState`: o texto
    /// tem histórico próprio. Os braços `189`/`190` do load leem-no acrescentando o `0` deste `Vec` vazio.
    pub(crate) scripts: Vec<u8>,
    /// **As MARCAS DO ARTISTA** (v193, W9 das Macros) — por imagem com macro, onde o artista escreveu
    /// depois da gravação (`ph2d_app_painter::macro_track::PalcosDoPainter::marcas_a_gravar`), com versão
    /// PRÓPRIA. Fora do `ProjectState`: a marca anda com o desfazer de quem escreve os píxeis. Sem ela,
    /// abrir o projeto tirava do Play tudo o que se pintou depois de gravar. Os braços `189`..`192` do
    /// load leem-no acrescentando o `0` deste `Vec` vazio.
    pub(crate) marcas_do_artista: Vec<u8>,
    /// **O FORMATO DO JOGO** (v198, plano GEF 17 D-F/D-V) — a resolução que o jogo mostra e quando a moldura da
    /// câmera se vê ([`crate::project_settings::collect_jogo`]), com versão PRÓPRIA. Fora do `ProjectState` como o
    /// `settings`. ⚠️ Os braços `189`..`197` do load leem-no acrescentando o `0` deste `Vec`
    /// vazio (= o formato de fábrica).
    pub(crate) jogo: Vec<u8>,
    /// **A MESA DE SOM** (v204, F30 §B2) — `ph2d_audio::Mesa::to_bytes`, versão PRÓPRIA (vazio = a de
    /// fábrica). Fora do `ProjectState`: a mesa nunca teve undo.
    pub(crate) mesa: Vec<u8>,
    /// **Os KANBANS** (v208, ADR-0195; `ph2d_app_kanban::project`), fora do `ProjectState` (desfazer
    /// próprio). Os braços `189`..`207` do load acrescentam o `0` deste `Vec`.
    pub(crate) kanbans: Vec<u8>,
    /// **Os PIXEL SPRITES** (v209, ADR-0196) — os documentos de pixels dos objectos Pixel Sprite
    /// (`ph2d_app_pixel::persist`, formato `PH2DPIXEL` com versão PRÓPRIA), chaveados pelo
    /// `PixelSprite(u32)` de cada objecto. Fora do `ProjectState`: cada sprite tem desfazer próprio.
    pub(crate) pixel_sprites: Vec<u8>,
    /// **Os SONS do projeto** (v214, GEF 21 E2) — `ph2d_ecs::ProjectSounds`, blob `PH2DSONS` com versão
    /// PRÓPRIA (vazio = sem sons); o `AudioSource2D.sound` nomeia-os. O `falta_no_fim` e os braços
    /// `189`..`192` do load acrescentam o `0` deste `Vec` vazio.
    pub(crate) sounds: Vec<u8>,
    /// **As RECEITAS DE SOM e o material** (v217, SFX Lab W2): `ph2d_sfx_recipe::SfxLibrary`, com versão PRÓPRIA.
    pub(crate) sfx_lab: Vec<u8>,
    /// **Os nomes das camadas de colisão** (v218, ficha f1): `ph2d_physics_ecs::LayerNames`, com versão PRÓPRIA.
    pub(crate) physics_layers: Vec<u8>,
    /// **A identidade do JOGO** (v232, plano InputMap §4): sorteada quando o projecto NASCE
    /// ([`ph2d_player_controls::new_game_id`]) e mantida por save/load; chaveia o ficheiro do jogador.
    /// Fora do `ProjectState`: um undo não troca de jogo. ⚠️ O último campo (postcard é posicional).
    pub(crate) game_id: u64,
}

/// A identidade do jogo, no mundo (mora em `ph2d_player_controls`, que a aba Input Map também lê).
pub(crate) use ph2d_player_controls::GameId;

impl crate::App {
    /// A identidade do jogo deste projecto (ver [`ProjectFile::game_id`]); `0` só sem mundo.
    pub(crate) fn game_id(&mut self) -> u64 {
        let g = self.gfx.as_mut();
        g.map_or(0, |g| ph2d_player_controls::game_id_of(g.sim.world_mut()))
    }
}

/// Uma imagem de sprite embutida no projeto: os pixels RGBA + a célula de atlas que
/// o `Sprite.source` referencia. (Fase 2b.)
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct SavedAsset {
    /// A célula de atlas (`SpriteSource::Atlas { key }`) que estes pixels ocupam.
    key: u32,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

/// **O que a troca de documento faz os produtores VIVOS esquecerem** — irmão pelo teto de LOC
/// (HR-18), e o corte é por responsabilidade: aqui vive *o que não sobrevive a um load*.
#[path = "project_forget.rs"]
mod forget;

/// **O lado da ESCRITA** — irmão pelo teto de LOC (HR-18); o corte é por
/// responsabilidade: aqui fica *o que um arquivo É*, lá *como ele é escrito*.
#[path = "project_save.rs"]
mod save;

/// **O lado da LEITURA** — irmão pelo teto de LOC (HR-18); o corte é por
/// responsabilidade: aqui fica *o que um arquivo É e como ele é escrito*, lá *como
/// ele é lido e a sessão esquece o documento anterior*.
#[path = "project_load.rs"]
mod load;

/// A metade do load que decide QUE BYTES SÃO ESTES — irmã do módulo acima por tecto de LOC.
#[path = "project_load_read.rs"]
mod load_read;

/// **Os pixels que o undo não guarda** — irmão pelo teto de LOC (HR-18); o corte é por
/// responsabilidade: aqui fica *o que um arquivo É*, lá *como os pixels vão e voltam do atlas*.
#[path = "project_assets.rs"]
mod assets;

#[cfg(test)]
#[path = "project_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "undo_app_restauro_tests.rs"]
mod restauro_tests;

/// O que a BIBLIOTECA deve ao arquivo — irmão pelo tecto de LOC (HR-18).
#[cfg(test)]
#[path = "project_library_file_tests.rs"]
mod library_file_tests;

/// **As settings FORA do `ProjectState` atravessam o arquivo** — irmão de `tests`,
/// cortado por assunto quando o pai bateu o cap de LOC.
#[cfg(test)]
#[path = "project_settings_tests.rs"]
mod settings_tests;

#[cfg(test)]
#[path = "project_schema_tests.rs"]
mod schema_tests;

/// **O INPUT MAP no undo** (v232): cada pedido da aba, pelo `frame` real, é UM passo, bit a bit.
#[cfg(test)]
#[path = "project_input_map_tests.rs"]
mod input_map_tests;
