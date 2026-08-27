//! As provas da LEI da máscara de sujidade — a metade que se mede sem uma GPU.
//!
//! ⚠️ **A propriedade nº 1 não é o enquadramento, é o NÃO-SANGRAMENTO.** Uma máscara que vem de
//! uma célula do atlas partilhado amostra uma textura em que os vizinhos são outras sprites, e
//! um UV que saia da célula lê *arte de outra pessoa* — o modo de falha que a célula da folha 11
//! descreveu como *"funciona com umas imagens e falha em silêncio com outras"*. É por isso que
//! ela é afirmada sobre uma VARREDURA e não sobre um exemplo.

use super::{DirtMask, scale_offset};

/// O intervalo de UV que uma tela `screen_aspect` amostra, dado o `scale_offset`.
fn sampled_range(so: [f32; 4]) -> ([f32; 2], [f32; 2]) {
    // O UV de tela varre `[0, 1]` nos dois eixos, então o mínimo é o offset e o
    // máximo é `offset + scale`.
    ([so[2], so[3]], [so[2] + so[0], so[3] + so[1]])
}

#[test]
fn the_cover_never_samples_outside_the_cell_it_was_given() {
    // Aspectos de imagem e de tela de uma ordem de grandeza para cada lado, e três
    // sub-rects: a textura inteira, um quarto no canto, e uma tira fina no meio.
    let rects = [
        [0.0, 0.0, 1.0, 1.0],
        [0.5, 0.5, 0.5, 0.5],
        [0.25, 0.4, 0.125, 0.0625],
    ];
    let aspects = [0.1_f32, 0.5, 0.75, 1.0, 1.3333, 1.7778, 2.35, 10.0];
    let mut checked = 0;
    for r in rects {
        for ia in aspects {
            for sa in aspects {
                let so = scale_offset(r, ia, sa);
                let (lo, hi) = sampled_range(so);
                // Uma folga de um ULP-ish: a conta é `(1−s)/2·w + x`, três operações.
                const EPS: f32 = 1e-6;
                assert!(
                    lo[0] >= r[0] - EPS && lo[1] >= r[1] - EPS,
                    "sangrou por baixo: rect {r:?} ia {ia} sa {sa} -> {so:?}"
                );
                assert!(
                    hi[0] <= r[0] + r[2] + EPS && hi[1] <= r[1] + r[3] + EPS,
                    "sangrou por cima: rect {r:?} ia {ia} sa {sa} -> {so:?}"
                );
                checked += 1;
            }
        }
    }
    // Controle positivo: a varredura correu mesmo (uma lista vazia passa em silêncio).
    assert_eq!(checked, 3 * 8 * 8);
}

#[test]
fn an_image_with_the_screens_aspect_uses_the_whole_cell() {
    // A identidade do enquadramento: mesmo aspecto ⇒ nada é cortado, e o
    // `scale_offset` é literalmente o sub-rect.
    for r in [[0.0, 0.0, 1.0, 1.0], [0.25, 0.5, 0.5, 0.25]] {
        let so = scale_offset(r, 1.7778, 1.7778);
        assert_eq!(so, [r[2], r[3], r[0], r[1]], "rect {r:?}");
    }
}

#[test]
fn the_wider_image_is_cropped_across_and_the_taller_one_down() {
    // Tela 16:9. Uma imagem 2:1 é MAIS larga ⇒ encosta em altura e perde largura.
    let wide = scale_offset([0.0, 0.0, 1.0, 1.0], 2.0, 16.0 / 9.0);
    assert!(
        wide[0] < 1.0,
        "a mais larga tinha de perder largura: {wide:?}"
    );
    assert_eq!(wide[1], 1.0, "e manter a altura inteira: {wide:?}");
    assert!(wide[2] > 0.0 && wide[3] == 0.0, "e centrar em x: {wide:?}");
    // Uma imagem 1:1 é MAIS ALTA que 16:9 ⇒ o contrário.
    let tall = scale_offset([0.0, 0.0, 1.0, 1.0], 1.0, 16.0 / 9.0);
    assert_eq!(
        tall[0], 1.0,
        "a mais alta tinha de manter a largura: {tall:?}"
    );
    assert!(tall[1] < 1.0, "e perder altura: {tall:?}");
    assert!(tall[3] > 0.0 && tall[2] == 0.0, "e centrar em y: {tall:?}");
}

#[test]
fn the_crop_is_centred_not_anchored() {
    // A metade que uma implementação que esquece o `·0,5` perde: a sobra tem de ficar
    // repartida pelos DOIS lados. Com a origem ancorada, `offset` seria `0`.
    let so = scale_offset([0.0, 0.0, 1.0, 1.0], 4.0, 1.0);
    let (lo, hi) = sampled_range(so);
    assert!(
        (lo[0] - (1.0 - hi[0])).abs() < 1e-6,
        "sobra assimétrica: {so:?}"
    );
    assert!(lo[0] > 0.0, "sem sobra nenhuma nao houve corte: {so:?}");
}

#[test]
fn a_junk_aspect_is_neutral_never_a_nan() {
    for (ia, sa) in [
        (f32::NAN, 1.0),
        (1.0, f32::NAN),
        (f32::INFINITY, 1.0),
        (0.0, 1.0),
        (1.0, 0.0),
        (-2.0, 1.0),
    ] {
        let so = scale_offset([0.25, 0.5, 0.5, 0.25], ia, sa);
        assert_eq!(so, [0.0, 0.0, 0.0, 0.0], "ia {ia} sa {sa}");
        assert!(so.iter().all(|v| v.is_finite()));
    }
}

/// ⚠️ **A identidade do quadro NÃO vive no `scale_offset`** — ela vive na textura preta de 1×1,
/// e este gate é o que diz onde ela vive. O neutro do enquadramento amostra o texel `(0,0)`; é
/// o CONTEÚDO daquele texel que garante que um `dirt_intensity` alto não muda o quadro quando
/// não há imagem escolhida.
#[test]
fn the_neutral_framing_is_not_what_makes_the_frame_identical() {
    let so = scale_offset([0.0, 0.0, 1.0, 1.0], f32::NAN, 1.0);
    let (lo, hi) = sampled_range(so);
    assert_eq!(lo, [0.0, 0.0]);
    assert_eq!(
        hi,
        [0.0, 0.0],
        "todo pixel le^ o MESMO texel — e ele e' preto"
    );
}

/// O tipo carrega o que o composite precisa e nada mais — um censo barato contra a
/// deriva de alguém que lhe acrescente estado de passe.
#[test]
fn the_mask_is_four_facts_and_a_view() {
    fn takes(m: DirtMask<'_>) -> ([f32; 4], f32, u64) {
        (m.uv_rect, m.aspect, m.key)
    }
    let _ = takes;
}
