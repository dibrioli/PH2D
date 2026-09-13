//! **Fase do quadro: A CÂMERA DE JOGO** (TOP-20 #7) — o herói da cena de smoke da câmera anda nos mesmos
//! tiques que ela, a câmera segue o mundo DESTE quadro, a vista só é tomada com a pré-visualização ligada,
//! e o relatório fala uma vez por mudança (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ **Depois dos relógios e ANTES do extract**, e as duas metades são load-bearing (ver o corpo). Os
//! dois parâmetros são `Copy` e o quadro continua a lê-los mais abaixo.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_game_camera(
        &mut self,
        player_input: ph2d_physics_ecs::PlayerInput,
        report: ph2d_core::FixedStepReport,
    ) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            surface,
            sim,
            camera,
            ..
        } = FrameGfx::of(gfx);

        // ⭐ **O herói da cena de smoke da câmera anda AQUI**, imediatamente antes do passe dela —
        // ver o doc do [`ph2d_app_components::camera_2d_smoke`] sobre porque ele é movido pelo TECLADO e não pelo
        // rato (um arrasto ancorado na vista realimenta uma câmera que segue). No-op sem a cena.
        if self.game_camera_smoke_done {
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

        // ⭐⭐⭐ **A CÂMERA DE JOGO** (TOP-20 #7) — a cena passa a poder mandar no enquadramento.
        //
        // ⚠️ **Aqui, depois dos relógios e ANTES do extract**, e as duas metades são load-bearing:
        // depois, porque a câmera segue o mundo **deste** quadro (um passe antes dos tiques
        // enquadraria o quadro anterior, e num alvo rápido isso lê-se como *«a câmera atrasa»*);
        // antes, porque o extract é quem lê a `camera` para desenhar.
        //
        // ⚠️ **Ele não escreve componente registado nenhum** — o `CameraRuntime` não é gravável —,
        // logo não passa pelo `preview_drive`.
        let (vista_da_cena, camera_report) = camera_2d::update(
            sim,
            camera_2d::aspect_of(surface.size()),
            report.ticks,
            self.fixed_step.fixed_dt(),
        );
        // ⭐⭐ **A vista só é TOMADA com a pré-visualização ligada.** Sem isto, toda cena que tenha
        // uma câmera roubaria o pan e o zoom do artista no primeiro quadro — e a `GameCamera` é um
        // componente que se anexa pela paleta, então isso aconteceria por acidente.
        if self.game_camera_preview
            && let Some(v) = vista_da_cena
        {
            camera.center = v.center;
            camera.height_world = v.height_world;
            camera.cull_mask = v.cull_mask;
        }
        // ⚠️ **Fala uma vez por MUDANÇA, nunca por quadro** — a mesma lei da linha do som.
        if self.signal_log_reader.is_some() && camera_report != self.last_camera_report {
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
    }
}
