//! ⭐⭐⭐ **UMA FILEIRA DE FUROS JÁ SE EXPRIME — e este ficheiro é a MEDIÇÃO que o diz.**
//!
//! # Por que ele existe
//!
//! *«Antes de construir um item de lista aberta, MEÇA se a composição já o exprime»* (`CLAUDE.md`
//! §5.0). Ofereci ao Enio *«uma repetição que fura»* como capacidade nova, e a leitura estava
//! errada em duas frentes: o item aberto chamado *«um laço que SUBTRAI»* (§79.3) é sobre o **gesto
//! do laço de selecção**, não sobre um modificador — e a repetição subtractiva **já funciona**,
//! porque duas leis que já existiam se compõem:
//!
//! 1. **o verbo é por FORMA** (W97, `Node::verb`) — uma forma pode dizer `Difference`;
//! 2. **os modificadores são do NÓ** e correm *antes* de o verbo dobrar a forma no resultado.
//!
//! ⇒ um cilindro com `verb = Difference` e `mods = [Array]` subtrai **todas** as cópias. *O que se
//! perde ao não reconferir não é tempo, é construir o que já existe.*
//!
//! ⚠️ **E isto é um gate, não uma nota:** a composição pode partir-se numa wave futura (a ordem
//! `mods` → `verbo` é o que a faz funcionar), e uma frase num documento não a defende.

use fidget::shape::EzShape;
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Unary, Xform, mods};

const COUNT: u32 = 5;
const SPACING: f32 = 0.30;
const RAIO: f32 = 0.07;

/// A barra com uma fileira de furos: a caixa é a base, o cilindro subtrai, e a repetição do
/// cilindro faz os `COUNT` furos.
fn barra_furada(furar: bool) -> FieldDoc {
    // ⚠️ A barra está CENTRADA no meio da fileira (`0 … (n−1)·s`), e não na origem — senão as
    // últimas cópias caíam fora dela e o gate mediria a barra a acabar, não o furo a faltar.
    let base = Node::new(
        Xform {
            translation: [(COUNT - 1) as f32 * SPACING * 0.5, 0.0, 0.0],
            ..Xform::IDENTITY
        },
        NodeKind::Leaf(Primitive::Box {
            half: [(COUNT - 1) as f32 * SPACING * 0.5 + 0.25, 0.20, 0.20],
            round: 0.0,
            chamfer: 0.0,
        }),
    );
    let mut furo = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Cylinder {
            radius: RAIO,
            half_height: 0.60,
            round: 0.0,
            chamfer: 0.0,
        }),
    );
    // ⚠️ O cilindro nasce no eixo Z e a barra é comprida em X — o furo atravessa a espessura em Z,
    // que é o eixo em que ela é fina. *Sem rotação nenhuma*: a fixtura escolhe-se para medir a lei,
    // não para exercitar a pose.
    if furar {
        // ⭐ **As DUAS leis que se compõem**: o verbo por forma e a repetição do nó.
        furo.verb = Some(Op::Difference(Blend::Sharp));
        furo.mods = vec![Unary::Array {
            count: COUNT,
            spacing: SPACING,
            joint: ph2d_field::Joint::SHARP,
            axis: mods::ARRAY_AXIS,
        }];
    } else {
        furo.verb = Some(Op::Union(Blend::Sharp));
    }
    let raiz = Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op: Op::Union(Blend::Sharp),
            children: vec![NodeId(0), NodeId(1)],
        },
    );
    FieldDoc::new(vec![base, furo, raiz], NodeId(2)).expect("a peça")
}

fn amostra(doc: &FieldDoc, pts: &[[f32; 3]]) -> Vec<f32> {
    let shape = ph2d_field_eval::Engine::from(ph2d_field_eval::compile(doc));
    let tape = shape.ez_float_slice_tape();
    let mut ev = ph2d_field_eval::Engine::new_float_slice_eval();
    let xs: Vec<f32> = pts.iter().map(|p| p[0]).collect();
    let ys: Vec<f32> = pts.iter().map(|p| p[1]).collect();
    let zs: Vec<f32> = pts.iter().map(|p| p[2]).collect();
    ev.eval(&tape, &xs, &ys, &zs).expect("avalia").to_vec()
}

/// ⭐⭐⭐ **A LEI: `verbo = Difference` + `Array` dá `COUNT` furos, e a barra continua inteira entre
/// eles.**
///
/// A régua tem as **duas** metades, e é a segunda que a torna uma prova: os centros dos furos estão
/// **fora** da peça (o campo é positivo) e os pontos **entre** dois furos estão **dentro** (o campo
/// é negativo). Sem a segunda, uma peça que tivesse desaparecido inteira passaria a primeira.
#[test]
fn a_row_of_holes_is_expressible_by_a_shape_verb_and_a_repetition() {
    let doc = barra_furada(true);
    // ⚠️⚠️ **A matriz NÃO é centrada** — ela põe a cópia `k` em `k·spacing` a partir da origem
    // LOCAL do nó, em `+X` (a lei está em `stack::array`: `clamp(round(x/s), 0, count−1)`). A 1.ª
    // redacção desta régua supôs `−(n−1)/2 … +(n−1)/2` e reprovou sobre produto **correcto**:
    // *uma fixtura que adivinha a convenção mede a adivinhação.*
    let centros: Vec<[f32; 3]> = (0..COUNT).map(|i| [i as f32 * SPACING, 0.0, 0.0]).collect();
    let entre: Vec<[f32; 3]> = (0..COUNT - 1)
        .map(|i| [(i as f32 + 0.5) * SPACING, 0.0, 0.0])
        .collect();

    let dentro_do_furo = amostra(&doc, &centros);
    for (i, v) in dentro_do_furo.iter().enumerate() {
        assert!(
            *v > 0.0,
            "o furo {i} não existe: o centro dele mede {v:.5} e devia estar FORA da peça"
        );
    }
    let entre_furos = amostra(&doc, &entre);
    for (i, v) in entre_furos.iter().enumerate() {
        assert!(
            *v < 0.0,
            "a barra partiu-se entre os furos {i} e {}: o meio mede {v:.5}",
            i + 1
        );
    }

    // ⛔ **O CONTROLE**: com o verbo em `Union` os mesmos pontos estão todos DENTRO — é isso que
    // prova que quem abriu os furos foi o verbo, e não a peça ter encolhido.
    let cheio = barra_furada(false);
    for (i, v) in amostra(&cheio, &centros).iter().enumerate() {
        assert!(
            *v < 0.0,
            "sem subtrair, o ponto {i} devia estar dentro da peça e mede {v:.5}"
        );
    }
}

/// ⭐⭐ **E uma COROA de rasgos é a mesma composição — com uma condição que o artista tem de saber.**
///
/// ⚠️⚠️ **Um modificador age no referencial LOCAL do nó, ANTES da pose dele** (a mesma lei que o
/// doc do `Unary::Mirror` já declara). ⇒ pôr `Radial` no próprio rasgo e afastá-lo pela pose **não
/// faz coroa nenhuma**: as `N` cópias rodam à volta da origem local do rasgo, que é onde ele já
/// está, e coincidem todas. Medido: o rasgo `0` abre e os outros `N−1` não.
///
/// ⭐ A composição que a exprime é **o GRUPO**: o desvio vive no filho, e o `Radial` no grupo — ali
/// a origem local é o centro do disco, que é onde a coroa tem de girar. *Não é uma limitação da
/// repetição: é a diferença entre «rodar a peça» e «rodar em torno de quê».*
#[test]
fn a_ring_of_slots_is_the_same_two_laws() {
    const N: u32 = 6;
    const R: f32 = 0.45;
    let disco = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Cylinder {
            radius: 0.70,
            half_height: 0.10,
            round: 0.0,
            chamfer: 0.0,
        }),
    );
    // O rasgo, **afastado dentro do grupo** — é este desvio que dá raio à coroa.
    let rasgo = Node::new(
        Xform {
            translation: [R, 0.0, 0.0],
            ..Xform::IDENTITY
        },
        NodeKind::Leaf(Primitive::Cylinder {
            radius: 0.09,
            half_height: 0.40,
            round: 0.0,
            chamfer: 0.0,
        }),
    );
    let mut coroa = Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op: Op::Union(Blend::Sharp),
            children: vec![NodeId(1)],
        },
    );
    coroa.verb = Some(Op::Difference(Blend::Sharp));
    coroa.mods = vec![Unary::Radial {
        count: N,
        joint: ph2d_field::Joint::SHARP,
        axis: mods::RADIAL_AXIS,
    }];
    let raiz = Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op: Op::Union(Blend::Sharp),
            children: vec![NodeId(0), NodeId(2)],
        },
    );
    let doc = FieldDoc::new(vec![disco, rasgo, coroa, raiz], NodeId(3)).expect("a peça");

    let no_rasgo: Vec<[f32; 3]> = (0..N)
        .map(|k| {
            let a = std::f32::consts::TAU * k as f32 / N as f32;
            [R * a.cos(), R * a.sin(), 0.0]
        })
        .collect();
    let entre: Vec<[f32; 3]> = (0..N)
        .map(|k| {
            let a = std::f32::consts::TAU * (k as f32 + 0.5) / N as f32;
            [R * a.cos(), R * a.sin(), 0.0]
        })
        .collect();
    for (k, v) in amostra(&doc, &no_rasgo).iter().enumerate() {
        assert!(*v > 0.0, "o rasgo {k} não foi aberto: mede {v:.5}");
    }
    for (k, v) in amostra(&doc, &entre).iter().enumerate() {
        assert!(
            *v < 0.0,
            "o disco partiu-se entre os rasgos {k} e o seguinte: {v:.5}"
        );
    }
}
