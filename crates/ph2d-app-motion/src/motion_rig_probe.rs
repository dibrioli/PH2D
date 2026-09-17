//! ⭐⭐ **A AUDITORIA DO GRUPO DO RIG E DOS CORPOS MOLES** (ciclo 9, passo 2 — doc 103 §5, doc 114).
//!
//! *«Coisas que se seguram»*: os oito ciclos anteriores puseram coisas na tela, fizeram-nas andar,
//! dobraram-nas, escolheram quem é afectado, entregaram-nas a uma lei, deram-lhes um cérebro, uma
//! cara, e disseram de onde vêm. Este grupo responde à pergunta que sobra: **o que mantém as partes
//! de uma coisa PRESAS umas às outras** — um osso ao pai, uma pele ao osso, um nó da corda ao
//! seguinte, uma partícula do bando aos vizinhos.
//!
//! ⚠️ **O grupo tem DUAS METADES em categorias diferentes da paleta**, e isso é o grupo e não um
//! acidente de arrumação: cinco nós **produzem** a coisa que se segura (categoria `Source`) e cinco
//! **agem** sobre ela (categoria `Transform`).
//!
//! As sondas de auditoria são `#[ignore]` e correm-se à mão, pelo filtro `audit_the_rig_group`.

use ph2d_node_registry::NodeUiCategory;

/// ⚠️ **Os nós da categoria `Source` que têm dono NOUTRO ciclo** — é a subtracção que torna a
/// metade dos corpos moles **derivada** em vez de escrita à mão.
///
/// ⛔ Sem isto, a metade mole seria uma lista de quatro nomes, e *uma lista escrita à mão envelhece
/// em silêncio no dia em que um nó nasce* — que é exactamente a doença que o
/// [`crate::motion_ciclo_probe::familia`] existe para evitar do outro lado. Com a subtracção, um nó
/// `Source` NOVO que ninguém reclame **cai neste grupo**, obrigando alguém a dar-lhe dono.
///
/// ⚠️ A lista é **censada** por [`the_owners_elsewhere_are_still_sources`]: um nome que deixe de ser
/// `Source` sai daqui, senão a subtracção passa a tirar o que já não está lá.
const COM_DONO_NOUTRO_CICLO: &[(&str, &str)] = &[
    // Ciclo 1 — ARRANJO: pôr muitos objectos na tela.
    ("motion.grid", "ciclo 1 (arranjo)"),
    ("motion.scatter", "ciclo 1 (arranjo)"),
    ("motion.distribute_radial", "ciclo 1 (arranjo)"),
    ("motion.distribute_curve", "ciclo 1 (arranjo)"),
    ("motion.fibonacci", "ciclo 1 (arranjo)"),
    ("motion.lattice", "ciclo 1 (arranjo)"),
    ("motion.voronoi", "ciclo 1 (arranjo)"),
    // Ciclo 5 — SIMULAÇÃO.
    ("sim.spawn", "ciclo 5 (simulacao)"),
    // Ciclo 6 — VALOR & PULSO.
    ("value.pattern", "ciclo 6 (valor e pulso)"),
    // Ciclo 8 — FONTES & DADOS (o emissor; as `source.*` saem pelo prefixo).
    ("motion.emitter", "ciclo 8 (fontes e dados)"),
];

/// Todo tipo oferecido na categoria `Source` da paleta — a população de que a metade mole é o resto.
fn fontes_oferecidas() -> Vec<&'static str> {
    let m = crate::motion_state::MotionState::new();
    m.registry
        .manifests()
        .filter(|man| !m.registry.is_fixture(man.id))
        .filter(|man| {
            m.registry
                .ui_manifest(man.id)
                .is_some_and(|u| u.category == NodeUiCategory::Source)
        })
        .map(|man| man.name)
        .collect()
}

/// A metade que **PRODUZ** — categoria `Source`, menos as `source.*` e menos quem tem dono noutro
/// ciclo. Em 2026-09-17: `motion.boids` · `motion.soft_body` · `motion.verlet_rope` ·
/// `motion.wave` · `rig.skeleton`.
pub fn metade_que_produz() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = fontes_oferecidas()
        .into_iter()
        .filter(|n| !n.starts_with("source."))
        .filter(|n| !COM_DONO_NOUTRO_CICLO.iter().any(|(d, _)| d == n))
        .collect();
    v.sort_unstable();
    v
}

/// O grupo inteiro: a família `rig.*` (pelo registry, nunca à mão) unida à metade que produz.
///
/// ⚠️ `rig.skeleton` está nos dois lados — ele é o `Source` do rig —, e a união dedupe-o.
pub fn grupo() -> Vec<&'static str> {
    let mut v = crate::motion_ciclo_probe::familia("rig.");
    for n in metade_que_produz() {
        if !v.contains(&n) {
            v.push(n);
        }
    }
    v.sort_unstable();
    v
}

/// A metade que **AGE** — o grupo menos a metade que produz.
pub fn metade_que_age() -> Vec<&'static str> {
    let produz = metade_que_produz();
    grupo()
        .into_iter()
        .filter(|n| !produz.contains(n))
        .collect()
}

#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn audit_the_rig_group() {
    crate::motion_ciclo_probe::retrato(&grupo());
}

/// **OS PARAMS DE CADA NÓ** — o que a auditoria compara contra Rive · Spine · Blender · RubberHose.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_each_rig_node_offers() {
    crate::motion_ciclo_probe::params_de(&grupo());
}

/// **O QUE O CARTÃO PINTA**, com o rótulo da tela.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn what_the_rig_card_shows() {
    crate::motion_ciclo_probe::cartao(&grupo());
}

/// **OS NOMES dos cartões** — o que o tutorial terá de escrever.
#[test]
#[ignore = "sonda de auditoria — corra à mão"]
fn the_rig_card_names() {
    crate::motion_ciclo_probe::nomes(&grupo());
}

/// ⚠️ **O GRUPO ESTÁ VIVO** — piso de população e **as duas metades**.
///
/// ⛔ Sem o piso, uma categoria renomeada deixava as sondas acima a auditar **zero** nós, e todas
/// passariam caladas. E sem as metades asseguradas **em separado**, um filtro que só apanhasse os
/// `rig.*` leria `6` e outro que só apanhasse os moles leria `5` — e nenhum dos dois é o grupo.
#[test]
fn the_rig_group_is_derived_and_not_empty() {
    let g = grupo();
    assert!(
        g.len() >= 10,
        "so' {} no(s) no grupo do ciclo 9 (10 em 2026-09-17): {g:?}",
        g.len()
    );
    let produz = metade_que_produz();
    let age = metade_que_age();
    assert!(
        produz.len() >= 5,
        "a metade que PRODUZ encolheu (5 em 2026-09-17): {produz:?}"
    );
    assert!(
        age.len() >= 5,
        "a metade que AGE encolheu (5 em 2026-09-17): {age:?}"
    );
    assert!(
        produz.contains(&"rig.skeleton"),
        "o esqueleto e' o `Source` do rig e tem de estar na metade que produz: {produz:?}"
    );
    assert!(
        age.iter().all(|n| n.starts_with("rig.")),
        "a metade que AGE e' toda `rig.*` hoje; se isso mudou, o doc 114 §1 tem de o dizer: {age:?}"
    );
}

/// ⚠️ **O CENSO DE OBSOLESCÊNCIA da subtracção** — *uma catraca sem censo vira licença*
/// (`CLAUDE.md` §5.0).
///
/// ⛔ Um nome em [`COM_DONO_NOUTRO_CICLO`] que deixe de ser um `Source` oferecido passa a subtrair
/// **nada**, e a lista lê-se como se ainda estivesse a proteger o grupo. As duas leituras — *«o
/// dono mudou de categoria»* e *«o nó foi apagado»* — dão o mesmo byte aqui, e as duas exigem que
/// alguém volte a esta lista.
#[test]
fn the_owners_elsewhere_are_still_sources() {
    let fontes = fontes_oferecidas();
    assert!(
        fontes.len() >= 21,
        "piso da categoria Source (21 em 2026-09-17): {}",
        fontes.len()
    );
    for (dono, ciclo) in COM_DONO_NOUTRO_CICLO {
        assert!(
            fontes.contains(dono),
            "`{dono}` ja' nao e' um `Source` oferecido — a subtracao do {ciclo} deixou de subtrair \
             alguma coisa, e esta lista deixou de descrever o grupo (doc 114 §1)"
        );
    }
}
