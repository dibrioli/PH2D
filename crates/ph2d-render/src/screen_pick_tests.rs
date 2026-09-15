//! Testes da [`super`] — a lei da composição e a ordem dos canais. ⚠️ A LEITURA da GPU não é
//! alcançável de um teste (precisa de um adaptador); o que se mede aqui é tudo o que ela decide
//! depois de ter os bytes na mão, que é onde os dois defeitos deste módulo estavam.

use super::compose_screen_bytes;

/// ⭐⭐⭐ **Sobre uma SPRITE o chrome é transparente — e é aí que o conta-gotas devolvia
/// `#00000000`.**
///
/// ⛔ Este é o report do dono, reduzido ao seu mecanismo: o canvas não tem chrome por cima, logo a
/// camada que a leitura de reserva lia está VAZIA ali. A cor tem de vir do MUNDO.
#[test]
fn where_there_is_no_chrome_the_colour_is_the_world() {
    // Mundo em B G R A: um vermelho puro guardado como `B=0, G=0, R=255`.
    let vermelho_bgra = [0, 0, 255, 255];
    assert_eq!(
        compose_screen_bytes(vermelho_bgra, [0, 0, 0, 0]),
        [255, 0, 0, 255],
        "sem chrome por cima, a cor do ecrã É a do mundo — e a ordem dos canais tem de ser desfeita"
    );
}

/// ⚠️ **A ordem dos canais é METADE desta porta.** A saída do tonemap é `Bgra8UnormSrgb` e a
/// intermédia do Vello é `Rgba8Unorm`; ler as duas com o mesmo código devolve o vermelho como azul.
#[test]
fn the_world_arrives_with_its_channels_swapped() {
    // B=10, G=20, R=30 ⇒ RGB = (30, 20, 10).
    assert_eq!(
        compose_screen_bytes([10, 20, 30, 255], [0, 0, 0, 0]),
        [30, 20, 10, 255]
    );
}

/// ⭐ **Chrome OPACO tapa o mundo** — é o caso da arte VECTORIAL do documento, que é desenhada no
/// passe do Vello e não no do mundo. *É por isso que o conta-gotas não pode simplesmente trocar de
/// camada: ele precisa das duas.*
#[test]
fn opaque_chrome_occludes_the_world() {
    assert_eq!(
        compose_screen_bytes([255, 255, 255, 255], [10, 20, 30, 255]),
        [10, 20, 30, 255]
    );
}

/// Meia alfa mistura meio a meio, como o `compositor.wgsl` — a mesma conta que o gate irmão da
/// `compositor` já afirma em vírgula flutuante, aqui em bytes.
#[test]
fn half_alpha_mixes_in_the_middle() {
    // Mundo preto, chrome branco a 50 % ⇒ cinzento médio.
    let c = compose_screen_bytes([0, 0, 0, 255], [255, 255, 255, 128]);
    assert!(
        (i32::from(c[0]) - 128).abs() <= 1,
        "meia alfa tem de cair no meio, e deu {c:?}"
    );
}

/// ⛔ **A alfa que sai é SEMPRE opaca.** Devolver a alfa da camada de cima diria *«a cor que viste
/// é meio transparente»* sobre um pixel perfeitamente sólido — e, sobre o canvas, diria ZERO, que
/// é literalmente o `#00000000` do report.
#[test]
fn the_screen_is_opaque_whatever_the_chrome_says() {
    for chrome_a in [0u8, 1, 128, 254, 255] {
        assert_eq!(
            compose_screen_bytes([40, 50, 60, 255], [1, 2, 3, chrome_a])[3],
            255,
            "a alfa do ecrã não é a da camada de cima (chrome_a = {chrome_a})"
        );
    }
}

/// ⭐⭐⭐ **AS DUAS FONTES DO MUNDO guardam os bytes da MESMA maneira** — e é essa suposição que o
/// [`compose_screen_bytes`] faz quando desfaz `B G R A`.
///
/// ⛔⛔ O quadro tem dois modos: o simples mostra a saída do tonemap, o INTERCALADO (ou com o vidro
/// do *Edit Prefab*) mostra o acumulador do mundo. Se um dia uma delas mudar de ordem de canais ou
/// de codificação, o conta-gotas devolveria o **vermelho como azul** em metade dos documentos — e
/// nada no caminho falharia a compilar. *Uma suposição partilhada por duas texturas é um gate.*
#[test]
fn the_two_world_sources_store_their_bytes_the_same_way() {
    assert_eq!(
        crate::Tonemap::OUTPUT_FORMAT,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        "a saída do tonemap deixou de ser BGRA-sRGB e o conta-gotas desfaz os canais à mão"
    );
    assert_eq!(
        crate::WorldRt::SAMPLE_FORMAT,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        "o acumulador do mundo deixou de ser lido como BGRA-sRGB — a outra metade da mesma suposição"
    );
}
