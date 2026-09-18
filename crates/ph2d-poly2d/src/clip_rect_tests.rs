//! Os gates do corte por rectângulo — ver o cabeçalho do [`super`].

use super::submesh_in_rect;
use crate::Mesh2d;

/// Uma grelha `n × n` sobre `[0, 100]²`, com um atributo por vértice: o próprio `x`.
///
/// ⭐ O atributo é uma função LINEAR da posição de propósito: assim o valor certo em qualquer ponto
/// novo é conhecido em forma fechada, e a régua da interpolação não precisa de uma tabela.
fn grelha(n: usize) -> (Mesh2d, Vec<f64>) {
    let lado = n + 1;
    let mut rest = Vec::new();
    for j in 0..lado {
        for i in 0..lado {
            rest.push([i as f64 * 100.0 / n as f64, j as f64 * 100.0 / n as f64]);
        }
    }
    let mut tris = Vec::new();
    for j in 0..n {
        for i in 0..n {
            let a = (j * lado + i) as u32;
            let (b, c, d) = (a + 1, a + lado as u32, a + lado as u32 + 1);
            tris.push([a, b, c]);
            tris.push([b, d, c]);
        }
    }
    let attrs = rest.iter().map(|p| p[0]).collect();
    (
        Mesh2d {
            rest,
            tris,
            size: [100, 100],
        },
        attrs,
    )
}

/// A área com sinal de uma malha.
fn area(m: &Mesh2d) -> f64 {
    m.tris
        .iter()
        .map(|t| {
            let [a, b, c] = t.map(|i| m.rest[i as usize]);
            ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs() / 2.0
        })
        .sum()
}

/// ⭐⭐⭐ **UM RECTÂNGULO QUE CONTÉM TUDO DEVOLVE A MALHA BYTE A BYTE.**
///
/// ⚠️ **É este curto-circuito que protege toda sprite normal:** a porta entra no caminho de TODA
/// imagem presa, e sem ele cada uma pagaria uma soldadura e uma reordenação de vértices por quadro.
///
/// (Mutação: apagar o `dentro_de_todos` ⇒ RED, os índices mudam de ordem.)
#[test]
fn um_rectangulo_que_contem_tudo_devolve_a_malha_ao_bit() {
    let (m, a) = grelha(4);
    let (out, oa) = submesh_in_rect(&m, &a, 1, [-1.0, -1.0, 101.0, 101.0]).expect("sobra tudo");
    assert_eq!(out.rest, m.rest);
    assert_eq!(out.tris, m.tris);
    assert_eq!(oa, a);
}

/// ⭐⭐⭐ **O CORTE CONSERVA A ÁREA** — a soma das nove fatias é a malha inteira.
///
/// ⚠️ **É a régua que apanha os dois defeitos opostos de uma vez:** um corte que perde um triângulo
/// soma a menos, e um que emite o mesmo pedaço em duas fatias soma a mais.
#[test]
fn as_nove_fatias_somam_a_malha_inteira() {
    let (m, a) = grelha(6);
    let xs = [0.0, 17.0, 63.0, 100.0];
    let ys = [0.0, 11.0, 42.0, 100.0];
    let mut soma = 0.0;
    for r in 0..3 {
        for c in 0..3 {
            if let Some((f, _)) = submesh_in_rect(&m, &a, 1, [xs[c], ys[r], xs[c + 1], ys[r + 1]]) {
                soma += area(&f);
            }
        }
    }
    let inteira = area(&m);
    assert!(
        (soma - inteira).abs() < 1e-9 * inteira,
        "as nove fatias somaram {soma} contra {inteira} da malha inteira"
    );
}

/// ⭐⭐⭐ **O ATRIBUTO DO VÉRTICE NOVO É O DA POSIÇÃO DELE** — medido em forma fechada.
///
/// ⛔ Um vértice de corte que herdasse o atributo de uma ponta (em vez de o interpolar) poria o peso
/// de um osso inteiro na linha da fatia, e a arte partir-se-ia exactamente ali.
#[test]
fn o_atributo_do_vertice_novo_sai_da_posicao_dele() {
    let (m, a) = grelha(3);
    let (f, fa) = submesh_in_rect(&m, &a, 1, [17.0, 0.0, 63.0, 100.0]).expect("sobra");
    let pior = f
        .rest
        .iter()
        .zip(&fa)
        .map(|(p, v)| (p[0] - v).abs())
        .fold(0.0_f64, f64::max);
    assert!(
        pior < 1e-9,
        "o atributo desviou {pior} da lei linear que o corpus declara"
    );
}

/// Uma grelha `n × n` com as posições TORTAS — a fixtura que contém o fenómeno da soldadura.
///
/// ⛔⛔ **A grelha regular NÃO o contém, e isso foi medido:** com vértices em múltiplos exactos, o
/// ponto de corte sai bit a bit igual pelos dois sentidos e a mutação que apaga a canonicalização
/// **SOBREVIVE**. Sobre pares genéricos, `28 %` deles divergem (sonda de `200 000` pares). *Uma
/// régua cuja fixtura não contém o fenómeno não afirma nada — e ela fica VERDE a dizê-lo.*
fn grelha_torta(n: usize) -> (Mesh2d, Vec<f64>) {
    let (mut m, _) = grelha(n);
    let mut s = 0x2545_F491_4F6C_DD1D_u64;
    let mut rnd = || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        (s >> 11) as f64 / (1u64 << 53) as f64
    };
    for p in &mut m.rest {
        p[0] += rnd() * 7.3 - 3.65;
        p[1] += rnd() * 7.3 - 3.65;
    }
    let attrs = m.rest.iter().map(|p| p[0]).collect();
    (m, attrs)
}

/// ⭐⭐ **A MALHA CORTADA É CONFORME — sem fendas.**
///
/// ⚠️⚠️ **É aqui que a canonicalização do corte se prova:** dois triângulos vizinhos percorrem a
/// aresta partilhada em sentidos opostos, e sem lerp a partir da ponta menor os dois pontos de corte
/// diferem no último bit — a soldadura falha e fica uma FENDA que nenhuma régua de área vê.
///
/// A prova é topológica: numa malha conforme **toda aresta interior é partilhada por exactamente
/// dois triângulos**, e as de fronteira por um.
#[test]
fn a_malha_cortada_nao_tem_fendas() {
    let (m, a) = grelha_torta(5);
    let (f, _) = submesh_in_rect(&m, &a, 1, [13.37, 29.71, 71.13, 83.29]).expect("sobra");
    let mut contagem: std::collections::BTreeMap<(u32, u32), usize> = Default::default();
    for t in &f.tris {
        for k in 0..3 {
            let (u, v) = (t[k], t[(k + 1) % 3]);
            *contagem.entry((u.min(v), u.max(v))).or_default() += 1;
        }
    }
    let maus: Vec<_> = contagem.iter().filter(|(_, n)| **n > 2).collect();
    assert!(
        maus.is_empty(),
        "{} aresta(s) com mais de dois donos — a malha sobrepoe-se: {:?}",
        maus.len(),
        &maus[..maus.len().min(4)]
    );
    // ⚠️⚠️ **A RÉGUA QUE DE FACTO MORDE, e a primeira redacção não era esta.** Uma fenda cria DOIS
    // vértices distintos quase no mesmo sítio, e então a aresta partilhada vira duas arestas com UM
    // dono cada — *nenhuma passa de dois donos, e o gate acima fica verde sobre a fenda*. Medido: a
    // mutação que apaga a canonicalização sobrevivia à contagem de donos e morre a esta.
    let mut quase = Vec::new();
    for i in 0..f.rest.len() {
        for j in i + 1..f.rest.len() {
            let (a, b) = (f.rest[i], f.rest[j]);
            let d = (a[0] - b[0]).hypot(a[1] - b[1]);
            if d > 0.0 && d < 1e-9 {
                quase.push((i, j, d));
            }
        }
    }
    assert!(
        quase.is_empty(),
        "{} par(es) de vertices a menos de 1e-9 um do outro — a soldadura partiu-se e ficou uma \
         FENDA (um fio de fundo a atravessar a arte): {:?}",
        quase.len(),
        &quase[..quase.len().min(4)]
    );
    // ⛔ O CONTROLO: uma malha soldada tem MENOS vértices que triângulos × 3 — sem soldadura
    // nenhuma este número seria exactamente `tris × 3`, e o gate acima passaria por vacuidade
    // (arestas nunca partilhadas nunca passam de um dono).
    assert!(
        f.rest.len() < f.tris.len() * 3,
        "{} vertices para {} triangulos — nada foi soldado",
        f.rest.len(),
        f.tris.len()
    );
}

/// ⛔ **UM RECTÂNGULO FORA DA MALHA DEVOLVE `None`** — e não uma malha vazia, que o chamador
/// desenharia como um pedaço invisível sem saber porquê.
#[test]
fn fora_da_malha_nao_ha_sub_malha() {
    let (m, a) = grelha(3);
    assert!(submesh_in_rect(&m, &a, 1, [200.0, 200.0, 300.0, 300.0]).is_none());
}

/// ⭐ **SEM ATRIBUTOS a saída também não tem** — a tabela vazia é a lei DERIVADA, e ela tem de
/// atravessar o corte sem virar uma tabela de zeros.
#[test]
fn sem_atributos_a_saida_tambem_nao_tem() {
    let (m, _) = grelha(3);
    let (f, fa) = submesh_in_rect(&m, &[], 0, [17.0, 0.0, 63.0, 100.0]).expect("sobra");
    assert!(fa.is_empty(), "{} atributos nascidos do nada", fa.len());
    assert!(!f.tris.is_empty());
}
