//! ⭐ **Os dois TECTOS das lâmpadas** — o da forma e o do quadro.
//!
//! ⚠️ Saíram do [`crate::trace`] em 2026-09-16 por TETO DE LINHAS, e a fronteira é real: aqui não há
//! despacho nenhum, só dois números e a medição de cada um.

/// ⭐⭐⭐ **QUANTAS LÂMPADAS CABEM NO UNIFORME** — o tecto da FORMA, e não o do quadro.
///
/// # ⚠️ De que recurso ele é
///
/// Do **bloco de uniforme**: as posições e as radiâncias viajam em dois `array<vec4, N>`, que a
/// `32` são `1 KiB` de um bloco com `64 KiB` de tecto. Ele é folgado de propósito — o tecto que
/// **morde** é outro, e é o [`lamps_that_fit`].
///
/// ⛔⛔ **A primeira redacção escreveu `8` aqui e disse que o recurso era o RELÓGIO**, com a conta
/// *«o passe cresce ~1 traçado por lâmpada»*. Medido a `1920×1080` (`load 4,8`, mínimo de 7):
///
/// | lâmpadas | 1 | 2 | 4 | 8 | 12 |
/// |---|---:|---:|---:|---:|---:|
/// | quadro | `4,20` | `4,28` | `4,47` | `6,94` | `7,48 ms` |
///
/// ⇒ `12` lâmpadas custam **`1,8×`** uma, não `12×`: a marcha de sombra só corre nos pixels que
/// acertam **e** que vêem aquela luz, e as direcções partilham a mesma cache. *Um tecto derivado
/// de uma estimativa em vez de uma medição erra para o lado de dentro — e foi o que este fez.*
pub const MAX_LAMPS: usize = 32;

/// ⭐⭐⭐ **QUANTAS LÂMPADAS CABEM NESTE QUADRO** — o tecto que de facto morde.
///
/// # ⛔⛔ O recurso é o TAMANHO DE LIGAÇÃO DE UM BUFFER, e foi a placa que o disse
///
/// O canal de luz guarda `1 + n_lamps` floats **por pixel** (o céu mais uma visibilidade por
/// lâmpada). A `1920×1080` com `16` lâmpadas isso são `141 004 800 B`, e a `wgpu` recusou:
///
/// ```text
/// Buffer binding 3 range 141004800 exceeds `max_*_buffer_binding_size` limit 134217728
/// ```
///
/// ⇒ o tecto é `limite / (pixels · 4) − 1`, e ele **depende da resolução**: `15` lâmpadas a
/// `1920×1080` e `144` a `640×360`. *Um tecto que muda com o tamanho da tela não é uma constante,
/// e escrevê-lo como uma teria posto o número do quadro assente a governar o de movimento.*
///
/// ⚠️ **A placa é quem diz o limite** ([`Tracer::binding_limit`]) — a `wgpu` garante `128 MiB` como
/// mínimo, e uma placa que ofereça mais fica com mais lâmpadas sem ninguém mexer num número.
#[must_use]
pub fn lamps_that_fit(binding_limit: u64, width: u32, height: u32) -> usize {
    let pixels = u64::from(width) * u64::from(height);
    if pixels == 0 {
        return MAX_LAMPS;
    }
    // ⚠️ O `saturating_sub(1)` é o slot do CÉU, que ocupa o primeiro lugar de cada pixel.
    #[allow(clippy::cast_possible_truncation)]
    let cabem = (binding_limit / (pixels * 4)).saturating_sub(1) as usize;
    cabem.min(MAX_LAMPS)
}
