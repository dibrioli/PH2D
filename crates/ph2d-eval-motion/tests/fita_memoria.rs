//! ⭐ **A RÉGUA DA FITA contra a memória que ela RETÉM** (doc 131 §4/§7, C.11).
//!
//! `Fita::bytes()` é o que o orçamento (`FITA_BYTES`) compara; se ele declara menos do que a fita
//! retém, o orçamento é uma fração do que diz ser (medido antes da cura: `20,6×` pelo RSS). Este
//! binário conta o heap com o `dhat` (o `unsafe` de um alocador próprio é proibido na workspace):
//! o retido é o que ficou vivo, `bytes + 16 · blocos` — o cabeçalho e o arredondamento que o
//! `malloc` acrescenta a cada pedido —, e o `bytes()` tem de ficar a `±25 %` dele, com 3, 30 e 300
//! entradas, com streams de várias colunas, com curvas e com a dobra a trabalhar.
//!
//! Um único `#[test]`: o contador é global, e testes em paralelo no mesmo binário somariam-se.

use dhat::{HeapStats, Profiler};
use ph2d_eval_motion::Fita;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::external::{All, External};

#[global_allocator]
static ALOCADOR: dhat::Alloc = dhat::Alloc;

/// O que o `malloc` reserva para o que está vivo agora: o pedido e `16` por bloco.
fn vivos() -> isize {
    let h = HeapStats::get();
    isize::try_from(h.curr_bytes + 16 * h.curr_blocks).expect("cabe")
}

/// Um cenário: `k` entradas, das quais as `mudam` primeiras mudam em todo tique, `t` tiques,
/// `colunas` por stream.
struct Cenario {
    k: usize,
    mudam: usize,
    t: u64,
    colunas: usize,
    orcamento: usize,
}

/// O que um cenário mediu.
struct Medida {
    declarado: usize,
    retido: usize,
    mudancas: u64,
    tiques: u64,
}

fn publica(nomes: &[String], t: u64, mudam: usize, colunas: usize, agora: &mut All) {
    for (e, nome) in nomes.iter().enumerate() {
        let t = if e < mudam { t } else { 0 };
        #[expect(clippy::cast_precision_loss, reason = "um valor de teste")]
        let v = (t as usize * 31 + e) as f32;
        // `colunas == 0`: uma CURVA (uma coluna de 8 pontos — a alocação partilhada).
        let mut s = if colunas == 0 {
            Stream::new(8).with("P", Column::Vec2(vec![[v, -v]; 8]))
        } else {
            Stream::new(1).with("v", Column::Scalar(vec![v]))
        };
        if colunas > 1 {
            s = s.with("P", Column::Vec2(vec![[v, -v]]));
        }
        agora.insert(
            nome.clone(),
            External {
                rev: ph2d_nodegraph::external::fingerprint(&s),
                value: s,
            },
        );
    }
}

fn mede(c: &Cenario) -> Medida {
    let nomes: Vec<String> = (0..c.k).map(|e| format!("$input:entrada_{e}")).collect();
    let antes = vivos();
    let mut f = Fita::new();
    f.set_orcamento(c.orcamento);
    let mut agora = All::new();
    for t in 0..c.t {
        publica(&nomes, t, c.mudam, c.colunas, &mut agora);
        f.grava(t, &agora);
    }
    // O que o `agora` segura sozinho (a árvore, os nomes) sai; as colunas do último tique são da
    // fita também (partilhadas) e ficam.
    drop(agora);
    let retido = usize::try_from(vivos() - antes).expect("a fita retém algo");
    let declarado = f.bytes();
    drop(f);
    Medida {
        declarado,
        retido,
        mudancas: c.mudam as u64 * c.t,
        tiques: c.t,
    }
}

#[test]
fn a_fita_declara_a_memoria_que_retem() {
    let _perfil = Profiler::builder().testing().build();
    let sem_teto = usize::MAX;
    let c = |k, mudam, t, colunas, orcamento| Cenario {
        k,
        mudam,
        t,
        colunas,
        orcamento,
    };
    let cenarios = [
        ("3 entradas", c(3, 3, 5_000, 1, sem_teto)),
        ("3 entradas, 1 a mudar", c(3, 1, 5_000, 1, sem_teto)),
        ("30 entradas", c(30, 30, 1_000, 1, sem_teto)),
        ("300 entradas", c(300, 300, 300, 1, sem_teto)),
        ("30 entradas × 2 colunas", c(30, 30, 1_000, 2, sem_teto)),
        ("30 curvas de 8 pontos", c(30, 30, 1_000, 0, sem_teto)),
        ("300 entradas, a dobrar", c(300, 300, 1_500, 1, 4 << 20)),
        ("3 entradas, a dobrar", c(3, 3, 20_000, 1, 512 << 10)),
    ];
    let mut falhas = Vec::new();
    for (nome, c) in &cenarios {
        let m = mede(c);
        #[expect(clippy::cast_precision_loss, reason = "uma razão")]
        let (d, r) = (m.declarado as f64, m.retido as f64);
        #[expect(clippy::cast_precision_loss, reason = "uma razão")]
        let n = m.mudancas as f64;
        #[expect(clippy::cast_precision_loss, reason = "uma razão")]
        let tiques = m.tiques as f64;
        eprintln!(
            "fita {nome}: declara {} B · retém {} B — {:.1} × {:.1} B/mudança, {:.0} B/tique ({:.2}×)",
            m.declarado,
            m.retido,
            d / n,
            r / n,
            r / tiques,
            r / d
        );
        if (d - r).abs() > 0.25 * r {
            falhas.push(format!(
                "{nome}: declara {d} B e retém {r} B ({:.2}×)",
                r / d
            ));
        }
        if c.orcamento != sem_teto {
            assert!(
                m.declarado <= c.orcamento,
                "{nome}: o orçamento é {} e a fita declara {}",
                c.orcamento,
                m.declarado
            );
        }
    }
    assert!(
        falhas.is_empty(),
        "a régua da fita mente:\n{}",
        falhas.join("\n")
    );
}
