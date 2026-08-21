//! `PH2D_DITHER_SMOKE` — **as faixas, e a cura**, lado a lado.
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Sprite \
//!   && env PH2D_DITHER_SMOKE=1 cargo run -p ph2d-host-desktop --release
//! ```
//!
//! Duas sprites nascem lado a lado, feitas do **mesmo** degradê de 16 bits e descidas para 8 bits
//! pelos **dois** caminhos:
//!
//! | | esquerda | direita |
//! |---|---|---|
//! | porta | `rgba16_to_rgba8` (fiel) | `rgba16_to_rgba8_dithered` (W6.1) |
//! | o que se vê | **faixas** verticais duras | um degradê liso |
//!
//! # Por que a fixture é este degradê e não outro
//!
//! ⚠️ **Ela atravessa poucos códigos de propósito.** O degradê vai de sRGB 140 a 146 — **seis**
//! códigos ao longo de 384 pixels, ou seja ~64 px por faixa. Um degradê que atravessasse 200 códigos
//! teria faixas de 2 px e ninguém as veria; a banda é um defeito de **degradês lentos**, e uma
//! fixture rápida esconderia exatamente o fenómeno que ela devia conter.
//!
//! ⚠️ **É cinzento neutro**, também de propósito: o dither aplica o mesmo viés aos três canais, e se
//! algum dia ele passar a ser por-canal a direita ganha franjas de cor — o que numa fixture colorida
//! passaria por «é o degradê».
//!
//! # O que este smoke NÃO mostra, e é honesto dizê-lo
//!
//! ⛔ **Não há uma terceira sprite «RGBA16» para comparar.** Ela mostraria faixas *na mesma*: o ecrã
//! é de 8 bits e a descida final **não** leva dither — foi medida e recusada (W6.2, folga de
//! 0,0283 LSB numa placa em que meio passo precisa de 0,431). Uma sprite dessas ao lado destas duas
//! ensinaria a coisa errada: que 16 bits não serve para nada. O que ele serve é para **não
//! requantizar entre edições**, e isso não se vê num ecrã — vê-se ao fim de cinco ferramentas.
//!
//! ⚠️ **Zoom.** As duas sprites entram selecionadas e enquadradas. A leitura honesta é a 1:1 ou
//! acima; muito afastado, o filtro bilinear alisa as faixas da esquerda e as duas parecem iguais.

use ph2d_asset::AssetDb;
use ph2d_core::Vec2;
use ph2d_ecs::SimWorld;
use ph2d_render::SpriteRenderer;

/// Largura do degradê, em pixels. Larga o suficiente para as seis faixas terem ~64 px cada.
const W: u32 = 384;
/// Altura — só precisa de ser alta o suficiente para as faixas se lerem como faixas.
const H: u32 = 256;
/// Os dois extremos do degradê, em código sRGB. **Seis** códigos: ver o cabeçalho.
const LO: f32 = 140.0;
const HI: f32 = 146.0;

pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_DITHER_SMOKE").is_some()
}

/// O degradê em meios-floats lineares — a fonte de 16 bits de que as duas sprites descem.
fn gradient_16() -> Vec<u16> {
    let one = ph2d_color::f32_to_half(1.0);
    let mut out = Vec::with_capacity((W * H * 4) as usize);
    for _ in 0..H {
        for x in 0..W {
            let t = x as f32 / (W - 1) as f32;
            let srgb_unit = (LO + t * (HI - LO)) / 255.0;
            let half = ph2d_color::f32_to_half(ph2d_color::srgb::srgb_to_linear_unit(srgb_unit));
            out.extend_from_slice(&[half, half, half, one]);
        }
    }
    out
}

/// Sobe uma imagem de 8 bits como sprite de textura própria e devolve os bits da entidade.
///
/// ⚠️ **`Individual` e não atlas**, porque estas duas sprites existem para serem comparadas pixel a
/// pixel: uma passagem pelo atlas partilhado é mais um sítio onde os bytes podiam mudar, e o smoke
/// ficaria a medir o atlas em vez do dither.
fn spawn_8bit(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    pixels: Vec<u8>,
    centre: Vec2,
    world: [f32; 2],
    label: &str,
) -> Option<u64> {
    let texture_id = match renderer.acquire_individual(W, H, &pixels) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("[dither-smoke] `{label}` nao subiu para a GPU: {e}");
            return None;
        }
    };
    let pixels_id = asset_db.insert_image_rgba8(W, H, pixels);
    let (_, bits) = crate::image_import::spawn_sprite(
        sim,
        crate::image_import::PackedSource::Individual {
            texture_id,
            pixels_id,
        },
        centre,
        world,
        label,
    );
    Some(bits)
}

/// Monta a cena. Devolve os bits da sprite **com** dither, para o host a selecionar — é a que
/// responde à pergunta.
pub(crate) fn spawn_if_enabled(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    pixels_per_meter: f32,
) -> Option<u64> {
    let ppm = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    let world = [W as f32 / ppm, H as f32 / ppm];
    let gap = world[0] * 0.08;
    let halves = gradient_16();

    // ⚠️ A MESMA fonte para as duas: se cada uma gerasse o seu degradê, a comparação passaria a ser
    // entre dois geradores e não entre as duas descidas.
    let plain = ph2d_color::rgba16_to_rgba8(&halves);
    let dithered = ph2d_color::rgba16_to_rgba8_dithered(&halves, W);

    let left = Vec2::new(-(world[0] + gap) * 0.5, 0.0);
    let right = Vec2::new((world[0] + gap) * 0.5, 0.0);
    spawn_8bit(
        sim,
        renderer,
        asset_db,
        plain,
        left,
        world,
        "Gradient · 8-bit, plain",
    );
    spawn_8bit(
        sim,
        renderer,
        asset_db,
        dithered,
        right,
        world,
        "Gradient · 8-bit, dithered",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠️ **A fixture tem de CONTER o fenómeno.** Um degradê que descesse para um byte só (ou para
    /// duzentos) não mostraria faixas nenhumas, e o smoke ficaria verde a não provar nada — a falha
    /// mais provável deste ficheiro é alguém «arredondar» os extremos e a cena deixar de ensinar.
    #[test]
    fn the_plain_descent_really_produces_visible_bands() {
        let plain = ph2d_color::rgba16_to_rgba8(&gradient_16());
        let row: Vec<u8> = plain
            .chunks_exact(4)
            .take(W as usize)
            .map(|px| px[0])
            .collect();

        let mut steps = 0usize;
        let mut widest = 0usize;
        let mut run = 1usize;
        for pair in row.windows(2) {
            if pair[0] == pair[1] {
                run += 1;
            } else {
                steps += 1;
                widest = widest.max(run);
                run = 1;
            }
        }
        widest = widest.max(run);

        assert!(
            (4..=8).contains(&steps),
            "a fixture devia atravessar ~6 codigos ao longo da largura, e atravessou {steps} \
             degraus. Menos e' um chapado; muito mais sao faixas de 2 px que ninguem ve'."
        );
        assert!(
            widest >= 32,
            "a faixa mais larga tem {widest} px — abaixo de ~32 o defeito nao se le' como faixa, e \
             o smoke deixa de ensinar o que existe para ensinar"
        );
    }

    /// **E o caminho com dither tem de as DESFAZER.** O par com o teste acima é o que torna a cena
    /// uma comparação em vez de duas imagens.
    ///
    /// ⚠️ **Nem toda coluna se mistura, e a primeira versão deste teste falhou por isso.** Ela
    /// escolheu `x = W/3`, que calhou a cair no **centro** de uma faixa — e um valor no centro está
    /// *em cima* da grelha de 8 bits, onde o dither promete não mexer. A cura não é «toda a coluna
    /// fica ruidosa»: é a fronteira entre duas faixas deixar de ser uma linha e passar a ser uma
    /// **transição** que atravessa a faixa inteira. Colunas do centro ficarem limpas é a promessa a
    /// funcionar, não uma falha.
    #[test]
    fn the_dithered_descent_dissolves_those_bands() {
        let halves = gradient_16();
        let plain = ph2d_color::rgba16_to_rgba8(&halves);
        let dithered = ph2d_color::rgba16_to_rgba8_dithered(&halves, W);
        assert_ne!(plain, dithered, "as duas descidas deram o MESMO resultado");

        let mixed_columns = |px: &[u8]| -> usize {
            (0..W)
                .filter(|&x| {
                    (0..H)
                        .map(|y| px[((y * W + x) * 4) as usize])
                        .collect::<std::collections::BTreeSet<u8>>()
                        .len()
                        > 1
                })
                .count()
        };

        assert_eq!(
            mixed_columns(&plain),
            0,
            "sem dither, TODA coluna tem de ser de um byte so' — e' isso que faz a faixa"
        );
        let mixed = mixed_columns(&dithered);
        // ⚠️ **MEDIDO: 216 das 384 (56%), e a previsão analítica dizia ~86%.** A diferença é uma
        // propriedade real da matriz de Bayer que vale a pena ter escrita: **o espalhamento dos
        // limiares por COLUNA é desigual**. Na coluna `x%8 == 0` os oito níveis são 0, 48, 12, 60,
        // 3, 51, 15, 63 — a faixa toda; na `x%8 == 3` são 40, 24, 36, 20, 43, 27, 39, 23, que só
        // cobrem o **meio** (±0,21 em vez de ±0,43). Metade das colunas dither*a* com meia amplitude,
        // e é daí que vem o xadrez característico do Bayer ordenado.
        //
        // ⛔ Isto **não** é defeito a curar: o olho integra ao longo de x, e a rampa continua a
        // atravessar as colunas largas. A barra fica em 45% — folgada abaixo dos 56% medidos, e
        // apanha «o dither desapareceu» sem reprovar por uma variação de fixture.
        assert!(
            mixed * 100 >= (W as usize) * 45,
            "so' {mixed} das {W} colunas se misturam (medido: 216) — abaixo disto a sprite da \
             direita volta a ler como faixas, e a cena mostra duas imagens iguais"
        );
    }
}
