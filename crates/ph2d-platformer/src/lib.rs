#![forbid(unsafe_code)]
//! **A LEI de um player de plataforma** — pura, sem rapier, sem ECS, sem shell.
//!
//! Dado *(config, o que o sensor viu, a velocidade do corpo, a gravidade, a
//! entrada, dt)*, esta crate responde **o que fazer com o corpo neste tick**.
//! Quem traduz isso em chamadas de solver é a ponte
//! (`ph2d-physics-ecs::bridge::player`); quem desenha e autora é a shell. Ver
//! [`docs/Physics/06_plano_player_plataforma.md`].
//!
//! # Por que uma CÁPSULA FLUTUANTE
//!
//! O personagem **não encosta no chão**: ele paira a [`RideConfig::float_height`]
//! sobre o que o sensor achou, e a **perna é uma mola**. A imprecisão de um corpo
//! dinâmico vem da negociação de contato — que o artista não controla —, e um
//! corpo que paira não tem contato de pé para negociar. Degrau, rampa e
//! plataforma móvel deixam de ser casos especiais: são o mesmo número
//! (`distance`) entrando na mesma mola.
//!
//! É o desenho do `bevy-tnua` e da cápsula flutuante do *Very Very Valet*, e a
//! pesquisa que o escolheu (com a tabela das cinco famílias) é o doc 05.
//!
//! # ⚠️ Duas grandezas, e a distinção NÃO é cosmética
//!
//! [`Motor`] carrega **aceleração** e **boost**, e a escolha entre elas é
//! por-termo, com critério:
//!
//! - **`accel`** é o regime CONTÍNUO (a mola, a caminhada). Vira força na ponte
//!   (`força = accel × massa`), então o solver a resolve **junto com** os
//!   contatos e os joints — é isso que mantém o personagem pendurável numa corda
//!   e empurrável por um caixote.
//! - **`boost`** é escrita DIRETA de velocidade, para o que precisa ser
//!   **exato**: amortecer a mola, parar no lugar, herdar a velocidade de uma
//!   plataforma. O `bevy-tnua` chegou aos dois pelo mesmo caminho e deixou o
//!   motivo em dois issues (#34 para a força, #39 para o boost).
//!
//! ⚠️ **E é o boost que torna o amortecimento independente de `dt`:** ele remove
//! `rel_v · damping` da velocidade de uma vez, então `damping = 1.0` amortece
//! por completo em UM tick. Acima de `1.0` ele começa a inverter a velocidade
//! em vez de matá-la, e em `2.0` a inversão é total — o limite de estabilidade
//! está medido em `ride::tests::the_damping_ceiling_is_where_the_boost_inverts`,
//! e é por isso que [`RideConfig::spring_damping`] é validado contra ele.
//!
//! # A composição
//!
//! Cada lei é uma função pura e independente ([`ride_spring`], [`walk`]), e
//! [`player_motor`] é a **porta única** que as soma e — mais importante —
//! responde **UMA vez** a pergunta *"isto aqui é chão?"* ([`footing`]). Duas
//! respostas para essa pergunta seriam a mola segurando o personagem numa parede
//! que a caminhada considera intransponível.

pub mod ride;
pub mod walk;

pub use ride::{RideConfig, ride_spring, within_reach};
pub use walk::{WalkConfig, walk};

/// Um vetor 2D em MUNDO (metros), na convenção do módulo (Y para cima).
pub type Vec2 = [f32; 2];

/// A perpendicular no sentido horário: com `up = [0, 1]` devolve `[1, 0]`.
///
/// É a porta única do *"para que lado é a direita?"* — do `up` sai o eixo
/// horizontal, e da normal do chão sai a tangente da rampa, com a MESMA função:
/// numa rampa que sobe para a direita a normal tomba para a esquerda, e a
/// tangente sai apontando para cima e para a direita, que é para onde se anda.
#[must_use]
pub fn perp_cw(v: Vec2) -> Vec2 {
    [v[1], -v[0]]
}

/// **O que o sensor de chão viu.** `None` no chamador significa *"nada ao
/// alcance"*, e a lei lê isso como estar no ar.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GroundSample {
    /// Distância do ponto de origem do raio até a superfície.
    pub distance: f32,
    /// A normal da superfície.
    ///
    /// ⚠️ Pode vir **degenerada** (`[0, 0]`) quando o raio nasce DENTRO da
    /// geometria — é o contrato do `cast_ray` do wrapper, que reporta a
    /// penetração em vez de a esconder. A [`footing`] a trata como chão plano:
    /// não sabemos a orientação, e a suposição menos daninha é a que deixa a
    /// mola empurrar o personagem para fora.
    pub normal: Vec2,
    /// **A velocidade do CHÃO no ponto de contato.**
    ///
    /// ⚠️ É ela que faz a plataforma móvel cair de graça: tudo nesta lei é
    /// medido *relativo ao chão*, então andar sobre um vagão é andar, e o vagão
    /// acelerando não derruba ninguém. Um chão estático manda `[0, 0]`.
    pub ground_velocity: Vec2,
}

/// A config inteira de um player — as duas metades que a [`footing`] precisa
/// consultar juntas.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PlayerConfig {
    /// A perna.
    pub ride: RideConfig,
    /// A caminhada.
    pub walk: WalkConfig,
}

impl PlayerConfig {
    /// O ponto de partida das duas metades — ⚠️ **não são defaults de produto**.
    pub const STARTING_POINT: Self = Self {
        ride: RideConfig::STARTING_POINT,
        walk: WalkConfig::STARTING_POINT,
    };
}

/// **A entrada do jogador neste tick.**
///
/// ⚠️ Não é config e não é componente: é o que o dedo do jogador estava fazendo.
/// Hoje a ponte a guarda como estado transiente (set-and-hold); a partir da W7
/// ela vem de uma **fita por tick**, o que torna o player uma função de
/// `(tick, fita)` e devolve o scrub bit-exato que o resto do módulo tem.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PlayerInput {
    /// O eixo de caminhada em `[-1, 1]`. Positivo é a direita.
    pub drive: f32,
}

/// **O que fazer com o corpo neste tick.** Ver a distinção accel/boost no topo.
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Motor {
    /// Aceleração desejada (m/s²) — vira FORÇA na ponte.
    pub accel: Vec2,
    /// Mudança instantânea de velocidade (m/s) — escrita direta.
    pub boost: Vec2,
}

impl Motor {
    /// Soma dois termos do motor.
    ///
    /// As leis são independentes e **aditivas** por desenho: é isso que permite
    /// gatear a mola sem a caminhada e vice-versa, e o que fará o pulo (W4)
    /// entrar sem reabrir nenhuma das duas.
    #[must_use]
    pub fn plus(self, other: Self) -> Self {
        Self {
            accel: [
                self.accel[0] + other.accel[0],
                self.accel[1] + other.accel[1],
            ],
            boost: [
                self.boost[0] + other.boost[0],
                self.boost[1] + other.boost[1],
            ],
        }
    }
}

/// **A amostra que a lei aceita como CHÃO** — a pergunta feita UMA vez.
///
/// Duas metades, e a segunda foi a entrega da W3: *perto o bastante* (a perna
/// alcança) **e** *rasa o bastante* (o personagem fica em pé nela).
///
/// ⚠️ **Uma parede de 80° está ao alcance do raio e não é chão.** Deixá-la
/// passar faria a mola segurar o personagem colado numa parede vertical, parado
/// no ar, porque a mola cancela a gravidade enquanto segura. Recusando-a, a
/// gravidade volta a agir inteira, o collider encosta na rampa e o personagem
/// **escorrega** — que é o que a Unity (`slopeLimit`) e o Godot
/// (`floor_max_angle`) fazem, e pela mesma razão.
///
/// A recusa é **binária de propósito**: é a resposta de toda a indústria, e uma
/// transição contínua faria a inclinação em que o personagem para de subir
/// depender da velocidade com que chegou.
#[must_use]
pub fn footing<'a>(
    cfg: &PlayerConfig,
    sample: Option<&'a GroundSample>,
    up: Vec2,
) -> Option<&'a GroundSample> {
    let s = sample?;
    if !within_reach(&cfg.ride, Some(s)) {
        return None;
    }
    // Normal degenerada (raio nascido dentro da geometria): trate como plano —
    // ver o aviso em `GroundSample::normal`.
    let n2 = s.normal[0] * s.normal[0] + s.normal[1] * s.normal[1];
    if n2 < 1.0e-6 {
        return Some(s);
    }
    let cos = s.normal[0] * up[0] + s.normal[1] * up[1];
    if cos < cfg.walk.max_slope_cos() {
        return None;
    }
    Some(s)
}

/// Estamos no chão? A pergunta pública, e a mesma que as leis consomem.
#[must_use]
pub fn is_grounded(cfg: &PlayerConfig, sample: Option<&GroundSample>, up: Vec2) -> bool {
    footing(cfg, sample, up).is_some()
}

/// **A PORTA ÚNICA** — o motor inteiro de um player neste tick.
///
/// Ela existe por uma razão que não é conveniência: [`footing`] é chamada
/// **uma** vez e o resultado é entregue às duas leis. Se cada uma perguntasse
/// por conta, uma rampa de 46° com `max_slope = 45` teria a mola a segurar o
/// personagem e a caminhada a considerá-lo no ar — o estado impossível que
/// nenhum gate de lei isolada consegue ver.
#[must_use]
pub fn player_motor(
    cfg: &PlayerConfig,
    sample: Option<&GroundSample>,
    input: PlayerInput,
    body_velocity: Vec2,
    gravity: Vec2,
    up: Vec2,
    dt: f32,
) -> Motor {
    let footing = footing(cfg, sample, up);
    let spring = ride_spring(&cfg.ride, footing, body_velocity, gravity, up);
    let step = walk(&cfg.walk, footing, body_velocity, up, input.drive, dt);
    spring.plus(step)
}

#[cfg(test)]
mod tests {
    use super::*;

    const UP: Vec2 = [0.0, 1.0];
    const G: Vec2 = [0.0, -9.81];
    const DT: f32 = 1.0 / 60.0;

    fn at(distance: f32, normal: Vec2) -> GroundSample {
        GroundSample {
            distance,
            normal,
            ground_velocity: [0.0, 0.0],
        }
    }

    /// Uma rampa RASA é chão; uma parede não é. O limite é o autorado.
    #[test]
    fn a_wall_is_not_ground() {
        let cfg = PlayerConfig::STARTING_POINT; // 45°
        let shallow = at(0.5, [-0.5, 0.866_025_4]); // 30°
        let steep = at(0.5, [-0.866_025_4, 0.5]); // 60°
        assert!(is_grounded(&cfg, Some(&shallow), UP), "30° e' chao");
        assert!(!is_grounded(&cfg, Some(&steep), UP), "60° nao e' chao");
        assert!(!is_grounded(&cfg, None, UP));
    }

    /// ⚠️ **A recusa da rampa alcança a MOLA, não só a caminhada.**
    ///
    /// É o gate da porta única: com a `footing` respondendo só à caminhada, a
    /// mola seguraria o personagem colado a uma parede — parado no ar, porque
    /// ela cancela a gravidade enquanto segura.
    #[test]
    fn the_spring_lets_go_of_a_wall() {
        let cfg = PlayerConfig::STARTING_POINT;
        let steep = at(cfg.ride.float_height, [-0.866_025_4, 0.5]); // 60°
        let m = player_motor(
            &cfg,
            Some(&steep),
            PlayerInput::default(),
            [0.0, 0.0],
            G,
            UP,
            DT,
        );
        assert_eq!(
            m,
            Motor::default(),
            "numa parede o motor inteiro se cala, e a gravidade age: {m:?}"
        );
    }

    /// A mesma rampa vira chão quando o artista sobe o limite — o número é dele.
    #[test]
    fn raising_the_limit_makes_the_ramp_walkable() {
        let mut cfg = PlayerConfig::STARTING_POINT;
        let steep = at(cfg.ride.float_height, [-0.866_025_4, 0.5]); // 60°
        assert!(!is_grounded(&cfg, Some(&steep), UP));
        cfg.walk.max_slope_deg = 70.0;
        assert!(is_grounded(&cfg, Some(&steep), UP));
    }

    /// Normal degenerada (raio nascido dentro da geometria) conta como chão
    /// plano — a suposição que deixa a mola empurrar o personagem para fora.
    #[test]
    fn a_degenerate_normal_counts_as_flat() {
        let cfg = PlayerConfig::STARTING_POINT;
        let inside = at(0.0, [0.0, 0.0]);
        assert!(is_grounded(&cfg, Some(&inside), UP));
        let m = player_motor(
            &cfg,
            Some(&inside),
            PlayerInput::default(),
            [0.0, 0.0],
            G,
            UP,
            DT,
        );
        assert!(
            m.accel[1] > 0.0,
            "a mola tem de empurrar para FORA: {:?}",
            m.accel
        );
    }

    /// A porta única SOMA as duas leis, sem uma comer a outra.
    #[test]
    fn the_door_sums_both_laws() {
        let cfg = PlayerConfig::STARTING_POINT;
        let ground = at(cfg.ride.float_height, [0.0, 1.0]);
        let input = PlayerInput { drive: 1.0 };
        let whole = player_motor(&cfg, Some(&ground), input, [0.0, 0.0], G, UP, DT);
        let spring = ride_spring(&cfg.ride, Some(&ground), [0.0, 0.0], G, UP);
        let step = walk(&cfg.walk, Some(&ground), [0.0, 0.0], UP, 1.0, DT);
        assert_eq!(whole, spring.plus(step));
        assert!(whole.accel[0] > 0.0, "anda");
        assert!(whole.accel[1] > 0.0, "e paira ao mesmo tempo");
    }
}
