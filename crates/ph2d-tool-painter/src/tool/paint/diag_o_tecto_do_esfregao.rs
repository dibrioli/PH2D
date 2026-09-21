//! SONDA — **o ESCOPO da perda do esfregão, e o TECTO que foi recusado** (2026-09-21).
//!
//! Irmã de [`super::diag_a_lei_do_esfregao`], cortada dela pelo tecto de LOC e melhor por isso:
//! lá caça-se a CAUSA, aqui mede-se o que ela ALCANÇA e o que custa curá-la.
//!
//! | # | pergunta | veredito |
//! |---|---|---|
//! | A8 | a FONTE ou a LEI? | **a lei**: um anel JÁ pintado perde os mesmos `15,6 %`; a recta `0,1 %` |
//! | A9 | discretização? | ⛔ **refutada** — refinar o passo `8×` recupera `4` pontos |
//! | A10 | o cisalhamento do peso? | ⛔ refutada, e ao contrário: a dureza `1` piora para `51 %` |
//! | A12 | o tecto mexe na recta? | não — pior coluna `0,07 %`, contagem de texels idêntica |
//! | A13 | qual tecto? | o joelho MEDIDO coincide com o `2 R` DERIVADO |
//!
//! ⛔⛔⛔ **E o veredito de produto é uma RECUSA:** o tecto mata o transporte longo que o dono
//! exigiu duas vezes. A troca inteira está em
//! [`ph2d_painter_brush::TECTO_MEDIDO_E_RECUSADO_EM_RAIOS`], e os gates que a prendem em
//! [`super::super::smear_curva_tests`].

use super::diag_composite_e_as_formas::cp2;
use super::*;
use ph2d_editor_core::tool::RasterEditTool;

const L: u32 = 700;

/// (A8) O ESCOPO — a tinta morre porque a FONTE está incompleta, ou porque a LEI a mata?
///
/// No lote da pilha a camada `Brush` deposita e a `Smear` re-amostra logo a seguir. Duas leituras:
///   · a fonte do esfregão ainda não tem o anel todo quando ele corre ⇒ defeito da PILHA;
///   · a lei mata tinta em qualquer caminho CURVO ⇒ defeito do ESFREGÃO, que é partilhado com a
///     ferramenta de canvas — curá-lo muda TODA pincelada deste app, e isso é decisão do dono.
///
/// ⚠️ **A 1.ª redacção desta sonda usava a Ellipse e leu `0,0 %`, e era a FIXTURA:** um 2.º gesto
/// sobre uma figura viva **re-edita a mesma figura**, logo o descascar repõe a tela de antes dela
/// existir e, com o pincel a Strength 0, não há anel nenhum para esfregar. *Um método de
/// re-carimbo não consegue exprimir «esfrega o que já lá está».* À mão livre (`Space`) não há
/// descascar, e o gesto 2 vê o anel do gesto 1.
///
/// O CONTROLO é a RECTA: mesmo comprimento, mesmo pincel, curvatura zero.
///
/// `cargo test -p ph2d-tool-painter --release --lib diag_a_fonte_ou_a_lei -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição: imprime uma tabela"]
fn diag_a_fonte_ou_a_lei() {
    use composite::CompositeOp::{Brush, Smear};
    let c = [350.0f32, 350.0];

    let alfa = |t: &PainterTool| -> f64 {
        (0..(L * L) as usize)
            .map(|i| f64::from(t.canvas_rgba[i * 4 + 3]))
            .sum::<f64>()
            / 255.0
    };

    // O caminho, à mão livre, em passos de ~3 px.
    let caminho_circulo = |r: f32| -> Vec<[f32; 2]> {
        let n = (std::f32::consts::TAU * r / 3.0) as usize;
        (0..=n)
            .map(|i| {
                let a = (i as f32) * std::f32::consts::TAU / (n as f32);
                [c[0] + r * a.cos(), c[1] + r * a.sin()]
            })
            .collect()
    };
    let caminho_recta = |comp: f32| -> Vec<[f32; 2]> {
        let n = (comp / 3.0) as usize;
        (0..=n)
            .map(|i| [60.0 + comp * (i as f32) / (n as f32), c[1]])
            .collect()
    };

    for (nome, caminho) in [
        ("circulo r=215", caminho_circulo(215.0)),
        ("RECTA (controlo)", caminho_recta(580.0)),
    ] {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; (L * L * 4) as usize], L, L);
        t.paint.brush.radius_px = 30.0;
        t.paint.brush.color = [0.75, 0.12, 0.12];
        t.paint.brush.space_attenuation = false;
        t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Space;
        t.paint.composite_enabled = true;
        for (op, s) in [(Smear, 0.0f32), (Brush, 1.0)] {
            t.acrescenta_camada(op.to_u8());
            let pos = t.composite_len() - 1;
            t.set_composite_layer_strength(pos, s);
        }
        let gesto = |t: &mut PainterTool, cam: &[[f32; 2]]| {
            t.on_canvas_pointer(cp2(cam[0], PointerPhase::Down));
            for p in &cam[1..] {
                t.on_canvas_pointer(cp2(*p, PointerPhase::Move));
            }
            t.on_canvas_pointer(cp2(cam[cam.len() - 1], PointerPhase::Up));
        };

        gesto(&mut t, &caminho);
        let so_tinta = alfa(&t);
        t.set_composite_layer_strength(0, 1.0);
        t.set_composite_layer_strength(1, 0.0);
        gesto(&mut t, &caminho);
        let depois = alfa(&t);
        println!(
            "  {nome:>18} · tinta {so_tinta:8.0} → depois do esfregao {depois:8.0}   \
guardado {:6.1} %",
            100.0 * depois / so_tinta.max(1.0)
        );
    }
}

/// (A9) ERRO DE DISCRETIZAÇÃO ou ESTRUTURAL? — o discriminador clássico: refinar o passo.
///
/// A composição `disp_new(p) = v(p) + disp_old(p − v(p))` é um traçado PARA TRÁS ao longo do
/// fluxo. Se ele fosse exacto, o ponto de origem cairia **no anel** e não se perderia tinta —
/// o que a (A8) mede é que ele deriva `36 px` para fora. Duas leituras, com curas opostas:
///   · **discretização** — o erro por passo é `O(v²·κ)` e some ao refinar ⇒ a cura é um integrador
///     melhor, e uma recta fica byte-idêntica por construção (ali `κ = 0`);
///   · **estrutural** — a deriva não depende do passo ⇒ a lei tem de mudar, e isso muda toda
///     pincelada deste app.
///
/// `cargo test -p ph2d-tool-painter --release --lib diag_o_passo_e_a_deriva -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição: imprime uma tabela"]
fn diag_o_passo_e_a_deriva() {
    use composite::CompositeOp::{Brush, Smear};
    let (c, r) = ([350.0f32, 350.0], 215.0f32);
    let alfa = |t: &PainterTool| -> f64 {
        (0..(L * L) as usize)
            .map(|i| f64::from(t.canvas_rgba[i * 4 + 3]))
            .sum::<f64>()
            / 255.0
    };
    // ⚠️ `PainterTool` não é `Clone` — cada lado corre a sua própria cena, de propósito.
    let corre = |sp: f32, esfregao: f32| -> (f64, Vec<[f32; 2]>, usize) {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; (L * L * 4) as usize], L, L);
        t.paint.brush.radius_px = 30.0;
        t.paint.brush.spacing = sp;
        t.paint.brush.color = [0.75, 0.12, 0.12];
        t.paint.brush.space_attenuation = false;
        t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
        t.paint.composite_enabled = true;
        for (op, st) in [(Smear, esfregao), (Brush, 1.0f32)] {
            t.acrescenta_camada(op.to_u8());
            let pos = t.composite_len() - 1;
            t.set_composite_layer_strength(pos, st);
        }
        t.on_canvas_pointer(cp2(c, PointerPhase::Down));
        for i in 1..=8 {
            let u = i as f32 / 8.0;
            t.on_canvas_pointer(cp2([c[0] + r * u, c[1]], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Up));
        let dabs = super::smear_warp::espia::ultimos_dabs().len();
        (alfa(&t), super::smear_warp::espia::ultimo(), dabs)
    };

    println!("  spacing  passo(px)  n dabs   guardado   |disp| medio   radial medio");
    for sp in [0.2f32, 0.1, 0.05, 0.025] {
        let (com, disp, n_dabs) = corre(sp, 1.0);
        let (sem, _, _) = corre(sp, 0.0);
        let passo = sp * 2.0 * 30.0;

        // |disp| e radial na metade do anel já saturada (θ = 180°).
        let (mut m, mut rd, mut n) = (0.0f32, 0.0f32, 0u32);
        let a = std::f32::consts::PI;
        let (sn, cs) = a.sin_cos();
        let mut rr = r - 25.0;
        while rr <= r + 25.0 {
            let x = (c[0] + cs * rr).round() as i32;
            let y = (c[1] + sn * rr).round() as i32;
            if (0..L as i32).contains(&x) && (0..L as i32).contains(&y) && !disp.is_empty() {
                let d = disp[(y as u32 * L + x as u32) as usize];
                m += d[0].hypot(d[1]);
                rd += (d[0] * cs + d[1] * sn).abs();
                n += 1;
            }
            rr += 1.0;
        }
        let n = n.max(1) as f32;
        println!(
            "  {sp:7.3}  {passo:9.2}  {n_dabs:6}   {:6.1} %   {:12.2}   {:12.2}",
            100.0 * com / sem,
            m / n,
            rd / n
        );
    }
}

/// (A10) O CISALHAMENTO — a deriva vem do PESO variar dentro do dab?
///
/// A (A9) mostrou que a deriva é estrutural. A composição `D(p) = v·w(p) + D(p − v·w(p))` não é
/// uma translação rígida: `w` cai do centro para a borda, logo cada dab CISALHA o mapa. Com a
/// direcção de `v` a rodar ao longo de uma curva, cisalhamentos sucessivos em referenciais
/// diferentes não comutam — e o traçado sai do anel.
///
/// A dureza é o botão que controla `w`: a `1` o núcleo é chapado (cisalhamento só na orla), a `0`
/// ele varia em todo o dab. Se a conservação subir com a dureza, o cisalhamento é o mecanismo.
///
/// `cargo test -p ph2d-tool-painter --release --lib diag_a_dureza_e_a_deriva -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição: imprime uma tabela"]
fn diag_a_dureza_e_a_deriva() {
    use composite::CompositeOp::{Brush, Smear};
    let (c, r) = ([350.0f32, 350.0], 215.0f32);
    let alfa = |t: &PainterTool| -> f64 {
        (0..(L * L) as usize)
            .map(|i| f64::from(t.canvas_rgba[i * 4 + 3]))
            .sum::<f64>()
            / 255.0
    };
    let corre = |dureza: f32, esfregao: f32| -> (f64, Vec<[f32; 2]>) {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; (L * L * 4) as usize], L, L);
        t.paint.brush.radius_px = 30.0;
        t.paint.brush.hardness = dureza;
        t.paint.brush.color = [0.75, 0.12, 0.12];
        t.paint.brush.space_attenuation = false;
        t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
        t.paint.composite_enabled = true;
        for (op, st) in [(Smear, esfregao), (Brush, 1.0f32)] {
            t.acrescenta_camada(op.to_u8());
            let pos = t.composite_len() - 1;
            t.set_composite_layer_strength(pos, st);
        }
        t.on_canvas_pointer(cp2(c, PointerPhase::Down));
        for i in 1..=8 {
            let u = i as f32 / 8.0;
            t.on_canvas_pointer(cp2([c[0] + r * u, c[1]], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Up));
        (alfa(&t), super::smear_warp::espia::ultimo())
    };
    println!("  dureza   guardado   |disp| θ=180   radial θ=180   raio de p-D (anel: 185..245)");
    for dureza in [0.0f32, 0.5, 0.9, 1.0] {
        let (com, disp) = corre(dureza, 1.0);
        let (sem, _) = corre(dureza, 0.0);
        let a = std::f32::consts::PI;
        let (sn, cs) = a.sin_cos();
        let (mut m, mut rd, mut raio, mut n) = (0.0f32, 0.0f32, 0.0f32, 0u32);
        let mut rr = r - 25.0;
        while rr <= r + 25.0 {
            let (x, y) = ((c[0] + cs * rr).round(), (c[1] + sn * rr).round());
            if x >= 0.0 && y >= 0.0 && (x as u32) < L && (y as u32) < L && !disp.is_empty() {
                let d = disp[((y as u32) * L + x as u32) as usize];
                m += d[0].hypot(d[1]);
                rd += d[0] * cs + d[1] * sn;
                raio += (x - d[0] - c[0]).hypot(y - d[1] - c[1]);
                n += 1;
            }
            rr += 1.0;
        }
        let n = n.max(1) as f32;
        println!(
            "  {dureza:6.2}   {:6.1} %   {:12.2}   {:13.2}   {:10.1}",
            100.0 * com / sem,
            m / n,
            rd / n,
            raio / n
        );
    }
}

/// (A12) O TECTO MUDA A RECTA? — a pergunta de PRODUTO, ao pixel.
///
/// Conservar tinta é uma SOMA e uma soma não diz que os pixels são os mesmos. O esfregão em
/// linha recta é um look que o dono já aprovou, logo a cura só é uma correcção de defeito — e não
/// uma mudança de produto — se ela deixar a recta onde estava.
///
/// Corre-se DUAS vezes, com e sem o tecto na lei, e comparam-se os números.
///
/// `cargo test -p ph2d-tool-painter --release --lib diag_o_tecto_muda_a_recta -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição: imprime uma tabela"]
fn diag_o_tecto_muda_a_recta() {
    use composite::CompositeOp::{Brush, Smear};
    let y = 350.0f32;
    let mut t = PainterTool::default();
    t.set_source(vec![0u8; (L * L * 4) as usize], L, L);
    t.paint.brush.radius_px = 30.0;
    t.paint.brush.color = [0.75, 0.12, 0.12];
    t.paint.brush.space_attenuation = false;
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Space;
    t.paint.composite_enabled = true;
    for (op, s) in [(Smear, 1.0f32), (Brush, 1.0)] {
        t.acrescenta_camada(op.to_u8());
        let pos = t.composite_len() - 1;
        t.set_composite_layer_strength(pos, s);
    }
    let n = 200;
    t.on_canvas_pointer(cp2([60.0, y], PointerPhase::Down));
    for i in 1..=n {
        t.on_canvas_pointer(cp2(
            [60.0 + 580.0 * (i as f32) / (n as f32), y],
            PointerPhase::Move,
        ));
    }
    t.on_canvas_pointer(cp2([640.0, y], PointerPhase::Up));

    // Um checksum de ordem-sensível sobre o alfa, mais o perfil ao longo do traço.
    let mut soma: u64 = 0;
    let mut n_tex = 0u64;
    for i in 0..(L * L) as usize {
        let a = u64::from(t.canvas_rgba[i * 4 + 3]);
        if a > 0 {
            n_tex += 1;
            soma = soma
                .wrapping_mul(1_000_003)
                .wrapping_add(a + i as u64 % 251);
        }
    }
    let disp = super::smear_warp::espia::ultimo();
    let maxd = disp.iter().map(|d| d[0].hypot(d[1])).fold(0.0f32, f32::max);
    println!("  RECTA · texels {n_tex} · checksum {soma:020} · |disp| MAX {maxd:.2}");
    // O perfil ao longo do traço, que é onde o transporte se veria.
    let col = |x: u32| -> u32 {
        (0..L)
            .map(|yy| u32::from(t.canvas_rgba[((yy * L + x) * 4 + 3) as usize]))
            .sum()
    };
    let amostras: Vec<u32> = [70u32, 150, 300, 450, 600, 635]
        .iter()
        .map(|&x| col(x))
        .collect();
    println!("  colunas x=70,150,300,450,600,635: {amostras:?}");
}

/// (A13) A VARREDURA DO TECTO — o número tem de sair de uma medição, não de uma derivação sozinha.
///
/// O `2` deriva do alcance do dab (um texel é atravessado por, no máximo, um diâmetro de cursor
/// enquanto está coberto). A varredura põe isso ao lado do que ele COMPRA na curva e do que ele
/// CUSTA na recta — que é o look aprovado.
///
/// `cargo test -p ph2d-tool-painter --release --lib diag_a_varredura_do_tecto -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição: imprime uma tabela"]
fn diag_a_varredura_do_tecto() {
    use composite::CompositeOp::{Brush, Smear};
    let (c, r) = ([350.0f32, 350.0], 215.0f32);
    let alfa = |t: &PainterTool| -> f64 {
        (0..(L * L) as usize)
            .map(|i| f64::from(t.canvas_rgba[i * 4 + 3]))
            .sum::<f64>()
            / 255.0
    };
    let cena = |esfregao: f32, metodo: ph2d_painter_brush::StrokeMethod| -> PainterTool {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; (L * L * 4) as usize], L, L);
        t.paint.brush.radius_px = 30.0;
        t.paint.brush.color = [0.75, 0.12, 0.12];
        t.paint.brush.space_attenuation = false;
        t.paint.brush.stroke_method = metodo;
        t.paint.composite_enabled = true;
        for (op, s) in [(Smear, esfregao), (Brush, 1.0f32)] {
            t.acrescenta_camada(op.to_u8());
            let pos = t.composite_len() - 1;
            t.set_composite_layer_strength(pos, s);
        }
        t
    };
    let anel = |t: &mut PainterTool| {
        t.on_canvas_pointer(cp2(c, PointerPhase::Down));
        for i in 1..=8 {
            let u = i as f32 / 8.0;
            t.on_canvas_pointer(cp2([c[0] + r * u, c[1]], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp2([c[0] + r, c[1]], PointerPhase::Up));
    };
    let recta = |t: &mut PainterTool| {
        let n = 200;
        t.on_canvas_pointer(cp2([60.0, 350.0], PointerPhase::Down));
        for i in 1..=n {
            let x = 60.0 + 580.0 * (i as f32) / (n as f32);
            t.on_canvas_pointer(cp2([x, 350.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp2([640.0, 350.0], PointerPhase::Up));
    };
    let coluna = |t: &PainterTool, x: u32| -> u32 {
        (0..L)
            .map(|yy| u32::from(t.canvas_rgba[((yy * L + x) * 4 + 3) as usize]))
            .sum()
    };

    // A referência da recta: o tecto INFINITO é o comportamento que o dono já aprovou.
    super::smear_warp::espia::poe_tecto(f32::INFINITY);
    let mut rf = cena(1.0, ph2d_painter_brush::StrokeMethod::Space);
    recta(&mut rf);
    let ref_cols: Vec<u32> = [70u32, 150, 300, 450, 600, 635]
        .iter()
        .map(|&x| coluna(&rf, x))
        .collect();

    println!("  tecto(R)   anel guardado   |disp| MAX   recta: pior coluna vs o de ontem");
    for tecto in [0.5f32, 1.0, 1.5, 2.0, 3.0, 4.0, 8.0, f32::INFINITY] {
        super::smear_warp::espia::poe_tecto(tecto);
        let mut t = cena(1.0, ph2d_painter_brush::StrokeMethod::Ellipse);
        anel(&mut t);
        let disp = super::smear_warp::espia::ultimo();
        let maxd = disp.iter().map(|d| d[0].hypot(d[1])).fold(0.0f32, f32::max);
        let mut s0 = cena(0.0, ph2d_painter_brush::StrokeMethod::Ellipse);
        anel(&mut s0);

        super::smear_warp::espia::poe_tecto(tecto);
        let mut rt = cena(1.0, ph2d_painter_brush::StrokeMethod::Space);
        recta(&mut rt);
        let pior = [70u32, 150, 300, 450, 600, 635]
            .iter()
            .enumerate()
            .map(|(k, &x)| {
                let a = coluna(&rt, x) as f32;
                let b = ref_cols[k] as f32;
                100.0 * (a - b).abs() / b.max(1.0)
            })
            .fold(0.0f32, f32::max);
        println!(
            "  {:8}   {:11.1} %   {maxd:10.2}   {pior:24.3} %",
            if tecto.is_finite() {
                format!("{tecto:.1}")
            } else {
                "INF".into()
            },
            100.0 * alfa(&t) / alfa(&s0)
        );
    }
    super::smear_warp::espia::poe_tecto(ph2d_painter_brush::SEM_TECTO);
}
