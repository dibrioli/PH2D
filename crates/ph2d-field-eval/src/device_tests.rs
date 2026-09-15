//! Os gates da compilação para o dispositivo — ver [`super`].

use super::DeviceField;
use crate::hybrid::{Registry, Sampled, SampledGrid};
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use std::sync::Arc;

/// Uma grade densa que é uma esfera — a folha amostrada mais simples que serve de sujeito.
struct Bola {
    dims: [u32; 3],
    origin: [f32; 3],
    step: f32,
    dist: Vec<f32>,
}

impl Bola {
    fn nova(raio: f32, n: u32) -> Self {
        let step = 4.0 * raio / (n - 1) as f32;
        let origin = [-2.0 * raio; 3];
        let mut dist = Vec::with_capacity((n * n * n) as usize);
        for z in 0..n {
            for y in 0..n {
                for x in 0..n {
                    let p = [x, y, z].map(|i| (i as f32).mul_add(step, origin[0]));
                    dist.push((p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt() - raio);
                }
            }
        }
        Self {
            dims: [n; 3],
            origin,
            step,
            dist,
        }
    }
}

impl Sampled for Bola {
    fn at(&self, _p: [f32; 3]) -> f32 {
        unreachable!("estes gates medem a COMPILAÇÃO, não a avaliação de CPU")
    }
    fn bounding_radius(&self) -> f32 {
        2.0 * self.origin[0].abs()
    }
    fn grid(&self) -> Option<SampledGrid<'_>> {
        Some(SampledGrid {
            dims: self.dims,
            origin: self.origin,
            step: self.step,
            values: &self.dist,
        })
    }
}

/// Uma folha amostrada que **não** sabe entregar a grade — o lado que tem de recusar.
struct SemGrade;
impl Sampled for SemGrade {
    fn at(&self, _p: [f32; 3]) -> f32 {
        0.0
    }
    fn bounding_radius(&self) -> f32 {
        1.0
    }
}

fn combina(op: Op, filhos: Vec<NodeId>) -> Node {
    Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op,
            children: filhos,
        },
    )
}

/// Uma caixa unida a uma escultura — a forma que o produto tem.
fn peca() -> FieldDoc {
    FieldDoc::new(
        vec![
            crate::leaf(
                Primitive::Box {
                    half: [0.4, 0.15, 0.4],
                    round: 0.02,
                    chamfer: 0.0,
                },
                Xform::at(0.0, -0.4, 0.0),
            ),
            Node::new(
                Xform::at(0.1, 0.0, 0.0),
                NodeKind::Sampled { key: "bola".into() },
            ),
            combina(
                Op::Union(Blend::Exact { radius: 0.1 }),
                vec![NodeId(0), NodeId(1)],
            ),
        ],
        NodeId(2),
    )
    .expect("a peça com escultura")
}

fn registo() -> Registry {
    let mut r = Registry::new();
    r.insert(
        "bola".into(),
        Arc::new(Bola::nova(0.35, 24)) as Arc<dyn Sampled>,
    );
    r
}

/// ⭐⭐⭐ **A ESCULTURA VIRA UMA VARIÁVEL, e a árvore continua a ser uma só.**
///
/// ⚠️ **É isto que evita uma TERCEIRA cópia da lei das booleanas** — ver a nota do módulo. O gate
/// mede as duas metades: a fita existe (logo o filete e a união ficaram lá dentro) e ela **chama**
/// a escultura.
#[test]
fn a_escultura_entra_na_fita_como_uma_chamada() {
    let campo = DeviceField::new(&peca(), &registo()).expect("a peça compila para o dispositivo");
    assert_eq!(campo.sculpts().len(), 1, "a peça tem uma escultura");
    let fita = campo.tape_wgsl().expect("a fita existe");
    let chamada = format!("{}0(p)", crate::wgsl::ESCULTURA);
    assert!(
        fita.source.contains(&chamada),
        "a fita não chama a escultura — ela desapareceria da peça:\n{}",
        &fita.source[..fita.source.len().min(400)]
    );
    // ⭐ **O CONTROLO: a união e o filete continuam lá.** Sem ele, uma fita que fosse SÓ a chamada
    // à escultura passaria a primeira asserção — e a peça sairia sem a caixa.
    //
    // ⛔⛔ **A 1.ª redacção contava LINHAS do shader, e isso era uma procuração que se moveu
    // sozinha:** quando o emissor deixou de dar um `let` a cada constante (elas são escritas onde
    // são usadas), a mesma peça passou de `41` para `38` linhas e este gate reprovou **com a
    // álgebra inteira lá dentro**. ⇒ o piso é sobre o que a peça CALCULA (`guardados`), que é uma
    // propriedade da peça, e não sobre como o gerador a escreve.
    let guardados = campo.tape_shape().expect("a forma").guardados;
    assert!(
        fita.source.matches("min(").count() >= 1 && guardados > 30,
        "a fita calcula só {guardados} valores — a álgebra da peça não está lá"
    );
}

/// ⛔⛔ **UMA FOLHA SEM GRADE RECUSA A PEÇA INTEIRA** — tudo ou nada.
///
/// ⚠️ Uma escultura em falta não é «uma folha a menos»: é a peça **com um buraco** onde havia
/// matéria, e o resto dela perfeito. *É exactamente o defeito que a porta antiga existia para
/// impedir, e ele não deixou de existir por a grade passar a atravessar.*
#[test]
fn uma_folha_sem_grade_recusa_a_peca() {
    let mut r = Registry::new();
    r.insert("bola".into(), Arc::new(SemGrade) as Arc<dyn Sampled>);
    assert!(
        DeviceField::new(&peca(), &r).is_none(),
        "a peça foi aceite com uma folha que não sabe entregar a grade"
    );
    // ⭐ O controlo: com a grade, ela é aceite.
    assert!(DeviceField::new(&peca(), &registo()).is_some());
}

/// ⭐⭐ **UM NOME DESCONHECIDO É ESPAÇO VAZIO**, e a peça continua a abrir — a mesma resposta do
/// [`crate::hybrid`].
#[test]
fn um_nome_desconhecido_le_como_espaco_vazio() {
    let campo = DeviceField::new(&peca(), &Registry::new()).expect("a peça abre sem o registo");
    assert!(
        campo.sculpts().is_empty(),
        "um nome que o registo não conhece não produz escultura nenhuma"
    );
    assert!(campo.tape_wgsl().is_some(), "e a fita continua a existir");
}

/// ⭐⭐⭐ **SEM ESCULTURA, A FITA É A DE SEMPRE — ao byte.**
///
/// ⚠️ **É a metade que prova a inércia.** Toda esta wave existe para a peça COM escultura; se ela
/// mudasse um bit da peça sem, o caminho de omissão teria mudado sem ninguém pedir.
#[test]
fn sem_escultura_a_fita_e_a_de_sempre() {
    let doc = FieldDoc::new(
        vec![crate::leaf(
            Primitive::Sphere { radius: 0.4 },
            Xform::at(0.1, 0.0, 0.0),
        )],
        NodeId(0),
    )
    .expect("a esfera");
    let device = DeviceField::new(&doc, &Registry::new())
        .expect("compila")
        .tape_wgsl()
        .expect("a fita");
    let sempre = crate::Field::new(&doc)
        .tape_wgsl()
        .expect("a fita de sempre");
    assert_eq!(device.source, sempre.source, "o texto da fita mudou");
    assert_eq!(
        device.consts, sempre.consts,
        "as constantes da fita mudaram"
    );
}
