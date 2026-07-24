//! Ghost Frames (onion skin) — configuração **e a seleção dos vizinhos** (W3.T3.3).
//!
//! Nome de produto: **Ghost Frames** (não "onion skin"). Defaults canônicos do
//! Grease Pencil 5.2 (`DNA_grease_pencil_types.h:409-436`).
//!
//! A escolha de QUEM aparece como fantasma é uma **função pura** ([`ghosts`]) —
//! port do `get_frame_id` (`grease_pencil_utils.cc:534-603`, `02_referencia §8`).
//! Ela é pura de propósito: o render só desenha a lista que ela devolve, e o teste
//! prova os 3 modos sem GPU nenhuma.
//!
//! Duas armadilhas do original, portadas conscientemente:
//! - **Δ conta CHAVES, não quadros** (no modo `Relative`, o default): num hold de
//!   12 quadros o fantasma Δ=−1 é o DESENHO anterior, não o quadro anterior (que
//!   seria o mesmo desenho). É isso que faz o onion do animador funcionar.
//! - **Antes da 1ª chave, `Δ += 1`**: senão a primeira chave (que ainda está no
//!   futuro) sairia com Δ=0 e seria confundida com a corrente.

use crate::color::Rgba;
use crate::frame::KeyKind;
use crate::ids::{DrawingId, Frame};
use crate::layer::FlipLayer;
use serde::{Deserialize, Serialize};

/// Como os deltas de vizinhança são contados.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum OnionMode {
    /// Δ em número absoluto de frame.
    Absolute,
    /// Δ em número de keyframe (o default do GP) — pula holds, conta DESENHOS.
    #[default]
    Relative,
    /// Só os keyframes selecionados (**ignora o alcance** before/after — é o GP).
    Selected,
}

/// Piso da opacidade de um fantasma (o GP nunca deixa um ghost sumir de todo).
pub const GHOST_MIN_ALPHA: f32 = 0.1;

/// Configuração de Ghost Frames de um objeto. Defaults = GP 5.2.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OnionSettings {
    /// Liga/desliga o desenho dos fantasmas.
    pub enabled: bool,
    /// Opacidade global dos fantasmas (`0.5` no GP).
    pub opacity: f32,
    /// Quantos quadros antes mostrar (`1`).
    pub frames_before: u32,
    /// Quantos quadros depois mostrar (`1`).
    pub frames_after: u32,
    /// Tint dos quadros anteriores (verde do GP: `0.145, 0.420, 0.137`). Os
    /// valores são os do GP (display); o espaço de cor exato é resolvido no
    /// render.
    pub color_before: Rgba,
    /// Tint dos quadros seguintes (azul do GP: `0.125, 0.082, 0.529`).
    pub color_after: Rgba,
    /// Esmaecer com a distância (`alpha ∝ 1/|Δ|`) — `USE_FADE` ligado no GP.
    pub fade: bool,
    /// Como contar os deltas.
    pub mode: OnionMode,
    /// Filtro por tipo de chave: bitmask de `1 << (KeyKind as u8)`. **`0` = tudo
    /// passa** (a bitmask VAZIA do GP não filtra nada — não é "nada passa").
    pub kind_filter: u8,
}

impl Default for OnionSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            opacity: 0.5,
            frames_before: 1,
            frames_after: 1,
            color_before: Rgba::new(0.145, 0.420, 0.137, 1.0),
            color_after: Rgba::new(0.125, 0.082, 0.529, 1.0),
            fade: true,
            mode: OnionMode::Relative,
            kind_filter: 0,
        }
    }
}

/// Um fantasma a desenhar: o desenho, a chave de origem e como pintá-lo.
///
/// `tint` é a cor da **silhueta 100% recolorida** (o ghost do GP NÃO é um blend da
/// arte original — é a forma dela, chapada na cor do lado); `alpha` já traz o fade
/// e a opacidade global, com o piso do GP aplicado.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ghost {
    /// A chave de origem (quadro).
    pub key: Frame,
    /// O desenho a rasterizar.
    pub drawing: DrawingId,
    /// Distância à corrente: negativo = passado, positivo = futuro (nunca `0`).
    pub delta: i32,
    /// Cor da silhueta (before/after).
    pub tint: Rgba,
    /// Opacidade final `[GHOST_MIN_ALPHA, 1]`.
    pub alpha: f32,
}

/// **Os fantasmas da camada `layer` no quadro `current`** — a função pura do onion.
///
/// `selected` são as chaves selecionadas na tira (só o modo [`OnionMode::Selected`]
/// as lê). A lista sai ordenada por `|Δ|` DECRESCENTE: o render desenha do mais
/// distante ao mais próximo, então o vizinho imediato fica por cima — a ordem do GP.
///
/// `pinned` é o **light table** (T3.9, o *bookmark* do TVPaint/OpenToonz): quadros que o
/// animador marcou como REFERÊNCIA e que aparecem **além** dos vizinhos — não em vez
/// deles. Um pin ignora modo, alcance e filtro de tipo, e a razão é a mesma nos três: cada
/// um deles responde *"que vizinho conta?"*, e um pin não é um vizinho — é uma escolha
/// explícita. (O que ele NÃO ignora é a chave ativa: essa já está na tela, e desenhá-la
/// como fantasma pintaria a silhueta por cima do próprio desenho.)
///
/// Os *gates* (`enabled`, "some no play", camada com `use_onion`) são do CHAMADOR:
/// esta função responde "quem seria", não "se é hora".
#[must_use]
pub fn ghosts(
    layer: &FlipLayer,
    current: Frame,
    s: &OnionSettings,
    selected: &[Frame],
    pinned: &[Frame],
) -> Vec<Ghost> {
    // As chaves REAIS (sentinela de fim não é desenho), em ordem, indexadas.
    let keys: Vec<(Frame, DrawingId, KeyKind)> = layer
        .frames()
        .iter()
        .filter_map(|(&k, f)| f.drawing.map(|d| (k, d, f.kind)))
        .collect();
    if keys.is_empty() {
        return Vec::new();
    }
    // A chave ATIVA agora (a maior ≤ current). `None` = estamos ANTES da 1ª chave —
    // e é aí que o `+1` do GP entra (senão a 1ª chave sairia com Δ=0 = "corrente").
    let active = keys.iter().rposition(|(k, _, _)| *k <= current);
    let before_first = active.is_none();

    let mut out: Vec<Ghost> = Vec::new();
    for (i, &(key, drawing, kind)) in keys.iter().enumerate() {
        if Some(i) == active {
            continue; // a corrente entra no passe normal, não como fantasma
        }
        // Um PIN entra sempre, pelo Δ absoluto (que é o que decide cor e fade). Os três
        // filtros abaixo perguntam "que vizinho conta?", e um pin não é um vizinho.
        let is_pinned = pinned.contains(&key);
        if !is_pinned && s.kind_filter != 0 && s.kind_filter & (1u8 << (kind as u8)) == 0 {
            continue;
        }
        let raw = if is_pinned {
            key - current
        } else {
            match s.mode {
                OnionMode::Selected => {
                    if !selected.contains(&key) {
                        continue;
                    }
                    key - current // SELECTED ignora o alcance; o Δ só decide cor/fade
                }
                OnionMode::Absolute => key - current,
                OnionMode::Relative => i as i32 - active.unwrap_or(0) as i32,
            }
        };
        let delta = if before_first { raw + 1 } else { raw };
        if delta == 0 {
            continue;
        }
        if !is_pinned && s.mode != OnionMode::Selected {
            let range = if delta < 0 {
                s.frames_before
            } else {
                s.frames_after
            };
            if delta.unsigned_abs() > range {
                continue;
            }
        }
        let fade = if s.fade {
            1.0 / delta.unsigned_abs() as f32
        } else {
            1.0
        };
        out.push(Ghost {
            key,
            drawing,
            delta,
            tint: if delta < 0 {
                s.color_before
            } else {
                s.color_after
            },
            alpha: (fade * s.opacity).clamp(GHOST_MIN_ALPHA, 1.0),
        });
    }
    // Do mais distante ao mais próximo (o vizinho imediato pinta por último = por
    // cima). Desempate por chave: determinístico.
    out.sort_by_key(|g| (std::cmp::Reverse(g.delta.abs()), g.key));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::Hold;
    use crate::ids::LayerId;

    /// Uma camada com chaves em `keys` (desenhos 0..n), holds implícitos.
    fn layer_with(keys: &[Frame]) -> FlipLayer {
        let mut l = FlipLayer::new(LayerId(0), "L");
        for (i, &k) in keys.iter().enumerate() {
            l.add_frame(
                k,
                Some(DrawingId(i as u32)),
                KeyKind::Keyframe,
                Hold::Implicit,
            );
        }
        l
    }

    #[test]
    fn defaults_mirror_grease_pencil() {
        let o = OnionSettings::default();
        assert_eq!(o.opacity, 0.5);
        assert_eq!((o.frames_before, o.frames_after), (1, 1));
        assert!(o.fade);
        assert_eq!(o.mode, OnionMode::Relative);
        assert_eq!(o.color_before, Rgba::new(0.145, 0.420, 0.137, 1.0));
        assert_eq!(o.kind_filter, 0, "bitmask vazia = tudo passa");
    }

    /// **RELATIVE conta DESENHOS, não quadros** — a razão de ser do modo. No meio
    /// do hold da chave 12, o fantasma −1 é a chave 0 (o desenho anterior).
    #[test]
    fn relative_counts_drawings_across_holds() {
        let l = layer_with(&[0, 12, 24]);
        let s = OnionSettings::default(); // 1 antes, 1 depois, Relative
        let g = ghosts(&l, 18, &s, &[], &[]); // dentro do hold da chave 12
        let got: Vec<_> = g.iter().map(|g| (g.key, g.delta)).collect();
        assert_eq!(got, vec![(0, -1), (24, 1)], "os DESENHOS vizinhos");
    }

    /// ABSOLUTE conta QUADROS: sobre a chave 12, com alcance 1/1, os vizinhos
    /// estão a 12 quadros — nenhum fantasma (onde o RELATIVE mostraria os dois).
    /// É exatamente a diferença que o modo faz.
    #[test]
    fn absolute_counts_frames() {
        let l = layer_with(&[0, 12, 24]);
        let mut s = OnionSettings {
            mode: OnionMode::Absolute,
            ..Default::default()
        };
        assert!(
            ghosts(&l, 12, &s, &[], &[]).is_empty(),
            "ninguém a ±1 QUADRO"
        );
        assert_eq!(
            ghosts(&l, 12, &OnionSettings::default(), &[], &[]).len(),
            2,
            "no Relative, os mesmos vizinhos estão a ±1 DESENHO"
        );
        // Com alcance de 12 quadros, os dois entram.
        s.frames_before = 12;
        s.frames_after = 12;
        let g = ghosts(&l, 12, &s, &[], &[]);
        let got: Vec<_> = g.iter().map(|g| (g.key, g.delta)).collect();
        assert_eq!(got, vec![(0, -12), (24, 12)]);
    }

    /// SELECTED ignora o alcance (o comportamento do GP, portado de propósito).
    #[test]
    fn selected_ignores_the_range() {
        let l = layer_with(&[0, 12, 24, 36]);
        let s = OnionSettings {
            mode: OnionMode::Selected,
            frames_before: 1,
            frames_after: 1,
            ..Default::default()
        };
        let g = ghosts(&l, 12, &s, &[0, 36], &[]);
        let got: Vec<_> = g.iter().map(|g| g.key).collect();
        assert_eq!(got, vec![36, 0], "distante primeiro; o alcance não filtrou");
    }

    /// 🔴 **O light table entra ALÉM do alcance, não em vez dele** (T3.9).
    ///
    /// É a diferença entre um pin e o modo `Selected`: o `Selected` SUBSTITUI a vizinhança
    /// (só o que está marcado aparece), o pin a ACOMPANHA. Um animador que fixa o extremo
    /// de uma cena quer vê-lo *e* continuar vendo o quadro anterior enquanto desenha — foi
    /// para isso que o TVPaint/OpenToonz inventaram o bookmark.
    ///
    /// Mutação que sangra: filtrar o pin pelo alcance (`is_pinned` fora da condição) — a
    /// chave 36 some e sobra só a vizinhança.
    #[test]
    fn a_pinned_key_shows_up_alongside_the_neighbours_not_instead_of_them() {
        let l = layer_with(&[0, 12, 24, 36]);
        let s = OnionSettings {
            frames_before: 1,
            frames_after: 1,
            ..Default::default()
        };
        let g = ghosts(&l, 12, &s, &[], &[36]);
        let got: Vec<_> = g.iter().map(|g| g.key).collect();
        // A ordem é a de sempre — `|Δ|` decrescente, desempate por chave: o pin tem
        // `Δ = +24` (absoluto, porque um pin não conta vizinhança) e os dois vizinhos
        // empatam em `|Δ| = 1` no modo Relative, então o 0 precede o 24. (Eu escrevi
        // `[36, 24, 0]` aqui antes de calcular, e o gate nasceu vermelho sobre um produto
        // correto — a expectativa é que estava errada.)
        assert_eq!(
            got,
            vec![36, 0, 24],
            "o pin (36) entrou SEM tirar os vizinhos (0 e 24)"
        );
    }

    /// 🔴 **Um pin não é contado duas vezes** quando também cai no alcance — a lista é de
    /// fantasmas, e o mesmo desenho pintado duas vezes dobraria a opacidade dele.
    #[test]
    fn a_pin_that_is_also_a_neighbour_appears_once() {
        let l = layer_with(&[0, 12, 24]);
        let s = OnionSettings::default(); // ±1 vizinho
        let g = ghosts(&l, 12, &s, &[], &[0, 24]);
        assert_eq!(g.len(), 2, "dois fantasmas, não quatro: {g:?}");
    }

    /// 🔴 **O pin sobrevive ao filtro de TIPO.** O `kind_filter` responde *"que vizinho
    /// conta?"*, e um pin não é um vizinho: o artista o escolheu a dedo. Sem esta isenção,
    /// fixar um breakdown num documento filtrado por keyframes dá um botão que não faz nada.
    #[test]
    fn a_pin_survives_the_kind_filter() {
        let mut l = FlipLayer::new(LayerId(0), "L");
        l.add_frame(0, Some(DrawingId(0)), KeyKind::Keyframe, Hold::Implicit);
        l.add_frame(6, Some(DrawingId(1)), KeyKind::Breakdown, Hold::Implicit);
        l.add_frame(12, Some(DrawingId(2)), KeyKind::Keyframe, Hold::Implicit);
        let s = OnionSettings {
            kind_filter: 1 << (KeyKind::Keyframe as u8), // só keyframes
            frames_before: 4,
            frames_after: 4,
            ..Default::default()
        };
        assert!(
            !ghosts(&l, 0, &s, &[], &[]).iter().any(|g| g.key == 6),
            "sem pin, o breakdown é filtrado (o controle)"
        );
        assert!(
            ghosts(&l, 0, &s, &[], &[6]).iter().any(|g| g.key == 6),
            "com pin, o breakdown aparece"
        );
    }

    /// A chave ATIVA nunca vira fantasma, nem fixada: ela já está na tela, e a silhueta
    /// tingida seria pintada por cima do próprio desenho.
    #[test]
    fn pinning_the_current_key_does_not_ghost_it() {
        let l = layer_with(&[0, 12, 24]);
        let s = OnionSettings::default();
        let g = ghosts(&l, 12, &s, &[], &[12]);
        assert!(!g.iter().any(|g| g.key == 12), "a corrente não se fantasma");
    }

    /// Antes da 1ª chave, todo Δ ganha `+1` — senão a 1ª chave (futura) sairia com
    /// Δ=0, seria confundida com a corrente e sumiria da lista.
    #[test]
    fn before_the_first_key_every_delta_shifts_by_one() {
        let l = layer_with(&[10, 20]);
        let s = OnionSettings {
            frames_after: 2,
            ..Default::default()
        };
        let g = ghosts(&l, 5, &s, &[], &[]); // antes de tudo
        let got: Vec<_> = g.iter().map(|g| (g.key, g.delta)).collect();
        assert_eq!(got, vec![(20, 2), (10, 1)], "a 1ª chave é Δ=+1, não 0");
    }

    /// O fade cai como 1/|Δ| e satura no piso (o ghost nunca some).
    #[test]
    fn fade_falls_as_one_over_delta_with_a_floor() {
        let keys: Vec<Frame> = (0..=12).collect();
        let l = layer_with(&keys);
        let s = OnionSettings {
            frames_before: 12,
            opacity: 1.0,
            ..Default::default()
        };
        let g = ghosts(&l, 12, &s, &[], &[]);
        let by_delta = |d: i32| g.iter().find(|g| g.delta == d).unwrap().alpha;
        assert!((by_delta(-1) - 1.0).abs() < 1e-6);
        assert!((by_delta(-2) - 0.5).abs() < 1e-6);
        assert!((by_delta(-3) - 1.0 / 3.0).abs() < 1e-6);
        assert!(
            (by_delta(-12) - GHOST_MIN_ALPHA).abs() < 1e-6,
            "piso, não zero"
        );
        // O mais distante vem primeiro (é desenhado primeiro = fica por baixo).
        assert_eq!(g[0].delta, -12);
        assert_eq!(g[0].tint, s.color_before, "passado = verde");
    }

    #[test]
    fn the_kind_filter_selects_by_key_type() {
        let mut l = layer_with(&[0, 10]);
        l.add_frame(5, Some(DrawingId(9)), KeyKind::Breakdown, Hold::Implicit);
        let s = OnionSettings {
            frames_before: 4,
            frames_after: 4,
            kind_filter: 1u8 << (KeyKind::Keyframe as u8), // só keyframes
            ..Default::default()
        };
        let g = ghosts(&l, 10, &s, &[], &[]);
        assert!(
            g.iter().all(|g| g.key != 5),
            "o breakdown foi filtrado: {g:?}"
        );
        assert!(g.iter().any(|g| g.key == 0));
    }

    #[test]
    fn round_trips_through_postcard() {
        let o = OnionSettings::default();
        let bytes = postcard::to_allocvec(&o).unwrap();
        let back: OnionSettings = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(back, o);
    }
}
