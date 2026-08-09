//! PROBE + DIAGNÓSTICO — a regressão do **Composite Brush** (Enio 2026-08-09): *"agora não consegue
//! pintar mais que uma mancha de tinta"*, com os artefatos retangulares da família do
//! `BUGS_painter.md`.
//!
//! ⚠️ **NÃO é o Per-Layer Color** (bugs #2/#11, que são do slot Shape). O Composite Brush é a pilha de
//! três camadas Brush·Smear·Blur do `composite.rs`.
//!
//! **Medido, colunas entintadas ao longo do caminho (141 possíveis, raio 8):**
//!
//! | pilha | colunas | mapa |
//! |---|---|---|
//! | composite OFF (controle) | 141 | `#############################` |
//! | Brush + Smear + Blur | 108 | `####..######..######..#######` |
//! | **Brush + Smear** | **108** | idem — o Smear sozinho reproduz |
//! | Brush + Blur | 141 | limpo — **o Blur está inocente** |
//!
//! **A ablação nomeia a causa**: com o `restore_before` da sessão de smear removido o traço volta a
//! **141/141**; sem o `reset_stroke_height` ele continua em 108. É o **RESTORE** que apaga.
//!
//! **O mecanismo:** desde a wave do campo de smear, uma esfregada *acumula um mapa de deslocamento e
//! resolve UMA vez a partir dos pixels CONGELADOS no pen-down* — é essa lei que matou o filamento. Em
//! composite, porém, a camada **Brush deposita tinta no mesmo canvas durante o traço**, e o render de
//! smear do batch SEGUINTE reescreve aquela região a partir da base congelada, **levando embora o que o
//! Brush acabou de pôr**. As falhas são periódicas porque acompanham as regiões dos batches, e é isso
//! que a foto mostra como listras retangulares.
//!
//! ⚠️ **Os dois tempos de vida se contradizem:** o smear é **por TRAÇO** (resolve de uma base
//! congelada) e o composite promete *"cada operação processa o canvas como a de baixo o deixou"*, que é
//! **por BATCH**. Nenhum dos dois está errado sozinho.
//!
//! **A cura tem de escolher um**, e as duas custam desenho, não uma linha:
//! 1. a base congelada do smear **absorve** a tinta que o Brush deposita durante o traço — fisicamente
//!    é o que a pilha promete (*pinta, depois esfrega o que pintou*), e a tinta posta mais tarde é
//!    empurrada menos, o que é correto; mas mexe no invariante que curou o filamento e precisa do
//!    gate de espaçamento junto;
//! 2. o composite roda o Brush **fora** do laço por-batch, uma vez no fim — mais barato de escrever e
//!    muda o resultado (a tinta deixa de ser esfregada pela própria pilha, que é o ponto da feature).
//!
//! Enquanto isso não é decidido, este probe é o número: rode-o e compare com a tabela.
use super::*;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::Falloff;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn drag(t: &mut PainterTool, y: f32, x0: f32, x1: f32) {
    t.on_canvas_pointer(cp([x0, y], PointerPhase::Down));
    let mut x = x0;
    while x < x1 {
        x += 1.0;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([x1, y], PointerPhase::Up));
}

/// How many columns along the path carry ink — a full stroke inks (nearly) all of them, "one blob" inks
/// only the few under the last dabs.
fn inked_columns(t: &PainterTool, y: u32, size: u32, xs: std::ops::Range<u32>) -> u32 {
    let px = &t.canvas_rgba;
    xs.filter(|x| {
        let i = ((y * size + x) * 4) as usize;
        px.get(i).is_some_and(|&r| r < 200)
    })
    .count() as u32
}

#[test]
fn probe_composite_lays_a_whole_stroke() {
    const SIZE: u32 = 200;
    let radius: f32 = std::env::var("PROBE_R")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8.0);
    let _ = radius;
    // Each row: what the three stack positions hold. Position 0 is the TOP (runs LAST).
    let cases: [(&str, [Option<CompositeOp>; 3]); 4] = [
        ("composite OFF (control)", [None, None, None]),
        (
            "Brush + Smear + Blur (the default stack)",
            [
                Some(CompositeOp::Brush),
                Some(CompositeOp::Smear),
                Some(CompositeOp::Blur),
            ],
        ),
        (
            "Brush + Smear only",
            [Some(CompositeOp::Brush), Some(CompositeOp::Smear), None],
        ),
        (
            "Brush + Blur only",
            [Some(CompositeOp::Brush), None, Some(CompositeOp::Blur)],
        ),
    ];
    for (name, stack) in cases {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
        t.paint.brush.radius_px = radius;
        t.paint.brush.hardness = 1.0;
        t.paint.brush.falloff = Falloff::Constant;
        t.paint.brush.color = [0.6, 0.0, 0.0];
        t.paint.brush.space_attenuation = false;
        if stack.iter().any(Option::is_some) {
            t.paint.composite_enabled = true;
            for (pos, op) in stack.iter().enumerate() {
                match op {
                    Some(op) => {
                        t.paint.composite[pos] = CompositeLayer {
                            op: *op,
                            strength: if matches!(op, CompositeOp::Brush) {
                                1.0
                            } else {
                                0.5
                            },
                        };
                    }
                    None => t.paint.composite[pos].strength = 0.0,
                }
            }
        }
        drag(&mut t, 100.0, 30.0, 170.0);
        let inked = inked_columns(&t, 100, SIZE, 30..171);
        // WHERE the ink is decides the mechanism: if a later smear re-render is putting the region back
        // to the base frozen at pen-down, the loss is at the START of the path, not scattered.
        let px = &t.canvas_rgba;
        let hit = |x: u32| px[((100 * SIZE + x) * 4) as usize] < 200;
        let first = (30..171).find(|&x| hit(x));
        let last = (30..171).rev().find(|&x| hit(x));
        let map: String = (30..171)
            .step_by(5)
            .map(|x| if hit(x) { '#' } else { '.' })
            .collect();
        eprintln!("{name}: {inked}/141  span={first:?}..{last:?}  {map}");
    }
}
