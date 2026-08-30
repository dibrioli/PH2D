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
        art: Some(VecPathId::from(1u64)),
        fallback: Rgba8::new(1, 2, 3, 255),
        spacing: 1.0,
        offset: 0.0,
        flip: false,
        rotation_deg: 0.0,
        scale: 1.0,
    }
}

/// ⭐ **O TRAÇO que carrega o pincel** — a porta por onde o motor recebe tudo o que precisa: a
/// arte, a largura da faixa e o tracejado, os três do MESMO objecto.
fn traco(b: &BrushStroke, width: f64, dash: Option<(f64, f64)>) -> crate::StrokeSpec {
    let mut s = crate::StrokeSpec::new(b.fallback, width);
    s.paint = crate::StrokePaint::Brush(Box::new(b.clone()));
    s.dash = dash;
    s
}

/// Uma reta ABERTA de `(0,0)` a `(l,0)` — a guia em que **a posição de arco é a coordenada `x`**,
/// e é por isso que ela existe: num quadrado não há como ler onde uma cópia caiu sem refazer a
/// travessia, e uma régua que refaz a conta que mede não mede nada.
fn segmento(l: f64) -> VecPath {
    VecPath {
        verts: [[0.0, 0.0], [l, 0.0]].map(VecVertex::corner).to_vec(),
        closed: false,
        ..VecPath::default()
    }
}

/// O centro da caixa de uma cópia.
fn centro(p: &VecPath) -> [f64; 2] {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for v in &p.verts {
        for k in 0..2 {
            lo[k] = lo[k].min(v.anchor[k]);
            hi[k] = hi[k].max(v.anchor[k]);
        }
    }
    [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5]
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
    let c = brush_along_path(
        &quadrado(4.0),
        &arte(1.0, 1.0),
        &traco(&pincel(), 1.0, None),
    );
    assert!(
        c.len() > 4,
        "o pincel nao poe copias ao longo do contorno (saiu {})",
        c.len()
    );
    // CONTROLO: uma arte degenerada não produz nada — e não um panic nem cópias de tamanho zero.
    assert!(
        brush_along_path(
            &quadrado(4.0),
            &arte(1.0, 0.0),
            &traco(&pincel(), 1.0, None)
        )
        .is_empty()
    );
    // CONTROLO: largura zero também não.
    assert!(
        brush_along_path(
            &quadrado(4.0),
            &arte(1.0, 1.0),
            &traco(&pincel(), 0.0, None)
        )
        .is_empty()
    );
    // ⚠️ **CONTROLO — um traço que NÃO é pincel não devolve cópias.** A pergunta *"que cópias este
    // traço põe?"* não tem resposta num traço sólido, e inventar um pincel para responder seria
    // pior que devolver nada.
    assert!(
        brush_along_path(
            &quadrado(4.0),
            &arte(1.0, 1.0),
            &crate::StrokeSpec::new(Rgba8::new(1, 2, 3, 255), 1.0)
        )
        .is_empty(),
        "um traco SOLIDO devolveu copias de pincel"
    );
}

/// ⭐⭐⭐ **A ARTE ESCALA COM A LARGURA DO TRAÇO** — a lei CONTRÁRIA à do padrão, e a do
/// *Pattern Brush*.
///
/// ⚠️ **O contra-exemplo está no ficheiro irmão**: o padrão guarda um `size` ABSOLUTO e não olha
/// para a largura. *Se as duas leis fossem a mesma, um dos dois modelos estaria errado.*
#[test]
fn the_brush_art_scales_with_the_stroke_width() {
    let fino = brush_along_path(
        &quadrado(8.0),
        &arte(1.0, 1.0),
        &traco(&pincel(), 0.5, None),
    );
    let grosso = brush_along_path(
        &quadrado(8.0),
        &arte(1.0, 1.0),
        &traco(&pincel(), 2.0, None),
    );
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
    let c = brush_along_path(&quadrado(8.0), &arte(1.0, 1.0), &traco(&dobro, 0.5, None));
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
    let copias = brush_along_path(&quadrado(7.0), &art, &traco(&pincel(), 1.0, None));
    assert!(copias.len() > 2, "sem cópias não há o que medir");
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
    let so_fora = brush_along_path(
        &quadrado(8.0),
        &arte(1.0, 1.0),
        &traco(&pincel(), 1.0, None),
    );
    let com_furo = brush_along_path(&p, &arte(1.0, 1.0), &traco(&pincel(), 1.0, None));
    assert!(
        com_furo.len() > so_fora.len(),
        "o contorno de dentro nao recebeu copias ({} contra {})",
        com_furo.len(),
        so_fora.len()
    );
}

/// ⭐ **O `fit_span` é OPT-IN, e o consumidor de hoje sai byte a byte como saía.**
///
/// ⚠️ O *Pattern on Path* (plano 23) tila pelo avanço nominal e deixa a cauda sobrar — é o
/// comportamento dele, gateado, e mexer-lhe seria mudar uma feature entregue por causa de outra.
#[test]
fn the_fit_is_opt_in_and_the_old_consumer_is_untouched() {
    assert!(
        crate::pattern_path::PatternSpec::default()
            .fit_span
            .is_none(),
        "o encaixe passou a ser o default - o Pattern on Path mudou de comportamento sem ninguem \
         pedir"
    );
}

/// ⭐⭐⭐ **COM O VÃO A ZERO AS DUAS LEIS DE ENCAIXE SÃO A MESMA** — a menos de UMA cópia, e só nos
/// empates do `round`. É isto que deixa a fatia de um traço usar a porta do tracejado sem uma
/// bandeira própria.
///
/// As duas leis do [`crate::dash_fit::fit`] são `round(T/p)·p` (fechada) e
/// `round((T−d)/p)·p + d` (aberta). Com `p = d` as duas dizem *"`round(T/p)` cópias de comprimento
/// `p`, no mínimo uma"* — a mesma frase. ⚠️ **Em `f64` não são a mesma EXPRESSÃO:** quando `T/p`
/// cai perto de um meio-inteiro, `(T−p)/p` e `T/p − 1` aterram em lados opostos da fronteira do
/// `round` e as duas escolhem um número de cópias que **difere de um** — MEDIDO: **`5,84e-4`** de
/// avanço no pior caso desta varredura (`n ≈ 1712` cópias).
///
/// ⚠️⚠️ **TRÊS redacções minhas erraram antes desta, e a última custou código:** a 1.ª afirmou
/// igualdade EXACTA (reprovou a 2 ULP); a 2.ª afirmou *"2 ULP"* (reprovou a `5,84e-4`, três ordens
/// acima); a 3.ª concluiu daí que a bandeira **era** load-bearing e escreveu uma derivação
/// (*"uma fatia de traço é aberta mesmo numa guia fechada"*) com um gate a defendê-la — e a
/// **mutação que a desfazia SOBREVIVEU**, porque o empate do `round` é knife-edge e nenhuma
/// fixtura honesta cai nele. A derivação saiu. *Uma afirmação sobre `f64` que não foi varrida é uma
/// conjectura com cara de teorema; e uma guarda que mutação nenhuma mata é código sem sujeito.*
#[test]
fn the_zero_gap_fit_is_the_same_law_either_way_within_one_copy() {
    let mut vistos = 0u32;
    let mut pior = 0.0f64;
    for i in 1..=400u32 {
        let total = f64::from(i) * 0.137;
        for avanco in [0.01, 0.37, 1.0, 2.5, 13.0, 400.0] {
            let fechado = crate::dash_fit::fit([avanco, 0.0], total, true)[0];
            let aberto = crate::dash_fit::fit([avanco, 0.0], total, false)[0];
            pior = pior.max((fechado - aberto).abs() / fechado.max(f64::MIN_POSITIVE));
            vistos += 1;
        }
    }
    assert_eq!(vistos, 2400, "a varredura encolheu");
    // ⚠️ **A metade que dá SUJEITO ao gate**: se as duas leis concordassem sempre, a bandeira não
    // teria de ser derivada e este ficheiro estaria a defender uma distinção sem diferença.
    assert!(
        pior > 0.0,
        "as duas leis de encaixe passaram a concordar em toda a varredura - a bandeira deixou de \
         ser load-bearing, e a derivacao no `pattern_along` passa a ser codigo sem sujeito"
    );
    // E a diferença é LIMITADA a uma cópia sobre o vão: ~1/n, e nunca uma mudança de carácter.
    assert!(
        pior < 1e-3,
        "as duas leis divergiram {pior:e} relativo - acima de uma copia sobre o vao, o que quer \
         dizer que uma delas deixou de ser a mesma conta"
    );
    // ⚠️ **CONTROLO — com um VÃO as duas leis divergem de FORMA, não por uma cópia.** Sem esta
    // metade, o gate ficaria verde num dia em que o `fit` deixasse de olhar para o `closed`.
    let com_vao = |c| crate::dash_fit::fit([1.0, 1.0], 7.0, c);
    assert_ne!(
        com_vao(true),
        com_vao(false),
        "o `fit` deixou de distinguir fechado de aberto - este gate perdeu o sujeito"
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
        total += brush_along_path(&guia, &art, &traco(&b, 1.0, None)).len();
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

// ─────────────────────────────────────────────────────────────────────────────────────────────
// W3-bis — **o TRACEJADO**: a arte reinicia em cada traço (a lei do *Pattern Brush*).
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **A ARTE VIVE DENTRO DOS TRAÇOS, e os vãos ficam VAZIOS** — o buraco que o Enio abriu ao
/// perguntar *"mas não posso usar o dash com pattern?"*.
///
/// ⚠️ **A guia é uma RETA ABERTA de propósito**: nela a posição de arco de uma cópia é a
/// coordenada `x` do centro dela, e a régua pode ser lida sem refazer a travessia que o motor faz.
/// *Uma régua que recalcula o que mede não é um instrumento, é uma segunda implementação.*
#[test]
fn the_art_lives_inside_the_dashes_and_the_gaps_stay_empty() {
    let l = 20.0;
    let guia = segmento(l);
    let s = traco(&pincel(), 1.0, Some((2.0, 2.0)));
    // O MESMO par que o traçador desenharia — a porta única (`dash_lengths_for`), não uma
    // segunda conta.
    let [d, g] = crate::dash_fit::dash_lengths_for(&guia, &s).expect("a fixtura tem tracejado");
    let periodo = d + g;
    let copias = brush_along_path(&guia, &arte(1.0, 1.0), &s);
    assert!(copias.len() >= 4, "sem cópias não há o que medir");
    // ⚠️ Meia largura de cópia de folga: a cópia inteira cabe na fatia, então o CENTRO dela está
    // pelo menos a meia cópia de cada borda — a régua é sobre onde a arte caiu, não sobre um
    // epsilon de aritmética.
    for c in &copias {
        let x = centro(c)[0];
        let fase = x % periodo;
        assert!(
            fase <= d + 1e-9,
            "uma copia caiu no VAO: x={x}, fase={fase} num traco de {d} (periodo {periodo})"
        );
    }
    // ⚠️⚠️ **CONTROLO — a fixtura CONTÉM o fenómeno.** Sem o tracejado, o mesmo pincel enche a reta
    // toda e cópias caem exactamente onde os vãos estariam. Sem esta metade, o gate ficaria verde
    // sobre um pincel que não desenhasse nada.
    let sem = brush_along_path(&guia, &arte(1.0, 1.0), &traco(&pincel(), 1.0, None));
    assert!(
        sem.iter().any(|c| centro(c)[0] % periodo > d + 1e-9),
        "sem tracejado nenhuma copia cai onde um vao estaria - a fixtura nao contem o fenomeno"
    );
    assert!(
        sem.len() > copias.len(),
        "o tracejado nao tirou copias nenhumas ({} contra {})",
        sem.len(),
        copias.len()
    );
}

/// ⭐⭐ **TODO TRAÇO LEVA O MESMO RITMO** — o avanço encaixa no TRAÇO, uma vez, e vale para todos.
///
/// ⛔ A alternativa (encaixar cada fatia em si mesma) daria à fatia truncada do fim de um anel uma
/// cadência própria, à vista. *O alvo é um número só de propósito.*
#[test]
fn every_dash_carries_the_same_rhythm() {
    let guia = segmento(20.0);
    let s = traco(&pincel(), 1.0, Some((2.0, 2.0)));
    let [d, _] = crate::dash_fit::dash_lengths_for(&guia, &s).expect("a fixtura tem tracejado");
    let copias = brush_along_path(&guia, &arte(1.0, 1.0), &s);
    let xs: Vec<f64> = copias.iter().map(|c| centro(c)[0]).collect();
    assert!(xs.len() >= 4);
    // Dentro de um traço o passo entre cópias é o avanço encaixado; ele tem de ser o MESMO em
    // todos os traços, e dividir o comprimento do traço num número inteiro de vezes.
    let mut passos: Vec<f64> = Vec::new();
    for par in xs.windows(2) {
        let p = par[1] - par[0];
        if p < d {
            passos.push(p); // dois vizinhos DENTRO do mesmo traço
        }
    }
    assert!(
        passos.len() >= 2,
        "cada traco levou uma copia so' - sem ritmo a medir"
    );
    let primeiro = passos[0];
    for p in &passos {
        assert!(
            (p - primeiro).abs() < 1e-9,
            "o ritmo mudou entre tracos: {primeiro} contra {p}"
        );
    }
    let n = d / primeiro;
    assert!(
        (n - n.round()).abs() < 1e-6,
        "o avanco {primeiro} nao divide o traco {d} num numero inteiro ({n}) - o traco nao comeca \
         nem acaba com uma copia inteira"
    );
}

/// ⚠️ **A LEI DAS FATIAS, pura** — sem tracejado é uma; com tracejado é uma por traço, truncada no
/// fim do contorno como um traçador faz.
#[test]
fn the_spans_are_one_without_a_dash_and_one_per_dash_with_one() {
    assert_eq!(crate::brush_spans(10.0, None), vec![(0.0, 10.0)]);
    // Um traço de 2 e um vão de 2 num contorno de 10: `[0,2] [4,6] [8,10]`.
    assert_eq!(
        crate::brush_spans(10.0, Some([2.0, 2.0])),
        vec![(0.0, 2.0), (4.0, 6.0), (8.0, 10.0)]
    );
    // ⚠️ A última fatia é TRUNCADA — um composto tem um par de tracejado só (fitado ao contorno
    // mais longo), então os outros anéis acabam a meio de um traço.
    assert_eq!(
        crate::brush_spans(9.0, Some([2.0, 2.0])),
        vec![(0.0, 2.0), (4.0, 6.0), (8.0, 9.0)]
    );
    // Degenerados: um contorno sem comprimento não tem fatia nenhuma, e um "tracejado" de traço
    // nulo é uma linha contínua (a mesma leitura do `dash_lengths`).
    assert!(crate::brush_spans(0.0, None).is_empty());
    assert!(crate::brush_spans(f64::NAN, Some([2.0, 2.0])).is_empty());
    assert_eq!(
        crate::brush_spans(10.0, Some([0.0, 2.0])),
        vec![(0.0, 10.0)]
    );
    // ⛔ **O TETO** — o recurso é tempo, e o número está medido no `MAX_DASHES`.
    let muitas = crate::brush_spans(1.0e6, Some([0.1, 0.1]));
    assert_eq!(
        muitas.len(),
        super::MAX_DASHES,
        "as fatias deixaram de ser limitadas - um tracejado fino num contorno longo passa a custar \
         sem teto"
    );
}

/// ⭐ **UM TRAÇO SÓLIDO SAI EXACTAMENTE COMO SAÍA** (W2) — o tracejado é opt-in do documento.
#[test]
fn a_solid_brush_stroke_is_untouched_by_the_dash_law() {
    let guia = quadrado(7.0);
    let art = arte(1.3, 1.0);
    let antes = brush_copies(
        &crate::Contour {
            verts: guia.verts.clone(),
            closed: true,
        },
        &art,
        &pincel(),
        1.0,
        None,
    );
    let agora = brush_along_path(&guia, &art, &traco(&pincel(), 1.0, None));
    assert_eq!(
        antes.len(),
        agora.len(),
        "a porta por contorno e a do caminho deixaram de concordar"
    );
    for (a, b) in antes.iter().zip(&agora) {
        assert_eq!(
            centro(a),
            centro(b),
            "as copias mudaram de sitio sem tracejado"
        );
    }
}

/// ⭐⭐ **O CUSTO do tracejado, MEDIDO** — é o que fixa o `MAX_DASHES`.
///
/// ⚠️ O que cresce com o número de traços **não são as cópias** (a soma delas é a mesma do contorno
/// inteiro), é o custo FIXO por fatia: uma medida do bbox da arte e uma divisão.
#[test]
#[ignore = "medicao: --release, maquina calma"]
fn measure_the_dashed_brush_recook() {
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
    let guia = quadrado(50.0); // perímetro 200
    for dash in [
        None,
        Some((0.5, 1.5)),
        Some((0.125, 0.375)),
        Some((0.05, 0.1451)),
        Some((0.025, 0.0725)),
        Some((0.0125, 0.03625)),
        Some((0.00625, 0.018125)),
        Some((0.003125, 0.0090625)),
    ] {
        let s = traco(&pincel(), 1.0, dash);
        let fatias = crate::dash_fit::dash_lengths_for(&guia, &s)
            .map_or(1, |d| crate::brush_spans(200.0, Some(d)).len());
        let t = std::time::Instant::now();
        let n = 20;
        let mut total = 0usize;
        for _ in 0..n {
            total += brush_along_path(&guia, &art, &s).len();
        }
        let ms = t.elapsed().as_secs_f64() * 1000.0 / f64::from(n);
        println!(
            "  [plano 36 W3-bis] {fatias:>5} fatias · {:>5} copias · {ms:.3} ms   (kill = 8 ms)",
            total / n as usize
        );
        assert!(
            ms < 8.0,
            "o re-cook do pincel tracejado custa {ms:.3} ms com {fatias} fatias, acima do kill de 8"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// W5 — **AS QUINAS**: a MEDIÇÃO do defeito, antes de qualquer desenho de cura.
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// ⭐ **O DESVIO de uma cópia** — a maior distância de um vértice dela à GUIA que ela devia
/// percorrer, em unidades de mundo.
///
/// ⚠️ **Régua LOCAL, por cópia**, e é de propósito: a lição que o quad-remesh desta casa pagou três
/// vezes é que *uma régua que resume o conjunto é cega ao caso que interessa* — aqui o defeito são
/// duas ou três cópias numa volta de dezenas, e uma mediana não se mexe.
fn desvio(copia: &VecPath, guia: &crate::arc_path::ArcPath) -> f64 {
    copia
        .verts
        .iter()
        .map(|v| {
            let s = guia.closest_arc(v.anchor);
            let (p, _) = guia.frame_at(s);
            (p[0] - v.anchor[0]).hypot(p[1] - v.anchor[1])
        })
        .fold(0.0, f64::max)
}

/// ⭐⭐ **O ERRO DE ORIENTAÇÃO de uma cópia**, em GRAUS — o ângulo entre o eixo que a cópia adoptou
/// (a tangente no CENTRO da fatia dela) e a tangente da guia nas duas PONTAS dessa fatia.
///
/// ⚠️ Esta é a régua que nomeia o mecanismo, e a outra só nomeia o sintoma: a cópia é rígida e
/// recebe **um** referencial; numa recta as três tangentes coincidem e o erro é `0`; numa quina
/// elas divergem pela **metade do ângulo de viragem**, e é isso que faz a arte furar a forma.
fn erro_de_orientacao(guia: &crate::arc_path::ArcPath, centro_s: f64, avanco: f64) -> f64 {
    let ang = |s: f64| {
        let (_, t) = guia.frame_at(s.rem_euclid(guia.total()));
        t[1].atan2(t[0])
    };
    let meio = ang(centro_s);
    [centro_s - avanco * 0.5, centro_s + avanco * 0.5]
        .into_iter()
        .map(|s| {
            let d = (ang(s) - meio).abs();
            let d = if d > std::f64::consts::PI {
                std::f64::consts::TAU - d
            } else {
                d
            };
            d.to_degrees()
        })
        .fold(0.0, f64::max)
}

fn guia_de(p: &VecPath) -> crate::arc_path::ArcPath {
    crate::arc_path::ArcPath::from_contour(&p.verts, p.closed).expect("guia com comprimento")
}

/// ⭐⭐⭐ **QUANTO É QUE A QUINA PARTE O PINCEL** — a medição que abre a W5.
///
/// ⚠️⚠️ **A 1.ª fixtura desta sonda NÃO continha o fenómeno e leu o defeito como AUSENTE.** Com uma
/// arte de largura `1` num quadrado de lado `7`, o avanço encaixa em `1,0` e os centros caem em
/// `0,5 · 1,5 · …` — as quinas (`7 · 14 · 21 · 28`) caem **exactamente ENTRE** duas cópias, e
/// nenhuma atravessa uma. *A fixtura mais azarada possível é a que diz que está tudo bem.*
///
/// ⚠️⚠️ **E a 1.ª RÉGUA era cega ao defeito principal:** ela media o desvio das cópias **EMITIDAS**,
/// e o que uma quina faz hoje é **não emitir**. O `ArcPath` devolve tangente NULA numa cúspide, o
/// `GlyphFrame::on_path` devolve `None`, e o `pattern_along` **pula a cópia** — fica um BURACO.
/// *Uma régua que percorre o que existe não vê o que faltou.*
///
/// O CONTROLO é um círculo do MESMO perímetro com a MESMA arte: ele dá a contagem que a quina
/// deveria ter dado.
#[test]
#[ignore = "medicao: imprime a tabela da W5"]
fn measure_how_far_a_corner_throws_the_copies_off_the_guide() {
    let largura = 1.0;
    let b = pincel();
    let s = traco(&b, largura, None);
    let meia_altura = crate::brush_height(&b, largura) * 0.5;
    let raio = 28.0 / std::f64::consts::TAU;
    println!("\n  [plano 36 W5] meia-altura da arte = {meia_altura:.4} (o desvio 'de graca')");
    println!("  perimetro 28 nas duas formas; o CIRCULO da a contagem que a quina devia dar\n");

    for w in [1.0, 1.3, 1.7, 2.0] {
        let art = arte(w, 1.0);
        let quad = brush_along_path(&quadrado(7.0), &art, &s);
        let circ = brush_along_path(&crate::ellipse([0.0, 0.0], raio, raio), &art, &s);
        let guia = guia_de(&quadrado(7.0));
        let pior = quad
            .iter()
            .map(|c| desvio(c, &guia))
            .fold(0.0_f64, f64::max);
        let buracos = circ.len() as i64 - quad.len() as i64;
        println!(
            "  arte {w:>4.1}  ·  circulo {:>3} copias  ·  quadrado {:>3}  ⇒  BURACOS {buracos:>2}  \
             ·  desvio pior no quadrado {pior:.4} ({:.2}x a meia-altura)",
            circ.len(),
            quad.len(),
            pior / meia_altura
        );
    }

    // ⭐⭐ **O REGIME EM QUE DÓI** — a queixa nº 1 dos fóruns do Illustrator é *"apliquei o pincel a
    // um rectângulo pequeno e os lados sobrepõem-se nas quinas"*. A grandeza que manda não é o
    // tamanho da forma nem o da arte: é a RAZÃO entre eles.
    println!("\n  lado do quadrado / largura da arte  ->  o desvio, em multiplos da meia-altura");
    for lado in [2.0_f64, 3.0, 5.0, 7.0, 12.0, 20.0] {
        let art = arte(1.3, 1.0);
        let quad = quadrado(lado);
        let guia = guia_de(&quad);
        let copias = brush_along_path(&quad, &art, &s);
        if copias.is_empty() {
            println!("  lado {lado:>5.1}  ·  SEM COPIAS");
            continue;
        }
        let circ = brush_along_path(
            &crate::ellipse(
                [0.0, 0.0],
                lado * 4.0 / std::f64::consts::TAU,
                lado * 4.0 / std::f64::consts::TAU,
            ),
            &art,
            &s,
        );
        let pior = copias
            .iter()
            .map(|c| desvio(c, &guia))
            .fold(0.0_f64, f64::max);
        let acima: usize = copias
            .iter()
            .filter(|c| desvio(c, &guia) > meia_altura * 1.2)
            .count();
        println!(
            "  lado {lado:>5.1}  (lado/arte = {:>5.1})  copias {:>3}  buracos {:>2}  desvio pior \
             {:>5.2}x  ·  {acima} copia(s) acima de 1,2x",
            lado / 1.3,
            copias.len(),
            circ.len() as i64 - copias.len() as i64,
            pior / meia_altura
        );
    }

    // ⚠️ **A CÚSPIDE é o mecanismo, e mede-se directamente**: quantas posições de arco de uma volta
    // do quadrado devolvem tangente NULA? Um contorno de 4 quinas tem 4.
    let guia = guia_de(&quadrado(7.0));
    let mut cuspides = 0;
    for k in 0..=4000 {
        let arco = f64::from(k) / 4000.0 * guia.total();
        let (_, t) = guia.frame_at(arco);
        if t[0] == 0.0 && t[1] == 0.0 {
            cuspides += 1;
        }
    }
    println!("\n  posicoes de arco com tangente NULA numa volta (4001 amostras): {cuspides}");
}
