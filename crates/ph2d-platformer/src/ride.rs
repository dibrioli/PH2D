//! **A MOLA** — a perna da cápsula flutuante (W2).
//!
//! Módulo irmão do [`crate::walk`] pelo mesmo motivo que o resto deste repo
//! corta arquivos: **uma lei por módulo**. O pulo (W4) e a tolerância (W8) já
//! têm onde nascer sem que ninguém precise reabrir este.

use crate::{GroundSample, Motor, Vec2};

/// Os ganhos da perna.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RideConfig {
    /// A que altura o personagem paira, medida do ponto de origem do raio.
    pub float_height: f32,
    /// Quanto ACIMA da altura de repouso a mola ainda age.
    ///
    /// Dentro desta faixa a mola puxa o personagem de volta — inclusive para
    /// BAIXO. Fora dela ele está no ar, e a mola se cala: é esta distância que
    /// separa *"subi um degrau"* de *"pulei"*.
    pub cling_distance: f32,
    /// Rigidez, em aceleração-por-metro (não é N/m: a força sai na ponte).
    pub spring_strength: f32,
    /// Amortecimento, em **fração da velocidade relativa removida por tick**.
    ///
    /// ⚠️ `1.0` mata a velocidade relativa por completo num tick; acima disso
    /// ela é invertida. Ver o aviso no topo do `lib.rs` e o gate do teto.
    pub spring_damping: f32,
}

impl RideConfig {
    /// ⚠️ **O teto do amortecimento, MEDIDO** ([`tests::the_damping_ceiling_is_where_the_boost_inverts`]).
    ///
    /// Em `1.0` o boost remove exatamente a velocidade relativa. Em `2.0` ele a
    /// inverte inteira (`v → −v`), que é uma colisão perfeitamente elástica com
    /// um chão imaginário — o personagem pipoca. O teto que shipa é o ponto
    /// onde a mola ainda **converge**, e o `clamp` da porta o honra.
    pub const MAX_DAMPING: f32 = 1.0;

    /// Um perfil de partida — ⚠️ **NÃO são defaults de produto.**
    ///
    /// Os números que shipam saem da varredura da wave (o molde das tabelas do
    /// `GRAB_STIFFNESS`), e enquanto ela não existe estes são explicitamente um
    /// ponto de partida: `400` é a rigidez que o `bevy-tnua` usa, e o
    /// amortecimento está em metade do teto medido.
    pub const STARTING_POINT: Self = Self {
        float_height: 0.5,
        cling_distance: 0.25,
        spring_strength: 400.0,
        spring_damping: 0.5,
    };

    /// ⚠️ **A `float_height` MÍNIMA de uma cápsula, e ela é GEOMETRIA, não
    /// gosto** — o número que a W3 mediu e que a config de partida não conhece.
    ///
    /// O sensor mede na **vertical**, mas quem encosta na rampa é a cápsula ao
    /// longo da **normal** dela. Numa inclinação `θ` a folga perpendicular vale
    /// `float_height · cos θ` e a cápsula ocupa `radius + half_height · cos θ`,
    /// então flutuar de verdade exige
    ///
    /// ```text
    ///     float_height  >  half_height + radius / cos(max_slope)
    /// ```
    ///
    /// ## A tabela MEDIDA (cápsula `half_height = 0,3`, `radius = 0,2`)
    ///
    /// | `float_height` | inclinação máxima em que ela ainda flutua |
    /// |---|---|
    /// | **0,5** (o ponto de partida) | **NENHUMA** — ela fica tangente já no plano |
    /// | 0,7 | 60,0° |
    /// | 0,9 | 70,5° |
    /// | 1,2 | 77,2° |
    ///
    /// ⚠️ **A primeira linha é a que importa:** com o `STARTING_POINT` e essa
    /// cápsula o personagem **não paira** — ele fica encostado, e a mola passa a
    /// disputar com o solver de contato uma rampa que ela deveria só sobrevoar.
    /// A caminhada num plano ainda funciona (foi o que o W2 mediu), e é por isso
    /// que o defeito só apareceu quando a rampa entrou.
    ///
    /// A cura de PRODUTO é semear a `float_height` a partir do collider, como o
    /// collider já nasce da caixa do sprite — item nomeado para a W5, onde a
    /// autoria mora. Esta função é a porta única daquele número.
    ///
    /// ⚠️ **É a fórmula de uma CÁPSULA.** Uma caixa tem a sua
    /// (`half_y + half_x · tan θ`), porque a extensão dela ao longo da normal
    /// não é isotrópica. Quando a segunda forma precisar do número, ela ganha o
    /// próprio braço aqui — nunca uma segunda cópia noutro arquivo.
    #[must_use]
    pub fn min_float_height(half_height: f32, radius: f32, max_slope_cos: f32) -> f32 {
        if max_slope_cos <= 0.0 {
            return f32::INFINITY;
        }
        half_height + radius / max_slope_cos
    }
}

/// A metade DISTÂNCIA da pergunta *"estou no chão?"* — o sensor achou algo ao
/// alcance da perna?
///
/// ⚠️ **Não é a pergunta inteira**, e é por isso que ela mudou de nome nesta
/// wave: uma parede de 80° está ao alcance do raio e **não é chão**. A pergunta
/// completa é [`crate::footing`], que é quem os dois laws consomem — esta aqui
/// é a camada de dentro, e existe para que a mola nunca aja sobre uma amostra
/// longe, seja qual for o caminho pelo qual ela chegou.
#[must_use]
pub fn within_reach(cfg: &RideConfig, sample: Option<&GroundSample>) -> bool {
    match sample {
        Some(s) => s.distance <= cfg.float_height + cfg.cling_distance,
        None => false,
    }
}

/// **A MOLA.** O termo que faz o personagem pairar.
///
/// - `sample`: o chão que a lei ACEITA (o resultado de [`crate::footing`]).
/// - `body_velocity`: a velocidade do corpo, em mundo.
/// - `gravity`: a gravidade do mundo (m/s²), que a mola **compensa** enquanto
///   segura o personagem — sem isso ela teria de vencer o peso com o próprio
///   erro, e o personagem pairaria mais baixo do que a `float_height` pede,
///   por uma quantidade que depende da gravidade.
/// - `up`: a direção "para cima" (normalmente `[0, 1]`).
///
/// No ar devolve [`Motor::default`] — zerado. A gravidade extra de queda é do
/// pulo (W4), não da perna: são duas perguntas, e misturá-las faria a perna
/// mudar a altura do salto.
#[must_use]
pub fn ride_spring(
    cfg: &RideConfig,
    sample: Option<&GroundSample>,
    body_velocity: Vec2,
    gravity: Vec2,
    up: Vec2,
) -> Motor {
    let Some(s) = sample else {
        return Motor::default();
    };
    if !within_reach(cfg, Some(s)) {
        return Motor::default();
    }

    // Positivo = está BAIXO demais e a mola empurra para cima; negativo = está
    // alto demais dentro do `cling_distance` e ela puxa para baixo.
    let offset = cfg.float_height - s.distance;

    // ⚠️ Tudo RELATIVO ao chão: sobre uma plataforma que sobe, a velocidade do
    // corpo já contém a dela, e amortecer contra o referencial do mundo faria a
    // mola lutar contra a plataforma em vez de contra a oscilação.
    let rel = [
        body_velocity[0] - s.ground_velocity[0],
        body_velocity[1] - s.ground_velocity[1],
    ];
    let rel_along_up = rel[0] * up[0] + rel[1] * up[1];

    let damping = cfg.spring_damping.clamp(0.0, RideConfig::MAX_DAMPING);
    let spring = offset * cfg.spring_strength;

    Motor {
        // A mola + o cancelamento da gravidade.
        accel: [up[0] * spring - gravity[0], up[1] * spring - gravity[1]],
        // O amortecimento é BOOST — ver o aviso do `lib.rs`.
        boost: [
            -up[0] * rel_along_up * damping,
            -up[1] * rel_along_up * damping,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UP: Vec2 = [0.0, 1.0];
    const G: Vec2 = [0.0, -9.81];

    fn flat(distance: f32) -> GroundSample {
        GroundSample {
            distance,
            normal: [0.0, 1.0],
            ground_velocity: [0.0, 0.0],
        }
    }

    /// Sem chão ao alcance, a perna se cala — e é o que o pulo (W4) precisa para
    /// existir.
    #[test]
    fn no_ground_means_no_motor() {
        let cfg = RideConfig::STARTING_POINT;
        assert_eq!(ride_spring(&cfg, None, [0.0, 0.0], G, UP), Motor::default());
        // Além do alcance também é ar, mesmo com o sensor tendo achado algo.
        let far = flat(cfg.float_height + cfg.cling_distance + 0.01);
        assert_eq!(
            ride_spring(&cfg, Some(&far), [0.0, 0.0], G, UP),
            Motor::default()
        );
    }

    /// **Na altura exata, a mola só segura o peso** — nada de empurrão extra.
    ///
    /// É este o gate que prova a compensação da gravidade: sem ela a aceleração
    /// aqui seria zero, o personagem cairia, e só pararia onde o erro da mola
    /// igualasse o peso — mais baixo que a `float_height` pedida, por uma
    /// quantidade que depende da gravidade do mundo.
    #[test]
    fn at_rest_height_the_spring_only_carries_the_weight() {
        let cfg = RideConfig::STARTING_POINT;
        let m = ride_spring(&cfg, Some(&flat(cfg.float_height)), [0.0, 0.0], G, UP);
        assert!(
            (m.accel[1] - 9.81).abs() < 1.0e-4,
            "a aceleracao tem de cancelar a gravidade exatamente: {:?}",
            m.accel
        );
        assert_eq!(m.boost, [0.0, 0.0], "parado, nada a amortecer");
    }

    /// Baixo demais empurra para CIMA; alto demais (dentro do cling) puxa para
    /// BAIXO. A segunda metade é o que impede o personagem de subir escada
    /// flutuando.
    #[test]
    fn the_spring_pushes_both_ways() {
        let cfg = RideConfig::STARTING_POINT;
        let low = ride_spring(&cfg, Some(&flat(cfg.float_height - 0.1)), [0.0, 0.0], G, UP);
        let high = ride_spring(&cfg, Some(&flat(cfg.float_height + 0.1)), [0.0, 0.0], G, UP);
        assert!(low.accel[1] > 9.81, "baixo demais: empurra alem do peso");
        assert!(high.accel[1] < 9.81, "alto demais: alivia e deixa descer");
    }

    /// ⚠️ **O TETO DO AMORTECIMENTO, medido em vez de citado.**
    ///
    /// O boost remove `rel_v · d`. Em `d = 1` a velocidade relativa vai a zero;
    /// em `d = 2` ela seria invertida inteira. O `clamp` da porta segura em
    /// [`RideConfig::MAX_DAMPING`], então pedir 2.0 **não** inverte.
    #[test]
    fn the_damping_ceiling_is_where_the_boost_inverts() {
        let mut cfg = RideConfig::STARTING_POINT;
        let v = [0.0, -3.0]; // caindo a 3 m/s

        cfg.spring_damping = 1.0;
        let m = ride_spring(&cfg, Some(&flat(cfg.float_height)), v, G, UP);
        assert!(
            (v[1] + m.boost[1]).abs() < 1.0e-4,
            "em d=1 a velocidade relativa vai a ZERO: v={} boost={}",
            v[1],
            m.boost[1]
        );

        // Pedir o dobro do teto não pode inverter a velocidade: o clamp segura.
        cfg.spring_damping = 2.0;
        let m2 = ride_spring(&cfg, Some(&flat(cfg.float_height)), v, G, UP);
        let after = v[1] + m2.boost[1];
        assert!(
            after.abs() < 1.0e-4,
            "acima do teto o clamp segura em d=1 (nao inverte): sobrou {after}"
        );
    }

    /// ⚠️ **A altura de flutuação mínima é geometria** — e o ponto de partida
    /// NÃO a satisfaz para a cápsula que os smokes usam.
    ///
    /// O gate afirma as duas metades: a fórmula, e o fato desconfortável de que
    /// `STARTING_POINT.float_height` deixa aquela cápsula **tangente** ao plano.
    /// Sem a segunda, o número seria uma curiosidade; com ela, ele é uma dívida
    /// nomeada (a semeadura pelo collider, W5).
    #[test]
    fn the_float_height_has_a_geometric_floor() {
        // A tabela do doc de `min_float_height`, conferida.
        let flat = RideConfig::min_float_height(0.3, 0.2, 1.0);
        assert!((flat - 0.5).abs() < 1.0e-6, "no plano: {flat}");
        let at45 = RideConfig::min_float_height(0.3, 0.2, core::f32::consts::FRAC_1_SQRT_2);
        assert!((at45 - 0.582_842_7).abs() < 1.0e-5, "a 45°: {at45}");
        let at60 = RideConfig::min_float_height(0.3, 0.2, 0.5);
        assert!((at60 - 0.7).abs() < 1.0e-6, "a 60°: {at60}");

        // ⚠️ E o ponto de partida NÃO paira sobre essa cápsula nem no plano.
        assert!(
            RideConfig::STARTING_POINT.float_height <= flat,
            "o STARTING_POINT deixa a capsula (0.3, 0.2) TANGENTE: {} vs o minimo {flat}",
            RideConfig::STARTING_POINT.float_height
        );

        // Vertical é degenerado: nenhuma altura basta.
        assert!(RideConfig::min_float_height(0.3, 0.2, 0.0).is_infinite());
    }

    /// ⚠️ **A mola mede tudo RELATIVO ao chão** — a metade que faz a plataforma
    /// móvel funcionar sem uma linha de código de plataforma.
    ///
    /// Um personagem parado SOBRE uma plataforma que sobe a 2 m/s sobe a 2 m/s
    /// também. Se a mola amortecesse contra o mundo, ela leria isso como
    /// oscilação e lutaria contra a plataforma — o personagem afundaria nela.
    #[test]
    fn the_spring_damps_against_the_ground_not_the_world() {
        let cfg = RideConfig::STARTING_POINT;
        let rising = GroundSample {
            distance: cfg.float_height,
            normal: [0.0, 1.0],
            ground_velocity: [0.0, 2.0],
        };
        // O corpo já viaja com a plataforma: velocidade relativa ZERO.
        let m = ride_spring(&cfg, Some(&rising), [0.0, 2.0], G, UP);
        assert_eq!(
            m.boost,
            [0.0, 0.0],
            "viajando junto com o chao nao ha' o que amortecer"
        );

        // E um corpo parado no MUNDO está descendo em relação à plataforma ⇒ a
        // mola o acelera para cima para acompanhá-la.
        let m2 = ride_spring(&cfg, Some(&rising), [0.0, 0.0], G, UP);
        assert!(
            m2.boost[1] > 0.0,
            "parado no mundo, ele esta' AFUNDANDO na plataforma: {:?}",
            m2.boost
        );
    }
}
