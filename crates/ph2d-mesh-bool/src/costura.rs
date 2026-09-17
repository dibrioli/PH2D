//! **A LIMPEZA DA COSTURA** — o que o motor de booleana deixa na curva de
//! interseção, e que nenhum artista pediu.
//!
//! # ⛔⛔⛔ O defeito, MEDIDO (report do dono, 2026-09-15)
//!
//! *«O algoritmo remesh produz bordas mais corretas que o algoritmo da Box
//! Trim. Tente melhorar a topologia das bordas do corte.»*
//!
//! Cortando uma esfera de `49 612` triângulos (aresta alvo `0,0242`) com um
//! cilindro, a saída crua do motor traz:
//!
//! | | aspecto p90 | p99 | **MAX** | aresta mínima |
//! |---|---|---|---|---|
//! | cru | `3,45` | `25,7` | **`2 573 809`** | **`8,74e-9`** |
//!
//! ⇒ uma aresta **`2,8` milhões de vezes** menor que a malha: são **vértices
//! duplicados** que o motor emite onde a curva de interseção passa quase por um
//! vértice da peça. *Um triângulo com aspecto de dois milhões não tem normal
//! utilizável, e é isso que a borda mostra.*
//!
//! # ⭐ A cura, e as DUAS cercas que a tornam segura
//!
//! Soldar os coincidentes e colapsar as arestas curtas — **restrito aos
//! vértices que o corte CRIOU**. As duas cercas não são zelo:
//!
//! 1. ⛔ **Uma aresta entre DOIS vértices antigos é da PEÇA, e não nossa para
//!    tocar.** Sem esta cerca a limpeza varre a malha inteira: medido, ela
//!    colapsava arestas curtas naturais da esfera e **`1 251` de `14 136`**
//!    vértices longe do corte deixavam de ser bit-idênticos — isto é, ela
//!    quebrava a propriedade que decide a arquitectura desta linha.
//! 2. ⭐ **Quando um extremo é antigo, o sobrevivente é ELE.** O vértice que já
//!    existia não se move um bit; quem anda é o que o corte acabou de criar.
//!
//! Com as duas: **`14 136` de `14 136`** sobrevivem ao bit, o `MAX` cai para
//! `33`, o bordo continua em `0`, o não-manifold em `0` e o volume muda
//! `−2,2e-6` relativo.
//!
//! # ⭐⭐⭐ O quarto passo: ENDIREITAR as lascas (17/09)
//!
//! Ver [`endireita_as_lascas`]. ⛔ **O flip GLOBAL continua fora** — o
//! [`ph2d_mesh::relax_valence`] por cima disto paga por mudar a ligação da
//! malha **longe do corte**, e *uma cura que muda a peça inteira para ganhar
//! oito triângulos não é uma cura.* O que entrou é o oposto: uma troca de
//! diagonal **restrita à costura**, com cinco cercas.

use ph2d_mesh::{Face, Mesh};

/// **A fracção da aresta da peça abaixo da qual uma aresta da costura é lixo.**
///
/// ⭐⭐ **MEDIDA, e é o joelho de uma curva** (mesma peça do cabeçalho; a coluna
/// que decide é o `MAX`, porque é o triângulo impossível que estraga a borda):
///
/// | fracção | piores > 20 | p99 | **MAX** | volume |
/// |---|---|---|---|---|
/// | `0,05` | `676` | `14,4` | `249` | `−1,8e-7` |
/// | `0,10` | `658` | `13,2` | `249` | `−4,8e-7` |
/// | **`0,20`** | `640` | `13,2` | **`33`** | `−2,2e-6` |
/// | `0,35` | `637` | `13,2` | `30` | `−1,7e-5` |
///
/// ⇒ o `MAX` cai `7,5 ×` entre `0,10` e `0,20` e mais nada entre `0,20` e
/// `0,35`, enquanto o volume paga `8 ×`. *O joelho é `0,20`.*
///
/// ⛔⛔⛔ **E a frase que estava aqui era FALSA — os `~632` NÃO são da costura.**
///
/// Ela dizia *«são cunhas finas onde a curva de interseção passa rente a um
/// vértice da peça, e curá-las mexeria na malha da peça»*. Medido (17/09): a
/// **esfera de ENTRADA** tem `632` triângulos piores que `20` e **todos** têm
/// `|y| > 0,99` — são o **leque do PÓLO** de uma esfera UV, exactamente o que o
/// doc da [`ph2d_mesh::shapes::sculpt_sphere`] descreve por escrito ao explicar
/// porque é que o módulo de escultura **não** abre com uma. *A régua somava a
/// peça inteira; o corte não lhes toca.*
///
/// ⚠️ **E a sonda que já existia imprimia a resposta**: o `diag_o_pico_na_borda`
/// escreve `vértices ANTIGOS: 3/3` ao lado de cada uma. *Quando uma página
/// imprime o que desmente a hipótese, isso É o achado — e eu li aquilo como
/// confirmação.*
///
/// A costura de verdade, sozinha, mede: **`8`** piores que `20` em `28 096`
/// (corte centrado) e **`16`** em `13 476` (corte a sair pela silhueta).
pub const FRACCAO_DA_ARESTA: f32 = 0.20;

/// A tolerância da SOLDA, em fracção da aresta da peça.
///
/// ⚠️ **Três ordens de grandeza abaixo do colapso, de propósito:** aqui não se
/// decide nada — dois vértices a `8,74e-9` um do outro **são o mesmo ponto**, e
/// juntá-los não move geometria nenhuma. A decisão mora no
/// [`FRACCAO_DA_ARESTA`].
const FRACCAO_DA_SOLDA: f32 = 1e-3;

/// **Limpa a costura que o motor deixou** — ver o cabeçalho.
///
/// `peca` é a entrada do corte, e ela é obrigatória: é dela que sai **qual
/// vértice é antigo** (a cerca 1) e **qual é a aresta da malha** (o limiar).
#[must_use]
pub fn limpa_a_costura(saida: &Mesh, peca: &Mesh) -> Mesh {
    limpa_relatando(saida, peca).0
}

/// **O mesmo, dizendo quantas ALMOFADAS descartou** — ver
/// [`descarta_almofadas`].
///
/// ⛔ **Só existe para os gates**, e atravessa a fronteira da crate pela mesma
/// feature de um item que o [`super::corta_cru`] usa: *o controlo de que a
/// limpeza faz alguma coisa tem de correr onde a lâmina real é construída.*
#[cfg(any(test, feature = "test-support"))]
#[must_use]
pub fn limpa_a_costura_relatando(saida: &Mesh, peca: &Mesh) -> (Mesh, usize) {
    limpa_relatando(saida, peca)
}

fn limpa_relatando(saida: &Mesh, peca: &Mesh) -> (Mesh, usize) {
    let alvo = aresta_da_peca(peca);
    if !(alvo.is_finite() && alvo > 0.0) {
        return (saida.clone(), 0);
    }
    let (soldar, curta) = (alvo * FRACCAO_DA_SOLDA, alvo * FRACCAO_DA_ARESTA);
    let antigos: std::collections::BTreeSet<[u32; 3]> = peca.positions().iter().map(bits).collect();

    // (1) SOLDA — dois vértices no mesmo ponto viram um.
    let p = saida.positions();
    let mut celas = std::collections::BTreeMap::new();
    let mut remap = vec![0u32; p.len()];
    let mut pos: Vec<[f32; 3]> = Vec::with_capacity(p.len());
    for (i, v) in p.iter().enumerate() {
        let chave = [
            (v[0] / soldar).round() as i64,
            (v[1] / soldar).round() as i64,
            (v[2] / soldar).round() as i64,
        ];
        let e = *celas.entry(chave).or_insert_with(|| {
            pos.push(*v);
            u32::try_from(pos.len() - 1).unwrap_or(u32::MAX)
        });
        remap[i] = e;
    }

    let mut tris = Vec::new();
    for f in saida.faces() {
        f.triangles(&mut tris);
    }
    let mut t: Vec<[u32; 3]> = tris
        .iter()
        .map(|x| {
            [
                remap[x[0] as usize],
                remap[x[1] as usize],
                remap[x[2] as usize],
            ]
        })
        .filter(|x| x[0] != x[1] && x[1] != x[2] && x[2] != x[0])
        .collect();

    // (2) COLAPSO — com as duas cercas do cabeçalho.
    let mut pai: Vec<u32> = (0..u32::try_from(pos.len()).unwrap_or(u32::MAX)).collect();
    let novo = |v: [f32; 3]| !antigos.contains(&bits(&v));
    for x in &t {
        for (a, b) in [(x[0], x[1]), (x[1], x[2]), (x[2], x[0])] {
            let (ra, rb) = (raiz(&mut pai, a), raiz(&mut pai, b));
            if ra == rb || dist(pos[ra as usize], pos[rb as usize]) >= curta {
                continue;
            }
            let (lo, hi) = match (novo(pos[ra as usize]), novo(pos[rb as usize])) {
                // ⛔ Os dois antigos: é aresta da PEÇA.
                (false, false) => continue,
                // ⭐ O antigo SOBREVIVE — ele não se move um bit.
                (false, true) => (ra, rb),
                (true, false) => (rb, ra),
                (true, true) if ra < rb => (ra, rb),
                (true, true) => (rb, ra),
            };
            pai[hi as usize] = lo;
        }
    }
    for x in &mut t {
        for i in x.iter_mut() {
            *i = raiz(&mut pai, *i);
        }
    }
    t.retain(|x| x[0] != x[1] && x[1] != x[2] && x[2] != x[0]);

    // (3) ALMOFADAS — ver [`descarta_almofadas`].
    let almofadas = descarta_almofadas(&mut t);

    // (4) LASCAS — ver [`endireita_as_lascas`].
    endireita_as_lascas(&pos, &mut t, &novo_por_indice(&pos, &antigos), alvo);

    let faces: Vec<Face> = t.iter().map(|x| Face::tri(x[0], x[1], x[2])).collect();
    let (pos, faces, _) = ph2d_mesh::compact_for_faces(&pos, &faces);
    (
        Mesh::from_parts(pos, faces).unwrap_or_else(|_| saida.clone()),
        almofadas,
    )
}

/// A aresta que a peça TEM — a mesma régua que a lâmina usa para se tesselar
/// ([`ph2d_mesh::edge_for_tri_count`], ancorada na ÁREA).
fn aresta_da_peca(peca: &Mesh) -> f32 {
    let tris: usize = peca
        .faces()
        .iter()
        .map(|f| f.verts().len().saturating_sub(2))
        .sum();
    #[expect(
        clippy::cast_precision_loss,
        reason = "a contagem entra numa raiz; um ULP de contagem não move o limiar"
    )]
    ph2d_mesh::edge_for_tri_count(peca.surface_area(), tris as f32)
}

fn bits(v: &[f32; 3]) -> [u32; 3] {
    [v[0].to_bits(), v[1].to_bits(), v[2].to_bits()]
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn raiz(pai: &mut [u32], mut i: u32) -> u32 {
    while pai[i as usize] != i {
        pai[i as usize] = pai[pai[i as usize] as usize];
        i = pai[i as usize];
    }
    i
}

/// ⭐⭐⭐ **DESCARTA AS ALMOFADAS** — o mesmo triângulo emitido DUAS vezes, com o
/// enrolamento invertido.
///
/// # ⛔⛔⛔ O defeito é MEU, e a medição corrigiu a minha 1.ª explicação
///
/// Report do dono (2026-09-15, com foto): *«Borda melhorou mas não está
/// perfeita»* — um **espigão** a sair da silhueta da peça. Medido com o corte a
/// **SAIR pela beira**: `2` pares **espelhados** — dois triângulos sobre os
/// mesmos três vértices, um virado ao contrário. Juntos encerram volume
/// **ZERO**: são uma aba infinitamente fina, e é isso que o sombreamento desenha
/// como uma farpa.
///
/// ⛔⛔ **A minha 1.ª redacção dizia que o MOTOR os emitia, e é FALSO — quem os
/// cria é o COLAPSO desta mesma limpeza.** O gate escrito para o provar reprovou
/// no controlo: a saída **crua** traz `0` almofadas em todas as posições
/// varridas. *Fundir dois vértices faz dois triângulos distintos passarem a ter
/// o mesmo trio*, e foi a cura da wave anterior que abriu este defeito. ⇒ *uma
/// cura que cria uma segunda espécie de lixo tem de a varrer também.*
///
/// ⚠️⚠️ **E a fixtura CENTRADA não contém o fenómeno:** com o círculo no meio da
/// peça a borda do corte vive a `|z| = 0,8` e **nunca encontra a silhueta** (que
/// é o equador) — `0` almofadas. ⛔ A lâmina GROSSA (o cubo de seis faces) também
/// não as produz em posição nenhuma. *O defeito do dono estava exactamente onde
/// a régua não olhava*, que é a sétima vez que este módulo escreve esta frase.
///
/// ⭐ **Descartam-se os DOIS lados**, e não um: eles não são «um triângulo a
/// mais», são um par que não descreve superfície nenhuma. Guardar um deixaria
/// uma aba de face única pendurada na malha. É a mesma decisão que a linha do
/// quad remesh tomou quando achou a almofada dela (`mirrored_cells`).
///
/// ⚠️ **Corre DEPOIS do colapso**, porque o colapso pode criar uma: fundir dois
/// vértices faz dois triângulos distintos passarem a ter o mesmo trio.
fn descarta_almofadas(t: &mut Vec<[u32; 3]>) -> usize {
    let chave = |x: &[u32; 3]| {
        let mut k = *x;
        k.sort_unstable();
        k
    };
    let mut por_chave: std::collections::BTreeMap<[u32; 3], Vec<usize>> =
        std::collections::BTreeMap::new();
    for (i, x) in t.iter().enumerate() {
        por_chave.entry(chave(x)).or_default().push(i);
    }
    let mesmo_ciclo =
        |a: &[u32; 3], b: &[u32; 3]| (0..3).any(|r| (0..3).all(|i| a[i] == b[(i + r) % 3]));
    let mut fora = vec![false; t.len()];
    let mut pares = 0usize;
    for lista in por_chave.values().filter(|v| v.len() > 1) {
        // Empareha o primeiro que ainda está de pé com o primeiro ESPELHO dele.
        for (pos, &i) in lista.iter().enumerate() {
            if fora[i] {
                continue;
            }
            if let Some(&j) = lista[pos + 1..]
                .iter()
                .find(|&&j| !fora[j] && !mesmo_ciclo(&t[i], &t[j]))
            {
                fora[i] = true;
                fora[j] = true;
                pares += 1;
            }
        }
    }
    let mut k = 0usize;
    t.retain(|_| {
        let manter = !fora[k];
        k += 1;
        manter
    });
    pares
}

/// Um fecho que responde *«este índice é um vértice que o corte CRIOU?»*.
fn novo_por_indice<'a>(
    pos: &'a [[f32; 3]],
    antigos: &'a std::collections::BTreeSet<[u32; 3]>,
) -> impl Fn(u32) -> bool + 'a {
    move |i: u32| !antigos.contains(&bits(&pos[i as usize]))
}

/// **O aspecto de um triângulo** — a mesma régua que os gates do corte lêem.
/// Um equilátero mede `1,73`; área zero devolve infinito.
fn aspecto(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let (l0, l1, l2) = (dist(a, b), dist(b, c), dist(c, a));
    let s = (l0 + l1 + l2) * 0.5;
    let area = (s * (s - l0) * (s - l1) * (s - l2)).max(0.0).sqrt();
    if area > 1e-14 {
        l0.max(l1).max(l2) * s / (2.0 * area)
    } else {
        f32::INFINITY
    }
}

fn normal_crua(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ]
}

fn ponto(n: [f32; 3], m: [f32; 3]) -> f32 {
    n[0] * m[0] + n[1] * m[1] + n[2] * m[2]
}

fn comprimento(n: [f32; 3]) -> f32 {
    ponto(n, n).sqrt()
}

/// **A partir de que aspecto um triângulo da costura é uma LASCA.**
const ASPECTO_DA_LASCA: f32 = 20.0;

/// **Quão PLANAS as duas faces têm de ser, em graus, para a troca ser legítima.**
///
/// ⚠️ **CARRIL NOMEADO, e a mutação que o apaga SOBREVIVE.** Ele dispara `47`
/// vezes nas seis posições do corte e o que muda na saída é o volume em
/// `6e-9` relativo — *abaixo de toda barra que este repo tem*. Ele fica porque
/// o modo de falha que impede é a **quina do corte** ser aparada por uma troca
/// de diagonal, e ficaria alcançável no dia em que alguém alargasse a cerca do
/// tamanho ou o limiar da lasca. ⛔ Quem o apagar tem de trazer a fixtura que o
/// torna observável — não a havia em 17/09.
const PLANAS_ATE_GRAUS: f32 = 5.0;

/// Quantas passagens no máximo — uma troca pode expor a seguinte.
const PASSAGENS: usize = 4;

/// **Até que tamanho, em arestas da peça, uma face é COSTURA.**
///
/// ⭐ **Medido, e o vale é de uma ordem de grandeza** (seis posições do corte,
/// esfera de `49 612` triângulos): a aresta longa de uma lasca da costura mede
/// `0,8`–`1,7` arestas da peça; a da parede de uma lâmina **mínima** mede `~40`.
/// *O número não está no meio de nada: está num vazio de `23×`.*
const ESCALA_DA_COSTURA: f32 = 3.0;

/// ⭐⭐⭐ **ENDIREITA AS LASCAS** — troca a diagonal do par de faces onde a
/// costura deixou um triângulo fino.
///
/// # ⛔ O defeito que ela cura NÃO é o aspecto
///
/// As lascas que o colapso deixa são finas e **PLANAS**, e uma face plana tem
/// normal perfeita: medida contra as vizinhas, a lasca discorda `≤ 6,93°` onde
/// a costura sadia discorda até `46°` (a quina do corte). ⇒ *o aspecto não é o
/// que se vê.*
///
/// O que se vê é a **normal do VÉRTICE**, e ela estraga porque a
/// [`ph2d_mesh::normals`] soma normais de face **unitárias** — um *gather* sem
/// peso de área ⇒ **uma lasca vota com peso cheio**. Medido sobre os vértices
/// cujas faces estão TODAS na casca (a esfera de `49 612` T, seis posições do
/// corte, desvio da radial em graus):
///
/// | centro | peça | saída CRUA | limpa SEM esta troca | **limpa** |
/// |---|---|---|---|---|
/// | `0,0` | `0,03°` | `0,19°` | `0,19°` | `0,19°` |
/// | `0,3` | `0,03°` | **`28,40°`** (2 vértices) | **`18,37°`** (1) | **`0,89°`** (0) |
/// | `0,5` | `0,03°` | `35,18°` (5) | `0,18°` | `0,19°` |
/// | `0,7` | `0,03°` | `29,88°` (2) | `0,25°` | `0,25°` |
/// | `0,8` | `0,03°` | `15,13°` (1) | `0,27°` | `0,27°` |
/// | `0,9` | `0,03°` | `56,07°` (6) | `0,54°` | `0,54°` |
///
/// ⇒ **zero vértices acima de `5°` nas seis posições**, e o que sobrava era
/// **um**. De graça, as faces de área ZERO que o motor deixa na costura (juntas
/// em T que ele sela com uma face sem área) vão de `9` para `1` — *a troca da
/// diagonal de uma delas É a divisão em T*.
///
/// # As cinco cercas
///
/// ⚠️ **A cerca 1b nasceu de um gate VERMELHO do vizinho:** sem o limite de
/// TAMANHO a troca reescrevia a parede inteira de uma lâmina grossa, e o
/// `a_face_que_o_corte_deixa_tem_a_densidade_da_peca` reprovou **no CONTROLO**
/// dele. *Uma cura que melhora o controlo de outra cura apagou a régua dela* —
/// e o veredito certo é o que aquele gate já escreve: a face grossa de uma
/// lâmina grossa cura-se **ADENSANDO a lâmina**.
///
/// ⚠️ **A cerca 1a não tinha régua NENHUMA até esta wave**, e a mutação que a
/// apaga sobrevivia: o gate irmão mede **posições**, e uma troca de diagonal
/// **não move um vértice** — ela reescreve a LIGAÇÃO. ⇒
/// `a_ligacao_da_peca_sobrevive_ao_corte`.
fn endireita_as_lascas(
    pos: &[[f32; 3]],
    t: &mut [[u32; 3]],
    novo: &impl Fn(u32) -> bool,
    alvo: f32,
) -> usize {
    let mut trocas = 0usize;
    for _ in 0..PASSAGENS {
        let mut por_aresta: std::collections::BTreeMap<(u32, u32), Vec<usize>> =
            std::collections::BTreeMap::new();
        for (i, x) in t.iter().enumerate() {
            for (a, b) in [(x[0], x[1]), (x[1], x[2]), (x[2], x[0])] {
                por_aresta.entry((a.min(b), a.max(b))).or_default().push(i);
            }
        }
        let mut tocado = vec![false; t.len()];
        let mut feitas = 0usize;
        for i in 0..t.len() {
            if tocado[i] {
                continue;
            }
            let x = t[i];
            let (a, b, c) = (pos[x[0] as usize], pos[x[1] as usize], pos[x[2] as usize]);
            if aspecto(a, b, c) <= ASPECTO_DA_LASCA {
                continue;
            }
            // A aresta mais longa, **dirigida dentro desta face** — é dela que
            // sai a orientação das duas faces novas.
            let dirigidas = [(x[0], x[1]), (x[1], x[2]), (x[2], x[0])];
            let Some(&(u, v)) = dirigidas.iter().max_by(|p, q| {
                dist(pos[p.0 as usize], pos[p.1 as usize])
                    .total_cmp(&dist(pos[q.0 as usize], pos[q.1 as usize]))
            }) else {
                continue;
            };
            let k = (u.min(v), u.max(v));
            let Some(lista) = por_aresta.get(&k) else {
                continue;
            };
            if lista.len() != 2 {
                continue;
            }
            let Some(&j) = lista.iter().find(|&&j| j != i) else {
                continue;
            };
            if tocado[j] {
                continue;
            }
            // ⛔ **CERCA 1 — o par não pode ser da PEÇA, e tem de ter o TAMANHO
            // DA MALHA dela.**
            //
            // A 1.ª metade é a mesma lei da cerca do colapso, um nível acima: o
            // que o corte criou é nosso; o que estava lá antes não é.
            //
            // ⛔⛔ **A 2.ª metade foi escrita por um gate vermelho.** Sem ela a
            // troca reescrevia **a parede inteira de uma lâmina grossa** — o
            // leque de lascas com que o motor tapa a fronteira dela —, e o
            // `a_face_que_o_corte_deixa_tem_a_densidade_da_peca` reprovou **no
            // CONTROLO**: a face grossa passou a medir `0,042` onde tinha de
            // medir `≥ 0,125`. *Uma cura que melhora o controlo de outra cura
            // apagou a régua dela.* E o veredito certo é o que aquele gate já
            // escreve: **a face grossa de uma lâmina grossa cura-se ADENSANDO a
            // lâmina (§45)**, nunca trocando diagonais por cima.
            //
            // ⇒ a fronteira é o **tamanho**, e ela separa por uma ordem de
            // grandeza: na costura a aresta longa de uma lasca mede `0,8`–`1,7`
            // arestas da peça (medido nas seis posições do corte), e na parede
            // de uma lâmina mínima mede **`~40`**.
            if t[i].iter().all(|&w| !novo(w)) || t[j].iter().all(|&w| !novo(w)) {
                continue;
            }
            if dist(pos[u as usize], pos[v as usize]) > alvo * ESCALA_DA_COSTURA {
                continue;
            }
            let Some(&p) = t[i].iter().find(|&&w| w != u && w != v) else {
                continue;
            };
            let Some(&q) = t[j].iter().find(|&&w| w != u && w != v) else {
                continue;
            };
            // ⛔ **CERCA 2 — a aresta NOVA não pode já existir.** Se existir, a
            // troca escreve uma face repetida, que é a «almofada» que o passo
            // (3) existe para varrer.
            if por_aresta.contains_key(&(p.min(q), p.max(q))) {
                continue;
            }
            // ⛔ **CERCA 3 — as duas faces têm de ser PLANAS uma com a outra.**
            // É ela que protege a QUINA do corte: trocar a diagonal através da
            // aresta onde a parede encontra a casca mudaria a silhueta.
            let ni = normal_crua(a, b, c);
            let nj = normal_crua(
                pos[t[j][0] as usize],
                pos[t[j][1] as usize],
                pos[t[j][2] as usize],
            );
            let (li, lj) = (comprimento(ni), comprimento(nj));
            // ⚠️ Uma face de área ZERO não tem direcção e por isso não pode
            // discordar de ninguém — o par conta como plano, e é este braço que
            // alcança o triângulo degenerado (a junta em T que o motor sela com
            // uma face de área nula).
            let referencia = if li > 0.0 { ni } else { nj };
            if li > 0.0 && lj > 0.0 {
                let cos = (ponto(ni, nj) / (li * lj)).clamp(-1.0, 1.0);
                if cos.acos().to_degrees() > PLANAS_ATE_GRAUS {
                    continue;
                }
            }
            if comprimento(referencia) <= 0.0 {
                continue;
            }
            // As duas faces novas, com a orientação derivada de `u → v`.
            let (ti, tj) = ([v, p, q], [p, u, q]);
            let leia = |y: [u32; 3]| (pos[y[0] as usize], pos[y[1] as usize], pos[y[2] as usize]);
            let (ai, bi, ci) = leia(ti);
            let (aj, bj, cj) = leia(tj);
            // ⛔ **CERCA 4 — melhora ESTRITA**: é ela que faz o laço TERMINAR,
            // porque cada troca reduz estritamente o pior aspecto do par, logo
            // nenhum par pode voltar atrás e a saída não depende de
            // [`PASSAGENS`].
            //
            // ⚠️⚠️ **CARRIL NOMEADO, e a mutação que o apaga SOBREVIVE.** Ele
            // dispara **uma** vez nas seis posições do corte e a saída sem ele
            // é **byte-idêntica** (volume `−2,199e-6` · `−5,809e-6` ·
            // `−4,047e-6`, os mesmos dígitos). ⛔ Não foi possível construir a
            // fixtura que o torna observável: *o candidato é sempre uma lasca, e
            // trocar a diagonal de uma lasca melhora por construção.* Ele fica
            // pelo argumento de TERMINAÇÃO, que é uma propriedade do laço e não
            // da saída — e o `a_limpeza_e_um_ponto_fixo` mede a consequência.
            //
            // ⚠️ Escrito pelo `partial_cmp` de propósito: um `!(depois < antes)`
            // diz a mesma coisa e esconde o que acontece com `NaN` — aqui um
            // aspecto indefinido **recusa** a troca, que é o lado seguro.
            let antes = aspecto(a, b, c).max(aspecto(
                pos[t[j][0] as usize],
                pos[t[j][1] as usize],
                pos[t[j][2] as usize],
            ));
            let depois = aspecto(ai, bi, ci).max(aspecto(aj, bj, cj));
            if depois.partial_cmp(&antes) != Some(std::cmp::Ordering::Less) {
                continue;
            }
            // ⛔ **CERCA 5 — sem INVERSÃO.** Uma troca num quadrilátero não
            // convexo entrega uma face virada ao contrário.
            if ponto(normal_crua(ai, bi, ci), referencia) <= 0.0
                || ponto(normal_crua(aj, bj, cj), referencia) <= 0.0
            {
                continue;
            }
            t[i] = ti;
            t[j] = tj;
            tocado[i] = true;
            tocado[j] = true;
            feitas += 1;
        }
        trocas += feitas;
        if feitas == 0 {
            break;
        }
    }
    trocas
}

#[cfg(test)]
#[path = "costura_lascas_tests.rs"]
mod lascas_tests;
