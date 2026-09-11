//! ⛔ **O vocabulário matemático ANTIGO não volta** — a cerca que a migração de 2026-08-29 precisa
//! e que o compilador **não** dá.
//!
//! # Por que este lint existe, e por que ele é estrutural
//!
//! Na subida da `rapier2d` 0.31 → 0.35 a matemática dela trocou de `nalgebra` para `glam` (via o
//! invólucro `glamx`), e esta crate migrou 25 ficheiros para os aliases do `parry` — o vocabulário
//! que vive em [`ph2d_physics::rmath`].
//!
//! ⚠️ **Mas a `rapier2d` 0.35 continua a re-exportar `nalgebra as na`**, porque ela própria ainda o
//! usa no código SIMD e nos jacobianos de multibody. ⇒ um `use rapier2d::na::Vector2` esquecido —
//! ou reintroduzido por alguém a copiar código de um exemplo antigo — **compila**. Ele não falha no
//! sítio do `use`; ele produz um tipo estrangeiro que só se manifesta como *mismatch* algures a
//! jusante, ou, pior, não se manifesta de todo se a expressão for genérica.
//!
//! *Uma migração cujo estado antigo ainda resolve não está terminada: ela está de pé por acordo.*
//!
//! # O que ele NÃO proíbe, e porquê
//!
//! - **`DVector`** (`rapier2d::math::DVector`, que é `na::DVector<Real>`) é legítimo e obrigatório:
//!   a `Multibody::inverse_kinematics` toma-o, o `glam` é de dimensão fixa e não tem equivalente.
//!   O `ik_coords.rs` e o `ik.rs` importam-no de propósito, com a razão escrita ao lado.
//! - **Comentários e docs** que citem os nomes mortos ficam de fora: eles são o *registo* de que a
//!   troca aconteceu, e um lint que se apanhasse a si próprio seria desligado no primeiro dia — a
//!   mesma razão do `no_untracked_writes_in_the_sim_crates`.
//!
//! # ⚠️ E este ficheiro nasce com a lista VAZIA, o que é o ponto
//!
//! Ele não conserta nada: mantém uma propriedade que **hoje é verdadeira** e que o compilador
//! deixou de garantir. É o mesmo movimento do `no_hex_in_ui` — a hora de escrever a cerca é quando
//! o campo do outro lado dela passa a valer alguma coisa.

use std::path::{Path, PathBuf};

/// Os caminhos que trazem de volta o vocabulário morto.
///
/// ⚠️ `rapier2d::na::` e `rapier2d::prelude::nalgebra` são as duas portas — a segunda porque o
/// `prelude` também o re-exporta, e um `use rapier2d::prelude::*` seguido de `nalgebra::Vector2`
/// resolve pelo mesmo caminho sem nomear `na`.
const FORBIDDEN: &[(&str, &str)] = &[
    (
        "rapier2d::na::",
        "o `nalgebra` que a rapier ainda re-exporta — use `crate::rmath`",
    ),
    (
        "prelude::nalgebra",
        "o mesmo, pela porta do prelude — use `crate::rmath`",
    ),
    (
        "nalgebra::Vector2",
        "o tipo morto: hoje é `crate::rmath::Vector`",
    ),
    (
        "nalgebra::Point2",
        "o tipo morto: o parry fundiu ponto e vetor em `crate::rmath::Vector`",
    ),
    (
        "nalgebra::Isometry2",
        "o tipo morto: hoje é `crate::rmath::Pose`",
    ),
    (
        "nalgebra::UnitComplex",
        "o tipo morto: hoje é `crate::rmath::Rotation`",
    ),
];

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn no_file_in_this_crate_reaches_for_rapiers_nalgebra() {
    let root = crate_root();
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    rust_files(&root.join("tests"), &mut files);

    let mut offenders = Vec::new();
    for f in &files {
        // ⚠️ Este ficheiro NOMEIA as agulhas que procura — saltá-lo é o que impede o lint de se
        // apanhar a si próprio, exactamente como o `no_untracked_writes_in_the_sim_crates` faz.
        if f.file_name()
            .is_some_and(|n| n == "the_old_math_vocabulary_is_gone.rs")
        {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(f) else {
            continue;
        };
        for (n, line) in src.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") || code.starts_with('*') {
                continue;
            }
            for (needle, why) in FORBIDDEN {
                if line.contains(needle) {
                    offenders.push(format!(
                        "  {}:{} — `{needle}` ({why})",
                        f.strip_prefix(&root).unwrap_or(f).display(),
                        n + 1
                    ));
                }
            }
        }
    }

    assert!(
        files.len() > 60,
        "o lint varreu so' {} ficheiros — ele nao esta' a olhar para onde pensa que olha, e um lint \
         apontado ao vazio passa sempre",
        files.len()
    );
    assert!(
        offenders.is_empty(),
        "o vocabulario matematico ANTIGO voltou a esta crate.\n{}\n\n\
         A `rapier2d` 0.35 ainda re-exporta `nalgebra as na` (ela usa-o no SIMD e nos jacobianos de \
         multibody dela), entao este `use` COMPILA -- e produz um tipo estrangeiro que so' se \
         manifesta como mismatch algures a jusante, ou nao se manifesta de todo.\n\
         O vocabulario desta crate e' o do `crate::rmath` (`Vector`, `Pose`, `Rotation`, `Real`), e \
         o modulo traz o aviso que interessa: o parry FUNDIU ponto e vetor no mesmo tipo, entao a \
         rede que o compilador dava desapareceu.\n\
         ⚠️ A UNICA excepcao legitima e' o `DVector` (`rapier2d::math::DVector`), que a \
         `Multibody::inverse_kinematics` exige e que o glam nao tem -- e ele NAO casa com nenhuma \
         das agulhas acima, de proposito.",
        offenders.join("\n")
    );
}
