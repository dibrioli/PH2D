//! ⭐ **O convidado da lista** (dono 05/11: *«o transform dividiu o painel. Deveria ficar dentro do
//! painel»*): o painel à frente de um encaixe convida o de trás a mostrar os cartões que este declara
//! como dele. O convidado continua dono do cartão (soltar, alfinete, sujeito, pedidos); ninguém depende
//! de ninguém. O anfitrião oferece o assento (a coluna do quadro anterior), a travessia (`panel_walk`)
//! corre o passo e o anfitrião cola a cena do passo na lista dele, antes do que pinta por cima (os
//! popovers). ⭐ O convidado é UMA ENTRADA da lista do anfitrião (dono 05/11: *«não aceita trocar de lugar
//! com os outros cartões»*): a chave [`WidgetStore::card_guest_place_over`] entra na pilha e na ordem
//! gravada como as outras; a mão que levanta o cartão dele conduz as duas pilhas, a do anfitrião decide o
//! vão, o assentar e a ordem, a do convidado pinta o voo e solta para a tela.

use super::{CardDrag, CardDrop, WidgetStore};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_vector::VectorScene;

/// A coluna que o anfitrião oferece ao convidado.
#[derive(Debug, Copy, Clone)]
pub struct GuestSeat {
    pub host: NodeId,
    pub x: f32,
    pub w: f32,
    /// A largura dos cartões do anfitrião (sem a barra de rolagem).
    pub cw: f32,
    /// O topo do corpo, antes da rolagem.
    pub body_top: f32,
    /// O `y` do lugar do convidado na lista do anfitrião, desde a origem dela.
    pub y: f32,
    /// O recorte do corpo: o do que se vê e o do que se clica.
    pub clip: Rect,
}

/// O convite e o passo. Ver o cabeçalho do módulo.
#[derive(Default)]
pub struct CardGuest {
    /// `(anfitrião, convidado)` que a última pintura do anfitrião publicou.
    invite: Option<(NodeId, NodeId)>,
    /// O convidado do passo em curso.
    pass: Option<NodeId>,
    /// `(anfitrião, altura)` do passo deste quadro.
    served: Option<(NodeId, f32)>,
    /// O convidado do passo deste quadro (já fechado).
    served_guest: Option<NodeId>,
    /// A origem em que o passo pintou (o anfitrião cola a cena no `y` de agora).
    origin: f32,
    /// A queda da mão que levantou o cartão da coluna, à espera do anfitrião.
    dropped: Option<CardDrop>,
    /// As secções que o convidado mostra na lista dele durante o passo.
    only: Vec<NodeId>,
    seat: Option<GuestSeat>,
    /// O que o passo pintou, à espera do anfitrião: os cartões e o que voa.
    scene: Option<VectorScene>,
    flight: Option<VectorScene>,
}

impl std::fmt::Debug for CardGuest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CardGuest")
            .field("invite", &self.invite)
            .field("pass", &self.pass)
            .field("served", &self.served)
            .field("seat", &self.seat)
            .finish_non_exhaustive()
    }
}

/// A chave do estado da lista curta: a pilha e as alturas da lista inteira não se misturam com as dela.
fn chave_curta(panel: NodeId) -> NodeId {
    const SAL_DA_LISTA_CURTA: u64 = 0x7C31_A05E_92D4_18BB;
    NodeId(panel.0 ^ SAL_DA_LISTA_CURTA)
}

impl WidgetStore {
    fn troca_as_listas(&mut self, panel: NodeId) {
        let k = chave_curta(panel);
        let lists = &mut self.card_lists.lists;
        let (a, b) = (lists.remove(&panel), lists.remove(&k));
        if let Some(b) = b {
            lists.insert(panel, b);
        }
        if let Some(a) = a {
            lists.insert(k, a);
        }
    }

    /// O anfitrião, na pintura dele: `guest` entra na coluna no quadro seguinte. Sem novo convite, sai.
    pub fn invite_card_guest(&mut self, host: NodeId, guest: NodeId) {
        self.card_lists.guest.invite = Some((host, guest));
    }

    /// A lista do anfitrião convidante oferece a coluna dela.
    pub(crate) fn offer_card_guest_seat(
        &mut self,
        panel: NodeId,
        (x, w, cw): (f32, f32, f32),
        body_top: f32,
        clip: Rect,
        lugar_y: Option<f32>,
    ) {
        let g = &mut self.card_lists.guest;
        if g.invite.is_some_and(|(host, _)| host == panel) {
            let y = lugar_y.or(g.seat.map(|s| s.y)).unwrap_or(0.0);
            g.seat = Some(GuestSeat {
                host: panel,
                x,
                w,
                cw,
                body_top,
                y,
                clip,
            });
        }
    }

    /// `host` já ofereceu a coluna dele?
    #[must_use]
    pub fn card_guest_seat_of(&self, host: NodeId) -> bool {
        self.card_lists.guest.seat.is_some_and(|s| s.host == host)
    }

    /// A coluna do passo.
    #[must_use]
    pub fn card_guest_seat(&self) -> Option<GuestSeat> {
        self.card_lists.guest.seat
    }

    /// A travessia, no começo: o convite do quadro anterior, e o quadro novo começa sem passo.
    pub fn take_card_guest_invite(&mut self) -> Option<(NodeId, NodeId)> {
        let g = &mut self.card_lists.guest;
        g.served = None;
        g.served_guest = None;
        g.dropped = None;
        g.scene = None;
        g.flight = None;
        g.invite.take()
    }

    /// Abre o passo de `guest` na coluna de `host`.
    pub fn begin_card_guest_pass(&mut self, host: NodeId, guest: NodeId) {
        self.troca_as_listas(guest);
        let g = &mut self.card_lists.guest;
        g.pass = Some(guest);
        g.served = Some((host, 0.0));
        g.served_guest = Some(guest);
        g.only.clear();
    }

    /// Fecha o passo: o convidado não tem rect próprio, a roda e a pressão são da lista do anfitrião. Os
    /// rects dos soltos ficam.
    pub fn end_card_guest_pass(&mut self, guest: NodeId) {
        self.card_lists.guest.pass = None;
        self.troca_as_listas(guest);
        self.panel_rects.remove(&guest);
        self.floating_panels.remove(&guest);
    }

    /// `panel` pinta a lista curta neste passo?
    #[must_use]
    pub fn in_card_guest_pass(&self, panel: NodeId) -> bool {
        self.card_lists.guest.pass == Some(panel)
    }

    /// O convidado, na pintura: quais das secções dele aparecem na lista curta.
    pub fn set_card_guest_only(&mut self, sections: &[NodeId]) {
        self.card_lists.guest.only = sections.to_vec();
    }

    /// As secções da lista curta.
    #[must_use]
    pub fn card_guest_only(&self) -> &[NodeId] {
        &self.card_lists.guest.only
    }

    /// A lista curta mediu-se.
    pub(crate) fn set_card_guest_h(&mut self, h: f32) {
        if let Some((_, antes)) = &mut self.card_lists.guest.served {
            *antes = h;
        }
    }

    /// A altura que o convidado ocupa na coluna de `host` neste quadro (`0` = sem convidado).
    #[must_use]
    pub fn card_guest_h_over(&self, host: NodeId) -> f32 {
        match self.card_lists.guest.served {
            Some((h, altura)) if h == host => altura,
            _ => 0.0,
        }
    }

    /// A chave da entrada do convidado na lista de `host` neste quadro, enquanto ele ocupa ou voa. É estável
    /// entre sessões, e a ordem gravada não se mistura com a dos cartões na lista do próprio convidado.
    pub(crate) fn card_guest_place_over(&mut self, host: NodeId) -> Option<NodeId> {
        let g = &self.card_lists.guest;
        let (dono, h) = g.served?;
        if dono != host {
            return None;
        }
        let lugar = chave_curta(g.served_guest?);
        let sem_mao = self.card_lists.drag.is_none() && g.dropped.is_none();
        let lista = self.card_lists.lists.entry(host).or_default();
        let voa = lista.stack.flight().is_some_and(|f| f.key == lugar);
        if voa && sem_mao {
            let _ = lista.stack.detach();
        }
        (h > 0.0 || (voa && !sem_mao)).then_some(lugar)
    }

    /// A mão que levou um cartão do convidado, como arrasto do lugar dele na lista de `host`.
    pub(crate) fn card_guest_hand(&mut self, host: NodeId) -> Option<CardDrag> {
        let d = self.card_lists.drag?;
        let lugar = self.card_guest_place_over(host)?;
        let leva = d.active && !d.loose && self.card_lists.guest.only.contains(&d.section);
        leva.then_some(CardDrag {
            section: lugar,
            ..d
        })
    }

    pub(crate) fn set_card_guest_drop(&mut self, d: CardDrop) {
        self.card_lists.guest.dropped = Some(d);
    }

    /// A queda da mão, como queda do lugar do convidado na lista de `host`.
    pub(crate) fn take_card_guest_drop(&mut self, host: NodeId) -> Option<CardDrop> {
        self.card_lists.guest.dropped?;
        let lugar = self.card_guest_place_over(host)?;
        let d = self.card_lists.guest.dropped.take()?;
        Some(CardDrop {
            section: lugar,
            ..d
        })
    }

    pub(crate) fn set_card_guest_origin(&mut self, origin: f32) {
        self.card_lists.guest.origin = origin;
    }

    pub(crate) fn card_guest_origin(&self) -> f32 {
        self.card_lists.guest.origin
    }

    pub(crate) fn set_card_guest_scene(&mut self, scene: VectorScene) {
        self.card_lists.guest.scene = Some(scene);
    }

    pub(crate) fn set_card_guest_flight(&mut self, scene: VectorScene) {
        self.card_lists.guest.flight = Some(scene);
    }

    /// Os cartões do convidado de `host`, uma vez por quadro.
    pub(crate) fn take_card_guest_scene(&mut self, host: NodeId) -> Option<VectorScene> {
        let g = &mut self.card_lists.guest;
        g.served
            .is_some_and(|(h, _)| h == host)
            .then(|| g.scene.take())
            .flatten()
    }

    /// O voo do convidado de `host`, uma vez por quadro.
    pub(crate) fn take_card_guest_flight(&mut self, host: NodeId) -> Option<VectorScene> {
        let g = &mut self.card_lists.guest;
        g.served
            .is_some_and(|(h, _)| h == host)
            .then(|| g.flight.take())
            .flatten()
    }
}
