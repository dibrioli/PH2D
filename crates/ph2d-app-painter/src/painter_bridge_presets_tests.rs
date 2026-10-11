//! A ponte dos presets sobre o disco: a fábrica REAL (`assets/brushes/`, só lida) e uma pasta do
//! artista temporária. Cada pedido do painel → o pincel e os ficheiros mudam como o botão diz.

use super::*;
use ph2d_editor_core::toast::ToastSeverity;
use ph2d_tool_painter::PaintMedia;

const ROUND: &str = "WetPainter/Watercolor/Fluid/Wet Round.ph2d-brush.png";
const WASH: &str = "WetPainter/Watercolor/Fluid/Wet Wash.ph2d-brush.png";

fn pasta(nome: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ph2d_bridge_presets_{}_{nome}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn abre(nome: &str) -> (Presets, PainterTool, ToastQueue, PathBuf) {
    let uti = pasta(nome);
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    let mut toasts = ToastQueue::default();
    let p = Presets::abrir_em(pasta_de_fabrica(), Some(uti.clone()), &mut t, &mut toasts);
    (p, t, toasts, uti)
}

fn sem_erros(toasts: &ToastQueue) {
    let erros: Vec<&str> = toasts
        .iter()
        .filter(|t| t.severity == ToastSeverity::ErrorState)
        .map(|t| t.message.as_str())
        .collect();
    assert!(erros.is_empty(), "{erros:?}");
}

/// Um knob do pincel pela chave do registo (a mesma que o preset grava).
fn knob(t: &PainterTool, chave: &str) -> f64 {
    let base = Parametros::novo(MEIO_DO_PRESET, "", "");
    t.wet_preset_capturar(&base).0.engine[chave]
}

#[test]
fn escolher_poe_o_preset_no_pincel_e_lembra_o_ativo() {
    let (mut p, mut t, mut toasts, uti) = abre("escolher");
    assert_eq!(p.vista.entradas.len(), 4, "a fábrica inteira no navegador");
    assert_eq!(p.vista.ativo, None);
    p.atender(&mut t, PedidoPreset::Escolher(WASH.into()), &mut toasts);
    sem_erros(&toasts);
    assert_eq!(
        knob(&t, "pigmentPerDab"),
        200.0,
        "o Wash do 02 §6.9 (calibrado contra o Rebelle 10/10)"
    );
    assert_eq!(knob(&t, "bristleCount"), 3000.0);
    assert_eq!(p.vista.ativo.as_deref(), Some(WASH));
    // Reaberto (outra sessão): o pincel abre com o Wash.
    let mut t2 = PainterTool::default();
    t2.set_paint_media(PaintMedia::WetPaint);
    let p2 = Presets::abrir_em(pasta_de_fabrica(), Some(uti), &mut t2, &mut toasts);
    assert_eq!(p2.vista.ativo.as_deref(), Some(WASH));
    assert_eq!(knob(&t2, "pigmentPerDab"), 200.0);
}

#[test]
fn a_politica_decide_o_que_volta_com_o_preset() {
    for (politica, esperado) in [(0u8, 0.3), (1, 0.3), (2, 1.0)] {
        let (mut p, mut t, mut toasts, _) = abre(&format!("politica{politica}"));
        p.atender(&mut t, PedidoPreset::Politica(politica), &mut toasts);
        p.atender(&mut t, PedidoPreset::Escolher(ROUND.into()), &mut toasts);
        t.wet_preset_aplicar_volumes(&ph2d_brush_presets::Volumes {
            water: Some(0.3),
            ..Default::default()
        });
        p.atender(&mut t, PedidoPreset::Escolher(WASH.into()), &mut toasts);
        p.atender(&mut t, PedidoPreset::Escolher(ROUND.into()), &mut toasts);
        assert_eq!(t.wet_knobs().water, esperado, "política {politica}");
        sem_erros(&toasts);
    }
}

#[test]
fn guardar_como_novo_cria_um_ficheiro_do_artista_e_passa_a_activo() {
    let (mut p, mut t, mut toasts, uti) = abre("novo");
    p.atender(&mut t, PedidoPreset::Escolher(ROUND.into()), &mut toasts);
    t.set_brush_size_px(9.0);
    p.atender(&mut t, PedidoPreset::GuardarComoNovo, &mut toasts);
    sem_erros(&toasts);
    let rel = "WetPainter/Watercolor/Fluid/Wet Round 2.ph2d-brush.png";
    assert!(uti.join(rel).is_file(), "o ficheiro do artista existe");
    assert_eq!(p.vista.ativo.as_deref(), Some(rel));
    let e = p.bib.procurar(rel).unwrap();
    assert_eq!(e.origem, Origem::Utilizador);
    assert_eq!(
        e.preset.parametros.volumes.size_px,
        Some(18.0),
        "guardou o diâmetro"
    );
    assert_ne!(e.preset.miniatura.largura, 1, "a miniatura foi pintada");
    assert!(p.bib.procurar(ROUND).is_some(), "o de fábrica continua lá");
}

#[test]
fn guardar_como_padrao_sobrepoe_e_o_reset_volta_a_fabrica() {
    let (mut p, mut t, mut toasts, uti) = abre("padrao");
    p.atender(&mut t, PedidoPreset::Escolher(ROUND.into()), &mut toasts);
    let fabrica = knob(&t, "pigmentPerDab");
    // O artista mexe no Pigment (pelo mesmo caminho que um preset usa para escrever um knob).
    let mut prm = p.bib.procurar(ROUND).unwrap().preset.clone();
    prm.parametros.engine.insert("pigmentPerDab".into(), 1500.0);
    let _ = t.wet_preset_aplicar(&prm);
    p.atender(&mut t, PedidoPreset::GuardarComoPadrao, &mut toasts);
    sem_erros(&toasts);
    assert!(uti.join(ROUND).is_file());
    let e = p.bib.procurar(ROUND).unwrap();
    assert_eq!((e.origem, e.tem_fabrica), (Origem::Utilizador, true));
    assert_eq!(e.preset.parametros.engine["pigmentPerDab"], 1500.0);

    p.atender(&mut t, PedidoPreset::Repor, &mut toasts);
    sem_erros(&toasts);
    assert!(
        !uti.join(ROUND).exists(),
        "o Reset apagou a cópia do artista"
    );
    assert_eq!(p.bib.procurar(ROUND).unwrap().origem, Origem::Fabrica);
    assert_eq!(
        knob(&t, "pigmentPerDab"),
        fabrica,
        "e o pincel voltou ao de fábrica"
    );
}

#[test]
fn o_favorito_aparece_na_vista() {
    let (mut p, mut t, mut toasts, _) = abre("favorito");
    p.atender(&mut t, PedidoPreset::Escolher(WASH.into()), &mut toasts);
    p.atender(&mut t, PedidoPreset::AlternarFavorito, &mut toasts);
    assert!(p.vista.procurar(WASH).unwrap().favorito);
    p.atender(&mut t, PedidoPreset::AlternarFavorito, &mut toasts);
    assert!(!p.vista.procurar(WASH).unwrap().favorito);
}
