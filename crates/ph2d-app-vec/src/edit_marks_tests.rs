//! **O fio do caminho** (F30 §E) — o que se vê de um caminho cuja tinta não se vê.
//!
//! Sem rasterizador de CPU na crate, o oráculo é o buffer do Vello: o fio é um `stroke` na tinta do
//! token `accent` (que não é transparente nem a cor do canvas — o «pixel ≠ fundo»), e os pontos dele,
//! em px de ecrã, cobrem o caminho.

use super::*;
use ph2d_tokens::ColorToken;
use ph2d_vec_edit::pencil_width::PenDynamics;
use ph2d_vec_scene::{Rgba8, VecScene, VecViewState, VecXforms, rectangle};

/// O `draw_data` de um traço no acento — a «cor» que o fio tem de deixar no buffer.
fn tinta_do_fio(theme: Theme) -> u32 {
    let mut s = VectorScene::new();
    let mut l = ph2d_vector::BezPath::new();
    l.move_to((0.0, 0.0));
    l.line_to((1.0, 0.0));
    // O oráculo fixa o ACENTO — nunca a constante do produto, que se moveria junto (mutação 6, F30).
    let ink = ph2d_vec_render::token_ink(ColorToken::Accent, theme);
    ph2d_vec_render::stroke_screen(&l, ink, ph2d_vec_render::WIRE_PX, &mut s);
    s.inner().encoding().draw_data[0]
}

/// Pinta o fio e devolve (o buffer tem a tinta do fio?, a caixa dos pontos em px de ecrã).
fn pinta(
    vec: &VecState,
    scene: &VecScene,
    cam: Affine,
    cursor: Option<[f64; 2]>,
) -> (bool, [f32; 4]) {
    let theme = Theme::default();
    let mut alvo = VectorScene::new();
    let (view, xf) = (VecViewState::default(), VecXforms::new());
    draw_wires(vec, scene, &view, &xf, cam, cursor, theme, &mut alvo);
    let e = alvo.inner().encoding();
    let tem = e.draw_data.contains(&tinta_do_fio(theme));
    let f: Vec<f32> = e.path_data.iter().map(|&b| f32::from_bits(b)).collect();
    let mut caixa = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    for p in f.chunks_exact(2) {
        caixa = [
            caixa[0].min(p[0]),
            caixa[1].min(p[1]),
            caixa[2].max(p[0]),
            caixa[3].max(p[1]),
        ];
    }
    (tem, caixa)
}

fn estilo_invisivel(vec: &mut VecState) {
    let mut st = vec.pen.style();
    st.stroke = Rgba8::new(0, 0, 0, 0);
    st.stroke_w = 0.0;
    st.fill = Rgba8::new(0, 0, 0, 0);
    vec.pen.set_style(st);
    vec.pencil.set_style(st);
}

const MAO: PenDynamics = PenDynamics {
    pressure: 1.0,
    t_ns: 0,
};

/// ⭐ **Width 0 e alpha 0: a meio do gesto, o caminho vê-se** — Caneta e Lápis.
#[test]
fn um_caminho_sem_traco_nem_preenchimento_ve_se_enquanto_se_desenha() {
    let theme = Theme::default();
    let tinta = ColorToken::Accent.resolve(theme);
    assert!(tinta.a > 0, "o fio é transparente");
    assert_ne!(
        tinta,
        ColorToken::Canvas.resolve(theme),
        "o fio tem a cor do fundo"
    );

    // CONTROLO: sem gesto e sem seleção, nada se pinta — o que o dono via.
    let mut cena = VecScene::new();
    let mut vec = VecState::default();
    estilo_invisivel(&mut vec);
    vec.draw_config.mode = DrawMode::Pen;
    assert!(
        !pinta(&vec, &cena, Affine::IDENTITY, None).0,
        "controlo: pintou sem caminho"
    );

    // CANETA: dois cliques; a mão solta, o cursor mais abaixo.
    vec.pen
        .on_press(&mut cena, [10.0, 10.0], 1.0, false, &mut |p| p);
    vec.pen.on_release();
    vec.pen
        .on_press(&mut cena, [90.0, 10.0], 1.0, false, &mut |p| p);
    vec.pen.on_release();
    let id = vec
        .pen
        .active_path()
        .expect("a caneta está a meio de um caminho");
    let p = cena.path(id).expect("o caminho vivo");
    assert!(
        p.stroke.as_ref().is_none_or(|s| s.width == 0.0),
        "o traço devia ter largura 0"
    );
    let (tem, c) = pinta(&vec, &cena, Affine::IDENTITY, None);
    assert!(
        tem,
        "a caneta a meio de um caminho invisível não pintou o fio"
    );
    assert!(
        c[0] <= 10.5 && c[2] >= 89.5,
        "o fio não cobre o caminho: {c:?}"
    );
    // E o segmento até ao cursor (o rubber band) chega ao cursor.
    let (_, c) = pinta(&vec, &cena, Affine::IDENTITY, Some([90.0, 70.0]));
    assert!(
        c[3] >= 69.5,
        "o segmento até ao cursor não chegou lá: {c:?}"
    );

    // LÁPIS: um traço a meio, pela câmara (×2).
    let mut cena = VecScene::new();
    let mut vec = VecState::default();
    estilo_invisivel(&mut vec);
    vec.draw_config.mode = DrawMode::Pencil;
    vec.pencil.on_press(&mut cena, [0.0, 0.0], 1.0, MAO);
    for i in 1..=20 {
        let x = f64::from(i) * 3.0;
        vec.pencil
            .on_drag(&mut cena, [x, (x * 0.1).sin() * 10.0], MAO);
    }
    assert!(
        vec.pencil.active_path().is_some(),
        "o lápis está a meio do traço"
    );
    let (tem, c) = pinta(&vec, &cena, Affine::scale(2.0), None);
    assert!(tem, "o lápis a meio de um traço invisível não pintou o fio");
    assert!(
        c[0] <= 0.5 && c[2] >= 119.5,
        "o fio não atravessou a câmara: {c:?}"
    );
}

/// ⭐ **A forma seleccionada mostra o fio** (o Figma/Illustrator) — e a não-seleccionada não.
#[test]
fn a_forma_seleccionada_mostra_o_fio() {
    let mut cena = VecScene::new();
    let id = cena.push_path(rectangle([0.0, 0.0], [10.0, 10.0]));
    let mut vec = VecState::default();
    vec.draw_config.mode = DrawMode::Select;
    assert!(
        !pinta(&vec, &cena, Affine::IDENTITY, None).0,
        "controlo: a forma não seleccionada também ganhou fio"
    );
    vec.pen.select(Some(id));
    let (tem, c) = pinta(&vec, &cena, Affine::scale(10.0), None);
    assert!(tem, "a forma seleccionada não mostra o fio");
    assert!(
        c[0] <= 0.5 && c[1] <= 0.5 && c[2] >= 99.5 && c[3] >= 99.5,
        "o fio não contorna a forma: {c:?}"
    );
}
