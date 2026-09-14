//! ⭐⭐ **Os gates que o CORPUS DO ORÁCULO NÃO PODE DAR.**
//!
//! A espec pede quinze gates pelo nome (§19.4). Os que o corpus já mede — o
//! sinal do avanço, as quatro quedas, as quatro curvas, as recusas — vivem na
//! bancada de paridade. Os que estão aqui são os que precisam de uma peça
//! **construída** para o caso, ou de duas passagens, ou de uma propriedade
//! **estrutural** que uma amostra do comportamento não prova.
//!
//! *Um corpus é uma amostra do comportamento do alvo, nunca uma prova da nossa
//! estrutura.*

use crate::estrutura::SEM_ANEL;
use crate::vetor::V3;
use crate::{Contorno, Controlos, Evento, Fatores, Modo, QuedaNoContorno, Topologia};

/// Uma grelha `n×n` de quads no plano `z = 0`, com passo `1/n` e canto em `0`.
fn grelha(n: usize) -> (Vec<V3>, Vec<Vec<u32>>) {
    let passo = 1.0 / n as f32;
    let mut pos = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            pos.push([i as f32 * passo, j as f32 * passo, 0.0]);
        }
    }
    let idx = |i: usize, j: usize| u32::try_from(j * (n + 1) + i).unwrap_or(u32::MAX);
    let mut faces = Vec::new();
    for j in 0..n {
        for i in 0..n {
            faces.push(vec![
                idx(i, j),
                idx(i + 1, j),
                idx(i + 1, j + 1),
                idx(i, j + 1),
            ]);
        }
    }
    (pos, faces)
}

fn normais_planas(n: usize) -> Vec<V3> {
    vec![[0.0, 0.0, 1.0]; (n + 1) * (n + 1)]
}

struct Arnes {
    pos: Vec<V3>,
    nrm: Vec<V3>,
    topo: Topologia,
    escondido: Vec<bool>,
}

fn arnes(n: usize) -> Arnes {
    let (pos, faces) = grelha(n);
    let escondido = vec![false; pos.len()];
    let topo = Topologia::construir(pos.len(), faces.iter().map(Vec::as_slice), &escondido);
    Arnes {
        nrm: normais_planas(n),
        pos,
        topo,
        escondido,
    }
}

impl Arnes {
    /// ⚠️ **O cursor NÃO fica numa quina nem na borda de cima:** a quina é
    /// recusada (grau `2`) e a borda de cima escolhe outra cadeia. Ele fica a
    /// uma célula da borda de baixo, no meio — que é onde o corpus o põe.
    fn contacto(&self, n: usize) -> V3 {
        let passo = 1.0 / n as f32;
        [0.5, passo * 0.5, 0.0]
    }

    fn comecar(&self, ctrl: &Controlos, contacto: V3) -> Contorno {
        let sob = crate::ancora::mais_proximo(&self.pos, &self.escondido, contacto)
            .expect("malha com vertices");
        Contorno::comecar(
            &self.topo,
            &self.pos,
            &self.nrm,
            &self.escondido,
            sob,
            contacto,
            ctrl,
            &crate::suave,
            Fatores::default(),
        )
        .expect("o arnes nao pode ser recusado")
    }
}

/// §19.4.3 — ⚠️⚠️ **A curva CONSTANTE também dá zero no anel `K`.**
///
/// A cláusula `d ≥ L ⇒ 0` vem **antes** da escolha da curva. *Um implementador
/// que ponha o `1,0` da constante antes do corte produz um anel a mais e um
/// degrau na borda da deformação* — e o corpus mede-o só num ponto, enquanto
/// este gate o afirma sobre a curva inteira.
#[test]
fn a_curva_constante_tambem_da_zero_no_anel_do_alcance() {
    let a = arnes(16);
    let ctrl = Controlos {
        raio_inicial: 0.25,
        raio_dinamico: 0.25,
        ..Default::default()
    };
    let k = a.comecar(&ctrl, a.contacto(16));
    let e = k.estrutura();
    assert!(e.alcance >= 2, "o arnes tem de alcancar mais de um anel");
    for curva in [
        &crate::suave as crate::Curva<'_>,
        &(|_p: f32| 1.0f32) as crate::Curva<'_>,
    ] {
        let no_alcance = crate::pesos::queda_de_profundidade(e.alcance, e.alcance, curva);
        assert_eq!(
            no_alcance, 0.0,
            "o anel K tem de dar peso zero mesmo na curva constante"
        );
        // ⭐ O controlo positivo: o anel imediatamente antes NÃO é zero, senão
        // este gate passaria sobre uma lei que zera tudo.
        assert!(
            crate::pesos::queda_de_profundidade(e.alcance - 1, e.alcance, curva) > 0.0,
            "o anel K−1 tem de ter peso — sem isto o gate mede o nada"
        );
    }
}

/// §19.4.4 — ⚠️⚠️ **A coluna do artista é ISENTA da queda no contorno, nos
/// QUATRO modos.**
///
/// Sob `RADIUS`, o ponto onde a mão carregou recebe sempre a deformação cheia,
/// mesmo que a curva já estivesse a esmorecer. *Quem a esquecer vê o pico do
/// traço cair para metade do valor medido.*
#[test]
fn a_coluna_da_ancora_e_isenta_da_queda_no_contorno() {
    let a = arnes(16);
    for queda in QuedaNoContorno::ALL {
        let ctrl = Controlos {
            queda_no_contorno: queda,
            modo: Modo::Agarrar,
            raio_inicial: 0.25,
            raio_dinamico: 0.25,
            ..Default::default()
        };
        let contacto = a.contacto(16);
        let k = a.comecar(&ctrl, contacto);
        let e = k.estrutura();
        let ancora = e.ancora as usize;
        assert_eq!(
            k.pesos()[ancora],
            1.0,
            "{queda:?}: o peso da ANCORA tem de ser 1 (anel 0 x coluna isenta)"
        );
    }
    // ⭐ **O controlo negativo, e sem ele este gate não afirma nada:** com a
    // queda `RADIUS`, uma coluna LONGE da âncora tem de ter peso **menor** —
    // senão a isenção estaria a valer para toda a cadeia e o gate acima passaria
    // sobre uma lei que nunca esmorece.
    let ctrl = Controlos {
        queda_no_contorno: QuedaNoContorno::Raio,
        raio_inicial: 0.25,
        raio_dinamico: 0.25,
        ..Default::default()
    };
    let contacto = a.contacto(16);
    let k = a.comecar(&ctrl, contacto);
    let e = k.estrutura();
    let longe = e
        .cadeia
        .iter()
        .copied()
        .filter(|&v| e.distancia_de_cadeia[v as usize] > 0.3)
        .max_by(|x, y| {
            e.distancia_de_cadeia[*x as usize].total_cmp(&e.distancia_de_cadeia[*y as usize])
        })
        .expect("a cadeia tem de chegar alem do raio");
    assert_eq!(
        k.pesos()[longe as usize],
        0.0,
        "com RADIUS, uma coluna alem do raio tem de ter peso zero"
    );
}

/// §19.4.8 — ⚠️⚠️ **A inversão NÃO inverte: ela ENCAIXA o ângulo.**
///
/// E o encaixe é **truncagem para baixo**, nunca `round` nem sobre o valor
/// absoluto: sobre um factor negativo ele **afasta-se de zero**, e é isso que
/// faz o traço invertido do corpus dar um valor **maior** que o normal.
#[test]
fn a_inversao_encaixa_o_angulo_em_vez_de_o_inverter() {
    assert_eq!(crate::leis::encaixar(0.26), 0.2);
    // ⭐ O caso que separa as três implementações possíveis: sobre um NEGATIVO,
    // a truncagem dá `−0,3` (mais longe de zero), o arredondamento dá `−0,3`
    // por outro caminho e o `abs` daria `−0,2`.
    assert_eq!(crate::leis::encaixar(-0.26), -0.3);
    assert_eq!(crate::leis::encaixar(-0.24), -0.3);
    // ⚠️ E um `round` daria `−0,2` aqui, que é a divergência que o corpus mede.
    assert_ne!(crate::leis::encaixar(-0.24), -0.2);
}

/// §19.4.12 — ⭐⭐ **PESO ZERO ⇒ TRANSLAÇÃO ZERO, provado com DUAS passagens.**
///
/// Nas leis que rodam à volta de um ponto, `P(v)` com ângulo `0` é `P₀(v)` —
/// que **não é** a posição actual se uma passagem anterior já a moveu. ⇒ a
/// translação seria `P₀(v) − actual(v)` e **desfaria** o trabalho da primeira
/// passagem. Os autores do alvo pagaram exactamente este defeito.
#[test]
fn peso_zero_nao_desfaz_o_que_a_passagem_anterior_fez() {
    let a = arnes(16);
    let ctrl = Controlos {
        modo: Modo::Dobrar,
        raio_inicial: 0.25,
        raio_dinamico: 0.25,
        ..Default::default()
    };
    let contacto = a.contacto(16);
    let k = a.comecar(&ctrl, contacto);
    let ev = Evento {
        arrasto: [0.0, -0.1, 0.0],
    };
    let mut posicoes = a.pos.clone();
    let movidos = k.passo(&a.topo, &ctrl, &ev, &a.pos, &a.nrm, &mut posicoes);
    assert!(movidos > 8, "a 1.a passagem tem de mover algo ({movidos})");
    let depois_da_primeira = posicoes.clone();

    // A segunda passagem com o MESMO contorno e o MESMO arrasto: ela recalcula a
    // deformação total a partir do repouso ⇒ tem de ser um **ponto fixo**.
    k.passo(&a.topo, &ctrl, &ev, &a.pos, &a.nrm, &mut posicoes);
    let pior = depois_da_primeira
        .iter()
        .zip(&posicoes)
        .map(|(x, y)| crate::vetor::distancia(*x, *y))
        .fold(0.0f32, f32::max);
    assert!(
        pior < 1e-6,
        "a 2.a passagem mexeu {pior:e} — a lei nao e' um ponto fixo"
    );
    // ⭐⭐ E a metade que o nome do gate promete: um vértice de peso ZERO tem de
    // estar **exactamente** onde estava, e não onde o repouso o punha.
    let e = k.estrutura();
    let sem_peso = (0..a.pos.len())
        .find(|&v| k.pesos()[v] == 0.0 && e.anel[v] != SEM_ANEL)
        .expect("tem de haver vertice no alcance com peso zero (o anel K)");
    assert_eq!(
        posicoes[sem_peso], a.pos[sem_peso],
        "um vertice de peso zero foi escrito"
    );
}

/// §19.4.15 — ⭐⭐⭐ **A DIVERGÊNCIA DECLARADA: o alcance é atado à peça.**
///
/// No alvo, quando a propagação pára por **acabarem os vértices**, `K` fica
/// maior do que o anel mais fundo que existe ⇒ **nenhum vértice está no anel
/// `K`** ⇒ os dados semeados ali ficam por preencher e o `EXPAND` deixa de
/// deformar por completo. É um defeito ABERTO do alvo, e o pedido do relator
/// dele é exactamente esta cura.
///
/// ⚠️ **Este gate mede a nossa posição, não a do alvo** — e é por isso que ele
/// tem o nome que tem.
#[test]
fn o_alcance_e_atado_a_peca_e_o_expandir_deforma_onde_o_alvo_fica_mudo() {
    let a = arnes(4);
    let ctrl = Controlos {
        modo: Modo::Expandir,
        // Um raio de propagação muito maior que a peça: `0,25 × 3 = 0,75` sobre
        // uma peça de lado `1,0` — a propagação consome a malha inteira.
        deslocamento_da_origem: 2.0,
        raio_inicial: 0.25,
        raio_dinamico: 0.25,
        ..Default::default()
    };
    let contacto = a.contacto(4);
    let k = a.comecar(&ctrl, contacto);
    let e = k.estrutura();
    let mais_fundo = e
        .anel
        .iter()
        .filter(|&&x| x != SEM_ANEL)
        .copied()
        .max()
        .expect("alguem foi alcancado");
    assert_eq!(
        e.alcance, mais_fundo,
        "o alcance tem de estar atado ao anel mais fundo que EXISTE"
    );
    assert!(
        e.fundo_da_coluna(e.ancora).is_some(),
        "com o alcance atado, a coluna da ancora TEM de ter um vertice no anel K \
         — e' isso que da' direccao ao expandir"
    );
    let ev = Evento {
        arrasto: [0.0, -0.1, 0.0],
    };
    let mut posicoes = a.pos.clone();
    let movidos = k.passo(&a.topo, &ctrl, &ev, &a.pos, &a.nrm, &mut posicoes);
    assert!(
        movidos > 0,
        "o expandir ficou MUDO — e' exactamente o defeito do alvo que esta \
         divergencia existe para curar"
    );
}

/// §19.4.2 — ⚠️⚠️ **O sujeito das recusas é o vértice SOB O CURSOR, não a
/// âncora.**
///
/// Numa quina de grelha o traço é recusado **por inteiro**, mesmo havendo a uma
/// célula dali vértices de borda perfeitamente sãos que serviriam de âncora.
#[test]
fn a_recusa_olha_para_o_vertice_sob_o_cursor() {
    let a = arnes(16);
    let ctrl = Controlos {
        raio_inicial: 0.25,
        raio_dinamico: 0.25,
        ..Default::default()
    };
    // A quina `(0,0)` — grau `2`.
    let quina = [0.0, 0.0, 0.0];
    let sob = crate::ancora::mais_proximo(&a.pos, &a.escondido, quina).expect("malha");
    assert_eq!(
        Contorno::comecar(
            &a.topo,
            &a.pos,
            &a.nrm,
            &a.escondido,
            sob,
            quina,
            &ctrl,
            &crate::suave,
            Fatores::default(),
        )
        .err(),
        Some(crate::Recusa::GrauDemasiadoBaixo),
        "a quina tem de recusar o traco inteiro"
    );
    // ⭐ O controlo: a MESMA borda, uma célula ao lado, não recusa.
    let ao_lado = [1.0 / 16.0, 0.0, 0.0];
    let sob = crate::ancora::mais_proximo(&a.pos, &a.escondido, ao_lado).expect("malha");
    assert!(
        Contorno::comecar(
            &a.topo,
            &a.pos,
            &a.nrm,
            &a.escondido,
            sob,
            ao_lado,
            &ctrl,
            &crate::suave,
            Fatores::default(),
        )
        .is_ok(),
        "uma celula ao lado da quina tem de ser um traco normal — senao a \
         recusa acima nao distingue nada"
    );
}

/// §6.2 — ⚠️ **O passeio PÁRA na quina, e o vértice que faz parar ENTRA na
/// cadeia.**
///
/// Numa peça rectangular a deformação fica confinada ao lado que o artista
/// apontou. *Pôr o teste antes de o vértice entrar faria a quina ficar de fora
/// da deformação, e o corpus mede-a dentro.*
#[test]
fn o_passeio_para_na_quina_mas_a_quina_entra_na_cadeia() {
    let a = arnes(8);
    let ctrl = Controlos {
        raio_inicial: 0.25,
        raio_dinamico: 0.25,
        ..Default::default()
    };
    let k = a.comecar(&ctrl, a.contacto(8));
    let e = k.estrutura();
    assert_eq!(
        e.cadeia.len(),
        9,
        "a cadeia tem de ser UM lado da grelha (9 vertices), e nao o perimetro"
    );
    // As duas quinas do lado de baixo estão lá dentro.
    for quina in [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]] {
        let v = crate::ancora::mais_proximo(&a.pos, &a.escondido, quina).expect("malha");
        assert!(
            e.distancia_de_cadeia[v as usize].is_finite(),
            "a quina {quina:?} tem de estar na cadeia"
        );
    }
    // E nenhum vértice do lado de cima entrou.
    assert!(
        e.cadeia.iter().all(|&v| a.pos[v as usize][1] < 0.5),
        "a cadeia saltou a quina e foi para o outro lado"
    );
}

/// §14.2 — ⭐⭐ **O resultado é função do arrasto TOTAL**, e não do número de
/// eventos — nos cinco modos conduzidos por `s`.
#[test]
fn o_resultado_nao_depende_do_numero_de_eventos() {
    let a = arnes(16);
    for modo in Modo::ALL {
        if !modo.segue_o_arrasto() {
            continue;
        }
        let ctrl = Controlos {
            modo,
            raio_inicial: 0.25,
            raio_dinamico: 0.25,
            ..Default::default()
        };
        let k = a.comecar(&ctrl, a.contacto(16));
        let total = [0.0f32, -0.1, 0.0];
        let correr = |passos: usize| {
            let mut p = a.pos.clone();
            for i in 1..=passos {
                let t = i as f32 / passos as f32;
                let ev = Evento {
                    arrasto: [total[0] * t, total[1] * t, total[2] * t],
                };
                k.passo(&a.topo, &ctrl, &ev, &a.pos, &a.nrm, &mut p);
            }
            p
        };
        let dois = correr(2);
        let oito = correr(8);
        let pior = dois
            .iter()
            .zip(&oito)
            .map(|(x, y)| crate::vetor::distancia(*x, *y))
            .fold(0.0f32, f32::max);
        assert!(
            pior < 1e-6,
            "{modo:?}: 2 e 8 eventos divergiram {pior:e} — a lei deixou de ser \
             funcao do arrasto TOTAL"
        );
        // ⭐ O controlo positivo: o modo de facto move alguma coisa.
        let moveu = a.pos.iter().zip(&oito).filter(|(x, y)| x != y).count();
        assert!(
            moveu > 8,
            "{modo:?} mal se mexeu ({moveu}) — o gate mede o nada"
        );
    }
}

/// §10.6 / §14.3 — ⛔ **E o alisar é a EXCEPÇÃO: ele acumula com o número de
/// eventos.**
///
/// Este gate existe para que a excepção seja **afirmada** e não descoberta: uma
/// implementação que o fizesse partir do repouso passaria o gate irmão acima e
/// divergiria de todas as fixtures de alisar do corpus.
#[test]
fn o_alisar_acumula_com_o_numero_de_eventos() {
    let a = arnes(16);
    let ctrl = Controlos {
        modo: Modo::Suavizar,
        raio_inicial: 0.25,
        raio_dinamico: 0.25,
        ..Default::default()
    };
    let k = a.comecar(&ctrl, a.contacto(16));
    // ⚠️ **Arrasto ZERO**, como as cinco fixtures de alisar do corpus: este modo
    // não é conduzido pelo arrasto, e é isso que o separa dos outros cinco.
    let ev = Evento { arrasto: [0.0; 3] };
    let correr = |passos: usize| {
        let mut p = a.pos.clone();
        for _ in 0..passos {
            k.passo(&a.topo, &ctrl, &ev, &a.pos, &a.nrm, &mut p);
        }
        a.pos
            .iter()
            .zip(&p)
            .map(|(x, y)| crate::vetor::distancia(*x, *y))
            .sum::<f32>()
    };
    let um = correr(1);
    let oito = correr(8);
    assert!(
        um > 0.0,
        "o alisar tem de deformar com o cursor PARADO — e' a prova de que ele \
         nao e' conduzido pelo arrasto"
    );
    assert!(
        oito > um * 1.5,
        "oito passos ({oito:e}) tem de somar bem mais que um ({um:e}) — o \
         alisar acumula, e sem isso ele estaria a partir do repouso"
    );
}

/// ⭐ **Determinismo**: a mesma entrada dá a mesma saída, ao bit.
#[test]
fn a_mesma_entrada_da_a_mesma_saida_ao_bit() {
    let a = arnes(16);
    let ctrl = Controlos {
        modo: Modo::Torcer,
        queda_no_contorno: QuedaNoContorno::Laco,
        raio_inicial: 0.25,
        raio_dinamico: 0.25,
        ..Default::default()
    };
    let correr = || {
        let k = a.comecar(&ctrl, a.contacto(16));
        let mut p = a.pos.clone();
        k.passo(
            &a.topo,
            &ctrl,
            &Evento {
                arrasto: [0.0, -0.1, 0.0],
            },
            &a.pos,
            &a.nrm,
            &mut p,
        );
        p
    };
    assert_eq!(correr(), correr(), "duas corridas iguais divergiram");
}
