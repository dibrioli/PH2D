//! ⭐ **A câmera no canvas** ([plano GEF 17](../../../docs/GameEngineFuncional/17_camera_no_canvas_plano.md) A1–A4,
//! [cura 17b](../../../docs/GameEngineFuncional/17b_auditoria_camera_e_plano.md)): o ícone, a moldura EXACTA do que
//! o jogo mostra, o canto que dá zoom, o miolo que move e as alças dos limites.
//!
//! A moldura da câmera activa é a vista que o passe da câmera calculou neste quadro (`camera_2d::update`); as
//! outras saem da MESMA lei ([`ph2d_ecs::camera_2d::enquadra`]) sobre a mira parada. A proporção é a do
//! formato do jogo ([`ph2d_editor_core::GameFormat::aspect`]).
//!
//! ⭐ **O miolo é um clique PENDENTE** (17b A1): o `Down` dentro da moldura da câmera selecionada fica seguro;
//! arrastar além de [`DRAG_THRESHOLD_PX`] move a câmera, soltar sem mexer devolve o clique à seleção (o objecto
//! por baixo). Com `follow`, mover é editar o offset do `follow` (17b A2): a câmera renasce no alvo a cada quadro
//! do editor parado, e a pose não a moveria.

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_ecs::{CameraFollow, CameraLimits, GameCamera};
use ph2d_ecs::{alvo_do_follow, enquadra, mira_parada};
use ph2d_editor_core::CameraFieldEdit;
use ph2d_editor_core::interaction::drag::DRAG_THRESHOLD_PX;

pub mod tinta;

/// O que o editor guarda entre quadros: as molduras deste quadro e o arrasto em curso.
#[derive(Clone, Debug, Default)]
pub struct CameraNoCanvas {
    pub molduras: Vec<Moldura>,
    pub arrasto: Option<Arrasto>,
    /// As alças só vivem com o relógio parado (17b A8, onda 18): a tocar a moldura é o jogo; pausado em
    /// qualquer instante é autoria (com o Auto Key, a chave cai ali).
    pub em_edicao: bool,
    /// A vista do editor no `Down` (17b A5): com o «Look through» a vista anda com a câmera, e converter o
    /// ponteiro por ela realimentaria o arrasto.
    pub vista_do_down: Option<ph2d_render::Camera2d>,
    /// O par do duplo-clique numa alça (GEF 20 C4).
    pub ultimo: UltimoPress,
}

/// O alvo de uma câmera que segue.
#[derive(Clone, Debug, PartialEq)]
pub struct Segue {
    pub nome: String,
    /// Onde o alvo está — `None` = o nome não casa com ninguém.
    pub alvo: Option<[f32; 2]>,
    /// O offset do `follow` (entra na mira, antes da cerca).
    pub offset: [f32; 2],
    /// A zona morta, em fracção da meia-janela por eixo (`0..=1`).
    pub zona: [f32; 2],
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
                zona: f.dead_zone,
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
    /// Uma borda da zona morta (onda 18, a mesma ordem de [`Alca::LimiteBorda`]): o eixo dela cresce ou encolhe,
    /// simétrico.
    ZonaMorta(u8),
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
    if let Some(i) = pegas_da_zona(m, tol)
        .iter()
        .position(|c| c.is_some_and(|c| perto(c, p, tol)))
    {
        return Some(Alca::ZonaMorta(i as u8));
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

/// O centro da zona morta: o MEIO da moldura, sem o offset da vista (dono 02/11, plano GEF 20 C3 — o idioma
/// Godot/Unity: a caixa é da tela da câmera e o alvo anda dentro dela).
#[must_use]
pub fn centro_da_zona(m: &Moldura) -> [f32; 2] {
    [m.centro[0] - m.offset[0], m.centro[1] - m.offset[1]]
}

/// `(min, max)` da zona morta de quem segue.
#[must_use]
pub fn caixa_da_zona(m: &Moldura) -> Option<([f32; 2], [f32; 2])> {
    let s = m.segue.as_ref()?;
    let c = centro_da_zona(m);
    let h = [
        m.meia[0] * s.zona[0].clamp(0.0, 1.0), // CLAMP-OK: a fracção da lei (`dead_zone_goal`)
        m.meia[1] * s.zona[1].clamp(0.0, 1.0), // CLAMP-OK: a fracção da lei
    ];
    Some(([c[0] - h[0], c[1] - h[1]], [c[0] + h[0], c[1] + h[1]]))
}

/// As pegas da zona morta nos meios das bordas (esquerda · direita · baixo · cima). ⭐ Uma zona estreita (ou zero)
/// abre as pegas a [`ZONA_PEGA_MIN`] agarres do centro: debaixo do ícone nenhuma se agarraria.
#[must_use]
pub fn pegas_da_zona(m: &Moldura, tol: f32) -> [Option<[f32; 2]>; 4] {
    let Some((lo, hi)) = caixa_da_zona(m) else {
        return [None; 4];
    };
    let c = centro_da_zona(m);
    let d = tol * ZONA_PEGA_MIN;
    let x = [(c[0] - lo[0]).max(d), (hi[0] - c[0]).max(d)];
    let y = [(c[1] - lo[1]).max(d), (hi[1] - c[1]).max(d)];
    [
        Some([c[0] - x[0], c[1]]),
        Some([c[0] + x[1], c[1]]),
        Some([c[0], c[1] - y[0]]),
        Some([c[0], c[1] + y[1]]),
    ]
}

/// A que distância do centro (em agarres) ficam as pegas de uma zona morta estreita: fora do ícone de fábrica
/// (44 px = 4 agarres de lado).
pub const ZONA_PEGA_MIN: f32 = 3.0;

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
    /// Onde a câmera está no mundo no `Down`, e a pose de mundo do PAI (com pai): o miolo leva a câmera a
    /// `pose_mundo0 + delta` no MUNDO e devolve a translação local pela inversa do pai.
    pose_mundo0: [f32; 2],
    pai: Option<ph2d_ecs::Transform>,
    limites0: ([f32; 2], [f32; 2]),
    limites: Option<([f32; 2], [f32; 2])>,
    mira0: [f32; 2],
    offset: [f32; 2],
    /// O offset do `follow` no `Down` — com ele o miolo move o offset e não a pose.
    follow0: Option<[f32; 2]>,
    /// A zona morta e o centro dela no `Down` (só quem segue).
    zona0: [f32; 2],
    centro_zona: [f32; 2],
    meia: [f32; 2],
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
            pose_mundo0: m.pose,
            pai: None,
            limites0: m.limites.unwrap_or_default(),
            limites: m.limites,
            mira0: m.mira,
            offset: m.offset,
            follow0: m.segue.as_ref().map(|s| s.offset),
            zona0: m.segue.as_ref().map_or([0.0; 2], |s| s.zona),
            centro_zona: centro_da_zona(m),
            meia: m.meia,
            ecra0,
            armado: alca != Alca::Dentro,
        }
    }

    /// A pose de mundo do pai da câmera — com ela o miolo segue o ponteiro com o pai rodado ou escalado.
    #[must_use]
    pub fn com_pai(mut self, pai: ph2d_ecs::Transform) -> Self {
        self.pai = Some(pai);
        self
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

    /// ⭐ GEF 20 C4 — o `Esc` a meio do arrasto: a escrita que devolve o que a alça mexe ao valor do `Down`.
    #[must_use]
    pub fn restaura(&self) -> Escrita {
        match self.alca {
            Alca::Canto(_) => Escrita::Altura(self.meia[1] * 2.0),
            Alca::Dentro => self
                .follow0
                .map_or(Escrita::Pose(self.pose0), Escrita::FollowOffset),
            Alca::LimiteCanto(_) | Alca::LimiteBorda(_) | Alca::LimiteCentro => {
                Escrita::Limites(self.limites0.0, self.limites0.1)
            }
            Alca::ZonaMorta(_) => Escrita::ZonaMorta(self.zona0),
        }
    }
}

/// ⭐ GEF 20 C4 — **o duplo-clique numa alça devolve-a à fábrica**: o zoom à altura de omissão, a zona morta a
/// zero, o offset do `follow` a zero, os limites ao `contorno` do nível. `None` = a alça não tem padrão (o miolo
/// sem `follow` move a pose: não há pose de fábrica) ou não há contorno.
#[must_use]
pub fn padrao(m: &Moldura, alca: Alca, contorno: Option<([f32; 2], [f32; 2])>) -> Option<Escrita> {
    match alca {
        Alca::Canto(_) => Some(Escrita::Altura(GameCamera::default().height_world)),
        Alca::ZonaMorta(_) => Some(Escrita::ZonaMorta(CameraFollow::default().dead_zone)),
        Alca::Dentro => m
            .segue
            .as_ref()
            .map(|_| Escrita::FollowOffset(CameraFollow::default().offset)),
        Alca::LimiteCanto(_) | Alca::LimiteBorda(_) | Alca::LimiteCentro => {
            contorno.map(|(min, max)| Escrita::Limites(min, max))
        }
    }
}

/// ⭐ GEF 20 C6 — **o contorno do nível**: a caixa de mundo de toda imagem visível que anda com o mundo (o céu
/// parado e os fundos em parallax, `ScrollFactor ≠ 1`, ficam de fora — andam com a câmera e não dizem onde o
/// nível acaba). `None` = nenhuma imagem.
#[must_use]
pub fn contorno_do_nivel(world: &mut World) -> Option<([f32; 2], [f32; 2])> {
    let mut q = world.query::<(Entity, &ph2d_render::Sprite)>();
    let sprites: Vec<(Entity, [f32; 2])> = q.iter(world).map(|(e, s)| (e, s.size)).collect();
    let mut caixa: Option<([f32; 2], [f32; 2])> = None;
    for (e, tam) in sprites {
        let fundo = world
            .get::<ph2d_ecs::ScrollFactor>(e)
            .is_some_and(|s| s.k != [1.0, 1.0]);
        let escondida = world
            .get::<ph2d_ecs::Visibility>(e)
            .is_some_and(|v| v.hidden);
        let Some(t) = ph2d_ecs::world_transform(world, e).filter(|_| !fundo && !escondida) else {
            continue;
        };
        let (s, c) = t.rotation.sin_cos();
        let (hw, hh) = (
            tam[0] * t.scale.x.abs() * 0.5,
            tam[1] * t.scale.y.abs() * 0.5,
        );
        let meia = [c.abs() * hw + s.abs() * hh, s.abs() * hw + c.abs() * hh];
        let p = [t.translation.x, t.translation.y];
        let (lo, hi) = (
            [p[0] - meia[0], p[1] - meia[1]],
            [p[0] + meia[0], p[1] + meia[1]],
        );
        caixa = Some(match caixa {
            None => (lo, hi),
            Some((a, b)) => (
                [a[0].min(lo[0]), a[1].min(lo[1])],
                [b[0].max(hi[0]), b[1].max(hi[1])],
            ),
        });
    }
    caixa.filter(|(lo, hi)| hi[0] - lo[0] > LIMITE_VAO_MIN && hi[1] - lo[1] > LIMITE_VAO_MIN)
}

/// ⭐ GEF 20 C6 — **a semente dos `CameraLimits`** (a porta de anexar, `component_seed`): a cerca nasce no
/// contorno do nível, e não nos ±10 m de fábrica. Sem imagem nenhuma fica a de fábrica.
pub fn semeia_limites(sim: &mut ph2d_ecs::SimWorld, entity_bits: u64) {
    let Some((min, max)) = contorno_do_nivel(sim.world_mut()) else {
        return;
    };
    if let Some(mut l) = sim
        .world_mut()
        .get_mut::<CameraLimits>(Entity::from_bits(entity_bits))
    {
        *l = CameraLimits { min, max };
    }
}

/// Os passos do Shift — os MESMOS do scrub do Inspector (`ph2d-panel-inspector/src/sections/camera.rs`): a
/// altura e os limites de meio metro, os offsets de 10 cm, a zona morta de 5 %.
pub const PASSO_ALTURA: f32 = 0.5; // LITERAL-PX-OK: metros (o passo do campo Height)
pub const PASSO_LIMITES: f32 = 0.5; // LITERAL-PX-OK: metros (o passo dos campos dos Limits)
pub const PASSO_OFFSET: f32 = 0.1; // LITERAL-PX-OK: metros (o passo dos offsets)
pub const PASSO_ZONA: f32 = 0.05; // LITERAL-PX-OK: fracção (o passo da Dead Zone)

/// ⭐ GEF 20 C4 — **o Shift encaixa** o que o arrasto escreve no passo do campo que ele mexe.
#[must_use]
pub fn encaixa(e: Escrita) -> Escrita {
    let r = |v: f32, p: f32| (v / p).round() * p;
    let r2 = |v: [f32; 2], p: f32| [r(v[0], p), r(v[1], p)];
    match e {
        Escrita::Altura(h) => Escrita::Altura(r(h, PASSO_ALTURA).max(PASSO_ALTURA)),
        Escrita::Pose(p) => Escrita::Pose(r2(p, PASSO_OFFSET)),
        Escrita::FollowOffset(o) => Escrita::FollowOffset(r2(o, PASSO_OFFSET)),
        Escrita::Limites(min, max) => {
            let (min, mut max) = (r2(min, PASSO_LIMITES), r2(max, PASSO_LIMITES));
            for i in 0..2 {
                max[i] = max[i].max(min[i] + PASSO_LIMITES);
            }
            Escrita::Limites(min, max)
        }
        Escrita::ZonaMorta(z) => Escrita::ZonaMorta(r2(z, PASSO_ZONA)),
    }
}

/// ⭐ GEF 20 C4 — **o duplo-clique conta-se aqui** (o canvas não emite `DoubleClick`, como nas alças do Vector):
/// o último press numa alça (instante, câmera, alça).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct UltimoPress(Option<(u128, u64, Alca)>);

impl UltimoPress {
    /// O press de agora em `(camera, alca)`: `true` = é o 2.º na MESMA alça dentro da janela do duplo-clique.
    pub fn duplo(&mut self, now_ns: u128, camera: u64, alca: Alca) -> bool {
        let duplo = self.0.is_some_and(|(t, c, a)| {
            c == camera
                && a == alca
                && now_ns.saturating_sub(t) < ph2d_editor_core::interaction::DOUBLE_CLICK_WINDOW_NS
        });
        self.0 = (!duplo).then_some((now_ns, camera, alca));
        duplo
    }
}

/// ⭐ GEF 20 C4 — **a etiqueta da alça** (o nome do que ela mexe e o valor), junto do ponteiro sob o hover e a
/// meio do arrasto — o idioma das alças das formas do Vector (`shape_handles_face::readout`).
#[must_use]
pub fn etiqueta(
    m: &Moldura,
    alca: Alca,
    unidade: ph2d_editor_core::project::DisplayUnit,
    ppm: f32,
) -> String {
    let casas = usize::from(unidade == ph2d_editor_core::project::DisplayUnit::Meters);
    let v = |metros: f32| {
        format!(
            "{:.casas$} {}",
            unidade.from_meters(metros, ppm),
            unidade.suffix()
        )
    };
    let par = |p: [f32; 2]| {
        format!(
            "{:.casas$}, {:.casas$} {}",
            unidade.from_meters(p[0], ppm),
            unidade.from_meters(p[1], ppm),
            unidade.suffix()
        )
    };
    let tr = |k: &str, a: &[(&str, &dyn std::fmt::Display)]| ph2d_i18n::tr_with(k, a);
    match alca {
        Alca::Canto(_) => tr(
            "game.camera_frame.handle.zoom",
            &[("V", &v(m.meia[1] * 2.0))],
        ),
        Alca::Dentro => match m.segue.as_ref() {
            Some(s) => tr(
                "game.camera_frame.handle.follow_offset",
                &[("V", &par(s.offset))],
            ),
            None => tr("game.camera_frame.handle.move", &[("V", &par(m.pose))]),
        },
        Alca::ZonaMorta(i) => {
            let z = m.segue.as_ref().map_or(0.0, |s| s.zona[usize::from(i / 2)]);
            tr(
                "game.camera_frame.handle.dead_zone",
                &[("V", &format!("{:.0}", z * 100.0))],
            )
        }
        Alca::LimiteBorda(i) => {
            let (min, max) = m.limites.unwrap_or_default();
            let (k, x) = match i {
                0 => ("game.camera_frame.handle.limit_left", min[0]),
                1 => ("game.camera_frame.handle.limit_right", max[0]),
                2 => ("game.camera_frame.handle.limit_bottom", min[1]),
                _ => ("game.camera_frame.handle.limit_top", max[1]),
            };
            tr(k, &[("V", &v(x))])
        }
        Alca::LimiteCanto(i) => {
            let (min, max) = m.limites.unwrap_or_default();
            tr(
                "game.camera_frame.handle.limit_corner",
                &[("V", &par(cantos(min, max)[usize::from(i)]))],
            )
        }
        Alca::LimiteCentro => tr("game.camera_frame.handle.limit_move", &[]),
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
    /// `CameraFollow::dead_zone` (fracção da meia-janela, `0..=1`).
    ZonaMorta([f32; 2]),
}

/// A edição que uma escrita manda pela porta do Inspector (a pose escreve-se directa no `Transform`). ⚠️ Uma
/// só: dois lados da cerca em duas edições no mesmo quadro apagam-se (17b A3).
#[must_use]
pub fn edicoes(e: Escrita) -> Option<CameraFieldEdit> {
    match e {
        Escrita::Altura(h) => Some(CameraFieldEdit::Height(h)),
        Escrita::FollowOffset(o) => Some(CameraFieldEdit::FollowOffset(o)),
        Escrita::Limites(min, max) => Some(CameraFieldEdit::Limits(min, max)),
        Escrita::ZonaMorta(z) => Some(CameraFieldEdit::DeadZone(z)),
        Escrita::Pose(_) => None,
    }
}

/// Escreve `e` na câmera `camera`: os campos pela porta do Inspector (o desfazer fecha o gesto), a pose directa no
/// `Transform`.
pub fn aplica(
    world: &mut World,
    bus: &mut ph2d_editor_core::action_bus::ActionBus,
    camera: u64,
    e: Escrita,
) {
    if let Some(edit) = edicoes(e) {
        bus.push(
            ph2d_editor_core::action_bus::EditorAction::InspectorCameraEdit {
                entity_bits: camera,
                edit,
            },
        );
    }
    if let Escrita::Pose(xy) = e
        && let Some(mut t) = world.get_mut::<ph2d_ecs::Transform>(Entity::from_bits(camera))
    {
        t.translation.x = xy[0];
        t.translation.y = xy[1];
    }
}

/// ⭐ GEF 20 C2 — **o clique do Eyedropper do Target**: o objecto de cima sob o clique (`debaixo`, do topo para
/// baixo) que não é a própria câmera e tem nome passa a ser o seguido, pela porta do Inspector (um passo de
/// desfazer). `false` = nada escolhível ali.
pub fn escolhe_alvo(
    world: &World,
    bus: &mut ph2d_editor_core::action_bus::ActionBus,
    camera: u64,
    debaixo: &[u64],
) -> bool {
    let nome = debaixo.iter().filter(|&&b| b != camera).find_map(|&b| {
        world
            .get_entity(Entity::from_bits(b))
            .ok()
            .and_then(|e| e.get::<ph2d_ecs::Name>().map(|n| n.0.trim().to_string()))
            .filter(|n| !n.is_empty())
    });
    let Some(nome) = nome else {
        return false;
    };
    bus.push(
        ph2d_editor_core::action_bus::EditorAction::InspectorCameraEdit {
            entity_bits: camera,
            edit: CameraFieldEdit::Target(nome),
        },
    );
    true
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
            None => Escrita::Pose(pose_local(a, d)),
        },
        Alca::ZonaMorta(i) => {
            let eixo = usize::from(i / 2);
            let mut z = a.zona0;
            z[eixo] = if a.meia[eixo] > 0.0 {
                ((p[eixo] - a.centro_zona[eixo]).abs() / a.meia[eixo]).clamp(0.0, 1.0) // CLAMP-OK: a fracção da lei
            } else {
                z[eixo]
            };
            Escrita::ZonaMorta(z)
        }
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

/// A translação local que põe a câmera em `pose_mundo0 + d`. Sem pai, mundo = local. Com um pai que não se inverte
/// (escala zero), a pose fica onde estava.
fn pose_local(a: &Arrasto, d: [f32; 2]) -> [f32; 2] {
    let Some(pai) = a.pai else {
        return [a.pose0[0] + d[0], a.pose0[1] + d[1]];
    };
    let mundo = ph2d_ecs::Transform {
        translation: ph2d_core::Vec2::new(a.pose_mundo0[0] + d[0], a.pose_mundo0[1] + d[1]),
        ..ph2d_ecs::Transform::default()
    };
    ph2d_ecs::Transform::inverse_compose(pai, mundo)
        .map_or(a.pose0, |l| [l.translation.x, l.translation.y])
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
