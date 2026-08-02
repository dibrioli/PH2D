//! Gates do **Grab** — o verbo que puxa, e a exceção declarada ao envelope.
//!
//! Filho do MESMO pai que os outros gates de verbo, então `use super::*` alcança
//! o `sphere` e o `dab_for` compartilhados — reimplementar a esfera aqui daria
//! uma segunda fixture para divergir da que os outros doze verbos medem.

use super::*;

fn grab() -> Brush {
    Brush {
        verb: Verb::Move,
        radius: 0.5,
        strength: 1.0,
        ..Brush::default()
    }
}

/// O vértice mais próximo de um ponto — o "miolo" da pegada.
fn nearest(mesh: &ph2d_mesh::Mesh, p: [f32; 3]) -> usize {
    (0..mesh.vert_count())
        .min_by(|&a, &b| {
            let d = |i: usize| {
                let q = mesh.positions()[i];
                (q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2) + (q[2] - p[2]).powi(2)
            };
            d(a).total_cmp(&d(b))
        })
        .expect("a esfera tem vértices")
}

/// ⚠️ **O DEFEITO que a exceção fecha, e ele é invisível num dab só.** A pegada
/// do Grab é presa no pen-down, então o peso de cada vértice é o do PRIMEIRO dab
/// e nunca mais sobe — o early-out do envelope (`w <= accum ⇒ pula`) congelaria
/// o alvo, e o barro andaria um evento e pararia com o cursor seguindo em
/// frente. Este gate arrasta em três passos, como a mão faz.
#[test]
fn the_clay_keeps_coming_while_the_finger_keeps_pulling() {
    let mut mesh = sphere();
    let at = [0.0, 0.0, 1.0];
    let core = nearest(&mesh, at);
    let start = mesh.positions()[core];

    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let mut seen = Vec::new();
    for step in 1..=3 {
        let pull = [0.0, 0.1 * step as f32, 0.0];
        stroke.dab(
            &mut mesh,
            &grab(),
            // ⚠️ O CENTRO não anda: o Grab prende a pegada. Quem move a pegada é
            // o Snake Hook, que é outro verbo.
            &Dab::pulling(at, 0.5, [0.0, 0.0, -1.0], pull),
            Symmetry::default(),
        );
        seen.push(mesh.positions()[core][1] - start[1]);
    }
    assert!(
        seen[1] > seen[0] + 0.05 && seen[2] > seen[1] + 0.05,
        "o barro tem de seguir o dedo a cada evento, e andou {seen:?}"
    );
    assert!(
        (seen[2] - 0.3).abs() < 0.02,
        "no fim ele está onde o dedo está ({}), e não onde o falloff o deixou",
        seen[2]
    );
}

/// **A lei do traço continua de pé, e é isto que separa o Grab de um `+=`.**
/// O alvo é função do `pre` congelado, então puxar de volta devolve o barro ao
/// lugar — e um `+=` por evento deixaria um rastro que nada desfaz.
#[test]
fn pulling_back_puts_the_clay_where_it_was() {
    let mut mesh = sphere();
    let at = [0.0, 0.0, 1.0];
    let core = nearest(&mesh, at);
    let start = mesh.positions()[core];

    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    for pull in [[0.0, 0.3, 0.0], [0.0, 0.5, 0.0], [0.0, 0.0, 0.0]] {
        stroke.dab(
            &mut mesh,
            &grab(),
            &Dab::pulling(at, 0.5, [0.0, 0.0, -1.0], pull),
            Symmetry::default(),
        );
    }
    let back = mesh.positions()[core];
    for k in 0..3 {
        assert!(
            (back[k] - start[k]).abs() < 1e-5,
            "voltar ao ponto de partida tem de devolver o barro: {start:?} -> {back:?}"
        );
    }
}

/// **Dois TRAÇOS compõem** — cada um congela um `pre` novo, então puxar duas
/// vezes puxa mais longe. É o que o `04.1` promete para esta família, e ele cai
/// da lei em vez de precisar de composição de mapas: aquela é o que o Snake Hook
/// vai pedir, porque nele a PEGADA anda.
#[test]
fn two_strokes_compose_because_each_one_freezes_a_new_pre() {
    let mut mesh = sphere();
    let at = [0.0, 0.0, 1.0];
    let core = nearest(&mesh, at);
    let start = mesh.positions()[core][1];

    let mut after = Vec::new();
    for _ in 0..2 {
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        stroke.dab(
            &mut mesh,
            &grab(),
            &Dab::pulling(at, 0.5, [0.0, 0.0, -1.0], [0.0, 0.2, 0.0]),
            Symmetry::default(),
        );
        after.push(mesh.positions()[core][1] - start);
    }
    // ⚠️ **E NÃO é soma: 0,2 vira 0,2996, não 0,4.** O oráculo ingênuo (somar os
    // dois puxões) falhou aqui, e o produto está certo — o vértice que o
    // primeiro traço levantou ficou mais LONGE do centro do dab, então o falloff
    // do segundo o pega mais fraco. É o que qualquer app de escultura faz: a
    // pegada é sobre a superfície onde ela ESTÁ, não onde ela estava. O que a
    // composição promete é *puxar mais*, não *puxar o dobro*.
    let gained = after[1] - after[0];
    assert!(
        gained > 0.4 * after[0],
        "o segundo traço tem de puxar de novo, e ganhou {gained} sobre {:?}",
        after[0]
    );
}

/// A borda fica onde está: o Grab é um pincel, não uma translação do objeto.
#[test]
fn the_rim_of_the_footprint_stays_put() {
    let mut mesh = sphere();
    let at = [0.0, 0.0, 1.0];
    let far = nearest(&mesh, [0.0, 0.0, -1.0]);
    let before = mesh.positions()[far];

    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    stroke.dab(
        &mut mesh,
        &grab(),
        &Dab::pulling(at, 0.5, [0.0, 0.0, -1.0], [0.0, 0.4, 0.0]),
        Symmetry::default(),
    );
    assert_eq!(
        mesh.positions()[far],
        before,
        "o outro lado da esfera não pode andar"
    );
}

/// Um dab sem gesto é um no-op — e é isso que torna seguro o `pull` nascer zero
/// no [`Dab::at`] que os outros doze verbos usam.
#[test]
fn a_dab_with_no_gesture_moves_nothing() {
    let mut mesh = sphere();
    let before: Vec<[f32; 3]> = mesh.positions().to_vec();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    stroke.dab(
        &mut mesh,
        &grab(),
        &Dab::at([0.0, 0.0, 1.0], 0.5, [0.0, 0.0, -1.0]),
        Symmetry::default(),
    );
    assert_eq!(mesh.positions(), &before[..]);
}

/// ⚠️ **O PERFIL, e é a metade que os gates do miolo não podiam ver.**
///
/// O aplicador multiplica `(alvo − base)` pelo `accum`, então um alvo que já
/// traz o peso aplica o falloff **duas vezes**. Até 2026-08-01 o alvo do Grab
/// era `add_vec(base, pull, shape)` e o pincel saía pontudo: medido a meio
/// raio, a referência (`Move.js:120`, uma aplicação) move `0,22500` e nós
/// movíamos `0,12226` — `pull·fall²` ao milésimo.
///
/// Nenhum gate do miolo pode pegar isto, porque em `fall == 1` os dois são o
/// mesmo número. O oráculo tem de ser a **curva**, e ela é comparada com o
/// falloff que o pincel declara — não com uma tabela de constantes, que
/// envelheceria no dia em que o default de falloff mudasse.
#[test]
fn the_grab_applies_its_falloff_once_not_twice() {
    let mut mesh = sphere();
    let before: Vec<[f32; 3]> = mesh.positions().to_vec();
    let (at, radius, pull) = ([0.0, 0.0, 1.0], 0.5f32, 0.4f32);

    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    stroke.dab(
        &mut mesh,
        &grab(),
        &Dab::pulling(at, radius, [0.0, 0.0, -1.0], [0.0, pull, 0.0]),
        Symmetry::default(),
    );

    let curve = crate::Falloff::default();
    let (mut worst_single, mut worst_double, mut checked) = (0.0f32, 0.0f32, 0);
    for (i, b) in before.iter().enumerate() {
        let d = ((b[0] - at[0]).powi(2) + (b[1] - at[1]).powi(2) + (b[2] - at[2]).powi(2)).sqrt()
            / radius;
        if !(0.1..=0.9).contains(&d) {
            continue;
        }
        let moved = mesh.positions()[i][1] - b[1];
        let f = curve.weight(d);
        worst_single = worst_single.max((moved - pull * f).abs());
        worst_double = worst_double.max((moved - pull * f * f).abs());
        checked += 1;
    }
    // A esfera desta suíte é grosseira de propósito (os gates são muitos); a
    // banda `0,1..0,9` do raio dá **50** vértices nela, medido.
    assert!(checked > 30, "a fixture tem de ter ombro: {checked}");
    // A malha é discreta, então o vértice não senta exatamente na banda — o que
    // decide é QUAL das duas leis está mais perto, e por muito.
    assert!(
        worst_single * 3.0 < worst_double,
        "o perfil tem de seguir `pull·fall` e não `pull·fall²`: erro {worst_single} \
         contra {worst_double}"
    );
}

/// ⚠️ **A pegada do Grab é CONGELADA, e sem isto ele PERDE barro.**
///
/// A consulta sai das posições vivas, então um vértice puxado para além do raio
/// SAI da esfera e deixa de ser escrito — ele congela onde estava, e o gesto de
/// voltar não o alcança mais. Medido antes da cura: raio 0,4 puxado a 0,6 e
/// trazido de volta deixava **0,52994** de resíduo, sobre um verbo cuja
/// propriedade declarada é *"puxar de volta devolve o barro ao lugar"*.
///
/// ⚠️ **O gate irmão (`pulling_back_puts_the_clay_where_it_was`) não podia ver**:
/// ele mede o vértice do MIOLO com puxão menor que o raio, e nesse regime nada
/// escapa. O que separa os dois é o puxão passar da pegada.
#[test]
fn a_grab_pulled_past_its_own_radius_still_gives_every_vertex_back() {
    let mut mesh = sphere();
    let before: Vec<[f32; 3]> = mesh.positions().to_vec();
    let (at, radius) = ([0.0, 0.0, 1.0], 0.4f32);
    let brush = Brush { radius, ..grab() };

    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let ups = (0..=12).map(|k| k as f32 * 0.05);
    let downs = (0..12).rev().map(|k| k as f32 * 0.05);
    let mut peak = 0.0f32;
    for y in ups.chain(downs) {
        stroke.dab(
            &mut mesh,
            &brush,
            &Dab::pulling(at, radius, [0.0, 0.0, -1.0], [0.0, y, 0.0]),
            Symmetry::default(),
        );
        peak = peak.max(
            before
                .iter()
                .zip(mesh.positions())
                .map(|(a, b)| (b[1] - a[1]).abs())
                .fold(0.0f32, f32::max),
        );
    }
    assert!(
        peak > radius,
        "a fixture tem de PASSAR do raio, senão nada escapa: pico {peak} contra {radius}"
    );
    let left = before
        .iter()
        .zip(mesh.positions())
        .map(|(a, b)| {
            ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt()
        })
        .fold(0.0f32, f32::max);
    assert!(
        left < 1e-5,
        "todo vértice tem de voltar, e sobrou {left} (a pegada descongelou)"
    );
}

/// ⚠️ **As duas cópias do espelho COMPARTILHAM o `touched`**, e é o filtro de
/// peso zero que as separa.
///
/// Quem SEGURA trabalha sobre o conjunto tocado (a pegada congelada), e esse
/// vetor é um só para o traço inteiro — as duas cópias da simetria escrevem
/// nele. Sem `w <= 0 ⇒ pula`, a segunda cópia visitaria os vértices da primeira,
/// mediria peso zero contra o centro DELA, e — porque quem puxa dispensa o
/// early-out do envelope — gravaria `accum ← 0` e `target ← base`, **desfazendo
/// o que a primeira acabou de fazer**. O artista veria um lado só se mover, com
/// o espelho ligado.
#[test]
fn a_mirrored_grab_pulls_both_copies() {
    let mut mesh = sphere();
    let before: Vec<[f32; 3]> = mesh.positions().to_vec();
    let (at, radius) = ([0.6, 0.0, 0.8], 0.4f32);

    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    stroke.dab(
        &mut mesh,
        &Brush { radius, ..grab() },
        &Dab::pulling(at, radius, [0.0, 0.0, -1.0], [0.0, 0.3, 0.0]),
        Symmetry::MIRROR_X,
    );

    // Quanto o lado positivo e o negativo de X andaram.
    let mut side = [0.0f32, 0.0];
    for (i, b) in before.iter().enumerate() {
        let moved = (mesh.positions()[i][1] - b[1]).abs();
        let k = usize::from(b[0] < 0.0);
        side[k] = side[k].max(moved);
    }
    assert!(
        side[0] > 0.2 && side[1] > 0.2,
        "as duas cópias têm de puxar, e mediram {side:?}"
    );
}
