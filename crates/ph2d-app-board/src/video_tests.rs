//! W19 — o tamanho do tocador segue o do ecrã só quando o zoom ASSENTA.

use std::time::{Duration, Instant};

use super::{SETTLE, settled};

/// ⭐ Um zoom contínuo não muda o tocador a cada quadro: só o último tamanho, [`SETTLE`] depois de parar.
#[test]
fn the_player_size_follows_the_screen_only_once_the_zoom_settles() {
    let t0 = Instant::now();
    let mut asked = None;
    let have = [560, 315];
    // A cada 16 ms um tamanho novo: nada muda.
    for k in 0..30u32 {
        let want = [560 + k * 8, 315 + k * 4];
        let now = t0 + Duration::from_millis(u64::from(k) * 16);
        assert_eq!(settled(&mut asked, have, want, now), None, "passo {k}");
    }
    let last = [560 + 29 * 8, 315 + 29 * 4];
    let stop = t0 + Duration::from_millis(29 * 16);
    assert_eq!(settled(&mut asked, have, last, stop + SETTLE / 2), None, "ainda a assentar");
    assert_eq!(settled(&mut asked, have, last, stop + SETTLE), Some(last), "assentou");
    // Já com o tamanho: nada a fazer.
    assert_eq!(settled(&mut asked, last, last, stop + SETTLE * 3), None);
    assert_eq!(asked, None);
}
