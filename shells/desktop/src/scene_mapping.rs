//! ⭐⭐⭐ **A JANELA em que a CENA vive — a porta única de todo mapeamento mundo↔tela do chrome.**
//!
//! # A lei, e as quatro vezes que ela foi paga
//!
//! O doc do [`ph2d_app_motion::field_gizmo::scene_window_wh`] escreve-a desde 2026-07-25:
//! *«todo mapeamento mundo↔tela do chrome da cena TEM de usar isto»*. Sob um split do centro
//! (a ferramenta MOTION activa — ⛔ **não** a timeline) a cena **não é a janela**: ela desenha num sub-rectângulo
//! `[0, 0, w, h·t]`, e a projecção MUDA — não é um recorte.
//!
//! | quando | quem foi posto na porta | quem ficou de fora |
//! |---|---|---|
//! | 2026-07-25 | a grade e o gizmo | todo o resto |
//! | 2026-08-25 | o **pan** (a cena andava `t` vezes o que o cursor andava) | todo o resto |
//! | 2026-09-17 | o `vec_world_at` (o botão do HUD, `~340 px` ao lado) | **50 chamadas** |
//! | **hoje** | **as 40 ligações que alimentam a câmera** | — e um CENSO impede a quinta |
//!
//! ⛔⛔ *Três vezes a cura foi pôr **um** consumidor na porta e deixar os outros.* É por isso que
//! esta wave não acaba numa conversão: acaba num **censo** ([`crate::tests`] na shell), que reprova
//! a chamada nova que não passe por aqui.
//!
//! # ⚠️ Os DOIS eixos, e não só o `y`
//!
//! O [`ph2d_render::Camera2d::screen_to_world`] deriva `aspect = w/h` e `half_w = half_h·aspect`.
//! Sob um split HORIZONTAL o `h` encolhe ⇒ o `aspect` **cresce** ⇒ o `x` também sai errado. A
//! leitura *«é o y que está deslocado»* descreve o sintoma mais visível, não a conta.
//!
//! # ⚠️ Por que a porta mora no `AppGfx` e não no `App`
//!
//! Havia **DUAS** respostas à mesma pergunta — `App::scene_window` (`&self`) e
//! `field_gizmo_host::scene_window_of` (`&AppGfx`) — e a segunda existe porque a primeira pede o
//! `App` INTEIRO emprestado, o que colide com o `&mut gfx.<campo>` do mesmo quadro (o
//! `gizmo_drag.rs` tem isso escrito ao lado do código). ⇒ a porta pede o **mínimo** (`&AppGfx`) e
//! as outras duas **delegam**: quem empresta menos serve os dois chamadores, e o contrário não.
//!
//! # ⭐ Fora do split é BYTE-IDÊNTICO
//!
//! `CenterSplit::None` ⇒ [`scene_camera_window`] devolve a janela inteira. É essa propriedade que
//! torna a conversão de 40 sítios segura, e ela tem gate próprio.

use ph2d_app_motion::field_gizmo::scene_camera_window;
use ph2d_editor_core::screens::layout::CenterSplit;
use ph2d_host::WindowSize;

/// **A mesma janela, quando o `hero` JÁ está emprestado** — a forma que o empréstimo obriga.
///
/// ⛔⛔ **Ela não é uma segunda resposta: é a MESMA conta com menos pedido.** O
/// [`crate::AppGfx::scene_window`] pede o `AppGfx` inteiro, e metade dos gestos do canvas corre
/// dentro de um `if let Some(hero) = gfx.hero_screen.as_mut()` — ali o `gfx` inteiro já não se
/// empresta, e **o compilador recusa**. Foi isso, e não desleixo, que fez a lei ser violada em 50
/// sítios: *a porta pedia mais do que o sítio tinha para dar*, e o caminho que compilava era o
/// errado (`gfx.surface.size()`, que toca UM campo).
///
/// ⇒ quem tem o `hero` na mão passa o split dele e o tamanho da superfície, que são campos
/// **disjuntos**. As duas formas chamam a mesma função da crate do Motion.
pub(crate) fn janela(split: CenterSplit, superficie: WindowSize) -> WindowSize {
    scene_camera_window(split, superficie)
}

impl crate::AppGfx {
    /// **A janela da CENA** — a que toda conta mundo↔tela do chrome tem de usar.
    ///
    /// ⚠️ Fora do split é a janela inteira, bit a bit (ver o cabeçalho).
    pub(crate) fn scene_window(&self) -> WindowSize {
        let split = self
            .hero_screen
            .as_ref()
            .map_or(CenterSplit::None, |h| h.view.center_split);
        scene_camera_window(split, self.surface.size())
    }
}
