//! W0 stub for the `smoke_fixture_renderable` gate
//! (Sprite_projeto §15.8.2). The spec line:
//!
//! > **Gate `smoke_fixture_renderable`** (W0.T0.X cria): cada
//! > fixture carrega no `./play.command` sem panic; goldens existem
//! > em `smoke_goldens/`. Sem isso, smoke é teatral.
//!
//! In W0 the actual scene fixtures don't exist yet (W2-W5 schema
//! features required to author them); the gate is reduced to a
//! **directory-presence contract** — both
//! `assets/smoke_fixtures/sprite_inspector_v2/` and
//! `docs/Sprite_projeto/smoke_goldens/` exist and carry a README
//! enumerating the wave-by-wave fixture roadmap.
//!
//! As each wave lands its `.scene` fixture + PNG goldens, the gate
//! grows to actually load the scenes through `./play.command`
//! headless and bit-compare against the goldens.
//!
//! # ⚠️ ESTADO MEDIDO EM 2026-08-21 (auditoria `docs/Sprite_projeto/20` §6.1)
//!
//! **Nada disto foi construído.** As duas pastas contêm **um README cada e zero ficheiros**; o
//! único teste que corre afirma que as pastas existem — uma tautologia. Os quatro testes de golden
//! são `unimplemented!()` sob `#[ignore]`.
//!
//! E o mais importante: **o gatilho escrito no `#[ignore]` de dois deles JÁ DISPAROU.** As notas
//! diziam *«quando a W2 aterrar per_corner_tint + self_tint + tint_fill + opacity»* e *«depende de
//! SortingLayer / ZIndexOverride / YSort / SortingGroup / ShowBehindParent»* — e as **nove** peças
//! existem, medidas. As notas ficaram a apontar para um futuro que já é passado.
//!
//! ⛔ **O bloqueio real não é o que elas dizem.** É a ausência de três coisas que ninguém escreveu:
//! os `.scene`, os PNG de referência, e um **arnês de render headless** que os compare. Cada nota
//! abaixo foi reescrita para nomear ISSO — *quem move o número que tornava algo inalcançável tem de
//! reconferir a nota* (`CLAUDE.md` §0.0).

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/ph2d-render; pop twice to reach
    // the workspace root deterministically.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crate is two levels below workspace root")
        .to_path_buf()
}

#[test]
fn smoke_fixture_dir_canonical_path_exists() {
    let dir = workspace_root().join("assets/smoke_fixtures/sprite_inspector_v2");
    assert!(
        dir.is_dir(),
        "spec §15.8.2 canonical smoke fixture dir missing: {dir:?}"
    );
    let readme = dir.join("README.md");
    assert!(
        readme.is_file(),
        "smoke fixture dir README enumerating the wave roadmap missing: {readme:?}"
    );
}

#[test]
fn smoke_goldens_dir_canonical_path_exists() {
    let dir = workspace_root().join("docs/Sprite_projeto/smoke_goldens");
    assert!(
        dir.is_dir(),
        "spec §15.8.2 canonical goldens dir missing: {dir:?}"
    );
    let readme = dir.join("README.md");
    assert!(
        readme.is_file(),
        "goldens dir README enumerating the per-wave PNG list missing: {readme:?}"
    );
}

#[test]
#[ignore = "W2 goldens NAO EXISTEM. ⚠️ O gatilho antigo ('quando a W2 aterrar per_corner_tint/self_tint/tint_fill/opacity') JA' DISPAROU -- os quatro existem desde 2026-05. O que falta e' o que ninguem escreveu: smoke_w2_color_tint.scene, os 5 PNG e um arnes de render headless."]
fn w2_smoke_scene_loads_without_panic_and_matches_goldens() {
    unimplemented!(
        "W2.T2.X smoke fixture not yet wired. Replace this body with: \
         (1) load assets/smoke_fixtures/sprite_inspector_v2/smoke_w2_color_tint.scene; \
         (2) headless-render via ph2d-host; \
         (3) bit-compare against docs/Sprite_projeto/smoke_goldens/w2_*.png (5 goldens). \
         Depends on W2 schema features: per_corner_tint + self_tint + tint_fill + opacity."
    );
}

#[test]
#[ignore = "W3 goldens NAO EXISTEM. ⚠️ O gatilho antigo (SortingLayer/ZIndexOverride/YSort/SortingGroup/ShowBehindParent) JA' DISPAROU -- os cinco componentes existem. Falta smoke_w3_sorting.scene, os 5 PNG e o arnes headless."]
fn w3_smoke_scene_loads_without_panic_and_matches_goldens() {
    unimplemented!(
        "W3.T3.X smoke fixture not yet wired. Replace this body with: \
         (1) load assets/smoke_fixtures/sprite_inspector_v2/smoke_w3_sorting.scene; \
         (2) headless-render via ph2d-host; \
         (3) bit-compare against docs/Sprite_projeto/smoke_goldens/w3_*.png (5 goldens). \
         Depends on W3 features: SortingLayer + ZIndexOverride + YSort + SortingGroup + ShowBehindParent."
    );
}

#[test]
#[ignore = "W4 goldens NAO EXISTEM, e o gatilho DISPAROU PELA METADE em 2026-08-23: o `SpriteAnimator` EXISTE (a §11 Animation nasceu, `docs/Sprite_projeto/21`), esta' registado e tem tique de passo fixo -- esta nota dizia o contrario e envelheceu num dia. O que continua a faltar: `Material`/`InstanceShaderParams` (o slot e' placeholder a` espera de um runtime de shader) e, como nas irmas, o arnes de render headless + os 3 PNG."]
fn w4_smoke_scene_loads_without_panic_and_matches_goldens() {
    unimplemented!(
        "W4.T4.X smoke fixture not yet wired. Replace this body with: \
         (1) load assets/smoke_fixtures/sprite_inspector_v2/smoke_w4_material_animation.scene; \
         (2) headless-render via ph2d-host; \
         (3) bit-compare against docs/Sprite_projeto/smoke_goldens/w4_*.png (3 goldens). \
         Depends on W4 features: Material + UseParentMaterial + InstanceShaderParams + SpriteAnimator."
    );
}

#[test]
#[ignore = "W5 goldens NAO EXISTEM. ⚠️ O gatilho DISPAROU em 2026-08-21: o `NamedAnchorList` existe, esta' registado, sobrevive ao disco e ja' tem gizmo de canvas. O que falta e' o mesmo das W2/W3 -- smoke_w5_anchors.scene, os PNG e um arnes de render headless. *Uma nota de diferido que descreve um mundo que acabou manda procurar o trabalho no sitio errado.*"]
fn w5_smoke_scene_loads_without_panic_and_matches_goldens() {
    unimplemented!(
        "W5.T5.X smoke fixture not yet wired. Replace this body with: \
         (1) load assets/smoke_fixtures/sprite_inspector_v2/smoke_w5_named_anchors.scene; \
         (2) headless-render via ph2d-host; \
         (3) bit-compare against docs/Sprite_projeto/smoke_goldens/w5_*.png (4 goldens). \
         Depends on W5 features: NamedAnchorList (socket / slice / 9slice) + per-frame anchor override."
    );
}
