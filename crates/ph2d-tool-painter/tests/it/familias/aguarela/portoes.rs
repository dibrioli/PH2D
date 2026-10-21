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

/// O pincel das réguas: o Wet Liner de antes do filme (pigmento 30 — o filme sem o tapete pousa ~3× o miolo da soma, e
/// o Liner a 100 passava o tecto de 3000 da célula), escrito aqui para o portão não andar com o ficheiro de fábrica.
const BASE: [(&str, f64); 3] = [("pigmentPerDab", 30.0), ("paperGate", 0.05), ("edgeDarkening", 100.0)];
/// A soma de sempre (o controlo): sem o filme, o arrasto e o tecto de água de omissão.
const SOMA: [(&str, f64); 6] = [BASE[0], BASE[1], BASE[2], ("strokeFilm", 0.0), ("drag", 0.16), ("waterCap", 8.0)];
/// O filme, com o arrasto a 0 (medido abaixo) e o tecto de água de omissão (8 — o filme não o lê).
const FILME: [(&str, f64); 6] = [BASE[0], BASE[1], BASE[2], ("strokeFilm", 1.0), ("drag", 0.0), ("waterCap", 8.0)];

/// ⭐ **O FILME do traço é liso** (`Knob::StrokeFilm`, J4 aguarela): com ele, o depósito de um recto do Wet Liner é um
/// véu — sem as listras das linhas do tapete das cerdas (que a soma dos dabs arrasta pelo traço inteiro) e sem o domo da
/// soma das cordas; sem ele (o controlo, a lei de sempre), as duas coisas que os seis revisores cegos viram.
#[test]
fn o_filme_do_traco_e_liso() {
    for (diametro, y) in [(200.0f32, 440.0f32), (40.0, 190.0)] {
        let soma = deposito("Wet Liner", &SOMA, diametro, y);
        // (O Drag de omissão puxa a tinta de trás para as células da crista ainda por encher e deixa-as acima do filme:
        // mancha 0,055 com o 0,16, 0,008 sem ele — o pincel de filme leva `drag` 0.)
        let filme = deposito("Wet Liner", &FILME, diametro, y);
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
/// janela, medido: mancha 0,095 → 0,038 no Mop) e sem o tecto de água da janela (o excesso saía pela média dela aos
/// bocados, e a 1.ª janela pousava o seu inteiro: o disco do 1.º toque, que o revisor cego viu no Liner a 200 px).
#[test]
fn o_filme_do_traco_continua_liso_depois_da_agua() {
    // E molhado (a água por dab no tecto da faixa: o filme passa o tecto de água da célula, como o Mop).
    let molhado = [FILME[0], FILME[1], FILME[2], FILME[3], FILME[4], FILME[5], ("waterPerDab", 4.0)];
    let (_, agua) = deposito_e_agua("Wet Liner", &molhado, 200.0, 440.0);
    println!("[filme] 200 px molhado depois de 240 quadros: {agua:.3?}");
    // (0,038 com a lei; 0,078 com o tecto da janela de volta — o disco do 1.º toque e os arcos da água.)
    assert!(agua[3] > 0.93 && agua[5] < 0.055, "molhado: ombros ÷ centro {:.3}, manchas {:.3}", agua[3], agua[5]);
    let (_, agua) = deposito_e_agua("Wet Liner", &FILME, 200.0, 440.0);
    println!("[filme] 200 px depois de 240 quadros: {agua:.3?}");
    // (A água nivela o que sobra das listras também na soma — aqui elas são só a régua do ruído: 0,009–0,011.)
    assert!(agua[2] < 0.015, "listras {:.4} depois da água", agua[2]);
    assert!(agua[3] > 0.93, "o domo (ombros ÷ centro {:.3}) depois da água", agua[3]);
    assert!(agua[5] < 0.03, "manchas ao longo do traço ({:.3}) depois da água", agua[5]);
}

/// ⭐ **O filme tem a MESMA espessura a todo o tamanho** (`Knob::StrokeFilm`): o miolo do recto de 5, 12, 40 e 200 px pousa a
/// mesma massa por célula — a espessura é a do miolo de um traço largo e o peso do traço fino desfaz-se. Tirada da corda
/// (`2r ÷ passo`, a 1.ª lei), o fino ficava a 0,71–0,75 do largo e o recto de 5 px do Wet Wash cobria 0,62 do de 40
/// (`leis_sistemicas`: o Rebelle 0,98).
#[test]
fn o_filme_tem_a_mesma_espessura_a_todo_o_tamanho() {
    let largo = deposito("Wet Liner", &FILME, 200.0, 320.0)[1];
    for d in [5.0f32, 12.0, 40.0] {
        let m = deposito("Wet Liner", &FILME, d, 320.0)[1];
        println!("[filme] miolo a {d} px {m:.1} · a 200 px {largo:.1}");
        assert!((m / largo - 1.0).abs() < 0.1, "a {d} px o filme tem {:.2} do de 200", m / largo);
    }
}
