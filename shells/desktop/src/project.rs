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
/// v14 (W7.5): a **pose da chave** do Flip virou AFIM (`FlipFrame.pose: Pose([f32;6])`, era
/// `offset: Vec2`) — o `ProjectState` embute o `FlipDoc`, e postcard é posicional: 8 bytes → 24
/// por chave posada. Sem o bump um v13 leria os coeficientes do afim como o `Vec2` + lixo.
/// v15 (W8): o traço do Flip ganhou `point_sel` (seleção no domínio Point, FLIP v6→7) —
/// campo novo no `FlipStroke`, layout posicional muda.
/// v16 (ADR-0131 W1): `RigidBody`/`Collider` foram REGISTRADOS no
/// `ComponentRegistry`, então uma cena com corpos físicos grava blobs novos
/// nas linhas do `WorldSnapshot` — um leitor v15 leria esses bytes na posição
/// errada. O `PhysicsBridge` em si NÃO é serializado (é derivado das
/// components no load); só os components viajam.
/// v17 (ADR-0131 W2): o `Collider` ganhou `restitution`/`friction` APENDADOS —
/// campo novo, layout posicional muda.
/// v18 (§4.C.6 do Flip): a **UNIDADE** do `Point.width` do Flip mudou (px de tela → MUNDO,
/// FLIP v7→8). ⚠️ **O layout NÃO mudou** — e é por isso que o bump é obrigatório: postcard lê
/// o `f32` antigo com sucesso e o interpreta na unidade nova, ~100× mais grosso, sem um
/// erro sequer. Todos os bumps anteriores quebravam LAYOUT (falham alto); este quebra
/// SIGNIFICADO (falharia calado). Arquivo v17 é recusado — ver o `load`.
/// v19 (ADR-0131 W2b): o arquivo carrega as **settings de MUNDO** da física
/// (`ProjectFile.physics`) — gravidade, solver, arrasto, sono. Campo novo,
/// layout posicional muda. Sem ele o painel do W2b seria um painel de knobs que
/// ESQUECEM: gravidade zero para um jogo top-down é uma decisão do projeto, e
/// perdê-la ao reabrir é o mesmo que não tê-la.
/// v20 (ADR-0131 W2b, pós-smoke): `PhysicsSettings` ganhou `air_drag` APENDADO —
/// o campo entra no layout de `ProjectFile.physics`. Nenhuma constante de esquema
/// mudou, então **nenhum gate podia ver isto**: postcard é posicional e um save
/// v19 lido como v20 devolveria lixo bem-formado.
/// v21 (ADR-0131 W2c): camadas de colisão — `Collider.layer` APENDADO ao
/// component (blob novo nas linhas do `WorldSnapshot`) **e**
/// `PhysicsSettings.layer_matrix` apendado. Duas quebras de layout no mesmo
/// bump, nenhuma visível a um gate de constante.
/// v22 (ADR-0132): `VecPath` ganhou a pilha de efeitos (Live Path Effects). v23: a entrada da
/// pilha virou `FxEntry` (o efeito + se está LIGADO). v24: a pilha ganhou os variants
/// `Repeat`/`Twist`/`Bloat` — apender variant não move os índices anteriores, então um arquivo
/// v23 continua a ser lido CERTO; o bump existe para que o caminho inverso (um v24 aberto por um
/// binário v23) morra como erro de versão em vez de como um postcard perdido.
/// ⚠️ Estas entradas nasceram como v19..v21 na `line/Vector` e foram **renumeradas +3 na
/// integração de 2026-07-19**, porque a `line/physics` bumpou três vezes na mesma jornada e o
/// contador se **CONTA**, não se escolhe.
/// v27 (ADR-0131 W7): triggers — `Collider.is_sensor` APENDADO ao component (blob novo nas
/// linhas do `WorldSnapshot`), mesmo padrão do v21 (`layer`). Layout posicional muda: um save
/// v26 lido como v27 leria além do fim do blob do `Collider`; um v27 lido por um binário v26 é
/// recusado como erro de versão em vez de virar um postcard perdido.
/// v28 (ADR-0131, Weld): `JointKind` ganhou o variant `Weld` APENDADO (discriminante 3). Apender
/// variant NÃO move os índices anteriores, então um save v27 (Pin/Spring/Rope) continua a ser
/// lido CERTO; o bump existe pro caminho INVERSO — um save com um Weld, aberto por um binário v27,
/// morre como erro de VERSÃO em vez de como um postcard perdido no discriminante 3 desconhecido
/// (mesmo raciocínio do v24, os variants do vetor).
/// v30 (ADR-0131, gold-standard joint anchors): `PhysicsJoint` ganhou `local_a`/`local_b`/`anchored`
/// APENDADOS — a âncora deixou de ser um ponto de MUNDO re-derivado (o `Transform` do joint) e
/// passou a ser autorada BODY-LOCAL por corpo (a rep nativa do rapier), pra a âncora seguir o
/// corpo quando ele se move. Layout posicional muda (mesmo padrão do v27/`is_sensor`): um save v29
/// lido como v30 leria além do fim do blob do `PhysicsJoint`; um v30 lido por um binário v29 é
/// recusado como erro de versão.
/// v31 (Flip, 03 §8): o `FlipStroke` ganhou `tip` + `dot_spacing` (o pincel pontilhado) — campos
/// no MEIO do struct (após `hardness`), então o layout posicional do `FlipDoc` embutido muda. É o
/// mesmo motivo dos bumps anteriores do `FlipStroke` (v7 `holes`/`hide_stroke`, v8 `selected`):
/// `FLIP_SCHEMA_VERSION` 8→9, e `PROJECT_SCHEMA` acompanha porque o `FlipDoc` viaja DENTRO do
/// `ProjectState`.
///
/// ⚠️ Este bump nasceu `30` na `line/FLIP` e virou **31** na integração de 2026-07-25: a
/// `line/physics` reivindicou o mesmo 30 na MESMA janela, por outro motivo (a âncora body-local
/// do joint, o parágrafo acima). O valor certo se CONTA, não se escolhe — ele não estava em
/// nenhum dos dois lados do conflito ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
/// v32 (ADR-0131, W-J6 servo + guincho): `PhysicsJoint` ganhou `motor_mode` +
/// `motor_target` APENDADOS — o motor deixou de ser só uma TAXA e passou a poder
/// mirar um LUGAR, e passou a existir também no Slider e na Rope. Mesmo padrão
/// posicional do v30: dois campos a mais no fim do blob, então um save v31 lido
/// como v32 leria além do fim dele, e um v32 lido por um binário v31 é recusado
/// como erro de versão em vez de virar um postcard perdido.
/// v33 (ADR-0131, W-J7 break force): `PhysicsJoint` ganhou `break_enabled` +
/// `break_force` + `break_torque` APENDADOS — um joint pode ser autorado para
/// ROMPER sob carga. Mesmo padrão posicional do v30/v32: três campos a mais no
/// fim do blob. ⚠️ O `∞ = off` NÃO é serializado: o componente guarda um
/// booleano e dois números finitos, e a ponte é quem os resolve na infinidade
/// que o solver quer — guardar `f32::INFINITY` faria o painel ter de mostrar
/// "inf" numa row numérica.
/// v34 (ADR-0131, W-J8 higiene do par): `PhysicsJoint` ganhou `active` +
/// `collide_connected` APENDADOS — desligar a restrição sem apagar o objeto, e
/// escolher se os dois corpos que ela une ainda se batem. Mesmo padrão
/// posicional do v30/v32/v33: dois campos a mais no fim do blob. ⚠️ O **Swap
/// A↔B** da mesma wave NÃO move nada aqui — ele reescreve campos que já
/// existem (as duas pontas, as duas âncoras, e os sinais medidos entre elas),
/// que é exatamente por que um bump se CONTA em vez de acompanhar a wave.
/// v35 (Flip, 2.5D multiplane, ADR-0114 §Decisão 3): a `FlipLayer` ganhou `depth` (a fração de
/// paralaxe da câmera) APENDADO — o `FlipDoc` viaja no `ProjectState`, então o layout posicional
/// muda e um save v34 lido como v35 leria `depth` além do fim do buffer. `FLIP_SCHEMA_VERSION`
/// 9→10, e o `PROJECT_SCHEMA` acompanha. ⚠️ A `line/FLIP` escreveu **32** aqui e a
/// `line/physics` reivindicou o MESMO 32 na mesma janela (o servo do W-J6) — a SEGUNDA vez
/// que estas duas linhas colidem no mesmo número, depois do 30 de 25/07. O valor certo se
/// CONTA a partir do `main` do dia (34 + 1), e não estava em nenhum dos dois lados.
/// v36 (Flip, Self Overlap, 03 §8): o `FlipStroke` ganhou `self_overlap` (auto-sobreposição com
/// acúmulo) no MEIO do struct (após `dot_spacing`) ⇒ layout posicional muda, um save v35 leria os
/// campos seguintes deslocados. `FLIP_SCHEMA_VERSION` 10→11.
/// v37 (Flip, Airbrush, 03 §8): o `FlipStroke` ganhou `airbrush` (falloff físico Beer-Lambert por
/// dab esférico) no MEIO do struct (após `self_overlap`) ⇒ mesmo raciocínio posicional.
/// `FLIP_SCHEMA_VERSION` 11→12.
/// v38 (Vector, plano 24 W6 — a LEI DE MISTURA por degrau): o `ph2d_ecs::FxOp` ganhou `blend`
/// APENDADO — um degrau da pilha de FX raster passa a dizer *como a cor dele encosta na que já
/// está ali* (Inner Shadow em Multiply escurece em vez de lavar; Color Overlay em Color troca a
/// matiz preservando a luminosidade). O `VecFilter` é componente registado, e postcard é
/// POSICIONAL, então um save v37 lido como v38 leria `blend` além do fim de cada degrau.
/// ⚠️ Não há como evitar o bump com `serde(default)`: o postcard não tem NOMES de campo, e um
/// buffer que acaba cedo é erro de decode, não um default.
/// ⚠️ O valor se CONTA a partir do `main` do dia — a `line/physics` e a `line/FLIP` já colidiram
/// DUAS vezes nesta escada (o 30 de 25/07 e o 32 de 27/07). Se a integração encontrar outro dono
/// para o 38, este é o que anda ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
const PROJECT_SCHEMA: u32 = 38;

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
            physics: self
                .gfx
                .as_ref()
                .map(|g| g.physics.settings())
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
            self.toast(format!(
                "Project refused: file format {ver}, this build reads {PROJECT_SCHEMA}"
            ));
            return;
        }
        // A ANIMAÇÃO É PARTE DO ARQUIVO, e um documento que este binário não sabe ler faz o
        // load inteiro ser RECUSADO — não "abre sem a animação".
        //
        // Abrir sem ela seria a pior das opções: a cena aparece, parece certa, a timeline
        // aparece vazia, e o próximo Ctrl+S grava esse vazio POR CIMA do arquivo. A animação
        // não some por um bug — some porque o app abriu, mentiu e salvou. Recusar é a única
        // leitura honesta (a mesma regra da versão do projeto, logo acima), e o parse vem ANTES
        // de qualquer mutação da sessão, então a recusa não custa nada ao documento aberto.
        let timeline = match crate::timeline_persist::install_from_project(&file.timeline) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[proj] timeline ilegivel — load RECUSADO: {e}");
                self.toast(format!(
                    "Project refused: its animation is from another version ({e})"
                ));
                return;
            }
        };
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
        // …e o Motion re-ENTRA. O auto-play dele é edge-triggered na entrada na tool, e um load
        // não muda a tool — muda o DOCUMENTO. Sem isto, quem estivesse com o Motion aberto abriria
        // um projeto e veria o grafo congelado em t=0 até apertar Space (o `pause` acima é novo).
        crate::render_loop::motion_bridge::forget_tool_transition();
        self.undo = ProjectUndo::default(); // documento novo, histórico novo
        // O mundo rígido é DERIVADO das components (ADR-0131 D2), então o do
        // documento anterior morre aqui — o `reconcile` do próximo frame o
        // re-deriva das components carregadas (entidades novas, bits novos).
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.physics.rebuild();
            // ...e as settings do ARQUIVO entram DEPOIS do rebuild, nunca antes:
            // `rebuild` constrói um `PhysicsWorld` novo, que nasce nos defaults
            // do motor. Instalar antes seria escrever num mundo que o rebuild
            // joga fora, e a cena carregaria com a gravidade do documento
            // ANTERIOR — em silêncio.
            gfx.physics.set_settings(file.physics);
        }
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
        // O memo do Offset vivo — pela razão do restore de undo: os `VecPathId` são reciclados
        // entre documentos, e um acerto de memo desenharia o offset do projeto ANTERIOR.
        self.offset_live.forget();
        // O Pattern vivo pela MESMA razão (id reciclado entre documentos).
        self.pattern_live.forget();
        self.contour_live.forget();
        // O FX raster vivo (plano 24) pela MESMA razão (id reciclado entre documentos).
        self.fx_live.forget();
        self.fx_silhouette.forget();
        self.vec_offset_mirrored = None;
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
        // **A ANIMAÇÃO** (W4.T6/B5). Já parseada lá em cima (um documento ilegível recusa o
        // arquivo INTEIRO, antes de tocar na sessão). As bindings entram DESTACADAS — `entity`
        // zerada — e o `upkeep` do frame as recola nos objetos que o `apply_project` acabou de
        // spawnar, pelo hash do `Name`: a MESMA função que cura delete+undo. Por isso este
        // passo não precisa do mundo, e por isso o load não pode divergir do undo — a
        // resolução por nome existe uma vez só.
        let tracks = timeline.doc.bindings().len();
        self.timeline = timeline;
        // **O LOOP mora no CLIP** e é POR-VISTA (`NamedClip.loop_range` para a timeline,
        // `keys_loop_range` para o relógio do clip), e cada `Playhead` é só a cópia viva.
        // Publicamos OS DOIS: sem isso o loop salvo nunca voltava — e, pior, o loop do projeto
        // ANTERIOR continuava armado no transporte, fazendo o projeto novo repetir sobre um
        // intervalo de um arquivo que o artista já fechou. Cada relógio adota o loop da SUA vista.
        ph2d_timeline::sync_transport_loop(&self.timeline.doc, &mut self.playhead, false);
        ph2d_timeline::sync_transport_loop(&self.timeline.doc, &mut self.clip_playhead, true);
        // **O baseline do undo fica DESARMADO** — e é o `post_frame_undo` que o arma, depois
        // que o frame assentar.
        //
        // Capturá-lo aqui seria capturar um mundo que ainda não terminou de virar este projeto:
        // o `restore_painted_docs` troca o `texture_id` de cada sprite pintado (o do save morreu
        // com o processo que o criou), e a animação recém-instalada só escreve a pose quando as
        // bindings recolam — um frame depois. Qualquer baseline tirado agora estaria errado por
        // uma dessas duas razões, e o `post_frame_undo` do MESMO frame (o Ctrl+O é input) veria a
        // diferença e gravaria um passo espúrio: o primeiro Ctrl+Z do artista não desfazia a
        // ação dele — devolvia uma textura morta, ou a pose do arquivo.
        //
        // `None` significa "ainda não sei", e o `post_frame_undo` já sabe o que fazer com isso:
        // arma o baseline com o mundo assentado e NÃO registra passo. A fila nova nasce vazia de
        // verdade. [[feedback_tool_unit_green_integration_dead]]
        self.undo_baseline = None;
        eprintln!("[proj] carregado: {path} ({tracks} track(s) de animacao)");
        self.toast(if tracks == 0 {
            "Project loaded".to_string()
        } else {
            format!("Project loaded · {tracks} animation track(s)")
        });
    }

    /// Um aviso na tela — não só no terminal. O Ctrl+O é destrutivo (troca a cena, zera o undo)
    /// e o Ctrl+S é silencioso; um `eprintln!` num app de janela é uma mensagem para ninguém.
    fn toast(&mut self, msg: String) {
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.toasts.push(ph2d_editor::Toast::info(msg));
        }
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
