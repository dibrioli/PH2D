//! **De que é feito um MOVE de aquarela** — a decomposição que o doc 28 §7 nomeou como o próximo passo.
//!
//! O censo dos quatro meios (`measure_the_four_media`) diz que a aquarela custa **3,1 ms/move** contra
//! **1,2 do Digital**, e que os dois são **planos no tamanho da tela** (3,07 → 3,12 e 1,17 → 1,21 de
//! 2048² para 4096²) — ou seja, limitados pela PEGADA, que é a forma correta. Ele não diz de que aquele
//! ~1,9 ms de diferença é feito, e sem isso qualquer otimização é palpite.
//!
//! ## Por que ABLAÇÃO POR ENTRADA, e não instrumentação
//!
//! A lição que resolveu o impasto: *uma sonda que re-implementa o laço fica CEGA à porta* — ela segue
//! imprimindo o custo antigo depois de o produto parar de pagá-lo. Então aqui nada é re-implementado:
//! cada linha dirige `on_canvas_pointer`, a porta de verdade, e o que muda entre linhas é um **knob que
//! o artista tem**.
//!
//! ⚠️ **E é por isso que a coluna se chama "o que esta FEATURE custa", não "o que dá para economizar".**
//! Desligar o Warp não é o mesmo trabalho sem uma etapa: é uma aquarela **diferente e mais barata**. O
//! número atribui custo a uma feature — que é a pergunta de PRODUTO ("vale o que cobra?") — e não
//! promete uma economia de graça, que seria vender o que a medição não mede.
//!
//! ## O que a tabela NÃO pode dizer
//!
//! As ablações **não somam** ao total: fases se compõem (o rewet lê campos que o spread constrói) e
//! desligar uma pode baratear outra. Ler a soma como decomposição exata seria o erro que a varredura de
//! raio deste repo já cometeu — lá o passo fixo confundia *raio* com *contagem de dabs*, e a coluna do
//! Digital chegou a FICAR MAIS BARATA com pincel maior. **Compare cada linha com o baseline, nunca as
//! linhas entre si.**

use super::*;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::Falloff;
use std::time::Instant;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um canvas armado em aquarela com o pincel de aquarela **REAL**.
///
/// ⚠️ `..Default::default()` não é atalho aqui: `reset_brush_watercolor` restaura exatamente de
/// `BrushSpec::default()`, então os defaults do spec **são** o pincel que o botão Reset entrega. Uma
/// fixture que montasse os knobs à mão mediria uma aquarela que ninguém pinta.
fn wash(size: u32, radius: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: radius,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_media(PaintMedia::Watercolor);
    t
}

/// Mediana do custo de um MOVE — a porta `on_canvas_pointer`, que é o que o app reporta como
/// `INPUT (fora do frame)`.
///
/// ⚠️ **O `take_preview_arc` é DESCARTADO**, então o tool é dono único do canvas e o `Arc::make_mut`
/// dele é grátis. É o caminho otimista de propósito (mede a aquarela, não a costura de preview), e a
/// diferença contra o app está nomeada no doc 28 §4.8.3 — citar este número como *"o que o artista
/// sente"* seria vender o que a sonda não mede.
fn move_ms(t: &mut PainterTool, size: u32, radius: f32) -> f64 {
    let mid = f64::from(size / 2) as f32;
    let x0 = radius + 20.0;
    const STEP_PX: f32 = 40.0;
    let x1 = x0 + STEP_PX * 20.0;
    assert!(
        x1 < (size as f32) - radius,
        "o traço tem de caber no canvas"
    );
    t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
    let _ = t.take_preview_arc();
    let mut moves = Vec::new();
    let mut x = x0 + STEP_PX;
    while x <= x1 {
        let t0 = Instant::now();
        t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
        moves.push(t0.elapsed().as_secs_f64() * 1e3);
        let _ = t.take_preview_arc();
        x += STEP_PX;
    }
    t.on_canvas_pointer(cp([x1, mid], PointerPhase::Up));
    moves.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    moves[moves.len() / 2]
}

/// O custo de um TRAÇO INTEIRO de comprimento fixo: `(pen-down, soma dos moves, pen-up)`.
///
/// ⚠️ **Por que o traço inteiro e não a mediana por move.** O espaçamento de um dab é
/// `spacing * 2 * radius` (`BrushSpec::dab_spacing_px`), então um passo de mouse FIXO emite ~10 dabs
/// a r=20 e **menos de um** a r=300 — a mediana por move passa a pegar justamente o move que não
/// carimbou nada. Foi exatamente assim que uma varredura anterior deste repo viu a coluna do Digital
/// FICAR MAIS BARATA com pincel maior (o aviso no topo deste arquivo). Sobre um caminho de
/// comprimento FIXO o buraco não existe: é o que o artista faz — arrastar o mouse uma distância —
/// e o total é atribuível porque o espaçamento e a contagem de dabs vão impressos ao lado.
fn stroke_ms(t: &mut PainterTool, size: u32, radius: f32, path_px: f32) -> (f64, f64, f64) {
    let mid = f64::from(size / 2) as f32;
    let x0 = radius + 20.0;
    const STEP_PX: f32 = 40.0;
    let x1 = x0 + path_px;
    assert!(
        x1 < (size as f32) - radius,
        "o traço tem de caber no canvas"
    );

    let t0 = Instant::now();
    t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
    let down = t0.elapsed().as_secs_f64() * 1e3;
    let _ = t.take_preview_arc();

    // ⚠️ UM quadro por move. A reconstrução óptica é devida ao tick desde 2026-08-02, então um laço
    // que só entrega Moves mede uma aquarela que **nunca recompõe** — e reportaria a cura como
    // gratuita. Um evento por quadro é também o A/B justo contra a rota antiga, que compunha uma vez
    // por evento: os dois fazem o MESMO número de composites, e o que sobra é o custo do trabalho.
    let mut moves = 0.0;
    let mut x = x0 + STEP_PX;
    while x <= x1 {
        let t0 = Instant::now();
        t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
        t.paint_tick(1.0 / 60.0);
        moves += t0.elapsed().as_secs_f64() * 1e3;
        let _ = t.take_preview_arc();
        x += STEP_PX;
    }

    let t0 = Instant::now();
    t.on_canvas_pointer(cp([x1, mid], PointerPhase::Up));
    let up = t0.elapsed().as_secs_f64() * 1e3;
    (down, moves, up)
}

/// **Como o custo da aquarela escala com o PINCEL** — a pergunta que a tabela de ablação não faz.
///
/// Aquela tabela mede um raio só (100). O slider do artista vai a `BRUSH_SIZE_MAX_PX = 512`, e o
/// relato mais recente do Enio (doc 28 §5.51, sobre o Wet Paint) é de pintar a **raio 300**. Um
/// número medido num ponto da faixa não diz nada sobre os outros, e a conclusão *"a aquarela não é
/// um problema de performance"* vale exatamente onde ela foi medida.
///
/// A coluna DIGITAL na mesma linha é o controle: ela diz quanto do crescimento é *ser aquarela* e
/// quanto é *carimbar um pincel grande*, que todo meio paga.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_how_the_watercolor_scales_with_the_brush() {
    const SIZE: u32 = 4096;
    const PATH_PX: f32 = 1200.0;

    fn digital(size: u32, radius: f32) -> PainterTool {
        let mut d = PainterTool::default();
        d.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let b = BrushSpec {
            radius_px: radius,
            hardness: 0.5,
            falloff: Falloff::Smooth,
            strength: 1.0,
            color: [0.8, 0.2, 0.1],
            space_attenuation: false,
            ..Default::default()
        };
        d.paint.brush = b;
        for slot in &mut d.paint.brush_by_mode {
            *slot = b;
        }
        d.set_paint_media(PaintMedia::Digital);
        d
    }

    println!(
        "\ntraço de {PATH_PX:.0} px, passo de mouse 40 px, canvas {SIZE}²\n\n\
         {:<6} {:>8} {:>6} | {:>7} {:>8} {:>7} {:>8} | {:>8} | {:>6}",
        "raio", "spacing", "dabs", "down", "moves", "up", "AQUAREL", "digital", "razão"
    );
    for radius in [20.0f32, 60.0, 100.0, 200.0, 300.0, 400.0] {
        let mut t = wash(SIZE, radius);
        let spacing = t.paint.brush.dab_spacing_px();
        let dabs = (PATH_PX / spacing).floor() as u32;
        let (d, m, u) = stroke_ms(&mut t, SIZE, radius, PATH_PX);
        let wc = d + m + u;
        let (dd, dm, du) = stroke_ms(&mut digital(SIZE, radius), SIZE, radius, PATH_PX);
        let dig = dd + dm + du;
        println!(
            "{radius:<6.0} {spacing:>8.1} {dabs:>6} | {d:>7.2} {m:>8.2} {u:>7.2} {wc:>8.2} | \
             {dd:>6.2} {dm:>7.2} {du:>6.2} {dig:>8.2} | {:>5.2}x",
            wc / dig
        );
    }
    println!();
}

/// **O custo por QUADRO cresce ao longo do traço?**
///
/// Depois da deferição, a reconstrução óptica roda uma vez por quadro e a janela dela é o rect do
/// QUADRO. Mas `pour_canvas_wet` caminha `wet_stroke_dirty` — a união **cumulativa** desde o pen-down —
/// e continua rodando uma vez por quadro. Se isso pesa, o custo por quadro sobe conforme o traço
/// cresce, e um traço longo fica quadrático no comprimento.
///
/// ⚠️ O oráculo é o custo do ÚLTIMO quarto do traço contra o do PRIMEIRO, no MESMO traço — não dois
/// traços de comprimentos diferentes, que trocariam de estado de máquina entre as amostras.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_whether_the_frame_cost_grows_along_the_stroke() {
    const SIZE: u32 = 4096;
    const RADIUS: f32 = 100.0;
    const EV: u32 = 4;
    const DT: f32 = 1.0 / 60.0;

    println!("\nraio {RADIUS:.0}, canvas {SIZE}², {EV} eventos/quadro\n");
    println!(
        "{:<10} {:>10} {:>12} {:>12} {:>10}",
        "quadros", "caminho px", "1º quarto", "4º quarto", "razão"
    );
    for frames in [24u32, 48, 96] {
        let path_px = f64::from(frames) as f32 * 40.0;
        let mut t = wash(SIZE, RADIUS);
        let mid = f64::from(SIZE / 2) as f32;
        let x0 = RADIUS + 20.0;
        let step = path_px / f64::from(frames * EV) as f32;
        t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let mut per_frame = Vec::new();
        let mut k = 0u32;
        for _ in 0..frames {
            let t0 = Instant::now();
            for _ in 0..EV {
                k += 1;
                let x = x0 + step * f64::from(k) as f32;
                t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
            }
            t.paint_tick(DT);
            per_frame.push(t0.elapsed().as_secs_f64() * 1e3);
            let _ = t.take_preview_arc();
        }
        t.on_canvas_pointer(cp([x0 + path_px, mid], PointerPhase::Up));
        let q = per_frame.len() / 4;
        let mean = |s: &[f64]| s.iter().sum::<f64>() / s.len() as f64;
        let first = mean(&per_frame[..q]);
        let last = mean(&per_frame[per_frame.len() - q..]);
        println!(
            "{frames:<10} {path_px:>10.0} {first:>12.3} {last:>12.3} {:>9.2}x",
            last / first
        );
    }
    println!();
}

/// **O mesmo caminho pinta o mesmo quadro, seja qual for a taxa de polling?**
///
/// Esta não é uma pergunta de performance — é a propriedade que decide se a reconstrução óptica pode
/// ser deferida. `apply_watercolor` roda hoje **uma vez por evento de ponteiro**
/// (`stroke_lifecycle.rs`, o braço `watercolor_render_active` de `paint_extend`), enquanto o doc dela
/// diz três vezes que a cadência é o QUADRO (*"each frame recomposites"*, *"renderFrame"*, *"the frame
/// dirty rect"*). Deferir para o quadro só é byte-seguro se o resultado **não depende de quantas vezes
/// a reconstrução rodou** — e isso é exatamente o que este teste mede, pelo lado observável.
///
/// ⚠️ E ele responde uma pergunta de PRODUTO junto: se os bytes divergirem, a aparência da aquarela
/// depende da taxa do mouse do artista, o que é um defeito por conta própria — um tablet a 240 Hz e um
/// mouse a 60 Hz pintariam quadros diferentes com o mesmo gesto.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_whether_the_wash_depends_on_the_polling_rate() {
    const SIZE: u32 = 1024;
    const RADIUS: f32 = 100.0;
    const PATH_PX: f32 = 600.0;

    fn paint(step: f32) -> Vec<u8> {
        let mut t = wash(SIZE, RADIUS);
        let mid = f64::from(SIZE / 2) as f32;
        let x0 = RADIUS + 20.0;
        let x1 = x0 + PATH_PX;
        t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
        let mut x = x0 + step;
        while x <= x1 {
            t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
            x += step;
        }
        t.on_canvas_pointer(cp([x1, mid], PointerPhase::Up));
        t.canvas_rgba.to_vec()
    }

    let coarse = paint(40.0);
    println!("\ncaminho de {PATH_PX:.0} px, raio {RADIUS:.0}, canvas {SIZE}²");
    println!(
        "referência: passo 40 px ({} eventos)\n",
        (PATH_PX / 40.0) as u32
    );
    println!(
        "{:<10} {:>8} {:>12} {:>12}",
        "passo px", "eventos", "bytes ≠", "pior delta"
    );
    for step in [40.0f32, 20.0, 10.0, 5.0] {
        let got = paint(step);
        assert_eq!(got.len(), coarse.len(), "mesmo canvas");
        let mut diff = 0u32;
        let mut worst = 0u8;
        for (a, b) in coarse.iter().zip(got.iter()) {
            if a != b {
                diff += 1;
                worst = worst.max(a.abs_diff(*b));
            }
        }
        println!(
            "{step:<10.0} {:>8} {diff:>12} {worst:>12}",
            (PATH_PX / step) as u32
        );
    }
    println!();
}

/// **O custo é por DAB ou por EVENTO?** — a pergunta que a varredura de raio levanta.
///
/// A varredura mostra os moves crescendo LINEARMENTE com o raio enquanto a contagem de dabs CAI
/// (o espaçamento é proporcional ao raio). Duas leituras cabem nisso e pedem curas opostas:
///
/// - **por DAB**: o dab grande custa mais, a taxa de polling não compra trabalho, e o número é o
///   preço honesto de um pincel grande — nada a consertar.
/// - **por EVENTO**: a aquarela paga trabalho do tamanho da PEGADA a cada chamada de
///   `on_canvas_pointer`, tenha ela emitido dab ou não. Aí a taxa de polling do mouse **multiplica**
///   o custo, e um tablet que reporta 240 Hz paga 4× o de um mouse a 60 Hz pelo mesmo desenho.
///
/// O experimento é o mesmo caminho entregue em passos diferentes. Se o TOTAL for invariante, é por
/// dab; se crescer com a contagem de eventos, é por evento. É o mesmo teste que refutou a "espiral"
/// do Wet Paint (doc 28 §5.48), e ali o total ficou em 1,00-1,07× — o controle de que o teste sabe
/// dar a resposta *"não há patologia"* quando não há.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_whether_the_watercolor_charges_per_dab_or_per_event() {
    const SIZE: u32 = 4096;
    const RADIUS: f32 = 100.0;
    const PATH_PX: f32 = 1200.0;

    /// UM TRAÇO, dirigido em QUADROS — o que o app de fato faz.
    ///
    /// ⚠️ Uma sonda que entrega eventos e nunca chama `paint_tick` mede uma aquarela que **nunca
    /// reconstrói**, e reportaria a rota por-quadro como gratuita. O laço aqui é o do produto: os
    /// eventos do quadro, depois UM tick (render_loop ~698 → ~1198), e o relógio cobre os dois.
    fn stroke_in_frames(
        t: &mut PainterTool,
        size: u32,
        radius: f32,
        path_px: f32,
        frames: u32,
        ev_per_frame: u32,
    ) -> f64 {
        const DT: f32 = 1.0 / 60.0;
        let mid = f64::from(size / 2) as f32;
        let x0 = radius + 20.0;
        let n = frames * ev_per_frame;
        let step = path_px / f64::from(n) as f32;
        t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let mut total = 0.0;
        let mut k = 0u32;
        for _ in 0..frames {
            let t0 = Instant::now();
            for _ in 0..ev_per_frame {
                k += 1;
                let x = x0 + step * f64::from(k) as f32;
                t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
            }
            t.paint_tick(DT);
            total += t0.elapsed().as_secs_f64() * 1e3;
            let _ = t.take_preview_arc();
        }
        t.on_canvas_pointer(cp([x0 + path_px, mid], PointerPhase::Up));
        total
    }

    // O traço leva o mesmo TEMPO (30 quadros a 60 fps = 0,5 s) e desenha o mesmo caminho; o que a
    // taxa do dispositivo muda é só quantos eventos caem dentro de cada quadro. É a pergunta de
    // produto: *um tablet de 480 Hz cobra mais que um mouse de 120 Hz pelo mesmo desenho?*
    const FRAMES: u32 = 30;
    println!(
        "\ntraço de {PATH_PX:.0} px em {FRAMES} quadros (0,5 s), raio {RADIUS:.0}, canvas {SIZE}²\n\n\
         {:<7} {:>8} {:>8} | {:>12} {:>8} | {:>12} {:>8} | {:>8}",
        "Hz", "ev/frame", "eventos", "POR QUADRO", "razão", "por evento", "razão", "ganho"
    );
    let (mut b_frame, mut b_event) = (0.0, 0.0);
    for (i, ev) in [2u32, 4, 8, 16].into_iter().enumerate() {
        let mut a = wash(SIZE, RADIUS);
        let per_frame = stroke_in_frames(&mut a, SIZE, RADIUS, PATH_PX, FRAMES, ev);

        let mut b = wash(SIZE, RADIUS);
        b.wash.per_event = true; // a rota congelada, no MESMO processo
        let per_event = stroke_in_frames(&mut b, SIZE, RADIUS, PATH_PX, FRAMES, ev);

        if i == 0 {
            b_frame = per_frame;
            b_event = per_event;
        }
        println!(
            "{:<7} {ev:>8} {:>8} | {per_frame:>12.2} {:>7.2}x | {per_event:>12.2} {:>7.2}x | {:>7.2}x",
            ev * 60,
            ev * FRAMES,
            per_frame / b_frame,
            per_event / b_event,
            per_event / per_frame
        );
    }
    println!();
}

/// **A tabela: o que cada feature da aquarela cobra por MOVE.**
///
/// A primeira linha é o baseline (o pincel de aquarela como ele sai do Reset); cada linha seguinte
/// desliga **um** knob e mede de novo. A última é o Digital, para dizer quanto do total é *carimbar
/// dabs* e quanto é *ser aquarela*.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_what_a_watercolor_move_is_made_of() {
    const SIZE: u32 = 4096;
    const RADIUS: f32 = 100.0;

    // A configuração que estamos medindo, PERGUNTADA ao produto e impressa — ler os defaults do fonte e
    // torcer é como se documenta uma medição de outra coisa.
    {
        let t = wash(SIZE, RADIUS);
        let b = &t.paint.brush;
        println!(
            "\n[wash] pincel medido: edge_gain={:.2} edge_spread={:.2} granulation={:.2} warp={:.2} \
             wet_smudge={:.2} wet_rewet={:.2} wet_charge={:.2} wet_dilution={:.2} wet_pull={:.2} \
             pigment={} pigment_mix={:.2} fill={:.2} depth={:.2}",
            b.edge_gain,
            b.edge_spread,
            b.granulation,
            b.warp,
            b.wet_smudge,
            b.wet_rewet,
            b.wet_charge,
            b.wet_dilution,
            b.wet_pull,
            b.pigment,
            b.pigment_mix,
            b.fill,
            b.depth,
        );
    }

    let base = move_ms(&mut wash(SIZE, RADIUS), SIZE, RADIUS);
    println!(
        "\n{:<26} {:>10} {:>12}",
        "configuração", "move ms", "vs baseline"
    );
    println!("{:<26} {base:>10.3} {:>12}", "AQUARELA (baseline)", "—");

    // Cada ablação é um knob do PAINEL, aplicado pela porta pública que o painel usa.
    /// Um knob do painel e o nome dele — a unidade de ablação desta tabela.
    type Ablation = (&'static str, fn(&mut PainterTool));
    let ablations: [Ablation; 7] = [
        ("sem Warp", |t| t.set_brush_warp(0.0)),
        ("sem Granulation", |t| t.set_brush_granulation(0.0)),
        ("sem Edge (gain 0)", |t| t.set_brush_edge_gain(0.0)),
        ("sem Spread", |t| t.set_brush_edge_spread(0.0)),
        ("sem Rewet", |t| t.set_brush_wet_rewet(0.0)),
        ("sem Smudge", |t| t.set_brush_wet_smudge(0.0)),
        ("sem Pigment mixing", |t| t.set_brush_pigment_mixing(0.0)),
    ];
    for (name, off) in ablations {
        let mut t = wash(SIZE, RADIUS);
        off(&mut t);
        let ms = move_ms(&mut t, SIZE, RADIUS);
        println!("{name:<26} {ms:>10.3} {:>11.3}", ms - base);
    }

    // Tudo desligado: o que sobra é o depósito de cobertura + o recompose óptico mínimo.
    let mut t = wash(SIZE, RADIUS);
    for (_, off) in ablations {
        off(&mut t);
    }
    let bare = move_ms(&mut t, SIZE, RADIUS);
    println!(
        "{:<26} {bare:>10.3} {:>11.3}",
        "TUDO desligado",
        bare - base
    );

    // E o Digital, na MESMA fixture: a diferença é o preço de ser aquarela.
    let mut d = PainterTool::default();
    d.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    let b = BrushSpec {
        radius_px: RADIUS,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    d.paint.brush = b;
    for slot in &mut d.paint.brush_by_mode {
        *slot = b;
    }
    d.set_paint_media(PaintMedia::Digital);
    let dig = move_ms(&mut d, SIZE, RADIUS);
    println!(
        "{:<26} {dig:>10.3} {:>11.3}",
        "DIGITAL (o carimbo)",
        dig - base
    );
    println!();
}

/// **Quanta ÁREA a aquarela caminha por quadro, com o pincel e os knobs do ARTISTA?**
///
/// ⚠️ Esta sonda **não olha o relógio**, de propósito: ela CONTA texels. Um retângulo tem área
/// reprodutível com a máquina saturada, e a §5.49 fixou que nenhum número de tempo desta máquina vale
/// nada com `load average` acima de ~5. A forma responde antes do relógio (§5.12).
///
/// Duas grandezas por quadro, e elas têm formas DIFERENTES:
///
/// - a **PEGADA** — o que um move DEVERIA custar. É `(2r)²` e não muda com a tela.
/// - o **POUR** — `pour_canvas_wet` caminha `wet_stroke_dirty`, a união **CUMULATIVA** desde o
///   pen-down (três sítios fazem união; só o pen-down zera), clampada ao canvas. Ele cresce com o
///   TRAÇO, não com a pegada — e o clamp é o que o faz responder ao tamanho da tela: o mesmo gesto
///   perto de uma borda é recortado a 2048² e não a 4096².
///
/// O que decide é a RAZÃO pour/pegada ao longo do traço: se ela cresce, o custo por quadro cresce
/// junto e um traço longo fica quadrático no comprimento.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_the_area_the_wash_walks_per_frame() {
    const RADIUS: f32 = 250.0;
    const EV: u32 = 4;
    const FRAMES: u32 = 48;
    const DT: f32 = 1.0 / 60.0;
    const PATH_PX: f32 = 1500.0;

    // Os knobs do report do Enio (screenshot de 2026-08-02).
    fn artist(t: &mut PainterTool) {
        for slot in &mut t.paint.brush_by_mode {
            slot.wet_charge = 0.755;
            slot.wet_dilution = 0.168;
            slot.wet_pull = 0.477;
            slot.wet_rewet = 0.400;
            slot.wet_smudge = 0.197;
        }
        t.paint.brush.wet_charge = 0.755;
        t.paint.brush.wet_dilution = 0.168;
        t.paint.brush.wet_pull = 0.477;
        t.paint.brush.wet_rewet = 0.400;
        t.paint.brush.wet_smudge = 0.197;
    }

    let foot = f64::from(2.0 * RADIUS) * f64::from(2.0 * RADIUS);
    println!("\nraio {RADIUS:.0}, {EV} ev/quadro, caminho {PATH_PX:.0} px");
    println!("pegada de um dab = {:.2} M texels\n", foot / 1e6);
    println!(
        "{:<8} {:>12} {:>12} {:>12} {:>12} {:>10}",
        "canvas", "tela M", "pour 1º M", "pour últ M", "razão pour", "pour/pegada"
    );

    for size in [2048u32, 4096] {
        let mut t = wash(size, RADIUS);
        artist(&mut t);
        let mid = f64::from(size / 2) as f32;
        let x0 = RADIUS + 20.0;
        assert!(
            x0 + PATH_PX + RADIUS < size as f32,
            "o traço tem de caber no canvas {size}"
        );
        let step = PATH_PX / f64::from(FRAMES * EV) as f32;
        t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
        let _ = t.take_preview_arc();

        let mut areas = Vec::new();
        let mut k = 0u32;
        for _ in 0..FRAMES {
            for _ in 0..EV {
                k += 1;
                let x = x0 + step * f64::from(k) as f32;
                t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
            }
            t.paint_tick(DT);
            // O rect que o `pour` do PRÓXIMO quadro vai caminhar (clampado como ele clampa).
            if let Some(r) = t.paint.wet_stroke_dirty {
                let x1 = u64::from(r.x + r.w).min(u64::from(size));
                let y1 = u64::from(r.y + r.h).min(u64::from(size));
                let w = x1.saturating_sub(u64::from(r.x));
                let h = y1.saturating_sub(u64::from(r.y));
                areas.push((w * h) as f64);
            }
            let _ = t.take_preview_arc();
        }
        t.on_canvas_pointer(cp([x0 + PATH_PX, mid], PointerPhase::Up));

        assert!(!areas.is_empty(), "a fixture tem de POUR — sem rect não há o que medir");
        let q = areas.len() / 4;
        let mean = |s: &[f64]| s.iter().sum::<f64>() / s.len() as f64;
        let first = mean(&areas[..q]);
        let last = mean(&areas[areas.len() - q..]);
        let screen = f64::from(size) * f64::from(size);
        println!(
            "{size:<8} {:>12.2} {:>12.2} {:>12.2} {:>11.2}x {:>9.1}x",
            screen / 1e6,
            first / 1e6,
            last / 1e6,
            last / first,
            last / foot
        );
    }
    println!();
}

/// **Quantos texels um QUADRO de aquarela caminha, e qual knob paga por eles?**
///
/// Ablação **pela ENTRADA** (os knobs do painel, nunca instrumentação — uma sonda que refaz o laço
/// fica cega à porta, §5.11), medindo a única grandeza que sobrevive à máquina disputada: a ÁREA da
/// janela de leitura, somada pelo próprio produto em `WashCadence::window_px`.
///
/// ⚠️ A janela é `dirty ⊕ pad` (saída) `⊕ pad` (leitura), então ela é `dirty + 4·pad` por eixo — e o
/// `pad` é FUNÇÃO DOS KNOBS: `Rewet > 0` faz `reach = spread`, e a água CARREGADA (`Dilution > 0`,
/// que aloca `stroke_water`) o DOBRA. Um pad de duas dezenas de texels desaparece num pincel grande e
/// domina num pequeno; é por isso que o número certo é a razão contra a PEGADA, não o absoluto.
#[test]
#[ignore = "measurement, not a gate"]
fn measure_the_area_a_watercolor_frame_walks() {
    const SIZE: u32 = 4096;
    const EV: u32 = 4;
    const FRAMES: u32 = 24;
    const DT: f32 = 1.0 / 60.0;

    // Os knobs do report do Enio (screenshot de 2026-08-02).
    fn artist(t: &mut PainterTool, dilution: f32, rewet: f32) {
        let set = |b: &mut BrushSpec| {
            b.wet_charge = 0.755;
            b.wet_pull = 0.477;
            b.wet_smudge = 0.197;
            b.wet_dilution = dilution;
            b.wet_rewet = rewet;
        };
        set(&mut t.paint.brush);
        for slot in &mut t.paint.brush_by_mode {
            set(slot);
        }
    }

    println!("\ncanvas {SIZE}², {EV} ev/quadro, {FRAMES} quadros — AREA, nao relogio\n");
    println!(
        "{:<7} {:>9} {:>26} {:>14} {:>12} {:>10}",
        "raio", "pegada M", "ablacao", "janela/quadro M", "composites", "vs pegada"
    );
    for radius in [60.0f32, 250.0] {
        let foot = f64::from(2.0 * radius) * f64::from(2.0 * radius);
        for (name, dil, rew) in [
            ("como o Enio ajustou", 0.168, 0.400),
            ("sem Dilution", 0.0, 0.400),
            ("sem Rewet", 0.168, 0.0),
            ("sem os dois", 0.0, 0.0),
        ] {
            let mut t = wash(SIZE, radius);
            artist(&mut t, dil, rew);
            let mid = f64::from(SIZE / 2) as f32;
            let x0 = radius + 20.0;
            let path = 900.0f32;
            let step = path / f64::from(FRAMES * EV) as f32;
            t.on_canvas_pointer(cp([x0, mid], PointerPhase::Down));
            let _ = t.take_preview_arc();
            t.wash.window_px = 0;
            t.wash.composites = 0;
            let mut k = 0u32;
            for _ in 0..FRAMES {
                for _ in 0..EV {
                    k += 1;
                    t.on_canvas_pointer(cp(
                        [x0 + step * f64::from(k) as f32, mid],
                        PointerPhase::Move,
                    ));
                }
                t.paint_tick(DT);
                let _ = t.take_preview_arc();
            }
            let (px, n) = (t.wash.window_px, t.wash.composites);
            t.on_canvas_pointer(cp([x0 + path, mid], PointerPhase::Up));
            assert!(n > 0, "a fixture tem de COMPOR — sem composite nao ha area");
            let per = px as f64 / f64::from(n);
            println!(
                "{radius:<7.0} {:>9.2} {name:>26} {:>14.2} {n:>12} {:>9.1}x",
                foot / 1e6,
                per / 1e6,
                per / foot
            );
        }
    }
    println!();
}
