//! **A cena do smoke «o texto na dope-sheet e o olho»** (F27, fecho da B2 e B3) —
//! `PH2D_AUTOKEY_SMOKE=30`, irmã da `=29` (`autokey_smoke_alvo`, de quem herda a montagem). A câmera
//! segue a Box A do quadro 0 e a Box C do 12 (gravado pela cena com o Auto Key); a cena pára no 6
//! com a câmera escolhida: a linha «Camera · Target» da dope-sheet escreve «Box A», não um índice.
//! O Auto Key fica ligado para o dono clicar no OLHO da Box B na Hierarquia (o canal do «Visible»).

use ph2d_core::Playhead;
use ph2d_ecs::{CameraFollow, Entity, SimWorld};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_timeline::TimelineState;
use std::sync::Mutex;

use crate::autokey_smoke::avanca as go;
use crate::autokey_smoke_alvo::{montar, seguir};
use crate::autokey_smoke_tudo::trilhas;

const QUADRO_0: f64 = 0.0;
const QUADRO_6: f64 = 0.25;
const QUADRO_12: f64 = 0.5;

/// A câmera.
static CAMERA: Mutex<Option<u64>> = Mutex::new(None);

fn camera() -> Option<u64> {
    CAMERA.lock().ok().and_then(|c| *c)
}

pub(crate) fn step(
    nivel: u8,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    tl: &mut TimelineState,
    ph: &mut Playhead,
) -> Option<String> {
    match nivel {
        1 => {
            let (c, _) = montar(sim);
            if let Ok(mut a) = CAMERA.lock() {
                *a = Some(c);
            }
            for p in ["timeline", "inspector", "hierarchy"] {
                hero.set_panel_visible(p, true);
            }
            go(2);
            Some("[autokey-smoke] =30 cena montada: Box A, Box B, Box C e a Camera".to_owned())
        }
        s @ 2..=60 => {
            go(s + 1);
            let c = camera();
            match s {
                8 => {
                    tl.flags.auto_key = true;
                    ph.pause();
                    ph.seek(QUADRO_0);
                    hero.gizmo.replace_selection(c);
                    None
                }
                12 => {
                    seguir(sim, c, "Box A");
                    None
                }
                16 => {
                    ph.seek(QUADRO_12);
                    None
                }
                20 => {
                    seguir(sim, c, "Box C");
                    None
                }
                24 => {
                    ph.seek(QUADRO_6);
                    None
                }
                // Uma medida segue-se a uma ida no tempo: o app só redesenha quando algo muda.
                30 => {
                    let alvo = c
                        .and_then(|b| sim.world().get::<CameraFollow>(Entity::from_bits(b)))
                        .map(|f| f.target.clone())
                        .unwrap_or_default();
                    let ok = alvo == "Box A";
                    Some(format!(
                        "[autokey-smoke] q6: a câmera segue «{alvo}» — tem de ser «Box A»: {} · \
                         trilhas da câmera {} · o Auto Key fica LIGADO: escolha a Box B na \
                         Hierarquia, vá ao quadro 12 e clique no olho dela",
                        if ok { "OK" } else { "PARADA" },
                        c.map_or_else(String::new, |e| trilhas(tl, e)),
                    ))
                }
                _ => None,
            }
        }
        _ => None,
    }
}
