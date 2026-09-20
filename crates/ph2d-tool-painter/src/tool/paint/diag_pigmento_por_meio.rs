//! **Em que MEIOS a lei do pigmento chega ao barro?** — a medição que decide onde o controlo
//! `Pigment` pode ser oferecido, e onde oferecê-lo seria um knob morto.
//!
//! Ela existe por causa da ordem do dono de 2026-09-20 (*«ligue o digital»*): a lei de mistura
//! subtractiva deixou de ser exclusiva da aquarela, e [`BrushSpec::effective_pigment_mix`] passou a
//! responder ao `pigment` sozinho. ⚠️ **Isso alarga o ALCANCE da lei para todo meio que componha um
//! dab pela porta [`ph2d_painter_brush::blend::blend_over_pigment`]**, e a casa já pagou duas vezes
//! por não medir a fronteira antes de decidir quem vê o botão:
//!
//! * um knob que o motor LÊ e o painel não pinta é um **INALCANÇÁVEL** (cura: ligar);
//! * um knob que o painel pinta e o motor não lê é um **MORTO** (cura: esconder).
//!
//! *As duas leem-se igual numa tabela de risco; só a medição as separa.* Esta sonda mede o barro
//! pela porta do PRODUTO (`set_brush_pigment_mixing`, que é o que o slider chama) e imprime, por
//! meio, a cor do pixel de sobreposição com a mistura desligada e ligada.
//!
//! ⚠️ **A fixtura é AZUL sobre AMARELO** de propósito: é o par em que a lei subtractiva e a aditiva
//! divergem mais (K–M dá verde, a média dá cinzento), logo um meio que não a leia devolve o MESMO
//! byte nas duas colunas.

use super::*;

const SIZE: u32 = 64;
const RADIO: f32 = 12.0;

/// Uma tela branca no meio pedido, com o pincel a **MEIA força**.
///
/// ⛔⛔ **A força parcial é o que faz a fixtura conter o fenómeno, e a 1.ª redacção não a tinha.**
/// Com cobertura cheia (`a = 1`) o [`blend_over_pigment`] devolve a cor de cima *seja qual for a
/// lei* — as duas colunas liam `26,64,217` nos quatro meios e a sonda teria concluído que a lei não
/// chega a lado nenhum. A mistura só é observável onde há o que misturar.
fn tela(media: PaintMedia) -> PainterTool {
    let mut t = white_canvas(SIZE, RADIO);
    t.paint.brush.strength = 0.5;
    t.paint.brush_by_mode.iter_mut().for_each(|b| {
        b.strength = 0.5;
    });
    t.set_paint_media(media);
    t
}

/// Um traço horizontal curto pelo centro, com a cor dada, seguido dos tiques que os meios com
/// relógio próprio (aquarela, fluido) precisam para assentar o depósito.
fn traco(t: &mut PainterTool, cor: [f32; 3]) {
    t.paint.brush.color = cor;
    t.paint.brush_by_mode.iter_mut().for_each(|b| b.color = cor);
    t.on_canvas_pointer(cp([20.0, 32.0], PointerPhase::Down));
    for x in (22..=44).step_by(2) {
        #[allow(clippy::cast_precision_loss)]
        t.on_canvas_pointer(cp([x as f32, 32.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([44.0, 32.0], PointerPhase::Up));
    for _ in 0..30 {
        frame(t);
    }
}

/// Amarelo por baixo, azul por cima — e devolve o pixel do meio da sobreposição.
fn azul_sobre_amarelo(media: PaintMedia, pigmento: f32) -> [u8; 4] {
    let mut t = tela(media);
    traco(&mut t, [0.98, 0.90, 0.10]);
    t.set_brush_pigment_mixing(pigmento);
    traco(&mut t, [0.10, 0.25, 0.85]);
    px(&t, SIZE, 32, 32)
}

/// A distância entre o canal mais alto e o mais baixo — *quão longe do cinzento* a cor está.
fn espalhamento(c: [u8; 4]) -> u32 {
    u32::from(c[..3].iter().copied().max().unwrap_or(0))
        - u32::from(c[..3].iter().copied().min().unwrap_or(0))
}

/// A soma dos três canais — o substituto de luminância que basta para dizer *mais escuro*.
fn brilho(c: [u8; 4]) -> u32 {
    c[..3].iter().map(|&v| u32::from(v)).sum()
}

/// ⭐⭐⭐ **O DIGITAL MISTURA COMO TINTA** — a ordem do dono de 2026-09-20 (*«ligue o digital»*),
/// medida pela porta do PRODUTO (`set_brush_pigment_mixing`, que é o que o slider chama).
///
/// A fixtura é o caso-bandeira: **azul sobre amarelo**, que são complementares. Com a lei aditiva
/// as duas tintas somam para um CINZENTO (`197,203,203` — os três canais a seis pontos um do
/// outro); com a lei do pigmento as absorvências SOMAM e o resultado é escuro e saturado
/// (`55,111,202`).
///
/// ⚠️ **As três metades, e cada uma existe por uma razão:**
///
/// 1. o **CONTROLO** de que a fixtura contém o fenómeno — sem o knob a saída TEM de ser cinzenta;
///    se um dia ela deixar de o ser, a metade seguinte passa a afirmar outra coisa;
/// 2. a **saturação** — é ela que distingue *misturar tinta* de *sobrepor luz*;
/// 3. o **escurecimento** — a consequência do Kubelka–Munk que o dono julgou no smoke da aguada, e
///    a razão pela qual esta ordem esperou pelo veredito dele antes de alcançar um terceiro meio.
///
/// ⛔ Antes de 2026-09-20 este teste era **impossível de escrever**: o `effective_pigment_mix`
/// devolvia `0` fora da aguada, logo as duas colunas liam o mesmo byte.
#[test]
fn o_pigmento_mistura_como_tinta_no_digital() {
    let sem = azul_sobre_amarelo(PaintMedia::Digital, 0.0);
    let com = azul_sobre_amarelo(PaintMedia::Digital, 1.0);
    assert!(
        espalhamento(sem) <= 20,
        "CONTROLO: sem pigmento, azul sobre amarelo tem de somar para um CINZENTO — leu {sem:?} \
         (espalhamento {}). Se a fixtura deixou de ser complementar, este gate deixou de medir a \
         diferença entre as duas leis.",
        espalhamento(sem)
    );
    assert!(
        espalhamento(com) >= 100,
        "com o Pigment a 1 a mistura tem de ser SATURADA (as absorvências somam), e leu {com:?} \
         com espalhamento {} — a lei não chegou ao meio Digital",
        espalhamento(com)
    );
    assert!(
        brilho(com) + 120 < brilho(sem),
        "a mistura subtractiva tem de ser mais ESCURA que a aditiva: {com:?} contra {sem:?}"
    );
    // ⛔⛔ **A quarta metade é a que impede o knob de voltar a ser INALCANÇÁVEL, e ela tinha de
    //    estar AQUI.** O gate do painel compara a TELA com a `offers_pigment_mixing` — logo, se
    //    alguém puser a porta a `false` para este meio, *os dois lados passam a concordar* e ele
    //    fica verde sobre uma lei viva sem botão nenhum. Só quem MEDE o barro pode amarrar a porta.
    assert!(
        PaintMedia::Digital.offers_pigment_mixing(),
        "o barro do Digital move-se com este knob, logo o painel TEM de oferecer a fileira"
    );
}

/// ⭐⭐ **O IMPASTO TAMBÉM A SENTE — e é por isso que ele recebe a fileira.**
///
/// ⛔ Ele compõe o dab pela MESMA porta que o Digital (`blend_over_pigment`); o corpo da tinta é um
/// plano de altura por cima. *Um meio que lê a lei e não vê o botão é um knob INALCANÇÁVEL*, e sem
/// este gate a única coisa que impediria esse estado era eu lembrar-me dele.
#[test]
fn o_pigmento_mistura_como_tinta_no_impasto() {
    let sem = azul_sobre_amarelo(PaintMedia::Impasto, 0.0);
    let com = azul_sobre_amarelo(PaintMedia::Impasto, 1.0);
    assert!(
        espalhamento(com) >= 100 && brilho(com) < brilho(sem),
        "o Impasto lê a mesma porta de composite que o Digital: {com:?} contra {sem:?}"
    );
    assert!(
        PaintMedia::Impasto.offers_pigment_mixing(),
        "o barro do Impasto move-se com este knob, logo o painel TEM de oferecer a fileira"
    );
}

/// ⛔⛔ **O WET PAINT NÃO A SENTE, e a ausência da fileira ali é uma DECISÃO MEDIDA.**
///
/// O depósito dele é do solver de fluido, que tem o **Kubelka–Munk próprio**
/// (`ph2d_wet_paint::ColorMix::Km`) e o slider de pigmento por dab dele. Oferecer esta fileira ali
/// seria um segundo controlo sobre a mesma pergunta — *e ele seria inerte*, que é o que este gate
/// afirma ao bit.
///
/// ⚠️ Ele é o par da `PaintMedia::offers_pigment_mixing`: no dia em que alguém a alargar ao Wet
/// Paint «por simetria», é aqui que a medição responde.
#[test]
fn o_wet_paint_nao_sente_o_pigmento_e_por_isso_nao_o_oferece() {
    let sem = azul_sobre_amarelo(PaintMedia::WetPaint, 0.0);
    let com = azul_sobre_amarelo(PaintMedia::WetPaint, 1.0);
    assert_eq!(
        sem, com,
        "o depósito do Wet Paint é do solver e ignora este knob — se passou a lê-lo, a \
         `offers_pigment_mixing` tem de o dizer, senão o controlo fica inalcançável"
    );
    assert!(
        !PaintMedia::WetPaint.offers_pigment_mixing(),
        "a porta do painel tem de concordar com a medição acima"
    );
}

/// ⚠️ **O PREÇO de ligar o `Pigment` no Digital** — a coluna que o dono precisa de ver antes de o
/// pôr no pincel de todos os dias.
///
/// ⛔⛔ **Ela mede o caminho da CPU, e esse não é o preço inteiro:** o `stamp_device::eligible`
/// exige `pigment_mix == 0`, logo ligar o knob **desliga o carimbo no dispositivo** e o traço volta
/// à rota em banda. Aqui não há ponte de device instalada, então as duas colunas já correm na CPU —
/// *o que esta tabela mede é a LEI; a queda do caminho rápido soma-se a ela no app*.
///
/// ⚠️ Corra em `--release`: em debug esta crate lê cerca de uma ordem de grandeza mais lenta, e a
/// RAZÃO é o que interessa.
///
/// **Medido 2026-09-20** (`--release`, mediana de 96 traços de 24 eventos, **89 % de CPU ociosa**):
/// `0,075 ms` sem contra `0,130 ms` com ⇒ **`1,73×`**. ⭐ O absoluto é ruído (bem abaixo de `1 %` de
/// um quadro): *a lei é barata e quem pode custar é o caminho rápido que ela fecha*.
#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn diag_o_preco_do_pigmento_no_digital() {
    use std::time::Instant;
    const N: u32 = 96;

    let traco_ms = |pigmento: f32| -> f64 {
        let mut t = tela(PaintMedia::Digital);
        t.set_brush_pigment_mixing(pigmento);
        t.paint.brush.color = [0.10, 0.25, 0.85];
        t.paint
            .brush_by_mode
            .iter_mut()
            .for_each(|b| b.color = [0.10, 0.25, 0.85]);
        let t0 = Instant::now();
        t.on_canvas_pointer(cp([8.0, 32.0], PointerPhase::Down));
        for x in (10..=56).step_by(2) {
            #[allow(clippy::cast_precision_loss)]
            t.on_canvas_pointer(cp([x as f32, 32.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([56.0, 32.0], PointerPhase::Up));
        t0.elapsed().as_secs_f64() * 1e3
    };

    let mediana = |pigmento: f32| -> f64 {
        let mut v: Vec<f64> = (0..N).map(|_| traco_ms(pigmento)).collect();
        v.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        v[v.len() / 2]
    };

    let sem = mediana(0.0);
    let com = mediana(1.0);
    println!("\n=== O PRECO DO PIGMENTO NO DIGITAL (traco de 24 eventos, mediana de {N}) ===\n");
    println!("sem pigmento   {sem:8.3} ms");
    println!("com pigmento   {com:8.3} ms   ({:.2}x)", com / sem);
    println!();
}

#[test]
#[ignore = "sonda de estudo; roda sob demanda"]
fn diag_o_pigmento_por_meio() {
    println!("\n=== AZUL SOBRE AMARELO, POR MEIO (pigmento 0 contra 1) ===\n");
    println!("{:<12} {:>18} {:>18} {:>8}", "meio", "sem", "com", "|Δ|max");
    for media in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        let sem = azul_sobre_amarelo(media, 0.0);
        let com = azul_sobre_amarelo(media, 1.0);
        let d = (0..3)
            .map(|i| i32::from(sem[i]).abs_diff(i32::from(com[i])))
            .max()
            .unwrap_or(0);
        println!(
            "{:<12} {:>18} {:>18} {d:>8}",
            format!("{media:?}"),
            format!("{},{},{}", sem[0], sem[1], sem[2]),
            format!("{},{},{}", com[0], com[1], com[2]),
        );
    }
    println!();
}
