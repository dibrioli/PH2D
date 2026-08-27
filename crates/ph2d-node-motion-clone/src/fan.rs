//! **O ATRASO POR CÓPIA** — o *Shape Time Offset* do Cavalry Duplicator (doc 89, folha 08).
//!
//! ## A recusa dissolveu, e quem a dissolveu foi outra wave desta mesma linha
//!
//! A célula respondia **NÃO** com a razão escrita *"as cópias são LINHAS, não sub-cooks"* — e
//! isso era verdade enquanto um nó só podia cozinhar a própria entrada **uma vez**. As
//! `TimeFans` do [ADR-0163] deixam-no cozinhá-la em **N instantes**, que é literalmente
//! *retimar cada cópia*. ⚠️ §0.0: *quem move o número que tornava algo inalcançável tem de
//! reconferir a nota.*
//!
//! ## O que este arquivo faz, e o que ele NÃO faz
//!
//! Ele monta o leque: a cópia `c` lê a entrada em `t − c · offset`, ou seja **para trás no
//! tempo**, que é o sentido que um rasto de cópias quer (a primeira é o agora, as outras são
//! o passado). O que ele não faz é a concatenação — essa é a `clone_fanned` do `lib.rs`.
//!
//! ⚠️ **`time_offset = 0` não monta leque nenhum**, e é isso que faz todo documento de hoje
//! cozinhar exactamente como antes: um mapa vazio deixa o cook no caminho de sempre.

use super::{SIZE_IDENTITY, radial, taper_t};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::TimeFans;
use ph2d_nodegraph::graph::Graph;
use ph2d_nodegraph::node::{NodeOp, RECOMMENDED_MAX_ELEMENTS, param_as_count};
use ph2d_nodegraph::time::{TimeMap, TimeMode};

/// O param que liga o leque, em SEGUNDOS por cópia.
pub(crate) const TIME_OFFSET: &str = "time_offset";

/// Abaixo desta magnitude o atraso é zero e o leque não se monta.
///
/// ⚠️ Não é gosto: um `offset` minúsculo montaria `k` cozeduras que devolvem **o mesmo
/// instante** — o custo inteiro do leque pelo desenho de sempre. O piso é um centésimo de
/// tique a 60 fps, abaixo do qual nem o relógio do shell distingue as fatias.
pub(crate) const MIN_OFFSET: f32 = 1.0 / 6000.0;

/// **Os leques desta marcha** — a assinatura que o shell junta às outras duas
/// (`motion.trail` e `motion.emitter`).
///
/// ⚠️ O `tick_seconds` **não** entra na conta aqui, ao contrário do rastro: o atraso deste nó
/// é autorado em SEGUNDOS e não em tiques, porque o que o artista quer dizer é *"meio segundo
/// entre cópias"* e não *"trinta quadros"* — um documento não pode mudar de aparência quando
/// a taxa de quadros muda. Ele fica na assinatura porque a porta é a mesma dos irmãos.
pub fn time_fans(
    graph: &Graph,
    ops: &dyn ph2d_nodegraph::cook::OpResolver,
    _tick_seconds: f64,
) -> TimeFans {
    let mut fans = TimeFans::new();
    for inst in graph.nodes() {
        if inst.type_name != super::MANIFEST.name {
            continue;
        }
        let Some(manifest) = ops.resolve(inst.type_id()).map(NodeOp::manifest) else {
            continue;
        };
        let overrides = graph.node_param_overrides(inst.id);
        let p = |name: &str| {
            overrides
                .and_then(|o| o.get(name).copied())
                .or_else(|| manifest.param_default(name))
                .unwrap_or(0.0)
        };
        let offset = p(TIME_OFFSET);
        if offset.abs() < MIN_OFFSET {
            continue;
        }
        let k = param_as_count(p("count"), RECOMMENDED_MAX_ELEMENTS).max(1);
        // ⚠️ **A cópia 0 é o AGORA, exactamente** (`offset = 0` no primeiro mapa), então o
        // conjunto que já existia continua ancorado onde estava e as outras cópias é que
        // recuam. Ancorá-lo no passado moveria o desenho inteiro ao ligar o knob.
        let maps: Vec<TimeMap> = (0..k)
            .map(|c| TimeMap {
                mode: TimeMode::Scale,
                scale: 1.0,
                offset: -f64::from(offset) * c as f64,
                ..TimeMap::default()
            })
            .collect();
        fans.insert(inst.id, maps);
    }
    fans
}

/// **AS CÓPIAS COM RELÓGIOS DIFERENTES** — a `clone_stream` quando cada cópia tem a SUA
/// entrada (doc 89, folha 08 · o *Shape Time Offset* do Cavalry).
///
/// ⚠️ **Ela não replica: ela CONCATENA.** Com o leque ligado as `k` fatias podem ter contagens
/// diferentes (um `sim.spawn` a montante nasce e mata entre os instantes), então a saída é a
/// **soma das contagens** e a **união das colunas** — a disciplina que o `motion.combine` já
/// escreve. Uma coluna que só existe nalgumas fatias é preenchida com a IDENTIDADE dela nas
/// outras, senão as linhas ficariam desalinhadas em silêncio.
///
/// ⚠️ **Ela é irmã da [`clone_stream`] e não a substitui**, e a razão é a byte-identidade: o
/// caminho de sempre (uma entrada, `k` cópias) tem de continuar a produzir exactamente os
/// mesmos bits para todo documento salvo. O gate
/// `the_fanned_law_agrees_with_the_plain_one_when_every_slice_is_the_same` prova que as duas
/// concordam onde se sobrepõem — que é a única coisa que impede duas leis de divergirem.
pub(crate) fn clone_fanned(
    slices: &[&Stream],
    place: &dyn Fn(usize) -> radial::Placement,
    scale_taper: f32,
    rot_taper: f32,
) -> Stream {
    let k = slices.len();
    let total: usize = slices.iter().map(|s| s.count()).sum();
    let scaling = scale_taper != 1.0;
    let turning = rot_taper != 0.0;
    let factor = |copy: usize| 1.0 + (scale_taper - 1.0) * taper_t(copy, k);
    let turn = |copy: usize| rot_taper * taper_t(copy, k);
    let mut out = Stream::new(total);

    // A UNIÃO dos nomes, na ordem em que a primeira fatia que os traz os apresenta — uma
    // ordem estável é o que faz o replay ser byte-idêntico.
    let mut names: Vec<String> = Vec::new();
    for s in slices {
        for (name, _) in s.columns() {
            if !names.iter().any(|n| n == name) {
                names.push(name.clone());
            }
        }
    }

    for name in &names {
        // O DIM da coluna sai da primeira fatia que a tem; as que não a têm contribuem com a
        // identidade dela, repetida pela própria contagem.
        let proto = slices
            .iter()
            .find_map(|s| s.get(name))
            .expect("a uniao so' tem nomes que alguma fatia traz");
        match (name.as_str(), proto) {
            // ⚠️ O índice global e a contagem são do CONJUNTO, como no caminho de sempre —
            // uma rampa de cor tem de ler `0..total` sem interrupção.
            ("Index", Column::Scalar(_)) => {
                out.set(
                    "Index",
                    Column::Scalar((0..total).map(|i| i as f32).collect()),
                );
            }
            ("Count", Column::Scalar(_)) => {
                out.set("Count", Column::Scalar(vec![total as f32; total]));
            }
            ("P", Column::Vec2(_)) => {
                let mut nv = Vec::with_capacity(total);
                for (copy, s) in slices.iter().enumerate() {
                    let pl = place(copy);
                    match s.get("P") {
                        Some(Column::Vec2(v)) => nv.extend(v.iter().map(|p| pl.apply(*p))),
                        _ => nv.extend((0..s.count()).map(|_| pl.apply([0.0, 0.0]))),
                    }
                }
                out.set("P", Column::Vec2(nv));
            }
            ("size", Column::Vec2(_)) if scaling => {
                let mut nv = Vec::with_capacity(total);
                for (copy, s) in slices.iter().enumerate() {
                    let f = factor(copy);
                    match s.get("size") {
                        Some(Column::Vec2(v)) => {
                            nv.extend(v.iter().map(|z| [z[0] * f, z[1] * f]));
                        }
                        _ => nv.extend(
                            (0..s.count()).map(|_| [SIZE_IDENTITY[0] * f, SIZE_IDENTITY[1] * f]),
                        ),
                    }
                }
                out.set("size", Column::Vec2(nv));
            }
            ("rot", Column::Scalar(_)) if turning => {
                let mut nv = Vec::with_capacity(total);
                for (copy, s) in slices.iter().enumerate() {
                    let a = turn(copy);
                    match s.get("rot") {
                        Some(Column::Scalar(v)) => nv.extend(v.iter().map(|r| r + a)),
                        _ => nv.extend((0..s.count()).map(|_| a)),
                    }
                }
                out.set("rot", Column::Scalar(nv));
            }
            _ => out.set(name.clone(), concat_or_identity(slices, name, proto)),
        }
    }

    // As colunas CUNHADAS pelos knobs, quando fatia nenhuma as trazia — a mesma lei da
    // `clone_stream`, e pelo mesmo motivo (senão o taper é um botão morto numa grelha).
    if scaling && !names.iter().any(|n| n == "size") {
        let mut nv = Vec::with_capacity(total);
        for (copy, s) in slices.iter().enumerate() {
            let f = factor(copy);
            nv.extend((0..s.count()).map(|_| [SIZE_IDENTITY[0] * f, SIZE_IDENTITY[1] * f]));
        }
        out.set("size", Column::Vec2(nv));
    }
    if turning && !names.iter().any(|n| n == "rot") {
        let mut nv = Vec::with_capacity(total);
        for (copy, s) in slices.iter().enumerate() {
            let a = turn(copy);
            nv.extend((0..s.count()).map(|_| a));
        }
        out.set("rot", Column::Scalar(nv));
    }
    out
}

/// Junta a coluna `name` das fatias, pondo a IDENTIDADE do tipo onde ela falta.
///
/// ⚠️ **O buraco tem de ser preenchido, não saltado:** as fatias são concatenadas por
/// POSIÇÃO, então uma coluna mais curta desalinharia todas as outras a partir dali — e em
/// silêncio, porque um `Stream` não valida comprimentos entre colunas.
fn concat_or_identity(slices: &[&Stream], name: &str, proto: &Column) -> Column {
    macro_rules! join {
        ($variant:ident, $id:expr) => {{
            let mut nv = Vec::new();
            for s in slices {
                match s.get(name) {
                    Some(Column::$variant(v)) => nv.extend_from_slice(v),
                    _ => nv.extend((0..s.count()).map(|_| $id)),
                }
            }
            Column::$variant(nv)
        }};
    }
    match proto {
        Column::Scalar(_) => join!(Scalar, 0.0),
        Column::Vec2(_) => join!(Vec2, [0.0; 2]),
        Column::Vec3(_) => join!(Vec3, [0.0; 3]),
        Column::Vec4(_) => join!(Vec4, [0.0; 4]),
    }
}
