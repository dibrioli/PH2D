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
