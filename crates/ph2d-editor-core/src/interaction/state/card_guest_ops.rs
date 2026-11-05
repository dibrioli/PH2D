//! ⭐ **O convidado da lista** (dono 05/11: *«faça o transform de vector no modo draw ser o mesmo do
//! modo object»*): o painel à frente de um encaixe convida o de trás a pintar, por cima dele, os
//! cartões que este declara. O convidado continua dono do cartão (soltar, alfinete, sujeito, pedidos);
//! ninguém depende de ninguém: a travessia (`panel_walk`) corre o passo, e o anfitrião só desce.

use super::WidgetStore;
use ph2d_a11y::NodeId;

/// O convite e o passo. Ver o cabeçalho do módulo.
#[derive(Debug, Default)]
pub struct CardGuest {
    /// `(anfitrião, convidado)` que a última pintura do anfitrião publicou.
    invite: Option<(NodeId, NodeId)>,
    /// O convidado do passo em curso.
    pass: Option<NodeId>,
    /// `(anfitrião, altura)` do passo deste quadro.
    served: Option<(NodeId, f32)>,
    /// O anfitrião que está de pé neste quadro: `(convidado, anfitrião)`.
    host_of: Option<(NodeId, NodeId)>,
    /// As secções que o convidado mostra na lista dele durante o passo.
    only: Vec<NodeId>,
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

    /// O anfitrião, na pintura dele: `guest` entra por cima no quadro seguinte. Sem novo convite, sai.
    pub fn invite_card_guest(&mut self, host: NodeId, guest: NodeId) {
        self.card_lists.guest.invite = Some((host, guest));
    }

    /// A travessia, no começo: o convite do quadro anterior, e o quadro novo começa sem passo.
    pub fn take_card_guest_invite(&mut self) -> Option<(NodeId, NodeId)> {
        let g = &mut self.card_lists.guest;
        g.served = None;
        g.host_of = None;
        g.invite.take()
    }

    /// Abre o passo de `guest` por cima de `host`.
    pub fn begin_card_guest_pass(&mut self, host: NodeId, guest: NodeId) {
        self.troca_as_listas(guest);
        let g = &mut self.card_lists.guest;
        g.pass = Some(guest);
        g.host_of = Some((guest, host));
        g.served = Some((host, 0.0));
        g.only.clear();
    }

    /// Fecha o passo: a lista curta guardou o rect dela, que recebe os cliques; o do encaixe não fica.
    pub fn end_card_guest_pass(&mut self, guest: NodeId) {
        self.card_lists.guest.pass = None;
        self.troca_as_listas(guest);
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

    /// A lista curta sem cartão não recebe clique; os rects dos soltos ficam.
    pub(crate) fn drop_card_guest_rect(&mut self, panel: NodeId) {
        self.panel_rects.remove(&panel);
        self.floating_panels.remove(&panel);
    }

    /// O que `host` deixa livre no topo da coluna neste quadro (`0` = sem convidado).
    #[must_use]
    pub fn card_guest_h_over(&self, host: NodeId) -> f32 {
        match self.card_lists.guest.served {
            Some((h, altura)) if h == host => altura,
            _ => 0.0,
        }
    }

    /// Quem responde pela coluna sob a lista curta: a roda e a pressão são do anfitrião (ela não rola,
    /// e trazer a aba de trás à frente tiraria a coluna do Draw à mão que a usa).
    #[must_use]
    pub fn column_owner(&self, panel: NodeId) -> NodeId {
        match self.card_lists.guest.host_of {
            Some((guest, host)) if guest == panel => host,
            _ => panel,
        }
    }
}
