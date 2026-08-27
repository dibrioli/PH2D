//! As provas da LEI da máscara de sujidade — a metade que se mede sem uma GPU.
//!
//! ⚠️ **A propriedade nº 1 não é o enquadramento, é o NÃO-SANGRAMENTO.** Uma máscara que vem de
//! uma célula do atlas partilhado amostra uma textura em que os vizinhos são outras sprites, e
//! um UV que saia da célula lê *arte de outra pessoa* — o modo de falha que a célula da folha 11
//! descreveu como *"funciona com umas imagens e falha em silêncio com outras"*. É por isso que
//! ela é afirmada sobre uma VARREDURA e não sobre um exemplo.

use super::{DirtMask, scale_offset};
use crate::AtlasRegion;

/// O intervalo de UV que uma tela `screen_aspect` amostra, dado o `scale_offset`.
fn sampled_range(so: [f32; 4]) -> ([f32; 2], [f32; 2]) {
    // O UV de tela varre `[0, 1]` nos dois eixos, então o mínimo é o offset e o
    // máximo é `offset + scale`.
    ([so[2], so[3]], [so[2] + so[0], so[3] + so[1]])
}

/// **As células de teste são DERIVADAS da função que as produz** ([`AtlasRegion::uv`]), nunca
/// escritas à mão.
///
/// ⚠️ **É a correcção que este ficheiro pagou caro.** A 1.ª versão inventava rects na convenção
/// `[x, y, largura, altura]`; a da casa é `[u0, v0, u1, v1]` (os dois CANTOS). O código lia-os
/// da mesma maneira errada, então **todos** estes gates passavam sobre uma feature que não fazia
/// nada — a máscara amostrava fora da célula, no vazio preto do átlas. As outras duas fontes de
/// textura devolvem `[0, 0, 1, 1]`, que é idêntico nas duas convenções, então só a célula de
/// átlas podia expor o defeito, e era justamente ela que a fixtura fabricava.
fn cell(x: u32, y: u32, w: u32, h: u32, sheet: u32) -> [f32; 4] {
    AtlasRegion { x, y, w, h }.uv(sheet)
}

#[test]
fn the_cover_never_samples_outside_the_cell_it_was_given() {
    // Aspectos de imagem e de tela de uma ordem de grandeza para cada lado, e três
    // sub-rects: a textura inteira, um quarto no canto, e uma tira fina no meio.
    const SHEET: u32 = 4096;
    let rects = [
        // A textura inteira (o que `Individual` / `CookedTexture` devolvem).
        [0.0, 0.0, 1.0, 1.0],
        // Um quarto no canto, uma célula quadrada, e uma tira fina — as três DERIVADAS.
        cell(2048, 2048, 2048, 2048, SHEET),
        cell(1088, 0, 256, 256, SHEET),
        cell(1024, 1638, 512, 256, SHEET),
    ];
    let aspects = [0.1_f32, 0.5, 0.75, 1.0, 1.3333, 1.7778, 2.35, 10.0];
    let mut checked = 0;
    for r in rects {
        for ia in aspects {
            for sa in aspects {
                let so = scale_offset(r, ia, sa);
                let (lo, hi) = sampled_range(so);
                // ⚠️ **`r` é `[u0, v0, u1, v1]`**, então a célula é `[r0, r2] × [r1, r3]` —
                // e é exactamente esta linha que a versão errada escrevia como `r[0]+r[2]`.
                const EPS: f32 = 1e-6;
                assert!(
                    lo[0] >= r[0] - EPS && lo[1] >= r[1] - EPS,
                    "sangrou por baixo: rect {r:?} ia {ia} sa {sa} -> {so:?}"
                );
                assert!(
                    hi[0] <= r[2] + EPS && hi[1] <= r[3] + EPS,
                    "sangrou por cima: rect {r:?} ia {ia} sa {sa} -> {so:?}"
                );
                // ⚠️ **E o CONTROLE que faltava, que é a definição de `cover`: UM dos eixos é
                // consumido por INTEIRO.** Sem ele, a contenção acima é passada com louvor por um
                // `scale_offset` que devolvesse o neutro — e foi *exactamente* assim que a versão
                // errada desta lei ficou verde enquanto a máscara não pintava nada. O outro eixo
                // encolhe pela razão dos aspectos, que é o corte, e nunca chega a zero.
                let (cw, ch) = (r[2] - r[0], r[3] - r[1]);
                let (gw, gh) = (hi[0] - lo[0], hi[1] - lo[1]);
                assert!(
                    (gw - cw).abs() <= cw * 1e-3 || (gh - ch).abs() <= ch * 1e-3,
                    "cover nao encheu eixo nenhum: rect {r:?} ia {ia} sa {sa} -> {so:?}"
                );
                assert!(
                    gw > 0.0 && gh > 0.0,
                    "a mascara colapsou: rect {r:?} ia {ia} sa {sa} -> {so:?}"
                );
                checked += 1;
            }
        }
    }
    // Controle positivo: a varredura correu mesmo (uma lista vazia passa em silêncio).
    assert_eq!(checked, 4 * 8 * 8);
}

#[test]
fn an_image_with_the_screens_aspect_uses_the_whole_cell() {
    // A identidade do enquadramento: mesmo aspecto ⇒ nada é cortado, e o
    // `scale_offset` é literalmente o sub-rect.
    for r in [[0.0, 0.0, 1.0, 1.0], cell(512, 1024, 640, 360, 4096)] {
        let so = scale_offset(r, 1.7778, 1.7778);
        // `scale` = a LARGURA da célula (`u1 − u0`), `offset` = o canto (`u0`).
        assert_eq!(so, [r[2] - r[0], r[3] - r[1], r[0], r[1]], "rect {r:?}");
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
        let so = scale_offset(cell(256, 512, 128, 128, 4096), ia, sa);
        assert_eq!(so, [0.0, 0.0, 0.0, 0.0], "ia {ia} sa {sa}");
        assert!(so.iter().all(|v| v.is_finite()));
    }
    // E um rect INVERTIDO (`u1 < u0`) — o que a leitura errada de `[x, y, w, h]` produz sobre
    // uma célula real. Ele é neutro, nunca uma amostragem ao contrário.
    assert_eq!(
        scale_offset([0.5, 0.5, 0.25, 0.25], 1.0, 1.0),
        [0.0, 0.0, 0.0, 0.0]
    );
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
