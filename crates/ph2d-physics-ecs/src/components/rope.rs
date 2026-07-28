//! **A RODA DA CORDA** — a roldana como ENTIDADE (ADR-0131, W-Pulley W1).
//!
//! # Por que uma entidade, e não dois campos no joint
//!
//! É o argumento do W3 desta mesma linha, palavra por palavra: *um joint guardado
//! NO corpo só pode ser um por corpo*. Uma roldana guardada **dentro** do
//! [`super::super::PhysicsJoint`] tem o teto **dois** — e quatro dos oito pedidos
//! do artista caem junto com esse teto:
//!
//! | pedido | sai de graça da entidade |
//! |---|---|
//! | nº de roldanas, em tempo real | é um `spawn` — Hierarquia, nome, delete, undo e save |
//! | selecionar e posicionar | ela **tem `Transform`**: a posição é o gizmo que já existe |
//! | motor por roldana (W2) | campo no componente dela, não um 9º campo do joint |
//! | break por centro (W2) | idem |
//!
//! ⚠️ **O `Transform.translation` É o centro** — não há um segundo campo de
//! posição para discordar dele. É a mesma escolha que o joint faz com a âncora, e
//! pelo mesmo motivo: um fato, um dono.
//!
//! ⚠️ **A corda é apontada por NOME** (`stable_name_id`), nunca por bits de
//! entidade — o undo respawna tudo com bits novos, e bits dentro dos *bytes de um
//! componente* envenenam o próprio undo (o `canonicalize` ordena pelas bytes e
//! remapeia só o `parent`). É a mesma chave que `PhysicsJoint::body_a` usa.

use ph2d_ecs::{Component, SimComponent};
use serde::{Deserialize, Serialize};

/// **De que lado a corda passa nesta roldana** — o item (7) do pedido do artista,
/// mais o escape manual ao lado dele.
///
/// O algoritmo acerta a esmagadora maioria (ponto fixo sobre a poligonal dos
/// centros; ver `ph2d_physics::world::rope_route::resolve_sides`), e a lição que a
/// linha do Flip pagou é que **um matcher automático precisa do escape manual ao
/// lado**: quando ele erra, o artista corrige em vez de reorganizar a cena até o
/// algoritmo concordar.
///
/// `Over`/`Under` são ditos como um humano olha para a tela — *a corda passa por
/// CIMA da roldana* significa que ela toca o topo dela, e portanto que o centro
/// fica ABAIXO da corda.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WrapSide {
    /// Descoberto pela geometria. O default, e o que o artista quase nunca troca.
    #[default]
    Auto,
    /// A corda passa pelo TOPO: o centro da roldana fica abaixo dela.
    Over,
    /// A corda passa por BAIXO: o centro fica acima dela.
    Under,
}

impl WrapSide {
    /// A ordem dos chips da §13 — e a MESMA lista que o painel pinta.
    pub const ALL: [Self; 3] = [Self::Auto, Self::Over, Self::Under];

    /// O tag que atravessa a fronteira do painel, que não conhece este enum.
    #[must_use]
    pub fn tag(self) -> u8 {
        match self {
            Self::Auto => 0,
            Self::Over => 1,
            Self::Under => 2,
        }
    }

    /// A volta. ⚠️ **`None` para tag desconhecido, nunca um default silencioso** —
    /// dobrar o desconhecido no primeiro variant é o defeito que o `BodyKind` do
    /// W4 pagou: com dois variants é redundante, com o terceiro vira um chip que
    /// seleciona outra coisa.
    #[must_use]
    pub fn from_tag(tag: u8) -> Option<Self> {
        Self::ALL.get(tag as usize).copied()
    }
}

/// **Uma roldana de uma polia — componente de uma entidade própria.**
///
/// Presente numa entidade com `Transform`, ela entra na rota da corda cujo nome
/// bate com [`Self::rope`], na posição [`Self::order`].
///
/// ⚠️ **Componente novo NÃO custa bump de `PROJECT_SCHEMA`** — o blob é chaveado
/// pelo hash do nome do tipo, enquanto apendar campo a um componente existente é
/// postcard POSICIONAL. É a mesma razão pela qual a família das zonas tem sete
/// componentes em vez de uma struct com sete campos.
///
/// Config, nunca estado vivo de solver: o ângulo de giro da roda (que a wave do
/// desenho mostra) mora na PONTE, porque um ângulo serializado seria um passo de
/// undo por frame.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PulleyWheel {
    /// `stable_name_id` da CORDA (a entidade-joint [`JointKind::Pulley`]) a que
    /// esta roldana pertence.
    ///
    /// `0` (o default de um componente recém-anexado) não casa com nome nenhum, e
    /// isso é deliberado: uma roldana órfã é inerte, nunca uma roldana que se
    /// prende à primeira corda que aparecer.
    ///
    /// [`JointKind::Pulley`]: super::super::JointKind::Pulley
    pub rope: u64,
    /// A posição ao longo da corda, de A para B. Empates são desempatados pelo
    /// `stable_name_id` do nome — que é estável através de undo, ao contrário dos
    /// bits de entidade.
    pub order: u16,
    /// O RAIO, em metros — o pedido (3) do artista, e o número que substituiu o
    /// `ratio`.
    ///
    /// Ele muda três coisas de uma vez: por onde a corda passa (a superfície, não
    /// o centro), quanto de corda existe (o arco), e quanto a roda gira (a mesma
    /// corda por um raio maior gira menos).
    pub radius: f32,
    /// De que lado a corda passa aqui.
    pub wrap: WrapSide,
    /// **O MOTOR — esta roldana é um TAMBOR**, em radianos por segundo.
    ///
    /// Positivo **RECOLHE** (a corda encurta e o que estiver pendurado sobe);
    /// negativo **PAGA** corda. `0` é uma roldana livre, que é o que toda roldana
    /// é até alguém dizer o contrário — não há um segundo booleano *"tem motor?"*
    /// para discordar do número (a lei P7, a mesma do `∞ = off` da ruptura).
    ///
    /// ⚠️ **A grandeza é ANGULAR, e é isso que faz o diâmetro ser o CÂMBIO.** A
    /// corda anda `ω·r` — então o mesmo motor num tambor grande recolhe mais
    /// depressa, e é essa a vantagem que o `ratio` prometia e não entregava
    /// (§3 do plano). Com a corda a `v = ω·r` e a roda girando a `ω = v/r`, o
    /// desenho do giro que o W1 já faz mostra o tambor **na velocidade em que ele
    /// foi mandado girar**, sem uma segunda conta.
    pub motor_speed: f32,
}

impl Default for PulleyWheel {
    fn default() -> Self {
        Self {
            rope: 0,
            order: 0,
            radius: Self::DEFAULT_RADIUS,
            wrap: WrapSide::Auto,
            motor_speed: 0.0,
        }
    }
}

impl PulleyWheel {
    /// **O raio de uma roldana que o gesto de criação não dimensionou**, metros.
    ///
    /// Só é usado por quem anexa o componente à mão; o gesto de criar uma polia
    /// deriva o raio da GEOMETRIA que o artista já montou (uma fração da altura
    /// que ele deu às roldanas), então ele escala com a cena. Este é o piso para
    /// quando não há geometria de onde derivar — um palmo, do tamanho dos corpos
    /// que as cenas de smoke usam.
    pub const DEFAULT_RADIUS: f32 = 0.25;

    /// **O menor raio aceito**, metros — e ele é ZERO, de propósito.
    ///
    /// Raio zero é uma roldana-PONTO, o modelo que a v1 desta wave shipou, e a
    /// rota o reproduz EXATAMENTE (é a âncora de regressão da wave), então ele não
    /// é um caso degenerado a proibir. O que este piso recusa é o raio
    /// **negativo**, que inverteria a tangente — a corda passaria a nascer do lado
    /// oposto da roda, sem nada na tela dizendo por quê.
    pub const MIN_RADIUS: f32 = 0.0;

    /// Um componente que chegou de um arquivo (ou de um teste) com números que a
    /// rota não sabe usar.
    ///
    /// A porta de carga, como o `clamped()` do joint: um raio negativo inverteria
    /// a tangente e um `NaN` envenenaria a pose E o hash C9.
    #[must_use]
    pub fn clamped(self) -> Self {
        Self {
            radius: if self.radius.is_finite() {
                self.radius.max(Self::MIN_RADIUS)
            } else {
                Self::DEFAULT_RADIUS
            },
            // Um `NaN` no motor viraria um comprimento de corda `NaN` no primeiro
            // sub-passo, e daí a pose e o hash C9. Não há teto: um tambor rápido
            // demais é uma escolha do artista, e a corda que ele puxa é a mesma.
            motor_speed: if self.motor_speed.is_finite() {
                self.motor_speed
            } else {
                0.0
            },
            ..self
        }
    }
}

impl SimComponent for PulleyWheel {}
