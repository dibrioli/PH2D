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
            hero_screen: gfx.hero_screen.as_mut(),
            playhead,
            audio_ready,
            camera_preview,
        })
    }

    /// No prólogo do quadro, uma vez. No-op sem a env.
    pub(crate) fn timer_smoke(&mut self) {
        if self.timer_smoke_done || std::env::var_os("PH2D_TIMER_SMOKE").is_none() {
            return;
        }
        let Some(mut cx) = self.components_ctx() else {
            return; // ainda não há mundo; tenta no quadro seguinte
        };
        ph2d_app_components::timer_smoke::timer_smoke(&mut cx);
        self.timer_smoke_done = true;
    }

    /// No prólogo do quadro, uma vez. No-op sem a env.
    pub(crate) fn signal_action_smoke(&mut self) {
        if self.signal_action_smoke_done || std::env::var_os("PH2D_SIGNAL_ACTION_SMOKE").is_none() {
            return;
        }
        let Some(mut cx) = self.components_ctx() else {
            return;
        };
        ph2d_app_components::signal_action_smoke::signal_action_smoke(&mut cx);
        self.signal_action_smoke_done = true;
    }

    /// No prólogo do quadro, uma vez. No-op sem a env.
    pub(crate) fn audio_2d_smoke(&mut self) {
        if self.audio_2d_smoke_done || std::env::var_os("PH2D_AUDIO_2D_SMOKE").is_none() {
            return;
        }
        let Some(mut cx) = self.components_ctx() else {
            return;
        };
        ph2d_app_components::audio_2d_smoke::audio_2d_smoke(&mut cx);
        self.audio_2d_smoke_done = true;
    }

    /// No prólogo do quadro, uma vez. No-op sem a env.
    ///
    /// ⚠️ **O latch desta é lido pelo `render_loop`** (é ele que decide mover o herói da cena), ao
    /// contrário dos outros quatro — por isso ele fica na `App` e não podia ser um `thread_local`
    /// da crate.
    pub(crate) fn game_camera_smoke(&mut self) {
        if self.game_camera_smoke_done || std::env::var_os("PH2D_GAME_CAMERA_SMOKE").is_none() {
            return;
        }
        let Some(mut cx) = self.components_ctx() else {
            return;
        };
        ph2d_app_components::camera_2d_smoke::game_camera_smoke(&mut cx);
        self.game_camera_smoke_done = true;
    }

    /// Prólogo do quadro, uma vez. No-op sem a env.
    pub(crate) fn instance_smoke(&mut self) {
        if self.instance_smoke_done || std::env::var_os("PH2D_INSTANCE_SMOKE").is_none() {
            return;
        }
        let Some(mut cx) = self.components_ctx() else {
            return; // o mundo ainda não subiu; tenta no próximo quadro
        };
        ph2d_app_components::instance_smoke::instance_smoke(&mut cx);
        self.instance_smoke_done = true;
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
