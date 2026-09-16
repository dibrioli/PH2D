//! **O TRAÇO DO PINCEL DE PLANO DESFAZ E REFAZ** — o report do dono de 2026-09-16
//! (*«smoke ok. Mas sem undo/redo.»*), medido na família pelas duas rotas.
//!
//! Irmão de [`super::tests`], cortado dele pelo tecto de LOC (HR-18) e pelo assunto: lá os canais
//! de um traço genérico, aqui o pincel que só move barro **a andar e sobre relevo**.
//!
//! ⚠️ **As duas rotas passaram verdes com o defeito vivo** — ele estava na SHELL (a ferramenta
//! vectorial em mãos matava as teclas), e o elo de shell tem gate próprio
//! (`a_sculpt_gesture_releases_the_tool_in_hand`). Estes provam a metade de cá: a cena grava,
//! desfaz e refaz o traço, pelas funções e pelas portas que a shell chama.

use ph2d_sculpt3d::Verb;

use super::super::Sculpt3dScene;

/// Abre a GPU, ou diz que não há nada a afirmar (cópia local, como nos irmãos).
macro_rules! gpu_or_skip {
    () => {
        match ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) {
            Ok(g) => g,
            Err(_) => {
                eprintln!("no GPU adapter on this machine — nothing to assert");
                return;
            }
        }
    };
}

/// O meio do viewport — onde a peça enquadrada está.
const CENTRE: (f32, f32) = (450.0, 350.0);

/// ⛔⛔ **UM TRAÇO DO PINCEL DE PLANO DESFAZ E REFAZ** — report do dono
/// (2026-09-16): *«smoke ok. Mas sem undo/redo.»*
///
/// ⚠️ **O traço ANDA, e com bossas debaixo** — é a única configuração em que o
/// pincel move barro: o primeiro dab de uma passagem é inerte por LEI (espec §1)
/// e numa bola lisa ele pára sozinho (§8). *Um gate sobre um traço de um dab, ou
/// sobre uma esfera lisa, ficava verde a medir o nada.*
#[test]
#[ignore = "precisa de GPU"]
fn a_plane_stroke_undoes_and_redoes() {
    let gpu = gpu_or_skip!();
    let mut s = Sculpt3dScene::new(&gpu.device, crate::scenes::plano::peca(), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    s.brush.verb = Verb::Plane;
    let antes: Vec<[f32; 3]> = s.mesh().positions().to_vec();

    assert!(s.aim(CENTRE.0, CENTRE.1), "o raio errou a peca enquadrada");
    s.stroke.begin(s.objects[s.active].stack.mesh());
    for k in 0..12u8 {
        let x = CENTRE.0 - 60.0 + 10.0 * f32::from(k);
        s.sculpt_at(x, CENTRE.1);
    }
    let moveu = (0..antes.len())
        .filter(|v| s.mesh().positions()[*v] != antes[*v])
        .count();
    assert!(
        moveu > 0,
        "premissa: o traco do pincel de plano nao moveu nada -- sem isso o \
         desfazer nao tem o que desfazer"
    );

    s.close_stroke();
    s.objects[s.active].uploaded = true;
    assert!(
        s.undo_stroke(),
        "⛔ o traco do PINCEL DE PLANO nao gravou passo de undo -- e' o report de 16/09"
    );
    let sobra = (0..antes.len())
        .filter(|v| s.mesh().positions()[*v] != antes[*v])
        .count();
    assert_eq!(
        sobra, 0,
        "o desfazer deixou {sobra} de {moveu} vertices deslocados"
    );
    assert!(
        !s.objects[s.active].uploaded,
        "⛔ o desfazer nao avisou a TELA"
    );
    assert!(s.redo_stroke(), "⛔ o pincel de plano nao refaz");
    let voltou = (0..antes.len())
        .filter(|v| s.mesh().positions()[*v] != antes[*v])
        .count();
    assert_eq!(voltou, moveu, "o refazer devolveu {voltou} dos {moveu}");
}

/// Uma shell de teste com o mínimo que as portas de ponteiro perguntam.
struct HostDeTeste {
    ponteiro: (f32, f32),
    ctrl: bool,
}

impl ph2d_app_host::AppHost for HostDeTeste {
    fn pointer(&self) -> (f32, f32) {
        self.ponteiro
    }
    fn mods(&self) -> ph2d_app_host::HostMods {
        ph2d_app_host::HostMods {
            shift: false,
            control: self.ctrl,
            alt: false,
            super_key: false,
        }
    }
    fn pointer_over_chrome(&self, _x: f32, _y: f32) -> bool {
        false
    }
    fn modal_takes_the_pointer(&self) -> bool {
        false
    }
    fn note_authored_change(&mut self) {}
}

/// ⭐⭐ **O MESMO report, pelo CAMINHO INTEIRO que a shell percorre** — o
/// pen-down, os movimentos, o pen-up e as DUAS teclas, cada uma pela porta que a
/// shell chama.
///
/// ⚠️ **O gate irmão acima chama as funções de dentro** (`aim` → `sculpt_at` →
/// `close_stroke`), e um defeito que viva no despacho do gesto ou do teclado passa
/// por cima dele. *Um recurso de arrastar prova-se pelo gesto, não pela função que
/// o gesto chama.*
#[test]
#[ignore = "precisa de GPU"]
fn a_plane_stroke_undoes_through_the_pointer_and_the_keyboard() {
    use winit::keyboard::KeyCode as K;
    let gpu = gpu_or_skip!();
    let mut s = Sculpt3dScene::new(&gpu.device, crate::scenes::plano::peca(), 1.0);
    s.note_canvas(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 900.0, 700.0));
    s.brush.verb = Verb::Plane;
    let antes: Vec<[f32; 3]> = s.mesh().positions().to_vec();

    for inverter in [false, true] {
        let mut host = HostDeTeste {
            ponteiro: (CENTRE.0 - 60.0, CENTRE.1),
            ctrl: inverter,
        };
        assert!(
            crate::input_down::pointer_down(&mut host, &mut s, winit::event::MouseButton::Left),
            "o pen-down nao foi da cena"
        );
        for k in 1..=12u8 {
            crate::input::pointer_move(&mut s, CENTRE.0 - 60.0 + 10.0 * f32::from(k), CENTRE.1);
        }
        crate::input::pointer_up(&mut s);
    }
    let moveu = (0..antes.len())
        .filter(|v| s.mesh().positions()[*v] != antes[*v])
        .count();
    assert!(moveu > 0, "premissa: os dois tracos nao moveram nada");

    let mut req = crate::Sculpt3dRequests::default();
    let factos = crate::keys::keys_delete::DeleteFacts {
        clay_on_screen: true,
        text_focused: false,
        over_panel: false,
        vector_has_selection: false,
    };
    let tecla = |s: &mut Sculpt3dScene, req: &mut crate::Sculpt3dRequests, shift: bool| {
        crate::keys::key(
            s,
            req,
            None,
            crate::keys::KeyPress {
                code: K::KeyZ,
                ctrl: true,
                shift,
            },
            &factos,
            "",
        )
    };
    assert!(
        tecla(&mut s, &mut req, false),
        "⛔ Ctrl+Z nao foi consumido pela cena"
    );
    assert!(
        tecla(&mut s, &mut req, false),
        "⛔ o 2.o Ctrl+Z nao foi consumido"
    );
    let sobra = (0..antes.len())
        .filter(|v| s.mesh().positions()[*v] != antes[*v])
        .count();
    assert_eq!(
        sobra, 0,
        "dois Ctrl+Z deixaram {sobra} de {moveu} vertices fora"
    );
    assert!(
        tecla(&mut s, &mut req, true),
        "⛔ Ctrl+Shift+Z nao foi consumido"
    );
    assert!(
        tecla(&mut s, &mut req, true),
        "⛔ o 2.o Ctrl+Shift+Z nao foi consumido"
    );
    let voltou = (0..antes.len())
        .filter(|v| s.mesh().positions()[*v] != antes[*v])
        .count();
    assert_eq!(voltou, moveu, "o refazer devolveu {voltou} dos {moveu}");
}
