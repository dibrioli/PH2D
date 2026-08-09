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

/// **A pilha pinta o traço INTEIRO.**
///
/// O gate da regressão: *"o Composite Brush não consegue pintar mais que uma mancha"*. Com o
/// `fold_brush_into_smear_base` no lugar as três pilhas medem 141/141; sem ele, 108.
///
/// ⚠️ **A OUTRA metade — *"e o Smear da pilha ainda esfrega"* — NÃO está gateada, e o motivo é fixture,
/// não preguiça.** Três tentativas, cada uma saturada por um mecanismo diferente, medidas:
/// 1. texels FORA do eixo do traço: **0 contra 0** — a esfregada desloca *ao longo* do traço, não para
///    os lados, então não há nada a medir ali;
/// 2. o ALCANCE da tinta além de onde o pincel parou: **176 contra 176** — é a pegada do próprio Brush,
///    e esfregar uma faixa uniforme ao longo do próprio eixo é **no-op por simetria**;
/// 3. uma marca AZUL pré-pintada para o Smear carregar: **0 contra 0** com o Brush opaco (ele cobre a
///    marca) e também com ele translúcido (`strength 0.3`), onde a 1ª metade deixa de entintar.
///
/// ⇒ O risco que fica NOMEADO: se a absorção da base tornasse o Smear inerte dentro da pilha, este gate
/// **não veria**. O smoke é quem julga isso hoje; a fixture que separa as duas provavelmente precisa de
/// uma esfregada TRANSVERSAL sobre tinta seca, e não de um traço reto sobre uma marca.
///
/// **Mutação que must bleed:** apagar a chamada de `fold_brush_into_smear_base` na rota do Brush ⇒
/// 108 de 141.
#[test]
fn the_composite_stack_lays_the_whole_stroke() {
    const SIZE: u32 = 200;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    t.paint.brush.radius_px = 8.0;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.falloff = Falloff::Constant;
    t.paint.brush.color = [0.6, 0.0, 0.0];
    t.paint.brush.space_attenuation = false;
    t.paint.composite_enabled = true;
    t.paint.composite[0] = CompositeLayer {
        op: CompositeOp::Brush,
        strength: 1.0,
    };
    t.paint.composite[1] = CompositeLayer {
        op: CompositeOp::Smear,
        strength: 0.5,
    };
    t.paint.composite[2] = CompositeLayer {
        op: CompositeOp::Blur,
        strength: 0.5,
    };
    drag(&mut t, 100.0, 30.0, 170.0);
    let whole = inked_columns(&t, 100, SIZE, 30..171);
    assert_eq!(
        whole, 141,
        "a pilha perdeu tinta: {whole} de 141 colunas entintadas ao longo do caminho"
    );
}

/// SONDA — a COSTURA vertical que sobrou (Enio 2026-08-09, foto com a seta).
///
/// ⚠️ **ELA NÃO REPRODUZ AQUI, e o resultado negativo é o achado:** as quatro pilhas medem **3 níveis**
/// de degrau, idêntico ao Brush sozinho ⇒ a fixture **não contém o fenômeno**, e escrever mais uma dobra
/// contra ela seria um chute (já foram três). O que a foto mostra é uma **coluna VERTICAL de altura
/// inteira**, não a bbox de um dab — a fronteira de uma região do pipeline de DISPLAY (composite parcial
/// · upload parcial de GPU · a banda de um passe paralelo), não da operação que esta wave consertou.
///
/// **O que a fixture não tem, e provavelmente é o que falta:** pincel grande, traço CURVO, muitos batches
/// e canvas do tamanho do produto — o Enio observa a costura com o Smear na pilha, então a suspeita é a
/// REGIÃO que o render de smear reescreve, não o mapa que ele acumula.
///
/// ⇒ O próximo passo **não é código**: é a armadilha que o BUGS_painter #11 já deixou armada —
/// `PH2D_PREVIEW_DIAG=1` diz qual produtor tem o slot e que bbox subiu, e `PH2D_PREVIEW_DUMP=<dir>` grava
/// o composite EXATO antes de qualquer overlay. Retângulo nos PNGs ⇒ está no composite; PNGs limpos com a
/// costura na tela ⇒ é upload ou overlay. É literalmente a lição daquele bug: *pare o harness mais cedo e
/// instrumente o app*.
///
/// O gate de colunas entintadas é **cego a isto**: uma costura não tira tinta, ela põe um degrau. Aqui o
/// oráculo é o maior salto HORIZONTAL entre colunas vizinhas ao longo de uma linha — um degrau
/// axis-aligned aparece como um pico isolado, e uma borda honesta de pincel não.
#[test]
fn probe_composite_vertical_seam() {
    const SIZE: u32 = 200;
    for (name, ops) in [
        ("Brush só", vec![CompositeOp::Brush]),
        ("Brush + Blur", vec![CompositeOp::Brush, CompositeOp::Blur]),
        (
            "Brush + Smear",
            vec![CompositeOp::Brush, CompositeOp::Smear],
        ),
        (
            "Brush + Blur + Smear",
            vec![CompositeOp::Brush, CompositeOp::Blur, CompositeOp::Smear],
        ),
    ] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
        t.paint.brush.radius_px = 14.0;
        t.paint.brush.color = [0.6, 0.0, 0.0];
        t.paint.brush.space_attenuation = false;
        t.paint.composite_enabled = true;
        for pos in 0..3 {
            t.paint.composite[pos] = CompositeLayer {
                op: *ops.get(pos).unwrap_or(&CompositeOp::Brush),
                strength: if pos < ops.len() { 0.6 } else { 0.0 },
            };
        }
        t.paint.composite[0].strength = 1.0;
        drag(&mut t, 100.0, 30.0, 170.0);
        // A linha do EIXO satura; a costura mora no ombro, onde o valor varia devagar.
        let px = &t.canvas_rgba;
        let at = |x: u32| i32::from(px[((88 * SIZE + x) * 4) as usize]);
        let (mut worst, mut wx) = (0i32, 0u32);
        for x in 31..170 {
            let d = (at(x) - at(x - 1)).abs();
            if d > worst {
                worst = d;
                wx = x;
            }
        }
        eprintln!("{name}: maior degrau {worst} niveis em x={wx}");
    }
}

/// SONDA — a LARGURA DA RAMPA da borda (Enio 2026-08-09: *"margens duras e pixeladas"*).
///
/// ⚠️ **VEREDITO: a pilha está INOCENTE, e o controle é a prova.** Medido (`PROBE_PASSES=n`):
///
/// | passadas | Brush só | + Smear | + Blur | + Blur + Smear |
/// |---|---|---|---|---|
/// | 1 | 7 texels | 7 | 7 | 7 |
/// | 5 | 3 | 3 | 4 | 3 |
/// | 15 | **2** | 2 | 3 | 2 |
///
/// O **brush digital sozinho** endurece exatamente igual ⇒ o que a foto mostra é o **defeito ABERTO** que
/// a §13.10 do [doc 25](../../../../docs/Painter/25_avaliacao_gpu.md) já mede e nomeia
/// (`the_documented_hardening_is_still_there_and_this_is_its_number`), e **não** uma consequência do
/// Composite Brush nem da wave que consertou a dobra do smear.
///
/// ⚠️ **E isto fecha o Bug #16 como pista falsa para cá:** a cura dele (a fração como alpha LINEAR na
/// aparência) mora no render da AQUARELA, e a rota digital tem outra doença — não a saturação óptica
/// comendo o AA, mas o **produto por-dab afiando a cauda do falloff**. A §13.10 já tentou as duas leis de
/// acúmulo possíveis, e cada uma tem artefato (produto = endurece · envelope = CONTAS), então a próxima
/// hipótese **não pode ser uma terceira lei** — os candidatos que sobram estão escritos lá: o overlay, os
/// defaults do pincel (o Spacing governa quantos dabs por texel) ou aceitar o endurecimento.
///
/// ⚠️ **Por que a pilha o torna mais VISÍVEL sem o causar** (hipótese, não medida): o Smear e o Blur
/// re-tocam a MESMA região a cada batch, então um traço da pilha atravessa a mesma cauda de falloff
/// muitas vezes onde o Brush sozinho a atravessa uma. É a mesma aritmética das quinze passadas, dentro de
/// um gesto só.
///
/// O oráculo é o do Bug #16: quantos texels a borda leva para ir de tinta a papel. Uma borda honesta de
/// pincel tem rampa de ~0,4·raio; um degrau binário mede ~1. Medido no OMBRO de um traço, perpendicular
/// a ele, no MEIO (a ponta tem o taper e mediria outra coisa).
#[test]
fn probe_composite_edge_ramp() {
    const SIZE: u32 = 240;
    for (name, ops) in [
        ("Brush só (controle)", vec![CompositeOp::Brush]),
        (
            "Brush + Smear",
            vec![CompositeOp::Brush, CompositeOp::Smear],
        ),
        ("Brush + Blur", vec![CompositeOp::Brush, CompositeOp::Blur]),
        (
            "Brush + Blur + Smear",
            vec![CompositeOp::Brush, CompositeOp::Blur, CompositeOp::Smear],
        ),
    ] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
        t.paint.brush.radius_px = 20.0;
        t.paint.brush.color = [0.6, 0.0, 0.0];
        t.paint.brush.space_attenuation = false;
        t.paint.composite_enabled = true;
        for pos in 0..3 {
            t.paint.composite[pos] = CompositeLayer {
                op: *ops.get(pos).unwrap_or(&CompositeOp::Brush),
                strength: if pos < ops.len() { 0.6 } else { 0.0 },
            };
        }
        t.paint.composite[0].strength = 1.0;
        // ⚠️ A doença desta família é de REPETIÇÃO, não de um traço: a §13.10 mediu a banda indo de
        // 3,53 px numa passada para 1,38 em quinze. Uma passada só não a contém.
        let passes: u32 = std::env::var("PROBE_PASSES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        for _ in 0..passes {
            drag(&mut t, 120.0, 40.0, 200.0);
        }
        // Coluna no MEIO do traço, varrendo para fora: quantos texels entre 90% e 10% da tinta.
        let px = &t.canvas_rgba;
        let at = |y: u32| f32::from(255 - px[((y * SIZE + 120) * 4) as usize]) / 255.0;
        let peak = (100..120).map(at).fold(0.0f32, f32::max);
        let (mut hi, mut lo) = (None, None);
        for y in 120..170 {
            let v = at(y) / peak.max(1e-6);
            if hi.is_none() && v < 0.9 {
                hi = Some(y);
            }
            if lo.is_none() && v < 0.1 {
                lo = Some(y);
                break;
            }
        }
        match (hi, lo) {
            (Some(a), Some(b)) => eprintln!("{name}: rampa {} texels (90%..10%)", b - a),
            _ => eprintln!("{name}: rampa NAO MEDIDA (hi={hi:?} lo={lo:?}, peak={peak:.3})"),
        }
    }
}
