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
const PROJECT_SCHEMA: u32 = 1;

/// O conteúdo de um arquivo de projeto.
#[derive(serde::Serialize, serde::Deserialize)]
struct ProjectFile {
    /// Mundo (ECS) + geometria vetorial — a unidade do undo.
    state: ProjectState,
    /// Pixels dos sprites, para re-materializar o atlas noutra sessão (Fase 2b).
    /// Vazio na Fase 2a.
    assets: Vec<SavedAsset>,
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
        let Some(state) = self.capture_project() else {
            return;
        };
        let assets = self.collect_assets();
        let file = ProjectFile { state, assets };
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
        self.undo = ProjectUndo::default();
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
        let state = ProjectState {
            world: WorldSnapshot::new(),
            vec,
        };
        let file = ProjectFile {
            state: state.clone(),
            assets: vec![SavedAsset {
                key: 16,
                width: 2,
                height: 2,
                rgba: vec![10, 20, 30, 40],
            }],
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
    }
}
