//! **A mão das fotos e das provas no app real** (`PH2D_GAME_SMOKE=4`): liga pela porta do botão, agarra uma alça
//! e arrasta em ecrã pelas portas do ponteiro. O shell só executa cada passo.

use crate::{Alca, Medida, Pose, no_ecra, ponto_local};
use bevy_ecs::world::World;
use ph2d_ecs::Entity;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::Collider;
use ph2d_render::Camera2d;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

/// Um roteiro: a alça, para onde ela vai (m, no referencial do mundo) e o que vem depois.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plano {
    pub alca: Option<Alca>,
    pub delta: [f32; 2],
    /// Fica de botão apertado: a foto do arrasto a meio.
    pub segura: bool,
    /// Depois de soltar, Ctrl+Z.
    pub desfaz: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PassoDaMao {
    Liga,
    Desce(f32, f32),
    Move(f32, f32),
    Sobe,
    Desfaz,
    Relata(String),
}

const INICIO: u32 = 45;
const PASSOS: u32 = 6;

static QUADRO: AtomicU32 = AtomicU32::new(0);
/// Onde a alça estava no mundo no `Down`, para o caminho da mão não correr atrás da própria alça.
static DE: Mutex<[f32; 2]> = Mutex::new([0.0; 2]);
/// Onde o ponteiro está, em ecrã.
static ULTIMO: Mutex<(f32, f32)> = Mutex::new((0.0, 0.0));

/// A pose e a medida de agora.
fn forma(world: &World, alvo: u64) -> Option<(Pose, Medida)> {
    let e = Entity::from_bits(alvo);
    let col = world.get::<Collider>(e)?;
    Some((Pose::de(world, e)?, Medida::de(col)))
}

/// O passo deste quadro (um por chamada, um contador próprio).
#[must_use]
pub fn passo_da_mao(
    plano: Plano,
    world: &World,
    alvo: u64,
    cam: &Camera2d,
    win: WindowSize,
) -> Option<PassoDaMao> {
    let quadro = QUADRO.fetch_add(1, Ordering::Relaxed);
    if quadro == INICIO - 5 {
        return Some(PassoDaMao::Liga);
    }
    let k = quadro.checked_sub(INICIO)?;
    let alca = plano.alca?;
    let (pose, m) = forma(world, alvo)?;
    let mut anda = |p: [f32; 2]| {
        let s = cam.world_to_screen(p, win);
        *ULTIMO.lock().ok()? = s;
        Some(s)
    };
    match k {
        0 => {
            let w = pose.para_mundo(ponto_local(&pose, &m, alca));
            *DE.lock().ok()? = w;
            anda(w).map(|s| PassoDaMao::Desce(s.0, s.1))
        }
        k if k <= PASSOS => {
            let w = *DE.lock().ok()?;
            #[allow(clippy::cast_precision_loss)]
            let t = k as f32 / PASSOS as f32;
            let p = [w[0] + plano.delta[0] * t, w[1] + plano.delta[1] * t];
            anda(p).map(|s| PassoDaMao::Move(s.0, s.1))
        }
        k if k == PASSOS + 2 || k == PASSOS + 14 || k == PASSOS + 30 => {
            let s = no_ecra(&pose, &m, alca, cam, win);
            let u = *ULTIMO.lock().ok()?;
            Some(PassoDaMao::Relata(format!(
                "k={k} {:?} offset={:?} alca_ecra=({:.2},{:.2}) ponteiro=({:.2},{:.2}) erro_px={:.3} {}",
                m.shape,
                m.offset,
                s.0,
                s.1,
                u.0,
                u.1,
                (s.0 - u.0).hypot(s.1 - u.1),
                motor_de_prova(world, alvo),
            )))
        }
        k if k == PASSOS + 3 && !plano.segura => Some(PassoDaMao::Sobe),
        k if k == PASSOS + 20 && plano.desfaz => Some(PassoDaMao::Desfaz),
        _ => None,
    }
}

fn motor_de_prova(world: &World, alvo: u64) -> String {
    format!("editando={}", crate::editando(world) == Some(alvo))
}
