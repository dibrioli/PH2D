//! **A compressão espera o silêncio** (n.º 7 item 3, [medição](../../../../../../../../docs/Painter/missao_4k/medicoes/N7_ITEM3_mais.md)
//! «Continuação 3»): o fio de [`super::TintaAdormecida::encolhe`] nascia no flush do «+» e, com o pool rayon e a
//! memória, atrasava o «adormeceu» e o 1.º preview da camada nova. Agora ele só começa depois de [`SILENCIO_MS`] sem
//! evento de caneta (contados a partir do adormecer), e nunca depois de [`ESPERA_MAX_MS`]: a RAM da sessão por
//! comprimir fica limitada no tempo. Acordar a tinta cancela a espera (o fio sai sem a tocar).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::SeqCst};
use std::sync::{Arc, OnceLock, mpsc};
use std::time::{Duration, Instant};

/// Silêncio de caneta que a compressão espera: cobre o «+» (≤ 45 ms) e os primeiros quadros (3 × 16,7 ms) com folga.
const SILENCIO_MS: u64 = 300;
/// Tecto da espera, de RAM: sob traço contínuo a sessão comprime na mesma, no máximo isto depois de adormecer.
const ESPERA_MAX_MS: u64 = 3000;

thread_local! {
    /// **A chave do A×B**: `true` = o fio espera o silêncio; `false` = começa logo, como antes. Lida no fio que adormece.
    pub(crate) static ENCOLHE_NO_SILENCIO: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// n.º 10 (B18.7): a compressão corre num pool rayon PRÓPRIO, de ¼ dos núcleos: no global, os `par_iter` dela enchiam a
/// fila dos operários e o traço que chegasse depois do silêncio esperava por eles. `PH2D_COMPRIME_NO_POOL_GLOBAL=1` =
/// antes (lida uma vez).
fn pool_da_compressao() -> Option<&'static rayon::ThreadPool> {
    static POOL: OnceLock<Option<rayon::ThreadPool>> = OnceLock::new();
    POOL.get_or_init(|| {
        if std::env::var("PH2D_COMPRIME_NO_POOL_GLOBAL").is_ok_and(|v| v == "1") {
            return None;
        }
        let n = std::thread::available_parallelism().map_or(1, |n| (n.get() / 4).max(1));
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .thread_name(|i| format!("ph2d-comprime-{i}"))
            .build()
            .ok()
    })
    .as_ref()
}

thread_local! {
    /// O A×B do gate: `true` = o pool global, como antes. Lida no fio que adormece.
    pub(crate) static COMPRIME_NO_POOL_GLOBAL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

static ULTIMO_GESTO_MS: AtomicU64 = AtomicU64::new(0);

fn agora_ms() -> u64 {
    static EPOCA: OnceLock<Instant> = OnceLock::new();
    u64::try_from(EPOCA.get_or_init(Instant::now).elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// Um evento de caneta na tela: a compressão em espera recomeça a contar.
pub(crate) fn marca_gesto() {
    ULTIMO_GESTO_MS.store(agora_ms(), SeqCst);
}

/// A espera de um fio de compressão: quem baixar `espera` primeiro decide (o fio comprime, ou quem acorda cancela).
#[derive(Clone)]
pub(super) struct Espera {
    espera: Arc<AtomicBool>,
    acorda: mpsc::Sender<()>,
}

impl Espera {
    /// A compressão ainda não começou (nem foi cancelada).
    pub(super) fn pendente(&self) -> bool {
        self.espera.load(SeqCst)
    }

    /// Cancela a compressão ainda por começar. `true` = cancelou: o fio nunca a toca.
    pub(super) fn cancela(&self) -> bool {
        let c = self.espera.swap(false, SeqCst);
        if c {
            let _ = self.acorda.send(());
        }
        c
    }
}

#[cfg(test)]
thread_local! {
    /// A hora de início e de fim do último fio de compressão nascido neste fio (para a sonda e o gate).
    pub(crate) static ULTIMO_FIO: std::cell::Cell<Option<Horas>> =
        const { std::cell::Cell::new(None) };
}

#[cfg(test)]
type Horas = Arc<std::sync::Mutex<(Option<Instant>, Option<Instant>)>>;

#[cfg(test)]
thread_local! {
    /// Os operários do pool rayon onde correu o último fio nascido neste fio (0 = nenhum: os `par_iter` vão ao global).
    pub(crate) static ULTIMO_POOL: std::cell::Cell<Option<Arc<std::sync::atomic::AtomicUsize>>> =
        const { std::cell::Cell::new(None) };
}
#[cfg(not(test))]
struct Horas;

#[cfg_attr(not(test), allow(clippy::missing_const_for_fn))]
fn horas_novas() -> Horas {
    #[cfg(test)]
    {
        let h = Arc::new(std::sync::Mutex::new((None, None)));
        ULTIMO_FIO.set(Some(Arc::clone(&h)));
        h
    }
    #[cfg(not(test))]
    Horas
}

#[cfg_attr(not(test), allow(clippy::missing_const_for_fn))]
fn marca(h: &Horas, fim: bool) {
    #[cfg(test)]
    {
        let mut g = h.lock().expect("as horas do fio");
        if fim {
            g.1 = Some(Instant::now())
        } else {
            g.0 = Some(Instant::now())
        }
    }
    #[cfg(not(test))]
    let _ = (h, fim);
}

/// Lança `comprime` num fio: logo (chave desligada) ou depois do silêncio. Devolve a espera para cancelar.
pub(super) fn lanca(comprime: impl FnOnce() + Send + 'static) -> Option<Espera> {
    let silencio = ENCOLHE_NO_SILENCIO.get();
    let horas = horas_novas();
    let pool = pool_da_compressao().filter(|_| !COMPRIME_NO_POOL_GLOBAL.get());
    #[cfg(test)]
    let fios_vistos = {
        let f = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        ULTIMO_POOL.set(Some(Arc::clone(&f)));
        f
    };
    let corre = move || {
        let faz = || {
            #[cfg(test)]
            fios_vistos.store(
                rayon::current_thread_index().map_or(0, |_| rayon::current_num_threads()),
                SeqCst,
            );
            marca(&horas, false);
            comprime();
            marca(&horas, true);
        };
        match pool {
            Some(p) => p.install(faz),
            None => faz(),
        }
    };
    if !silencio {
        std::thread::spawn(corre);
        return None;
    }
    let (tx, rx) = mpsc::channel();
    let espera = Espera {
        espera: Arc::new(AtomicBool::new(true)),
        acorda: tx,
    };
    let flag = Arc::clone(&espera.espera);
    let inicio = agora_ms();
    std::thread::spawn(move || {
        loop {
            let agora = agora_ms();
            let quieto = ULTIMO_GESTO_MS.load(SeqCst).max(inicio);
            let falta_silencio = (quieto + SILENCIO_MS).saturating_sub(agora);
            let falta_tecto = (inicio + ESPERA_MAX_MS).saturating_sub(agora);
            let falta = falta_silencio.min(falta_tecto);
            if falta == 0 {
                break;
            }
            match rx.recv_timeout(Duration::from_millis(falta)) {
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                _ => return, // cancelada, ou a tinta já saiu
            }
        }
        if flag.swap(false, SeqCst) {
            corre();
        }
    });
    Some(espera)
}
