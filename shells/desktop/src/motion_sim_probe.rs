//! ⭐⭐ **A AUDITORIA DO GRUPO DA SIMULAÇÃO** (ciclo 5, passo 2 — doc 103 §5).
//!
//! *«Deixar a física decidir»*: até aqui o artista disse **onde** cada coisa está, **como** ela
//! anda e **quem** é afectado. Este grupo entrega a última decisão a uma lei — e o que ele pede
//! em troca é um **relógio**.
//!
//! ⚠️ **O grupo é meio LISTA e meio FAMÍLIA**, e a metade família é derivada do registry: uma
//! `force.*` que nasça amanhã entra na auditoria sozinha. *Uma lista escrita à mão envelhece em
//! silêncio no dia em que a família cresce.*
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture audit_the_sim_group
//! ```

/// A metade escrita: os nós do ciclo que **não** são uma família.
const NOMEADOS: [&str; 6] = [
    "sim.zone",
    "sim.spawn",
    "sim.step",
    "sim.lifetime",
    "sim.collide",
    "motion.integrate",
];

/// O grupo inteiro — os nomeados mais **toda** a família `force.*`, pedida ao registry.
pub(crate) fn grupo() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = NOMEADOS.to_vec();
    v.extend(crate::motion_ciclo_probe::familia("force."));
    v
}

#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn audit_the_sim_group() {
    crate::motion_ciclo_probe::retrato(&grupo());
}

/// **OS PARAMS DE CADA NÓ DO GRUPO** — o que a auditoria compara contra as referências.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_each_sim_node_offers() {
    crate::motion_ciclo_probe::params_de(&grupo());
}

/// **O QUE O CARTÃO PINTA**, com o rótulo da tela.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_the_sim_card_shows() {
    crate::motion_ciclo_probe::cartao(&grupo());
}

/// **OS NOMES dos cartões** — o que o tutorial terá de escrever.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn the_sim_card_names() {
    crate::motion_ciclo_probe::nomes(&grupo());
}

/// **O VOCABULÁRIO do grupo** — dois nós que guardam a mesma pergunta chamam-lhe o mesmo nome?
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn the_sim_vocabulary() {
    crate::motion_ciclo_probe::vocabulario(&grupo());
}

/// ⚠️ **A FAMÍLIA `force.*` NÃO ESTÁ VAZIA, e o grupo cresce com ela.**
///
/// ⛔ Sem este piso, um `familia("force.")` que devolvesse nada (um prefixo mal escrito, um
/// registo que mudou de nome) deixaria as cinco sondas acima a auditar **seis** nós em vez de
/// doze — e todas passariam, caladas.
#[test]
fn the_force_family_is_derived_and_not_empty() {
    let forcas = crate::motion_ciclo_probe::familia("force.");
    assert!(
        forcas.len() >= 6,
        "so' {} no(s) `force.*` -- o prefixo ou o registo mudaram: {forcas:?}",
        forcas.len()
    );
    let g = grupo();
    assert_eq!(
        g.len(),
        NOMEADOS.len() + forcas.len(),
        "o grupo tem de ser os nomeados MAIS a familia inteira"
    );
}

/// ⭐⭐⭐ **QUEM DESTE GRUPO OCUPA LUGAR NO MUNDO E NÃO TEM ALÇA?**
///
/// ⚠️ **É a mesma pergunta que abriu o ciclo 4**, e ela vale por grupo: um nó com `center_x` e
/// `center_y` é uma coisa **posta** algures, e pô-la com dois sliders é caçar a posição. A régua
/// é derivada — quem declara o vocabulário espacial contra quem o
/// [`crate::field_gizmo::spec_for`] conhece.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins -- --ignored --nocapture which_sim_nodes_have_a_place
/// ```
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn which_sim_nodes_have_a_place() {
    use ph2d_nodegraph::cook::OpResolver;
    let m = crate::motion_state::MotionState::new();
    eprintln!("\n  nó                        | centro | raio | ângulo | alça de canvas");
    eprintln!("  --------------------------|--------|------|--------|---------------");
    for nome in grupo() {
        let tid = ph2d_nodegraph::node::NodeTypeId::of(nome);
        let Some(op) = m.registry.resolve(tid) else {
            continue;
        };
        let tem = |p: &str| op.manifest().params.iter().any(|s| s.name == p);
        let centro = tem("center_x") && tem("center_y");
        if !centro {
            continue;
        }
        eprintln!(
            "  {nome:<26} |  sim   | {:^4} | {:^6} | {}",
            if tem("radius") { "sim" } else { "—" },
            if tem("angle") || tem("rotation") {
                "sim"
            } else {
                "—"
            },
            if crate::field_gizmo::spec_for(tid, &|_| 0.0).is_some() {
                "sim"
            } else {
                "⛔ NAO"
            }
        );
    }
    eprintln!();
}
