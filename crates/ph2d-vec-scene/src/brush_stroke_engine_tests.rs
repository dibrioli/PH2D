//! Os gates do **motor** do pincel de contorno (plano 36, W2).

use super::*;
use crate::{BrushStroke, Rgba8, VecPathId, VecVertex};

/// Um quadrado de lado `l`, FECHADO — o contorno que o pincel percorre.
fn quadrado(l: f64) -> VecPath {
    VecPath {
        verts: [[0.0, 0.0], [l, 0.0], [l, l], [0.0, l]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

/// A arte: um losango de `w × h`, centrado na origem.
fn arte(w: f64, h: f64) -> VecPath {
    VecPath {
        verts: [
            [-w * 0.5, 0.0],
            [0.0, -h * 0.5],
            [w * 0.5, 0.0],
            [0.0, h * 0.5],
        ]
        .map(VecVertex::corner)
        .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

fn pincel() -> BrushStroke {
    BrushStroke {
        art: VecPathId::from(1u64),
        fallback: Rgba8::new(1, 2, 3, 255),
        spacing: 1.0,
        offset: 0.0,
        flip: false,
        rotation_deg: 0.0,
        scale: 1.0,
    }
}

/// A altura de cada cópia, medida na saída.
fn altura(copias: &[VecPath]) -> f64 {
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    for v in copias.first().map(|c| c.verts.clone()).unwrap_or_default() {
        lo = lo.min(v.anchor[1]);
        hi = hi.max(v.anchor[1]);
    }
    hi - lo
}

/// ⭐⭐ **O BURACO INTEIRO: um contorno recebe cópias da arte ao longo dele.**
#[test]
fn a_brush_lays_copies_along_the_contour() {
    let c = brush_along_path(&quadrado(4.0), &arte(1.0, 1.0), &pincel(), 1.0);
    assert!(
        c.len() > 4,
        "o pincel nao poe copias ao longo do contorno (saiu {})",
        c.len()
    );
    // CONTROLO: uma arte degenerada não produz nada — e não um panic nem cópias de tamanho zero.
    assert!(brush_along_path(&quadrado(4.0), &arte(1.0, 0.0), &pincel(), 1.0).is_empty());
    // CONTROLO: largura zero também não.
    assert!(brush_along_path(&quadrado(4.0), &arte(1.0, 1.0), &pincel(), 0.0).is_empty());
}

/// ⭐⭐⭐ **A ARTE ESCALA COM A LARGURA DO TRAÇO** — a lei CONTRÁRIA à do padrão, e a do
/// *Pattern Brush*.
///
/// ⚠️ **O contra-exemplo está no ficheiro irmão**: o padrão guarda um `size` ABSOLUTO e não olha
/// para a largura. *Se as duas leis fossem a mesma, um dos dois modelos estaria errado.*
#[test]
fn the_brush_art_scales_with_the_stroke_width() {
    let fino = brush_along_path(&quadrado(8.0), &arte(1.0, 1.0), &pincel(), 0.5);
    let grosso = brush_along_path(&quadrado(8.0), &arte(1.0, 1.0), &pincel(), 2.0);
    assert!(!fino.is_empty() && !grosso.is_empty());
    let (a, b) = (altura(&fino), altura(&grosso));
    assert!(
        (b / a - 4.0).abs() < 1e-9,
        "a arte nao escalou com a largura: {a} contra {b} (esperado 4x)"
    );
    // E o `scale` multiplica isso — o neutro é `1,0`.
    let dobro = BrushStroke {
        scale: 2.0,
        ..pincel()
    };
    let c = brush_along_path(&quadrado(8.0), &arte(1.0, 1.0), &dobro, 0.5);
    assert!(
        (altura(&c) / a - 2.0).abs() < 1e-9,
        "o `scale` nao multiplica a altura derivada"
    );
}

/// ⭐⭐ **NUM CONTORNO FECHADO AS CÓPIAS FECHAM EXACTAMENTE** — sem cauda na emenda.
///
/// ⚠️ **É o defeito que o Enio reportou em 22/08 para o tracejado** (*"um traço curto encostado a um
/// longo, sempre na mesma quina"*), e a cura é a MESMA porta (`dash_fit::fit`) com o avanço no lugar
/// do período. *Duas leis de encaixe divergiriam no dia em que uma ganhasse um cuidado.*
///
/// A régua: o avanço EFECTIVO (o passo entre centros de cópias consecutivas) tem de dividir o
/// perímetro num número inteiro de vezes.
#[test]
fn on_a_closed_contour_the_copies_close_exactly() {
    // Perímetro 4·7 = 28; a arte mede 1 de largura ⇒ o avanço nominal é 1, que já divide 28.
    // ⇒ a fixtura tem de conter o fenómeno: uma largura de arte que NÃO divide o perímetro.
    let art = arte(1.3, 1.0);
    let copias = brush_along_path(&quadrado(7.0), &art, &pincel(), 1.0);
    assert!(copias.len() > 2, "sem cópias não há o que medir");
    let centro = |p: &VecPath| {
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for v in &p.verts {
            for k in 0..2 {
                lo[k] = lo[k].min(v.anchor[k]);
                hi[k] = hi[k].max(v.anchor[k]);
            }
        }
        [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5]
    };
    let (a, b) = (centro(&copias[0]), centro(&copias[1]));
    let passo = (b[0] - a[0]).hypot(b[1] - a[1]);
    let perimetro = 4.0 * 7.0;
    let n = perimetro / passo;
    assert!(
        (n - n.round()).abs() < 1e-3,
        "o avanço {passo} não divide o perímetro {perimetro} num número inteiro ({n}) - a emenda \
         deixa uma cauda, que é o report de 22/08 com outro sujeito"
    );
    // ⚠️ **CONTROLO — a fixtura CONTÉM o fenómeno**: sem encaixe o avanço seria a largura crua da
    // arte, e ela NÃO divide o perímetro. Sem esta metade o gate ficaria verde sobre uma arte que
    // encaixa por acidente.
    let crua = 1.3;
    assert!(
        (perimetro / crua - (perimetro / crua).round()).abs() > 1e-2,
        "a fixtura escolheu uma arte que já encaixava - o gate não mede o encaixe"
    );
}

/// ⚠️ **CADA CONTORNO de um composto recebe as suas cópias** — e cada um fecha.
///
/// ⛔ O `dash_fit` escolhe o contorno **mais longo** porque o traçador recebe **um** par
/// `[traço, vão]` para o caminho inteiro. Aqui essa restrição não existe, e herdá-la sem perguntar
/// seria uma limitação **inventada**.
#[test]
fn every_contour_of_a_compound_gets_its_own_copies() {
    let mut p = quadrado(8.0);
    p.subpaths.push(crate::Contour {
        verts: [[2.0, 2.0], [6.0, 2.0], [6.0, 6.0], [2.0, 6.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
    });
    let so_fora = brush_along_path(&quadrado(8.0), &arte(1.0, 1.0), &pincel(), 1.0);
    let com_furo = brush_along_path(&p, &arte(1.0, 1.0), &pincel(), 1.0);
    assert!(
        com_furo.len() > so_fora.len(),
        "o contorno de dentro nao recebeu copias ({} contra {})",
        com_furo.len(),
        so_fora.len()
    );
}

/// ⭐ **O `fit_to_guide` é OPT-IN, e o consumidor de hoje sai byte a byte como saía.**
///
/// ⚠️ O *Pattern on Path* (plano 23) tila pelo avanço nominal e deixa a cauda sobrar — é o
/// comportamento dele, gateado, e mexer-lhe seria mudar uma feature entregue por causa de outra.
#[test]
fn the_fit_is_opt_in_and_the_old_consumer_is_untouched() {
    assert!(
        !crate::pattern_path::PatternSpec::default().fit_to_guide,
        "o encaixe passou a ser o default - o Pattern on Path mudou de comportamento sem ninguem \
         pedir"
    );
}

/// ⭐⭐ **O KILL-CRITERION do plano 36, MEDIDO** — não presumido.
///
/// O plano 23 mediu **0,597 ms** para 200 cópias × 40 vértices e fixou o *kill* em **8 ms** (um
/// re-cook por tecla tem de caber num quadro). O pincel corre o MESMO motor, mais o encaixe (uma
/// divisão) e a escala da arte (um passe sobre os vértices dela, **uma vez**, não por cópia).
///
/// ⛔ **Se passar de 8 ms, a feature não existe nesta forma** e o passo seguinte é cache
/// por-params — ⛔ **não** subir o teto.
#[test]
#[ignore = "medicao: --release, maquina calma"]
fn measure_the_brush_recook() {
    // Um motivo de ~40 vértices, e uma guia que caiba ~200 cópias.
    let art = {
        let n = 40;
        let verts = (0..n)
            .map(|i| {
                let a = f64::from(i) / f64::from(n) * std::f64::consts::TAU;
                VecVertex::corner([a.cos() * 0.5, a.sin() * 0.5])
            })
            .collect();
        VecPath {
            verts,
            closed: true,
            ..VecPath::default()
        }
    };
    let guia = quadrado(50.0); // perímetro 200 ⇒ ~200 cópias com a arte de largura 1
    let b = pincel();
    let t = std::time::Instant::now();
    let n = 20;
    let mut total = 0usize;
    for _ in 0..n {
        total += brush_along_path(&guia, &art, &b, 1.0).len();
    }
    let ms = t.elapsed().as_secs_f64() * 1000.0 / f64::from(n);
    println!(
        "\n  [plano 36 W2] re-cook do pincel: {ms:.3} ms  ({} copias)  — kill = 8 ms; o plano 23 \
         mediu 0,597 ms para 200x40",
        total / n as usize
    );
    assert!(
        ms < 8.0,
        "o re-cook do pincel custa {ms:.3} ms, acima do kill de 8 - a feature nao existe nesta \
         forma, e o passo seguinte e' cache por-params, NAO subir o teto"
    );
}
