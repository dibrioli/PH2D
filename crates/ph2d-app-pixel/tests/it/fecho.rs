//! **O fecho da linha** (auditoria 28/10): o premir que não foi da família não é largado por ela,
//! abrir um projecto larga as células levantadas do anterior, a moldura gigante só aloca a tela e o
//! eixo da simetria sob o cursor dá a dica dele.

use ph2d_app_pixel::compose::{Mods, Pointer, View};
use ph2d_app_pixel::{Tool, persist};
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::pixel_panel::{PixelRequest, PixelTransformField as F};
use ph2d_host::{PointerButton, PointerKind};
use winit::keyboard::KeyCode;

use crate::edit::{PPM, WINDOW, at, spawn_root};
use crate::selecao::Palco;

fn ponteiro(p: &mut Palco, k: PointerKind, b: PointerButton, c: (i32, i32), free: bool) -> Pointer {
    let xy = at(&p.app, &p.sim, &p.cam, p.bits, c.0, c.1);
    let mut toasts = ToastQueue::new();
    let mut v = View {
        world: p.sim.world(),
        camera: &mut p.cam,
        toasts: &mut toasts,
        window: WINDOW,
        ppm: PPM,
        editing: Some(p.bits),
    };
    p.app.pointer(&mut v, k, b, xy, Mods::default(), free)
}

fn move_sem_botao(p: &mut Palco, c: (i32, i32)) -> bool {
    let xy = at(&p.app, &p.sim, &p.cam, p.bits, c.0, c.1);
    let mut toasts = ToastQueue::new();
    let v = View {
        world: p.sim.world(),
        camera: &mut p.cam,
        toasts: &mut toasts,
        window: WINDOW,
        ppm: PPM,
        editing: Some(p.bits),
    };
    p.app.pointer_moved(&v, xy, Mods::default())
}

/// Com o polígono à espera do próximo vértice, o clique no chrome, o pan do botão do meio e o mover
/// sem botão são da casa: antes o `Up` de cada um era engolido (o botão do painel não reagia e a
/// âncora do pan nunca largava).
#[test]
fn a_pending_polygon_never_takes_a_release_or_a_move_that_is_not_its_own() {
    use PointerButton::{Middle, Primary};
    use PointerKind::{Down, Up};
    let mut p = Palco::limpo();
    p.ferramenta(Tool::Polygon);
    for c in [(4, 4), (20, 6)] {
        assert_eq!(ponteiro(&mut p, Down, Primary, c, true), Pointer::Done);
        assert_eq!(ponteiro(&mut p, Up, Primary, c, true), Pointer::Done);
    }
    assert!(p.app.stroking(), "o polígono espera o 3.º vértice");
    assert_eq!(
        ponteiro(&mut p, Down, Primary, (9, 9), false),
        Pointer::NotMine
    );
    assert_eq!(
        ponteiro(&mut p, Up, Primary, (9, 9), false),
        Pointer::NotMine,
        "o largar do clique no chrome é do chrome"
    );
    assert_eq!(
        ponteiro(&mut p, Down, Middle, (9, 9), true),
        Pointer::NotMine
    );
    assert!(
        !move_sem_botao(&mut p, (12, 12)),
        "o mover sem botão não é consumido"
    );
    assert_eq!(
        ponteiro(&mut p, Up, Middle, (12, 12), true),
        Pointer::NotMine
    );
    assert!(p.app.stroking(), "e o polígono continua à espera");
    // Controlo: o vértice seguinte continua a ser dele.
    assert_eq!(
        ponteiro(&mut p, Down, Primary, (8, 20), true),
        Pointer::Done
    );
    assert_eq!(ponteiro(&mut p, Up, Primary, (8, 20), true), Pointer::Done);
}

/// Abrir um projecto com células levantadas: o quadro seguinte (fora do Draw) assentava-as no
/// documento NOVO com a mesma chave, com um passo no desfazer dele.
#[test]
fn opening_a_project_drops_the_floating_cells_of_the_old_one() {
    let mut p = Palco::limpo();
    let bytes = persist::to_bytes(&mut p.app, p.sim.world());
    p.ferramenta(Tool::RectMarquee);
    p.arrasta((2, 2), (12, 12), Mods::default());
    p.tecla(KeyCode::KeyT, true, false);
    assert!(p.app.floating(), "levantou");
    persist::install(&mut p.app, persist::read(&bytes).expect("lê"));
    p.app
        .frame(&mut p.sim, &mut p.cam, WINDOW, PPM, None, spawn_root);
    assert_eq!(p.passos(), 0, "o documento aberto nasce sem passos");
    assert!(!p.app.floating());
}

/// W e H até 65535 (o campo) não alocam a caixa inteira: o assentar só escreve na tela, e é só a
/// tela que se calcula. Antes: 65535² células × 2 imagens.
#[test]
fn a_huge_frame_only_computes_the_canvas_and_still_applies_as_one_step() {
    let mut p = Palco::limpo();
    p.ferramenta(Tool::RectMarquee);
    p.arrasta((2, 2), (12, 12), Mods::default());
    p.tecla(KeyCode::KeyT, true, false);
    let n = p.passos();
    for field in [F::W, F::H] {
        p.pede(PixelRequest::TransformField {
            field,
            value: 65_535,
        });
    }
    let t = p.quadro().transform.expect("solta");
    assert_eq!((t.w, t.h), (65_535, 65_535), "a moldura é a pedida");
    p.tecla(KeyCode::Enter, false, false);
    assert_eq!(p.passos(), n + 1, "assentar é UM passo");
}

/// A dica `pixel.hint.axis` não tinha leitor: com o cursor sobre o eixo da simetria o painel diz-o.
#[test]
fn hovering_the_symmetry_axis_tells_the_panel() {
    let mut p = Palco::limpo();
    p.ferramenta(Tool::Pencil);
    p.opcoes(|o| o.sym_h = true);
    let w = i32::try_from(p.sprite().width()).expect("w");
    move_sem_botao(&mut p, (2, 2));
    assert!(!p.quadro().on_axis, "longe do eixo (controlo)");
    let meio = p.canto(f64::from(w) / 2.0, 4.0);
    let mut toasts = ToastQueue::new();
    let v = View {
        world: p.sim.world(),
        camera: &mut p.cam,
        toasts: &mut toasts,
        window: WINDOW,
        ppm: PPM,
        editing: Some(p.bits),
    };
    p.app.pointer_moved(&v, meio, Mods::default());
    assert!(p.quadro().on_axis, "sobre o eixo");
}
