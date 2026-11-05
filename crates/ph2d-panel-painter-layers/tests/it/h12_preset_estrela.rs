//! **O topo do Painter na ordem do dono (04/11):** Paint Mode, prévia, Preset com a estrela do favorito à direita,
//! os botões de guardar. A estrela é a mesma `PAINTER_WETPAINT_PRESET_FAVORITE` da caixa; `PH2D_CARDS=0` fica como era.

use super::h1_os_cartoes::{Mesa, pincel};
use super::h2_os_cartoes::{ComPresets, ROUND, WASH, previa};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::censo;
use ph2d_editor_core::widget::dropdown_label_budget;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_panel_painter_layers::{
    PainterLayersPanel, VistaPreset, VistaPresets, rect_da_preview, set_current_wet_presets,
};
use ph2d_tool_painter::PaintMedia;
use ph2d_tool_painter::ids as p;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

const ESTRELA: ph2d_a11y::NodeId = p::PAINTER_WETPAINT_PRESET_FAVORITE;
const CHIP: ph2d_a11y::NodeId = p::PAINTER_WETPAINT_PRESET;
const JANELA_DE_FABRICA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 6000.0,
};

/// A vista que a ponte publica: um preset activo, favorito ou não.
fn um_preset(nome: &str, favorito: bool) -> Arc<VistaPresets> {
    Arc::new(VistaPresets {
        entradas: vec![VistaPreset {
            relativo: WASH.to_owned(),
            nome: nome.to_owned(),
            categoria: "Watercolor".to_owned(),
            favorito,
            apagavel: false,
        }],
        ativo: Some(WASH.to_owned()),
        politica: 1,
        ilegiveis: 0,
        ordem_manual: true,
    })
}

fn mesa_digital(cartoes: bool) -> Mesa {
    let (b, _) = pincel(PaintMedia::Digital, "brush");
    Mesa::nova(cartoes, b, false)
}

fn fundo(r: Rect) -> f32 {
    r.y + r.h
}

/// A tinta do quadro: cada cor de desenho, com quantas vezes aparece.
fn tinta(mesa: &mut Mesa) -> BTreeMap<u32, i32> {
    // O arnês não tem o tique que publica o pairado: sem ele, todo id conta como pairado.
    mesa.host.store_mut().set_hover_live(ESTRELA, 0.0);
    let dados = mesa
        .host
        .paint_draw_data::<PainterLayersPanel>(&mut mesa.st, mesa.janela);
    let mut contagem = BTreeMap::new();
    for cor in dados {
        *contagem.entry(cor).or_insert(0) += 1;
    }
    contagem
}

/// O que `b` tem a mais do que `a` e o que tem a menos, em número de desenhos.
fn diferenca(a: &BTreeMap<u32, i32>, b: &BTreeMap<u32, i32>) -> (Vec<i32>, Vec<i32>) {
    let (mut mais, mut menos) = (Vec::new(), Vec::new());
    let chaves: std::collections::BTreeSet<_> = a.keys().chain(b.keys()).collect();
    for cor in chaves {
        let d = b.get(cor).unwrap_or(&0) - a.get(cor).unwrap_or(&0);
        if d > 0 {
            mais.push(d);
        } else if d < 0 {
            menos.push(-d);
        }
    }
    mais.sort_unstable();
    menos.sort_unstable();
    (mais, menos)
}

/// A ordem pedida, medida em y, com o rótulo «Preset» por cima do chip.
#[test]
fn the_top_reads_mode_preview_preset_and_star_then_the_save_buttons() {
    let _p = ComPresets::com(Some(WASH));
    previa();
    let mut mesa = mesa_digital(true);
    let modo = mesa.rect(p::PAINTER_BRUSH_MEDIA).expect("Paint Mode");
    let prev = rect_da_preview().expect("a prévia");
    let chip = mesa.rect(CHIP).expect("Preset");
    let estrela = mesa.rect(ESTRELA).expect("a estrela");
    let guardar = mesa
        .rect(p::PAINTER_WETPAINT_PRESET_SAVE_NEW)
        .expect("Save as New");
    let padrao = mesa
        .rect(p::PAINTER_WETPAINT_PRESET_SAVE_DEFAULT)
        .expect("Save as Default");
    println!(
        "MEDIDA modo {} chip {chip:?} estrela {estrela:?} guardar.y {} politica.y {:?}",
        modo.y,
        guardar.y,
        mesa.rect(p::PAINTER_WETPAINT_PRESET_POLICY).map(|r| r.y),
    );
    assert!(fundo(modo) <= prev.y, "{modo:?} × {prev:?}");
    assert!(fundo(prev) <= chip.y, "{prev:?} × {chip:?}");
    assert!(fundo(chip) <= guardar.y, "{chip:?} × {guardar:?}");
    assert_eq!(
        (estrela.y, estrela.h),
        (chip.y, chip.h),
        "a estrela na fila do chip"
    );
    assert_eq!(estrela.w, estrela.h, "um quadrado");
    assert!(estrela.x >= chip.x + chip.w, "à direita do chip");
    assert!(
        (estrela.x + estrela.w - (padrao.x + padrao.w)).abs() < 1.0,
        "a borda direita é a da grelha: {estrela:?} × {padrao:?}"
    );
    // Nada grudado: a prévia e o rótulo têm o vão, e o rótulo fica entre a prévia e o chip.
    let (_, c) = censo::grava(|| mesa.quadro());
    let linha = c
        .linhas
        .iter()
        .find(|l| l.celulas.iter().any(|x| x.nome == "Preset"))
        .expect("o rótulo Preset no censo");
    assert!(
        linha.topo >= fundo(prev) && linha.topo < chip.y,
        "o rótulo em cima: {} entre {} e {}",
        linha.topo,
        fundo(prev),
        chip.y
    );
}

/// Sem preset activo não há o que favoritar: o chip toma a fila inteira.
#[test]
fn without_an_active_preset_there_is_no_star_and_the_chip_takes_the_row() {
    let _p = ComPresets::com(None);
    let mesa = mesa_digital(true);
    assert!(mesa.rect(ESTRELA).is_none(), "a condição é a de antes");
    let chip = mesa.rect(CHIP).expect("Preset");
    let padrao = mesa
        .rect(p::PAINTER_BRUSH_MEDIA)
        .expect("Paint Mode");
    assert!((chip.x + chip.w - (padrao.x + padrao.w)).abs() < 1.0);
}

/// Cheia e acesa quando favorita, vazia quando não. A diferença exacta entre as duas caras:
/// a favorita ganha o fundo aceso e dois desenhos no tom aceso (o miolo e o contorno), e perde o contorno apagado.
#[test]
fn the_favorite_star_is_full_and_lit_and_the_other_is_empty() {
    let _p = ComPresets::com(Some(WASH));
    set_current_wet_presets(Some(um_preset("Wet Wash", false)));
    let vazia = tinta(&mut mesa_digital(true));
    set_current_wet_presets(Some(um_preset("Wet Wash", true)));
    let cheia = tinta(&mut mesa_digital(true));
    let (mais, menos) = diferenca(&vazia, &cheia);
    assert_eq!(mais, vec![1, 2], "fundo aceso + miolo e contorno acesos");
    assert_eq!(menos, vec![1], "o contorno apagado sai");
}

/// O gesto real: o clique pede o que a caixa pedia; a ponte publica; a tinta e a dica mudam; o 2.º clique volta.
#[test]
fn clicking_the_star_toggles_the_favorite_and_changes_its_ink_and_tooltip() {
    let _p = ComPresets::com(Some(WASH));
    set_current_wet_presets(Some(um_preset("Wet Wash", false)));
    let pedido = mesa_digital(false).clica(ESTRELA);
    let mut mesa = mesa_digital(true);
    let dica = |m: &Mesa| m.host.store().tooltip_for(ESTRELA).map(str::to_owned);
    let vazia = tinta(&mut mesa);
    let dica_vazia = dica(&mesa).expect("dica da estrela vazia");

    let primeiro = mesa.clica(ESTRELA);
    assert!(!primeiro.is_empty(), "nada no barramento");
    assert_eq!(primeiro, pedido, "o mesmo pedido da caixa de antes");
    let _ = mesa.ponteiro(PointerKind::Move, 1.0, 1.0);
    set_current_wet_presets(Some(um_preset("Wet Wash", true)));
    mesa.assenta();
    let cheia = tinta(&mut mesa);
    let dica_cheia = dica(&mesa).expect("dica da estrela cheia");
    assert_ne!(vazia, cheia, "a tinta muda");
    assert_ne!(dica_vazia, dica_cheia, "a dica diz o que o clique fará");
    assert!(dica_vazia.contains("Add"), "{dica_vazia}");
    assert!(dica_cheia.contains("Remove"), "{dica_cheia}");

    let segundo = mesa.clica(ESTRELA);
    assert_eq!(segundo, pedido, "o mesmo pedido alterna de volta");
    let _ = mesa.ponteiro(PointerKind::Move, 1.0, 1.0);
    set_current_wet_presets(Some(um_preset("Wet Wash", false)));
    mesa.assenta();
    assert_eq!(tinta(&mut mesa), vazia, "volta à cara de antes");
    assert_eq!(dica(&mesa).as_deref(), Some(dica_vazia.as_str()));
}

fn nomes_de_fabrica(pasta: &Path, nomes: &mut Vec<String>) {
    for e in std::fs::read_dir(pasta).expect("a pasta dos pincéis") {
        let c = e.expect("entrada").path();
        if c.is_dir() {
            nomes_de_fabrica(&c, nomes);
        } else if let Some(n) = c
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_suffix(".ph2d-brush.png"))
        {
            nomes.push(n.to_owned());
        }
    }
}

/// Na largura de fábrica, com a estrela ao lado, o chip mostra inteiro o nome de cada pincel de fábrica.
#[test]
fn the_chip_beside_the_star_shows_every_factory_name_whole() {
    let _p = ComPresets::com(Some(WASH));
    let (b, _) = pincel(PaintMedia::Digital, "brush");
    let mesa = Mesa::na_janela(true, b, false, JANELA_DE_FABRICA);
    let chip = mesa.rect(CHIP).expect("Preset");
    let cabe = dropdown_label_budget(chip);
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let mut nomes = Vec::new();
    nomes_de_fabrica(&raiz, &mut nomes);
    assert!(nomes.len() >= 77, "só {} pincéis de fábrica", nomes.len());
    let mut texto = ph2d_text::TextSystem::without_system_fonts();
    let (maior, largura) = nomes
        .iter()
        .map(|n| (n, texto.prefix_width(n, m::VALUE_PX)))
        .fold((&nomes[0], 0.0_f32), |a, b| if b.1 > a.1 { b } else { a });
    println!("MEDIDA o maior nome {maior:?} pede {largura}, o chip dá {cabe}");
    assert!(largura <= cabe, "{maior:?} pede {largura}, o chip dá {cabe}");
}

/// `PH2D_CARDS=0`: a caixa `Favorite` em fila própria, entre o Preset e a prévia, como era.
#[test]
fn cards_off_keeps_the_favorite_box_between_the_preset_and_the_preview() {
    let _p = ComPresets::com(Some(ROUND));
    previa();
    let mesa = mesa_digital(false);
    let chip = mesa.rect(CHIP).expect("Preset");
    let caixa = mesa.rect(ESTRELA).expect("Favorite");
    let prev = rect_da_preview().expect("a prévia");
    assert!(fundo(chip) <= caixa.y && fundo(caixa) <= prev.y);
    assert!(caixa.w > caixa.h * 2.0, "a linha inteira é o alvo da caixa");
}

/// As duas caras da estrela (a cena do app só mostra a do preset dela): `PH2D_FOTO_DIR=<pasta>`, `--ignored`.
#[test]
#[ignore = "fotos: PH2D_FOTO_DIR=<pasta> e --ignored"]
fn h12_foto_preset_estrela() {
    let Ok(dir) = std::env::var("PH2D_FOTO_DIR") else {
        return;
    };
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("a foto precisa da placa");
    let _p = ComPresets::com(Some(WASH));
    let (b, _) = pincel(PaintMedia::Digital, "brush");
    for (nome, preset, favorito) in [
        ("h12_estrela_vazia", "Compressed Charcoal", false),
        ("h12_estrela_cheia", "Compressed Charcoal", true),
    ] {
        let mut l = vec![0u8; 256 * 96 * 4];
        for (i, px) in l.chunks_mut(4).enumerate() {
            px.copy_from_slice(&[225, 215, 195, 255]);
            if i % 256 > 40 && i % 256 < 216 && (i / 256) % 96 > 44 && (i / 256) % 96 < 52 {
                px.copy_from_slice(&[70, 60, 55, 255]);
            }
        }
        ph2d_panel_painter_layers::set_current_wet_preview(Some((Arc::new(l), 256, 96)));
        set_current_wet_presets(Some(um_preset(preset, favorito)));
        let mut m = Mesa::na_janela(true, b, false, Rect::new(0.0, 0.0, 1600.0, 2400.0));
        m.so_abertos(&[p::PAINTER_CARD_TOPO]);
        m.assenta();
        m.host.store_mut().set_hover_live(ESTRELA, 0.0);
        super::h10_fotos::foto(&gpu, &dir, nome, &mut m);
    }
}
