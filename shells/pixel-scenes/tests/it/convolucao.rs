//! **P8 Edit ▸ FX ▸ Convolution Matrix… (`F9`) pelo caminho real** — irmão de `filtros.rs`: o
//! pedido do menu/tecla, a lista de kernels que a família publica, o Preview (`shown_rgba`) sem
//! gravar, o OK = UM passo que um Ctrl+Z desfaz, o Cancel intacto e o Tiled que muda a borda.

use ph2d_app_pixel::Tool;
use ph2d_app_pixel::compose::Mods;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{ContextMenuKind, WidgetEvent};
use ph2d_editor_core::pixel_panel::{
    PixelDialogChoice, PixelDialogKind, PixelEditAction as E, PixelRequest, PixelSelectAction,
};
use ph2d_pixel_core::filter::convolution::KERNELS;
use winit::keyboard::KeyCode;

use crate::selecao::Palco;

const RED: [u8; 4] = [255, 0, 0, 255];

/// Um quadrado vermelho opaco `(100..=104, 40..=44)`, sem selecção no fim.
fn com_quadrado() -> Palco {
    let mut p = Palco::limpo();
    p.app.fg = RED;
    p.ferramenta(Tool::RectMarquee);
    p.arrasta((100, 40), (104, 44), Mods::default());
    p.tecla(KeyCode::KeyF, false, false);
    p.pede(PixelRequest::Select(PixelSelectAction::Deselect));
    assert_eq!(p.rgba(102, 42), RED);
    p
}

fn aberto(p: &Palco) -> Option<PixelDialogKind> {
    let open = p
        .hero
        .store
        .context_menu()
        .is_some_and(|r| r.kind == ContextMenuKind::PixelEditDialog);
    open.then(|| p.hero.store.pixel_edit_dialog().kind)
}

fn carrega(p: &mut Palco, id: ph2d_editor_core::NodeId) {
    p.hero.apply_event(WidgetEvent::Click(id));
    p.quadro();
}

fn mostrado(p: &mut Palco, c: (i32, i32)) -> [u8; 4] {
    let key = p.key;
    p.app.shown_rgba(key, c.0, c.1).expect("doc")
}

fn kernel(nome: &str) -> ph2d_editor_core::NodeId {
    let i = KERNELS.iter().position(|k| k.name == nome).expect(nome);
    ids::pixel_filter_kernel_id(i).expect("na escada")
}

#[test]
fn convolution_opens_by_its_menu_with_the_core_list_and_brightness_first() {
    assert!(
        KERNELS.len() <= ids::PIXEL_FILTER_KERNEL_SLOTS,
        "a escada de ids cobre a lista do núcleo"
    );
    let mut p = com_quadrado();
    p.app
        .pede(ph2d_editor_core::pixel_panel::PixelRequest::Edit(
            ph2d_editor_core::pixel_panel::PixelEditAction::ConvolutionMatrix,
        ));
    let _ = p.quadro();
    assert_eq!(aberto(&p), Some(PixelDialogKind::ConvolutionMatrix), "F9");
    let f = p.hero.store.pixel_edit_dialog().filter;
    let nomes: Vec<&str> = KERNELS.iter().map(|k| k.name).collect();
    assert_eq!(
        f.kernels,
        nomes.as_slice(),
        "a lista é a do núcleo, na ordem"
    );
    assert_eq!(f.kernel, 0, "«brightness» escolhido ao abrir");
    assert!(!f.tiled, "Tiled desligado");
    // Os kernels são um DROPDOWN (dono 02/11), fechado ao abrir e com o escolhido.
    assert!(
        matches!(
            p.hero.store.get(ids::PIXEL_FILTER_KERNEL),
            Some(ph2d_editor_core::interaction::InteractiveState::Dropdown {
                open: false,
                selected_index: Some(0),
                ..
            })
        ),
        "o dropdown dos kernels registado"
    );
    // brightness = +8 por canal; o alfa fica.
    assert_eq!(mostrado(&mut p, (102, 42)), [255, 8, 8, 255]);
    carrega(&mut p, ids::PIXEL_EDIT_CANCEL);
    p.pede(PixelRequest::Edit(E::ConvolutionMatrix));
    assert_eq!(
        aberto(&p),
        Some(PixelDialogKind::ConvolutionMatrix),
        "o menu"
    );
}

#[test]
fn choosing_a_kernel_previews_without_writing_and_ok_is_one_step_that_undo_restores() {
    let mut p = com_quadrado();
    let antes = p.sprite().clone();
    let n = p.passos();
    p.app
        .pede(ph2d_editor_core::pixel_panel::PixelRequest::Edit(
            ph2d_editor_core::pixel_panel::PixelEditAction::ConvolutionMatrix,
        ));
    let _ = p.quadro();
    carrega(&mut p, kernel("negative"));
    assert_eq!(mostrado(&mut p, (102, 42)), [0, 255, 255, 255], "negative");
    assert_eq!(*p.sprite(), antes, "o Preview não grava");
    carrega(&mut p, ids::PIXEL_EDIT_OK);
    assert_eq!(aberto(&p), None);
    assert_eq!(
        p.rgba(102, 42),
        [0, 255, 255, 255],
        "o OK grava o que se via"
    );
    assert_eq!(p.passos(), n + 1, "UM passo");
    p.tecla(KeyCode::KeyZ, true, false);
    assert_eq!(*p.sprite(), antes, "um Ctrl+Z desfaz o filtro inteiro");
    // A sessão lembra o kernel.
    p.app
        .pede(ph2d_editor_core::pixel_panel::PixelRequest::Edit(
            ph2d_editor_core::pixel_panel::PixelEditAction::ConvolutionMatrix,
        ));
    let _ = p.quadro();
    let i = KERNELS.iter().position(|k| k.name == "negative");
    assert_eq!(
        Some(usize::from(p.hero.store.pixel_edit_dialog().filter.kernel)),
        i
    );
}

#[test]
fn convolution_cancel_leaves_the_document_byte_for_byte() {
    let mut p = com_quadrado();
    let antes = p.sprite().clone();
    let n = p.passos();
    p.app
        .pede(ph2d_editor_core::pixel_panel::PixelRequest::Edit(
            ph2d_editor_core::pixel_panel::PixelEditAction::ConvolutionMatrix,
        ));
    let _ = p.quadro();
    carrega(&mut p, kernel("blur-3x3"));
    assert_ne!(
        mostrado(&mut p, (99, 42)),
        antes.rgba_of_value(0),
        "pré-visualizado"
    );
    carrega(&mut p, ids::PIXEL_EDIT_CANCEL);
    assert_eq!(aberto(&p), None);
    assert_eq!(*p.sprite(), antes, "o documento ao byte");
    assert_eq!(p.passos(), n, "nenhum passo");
    assert_eq!(mostrado(&mut p, (102, 42)), RED, "a pré-visualização sai");
}

#[test]
fn convolution_tiled_wraps_the_border() {
    let mut p = Palco::limpo();
    let w = i32::try_from(p.sprite().width()).expect("lado");
    p.app.fg = RED;
    // Uma faixa limpa de borda a borda, e um ponto vermelho encostado à esquerda.
    p.ferramenta(Tool::RectMarquee);
    p.arrasta((0, 5), (w - 1, 15), Mods::default());
    p.tecla(KeyCode::Delete, false, false);
    p.pede(PixelRequest::Select(PixelSelectAction::Deselect));
    p.ferramenta(Tool::Pencil);
    p.press((0, 10), Mods::default());
    p.release();
    assert_eq!(p.rgba(0, 10), RED);
    p.app
        .pede(ph2d_editor_core::pixel_panel::PixelRequest::Edit(
            ph2d_editor_core::pixel_panel::PixelEditAction::ConvolutionMatrix,
        ));
    let _ = p.quadro();
    carrega(&mut p, kernel("blur-3x3"));
    let preso = mostrado(&mut p, (w - 1, 10));
    carrega(&mut p, ids::PIXEL_FILTER_TILED);
    let volta = mostrado(&mut p, (w - 1, 10));
    assert_eq!(preso[0], 0, "sem Tiled a borda direita não vê a esquerda");
    assert!(volta[0] > 0, "com Tiled a borda dá a volta: {volta:?}");
    p.hero.store.request_pixel_edit();
    let pedido = p.hero.store.take_pixel_edit_request();
    assert!(
        matches!(
            pedido,
            Some(PixelDialogChoice::Convolution { tiled: true, .. })
        ),
        "{pedido:?}"
    );
}

/// Escolher um kernel põe os botões R G B A nos dele (Trial, `A5_kernel_channels`): blur-3x3 liga o
/// A, negative desliga-o; e com o A desligado o negative não mexe no alfa.
#[test]
fn choosing_blur_turns_alpha_on_and_negative_turns_it_off() {
    let mut p = com_quadrado();
    p.app
        .pede(ph2d_editor_core::pixel_panel::PixelRequest::Edit(
            ph2d_editor_core::pixel_panel::PixelEditAction::ConvolutionMatrix,
        ));
    let _ = p.quadro();
    carrega(&mut p, kernel("blur-3x3"));
    assert_eq!(
        p.hero.store.pixel_edit_dialog().filter.channels,
        [true; 4],
        "blur-3x3 = R G B A"
    );
    carrega(&mut p, kernel("negative"));
    assert_eq!(
        p.hero.store.pixel_edit_dialog().filter.channels,
        [true, true, true, false],
        "negative = R G B, A solto"
    );
    assert_eq!(mostrado(&mut p, (102, 42))[3], 255, "o alfa fica");
    carrega(&mut p, ids::PIXEL_FILTER_CHANNELS[3].1);
    assert!(
        p.hero.store.pixel_edit_dialog().filter.channels[3],
        "o A clicado depois do kernel"
    );
    assert_eq!(
        mostrado(&mut p, (102, 42))[3],
        0,
        "negative com A: o alfa inverte"
    );
}

/// A 1.ª abertura já põe os botões R G B A nos do kernel escolhido (brightness = R G B, A solto,
/// como o Trial), e reabrir volta aos do kernel lembrado mesmo que o A tenha sido clicado.
#[test]
fn opening_the_convolution_puts_the_channels_of_the_chosen_kernel() {
    let mut p = com_quadrado();
    p.app
        .pede(ph2d_editor_core::pixel_panel::PixelRequest::Edit(
            ph2d_editor_core::pixel_panel::PixelEditAction::ConvolutionMatrix,
        ));
    let _ = p.quadro();
    let f = p.hero.store.pixel_edit_dialog().filter;
    assert_eq!(f.channels, KERNELS[0].channels, "os do kernel ao abrir");
    assert_eq!(f.channels, [true, true, true, false], "brightness = R G B");
    carrega(&mut p, kernel("blur-3x3"));
    carrega(&mut p, ids::PIXEL_FILTER_CHANNELS[3].1);
    carrega(&mut p, ids::PIXEL_EDIT_CANCEL);
    p.app
        .pede(ph2d_editor_core::pixel_panel::PixelRequest::Edit(
            ph2d_editor_core::pixel_panel::PixelEditAction::ConvolutionMatrix,
        ));
    let _ = p.quadro();
    assert_eq!(
        p.hero.store.pixel_edit_dialog().filter.channels,
        [true; 4],
        "reaberto: os do blur-3x3 lembrado"
    );
}

/// O Convolution usa os canais do kernel À PARTE: fechá-lo e abrir o Invert devolve os deste (o A
/// ligado de fábrica), e reabrir o Convolution volta aos do kernel.
#[test]
fn the_convolution_channels_do_not_leak_into_the_next_dialog() {
    let mut p = com_quadrado();
    p.app
        .pede(ph2d_editor_core::pixel_panel::PixelRequest::Edit(
            ph2d_editor_core::pixel_panel::PixelEditAction::ConvolutionMatrix,
        ));
    let _ = p.quadro();
    assert!(
        !p.hero.store.pixel_edit_dialog().filter.channels[3],
        "A solto"
    );
    carrega(&mut p, ids::PIXEL_EDIT_CANCEL);
    p.pede(PixelRequest::Edit(E::Invert));
    assert_eq!(aberto(&p), Some(PixelDialogKind::Invert));
    assert_eq!(
        p.hero.store.pixel_edit_dialog().filter.channels,
        [true; 4],
        "o Invert com os seus"
    );
    carrega(&mut p, ids::PIXEL_EDIT_CANCEL);
    p.app
        .pede(ph2d_editor_core::pixel_panel::PixelRequest::Edit(
            ph2d_editor_core::pixel_panel::PixelEditAction::ConvolutionMatrix,
        ));
    let _ = p.quadro();
    assert!(!p.hero.store.pixel_edit_dialog().filter.channels[3]);
}
