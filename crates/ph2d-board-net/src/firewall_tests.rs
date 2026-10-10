//! O botão da firewall: as decisões puras (qual firewall, que comandos, o que se lembra).

use std::net::Ipv4Addr;

use crate::Subnet;
use crate::firewall::{Allowed, Change, Firewall, Outcome, Probe, command, detect, outcome};

fn net(s: &str) -> Subnet {
    s.parse().expect("sub-rede")
}

#[test]
fn a_sub_rede_e_a_do_endereco_com_a_mascara() {
    let home = Subnet::new(Ipv4Addr::new(192, 168, 15, 12), 24).expect("/24");
    assert_eq!(home.to_string(), "192.168.15.0/24");
    assert!(home.contains(Ipv4Addr::new(192, 168, 15, 200)));
    assert!(!home.contains(Ipv4Addr::new(192, 168, 16, 1)));
    assert_eq!(net("10.1.2.3/8").to_string(), "10.0.0.0/8");
    assert_eq!(net("172.20.5.9/20").to_string(), "172.20.0.0/20");
    assert_eq!(net("10.0.0.7/32").to_string(), "10.0.0.7/32");
    assert_eq!(net("10.0.0.7/0").to_string(), "0.0.0.0/0");
    assert_eq!(Subnet::new(Ipv4Addr::LOCALHOST, 33), None);
    for bad in [
        "",
        "10.0.0.1",
        "10.0.0.1/",
        "10.0.0/8",
        "x/8",
        "10.0.0.1/24; rm -rf /",
    ] {
        assert!(bad.parse::<Subnet>().is_err(), "{bad:?}");
    }
}

#[test]
fn quem_manda_ufw_depois_firewalld_depois_a_regra_a_mao() {
    let on = Some("# ufw\nENABLED=yes\nLOGLEVEL=low\n".to_owned());
    let off = Some("ENABLED=no\n".to_owned());
    let p = |ufw_conf: Option<String>, firewalld_active, other_active, pkexec| Probe {
        ufw_conf,
        firewalld_active,
        other_active,
        pkexec,
    };
    assert_eq!(detect(&p(on.clone(), true, true, true)), Firewall::Ufw);
    assert_eq!(
        detect(&p(off.clone(), true, true, true)),
        Firewall::Firewalld
    );
    assert_eq!(detect(&p(None, true, false, true)), Firewall::Firewalld);
    assert_eq!(detect(&p(off.clone(), false, true, true)), Firewall::Manual);
    assert_eq!(detect(&p(off, false, false, true)), Firewall::Off);
    assert_eq!(detect(&Probe::default()), Firewall::Off);
    // Sem quem peça a senha, não há botão: a frase com a porta.
    assert_eq!(detect(&p(on, false, false, false)), Firewall::Manual);
    assert_eq!(detect(&p(None, true, false, false)), Firewall::Manual);
    assert!(Firewall::Ufw.supported() && Firewall::Firewalld.supported());
    assert!(!Firewall::Manual.supported() && !Firewall::Off.supported());
}

#[test]
fn o_ufw_libera_so_a_sub_rede_as_portas_e_o_mdns_e_desfaz_o_mesmo() {
    let home = [net("192.168.15.0/24")];
    let allow = command(Firewall::Ufw, Change::Allow, &home).expect("ufw");
    assert_eq!(allow[..3], ["pkexec", "/bin/sh", "-c"]);
    assert_eq!(
        allow[3],
        "ufw allow from 192.168.15.0/24 to any port 47320:47329 proto tcp \
         && ufw allow from 192.168.15.0/24 to any port 5353 proto udp"
    );
    let remove = command(Firewall::Ufw, Change::Remove, &home).expect("ufw");
    assert_eq!(
        remove[3],
        "ufw delete allow from 192.168.15.0/24 to any port 47320:47329 proto tcp \
         && ufw delete allow from 192.168.15.0/24 to any port 5353 proto udp"
    );
    assert_eq!(allow[3].replace("ufw allow", "ufw delete allow"), remove[3]);
    assert!(!allow[3].contains("any port 47320 ") && !allow[3].contains(" from any "));
}

#[test]
fn o_firewalld_poe_regras_permanentes_e_recarrega() {
    let two = [net("192.168.15.0/24"), net("10.0.0.0/8")];
    let allow = command(Firewall::Firewalld, Change::Allow, &two).expect("firewalld");
    let rule = |n: &str, port: &str, proto: &str| {
        format!(
            "rule family=\"ipv4\" source address=\"{n}\" port port=\"{port}\" protocol=\"{proto}\" accept"
        )
    };
    let want: Vec<String> = [
        ("192.168.15.0/24", "47320-47329", "tcp"),
        ("192.168.15.0/24", "5353", "udp"),
        ("10.0.0.0/8", "47320-47329", "tcp"),
        ("10.0.0.0/8", "5353", "udp"),
    ]
    .iter()
    .map(|(n, port, proto)| {
        format!(
            "firewall-cmd --permanent --add-rich-rule='{}'",
            rule(n, port, proto)
        )
    })
    .chain(["firewall-cmd --reload".to_owned()])
    .collect();
    assert_eq!(allow[3], want.join(" && "));
    let remove = command(Firewall::Firewalld, Change::Remove, &two).expect("firewalld");
    assert_eq!(
        remove[3],
        allow[3].replace("--add-rich-rule", "--remove-rich-rule")
    );
}

#[test]
fn sem_firewall_suportada_ou_sem_rede_nao_ha_comando() {
    let home = [net("192.168.15.0/24")];
    for fw in [Firewall::Off, Firewall::Manual] {
        assert_eq!(command(fw, Change::Allow, &home), None, "{fw:?}");
    }
    assert_eq!(command(Firewall::Ufw, Change::Allow, &[]), None);
}

#[test]
fn o_que_se_liberou_lembra_se_e_le_se_igual() {
    let a = Allowed {
        firewall: Firewall::Firewalld,
        subnets: vec![net("192.168.15.0/24"), net("10.0.0.0/8")],
    };
    assert_eq!(a.to_text(), "firewalld\n192.168.15.0/24\n10.0.0.0/8\n");
    assert_eq!(Allowed::parse(&a.to_text()), Some(a.clone()));
    assert_eq!(Allowed::parse(""), None);
    assert_eq!(Allowed::parse("ufw\n"), None, "nada liberado");
    assert_eq!(Allowed::parse("iptables\n10.0.0.0/8\n"), None);
    assert!(a.covers(Firewall::Firewalld, &[net("10.0.0.0/8")]));
    assert!(a.covers(Firewall::Firewalld, &[]));
    assert!(
        !a.covers(Firewall::Ufw, &[net("10.0.0.0/8")]),
        "outra firewall"
    );
    assert!(
        !a.covers(Firewall::Firewalld, &[net("192.168.1.0/24")]),
        "outra rede (o portátil noutra casa)"
    );
}

#[test]
fn o_desfecho_le_o_codigo_do_pkexec() {
    assert_eq!(outcome(Some(0), "aviso\n"), Outcome::Done);
    assert_eq!(outcome(Some(126), ""), Outcome::Cancelled);
    assert_eq!(
        outcome(Some(127), "Error executing command"),
        Outcome::Cancelled
    );
    assert_eq!(
        outcome(Some(1), "WARN x\nERROR: Bad port\n\n"),
        Outcome::Failed("ERROR: Bad port".to_owned())
    );
    assert_eq!(outcome(None, ""), Outcome::Failed("?".to_owned()));
}
