//! **A FILA DO DAB** (N10 F2c) — o depósito do Wet num operário próprio, fora do evento da caneta (filho de
//! [`super`], a rota do dab). Desenho e números: `docs/Painter/missao_4k/medicoes/N10_cerdas_mistura.md`.
//!
//! Viaja a [`WetSession`] INTEIRA (o `pousa_o_dab` lê lanes, `po`, `medias_do_grao` e o motor) com o
//! [`LoteDoDab`] dono dos dados, os sub-dabs e o `DabRng`; o operário pousa na ordem de chegada e devolve tudo.
//! [`drena_os_dabs`] é a ÚNICA espera: nunca dentro de um job do rayon (Bug #66, a mesma lei do `bring_home`).
//!
//! `PH2D_DAB_SINCRONO=1` (lida uma vez) pousa no fio do evento, como antes. Nos testes o síncrono é o padrão (os
//! oráculos da rota vivem em `thread_local`); um gate liga a fila com [`fila_no_teste`].

use super::super::WetSession;
use super::pousa_o_dab::{self, LoteDoDab};
use ph2d_painter_brush::Dab;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Mutex, OnceLock};

/// Um sub-dab do lote: o índice do dab, `k` (de trás para a frente), o sub-passo da cápsula e o dab.
pub(super) type SubDab = (usize, u64, Option<f32>, Dab);

/// O que viaja ao operário e volta dele.
pub(super) struct Deposito {
    pub(super) lote: LoteDoDab,
    pub(super) sub_dabs: Vec<SubDab>,
    pub(super) rng: super::super::tiling::DabRng,
    pub(super) sess: WetSession,
}

const _: () = {
    const fn envia<T: Send + 'static>() {}
    envia::<Deposito>();
};

/// O corpo do laço, o mesmo nos dois fios: cada sub-dab entra na água, na ordem da mão.
pub(super) fn deposita(d: &mut Deposito) {
    let Deposito {
        lote,
        sub_dabs,
        rng,
        sess,
    } = d;
    for (didx, sub_k, passo, dab) in sub_dabs.iter() {
        pousa_o_dab::pousa_o_dab(lote, sess, rng, *didx, *sub_k, *passo, dab);
    }
}

type Volta = std::thread::Result<Box<Deposito>>;

struct Operario {
    ida: Sender<Box<Deposito>>,
    volta: Receiver<Volta>,
}

fn operario() -> &'static Mutex<Operario> {
    static O: OnceLock<Mutex<Operario>> = OnceLock::new();
    O.get_or_init(|| {
        let (ida, rx) = channel::<Box<Deposito>>();
        let (tx, volta) = channel::<Volta>();
        std::thread::Builder::new()
            .name("ph2d-wet-dab".into())
            .spawn(move || {
                while let Ok(mut d) = rx.recv() {
                    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                        deposita(&mut d);
                        d
                    }));
                    if tx.send(r).is_err() {
                        return;
                    }
                }
            })
            .expect("o fio do depósito do Wet");
        Mutex::new(Operario { ida, volta })
    })
}

#[cfg(test)]
thread_local! {
    static FILA_NO_TESTE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Porta de TESTE: liga a fila neste fio (o padrão dos testes é o síncrono).
#[cfg(test)]
pub(in crate::tool::paint) fn fila_no_teste(ligada: bool) {
    FILA_NO_TESTE.set(ligada);
}

/// `true` = o depósito pousa no fio do evento (`PH2D_DAB_SINCRONO=1`, ou um teste sem [`fila_no_teste`]).
pub(super) fn sincrono() -> bool {
    #[cfg(test)]
    {
        !FILA_NO_TESTE.get()
    }
    #[cfg(not(test))]
    {
        static S: OnceLock<bool> = OnceLock::new();
        *S.get_or_init(|| std::env::var("PH2D_DAB_SINCRONO").is_ok_and(|v| v == "1"))
    }
}

/// Envia o lote ao operário. O lote seguinte espera atrás dele: a ordem da mão é a ordem da fila.
pub(super) fn enfileira(d: Deposito) -> EmVoo {
    let o = operario()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert!(
        o.ida.send(Box::new(d)).is_ok(),
        "o fio do depósito do Wet morreu"
    );
    EmVoo(())
}

/// Um lote com o operário: só [`drena_os_dabs`] o traz de volta.
#[must_use = "a sessão está com o operário: drene-a"]
pub(super) struct EmVoo(());

/// **Espera o lote em voo** e devolve-o. Um pânico do operário volta a este fio.
pub(super) fn drena_os_dabs(_: EmVoo) -> Deposito {
    assert!(
        rayon::current_thread_index().is_none(),
        "espera pelo depósito dentro de um job do rayon (Bug #66): drene antes de `no_pool`"
    );
    let o = operario()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match o.volta.recv().expect("o fio do depósito do Wet morreu") {
        Ok(d) => *d,
        Err(p) => {
            drop(o);
            std::panic::resume_unwind(p)
        }
    }
}

#[cfg(test)]
#[path = "fila_do_dab_tests.rs"]
mod tests;
