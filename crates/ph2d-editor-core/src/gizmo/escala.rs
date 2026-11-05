//! ⭐ **A lei do par travado no canvas** ([`crate::par_travado`]): a escala de um arrasto guarda o sinal
//! do objecto, um eixo a zero recupera-se pela alça, e o pivô é o oposto da mão.

use super::drag::{GizmoDragKind, GizmoDragState, GizmoTarget, TransformSnapshot};
use super::transform::anchor_pivot_world;

/// O menor módulo a que um arrasto leva um eixo: a caixa fica por pegar. Um eixo que já parte abaixo
/// disto fica onde está (agarrar não o puxa para cima).
const PISO_DO_GESTO: f32 = 0.001;
/// Abaixo disto um eixo lê-se zero: não há razão a ler (o mesmo ε de `ParTravado`).
pub(super) const EIXO_A_ZERO: f32 = 1e-6;

/// ⭐ A lei do par travado (`crate::par_travado`) no canvas: o sinal é da escala de partida e um arrasto
/// não o troca (espelhar é verbo; atravessar o pivô pára no piso, com o mesmo sinal).
pub(super) fn escala_do_eixo(partida: f32, razao: f32) -> f32 {
    let piso = PISO_DO_GESTO.min(partida.abs());
    partida.signum() * (partida.abs() * razao).max(piso)
}

/// ⭐ Um eixo a zero não tem razão a ler nem lado oposto: as duas bordas estão na mesma reta e abrem
/// juntas, com a borda sob o cursor. `andou` é o deslocamento de MUNDO desde o pen-down; sem caixa
/// intrínseca (`meio == 0`), uma unidade por unidade. Um zero não tem sinal: abre positivo.
pub(super) fn abre_do_zero(andou: f32, meio: f32, pai: f32) -> f32 {
    let ou_um = |v: f32| if v.abs() > EIXO_A_ZERO { v.abs() } else { 1.0 };
    andou.abs() / ou_um(meio) / ou_um(pai)
}

/// ⭐ O gizmo GLOBAL é do grupo: a razão sai do cursor e do pivô partilhado, e o zero de um eixo do
/// primário não lhe tira o fator. Dá ao eixo a zero uma partida de `1`, para a conta devolver o fator
/// nele; quem a escreve repõe o zero por [`escala_do_primario`] (`0 × fator`, como todo membro a zero).
#[must_use]
pub fn partida_do_grupo(mut drag: GizmoDragState) -> GizmoDragState {
    if matches!(drag.target, GizmoTarget::Global) {
        for s in &mut drag.start_transform.scale {
            if s.abs() <= EIXO_A_ZERO {
                *s = 1.0;
            }
        }
    }
    drag
}

/// O que se escreve no primário: no gizmo global, um eixo a zero fica a zero. Fora dele, um eixo a
/// zero abre pela alça.
#[must_use]
pub fn escala_do_primario(drag: &GizmoDragState, nova: [f32; 2]) -> [f32; 2] {
    let real = drag.start_transform.scale;
    let global = matches!(drag.target, GizmoTarget::Global);
    [0, 1].map(|e| {
        if global && real[e].abs() <= EIXO_A_ZERO {
            real[e]
        } else {
            nova[e]
        }
    })
}

/// O fator dos membros: de um eixo a zero não há razão a ler, e os outros não mudam nele.
#[must_use]
pub fn fator_do_grupo(partida: [f32; 2], nova: [f32; 2]) -> [f32; 2] {
    [0, 1].map(|e| {
        if partida[e].abs() > EIXO_A_ZERO {
            nova[e] / partida[e]
        } else {
            1.0
        }
    })
}

/// Seleção rodada: a escala de grupo é proporcional, pelo eixo que a mão mais move. O eixo a zero não
/// entra na conta: abre pela alça, ou o arrasto ficava morto.
#[must_use]
pub fn uniforme_do_grupo(partida: [f32; 2], nova: [f32; 2]) -> [f32; 2] {
    let [fx, fy] = fator_do_grupo(partida, nova);
    let u = if (fx - 1.0).abs() >= (fy - 1.0).abs() {
        fx
    } else {
        fy
    };
    [0, 1].map(|e| {
        if partida[e].abs() > EIXO_A_ZERO {
            partida[e] * u
        } else {
            nova[e]
        }
    })
}

/// ⭐ O pivô de uma escala visto da mão: o oposto da alça ONDE ELA ESTÁ. Os sinais de `kind` são os da
/// caixa pintada, com meio-tamanho positivo; `anchor_pivot_world` multiplica-os pela escala com sinal,
/// e num eixo espelhado o «oposto» caía em cima da própria alça. Sem espelho, a resposta é a dele,
/// ao bit. Quem manda no lado é o cursor do pen-down, seja qual for a convenção de quem pintou (o vetor
/// põe o espelho no meio-tamanho; o sprite, em X, numa rotação de meia volta).
#[must_use]
pub fn grip_pivot_world(
    kind: GizmoDragKind,
    sprite_anchor: [f32; 2],
    sprite_half_size: [f32; 2],
    snap: TransformSnapshot,
    cursor_world: [f32; 2],
) -> [f32; 2] {
    let pivot = anchor_pivot_world(kind, sprite_anchor, sprite_half_size, snap, false);
    if !matches!(
        kind,
        GizmoDragKind::ScaleCorner { .. } | GizmoDragKind::ScaleEdge { .. }
    ) {
        return pivot;
    }
    let (sin_r, cos_r) = libm::sincosf(snap.rotation);
    let (ax, ay) = (
        sprite_anchor[0] * snap.scale[0],
        sprite_anchor[1] * snap.scale[1],
    );
    let centro = [
        snap.translation[0] + ax * cos_r - ay * sin_r,
        snap.translation[1] + ax * sin_r + ay * cos_r,
    ];
    let na_caixa = |p: [f32; 2]| {
        let (dx, dy) = (p[0] - centro[0], p[1] - centro[1]);
        [dx * cos_r + dy * sin_r, -dx * sin_r + dy * cos_r]
    };
    let (p, c) = (na_caixa(pivot), na_caixa(cursor_world));
    let mesmo_lado = [p[0] * c[0] > 0.0, p[1] * c[1] > 0.0];
    if mesmo_lado == [false, false] {
        return pivot;
    }
    let [ox, oy] = [0, 1].map(|e| if mesmo_lado[e] { -p[e] } else { p[e] });
    [
        centro[0] + ox * cos_r - oy * sin_r,
        centro[1] + ox * sin_r + oy * cos_r,
    ]
}
