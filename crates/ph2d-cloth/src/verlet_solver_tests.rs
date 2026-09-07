//! ⭐⭐⭐ **AS LEIS DO SOLVER**, uma varredura de cada vez — irmão do
//! [`super::verlet_gesto_tests`], e o corte é *o que a RELAXAÇÃO faz com uma
//! restrição* (aqui) contra *o que o GESTO escreve* (lá).
//!
//! ⚠️ **Todos estes gates correm com o vértice INACTIVO**, e isso é o que os
//! torna leituras da relaxação e não do passo inteiro: a integração (§5.4) só
//! toca vértices activos, então com `activo = false` o [`Verlet::passo`] é
//! exactamente as varreduras. *Medir a relaxação através da integração seria
//! medir as duas e atribuir o resultado a uma.*
//!
//! ⛔⛔ **Estes gates existem porque a espec propõe 31 e o corpus só alcança
//! alguns.** Um traço do oráculo mede a composição de tudo; estas leis são
//! taxas e invariantes de UMA restrição, e um erro numa delas pode ser
//! compensado por outro erro no gesto sem nenhum traço se mexer.

use crate::V3;
use crate::verlet::{Alvo, Solver, Verlet};

/// Um solver de `k` varreduras, sem amortecimento.
fn solver(k: u32) -> Solver {
    Solver {
        varreduras: k,
        ..Solver::default()
    }
}

/// Prepara `n` vértices no repouso dado, todos INACTIVOS e com `φ = 1`.
fn parado(repouso: Vec<V3>) -> Verlet {
    let n = repouso.len();
    let mut v = Verlet::nascer(repouso);
    for i in 0..n {
        v.phi[i] = 1.0;
        v.activo[i] = false;
    }
    v
}

/// ⭐ **GATE 9 — cinco varreduras, `0,6`, e METADE para cada lado.** Numa
/// restrição estrutural isolada entre dois vértices livres com `φ = 1`, o erro
/// de comprimento após `k` varreduras é `(1 − 0,6)^k` do inicial.
///
/// Cada extremo leva `Δ/2`, logo juntos fecham `Δ = 0,6 ·` a folga — é por isso
/// que a taxa da estrutural é `0,6` e a de uma âncora (onde o outro extremo não
/// é vértice) é metade disso (gate 26).
#[test]
fn uma_restricao_estrutural_fecha_seis_decimos_da_folga_por_varredura() {
    let l = 1.0;
    for k in [1u32, 2, 5] {
        let mut v = parado(vec![[0.0, 0.0, 0.0], [l, 0.0, 0.0]]);
        v.construido[0] = true;
        v.restricoes.push(crate::verlet::Restricao {
            a: 0,
            b: Alvo::Vertice(1),
            l,
            s: 1.0,
        });
        // Esticada a `1,5 ℓ`: a folga inicial é `0,5`.
        v.x[1] = [1.5, 0.0, 0.0];
        v.passo(&solver(k));
        let folga = (v.x[1][0] - v.x[0][0]) - l;
        let esperado = 0.5 * 0.4f64.powi(k as i32);
        assert!(
            (folga - esperado).abs() < 1e-12,
            "k={k}: folga {folga} contra {esperado} -- a taxa da estrutural nao e' 0,6"
        );
        // ⭐ E os DOIS lados moveram-se por igual: `Δ/2` cada.
        assert!(
            (v.x[0][0] + (v.x[1][0] - 1.5) - 0.0).abs() < 1e-12,
            "os dois extremos nao levaram metade cada: {:?} {:?}",
            v.x[0],
            v.x[1]
        );
    }
}

/// ⭐⭐ **GATE 26 — a ÂNCORA vive na mesma lista e recebe `Δ/2`**, logo fecha
/// `0,3` por varredura e não `0,6`: o outro extremo não é vértice, então o
/// segundo `Δ/2` não é aplicado a ninguém (espec §5.2 nº 2).
///
/// O **pino** segue a mesma lei. ⭐ E o **corpo mole** fecha à MESMA taxa para
/// qualquer plasticidade — `0,3ρ` do lado do vértice mais `0,3(1−ρ)` do lado da
/// memória dá `0,3` —, *o que o distingue não é a taxa, é as duas pontas se
/// moverem.*
#[test]
fn as_tres_especies_de_alvo_proprio_fecham_tres_decimos_por_varredura() {
    let alvo = [1.0, 0.0, 0.0];
    let casos: [(&str, Alvo); 3] = [
        ("ancora", Alvo::Ancora),
        ("pino", Alvo::Repouso),
        ("corpo mole", Alvo::Memoria),
    ];
    for (nome, especie) in casos {
        for k in [1u32, 2, 5] {
            let mut v = parado(vec![alvo]);
            v.ancora[0] = alvo;
            v.memoria[0] = alvo;
            v.sigma[0] = 1.0;
            v.restricoes.push(crate::verlet::Restricao {
                a: 0,
                b: especie,
                l: 0.0,
                s: 1.0,
            });
            v.x[0] = [0.0, 0.0, 0.0];
            let mut s = solver(k);
            s.plasticidade = 0.5;
            v.passo(&s);
            // A SEPARAÇÃO entre o vértice e o alvo dele, seja qual for a
            // espécie — no corpo mole as duas pontas andam.
            let b = match especie {
                Alvo::Memoria => v.memoria[0],
                Alvo::Repouso => v.repouso[0],
                _ => v.ancora[0],
            };
            let sep = b[0] - v.x[0][0];
            let esperado = 1.0 * 0.7f64.powi(k as i32);
            assert!(
                (sep - esperado).abs() < 1e-12,
                "{nome} k={k}: separacao {sep} contra {esperado} -- a taxa nao e' 0,3"
            );
        }
    }
}

/// ⭐⭐⭐ **GATE 27 — o alvo de cada espécie é lido no INSTANTE da projecção, e a
/// MEMÓRIA DE FORMA ANDA** (espec §5.2 nº 3).
///
/// Os dois convergem para **`(1 − ρ)·A₀ + ρ·B₀`**, que é a combinação que a
/// projecção conserva: cada varredura move `A` de `+0,3ρ(B−A)` e `B` de
/// `−0,3(1−ρ)(B−A)`, e `(1−ρ)A + ρB` fica invariante.
/// ⛔ **Não é o ponto médio:** com `ρ = 0` o encontro é em `A₀` (a memória vai ter
/// com o vértice) e com `ρ = 1` é em `B₀`.
///
/// ⚠️ **CONTROLO, e é ele que separa o corpo mole das outras duas:** com a âncora
/// de deformação o alvo fica **exactamente** onde o gesto o pôs, ao bit, no fim
/// das varreduras. *Um port com um braço só, guardado pela igualdade de índices,
/// congela a memória de forma e a plasticidade deixa de existir sem aviso.*
#[test]
fn a_memoria_de_forma_anda_e_o_encontro_nao_e_o_ponto_medio() {
    let (a0, b0) = (0.0f64, 1.0f64);
    for rho in [0.0f64, 0.25, 0.5, 1.0] {
        let mut v = parado(vec![[a0, 0.0, 0.0]]);
        v.memoria[0] = [b0, 0.0, 0.0];
        v.restricoes.push(crate::verlet::Restricao {
            a: 0,
            b: Alvo::Memoria,
            l: 0.0,
            s: 1.0,
        });
        let mut s = solver(200);
        s.plasticidade = rho;
        v.passo(&s);
        let encontro = (1.0 - rho) * a0 + rho * b0;
        assert!(
            (v.x[0][0] - encontro).abs() < 1e-9 && (v.memoria[0][0] - encontro).abs() < 1e-9,
            "ρ={rho}: vertice {} e memoria {} nao se encontraram em {encontro}",
            v.x[0][0],
            v.memoria[0][0]
        );
        // ⛔ E o ponto MÉDIO só coincide com o encontro em `ρ = 0,5`.
        if (rho - 0.5).abs() > 1e-12 {
            assert!(
                (encontro - 0.5 * (a0 + b0)).abs() > 1e-9,
                "ρ={rho}: a fixtura nao separa o encontro do ponto medio"
            );
        }
    }
    // CONTROLO: a âncora de deformação não se mexe, ao bit.
    let mut v = parado(vec![[0.0, 0.0, 0.0]]);
    v.ancora[0] = [1.0, 0.0, 0.0];
    v.sigma[0] = 1.0;
    v.restricoes.push(crate::verlet::Restricao {
        a: 0,
        b: Alvo::Ancora,
        l: 0.0,
        s: 1.0,
    });
    v.passo(&solver(200));
    assert_eq!(
        v.ancora[0],
        [1.0, 0.0, 0.0],
        "a ancora de deformacao MOVEU-SE -- so' o corpo mole tem a ponta que anda"
    );
}

/// ⭐⭐ **GATE 8 — o padrão de restrições numa grelha regular é `4 + 2 + 4` por
/// vértice interior**, sem duplicados DENTRO de uma construção (espec §3.1).
///
/// As **4** arestas (`h`) + as **2** «diagonais longas» N-S / E-O (`2h`, que é o
/// papel de dobra) + as **4** diagonais N-E… (`√2·h`, cisalhamento). ⛔ **Nenhuma
/// diagonal do quad é VIZINHA** — ela só aparece como par do anel. ⇒ *a rigidez
/// de dobra não tem modelo próprio: é a restrição de distância ao segundo
/// vizinho pelo anel.*
#[test]
fn a_construcao_de_um_vertice_interior_da_quatro_mais_dois_mais_quatro() {
    let h = 0.25;
    // Um vértice no centro com os 4 vizinhos de grelha.
    let mut v = parado(vec![
        [0.0, 0.0, 0.0],
        [0.0, -h, 0.0],
        [-h, 0.0, 0.0],
        [h, 0.0, 0.0],
        [0.0, h, 0.0],
    ]);
    v.construir(0, &[1, 2, 3, 4]);
    assert_eq!(
        v.restricoes.len(),
        10,
        "a construcao de um vertice interior tem de dar 10 restricoes"
    );
    let mut arestas = 0;
    let mut longas = 0;
    let mut curtas = 0;
    for r in &v.restricoes {
        assert!(
            matches!(r.b, Alvo::Vertice(_)),
            "a construcao so' cria estruturais"
        );
        if (r.l - h).abs() < 1e-12 {
            arestas += 1;
        } else if (r.l - 2.0 * h).abs() < 1e-12 {
            longas += 1;
        } else if (r.l - h * 2.0f64.sqrt()).abs() < 1e-12 {
            curtas += 1;
        } else {
            panic!("comprimento de repouso inesperado: {}", r.l);
        }
    }
    assert_eq!(
        (arestas, longas, curtas),
        (4, 2, 4),
        "o padrao nao e' 4 + 2 + 4"
    );
    // ⛔⛔ **Sem duplicados DENTRO de uma construção — e a régua tem de ser
    // sobre vértices DIFERENTES.** A 1.ª redacção repetia `construir(0, ..)` e
    // era VÁCUA: aquela porta sai cedo num vértice já construído, então apagar o
    // registo de pares inteiro deixava-a verde (mutação medida em 06/09).
    // *O registo existe para o par que DOIS vértices vizinhos ambos criam.*
    let n = 4usize;
    let h = 0.25;
    let mut pos = Vec::new();
    for j in 0..n {
        for i in 0..n {
            pos.push([i as f64 * h, j as f64 * h, 0.0]);
        }
    }
    let idx = |i: usize, j: usize| u32::try_from(j * n + i).expect("u32");
    let mut v = parado(pos);
    let mut construidos = 0;
    for j in 1..n - 1 {
        for i in 1..n - 1 {
            v.construir(
                idx(i, j),
                &[idx(i, j - 1), idx(i - 1, j), idx(i + 1, j), idx(i, j + 1)],
            );
            construidos += 1;
        }
    }
    assert!(
        construidos > 1,
        "um vertice so' nao testa o registo de pares"
    );
    let mut chaves: Vec<(u32, u32)> = v
        .restricoes
        .iter()
        .map(|r| match r.b {
            Alvo::Vertice(b) if r.a < b => (r.a, b),
            Alvo::Vertice(b) => (b, r.a),
            _ => unreachable!("a construcao so' cria estruturais"),
        })
        .collect();
    let antes = chaves.len();
    chaves.sort_unstable();
    chaves.dedup();
    assert_eq!(
        chaves.len(),
        antes,
        "a construcao deixou {} pares repetidos -- o registo de duplicados nao esta' a guardar",
        antes - chaves.len()
    );
}
