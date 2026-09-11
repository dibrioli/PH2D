//! ⭐⭐⭐ **AS ALÇAS DE VÉRTICE** (W133) — a projecção dos pontos que o artista digita.
//!
//! Enio, 2026-09-07: *«os vertex devem aparecer no canvas em tempo real e o usuário então poderá
//! movê-los através do gizmo no próprio canvas»*.
//!
//! # Por que um módulo-filho, e não uma linha a mais no pai
//!
//! O [`super`] chegou às `600` do gate de LOC do shell ao receber esta wave, e **o corte é por
//! assunto**: ele responde por um **VERBO** (mover, rodar, escalar o nó) e muda inteiro com o
//! seletor; isto responde pela **FORMA** — os pontos existem nos três verbos, porque um vértice não
//! é um verbo. ⛔ *Split, nunca allowlist.*
//!
//! ⚠️ **Módulo-filho por `#[path]`, como a lei do arrasto** (`field3d_gizmo_drag.rs`): ele partilha
//! os tipos e as constantes do pai por `use super::*`, e todos os caminhos que já existiam continuam
//! a resolver pelo re-export. *Cortar um arquivo não pode custar uma reescrita a cada chamador.*

use super::*;

/// ⭐⭐⭐ **AS ALÇAS DOS VÉRTICES** (W133) — uma por ponto do contorno, no plano da peça.
///
/// ⚠️ **Elas NÃO fazem parte do [`project`], e a separação é o desenho.** Aquele responde por um
/// VERBO (mover, rodar, escalar o nó) e muda inteiro com o seletor; estas são a **forma**, e existem
/// nos três verbos porque um vértice não é um verbo. *Juntá-las obrigaria o `project` a receber os
/// pontos e a decidir por modo — o mesmo `if` em três braços.*
///
/// A posição de mundo do vértice `(px, py)` é `origem + x·px + y·py`, com os eixos locais **já
/// escalados** ([`Anchor::local`]) — a mesma conta que o arrasto desfaz.
///
/// ⚠️ **Um vértice que não projecta é `live = false`**, e não é omitido: a lista é indexada pela
/// ordem, e saltar um faria a alça `n` passar a ser a `n+1` — a peça mexeria no ponto errado.
pub fn project_vertices(
    anchor: Anchor,
    points: &[[f32; 2]],
    cam: &Orbit,
    screen: Screen,
) -> Vec<Projected> {
    points
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let w = [
                anchor.origin[0] + anchor.local[0][0] * v[0] + anchor.local[1][0] * v[1],
                anchor.origin[1] + anchor.local[0][1] * v[0] + anchor.local[1][1] * v[1],
                anchor.origin[2] + anchor.local[0][2] * v[0] + anchor.local[1][2] * v[1],
            ];
            let center = cam.project(w, screen).map(|(p, _)| p);
            Projected {
                handle: Handle::Vertex(i),
                shape: Shape::Point {
                    center: center.unwrap_or([f32::NAN; 2]),
                },
                live: center.is_some(),
            }
        })
        .collect()
}
