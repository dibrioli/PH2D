//! ⭐⭐⭐ **UM GATE POR BOTÃO** — o censo que o `CLAUDE.md` §5.0 diz que nenhum instrumento deste
//! repo faz: *«nenhum instrumento pergunta se o VALOR chega a um consumidor»*.
//!
//! # Porque é um ficheiro irmão
//!
//! O [`super`] afirma as leis que atravessam a crate (a identidade de fábrica, a porta, a forma das
//! expressões, a ponte para o dispositivo). Isto afirma **uma coisa por controlo**, e são duas
//! populações que crescem por razões diferentes: aquela cresce quando a lei muda, esta cresce
//! **sempre que nasce um botão**.
//!
//! ⛔ O corte foi forçado pelo tecto de `700` linhas — *corte por responsabilidade, nunca uma entrada
//! no `FILE_OVERAGE_OK`* (`CLAUDE.md` §5.0) — e é melhor por isso: quem acrescenta um botão sabe
//! exactamente onde escrever o gate dele.

use crate::tests::bits;
use crate::{Curvature, Point, Rim, Style, Zones};

/// ⭐⭐⭐ **CADA BOTÃO CHEGA AO BARRO** — o censo que o `CLAUDE.md` §5.0 diz que nenhum instrumento
/// deste repo faz.
///
/// Cada linha move **um** botão a partir da fábrica e exige que a saída mude. ⛔ Um botão que
/// passasse este gate a zero seria um controlo morto com a lei toda escrita à volta dele.
#[test]
fn cada_botao_move_a_saida() {
    let base = Style::default();
    let luz = [0.4, 0.3, 0.2];
    // Um ponto que acende TODOS: fora da frente (contorno), curvo (tinta), e com luminância viva.
    let p = Point {
        facing: 0.3,
        curvature: 0.7,
    };
    let referencia = base.apply(luz, p);

    let casos: [(&str, Style); 9] = [
        (
            "rim.strength",
            Style {
                rim: Rim {
                    strength: 0.5,
                    ..base.rim
                },
                ..base
            },
        ),
        (
            "rim.color",
            Style {
                rim: Rim {
                    strength: 0.5,
                    color: [1.0, 0.0, 0.0],
                    ..base.rim
                },
                ..base
            },
        ),
        (
            "rim.width",
            Style {
                rim: Rim {
                    strength: 0.5,
                    width: 12.0,
                    ..base.rim
                },
                ..base
            },
        ),
        (
            "curvature.convex",
            Style {
                curvature: Curvature {
                    convex: [1.0, 0.4, 0.4],
                    ..base.curvature
                },
                ..base
            },
        ),
        (
            "curvature.edge_sharpness",
            Style {
                curvature: Curvature {
                    convex: [1.0, 0.4, 0.4],
                    edge_sharpness: 0.2,
                    ..base.curvature
                },
                ..base
            },
        ),
        (
            "curvature.cavity_sharpness",
            Style {
                curvature: Curvature {
                    concave: [0.4, 0.4, 1.0],
                    cavity_sharpness: 0.2,
                    ..base.curvature
                },
                ..base
            },
        ),
        (
            "zones.shadow",
            Style {
                zones: Zones {
                    shadow: [0.3, 0.4, 1.0],
                    ..base.zones
                },
                ..base
            },
        ),
        (
            "zones.highlight",
            Style {
                zones: Zones {
                    highlight: [1.0, 0.8, 0.3],
                    ..base.zones
                },
                ..base
            },
        ),
        (
            "zones.pivot",
            Style {
                zones: Zones {
                    highlight: [1.0, 0.8, 0.3],
                    pivot: 8.0,
                    ..base.zones
                },
                ..base
            },
        ),
    ];
    for (nome, s) in casos {
        // ⚠️ **A cova precisa de um ponto CÔNCAVO** — o `p` acima é convexo, e um censo que medisse
        // a tinta da cova num ponto de aresta leria `0` e acusaria um botão VIVO de morto.
        let ponto = if nome.contains("cavity") {
            Point {
                curvature: -p.curvature,
                ..p
            }
        } else {
            p
        };
        assert_ne!(
            bits(s.apply(luz, ponto)),
            bits(base.apply(luz, ponto)),
            "o botão {nome} não move a saída"
        );
    }
    // ⛔⛔ **O DÉCIMO BOTÃO NÃO PODE ESTAR NESTA LISTA, e dizê-lo é metade do censo.**
    //
    // A [`Curvature::softness`] é consumida **A MONTANTE desta crate**: ela decide a DISTÂNCIA a
    // que o chamador mede a curvatura (`ph2d_field_render::Presentation::curvature_eps`), e o que
    // chega aqui é já o resultado dessa medição. ⇒ *ela não pode mover a saída de `apply`, e um
    // censo que a incluísse estaria a exigir o impossível.*
    //
    // ⚠️ **A ausência fica AFIRMADA, e não subentendida:** sem esta metade, o dia em que alguém a
    // ligue por engano a esta lei passa despercebido, e o dia em que ela deixe de ser lida lá em
    // cima também. Quem a mede é o gate da BORDA, no caminho do produto.
    let so_a_suavidade = Style {
        curvature: Curvature {
            softness: Curvature::MAX_SOFTNESS,
            ..base.curvature
        },
        ..base
    };
    assert_eq!(
        bits(so_a_suavidade.apply(luz, p)),
        bits(referencia),
        "a suavidade mexeu na lei POR PONTO: ela é da MEDIÇÃO, a montante"
    );
    // ⚠️ **O nono vive noutra porta** — ver [`Style::indirect_saturation`].
    let s = Style {
        indirect_saturation: 0.0,
        ..base
    };
    assert_ne!(
        bits(s.saturate_indirect(luz)),
        bits(luz),
        "a saturação da indirecta não move a saída"
    );
}

/// ⭐⭐⭐ **O SINAL DA CURVATURA SEPARA A ARESTA DA COVA** — a wave, num gate.
///
/// ⚠️ Sem isto a lei seria «tinge onde há curvatura», que é sujidade e não contorno pintado.
#[test]
fn a_aresta_e_a_cova_recebem_tintas_diferentes() {
    let s = Style {
        curvature: Curvature {
            convex: [1.0, 0.5, 0.5],
            concave: [0.5, 0.5, 1.0],
            edge_sharpness: 1.0,
            cavity_sharpness: 1.0,
            ..Curvature::default()
        },
        ..Style::default()
    };
    let luz = [0.5; 3];
    let aresta = s.curvature_tinted(luz, 1.0);
    let cova = s.curvature_tinted(luz, -1.0);
    let plano = s.curvature_tinted(luz, 0.0);
    assert_eq!(bits(plano), bits(luz), "o plano não recebe tinta nenhuma");
    assert!(
        aresta[0] > aresta[2],
        "a aresta puxa ao VERMELHO: {aresta:?}"
    );
    assert!(cova[2] > cova[0], "a cova puxa ao AZUL: {cova:?}");
    // ⚠️ **A saturação do peso é lei**: acima de `±1` a tinta é a mesma — senão uma quina afiada
    // continuaria a escurecer sem tecto e viraria um buraco preto.
    assert_eq!(
        bits(s.curvature_tinted(luz, 40.0)),
        bits(aresta),
        "o peso satura em +1"
    );
    assert_eq!(
        bits(s.curvature_tinted(luz, -40.0)),
        bits(cova),
        "o peso satura em −1"
    );
}

/// ⭐⭐ **O CONTORNO acende na silhueta e cala-se de FRENTE** — senão ele é uma lavagem, não um
/// contorno.
#[test]
fn o_contorno_acende_na_silhueta_e_nao_de_frente() {
    let s = Style {
        rim: Rim {
            color: [1.0; 3],
            strength: 2.0,
            width: 3.0,
        },
        ..Style::default()
    };
    let luz = [0.1; 3];
    let silhueta = s.rim_lit(luz, 0.0);
    let meio = s.rim_lit(luz, 0.5);
    let frente = s.rim_lit(luz, 1.0);
    assert!(silhueta[0] > meio[0], "a silhueta acende mais que o meio");
    assert!(meio[0] > frente[0], "o meio acende mais que a frente");
    assert_eq!(
        bits(frente),
        bits(luz),
        "de frente o contorno é a identidade AO BIT"
    );
    // ⚠️ **A largura é o que aperta a banda**, e a prova é a comparação a meio caminho.
    let largo = Style {
        rim: Rim {
            width: 1.0,
            ..s.rim
        },
        ..s
    };
    assert!(
        largo.rim_lit(luz, 0.5)[0] > meio[0],
        "um expoente menor alarga a banda"
    );
}

/// ⭐⭐ **O PIVÔ das zonas reparte no cinzento MÉDIO** — o número não é escolhido, e o gate diz qual.
#[test]
fn o_pivo_reparte_meio_a_meio_no_cinzento_medio() {
    let s = Style {
        zones: Zones {
            shadow: [0.0; 3],
            highlight: [2.0; 3],
            pivot: Zones::MIDDLE_GREY,
        },
        ..Style::default()
    };
    // Uma luz cuja LUMINÂNCIA é exactamente o pivô: cinzento neutro nesse valor.
    let cinzento = [Zones::MIDDLE_GREY; 3];
    let out = s.graded(cinzento);
    // `h = ½` ⇒ tinta `= 0 + (2 − 0)·½ = 1` ⇒ a luz atravessa.
    for k in 0..3 {
        assert!(
            (out[k] - cinzento[k]).abs() < 1e-6,
            "no pivô a grade é neutra: {out:?} contra {cinzento:?}"
        );
    }
    // ⚠️ E os dois lados vão para lados OPOSTOS — sem isto «repartir» não quer dizer nada.
    let escuro = s.graded([0.01; 3]);
    let claro = s.graded([10.0; 3]);
    assert!(escuro[0] < 0.01, "o escuro puxa à tinta das sombras");
    assert!(claro[0] > 10.0, "o claro puxa à tinta das luzes");
    // ⚠️ **E a repartição NUNCA satura** — é isso que a razão compra sobre dois limiares.
    let enorme = s.graded([1.0e6; 3]);
    assert!(
        enorme[0].is_finite() && enorme[0] > 1.0e6,
        "o HDR continua a repartir: {enorme:?}"
    );
}

/// ⭐ **A saturação da indirecta move-se nos DOIS sentidos**, e `1` é a identidade ao bit.
#[test]
fn a_saturacao_da_indirecta_lava_e_afirma() {
    let luz = [0.6, 0.2, 0.1];
    let lavada = Style {
        indirect_saturation: 0.0,
        ..Style::default()
    }
    .saturate_indirect(luz);
    assert!(
        (lavada[0] - lavada[1]).abs() < 1e-6 && (lavada[1] - lavada[2]).abs() < 1e-6,
        "saturação zero devolve CINZENTO: {lavada:?}"
    );
    let forte = Style {
        indirect_saturation: 2.0,
        ..Style::default()
    }
    .saturate_indirect(luz);
    let (dl, df) = (luz[0] - luz[2], forte[0] - forte[2]);
    assert!(df > dl, "acima de 1 a cor abre: {df} contra {dl}");
}

/// ⭐⭐⭐ **A PORTA QUE DECIDE SE ALGUÉM PAGA A AMOSTRA DE CURVATURA.**
///
/// ⚠️ A metade **negativa** é o que a torna barata, e é a que um gate ingénuo esqueceria: com as
/// tintas em branco a resposta é `false` mesmo com a nitidez alta, porque branco é a identidade.
#[test]
fn a_curvatura_so_se_le_quando_alguem_a_tinge() {
    let d = Style::default();
    assert!(!d.reads_curvature(), "a fábrica não paga a amostra");
    assert!(
        !Style {
            curvature: Curvature {
                edge_sharpness: 9.0,
                cavity_sharpness: 9.0,
                ..d.curvature
            },
            ..d
        }
        .reads_curvature(),
        "nitidez sem tinta continua a não pagar"
    );
    assert!(
        Style {
            curvature: Curvature {
                convex: [1.0, 0.9, 0.9],
                ..d.curvature
            },
            ..d
        }
        .reads_curvature(),
        "uma tinta de aresta paga"
    );
    assert!(
        Style {
            curvature: Curvature {
                concave: [0.9, 0.9, 1.0],
                ..d.curvature
            },
            ..d
        }
        .reads_curvature(),
        "uma tinta de cova paga"
    );
}

/// ⭐⭐⭐ **OS DOIS LIMIARES SÃO INDEPENDENTES** — a wave da auditoria de 2026-09-19, num gate.
///
/// # ⛔ Porque ele existe
///
/// Numa peça real as duas famílias vivem a uma ordem de grandeza uma da outra (filetes `H·R ≈ 11`–
/// `34`, covas `≈ −3`–`−5`), e enquanto o limiar era **um só** não existia posição que servisse os
/// dois: a que acende as covas satura o filete `3×`–`9×`.
///
/// **Mutação que deve sangrar:** o braço da cova voltar a ler `e.convexo.w` / `edge_sharpness`.
#[test]
fn os_dois_limiares_sao_independentes() {
    let s = |e: f32, c: f32| Style {
        curvature: Curvature {
            convex: [1.0, 0.0, 0.0],
            concave: [0.0, 0.0, 1.0],
            edge_sharpness: e,
            cavity_sharpness: c,
            ..Curvature::default()
        },
        ..Style::default()
    };
    let luz = [0.5; 3];
    // ⚠️ A curvatura da ARESTA é `10×` a da COVA — é essa desproporção que o gate existe para
    // exprimir, e é a que a cena real tem.
    let (aresta, cova) = (10.0, -1.0);

    // (1) Mexer no limiar da ARESTA não toca na cova, e vice-versa.
    let base = s(0.05, 0.5);
    let so_aresta = s(0.10, 0.5);
    let so_cova = s(0.05, 1.0);
    assert_eq!(
        bits(so_aresta.curvature_tinted(luz, cova)),
        bits(base.curvature_tinted(luz, cova)),
        "subir a nitidez da ARESTA mexeu na tinta da COVA"
    );
    assert_eq!(
        bits(so_cova.curvature_tinted(luz, aresta)),
        bits(base.curvature_tinted(luz, aresta)),
        "subir a nitidez da COVA mexeu na tinta da ARESTA"
    );

    // (2) ⭐ E o que o limiar partilhado NÃO conseguia: os dois lados a meia tinta AO MESMO TEMPO.
    //     Com um só, `1/g` serve uma família e satura ou apaga a outra.
    let meio = s(0.05, 0.5);
    let ta = meio.curvature_tinted(luz, aresta);
    let tc = meio.curvature_tinted(luz, cova);
    assert!(
        (ta[1] - luz[1]).abs() > 1e-4 && ta[1] < luz[1],
        "a aresta não está a meia tinta: {ta:?}"
    );
    assert!(
        (tc[1] - luz[1]).abs() > 1e-4 && tc[1] < luz[1],
        "a cova não está a meia tinta: {tc:?}"
    );
    // ⚠️ **O CONTROLO da própria régua:** com um limiar PARTILHADO no valor da aresta, a cova
    // recebe `10×` menos tinta — é a desproporção que a wave existe para curar, e sem esta metade
    // o gate acima ficaria verde sobre a lei antiga.
    let partilhado = s(0.05, 0.05);
    let tc_antigo = partilhado.curvature_tinted(luz, cova);
    assert!(
        (luz[1] - tc_antigo[1]) < (luz[1] - tc[1]) * 0.25,
        "o controlo não reproduz a lei antiga: {tc_antigo:?} contra {tc:?}"
    );
}

/// ⭐⭐⭐ **A ESCALA DA MEDIÇÃO tem CERCAS de RECURSO nas duas pontas, e elas não são gosto.**
///
/// ⚠️ **As duas pontas protegem coisas DIFERENTES**, e é por isso que estão as duas na porta:
/// - o **piso** protege a aritmética (abaixo do óptimo a segunda diferença cancela e devolve ruído);
/// - o **tecto** protege um CONTROLO de morrer em silêncio (acima dele as covas deixam de ser
///   côncavas e a [`Curvature::concave`] deixa de ter sujeito).
///
/// **Mutação que deve sangrar:** tirar o `clamp` do `softness` no [`Style::sanitized`].
#[test]
fn a_escala_da_medicao_e_apertada_nas_duas_pontas() {
    let com = |v: f32| {
        Style {
            curvature: Curvature {
                softness: v,
                ..Curvature::default()
            },
            ..Style::default()
        }
        .sanitized()
        .curvature
        .softness
    };
    assert!(
        (com(0.0) - Curvature::MIN_SOFTNESS).abs() < f32::EPSILON,
        "o piso não apertou: {}",
        com(0.0)
    );
    assert!(
        (com(1e9) - Curvature::MAX_SOFTNESS).abs() < f32::EPSILON,
        "o tecto não apertou: {}",
        com(1e9)
    );
    assert!(
        (com(f32::NAN) - Curvature::SOFTNESS).abs() < f32::EPSILON,
        "ilegível não voltou à fábrica"
    );
}

/// ⭐⭐⭐ **AS DUAS LEIS DA JANELA DA SUAVIDADE SÃO ERRO DE COMPILAÇÃO, e não um teste.**
///
/// ⚠️ **Um `assert!` sobre duas CONSTANTES é dobrado pelo compilador antes de correr** — o clippy
/// di-lo em voz alta (`this assertion has a constant value`), e esta casa já pagou a lição na linha
/// da escultura. ⇒ o sítio certo é um `const _`, onde a violação **não compila** em vez de reprovar
/// um teste que alguém pode filtrar.
///
/// - **o piso de população**: com as duas pontas coladas o botão não teria curso nenhum, e o gate
///   das cercas ficaria trivialmente verde;
/// - **a fábrica vive DENTRO delas**: uma omissão fora seria apertada na porta, e o número que o
///   artista vê no painel deixaria de ser o que a lei corre.
const _: () = assert!(Curvature::MAX_SOFTNESS > Curvature::MIN_SOFTNESS * 4.0);
const _: () = assert!(
    Curvature::SOFTNESS >= Curvature::MIN_SOFTNESS
        && Curvature::SOFTNESS <= Curvature::MAX_SOFTNESS
);
