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
//! # ⛔ O que NÃO entra, medido
//!
//! O **flip de arestas** ([`ph2d_mesh::relax_valence`]) por cima disto compra
//! `640 → 632` piores-que-`20` e leva o `MAX` de `33` para `26` — e paga por
//! mudar a LIGAÇÃO da malha **longe do corte**, que é a outra metade da
//! propriedade que a cerca 1 defende. *Uma cura que muda a peça inteira para
//! ganhar oito triângulos não é uma cura.*

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
/// ⚠️ **Os `~632` que sobram NÃO são arestas curtas** — são cunhas finas onde a
/// curva de interseção passa rente a um vértice da peça, e nenhum colapso as
/// alcança. Elas ficam **nomeadas**: a cura delas mexeria na malha da peça.
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
