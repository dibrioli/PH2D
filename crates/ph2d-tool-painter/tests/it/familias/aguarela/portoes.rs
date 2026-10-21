//! Os portões do MOTOR da família `aguarela` (J4 — `docs/Painter/Pinceis/exames/aguarela.md` «Motor J4»).

use super::sonda::{medidas, perfil};
use crate::exame_bancada::{Fonte, pinceis};
use crate::sonda_gestos::{AZUL, gesto, tela};
use ph2d_editor_core::Tool;

/// O recto de `diametro` px do pincel `nome` com o mapa `engine` sobreposto, lido no GRID logo depois do traço (com a
/// caneta em baixo a água não corre: é o depósito) — `[largura, média, listras, oco, buracos, mancha]`.
fn deposito(nome: &str, engine: &[(&str, f64)], diametro: f32, y: f32) -> [f64; 6] {
    deposito_e_agua(nome, engine, diametro, y).0
}

/// [`deposito`] e o mesmo traço depois de 240 quadros de água (a massa `susp + sett`).
fn deposito_e_agua(nome: &str, engine: &[(&str, f64)], diametro: f32, y: f32) -> ([f64; 6], [f64; 6]) {
    let p = pinceis(nome).remove(0);
    let Fonte::Preset { preset, .. } = &p.fonte else {
        unreachable!("{nome} é um preset de ficheiro")
    };
    let mut pr = (**preset).clone();
    for (k, v) in engine {
        pr.parametros.engine.insert((*k).to_owned(), *v);
    }
    let mut t = tela();
    t.set_semente_dos_tracos(0);
    t.set_wet_relogio_fixo(true);
    let falhas = t.preset_aplicar(&pr).expect("meio");
    // Um knob que o motor não conhece não pinta nada de novo: a régua abaixo é que o diz (o vermelho de antes do filme).
    assert!(falhas.is_empty() || !engine.is_empty(), "{nome}: {falhas:?}");
    if !falhas.is_empty() {
        println!("[filme] {nome}: {falhas:?}");
    }
    t.set_brush_size_px(diametro / 2.0);
    t.set_brush_color_srgb8(AZUL);
    gesto(&mut t, &[[60.0, y], [940.0, y]], 3.0, |_| 1.0);
    let m = f64::from(diametro) * 0.8;
    let (y0, y1) = (f64::from(y) - m, f64::from(y) + m);
    let antes = medidas(&perfil(&mut t, y0, y1, 300.0, 700.0), 0).expect("o traço pousou");
    for _ in 0..240 {
        t.on_tick(1000.0 / 60.0);
    }
    (antes, medidas(&perfil(&mut t, y0, y1, 300.0, 700.0), 9).expect("a tinta ficou"))
}

/// ⭐ **O FILME do traço é liso** (`Knob::StrokeFilm`, J4 aguarela): com ele, o depósito de um recto do Wet Liner é um
/// véu — sem as listras das linhas do tapete das cerdas (que a soma dos dabs arrasta pelo traço inteiro) e sem o domo da
/// soma das cordas; sem ele (o controlo, a lei de sempre), as duas coisas que os seis revisores cegos viram.
#[test]
fn o_filme_do_traco_e_liso() {
    for (diametro, y) in [(200.0f32, 440.0f32), (40.0, 190.0)] {
        // Pigmento 30 nos dois: o filme sem o tapete pousa ~3× o miolo de antes, e o Liner (100) passava o tecto de 3000
        // da célula (o excesso sai pela média da janela — outro mecanismo).
        let soma = deposito("Wet Liner", &[("pigmentPerDab", 30.0)], diametro, y);
        // O Drag do preset puxa a tinta de trás para as células da crista que ainda não estão cheias e deixa-as acima do
        // filme (medido: mancha 0,055 com o 0,16 de omissão, 0,008 sem ele) — um pincel de filme leva `drag` 0.
        let filme = deposito("Wet Liner", &[("pigmentPerDab", 30.0), ("strokeFilm", 1.0), ("drag", 0.0)], diametro, y);
        println!("[filme] {diametro} px · soma {soma:.3?} · filme {filme:.3?}");
        assert!(soma[2] > 0.03, "{diametro} px: o controlo já não tem listras ({:.4}) — a régua não vê o defeito", soma[2]);
        assert!(filme[2] < 0.01, "{diametro} px: listras {:.4} com o filme", filme[2]);
        assert!(filme[3] > 0.93, "{diametro} px: o domo (ombros ÷ centro {:.3}) com o filme", filme[3]);
        assert!(filme[4] == 0.0, "{diametro} px: buracos {:.3} no miolo do filme", filme[4]);
        assert!(filme[5] < 0.02, "{diametro} px: manchas ao longo do traço ({:.3}) com o filme", filme[5]);
    }
}

/// ⭐ **E a água não o desfaz** (`Knob::StrokeFilm`): depois de 240 quadros o filme do Wet Liner a 200 px continua um
/// véu — sem o 2.º gole da célula virgem (que dependia de quanto a 1.ª janela lhe trouxe e deixava um ARCO de água por
/// janela, medido: mancha 0,095 → 0,038 no Mop) e com o tecto de água que o filme pede (`waterCap` 30: com o 8 de
/// omissão o excesso sai pela média da janela, outro arco).
#[test]
fn o_filme_do_traco_continua_liso_depois_da_agua() {
    let filme = [("pigmentPerDab", 30.0), ("strokeFilm", 1.0), ("drag", 0.0), ("waterCap", 30.0)];
    let (_, agua) = deposito_e_agua("Wet Liner", &filme, 200.0, 440.0);
    println!("[filme] 200 px depois de 240 quadros: {agua:.3?}");
    assert!(agua[2] < 0.01, "listras {:.4} depois da água", agua[2]);
    assert!(agua[3] > 0.93, "o domo (ombros ÷ centro {:.3}) depois da água", agua[3]);
    assert!(agua[5] < 0.03, "manchas ao longo do traço ({:.3}) depois da água", agua[5]);
}
