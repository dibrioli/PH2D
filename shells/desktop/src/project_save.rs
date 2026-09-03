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

use super::{PROJECT_SCHEMA, ProjectFile};

impl crate::App {
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

    /// Serializa o projeto inteiro (mundo + geometria + pixels) para `path`.
    ///
    /// ⚠️ **O caminho é PARÂMETRO, e é a única porta.** Até 2026-08-23 havia um `project_save()`
    /// que resolvia o destino aqui dentro — uma env var lida a cada gravação, com
    /// `ph2d_project.postcard` no diretório corrente como default silencioso. Quem decide *onde*
    /// é hoje o [`crate::App::project_save_gesture`], que **pergunta** quando ainda não há
    /// ficheiro; esta função só sabe escrever. É a mesma razão do `project_load_from`: uma decisão
    /// escondida dentro de quem executa não é alcançável nem por um gate nem por um diálogo.
    pub(crate) fn project_save_to(&mut self, path: &str) {
        let assets = self.collect_assets();
        // Os documentos pintados carimbam a identidade estável no mundo, então isto tem de rodar ANTES
        // da captura — senão o `PaintedDoc` recém-inserido ficaria de fora do snapshot e o load não
        // teria a quem devolver o documento.
        let painted = self.collect_painted_docs();
        // Os pixels próprios de todo sprite Individual (plano `docs/Sprite_projeto/17` §3).
        // ⚠️ DEPOIS do `collect_painted_docs`: é ele que carimba o `PaintedDoc`, e a colheita
        // daqui SALTA quem o tem — um sprite pintado tem dono mais rico, e guardar as duas
        // coisas seria gravar duas verdades sobre o mesmo sprite. Antes dele, o carimbo ainda
        // não existiria e o achatado entraria no arquivo à socapa.
        let sprite_pixels = self.collect_sprite_pixels();
        // A ARTE DOS PADRÕES de textura (plano 33 W4) — os pixels que cada `Paint::Pattern` da
        // cena vectorial nomeia por `AssetId`. Vazio quando não há padrão nenhum.
        let pattern_art = self
            .gfx
            .as_ref()
            .map(|g| g.vec_scene.clone())
            .map(|scene| self.collect_texture_pattern_art(&scene))
            .unwrap_or_default();
        // A animação. O `serialize` carimba em cada binding a IDENTIDADE do objeto
        // (`StableId`) — é por ela que a track reencontra o objeto do outro lado do arquivo
        // (os bits de entidade não sobrevivem a um respawn). Precisa do mundo, então vem antes
        // da captura — e por isso pede `&mut`: os ids têm de existir aqui, não depois.
        let timeline = match self.gfx.as_mut() {
            Some(gfx) => {
                let world = gfx.sim.world_mut();
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
            sprite_pixels,
            // ⚠️ **O contador de identidade** (ADR-0164 F1). Ele vive no MUNDO como recurso e
            // sobe sozinho; aqui ele é só fotografado. Gravar um valor atrasado faria a
            // sessão seguinte entregar um id que já está vivo no ficheiro — por isso o load
            // reconcilia contra os ids que de facto lá estão, e não confia só neste número.
            stable_id_counter: self
                .gfx
                .as_ref()
                .and_then(|g| g.sim.world().get_resource::<ph2d_ecs::StableIdCounter>())
                .map_or(ph2d_ecs::StableId::FIRST, |c| c.next_free()),
            // ⚠️ O mapa vai por VALOR, e nao ha' traducao: ele E' o documento (v97).
            // ⚠️ **O dono e' o `HeroScreen`**, como as `ProjectSettings` -- e a janela que o edita
            // e' pintada por `paint_hero_screen`, que so' recebe o hero. Sem hero (a GPU ainda nao
            // subiu) grava-se o vazio, que e' o que o projecto de facto tem.
            input_map: self
                .gfx
                .as_ref()
                .and_then(|g| g.hero_screen.as_ref())
                .map(|h| h.input_map.clone())
                .unwrap_or_default(),
            pattern_art,
        };
        let bytes = match postcard::to_allocvec(&(PROJECT_SCHEMA, &file)) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("[proj] falha ao serializar: {e}");
                return;
            }
        };
        match std::fs::write(path, &bytes) {
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
    ///
    /// ⚠️ **`pub(crate)` desde 2026-09-02**: era `pub(super)` (visível só dentro do `project`, de
    /// que este ficheiro é submódulo), e o *Export SVG…* precisa da MESMA porta — um segundo
    /// mecanismo de aviso divergiria no estilo e no tempo de vida da mensagem.
    pub(crate) fn toast(&mut self, msg: String) {
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.toasts.push(ph2d_editor::Toast::info(msg));
        }
    }
}
