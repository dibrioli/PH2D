use super::*;

fn max_dev(a: &[[f32; 2]], b: &[[f32; 2]]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(p, q)| ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt())
        .fold(0.0, f32::max)
}

/// Deixa a corda cair de uma pose recta com as pontas PARADAS, durante `seconds`, a `fps`.
fn settle(fps: f32, seconds: f32) -> Vec<[f32; 2]> {
    let dt = 1.0 / fps;
    let mut t = Tether::new(NODES);
    let (control, effect) = ([100.0, 100.0], [400.0, 180.0]);
    t.advance(control, effect, dt, true); // o 1.º põe na recta (`fresh`)
    for _ in 0..(seconds * fps).round() as usize {
        t.advance(control, effect, dt, true);
    }
    t.points().to_vec()
}

/// O mesmo, com o efeito a andar em função do TEMPO.
fn drive(fps: f32, seconds: f32) -> Vec<[f32; 2]> {
    let dt = 1.0 / fps;
    let mut t = Tether::new(NODES);
    let control = [100.0, 100.0];
    for s in 0..(seconds * fps).round() as usize {
        let time = s as f32 * dt;
        t.advance(
            control,
            [300.0 + 60.0 * time, 140.0 + 30.0 * time],
            dt,
            true,
        );
    }
    t.points().to_vec()
}

/// ⭐ **A forma é facto do relógio de parede, não da taxa de quadros.**
///
/// Com as pontas PARADAS a entrada é idêntica nas duas taxas, então o que sobra mede só o solver —
/// e ele tem de dar a MESMA forma. É o gate que impede a corda de ser bonita na máquina de quem a
/// escreveu.
///
/// **Mutação que deve sangrar:** trocar o passo interno fixo por `dt` do quadro (com ou sem o
/// `dt/dt_prev` do plano). Medido: o TCV sozinho erra **29,7 px** num gesto de 1,5 s, porque cura a
/// integração e deixa de pé o amortecimento por-quadro e — a maior — a RIGIDEZ, que é `ITERS`
/// passagens por QUADRO e portanto quatro vezes mais forte a 120 fps.
#[test]
fn the_rope_is_a_fact_of_the_wall_clock_not_of_the_frame_rate() {
    let dev = max_dev(&settle(30.0, 1.5), &settle(120.0, 1.5));
    assert!(
        dev < 0.05,
        "pontas paradas: 30 e 120 fps têm de dar a MESMA forma; desvio {dev:.4} px"
    );
}

/// E com as pontas a MOVER-SE o resíduo é da ENTRADA, não do solver — nomeado com o número em vez
/// de escondido: a 30 fps a corda persegue um alvo amostrado 4× mais raramente.
///
/// ⚠️ A barra é o que a medição deu (1,68 px) com folga, e o gate diz de que é o resíduo. Sem esta
/// distinção, alguém apertaria a barra do gate acima até ela mentir.
#[test]
fn a_moving_endpoint_leaves_a_residue_and_it_is_the_input_sampling() {
    let dev = max_dev(&drive(30.0, 1.5), &drive(120.0, 1.5));
    assert!(
        dev < 4.0,
        "resíduo de amostragem da entrada, esperado ~1,7 px; medido {dev:.4} px"
    );
    assert!(
        dev > 0.05,
        "e ele EXISTE — se fosse zero, a fixture não continha o fenómeno que este gate nomeia"
    );
}

/// As pontas são exactamente o controlo e o efeito. **Uma corda que derive das pontas deixa de
/// descrever a relação que ela existe para mostrar.**
#[test]
fn a_pinned_end_is_exactly_the_control() {
    let mut t = Tether::new(NODES);
    let (control, effect) = ([100.0, 100.0], [400.0, 180.0]);
    for _ in 0..200 {
        t.advance(control, effect, 1.0 / 60.0, true);
    }
    let p = t.points();
    assert_eq!(p[0], control, "a ponta do controlo não deriva");
    assert_eq!(p[p.len() - 1], effect, "nem a do efeito");
}

/// **Em Discreto ela é uma RECTA e não simula nada** (plano §5.3): o significado sobrevive, o peso
/// é que sai. Simular e desenhar recto seria custo sem efeito.
///
/// ⚠️ **A fixture SIMULA PRIMEIRO, e é isso que a torna um gate.** A primeira versão começava em
/// Discreto e a mutação — tirar o `!simulate` do early-return — **sobreviveu**: com a corda ainda
/// `fresh`, o outro braço da mesma condição devolvia-a à recta na mesma, e o teste não distinguia
/// *não simulou* de *simulou e foi reposto*. É preciso GASTAR o `fresh` em Expressivo antes de
/// perguntar o que Discreto faz.
#[test]
fn the_discrete_character_draws_a_straight_line_and_simulates_nothing() {
    let mut t = Tether::new(NODES);
    let (control, effect) = ([100.0, 100.0], [400.0, 100.0]);
    // Gasta o `fresh`: a partir daqui a corda TEM pose e uma simulação a mais seria visível.
    for _ in 0..60 {
        t.advance(control, effect, 1.0 / 60.0, true);
    }
    assert!(
        t.points()[NODES / 2][1] > 110.0,
        "a fixture não contém o fenómeno: a corda tinha de estar pendurada antes de trocar de carácter"
    );
    for _ in 0..200 {
        t.advance(control, effect, 1.0 / 60.0, false);
    }
    for (i, p) in t.points().iter().enumerate() {
        let x = i as f32 / (NODES - 1) as f32;
        assert!(
            (p[1] - 100.0).abs() < 1e-3,
            "nó {i} caiu {:.3} px — em Discreto a corda é a recta",
            p[1] - 100.0
        );
        assert!((p[0] - (100.0 + 300.0 * x)).abs() < 1e-3);
    }
}

/// **A corda PENDURA, e os elos ficam do tamanho de repouso.**
///
/// ⚠️ Nasceu porque a mutação que desliga metade da correcção de distância **sobreviveu** ao gate
/// das pontas: aquele mede só `p[0]` e `p[n-1]`, que continuam pinados enquanto o meio se desfaz.
/// *Pinar as pontas e resolver a corda são duas perguntas, e precisavam de dois gates.*
#[test]
fn the_rope_hangs_and_its_links_keep_their_rest_length() {
    let mut t = Tether::new(NODES);
    let (control, effect) = ([100.0, 100.0], [400.0, 100.0]);
    for _ in 0..240 {
        t.advance(control, effect, 1.0 / 60.0, true);
    }
    let p = t.points();
    let span = 300.0_f32;
    let rest = span * 1.22 / (NODES - 1) as f32;
    for a in 0..NODES - 1 {
        let d = [p[a + 1][0] - p[a][0], p[a + 1][1] - p[a][1]];
        let l = (d[0] * d[0] + d[1] * d[1]).sqrt();
        assert!(
            (l - rest).abs() < rest * 0.35,
            "elo {a} mede {l:.2} px contra um repouso de {rest:.2} — a restrição não está a resolver"
        );
    }
    // E o conjunto pendura: o ponto mais baixo fica bem abaixo da linha das pontas.
    let lowest = p.iter().map(|q| q[1]).fold(f32::MIN, f32::max);
    assert!(
        lowest > 140.0,
        "o ponto mais baixo está em {lowest:.1} — uma corda com folga 1,22 tem de cair bem mais"
    );
}

/// O carácter pode mudar A MEIO, e a corda tem de cair **da recta nova** — nunca voar da pose que
/// tinha antes de o artista escolher Expressivo.
#[test]
fn switching_to_expressive_starts_from_the_line_not_from_the_old_pose() {
    let mut t = Tether::new(NODES);
    for _ in 0..30 {
        t.advance([100.0, 100.0], [400.0, 100.0], 1.0 / 60.0, false);
    }
    // Muda de sítio E de carácter no mesmo quadro.
    t.advance([600.0, 500.0], [900.0, 500.0], 1.0 / 60.0, true);
    for p in t.points() {
        assert!(
            (p[1] - 500.0).abs() < 1e-3,
            "o 1.º quadro a simular ainda é a recta NOVA: sem isso a corda voa do sítio antigo"
        );
    }
}

/// Controlo e efeito no mesmo ponto: não há corda para desenhar, e quem pergunta é a porta.
#[test]
fn a_zero_length_tether_is_not_drawable() {
    assert!(!Tether::is_drawable([10.0, 10.0], [10.0, 10.4]));
    assert!(Tether::is_drawable([10.0, 10.0], [40.0, 10.0]));
}

/// **Um quadro lento não compra passos sem tecto.** Com um engasgo de meio segundo, o `MAX_STEPS`
/// corta e o resto é DEITADO FORA — se fosse dívida, o quadro seguinte pagaria os passos deste e a
/// realimentação em `dt` que a sim do Wet Paint mediu reapareceria aqui.
#[test]
fn a_stalled_frame_does_not_buy_unbounded_steps() {
    let mut t = Tether::new(NODES);
    let (control, effect) = ([100.0, 100.0], [400.0, 180.0]);
    t.advance(control, effect, 1.0 / 60.0, true);
    t.advance(control, effect, 0.5, true); // engasgo: 60 passos pedidos, 8 permitidos
    let after_stall = t.points().to_vec();
    t.advance(control, effect, 1.0 / 60.0, true);
    let next = t.points().to_vec();
    // Um quadro normal a seguir ao engasgo anda o que um quadro normal anda — não os 52 passos
    // que teriam ficado em dívida.
    let moved = max_dev(&after_stall, &next);
    assert!(
        moved < 8.0,
        "o quadro seguinte ao engasgo andou {moved:.2} px — está a pagar dívida acumulada"
    );
}

/// SONDA: os números que decidem a wave, impressos em vez de afirmados.
#[test]
#[ignore = "sonda: rode com -- --ignored --nocapture"]
fn measure_the_shape_across_frame_rates() {
    println!(
        "[tether] pontas PARADAS, 30 vs 120 fps: {:.4} px (o solver)",
        max_dev(&settle(30.0, 1.5), &settle(120.0, 1.5))
    );
    println!(
        "[tether] pontas a MOVER, 30 vs 120 fps: {:.4} px (a amostragem da entrada)",
        max_dev(&drive(30.0, 1.5), &drive(120.0, 1.5))
    );
}

/// **As duas PONTAS da curva são exactas.**
///
/// O truque da polilinha suave passa pelos pontos MÉDIOS, então é fácil escrever uma versão em que
/// a curva começa e acaba a meio caminho do primeiro e do último elo — e aí a corda descola da
/// âncora e do card, desenhando uma relação que não existe.
///
/// **Mutação que deve sangrar:** trocar o `quad_to(p[n-2], p[n-1])` final por
/// `quad_to(p[n-2], mid(p[n-2], p[n-1]))`.
#[test]
fn the_curve_starts_at_the_control_and_ends_at_the_effect() {
    let mut t = Tether::new(NODES);
    t.advance([10.0, 10.0], [210.0, 60.0], 0.5, true);
    let pts = t.points().to_vec();
    let path = t.path();
    let els: Vec<_> = path.elements().to_vec();
    let first = match els.first() {
        Some(ph2d_vector::PathEl::MoveTo(p)) => *p,
        other => panic!("a curva não começa por um MoveTo: {other:?}"),
    };
    let last = match els.last() {
        Some(ph2d_vector::PathEl::QuadTo(_, p)) => *p,
        other => panic!("a curva não acaba por um QuadTo: {other:?}"),
    };
    let d0 = (first.x - f64::from(pts[0][0])).hypot(first.y - f64::from(pts[0][1]));
    let dn = (last.x - f64::from(pts[NODES - 1][0])).hypot(last.y - f64::from(pts[NODES - 1][1]));
    assert!(d0 < 1e-6, "a ponta do CONTROLO descolou {d0:.4} px");
    assert!(dn < 1e-6, "a ponta do EFEITO descolou {dn:.4} px");
}

/// **Subir a contagem de nós resolve a MESMA curva, não outra.**
///
/// O comprimento de repouso de um elo é `SLACK · distância / (n − 1)`, então o comprimento TOTAL
/// da corda não depende de `n` — o que muda é a finura. Sem esta propriedade, mexer na resolução
/// (que é número de aparência, decidido no smoke) mudaria a silhueta que o Enio aprovou, e o
/// próximo a subir a contagem descobria-o na tela.
///
/// O oráculo é a FLECHA: quanto a corda pendura abaixo da recta entre as pontas.
///
/// **Mutação que deve sangrar:** fazer o comprimento de repouso do elo constante em vez de
/// dividido por `n − 1`.
#[test]
fn more_nodes_resolve_the_same_hang_not_a_different_one() {
    let sag = |n: usize| {
        let mut t = Tether::new(n);
        // Assenta: a flecha é o estado de repouso, não o primeiro quadro.
        for _ in 0..240 {
            t.advance([0.0, 0.0], [200.0, 0.0], 1.0 / 60.0, true);
        }
        t.points().iter().map(|p| p[1]).fold(f32::MIN, f32::max)
    };
    let (a, b) = (sag(12), sag(28));
    let rel = (a - b).abs() / a.max(1.0);
    assert!(
        rel < 0.12,
        "a flecha mudou {:.1}% ao dobrar os nós (12 nós: {a:.2} px, 28 nós: {b:.2} px): a resolução \
         está a mudar a SILHUETA, e aí o número de nós deixa de ser livre",
        rel * 100.0
    );
}

/// **Quanto o número de nós e o número de iterações mexem na FLECHA** — a tabela que decide se a
/// resolução é livre ou se está acoplada ao solver.
#[test]
#[ignore = "sonda"]
fn measure_how_the_sag_depends_on_nodes_and_iterations() {
    println!("  nós | flecha (px)");
    for n in [8_usize, 12, 16, 20, 24, 28, 36, 48] {
        let mut t = Tether::new(n);
        for _ in 0..240 {
            t.advance([0.0, 0.0], [200.0, 0.0], 1.0 / 60.0, true);
        }
        let sag = t.points().iter().map(|p| p[1]).fold(f32::MIN, f32::max);
        // O comprimento REAL da corda, para separar "estica" de "resolve melhor".
        let len: f32 = t
            .points()
            .windows(2)
            .map(|w| ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt())
            .sum();
        println!(
            "  {n:3} | {sag:6.2}  (comprimento {len:6.2}, folga pedida {:.2})",
            200.0 * super::SLACK
        );
    }
}

/// **O reduced motion PARA o crescimento e deixa a tinta em paz.**
///
/// Os dois eixos vivem no MESMO escalar (uma track por chip), então a separação não pode vir da
/// [`ph2d_editor_core::motion::UiMotion::law`] — ela é por `Role`. Vem do consumidor: a geometria
/// pergunta, a cor não.
#[test]
fn reduced_motion_stops_the_growth_and_leaves_the_tint_alone() {
    use crate::motion::{UiCharacter, UiMotion, hover_lift};
    use crate::zones::Rect;
    let r = Rect::new(10.0, 10.0, 36.0, 36.0);
    let mut m = UiMotion::default();
    m.set_character(UiCharacter::Expressive);
    assert!(m.travels(), "sem reduced, um chip pode mexer-se");
    let grown = hover_lift(r, 1.0, m.travels());
    assert!(grown.w > r.w, "o chip devia ter crescido e não cresceu");

    m.set_reduced_motion(true);
    assert!(!m.travels(), "com reduced, nada se mexe");
    let still = hover_lift(r, 1.0, m.travels());
    assert!(
        (still.w - r.w).abs() < 1e-6 && (still.x - r.x).abs() < 1e-6,
        "com reduced motion o chip ainda cresce {:.2} px",
        still.w - r.w
    );
    // ⚠️ E a TINTA sobrevive: `Fade` não é morta pelo reduced, então o par
    // *Expressivo + reduced* continua a ter uma transição — só não tem percurso.
    assert!(
        m.law(crate::motion::Role::Fade).is_some(),
        "o reduced motion matou o fade, e não devia: o gatilho vestibular é a ÁREA a \
         deslocar-se, não a tinta a mudar"
    );
}
