//! ⭐⭐⭐ **O SALTO DO LADRILHO NA VOLTA** — *este desenho encaixa consigo próprio?* (plano 33, W10).
//!
//! # A lei, medida
//!
//! Uma investigação de 2026-08-30 (sonda `pattern_seam_probe`, GPU real, oráculo por periodicidade)
//! partiu de uma acusação ao amostrador do Vello — em `Extend::Repeat` ele embrulha a *coordenada* e
//! depois **grampeia os taps** do filtro contra o rectângulo da imagem no atlas, em vez de os deixar
//! dar a volta. A acusação é verdadeira, e o que a medição mostrou é que ela **não é a história**:
//!
//! | ladrilho | salto dele na volta | costura medida |
//! |---|---|---|
//! | ruído cru | `236` | `100` níveis, 22 colunas |
//! | ruído **espelhado** | `0` | `7` níveis, 6 colunas |
//! | onda quadrada crua | `215` | `107` níveis, 22 colunas |
//! | onda quadrada **espelhada** | `0` | **`0`** |
//!
//! ⇒ **o grampo custa exactamente o salto do PRÓPRIO ladrilho, e quase nada além disso.** Um
//! ladrilho que fecha não tem costura de amostrador — em qualidade nenhuma.
//!
//! ⛔⛔ **E é por isso que a cura NÃO é o filtro.** Baixar a qualidade para `Medium` foi medido e
//! **REFUTADO**: sob um deslocamento de meio pixel — que é o que um `pan` faz o tempo todo — o
//! `Medium` e o `High` chegam ao **mesmo** pico (`107`); o `Medium` só estreita a banda de ~3 texels
//! para ~1, e paga fidelidade de ampliação medida. *O `0` que o `Medium` marca a 1:1 existe só no
//! alinhamento inteiro, que é medida zero na prática.*
//!
//! # O que isto entrega ao artista
//!
//! Um ladrilho com salto grande mostra **uma aresta dura em cada fronteira** — um defeito de
//! CONTEÚDO, muito maior que a banda do filtro, e que o artista vê imediatamente sem saber porquê.
//! O app tem os bytes e pode dizê-lo. *Uma ferramenta que ignora em silêncio é pior que uma que
//! recusa.*

use crate::bake::Tile;

/// **O limiar em que a costura deixa de ser uma linha fina e passa a ser uma banda** — `16` níveis
/// de 8 bits.
///
/// ⛔ **Não é um número escolhido.** Sai do joelho de uma varredura na GPU (`salto` pedido contra
/// `costura` medida, `High`, 4x, deslocamento de meio pixel — o pior caso):
///
/// | salto | 0 | 2 | 4 | 8 | **16** | **32** | 64 | 128 | 200 |
/// |---|---|---|---|---|---|---|---|---|---|
/// | costura (níveis) | 8 | 9 | 10 | 12 | **16** | **21** | 37 | 71 | 104 |
/// | colunas erradas | 6 | 6 | 6 | 6 | **6** | **14** | 18 | 22 | 22 |
///
/// A **largura** da banda salta de `6` para `14` colunas entre `16` e `32`: abaixo do joelho o
/// defeito é uma linha, acima dele é uma faixa. O recurso é a **percepção**, não a memória nem o
/// relógio.
pub const SEAM_VISIBLE: u8 = 16;

/// O maior degrau que aparece quando se põe uma cópia do ladrilho **ao lado de si mesmo** — nos dois
/// eixos. `0` significa que ele fecha exactamente.
///
/// ⚠️⚠️ **A comparação é em alfa PRÉ-MULTIPLICADO, e a escolha é load-bearing.** Todo PNG com
/// transparência carrega RGB arbitrário debaixo de alfa zero; em RGB reto, duas bordas
/// **invisíveis** com lixo diferente acusariam um salto que ninguém vê. É a mesma lei que o
/// `downscale` deste crate já paga (`the_downscale_does_not_bleed_colour_from_under_zero_alpha`).
///
/// ⚠️ Um motivo assado de uma FORMA tem a caixa justa, então a cobertura vai a zero nos quatro
/// lados — e é por isso que ele mede `0` aqui e **não tem costura nenhuma** na GPU. O defeito é da
/// arte que vai de bordo a bordo (uma fotografia de tecido, de papel, de granito) e não foi feita
/// para repetir.
#[must_use]
pub fn wrap_seam(tile: &Tile) -> u8 {
    let (w, h) = (tile.width, tile.height);
    if w == 0 || h == 0 {
        return 0;
    }
    let px = &tile.rgba;
    let at = |x: u32, y: u32| -> [u8; 4] {
        let o = ((y as usize) * (w as usize) + x as usize) * 4;
        [px[o], px[o + 1], px[o + 2], px[o + 3]]
    };
    let mut pior = 0u8;
    // A junta VERTICAL: a última coluna encosta na primeira.
    for y in 0..h {
        pior = pior.max(premul_diff(at(w - 1, y), at(0, y)));
    }
    // A junta HORIZONTAL: a última linha encosta na primeira.
    for x in 0..w {
        pior = pior.max(premul_diff(at(x, h - 1), at(x, 0)));
    }
    pior
}

/// **A comparação contra o joelho, escrita UMA vez.**
///
/// ⚠️ Ela existe porque tem **dois** chamadores em crates diferentes — o veredito local
/// ([`tiles_cleanly`]) e a shell, que só tem o número (ele viaja no `PatternTile`, medido no assado)
/// e não o ladrilho. *Uma lei escrita em dois sítios ainda não é uma lei — só uma PORTA é*, e um
/// `>` que virasse `>=` num dos lados só apareceria como um aviso que pisca.
#[must_use]
pub fn seam_is_visible(wrap_seam: u8) -> bool {
    wrap_seam > SEAM_VISIBLE
}

/// *Este ladrilho encaixa consigo próprio?* — o veredito que o painel mostra.
#[must_use]
pub fn tiles_cleanly(tile: &Tile) -> bool {
    !seam_is_visible(wrap_seam(tile))
}

/// O maior desvio entre dois texels, com o RGB **pré-multiplicado** pela alfa de cada um.
fn premul_diff(a: [u8; 4], b: [u8; 4]) -> u8 {
    let pm = |t: [u8; 4]| -> [u8; 4] {
        let al = u16::from(t[3]);
        let c = |v: u8| -> u8 {
            #[allow(clippy::cast_possible_truncation)]
            {
                ((u16::from(v) * al + 127) / 255) as u8
            }
        };
        [c(t[0]), c(t[1]), c(t[2]), t[3]]
    };
    let (a, b) = (pm(a), pm(b));
    (0..4).map(|i| a[i].abs_diff(b[i])).max().unwrap_or(0)
}
