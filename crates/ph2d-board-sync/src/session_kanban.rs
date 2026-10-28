//! W5 — os Kanbans que o quadro referencia viajam na MESMA sessão (ADR-0195 §3): o anfitrião e o
//! convidado só guardam as mensagens [`Msg::Kanban`]; quem as lê e responde é a `ph2d-kanban-sync`.

use super::{ConnId, Guest, Host, Msg, PeerId};

impl Host {
    /// O que chegou para o Kanban: `(ligação, quem, bytes)`. A resposta vai como `Msg::Kanban`.
    pub fn take_kanban(&mut self) -> Vec<(ConnId, PeerId, Vec<u8>)> {
        std::mem::take(&mut self.kanban_in)
    }
}

impl Guest {
    /// O que chegou do anfitrião para o Kanban.
    pub fn take_kanban(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.kanban_in)
    }

    /// Embrulha bytes do Kanban para o anfitrião.
    #[must_use]
    pub fn kanban(bytes: Vec<u8>) -> Msg {
        Msg::Kanban(bytes)
    }
}

#[cfg(test)]
mod tests {
    use ph2d_board_model::BoardSet;

    use super::*;

    /// Os bytes do Kanban atravessam a sala nos dois sentidos, intactos, e só de quem entrou.
    #[test]
    fn kanban_bytes_ride_the_board_session_both_ways() {
        let mut set = BoardSet::default();
        let id = set.create("Quadro".into());
        let mut hb = set.get_mut(id).unwrap().clone();
        let mut host = Host::open(1, "Ana".into(), &mut hb, 7);
        let stranger = Msg::Kanban(vec![9]).encode();
        let _ = host.frame(&mut hb, 5, &stranger, 0);
        assert!(host.take_kanban().is_empty(), "quem não entrou não fala");

        let mut g = super::super::Guest::new(2, "Bia".into(), 8, 99);
        let (out, _) = host.frame(&mut hb, 5, &g.hello().encode(), 0);
        let welcome = out
            .iter()
            .find(|(_, m)| matches!(m, Msg::Welcome { .. }))
            .unwrap();
        let _ = g.frame(None, &welcome.1.encode(), 0);
        let _ = host.frame(&mut hb, 5, &Guest::kanban(vec![1, 2, 3]).encode(), 1);
        assert_eq!(host.take_kanban(), vec![(5, 2, vec![1, 2, 3])]);
        assert!(host.take_kanban().is_empty());
        let _ = g.frame(None, &Msg::Kanban(vec![4]).encode(), 2);
        assert_eq!(g.take_kanban(), vec![vec![4]]);
    }
}
