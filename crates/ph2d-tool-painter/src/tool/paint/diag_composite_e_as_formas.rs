//! SONDA — **a pilha do Composite contra os métodos de RE-CARIMBO** (report do dono, 2026-09-21:
//! *"os Stroke:Method vivos (booleanos) — Ellipse, Polygon, Line — não funcionam corretamente com o
//! composite, mudam de aparência e o Boolean não funciona"* + *"undo/redo … podem deixar resíduos"*).
//!
//! ⚠️ **A ORDEM da pilha é ingrediente da fixtura:** [`PainterTool::acrescenta_camada`] põe a camada
//! nova no FUNDO, logo a PRIMEIRA que se cria é a de CIMA. Uma pilha `Brush` + `Blur` nesta ordem
//! tem o borrão por BAIXO da tinta — ele borra a tela em branco e não faz nada, e a 1ª redacção
//! desta sonda leu essa configuração como *"o Blur não funciona"*.

use super::*;
use ph2d_editor_core::tool::RasterEditTool;

const S: u32 = 256;

/// A pilha de cada coluna, **de cima para baixo** (a ordem em que se criam).
type Pilha<'a> = &'a [(composite::CompositeOp, f32)];

fn cena(composite: bool, camadas: Pilha<'_>) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (S * S * 4) as usize], S, S);
    t.paint.brush.radius_px = 3.0;
    t.paint.brush.hardness = 1.0;
    t.paint.brush.color = [0.0, 0.0, 0.0];
    t.paint.brush.space_attenuation = false;
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
    t.paint.composite_enabled = composite;
    for (op, s) in camadas {
        t.acrescenta_camada(op.to_u8());
        let pos = t.composite_len() - 1;
        t.set_composite_layer_strength(pos, *s);
    }
    t
}

fn tinta(t: &PainterTool) -> usize {
    (0..(S * S) as usize)
        .filter(|&i| t.canvas_rgba[i * 4] < 250)
        .count()
}

/// Uma impressão da tela — *contar texels escuros não distingue duas imagens com a mesma área*.
fn soma(t: &PainterTool) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in t.canvas_rgba.iter() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    h
}

pub(super) fn cp2(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Desenha uma elipse centrada em `c` de raio final `r`, **arrastando** em `moves` passos — que é o
/// que a mão faz, e cada passo é um re-carimbo da figura inteira sobre outra geometria.
fn elipse(t: &mut PainterTool, c: [f32; 2], r: f32, moves: usize) {
    t.on_canvas_pointer(cp2(c, PointerPhase::Down));
    for i in 1..=moves {
        let u = i as f32 / moves as f32;
        t.on_canvas_pointer(cp2([c[0] + r * u, c[1]], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Up));
}

fn estado(t: &PainterTool) -> String {
    format!(
        "activo={} len={} forcas={:?}",
        t.composite_active(),
        t.composite_len(),
        (0..t.composite_len())
            .map(|i| t.paint.composite[i].strength)
            .collect::<Vec<_>>()
    )
}

/// As pilhas medidas — **a ordem é de CIMA para baixo**.
/// Uma coluna da tabela: o rótulo, se a pilha está ligada, e as camadas de CIMA para baixo.
type Coluna = (&'static str, bool, Vec<(composite::CompositeOp, f32)>);

fn pilhas() -> Vec<Coluna> {
    use composite::CompositeOp::{Blur, Brush, Erase, Smear};
    vec![
        ("sem pilha            ", false, vec![]),
        ("1: Brush             ", true, vec![(Brush, 1.0)]),
        (
            "2: Blur / Brush      ",
            true,
            vec![(Blur, 1.0), (Brush, 1.0)],
        ),
        (
            "2: Smear / Brush     ",
            true,
            vec![(Smear, 1.0), (Brush, 1.0)],
        ),
        (
            "2: Brush / Brush     ",
            true,
            vec![(Brush, 0.5), (Brush, 0.5)],
        ),
        (
            "2: Erase / Brush     ",
            true,
            vec![(Erase, 0.4), (Brush, 1.0)],
        ),
        (
            "3: Blur/Smear/Brush  ",
            true,
            vec![(Blur, 1.0), (Smear, 1.0), (Brush, 1.0)],
        ),
    ]
}

/// (1) O re-carimbo é IDEMPOTENTE? A mesma figura desenhada com 1 e com 20 movimentos tem de dar a
/// MESMA tela — o preview restaura e re-carimba, logo o número de eventos não pode aparecer.
#[test]
#[ignore = "sonda de diagnóstico"]
fn diag_o_recarimbo_e_idempotente() {
    for (nome, comp, camadas) in pilhas() {
        let mut linha = String::new();
        for moves in [1usize, 2, 5, 20] {
            super::composite_pilha::CONTA_DA_PILHA.with(|c| c.set((0, 0, 0)));
            let mut t = cena(comp, &camadas);
            elipse(&mut t, [128.0, 128.0], 60.0, moves);
            let (ev, _, _) = super::composite_pilha::CONTA_DA_PILHA.with(std::cell::Cell::get);
            linha.push_str(&format!(
                " {moves:>2}m=[{:>5}t {:012x} ev={ev:>2}]",
                tinta(&t),
                soma(&t) & 0xffff_ffff_ffff
            ));
        }
        eprintln!("{nome} |{linha}");
    }
}

/// (2) O BOOLEAN: duas circunferências `Add` que se cruzam — o arco interior tem de SUMIR.
#[test]
#[ignore = "sonda de diagnóstico"]
fn diag_o_boolean_com_a_pilha() {
    let c1 = [100.0f32, 128.0];
    let c2 = [156.0f32, 128.0];
    let r = 45.0f32;
    // Um ponto do contorno de 1 BEM dentro de 2 (o arco que o boolean apaga): ângulo 20° para o lado
    // de 2, logo d1 = r e d2 < r com folga.
    let ang = 20.0f32.to_radians();
    let interior = [c1[0] + r * ang.cos(), c1[1] + r * ang.sin()];
    for (nome, comp, camadas) in pilhas() {
        let mut t = cena(comp, &camadas);
        t.set_stroke_op_mode(1); // Add
        elipse(&mut t, c1, r, 6);
        elipse(&mut t, c2, r, 6);
        let d2 = ((interior[0] - c2[0]).powi(2) + (interior[1] - c2[1]).powi(2)).sqrt();
        // A tinta mais escura numa janela 5×5 à volta do ponto do arco interior.
        let mut pior = 255u8;
        for dy in -2i32..=2 {
            for dx in -2i32..=2 {
                let x = (interior[0] as i32 + dx).clamp(0, S as i32 - 1) as u32;
                let y = (interior[1] as i32 + dy).clamp(0, S as i32 - 1) as u32;
                pior = pior.min(t.canvas_rgba[((y * S + x) * 4) as usize]);
            }
        }
        eprintln!(
            "{nome} | parqueadas={} tinta={:>5} arco interior (d2={d2:.1} < r={r:.0}) mais escuro={pior:>3}",
            t.paint.parked_shapes.len(),
            tinta(&t),
        );
    }
}

/// (3) O RESÍDUO: desfazer TUDO tem de devolver a tela BRANCA.
#[test]
#[ignore = "sonda de diagnóstico"]
fn diag_o_residuo_do_undo() {
    for (nome, comp, camadas) in pilhas() {
        let mut t = cena(comp, &camadas);
        let limpo = soma(&t);
        elipse(&mut t, [128.0, 128.0], 60.0, 6);
        let vivo = tinta(&t);
        t.commit_open_shape();
        let aplicado = tinta(&t);
        let mut n = 0;
        while t.undo_last() && n < 40 {
            n += 1;
        }
        eprintln!(
            "{nome} | vivo={vivo:>5} aplicado={aplicado:>5} undos={n:>2} sobra={:>5} tela_limpa={}",
            tinta(&t),
            soma(&t) == limpo,
        );
        let _ = estado(&t);
    }
}

/// (4) O RESÍDUO de DUAS figuras booleanas — o caminho da foto do dono.
#[test]
#[ignore = "sonda de diagnóstico"]
fn diag_o_residuo_de_duas_figuras() {
    for (nome, comp, camadas) in pilhas() {
        let mut t = cena(comp, &camadas);
        let limpo = soma(&t);
        t.set_stroke_op_mode(1);
        elipse(&mut t, [100.0, 128.0], 45.0, 6);
        elipse(&mut t, [156.0, 128.0], 45.0, 6);
        t.commit_open_shape();
        let aplicado = tinta(&t);
        let mut n = 0;
        while t.undo_last() && n < 40 {
            n += 1;
        }
        eprintln!(
            "{nome} | aplicado={aplicado:>5} undos={n:>2} sobra={:>5} tela_limpa={}",
            tinta(&t),
            soma(&t) == limpo,
        );
    }
}

/// (5) O RASTO: desenhar UMA figura e **arrastá-la** — a tela tem de mostrar UMA figura, no destino.
#[test]
#[ignore = "sonda de diagnóstico"]
fn diag_o_rasto_ao_arrastar_a_figura() {
    for (nome, comp, camadas) in pilhas() {
        let mut t = cena(comp, &camadas);
        elipse(&mut t, [90.0, 128.0], 40.0, 6);
        let uma = tinta(&t);
        // Agarrar o centro e levar a figura 70 px para a direita, em 7 passos.
        t.on_canvas_pointer(cp2([90.0, 128.0], PointerPhase::Down));
        for i in 1..=7 {
            t.on_canvas_pointer(cp2([90.0 + i as f32 * 10.0, 128.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp2([160.0, 128.0], PointerPhase::Up));
        let depois = tinta(&t);
        eprintln!(
            "{nome} | uma figura={uma:>5}  depois de arrastar={depois:>5}  excesso={:+.0}%",
            (depois as f64 / uma as f64 - 1.0) * 100.0
        );
    }
}

/// (6) UM undo de cada vez, depois de duas figuras booleanas aplicadas.
#[test]
#[ignore = "sonda de diagnóstico"]
fn diag_undo_passo_a_passo() {
    for (nome, comp, camadas) in pilhas() {
        let mut t = cena(comp, &camadas);
        let limpo = soma(&t);
        t.set_stroke_op_mode(1);
        elipse(&mut t, [100.0, 128.0], 45.0, 6);
        elipse(&mut t, [156.0, 128.0], 45.0, 6);
        t.commit_open_shape();
        let mut passos = vec![tinta(&t)];
        for _ in 0..4 {
            if !t.undo_last() {
                break;
            }
            passos.push(tinta(&t));
        }
        eprintln!(
            "{nome} | {passos:?}  tela_limpa_no_fim={}",
            soma(&t) == limpo
        );
    }
}

/// (7) DEPOIS DO APPLY: a figura aplicada tem de SOBREVIVER à figura seguinte.
#[test]
#[ignore = "sonda de diagnóstico"]
fn diag_a_figura_aplicada_sobrevive_a_seguinte() {
    // As duas SOBREPÕEM-SE: é ali que a caixa da 2ª cobre pixels que a 1ª deixou.
    for (nome, comp, camadas) in pilhas() {
        let mut t = cena(comp, &camadas);
        elipse(&mut t, [100.0, 128.0], 40.0, 4);
        t.commit_open_shape();
        let primeira = tinta(&t);
        elipse(&mut t, [150.0, 128.0], 40.0, 4);
        let duas = tinta(&t);
        eprintln!(
            "{nome} | 1ª aplicada={primeira:>5}  com a 2ª aberta={duas:>5}  (esperado ≈ 2×) \
             perdeu_a_1ª={}",
            duas < primeira + primeira / 2
        );
    }
}

/// (8) O CICLO DE VIDA da pilha durante uma sessão de figuras — *o que a composição chama de `pre`*.
#[test]
#[ignore = "sonda de diagnóstico"]
fn diag_o_ciclo_de_vida_da_pilha() {
    use composite::CompositeOp::{Blur, Brush};
    let mut t = cena(true, &[(Blur, 1.0), (Brush, 1.0)]);
    t.set_stroke_op_mode(1);
    let branco = 255u8;
    let conta_pre = |t: &PainterTool| -> (usize, usize) {
        let n = t.paint.pilha.pre.len() / 4;
        let escuros = (0..n).filter(|&i| t.paint.pilha.pre[i * 4] < 250).count();
        (t.paint.pilha.pre.len(), escuros)
    };
    let planos = |t: &PainterTool| -> Vec<usize> {
        t.paint
            .pilha
            .planos
            .iter()
            .map(|p| p.len())
            .filter(|&l| l > 0)
            .collect()
    };
    let passo = |t: &PainterTool, o: &str| {
        let (len, esc) = conta_pre(t);
        eprintln!(
            "  {o:<24} tela={:>5}  pre.len={len:>7} pre_escuros={esc:>5} planos={:?} preview={}",
            tinta(t),
            planos(t),
            t.paint.drag_preview.is_some(),
        );
    };
    let _ = branco;
    passo(&t, "inicio");
    t.on_canvas_pointer(cp2([100.0, 128.0], PointerPhase::Down));
    passo(&t, "down 1");
    t.on_canvas_pointer(cp2([145.0, 128.0], PointerPhase::Move));
    passo(&t, "move 1");
    t.on_canvas_pointer(cp2([145.0, 128.0], PointerPhase::Up));
    passo(&t, "up 1");
    t.on_canvas_pointer(cp2([156.0, 128.0], PointerPhase::Down));
    passo(&t, "down 2");
    t.on_canvas_pointer(cp2([201.0, 128.0], PointerPhase::Move));
    passo(&t, "move 2");
    t.on_canvas_pointer(cp2([201.0, 128.0], PointerPhase::Up));
    passo(&t, "up 2");
}

/// (9) Os métodos de re-carimbo que pintam SOB A MÃO (Drag Dot · Anchored · Line): eles não passam
/// pelo rascunho, logo cada movimento é um carimbo de verdade — e é ali que o rasto aparece.
#[test]
#[ignore = "sonda de diagnóstico"]
fn diag_os_metodos_que_pintam_sob_a_mao() {
    use ph2d_painter_brush::StrokeMethod::{Anchored, DragDot, Line};
    for (rotulo, metodo) in [
        ("DragDot", DragDot),
        ("Anchored", Anchored),
        ("Line ", Line),
    ] {
        for (nome, comp, camadas) in pilhas() {
            let mut t = cena(comp, &camadas);
            t.paint.brush.stroke_method = metodo;
            t.paint.brush.radius_px = 8.0;
            // Um arrasto de 90 px em 9 passos: a figura ANDA, logo um acumulador que não é
            // descascado pinta as nove posições.
            t.on_canvas_pointer(cp2([60.0, 128.0], PointerPhase::Down));
            for i in 1..=9 {
                t.on_canvas_pointer(cp2([60.0 + i as f32 * 10.0, 128.0], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp2([150.0, 128.0], PointerPhase::Up));
            eprintln!("{rotulo} | {nome} | tinta={:>5}", tinta(&t));
        }
        eprintln!();
    }
}

/// (10) ⚠️ **A SUSPEITA:** o `peel_drag_preview` também é chamado pelo traço À MÃO LIVRE em
/// `Style: Solid` — ali cada LOTE descasca. Se o descascar apagasse a pilha, um traço cumulativo
/// mostraria só o último lote.
#[test]
#[ignore = "sonda de diagnóstico"]
fn diag_o_solid_a_mao_livre_com_a_pilha() {
    for solid in [false, true] {
        for (nome, comp, camadas) in pilhas() {
            let mut t = cena(comp, &camadas);
            t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Space;
            t.paint.brush.style_solid = solid;
            t.paint.brush.radius_px = 6.0;
            let ramo = t.freehand_solid_fill_live();
            t.on_canvas_pointer(cp2([40.0, 128.0], PointerPhase::Down));
            for i in 1..=20 {
                t.on_canvas_pointer(cp2([40.0 + i as f32 * 8.0, 128.0], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp2([200.0, 128.0], PointerPhase::Up));
            eprintln!(
                "solid={solid} | {nome} | tinta={:>5} {:012x} ramo_solid={}",
                tinta(&t),
                soma(&t) & 0xffff_ffff_ffff,
                ramo
            );
        }
    }
}

/// (11) As duas metades da cura que os gates do re-carimbo NÃO discriminam: o **cap de Accumulate**
/// (só observável com `strength < 1`) e o **acumulador de ARCO** (só com uma camada MAIOR).
/// Aqui a figura é **re-carimbada NO MESMO SÍTIO** (um empurrão de 2 px), que é o regime em que os
/// dois mordem — arrastá-la para longe põe o carimbo em pixels virgens, onde o cap ainda é zero.
#[test]
#[ignore = "sonda de diagnóstico"]
fn diag_o_cap_e_o_arco_num_recarimbo_no_mesmo_sitio() {
    use composite::CompositeOp::Brush;
    for (rotulo, forca, tamanho) in [
        ("cap  (strength 0,5 · size 1)", 0.5f32, 1.0f32),
        ("arco (strength 1,0 · size 3)", 1.0, 3.0),
    ] {
        // Uma figura desenhada UMA vez, como referência.
        let mut ref_ = cena(true, &[(Brush, forca), (Brush, forca)]);
        t_tamanho(&mut ref_, tamanho);
        elipse(&mut ref_, [128.0, 128.0], 50.0, 4);
        let (a_ref, escuro_ref) = (tinta(&ref_), min_canal(&ref_));

        // A mesma figura, e depois empurrada 2 px — ela é RE-CARIMBADA quase no mesmo sítio.
        let mut t = cena(true, &[(Brush, forca), (Brush, forca)]);
        t_tamanho(&mut t, tamanho);
        elipse(&mut t, [128.0, 128.0], 50.0, 4);
        t.on_canvas_pointer(cp2([128.0, 128.0], PointerPhase::Down));
        t.on_canvas_pointer(cp2([130.0, 128.0], PointerPhase::Move));
        t.on_canvas_pointer(cp2([130.0, 128.0], PointerPhase::Up));
        eprintln!(
            "{rotulo} | uma vez: tinta={a_ref:>5} min={escuro_ref:>3}  |  re-carimbada: \
             tinta={:>5} min={:>3}",
            tinta(&t),
            min_canal(&t)
        );
    }
}

fn t_tamanho(t: &mut PainterTool, tamanho: f32) {
    for pos in 0..t.composite_len() {
        t.set_composite_layer_size(pos, tamanho);
    }
}

fn min_canal(t: &PainterTool) -> u8 {
    (0..(S * S) as usize)
        .map(|i| t.canvas_rgba[i * 4])
        .min()
        .unwrap_or(255)
}

/// (12) O `Ctrl+Z` **a meio** de uma sessão de figuras (sem Apply): desfazer a 2.ª figura tem de
/// deixar a tela igual à que tinha só a 1.ª.
#[test]
#[ignore = "sonda de diagnóstico"]
fn diag_undo_a_meio_de_uma_sessao() {
    for (nome, comp, camadas) in pilhas() {
        let mut t = cena(comp, &camadas);
        elipse(&mut t, [90.0, 128.0], 40.0, 4);
        let so_a_primeira = soma(&t);
        let t1 = tinta(&t);
        elipse(&mut t, [170.0, 128.0], 40.0, 4);
        let duas = tinta(&t);
        let mut n = 0;
        // Desfaz até a tela voltar a ter só a 1.ª figura (ou até desistir).
        while n < 6 && soma(&t) != so_a_primeira {
            if !t.undo_last() {
                break;
            }
            n += 1;
        }
        eprintln!(
            "{nome} | 1ª={t1:>5} duas={duas:>5} undos={n} → {:>5}  volta_exacta={}",
            tinta(&t),
            soma(&t) == so_a_primeira
        );
    }
}

/// (13) RENDER-AND-LOOK do report de 2026-09-21: *«2 círculos com o mesmo pincel e um está
/// diferente do outro»*. O oráculo é a FOTO, logo a sonda desenha e grava PNG.
/// `PH2D_FORMAS_LOOK_DIR=/tmp/look cargo test -p ph2d-tool-painter --lib diag_look_duas_figuras
/// -- --ignored --nocapture`
#[test]
#[ignore = "render-and-look: escreve PNG"]
fn diag_look_duas_figuras() {
    use composite::CompositeOp::{Blur, Brush, Smear};
    const L: u32 = 512;
    let dir = std::env::var("PH2D_FORMAS_LOOK_DIR").unwrap_or_else(|_| "/tmp/look".into());
    std::fs::create_dir_all(&dir).unwrap();
    for (nome, camadas) in [
        ("a_so_brush", &[(Brush, 1.0f32)][..]),
        ("b_blur_brush", &[(Blur, 1.0), (Brush, 1.0)][..]),
        ("c_brush_blur", &[(Brush, 1.0), (Blur, 1.0)][..]),
        (
            "d_blur_smear_brush",
            &[(Blur, 1.0), (Smear, 1.0), (Brush, 1.0)][..],
        ),
    ] {
        let mut t = PainterTool::default();
        // ⚠️ **A camada do artista é TRANSPARENTE.** Numa tela opaca branca o Blur não tem
        // vizinho vazio de onde puxar, e a foto do dono mostra exactamente uma orla ESCURA.
        // ⚠️⚠️ **Uma camada VAZIA é tudo ZERO** — RGB preto com alfa 0, e não branco transparente.
        // A diferença decide o que um Blur puxa da vizinhança.
        t.set_source(vec![0u8; (L * L * 4) as usize], L, L);
        t.paint.brush.radius_px = 12.0;
        t.paint.brush.color = [0.75, 0.12, 0.12];
        t.paint.brush.space_attenuation = false;
        t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
        t.paint.composite_enabled = true;
        for (op, s) in camadas {
            t.acrescenta_camada(op.to_u8());
            let pos = t.composite_len() - 1;
            t.set_composite_layer_strength(pos, *s);
        }
        let mut circulo = |c: [f32; 2], r: f32| {
            t.on_canvas_pointer(cp2(c, PointerPhase::Down));
            for i in 1..=6 {
                let u = i as f32 / 6.0;
                t.on_canvas_pointer(cp2([c[0] + r * u, c[1]], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Up));
        };
        circulo([180.0, 170.0], 120.0);
        circulo([320.0, 320.0], 140.0);
        // Compor sobre BRANCO para ver o que o artista vê (o PNG cru mostraria a transparência).
        let sobre_branco: Vec<u8> = t
            .canvas_rgba
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| {
                let a = f32::from(p[3]) / 255.0;
                let c = |i: usize| (f32::from(p[i]) + 255.0 * (1.0 - a)).min(255.0) as u8;
                [c(0), c(1), c(2), 255]
            })
            .collect();
        let png = super::super::composite_look::png_rgba(&sobre_branco, L, L);
        let caminho = format!("{dir}/{nome}.png");
        std::fs::write(&caminho, png).unwrap();
        // E o NÚMERO ao lado da imagem: a fatia mais escura de cada anel.
        eprintln!("{nome:<20} escrito {caminho}");
    }
}

/// (14) O A/B que ISOLA o mecanismo do report *«um está diferente do outro»*: a MESMA figura,
/// sozinha e depois com uma segunda **LONGE** dela. Se os pixels dela mudarem, a causa é a
/// concatenação das duas listas de dabs num lote só (a activa + as parqueadas).
#[test]
#[ignore = "sonda de diagnóstico"]
fn diag_a_primeira_figura_muda_quando_nasce_a_segunda() {
    use composite::CompositeOp::{Blur, Brush, Erase, Smear};
    let a = [70.0f32, 70.0];
    let b = [190.0f32, 190.0]; // longe: os dois anéis não se tocam
    let r = 40.0f32;
    // A janela que contém SÓ a 1.ª figura.
    let janela = |t: &PainterTool| -> (usize, u32, u64) {
        let (mut n, mut pior, mut soma_) = (0usize, 255u8, 0u64);
        for y in 0..130u32 {
            for x in 0..130u32 {
                let i = ((y * S + x) * 4) as usize;
                let v = t.canvas_rgba[i + 3]; // o ALFA: a camada nasce vazia
                if v > 4 {
                    n += 1;
                    soma_ += u64::from(v);
                }
                pior = pior.min(t.canvas_rgba[i]);
            }
        }
        (n, u32::from(pior), soma_)
    };
    for (nome, camadas) in [
        ("1: Brush            ", &[(Brush, 1.0f32)][..]),
        ("2: Blur / Brush     ", &[(Blur, 1.0), (Brush, 1.0)][..]),
        ("2: Smear / Brush    ", &[(Smear, 1.0), (Brush, 1.0)][..]),
        ("2: Brush / Brush    ", &[(Brush, 1.0), (Brush, 1.0)][..]),
        ("2: Erase / Brush    ", &[(Erase, 0.4), (Brush, 1.0)][..]),
        (
            "3: Blur/Smear/Brush ",
            &[(Blur, 1.0), (Smear, 1.0), (Brush, 1.0)][..],
        ),
    ] {
        let vazia = |camadas: &[(composite::CompositeOp, f32)]| {
            let mut t = PainterTool::default();
            t.set_source(vec![0u8; (S * S * 4) as usize], S, S);
            t.paint.brush.radius_px = 6.0;
            t.paint.brush.color = [0.75, 0.12, 0.12];
            t.paint.brush.space_attenuation = false;
            t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
            t.paint.composite_enabled = true;
            for (op, s) in camadas {
                t.acrescenta_camada(op.to_u8());
                let pos = t.composite_len() - 1;
                t.set_composite_layer_strength(pos, *s);
            }
            t
        };
        let mut so_a = vazia(camadas);
        elipse(&mut so_a, a, r, 4);
        let (n1, p1, s1) = janela(&so_a);

        let mut as_duas = vazia(camadas);
        elipse(&mut as_duas, a, r, 4);
        elipse(&mut as_duas, b, r, 4);
        let (n2, p2, s2) = janela(&as_duas);
        eprintln!(
            "{nome} | sozinha: n={n1:>5} min={p1:>3} soma={s1:>8}  |  com a 2ª: n={n2:>5} \
             min={p2:>3} soma={s2:>8}  {}",
            if (n1, p1, s1) == (n2, p2, s2) {
                "IGUAL"
            } else {
                "*** DIFERENTE ***"
            }
        );
    }
}
