//! ⭐⭐⭐ **A SAÍDA SEGUE O CAMPO?** — a sonda que decide de que fase é a dívida do
//! enviesamento, e a última pergunta em aberto de 2026-08-22.
//!
//! # O que já se sabia, e porque não bastava
//!
//! | medido | quem | valor |
//! |---|---|---|
//! | enviesamento p50 da nossa saída | nós | `18–27°` |
//! | enviesamento p50 do oráculo | ele | ⭐ `5–6°` |
//! | a mesma coisa depois de 16 rondas de ajuste de quadrado | nós | ⛔ `26°` — **não move** |
//!
//! ⛔ **Que a relaxação não mexa no p50 é a prova de que o defeito não está nas
//! POSIÇÕES.** Uma relaxação move vértices e mais nada; se depois de dezasseis
//! rondas a mediana ficou onde estava, então endireitar um quad desendireita o
//! vizinho — *o esmagamento está na CONECTIVIDADE, em que direcção as linhas da
//! grade correm.* E uma direcção de linha de grade não é coisa que um alisador
//! possa mudar.
//!
//! # A pergunta que esta sonda responde
//!
//! Se as linhas da nossa grade **seguissem** o campo cruzado que o F2 calcula, elas
//! estariam certas por construção — as quatro direcções de uma cruz são
//! ortogonais, e uma grade que as siga é quadrada. Então:
//!
//! | se a medição der | a dívida é de |
//! |---|---|
//! | ⭐ **desvio grande** (a saída ignora o nosso próprio campo) | **F5** — a montagem, que nem sequer RECEBE o campo |
//! | desvio pequeno mas o ângulo mau | **F2** — o campo, que está torto |
//!
//! ⚠️ **E a assinatura de [`ph2d_quadfill::fill_with`] já sugere a resposta:** ela
//! recebe a malha, o layout e a quantização — **e não recebe o campo**. Uma fase
//! que não tem o campo entre os argumentos não o pode seguir. *Esta sonda existe
//! para pôr o número ao lado da observação, porque «não podia» e «não segue» são
//! afirmações diferentes e só uma delas é medição.*
//!
//! ⭐ **O CONTROLO é o oráculo contra o campo DELE**, lido dos mesmos ficheiros de
//! bancada. Sem ele, um desvio de `25°` não se sabe se é mau — talvez nenhuma
//! quadrangulação siga o campo tão de perto.

/// Onde a bancada mora. ⚠️ Fora da árvore (ADR-0162).
const BENCH: &str = "/home/enio/Documentos/Projetos/ph2d-quadbench/ref";

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn unit(v: [f32; 3]) -> [f32; 3] {
    let l = dot(v, v).sqrt().max(1.0e-12);
    [v[0] / l, v[1] / l, v[2] / l]
}

/// **O desvio 4-RoSy entre duas direções, no plano de `n`, em graus** — dobrado em
/// `[0°, 45°]`.
///
/// ⚠️ **Uma cruz tem quatro braços**, então `d` e `d` rodado de 90° dizem a mesma
/// coisa; medir o ângulo cru daria `90°` a um acordo perfeito. ⛔ E a dobra tem de
/// ser `deg % 90` **antes** do `min`: o `acos` chega a `180°`, e a forma ingénua
/// `45 − |45 − deg|` fica **negativa** ali (medido em 2026-08-22: uma média de
/// `−33,9°`, que é um número que não existe).
fn rosy_deg(a: [f32; 3], b: [f32; 3], n: [f32; 3]) -> f32 {
    let proj = |v: [f32; 3]| {
        let d = dot(v, n);
        unit([
            d.mul_add(-n[0], v[0]),
            d.mul_add(-n[1], v[1]),
            d.mul_add(-n[2], v[2]),
        ])
    };
    let (p, q) = (proj(a), proj(b));
    let deg = dot(p, q).clamp(-1.0, 1.0).acos().to_degrees();
    let m = deg % 90.0;
    m.min(90.0 - m)
}

fn pct(sorted: &[f32], p: f32) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    let i = ((sorted.len() - 1) as f32 * p).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}

/// **Quanto as arestas de cada quad de `out` desviam do campo `dir_of` medido sobre
/// `field_mesh`.** Devolve `(p50, p95, média)` em graus.
///
/// ⚠️ **A busca do vizinho DOBRA o raio até achar alguém**, com teto. Um raio fixo
/// devolveria «sem vizinho» num quad grande sobre uma zona rala — e um quad sem
/// vizinho não conta, o que faria a média **melhorar** exactamente onde a malha
/// está pior ([`ph2d_quadfill::folded_against`] paga esta mesma lição).
fn deviation(
    out: &ph2d_mesh::Mesh,
    field_mesh: &ph2d_mesh::Mesh,
    dir_of: &dyn Fn(usize) -> [f32; 3],
) -> (f32, f32, f32) {
    let normals = field_mesh.face_normals();
    let b = field_mesh.bounds();
    let seed = dot(sub(b.max, b.min), sub(b.max, b.min)).sqrt() * 0.02;
    let pos = out.positions();
    let mut hits: Vec<u32> = Vec::new();
    let mut devs: Vec<f32> = Vec::new();
    let mut pares: Vec<f32> = Vec::new();
    for f in out.faces() {
        let v = f.verts();
        if v.len() < 3 {
            continue;
        }
        let mut c = [0.0f32; 3];
        for &i in v {
            let q = pos[i as usize];
            for k in 0..3 {
                c[k] += q[k] / v.len() as f32;
            }
        }
        let mut best = (f32::INFINITY, usize::MAX);
        let mut radius = seed;
        while best.1 == usize::MAX && radius < seed * 64.0 {
            field_mesh.octree().faces_in_sphere(c, radius, &mut hits);
            for &fi in &hits {
                let rv = field_mesh.faces()[fi as usize].verts();
                let mut rc = [0.0f32; 3];
                for &i in rv {
                    let q = field_mesh.positions()[i as usize];
                    for k in 0..3 {
                        rc[k] += q[k] / rv.len() as f32;
                    }
                }
                let d = dot(sub(rc, c), sub(rc, c)).sqrt();
                if d < best.0 {
                    best = (d, fi as usize);
                }
            }
            radius *= 2.0;
        }
        let Some(&n) = normals.get(best.1) else {
            continue;
        };
        // ⛔⛔ **AS DUAS FAMÍLIAS, e a primeira versão desta sonda media só UMA.**
        //
        // Uma cruz tem quatro braços a 90°, então perguntar *"a aresta 0 está perto
        // de um braço?"* é uma pergunta que um quad **esmagado passa**: basta a
        // família `u` seguir o campo, e a família `v` pode sair a 70° em vez de 90°
        // que a aresta 0 continua a marcar `6°`. ⚠️ *Medir uma família não mede uma
        // grade* — e foi com a versão de uma família que esta sonda disse, em
        // 2026-08-22, que a nossa saída seguia o campo **melhor que a do oráculo**,
        // sobre uma malha com quatro vezes o enviesamento dela.
        let d = dir_of(best.1);
        let a = rosy_deg(sub(pos[v[1] as usize], pos[v[0] as usize]), d, n);
        let b = rosy_deg(sub(pos[v[2] as usize], pos[v[1] as usize]), d, n);
        devs.push(a);
        pares.push(a.max(b));
    }
    devs.sort_by(f32::total_cmp);
    pares.sort_by(f32::total_cmp);
    let mean = pares.iter().sum::<f32>() / pares.len().max(1) as f32;
    (pct(&devs, 0.50), pct(&pares, 0.50), mean)
}

/// ⭐⭐⭐ **A SONDA.** Corre a nossa cadeia e mede a saída contra o **nosso** campo;
/// depois lê a bancada e mede a saída do oráculo contra o campo **dele**.
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   does_the_output_follow_the_field -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- compara a saida com o campo que a produziu"]
fn does_the_output_follow_the_field() {
    for (name, piece, reference) in [
        (
            "ORELHA",
            "sculpt_eared",
            crate::sculpt3d::fixtures::eared_sphere(),
        ),
        (
            "GANCHO",
            "sculpt_hooked",
            crate::sculpt3d::fixtures::hooked_sphere(),
        ),
        (
            "ENRUGADA",
            "sculpt_wrinkled",
            crate::sculpt3d::fixtures::wrinkled_sphere(),
        ),
    ] {
        eprintln!("── {name} ──");
        let mut work = reference.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        let dual = ph2d_crossfield::Dual::build(&work);
        let (field, _) = ph2d_crossfield::solve_miq(&dual);
        let layout = ph2d_trace::trace_patches(&work, &dual, &field);
        for detail in [0.5f32, 1.0] {
            let target = ph2d_quadflow::edge_for_detail_with(
                &reference,
                detail,
                ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
            );
            let Ok(spec) = layout.to_layout(target) else {
                continue;
            };
            let Ok((quant, _)) =
                ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
            else {
                continue;
            };
            let Ok((out, _)) = ph2d_quadfill::fill(
                &work,
                &reference,
                &layout,
                &quant,
                ph2d_quadfill::SMOOTHING_ROUNDS,
            ) else {
                continue;
            };
            let (p50, p95, mean) = deviation(&out, &work, &|f| field.direction(&dual, f));
            eprintln!(
                "  NOSSO d={detail:.2}: desvio da grade ao NOSSO campo — so a familia u {p50:>5.1}° · ⭐AS DUAS {p95:>5.1}° · media das duas {mean:>5.1}°  ({} quads)",
                out.faces().len()
            );
        }
        // ── ⭐ O CONTROLO: o oráculo contra o campo dele.
        let dir = std::path::Path::new(BENCH).join(piece);
        let (Ok(obj), Ok(rosy), Ok(quad)) = (
            std::fs::read_to_string(dir.join(format!("{piece}_rem.obj"))),
            std::fs::read_to_string(dir.join(format!("{piece}_rem.rosy"))),
            std::fs::read_to_string(
                dir.join(format!("{piece}_rem_p0_123_quadrangulation_smooth.obj")),
            ),
        ) else {
            eprintln!("  (a bancada nao esta nesta maquina — sem controlo)");
            continue;
        };
        let Some(mut omesh) = ph2d_mesh::import_obj(&obj).ok().and_then(|mut v| v.pop()) else {
            continue;
        };
        omesh.mesh.triangulate();
        let Some(oquad) = ph2d_mesh::import_obj(&quad).ok().and_then(|mut v| v.pop()) else {
            continue;
        };
        // O formato: contagem, o `N` do `N`-RoSy, e depois uma direção por face.
        let mut it = rosy.lines();
        let Some(Ok(count)) = it.next().map(|l| l.trim().parse::<usize>()) else {
            continue;
        };
        let _ = it.next();
        let dirs: Vec<[f32; 3]> = it
            .filter_map(|l| {
                let v: Vec<f32> = l
                    .split_whitespace()
                    .filter_map(|t| t.parse().ok())
                    .collect();
                (v.len() >= 3).then(|| [v[0], v[1], v[2]])
            })
            .collect();
        if dirs.len() != count || count != omesh.mesh.faces().len() {
            eprintln!("  (o campo do oraculo nao alinha com a malha dele — sem controlo)");
            continue;
        }
        let (p50, p95, mean) = deviation(&oquad.mesh, &omesh.mesh, &|f| dirs[f]);
        eprintln!(
            "  ⭐ ORACULO: desvio da grade ao campo DELE — so a familia u {p50:>5.1}° · ⭐AS DUAS {p95:>5.1}° · media das duas {mean:>5.1}°  ({} quads)",
            oquad.mesh.faces().len()
        );
    }
}

/// ⭐⭐⭐ **O CASO MAIS SIMPLES QUE EXISTE** — uma esfera lisa, sem relevo nenhum.
///
/// ⛔ **Seis hipóteses morreram em duas jornadas** (relaxação · interior alinhado ao
/// campo · domínio ∝ segmentos · «é o alisamento» · «o campo não é combável» ·
/// densidade), e todas foram medidas sobre **esculturas**. ⚠️ *Nunca se perguntou o
/// que a cadeia faz com uma peça que não tem defeito nenhum para expor.*
///
/// ⭐ **É a pergunta que particiona melhor do que qualquer uma das seis:**
///
/// | se a esfera lisa der | então |
/// |---|---|
/// | `~6°`, como o oráculo | o defeito é **acordado pela FEIÇÃO**, e a fixtura certa é o vinco |
/// | `~20°`, como as esculturas | ⭐ o defeito é do **NÚCLEO**, reproduzível sem relevo nenhum — e é aí que ele é mais barato de perseguir |
///
/// ⚠️ **E a esfera lisa está no corpus da bancada** (`sphere_uv_96x144`), então o
/// oráculo já a resolveu: `aspecto p50 1,22 · enviesamento p50 6°`. *A comparação é
/// contra um número que já existe, não contra uma expectativa.*
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   what_does_the_chain_do_to_a_plain_sphere -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- o caso mais simples, contra o mesmo caso no oraculo"]
fn what_does_the_chain_do_to_a_plain_sphere() {
    for (name, piece, reference) in [
        (
            "ESFERA LISA",
            Some("sphere_uv_96x144"),
            ph2d_mesh::shapes::uv_sphere(96, 144, 1.0),
        ),
        (
            "TORO",
            Some("torus_64x32"),
            ph2d_mesh::shapes::torus(64, 32, 1.0, 0.35),
        ),
        // ⚠️ Uma esfera GROSSA: o F1 REFINA em vez de grosseirar, e o caminho é
        // outro. Sem ela, «a esfera lisa está bem» é uma afirmação sobre uma só
        // rota do F1.
        (
            "ESFERA 24x36",
            None,
            ph2d_mesh::shapes::uv_sphere(24, 36, 1.0),
        ),
    ] {
        eprintln!("── {name} ──");
        let mut work = reference.clone();
        work.triangulate();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        let dual = ph2d_crossfield::Dual::build(&work);
        let (field, _) = ph2d_crossfield::solve_miq(&dual);
        let layout = ph2d_trace::trace_patches(&work, &dual, &field);
        // ⭐⭐⭐ **DE QUANTOS LADOS SÃO OS PATCHES.** ⚠️ A dedução «num patch de 4
        // lados a grade é um rectângulo no domínio, logo o enviesamento tem de
        // nascer no mapa» só vale se os patches FOREM de 4 lados — e isso nunca foi
        // contado. *Um raciocínio sobre `n = 4` não descreve uma malha de leques.*
        {
            let mut hist: std::collections::BTreeMap<usize, usize> =
                std::collections::BTreeMap::new();
            for sides in &layout.side_arcs {
                *hist.entry(sides.len()).or_default() += 1;
            }
            eprintln!("  lados por patch: {hist:?}");
        }
        for detail in [0.35f32, 0.55, 0.8] {
            let target = ph2d_quadflow::edge_for_detail_with(
                &reference,
                detail,
                ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
            );
            let Ok(spec) = layout.to_layout(target) else {
                eprintln!("  d={detail:.2} | o layout RECUSOU");
                continue;
            };
            let Ok((quant, _)) =
                ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
            else {
                eprintln!("  d={detail:.2} | a quantizacao RECUSOU");
                continue;
            };
            // ⭐⭐⭐ **AS DUAS ORIGENS DO INTERIOR, na fixtura LIMPA.** ⚠️ As duas
            // curas de 2026-08-23 foram medidas sobre a orelha a `d = 1,0` — 78 mil
            // quads com todas as patologias ao mesmo tempo. *Uma cura medida numa
            // fixtura que não isola o fenómeno lê-se como inútil*
            // ([[feedback_a_cure_measured_on_a_fixture_that_lacks_the_phenomenon_reads_as_useless]]),
            // e aqui o sinal é um número só.
            for interior in [
                ph2d_quadfill::Interior::FromBoundary,
                ph2d_quadfill::Interior::AlignedToField,
            ] {
                let Ok((out, r)) = ph2d_quadfill::fill_with(
                    &work,
                    &reference,
                    &layout,
                    &quant,
                    ph2d_quadfill::SMOOTHING_ROUNDS,
                    ph2d_quadfill::SQUARE_ROUNDS,
                    interior,
                ) else {
                    eprintln!("  d={detail:.2} | a montagem RECUSOU");
                    continue;
                };
                let s = ph2d_quadfill::quad_shape(&out);
                let rotulo = match interior {
                    ph2d_quadfill::Interior::FromBoundary => "fronteira",
                    ph2d_quadfill::Interior::AlignedToField => "⭐CAMPO  ",
                };
                eprintln!(
                    "  d={detail:.2} {rotulo} {:>6} quads · {} patches | aspecto p50 {:.2} p99 {:>5.1} \
                     | ⭐enviesamento p50 {:>3.0}° p99 {:>3.0}° (>60°: {}) | dobras {} \
                     | ⭐rectangulo {:>3.0}° LEQUE {:>3.0}° \
                     | ⭐⭐DOMINIO rect {:>4.1}° (n={}) leque {:>4.1}° (n={}) \
                     | ⭐⭐⭐deslizou {}/{} (quads {}) recusas {:?} | ⭐CONFORME {:.2}",
                    out.faces().len(),
                    r.patches,
                    s.aspect_p50,
                    s.aspect_p99,
                    s.skew_p50,
                    s.skew_p99,
                    s.skew_over_60,
                    r.folded_local,
                    r.skew_by_fan.0,
                    r.skew_by_fan.1,
                    r.domain_skew.0,
                    // ⭐⭐⭐ **A CONTAGEM ao lado da mediana, e ela não é decoração:**
                    // esta coluna imprimiu `0,0°` durante um dia com o balde VAZIO, e
                    // esse zero leu-se como «perfeito». Ver `ph2d_quadfill::FillReport::domain_cells`.
                    r.domain_cells.0,
                    r.domain_skew.1,
                    r.domain_cells.1,
                    r.slid,
                    // ⚠️ O denominador é o total de patches: o `quad_patches` só serve
                    // o mapa do rectângulo, e com o LSCM a fracção saía `16/5`.
                    r.patches,
                    r.quad_patches,
                    // ⭐⭐⭐ **O MOTIVO ao lado do numerador** — ver
                    // `ph2d_quadfill::FillReport::slid_refused`. Sem ele, `1/2` não
                    // distingue «o mapa não serve» de «uma rede é severa demais».
                    r.slid_refused,
                    // ⭐⭐⭐ **O CONTROLO da troca de achatamento** — `1,00` é conforme
                    // perfeito. Sem ele, «o mapa novo não melhorou» não distingue *o
                    // mapa não é o constrangimento* de *o meu mapa tem um bug*.
                    r.conformal,
                );
            }
        }
        let Some(piece) = piece else { continue };
        let path = std::path::Path::new(BENCH)
            .join(piece)
            .join(format!("{piece}_rem_p0_123_quadrangulation_smooth.obj"));
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Some(o) = ph2d_mesh::import_obj(&text).ok().and_then(|mut v| v.pop()) {
            let s = ph2d_quadfill::quad_shape(&o.mesh);
            eprintln!(
                "  ⭐ORACULO {:>6} quads              | aspecto p50 {:.2} p99 {:>5.1} \
                 | ⭐enviesamento p50 {:>3.0}° p99 {:>3.0}° (>60°: {})",
                o.mesh.faces().len(),
                s.aspect_p50,
                s.aspect_p99,
                s.skew_p50,
                s.skew_p99,
                s.skew_over_60,
            );
        }
    }
}
