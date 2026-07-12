//! Ciclos — o **pre/post behavior** de uma camada (W3.T3.2).
//!
//! Um ciclo NÃO duplica quadros: é o *wrap-mode* do amostrador. A camada tem um
//! **vão** (`span`) — do primeiro quadro exposto ao último — e o que acontece
//! FORA dele é decisão da camada: nada, segurar, repetir, vai-e-volta. É o mesmo
//! modelo do Harmony (Layer Properties → pre/post behaviour) e do TVPaint, e o
//! oposto do "copie 12 vezes o ciclo de caminhada" (que enche o documento de
//! desenhos instanciados e some com a intenção).
//!
//! **Os defaults reproduzem o comportamento pré-W3 byte a byte** (`pre = None`,
//! `post = Hold`): antes da primeira chave não aparece nada; depois da última, o
//! último desenho segura para sempre. Ligar um ciclo é opt-in.
//!
//! O vão é derivado (não guardado): `[primeira chave, fim)`, onde `fim` é a
//! sentinela de fim, se houver, ou `última chave + 1` (a última chave real expõe
//! um quadro — a tira deixa o usuário esticar esse hold, o que CRIA a sentinela e
//! define a duração de propósito).

use crate::ids::Frame;
use serde::{Deserialize, Serialize};

/// O que acontece fora do vão da camada, de cada lado.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CycleMode {
    /// Nada aparece (o default ANTES da primeira chave).
    #[default]
    None,
    /// O quadro da borda segura para sempre (o default DEPOIS da última chave).
    Hold,
    /// Repete o vão indefinidamente.
    Loop,
    /// Vai-e-volta: o vão é espelhado, sem repetir as bordas (o período é
    /// `2·len − 2`, a convenção clássica).
    PingPong,
}

/// Pre/post behavior de uma camada. `Default` = o comportamento pré-ciclos.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerCycle {
    /// Antes da primeira chave.
    pub pre: CycleMode,
    /// Depois do fim do vão.
    pub post: CycleMode,
}

impl Default for LayerCycle {
    fn default() -> Self {
        Self {
            pre: CycleMode::None,
            post: CycleMode::Hold,
        }
    }
}

impl LayerCycle {
    /// Nenhum lado cicla — o amostrador cru serve (atalho para o caminho comum).
    #[must_use]
    pub fn is_default(self) -> bool {
        self == Self::default()
    }
}

/// Mapeia `frame` para dentro do vão `[first, end)` conforme o ciclo. `None` =
/// nada aparece neste quadro. Pura, determinística e sem transcendentais (HR-5).
///
/// Dentro do vão a identidade vale (é o caminho quente). Fora, o lado
/// correspondente decide.
#[must_use]
pub fn map_frame(cycle: LayerCycle, span: (Frame, Frame), frame: Frame) -> Option<Frame> {
    let (first, end) = span;
    if end <= first {
        return None; // vão degenerado (camada sem chave real)
    }
    if frame >= first && frame < end {
        return Some(frame);
    }
    let mode = if frame < first { cycle.pre } else { cycle.post };
    let len = i64::from(end) - i64::from(first); // ≥ 1
    let off = i64::from(frame) - i64::from(first); // pode ser negativo
    let mapped = match mode {
        CycleMode::None => return None,
        CycleMode::Hold => {
            if frame < first {
                0
            } else {
                len - 1
            }
        }
        CycleMode::Loop => off.rem_euclid(len),
        CycleMode::PingPong => {
            if len < 2 {
                0
            } else {
                let period = 2 * len - 2;
                let x = off.rem_euclid(period);
                if x < len { x } else { period - x }
            }
        }
    };
    // `first + mapped` cabe em i32: `mapped ∈ [0, len)` e `first + len = end`.
    Some(first + mapped as Frame)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Um vão de 3 quadros: `[10, 13)` — chaves em 10, 11, 12.
    const SPAN: (Frame, Frame) = (10, 13);

    fn cyc(pre: CycleMode, post: CycleMode) -> LayerCycle {
        LayerCycle { pre, post }
    }

    #[test]
    fn inside_the_span_is_the_identity() {
        for f in 10..13 {
            assert_eq!(map_frame(LayerCycle::default(), SPAN, f), Some(f));
        }
    }

    /// O default REPRODUZ o pré-W3: nada antes, segura depois.
    #[test]
    fn the_default_matches_the_pre_cycle_behaviour() {
        let c = LayerCycle::default();
        assert_eq!(map_frame(c, SPAN, 9), None, "antes da 1ª chave: nada");
        assert_eq!(map_frame(c, SPAN, 0), None);
        assert_eq!(map_frame(c, SPAN, 13), Some(12), "depois: segura a última");
        assert_eq!(map_frame(c, SPAN, 999), Some(12));
    }

    #[test]
    fn loop_wraps_in_both_directions() {
        let c = cyc(CycleMode::Loop, CycleMode::Loop);
        // Depois: 13→10, 14→11, 15→12, 16→10 …
        assert_eq!(map_frame(c, SPAN, 13), Some(10));
        assert_eq!(map_frame(c, SPAN, 14), Some(11));
        assert_eq!(map_frame(c, SPAN, 16), Some(10));
        // Antes (rem_euclid é o que faz o negativo funcionar): 9→12, 8→11, 7→10.
        assert_eq!(map_frame(c, SPAN, 9), Some(12));
        assert_eq!(map_frame(c, SPAN, 8), Some(11));
        assert_eq!(map_frame(c, SPAN, 7), Some(10));
    }

    /// Ping-pong espelha SEM repetir as bordas: 10,11,12,11,10,11,12…
    #[test]
    fn pingpong_mirrors_without_repeating_the_edges() {
        let c = cyc(CycleMode::PingPong, CycleMode::PingPong);
        let seq: Vec<_> = (10..=17).map(|f| map_frame(c, SPAN, f).unwrap()).collect();
        assert_eq!(seq, vec![10, 11, 12, 11, 10, 11, 12, 11]);
        // E para trás, simétrico.
        assert_eq!(map_frame(c, SPAN, 9), Some(11));
        assert_eq!(map_frame(c, SPAN, 8), Some(12));
    }

    #[test]
    fn degenerate_spans_are_safe() {
        // Vão de 1 quadro: todo ciclo colapsa nele.
        let one = (5, 6);
        for m in [
            CycleMode::Loop,
            CycleMode::PingPong,
            CycleMode::Hold,
            CycleMode::None,
        ] {
            let c = cyc(m, m);
            let got = map_frame(c, one, 42);
            assert!(
                got == Some(5) || (m == CycleMode::None && got.is_none()),
                "{m:?} num vão de 1 quadro"
            );
        }
        // Vão vazio: nada, em qualquer modo.
        assert_eq!(map_frame(LayerCycle::default(), (7, 7), 7), None);
    }

    #[test]
    fn the_two_sides_are_independent() {
        let c = cyc(CycleMode::Hold, CycleMode::Loop);
        assert_eq!(map_frame(c, SPAN, 5), Some(10), "pre = Hold");
        assert_eq!(map_frame(c, SPAN, 14), Some(11), "post = Loop");
    }
}
