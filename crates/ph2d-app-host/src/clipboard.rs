//! **LER a área de transferência com PRAZO** (MiroClone W16): com a feature `wayland-data-control`
//! do `arboard` (o que lê o que OUTRO app copiou numa janela Wayland nativa) a leitura é um
//! `read_to_end` num pipe SEM prazo (`arboard-3.6.1/src/platform/linux/wayland.rs:86`) — um app dono
//! da área de transferência que trave congelava o PH2D. A leitura corre num fio próprio, e passado o
//! prazo desiste-se dela (o fio acaba quando o dono largar o pipe).

use std::sync::mpsc::sync_channel;
use std::time::Duration;

/// O prazo de uma leitura: o do caminho X11 do próprio `arboard`, o que o app tinha antes
/// (`LONG_TIMEOUT_DUR`, `arboard-3.6.1/src/platform/linux/x11.rs:101`).
pub const READ_DEADLINE: Duration = Duration::from_millis(4000);

/// O dono da área de transferência não respondeu dentro do [`READ_DEADLINE`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoAnswer;

/// ⭐ Lê com `f` numa área de transferência própria, num fio, com prazo. `Ok(None)` = não há o que
/// `f` pede; `Err(NoAnswer)` = o dono não respondeu (diga-o na tela: `clipboard.no_answer`).
pub fn read<T: Send + 'static>(
    f: impl FnOnce(&mut arboard::Clipboard) -> Option<T> + Send + 'static,
) -> Result<Option<T>, NoAnswer> {
    within(READ_DEADLINE, move || {
        arboard::Clipboard::new().ok().and_then(|mut c| f(&mut c))
    })
}

/// `job` num fio, à espera dele no máximo `deadline`.
fn within<T: Send + 'static>(
    deadline: Duration,
    job: impl FnOnce() -> Option<T> + Send + 'static,
) -> Result<Option<T>, NoAnswer> {
    let (tx, rx) = sync_channel(1);
    let spawned = std::thread::Builder::new()
        .name("ph2d-clipboard-read".into())
        .spawn(move || {
            let _ = tx.send(job());
        });
    if spawned.is_err() {
        return Ok(None);
    }
    rx.recv_timeout(deadline).map_err(|_| NoAnswer)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ Um dono que não responde não prende quem lê: passado o prazo, `NoAnswer`; a tempo, o valor.
    #[test]
    fn a_reader_that_never_answers_is_abandoned_at_the_deadline() {
        let t = std::time::Instant::now();
        let slow = within(Duration::from_millis(50), || {
            std::thread::sleep(Duration::from_secs(2));
            Some(1)
        });
        assert_eq!(slow, Err(NoAnswer));
        assert!(t.elapsed() < Duration::from_secs(1), "{:?}", t.elapsed());
        assert_eq!(within(Duration::from_secs(5), || Some(7)), Ok(Some(7)));
    }
}
