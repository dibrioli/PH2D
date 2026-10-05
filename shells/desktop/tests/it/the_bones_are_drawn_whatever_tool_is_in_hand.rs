//! ⭐⭐⭐ **OS OSSOS DESENHAM-SE SEJA QUAL FOR A FERRAMENTA NA MÃO** (A14, plano
//! `docs/Skeleton/05_plano_o_esqueleto_e_um_objecto.md` §0).
//!
//! ⛔⛔ 2.º report do dono (05/10): *«o esqueleto se move mas fica invisível»*. O gesto chegava e o
//! desenho não: a fase do overlay só desenhava com `overlay.bones`, que era `vector_active` — a
//! ferramenta Vector, que desde a onda dos modos só está na mão no Edit de uma forma.
//!
//! ⚠️ Textual pela razão do irmão `the_bone_handles_are_painted_over_the_bones.rs`: o alvo é uma
//! cena do Vello que não se interroga. Controlo positivo: a porta dos ossos à vista é achada uma
//! vez (o filtro do olho tem gate próprio em `ph2d-skeleton-live`, `vista_tests`).

use std::path::PathBuf;

fn code_only(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("{} não se leu: {e}", p.display()))
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_bone_overlay_reads_the_eye_and_never_the_vector_tool() {
    let fase = code_only("src/render_loop/fase_vector_bone_overlay.rs");
    assert_eq!(
        fase.matches("visible_bone_polylines(").count(),
        1,
        "controlo: a fase desenha pela porta dos ossos À VISTA"
    );
    for veneno in ["overlay.bones", "VecOverlayPlan", "vector_active"] {
        assert!(
            !fase.contains(veneno),
            "a fase dos ossos lê `{veneno}` — o desenho volta a depender da ferramenta Vector, e o \
             esqueleto movido fora do Edit do vetor fica invisível (report de 05/10)"
        );
    }
    let cena = code_only("src/render_loop/fase_hero_scene.rs");
    assert_eq!(
        cena.matches("self.fase_vector_bone_overlay(vec_px_to_world, cam_affine)").count(),
        1,
        "a fase é chamada sem o plano do vetor"
    );
}
