//! Determinism replay test for [`propagate_transforms`] (HR-5).
//!
//! Builds a deterministic mixed hierarchy (100 entities with seeded
//! roots and child relationships), runs propagation, hashes the
//! `GlobalTransform` matrix bytes, and asserts the hash is stable
//! across two independent runs in the same process.
//!
//! ## Cross-OS scope (T1.3.5)
//!
//! Pre-T1.3.5 this gate was effectively SAME-PROCESS-ONLY: Rust std
//! `f32::sin_cos` routes to platform-native libm (libsystem on macOS,
//! glibc/musl on linux, MSVC CRT on windows) and those impls diverge
//! in the last 1-2 ulps for some inputs. The "CI cross-platform matrix
//! verifies the hash" claim was aspirational, not load-bearing —
//! a linux-CI hash and a macOS-author hash would differ for any
//! non-trivial rotation.
//!
//! Post-T1.3.5, `Transform::compose` + `GlobalTransform::from_transform`
//! both route through `libm::sincosf` (pure-Rust port of MUSL libm —
//! platform-independent IEEE 754). The CI matrix (linux + macOS +
//! windows) now actually verifies bit-identical hashes across hosts;
//! this gate is the canonical cross-OS bit-identical reference.

use ph2d_core::Vec2;
use ph2d_ecs::{
    ChildOf, GlobalTransform, PresentWorld, SimRef, SimWorld, Transform, TransformPropagationState,
    WorklistBuf, propagate_transforms_into_present,
};

/// Deterministic LCG so this test is free of any RNG crate. Same
/// seeds produce identical streams on every architecture/OS.
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next_u32(&mut self) -> u32 {
        // Numerical Recipes LCG constants — known-good full period.
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.state >> 32) as u32
    }
    fn next_f32_unit(&mut self) -> f32 {
        (self.next_u32() as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

fn build_world() -> SimWorld {
    let mut sim = SimWorld::new();
    let mut rng = Lcg::new(0xC0FFEE_u64);

    // 20 roots, each with 4 children → 100 entities total.
    let mut roots = Vec::with_capacity(20);
    for _ in 0..20 {
        let pos = Vec2::new(rng.next_f32_unit() * 5.0, rng.next_f32_unit() * 5.0);
        let rot = rng.next_f32_unit() * std::f32::consts::TAU;
        let scale_factor = 0.5 + rng.next_f32_unit().abs() * 1.5;
        let e = sim
            .world_mut()
            .spawn(Transform {
                translation: pos,
                rotation: rot,
                scale: Vec2::new(scale_factor, scale_factor),
                // skew=0 → compose/from_transform degenerate bit-
                // identically to v1, so EXPECTED_GLOBALS_HASH holds.
                ..Transform::IDENTITY
            })
            .id();
        roots.push(e);
    }
    for &root in &roots {
        for _ in 0..4 {
            let pos = Vec2::new(rng.next_f32_unit(), rng.next_f32_unit());
            let rot = rng.next_f32_unit() * std::f32::consts::PI;
            sim.world_mut().spawn((
                Transform {
                    translation: pos,
                    rotation: rot,
                    scale: Vec2::new(1.0, 1.0),
                    ..Transform::IDENTITY
                },
                ChildOf(root),
            ));
        }
    }
    sim
}

fn hash_globals(sim: &mut SimWorld) -> [u8; 32] {
    let mut present = PresentWorld::new();
    let mut state = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::new();
    ph2d_ecs::extract!(*sim => present, |sim_w, present_w| {
        propagate_transforms_into_present(sim_w, &mut state, present_w, &mut worklist);
    });

    let mut q = present.world_mut().query::<(&SimRef, &GlobalTransform)>();
    let mut pairs: Vec<(u64, [f32; 9])> = q
        .iter(present.world())
        .map(|(sref, gt)| {
            let m = &gt.matrix;
            (
                sref.0.to_bits(),
                [
                    m.x_axis.x, m.x_axis.y, m.x_axis.z, m.y_axis.x, m.y_axis.y, m.y_axis.z,
                    m.z_axis.x, m.z_axis.y, m.z_axis.z,
                ],
            )
        })
        .collect();
    pairs.sort_by_key(|(e, _)| *e);

    let mut hasher = blake3::Hasher::new();
    for (e, m) in &pairs {
        hasher.update(&e.to_le_bytes());
        for f in m {
            hasher.update(&f.to_le_bytes());
        }
    }
    *hasher.finalize().as_bytes()
}

/// Como [`hash_globals`], mas **só os floats** — a metade que responde *«a matemática mexeu?»*.
///
/// A ordem é a mesma (pares ordenados por `to_bits`), então um re-rotular uniforme das entidades
/// não a altera; uma **permutação** real alteraria, porque mudaria que matriz vem em que posição.
fn hash_matrices_only(sim: &mut SimWorld) -> [u8; 32] {
    let mut present = PresentWorld::new();
    let mut state = TransformPropagationState::new(sim.world_mut());
    let mut worklist = WorklistBuf::new();
    ph2d_ecs::extract!(*sim => present, |sim_w, present_w| {
        propagate_transforms_into_present(sim_w, &mut state, present_w, &mut worklist);
    });

    let mut q = present.world_mut().query::<(&SimRef, &GlobalTransform)>();
    let mut pairs: Vec<(u64, [f32; 9])> = q
        .iter(present.world())
        .map(|(sref, gt)| {
            let m = &gt.matrix;
            (
                sref.0.to_bits(),
                [
                    m.x_axis.x, m.x_axis.y, m.x_axis.z, m.y_axis.x, m.y_axis.y, m.y_axis.z,
                    m.z_axis.x, m.z_axis.y, m.z_axis.z,
                ],
            )
        })
        .collect();
    pairs.sort_by_key(|(e, _)| *e);

    let mut hasher = blake3::Hasher::new();
    for (_, m) in &pairs {
        for f in m {
            hasher.update(&f.to_le_bytes());
        }
    }
    *hasher.finalize().as_bytes()
}

#[test]
fn same_world_twice_identical_hash() {
    let mut sim_a = build_world();
    let mut sim_b = build_world();
    let h_a = hash_globals(&mut sim_a);
    let h_b = hash_globals(&mut sim_b);
    assert_eq!(
        h_a, h_b,
        "two independently-built identical worlds produced different propagation hashes"
    );
}

#[test]
fn repeated_propagation_same_world_stable() {
    let mut sim = build_world();
    let h1 = hash_globals(&mut sim);
    let h2 = hash_globals(&mut sim);
    assert_eq!(h1, h2, "running propagation twice on the same sim diverged");
}

/// Cross-OS bit-identical golden hash (T1.3.5; R1 Lens C C-C1+C-C2).
///
/// Captured 2026-05-28 on macOS aarch64 after the workspace-wide
/// libm sweep (`libm = "=0.2.16", default-features = false`). The
/// CI matrix (linux + macOS + windows) runs THIS exact test and
/// MUST produce the same hash on every host; any drift = a
/// determinism regression to investigate (libm bump? glam bump?
/// accidental re-introduction of `f32::sin_cos` in the propagation
/// path? new FMA-enabled feature?). DO NOT update this constant
/// without first re-running cross-OS and documenting the cause in
/// an ADR amendment (HR-5 contract).
///
/// Reproducible — same `build_world()` LCG seed (0xC0FFEE) +
/// deterministic spawn order = same hash forever.
///
/// To re-capture after a deliberate libm/glam bump:
///   1. Set this to `[0u8; 32]` temporarily.
///   2. `cargo test -p ph2d-ecs --test transform_determinism cross_os_golden_hash_pinned`.
///   3. Read the actual hash bytes from the panic message.
///   4. Replace the constant + commit + cross-OS CI run verifies.
const EXPECTED_GLOBALS_HASH: [u8; 32] = [
    // blake3("987aa255a1b89681ecae402bf0b60fe484d1184e78b7075756f31ee38d900024")
    // recapturado 2026-08-29 (linux x86_64, libm =0.2.16 default-features=false,
    // glam 0.30.10, **bevy_ecs 0.19.1**). MUST match on macOS + windows CI hosts.
    //
    // ⚠️ O anterior era `d2a3ca34…`, capturado 2026-05-28 sob bevy_ecs 0.18.1. A deriva
    // é **rotulagem de entidade, não matemática** — ver `EXPECTED_MATRICES_HASH` abaixo.
    0x98, 0x7a, 0xa2, 0x55, 0xa1, 0xb8, 0x96, 0x81, 0xec, 0xae, 0x40, 0x2b, 0xf0, 0xb6, 0x0f, 0xe4,
    0x84, 0xd1, 0x18, 0x4e, 0x78, 0xb7, 0x07, 0x57, 0x56, 0xf3, 0x1e, 0xe3, 0x8d, 0x90, 0x00, 0x24,
];

/// ⭐ **A METADE QUE NUNCA PODE MEXER POR OUTRA RAZÃO QUE NÃO A MATEMÁTICA.**
///
/// O [`EXPECTED_GLOBALS_HASH`] mistura duas coisas: os **bits de identidade** de cada entidade e os
/// **nove floats** da matriz dela. Isso torna-o sensível a qualquer mudança interna do ECS que
/// re-rotule entidades — e foi exactamente o que aconteceu ao subir o `bevy_ecs` para 0.19, que
/// gasta o índice 0 do alocador numa entidade sua no arranque e desloca **todos** os ids em um.
///
/// ⚠️ Nessa altura o portão fica vermelho e a mensagem pergunta *«libm? glam?»* — as duas respostas
/// erradas. Quem estivesse com pressa carimbava o valor novo e seguia, e **é aí que uma regressão
/// de determinismo real passaria despercebida**: ela produz o mesmo sintoma.
///
/// Esta constante separa as duas perguntas. Ela hasheia **só os floats**, na mesma ordem:
///
/// | vermelho | significado |
/// |---|---|
/// | só o `GLOBALS` | **re-rotulagem** de entidades — o ECS mudou, a matemática não. Recapture. |
/// | **este** (com ou sem o outro) | ⛔ **a matemática mexeu** — libm, glam, FMA, `f32::sin_cos`. Investigue. |
///
/// **Medido em 2026-08-29, A/B do bevy_ecs 0.18.1 contra 0.19.1** (a 0.18 corrida numa árvore
/// separada, no commit anterior): `GLOBALS` mudou de `d2a3ca34…` para `987aa255…` e este ficou
/// **byte-idêntico**; o primeiro `to_bits` foi de `4294967196` para `4294967195`, um deslocamento
/// de exactamente **1** em todos os 100. *Foi isso, e só isso.*
const EXPECTED_MATRICES_HASH: [u8; 32] = [
    // blake3("0308874dbfe9f12960f72195f40597fbe9b1ccf1c16542909c158ae5fd4cce17")
    // O MESMO valor sob bevy_ecs 0.18.1 e 0.19.1.
    0x03, 0x08, 0x87, 0x4d, 0xbf, 0xe9, 0xf1, 0x29, 0x60, 0xf7, 0x21, 0x95, 0xf4, 0x05, 0x97, 0xfb,
    0xe9, 0xb1, 0xcc, 0xf1, 0xc1, 0x65, 0x42, 0x90, 0x9c, 0x15, 0x8a, 0xe5, 0xfd, 0x4c, 0xce, 0x17,
];

fn hash_hex(h: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in h {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// Arch-gate: every Cargo.toml that ships libm must carry the EXACT
/// pin `=0.2.16` + `default-features = false`. Symmetric to
/// `postcard_exact_version_pin_enforced_in_cargo_toml` in
/// ph2d-render — catches an agent stripping the `=` prefix or
/// re-enabling default-features (which would pull the `arch`
/// platform-intrinsics feature and defeat the determinism contract).
/// R2 Lens E-C3 + meta-C2.
#[test]
fn libm_exact_version_pin_enforced_in_workspace() {
    // Each entry: (relative path from CARGO_MANIFEST_DIR, label for
    // diagnostics). All 4 crates with `f32 → libm::sincosf` swaps
    // must pin EXACTLY the same version + features.
    //
    // CARGO_MANIFEST_DIR for this test crate (ph2d-ecs) is
    // `crates/ph2d-ecs`; we step out to the workspace root.
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("ph2d-ecs is two levels below workspace root");
    let crates = [
        ("crates/ph2d-ecs/Cargo.toml", "ph2d-ecs"),
        ("crates/ph2d-editor-core/Cargo.toml", "ph2d-editor-core"),
        (
            "crates/ph2d-tool-rasterize/Cargo.toml",
            "ph2d-tool-rasterize",
        ),
        ("shells/desktop/Cargo.toml", "ph2d-host-desktop"),
        ("tools/asset-cooker/Cargo.toml", "ph2d-asset-cooker"),
    ];
    for (rel, label) in crates {
        let path = workspace.join(rel);
        let toml = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
        // Find the libm dep line (allowing column-padding alignment
        // like `libm                  = { ... }`). Match any line
        // starting with `libm` (after trim) and a `=` followed by
        // a TOML inline table with the exact-pin + default-features.
        let libm_line = toml
            .lines()
            .find(|l| {
                let t = l.trim_start();
                t.starts_with("libm") && t.contains('=') && t.contains("0.2.16")
            })
            .unwrap_or_else(|| {
                panic!("{label} ({rel}): no libm dep line found");
            });
        assert!(
            libm_line.contains(r#"version = "=0.2.16""#),
            "{label} ({rel}) libm line `{}` lost the `=0.2.16` exact-pin syntax — \
             HR-5 cross-OS golden hash depends on this. Any bump = regenerate \
             EXPECTED_GLOBALS_HASH + cross-OS re-verify + ADR amendment.",
            libm_line.trim()
        );
        assert!(
            libm_line.contains("default-features = false"),
            "{label} ({rel}) libm dep line `{}` must carry `default-features = false` — \
             the `arch` feature enables platform-specific intrinsics that defeat \
             cross-OS bit-identical.",
            libm_line.trim()
        );
    }
}

#[test]
fn cross_os_golden_hash_pinned() {
    let mut sim = build_world();

    // ⭐ **A metade da MATEMÁTICA primeiro, e é de propósito.** Se as duas estiverem vermelhas, a
    // primeira mensagem que se lê tem de ser a que nomeia a causa mais grave — senão a leitura
    // começa pela pergunta errada, que é como uma regressão de determinismo se disfarça de
    // re-rotulagem inofensiva.
    let m = hash_matrices_only(&mut sim);
    if m != EXPECTED_MATRICES_HASH {
        panic!(
            "⛔ A MATEMATICA MEXEU — o hash das MATRIZES (sem os ids das entidades) derivou.\n\
             actual   = {}\n\
             expected = {}\n\
             Isto NAO e' re-rotulagem de entidade: os ids nem entram nesta conta. Investigue, por \
             ordem de probabilidade: bump do `libm` (o pin tem de ser `=0.2.16`, \
             `default-features = false`) · bump do `glam` · reintroducao de `f32::sin_cos` em \
             algum caminho de transform (ele vai ao libm da PLATAFORMA e diverge 1-2 ulps entre \
             SOs) · uma feature nova que ligue FMA.\n\
             ⚠️ NAO recapture este valor sem saber qual das quatro foi: ele e' a unica coisa que \
             separa «o ECS re-rotulou» de «perdemos o bit-identico entre SOs», e recapturar as \
             duas ao mesmo tempo apaga essa distincao para sempre.\n  {:?}",
            hash_hex(&m),
            hash_hex(&EXPECTED_MATRICES_HASH),
            m
        );
    }

    let h = hash_globals(&mut sim);
    if h != EXPECTED_GLOBALS_HASH {
        panic!(
            "cross-OS golden hash drifted — mas a metade da MATEMATICA esta' VERDE.\n\
             actual = {}\n\
             expected = {}\n\
             ⇒ o que mudou foram os IDS das entidades, nao os floats. A causa provavel e' um bump \
             do `bevy_ecs` (a 0.19 gasta o indice 0 do alocador numa entidade sua no arranque e \
             desloca todos os ids em um -- medido em 2026-08-29). Nesse caso, recapturar este \
             valor e' a resposta certa:\n  {:?}\n\
             Se NAO houve bump de ECS, entao alguma coisa mudou a ORDEM ou a QUANTIDADE de \
             entidades da fixtura -- e ai' o hash certo e' outro pela razao errada.",
            hash_hex(&h),
            hash_hex(&EXPECTED_GLOBALS_HASH),
            h
        );
    }
}

// blake3 is already a transitive dep via the workspace (ph2d-asset
// pulls it; here we'd need it as a dev-dep). Add it to ph2d-ecs/
// Cargo.toml [dev-dependencies] — see that file.
