//! **Os INVÓLUCROS dos gestos de junta** — a costura entre `&mut App` e a
//! assinatura que a [`ph2d_app_physics`] expõe (W2/L2 Fase C, 2026-09-12).
//!
//! ⚠️⚠️ **Isto NÃO é dívida — é o PADRÃO DE CHEGADA desta wave** (`ESTADO_W2` §2,
//! medido na `line/app-flip`: *«o que sobra dela na shell são 36 invólucros
//! `*_app.rs` … isso é a costura, e fica por desenho»*). A lei inteira dos três
//! gestos vive na crate; o que fica aqui é o que decide a ordem do quadro e o
//! que lê campos da `App` — que é exactamente o corte do HOWTO §4.
//!
//! # ⭐ O que cada invólucro faz, e por que é ele a fazê-lo
//!
//! ⛔⛔ **E o construtor do contexto é REPETIDO nos quatro, de propósito — uma
//! função auxiliar aqui NÃO compila.** A 1.ª versão tinha um
//! `fn physics_canvas_ctx(&mut self) -> Option<CanvasCtx<'_>>`, e ele empresta
//! **`self` INTEIRO**: o `joint_draw_move` a seguir pedia `&mut self.physics` e
//! o compilador respondia `E0499`. ⇒ o empréstimo disjunto só existe quando os
//! campos são nomeados **no mesmo escopo** que os usa. *É o mesmo mecanismo que
//! obrigava o `resolve_player_input` a um `std::mem::take` e o `disarm` a ser
//! função livre — e ele não desaparece por se lhe dar um nome.*
//!
//! 1. **Constrói o [`CanvasCtx`]** a partir dos seis campos do `AppGfx`. Os seis
//!    empréstimos são disjuntos por construção, e é por isso que isto cabe numa
//!    expressão só: cada um é um campo diferente.
//! 2. **Escreve o `any_input_this_frame`.** Ele é da `App` e governa se o quadro
//!    regista um passo de undo — pô-lo na crate exigiria uma **porta nova** no
//!    `AppHost`, e nenhuma das três funções precisa de mais nada dela.
//!    ⚠️ **A ordem é load-bearing e é a de antes:** *armado* ⇒ a mão fez algo,
//!    mesmo que o gesto não chegue a produzir nada. Um `press` num quadro sem
//!    `gfx` continua a marcar o quadro como autorado, como marcava.
//! 3. **Traduz o modificador.** O `advance_joint_anchor_drag` recebe `ctrl: bool`
//!    já resolvido: Ctrl (Cmd no macOS) é o modificador de encaixe do editor — a
//!    mesma tecla que a alça do pivô lê — e quem sabe a tradução é quem tem o
//!    teclado. Passar o estado cru poria a mesma lei em dois sítios.
//!
//! ⛔ **Zero sextos métodos no `AppHost`.** Escritos em tipos, os três gestos
//! pediam sete coisas que a `App` por acaso segurava, e nenhuma é a `App`: a
//! tabela está no doc do [`CanvasCtx`]. ⛔ Um método que devolvesse o `gfx` seria
//! um HANDLE, que o trait proíbe por escrito — a fronteira desfazia-se nele.

use ph2d_app_physics::{CanvasCtx, joint_anchor_drag, joint_draw};

use crate::App;

impl App {
    /// **Esc cancela**, e só consome a tecla quando há o que cancelar.
    pub(crate) fn joint_draw_cancel_key(&mut self) -> bool {
        if !self.physics.joint_draw_armed {
            return false;
        }
        self.any_input_this_frame = true;
        let Some(gfx) = self.gfx.as_mut() else {
            // Sem janela não há toast a mostrar, mas o gesto morre na mesma: um
            // modo que sobrevivesse ao Esc tomaria o Move/Up seguinte.
            joint_draw::disarm_joint_draw(&mut self.physics);
            return true;
        };
        joint_draw::joint_draw_cancel_key(&mut self.physics, &mut gfx.toasts)
    }

    /// **O press.** Com o gesto ARMADO, começa a banda no corpo sob o cursor.
    pub(crate) fn joint_draw_press(&mut self, sx: f32, sy: f32) -> bool {
        if !self.physics.joint_draw_armed {
            return false;
        }
        self.any_input_this_frame = true;
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let mut ctx = CanvasCtx {
            camera: &gfx.camera,
            window: gfx.scene_window(),
            sim: &mut gfx.sim,
            bridge: &gfx.physics,
            present: &mut gfx.present,
            toasts: &mut gfx.toasts,
            hero: gfx.hero_screen.as_mut(),
        };
        joint_draw::joint_draw_press(&mut self.physics, &mut ctx, sx, sy)
    }

    /// **O arrasto.** Só move a ponta da banda; nada é autorado até o release.
    pub(crate) fn joint_draw_move(&mut self, sx: f32, sy: f32) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let ctx = CanvasCtx {
            camera: &gfx.camera,
            window: gfx.scene_window(),
            sim: &mut gfx.sim,
            bridge: &gfx.physics,
            present: &mut gfx.present,
            toasts: &mut gfx.toasts,
            hero: gfx.hero_screen.as_mut(),
        };
        joint_draw::joint_draw_move(&mut self.physics, &ctx, sx, sy);
    }

    /// **O release** — onde o joint nasce, ou onde a recusa é explicada.
    pub(crate) fn joint_draw_release(&mut self, sx: f32, sy: f32) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let mut ctx = CanvasCtx {
            camera: &gfx.camera,
            window: gfx.scene_window(),
            sim: &mut gfx.sim,
            bridge: &gfx.physics,
            present: &mut gfx.present,
            toasts: &mut gfx.toasts,
            hero: gfx.hero_screen.as_mut(),
        };
        joint_draw::joint_draw_release(&mut self.physics, &mut ctx, sx, sy);
    }

    /// Segue o cursor com o arrasto de âncora aberto. No-op quando não há um.
    pub(crate) fn advance_joint_anchor_drag(&mut self) {
        if self.physics.joint_anchor_drag.is_none() {
            return;
        }
        // Ctrl (Cmd no macOS) é o modificador de encaixe do editor — a mesma
        // tecla que a alça do pivô lê, para um gesto significar uma coisa.
        let ctrl = self.modifiers.control_key() || self.modifiers.super_key();
        let pointer = self.last_pointer;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let mut ctx = CanvasCtx {
            camera: &gfx.camera,
            window: gfx.scene_window(),
            sim: &mut gfx.sim,
            bridge: &gfx.physics,
            present: &mut gfx.present,
            toasts: &mut gfx.toasts,
            hero: gfx.hero_screen.as_mut(),
        };
        joint_anchor_drag::advance_joint_anchor_drag(&mut self.physics, &mut ctx, ctrl, pointer);
    }
}
