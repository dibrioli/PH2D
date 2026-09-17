//! ⭐⭐ **Os gates que o CORPUS DO ORÁCULO NÃO PODE DAR.**
//!
//! A prova de mutação da bancada de paridade matou `11` de `14` mutantes. Os
//! três sobreviventes têm todos a **mesma** causa, e ela não é fraqueza dos
//! gates: são propriedades que os `69` traços publicados **não discriminam**.
//! *Um corpus é uma amostra do comportamento do alvo, nunca uma prova da nossa
//! estrutura* — e é por isso que estes três vivem aqui, escritos contra a lei e
//! não contra o oráculo.
//!
//! | mutante que sobreviveu | porque é que o corpus é cego | o gate que o mata |
//! |---|---|---|
//! | semente por ordem de chegada | com simetria em X há no máximo **2** sementes, e o desempate quase nunca decide | [`a_semente_sai_ordenada`] |
//! | espremer **resolve** a cadeia | ⭐ **todas** as fixturas de espremer são ancoradas, e com âncora e um segmento resolver é um **no-op** | [`o_espremer_nao_resolve_a_cadeia`] |
//! | sem a guarda de `1e-5` | nenhum traço leva o quociente abaixo do limiar | [`a_guarda_do_espremer_impede_o_infinito`] |

use crate::cadeia::Segmento;
use crate::vetor::V3;
use crate::{Controlos, Evento, Modo, Pose, Vizinhanca};

/// Uma grelha `n×n` no plano `z = 0`, com passo `1/n`.
fn grelha(n: usize) -> (Vec<V3>, Vec<Vec<u32>>) {
    let passo = 1.0 / n as f32;
    let mut pos = Vec::new();
    for j in 0..=n {
        for i in 0..=n {
            pos.push([i as f32 * passo, j as f32 * passo, 0.0]);
        }
    }
    let idx = |i: usize, j: usize| (j * (n + 1) + i) as u32;
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

/// ⚠️⚠️ **O cursor NÃO pode ficar no centro da grelha, e isto é load-bearing.**
/// No centro a franja é um anel simétrico, a média dela cai **em cima do
/// cursor**, o primeiro segmento nasce com comprimento nulo e o traço inteiro é
/// **inerte** (§11.1). A primeira redacção destes gates fez isso e eles ficaram
/// verdes a medir um sujeito morto — o mutante que apaga a guarda do espremer
/// sobreviveu **duas vezes** por causa disto. ⇒ o cursor vive perto da borda,
/// onde a franja é assimétrica e o pivô é empurrado para dentro da peça, e
/// [`traco`] **afirma** que a cadeia nasceu viva.
const CURSOR: V3 = [0.5, 0.1, 0.0];

fn traco(ctrl: &Controlos, arrasto: V3) -> (Vec<V3>, Vec<Segmento>) {
    let (pose, pos) = pose_apos(ctrl, arrasto);
    let mut saida = Vec::new();
    pose.posicoes(ctrl, &pos, Default::default(), &mut saida);
    (saida, pose.cadeia().segmentos.clone())
}

/// O traço inteiro, mas devolvendo a **pose** em vez das posições — é o que os
/// gates do osso precisam, e partilhar o arnês é o que impede que eles meçam
/// outro gesto que não o dos irmãos.
fn pose_apos(ctrl: &Controlos, arrasto: V3) -> (Pose, Vec<V3>) {
    let (pos, faces) = grelha(24);
    let escondido = vec![false; pos.len()];
    let viz = Vizinhanca::construir(pos.len(), faces.iter().map(Vec::as_slice), &escondido);
    let cursor = CURSOR;
    let eleito = crate::cadeia::mais_proximo_global(&pos, &escondido, cursor).expect("malha");
    let mut pose = Pose::comecar(&viz, &pos, &escondido, eleito, cursor, ctrl);
    // ⭐ **O controlo positivo do próprio sujeito:** sem isto um arnês
    // degenerado deixa todo gate desta pasta verde a medir zero.
    assert!(
        !pose.inerte(),
        "o arnes nasceu inerte — o pivo caiu em cima do cursor e nao ha nada a medir"
    );
    for k in 1..=8 {
        let t = k as f32 / 8.0;
        pose.evento(
            ctrl,
            &Evento {
                arrasto: [arrasto[0] * t, arrasto[1] * t, arrasto[2] * t],
                dx_pixels: arrasto[0] * t * 100.0,
            },
            &crate::suave,
        );
    }
    (pose, pos)
}

/// §2.1 — ⚠️ **a semente é ordenada por índice crescente antes de começar**, e
/// isso é exigência de **determinismo**: a ordem de visita decide qual vértice
/// fica registado como «o mais afastado» (§2.3) em caso de empate.
///
/// ⛔ O corpus não o vê porque com um só eixo de espelho a semente tem dois
/// membros; a asserção é sobre a **estrutura**, e por isso mora aqui.
#[test]
fn a_semente_sai_ordenada() {
    let (pos, _) = grelha(12);
    let escondido = vec![false; pos.len()];
    let lista = crate::cadeia::semente_para_teste(&pos, &escondido, 100, [true, true, false], 2.0);
    assert!(
        lista.windows(2).all(|p| p[0] < p[1]),
        "a semente tem de sair estritamente crescente: {lista:?}"
    );
    assert!(
        lista.len() >= 2,
        "com dois eixos de espelho a semente tem de trazer parceiros: {lista:?}"
    );
}

/// §5.5 / item **13** da lista de verificação — ⚠️⚠️ **o espremer NÃO resolve a
/// cadeia**: cabeça, origem e rotação ficam nos valores iniciais o traço todo.
///
/// ⛔⛔ **E este gate existe porque o corpus não o impõe:** todas as fixturas de
/// espremer publicadas são **ancoradas**, e com âncora ligada e um segmento a
/// resolução repõe a origem exactamente onde estava ⇒ resolver ou não resolver
/// dá a **mesma** saída. *A divergência só se torna observável com a âncora
/// desligada — e essa fixtura não existe.*
#[test]
fn o_espremer_nao_resolve_a_cadeia() {
    let ctrl = Controlos {
        modo: Modo::EspremerEsticar,
        ancorado: false,
        raio: 0.25,
        ..Default::default()
    };
    let (_, segmentos) = traco(&ctrl, [0.0, 0.35, 0.0]);
    for (i, s) in segmentos.iter().enumerate() {
        assert_eq!(
            s.origem, s.origem_inicial,
            "segmento {i}: o espremer moveu a origem — a cadeia foi resolvida"
        );
        assert_eq!(
            s.rot,
            crate::Rot::IDENTIDADE,
            "segmento {i}: rotacao mexida"
        );
    }
    // E o controlo positivo, na mesma malha: o modo que **resolve** move-a.
    let girar = Controlos {
        modo: Modo::GirarTorcer,
        ancorado: false,
        raio: 0.25,
        ..Default::default()
    };
    let (_, resolvidos) = traco(&girar, [0.0, 0.35, 0.0]);
    assert_ne!(
        resolvidos[0].origem, resolvidos[0].origem_inicial,
        "o controlo positivo falhou: nem o modo que resolve moveu a origem"
    );
}

/// §5.5 / item **9** — ⭐ a guarda de `1e-5`, e a razão pública de ela existir:
/// sem ela a malha ia a `NaN` **e o desfazer não a recuperava**.
///
/// ⛔ Nenhum traço do corpus leva o quociente abaixo do limiar, então o mutante
/// que apaga a guarda **sobrevive à paridade inteira**. Aqui construímos o caso.
#[test]
fn a_guarda_do_espremer_impede_o_infinito() {
    let ctrl = Controlos {
        modo: Modo::EspremerEsticar,
        raio: 0.25,
        ..Default::default()
    };
    // ⚠️⚠️ **O regime da guarda NÃO é o polo — é o OUTRO extremo, e a primeira
    // redacção deste gate procurou-o no sítio errado.** O quociente é
    // `c₀/(c₀ − δ)`: ele **explode** quando `δ → c₀` (o polo, §11.2, onde não há
    // guarda nenhuma) e vai a **ZERO** quando `|δ| → ∞`. A guarda de `1e-5`
    // defende o segundo, e com `c₀ = 0,25` ele só começa a `|δ| > 25 000` —
    // varrer `±4` nunca lá chega, e o mutante que apaga a guarda **sobreviveu**
    // ao gate inteiro por causa disso. *Um gate que não alcança o regime que diz
    // defender é um gate verde sobre código morto.*
    let mut pior = 0.0f32;
    for escala in [1e5f32, 1e6, 1e7, 1e8] {
        for sinal in [-1.0f32, 1.0] {
            let (saida, _) = traco(&ctrl, [0.0, sinal * escala, 0.0]);
            for p in &saida {
                for c in p {
                    assert!(
                        c.is_finite(),
                        "posicao nao finita com arrasto {}: {p:?}",
                        sinal * escala
                    );
                    pior = pior.max(c.abs());
                }
            }
        }
    }
    // ⭐ Com a guarda, a escala colapsa a `(0,0,0)` e os vértices de peso `1`
    // aterram **no pivô** — logo a saída fica dentro da própria peça. Sem ela,
    // `√(1/|z|)` a `|z| ~ 2,5e-8` multiplica tudo por `~6 300`.
    assert!(
        pior < 10.0,
        "a guarda existe mas a saida saiu da peca, ate {pior:e}"
    );
}

/// ⭐ **Determinismo**: a mesma entrada dá a mesma saída, ao bit.
///
/// É a metade que podemos afirmar da promessa do §12 — o nosso Jacobi é limpo e
/// a travessia é FIFO sobre uma adjacência ordenada.
#[test]
fn a_mesma_entrada_da_a_mesma_saida_ao_bit() {
    let ctrl = Controlos {
        segmentos: 3,
        suavizacoes_do_peso: 6,
        raio: 0.3,
        ..Default::default()
    };
    let (a, _) = traco(&ctrl, [0.2, 0.3, 0.1]);
    let (b, _) = traco(&ctrl, [0.2, 0.3, 0.1]);
    assert_eq!(a, b, "duas corridas iguais divergiram");
}

/// ⭐ **O osso em REPOUSO é o par inicial** — o indicador que se vê antes de
/// premir descreve a cadeia que o pen-down vai construir.
///
/// ⚠️ **A barra não é o bit e a razão é aritmética, não tolerância a defeito:**
/// a cabeça sai de `origem + M(cabeça₀ − origem₀)` e em `f32` `a + (b − a)` não
/// devolve `b` exactamente. A barra é relativa ao tamanho da peça.
#[test]
fn o_osso_em_repouso_e_o_par_inicial() {
    let ctrl = Controlos {
        segmentos: 3,
        raio: 0.3,
        ..Default::default()
    };
    let (pos, faces) = grelha(24);
    let escondido = vec![false; pos.len()];
    let viz = Vizinhanca::construir(pos.len(), faces.iter().map(Vec::as_slice), &escondido);
    let eleito = crate::cadeia::mais_proximo_global(&pos, &escondido, CURSOR).expect("malha");
    let pose = Pose::comecar(&viz, &pos, &escondido, eleito, CURSOR, &ctrl);
    assert!(!pose.inerte(), "o arnes nasceu inerte");
    let mut ossos = Vec::new();
    pose.ossos(&ctrl, &mut ossos);
    assert_eq!(
        ossos.len(),
        pose.cadeia().segmentos.len(),
        "um osso por segmento"
    );
    for (osso, seg) in ossos.iter().zip(&pose.cadeia().segmentos) {
        assert_eq!(
            osso[0], seg.origem_inicial,
            "a origem em repouso e' a inicial"
        );
        let erro = crate::vetor::distancia(osso[1], seg.cabeca_inicial);
        assert!(
            erro < 1e-6,
            "a cabeca em repouso desviou {erro:e} da inicial"
        );
    }
}

/// ⭐⭐ **O osso é a LEI aplicada à cabeça, e este gate mata o atalho.**
///
/// ⛔ O atalho plausível — `origem + rot·(cabeça₀ − origem₀)` — concorda com a
/// lei em quatro das cinco deformações, e é **por isso** que ele entra sem
/// ninguém ver. No espremer/esticar a rotação é a identidade e a escala vive
/// numa base **local ao segmento**: o atalho desenha um osso do tamanho
/// original enquanto a peça estica.
///
/// ⚠️ **O controlo negativo está DENTRO do gate**: ele afirma que o atalho de
/// facto diverge aqui, senão a comparação principal passaria por os dois
/// caminhos coincidirem.
#[test]
fn o_osso_e_a_lei_aplicada_a_cabeca_e_nao_o_atalho() {
    let ctrl = Controlos {
        modo: Modo::EspremerEsticar,
        raio: 0.3,
        ..Default::default()
    };
    let (pose, _) = pose_apos(&ctrl, [0.0, 0.18, 0.0]);
    let mut ossos = Vec::new();
    pose.ossos(&ctrl, &mut ossos);
    let seg = &pose.cadeia().segmentos[0];
    let atalho = {
        let d = crate::vetor::sub(seg.cabeca_inicial, seg.origem_inicial);
        crate::vetor::add(seg.origem, seg.rot.aplicar(d))
    };
    let divergencia = crate::vetor::distancia(ossos[0][1], atalho);
    assert!(
        divergencia > 1e-3,
        "o controlo negativo caiu: o atalho concorda com a lei ({divergencia:e}), \
         logo este gate nao esta' a medir nada"
    );
    // E a lei é mesmo a do §6: o osso tem o comprimento que a escala do
    // segmento manda, medido ao longo da própria direcção.
    let esticado = crate::vetor::distancia(ossos[0][0], ossos[0][1]);
    let razao = esticado / seg.comprimento;
    assert!(
        (razao - seg.escala[2]).abs() < 1e-3,
        "o osso esticou {razao} e a escala do segmento e' {:?}",
        seg.escala
    );
}

/// ⚠️⚠️ **Com simetria ligada e a âncora em coordenada NEGATIVA, o osso fica do
/// lado da mão.** As reflexões do §6 cancelam-se em pares, logo o mapa não
/// reflectido é o do octante **da âncora** — usar o `0` desenharia o indicador
/// espelhado no outro lado da peça, e só em malhas cujo cursor cai em `x < 0`.
#[test]
fn o_osso_fica_do_lado_da_ancora_com_simetria() {
    let ctrl = Controlos {
        simetria: [true, false, false],
        raio: 0.25,
        ..Default::default()
    };
    // A grelha vive em `x ∈ [0,1]`; deslocá-la põe o cursor em `x < 0`.
    let (mut pos, faces) = grelha(24);
    for p in pos.iter_mut() {
        p[0] -= 1.0;
    }
    let cursor = [CURSOR[0] - 1.0, CURSOR[1], CURSOR[2]];
    let escondido = vec![false; pos.len()];
    let viz = Vizinhanca::construir(pos.len(), faces.iter().map(Vec::as_slice), &escondido);
    let eleito = crate::cadeia::mais_proximo_global(&pos, &escondido, cursor).expect("malha");
    let mut pose = Pose::comecar(&viz, &pos, &escondido, eleito, cursor, &ctrl);
    assert!(!pose.inerte(), "o arnes nasceu inerte");
    pose.evento(
        &ctrl,
        &Evento {
            arrasto: [0.0, 0.2, 0.0],
            dx_pixels: 0.0,
        },
        &crate::suave,
    );
    let mut ossos = Vec::new();
    pose.ossos(&ctrl, &mut ossos);
    for osso in &ossos {
        assert!(
            osso[1][0] < 0.0,
            "o osso saltou para x >= 0 — o octante usado foi o do outro lado: {osso:?}"
        );
    }
}

/// ⛔⛔ **GATE — a `Deformacao` e o par `(modo, invertido)` são a MESMA coisa nos
/// dois sentidos.**
///
/// A lei lê o par; o artista escolhe a deformação. Enquanto forem duas tabelas,
/// elas divergem no dia da sexta entrada — e a divergência é **muda**: o painel
/// mostraria um nome e o barro faria outro gesto.
///
/// ⚠️ **As duas metades são necessárias, e a segunda é a que morde:** a ida
/// prova que toda deformação tem par; a VOLTA prova que todo par que a lei sabe
/// resolver é **alcançável** pelo painel — sem ela, uma deformação podia
/// desaparecer da lista e o gate ficava verde.
#[test]
fn a_deformacao_e_o_par_da_lei_sao_a_mesma_coisa() {
    use crate::{Controlos, Deformacao, Modo};

    for d in Deformacao::ALL {
        let (modo, invertido) = d.modo_e_inversao();
        let ctrl = Controlos {
            modo,
            invertido,
            ..Controlos::default()
        };
        assert_eq!(
            ctrl.deformacao(),
            d,
            "a ida-e-volta de {d:?} não fecha: o painel escolheria uma coisa e a \
             lei faria outra"
        );
    }
    // A VOLTA: todo par que a lei sabe resolver é alcançável pela lista.
    for modo in Modo::ALL {
        for invertido in [false, true] {
            let d = Controlos {
                modo,
                invertido,
                ..Controlos::default()
            }
            .deformacao();
            assert!(
                Deformacao::ALL.contains(&d),
                "a lei resolve ({modo:?}, {invertido}) em {d:?} e o painel não \
                 tem esse botão — ele seria inalcançável"
            );
        }
    }
    assert_eq!(
        Deformacao::ALL.len(),
        5,
        "o piso de população: os três modos dão CINCO deformações distintas \
         (o modificador não muda o espremer/esticar, e está medido)"
    );
}

/// ⭐⭐⭐ **GATE — ao longo do osso as duas leis do arrasto COINCIDEM**, que é a
/// propriedade que mantém as `69` fixturas do oráculo fora do alcance do botão
/// novo (ordem do dono, 17/09).
///
/// ⚠️ **A barra não é um epsilon escolhido:** para `d = α·n̂` a identidade é
/// exacta em aritmética real (`sign(α)·|α| = α`), e o que sobra em `f32` é o
/// arredondamento de `‖d‖ = |α|·√(n̂·n̂)` quando `n̂·n̂` não é exactamente `1`.
/// ⇒ a barra é **relativa** à magnitude, e o gate varre normais que **não** são
/// eixos — sobre `[0,0,1]` ela seria trivialmente verdadeira e o gate não
/// afirmaria nada.
#[test]
fn ao_longo_do_osso_as_duas_leis_do_arrasto_coincidem() {
    let normais = [
        crate::vetor::normalizar([0.3, -0.7, 0.65]).expect("normal"),
        crate::vetor::normalizar([1.0, 1.0, 1.0]).expect("normal"),
        crate::vetor::normalizar([-0.2, 0.9, 0.1]).expect("normal"),
    ];
    for n in normais {
        for alfa in [-3.5f32, -1.0, -0.25, 0.0, 0.25, 1.0, 3.5] {
            let d = crate::vetor::escalar(n, alfa);
            let a = crate::Arrasto::AoLongoDoOsso.alavanca(d, n);
            let c = crate::Arrasto::Completo.alavanca(d, n);
            assert!(
                (a - c).abs() <= 1e-6 * alfa.abs().max(1.0),
                "n={n:?} alfa={alfa}: ao longo do osso as duas tinham de \
                 coincidir, e deram {a} contra {c}"
            );
        }
    }
}

/// ⛔⛔ **GATE — e num arrasto TRANSVERSAL elas TÊM de diferir.**
///
/// *Sem esta metade o botão é decorativo*: um `Completo` que por acidente
/// devolvesse a projecção passaria no gate irmão e o artista teria dois chips
/// que fazem a mesma coisa. ⇒ o gate **exige** que a divergência exista, que é
/// a mesma lei do tecto que vira licença.
///
/// ⭐ **E a direcção da diferença é afirmada, não só a magnitude:** a lei nova
/// nunca lê MENOS que a projecção (‖d‖ ≥ |dot(d,n̂)| por Cauchy–Schwarz), logo o
/// módulo da alavanca só pode crescer.
#[test]
fn num_arrasto_transversal_as_duas_leis_do_arrasto_diferem() {
    let n = crate::vetor::normalizar([0.0, 0.0, 1.0]).expect("normal");
    let mut vistos = 0;
    for lateral in [0.5f32, 1.0, 2.0] {
        for axial in [-1.0f32, 0.25, 1.0] {
            let d = [lateral, 0.0, axial];
            let a = crate::Arrasto::AoLongoDoOsso.alavanca(d, n);
            let c = crate::Arrasto::Completo.alavanca(d, n);
            assert!(
                c.abs() > a.abs() + 1e-6,
                "lateral={lateral} axial={axial}: a lei completa tinha de ler \
                 MAIS que a projeccao, e deu {c} contra {a}"
            );
            assert!(
                a == 0.0 || c.signum() == a.signum(),
                "lateral={lateral} axial={axial}: a lei completa inverteu o \
                 sentido ({c} contra {a})"
            );
            vistos += 1;
        }
    }
    assert_eq!(
        vistos, 9,
        "a varredura encolheu e o gate mede menos do que diz"
    );
}

/// ⚠️ **GATE — o valor de FÁBRICA é a projecção**, e isto é um gate porque é o
/// que mantém o corpus do oráculo a medir o pincel que ele gravou.
#[test]
fn o_arrasto_de_fabrica_e_a_projeccao_no_osso() {
    assert_eq!(
        Controlos::default().lei_do_arrasto,
        crate::Arrasto::AoLongoDoOsso,
        "o default mudou — as 69 fixturas do oraculo passam a medir outro pincel"
    );
    assert_eq!(
        crate::Arrasto::ALL[0],
        crate::Arrasto::AoLongoDoOsso,
        "a ordem dos chips mudou"
    );
}
