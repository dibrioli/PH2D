//! Os gates da cena `=120` — a do ciclo 9 (doc 114 §10).
//!
//! ⚠️⚠️ **Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente**
//! (`CLAUDE.md` §5.0). O anúncio promete nove coisas; cada uma que se pode medir é um gate aqui —
//! e as que se medem por PARES têm o controlo dentro do próprio gate, porque *«mexeu»* não separa
//! a lei de um grafo diferente.
//!
//! ⚠️ **Esta cena não tem membranas** (ao contrário da `=119`): as seis cadeias fabricam tudo a
//! partir de params, logo não há nada a publicar antes de cozer. *A ausência é uma propriedade do
//! grupo — um esqueleto não vem de fora.*

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::Graph;

/// Os seis panos, pela ordem em que [`build`] os empilha.
const CORDA: usize = 0;
const CAMPO: usize = 1;
const FK: usize = 2;
const IK: usize = 3;
const PELE_IGUAL: usize = 4;
const PELE_QUINHAO: usize = 5;

/// Quantos tiques correr antes de ler, quando o gate precisa da cena EM REGIME. ⚠️ A corda e o
/// campo nascem parados: medir no tique zero mediria o repouso, que é o que a cena mostra ANTES
/// do `PLAY`.
const TIQUES: usize = 40;
const DT: f64 = 1.0 / 60.0;

/// O primeiro nó de um dado tipo no grafo.
///
/// ⚠️ **A cena tem UM de cada um dos tipos que estes gates procuram** (`motion.falloff` e
/// `motion.wave` vivem cada um num pano só), e os gates que mexem numa CANETA — de que há
/// quatro — usam a [`caneta_da_coluna`], que desempata pela coluna escrita.
fn primeiro(g: &Graph, tipo: &str) -> NodeId {
    g.nodes()
        .iter()
        .find(|n| n.type_name == tipo)
        .map(|n| n.id)
        .unwrap_or_else(|| panic!("a cena nao tem nenhum `{tipo}`"))
}

/// A caneta (`motion.drive`) que escreve uma dada coluna, no pano `k`-ésimo a ser encontrado.
///
/// ⛔ **Desempatar pela COLUNA e não pela ordem** — a cena tem quatro `motion.drive` (dois a
/// escrever `rot`, um `bone_weight`, e o do pano do FK), e um índice trocaria de sujeito no dia
/// em que alguém acrescentasse um pano antes.
fn caneta_da_coluna(g: &Graph, coluna: &str) -> Vec<NodeId> {
    g.nodes()
        .iter()
        .filter(|n| n.type_name == "motion.drive")
        .filter(|n| {
            g.node_text_param_overrides(n.id)
                .and_then(|m| m.get("column"))
                .is_some_and(|c| c == coluna)
        })
        .map(|n| n.id)
        .collect()
}

/// Monta uma cena NOVA, aplica `mexe`, corre `tiques` e devolve o stream do sink `k`.
///
/// ⚠️ **Cena nova por medição:** a corda e o campo carregam ESTADO, e um `Cook` que já andou
/// entrega outro pano a meio do caminho.
fn corre(k: usize, tiques: usize, mexe: impl FnOnce(&mut Graph)) -> Stream {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
    mexe(&mut m.doc.graph);
    let sink = sinks[k];
    let mut t = 0.0f64;
    for _ in 0..tiques {
        let _ = m.pump.cook.cook(&m.doc.graph, &m.registry, sink, t);
        let _ = m.pump.cook.advance_tick(&m.doc.graph, &m.registry, t);
        t += DT;
    }
    m.pump
        .cook
        .cook(&m.doc.graph, &m.registry, sink, t)
        .expect("coze")[0]
        .as_stream()
        .clone()
}

fn pontos(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// **A ASSINATURA de um pano** — a POSIÇÃO e o TAMANHO de cada peça, numa lista só.
///
/// ⛔⛔ **A primeira redacção destes gates lia só o `P`, e o campo ficava a ZERO** — o
/// `motion.wave` escreve a altura no canal `Size` por omissão (`height_channel = 0`), logo as
/// posições da grelha dele **nunca se mexem** e a régua declarava *«uma simulação que não
/// simula»* sobre produto correcto. ⚠️ *Uma régua que lê UMA das grandezas em que um pano se
/// exprime é cega a todos os nós que escrevem noutra*, e neste grupo isso é metade deles.
///
/// ⭐ A assinatura resolve-o pela raiz: um pano «mexeu-se» se **alguma** coisa que o desenha
/// mudou, que é exactamente a promessa que o anúncio faz ao dono.
fn assinatura(s: &Stream) -> Vec<f32> {
    let mut v = Vec::new();
    for c in ["P", "size"] {
        if let Some(Column::Vec2(p)) = s.get(c) {
            for q in p {
                v.push(q[0]);
                v.push(q[1]);
            }
        }
    }
    v
}

/// O maior desvio entre duas assinaturas com a mesma contagem. ⛔ Devolve `f32::INFINITY` quando
/// as contagens diferem — *comparar prefixos de listas de tamanhos diferentes lê `0` sobre dois
/// panos que não têm nada a ver um com o outro*.
fn maior_desvio_bruto(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return f32::INFINITY;
    }
    a.iter()
        .zip(b)
        .map(|(p, q)| (p - q).abs())
        .fold(0.0f32, f32::max)
}

/// O maior deslocamento entre dois conjuntos de PONTOS com a mesma contagem.
fn maior_desvio(a: &[[f32; 2]], b: &[[f32; 2]]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return f32::INFINITY;
    }
    a.iter()
        .zip(b)
        .map(|(p, q)| ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt())
        .fold(0.0f32, f32::max)
}

/// ⭐ **A cena monta os SEIS panos e nenhum vem vazio.**
///
/// ⚠️ **O piso de população é metade do gate:** um pano vazio é exactamente o modo de falha que o
/// anúncio manda o dono procurar (*«um pano vazio quer dizer que a fonte dele não entregou
/// nada»*), e um `assert` só sobre a contagem de sinks passaria com os seis a zero peças.
#[test]
fn a_cena_monta_seis_panos_e_nenhum_vem_vazio() {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
    assert_eq!(sinks.len(), 6, "a cena tem seis panos");
    for (k, &sink) in sinks.iter().enumerate() {
        let s = m
            .pump
            .cook
            .cook(&m.doc.graph, &m.registry, sink, 0.0)
            .unwrap_or_else(|e| panic!("o pano {k} nao coze: {e:?}"))[0]
            .as_stream()
            .clone();
        assert!(
            !pontos(&s).is_empty(),
            "o pano {k} veio VAZIO — a fonte dele nao entregou nada"
        );
    }
}

/// ⭐ **A fileira de CIMA mexe-se com o tempo** (passo 1 e 2 do anúncio).
///
/// ⚠️ **Os dois estão no mesmo gate porque a promessa é UMA:** *«a fileira de cima mexe-se
/// sozinha»*. Separá-los em dois gates faria a promessa passar por metade.
#[test]
fn a_corda_balanca_e_o_campo_ondula() {
    for (k, nome) in [(CORDA, "a corda"), (CAMPO, "o campo")] {
        let parado = assinatura(&corre(k, 0, |_| {}));
        let andado = assinatura(&corre(k, TIQUES, |_| {}));
        let d = maior_desvio_bruto(&parado, &andado);
        assert!(
            d > 1e-3,
            "{nome} nao se mexeu em {TIQUES} tiques (maior desvio {d:e}) — \
             o pano ensinaria que uma simulacao nao simula"
        );
    }
}

/// ⭐⭐⭐ **O par do MEIO não dá a mesma coisa — que é a lição inteira da fileira** (passo 5).
///
/// ⚠️⚠️ **É esta a promessa mais fácil de partir sem ninguém ver:** as duas cadeias partem do
/// MESMO `rig.skeleton` com os MESMOS params, e se o alvo não chegar ao solver a saída do IK cai
/// na pose de repouso — que é *quase* a do FK. O gate exige uma diferença que se VEJA no pano
/// (`0,05` de mundo, com as peças a medirem `0,16`).
#[test]
fn a_cinematica_directa_e_a_inversa_dao_panos_diferentes() {
    let fk = pontos(&corre(FK, TIQUES, |_| {}));
    let ik = pontos(&corre(IK, TIQUES, |_| {}));
    let d = maior_desvio(&fk, &ik);
    assert!(
        d > 0.05,
        "os dois panos do MEIO ficaram iguais (maior desvio {d:e}) — \
         a cena ensinaria que FK e IK sao dois nomes para o mesmo no"
    );
}

/// ⭐⭐ **A mão SEGUE o alvo** (passo 5): o pano do IK muda quando o alvo varre.
///
/// ⚠️ **O CONTROLO é o pano do FK no mesmo par de instantes.** Sem ele, *«o IK mudou entre t1 e
/// t2»* não separa a mão a seguir o alvo de a cena inteira estar a andar por outro motivo — e o
/// FK desta cena é `Effect::Pure`, logo tem de dar exactamente o MESMO pano nos dois instantes.
#[test]
fn a_mao_segue_o_alvo_e_o_pano_do_fk_nao_se_mexe() {
    let ik_cedo = pontos(&corre(IK, 10, |_| {}));
    let ik_tarde = pontos(&corre(IK, 70, |_| {}));
    let d_ik = maior_desvio(&ik_cedo, &ik_tarde);
    assert!(
        d_ik > 0.05,
        "a mao nao seguiu o alvo entre os dois instantes (desvio {d_ik:e})"
    );

    let fk_cedo = pontos(&corre(FK, 10, |_| {}));
    let fk_tarde = pontos(&corre(FK, 70, |_| {}));
    let d_fk = maior_desvio(&fk_cedo, &fk_tarde);
    assert!(
        d_fk <= 1e-5,
        "o CONTROLO mexeu-se: o pano do FK e' Pure e devia dar o mesmo nos dois \
         instantes (desvio {d_fk:e}) — sem ele o gate acima nao afirma nada sobre o ALVO"
    );
}

/// ⭐⭐⭐ **APERTAR O RAIO TIRA A MÃO DO ALVO — a W2 deste ciclo, pelo caminho da CENA** (passo 6).
///
/// A força de uma restrição é a coluna `falloff`, e quem a escreve neste pano é o
/// `motion.falloff`. Com o raio grande a corrente inteira lê `≈ 1` e o solver manda; apertado, as
/// juntas distantes ficam com a pose de REPOUSO e a mão fica a meio caminho.
///
/// ⚠️⚠️ **O gate mede o BARRO e não o param:** ler `radius` de volta provaria que o `set_param`
/// funciona, que é uma pergunta sobre o `Graph`. A pergunta desta wave é se o número **chega ao
/// solver** — o ponto cego que o `CLAUDE.md` §5.0 nomeia sobre si mesmo.
#[test]
fn apertar_a_forca_da_restricao_muda_a_pose_do_ik() {
    let cheia = pontos(&corre(IK, TIQUES, |_| {}));
    let apertada = pontos(&corre(IK, TIQUES, |g| {
        let alvo = primeiro(g, "motion.falloff");
        g.set_param(alvo, "radius", 1.0);
    }));
    let d = maior_desvio(&cheia, &apertada);
    assert!(
        d > 0.02,
        "apertar o raio da forca nao mudou a pose (desvio {d:e}) — \
         o passo 6 do anuncio ensinaria um controlo inerte"
    );
}

/// ⭐⭐⭐ **O par de BAIXO não dá a mesma pele — a W1′ deste ciclo, pelo caminho da CENA**
/// (passo 8).
///
/// ⚠️ **As duas cadeias são a MESMA menos dois cartões** (o `value.instance_field` + o
/// `motion.drive` que escrevem `bone_weight`): é essa a definição de controlo que esta cena podia
/// ter, e é por isso que o pano da esquerda existe.
#[test]
fn o_quinhao_por_osso_muda_a_pele() {
    let igual = pontos(&corre(PELE_IGUAL, 0, |_| {}));
    let quinhao = pontos(&corre(PELE_QUINHAO, 0, |_| {}));
    let d = maior_desvio(&igual, &quinhao);
    assert!(
        d > 1e-3,
        "os dois panos de BAIXO ficaram iguais (desvio {d:e}) — \
         o quinhao por osso nao chegou a' pele"
    );
}

/// ⭐⭐ **Com o quinhão a ZERO nenhum osso puxa** (passo 9): a pele da direita fica onde a grelha
/// a pôs.
///
/// ⚠️⚠️ **O gate mede contra a GRELHA e não contra «não mudou»:** uma pele que ficasse presa numa
/// pose qualquer também «não mudaria» entre duas corridas. A pergunta é se ela volta ao REPOUSO,
/// e o repouso é a grelha que entra no `rig.skin_deformer`.
#[test]
fn o_quinhao_a_zero_devolve_a_pele_ao_repouso() {
    let zero = pontos(&corre(PELE_QUINHAO, 0, |g| {
        // A caneta do envelope é o `motion.drive` que escreve `bone_weight`; pôr a escala a zero
        // dá `bone_weight = 0` em todos os ossos.
        let canetas = caneta_da_coluna(g, "bone_weight");
        assert_eq!(
            canetas.len(),
            1,
            "a cena tem de ter UMA caneta de `bone_weight` — o pano da direita"
        );
        g.set_param(canetas[0], "scale", 0.0);
    }));
    let igual = pontos(&corre(PELE_IGUAL, 0, |_| {}));
    assert!(
        !zero.is_empty() && zero.len() == igual.len(),
        "a pele nao produziu peças: {} contra {}",
        zero.len(),
        igual.len()
    );
    // Com todos os ossos a zero a pele não é deformada por nenhum: ela é a grelha de repouso, que
    // é a MESMA dos dois lados — logo a diferença para o pano da esquerda é a deformação inteira.
    let d = maior_desvio(&zero, &igual);
    assert!(
        d > 1e-3,
        "com o quinhao a ZERO a pele ficou igual a' do pano que e' deformado (desvio {d:e}) — \
         o passo 9 do anuncio ensinaria um slider inerte"
    );
}

/// ⭐⭐⭐ **O TECTO DO CAMPO CHEGA PELA CENA** (passo 4) — a W4-bis medida pela porta que o dono usa.
///
/// ⚠️⚠️ **Este gate é irmão e NÃO cópia do `o_campo_chega_ao_tecto_pela_porta_do_produto`**: aquele
/// monta o nó sozinho, este passa pelo `motion.scale`/`motion.move` da cena e prova que o passo
/// escrito no anúncio é executável. *Um tecto que subiu e que o passo do tutorial não alcança não
/// subiu para o artista.*
#[test]
fn o_passo_do_tecto_entrega_meio_milhao_de_celulas() {
    const LADO: f32 = 512.0;
    let s = corre(CAMPO, 0, |g| {
        let w = primeiro(g, "motion.wave");
        g.set_param(w, "rows", LADO);
        g.set_param(w, "cols", LADO);
        g.set_param(w, "spacing", 0.004);
    });
    let n = pontos(&s).len();
    #[expect(clippy::cast_possible_truncation, reason = "um lado de grelha")]
    let esperado = (LADO as usize) * (LADO as usize);
    assert_eq!(
        n, esperado,
        "o artista escreveu {LADO}x{LADO} e recebeu {n} celulas — o passo 4 do anuncio ensinaria \
         um tecto que nao e' o que o nó tem"
    );
}
