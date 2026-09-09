//! ⭐⭐⭐ **AS QUATRO JUNTAS DA W145, medidas pelo caminho do PRODUTO** (pedido do Enio, 09/09).
//!
//! O oráculo das CONSTANTES é o `the_four_characters` (é lá que a calibração do `Soft` se prende).
//! Aqui estão as três perguntas que sobram, e cada uma tem um modo de falha próprio:
//!
//! | pergunta | o que morre sem ela |
//! |---|---|
//! | a junta **faz** alguma coisa? | um chip bonito e inerte — a doença que este módulo já apanhou no `round` do cone |
//! | ela cabe no **balde da marcha**? | a peça **fura**, e o sintoma é fundo no meio do sólido |
//! | o sulco **escava** e o friso **levanta**? | os dois trocados leem-se como «funciona» numa foto e como o contrário na peça |
//!
//! ⚠️ **Tudo aqui passa pelo `FieldDoc`**, nunca por uma árvore montada à mão: a sonda irmã
//! (`probe_the_other_junctions`) mede operadores crus, e um operador certo com o `blended` errado
//! deixaria essa sonda verde e o produto partido.

use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_eval::Field;

const R: f32 = 0.25;
/// A barra da marcha: o `√2` que o arredondamento exacto desta casa já paga
/// (`ph2d_field_eval::gradient_bound`).
const BALDE: f64 = std::f64::consts::SQRT_2;

/// Um semiespaço `{n · p ≤ 0}` com a normal exterior a `graus` do eixo `+x`, feito de uma caixa
/// grande — a mesma bancada do `the_four_characters`, com um ângulo dentro.
fn parede(graus: f32) -> Node {
    const H: f32 = 2.0;
    let r = graus.to_radians();
    let (c, s) = (r.cos(), r.sin());
    let meio = (r * 0.5).sin();
    Node::new(
        Xform {
            translation: [-H * c, -H * s, 0.0],
            rotation: [0.0, 0.0, meio, (r * 0.5).cos()],
            scale: 1.0,
        },
        NodeKind::Leaf(Primitive::Box {
            half: [H, H, H],
            round: 0.0,
            chamfer: 0.0,
        }),
    )
}

/// ⭐ **A CUNHA de `graus` de abertura**, com a junta pedida.
///
/// As normais exteriores fazem `n_a · n_b = −cos(graus)`, logo `graus = 90` é o canto ortogonal em
/// que toda a bancada deste módulo foi definida.
fn cunha(graus: f32, blend: Blend) -> FieldDoc {
    let meia = graus * 0.5;
    FieldDoc::new(
        vec![
            parede(90.0 - meia),
            parede(90.0 + meia),
            Node::new(
                Xform::IDENTITY,
                NodeKind::Combine {
                    op: Op::Union(blend),
                    children: vec![NodeId(0), NodeId(1)],
                },
            ),
        ],
        NodeId(2),
    )
    .expect("a cunha")
}

fn pior_gradiente(doc: &FieldDoc) -> f64 {
    let f = Field::new(doc);
    let mut pior = 0.0f64;
    const N: usize = 33;
    for i in 0..N {
        for j in 0..N {
            let c = |t: usize| -0.6 + 1.2 * (t as f64 + 0.5) / N as f64;
            let g = f.gradient_norm(c(i), c(j), 0.0, 1.0e-4);
            if g.is_finite() {
                pior = pior.max(g);
            }
        }
    }
    pior
}

/// As juntas que a W145 trouxe, todas ao mesmo tamanho.
fn novas() -> Vec<(&'static str, Blend)> {
    vec![
        ("Soft", Blend::Soft { radius: R }),
        ("Bead", Blend::Bead { radius: R }),
        (
            "Groove",
            Blend::Groove {
                radius: R,
                width: R * Blend::SEAM_WIDTH_RATIO,
            },
        ),
        (
            "Ridge",
            Blend::Ridge {
                radius: R,
                width: R * Blend::SEAM_WIDTH_RATIO,
            },
        ),
        (
            "Bevel",
            Blend::Bevel {
                radius: R,
                bias: 2.5,
            },
        ),
    ]
}

/// ⭐⭐⭐ **NENHUMA DELAS É INERTE** — a lei dos controlos mortos, aplicada na hora em que nascem.
///
/// ⚠️ **A régua é a SUPERFÍCIE e não o volume:** um sulco tira matéria e um friso põe, e um teste
/// que somasse volume assinado podia ler zero num par que se cancela. Aqui conta-se em quantos
/// pontos da grelha o campo **mudou de lado** contra a junta viva — o que o olho de facto vê.
///
/// ⛔ Este módulo já pagou esta lição: o `round` do cone e do prisma foi **inerte durante meses**
/// (`+0,0 %` de volume, campo bit a bit igual) com o slider a mexer-se na tela.
#[test]
fn every_new_character_changes_the_piece() {
    let viva = Field::new(&cunha(90.0, Blend::Sharp));
    for (nome, b) in novas() {
        let f = Field::new(&cunha(90.0, b));
        let (mut mudou, mut total) = (0usize, 0usize);
        for i in 0..40 {
            for j in 0..40 {
                let p = |t: usize| -0.5 + 1.0 * (t as f64 + 0.5) / 40.0;
                let (x, y) = (p(i), p(j));
                total += 1;
                if (f.at(x, y, 0.0) < 0.0) != (viva.at(x, y, 0.0) < 0.0) {
                    mudou += 1;
                }
            }
        }
        let pct = 100.0 * mudou as f64 / total as f64;
        println!("  {nome:>8}: {pct:5.2} % da secção mudou de lado contra a junta viva");
        assert!(
            mudou > 0,
            "o caracter {nome} nao muda um unico ponto da peca contra a aresta viva — ele e' um \
             chip que nao faz nada, que e' a doenca que o `round` do cone teve durante meses"
        );
    }
}

/// ⭐⭐⭐ **ELAS CABEM NO BALDE DA MARCHA, e o ângulo é parte da pergunta** — porque o `s = (a−b)/√2`
/// das decorações só é uma distância exacta com os dois gradientes perpendiculares.
///
/// ⛔ **Um balde a menos FURA a peça** (`ph2d_field_eval::step::gradient_bound` anda o valor do
/// campo), e o defeito não aparece a 90°, que é onde toda a bancada deste módulo vive. É por isso
/// que este gate varre de `30°` a `150°`.
#[test]
fn the_seam_characters_stay_inside_the_march_bucket() {
    println!("  carácter |     30° |     60° |     90° |    120° |    150°");
    for (nome, b) in novas() {
        let mut col = Vec::new();
        for g in [30.0, 60.0, 90.0, 120.0, 150.0] {
            col.push(pior_gradiente(&cunha(g, b)));
        }
        println!(
            "  {nome:>8} | {:7.4} | {:7.4} | {:7.4} | {:7.4} | {:7.4}",
            col[0], col[1], col[2], col[3], col[4]
        );
        let pior = col.iter().copied().fold(0.0f64, f64::max);
        assert!(
            pior <= BALDE + 1.0e-3,
            "o caracter {nome} le ‖∇f‖ = {pior:.4}, acima do balde de {BALDE:.4} que o `inflates` \
             lhe da' — a marcha anda mais do que a distancia e a peca FURA"
        );
    }
}

/// ⭐⭐ **O SULCO escava e o FRISO levanta** — os dois sentidos, cada um no ponto onde só ele fala.
///
/// ⚠️ **Trocados, os dois passam em qualquer teste de «mudou alguma coisa»**: são a mesma fórmula
/// com o `min` e o `max` invertidos, e é exactamente por isso que a direcção precisa de gate.
#[test]
fn the_groove_carves_and_the_ridge_protrudes() {
    let t = f64::from(R) * 0.3;
    // Um ponto DENTRO do sólido, junto da costura: o sulco tem de o pôr fora.
    let sulco = Field::new(&cunha(
        90.0,
        Blend::Groove {
            radius: R,
            width: R * Blend::SEAM_WIDTH_RATIO,
        },
    ));
    assert!(
        sulco.at(-t, -t, 0.0) > 0.0,
        "o sulco nao escavou: um ponto a {t:.3} dentro da costura continua em materia (leu {:.4})",
        sulco.at(-t, -t, 0.0)
    );
    // Um ponto FORA, no vão do canto: o friso e o cordão têm de o pôr dentro.
    for (nome, b) in [
        (
            "friso",
            Blend::Ridge {
                radius: R,
                width: R * Blend::SEAM_WIDTH_RATIO,
            },
        ),
        ("cordão", Blend::Bead { radius: R }),
    ] {
        let f = Field::new(&cunha(90.0, b));
        assert!(
            f.at(t, t, 0.0) < 0.0,
            "o {nome} nao acrescentou materia: o vao do canto a {t:.3} continua vazio (leu {:.4})",
            f.at(t, t, 0.0)
        );
    }
}

/// ⭐⭐⭐ **UM `Bevel` SIMÉTRICO É O CHANFRO, PONTO A PONTO** — e é isso que permite ao chanfro
/// desigual não ter chip próprio.
///
/// ⚠️ **A igualdade tem de ser EXACTA e não aproximada:** o [`Blend::with_second`] devolve um
/// `Chamfer` quando o desequilíbrio volta a `1`, e se as duas leis divergissem o artista veria a
/// peça saltar ao passar por esse ponto do slider.
#[test]
fn a_bevel_with_bias_one_is_the_chamfer() {
    let chanfro = Field::new(&cunha(90.0, Blend::Chamfer { radius: R }));
    let bevel = Field::new(&cunha(
        90.0,
        Blend::Bevel {
            radius: R,
            bias: 1.0,
        },
    ));
    let mut pior = 0.0f64;
    for i in 0..60 {
        for j in 0..60 {
            let p = |t: usize| -0.6 + 1.2 * (t as f64 + 0.5) / 60.0;
            let (x, y) = (p(i), p(j));
            pior = pior.max((chanfro.at(x, y, 0.0) - bevel.at(x, y, 0.0)).abs());
        }
    }
    println!("  |Bevel(1,0) − Chamfer| pior = {pior:.3e}");
    assert!(
        pior == 0.0,
        "um chanfro de desequilibrio 1,0 difere do chanfro simetrico em {pior:.3e} — a democao do \
         `with_second` faz a peca saltar ao passar por esse ponto do slider"
    );
}

/// ⭐⭐ **E o desequilíbrio CORTA MAIS de um lado** — sem isto o segundo número podia estar ligado a
/// nada e o gate acima continuaria verde.
///
/// ⛔⛔ **A 1.ª redacção desta régua mediu os EIXOS e leu `1,0000` contra `0,0000`.** A
/// [`cunha`] põe as duas paredes a `±45°` do `+y`, e não em `x = 0` e `y = 0` — *a bancada de todo o
/// resto deste módulo é ortogonal, e eu escrevi a régua como se esta também fosse*. A cura é medir
/// ao longo da **tangente de cada parede**, que se deriva do ângulo dela.
#[test]
fn the_bevel_bias_cuts_deeper_on_one_side() {
    const BIAS: f32 = 3.0;
    let f = Field::new(&cunha(
        90.0,
        Blend::Bevel {
            radius: R,
            bias: BIAS,
        },
    ));
    // ⭐ **O recuo ao longo de UMA parede**: caminha-se pela superfície dela a partir da quina, e o
    // recuo é o ponto a partir do qual a parede volta a ser a superfície. Antes dele a junta
    // acrescentou matéria e o plano da parede já está **dentro** do sólido.
    let recuo = |graus: f32| -> f64 {
        let r = f64::from(graus.to_radians());
        // A tangente que entra no vão do canto — o vão abre para `+y`, logo é a que sobe.
        let (tx, ty) = (-r.sin(), r.cos());
        let (tx, ty) = if ty < 0.0 { (-tx, -ty) } else { (tx, ty) };
        let (mut lo, mut hi) = (0.0f64, 1.5f64);
        for _ in 0..60 {
            let m = f64::midpoint(lo, hi);
            if f.at(m * tx, m * ty, 0.0) < -1.0e-4 {
                lo = m;
            } else {
                hi = m;
            }
        }
        f64::midpoint(lo, hi)
    };
    let (a, b) = (recuo(45.0), recuo(135.0));
    let razao = b.max(a) / b.min(a);
    println!("  recuo numa face = {a:.4} · na outra = {b:.4} · razão = {razao:.3} (bias {BIAS})");
    assert!(
        razao > 1.5,
        "o desequilibrio de {BIAS} deu recuos de {a:.4} e {b:.4} (razao {razao:.3}) — o segundo \
         numero do chanfro nao esta a chegar a geometria nenhuma"
    );
    // ⛔ **O CONTROLO**: a mesma régua sobre o chanfro SIMÉTRICO tem de ler `1,000`. Sem ele, uma
    // régua que sempre devolvesse dois números diferentes passaria este gate sem medir nada.
    let simetrico = Field::new(&cunha(90.0, Blend::Chamfer { radius: R }));
    let recuo_sim = |graus: f32| -> f64 {
        let r = f64::from(graus.to_radians());
        let (tx, ty) = (-r.sin(), r.cos());
        let (tx, ty) = if ty < 0.0 { (-tx, -ty) } else { (tx, ty) };
        let (mut lo, mut hi) = (0.0f64, 1.5f64);
        for _ in 0..60 {
            let m = f64::midpoint(lo, hi);
            if simetrico.at(m * tx, m * ty, 0.0) < -1.0e-4 {
                lo = m;
            } else {
                hi = m;
            }
        }
        f64::midpoint(lo, hi)
    };
    let (ca, cb) = (recuo_sim(45.0), recuo_sim(135.0));
    println!("  CONTROLO, chanfro simétrico: {ca:.4} e {cb:.4}");
    assert!(
        (ca - cb).abs() < 1.0e-3,
        "a regua acusa desequilibrio ({ca:.4} contra {cb:.4}) num chanfro SIMETRICO — ela mede o \
         sitio errado, e o gate acima estaria verde sobre nada"
    );
}

/// ⭐⭐⭐ **UM SULCO ESCAVA NAS TRÊS OPERAÇÕES, e um friso levanta nas três** (W145).
///
/// # ⛔⛔ O defeito que ele existe para impedir, e que a construção quase shipou
///
/// O dual de um sulco **é um friso**: `¬groove(¬a, ¬b)` dá, termo a termo, a fórmula da nervura. ⇒
/// pela lei de De Morgan que o resto desta crate honra, um chip **Groove** numa **subtração**
/// levantaria uma nervura à volta do furo — *um controlo que faz o contrário do que o rótulo diz*,
/// que é a família de defeito que esta casa mede desde 2026-08-30.
///
/// ⚠️ **A régua conta as duas direcções separadamente**, e é isso que a torna um gate e não uma
/// verificação de que «mudou alguma coisa»: um sulco só pode **tirar**, um friso e um cordão só
/// podem **pôr**. Trocados, os números aparecem na coluna errada.
#[test]
fn a_groove_carves_and_a_ridge_lifts_in_all_three_operations() {
    let caixa = |x: f32, y: f32| {
        Node::new(
            Xform {
                translation: [x, y, 0.0],
                ..Xform::IDENTITY
            },
            NodeKind::Leaf(Primitive::Box {
                half: [0.3, 0.3, 0.3],
                round: 0.0,
                chamfer: 0.0,
            }),
        )
    };
    // ⚠️ **Deslocadas nos DOIS eixos de propósito**: com um deslocamento só, as faces que se
    // encontram na intersecção são **paralelas** e não há aresta nenhuma para decorar.
    let peca = |op: fn(Blend) -> Op, b: Blend| {
        FieldDoc::new(
            vec![
                caixa(-0.1, -0.1),
                caixa(0.1, 0.1),
                Node::new(
                    Xform::IDENTITY,
                    NodeKind::Combine {
                        op: op(b),
                        children: vec![NodeId(0), NodeId(1)],
                    },
                ),
            ],
            NodeId(2),
        )
        .expect("o par")
    };
    let conta = |op: fn(Blend) -> Op, b: Blend| -> (usize, usize) {
        let viva = Field::new(&peca(op, Blend::Sharp));
        let f = Field::new(&peca(op, b));
        let (mut tirou, mut pos) = (0usize, 0usize);
        for i in 0..34 {
            for j in 0..34 {
                for k in 0..34 {
                    let c = |t: usize| -0.55 + 1.1 * (t as f64 + 0.5) / 34.0;
                    let (x, y, z) = (c(i), c(j), c(k));
                    let (antes, agora) = (viva.at(x, y, z) < 0.0, f.at(x, y, z) < 0.0);
                    if antes && !agora {
                        tirou += 1;
                    }
                    if !antes && agora {
                        pos += 1;
                    }
                }
            }
        }
        (tirou, pos)
    };
    let largura = R * Blend::SEAM_WIDTH_RATIO;
    for (verbo, op) in [
        ("União", Op::Union as fn(Blend) -> Op),
        ("Intersecção", Op::Intersection as fn(Blend) -> Op),
        ("Subtração", Op::Difference as fn(Blend) -> Op),
    ] {
        let (t_sulco, p_sulco) = conta(
            op,
            Blend::Groove {
                radius: R,
                width: largura,
            },
        );
        let (t_friso, p_friso) = conta(
            op,
            Blend::Ridge {
                radius: R,
                width: largura,
            },
        );
        let (t_cordao, p_cordao) = conta(op, Blend::Bead { radius: R });
        println!(
            "  {verbo:>12}: sulco −{t_sulco}/+{p_sulco} · friso −{t_friso}/+{p_friso} · cordão \
             −{t_cordao}/+{p_cordao}"
        );
        assert!(
            t_sulco > 0 && p_sulco == 0,
            "{verbo}: o sulco tirou {t_sulco} e PÔS {p_sulco} — ele virou friso nesta operação"
        );
        assert!(
            p_friso > 0 && t_friso == 0,
            "{verbo}: o friso pôs {p_friso} e TIROU {t_friso} — ele virou sulco nesta operação"
        );
        assert!(
            p_cordao > 0 && t_cordao == 0,
            "{verbo}: o cordão pôs {p_cordao} e TIROU {t_cordao} — ele virou canal nesta operação"
        );
    }
}
