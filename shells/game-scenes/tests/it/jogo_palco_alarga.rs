//! ⭐ O palco ALARGA (Godot `expand`): na tela cheia e na janela do jogo, um ecrã com outra proporção que a do
//! formato mostra mais mundo — nunca tarjas — e a cerca mede a área visível (`ph2d_camera_frame::palco`).
//! Defeito do dono (11/11): faixa preta por baixo do chão na tela cheia (1600 × 972: 36 px em cima e em baixo).

use ph2d_app_game::encaixe::*;
use ph2d_ecs::CameraLimits;
use ph2d_editor_core::{GameFormat, GamePreset as P};

fn formato(p: P) -> GameFormat {
    GameFormat {
        preset: p,
        ..Default::default()
    }
}

/// O mapa mundo→pixel de uma câmara que cobre a janela (a mesma conta do `Camera2d`).
fn px(c: ([f32; 2], f32), janela: [f32; 2], p: [f32; 2]) -> [f32; 2] {
    let s = c.1 / janela[1];
    [
        janela[0] * 0.5 + (p[0] - c.0[0]) / s,
        janela[1] * 0.5 - (p[1] - c.0[1]) / s,
    ]
}

/// A porta inteira do quadro no palco (`fase_game_camera` + `tarjas_do_palco`): a câmara da janela e a área.
fn palco(
    janela: [f32; 2],
    f: GameFormat,
    lim: Option<&CameraLimits>,
    v: [f32; 2],
) -> (([f32; 2], f32), [f32; 4], [f32; 4]) {
    let canvas = [0.0, 0.0, janela[0], janela[1]];
    let caixa = caixa_do_jogo(canvas, f);
    let area = area_do_palco(canvas, f);
    let c = centro_no_palco(lim, v, v, 8.0, caixa, area);
    (na_janela(c, 8.0, janela, caixa), caixa, area)
}

const ECRAS: [[f32; 2]; 5] = [
    [1280.0, 720.0],
    [1600.0, 1000.0],
    [1024.0, 768.0],
    [1280.0, 692.0],
    [1600.0, 972.0],
];

/// ⭐ Num 16:9 em qualquer ecrã o mundo cobre o ecrã inteiro (área = ecrã, as tarjas não pintam nada) e o
/// jogador fica no mesmo sítio da caixa do jogo; controlo: o ecrã 16:9 dá a vista de antes.
#[test]
fn na_tela_cheia_o_mundo_cobre_o_ecra_inteiro_e_o_jogador_fica_no_seu_sitio() {
    let f = formato(P::Wide16x9);
    let (v, jogador) = ([-5.9, 3.0], [-8.0, 1.5]);
    for janela in ECRAS {
        let (cam, caixa, area) = palco(janela, f, None, v);
        assert_eq!(area, [0.0, 0.0, janela[0], janela[1]], "{janela:?}: tarjas");
        // O jogador no mesmo sítio RELATIVO da caixa do jogo (a vista que o formato promete).
        let p = px(cam, janela, jogador);
        let rel = [(p[0] - caixa[0]) / caixa[2], (p[1] - caixa[1]) / caixa[3]];
        let meia = [8.0 * f.aspect() * 0.5, 4.0];
        let esperado = [
            0.5 + (jogador[0] - v[0]) / (2.0 * meia[0]),
            0.5 - (jogador[1] - v[1]) / (2.0 * meia[1]),
        ];
        assert!(
            (rel[0] - esperado[0]).abs() < 1e-4 && (rel[1] - esperado[1]).abs() < 1e-4,
            "{janela:?}: {rel:?} × {esperado:?}"
        );
    }
    // Controlo: aspecto igual ao do formato ⇒ caixa = ecrã e a câmara de antes.
    let (cam, caixa, _) = palco([1280.0, 720.0], f, None, v);
    assert_eq!(caixa, [0.0, 0.0, 1280.0, 720.0]);
    assert_eq!(cam, (v, 8.0));
}

/// ⭐ Com limites (o nível 2 do smoke), a área ALARGADA não sai deles: encostada ao limite da esquerda num ecrã
/// mais largo, a borda esquerda do ecrã é a do limite; mais alto, a de baixo. Controlo: no 16:9 nada muda.
#[test]
fn a_cerca_mede_a_area_visivel_e_nao_mostra_o_fora_do_nivel() {
    let f = formato(P::Wide16x9);
    let lim = CameraLimits {
        min: [-13.0, -1.0],
        max: [17.0, 9.0],
    };
    let meia_l = 8.0 * f.aspect() * 0.5;
    let v = [-13.0 + meia_l, 3.0];
    let (cam, ..) = palco([1280.0, 692.0], f, Some(&lim), v);
    let esq = px(cam, [1280.0, 692.0], [-13.0, 0.0]);
    assert!(esq[0].abs() < 1e-2, "borda esquerda em {esq:?}");
    let v = [0.0, -1.0 + 4.0];
    let (cam, ..) = palco([1600.0, 1000.0], f, Some(&lim), v);
    let baixo = px(cam, [1600.0, 1000.0], [0.0, -1.0]);
    assert!(
        (baixo[1] - 1000.0).abs() < 1e-2,
        "borda de baixo em {baixo:?}"
    );
    let (cam, ..) = palco([1280.0, 720.0], f, Some(&lim), v);
    assert_eq!(cam, (v, 8.0));
}

/// Uma consola antiga guarda o formato à risca (Godot `keep`): a área é a caixa, a cerca não mexe.
#[test]
fn uma_consola_antiga_guarda_as_tarjas() {
    let f = formato(P::GameBoy);
    let canvas = [0.0, 0.0, 1600.0, 972.0];
    assert_eq!(area_do_palco(canvas, f), caixa_do_jogo(canvas, f));
    let lim = CameraLimits {
        min: [-13.0, -1.0],
        max: [17.0, 9.0],
    };
    let (cam, ..) = palco([1600.0, 972.0], f, Some(&lim), [0.0, 3.0]);
    assert_eq!(cam.0, [0.0, 3.0]);
}
