//! ⭐ **A câmera no canvas** ([plano GEF 17](../../../docs/GameEngineFuncional/17_camera_no_canvas_plano.md) A1–A4):
//! o ícone, a moldura EXACTA do que o jogo mostra, o canto que dá zoom, o miolo que move e as alças dos limites.
//!
//! A moldura da câmera activa é a vista que o passe da câmera calculou neste quadro (`camera_2d::update`, com o
//! seguimento, os limites e o abanão); as outras são a pose + `offset`. A proporção é a do formato do jogo
//! ([`ph2d_editor_core::GameFormat::aspect`]) — a mesma conta da janela do jogo e do «Look through».
//!
//! ⚠️ **O miolo só agarra o que nada mais agarraria** (a lição da Godot, #104190): com outro objecto sob o
//! ponteiro o clique segue para a seleção. A câmera não selecionada não agarra nada — pega-se pelo ícone.

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_ecs::{CameraLimits, GameCamera};

/// O que o editor guarda entre quadros: as molduras deste quadro e o arrasto em curso.
#[derive(Clone, Debug, Default)]
pub struct CameraNoCanvas {
    pub molduras: Vec<Moldura>,
    pub arrasto: Option<Arrasto>,
}

/// Uma câmera da cena, como o canvas a desenha.
#[derive(Clone, Debug, PartialEq)]
pub struct Moldura {
    pub camera: u64,
    pub nome: String,
    /// Onde o objecto câmera está (o ícone).
    pub pose: [f32; 2],
    /// O centro e a meia-extensão do que ela filma.
    pub centro: [f32; 2],
    pub meia: [f32; 2],
    pub ativa: bool,
    pub selecionada: bool,
    /// `(min, max)` dos limites, se os tiver.
    pub limites: Option<([f32; 2], [f32; 2])>,
}

/// Toda câmera da cena, por identidade. `vista_ativa` = o centro que o passe da câmera deu à activa.
pub fn molduras(
    world: &mut World,
    vista_ativa: Option<[f32; 2]>,
    aspecto: f32,
    selecao: Option<u64>,
) -> Vec<Moldura> {
    let ativa = ph2d_ecs::active_camera_of(world);
    let mut q = world.query::<(Entity, &GameCamera)>();
    let mut cams: Vec<(u64, Entity, GameCamera)> = q
        .iter(world)
        .map(|(e, c)| {
            let id = world.get::<ph2d_ecs::StableId>(e).map_or(0, |s| s.0);
            (id, e, c.clone())
        })
        .collect();
    cams.sort_unstable_by_key(|c| c.0);
    cams.into_iter()
        .map(|(_, e, c)| {
            let pose = ph2d_ecs::world_transform(world, e)
                .map_or([0.0, 0.0], |t| [t.translation.x, t.translation.y]);
            let e_ativa = ativa == Some(e);
            let centro = match vista_ativa {
                Some(v) if e_ativa => v,
                _ => [pose[0] + c.offset[0], pose[1] + c.offset[1]],
            };
            Moldura {
                camera: e.to_bits(),
                nome: world
                    .get::<ph2d_ecs::Name>(e)
                    .map(|n| n.0.clone())
                    .unwrap_or_default(),
                pose,
                centro,
                meia: ph2d_ecs::half_extent(c.height_world, aspecto),
                ativa: e_ativa,
                selecionada: selecao == Some(e.to_bits()),
                limites: world.get::<CameraLimits>(e).map(|l| (l.min, l.max)),
            }
        })
        .collect()
}

/// O que se agarra numa moldura.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alca {
    /// Um canto da moldura (`0` = baixo-esquerda, sentido anti-horário): o zoom.
    Canto(u8),
    /// O miolo da moldura: mover a câmera.
    Dentro,
    /// Um canto dos limites (a mesma ordem).
    LimiteCanto(u8),
    /// Uma borda dos limites: `0` esquerda · `1` direita · `2` baixo · `3` cima.
    LimiteBorda(u8),
    /// A alça do centro dos limites: mover a cerca inteira.
    LimiteCentro,
}

/// Os quatro cantos de uma caixa `(min, max)`, na ordem de [`Alca::Canto`].
#[must_use]
pub fn cantos(min: [f32; 2], max: [f32; 2]) -> [[f32; 2]; 4] {
    [min, [max[0], min[1]], max, [min[0], max[1]]]
}

fn perto(a: [f32; 2], b: [f32; 2], tol: f32) -> bool {
    (a[0] - b[0]).abs() <= tol && (a[1] - b[1]).abs() <= tol
}

/// A alça de `m` sob o ponto `p` (mundo), com a tolerância `tol` (mundo). Só a câmera selecionada tem alças.
/// `outro_por_baixo` = outro objecto responde ao clique ali: então o miolo NÃO agarra.
#[must_use]
pub fn alca_em(m: &Moldura, p: [f32; 2], tol: f32, outro_por_baixo: bool) -> Option<Alca> {
    if !m.selecionada {
        return None;
    }
    let (lo, hi) = caixa(m);
    if let Some(i) = cantos(lo, hi).iter().position(|c| perto(*c, p, tol)) {
        return Some(Alca::Canto(i as u8));
    }
    if let Some((min, max)) = m.limites {
        if let Some(i) = cantos(min, max).iter().position(|c| perto(*c, p, tol)) {
            return Some(Alca::LimiteCanto(i as u8));
        }
        let meio = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
        if perto(meio, p, tol) {
            return Some(Alca::LimiteCentro);
        }
        let dentro_y = p[1] >= min[1] && p[1] <= max[1];
        let dentro_x = p[0] >= min[0] && p[0] <= max[0];
        for (i, (eixo, v, dentro)) in [
            (0, min[0], dentro_y),
            (0, max[0], dentro_y),
            (1, min[1], dentro_x),
            (1, max[1], dentro_x),
        ]
        .into_iter()
        .enumerate()
        {
            if dentro && (p[eixo] - v).abs() <= tol {
                return Some(Alca::LimiteBorda(i as u8));
            }
        }
    }
    let dentro = (0..2).all(|i| p[i] >= lo[i] && p[i] <= hi[i]);
    (dentro && !outro_por_baixo).then_some(Alca::Dentro)
}

/// `(min, max)` do que a câmera filma.
#[must_use]
pub fn caixa(m: &Moldura) -> ([f32; 2], [f32; 2]) {
    (
        [m.centro[0] - m.meia[0], m.centro[1] - m.meia[1]],
        [m.centro[0] + m.meia[0], m.centro[1] + m.meia[1]],
    )
}

/// Um arrasto em curso, congelado no `Down`.
#[derive(Clone, Debug, PartialEq)]
pub struct Arrasto {
    pub camera: u64,
    pub alca: Alca,
    p0: [f32; 2],
    centro0: [f32; 2],
    aspecto: f32,
    /// A translação LOCAL da câmera no `Down`.
    pose0: [f32; 2],
    limites0: ([f32; 2], [f32; 2]),
}

impl Arrasto {
    #[must_use]
    pub fn novo(m: &Moldura, alca: Alca, p0: [f32; 2], aspecto: f32, pose_local: [f32; 2]) -> Self {
        Self {
            camera: m.camera,
            alca,
            p0,
            centro0: m.centro,
            aspecto,
            pose0: pose_local,
            limites0: m.limites.unwrap_or_default(),
        }
    }
}

/// O que um arrasto escreve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Escrita {
    /// `GameCamera::height_world` — o centro fica.
    Altura(f32),
    /// A translação LOCAL da câmera.
    Pose([f32; 2]),
    /// `CameraLimits { min, max }` — nunca invertidos.
    Limites([f32; 2], [f32; 2]),
}

/// O raio de agarre das alças, em píxeis de tela — o mesmo dos gizmos do Motion (`warp_gizmo::GRAB_PX`).
pub const AGARRE_PX: f32 = 11.0; // LITERAL-PX-OK: raio de agarre

/// O menor vão entre `min` e `max` dos limites que um arrasto deixa (metros): um centímetro.
pub const LIMITE_VAO_MIN: f32 = 0.01;

/// O que o arrasto `a` escreve com o ponteiro em `p` (mundo).
#[must_use]
pub fn arrastar(a: &Arrasto, p: [f32; 2]) -> Escrita {
    let d = [p[0] - a.p0[0], p[1] - a.p0[1]];
    let (mut min, mut max) = a.limites0;
    match a.alca {
        Alca::Canto(_) => {
            let dy = (p[1] - a.centro0[1]).abs() * 2.0;
            let dx = (p[0] - a.centro0[0]).abs() * 2.0 / a.aspecto.max(f32::MIN_POSITIVE);
            Escrita::Altura(dx.max(dy).clamp(
                ph2d_ecs::camera_2d::CAMERA_MIN_HEIGHT_WORLD,
                ph2d_ecs::camera_2d::CAMERA_MAX_HEIGHT_WORLD, // CLAMP-OK: o tecto é o do motor
            ))
        }
        Alca::Dentro => Escrita::Pose([a.pose0[0] + d[0], a.pose0[1] + d[1]]),
        Alca::LimiteCentro => Escrita::Limites(
            [min[0] + d[0], min[1] + d[1]],
            [max[0] + d[0], max[1] + d[1]],
        ),
        Alca::LimiteCanto(i) => {
            let c = cantos(min, max)[usize::from(i)];
            let n = [c[0] + d[0], c[1] + d[1]];
            if c[0] == min[0] {
                min[0] = n[0].min(max[0] - LIMITE_VAO_MIN);
            } else {
                max[0] = n[0].max(min[0] + LIMITE_VAO_MIN);
            }
            if c[1] == min[1] {
                min[1] = n[1].min(max[1] - LIMITE_VAO_MIN);
            } else {
                max[1] = n[1].max(min[1] + LIMITE_VAO_MIN);
            }
            Escrita::Limites(min, max)
        }
        Alca::LimiteBorda(i) => {
            match i {
                0 => min[0] = (min[0] + d[0]).min(max[0] - LIMITE_VAO_MIN),
                1 => max[0] = (max[0] + d[0]).max(min[0] + LIMITE_VAO_MIN),
                2 => min[1] = (min[1] + d[1]).min(max[1] - LIMITE_VAO_MIN),
                _ => max[1] = (max[1] + d[1]).max(min[1] + LIMITE_VAO_MIN),
            }
            Escrita::Limites(min, max)
        }
    }
}

#[cfg(test)]
#[path = "moldura_tests.rs"]
mod tests;
