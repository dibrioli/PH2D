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

use crate::undo::{ProjectState, ProjectUndo};

/// Versão do formato de arquivo de projeto. Bump ⇒ migração ou hard-break.
/// v2 (ADR-0114): `ProjectState` ganhou o campo `flip: FlipDoc` (3º) — postcard é
/// posicional, então um arquivo v1 não desserializa. Sem custo real: a
/// persistência ainda é stub (sem diálogo de arquivo), sem saves publicados.
/// v3: `ProjectFile` ganhou os **documentos do Painter** (3º campo). Sem eles o projeto salvava um
/// sprite apontando para uma textura de runtime que morre com o processo — pintar, salvar e reabrir
/// devolvia o quadro em branco. Ver [`crate::project_painter`].
/// v4: o `Layer` do Painter ganhou o **Impasto por camada** (`impasto_depth` / `impasto_composite` /
/// `has_relief`) — postcard é posicional, então um documento pintado v3 lê lixo nos campos seguintes.
/// Rejeitar é a única leitura honesta.
/// v5 (doc 56): `ProjectFile` ganhou `motion` (o grafo de Motion Nodes, em texto) — 4º campo. Pelo
/// mesmo motivo posicional, um v4 não desserializa aqui.
/// v6 (ADR-0114 W3): a `FlipDoc` — que vive DENTRO do `ProjectState` — mudou de forma: a camada
/// ganhou `cycle` + `use_onion` e o `OnionSettings` ganhou `kind_filter` (`FLIP_SCHEMA_VERSION` 1→2).
/// Não é campo novo no ARQUIVO, é o MESMO campo com outro layout — e posicional é posicional.
/// v7 (ADR-0114 W4): o `FlipStroke` ganhou `holes` + `hide_stroke` (o balde —
/// `FLIP_SCHEMA_VERSION` 2→3). Mesma regra: a forma mudou, então a versão sobe.
/// Sem o bump, um arquivo v6 passaria na checagem de versão e seria lido com o
/// layout NOVO — postcard não tem nomes de campo para reclamar, e o que sai é
/// geometria embaralhada em vez de um erro honesto.
/// v8 (ADR-0114 W6): o `FlipStroke` ganhou `selected` — a seleção é ATRIBUTO do traço
/// (o Edit Mode; `FLIP_SCHEMA_VERSION` 3→4), e não estado do shell. Idem: a forma do
/// `FlipDoc` mudou dentro do `ProjectState`, então o par sobe junto.
const PROJECT_SCHEMA: u32 = 8;

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

impl crate::App {
    /// Caminho do arquivo de projeto (env `PH2D_PROJECT_PATH`, default no CWD).
    fn project_path() -> String {
        std::env::var("PH2D_PROJECT_PATH").unwrap_or_else(|_| "ph2d_project.postcard".to_string())
    }

    /// Ctrl+S: serializa o projeto inteiro (mundo + geometria + pixels) para o disco.
    pub(crate) fn project_save(&mut self) {
        let assets = self.collect_assets();
        // Os documentos pintados carimbam a identidade estável no mundo, então isto tem de rodar ANTES
        // da captura — senão o `PaintedDoc` recém-inserido ficaria de fora do snapshot e o load não
        // teria a quem devolver o documento.
        let painted = self.collect_painted_docs();
        let Some(state) = self.capture_project() else {
            return;
        };
        let file = ProjectFile {
            state,
            assets,
            painted,
            motion: self
                .gfx
                .as_ref()
                .map(|g| g.motion.doc.to_text())
                .unwrap_or_default(),
        };
        let bytes = match postcard::to_allocvec(&(PROJECT_SCHEMA, &file)) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("[proj] falha ao serializar: {e}");
                return;
            }
        };
        let path = Self::project_path();
        match std::fs::write(&path, &bytes) {
            Ok(()) => eprintln!("[proj] salvo: {path} ({} bytes)", bytes.len()),
            Err(e) => eprintln!("[proj] erro ao gravar {path}: {e}"),
        }
    }

    /// Ctrl+O: carrega um projeto do disco, substituindo a cena atual. Zera a fila de
    /// undo (documento novo, histórico novo).
    pub(crate) fn project_load(&mut self) {
        let path = Self::project_path();
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("[proj] sem arquivo {path}: {e}");
                return;
            }
        };
        let (ver, file): (u32, ProjectFile) = match postcard::from_bytes(&bytes) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[proj] erro ao ler {path}: {e}");
                return;
            }
        };
        if ver != PROJECT_SCHEMA {
            eprintln!("[proj] schema {ver} != {PROJECT_SCHEMA} — recusado");
            return;
        }
        self.materialize_assets(&file.assets);
        self.apply_project(&file.state);
        // Depois do mundo: os sprites já existem (com bits novos), e é pelo `PaintedDoc` que cada um
        // reencontra o documento que era dele.
        self.restore_painted_docs(file.painted);
        // O grafo de Motion. Um erro de parse NÃO aborta o load: a cena, a geometria e os
        // pixels já entraram, e recusar tudo por causa do grafo perderia o resto do
        // trabalho. O grafo em memória permanece, e o motivo vai pro log.
        if !file.motion.is_empty()
            && let Some(gfx) = self.gfx.as_mut()
            && let Err(e) = gfx.motion.load_text(&file.motion)
        {
            eprintln!("[proj] grafo de motion ilegivel, mantido o atual: {e:?}");
        }
        // O relógio volta a zero. O Motion não tem transporte próprio (W4.T7) — o `install`
        // não pode se rebobinar sozinho, e é aqui que a rebobinada pertence: um projeto
        // recém-carregado começa no início, e um playhead adiantado sobre uma simulação que
        // nunca rodou mentiria sobre um estado que não existe.
        self.playhead.rewind();
        self.undo = ProjectUndo::default();
        self.flip_live_clear(); // documento novo: o alvo vivo do anterior morreu
        eprintln!("[proj] carregado: {path}");
    }

    /// Coleta os pixels de cada imagem importada, para embutir no arquivo.
    ///
    /// A lista canônica é o `atlas_asset_map` (`key → AssetId`) — o `AssetDb` não
    /// itera. Cobre só `SpriteSource::Atlas` (o caminho comum de import); `Individual`
    /// (Painter/Apply) e `CookedTexture` (KTX2) ficam fora do 1º corte.
    fn collect_assets(&self) -> Vec<SavedAsset> {
        let Some(gfx) = self.gfx.as_ref() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (&key, asset_id) in &gfx.atlas_asset_map {
            if let Some(asset) = gfx.asset_db.get(asset_id)
                && let ph2d_asset::Asset::ImageRgba8 {
                    width,
                    height,
                    pixels,
                } = &*asset
            {
                out.push(SavedAsset {
                    key,
                    width: *width,
                    height: *height,
                    rgba: pixels.to_vec(),
                });
            }
        }
        out
    }

    /// Re-insere os pixels no `AssetDb` e re-empacota o atlas **nas mesmas células**
    /// (`key`) que os sprites restaurados referenciam. O `key` é caller-supplied, então
    /// os `Sprite.source = Atlas { key }` do WorldSnapshot resolvem de novo.
    fn materialize_assets(&mut self, assets: &[SavedAsset]) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        // Campos irmãos de `AppGfx` — refs disjuntas para o closure `fetch` (só lê o
        // mapa/db) coexistir com o `&mut renderer`.
        let renderer = &mut gfx.renderer;
        let asset_db = &gfx.asset_db;
        let atlas_asset_map = &mut gfx.atlas_asset_map;

        // 1. Pixels no AssetDb + o vínculo key→AssetId, ANTES dos inserts (um regrow
        //    disparado no meio precisa ver todos os keys já mapeados).
        for a in assets {
            let id = asset_db.insert_image_rgba8(a.width, a.height, a.rgba.clone());
            atlas_asset_map.insert(a.key, id);
        }
        // 2. Empacota cada um no atlas (upload GPU + mips internos). O `fetch` é o
        //    mesmo de `image_import::pack_image`: re-materializa as regiões num regrow.
        for a in assets {
            let fetch = |key: u32| -> Option<Vec<u8>> {
                let aid = atlas_asset_map.get(&key)?;
                match &*asset_db.get(aid)? {
                    ph2d_asset::Asset::ImageRgba8 { pixels, .. } => Some(pixels.to_vec()),
                    _ => None,
                }
            };
            if let Err(e) =
                renderer.insert_atlas_sprite_with_regrow(a.key, a.width, a.height, &a.rgba, fetch)
            {
                eprintln!("[proj] atlas insert key={}: {e}", a.key);
            }
        }
        // 3. Imports futuros nesta sessão não podem colidir com os keys do projeto.
        if let Some(max_key) = assets.iter().map(|a| a.key).max() {
            gfx.next_import_cell = gfx.next_import_cell.max(max_key.saturating_add(1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::undo::ProjectState;
    use ph2d_ecs::scene::WorldSnapshot;
    use ph2d_vec_scene::{VecScene, rectangle};

    /// O arquivo de projeto sobrevive ao round-trip postcard: geometria, versão e os
    /// pixels embutidos voltam idênticos.
    #[test]
    fn project_file_round_trips_through_postcard() {
        let mut vec = VecScene::new();
        vec.push_path(rectangle([0.0, 0.0], [2.0, 2.0]));
        // ADR-0114: o FlipDoc entra no ProjectState (3º campo) → o save o carrega
        // de graça (mesma captura do undo).
        let mut flip = ph2d_flip::FlipDoc::new();
        flip.push_object("Anim");
        let state = ProjectState {
            world: WorldSnapshot::new(),
            vec,
            flip,
        };
        // O grafo de Motion viaja como TEXTO canônico — a forma real que o `MotionDoc`
        // serializa (doc 56), não uma string inventada: se o formato mudar, o teste viaja
        // junto em vez de mentir que sobreviveu.
        let motion = ph2d_motion_doc::MotionDoc::new().to_text();
        let file = ProjectFile {
            state: state.clone(),
            assets: vec![SavedAsset {
                key: 16,
                width: 2,
                height: 2,
                rgba: vec![10, 20, 30, 40],
            }],
            painted: Vec::new(),
            motion: motion.clone(),
        };
        let bytes = postcard::to_allocvec(&(PROJECT_SCHEMA, &file)).unwrap();
        let (ver, back): (u32, ProjectFile) = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(ver, PROJECT_SCHEMA);
        assert_eq!(back.state, state, "estado (mundo + geometria) preservado");
        assert_eq!(back.assets.len(), 1);
        assert_eq!(back.assets[0].key, 16);
        assert_eq!(
            back.assets[0].rgba,
            vec![10, 20, 30, 40],
            "pixels preservados"
        );
        assert_eq!(back.motion, motion, "o grafo de Motion preservado");
        assert!(
            ph2d_motion_doc::MotionDoc::from_text(&back.motion).is_ok(),
            "…e ainda parseável do outro lado do arquivo"
        );
    }

    /// **Estopim de esquema.** O `ProjectState` embute o `FlipDoc` inteiro, e o
    /// postcard é POSICIONAL: qualquer campo novo em qualquer struct do Flip muda
    /// o layout do arquivo de projeto. Sem bump, o loader aceita o arquivo velho
    /// (a versão bate) e o lê com o layout novo — sai geometria embaralhada, não
    /// um erro. Foi o que quase aconteceu na W4 (`holes`/`hide_stroke`).
    ///
    /// Este par existe para que bumpar UM sem pensar no OUTRO fique vermelho.
    ///
    /// O `PROJECT_SCHEMA` é 7 (e não o 4 que a linha Flip trazia sozinha) porque na
    /// árvore integrada ele conta TODAS as quebras de layout, não só as do Flip: v3/v4
    /// do Painter (documentos + impasto), v5 do Motion (o grafo), v6/v7 do Flip. Cada
    /// linha subiu o mesmo contador por um motivo diferente — e o contador é um só.
    #[test]
    fn a_flip_schema_bump_must_bump_the_project_schema() {
        assert_eq!(
            (PROJECT_SCHEMA, ph2d_flip::FLIP_SCHEMA_VERSION),
            (8, 4),
            "a forma do FlipDoc mudou (ou o esquema do projeto): suba o PROJECT_SCHEMA \
             junto e atualize este par. Postcard nao avisa - ele so le errado."
        );
    }
}
