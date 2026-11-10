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

/// O botão ou campo do cartão onde a mão clica; o id é da shell, que conhece o Inspector.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Botao {
    #[default]
    EditShape,
    FitToPicture,
    Largura,
}

/// Um roteiro: a alça, para onde ela vai (m, no referencial do mundo) e o que vem depois.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Plano {
    pub alca: Option<Alca>,
    pub delta: [f32; 2],
    /// Fica de botão apertado: a foto do arrasto a meio.
    pub segura: bool,
    /// Depois de soltar, Ctrl+Z.
    pub desfaz: bool,
    /// O `Esc` a meio do arrasto e, depois de soltar, o segundo `Esc`.
    pub esc: bool,
    /// Depois de soltar, o segundo clique no botão.
    pub desliga: bool,
    pub teclas: crate::Teclas,
    /// O que escrever no campo Width antes de tudo.
    pub largura: Option<&'static str>,
    /// O botão do início.
    pub botao: Botao,
    /// Sem arrasto: só o botão e os relatos.
    pub sem_arrasto: bool,
    /// Clique parado dentro da forma, junto da alça do lado direito, sem a agarrar.
    pub na_ponta: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PassoDaMao {
    /// O clique no botão do cartão.
    Clica(Botao),
    /// Teclas no campo focado, e Enter.
    Escreve(&'static str),
    Esc,
    Desce(f32, f32),
    Move(f32, f32, crate::Teclas),
    Sobe,
    Desfaz,
    Relata(String),
}

const INICIO: u32 = 45;
const PASSOS: u32 = 6;
/// Dentro da forma e fora do dedo da alça ([`crate::AGARRE_PX`]).
const PONTA_PX: f32 = 15.0; // LITERAL-PX-OK: a mão do smoke, em ecrã

static QUADRO: AtomicU32 = AtomicU32::new(0);
/// Onde a alça estava no mundo no `Down`, para o caminho da mão não correr atrás da própria alça.
static DE: Mutex<[f32; 2]> = Mutex::new([0.0; 2]);
/// Onde o ponteiro está, em ecrã.
static ULTIMO: Mutex<(f32, f32)> = Mutex::new((0.0, 0.0));

/// A pose e a medida de agora.
fn forma(world: &World, alvo: u64) -> Option<(Pose, Medida)> {
    let e = Entity::from_bits(alvo);
    Some((Pose::de(world, e)?, Medida::da_forma(world, e)?))
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
    if plano.largura.is_some() && quadro == INICIO - 25 {
        return Some(PassoDaMao::Clica(Botao::Largura));
    }
    if quadro == INICIO - 20 {
        return plano.largura.map(PassoDaMao::Escreve);
    }
    if quadro == INICIO - 5 {
        return Some(PassoDaMao::Clica(plano.botao));
    }
    let alca = plano.alca?;
    let (pose, m) = forma(world, alvo)?;
    let relata = |rotulo: String| {
        let s = no_ecra(&pose, &m, alca, cam, win);
        let u = *ULTIMO.lock().ok()?;
        let nome = world.get::<ph2d_ecs::Name>(Entity::from_bits(alvo));
        Some(PassoDaMao::Relata(format!(
            "k={rotulo} {:?} offset={:?} alca_ecra=({:.2},{:.2}) ponteiro=({:.2},{:.2}) erro_px={:.3} alvo={} {}",
            m.shape,
            m.offset,
            s.0,
            s.1,
            u.0,
            u.1,
            (s.0 - u.0).hypot(s.1 - u.1),
            nome.map_or("?", |n| n.0.as_str()),
            format_args!("editando={}", crate::editando(world) == Some(alvo)),
        )))
    };
    if quadro == INICIO - 6 {
        return relata("antes".into());
    }
    let k = quadro.checked_sub(INICIO)?;
    let anda = |p: [f32; 2]| {
        let s = cam.world_to_screen(p, win);
        *ULTIMO.lock().ok()? = s;
        Some(s)
    };
    match k {
        0 if plano.sem_arrasto => None,
        0 if plano.na_ponta => {
            let s = no_ecra(&pose, &m, Alca::Lado(1, 0), cam, win);
            let s = (s.0 - PONTA_PX, s.1);
            *ULTIMO.lock().ok()? = s;
            Some(PassoDaMao::Desce(s.0, s.1))
        }
        0 => {
            let w = pose.para_mundo(ponto_local(&pose, &m, alca));
            *DE.lock().ok()? = w;
            anda(w).map(|s| PassoDaMao::Desce(s.0, s.1))
        }
        k if k <= PASSOS && (plano.sem_arrasto || plano.na_ponta) => None,
        k if k <= PASSOS => {
            let w = *DE.lock().ok()?;
            #[allow(clippy::cast_precision_loss)]
            let t = k as f32 / PASSOS as f32;
            let p = [w[0] + plano.delta[0] * t, w[1] + plano.delta[1] * t];
            anda(p).map(|s| PassoDaMao::Move(s.0, s.1, plano.teclas))
        }
        k if k == PASSOS + 2 || k == PASSOS + 14 || k == PASSOS + 30 => relata(k.to_string()),
        k if k == PASSOS + 3 && plano.esc => Some(PassoDaMao::Esc),
        k if k == PASSOS + 4 && plano.esc => Some(PassoDaMao::Sobe),
        k if k == PASSOS + 3 && !plano.segura && !plano.sem_arrasto => Some(PassoDaMao::Sobe),
        k if k == PASSOS + 16 && plano.esc => Some(PassoDaMao::Esc),
        k if k == PASSOS + 16 && plano.desliga => Some(PassoDaMao::Clica(plano.botao)),
        k if k == PASSOS + 20 && plano.desfaz => Some(PassoDaMao::Desfaz),
        _ => None,
    }
}
