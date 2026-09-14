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
            faces.push(vec![idx(i, j), idx(i + 1, j), idx(i + 1, j + 1), idx(i, j + 1)]);
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
    let (pos, faces) = grelha(24);
    let escondido = vec![false; pos.len()];
    let viz = Vizinhanca::construir(pos.len(), &faces, &escondido);
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
        );
    }
    let mut saida = Vec::new();
    pose.posicoes(ctrl, &pos, Default::default(), &mut saida);
    (saida, pose.cadeia().segmentos.clone())
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
        assert_eq!(s.rot, crate::Rot::IDENTIDADE, "segmento {i}: rotacao mexida");
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
