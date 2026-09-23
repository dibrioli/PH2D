//! ⭐⭐ **O FILTRO DO PROJECTO chega às imagens do Motion na cena vectorial** (doc 118 §8).
//!
//! ⚠️ **Por que um gate de FONTE:** a lei (`qualidade_da_imagem`) e o pixel têm gates na família,
//! e os dois entram pelo `encode` com o filtro que o TESTE escolhe. O que nenhum deles vê é qual
//! filtro o QUADRO passa — e com um `ImageFilterMode::Smooth` cravado aqui a suíte da família fica
//! inteira verde enquanto um projecto em `PixelArt` desenha as folhas borradas.

/// **O quadro entrega o filtro do PROJECTO ao `encode`, e não uma constante.**
#[test]
fn the_frame_hands_the_project_filter_to_the_motion_encoder() {
    let s = crate::frame_text::render_frame()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let inicio = s
        .find("ph2d_app_motion::motion_shape_gen::encode(")
        .expect("CONTROLE: o quadro chama o encode do Motion");
    let chamada = &s[inicio..];
    let chamada = &chamada[..chamada.find(");").expect("a chamada fecha")];
    assert!(
        chamada.contains("hero.project.image_filter,"),
        "o encode do Motion tem de receber o filtro do PROJECTO — o MESMO que o \
         `fase_extract_inputs` dá a uma sprite `Inherit`: {chamada}"
    );
    assert!(
        !chamada.contains("ImageFilterMode::"),
        "um filtro CRAVADO no quadro ignora o projecto: {chamada}"
    );
}
