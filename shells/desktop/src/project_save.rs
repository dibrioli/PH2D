//! **O lado da ESCRITA** — irmão de `project.rs` pelo teto de LOC (HR-18), e o
//! corte é por RESPONSABILIDADE, não por tamanho: no pai fica *o que um arquivo
//! É* (a escada de schema, o `ProjectFile`, o `SavedAsset`), aqui *como ele é
//! ESCRITO*, e no irmão `project_load.rs` *como ele é lido e a sessão esquece o
//! documento anterior*.
//!
//! ⚠️ O pai cruzou o cap na INTEGRAÇÃO, não numa linha: cinco linhas paralelas
//! apendaram degraus à escada de schema na mesma janela e nenhuma o cruzava
//! sozinha. O gate de LOC da shell só corre na varredura impactada, então quem
//! o viu foi a árvore combinada.

use super::{PROJECT_SCHEMA, ProjectFile, SavedAsset};

impl crate::App {
    /// Caminho do arquivo de projeto (env `PH2D_PROJECT_PATH`, default no CWD).
    pub(super) fn project_path() -> String {
        std::env::var("PH2D_PROJECT_PATH").unwrap_or_else(|_| "ph2d_project.postcard".to_string())
    }

    /// **Os bytes da escultura que este save vai gravar.**
    ///
    /// A cena VIVA é a verdade sempre que ela existe; quando não existe, a verdade são
    /// os bytes como vieram do arquivo — o que cobre os dois casos honestos: um projeto
    /// aberto antes de a GPU aparecer (o `pending` ainda não instalou) e um binário
    /// construído **sem** o módulo, que carrega adiante o que não sabe ler.
    pub(super) fn sculpt_bytes_for_save(&self) -> Vec<u8> {
        #[cfg(feature = "sculpt3d")]
        if let Some(gfx) = self.gfx.as_ref()
            && let Some(scene) = gfx.sculpt3d.as_ref()
        {
            return scene.to_doc_bytes();
        }
        self.sculpt_doc.clone()
    }

    /// Ctrl+S: serializa o projeto inteiro (mundo + geometria + pixels) para o disco.
    pub(crate) fn project_save(&mut self) {
        let assets = self.collect_assets();
        // Os documentos pintados carimbam a identidade estável no mundo, então isto tem de rodar ANTES
        // da captura — senão o `PaintedDoc` recém-inserido ficaria de fora do snapshot e o load não
        // teria a quem devolver o documento.
        let painted = self.collect_painted_docs();
        // A animação. O `serialize` carimba em cada binding o hash do NOME do objeto — é por
        // ele que a track reencontra o objeto do outro lado do arquivo (os bits de entidade
        // não sobrevivem a um respawn). Precisa do mundo, então vem antes da captura.
        let timeline = match self.gfx.as_ref() {
            Some(gfx) => {
                let world = gfx.sim.world();
                match crate::timeline_persist::serialize(&mut self.timeline, world) {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("[proj] timeline nao serializou, projeto salvo SEM ela: {e}");
                        Vec::new()
                    }
                }
            }
            None => Vec::new(),
        };
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
            timeline,
            physics: self
                .gfx
                .as_ref()
                .map(|g| g.physics.settings())
                .unwrap_or_default(),
            tokens: crate::project_tokens::collect(),
            // O mesmo formato dos irmãos acima: sem janela não há `HeroScreen`, e
            // um save headless grava os defaults — que é o estado que ele de fato tem.
            settings: crate::project_settings::collect(
                self.gfx
                    .as_ref()
                    .and_then(|g| g.hero_screen.as_ref())
                    .map(|h| h.project)
                    .unwrap_or_default(),
            ),
            sculpt: self.sculpt_bytes_for_save(),
            baked_forms: self.collect_baked_forms(),
            // A corrida que o artista jogou (W17). O `to_wire` é a única tradução
            // — o `PlayerInput` da crate da LEI não conhece serde de propósito.
            player_tape: self.player_tape.to_wire(),
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
            Ok(()) => {
                eprintln!("[proj] salvo: {path} ({} bytes)", bytes.len());
                let n = self.timeline.doc.bindings().len();
                self.toast(format!(
                    "Project saved · {} KB · {n} animation track(s)",
                    bytes.len() / 1024
                ));
            }
            Err(e) => {
                eprintln!("[proj] erro ao gravar {path}: {e}");
                self.toast(format!("Project save FAILED: {e}"));
            }
        }
    }

    /// Um aviso na tela — não só no terminal. O Ctrl+O é destrutivo (troca a cena, zera o undo)
    /// e o Ctrl+S é silencioso; um `eprintln!` num app de janela é uma mensagem para ninguém.
    pub(super) fn toast(&mut self, msg: String) {
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.toasts.push(ph2d_editor::Toast::info(msg));
        }
    }
}
