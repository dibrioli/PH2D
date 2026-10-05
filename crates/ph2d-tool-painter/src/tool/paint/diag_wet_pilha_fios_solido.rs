//! DIAGNÓSTICO — o preço do **Composite Brush**, dos **fios** e do **Solid** no Wet Paint (doc 46
//! itens 8 e 9; os kill-criteria do §1 escritos antes de qualquer build).
//!
//! Mede pela porta do produto (`on_canvas_pointer` + um `paint_tick` por evento, que é o quadro):
//!
//! | linha | o que corre | o que estima |
//! |---|---|---|
//! | `D pincel` · `D pilha 4` | Digital, uma camada · o Composite com Brush · Smear · Blur · Erase | a régua do kill-criterion |
//! | `W Paint/Smear/Blend/Erase` | cada ferramenta da água SOZINHA no mesmo gesto | a pilha na água ≥ a SOMA (cada camada é um despacho por carimbo) |
//! | `D fios` · `W fios` | o Sketchy denso no Digital · na água, pela porta da máscara (`wetpaint::mascara`) | a teia densa na água |
//! | `D solid` · `W solid` | o Solid no Digital · na água, pela porta da máscara | a mancha na água |
//!
//! ⛔ **As rotas recusadas, medidas aqui ANTES da porta** (2026-10-05, doc 46 itens 8–9): os fios como
//! carimbos de água de 1 px (os mesmos 23 769 fios, `2 194 425` carimbos) `283,6 ms` por quadro; o
//! Solid como um carimbo da área da região `1,155`; a pilha de quatro, pela soma das ferramentas,
//! `1,28×` a régua.
//!
//! ⛔ **A soma é um PISO, não uma previsão:** no Digital a pilha corre por CAMADA sobre o traço
//! inteiro (`composite_pilha`), e esse custo a mais não tem análogo medível sem o construir.
//!
//! O gesto é um RABISCO (espiral de 4 voltas que volta à própria vizinhança — o regime em que a
//! premissa da recomposição regional caiu, `composite_pilha` §refutada), num canvas de `1024²`, raio
//! `24`. As variantes correm INTERCALADAS ([`RODADAS`] rodadas, ordem rodada) e fica o MÍNIMO
//! (a mediana ao lado é o controlo); o `loadavg` sai ao lado.
//!
//! Rodar: `cargo test -p ph2d-tool-painter --profile smoke --lib diag_o_preco_da_agua -- --ignored
//! --nocapture` (uma compilação, sem esperar a máquina calma).
use super::*;
use ph2d_painter_brush::line_kind::{LineKind, SKETCHY_DENSITY_MAX};
use ph2d_painter_brush::stroke::threads::Thread;
use ph2d_painter_brush::{Stroke, StrokePoint};
use std::time::Instant;

const SIZE: u32 = 1024;
const RAIO: f32 = 24.0;
/// Rodadas INTERCALADAS (regra do dono, 2026-10-05: `docs/DevOps/MEDIR_VELOCIDADE.md`): cada rodada
/// corre um gesto de CADA variante, com a ordem rodada; fica-se com o MÍNIMO das rodadas.
const RODADAS: usize = 7;
const VOLTAS: usize = 4;
const PASSOS: usize = VOLTAS * 40;

fn ponteiro(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// O ponto `i` da espiral (raio 20 → 200 px em torno do centro).
fn ponto(i: usize) -> [f32; 2] {
    #[allow(clippy::cast_precision_loss)]
    let u = i as f32 / PASSOS as f32;
    #[allow(clippy::cast_precision_loss)]
    let ang = u * VOLTAS as f32 * std::f32::consts::TAU;
    let r = 20.0 + u * 180.0;
    let c = (SIZE / 2) as f32;
    [c + r * ang.cos(), c + r * ang.sin()]
}

fn ferramenta() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SIZE * SIZE * 4) as usize], SIZE, SIZE);
    t.paint.brush.radius_px = RAIO;
    t.paint.brush.color = [0.6, 0.0, 0.0];
    t.paint.brush.strength = 1.0;
    t.paint.brush.space_attenuation = false;
    t
}

fn molhada(t: &mut PainterTool, ferr: usize) {
    t.set_paint_tool_mode("wetpaint");
    t.pick_wet_tool(ferr);
}

/// Um gesto inteiro pela porta do produto; `extra(t, i)` corre depois do Move `i` (antes do quadro).
/// Devolve os ms do gesto (Down → Moves com o quadro → Up).
fn gesto(t: &mut PainterTool, extra: &mut dyn FnMut(&mut PainterTool, usize)) -> f64 {
    let t0 = Instant::now();
    t.on_canvas_pointer(ponteiro(ponto(0), PointerPhase::Down));
    for i in 1..=PASSOS {
        t.on_canvas_pointer(ponteiro(ponto(i), PointerPhase::Move));
        extra(t, i);
        t.paint_tick(1.0 / 60.0);
    }
    t.on_canvas_pointer(ponteiro(ponto(PASSOS), PointerPhase::Up));
    t0.elapsed().as_secs_f64() * 1e3
}

fn nada(_: &mut PainterTool, _: usize) {}

/// A tela depois de UM gesto — o CONTROLO de que a linha da tabela pinta o que diz.
fn tela(prep: &dyn Fn() -> PainterTool, extra: &mut dyn FnMut(&mut PainterTool, usize)) -> Vec<u8> {
    let mut t = prep();
    gesto(&mut t, extra);
    t.canvas_rgba.to_vec()
}

/// Píxeis que diferem entre duas telas.
fn diferem(a: &[u8], b: &[u8]) -> usize {
    a.as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0)
        .filter(|(x, y)| x != y)
        .count()
}

/// Uma camada de tinta molhada por baixo (fora do relógio), e depois a ferramenta `ferr`.
fn agua_com_tinta(ferr: usize) -> PainterTool {
    let mut t = ferramenta();
    molhada(&mut t, 0);
    gesto(&mut t, &mut nada);
    molhada(&mut t, ferr);
    t
}

fn sketchy(t: &mut PainterTool) {
    t.paint.brush.line_kind = LineKind::Sketchy;
    t.paint.brush.sketchy_reach = 3.0;
    t.paint.brush.sketchy_density = SKETCHY_DENSITY_MAX;
    t.paint.brush.thread_width_px = 1.0;
    t.paint.brush.thread_opacity = 0.25;
}

/// Os fios que o motor de traço costura em cada evento da espiral, com o pincel de `t`.
fn fios_por_evento(t: &PainterTool) -> Vec<Vec<Thread>> {
    let mut s = Stroke::new(t.stroke_spec(), t.paint.dynamics, 7);
    let mut dabs = Vec::new();
    let mut fios = Vec::new();
    s.begin(
        StrokePoint {
            pos: ponto(0),
            pressure: 1.0,
        },
        &mut dabs,
    );
    let mut por_evento = vec![Vec::new()];
    for i in 1..=PASSOS {
        dabs.clear();
        s.extend(
            StrokePoint {
                pos: ponto(i),
                pressure: 1.0,
            },
            &mut dabs,
        );
        s.take_threads(&mut fios);
        por_evento.push(fios.clone());
    }
    por_evento
}

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
        .unwrap_or_else(|_| "?".into())
}

fn pilha_cheia() -> PainterTool {
    let mut t = ferramenta();
    t.paint.composite_enabled = true;
    t.paint.composite_len = 4;
    for (pos, (op, s)) in [
        (CompositeOp::Blur, 0.5),
        (CompositeOp::Smear, 0.5),
        (CompositeOp::Erase, 0.3),
        (CompositeOp::Brush, 1.0),
    ]
    .into_iter()
    .enumerate()
    {
        t.paint.composite[pos] = CompositeLayer {
            op,
            strength: s,
            ..CompositeLayer::default()
        };
    }
    t
}

fn com_sketchy() -> PainterTool {
    let mut t = ferramenta();
    sketchy(&mut t);
    t
}

fn com_solid() -> PainterTool {
    let mut t = ferramenta();
    t.paint.brush.style_solid = true;
    t
}

fn agua_sketchy() -> PainterTool {
    let mut t = so_agua();
    sketchy(&mut t);
    t
}

fn agua_solid() -> PainterTool {
    let mut t = so_agua();
    t.paint.brush.style_solid = true;
    t
}

fn so_agua() -> PainterTool {
    let mut t = ferramenta();
    molhada(&mut t, 0);
    t
}

/// Os CONTROLOS: cada linha da tabela pinta o que diz. Sem relógio, logo valem sob carga.
fn controlos() {
    let d_base = tela(&ferramenta, &mut nada);
    let c_pilha = diferem(&d_base, &tela(&pilha_cheia, &mut nada));
    let c_dfios = diferem(&d_base, &tela(&com_sketchy, &mut nada));
    let c_dsolid = diferem(&d_base, &tela(&com_solid, &mut nada));
    let base = tela(&so_agua, &mut nada);
    let c_fios = diferem(&base, &tela(&agua_sketchy, &mut nada));
    let c_solid = diferem(&base, &tela(&agua_solid, &mut nada));
    // A tinta de baixo SOZINHA (sem o 2.º gesto): é contra ela que a ferramenta tem de mexer.
    let com_tinta = agua_com_tinta(0).canvas_rgba.to_vec();
    let c_smear = diferem(&com_tinta, &tela(&|| agua_com_tinta(2), &mut nada));
    let c_blend = diferem(&com_tinta, &tela(&|| agua_com_tinta(3), &mut nada));
    let c_erase = diferem(&com_tinta, &tela(&|| agua_com_tinta(1), &mut nada));
    eprintln!(
        "controlos (píxeis que mudam): D pilha {c_pilha} · D fios {c_dfios} · D solid {c_dsolid} · \
         W fios {c_fios} · W solid {c_solid} · W Smear {c_smear} · W Blend {c_blend} · W Erase \
         {c_erase}"
    );
    for (nome, n) in [
        ("D pilha", c_pilha),
        ("D fios", c_dfios),
        ("D solid", c_dsolid),
        ("W fios", c_fios),
        ("W solid", c_solid),
        ("W Smear", c_smear),
        ("W Blend", c_blend),
        ("W Erase", c_erase),
    ] {
        assert!(n > 0, "a linha «{nome}» mediria uma porta que não pinta");
    }
}

/// Os controlos sozinhos (sem relógio): `cargo test -p ph2d-tool-painter diag_os_controlos_da_agua
/// -- --ignored --nocapture`.
#[test]
#[ignore = "controlo da sonda de relógio abaixo"]
fn diag_os_controlos_da_agua() {
    controlos();
}

/// Uma variante da tabela: como armar a ferramenta (fora do relógio) e o que corre depois de cada Move.
struct Variante<'a> {
    nome: &'static str,
    prep: Box<dyn Fn() -> PainterTool + 'a>,
    extra: Box<Extra<'a>>,
}

/// O que corre depois de cada Move de uma variante.
type Extra<'a> = dyn FnMut(&mut PainterTool, usize) + 'a;

fn variante<'a>(
    nome: &'static str,
    prep: impl Fn() -> PainterTool + 'a,
    extra: impl FnMut(&mut PainterTool, usize) + 'a,
) -> Variante<'a> {
    Variante {
        nome,
        prep: Box::new(prep),
        extra: Box::new(extra),
    }
}

/// `(mínimo, mediana)` em ms POR QUADRO de cada variante, sobre [`RODADAS`] rodadas intercaladas com a
/// ordem rodada (a rodada `r` começa na variante `r`), para a deriva da máquina cair em todas.
fn intercalado(vs: &mut [Variante<'_>]) -> Vec<(f64, f64)> {
    let n = vs.len();
    let mut ms = vec![Vec::with_capacity(RODADAS); n];
    #[allow(clippy::cast_precision_loss)]
    let quadros = PASSOS as f64;
    for r in 0..RODADAS {
        for k in 0..n {
            let v = &mut vs[(r + k) % n];
            let mut t = (v.prep)();
            ms[(r + k) % n].push(gesto(&mut t, &mut *v.extra) / quadros);
        }
    }
    ms.into_iter()
        .map(|mut m| {
            m.sort_by(f64::total_cmp);
            (m[0], m[m.len() / 2])
        })
        .collect()
}

/// SONDA — as três peças do Wet Paint contra o kill-criterion, numa rodada só, INTERCALADA.
#[test]
#[ignore = "diagnóstico de relógio: roda sob demanda, no perfil smoke"]
fn diag_o_preco_da_agua() {
    controlos();
    let n_fios: usize = fios_por_evento(&com_sketchy()).iter().map(Vec::len).sum();
    let antes = carga();
    let mut vs = [
        variante("D pincel", ferramenta, nada),
        variante("D pilha 4", pilha_cheia, nada),
        variante("D fios", com_sketchy, nada),
        variante("D solid", com_solid, nada),
        variante("W Paint", so_agua, nada),
        variante("W Smear", || agua_com_tinta(2), nada),
        variante("W Blend", || agua_com_tinta(3), nada),
        variante("W Erase", || agua_com_tinta(1), nada),
        // O PRODUTO (doc 46 item 9): o Sketchy denso e o Solid na água, pela porta da máscara.
        variante("W fios", agua_sketchy, nada),
        variante("W solid", agua_solid, nada),
    ];
    let r = intercalado(&mut vs);
    let m = |nome: &str| r[vs.iter().position(|v| v.nome == nome).expect("variante")].0;
    eprintln!(
        "loadavg antes {antes} · depois {} · canvas {SIZE}² · raio {RAIO} · espiral {VOLTAS} voltas \
         ({PASSOS} quadros) · {RODADAS} rodadas intercaladas · {n_fios} fios no gesto",
        carga()
    );
    eprintln!("ms POR QUADRO          mínimo   (mediana)");
    for (v, (min, med)) in vs.iter().zip(&r) {
        eprintln!("  {:<18} {min:>8.3}   ({med:.3})", v.nome);
    }
    let regua = m("D pilha 4");
    let soma = m("W Paint") + m("W Smear") + m("W Blend") + m("W Erase");
    eprintln!("kill-criterion (régua = D pilha 4 = {regua:.3} ms):");
    eprintln!("  W soma das 4 (piso) {soma:>8.3} → {:.2}×", soma / regua);
    eprintln!(
        "  W fios              {:>8.3} → {:.2}× a régua · {:.2}× o D fios",
        m("W fios"),
        m("W fios") / regua,
        m("W fios") / m("D fios")
    );
    eprintln!(
        "  W solid             {:>8.3} → {:.2}× a régua · {:.2}× o D solid",
        m("W solid"),
        m("W solid") / regua,
        m("W solid") / m("D solid")
    );
}
