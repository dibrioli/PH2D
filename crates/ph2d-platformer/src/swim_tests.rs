//! Os gates da lei do nado (W-Swim) — a trava, o servo e a inércia.

use super::*;
use crate::GroundSample;

const UP: Vec2 = [0.0, 1.0];
const DT: f32 = 1.0 / 60.0;

/// Um nado ligado com números redondos, para as contas serem legíveis.
fn armed() -> SwimConfig {
    SwimConfig {
        speed: 4.0,
        acceleration: 12.0,
        ..SwimConfig::STARTING_POINT
    }
}

fn flat() -> GroundSample {
    GroundSample {
        grip: 1.0,
        distance: 0.5,
        normal: UP,
        ground_velocity: [0.0, 0.0],
        one_way: false,
        brink: crate::Brink::NONE,
    }
}

const DRY: SwimState = SwimState { active: false };
const WET: SwimState = SwimState { active: true };

// ── A TRAVA ──────────────────────────────────────────────────────────────────

/// **Nasce desligado**, e desligado é inerte — o mundo de antes desta wave.
///
/// **Mutação que deve sangrar:** tirar o `armed()` do [`swim_step`].
#[test]
fn the_capability_is_born_off_and_off_is_inert() {
    let off = SwimConfig::STARTING_POINT;
    assert!(!off.armed(), "o ponto de partida tem de nascer DESLIGADO");
    // Fundo, molhado, sem chão: tudo o que arma — menos a capacidade.
    let s = swim_step(&off, DRY, None, Buoyed(4.0));
    assert!(!s.active, "desligado nao nada: {s:?}");
    // E o motor não escreve nada.
    assert_eq!(
        swim_motor(&off, 1.0, 1.0, [0.0, 0.0], UP, DT),
        Motor::default(),
        "desligado nao produz motor"
    );
}

/// **Aceleração zero também é desligado** — um regime que silencia a perna e a
/// caminhada sem orçamento para corrigir é pior do que não existir.
#[test]
fn a_stroke_with_no_budget_counts_as_off() {
    let cfg = SwimConfig {
        acceleration: 0.0,
        ..armed()
    };
    assert!(!cfg.armed());
    assert!(!swim_step(&cfg, DRY, None, Buoyed(4.0)).active);
}

/// ⚠️ **A frase que o `player_motor` cita em vez de re-escrever:** com chão ao
/// alcance dos pés não se nada — vadeia-se. É este gate que torna redundante o
/// `&& !swimming` que a `standing` NÃO carrega.
///
/// **Mutação que deve sangrar:** tirar o `footing.is_none()` do [`swim_step`].
#[test]
fn the_swim_never_runs_with_ground_under_it() {
    let cfg = armed();
    let g = flat();
    // Submerso até o pescoço, mas de pé no fundo:
    assert!(
        !swim_step(&cfg, DRY, Some(&g), Buoyed(4.0)).active,
        "vadear e' andar, nao nadar"
    );
    // E o chão DESARMA quem já nadava — é como se sai numa praia.
    assert!(
        !swim_step(&cfg, WET, Some(&g), Buoyed(4.0)).active,
        "o chao tem de desarmar a trava"
    );
}

/// **Raspar a superfície não arma** — o limiar existe para isto: atravessar uma
/// poça de um salto não pode interromper o arco com um servo de nado.
///
/// **Mutação que deve sangrar:** trocar `buoyed >= enter` por `buoyed > 0`.
#[test]
fn grazing_the_surface_does_not_arm_the_stroke() {
    let cfg = armed();
    // Os números são os da tabela do `measure_the_swim_threshold`: 9,6% submerso
    // lê `0,25`, e o default do limiar é `1.0` (a linha de flutuação, 27,2%).
    for graze in [0.05f32, 0.25, 0.68, 0.99] {
        assert!(
            !swim_step(&cfg, DRY, None, Buoyed(graze)).active,
            "buoyed {graze} esta' abaixo do limiar e nao pode armar"
        );
    }
    assert!(
        swim_step(&cfg, DRY, None, Buoyed(1.0)).active,
        "no limiar exato ele ARMA"
    );
}

/// **A saída é uma TRAVA, não o limiar ao contrário** — quem já nada continua a
/// nadar com qualquer água, e só larga quando sai dela por completo.
///
/// Sem esta histerese o nadador oscilaria em torno do limiar exatamente onde o
/// jogador está a tentar sair.
///
/// **Mutação que deve sangrar:** tirar o `was.active ||` do [`swim_step`].
#[test]
fn once_swimming_a_shallower_reading_keeps_it() {
    let cfg = armed();
    // Subiu até quase fora: 5% de um peso ainda é água.
    let up = swim_step(&cfg, WET, None, Buoyed(0.05));
    assert!(up.active, "ainda molhado, ainda a nadar: {up:?}");
    // Fora de todo: a trava larga.
    let out = swim_step(&cfg, WET, None, Buoyed(0.0));
    assert!(!out.active, "fora da agua a trava larga: {out:?}");
}

// ── O SERVO ──────────────────────────────────────────────────────────────────

/// **Dois eixos** — e o vertical é o `up`, não a tangente de um chão que não há.
#[test]
fn the_stroke_drives_both_axes() {
    let cfg = armed();
    let right = swim_motor(&cfg, 1.0, 0.0, [0.0, 0.0], UP, DT);
    assert!(right.accel[0] > 0.0, "para a direita: {right:?}");
    assert_eq!(right.accel[1], 0.0, "sem pedido vertical, nada na vertical");

    let rise = swim_motor(&cfg, 0.0, 1.0, [0.0, 0.0], UP, DT);
    assert!(rise.accel[1] > 0.0, "para cima: {rise:?}");
    assert_eq!(rise.accel[0], 0.0);

    let dive = swim_motor(&cfg, 0.0, -1.0, [0.0, 0.0], UP, DT);
    assert!(dive.accel[1] < 0.0, "para baixo: {dive:?}");
}

/// **Um `rise` de zero é o freio** — o motor levado a parar nos dois eixos.
///
/// ⚠️ **Isto é sobre o MOTOR, não sobre o repouso da lei:** o que um nadador
/// parado recebe já não é zero, é a [`swim_rise`] (a linha). O que este gate
/// pina é a outra metade — *dado um alvo de zero, o servo freia* —, e ela é o
/// que faz a linha valer alguma coisa: sem freio o corpo passaria por ela.
#[test]
fn an_idle_swimmer_brakes_toward_a_stop() {
    let cfg = armed();
    let m = swim_motor(&cfg, 0.0, 0.0, [3.0, -2.0], UP, DT);
    assert!(m.accel[0] < 0.0, "freia o que ia para a direita: {m:?}");
    assert!(m.accel[1] > 0.0, "e o que ia para baixo: {m:?}");
}

/// **Na velocidade de nado a lei para de empurrar**, e sem oscilar em torno
/// dela — a última fração vira boost exato, como na caminhada.
#[test]
fn cruising_speed_is_reached_and_not_passed() {
    let cfg = armed();
    let at = swim_motor(&cfg, 1.0, 0.0, [cfg.speed, 0.0], UP, DT);
    assert_eq!(
        at,
        Motor::default(),
        "no alvo nao ha' o que empurrar: {at:?}"
    );

    let window = cfg.acceleration * DT;
    let near = swim_motor(&cfg, 1.0, 0.0, [cfg.speed - window * 0.5, 0.0], UP, DT);
    assert_eq!(near.accel, [0.0, 0.0], "perto do alvo e' BOOST, nao forca");
    assert!(
        (near.boost[0] - window * 0.5).abs() < 1.0e-5,
        "o boost e' exatamente o que falta: {near:?}"
    );
}

/// ⚠️ **A autoridade é por EIXO** — a braçada na diagonal não pode ser mais
/// fraca em cada eixo do que a mesma braçada na horizontal.
///
/// ⚠️ **O oráculo é o ALVO, e a primeira versão deste gate media a coisa
/// errada:** ela comparava o empurrão partindo do repouso, e ali o servo está
/// **saturado** (`|delta| >> a·dt`) — o `accel` vale `±acceleration` seja qual
/// for o alvo, então normalizar o alvo deixava-a **VERDE**. O que distingue as
/// duas leis é onde o servo PARA.
///
/// **Mutação que deve sangrar:** normalizar o alvo, ou dividir um orçamento
/// único entre os dois eixos.
#[test]
fn the_diagonal_is_not_weaker_per_axis() {
    let cfg = armed();
    // A velocidade de cruzeiro na diagonal É `speed` em cada eixo — chegando
    // lá, não há o que empurrar.
    let cruising = swim_motor(&cfg, 1.0, 1.0, [cfg.speed, cfg.speed], UP, DT);
    assert_eq!(
        cruising,
        Motor::default(),
        "a diagonal cruza a `speed` em CADA eixo: {cruising:?}"
    );
    // E o empurrão partindo do repouso é o mesmo dos dois lados.
    let diagonal = swim_motor(&cfg, 1.0, 1.0, [0.0, 0.0], UP, DT);
    let flat_push = swim_motor(&cfg, 1.0, 0.0, [0.0, 0.0], UP, DT);
    assert_eq!(diagonal.accel[0], flat_push.accel[0]);
    assert_eq!(diagonal.accel[1].abs(), diagonal.accel[0].abs());
}

/// O eixo vertical: os dois botões, e o que os dois juntos significam.
#[test]
fn the_vertical_axis_is_symmetric_and_cancels() {
    assert_eq!(vertical_drive(true, false), 1.0);
    assert_eq!(vertical_drive(false, true), -1.0);
    assert_eq!(
        vertical_drive(true, true),
        0.0,
        "subir e descer ao mesmo tempo e' ficar onde esta'"
    );
    assert_eq!(vertical_drive(false, false), 0.0);
}

/// **O DEDO VENCE A LINHA** — sem isto, sair da água e ir ao fundo seriam
/// gestos que a lei desfaz sozinha no tique seguinte.
///
/// ⚠️ A fixture põe um erro de flutuação GRANDE contra o botão, e nos dois
/// sentidos: um `if` invertido daria a metade certa por acaso.
#[test]
fn the_finger_beats_the_line() {
    let cfg = armed();
    // Fundo (`buoyed` muito acima da linha, que pede SUBIR) e o dedo em BAIXO.
    assert_eq!(swim_rise(&cfg, false, true, Buoyed(3.9)), -1.0);
    // E o inverso: quase fora da água (a linha pede DESCER) com o dedo em CIMA.
    assert_eq!(swim_rise(&cfg, true, false, Buoyed(0.05)), 1.0);
}

/// **SEM O DEDO, ELE PROCURA A LINHA** — o repouso desta lei, e o sinal.
///
/// ⚠️ **O sinal é a metade que se erra em silêncio:** `buoyed` cresce com a
/// profundidade, então estar ABAIXO da linha (mais empuxo do que peso) é o caso
/// que pede SUBIR. Trocá-lo daria um nadador que afunda ao soltar os controlos —
/// e o gate de produto que o apanha mede mais de meio metro de erro.
#[test]
fn an_idle_swimmer_seeks_the_line() {
    let cfg = armed();
    assert!(
        swim_rise(&cfg, false, false, Buoyed(cfg.enter + 0.3)) > 0.0,
        "abaixo da linha ele SOBE"
    );
    assert!(
        swim_rise(&cfg, false, false, Buoyed(cfg.enter - 0.3)) < 0.0,
        "acima dela ele DESCE"
    );
    assert_eq!(
        swim_rise(&cfg, false, false, Buoyed(cfg.enter)),
        0.0,
        "e na linha nao rema"
    );
}

/// **A procura satura numa braçada cheia** — o erro é medido em PESOS e o alvo em
/// frações da velocidade, então *um peso de erro é uma braçada*, e mais do que
/// isso continua a ser uma.
///
/// ⚠️ Sem o clamp, uma poça `20×` pediria vinte braçadas: o servo mira uma
/// velocidade que o `speed` não tem, e o nadador sai da água como um foguete.
#[test]
fn the_seek_saturates_at_one_stroke() {
    let cfg = armed();
    assert_eq!(swim_rise(&cfg, false, false, Buoyed(19.0)), 1.0);
    assert_eq!(
        swim_rise(
            &SwimConfig { enter: 20.0, ..cfg },
            false,
            false,
            Buoyed(0.0)
        ),
        -1.0
    );
}

/// `dt <= 0` não produz motor — a mesma cautela do resto da crate (um servo com
/// janela zero dividiria por nada e escreveria um empurrão infinito).
#[test]
fn a_stopped_clock_produces_no_stroke() {
    assert_eq!(
        swim_motor(&armed(), 1.0, 1.0, [0.0, 0.0], UP, 0.0),
        Motor::default()
    );
}
