//! **O INSTRUMENTO DAS TRÊS PORTAS** — o que a sonda `PH2D_PAN_DIAG` não conseguia ser.
//!
//! A sonda comparava a rota das sprites com a rota do Vello e imprimia `Δpx = 0,000` sobre
//! um defeito visível. A auditoria de 2026-08-26 mostrou porquê: aquelas duas são a **mesma
//! expressão** (`x = W/2 + (X−cx)·Hs/hw` termo a termo), então a igualdade delas é uma
//! identidade algébrica, não uma medição — nenhuma máquina, nenhum `t` e nenhum centro de
//! câmera pode fazê-la imprimir outra coisa. *Uma sonda que só pode imprimir zero não é uma
//! sonda.*
//!
//! ⚠️ **Havia uma TERCEIRA rota raster, e é a que quebrava**: o passe do Flip, que nunca
//! recebeu o sub-retângulo da cena e projetava a janela CHEIA. Sob o split isso é `1/t ≈
//! 1,82×` de escala — e como o pan converte o arrasto pela altura DA CENA, o erro **não é um
//! offset: ele multiplica a distância arrastada**. Parada a câmera, é um desalinhamento fixo;
//! a arrastar, é uma abertura que cresce sem teto e só fecha ao voltar. Foi isso que o Enio
//! reportou como *"a imagem de referência sofre um drift no pan"*.
//!
//! Por isso este gate mede **três** portas, cada uma pela aritmética que o PRODUTO usa, e
//! mede-as **em dois centros de câmera** — porque *drift* é a diferença entre dois instantes,
//! e nenhum quadro sozinho pode responder por ela.

use super::*;

/// O alvo. Números do report do Enio (a janela pequena, onde o efeito era maior).
const TW: u32 = 1024;
const TH: u32 = 768;
/// A fração do split da tool Motion. ⚠️ **Com `floor`** — a cena é um número INTEIRO de
/// pixels desde o Bug #9, e é esse inteiro que as três portas têm de partilhar.
const SPLIT_T: f32 = 0.55;

fn target() -> WindowSize {
    WindowSize::new(TW, TH)
}

/// O sub-retângulo horizontal (a cena em cima, o grafo em baixo) — a mesma conta de
/// [`ph2d_editor::screens::layout::CenterSplit::scene_viewport`], escrita aqui à mão de
/// propósito: se aquela porta mudar, este gate tem de reprovar, não de acompanhar.
fn horizontal_subrect() -> [f32; 4] {
    [0.0, 0.0, TW as f32, (TH as f32 * SPLIT_T).floor()]
}

fn vertical_subrect() -> [f32; 4] {
    [0.0, 0.0, (TW as f32 * SPLIT_T).floor(), TH as f32]
}

/// NDC (y-up, como o `orthographic_rh` emite) → pixel do ALVO, y-down, dentro do rect dado.
fn ndc_to_px(ndc: [f64; 2], rect: [f32; 4]) -> [f64; 2] {
    let [x0, y0, w, h] = rect.map(f64::from);
    [x0 + (ndc[0] * 0.5 + 0.5) * w, y0 + (0.5 - ndc[1] * 0.5) * h]
}

/// Aplica uma matriz col-major (`m[col][row]`) a um ponto de mundo 2D.
fn apply(m: &[[f32; 4]; 4], p: [f32; 2]) -> [f64; 2] {
    let x = m[0][0] * p[0] + m[1][0] * p[1] + m[3][0];
    let y = m[0][1] * p[0] + m[1][1] * p[1] + m[3][1];
    [f64::from(x), f64::from(y)]
}

/// **PORTA 1 — o passe de SPRITES** (`Chip`, as cópias carimbadas):
/// `uniform_for_subrect(w,h)` no uniform + `set_viewport(x,y,w,h)` no passe.
fn sprite_px(cam: &Camera2d, sub: [f32; 4], p: [f32; 2]) -> [f64; 2] {
    let m = cam.view_proj_for_subrect(sub[2], sub[3]).to_cols_array_2d();
    ndc_to_px(apply(&m, p), sub)
}

/// **PORTA 2 — o VELLO** (`Star`): o afim mundo→tela montado com as dims DA CENA, aplicado na
/// CPU em `f64`, num alvo de janela cheia (o conteúdo cai no canto, sem recorte).
fn vector_px(cam: &Camera2d, sub: [f32; 4], p: [f32; 2]) -> [f64; 2] {
    let c = cam
        .world_to_screen_affine(WindowSize::new(sub[2] as u32, sub[3] as u32))
        .as_coeffs();
    let (x, y) = (f64::from(p[0]), f64::from(p[1]));
    [
        c[0] * x + c[2] * y + c[4] + f64::from(sub[0]),
        c[1] * x + c[3] * y + c[5] + f64::from(sub[1]),
    ]
}

/// **PORTA 3 — o passe do FLIP** (`Object`, a arte de quatro cores): a câmera que este módulo
/// constrói, lida como o shader a lê — NDC do alvo INTEIRO, porque é nele que o passe compõe.
fn flip_px(cam: &Camera2d, window: WindowSize, sub: Option<[f32; 4]>, p: [f32; 2]) -> [f64; 2] {
    let c = camera_scene(cam, window, sub);
    ndc_to_px(
        apply(&c.world_to_clip, p),
        [0.0, 0.0, c.viewport[0], c.viewport[1]],
    )
}

/// A barra. As três portas são a MESMA conta em precisões diferentes (`f32` na matriz das duas
/// raster, `f64` no afim do Vello), então o resíduo é de arredondamento — milésimos de pixel.
/// ⚠️ Não é um `< ε` complacente: o defeito que este gate existe para apanhar vale **121 px**
/// nestes números.
const BAR_PX: f64 = 1e-3;

fn assert_close(what: &str, a: [f64; 2], b: [f64; 2]) {
    assert!(
        (a[0] - b[0]).abs() <= BAR_PX && (a[1] - b[1]).abs() <= BAR_PX,
        "{what}: {a:?} contra {b:?} (barra {BAR_PX} px)"
    );
}

/// Os pontos de prova: a origem (ponto FIXO de toda a família quando a câmera está centrada —
/// é ele que faz um erro de ESCALA puro passar despercebido) e quatro pontos fora dela.
const PROBES: [[f32; 2]; 5] = [
    [0.0, 0.0],
    [1.6, 0.0],
    [0.0, -2.4],
    [-3.1, 1.7],
    [4.25, 4.25],
];

#[test]
fn the_three_doors_put_the_same_world_point_on_the_same_pixel() {
    for sub in [horizontal_subrect(), vertical_subrect()] {
        for center in [[0.0, 0.0], [-2.9, 5.6]] {
            let cam = Camera2d::new(center, 10.0);
            for p in PROBES {
                let s = sprite_px(&cam, sub, p);
                assert_close("sprites vs vetor", s, vector_px(&cam, sub, p));
                assert_close("sprites vs flip", s, flip_px(&cam, target(), Some(sub), p));
            }
        }
    }
}

/// ⭐ **O GATE DO DRIFT.** Um offset constante é um desalinhamento; o que o Enio vê é a
/// separação a CRESCER com o arrasto, e isso só se mede movendo a câmera e comparando os
/// **deslocamentos**. Uma porta com outra escala px/mundo entrega outro deslocamento para o
/// mesmo pan — e é exatamente isso que o passe do Flip fazia (`H/floor(H·t) = 1,82×`).
#[test]
fn the_three_doors_move_by_the_same_pixels_when_the_camera_pans() {
    for sub in [horizontal_subrect(), vertical_subrect()] {
        for pan in [[3.5, 0.0], [0.0, -2.25], [-7.0, 4.5]] {
            let a = Camera2d::new([0.0, 0.0], 10.0);
            let b = Camera2d::new(pan, 10.0);
            for p in PROBES {
                let d_sprite = delta(sprite_px(&a, sub, p), sprite_px(&b, sub, p));
                let d_vector = delta(vector_px(&a, sub, p), vector_px(&b, sub, p));
                let d_flip = delta(
                    flip_px(&a, target(), Some(sub), p),
                    flip_px(&b, target(), Some(sub), p),
                );
                assert_close("deslocamento sprites vs vetor", d_sprite, d_vector);
                assert_close("deslocamento sprites vs flip", d_sprite, d_flip);
            }
        }
    }
}

fn delta(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [b[0] - a[0], b[1] - a[1]]
}

/// O CONTROLE. Fora do split o passe do Flip tem de ser o que shipava — **ao bit**, não
/// «parecido»: é o caminho comum de toda sessão que não está na tool Motion.
#[test]
fn without_a_split_the_flip_camera_is_the_one_that_shipped() {
    let cam = Camera2d::new([-2.9, 5.6], 7.25);
    let old = camera_raw(&cam, target());
    let new = camera_scene(&cam, target(), None);
    assert_eq!(new.world_to_clip, old.world_to_clip);
    assert_eq!(new.viewport, old.viewport);
    assert_eq!(new.px_per_world, old.px_per_world);
}

/// O segundo controle: um sub-retângulo que É a janela inteira não pode mover nada. Separa
/// «o remapeamento está certo» de «o remapeamento é inerte» — sem ele, um `camera_scene` que
/// devolvesse `camera_raw` para TODO caso passaria no teste acima.
#[test]
fn a_full_window_subrect_is_the_no_split_camera() {
    let cam = Camera2d::new([1.5, -3.25], 12.0);
    let full = [0.0, 0.0, TW as f32, TH as f32];
    for p in PROBES {
        assert_close(
            "sub-retângulo cheio",
            flip_px(&cam, target(), Some(full), p),
            flip_px(&cam, target(), None, p),
        );
    }
}

/// A escala do TRAÇO acompanha a da geometria. O `px_per_world` é o que dá a espessura em
/// pixels; se ele ficasse na janela cheia enquanto a geometria passou para a cena, o traço
/// engrossaria `1/t` — o mesmo defeito, noutra grandeza.
#[test]
fn the_stroke_scale_follows_the_scene_not_the_window() {
    let cam = Camera2d::new([0.0, 0.0], 10.0);
    let sub = horizontal_subrect();
    let c = camera_scene(&cam, target(), Some(sub));
    let expected = sub[3] / cam.height_world;
    assert!(
        (c.px_per_world - expected).abs() <= 1e-4,
        "px_per_world {} contra {expected} (a altura da CENA, não a da janela)",
        c.px_per_world
    );
    assert!(
        c.px_per_world < camera_raw(&cam, target()).px_per_world,
        "sob o split a cena é mais baixa que a janela"
    );
}
