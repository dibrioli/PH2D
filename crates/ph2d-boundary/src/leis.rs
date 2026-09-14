//! §9, §10 e §11 — **o avanço que a mão produz, as seis leis de deformação, e a
//! aplicação.**
//!
//! Estas são as fases que correm **por passo do traço**; tudo o que está acima
//! delas (A–E) é a fotografia do pen-down.

use crate::estrutura::{Estrutura, SEM_ANEL};
use crate::topologia::Topologia;
use crate::vetor::{V3, add, escalar, normalizar, ponto, rodar, sub};
use crate::{Controlos, Evento, Modo};

/// §9.1 — **o avanço `s`**: a projecção do arrasto na direcção que aponta para
/// DENTRO da peça.
///
/// ⚠️⚠️ **O sinal é o ponto em que se erra.** Arrastar **para dentro** dá
/// `s > 0`; arrastar **para fora** dá `s < 0`. E a consequência medida é que o
/// `EXPAND` anda ao **contrário** da componente de `n̂`: puxar para fora da peça
/// encolhe a borda para dentro. *Não é defeito — é a lei, medida nos dois
/// sentidos.*
///
/// ⭐ **Arrasto TANGENTE ao contorno não faz nada**, e é a prova directa de que
/// só a projecção conta: `0` vértices movidos no corpus.
pub fn avanco(arrasto: V3, direccao: V3) -> f32 {
    ponto(arrasto, direccao)
}

/// §9.2 — a força do traço.
///
/// ⚠️ **Sem pressão, sem inversão de sinal, e SEM elevar ao quadrado** — vários
/// pincéis do alvo usam `Força²`; este não, e está medido: força `0,5` dá
/// deslocamentos exactamente metade.
pub fn forca_do_traco(ctrl: &Controlos) -> f32 {
    ctrl.forca * ctrl.esbatimento_da_simetria
}

/// §9.3 — **o modificador de inversão NÃO inverte a deformação: ele ENCAIXA o
/// ângulo.**
///
/// ```text
/// factor_encaixado = ⌊factor × 10⌋ / 10
/// ```
///
/// ⚠️⚠️ **É truncagem para baixo, e nunca `round` nem `abs`.** Medido: o mesmo
/// traço curto dá máximo `0,266112` normal e `0,283744` com a inversão — *um
/// valor DIFERENTE e MAIOR*, que é a assinatura de uma truncagem sobre um factor
/// **negativo**. ⛔ *Quem o implementar como «arredonda para o décimo mais
/// próximo» diverge; e quem o implementar sobre o valor absoluto diverge no
/// outro sentido.*
pub fn encaixar(factor: f32) -> f32 {
    (factor * 10.0).floor() / 10.0
}

/// O par (ponto, eixo) do `BEND` e a direcção do `EXPAND`, **por coluna**.
///
/// ⚠️ **Indexado pela posição na cadeia**, e não por vértice: guardar três `V3`
/// por vértice da malha custaria `36 B` vezes a malha inteira, e as colunas são
/// tantas quantos os vértices da **borda**.
#[derive(Clone, Debug, Default)]
pub struct PorColuna {
    /// `P₀` do vértice do anel `K` da coluna — o ponto de rotação do `BEND`.
    pub ponto: Vec<V3>,
    /// O eixo do `BEND`, já normalizado.
    pub eixo: Vec<V3>,
    /// A direcção do `EXPAND`, a apontar para **FORA**.
    pub direccao: Vec<V3>,
    /// Onde cada vértice da cadeia está na lista acima — `u32::MAX` fora dela.
    pub indice: Vec<u32>,
}

impl PorColuna {
    /// §10.1 e §10.2 — derivadas UMA vez, no pen-down.
    ///
    /// ⚠️⚠️ **A normal do `BEND` sai do vértice do anel `K`** — não a do vértice
    /// que se move, nem a da borda. Os autores do alvo tiveram um defeito
    /// exactamente aqui, e é por isso que esta linha tem um gate com o nome dela.
    pub fn construir(e: &Estrutura, repouso: &[V3], normais: &[V3]) -> PorColuna {
        let mut p = PorColuna {
            ponto: Vec::with_capacity(e.cadeia.len()),
            eixo: Vec::with_capacity(e.cadeia.len()),
            direccao: Vec::with_capacity(e.cadeia.len()),
            indice: vec![u32::MAX; e.anel.len()],
        };
        for (k, &c) in e.cadeia.iter().enumerate() {
            p.indice[c as usize] = u32::try_from(k).unwrap_or(u32::MAX);
            let cadeia_p = repouso.get(c as usize).copied().unwrap_or([0.0; 3]);
            let Some(fundo) = e.fundo_da_coluna(c) else {
                // A coluna não chega ao anel `K` — os dados ficam neutros, e a
                // lei que os lê não move a coluna. (A nossa divergência da
                // §13.1 torna este caso raro, mas não impossível: uma coluna
                // pode ser mais curta que a da âncora.)
                p.ponto.push(cadeia_p);
                p.eixo.push([0.0, 0.0, 0.0]);
                p.direccao.push([0.0, 0.0, 0.0]);
                continue;
            };
            let fundo_p = repouso.get(fundo as usize).copied().unwrap_or([0.0; 3]);
            let braco = sub(cadeia_p, fundo_p);
            let n = normais
                .get(fundo as usize)
                .copied()
                .unwrap_or([0.0, 0.0, 1.0]);
            p.ponto.push(fundo_p);
            p.eixo
                .push(normalizar(crate::vetor::cruz(braco, n)).unwrap_or([0.0; 3]));
            p.direccao.push(normalizar(braco).unwrap_or([0.0; 3]));
        }
        p
    }
}

/// §10 e §11 — calcula a posição nova de cada vértice e aplica-a como
/// **translação**.
///
/// Devolve quantos vértices se moveram.
#[allow(clippy::too_many_arguments)]
pub fn aplicar(
    e: &Estrutura,
    topo: &Topologia,
    ctrl: &Controlos,
    ev: &Evento,
    pesos: &[f32],
    colunas: &PorColuna,
    repouso: &[V3],
    normais: &[V3],
    direccao_do_avanco: V3,
    centro_do_torcer: V3,
    contacto: V3,
    posicoes: &mut [V3],
) -> usize {
    let f = forca_do_traco(ctrl);
    let s = avanco(ev.arrasto, direccao_do_avanco);
    let mut factor = f * s / ctrl.raio_dinamico.max(1e-9);
    if ctrl.encaixar_angulo {
        factor = encaixar(factor);
    }
    let angulo_total = std::f32::consts::PI * factor;
    let eixo_do_torcer = normalizar(sub(
        e.ponto_origem,
        repouso.get(e.ancora as usize).copied().unwrap_or([0.0; 3]),
    ))
    .unwrap_or([0.0; 3]);

    // ⭐⭐ **O `SMOOTH` lê as posições ACTUAIS** (§10.6.2) e por isso acumula ao
    // longo do traço. ⚠️ **A média é de JACOBI — todas contra o mesmo estado** —
    // e isso é uma **decisão declarada**: escrever em cima enquanto se percorre
    // faria o resultado depender da ordem dos índices, e a espec não fixa
    // ordem nenhuma. *Uma lei cuja saída depende da ordem de visita não é
    // determinística sob uma renumeração da malha.*
    let medias = (ctrl.modo == Modo::Suavizar).then(|| medias_do_anel(e, topo, posicoes));

    let mut movidos = 0usize;
    for v in 0..pesos.len() {
        let w = pesos[v];
        // ⚠️⚠️ **PESO ZERO TEM DE VIRAR TRANSLAÇÃO ZERO, explicitamente**
        // (§11.2) — e não basta que o peso zere o termo da lei: nas leis que
        // rodam à volta de um ponto, `P(v)` com ângulo `0` é `P₀(v)`, que **não
        // é** a posição actual se um passo anterior (ou uma passagem de
        // simetria) já a moveu ⇒ a translação seria `P₀(v) − actual(v)`,
        // **desfazendo** o trabalho já feito. Os autores do alvo pagaram
        // exactamente este defeito.
        if w == 0.0 || e.anel[v] == SEM_ANEL {
            continue;
        }
        // §12.2 — cada passagem de simetria só toca a sua região.
        if !na_regiao(ctrl, contacto, repouso[v]) {
            continue;
        }
        let p0 = repouso[v];
        let actual = posicoes[v];
        let nova = match ctrl.modo {
            Modo::Dobrar => {
                let Some(k) = coluna(colunas, e, v) else {
                    continue;
                };
                rodar(p0, colunas.ponto[k], colunas.eixo[k], angulo_total * w)
            }
            Modo::Expandir => {
                let Some(k) = coluna(colunas, e, v) else {
                    continue;
                };
                add(p0, escalar(colunas.direccao[k], f * s * w))
            }
            Modo::Inflar => {
                let n = normais.get(v).copied().unwrap_or([0.0; 3]);
                add(p0, escalar(n, f * s * w))
            }
            // ⭐ **O único modo que usa o VECTOR `Δ` e não o escalar `s`** — e
            // portanto o único cujo movimento segue a mão em qualquer direcção,
            // inclusive a tangente.
            Modo::Agarrar => add(p0, escalar(ev.arrasto, f * w)),
            Modo::Torcer => rodar(p0, centro_do_torcer, eixo_do_torcer, angulo_total * w),
            Modo::Suavizar => {
                let Some(m) = medias.as_ref().and_then(|m| m[v]) else {
                    // §10.6: sem vizinho no mesmo anel, o peso é **zerado**.
                    continue;
                };
                // ⚠️ Ele lê a posição ACTUAL e **não usa o avanço `s`** — a
                // força do traço entra directamente. ⇒ ele actua já no primeiro
                // passo, quando os outros cinco estão parados.
                add(actual, escalar(sub(m, actual), f * w))
            }
        };
        let t = sub(nova, actual);
        if t != [0.0; 3] {
            posicoes[v] = add(actual, t);
            movidos += 1;
        }
    }
    movidos
}

/// §12.2 — o filtro por região.
///
/// ⭐ **Com queda `CONSTANT` as duas passagens escrevem o mesmo valor e a
/// ausência do filtro não se nota; com `RADIUS`/`LOOP` elas escrevem valores
/// diferentes e a segunda APAGA a primeira.** Os autores do alvo acharam isto no
/// dia seguinte ao lançamento do pincel. ⇒ *o gate desta lei usa `RADIUS`, nunca
/// `CONSTANT`.*
fn na_regiao(ctrl: &Controlos, pivo: V3, p: V3) -> bool {
    for (i, &espelhado) in ctrl.simetria.iter().enumerate() {
        if !espelhado {
            continue;
        }
        if pivo[i] == 0.0 {
            if p[i] > 0.0 {
                return false;
            }
        } else if p[i] * pivo[i] < 0.0 {
            return false;
        }
    }
    true
}

fn coluna(c: &PorColuna, e: &Estrutura, v: usize) -> Option<usize> {
    let patrono = *e.patrono.get(v)?;
    let k = *c.indice.get(patrono as usize)?;
    (k != u32::MAX).then_some(k as usize)
}

/// §10.6 — **a média SÓ com os vizinhos do MESMO anel**.
///
/// ⭐ É isto que faz o modo alisar **ao longo** do contorno e não **através**
/// dele: numa grelha regular plana os vértices do meio de cada anel não se
/// movem (a média dos dois vizinhos do mesmo anel é o próprio vértice), e só as
/// PONTAS das fileiras andam — a fileira contrai-se sobre si mesma.
fn medias_do_anel(e: &Estrutura, topo: &Topologia, posicoes: &[V3]) -> Vec<Option<V3>> {
    let mut saida = vec![None; posicoes.len()];
    for (v, destino) in saida.iter_mut().enumerate() {
        let anel = e.anel[v];
        if anel == SEM_ANEL {
            continue;
        }
        let mut soma = [0.0f32; 3];
        let mut n = 0u32;
        for &u in topo.vizinhos(u32::try_from(v).unwrap_or(u32::MAX)) {
            if e.anel.get(u as usize).copied() == Some(anel) {
                soma = add(soma, posicoes[u as usize]);
                n += 1;
            }
        }
        if n > 0 {
            *destino = Some(escalar(soma, 1.0 / n as f32));
        }
    }
    saida
}
