//! **PULAR** (W4) — a altura é AUTORADA, e a gravidade tem fases.
//!
//! # ⚠️ Parametrizado por ALTURA, nunca por força
//!
//! É a lição que o `bevy-tnua` documenta e a razão de o campo se chamar
//! `jump_height`: uma força (ou um impulso) dá **alturas diferentes** conforme a
//! velocidade vertical no instante do toque e conforme onde na mola o
//! personagem estava. O artista não pensa em newtons — ele pensa *"este pulo
//! alcança aquela plataforma"*, e o número que ele digita tem de ser essa
//! frase. A conversão é `v₀ = √(2·|g|·h)`, feita **uma vez**, no salto.
//!
//! ⚠️ **E a altura é a de gravidade NEUTRA.** Os multiplicadores abaixo existem
//! precisamente para dobrá-la, então prometer que ela é invariante seria mentir:
//! `peak_gravity < 1` sobe MAIS (é o que alongar significa) e `cut_gravity`
//! corta. O gate da altura autorada roda com os multiplicadores em `1.0`, que é
//! o controle, e os outros medem quanto cada um a move.
//!
//! # ⚠️ O ÁPICE tem duas escolas, e elas são OPOSTAS
//!
//! O Celeste **ALONGA** o topo (gravidade reduzida — é *forgiveness*: dá ao
//! jogador a janela para mirar) e o `bevy-tnua` **ENCURTA** (`peak_prevention`,
//! é *snappiness*). **Decisão: alongar** — o pedido é um platformer PRECISO, e a
//! precisão vive na janela em que o jogador ainda pode decidir. Quem quiser o
//! oposto põe `peak_gravity > 1`: é o mesmo knob, do outro lado do 1, em vez de
//! um segundo controle dizendo o contrário do primeiro.
//!
//! # ⚠️ A mola CALA enquanto o pulo sobe
//!
//! É o defeito que este módulo existiria para ter: no instante da decolagem o
//! raio **ainda vê o chão** (o personagem não saiu do `cling_distance`), então
//! uma mola viva puxaria de volta o que o boost acabou de dar. Enquanto
//! [`JumpState::airborne`], a perna se cala; ela volta quando o personagem está
//! DESCENDO e o sensor acha chão — que é também o que faz bater a cabeça num
//! teto pousar corretamente.
//!
//! # ⚠️ A gravidade extra é `(escala − 1)·g`, e o `1.0` é BYTE-IDÊNTICO
//!
//! O solver já aplica a gravidade; esta lei só acrescenta a diferença. Com todos
//! os multiplicadores em `1.0` o termo é exatamente zero e a queda é a do mundo
//! — o que torna cada knob uma escolha, e não um imposto.

use crate::{GroundSample, Motor, Vec2};

/// **Os knobs** — módulo irmão, re-exportado (ver o doc dele).
#[path = "jump_config.rs"]
mod jump_config;
pub use jump_config::JumpConfig;

/// O estado VIVO de um pulo — o que o tick anterior deixou.
///
/// ⚠️ **Mora na PONTE, nunca no componente**: um campo que muda por tick dentro
/// de um componente faria o `canonicalize` do undo ver cada frame como um passo
/// (a mesma lei que manteve velocidade e sono fora do `RigidBody`). E a partir
/// da W7 ele deixa de ser guardado e passa a ser **derivado da fita de entrada**,
/// que é o que o torna reproduzível num scrub.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct JumpState {
    /// Em voo por causa de um pulo AUTORADO — não por ter andado para fora de
    /// uma borda.
    ///
    /// É ele que **cala a mola** (ver o aviso do módulo) e é ele que impede um
    /// segundo pulo no ar.
    pub airborne: bool,
    /// O botão foi solto desde a decolagem: o corte já está armado.
    ///
    /// ⚠️ Uma vez armado ele NÃO desarma segurando de novo. Soltar é uma
    /// decisão sobre ESTE pulo, e deixar o jogador desfazê-la no ar tornaria a
    /// altura função de quantas vezes ele tamborilou o dedo.
    pub cut: bool,
    /// O botão estava segurado no tick anterior — a BORDA sai daqui.
    ///
    /// Sem ela, segurar a tecla no chão re-pularia a cada tick.
    pub was_held: bool,

    /// **Segundos de coyote restantes** (W8) — enche com o pé no chão, escorre
    /// no ar, e a decolagem o ZERA.
    ///
    /// ⚠️ **É um CONTADOR, e é ele que torna o push incondicional do estado
    /// load-bearing** (ver o aviso em `bridge/player.rs`): todo campo acima é
    /// função pura da entrada deste tick, então perdê-los num tick sem motor não
    /// custava nada. Um contador congelado é uma tolerância que deixa de
    /// existir, **sem sintoma** — o pulo continua saindo.
    pub coyote: f32,

    /// **Segundos de aperto guardado** (W8) — enche na BORDA do botão, escorre
    /// sempre, e a decolagem o ZERA.
    ///
    /// ⚠️ Zerar na decolagem não é higiene: sem isso um aperto só dispararia o
    /// pulo e, no tique seguinte ainda em contato, dispararia de novo.
    pub buffer: f32,

    /// **Quantos pulos do ar ainda restam** (`W-MultiJump`) — enche no CHÃO com
    /// [`JumpConfig::air_jumps`], e cada pulo do ar gasta um.
    ///
    /// ⚠️ **É um CONTADOR, como o coyote**, então vale para ele o mesmo aviso:
    /// o push incondicional do estado é load-bearing, porque um contador
    /// congelado é uma carga que deixa de recarregar **sem sintoma nenhum** até
    /// o jogador tentar o segundo pulo.
    pub air_jumps_left: u32,

    /// **A velocidade do chão que se deixou** (W10) — o referencial que o
    /// controle aéreo continua usando por [`JumpConfig::lift_momentum`].
    ///
    /// ⚠️ **NÃO é consumida pela decolagem**, ao contrário dos dois relógios
    /// acima, e a assimetria é o ponto: coyote e buffer perdoam um erro do
    /// jogador e gastar-se é o que os impede de virar segunda chance; esta é um
    /// **fato sobre o mundo** (*"eu vinha de um vagão a 5 m/s"*) e continua
    /// verdadeiro depois do pulo — aliás é no pulo que ela mais importa.
    pub lift: Vec2,

    /// **Segundos de memória restantes** (W10) — enche com o pé no chão, escorre
    /// no ar, e o quociente por `lift_momentum` é o desvanecimento.
    pub lift_time: f32,

    /// **Segundos de silêncio do controle aéreo** (W13) — enche num pulo de
    /// PAREDE, escorre no ar, e o pé no chão o ZERA.
    ///
    /// ⚠️ **Zerar ao pousar não é higiene:** um jogador que acabou de aterrar
    /// espera dirigir, e uma janela sobrevivente lhe tiraria o controle no chão
    /// por um motivo que aconteceu no ar. É a mesma razão pela qual o coyote
    /// enche no chão em vez de acumular.
    pub wall_lock: f32,

    /// **O FLUIDO me tomou, e ainda não pousei em nada sólido** — o interruptor
    /// que cala a modelagem do arco (`W-Submerged`).
    ///
    /// # ⚠️ Por que é uma TRAVA e não a fração instantânea
    ///
    /// A modelagem do arco é **não-conservativa por construção**: subir com `g`
    /// e descer com `2·g` devolve o corpo ao mesmo nível com **`√2` da
    /// velocidade** — o dobro da energia, por ciclo. Num platformer isso é
    /// inofensivo porque todo arco termina absorvido pelo CHÃO; sobre uma
    /// superfície que RESTAURA (uma poça, e um trampolim no dia em que houver
    /// um) a ficção passa a acumular, e o personagem sai de quadro.
    ///
    /// ⚠️ **E a metade que uma fração instantânea NÃO alcança é o AR:** medido,
    /// desvanecer só enquanto submerso cura o repouso (amplitude 0,0046 contra
    /// 0,0050 do controle) e deixa o corpo largado de 1,5 m divergindo a **11 m**
    /// — porque a energia é ganha entre dois mergulhos, onde não há fluido
    /// nenhum a medir. A ablação nomeia o culpado sem ambiguidade: com
    /// `fall_gravity = 1` a amplitude cai de **14,55 para 0,11** e `peak`/
    /// `takeoff` não movem um centímetro.
    ///
    /// **A lei, numa frase:** *a modelagem do arco vale entre dois contatos com
    /// o CHÃO; um corpo que o fluido tomou saiu desse arco e só volta a ele
    /// quando pousa em algo sólido.*
    ///
    /// ⚠️ **Consequência nomeada:** raspar a água ao atravessar uma poça de um
    /// salto deixa o resto daquela queda com a gravidade do MUNDO — mais leve do
    /// que o autorado. É o preço de não ter um limiar mágico entre *"encostei"*
    /// e *"estou dentro"*, e ele erra para o lado da física honesta.
    pub waterborne: bool,
}

impl JumpState {
    /// **O pé está no chão?** — a porta única da pergunta.
    ///
    /// ⚠️ Ela existe porque tem **dois** consumidores: esta lei (para encher o
    /// coyote e a memória do chão) e o ARRANQUE (para repor a carga, W14). Duas
    /// cópias do predicado dariam um tique em que o pulo recarrega e o arranque
    /// não — e o sintoma seria *"às vezes só posso arrancar uma vez"*, que
    /// ninguém liga a um `&&` escrito duas vezes.
    ///
    /// ⚠️ **As duas metades importam**: o sensor achar chão não basta, porque no
    /// tique da decolagem o raio ainda o vê (o personagem não saiu do
    /// `cling_distance`) — é o [`Self::airborne`] que separa *"estou de pé"* de
    /// *"acabei de sair daqui"*.
    #[must_use]
    pub fn on_ground(&self, footing: Option<&GroundSample>) -> bool {
        footing.is_some() && !self.airborne
    }
}

/// **QUAL dos três pulos saiu** (`W-PlayerOut`).
///
/// ⚠️ **A LEI é a única que sabe, e é por isso que ela o diz.** A ponte teria de
/// adivinhar por *"ele estava no chão no tique anterior?"* — e a adivinhação
/// erra exactamente no caso interessante: um pulo de PAREDE acontece com o pé no
/// ar, tal como um pulo do ar, então o proxy os funde. Publicar o veredito é uma
/// linha; re-derivá-lo lá fora seria uma segunda resposta que o consumidor de
/// som e de animação lê como *"o pulo de parede não existe"*.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum JumpKind {
    /// O pé empurrou o CHÃO — o único que a 3.ª lei devolve à jangada.
    Ground,
    /// Um pulo do AR (`W-MultiJump`).
    Air,
    /// Um pulo de PAREDE (W13).
    Wall,
}

/// O que a lei do pulo decidiu neste tick.
pub struct JumpStep {
    /// O termo do motor: o impulso da decolagem (boost) e a gravidade extra
    /// (accel).
    pub motor: Motor,
    /// O estado a guardar para o próximo tick.
    pub state: JumpState,
    /// **A perna pode agir?** Falso enquanto o pulo sobe — ver o aviso do
    /// módulo.
    pub spring_armed: bool,
    /// **QUAL pulo saiu neste tique**, ou `None` se não saiu nenhum.
    ///
    /// ⚠️ **Era DOIS `bool`, e a troca não é cosmética.** O `takeoff` (*o pé
    /// empurrou o chão*) e o `jumped` (*saiu um pulo, de qualquer dos três*)
    /// eram escritos à mão em **seis** sítios de construção — e um quarto tipo
    /// de pulo teria de acertar os dois em cada um deles, sem que nada o
    /// obrigasse. Sendo eles VISTAS ([`Self::takeoff`], [`Self::jumped`]), um
    /// sítio novo só consegue escrever **um** fato, e os dois vereditos deixam
    /// de poder discordar.
    ///
    /// ⚠️ E é este campo que a saída do player (`W-PlayerOut`) publica: sem o
    /// tipo, um pulo de PAREDE e um pulo do AR são indistinguíveis para quem
    /// toca o som.
    pub kind: Option<JumpKind>,
    /// **O aperto virou uma DESCIDA** (W12) — ver [`crate::PlayerStep::drop_through`].
    ///
    /// ⚠️ Verdadeiro **em vez de** [`Self::takeoff`], nunca junto: os dois são o
    /// que o mesmo botão fez neste tique, e um tique em que ambos fossem
    /// verdadeiros seria um personagem que pula e atravessa o chão ao mesmo
    /// tempo.
    pub drop_through: bool,
}

impl JumpStep {
    /// ⚠️ **Este motor é um EMPURRÃO no chão** — verdadeiro só no tick da
    /// decolagem.
    ///
    /// A reação da 3ª lei ([`crate::react`]) precisa distinguir o empurrão da
    /// decolagem (contato: pular de uma jangada a afunda) da **gravidade de
    /// FASE** do arco (ficção aplicada ao personagem no ar, que devolvida ao
    /// chão seria ele empurrar uma plataforma que não está tocando).
    ///
    /// ⚠️ Hoje os dois são distinguíveis pelo acidente de um usar `boost` e o
    /// outro `accel` — e é exatamente por ser acidente, e não contrato, que esta
    /// pergunta existe.
    #[must_use]
    pub fn takeoff(&self) -> bool {
        matches!(self.kind, Some(JumpKind::Ground))
    }

    /// **Um pulo saiu neste tique — de QUALQUER dos três** (chão, parede, ar).
    ///
    /// ⚠️ **Ele existe porque o proxy que o ARRANQUE usava apodreceu com o
    /// terceiro pulo:** o `lib.rs` perguntava *"a TRANSIÇÃO para o ar"*
    /// (`!antes.airborne && depois.airborne`), o que era exato enquanto todo
    /// pulo começava no chão ou na parede — e um pulo do AR acontece com
    /// `airborne` **já verdadeiro**, então a transição não existe e o arranque
    /// não seria cancelado, contra o que o próprio comentário de lá promete
    /// (*"um pulo de QUALQUER tipo cancela o arranque"*).
    ///
    /// ⚠️ **Não é o [`Self::takeoff`]**, e a distinção é FÍSICA: aquele diz *o
    /// pé empurrou o CHÃO* (a 3ª lei o devolve à jangada) e este diz *um pulo
    /// saiu*. Com `air_jumps = 0` os dois concordam com o proxy antigo em todo
    /// tique, o que é o que torna aquela wave byte-idêntica onde está desligada.
    #[must_use]
    pub fn jumped(&self) -> bool {
        self.kind.is_some()
    }
}

/// **PULAR.** Ver a lei no topo do módulo.
///
/// - `footing`: o chão que a lei aceita ([`crate::footing`]), `None` no ar.
/// - `rel_up`: a velocidade de subida **relativa ao chão** — é ela que faz
///   pular de um elevador que sobe levar a velocidade dele junto.
/// - `held`: o botão de pulo está pressionado AGORA.
/// - `down`: o botão de BAIXO está pressionado agora — é ele que decide se o
///   aperto vira pulo ou **descida** (W12).
/// - `wall`: o que a parede OFERECE a este aperto (W13), `None` quando não há
///   parede agarrada. Ver [`crate::wall_launch`].
#[must_use]
// ⚠️ **Dez argumentos, e o último é o `dt`** (W8/W12/W13) — os dois relógios do perdão
// escorrem em segundos, nunca em contagem de tiques. Agrupar `(footing, rel_up,
// held)` num "o que o mundo diz" seria inventar um tipo com um chamador só; o
// precedente é o `body_desc` do W-LockRot, que levou o mesmo `allow`.
#[allow(clippy::too_many_arguments)]
pub fn jump_step(
    cfg: &JumpConfig,
    state: JumpState,
    footing: Option<&GroundSample>,
    rel_up: f32,
    held: bool,
    down: bool,
    wall: Option<crate::WallLaunch>,
    gravity: Vec2,
    up: Vec2,
    dt: f32,
    buoyed: crate::Buoyed,
) -> JumpStep {
    let pressed = held && !state.was_held;
    let mut next = JumpState {
        was_held: held,
        ..state
    };

    // ── OS DOIS RELÓGIOS DO PERDÃO (W8) ──────────────────────────────────────
    // ⚠️ Eles correm ANTES da decolagem, e a ordem é a lei: um aperto feito
    // NESTE tique tem de poder disparar NESTE tique. Enchendo depois, um aperto
    // com o pé no chão só sairia no tique seguinte — a assistência atrasaria o
    // que ela existe para adiantar.
    //
    // ⚠️ E os dois escorrem por `dt` de RELÓGIO, nunca por contagem de tiques:
    // uma tolerância medida em quadros muda de tamanho quando o `fixed_dt` muda,
    // e "0,1 s" é uma frase sobre o dedo do jogador, não sobre a taxa da sim.
    // ── A TRAVA DO FLUIDO (W-Submerged) ──────────────────────────────────────
    // ⚠️ **A ordem é a lei:** o chão desarma, o fluido arma. Quem está de pé num
    // fundo submerso — vadeando — está GROUNDED e molhado no mesmo tique, e ali
    // a modelagem já é inerte pelo early-out de baixo; o que a ordem decide é o
    // tique em que ele PULA de dentro d'água, e ali o arco é mesmo da água.
    if footing.is_some() {
        next.waterborne = false;
    }
    if buoyed.carries_weight() {
        next.waterborne = true;
    }

    let grounded = state.on_ground(footing);
    if grounded {
        // No chão o coyote está CHEIO — ele é o que sobra depois de sair, não um
        // tempo acumulado por ficar parado.
        next.coyote = cfg.coyote_time.max(0.0);
        // ⚠️ **E as cargas do ar enchem AQUI, na MESMA resposta** — o terceiro
        // consumidor do `on_ground` (os outros dois são o coyote acima e o
        // ARRANQUE, no `lib.rs`), sem uma 2ª cópia do predicado.
        next.air_jumps_left = cfg.air_jumps;
    } else {
        next.coyote = (state.coyote - dt).max(0.0);
    }

    // ── A MEMÓRIA DO CHÃO QUE SE DEIXOU (W10) ────────────────────────────────
    // ⚠️ Ela é RE-CARIMBADA a cada tique no chão, e não só na saída: a saída não
    // é um evento que esta lei observa (o sensor simplesmente para de achar
    // chão), e guardar "a última velocidade vista" é a forma de saber o que ele
    // valia no instante em que ainda havia chão.
    if grounded {
        next.lift = footing.map_or([0.0, 0.0], |s| s.ground_velocity);
        next.lift_time = cfg.lift_momentum.max(0.0);
    } else {
        next.lift_time = (state.lift_time - dt).max(0.0);
    }
    next.buffer = if pressed {
        cfg.jump_buffer.max(0.0)
    } else {
        (state.buffer - dt).max(0.0)
    };
    // ⚠️ O silêncio do controle aéreo (W13) escorre como os outros dois, e o pé
    // no chão o ZERA — ver [`JumpState::wall_lock`].
    next.wall_lock = if grounded {
        0.0
    } else {
        (state.wall_lock - dt).max(0.0)
    };

    // ── POUSO ────────────────────────────────────────────────────────────────
    // ⚠️ **ANTES da decolagem, e a ordem é o JUMP BUFFER** (W8): um aperto
    // guardado tem de disparar NO tique em que o pé toca. Com a decolagem
    // primeiro, ela lê o `airborne` que ainda não caiu, recusa, e o pulo sai um
    // tique depois — 16 ms de atraso exatamente no gesto que o buffer existe
    // para adiantar. Ordenado assim, o mesmo tique pousa e decola.
    //
    // ⚠️ E isto **não** faz o pouso disparar no tique da decolagem: ali
    // `state.airborne` é falso na entrada, e é ele (não o `next`) que este ramo
    // pergunta.
    // O pulo acaba quando ele está DESCENDO e o sensor acha chão. As duas
    // metades importam: só "achou chão" pousaria no tick da decolagem (o raio
    // ainda vê o chão), e só "descendo" nunca pousaria.
    if state.airborne && rel_up <= 0.0 && footing.is_some() {
        next.airborne = false;
        next.cut = false;
    }

    // ── DECOLAGEM ────────────────────────────────────────────────────────────
    // ⚠️ **As duas metades são agora RELÓGIOS, e a borda vive DENTRO deles.**
    // "Estou no chão" virou *"estou no chão ou saí dele há pouco"* (coyote) e
    // "apertei agora" virou *"apertei agora ou há pouco"* (buffer) — e o `>= 0`
    // do buffer é o que mantém `jump_buffer = 0` sendo o comportamento de antes
    // desta wave AO BIT: com a janela em zero, `next.buffer > 0.0` é falso e só
    // o aperto DESTE tique passa.
    let can_reach_ground = footing.is_some() || next.coyote > 0.0;
    let wants_to_jump = pressed || next.buffer > 0.0;

    // ── DESCER ATRAVÉS DA PLATAFORMA (W12) ───────────────────────────────────
    // ⚠️ **ANTES da decolagem, e a ordem É o gesto:** com o baixo segurado em
    // cima de uma plataforma jump-through, o MESMO aperto que pularia passa a
    // descer. Depois da decolagem, o pulo já teria saído e a descida chegaria a
    // um personagem no ar.
    //
    // ⚠️ **Pergunta ao `footing` VIVO, nunca ao coyote:** descer exige estar EM
    // CIMA da coisa que se atravessa. O coyote perdoa um erro de TEMPO ao sair
    // de uma borda; herdá-lo aqui deixaria o personagem cair através de uma
    // plataforma de que ele já saiu — perdão para um erro que ninguém cometeu.
    if !next.airborne && wants_to_jump && down && footing.is_some_and(|s| s.one_way) {
        // ⚠️ O aperto é CONSUMIDO, pela mesma razão da decolagem: um buffer que
        // sobrevive à própria descida re-dispara no tique seguinte — e ali o
        // personagem já está DENTRO da plataforma, onde o mesmo aperto viraria
        // um pulo que o empurra de volta para cima.
        next.buffer = 0.0;
        return JumpStep {
            motor: Motor::default(),
            state: next,
            // ⚠️ **A perna cala NESTE tique, e é o que faz a descida começar
            // agora:** o raio ainda vê a plataforma (a ponte só a exclui do
            // sensor a partir do próximo), então uma mola viva seguraria o
            // personagem em cima do que ele acabou de pedir para atravessar.
            spring_armed: false,
            kind: None,
            drop_through: true,
        };
    }

    if can_reach_ground && !next.airborne && wants_to_jump {
        let g = (gravity[0] * gravity[0] + gravity[1] * gravity[1]).sqrt();
        let v0 = (2.0 * g * cfg.jump_height.max(0.0)).sqrt();
        next.airborne = true;
        next.cut = false;
        // ⚠️ **Os dois são CONSUMIDOS**, e por razões diferentes: o coyote
        // porque perdoar um erro de tempo não é dar uma segunda chance (sem
        // isto, sair da borda e pular deixaria a janela viva no ar), e o buffer
        // porque um aperto guardado que sobrevive à própria decolagem re-dispara
        // no tique seguinte, enquanto o pé ainda toca.
        next.coyote = 0.0;
        next.buffer = 0.0;
        // ⚠️ O boost leva a velocidade AO valor, não SOMA a ele: pular já
        // subindo (numa plataforma que sobe, ou logo depois de um degrau) tem de
        // dar a mesma altura que pular parado — que é a frase inteira do
        // "parametrizado por altura".
        let delta = v0 - rel_up;
        return JumpStep {
            motor: Motor {
                accel: [0.0, 0.0],
                boost: [up[0] * delta, up[1] * delta],
            },
            state: next,
            spring_armed: false,
            kind: Some(JumpKind::Ground),
            drop_through: false,
        };
    }

    // ── O PULO DE PAREDE (W13) ───────────────────────────────────────────────
    // ⚠️ **DEPOIS da decolagem do chão, e a precedência é deliberada:** com o pé
    // no chão o aperto é um pulo normal, mesmo encostado a uma parede. O chão é
    // o apoio mais forte, e disputar isso daria um pulo cuja altura depende de
    // haver ou não uma parede por perto.
    //
    // ⚠️ **E ele NÃO pergunta pelo `airborne`**, ao contrário de tudo o que está
    // acima: a razão de existir de um pulo de parede é sair de um voo que já
    // está a acontecer. Pedir *"não estar no ar"* aqui tornaria a feature
    // inalcançável — e é exactamente o guard que faria o gate ficar verde
    // enquanto o produto não pula parede nenhuma.
    if let Some(w) = wall
        && wants_to_jump
    {
        let g = (gravity[0] * gravity[0] + gravity[1] * gravity[1]).sqrt();
        let v0 = (2.0 * g * w.height.max(0.0)).sqrt();
        next.airborne = true;
        next.cut = false;
        // Os dois relógios são consumidos pela MESMA razão da decolagem do chão:
        // um aperto guardado que sobrevive ao próprio pulo re-dispara no tique
        // seguinte, e o personagem ainda está encostado.
        next.coyote = 0.0;
        next.buffer = 0.0;
        // ⚠️ **E o controle aéreo fica calado**, senão o jogador — que ainda
        // segura a direção da parede — é puxado de volta para ela e o resto do
        // voo é gasto a raspar. Medido: 0,44 m de afastamento e 76% da altura
        // autorada. Ver [`crate::WallConfig::jump_lockout`].
        next.wall_lock = w.lockout.max(0.0);
        // ⚠️ **A vertical vai AO valor e a horizontal é SOMADA**, e a assimetria
        // é um fato sobre a cena, não descuido: agarrado, a velocidade vertical
        // é a do escorregamento (que este pulo tem de apagar) e a horizontal é
        // ~zero, porque a parede a está a segurar. Somar ali é o mesmo que
        // atribuir, e não exige a esta lei um argumento novo com a velocidade
        // horizontal — que ela não tem e para mais nada precisaria.
        let rise = v0 - rel_up;
        return JumpStep {
            motor: Motor {
                accel: [0.0, 0.0],
                boost: [up[0] * rise + w.away[0], up[1] * rise + w.away[1]],
            },
            state: next,
            spring_armed: false,
            // ⚠️ **Um pulo de PAREDE não é `takeoff`, e é FÍSICA:** a 3ª lei
            // devolve ao CHÃO o que o pé nele empurrou, e este pé não empurrou
            // chão nenhum — ele empurrou uma parede. Dizer `Ground` aqui faria o
            // personagem afundar uma jangada com um pulo dado numa parede do
            // outro lado da cena.
            kind: Some(JumpKind::Wall),
            drop_through: false,
        };
    }

    // ── O PULO DO AR (`W-MultiJump`) ─────────────────────────────────────────
    // ⚠️ **POR ÚLTIMO dos três, e a ordem é a força do APOIO:** o chão é o apoio
    // mais forte, a parede é um apoio real, e o ar não é apoio nenhum. Gastar
    // uma carga encostado a uma parede cobraria ao jogador um recurso por um
    // pulo que ele nem pediu.
    //
    // ⚠️ **Sem guard de *"não estou no chão"*, e a ausência é deliberada:** os
    // dois ramos acima já RETORNARAM em todo caso em que há chão ou parede sob o
    // aperto, então chegar aqui **é** estar no ar. Um `!grounded` seria a
    // segunda cópia de uma condição que já foi decidida — e a cópia que
    // envelhece quando um quarto apoio aparecer.
    if next.air_jumps_left > 0 && wants_to_jump {
        let g = (gravity[0] * gravity[0] + gravity[1] * gravity[1]).sqrt();
        let v0 = (2.0 * g * cfg.air_jump_height.max(0.0)).sqrt();
        next.air_jumps_left -= 1;
        next.airborne = true;
        next.cut = false;
        // ⚠️ **O buffer É consumido, e sem isto a wave se autodestrói:** um
        // aperto guardado sobrevive `jump_buffer` segundos, e um contador que
        // ele possa disparar de novo no tique seguinte **queima todas as cargas
        // num aperto só** — com `jump_buffer = 0,1` e 3 cargas, as três somem em
        // ~6 tiques. O coyote não é tocado: ele já é zero aqui por construção
        // (a decolagem do chão o consome, e ele só enche com o pé no chão).
        next.buffer = 0.0;
        // ⚠️ **O boost leva a velocidade AO valor, exatamente como o do chão** —
        // é o que faz um pulo do ar dado em plena QUEDA alcançar a altura
        // autorada em vez de ser comido pela velocidade que já havia. É também o
        // que o jogador espera do gesto: a segunda chance apaga o erro.
        let delta = v0 - rel_up;
        return JumpStep {
            motor: Motor {
                accel: [0.0, 0.0],
                boost: [up[0] * delta, up[1] * delta],
            },
            state: next,
            spring_armed: false,
            // ⚠️ **Não é `Ground`, pela MESMA física do pulo de parede** — a 3ª
            // lei devolve ao chão o que o pé nele empurrou, e este pé não
            // empurrou coisa alguma. Dizê-lo afundaria uma jangada com um pulo
            // dado no ar acima dela.
            kind: Some(JumpKind::Air),
            drop_through: false,
        };
    }

    // ── O CORTE ──────────────────────────────────────────────────────────────
    // Soltar durante a subida arma o corte, e ele não desarma (ver `JumpState`).
    if next.airborne && !held {
        next.cut = true;
    }

    let spring_armed = !next.airborne;

    // ── A GRAVIDADE POR FASE ─────────────────────────────────────────────────
    // No chão a perna manda, e uma gravidade extra ali brigaria com ela.
    if footing.is_some() && spring_armed {
        return JumpStep {
            motor: Motor::default(),
            state: next,
            spring_armed,
            kind: None,
            drop_through: false,
        };
    }

    let scale = if rel_up > cfg.peak_speed {
        // SUBINDO.
        if next.cut {
            cfg.cut_gravity
        } else if rel_up > cfg.takeoff_speed {
            cfg.takeoff_gravity
        } else {
            1.0
        }
    } else if rel_up < -cfg.peak_speed {
        cfg.fall_gravity
    } else {
        // O ÁPICE — e também o instante em que se anda para fora de uma borda,
        // que é o mesmo `|v| ≈ 0`. Ganhar ali a mesma flutuação é o *coyote
        // feel* do Celeste, e é de propósito.
        cfg.peak_gravity
    };

    // ⚠️ `(escala − 1)·g`, então `1.0` é EXATAMENTE zero — a queda do mundo,
    // byte a byte.
    //
    // ⚠️ **E a modelagem CALA enquanto o fluido o tiver** (ver
    // [`JumpState::waterborne`]): ela descreve um arco entre dois contatos com o
    // chão, e um corpo que a água tomou não está num. Uma cena sem poça nunca
    // arma a trava, então o termo é byte-idêntico ao que shipava.
    let extra = if next.waterborne { 0.0 } else { scale - 1.0 };
    JumpStep {
        motor: Motor {
            accel: [gravity[0] * extra, gravity[1] * extra],
            boost: [0.0, 0.0],
        },
        state: next,
        spring_armed,
        kind: None,
        drop_through: false,
    }
}

/// **O referencial em que o controle aéreo mede a velocidade** (W10) — a memória
/// do chão que se deixou, já desvanecida.
///
/// Devolve `[0, 0]` quando a janela está fechada, quando ela é zero (a
/// assistência desligada) ou quando o chão de onde se saiu estava parado — e é
/// esse último caso que torna o default ligado **inerte** em toda cena de chão
/// estático, byte a byte.
///
/// ⚠️ **Lê o estado DEPOIS do tique**, o que [`jump_step`] devolveu: a memória
/// escorre e é usada no mesmo tique, exatamente como o coyote enche e é
/// consultado no mesmo. Ler o estado de ENTRADA daria ao ar um tique de memória
/// a mais do que a janela autorada — pequeno, invisível, e errado.
#[must_use]
pub fn carried_frame(cfg: &JumpConfig, state: &JumpState) -> Vec2 {
    if !cfg.lift_momentum.is_finite() || cfg.lift_momentum <= 0.0 || state.lift_time <= 0.0 {
        return [0.0, 0.0];
    }
    state.lift
}

/// Os gates desta lei — irmão por `#[path]` (o `mod tests` continua FILHO, então
/// `use super::*` alcança os privados) pelo cap de 700 LOC.
#[cfg(test)]
#[path = "jump_tests.rs"]
mod tests;

/// Os gates do PULO DO AR — irmão por ASSUNTO (`W-MultiJump`), ver o doc dele.
#[cfg(test)]
#[path = "jump_air_tests.rs"]
mod air_tests;
