//! ⭐⭐⭐ **A HOLONOMIA DENTRO DE UM PATCH** — quantos deles têm singularidade dentro.
//!
//! ⚠️ **Irmã da [`super::field_follow`] pelo teto de LOC da shell (HR-18, 600) e por
//! ASSUNTO:** lá se pergunta se a SAÍDA segue o campo; aqui se o CAMPO é combável
//! dentro de cada patch, que é uma propriedade da decomposição e não da malha final.
//!
//! ⛔⛔ **A linha «⭐ORACULO» desta sonda está RETIRADA** (2026-08-23): ela cruzava o
//! campo do `_rem.obj` (9 534 faces na enrugada) com os patches do `_rem_p0.obj`
//! (9 638 — o primeiro já cortado nas *feature lines*), e a conferência nomeava o risco
//! sem nunca comparar as duas contagens. ⭐ O fenómeno tem hoje instrumento próprio e
//! sem controlo emprestado: *singularidades sem nó*, em [`super::patch_valence`].

const BENCH: &str = "/home/enio/Documentos/Projetos/ph2d-quadbench/ref";

/// ⭐⭐⭐ **QUANTOS PATCHES TÊM SINGULARIDADE DENTRO** — a distribuição que o máximo
/// de 2026-08-23 não dava, e o gabarito do oráculo ao lado.
///
/// ⛔ **O que se sabia:** pentear o campo dentro de um patch deixava `29°` (orelha),
/// `44°` (gancho) e `16°` (enrugada) de resíduo, onde um patch sem singularidade
/// interior daria ~`0°`. ⚠️ **Mas aquilo era um MÁXIMO sobre todas as arestas de
/// todos os patches** — ele não distingue *«um patch está sujo»* de *«todos estão»*,
/// e as duas leituras mandam construir coisas diferentes.
///
/// ⭐⭐ **E o CONTROLO é o oráculo com a decomposição DELE**, lida da bancada
/// (`*_rem_p0.patch`, o dono de cada face). Se os patches dele também tiverem
/// resíduo, a promessa «as singularidades ficam nos cantos» é uma expectativa minha
/// e não uma propriedade da família — e o diagnóstico muda de dono.
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   how_many_patches_are_uncombable -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- a distribuicao da holonomia, nossa e do oraculo"]
fn how_many_patches_are_uncombable() {
    use std::collections::BTreeMap;

    // ── O nosso lado.
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
        report("  NOSSO  ", &work, &layout.face_patch, &layout.face_dir);

        // ── ⭐ O CONTROLO: a malha, o campo e a decomposição DELE.
        let dir = std::path::Path::new(BENCH).join(piece);
        let (Ok(obj), Ok(rosy), Ok(pat)) = (
            std::fs::read_to_string(dir.join(format!("{piece}_rem_p0.obj"))),
            std::fs::read_to_string(dir.join(format!("{piece}_rem.rosy"))),
            std::fs::read_to_string(dir.join(format!("{piece}_rem_p0.patch"))),
        ) else {
            eprintln!("  (a bancada nao esta nesta maquina — sem controlo)");
            continue;
        };
        let Some(mut om) = ph2d_mesh::import_obj(&obj).ok().and_then(|mut v| v.pop()) else {
            continue;
        };
        om.mesh.triangulate();
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
        let mut pit = pat.split_whitespace();
        let Some(Ok(pn)) = pit.next().map(str::parse::<usize>) else {
            continue;
        };
        let owner: Vec<u32> = pit.filter_map(|t| t.parse().ok()).collect();
        // ⛔⛔⛔ **A CONFERÊNCIA ESTAVA INCOMPLETA, e ela mediu o que dizia não
        // medir** (achado 2026-08-23). O comentário abaixo já nomeava o risco —
        // *«o campo vem do `_rem.obj` e a decomposição do `_rem_p0.obj`»* — e as três
        // condições **nunca comparavam `count` com `pn`**: elas só exigiam que o campo
        // batesse consigo próprio e a decomposição consigo própria.
        //
        // ⚠️ **Medido: `_rem.obj` tem `9 534` faces e `_rem_p0.obj` tem `9 638`** (a
        // enrugada) — o segundo é o primeiro já **cortado nas feature lines**. ⇒ a
        // linha «⭐ORACULO» desta sonda cruzava o campo de uma malha com os patches de
        // outra, e o número saía plausível.
        //
        // ⛔ **Os dois ficheiros não são cruzáveis**, e não há terceiro que os ligue.
        // ⭐ O fenómeno tem hoje um instrumento melhor e sem controlo emprestado:
        // «singularidades sem canto» em `sculpt3d_patch_valence.rs`, que conta
        // directamente quantos patches NOSSOS contêm uma singularidade dentro.
        if dirs.len() != count || owner.len() != pn || pn != om.mesh.faces().len() || count != pn {
            eprintln!(
                "  ⚠️ o gabarito nao alinha: {} faces · {} do campo · {} da decomposicao",
                om.mesh.faces().len(),
                dirs.len(),
                owner.len()
            );
            continue;
        }
        report("  ⭐ORACULO", &om.mesh, &owner, &dirs);
    }

    /// Agrupa as faces por patch, penteia cada um e imprime a distribuição.
    fn report(rotulo: &str, mesh: &ph2d_mesh::Mesh, owner: &[u32], dirs: &[[f32; 3]]) {
        let mut by: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        for (f, &p) in owner.iter().enumerate() {
            if let Ok(f) = u32::try_from(f) {
                by.entry(p).or_default().push(f);
            }
        }
        let mut worst: Vec<(f32, u32, usize)> = Vec::new();
        let mut sujos = 0usize;
        let mut sem_resposta = 0usize;
        let mut fora = 0usize;
        let mut todos: Vec<ph2d_crossfield::Holonomy> = Vec::new();
        for (&p, faces) in &by {
            match ph2d_crossfield::holonomy(mesh, faces, dirs) {
                Some(h) => {
                    // ⛔ **A contagem de «sujos» FICA sem barra própria**: ver
                    // `Holonomy::_MEASURED_AND_REJECTED_CLEAN_BAR`. O que se lê é a
                    // distribuição, e o número aqui é só para a comparação de
                    // ordens de grandeza com a linha do oráculo abaixo.
                    if h.max > 1.0 {
                        sujos += 1;
                    }
                    fora += h.skipped;
                    todos.push(h);
                    worst.push((h.max, p, faces.len()));
                }
                None => sem_resposta += 1,
            }
        }
        worst.sort_by(|a, b| b.0.total_cmp(&a.0));
        eprintln!(
            "{rotulo}: {} patches · {sujos} com max > 1° · \
             {sem_resposta} sem resposta · ⚠️{fora} faces FORA da conta",
            by.len(),
        );
        // ⭐⭐⭐ **A DISTRIBUIÇÃO, e não o máximo.** ⛔ O `max` de um patch é um
        // extremo sobre milhares de arestas e é dominado por ruído: medido em
        // 2026-08-23, o ORÁCULO dá `15°–38°` de máximo nos patches dele — ou seja,
        // uma barra sobre o máximo classifica a REFERÊNCIA como suja e não
        // discrimina nada. *É a mesma lição do extremo global contra a régua
        // por-face, um nível acima.*
        let mut p50: Vec<f32> = todos.iter().map(|h| h.p50).collect();
        let mut p95: Vec<f32> = todos.iter().map(|h| h.p95).collect();
        p50.sort_by(f32::total_cmp);
        p95.sort_by(f32::total_cmp);
        let med = |v: &[f32]| v.get(v.len() / 2).copied().unwrap_or(0.0);
        eprintln!(
            "        ⭐ residuo por patch: p50 mediano {:.3}° · p95 mediano {:.2}° · pior max {:.1}°",
            med(&p50),
            med(&p95),
            worst.first().map_or(0.0, |w| w.0),
        );
        for (h, p, n) in worst.iter().take(3) {
            eprintln!("        patch {p:<4} ({n:>6} faces) residuo max {h:>6.1}°");
        }
    }
}
