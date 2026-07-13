//! Desenho das alças de **raio de quina** (Live Corners) — módulo irmão do overlay de
//! âncoras (LOC cap).
//!
//! Uma bolinha por quina do path selecionado, sobre a **bissetriz**, ligada à âncora por
//! um fio fino. Arrastar para dentro arredonda; arrastar de volta afia.
//!
//! **Cheia** = a quina já tem raio. **Vazada** = a quina está afiada e a bolinha está só
//! estacionada, esperando ser puxada. É a mesma gramática do `connector.rs`: a forma e o
//! preenchimento carregam informação, para que o estado se leia sem clicar.
//!
//! O raio da bolinha e a posição dela **não são calculados aqui** — vêm de
//! `ph2d_vec_scene::corner_live` e de `ph2d_vec_edit::corner_handle`, as mesmas funções
//! que o hit-test usa. Um segundo número aqui faria o usuário clicar no meio da alça e não
//! pegar nada.
//!
//! Ao contrário do resto deste crate, as cores aqui saem de **tokens** (HR-15). O overlay
//! de âncoras e as alças de conector ainda usam literais — dívida que o doc-comment deles
//! registra desde a Fase 1. Esta alça nasce do lado certo da linha; migrar as outras duas
//! muda a APARÊNCIA do que já existe, e isso é decisão do Enio, não refactor de quem
//! passava por aqui.

use ph2d_tokens::{ColorToken, Theme};
use ph2d_vec_scene::corner_live::{CORNER_HANDLE_R_PX, CornerHandleView};
use ph2d_vector::{
    Affine, BezPath, Brush, Circle, Color as VelloColor, Fill, Point, Stroke, VectorScene,
};

/// Espessura do contorno da bolinha e do fio até a âncora, em pixels.
const LINE_PX: f64 = 1.5;

/// Desenha as alças de raio. `handles` já vem em MUNDO (a shell as calculou com
/// `ph2d_vec_edit::corner_handle::view`); `transform` leva mundo→tela.
///
/// A bolinha é desenhada em espaço de TELA — o raio é em pixels e não pode crescer com o
/// zoom —, então o ponto sobe pelo afim e o raio não.
pub fn draw_corner_handles(
    handles: &[CornerHandleView],
    transform: Affine,
    theme: Theme,
    target: &mut VectorScene,
) {
    // O token resolve num RGBA de 8 bits; o Vello quer o `Color` dele. É a mesma ponte que
    // o `token_to_vello` do editor-core faz (não dá para reusá-la: este crate não depende
    // do editor-core, e não deveria — a pipeline de render não conhece o chrome).
    let vello = |t: ColorToken| {
        let c = t.resolve(theme);
        VelloColor::from_rgba8(c.r, c.g, c.b, c.a)
    };
    let (fill, edge, wire) = (
        vello(ColorToken::Accent),
        vello(ColorToken::BorderEmph),
        vello(ColorToken::AccentSoft),
    );

    for h in handles {
        let p = transform * Point::new(h.pos[0], h.pos[1]);
        let a = transform * Point::new(h.anchor[0], h.anchor[1]);
        // O fio âncora→alça: diz a QUEM a bolinha pertence quando várias quinas estão
        // próximas, e torna a direção da bissetriz visível antes do arrasto começar.
        target.inner_mut().stroke(
            &Stroke::new(LINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(wire),
            None,
            &{
                let mut wire_path = BezPath::new();
                wire_path.move_to(a);
                wire_path.line_to(p);
                wire_path
            },
        );
        let dot = Circle::new(p, CORNER_HANDLE_R_PX);
        if h.rounded {
            target.inner_mut().fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(fill),
                None,
                &dot,
            );
        }
        // O contorno vem sempre: destaca a bolinha de qualquer fundo, e é o corpo INTEIRO
        // da alça enquanto a quina está afiada (vazada).
        target.inner_mut().stroke(
            &Stroke::new(LINE_PX),
            Affine::IDENTITY,
            &Brush::Solid(edge),
            None,
            &dot,
        );
    }
}
