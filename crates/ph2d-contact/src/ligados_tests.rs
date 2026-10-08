//! O filtro dos pares que um CORPO segura ([`crate::Ligados`], doc 124 §6).

use crate::{Colisor, Pecas, Saida};

fn separa(p: &mut [[f32; 2]], corpo: Option<(&[[f32; 2]], &[f32])>) {
    let c = vec![Some(Colisor::disco(0.28)); p.len()];
    let w = vec![1.0; p.len()];
    let i = vec![0.0; p.len()];
    let mut giro = vec![0.0; p.len()];
    let pecas = Pecas::novas(&c, &w, &i);
    let pecas = match corpo {
        Some((rest, body)) => pecas.com_ligados(rest, body),
        None => pecas,
    };
    crate::separate(p, &mut Saida { giro: &mut giro }, &pecas, 8);
}

/// ⭐ **O par que o repouso já sobrepõe não colide; os outros sim.** Do MESMO corpo e a `0,3`
/// (`< 0,56`) no repouso: fica onde está. O mesmo par longe no repouso, de corpos diferentes, ou
/// sem corpo (`0`): separa-se até ao diâmetro. E sem `Ligados` o motor é o de sempre.
#[test]
fn a_pair_the_body_holds_is_left_alone() {
    let par = [[0.0, 0.0], [0.3, 0.0]];
    let mut p = par;
    separa(&mut p, Some((&[[0.0, 0.0], [0.3, 0.0]], &[1.0, 1.0])));
    assert_eq!(p, par, "o par ligado nao se mexe");
    let mut sem = par;
    separa(&mut sem, None);
    for (o_que, rest, body) in [
        ("longe no repouso", [[0.0, 0.0], [0.6, 0.0]], [1.0, 1.0]),
        ("corpos diferentes", [[0.0, 0.0], [0.3, 0.0]], [1.0, 2.0]),
        ("sem corpo", [[0.0, 0.0], [0.3, 0.0]], [0.0, 0.0]),
    ] {
        let mut q = par;
        separa(&mut q, Some((&rest, &body)));
        assert_eq!(q, sem, "{o_que}: colide como sem o filtro");
    }
    let d = (sem[1][0] - sem[0][0]).hypot(sem[1][1] - sem[0][1]);
    assert!(
        (d - 0.56).abs() < 1e-3,
        "CONTROLO: sem o filtro o par vai ao diametro ({d})"
    );
}
