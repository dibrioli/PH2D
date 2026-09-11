//! **A SONDA DA BANCADA** — quantiza os layouts que o oráculo exportou.
//!
//! É a resposta à pergunta que o §5 do `PLAN.md` diz que pode matar o plano:
//! *"o solver de quantização fecha?"* — medida **antes** de o nosso traçado (F3)
//! existir, sobre os patches que o oráculo já publica em texto.
//!
//! ⚠️ **Nada aqui lê um formato do oráculo.** A bancada (`layout.py`, fora da
//! árvore) converte `.patch`/`.corners` num ficheiro `.layout` de números; esta
//! sonda lê só isso. ADR-0162, Trilha B.
//!
//! ```text
//! cd .../Worktrees/line-sculpt3d && cargo test -p ph2d-quantize --release \
//!     --test oracle_bench -- --ignored --nocapture
//! ```

use std::path::Path;
use std::time::Instant;

use ph2d_quantize::{ArcSpec, Budget, Layout, PatchSpec, quantize_within};

/// Onde a bancada escreve os layouts — **fora** do repositório da engine.
const LAYOUTS: &str = "/home/enio/Documentos/Projetos/ph2d-quadbench/layouts";

/// O orçamento desta sonda para os layouts pequenos. ⚠️ Deliberadamente **acima**
/// do que os layouts do oráculo gastaram (máximo medido: 222 expansões, 447
/// resoluções), para que a coluna `prova` diga algo: com um teto justo, `NAO`
/// significaria só *"o teto"*.
const BUDGET: Budget = Budget {
    expansions: 4096,
    solves: 4096,
    augmentations: 250_000,
};

/// ⚠️ **Acima deste tamanho o orçamento encolhe**, e o das RESOLUÇÕES com ele.
/// O que custa nesta fase é a **heterogeneidade dos alvos**, não o número de
/// arcos: medido em 2026-08-20 numa grelha com alvos dispersos, 2 048 arcos
/// custam ~1,7 s por resolução contra ~0 ms com alvos uniformes
/// ([`tests/scaling.rs`]). Um teto de resoluções é o que impede a sonda de moer
/// em vez de dizer o que se passou.
const BIG: usize = 1024;
/// O orçamento para os layouts grandes.
///
/// ⚠️ **Ele é pequeno porque o custo por resolução não é.** Medido em 2026-08-20
/// na `sphere_noisy` (3 613 arcos): cada resolução custa ~2,3 s, então 256 delas
/// são **dez minutos** só para dizer *"não deu"*. Um teto em CONTAGEM não limita
/// o relógio — quem o limita é quem conhece o tamanho, e aqui é a sonda.
const BIG_BUDGET: Budget = Budget {
    expansions: 4,
    solves: 16,
    augmentations: 250_000,
};

/// Lê o formato neutro descrito em `layout.py`.
fn read_layout(path: &Path) -> Option<Layout> {
    let text = std::fs::read_to_string(path).ok()?;
    let tok: Vec<&str> = text.split_ascii_whitespace().collect();
    let mut i = 0usize;
    let num = |i: &mut usize| -> usize {
        let v = tok.get(*i).and_then(|s| s.parse().ok()).unwrap_or(0);
        *i += 1;
        v
    };
    let n_arcs = num(&mut i);
    let n_patches = num(&mut i);
    let mut arcs = Vec::with_capacity(n_arcs);
    for _ in 0..n_arcs {
        let target: f64 = tok.get(i).and_then(|s| s.parse().ok()).unwrap_or(1.0);
        i += 1;
        arcs.push(ArcSpec::new(target));
    }
    let mut patches = Vec::with_capacity(n_patches);
    for _ in 0..n_patches {
        let n_sides = num(&mut i);
        let mut sides = Vec::with_capacity(n_sides);
        for _ in 0..n_sides {
            let k = num(&mut i);
            sides.push((0..k).map(|_| num(&mut i) as u32).collect());
        }
        patches.push(PatchSpec { sides });
    }
    Layout::new(arcs, patches).ok()
}

/// Quantos quads o leque de um patch produz: `Σ e_i · e_{i+1}`.
///
/// ⚠️ Para um patch de 4 lados isto colapsa em `L_0 · L_1`, a grade — e é a
/// forma de o mesmo número servir todas as valências.
fn quads_of(corners: &[Vec<u32>]) -> u64 {
    corners
        .iter()
        .map(|e| {
            (0..e.len())
                .map(|i| u64::from(e[i]) * u64::from(e[(i + 1) % e.len()]))
                .sum::<u64>()
        })
        .sum()
}

#[test]
#[ignore = "sonda de bancada -- le os layouts que o oraculo exportou, fora da arvore (ADR-0162)"]
fn quantize_the_oracle_layouts() {
    let dir = Path::new(LAYOUTS);
    let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(dir)
        .expect("a bancada escreveu os layouts (python3 layout.py)")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "layout"))
        .collect();
    entries.sort();

    println!(
        "{:<20} {:>7} {:>6} {:>7} {:>6} {:>7} {:>8} {:>6} {:>9} {:>9} {:>8} {:>9}",
        "malha",
        "patches",
        "arcos",
        "meio-1/2",
        "expan",
        "fluxos",
        "aument",
        "prova",
        "custo",
        "limite",
        "quads",
        "ms"
    );
    for path in entries {
        let name = path.file_stem().unwrap_or_default().to_string_lossy();
        let Some(layout) = read_layout(&path) else {
            println!("{name:<20}    layout recusado (arco de bordo ou malformado)");
            continue;
        };
        let budget = if layout.arcs().len() > BIG {
            BIG_BUDGET
        } else {
            BUDGET
        };
        eprintln!("[bench] {name}: {} arcos…", layout.arcs().len());
        let t = Instant::now();
        match quantize_within(&layout, budget) {
            Ok((q, r)) => {
                let ms = t.elapsed().as_secs_f64() * 1000.0;
                println!(
                    "{:<20} {:>7} {:>6} {:>7} {:>6} {:>7} {:>8} {:>6} {:>9.2} {:>9.2} {:>8} {:>9.0}",
                    name,
                    layout.patches().len(),
                    layout.arcs().len(),
                    r.half_integral,
                    r.expansions,
                    r.solves,
                    r.augmentations,
                    if r.proved { "sim" } else { "NAO" },
                    r.cost,
                    r.lower_bound,
                    quads_of(&q.corners),
                    ms
                );
                assert_eq!(r.cap_binding, 0, "{name}: o teto da rede mordeu");
            }
            // ⚠️ **"Não deu" e "não existe" são coisas diferentes**, e imprimi-las
            // igual seria dizer que um layout perfeitamente quantizável é
            // impossível.
            Err(ph2d_quantize::SolveError::Exhausted { solves }) => println!(
                "{name:<20} {:>7} {:>6}   ORCAMENTO ESGOTADO ({solves} resolucoes, {:.0} ms)",
                layout.patches().len(),
                layout.arcs().len(),
                t.elapsed().as_secs_f64() * 1000.0
            ),
            Err(e) => println!("{name:<20}    SEM QUANTIZACAO REGULAR: {e:?}"),
        }
    }
}
