//! **O QUE O CARIMBO CUSTA** — irmão de [`super::measure_wetpaint_contention`]
//! (o que a MÁQUINA faz com o passo), separado por RESPONSABILIDADE quando o pai
//! cruzou o teto de LOC.
//!
//! O corte é o assunto: lá o sujeito é a máquina (núcleos, banda, pool); aqui é
//! o **traço** — o que uma entrega de ponteiro custa, do que esse custo é função
//! e o que o artista de fato recebe quando pede um pincel grande.
//!
//! ⚠️ **Três hipóteses minhas morreram aqui e a quarta era um CAP:** o custo é
//! por DAB e não por evento · o handshake é por QUADRO e não por evento · a
//! poça já existente **não custa nada** (0,97-0,99×) · e o que sobrou foi o
//! `TRAIL_HALF`, o teto de raio do modelo JS de referência definindo o teto
//! deste produto (doc 28 §5.51).

use super::*;

/// **O CUSTO DE UM EVENTO DE PONTEIRO É POR DAB OU FIXO POR EVENTO?** — a
/// pergunta que decide a cura do traço lento.
///
/// Medido no produto (2026-07-31): `stamps: media 105,82ms pico 531,42ms em
/// 26/120 frames`, com o custo por MOVE em **2,03 ms** (censo dos quatro
/// meios) ⇒ **~52 eventos por quadro**. É a espiral que o próprio comentário do
/// `painter_canvas_move` descreve — *mais eventos → quadro mais lento → mais
/// eventos* —, e os métodos incrementais estão **isentos do coalescing de
/// propósito**, porque cada evento deposita dabs e engolir eventos engoliria
/// tinta.
///
/// ⚠️ **Mas a isenção só é honesta se o custo for POR DAB.** Um mouse a 1000 Hz
/// sobre um quadro de 95 ms entrega ~50 eventos que somam o MESMO caminho que
/// 5 eventos entregariam — mesma tinta, mesmos dabs. Se o custo por evento for
/// ~constante na distância, ele é **overhead fixo**, o trabalho cresce com a
/// TAXA DE POLLING e não com o traço, e isso contradiz a lei que esta linha já
/// pagou cinco vezes: *o depósito é propriedade do pincel e do CAMINHO, nunca
/// de quão fino o motor amostrou o caminho*.
///
/// A tabela varre a distância do passo com o caminho TOTAL fixo: o mesmo traço,
/// entregue em `n` eventos grandes ou `16n` pequenos. Custo total constante ⇒ é
/// por dab (nada a consertar). Custo total subindo com o número de eventos ⇒ é
/// overhead, e o número é o preço da espiral.
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_whether_a_pointer_event_costs_by_dab_or_by_event() {
    const SIDE: u32 = 4096;
    const PATH_PX: f32 = 640.0;

    println!(
        "\n  O MESMO CAMINHO DE {PATH_PX:.0} px, ENTREGUE EM EVENTOS DE TAMANHOS DIFERENTES\n"
    );
    println!(
        "    {:<14} {:>7} {:>8} {:>12} {:>12} {:>12}",
        "meio", "passo", "eventos", "por evento", "TOTAL ms", "vs 40px"
    );

    for media in [
        PaintMedia::Digital,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        let mut base = 0.0f64;
        for step in [40.0f32, 10.0, 2.5, 1.0] {
            let mut t = wetted(SIDE, 100.0);
            t.set_paint_media(media);
            let (x0, y0) = (300.0f32, 2048.0f32);
            t.on_canvas_pointer(cp([x0, y0], PointerPhase::Down));
            let n = (PATH_PX / step).round() as u32;
            let mut ms = Vec::with_capacity(n as usize);
            // ⚠️ **A sim RODANDO, e é o que torna esta tabela decisiva.** A 1ª
            // versão media com o motor PARADO e concluiu que o custo é do
            // CAMINHO — verdade para o depósito, e insuficiente: no produto
            // cada evento chama `bring_home()` BLOQUEANTE, e isso é custo por
            // EVENTO. Com o motor parado o handshake não existe, então aquela
            // tabela não podia ver a diferença que ela existe para procurar.
            let mut since_tick = Instant::now();
            for k in 1..=n {
                // Um tick por QUADRO, como o produto — nunca por evento.
                if since_tick.elapsed().as_micros() >= 16_000 {
                    t.wetpaint_tick(1.0 / 60.0);
                    since_tick = Instant::now();
                }
                let x = x0 + step * k as f32;
                let t0 = Instant::now();
                t.on_canvas_pointer(cp([x, y0], PointerPhase::Move));
                ms.push(t0.elapsed().as_secs_f64() * 1e3);
                let _ = t.take_preview_arc();
            }
            t.on_canvas_pointer(cp([x0 + PATH_PX, y0], PointerPhase::Up));
            ms.sort_by(f64::total_cmp);
            let per = ms[ms.len() / 2];
            let total: f64 = ms.iter().sum();
            if base == 0.0 {
                base = total;
            }
            println!(
                "    {:<14} {step:>6.1}p {n:>8} {per:>11.3}m {total:>11.2}m {:>11.2}x",
                format!("{media:?}"),
                total / base,
            );
        }
    }
    println!(
        "\n    Leitura: o CAMINHO e o mesmo nas quatro linhas de cada meio, entao a coluna\n    \
         TOTAL deveria ser CONSTANTE — o pincel deposita a mesma tinta. O quanto ela sobe\n    \
         e exatamente o overhead que a taxa de polling do mouse compra, e o combustivel\n    \
         da espiral que o comentario do `painter_canvas_move` descreve."
    );
}

/// **O INSTRUMENTO NÃO PODE CUSTAR O QUE ELE MEDE** — a régua da poça
/// (`Grid::live_span_cells`) contra o passo que ela divide.
///
/// ⚠️ **Escrito porque um smoke sob máquina saturada é indistinguível de uma
/// regressão** (2026-07-31): com `load average 74` em 32 núcleos, a linha
/// *controle* da tabela irmã — mesmo código, mesma fixture — foi de **14,2 para
/// 46,6 ms/passo** sem uma linha mudar, e o produto reportou `130-200
/// ns/célula` contra os 7,5 de uma hora antes. Nesse regime **nenhum número
/// absoluto decide nada**, e a suspeita cai sobre a última coisa que mudou.
///
/// A resposta que sobrevive a máquina carregada é uma **RAZÃO medida na MESMA
/// corrida**: as duas metades sobem juntas, então o quociente é estável. Se a
/// régua vale uma fração desprezível do passo, ela está exonerada — e continua
/// exonerada amanhã, com a máquina noutro estado.
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_that_the_ruler_costs_nothing_against_the_step_it_divides() {
    const REPS: usize = 9;

    let mut t = heavy_puddle();
    t.wet_bring_home();
    let sess = t.paint.wetpaint.session.as_mut().expect("sessao");
    let e = &mut *sess.engine;
    ph2d_wet_paint::solver::rebuild_active_region(e.active_grid_mut());
    let cells = e.active_grid().live_span_cells();
    let snap = ph2d_wet_paint::grid::snapshot_grid(e.active_grid());
    let frame0 = e.sim.frame;

    // A RÉGUA, medida como o worker a chama: uma vez por passo fechado.
    let mut r = Vec::with_capacity(REPS);
    for _ in 0..REPS {
        let t0 = Instant::now();
        let n = e.active_grid().live_span_cells();
        r.push(t0.elapsed().as_secs_f64() * 1e3);
        assert_eq!(n, cells, "a regua e pura");
    }
    // E o PASSO, na mesma corrida e no mesmo estado de máquina.
    let mut s = Vec::with_capacity(REPS);
    for _ in 0..REPS {
        ph2d_wet_paint::grid::restore_grid(e.active_grid_mut(), &snap);
        e.sim.frame = frame0;
        let t0 = Instant::now();
        e.step_simulation();
        s.push(t0.elapsed().as_secs_f64() * 1e3);
    }
    r.sort_by(f64::total_cmp);
    s.sort_by(f64::total_cmp);
    let (ruler, step) = (r[REPS / 2], s[REPS / 2]);
    println!("\n  A REGUA CONTRA O PASSO QUE ELA DIVIDE (mediana de {REPS}, MESMA corrida)\n");
    println!("    poca            {:.2} M celulas", cells as f64 / 1e6);
    println!("    regua           {ruler:9.4} ms");
    println!("    passo           {step:9.4} ms");
    println!(
        "    a regua vale    {:9.4}% do passo",
        100.0 * ruler / step.max(1e-9)
    );
    println!(
        "\n    Leitura: e uma RAZAO, entao ela sobrevive a maquina carregada — as duas\n    \
         metades sobem juntas. Abaixo de ~1% o instrumento esta exonerado, e continua\n    \
         exonerado amanha, com a maquina noutro estado."
    );
}

/// **O CARIMBO COM A SIM RODANDO** — o defeito de fixture que fazia o meu
/// número não bater com o do produto.
///
/// ⚠️ **O censo dos quatro meios mede o carimbo com o motor PARADO** (ele nunca
/// chama o tick, então o worker nunca recebe o engine) e reporta **2,03
/// ms/move** a 4096². O produto reportou `13,12 ms cada` — e mesmo descontando
/// a máquina saturada daquele log, a fixture e o produto não estão medindo a
/// mesma coisa: **no produto o carimbo disputa o motor com o worker.**
///
/// Cada evento de ponteiro chama `WetSession::bring_home()`, que é
/// **BLOQUEANTE e sem timeout** — o irmão `try_bring_home` do tick espera no
/// máximo `TICK_WAIT`, o do carimbo espera o que for preciso. O worker só
/// devolve em **fronteira de estágio**, e a `ESPERA` medida no produto é de
/// **2,3-2,6 ms**: é isso que um estágio custa.
///
/// A tabela mede as duas condições lado a lado, na MESMA corrida (razão imune
/// à carga da máquina), varrendo também o RAIO — porque o custo de um dab é a
/// pegada, e o artista pinta com pincel grande.
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_the_stamp_while_the_sim_is_actually_running() {
    const SIDE: u32 = 4096;
    const MOVES: u32 = 60;
    const STEP_PX: f32 = 12.0;
    /// O vão entre a entrega do motor e o carimbo seguinte — um quadro.
    const GAP_US: u64 = 16_000;

    println!("\n  O CARIMBO COM O MOTOR PARADO x COM O WORKER SIMULANDO ({SIDE}x{SIDE})\n");
    println!(
        "    {:>6} {:>14} {:>14} {:>10}   {:>12}",
        "raio", "parado ms", "simulando ms", "razao", "por evento"
    );

    for radius in [60.0f32, 100.0, 200.0] {
        let mut row = [0.0f64; 2];
        for (slot, live) in [(0usize, false), (1, true)] {
            let mut t = wetted(SIDE, radius);
            t.set_paint_media(PaintMedia::WetPaint);
            let (x0, y0) = (400.0f32, 2048.0f32);
            t.on_canvas_pointer(cp([x0, y0], PointerPhase::Down));
            let mut ms = Vec::with_capacity(MOVES as usize);
            for k in 1..=MOVES {
                // A condição do PRODUTO: um tick por quadro entrega o motor ao
                // worker, e o carimbo seguinte tem de o trazer de volta.
                if live {
                    t.wetpaint_tick(1.0 / 60.0);
                    // ⚠️ **O VÃO é o ingrediente essencial da fixture.** Sem ele o
                    // tick entrega o motor e o carimbo o retoma no instante
                    // seguinte — o worker nunca chega a ENTRAR num estágio, e o
                    // `bring_home` bloqueante nunca bloqueia: a 1ª versão desta
                    // sonda mediu 0,99× e concluiu, erradamente, que o handshake
                    // é grátis. No produto há um quadro inteiro entre a entrega
                    // e o carimbo seguinte, e é nesse vão que o worker está no
                    // meio de um estágio quando o artista mexe o dedo.
                    std::thread::sleep(std::time::Duration::from_micros(GAP_US));
                }
                let x = x0 + STEP_PX * k as f32;
                let t0 = Instant::now();
                t.on_canvas_pointer(cp([x, y0], PointerPhase::Move));
                ms.push(t0.elapsed().as_secs_f64() * 1e3);
                let _ = t.take_preview_arc();
            }
            t.on_canvas_pointer(cp([x0 + STEP_PX * MOVES as f32, y0], PointerPhase::Up));
            row[slot] = ms.iter().sum();
        }
        println!(
            "    {radius:>5.0}p {:>13.2} {:>13.2} {:>9.2}x   {:>11.3}ms",
            row[0],
            row[1],
            row[1] / row[0].max(1e-9),
            row[1] / f64::from(MOVES),
        );
    }
    println!(
        "\n    Leitura: a coluna PARADO e a que o censo dos quatro meios mede; a SIMULANDO\n    \
         e a que o artista sente. A razao entre elas e o preco do handshake — cada evento\n    \
         chama `bring_home()` BLOQUEANTE, e o worker so devolve em fronteira de estagio."
    );
}

/// **O CARIMBO SOBRE UMA POÇA GRANDE** — o ingrediente que faltava, e o log
/// calmo do Enio é quem o nomeia.
///
/// ⚠️ **Este é o primeiro log de traço com a máquina SÃ** (2026-07-31): o
/// `ns/célula` fica em **7,0-7,1 em TODAS as janelas**, inclusive nas que
/// carimbam, então a contenção está descartada por medição. E mesmo assim:
///
/// ```text
/// stamps: media  58,92ms pico 173,07ms em 42/120 (315 entregas,  7,86ms cada)
/// stamps: media 130,89ms pico 212,38ms em  4/120 ( 49 entregas, 10,69ms cada)
/// ```
///
/// **7,86-10,69 ms por entrega**, contra os **0,78 ms** que a sonda irmã mede
/// com a mesma razão de grade e a sim rodando. **10-14×**, e a diferença não é
/// a máquina.
///
/// A fixture da irmã pinta numa folha **limpa**; o artista pinta sobre a poça
/// que ele acabou de fazer — **1,58-1,90 M células vivas** no log. Um dab que
/// cai em água tem outro trabalho pela frente (as raias do trail, o pickup, a
/// tinta que já está lá), e nada na sonda anterior continha isso.
///
/// A tabela mede o MESMO traço nas duas condições, na mesma corrida.
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_the_stamp_landing_on_a_puddle_that_is_already_there() {
    const MOVES: u32 = 40;
    const STEP_PX: f32 = 24.0;

    println!("\n  O CARIMBO NA FOLHA LIMPA x SOBRE A POCA QUE O ARTISTA JA FEZ\n");
    println!(
        "    {:>6} {:>16} {:>16} {:>10}   {:>14}",
        "raio", "folha limpa ms", "sobre a poca ms", "razao", "por entrega"
    );

    for radius in [100.0f32, 200.0, 300.0] {
        let mut row = [0.0f64; 2];
        for (slot, wet) in [(0usize, false), (1, true)] {
            // ⚠️ A poça é construída pela porta do PRODUTO (`heavy_puddle`), a
            // mesma que o worker do log de fato simula — 1,6 M células vivas.
            let mut t = if wet {
                heavy_puddle()
            } else {
                wetted(4096, radius)
            };
            t.set_paint_media(PaintMedia::WetPaint);
            t.set_brush_size_px(radius * 2.0);
            // Um traço NOVO por cima, no meio da poça.
            let (x0, y0) = (900.0f32, 900.0f32);
            t.on_canvas_pointer(cp([x0, y0], PointerPhase::Down));
            let mut ms = Vec::with_capacity(MOVES as usize);
            let mut since_tick = Instant::now();
            for k in 1..=MOVES {
                if since_tick.elapsed().as_micros() >= 16_000 {
                    t.wetpaint_tick(1.0 / 60.0);
                    since_tick = Instant::now();
                }
                let x = x0 + STEP_PX * k as f32;
                let t0 = Instant::now();
                t.on_canvas_pointer(cp([x, y0], PointerPhase::Move));
                ms.push(t0.elapsed().as_secs_f64() * 1e3);
                let _ = t.take_preview_arc();
            }
            t.on_canvas_pointer(cp([x0 + STEP_PX * MOVES as f32, y0], PointerPhase::Up));
            row[slot] = ms.iter().sum();
        }
        println!(
            "    {radius:>5.0}p {:>15.2} {:>15.2} {:>9.2}x   {:>13.3}ms",
            row[0],
            row[1],
            row[1] / row[0].max(1e-9),
            row[1] / f64::from(MOVES),
        );
    }
    println!(
        "\n    Leitura: o produto reporta 7,86-10,69 ms por entrega com a maquina SA. Se a\n    \
         coluna SOBRE A POCA alcancar esse numero, o ingrediente que faltava e a agua que\n    \
         ja esta la — e o alvo e o dab que cai em agua, nao o dab."
    );
}

/// **O QUE O ARTISTA PEDE × O QUE O TRAÇO DEIXA** — o cap que o Enio bate.
///
/// Report (2026-08-01): *"todos esses testes tenho feito com raio 300, mas na
/// prática o app limita o tamanho para aproximadamente 200"*.
///
/// ⚠️ **Um cap se MEDE antes de se escrever** (CLAUDE.md §0), e este tem
/// número herdado: `TRAIL_HALF = 61 // ceil(35 + 4*6) + 2` — a janela do trail
/// é dimensionada para **raio máximo 35 CÉLULAS**, que é o teto do modelo JS de
/// referência. O slider do Painter vai a `BRUSH_SIZE_MAX_PX = 512`.
///
/// Esta sonda não lê constante nenhuma: pinta UM dab por raio pedido e mede a
/// largura que de fato apareceu no canvas. Se a curva satura, o cap é real e o
/// número dele sai daqui — não de uma leitura de fonte.
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_what_the_artist_asks_for_against_what_the_stroke_lays() {
    const SIDE: u32 = 2048;

    println!("\n  O RAIO PEDIDO x O QUE O TRACO DE FATO DEIXA ({SIDE}x{SIDE}, um dab)\n");
    println!(
        "    {:<12} {:>10} {:>14} {:>14} {:>10}",
        "meio", "pedido", "largura px", "raio efetivo", "razao"
    );

    for media in [PaintMedia::Digital, PaintMedia::WetPaint] {
        for want in [25.0f32, 50.0, 100.0, 200.0, 300.0, 400.0] {
            let mut t = wetted(SIDE, want);
            t.set_paint_media(media);
            t.set_brush_size_px(want);
            let (cx, cy) = (1024.0f32, 1024.0f32);
            t.on_canvas_pointer(cp([cx, cy], PointerPhase::Down));
            // Um traço CURTO: o dab e a sua vizinhança, sem somar caminho.
            t.on_canvas_pointer(cp([cx + 2.0, cy], PointerPhase::Move));
            t.on_canvas_pointer(cp([cx + 4.0, cy], PointerPhase::Up));
            for _ in 0..4 {
                t.wetpaint_tick(1.0 / 60.0);
            }
            // A LARGURA que apareceu: varre a linha do centro e conta os texels
            // que deixaram de ser papel. Oráculo de APARÊNCIA, nunca a const.
            let (w, h) = t.canvas_size();
            let px = t.canvas_rgba.clone();
            let mut lo = w as i64;
            let mut hi = -1i64;
            if h > 0 {
                let row = (cy as usize).min(h as usize - 1) * w as usize * 4;
                for x in 0..w as usize {
                    let o = row + x * 4;
                    // O papel de fundo é uniforme; qualquer desvio é tinta.
                    let painted = px[o + 3] > 0
                        && (i32::from(px[o]) - 255).abs()
                            + (i32::from(px[o + 1]) - 255).abs()
                            + (i32::from(px[o + 2]) - 255).abs()
                            > 12;
                    if painted {
                        lo = lo.min(x as i64);
                        hi = hi.max(x as i64);
                    }
                }
            }
            let width = if hi >= lo { (hi - lo + 1) as f64 } else { 0.0 };
            println!(
                "    {:<12} {want:>9.0}p {width:>13.0} {:>13.1} {:>9.2}x",
                format!("{media:?}"),
                width / 2.0,
                (width / 2.0) / f64::from(want),
            );
        }
    }
    println!(
        "\n    Leitura: a coluna RAZAO deveria ser ~1,0 em toda linha. Onde ela cai, o\n    \
         traco parou de crescer com o pedido — e o numero em que ela cai E o cap,\n    \
         medido em vez de lido de uma constante."
    );
}

/// **O PINCEL GRANDE FICOU CARO — E O SLIDER QUE JÁ SHIPA É A RESPOSTA?**
///
/// ⚠️ A wave do cap (§5.51) tirou o teto do pincel, e a decomposição seguinte
/// (`ph2d-wet-paint/tests/measure_dab_halves.rs`) mediu o depósito **plano em
/// ~30 ns/r²** de raio 60 a 400 — ou seja, **honestamente limitado pela
/// PEGADA**, sem anomalia a consertar. Isso RE-PRECIFICA a nota do §5.50, que
/// via escala sub-linear (1 : 1,8 : 2,3) e a atribuía ao cap: era o cap
/// ESCONDENDO trabalho, e a "wave com alvo" que ela nomeava não existe.
///
/// Mas um custo honesto continua sendo um custo: `O(r²)` sem teto significa que
/// o artista pode pedir um pincel 8× mais caro que o de raio 141. A pergunta de
/// PRODUTO passa a ser se a resposta já shipa — e ela deveria: o `Grid Size`
/// mede o dab em CÉLULAS (`cell_r = raio / razão`), então a pegada em células
/// cai com **razão²**.
///
/// ⚠️ **Previsão, não afirmação:** razão 2 deveria custar ~1/4 e razão 4 ~1/16.
/// Se a tabela confirmar, não há wave — há um slider que o smoke tem de dizer
/// se é descobrível. Se NÃO confirmar, o slider não protege o caso que ele
/// existe para proteger, e aí sim há trabalho.
#[test]
#[ignore = "sonda de medicao (release); rode com --ignored --nocapture"]
fn measure_whether_the_grid_slider_pays_for_a_big_brush() {
    const SIDE: u32 = 4096;
    const MOVES: u32 = 24;
    const STEP_PX: f32 = 20.0;

    println!("\n  O SLIDER `Grid Size` CONTRA O PINCEL GRANDE ({SIDE}x{SIDE})\n");
    println!(
        "    {:>7} {:>8} {:>14} {:>14} {:>12}",
        "raio", "grid", "total ms", "por entrega", "vs razao 1"
    );

    // ⚠️ Os DOIS raios, porque a pergunta que sobra e' de que o piso e' feito:
    // se o residual em razao 4 cair com a area em PIXELS, ele e' trabalho de
    // pixel (o composite), que a grade do fluido nao encolhe por construcao.
    for radius in [200.0f32, 400.0] {
        let mut base = 0.0f64;
        for ratio in [1.0f64, 2.0, 4.0] {
            let mut t = wetted(SIDE, radius);
            t.set_paint_media(PaintMedia::WetPaint);
            // ⚠️ A razão é congelada por SESSÃO, então tem de ser escrita ANTES
            // do pen-down — trocá-la no meio encerra a água (é o bake).
            t.set_wet_grid_ratio(ratio);
            t.set_brush_size_px(radius * 2.0);
            let (x0, y0) = (600.0f32, 2048.0f32);
            t.on_canvas_pointer(cp([x0, y0], PointerPhase::Down));
            let mut ms = 0.0f64;
            for k in 1..=MOVES {
                let x = x0 + STEP_PX * k as f32;
                let t0 = Instant::now();
                t.on_canvas_pointer(cp([x, y0], PointerPhase::Move));
                ms += t0.elapsed().as_secs_f64() * 1e3;
                let _ = t.take_preview_arc();
            }
            t.on_canvas_pointer(cp([x0 + STEP_PX * MOVES as f32, y0], PointerPhase::Up));
            if ratio == 1.0 {
                base = ms;
            }
            println!(
                "    {radius:>6.0}p {ratio:>7.0} {ms:>13.2} {:>13.3}ms {:>11.2}x",
                ms / f64::from(MOVES),
                ms / base.max(1e-9),
            );
        }
    }
    println!(
        "\n    MEDIDO (2026-08-01): 1,00 / 0,34-0,35 / 0,22 — o slider paga 2,9x na razao 2\n    \
         e 4,5x na razao 4, contra os 4x e 16x que a contagem de celulas sozinha preveria.\n    \
         Resolvendo, ~13-18% de uma entrega NAO cai com a grade do fluido.\n\n    \
         ⚠️ A coluna e' IDENTICA nos dois raios, e isso NAO discrimina de que o piso e'\n    \
         feito: todo termo escala com r², entao o r cancela na razao por construcao. Os\n    \
         candidatos (o composite, que escreve PIXELS, e o AA do `cell_subsamples`, cujo\n    \
         n = min(razao, MAX_AA) mantem as avaliacoes de silhueta ~constantes em area de\n    \
         canvas) ficam NOMEADOS e NAO atribuidos — separa-los exige um relogio por fase\n    \
         dentro da entrega, e o veredito da sonda nao depende disso."
    );
}
