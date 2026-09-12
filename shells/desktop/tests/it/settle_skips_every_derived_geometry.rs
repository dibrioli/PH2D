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
/// - `VecEnvelope` (ADR-0129): geometria de MUNDO, a fonte autorada deformada pela gaiola,
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
    let src = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/ph2d-vec-entities/src/transform.rs"
    ))
    .expect("transform.rs");
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
    // ⛔⛔ **DUAS ÁRVORES desde a W2 Fase C, e o PISO foi quem o descobriu.** O `connector_live`, o
    // `blend_live` e o `morph_live` — os três hosts que a mensagem abaixo nomeia — mudaram-se para
    // `ph2d-app-vec`, e este censo passou a ler **1**. ⭐ Sem o piso ele leria uma lista quase vazia
    // e `missing.is_empty()` seria trivialmente verdadeiro: *o censo que varre por directório fica
    // verde a varrer NADA* (HOWTO §2.7), e é precisamente contra isto que o piso existe.
    // ⚠️ **Ele fica MAIS FORTE do que era antes da mudança**, porque agora nomeia as duas árvores.
    const ARVORES: [&str; 2] = [
        concat!(env!("CARGO_MANIFEST_DIR"), "/src"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/ph2d-app-vec/src"),
    ];
    let hosts: Vec<(String, String)> = ARVORES
        .iter()
        .flat_map(|raiz| {
            fs::read_dir(raiz)
                .unwrap_or_else(|e| panic!("{raiz} nao foi lido: {e}"))
                .filter_map(Result::ok)
                .filter_map(move |e| {
                    let n = e.file_name().into_string().ok()?;
                    Some((n, (*raiz).to_string()))
                })
        })
        .filter(|(n, _)| n.ends_with(".rs"))
        // `vec_expand.rs` também força a identidade — no sentido OPOSTO ao que este gate
        // vigia: o RETUNE do Offset devolve a entidade à identidade exatamente PARA que o
        // `settle_origins` a re-assente neste frame (geometria de mundo re-inserida sob
        // pose já assentada dobraria a pose — `9c0446df`). E o skip do preview VIVO dele
        // não é por componente: é pela lista `drawing`, cobrado pelo gate irmão
        // `the_live_offset_preview_is_a_gesture_to_the_settle`.
        .filter(|(n, _)| n != "vec_expand.rs" && n != "expand.rs")
        // ⚠️ **Os módulos de teste irmãos não são hosts.** O detector procura um LITERAL
        // (`Transform::IDENTITY;`), e uma fixture que monte um mundo com ele é indistinguível de
        // um produtor de geometria de mundo — mas ela nunca corre no produto, então exigir-lhe um
        // componente em `DERIVED` seria pedir que um teste declarasse uma regra de renderização.
        // Excluir aqui não cega o gate: um host de verdade mora em código de produto.
        .filter(|(n, _)| !n.ends_with("_tests.rs"))
        .filter_map(|(n, raiz)| {
            let src = fs::read_to_string(format!("{raiz}/{n}")).ok()?;
            src.contains(WORLD_GEOMETRY_MARK).then_some((n, src))
        })
        .collect();
    assert!(
        hosts.len() >= 3,
        "só {} hosts de geometria de mundo ({:?}) — o conector, o blend e o morph existem nas \
         duas árvores varridas, então o detector cegou (alguém mudou a forma de forçar a \
         identidade, ou um host mudou de crate outra vez?)",
        hosts.len(),
        hosts.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>()
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
