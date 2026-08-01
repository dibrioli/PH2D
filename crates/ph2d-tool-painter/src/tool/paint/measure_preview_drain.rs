//! **O QUE O `painter-dispatch` CUSTA QUANDO NINGUÉM ESTÁ PINTANDO** — irmã de
//! [`super::measure_wetpaint_stamp`], separada por ASSUNTO: lá o sujeito é o
//! carimbo (o que uma entrega de ponteiro custa), aqui é o **dreno do preview**
//! (o que um QUADRO custa com a água correndo e a mão parada).
//!
//! ⚠️ **O log do Enio (2026-08-01) nomeou isto sem divisor:**
//!
//! ```text
//! [frame] total=16.21ms (~62 fps) | painter-dispatch(cpu)=11.80ms
//!   tool-tick: media 3.89ms em 115/120 | stamps: 0 entregas
//!   worker: busy 66% away 18% sleep 16% | TAXA DA AGUA 38.6 Hz
//!   poca: 2.37 M celulas | 7.2 ns/celula
//! ```
//!
//! **11,80 ms de dispatch sem um único carimbo**, e `ns/célula` constante
//! (7,2 contra 7,5 na janela que carimba) ⇒ **não é contenção**. Era 6,90 ms
//! quando a §5.47 o nomeou, então quase dobrou.
//!
//! O `FRAME_PROF_DISPATCH_US` é **um balde só** — a mesma doença que a §5.48
//! curou no carimbo, um sistema adiante. O split existe, mas noutro instrumento
//! (`PH2D_PAINT_PERF`, que divide em `preview`/`panel`/`overlay`/`upload`), e o
//! smoke não o liga. Esta sonda mede a metade que mora no TOOL — o
//! `take_preview_arc`, que é o dreno de CPU e a maior parte da fase `preview` —
//! **antes** de gastar outro smoke pedindo o split.

use super::*;

/// O dreno do preview por QUADRO, com a água correndo e a mão parada.
///
/// ⚠️ A fixture reproduz a janela 3 do log: poça de ~2,4 M células a 4096², um
/// tick por quadro (a água a ~38 Hz), **zero eventos de ponteiro**. O que sai é
/// o custo por quadro do dreno + a ÁREA que ele publica — porque um custo sem
/// o tamanho da coisa que ele move é outro número sem divisor.
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_what_the_preview_drain_costs_with_the_water_running() {
    const FRAMES: u32 = 90;

    println!("\n  O DRENO DO PREVIEW POR QUADRO, AGUA CORRENDO E MAO PARADA (4096x4096)\n");
    println!(
        "    {:>26} {:>12} {:>14} {:>14}",
        "condicao", "dreno ms", "px publicados", "ns/px"
    );

    for (label, wet) in [
        ("poca viva (a janela 3)", true),
        ("tela seca (controle)", false),
    ] {
        let mut t = if wet {
            heavy_puddle()
        } else {
            wetted(4096, 100.0)
        };
        // Aquece: o 1º dreno depois da poça paga o que o produto paga uma vez.
        t.wetpaint_tick(1.0 / 60.0);
        let _ = t.take_preview_arc();

        let mut ms = Vec::with_capacity(FRAMES as usize);
        let mut px_total = 0u64;
        let mut drained = 0u32;
        for _ in 0..FRAMES {
            // ⚠️ **O VÃO DE UM QUADRO É INGREDIENTE DA FIXTURE**, e a 1ª versão
            // desta sonda o esqueceu: sem ele os 90 "quadros" passam em
            // microssegundos, o worker nunca acorda (`IDLE_SLEEP` = 4 ms), o
            // `fresh` é sempre falso, o composite nunca roda e o dreno devolve
            // `None` — a tabela saiu **0,000 ms e 0 px publicados**, medindo
            // uma água parada. É a mesma lição que a sonda irmã do carimbo já
            // carrega escrita, violada um arquivo adiante.
            std::thread::sleep(std::time::Duration::from_micros(16_000));
            t.wetpaint_tick(1.0 / 60.0);
            let t0 = Instant::now();
            let got = t.take_preview_arc();
            ms.push(t0.elapsed().as_secs_f64() * 1e3);
            if got.is_some() {
                drained += 1;
                // A REGIÃO que o dreno de fato publicou — o divisor.
                // ⚠️ `(x, y, w, h)`, nao cantos — ver o fix gemeo no
                // `painter_bridge`. A 1a versao desta sonda reportou 8,26 M px
                // com a formula ERRADA, e esse numero foi publicado num doc.
                if let Some((_, _, bw, bh)) = t.take_preview_upload_bbox() {
                    px_total += u64::from(bw) * u64::from(bh);
                } else {
                    let (w, h) = t.canvas_size();
                    px_total += u64::from(w) * u64::from(h);
                }
            }
        }
        ms.sort_by(f64::total_cmp);
        let p50 = ms[ms.len() / 2];
        // ⚠️ **Por quadro que DRENOU, nunca por quadro do laço** — a água corre a
        // ~38 Hz contra 60 de display, então 36 dos 90 quadros não tinham nada
        // novo a mostrar. Dividir pelos 90 diluía o retângulo em 40% e era o
        // divisor errado no instrumento que existe para consertar divisores.
        let px = px_total / u64::from(drained.max(1));
        // A poça VIVA, pela mesma porta que o worker publica no log — trazendo
        // o motor para casa, que é o que o `Deref` do slot exige.
        t.wet_bring_home();
        let cells = t
            .paint
            .wetpaint
            .session
            .as_mut()
            .map_or(0, |s| s.engine.active_grid().live_span_cells());
        println!(
            "    {label:>26} {p50:>11.3} {px:>13} {:>13.2}   ({drained}/{FRAMES} drenaram, \
             {cells} celulas vivas => o retangulo pede {:.2}x)",
            if px > 0 { p50 * 1e6 / px as f64 } else { 0.0 },
            px as f64 / (cells as f64).max(1.0),
        );
    }
    println!(
        "\n    ⚠️ A RAZAO DESTA FIXTURE E' PESSIMISTA: ela usa tres tracos DIAGONAIS, e a bbox\n    \
         de uma faixa diagonal e' um multiplo enorme da faixa. Medido pela porta do PRODUTO\n    \
         (2026-08-01): 1,98-2,10 M px publicados para 1,44-1,52 M celulas vivas = 1,37x, nao\n    \
         os 4,30x daqui. A poca que o artista faz e' compacta.\n"
    );
    println!(
        "    Leitura: se o dreno da poca viva alcancar os ~11,8 ms do log, o alvo esta no\n    \
         TOOL e a atribuicao fecha aqui. Se ficar muito abaixo, o custo mora nas outras\n    \
         fases do dispatch (panel/overlay/upload), que so o `PH2D_PAINT_PERF` separa —\n    \
         e ai o proximo passo e' o DIVISOR na linha `[frame]`, nao uma hipotese."
    );
}

/// **A BBOX DE UPLOAD É `(x, y, w, h)`, NÃO DOIS CANTOS** — e este gate existe
/// porque eu li errado e publiquei o número num doc.
///
/// ⚠️ **A história, para ninguém repetir.** Esta wave precisava da ÁREA que o
/// dreno publica. `take_preview_upload_bbox` devolve `(x, y, w, h)` — o
/// doc-comment dele diz isso quatro linhas acima da assinatura — e eu computei
/// `(x1 - x0) * (y1 - y0)`, ou seja `(w - x) * (h - y)`.
///
/// **O que torna esse erro perigoso é que ele às vezes dá um número plausível:**
/// na fixture headless os retângulos caem perto da origem, `w > x`, e a sonda
/// reportou **8,26 M px** contra os **8,89 M** verdadeiros — perto o bastante
/// para eu escrever um doc em cima. No PRODUTO os retângulos vivem longe da
/// origem, `w < x`, o `saturating_sub` devolve zero, e o log do smoke veio
/// `0.00 M px publicados em 80 quadros`: **o sítio de contagem disparando e o
/// valor sendo lixo.** Quem pegou foi o instrumento do produto, não a sonda.
///
/// ⚠️ **E os dois arch-gates da wave não podiam pegar isto:** um afirma que o
/// placeholder é IMPRESSO, o outro que a contagem mora no sítio certo — nenhum
/// olha para o VALOR. *Um gate de instrumento que nunca afirma que o número é
/// plausível deixa passar exatamente a classe de erro que ele existe para
/// caçar.*
///
/// Mutação que sangra: qualquer consumidor que troque `w * h` por
/// `(w - x) * (h - y)` — nesta fixture `x > w`, então a leitura errada satura
/// em zero e a asserção de área não-vazia morre.
#[test]
fn the_upload_bbox_is_a_rect_and_a_corner_reading_gives_nonsense() {
    const SIDE: u32 = 2048;
    // ⚠️ **LONGE da origem é o ingrediente da fixture.** Perto dela a leitura
    // errada devolve um número plausível — foi exatamente assim que ela chegou
    // a um doc —, e um gate que não contém o fenômeno é verde sobre o defeito.
    const CX: f32 = 1500.0;
    const CY: f32 = 1500.0;

    let mut t = wetted(SIDE, 40.0);
    t.set_paint_media(PaintMedia::WetPaint);
    t.on_canvas_pointer(cp([CX, CY], PointerPhase::Down));
    for k in 1..=6 {
        t.on_canvas_pointer(cp([CX + 12.0 * k as f32, CY], PointerPhase::Move));
        let _ = t.take_preview_arc();
    }
    t.on_canvas_pointer(cp([CX + 84.0, CY], PointerPhase::Up));
    let _ = t.take_preview_arc();

    let Some((x, y, w, h)) = t.take_preview_upload_bbox() else {
        // `None` = "suba a textura inteira", e aí não há tupla a interpretar: a
        // fixture não contém o fenômeno, e dizer isso é melhor que passar.
        panic!(
            "o dreno nao publicou uma sub-regiao — a fixture nao contem o \
             fenomeno que este gate julga"
        );
    };

    assert!(
        w > 0 && h > 0,
        "a bbox publicada tem area vazia ({w}x{h}) — o dreno diz nao ter tocado \
         nada e mesmo assim pediu um upload parcial"
    );
    assert!(
        x + w <= SIDE && y + h <= SIDE,
        "a bbox ({x},{y},{w},{h}) sai da tela {SIDE}x{SIDE}: se `w`/`h` fossem \
         CANTOS isto seria normal, e e' a leitura que este gate recusa"
    );
    // A prova de que a leitura-por-cantos e' absurda NESTA fixture.
    assert!(
        x > w && y > h,
        "a fixture deixou de ser 'longe da origem' ({x},{y},{w},{h}): a leitura \
         errada voltaria a dar um numero plausivel e o gate perderia os dentes"
    );
}
