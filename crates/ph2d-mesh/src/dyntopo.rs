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

use crate::face::Face;
use crate::mesh::Mesh;

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
        if !one_pass(mesh, center, radius, edge_max, births, &mut touched) {
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
    crate::dyntopo_flip::relax(mesh, &touched);
    Refine::Done {
        verts_added: mesh.vert_count() - v0,
        faces_added: mesh.face_count() - f0,
        passes,
    }
}

/// A aresta mais longa de uma face: `(comprimento², id)`.
fn longest_edge(
    edges: &crate::edges::Edges,
    fi: usize,
    v: &[u32],
    positions: &[[f32; 3]],
) -> Option<(f32, u32)> {
    let mut best: Option<(f32, u32)> = None;
    for k in 0..v.len() {
        let e = edges.face_edge(fi, k)?;
        let (pa, pb) = (
            positions[v[k] as usize],
            positions[v[(k + 1) % v.len()] as usize],
        );
        let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let l2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        if best.is_none_or(|(b, _)| l2 > b) {
            best = Some((l2, e));
        }
    }
    best
}

/// Um passe. `true` se alguma coisa foi partida.
fn one_pass(
    mesh: &mut Mesh,
    center: [f32; 3],
    radius: f32,
    edge_max: f32,
    births: &mut Vec<Birth>,
    touched: &mut Vec<u32>,
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
            if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= emax2 {
                continue;
            }
            long.push((f, u8::try_from(k).unwrap_or(u8::MAX)));
        }
    }
    // A negativa sai daqui, e ela é o desfecho NORMAL de um traço.
    if long.is_empty() {
        return false;
    }

    // ⚠️ **A aresta tem NÚMERO, e é por isso que não há mapa aqui.** O
    // `Edges` já dá um id canônico por par de vértices, então marcar é escrever
    // num vetor indexado por ele — determinístico por construção, enquanto a
    // ordem de iteração de um `HashMap` decidiria em que ordem os vértices
    // novos nascem e, com ela, os índices da malha inteira.
    let edges = mesh.edges();
    // Duas listas e não uma: `pending` diz *esta aresta parte*, `split_at` diz
    // *o vértice em que ela partiu*. Colapsá-las exigiria um valor sentinela que
    // também é um índice válido, e o dia em que o vértice 0 for um meio o refino
    // silenciosamente pula uma face.
    let mut pending = vec![false; edges.len()];
    let mut split_at = vec![u32::MAX; edges.len()];
    let mut marked: Vec<u32> = Vec::new();
    // A ORDEM é a da varredura acima — face por face, canto por canto —, que é
    // exatamente a que numerava as marcas quando a escolha e a marcação eram o
    // mesmo laço. Ela decide em que ordem os vértices novos nascem, logo os
    // índices da malha, logo a `births`.
    for &(f, k) in &long {
        let Some(e) = edges.face_edge(f as usize, k as usize) else {
            continue;
        };
        if !pending[e as usize] {
            pending[e as usize] = true;
            marked.push(e);
        }
    }
    if marked.is_empty() {
        return false;
    }
    let positions = mesh.positions();

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
    let mut changed = true;
    while changed {
        changed = false;
        for (fi, face) in mesh.faces().iter().enumerate() {
            let v = face.verts();
            let touched = (0..v.len())
                .filter_map(|k| edges.face_edge(fi, k))
                .any(|e| pending[e as usize]);
            if !touched {
                continue;
            }
            let Some((_, e)) = longest_edge(&edges, fi, v, positions) else {
                continue;
            };
            if !pending[e as usize] {
                pending[e as usize] = true;
                marked.push(e);
                changed = true;
            }
        }
    }
    if marked.is_empty() {
        return false;
    }

    let mut positions = mesh.positions().to_vec();
    let normals = mesh.normals().to_vec();
    let mut colors = mesh.colors().map(<[[f32; 3]]>::to_vec);
    let mut masks = mesh.masks().map(<[f32]>::to_vec);
    let mut ends: Vec<(u32, u32)> = vec![(0, 0); edges.len()];

    // Para saber os EXTREMOS de cada aresta marcada sem um segundo grafo: as
    // faces já os dão.
    //
    // ⚠️ **Sobre TODAS as faces, e não só as do octree.** O fecho de aresta mais
    // longa marca fora da esfera, então uma varredura restrita às candidatas
    // deixaria `ends` em `(0, 0)` para essas — e o vértice novo nasceria no meio
    // do segmento do vértice 0 consigo mesmo, ou seja **em cima do vértice 0**.
    // Um defeito calado: a malha continua fechada e a forma colapsa num ponto.
    for (fi, face) in mesh.faces().iter().enumerate() {
        let v = face.verts();
        for k in 0..v.len() {
            if let Some(e) = edges.face_edge(fi, k).filter(|&e| pending[e as usize]) {
                ends[e as usize] = (v[k], v[(k + 1) % v.len()]);
            }
        }
    }

    for &e in &marked {
        let (a, b) = ends[e as usize];
        let new = u32::try_from(positions.len()).unwrap_or(u32::MAX);
        split_at[e as usize] = new;
        births.push(Birth { vert: new, a, b });
        let m = midpoint(&positions, &normals, a, b);
        positions.push(m);
        if let Some(c) = colors.as_mut() {
            let (ca, cb) = (c[a as usize], c[b as usize]);
            c.push([
                (ca[0] + cb[0]) * 0.5,
                (ca[1] + cb[1]) * 0.5,
                (ca[2] + cb[2]) * 0.5,
            ]);
        }
        if let Some(m) = masks.as_mut() {
            let v = (m[a as usize] + m[b as usize]) * 0.5;
            m.push(v);
        }
    }

    // ⚠️ **Toda face é examinada, não só as do dab — e a razão que eu escrevi
    // aqui primeiro estava ERRADA.** Dizia que a varredura é o que fecha a
    // rachadura; a mutação (*re-triangular só o que está em `hits`*) passou nos
    // dez gates, e ao ler o `faces_in_sphere` a causa apareceu: ele devolve
    // TODA face das folhas que a esfera toca, e uma aresta só é marcada com o
    // MEIO dentro da esfera ⇒ as duas faces que a dividem estão sempre nas
    // candidatas. Quem fecha a rachadura é o **padrão**, que é total sobre
    // qualquer subconjunto de arestas partidas (gate `every_split_pattern…`).
    //
    // A varredura FICA como camada, e agora pelo motivo verdadeiro: ela torna a
    // ausência de rachadura independente da REGRA DE MARCAÇÃO. O dia em que
    // alguém marcar por outro critério — curvatura, uma máscara, um pincel de
    // detalhe — a garantia continua de pé sem ninguém ter de reprovar este
    // raciocínio. Ela é `O(faces)` e o `rebuild` logo abaixo é `O(faces)` e ~20×
    // mais caro; quando ele sair (a estrutura mutável), esta varredura sai com
    // ele. ⚠️ Mutação registrada e NÃO sangrando, de propósito.
    // ⚠️ **A REGIÃO TEM DE ATRAVESSAR OS PASSES, e por isso ela é re-expressa
    // aqui em vez de acumulada.** O emit renumera tudo — uma face de entrada
    // vira de uma a quatro de saída —, então um índice que o passe anterior
    // guardou passa a apontar para outra face. O que sobrevive é a MARCA, lida
    // antes e reescrita depois.
    let mut was_touched = vec![false; mesh.face_count()];
    for &f in touched.iter() {
        was_touched[f as usize] = true;
    }
    touched.clear();

    let mut faces = Vec::with_capacity(mesh.face_count() + marked.len() * 2);
    for (fi, face) in mesh.faces().iter().enumerate() {
        let v = face.verts();
        let mid = |k: usize| -> Option<u32> {
            let e = edges.face_edge(fi, k)?;
            let m = split_at[e as usize];
            (m != u32::MAX).then_some(m)
        };
        let start = faces.len();
        emit_triangle(&mut faces, [v[0], v[1], v[2]], [mid(0), mid(1), mid(2)]);
        // Mais de uma saída ⇒ esta face foi partida. É `start + 1` e não `start`
        // porque toda face de entrada produz pelo menos uma de saída.
        if faces.len() > start + 1 || was_touched[fi] {
            touched.extend((start..faces.len()).map(|i| u32::try_from(i).unwrap_or(u32::MAX)));
        }
    }

    swap_topology(mesh, positions, faces, colors, masks);
    true
}

/// **Troca a topologia da malha preservando os canais por-vértice.**
///
/// Porta única do corte e do [`flip`](super::dyntopo_flip): os dois reescrevem
/// faces e nenhum dos dois pode perder cor ou máscara pelo caminho. Duas cópias
/// desta função divergiriam no dia em que entrasse o quinto canal, e o modo de
/// falha é uma máscara que some sem ninguém ver.
pub(crate) fn swap_topology(
    mesh: &mut Mesh,
    positions: Vec<[f32; 3]>,
    faces: Vec<Face>,
    colors: Option<Vec<[f32; 3]>>,
    masks: Option<Vec<f32>>,
) {
    let mut out = Mesh::from_parts(positions, faces).expect("o refino não inventa índice");
    if let Some(c) = colors {
        out.colors_mut().copy_from_slice(&c);
    }
    if let Some(m) = masks {
        out.put_masks(m);
    }
    *mesh = out;
}

/// A mesma troca quando só a LIGAÇÃO muda — os vértices ficam onde estão.
pub(crate) fn rebuild_with_faces(mesh: &mut Mesh, faces: Vec<Face>) {
    let positions = mesh.positions().to_vec();
    let colors = mesh.colors().map(<[[f32; 3]]>::to_vec);
    let masks = mesh.masks().map(<[f32]>::to_vec);
    swap_topology(mesh, positions, faces, colors, masks);
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
