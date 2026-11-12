//! **Fase do quadro: A CÂMERA DE JOGO** (TOP-20 #7) — o herói da cena de smoke da câmera anda nos mesmos
//! tiques que ela, a câmera segue o mundo DESTE quadro, a vista só é tomada com a pré-visualização ligada,
//! e o relatório fala uma vez por mudança (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ **Depois dos relógios e ANTES do extract**, e as duas metades são load-bearing (ver o corpo). Os
//! dois parâmetros são `Copy` e o quadro continua a lê-los mais abaixo.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    ///
    /// ⭐ **Devolve o rectângulo VISÍVEL da câmera do jogo** (centro e meia-janela, em metros), que
    /// é o que o `DestroyOutside` (TOP-20 #12) mede. ⛔ **Nunca a vista do editor:** uma corrida que
    /// dependesse de onde o artista rolou o ecrã seria outra corrida em cada máquina. `None` = não
    /// há câmera de jogo, e então o fora-do-ecrã não mede nada — o painel di-lo.
    pub(super) fn fase_game_camera(
        &mut self,
        player_input: ph2d_physics_ecs::PlayerInput,
        report: ph2d_core::FixedStepReport,
    ) -> Option<([f32; 2], [f32; 2])> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            surface,
            sim,
            camera,
            hero_screen,
            vello_pass,
            miniatura,
            ..
        } = FrameGfx::of(gfx);
        // O canvas VISÍVEL (entre os painéis; a janela inteira no palco) e, dentro dele, a caixa do JOGO (o formato
        // do projecto, plano GEF 17): a câmara do jogo mede-se e encaixa-se nela (`ph2d_app_game::encaixe`).
        let formato = hero_screen
            .as_ref()
            .map(|h| h.project.game_format)
            .unwrap_or_default();
        let (janela, canvas, visivel) = {
            let w = surface.size();
            let full = [0.0, 0.0, w.width as f32, w.height as f32];
            let c = hero_screen.as_ref().map(|h| h.last_canvas);
            let c = c
                .filter(|r| r.w > 0.0 && r.h > 0.0)
                .map_or(full, |r| [r.x, r.y, r.w, r.h]);
            (
                [full[2], full[3]],
                ph2d_app_game::encaixe::caixa_do_jogo(c, formato),
                c,
            )
        };
        let aspecto = formato.aspect();
        let segue = hero_screen
            .as_ref()
            .is_some_and(|h| h.view.camera_canvas.follows);

        // ⭐ **O herói da cena de smoke da câmera anda AQUI**, imediatamente antes do passe dela —
        // ver o doc do [`ph2d_app_components::camera_2d_smoke`] sobre porque ele é movido pelo TECLADO e não pelo
        // rato (um arrasto ancorado na vista realimenta uma câmera que segue). No-op sem a cena.
        if self.components.smokes.game_camera {
            ph2d_app_components::camera_2d_smoke::drive_smoke_hero(
                sim,
                player_input,
                // ⭐ **Os MESMOS `ticks` que a câmera recebe uma linha abaixo** — o herói e a
                // câmera que o segue partilham o relógio, senão a diferença entre os dois é
                // desenhada como tremor. Ver o doc do `camera_2d_smoke`.
                report.ticks,
                self.fixed_step.fixed_dt(),
            );
        }

        // ⭐⭐⭐ A CÂMERA DE JOGO e o ABANÃO: depois dos relógios (segue o mundo DESTE quadro, latência
        // zero ao contacto) e ANTES do extract (que lê a `camera`). Não escrevem componente registado,
        // logo não passam pelo `preview_drive`. O `dt` é o do passo fixo × tiques: em pausa congela.
        let passo = self.fixed_step.fixed_dt() * f64::from(report.ticks);
        let abanao = ph2d_app_components::shake_bridge::drive_camera_shake(
            sim,
            &self.signals,
            &mut self.signal_readers.shake,
            passo as f32,
        );
        let (vista_da_cena, camera_report) = ph2d_app_components::camera_2d::update(
            sim,
            aspecto,
            report.ticks,
            self.fixed_step.fixed_dt(),
            abanao.offset,
        );
        // ⭐⭐ **A vista só é TOMADA com a pré-visualização ligada.** Sem isto, toda cena que tenha
        // uma câmera roubaria o pan e o zoom do artista no primeiro quadro — e a `GameCamera` é um
        // componente que se anexa pela paleta, então isso aconteceria por acidente.
        if self.game_camera_preview
            && let Some(v) = vista_da_cena
        {
            // O palco ALARGA (`ph2d_camera_frame::palco`): a caixa fica, a cerca mede a área visível.
            let palco = hero_screen.as_ref().is_some_and(|h| h.view.palco_do_jogo);
            let area = ph2d_app_game::encaixe::area_do_palco(visivel, formato);
            let limites =
                palco.then(|| ph2d_camera_frame::palco::limites_da_activa(sim.world_mut()));
            let c = ph2d_app_game::encaixe::centro_no_palco(
                limites.flatten().as_ref(),
                v.center,
                camera_report.center,
                v.height_world,
                canvas,
                area,
            );
            let (centro, altura) =
                ph2d_app_game::encaixe::na_janela(c, v.height_world, janela, canvas);
            camera.center = centro;
            camera.height_world = altura;
            camera.cull_mask = v.cull_mask;
        } else if segue && let Some(v) = vista_da_cena {
            // «Editor View: Follows» (onda 18): o centro é o do jogo, o zoom fica o do artista.
            camera.center = ph2d_app_game::encaixe::centro_que_segue(
                v.center,
                camera.height_world,
                janela,
                canvas,
            );
        }
        // ⭐ O rectângulo que o fora-do-ecrã mede, derivado da MESMA vista que o extract desenha —
        // ⛔ e não de uma segunda leitura da câmera, que divergiria no dia em que uma delas mudasse.
        let rect = vista_da_cena.map(|v| {
            (
                v.center,
                ph2d_ecs::camera_2d::half_extent(v.height_world, aspecto),
            )
        });
        // ⭐ **A câmera no canvas** (plano GEF 17): as molduras deste quadro, a activa com a MESMA vista.
        let selecao = hero_screen.as_ref().and_then(|h| h.gizmo.selection);
        self.undo.camera_no_canvas.molduras = ph2d_camera_frame::molduras(
            sim.world_mut(),
            vista_da_cena.map(|v| v.center),
            aspecto,
            selecao,
        );
        // As alças com o relógio PARADO, em qualquer instante (onda 18, medido no app real): a tocar a moldura é o
        // jogo (17b A8); pausado a meio é onde o Auto Key anima — presas ao zero, o arrasto ia para a seleção.
        self.undo.camera_no_canvas.em_edicao = !self.playhead.is_playing();
        // ⭐ A miniatura (A6): o que a câmera seleccionada vê, no canto do canvas, com «Preview» ligado e fora do palco.
        let preview = hero_screen
            .as_ref()
            .is_some_and(|h| h.view.camera_canvas.preview && !h.view.palco_do_jogo);
        let chao = hero_screen
            .as_ref()
            .and_then(|h| h.last_layout.as_ref())
            .map_or(f32::INFINITY, |l| l.bottom_hud.y);
        let pedido = preview
            .then(|| ph2d_app_game::miniatura::escolhida(&self.undo.camera_no_canvas.molduras))
            .flatten()
            .and_then(|m| {
                let caixa = ph2d_app_game::miniatura::caixa(visivel, aspecto, chao)?;
                Some((ph2d_app_game::miniatura::vista(sim.world(), m), caixa))
            });
        miniatura.pede(surface.gpu(), vello_pass, pedido);
        // ⚠️ **Fala uma vez por MUDANÇA, nunca por quadro** — a mesma lei da linha do som.
        if self.signal_readers.logging() && camera_report != self.last_camera_report {
            self.last_camera_report = camera_report.clone();
            eprintln!(
                "[camera-2d] {} camera(s) · segue={} · alvo `{}` {} · centro ({:.2}, {:.2}){}",
                camera_report.cameras,
                camera_report.following,
                camera_report.target_name,
                if camera_report.target_found {
                    "ACHADO"
                } else {
                    "POR ACHAR"
                },
                camera_report.center[0],
                camera_report.center[1],
                if camera_report.limited {
                    " · na CERCA"
                } else {
                    ""
                }
            );
        }
        rect
    }
}
