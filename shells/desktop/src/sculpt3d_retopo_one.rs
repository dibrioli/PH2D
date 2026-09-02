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
    density: f32,
) -> Result<Candidata, RemeshRefusal> {
    // ⭐⭐⭐ **O PASSO POR VÉRTICE É CALCULADO AQUI EM CIMA desde 2026-09-01**, e não já
    // abaixo, porque ele deixou de ser só o `Step` do mapa: ele entra **no campo**.
    let sizing = sizing_field(cx.work, cx.target, adaptive);
    let owned;
    let base: &ph2d_crossfield::Dual =
        if features || super::super::retopo_extract::features_requested() {
            owned = with_features(cx);
            &owned
        } else {
            cx.dual
        };
    // ⭐⭐⭐ **A DENSIDADE ENTRA NO CAMPO** — ver
    // [`ph2d_crossfield::Dual::scale_by_density`], e o report do dono de 2026-09-01 (a tampa
    // chata no bico) é a razão.
    //
    // ⛔⛔ **Medido: pedir ao MAPA um passo mais fino na ponta não o entrega** — `14 %` de
    // pedido move a saída `3 %`, e a fase zero, que já entrega a ponta fina, vê o mapa
    // desfazê-la. ⚠️ *Não é implementação: num mapa de grade inteira `∇σ = J∇θ`, logo a
    // densidade realizável é ditada pelo CAMPO e o mínimo quadrado projecta fora o resto.*
    //
    // ⚠️ **`s = log(alvo / h)`** — positivo onde se quer fino. Constantes não entram (só as
    // diferenças ao longo de cada aresta), então não há normalização a escolher.
    //
    // ⛔ **Nasce DESLIGADA** (`PH2D_FIELD_DENSITY=<força>`): ela move as singularidades, que é
    // a grandeza de que todas as réguas desta linha dependem — ligar sem a tabela seria
    // exactamente o que o §0.0 proíbe.
    let owned_den;
    let dual: &ph2d_crossfield::Dual = match field_density_strength().unwrap_or(density) {
        k if k != 0.0 && !sizing.is_empty() => {
            let mut d = base.clone();
            let logs: Vec<f32> = sizing
                .iter()
                .map(|h| {
                    if *h > 0.0 && h.is_finite() {
                        (cx.target / h).ln()
                    } else {
                        0.0
                    }
                })
                .collect();
            d.scale_by_density(cx.work, &logs, k);
            owned_den = d;
            &owned_den
        }
        _ => base,
    };
    // ⭐ **`PH2D_TIP_ALIGN=<factor>` reforça o alinhamento ao relevo SÓ na CALOTA de cada
    // espinho afiado** — experimento de 2026-09-02 (plano, Parte XII §101). Medido na escada
    // de singularidades: a malha que o dono aprovou fecha CADA bico com um pólo `+1` (quatro
    // `+¼` a `≤ 2 h`); o nosso campo fecha o `3138` com `+¾` e a saída fica com `+½` perto e
    // a terceira a `6 h` — e a grade do bico é monótona nisso (`1,2 h → 0,51` · `2,4 h → 0,88`
    // · `6,1 h → 1,36`). No flanco de um cone a anisotropia já é `1`; o que falta é PESO, e o
    // [`ph2d_crossfield::ALIGN_WEIGHT`] global foi medido a partir em todo o lado (`1,0` dá
    // `48` singularidades numa peça com cristas). ⛔ Sem a env o dual é o de sempre, ao bit.
    let owned_tip;
    let dual: &ph2d_crossfield::Dual = match tip_align_factor() {
        Some(factor) => {
            let (_, apex) = ph2d_quadfill::apices(cx.reference, cx.target);
            let rpos = cx.reference.positions();
            let wpos = cx.work.positions();
            let nbr = ph2d_quadfill::adjacency(cx.work);
            let radius = TIP_ALIGN_RADIUS * cx.target;
            let mut faces: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
            for &a in &apex {
                let p = rpos[a];
                let Some(seed) = (0..wpos.len()).min_by(|&i, &j| {
                    let di = ph2d_quadfill_dist(wpos[i], p);
                    let dj = ph2d_quadfill_dist(wpos[j], p);
                    di.total_cmp(&dj)
                }) else {
                    continue;
                };
                let ball = ph2d_quadfill::path_ball(wpos, &nbr, seed, radius);
                for (fi, f) in cx.work.faces().iter().enumerate() {
                    if f.verts().iter().any(|v| ball.contains_key(&(*v as usize))) {
                        faces.insert(fi);
                    }
                }
            }
            eprintln!(
                "[sculpt3d] calota: {} espinho(s), {} faces com alinhamento x{factor:.1} a <= {TIP_ALIGN_RADIUS} h",
                apex.len(),
                faces.len()
            );
            let mut d = dual.clone();
            d.boost_align(faces.iter().copied(), factor);
            owned_tip = d;
            &owned_tip
        }
        None => dual,
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
    let index = ph2d_crossfield::vertex_index(cx.work, dual, &field);
    let singular: Vec<u32> = index
        .iter()
        .enumerate()
        .filter(|(_, k)| **k != 0)
        .filter_map(|(v, _)| u32::try_from(v).ok())
        .collect();
    // ⭐ **`PH2D_SING_DUMP=<dir>` escreve as singularidades do CAMPO desta candidata**
    // (`x y z índice`, uma por linha, um ficheiro por candidata) — diagnóstico de
    // 2026-09-02. ⛔ A escada de vértices irregulares da SAÍDA mostrou que a grade termina
    // onde as singularidades param (na malha aprovada, quatro a `≤ 2 h` do bico; nas nossas
    // reprovadas, a `9`–`15 h`) — e só o campo diz se elas já nascem longe ou se a jusante
    // as move. *Um dump por candidata, porque as candidatas correm em paralelo.*
    if let Ok(dir) = std::env::var("PH2D_SING_DUMP") {
        let pos = cx.work.positions();
        let mut text = String::new();
        for (v, k) in index.iter().enumerate() {
            if *k != 0 {
                let p = pos[v];
                text.push_str(&format!("{} {} {} {k}\n", p[0], p[1], p[2]));
            }
        }
        let name = format!(
            "sing_w{w:.3}_f{}_a{adaptive:.2}_d{density:.1}_t{}.txt",
            u8::from(features),
            if travel.is_finite() { "cerca" } else { "livre" }
        );
        let _ = std::fs::write(std::path::Path::new(&dir).join(name), text);
    }

    // ── G3 + G5. O mapa, e o arredondamento uma-a-uma que o torna inteiro.
    // ⭐ O G3 soldado é o default DENTRO deste caminho (que já shipa desligado);
    // `PH2D_GRIDMAP_WELD=0` volta ao penalizado, para bissecar.
    let welded = ph2d_gridmap::welded_enabled();
    let opts = ph2d_gridmap::RoundOptions::default();
    // ⭐⭐⭐ **A DENSIDADE SEGUE A FORMA** — ver [`sizing_field`]. Com
    // `adaptive == 0` o campo é constante e o passo é o escalar de sempre.
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
    // ⭐⭐⭐ **A UNIDADE das réguas da ponta é o ALVO, e é a MESMA em todas as candidatas do
    // clique** (2026-09-02). ⛔ A lei do ápice ([`ph2d_quadfill::apices`]) decide *o que é
    // um espinho* à escala da unidade; com a aresta mediana de cada candidata, duas
    // candidatas de `19 154` e `21 650` quads teriam censos DIFERENTES e o `worse` compararia
    // *«3 pontas más de 8»* com *«2 de 7»*. A bancada e as sondas usam a mediana
    // ([`ph2d_quadfill::median_edge`]) porque uma saída de outra ferramenta não tem alvo; as
    // duas diferem `~8 %` e o registo diz qual é.
    let unit = cx.target;
    let dev = ph2d_quadfill::tip_deviation(cx.reference, &out, unit);
    // ⭐⭐⭐ **A GRADE NA PONTA** — o report do dono de 2026-09-01, com foto e seta:
    // *«essa área deveria ser levada à ponta, mas fica a meio caminho»*. ⛔ A `ENTREGA`
    // acima mede coroas RADIAIS e faz média das pontas todas — ver
    // [`ph2d_quadfill::tip_density`].
    let den = ph2d_quadfill::tip_density(cx.reference, &out, unit);
    super::rulers::log_candidate(
        w, features, adaptive, &out, &shape, &round, &cut_rep, dev, den,
    );
    // ⭐ **`PH2D_CANDIDATE_DUMP=<dir>` grava a malha de CADA candidata** — o par do
    // `PH2D_SING_DUMP`. ⛔ Sem isto só a vencedora é observável, e a candidata que interessa
    // (2026-09-02: grade `0,86` em todas as pontas, `0` amputadas, `5` arestas de bordo) perde
    // na chave da frente e evapora — *um knob descartado e um knob fraco liam-se igual*.
    if let Ok(dir) = std::env::var("PH2D_CANDIDATE_DUMP") {
        let name = format!(
            "cand_w{w:.3}_f{}_a{adaptive:.2}_d{density:.1}_t{}.obj",
            u8::from(features),
            if travel.is_finite() { "cerca" } else { "livre" }
        );
        let text = ph2d_mesh::write_obj(&[ph2d_mesh::ExportPiece {
            name: Some("cand"),
            mesh: &out,
            pose: ph2d_mesh::Pose::default(),
        }]);
        let _ = std::fs::write(std::path::Path::new(&dir).join(name), text);
    }
    Ok((out, e, round.shift_frac_max, shape, dev, den))
}

/// ⭐⭐⭐ **AS DUAS EM PARALELO, ou em série se alguém estiver a bissecar.**
///
/// ⛔⛔ **Ela existe porque o par estava escrito TRÊS vezes** — a ronda de abertura, as
/// candidatas com densidade e a recaída do campo adaptativo —, cada uma com o seu `if` sobre
/// a mesma variável de ambiente. ⚠️ *Um gate desta linha já vigia que a recaída corra as DUAS
/// candidatas nos DOIS ramos, precisamente porque os ramos podiam divergir* — com uma porta
/// só, não podem.
///
/// ⚠️ `PH2D_RETOPO_SERIAL=1` é o A/B do paralelismo, e é o que permite dizer quanto ele vale
/// nesta máquina em vez de o supor.
pub(super) fn par<T: Send>(a: impl FnOnce() -> T + Send, b: impl FnOnce() -> T + Send) -> (T, T) {
    if std::env::var("PH2D_RETOPO_SERIAL").as_deref() == Ok("1") {
        (a(), b())
    } else {
        rayon::join(a, b)
    }
}

/// ⭐⭐⭐ **A FORÇA DA CORRECÇÃO CONFORME, e ela NÃO tem curso** — ver
/// [`ph2d_crossfield::Dual::scale_by_density`].
///
/// ⚠️ **`1` é o valor da TEORIA, não uma afinação:** a correcção é `α = −∗ds` exactamente, e
/// qualquer outro factor deixa de ser a métrica pedida. ⛔ Medido na escultura do dono, os
/// valores maiores sobre-conduzem — a `1,5` o mapa passa de `17` para **`105`** dobras e a
/// `2` o enviesamento mediano vai de `4,2°` para `7,7°`. *Escolher uma força seria inventar
/// um número onde a teoria já deu um.*
///
/// # ⭐ Medido na escultura do dono, pelo botão inteiro (`sem` · `com`)
///
/// | `Detail` | pontas cortadas | desvio `p50` | dobras do mapa |
/// |---|---|---|---|
/// | `1,00` | `0 de 4` · `0 de 4` | `0,47` · ⭐ **`0,22`** | `17` · ⭐ **`6`** |
/// | `0,95` | `0 de 4` · `0 de 4` | `0,88` · ⭐ **`0,27`** | `18` · ⭐ **`8`** |
/// | ⛔ `0,75` | `1 de 4` · `1 de 4` | `0,67` · ⛔ **`3,00`** | `20` · ⛔ **`166`** |
///
/// ⇒ é por isso que ela é **candidata** e não interruptor.
pub(super) const FIELD_DENSITY: f32 = 1.0;

/// ⭐ **O OVERRIDE da força da correcção** — `None` deixa o CHAMADOR mandar. ⚠️ A env força
/// o valor em TODAS as candidatas, o que é o que se quer de uma bissecção e ⛔ nunca do
/// produto — ele passa a força por argumento, uma candidata de cada vez.
///
/// ⛔ A env é lida **aqui, no chamador**, e não dentro da `ph2d-crossfield`: lá dentro ela
/// alcançaria a bancada, os gates e o produto de uma vez.
/// O raio da CALOTA em que o alinhamento é reforçado, em quads do alvo — a vizinhança em que
/// a escada de singularidades da malha aprovada vive (`≤ 2 h` para as quatro `+¼`, e a
/// compensação `−¼` a `8`–`15 h`). Experimento; ver [`tip_align_factor`].
const TIP_ALIGN_RADIUS: f32 = 8.0;

/// `PH2D_TIP_ALIGN=<factor>` — o reforço do alinhamento na calota dos espinhos afiados.
/// `None` sem a env, ou com um factor que não seja `> 1`.
fn tip_align_factor() -> Option<f32> {
    std::env::var("PH2D_TIP_ALIGN")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .filter(|k| k.is_finite() && *k > 1.0)
}

fn ph2d_quadfill_dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
}

fn field_density_strength() -> Option<f32> {
    std::env::var("PH2D_FIELD_DENSITY")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .filter(|k| k.is_finite() && *k != 0.0)
}
