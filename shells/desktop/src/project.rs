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
struct ProjectFile {
    /// Mundo (ECS) + geometria vetorial — a unidade do undo.
    state: ProjectState,
    /// Pixels dos sprites, para re-materializar o atlas noutra sessão (Fase 2b).
    /// Vazio na Fase 2a.
    assets: Vec<SavedAsset>,
    /// Os **documentos do Painter** (camadas + pixels + relevo), por identidade estável
    /// (`ph2d_ecs::PaintedDoc`). Vazio quando nada foi pintado. Ver [`crate::project_painter`].
    painted: Vec<ph2d_tool_painter::PaintedDocument>,
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
    motion: String,
    /// O **`TimelineDoc`** (clips, faixas, tracks, keys) em postcard — a animação inteira.
    ///
    /// Fora do `ProjectState` pelo mesmo motivo do `motion`: o `ProjectState` é a unidade do
    /// undo GLOBAL, e a timeline tem undo próprio. Enfiá-la ali faria cada Ctrl+Z do canvas
    /// rebobinar a animação junto.
    ///
    /// As bindings viajam com o **`wire_id`** (hash do `Name` do objeto) carimbado no save, e
    /// NÃO com os bits de entidade — que o load recicla. Quem as recola é o `upkeep` do frame,
    /// a mesma função que cura delete+undo (ver [`crate::timeline_persist`]). Um projeto sem
    /// animação carrega `vec![]`.
    timeline: Vec<u8>,
    /// As **settings de MUNDO** da física (ADR-0131 D8 / W2b).
    ///
    /// Fora do `ProjectState` de propósito: o `ProjectState` é a unidade do undo
    /// GLOBAL, e um Ctrl+Z do canvas não deve rebobinar a gravidade da cena —
    /// o mesmo motivo que mantém `motion` e `timeline` aqui fora.
    ///
    /// O mundo rapier em si **não** viaja (D2: ele é derivado); o que viaja é o
    /// que o artista autorou.
    physics: ph2d_physics_ecs::PhysicsSettings,
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
    tokens: Vec<crate::project_tokens::SavedToken>,
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
    settings: crate::project_settings::SavedSettings,
    /// **A ESCULTURA** (ADR-0150 W8.3) — a lista de peças, cada uma com a pilha de
    /// níveis e a pose, em postcard. Ver [`crate::sculpt3d`] (`sculpt3d_doc.rs`).
    ///
    /// Fora do `ProjectState` pelo mesmo motivo de `motion`/`timeline`/`physics`: o
    /// `ProjectState` é a unidade do undo GLOBAL, e a escultura tem fila própria —
    /// um Ctrl+Z do canvas não pode rebobinar uma pincelada de barro.
    ///
    /// ⚠️ **`Vec<u8>` opaco e SEM `cfg`**, e é isso que sustenta a promessa de
    /// removibilidade do `docs/3D/02.3`: o campo existe com o módulo desligado (o
    /// postcard é posicional — um campo condicional daria DUAS formas de arquivo com o
    /// mesmo número de schema), e um binário sem escultura **carrega os bytes adiante**
    /// em vez de os triturar. Ele carrega a própria versão lá dentro.
    sculpt: Vec<u8>,
    /// **OS CANAIS ASSADOS** (ADR-0150 W8.7) — por objeto: os pixels antes da luz, o G-buffer que
    /// uma malha doou, e o rig com que aquilo foi aceso. Ver [`crate::project_baked_form`].
    ///
    /// ⚠️ **Campo de SPRITE, e não parte do blob `sculpt` acima**, embora aquele já guarde as
    /// malhas. O parser da escultura é `#[cfg(feature = "sculpt3d")]`; guardar os canais lá os
    /// tornaria legíveis só com o módulo 3D no build — o oposto exato do que a *rota A* promete
    /// (`docs/3D/02.2`: a malha some do build, o objeto continua reluminável). Ele fica ao lado do
    /// `painted`, que resolve o mesmo problema para o outro produtor de `SpriteSource::Individual`.
    ///
    /// Vazio quando nada foi assado.
    baked_forms: Vec<crate::project_baked_form::BakedFormDocument>,
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
    player_tape: ph2d_physics_ecs::TapeWire,
    /// **OS PIXELS PRÓPRIOS** (plano [`docs/Sprite_projeto/17`] §3) — os bytes de todo sprite
    /// `SpriteSource::Individual`, nomeados pelo `ph2d_ecs::SpritePixels` que ele carrega.
    /// Ver [`crate::project_sprite_pixels`].
    ///
    /// ⚠️ **Ele fecha uma perda de dados que já acontecia**, e não é uma capacidade nova: o
    /// `texture_id` do `Individual` é um id de alocação da GPU, e o store recomeça em `1` a cada
    /// processo — um sprite tocado por qualquer ferramenta de imagem reabria **invisível**, ou a
    /// exibir os pixels de outro sprite. O `painted` (v3) e o `baked_forms` resolveram isto para
    /// os DOIS produtores ricos; este campo é o chão que faltava debaixo deles, e cobre o funil
    /// que todas as ferramentas atravessam (`commit_edited_texture`).
    ///
    /// ⚠️ **`Vec<u8>` opaco, e carrega a própria versão lá dentro** (`SHEET_DOC_VERSION`) — o
    /// precedente literal do `timeline` e do `sculpt`. É o que faz as REGIÕES do hand-packed,
    /// que entram neste mesmo documento, não voltarem a bumpar o `PROJECT_SCHEMA`.
    ///
    /// Vazio num projeto sem sprites individuais.
    sprite_pixels: Vec<u8>,
}

/// Uma imagem de sprite embutida no projeto: os pixels RGBA + a célula de atlas que
/// o `Sprite.source` referencia. (Fase 2b.)
#[derive(serde::Serialize, serde::Deserialize)]
struct SavedAsset {
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

/// **Os pixels que o undo não guarda** — irmão pelo teto de LOC (HR-18); o corte é por
/// responsabilidade: aqui fica *o que um arquivo É*, lá *como os pixels vão e voltam do atlas*.
#[path = "project_assets.rs"]
mod assets;

#[cfg(test)]
#[path = "project_tests.rs"]
mod tests;

/// **As settings FORA do `ProjectState` atravessam o arquivo** — irmão de `tests`,
/// cortado por assunto quando o pai bateu o cap de LOC.
#[cfg(test)]
#[path = "project_settings_tests.rs"]
mod settings_tests;

#[cfg(test)]
#[path = "project_schema_tests.rs"]
mod schema_tests;
