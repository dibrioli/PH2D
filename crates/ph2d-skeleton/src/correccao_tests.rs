//! Os gates da CORRECÇÃO DE PESO à mão — o no-op, a mancha, o sinal, e as duas leis.

use super::*;

/// Dois ossos rectos lado a lado sobre o `+X`, cada um com alcance `1`.
///
/// ⚠️ **DOIS e não um:** com um osso só os pesos renormalizam para `1` e toda correcção é
/// **inerte por construção** — *uma fixtura que não contém o fenómeno não prova nada sobre ele*.
fn dois_ossos() -> Skin {
    let bone = |x: f64| SkinBone {
        rest_a: [x, 0.0],
        rest_b: [x + 1.0, 0.0],
        radius: 1.0,
        pose: Xform::IDENTITY,
        sub: (0, 1),
        tendon: if x == 0.0 { 0 } else { 1 },
    };
    Skin::new(vec![bone(0.0), bone(1.0)]).expect("a pele nasce")
}

/// Os pesos de `p` com `cs` aplicado, pela lei `guardados`.
fn pesos(pele: &Skin, p: [f64; 2], guardados: Option<&[f64]>, cs: &[Correccao]) -> Vec<f64> {
    let mut w = pele.scratch();
    pele.weights_corrected(p, guardados, &mut w, cs);
    w
}

/// ⭐⭐⭐ **SEM CORRECÇÃO, AS DUAS LEIS SÃO BYTE-IDÊNTICAS AO QUE ERAM** — a lei da casa (*todo
/// motor novo é no-op no ponto neutro*), e aqui ela tem de valer para as DUAS portas.
#[test]
fn sem_correccao_as_duas_leis_ficam_ao_bit() {
    let pele = dois_ossos();
    for p in [[0.5, 0.2], [1.5, -0.3], [1.0, 0.0], [5.0, 5.0]] {
        let mut a = pele.scratch();
        pele.weights_at(p, &mut a);
        assert_eq!(a, pesos(&pele, p, None, &[]), "a lei euclidiana mudou");

        let tabela = [0.25, 0.75];
        let mut b = pele.scratch();
        pele.weights_from(p, &tabela, &mut b);
        assert_eq!(
            b,
            pesos(&pele, p, Some(&tabela), &[]),
            "a lei do padrao-ouro mudou"
        );
    }
}

/// ⭐⭐⭐ **A CORRECÇÃO MOVE O PESO DAQUELE OSSO, NAQUELE SÍTIO** — e as duas leis obedecem-lhe.
///
/// ⚠️ **O artista corrige o ponto, e de que lei veio o peso que ele está a corrigir não é pergunta
/// dele** — é por isso que a porta é uma só.
#[test]
fn a_correccao_sobe_o_peso_do_osso_que_ela_nomeia() {
    let pele = dois_ossos();
    let p = [1.0, 0.0];
    let c = [Correccao {
        tendon: 0,
        centro: p,
        raio: 0.5,
        especie: Especie::Soma(0.5),
    }];
    for guardados in [None, Some(&[0.5, 0.5][..])] {
        let base = pesos(&pele, p, guardados, &[]);
        let com = pesos(&pele, p, guardados, &c);
        assert!(
            com[0] > base[0] + 1e-9,
            "o peso do osso 0 nao subiu ({} contra {})",
            com[0],
            base[0]
        );
        assert!(
            com[1] < base[1] - 1e-9,
            "o peso do osso 1 nao desceu: a renormalizacao nao correu"
        );
        let soma: f64 = com.iter().sum();
        assert!(
            (soma - 1.0).abs() < 1e-12,
            "os pesos corrigidos somam {soma} e nao 1"
        );
    }
}

/// ⭐⭐ **O SINAL É A DIRECÇÃO** — um delta negativo TIRA, e não há um segundo modo a lembrar.
#[test]
fn um_delta_negativo_tira_peso() {
    let pele = dois_ossos();
    let p = [1.0, 0.0];
    let base = pesos(&pele, p, None, &[]);
    let com = pesos(
        &pele,
        p,
        None,
        &[Correccao {
            tendon: 0,
            centro: p,
            raio: 0.5,
            especie: Especie::Soma(-0.5),
        }],
    );
    assert!(
        com[0] < base[0] - 1e-9,
        "o delta negativo nao tirou peso ({} contra {})",
        com[0],
        base[0]
    );
}

/// ⭐⭐⭐ **FORA DA MANCHA NADA MUDA, E A BORDA É SUAVE** — o bump `(1 − x²)²` é o MESMO da lei
/// euclidiana.
///
/// ⛔ **Uma queda linear deixaria uma aresta visível no sítio exacto onde o artista pintou** — o
/// estalo que a continuidade C¹ da casa existe para não ter. A régua é a **segunda diferença** ao
/// atravessar a borda: num bump C¹ ela encolhe com o passo, numa quina ela não.
#[test]
fn fora_da_mancha_nada_muda_e_a_borda_nao_estala() {
    let pele = dois_ossos();
    let c = [Correccao {
        tendon: 0,
        centro: [1.0, 0.0],
        raio: 0.4,
        especie: Especie::Soma(0.5),
    }];
    // (a) FORA: byte-idêntico.
    let fora = [1.0, 0.9];
    assert_eq!(
        pesos(&pele, fora, None, &[]),
        pesos(&pele, fora, None, &c),
        "um ponto FORA da mancha mudou de peso"
    );
    // (b) NA BORDA: a segunda diferença encolhe com o passo (C¹), e num degrau não encolheria.
    // ⚠️ **A régua mede a CONTRIBUIÇÃO da mancha** (`corrigido − base`) e não o peso final: o peso
    // final carrega a curvatura da própria lei de base, que é grande ali e afogaria o que se quer
    // medir. *Uma régua que soma duas curvaturas não distingue qual delas estala.*
    let em = |x: f64| {
        let p = [x, 0.0];
        pesos(&pele, p, None, &c)[0] - pesos(&pele, p, None, &[])[0]
    };
    let salto = |h: f64| {
        let b = 1.0 + 0.4; // a borda da mancha
        // A segunda diferença: `f(b−h) − 2·f(b) + f(b+h)`.
        (em(b - h) + em(b + h) - 2.0 * em(b)).abs()
    };
    let (grosso, fino) = (salto(0.05), salto(0.0125));
    assert!(
        fino < grosso * 0.5,
        "a borda estala: a segunda diferenca leu {fino:.9} a passo fino contra {grosso:.9} a passo \
         grosso — num bump C¹ ela tinha de encolher com o passo"
    );
}

/// ⛔⛔ **TIRAR TUDO DEIXA O PONTO ONDE ESTÁ, e nunca na origem** — a mesma lei do [`Skin::blend`].
///
/// ⚠️ Sem esta cerca a renormalização dividiria por zero e a arte inteira viraria `NaN`.
#[test]
fn tirar_todo_o_peso_deixa_o_ponto_intacto() {
    let pele = dois_ossos();
    let p = [1.0, 0.0];
    let cs: Vec<Correccao> = (0..2)
        .map(|t| Correccao {
            tendon: t,
            centro: p,
            raio: 2.0,
            especie: Especie::Soma(-1.0),
        })
        .collect();
    let w = pesos(&pele, p, None, &cs);
    assert!(
        w.iter().all(|v| v.abs() < 1e-12),
        "os pesos deviam ter ido todos a zero: {w:?}"
    );
    assert!(
        w.iter().all(|v| v.is_finite()),
        "a renormalizacao dividiu por zero: {w:?}"
    );
    let mut scratch = pele.scratch();
    let q = pele.point_corrected(p, None, &mut scratch, &cs);
    assert_eq!(q, p, "o ponto saltou em vez de ficar onde estava");
}

/// ⚠️ **Uma mancha com raio zero ou delta absurdo é SALTADA** — um ficheiro editado à mão chega
/// aqui, e a resposta certa é não fazer nada em vez de escrever `NaN` na arte.
#[test]
fn uma_mancha_degenerada_e_saltada() {
    let pele = dois_ossos();
    let p = [1.0, 0.0];
    let base = pesos(&pele, p, None, &[]);
    for c in [
        Correccao {
            tendon: 0,
            centro: p,
            raio: 0.0,
            especie: Especie::Soma(0.5),
        },
        Correccao {
            tendon: 0,
            centro: p,
            raio: -1.0,
            especie: Especie::Soma(0.5),
        },
        Correccao {
            tendon: 0,
            centro: p,
            raio: 0.5,
            especie: Especie::Soma(f64::NAN),
        },
        // ⚠️ Um tendão que não existe na pele: o osso foi apagado depois de o artista corrigir.
        Correccao {
            tendon: 99,
            centro: p,
            raio: 0.5,
            especie: Especie::Soma(0.5),
        },
    ] {
        assert_eq!(
            pesos(&pele, p, None, &[c]),
            base,
            "uma mancha degenerada ({c:?}) mexeu nos pesos"
        );
    }
}
