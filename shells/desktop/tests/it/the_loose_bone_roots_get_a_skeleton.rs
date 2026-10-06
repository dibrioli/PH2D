//! ⭐ **A porta das raízes soltas corre em todo quadro, antes das famílias de modo** (A14). A lei
//! (um esqueleto por raiz, pose ao bit) tem gate em `ph2d-app-skeleton` (`loose_tests`); aqui mede-se
//! que a shell a chama — sem a chamada, um projecto antigo abre com ossos que nenhum modo alcança.

#[test]
fn the_loose_roots_are_adopted_before_the_mode_families() {
    let p = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/render_loop/fase_object_mode.rs"
    );
    let src: String = std::fs::read_to_string(p)
        .expect("a fase do modo")
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let porta = src
        .find("loose::adopt_loose_roots(sim,")
        .expect("a porta das raízes soltas não corre no quadro");
    let familia = src
        .find("skeleton_mode::Family::new(")
        .expect("controlo: a família do esqueleto é composta aqui");
    assert!(
        porta < familia,
        "a porta corre depois das famílias — o modo vê a raiz solta"
    );
}
