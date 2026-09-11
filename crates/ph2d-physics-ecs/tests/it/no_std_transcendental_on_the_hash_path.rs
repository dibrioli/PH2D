//! **A REGRA DO TRANSCENDENTAL ERA PROSA, E O HARNESS QUE A VIGIA QUEBROU-A.**
//!
//! O `physics_ecs_c9` é um hash de 120 passos comparado entre **três sistemas
//! operacionais** no CI, e a lei do módulo é que *1 ulp entre dois SOs é um bug*.
//! A regra que a protege — *transcendental passa por `libm`, nunca pelo `std`* —
//! vivia em **quatro doc-comments** (dois no `main.rs` do próprio c9, um no
//! `Cargo.toml` da `ph2d-platformer`, um no `player_leg.rs`) e em **zero gates**.
//!
//! ⚠️ **E um doc-comment não reprova ninguém.** A auditoria final desta linha
//! achou `f32::tan` e `f32::cos` a computar a pose de NASCIMENTO de dois corpos
//! hasheados — em `src/bin/physics_ecs_c9/player.rs`, ou seja **dentro do
//! harness que existe para vigiar exactamente isto**, trezentas linhas abaixo do
//! comentário que diz *"a `f32::sin_cos` here would diverge in the last ulps and
//! this hash would split across OSes"*.
//!
//! # ⚠️ O ponto cego tem FORMA, e é ela que este gate fecha
//!
//! A lei (`ph2d-platformer`) e a ponte (`bridge/player*.rs`) estavam **limpas** —
//! quem as escreveu tinha a regra na cabeça. O que escapou foi o **`src/bin/`**:
//! o harness, que ninguém pensa em auditar porque ele *é* a auditoria. Um gate
//! que varre `src/**` e **esquece o `src/bin/`** teria ficado verde sobre os dois.
//!
//! # ⚠️ Por que `sqrt` NÃO está na lista
//!
//! O IEEE-754 **especifica** a raiz quadrada (correctamente arredondada,
//! obrigatória) — as três plataformas devolvem o mesmo bit. `sin`/`cos`/`tan`/
//! `atan2`/`hypot`/`powf`/`exp`/`ln` estão na categoria *recomendada*, e nenhuma
//! libc é obrigada a concordar com outra. É por isso que o `step_limit` da lei
//! deriva a tangente do limite de rampa por `sqrt(1−c²)/c` sobre um cosseno do
//! `libm`: ele tomou o caminho caro **de propósito**.
//!
//! # ⚠️ A allowlist é por SÍTIO e diz por que aquele sítio não alcança o hash
//!
//! Ela não é uma isenção de arquivo: cada entrada nomeia o mecanismo, e o gate
//! **recusa entrada morta** (a que já não casa com nada) — senão ela vira o lugar
//! onde o próximo transcendental se esconde.

use std::path::{Path, PathBuf};

/// Os nomes que a libc **não é obrigada** a arredondar igual entre plataformas.
///
/// ⚠️ Como METODO (`.tan(`), que é a forma que de facto aparece em Rust; a forma
/// livre (`f32::tan(x)`) é rara aqui e cairia no mesmo detector se alguém a
/// escrevesse com o ponto.
const BANNED: &[&str] = &[
    ".sin(",
    ".cos(",
    ".tan(",
    ".sin_cos(",
    ".asin(",
    ".acos(",
    ".atan(",
    ".atan2(",
    ".hypot(",
    ".powf(",
    ".exp(",
    ".ln(",
    ".log(",
    ".log2(",
    ".log10(",
    ".cbrt(",
];

/// As crates cujo código pode alcançar o `physics_ecs_c9`.
const CRATES: &[&str] = &["ph2d-platformer", "ph2d-physics", "ph2d-physics-ecs"];

/// **Os sítios que EXISTEM e não alcançam o hash** — cada um com o mecanismo.
///
/// ⚠️ **Medido chamador a chamador, não presumido.** O critério é sempre o
/// mesmo: o `deterministic_hash` alimenta o hasher com `translation`/`rotation`
/// de cada corpo do `PhysicsBridge::bodies`, então um transcendental só importa
/// se o número dele chegar a uma pose. Gesto de editor, hit-test de clique e
/// guarda de finitude não chegam.
const ALLOWED: &[(&str, &str, &str)] = &[
    (
        "ph2d-physics/src/world/joints.rs",
        ".hypot(",
        "guarda de finitude do eixo (`len.is_finite() && len > 1e-6`): o \
         resultado escolhe um RAMO, nunca vira um float hasheado — o eixo sai de \
         `UnitVector2::new_normalize`",
    ),
    (
        "ph2d-physics/src/world/ik_coords.rs",
        ".atan2(",
        "IK: gesto de POSE transiente (ADR-0149), fora do laço do solver; o c9 \
         não tem lane de IK",
    ),
    (
        "ph2d-physics-ecs/src/bridge/ik_lead.rs",
        ".hypot(",
        "lead-drag do IK: gesto de editor, a mesma razão do irmão acima",
    ),
    (
        "ph2d-physics-ecs/src/bridge/fk.rs",
        ".hypot(",
        "gesto FK: idem — arrasto de autoria, nunca um tique simulado",
    ),
    (
        "ph2d-physics-ecs/src/bridge/rope.rs",
        ".hypot(",
        "`wheel_at_world`: HIT-TEST de clique do editor; o c9 não clica",
    ),
];

fn workspace_root() -> PathBuf {
    // `CARGO_MANIFEST_DIR` é `<root>/crates/ph2d-physics-ecs`.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("a raiz da workspace")
        .to_path_buf()
}

/// Percorre `src/**` de uma crate — ⚠️ **incluindo `src/bin/`**, que é onde os
/// dois defeitos que motivaram este gate viviam.
fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// **O detector, isolado do disco** — é ele que o controle positivo exercita.
///
/// Devolve `(linha_1based, agulha)` para cada ocorrência que conta. Ignora
/// comentários de linha e **para no `mod tests`**: um teste pode usar o que
/// quiser, porque nada dele entra no binário do harness.
fn offences(src: &str) -> Vec<(usize, &'static str)> {
    let mut out = Vec::new();
    for (i, raw) in src.lines().enumerate() {
        let line = raw.trim_start();
        // O corpo de teste começa aqui e vai até ao fim do arquivo (a convenção
        // deste repo põe `mod tests` por último, ou num irmão `*_tests.rs`).
        if line.starts_with("mod tests") || line.starts_with("pub mod tests") {
            break;
        }
        if line.starts_with("//") {
            continue;
        }
        for needle in BANNED {
            if line.contains(needle) {
                out.push((i + 1, *needle));
            }
        }
    }
    out
}

/// ⚠️ **O CONTROLE POSITIVO, e ele vem primeiro de propósito.** Um arch-gate que
/// varre e não acha nada é indistinguível de um arch-gate quebrado — esta linha
/// é a que prova que a varredura sabe achar. (A cicatriz é desta casa: o
/// `the_shape_slot_goes_through_the_shape_door` pegou o próprio scanner partido
/// com o controle positivo, na hora.)
#[test]
fn the_detector_finds_what_it_is_looking_for() {
    let sample = "\
fn a() { let y = 6.0 * slope.abs().tan() + 1.1; }
// let z = x.cos(); -- comentado, não conta
fn b() { let w = v[0].hypot(v[1]); }
mod tests {
    fn c() { let q = t.sin_cos(); }
}
";
    let hits = offences(sample);
    assert_eq!(
        hits.len(),
        2,
        "o detector tem de achar o `.tan(` e o `.hypot(`, ignorar o comentário e \
         PARAR no `mod tests` — achou {hits:?}"
    );
    assert_eq!(hits[0], (1, ".tan("));
    assert_eq!(hits[1], (3, ".hypot("));

    // E a metade oposta: um `sqrt` é IEEE-754-especificado e NÃO é ofensa.
    assert!(
        offences("fn a() { let r = x.sqrt(); }").is_empty(),
        "o `sqrt` é obrigatório e correctamente arredondado — bani-lo seria \
         proibir a cura que a lei usa para evitar a tangente do `std`"
    );
}

#[test]
fn no_std_transcendental_reaches_the_deterministic_hash() {
    let root = workspace_root();
    let mut files = Vec::new();
    for krate in CRATES {
        rust_sources(&root.join("crates").join(krate).join("src"), &mut files);
    }
    assert!(
        files.len() > 50,
        "a varredura tem de ver as três crates (viu {} arquivos) — um scanner \
         que não acha arquivo nenhum passa VERDE sobre qualquer defeito",
        files.len()
    );

    let mut bad = Vec::new();
    let mut used = vec![false; ALLOWED.len()];
    for path in &files {
        // ⚠️ Um `*_tests.rs` é o irmão de um `mod tests` — mesma isenção, mesma
        // razão: nada dele entra no binário.
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with("_tests.rs"))
        {
            continue;
        }
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        let rel = rel.trim_start_matches("crates/").to_string();
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        for (line, needle) in offences(&src) {
            match ALLOWED
                .iter()
                .position(|(p, n, _)| *p == rel && *n == needle)
            {
                Some(i) => used[i] = true,
                None => bad.push(format!("{rel}:{line}  {needle}")),
            }
        }
    }

    assert!(
        bad.is_empty(),
        "transcendental do `std` no caminho do `physics_ecs_c9` — a libc de cada \
         SO devolve o último ulp diferente, e o hash parte entre as três \
         plataformas. Use `libm::` (pin `=0.2.16`, já é dependência das três \
         crates) ou reformule em forma fechada, como o `step_limit` faz com \
         `sqrt`. Sítios:\n  {}",
        bad.join("\n  ")
    );

    // ⚠️ **Entrada morta é onde o próximo transcendental se esconde.** Se um
    // sítio isento desapareceu, a isenção some com ele.
    let stale: Vec<_> = ALLOWED
        .iter()
        .zip(&used)
        .filter(|(_, u)| !**u)
        .map(|((p, n, _), _)| format!("{p}  {n}"))
        .collect();
    assert!(
        stale.is_empty(),
        "isenção STALE — o sítio já não existe, então a linha da allowlist tem \
         de sair:\n  {}",
        stale.join("\n  ")
    );
}
