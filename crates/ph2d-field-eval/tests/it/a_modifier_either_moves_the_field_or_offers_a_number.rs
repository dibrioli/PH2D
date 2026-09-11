//! ⭐⭐⭐ **A LEI QUE O ESPELHO VIOLAVA** — report do Enio, 2026-09-04: *«Mirror não funcionou»*.
//!
//! # O defeito, medido
//!
//! O plano do espelho era o **`x = 0` do próprio nó**, e uma primitiva é construída **em volta da
//! origem local dela por construção** — logo o plano passa pelo centro da forma, sempre, e a dobra
//! `x → |x|` devolve a mesma peça. Medido pela sonda que abriu esta wave, sobre uma caixa:
//!
//! | caso | maior diferença de campo |
//! |---|---:|
//! | folha na origem | **`0.000000`** |
//! | folha **movida** `0,5` em `x` | **`0.000000`** |
//! | espelho numa **operação** com filho descentrado | `1.000000` |
//!
//! ⚠️ **Mover a peça não ajudava**, e é isso que fechava a porta: a pose do nó é aplicada **depois**
//! da pilha, então o plano viaja com o objeto. Não havia gesto nenhum, em todo o produto, pelo qual
//! um espelho posto numa forma mudasse um pixel — o chip acendia e a peça ficava igual.
//!
//! # A lei, e por que ela não é «todo modificador tem de mudar a peça»
//!
//! **Dois** modificadores nascem no ponto neutro de propósito e a razão está escrita no
//! [`ph2d_field::Unary::born`]: o afastamento (*«um afastamento de zero é literalmente nada a
//! acontecer, e é o sítio certo para começar a arrastar»*) e a inclinação. Eles não são controles
//! mortos — eles **oferecem uma linha de número** no painel, que é onde o artista descobre o gesto.
//!
//! ⇒ a lei é a disjunção: **ou o modificador muda o campo ao nascer, ou ele oferece um número.**
//! O espelho não fazia nem uma coisa nem outra, e era o único.
//!
//! ⚠️ **A lista é [`ph2d_field::UnaryKind::ALL`]**, e não uma escrita à mão: um modificador novo que
//! nasça mudo entra aqui sozinho.

use ph2d_field::{FieldDoc, NodeId, Primitive, Unary, UnaryKind, Xform};
use ph2d_field_eval::{Field, leaf};

/// A caixa de prova. ⚠️ **Não é um cubo**: uma forma com as três meias-extensões iguais esconderia
/// um eixo trocado, que é o defeito que um espelho de eixo errado É.
fn caixa() -> Primitive {
    Primitive::Box {
        half: [0.20, 0.14, 0.09],
        round: 0.0,
        chamfer: 0.0,
    }
}

/// A escala característica desta caixa — a mesma que o [`ph2d_field_ecs::add_mod`] passa ao
/// nascimento (a menor meia-extensão).
const ESCALA: f32 = 0.09;

/// As meias-extensões da peça, **por eixo** — é delas que o plano de um espelho nasce fora dela.
/// ⚠️ **Por eixo, e não um raio**: o espelho nomeia o eixo dele, e um raio único poria o gémeo
/// longe do corpo no eixo mais fino.
const ALCANCE: [f32; 3] = [0.20, 0.14, 0.09];

fn campo(mods: &[Unary]) -> Vec<f64> {
    let mut n = leaf(caixa(), Xform::default());
    n.mods.extend_from_slice(mods);
    let doc = FieldDoc::new(vec![n], NodeId(0)).expect("a peça");
    let f = Field::new(&doc);
    let mut v = Vec::new();
    for i in 0..9 {
        for j in 0..9 {
            for k in 0..9 {
                let p = |n: i32| f64::from(n) * 0.125 - 0.5;
                v.push(f.at(p(i), p(j), p(k)));
            }
        }
    }
    v
}

fn maior_diferenca(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0, f64::max)
}

/// ⛔ **Um modificador que nasce sem mexer no campo E sem número é um chip que acende e não faz
/// nada** — e o artista não tem como descobrir que ele existe.
#[test]
fn every_modifier_either_moves_the_field_or_offers_a_number() {
    let base = campo(&[]);
    let mut mudos = Vec::new();
    for kind in UnaryKind::ALL {
        let m = Unary::born(kind, ESCALA, ALCANCE);
        let mexe = maior_diferenca(&base, &campo(&[m])) > 1.0e-6;
        let numeros = !m.dims().is_empty();
        if !mexe && !numeros {
            mudos.push(format!("{kind:?}"));
        }
    }
    assert!(
        mudos.is_empty(),
        "⛔ {mudos:?} nasce(m) sem mudar o campo e sem oferecer um número — o chip acende e a peça \
         fica igual, que é o report de 2026-09-04. Ou o valor de nascimento muda a peça, ou o \
         modificador tem uma linha para o artista arrastar."
    );
}

/// ⭐⭐ **E o espelho tem de mudar a peça numa FOLHA** — o caso que o artista de facto alcança.
///
/// ⚠️ O gate irmão desta crate (`a_mirror_on_an_operation_folds_an_off_centre_child`) prova o
/// espelho numa **operação**, e passava verde durante todo o tempo em que a forma escolhida — que é
/// o primeiro objeto que existe numa cena — não podia ser espelhada. *Um gate que mede o caso raro
/// deixa o caso normal sem ruler.*
#[test]
fn a_mirror_born_on_a_shape_changes_the_shape() {
    let base = campo(&[]);
    for kind in [UnaryKind::Mirror, UnaryKind::MirrorY, UnaryKind::MirrorZ] {
        let m = Unary::born(kind, ESCALA, ALCANCE);
        let d = maior_diferenca(&base, &campo(&[m]));
        assert!(
            d > 1.0e-3,
            "{kind:?}: o espelho numa folha mudou o campo em {d:.6} — era `0.000000` antes desta \
             wave, e é literalmente o report do dono"
        );
    }
}

/// ⭐ **E cada eixo é o SEU eixo** — sem esta metade os três braços podiam ser o mesmo braço.
#[test]
fn each_mirror_folds_its_own_axis() {
    for (eixo, kind) in [
        (0usize, UnaryKind::Mirror),
        (1, UnaryKind::MirrorY),
        (2, UnaryKind::MirrorZ),
    ] {
        let m = Unary::born(kind, ESCALA, ALCANCE);
        let mut n = leaf(caixa(), Xform::default());
        n.mods.push(m);
        let doc = FieldDoc::new(vec![n], NodeId(0)).expect("a peça");
        let f = Field::new(&doc);
        // O gémeo nasce do lado NEGATIVO do eixo espelhado, para lá do plano.
        let mut la = [0.0f64; 3];
        la[eixo] = -2.0 * f64::from(ALCANCE[eixo]);
        assert!(
            f.at(la[0], la[1], la[2]) < 0.0,
            "{kind:?}: não há peça nenhuma do outro lado do plano"
        );
        // E nada apareceu nos outros dois.
        for outro in 0..3 {
            if outro == eixo {
                continue;
            }
            let mut fora = [0.0f64; 3];
            fora[outro] = -2.0 * f64::from(ALCANCE[outro]);
            assert!(
                f.at(fora[0], fora[1], fora[2]) > 0.0,
                "{kind:?}: apareceu peça no eixo {outro}, que não é o espelhado"
            );
        }
    }
}
