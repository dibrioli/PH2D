//! **O ENTER E A FIGURA SEGUINTE** — a sonda e o gate do report do dono de 2026-09-21:
//! *«apertei enter para fixar o desenho das formas vivas e tentei desenhar de novo com a elipse:
//! o retângulo voltou mas com a cor do canvas cobrindo o desenho anterior»*.
//!
//! ⚠️ Assunto PRÓPRIO, cortado do [`super::diag_o_resto_do_descasque`] pelo tecto de LOC: ali
//! mede-se o que um DESCASQUE deixa para trás (a caixa do lote), aqui o que **FIXAR** deixa por
//! fechar (a base da pilha). *Os dois reports leem-se iguais na tela e não têm um mecanismo em
//! comum* — e é por isso que cada um leva a régua dele.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --release --lib \
//!     diag_o_enter -- --ignored --nocapture --test-threads=1
//! ```

use super::diag_auditoria_da_pilha::{Caso, cp, pilha_do_dono};
// ⚠️ A régua de impressão é a da sonda irmã — *um segundo `conta` aqui seria a mesma
// resposta escrita duas vezes, e as duas tabelas deixariam de ser comparáveis.*
use super::diag_o_resto_do_descasque::{conta, mao_livre};
use super::*;

const S: u32 = 1024;

/// ⛔⛔⛔ **O ENTER E A FIGURA SEGUINTE** — report do dono (2026-09-21, foto): *«apertei enter para
/// fixar o desenho das formas vivas e tentei desenhar de novo com a elipse: o retângulo voltou mas
/// com a cor do canvas cobrindo o desenho anterior»*.
///
/// ⭐ Ele deu o gatilho exacto, e ele é um ACTO e não um gesto: o **Enter**.
#[test]
#[ignore = "sonda: corre à mão, --release --nocapture"]
fn diag_o_enter_e_a_figura_seguinte() {
    println!("\n  O ENTER E A FIGURA SEGUINTE — quanto da arte JÁ FIXADA a figura nova apaga");
    println!("  canvas {S}² PRETA TRANSPARENTE · pilha do dono · Ellipse\n");
    println!("  caso                                 | texels | pior |     caixa | por oitante");
    println!("  -------------------------------------+--------+------+-----------+------------");

    let cena = || {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; (S * S * 4) as usize], S, S);
        t.paint.brush.radius_px = 24.0;
        t.paint.brush.hardness = 1.0;
        t.paint.brush.strength = 1.0;
        t.paint.brush.space_attenuation = false;
        t.paint.composite_enabled = true;
        for pos in 0..composite::N_CAMADAS {
            t.paint.composite[pos] = composite::CompositeLayer::default();
        }
        pilha_do_dono(&mut t);
        t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
        t
    };
    let figura = |t: &mut PainterTool, c: [f32; 2], r: f32| {
        t.on_canvas_pointer(cp(c, PointerPhase::Down));
        for i in 1..=4 {
            t.on_canvas_pointer(cp([c[0] + r * i as f32 / 4.0, c[1]], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([c[0] + r, c[1]], PointerPhase::Up));
    };
    // Os texels da arte JÁ FIXADA que a figura seguinte destruiu.
    let destruido = |antes: &[u8], t: &PainterTool| {
        let mut v = Vec::new();
        for y in 0..S {
            for x in 0..S {
                let i = ((y * S + x) * 4 + 3) as usize;
                let (a, b) = (antes[i], t.canvas_rgba[i]);
                if a > 40 && b + 20 < a {
                    v.push((x, y, a - b));
                }
            }
        }
        v
    };

    let corre = |com_enter: bool, calar: Option<usize>| {
        let mut t = cena();
        if let Some(pos) = calar {
            t.paint.composite[pos].strength = 0.0;
        }
        figura(&mut t, [420.0, 512.0], 110.0);
        if com_enter {
            assert!(t.commit_open_shape(), "o Enter não fixou nada");
        }
        let antes = t.canvas_rgba.to_vec();
        figura(&mut t, [600.0, 512.0], 110.0);
        (antes, t)
    };
    for (nome, com_enter) in [
        ("SEM Enter (duas figuras na sessão)", false),
        ("COM Enter entre as duas", true),
    ] {
        let (antes, t) = corre(com_enter, None);
        conta(nome, &destruido(&antes, &t));
    }
    println!();
    for pos in 0..composite::N_CAMADAS {
        let base = cena();
        if base.paint.composite[pos].strength <= 0.0 {
            continue;
        }
        let op = base.paint.composite[pos].op;
        let (antes, t) = corre(true, Some(pos));
        conta(
            &format!("  COM Enter · camada {pos} ({op:?}) calada"),
            &destruido(&antes, &t),
        );
    }
    // ⭐ E as MESMAS duas células com a cura DESLIGADA — o controlo que diz quanto ela comprou.
    println!();
    for calar in [None, Some(4usize)] {
        super::composite_reposicoes::COMMIT_SEM_FECHAR.with(|c| c.set(true));
        let (antes, t) = corre(true, calar);
        super::composite_reposicoes::COMMIT_SEM_FECHAR.with(|c| c.set(false));
        let nome = match calar {
            None => "  [SEM a cura] COM Enter".to_string(),
            Some(p) => format!("  [SEM a cura] COM Enter · camada {p} calada"),
        };
        conta(&nome, &destruido(&antes, &t));
    }
}

/// ⭐⭐⭐ **FIXAR FECHA A PILHA — a figura seguinte não apaga o que o ENTER acabou de assar.**
///
/// Report do dono (2026-09-21, foto): *«apertei enter para fixar o desenho das formas vivas e
/// tentei desenhar de novo com a elipse: o retângulo voltou mas com a cor do canvas cobrindo o
/// desenho anterior»*.
///
/// ⛔⛔ **O pen-up de uma figura NÃO fecha o traço** — ela fica editável, e é isso que faz a sessão
/// inteira ser UM traço; enquanto ela dura, cada quadro re-carimba TODAS as figuras vivas a partir
/// do `pre` e nada se perde. O **Enter** assa os pixels e larga os editores, e o `pre` da pilha
/// continuava a ser a tela de **antes** delas ⇒ a figura seguinte reconstruía-se dessa base velha e,
/// na região dela, **apagava o que acabou de ser fixado**.
///
/// ⭐ **A cura é o QUARTO canal da lista do
/// [`super::stamp_preview::PainterTool::commit_drag_preview`]**, que já matava o relevo do traço, a
/// sessão do escultor e a da borracha pelo mesmo motivo — *o que foi fixado é permanente, logo o
/// estado por-traço que o descrevia deixou de valer*.
///
/// **Medido** (tela preta transparente, pilha do dono, duas elipses de raio `110`):
///
/// | | arte fixada destruída | calando o Smear |
/// |---|---|---|
/// | sem Enter entre as duas | `0` | — |
/// | **com Enter, SEM a cura** | **`12 530`** | `12 530` — *o esfregão não é a causa* |
/// | **com Enter, com a cura** | `3 083` | **`0`** — *o que sobra é SÓ o esfregão* |
///
/// ⏳ Os `3 083` que ficam são o resíduo da **base congelada do esfregão** já nomeado no §5.3 da
/// [auditoria](../../../../../docs/Painter/40_auditoria_da_pilha_2026-09-21.md) — *outro mecanismo,
/// com a cura endereçada lá*. É por isso que a metade comportamental deste gate cala o Smear: ela
/// afirma o que ESTA cura comprou, e não o que a próxima deve comprar.
#[test]
fn fixar_fecha_a_pilha() {
    let cena = || {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; (S * S * 4) as usize], S, S);
        t.paint.brush.radius_px = 24.0;
        t.paint.brush.hardness = 1.0;
        t.paint.brush.strength = 1.0;
        t.paint.brush.space_attenuation = false;
        t.paint.composite_enabled = true;
        for pos in 0..composite::N_CAMADAS {
            t.paint.composite[pos] = composite::CompositeLayer::default();
        }
        pilha_do_dono(&mut t);
        t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Ellipse;
        t
    };
    let figura = |t: &mut PainterTool, c: [f32; 2], r: f32| {
        t.on_canvas_pointer(cp(c, PointerPhase::Down));
        for i in 1..=4 {
            t.on_canvas_pointer(cp([c[0] + r * i as f32 / 4.0, c[1]], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([c[0] + r, c[1]], PointerPhase::Up));
    };
    let destruido = |antes: &[u8], t: &PainterTool| {
        (0..(S * S) as usize)
            .filter(|&i| {
                let (a, b) = (antes[i * 4 + 3], t.canvas_rgba[i * 4 + 3]);
                a > 40 && b + 20 < a
            })
            .count()
    };
    // Sem o esfregão, para a régua afirmar SÓ o que esta cura comprou.
    let corre = |sem_cura: bool| {
        super::composite_reposicoes::COMMIT_SEM_FECHAR.with(|c| c.set(sem_cura));
        let mut t = cena();
        t.paint.composite[4].strength = 0.0;
        figura(&mut t, [420.0, 512.0], 110.0);
        assert!(t.commit_open_shape(), "o Enter não fixou nada");
        // ⚠️ A pilha mede-se AQUI, e não no fim: a 2.ª figura reabre-a e re-semeia o `pre` — *uma
        // régua lida depois do gesto seguinte não afirma nada sobre o commit*.
        let fechou = t.paint.pilha.pre.is_empty();
        let antes = t.canvas_rgba.to_vec();
        let fixado = (0..(S * S) as usize)
            .filter(|&i| t.canvas_rgba[i * 4 + 3] > 40)
            .count();
        figura(&mut t, [600.0, 512.0], 110.0);
        super::composite_reposicoes::COMMIT_SEM_FECHAR.with(|c| c.set(false));
        (fixado, destruido(&antes, &t), fechou)
    };

    // CONTROLO POSITIVO: o Enter de facto assou arte — senão «não apaga nada» é vácuo.
    let (fixado, n, fechou) = corre(false);
    assert!(fixado > 20_000, "o Enter mal assou arte ({fixado} texels)");

    // ⛔ O CONTROLO DA CURA: sem ela a figura seguinte APAGA o que foi fixado.
    let (_, sem_cura, fechou_sem_cura) = corre(true);
    assert!(
        sem_cura > 5_000,
        "a fixtura não contém o fenómeno: sem a cura só {sem_cura} texels foram destruídos"
    );

    // ⭐ A LEI, nas duas metades: a pilha fecha, e nada do que foi fixado é apagado.
    assert!(
        fechou,
        "o Enter não fechou a pilha — o `pre` sobreviveu ao commit"
    );
    assert!(
        !fechou_sem_cura,
        "o interruptor de bissecção não desarmou a cura: a pilha fechou na mesma"
    );
    assert_eq!(
        n, 0,
        "a figura seguinte apagou {n} texels da arte fixada (sem a cura eram {sem_cura})"
    );
}

/// ⚠️⚠️ **A PREMISSA DESTA SONDA MORREU, e ela fica com a morte à vista.**
///
/// Ela foi escrita para achar o rectângulo da 2.ª e 3.ª fotos e lê **`0` em todas as células** —
/// porque procura o defeito num gesto **SEM o Enter**, que é exactamente onde ele não está. O que
/// ela mede continua a ser verdade e é o CONTROLO da sonda ao lado: *uma figura sozinha não deixa
/// a tela opaca*. ⛔ Apagá-la levaria a medição junto.
/// ⛔⛔⛔ **O RECTÂNGULO OPACO numa sprite TRANSPARENTE** — a 2.ª e a 3.ª fotos do dono
/// (2026-09-21): *«usei também uma sprite transparente e o retângulo aparece (o brush não tem seu
/// fundo transparente)»*.
///
/// ⭐⭐⭐ **A régua anterior estava APONTADA AO CONTRÁRIO, e por isso lia `0`:** ela procurava alfa a
/// CAIR (arte apagada) e o que acontece é o alfa a SUBIR — a tela transparente fica **opaca**. E a
/// fixtura piorava-o: `tela_com(_, 0)` é branco transparente, onde um rectângulo BRANCO só difere
/// no alfa. Aqui a tela nasce **preta transparente** (`0,0,0,0`), onde um rectângulo branco opaco é
/// inconfundível nos quatro canais.
#[test]
#[ignore = "sonda: corre à mão, --release --nocapture"]
fn diag_o_rectangulo_opaco() {
    println!(
        "\n  O RECTÁNGULO OPACO — tela PRETA TRANSPARENTE, opacidade a MAIS de 150 px da figura"
    );
    println!("  canvas {S}² · pilha do dono\n");
    println!("  caso                                 | texels | pior |     caixa | por oitante");
    println!("  -------------------------------------+--------+------+-----------+------------");

    let tela_transparente = || {
        let mut t = PainterTool::default();
        t.set_source(vec![0u8; (S * S * 4) as usize], S, S);
        t.paint.brush.radius_px = 24.0;
        t.paint.brush.hardness = 1.0;
        t.paint.brush.strength = 1.0;
        t.paint.brush.space_attenuation = false;
        t.paint.composite_enabled = true;
        for pos in 0..composite::N_CAMADAS {
            t.paint.composite[pos] = composite::CompositeLayer::default();
        }
        pilha_do_dono(&mut t);
        t
    };
    // ⚠️ **A régua tem de EXCLUIR a figura, senão ela mede a tinta legítima.** A 1.ª redacção
    // contava todo texel opaco e leu `27 006` — *o desenho*, com o CONTROLO cumulativo a ler
    // `19 436` e a mesma caixa. O que o dono vê é opacidade **longe** de onde o pincel passou.
    const LONGE: f64 = 150.0;
    let opacos_longe = |t: &PainterTool, de: [f32; 2], ate: [f32; 2]| {
        let (ax, ay) = (f64::from(de[0]), f64::from(de[1]));
        let (bx, by) = (f64::from(ate[0]), f64::from(ate[1]));
        let (dx, dy) = (bx - ax, by - ay);
        let ll = dx.mul_add(dx, dy * dy).max(1e-9);
        let mut v = Vec::new();
        for y in 0..S {
            for x in 0..S {
                let a = t.canvas_rgba[((y * S + x) * 4 + 3) as usize];
                if a <= 8 {
                    continue;
                }
                let (px, py) = (f64::from(x) - ax, f64::from(y) - ay);
                let u = px.mul_add(dx, py * dy).div_euclid(1.0).min(ll).max(0.0) / ll;
                let d = (px - u * dx).hypot(py - u * dy);
                if d > LONGE {
                    v.push((x, y, a));
                }
            }
        }
        v
    };
    let opacos = |t: &PainterTool| opacos_longe(t, [380.0, 440.0], [640.0, 580.0]);
    // ⚠️ **Só os gestos que a régua sabe EXCLUIR entram na tabela.** A 1.ª redacção corria também
    // Ellipse e Anchored contra a geometria da MÃO LIVRE — e as duas «acusaram» a própria figura
    // delas, que a régua não sabia onde ficava. *Uma exclusão que não descreve o sujeito mede-o.*
    let casos: [Caso; 2] = [
        ("FreeHand largada", |t| {
            mao_livre(t, [380.0, 440.0], [640.0, 580.0], true);
        }),
        ("CONTROLO: Space (cumulativo) no mesmo caminho", |t| {
            t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::Space;
            t.on_canvas_pointer(cp([380.0, 440.0], PointerPhase::Down));
            for i in 1..=40 {
                let u = i as f32 / 40.0;
                t.on_canvas_pointer(cp(
                    [380.0 + 260.0 * u, 440.0 + 140.0 * u],
                    PointerPhase::Move,
                ));
            }
            t.on_canvas_pointer(cp([640.0, 580.0], PointerPhase::Up));
        }),
    ];
    for (nome, gesto) in casos {
        let mut t = tela_transparente();
        gesto(&mut t);
        let v = opacos(&t);
        conta(nome, &v);
        if let Some(&(x, y, _)) = v.first() {
            let i = ((y * S + x) * 4) as usize;
            println!(
                "      1.º texel ({x},{y}) rgba = [{},{},{},{}]",
                t.canvas_rgba[i],
                t.canvas_rgba[i + 1],
                t.canvas_rgba[i + 2],
                t.canvas_rgba[i + 3]
            );
        }
    }
    // ABLAÇÃO sobre o caso mais forte.
    println!();
    for alvo in 0..composite::N_CAMADAS {
        let op = tela_transparente().paint.composite[alvo].op;
        if tela_transparente().paint.composite[alvo].strength <= 0.0 {
            continue;
        }
        let mut t = tela_transparente();
        t.paint.composite[alvo].strength = 0.0;
        mao_livre(&mut t, [380.0, 440.0], [640.0, 580.0], true);
        conta(
            &format!("  ablação: camada {alvo} ({op:?}) calada"),
            &opacos(&t),
        );
    }
}
