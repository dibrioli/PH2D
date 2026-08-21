//! Os gates da **COSTURA** do gesto — o caminho inteiro: ponteiro, alça, lei, intent.
//!
//! ⭐ **A costura ponteiro → gizmo → peça**, no caminho de produção inteiro.
//!
//! ⚠️ **É este o gate que a `DIRETIVA_IMPLEMENTACAO` §1 exige**, e não os dois de cima. Ele pergunta
//! *"clicar numa seta faz a peça andar?"* — a pergunta que a lei pura e a pintura, cada uma verde no
//! seu canto, **não** respondem. A causa nº 1 da semana perdida no Painter foi exatamente esta:
//! costura não-testada, com os dois lados dela corretos.

//! ⚠️ Módulo-filho do arquivo de gates da entrada: `use super::*` traz as fixtures do pai, que
//! continuam a existir **uma vez**.

use crate::field3d_gizmo::{self, Handle};
use crate::field3d_input::{advance, begin, hot_handle};
use crate::field3d_smoke::{Drag, set_armed_by_panel, with_smoke};
use ph2d_field_render::Screen;

const AREA: ph2d_editor::zones::Rect = ph2d_editor::zones::Rect {
    x: 40.0,
    y: 24.0,
    w: 800.0,
    h: 600.0,
};

/// Arma o módulo e põe o smoke num estado de quadro: com área desenhada e com o gizmo ancorado
/// na origem. É o que a ponte com a cena publica.
fn armed<R>(f: impl FnOnce(&mut crate::field3d_smoke::Smoke) -> R) -> R {
    set_armed_by_panel(true);
    with_smoke(|s| {
        s.area = Some(AREA);
        s.gizmo = Some(field3d_gizmo::Anchor::global(7, [0.0, 0.0, 0.0]));
        s.pending_move = None;
        s.drag = None;
        s.gizmo_hot = None;
        f(s)
    })
    .expect("o módulo está armado")
}

fn translation_of(m: field3d_gizmo::Motion) -> [f32; 3] {
    match m {
        field3d_gizmo::Motion::Translate(d) => d,
        other => panic!("o modo de mover pede translação, e veio {other:?}"),
    }
}

fn screen_of(s: &crate::field3d_smoke::Smoke) -> Screen {
    Screen::new(AREA.w as u32, AREA.h as u32, s.cam.half_extent)
}

/// O ponto de janela, em pixels, do meio da haste do eixo `n`.
fn mid_of_axis(s: &crate::field3d_smoke::Smoke, n: usize) -> (f32, f32) {
    let anchor = s.gizmo.expect("ancorado");
    let handles = field3d_gizmo::project(anchor, &s.cam, screen_of(s), s.gizmo_mode);
    let h = handles
        .iter()
        .find(|h| h.handle == Handle::Axis(n))
        .expect("o eixo existe");
    let field3d_gizmo::Shape::Arrow { from, to } = h.shape else {
        panic!("um eixo é uma seta");
    };
    (
        AREA.x + (from[0] + to[0]) * 0.5,
        AREA.y + (from[1] + to[1]) * 0.5,
    )
}

/// ⭐ **Carregar numa seta agarra a seta — e não orbita a câmera.**
#[test]
fn pressing_on_an_arrow_grabs_it_instead_of_orbiting() {
    armed(|s| {
        let p = mid_of_axis(s, 0);
        let before = s.cam;
        assert!(begin(s, winit::event::MouseButton::Left, Drag::Orbit, p));
        assert_eq!(
            s.drag,
            Some(Drag::Gizmo(Handle::Axis(0))),
            "o clique sobre a seta virou gesto de câmera — a alça está pintada e morta"
        );
        assert_eq!(hot_handle(s), Some(Handle::Axis(0)), "e ela acende");

        // E arrastar move a PEÇA, não a vista.
        assert!(advance(s, p.0 + 60.0, p.1));
        assert_eq!(s.cam, before, "a câmera não pode ter-se mexido");
        let (entity, motion) = s.pending_move.expect("o arrasto tem de pedir um movimento");
        assert_eq!(entity, 7, "e tem de pedi-lo para a entidade da âncora");
        assert!(
            !motion.is_idle(),
            "o pedido saiu vazio: {motion:?} — o ponteiro não chegou à lei do arrasto"
        );
    });
}

/// **Longe do gizmo, o botão esquerdo continua a orbitar.** Sem isto o gizmo sequestraria a
/// navegação da janela inteira.
#[test]
fn pressing_away_from_the_gizmo_still_orbits() {
    armed(|s| {
        let far = (AREA.x + AREA.w - 5.0, AREA.y + 5.0);
        assert!(begin(s, winit::event::MouseButton::Left, Drag::Orbit, far));
        assert_eq!(s.drag, Some(Drag::Orbit));
        assert!(s.pending_move.is_none());
    });
}

/// ⚠️ **O botão DIREITO orbita mesmo por cima da alça** — é a saída de quem quer girar a vista
/// sem primeiro tirar o rato de cima da peça.
#[test]
fn the_right_button_orbits_even_over_a_handle() {
    armed(|s| {
        let p = mid_of_axis(s, 0);
        assert!(begin(s, winit::event::MouseButton::Right, Drag::Orbit, p));
        assert_eq!(s.drag, Some(Drag::Orbit));
    });
}

/// ⭐ **Os pedidos ACUMULAM entre quadros.**
///
/// ⚠️ Entre dois quadros chegam vários eventos de ponteiro. Guardar só o último faria a peça
/// andar menos do que a mão — devagar, e **só quando o rato vai depressa**, que é o defeito mais
/// difícil de acreditar quando alguém o reporta.
#[test]
fn pointer_events_between_two_frames_add_up() {
    armed(|s| {
        let p = mid_of_axis(s, 0);
        begin(s, winit::event::MouseButton::Left, Drag::Orbit, p);
        advance(s, p.0 + 30.0, p.1);
        let one = translation_of(s.pending_move.expect("primeiro evento").1);
        advance(s, p.0 + 60.0, p.1);
        let two = translation_of(s.pending_move.expect("segundo evento").1);
        assert!(
            (two[0] - one[0] * 2.0).abs() < one[0].abs() * 1e-3,
            "dois passos iguais têm de somar: {one:?} depois {two:?}"
        );
    });
}

/// **Sem arrasto, mover o rato acende a alça e NÃO consome o evento.**
///
/// ⚠️ As duas metades importam: sem a primeira o artista não sabe o que vai agarrar; com a
/// segunda invertida, a janela 3D engoliria todo movimento de rato do app 2D.
#[test]
fn hover_lights_the_handle_without_swallowing_the_event() {
    armed(|s| {
        let p = mid_of_axis(s, 1);
        assert!(!advance(s, p.0, p.1), "hover não é um gesto desta janela");
        assert_eq!(hot_handle(s), Some(Handle::Axis(1)));
        assert!(!advance(s, AREA.x + 2.0, AREA.y + 2.0));
        assert_eq!(hot_handle(s), None, "e apaga-se ao sair");
    });
}

/// ⭐ **Soltar sem ter arrastado é um CLIQUE**, e um clique pede uma seleção.
///
/// ⚠️ É a metade de entrada da seleção por clique. A outra (de quem é o ponto) precisa do MUNDO
/// e vive na ponte; sem esta, ela nunca é chamada — e o gizmo continuaria a só chegar pela
/// Hierarquia.
#[test]
fn a_press_and_release_without_dragging_asks_for_a_pick() {
    armed(|s| {
        let far = (AREA.x + AREA.w - 60.0, AREA.y + 40.0);
        begin(s, winit::event::MouseButton::Left, Drag::Orbit, far);
        s.last_pointer = far;
        crate::field3d_input::finish_for_test(s);
        let px = s.pending_pick.expect("um clique tem de pedir uma seleção");
        assert!(
            (px[0] - (far.0 - AREA.x)).abs() < 0.01 && (px[1] - (far.1 - AREA.y)).abs() < 0.01,
            "o pixel pedido tem de ser o do CLIQUE, no referencial da área: {px:?}"
        );
    });
}

/// ⚠️ **Um arrasto NÃO é um clique** — girar a vista não pode trocar a seleção.
#[test]
fn dragging_the_camera_is_not_a_click() {
    armed(|s| {
        let far = (AREA.x + AREA.w - 60.0, AREA.y + 40.0);
        begin(s, winit::event::MouseButton::Left, Drag::Orbit, far);
        advance(s, far.0 + 40.0, far.1 + 30.0);
        crate::field3d_input::finish_for_test(s);
        assert!(
            s.pending_pick.is_none(),
            "orbitar a vista pediu uma seleção — o limiar de clique não está a morder"
        );
    });
}

/// ⚠️ **Soltar uma ALÇA do gizmo nunca é uma seleção.**
///
/// Sem isto, mover um objeto trocaria a seleção para o que estivesse por baixo dele no fim do
/// gesto — e o artista perderia o objeto que acabou de posicionar.
#[test]
fn releasing_a_gizmo_handle_is_never_a_selection() {
    armed(|s| {
        let p = mid_of_axis(s, 0);
        begin(s, winit::event::MouseButton::Left, Drag::Orbit, p);
        assert!(matches!(s.drag, Some(Drag::Gizmo(_))));
        crate::field3d_input::finish_for_test(s);
        assert!(s.pending_pick.is_none());
    });
}

/// ⚠️ **Durante o arrasto o realce fica na alça AGARRADA**, mesmo com o cursor longe dela — que
/// é onde o cursor está, porque arrastar é isso.
#[test]
fn the_grabbed_handle_stays_lit_while_the_cursor_walks_away() {
    armed(|s| {
        let p = mid_of_axis(s, 2);
        begin(s, winit::event::MouseButton::Left, Drag::Orbit, p);
        advance(s, AREA.x + AREA.w - 3.0, AREA.y + AREA.h - 3.0);
        assert_eq!(hot_handle(s), Some(Handle::Axis(2)));
    });
}
