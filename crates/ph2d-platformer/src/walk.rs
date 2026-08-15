//! **ANDAR** (W3) — o termo que leva a velocidade RELATIVA AO CHÃO até a que o
//! jogador pediu.
//!
//! # A lei, em uma frase
//!
//! Existe um **eixo** (a tangente do chão, ou a horizontal no ar), um **alvo**
//! (`drive × speed`, relativo ao chão) e uma **sobra** (`Δv`). Se um empurrão
//! deste tick passaria do alvo, a sobra é escrita como **boost**; senão ela vira
//! **força**, na aceleração do regime.
//!
//! # ⚠️ Três decisões, e cada uma tem um preço nomeado
//!
//! **1. A tangente do CHÃO, não a horizontal.** Andar numa rampa de 30° com o
//! eixo horizontal empurraria o personagem CONTRA a rampa, e quem o levantaria
//! seria a mola — a subida viraria função da rigidez da perna. Com a tangente, o
//! motor sobe a rampa e a mola não é chamada a resolver nada.
//!
//! **2. Parar é BOOST, e é ele que faz "preciso" significar alguma coisa.** Uma
//! força de freio para *perto* do lugar e o resto vira derrapada de meio tick —
//! é o issue #39 do `bevy-tnua`. A regra `|Δv| ≤ a·dt ⇒ escreva` é a mesma que
//! impede o personagem de passar da velocidade de cruzeiro, então **é uma regra,
//! não duas**.
//!
//! ⚠️ **E é ela que preserva a interação física que o Enio pediu.** Um boost
//! sobrescreve a velocidade, então um caixote empurrando o personagem parado
//! seria anulado — *se* o boost pudesse anular qualquer coisa. Ele não pode: só
//! dispara dentro de `a·dt`, que com a config de partida é **1,0 m/s por tick**.
//! Um empurrão mais forte que isso não é apagado, é **resistido** no orçamento
//! de aceleração do personagem — que é exatamente o que um corpo faz ao se
//! escorar. O caixote que empurra mais forte do que o personagem anda **ganha**.
//!
//! **3. O fator de mudança de direção NÃO é o `1.5 − 0.5·cos` do tnua**, e a
//! divergência é deliberada: num side-scroller o eixo de caminhada é **1-D**,
//! então aquele cosseno só assume ±1 e a fórmula vira um DEGRAU de 1× para 2× no
//! cruzamento do zero — um solavanco que o artista sente como engasgo. Aqui o
//! fator é `1 + |Δv| / (2·speed)`, saturado em 2: **reproduz os dois extremos que
//! o tnua quer** (2× num 180° cheio, 1× já no alvo) e é contínuo no meio.
//!
//! **4. FREAR não é ACELERAR** (`W-Brake`). Até esta wave a lei gastava
//! `cfg.acceleration` nos DOIS sentidos, e o fator de viragem — que cobre
//! **inverter** — não cobre **largar o direcional**: um personagem que arranca
//! rápido era obrigado a parar rápido. O [`WalkConfig::brake_scale`] é a fração
//! do orçamento usada quando o eixo está solto.
//!
//! ⚠️ **"Frear" é o eixo SOLTO, e não *"a velocidade está a cair"*** — é a
//! definição do `bUseSeparateBrakingFriction` do Unreal (*braking = quando não se
//! acelera*). Com o eixo apertado quem manda é o fator de viragem, que já existe
//! e já está medido; e um personagem acima do cruzeiro com o dedo no acelerador
//! está a ser **contido**, não a frear.
//!
//! **5. O CHÃO tem uma palavra sobre o orçamento** (`W-Surface`). Até esta wave
//! toda superfície valia o mesmo, e `friction` não aparecia uma única vez neste
//! crate — o que é honesto, porque a perna FLUTUA e o atrito de contato do solver
//! nunca alcança este personagem. O [`GroundSample::grip`] multiplica o orçamento
//! do tique: gelo é baixo, borracha é alto.
//!
//! ⚠️ **Ele multiplica os DOIS sentidos, e é isso que faz gelo ser gelo** —
//! arrancar e parar usam a mesma tração. Quem os separa é o item 4: com
//! `brake_scale` o artista escolhe *quanto* do escorregadio é "não consigo parar"
//! contra "não consigo arrancar", e sem ele a razão entre os dois seria fixa. É
//! por isso que esta lei chega DEPOIS daquela.

use crate::{Brink, GroundSample, Motor, Vec2, perp_cw};

/// **PARA ONDE ELE OLHA** — a direção que o eixo de caminhada deixou.
///
/// ⚠️ **Um eixo neutro NÃO apaga a direção:** parar de andar não é virar-se para
/// lugar nenhum. É por isso que a lei é *"o último sinal não-nulo"* e não
/// `drive.signum()`, que devolveria zero — e zero não é uma direção.
///
/// ⚠️ **Mora aqui porque o fato é do EIXO DE CAMINHADA**, e não do arranque, que
/// apenas o lê. Ver [`crate::PlayerState::facing`].
#[must_use]
pub fn facing_step(prev: f32, drive: f32) -> f32 {
    if drive > 0.0 {
        1.0
    } else if drive < 0.0 {
        -1.0
    } else {
        prev
    }
}

/// Como o personagem ANDA.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WalkConfig {
    /// Velocidade de cruzeiro, m/s — **relativa ao chão**.
    pub speed: f32,
    /// Aceleração no chão, m/s².
    pub acceleration: f32,
    /// Aceleração no ar (o controle aéreo), m/s².
    ///
    /// ⚠️ **Sem input, ela também FREIA** — é a mesma lei, e é o que o
    /// `bevy-tnua` e o atrito de ar do Celeste fazem. Quem quiser conservar o
    /// impulso do salto por completo põe `0.0` aqui; o número é do artista, não
    /// um caso especial do motor.
    pub air_acceleration: f32,
    /// **Quanto do orçamento de aceleração é gasto a FREAR** — a fração aplicada
    /// no chão quando o eixo está SOLTO (item 4 do topo do módulo).
    ///
    /// ⚠️ **`1` reduz LITERALMENTE ao mundo de antes desta wave** (`x * 1.0` é
    /// `x` em IEEE-754), e é isso que mantém o `physics_ecs_c9` byte-idêntico e a
    /// arte já autorada intocada.
    ///
    /// ⚠️ **`0` é legítimo e significa *não freia*** — não é *"desligado"*. É
    /// por isso que este número é um MULTIPLICADOR e não um segundo valor
    /// absoluto: num campo absoluto o `0` teria de significar *"usa a
    /// aceleração"*, e o valor que o artista quer para gelo ficaria inexprimível.
    ///
    /// ⚠️ **Não tem teto, e a ausência é MEDIDA:** a lei é auto-limitada por
    /// construção — quando a sobra cabe num tique ela é escrita EXATA (o `boost`),
    /// então subir este número só encurta a parada até ela ficar instantânea e
    /// nunca ultrapassa o alvo. O ponto de saturação é `speed / (turn·accel·dt)`,
    /// **função da config**, não uma constante que caiba num `MAX_*` (ver
    /// `measure_the_brake`).
    ///
    /// ⚠️ **O AR é isento, de propósito:** [`Self::air_acceleration`] já é a
    /// resposta do ar à mesma pergunta — o doc dele promete que `0` conserva o
    /// arco —, e um segundo número sobre aquele comportamento é a falha de duas
    /// portas. Há gate a pinar isto (`the_brake_leaves_the_air_alone`).
    pub brake_scale: f32,
    /// A inclinação máxima em que o personagem fica DE PÉ, em graus.
    ///
    /// Acima dela a superfície deixa de ser chão (ver [`crate::footing`]): a
    /// perna se cala, a gravidade age inteira, e o personagem **escorrega** pela
    /// rampa — a resposta do `slopeLimit` da Unity e do `floor_max_angle` do
    /// Godot, pela mesma razão.
    pub max_slope_deg: f32,
    /// **Ele PODE andar para fora de um patamar?** (`W-Brink` — o
    /// `bCanWalkOffLedges` do Unreal, que o serve para IA e para *andar com
    /// cuidado*.)
    ///
    /// ⚠️ **`true` é o mundo que já shipava, e devolve o alvo VERBATIM** — sem
    /// a trava armada o sensor nem sequer casta ([`crate::brink_probe_wanted`]),
    /// então uma cena que nunca a autora não paga um raio nem um `if`.
    ///
    /// ⚠️ **Ela governa ANDAR, e só isso.** Pular da beirada continua a
    /// funcionar (outra lei), ser LEVADO por uma esteira ou por uma plataforma
    /// para fora dela também — a caminhada mede tudo *relativo ao chão*, e o que
    /// esta trava corta é o alvo relativo, não o transporte. É o que o nome
    /// promete, e a alternativa (cortar o transporte) faria a trava desmentir a
    /// plataforma em que o personagem está de pé.
    ///
    /// ⚠️ **Ela NÃO alcança o arranque nem um empurrão** ([`crate::dash`],
    /// `launch_player`): os dois escrevem velocidade por cima da caminhada, de
    /// propósito, e um `bCanWalkOffLedges` que travasse um *dash* seria a trava
    /// a decidir sobre um gesto que não é andar.
    pub walk_off_ledges: bool,
}

impl WalkConfig {
    /// Um perfil de partida — ⚠️ **NÃO são defaults de produto** (a mesma nota
    /// do [`crate::RideConfig::STARTING_POINT`]).
    ///
    /// `60 m/s²` com `dt = 1/60` dá exatamente **1,0 m/s de janela de boost por
    /// tick**, que é o número citado no topo deste módulo.
    pub const STARTING_POINT: Self = Self {
        speed: 6.0,
        acceleration: 60.0,
        air_acceleration: 20.0,
        brake_scale: 1.0,
        max_slope_deg: 45.0,
        // ⚠️ **O mundo que já shipava** — ver o doc do campo.
        walk_off_ledges: true,
    };
}

/// **A fração do orçamento que ESTE tique pode gastar** — `1` em toda a parte
/// menos a travagem no chão (item 4 do topo do módulo).
///
/// ⚠️ **O piso em zero é load-bearing, não higiene:** um `brake_scale` negativo
/// faria `a` negativo, e `push = a · delta.signum()` empurraria para o lado
/// ERRADO — o personagem *acelera* ao largar o direcional. O clamp da UI é a
/// fronteira; este é o consumidor, e é aqui que a garantia tem de viver.
#[must_use]
fn brake_scale(cfg: &WalkConfig, grounded: bool, drive: f32) -> f32 {
    if grounded && drive == 0.0 && cfg.brake_scale.is_finite() {
        cfg.brake_scale.max(0.0)
    } else {
        1.0
    }
}

/// **O que a SUPERFÍCIE deixa o pé aproveitar** — o multiplicador do chão
/// achado (item 5 do topo do módulo), lido da amostra do sensor.
///
/// ⚠️ **O piso em zero é a mesma garantia do [`brake_scale`], e pelo mesmo
/// motivo:** um `grip` negativo faria `a` negativo, e `push = a · delta.signum()`
/// empurraria para o lado ERRADO — o personagem *acelera* na direção contrária à
/// que o dedo pede. Um `NaN` vindo do arquivo cai no neutro em vez de envenenar a
/// força (e daí o `Transform`, e daí o hash).
///
/// ⚠️ **Não há teto, e a ausência é a MESMA do `brake_scale`:** a lei é
/// auto-limitada por construção — quando a sobra cabe num tique ela é escrita
/// EXATA (o ramo do `boost`) —, então subir o `grip` só encurta o arranque e a
/// paragem até ficarem instantâneos, e nunca passa do alvo.
#[must_use]
fn surface_grip(s: &GroundSample) -> f32 {
    if s.grip.is_finite() {
        s.grip.max(0.0)
    } else {
        GroundSample::NEUTRAL_GRIP
    }
}

/// **ANDAR.** Ver a lei no topo do módulo.
///
/// - `footing`: o chão que a lei ACEITA (o resultado de [`crate::footing`]),
///   `None` no ar.
/// - `drive`: a entrada do jogador em `[-1, 1]` ao longo do eixo de caminhada.
/// - `carried`: o referencial em que medir a velocidade **no ar** (W10) — a
///   memória do chão que se deixou ([`crate::carried_frame`]). ⚠️ `[0, 0]` é o
///   comportamento de antes daquela wave AO BIT, e é o que uma cena de chão
///   estático produz sozinha.
/// - `dt`: o passo do relógio — **o mesmo** que a ponte usa para transformar
///   aceleração em impulso (`a · m · dt`). Se os dois divergirem, o limiar
///   força↔boost passa a descrever um tick que não existe.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn walk(
    cfg: &WalkConfig,
    footing: Option<&GroundSample>,
    body_velocity: Vec2,
    up: Vec2,
    drive: f32,
    carried: Vec2,
    dt: f32,
) -> Motor {
    if !dt.is_finite() || dt <= 0.0 {
        return Motor::default();
    }
    let drive = if drive.is_finite() {
        drive.clamp(-1.0, 1.0)
    } else {
        0.0
    };

    // O eixo, o referencial, o orçamento e a SUPERFÍCIE mudam com o regime — e é
    // a ÚNICA ramificação da lei.
    // ⚠️ No ar o referencial é o `carried` (W10), e ele é `[0, 0]` sempre que a
    // memória fechou — então esta linha é byte-idêntica ao que ela era enquanto
    // não houver plataforma móvel na cena.
    // ⚠️ **E o `grip` do AR sai daqui, não de um `if`:** não há superfície no ar,
    // então o neutro é a resposta certa — a representação apaga o caso especial,
    // e não existe um segundo lugar onde alguém possa esquecer de o neutralizar.
    let (axis, ground_v, accel, grip) = match footing {
        Some(s) => (
            perp_cw(s.normal),
            s.ground_velocity,
            cfg.acceleration,
            surface_grip(s),
        ),
        None => (
            perp_cw(up),
            carried,
            cfg.air_acceleration,
            GroundSample::NEUTRAL_GRIP,
        ),
    };
    if accel <= 0.0 || !accel.is_finite() {
        return Motor::default();
    }

    // ⚠️ Relativo ao CHÃO: andar sobre um vagão é andar, e ficar parado sobre
    // ele é ficar parado — a plataforma móvel cai de graça daqui.
    let rel = [
        body_velocity[0] - ground_v[0],
        body_velocity[1] - ground_v[1],
    ];
    let rel_along = rel[0] * axis[0] + rel[1] * axis[1];

    // ── A QUINA (`W-Brink`) ──────────────────────────────────────────────────
    // ⚠️ **A trava mexe no ALVO, e mais nada** — nem num ramo, nem numa força,
    // nem num limite de velocidade. E é isso que a torna correcta de graça: com
    // o alvo em zero e o personagem já a correr para a beirada, `delta` fica
    // negativo e a lei **trava com o orçamento inteiro**, que é exactamente o
    // que travar é. Um `drive = 0` daria a mesma direcção com a fração de
    // travagem, ou seja *mais devagar* justamente onde é preciso parar.
    //
    // ⚠️ **O `walk_off_ledges` devolve o alvo VERBATIM**, e é por isso que a
    // wave inteira é byte-idêntica para quem não a arma — a quina nem sequer é
    // castada nesse caso, mas a lei não depende disso para ser inerte.
    let target = drive * cfg.speed;
    let target = if cfg.walk_off_ledges {
        target
    } else {
        footing
            .map_or(Brink::NONE, |s| s.brink)
            .clamp_target(target)
    };
    let delta = target - rel_along;

    // O fator de mudança de direção (ver o item 3 do topo).
    let turn = if cfg.speed > 0.0 {
        (1.0 + delta.abs() / (2.0 * cfg.speed)).min(WalkConfig::MAX_TURN_BOOST)
    } else {
        1.0
    };
    let a = accel * turn * brake_scale(cfg, footing.is_some(), drive) * grip;

    // Um empurrão deste tick move a velocidade em `a·dt`. Se isso passaria do
    // alvo, escreva o que falta — senão empurre.
    if delta.abs() <= a * dt {
        Motor {
            accel: [0.0, 0.0],
            boost: [axis[0] * delta, axis[1] * delta],
        }
    } else {
        let push = a * delta.signum();
        Motor {
            accel: [axis[0] * push, axis[1] * push],
            boost: [0.0, 0.0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UP: Vec2 = [0.0, 1.0];
    const DT: f32 = 1.0 / 60.0;

    fn flat() -> GroundSample {
        GroundSample {
            grip: 1.0,
            distance: 0.5,
            normal: [0.0, 1.0],
            ground_velocity: [0.0, 0.0],
            one_way: false,
            brink: crate::Brink::NONE,
        }
    }

    /// Parado com o dedo no acelerador, a lei EMPURRA — e no eixo do chão.
    #[test]
    fn a_standing_body_is_pushed_along_the_ground() {
        let cfg = WalkConfig::STARTING_POINT;
        let m = walk(&cfg, Some(&flat()), [0.0, 0.0], UP, 1.0, [0.0, 0.0], DT);
        assert!(m.accel[0] > 0.0, "empurra para a direita: {:?}", m.accel);
        assert_eq!(m.accel[1], 0.0, "chao plano: nada na vertical");
        assert_eq!(m.boost, [0.0, 0.0], "longe do alvo, e' FORCA");
    }

    /// **Na velocidade de cruzeiro a lei para de empurrar** — e sem oscilar em
    /// torno dela, porque a última fração vira boost exato.
    #[test]
    fn cruising_speed_is_reached_and_not_passed() {
        let cfg = WalkConfig::STARTING_POINT;
        let at = walk(
            &cfg,
            Some(&flat()),
            [cfg.speed, 0.0],
            UP,
            1.0,
            [0.0, 0.0],
            DT,
        );
        assert_eq!(at.accel, [0.0, 0.0], "no alvo nao ha' o que empurrar");
        assert_eq!(at.boost, [0.0, 0.0]);

        // Uma fração abaixo: a sobra cabe num tick ⇒ escreve o exato.
        let near = walk(
            &cfg,
            Some(&flat()),
            [cfg.speed - 0.1, 0.0],
            UP,
            1.0,
            [0.0, 0.0],
            DT,
        );
        assert_eq!(near.accel, [0.0, 0.0], "perto do alvo e' BOOST, nao forca");
        assert!(
            (near.boost[0] - 0.1).abs() < 1.0e-5,
            "o boost e' exatamente o que falta: {:?}",
            near.boost
        );
    }

    /// ⚠️ **A janela do boost é `a·dt`, e é ela que preserva a interação
    /// física** — um empurrão maior que isso é resistido, nunca apagado.
    #[test]
    fn a_shove_bigger_than_one_ticks_budget_is_resisted_not_erased() {
        let cfg = WalkConfig::STARTING_POINT;
        // Sem input: o alvo é ficar parado (relativo ao chão).
        let window = cfg.acceleration * DT;
        assert!(
            (window - 1.0).abs() < 1.0e-5,
            "a janela da config de partida e' 1 m/s por tick: {window}"
        );

        // Empurrão DENTRO da janela: some neste tick (o personagem se escora).
        let small = walk(&cfg, Some(&flat()), [0.5, 0.0], UP, 0.0, [0.0, 0.0], DT);
        assert!(
            (small.boost[0] + 0.5).abs() < 1.0e-5,
            "cancela o empurrao pequeno: {:?}",
            small.boost
        );

        // Empurrão MAIOR: nada de boost — ele é freado no orçamento, e o que
        // sobra continua movendo o personagem. É isto que faz um caixote pesado
        // ganhar de um personagem que anda.
        let big = walk(&cfg, Some(&flat()), [5.0, 0.0], UP, 0.0, [0.0, 0.0], DT);
        assert_eq!(big.boost, [0.0, 0.0], "acima da janela nao se escreve nada");
        assert!(big.accel[0] < 0.0, "freia, mas nao apaga: {:?}", big.accel);
        let removed = big.accel[0].abs() * DT;
        assert!(
            removed < 5.0,
            "um tick nao pode comer os 5 m/s inteiros: removeu {removed}"
        );
    }

    /// **Virar custa mais aceleração que arrancar** — o fator do topo do módulo.
    ///
    /// E o gate afirma os DOIS extremos, porque é o que separa esta lei de um
    /// ganho constante: no alvo o fator é 1, num 180° cheio ele é 2.
    #[test]
    fn reversing_gets_twice_the_acceleration_of_starting() {
        let cfg = WalkConfig::STARTING_POINT;
        let from_rest = walk(&cfg, Some(&flat()), [0.0, 0.0], UP, 1.0, [0.0, 0.0], DT);
        let reversing = walk(
            &cfg,
            Some(&flat()),
            [-cfg.speed, 0.0],
            UP,
            1.0,
            [0.0, 0.0],
            DT,
        );

        assert!(
            (from_rest.accel[0] - cfg.acceleration * 1.5).abs() < 1.0e-3,
            "parado: meia janela de sobra ⇒ 1,5×: {:?}",
            from_rest.accel
        );
        assert!(
            (reversing.accel[0] - cfg.acceleration * WalkConfig::MAX_TURN_BOOST).abs() < 1.0e-3,
            "180° cheio ⇒ 2×: {:?}",
            reversing.accel
        );
        assert!(
            reversing.accel[0] > from_rest.accel[0] * 1.2,
            "virar TEM de responder mais que arrancar: {} vs {}",
            reversing.accel[0],
            from_rest.accel[0]
        );
    }

    /// ⚠️ **O eixo é a tangente do CHÃO** — numa rampa, andar SOBE.
    ///
    /// O oráculo não é o sinal do X (que também é positivo numa horizontal): é a
    /// componente VERTICAL, que só existe se o eixo seguir a rampa.
    #[test]
    fn walking_up_a_ramp_climbs_it() {
        let cfg = WalkConfig::STARTING_POINT;
        // Rampa de 30° subindo para a direita: a normal tomba para a esquerda.
        let s = 0.5_f32; // sin 30°
        let c = 0.866_025_4_f32; // cos 30°
        let ramp = GroundSample {
            grip: 1.0,
            distance: 0.5,
            normal: [-s, c],
            ground_velocity: [0.0, 0.0],
            one_way: false,
            brink: crate::Brink::NONE,
        };
        let m = walk(&cfg, Some(&ramp), [0.0, 0.0], UP, 1.0, [0.0, 0.0], DT);
        assert!(m.accel[0] > 0.0, "vai para a direita: {:?}", m.accel);
        assert!(
            m.accel[1] > 0.0,
            "e SOBE — o eixo e' a tangente, nao a horizontal: {:?}",
            m.accel
        );
        // A razão entre as componentes É a inclinação da rampa.
        let slope = m.accel[1] / m.accel[0];
        assert!(
            (slope - s / c).abs() < 1.0e-3,
            "a direcao tem de ser a da rampa: {slope} vs {}",
            s / c
        );
    }

    /// **Andar sobre um vagão é andar** — a velocidade é medida relativa a ele.
    #[test]
    fn the_target_is_relative_to_the_ground() {
        let cfg = WalkConfig::STARTING_POINT;
        let wagon = GroundSample {
            grip: 1.0,
            distance: 0.5,
            normal: [0.0, 1.0],
            ground_velocity: [4.0, 0.0],
            one_way: false,
            brink: crate::Brink::NONE,
        };
        // Viajando com o vagão, sem input: nada a fazer.
        let riding = walk(&cfg, Some(&wagon), [4.0, 0.0], UP, 0.0, [0.0, 0.0], DT);
        assert_eq!(riding.accel, [0.0, 0.0]);
        assert_eq!(riding.boost, [0.0, 0.0]);

        // Andando para a frente EM CIMA dele: o alvo é `4 + speed`, não `speed`.
        let walking = walk(
            &cfg,
            Some(&wagon),
            [4.0 + cfg.speed, 0.0],
            UP,
            1.0,
            [0.0, 0.0],
            DT,
        );
        assert_eq!(
            walking.accel,
            [0.0, 0.0],
            "ja' esta' na velocidade de cruzeiro RELATIVA ao vagao"
        );
    }

    /// No ar o orçamento é outro, e o eixo volta a ser a horizontal.
    #[test]
    fn air_control_uses_its_own_budget_and_the_horizontal_axis() {
        let mut cfg = WalkConfig::STARTING_POINT;
        cfg.air_acceleration = 20.0;
        let m = walk(&cfg, None, [0.0, -8.0], UP, 1.0, [0.0, 0.0], DT);
        assert!(m.accel[0] > 0.0, "controle aereo empurra: {:?}", m.accel);
        assert_eq!(
            m.accel[1], 0.0,
            "e NUNCA na vertical — o arco do pulo e' do pulo"
        );
        // O orçamento do ar é menor que o do chão, na mesma situação.
        let ground = walk(&cfg, Some(&flat()), [0.0, 0.0], UP, 1.0, [0.0, 0.0], DT);
        assert!(
            m.accel[0] < ground.accel[0],
            "ar {} tem de ser menor que chao {}",
            m.accel[0],
            ground.accel[0]
        );
    }

    /// `air_acceleration = 0` conserva o impulso do salto — o escape que a doc
    /// do campo promete, verificado.
    #[test]
    fn zero_air_acceleration_conserves_the_arc() {
        let mut cfg = WalkConfig::STARTING_POINT;
        cfg.air_acceleration = 0.0;
        let m = walk(&cfg, None, [3.0, -8.0], UP, 0.0, [0.0, 0.0], DT);
        assert_eq!(m, Motor::default(), "sem orcamento, sem motor: {m:?}");
    }

    /// ⚠️ **O referencial carregado (W10) só existe NO AR** — no chão quem manda
    /// é o chão de verdade, e um `carried` que vazasse para lá faria o
    /// personagem andar contra a plataforma em que está de pé.
    ///
    /// **Mutação que deve sangrar:** usar `carried` também no braço `Some(s)`.
    #[test]
    fn the_carried_frame_is_the_airs_and_never_the_grounds() {
        let cfg = WalkConfig::STARTING_POINT;
        // No chão, com uma memória absurda: ela não pode ser consultada.
        let on_ground = walk(&cfg, Some(&flat()), [0.0, 0.0], UP, 0.0, [9.0, 0.0], DT);
        let control = walk(&cfg, Some(&flat()), [0.0, 0.0], UP, 0.0, [0.0, 0.0], DT);
        assert_eq!(
            on_ground, control,
            "de pe' o referencial e' o do chao, e mais nada: {on_ground:?}"
        );
    }

    /// **Sair de um vagão a 4 m/s não apaga os 4 m/s** (W10).
    ///
    /// Sem a memória, o controle aéreo mede no MUNDO e passa a frear a
    /// velocidade que a plataforma deu — o gate compara os dois referenciais na
    /// mesma situação, e o oráculo é o SINAL da aceleração.
    #[test]
    fn the_air_control_measures_in_the_frame_it_left() {
        let mut cfg = WalkConfig::STARTING_POINT;
        cfg.air_acceleration = 20.0;
        // Voando a 4 m/s, sem input, tendo saído de um vagão que ia a 4 m/s.
        let forgetful = walk(&cfg, None, [4.0, 2.0], UP, 0.0, [0.0, 0.0], DT);
        assert!(
            forgetful.accel[0] < 0.0 || forgetful.boost[0] < 0.0,
            "sem memoria, o ar FREIA a velocidade do vagao: {forgetful:?}"
        );

        let carried = walk(&cfg, None, [4.0, 2.0], UP, 0.0, [4.0, 0.0], DT);
        assert_eq!(
            carried,
            Motor::default(),
            "com a memoria, viajar com o vagao e' estar parado: {carried:?}"
        );

        // E o alvo com input soma ao referencial, como no chão.
        let driving = walk(&cfg, None, [4.0 + cfg.speed, 0.0], UP, 1.0, [4.0, 0.0], DT);
        assert_eq!(
            driving,
            Motor::default(),
            "o alvo e' `vagao + speed`, nao `speed`: {driving:?}"
        );
    }

    /// O limite de rampa em graus vira cosseno pela porta única — e os dois
    /// extremos são os que a `footing` compara.
    #[test]
    fn the_slope_limit_is_a_cosine() {
        let mut cfg = WalkConfig::STARTING_POINT;
        cfg.max_slope_deg = 0.0;
        assert!((cfg.max_slope_cos() - 1.0).abs() < 1.0e-6);
        cfg.max_slope_deg = 90.0;
        assert!(cfg.max_slope_cos().abs() < 1.0e-6);
        cfg.max_slope_deg = 60.0;
        assert!((cfg.max_slope_cos() - 0.5).abs() < 1.0e-5);
    }
}

#[cfg(test)]
#[path = "walk_sim_tests.rs"]
mod sim_tests;

#[cfg(test)]
#[path = "walk_brake_tests.rs"]
mod brake_tests;

#[cfg(test)]
#[path = "walk_grip_tests.rs"]
mod grip_tests;

#[cfg(test)]
#[path = "walk_brink_tests.rs"]
mod brink_tests;

/// **A trava de beirada** (`W-Brink`) — irmã por RESPONSABILIDADE: aqui mora
/// *quanto empurrar*, lá *até onde é seguro andar*.
#[path = "walk_brink.rs"]
mod brink;
pub use brink::brink_probe_wanted;
