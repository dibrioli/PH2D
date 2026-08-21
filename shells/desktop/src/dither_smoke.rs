//! `PH2D_DITHER_SMOKE` — **as faixas e a cura, na MESMA imagem**.
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Sprite \
//!   && env PH2D_DITHER_SMOKE=1 cargo run -p ph2d-host-desktop --release
//! ```
//!
//! **Uma** sprite, partida ao meio na horizontal. As duas metades saem do **mesmo** degradê de 16
//! bits, descido para 8 bits pelos dois caminhos:
//!
//! | metade | porta | o que se vê |
//! |---|---|---|
//! | **cima** | `rgba16_to_rgba8` (fiel) | faixas verticais, com arestas duras |
//! | **baixo** | `rgba16_to_rgba8_dithered` (W6.1) | liso |
//!
//! # Por que UMA sprite partida, e não duas lado a lado
//!
//! ⚠️ **A primeira versão eram duas sprites separadas, e o Enio não viu diferença nenhuma.** Duas
//! metades adjacentes valem muito mais que duas imagens vizinhas: em cada coluna, cima e baixo
//! partem do **mesmo** valor, e as arestas da metade de cima **param a meio da imagem**. O olho é
//! excelente a detectar uma descontinuidade que atravessa uma fronteira partilhada, e péssimo a
//! comparar dois quadrados separados por um vão — sobretudo quando a diferença é de **um** código,
//! que é o passo mais pequeno que existe.
//!
//! # A causa MEDIDA de as duas primeiras versões saírem lisas
//!
//! ⚠️ **O filtro de textura do projeto é `Smooth` por omissão** (bilinear + anisotropia 16×,
//! [`ph2d_host::ImageFilterMode`]). Em **ampliação**, o bilinear interpola entre texels vizinhos — e
//! um degrau de um código transforma-se numa rampa suave. *Aproximar o zoom não mostrava o defeito:
//! apagava-o.* A instrução «dê zoom até ver os pixels» estava exactamente ao contrário.
//!
//! A cura é a sprite trazer o seu próprio [`ph2d_ecs::TextureFilter`] em `Nearest`: assim o que
//! aparece no ecrã são os bytes, a qualquer zoom, e a cena deixa de depender de uma preferência de
//! projeto que o artista pode ter mudado.
//!
//! # O que se vê a cada zoom, e as duas leituras são diferentes
//!
//! - **Zoom normal (a cena abre assim):** em cima, faixas verticais largas; em baixo, liso. É a
//!   leitura de **produto** — é isto que o defeito é.
//! - **Zoom fundo:** em baixo aparece um xadrez fino. ⚠️ **Não é ruído — é o mecanismo.** A aresta
//!   dura de cima foi trocada por uma mistura dos dois tons vizinhos; a 100% ela é fina demais para
//!   se ver, e o que fica é o degradê liso.
//!
//! # O que este smoke NÃO mostra, e é honesto dizê-lo
//!
//! ⛔ **Não há uma metade «RGBA16» para comparar.** Ela mostraria faixas *na mesma*: o ecrã é de
//! 8 bits e a descida final **não** leva dither — foi medida e recusada (W6.2, folga de 0,0283 LSB
//! numa placa em que meio passo precisa de 0,431). Uma terceira metade ensinaria a coisa errada:
//! que 16 bits não serve para nada. O que ele serve é para **não requantizar entre edições**, e isso
//! não se vê num ecrã — vê-se ao fim de cinco ferramentas.

use ph2d_asset::AssetDb;
use ph2d_core::Vec2;
use ph2d_ecs::SimWorld;
use ph2d_render::SpriteRenderer;

/// Largura do degradê, em pixels.
const W: u32 = 512;
/// Altura total. A metade de cima é a descida fiel, a de baixo a com dither.
const H: u32 = 512;
/// Os dois extremos do degradê, em código sRGB.
///
/// ⚠️ **Seis códigos ao longo de 512 px — faixas de ~85 px.** Faixas largas de propósito: duas cores
/// vizinhas de 8 bits estão a **um** código de distância, que é perto do limiar do olho, e o que faz
/// esse degrau aparecer é a **área** de cada lado (as bandas de Mach). Um degradê que atravessasse
/// 200 códigos teria faixas de 2 px e ninguém as veria — a banda é um defeito de degradês **lentos**.
const LO: f32 = 142.0;
const HI: f32 = 148.0;

pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_DITHER_SMOKE").is_some()
}

/// O degradê em meios-floats lineares — a fonte de 16 bits de que as duas metades descem.
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

/// A imagem da cena: metade de cima da descida fiel, metade de baixo da descida com dither.
///
/// ⚠️ **O dither é calculado sobre a imagem INTEIRA e só depois se corta a metade**, e não sobre um
/// meio-quadro. A matriz de Bayer é ladrilhada por `(x, y)` absoluto; gerá-la para uma imagem de
/// meia altura daria outra fase, e a metade de baixo deixaria de ser «a mesma imagem, outra porta».
fn split_image() -> Vec<u8> {
    let halves = gradient_16();
    let plain = ph2d_color::rgba16_to_rgba8(&halves);
    let dithered = ph2d_color::rgba16_to_rgba8_dithered(&halves, W);
    let seam = (H / 2 * W * 4) as usize;
    let mut out = plain;
    out.truncate(seam);
    out.extend_from_slice(&dithered[seam..]);
    out
}

/// Monta a cena e devolve os bits da sprite.
pub(crate) fn spawn_if_enabled(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    pixels_per_meter: f32,
) -> Option<u64> {
    let ppm = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    let pixels = split_image();
    let texture_id = match renderer.acquire_individual(W, H, &pixels) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("[dither-smoke] a imagem nao subiu para a GPU: {e}");
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
        Vec2::ZERO,
        [W as f32 / ppm, H as f32 / ppm],
        "Gradient · top plain, bottom dithered",
    );
    // ⚠️ **O `Nearest` é o que torna esta cena legível**, e não uma preferência de gosto: com o
    // filtro `Smooth` do projeto, ampliar interpola entre texels vizinhos e o degrau de um código
    // vira uma rampa — as duas metades saem lisas e a cena não ensina nada (foi o que aconteceu na
    // primeira tentativa). Aqui a sprite traz o seu próprio filtro, e o que se vê são os bytes.
    sim.world_mut()
        .entity_mut(ph2d_ecs::Entity::from_bits(bits))
        .insert(ph2d_ecs::TextureFilter(ph2d_ecs::FilterMode::Nearest));
    Some(bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O byte no pixel `(x, y)` da imagem da cena.
    fn at(px: &[u8], x: u32, y: u32) -> u8 {
        px[((y * W + x) * 4) as usize]
    }

    /// A metade de cima e a de baixo.
    const TOP_Y: u32 = H / 4;
    const BOTTOM_Y: u32 = H / 2 + H / 4;

    /// ⚠️ **A fixture tem de CONTER o fenómeno.** Um degradê que descesse para um byte só (ou para
    /// duzentos) não mostraria faixas nenhumas, e o smoke ficaria verde a não provar nada — a falha
    /// mais provável deste ficheiro é alguém «arredondar» os extremos e a cena deixar de ensinar.
    #[test]
    fn the_top_half_really_shows_wide_hard_bands() {
        let img = split_image();
        let row: Vec<u8> = (0..W).map(|x| at(&img, x, TOP_Y)).collect();

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
             degraus. Menos e' um chapado; muito mais sao faixas finas que ninguem ve'."
        );
        assert!(
            widest >= 64,
            "a faixa mais larga tem {widest} px. Duas cores vizinhas de 8 bits estao a UM codigo \
             de distancia, que e' perto do limiar do olho — o que faz o degrau aparecer e' a AREA \
             de cada lado. Abaixo de ~64 px a cena deixa de ensinar."
        );
    }

    /// **E a metade de baixo tem de as DESFAZER.**
    ///
    /// ⚠️ **Nem toda coluna se mistura, e a primeira versão deste teste falhou por isso.** Ela
    /// escolheu uma coluna que calhou no **centro** de uma faixa — e um valor no centro está *em
    /// cima* da grelha de 8 bits, onde o dither promete não mexer. A cura não é «fica tudo
    /// ruidoso»: é a fronteira entre duas faixas deixar de ser uma linha e passar a ser uma
    /// **transição**. Colunas do centro ficarem limpas é a promessa a funcionar.
    #[test]
    fn the_bottom_half_dissolves_those_bands() {
        let img = split_image();

        let mixed = |y0: u32, y1: u32| -> usize {
            (0..W)
                .filter(|&x| {
                    (y0..y1)
                        .map(|y| at(&img, x, y))
                        .collect::<std::collections::BTreeSet<u8>>()
                        .len()
                        > 1
                })
                .count()
        };

        assert_eq!(
            mixed(0, H / 2),
            0,
            "a metade de CIMA tem colunas misturadas — ela e' a descida fiel, e toda coluna tem de \
             ser de um byte so'. E' isso que faz a faixa."
        );
        let below = mixed(H / 2, H);
        // ⚠️ **MEDIDO: ~56%, e a previsão analítica dizia ~86%.** A diferença é uma propriedade real
        // da matriz de Bayer que vale a pena ter escrita: **o espalhamento dos limiares por COLUNA é
        // desigual**. Na coluna `x%8 == 0` os oito níveis são 0, 48, 12, 60, 3, 51, 15, 63 — a faixa
        // toda; na `x%8 == 3` são 40, 24, 36, 20, 43, 27, 39, 23, que só cobrem o **meio** (±0,21 em
        // vez de ±0,43). Metade das colunas dither*a* com meia amplitude, e é daí que vem o xadrez
        // característico do dither ordenado. ⛔ Não é defeito a curar: o olho integra ao longo de x.
        assert!(
            below * 100 >= (W as usize) * 45,
            "so' {below} das {W} colunas da metade de baixo se misturam — abaixo disto ela volta a \
             ler como faixas e a cena mostra duas metades iguais"
        );
    }

    /// ⚠️ **As duas metades têm de partir do MESMO valor em cada coluna.** Sem isto a cena compara
    /// dois degradês em vez de duas descidas, e a costura no meio mostraria um salto que não tem
    /// nada a ver com o dither.
    #[test]
    fn both_halves_come_from_the_same_gradient() {
        let img = split_image();
        for x in (0..W).step_by(7) {
            let top = at(&img, x, TOP_Y);
            let bottom = at(&img, x, BOTTOM_Y);
            assert!(
                bottom.abs_diff(top) <= 1,
                "na coluna {x} a metade de cima deu {top} e a de baixo {bottom} — o dither so' pode \
                 escolher entre os DOIS bytes vizinhos, logo as duas metades divergiram na fonte"
            );
        }
    }
}
