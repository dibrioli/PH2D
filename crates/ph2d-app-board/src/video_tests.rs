//! W19 — o tamanho do tocador segue o do ecrã só quando o zoom ASSENTA. W20 — o tocador segue o vídeo que
//! TEM, não o activo.

use std::time::{Duration, Instant};

use super::{SETTLE, settled, target};
use ph2d_board_embed::OnScreen;
use ph2d_board_model::{ElementId, Video};
use ph2d_board_render::{Placement, Videos};
use ph2d_vector::Affine;

/// ⭐⭐ W20 — desactivado (clicou fora: `active = None`), o vídeo que o tocador tem continua o alvo — vivo,
/// não pausa; todo fora da vista deixa de ser (pausa); sem tocador, nada.
#[test]
fn the_player_follows_the_video_it_holds_not_the_active_one() {
    let id = ElementId(7);
    let mut v = Videos::default();
    v.activate(id);
    let size = [560.0, 315.0];
    v.on_screen.insert(
        id,
        Placement {
            screen: OnScreen {
                size,
                angle: 0.0,
                covered: false,
                visible: size,
            },
            live: true,
            to_screen: Affine::IDENTITY,
        },
    );
    let video = Video::YouTube {
        id: "aqz-KE-bpKQ".into(),
        start: 0,
        vertical: false,
    };
    v.videos.insert(id, video.clone());
    assert_eq!(target(&v), Some((id, video.clone(), true, size)));
    v.active = None;
    assert_eq!(
        target(&v),
        Some((id, video, true, size)),
        "desactivado, o tocador largou o vídeo (pausava)"
    );
    v.on_screen.clear();
    assert_eq!(target(&v), None, "todo fora da vista: pausa");
    v.player = None;
    assert_eq!(target(&v), None);
}

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
    assert_eq!(
        settled(&mut asked, have, last, stop + SETTLE / 2),
        None,
        "ainda a assentar"
    );
    assert_eq!(
        settled(&mut asked, have, last, stop + SETTLE),
        Some(last),
        "assentou"
    );
    // Já com o tamanho: nada a fazer.
    assert_eq!(settled(&mut asked, last, last, stop + SETTLE * 3), None);
    assert_eq!(asked, None);
}
