//! **A RASTERIZAÇÃO DOS FIOS** — um fio do Sketchy vira cobertura por **área exata**, e os fios
//! compõem **`over`, um sobre o outro** (plano 38 W3).
//!
//! O motor responde *que alfa este feixe de fios deixa nesta janela*; quem decide **com que cor** e
//! **onde escrever** é o tool, exactamente como no irmão [`crate::solid`].
//!
//! # Por que um fio é um SEGMENTO, e não uma corrente de dabs
//!
//! A W0.3 mediu: com densidade cheia os fios somam **~50× o arco do traço** no alcance de um
//! diâmetro. Carimbar um fio fino à spacing de sub-pixel poria a conta em 10⁴–10⁶ dabs por traço —
//! é por isso que o Harmony e o Krita o desenham como segmento, e é por isso que ele sai por um
//! canal PRÓPRIO ([`crate::stroke::sketchy::Thread`]) em vez de entrar na lista de dabs.
//!
//! # Por que `over` por fio, e nunca um preenchimento só de todos os quads
//!
//! **O hachurado é ACÚMULO.** Onde dois fios se cruzam a tinta tem de ficar mais escura — é disso
//! que o traço tira o tom. Um [`solid::fill_coverage`] único sobre o conjunto **satura** a cobertura
//! em `1` no cruzamento (a regra não-zero é `|soma|.min(1)`), então dez fios sobrepostos leriam como
//! um e a teia sairia CHAPADA: o desenho perderia exactamente a informação que o feixe carrega.
//!
//! ⚠️ **E a razão que eu ia escrever aqui era FALSA, conferida em vez de suposta:** *"dois fios de
//! sentidos opostos cancelariam o winding e abririam um buraco"*. Não abrem — inverter um segmento
//! inverte **a ordem dos vértices E a normal**, então a orientação do quad é invariante ao sentido
//! do fio e a soma nunca muda de sinal (medido: os dois sentidos dão área com sinal `−60`). O
//! preenchimento único não é *unsound*, é **chapado** — e chapado já basta para não o usar.
//!
//! Cada fio é preenchido na **sua própria caixa** (uma faixa de poucos pixels) e composto no alfa da
//! janela; a janela inteira é escrita **uma vez** pelo tool.

use crate::solid;
use crate::stroke::sketchy::Thread;

/// Com que tinta um feixe de fios é desenhado — a espessura e a opacidade, e mais nada.
///
/// ⚠️ **O `Magnetify` NÃO mora aqui, e a primeira versão desta wave o pôs aqui por engano.** Ele foi
/// construído como uma rampa de opacidade por distância (*fio perto puxa mais*) e o manual do Krita
/// diz outra coisa, verbatim: *"It's what causes curve lines to form **between two close line
/// sections** … With Magnetify off, the curve line just forms on either side of the current active
/// portion of your connection line."* Ele decide **QUE PARES viram fio**, não com que força um fio
/// desenha — e é por isso que vive no motor ([`crate::stroke::sketchy`]), onde os pares nascem.
/// O que esta rampa fazia é o que o Krita chama de *Distance Opacity* / *Use Distance Density*, dois
/// controles SEPARADOS que este card não oferece.
#[derive(Clone, Copy, Debug)]
pub struct ThreadInk {
    /// A espessura de um fio, em pixels de canvas.
    pub width_px: f32,
    /// A opacidade de UM fio (`0..=1`) — a do cruzamento sai do acúmulo, não daqui.
    pub opacity: f32,
}

/// O retângulo do canvas que um feixe toca, já recortado a `w × h`. `None` quando não sobra nada
/// dentro da tela — a mesma resposta (e a mesma lei de rejeição-antes-do-clamp) do
/// [`solid::loops_bbox`], que é quem de facto responde.
#[must_use]
pub fn threads_bbox(threads: &[Thread], width_px: f32, w: usize, h: usize) -> Option<[usize; 4]> {
    let quads = quads(threads, width_px, [0.0, 0.0]);
    solid::loops_bbox(&quads, w, h)
}

/// O alfa `0..=255` que o feixe deixa numa janela `w × h` cuja origem (canto superior-esquerdo, em
/// pixels de canvas) é `origin`.
#[must_use]
pub fn threads_alpha(
    threads: &[Thread],
    ink: ThreadInk,
    w: usize,
    h: usize,
    origin: [f32; 2],
) -> Vec<u8> {
    let mut alpha = vec![0u8; w * h];
    if w == 0 || h == 0 || ink.width_px <= 0.0 || ink.opacity <= 0.0 {
        return alpha;
    }
    // Um laço reusado: o `fill_coverage` recebe uma fatia, então a alocação do quad não precisa
    // nascer por fio.
    let mut one: [Vec<[f32; 2]>; 1] = [Vec::with_capacity(4)];
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let op255 = (ink.opacity.clamp(0.0, 1.0) * 255.0 + 0.5) as u32;
    if op255 == 0 {
        return alpha;
    }
    for t in threads {
        one[0].clear();
        if !push_quad(&mut one[0], *t, ink.width_px * 0.5, origin) {
            continue;
        }
        let Some([bx, by, bw, bh]) = solid::loops_bbox(&one, w, h) else {
            continue;
        };
        #[allow(clippy::cast_precision_loss)]
        let cov = solid::fill_coverage(&one, bw, bh, [bx as f32, by as f32]);
        for row in 0..bh {
            let dst = (by + row) * w + bx;
            let src = row * bw;
            for cx in 0..bw {
                let c = u32::from(cov[src + cx]);
                if c == 0 {
                    continue;
                }
                let m = c * op255 / 255;
                let a = u32::from(alpha[dst + cx]);
                #[allow(clippy::cast_possible_truncation)]
                {
                    alpha[dst + cx] = (m + a * (255 - m) / 255) as u8;
                }
            }
        }
    }
    alpha
}

/// Os quads de todos os fios, em coordenadas de janela (`origin` já subtraída).
fn quads(threads: &[Thread], width_px: f32, origin: [f32; 2]) -> Vec<Vec<[f32; 2]>> {
    let half = width_px * 0.5;
    let mut out = Vec::with_capacity(threads.len());
    for t in threads {
        let mut q = Vec::with_capacity(4);
        if push_quad(&mut q, *t, half, origin) {
            out.push(q);
        }
    }
    out
}

/// O retângulo de um fio: o segmento engrossado por `half` para cada lado. `false` quando o fio é
/// degenerado (as duas pontas no mesmo lugar) — ele não cerca área nenhuma.
///
/// ⚠️ **As pontas são CHATAS**, e é uma decisão e não um esquecimento: um fio mede ~1 px, e uma
/// tampa redonda custaria um arco por ponta (duas por fio, milhares por traço) para mudar meio
/// pixel que ninguém distingue. O irmão que tem tampa é o TRAÇO, onde ela mede a espessura inteira.
fn push_quad(out: &mut Vec<[f32; 2]>, t: Thread, half: f32, origin: [f32; 2]) -> bool {
    let d = [t[2] - t[0], t[3] - t[1]];
    let len = d[0].hypot(d[1]);
    if len <= f32::EPSILON || half <= 0.0 {
        return false;
    }
    let n = [-d[1] / len * half, d[0] / len * half];
    let (ax, ay) = (t[0] - origin[0], t[1] - origin[1]);
    let (bx, by) = (t[2] - origin[0], t[3] - origin[1]);
    out.push([ax + n[0], ay + n[1]]);
    out.push([bx + n[0], by + n[1]]);
    out.push([bx - n[0], by - n[1]]);
    out.push([ax - n[0], ay - n[1]]);
    true
}
