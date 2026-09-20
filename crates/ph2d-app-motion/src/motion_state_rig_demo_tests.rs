//! Os gates da cena `=120` — a do ciclo 9 (doc 114 §10).
//!
//! ⚠️⚠️ **Uma cena de smoke que ensina o CONTRÁRIO do que acontece é pior que uma cena ausente**
//! (`CLAUDE.md` §5.0). O anúncio promete nove coisas; cada uma que se pode medir é um gate aqui —
//! e as que se medem por PARES têm o controlo dentro do próprio gate, porque *«mexeu»* não separa
//! a lei de um grafo diferente.
//!
//! ⚠️ **Esta cena não tem membranas** (ao contrário da `=119`): as seis cadeias fabricam a
//! CORRENTE a partir de params, logo não há dados de fora a publicar. *A ausência é uma
//! propriedade do grupo — um esqueleto não vem de fora.*
//!
//! ⛔⛔ **E a frase que aqui estava — *«logo não há nada a publicar antes de cozer»* — MORREU em
//! 2026-09-20**, quando a cena passou a vestir cada pano com uma forma. Ela era verdade sobre as
//! MEMBRANAS e foi lida como verdade sobre o `publish` inteiro: a geometria de um `source.shape` é
//! gerada no QUADRO (`motion_shape_gen`), não no cozimento, logo **todo** arnês desta cena tem de
//! a publicar. *Uma ausência afirmada sobre uma categoria (dados de fora) passa a mentir no dia em
//! que uma segunda categoria (geometria de forma) entra pela mesma porta.*

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::Graph;

/// Os seis panos, pela ordem em que [`build`] os empilha.
pub(super) const CORDA: usize = 0;
pub(super) const CAMPO: usize = 1;
pub(super) const FK: usize = 2;
const IK: usize = 3;
const PELE_IGUAL: usize = 4;
const PELE_QUINHAO: usize = 5;

/// Quantos tiques correr antes de ler, quando o gate precisa da cena EM REGIME. ⚠️ A corda e o
/// campo nascem parados: medir no tique zero mediria o repouso, que é o que a cena mostra ANTES
/// do `PLAY`.
pub(super) const TIQUES: usize = 40;
pub(super) const DT: f64 = 1.0 / 60.0;

/// **A BARRA DO QUE SE VÊ**, em unidades de mundo — quase três peças da pele (`PELE_PECA = 0,11`).
///
/// ⛔⛔ **Ela nasceu de um defeito que a suíte VERDE não via.** Os gates da pele pediam só
/// `d > 1e-3` — *«os dois panos diferem»* —, e a primeira redacção do envelope passava-os com
/// `0,39` do lado de UMA peça de diferença: duas figuras que o leitor do tutorial lê como
/// **iguais**, debaixo de duas legendas que prometem coisas diferentes. ⚠️ *Uma régua que só vê o
/// SINAL não vê a MAGNITUDE* — a mesma família do `edge_max` cego ao quad fino —, e quem a apanhou
/// foi olhar para a imagem, não correr a suíte.
///
/// ⭐ Com a banda no lugar da rampa o par mede `0,63` de mundo (`5,7` peças), logo a barra tem
/// folga de `2×` e o que ela proíbe é a REGRESSÃO ao invisível.
const VISIVEL: f32 = 0.3;

/// **A BANDA que o app de facto desenha**, em unidades de mundo — MEDIDA numa fotografia.
///
/// ⚠️⚠️ **Ela não é a janela.** Com a ferramenta Motion na mão o chrome da cena desenha num
/// sub-rectângulo entre a barra do topo e a timeline. Lida numa foto de `1930×1040` com a
/// arrumação de fábrica, a escala sai das PRÓPRIAS fichas da cena (duas fichas separadas por
/// `ROW_GAP` de mundo ocupavam `178 px` ⇒ `55,6 px/unidade`) e a banda ocupava `482 px`.
///
/// ⛔ **E ela não é centrada na origem** — o que fez a fileira de cima sair pelo topo enquanto
/// sobrava espaço em baixo.
const BANDA_ALTURA: f32 = 8.665;
const BANDA_CENTRO: f32 = -0.927;
/// ⚠️ **Pequena de propósito:** os dois números acima vêm de UMA janela, e uma maior dá mais
/// banda — logo eles são conservadores. O que este gate proíbe é a regressão ao invisível.
const MARGEM_DA_BANDA: f32 = 0.15;

/// O primeiro nó de um dado tipo no grafo.
///
/// ⚠️ **A cena tem UM de cada um dos tipos que estes gates procuram** (`motion.falloff` e
/// `motion.wave` vivem cada um num pano só), e os gates que mexem numa CANETA — de que há
/// quatro — usam a [`caneta_da_coluna`], que desempata pela coluna escrita.
pub(super) fn primeiro(g: &Graph, tipo: &str) -> NodeId {
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
    // ⛔⛔ **Sem esta linha o `source.shape` coze VAZIO, e com ele todos os seis panos.** A
    // geometria de uma forma é gerada no QUADRO (`motion_shape_gen`), não no cozimento — logo um
    // arnês headless que salte o passo mede um grafo em que a porta `shape` do duplicador não
    // entrega nada. ⚠️ *O modo de falha é o pior possível: nenhum erro, contagem zero, e os gates
    // que leem `P` acusam a FONTE de não ter entregado nada.* É a armadilha 2 do
    // [doc 115 §32.1](../../docs/Motion%20Nodes/115_o_colisor_sai_do_grafo.md), e o padrão já
    // vive em cinco sítios desta crate.
    crate::motion_shape_gen::publish(&mut m, 0.0);
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

pub(super) fn pontos(s: &Stream) -> Vec<[f32; 2]> {
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
    // ⚠️ A mesma linha do [`corre`], pela mesma razão: sem ela o `source.shape` de cada pano coze
    // vazio e os seis leem `0` peças — e a mensagem deste gate acusaria a FONTE de cada cadeia.
    crate::motion_shape_gen::publish(&mut m, 0.0);
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
        d > VISIVEL,
        "os dois panos de BAIXO diferem {d:e}, abaixo dos {VISIVEL} que se VEEM — \
         o quinhao por osso ou nao chegou a' pele, ou chegou onde ela ja' nao se mexia"
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
        // ⭐ A alavanca do quinhão é o FIM DA BANDA: a `0` nenhuma junta cai dentro dela, logo
        // todos os ossos ficam com quinhão zero e nenhum puxa. ⚠️ **Não é a `scale` da caneta** —
        // ali o valor já vem do campo, e mexer nela mediria outra coisa.
        let canetas = caneta_da_coluna(g, "bone_weight");
        assert_eq!(
            canetas.len(),
            1,
            "a cena tem de ter UMA caneta de `bone_weight` — o pano da direita"
        );
        let banda = primeiro(g, "field.index_range");
        g.set_param(banda, "end", 0.0);
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
        d > VISIVEL,
        "com o quinhao a ZERO a pele difere {d:e} da que e' deformada, abaixo dos {VISIVEL} que \
         se VEEM — o passo 10 do anuncio ensinaria um controlo inerte"
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

/// ⭐⭐⭐ **CADA PANO CHEGA A PIXEL** — a pergunta que os oito gates acima NÃO fazem.
///
/// ⛔⛔⛔ **Esta cena esteve a desenhar NADA durante uma jornada inteira, com a suíte VERDE.**
/// Em 2026-09-19 o dono mandou retirar os gizmos dos nós que só passam posições, e a lei que
/// ficou no lugar deles ([`ph2d_eval_motion::tem_aparencia`], ligada por omissão desde então)
/// diz que *uma corrente que não veio de uma forma não vira pixel*. A cena é de 17/09: as seis
/// cadeias acabavam num `motion.output` sem uma única forma no caminho.
///
/// ⚠️⚠️ **Os outros oito gates leem `P`, e `P` continuava perfeito** — a corda balançava, o IK
/// seguia o alvo, o quinhão separava as duas peles. *Eles medem o que a cena CALCULA; nenhum
/// perguntava se ela se VÊ*, que é a única coisa que o dono pode julgar. É a forma exacta que o
/// `CLAUDE.md` §5.0 chama de *«o consumidor que PROJECTA o valor fora»*, um andar acima: aqui o
/// consumidor é o renderer, e o que ele descarta é o pano inteiro.
///
/// FALSIFICADO por tirar o `source.shape`/`motion.duplicator` de um pano só — o `k` da mensagem
/// nomeia qual.
#[test]
fn cada_pano_veste_uma_forma_e_por_isso_desenha() {
    for (k, nome) in [
        (CORDA, "a corda"),
        (CAMPO, "o campo"),
        (FK, "o FK"),
        (IK, "o IK"),
        (PELE_IGUAL, "a pele igual"),
        (PELE_QUINHAO, "a pele com quinhao"),
    ] {
        let s = corre(k, 1, |_| {});
        assert!(
            ph2d_eval_motion::tem_aparencia(&s),
            "o pano {k} ({nome}) nao tem aparencia: o renderer descarta-o inteiro"
        );
    }
}

/// ⭐ **A CADEIA DO PANO DA CORDA, ETAPA A ETAPA** — o gate que diz ONDE, quando o irmão diz QUE.
///
/// ⚠️ **Ele nasceu como uma sonda que IMPRIMIA e passava sempre**, e ficou assim tempo suficiente
/// para encontrar o defeito de arnês que a migração desta cena pagou (o `source.shape` a cozer
/// `0`). *Uma tabela impressa que ninguém lê é a forma que este repo já pagou meia dúzia de
/// vezes*, então ela afirma: cada etapa entrega a contagem que a lei manda, e o `source.shape`
/// entrega **uma** forma — se ele entregar zero, o duplicador não tem o que copiar e os SEIS panos
/// da cena ficam vazios de uma só vez.
///
/// FALSIFICADO por qualquer etapa devolver zero (ou o `publish` sair do [`corre`]).
#[test]
fn a_cadeia_de_um_pano_entrega_em_cada_etapa() {
    let mut m = MotionState::new();
    let _ = build(&mut m.doc, &m.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut m, 0.0);
    #[expect(clippy::cast_possible_truncation, reason = "vinte pontos de corda")]
    let pontos_da_corda = CORDA_PONTOS as usize;
    // ⚠️⚠️ **A PREMISSA MUDOU em 2026-09-20 e a mudança está aqui à vista:** este gate dizia
    // `esperado` em todas as etapas, e a etapa do `motion.scale` passou a ler **`n − 1`**. Não é
    // uma barra afrouxada — é a corda a vestir-se de SEGMENTOS: o `rig.bones` entra entre ela e o
    // carimbo, e *uma corrente de `n` juntas tem `n − 1` ossos* (a raiz é a única junta sem osso a
    // chegar), exactamente como nas duas fileiras do meio. ⭐ Ler `20` depois do `rig.bones` seria
    // o defeito, não o contrário.
    let segmentos = pontos_da_corda - 1;
    for (tipo, quantos) in [
        ("motion.verlet_rope", pontos_da_corda),
        ("rig.bones", segmentos),
        ("motion.scale", segmentos),
        ("motion.move", segmentos),
        // ⚠️ UMA forma: ela é o molde, e o duplicador é quem a multiplica pelos pontos.
        ("source.shape", 1),
        ("motion.duplicator", segmentos),
        ("motion.output", segmentos),
    ] {
        let n = primeiro(&m.doc.graph, tipo);
        let r = m
            .pump
            .cook
            .cook(&m.doc.graph, &m.registry, n, 0.0)
            .unwrap_or_else(|e| panic!("o `{tipo}` nao coze: {e:?}"));
        assert_eq!(
            pontos(r[0].as_stream()).len(),
            quantos,
            "o `{tipo}` devia entregar {quantos} — se for ZERO, e' aqui que a cadeia se perde"
        );
    }
}

/// ⭐⭐ **A FILEIRA DO MEIO ENTREGA OSSOS, E NÃO JUNTAS.**
///
/// ⛔⛔ **Sem o `rig.bones` a `Shape: Bone` desenha-se UMA JUNTA À FRENTE.** A lei do `fk::resolve`
/// é `P[i] = P[pai] + len[i]·(cos wrot[i], sin wrot[i])`: o `len`/`rot` que o elemento `i` carrega
/// são os do osso que **CHEGA** a ele, e o `P[i]` dele é a **PONTA** desse osso ⇒ carimbar a forma
/// na junta põe cada peça pelo lado de fora do arco, com a última pendurada para lá da corrente
/// (medido no cabeçalho do [`ph2d_node_rig_bones`], com a tabela).
///
/// ⚠️ **E isso é INVISÍVEL a todos os outros gates desta cena:** eles comparam os dois panos do
/// meio um com o outro, e sem o nó os DOIS ficam errados da mesma maneira. A régua que o separa é
/// a CONTAGEM — uma corrente de `n` juntas tem `n − 1` ossos, porque a raiz é a única junta sem
/// osso a chegar a ela.
///
/// FALSIFICADO por tirar o `ossos_de` de qualquer um dos dois panos.
#[test]
fn os_dois_panos_do_meio_entregam_ossos_e_nao_juntas() {
    #[expect(clippy::cast_possible_truncation, reason = "cinco juntas")]
    let juntas = OSSOS_JUNTAS as usize;
    for (k, nome) in [(FK, "o FK"), (IK, "o IK")] {
        let n = pontos(&corre(k, 1, |_| {})).len();
        assert_eq!(
            n,
            juntas - 1,
            "{nome} entregou {n} pecas: uma corrente de {juntas} juntas tem {} OSSOS, e sem o \
             `rig.bones` a forma cai uma junta a' frente",
            juntas - 1
        );
    }
}

/// ⭐⭐⭐ **A CENA CABE NA BANDA QUE O APP DESENHA** — o gate que a FOTO escreveu.
///
/// ⛔⛔⛔ **Ela não cabia, e nenhum dos gates acima o podia ver:** os catorze medem o que a cena
/// CALCULA (posições, tamanhos, diferenças entre panos) e a pergunta aqui é *o dono vê isto?*.
/// Fotografada (`docs/Components/ferramentas/fotografa_cena.sh`, `1930×1040`, arrumação de
/// fábrica), a fileira de CIMA saía pelo topo e **as duas fichas dela eram invisíveis** — o passo
/// 2 do tutorial fala de dois panos que o artista não consegue nomear.
///
/// ⚠️⚠️ **A banda NÃO é a janela e NÃO é centrada na origem.** Com a ferramenta Motion na mão o
/// chrome desenha num sub-rectângulo entre a barra do topo e a timeline; lida a escala pelas
/// PRÓPRIAS fichas da cena (`ROW_GAP` de mundo contra os píxeis que as separam), ela mostra
/// [`BANDA_ALTURA`] unidades centradas em [`BANDA_CENTRO`]. *Uma cena enquadrada contra a janela
/// é uma cena enquadrada contra uma superfície em que ela não é desenhada.*
///
/// ⚠️ **Os dois números são de UMA fotografia e dizem-no de si mesmos.** Eles são conservadores
/// por construção (uma janela maior dá mais banda), e a margem exigida é deliberadamente pequena:
/// o que este gate proíbe é a REGRESSÃO ao invisível, não uma disposição em particular.
///
/// FALSIFICADO por repor o `ROW_GAP` em `3,2`, ou por apagar o [`CENA_Y`].
#[test]
fn a_cena_cabe_na_banda_que_o_app_desenha() {
    let (lo, hi) = extensao_da_cena();
    let (banda_lo, banda_hi) = (
        BANDA_CENTRO - BANDA_ALTURA / 2.0,
        BANDA_CENTRO + BANDA_ALTURA / 2.0,
    );
    assert!(
        hi <= banda_hi - MARGEM_DA_BANDA,
        "a cena sai pelo TOPO: {hi:+.3} contra {:+.3} (a ficha da fileira de cima fica invisivel)",
        banda_hi - MARGEM_DA_BANDA
    );
    assert!(
        lo >= banda_lo + MARGEM_DA_BANDA,
        "a cena sai por BAIXO: {lo:+.3} contra {:+.3}",
        banda_lo + MARGEM_DA_BANDA
    );
    // ⭐ **O CONTROLO:** sem ele um `ROW_GAP` minúsculo passaria — e a cena seria ilegível por
    // outro motivo. *Um gate que só proíbe «grande demais» aprova «pequeno demais».*
    assert!(
        hi - lo > BANDA_ALTURA * 0.7,
        "a cena encolheu demais ({:.3} de {BANDA_ALTURA:.3}): tres fileiras num canto nao se leem",
        hi - lo
    );
}

/// A extensão vertical de tudo o que a cena desenha — os seis panos (com a meia-peça de cada um)
/// e as seis fichas.
///
/// ⚠️ **A meia-peça entra na conta**: uma posição não é um ponto na tela, é o centro de uma forma
/// que abrange `2 × size` — e foi exactamente essa diferença que pôs a fileira de cima fora.
fn extensao_da_cena() -> (f32, f32) {
    let mut m = MotionState::new();
    let sinks = build(&mut m.doc, &m.registry).expect("a cena monta");
    crate::motion_shape_gen::publish(&mut m, 0.0);
    let mut t = 0.0f64;
    for _ in 0..TIQUES {
        for s in &sinks {
            let _ = m.pump.cook.cook(&m.doc.graph, &m.registry, *s, t);
        }
        let _ = m.pump.cook.advance_tick(&m.doc.graph, &m.registry, t);
        t += DT;
    }
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for s in &sinks {
        let st = m
            .pump
            .cook
            .cook(&m.doc.graph, &m.registry, *s, t)
            .expect("coze")[0]
            .as_stream()
            .clone();
        let meia = match st.get("size") {
            Some(Column::Vec2(v)) => v.iter().map(|q| q[1].abs()).fold(0.0f32, f32::max),
            _ => 0.0,
        };
        for q in pontos(&st) {
            lo = lo.min(q[1] - meia);
            hi = hi.max(q[1] + meia);
        }
    }
    for c in captions() {
        lo = lo.min(c.world[1]);
        hi = hi.max(c.world[1]);
    }
    (lo, hi)
}

/// **A SONDA do enquadramento** — para quem tiver de mexer nos números.
///
/// ⚠️ **Ela lê a MESMA [`extensao_da_cena`] que o gate.** Escrita por si própria, seria uma
/// segunda resposta à mesma pergunta — e a que o humano lê não seria a que o portão mede.
#[test]
#[ignore = "sonda de enquadramento"]
fn diag_a_extensao_da_cena() {
    let (lo, hi) = extensao_da_cena();
    for c in captions() {
        println!("ficha : y {:+.3}  {}", c.world[1], c.text);
    }
    println!(
        "CENA  : y {lo:+.3} .. {hi:+.3}   altura {:.3}   centro {:+.3}",
        hi - lo,
        (lo + hi) / 2.0
    );
    println!(
        "BANDA : y {:+.3} .. {:+.3}   altura {BANDA_ALTURA:.3}   centro {BANDA_CENTRO:+.3}",
        BANDA_CENTRO - BANDA_ALTURA / 2.0,
        BANDA_CENTRO + BANDA_ALTURA / 2.0
    );
}
