//! Os gates do CATÁLOGO de formas (W100) — ver [`super`].

use super::{Family, Make, SHAPES, available, shape_at, slot_of};

/// ⭐⭐⭐ **A CORRENTE QUE FECHA O BURACO: toda primitiva que o motor sabe fazer tem uma linha aqui.**
///
/// ⚠️ Este gate herdou o trabalho do `every_primitive_the_engine_can_make_has_a_button`, e a razão
/// dele é a W53: o `Extrude`/`Revolve` existiam **desde a W3** e nenhum botão os alcançava — uma
/// família de features inteira, completa e invisível. A corrente é
/// `Primitive` novo ⇒ erro de compilação em `Primitive::kind` ⇒ variante nova em `PrimitiveKind`
/// ⇒ `PrimitiveKind::ALL` não compila sem ela ⇒ **este laço reprova até haver a linha**.
#[test]
fn every_primitive_the_engine_can_make_is_in_the_catalogue() {
    for k in ph2d_field::PrimitiveKind::ALL {
        assert!(
            SHAPES.iter().any(|s| s.key.ends_with(k.key())),
            "a primitiva {k:?} não tem linha no catálogo - ela é inalcançável pela paleta"
        );
    }
}

/// ⭐⭐⭐ **A chave é a IDENTIDADE, então nenhuma se repete.**
///
/// ⚠️ Duas linhas com a mesma chave dariam **o mesmo id de item** na paleta, e o pick resolveria
/// sempre na primeira — a segunda forma seria pintada, clicável, e criaria a outra. É o modo de
/// falha mais caro que existe (parece funcionar), e com 60 linhas por vir uma colisão de
/// copiar-e-colar é o erro esperado, não o improvável.
#[test]
fn no_two_shapes_share_a_key() {
    for (i, a) in SHAPES.iter().enumerate() {
        for b in SHAPES.iter().skip(i + 1) {
            assert_ne!(a.key, b.key, "duas formas partilham a chave {}", a.key);
        }
    }
}

/// ⭐⭐ **O construtor da linha é o que decide, e não a posição** — a lei que a W100 comprou.
///
/// ⚠️ **Este é o gate que a W53 não podia ter.** Antes, as quatro entradas não-primitivas eram
/// alcançadas por `SHAPES.len() - 4`, `- 3`, `- 2`, `- 1`: acrescentar uma forma **no fim** fazia o
/// *Extrude* passar a abrir o diálogo de escultura, sem erro nenhum. Aqui a pergunta é sobre o
/// [`Make`] da própria linha, então a lista pode crescer em qualquer sítio.
#[test]
fn only_the_formula_shapes_build_from_a_radius() {
    for (slot, shape) in SHAPES.iter().enumerate() {
        let built = shape_at(slot, 0.5);
        match shape.make {
            Make::Formula(_) => assert!(
                built.is_some(),
                "{} diz-se de fórmula e não construiu nada",
                shape.key
            ),
            // ⚠️ Um contorno desenhado e um arquivo vivem **fora do mundo**: quem os trata é o
            // braço próprio, e um `Some` aqui seria uma forma nascida do nada no sítio errado.
            Make::Extrude | Make::Revolve | Make::Sculpt | Make::SculptScene => assert!(
                built.is_none(),
                "{} não sai de um raio - o `shape_at` tem de recusar",
                shape.key
            ),
        }
    }
}

/// ⭐ **Fora da lista não há forma nenhuma** — e é `None`, não a primeira.
#[test]
fn a_slot_past_the_catalogue_builds_nothing() {
    assert!(shape_at(SHAPES.len(), 0.5).is_none());
    assert!(slot_of("panel.model3d.add.nao_existe").is_none());
}

/// ⭐⭐ **A DISPONIBILIDADE é a lei da W34**, e cada `Make` responde por si.
///
/// ⚠️ O controlo é o que faz o gate valer: com as duas condições **desligadas**, as de fórmula
/// continuam disponíveis. Sem ele, um `available` que devolvesse sempre `false` passaria a metade
/// de cima e o defeito seria *"nenhum botão faz nada"*.
#[test]
fn only_what_needs_a_selection_waits_for_one() {
    for shape in SHAPES {
        let sempre = available(shape, false, false);
        match shape.make {
            Make::Formula(_) | Make::Sculpt => assert!(
                sempre,
                "{} não depende de nada e devia estar sempre disponível",
                shape.key
            ),
            Make::Extrude | Make::Revolve => {
                assert!(!sempre, "{} precisa de um contorno", shape.key);
                assert!(available(shape, false, true), "{} com contorno", shape.key);
            }
            Make::SculptScene => {
                assert!(!sempre, "{} precisa de escultura na cena", shape.key);
                assert!(available(shape, true, false), "{} com escultura", shape.key);
            }
        }
    }
}

/// ⭐ **Toda família da paleta tem título e tinta próprios.**
///
/// ⚠️ Duas famílias com a mesma tinta leem-se como uma só na paleta, e o título é o que separa os
/// grupos — é a cor que *ensina o mapa* do catálogo (a lição que a biblioteca do Motion registou).
#[test]
fn each_family_has_its_own_title_and_colour() {
    for (i, a) in Family::ALL.iter().enumerate() {
        for b in Family::ALL.iter().skip(i + 1) {
            assert_ne!(a.title(), b.title(), "{a:?} e {b:?} têm o mesmo título");
            assert_ne!(a.color(), b.color(), "{a:?} e {b:?} têm a mesma tinta");
        }
    }
}

/// ⭐⭐ **Uma forma nova nasce com o `round` que tem direito** — e não a zero.
///
/// ⚠️ Este é o módulo cujo argumento **é** o arredondamento: uma caixa de aresta viva ao nascer
/// esconde exatamente aquilo que ele faz melhor que o Blender. A propriedade é sobre a **família**
/// (toda forma que aceita `round` nasce com um), então ela vale para as 60 que vêm, não para as 4
/// que existem.
#[test]
fn every_new_shape_that_can_round_is_born_round() {
    for (slot, shape) in SHAPES.iter().enumerate() {
        let Some(prim) = shape_at(slot, 0.5) else {
            continue;
        };
        let Some(r) = ph2d_field::NodeShape::Leaf(prim).radius() else {
            // Sem `round` no modelo (esfera, toro) — a ausência é do documento, não desta lei.
            continue;
        };
        assert!(
            r > 0.0,
            "{} nasce de aresta viva - o módulo do arredondamento a esconder o que faz",
            shape.key
        );
    }
}

/// ⛔⛔⛔ **O QUE O ARTISTA CRIA MARCHA EM SEGURANÇA?** — o gate que o report do Enio de 30/08
/// comprou (duas fotos: **rasgos pretos** nas junções da cruz e do cone redondo).
///
/// # ⚠️ O buraco que ele fecha
///
/// O censo (`the_census_of_every_primitive`) já pergunta *«a marcha é segura?»* — mas sobre o
/// **representante DELE**, escrito à mão dentro do teste. A paleta cria outra coisa: outros
/// números, outras proporções, outro `round`. ⇒ *o gate media uma forma que o artista nunca cria.*
///
/// ⭐ É a mesma lição que esta casa já tem escrita: **onde os objectos NASCEM é a fixtura que os
/// gates não têm**. Aqui o sujeito é a [`SHAPES`] com o **construtor de cada linha**, que é
/// literalmente o que o botão faz.
///
/// # A lei
///
/// A marcha anda `d · passo` e é segura enquanto `passo · ‖∇f‖ ≤ 1`. Um campo que suba mais
/// depressa **atravessa** a superfície, e o sintoma é um rasgo preto no meio da peça.
#[test]
fn every_shape_the_palette_creates_marches_safely() {
    use ph2d_field::{FieldDoc, NodeId, Xform};
    use ph2d_field_eval::{Field, leaf, safe_march_step};

    const R: f32 = 0.5;
    // Folga de AMOSTRAGEM: a norma sai de diferenças finitas e numa quina lê um pouco acima.
    const SLACK: f64 = 1.02;
    let mut medidas = 0;
    let mut falhas: Vec<String> = Vec::new();
    for (slot, shape) in SHAPES.iter().enumerate() {
        let Some(p) = shape_at(slot, R) else { continue };
        let doc = FieldDoc::new(vec![leaf(p, Xform::IDENTITY)], NodeId(0)).expect("folha válida");
        let passo = f64::from(safe_march_step(&doc));
        let f = Field::new(&doc);
        let (mut pior, mut onde) = (0.0_f64, [0.0_f64; 3]);
        const N: usize = 34;
        for i in 0..N {
            for j in 0..N {
                for k in 0..N {
                    let c = |n: usize| (n as f64 / (N - 1) as f64) * 2.8 - 1.4;
                    let (x, y, z) = (c(i), c(j), c(k));
                    let g = f.gradient_norm(x, y, z, 1.0e-4);
                    if g > pior {
                        pior = g;
                        onde = [x, y, z];
                    }
                }
            }
        }
        medidas += 1;
        if passo * pior > SLACK {
            falhas.push(format!(
                "  «{}»: passo {passo:.4} x |grad| {pior:.4} = {:.4}  (pior em {onde:?})",
                shape.key.rsplit('.').next().unwrap_or(shape.key),
                passo * pior
            ));
        }
    }
    assert!(
        medidas >= 20,
        "só {medidas} formas de fórmula foram medidas — o gate perdeu o sujeito"
    );
    assert!(
        falhas.is_empty(),
        "{} forma(s) que a paleta CRIA atravessam a superfície:\n{}",
        falhas.len(),
        falhas.join("\n")
    );
}

/// ⛔⛔⛔ **A SONDA QUE MEDE O RASGO, e não um proxy dele** (report do Enio, 30/08 — duas fotos com
/// setas para buracos pretos nas junções).
///
/// # ⚠️ Por que uma sonda de GRADIENTE não chega
///
/// `passo × ‖∇f‖ ≤ 1` é a **condição suficiente** para a marcha não atravessar. Ela é medida numa
/// grelha, e uma grelha de `34³` **não vê** um pico que vive numa película fina à volta de uma
/// costura. ⇒ esta sonda mede o **sintoma**: lança raios como o traçador lança, e conta aqueles em
/// que a marcha **falha** uma superfície que a bissecção encontra.
///
/// *Um furo é um raio que devia bater e não bateu — mede-se isso, não um limite superior dele.*
#[test]
#[ignore = "sonda: conta os raios que a marcha ATRAVESSA"]
fn measure_marching_holes_on_a_combined_scene() {
    use ph2d_field::{Blend, FieldDoc, NodeId, NodeKind, Op, Xform};
    use ph2d_field_eval::{Field, leaf, safe_march_step};

    let combine = |op: Op, children: Vec<NodeId>| ph2d_field::Node {
        xform: Xform::IDENTITY,
        kind: NodeKind::Combine { op, children },
        mods: Vec::new(),
        verb: None,
    };

    const R: f32 = 0.5;
    let em = |x: f32| Xform {
        translation: [x, 0.0, 0.0],
        ..Xform::IDENTITY
    };
    // Os nomes vêm da paleta, então o que a sonda mede é o que o botão cria.
    let acha = |chave: &str| {
        let slot = SHAPES
            .iter()
            .position(|s| s.key.ends_with(chave))
            .unwrap_or_else(|| panic!("«{chave}» não está na paleta"));
        shape_at(slot, R).unwrap_or_else(|| panic!("«{chave}» não é de fórmula"))
    };

    // ⭐⭐⭐ **A VARREDURA DA FOLGA** — é ela que reproduz o report. Com as peças AFASTADAS está
    // tudo limpo (`|grad| = 0,9992`); a queixa do Enio diz *«esta' atras do elipsoide e NAO
    // COLADO»*, e é aí que as duas superfícies se olham de perto.
    let mut casos: Vec<(String, Op, f32)> = Vec::new();
    for folga in [0.60_f32, 0.30, 0.12, 0.05, 0.02, 0.008, 0.002] {
        casos.push((
            format!("viva  folga {folga:.3}"),
            Op::Union(Blend::Sharp),
            folga,
        ));
    }
    for (nome, verbo, folga) in casos {
        // O cone redondo tem raio de base `0,275` e a cruz meio-braço `0,5`: encostam-se quando a
        // distância entre centros é `0,775`.
        let d = 0.775 + folga;
        let doc = FieldDoc::new(
            vec![
                leaf(acha("cross"), em(-d * 0.5)),
                leaf(acha("round_cone"), em(d * 0.5)),
                combine(verbo, vec![NodeId(0), NodeId(1)]),
            ],
            NodeId(2),
        )
        .expect("cena válida");
        let passo = f64::from(safe_march_step(&doc));
        let f = Field::new(&doc);

        // Lança raios paralelos ao eixo Z (como uma câmera ortográfica de frente), varrendo XY.
        const N: usize = 56;
        const LONGE: f64 = 3.0;
        let (mut furos, mut batidas) = (0usize, 0usize);
        let (mut degenerados, mut menor) = (0usize, f64::INFINITY);
        // O mesmo `eps` que o traçador usa numa vista normal (meio pixel).
        const EPS_NORMAL: f64 = 1.5e-3;
        for i in 0..N {
            for j in 0..N {
                let c = |n: usize| (n as f64 / (N - 1) as f64) * 2.4 - 1.2;
                let (x, y) = (c(i), c(j));
                // A MARCHA, como o traçador a faz.
                let mut t = -LONGE;
                let mut bateu_marcha = false;
                for _ in 0..512 {
                    let d = f.at(x, y, t);
                    if d < 1.0e-4 {
                        bateu_marcha = true;
                        break;
                    }
                    t += d.max(1.0e-5) * passo;
                    if t > LONGE {
                        break;
                    }
                }
                // A VERDADE: amostragem densa + bissecção.
                const AMOSTRAS: usize = 700;
                let mut bateu_verdade = false;
                let mut anterior = f.at(x, y, -LONGE);
                for k in 1..=AMOSTRAS {
                    let z = -LONGE + (k as f64 / AMOSTRAS as f64) * 2.0 * LONGE;
                    let v = f.at(x, y, z);
                    if anterior > 0.0 && v <= 0.0 {
                        bateu_verdade = true;
                        break;
                    }
                    anterior = v;
                }
                if bateu_verdade {
                    batidas += 1;
                    if bateu_marcha {
                        // ⭐⭐⭐ **A NORMAL no ponto em que a marcha parou.** O traçador lê-a por
                        // diferença central com um `eps` do tamanho de meio pixel; num campo de
                        // distância limpa `‖∇f‖ = 1`. ⚠️ Numa costura entre duas superfícies
                        // QUASE a tocar-se, as duas normais exteriores apontam **uma para a
                        // outra**, a diferença central soma-as e o resultado **cancela**.
                        // ⇒ `‖∇f‖ → 0`, e o que sobra é ruído: a normal aponta para qualquer
                        // lado, inclusive para DENTRO ⇒ o pixel sai **preto**.
                        let g = f.gradient_norm(x, y, t, EPS_NORMAL);
                        if g < 0.5 {
                            degenerados += 1;
                        }
                        menor = menor.min(g);
                    } else {
                        furos += 1;
                    }
                }
            }
        }
        println!(
            "  [{nome}] passo {passo:.4} — {batidas} raios, {furos} furos, \
             {degenerados} NORMAIS DEGENERADAS ({:.2} %), menor |grad| = {menor:.4}",
            100.0 * degenerados as f64 / batidas.max(1) as f64
        );
    }
}
