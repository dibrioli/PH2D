//! ⭐ **O quadro ao vivo no ecrã** (MiroClone W11, ADR-0181, `docs/MiroClone/08_ao_vivo.md`): o botão
//! «Live» do canto, o painel (abrir à rede · entrar com o código · a sala), os avatares, seguir
//! alguém (c21) e o meu cursor. Módulo filho da equipa ([`super`]): o painel ocupa a MESMA coluna dos
//! painéis dela, um de cada vez.
//!
//! O ecrã SÓ lê o [`crate::documents::live::LiveView`] e pede por `ask_live`; a rede e a sessão são da
//! família (`ph2d-app-board::live`).

use super::super::super::HeroScreen;
use crate::documents::live::{LiveRequest, LiveRole};
use crate::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use crate::widget::ButtonState;
use ph2d_a11y::NodeId;
use ph2d_board_model::Camera;

#[path = "board_live_others.rs"]
mod others;
#[path = "board_live_paint.rs"]
mod paint;

pub use others::paint as paint_others;
#[cfg(test)]
pub(crate) use paint::row_texts;
pub use paint::{paint, paint_button};

/// De quanto em quanto tempo o painel aberto (e sem sessão) volta a procurar quadros na rede. É o
/// ritmo do aviso da família na rede local, não um tecto: uma procura é um pacote.
pub const DISCOVER_EVERY_S: f64 = 3.0; // LITERAL-PX-OK: segundos entre procuras (o pedido do dono)

/// O comprimento do código de uma sala.
pub const CODE_LEN: usize = 6;

/// Um botão de tamanho fixo do quadro ao vivo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiveButton {
    Close,
    /// O fundo do painel (toma o clique).
    Surface,
    /// `Off` — abrir o quadro activo à rede.
    Open,
    /// `Off` — entrar com o código (e o endereço, se escrito).
    Join,
    /// `Joining` — desistir.
    Cancel,
    /// «Trazer todos» para a minha vista.
    Summon,
    /// O anfitrião fecha a sala.
    End,
    /// O convidado sai.
    Leave,
    /// O × do selo «Following …».
    Unfollow,
    /// O corpo do selo (toma o clique).
    Badge,
    /// «Permitir na rede de casa» (a firewall; pede a senha do sistema).
    FirewallAllow,
    /// «Remover a permissão».
    FirewallRemove,
    /// `Hosting` — ADR-0187: abrir à internet e copiar o link (outra vez, se já aberta).
    InviteInternet,
    /// `Off` — ADR-0187: entrar pelo link colado no campo do código.
    JoinLink,
}

/// ⛔ Só se ACRESCENTA no fim: o id de um botão é a posição dele aqui.
const ALL: [LiveButton; 14] = [
    LiveButton::Close,
    LiveButton::Surface,
    LiveButton::Open,
    LiveButton::Join,
    LiveButton::Cancel,
    LiveButton::Summon,
    LiveButton::End,
    LiveButton::Leave,
    LiveButton::Unfollow,
    LiveButton::Badge,
    LiveButton::FirewallAllow,
    LiveButton::FirewallRemove,
    LiveButton::InviteInternet,
    LiveButton::JoinLink,
];

/// O que um id derivado é.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    Button(LiveButton),
    /// O avatar pintado `i` (o par em `LivePanel::avatars[i]`).
    Avatar(usize),
    /// A linha de achados `i` (o endereço em `LivePanel::found[i]`).
    Found(usize),
}

/// ⛔ Bits 44–47 diferentes dos da navegação, da facilitação e da equipa (`0x4e61_1…`–`0x4e61_a…`).
const SALT: u64 = 0x4e61_b000_0000_0000;
const SALT_AVATAR: u64 = 0x4e61_c000_0000_0000;
const SALT_FOUND: u64 = 0x4e61_d000_0000_0000;

/// O id de um alvo do quadro ao vivo.
#[must_use]
pub fn node(h: Hit) -> NodeId {
    NodeId(match h {
        Hit::Button(b) => SALT ^ (ALL.iter().position(|x| *x == b).expect("na tabela") as u64 + 1),
        Hit::Avatar(i) => SALT_AVATAR ^ (i as u64 + 1),
        Hit::Found(i) => SALT_FOUND ^ (i as u64 + 1),
    })
}

/// O alvo de um id (`None` = não é do quadro ao vivo). Cada descodificador exige os bits acima do
/// que carrega a zero.
#[must_use]
pub fn hit_of(id: NodeId) -> Option<Hit> {
    let low = |salt: u64| {
        let v = id.0 ^ salt;
        (v >> 40 == 0 && v != 0).then(|| usize::try_from(v - 1).unwrap_or(usize::MAX))
    };
    if let Some(i) = low(SALT) {
        return ALL.get(i).map(|b| Hit::Button(*b));
    }
    if let Some(i) = low(SALT_AVATAR) {
        return Some(Hit::Avatar(i));
    }
    low(SALT_FOUND).map(Hit::Found)
}

fn tooltip_key(b: LiveButton) -> &'static str {
    match b {
        LiveButton::Close => "board.live.close",
        LiveButton::Surface | LiveButton::Badge => "board.live.surface",
        LiveButton::Open => "board.live.open_tip",
        LiveButton::Join => "board.live.join_tip",
        LiveButton::Cancel => "board.live.cancel",
        LiveButton::Summon => "board.live.summon_tip",
        LiveButton::End => "board.live.end",
        LiveButton::Leave => "board.live.leave",
        LiveButton::Unfollow => "board.live.unfollow_tip",
        LiveButton::FirewallAllow => "board.live.fw_allow_tip",
        LiveButton::FirewallRemove => "board.live.fw_remove_tip",
        LiveButton::InviteInternet => "board.live.invite_net_tip",
        LiveButton::JoinLink => "board.live.join_link_tip",
    }
}

pub(crate) fn register(store: &mut WidgetStore, id: NodeId) {
    if store.button_state(id).is_none() {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// Regista os botões fixos (pintado ⇒ clicável). Chamado pelo `board_nav::populate`.
pub fn populate(store: &mut WidgetStore) {
    for b in ALL {
        let id = node(Hit::Button(b));
        register(store, id);
        store.set_tooltip(id, ph2d_i18n::tr(tooltip_key(b)));
    }
}

/// Os dois campos do `Off` existem (vazios, sem teclado) para o pintor os desenhar.
fn ensure_fields(store: &mut WidgetStore) {
    for id in [
        crate::ids::BOARD_LIVE_CODE_INPUT,
        crate::ids::BOARD_LIVE_ADDR_INPUT,
    ] {
        if store.get(id).is_none() {
            store.register(
                id,
                InteractiveState::TextInput {
                    state: crate::widget::TextInputState::Normal,
                    text: String::new(),
                    caret: 0,
                    selection_anchor: None,
                },
            );
            store.mark_cancel_on_escape(id);
        }
    }
}

/// ⭐ O botão «Live» do canto: abre ou fecha o painel. Abrir fecha os painéis da equipa (um de cada
/// vez) e, sem sessão, procura quadros na rede logo.
pub fn toggle(hero: &mut HeroScreen) {
    if hero.documents.live_view.panel.open {
        hero.documents.live_view.panel.open = false;
        return;
    }
    super::only_panel(&mut hero.documents.live.team);
    ensure_fields(&mut hero.store);
    let lv = &mut hero.documents.live_view;
    lv.panel.open = true;
    lv.panel.discover_in = DISCOVER_EVERY_S;
    if lv.role == LiveRole::Off {
        hero.documents.ask_live(LiveRequest::Discover);
    }
}

/// O painel está aberto?
#[must_use]
pub fn is_open(hero: &HeroScreen) -> bool {
    hero.documents.live_view.panel.open
}

/// Um painel da equipa aberto DEPOIS do «Live» (abrir o «Live» fecha-os) ganha-lhe a coluna.
pub(super) fn yield_to_team(hero: &mut HeroScreen) {
    let t = &hero.documents.live.team;
    if t.list_open || t.versions_open || t.export.is_some() {
        hero.documents.live_view.panel.open = false;
    }
}

/// ⭐ O tique (uma chamada por imagem, com o `dt` de parede): a procura a cada
/// [`DISCOVER_EVERY_S`] com o painel aberto e sem sessão, e o seguir larga quem já não está na sala.
pub fn tick(hero: &mut HeroScreen, dt: f64) {
    let on_board = hero.documents.active().is_some();
    let lv = &mut hero.documents.live_view;
    if let Some(f) = lv.following
        && !lv.peers.iter().any(|p| p.peer == f && !p.me)
    {
        lv.following = None;
    }
    if !(on_board && lv.panel.open && lv.role == LiveRole::Off) {
        return;
    }
    lv.panel.discover_in -= dt;
    if lv.panel.discover_in <= 0.0 {
        lv.panel.discover_in = DISCOVER_EVERY_S;
        hero.documents.ask_live(LiveRequest::Discover);
    }
}

/// c21 — um gesto MEU na vista (pan, zoom, carregar no quadro, enquadrar) pára de seguir.
pub fn unfollow(hero: &mut HeroScreen) {
    hero.documents.live_view.following = None;
}

/// A câmara do quadro activo (para ver se um clique ou uma tecla a mexeu).
#[must_use]
pub fn camera(hero: &HeroScreen) -> Option<Camera> {
    hero.documents.active_board().map(|b| b.camera)
}

/// Pára de seguir se a câmara mudou desde `before` (o gesto foi meu).
pub fn unfollow_if_moved(hero: &mut HeroScreen, before: Option<Camera>) {
    if camera(hero) != before {
        unfollow(hero);
    }
}

/// O meu cursor no mundo do quadro da sala: escrito a cada movimento sobre a área dele, `None` fora
/// dela ou noutra aba.
pub fn cursor(hero: &mut HeroScreen, x: f32, y: f32) {
    let c = hero.last_canvas;
    let over = x >= c.x && y >= c.y && x < c.x + c.w && y < c.y + c.h;
    let room = hero.documents.live_view.board;
    let world = hero
        .documents
        .active_board()
        .filter(|b| over && room == Some(b.id))
        .map(|b| {
            let area = super::super::super::board_view::area_of(c);
            b.camera.to_world(area, [f64::from(x), f64::from(y)])
        });
    hero.documents.live_view.my_cursor = world;
}

/// O código como se escreve na sala: só letras e algarismos, em maiúsculas (aceita `abc def`,
/// `abc-def`).
#[must_use]
pub fn normalize_code(s: &str) -> String {
    s.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

/// `ABCDEF` → `ABC-DEF` (como o anfitrião o lê em voz alta).
#[must_use]
pub fn show_code(code: &str) -> String {
    let c = normalize_code(code);
    if c.len() == CODE_LEN {
        let (a, b) = c.split_at(CODE_LEN / 2);
        format!("{a}-{b}")
    } else {
        c
    }
}

fn field(hero: &HeroScreen, id: NodeId) -> String {
    super::super::field_text(&hero.store, id)
}

/// ADR-0187 — o texto é um link de convite (`ph2d-board://…`), não um código.
#[must_use]
pub fn is_link(s: &str) -> bool {
    s.trim_start()
        .get(..LINK_SCHEME.len())
        .is_some_and(|p| p.eq_ignore_ascii_case(LINK_SCHEME))
}

/// O começo de todo convite (o mesmo de `ph2d_board_net::SCHEME`, provado no teste da família: o
/// ecrã não depende da rede).
pub const LINK_SCHEME: &str = "ph2d-board://";

/// «Join by link»: o link colado no campo do código vai inteiro à família (a chave distingue
/// maiúsculas: nunca se normaliza).
fn join_link(hero: &mut HeroScreen) {
    let link = field(hero, crate::ids::BOARD_LIVE_CODE_INPUT);
    if !is_link(&link) {
        super::notice(hero, "board.live.link_bad");
        return;
    }
    hero.store.set_focus(None);
    hero.documents.ask_live(LiveRequest::JoinLink { link });
}

/// «Join»: o código normalizado (6 caracteres, senão a frase que o diz) e o endereço escolhido; um
/// link colado no lugar do código entra como link.
fn join(hero: &mut HeroScreen) {
    if is_link(&field(hero, crate::ids::BOARD_LIVE_CODE_INPUT)) {
        join_link(hero);
        return;
    }
    let code = normalize_code(&field(hero, crate::ids::BOARD_LIVE_CODE_INPUT));
    if code.len() != CODE_LEN {
        super::notice(hero, "board.live.code_short");
        return;
    }
    let addr = field(hero, crate::ids::BOARD_LIVE_ADDR_INPUT)
        .trim()
        .to_owned();
    let addr = (!addr.is_empty()).then_some(addr);
    hero.store.set_focus(None);
    hero.documents.ask_live(LiveRequest::Join { addr, code });
}

/// O campo do código mostra-se em maiúsculas enquanto se escreve (o caret fica onde estava).
fn upper_code(store: &mut WidgetStore) {
    if let Some(InteractiveState::TextInput { text, .. }) =
        store.get_mut(crate::ids::BOARD_LIVE_CODE_INPUT)
    {
        let up = text.to_ascii_uppercase();
        // Um link colado fica como está: a chave dele distingue maiúsculas.
        if *text != up && !is_link(text) {
            *text = up;
        }
    }
}

/// ⭐ Os cliques e os campos do quadro ao vivo. Corre no pré-despacho (ids derivados).
pub fn apply_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    use crate::ids::{BOARD_LIVE_ADDR_INPUT as ADDR, BOARD_LIVE_CODE_INPUT as CODE};
    match event {
        WidgetEvent::Submit(id) if id == CODE || id == ADDR => {
            join(hero);
            return true;
        }
        WidgetEvent::TextChanged(id) if id == CODE => {
            upper_code(&mut hero.store);
            return true;
        }
        WidgetEvent::Cancel(id) | WidgetEvent::Blur(id) | WidgetEvent::TextChanged(id)
            if id == CODE || id == ADDR =>
        {
            return true;
        }
        _ => {}
    }
    let (WidgetEvent::Click(id) | WidgetEvent::DoubleClick(id)) = event else {
        return false;
    };
    let Some(h) = hit_of(id) else {
        return false;
    };
    let lv = &mut hero.documents.live_view;
    match h {
        // W12 — com os nomes anónimos não se segue ninguém (o Miro: «Follow is no longer available»).
        Hit::Avatar(_) if lv.fac.private.as_ref().is_some_and(|p| p.anonymous) => {
            hero.documents
                .tell(ph2d_i18n::tr("board.private.no_follow").to_owned());
        }
        Hit::Avatar(i) => {
            if let Some(&peer) = lv.panel.avatars.get(i) {
                lv.following = (lv.following != Some(peer)).then_some(peer);
            }
        }
        Hit::Found(i) => {
            if let Some(addr) = lv.panel.found.get(i).cloned() {
                ensure_fields(&mut hero.store);
                if let Some(InteractiveState::TextInput { text, caret, .. }) =
                    hero.store.get_mut(ADDR)
                {
                    *caret = addr.len();
                    *text = addr;
                }
                let code = field(hero, CODE);
                super::super::open_field(&mut hero.store, CODE, code);
            }
        }
        Hit::Button(b) => button(hero, b),
    }
    true
}

fn button(hero: &mut HeroScreen, b: LiveButton) {
    let ask = |hero: &mut HeroScreen, r: LiveRequest| hero.documents.ask_live(r);
    match b {
        LiveButton::Close => hero.documents.live_view.panel.open = false,
        LiveButton::Open => ask(hero, LiveRequest::Host),
        LiveButton::Join => join(hero),
        LiveButton::Cancel | LiveButton::End | LiveButton::Leave => ask(hero, LiveRequest::Leave),
        LiveButton::Summon => ask(hero, LiveRequest::Summon),
        LiveButton::Unfollow => unfollow(hero),
        LiveButton::FirewallAllow => ask(hero, LiveRequest::FirewallAllow),
        LiveButton::FirewallRemove => ask(hero, LiveRequest::FirewallRemove),
        LiveButton::InviteInternet => ask(hero, LiveRequest::InviteInternet),
        LiveButton::JoinLink => join_link(hero),
        LiveButton::Surface | LiveButton::Badge => {}
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn every_id_reads_back_and_none_belongs_to_another_family() {
        let mut all: Vec<Hit> = ALL.iter().map(|b| Hit::Button(*b)).collect();
        all.extend((0..40).map(Hit::Avatar));
        all.extend((0..40).map(Hit::Found));
        for h in &all {
            assert_eq!(hit_of(node(*h)), Some(*h), "{h:?}");
            assert_eq!(
                super::super::hit_of(node(*h)),
                None,
                "{h:?} lê-se como da equipa"
            );
        }
        for b in [
            super::super::TeamButton::CardClose,
            super::super::TeamButton::PageDown,
        ] {
            assert_eq!(
                hit_of(super::super::node(super::super::Hit::Button(b))),
                None
            );
        }
        let nav = super::super::super::button_node(super::super::super::NavButton::Live);
        assert_eq!(hit_of(nav), None);
    }

    #[test]
    fn the_code_is_typed_any_way_and_shown_in_halves() {
        assert_eq!(normalize_code(" ab3-x9z "), "AB3X9Z");
        assert_eq!(show_code("ab3x9z"), "AB3-X9Z");
        assert_eq!(show_code("ab"), "AB");
    }
}
