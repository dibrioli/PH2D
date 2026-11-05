//! ⭐ **Bug do congelamento de 05/11** (`docs/Painter/BUGS_painter.md`): esperar o motor DENTRO de um job do rayon é
//! deadlock. O operário que esperava um pedaço de um estágio da água rouba o job injectado pelo fio principal e, por
//! cima desse pedaço, dorme no `recv` à espera de um motor que o estágio só devolve quando o pedaço acabar.
//!
//! Tem de morar em `tests/it`: aqui a lib compila sem `cfg(test)`, e só assim o `PainterTool::no_pool` entra no pool.
//! O 1.º lote de cada traço chega com o motor no worker (um quadro passa entre o pen-down e o 1.º movimento), que é a
//! porta que bloqueava. Um vigia encerra o processo se nada andar, para o impasse não pendurar a suíte.

use crate::exame_bancada::pinceis;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool, Tool};
use ph2d_tool_painter::PainterTool;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

const LADO: u32 = 2048;
const VOLTAS: u64 = 20;
const ORCAMENTO: Duration = Duration::from_secs(15);
/// Uma volta custa dezenas de ms; este prazo só se esgota sem progresso nenhum, mesmo com a máquina cheia.
const SEM_PROGRESSO: Duration = Duration::from_secs(60);

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn voltas(andou: &AtomicU64) {
    let mut t = PainterTool::default();
    let n = LADO as usize;
    t.set_source(vec![255u8; n * n * 4], LADO, LADO);
    // A poça: com tinta molhada o worker corre estágios paralelos enquanto o traço pede o motor.
    let poca = pinceis("Wet Wash");
    poca[0].aplica(&mut t, false);
    for k in 0..6 {
        let y = 200.0 + k as f32 * 260.0;
        t.on_canvas_pointer(cp([60.0, y], PointerPhase::Down));
        for j in 1..=30 {
            t.on_canvas_pointer(cp([60.0 + j as f32 * 62.0, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([1920.0, y], PointerPhase::Up));
    }
    let pincel = pinceis("Knife Grainy");
    pincel[0].aplica(&mut t, false);
    let t0 = Instant::now();
    for i in 0..VOLTAS {
        if t0.elapsed() > ORCAMENTO {
            break;
        }
        let y = 200.0 + (i % 40) as f32 * 40.0;
        let mut x = 100.0f32;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Down));
        // A faca só pousa no 1.º movimento: um quadro aqui entrega o motor ao worker com o traço ainda por abrir.
        t.on_tick(16.0);
        // Varia o instante do pedido dentro do estágio em voo.
        std::thread::sleep(Duration::from_micros(200 + 700 * (i % 8)));
        for _ in 0..12 {
            x += 60.0;
            t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([x, y], PointerPhase::Up));
        // O tique entrega o motor ao worker: o próximo traço abre com ele fora de casa.
        t.on_tick(16.0);
        andou.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn o_traco_nao_espera_o_motor_dentro_do_pool() {
    // Com muitos fios há quase sempre um livre para o job e o impasse fica raro. Com dois, os dois estão no estágio.
    // (O nextest dá um processo a cada teste; noutro executor, o pool já criado fica e só a régua perde força.)
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build_global();
    let andou = Arc::new(AtomicU64::new(0));
    let do_fio = Arc::clone(&andou);
    let fio = std::thread::Builder::new()
        .name("voltas".into())
        .spawn(move || voltas(&do_fio))
        .expect("o fio das voltas nasce");
    let (mut visto, mut desde) = (0, Instant::now());
    while !fio.is_finished() {
        std::thread::sleep(Duration::from_millis(50));
        let agora = andou.load(Ordering::Relaxed);
        if agora != visto {
            (visto, desde) = (agora, Instant::now());
        } else if desde.elapsed() > SEM_PROGRESSO {
            // Os fios do impasse não acordam: só o fim do processo solta o teste.
            eprintln!(
                "IMPASSE: o traço parou na volta {visto}, sem progresso em {SEM_PROGRESSO:?} \
                 (espera sem prazo dentro de um job do pool)"
            );
            std::process::exit(101);
        }
    }
    fio.join().expect("as voltas não entram em pânico");
    let feitas = andou.load(Ordering::Relaxed);
    println!("[pool] {feitas} voltas");
    assert!(feitas >= 5, "a régua andou pouco: {feitas} voltas");
}
