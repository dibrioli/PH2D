//! Os gates do [`super::rect_no_ecra`] — a régua é o PIXEL, e a fixtura é aritmética fechada.

use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, Sprite};

/// Uma janela `1600×900` com a câmera na origem a abranger `9` metros de altura ⇒ **`100` px por
/// metro**, que é o que torna toda a tabela abaixo conferível à mão.
const JANELA: WindowSize = WindowSize {
    width: 1600,
    height: 900,
};

fn camera() -> Camera2d {
    Camera2d {
        center: [0.0, 0.0],
        height_world: 9.0,
        ..Camera2d::default()
    }
}

fn sprite(size: [f32; 2]) -> Sprite {
    Sprite {
        size,
        ..Sprite::atlas(0, size, [1.0; 4])
    }
}

/// ⭐⭐⭐ **A LEI, com a conta fechada à mão:** um sprite de `2×1` m na origem, a `100` px/m, mede
/// `200×100` px e fica centrado na janela — `x = 800 − 100`, `y = 450 − 50`.
#[test]
fn um_sprite_na_origem_cai_no_centro_da_janela() {
    let r = super::rect_no_ecra(
        ph2d_ecs::Transform::default(),
        &sprite([2.0, 1.0]),
        None,
        &camera(),
        JANELA,
    );
    for (i, (lido, esperado)) in r.iter().zip([700.0, 400.0, 200.0, 100.0]).enumerate() {
        assert!(
            (lido - esperado).abs() < 1e-3,
            "componente {i}: {lido} contra {esperado} (rect {r:?})"
        );
    }
}

/// ⚠️ **Mover o sprite move o rectângulo, e o `y` é INVERTIDO** — o mundo tem `y` para cima e a
/// janela para baixo. *Um sinal trocado aqui poria o assado de cabeça para baixo e nada na
/// aritmética o diria.*
#[test]
fn o_y_do_mundo_sobe_e_o_da_janela_desce() {
    let mut tr = ph2d_ecs::Transform::default();
    tr.translation.x = 1.0;
    tr.translation.y = 1.0;
    let r = super::rect_no_ecra(tr, &sprite([2.0, 1.0]), None, &camera(), JANELA);
    assert!(
        (r[0] - 800.0).abs() < 1e-3,
        "andou 1 m = 100 px para a direita: {r:?}"
    );
    assert!(
        (r[1] - 300.0).abs() < 1e-3,
        "e 100 px para CIMA na janela: {r:?}"
    );
}

/// ⚠️ **A ESCALA do `Transform` entra** — ela multiplica o quad, e esquecê-la faria o assado sair à
/// escala de repouso de uma peça que o artista ampliou.
#[test]
fn a_escala_do_transform_entra_no_rectangulo() {
    let mut tr = ph2d_ecs::Transform::default();
    tr.scale.x = 3.0;
    tr.scale.y = 0.5;
    let r = super::rect_no_ecra(tr, &sprite([2.0, 1.0]), None, &camera(), JANELA);
    assert!((r[2] - 600.0).abs() < 1e-3, "largura: {r:?}");
    assert!((r[3] - 50.0).abs() < 1e-3, "altura: {r:?}");
}

/// ⭐⭐ **Numa FOLHA o quad DESDOBRA-SE, e é a mesma lei do afim de que este rect sai** — ver o
/// [`super::unfolded_quad`]: o contrato é *«a imagem INTEIRA sobre o quad deste sprite»*, e numa
/// grelha a imagem inteira é a folha toda.
///
/// ⚠️ Sem esta metade, um sprite com grelha seria enquadrado pelo rectângulo de UMA célula e o
/// assado sairia `hframes` vezes pequeno — o defeito que o report de 2026-08-23 já pagou noutra
/// superfície.
#[test]
fn uma_folha_mede_a_folha_inteira() {
    let grid = ph2d_ecs::SpriteGrid {
        hframes: 4,
        vframes: 2,
        ..ph2d_ecs::SpriteGrid::default()
    };
    let uma = super::rect_no_ecra(
        ph2d_ecs::Transform::default(),
        &sprite([2.0, 1.0]),
        None,
        &camera(),
        JANELA,
    );
    let folha = super::rect_no_ecra(
        ph2d_ecs::Transform::default(),
        &sprite([2.0, 1.0]),
        Some(grid),
        &camera(),
        JANELA,
    );
    assert!(
        (folha[2] - uma[2] * 4.0).abs() < 1e-3 && (folha[3] - uma[3] * 2.0).abs() < 1e-3,
        "a folha tem de medir 4x2 celulas: {folha:?} contra {uma:?}"
    );
}

/// ⚠️ **Com rotação ele é a CAIXA, e a divergência é DECLARADA** — um quadrado rodado `45°` tem
/// caixa `√2` vezes maior, e o gate EXIGE que ela exista: sem isto alguém leria *«o rect é o
/// quad»* como lei e assaria uma peça rodada no sítio errado sem nada a acusar.
#[test]
fn com_rotacao_o_rect_e_a_caixa_e_isso_e_declarado() {
    let tr = ph2d_ecs::Transform {
        rotation: core::f32::consts::FRAC_PI_4,
        ..Default::default()
    };
    let r = super::rect_no_ecra(tr, &sprite([1.0, 1.0]), None, &camera(), JANELA);
    let lado = 100.0 * core::f32::consts::SQRT_2;
    assert!(
        (r[2] - lado).abs() < 1e-2 && (r[3] - lado).abs() < 1e-2,
        "a caixa de um quadrado a 45 graus mede lado x raiz de dois: {r:?}"
    );
}
