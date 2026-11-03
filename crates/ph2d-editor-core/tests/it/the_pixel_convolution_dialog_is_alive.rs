//! **P8 Edit ▸ FX ▸ Convolution Matrix…** do rato e do teclado ao pedido — irmão de
//! `the_pixel_edit_dialogs_are_alive` (o mesmo [`Ecra`]): o diálogo tem a largura dos irmãos e os
//! kernels que a família publica vão num DROPDOWN (dono 02/11: as três colunas em largura dupla
//! tapavam metade do sprite); a lista abre, rola, cada opção é viva, escolher muda o kernel e os
//! canais e refaz o pedido do Preview, o Tiled entra no pedido, o OK arma-o e a sessão lembra o
//! kernel; o Esc com a lista aberta fecha só a lista; nenhum rótulo se corta.

use crate::the_pixel_edit_dialogs_are_alive::Ecra;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::KEY_ESCAPE;
use ph2d_editor_core::pixel_panel::{PixelDialogChoice, PixelDialogKind};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::text_elide::elisao;
use ph2d_text::TextSystem;

/// 39 nomes, tantos quantos a lista do Trial (o núcleo tem 37; o chrome não o vê): os mais longos
/// da lista real à frente.
fn nomes() -> &'static [&'static str] {
    static N: std::sync::LazyLock<Vec<&'static str>> = std::sync::LazyLock::new(|| {
        let mut v = vec![
            "edges-find-horizontal",
            "edges-find-vertical",
            "blur-5x5-diagonal(\\)",
            "blur-5x5-diagonal(/)",
            "misc-rock-edges",
        ];
        v.extend((v.len()..39).map(|i| &*Box::leak(format!("kernel-{i}x{i}").into_boxed_str())));
        v
    });
    &N
}

/// Os canais de cada kernel: o 5 sem o A (como os `misc-*` do Trial), os outros RGBA.
fn canais() -> &'static [[bool; 4]] {
    static C: std::sync::LazyLock<Vec<[bool; 4]>> = std::sync::LazyLock::new(|| {
        (0..39)
            .map(|i| {
                if i == 5 {
                    [true, true, true, false]
                } else {
                    [true; 4]
                }
            })
            .collect()
    });
    &C
}

/// O centro do rect que a pintura publicou para `id` (já recortado pela lista), se o clique
/// nesse ponto chega MESMO a `id` pelo `hit` que o dispatch usa — sem varrer a janela.
fn centro_publicado(e: &Ecra, id: ph2d_a11y::NodeId) -> Option<(f32, f32)> {
    let r = e.hero.hit_index.rect_for(id)?;
    let (x, y) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    (e.hero.hit_index.hit(x, y) == Some(id)).then_some((x, y))
}

/// Rola a lista aberta, a partir da rolagem corrente, até a opção `i` ter clique (o que a roda faz).
fn rola_ate(e: &mut Ecra, i: usize) -> ph2d_a11y::NodeId {
    let id = ids::pixel_filter_kernel_id(i).expect("na escada");
    let mut s = e.hero.store.panel_scroll(ids::PIXEL_FILTER_KERNEL);
    while centro_publicado(e, id).is_none() {
        s += 8.0;
        assert!(s < 4000.0, "a opção {i} nunca ficou à vista");
        e.hero.store.set_panel_scroll(ids::PIXEL_FILTER_KERNEL, s);
        e.pinta();
    }
    id
}

fn aberto() -> Ecra {
    let mut e = Ecra {
        hero: HeroScreen::new(ph2d_a11y::NodeId(1)),
        text: TextSystem::without_system_fonts(),
        janela: None,
    };
    e.hero.store.set_pixel_filter_kernels(nomes());
    e.hero.store.set_pixel_filter_kernel_channels(canais());
    e.hero
        .store
        .open_pixel_edit_dialog(PixelDialogKind::ConvolutionMatrix, (32, 24));
    e.pinta();
    e
}

#[test]
fn the_dialog_has_the_siblings_width_and_every_control_is_alive() {
    let e = aberto();
    assert!(e.aberto_ainda());
    let mut todos: Vec<_> = ids::PIXEL_FILTER_CHANNELS
        .iter()
        .map(|(_, id, _)| *id)
        .collect();
    todos.extend([
        ids::PIXEL_FILTER_KERNEL,
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
    // Fechada, nenhuma opção da lista apanha clique.
    assert!(
        e.centro(ids::pixel_filter_kernel_id(0).expect("linha"))
            .is_none()
    );
    // A largura: Cancel | OK ocupam a caixa — a mesma dos irmãos (o Hue/Saturation).
    let caixa = |e: &Ecra| {
        let c = e
            .hero
            .hit_index
            .rect_for(ids::PIXEL_EDIT_CANCEL)
            .expect("Cancel");
        let o = e.hero.hit_index.rect_for(ids::PIXEL_EDIT_OK).expect("OK");
        o.x + o.w - c.x
    };
    let conv = caixa(&e);
    let mut hue = Ecra {
        hero: HeroScreen::new(ph2d_a11y::NodeId(1)),
        text: TextSystem::without_system_fonts(),
        janela: None,
    };
    hue.hero
        .store
        .open_pixel_edit_dialog(PixelDialogKind::HueSaturation, (32, 24));
    hue.pinta();
    assert!((conv - caixa(&hue)).abs() < 0.5, "{conv} ≠ {}", caixa(&hue));
    let f = e.hero.store.pixel_edit_dialog().filter;
    assert_eq!(f.kernel, 0, "o 1.º escolhido ao abrir");
    assert!(f.preview && !f.tiled);
}

#[test]
fn opening_the_kernel_dropdown_lists_every_kernel_alive_and_scrolls() {
    let mut e = aberto();
    e.carrega(ids::PIXEL_FILTER_KERNEL);
    assert!(
        e.hero.store.pixel_filter_kernel_list_open(),
        "o clique no chip abre a lista"
    );
    assert!(e.aberto_ainda(), "abrir a lista não fecha o diálogo");
    let (dono, painel) = e.hero.store.dropdown_popover().expect("a lista publicada");
    assert_eq!(dono, ids::PIXEL_FILTER_KERNEL);
    let vp = e.hero.last_viewport;
    assert!(
        painel.y >= vp.y && painel.y + painel.h <= vp.y + vp.h + 0.5,
        "{painel:?}"
    );
    for i in 0..nomes().len() {
        let id = rola_ate(&mut e, i);
        assert!(
            e.hero.store.get(id).is_some(),
            "opção {i} sem estado: MORTA"
        );
        assert!(ids::pixel_edit_dialog_owns(id), "opção {i} fora do dono");
        let (x, y) = centro_publicado(&e, id).expect("à vista");
        assert!(painel.contains(x, y), "opção {i} fora da lista");
    }
}

/// Smoke do dono 02/11 (cena 19, 1930×984): a lista descia por cima da Timeline até à borda da
/// janela e a última linha saía cortada. Na janela da cena e numa janela BAIXA a lista fica dentro
/// dela com folga, mostra só linhas inteiras, e cada kernel alcança-se pela rolagem.
#[test]
fn the_kernel_list_fits_the_window_whole_rows_tall_and_short() {
    use ph2d_editor_core::zones::Rect;
    for janela in [
        Rect::new(0.0, 0.0, 1930.0, 984.0),
        Rect::new(0.0, 0.0, 1930.0, 560.0),
        Rect::new(0.0, 0.0, 1280.0, 420.0),
    ] {
        let mut e = aberto();
        e.janela = Some(janela);
        e.pinta();
        e.carrega(ids::PIXEL_FILTER_KERNEL);
        assert!(e.hero.store.pixel_filter_kernel_list_open(), "{janela:?}");
        let (_, painel) = e.hero.store.dropdown_popover().expect("a lista publicada");
        let chip = e
            .hero
            .hit_index
            .rect_for(ids::PIXEL_FILTER_KERNEL)
            .expect("chip");
        let margem = ph2d_tokens::Spacing::Md.px();
        assert!(
            painel.y >= janela.y + margem && painel.y + painel.h <= janela.y + janela.h - margem,
            "{janela:?}: a lista encosta ou sai da janela: {painel:?}"
        );
        assert!(
            painel.x >= janela.x && painel.x + painel.w <= janela.x + janela.w,
            "{janela:?}: {painel:?}"
        );
        // Só linhas inteiras: o que sobra da altura depois das linhas é o mesmo recuo da lista
        // inteira (a altura do conteúdo = N linhas + o recuo).
        let fraccao = |h: f32| h - (h / chip.h).floor() * chip.h;
        let conteudo = e
            .hero
            .store
            .panel_content_h(ids::PIXEL_FILTER_KERNEL)
            .expect("publicada");
        assert!(conteudo > painel.h, "{janela:?}: a lista tem de rolar");
        assert!(
            (fraccao(painel.h) - fraccao(conteudo)).abs() < 0.5,
            "{janela:?}: a última linha corta-se ({} px de sobra em {painel:?})",
            fraccao(painel.h)
        );
        for i in 0..nomes().len() {
            let id = rola_ate(&mut e, i);
            let (x, y) = centro_publicado(&e, id).expect("à vista");
            assert!(painel.contains(x, y), "{janela:?}: opção {i} fora da lista");
        }
    }
}

#[test]
fn choosing_a_kernel_in_the_dropdown_changes_the_kernel_and_its_channels() {
    let mut e = aberto();
    e.carrega(ids::PIXEL_FILTER_KERNEL);
    e.carrega(ids::pixel_filter_kernel_id(5).expect("linha"));
    assert!(
        !e.hero.store.pixel_filter_kernel_list_open(),
        "escolher fecha a lista"
    );
    assert!(e.aberto_ainda());
    let f = e.hero.store.pixel_edit_dialog().filter;
    assert_eq!(f.kernel, 5);
    assert_eq!(
        f.channels,
        [true, true, true, false],
        "os canais do kernel 5"
    );
    assert_eq!(
        e.hero.store.pixel_filter_preview(),
        Some(PixelDialogChoice::Convolution {
            kernel: 5,
            tiled: false,
            channels: [true, true, true, false],
            all: false,
        })
    );
    // Um kernel RGBA devolve o A.
    e.carrega(ids::PIXEL_FILTER_KERNEL);
    e.carrega(ids::pixel_filter_kernel_id(1).expect("linha"));
    assert_eq!(e.hero.store.pixel_edit_dialog().filter.channels, [true; 4]);
    // O Esc com a lista aberta fecha só a lista.
    e.carrega(ids::PIXEL_FILTER_KERNEL);
    e.tecla(KEY_ESCAPE, false);
    assert!(!e.hero.store.pixel_filter_kernel_list_open());
    assert!(
        e.aberto_ainda(),
        "o Esc fechou o diálogo com a lista aberta"
    );
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
            all: false,
        })
    );
    e.carrega(ids::PIXEL_FILTER_KERNEL);
    let k24 = rola_ate(&mut e, 24);
    e.carrega(k24);
    e.carrega(ids::PIXEL_FILTER_TILED);
    e.carrega(ids::PIXEL_FILTER_CHANNELS[3].1);
    let quer = PixelDialogChoice::Convolution {
        kernel: 24,
        tiled: true,
        channels: [true, true, true, false],
        all: false,
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
    e.carrega(ids::PIXEL_FILTER_KERNEL);
    e.carrega(ids::pixel_filter_kernel_id(3).expect("linha"));
    e.tecla(KEY_ESCAPE, false);
    assert!(!e.aberto_ainda());
    assert_eq!(e.pedido(), None);
}

#[test]
fn no_kernel_name_nor_label_of_the_convolution_dialog_is_cut() {
    let mut e = aberto();
    let esperados = [
        "pixel.filter_dialog.convolution.title",
        "pixel.filter_dialog.kernel",
        "pixel.filter_dialog.tiled",
        "pixel.filter_dialog.preview",
    ]
    .map(ph2d_i18n::tr);
    let ((), medidos) = elisao::medindo(|| e.pinta());
    for t in esperados {
        let md = medidos
            .iter()
            .find(|md| md.texto == t)
            .unwrap_or_else(|| panic!("«{t}» nunca foi medido"));
        assert!(md.coube(), "«{t}» cortado em «{}»", md.pintado);
    }
    // O chip na coluna do valor cabe o nome mais largo (o Medium, mais largo que o do chip); a
    // lista aberta mede-se a si própria (`Dropdown::list_width`).
    let chip = e
        .hero
        .hit_index
        .rect_for(ids::PIXEL_FILTER_KERNEL)
        .expect("chip");
    let largo = ph2d_editor_core::paint::label_column_width(
        &mut e.text,
        ph2d_tokens::TypeToken::Base.px(),
        nomes().iter().copied(),
    );
    let precisa = ph2d_editor_core::widget::dropdown_chip_width_for(largo, chip.h);
    assert!(
        precisa <= chip.w,
        "o chip ({}) corta o nome mais largo ({precisa})",
        chip.w
    );
}

/// **A regra de onde o diálogo de filtro assenta** (`place_dialog`), com os números da foto da
/// cena 19 (área livre 1290 × 645 em (332, 95), sprite 385 × 289 em (785, 248)): o Hue/Saturation
/// e o Convolution (com o dropdown, a largura dos irmãos e 6 filas) cabem ao lado e não tocam o
/// sprite; um diálogo largo que não cabe em lado nenhum fica no encosto de MENOR sobreposição dos
/// oito; nenhum desce à faixa da dica.
#[test]
fn a_filter_dialog_sits_where_it_covers_the_sprite_least() {
    use ph2d_editor_core::pixel_panel::place_dialog;
    use ph2d_editor_core::zones::Rect;
    let view = Rect::new(332.0, 95.0, 1290.0, 645.0);
    let sprite = (785, 248, 385, 289);
    let s = Rect::new(785.0, 248.0, 385.0, 289.0);
    let (margem, dica) = (12.0, 64.0);
    let toca = |r: Rect| {
        let ix = (r.x + r.w).min(s.x + s.w) - r.x.max(s.x);
        let iy = (r.y + r.h).min(s.y + s.h) - r.y.max(s.y);
        ix.max(0.0) * iy.max(0.0)
    };
    let hue = place_dialog(view, (320.0, 235.0), Some(sprite), margem, dica);
    assert_eq!(toca(hue), 0.0, "o Hue/Saturation ao lado: {hue:?}");
    assert!(hue.y + hue.h <= view.y + view.h - dica, "acima da dica");
    // O Convolution de hoje: a largura dos irmãos, 6 filas (título · Kernel · Tiled · canais ·
    // Preview · botões) — a altura dele é menor que a do Hue/Saturation (9 filas).
    let conv_hoje = place_dialog(view, (320.0, 200.0), Some(sprite), margem, dica);
    assert_eq!(toca(conv_hoje), 0.0, "o Convolution ao lado: {conv_hoje:?}");
    assert!(
        conv_hoje.y + conv_hoje.h <= view.y + view.h - dica,
        "acima da dica"
    );
    let conv = place_dialog(view, (650.0, 460.0), Some(sprite), margem, dica);
    assert!(conv.y + conv.h <= view.y + view.h - dica, "acima da dica");
    // O mínimo de verdade: varrer a vista inteira de 4 em 4 px não acha encosto que toque menos.
    let mut minimo = f32::MAX;
    let mut y = view.y;
    while y + 460.0 <= view.y + view.h - dica {
        let mut x = view.x;
        while x + 650.0 <= view.x + view.w {
            minimo = minimo.min(toca(Rect::new(x, y, 650.0, 460.0)));
            x += 4.0;
        }
        y += 4.0;
    }
    eprintln!(
        "[sonda cena 19] diálogo largo: sobreposição {} px² (mínimo varrido {minimo})",
        toca(conv)
    );
    assert!(
        toca(conv) <= minimo * 1.1 + 1.0,
        "{conv:?} toca {} > {minimo}",
        toca(conv)
    );
    // ⚠️ O antigo Convolution de 3 colunas (650 × 460) não cabia ao lado: o mínimo geométrico ainda
    // tapava ~51 % do sprite (56 933 de 111 265 px²); a regra continua a valer para um diálogo largo.
}

/// [`Dropdown::region_in_whole_rows`] directo, com o chip a percorrer uma linha inteira de
/// posições perto do TOPO (a lista abre abaixo) e do FUNDO (abre ACIMA) de uma janela baixa: a
/// lista clampada fica a `Spacing::Md` de cada borda e só mostra linhas inteiras.
#[test]
fn the_whole_rows_region_keeps_the_margin_below_and_above_the_chip() {
    use ph2d_a11y::NodeId;
    use ph2d_editor_core::widget::{Dropdown, DropdownOption};
    use ph2d_editor_core::zones::Rect;
    use ph2d_tokens::Spacing;
    let opcoes = (0..39)
        .map(|i| DropdownOption::new(NodeId(9_300_001 + i as u64), i, format!("{i}")))
        .collect();
    let dd = Dropdown::new(NodeId(9_300_000), "", opcoes);
    let janela = Rect::new(0.0, 0.0, 640.0, 300.0);
    let margem = Spacing::Md.px();
    let row_h = Spacing::Xl.px();
    let gap = Spacing::Xs.px();
    let resto = |h: f32| h.rem_euclid(row_h);
    let inteira = dd
        .popover_rect_clamped(
            Rect::new(0.0, 0.0, 80.0, row_h),
            Rect::new(0.0, 0.0, 640.0, 1.0e5),
        )
        .h;
    for passo in 0..(row_h as i32 * 2) {
        let d = passo as f32 * 0.5;
        for (chip, acima) in [
            (Rect::new(40.0, row_h + d, 80.0, row_h), false),
            (
                Rect::new(40.0, janela.h - 2.0 * row_h - d, 80.0, row_h),
                true,
            ),
        ] {
            let regiao = dd.region_in_whole_rows(chip, janela, margem);
            let p = dd.popover_rect_clamped(chip, regiao);
            let ctx = format!("chip {chip:?} → {p:?}");
            assert!(p.y >= janela.y + margem - 1e-3, "margem de cima: {ctx}");
            assert!(
                p.y + p.h <= janela.y + janela.h - margem + 1e-3,
                "margem de baixo: {ctx}"
            );
            if acima {
                assert!(p.y + p.h <= chip.y - gap + 1e-3, "abre ACIMA: {ctx}");
            } else {
                assert!(p.y >= chip.y + chip.h + gap - 1e-3, "abre abaixo: {ctx}");
            }
            assert!(
                (resto(p.h) - resto(inteira)).abs() < 1e-3,
                "só linhas inteiras: {ctx}"
            );
        }
    }
}
