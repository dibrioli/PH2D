#![forbid(unsafe_code)]
//! **A LEI de um player de plataforma** — pura, sem rapier, sem ECS, sem shell.
//!
//! Dado *(config, o que o sensor viu, a velocidade do corpo, a gravidade)*, esta
//! crate responde **o que fazer com o corpo neste tick**. Quem traduz isso em
//! chamadas de solver é a ponte (`ph2d-physics-ecs::bridge::player`); quem
//! desenha e autora é a shell. Ver [`docs/Physics/06_plano_player_plataforma.md`].
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
//! está medido em [`tests::the_damping_ceiling_is_where_the_boost_inverts`], e é
//! por isso que [`RideConfig::spring_damping`] é validado contra ele.

/// Um vetor 2D em MUNDO (metros), na convenção do módulo (Y para cima).
pub type Vec2 = [f32; 2];

/// **O que o sensor de chão viu.** `None` no chamador significa *"nada ao
/// alcance"*, e a lei lê isso como estar no ar.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GroundSample {
    /// Distância do ponto de origem do raio até a superfície.
    pub distance: f32,
    /// A normal da superfície.
    pub normal: Vec2,
    /// **A velocidade do CHÃO no ponto de contato.**
    ///
    /// ⚠️ É ela que faz a plataforma móvel cair de graça: tudo nesta lei é
    /// medido *relativo ao chão*, então andar sobre um vagão é andar, e o vagão
    /// acelerando não derruba ninguém. Um chão estático manda `[0, 0]`.
    pub ground_velocity: Vec2,
}

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
    /// ela é invertida. Ver o aviso no topo do módulo e o gate do teto.
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
}

/// **O que fazer com o corpo neste tick.** Ver a distinção accel/boost no topo.
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Motor {
    /// Aceleração desejada (m/s²) — vira FORÇA na ponte.
    pub accel: Vec2,
    /// Mudança instantânea de velocidade (m/s) — escrita direta.
    pub boost: Vec2,
}

/// Estamos no chão? A resposta que o resto da lei consome.
#[must_use]
pub fn is_grounded(cfg: &RideConfig, sample: Option<&GroundSample>) -> bool {
    match sample {
        Some(s) => s.distance <= cfg.float_height + cfg.cling_distance,
        None => false,
    }
}

/// **A MOLA.** O termo que faz o personagem pairar.
///
/// - `sample`: o que o sensor viu (`None` = ar).
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
    if !is_grounded(cfg, Some(s)) {
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
        // O amortecimento é BOOST — ver o aviso do topo.
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
