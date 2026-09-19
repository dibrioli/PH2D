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

use super::{Curvature, Point, Rim, Style, Zones, bits};

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

    let casos: [(&str, Style); 8] = [
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
            "curvature.sharpness",
            Style {
                curvature: Curvature {
                    convex: [1.0, 0.4, 0.4],
                    sharpness: 0.2,
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
        assert_ne!(
            bits(s.apply(luz, p)),
            bits(referencia),
            "o botão {nome} não move a saída"
        );
    }
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
            sharpness: 1.0,
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
                sharpness: 9.0,
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
