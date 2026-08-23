//! **A COSTURA** — amostrar uma vez, montar, alisar, reprojetar.
//!
//! ⚠️ **A regra que governa este ficheiro inteiro:** *um ponto de fronteira
//! pertence ao ARCO, nunca ao patch.* Os dois patches que partilham um arco
//! pedem-lhe os mesmos índices — um deles ao contrário. Amostrar por patch daria
//! dois conjuntos de pontos quase iguais sobre a mesma curva, e a malha sairia
//! **rasgada** ao longo de toda fronteira de patch: um erro pequeno demais para
//! se ver num render e grande demais para uma malha ser usável.

use std::collections::BTreeMap;

use ph2d_mesh::{Face, Mesh};
use ph2d_quantize::Quantization;
use ph2d_trace::PatchLayout;

use crate::aligned::Interior;
use crate::fan::{resample_by, segment};
use crate::finish::{bbox_seed, measure, signed_volume, smooth_once};
use crate::param::PatchParam;
use crate::patch::{Chains, Chains2, Domain, build_grid, fill_rectangle, side_uv};
use crate::report::{FillError, FillReport, Points, Provenance};

/// ⚠️ **Quantas rondas de alisamento por omissão.** Elas não mudam a topologia —
/// só onde os pontos ficam — e cada uma **reprojeta** sobre a superfície de
/// referência, então a forma não escorre. O número está aqui e não numa opinião:
/// ver a tabela do `PLAN.md` §4-sexies.
pub const SMOOTHING_ROUNDS: usize = 6;

/// **MONTA A MALHA DE QUADS.**
///
/// # Os DOIS mesh, e por que eles são dois parâmetros
///
/// ⭐ **`indexed` é a malha que o `layout` INDEXA; `surface` é onde o resultado
/// pousa.** Elas são a mesma coisa em quase todo chamador — e na cadeia do produto
/// **não são**: o traçado corre sobre a saída da remalha isotrópica (F1), que tem
/// espaço de índice próprio, enquanto a forma que o artista esculpiu vive na malha
/// original.
///
/// ⛔ **Isto era UM parâmetro chamado `reference`, e a confusão custou o produto
/// inteiro** (auditoria de 2026-08-21). A porta do shell passou-lhe a malha
/// original — raciocinando, **corretamente**, sobre o papel de reprojeção — e com
/// isso cada `arc_chain[i]`, que é um índice da malha remalhada, foi ler a posição
/// de um vértice **arbitrário** da original. Medido: aresta mediana **4,6× o
/// alvo**, aresta máxima **2,01 numa peça de raio 1,0** — o diâmetro, uma aresta a
/// atravessar a esfera de lado a lado. *E os quatro números do relatório saíram
/// **bit-a-bit iguais** aos da corrida correta.*
///
/// ⚠️ **A cura não foi trocar o argumento: foi partir o parâmetro**, porque a
/// assinatura antiga **não permitia exprimir** a intenção certa. Um erro que a
/// assinatura torna inexprimível não precisa de gate.
///
/// # Errors
/// [`FillError`] quando a estrutura a montante não fecha — ver as variantes.
pub fn fill(
    indexed: &Mesh,
    surface: &Mesh,
    layout: &PatchLayout,
    quant: &Quantization,
    smoothing: usize,
) -> Result<(Mesh, FillReport), FillError> {
    fill_with(
        indexed,
        surface,
        layout,
        quant,
        smoothing,
        crate::relax::SQUARE_ROUNDS,
        crate::aligned::INTERIOR,
    )
}

/// **A MONTAGEM com as duas relaxações à vista** — ver [`fill`], de que esta é a
/// forma aberta.
///
/// ⭐⭐ **Ela existe porque as duas rondas medem coisas diferentes e a tabela que
/// escolhe os números precisa de as variar em separado:** `smoothing` é o
/// Laplaciano (comprimentos de aresta) e `square` é o ajuste de quadrado
/// (ângulos). ⚠️ *Um parâmetro só, a somar as duas, tornaria a tabela
/// inexprimível* — e o número que shipa sairia de um palpite em vez de uma
/// medição.
///
/// # Errors
/// [`FillError`] quando a estrutura a montante não fecha — ver as variantes.
pub fn fill_with(
    indexed: &Mesh,
    surface: &Mesh,
    layout: &PatchLayout,
    quant: &Quantization,
    smoothing: usize,
    square: usize,
    interior: crate::aligned::Interior,
) -> Result<(Mesh, FillReport), FillError> {
    // ⭐⭐ **A PRÉ-CONDIÇÃO, e ela é a mais barata que existe.** Ver
    // [`check_arcs_belong_to`].
    crate::report::check_arcs_belong_to(indexed, layout)?;
    // ⭐⭐⭐ **A RE-GRADUAÇÃO DO ARCO, se ligada** — ver [`crate::regraduate`]. Ela troca
    // **uma** régua e com isso os TRÊS sítios que a leem (a reamostragem abaixo, o pino
    // da fronteira no achatamento e o `uv` dos pontos de saída) mudam por construção.
    // ⚠️ *É por isso que ela é um `arc_tau` novo e não um parâmetro a mais.*
    let regraduated = crate::regraduate::REGRADUATE
        .then(|| crate::regraduate::conformal_arc_tau(indexed, layout, quant))
        .flatten();
    let regraduated_arcs = regraduated.as_ref().map_or(0, |r| r.changed);
    let arc_tau: &[Vec<f32>] = regraduated
        .as_ref()
        .map_or(&layout.arc_tau, |r| r.tau.as_slice());
    let src = indexed.positions();
    // ⭐⭐ **TODO ponto de INTERIOR nasce POUSADO na superfície.** Ver
    // [`Points::push_facing`].
    let seed = bbox_seed(surface);
    let mut pts = Points::new();
    let mut faces: Vec<Face> = Vec::new();

    // ── 1. Os pontos de cada ARCO, amostrados UMA vez.
    let mut corner_vid: BTreeMap<u32, u32> = BTreeMap::new();
    let mut arc_points: Vec<Vec<u32>> = Vec::with_capacity(layout.arc_chain.len());
    for (a, chain) in layout.arc_chain.iter().enumerate() {
        let n = quant.arc[a].max(1) as usize;
        let curve: Vec<[f32; 3]> = chain.iter().map(|&v| src[v as usize]).collect();
        // ⭐⭐ **Pelo `τ` do layout, e não pelo comprimento.** É o MESMO número que
        // decidiu quantos segmentos este arco leva — ver
        // [`ph2d_trace::PatchLayout::arc_tau`]. Sem graduação os dois coincidem;
        // com ela, os pontos adensam onde o campo de tamanho pede.
        let sampled = resample_by(&curve, &arc_tau[a], n);
        let mut ids = Vec::with_capacity(n + 1);
        for (k, p) in sampled.iter().enumerate() {
            // As pontas são cantos de malha, e um canto é de TODOS os arcos que
            // lá chegam — por isso a chave é o vértice, não o par (arco, índice).
            let id = if k == 0 || k == n {
                let v = if k == 0 {
                    chain[0]
                } else {
                    *chain.last().unwrap_or(&chain[0])
                };
                *corner_vid
                    .entry(v)
                    .or_insert_with(|| pts.push(src[v as usize], Provenance::Corner))
            } else {
                pts.push(*p, Provenance::Arc)
            };
            ids.push(id);
        }
        arc_points.push(ids);
    }

    // ⭐ **AS FACES DE CADA PATCH**, uma passagem só. É o que o achatamento
    // consome, e reconstruí-lo por patch seria `O(patches × faces)`.
    let mut patch_faces: Vec<Vec<u32>> = vec![Vec::new(); layout.side_arcs.len()];
    for (f, &pp) in layout.face_patch.iter().enumerate() {
        if let Some(slot) = patch_faces.get_mut(pp as usize) {
            slot.push(u32::try_from(f).unwrap_or(0));
        }
    }
    let mut flattened = 0usize;
    let (mut sampled, mut misses) = (0usize, 0usize);
    let (mut flatten_rounds, mut flatten_residual) = (0usize, 0.0f32);
    let (mut rough, mut fell_back, mut from_fan) = (0.0f32, 0usize, Vec::<bool>::new());
    // ⭐⭐⭐ **Quantos patches trazem singularidade DENTRO** — a régua que decide se a
    // dívida é desta fase ou do traçado. ⛔ Contam-se *patches*, não voltas: dois
    // defeitos no mesmo patch são um patch impossível, não dois.
    let (mut dirty_patches, mut combed_patches) = (0usize, 0usize);
    // ⭐⭐⭐ **O denominador vem junto** — ver [`crate::FillReport::slid`]. `quads` é
    // quantos patches de quatro lados existem; sem ele, `slid = 0` não distingue
    // *nenhum deslizou* de *não havia nenhum*.
    let (mut slid, mut quads) = (0usize, 0usize);
    let mut slid_refused = [0usize; 5];
    let mut conformal: Vec<f32> = Vec::new();
    let (mut dom_rect, mut dom_fan): (Vec<f32>, Vec<f32>) = (Vec::new(), Vec::new());

    // ── 2. Cada patch vira o seu leque.
    for (p, sides) in layout.side_arcs.iter().enumerate() {
        let n = sides.len();
        let e = &quant.corners[p];
        if n < 3 || e.len() != n {
            return Err(FillError::Mismatch {
                patch: p,
                side: 0,
                expected: u32::try_from(n).unwrap_or(0),
                got: u32::try_from(e.len()).unwrap_or(0),
            });
        }
        // Os pontos de cada lado, do canto de entrada ao de saída.
        let mut side_pts: Vec<Vec<u32>> = Vec::with_capacity(n);
        for (i, side) in sides.iter().enumerate() {
            let mut pts: Vec<u32> = Vec::new();
            for &(a, rev) in side {
                let mut ids = arc_points[a as usize].clone();
                if rev {
                    ids.reverse();
                }
                if pts.is_empty() {
                    pts = ids;
                } else {
                    if *pts.last().unwrap_or(&u32::MAX) != ids[0] {
                        return Err(FillError::Broken {
                            patch: p,
                            side: i,
                            ends_at: pts.last().copied(),
                            next_starts_at: Some(ids[0]),
                            sides: n,
                        });
                    }
                    pts.extend_from_slice(&ids[1..]);
                }
            }
            side_pts.push(pts);
        }
        // ⚠️ A lei, conferida: se ela não bate aqui, o resto é geometria torcida.
        for i in 0..n {
            let want = e[(i + n - 1) % n] + e[(i + 1) % n];
            let got = u32::try_from(side_pts[i].len() - 1).unwrap_or(u32::MAX);
            if want != got {
                return Err(FillError::Mismatch {
                    patch: p,
                    side: i,
                    expected: want,
                    got,
                });
            }
            if side_pts[i].last() != side_pts[(i + 1) % n].first() {
                return Err(FillError::Broken {
                    patch: p,
                    side: i,
                    ends_at: side_pts[i].last().copied(),
                    next_starts_at: side_pts[(i + 1) % n].first().copied(),
                    sides: n,
                });
            }
        }

        // ⭐⭐ **O ACHATAMENTO DO PATCH** — ver [`crate::param`]. A partir daqui a
        // grade é construída no DOMÍNIO e volta pela triangulação; a interpolação
        // em `ℝ³` fica como caminho de recurso para o patch que não achatar.
        // ⚠️ **A fronteira leva o `τ` junto, e não é conforto.** O achatamento põe
        // cada vértice de malha da fronteira na aresta do polígono pela sua fração
        // de `τ`, e o [`side_uv`] põe cada ponto de SAÍDA pela dele — se as duas
        // usassem réguas diferentes (uma o comprimento, outra o `τ`), um ponto de
        // fronteira teria um `uv` que discorda dos vértices de malha à volta dele, e
        // a grade nasceria torcida junto ao bordo. *Uma régua, duas leituras.*
        //
        // ⚠️ **Um arco percorrido ao contrário tem o `τ` ESPELHADO**
        // (`τ' = τ_fim − τ`), não invertido na ordem: `τ` é uma medida acumulada, e
        // virar a lista sem virar os valores daria uma cadeia decrescente.
        let mut mesh_sides: Vec<Vec<u32>> = Vec::with_capacity(n);
        let mut mesh_tau: Vec<Vec<f32>> = Vec::with_capacity(n);
        for side in sides {
            let (mut chain, mut tau): (Vec<u32>, Vec<f32>) = (Vec::new(), Vec::new());
            for &(a, rev) in side {
                let mut c = layout.arc_chain[a as usize].clone();
                let src_tau = &arc_tau[a as usize];
                let end = src_tau.last().copied().unwrap_or(0.0);
                let mut t: Vec<f32> = if rev {
                    src_tau.iter().rev().map(|v| end - v).collect()
                } else {
                    src_tau.clone()
                };
                if rev {
                    c.reverse();
                }
                // ⭐⭐⭐ **EM ESPAÇO DE SEGMENTO, se ligado** — ver
                // [`crate::param::SEGMENT_SPACE_DOMAIN`]. Dentro de um arco os dois
                // espaços são proporcionais (a reamostragem é por `τ` igual), então
                // basta escalar; o que muda é a soma **entre** arcos.
                if crate::param::SEGMENT_SPACE_DOMAIN && end > 0.0 {
                    #[allow(clippy::cast_precision_loss)]
                    let segs = quant.arc[a as usize].max(1) as f32;
                    for v in &mut t {
                        *v = *v / end * segs;
                    }
                }
                let base = tau.last().copied().unwrap_or(0.0);
                for v in &mut t {
                    *v += base;
                }
                if chain.is_empty() {
                    chain = c;
                    tau = t;
                } else {
                    chain.extend_from_slice(&c[1..]);
                    tau.extend_from_slice(&t[1..]);
                }
            }
            mesh_sides.push(chain);
            mesh_tau.push(tau);
        }
        // ⭐ **O CAMPO, se o layout o trouxe** — [`crate::aligned`]. Sem ele (gates que
        // chamam `decompose` direto) o achatamento volta ao harmónico de sempre.
        let dir = (interior == Interior::AlignedToField && !layout.face_dir.is_empty())
            .then_some(layout.face_dir.as_slice());
        // ⭐ **QUANTOS SEGMENTOS cada lado carrega** — [`crate::param::corners_for_sides`].
        let seg: Vec<u32> = (0..n)
            .map(|i| sides[i].iter().map(|&(a, _)| quant.arc[a as usize]).sum())
            .collect();
        let param = PatchParam::build(
            indexed,
            &patch_faces[p],
            &mesh_sides,
            &mesh_tau,
            &seg,
            dir,
            false,
        );
        if param.is_some() {
            flattened += 1;
        }
        if let Some(q) = param.as_ref() {
            flatten_rounds = flatten_rounds.max(q.rounds);
            flatten_residual = flatten_residual.max(q.residual);
            rough = rough.max(q.rough);
            dirty_patches += usize::from(q.defects > 0);
            combed_patches += usize::from(q.combed);
            fell_back += usize::from(q.fell_back);
            slid += usize::from(q.slid());
            conformal.push(q.conformal);
            if let Some(c) = q.slid_refused {
                slid_refused[c.min(4)] += 1;
            }
        }
        let dom = Domain {
            param: param.as_ref(),
            side_uv: (0..n)
                .map(|i| side_uv(arc_tau, quant, sides, i, &seg, param.as_ref()))
                .collect(),
            tally: std::cell::Cell::new((0, 0)),
            dom_skew: std::cell::RefCell::new(Vec::new()),
        };

        // ⭐⭐ **UM PATCH DE QUATRO LADOS É UM RETÂNGULO, e um retângulo não
        // precisa de leque.** Ver [`fill_rectangle`].
        //
        // ⛔⛔⛔ **O `else` É LOAD-BEARING, e o `continue` que ele substituiu custou
        // um dia inteiro de conclusões erradas** (2026-08-23). A escrituração do fim
        // deste laço — o `dom_skew` por valência e o `from_fan` de cada face — foi
        // escrita para o caminho do leque; o rectângulo saía por `continue` **antes
        // dela**. ⇒ `dom_rect` ficou SEMPRE vazio e a mediana de um vector vazio é
        // `0,0`, que se lê como *«a grade do rectângulo nasce perfeita»* — e foi essa
        // leitura que partiu a investigação em duas metades, uma delas inexistente.
        // ⚠️ E o `from_fan` ficava por estender, então as faces do rectângulo eram
        // **rotuladas como leque** pelo primeiro patch de leque a seguir.
        //
        // ⇒ *Um `continue` a meio de um laço com contabilidade no fim é uma armadilha
        // que se arma sozinha na próxima ramificação; um `else` não a pode armar.*
        if n == 4 {
            quads += 1;
            fill_rectangle(&mut pts, &mut faces, surface, seed, &side_pts, &dom);
        } else {
            // O centro: o do polígono no domínio, e a média de fronteira como recurso.
            //
            // ⚠️ **No domínio o centro é `(0,0)` e não uma média** — o polígono é
            // regular e convexo, então o seu centro está lá por construção. A média dos
            // pontos de fronteira em `ℝ³` é o que sobra quando o patch não achata, e é
            // exactamente ela que, num patch dobrado sobre um gancho, cai **dentro** da
            // peça.
            let mut c = [0.0f32; 3];
            let mut count = 0usize;
            for side in &side_pts {
                for &v in &side[..side.len() - 1] {
                    let q = pts.pos[v as usize];
                    for k in 0..3 {
                        c[k] += q[k];
                    }
                    count += 1;
                }
            }
            #[allow(clippy::cast_precision_loss)]
            let inv = if count == 0 { 0.0 } else { 1.0 / count as f32 };
            // ⭐ **O centro é o CENTRÓIDE DOS CORTES no domínio, não a origem.** Os `n`
            // cortes são as pontas dos raios; o ponto que os equilibra é o que faz as
            // `n` sub-grades saírem parecidas. A origem só coincide com ele quando os
            // cortes estão simétricos, que é o caso raro.
            let center_uv = {
                let mut c = [0.0f32; 2];
                for i in 0..n {
                    let cut = e[(i + n - 1) % n] as usize;
                    let q = dom.side_uv[i][cut];
                    c[0] += q[0];
                    c[1] += q[1];
                }
                #[allow(clippy::cast_precision_loss)]
                let inv = 1.0 / n as f32;
                [c[0] * inv, c[1] * inv]
            };
            let center_vid = dom.place(
                &mut pts,
                surface,
                seed,
                center_uv,
                [c[0] * inv, c[1] * inv, c[2] * inv],
                Provenance::Center,
            );

            // Os raios: do centro ao corte de cada lado, rectos NO DOMÍNIO.
            let mut spoke: Vec<Vec<u32>> = Vec::with_capacity(n);
            let mut spoke_uv: Vec<Vec<[f32; 2]>> = Vec::with_capacity(n);
            for i in 0..n {
                let cut = e[(i + n - 1) % n] as usize;
                let tip = side_pts[i][cut];
                let steps = e[i] as usize;
                let tip_uv = dom.side_uv[i][cut];
                let line = segment(pts.pos[center_vid as usize], pts.pos[tip as usize], steps);
                let line_uv = segment(center_uv, tip_uv, steps);
                let mut ids = Vec::with_capacity(steps + 1);
                ids.push(center_vid);
                for (k, q) in line.iter().enumerate().take(steps).skip(1) {
                    ids.push(dom.place(&mut pts, surface, seed, line_uv[k], *q, Provenance::Spoke));
                }
                ids.push(tip);
                spoke.push(ids);
                spoke_uv.push(line_uv);
            }

            // As `n` grades.
            for i in 0..n {
                let j = (i + 1) % n;
                let (s, t) = (e[j] as usize, e[i] as usize);
                let cut_i = e[(i + n - 1) % n] as usize;
                let cut_j = e[i] as usize;
                let bottom = &side_pts[i][cut_i..];
                let right = &side_pts[j][..=cut_j];
                let top = &spoke[j];
                let left: Vec<u32> = spoke[i].iter().rev().copied().collect();
                let bottom_uv = &dom.side_uv[i][cut_i..];
                let right_uv = &dom.side_uv[j][..=cut_j];
                let left_uv: Vec<[f32; 2]> = spoke_uv[i].iter().rev().copied().collect();
                let grid = build_grid(
                    &mut pts,
                    surface,
                    seed,
                    Chains {
                        bottom,
                        top,
                        left: &left,
                        right,
                    },
                    Chains2 {
                        bottom: bottom_uv,
                        top: &spoke_uv[j],
                        left: &left_uv,
                        right: right_uv,
                    },
                    (s, t),
                    &dom,
                );
                for k in 0..s {
                    for l in 0..t {
                        faces.push(Face::quad(
                            grid[k][l],
                            grid[k + 1][l],
                            grid[k + 1][l + 1],
                            grid[k][l + 1],
                        ));
                    }
                }
            }
        }
        let (o, m) = dom.tally.get();
        sampled += o;
        misses += m;
        let bucket = if n == 4 { &mut dom_rect } else { &mut dom_fan };
        bucket.append(&mut dom.dom_skew.borrow_mut());
        // ⭐ Valência de origem — [`crate::shape::skew_by_fan`]. ⚠️ A inversão de
        // orientação abaixo troca vértices e **não reordena** faces.
        from_fan.resize(faces.len(), n != 4);
    }

    // ── 3. A malha, com a orientação conferida.
    let mut flipped = 0usize;
    if signed_volume(&pts.pos, &faces) < 0.0 {
        // ⚠️ **Ou todas ou nenhuma.** A orientação da grade sai da orientação da
        // fronteira do patch, que é a mesma para todos os patches; se ela estiver
        // ao contrário, está ao contrário em bloco. Inverter face a face por
        // teste local é que produziria uma malha inconsistente.
        for f in &mut faces {
            let v = f.verts().to_vec();
            *f = Face::quad(v[3], v[2], v[1], v[0]);
        }
        flipped = faces.len();
    }
    let mut mesh =
        Mesh::from_parts(pts.pos, faces).map_err(|e| FillError::Mesh(format!("{e:?}")))?;

    // ── 4. Alisar, reprojetando sempre.
    for _ in 0..smoothing {
        smooth_once(&mut mesh, surface);
    }
    // ── 5. ⭐⭐ **Endireitar os ÂNGULOS** — ver [`crate::relax`]. Corre DEPOIS do
    // Laplaciano de propósito: aquele iguala comprimentos e é quem tira a malha do
    // estado em que nasce; este pega numa malha já de passo regular e ataca a
    // única coisa que o outro não vê. ⚠️ *A ordem inversa foi medida* — ver a
    // tabela em [`SQUARE_ROUNDS`].
    for _ in 0..square {
        crate::relax::square_once(&mut mesh, surface, seed);
    }

    let mut report = measure(&mesh, surface, &pts.prov, smoothing + square, flipped);
    report.flattened = flattened;
    report.patches = layout.side_arcs.len();
    report.sampled = sampled;
    report.sample_misses = misses;
    report.flatten_rounds = flatten_rounds;
    report.flatten_residual = flatten_residual;
    // ⚠️ **Toda face leva a valência do patch que a fez.** O `else` do laço acima é
    // quem o garante; isto é a cerca que morde se alguém lá voltar a pôr um `continue`.
    debug_assert_eq!(
        from_fan.len(),
        mesh.faces().len(),
        "a etiquetagem por valência tem de cobrir a malha inteira"
    );
    report.skew_by_fan = crate::shape::skew_by_fan(&mesh, &from_fan);
    let med = |v: &mut Vec<f32>| {
        v.sort_by(f32::total_cmp);
        v.get(v.len() / 2).copied().unwrap_or(0.0)
    };
    report.domain_cells = (dom_rect.len(), dom_fan.len());
    report.domain_skew = (med(&mut dom_rect), med(&mut dom_fan));
    report.rough = rough;
    report.dirty_patches = dirty_patches;
    report.combed_patches = combed_patches;
    report.fell_back = fell_back;
    report.slid = slid;
    report.quad_patches = quads;
    report.slid_refused = slid_refused;
    report.regraduated = regraduated_arcs;
    // ⭐⭐⭐ **O CONTROLO da troca de achatamento** — ver [`crate::lscm::conformal_error`].
    conformal.sort_by(f32::total_cmp);
    report.conformal = conformal.get(conformal.len() / 2).copied().unwrap_or(0.0);
    Ok((mesh, report))
}
