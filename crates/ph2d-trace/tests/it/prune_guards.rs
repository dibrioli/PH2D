//! **OS GATES DA PODA** — ver [`ph2d_trace::prune`].
//!
//! ⚠️ **A poda shipa DESLIGADA** ([`ph2d_trace::prune::PRUNE_STEMS`]), por medição: ela
//! cura a topologia (irregulares `18 → 9` na esfera lisa, e a orelha passa a empatar com
//! o oráculo) e **colapsa a geometria** (enviesamento `18° → 38°`, dobras `0 → 29`),
//! porque um patch a valer um terço de esfera é grande demais para o achatamento actual.
//!
//! ⛔ **Um interruptor desligado sem gate é código morto**, e código morto apodrece
//! antes de o bloqueio que o trava sair da frente. Estes gates correm a poda pelo
//! **caminho real** ([`ph2d_trace::trace_patches_with`]) e provam as guardas dela — as
//! três que a medição de 2026-08-23 obrigou a acrescentar, uma de cada vez, depois de
//! cada uma faltar.
//!
//! ⚠️ **Pelo caminho REAL e não pela função crua**, e a diferença mordeu: chamar o
//! [`ph2d_trace::prune::prune_stems`] sobre as paredes do passeio devolve **zero**
//! remoções, porque o layout cru ainda tem lascas e as guardas julgam contra um estado
//! que a limpeza ainda ia mudar. *Um gate que reconstrói o caminho mede a reconstrução.*

use ph2d_crossfield::{Dual, solve_miq, vertex_index};
use ph2d_mesh::{Mesh, shapes};
use ph2d_trace::{PatchLayout, trace_patches_with};

/// ⚠️ **REMALHA antes**, como o produto faz. Sem isso os dois polos de uma
/// `uv_sphere` levam singularidades de índice `4` e a esfera fica com **duas**
/// singularidades em vez de oito — a soma de Poincaré–Hopf continua `8`, e é a
/// *contagem* que muda. *A invariante é a soma; a contagem é o produto* — e um gate
/// sobre a contagem numa fixtura crua mede a `uv_sphere`, não a cadeia.
///
/// ⚠️ **`48×72` e não `24×36`, e a razão é a fixtura conter o fenómeno:** na grossa a
/// poda não encontra **um único** toco removível — as guardas recusam tudo —, e o gate
/// da presença reprovaria sobre uma peça que simplesmente não tem o defeito. *Trocar a
/// fixtura até o gate passar seria o erro; trocá-la porque a primeira não contém o que
/// se mede é a regra.*
fn fixture() -> Mesh {
    let mut m = shapes::uv_sphere(48, 72, 1.0);
    m.triangulate();
    ph2d_remesh_iso::remesh_isotropic(&mut m, ph2d_remesh_iso::ALPHA);
    m.triangulate();
    m
}

/// Os dois lados: sem poda e com poda, sobre a mesma peça e o mesmo campo.
fn both() -> (PatchLayout, PatchLayout, Vec<i32>) {
    let mesh = fixture();
    let dual = Dual::build(&mesh);
    let (field, _) = solve_miq(&dual);
    let index = vertex_index(&mesh, &dual, &field);
    (
        trace_patches_with(&mesh, &dual, &field, false),
        trace_patches_with(&mesh, &dual, &field, true),
        index,
    )
}

/// Os nós do traçado — as pontas de todos os arcos.
fn nodes(layout: &PatchLayout) -> std::collections::BTreeSet<u32> {
    layout
        .arc_chain
        .iter()
        .filter_map(|c| Some((*c.first()?, *c.last()?)))
        .flat_map(|(a, b)| [a, b])
        .collect()
}

/// ⭐⭐ **PRESENÇA: a poda remove tocos, e o relatório diz quantos.**
///
/// ⛔ Sem este lado, todos os outros gates ficariam verdes sobre uma função que devolve
/// o que recebeu.
#[test]
fn the_prune_removes_stems_and_says_how_many() {
    let (before, after, index) = both();
    assert_eq!(
        before.report.pruned, 0,
        "o caminho SEM poda reportou remocoes"
    );
    assert!(
        after.report.pruned > 0,
        "a poda nao removeu nada nesta fixtura -- ou e' um `if` morto, ou a fixtura deixou \
         de ter nos em vertice regular (e ai' o gate mede o nada)"
    );
    assert!(
        after.side_arcs.len() < before.side_arcs.len(),
        "removeu {} arcos e o numero de patches nao desceu ({} -> {})",
        after.report.pruned,
        before.side_arcs.len(),
        after.side_arcs.len()
    );
    // ⭐ **E o que ela remove é o que o diagnóstico nomeou**: nós em vértice REGULAR.
    let regular = |l: &PatchLayout| {
        nodes(l)
            .into_iter()
            .filter(|&v| index.get(v as usize).copied().unwrap_or(0) == 0)
            .count()
    };
    let (was, is) = (regular(&before), regular(&after));
    assert!(
        is < was,
        "a poda correu e os nos em vertice regular nao desceram ({was} -> {is}) -- ela esta' \
         a remover outra coisa que nao os tocos"
    );
}

/// ⭐⭐⭐ **AS GUARDAS 4 e 5: o CAMPO sobrevive à poda.**
///
/// ⛔ **Estes dois números são a diferença entre a poda e apagar o traçado.** Sem a
/// guarda 4 as quatro fixturas iam a `2` patches com **6 das 8 singularidades enterradas
/// dentro deles** — e as guardas topológicas ficavam todas contentes, porque uma esfera
/// cortada por um laço fechado dá dois discos com `χ = 2`. Sem a guarda 5, uma
/// singularidade que estava numa junta passava a ficar **no meio** de um arco fundido:
/// mantinha o grau e deixava de ser canto de patch nenhum.
#[test]
fn the_prune_never_buries_a_singularity() {
    let (before, after, index) = both();
    // ⚠️ **A INVARIANTE é a SOMA, e a fixtura confere-se pelas DUAS.** Poincaré–Hopf
    // exige `Σ = 8` numa esfera qualquer que seja a contagem; o que este gate precisa é
    // de singularidades **em número** para haver o que enterrar.
    let sum: i32 = index.iter().sum();
    assert_eq!(
        sum, 8,
        "a fixtura deixou de ser uma esfera (Sigma indice = {sum})"
    );
    let sing: Vec<u32> = index
        .iter()
        .enumerate()
        .filter(|(_, k)| **k != 0)
        .filter_map(|(v, _)| u32::try_from(v).ok())
        .collect();
    assert!(
        sing.len() >= 4,
        "a fixtura tem so' {} singularidades -- com poucas nao ha' o que enterrar",
        sing.len()
    );
    let (was, now) = (nodes(&before), nodes(&after));
    for v in sing {
        assert!(
            !was.contains(&v) || now.contains(&v),
            "a poda enterrou a singularidade {v}: ela era no' do tracado e deixou de ser. \
             Um canto que nao esta' onde a grade tem de virar e' o defeito que a poda \
             existe para curar -- ver a guarda 5 em `prune.rs`"
        );
    }
}

/// ⭐⭐ **A GUARDA 6: o que sai da poda o F4 ainda resolve.**
///
/// ⛔ Sem ela a orelha saía `Infeasible` — *«nenhuma quantização regular existe»*, não
/// «acabou o orçamento». ⚠️ E nenhum predicado **local** o soube prever: a primeira
/// tentativa foi a auto-adjacência, deduzida de uma paridade que se revelou coincidência
/// de quatro amostras. *A única guarda honesta foi perguntar à fase seguinte.*
#[test]
fn what_the_prune_leaves_still_quantizes() {
    let (_, after, _) = both();
    assert!(after.report.pruned > 0, "a poda nao correu");
    let spec = after
        .to_layout(0.25)
        .expect("o layout podado deixou de fechar");
    ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
        .expect("o F4 recusou o layout podado -- a guarda 6 nao esta' a morder");
}

/// ⭐ **AUSÊNCIA: o que SHIPA não poda.**
///
/// ⚠️ Ele lê a constante e não o comportamento de propósito: quem a ligar tem de vir
/// aqui, ler a tabela da rejeição e decidir — em vez de descobrir pelo gate vermelho de
/// outra crate.
///
/// ⚠️ **O `assert_eq!` contra `false` em vez de `assert!(!…)`** é para o clippy não o
/// ver como asserção de valor constante e o apagar do binário — *um gate optimizado
/// para fora é um gate que não existe.*
#[test]
fn the_shipped_trace_does_not_prune() {
    assert_eq!(
        ph2d_trace::prune::PRUNE_STEMS,
        std::hint::black_box(false),
        "a poda foi LIGADA. Ela cura a topologia e colapsa a geometria (enviesamento \
         18° -> 38°, dobras 0 -> 29 na esfera lisa) -- leia a tabela em \
         `prune::PRUNE_STEMS` antes de mexer aqui"
    );
}
