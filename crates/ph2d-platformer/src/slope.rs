//! **O que o sensor viu, DEPOIS da lei** — o veredito de chão e o que ele
//! proíbe.
//!
//! Módulo irmão do [`crate`] (o cap de LOC do arquivo), e o corte é por
//! ASSUNTO: aqui mora a única pergunta que a porta única faz **uma vez** —
//! *"isto aqui é chão?"* — e a consequência que ela passou a ter na W9, quando a
//! resposta deixou de ser um `bool` disfarçado de `Option` e virou as três que
//! ela sempre teve.
//!
//! O `lib.rs` fica com o que ele é: os tipos compartilhados e a porta que soma
//! as leis.

use crate::{GroundSample, Motor, PlayerConfig, Vec2, perp_cw, within_reach};

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
    footing_verdict(cfg, sample, up).ground()
}

/// **As TRÊS respostas do sensor, não duas** (W9).
///
/// ⚠️ *"Não é chão"* colapsava dois estados que pedem coisas OPOSTAS da
/// caminhada: **estar no ar** (não há em que se apoiar) e **estar encostado numa
/// rampa íngreme demais** (há, e é justamente por isso que empurrar contra ela
/// carrega o personagem para cima). Enquanto os dois eram o mesmo `None`, o
/// empurrão horizontal do modo-ar escalava a rampa que a perna tinha acabado de
/// recusar — e o número que o artista escreve em *Max Slope* deixava de ser o
/// número que o produto honra.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Footing<'a> {
    /// Nada ao alcance da perna.
    Airborne,
    /// Há superfície ao alcance, e ela é **íngreme demais** para se ficar em pé.
    Steep(&'a GroundSample),
    /// Chão: perto o bastante **e** raso o bastante.
    Ground(&'a GroundSample),
}

impl<'a> Footing<'a> {
    /// O chão que a lei ACEITA — a resposta que [`ride_spring`], [`walk`] e
    /// [`jump_step`] consomem.
    #[must_use]
    pub fn ground(self) -> Option<&'a GroundSample> {
        match self {
            Footing::Ground(s) => Some(s),
            _ => None,
        }
    }

    /// A superfície ao alcance que a lei RECUSOU por inclinação.
    #[must_use]
    pub fn steep(self) -> Option<&'a GroundSample> {
        match self {
            Footing::Steep(s) => Some(s),
            _ => None,
        }
    }

    /// **A POSTURA, sem a amostra** — a mesma resposta, publicável
    /// (`W-PlayerOut`).
    ///
    /// ⚠️ **Ela existe porque o readout do player não pode emprestar nada:** o
    /// [`Footing`] carrega uma referência à amostra do tique, e um estado que se
    /// publica sobrevive ao tique que o produziu.
    ///
    /// ⚠️ **E são TRÊS, não um `bool`** — quem publicasse *"estou no chão?"*
    /// re-colapsaria exactamente o que a W9 separou: *no ar* e *encostado numa
    /// rampa íngreme demais* pedem coisas OPOSTAS da caminhada, e um consumidor
    /// de animação que não os distingue toca a queda em quem está a escorregar.
    #[must_use]
    pub fn kind(self) -> FootingKind {
        match self {
            Footing::Airborne => FootingKind::Airborne,
            Footing::Steep(_) => FootingKind::Steep,
            Footing::Ground(_) => FootingKind::Ground,
        }
    }
}

/// A [`Footing`] sem a amostra — ver [`Footing::kind`].
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum FootingKind {
    /// Nada ao alcance da perna.
    #[default]
    Airborne,
    /// Há superfície, e ela é íngreme demais para se ficar em pé.
    Steep,
    /// Chão.
    Ground,
}

/// A classificação, feita **UMA vez** — [`footing`] e [`Footing::steep`] são
/// duas VISTAS dela, nunca dois testes.
///
/// ⚠️ Duas cópias do par *alcance + inclinação* divergiriam no dia em que uma
/// delas ganhasse um caso, e a divergência seria um personagem que a perna
/// recusa e o empurrão aceita — que é exatamente o defeito que esta função
/// existe para tornar inexprimível.
#[must_use]
pub fn footing_verdict<'a>(
    cfg: &PlayerConfig,
    sample: Option<&'a GroundSample>,
    up: Vec2,
) -> Footing<'a> {
    let Some(s) = sample else {
        return Footing::Airborne;
    };
    if !within_reach(&cfg.ride, Some(s)) {
        return Footing::Airborne;
    }
    // Normal degenerada (raio nascido dentro da geometria): trate como plano —
    // ver o aviso em `GroundSample::normal`.
    let n2 = s.normal[0] * s.normal[0] + s.normal[1] * s.normal[1];
    if n2 < 1.0e-6 {
        return Footing::Ground(s);
    }
    let cos = s.normal[0] * up[0] + s.normal[1] * up[1];
    if cos < cfg.walk.max_slope_cos() {
        return Footing::Steep(s);
    }
    Footing::Ground(s)
}

/// **Um empurrão não sobe o que a perna RECUSOU** (W9).
///
/// # O defeito, MEDIDO (2026-08-04, report do Enio)
///
/// Com o limite autorado em **45°** o personagem ainda subia uma rampa de
/// **50°** a `+4,0 m` em 3 s. A perna estava certa — o controle prova que o
/// número move a fronteira dela (rampa 55°: limite 54 ⇒ `+0,17 m`, limite 56 ⇒
/// `+13,25 m`). Quem escalava era o **modo-ar**: recusada a superfície, a
/// caminhada troca o eixo da rampa pela HORIZONTAL, e um empurrão horizontal
/// contra uma rampa é redirecionado morro acima pelo contato.
///
/// A ablação por ENTRADA fecha a atribuição sem deixar dúvida — mesma rampa,
/// mesmo limite, só o controle aéreo mudando:
///
/// | rampa | `air = 20` (o default) | `air = 5` | `air = 0` |
/// |---|---|---|---|
/// | 46° | **+4,375 m** | +0,041 m | −20,826 m |
/// | 50° | **+4,010 m** | +0,004 m | −28,873 m |
/// | 52° | **+3,367 m** | −0,027 m | −33,369 m |
///
/// ⚠️ **O teto efetivo era função de OUTRO knob** (`air_acceleration`), que é a
/// assinatura de um bug de DESIGN e não de afinação
/// ([[feedback_ergonomics_verdict_is_a_design_bug]]): mexer na aceleração aérea
/// movia, em silêncio, a inclinação máxima que o personagem sobe.
///
/// # A lei
///
/// Numa superfície recusada, o termo de CAMINHADA perde o que apontaria **morro
/// acima**; morro abaixo passa inteiro. É o `slopeLimit` da Unity e o
/// `floor_max_angle` do Godot ditos no vocabulário deste motor.
///
/// ⚠️ **Zera o CANAL inteiro, não só a componente tangencial**, e é decisão de
/// produto com preço nomeado: guardar a componente que entra na rampa manteria o
/// personagem *prensado* contra ela, e o atrito extra o faria **grudar** em vez
/// de escorregar — o oposto do que a cena promete (*"ele NAO sobe -- escorrega"*).
/// Uma pessoa diante de uma ladeira que não dá para subir não se enfia nela.
///
/// ⚠️ **`None` devolve o motor INTOCADO**, bit a bit: no ar e no chão esta lei
/// não existe. É o que mantém tudo o que a W3..W8 shipou byte-idêntico.
#[must_use]
pub fn no_uphill(motor: Motor, steep: Option<&GroundSample>, up: Vec2) -> Motor {
    let Some(s) = steep else {
        return motor;
    };
    // A tangente que aponta para CIMA. `perp_cw` é a porta única do *"para que
    // lado?"*, e o sinal se decide contra o `up` — não contra o eixo Y.
    let t = perp_cw(s.normal);
    let t_up = t[0] * up[0] + t[1] * up[1];
    let uphill = if t_up > 0.0 {
        t
    } else if t_up < 0.0 {
        [-t[0], -t[1]]
    } else {
        // Normal paralela ao `up`: a superfície é plana e nunca chega aqui pela
        // `footing_verdict`. Sem "morro acima" definido, nada a remover.
        return motor;
    };
    let kill = |v: Vec2| {
        if v[0] * uphill[0] + v[1] * uphill[1] > 0.0 {
            [0.0, 0.0]
        } else {
            v
        }
    };
    Motor {
        accel: kill(motor.accel),
        boost: kill(motor.boost),
    }
}

/// Estamos no chão? A pergunta pública, e a mesma que as leis consomem.
#[must_use]
pub fn is_grounded(cfg: &PlayerConfig, sample: Option<&GroundSample>, up: Vec2) -> bool {
    footing(cfg, sample, up).is_some()
}
