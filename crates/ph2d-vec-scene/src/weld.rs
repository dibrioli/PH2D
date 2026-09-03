//! ⭐⭐⭐ **SOLDAR** (plano 39) — linhas cruzadas passam a partilhar o nó.
//!
//! Ideia do Enio (2026-08-31): *"e se pudéssemos soldar linhas cruzadas? Ou seja: linhas cruzadas
//! compartilham o mesmo nó de modo que criem várias áreas fechadas interligadas?"*
//!
//! # A lei
//!
//! > **Cada contorno parte-se em ARCOS nos pontos onde encontra os outros** — e as pontas dos arcos
//! > vizinhos caem exactamente no mesmo sítio, porque saem do mesmo cruzamento.
//!
//! O grafo não é uma estrutura de dados: ele é **implícito nas coordenadas coincidentes**. É o
//! modelo do desenho de CAD (um esboço do Fusion é uma rede desde sempre, e é por isso que o Trim
//! de lá parece natural) e a metade barata do *vector network* da Figma.
//!
//! # ⛔ O que isto NÃO é
//!
//! **Não é o `Pathfinder > Divide` do Illustrator.** Aquele *"perde as partes de caminhos abertos
//! que ficam de fora"* — é a queixa documentada dele. Aqui **todo arco sobrevive**, incluindo o toco
//! que sobra para fora: o que sai é a rede inteira, não só as faces.
//!
//! **Não é a rede da Figma.** Lá um ponto é um nó de um multigrafo e sobrevive à edição; aqui a
//! soldadura é um **acto**, e arrastar um nó depois separa outra vez as duas pontas. ⚠️ Manter a
//! rede soldada durante a edição é modelo novo — a Figma conta que refazer o tipo *caminho* teve
//! *"becos sem saída"* e que quase desistiram. **Decisão do Enio: soldar CONSOME os traços.**
//!
//! # ⛔ E não é automático
//!
//! Se cruzar duas linhas as colasse sozinho, seria impossível apenas **sobrepor** dois traços. O
//! gesto é um verbo explícito sobre a selecção.

use crate::VecVertex;

/// Duas fracções mais próximas que isto são o mesmo corte.
const EPS: f64 = 1e-9;

/// **OS ARCOS em que este contorno se parte** nos `cruzamentos` dados (fracções de arco `0..=1`,
/// em qualquer ordem).
///
/// Sem cruzamento nenhum devolve o contorno **intacto** — e intacto quer dizer os mesmos vértices,
/// não uma reconstrução: um caminho que não encontra ninguém não pode pagar o custo de ser cortado
/// e recosturado.
///
/// | o contorno | os cruzamentos | o que sai |
/// |---|---|---|
/// | qualquer | nenhum | ele próprio, intacto |
/// | aberto | `n` | `n + 1` arcos abertos |
/// | fechado | `n` | `n` arcos abertos (o último dá a volta pela emenda) |
///
/// ⚠️ **Um fechado com UM cruzamento vira UM arco aberto** — o anel é cortado num ponto e passa a
/// ter duas pontas, que caem no mesmo sítio. Não é degenerado: é um anel aberto.
#[must_use]
pub fn split_at(
    verts: &[VecVertex],
    closed: bool,
    cruzamentos: &[f64],
) -> Vec<(Vec<VecVertex>, bool)> {
    split_at_fracs(verts, closed, cruzamentos)
        .into_iter()
        .map(|(v, c, _, _)| (v, c))
        .collect()
}

/// ⭐⭐⭐ **O MESMO CORTE, dizendo de que FATIA do contorno cada arco veio** — `(vértices, fecha?,
/// de, até)` em fracções de arco.
///
/// # Porque a proveniência existe
///
/// Ela é a **âncora** com que uma tinta se agarra ao desenho (plano 40 §11). Uma região do balde
/// deixou de ser descrita por *onde ela estava* — uma coordenada, que deriva a cada quadro — e
/// passa a ser descrita pelos **arcos que a cercam**, que são pedaços das curvas que o artista
/// desenhou e que **não derivam**: arrastar um nó move a curva, não muda de que curva o arco é.
///
/// ⚠️ **Num contorno FECHADO o último arco DÁ A VOLTA pela emenda** — ele sai com `de > até`, e quem
/// pergunta *"esta fracção está neste arco?"* tem de saber disso.
///
/// ⚠️ **Sem corte nenhum a fatia é o contorno inteiro** (`0` a `1`), e não uma fatia degenerada.
#[must_use]
pub fn split_at_fracs(
    verts: &[VecVertex],
    closed: bool,
    cruzamentos: &[f64],
) -> Vec<(Vec<VecVertex>, bool, f64, f64)> {
    // ⛔⛔ **Num contorno FECHADO, a fracção `0` é um ponto INTERIOR — a emenda não é fronteira.**
    //
    // A 1.ª redacção filtrava `f > EPS && f < 1-EPS` para os dois casos, e num anel isso **perdia um
    // nó inteiro**: um círculo cruzado exactamente sobre a âncora de partida devolvia o cruzamento
    // em `0.0`, o corte era descartado, e o anel saía com **um** arco em vez de dois. Achado pelo
    // balde (plano 40), em `ellipse` — cuja 1.ª âncora está em `(cx + r, cy)`, que é justamente
    // onde uma recta horizontal pelo centro o corta.
    //
    // ⚠️ **Num contorno ABERTO o filtro continua certo**: ali `0` e `1` são as PONTAS, e cortar uma
    // ponta não parte nada — daria um arco de comprimento zero.
    let mut cortes: Vec<f64> = cruzamentos
        .iter()
        .copied()
        .filter(|f| f.is_finite())
        .map(|f| if closed { f.rem_euclid(1.0) } else { f })
        .filter(|f| closed || (*f > EPS && *f < 1.0 - EPS))
        .collect();
    cortes.sort_by(f64::total_cmp);
    cortes.dedup_by(|a, b| (*a - *b).abs() < EPS);
    if cortes.is_empty() {
        return vec![(verts.to_vec(), closed, 0.0, 1.0)];
    }
    // As FRONTEIRAS dos arcos. Num aberto as duas pontas entram; num fechado a lista fecha-se
    // sobre si mesma e o último arco dá a volta pela emenda.
    let mut arcos = Vec::with_capacity(cortes.len() + 1);
    let mut emitir = |de: f64, ate: f64| {
        if let Some(v) = crate::trim_tool::piece_geometry(verts, closed, de, ate) {
            arcos.push((v, false, de, ate));
        }
    };
    if closed {
        for w in 0..cortes.len() {
            emitir(cortes[w], cortes[(w + 1) % cortes.len()]);
        }
    } else {
        let mut anterior = 0.0;
        for &c in &cortes {
            emitir(anterior, c);
            anterior = c;
        }
        emitir(anterior, 1.0);
    }
    arcos
}

/// ⭐⭐⭐ **A PONTA MUDA-SE PARA O NÓ** — e a alça vai com ela.
///
/// ⚠️ Mover só a âncora mudaria a CURVA em vez de a deslocar, e o arco descolaria da forma que
/// tinha. É a mesma lei do `shift_vert` da edição, escrita aqui porque esta crate não a alcança.
pub fn mover_ponta(v: &mut VecVertex, no: [f64; 2]) {
    let d = [no[0] - v.anchor[0], no[1] - v.anchor[1]];
    v.anchor = no;
    v.in_handle = [v.in_handle[0] + d[0], v.in_handle[1] + d[1]];
    v.out_handle = [v.out_handle[0] + d[0], v.out_handle[1] + d[1]];
}

/// ⭐⭐⭐ **QUAIS PONTAS SÃO O MESMO NÓ** — a porta única da pergunta.
///
/// # Porque não basta cortar
///
/// Report do Enio (2026-08-31, com foto): *"weld dividiu e não soldou (eu que afastei os pontos)"*.
/// E ele estava certo — **cortar não é soldar**. As duas metades de um cruzamento nascem de
/// contornos DIFERENTES: cada um converte a mesma travessia para a **sua** fracção de arco e depois
/// avalia a **sua** cúbica ali. Os dois pontos ficam perto e **não iguais** — e dois pontos perto
/// não são um nó, são dois nós.
///
/// ⚠️ **Uma fixtura que prove isto tem de ser CURVA.** Com duas retas cruzando em coordenadas
/// redondas os dois lados já calculam o MESMO ponto por acaso, e o gate passa com a fusão
/// desligada — foi a primeira redacção dos gates de 31/08, e três mutações sobreviveram a ela.
/// *A fixtura mais azarada possível é a que aprova.*
///
/// Devolve, para cada ponta de entrada, **o índice do nó a que ela pertence** (`None` = está
/// sozinha, e uma ponta sozinha não é junta de nada), mais **a coordenada de cada nó**.
///
/// ⚠️ **O CENTROIDE, e não a primeira**: escolher uma das pontas faria a junta depender da ordem
/// em que os arcos saíram, e a mesma solda dava geometria diferente conforme a ordem da selecção.
///
/// ⚠️⚠️ **Existe separada do [`fuse_endpoints`] porque as pontas de um nó podem vir de DOIS
/// substratos**: de um arco recém-cortado (que vive num vector) e de um caminho que ninguém cortou
/// (que vive na cena, com pose própria e id a preservar). *Duas contas de "quem é o mesmo nó"
/// divergiriam no dia em que uma delas mudasse de tolerância* — aqui a resposta é uma só, e cada
/// chamador escreve o resultado no seu substrato.
#[must_use]
pub fn cluster_endpoints(pontas: &[[f64; 2]], tol: f64) -> (Vec<Option<usize>>, Vec<[f64; 2]>) {
    let t2 = tol * tol;
    let mut de_quem: Vec<Option<usize>> = vec![None; pontas.len()];
    let mut nos: Vec<[f64; 2]> = Vec::new();
    for a in 0..pontas.len() {
        if de_quem[a].is_some() {
            continue;
        }
        let mut grupo = vec![a];
        for (b, alvo) in de_quem.iter().enumerate().skip(a + 1) {
            if alvo.is_some() {
                continue;
            }
            let d = [pontas[a][0] - pontas[b][0], pontas[a][1] - pontas[b][1]];
            if d[0].mul_add(d[0], d[1] * d[1]) <= t2 {
                grupo.push(b);
            }
        }
        if grupo.len() < 2 {
            continue; // uma ponta sozinha não é junta
        }
        let (mut sx, mut sy) = (0.0, 0.0);
        for &g in &grupo {
            sx += pontas[g][0];
            sy += pontas[g][1];
        }
        #[allow(clippy::cast_precision_loss)]
        let n = grupo.len() as f64;
        let no = nos.len();
        nos.push([sx / n, sy / n]);
        for &g in &grupo {
            de_quem[g] = Some(no);
        }
    }
    (de_quem, nos)
}

#[cfg(test)]
#[path = "weld_tests.rs"]
mod tests;
