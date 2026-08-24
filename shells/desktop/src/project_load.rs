//! **O LADO DA LEITURA** de um arquivo de projeto — irmão de [`super`] pelo teto
//! de LOC (HR-18, 600), e o corte é de responsabilidade: lá *o que um arquivo É*
//! e como ele é ESCRITO, aqui *como ele é LIDO e a sessão esquece o anterior*.
//!
//! Filho (`#[path]`) porque a leitura usa o que o pai possui — a forma do
//! `ProjectFile`, o `PROJECT_SCHEMA` e o `toast` —, e um módulo irmão de
//! verdade obrigaria a abrir a privacidade de tudo isso só para movê-lo.
//!
//! ⚠️ **O fio condutor: um load faz a sessão ESQUECER o documento anterior.** O
//! relógio, a fila de undo, o baseline do undo, a timeline, o alvo vivo do Flip
//! e os pins do autokey são todos re-armados aqui — cada um por um defeito que
//! já aconteceu, e cada um nomeado no ponto onde é resolvido.

use super::*;

impl crate::App {
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
        // ⭐ **A PRIMEIRA migração da história do repo** (ADR-0164 F1). Até esta wave a
        // política de facto era *"versão diferente = recusado"* — a auditoria de 21/08
        // registou-a como ambiguidade em aberto (§8 item 7: HR-14 exige `migrate_vN_to_vN+1`
        // e havia **zero**).
        //
        // ⚠️ A versão tem de ser lida **antes** do resto: o postcard é posicional, então
        // desserializar bytes v95 com o tipo v96 não dá erro — dá lixo. Por isso o `ver` sai
        // sozinho primeiro, e só depois se escolhe o tipo com que ler o corpo.
        let ver: u32 = match postcard::take_from_bytes::<u32>(&bytes) {
            Ok((v, _)) => v,
            Err(e) => {
                eprintln!("[proj] erro ao ler a versao de {path}: {e}");
                return;
            }
        };
        let (mut file, migrated_counter) = match ver {
            PROJECT_SCHEMA => match postcard::from_bytes::<(u32, ProjectFile)>(&bytes) {
                Ok((_, f)) => (f, None),
                Err(e) => {
                    eprintln!("[proj] erro ao ler {path}: {e}");
                    return;
                }
            },
            95 => {
                match postcard::from_bytes::<(u32, crate::project_migrate::ProjectFileV95)>(&bytes)
                {
                    Ok((_, old)) => {
                        let m = crate::project_migrate::migrate_v95_to_v96(old);
                        eprintln!(
                            "[proj] migrado v95 -> v{PROJECT_SCHEMA} ({} objetos receberam identidade)",
                            m.file.state.world.entities.len()
                        );
                        self.toast(format!(
                            "Project migrated from format 95 to {PROJECT_SCHEMA}"
                        ));
                        (m.file, Some(m.stable_id_counter))
                    }
                    Err(e) => {
                        eprintln!("[proj] v95 ilegivel: {e}");
                        self.toast(format!(
                            "Project refused: format 95 file is unreadable ({e})"
                        ));
                        return;
                    }
                }
            }
            _ => {
                eprintln!("[proj] schema {ver} != {PROJECT_SCHEMA} — recusado");
                self.toast(format!(
                    "Project refused: file format {ver}, this build reads {PROJECT_SCHEMA}"
                ));
                return;
            }
        };
        // ⚠️ **A semente do contador.** Num ficheiro v96 ela vem do campo; num migrado, da
        // contagem de linhas. Sem ela a primeira entidade criada depois do load reusaria um id
        // que já está vivo — e o `reconcile_at_least` do próprio contador é a segunda rede,
        // porque ele também se compara com os ids que o mundo de facto tem.
        let stable_id_seed = migrated_counter.unwrap_or(file.stable_id_counter);
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
        // **E A ESCULTURA, pela MESMA lei** (ADR-0150 W8.3): um documento que este
        // binário não sabe ler faz o load inteiro ser RECUSADO. Abrir sem ela mostraria
        // a cena, pareceria certo, e o próximo Ctrl+S gravaria o vazio por cima — a obra
        // não sumiria por um bug, sumiria porque o app abriu, mentiu e salvou.
        //
        // ⚠️ Ela é decodificada AQUI e o resultado é guardado: a decodificação é
        // `O(vértices)` com octree e adjacência POR NÍVEL, e fazê-la de novo na hora de
        // instalar seria pagar duas vezes pelo mesmo documento.
        #[cfg(feature = "sculpt3d")]
        let sculpt = if file.sculpt.is_empty() {
            None
        } else {
            match crate::sculpt3d::decode_doc(&file.sculpt) {
                Ok(v) => Some(v),
                Err(e) => {
                    eprintln!("[proj] escultura ilegivel — load RECUSADO: {e}");
                    self.toast(format!(
                        "Project refused: its sculpture is from another version ({e})"
                    ));
                    return;
                }
            }
        };
        // **E OS PIXELS PRÓPRIOS DOS SPRITES, pela MESMÍSSIMA lei** (plano
        // `docs/Sprite_projeto/17` §3). Aqui ela é ainda mais afiada, porque isto **são os
        // pixels**: abrir sem eles mostraria a cena com os sprites em branco — ou invisíveis —,
        // pareceria um bug de render, e o próximo Ctrl+S gravaria o vazio por cima da arte. O
        // parse vem ANTES de qualquer mutação da sessão, então recusar não custa nada ao
        // documento aberto.
        let (sprite_pixels, sprite_sheets) = match ph2d_sprite_sheet::decode(&file.sprite_pixels) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[proj] pixels de sprite ilegiveis — load RECUSADO: {e}");
                self.toast(format!(
                    "Project refused: its sprite images are from another version ({e})"
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
        // **A TABELA DE COR do documento anterior morre aqui, e a do arquivo entra** (W6) —
        // detalhe e razão no irmão `project_tokens`.
        // O número de descartes é DITO pelo próprio `install` (uma voz); aqui ele não é decisão.
        let _ = crate::project_tokens::install(&file.tokens);
        // **AS SETTINGS do documento anterior morrem aqui, e as do arquivo entram**
        // (doc 88, D3) — a escala do mundo e a unidade que o artista lê. Como a
        // tabela de cor acima, instalar é SOBRESCREVER: sem isto, abrir um projeto
        // de fábrica depois de um afinado em `32 px/m` deixaria o app medindo tudo
        // pela escala do documento ANTERIOR, e nada na tela diria porquê.
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            crate::project_settings::install(&mut hero.project, &file.settings);
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
        // E as poses que uma expressão ainda DEVE de volta (`expr_owed`, ADR-0144): elas são
        // chaveadas por `AnimTarget`, e o documento novo aloca os SEUS targets a partir do
        // zero — então uma nota do projeto A seria entregue ao binding do projeto B que
        // herdou o número. É a MESMA razão da linha acima, um nível abaixo.
        ph2d_timeline::expr_owed::forget_owed_poses();
        self.forget_live_producers();
        // E as ESCULTURAS que o módulo de modelagem 3D já tentou ler (ADR-0161 W23). A **peça**
        // atravessa o arquivo sozinha — ela é uma árvore de entidades, e o `ProjectState` é o mundo
        // inteiro. O que não atravessava era a memória de *"já tentei este arquivo"*: ela existe
        // para o aviso não repetir em todo quadro, e o limite dela é o **documento**.
        //
        // ⚠️ Sem esta linha, um arquivo de escultura CONSERTADO no disco nunca era relido — e o
        // segundo silêncio era idêntico ao de quando estava tudo certo. Mesma família das duas
        // linhas acima: *o que o documento anterior possuía e não pode atravessar*.
        crate::field3d_reload::forget_tried();
        // E o ISOLAMENTO da vista (W43): desde que a vista sobrevive a fechar o painel, ela pode
        // atravessar um Ctrl+O — e o `isolated` guarda **bits de entidade**, que o mundo novo
        // realoca. Ver [`crate::field3d_smoke::forget_isolation_across_documents`]: a câmera fica,
        // este campo não.
        crate::field3d_smoke::forget_isolation_across_documents();
        // ⭐⭐ **E o projeto que traz uma PEÇA de modelagem abre o painel dela** (W45).
        //
        // ⚠️ A lei é a do módulo irmão, **lida e não decidida**: *"um projeto com escultura ARMA o
        // módulo… a alternativa seria abrir o arquivo, descartar a obra em silêncio"*. Aqui a obra
        // não se perde — ela é uma árvore de entidades e o save leva o mundo inteiro (W35) — mas o
        // **silêncio** era o mesmo: o arquivo abria e a tela ficava vazia.
        //
        // ⛔ **E a porta estava trancada por dentro:** o pedido de abrir o painel só era aceite com
        // o módulo já **armado**, e o único caminho que o arma é a visibilidade do painel.
        //
        // ⚠️ Só uma **PERGUNTA**, respondida no quadro: o mundo vive no `gfx`, e este caminho corre
        // sem janela (`apply_project` volta cedo). Ver `field3d_smoke::ask_open_panel_if_part`.
        //
        // ⚠️ **E cede a um projeto que também traz ESCULTURA**: os dois querem o canvas, e a lei do
        // dono único (W40) diz que ele é de um só. Quem chegou pelo mesmo arquivo não se disputa —
        // a escultura já arma o módulo dela, e o MODEL fica a um clique.
        //
        // ⚠️⚠️ **A pergunta é feita DEPOIS de a escultura deste load ser instalada**, e a primeira
        // escrita fazia-a antes: ali o `sculpt3d_pending` ainda é o do **documento anterior**, e a
        // condição respondia sobre o arquivo errado — verde em todo gate que abrisse um projeto de
        // cada vez. *Uma condição sobre estado mutável tem um instante, e ele faz parte da lei.*
        // **A ESCULTURA do documento anterior morre aqui.** Os bytes ficam para o save
        // (o passa-adiante), e a cena viva é substituída — ou APAGADA, quando o projeto
        // novo não tem escultura nenhuma: a lista nunca-vazia é o invariante que torna
        // `obj()` total, então "sem peças" se diz com a cena inteira fora, não com uma
        // cena vazia.
        self.sculpt_doc = std::mem::take(&mut file.sculpt);
        #[cfg(feature = "sculpt3d")]
        {
            self.sculpt3d_pending = sculpt;
            if self.sculpt3d_pending.is_none()
                && let Some(gfx) = self.gfx.as_mut()
            {
                gfx.sculpt3d = None;
            }
        }
        // Agora sim — ver o bloco acima sobre o **instante** em que esta pergunta é feita.
        #[cfg(feature = "sculpt3d")]
        let solo = self.sculpt3d_pending.is_none();
        #[cfg(not(feature = "sculpt3d"))]
        let solo = true;
        if solo {
            crate::field3d_smoke::ask_open_panel_if_part();
        }
        // ⭐ **E a peça nasce ENQUADRADA** (W46), abra o painel agora ou daqui a uma hora: o pedido
        // fica de pé até a ponte o servir, e ela só corre com o módulo armado. ⚠️ Sem `solo`, de
        // propósito — enquadrar não disputa o canvas com ninguém.
        crate::field3d_smoke::ask_frame_the_part();
        self.timeline_insert_key = false;
        self.timeline_reveal_after_apply = false;
        self.autokey = Default::default(); // pins/baselines de pose keyados por bits mortos
        self.materialize_assets(&file.assets);
        self.apply_project(&file.state);
        // ⚠️ **A semente do contador entra AQUI, logo depois de o mundo existir** (ADR-0164
        // F1). Antes do `apply_project` não haveria mundo onde a pôr; muito depois, uma
        // entidade criada no meio já teria consumido um id vivo.
        //
        // `reconcile_at_least` e não uma escrita crua: ele **sobe, nunca desce**. Um ficheiro
        // com o contador atrasado (editado à mão, vindo de um branch antigo, ou migrado de um
        // v95 cujo mundo tinha mais linhas do que a conta diz) é corrigido contra os ids que o
        // mundo de facto tem — que é a segunda rede da mesma invariante.
        if let Some(gfx) = self.gfx.as_mut() {
            let mut counter = gfx
                .sim
                .world()
                .get_resource::<ph2d_ecs::StableIdCounter>()
                .copied()
                .unwrap_or_default();
            counter.reconcile_at_least(stable_id_seed);
            gfx.sim.world_mut().insert_resource(counter);
            // ⭐ **E as referências da física passam a apontar por IDENTIDADE** (ADR-0164 F1).
            //
            // Um ficheiro v95 guarda `body_a = hash(nome)`; a ponte, desde esta wave, resolve
            // por `StableId`. Sem esta tradução um projeto antigo abriria com **todas as
            // juntas soltas** — a cena apareceria certa e nada prenderia.
            //
            // ⚠️ Só na rota da MIGRAÇÃO: um ficheiro v96 já guarda identidades, e a função é
            // idempotente de qualquer maneira (um `StableId` é pequeno e sequencial; o mapa só
            // tem hashes FNV-1a, que são grandes).
            if migrated_counter.is_some() {
                let n = ph2d_physics_ecs::resolve_body_names(gfx.sim.world_mut());
                if n.total() > 0 {
                    eprintln!(
                        "[proj] migracao: {} junta(s) e {} roldana(s) passaram a apontar por identidade",
                        n.joints, n.wheels
                    );
                }
            }
        }
        // **OS PIXELS PRÓPRIOS** (plano `docs/Sprite_projeto/17` §3) — depois do mundo, porque é
        // pelo `SpritePixels` (que viaja no snapshot) que cada sprite reencontra os bytes que
        // eram dele. Fecha a perda que atingia TODA ferramenta de imagem: o `texture_id` do save
        // é um id de alocação da GPU e o store recomeça em `1` a cada processo.
        //
        // ⚠️ **A ORDEM É A PRECEDÊNCIA, e é de propósito que ele vem PRIMEIRO:** um sprite
        // pintado ou assado tem documento mais rico, e os dois restores abaixo escrevem por cima
        // deste. (A colheita já os salta, então na prática não há duplo trabalho — mas a ordem é
        // o que torna isso verdade mesmo para um arquivo salvo por um binário anterior.)
        self.restore_sprite_pixels(sprite_pixels);
        // As FOLHAS hand-packed, pelo mesmo motivo e na mesma janela: uma folha sobe UMA vez
        // e os N sprites dela reatam a textura partilhada + o retangulo da regiao.
        self.restore_sprite_sheets(sprite_sheets);
        // Depois do mundo: os sprites já existem (com bits novos), e é pelo `PaintedDoc` que cada um
        // reencontra o documento que era dele.
        self.restore_painted_docs(file.painted);
        // **OS CANAIS ASSADOS** (`docs/3D/02.2`, rota A) — mesma dança, mesmo motivo: o
        // `texture_id` do save morreu com o processo que o criou, e é pelo `BakedForm` que cada
        // sprite reencontra os canais que eram dele.
        //
        // ⚠️ Ele NÃO acende: o slot nasce vazio e quem o acende é o passe de re-acendida no
        // primeiro frame, que é a MESMA porta que a lâmpada usa. Acender aqui seria a segunda
        // resposta a *como um objeto assado vira pixels*.
        self.restore_baked_forms(file.baked_forms);
        // **A CORRIDA GRAVADA** (W17). Instalada, nunca fundida: um load é uma
        // troca de documento, e uma fita costurada com a da sessão anterior
        // descreveria uma corrida que ninguém deu — o irmão exato do que o
        // `project_forget` faz com o relógio, a fila de undo e a timeline.
        self.player_tape = ph2d_physics_ecs::InputTape::from_wire(&file.player_tape);
        // **O INPUT MAP** (v96). Instalado, nunca fundido — pelo motivo da fita logo acima: um
        // load é uma troca de DOCUMENTO, e costurar o mapa do projecto novo com o do anterior
        // deixaria acções de um jogo a viver dentro de outro. Um projecto ≤ v95 nunca chega aqui
        // (o schema recusa antes), então o vazio que isto instala é sempre um vazio autorado.
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.input_map = file.input_map.clone();
            // ⛔⛔ **AS LINHAS TÊM DE SER RE-REGISTADAS** — auditoria 2026-08-24, apanhado por duas
            // lentes. Com a janela aberta, abrir um projecto trocava o mapa e deixava o
            // `WidgetStore` com os widgets das linhas do documento ANTERIOR: as linhas novas eram
            // pintadas e ficavam **mortas sob o ponteiro**.
            let map = hero.input_map.clone();
            ph2d_editor::screens::hero::chrome::sync_input_map_rows(&mut hero.store, &map);
            // ⛔ **E a ESCUTA morre com o documento.** O `ActionId` é um contador POR-MAPA: uma
            // escuta armada em `jump` do projecto anterior re-aponta, no mapa novo, para **outra
            // acção** com o mesmo número — e a próxima tecla ligar-se-ia a ela, em silêncio.
            hero.store.stop_listening();
            // A rolagem também: a lista nova tem outro tamanho, e uma rolagem herdada abre a
            // janela num vazio.
            hero.store.scroll_input_map(f32::NEG_INFINITY, 0.0);
        }
        // ⚠️ E o estado RESOLVIDO zera junto: ele guarda um tique atrás, e o tique atrás de um
        // documento que acabou de fechar é de outro jogo — uma borda `just_pressed` fantasma no
        // primeiro quadro depois do load.
        self.input_actions = ph2d_input::ActionState::new();
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
        // **O `autoplay` faz o que promete AQUI** (spec Sprite 08 §8.4): o projeto acabou de
        // abrir, e é este o único momento em que «começar a tocar» é uma aresta observável.
        //
        // ⚠️ Ele não pode viver no tique — lá só há estados, e detetar a aresta pediria um bit
        // «já comecei» que o primeiro `Ctrl+Z` dessincronizaria.
        // ⚠️ E fica ACIMA do `undo_baseline = None` de propósito: o baseline é armado pelo
        // `post_frame_undo` com o mundo já assente, e este passo é parte de o assentar.
        if let Some(gfx) = self.gfx.as_mut() {
            crate::render_loop::start_autoplay_animations(&mut gfx.sim);
        }
        self.undo_baseline = None;
        eprintln!("[proj] carregado: {path} ({tracks} track(s) de animacao)");
        self.toast(if tracks == 0 {
            "Project loaded".to_string()
        } else {
            format!("Project loaded · {tracks} animation track(s)")
        });
    }
}
