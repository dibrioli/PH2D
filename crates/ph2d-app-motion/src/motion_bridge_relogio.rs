//! ⭐ doc 121 §9.19 (5) — **a régua do quadro do Motion por janela CURTA** (`PH2D_MOTION_RELOGIO=1`): a cada `30`
//! quadros, a parede do quadro, o tempo dos tiques do Motion (média e máximo) e quantos tiques cada quadro
//! recuperou. A janela de `120` quadros do `[frame]` cobre um ciclo inteiro da `=114` (a queda E a pilha) e a
//! média esconde o instante da pilha — onde o quadro cai e o relógio passa a recuperar vários tiques de uma vez.

use std::sync::{LazyLock, Mutex};
use std::time::Instant;

static LIGADO: LazyLock<bool> =
    LazyLock::new(|| std::env::var("PH2D_MOTION_RELOGIO").is_ok_and(|v| v == "1"));

/// O acumulado da janela: quadros, soma e máximo do Motion (ms), soma e máximo dos tiques, o início.
static JANELA: Mutex<(u32, f64, f64, usize, usize, Option<Instant>)> =
    Mutex::new((0, 0.0, 0.0, 0, 0, None));

const QUADROS: u32 = 30;

/// O início dos tiques deste quadro (`None` com a régua desligada).
pub(super) fn comeca(tiques: usize) -> Option<(Instant, usize)> {
    LIGADO.then(|| (Instant::now(), tiques))
}

/// O fim dos tiques deste quadro; imprime a linha `[motion-quadro]` a cada [`QUADROS`].
pub(super) fn regista(inicio: Option<(Instant, usize)>, t: f64) {
    let Some((t0, n)) = inicio else {
        return;
    };
    let ms = t0.elapsed().as_secs_f64() * 1e3;
    let Ok(mut j) = JANELA.lock() else {
        return;
    };
    j.0 += 1;
    j.1 += ms;
    j.2 = j.2.max(ms);
    j.3 += n;
    j.4 = j.4.max(n);
    let desde = *j.5.get_or_insert_with(Instant::now);
    if j.0 == QUADROS {
        let parede = desde.elapsed().as_secs_f64() * 1e3 / f64::from(QUADROS);
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de tiques")]
        let tiques = j.3 as f64 / f64::from(QUADROS);
        eprintln!(
            "[motion-quadro] t {t:.2} s · quadro {parede:.1} ms ({:.0} fps) · motion med {:.1} max {:.1} ms · tiques/quadro {tiques:.2} max {}",
            1000.0 / parede,
            j.1 / f64::from(QUADROS),
            j.2,
            j.4
        );
        *j = (0, 0.0, 0.0, 0, 0, Some(Instant::now()));
    }
}
