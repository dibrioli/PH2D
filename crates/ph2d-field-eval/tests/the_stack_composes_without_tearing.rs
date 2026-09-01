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

            axis: ph2d_field::mods::ARRAY_AXIS,
        },
        UnaryKind::Radial => Unary::Radial {
            count: 6,
            joint: ph2d_field::Joint::SHARP,

            axis: ph2d_field::mods::RADIAL_AXIS,
        },
        UnaryKind::Taper => Unary::Taper {
            slope: 0.6,
            axis: ph2d_field::mods::TAPER_AXIS,
        },
        UnaryKind::Twist => Unary::Twist {
            turns: 0.35,
            lower: -2.0,
            upper: 2.0,
            falloff: 0.1,

            axis: ph2d_field::mods::TWIST_AXIS,
        },
        UnaryKind::Bend => Unary::Bend {
            turns: 0.12,
            lower: -2.0,
            upper: 2.0,
            falloff: 0.1,

            axis: ph2d_field::mods::BEND_AXIS,
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
/// | ~~`[Taper, Radial]`~~ | ✅ **CURADO**: `730,5 → 0,6822`, pela janela de fatias medida (`stack::RADIAL_WINDOW`) | era pré-existente desde a W18 |
/// | ~~`[Twist, Bend]`~~ | ✅ **CURADO**: `44,6 → 0,2077`, e a causa era o **corte de ramo do `atan2`** dentro da caixa de recorte (ver `stack::bend`) | nasceu com a dobra |
///
/// ⭐⭐⭐ **A LISTA ESTÁ VAZIA** — e ficar vazia é a única coisa que uma catraca pode fazer de bom.
///
/// # ⛔⛔⛔ A DÍVIDA INTEIRA DA DOBRA ERA ARITMÉTICA, e as DUAS curas anteriores erraram o alvo
///
/// Este bloco listou, em dias sucessivos, três curas para o mesmo número — todas sobre a
/// **curvatura**. A causa nunca esteve lá:
///
/// 1. a caixa de recorte alcança `x = 1,4036` com `ρ = 1,3263`, logo `a = ρ − x` fica **negativo**;
/// 2. ali `atan2(b, a)` salta de `+π` para `−π` ao cruzar `b = 0`;
/// 3. a banda clampa os dois lados do salto em **bordas opostas**, e o campo rasga.
///
/// ⇒ empurrar `a` para a parede da peça (o `piso` que a função já declarava, agora aplicado ao
/// **ponto** e não só ao raio) leva `[Twist, Bend]` a **`0,2077`**, `[Bend]` sozinha a **`0,8130`**
/// e `[Bend, Radial]` de `245,77` a **`0,28`–`0,49`** em toda a faixa de contagens — **sem tocar na
/// curvatura**, logo sem tocar no que o artista vê.
///
/// ⛔⛔ **E foi por isso que a cura de 2026-08-30 foi REJEITADA PELO DONO.** Apertar a parede
/// (`bend_curvature(turns, final_ball)`) baixava os mesmos números **e fazia a peça deixar de
/// dobrar**: num bloco, `0,3`, `0,6` e `1,0` voltas passavam a dar a mesma coisa. Report do Enio:
/// *«VC danificou o Bend que funcionava antes das últimas mudanças»*.
///
/// ⭐⭐⭐ **A imagem tinha dito isso, com número, antes de eu perceber porquê**: o gate
/// `ph2d_field_render::the_bend_draws_what_an_honest_march_draws` media **`0` de
/// `1 678`/`1 672`/`4 274`/`4 274` pixels** fora de `12°`. *Um `‖∇f‖` alto sem um pixel em desacordo
/// não é a peça a rasgar — é a sonda a atravessar uma descontinuidade que nenhum raio visita.*
///
/// ⚠️ **A lição de método:** quando um gate de razão acusa e a imagem não confirma, procure a
/// **singularidade da fórmula** dentro do domínio de amostragem antes de mexer no modelo.
const TOLERADOS: &[(&str, f64)] = &[];

/// ⭐⭐⭐ **UM MODIFICADOR SOZINHO, EM TODA A FAIXA DO PARÂMETRO DELE** — o ponto cego que faltava.
///
/// # ⛔⛔ Nenhum gate deste ficheiro media UM modificador
///
/// O irmão varre **pares**, e por isso `[Bend]` sozinha nunca foi medida. Ela **rasgava**:
/// `‖∇f‖ = 1,7210` a `0,12` voltas, `1,2357` a `0,50` — um clique do nascimento.
///
/// ⚠️ **E o par escondia-o**: `[Bend, Bend]` tem um envelope maior, logo um divisor maior, logo
/// mede-se **segura**. *Compor duas coisas pode CURAR o defeito de uma delas, e aí a sonda de pares
/// dá verde ao que a de uma só reprovaria.*
///
/// ⚠️ A faixa é do parâmetro de cada natureza, e o `match` é **exaustivo**: uma natureza nova não
/// compila até alguém dizer em que faixa ela se mede.
#[test]
fn every_modifier_alone_keeps_the_field_marchable() {
    const SLACK: f64 = 1.02;
    let mut maus = Vec::new();
    let mut pior_dobra = 0.0f64;
    let mut medidos = 0;
    for k in UnaryKind::ALL {
        let faixa: Vec<Unary> = match k {
            UnaryKind::Shell => [0.02f32, 0.06, 0.2, 0.5]
                .iter()
                .map(|&t| Unary::Shell { thickness: t })
                .collect(),
            UnaryKind::Offset => [0.02f32, 0.05, 0.2, 0.5]
                .iter()
                .map(|&d| Unary::Offset { distance: d })
                .collect(),
            UnaryKind::Mirror => vec![Unary::Mirror],
            UnaryKind::MirrorY => vec![Unary::MirrorY],
            UnaryKind::MirrorZ => vec![Unary::MirrorZ],
            UnaryKind::Array => [2u32, 3, 8, 64]
                .iter()
                .map(|&count| Unary::Array {
                    count,
                    spacing: 0.5,
                    joint: ph2d_field::Joint::SHARP,

                    axis: ph2d_field::mods::ARRAY_AXIS,
                })
                .collect(),
            UnaryKind::Radial => [2u32, 6, 12, 64]
                .iter()
                .map(|&count| Unary::Radial {
                    count,
                    joint: ph2d_field::Joint::SHARP,

                    axis: ph2d_field::mods::RADIAL_AXIS,
                })
                .collect(),
            UnaryKind::Taper => [0.1f32, 0.6, 1.2, 2.0]
                .iter()
                .map(|&slope| Unary::Taper {
                    slope,
                    axis: ph2d_field::mods::TAPER_AXIS,
                })
                .collect(),
            UnaryKind::Twist => [0.05f32, 0.35, 1.0, 2.0]
                .iter()
                .map(|&turns| Unary::Twist {
                    turns,
                    lower: -2.0,
                    upper: 2.0,
                    falloff: 0.1,

                    axis: ph2d_field::mods::TWIST_AXIS,
                })
                .collect(),
            UnaryKind::Bend => [0.05f32, 0.12, 0.25, 0.5, 1.0]
                .iter()
                .map(|&turns| Unary::Bend {
                    turns,
                    lower: -2.0,
                    upper: 2.0,
                    falloff: 0.1,

                    axis: ph2d_field::mods::BEND_AXIS,
                })
                .collect(),
        };
        for m in faixa {
            medidos += 1;
            let g = worst_gradient(&peca(vec![m]), 32);
            // ⭐ **A dobra deixou de ser excepção em 2026-08-31** — ver o `TOLERADOS` deste
            // ficheiro. Ela mede-se hoje pela MESMA barra de todos os outros.
            if matches!(m, Unary::Bend { .. }) {
                pior_dobra = pior_dobra.max(g);
            }
            if g > SLACK {
                maus.push(format!("{m:?} -> {g:.4}"));
            }
        }
    }
    assert!(
        medidos >= 30,
        "só {medidos} exemplares — a lista derivada de `UnaryKind::ALL` partiu-se"
    );
    assert!(
        maus.is_empty(),
        "estes modificadores rasgam o campo SOZINHOS, a um clique do nascimento: {maus:?}"
    );
    // ⛔ **O CONTROLE da dobra**, e ele fica no sítio onde a tolerância dela estava: a barra acima
    // só quer dizer alguma coisa se a sonda de facto vir o modificador. Uma dobra que medisse `0`
    // em toda a faixa passaria a barra **por não estar a ser aplicada**.
    assert!(
        pior_dobra > 0.2,
        "a dobra mede {pior_dobra:.4} em toda a faixa — a sonda não a está a aplicar"
    );
}

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
    /// ⭐⭐⭐ **VAZIA desde 2026-08-31** — `Bend × {16..64}` lia `245,7732` e lê hoje `0,28`–`0,49`.
    ///
    /// ⚠️ **Duas curas foram tentadas contra este número e as duas erraram o alvo**: alargar a
    /// janela de fatias (o `245,7732` era **invariante** a ela, de `n = 3` até `count/2`) e encolher
    /// a curvatura da dobra (funcionava, e **o dono rejeitou** — a peça deixava de dobrar). A causa
    /// era o **corte de ramo do `atan2`** dentro da caixa de recorte, que a repetição radial só
    /// tornava mais fácil de encontrar. Ver o `TOLERADOS` deste ficheiro.
    const TOLERADAS: &[u32] = &[];
    /// ⚠️ **A grelha do irmão (`20`) é CEGA a isto** — medido: a dobra a `count = 16` lê `0,17` a
    /// `20³` e **`245,77`** a `40³` e a `80³`. *Um extremo procurado numa grelha grossa mede a
    /// grelha.*
    const GRELHA: i32 = 40;
    let mut maus = Vec::new();
    let mut obsoletas = Vec::new();
    for count in [3u32, 4, 5, 6, 7, 8, 10, 12, 16, 24, 32, 48, 64] {
        for (nome, def) in [
            (
                "Taper 0,6",
                Unary::Taper {
                    slope: 0.6,
                    axis: ph2d_field::mods::TAPER_AXIS,
                },
            ),
            (
                "Twist",
                Unary::Twist {
                    turns: 0.35,
                    lower: -2.0,
                    upper: 2.0,
                    falloff: 0.1,

                    axis: ph2d_field::mods::TWIST_AXIS,
                },
            ),
            (
                "Bend",
                Unary::Bend {
                    turns: 0.12,
                    lower: -2.0,
                    upper: 2.0,
                    falloff: 0.1,

                    axis: ph2d_field::mods::BEND_AXIS,
                },
            ),
        ] {
            let doc = peca(vec![
                def,
                Unary::Radial {
                    count,
                    joint: ph2d_field::Joint::SHARP,

                    axis: ph2d_field::mods::RADIAL_AXIS,
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
            // ⛔⛔ **A grelha era `20`, e o irmão logo abaixo declara-a CEGA** (auditoria de
            // 2026-08-30): `[Bend, Twist]` lia `0,5630` a `20³` e `1,3448` a `40³`. *Um gate que
            // usa a grelha que o vizinho chama de cega mede a grelha.* Hoje os 100 pares estão a
            // `0` em ambas, e a barra corre na que vê.
            let g = worst_gradient(&peca(mods), 40);
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
