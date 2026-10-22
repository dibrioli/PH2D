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
fn deposito_e_agua(
    nome: &str,
    engine: &[(&str, f64)],
    diametro: f32,
    y: f32,
) -> ([f64; 6], [f64; 6]) {
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
    assert!(
        falhas.is_empty() || !engine.is_empty(),
        "{nome}: {falhas:?}"
    );
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
    (
        antes,
        medidas(&perfil(&mut t, y0, y1, 300.0, 700.0), 9).expect("a tinta ficou"),
    )
}

/// O pincel das réguas: o Wet Liner de antes do filme (pigmento 30 — o filme sem o tapete pousa ~3× o miolo da soma, e
/// o Liner a 100 passava o tecto de 3000 da célula), escrito aqui para o portão não andar com o ficheiro de fábrica.
/// (3.ª ronda: e a água, a capilaridade, o vidrado, a sujidade da ponta e a parede de OMISSÃO — o Liner de fábrica levou-os.)
const BASE: [(&str, f64); 8] = [
    ("pigmentPerDab", 30.0),
    ("paperGate", 0.05),
    ("edgeDarkening", 100.0),
    ("waterPerDab", 1.0),
    ("capillary", 0.2),
    ("glaze", 0.0),
    ("pickup", 0.005),
    ("levelingOwnStroke", 0.0),
];
/// `BASE` e mais três.
const fn com(
    a: (&'static str, f64),
    b: (&'static str, f64),
    c: (&'static str, f64),
) -> [(&'static str, f64); 11] {
    [
        BASE[0], BASE[1], BASE[2], BASE[3], BASE[4], BASE[5], BASE[6], BASE[7], a, b, c,
    ]
}
/// A soma de sempre (o controlo): sem o filme, o arrasto e o tecto de água de omissão.
const SOMA: [(&str, f64); 11] = com(("strokeFilm", 0.0), ("drag", 0.16), ("waterCap", 8.0));
/// O filme, com o arrasto a 0 (medido abaixo) e o tecto de água de omissão (8 — o filme não o lê).
const FILME: [(&str, f64); 11] = com(("strokeFilm", 1.0), ("drag", 0.0), ("waterCap", 8.0));

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
        assert!(
            soma[2] > 0.03,
            "{diametro} px: o controlo já não tem listras ({:.4}) — a régua não vê o defeito",
            soma[2]
        );
        assert!(
            filme[2] < 0.01,
            "{diametro} px: listras {:.4} com o filme",
            filme[2]
        );
        assert!(
            filme[3] > 0.93,
            "{diametro} px: o domo (ombros ÷ centro {:.3}) com o filme",
            filme[3]
        );
        assert!(
            filme[4] == 0.0,
            "{diametro} px: buracos {:.3} no miolo do filme",
            filme[4]
        );
        assert!(
            filme[5] < 0.02,
            "{diametro} px: manchas ao longo do traço ({:.3}) com o filme",
            filme[5]
        );
    }
}

/// ⭐ **E a água não o desfaz** (`Knob::StrokeFilm`): depois de 240 quadros o filme do Wet Liner a 200 px continua um
/// véu — sem o 2.º gole da célula virgem (que dependia de quanto a 1.ª janela lhe trouxe e deixava um ARCO de água por
/// janela, medido: mancha 0,095 → 0,038 no Mop) e sem o tecto de água da janela (o excesso saía pela média dela aos
/// bocados, e a 1.ª janela pousava o seu inteiro: o disco do 1.º toque, que o revisor cego viu no Liner a 200 px).
#[test]
fn o_filme_do_traco_continua_liso_depois_da_agua() {
    // E molhado (a água por dab no tecto da faixa: o filme passa o tecto de água da célula, como o Mop).
    let mut molhado = FILME.to_vec();
    molhado.push(("waterPerDab", 4.0));
    let (_, agua) = deposito_e_agua("Wet Liner", &molhado, 200.0, 440.0);
    println!("[filme] 200 px molhado depois de 240 quadros: {agua:.3?}");
    // (0,038 com a lei; 0,078 com o tecto da janela de volta — o disco do 1.º toque e os arcos da água.)
    assert!(
        agua[3] > 0.93 && agua[5] < 0.055,
        "molhado: ombros ÷ centro {:.3}, manchas {:.3}",
        agua[3],
        agua[5]
    );
    let (_, agua) = deposito_e_agua("Wet Liner", &FILME, 200.0, 440.0);
    println!("[filme] 200 px depois de 240 quadros: {agua:.3?}");
    // (A água nivela o que sobra das listras também na soma — aqui elas são só a régua do ruído: 0,009–0,011.)
    assert!(agua[2] < 0.015, "listras {:.4} depois da água", agua[2]);
    assert!(
        agua[3] > 0.93,
        "o domo (ombros ÷ centro {:.3}) depois da água",
        agua[3]
    );
    assert!(
        agua[5] < 0.03,
        "manchas ao longo do traço ({:.3}) depois da água",
        agua[5]
    );
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
        assert!(
            (m / largo - 1.0).abs() < 0.1,
            "a {d} px o filme tem {:.2} do de 200",
            m / largo
        );
    }
}

/// ⭐ **O filme não tem o disco do 1.º toque** (`Knob::StrokeFilm`): a água e o pigmento do início do recto de 190 px são
/// os do meio. Com o peso por comprimento (o pigmento ÷ o passo a que foi afinado) o 1.º dab da lane pesa 1 e os
/// seguintes 4 ÷ 4,775 — com a soma é um dab entre 48, com o máximo é o filme do disco inteiro (1,19×: o revisor cego viu-o).
#[test]
fn o_filme_nao_tem_o_disco_do_primeiro_toque() {
    let p = pinceis("Wet Liner").remove(0);
    let Fonte::Preset { preset, .. } = &p.fonte else {
        unreachable!()
    };
    let mut pr = (**preset).clone();
    for (k, v) in FILME {
        pr.parametros.engine.insert(k.to_owned(), v);
    }
    let mut t = tela();
    t.set_semente_dos_tracos(0);
    t.set_wet_relogio_fixo(true);
    assert!(t.preset_aplicar(&pr).expect("meio").is_empty());
    t.set_brush_size_px(95.0);
    t.set_brush_color_srgb8(AZUL);
    gesto(&mut t, &[[60.0, 320.0], [940.0, 320.0]], 3.0, |_| 1.0);
    let mut faixa = |x0: f64, x1: f64| -> f64 {
        let mut s = 0.0;
        t.wet_sonda_motor(|e| {
            let g = e.active_grid();
            let razao = 1000.0 / (g.w as f64);
            let cel = |v: f64| (v / razao) as usize + 1;
            let (mut n, mut soma) = (0.0, 0.0);
            for cy in cel(310.0)..=cel(330.0) {
                for cx in cel(x0)..=cel(x1) {
                    soma += f64::from(g.film[cx + cy * g.s]);
                    n += 1.0;
                }
            }
            s = soma / n;
        });
        s
    };
    let (inicio, meio) = (faixa(60.0, 120.0), faixa(400.0, 600.0));
    println!("[filme] a água no início {inicio:.3} · no meio {meio:.3}");
    assert!(
        (inicio / meio - 1.0).abs() < 0.03,
        "o 1.º toque tem {:.3} da água do meio",
        inicio / meio
    );
}

/// A cena `cena` da bateria (foto A) pintada pelo Wet `nome` com o mapa `engine` sobreposto: os píxeis RGBA e a largura.
fn foto_a(nome: &str, engine: &[(&str, f64)], cena: &str) -> (Vec<u8>, usize) {
    use crate::exame_bancada::{Pincel, bateria, pinta_cena};
    let p = pinceis(nome).remove(0);
    let Fonte::Preset { preset, relativo } = p.fonte else {
        unreachable!()
    };
    let mut pr = preset;
    for (k, v) in engine {
        pr.parametros.engine.insert((*k).to_owned(), *v);
    }
    let p = Pincel {
        nome: p.nome,
        fonte: Fonte::Preset {
            preset: pr,
            relativo,
        },
    };
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let c = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == cena)
        .unwrap()
        .clone();
    (pinta_cena(&p, None, &c, tela, papel, 0).1, tela[0] as usize)
}

/// A SUJIDADE que o azul leva da faixa vermelha MOLHADA (cena `sobre`, foto A; `exames/aguarela.md` 3.ª ronda): R − B do
/// miolo de cada vertical logo abaixo da faixa (y 190–280) menos o mesmo acima dela (y 70–110, a ponta limpa), a média
/// dos três. O Rebelle `Liner` −3,2 · `Round` −0,1 · `Flat` −12 (o azul sai limpo).
fn sujidade(px: &[u8], w: usize) -> f64 {
    let rb = |c: usize, y0: usize, y1: usize| {
        let mut s = 0.0;
        for y in y0..y1 {
            for x in c - 5..=c + 5 {
                let i = (y * w + x) * 4;
                s += f64::from(px[i]) - f64::from(px[i + 2]);
            }
        }
        s / ((y1 - y0) * 11) as f64
    };
    [250, 500, 750]
        .iter()
        .map(|&c| rb(c, 190, 280) - rb(c, 70, 110))
        .sum::<f64>()
        / 3.0
}

/// ⭐ **O azul não leva o vermelho molhado** (J4 3.ª ronda, o cruzamento molhado): a ponta da aguarela do Rebelle não se
/// suja — o vertical sai da faixa vermelha molhada tão azul como entrou. O nosso apanhava a tinta debaixo da ponta
/// (`Knob::Pickup`, 0,005 por janela, e a ponta só se limpa a 0,001) e levava-a pelo vertical inteiro: R − B +33. Não é
/// motor: a ponta do Wet Liner de fábrica não apanha (`pickup` 0). CONTROLO: o mesmo com a sujidade de omissão.
#[test]
fn o_azul_nao_leva_o_vermelho_molhado() {
    let [(limpo, w), (sujo, _)] = std::thread::scope(|s| {
        let a = s.spawn(|| foto_a("Wet Liner", &[], "sobre"));
        let b = s.spawn(|| foto_a("Wet Liner", &[("pickup", 0.005)], "sobre"));
        [a.join().unwrap(), b.join().unwrap()]
    });
    let (l, c) = (sujidade(&limpo, w), sujidade(&sujo, w));
    println!(
        "[cruzamento] Wet Liner R−B abaixo da faixa molhada {l:.1} (Rebelle −3,2) · com a ponta que se suja {c:.1}"
    );
    assert!(
        c > 15.0,
        "o controlo (a ponta que se suja) lê {c:.1}: a régua não vê o vermelho levado"
    );
    assert!(
        l < 6.0,
        "o Wet Liner leva o vermelho molhado pelo vertical: R−B {l:.1} (Rebelle −3,2)"
    );
}

/// A ORLA do recto de 40 px (cena `tamanhos`, foto A, y 190, x 300..700): o perfil através (a escuridão média em x) — o
/// máximo nos 6 % de fora de cada lado ÷ o centro (40..60 %). A régua do liso da 3.ª ronda.
fn orla_40(px: &[u8], w: usize) -> f64 {
    let esc = |x: usize, y: usize| {
        let i = (y * w + x) * 4;
        1.0 - (0.2126 * f64::from(px[i])
            + 0.7152 * f64::from(px[i + 1])
            + 0.0722 * f64::from(px[i + 2]))
            / 255.0
    };
    let p: Vec<f64> = (150..230)
        .map(|y| (300..700).map(|x| esc(x, y)).sum::<f64>() / 400.0)
        .collect();
    let mx = p.iter().copied().fold(0.0, f64::max);
    let on: Vec<usize> = (0..p.len()).filter(|&i| p[i] > 0.25 * mx).collect();
    let (a, b) = (on[0], on[on.len() - 1] + 1);
    let f = |t: f64| a + (t * (b - a) as f64).round() as usize;
    let centro = p[f(0.4)..f(0.6)].iter().sum::<f64>() / (f(0.6) - f(0.4)) as f64;
    let fora = p[a..=f(0.06)]
        .iter()
        .chain(&p[f(0.94) - 1..b])
        .copied()
        .fold(0.0, f64::max);
    fora / centro
}

/// ⭐ **O fio escuro da borda** (J4 3.ª ronda): o Liner do Rebelle tem um fio escuro de 1–2 px à volta do traço (orla
/// 1,25 a 40 px). A orla da aguarela nasce na borda que SECA antes do miolo (`drying.rs`, a evaporação pela borda) — e o
/// filme encharcava: 3,5 de água por célula (~1 sem ele), aos 240 quadros nada secara e a orla lia 1,00–1,04. Não é motor:
/// água por dab 0,15 (e capilaridade 0,05, a mancha da água pouca) — orla 1,18. CONTROLO: a água por dab de omissão.
#[test]
fn o_liner_tem_o_fio_escuro_da_borda() {
    let [(fio, w), (sem, _)] = std::thread::scope(|s| {
        let a = s.spawn(|| foto_a("Wet Liner", &[], "tamanhos"));
        let b = s.spawn(|| {
            foto_a(
                "Wet Liner",
                &[("waterPerDab", 1.0), ("capillary", 0.2)],
                "tamanhos",
            )
        });
        [a.join().unwrap(), b.join().unwrap()]
    });
    let (o, c) = (orla_40(&fio, w), orla_40(&sem, w));
    println!("[orla] Wet Liner a 40 px {o:.3} (Rebelle 1,247) · com a água de omissão {c:.3}");
    assert!(
        c < 1.06,
        "o controlo (o filme encharcado) já tem orla {c:.3}: a régua não vê o fio"
    );
    assert!(
        o > 1.12,
        "o Wet Liner sem o fio escuro da borda: orla {o:.3} (Rebelle 1,247)"
    );
}
