//! ⭐⭐⭐ **A COSTURA ENTRE AS CÓPIAS** — os gates do pedido do Enio de 2026-08-30.
//!
//! > *«em radial e outros modificadores que geram cópias da mesma peça não temos nem filet nem
//! > chamfer para a união entre as peças»*
//!
//! ⚠️ **O gate que importa aqui não é o que prova que a junta morde** — esse é fácil. É o
//! [`the_seam_does_not_move_the_face_of_an_end_copy`], que prova que ela **não** morde onde não há
//! costura nenhuma. As leis de repetição desta casa prendem o índice da cópia vizinha, e nesses
//! pontos a "vizinha" é a própria cópia: `min(a, a)` é `a`, mas `blend(a, a)` **não é**.

use ph2d_field::{FieldDoc, Joint, NodeId, Primitive, Unary, Xform};
use ph2d_field_eval::{Field, leaf};

/// Uma esfera com a pilha de modificadores dada — o caminho do produto, do documento ao campo.
fn com_mods(mods: Vec<Unary>) -> FieldDoc {
    let mut node = leaf(Primitive::Sphere { radius: R }, Xform::IDENTITY);
    node.mods = mods;
    FieldDoc::new(vec![node], NodeId(0)).expect("a peça")
}

const R: f32 = 0.35;
/// As cópias **cruzam-se**: sem vinco côncavo não há o que costurar.
const S: f32 = 0.55;

fn campo(doc: &FieldDoc) -> Field {
    Field::from_tree(&ph2d_field_eval::compile(doc))
}

/// O `y` da superfície acima de `(x, 0, 0)`, por bissecção a partir de dentro.
fn superficie_y(f: &Field, x: f64) -> Option<f64> {
    if f.at(x, 0.0, 0.0) > 0.0 {
        return None;
    }
    let (mut lo, mut hi) = (0.0f64, 4.0);
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if f.at(x, mid, 0.0) <= 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

/// O `x` da superfície à direita de `(x0, 0, 0)`, por bissecção — a **face de fora** de uma cópia.
fn superficie_x(f: &Field, x0: f64, ate: f64) -> Option<f64> {
    if f.at(x0, 0.0, 0.0) > 0.0 {
        return None;
    }
    let (mut lo, mut hi) = (x0, ate);
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if f.at(mid, 0.0, 0.0) <= 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

/// ⭐ **O CAMINHO DE OMISSÃO É BYTE-IDÊNTICO** — uma junta viva não põe um nó na árvore.
///
/// ⚠️ É a régua da FORMA da árvore, e não do valor: o traçado especializa a fita por ladrilho, e um
/// nó a mais em toda peça já autorada custa em cada uma delas. Ver a lei do `stacked`.
#[test]
fn a_sharp_seam_costs_nothing() {
    for mods in [
        vec![Unary::Array {
            count: 4,
            spacing: S,
            joint: Joint::SHARP,

            axis: ph2d_field::mods::ARRAY_AXIS,
        }],
        vec![Unary::Radial {
            count: 6,
            joint: Joint::SHARP,

            axis: ph2d_field::mods::RADIAL_AXIS,
        }],
    ] {
        let doc = com_mods(mods.clone());
        let arvore = ph2d_field_eval::compile(&doc);
        // A árvore de uma junta viva tem de ser a MESMA que a de antes da feature existir, e a única
        // forma honesta de o afirmar é contar: um `union_between_copies` com `Joint::SHARP` devolve
        // o `min` sem tocar na árvore.
        let f = Field::from_tree(&arvore);
        let dentro = f.at(0.0, 0.0, 0.0);
        assert!(
            (dentro - f64::from(-R)).abs() < 1e-9,
            "{mods:?}: o centro da cópia 0 tem de ler −R exacto, leu {dentro}"
        );
    }
}

/// A distância da origem à superfície ao longo da direcção `(dx, dy)`, por bissecção.
fn superficie_no_raio(f: &Field, dx: f64, dy: f64, ate: f64) -> f64 {
    let (mut lo, mut hi) = (0.0f64, ate);
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if f.at(dx * mid, dy * mid, 0.0) <= 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// ⭐ **A JUNTA MORDE O VINCO DA MATRIZ** — os dois números, com o chanfro a morder mais.
#[test]
fn the_seam_bites_the_crease_of_an_array() {
    // Onde o vinco está sem junta: a circunferência de intersecção das duas esferas.
    let vinco = f64::from(R * R - S * S * 0.25).sqrt();
    let em = |joint: Joint| {
        superficie_y(
            &campo(&com_mods(vec![Unary::Array {
                count: 2,
                spacing: S,
                joint,

                axis: ph2d_field::mods::ARRAY_AXIS,
            }])),
            f64::from(S) * 0.5,
        )
        .expect("a peça existe")
    };
    let vivo = em(Joint::SHARP);
    let filetado = em(Joint {
        chamfer: 0.0,
        fillet: 0.08,
    });
    let chanfrado = em(Joint {
        chamfer: 0.08,
        fillet: 0.0,
    });
    assert!(
        (vivo - vinco).abs() < 1e-4,
        "sem junta a superfície tem de ficar no vinco ({vinco:.5}), ficou {vivo:.5}"
    );
    assert!(
        filetado > vivo + 0.02,
        "o filete tem de ENCHER o vinco — {vivo:.5} -> {filetado:.5}"
    );
    // ⭐ O chanfro morde MAIS que o filete com o mesmo número, e é a FORMA dele: medido `1,71×` em
    // `the_four_characters`, e `1,68×` nesta fixtura.
    assert!(
        chanfrado > filetado,
        "o chanfro morde mais que o filete — {filetado:.5} contra {chanfrado:.5}"
    );
}

/// ⭐ **A JUNTA MORDE O VINCO DA COROA** — e este gate faltava.
///
/// # ⛔ Ele existe porque uma prova de mutação SOBREVIVEU
///
/// A 1.ª redacção media as duas repetições no mesmo laço e fechava as asserções dentro de um
/// `if nome == "Array"`. ⇒ o braço do `Radial` **calculava três números e não afirmava nenhum**, e
/// apagar a junta da coroa inteira (`radial(.., Joint::SHARP, ..)`) deixava a suíte **verde**.
/// *Um laço sobre dois casos com as asserções num deles é um caso testado e outro visitado.*
#[test]
fn the_seam_bites_the_crease_of_a_radial_crown() {
    // ⚠️ A forma tem de estar fora do eixo **no espaço do modificador**, e a pilha corre ANTES da
    // pose — ver `the_seam_does_not_move_the_surface_at_a_wedge_centre`. Daí o grupo.
    // Com braço `0,5` e `6` cópias os centros ficam a `0,5` e as esferas de `0,35` cruzam-se.
    let coroa = |joint: Joint| {
        let filho = leaf(Primitive::Sphere { radius: R }, Xform::at(0.5, 0.0, 0.0));
        let mut grupo = ph2d_field::Node::new(
            Xform::IDENTITY,
            ph2d_field::NodeKind::Combine {
                op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                children: vec![NodeId(0)],
            },
        );
        grupo.mods = vec![Unary::Radial {
            count: 6,
            joint,
            axis: ph2d_field::mods::RADIAL_AXIS,
        }];
        campo(&FieldDoc::new(vec![filho, grupo], NodeId(1)).expect("a coroa"))
    };
    // A costura entre a cópia `0` (em `θ = 0`) e a `1` (em `θ = 60°`) fica na bissectriz, `θ = 30°`.
    let (dx, dy) = (30.0f64.to_radians().cos(), 30.0f64.to_radians().sin());
    let em = |joint: Joint| superficie_no_raio(&coroa(joint), dx, dy, 3.0);
    let vivo = em(Joint::SHARP);
    let filetado = em(Joint {
        chamfer: 0.0,
        fillet: 0.08,
    });
    let chanfrado = em(Joint {
        chamfer: 0.08,
        fillet: 0.0,
    });
    assert!(
        filetado > vivo + 0.01,
        "o filete tem de ENCHER o vinco da coroa — {vivo:.5} -> {filetado:.5}"
    );
    assert!(
        chanfrado > filetado,
        "o chanfro morde mais que o filete — {filetado:.5} contra {chanfrado:.5}"
    );
}

/// ⭐⭐⭐ **O GATE QUE JUSTIFICA A LEI — a costura NÃO move a face de fora de uma cópia da ponta.**
///
/// # ⛔⛔ O defeito, e por que nenhuma sonda desta casa o via
///
/// A matriz avalia **duas** células: a do ponto e a do lado para onde ele pende, com o índice
/// **preso** a `[0, count−1]`. Na ponta de fora da primeira cópia o ponto pende para `−1`, o `clamp`
/// devolve `0`, e as duas avaliações são a **mesma** cópia.
///
/// `min(a, a) = a`, e por isso isto nunca incomodou ninguém. Mas
/// `union_round(a, a, r) = max(a, r) − √2·(r−a)⁺`, que em `a = 0` vale `r(1 − √2) = −0,414·r` — o
/// campo diz *«dentro»* sobre um ponto que está **na** superfície, e a face de fora da primeira e da
/// última cópia **incha**.
///
/// ⇒ a cura é o portão do `ops_joint::union_between_copies`, e este gate é o que a defende.
#[test]
fn the_seam_does_not_move_the_face_of_an_end_copy() {
    let sem = campo(&com_mods(vec![Unary::Array {
        count: 3,
        spacing: S,
        joint: Joint::SHARP,

        axis: ph2d_field::mods::ARRAY_AXIS,
    }]));
    let com = campo(&com_mods(vec![Unary::Array {
        count: 3,
        spacing: S,
        joint: Joint {
            chamfer: 0.06,
            fillet: 0.06,
        },

        axis: ph2d_field::mods::ARRAY_AXIS,
    }]));
    // A face de FORA da primeira cópia: caminhamos de dentro dela para `−x`.
    let borda = |f: &Field| {
        let (mut lo, mut hi) = (f64::from(-R) * 3.0, 0.0f64);
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if f.at(mid, 0.0, 0.0) > 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    };
    let (a, b) = (borda(&sem), borda(&com));
    assert!(
        (a - b).abs() < 1e-6,
        "a costura moveu a face de FORA da primeira cópia: {a:.7} -> {b:.7}. \
         É o `blend(a, a) != a` — ver `ops_joint::union_between_copies`"
    );
    // ⭐ E o mesmo do outro lado, na última cópia — o `clamp` prende as duas pontas.
    let fim = |f: &Field| superficie_x(f, f64::from(S) * 2.0, f64::from(S) * 2.0 + 4.0);
    let (a, b) = (fim(&sem).expect("sem"), fim(&com).expect("com"));
    assert!(
        (a - b).abs() < 1e-6,
        "a costura moveu a face de FORA da última cópia: {a:.7} -> {b:.7}"
    );
}

/// ⭐⭐⭐ **A costura não move a superfície no CENTRO EXACTO de uma fatia** — o outro sítio em que a
/// vizinha é a própria cópia.
///
/// ⚠️ Ali quem devolve o índice repetido não é o `clamp`, é o `compare`, que vale `0` quando o ponto
/// está exactamente no centro. ⛔ E é o pior sítio possível para um defeito: o centro de uma célula é
/// onde a superfície daquela cópia passa, então a peça inteira ficaria com uma crista por cópia.
#[test]
fn the_seam_does_not_move_the_surface_at_a_cell_centre() {
    for joint in [
        Joint::SHARP,
        Joint {
            chamfer: 0.0,
            fillet: 0.07,
        },
    ] {
        let f = campo(&com_mods(vec![Unary::Array {
            count: 4,
            spacing: S,
            joint,

            axis: ph2d_field::mods::ARRAY_AXIS,
        }]));
        // O centro da célula 1 é `x = S`. A superfície acima dele é o topo daquela esfera.
        let y = superficie_y(&f, f64::from(S)).expect("a cópia 1 existe");
        assert!(
            (y - f64::from(R)).abs() < 1e-6,
            "{joint:?}: no centro de uma célula a superfície tem de ser a da própria cópia \
             ({R}), leu {y:.7}"
        );
    }
}

/// ⭐⭐⭐ **O MESMO no centro de uma FATIA da coroa** — e este é o gate que faltava.
///
/// # ⛔ Ele existe porque uma prova de mutação SOBREVIVEU
///
/// A 1.ª versão desta wave curava o índice repetido **duas** vezes: pelo portão *e* por um «sinal
/// sem zero» que impedia o `compare` de apontar para a própria célula. Trocar o segundo de volta
/// pelo `compare` cru deixou todos os gates verdes — porque na **matriz** o portão já bastava.
///
/// ⇒ ficou uma lei só (o portão), e este gate é o que a defende no sítio onde ela é a **única** cura:
/// a coroa não tem `clamp`, então ali o índice repetido só nasce do `compare` valer `0`.
///
/// ⚠️ **A fixtura tem de pôr a peça FORA do eixo** — uma esfera centrada na origem é invariante à
/// rotação, e a repetição radial dela é ela mesma: o defeito existiria e não se veria.
///
/// ⛔⛔ **E a 1.ª redacção media a DEGENERESCÊNCIA, não a lei — a causa é uma que morde qualquer
/// fixtura deste módulo: a pilha de modificadores corre ANTES da pose do nó.**
///
/// Pôr `Xform::at(braço, 0, 0)` num nó-**folha** não afasta a forma do eixo do modificador: a coroa
/// repetia uma esfera centrada na origem local — **invariante à rotação** —, as cópias coincidiam
/// ponto a ponto, e a superfície subia `r·(1 − 1/√2) = 0,0205` em toda a peça. ⚠️ O sinal de que era
/// isso: mudar o braço de `0,5` para `1,0` deu **exactamente o mesmo número**. *Uma medição que não
/// se mexe quando o parâmetro se mexe está a medir outra coisa.*
///
/// ⇒ a forma fica fora do eixo por ser **filha posada de um grupo**, e é o grupo que leva a coroa.
#[test]
fn the_seam_does_not_move_the_surface_at_a_wedge_centre() {
    // A `1,0` os centros ficam a `1,176` e a vizinha está a `0,566` do ponto medido — oito vezes o
    // filete. Tudo o que se mexer aqui é o índice repetido.
    let braco = 1.0f32;
    let ao_lado = |joint: Joint| {
        let filho = leaf(Primitive::Sphere { radius: R }, Xform::at(braco, 0.0, 0.0));
        let mut grupo = ph2d_field::Node::new(
            Xform::IDENTITY,
            ph2d_field::NodeKind::Combine {
                op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                children: vec![NodeId(0)],
            },
        );
        grupo.mods = vec![Unary::Radial {
            count: 5,
            joint,
            axis: ph2d_field::mods::RADIAL_AXIS,
        }];
        campo(&FieldDoc::new(vec![filho, grupo], NodeId(1)).expect("a coroa"))
    };
    // ⚠️ **A medição tem de ser EM `θ = 0`**, e a 1.ª redacção não era: ela subia em `y` a partir de
    // `(braço, 0, 0)` e cruzava a superfície em `θ = 19,3°`, onde o `compare` já vale `1` e o portão
    // não é a lei em jogo — a mutação que o forçava a `1` **sobrevivia**. O semi-eixo `+X` é o
    // centro da fatia `0`, e ali a superfície está a `braço + R` da origem.
    for joint in [
        Joint::SHARP,
        Joint {
            chamfer: 0.0,
            fillet: 0.07,
        },
    ] {
        let f = ao_lado(joint);
        let x = superficie_no_raio(&f, 1.0, 0.0, 4.0);
        let esperado = f64::from(braco + R);
        assert!(
            (x - esperado).abs() < 1e-6,
            "{joint:?}: no centro de uma FATIA a superfície tem de ser a da própria cópia \
             ({esperado}), leu {x:.7} — é o `compare` a devolver 0, ver \
             `ops_joint::distinct_copies`"
        );
    }
}

/// ⭐⭐ **A lei das DUAS células com portão concorda com o oráculo** — todas as cópias, escritas uma
/// a uma.
///
/// ⚠️ E a coluna que decide não é o `|Δ|`, é a **sobre-estimativa**: um campo maior do que a
/// distância faz a marcha saltar por cima da superfície, e é isso que fura a peça. Um campo menor
/// só custa passos.
#[test]
fn the_two_cell_law_never_overestimates_with_a_seam() {
    for fillet in [0.0f32, 0.04, 0.08, 0.16] {
        let joint = Joint {
            chamfer: 0.0,
            fillet,
        };
        let nosso = campo(&com_mods(vec![Unary::Array {
            count: 4,
            spacing: S,
            joint,

            axis: ph2d_field::mods::ARRAY_AXIS,
        }]));
        // O oráculo: quatro esferas em união, cada uma no lugar dela, com a MESMA junta.
        let mut nodes = Vec::new();
        for i in 0..4u32 {
            nodes.push(leaf(
                Primitive::Sphere { radius: R },
                Xform::at(S * i as f32, 0.0, 0.0),
            ));
        }
        let filhos: Vec<NodeId> = (0..4).map(NodeId).collect();
        nodes.push(ph2d_field::Node::new(
            Xform::IDENTITY,
            ph2d_field::NodeKind::Combine {
                op: ph2d_field::Op::Union(ph2d_field::Blend::Exact { radius: fillet }),
                children: filhos,
            },
        ));
        let doc = FieldDoc::new(nodes, NodeId(4)).expect("o oráculo");
        let orac = campo(&doc);
        let mut acima = 0.0f64;
        for i in 0..41 {
            for j in 0..41 {
                for k in 0..41 {
                    let u = |n: i32| f64::from(n) / 40.0;
                    let x = u(i).mul_add(3.0f64.mul_add(f64::from(S), 1.4), -0.7);
                    let y = u(j).mul_add(1.4, -0.7);
                    let z = u(k).mul_add(1.4, -0.7);
                    let d = nosso.at(x, y, z) - orac.at(x, y, z);
                    if d.is_finite() {
                        acima = acima.max(d);
                    }
                }
            }
        }
        assert!(
            acima < 1e-6,
            "filete {fillet}: a lei das duas células SOBRE-estimou por {acima:.6} — \
             um campo maior que a distância salta por cima da superfície"
        );
    }
}

/// ⭐⭐⭐ **O PASSO DA MARCHA CONTA A COSTURA** — a metade que impede a peça de furar.
///
/// ⛔ O [`ph2d_field_eval::gradient_bound`] declarava, com medição ao lado, que `Array`/`Radial` lêem
/// `‖∇f‖ = 1,000` e por isso não entravam na conta. Uma repetição com junta é uma união arredondada
/// como qualquer outra, e torna essa frase falsa. *Uma lei medida envelhece no dia em que alguém
/// acrescenta um produtor novo do mesmo efeito.*
#[test]
fn the_march_step_pays_for_the_seam() {
    let vivo = com_mods(vec![Unary::Radial {
        count: 6,
        joint: Joint::SHARP,

        axis: ph2d_field::mods::RADIAL_AXIS,
    }]);
    let costurado = com_mods(vec![Unary::Radial {
        count: 6,
        joint: Joint {
            chamfer: 0.0,
            fillet: 0.05,
        },

        axis: ph2d_field::mods::RADIAL_AXIS,
    }]);
    let (a, b) = (
        ph2d_field_eval::safe_march_step(&vivo),
        ph2d_field_eval::safe_march_step(&costurado),
    );
    assert!(
        (a - 1.0).abs() < 1e-6,
        "uma costura viva não pode custar passo nenhum, custou {a}"
    );
    assert!(
        b < a,
        "a costura tem de encurtar o passo — {a} contra {b}. Sem isto a marcha anda o valor \
         cheio sobre um campo que sobe mais depressa que a distância, e a peça fura"
    );
    // ⭐ E o número é o do balde que a casa já paga por um arredondamento exacto: `1/√2`.
    assert!(
        (b - 1.0 / std::f32::consts::SQRT_2).abs() < 1e-6,
        "o passo com costura tem de ser 1/√2, é {b}"
    );
}

/// ⭐ **O bordo da peça cresce com a costura** — um bordo que não a conte recorta a peça na marcha e
/// na exportação, que é o defeito que a inclinação custou a esta linha em 2026-08-30.
#[test]
fn the_bounding_ball_grows_with_the_seam() {
    let reg = ph2d_field_eval::hybrid::Registry::default();
    let bola = |joint: Joint| {
        ph2d_field_eval::bounds::bounding_ball(
            &com_mods(vec![Unary::Array {
                count: 3,
                spacing: S,
                joint,

                axis: ph2d_field::mods::ARRAY_AXIS,
            }]),
            &reg,
        )
        .expect("a bola")
        .radius
    };
    let vivo = bola(Joint::SHARP);
    let costurado = bola(Joint {
        chamfer: 0.05,
        fillet: 0.03,
    });
    assert!(
        (costurado - vivo - 0.08).abs() < 1e-5,
        "a bola tem de crescer o alcance da junta (0,08): {vivo} -> {costurado}"
    );
}
