//! ⛔⛔⛔ **UM DEFORMADOR ANTES DA REPETIÇÃO RASGAVA O CAMPO** — auditoria de 2026-08-30.
//!
//! O mecanismo-título da wave da torção — *«o bordo anda ao lado da árvore»* — **não tinha gate**:
//! nenhum teste compunha um deformador com outro modificador. Duas mutações sobreviviam (apagar o
//! `ball = step_mod(...)`; tirar o `hypot(centro)` do `axis_reach`), e foi por aí que passaram dois
//! defeitos de furar a peça.
//!
//! ⚠️ **E um deles é PRÉ-EXISTENTE**: `[Taper, Radial]` media `‖∇f‖ = 37,3` desde a W18.
//!
//! # A régua
//!
//! `‖∇f‖ ≤ 1` **dentro da caixa de recorte** — a AABB da `bounding_ball`, que é a caixa a que a
//! marcha está presa. Fora dela ninguém pergunta nada, e medir lá acusa código correcto.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Unary, UnaryKind, Xform};
use ph2d_field_eval::Field;

/// ⭐⭐⭐ **Um exemplar VIVO de cada natureza** — e não o de nascimento.
///
/// ⛔ **O `Unary::born` do `Offset` e do `Taper` nasce NEUTRO** (`0,0`), de propósito: ali o zero é um
/// estado que o artista quer ter. ⚠️ Mas uma sonda que os instancia por `born` mede **o modificador
/// desligado** — e foi exactamente o que deixou a mutação da bola do `Taper` sobreviver ao primeiro
/// arnês deste ficheiro. *Um param no default mede o param desligado.*
///
/// ⚠️ O `match` é exaustivo: uma natureza nova **não compila** até alguém dizer com que valor ela se
/// mede.
fn vivo(k: UnaryKind) -> Unary {
    match k {
        UnaryKind::Shell => Unary::Shell { thickness: 0.06 },
        UnaryKind::Offset => Unary::Offset { distance: 0.05 },
        UnaryKind::Mirror => Unary::Mirror,
        UnaryKind::MirrorY => Unary::MirrorY,
        UnaryKind::MirrorZ => Unary::MirrorZ,
        UnaryKind::Array => Unary::Array {
            count: 3,
            spacing: 0.5,
            joint: ph2d_field::Joint::SHARP,
        },
        UnaryKind::Radial => Unary::Radial {
            count: 6,
            joint: ph2d_field::Joint::SHARP,
        },
        UnaryKind::Taper => Unary::Taper { slope: 0.6 },
        UnaryKind::Twist => Unary::Twist {
            turns: 0.35,
            lower: -2.0,
            upper: 2.0,
            falloff: 0.1,
        },
        UnaryKind::Bend => Unary::Bend {
            turns: 0.12,
            lower: -2.0,
            upper: 2.0,
            falloff: 0.1,
        },
    }
}

fn peca(mods: Vec<Unary>) -> FieldDoc {
    let mut n = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Box {
            half: [0.35, 0.35, 0.30],
            round: 0.0,
            chamfer: 0.0,
        }),
    );
    n.mods = mods;
    FieldDoc::new(vec![n], NodeId(0)).expect("peça")
}

fn worst_gradient_eps(doc: &FieldDoc, steps: i32, eps: f64) -> f64 {
    let reg = ph2d_field_eval::hybrid::Registry::default();
    let bola = ph2d_field_eval::bounds::bounding_ball(doc, &reg).expect("bordo");
    let (lo, hi_box) = bola.aabb();
    let f = Field::new(doc);
    let mut hi = 0.0f64;
    for i in 0..=steps {
        for j in 0..=steps {
            for k in 0..=steps {
                let p = |n: i32, e: usize| {
                    let t = f64::from(n) / f64::from(steps);
                    f64::from(lo[e]) + t * f64::from(hi_box[e] - lo[e])
                };
                let g = f.gradient_norm(p(i, 0), p(j, 1), p(k, 2), eps);
                if g.is_finite() && g > 1e-6 {
                    hi = hi.max(g);
                }
            }
        }
    }
    hi
}

/// ⭐⭐ **A leitura é a do ε MAIS FINO, e isso é uma correcção.**
///
/// ⛔ Uma diferença central que atravessa um **vinco** (o ápice da inclinação, a costura de uma
/// matriz) lê acima de `1` sem que o campo esteja errado — e a assinatura é que o número **cai com o
/// ε**. O auditor de 30/08 mediu-a no `radial`: `1,0141` a `1e-3`, `1,0002` a `1e-4`, `1,0000` a
/// `1e-5`. *Um extremo de grelha em torno de uma quina mede a grelha.*
///
/// ⚠️ **Ela não afrouxa a barra**: um operador que de facto infla lê o MESMO número em qualquer ε
/// (foi assim que `[Array, Taper]` acusou `1,5049`). O que ela tira é o artefacto.
fn worst_gradient(doc: &FieldDoc, steps: i32) -> f64 {
    worst_gradient_eps(doc, steps, 1.0e-5)
}

/// ⛔⛔ **A DÍVIDA CONTADA** — os pares que ainda atravessam, com o número medido em 2026-08-30.
///
/// ⚠️ **Ela só ENCOLHE**, e há duas metades a prová-lo: um par que deixe de estourar **reprova** (sai
/// da lista) e um que piore mais de 15 % **reprova**. *Uma catraca sem censo de obsolescência não
/// desce: ela vira licença.*
///
/// | par | mecanismo | idade |
/// |---|---|---|
/// | ~~`[Taper, Radial]`~~ | ✅ **CURADO em 2026-08-30**: `730,5 → 0,6822`. A cura era a que esta nota já prescrevia — derivar **quantas** fatias a pegada deformada exige (`stack::radial_slices`), em vez de olhar três. | era pré-existente desde a W18 |
/// | `[Twist, Bend]` | dois deformadores encadeados: o segundo lê um envelope que o primeiro já deformou | nasceu com a dobra |
const TOLERADOS: &[(&str, f64)] = &[("[Twist, Bend]", 44.6)];

/// ⭐⭐⭐ **A REPETIÇÃO RADIAL VARRIDA EM TODA A FAIXA DE `count`** — e ela existe porque o gate
/// irmão testa **uma** contagem.
///
/// # ⛔⛔ O que uma contagem só esconde
///
/// A exigência da janela de fatias **não é monótona em `count`**: com `n = 1` as contagens `5`, `6`,
/// `7`, `10` e `12` rasgam (até `3 684`) e a partir de `16` está limpo — as cópias ficam tão densas
/// que a união é quase um sólido de revolução. Um gate que medisse só `count = 6` daria verde à
/// `12`, e um que medisse só `32` daria verde a tudo. *Uma família parametrizada mede-se na faixa,
/// não num ponto.*
///
/// ⚠️ **Os deformadores também**: `Taper` a `0,6` e no máximo, e `Twist` — os três limpos com
/// `RADIAL_WINDOW = 3`.
#[test]
fn the_radial_repetition_holds_over_every_count() {
    const SLACK: f64 = 1.02;
    /// ⛔⛔ **MEDIDO E NOMEADO (2026-08-30): a DOBRA antes da radial rasga a partir de `16`**, e
    /// ⚠️ **isso NÃO é a janela de fatias** — o número é **invariante** a ela: `245,7732` com `n` de
    /// `3` a `count/2` (a janela inteira). ⇒ a causa está a montante, no divisor da dobra, que se
    /// mede contra o **envelope** da pilha — e o envelope da radial muda com `count`.
    ///
    /// *A cura é outra wave, e a primeira coisa que ela tem de saber é que alargar a janela já foi
    /// tentado e não faz nada.*
    const TOLERADAS: &[u32] = &[16, 24, 32, 48, 64];
    /// ⚠️ **A grelha do irmão (`20`) é CEGA a isto** — medido: a dobra a `count = 16` lê `0,17` a
    /// `20³` e **`245,77`** a `40³` e a `80³`. *Um extremo procurado numa grelha grossa mede a
    /// grelha.*
    const GRELHA: i32 = 40;
    let mut maus = Vec::new();
    let mut obsoletas = Vec::new();
    for count in [3u32, 4, 5, 6, 7, 8, 10, 12, 16, 24, 32, 48, 64] {
        for (nome, def) in [
            ("Taper 0,6", Unary::Taper { slope: 0.6 }),
            (
                "Twist",
                Unary::Twist {
                    turns: 0.35,
                    lower: -2.0,
                    upper: 2.0,
                    falloff: 0.1,
                },
            ),
            (
                "Bend",
                Unary::Bend {
                    turns: 0.12,
                    lower: -2.0,
                    upper: 2.0,
                    falloff: 0.1,
                },
            ),
        ] {
            let doc = peca(vec![
                def,
                Unary::Radial {
                    count,
                    joint: ph2d_field::Joint::SHARP,
                },
            ]);
            let g = worst_gradient(&doc, GRELHA);
            let tolerada = nome == "Bend" && TOLERADAS.contains(&count);
            if tolerada && g <= SLACK {
                obsoletas.push(format!("Bend × {count} já não rasga ({g:.4})"));
            } else if !tolerada && g > SLACK {
                maus.push(format!("{nome} × {count}: {g:.4}"));
            }
        }
    }
    assert!(
        maus.is_empty(),
        "a repetição radial rasga o campo nestas combinações: {maus:?}"
    );
    assert!(
        obsoletas.is_empty(),
        "APAGUE estas contagens de `TOLERADAS`: {obsoletas:?}"
    );
}

/// ⭐⭐⭐ **TODO PAR de modificadores, nas DUAS ordens** — derivado do `UnaryKind::ALL`.
///
/// ⚠️ Uma lista de pares escrita à mão seria a terceira cópia da mesma pergunta, e envelheceria no
/// modificador seguinte. Aqui um modificador novo entra em `2n` pares **de graça**.
#[test]
fn every_pair_of_modifiers_keeps_the_field_marchable() {
    const SLACK: f64 = 1.02;
    let mut pior = (0.0f64, String::new());
    let mut maus: Vec<String> = Vec::new();
    for a in UnaryKind::ALL {
        for b in UnaryKind::ALL {
            let mods = vec![vivo(a), vivo(b)];
            let nome = format!("[{a:?}, {b:?}]");
            let g = worst_gradient(&peca(mods), 20);
            if g > pior.0 {
                pior = (g, nome.clone());
            }
            match TOLERADOS.iter().find(|(p, _)| *p == nome) {
                // ⛔ **A metade que faz a catraca DESCER**: um par tolerado que já não estoura tem
                // de sair da lista. *Uma catraca sem censo de obsolescência vira licença.*
                Some((_, medido)) => {
                    assert!(
                        g > SLACK,
                        "{nome} já não atravessa ({g:.4}) — tire-o da lista de TOLERADOS"
                    );
                    assert!(
                        g <= medido * 1.15,
                        "{nome} piorou: {g:.4} contra os {medido:.1} medidos em 2026-08-30"
                    );
                }
                None if g > SLACK => maus.push(format!("{nome} {g:.4}")),
                None => {}
            }
        }
    }
    assert!(
        maus.is_empty(),
        "{} par(es) atravessam a superfície dentro da caixa de recorte, e cada um alcança-se em \
         DOIS cliques: {}",
        maus.len(),
        maus.join(" · ")
    );
    // ⛔ **O CONTROLE**: se a sonda medisse zero em todo o lado, o gate acima passaria vazio. O par
    // mais castigado tem de estar acima do trivial — os modificadores fazem alguma coisa.
    assert!(
        pior.0 > 0.2,
        "o par mais castigado mede {:.4} ({}) — a sonda não está a ver os modificadores",
        pior.0,
        pior.1
    );
}

/// ⭐⭐ **E o campo não SALTA** — a régua que apanha um rasgo, que um gradiente médio não vê.
///
/// ⚠️ O defeito de 2026-08-30 era uma **descontinuidade**: `f` saltava de `0,0035` para `0,0207`
/// entre dois pontos a `0,0005` um do outro. Um campo 1-Lipschitz não pode mudar mais do que a
/// distância andada.
#[test]
fn a_deformer_before_a_radial_does_not_tear_the_field() {
    let doc = peca(vec![vivo(UnaryKind::Twist), vivo(UnaryKind::Radial)]);
    let f = Field::new(&doc);
    const PASSO: f64 = 5.0e-4;
    let mut pior = 0.0f64;
    for i in -14..=14 {
        for k in -10..=10 {
            let (x, z) = (f64::from(i) * 0.05, f64::from(k) * 0.05);
            // Atravessa a costura `y = 0`, que é onde as fatias se encontram.
            let salto = (f.at(x, PASSO, z) - f.at(x, -PASSO, z)).abs() / (2.0 * PASSO);
            if salto.is_finite() {
                pior = pior.max(salto);
            }
        }
    }
    assert!(
        pior <= 1.05,
        "o campo muda {pior:.2}× a distância andada ao atravessar a costura das fatias — ele está \
         RASGADO, e nenhuma média o mostra"
    );
}

/// ⭐⭐⭐ **O BORDO CONTÉM A PEÇA, mesmo com o nó longe da origem** — auditoria de 2026-08-30.
///
/// ⛔ A lei do `Taper` ignorava o **centro** da bola: uma caixa em `x = 3` com declive `1,0` dava
/// bordo até `3,4664` e a peça chegava a **`3,8400`**. *«Um bordo menor corta a peça e não diz
/// nada»* — é o modo de falha que o `bounds.rs` declara impossível, e era dívida desde a W18.
#[test]
fn the_bound_contains_the_piece_even_far_from_the_origin() {
    let reg = ph2d_field_eval::hybrid::Registry::default();
    for (nome, mods) in [
        ("Taper", vec![vivo(UnaryKind::Taper)]),
        (
            "Taper+Twist",
            vec![vivo(UnaryKind::Taper), vivo(UnaryKind::Twist)],
        ),
        ("Array", vec![vivo(UnaryKind::Array)]),
    ] {
        // ⚠️ **Longe da origem de propósito** — é isso que a lei antiga não via.
        let mut n = Node::new(
            Xform::at(3.0, 0.0, 0.0),
            NodeKind::Leaf(Primitive::Box {
                half: [0.2; 3],
                round: 0.0,
                chamfer: 0.0,
            }),
        );
        n.mods = mods;
        let doc = FieldDoc::new(vec![n], NodeId(0)).expect("peça");
        let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("bordo");
        let f = Field::new(&doc);
        // Onde a peça de facto chega, varrendo bem para além do bordo declarado.
        let mut fora = 0.0f64;
        const N: i32 = 90;
        for i in 0..=N {
            for j in 0..=N {
                for k in 0..=N {
                    let p = |t: i32| f64::from(t) / f64::from(N) * 12.0 - 6.0;
                    let (x, y, z) = (p(i), p(j), p(k));
                    if f.at(x, y, z) < 0.0 {
                        let d = ((x - f64::from(bola.center[0])).powi(2)
                            + (y - f64::from(bola.center[1])).powi(2)
                            + (z - f64::from(bola.center[2])).powi(2))
                        .sqrt();
                        fora = fora.max(d);
                    }
                }
            }
        }
        assert!(
            fora <= f64::from(bola.radius) * 1.02,
            "{nome}: a peça chega a {fora:.4} do centro e o bordo diz {:.4} — a exportação corta em \
             silêncio, e o divisor da torção bebe deste número",
            bola.radius
        );
    }
}
