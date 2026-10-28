//! J7 — a família Kanban e a sala do QUADRO falam pela porta (`ph2d_kanban_sync::port`), duas
//! famílias que não se conhecem: eu digo onde estou (o Kanban à frente, o cartão aberto); a sala diz
//! quem está nos Kanbans dela — os avatares, o anel no cartão, e esmaecidos enquanto ela volta; o
//! convite de Kanban colado no «Join» do Quadro abre a sala aqui, e o do Quadro colado aqui vai lá.

use crate::Mesa;
use ph2d_board_net::{Invite, InviteKey, InviteKind};
use ph2d_kanban_model::KanbanId;
use ph2d_kanban_sync::{KanbanPresence, Viewers, port};
use std::collections::BTreeMap;

#[test]
fn j7_a_porta_leva_onde_estou_traz_quem_esta_na_sala_do_quadro_e_os_convites() {
    port::reset_for_tests();
    let (mut m, id) = Mesa::cena1();
    m.docs().share_local_for_tests();
    m.quadro();
    assert_eq!(port::here().map(|h| (h.0, h.1)), Some((id, None)));
    let card = *m.kanban(id).cards.keys().next().expect("um cartão");
    m.docs().open_back(id.0, card);
    m.quadro();
    assert_eq!(
        port::here().map(|h| (h.0, h.1)),
        Some((id, Some(card))),
        "o verso aberto"
    );

    // A sala do Quadro: a Bia (par 7) tem o mesmo cartão aberto no Kanban (sessão W = o meu `id`).
    let wire = KanbanId((1 << 63) | 5);
    let mut viewers = Viewers::default();
    viewers.heard(
        1,
        KanbanPresence {
            peer: 7,
            kanban: Some(wire),
            card: Some(card),
            cursor: None,
        },
    );
    let room = |reconnecting| port::Room {
        viewers: viewers.clone(),
        names: BTreeMap::from([(7, "Bia".to_owned())]),
        pairs: vec![(wire, id)],
        now: 1,
        reconnecting,
    };
    port::set_room(Some(room(false)));
    m.quadro();
    assert!(
        m.docs().probe_share(id.0).2.contains(&"Bia".to_owned()),
        "o avatar"
    );
    assert_eq!(
        m.docs().probe_seen(id.0).0,
        vec![(card, vec!["Bia".to_owned()])],
        "o anel"
    );
    assert!(!m.docs().probe_reconnecting(id.0));
    port::set_room(Some(room(true)));
    m.quadro();
    assert!(
        m.docs().probe_reconnecting(id.0),
        "a voltar: esmaecidos e o selo"
    );
    port::set_room(None);
    m.quadro();
    assert!(
        m.docs().probe_share(id.0).2.iter().all(|p| p != "Bia"),
        "a sala acabou"
    );

    // Os convites.
    let invite = Invite {
        key: InviteKey::random(),
        addrs: vec![],
    };
    port::post_kanban_link(invite.link_as(InviteKind::Kanban));
    m.quadro();
    assert_eq!(
        m.docs().probe_share(id.0).0,
        "Joining",
        "a sala do Kanban abre-se"
    );
    m.docs().share_shutdown();
    m.docs().join_shared(&invite.link());
    assert_eq!(m.docs().probe_share(id.0).0, "Off");
    assert_eq!(
        port::take_board_link(),
        Some(invite.link()),
        "vai ao Quadro"
    );
}
