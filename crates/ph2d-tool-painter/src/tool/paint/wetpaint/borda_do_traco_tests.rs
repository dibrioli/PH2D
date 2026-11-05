//! **O Edge Darkening é de cada traço** (Bug #69, dono 05/11): trocar de pincel não muda a borda da tinta pousada, e o
//! traço novo seca com a borda do pincel dele. Sempre contra um braço de controle com os mesmos tiques.

use super::offthread::otica::forca_no_teste;
use super::sonda_bordas_longe::{H, REGIAO, W, dif, ecra, por_nome, tiques, traco};
use crate::tool::PainterTool;
use ph2d_brush_presets::Biblioteca;
use ph2d_wet_paint::tuning::Knob;

fn biblioteca() -> Biblioteca {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    Biblioteca::carregar(&raiz, None)
}

/// A pinta o traço, espera 60 tiques, `troca` age, e correm mais `depois` tiques: o que o ecrã mostra.
fn cena(lib: &Biblioteca, a: &str, troca: &dyn Fn(&mut PainterTool), depois: usize) -> Vec<u8> {
    forca_no_teste(Some(false));
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_brush_paper_kind(ph2d_painter_brush::TextureKind::PaperCold.to_u8());
    t.set_substrate_depth(0.7);
    t.preset_aplicar(por_nome(lib, a)).expect("preset A");
    t.set_brush_size_px(34.0);
    t.set_brush_color_srgb8([120, 120, 125]);
    traco(&mut t, 70.0, false);
    tiques(&mut t, 60, false);
    troca(&mut t);
    tiques(&mut t, depois, false);
    forca_no_teste(None);
    ecra(&mut t)
}

/// O controlo sozinho, sem pincelada: o caso medido de 4 332 px, Δ 30.
#[test]
fn mudar_o_controlo_nao_toca_a_borda_pousada() {
    let lib = biblioteca();
    let ctl = cena(&lib, "Wet Liner", &|_| {}, 600);
    let outra = cena(
        &lib,
        "Wet Liner",
        &|t| t.paint.wetpaint.knobs.set(Knob::EdgeDarkening, 56.25),
        600,
    );
    assert_eq!(dif(&ctl, &outra, 0, REGIAO), (0, 0));
}

/// O outro sentido: tinta com borda forte, controlo a 0.
#[test]
fn baixar_o_controlo_nao_amacia_a_borda_pousada() {
    let lib = biblioteca();
    let ctl = cena(&lib, "Wet Round", &|_| {}, 600);
    let outra = cena(
        &lib,
        "Wet Round",
        &|t| t.paint.wetpaint.knobs.set(Knob::EdgeDarkening, 0.0),
        600,
    );
    assert_eq!(dif(&ctl, &outra, 0, REGIAO), (0, 0));
}

/// O traço novo seca com a borda DELE, e a antiga não: o mesmo B sobre tela com A e sobre tela só dele, na região de B.
#[test]
fn o_traco_novo_seca_com_a_borda_do_pincel_dele() {
    let lib = biblioteca();
    let b = por_nome(&lib, "Wet Dry Brush");
    let pinta_b = |t: &mut PainterTool| {
        t.preset_aplicar(b).expect("preset B");
        t.set_brush_size_px(34.0);
        traco(t, 270.0, false);
    };
    let com_a = cena(&lib, "Wet Liner", &pinta_b, 300);
    let so_a = cena(&lib, "Wet Liner", &|_| {}, 300);
    // A tinta de B escureceu a borda dela: sem isto o controlo teria virado letra morta para o traço novo.
    let (px, _) = dif(&com_a, &so_a, REGIAO, H);
    assert!(px > 1000, "o traço B apareceu: {px} px");

    // O mesmo B com o controlo a 0: a borda tem de mudar, ou a célula não leu o valor do pincel.
    let sem_borda = cena(
        &lib,
        "Wet Liner",
        &|t| {
            t.preset_aplicar(b).expect("preset B");
            t.set_brush_size_px(34.0);
            t.paint.wetpaint.knobs.set(Knob::EdgeDarkening, 0.0);
            traco(t, 270.0, false);
        },
        300,
    );
    let (px, delta) = dif(&com_a, &sem_borda, REGIAO, H);
    assert!(px > 100 && delta > 3, "a borda de B: {px} px, Δ {delta}");
}
