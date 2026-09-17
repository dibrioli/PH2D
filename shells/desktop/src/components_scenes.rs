//! **O PRÓLOGO das cenas da família das instâncias** — o invólucro que traduz `&mut App` para a
//! assinatura que a [`ph2d_app_components`] expõe.
//!
//! # Por que estas seis funções ficam aqui
//!
//! É a lei que a `line/app-physics` pagou na Fase C e que o `ESTADO_W2` escreve: *o que sai são os
//! CORPOS; o que decide a ordem do quadro fica.* Cada método abaixo é **só** o prólogo — três
//! guardas e a construção do contexto:
//!
//! 1. *já corri?* (o latch, que vive na `App` porque é a shell que sabe o que é «uma vez»);
//! 2. *a env está posta?*;
//! 3. *o mundo já subiu?* — e se não, tenta no quadro seguinte.
//!
//! ⛔ **Nenhuma destas três é uma pergunta da família.** Um latch dentro da crate seria ela a ter
//! opinião sobre **quando** o quadro a chama, e é isso que o `render_loop` decide.
//!
//! ⭐ E é por isso que o `AppHost` não ganhou um sexto método: escrito em **tipos**, o que uma cena
//! pede é o mundo, a cena vectorial, o registo e o relógio — que são tipos de **crates irmãs**, não
//! perguntas à shell. O molde é o `MotionSceneCtx` da `line/app-motion`.

use ph2d_app_components::scene_ctx::SceneCtx;

impl crate::App {
    /// Empresta à família o que uma cena dela toca.
    ///
    /// ⚠️ **Os quatro primeiros campos são do `AppGfx` e os outros da `App`** — campos disjuntos,
    /// logo o compilador aceita os empréstimos simultâneos. É por isso que isto é uma função e não
    /// um `struct` guardado: um contexto **guardado** teria de escolher um dono para o empréstimo.
    pub(crate) fn components_ctx(&mut self) -> Option<SceneCtx<'_>> {
        // ⚠️ Lido ANTES dos empréstimos mutáveis: é outro campo, mas escrevê-lo depois de o `gfx`
        // sair emprestado obrigaria a uma segunda travessia sem necessidade nenhuma.
        let audio_ready = self.audio.is_some();
        let camera_preview = &mut self.game_camera_preview;
        let vec_entities = &mut self.vec.entities;
        let playhead = &mut self.playhead;
        let gfx = self.gfx.as_mut()?;
        Some(SceneCtx {
            sim: &mut gfx.sim,
            vec_scene: &mut gfx.vec_scene,
            vec_entities,
            registry: &gfx.component_registry,
            tags: &mut gfx.tags,
            hero_screen: gfx.hero_screen.as_mut(),
            playhead,
            audio_ready,
            camera_preview,
        })
    }

    /// No prólogo do quadro, uma vez. No-op sem a env.
    pub(crate) fn timer_smoke(&mut self) {
        if self.components.smokes.timer || std::env::var_os("PH2D_TIMER_SMOKE").is_none() {
            return;
        }
        let Some(mut cx) = self.components_ctx() else {
            return; // ainda não há mundo; tenta no quadro seguinte
        };
        ph2d_app_components::timer_smoke::timer_smoke(&mut cx);
        self.components.smokes.timer = true;
    }

    /// No prólogo do quadro, uma vez. No-op sem a env.
    pub(crate) fn signal_action_smoke(&mut self) {
        if self.components.smokes.signal_action
            || std::env::var_os("PH2D_SIGNAL_ACTION_SMOKE").is_none()
        {
            return;
        }
        let Some(mut cx) = self.components_ctx() else {
            return;
        };
        ph2d_app_components::signal_action_smoke::signal_action_smoke(&mut cx);
        self.components.smokes.signal_action = true;
    }

    /// No prólogo do quadro, uma vez. No-op sem a env.
    pub(crate) fn audio_2d_smoke(&mut self) {
        if self.components.smokes.audio_2d || std::env::var_os("PH2D_AUDIO_2D_SMOKE").is_none() {
            return;
        }
        let Some(mut cx) = self.components_ctx() else {
            return;
        };
        ph2d_app_components::audio_2d_smoke::audio_2d_smoke(&mut cx);
        self.components.smokes.audio_2d = true;
    }

    /// No prólogo do quadro, uma vez. No-op sem a env.
    ///
    /// ⚠️ **O latch desta é lido pelo `render_loop`** (é ele que decide mover o herói da cena), ao
    /// contrário dos outros quatro — por isso ele fica na `App` e não podia ser um `thread_local`
    /// da crate.
    pub(crate) fn game_camera_smoke(&mut self) {
        if self.components.smokes.game_camera
            || std::env::var_os("PH2D_GAME_CAMERA_SMOKE").is_none()
        {
            return;
        }
        let Some(mut cx) = self.components_ctx() else {
            return;
        };
        ph2d_app_components::camera_2d_smoke::game_camera_smoke(&mut cx);
        self.components.smokes.game_camera = true;
    }

    /// ⭐⭐⭐ **As TAGS** (TOP-20 #9, W4) — no prólogo do quadro, uma vez. No-op sem a env.
    ///
    /// ⚠️ **O NÍVEL é lido aqui e passado**, ao contrário dos quatro irmãos acima (que são
    /// interruptores): esta env tem duas cenas, e um valor que a crate não reconheça cai na `=1` —
    /// *uma cena ausente ensina menos que uma cena errada, mas um ecrã VAZIO não ensina nada*.
    pub(crate) fn tags_smoke(&mut self) {
        if self.components.smokes.tags {
            self.tags_smoke_traz_o_inspector();
            return;
        }
        let Some(v) = std::env::var_os("PH2D_TAGS_SMOKE") else {
            return;
        };
        let nivel = v.to_str().and_then(|s| s.parse().ok()).unwrap_or(1);
        let Some(mut cx) = self.components_ctx() else {
            return;
        };
        let (cena, sujeito) = ph2d_app_components::tags_smoke::tags_smoke(&mut cx, nivel);
        self.components.smokes.tags = true;
        // ⛔⛔⛔ **A `=1` ABRE COM O HERÓI ESCOLHIDO E O INSPECTOR À FRENTE, e foi o DONO que o
        // disse:** o smoke desta cena mandava ler a secção *Tags* do painel da direita e isso era
        // **impossível** — a cena abria sem selecção, logo o Inspector mostrava o estado vazio e não
        // havia um único chip no ecrã (report de 2026-09-19: *«não faço ideia do que seja»*).
        // *A cena estava certa como DADOS e era impossível como GESTO.*
        //
        // ⚠️ **E a subida do `z` é obrigatória, não defensiva:** a arrumação vive FORA do repositório
        // (`~/.ph2d/layout.txt`), e naquele encaixe pode estar outro painel por cima — foi a FOTO
        // que o disse aqui e no #18. ⇒ o mesmo `bump_panel_z` por alguns quadros, porque o
        // `reconcile_z` acrescenta os painéis em falta no INÍCIO de cada quadro.
        if let Some(sujeito) = sujeito
            && let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut())
        {
            hero.panel_visibility.insert("inspector", true);
            hero.gizmo.selection = Some(sujeito);
            hero.gizmo.extra_selection.clear();
            self.components_smokes.tags_raise = 3;
        }
        // ⭐⭐⭐ **A `=2` é de FÍSICA, e sem isto ela demonstra um mundo CONGELADO.**
        //
        // ⛔⛔ *Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena
        // ausente* (`CLAUDE.md` §5.0): sem o `simulate_physics` armado e sem o play, os dois
        // corpos ficam pendurados no ar e o artista lê *«a armadilha não dispara para ninguém»* —
        // que é exactamente o veredito errado sobre um filtro que funciona.
        //
        // ⚠️ **Aqui e não na cena**, pela lei da Fase C da `line/app-physics`: *o que sai são os
        // CORPOS; o que decide a ordem do quadro fica*. E a **RÉGUA** abre junto, pelo motivo que a
        // cena 67 da física pagou em produto — uma instrução que manda rebobinar sobre um ecrã sem
        // transporte devolve *«que régua?»*.
        if cena == 2 {
            self.timeline.flags.simulate_physics = true;
            if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                hero.panel_visibility.insert("timeline", true);
            }
            self.playhead.rewind();
            self.playhead.play();
        }
    }

    /// Traz o Inspector à frente **e rola-o até à secção das TAGS**, por alguns quadros.
    ///
    /// ⛔⛔ **As duas coisas TÊM de acontecer nos quadros SEGUINTES, e por razões diferentes:**
    ///
    /// - a subida do `z`, porque o `reconcile_z` acrescenta no INÍCIO de cada quadro os painéis que
    ///   faltam na ordem (a lição do #18, medida numa foto);
    /// - a ROLAGEM, porque o pintor **corta** o valor contra `conteúdo − visível`, e esses dois
    ///   números só existem DEPOIS do primeiro desenho do painel: pedi-la no quadro em que a cena
    ///   monta é pedi-la contra um conteúdo de altura `0`, e ela volta cortada ao topo.
    ///   ⚠️ **Foi a FOTO que o disse** — a 1.ª tentativa punha o Inspector à frente e aberto no
    ///   TOPO (*Transform*, *Render Source*, *Color & Tint*, *Sprite Sheet*…), com a secção *Tags*
    ///   fora do ecrã: o passo continuava a ser uma caça.
    ///
    /// ⚠️ O valor é `f32::MAX` de propósito: quem sabe onde o painel acaba é o PINTOR, contra o
    /// conteúdo real daquele objecto — um número escrito aqui seria a segunda resposta à mesma
    /// pergunta, e envelhecia na primeira secção nova.
    fn tags_smoke_traz_o_inspector(&mut self) {
        if self.components_smokes.tags_raise == 0 {
            return;
        }
        self.components_smokes.tags_raise -= 1;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            let insp = ph2d_editor_core::ids::INSP_PANEL;
            hero.store.bump_panel_z(insp);
            // ⭐ **O fim é DERIVADO do que o painel publicou**, nunca um número escolhido: ele
            //    escreve a altura do conteúdo e a da faixa visível ao fim de cada desenho.
            let fim = match (
                hero.store.panel_content_h(insp),
                hero.store.panel_visible_h(insp),
            ) {
                (Some(c), Some(v)) => (c - v).max(0.0),
                _ => return, // ainda não desenhou: tenta no quadro seguinte
            };
            // ⚠️ **As DUAS metades**: o `scroll` é o ALVO autorado e o `scroll_live` é onde a
            //    superfície está agora — escrever só o alvo faz a rolagem suave começar e a cena
            //    abre a meio caminho, porque o contador de quadros acaba antes dela.
            hero.store.set_panel_scroll(insp, fim);
            hero.store.set_panel_scroll_live(insp, fim);
        }
    }

    /// ⭐⭐⭐ **A FÁBRICA e o CICLO DE VIDA** (TOP-20 #11 e #12, W4). Prólogo do quadro, uma vez.
    ///
    /// ⚠️ **As DUAS cenas são de FÍSICA e as duas precisam do relógio A ANDAR**, e por duas razões
    /// diferentes que só juntas se lêem: a física move o que nasce, e **a corrida é o relógio** —
    /// a fábrica é gateada em `playhead.is_playing()`. Sem isto o artista vê uma cena parada e lê
    /// *«a fábrica não faz nada»*, que é o veredito errado sobre uma fábrica que funciona.
    ///
    /// ⛔⛔ *Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente*
    /// (`CLAUDE.md` §5.0).
    pub(crate) fn factory_smoke(&mut self) {
        if self.components.smokes.factory {
            return;
        }
        let Some(v) = std::env::var_os("PH2D_FACTORY_SMOKE") else {
            return;
        };
        let nivel = v.to_str().and_then(|s| s.parse().ok()).unwrap_or(1);
        let Some(mut cx) = self.components_ctx() else {
            return;
        };
        let cena = ph2d_app_components::factory_smoke::factory_smoke(&mut cx, nivel);
        self.components.smokes.factory = true;
        // ⭐⭐⭐ **A `=2` TOMA a vista da câmera do jogo, e sem isso ela ENSINA O CONTRÁRIO.**
        //
        // O *Destroy Outside* mede contra o rectângulo da `GameCamera` — nunca contra a vista do
        // editor (uma corrida não pode depender de onde o artista rolou o ecrã). Com a vista do
        // editor, que é mais larga, as cópias somem **no meio do ecrã** e o artista lê *«elas
        // desaparecem sozinhas»* em vez de *«elas saem do ecrã do jogo»*.
        if cena == 2 {
            self.game_camera_preview = true;
        }
        self.timeline.flags.simulate_physics = true;
        // ⚠️ **A régua abre junto** — uma instrução que manda rebobinar sobre um ecrã sem
        // transporte devolve *«que régua?»* (a lição da cena 67 da física).
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.panel_visibility.insert("timeline", true);
        }
        self.playhead.rewind();
        self.playhead.play();
    }

    /// ⭐⭐⭐ **O CÉREBRO AUTORÁVEL** (TOP-20 #15, W4). Prólogo do quadro, uma vez.
    ///
    /// ⚠️ **A cena precisa do relógio A ANDAR**, e por DUAS razões que só juntas se leem: o botão é
    /// um `Timer`, que corre no **passo fixo**; e a máquina avança no **dreno de sinais**, que só
    /// tem sinais quando alguém os publica. Com o transporte parado o artista vê uma porta imóvel e
    /// lê *«não faz nada»* — que é o veredito errado sobre um componente que funciona.
    ///
    /// ⛔⛔ *Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente*
    /// (`CLAUDE.md` §5.0).
    pub(crate) fn statemachine_smoke(&mut self) {
        if self.components.smokes.statemachine {
            return;
        }
        let Some(v) = std::env::var_os("PH2D_STATEMACHINE_SMOKE") else {
            return;
        };
        let nivel = v.to_str().and_then(|s| s.parse().ok()).unwrap_or(1);
        let Some(cx) = self.components_ctx() else {
            return;
        };
        let _ = ph2d_app_components::statemachine_smoke::montar(cx.sim.world_mut(), nivel);
        self.components.smokes.statemachine = true;
        // ⚠️ A régua abre junto — uma instrução que fala do transporte sobre um ecrã sem ele
        // devolve *«que régua?»* (a lição da cena 67 da física).
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.panel_visibility.insert("timeline", true);
        }
        self.playhead.rewind();
        self.playhead.play();
    }

    /// ⭐⭐⭐ **O SCRIPT DO ARTISTA** (TOP-20 #16, W4). Prólogo do quadro, uma vez — o molde do
    /// cérebro, acima: a cena, a régua aberta, e o relógio a andar (um script só corre a tocar).
    ///
    /// ⚠️ **O ficheiro vive em `~/.ph2d/smoke`**, fora do repositório, porque é para o dono o editar.
    pub(crate) fn script_smoke(&mut self) {
        if self.components.smokes.script {
            return;
        }
        let Some(v) = std::env::var_os("PH2D_SCRIPT_SMOKE") else {
            return;
        };
        let nivel = v.to_str().and_then(|s| s.parse().ok()).unwrap_or(1);
        let Some(cx) = self.components_ctx() else {
            return;
        };
        let dir = ph2d_app_components::script_smoke::default_dir();
        let montada = ph2d_app_components::script_smoke::montar(cx.sim.world_mut(), nivel, &dir);
        self.components.smokes.script = true;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.panel_visibility.insert("timeline", true);
            match montada {
                // ⚠️ O `clear()` anda colado ao `selection` (a lei da cena de física).
                Ok(m) => {
                    hero.gizmo.selection = Some(m.escolhido);
                    hero.gizmo.extra_selection.clear();
                }
                Err(e) => eprintln!(
                    "[script-smoke] nao escrevi o script em {}: {e}",
                    dir.display()
                ),
            }
        }
        self.playhead.rewind();
        self.playhead.play();
    }

    /// ⭐⭐⭐ **O MOVER DE VISTA DE CIMA** (TOP-20 #13, W2). Prólogo do quadro, uma vez.
    ///
    /// ⚠️ **As duas cenas precisam do relógio A ANDAR**, e pela razão do irmão acima: a ponte do
    /// mover corre no **passo fixo**, logo com o transporte parado o boneco não anda e o artista lê
    /// *«as setas não fazem nada»* — que é o veredito errado sobre um componente que funciona.
    ///
    /// ⚠️ **E o TECLADO tem de chegar lá:** o mover lê as acções nomeadas do Input Map
    /// (`move_left`/`move_right`/`move_up`/`move_down`), que o `resolve_player_input` resolve todo
    /// o quadro — as duas primeiras já existiam, as duas últimas nasceram nesta wave.
    pub(crate) fn topdown_smoke(&mut self) {
        if self.components.smokes.topdown {
            return;
        }
        let Some(v) = std::env::var_os("PH2D_TOPDOWN_SMOKE") else {
            return;
        };
        let nivel = v.to_str().and_then(|s| s.parse().ok()).unwrap_or(1);
        let Some(cx) = self.components_ctx() else {
            return;
        };
        let _ = ph2d_app_components::topdown_smoke::montar(cx.sim.world_mut(), nivel);
        self.components.smokes.topdown = true;
        self.timeline.flags.simulate_physics = true;
        // ⚠️ A régua abre junto — uma instrução que fala do transporte sobre um ecrã sem ele
        // devolve *«que régua?»* (a lição da cena 67 da física).
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.panel_visibility.insert("timeline", true);
        }
        self.playhead.rewind();
        self.playhead.play();
    }

    /// ⭐⭐⭐ **O PROJÉCTIL** (TOP-20 #14, W4). Prólogo do quadro, uma vez.
    ///
    /// ⚠️ **As duas cenas precisam do relógio A ANDAR**, e por duas razões que só juntas se lêem:
    /// a ponte do projéctil corre no **passo fixo**, e o voo só COMEÇA quando o primeiro tique
    /// corre. Sem isto o artista vê quatro rectângulos parados e lê *«as balas não saem»* — que é
    /// o veredito errado sobre um componente que funciona.
    ///
    /// ⛔⛔ *Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente*
    /// (`CLAUDE.md` §5.0).
    pub(crate) fn projectile_smoke(&mut self) {
        if self.components.smokes.projectile {
            return;
        }
        let Some(v) = std::env::var_os("PH2D_PROJECTILE_SMOKE") else {
            return;
        };
        let nivel = v.to_str().and_then(|s| s.parse().ok()).unwrap_or(1);
        let Some(cx) = self.components_ctx() else {
            return;
        };
        let _ = ph2d_app_components::projectile_smoke::montar(cx.sim.world_mut(), nivel);
        self.components.smokes.projectile = true;
        self.timeline.flags.simulate_physics = true;
        // ⚠️ A régua abre junto — uma instrução que fala do transporte sobre um ecrã sem ele
        // devolve *«que régua?»* (a lição da cena 67 da física).
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.panel_visibility.insert("timeline", true);
        }
        self.playhead.rewind();
        self.playhead.play();
    }

    /// ⭐⭐⭐ **O EMISSOR DE PARTÍCULAS** (TOP-20 #18) — `PH2D_PARTICLES_SMOKE=1|2`.
    ///
    /// ⚠️ **O prólogo ARMA O RELÓGIO**, e sem ele a cena ensina o contrário do que diz: a corrida
    /// das partículas é o relógio A ANDAR, e sobre um transporte parado as quatro fontes ficam
    /// vazias com todos os números certos — exactamente o que a linha de aviso da secção existe
    /// para explicar. ⚠️ A `=2` precisa TAMBÉM da física (os dois voos são projécteis).
    pub(crate) fn particles_smoke(&mut self) {
        if self.components.smokes.particles {
            self.particles_smoke_traz_o_inspector();
            return;
        }
        let Some(v) = std::env::var_os("PH2D_PARTICLES_SMOKE") else {
            return;
        };
        let nivel = v.to_str().and_then(|s| s.parse().ok()).unwrap_or(1);
        let Some(cx) = self.components_ctx() else {
            return;
        };
        let montada = ph2d_app_components::particles_smoke::montar(cx.sim.world_mut(), nivel);
        self.components.smokes.particles = true;
        self.timeline.flags.simulate_physics = true;
        // ⚠️ A régua abre junto — uma instrução que fala do transporte sobre um ecrã sem ele
        // devolve *«que régua?»* (a lição da cena 67 da física).
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.panel_visibility.insert("timeline", true);
            // ⛔⛔ **O INSPECTOR É TRAZIDO À FRENTE, e foi a FOTO que o disse:** a instrução manda
            // clicar numa fonte e ver a secção *«no painel da direita»*, e naquele encaixe estava o
            // painel do esqueleto por cima — o passo nomeava uma superfície que o dono não tinha à
            // vista. ⚠️ **A arrumação vive FORA do repositório** (`~/.ph2d/layout.txt`), logo isto
            // não é defensivo: é a única forma de a cena não depender do que ficou aberto ontem.
            hero.panel_visibility.insert("inspector", true);
            // ⚠️ O `clear()` anda colado ao `selection` (a lei da cena de física).
            hero.gizmo.selection = Some(montada.escolhido);
            hero.gizmo.extra_selection.clear();
        }
        // ⚠️ A subida acontece nos quadros SEGUINTES — ver o campo `particles_raise`.
        self.components.smokes.particles_raise = 3;
        self.playhead.rewind();
        self.playhead.play();
    }

    /// Traz o Inspector à frente no encaixe dele, por alguns quadros. Ver `particles_raise`.
    fn particles_smoke_traz_o_inspector(&mut self) {
        if self.components.smokes.particles_raise == 0 {
            return;
        }
        self.components.smokes.particles_raise -= 1;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.store.bump_panel_z(ph2d_editor_core::ids::INSP_PANEL);
        }
    }

    /// Prólogo do quadro, uma vez. No-op sem a env.
    pub(crate) fn instance_smoke(&mut self) {
        if self.components.smokes.instance || std::env::var_os("PH2D_INSTANCE_SMOKE").is_none() {
            return;
        }
        let Some(mut cx) = self.components_ctx() else {
            return; // o mundo ainda não subiu; tenta no próximo quadro
        };
        ph2d_app_components::instance_smoke::instance_smoke(&mut cx);
        self.components.smokes.instance = true;
    }

    /// ⚠️ **Depois do quadro e ANTES do `post_frame_undo`**, e as duas metades são a razão:
    ///
    /// - *antes da captura*, senão a escrita do sync chega ao mundo depois da fotografia e vira um
    ///   passo de undo espúrio no quadro seguinte — um passo que o artista não deu;
    /// - *depois do quadro*, porque é aí que as edições do Inspector já foram aplicadas ao mundo
    ///   (`apply_editor_commands` corre no fim do laço). Pô-lo antes faria as instâncias andarem
    ///   **um quadro atrás do mestre**: o artista veria a peça da receita mudar e as cópias
    ///   seguirem depois, que é exatamente o que *«mudam no mesmo quadro»* proíbe.
    ///
    /// ⚠️ **Esta ponte não usa o [`SceneCtx`]**, e a diferença é o assunto: ela passa o `physics` e
    /// o **eco** (`instance_echo`), que não são coisas que uma CENA toque. *A lei já era pura* —
    /// `sync_instances(sim, registry, bridge, echo, docs)` vive na crate desde sempre; o que aqui
    /// estava era só quem lhe entrega os cinco.
    pub(crate) fn sync_instances(&mut self) {
        let vec_entities = &mut self.vec.entities;
        let echo = &mut self.instance_echo;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        // Campos disjuntos do `AppGfx` (+ o eco e o mapa, que são do `App`) — sem clonar nada.
        ph2d_app_components::instance_sync::sync_instances(
            &mut gfx.sim,
            &gfx.component_registry,
            &gfx.physics,
            echo,
            &mut ph2d_app_components::instance_docs::OwnedDocs {
                vec_scene: &mut gfx.vec_scene,
                vec_entities,
            },
        );
    }
}
