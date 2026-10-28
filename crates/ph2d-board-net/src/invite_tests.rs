use super::*;

fn sample() -> Invite {
    Invite {
        key: InviteKey::from_bytes(*b"0123456789abcdef"),
        addrs: vec![
            "203.0.113.5:47321".parse().unwrap(),
            "[2001:db8::1]:47321".parse().unwrap(),
        ],
    }
}

#[test]
fn o_link_volta_ao_mesmo_convite() {
    let inv = sample();
    let link = inv.link();
    assert_eq!(
        link,
        "ph2d-board://v1/203.0.113.5:47321,[2001:db8::1]:47321#MDEyMzQ1Njc4OWFiY2RlZg"
    );
    assert_eq!(Invite::parse(&link), Ok(inv));
}

#[test]
fn colado_partido_em_linhas_ainda_e_o_convite() {
    let link = sample().link();
    let (a, b) = link.split_at(30);
    assert_eq!(Invite::parse(&format!("  {a}\n {b} \n")), Ok(sample()));
}

#[test]
fn o_esquema_em_maiusculas_serve_mas_a_chave_nao_muda() {
    let link = sample().link().replacen("ph2d-board", "PH2D-BOARD", 1);
    assert_eq!(Invite::parse(&link), Ok(sample()));
    let link = sample().link();
    let (head, key) = link.split_once('#').unwrap();
    let upper_key = format!("{head}#{}", key.to_ascii_uppercase());
    assert_ne!(
        Invite::parse(&upper_key).map(|i| i.key),
        Ok(sample().key),
        "a chave distingue maiúsculas"
    );
}

#[test]
fn sem_enderecos_e_valido() {
    let inv = Invite {
        key: InviteKey::random(),
        addrs: vec![],
    };
    assert_eq!(Invite::parse(&inv.link()), Ok(inv));
}

#[test]
fn o_que_nao_e_convite_diz_porque() {
    let ok = sample().link();
    assert_eq!(Invite::parse("ABC-DEF"), Err(InviteError::NotALink));
    assert_eq!(
        Invite::parse(ok.split('#').next().unwrap()),
        Err(InviteError::NotALink)
    );
    assert_eq!(
        Invite::parse(&ok.replace("/v1/", "/v2/")),
        Err(InviteError::OtherVersion)
    );
    // Uma letra a menos na chave: 127 bits não são a chave.
    assert_eq!(Invite::parse(&ok[..ok.len() - 1]), Err(InviteError::BadKey));
    assert_eq!(
        Invite::parse(&ok.replace("203.0.113.5", "203.0.113")),
        Err(InviteError::BadAddr)
    );
    let many = vec!["10.0.0.1:1"; MAX_ADDRS + 1].join(",");
    assert_eq!(
        Invite::parse(&format!("ph2d-board://v1/{many}#MDEyMzQ1Njc4OWFiY2RlZg")),
        Err(InviteError::BadAddr)
    );
}

#[test]
fn a_chave_nao_aparece_no_debug() {
    let inv = sample();
    assert!(!format!("{inv:?}").contains("MDEy"));
    assert!(!format!("{:?}", inv.key).contains("0123"));
}

/// J4 — o link de um Kanban tem esquema próprio; cada «Join» reconhece o da OUTRA sala e di-lo
/// (nunca o lê como um link estragado nem entra na sala errada).
#[test]
fn o_link_do_kanban_tem_esquema_proprio_e_cada_sala_reconhece_o_da_outra() {
    let k = sample().link_as(InviteKind::Kanban);
    assert!(k.starts_with(KANBAN_SCHEME), "{k}");
    assert_eq!(Invite::parse_as(&k, InviteKind::Kanban), Ok(sample()));
    assert_eq!(
        Invite::parse(&k),
        Err(InviteError::OtherKind(InviteKind::Kanban))
    );
    assert_eq!(
        Invite::parse_as(&sample().link(), InviteKind::Kanban),
        Err(InviteError::OtherKind(InviteKind::Board))
    );
    assert_eq!(
        Invite::parse_any(&k.replacen("ph2d-kanban", "PH2D-KANBAN", 1)),
        Ok((InviteKind::Kanban, sample()))
    );
    assert_eq!(
        Invite::parse_any("ph2d-other://v1/#x"),
        Err(InviteError::NotALink)
    );
}
