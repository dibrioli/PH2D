//! ⭐⭐⭐ **UMA TENTATIVA DO BOTÃO, DE PONTA A PONTA** — irmão de
//! [`super::retopo_extract`] por RESPONSABILIDADE, e o terceiro corte deste caminho.
//!
//! ⚠️ **O corte é o que a cascata já dizia:** lá decide-se **qual** candidata vence
//! ([`super::rulers::melhor`]); aqui corre-se **uma** — campo, traçado, corte, penteado,
//! mapa, arredondamento, extracção, acabamento, e as réguas que a descrevem. *O ficheiro que
//! escolhe não tem de saber como se produz aquilo entre que escolhe.*
//!
//! ⛔ **Ele nasceu do tecto de LOC do shell** (HR-18, 600 — o irmão chegou a `694` ao ganhar
//! a quinta tentativa), e a fronteira **não** foi escolhida pelo tamanho: é a mesma que já
//! separava o [`super::target`] (o alvo) e o [`super::rulers`] (as réguas).

use super::RemeshRefusal;
use super::decide::Candidata;
use super::target::sizing_field;

/// **O QUE TODA TENTATIVA LÊ E NUNCA ESCREVE** — o que o botão fixou antes da 1.ª candidata.
///
/// ⚠️ **Tudo emprestado, e é isso que torna as tentativas paralelizáveis:** elas partilham
/// só leituras e não escrevem nada em comum — é o que faz o `rayon::join` do chamador ser
/// honesto, e não uma aposta.
pub(super) struct Ctx<'a> {
    /// A malha de TRABALHO — a saída da fase zero.
    pub work: &'a ph2d_mesh::Mesh,
    /// A ESCULTURA que o artista trouxe — a superfície em que o acabamento pousa. ⛔ Nunca a
    /// `work`: reprojectar sobre a remalhada somaria os dois erros.
    pub reference: &'a ph2d_mesh::Mesh,
    /// O dual do campo cruzado, já com o bordo restringido.
    pub dual: &'a ph2d_crossfield::Dual,
    /// O passo alvo da grade.
    pub target: f32,
}

/// **O CAMPO COM AS LINHAS DE FEIÇÃO** — o que a 3.ª tentativa do botão pede.
fn with_features(cx: &Ctx<'_>) -> ph2d_crossfield::Dual {
    let d = cx.dual;
    let mut d = d.clone();
    // ⚠️ **O `h` é o `target`**, e não uma medida da malha: a lei da feição mede-se
    // em múltiplos do **passo alvo da grade**, que é exactamente o número que o G3
    // recebe três blocos abaixo. *Medi-lo outra vez daria duas respostas à mesma
    // pergunta, e a que envelhece é a que ninguém vê.*
    let (fd, _) = ph2d_mesh::feature_dirs(cx.work, cx.target, ph2d_mesh::FeatureOptions::default());
    let (fe, _) = ph2d_mesh::feature_edges(cx.work, &fd, ph2d_mesh::FEATURE_EDGE_MIN_COS);
    d.constrain(cx.work, &fe);
    d
}

/// ⭐⭐⭐ **CORRE UMA CANDIDATA, de ponta a ponta.**
///
/// `w` é o peso do alinhamento ao relevo, `features` liga as linhas de feição, `adaptive`
/// gradua a densidade da fase zero e `travel` é a cerca de viagem do acabamento (ver
/// [`ph2d_quadfill::EXTRACT_TRAVEL`]).
pub(super) fn one(
    cx: &Ctx<'_>,
    w: f32,
    features: bool,
    adaptive: f32,
    travel: f32,
) -> Result<Candidata, RemeshRefusal> {
    let owned;
    let dual: &ph2d_crossfield::Dual =
        if features || super::super::retopo_extract::features_requested() {
            owned = with_features(cx);
            &owned
        } else {
            cx.dual
        };
    let (field, _) = if (w - ph2d_crossfield::ALIGN_WEIGHT).abs() < f32::EPSILON {
        ph2d_crossfield::solve_miq(dual)
    } else {
        ph2d_crossfield::solve_miq_aligned(dual, ph2d_crossfield::Rounding::default(), w)
    };
    let layout = ph2d_trace::trace_patches(cx.work, dual, &field);
    let (cut, cut_rep) = ph2d_gridmap::cut_along_patches(cx.work, &layout);
    let (combed, _) = ph2d_gridmap::comb_patches(cx.work, &layout, &cut);

    // ⭐ As singularidades saem do CAMPO — o índice por-vértice é um facto dele, e
    // pedir à `ph2d-gridmap` que o re-derive seria reconstruir o que já existe.
    let singular: Vec<u32> = ph2d_crossfield::vertex_index(cx.work, dual, &field)
        .into_iter()
        .enumerate()
        .filter(|(_, k)| *k != 0)
        .filter_map(|(v, _)| u32::try_from(v).ok())
        .collect();

    // ── G3 + G5. O mapa, e o arredondamento uma-a-uma que o torna inteiro.
    // ⭐ O G3 soldado é o default DENTRO deste caminho (que já shipa desligado);
    // `PH2D_GRIDMAP_WELD=0` volta ao penalizado, para bissecar.
    let welded = ph2d_gridmap::welded_enabled();
    let opts = ph2d_gridmap::RoundOptions::default();
    // ⭐⭐⭐ **A DENSIDADE SEGUE A FORMA** — ver [`sizing_field`]. Com
    // `adaptive == 0` o campo é constante e o passo é o escalar de sempre.
    let sizing = sizing_field(cx.work, cx.target, adaptive);
    let step = ph2d_gridmap::Step {
        h: cx.target,
        per_vertex: &sizing,
    };
    let (map, round) = if welded {
        ph2d_gridmap::round_welded(cx.work, &cut, &combed, step, opts, &singular)
    } else {
        ph2d_gridmap::round_to_integers(cx.work, &cut, &combed, step, opts, &singular)
    };

    // ── A extracção das isolinhas.
    let (tris, uv) = ph2d_gridmap::corner_map(&cut, &map);
    let cm = ph2d_quadextract::CornerMap {
        pos: cx.work.positions(),
        tris: &tris,
        uv: &uv,
    };
    let (mut out, e) = ph2d_quadextract::extract(&cm, None).map_err(RemeshRefusal::Extract)?;
    if out.faces().is_empty() {
        return Err(RemeshRefusal::TooCoarseToResolve);
    }

    // ⭐⭐⭐ **O ACABAMENTO — e este caminho não o tinha.**
    //
    // ⛔⛔ O irmão dele, o `ph2d_quadfill::fill`, corre [`ph2d_quadfill::SMOOTHING_ROUNDS`]
    // passos de Laplaciano tangencial com reprojeção **desde sempre**; a extracção
    // entregava a malha **crua**. *Dois caminhos para o mesmo botão, e só um com
    // acabamento.*
    //
    // ⚠️ **A superfície é a `reference` — a escultura — e nunca a `work`.** É a mesma lei
    // que o doc do `fill` escreve com o defeito de 2026-08-21 ao lado: reprojectar sobre a
    // remalhada somaria os dois erros.
    //
    // Medido 2026-08-26 na `sculpt_t003` do artista, na densidade fina:
    //
    // | régua | cru | **com acabamento** |
    // |---|---|---|
    // | distância à ESCULTURA p95 | `0,106 %` | ⭐ **`0,000 %`** |
    // | enviesamento p99 · `>60°` | `39,3°` · `18` | ⭐ **`29,1°` · `1`** |
    // | aspecto p99 · `>4×` | `2,05` · `7` | ⭐ **`1,63` · `0`** |
    //
    // ⚠️ **Ele NÃO alisa a superfície, e isso é o achado:** a rugosidade fica onde estava
    // (`14,2° ⇒ 14,3°`) porque a reprojecção repõe os vértices na peça. *A aspereza que o
    // artista vê é a da escultura dele — a grade fina RESOLVE-A, a cadeia não a inventa.*
    // ⭐ **O preço, medido:** `425 ms` sobre `7 750` quads numa cadeia de `7,0 s` —
    // **6 %**, na densidade mais fina medida (melhor de 3, `6 979` contra `7 404 ms`).
    // ⚠️ `PH2D_EXTRACT_FINISH=0` desliga, para bissecar.
    //
    // ⭐⭐⭐ **E DESDE 2026-08-28 O ACABAMENTO É UMA PORTA, não uma linha aqui** — a
    // mesma que a `ph2d-quadchain` chama, porque *duas ordens para o mesmo botão com
    // acabamentos diferentes é uma lei que gate nenhum defende*. Ela corre o
    // Laplaciano como **ronda zero** e depois o ajuste de quadrado **alinhado ao
    // relevo**, e entrega a MELHOR ronda — ver `ph2d_quadfill::finish_extract`.
    //
    // ⚠️ **O ganho, medido na densidade que este botão usa** (`sculpt_eared`, 524
    // quads): enviesamento mediano `10,4° → 3,8°`, aspecto `1,14 → 1,07`, faces
    // péssimas `0 → 0`, e o preço `21 ms → ~400 ms` numa cadeia de segundos.
    // ⭐⭐⭐ **A CERCA DE VIAGEM é escolhida AQUI, no chamador** — ver
    // [`ph2d_quadfill::EXTRACT_TRAVEL`]. ⛔ Lida dentro da biblioteca, ela alcançava a
    // bancada, os gates e o produto de uma vez; aqui é uma escolha deste botão, e o
    // ramo de omissão fica com a chamada que o gate do fonte pina, letra por letra.
    let travel = std::env::var("PH2D_EXTRACT_TRAVEL")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(travel);
    if std::env::var("PH2D_EXTRACT_FINISH").as_deref() != Ok("0") {
        if travel.is_finite() {
            ph2d_quadfill::finish_extracted_travel(&mut out, cx.reference, travel);
        } else {
            ph2d_quadfill::finish_extracted(&mut out, cx.reference);
        }
    }
    let out = out;

    let shape = ph2d_quadfill::quad_shape(&out);
    // ⭐⭐⭐ **CADA CANDIDATA DIZ O QUE É** — ver [`super::rulers::log_candidate`], que
    // mora ao lado do [`worse`] de propósito: *o registo que explica uma escolha
    // tem de ler as mesmas grandezas que a fazem.*
    let dev = ph2d_quadfill::tip_deviation(cx.reference, &out, cx.target);
    // ⭐⭐⭐ **A GRADE NA PONTA** — o report do dono de 2026-09-01, com foto e seta:
    // *«essa área deveria ser levada à ponta, mas fica a meio caminho»*. ⛔ A `ENTREGA`
    // acima mede coroas RADIAIS e faz média das pontas todas — ver
    // [`ph2d_quadfill::tip_density`].
    let den = ph2d_quadfill::tip_density(cx.reference, &out, cx.target);
    super::rulers::log_candidate(
        w, features, adaptive, &out, &shape, &round, &cut_rep, dev, den,
    );
    Ok((out, e, round.shift_frac_max, shape, dev, den))
}
