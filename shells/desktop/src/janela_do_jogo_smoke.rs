//! As fotos da câmera do smoke do jogo (`camera_chave`, `camera_desfaz`, onda 18): nomes da cena e
//! linhas de consola para quem corre o smoke.

/// As chaves que a timeline guarda para a «Camera» da cena: `propriedade=n` por ligação (a foto `camera_chave`).
pub(crate) fn chaves_da_camera(
    sim: &mut ph2d_ecs::SimWorld,
    tl: &ph2d_timeline::TimelineState,
) -> String {
    let Some(cam) = ph2d_app_game::smoke::por_nome(sim, "Camera") else {
        return "sem câmera".to_owned();
    };
    let clip = tl.doc.active_clip();
    let v: Vec<String> = tl
        .doc
        .bindings()
        .iter()
        .filter(|b| b.entity == cam.to_bits())
        .map(|b| {
            let n = clip.track(b.target).map_or(0, |t| t.len());
            format!("{:?}={n}", b.prop)
        })
        .collect();
    if v.is_empty() {
        "nenhuma ligação".to_owned()
    } else {
        v.join(" · ")
    }
}

impl crate::App {
    /// A foto `camera_desfaz` (onda 18): no quadro 40 o formato vai a Game Boy pela porta do Inspector (com a
    /// entrada que um clique dá), e depois Ctrl+Z e Ctrl+Y — `[camera-desfaz]` diz o formato a cada passo.
    pub(super) fn formato_desfaz_do_smoke(&mut self, quadro: u32) {
        use ph2d_editor_core::{CameraFieldEdit, GamePreset};
        let relata = |app: &Self, quando: &str| {
            if let Some(h) = app.gfx.as_ref().and_then(|g| g.hero_screen.as_ref()) {
                eprintln!(
                    "[camera-desfaz] {quando}: {:?}",
                    h.project.game_format.preset
                );
            }
        };
        match quadro {
            40 => {
                relata(self, "antes");
                self.any_input_this_frame = true;
                let cam = self
                    .gfx
                    .as_mut()
                    .and_then(|g| ph2d_app_game::smoke::por_nome(&mut g.sim, "Camera"));
                let i = GamePreset::ALL
                    .iter()
                    .position(|p| *p == GamePreset::GameBoy)
                    .unwrap_or(0);
                if let (Some(cam), Some(h)) =
                    (cam, self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()))
                {
                    h.bus.push(
                        ph2d_editor_core::action_bus::EditorAction::InspectorCameraEdit {
                            entity_bits: cam.to_bits(),
                            edit: CameraFieldEdit::Format(i as u8),
                        },
                    );
                }
            }
            // O clique do artista é entrada nesses quadros (a fila só fecha passos com entrada).
            41..=43 => self.any_input_this_frame = true,
            50 => {
                relata(self, "trocado");
                self.smoke_undo(false);
            }
            60 => {
                relata(self, "depois do Ctrl+Z");
                self.smoke_undo(true);
            }
            70 => relata(self, "depois do Ctrl+Y"),
            _ => {}
        }
    }
}
