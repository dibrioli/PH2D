//! ⭐ **A câmera no canvas** ([plano GEF 17](../../../docs/GameEngineFuncional/17_camera_no_canvas_plano.md) A1–A4,
//! [cura 17b](../../../docs/GameEngineFuncional/17b_auditoria_camera_e_plano.md)): o ícone, a moldura EXACTA do que
//! o jogo mostra, o canto que dá zoom, o miolo que move e as alças dos limites.
//!
//! A moldura da câmera activa é a vista que o passe da câmera calculou neste quadro (`camera_2d::update`); as
//! outras saem da MESMA lei ([`ph2d_app_components::camera_2d::enquadra`]) sobre a mira parada. A proporção é a do
//! formato do jogo ([`ph2d_editor_core::GameFormat::aspect`]).
//!
//! ⭐ **O miolo é um clique PENDENTE** (17b A1): o `Down` dentro da moldura da câmera selecionada fica seguro;
//! arrastar além de [`DRAG_THRESHOLD_PX`] move a câmera, soltar sem mexer devolve o clique à seleção (o objecto
//! por baixo). Com `follow`, mover é editar o offset do `follow` (17b A2): a câmera renasce no alvo a cada quadro
//! do editor parado, e a pose não a moveria.

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_app_components::camera_2d::{alvo_do_follow, enquadra, mira_parada};
use ph2d_ecs::{CameraFollow, CameraLimits, GameCamera};
use ph2d_editor_core::CameraFieldEdit;
use ph2d_editor_core::interaction::drag::DRAG_THRESHOLD_PX;

/// O que o editor guarda entre quadros: as molduras deste quadro e o arrasto em curso.
#[derive(Clone, Debug, Default)]
pub struct CameraNoCanvas {
    pub molduras: Vec<Moldura>,
    pub arrasto: Option<Arrasto>,
    /// As alças só vivem com o editor parado no início (17b A8): a correr — ou pausado a meio — a moldura é o
    /// jogo, e um arrasto escreveria no documento a meio da corrida.
    pub em_edicao: bool,
    /// A vista do editor no `Down` (17b A5): com o «Look through» a vista anda com a câmera, e converter o
    /// ponteiro por ela realimentaria o arrasto.
    pub vista_do_down: Option<ph2d_render::Camera2d>,
}

/// O alvo de uma câmera que segue.
#[derive(Clone, Debug, PartialEq)]
pub struct Segue {
    pub nome: String,
    /// Onde o alvo está — `None` = o nome não casa com ninguém.
    pub alvo: Option<[f32; 2]>,
    /// O offset do `follow` (entra na mira, antes da cerca).
    pub offset: [f32; 2],
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
    /// A mira parada (antes da cerca) e o offset da VISTA (soma depois dela).
    pub mira: [f32; 2],
    pub offset: [f32; 2],
    pub segue: Option<Segue>,
}

/// Toda câmera visível da cena, por identidade. `vista_ativa` = o centro que o passe da câmera deu à activa.
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
        .filter(|(e, _)| {
            !world
                .get::<ph2d_ecs::Visibility>(*e)
                .is_some_and(|v| v.hidden)
        })
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
            let meia = ph2d_ecs::half_extent(c.height_world, aspecto);
            let limites = world.get::<CameraLimits>(e).map(|l| (l.min, l.max));
            let mira = mira_parada(world, e);
            let e_ativa = ativa == Some(e);
            let centro = match vista_ativa {
                Some(v) if e_ativa => v,
                _ => centro_da_lei(mira, meia, limites, c.offset),
            };
            let segue = world.get::<CameraFollow>(e).cloned().map(|f| Segue {
                alvo: alvo_do_follow(world, e).map(|a| a.1),
                nome: f.target.trim().to_string(),
                offset: f.offset,
            });
            Moldura {
                camera: e.to_bits(),
                nome: world
                    .get::<ph2d_ecs::Name>(e)
                    .map(|n| n.0.clone())
                    .unwrap_or_default(),
                pose,
                centro,
                meia,
                ativa: e_ativa,
                selecionada: selecao == Some(e.to_bits()),
                limites,
                mira,
                offset: c.offset,
                segue,
            }
        })
        .collect()
}

/// O centro do que a câmera filma: a mira presa à cerca, mais o offset da vista.
fn centro_da_lei(
    mira: [f32; 2],
    meia: [f32; 2],
    limites: Option<([f32; 2], [f32; 2])>,
    offset: [f32; 2],
) -> [f32; 2] {
    let l = limites.map(|(min, max)| CameraLimits { min, max });
    let c = enquadra(mira, meia, l.as_ref());
    [c[0] + offset[0], c[1] + offset[1]]
}

/// O que se agarra numa moldura.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alca {
    /// Um canto da moldura (`0` = baixo-esquerda, sentido anti-horário): o zoom.
    Canto(u8),
    /// O miolo da moldura: mover a câmera (ou o offset do `follow`).
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
#[must_use]
pub fn alca_em(m: &Moldura, p: [f32; 2], tol: f32) -> Option<Alca> {
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
    (0..2)
        .all(|i| p[i] >= lo[i] && p[i] <= hi[i])
        .then_some(Alca::Dentro)
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
    aspecto: f32,
    /// A translação LOCAL da câmera no `Down`.
    pose0: [f32; 2],
    limites0: ([f32; 2], [f32; 2]),
    limites: Option<([f32; 2], [f32; 2])>,
    mira0: [f32; 2],
    offset: [f32; 2],
    /// O offset do `follow` no `Down` — com ele o miolo move o offset e não a pose.
    follow0: Option<[f32; 2]>,
    /// Onde o `Down` caiu na tela, e se o arrasto já andou o limiar (o miolo nasce por armar).
    ecra0: (f32, f32),
    armado: bool,
}

impl Arrasto {
    #[must_use]
    pub fn novo(
        m: &Moldura,
        alca: Alca,
        p0: [f32; 2],
        aspecto: f32,
        pose_local: [f32; 2],
        ecra0: (f32, f32),
    ) -> Self {
        Self {
            camera: m.camera,
            alca,
            p0,
            aspecto,
            pose0: pose_local,
            limites0: m.limites.unwrap_or_default(),
            limites: m.limites,
            mira0: m.mira,
            offset: m.offset,
            follow0: m.segue.as_ref().map(|s| s.offset),
            ecra0,
            armado: alca != Alca::Dentro,
        }
    }

    /// O ponteiro está em `ecra`: o miolo arma-se depois de andar o limiar. `true` = o arrasto escreve.
    pub fn arma(&mut self, ecra: (f32, f32)) -> bool {
        let d = (ecra.0 - self.ecra0.0).hypot(ecra.1 - self.ecra0.1);
        self.armado |= d > DRAG_THRESHOLD_PX;
        self.armado
    }

    /// Soltou sem nunca armar: era um CLIQUE, e ele é da seleção.
    #[must_use]
    pub fn era_um_clique(&self) -> bool {
        !self.armado
    }

    /// Onde o `Down` caiu na tela.
    #[must_use]
    pub fn ecra0(&self) -> (f32, f32) {
        self.ecra0
    }
}

/// O que um arrasto escreve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Escrita {
    /// `GameCamera::height_world`.
    Altura(f32),
    /// A translação LOCAL da câmera.
    Pose([f32; 2]),
    /// `CameraFollow::offset`.
    FollowOffset([f32; 2]),
    /// `CameraLimits { min, max }` — nunca invertidos.
    Limites([f32; 2], [f32; 2]),
}

/// A edição que uma escrita manda pela porta do Inspector (a pose escreve-se directa no `Transform`). ⚠️ Uma
/// só: dois lados da cerca em duas edições no mesmo quadro apagam-se (17b A3).
#[must_use]
pub fn edicoes(e: Escrita) -> Option<CameraFieldEdit> {
    match e {
        Escrita::Altura(h) => Some(CameraFieldEdit::Height(h)),
        Escrita::FollowOffset(o) => Some(CameraFieldEdit::FollowOffset(o)),
        Escrita::Limites(min, max) => Some(CameraFieldEdit::Limits(min, max)),
        Escrita::Pose(_) => None,
    }
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
        Alca::Canto(i) => Escrita::Altura(altura_do_canto(a, i, p)),
        Alca::Dentro => match a.follow0 {
            Some(o) => Escrita::FollowOffset([o[0] + d[0], o[1] + d[1]]),
            None => Escrita::Pose([a.pose0[0] + d[0], a.pose0[1] + d[1]]),
        },
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

/// ⭐ **A altura que põe o canto `i` sob o ponteiro** (17b A7), com o centro que a CERCA dá a cada altura — e não
/// o do `Down`: crescer encostado a um limite empurra o centro para dentro. Por eixo, a menor altura cujo canto
/// chega ao ponteiro (a posição do canto não decresce com a altura: livre, preso do lado do canto, preso do outro
/// lado, ou no meio da caixa — quatro rectas não-decrescentes); a altura é a do eixo que manda.
fn altura_do_canto(a: &Arrasto, i: u8, p: [f32; 2]) -> f32 {
    use ph2d_ecs::camera_2d::{CAMERA_MAX_HEIGHT_WORLD as MAX, CAMERA_MIN_HEIGHT_WORLD as MIN};
    let sinal = [
        if i == 1 || i == 2 { 1.0 } else { -1.0 },
        if i >= 2 { 1.0 } else { -1.0 },
    ];
    let canto = |h: f32, eixo: usize| {
        let meia = ph2d_ecs::half_extent(h, a.aspecto.max(f32::MIN_POSITIVE));
        let c = centro_da_lei(a.mira0, meia, a.limites, a.offset);
        sinal[eixo] * c[eixo] + meia[eixo]
    };
    let por_eixo = |eixo: usize| {
        let alvo = sinal[eixo] * p[eixo];
        if canto(MIN, eixo) >= alvo {
            return MIN;
        }
        if canto(MAX, eixo) < alvo {
            return MAX;
        }
        let (mut lo, mut hi) = (MIN, MAX);
        // 40 metades de 99,5 m: abaixo do ulp de um `f32` nessa escala.
        for _ in 0..40 {
            let m = 0.5 * (lo + hi);
            if canto(m, eixo) >= alvo {
                hi = m;
            } else {
                lo = m;
            }
        }
        hi
    };
    por_eixo(0).max(por_eixo(1))
}

#[cfg(test)]
#[path = "moldura_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "moldura_ponteiro_tests.rs"]
mod ponteiro_tests;
