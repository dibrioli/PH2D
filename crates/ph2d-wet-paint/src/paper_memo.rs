//! **A memória dos papéis** — o tile do papel é uma FUNÇÃO das entradas, então quem já o fez não o
//! refaz (fila 44, item 7).
//!
//! [`generate_paper_tile`] é determinístico por `(preset, folha, knobs)` e é caro: a caminhada das
//! fibras e as somas em `f64` correm em série DE PROPÓSITO, porque a ordem do gerador aleatório é a
//! impressão digital do motor (§7.4 do handoff da linha). Paralelizá-lo está recusado. O que sobra é
//! não o repetir — e ele repetia-se muito mais do que «uma vez por sessão»: a sessão morre a cada
//! Ctrl+Z (a guarda de identidade da tela) e a cada troca de modo, e cada nascimento corria o gerador
//! pelo menos uma vez (o construtor) e duas com um papel autorado (o `reconcile_facts` re-coze).
//!
//! ⭐ **É exacto por construção:** a chave são os BITS das entradas, logo um acerto devolve o tile que
//! o gerador daria. Não é uma aproximação nem uma cache com invalidação — não há estado de fora que
//! o tile leia. O gate compara o devolvido com um gerado de fresco, bit a bit.
//!
//! ⚠️ **A chave é por BITS e não por `==`:** `-0.0 == 0.0` é verdade em `f64` e os bits diferem. Com a
//! chave por bits, duas entradas iguais no `==` e diferentes nos bits geram DUAS vezes (o custo de
//! um acerto perdido); com `==` elas partilhariam um tile — o que só é seguro se o gerador não ler o
//! sinal de um zero, e ninguém mediu isso.
//!
//! ⚠️ **Global e com trava, e não por thread:** o motor nasce na thread da interface e é entregue ao
//! trabalhador da simulação, e um re-cozido por knob pode correr em qualquer das duas. Uma memória
//! por thread perderia exactamente esses acertos. O gerador corre **fora** da trava.
//!
//! O tecto [`MEMO_CAP`] é de MEMÓRIA: cada tile tem `512² × 4 B = 1 MiB`.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::paper::{PaperKnobs, PaperPreset, TILE_SIZE, generate_paper_tile};

/// **Quantos tiles a memória segura — `8`, isto é `8 MiB`.**
///
/// O recurso é a memória (1 MiB por tile). Os papéis vivos numa sessão de trabalho são poucos: o de
/// fábrica (o construtor coze-o sempre), o autorado corrente e os que o artista acabou de experimentar.
/// Arrastar um knob do papel gera uma chave por valor, e a memória esquece os mais velhos primeiro
/// (recente à frente). `8` deixa lugar ao de fábrica, ao corrente e a seis experiências.
pub const MEMO_CAP: usize = 8;

type Chave = (PaperPreset, u32, [u64; 3]);

/// Os tiles guardados, o usado mais recentemente à frente.
type Fila = VecDeque<(Chave, Arc<[f32]>)>;

static MEMORIA: Mutex<Fila> = Mutex::new(VecDeque::new());

/// **O acerto:** devolve o tile e põe a entrada à FRENTE — quem é usado fica. O papel de fábrica é
/// pedido a cada nascimento do motor; numa fila por chegada ele sairia ao fim de [`MEMO_CAP`]
/// experiências com os knobs, por mais que fosse usado.
fn procura(m: &mut Fila, k: &Chave) -> Option<Arc<[f32]>> {
    let pos = m.iter().position(|(c, _)| c == k)?;
    let entrada = m.remove(pos)?;
    let tile = Arc::clone(&entrada.1);
    m.push_front(entrada);
    Some(tile)
}

/// **Guarda um tile acabado de gerar**, à frente, e o tecto esquece o menos usado. Outra thread pode
/// ter feito o mesmo tile enquanto este corria fora da trava: os dois são o mesmo, e guarda-se um só.
fn guarda(m: &mut Fila, k: Chave, tile: Arc<[f32]>) {
    if !m.iter().any(|(c, _)| *c == k) {
        m.push_front((k, tile));
        m.truncate(MEMO_CAP);
    }
}

fn trava() -> std::sync::MutexGuard<'static, Fila> {
    MEMORIA.lock().unwrap_or_else(|e| e.into_inner())
}

fn chave(preset: PaperPreset, sheet_index: u32, knobs: PaperKnobs) -> Chave {
    (
        preset,
        sheet_index,
        [
            knobs.contrast.to_bits(),
            knobs.fibres.to_bits(),
            knobs.grooves.to_bits(),
        ],
    )
}

/// **O tile do papel, feito uma vez por entrada.** A porta que o [`crate::painter::Engine`] usa;
/// o resultado é o de [`generate_paper_tile`] ao bit.
pub fn paper_tile(preset: PaperPreset, sheet_index: u32, knobs: PaperKnobs) -> Vec<f32> {
    let k = chave(preset, sheet_index, knobs);
    let achado = procura(&mut trava(), &k);
    if let Some(tile) = achado {
        return tile.to_vec();
    }
    // O gerador corre FORA da trava.
    let tile = generate_paper_tile(preset, sheet_index, knobs);
    debug_assert_eq!(tile.len(), TILE_SIZE * TILE_SIZE);
    guarda(&mut trava(), k, Arc::from(tile.as_slice()));
    tile
}

#[cfg(test)]
#[path = "paper_memo_tests.rs"]
mod tests;
