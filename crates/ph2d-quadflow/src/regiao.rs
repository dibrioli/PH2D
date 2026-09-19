//! **O CAMPO NUMA MANCHA DA MALHA** — os mesmos dois campos do botão, sobre a
//! **pegada de um pincel** em vez da peça inteira.
//!
//! # Porque este módulo existe
//!
//! O resto desta crate responde a *«como fica a grade desta PEÇA?»*, sob
//! comando, em segundos. O pincel faz a outra pergunta: *«como fica a grade
//! desta MANCHA, agora, em menos de oito milissegundos, sem tocar em nada fora
//! dela?»* — e as duas têm a **mesma lei**. O que muda é o domínio.
//!
//! ⛔ **Isto NÃO é um segundo motor.** As duas funções centrais da referência
//! ([`crate::orientation::smooth_on`] e [`crate::position::smooth_on`]) já
//! recebem **fatias com a adjacência explícita** — foram escritas assim para a
//! [`crate::hierarchy`], que também não tem uma [`Mesh`] —, e é exactamente a
//! assinatura de que um pincel precisa. Este módulo só constrói o domínio:
//! quais vértices, com que vizinhança, e **quais estão pregados**.
//!
//! # A FRONTEIRA é a razão de o resultado ser honesto
//!
//! Uma mancha tem miolo e tem franja. Um vértice de franja tem vizinhos **fora**
//! da pegada, logo a linha dele no laplaciano está truncada e a lei não vale
//! para ele. A resposta não é corrigir o peso — é **pregá-lo**: ele não se move
//! e serve de **condição de fronteira** a quem se move. É a mesma frase que o
//! pincel já diz do outro lado: *fora da pegada, nem um bit*.
//!
//! ⭐⭐ E isso torna a mancha **exacta**, não aproximada: um vértice de miolo
//! tem o anel inteiro dentro da mancha, logo os pesos das arestas dele são os
//! **mesmos números** que a peça inteira daria — e há gate a afirmá-lo ao bit
//! (`o_miolo_de_uma_mancha_e_a_peca_inteira`).
//!
//! # ⚠️ O que ela custa, e contra quê
//!
//! `O(anel da pegada)`: as faces vêm do CSR de adjacência que a [`Mesh`] já
//! mantém, e o mapa de arestas cobre só elas. A [`crate::im_weights::cotangent_adjacency`]
//! é `O(faces da peça)` mais um mapa de **todas** as arestas — numa peça de
//! `100 k` vértices isso é o orçamento inteiro do dab gasto antes da primeira
//! multiplicação.

use core::cmp::Ordering;

use ph2d_mesh::Mesh;

use crate::im_weights::Link;

/// **A PEGADA, pronta para os dois campos.**
///
/// Os índices aqui são **locais** (`0..ids.len()`); [`Mancha::ids`] traduz para
/// os da malha. É a mesma troca que a [`crate::hierarchy`] faz entre níveis, e
/// pela mesma razão: as leis não querem saber de que malha os números vieram.
#[derive(Clone, Debug)]
pub struct Mancha {
    /// O id de malha de cada índice local, em ordem **crescente**.
    ///
    /// ⚠️ A ordem é crescente **de propósito**: as duas suavizações são
    /// Gauss-Seidel (elas lêem o vizinho vivo), logo o resultado depende da
    /// ordem de varredura. Ordenar é o que faz o miolo da mancha ser percorrido
    /// na mesma ordem relativa em que a peça inteira o percorreria — que é a
    /// metade que falta para a igualdade ao bit do gate.
    pub ids: Vec<u32>,
    /// A posição de cada vértice local.
    pub pos: Vec<[f32; 3]>,
    /// A normal de cada vértice local.
    pub nrm: Vec<[f32; 3]>,
    /// O anel de cada vértice local, com o peso cotangente, em índices locais.
    pub adj: Vec<Vec<Link>>,
    /// **Quem está pregado** — a franja da mancha e a borda da malha aberta.
    pub fronteira: Vec<bool>,
}

impl Mancha {
    /// Quantos vértices a mancha tem.
    #[must_use]
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// A mancha está vazia?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Quantos vértices de **miolo** — os que a lei de facto move.
    ///
    /// ⚠️ **Uma mancha pode ter centenas de vértices e miolo ZERO** (uma pegada
    /// mais fina que uma aresta, um traço na beira de uma peça aberta). Quem
    /// consome tem de saber distinguir *«não fez nada»* de *«não havia o que
    /// fazer»*, que é a diferença entre um pincel partido e um pincel calado.
    #[must_use]
    pub fn miolo(&self) -> usize {
        self.fronteira.iter().filter(|f| !**f).count()
    }
}

/// **A MANCHA de um conjunto de vértices da malha.**
///
/// `ids` pode vir em qualquer ordem e com repetições — o que sai é ordenado e
/// deduplicado. Ids fora da malha são descartados.
#[must_use]
pub fn mancha(mesh: &Mesh, ids: &[u32]) -> Mancha {
    let n = mesh.vert_count();
    let mut locais: Vec<u32> = ids.iter().copied().filter(|&v| (v as usize) < n).collect();
    locais.sort_unstable();
    locais.dedup();
    if locais.is_empty() {
        return Mancha {
            ids: locais,
            pos: Vec::new(),
            nrm: Vec::new(),
            adj: Vec::new(),
            fronteira: Vec::new(),
        };
    }

    let adjacency = mesh.adjacency();
    let p = mesh.positions();
    let nrm_all = mesh.normals();

    // ⚠️ **As faces do ANEL de cada vértice, não as faces CONTIDAS na mancha.**
    // O peso de uma aresta soma os DOIS triângulos que a partilham; colher só o
    // que está dentro entregaria meia cotangente a toda aresta do bordo do
    // miolo — e um peso pela metade não se lê como erro, lê-se como um campo
    // ligeiramente mais mole.
    let mut faces: Vec<u32> = Vec::new();
    for &v in &locais {
        faces.extend_from_slice(adjacency.vert_faces.neighbours(v as usize));
    }
    faces.sort_unstable();
    faces.dedup();

    let pesos = crate::im_weights::cotangent_edge_weights_on(mesh, &faces);

    let indice = |v: u32| -> Option<usize> { locais.binary_search(&v).ok() };

    // ⭐⭐⭐ **A FRANJA sai da adjacência que a LEI lê, nunca da lista de
    // vizinhos da malha.** Medido: numa malha de QUADS os pesos cotangente
    // incluem a **DIAGONAL** de cada quadrilátero — ela não é aresta, logo o
    // `vert_verts` não a conhece —, e um vértice com o anel inteiro dentro
    // podia ter o parceiro da diagonal **fora**. O gate do miolo apanhou-o à
    // primeira, com `4` de `55` vértices a discordar da peça inteira por **um
    // ULP**: os pesos em causa valiam `~1e-7` e o `smooth_on` só salta o ZERO
    // exacto. *Um link que quase não pesa ainda é um link que falta.*
    let mut truncado = vec![false; locais.len()];
    let mut adj: Vec<Vec<Link>> = vec![Vec::new(); locais.len()];
    for ((a, b), w) in pesos {
        match (indice(a), indice(b)) {
            (Some(ia), Some(ib)) => {
                adj[ia].push(Link {
                    id: ib as u32,
                    weight: w,
                });
                adj[ib].push(Link {
                    id: ia as u32,
                    weight: w,
                });
            }
            (Some(ia), None) => truncado[ia] = true,
            (None, Some(ib)) => truncado[ib] = true,
            (None, None) => {}
        }
    }
    for list in &mut adj {
        list.sort_by_key(|l| l.id);
    }

    let fronteira: Vec<bool> = locais
        .iter()
        .enumerate()
        // A beira de uma malha ABERTA fica pregada pela mesma razão que o
        // operador de Meyer et al. lhe devolve `None`: a fórmula pede os dois
        // ângulos opostos a cada aresta, e ali só existe um.
        .map(|(i, &v)| truncado[i] || adjacency.is_border(v as usize))
        .collect();

    Mancha {
        ids: locais.clone(),
        pos: locais.iter().map(|&v| p[v as usize]).collect(),
        nrm: locais.iter().map(|&v| nrm_all[v as usize]).collect(),
        adj,
        fronteira,
    }
}

/// **O CAMPO DE ORIENTAÇÃO da mancha, semeado pelo TRAÇO.**
///
/// Cada vértice nasce com a direcção do traço projectada no plano tangente
/// dele, e a suavização 4-RoSy torna o campo **consistente** sobre a curvatura
/// — que é o trabalho que a projecção sozinha não faz: dois vértices vizinhos
/// com normais diferentes recebem tangentes que, comparadas, podem estar a 90°
/// uma da outra sem que nada esteja errado, e é a compatibilização extrínseca
/// que escolhe o representante certo de cada uma.
///
/// ⚠️ **A franja é semeada como o miolo e depois PREGADA.** Ela é a direcção
/// que o artista pediu, não a que a superfície tinha — o dab IMPÕE, e a
/// atenuação do pincel é que decide quanto do que o campo diz chega ao barro.
///
/// ⛔ **Direcção nula devolve o campo VAZIO**, nunca um eixo inventado: é a
/// mesma degenerescência que o `ph2d_rake::pentear` declara — com menos de dois
/// carimbos não há direcção, e não há lei que dizer.
#[must_use]
pub fn orientacao_semeada(m: &Mancha, direccao: [f32; 3], iteracoes: usize) -> Vec<[f32; 3]> {
    if m.is_empty() || norm(direccao) <= 0.0 {
        return Vec::new();
    }
    let mut dirs: Vec<[f32; 3]> = m
        .nrm
        .iter()
        .map(|&n| crate::orientation::project_tangent(direccao, n))
        .collect();
    debug_assert_eq!(dirs.len(), m.len());
    crate::orientation::smooth_on_fixed(&mut dirs, &m.nrm, &m.adj, &m.fronteira, iteracoes);
    dirs
}

/// ⭐⭐⭐ **O CAMPO DE POSIÇÃO da mancha — a RETÍCULA de lado `passo`.**
///
/// Devolve, para cada vértice, **o ponto da grelha quadrada onde ele devia
/// estar**: uma retícula de lado `passo` alinhada com [`orientacao_semeada`],
/// consensual entre vizinhos, e reduzida ao ponto mais perto do vértice.
///
/// # Porque esta é a classe certa e a troca de diagonais não era
///
/// Trocar diagonais escolhe **que vértices se ligam**; ela não tem como dizer
/// *«e a que distância»*. Uma retícula diz as duas coisas de uma vez — o
/// alinhamento e o espaçamento **igual nas duas direcções** são a mesma
/// construção —, e é por isso que ela não compra uma à custa da outra.
///
/// # ⚠️ O deslocamento é TANGENCIAL e LIMITADO, e as duas coisas são por
/// construção
///
/// A `position_round_4` monta o ponto a partir de `q` e `n×q`, que são tangentes
/// ao vértice, e devolve o **mais perto** da posição dele ⇒ o alvo vive no plano
/// tangente e nunca está a mais de meia diagonal (`0,71 · passo`). *A malha
/// desliza; ela não incha nem encolhe.*
///
/// ⛔ **`dirs` tem de ser o campo desta mancha** (mesmo comprimento). Com
/// `passo` não positivo, ou sem campo, devolve vazio — não há retícula que
/// dizer.
#[must_use]
pub fn posicao_da_mancha(
    m: &Mancha,
    dirs: &[[f32; 3]],
    passo: f32,
    iteracoes: usize,
) -> Vec<[f32; 3]> {
    posicao_da_mancha_com(m, dirs, passo, iteracoes, false)
}

/// A mesma, com a **SEMENTE** por parâmetro — a variável que a sonda do arame
/// varre. `false` é o que a referência faz (cada vértice é a própria origem).
#[must_use]
pub fn posicao_da_mancha_com(
    m: &Mancha,
    dirs: &[[f32; 3]],
    passo: f32,
    iteracoes: usize,
    semente_unica: bool,
) -> Vec<[f32; 3]> {
    if m.is_empty() || dirs.len() != m.len() || passo.partial_cmp(&0.0) != Some(Ordering::Greater) {
        return Vec::new();
    }
    let escalas = vec![passo; m.len()];
    // ⚠️ **A SEMENTE decide a COERÊNCIA**, e é ela que a sonda varre: com cada
    // vértice a nascer como a própria origem, a suavização só faz consenso
    // LOCAL; com uma origem só, a mancha inteira partilha uma retícula.
    let mut pos = if semente_unica {
        let n = m.len() as f32;
        let c = m.pos.iter().fold([0.0f32; 3], |a, p| {
            [a[0] + p[0] / n, a[1] + p[1] / n, a[2] + p[2] / n]
        });
        vec![c; m.len()]
    } else {
        m.pos.clone()
    };
    crate::position::smooth_on_fixed(
        &mut pos,
        &m.pos,
        &m.nrm,
        dirs,
        &escalas,
        &m.adj,
        &m.fronteira,
        iteracoes,
    );
    pos
}

/// **O LADO DA RETÍCULA que a própria pegada pede** — a aresta média dela.
///
/// ⭐ Ele sai da MALHA e não de um slider, pela mesma razão que o alvo do dab
/// saiu do raio do pincel e passou a sair da ÁREA da peça: *o pincel diz ONDE, a
/// malha diz a que escala*. E é isto que faz a lei curar o esticão em vez de o
/// prescrever — num remendo esticado a média fica entre o comprido e o curto, e
/// a retícula puxa os dois para ela.
///
/// ⚠️ Conta as arestas **da malha** (nunca a diagonal de um quad, que os pesos
/// cotangente incluem e que não é uma aresta), e conta as de dentro duas vezes —
/// é um estimador, e o que importa é ser **determinístico**.
#[must_use]
pub fn passo_da_pegada(mesh: &Mesh, ids: &[u32]) -> f32 {
    let p = mesh.positions();
    let viz = &mesh.adjacency().vert_verts;
    let (mut soma, mut n) = (0.0f64, 0usize);
    for &v in ids {
        let pv = p[v as usize];
        for &j in viz.neighbours(v as usize) {
            let pj = p[j as usize];
            let d = [pv[0] - pj[0], pv[1] - pj[1], pv[2] - pj[2]];
            soma += f64::from(norm(d));
            n += 1;
        }
    }
    if n == 0 {
        0.0
    } else {
        (soma / n as f64) as f32
    }
}

/// ⭐⭐⭐ **ARRUMA A PEGADA NA GRELHA** — a lei nova do pente, sobre a malha.
///
/// Para cada vértice do **miolo** da pegada: constrói-se o campo de orientação
/// semeado pelo traço, dele o campo de posição, e o vértice caminha `peso` do
/// caminho até ao ponto de retícula dele. Devolve quantos se moveram.
///
/// # ⛔ Ela NÃO é a lei da bancada de paridade, e isso é DECLARADO
///
/// O corpus do pente mede a lei de DESLOCAMENTO (a média do anel, guardada pela
/// direcção do traço) contra as `221` corridas do alvo, e essa lei fica onde
/// está, intocada. Esta corre no **passe de topologia**, ao lado da troca de
/// diagonais — que a bancada também não alcança, porque o `correr_com` dela
/// nunca chama os motores de topologia. *Pôr a retícula no caminho do carimbo
/// re-basearia o corpus em silêncio.*
///
/// # ⚠️ O que ela promete, e o que não
///
/// - **Fora da pegada, nem um bit** — só o miolo anda, e a franja é a condição
///   de fronteira dos dois campos ([`mancha`]).
/// - **Tangencial e limitada** — ver [`posicao_da_mancha`].
/// - ⛔ **Ela não refresca a malha.** Quem chama despeja a lista devolvida no
///   `refresh_region` dele, como o resto do passe de topologia faz — o pente e o
///   refino partilham a janela da chamada.
///
/// `peso` recebe a posição do vértice e devolve quanto do caminho ele anda
/// (`0` = fica, `1` = pousa no ponto da grelha). Fora de `[0,1]` é cortado.
pub fn arruma_na_grelha(
    mesh: &mut Mesh,
    centro: [f32; 3],
    raio: f32,
    direccao: [f32; 3],
    peso: &(dyn Fn([f32; 3]) -> f32 + Sync),
    iteracoes: usize,
    movidos: &mut Vec<u32>,
) -> usize {
    arruma_na_grelha_com(
        mesh, centro, raio, direccao, peso, iteracoes, 1.0, movidos,
    )
}

/// A mesma, com o **factor do lado da célula** por parâmetro — a variável que a
/// sonda da escada varre. `1,0` é a aresta média da pegada, que é o que a
/// [`arruma_na_grelha`] passa.
#[allow(clippy::too_many_arguments)]
pub fn arruma_na_grelha_com(
    mesh: &mut Mesh,
    centro: [f32; 3],
    raio: f32,
    direccao: [f32; 3],
    peso: &(dyn Fn([f32; 3]) -> f32 + Sync),
    iteracoes: usize,
    k_passo: f32,
    movidos: &mut Vec<u32>,
) -> usize {
    arruma_na_grelha_semeada(
        mesh, centro, raio, direccao, peso, iteracoes, k_passo, false, movidos,
    )
}

/// A mesma, com a SEMENTE do campo de posição por parâmetro — ver
/// [`posicao_da_mancha_com`].
#[allow(clippy::too_many_arguments)]
pub fn arruma_na_grelha_semeada(
    mesh: &mut Mesh,
    centro: [f32; 3],
    raio: f32,
    direccao: [f32; 3],
    peso: &(dyn Fn([f32; 3]) -> f32 + Sync),
    iteracoes: usize,
    k_passo: f32,
    semente_unica: bool,
    movidos: &mut Vec<u32>,
) -> usize {
    movidos.clear();
    if raio <= 0.0 || norm(direccao) <= 0.0 {
        return 0;
    }
    // ⚠️ **Pela ÁRVORE e não por uma consulta com carimbo de época** — aquela
    // aloca um vector do tamanho da PEÇA por chamada, e o que um dab tem para
    // gastar é a pegada dele. É a mesma porta que o [`crate::regiao`] descreve
    // no cabeçalho e que o alinhamento de arestas já usa.
    let mut faces = Vec::new();
    mesh.octree().faces_in_sphere(centro, raio, &mut faces);
    if faces.is_empty() {
        return 0;
    }
    let r2 = raio * raio;
    let mut ids: Vec<u32> = Vec::new();
    {
        let p = mesh.positions();
        let todas = mesh.faces();
        for &f in &faces {
            for &v in todas[f as usize].verts() {
                let q = p[v as usize];
                let d = [q[0] - centro[0], q[1] - centro[1], q[2] - centro[2]];
                if d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])) <= r2 {
                    ids.push(v);
                }
            }
        }
    }
    if ids.is_empty() {
        return 0;
    }

    let passo = passo_da_pegada(mesh, &ids) * k_passo;
    let m = mancha(mesh, &ids);
    if m.miolo() == 0 || passo.partial_cmp(&0.0) != Some(Ordering::Greater) {
        return 0;
    }
    let dirs = orientacao_semeada(&m, direccao, iteracoes);
    let grelha = posicao_da_mancha_com(&m, &dirs, passo, iteracoes, semente_unica);
    if grelha.len() != m.len() {
        return 0;
    }

    // ⭐⭐⭐ **JACOBI, e a cerca da FORMA no meio.** Primeiro decide-se tudo
    // contra as posições de ENTRADA; só depois se escreve. Ler o vizinho vivo
    // faria a saída depender da ordem da pegada — o defeito que o
    // `ph2d_rake::pentear` já corrigiu uma vez e que o
    // [`crate::position::smooth_on`] evita com o buffer próprio.
    let mut aprovados: Vec<(u32, [f32; 3])> = Vec::new();
    for (i, &alvo) in grelha.iter().enumerate() {
        if m.fronteira[i] {
            continue;
        }
        let v = m.ids[i];
        let w = peso(m.pos[i]).clamp(0.0, 1.0);
        if w <= 0.0 {
            continue;
        }
        let antes = m.pos[i];
        let depois = [
            w.mul_add(alvo[0] - antes[0], antes[0]),
            w.mul_add(alvo[1] - antes[1], antes[1]),
            w.mul_add(alvo[2] - antes[2], antes[2]),
        ];
        if depois == antes {
            continue;
        }
        if lasca(mesh, v, depois) {
            continue;
        }
        aprovados.push((v, depois));
    }

    let posicoes = mesh.positions_mut();
    for (v, p) in aprovados {
        posicoes[v as usize] = p;
        movidos.push(v);
    }
    movidos.len()
}

/// **Este vértice, posto em `destino`, afina um triângulo do anel dele?**
///
/// ⭐⭐⭐ **A cerca da FORMA, e ela é a mesma que a troca de diagonal já tinha
/// por escrito:** *este passe não pode piorar um triângulo*. Ela nasceu de um
/// gate VERMELHO — a primeira corrida da retícula pelo caminho do produto
/// deixou `3` lascas abaixo de `5°` em `2 379` triângulos, onde a lei que ela
/// substituiu deixava **zero**, e *uma lasca não tem normal utilizável*.
///
/// ⚠️ **É `pior && abaixo do chão`, nunca só uma das duas.** Só *«pior»*
/// congelaria a malha (a retícula reforma triângulos de propósito); só *«abaixo
/// do chão»* prenderia para sempre um vértice cujo anel já nasceu com uma lasca
/// — e essa é a metade que o produto encontra numa peça esculpida.
///
/// ⚠️ **Sem transcendental:** o menor ângulo é o de maior COSSENO, e *abaixo de
/// `θ`* é *cosseno acima de `cos θ`*. Materializar o ângulo seria pagar um
/// `acos` por canto para responder o que a comparação já responde.
fn lasca(mesh: &Mesh, v: u32, destino: [f32; 3]) -> bool {
    let p = mesh.positions();
    let faces = mesh.faces();
    let (mut antes, mut depois) = (-1.0f32, -1.0f32);
    for &f in mesh.adjacency().vert_faces.neighbours(v as usize) {
        let face = faces[f as usize];
        for t in 0..face.tri_count() {
            let tri = face.tri_at(t);
            if !tri.contains(&v) {
                continue;
            }
            let leia = |i: u32| if i == v { destino } else { p[i as usize] };
            antes = antes.max(maior_cosseno([
                p[tri[0] as usize],
                p[tri[1] as usize],
                p[tri[2] as usize],
            ]));
            depois = depois.max(maior_cosseno([leia(tri[0]), leia(tri[1]), leia(tri[2])]));
        }
    }
    depois > antes && depois > CHAO_DA_LASCA
}

/// `cos(5°)` — o chão da lasca.
///
/// ⚠️ **O número é o do gate da cena** (`LIMIAR_DA_LASCA`, `5°`), e não um valor
/// escolhido aqui: é ali que o dono julga o resultado, e duas respostas à
/// pergunta *«isto é uma lasca?»* divergiriam no dia em que uma delas mudasse.
const CHAO_DA_LASCA: f32 = 0.996_194_7;

/// O **maior cosseno** dos três cantos — ou seja, o cosseno do MENOR ângulo.
fn maior_cosseno(t: [[f32; 3]; 3]) -> f32 {
    let mut pior = -1.0f32;
    for k in 0..3 {
        let (a, b, c) = (t[k], t[(k + 1) % 3], t[(k + 2) % 3]);
        let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let w = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let (lu, lw) = (norm(u), norm(w));
        if lu <= 0.0 || lw <= 0.0 {
            // Uma aresta de comprimento zero não tem canto: ela é a própria
            // degenerescência, e devolver `1` (ângulo nulo) é dizê-lo.
            return 1.0;
        }
        let c = u[0].mul_add(w[0], u[1].mul_add(w[1], u[2] * w[2])) / (lu * lw);
        pior = pior.max(c.clamp(-1.0, 1.0));
    }
    pior
}

fn norm(a: [f32; 3]) -> f32 {
    a[0].mul_add(a[0], a[1].mul_add(a[1], a[2] * a[2])).sqrt()
}

#[cfg(test)]
#[path = "regiao_tests.rs"]
mod tests;
