//! **P8 Edit ▸ FX ▸ Convolution Matrix…** do rato e do teclado ao pedido — irmão de
//! `the_pixel_edit_dialogs_are_alive` (o mesmo [`Ecra`]): a lista que a família publica pinta-se
//! inteira e viva dentro da janela, o clique escolhe o kernel e refaz o pedido do Preview, o Tiled
//! entra no pedido, o OK arma-o e a sessão lembra o kernel; o Esc não arma nada; nenhum nome se corta.

use crate::the_pixel_edit_dialogs_are_alive::Ecra;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::KEY_ESCAPE;
use ph2d_editor_core::pixel_panel::{PixelDialogChoice, PixelDialogKind};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::text_elide::elisao;
use ph2d_text::TextSystem;

/// 25 nomes como os do núcleo (o chrome não o vê): os mais longos da lista real à frente.
fn nomes() -> &'static [&'static str] {
    static N: std::sync::LazyLock<Vec<&'static str>> = std::sync::LazyLock::new(|| {
        let mut v = vec![
            "blur-5x5-diagonal(\\)",
            "blur-5x5-diagonal(/)",
            "misc-rock-edges",
        ];
        v.extend((v.len()..25).map(|i| &*Box::leak(format!("kernel-{i}x{i}").into_boxed_str())));
        v
    });
    &N
}

fn aberto() -> Ecra {
    let mut e = Ecra {
        hero: HeroScreen::new(ph2d_a11y::NodeId(1)),
        text: TextSystem::without_system_fonts(),
    };
    e.hero.store.set_pixel_filter_kernels(nomes());
    e.hero
        .store
        .open_pixel_edit_dialog(PixelDialogKind::ConvolutionMatrix, (32, 24));
    e.pinta();
    e
}

#[test]
fn every_kernel_row_and_control_is_painted_alive_and_inside_the_window() {
    let e = aberto();
    assert!(e.aberto_ainda());
    let mut todos: Vec<_> = (0..nomes().len())
        .map(|i| ids::pixel_filter_kernel_id(i).expect("na escada"))
        .collect();
    todos.extend(ids::PIXEL_FILTER_CHANNELS.iter().map(|(_, id, _)| *id));
    todos.extend([
        ids::PIXEL_FILTER_PREVIEW,
        ids::PIXEL_FILTER_TILED,
        ids::PIXEL_EDIT_OK,
        ids::PIXEL_EDIT_CANCEL,
    ]);
    let vp = e.hero.last_viewport;
    for id in todos {
        let (_, y) = e.centro(id).unwrap_or_else(|| panic!("{id:?} sem clique"));
        assert!(y < vp.y + vp.h, "{id:?} fora da janela");
        assert!(e.hero.store.get(id).is_some(), "{id:?} sem estado: MORTO");
        assert!(ids::pixel_edit_dialog_owns(id), "{id:?} fora do dono");
    }
    let f = e.hero.store.pixel_edit_dialog().filter;
    assert_eq!(f.kernel, 0, "o 1.º escolhido ao abrir");
    assert!(f.preview && !f.tiled);
}

#[test]
fn clicking_a_kernel_and_tiled_reach_the_preview_the_ok_and_the_session() {
    let mut e = aberto();
    assert_eq!(
        e.hero.store.pixel_filter_preview(),
        Some(PixelDialogChoice::Convolution {
            kernel: 0,
            tiled: false,
            channels: [true; 4],
        })
    );
    e.carrega(ids::pixel_filter_kernel_id(24).expect("linha"));
    e.carrega(ids::PIXEL_FILTER_TILED);
    e.carrega(ids::PIXEL_FILTER_CHANNELS[3].1);
    let quer = PixelDialogChoice::Convolution {
        kernel: 24,
        tiled: true,
        channels: [true, true, true, false],
    };
    assert_eq!(e.hero.store.pixel_filter_preview(), Some(quer));
    e.carrega(ids::PIXEL_EDIT_OK);
    assert!(!e.aberto_ainda());
    assert_eq!(e.pedido(), Some(quer));
    e.hero
        .store
        .open_pixel_edit_dialog(PixelDialogKind::ConvolutionMatrix, (32, 24));
    assert_eq!(
        e.hero.store.pixel_edit_dialog().filter.kernel,
        24,
        "lembrado na sessão"
    );
}

#[test]
fn convolution_esc_closes_without_a_request() {
    let mut e = aberto();
    e.carrega(ids::pixel_filter_kernel_id(3).expect("linha"));
    e.tecla(KEY_ESCAPE, false);
    assert!(!e.aberto_ainda());
    assert_eq!(e.pedido(), None);
}

#[test]
fn no_kernel_name_nor_label_of_the_convolution_dialog_is_cut() {
    let mut e = aberto();
    let mut esperados: Vec<&str> = nomes().to_vec();
    esperados.extend(
        [
            "pixel.filter_dialog.convolution.title",
            "pixel.filter_dialog.tiled",
            "pixel.filter_dialog.preview",
        ]
        .map(ph2d_i18n::tr),
    );
    let ((), medidos) = elisao::medindo(|| e.pinta());
    for t in esperados {
        let md = medidos
            .iter()
            .find(|md| md.texto == t)
            .unwrap_or_else(|| panic!("«{t}» nunca foi medido"));
        assert!(md.coube(), "«{t}» cortado em «{}»", md.pintado);
    }
}
