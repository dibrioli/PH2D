//! §2 e §3 — achar o pivô, e crescer a cadeia de segmentos.
//!
//! ⭐ **A ideia inteira numa frase:** uma varredura em largura sai do vértice sob
//! o cursor e pára no raio; os vértices do **primeiro anel para lá do raio** —
//! a **franja** — são promediados, e essa média é o pivô. ⚠️ **É a média da
//! FRANJA, não do interior**, e é isso que faz o pivô cair *do lado do corpo*
//! quando o cursor está numa extremidade: a franja de um dedo é quase toda do
//! lado da mão. *Uma implementação que promedie o interior põe o pivô no meio do
//! dedo e o dedo roda em torno de si mesmo.*

use crate::vetor::{add, distancia, escalar, normalizar, sub, V3};
use crate::vizinhanca::Vizinhanca;
use crate::Controlos;

/// Um segmento da cadeia. O estado **inicial** é a referência do traço todo; o
/// estado vivo é reescrito a cada evento (§5.1-bis).
#[derive(Clone, Debug)]
pub struct Segmento {
    pub origem_inicial: V3,
    pub cabeca_inicial: V3,
    pub comprimento: f32,
    /// A origem em que o **evento anterior** deixou o segmento (`O⁻` do §5.1).
    pub origem: V3,
    pub rot: crate::vetor::Rot,
    pub escala: V3,
}

/// A cadeia construída no primeiro evento e **reutilizada até o traço acabar**.
///
/// ⭐⭐ **É aqui que ganhamos ao alvo sem mudar uma vírgula da lei:** o alvo
/// reconstrói tudo isto *a cada movimento do rato*, mesmo sem traço nenhum, só
/// para desenhar o indicador sob o cursor — e é essa a causa registada da má
/// fama de desempenho do pincel (§10, §13, quatro relatos públicos). A cadeia é
/// a mesma do princípio ao fim do traço; construí-la uma vez não é optimização,
/// é ler a §10.
pub struct Cadeia {
    pub segmentos: Vec<Segmento>,
    /// `n_segmentos × n_vertices`, em linha por segmento.
    pub pesos: Vec<f32>,
    pub n_vertices: usize,
    /// `C` — o ponto de aplicação, fixado no primeiro evento.
    pub ancora: V3,
}

impl Cadeia {
    pub fn peso(&self, segmento: usize, v: usize) -> f32 {
        self.pesos[segmento * self.n_vertices + v]
    }
}

impl Segmento {
    /// ⭐ **A direcção que o segmento aponta no início do traço — ou `None`
    /// quando ela é só ruído de representação** (§11.1).
    ///
    /// ⚠️ **Os QUATRO sítios que precisam de uma direcção inicial passam por
    /// aqui** — a rotação da cadeia, o eixo da torção, a normal do plano de
    /// escala e a base local do espremer. *Escrever a guarda em três deles e
    /// esquecer o quarto é como este defeito voltaria, e voltaria só na fixtura
    /// de um modo.*
    pub fn direccao_inicial(&self) -> Option<V3> {
        crate::vetor::direccao_entre_posicoes(self.origem_inicial, self.cabeca_inicial)
    }
}

/// Os centros contra os quais o predicado de região mede: `C` e as imagens
/// espelhadas dele em cada combinação de eixos de simetria activos (§2.2).
pub(crate) fn centros(c: V3, simetria: [bool; 3]) -> Vec<V3> {
    let mut saida = vec![c];
    for eixo in 0..3 {
        if !simetria[eixo] {
            continue;
        }
        let atuais = saida.clone();
        for mut p in atuais {
            p[eixo] = -p[eixo];
            saida.push(p);
        }
    }
    saida
}

/// §2.5 — o teste de lado. ⚠️ **É um predicado de DOIS argumentos, e o segundo
/// muda conforme quem o usa:** na §2.2 é `C`, o ponto de aplicação; na §3.1 é o
/// **alvo corrente** do crescimento. Enunciá-lo sem dizer qual deixa a lei
/// ambígua exactamente onde ela decide o pivô.
pub(crate) fn teste_de_lado(p: V3, q: V3, simetria: [bool; 3]) -> bool {
    for eixo in 0..3 {
        if !simetria[eixo] {
            continue;
        }
        if q[eixo] == 0.0 && p[eixo] > 0.0 {
            return false;
        }
        if p[eixo] * q[eixo] < 0.0 {
            return false;
        }
    }
    true
}

/// §12.1 — ⚠️⚠️ **a comparação é ESTRITA e em `f32`.** Um único vértice a
/// atravessar esta fronteira muda a franja, logo o pivô, logo a deformação
/// inteira — e não por um infinitésimo. Medido: em `f64` a paridade com o
/// oráculo degrada de `1e-7` para `1e-2` nas malhas cujos vértices calham sobre
/// o raio.
fn dentro(p: V3, centros: &[V3], raio: f32) -> bool {
    centros.iter().any(|&c| distancia(p, c) < raio)
}

/// §2.1 — o vértice mais próximo de `C` por busca **sem limite de distância**,
/// saltando escondidos.
pub fn mais_proximo_global(posicoes: &[V3], escondido: &[bool], c: V3) -> Option<u32> {
    let mut melhor: Option<(f32, u32)> = None;
    for (i, &p) in posicoes.iter().enumerate() {
        if escondido.get(i).copied().unwrap_or(false) {
            continue;
        }
        let d = distancia(p, c);
        if melhor.is_none_or(|(md, _)| d < md) {
            melhor = Some((d, i as u32));
        }
    }
    melhor.map(|(_, i)| i)
}

/// A semente da varredura (§2.1): o vértice **eleito**, mais — para cada
/// combinação de eixos de simetria activa — o vértice mais próximo da imagem
/// espelhada dele, **e só se essa distância for menor que `R`**.
///
/// ⚠️ A lista é **ordenada por índice crescente** antes de começar, e isso é
/// exigência de determinismo: a ordem de visita decide qual vértice fica
/// registado como «o mais afastado» (§2.3) em caso de empate.
/// A porta pela qual o gate da ordem da semente lhe chega — a função é interna
/// à lei, e o que o gate afirma é uma propriedade **dela**, não da saída.
#[cfg(test)]
pub(crate) fn semente_para_teste(
    posicoes: &[V3],
    escondido: &[bool],
    eleito: u32,
    simetria: [bool; 3],
    raio: f32,
) -> Vec<u32> {
    semente(posicoes, escondido, eleito, simetria, raio)
}

fn semente(
    posicoes: &[V3],
    escondido: &[bool],
    eleito: u32,
    simetria: [bool; 3],
    raio: f32,
) -> Vec<u32> {
    let mut lista = vec![eleito];
    let p = posicoes[eleito as usize];
    for espelho in centros(p, simetria).into_iter().skip(1) {
        if let Some(v) = mais_proximo_global(posicoes, escondido, espelho)
            && distancia(posicoes[v as usize], espelho) < raio
        {
            lista.push(v);
        }
    }
    lista.sort_unstable();
    lista.dedup();
    lista
}

/// §2.2 + §2.3 — a travessia e a origem.
fn pivo(
    viz: &Vizinhanca,
    posicoes: &[V3],
    escondido: &[bool],
    semente: &[u32],
    c: V3,
    ctrl: &Controlos,
    pesos: &mut [f32],
) -> V3 {
    let centros = centros(c, ctrl.simetria);
    let n = viz.n_vertices();
    let mut visto = vec![false; n];
    let mut fila = std::collections::VecDeque::new();
    let mut soma_franja = [0.0f32; 3];
    let mut n_franja = 0u32;
    let mut mais_afastado: Option<(f32, V3)> = None;
    let mut vizinhos = Vec::new();

    let registar = |v: u32, visto: &mut Vec<bool>, pesos: &mut [f32]| -> bool {
        if visto[v as usize] || escondido.get(v as usize).copied().unwrap_or(false) {
            return false;
        }
        visto[v as usize] = true;
        pesos[v as usize] = 1.0;
        true
    };

    for &s in semente {
        if registar(s, &mut visto, pesos) {
            fila.push_back(s);
        }
    }

    while let Some(v) = fila.pop_front() {
        let p = posicoes[v as usize];
        let d = distancia(p, c);
        if mais_afastado.is_none_or(|(md, _)| d > md) {
            mais_afastado = Some((d, p));
        }
        if !dentro(p, &centros, ctrl.raio) {
            // Franja: o primeiro anel para lá do raio. Não continua a varredura.
            if teste_de_lado(p, c, ctrl.simetria) {
                soma_franja = add(soma_franja, p);
                n_franja += 1;
            }
            continue;
        }
        viz.para_travessia(v, &mut vizinhos);
        for &u in &vizinhos {
            if registar(u, &mut visto, pesos) {
                fila.push_back(u);
            }
        }
    }

    if n_franja > 0 {
        return escalar(soma_franja, 1.0 / n_franja as f32);
    }
    mais_afastado.map_or(c, |(_, p)| p)
}

/// O resultado de uma varredura de crescimento (§3.1).
struct Varredura {
    /// A média das posições dos recém-alcançados que passam no teste de lado.
    media: Option<V3>,
}

/// §3.1 — uma varredura de crescimento, passo de **Jacobi**: lê só o estado
/// anterior.
fn crescer_uma_vez(
    viz: &Vizinhanca,
    posicoes: &[V3],
    escondido: &[bool],
    anterior: &[f32],
    agora: &mut [f32],
    alvo: V3,
    simetria: [bool; 3],
) -> Varredura {
    agora.copy_from_slice(anterior);
    let mut soma = [0.0f32; 3];
    let mut n = 0u32;
    let mut vizinhos = Vec::new();
    for v in 0..viz.n_vertices() {
        if escondido.get(v).copied().unwrap_or(false) {
            continue;
        }
        viz.para_travessia(v as u32, &mut vizinhos);
        let mut m = anterior[v];
        for &u in &vizinhos {
            let w = anterior[u as usize];
            if w > m {
                m = w;
            }
        }
        if m > anterior[v] {
            agora[v] = m;
            let p = posicoes[v];
            if teste_de_lado(p, alvo, simetria) {
                soma = add(soma, p);
                n += 1;
            }
        }
    }
    Varredura {
        media: (n > 0).then(|| escalar(soma, 1.0 / n as f32)),
    }
}

/// §3.2 — crescer até parar, com a regra do uso.
///
/// ⚠️⚠️ **O retrocesso dos pesos é load-bearing:** a varredura que dispara a
/// paragem é **descartada** e é a anterior que fica. Sem isso cada segmento
/// ficaria um anel mais gordo do que deve — e o defeito seria mudo, porque a
/// forma continuaria plausível.
/// A malha e o que nela é constante durante a construção da cadeia.
///
/// ⚠️ Existe porque estes três andam **sempre** juntos: passá-los soltos fazia a
/// assinatura crescer até o clippy a acusar, e uma assinatura de oito
/// argumentos posicionais é onde dois deles se trocam sem ninguém ver.
pub(crate) struct Terreno<'a> {
    pub viz: &'a Vizinhanca,
    pub posicoes: &'a [V3],
    pub escondido: &'a [bool],
}

fn crescer_ate_parar(
    t: &Terreno<'_>,
    pesos: &mut Vec<f32>,
    alvo: V3,
    comprimento_alvo: f32,
    simetria: [bool; 3],
    aproximacao: bool,
) -> V3 {
    let (viz, posicoes, escondido) = (t.viz, t.posicoes, t.escondido);
    let mut agora = pesos.clone();
    let mut distancia_anterior = f32::INFINITY;
    loop {
        let anterior = pesos.clone();
        let r = crescer_uma_vez(
            viz, posicoes, escondido, &anterior, &mut agora, alvo, simetria,
        );
        let Some(media) = r.media else {
            // Não alcançou ninguém: pára, e a origem é o próprio alvo.
            return alvo;
        };
        let d = distancia(media, alvo);
        let continuar = if aproximacao {
            d < distancia_anterior
        } else {
            d < comprimento_alvo
        };
        if !continuar {
            // Os pesos voltam ao estado da varredura ANTERIOR; a média desta
            // varredura é que dá a origem nova.
            *pesos = anterior;
            return if aproximacao { alvo } else { media };
        }
        distancia_anterior = d;
        *pesos = agora.clone();
    }
}

/// Constrói a cadeia inteira (§2–§3). Corre **uma vez por traço** (§10).
pub fn construir(
    viz: &Vizinhanca,
    posicoes: &[V3],
    escondido: &[bool],
    eleito: u32,
    c: V3,
    ctrl: &Controlos,
) -> Cadeia {
    let n = viz.n_vertices();
    // §3 — ⚠️ nos modos de escala e de espremer/esticar a cadeia tem
    // exactamente UM segmento, seja qual for o valor do controlo. *Observado:*
    // as fixturas que pedem `3` no cabeçalho produzem a cadeia de um segmento.
    let n_segmentos = if ctrl.modo.cadeia_de_um_segmento() {
        1
    } else {
        ctrl.segmentos.max(1) as usize
    };

    let terreno = Terreno {
        viz,
        posicoes,
        escondido,
    };
    let mut w = vec![0.0f32; n];
    // §2.1 — o pré-peso do mais-próximo GLOBAL, que é outra grandeza que não o
    // eleito. ⭐ **Medido INERTE neste corpus** (a varredura alcança-o em 69 de
    // 69), e mantido porque as duas só se separam quando ele não é alcançável
    // — a fixtura que as separaria está nomeada e não existe (§12.4).
    if let Some(g) = mais_proximo_global(posicoes, escondido, c) {
        w[g as usize] = 1.0;
    }
    let sementes = semente(posicoes, escondido, eleito, ctrl.simetria, ctrl.raio);
    let mut origem = pivo(viz, posicoes, escondido, &sementes, c, ctrl, &mut w);

    // §2.6 — o desvio da origem, e a compensação por crescimento que o torna
    // utilizável (é a cura publicada do defeito que partia o pincel).
    if ctrl.desvio_da_origem != 0.0 {
        if let Some(dir) = normalizar(sub(origem, c)) {
            origem = add(origem, escalar(dir, ctrl.raio * ctrl.desvio_da_origem));
        }
        crescer_ate_parar(&terreno, &mut w, origem, 0.0, ctrl.simetria, true);
    }

    let comprimento_alvo = ctrl.raio * (1.0 + ctrl.desvio_da_origem);
    let mut origens = vec![origem];
    let mut pesos = vec![0.0f32; n_segmentos * n];
    // §3.3 — o peso de cada segmento é a **diferença** contra o estado depois do
    // segmento anterior ⇒ os segmentos repartem a malha em anéis **disjuntos**.
    pesos[..n].copy_from_slice(&w);
    let mut anterior = w.clone();

    for i in 1..n_segmentos {
        let alvo = origens[i - 1];
        let nova = crescer_ate_parar(
            &terreno,
            &mut w,
            alvo,
            comprimento_alvo,
            ctrl.simetria,
            false,
        );
        origens.push(nova);
        for v in 0..n {
            pesos[i * n + v] = w[v] - anterior[v];
        }
        anterior.copy_from_slice(&w);
    }

    // §3.4 — cabeças, origens e comprimentos.
    let segmentos = origens
        .iter()
        .enumerate()
        .map(|(i, &origem)| {
            let cabeca = if i == 0 { c } else { origens[i - 1] };
            Segmento {
                origem_inicial: origem,
                cabeca_inicial: cabeca,
                comprimento: distancia(cabeca, origem),
                origem,
                rot: crate::vetor::Rot::IDENTIDADE,
                escala: [1.0, 1.0, 1.0],
            }
        })
        .collect();

    Cadeia {
        segmentos,
        pesos,
        n_vertices: n,
        ancora: c,
    }
}
