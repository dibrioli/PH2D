//! **O ACCUMULATE** — a lei que troca o envelope por uma integral de linha.
//!
//! O irmão 3D do `BRUSH_ACCUMULATE` do Blender e do campo de mesmo nome do
//! pincel 2D. Desarmado, cruzar o próprio traço não intensifica nada; armado,
//! passar duas vezes soma duas vezes.
//!
//! ⚠️ **A pergunta que estes gates existem para responder não é *"acumula?"*, é
//! *"acumula em função do QUÊ?"***. Uma soma crua sobre a lista de dabs seria
//! função de quantos dabs o motor emitiu — a doença que a `line/Painter` pagou
//! quatro vezes e que o cabeçalho do `stroke.rs` enuncia como a lei da casa. A
//! normalização pelo passo (`ACCUM_PER_DAB`) é o que a torna função do CAMINHO,
//! e é isso que o gate da proporcionalidade mede.

use super::*;
use ph2d_mesh::{Mesh, shapes};

fn sphere() -> Mesh {
    shapes::uv_sphere(32, 48, 1.0)
}

/// O raio de mundo de um pincel que cobre uma calota confortável da esfera.
const R: f32 = 0.35;

/// **Uma passada reta pelo topo da esfera**, carimbada no passo geométrico que o
/// produto usa.
///
/// ⚠️ Ela anda em `x` sobre o polo `+Z` e mantém `z` na superfície: é a fatia da
/// esfera onde o olho (`-Z`) vê tudo de frente, então o culling não entra na
/// conta e o gate mede a LEI, não a visibilidade.
fn sweep(mesh: &mut Mesh, stroke: &mut SculptStroke, brush: &Brush, passes: usize) {
    let step = crate::min_spacing(R);
    let (from, to) = (-0.5_f32, 0.5_f32);
    let n = ((to - from) / step).floor() as usize;
    for _ in 0..passes {
        for k in 0..=n {
            let x = step.mul_add(k as f32, from);
            let z = (1.0 - x * x).max(0.0).sqrt();
            stroke.dab(
                mesh,
                brush,
                &Dab::at([x, 0.0, z], R, [0.0, 0.0, -1.0]),
                Symmetry::default(),
            );
        }
    }
}

/// Quanto o vértice mais alto da calota subiu.
fn lift(before: &Mesh, after: &Mesh) -> f32 {
    let mut best = 0.0_f32;
    for (i, b) in before.positions().iter().enumerate() {
        // Só o miolo do caminho: as pontas veem meia passada por construção.
        if b[0].abs() > 0.2 || b[1].abs() > 0.1 || b[2] < 0.5 {
            continue;
        }
        let a = after.positions()[i];
        let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
        best = best.max((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
    }
    best
}

fn drawing(accumulate: bool) -> Brush {
    Brush {
        verb: Verb::Draw,
        strength: 0.5,
        radius: R,
        accumulate,
        ..Brush::default()
    }
}

/// Uma varredura de `passes` passadas, e o quanto ela levantou o miolo.
fn measure(accumulate: bool, passes: usize) -> f32 {
    let base = sphere();
    let mut mesh = sphere();
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    sweep(&mut mesh, &mut stroke, &drawing(accumulate), passes);
    lift(&base, &mesh)
}

/// **O ENVELOPE SATURA e o ACCUMULATE EMPILHA** — o gate da feature.
///
/// ⚠️ As duas metades são um gate só de propósito: a de cima sozinha ficaria
/// verde com o accumulate ligado por engano no default (o envelope também
/// "levanta"), e a de baixo sozinha ficaria verde com um accumulate que
/// simplesmente multiplicasse a força.
#[test]
fn the_envelope_saturates_and_the_accumulate_piles() {
    let (env1, env2) = (measure(false, 1), measure(false, 2));
    assert!(
        (env2 - env1).abs() < 1e-4,
        "o ENVELOPE intensificou na segunda passada: {env1} -> {env2}"
    );

    let (acc1, acc2) = (measure(true, 1), measure(true, 2));
    assert!(
        acc2 > acc1 * 1.8,
        "o ACCUMULATE não empilhou: {acc1} -> {acc2} (esperado ~2x)"
    );
}

/// **A pilha é proporcional ao CAMINHO percorrido** — o gate que sustenta a
/// normalização.
///
/// ⚠️ Sem ele a feature seria *"a força depende de quantos dabs o motor
/// emitiu"*, que é exatamente o que a lei do traço existe para não ter. Duas
/// passadas somam duas vezes; quatro somam quatro. A tolerância é folgada de
/// propósito — o gate afirma a PROPORCIONALIDADE, não um número.
#[test]
fn the_pile_is_proportional_to_the_path_walked() {
    let one = measure(true, 1);
    assert!(one > 1e-4, "a fixture não levantou nada");
    for passes in [2usize, 4] {
        let got = measure(true, passes);
        let want = one * passes as f32;
        assert!(
            (got / want - 1.0).abs() < 0.10,
            "{passes} passadas deram {got}, e uma passada vale {one} (esperado ~{want})"
        );
    }
}

/// **A primeira passada acumulada é MAIS FRACA que a do envelope** — o preço, e
/// ele é medido em vez de estimado.
///
/// O envelope entrega o PICO do falloff (o dab que passou pelo centro do
/// vértice); a integral entrega a MÉDIA dele sobre a corda. É a consequência
/// honesta de a lei ser uma soma, e ela está nomeada no doc do
/// [`crate::ACCUM_PER_DAB`] — este gate é o que impede aquela frase de apodrecer.
#[test]
fn the_first_accumulated_pass_is_weaker_than_the_envelope() {
    let (env, acc) = (measure(false, 1), measure(true, 1));
    assert!(
        acc < env,
        "a primeira passada acumulada ({acc}) não é mais fraca que a do envelope ({env})"
    );
    // E não é uma ordem de grandeza: se fosse, a calibração estaria errada e o
    // artista teria de dobrar a força ao armar o interruptor.
    assert!(
        acc > env * 0.25,
        "a primeira passada acumulada é fraca DEMAIS: {acc} contra {env}"
    );
}

/// **O interruptor é INERTE nos verbos que não carimbam** — byte a byte.
///
/// ⚠️ Quem tem âncora carrega o gesto TOTAL desde o pen-down; somar totais
/// multiplicaria o gesto pelo número de eventos de ponteiro. A porta que decide
/// é [`Verb::accumulates`], e este gate é o que prova que ela é honrada em vez
/// de meramente oferecida.
#[test]
fn the_switch_is_inert_for_every_verb_that_does_not_stamp() {
    for verb in Verb::ALL {
        // ⚠️ **A seleção é pelo GRIP, e não por `accumulates()`.** A primeira
        // versão filtrava pela própria função sob teste, e a mutação que a faz
        // devolver `true` para todo verbo esvaziava o laço — o gate passava
        // sobre NADA, verde por vácuo. O grip é o fato independente: se alguém
        // fizer o Grab acumular, este laço ainda o escolhe e a asserção sangra.
        if matches!(verb.grip(), Grip::Stamp) {
            continue;
        }
        let mut snap = [const { Vec::new() }; 2];
        for (k, accumulate) in [false, true].into_iter().enumerate() {
            let mut mesh = sphere();
            let mut stroke = SculptStroke::default();
            stroke.begin(&mesh);
            let brush = Brush {
                verb,
                accumulate,
                ..drawing(false)
            };
            // Um gesto com âncora: pega e puxa, duas vezes pelo mesmo caminho.
            for pull in [0.05_f32, 0.10] {
                stroke.dab(
                    &mut mesh,
                    &brush,
                    &Dab::pulling([0.0, 0.0, 1.0], R, [0.0, 0.0, -1.0], [pull, 0.0, 0.0]),
                    Symmetry::default(),
                );
            }
            snap[k] = mesh.positions().to_vec();
        }
        assert_eq!(
            snap[0], snap[1],
            "o accumulate mexeu no verbo {verb:?}, que não carimba"
        );
    }
}

/// **Desarmado, o traço toma a estrada do ENVELOPE** — e o oráculo é o TRABALHO.
///
/// ⚠️ **A primeira versão deste gate era tautológica:** ela rodava a mesma
/// varredura duas vezes com o MESMO pincel e comparava os resultados — dois
/// caminhos idênticos concordando, verde por construção. O que ela queria dizer
/// era *"o desarmado é a lei que já shipava"*, e a assinatura dessa lei não é
/// uma igualdade consigo mesma: é a **IDEMPOTÊNCIA**. Re-carimbar a mesma lista
/// de dabs não escreve um vértice sob o envelope (o early-out descarta quem não
/// supera), e escreve TODOS sob o accumulate — que é precisamente a troca.
#[test]
fn the_switch_off_takes_the_envelope_road() {
    assert!(
        !Brush::default().accumulate,
        "o default tem de ser DESARMADO"
    );
    for (accumulate, idempotent) in [(false, true), (true, false)] {
        let mut mesh = sphere();
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        let brush = drawing(accumulate);
        let dab = Dab::at([0.0, 0.0, 1.0], R, [0.0, 0.0, -1.0]);
        stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
        assert!(!stroke.last_moved().is_empty(), "o 1º dab não moveu nada");
        stroke.dab(&mut mesh, &brush, &dab, Symmetry::default());
        assert_eq!(
            stroke.last_moved().is_empty(),
            idempotent,
            "com accumulate={accumulate} o dab repetido {} vértices",
            stroke.last_moved().len()
        );
    }
}
