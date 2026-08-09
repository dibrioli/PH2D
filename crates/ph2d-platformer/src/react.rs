//! **A REAÇÃO** (W6) — a 3ª lei: o que o personagem faz ao chão que o segura.
//!
//! É o que separa este controlador do `bevy-tnua`. Uma cápsula flutuante que só
//! empurra a si mesma é um **fantasma**: ela paira sobre uma jangada sem a
//! afundar, atravessa uma gangorra sem a inclinar e uma balança não a pesa. A
//! perna empurra o chão para baixo com exatamente a força com que o chão a
//! segura, e devolver esse número ao outro corpo é uma linha de física, não uma
//! feature de jangada.
//!
//! # ⚠️ A reação é sobre CONTATO, e nem tudo no motor é contato
//!
//! O motor do player soma termos de naturezas diferentes, e só alguns são
//! forças que passam pelo pé:
//!
//! - **a mola** (incluindo o cancelamento da gravidade) — é o PESO sendo
//!   carregado. Reagir a ele é o que faz a balança marcar.
//! - **o empurrão da decolagem** — pular de uma jangada a afunda. É contato.
//! - **a caminhada** — é atrito contra o chão. É contato, e tem escalar PRÓPRIO.
//! - ⛔ **a gravidade de FASE do pulo** (o arco de quatro fases) — **não é
//!   contato**. Ela é uma ficção aplicada só ao personagem, no ar, para o arco
//!   ter a forma que o artista autorou; devolvê-la ao chão seria o personagem
//!   empurrar uma plataforma **que ele não está tocando**.
//!
//! É por isso que [`crate::JumpStep`] carrega o `takeoff`: sem ele, distinguir
//! *"o boost da decolagem"* de *"a gravidade de fase"* dependeria do acidente de
//! um deles usar `accel` e o outro `boost`, que é verdade hoje e não é contrato.
//!
//! # ⚠️ UMA ESCRITA DE VELOCIDADE POR TICK NÃO É UMA FORÇA, e não volta
//!
//! **Medido, e é a decisão mais importante desta wave.** O amortecedor da perna
//! não é `accel`: ele é um `boost`, uma escrita DIRETA de velocidade calculada a
//! partir da velocidade RELATIVA ao chão. Devolvê-lo fecha um laço no nível da
//! VELOCIDADE — o chão muda de velocidade, a relativa do tick seguinte muda, o
//! amortecedor responde, e o solver nunca media a troca porque nenhum dos dois
//! lados é uma força que ele resolva.
//!
//! Numa jangada pendurada por molas o preço foi visível na primeira medição: ela
//! disparava para **−0,946 m**, ultrapassava o personagem (folga 2,48 m contra o
//! alcance de 1,4 da perna), ele perdia o chão, caía até o piso, e ela ficava a
//! **DERIVAR para cima** sem ninguém em cima. Excluindo o boost do amortecedor
//! ela assenta em **−0,194** e fica lá, com o personagem a bordo e inclinada 2°.
//!
//! ⚠️ **O boost da DECOLAGEM continua voltando**, e a distinção é o que torna a
//! regra honesta: um pulo é **um** tick, então não há laço a fechar — é um
//! empurrão, e pular de uma jangada tem de a afundar. O que é excluído é o que
//! se repete *todo* tick.
//!
//! É por isso que [`reaction`] recebe os canais SEPARADOS em vez de um motor já
//! somado: quem chama é quem sabe o que é força contínua e o que é impulso.
//!
//! # ⚠️ Dois escalares com defaults OPOSTOS, e o motivo é de PRODUTO
//!
//! O `bevy-tnua` já pagou esta: os dois números existem separados porque as
//! respostas certas são opostas.
//!
//! - [`ReactionConfig::support`] nasce em **1,0** — o peso é transmitido
//!   inteiro. É a física, e desligá-la é que seria a escolha estranha.
//! - [`ReactionConfig::movement`] nasce em **0,0** — senão a plataforma
//!   **escorrega para trás como um tapete** quando o personagem anda em cima
//!   dela. É atrito honesto e é péssimo de jogar; quem quiser o tapete sobe o
//!   número.
//!
//! # ⚠️ E o terceiro canal é LATERAL, não vertical (W-KinPush)
//!
//! Os dois escalares acima descrevem o que passa pelo **pé**. Um personagem
//! também empurra o que está **ao lado** dele, e sob Snap isso não acontece
//! sozinho: um corpo cinemático tem massa infinita para o solver, então o
//! `move_shape` desliza contra um caixote solto sem lhe transmitir nada —
//! medido, o dinâmico empurra o caixote **16,55 m em 3 s** e o cinemático
//! **0,0000**. [`ReactionConfig::push`] é esse canal, e a lei dele é a
//! [`push_transfer`].

use crate::{Motor, Vec2};

/// Quanto da força do personagem volta para o chão.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ReactionConfig {
    /// O que a perna e a decolagem devolvem — o PESO. Ver o aviso do módulo.
    pub support: f32,
    /// O que a caminhada devolve — o TAPETE. Ver o aviso do módulo.
    pub movement: f32,
    /// **O que um deslocamento BLOQUEADO devolve** — o EMPURRÃO (W-KinPush).
    ///
    /// ⚠️ Nasce em **1,0**, com o `support` e ao contrário do `movement`, e a
    /// razão é a mesma: empurrar um caixote em que se esbarra é o que um corpo
    /// faz. Sob Spring o solver já o faz sozinho e este número é inerte; sob
    /// Snap ele é a única coisa que o faz acontecer.
    pub push: f32,
}

impl ReactionConfig {
    /// ⚠️ **Defaults opostos de propósito** — ver o aviso do módulo.
    pub const STARTING_POINT: Self = Self {
        support: 1.0,
        movement: 0.0,
        push: 1.0,
    };

    /// A reação DESLIGADA — o personagem volta a ser um fantasma.
    ///
    /// Existe para o gate de regressão e para o lado de controle da cena de
    /// smoke: um olho só julga *"a jangada afunda"* contra uma que não afunda.
    pub const OFF: Self = Self {
        support: 0.0,
        movement: 0.0,
        push: 0.0,
    };
}

impl Default for ReactionConfig {
    fn default() -> Self {
        Self::STARTING_POINT
    }
}

/// **O que devolver ao chão**, nas MESMAS unidades do [`Motor`] do player.
///
/// ⚠️ A conversão para força é da ponte, e ela multiplica pela massa **do
/// PLAYER** — é o `m·a` do lado dele. O que o chão faz com isso depende da massa
/// DELE, e quem responde isso é o solver: uma jangada leve afunda, um continente
/// não se mexe, e nenhum dos dois precisa de código.
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Reaction {
    /// A aceleração a devolver (m/s²), a ser multiplicada pela massa do player.
    pub accel: Vec2,
    /// O boost a devolver (m/s), idem.
    pub boost: Vec2,
}

impl Reaction {
    /// Nada é devolvido.
    #[must_use]
    pub fn is_zero(self) -> bool {
        self.accel == [0.0, 0.0] && self.boost == [0.0, 0.0]
    }
}

/// **A 3ª lei**, com os dois escalares e os canais SEPARADOS.
///
/// - `support` — a FORÇA contínua da perna. ⚠️ Só o `accel` dela volta; o
///   `boost` do amortecedor é ignorado, e o porquê está medido no topo do
///   módulo.
/// - `impulse` — o que é **um só tick** (o empurrão da decolagem). Aqui o
///   `boost` volta, porque um pulo não tem o tick seguinte com que realimentar.
/// - `movement` — a caminhada, com o escalar próprio. ⚠️ Tratada como o
///   `support` pela MESMA razão: uma escrita de velocidade contra o chão, toda
///   vez, é o mesmo laço com outro nome.
///
/// A soma sai **negada** — é o que *reação* quer dizer.
#[must_use]
pub fn reaction(cfg: &ReactionConfig, support: Motor, impulse: Motor, movement: Motor) -> Reaction {
    let s = cfg.support.max(0.0);
    let m = cfg.movement.max(0.0);
    Reaction {
        accel: [
            -(support.accel[0] * s + impulse.accel[0] * s + movement.accel[0] * m),
            -(support.accel[1] * s + impulse.accel[1] * s + movement.accel[1] * m),
        ],
        // ⚠️ **Só o impulso** — ver o aviso do módulo.
        boost: [-(impulse.boost[0] * s), -(impulse.boost[1] * s)],
    }
}

/// **O EMPURRÃO** (W-KinPush) — quanto de um deslocamento BLOQUEADO volta para
/// a superfície que o bloqueou, em METROS de um tique.
///
/// `blocked` é *o que o mundo não deixou acontecer* (o pedido menos o efetivo) e
/// `normal` é a normal da superfície em que se bateu. O resultado é a projeção
/// de `blocked` na **linha** da normal, escalada — e quem o converte em impulso
/// é a ponte, com a massa do PLAYER, exatamente como faz com a [`Reaction`].
///
/// # ⚠️ A projeção é na LINHA, e é isso que torna o sinal da normal irrelevante
///
/// `n · (blocked · n)` é invariante à troca de sinal de `n`, e o resultado
/// aponta sempre para o mesmo lado que a componente bloqueada — ou seja, **para
/// dentro do obstáculo**, que é a direção do empurrão. O rapier usa a mesma
/// forma no `solve_character_collision_impulses`, e não por acaso: qual das
/// duas testemunhas de um shapecast dá a normal *"de fora"* é convenção de
/// biblioteca, e uma lei que dependesse dela seria um sinal invertido à espera
/// de uma atualização de dependência.
///
/// ⚠️ **A normal é normalizada aqui**, então uma normal de comprimento ≠ 1 não
/// escala o resultado — uma segunda escala escondida seria um número que ninguém
/// autora a multiplicar o que o artista autora.
///
/// ⚠️ **Escalar negativo é recusado, não invertido** — a mesma regra da
/// [`reaction`]: um empurrão com o sinal trocado é o personagem a *puxar* para
/// si o caixote em que esbarrou.
#[must_use]
pub fn push_transfer(cfg: &ReactionConfig, blocked: Vec2, normal: Vec2) -> Vec2 {
    let s = cfg.push.max(0.0);
    if s == 0.0 {
        return [0.0, 0.0];
    }
    let len2 = normal[0] * normal[0] + normal[1] * normal[1];
    // ⚠️ A ordem é NaN-safe: `is_finite` primeiro, porque `NaN <= 0.0` é FALSO
    // e sozinha a comparação deixaria um `NaN` passar para a divisão.
    if !len2.is_finite() || len2 <= 0.0 {
        return [0.0, 0.0];
    }
    let k = (blocked[0] * normal[0] + blocked[1] * normal[1]) / len2 * s;
    if !k.is_finite() {
        return [0.0, 0.0];
    }
    [normal[0] * k, normal[1] * k]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(accel: Vec2, boost: Vec2) -> Motor {
        Motor { accel, boost }
    }

    /// **O peso volta INTEIRO, e o tapete não existe** — os dois defaults, que
    /// são opostos.
    #[test]
    fn the_weight_comes_back_whole_and_the_carpet_does_not_move() {
        let cfg = ReactionConfig::STARTING_POINT;
        let r = reaction(
            &cfg,
            m([0.0, 9.81], [0.0, 0.0]),
            Motor::default(),
            m([5.0, 0.0], [0.0, 0.0]),
        );
        assert!(
            (r.accel[1] + 9.81).abs() < 1.0e-5,
            "o peso volta inteiro: {}",
            r.accel[1]
        );
        assert!(
            r.accel[0].abs() < 1.0e-5,
            "e a caminhada NAO empurra a plataforma para tras: {}",
            r.accel[0]
        );
    }

    /// Subir o escalar do movimento **liga o tapete** — o número é vivo.
    #[test]
    fn raising_the_movement_scale_turns_the_carpet_on() {
        let cfg = ReactionConfig {
            movement: 1.0,
            ..ReactionConfig::STARTING_POINT
        };
        let r = reaction(
            &cfg,
            Motor::default(),
            Motor::default(),
            m([5.0, 0.0], [0.0, 0.0]),
        );
        assert!(
            (r.accel[0] + 5.0).abs() < 1.0e-5,
            "a caminhada passa a empurrar para tras: {}",
            r.accel[0]
        );
    }

    /// ⚠️ **`OFF` é zero AO BIT** — é o que torna a reação uma adição honesta:
    /// desligada, o mundo é exatamente o de antes desta wave.
    #[test]
    fn off_gives_back_nothing_at_all() {
        let r = reaction(
            &ReactionConfig::OFF,
            m([1.0, 9.81], [0.3, -0.7]),
            m([0.0, 0.0], [0.0, 4.0]),
            m([5.0, 0.0], [0.2, 0.0]),
        );
        assert_eq!(r, Reaction::default(), "desligada nao devolve nada");
        assert!(r.is_zero());
    }

    /// ⚠️ **O boost do AMORTECEDOR não volta; o da DECOLAGEM volta.**
    ///
    /// As duas metades num gate só porque são a MESMA regra vista dos dois
    /// lados, e a regra saiu de uma medição: um `boost` que se repete todo tick
    /// fecha um laço de velocidade que derruba o personagem da jangada (o aviso
    /// do módulo tem os números), e um que acontece uma vez é um empurrão.
    #[test]
    fn the_dampers_boost_stays_home_but_the_takeoffs_comes_back() {
        let cfg = ReactionConfig::STARTING_POINT;
        let damper = reaction(
            &cfg,
            m([0.0, 0.0], [0.0, -0.4]),
            Motor::default(),
            Motor::default(),
        );
        assert_eq!(
            damper.boost,
            [0.0, 0.0],
            "o boost do amortecedor NAO pode voltar: {:?}",
            damper.boost
        );
        let takeoff = reaction(
            &cfg,
            Motor::default(),
            m([0.0, 0.0], [0.0, 6.26]),
            Motor::default(),
        );
        assert!(
            (takeoff.boost[1] + 6.26).abs() < 1.0e-5,
            "o da decolagem volta negado: {}",
            takeoff.boost[1]
        );
    }

    /// ⚠️ **O empurrão vai para DENTRO do obstáculo, aponte a normal para onde
    /// apontar** — as duas metades num gate só porque são a mesma propriedade:
    /// a projeção é na LINHA da normal, e é isso que tira a convenção de sinal
    /// da biblioteca do caminho da lei.
    #[test]
    fn the_push_goes_into_the_wall_whichever_way_the_normal_points() {
        let cfg = ReactionConfig::STARTING_POINT;
        // O personagem queria andar 0,1 m para a direita e a parede o impediu.
        let blocked = [0.1, 0.0];
        let out = push_transfer(&cfg, blocked, [1.0, 0.0]);
        let inn = push_transfer(&cfg, blocked, [-1.0, 0.0]);
        assert!(
            (out[0] - 0.1).abs() < 1.0e-6 && out[1].abs() < 1.0e-6,
            "para dentro da parede: {out:?}"
        );
        assert_eq!(out, inn, "o sinal da normal nao pode importar: {inn:?}");
    }

    /// **Só a componente que a superfície de facto bloqueou viaja.** Um
    /// deslocamento bloqueado PARALELO à parede não é empurrão nenhum — se ele
    /// transferisse, andar ao longo de um muro empurraria o muro.
    #[test]
    fn only_the_component_along_the_normal_travels() {
        let cfg = ReactionConfig::STARTING_POINT;
        let along = push_transfer(&cfg, [0.0, 0.1], [1.0, 0.0]);
        assert_eq!(along, [0.0, 0.0], "paralelo nao empurra: {along:?}");

        // Num obstáculo a 45° só metade da diagonal atravessa.
        let n = [1.0_f32, 1.0];
        let d = push_transfer(&cfg, [0.1, 0.0], n);
        assert!(
            (d[0] - 0.05).abs() < 1.0e-6 && (d[1] - 0.05).abs() < 1.0e-6,
            "a 45 graus metade atravessa, na direcao da normal: {d:?}"
        );
    }

    /// ⚠️ **Uma normal de comprimento ≠ 1 dá o MESMO empurrão** — sem a
    /// normalização, o comprimento de um vetor que o artista nunca vê
    /// multiplicaria o número que ele autora.
    #[test]
    fn a_non_unit_normal_does_not_scale_the_push() {
        let cfg = ReactionConfig::STARTING_POINT;
        let unit = push_transfer(&cfg, [0.1, 0.0], [1.0, 0.0]);
        let long = push_transfer(&cfg, [0.1, 0.0], [7.0, 0.0]);
        assert!(
            (unit[0] - long[0]).abs() < 1.0e-6,
            "unitaria {unit:?} contra longa {long:?}"
        );
        assert_eq!(
            push_transfer(&cfg, [0.1, 0.0], [0.0, 0.0]),
            [0.0, 0.0],
            "normal degenerada nao empurra"
        );
    }

    /// **`OFF` e escalar negativo não empurram nada** — o zero é o controle da
    /// cena de smoke, e o negativo seria o personagem a PUXAR o caixote.
    #[test]
    fn the_push_is_off_at_zero_and_refused_below_it() {
        assert_eq!(
            push_transfer(&ReactionConfig::OFF, [0.1, 0.0], [1.0, 0.0]),
            [0.0, 0.0]
        );
        let negative = ReactionConfig {
            push: -3.0,
            ..ReactionConfig::STARTING_POINT
        };
        assert_eq!(
            push_transfer(&negative, [0.1, 0.0], [1.0, 0.0]),
            [0.0, 0.0],
            "negativo e' tratado como zero, nunca invertido"
        );
    }

    /// Escalar NEGATIVO é recusado — devolver força com o sinal trocado é o
    /// personagem **puxando** a jangada para cima ao pisar nela.
    #[test]
    fn a_negative_scale_is_refused_not_inverted() {
        let cfg = ReactionConfig {
            support: -3.0,
            movement: -3.0,
            push: -3.0,
        };
        let r = reaction(
            &cfg,
            m([0.0, 9.81], [0.0, 0.0]),
            Motor::default(),
            m([5.0, 0.0], [0.0, 0.0]),
        );
        assert_eq!(r, Reaction::default(), "negativo e' tratado como zero");
    }
}
