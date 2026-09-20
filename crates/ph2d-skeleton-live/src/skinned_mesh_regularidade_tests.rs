//! ⭐⭐⭐ **A REGULARIDADE DO CONTORNO — a régua que o report de 2026-09-20 pediu.**
//!
//! # ⛔⛔⛔ Porque ela existe
//!
//! Report do dono, com foto e oito setas: *«a deformação de vetores com esse algoritmo deixa tudo
//! irregular. A ideia do lattice era tornar o resultado com Vetor similar ao resultado com
//! imagem»*. As setas apontam troços do contorno que ficaram **RECTOS** onde a forma de repouso é
//! curva — faceta, não desvio.
//!
//! ⚠️⚠️ **Nenhuma régua desta linha via isso, e a razão é estrutural.** Elas mediam:
//!
//! | régua | o que responde | porque é cega à faceta |
//! |---|---|---|
//! | `pior_desvio_do_desenho` | *«a que distância estão as duas saídas?»* | um polígono inscrito numa curva lisa tem desvio **pequeno** |
//! | a quina nos nós (`reconcilia`) | *«o nó tem bico?»* | a faceta vive **ENTRE** os nós |
//! | o esticão da aresta de dentro | *«o cotovelo esmaga?»* | é um extremo LOCAL num sítio só |
//!
//! ⇒ *três réguas verdes sobre um contorno que o artista chama de irregular.* A grandeza que falta
//! é **onde a curvatura MORA**: numa curva lisa ela reparte-se pelo contorno; num polígono ela
//! concentra-se em meia dúzia de pontos e é ZERO no meio.

use super::ouro_reguas_tests::*;

// ⛔⛔⛔ **AS DUAS PRIMEIRAS RÉGUAS DESTE FICHEIRO FORAM DEITADAS FORA, e a razão vale a pena.**
//
// Elas mediam o ângulo entre duas amostras CONSECUTIVAS. O caminho é amostrado `32` pontos por
// cúbica **uniformemente em `t`**, e onde as alças encolhem duas amostras ficam a `1e-6` uma da
// outra ⇒ a direcção entre elas é **ruído normalizado**, e um MÁXIMO sobre amostras é dominado por
// esse ruído. A tabela lia `145,8°` de quina numa figura que o desenho mostra **lisa**.
//
// ⚠️⚠️ *É a quarta régua desta jornada a falhar pela mesma forma, e a cura já estava escrita nesta
// crate:* a [`b_menger`] mede a curvatura sobre uma **janela FÍSICA** (`B_H`), logo é imune à
// densidade da amostragem. ⇒ **reusa-se o instrumento vetado em vez de inventar um.**

/// **QUANTO DO CONTORNO ESTÁ RECTO** — a fracção do arco cuja curvatura é desprezável.
///
/// ⚠️ O limiar é **relativo à curvatura média desta própria peça** (`5 %` dela): uma peça grande e
/// uma pequena têm curvaturas diferentes e a mesma forma.
pub(super) fn fraccao_recta(poli: &[[f64; 2]]) -> f64 {
    let k = b_menger(poli, B_H);
    if k.is_empty() {
        return 0.0;
    }
    #[expect(clippy::cast_precision_loss, reason = "contagem de amostras")]
    let media = k.iter().sum::<f64>() / k.len() as f64;
    if media <= 0.0 {
        return 0.0;
    }
    let limiar = media * 0.05;
    #[expect(clippy::cast_precision_loss, reason = "contagem de amostras")]
    let f = k.iter().filter(|c| **c < limiar).count() as f64 / k.len() as f64;
    f
}

/// **A PIOR QUINA do contorno**, em graus — a curvatura máxima lida como ângulo sobre a janela
/// [`B_H`]. É o que o olho lê como *«tem um canto aqui»*.
///
/// ⚠️ **`p99` e não o MÁXIMO** — e a diferença está MEDIDA e é pequena aqui: `155,3°` contra
/// `175,3°` no pior caso, com o **mesmo veredito em todas as células**
/// (`diag_b_as_duas_escolhas_que_a_mutacao_nao_mata`). ⛔ *Uma mutação que troca uma pela outra
/// SOBREVIVE, e isso fica nomeado em vez de escondido.* O `p99` fica porque *o que se vê é um
/// canto e não um ponto* — e porque a 1.ª régua desta família, escrita à mão sobre amostras
/// consecutivas, lia `145,8°` de máximo numa figura lisa.
pub(super) fn pior_quina(poli: &[[f64; 2]]) -> f64 {
    let mut k = b_menger(poli, B_H);
    if k.is_empty() {
        return 0.0;
    }
    k.sort_by(f64::total_cmp);
    #[expect(clippy::cast_precision_loss, reason = "contagem de amostras")]
    let i = ((k.len() - 1) as f64 * 0.99).round() as usize;
    b_quina(k[i], B_H)
}

/// ⭐⭐⭐ **SONDA — O VECTOR CONTRA A IMAGEM, e o que REPARTIR A DOBRA lhes faz.**
///
/// As duas primeiras colunas são a MESMA peça deformada pelas duas mídias: `VECTOR` é o que o
/// produto desenha e `IMAGEM` é a lei da 2.ª mídia sobre o mesmo campo — o *«resultado com
/// imagem»* de que o report fala. A varredura é em `segments`, o *bendy bone* que o painel chama
/// **Curve Handles**.
///
/// ⚠️ **A pose é um S** ([`BPalco::dobra_em_s`]), que é a da cena do dono — num **C** o contorno sai
/// liso e a sonda não contém o fenómeno.
#[test]
fn diag_b_a_regularidade_do_contorno() {
    let mut p = b_palco(true);
    let rest = b_amostra(&p.fonte);
    println!("\n{:=<108}", "");
    println!("SONDA · A REGULARIDADE DO CONTORNO — dobra de 90° em S, repartida por N sub-ossos");
    println!("  «recto» = fracção do contorno com curvatura desprezável · «quina» = o pior canto");
    println!("{:=<108}", "");
    println!(
        "{:>5} {:>6} | {:>10} {:>10} | {:>10} | {:>12}",
        "graus", "sub-os", "quina COM", "quina SEM", "Δ do campo", "vs a IMAGEM"
    );
    for graus in [45.0_f32, 70.0, 90.0] {
        for segs in [1_u8, 2, 4, 8] {
            p.reparte(segs);
            p.dobra_em_s(graus);
            let pele = p.pele();
            let ouro = p.ouro(&pele, &rest);
            let com = b_amostra(&p.produto(true, true));
            let sem = b_amostra(&p.produto(true, false));
            let desvio = com
                .iter()
                .zip(&ouro)
                .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
                .fold(0.0_f64, f64::max);
            let (qc, qs) = (pior_quina(&com), pior_quina(&sem));
            println!(
                "{:>5} {segs:>6} | {qc:>9.1}° {qs:>9.1}° | {:>+9.1}° | {desvio:>12.4}",
                if segs == 1 {
                    format!("{graus:.0}")
                } else {
                    String::new()
                },
                qc - qs
            );
        }
        println!("{:-<70}", "");
    }
    println!("{:-<108}", "");
    println!(
        "  CONTROLO · a forma em REPOUSO: recto {:.1} %, pior quina {:.1}°",
        fraccao_recta(&rest) * 100.0,
        pior_quina(&rest)
    );
    println!(
        "  ⛔ A coluna «quina» da IMAGEM com `segs >= 2` NAO esta' explicada e NAO se cita: o\n\
         \x20    desenho mostra as duas curvas UMA SOBRE A OUTRA, e esta leitura diz o contrario.\n\
         \x20    Suspeita com endereco: o `b_ouro_pt` reparte a tabela do campo por TENDAO, e com\n\
         \x20    sub-ossos `campo.ossos() != pele.len()`. Quem a for medir comeca por ai'."
    );
    println!("{:=<108}", "");
    println!(
        "loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

/// ⭐⭐⭐ **SONDA — O VECTOR E A IMAGEM DESENHADOS UM SOBRE O OUTRO, com os NÓS à vista.**
///
/// ⛔⛔ **Ela vem ANTES de qualquer número nesta família.** Em 2026-09-20 duas tabelas minhas
/// disseram o contrário do que a imagem mostrava, e nos dois casos a imagem tinha razão. A faceta
/// que o dono apontou vive **entre os nós**, logo o desenho tem de os marcar: é a única forma de
/// separar *«a linha está recta entre dois nós»* de *«há um bico NUM nó»*.
///
/// `PH2D_REG_SVG=<ficheiro>`; sem a variável não faz nada.
#[test]
fn diag_b_desenha_o_vector_contra_a_imagem() {
    let Some(saida) = std::env::var_os("PH2D_REG_SVG") else {
        return;
    };
    let mut p = b_palco(true);
    let rest = b_amostra(&p.fonte);
    let mut svg = String::from(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='-11 -9 22 30' width='1400' height='1900'>\n\
         <rect x='-11' y='-9' width='22' height='30' fill='#17171a'/>\n",
    );
    let poli = |v: &[[f64; 2]], dy: f64| {
        let mut s = String::new();
        for (i, q) in v.iter().enumerate() {
            s.push_str(&format!(
                "{}{:.4},{:.4} ",
                if i == 0 { "M " } else { "L " },
                q[0],
                -q[1] + dy
            ));
        }
        s.push('Z');
        s
    };
    // ⭐ As três linhas são a MESMA dobra de 90° em S: o que o produto faz HOJE, o que repartir a
    // dobra compra, e a COMBINAÇÃO (repartir + campo desligado), que é a melhor medida.
    for (k, (segs, campo, rot)) in [
        (1_u8, true, "HOJE (1 sub-osso, campo ligado)"),
        (4, true, "4 sub-ossos, campo ligado"),
        (4, false, "4 sub-ossos, campo DESLIGADO"),
    ]
    .into_iter()
    .enumerate()
    {
        #[expect(clippy::cast_precision_loss, reason = "k <= 2")]
        let dy = k as f64 * 9.0;
        let graus = 90.0_f32;
        p.reparte(segs);
        p.dobra_em_s(graus);
        let pele = p.pele();
        let ouro = p.ouro(&pele, &rest);
        let prod = p.produto(true, campo);
        let vector = b_amostra(&prod);
        svg.push_str(&format!(
            "<text x='-10.6' y='{:.2}' font-size='0.5' fill='#999'>{rot} — quina {:.0}°</text>\n",
            dy - 6.2,
            pior_quina(&b_amostra(&p.produto(true, campo)))
        ));
        svg.push_str(&format!(
            "<path d='{}' fill='none' stroke='#3a3a40' stroke-width='0.04'/>\n",
            poli(&rest, dy)
        ));
        svg.push_str(&format!(
            "<path d='{}' fill='none' stroke='#4ce07a' stroke-width='0.07'/>\n",
            poli(&ouro, dy)
        ));
        svg.push_str(&format!(
            "<path d='{}' fill='none' stroke='#e0a03a' stroke-width='0.05'/>\n",
            poli(&vector, dy)
        ));

        // ⭐ Os NÓS do caminho vectorial — é entre eles que a faceta vive.
        for c in 0..prod.contour_count() {
            let Some((vs, _)) = prod.contour(c) else {
                continue;
            };
            for v in vs {
                svg.push_str(&format!(
                    "<circle cx='{:.4}' cy='{:.4}' r='0.07' fill='#ff4d4d'/>\n",
                    v.anchor[0],
                    -v.anchor[1] + dy
                ));
            }
        }
    }
    svg.push_str(
        "<text x='-10.6' y='19.6' font-size='0.5' fill='#4ce07a'>IMAGEM (a 2.ª mídia — o alvo)</text>\n\
         <text x='-10.6' y='20.3' font-size='0.5' fill='#e0a03a'>VECTOR (o produto)</text>\n\
         <text x='-10.6' y='21.0' font-size='0.5' fill='#ff4d4d'>os NÓS do caminho</text>\n\
         </svg>\n",
    );
    std::fs::write(&saida, svg).expect("escreve o svg");
    println!("svg: {}", saida.to_string_lossy());
}

/// ⭐⭐⭐ **GATE — O QUE FAZ O CANTO QUE O DONO FOTOGRAFOU: o campo ESTREITA a passagem.**
///
/// Report de 2026-09-20, com foto e oito setas: *«a deformação de vetores com esse algoritmo deixa
/// tudo irregular. A ideia do lattice era tornar o resultado com Vetor similar ao resultado com
/// imagem»*.
///
/// ⛔⛔⛔ **A minha 1.ª redacção deste gate afirmava que o lattice NÃO era a causa, e ele reprovou
/// na primeira corrida.** Eu tinha desenhado o vector, a lei da imagem e o caminho com o campo
/// desligado uns sobre os outros, vi três traços coincidentes e concluí que o campo não mexia no
/// canto. À espessura de traço daquele desenho ele não mexia; **medido, mexe `+43,5°`**.
/// *Um desenho responde «são a mesma forma?» e não «qual delas tem o canto mais duro?»* — e foi a
/// segunda metade deste gate, escrita para provar que o campo estava ilibado, que me travou.
///
/// Medido pela porta do produto, dobra de `90°` em S, a quina lida sobre a janela física [`B_H`]:
///
/// | | com o campo | sem o campo |
/// |---|---:|---:|
/// | `1` sub-osso (o que nasce por omissão) | **`155,3°`** | `111,8°` |
/// | `4` sub-ossos (*Curve Handles*) | `61,7°` | **`30,8°`** |
///
/// ⭐⭐ **O MECANISMO está medido e não é um defeito de implementação** (a
/// [`diag_b_porque_o_campo_faz_cantos`]): a passagem de um osso para o outro faz-se em **`0,97`**
/// unidades de contorno com o campo e em **`5,53`** sem ele — **`5,7×` mais estreita**. A forma
/// fechada do vinco é `|1 − θ̄′·r|` e `θ̄′` é a rotação a dividir pela largura em que ela acontece
/// ⇒ *a mesma dobra espremida em um quinto do contorno faz um canto muito mais duro.* Um campo
/// BBW é **localizado por construção** — é isso que o torna correcto, e é isso que vinca.
///
/// ⚠️ **E a hipótese ÓBVIA caiu antes desta**: eu esperava vincos no peso (o campo é linear por
/// triângulo, logo a derivada dele salta em cada aresta). Medida a 2.ª diferença do peso ao longo
/// do contorno, o campo é **mais liso** que a euclidiana (`9,7e-14` contra `5,0e-5` no p50).
#[test]
fn o_campo_estreita_a_passagem_e_e_isso_que_faz_o_canto() {
    let mut p = b_palco(true);
    let q = |p: &BPalco, campo: bool| pior_quina(&b_amostra(&p.produto(true, campo)));

    p.reparte(1);
    p.dobra_em_s(90.0);
    let (um_com, um_sem) = (q(&p, true), q(&p, false));
    p.reparte(4);
    p.dobra_em_s(90.0);
    let (q4_com, q4_sem) = (q(&p, true), q(&p, false));
    println!(
        "  1 sub-osso: {um_com:.1}° com campo · {um_sem:.1}° sem · \
         4 sub-ossos: {q4_com:.1}° com · {q4_sem:.1}° sem"
    );

    // (1) A fixtura contém o fenómeno que o dono fotografou.
    assert!(
        um_com > 140.0,
        "com um sub-osso e o campo ligado o contorno devia ter a quina do report (~155°) \
         e leu {um_com:.1}° — esta fixtura deixou de conter o fenómeno"
    );
    // (2) ⛔ O CAMPO PIORA O CANTO. É a metade que reprovou a minha leitura do desenho.
    assert!(
        um_com > um_sem + 25.0,
        "o campo devia ENDURECER o canto em dezenas de graus e foi de {um_sem:.1}° para \
         {um_com:.1}° — se isto deixar de ser verdade, a prosa acima tem de ser reescrita"
    );
    // (3) Repartir a dobra corta a quina para menos de metade, com campo e sem ele.
    assert!(
        q4_com < um_com * 0.5 && q4_sem < um_sem * 0.5,
        "repartir por 4 sub-ossos devia cortar a quina para menos de metade nas DUAS colunas \
         ({um_com:.1}→{q4_com:.1} com campo, {um_sem:.1}→{q4_sem:.1} sem)"
    );
    // (4) ⭐ E o melhor de todos é a COMBINAÇÃO, que é o que se responde ao dono.
    // ⚠️ ⛔ **DUAS mutações desta família SOBREVIVEM de propósito, com a medição ao lado:** trocar
    // o `p99` pelo MÁXIMO na [`pior_quina`] (mesmo veredito em todas as células) e trocar a dobra
    // em S pela dobra em C (`155,3°` contra `158,7°` — a pose do dono é fidelidade à cena dele,
    // não o que faz o canto). Ver `diag_b_as_duas_escolhas_que_a_mutacao_nao_mata`.
    assert!(
        q4_sem < um_com * 0.25,
        "a combinação (repartir + campo desligado) devia ser a melhor de longe e leu \
         {q4_sem:.1}° contra os {um_com:.1}° de hoje"
    );
}

/// ⭐⭐⭐ **SONDA — PORQUÊ: o PERFIL DO PESO ao longo do contorno, campo contra euclidiana.**
///
/// A tabela da [`diag_b_a_regularidade_do_contorno`] diz que o campo **piora** a quina em todas as
/// células. Esta pergunta *porquê*, e a hipótese tem endereço: o [`ph2d_vec_skin::pesos`] resolve
/// os pesos numa **MALHA DE TRIÂNGULOS** e o valor num ponto é a interpolação **LINEAR** dentro do
/// triângulo que o contém ⇒ o campo é contínuo e **a derivada dele SALTA em cada aresta**. A lei
/// euclidiana é lisa por construção.
///
/// Um peso com derivada aos saltos multiplica uma rotação ⇒ o contorno herda um canto por aresta
/// atravessada. **A régua é a 2.ª diferença do peso ao longo do contorno**, que é zero num campo
/// linear e dispara num vinco.
#[test]
fn diag_b_porque_o_campo_faz_cantos() {
    let mut p = b_palco(true);
    let rest = b_amostra(&p.fonte);
    p.reparte(1);
    p.dobra_em_s(90.0);
    let pele = p.pele();
    let correcoes = p.correcoes.clone();

    println!("\n{:=<96}", "");
    println!("SONDA · PORQUE o campo faz cantos — a 2.ª diferença do PESO ao longo do contorno");
    println!(
        "  um campo LISO tem 2.ª diferença ~0 · um campo linear por TRIÂNGULO salta na aresta"
    );
    println!("{:=<96}", "");
    println!(
        "{:<28} | {:>10} {:>10} {:>10} | {:>12}",
        "lei do peso", "p50", "p90", "MÁX", "saltos"
    );
    for (rot, usa_campo) in [
        ("CAMPO (o lattice)", true),
        ("EUCLIDIANA (o controlo)", false),
    ] {
        // O peso do 2.º tendão em cada amostra do contorno.
        let perfil: Vec<f64> = rest
            .iter()
            .map(|&x| {
                let mut w = pele.scratch();
                if usa_campo {
                    let linha = p
                        .campo
                        .linha(x)
                        .unwrap_or_else(|| b_mais_proximo(&p.campo, x));
                    pele.weights_corrected(x, Some(&linha), &mut w, &correcoes);
                } else {
                    pele.weights_corrected(x, None, &mut w, &correcoes);
                }
                w.get(1).copied().unwrap_or(0.0)
            })
            .collect();
        let n = perfil.len();
        let mut d2: Vec<f64> = (0..n)
            .map(|i| {
                let (a, b, c) = (perfil[(i + n - 1) % n], perfil[i], perfil[(i + 1) % n]);
                (a - 2.0 * b + c).abs()
            })
            .collect();
        // ⚠️ O «salto» é relativo à excursão do próprio perfil — um peso que mal varia não pode
        // ter um vinco grande em absoluto, e compará-los cruamente leria a lei mais suave como
        // mais lisa só por ser mais fraca.
        let faixa = perfil.iter().copied().fold(f64::MIN, f64::max)
            - perfil.iter().copied().fold(f64::MAX, f64::min);
        let saltos = d2.iter().filter(|v| **v > faixa * 0.01).count();
        let (p50, p90, max) = b_pct(&mut d2);
        println!("{rot:<28} | {p50:>10.2e} {p90:>10.2e} {max:>10.2e} | {saltos:>6} / {n}");
        // ⭐⭐⭐ **A LARGURA DA TRANSIÇÃO** — quanto contorno o peso leva a passar de `0,1` a `0,9`.
        // É ela que a forma fechada do vinco aponta: o esticão da aresta de dentro vale
        // `|1 − θ̄′·r|`, e `θ̄′` é a rotação a dividir pela largura em que ela acontece. *Uma
        // transição mais ESTREITA concentra a mesma dobra em menos contorno e faz um canto mais
        // duro* — e um campo BBW é mais localizado que uma queda euclidiana por construção.
        let cum = b_cum(&rest);
        let mut larguras = Vec::new();
        let (mut entrou, mut s0) = (false, 0.0_f64);
        for i in 0..n {
            let v = perfil[i];
            if !entrou && v > 0.1 && v < 0.9 {
                entrou = true;
                s0 = cum[i];
            } else if entrou && !(0.1..=0.9).contains(&v) {
                entrou = false;
                larguras.push(cum[i] - s0);
            }
        }
        larguras.sort_by(f64::total_cmp);
        println!(
            "{:<28} | transição 0,1→0,9: {} troço(s), a mais estreita {:.4} u de contorno",
            "",
            larguras.len(),
            larguras.first().copied().unwrap_or(f64::NAN)
        );
    }
    println!("{:=<96}", "");
}

/// ⚠️ **SONDA — as duas escolhas desta família que uma mutação NÃO mata.**
///
/// A prova de mutação do [`o_campo_estreita_a_passagem_e_e_isso_que_faz_o_canto`] teve duas
/// sobreviventes, e em vez de as esconder mede-se o que elas valem:
///
/// 1. **`p99` contra o MÁXIMO** na [`pior_quina`];
/// 2. **a dobra em S contra a dobra em C** na fixtura.
#[test]
fn diag_b_as_duas_escolhas_que_a_mutacao_nao_mata() {
    let mut p = b_palco(true);
    let max = |poli: &[[f64; 2]]| {
        let mut k = b_menger(poli, B_H);
        k.sort_by(f64::total_cmp);
        b_quina(*k.last().expect("amostras"), B_H)
    };
    println!("\n{:=<86}", "");
    println!("SONDA · as duas escolhas que a mutação não mata, medidas");
    println!("{:=<86}", "");
    println!(
        "{:<28} | {:>10} {:>10} | {:>10} {:>10}",
        "caso", "p99 COM", "MÁX COM", "p99 SEM", "MÁX SEM"
    );
    for (rot, em_s, segs) in [
        ("S · 1 sub-osso", true, 1_u8),
        ("S · 4 sub-ossos", true, 4),
        ("C · 1 sub-osso", false, 1),
        ("C · 4 sub-ossos", false, 4),
    ] {
        p.reparte(segs);
        if em_s {
            p.dobra_em_s(90.0);
        } else {
            p.dobra(90.0);
        }
        let (com, sem) = (
            b_amostra(&p.produto(true, true)),
            b_amostra(&p.produto(true, false)),
        );
        println!(
            "{rot:<28} | {:>9.1}° {:>9.1}° | {:>9.1}° {:>9.1}°",
            pior_quina(&com),
            max(&com),
            pior_quina(&sem),
            max(&sem)
        );
    }
    println!("{:=<86}", "");
}

/// ⭐⭐⭐ **SONDA — OS BOTÕES QUE O ARTISTA TEM, medidos.**
///
/// ⛔⛔⛔ **Ela existe porque eu mandei o dono carregar numa fileira que não está na tela.** Em
/// 2026-09-20 escrevi-lhe *«ponha Curve Handles em From Chain e suba Segments para 4»*, e ele
/// respondeu com duas fotos do painel: *«não existe Curve Handles ou From Chain em lugar nenhum»*.
/// Ele tem razão — o `handles_row` **não pinta** a fileira com `Segments = 1`, que é o que nasce
/// (a razão está escrita lá e é boa: com um segmento a curvatura é provadamente inerte). ⇒ *os
/// passos estavam na ORDEM errada, e a fileira que ensina a usar o `Segments` só aparece depois
/// de ele já estar usado.*
///
/// ⚠️⚠️ **E a segunda foto mostrou uma fileira que eu não tinha medido:** *Deform By ·
/// **Artwork** | **Bone Reach***. Eu estava a mandá-lo bissectar por `PH2D_SKIN_CAMPO=0`, que é
/// **outra coisa** — aquele corta só a consulta ao campo ENTRE os nós, e o *Bone Reach* troca a
/// lei do peso inteira. *Quase lhe mandei carregar num botão cujo efeito eu nunca tinha medido.*
///
/// ⇒ esta sonda mede **só o que tem botão**, na dobra de `90°` em S.
#[test]
fn diag_b_os_botoes_que_o_artista_tem() {
    let mut p = b_palco(true);
    println!("\n{:=<92}", "");
    println!("SONDA · OS BOTOES QUE O ARTISTA TEM — dobra de 90° em S, a pior quina do contorno");
    println!("{:=<92}", "");
    println!(
        "{:<34} {:>10} | {:>34}",
        "Deform By / Segments", "quina", "onde se carrega"
    );
    for (envelope, segs, corrente, onde) in [
        (false, 1_u8, false, "o que NASCE"),
        (false, 2, false, "so' escrever Segments 2"),
        (false, 4, false, "so' escrever Segments 4"),
        (false, 8, false, "so' escrever Segments 8"),
        (false, 4, true, "Segments 4 + Curve Handles"),
        (false, 8, true, "Segments 8 + Curve Handles"),
        (true, 1, false, "Deform By: Bone Reach"),
        (true, 4, true, "Bone Reach + as duas"),
    ] {
        p.lei_do_peso(envelope);
        p.reparte_com(segs, corrente);
        p.dobra_em_s(90.0);
        let rot = format!(
            "{} · {segs} seg · alcas {}",
            if envelope { "Bone Reach" } else { "Artwork" },
            if corrente && segs > 1 {
                "da corrente"
            } else {
                "manuais"
            }
        );
        println!(
            "{rot:<34} {:>9.1}° | {onde:>34}",
            pior_quina(&b_amostra(&p.produto(true, true)))
        );
    }
    println!("{:=<92}", "");
}

/// ⭐⭐⭐ **GATE — `Segments` SOZINHO NÃO MOVE UM BIT, e a fileira que o torna útil só aparece
/// DEPOIS dele.**
///
/// Report do dono (2026-09-20, duas fotos do painel): *«não existe Curve Handles ou From Chain em
/// lugar nenhum»*. **Ele tem razão**, e a cadeia de causas é esta:
///
/// 1. O `handles_row` do painel **não pinta** a fileira *Curve Handles* enquanto o osso for rígido
///    e `Segments <= 1` — e `1` é o que nasce. A razão está escrita lá e é boa: *com um segmento a
///    curvatura é provadamente inerte, e um segmentado que grava sem mudar um pixel é o painel a
///    mentir.*
/// 2. ⛔⛔⛔ **Mas `Segments` sozinho também não muda um pixel.** Com as alças em `Manual` (o que
///    nasce) e a curvatura em [`ph2d_skeleton::bend::Bend::STRAIGHT`] (idem), o osso é o rígido de
///    sempre **ao bit**, seja qual for o número — e o doc do `Bone::curve` diz isso por escrito.
///
/// ⇒ *o knob que faz alguma coisa está escondido atrás de um knob que não faz nada.* O artista
/// escreve `4`, **não vê diferença nenhuma**, e não tem razão para reparar que nasceu uma fileira
/// por baixo. É a espécie *«aceita e mente»* que o `CLAUDE.md` §5.0 nomeia, com uma volta a mais.
///
/// | o que se carrega | a pior quina |
/// |---|---:|
/// | o que nasce | `155,3°` |
/// | só escrever `Segments` `2`, `4` ou `8` | **`155,3°` — o MESMO** |
/// | `Segments 4` **+** *Curve Handles: From Chain* | **`61,7°`** |
#[test]
fn o_numero_de_segmentos_sozinho_nao_move_um_bit() {
    let mut p = b_palco(true);
    p.lei_do_peso(false);
    p.reparte_com(1, false);
    p.dobra_em_s(90.0);
    let base = b_amostra(&p.produto(true, true));

    // (1) ⛔ Escrever o número sozinho é BYTE-IDÊNTICO — não é «pouco», é nada.
    for segs in [2_u8, 4, 8] {
        p.reparte_com(segs, false);
        p.dobra_em_s(90.0);
        let so_numero = b_amostra(&p.produto(true, true));
        assert_eq!(so_numero.len(), base.len(), "a amostragem mudou de tamanho");
        for (i, (a, b)) in base.iter().zip(&so_numero).enumerate() {
            assert_eq!(
                (a[0].to_bits(), a[1].to_bits()),
                (b[0].to_bits(), b[1].to_bits()),
                "com Segments = {segs} e as alças MANUAIS o ponto {i} mexeu-se — se isto passar a \
                 ser falso, o painel deixou de mentir e esta prosa tem de ser reescrita"
            );
        }
    }

    // (2) ⭐ E com a fileira que só aparece depois, ela vale mais de metade da quina.
    p.reparte_com(4, true);
    p.dobra_em_s(90.0);
    let com_alcas = pior_quina(&b_amostra(&p.produto(true, true)));
    let q0 = pior_quina(&base);
    println!(
        "  quina: o que nasce {q0:.1}° · Segments 4 sozinho {q0:.1}° (ao bit) · com as alças {com_alcas:.1}°"
    );
    assert!(
        com_alcas < q0 * 0.5,
        "as duas juntas deviam cortar a quina para menos de metade e foram de {q0:.1}° para \
         {com_alcas:.1}°"
    );
}

/// ⭐⭐ **GATE — a fileira *Curve Handles* está ESCONDIDA no estado em que o painel nasce.**
///
/// A metade do report que é sobre a TELA: o dono procurou a fileira e ela não estava lá. Este gate
/// lê o guarda do painel pelo texto — *um gate que reimplementasse a condição estaria a afirmar
/// sobre a sua própria cópia dela*.
#[test]
fn a_fileira_das_alcas_nasce_escondida_e_e_isso_que_o_dono_nao_achou() {
    let fonte = include_str!("../../ph2d-panel-skeleton/src/section.rs");
    let guarda = "if osso.is_rigid() && ph2d_skeleton::bend::segments_of(osso.segments) <= 1 {";
    assert!(
        fonte.contains(guarda),
        "o guarda que esconde a fileira `Curve Handles` mudou de forma — o report de 2026-09-20 \
         dependia dele, e a prosa do gate irmão descreve-o"
    );
    // ⚠️ O CONTROLO: a fileira EXISTE (não foi apagada), senão este gate leria a ausência dela
    // como «está escondida» e as duas curas seriam opostas.
    assert!(
        fonte.contains("tr(\"panel.vector.bone.handles\")"),
        "a fileira `Curve Handles` deixou de ser pintada de todo — isso não é «escondida», é \
         AUSENTE, e a cura é outra"
    );
}
