//! ADR-0187 — o quadro ao vivo PELA INTERNET na família: o convidado entra por uma ROTA (a rede local
//! com o código, ou o link do convite) e o anfitrião abre a entrada pela internet numa thread
//! (`ph2d_board_internet::InternetHost`) e copia o link. Módulo filho de [`super`] (o `live.rs`).

use std::net::SocketAddr;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use ph2d_board_internet::{DhtMode, InternetConfig, InternetHost, LinkGuest, Mapping};
use ph2d_board_net as net;
use ph2d_board_sync as sync;
use ph2d_editor_core::documents::live::{LiveInternet, LiveLink, LiveRole};
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
    /// ADR-0187 — a internet: o convite (candidatos + chave) e o nó e o ponto de encontro da sessão,
    /// os mesmos a cada volta (A12); `local` = só esta máquina (testes).
    Link { guest: LinkGuest, local: bool },
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
            let v = hero.documents.live_view_mut();
            v.internet = LiveInternet::Ready { link };
            v.link.strict_nat = strict(ih.mapping());
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
    // Por link, o painel diz que procura na internet e por quanto tempo no máximo.
    let searching = matches!(route, Route::Link { .. })
        .then(|| u32::try_from(LINK_TIMEOUT.as_secs()).unwrap_or(u32::MAX));
    hero.documents.live_view_mut().link = LiveLink {
        searching,
        ..LiveLink::default()
    };
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

/// Uma tentativa de ligar a decorrer (numa thread): o resultado e, por link, a medição do NAT deste
/// lado (chega antes, quando os STUN respondem).
pub(super) struct Dialing {
    rx: mpsc::Receiver<Result<net::Guest, net::JoinError>>,
    nat: Option<mpsc::Receiver<Mapping>>,
}

impl Dialing {
    /// O resultado, se acabou. A medição do NAT vai para o painel logo que chega; entrar acaba a
    /// espera por link.
    pub(super) fn poll(&self, hero: &mut HeroScreen) -> Option<Result<net::Guest, net::JoinError>> {
        if let Some(m) = self.nat.as_ref().and_then(|n| n.try_recv().ok()) {
            hero.documents.live_view_mut().link.strict_nat = strict(m);
        }
        let r = self.rx.try_recv().ok()?;
        if r.is_ok() {
            hero.documents.live_view_mut().link.searching = None;
        }
        Some(r)
    }
}

/// Liga numa thread à parte: o ecrã nunca espera pela rede.
pub(super) fn dial(route: &Route) -> Dialing {
    let (tx, rx) = mpsc::channel();
    let (ntx, nrx) = mpsc::channel();
    let link = matches!(route, Route::Link { .. });
    let route = route.clone();
    std::thread::spawn(move || {
        let r = match route {
            Route::Lan { addr, code } => net::Guest::connect(addr, &code, CONNECT_TIMEOUT, waker()),
            Route::Link { guest, local } => guest.join(
                &internet_config(local),
                LINK_TIMEOUT,
                net::Liveness::default(),
                waker(),
                move |m| {
                    let _ = ntx.send(m);
                },
            ),
        };
        let _ = tx.send(r);
    });
    Dialing {
        rx,
        nat: link.then_some(nrx),
    }
}

/// ⭐ A entrada por link falhou antes de chegar ao quadro: a frase diz o que fazer, pelo porquê (a
/// sala fechou, o link é velho, esta máquina está sem internet, ou não se achou — com a rede que
/// bloqueia, se foi medida), e fica no painel até a próxima tentativa. `false` = não foi por link.
pub(super) fn link_failed(hero: &mut HeroScreen, route: &Route, e: &net::JoinError) -> bool {
    use std::io::ErrorKind as K;
    if !matches!(route, Route::Link { .. }) {
        return false;
    }
    let key = match e {
        net::JoinError::WrongCode | net::JoinError::InvalidCode => "board.live.link_old",
        net::JoinError::Unreachable(K::ConnectionRefused) => "board.live.link_closed",
        net::JoinError::Unreachable(
            K::NetworkUnreachable | K::NetworkDown | K::AddrNotAvailable,
        ) => "board.live.link_offline",
        net::JoinError::Unreachable(_) if hero.documents.live_view().link.strict_nat => {
            "board.live.link_unreachable_strict"
        }
        net::JoinError::Unreachable(_) => "board.live.link_unreachable",
        _ => return false,
    };
    let why = tr(key).to_owned();
    hero.documents.live_view_mut().link.failed = Some(why.clone());
    hero.documents.tell(why);
    true
}

/// O NAT desta casa muda o endereço por destino (simétrico): furar não chega, o convite pode não
/// passar.
fn strict(m: Mapping) -> bool {
    #[cfg(test)]
    let m = FAKE_NAT.get().unwrap_or(m);
    m == Mapping::Dependent
}

#[cfg(test)]
thread_local! {
    static FAKE_NAT: std::cell::Cell<Option<Mapping>> = const { std::cell::Cell::new(None) };
}

/// Para os testes da família: nesta thread, os STUN «medem» `m` (a rede de casa dos testes não tem
/// NAT nenhum).
#[cfg(test)]
pub(crate) fn fake_nat_for_tests(m: Mapping) {
    FAKE_NAT.set(Some(m));
}

#[cfg(test)]
#[path = "live_link_tests.rs"]
mod tests;
