//! **TOPOLOGIA DINÂMICA** — o traço acrescenta detalhe onde o pincel toca.
//!
//! A promessa é a do SculptGL, e ela é sobre *não ficar preso a uma resolução*:
//! você começa numa esfera grosseira, puxa um dedo, e a malha ganha triângulos
//! exatamente ali — em vez de subdividir o modelo inteiro para poder detalhar
//! uma unha.
//!
//! # A lei, numa frase
//!
//! Uma aresta mais longa que o alvo, **dentro da esfera do dab**, é partida ao
//! meio; toda face que toca uma aresta partida é re-triangulada. A segunda
//! metade não é detalhe de implementação — é o que torna o refino
//! **livre de rachaduras por construção**: a vizinha que ninguém pediu para
//! refinar *tem* de aprender o vértice novo, senão sobra um T-vértice e a malha
//! abre um buraco fino que só aparece na luz.
//!
//! # O alvo de aresta é DERIVADO do raio, e a fórmula é a da referência
//!
//! `d2Max = radius² × (1.1 − detail) × 0.2` (`SculptBase.js::dynamicTopology`).
//! Ou seja: o detalhe que o artista escolhe é uma **fração do pincel**, nunca um
//! comprimento de mundo — um pincel pequeno detalha fino e um grande detalha
//! grosso, que é o que a mão espera. Ver [`edge_target`].
//!
//! # Duas recusas, e as duas são geometria
//!
//! * **Quads não entram** ([`Refine::NotTriangles`]): partir uma aresta de um
//!   quad devolve um triângulo e um pentágono. O SculptGL resolve isto
//!   triangulando ao entrar no modo (`MeshDynamic`, *"triangles only"*), e é o
//!   que [`crate::Mesh::triangulate`] faz — a recusa aqui existe para o motor
//!   nunca depender de quem chamou ter lembrado.
//! * **Alvo não-positivo** é no-op: um alvo zero pediria infinitos vértices, e
//!   *"o app travou"* é a pior forma de dizer que um número está errado.
//!
//! # O que esta wave NÃO faz, e o número que decide quando fará
//!
//! ⚠️ **Cada passe termina num [`crate::Mesh::rebuild`] inteiro** — anéis,
//! octree e normais. O plano chamava a estrutura mutável de *pré-requisito*, e
//! a medição (`tests/measure_dyntopo.rs`) diz que ela é a wave SEGUINTE:
//!
//! | vértices | rebuild | cabe no dab (8 ms)? |
//! |---|---|---|
//! | 6 050 | 0,59 ms | sim |
//! | 24 386 | 1,55 ms | sim |
//! | 97 922 | 5,50 ms | sim, apertado |
//!
//! E o mesmo dab toca **0,33% das faces** a 98k ⇒ o `rebuild` faz ~300× o
//! trabalho que a mudança pede. Ele cabe hoje e **deixa de caber** quando o
//! artista cruzar ~100k vértices — que é exatamente o que a topologia dinâmica
//! existe para fazer. O gatilho é um número medido, não um palpite.

use crate::adjacency::Adjacency;
use crate::edges::EdgeIds;
use crate::face::Face;
use crate::mesh::{Mesh, RegionScratch, VertexAppend};

/// **DE ONDE UM VÉRTICE NOVO VEIO** — o par cuja aresta foi partida para criá-lo.
///
/// ⚠️ **A parentela não é diagnóstico: ela é o que torna o refino compatível com
/// um traço em voo.** Um vértice que nasce no meio de um gesto herda a posição
/// média dos pais *como eles estão AGORA*, ou seja **já deslocados** pelo traço;
/// quem só recebesse a contagem nova o trataria como nunca-visto e o deslocaria
/// outra vez, contando o mesmo deslocamento duas vezes. Medido: o desvio de
/// guarda-chuva do p99 vai de 0,31 (o traço sem refino) para 0,61 — a superfície
/// de AGULHAS que o smoke de 2026-08-04 reprovou.
///
/// Quem sabe *onde ele estava antes* é o par: o ponto médio dos `pre` deles.
/// Ver `SculptStroke::grow_with`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Birth {
    /// O índice do vértice novo.
    pub vert: u32,
    /// Os dois extremos da aresta partida.
    pub a: u32,
    pub b: u32,
}

/// O que o refino fez, ou por que não fez nada.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refine {
    /// Refinou. Os números são o que **cresceu**, não o total.
    Done {
        verts_added: usize,
        faces_added: usize,
        passes: usize,
    },
    /// Nenhuma aresta na esfera passou do alvo — a malha já tem a densidade
    /// pedida ali. É o desfecho NORMAL do meio de um traço.
    Enough,
    /// A malha tem quads. Ver o cabeçalho.
    NotTriangles,
}

/// **O alvo de comprimento de aresta**, em unidades de OBJETO.
///
/// `detail` anda em `[0, 1]`: `0` é o mais grosso, `1` o mais fino. A fórmula é
/// a do `SculptBase.js` (`d2Max = radius² × (1.1 − sub) × 0.2`), tirada da raiz
/// para virar comprimento — a razão de a raiz vir aqui e não no chamador é que
/// **o quadrado é detalhe do laço**, e um alvo em comprimento é o que se pode
/// comparar com uma aresta num log ou num gate.
///
/// Os extremos, para quem for reafinar: `detail = 0` dá `0,469 × raio` e
/// `detail = 1` dá `0,141 × raio` — o mais fino que a referência oferece é
/// cerca de **um sétimo do pincel**.
#[must_use]
pub fn edge_target(radius: f32, detail: f32) -> f32 {
    let d = detail.clamp(0.0, 1.0);
    radius * ((1.1 - d) * 0.2).sqrt()
}

/// Quantos passes um único dab pode gastar.
///
/// ⚠️ **É um teto de RECURSO, e o recurso é o quadro:** cada passe custa um
/// `rebuild`, e um dab que gastasse dez deles perderia o frame. Três é o que
/// leva uma aresta de `8×` o alvo até ele (cada passe a divide por dois), e uma
/// aresta assim é a que aparece quando o artista aumenta o detalhe de uma vez
/// no meio do traço — o caso raro. O comum termina em **um**.
const MAX_PASSES: usize = 3;

/// **Refina a malha dentro da esfera** até nenhuma aresta ali passar do alvo.
///
/// `center` e `radius` são do espaço da MALHA (o dab já os traz assim), e o
/// `edge_max` sai do [`edge_target`] — ele é passado em vez de derivado aqui
/// porque quem conhece o `detail` é o dono do pincel, e derivá-lo duas vezes é
/// como o log e a geometria passam a discordar.
///
/// ⚠️ **`births` é LIMPO e preenchido com um [`Birth`] por vértice novo, na
/// ordem em que eles nascem.** A ordem é load-bearing: um passe pode partir uma
/// aresta cujo extremo é um vértice do passe ANTERIOR, então quem consome a
/// lista tem de a percorrer para a frente para que o pai já esteja resolvido.
///
/// ⚠️ **É parâmetro obrigatório, e não um `Option` nem uma porta irmã.** A lição
/// é do `with_arc_len` do Painter 2D: um canal opcional chegava em 2 de 7 rotas e
/// a feature simplesmente não acontecia nas outras cinco, em silêncio. Aqui o
/// preço de esquecer não é um efeito ausente — é a superfície de agulhas do
/// [`Birth`].
pub fn refine_in_sphere(
    mesh: &mut Mesh,
    center: [f32; 3],
    radius: f32,
    edge_max: f32,
    births: &mut Vec<Birth>,
    scratch: &mut RegionScratch,
) -> Refine {
    refine_in_sphere_sized(mesh, center, radius, edge_max, None, births, scratch)
}

/// ⭐⭐⭐ **O MESMO, com um LIMIAR QUE VARIA COM O SÍTIO** — ver [`crate::Sizing`].
///
/// ⛔ **A irmã do [`crate::collapse_in_sphere_sized`], e as duas andam juntas:** proteger a
/// agulha do colapso sem lhe dar resolução deixa-a com triângulos gigantes e mal formados —
/// medido, é isso que faz o campo cruzado a jusante perder-se. *Guardar a ponta e desenhar a
/// grade são um trabalho só.*
///
/// `sizing = None` é **byte-idêntico** a [`refine_in_sphere`].
pub fn refine_in_sphere_sized(
    mesh: &mut Mesh,
    center: [f32; 3],
    radius: f32,
    edge_max: f32,
    sizing: crate::Sizing<'_>,
    births: &mut Vec<Birth>,
    scratch: &mut RegionScratch,
) -> Refine {
    births.clear();
    if !mesh.faces().iter().all(Face::is_tri) {
        return Refine::NotTriangles;
    }
    if edge_max <= 0.0 || radius <= 0.0 || !edge_max.is_finite() {
        return Refine::Enough;
    }
    let (v0, f0) = (mesh.vert_count(), mesh.face_count());
    let mut passes = 0;
    // As faces que o corte de fato mexeu, em índices da malha que sai do último
    // passe — é a REGIÃO que o flip tem de reparar, e ela viaja de passe em
    // passe porque o emit reordena os índices (ver `one_pass`).
    let mut touched: Vec<u32> = Vec::new();
    while passes < MAX_PASSES {
        if !one_pass(
            mesh,
            center,
            radius,
            edge_max,
            sizing,
            births,
            &mut touched,
            scratch,
        ) {
            break;
        }
        passes += 1;
    }
    debug_assert_eq!(
        births.len(),
        mesh.vert_count() - v0,
        "todo vértice novo tem de declarar de onde veio"
    );
    if passes == 0 {
        return Refine::Enough;
    }
    // ⚠️ **A SEGUNDA METADE DO REFINO, e sem ela a primeira apodrece a malha.**
    // O corte não consegue manter a forma dos triângulos — a vizinha de uma face
    // escolhida tem de aprender o vértice novo, e isso a parte pela metade do
    // ângulo. O flip devolve a qualidade sem criar nem mover vértice, então ele
    // roda DEPOIS de todos os passes e não fala com o traço em voo. Os números
    // que o justificam estão no cabeçalho do [`crate::dyntopo_flip`].
    //
    // ⚠️ **Ele recebe a REGIÃO, e isso não é otimização: é o escopo honesto.** O
    // corte só pode ter estragado a forma das faces que ele partiu e das
    // vizinhas que aprenderam o vértice novo; o resto da malha está como o
    // artista a deixou, e varrê-la inteira era pagar `O(malha)` para descobrir
    // isso — medido, 33,2 de 66,1 ms num dab a 98k, metade num round que acha
    // ZERO.
    crate::dyntopo_flip::relax(mesh, &touched, scratch);
    Refine::Done {
        verts_added: mesh.vert_count() - v0,
        faces_added: mesh.face_count() - f0,
        passes,
    }
}

/// A aresta mais longa de uma face: `(comprimento², id, a, b)`.
fn longest_edge(
    ids: &EdgeIds,
    adj: &Adjacency,
    v: &[u32],
    positions: &[[f32; 3]],
) -> Option<(f32, u32, u32, u32)> {
    let mut best: Option<(f32, u32, u32, u32)> = None;
    for k in 0..v.len() {
        let (a, b) = (v[k], v[(k + 1) % v.len()]);
        let e = ids.id_of(adj, a, b)?;
        let (pa, pb) = (
            positions[v[k] as usize],
            positions[v[(k + 1) % v.len()] as usize],
        );
        let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let l2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        if best.is_none_or(|(w, ..)| l2 > w) {
            best = Some((l2, e, a, b));
        }
    }
    best
}

/// **O FECHO DE RIVARA, POR FRENTE.** Marca, a partir do que já está marcado,
/// até o ponto fixo; empurra em `front` toda face que toca uma aresta marcada.
///
/// ⚠️ **É a mesma resposta que a varredura dava, e o gate prova isso contra ela**
/// (`the_front_closes_the_same_set_the_sweep_did`, com a varredura CONGELADA sob
/// `cfg(test)` — um `pub` sem chamador seria uma segunda resposta esperando
/// alguém chamá-la). O que muda é só a ORDEM em que as marcas do fecho entram,
/// e com ela o índice dos vértices que elas geram.
///
/// ⚠️ **A fila é limitada por construção:** cada aresta é marcada uma vez e
/// empurra as ≤ 2 faces que a dividem, então `front` recebe no máximo
/// `2 × |marcadas|` entradas. Re-entrar é inofensivo — a mais longa daquela face
/// já está marcada —, e é por isso que não há carimbo de *já visitada*, que seria
/// um vetor do tamanho da malha dentro do passe que existe para não ter um.
fn close_lepp(
    ids: &EdgeIds,
    adj: &Adjacency,
    faces: &[Face],
    positions: &[[f32; 3]],
    pending: &mut [bool],
    marked: &mut Vec<(u32, u32, u32)>,
    front: &mut Vec<u32>,
) {
    let mut read = 0;
    while read < front.len() {
        let fi = front[read] as usize;
        read += 1;
        let v = faces[fi].verts();
        let Some((_, e, a, b)) = longest_edge(ids, adj, v, positions) else {
            continue;
        };
        if !pending[e as usize] {
            pending[e as usize] = true;
            marked.push((e, a, b));
            faces_of_edge(adj, faces, a, b, front);
        }
    }
}

/// **A VARREDURA que a frente substituiu**, congelada como oráculo.
///
/// Ela é o código que shipava: repetir `for TODA face da malha` até nada mudar.
/// Fica aqui — e só aqui — porque a única pergunta que ela ainda responde é
/// *"o fecho por frente alcança o mesmo conjunto?"*.
#[cfg(test)]
fn close_lepp_by_sweep(
    ids: &EdgeIds,
    adj: &Adjacency,
    faces: &[Face],
    positions: &[[f32; 3]],
    pending: &mut [bool],
) {
    let mut changed = true;
    while changed {
        changed = false;
        for face in faces {
            let v = face.verts();
            let touched = (0..v.len()).any(|k| {
                ids.id_of(adj, v[k], v[(k + 1) % v.len()])
                    .is_some_and(|e| pending[e as usize])
            });
            if !touched {
                continue;
            }
            let Some((_, e, ..)) = longest_edge(ids, adj, v, positions) else {
                continue;
            };
            if !pending[e as usize] {
                pending[e as usize] = true;
                changed = true;
            }
        }
    }
}

/// **AS FACES QUE COMPARTILHAM A ARESTA `a—b`** — o passo da frente, e a razão
/// de o corte não precisar mais varrer a malha para achar quem uma marca afeta.
///
/// ⚠️ Ela sai da adjacência que a malha JÁ carrega: as faces de `a` que também
/// contêm `b`. Custa a valência de `a`, meia dúzia — contra as 196 mil faces que
/// a varredura por iteração examinava para descobrir a mesma coisa.
fn faces_of_edge(adj: &Adjacency, faces: &[Face], a: u32, b: u32, out: &mut Vec<u32>) {
    for &fi in adj.vert_faces.neighbours(a as usize) {
        if faces[fi as usize].verts().contains(&b) {
            out.push(fi);
        }
    }
}

/// Um passe. `true` se alguma coisa foi partida.
///
/// ⚠️ **Oito argumentos, e o oitavo é a cerca por sítio** — ver [`crate::Sizing`]. Agrupá-los
/// num `struct` seria uma indireção por aresta num laço que corre sobre a malha inteira.
#[allow(clippy::too_many_arguments)]
fn one_pass(
    mesh: &mut Mesh,
    center: [f32; 3],
    radius: f32,
    edge_max: f32,
    sizing: crate::Sizing<'_>,
    births: &mut Vec<Birth>,
    touched: &mut Vec<u32>,
    region: &mut RegionScratch,
) -> bool {
    let r2 = radius * radius;
    let emax2 = edge_max * edge_max;

    // As candidatas saem do octree — é ele que torna o dab local, e é o mesmo
    // caminho que o pincel usa para achar os vértices que move.
    let mut hits = Vec::new();
    mesh.octree().faces_in_sphere(center, radius, &mut hits);
    if hits.is_empty() {
        return false;
    }

    // ⚠️ **A ESCOLHA ACONTECE ANTES DO GRAFO DE ARESTAS, e é isso que faz um dab
    // EM REGIME custar a pegada.** Depois que a região alcança a densidade
    // pedida, todo evento de ponteiro chega aqui para ouvir *não há o que
    // partir* — e a resposta não precisa de aresta NUMERADA nenhuma: um
    // comprimento sai de duas posições, e um centroide sai de três. Construir o
    // grafo inteiro para então descobrir que a lista está vazia era pagar
    // `O(malha)` pela negativa: medido, **1,84 de 2,14 ms (86%) do custo de um
    // dab em regime a 113k**.
    //
    // A lista guarda `(face, canto)` porque é isso que a geometria conhece; o
    // NÚMERO da aresta só é preciso para MARCAR, e marcar só acontece se houver
    // o que marcar.
    let positions = mesh.positions();
    let mut long: Vec<(u32, u8)> = Vec::new();

    // ⚠️ **A ESCOLHA É POR FACE, E ELA É A WAVE INTEIRA.**
    //
    // A primeira versão escolhia por ARESTA — toda aresta longa cujo meio caísse
    // na esfera. O efeito é que uma face que ATRAVESSA a fronteira do pincel tem
    // uma ou duas arestas escolhidas e a terceira não, e o padrão a corta em
    // "verde": dois triângulos, cada um com METADE do ângulo daquele canto. Como
    // a esfera do dab ANDA, o dab seguinte escolhe outra aresta da mesma face e
    // corta de novo. Medido num traço de 24 dabs: o pior ângulo mínimo cai de
    // **21,21° para 1,53°** e **15% da malha** fica abaixo de 10°.
    //
    // ⚠️ **Uma lasca não desloca vértice nenhum** — o desvio de posição era
    // literalmente o mesmo com e sem o conserto do `pre` (0,7131 contra 0,7158),
    // e foi só medindo ÂNGULO que o defeito apareceu. É a superfície de agulhas
    // do smoke de 2026-08-04: a normal por-vértice de um triângulo fino não
    // aponta para lado nenhum, e a luz desenha isso como um espinho.
    //
    // Por FACE, a face é atômica: ou as TRÊS arestas dela entram (o corte 1→4,
    // cujas quatro filhas são **semelhantes à mãe** — ângulos preservados
    // exatamente, para sempre) ou nenhuma. O corte verde deixa de ser produzido
    // pela ESCOLHA e passa a existir só no ANEL de vizinhas que a costura
    // obriga, que é uma vez por face e não uma por dab.
    //
    // ⚠️ **E o fecho tem de PARAR aí.** Promover as vizinhas a vermelho também
    // (para lhes dar a mesma garantia) foi construído e MEDIDO: ele cascateia
    // pela malha inteira — 57 → **846** vértices no hemisfério que o pincel
    // nunca tocou, ou seja o oposto exato da promessa deste modo. O anel verde é
    // o preço, e ele é local.
    //
    // ⚠️ **O CENTROIDE tem de estar na esfera** (era o MEIO de cada aresta, pelo
    // mesmo motivo, um nível abaixo): uma face enorme que só encosta na esfera
    // com uma ponta nasceria refinada onde o artista não passou.
    for &f in &hits {
        let fi = f as usize;
        let Some(face) = mesh.faces().get(fi) else {
            continue;
        };
        let v = face.verts();
        let inv = 1.0 / v.len() as f32;
        let mut c = [0.0f32; 3];
        for &i in v {
            let p = positions[i as usize];
            for (acc, q) in c.iter_mut().zip(p) {
                *acc += q * inv;
            }
        }
        let dc = [c[0] - center[0], c[1] - center[1], c[2] - center[2]];
        if dc[0] * dc[0] + dc[1] * dc[1] + dc[2] * dc[2] > r2 {
            continue;
        }
        // Escolhida a face, entram TODAS as arestas dela que passam do alvo.
        //
        // ⚠️ **Marcar só a mais longa foi construído e MEDIDO — ele converge
        // devagar demais.** Uma bissecção corta uma aresta por face por passe, e
        // o `MAX_PASSES` é um teto de QUADRO; com três passes a região do dab
        // ficava com **132 arestas acima do alvo onde antes havia 18**, ou seja
        // o refino deixava de refinar. Quem carrega o teorema de Rivara aqui é o
        // FECHO logo abaixo, não a escolha.
        for k in 0..v.len() {
            let (pa, pb) = (
                positions[v[k] as usize],
                positions[v[(k + 1) % v.len()] as usize],
            );
            let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            // ⚠️ **O limiar é o do MEIO da aresta** — a mesma lei da irmã do colapso: os dois
            // extremos podem cair em bandas diferentes, e escolher um deles faria a decisão
            // depender de qual canto a face propôs primeiro.
            let limit2 = sizing.map_or(emax2, |g| {
                let mid = [
                    0.5 * (pa[0] + pb[0]),
                    0.5 * (pa[1] + pb[1]),
                    0.5 * (pa[2] + pb[2]),
                ];
                let h = g(mid);
                h * h
            });
            if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= limit2 {
                continue;
            }
            long.push((f, u8::try_from(k).unwrap_or(u8::MAX)));
        }
    }
    // A negativa sai daqui, e ela é o desfecho NORMAL de um traço.
    if long.is_empty() {
        return false;
    }

    // ⚠️ **A aresta tem NÚMERO, e é por isso que não há mapa aqui.** O par
    // `{lo, hi}` já é o nome canônico dela, mas marcar por par exigiria um mapa —
    // e a ordem de iteração de um `HashMap` decidiria em que ordem os vértices
    // novos nascem e, com ela, os índices da malha inteira. A [`EdgeIds`] torna o
    // nome um ÍNDICE, então marcar é escrever num vetor.
    //
    // ⚠️ **E ela é a metade BARATA do grafo, de propósito.** O `Edges` inteiro
    // também constrói o mapa `(face, canto) → id` sobre a malha toda, e medido a
    // 98k isso é **1,498 dos 1,682 ms** dele — 89% — para responder por umas
    // poucas arestas da região. Aqui a numeração custa **0,184 ms** e o id sai
    // sob demanda, `O(valência)`.
    let ids = EdgeIds::build(mesh.adjacency());
    // Duas listas e não uma: `pending` diz *esta aresta parte*, `split_at` diz
    // *o vértice em que ela partiu*. Colapsá-las exigiria um valor sentinela que
    // também é um índice válido, e o dia em que o vértice 0 for um meio o refino
    // silenciosamente pula uma face.
    let mut pending = vec![false; ids.len()];
    let mut split_at = vec![u32::MAX; ids.len()];
    // A marca carrega os EXTREMOS junto, e é isso que dispensa a varredura que
    // os redescobria. ⚠️ A ORDEM é a da varredura acima — face por face, canto
    // por canto —, e ela decide em que ordem os vértices novos nascem, logo os
    // índices da malha, logo a `births`.
    let mut marked: Vec<(u32, u32, u32)> = Vec::new();
    // As faces que tocam alguma aresta marcada: a FRENTE do fecho, e depois a
    // lista exata que o emit re-triangula.
    let mut front: Vec<u32> = Vec::new();
    {
        let faces = mesh.faces();
        let adj = mesh.adjacency();
        for &(f, k) in &long {
            let v = faces[f as usize].verts();
            let (a, b) = (v[k as usize], v[(k as usize + 1) % v.len()]);
            let Some(e) = ids.id_of(adj, a, b) else {
                continue;
            };
            if !pending[e as usize] {
                pending[e as usize] = true;
                marked.push((e, a, b));
                faces_of_edge(adj, faces, a, b, &mut front);
            }
        }
    }
    if marked.is_empty() {
        return false;
    }

    // ⚠️ **A PROPAGAÇÃO (a *LEPP* de Rivara), e ela é obrigatória.** Marcar uma
    // aresta obriga a vizinha que a divide a partir também — e se aquela aresta
    // não for a MAIS LONGA da vizinha, partir por ela a afinaria. Então a
    // vizinha marca a dela, o que pode obrigar a vizinha SEGUINTE, e assim por
    // diante: é a cadeia de aresta-mais-longa.
    //
    // ⚠️ **Ela TERMINA** porque a cadeia percorre arestas estritamente mais
    // longas, e uma malha finita tem uma maior. E **é ela que faz o refino
    // alcançar um pouco além do pincel** — o preço, medido no gate, de ter
    // qualidade sem refinar o modelo inteiro.
    //
    // ⚠️ **E ela é uma FRENTE, não uma varredura.** A versão anterior repetia
    // `for TODA face da malha` até o ponto fixo — `O(malha × iterações)` para
    // descobrir um fecho que é local. Aqui uma marca empurra as duas faces que
    // dividem a aresta, e só elas são reexaminadas; o conjunto que o ponto fixo
    // alcança é **o mesmo**, porque uma face só marca se tocar uma aresta
    // pendente, e toda aresta pendente empurrou as faces dela.
    //
    // ⚠️ **A fila é limitada por construção:** cada aresta é marcada uma vez e
    // empurra as suas ≤ 2 faces, então ela recebe no máximo `2 × |marcadas|`
    // entradas. Re-entrar é inofensivo — a mais longa da face já está marcada —,
    // e por isso não há carimbo de *já visitada*, que seria um vetor do tamanho
    // da malha dentro do passe que existe para não ter um.
    close_lepp(
        &ids,
        mesh.adjacency(),
        mesh.faces(),
        mesh.positions(),
        &mut pending,
        &mut marked,
        &mut front,
    );
    front.sort_unstable();
    front.dedup();

    // Os vértices novos são APENDADOS: o primeiro leva o índice que a malha tem
    // agora, e a porta os instala nessa mesma ordem. ⚠️ Os pais de um ponto médio
    // são sempre vértices da malha de ENTRADA (uma aresta dela), então ler
    // `mesh.positions()` aqui é ler o mesmo estado que a versão anterior lia do
    // clone que ela fazia — sem o clone.
    let mut born_pos: Vec<[f32; 3]> = Vec::with_capacity(marked.len());
    let mut born_parents: Vec<(u32, u32)> = Vec::with_capacity(marked.len());
    let first_vert = u32::try_from(mesh.vert_count()).unwrap_or(u32::MAX);
    for &(e, a, b) in &marked {
        let new = first_vert + u32::try_from(born_pos.len()).unwrap_or(0);
        split_at[e as usize] = new;
        births.push(Birth { vert: new, a, b });
        born_pos.push(midpoint(mesh.positions(), mesh.normals(), a, b));
        born_parents.push((a, b));
    }

    // ⚠️ **O EMIT ITERA A FRENTE, e a razão que aqui esteve escrita primeiro
    // estava ERRADA.** A versão original varria TODA face e dizia que era a
    // varredura que fechava a rachadura; a mutação (*re-triangular só o que está
    // em `hits`*) passou nos dez gates, e a causa apareceu ao ler o
    // `faces_in_sphere`: quem fecha a rachadura é o **padrão**, que é total sobre
    // qualquer subconjunto de arestas partidas (gate `every_split_pattern…`).
    //
    // Depois disso a varredura ficou *como camada*, sob o argumento de que ela
    // tornava a garantia independente da REGRA DE MARCAÇÃO — e esse argumento
    // segue de pé, só que a `front` o honra **melhor e sem `O(malha)`**: ela é
    // construída de *toda aresta marcada*, seja qual for o critério que a marcou,
    // porque quem a preenche é o `faces_of_edge` e não a regra.
    //
    // ⚠️ **A face que não muda não é sequer reescrita, e é isso que deixa a
    // REGIÃO atravessar os passes sem remapeamento.** O emit escreve a primeira
    // filha no SLOT da mãe e apenda as outras, então um índice de face guardado
    // pelo passe anterior continua apontando para a mesma face — a `touched`
    // apenas ACUMULA. A versão que montava um vetor de faces novo renumerava
    // tudo, e por isso precisava reexpressar a marca a cada passe.
    let mut edits: Vec<(u32, Face)> = Vec::new();
    let mut added: Vec<(Face, u32)> = Vec::new();
    let mut out: Vec<Face> = Vec::new();
    let first_face = u32::try_from(mesh.face_count()).unwrap_or(u32::MAX);
    for &f in &front {
        let face = mesh.faces()[f as usize];
        let v = face.verts();
        let mid = |k: usize| -> Option<u32> {
            let (a, b) = (v[k], v[(k + 1) % v.len()]);
            let e = ids.id_of(mesh.adjacency(), a, b)?;
            let m = split_at[e as usize];
            (m != u32::MAX).then_some(m)
        };
        let m = [mid(0), mid(1), mid(2)];
        if m.iter().all(Option::is_none) {
            continue;
        }
        out.clear();
        emit_triangle(&mut out, [v[0], v[1], v[2]], m);
        touched.push(f);
        edits.push((f, out[0]));
        for &child in &out[1..] {
            touched.push(first_face + u32::try_from(added.len()).unwrap_or(0));
            added.push((child, f));
        }
    }

    mesh.splice_topology(
        &VertexAppend {
            positions: &born_pos,
            parents: &born_parents,
        },
        &edits,
        &added,
        region,
    );
    true
}

/// O vértice do meio, **deslocado ao longo da normal média**.
///
/// ⚠️ **Sem o deslocamento o refino ACHATA a superfície**: partir uma aresta
/// curva no meio geométrico põe o vértice novo *dentro* da curva, e refinar
/// muitas vezes lixa a forma que o artista acabou de esculpir. A correção é a do
/// `Subdivision.js`: o desvio é proporcional ao ÂNGULO entre as duas normais
/// (superfície plana ⇒ ângulo zero ⇒ meio exato) e ao comprimento da aresta.
fn midpoint(pos: &[[f32; 3]], nor: &[[f32; 3]], a: u32, b: u32) -> [f32; 3] {
    let (pa, pb) = (pos[a as usize], pos[b as usize]);
    let m = [
        (pa[0] + pb[0]) * 0.5,
        (pa[1] + pb[1]) * 0.5,
        (pa[2] + pb[2]) * 0.5,
    ];
    let (na, nb) = (unit(nor[a as usize]), unit(nor[b as usize]));
    let dot = (na[0] * nb[0] + na[1] * nb[1] + na[2] * nb[2]).clamp(-1.0, 1.0);
    let angle = dot.acos();
    if !angle.is_finite() || angle <= 0.0 {
        return m;
    }
    let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    // O `0.12` é da referência. Ele é o que separa "o refino segue a forma" de
    // "o refino a arredonda", e mudá-lo é decisão de LOOK, com smoke.
    let off = angle * 0.12 * len;
    let n = unit([na[0] + nb[0], na[1] + nb[1], na[2] + nb[2]]);
    [m[0] + n[0] * off, m[1] + n[1] * off, m[2] + n[2] * off]
}

fn unit(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l > 0.0 {
        [v[0] / l, v[1] / l, v[2] / l]
    } else {
        [0.0, 0.0, 1.0]
    }
}

/// Re-triangula um triângulo dados os meios das suas três arestas.
///
/// Os quatro padrões (nenhum · um · dois · três meios) são o corte clássico, e
/// os três casos de UM meio são o MESMO caso girado — escrevê-los à mão seria
/// três chances de trocar a ordem e inverter a face, que é um buraco preto na
/// luz e nada mais.
fn emit_triangle(out: &mut Vec<Face>, v: [u32; 3], mid: [Option<u32>; 3]) {
    let n = mid.iter().filter(|m| m.is_some()).count();
    match n {
        0 => out.push(Face::tri(v[0], v[1], v[2])),
        1 => {
            // Gira até o meio ser o da aresta 0.
            let k = mid.iter().position(Option::is_some).unwrap_or(0);
            let (a, b, c) = (v[k], v[(k + 1) % 3], v[(k + 2) % 3]);
            let m = mid[k].unwrap_or(a);
            out.push(Face::tri(a, m, c));
            out.push(Face::tri(m, b, c));
        }
        2 => {
            // Gira até o canto SEM meio ser o `a`: as duas arestas partidas são
            // então (a,b) e (b,c)? Não — são as que NÃO tocam `a` só num caso.
            // A forma robusta é achar a aresta não partida e girar por ela.
            let k = mid.iter().position(Option::is_none).unwrap_or(0);
            // aresta `k` = (v[k], v[k+1]) inteira ⇒ o canto compartilhado pelas
            // duas partidas é v[k+2].
            let (a, b, c) = (v[k], v[(k + 1) % 3], v[(k + 2) % 3]);
            let p = mid[(k + 1) % 3].unwrap_or(b); // meio de (b, c)
            let q = mid[(k + 2) % 3].unwrap_or(c); // meio de (c, a)
            out.push(Face::tri(p, c, q));
            out.push(Face::tri(a, b, p));
            out.push(Face::tri(a, p, q));
        }
        _ => {
            let p = mid[0].unwrap_or(v[0]);
            let q = mid[1].unwrap_or(v[1]);
            let r = mid[2].unwrap_or(v[2]);
            out.push(Face::tri(v[0], p, r));
            out.push(Face::tri(p, v[1], q));
            out.push(Face::tri(r, q, v[2]));
            out.push(Face::tri(p, q, r));
        }
    }
}

#[cfg(test)]
#[path = "dyntopo_tests.rs"]
mod tests;
