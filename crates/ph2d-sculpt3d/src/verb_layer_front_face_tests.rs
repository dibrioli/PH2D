//! **O FRONT-FACE É UM FLAG DO PINCEL** — a lei que a referência sempre teve e
//! que nós aplicávamos incondicionalmente, e o que ela fazia com a DEMÃO.
//!
//! ⚠️ **Estes gates moram num irmão porque a FIXTURE é outra.** Toda a suíte do
//! [`crate::Verb::Layer`] esculpe numa GRADE PLANA, onde a normal é `+z` em
//! toda parte e o olho é `−z`: ali `max(−n·olho, 0)` vale **`1,0000` exato em
//! todo vértice**, e os dois mundos — flag ligado e desligado — são
//! byte-idênticos. *Nenhum gate escrito sobre a grade pode ver esta lei*, que é
//! por que ela atravessou a wave inteira. Aqui a malha é uma ESFERA e o dab é
//! grande de propósito: com raio `0,9` numa esfera de raio `1` o facing varre
//! `0,5800 … 1,0000` (medido, `measure_layer_front_face` P2), e um dab pequeno
//! no polo mal sai de `1,0` — ele também não veria nada.
//!
//! # A referência
//!
//! `layer.cc:149` — e `clay_strips.cc`, `sculpt_cloth.cc`, `paint_color.cc`,
//! `draw_face_sets.cc`, todos iguais:
//!
//! ```text
//! if (brush.flag & BRUSH_FRONTFACE) {
//!   calc_front_face(cache.view_normal_symm, vert_normals, verts, factors);
//! }
//! ```
//!
//! O bit é o checkbox *"Front Faces Only"* (`use_frontface`,
//! `properties_paint_common.py:1354`), e **nenhuma linha do Blender inteiro o
//! LIGA** — varrido: o único hit fora de leitura é
//! `use_front_face_ = brush_->flag & BRUSH_FRONTFACE`, que também lê.

use crate::{Brush, Dab, RefMode, SculptStroke, Symmetry, Verb};
use ph2d_mesh::{Mesh, shapes};

/// A esfera de fábrica, e o dab que atravessa o terminador dela.
fn sphere() -> Mesh {
    shapes::sculpt_sphere(1.0)
}

const CENTRE: [f32; 3] = [0.0, 0.0, 1.0];
const EYE: [f32; 3] = [0.0, 0.0, -1.0];
const R: f32 = 0.9;

/// Um pincel de demão DURO — é a dureza que torna a lei observável.
fn hard_coat(front_faces_only: bool) -> Brush {
    Brush {
        verb: Verb::Layer,
        mode: RefMode::B,
        radius: R,
        strength: Verb::Layer.default_strength(),
        falloff: Verb::Layer.default_falloff(RefMode::B),
        hardness: 0.9,
        front_faces_only,
        ..Brush::default()
    }
}

/// O deslocamento RADIAL médio num anel de raio `t·R` em torno do dab, medido
/// contra a malha de repouso.
///
/// ⚠️ **A régua é a distância 3-D ao centro do dab, a MESMA que o kernel usa.**
/// Um anel por raio XY casaria o hemisfério de TRÁS junto — normal `−z`, facing
/// zero — sobre vértices que o dab nem alcança, e a coluna sairia como a média
/// de `1` com `0`: uma constante que se lê como *"o facing não faz nada"*. Foi o
/// primeiro corte da sonda, e ele mediu exatamente isso.
fn ring(mesh: &Mesh, rest: &Mesh, t: f32) -> f32 {
    let len = |q: [f32; 3]| (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt();
    let (mut sum, mut n) = (0.0f32, 0usize);
    for (i, p) in mesh.positions().iter().enumerate() {
        let p0 = rest.positions()[i];
        let d = ((p0[0] - CENTRE[0]).powi(2)
            + (p0[1] - CENTRE[1]).powi(2)
            + (p0[2] - CENTRE[2]).powi(2))
        .sqrt()
            / R;
        if (d - t).abs() < 0.06 {
            sum += len(*p) - len(p0);
            n += 1;
        }
    }
    assert!(n > 0, "anel t={t} vazio: a fixture não contém o fenômeno");
    sum / n as f32
}

/// Um traço de `dabs` demãos no mesmo ponto, e os anéis que ele deixou.
fn coat_profile(front_faces_only: bool, dabs: usize) -> Vec<f32> {
    let rest = sphere();
    let mut mesh = sphere();
    let b = hard_coat(front_faces_only);
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for _ in 0..dabs {
        s.dab(&mut mesh, &b, &Dab::at(CENTRE, R, EYE), Symmetry::default());
    }
    [0.1f32, 0.3, 0.5, 0.7, 0.9]
        .iter()
        .map(|t| ring(&mesh, &rest, *t))
        .collect()
}

/// **A CURA, com o número do report.** Uma demão dura é uma MESA; com o
/// front-face ligado ela vira uma RAMPA, e a borda perde metade.
///
/// ⚠️ **O oráculo é a RAZÃO borda/centro, não um piso absoluto.** A demão
/// SATURA (`coat_step` converge para o teto qualquer que seja o peso), então a
/// altura final não distingue as duas leis num traço longo — o que distingue é
/// a FORMA, e ela é escala-invariante.
#[test]
fn a_hard_coat_is_a_table_and_the_front_face_makes_it_a_ramp() {
    let off = coat_profile(false, 1);
    let on = coat_profile(true, 1);

    let ratio = |v: &[f32]| v[4] / v[0];
    let (r_off, r_on) = (ratio(&off), ratio(&on));

    // Desligado — a referência de fábrica: o platô do hardness chega à borda.
    assert!(
        r_off > 0.70,
        "sem o front-face a demão dura tinha de ser uma MESA: borda/centro {r_off:.4} ({off:?})"
    );
    // Ligado — o cosseno da câmera come a borda.
    assert!(
        r_on < 0.50,
        "com o front-face ela tinha de virar RAMPA: borda/centro {r_on:.4} ({on:?})"
    );
    // E o CENTRO quase não se move: é a BORDA que a lei decide, e é isso que
    // torna o defeito difícil de ver sem medir o perfil inteiro.
    assert!(
        (on[0] - off[0]).abs() < off[0] * 0.05,
        "o centro tinha de ser quase o mesmo: {:.4} contra {:.4}",
        on[0],
        off[0]
    );
}

/// **O INTERRUPTOR APAGA A LEI** — a metade de cima do par cujo irmão
/// (`in_b_mode_a_grazing_vertex_weighs_nothing_and_the_ramp_has_no_step`) mede a
/// de baixo.
///
/// Com o flag desligado, o modo `B` tem de esculpir **byte a byte** como um modo
/// que não declara front-face nenhum.
#[test]
fn the_front_face_is_a_brush_flag_and_off_means_the_law_does_not_run() {
    let rest = sphere();
    let mut a = sphere();
    let mut b = sphere();

    // O mesmo pincel, com o interruptor desligado…
    let brush = hard_coat(false);
    let mut s = SculptStroke::default();
    s.begin(&a);
    s.dab(
        &mut a,
        &brush,
        &Dab::at(CENTRE, R, EYE),
        Symmetry::default(),
    );

    // …e o mesmo pincel sob um modo cuja lei é `FrontFace::Ignored`. Se o
    // interruptor de facto apaga a linha, os dois são o MESMO cálculo.
    //
    // ⚠️ **O modo é lido por `kernel_for`, que RECUA para `B` num verbo que o
    // modo não declara** — e o `S` não declara a demão. Por isso a ablação por
    // modo não serve aqui, e o oráculo é o par (flag ligado / desligado) do
    // gate acima; este mede que o desligado **não move nada além do que a curva
    // move**, comparando o dab com ele mesmo sob um `eye` que o vira do avesso:
    // se a lei corresse, um olho invertido zeraria a pegada inteira.
    let flipped = [0.0f32, 0.0, 1.0];
    let mut s2 = SculptStroke::default();
    s2.begin(&b);
    s2.dab(
        &mut b,
        &brush,
        &Dab::at(CENTRE, R, flipped),
        Symmetry::default(),
    );

    let mut worst = 0.0f32;
    for (i, p) in a.positions().iter().enumerate() {
        let q = b.positions()[i];
        worst = worst.max(
            (p[0] - q[0])
                .abs()
                .max((p[1] - q[1]).abs())
                .max((p[2] - q[2]).abs()),
        );
    }
    assert!(
        worst < 1e-6,
        "com o flag desligado o OLHO não pode importar: pior desvio {worst:.6}"
    );

    // O CONTROLE: com o flag LIGADO, o mesmo par de olhos dá resultados
    // diferentes — senão este gate estaria a medir uma malha que não se mexe.
    let mut c = sphere();
    let mut d = sphere();
    let armed = hard_coat(true);
    let mut s3 = SculptStroke::default();
    s3.begin(&c);
    s3.dab(
        &mut c,
        &armed,
        &Dab::at(CENTRE, R, EYE),
        Symmetry::default(),
    );
    let mut s4 = SculptStroke::default();
    s4.begin(&d);
    s4.dab(
        &mut d,
        &armed,
        &Dab::at(CENTRE, R, flipped),
        Symmetry::default(),
    );
    let mut moved_c = 0.0f32;
    let mut ctrl = 0.0f32;
    for (i, p) in c.positions().iter().enumerate() {
        let q = d.positions()[i];
        let r0 = rest.positions()[i];
        moved_c += (p[0] - r0[0]).abs() + (p[1] - r0[1]).abs() + (p[2] - r0[2]).abs();
        ctrl = ctrl.max(
            (p[0] - q[0])
                .abs()
                .max((p[1] - q[1]).abs())
                .max((p[2] - q[2]).abs()),
        );
    }
    assert!(moved_c > 1e-3, "a fixture não depositou nada: {moved_c:.6}");
    assert!(
        ctrl > 1e-4,
        "o CONTROLE não distingue: com o flag ligado o olho tinha de importar ({ctrl:.6})"
    );
}

/// **A DEMÃO NASCE COM O FLAG DESLIGADO** — o default da referência.
///
/// ⚠️ **Ele não menciona `false`**, e é isso que o torna um teste do default em
/// vez de uma cópia dele: a pergunta é feita à mesma porta que o produto
/// consulta.
#[test]
fn the_coat_is_born_without_the_front_face_like_the_reference() {
    assert!(
        !Verb::Layer.default_front_faces_only(),
        "a demão nasceu com o front-face ligado, e a referência não o liga"
    );
    // O `Brush::default()` DERIVA do verbo — um literal ali seria o mesmo fato
    // em dois lugares.
    assert_eq!(
        Brush::default().front_faces_only,
        Verb::Draw.default_front_faces_only(),
        "o pincel de fábrica não deriva a tabela do verbo"
    );
    // E a única célula ligada de hoje é a faixa, com o motivo no doc dela.
    // ⚠️ O CONTROLE existe para o gate distinguir *"a tabela é toda falsa"* de
    // *"a tabela tem exactamente uma exceção"*.
    assert!(
        Verb::ClayStrips.default_front_faces_only(),
        "a faixa perdeu o front-face, e o desenho dela foi smokado com ele"
    );
}

/// **O PLATÔ DE UMA DEMÃO DURA É CHATO** — a refutação do *pente*, pinada.
///
/// A foto do artista na faixa de dureza alta mostra **listras retangulares**, e
/// o handoff propôs duas leituras: o kernel ondula, ou a **parede** da mesa
/// escadeia pela grade de quads. A lei do `layer.cc` decide entre elas sem um
/// número escolhido — com `hardness = h` o `apply_hardness_to_distances` manda
/// toda distância abaixo de `h` para **zero**, a curva satura, e todo vértice
/// do disco interior tem o MESMO `shape` ⇒ a mesma altura absoluta.
///
/// ⚠️ **A régua é a ARESTA da malha, não um épsilon escolhido:** uma ondulação
/// menor que o espaçamento de vértices não tem como aparecer na tela, e um
/// limiar absoluto envelheceria na primeira mudança de subdivisão. Medido na
/// esfera de fábrica: **0,0093 de uma aresta** no pior caso.
///
/// ⚠️ **E ele mede a ALTURA junto, que é a outra metade do report** (*"o relevo
/// colapsa com dureza alta"*): esfregando, a demão fecha em **99,9% da meta**.
#[test]
fn a_hard_coat_lays_a_flat_plateau_at_the_authored_height() {
    let rest = sphere();
    let mut mesh = sphere();
    let b = Brush {
        hardness: 0.9,
        ..hard_coat(false)
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for _ in 0..12 {
        s.dab(&mut mesh, &b, &Dab::at(CENTRE, R, EYE), Symmetry::default());
    }

    let len = |q: [f32; 3]| (q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt();
    let d3 = |a: [f32; 3], c: [f32; 3]| {
        ((a[0] - c[0]).powi(2) + (a[1] - c[1]).powi(2) + (a[2] - c[2]).powi(2)).sqrt()
    };

    // A régua: o espaçamento mediano de aresta na região do dab.
    let (pos, adj) = (rest.positions(), rest.adjacency());
    let mut edges: Vec<f32> = Vec::new();
    for (i, p) in pos.iter().enumerate() {
        if d3(*p, CENTRE) <= R {
            for &j in adj.vert_verts.neighbours(i) {
                edges.push(d3(*p, pos[j as usize]));
            }
        }
    }
    edges.sort_by(|a, c| a.partial_cmp(c).unwrap());
    assert!(!edges.is_empty(), "a fixture não contém a pegada");
    let edge = edges[edges.len() / 2];

    // O disco onde a dureza saturou a curva.
    let inner = 0.9 * 0.95 * R;
    let mut hs: Vec<f32> = Vec::new();
    for (i, p) in mesh.positions().iter().enumerate() {
        let p0 = rest.positions()[i];
        if d3(p0, CENTRE) <= inner {
            hs.push(len(*p) - len(p0));
        }
    }
    assert!(
        hs.len() > 100,
        "platô com {} vértices — fixture fraca",
        hs.len()
    );

    let (lo, hi) = hs
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, c), &v| (a.min(v), c.max(v)));
    let ripple = (hi - lo) / edge;
    assert!(
        ripple < 0.1,
        "o platô ONDULA {ripple:.4} de uma aresta — o pente seria do KERNEL"
    );

    // E a altura é a autorada: o report dizia que ela colapsa.
    let mean = hs.iter().sum::<f32>() / hs.len() as f32;
    let target = b.layer_height;
    assert!(
        (mean - target).abs() < target * 0.02,
        "a demão dura fechou em {mean:.5} contra a meta {target:.5}"
    );
}

/// **A PORTA DO INTERRUPTOR responde pela LEI, nunca pelo flag** — o irmão do
/// `Verb::accumulates`.
///
/// ⚠️ **A metade que importa é a segunda:** uma caixa que se escondesse quando
/// desmarcada seria uma caixa que ninguém consegue marcar, e o default É
/// desmarcado.
#[test]
fn the_front_face_switch_is_offered_where_the_law_exists_in_either_position() {
    for on in [false, true] {
        let b = hard_coat(on);
        assert!(
            b.offers_front_faces(),
            "o modo B declara a lei e a porta recusou (flag={on})"
        );
    }
    // E onde a lei não existe, o interruptor não é oferecido.
    let ignored = Brush {
        verb: Verb::Draw,
        mode: RefMode::S,
        ..hard_coat(true)
    };
    assert!(
        !matches!(
            ignored.mode.kernel_for(ignored.verb).front_face,
            crate::FrontFace::Continuous
        ),
        "o CONTROLE perdeu a premissa: o S passou a declarar a lei"
    );
    assert!(
        !ignored.offers_front_faces(),
        "a porta ofereceu o interruptor num modo cuja lei é Ignored"
    );
}

/// **TROCAR DE VERBO TRAZ O DEFAULT DO VERBO, e só se o artista não mexeu.**
///
/// ⚠️ **A porta é do MOTOR e os dois chamadores a partilham** (o painel e o
/// atalho de teclado da shell) — antes dela a lei tinha duas cópias, e elas já
/// divergiam em dois campos.
#[test]
fn arming_a_verb_brings_its_front_face_default_but_never_erases_a_choice() {
    // Intocado: a faixa traz o `true` dela, e voltar traz o `false` da demão.
    let mut b = hard_coat(Verb::Layer.default_front_faces_only());
    b.verb = Verb::Layer;
    b.arm_verb_defaults(Verb::ClayStrips);
    assert!(
        b.front_faces_only,
        "a faixa não trouxe o default dela ao entrar"
    );
    b.verb = Verb::ClayStrips;
    b.arm_verb_defaults(Verb::Layer);
    assert!(
        !b.front_faces_only,
        "a demão não trouxe o default dela ao entrar"
    );

    // Tocado: o artista marcou a caixa na demão, e a troca a PRESERVA.
    let mut b = hard_coat(true);
    b.verb = Verb::Layer;
    b.arm_verb_defaults(Verb::Draw);
    assert!(
        b.front_faces_only,
        "a troca de verbo APAGOU uma escolha deliberada"
    );
}
