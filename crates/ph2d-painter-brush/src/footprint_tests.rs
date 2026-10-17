//! Os portões de [`super`] (a pegada): o contorno, a curvatura, a quina e a caixa — num ficheiro irmão
//! para o `footprint.rs` caber no tecto de LOC (prompt 44).

#[cfg(test)]
mod outline_tests {
    use super::super::*;

    /// ⭐⭐⭐ **O CONTORNO É A CURVA DE NÍVEL DO AMOSTRADOR** — `falloff_t` da fronteira é `1`.
    ///
    /// ⛔ É esta a amarra que impede o anel do cursor de desenhar uma elipse que a tinta não pinta:
    /// o anel percorre [`FootprintDeform::outline_at`] e o motor lê [`FootprintDeform::falloff_t`],
    /// e as duas só podem discordar se esta asserção cair.
    ///
    /// ⚠️ O corpus varre achatamento **e** ângulo: com `flatten = 0` a elipse é um círculo e
    /// qualquer contorno passa — *um corpus redondo não mede a forma*.
    #[test]
    fn the_outline_is_the_sampler_level_set() {
        let mut casos = 0;
        for flatten in [0.0_f32, 0.2, 0.5, DAB_FLATTEN_MAX] {
            for angle in [0_u16, 17, 90, 233] {
                for curve in CURVAS {
                    for quina in QUINAS {
                        let fp = FootprintDeform::new(flatten, angle)
                            .with_curve(curve)
                            .with_quina(quina);
                        for k in 0..64 {
                            let p = fp.outline_at(k as f32 / 64.0);
                            let t = fp.falloff_t(p[0], p[1]);
                            casos += 1;
                            assert!(
                                (t - 1.0).abs() < 1e-4,
                                "com flatten {flatten}, ângulo {angle}°, quina {quina} e curvatura \
                                 {curve:?}, o contorno em {k}/64 tem falloff {t} — ele não é a \
                                 fronteira que o amostrador usa"
                            );
                        }
                    }
                }
            }
        }
        assert_eq!(
            casos,
            4 * 4 * CURVAS.len() * QUINAS.len() * 64,
            "o corpus mudou de tamanho"
        );
    }

    /// As quinas do corpus: o disco (o caminho de sempre), uma ponta arredondada e o quadrado.
    const QUINAS: [f32; 3] = [0.0, 0.6, 1.0];

    /// ⭐ **A CAIXA CONTÉM A QUINA** (prompt 44): o motor percorre a caixa de [`FootprintDeform::extent`];
    /// um canto da ponta reta fora dela é tinta cortada. Varre-se o contorno exacto de cada pegada.
    #[test]
    fn the_box_contains_the_square_tip() {
        for flatten in [0.0_f32, 0.2, 0.5, DAB_FLATTEN_MAX] {
            for angle in [0_u16, 17, 45, 90, 233] {
                for quina in QUINAS {
                    let fp = FootprintDeform::new(flatten, angle).with_quina(quina);
                    let e = fp.extent();
                    let mut maior = [0.0_f32; 2];
                    for k in 0..720 {
                        let p = fp.outline_at(k as f32 / 720.0);
                        maior = [maior[0].max(p[0].abs()), maior[1].max(p[1].abs())];
                    }
                    for i in 0..2 {
                        assert!(
                            maior[i] <= e[i] * 1.000_1,
                            "flatten {flatten}, ângulo {angle}°, quina {quina}: o contorno chega a \
                             {maior:?} e a caixa é {e:?}"
                        );
                    }
                    if quina == 1.0 {
                        assert!(
                            (maior[0] - e[0]).abs() < 2e-3 && (maior[1] - e[1]).abs() < 2e-3,
                            "o quadrado tem a caixa EXACTA ({maior:?} contra {e:?})"
                        );
                    }
                }
            }
        }
    }

    /// ⭐ **A QUINA 0 É A PEGADA DE SEMPRE, AO BIT**, e a quina 1 é o QUADRADO: o canto do referencial
    /// `(0,9; 0,9)` fica dentro da ponta reta e fora da redonda.
    #[test]
    fn quina_zero_is_the_disc_and_one_is_the_square() {
        for flatten in [0.0_f32, 0.5] {
            for angle in [0_u16, 17, 233] {
                let nu = FootprintDeform::new(flatten, angle);
                assert_eq!(nu, nu.with_quina(0.0));
                for k in 0..32 {
                    let t = k as f32 / 32.0;
                    let p = [(t * 6.0).cos() * 0.9, (t * 6.0).sin() * 0.9];
                    assert_eq!(
                        nu.falloff_t(p[0], p[1]).to_bits(),
                        nu.with_quina(0.0).falloff_t(p[0], p[1]).to_bits()
                    );
                }
            }
        }
        let quadrado = FootprintDeform::identity().with_quina(1.0);
        assert!(quadrado.falloff_t(0.9, 0.9) < 1.0);
        assert!(FootprintDeform::identity().falloff_t(0.9, 0.9) > 1.0);
        assert!(
            (quadrado.falloff_t(1.0, 0.3) - 1.0).abs() < 1e-6,
            "o lado do quadrado é RECTO"
        );
    }

    /// O corpus de curvaturas: a plana (o caminho de sempre) e três dobras reais — uma quadrática
    /// pura, uma cúbica pura e uma mista. ⚠️ **Sem a plana lá dentro estes gates deixariam de medir
    /// a inércia; sem as outras três eles passariam sobre uma pegada que ignora a curvatura.**
    const CURVAS: [FootprintCurve; 4] = [
        FootprintCurve::flat(),
        FootprintCurve::from_rows([[0.25, 0.0, -0.15, 0.0, 0.0, 0.0, 0.0], [0.0; 7]]),
        FootprintCurve::from_rows([[0.0; 7], [0.0, 0.0, 0.0, 0.18, -0.09, 0.0, 0.12]]),
        FootprintCurve::from_rows([
            [0.11, -0.07, 0.05, 0.04, 0.0, -0.03, 0.0],
            [-0.06, 0.13, 0.0, 0.0, 0.07, 0.0, -0.05],
        ]),
    ];

    /// ⭐⭐⭐ **PLANA ⇒ O CAMINHO DE SEMPRE, AO BIT** — a metade que diz que esta wave não tocou em
    /// nenhuma pincelada sobre arte não deformada.
    ///
    /// ⛔ **E ela é um `if`, não uma esperança:** somar sete monómios a zero devolve o mesmo número
    /// *quase* sempre — `x + 0.0` é exacto em IEEE, mas a ORDEM da soma e o `f32` do produto
    /// intermédio não são coisas que um gate deva confiar. O atalho existe e este gate prova-o.
    #[test]
    fn a_flat_curve_is_the_footprint_that_shipped_bit_for_bit() {
        for flatten in [0.0_f32, 0.2, 0.5, DAB_FLATTEN_MAX] {
            for angle in [0_u16, 17, 90, 233] {
                let nu = FootprintDeform::new(flatten, angle);
                let com = nu.with_curve(FootprintCurve::flat());
                assert_eq!(nu, com, "pôr a curvatura PLANA mudou a pegada");
                for k in 0..32 {
                    let t = k as f32 / 32.0;
                    let p = [(t * 6.0).cos() * 0.7, (t * 6.0).sin() * 0.7];
                    assert_eq!(
                        nu.apply(p),
                        com.apply(p),
                        "a pegada plana divergiu da de sempre em {p:?}"
                    );
                    assert_eq!(
                        nu.outline_at(t),
                        com.outline_at(t),
                        "o contorno divergiu em {t}"
                    );
                }
            }
        }
    }

    /// ⭐⭐⭐ **A CURVATURA CHEGA A QUEM AMOSTRA** — e este é o gate que mata o atalho errado.
    ///
    /// ⛔⛔ Uma pegada **sem rotação e sem achatamento** mas COM curvatura lê-se, nos três campos
    /// antigos, exactamente como a identidade. Se o [`FootprintDeform::is_identity`] continuasse a
    /// perguntar só por eles, o `apply` devolvia `p` e a curvatura **evaporava em silêncio** — sem
    /// erro de compilação, sem gate vermelho, e com a foto do dono a repetir-se.
    #[test]
    fn a_curved_footprint_is_never_mistaken_for_the_identity() {
        let curva = CURVAS[3];
        let fp = FootprintDeform::new(0.0, 0).with_curve(curva);
        assert!(
            !fp.is_identity(),
            "uma pegada com curvatura declarou-se identidade — o `apply` vai devolver `p`"
        );
        let p = [0.6_f32, 0.35];
        let reta = FootprintDeform::new(0.0, 0).apply(p);
        let dobrada = fp.apply(p);
        let d = (reta[0] - dobrada[0]).hypot(reta[1] - dobrada[1]);
        assert!(
            d > 1e-3,
            "a curvatura não chegou ao amostrador (desvio {d}) — ela foi engolida pelo atalho"
        );
    }

    /// ⭐ **E ela SOBREVIVE ao giro do dab** (o *Jitter Rotate* e o rumo vivo do traço).
    ///
    /// ⛔ **A metade anti-vácuo é a segunda asserção:** largar `curve` no
    /// [`FootprintDeform::rotated_by`] deixa a primeira passar — uma pegada sem curvatura também
    /// roda. O que a mutação apaga é a curva de nível deixar de ser uma elipse, e é isso que se mede.
    #[test]
    fn the_curvature_survives_the_dab_spin() {
        let fp = FootprintDeform::new(0.3, 40).with_curve(CURVAS[3]);
        let rodada = fp.rotated_by([0.6, 0.8]);
        assert!(
            !rodada.curve().is_flat(),
            "o giro do dab deitou fora a curvatura da arte"
        );
        // A curva de nível da rodada não pode ser uma elipse: numa elipse os raios opostos são
        // IGUAIS, e uma dobra quebra essa simetria.
        let (mut pior, mut casos) = (0.0_f32, 0);
        for k in 0..32 {
            let a = rodada.outline_at(k as f32 / 64.0);
            let b = rodada.outline_at(k as f32 / 64.0 + 0.5);
            let (ra, rb) = (a[0].hypot(a[1]), b[0].hypot(b[1]));
            pior = pior.max((ra - rb).abs());
            casos += 1;
        }
        assert_eq!(casos, 32);
        assert!(
            pior > 1e-2,
            "o contorno da pegada rodada é simétrico como uma elipse (pior assimetria {pior}) — \
             a curvatura não está a ser honrada depois do giro"
        );
    }

    /// ⭐ **E a rotação do dab MOVE o contorno** — a metade anti-vácuo: sem ela a asserção de cima
    /// passa sobre um contorno que ignorasse o ângulo (num círculo tudo é fronteira).
    #[test]
    fn the_outline_turns_with_the_dab() {
        let reto = FootprintDeform::new(0.5, 0);
        let torto = FootprintDeform::new(0.5, 90);
        let a = reto.outline_at(0.0);
        let b = torto.outline_at(0.0);
        let d = (a[0] - b[0]).hypot(a[1] - b[1]);
        assert!(
            d > 0.5,
            "o contorno não roda com o ângulo do dab (desvio {d}): ele está a ignorar a orientação"
        );
    }
}

#[cfg(test)]
mod extent_tests {
    use super::super::*;

    /// ⭐⭐⭐ **A CAIXA CONTÉM A CURVA DE NÍVEL** — a única promessa que ela faz, varrida.
    ///
    /// ⛔ **A metade ANTI-VÁCUO é a segunda:** devolver `[1, 1]` sempre satisfaz a contenção e não
    /// serve para nada. Num dab achatado a caixa TEM de encolher, e é isso que paga o tecto do raio.
    #[test]
    fn the_extent_contains_the_level_set_and_is_tight() {
        let curvas = [
            FootprintCurve::flat(),
            FootprintCurve::from_rows([
                [0.10, -0.06, 0.04, 0.03, 0.0, -0.02, 0.0],
                [-0.05, 0.11, 0.0, 0.0, 0.06, 0.0, -0.04],
            ]),
        ];
        let mut casos = 0;
        for flatten in [0.0_f32, 0.25, 0.6, 0.9] {
            for angle in [0_u16, 23, 90, 197] {
                for curve in curvas {
                    let fp = FootprintDeform::new(flatten, angle).with_curve(curve);
                    let [ex, ey] = fp.extent();
                    // CONTENÇÃO: nenhum ponto com `falloff_t <= 1` pode cair fora da caixa.
                    for i in 0..=120 {
                        for j in 0..=120 {
                            let p = [-2.0 + 4.0 * i as f32 / 120.0, -2.0 + 4.0 * j as f32 / 120.0];
                            if fp.falloff_t(p[0], p[1]) <= 1.0 {
                                assert!(
                                    p[0].abs() <= ex + 1e-3 && p[1].abs() <= ey + 1e-3,
                                    "o ponto {p:?} está DENTRO da pegada e FORA da caixa \
                                     [{ex}, {ey}] (flatten {flatten}, ângulo {angle}°)"
                                );
                            }
                        }
                    }
                    casos += 1;
                }
            }
        }
        assert_eq!(casos, 4 * 4 * 2, "o corpus mudou de tamanho");
        // ⛔ TIGHTNESS: uma pena muito achatada tem de dar uma caixa MUITO menor num dos eixos.
        let lasca = FootprintDeform::new(0.9, 0);
        let [_, ey] = lasca.extent();
        assert!(
            ey < 0.15,
            "a caixa de uma pena a `flatten = 0,9` mede {ey} no eixo menor — ela não encolheu, e \
             o laço do dab continua a visitar texels de cobertura zero"
        );
    }
}
