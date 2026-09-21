//! DIAGNÓSTICO — **o pincel de cima é coberto pelo de baixo do carimbo SEGUINTE.**
//!
//! Report do dono, 2026-09-20: *«o brush de cima sempre deve ser desenhado por cima do Brush de
//! baixo. Atualmente o brush de baixo cobre o brush de cima do carimbo anterior.»*
//!
//! ⚠️ **A ordem DENTRO de um lote já está certa** — o [`PainterTool::stamp_dabs_composite`] corre
//! `for pos in (0..N).rev()`, ou seja de baixo para cima, sobre a fatia INTEIRA de dabs que aquele
//! lote traz. O que o report descreve acontece **ENTRE lotes**: um traço é entregue em dezenas de
//! eventos de ponteiro, cada um um lote, e o lote `k+1` começa pela camada de BAIXO — que aterra
//! por cima do que a camada de CIMA do lote `k` acabou de pintar.
//!
//! ⚠️⚠️ E ela é grande porque os dabs se sobrepõem quase todos: o vão entre dois dabs é
//! `spacing × diâmetro`, e o `spacing` de fábrica é uma fracção pequena ⇒ *a camada de cima só
//! sobrevive na lasca da frente*.
//!
//! # A régua
//!
//! Duas camadas **Brush** com Strength `1`, a de cima VERMELHA e a de baixo AZUL, num traço recto.
//! Conta-se, na linha central do traço, quantos pixels leem cada cor. A resposta que o dono pede é
//! *vermelho em toda a parte*; o que o motor entrega hoje é a coluna `azul`.
//!
//! ⛔ **A cor é lida no MIOLO da linha** (a fileira `y` do centro) e não numa média: uma média
//! sobre a área mistura a borda anti-serrilhada das duas camadas e lê um roxo que não existe em
//! pixel nenhum.

use super::*;

/// Canvas de trabalho — o mesmo da sonda irmã, para as duas tabelas serem comparáveis.
const SIZE: u32 = 512;
const RAIO: f32 = 24.0;
const Y: f32 = 256.0;
const X0: f32 = 120.0;
const X1: f32 = 400.0;

fn ponteiro(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// O traço, entregue em passos de `passo` px — é o passo que decide quantos LOTES há.
fn traco(t: &mut PainterTool, passo: f32) {
    t.on_canvas_pointer(ponteiro([X0, Y], PointerPhase::Down));
    let mut x = X0;
    while x < X1 {
        x += passo;
        t.on_canvas_pointer(ponteiro([x.min(X1), Y], PointerPhase::Move));
    }
    t.on_canvas_pointer(ponteiro([X1, Y], PointerPhase::Up));
}

/// Duas camadas Brush de cores opostas: topo VERMELHO, fundo AZUL.
fn ferramenta() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    t.paint.brush.radius_px = RAIO;
    t.paint.brush.strength = 1.0;
    t.paint.brush.space_attenuation = false;
    t.paint.composite_enabled = true;
    for pos in 0..crate::tool::paint::composite::N_CAMADAS {
        t.paint.composite[pos].strength = 0.0;
    }
    t.paint.composite[0] = CompositeLayer {
        op: CompositeOp::Brush,
        strength: 1.0,
        color: Some([1.0, 0.0, 0.0]),
        ..CompositeLayer::default()
    };
    t.paint.composite[1] = CompositeLayer {
        op: CompositeOp::Brush,
        strength: 1.0,
        color: Some([0.0, 0.0, 1.0]),
        ..CompositeLayer::default()
    };
    t
}

/// A média de R e de B na linha central — *a régua que diz de que COR o traço ficou*.
///
/// ⚠️ A contagem por baldes não chega: com o passo real quase nenhum pixel é puro, e uma coluna
/// «outro» a `270` de `281` não diz se o que lá está é a cor de cima com uma ponta da de baixo ou
/// o contrário. **A média separa-as num número só.**
fn media(t: &PainterTool) -> (f32, f32) {
    let y = Y as usize;
    let (mut sr, mut sb, mut n) = (0f32, 0f32, 0f32);
    for x in (X0 as usize)..=(X1 as usize) {
        let i = (y * SIZE as usize + x) * 4;
        sr += f32::from(t.canvas_rgba[i]);
        sb += f32::from(t.canvas_rgba[i + 2]);
        n += 1.0;
    }
    (sr / n, sb / n)
}

/// Quantos pixels da linha central leem VERMELHO (a camada de cima) e quantos AZUL (a de baixo).
fn conta(t: &PainterTool) -> (usize, usize, usize) {
    let y = Y as usize;
    let (mut vermelho, mut azul, mut outro) = (0usize, 0usize, 0usize);
    for x in (X0 as usize)..=(X1 as usize) {
        let i = (y * SIZE as usize + x) * 4;
        let px = &t.canvas_rgba[i..i + 3];
        let (r, g, b) = (px[0], px[1], px[2]);
        if r > 200 && g < 60 && b < 60 {
            vermelho += 1;
        } else if b > 200 && r < 60 && g < 60 {
            azul += 1;
        } else if r < 250 || g < 250 || b < 250 {
            outro += 1;
        }
    }
    (vermelho, azul, outro)
}

/// A pilha com `topo` sobre um Brush AZUL de baixo.
fn com_topo(topo: CompositeOp) -> PainterTool {
    let mut t = ferramenta();
    t.paint.composite[0].op = topo;
    t
}

/// A imagem da linha central, em RGBA — a fatia que as duas entregas têm de partilhar.
fn linha(t: &PainterTool) -> Vec<u8> {
    let y = Y as usize;
    let (a, b) = (X0 as usize, X1 as usize);
    t.canvas_rgba[(y * SIZE as usize + a) * 4..=(y * SIZE as usize + b) * 4 + 3].to_vec()
}

/// ⭐⭐⭐ **A RÉGUA UNIVERSAL: o mesmo traço entregue em UM lote e em N lotes tem de dar a MESMA
/// imagem.**
///
/// A ordem das camadas é uma propriedade da PILHA — ela não pode depender da taxa a que o rato
/// entrega os eventos. ⇒ o traço de referência é o de **um lote só** (onde a ordem corre inteira
/// sobre todos os dabs de uma vez, que é o que o dono pede) e o defeito é a distância até ele.
///
/// ⚠️ Ela vale para as QUATRO operações no topo com uma régua só — que é o que o report
/// *«tem que rever para todos»* exige —, e é gateável tal como está.
/// ⭐⭐⭐ **O CONTROLO que a régua relativa NÃO tem: a camada de cima faz alguma coisa?**
///
/// ⛔⛔ A [`diag_a_entrega_em_lotes_muda_a_imagem`] compara a entrega em N lotes com a de UM lote,
/// e lê `igual` para o Erase, o Smear e o Blur. Isso só prova que a taxa do rato não muda o
/// resultado — **não** que o resultado é o que o dono pede. *Uma referência que não contém o
/// fenómeno aprova os dois lados.*
///
/// Aqui mede-se em ABSOLUTO, com o mesmo traço, a camada de cima LIGADA contra DESLIGADA:
///
/// * **Erase** — o alfa da linha central. Ele apaga o azul de baixo ⇒ tem de cair.
/// * **Blur** — o maior salto entre pixels vizinhos ao atravessar a borda do traço. Borrar
///   ESBATE a borda ⇒ o salto tem de descer.
/// * **Smear** — quanto a linha se move na direcção do traço (o desvio em relação ao que o pincel
///   sozinho deixa).
/// ⭐⭐⭐ **O CONTROLO DE CIMA: quanto é que o Blur ISOLADO faz na MESMA borda?**
///
/// ⛔⛔ Sem ele a leitura *«o Blur na pilha mexe `24` de soma»* não diz nada: pode ser que a pilha
/// o esteja a estrangular, ou pode ser que **não haja quase nada para borrar** (o miolo de um
/// traço de cor chapada é um campo constante, e borrar um campo constante é um no-op — só a BORDA
/// tem gradiente). *Uma medição sem o lado que funciona não separa as duas.*
#[test]
#[ignore = "diagnóstico: corre à mão com --nocapture"]
fn diag_o_blur_isolado_na_mesma_borda() {
    let salto = |t: &PainterTool| {
        let x = ((X0 + X1) * 0.5) as usize;
        let mut pior = 0f64;
        for y in (Y as usize - RAIO as usize - 6)..(Y as usize) {
            let i = (y * SIZE as usize + x) * 4;
            let j = ((y + 1) * SIZE as usize + x) * 4;
            let a = f64::from(t.canvas_rgba[i]) + f64::from(t.canvas_rgba[i + 2]);
            let b = f64::from(t.canvas_rgba[j]) + f64::from(t.canvas_rgba[j + 2]);
            pior = pior.max((a - b).abs());
        }
        pior
    };

    // 1) Um traço de pincel, SEM pilha. É a borda que os dois lados vão tentar esbater.
    let mut base = ferramenta();
    base.paint.composite_enabled = false;
    base.paint.brush.color = [0.0, 0.0, 1.0];
    traco(&mut base, 2.0);
    println!("\n  O BLUR NA MESMA BORDA\n");
    println!("  só o pincel ............................ salto {:6.1}", salto(&base));

    // 2) O MESMO traço, agora com a ferramenta Blur ISOLADA a passar por cima.
    let mut isolado = base;
    isolado.paint.paint_mode = PaintMode::Blur;
    traco(&mut isolado, 2.0);
    println!("  + a ferramenta Blur isolada por cima ... salto {:6.1}", salto(&isolado));

    // 3) O MESMO traço, mas com a pilha [Blur(topo), Brush(fundo)] a fazer as duas coisas.
    let mut pilha = com_topo(CompositeOp::Blur);
    traco(&mut pilha, 2.0);
    println!("  a PILHA (Blur sobre Brush) ............. salto {:6.1}\n", salto(&pilha));
}

#[test]
#[ignore = "diagnóstico: corre à mão com --nocapture"]
fn diag_a_camada_de_cima_faz_alguma_coisa() {
    let alfa = |t: &PainterTool| {
        let y = Y as usize;
        let (mut s, mut n) = (0f64, 0f64);
        for x in (X0 as usize)..=(X1 as usize) {
            s += f64::from(t.canvas_rgba[(y * SIZE as usize + x) * 4 + 3]);
            n += 1.0;
        }
        s / n
    };
    // O maior salto de LUMA entre linhas vizinhas, atravessando a borda de cima do traço.
    let salto = |t: &PainterTool| {
        let x = ((X0 + X1) * 0.5) as usize;
        let mut pior = 0f64;
        for y in (Y as usize - RAIO as usize - 6)..(Y as usize) {
            let i = (y * SIZE as usize + x) * 4;
            let j = ((y + 1) * SIZE as usize + x) * 4;
            let a = f64::from(t.canvas_rgba[i]) + f64::from(t.canvas_rgba[i + 2]);
            let b = f64::from(t.canvas_rgba[j]) + f64::from(t.canvas_rgba[j + 2]);
            pior = pior.max((a - b).abs());
        }
        pior
    };

    println!("\n  A CAMADA DE CIMA FAZ ALGUMA COISA? (traço num lote só, topo LIGADO vs DESLIGADO)\n");
    for (nome, topo) in [
        ("Erase", CompositeOp::Erase),
        ("Blur", CompositeOp::Blur),
        ("Smear", CompositeOp::Smear),
    ] {
        let mut ligado = com_topo(topo);
        traco(&mut ligado, X1 - X0);
        let mut desligado = com_topo(topo);
        desligado.paint.composite[0].strength = 0.0;
        traco(&mut desligado, X1 - X0);
        let (a_on, a_off) = (alfa(&ligado), alfa(&desligado));
        let (s_on, s_off) = (salto(&ligado), salto(&desligado));
        let mut dif = 0u32;
        for (p, q) in linha(&ligado).iter().zip(linha(&desligado).iter()) {
            dif += u32::from(p.abs_diff(*q));
        }
        println!(
            "  {nome:6} | alfa {a_off:6.1} -> {a_on:6.1} | salto de borda {s_off:6.1} -> {s_on:6.1} | \
             soma |Δ| da linha {dif:8}"
        );
    }
    println!();
}

#[test]
#[ignore = "diagnóstico: corre à mão com --nocapture"]
fn diag_a_entrega_em_lotes_muda_a_imagem() {
    println!("\n  A ORDEM É DA PILHA, NÃO DA TAXA DO RATO");
    println!("  referência = o MESMO traço entregue num lote só (a ordem corre inteira)\n");
    println!("  topo sobre Brush azul |  passo |  |Δ| médio |  |Δ| pior | veredito");
    println!("  ----------------------+--------+-----------+-----------+---------");
    for (nome, topo) in [
        ("Brush vermelho", CompositeOp::Brush),
        ("Erase", CompositeOp::Erase),
        ("Smear", CompositeOp::Smear),
        ("Blur", CompositeOp::Blur),
    ] {
        let mut r = com_topo(topo);
        traco(&mut r, X1 - X0);
        let referencia = linha(&r);
        for passo in [2.0f32, 8.0] {
            let mut t = com_topo(topo);
            traco(&mut t, passo);
            let viva = linha(&t);
            let n = referencia.len().min(viva.len());
            let mut soma = 0f64;
            let mut pior = 0u8;
            for i in 0..n {
                let d = referencia[i].abs_diff(viva[i]);
                soma += f64::from(d);
                pior = pior.max(d);
            }
            let media = soma / n as f64;
            let veredito = if pior <= 2 { "igual" } else { "DIVERGE" };
            println!("  {nome:21} | {passo:6.0} | {media:9.2} | {pior:9} | {veredito}");
        }
    }
    println!();
}

#[test]
#[ignore = "diagnóstico: corre à mão com --nocapture"]
fn diag_a_camada_de_cima_e_coberta_pela_de_baixo_do_lote_seguinte() {
    println!("\n  A ORDEM ENTRE CARIMBOS — topo VERMELHO sobre fundo AZUL, ambos Strength 1");
    println!("  spacing de fábrica = {:.3}", PainterTool::default().paint.brush.spacing);
    println!("\n  passo |  dabs/lote |  vermelho |    azul |  outro | media da linha  | veredito");
    println!("  ------+------------+-----------+---------+--------+-----------------+---------");
    for passo in [2.0f32, 8.0, 32.0, (X1 - X0)] {
        let mut t = ferramenta();
        traco(&mut t, passo);
        let (v, a, o) = conta(&t);
        let (mr, mb) = media(&t);
        let veredito = if mb < 8.0 { "topo vence" } else { "FUNDO aparece" };
        println!(
            "  {passo:5.0} | {:10.1} | {v:9} | {a:7} | {o:6} | R {mr:6.1} · B {mb:6.1} | {veredito}",
            passo / (0.10 * 2.0 * RAIO)
        );
    }

    // O CONTROLO: uma camada só, para se ver que a régua vê a cor que a camada pinta.
    let mut t = ferramenta();
    t.paint.composite[1].strength = 0.0;
    traco(&mut t, 2.0);
    let (v, a, o) = conta(&t);
    let (mr, mb) = media(&t);
    println!("\n  CONTROLO (só o topo vermelho): vermelho {v} · azul {a} · outro {o} · R {mr:.1} B {mb:.1}");
    let mut t = ferramenta();
    t.paint.composite[0].strength = 0.0;
    traco(&mut t, 2.0);
    let (v, a, o) = conta(&t);
    let (mr, mb) = media(&t);
    println!("  CONTROLO (só o fundo azul):    vermelho {v} · azul {a} · outro {o} · R {mr:.1} B {mb:.1}\n");
}
