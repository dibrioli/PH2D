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
/// v9 (ADR-0114 W7.2): a CHAVE (`FlipFrame`) ganhou `offset` — a **pose do quadro**
/// (`FLIP_SCHEMA_VERSION` 4→5). É o que faz uma instância (duas chaves, um desenho) ser
/// mais que um hold: a arte é compartilhada e o lugar é de cada quadro.
/// v10 (ADR-0121): o `VecVertex` ganhou `corner_radius` (Live Corners —
/// `ph2d_vec_scene::corner_live`), e a `VecScene` vai embutida aqui
/// (`VEC_SCENE_SCHEMA_VERSION` 7→8). Mesma regra.
/// v11: o `PaintedDocument` ganhou `mats` — o MATERIAL do Impasto por camada
/// (Roughness/Metallic/Wax/Shine, por pixel). Sem o bump, um save anterior seria lido com o
/// layout novo e o material sairia dos bytes da COBERTURA. (O `Shine` deixou de ser global e
/// virou propriedade da TINTA — Enio, 2026-07-13.)
/// v12: o MESMO `mats` mudou de FORMA — 4 bytes → 7 (a **cor do Wax**, o filtro sobre a luz que
/// atravessa a tinta). Não é campo novo, é o mesmo campo com outro layout, e posicional é
/// posicional: sem o bump um v11 passaria na checagem e o material sairia dos bytes errados.
/// v13 (W4.T6/B5): `ProjectFile` ganhou `timeline` (o `TimelineDoc` em postcard) — 5º campo.
/// **A animação era perdida ao fechar o app**: nada a salvava (o "sidecar" que dizia salvá-la
/// era código morto — o Ctrl+S global já retornava antes). Os bytes trazem a própria versão
/// (`DOC_VERSION`), então um bump lá é RECUSADO com erro honesto e não obriga a bumpar aqui;
/// o campo NOVO, sim, obriga (posicional).
const PROJECT_SCHEMA: u32 = 13;

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

    /// Ctrl+O: carrega o projeto do caminho da sessão (env `PH2D_PROJECT_PATH`).
    pub(crate) fn project_load(&mut self) {
        self.project_load_from(&Self::project_path());
    }

    /// O load de verdade, com o caminho **injetado** — substitui a cena atual e assenta a
    /// sessão (relógio + histórico).
    ///
    /// O caminho é parâmetro (e não a env var) porque é isto que torna o load **dirigível
    /// sem janela**: um `App` recém-construído tem `window`/`host`/`gfx` em `None` (o winit
    /// só cria a janela no `resumed`), e todo passo daqui que depende de `gfx` já degrada
    /// para no-op. Os gates em `tests` dirigem ESTA função — o corpo inteiro que o Ctrl+O
    /// executa (o `project_load` acima só resolve o caminho) — e não uma cópia da decisão
    /// posta num helper que ninguém chama.
    pub(crate) fn project_load_from(&mut self, path: &str) {
        let bytes = match std::fs::read(path) {
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
        // ---- Daqui pra baixo o arquivo foi ACEITO. ----
        //
        // **A SESSÃO ESQUECE O DOCUMENTO ANTERIOR.** Este bloco fica colado na decisão de
        // aceitar, e não lá no fim: entre o aceite e o esquecimento não sobra nenhum passo que
        // dependa de `gfx`, então o que o gate headless observa é exatamente o que o app com
        // janela faz. (O `MotionState::install` faz a MESMA lista pro runtime dele; a lição é
        // a mesma — o que nomeia o documento antigo por id/bits/relógio não "fica velho", é
        // ADOTADO por quem herdar o número.)
        //
        // **O relógio volta a zero — e PAUSA.** O Motion não tem transporte próprio (W4.T7): o
        // `install` não pode se rebobinar sozinho, e o `Playhead` é do editor. Um playhead
        // adiantado sobre uma simulação que nunca rodou mentiria sobre um estado que não existe
        // (o pump entraria pelo caminho de SCRUB e abriria o documento no meio da nevasca —
        // `motion_state_tests::a_clock_that_was_not_rewound_opens_the_document_mid_scene`). E o
        // `rewind` NÃO pausa (`Playhead` doc: *"keeps rate + play state"*), então sem o `pause`
        // o projeto recém-aberto ficava em t=0 por UM frame e saía correndo — o app pausa o
        // próprio playhead no boot pela mesma razão (`main.rs`), e um load é o boot de um
        // documento.
        self.playhead.rewind();
        self.playhead.pause();
        self.undo = ProjectUndo::default(); // documento novo, histórico novo
        self.flip_live_clear(); // …e o alvo vivo do anterior morreu junto
        // **A TIMELINE do documento anterior morre aqui** — a do arquivo entra no fim (W4.T6/B5).
        //
        // Não é higiene: as bindings do documento anterior nomeiam entidades que o
        // `apply_project` vai despawnar logo abaixo — e o `timeline_persist::upkeep` RECONECTA
        // binding órfã por **hash do Name** (é o que cura delete+undo). Nomes se repetem entre
        // projetos ("Layer 1", "sprite_001"), então, deixada viva, a animação do projeto A
        // adotaria os objetos homônimos do projeto B no frame seguinte e passaria a dirigir a
        // pose deles — uma animação que não está em arquivo nenhum, e com a fila de undo já
        // zerada logo acima.
        self.timeline = ph2d_timeline::TimelineState::new();
        self.timeline_intents.clear();
        self.timeline_insert_key = false;
        self.timeline_reveal_after_apply = false;
        self.autokey = Default::default(); // pins/baselines de pose keyados por bits mortos
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
        // **A ANIMAÇÃO** (W4.T6/B5). As bindings entram DESTACADAS (bits de entidade zerados) e
        // o `upkeep` do frame as recola nos objetos que o `apply_project` acabou de spawnar,
        // pelo hash do `Name` — a MESMA função que cura delete+undo. Por isso este passo não
        // precisa do mundo, e por isso o load não pode divergir do undo: a resolução por nome
        // existe uma vez só. Um documento ilegível (outra era do `DOC_VERSION`) NÃO aborta o
        // load — a cena já entrou; a timeline fica vazia e o motivo vai pro log.
        if !file.timeline.is_empty() {
            match crate::timeline_persist::install_from_project(&mut self.timeline, &file.timeline)
            {
                Ok(n) => eprintln!("[proj] timeline: {n} track(s) a recolar por nome"),
                Err(e) => eprintln!("[proj] timeline ilegivel, projeto aberto SEM animacao: {e}"),
            }
        }
        // **O baseline do undo é a ÚLTIMA palavra — e sai do MUNDO, não do arquivo.**
        //
        // O `apply_project` armou o baseline com o estado do ARQUIVO; o `restore_painted_docs`
        // mexeu no mundo DEPOIS disso (cada sprite pintado recebe uma textura individual NOVA
        // — o `texture_id` do save morreu com o processo que o criou, e `Sprite` é componente
        // registrado, logo está no snapshot). Mundo ≠ baseline. E o `post_frame_undo` roda no
        // MESMO frame — o Ctrl+O é input, então `any_input_this_frame` é `true` — vê a
        // diferença e registra um passo. Resultado: a fila "nova" nascia com um passo dentro,
        // e o primeiro Ctrl+Z do artista não desfazia a ação dele: re-apontava cada sprite
        // pintado para um `texture_id` morto (ou, na mesma sessão, para a textura do projeto
        // ANTERIOR). Re-armar do mundo aqui é o que faz "histórico novo" ser verdade no FRAME,
        // e não só no retorno desta função. [[feedback_tool_unit_green_integration_dead]]
        self.undo_baseline = self.capture_project();
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
#[path = "project_tests.rs"]
mod tests;
