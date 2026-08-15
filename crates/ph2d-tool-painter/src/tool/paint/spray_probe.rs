//! **O QUE UM PONTO DO CAMINHO CUSTA QUANDO DEIXA `n` MARCAS** — a medição que decide o
//! `ph2d_painter_brush::stroke::spray::SPRAY_COUNT_MAX` (plano 38 W5).
//!
//! ⚠️ **Pela porta do PRODUTO** (`on_canvas_pointer`), nunca por um laço próprio: o que decide o teto
//! é o que o artista paga por movimento do dedo, e a lição de que uma sonda com laço próprio fica
//! CEGA à porta já custou duas waves a este módulo (doc 28 §5.11, §5.46).
//!
//! ⚠️ **A fixture arma o JITTER**, e não é enfeite: sem ele as `n` cópias caem umas sobre as outras,
//! tocam os MESMOS texels e o custo mente para baixo. Uma nuvem espalhada escreve em área maior, e é
//! ela que o artista de facto pinta — um spray com spread zero é o estado degenerado, não o caso.
//!
//! Rodar: `cargo test -p ph2d-tool-painter --release measure_the_spray -- --ignored --nocapture`

use super::measure_shape_system::{cp, tool};
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::StrokeMethod;
use ph2d_painter_brush::stroke::spray::SPRAY_COUNT_MAX;
use std::time::Instant;

/// Uma espiral apertada — a MESMA fixture das tabelas do Sketchy e do Wire, para os números serem
/// comparáveis entre si. Devolve `(ms por evento de Move, ms do pior evento)`.
fn spiral(count: u32, jitter: f32, radius: f32, turns: usize) -> (f64, f64) {
    let side = 2048u32;
    let mut t = tool(side, PaintMedia::Digital, radius);
    t.paint.brush.stroke_method = StrokeMethod::Space;
    t.paint.brush.spray_count = count;
    // O canto caro: as três randomizações por-dab armadas, que é o que espalha a nuvem de facto.
    t.paint.brush.jitter = jitter;
    t.paint.brush.jitter_scale = if jitter > 0.0 { 0.5 } else { 0.0 };
    t.paint.brush.jitter_rotate = if jitter > 0.0 { 1.0 } else { 0.0 };
    #[allow(clippy::cast_precision_loss)]
    let c = (side / 2) as f32;
    let steps = turns * 40;
    let pt = |i: usize| {
        #[allow(clippy::cast_precision_loss)]
        let u = i as f32 / steps as f32;
        // ⚠️ `cos`/`sin` aqui é de SONDA, não de produto: o HR-5 fala do caminho que pinta.
        let ang = u * (turns as f32) * std::f32::consts::TAU;
        let r = 20.0 + u * 180.0;
        [c + r * ang.cos(), c + r * ang.sin()]
    };
    t.on_canvas_pointer(cp(pt(0), PointerPhase::Down));
    let mut total = 0.0f64;
    let mut worst = 0.0f64;
    for i in 1..=steps {
        let t0 = Instant::now();
        t.on_canvas_pointer(cp(pt(i), PointerPhase::Move));
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        total += ms;
        worst = worst.max(ms);
    }
    t.on_canvas_pointer(cp(pt(steps), PointerPhase::Up));
    #[allow(clippy::cast_precision_loss)]
    (total / steps as f64, worst)
}

/// **O TETO DO COUNT** — a maior contagem cujo PIOR evento cabe no kill de 8 ms.
///
/// ⚠️ **A sonda que escolhe o teto não pode ser capada pelo teto**, e a primeira corrida desta wave
/// foi: o `spray_copies` clampa em [`SPRAY_COUNT_MAX`], então as linhas 24 / 32 / 48 / 64 mediram
/// todas **a mesma nuvem de 16** — 0,309 / 0,310 / 0,318 / 0,312 ms, uma tabela **plana** que se lê
/// como *"o custo satura"* e significa *"a sonda parou de medir"*. Para medir acima do teto de hoje,
/// suba a const, rode, e escreva o número que a corrida der.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn measure_the_spray_budget_per_event() {
    println!("[spray] espiral de 8 voltas, 2048², pela porta do produto, jitter cheio");
    println!("[spray] teto vigente: SPRAY_COUNT_MAX = {SPRAY_COUNT_MAX} (a sonda clampa nele)");
    // ⚠️ A PRIMEIRA espiral de uma corrida paga o *first-touch* dos planos e a memória do alocador, e
    // mede o dobro das seguintes (medido: `count 1, r=24` saiu em 0,500/2,666 contra 0,199/1,397 do
    // `count 2` logo abaixo — não-monotônico onde a lei é linear). Ela é descartada.
    let _ = spiral(1, 1.0, 24.0, 8);
    println!(
        "{:>7} {:>7}  {:>10} {:>10}  {:>10}",
        "count", "raio", "ms/evento", "pior ms", "veredito"
    );
    // ⚠️ TRÊS colunas de raio, porque o custo é um PRODUTO e um teto escalar tem de escolher o canto:
    // `24` é a referência das outras tabelas do plano 38, `200` é um pincel grande de verdade, e
    // `512` é o `BRUSH_SIZE_MAX_PX` — o canto que os dois sliders alcançam JUNTOS.
    for radius in [24.0f32, 200.0, 512.0] {
        for n in [1u32, 2, 4, 8, 16, 32, 64, 128, 256] {
            let (avg, worst) = spiral(n, 1.0, radius, 8);
            println!(
                "{n:>7} {radius:>7.0}  {avg:>10.3} {worst:>10.3}  {:>10}",
                if worst > 8.0 { "ESTOURA" } else { "sob o kill" }
            );
        }
    }
    println!("[spray] leitura: o teto é a maior contagem cujo PIOR evento cabe nos 8 ms.");
}

// ⚠️ **NÃO existe aqui uma segunda sonda "o custo através dos tamanhos de pincel", e a ausência é
// deliberada.** Ela foi escrita, rodada e removida: a varredura acima já percorre TRÊS raios, e a
// interação `contagem × área` é exatamente o que as suas colunas mostram — **dentro de UMA corrida**,
// que é o que as torna comparáveis entre si. A irmã media o mesmo fato numa corrida SEPARADA, e as
// duas discordaram no ponto que partilhavam: `r=200, count 16` deu **5,073 ms** de pior evento numa e
// **7,074** na outra, os mesmos ~40% de deriva de máquina que o doc 28 §5.46 já documenta. Duas
// sondas para um fato só podem divergir, e a que diverge é a que alguém vai citar.
