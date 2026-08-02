//! Gates do **TWIST** e do **LOCAL SCALE** — a terceira lei, a que gira em
//! torno de uma âncora.
//!
//! Filho do MESMO pai que os outros gates de verbo, então `use super::*`
//! alcança o `sphere` compartilhado.
//!
//! ⚠️ **A fixture aqui é o gesto INTEIRO, entregue em N eventos**, porque a
//! entrega da wave é justamente que o resultado **não depende do N**. Um dab
//! solto não contém esse fenômeno — ele é a resposta certa por vácuo.

use super::*;

/// O eixo do olho na cena destes gates: a esfera é unitária e o barro é pego no
/// polo `+Z`, então quem olha está em `−Z` e o eixo aponta para o observador.
const EYE: [f32; 3] = [0.0, 0.0, -1.0];
/// Onde a mão pega.
const PIVOT: [f32; 3] = [0.0, 0.0, 1.0];

fn turn(verb: Verb, radius: f32) -> Brush {
    Brush {
        verb,
        radius,
        strength: 1.0,
        ..Brush::default()
    }
}

/// **O laço do shell**: o gesto total `amount` entregue em `events` eventos, e
/// cada evento carrega o TOTAL acumulado até ele — que é o que
/// [`Grip::Turn`] especifica e o que `turn_at` de fato manda.
fn sweep_turn(brush: &Brush, amount: f32, events: usize) -> (ph2d_mesh::Mesh, Vec<[f32; 3]>) {
    let mut mesh = sphere();
    let before = snapshot(&mesh);
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    for e in 1..=events {
        let so_far = amount * e as f32 / events as f32;
        let dab = match brush.verb.grip() {
            crate::Grip::Turn(crate::Amount::Angle) => {
                Dab::turning(PIVOT, brush.radius, EYE, so_far)
            }
            _ => Dab::scaling(PIVOT, brush.radius, EYE, so_far),
        };
        stroke.dab(&mut mesh, brush, &dab, Symmetry::default());
    }
    (mesh, before)
}

/// A distância de um ponto ao EIXO da torção (a reta que passa pelo pivô na
/// direção do olho). Com `EYE` no eixo `Z`, é o raio no plano `XY`.
fn axis_radius(p: [f32; 3]) -> f32 {
    (p[0] * p[0] + p[1] * p[1]).sqrt()
}

/// **A ENTREGA do Twist**: o barro roda em torno da âncora.
///
/// O oráculo é o ÂNGULO no plano da tela, e ele é o único que separa uma torção
/// de qualquer outra coisa: um Draw levanta (muda `z`), um Local Scale afasta
/// (muda o raio), e só a torção muda a **fase** deixando o raio onde estava.
#[test]
fn a_twist_turns_the_clay_around_the_anchor() {
    let brush = turn(Verb::Twist, 0.5);
    let (mesh, before) = sweep_turn(&brush, 1.0, 8);
    // O vértice que mais girou, e quanto.
    let mut worst = 0.0f32;
    let mut on_axis = 0.0f32;
    for (a, b) in before.iter().zip(mesh.positions()) {
        let r = axis_radius(*a);
        if r < 1e-4 {
            // Um vértice em cima do eixo: uma rotação o deixa onde está.
            on_axis = on_axis.max(
                ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt(),
            );
            continue;
        }
        let phase = (a[0] * b[1] - a[1] * b[0]).atan2(a[0] * b[0] + a[1] * b[1]);
        worst = worst.max(phase.abs());
    }
    assert!(
        worst > 0.5,
        "o Twist mal girou o barro (maior fase {worst:.4} rad)"
    );
    assert!(
        on_axis < 1e-5,
        "o vértice EM CIMA do eixo andou {on_axis:.6} — uma rotação o deixa parado"
    );
}

/// ⚠️ **O SENTIDO, e ele não se argumenta — mede-se NA TELA.**
///
/// Este gate nasceu VERMELHO: o [`Dab::eye`] aponta *do olho para a superfície*
/// (para DENTRO da tela), e girar em torno dele pela regra da mão direita sai
/// **horário** para quem olha — medido, um vértice à direita da âncora descia
/// `−0,1063` com um gesto positivo. Varrer o dedo no anti-horário torceria o
/// barro ao contrário, que é *manipulação direta invertida*: o mesmo erro que o
/// smoke pegou nos dois sinais da órbita, e o mesmo remédio (o gate mede o barro
/// na tela em vez de argumentar sobre sinais).
///
/// A cena: o observador em `+Z` olhando para `−Z`, então `right = +X` e
/// `up = +Y`. Com um `amount` **positivo**, um vértice à DIREITA da âncora tem
/// de SUBIR — anti-horário, o mesmo lado para que a mão varreu.
#[test]
fn a_positive_sweep_turns_the_clay_the_same_way_the_hand_swept() {
    let mut mesh = sphere();
    let before = snapshot(&mesh);
    let brush = turn(Verb::Twist, 0.5);
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    stroke.dab(
        &mut mesh,
        &brush,
        &Dab::turning(PIVOT, brush.radius, EYE, 0.8),
        Symmetry::default(),
    );
    // O vértice mais à direita da âncora, a meio raio.
    let mut best = (f32::MAX, 0usize);
    for (i, p) in before.iter().enumerate() {
        let d = (p[0] - 0.25).powi(2) + p[1].powi(2) + (p[2] - 0.96).powi(2);
        if d < best.0 {
            best = (d, i);
        }
    }
    let (a, b) = (before[best.1], mesh.positions()[best.1]);
    let dy = b[1] - a[1];
    assert!(
        dy > 0.05,
        "um vértice à DIREITA da âncora andou {dy:+.4} em `y`: com um gesto positivo ele tem de \
         SUBIR (anti-horário na tela), senão o barro torce ao contrário da mão"
    );
}

/// ⚠️ **O gate que separa uma ROTAÇÃO de um lerp, e é ele que a corda quebra.**
///
/// Se o alvo fosse a posição girada pelo ângulo CHEIO e o peso entrasse pelo
/// `accum` (o `lerp(base, target, w)` dos onze verbos de carimbo), o vértice
/// andaria pela **corda** do arco em vez de pela circunferência — e a distância
/// dele ao eixo **encolheria**, tanto mais quanto maior o giro. Com o peso
/// dentro do ângulo, ela é a mesma no fim.
#[test]
fn the_twist_keeps_every_vertex_at_its_distance_from_the_axis() {
    let brush = turn(Verb::Twist, 0.5);
    // Um ângulo GRANDE de propósito: a corda e o arco só divergem de verdade
    // longe do zero, e um gate com meio radiano ficaria verde sobre o defeito.
    let (mesh, before) = sweep_turn(&brush, 2.0, 6);
    let mut worst = 0.0f32;
    for (a, b) in before.iter().zip(mesh.positions()) {
        let r = axis_radius(*a);
        if r < 1e-3 {
            continue;
        }
        worst = worst.max((axis_radius(*b) - r).abs() / r);
    }
    assert!(
        worst < 1e-4,
        "o raio ao eixo mudou {:.2}% — o barro está andando pela CORDA, não pelo arco",
        worst * 100.0
    );
}

/// **A LEI DO TRAÇO, no canal do Twist.**
///
/// O mesmo ângulo varrido, entregue em 1, 2, 8 e 64 eventos, tem de dar a MESMA
/// forma: o alvo é função do `pre` congelado e do gesto TOTAL, então o número de
/// parcelas não entra em lugar nenhum.
///
/// ⚠️ É o gate que o original **não** passaria: `Twist.js` aplica o incremento de
/// cada evento sobre o resultado do anterior.
#[test]
fn the_twist_is_a_fact_of_the_gesture_not_of_the_polling_rate() {
    let brush = turn(Verb::Twist, 0.5);
    let (one, _) = sweep_turn(&brush, 1.4, 1);
    for events in [2usize, 8, 64] {
        let (many, _) = sweep_turn(&brush, 1.4, events);
        let mut worst = 0.0f32;
        for (a, b) in one.positions().iter().zip(many.positions()) {
            worst = worst.max(
                ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt(),
            );
        }
        assert!(
            worst < 1e-5,
            "{events} eventos deram uma forma {worst:.6} diferente de um evento só"
        );
    }
}

/// **Varrer de volta devolve o barro ao lugar** — a promessa do `pre` congelado,
/// agora no canal do giro.
#[test]
fn sweeping_the_twist_back_gives_the_clay_back() {
    let brush = turn(Verb::Twist, 0.5);
    let mut mesh = sphere();
    let before = snapshot(&mesh);
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    // Vai até 1,5 rad e volta a zero, passando por vários pontos.
    for a in [0.5f32, 1.0, 1.5, 1.0, 0.5, 0.0] {
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::turning(PIVOT, brush.radius, EYE, a),
            Symmetry::default(),
        );
    }
    let residue = max_shift(&before, &mesh);
    assert!(
        residue < 1e-5,
        "sobrou {residue:.6} depois de varrer de volta ao zero"
    );
}

/// **A ENTREGA do Local Scale**: o barro se afasta da âncora, e o gesto ao
/// contrário o traz de volta para dentro.
#[test]
fn the_local_scale_pushes_the_clay_away_from_the_anchor_and_pulls_it_back() {
    let brush = turn(Verb::LocalScale, 0.5);
    let grow = sweep_turn(&brush, 0.6, 8);
    let shrink = sweep_turn(&brush, -0.6, 8);
    // ⚠️ **A razão é medida num ANEL, não por um extremo sobre a malha inteira.**
    // O falloff vai a zero na borda da pegada, então lá `db/da ≈ 1` sempre — um
    // `max` mede a BORDA nos dois casos e reporta 1,000 para um gesto que
    // encolheu tudo. Um anel dentro da pegada mede o que o gesto de fato faz.
    let ring_ratio = |m: &ph2d_mesh::Mesh, before: &[[f32; 3]]| {
        let (mut acc, mut n) = (0.0f32, 0u32);
        for (a, b) in before.iter().zip(m.positions()) {
            let da = ((a[0] - PIVOT[0]).powi(2) + (a[1] - PIVOT[1]).powi(2)).sqrt();
            if !(0.1..0.3).contains(&da) {
                continue;
            }
            let db = ((b[0] - PIVOT[0]).powi(2) + (b[1] - PIVOT[1]).powi(2)).sqrt();
            acc += db / da;
            n += 1;
        }
        assert!(n > 20, "o anel de medida pegou só {n} vértices");
        acc / n as f32
    };
    let up = ring_ratio(&grow.0, &grow.1);
    let down = ring_ratio(&shrink.0, &shrink.1);
    assert!(up > 1.15, "o Local Scale mal inflou (razão média {up:.3})");
    assert!(
        down < 0.85,
        "o gesto ao contrário devia ENCOLHER, e a razão média foi {down:.3}"
    );
}

/// ⚠️ **O barro NÃO vira do avesso.** Um fator de escala negativo não encolhe
/// mais — ele reflete a pegada através da âncora, e a malha fica com as normais
/// para dentro. O clamp em zero é onde a operação deixa de estar definida.
#[test]
fn the_local_scale_never_turns_the_clay_inside_out() {
    let brush = turn(Verb::LocalScale, 0.5);
    // Um gesto absurdo: encolher três vezes mais do que o colapso.
    let (mesh, before) = sweep_turn(&brush, -3.0, 4);
    for (a, b) in before.iter().zip(mesh.positions()) {
        let da = [a[0] - PIVOT[0], a[1] - PIVOT[1]];
        let db = [b[0] - PIVOT[0], b[1] - PIVOT[1]];
        // O vértice pode ter colapsado na âncora, mas nunca atravessado para o
        // outro lado dela.
        let dot = da[0] * db[0] + da[1] * db[1];
        assert!(
            dot >= -1e-6,
            "um vértice atravessou a âncora: {a:?} -> {b:?} (produto {dot:.6})"
        );
    }
}

/// A mesma lei do traço, no canal da escala.
#[test]
fn the_local_scale_is_a_fact_of_the_gesture_not_of_the_polling_rate() {
    let brush = turn(Verb::LocalScale, 0.5);
    let (one, _) = sweep_turn(&brush, 0.7, 1);
    for events in [2usize, 8, 64] {
        let (many, _) = sweep_turn(&brush, 0.7, events);
        let mut worst = 0.0f32;
        for (a, b) in one.positions().iter().zip(many.positions()) {
            worst = worst.max(
                ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt(),
            );
        }
        assert!(
            worst < 1e-5,
            "{events} eventos deram uma forma {worst:.6} diferente de um evento só"
        );
    }
}

/// ⚠️ **A pegada é CONGELADA no pen-down, e sem isso o Local Scale deixa barro
/// para trás no caminho de volta.** Uma consulta refeita a cada dab **perde** os
/// vértices que já saíram do raio, e quem sai leva consigo o último alvo escrito
/// — o gesto de volta nunca mais os alcança, e fica um degrau na fronteira do
/// pincel.
///
/// ⚠️ **O gesto tem de ser GRANDE, e a primeira versão deste gate não era.** Com
/// `s = 0,9` nenhum vértice escapa: o falloff vai a zero na borda, então o
/// máximo de `t·(1 + s·(1 − t²)²)` fica em ~0,93 do raio e o conjunto não muda
/// — a mutação *"não congele a pegada"* passava, sobre um gate escrito
/// exatamente para ela. É preciso empurrar até o barro cruzar o próprio raio.
#[test]
fn the_local_scale_carries_the_whole_footprint_it_took_at_pen_down() {
    let brush = turn(Verb::LocalScale, 0.5);
    let mut mesh = sphere();
    let before = snapshot(&mesh);
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    // Vai fundo e volta ao zero. O caminho de ida é o que empurra vértices para
    // fora do raio consultado; o de volta é onde a falta deles aparece.
    for a in [1.0f32, 3.0, 6.0, 3.0, 1.0, 0.0] {
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::scaling(PIVOT, brush.radius, EYE, a),
            Symmetry::default(),
        );
    }
    let residue = max_shift(&before, &mesh);
    assert!(
        residue < 1e-5,
        "sobrou {residue:.6} depois de voltar ao zero — barro que saiu da pegada \
         consultada e não foi mais alcançado"
    );
}

// ---------------------------------------------------------------------------
// O ESPELHO
// ---------------------------------------------------------------------------

/// O pior desvio entre a malha e a imagem espelhada dela em `X` — **o oráculo do
/// espelho**, e ele não conhece campo nenhum do dab.
///
/// ⚠️ **Ele mede o RESULTADO, e é isso que o torna oráculo em vez de espelho do
/// código.** *Simetria* quer dizer *a forma é igual à imagem dela no espelho*;
/// um gate que comparasse os campos do `Dab` estaria afirmando a implementação
/// que eu acabei de escrever.
fn mirror_error(m: &ph2d_mesh::Mesh) -> f32 {
    let mut worst = 0.0f32;
    for p in m.positions() {
        let q = [-p[0], p[1], p[2]];
        let mut best = f32::MAX;
        for r in m.positions() {
            let d = (r[0] - q[0]).powi(2) + (r[1] - q[1]).powi(2) + (r[2] - q[2]).powi(2);
            best = best.min(d);
        }
        worst = worst.max(best.sqrt());
    }
    worst
}

/// **O espelho alcança o DAB INTEIRO** — e cada canal dele por uma razão
/// geométrica diferente.
///
/// ⚠️ **Os três casos foram MEDIDOS antes do conserto**, e é por isso que a
/// fixture é esta e não outra:
///
/// | canal | erro de simetria ANTES |
/// |---|---|
/// | `pull` (Grab) | **0,343574** |
/// | `eye` (Draw na terminador) | **0,059441** |
/// | `eye` (Flatten na terminador) | **0,047978** |
/// | `eye` (Clay na terminador) | **0,094383** |
/// | controle (esfera intocada) | 0,000001 |
///
/// ⚠️ **O `eye` só é observável onde a pegada atravessa a TERMINADOR** — o mesmo
/// Draw a meio caminho do polo media **0,000001**, porque ali o conjunto frontal
/// é *tudo* dos dois lados. A fixture tem de conter o fenômeno, e o lugar dele é
/// o círculo onde `n·eye = 0`.
#[test]
fn the_mirror_of_a_dab_is_the_mirror_of_its_result() {
    // O olho bem inclinado, e a pegada centrada na terminador dele.
    //
    // ⚠️ **O raio é 0,25 porque as duas cópias não podem SE SOBREPOR.** Com 0,6
    // e centros a 0,6 de distância elas se cruzam, e aí a segunda consulta uma
    // malha que a primeira já moveu — o resultado fica assimétrico por *ordem de
    // aplicação*, um fenômeno diferente do que este gate mede. Foi assim que a
    // primeira versão dele reprovou o conserto certo.
    let oblique = [0.94, 0.0, -0.34];
    let on_terminator = [-0.3, 0.47, -0.83];
    let cases: [(&str, Verb, Dab); 5] = [
        (
            "GRAB (o pull)",
            Verb::Move,
            Dab::pulling([-0.7, 0.3, 0.6], 0.25, [0.3, -0.4, -0.85], [0.2, 0.1, 0.0]),
        ),
        (
            "DRAW (o eye)",
            Verb::Draw,
            Dab::at(on_terminator, 0.25, oblique),
        ),
        (
            "FLATTEN (o eye)",
            Verb::Flatten,
            Dab::at(on_terminator, 0.25, oblique),
        ),
        (
            "CLAY (o eye)",
            Verb::Clay,
            Dab::at(on_terminator, 0.25, oblique),
        ),
        (
            "TWIST (o amount)",
            Verb::Twist,
            Dab::turning([-0.7, 0.3, 0.6], 0.25, [0.3, -0.4, -0.85], 0.9),
        ),
    ];
    // O controle: a esfera intocada mede o piso do oráculo.
    let floor = mirror_error(&sphere());
    assert!(floor < 1e-4, "a fixture não é simétrica ({floor:.6})");

    for (name, verb, dab) in cases {
        let mut mesh = sphere();
        let brush = Brush {
            verb,
            radius: dab.radius,
            strength: 1.0,
            ..Brush::default()
        };
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        let moved = stroke.dab(&mut mesh, &brush, &dab, Symmetry::MIRROR_X);
        assert!(moved > 0, "{name}: o dab não moveu nada");
        let err = mirror_error(&mesh);
        assert!(
            err < 1e-3,
            "{name}: erro de espelho {err:.6} — o espelho não alcançou o gesto"
        );
    }
}

/// ⚠️ **Um redemoinho visto no espelho gira ao CONTRÁRIO**, e é isso que separa
/// um pseudoescalar de um escalar.
///
/// O gate acima já morre se o sinal não trocar — mas ele morre por *assimetria*,
/// que é um sintoma de várias coisas. Este diz o FATO: com o espelho em `X`, a
/// fase do barro na metade `−X` e a da metade `+X` têm sinais opostos.
///
/// ⚠️ **E a metade oposta é o [`Amount::Fraction`]:** uma escala não tem mão, e
/// negá-la faria a metade espelhada ENCOLHER enquanto a outra cresce — o gate do
/// espelho pegaria, este nomeia por quê.
#[test]
fn a_twist_seen_in_the_mirror_turns_the_other_way() {
    let mut mesh = sphere();
    let before = snapshot(&mesh);
    let brush = turn(Verb::Twist, 0.4);
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    // O eixo é `Z` puro, então a fase mora no plano `XY` e o espelho é em `X`.
    stroke.dab(
        &mut mesh,
        &brush,
        &Dab::turning([-0.6, 0.0, 0.8], 0.4, [0.0, 0.0, -1.0], 1.2),
        Symmetry::MIRROR_X,
    );
    // A fase média de cada metade, em torno do centro da SUA pegada.
    let phase = |sign: f32, before: &[[f32; 3]], m: &ph2d_mesh::Mesh| {
        let c = [-0.6 * sign, 0.0];
        let (mut acc, mut n) = (0.0f32, 0u32);
        for (a, b) in before.iter().zip(m.positions()) {
            let da = [a[0] - c[0], a[1] - c[1]];
            let r = (da[0] * da[0] + da[1] * da[1]).sqrt();
            if !(0.1..0.35).contains(&r) {
                continue;
            }
            let db = [b[0] - c[0], b[1] - c[1]];
            acc += (da[0] * db[1] - da[1] * db[0]).atan2(da[0] * db[0] + da[1] * db[1]);
            n += 1;
        }
        assert!(n > 20, "o anel de medida pegou só {n} vértices");
        acc / n as f32
    };
    let left = phase(1.0, &before, &mesh);
    let right = phase(-1.0, &before, &mesh);
    assert!(
        left.abs() > 0.1 && right.abs() > 0.1,
        "uma das metades mal girou (esquerda {left:.4}, direita {right:.4})"
    );
    assert!(
        left * right < 0.0,
        "as duas metades giraram para o MESMO lado (esquerda {left:.4}, direita {right:.4}) \
         — o ângulo é um pseudoescalar e o espelho tem de trocar o sinal dele"
    );
}

/// ⚠️ **Uma FRAÇÃO de escala não tem mão**, e o gate existe para ninguém
/// "completar" a lei do espelho negando-a junto com o ângulo: as duas metades
/// têm de CRESCER, e não uma crescer enquanto a outra encolhe.
#[test]
fn a_local_scale_seen_in_the_mirror_still_grows() {
    let mut mesh = sphere();
    let before = snapshot(&mesh);
    let brush = turn(Verb::LocalScale, 0.4);
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    stroke.dab(
        &mut mesh,
        &brush,
        &Dab::scaling([-0.6, 0.0, 0.8], 0.4, [0.0, 0.0, -1.0], 0.6),
        Symmetry::MIRROR_X,
    );
    for sign in [1.0f32, -1.0] {
        let c = [-0.6 * sign, 0.0, 0.8];
        let mut most = 0.0f32;
        for (a, b) in before.iter().zip(mesh.positions()) {
            let da = ((a[0] - c[0]).powi(2) + (a[1] - c[1]).powi(2)).sqrt();
            if !(0.1..0.35).contains(&da) {
                continue;
            }
            let db = ((b[0] - c[0]).powi(2) + (b[1] - c[1]).powi(2)).sqrt();
            most = most.max(db / da);
        }
        assert!(
            most > 1.1,
            "a metade em x={:.1} não cresceu (maior razão {most:.3})",
            c[0]
        );
    }
}
