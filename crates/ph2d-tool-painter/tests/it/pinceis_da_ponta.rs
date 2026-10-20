//! **Os pincéis de TESTE da ponta** (L9 · L18 · L19): o pincel Digital armado com a lei nova, gravado como preset
//! (`.ph2d-brush.png`) para a bancada medir o custo (`exame.sh --custo <ficheiro> 40,200`). Fora da fábrica.

use ph2d_brush_presets::{Miniatura, Origem, Parametros, Preset};
use ph2d_tool_painter::{LinhaDaPonta, PaintMedia, PainterTool, miniatura_do_preset, nome_do_meio};

/// Grava o pincel de `t` (Digital) em `dir/<nome>.ph2d-brush.png`.
pub(crate) fn grava(t: &PainterTool, dir: &str, nome: &str) -> std::path::PathBuf {
    let (parametros, texturas) =
        t.preset_capturar(&Parametros::novo(nome_do_meio(PaintMedia::Digital), "Ponta", nome));
    let mut p = Preset {
        parametros,
        origem: Origem::Utilizador,
        miniatura: Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![0; 4],
        },
        texturas,
    };
    p.miniatura = miniatura_do_preset(&p);
    let f = std::path::Path::new(dir).join(format!("{nome}.ph2d-brush.png"));
    std::fs::write(&f, ph2d_brush_presets::escrever(&p).unwrap()).unwrap();
    f
}

/// O lápis de píxel (L18) e o lápis de grão (L9: Linear Height pela pressão) em `PH2D_PONTA_OUT`.
#[test]
#[ignore = "gerador: os pincéis de teste da ponta (PH2D_PONTA_OUT=<pasta>)"]
fn gera_os_pinceis_de_teste_da_ponta() {
    let dir = std::env::var("PH2D_PONTA_OUT").expect("PH2D_PONTA_OUT");
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::Digital);
    t.set_ponta(LinhaDaPonta::Pixel, 2.0);
    eprintln!("{}", grava(&t, &dir, "Pixel Perfect").display());
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_texture_kind(27);
    t.set_brush_texture_mapping(1);
    t.set_brush_texture_size(0, 6.0);
    t.set_brush_texture_size(1, 6.0);
    t.set_ponta(LinhaDaPonta::GrainMode, 5.0);
    t.set_ponta(LinhaDaPonta::DepthJitter, 1.0);
    t.set_ponta(LinhaDaPonta::DepthPressure, 1.0);
    eprintln!("{}", grava(&t, &dir, "Grain Pencil").display());
}
