//! Gates do LÁPIS — arquivo irmão de `pencil.rs`.
//!
//! ⚠️ **A fixtura tem de conter o fenômeno**, e o fenômeno aqui é uma **mão**: um S paramétrico
//! amostrado denso, com tremor determinístico somado (nada de `rand` — um gate que flaka é um
//! gate que se silencia). Um traço de mouse limpo não exercita o decimador nem o ajuste; ele
//! passaria com o RDP desligado.

use super::*;

/// A dinâmica que estas fixtures passam: **um rato** (pressão cheia) e um relógio que anda um
/// passo por amostra.
///
/// ⚠️ Ela é PREMISSA, não asserção: os gates deste arquivo testam a GEOMETRIA do lápis, e a
/// largura variável tem gates próprios (`pencil_width_tests`). Passar a mesma dinâmica em todos
/// mantém a variável única — e o relógio anda porque um relógio parado é um caso degenerado com
/// gate dedicado, não o caso normal.
fn tick() -> crate::pencil_width::PenDynamics {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    crate::pencil_width::PenDynamics {
        pressure: 1.0,
        t_ns: u128::from(N.fetch_add(1, Ordering::Relaxed)) * 4_000_000,
    }
}
use ph2d_vec_scene::VecScene;

/// Mundo por pixel de tela nas fixturas — 1:1, para os limiares em px lerem direto em mundo.
const PX: f64 = 1.0;

/// A curva ideal que a "mão" tenta seguir: um S de largura 120 e amplitude 30.
fn ideal(t: f64) -> [f64; 2] {
    [120.0 * t, 30.0 * (t * std::f64::consts::TAU).sin()]
}

/// Tremor DETERMINÍSTICO de amplitude `amp` (px) — um par de senoides incomensuráveis, que é
/// ruído para o decimador (extremos locais densos) sem ser aleatório para o gate.
fn tremor(i: usize, amp: f64) -> [f64; 2] {
    let f = i as f64;
    [amp * (f * 1.9).sin() * 0.5, amp * (f * 2.7).cos() * 0.5]
}

/// `n` amostras do S com `amp` px de tremor — o que a mão de fato entrega.
fn hand(n: usize, amp: f64) -> Vec<[f64; 2]> {
    (0..n)
        .map(|i| {
            let p = ideal(i as f64 / (n - 1) as f64);
            let t = tremor(i, amp);
            [p[0] + t[0], p[1] + t[1]]
        })
        .collect()
}

/// Dirige o gesto INTEIRO pelo caminho do produto (press → moves → release) e devolve a cena
/// mais o id do traço, ou `None` se o release recusou.
fn draw(samples: &[[f64; 2]], fidelity: f64) -> (VecScene, Option<VecPathId>) {
    let mut scene = VecScene::new();
    let mut pencil = Pencil::default();
    pencil.set_fidelity_px(fidelity);
    let id = pencil.on_press(&mut scene, samples[0], PX, tick());
    for &p in &samples[1..] {
        pencil.on_drag(&mut scene, p, tick());
    }
    let kept = pencil.on_release(&mut scene);
    (scene, kept.then_some(id))
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// Distância de `p` ao caminho poligonal `poly` (a régua de "a curva ficou onde a mão passou").
fn dist_to_poly(p: [f64; 2], poly: &[[f64; 2]]) -> f64 {
    poly.windows(2)
        .map(|w| dist2_to_segment(p, w[0], w[1]).sqrt())
        .fold(f64::MAX, f64::min)
}

/// Amostra a spline de um path em `steps` pontos por segmento.
fn sample_path(scene: &VecScene, id: VecPathId, steps: usize) -> Vec<[f64; 2]> {
    let verts = &scene.path(id).expect("o traco existe").verts;
    let mut out = Vec::new();
    for i in 0..verts.len().saturating_sub(1) {
        let (p0, c1) = (verts[i].anchor, verts[i].out_handle);
        let (c2, p3) = (verts[i + 1].in_handle, verts[i + 1].anchor);
        for s in 0..=steps {
            let t = s as f64 / steps as f64;
            let mt = 1.0 - t;
            let (a, b, c, d) = (mt * mt * mt, 3.0 * mt * mt * t, 3.0 * mt * t * t, t * t * t);
            out.push([
                p0[0] * a + c1[0] * b + c2[0] * c + p3[0] * d,
                p0[1] * a + c1[1] * b + c2[1] * c + p3[1] * d,
            ]);
        }
    }
    out
}

/// **O traço aparece DESDE o press** — não no 2º move.
///
/// É o que o padrão da `ShapeTool` compra: o path vivo está na cena, então o preview é o render
/// normal. Sem isto o lápis pareceria não responder ao 1º contato, que é o momento em que o
/// artista decide se a ferramenta funciona.
#[test]
fn the_stroke_is_on_the_scene_from_the_press() {
    let mut scene = VecScene::new();
    let mut pencil = Pencil::default();
    let id = pencil.on_press(&mut scene, [10.0, 10.0], PX, tick());
    assert!(pencil.is_active());
    let path = scene.path(id).expect("o press pos o traco vivo na cena");
    assert_eq!(path.verts.len(), 1, "uma amostra, um vertice");
    assert!(path.stroke.is_some(), "o traco do lapis tem de ter TRACO");
    assert!(
        path.fill.is_none(),
        "o lapis desenha uma LINHA — preenche-la fecharia visualmente um caminho aberto"
    );
    assert!(!path.closed);
}

/// **A curva passa por onde a mão passou.** O oráculo é a APARÊNCIA: amostra-se a spline
/// resultante e mede-se a distância dela ao caminho que a mão deixou.
///
/// ⚠️ Este é o gate que separa Hobby de Schneider. Mutação que tem de sangrar: trocar o ajuste
/// interpolante por uma cúbica única entre as pontas — o desvio vai a **19,4** contra o bar de 3.
#[test]
fn the_curve_stays_where_the_hand_went() {
    let samples = hand(240, 0.0); // sem tremor: o que se mede é o AJUSTE, não o ruído
    let (scene, id) = draw(&samples, DEFAULT_FIDELITY_PX);
    let id = id.expect("um S de 120 px de largura e' um traco");
    let worst = sample_path(&scene, id, 8)
        .into_iter()
        .map(|p| dist_to_poly(p, &samples))
        .fold(0.0_f64, f64::max);
    assert!(
        worst < 3.0,
        "a curva fugiu {worst:.2} px do caminho da mao — um ajuste que nao passa pelas amostras \
         poe o traco onde a mao nao esteve"
    );
}

/// **As duas PONTAS são exatamente onde a mão começou e parou.** Uma ponta que escorrega é o
/// defeito mais visível de um lápis: o traço deixa de encostar no que o artista mirou.
#[test]
fn both_ends_land_exactly_on_the_hand() {
    let samples = hand(180, 0.6);
    let (scene, id) = draw(&samples, DEFAULT_FIDELITY_PX);
    let verts = &scene.path(id.expect("commitado")).expect("existe").verts;
    assert!(
        dist(verts[0].anchor, samples[0]) < 1e-9,
        "a ponta inicial escorregou"
    );
    assert!(
        dist(verts[verts.len() - 1].anchor, samples[samples.len() - 1]) < 1e-9,
        "a ponta final escorregou"
    );
}

/// **A fidelidade é uma DISTÂNCIA: mais tolerância, menos nós — monotonicamente.**
///
/// Um slider cuja resposta não é monótona é um slider que o artista não aprende. Os números
/// MEDIDOS estão no doc de [`DEFAULT_FIDELITY_PX`]; aqui o gate afirma a *forma* da resposta,
/// que é o que não pode regredir.
#[test]
fn more_fidelity_tolerance_means_fewer_nodes() {
    let samples = hand(480, 0.8);
    let mut prev = usize::MAX;
    for tol in [0.5, 1.0, 2.0, 4.0, 8.0] {
        let n = decimate(&samples, tol).len();
        assert!(
            n <= prev,
            "tolerancia {tol} deu MAIS nos ({n}) que a anterior ({prev}) — a resposta do slider \
             nao e' monotona"
        );
        assert!(
            n >= 2,
            "o decimador nunca pode devolver menos que as duas pontas"
        );
        prev = n;
    }
}

/// **Uma QUINA desenhada de propósito sobrevive a qualquer tolerância.**
///
/// É a propriedade do RDP que o torna o decimador certo aqui (ele nunca corta um extremo
/// local), e é o que faz um L desenhado à mão continuar sendo um L. Mutação que tem de sangrar:
/// decimar por passo fixo (1 amostra a cada N) — a quina cai fora quando ela não pousa no passo.
#[test]
fn a_deliberate_corner_survives_every_tolerance() {
    // Um L: 40 px para a direita, 40 px para cima, amostrado denso.
    let mut samples: Vec<[f64; 2]> = (0..=40).map(|i| [f64::from(i), 0.0]).collect();
    samples.extend((1..=40).map(|i| [40.0, f64::from(i)]));
    let corner = [40.0, 0.0];
    for tol in [0.5, 2.0, 8.0, 20.0] {
        let knots = decimate(&samples, tol);
        let closest = knots
            .iter()
            .map(|k| dist(*k, corner))
            .fold(f64::MAX, f64::min);
        assert!(
            closest < 1e-9,
            "tolerancia {tol}: a quina sumiu (no' mais proximo a {closest:.2} px) — o L virou uma \
             diagonal"
        );
    }
}

/// **Um clique perdido não deixa um traço** — e não deixa lixo na cena.
///
/// Sem isto, cada clique acidental com o lápis ativo criaria um path de comprimento zero:
/// invisível, selecionável, e um passo de undo por cima.
#[test]
fn a_stray_click_leaves_nothing_behind() {
    let mut scene = VecScene::new();
    let mut pencil = Pencil::default();
    pencil.on_press(&mut scene, [5.0, 5.0], PX, tick());
    pencil.on_drag(&mut scene, [5.4, 5.2], tick()); // abaixo do passo mínimo: nem é amostrado
    assert!(!pencil.on_release(&mut scene), "um toque nao e' um traco");
    assert!(
        scene.paths().is_empty(),
        "o clique perdido deixou um path na cena"
    );
    assert!(!pencil.is_active());
}

/// **Um traço commitado SOBREVIVE ao traço seguinte** — e o gesto acaba quando se solta.
///
/// ⚠️ Este gate nasceu de um defeito reportado pelo Enio (*"o traço desaparece no final"*), cuja
/// causa estava na SHELL: o braço de release do lápis vivia no `else` de um predicado que é
/// verdadeiro em modo Pencil, então `on_release` **nunca corria**. O `active` ficava para sempre, e
/// as duas consequências eram exactamente as que ele viu — o lápis **continuava a desenhar com o
/// botão em cima** (todo move seguinte entrava no traço) e o press seguinte **apagava o traço
/// anterior**, porque `on_press` remove o path que encontra vivo.
///
/// A limpeza do path órfão no `on_press` está CERTA e fica (um gesto que não fechou não pode deixar
/// lixo na cena); o que estava errado era um chamador que nunca fechava o gesto. Este gate afirma o
/// lado do MOTOR — soltar encerra —, que é o que torna aquela limpeza inofensiva; o lado do
/// despacho é gateado no `shells/desktop/tests/the_pencil_owns_its_whole_gesture.rs`.
#[test]
fn a_committed_stroke_survives_the_next_one() {
    let mut scene = VecScene::new();
    let mut pencil = Pencil::default();

    pencil.on_press(&mut scene, [0.0, 0.0], PX, tick());
    for &p in &hand(40, 0.0)[1..] {
        pencil.on_drag(&mut scene, p, tick());
    }
    let first = pencil.on_release(&mut scene);
    assert!(first, "o 1o traco foi commitado");
    assert!(
        !pencil.is_active() && pencil.active_path().is_none(),
        "soltar NAO encerrou o gesto — o lapis continua vivo e o proximo move ainda desenha, com o          botao em cima"
    );

    // O 2º traço, noutro lugar.
    pencil.on_press(&mut scene, [40.0, 40.0], PX, tick());
    for i in 1..40 {
        let t = f64::from(i) / 39.0;
        pencil.on_drag(&mut scene, [40.0 + 30.0 * t, 40.0 + 10.0 * t], tick());
    }
    assert!(pencil.on_release(&mut scene), "o 2o traco foi commitado");
    assert_eq!(
        scene.paths().len(),
        2,
        "o traco anterior DESAPARECEU quando o seguinte comecou"
    );
}

/// **Abortar não deixa rastro** (o botão direito / o Esc no meio do gesto).
#[test]
fn cancelling_removes_the_live_stroke() {
    let mut scene = VecScene::new();
    let mut pencil = Pencil::default();
    pencil.on_press(&mut scene, [0.0, 0.0], PX, tick());
    for &p in &hand(40, 0.0)[1..] {
        pencil.on_drag(&mut scene, p, tick());
    }
    assert_eq!(scene.paths().len(), 1);
    pencil.cancel(&mut scene);
    assert!(
        scene.paths().is_empty(),
        "abortar deixou o traco vivo na cena"
    );
    assert!(!pencil.is_active());
    assert!(pencil.samples().is_empty());
}

/// **O passo mínimo descarta amostras redundantes** — um cursor parado num pixel não pode
/// engordar o traço com nós coincidentes (o caso degenerado que o solver tem de defender).
#[test]
fn a_still_cursor_does_not_add_samples() {
    let mut scene = VecScene::new();
    let mut pencil = Pencil::default();
    pencil.on_press(&mut scene, [0.0, 0.0], PX, tick());
    for _ in 0..50 {
        assert!(
            !pencil.on_drag(&mut scene, [0.3, 0.2], tick()),
            "uma amostra a 0,36 px passou o passo minimo"
        );
    }
    assert_eq!(pencil.samples().len(), 1);
}

/// **A densidade de nós é do GESTO, não do zoom** — o mesmo traço desenhado ampliado 4× tem de
/// dar (aproximadamente) os mesmos nós, porque a tolerância é em pixels de TELA.
///
/// ⚠️ Mutação que tem de sangrar: usar a tolerância como se fosse mundo (ignorar o
/// `px_to_world`) ⇒ a contagem muda por um fator de 4 quando só o zoom mudou.
#[test]
fn the_node_count_follows_the_gesture_not_the_zoom() {
    let samples = hand(300, 0.8);
    let (a, ida) = draw(&samples, DEFAULT_FIDELITY_PX);
    // "Ampliar 4×" = a mesma mão a cobrir 4× o mundo, com 1 px de tela valendo ¼ de mundo.
    let big: Vec<[f64; 2]> = samples.iter().map(|p| [p[0] * 4.0, p[1] * 4.0]).collect();
    let mut scene_b = VecScene::new();
    let mut pencil = Pencil::default();
    pencil.set_fidelity_px(DEFAULT_FIDELITY_PX);
    let idb = pencil.on_press(&mut scene_b, big[0], PX * 4.0, tick());
    for &p in &big[1..] {
        pencil.on_drag(&mut scene_b, p, tick());
    }
    assert!(pencil.on_release(&mut scene_b));
    let na = a.path(ida.expect("commitado")).expect("existe").verts.len();
    let nb = scene_b.path(idb).expect("existe").verts.len();
    let ratio = na as f64 / nb as f64;
    assert!(
        (0.8..=1.25).contains(&ratio),
        "o mesmo gesto deu {na} nos a 1x e {nb} a 4x (razao {ratio:.2}) — a tolerancia esta' a ser \
         lida em MUNDO, entao dar zoom mudaria o traco"
    );
}

/// **Mexer a fidelidade re-ajusta o traço EM CURSO.** É o que faz o slider ser legível: o
/// artista vê a resposta enquanto arrasta, em vez de descobri-la no traço seguinte.
#[test]
fn changing_the_fidelity_refits_the_live_stroke() {
    let samples = hand(300, 0.8);
    let mut scene = VecScene::new();
    let mut pencil = Pencil::default();
    pencil.set_fidelity_px(0.5);
    let id = pencil.on_press(&mut scene, samples[0], PX, tick());
    for &p in &samples[1..] {
        pencil.on_drag(&mut scene, p, tick());
    }
    let fine = scene.path(id).expect("existe").verts.len();
    pencil.set_fidelity_px(8.0);
    // Um move a mais (a shell reajusta no frame seguinte, que é quando a amostra chega).
    pencil.on_drag(&mut scene, [200.0, 0.0], tick());
    let coarse = scene.path(id).expect("existe").verts.len();
    assert!(
        coarse < fine,
        "a fidelidade nao alcancou o traco vivo ({fine} -> {coarse} nos)"
    );
}

/// Sonda: os números do doc de [`DEFAULT_FIDELITY_PX`] (contagem de nós e desvio por tolerância).
/// `cargo test -p ph2d-vec-edit --release measure_pencil_fidelity -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição; rode com --nocapture"]
fn measure_pencil_fidelity() {
    let samples = hand(480, 0.8);
    let clean: Vec<[f64; 2]> = (0..480).map(|i| ideal(f64::from(i) / 479.0)).collect();
    println!("\namostras={} tremor=0,8 px", samples.len());
    for tol in [0.5, 1.0, 2.0, 4.0, 8.0] {
        let (scene, id) = draw(&samples, tol);
        let id = id.expect("commitado");
        let n = scene.path(id).expect("existe").verts.len();
        let worst = sample_path(&scene, id, 8)
            .into_iter()
            .map(|p| dist_to_poly(p, &clean))
            .fold(0.0_f64, f64::max);
        println!("  fidelity {tol:>4} px -> {n:>3} nos · desvio max da curva ideal {worst:.2} px");
    }
}

/// Sonda: o custo de um move (decimar + ajustar) contra o número de amostras acumuladas — o
/// número que justifica ajustar AO VIVO em vez de só no release.
/// `cargo test -p ph2d-vec-edit --release measure_pencil_fit -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição; rode com --nocapture"]
fn measure_pencil_fit() {
    println!();
    for n in [50usize, 200, 500, 2000] {
        let samples = hand(n, 0.8);
        let mut scene = VecScene::new();
        let mut pencil = Pencil::default();
        pencil.set_fidelity_px(DEFAULT_FIDELITY_PX);
        pencil.on_press(&mut scene, samples[0], PX, tick());
        for &p in &samples[1..n - 1] {
            pencil.on_drag(&mut scene, p, tick());
        }
        // O move MAIS CARO é o último: a lista de amostras está cheia.
        let mut best = f64::MAX;
        for _ in 0..32 {
            let t = std::time::Instant::now();
            pencil.on_drag(&mut scene, samples[n - 1], tick());
            best = best.min(t.elapsed().as_secs_f64() * 1000.0);
            pencil.samples.pop();
        }
        let nodes = pencil.fit().len();
        println!("  {n:>4} amostras -> {nodes:>3} nos · move {best:.3} ms");
    }
}
