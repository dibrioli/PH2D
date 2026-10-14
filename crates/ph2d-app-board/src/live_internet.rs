//! ADR-0187 — o quadro ao vivo PELA INTERNET na família: o convidado entra por uma ROTA (a rede local
//! com o código, ou o link do convite) e o anfitrião abre a entrada pela internet numa thread
//! (`ph2d_board_internet::InternetHost`) e copia o link. Módulo filho de [`super`] (o `live.rs`).

use std::net::SocketAddr;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use ph2d_board_internet::{DhtMode, InternetConfig, InternetHost};
use ph2d_board_net as net;
use ph2d_board_sync as sync;
use ph2d_editor_core::documents::live::{LiveInternet, LiveRole};
use ph2d_editor_core::screens::HeroScreen;
use ph2d_i18n::{tr, tr_with};

use super::{CONNECT_TIMEOUT, Live, State, random, waker};

/// ADR-0187 — o prazo de uma tentativa pelo link: o ponto de encontro (a DHT) leva segundos a dar os
/// endereços, e o anfitrião fura a cada volta dele (2 s).
const LINK_TIMEOUT: Duration = Duration::from_secs(30);

/// Por onde o convidado entra (e volta a entrar se cair).
#[derive(Clone)]
pub(super) enum Route {
    /// A rede local: o endereço e o código curto.
    Lan { addr: SocketAddr, code: String },
    /// ADR-0187 — a internet: o convite (candidatos + chave); `local` = só esta máquina (testes).
    Link { invite: net::Invite, local: bool },
}

/// ADR-0187 — a entrada pela internet do anfitrião.
#[derive(Default)]
pub(super) enum Internet {
    #[default]
    Off,
    Preparing(mpsc::Receiver<std::io::Result<InternetHost>>),
    On(InternetHost),
}

/// ADR-0187 — a configuração da internet: a sério, ou só esta máquina (os testes, e
/// `PH2D_BOARD_INTERNET=local` para ensaiar sem rede).
fn internet_config(local: bool) -> InternetConfig {
    let local = local || std::env::var("PH2D_BOARD_INTERNET").is_ok_and(|v| v.trim() == "local");
    if local {
        InternetConfig {
            stun: vec![],
            dht: DhtMode::Off,
            v6: false,
            port: 0,
            via: None,
        }
    } else {
        InternetConfig::public()
    }
}

/// ADR-0187 — «Invite over the internet»: prepara numa thread (DNS, STUN); já aberta, copia outra vez.
pub(super) fn invite_internet(st: &mut State, hero: &mut HeroScreen) {
    if !matches!(st.live, Some(Live::Host { .. })) {
        return;
    }
    match &st.internet {
        Internet::On(ih) => {
            hero.store.set_clipboard_copy(ih.link());
            hero.documents
                .tell(tr("board.live.invite_net_copied").to_owned());
        }
        Internet::Preparing(_) => {}
        Internet::Off => {
            let (tx, rx) = mpsc::channel();
            let local = st.internet_local;
            std::thread::spawn(move || {
                let _ = tx.send(InternetHost::prepare(&internet_config(local)));
            });
            st.internet = Internet::Preparing(rx);
            hero.documents.live_view_mut().internet = LiveInternet::Preparing;
        }
    }
}

/// ADR-0187 — a preparação acabou: a entrada abre no anfitrião e o link vai para a área de
/// transferência. Sem sala de anfitrião, a entrada fecha.
pub(super) fn internet_tick(st: &mut State, hero: &mut HeroScreen) {
    let Some(Live::Host { net, .. }) = &mut st.live else {
        st.internet = Internet::Off;
        return;
    };
    let Internet::Preparing(rx) = &st.internet else {
        return;
    };
    let got = match rx.try_recv() {
        Ok(r) => r,
        Err(mpsc::TryRecvError::Empty) => return,
        Err(mpsc::TryRecvError::Disconnected) => Err(std::io::ErrorKind::BrokenPipe.into()),
    };
    match got {
        Ok(ih) => {
            ih.attach(net);
            let link = ih.link();
            hero.store.set_clipboard_copy(link.clone());
            hero.documents.live_view_mut().internet = LiveInternet::Ready { link };
            hero.documents
                .tell(tr("board.live.invite_net_ready").to_owned());
            st.internet = Internet::On(ih);
        }
        Err(e) => {
            let why = tr_with("board.live.invite_net_failed", &[("error", &e.to_string())]);
            hero.documents.live_view_mut().internet = LiveInternet::Failed(why.clone());
            hero.documents.tell(why);
            st.internet = Internet::Off;
        }
    }
}

pub(super) fn join_by(st: &mut State, hero: &mut HeroScreen, route: Route) {
    let me = hero.documents.author().to_owned();
    st.live = Some(Live::Guest {
        net: None,
        s: sync::Guest::new(random(), me, random(), random()),
        board: None,
        dialing: Some(dial(&route)),
        route,
        lost: None,
        last_try: Some(Instant::now()),
    });
    hero.documents.live_view_mut().role = LiveRole::Joining;
}

/// Liga numa thread à parte: o ecrã nunca espera pela rede.
pub(super) fn dial(route: &Route) -> mpsc::Receiver<Result<net::Guest, net::JoinError>> {
    let (tx, rx) = mpsc::channel();
    let route = route.clone();
    std::thread::spawn(move || {
        let r = match route {
            Route::Lan { addr, code } => net::Guest::connect(addr, &code, CONNECT_TIMEOUT, waker()),
            Route::Link { invite, local } => ph2d_board_internet::join(
                &invite,
                &internet_config(local),
                LINK_TIMEOUT,
                net::Liveness::default(),
                waker(),
            ),
        };
        let _ = tx.send(r);
    });
    rx
}
