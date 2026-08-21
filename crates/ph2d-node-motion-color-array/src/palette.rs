//! A PALETA do `motion.color_array` — a lista de cores autorada como texto, e a
//! metade dela que o device consome.
//!
//! O que mora aqui é a LEI: *dada uma string, que lista de cores ela é?* — e ela
//! é a mesma nos dois caminhos (a CPU chama [`palette_of`]; o WGSL do `lib.rs` lê
//! o MESMO vetor pela LUT que [`fill_lut`] escreve, a partir da MESMA função).
//! Uma segunda cópia da gramática é como a face do artista e o device passam a
//! discordar sobre qual cor a peça `i` recebe.
//!
//! O canal e o layout são os do `value.pattern` (`[len, …]`, com a contagem no
//! slot 0), pela mesma razão: um `arrayLength` no device devolve a CAPACIDADE,
//! nunca quantas cores o artista escreveu.

use ph2d_color::{DEFAULT_PALETTE_FALLBACK, parse_palette};

/// Quantas cores a paleta carrega, no máximo.
///
/// ⚠️ **É teto de RECURSO, e o recurso é o BUFFER DO DEVICE.** A LUT é um
/// `storage` de [`LUT_LEN`] floats por nó `motion.color_array` no documento,
/// alocado a cada cook, e a `resolution` de um `LutSpec` é uma **const** — o
/// buffer não encolhe com a paleta curta. Medido:
///
/// | grandeza | valor |
/// |---|---|
/// | floats por buffer | `1 + 4 × 1024` = **4.097** |
/// | bytes por nó | **16.388** (16,0 KiB), CONSTANTE |
/// | cem nós numa cena | **1,6 MiB** de VRAM |
/// | `fill_lut` a 1024 cores | linear no que foi escrito, **uma vez por encode** |
///
/// ⚠️ **Este teto NÃO é o que o Enio mandou tirar.** O de antes era **quatro**, e
/// vinha de *"params são f32"* — quatro `ParamSpec` escritos à mão, um limite da
/// REPRESENTAÇÃO, sem recurso nenhum atrás (folha 09/15). Este diz de que recurso
/// é e traz a conta (§0). Uma tira de mil e vinte e quatro amostras não é uma
/// paleta que alguém edite à mão.
///
/// ⚠️ **A truncagem acontece nos DOIS caminhos**, porque os dois passam por
/// [`palette_of`]. Uma paleta longa não muda de cor entre a CPU e o device: ela
/// tem o mesmo comprimento nos dois.
pub const MAX_COLORS: usize = 1024;

/// O comprimento do buffer da LUT: o cabeçalho (a CONTAGEM) mais quatro floats
/// por cor.
pub const LUT_LEN: u32 = 1 + 4 * MAX_COLORS as u32;

/// A paleta que o nó de facto usa: a autorada, truncada em [`MAX_COLORS`], ou a
/// de fábrica quando não há nada autorado.
///
/// **Nunca devolve a lista vazia** — e isso é contrato, não sorte: `cycle` indexa
/// `palette[idx]` sem `get`, e o WGSL faz o mesmo com `lut[1 + 4·k]`. Os três
/// casos que colapsam para o default são *ausente*, *malformada* (o
/// `parse_palette` recusa a string inteira em vez de encurtá-la em silêncio) e
/// *`"p1"`* (a paleta explicitamente vazia). Uma paleta vazia deixaria o nó sem
/// cor nenhuma para escrever, que é um estado sem leitura visual.
#[must_use]
pub fn palette_of(text: Option<&str>) -> Vec<[f32; 4]> {
    let mut p = text
        .and_then(parse_palette)
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| DEFAULT_PALETTE_FALLBACK.to_vec());
    p.truncate(MAX_COLORS);
    p
}

/// Escreve a paleta no buffer da LUT — a metade do canal que mora no NÓ
/// (`ph2d-nodegraph` fica agnóstico de cor, exactamente como fica agnóstico de
/// curva e de gramática de tabela).
///
/// **O layout é `[len, r0,g0,b0,a0, r1,…]`**: o slot 0 carrega quantas cores
/// seguem. Ele nunca é zero (ver [`palette_of`]), então o corpo do kernel não
/// tem ramo de "paleta vazia" para escrever — a mesma coisa que a CPU não tem.
///
/// ⚠️ O `fill` recebe a string CRUA (`""` quando o param não foi autorado), e é
/// por isso que ele chama `palette_of(Some(text))` em vez de reimplementar o
/// fallback: `parse_palette("")` é `None`, que cai na paleta de fábrica — a mesma
/// que o `eval` usa quando `text_param` devolve `None`. *Os dois caminhos
/// concordam sobre "nada autorado" sem cada um ter a própria regra.*
pub fn fill_lut(text: &str, out: &mut [f32]) {
    if out.is_empty() {
        return;
    }
    let palette = palette_of(Some(text));
    // Cabe por construção (`LUT_LEN = 1 + 4·MAX_COLORS`), mas o `min` mantém o
    // preenchimento total mesmo que alguém encolha a `resolution`.
    let n = palette.len().min((out.len() - 1) / 4);
    #[expect(
        clippy::cast_precision_loss,
        reason = "n <= MAX_COLORS = 1024, exactamente representável em f32"
    )]
    {
        out[0] = n as f32;
    }
    for (k, c) in palette[..n].iter().enumerate() {
        out[1 + 4 * k..5 + 4 * k].copy_from_slice(c);
    }
}

#[cfg(test)]
#[path = "palette_tests.rs"]
mod tests;
