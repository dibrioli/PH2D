//! Os gates do encaixe da câmara do jogo no canvas (plano GEF 17) — só API pública. Vieram de
//! `ph2d-app-game/src/encaixe.rs` (A6, pela catraca das crates `ph2d-app-*`).

use ph2d_app_game::encaixe::*;

/// O mapa mundo→pixel de uma câmara que cobre a janela (a mesma conta do `Camera2d`).
fn px(c: ([f32; 2], f32), janela: [f32; 2], p: [f32; 2]) -> [f32; 2] {
    let s = c.1 / janela[1];
    [
        janela[0] * 0.5 + (p[0] - c.0[0]) / s,
        janela[1] * 0.5 - (p[1] - c.0[1]) / s,
    ]
}

#[test]
fn a_vista_do_jogo_cai_inteira_no_canvas_entre_os_paineis() {
    let janela = [1600.0, 900.0];
    let canvas = [300.0, 60.0, 960.0, 700.0];
    let (centro, altura) = ([5.0, 2.0], 10.0);
    let c = na_janela(centro, altura, janela, canvas);
    let meia_l = altura * canvas[2] / canvas[3] * 0.5;
    // O centro, o topo-esquerdo e o fundo-direito da vista do jogo nos cantos do canvas.
    let perto = |a: [f32; 2], b: [f32; 2]| (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3;
    assert!(perto(px(c, janela, centro), [780.0, 410.0]));
    assert!(perto(
        px(c, janela, [centro[0] - meia_l, centro[1] + altura * 0.5]),
        [300.0, 60.0]
    ));
    assert!(perto(
        px(c, janela, [centro[0] + meia_l, centro[1] - altura * 0.5]),
        [1260.0, 760.0]
    ));
}

fn formato(p: ph2d_editor_core::GamePreset) -> ph2d_editor_core::GameFormat {
    ph2d_editor_core::GameFormat {
        preset: p,
        ..Default::default()
    }
}

/// ⭐ **A moldura é o rectângulo do jogo, em dois formatos** — a caixa tem a proporção do JOGO (nunca a do
/// canvas), cabe inteira no canvas e toca-o em dois lados.
#[test]
fn a_caixa_tem_a_proporcao_do_jogo_e_nao_a_do_canvas() {
    use ph2d_editor_core::GamePreset as P;
    let canvas = [300.0, 60.0, 960.0, 700.0];
    for (p, esperado) in [
        (P::Wide16x9, [300.0, 140.0, 960.0, 540.0]),
        (P::GameBoy, [391.111_1, 60.0, 777.777_8, 700.0]),
        (P::Phone9x16, [583.125, 60.0, 393.75, 700.0]),
    ] {
        let c = caixa_do_jogo(canvas, formato(p));
        for i in 0..4 {
            assert!(
                (c[i] - esperado[i]).abs() < 1e-2,
                "{p:?}: caixa {c:?}, esperada {esperado:?}"
            );
        }
        assert!((c[2] / c[3] - formato(p).aspect()).abs() < 1e-4);
    }
}

/// ⭐ A janela do jogo abre com a proporção do formato; uma consola antiga por um inteiro.
#[test]
fn a_janela_do_jogo_abre_com_o_formato() {
    use ph2d_editor_core::GamePreset as P;
    assert_eq!(tamanho_da_janela(formato(P::Wide16x9)), [1280, 720]);
    assert_eq!(tamanho_da_janela(formato(P::GameBoy)), [960, 864]);
    assert_eq!(tamanho_da_janela(formato(P::Pico8)), [896, 896]);
    assert_eq!(tamanho_da_janela(formato(P::Phone9x16)), [506, 900]);
}

/// ⭐ Seguir põe o centro do jogo no MEIO do canvas (entre os painéis) e não mexe no zoom do artista.
#[test]
fn seguir_poe_o_centro_do_jogo_no_meio_do_canvas_com_o_zoom_do_artista() {
    let janela = [1600.0, 900.0];
    let canvas = [300.0, 60.0, 960.0, 700.0];
    let altura = 23.0;
    let c = centro_que_segue([5.0, 2.0], altura, janela, canvas);
    let p = px((c, altura), janela, [5.0, 2.0]);
    assert!(
        (p[0] - 780.0).abs() < 1e-3 && (p[1] - 410.0).abs() < 1e-3,
        "{p:?}"
    );
}

#[test]
fn com_o_canvas_a_janela_inteira_e_a_identidade() {
    let c = na_janela([3.0, -1.0], 8.0, [1600.0, 900.0], [0.0, 0.0, 1600.0, 900.0]);
    assert_eq!(c, ([3.0, -1.0], 8.0));
}
