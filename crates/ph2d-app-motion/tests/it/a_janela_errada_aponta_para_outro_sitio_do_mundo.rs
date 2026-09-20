//! ⭐⭐⭐ **A JANELA ERRADA APONTA PARA OUTRO SÍTIO DO MUNDO** — a metade que MEDE.
//!
//! ⚠️⚠️ **Ela mudou de crate na integração de 2026-09-20, e a razão é a lei do HOWTO §2.6:** *um
//! gate mora com o que EXERCITA*. Os seis censos irmãos dela leem a shell por CAMINHO e foram para
//! `ph2d-editor-core/tests/it/`, ao pé do `the_shell_only_shrinks`; este exercita
//! [`ph2d_app_motion::field_gizmo::scene_camera_window`], que é a lei desta crate — e a
//! `ph2d-editor-core` não pode depender do `ph2d-render` (gate de ciclo).
//!
//! ⛔ O gatilho foi a catraca da shell, `567` linhas acima do tecto na rodada de 20/09: a lei dela
//! é **mover**, nunca subir o número — e mover obrigou a perguntar de quem cada metade É.

/// ⭐⭐⭐ **QUANTO é que o dedo errava — o número, e a exigência de que ele EXISTA.**
///
/// Sem isto, os censos acima são afirmações sobre TEXTO. Esta metade mede a coisa: com o centro
/// partido a `55 %` (a ferramenta Motion activa) e a superfície da foto de 17/09
/// (`1930×1012`), o MESMO pixel de ecrã resolve para dois pontos do mundo diferentes.
///
/// ⛔ **E ela exige que a divergência seja GRANDE**, não que seja pequena: um tecto que aceitasse
/// «quase igual» tornaria a cura opcional. *A régua que aprova os dois lados não separa nada.*
///
/// ⚠️ **Os DOIS eixos**, e não só o `y`: o `aspect = w/h` cresce quando o `h` encolhe, logo o
/// `half_w` cresce com ele. A leitura *«está deslocado para baixo»* é o sintoma mais visível, não
/// a conta.
#[test]
fn a_janela_errada_aponta_para_outro_sitio_do_mundo() {
    use ph2d_editor_core::screens::layout::CenterSplit;
    use ph2d_host::WindowSize;

    let janela = WindowSize::new(1930, 1012);
    let banda = ph2d_app_motion::field_gizmo::scene_camera_window(
        CenterSplit::Horizontal { t: 0.55 },
        janela,
    );
    assert_ne!(
        banda.height, janela.height,
        "a fixtura não parte o centro — este gate mediria o nada"
    );
    let cam = ph2d_render::Camera2d::new([0.0, 0.0], 10.0);
    // O centro do botão do HUD na foto de 17/09.
    let ponto = (965.0_f32, 432.0_f32);
    let certo = cam.screen_to_world(ponto, banda);
    let errado = cam.screen_to_world(ponto, janela);
    let dx = f64::from((certo[0] - errado[0]).abs());
    let dy = f64::from((certo[1] - errado[1]).abs());
    assert!(
        dy > 1.0,
        "o eixo Y tinha de divergir mais de 1 metro e divergiu {dy:.3} — se a divergência \
         desaparecer, ou a lei do split mudou, ou esta régua deixou de a medir"
    );
    // ⚠️ No centro horizontal da vista o `x` coincide por SIMETRIA (`nx = 0` anula o `half_w`).
    //    Fora dele não coincide, e é isso que se afirma — senão alguém lê «é só o y» como lei.
    let fora = (1700.0_f32, 432.0_f32);
    let dx_fora = f64::from(
        (cam.screen_to_world(fora, banda)[0] - cam.screen_to_world(fora, janela)[0]).abs(),
    );
    assert!(
        dx < 1e-6 && dx_fora > 1.0,
        "no centro o X coincide por simetria ({dx:.6}) e fora dele NÃO ({dx_fora:.3}) — se esta \
         relação se inverter, a conta do `aspect` mudou"
    );
    println!(
        "[banda] com o centro a 55%: o mesmo pixel resolve {dy:.2} m ao lado no Y e {dx_fora:.2} m no X"
    );
}
