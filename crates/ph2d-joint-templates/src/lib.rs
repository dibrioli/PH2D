//! **Os gabaritos de montagem de juntas** (ADR-0201; plano `docs/JointEditor/01_plano.md`).
//!
//! Um gabarito diz: *que peças* a máquina tem (os [`Role`]), *que juntas* as prendem (as
//! [`JointRule`]) e *com que números* cada junta nasce (o [`Tune`], sempre «o padrão do tipo mais
//! estas diferenças»). É dado puro: quem monta no mundo é `ph2d_app_physics::joint_assembly`.
//!
//! ⚠️ **A forma-guia de cada papel é também a peça de fábrica**: o desenho tracejado da aba, a
//! peça que «Make the parts» cria e o corpo com que os testes medem o gabarito saem do mesmo
//! [`Guide`] — um gabarito cujos números só funcionam com outras peças reprovaria no próprio teste.
//!
//! Os números de fábrica e a medição de cada um: `docs/JointEditor/02_gabaritos_medidos.md`.

mod catalog;

pub use catalog::ALL;
pub use ph2d_physics_ecs::JointKind;
use ph2d_physics_ecs::{MotorMode, PhysicsJoint};

/// A identidade GRAVADA de um gabarito (`JointAssembly::template`). **Append-only**: um número
/// nunca muda de dono nem é reusado (gate `the_template_ids_are_pinned`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TemplateId(pub u16);

/// A silhueta de um papel, em metros.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Guide {
    Box { half: [f32; 2] },
    Disc { radius: f32 },
}

impl Guide {
    /// As meias-extensões da caixa que envolve a forma.
    #[must_use]
    pub const fn half(self) -> [f32; 2] {
        match self {
            Self::Box { half } => half,
            Self::Disc { radius } => [radius, radius],
        }
    }
}

/// Um papel: uma peça da máquina («Rear wheel»).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Role {
    /// O nome que o artista lê, e o nome por omissão da peça de fábrica.
    pub name: &'static str,
    pub guide: Guide,
    /// O centro da forma-guia no desenho do gabarito, metros (`y = 0` é o chão do desenho).
    pub at: [f32; 2],
    /// `Some` = o papel aceita VÁRIAS peças em fila (as tábuas da ponte, os elos da corrente).
    pub many: Option<Many>,
    /// O papel pode ficar por conta do CENÁRIO (o batente pregado na parede): a junta nasce
    /// presa ao mundo, sem inventar um corpo.
    pub may_be_world: bool,
    /// A peça de fábrica deste papel não se move (o poste, o trilho).
    pub fixed: bool,
}

/// A fila de um papel de várias peças.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Many {
    /// Quantas a fábrica faz.
    pub count: u16,
    /// O passo entre os centros de duas vizinhas, metros.
    pub step: [f32; 2],
}

/// De que lado da junta um ponto se mede.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Side {
    A,
    B,
}

/// Um ponto NA peça, em frações da caixa dela: `(0, 0)` é o centro, `(-1, 0)` o meio da borda
/// esquerda, `(0, 1)` o meio do topo. Segue a peça que o artista soltou, do tamanho que ela tiver.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Spot {
    pub on: Side,
    pub u: f32,
    pub v: f32,
}

impl Spot {
    #[must_use]
    pub const fn a(u: f32, v: f32) -> Self {
        Self { on: Side::A, u, v }
    }
    #[must_use]
    pub const fn b(u: f32, v: f32) -> Self {
        Self { on: Side::B, u, v }
    }
}

/// Onde a junta prende.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Anchor {
    /// UM ponto para os dois lados (o pino, o eixo da roda, o trilho).
    At(Spot),
    /// Duas pontas (a mola, a corda, a haste): a de A e a de B.
    Ends(Spot, Spot),
    /// Onde as duas silhuetas se encontram — a regra de «Join Selected Bodies».
    Seam,
}

/// Que pares de peças uma regra prende.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Link {
    /// A peça do papel `a` com a do papel `b`.
    Pair,
    /// Cada peça do papel `a` (de várias) com a seguinte da fila; `b == a`.
    Chain,
    /// A peça de `a` com a PRIMEIRA da fila de `b`.
    ToFirst,
    /// A peça de `a` com a ÚLTIMA da fila de `b`.
    ToLast,
}

/// O comprimento de uma junta que tem um (corda, mola, haste).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Length {
    /// A distância entre as duas pontas como estão na cena, mais a folga.
    Gap { slack: f32 },
}

/// O motor de fábrica. A unidade é a do tipo: rad/s numa dobradiça ou roda, m/s num trilho.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Motor {
    pub mode: MotorMode,
    /// A velocidade (`Velocity`) ou o alvo (`Position`).
    pub value: f32,
    pub max_force: f32,
}

/// As diferenças que o gabarito impõe sobre o padrão do tipo. `None` = fica o padrão.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tune {
    /// Liga os limites com esta faixa (rad no pino; m no trilho e no curso da roda).
    pub limits: Option<[f32; 2]>,
    pub motor: Option<Motor>,
    /// `[stiffness, damping]`.
    pub spring: Option<[f32; 2]>,
    pub length: Option<Length>,
    /// Liga «Breakable» com `[força N, torque N·m]`.
    pub breaks: Option<[f32; 2]>,
}

impl Tune {
    pub const NONE: Self = Self {
        limits: None,
        motor: None,
        spring: None,
        length: None,
        breaks: None,
    };
}

/// Uma junta do gabarito.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JointRule {
    /// O nome do papel da JUNTA («Front wheel»): o título do cartão dela.
    pub name: &'static str,
    /// Índices em [`Template::roles`].
    pub a: u16,
    pub b: u16,
    pub link: Link,
    pub kind: JointKind,
    pub anchor: Anchor,
    /// A direção do eixo (a rotação da entidade-junta), rad — o trilho e a suspensão.
    pub axis: Option<f32>,
    pub tune: Tune,
    /// As regras do mesmo grupo nascem «iguais entre si» (as duas rodas do carro).
    pub group: u8,
}

impl JointRule {
    /// **A junta de fábrica desta regra**: o padrão do tipo com as diferenças do gabarito.
    /// `gap` é a distância entre as duas pontas na cena (só lida por [`Length::Gap`]).
    ///
    /// Não escreve identidade nem âncora (`body_*`, `local_*`, `anchored`, `assembly`, `role`):
    /// quem monta é que as conhece.
    #[must_use]
    pub fn joint(&self, gap: f32) -> PhysicsJoint {
        let mut j = PhysicsJoint::of_kind(self.kind);
        let t = &self.tune;
        if let Some([lo, hi]) = t.limits {
            j.limits_enabled = true;
            j.limit_min = lo;
            j.limit_max = hi;
        }
        if let Some(m) = t.motor {
            j.motor_enabled = true;
            j.motor_mode = m.mode;
            match m.mode {
                MotorMode::Velocity => j.motor_speed = m.value,
                MotorMode::Position => j.motor_target = m.value,
            }
            j.motor_max_force = m.max_force;
        }
        if let Some([k, c]) = t.spring {
            j.stiffness = k;
            j.damping = c;
        }
        if let Some(Length::Gap { slack }) = t.length {
            let len = (gap + slack).max(PhysicsJoint::MIN_LENGTH);
            j.rest_length = len;
            j.max_length = len;
        }
        if let Some([f, tq]) = t.breaks {
            j.break_enabled = true;
            j.break_force = f;
            j.break_torque = tq;
        }
        j.clamped()
    }
}

/// Um gabarito de montagem.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Template {
    pub id: TemplateId,
    /// A família no seletor «Joint» da aba (o tipo que dá nome ao gabarito).
    pub kind: JointKind,
    /// O nome no seletor «Template» e o nome por omissão da montagem («Car» ⇒ «Car 1»).
    pub name: &'static str,
    pub roles: &'static [Role],
    pub joints: &'static [JointRule],
}

impl Template {
    #[must_use]
    pub fn role(&self, i: u16) -> Option<&'static Role> {
        self.roles.get(usize::from(i))
    }

    #[must_use]
    pub fn rule(&self, i: u16) -> Option<&'static JointRule> {
        self.joints.get(usize::from(i))
    }

    /// **O nome da peça de fábrica** do papel `role` (a `n`-ésima, a contar de 0, num papel de
    /// várias): «Car Rear Wheel», «Door» (o papel que É o gabarito não se repete), «Plank 3».
    #[must_use]
    pub fn part_name(&self, role: u16, n: u16) -> String {
        let Some(r) = self.role(role) else {
            return self.name.to_owned();
        };
        let title: Vec<String> = r
            .name
            .split(' ')
            .map(|w| {
                let mut c = w.chars();
                c.next()
                    .map(|f| f.to_uppercase().chain(c).collect())
                    .unwrap_or_default()
            })
            .collect();
        let title = title.join(" ");
        if r.many.is_some() {
            format!("{title} {}", n + 1)
        } else if r.name.eq_ignore_ascii_case(self.name) {
            title
        } else {
            format!("{} {title}", self.name)
        }
    }
}

/// O gabarito gravado com este id.
#[must_use]
pub fn by_id(id: u16) -> Option<&'static Template> {
    ALL.iter().find(|t| t.id.0 == id)
}

/// O gabarito de uma família com este nome («Wheel», «Car»).
#[must_use]
pub fn by_name(kind: JointKind, name: &str) -> Option<&'static Template> {
    ALL.iter().find(|t| t.kind == kind && t.name == name)
}

/// Os gabaritos de uma família, na ordem do seletor.
pub fn of_kind(kind: JointKind) -> impl Iterator<Item = &'static Template> {
    ALL.iter().filter(move |t| t.kind == kind)
}
