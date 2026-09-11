//! **A cena pronta para o smoke do gizmo de SELEÇÃO** (`PH2D_FLIP_XFORM_SMOKE=1`, §4.A).
//!
//! O Enio não monta cena (feedback_ready_to_smoke_example): o app abre com 1 objeto
//! Flip de arte EXCLUSIVA — um retângulo (SELECIONADO) + um triângulo (não) —, a tool
//! Flip em modo **Edit** e o gizmo da SELEÇÃO já visível enquadrando o retângulo.
//!
//! Roteiro: arrastar uma **quina** gira (anel de hover) / escala o retângulo em torno
//! do centro DELE; uma **borda** escala num eixo; arrastar a arte (fora dos handles)
//! **move** a seleção (o gesto do W6.1). Conferir que o **triângulo não se mexe**, que
//! **Ctrl+Z** desfaz o gesto inteiro (1 passo), e que clicar no vazio **fora** da caixa
//! (desmarca) **some** com o gizmo.
//!
//! **Os achados dos smokes anteriores têm alvo próprio na cena:**
//! - O retângulo é **VAZADO** (sem fill) de propósito: arrastar do **meio dele** (onde não
//!   há tinta nenhuma) tem de **agarrar a seleção** — antes virava marquee e ainda limpava
//!   a seleção. É a área do gizmo inteira que pega, não só a tinta.
//! - As **3 linhas** do triângulo e as **4** do retângulo são clicáveis, inclusive a de
//!   fechamento (a costura — BUGS #18).
//! - **O toggle `Select: Point` some com o gizmo NA HORA** — porque entrar no Point
//!   **começa DESSELECIONADO** (o gesto seguinte ali é escolher âncoras), e sem seleção não
//!   há caixa.
//! - **UMA âncora = sem gizmo** (não se rotaciona nem se escalona um ponto), só o realce —
//!   e ela arrasta. **DUAS ou mais = o gizmo volta**, enquadrando SÓ as âncoras acesas, e
//!   gira/escala só elas. Voltar ao **Stroke** promove por `any()` (o traço cujas âncoras
//!   você tocou fica selecionado) e o gizmo passa a enquadrar o traço inteiro.
//! - **A caixa tem FOLGA**: ela é sempre maior que o que move, então os handles ficam FORA
//!   das âncoras (dá para clicar no ponto) e **nunca se sobrepõem** — nem com duas âncoras
//!   na MESMA horizontal, em que a caixa crua teria altura zero. A folga mede o mesmo em
//!   qualquer zoom (é chrome, px de tela).

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::AtomicU32;

pub static FRAME: AtomicU32 = AtomicU32::new(0);

pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_XFORM_SMOKE").is_some())
}

/// Uma polilinha fechada pelos vértices dados, largura de tela grossa.
pub fn shape(verts: &[Vec2], color: Rgba, selected: bool) -> FlipStroke {
    let mut s = FlipStroke::new();
    for &p in verts {
        s.push_point(Point {
            pos: p,
            width: 6.0,
            opacity: 1.0,
            color,
        });
    }
    s.closed = true;
    s.selected = selected;
    s
}
