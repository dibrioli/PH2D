//! **Arch-gate: todo componente de geometria DERIVADA é pulado pelo `settle_origins`.**
//!
//! O `settle` assenta o pivô de um path no centro dele. Para geometria **autorada** isso é certo e
//! é o que o ADR-0112 pede. Para geometria **derivada** — a que um `*_live::recook` reescreve em
//! MUNDO a cada frame — é um desastre silencioso: assentar soma a geometria de mundo com um
//! `Transform` novo e a forma **desloca-se da posição que ela deveria descrever**.
//!
//! # Por que este gate lê o FONTE
//!
//! O `filter` do `settle_origins` é uma **lista que enumera os seus leitores**, e uma lista assim
//! apodrece: quem acrescentar o 5º componente derivado e esquecer esta linha não vê erro de
//! compilação — vê a forma nova a saltar, um frame depois, num sítio que não é o dela.
//! [[feedback_a_condition_that_enumerates_its_readers_rots]]
//!
//! O gate não sabe testar comportamento aqui (precisaria de uma cena por componente); o que ele
//! sabe é **cobrar a correspondência**: todo componente que um `*_live.rs` re-coza tem de aparecer
//! no `filter`. É um contador de símbolos, e por isso vale ZERO como auditoria — mas o que ele
//! guarda é exatamente uma omissão mecânica, que é o modo como esta lista falha.

use std::fs;

/// Os componentes que o `settle_origins` tem de pular, e **por que cada um** — as razões são duas,
/// e confundi-las foi o 1º erro deste gate:
///
/// - `VecConnector` / `VecBlend` / `VecMorph`: a geometria deles é **MUNDO**, reescrita por frame,
///   e eles vivem na **identidade**. Assentar somaria geometria + `Transform` e os deslocaria.
/// - `VecShape`: a geometria é derivada dos parâmetros, mas ele **tem pose própria** — está aqui
///   por outra razão (a origem fica onde a forma foi criada; "Set Center" a move).
/// - `VecEnvelope` (ADR-0123): geometria de MUNDO, a fonte autorada deformada pela gaiola,
///   reescrita por `envelope_live` a cada frame; vive na identidade.
const DERIVED: &[&str] = &[
    "VecShape",
    "VecConnector",
    "VecBlend",
    "VecMorph",
    "VecEnvelope",
];

/// A assinatura de "a minha geometria é MUNDO": o host **força a identidade** na entidade.
///
/// É o detector do gate irmão, e ele não é o nome do arquivo. A 1ª versão varria `*_live.rs` e
/// exigia que todo host estivesse em [`DERIVED`] — e gritou lobo no `flip_live.rs`, que é "o alvo
/// vivo" do painel do Flip e não tem uma linha de geometria vetorial. *Live* ali é outra palavra.
/// Um gate com falso positivo é um gate que alguém desliga:
/// [[reference_topic_oracle_discipline]]
///
/// Forçar a identidade, ao contrário, é exatamente a propriedade em causa: um host que a força
/// está a dizer *"a minha pose não significa nada, a geometria já está em mundo"* — e é essa a
/// condição sob a qual assentar o pivô destrói a forma.
const WORLD_GEOMETRY_MARK: &str = "Transform::IDENTITY;";

#[test]
fn settle_origins_skips_every_component_whose_geometry_is_derived() {
    let src = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/vec_transform.rs"))
        .expect("vec_transform.rs");
    let settle = src
        .split_once("fn settle_origins")
        .expect("settle_origins existe")
        .1;
    // Só o corpo da função — o `filter` mora nas primeiras linhas dela.
    let body = &settle[..settle.len().min(3000)];

    let missing: Vec<&str> = DERIVED
        .iter()
        .copied()
        .filter(|c| !body.contains(&format!("get::<ph2d_ecs::{c}>(e).is_none()")))
        .collect();

    assert!(
        missing.is_empty(),
        "o `settle_origins` NÃO pula {missing:?} — a geometria desses componentes é reescrita em \
         MUNDO a cada frame, e assentar o pivô soma geometria + Transform: a forma sai deslocada \
         da posição que ela descreve. Acrescente `&& sim.world().get::<ph2d_ecs::<Comp>>(e)\
         .is_none()` ao filter."
    );
}

/// **A lista deste gate não pode ficar para trás da realidade.**
///
/// O gate acima só vale enquanto [`DERIVED`] descrever o mundo: quem acrescentar um host de
/// geometria de MUNDO e não o listar aqui deixa o gate acima verde, a proteger uma lista
/// incompleta — uma barreira que se auto-desliga.
///
/// Este irmão vai ao contrário: varre os hosts que **de facto** escrevem geometria de mundo (os
/// que forçam a identidade — [`WORLD_GEOMETRY_MARK`]) e exige que o componente de cada um esteja
/// em [`DERIVED`] **e** no filter.
#[test]
fn every_host_that_writes_world_geometry_is_in_the_list() {
    let dir = fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).expect("src/");
    let hosts: Vec<(String, String)> = dir
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with(".rs"))
        .filter_map(|n| {
            let src = fs::read_to_string(format!("{}/src/{n}", env!("CARGO_MANIFEST_DIR"))).ok()?;
            src.contains(WORLD_GEOMETRY_MARK).then_some((n, src))
        })
        .collect();
    assert!(
        hosts.len() >= 3,
        "só {} hosts de geometria de mundo — o conector, o blend e o morph existem, então o \
         detector cegou (alguém mudou a forma de forçar a identidade?)",
        hosts.len()
    );

    for (host, src) in &hosts {
        let governs = DERIVED
            .iter()
            .any(|c| src.contains(&format!("<{c}>(")) || src.contains(&format!("{c},")));
        assert!(
            governs,
            "o host `{host}` força a identidade — ou seja, escreve geometria em MUNDO —, mas o \
             componente dele não está em DERIVED. Acrescente-o a DERIVED **e** ao filter do \
             `settle_origins`, senão o pivô será assentado e a forma sai deslocada."
        );
    }
}
