//! ⭐⭐⭐ **DE QUANTOS LADOS SÃO OS PATCHES — os nossos e os DELE.**
//!
//! # A pergunta, e por que ela decide de quem é a obra
//!
//! Medido em 2026-08-23, com as réguas de valência já corrigidas (`PLAN.md`
//! §4-duetquadragies): numa esfera **lisa**, as faces vindas de patches de
//! **quatro lados** medem `16°` de enviesamento e as vindas de **leque** medem `19°`,
//! contra `6°` do oráculo. ⚠️ **E o nosso traçado entrega, nessa mesma esfera, `8`
//! triângulos, `5` quadriláteros e `3` pentágonos.**
//!
//! ⇒ *Se o oráculo entregar sobretudo quadriláteros, o leque é sintoma de um **F3**
//! que emite patches a mais com `n ≠ 4`, e a obra é no traçado. Se ele também entregar
//! muitos leques e mesmo assim medir `6°`, a obra é no preenchimento.*
//!
//! # ⭐⭐⭐ A RESPOSTA (2026-08-23), e ela mata uma obra e abre outra
//!
//! | peça | NOSSOS patches | cantos | ⭐ DELE patches | cantos |
//! |---|---|---|---|---|
//! | esfera **lisa** | **16** | **26** | **8** | **6** |
//! | enrugada | 14 | 22 | **8** | **6** |
//! | orelha | 17 | 28 | **12** | 13 |
//! | gancho | 26 | 39 | **15** | 17 |
//!
//! ⛔⛔ **A família «reescrever o F3 para emitir só quadriláteros» MORRE aqui.** Na
//! esfera lisa e na enrugada o oráculo entrega **`{3: 8}` — oito patches TRIANGULARES,
//! `0 %` de quadriláteros** — e mede `6°` de enviesamento. *A referência usa mais
//! leques do que nós e sai mais quadrada.* ⇒ o leque **não** é a causa.
//!
//! ⭐⭐⭐ **E o que aparece no lugar é maior:** nós fragmentamos **o dobro**. Na esfera
//! lisa são `16` patches e `26` cantos contra `8` e `6`. Cada canto é uma esquina onde
//! a grade tem de mudar de direcção, e cada fronteira de patch é um sítio onde a
//! subdivisão por comprimento de arco impõe a discordância conforme que o
//! [`ph2d_quadfill::rectangle`] nomeou. *Menos patches não é elegância — é menos
//! fronteiras onde o defeito nomeado pode nascer.*
//!
//! # ⚠️ A MESMA régua nos dois lados, e as DUAS validações
//!
//! O oráculo não publica «quantos lados tem cada patch» — ele publica **o dono de cada
//! face** (`*_rem_p0.patch`). A valência tem de ser **derivada**: um **canto** é um
//! vértice onde três ou mais patches se encontram, e a valência de um patch é quantos
//! cantos ele toca.
//!
//! ⚠️ **O primeiro controlo REPROVOU, e ele estava certo em reprovar.** Comparar a
//! derivação com o [`ph2d_trace::PatchLayout::side_arcs`] do nosso lado dá números
//! diferentes — mas isso **não** condena a derivação: o `side_arcs` conta *lados*, e um
//! lado nosso pode ser feito de **vários arcos** que confinam com patches diferentes.
//! *São duas definições, não duas medições da mesma coisa.*
//!
//! ⭐⭐ **A validação que vale é a que não precisa de acreditar em nenhuma das duas:
//! `χ = V − E + F` sobre o complexo dos patches** (ver [`euler`]). Ela fecha em **`2`
//! nas quatro fixturas e nos dois lados** — logo a derivação descreve uma decomposição
//! de esfera coerente, aqui e lá. *É o único número desta sonda que se pode citar.*

use std::collections::{BTreeMap, BTreeSet};

use ph2d_mesh::Mesh;

const BENCH: &str = "/home/enio/Documentos/Projetos/ph2d-quadbench/ref";

/// **A VALÊNCIA DE CADA PATCH, derivada do dono de cada face.**
///
/// Devolve o histograma `lados -> quantos patches`. ⚠️ Um patch sem canto nenhum (um
/// anel ou uma calota inteira) aparece com `0`, e isso é uma resposta: ele não tem
/// esquina onde a grade possa mudar de direcção.
fn valence_from_owner(mesh: &Mesh, owner: &[u32]) -> (BTreeMap<usize, usize>, usize) {
    let mut at_vertex: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    for (f, &p) in owner.iter().enumerate() {
        let Some(face) = mesh.faces().get(f) else {
            continue;
        };
        for &v in face.verts() {
            at_vertex.entry(v).or_default().insert(p);
        }
    }
    let mut corners: BTreeMap<u32, usize> = BTreeMap::new();
    for &p in owner {
        corners.entry(p).or_insert(0);
    }
    let mut total = 0usize;
    for owners in at_vertex.values() {
        if owners.len() < 3 {
            continue;
        }
        total += 1;
        for &p in owners {
            *corners.entry(p).or_insert(0) += 1;
        }
    }
    let mut hist: BTreeMap<usize, usize> = BTreeMap::new();
    for n in corners.values() {
        *hist.entry(*n).or_default() += 1;
    }
    (hist, total)
}

fn quad_share(hist: &BTreeMap<usize, usize>) -> f32 {
    let total: usize = hist.values().sum();
    if total == 0 {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let q = *hist.get(&4).unwrap_or(&0) as f32 / total as f32;
    q * 100.0
}

/// ⭐⭐⭐ **A VALIDAÇÃO QUE NÃO DEPENDE DE ACREDITAR NA RÉGUA: `χ = V − E + F`.**
///
/// O complexo dos patches é ele próprio uma superfície: os **cantos** são vértices, os
/// **lados** arestas (cada uma partilhada por dois patches, logo `E = Σlados / 2`) e os
/// **patches** faces. Numa peça de género 0 isso tem de dar **`2`**.
///
/// ⚠️ **É a única forma de julgar a derivação sobre a decomposição DELE**, onde não há
/// um `side_arcs` para comparar. ⛔ *Uma tabela de valências que não fecha em `χ = 2`
/// não descreve uma decomposição de esfera nenhuma, e não se reporta.*
///
/// Devolve `(cantos, χ)` — `None` quando `Σlados` é ímpar, que já é a resposta *«estes
/// lados não são partilhados dois a dois»*.
fn euler(hist: &BTreeMap<usize, usize>, corners: usize) -> Option<(usize, i64)> {
    let faces: usize = hist.values().sum();
    let sides: usize = hist.iter().map(|(n, c)| n * c).sum();
    if !sides.is_multiple_of(2) {
        return None;
    }
    let e = i64::try_from(sides / 2).ok()?;
    let v = i64::try_from(corners).ok()?;
    let f = i64::try_from(faces).ok()?;
    Some((corners, v - e + f))
}

/// ⭐⭐⭐ **QUANTOS DOS PATCHES DELE SÃO QUADRILÁTEROS, E QUANTOS DOS NOSSOS.**
///
/// ```text
///   cargo test -p ph2d-host-desktop --release --bin ph2d-host-desktop \
///   how_many_sides_do_the_patches_have -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- le a bancada GPL-isolada fora da arvore"]
fn how_many_sides_do_the_patches_have() {
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
        (
            "ESFERA LISA",
            "sphere_uv_96x144",
            ph2d_mesh::shapes::uv_sphere(96, 144, 1.0),
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

        // ⭐⭐ **O CONTROLO POSITIVO DA RÉGUA** — ver o doc do módulo.
        let mut declared: BTreeMap<usize, usize> = BTreeMap::new();
        for sides in &layout.side_arcs {
            *declared.entry(sides.len()).or_default() += 1;
        }
        let (derived, our_corners) = valence_from_owner(&work, &layout.face_patch);
        eprintln!(
            "  NOSSO   {} patches · declarado {declared:?} ({:.0}% quads)",
            layout.side_arcs.len(),
            quad_share(&declared),
        );
        eprintln!(
            "          derivado {derived:?} ({:.0}% quads) · {our_corners} cantos · {}",
            quad_share(&derived),
            match euler(&derived, our_corners) {
                Some((_, 2)) => "⭐ χ = 2, a derivacao fecha".to_string(),
                Some((_, x)) => format!("⛔ χ = {x}, a derivacao NAO fecha"),
                None => "⛔ Σlados IMPAR -- os lados nao sao partilhados dois a dois".to_string(),
            }
        );

        // ── A decomposição DELE, com a mesma derivação.
        let dir = std::path::Path::new(BENCH).join(piece);
        let (Ok(obj), Ok(pat)) = (
            std::fs::read_to_string(dir.join(format!("{piece}_rem_p0.obj"))),
            std::fs::read_to_string(dir.join(format!("{piece}_rem_p0.patch"))),
        ) else {
            eprintln!("  (a bancada nao esta nesta maquina — sem controlo)");
            continue;
        };
        let Some(mut om) = ph2d_mesh::import_obj(&obj).ok().and_then(|mut v| v.pop()) else {
            continue;
        };
        om.mesh.triangulate();
        let mut it = pat.split_whitespace();
        let Some(Ok(pn)) = it.next().map(str::parse::<usize>) else {
            continue;
        };
        let owner: Vec<u32> = it.filter_map(|t| t.parse().ok()).collect();
        // ⚠️ **As duas fontes têm de falar da MESMA malha, e isso CONFERE-SE.**
        if owner.len() != pn || pn != om.mesh.faces().len() {
            eprintln!(
                "  ⚠️ o gabarito nao alinha: {} faces · {} da decomposicao",
                om.mesh.faces().len(),
                owner.len()
            );
            continue;
        }
        let (his, his_corners) = valence_from_owner(&om.mesh, &owner);
        eprintln!("  ⭐ORACULO {} patches", his.values().sum::<usize>(),);
        eprintln!(
            "          derivado {his:?} ({:.0}% quads) · {his_corners} cantos · {}",
            quad_share(&his),
            match euler(&his, his_corners) {
                Some((_, 2)) => "⭐ χ = 2, a derivacao fecha".to_string(),
                Some((_, x)) => format!("⛔ χ = {x}, a derivacao NAO fecha"),
                None => "⛔ Σlados IMPAR".to_string(),
            }
        );

        // ⭐⭐⭐ **E DE ONDE VÊM OS CANTOS: as SINGULARIDADES do campo.** Ver
        // [`sings`]. ⚠️ Ela é a fase ANTERIOR — se os dois campos tiverem a mesma
        // contagem, a fragmentação é do **traçado**; se o nosso tiver muito mais, a
        // dívida é do **campo**, e mexer no traçado seria tratar o sintoma.
        //
        // ⛔⛔ **O CAMPO DELE É DE OUTRA MALHA QUE NÃO A DA DECOMPOSIÇÃO, e isso
        // custou uma sonda** (medido 2026-08-23): o `*_rem.rosy` tem uma direção por
        // face do **`_rem.obj`** (9 534 na enrugada) e o `*_rem_p0.patch` um dono por
        // face do **`_rem_p0.obj`** (9 638) — o segundo é o primeiro já **cortado nas
        // feature lines**. ⚠️ *Cruzá-los mede o campo de uma face nos patches de outra*,
        // e o número sai plausível. ⇒ o campo carrega-se da malha DELE.
        let (Ok(fobj), Ok(rosy)) = (
            std::fs::read_to_string(dir.join(format!("{piece}_rem.obj"))),
            std::fs::read_to_string(dir.join(format!("{piece}_rem.rosy"))),
        ) else {
            continue;
        };
        let Some(fm) = ph2d_mesh::import_obj(&fobj).ok().and_then(|mut v| v.pop()) else {
            continue;
        };
        let mut fmesh = fm.mesh;
        // ⚠️ **Triangular NÃO pode mudar a contagem**, senão a `n`-ésima direção do
        // ficheiro deixa de ser a da `n`-ésima face.
        let before = fmesh.faces().len();
        fmesh.triangulate();
        if fmesh.faces().len() != before {
            eprintln!("  ⚠️ a malha do campo dele nao e' de triangulos — sem comparacao");
            continue;
        }
        let mut rit = rosy.lines();
        let Some(Ok(count)) = rit.next().map(|l| l.trim().parse::<usize>()) else {
            continue;
        };
        let _ = rit.next();
        let dirs: Vec<[f32; 3]> = rit
            .filter_map(|l| {
                let v: Vec<f32> = l
                    .split_whitespace()
                    .filter_map(|t| t.parse().ok())
                    .collect();
                (v.len() >= 3).then(|| [v[0], v[1], v[2]])
            })
            .collect();
        if dirs.len() != count || count != fmesh.faces().len() {
            eprintln!(
                "  ⚠️ o campo dele nao alinha com a malha dele ({count} direcoes · {} faces) — sem comparacao",
                fmesh.faces().len()
            );
            continue;
        }
        let his_dual = ph2d_crossfield::Dual::build(&fmesh);
        let Some(his_field) = ph2d_crossfield::CrossField::from_directions(&his_dual, &dirs) else {
            continue;
        };
        eprintln!(
            "          ⭐SINGULARIDADES: nos {} · ele {}",
            sings(&work, &dual, &field),
            sings(&fmesh, &his_dual, &his_field),
        );

        // ⭐⭐⭐ **E O PARTIDOR FINAL: quantos dos NOSSOS cantos estão numa
        // singularidade?**
        //
        // Um canto é uma esquina onde a grade muda de direcção, e **a única razão
        // legítima para ele existir é uma singularidade do campo** — é lá que a grade
        // não pode continuar recta. ⇒ um canto num vértice **regular** é uma esquina
        // que o traçado INVENTOU: nada no campo a pedia, e ela paga-se em irregulares
        // na saída e numa fronteira a mais onde a subdivisão por comprimento de arco
        // impõe a discordância conforme.
        //
        // ⚠️ *Sem esta linha, «fragmentamos o dobro» não diz se o excesso é legítimo.*
        // ⚠️⚠️ **DUAS COISAS DIFERENTES, e confundi-las custou uma afirmação errada**
        // (2026-08-23). O [`ph2d_trace::PatchLayout`] tem **cantos** próprios
        // (`corners`), decididos pelo **ângulo interno daquele patch naquele vértice**;
        // e tem **nós** — as pontas dos arcos —, que incluem toda junção em T. *Um nó
        // em T é canto para os patches do lado da haste e MEIO DE LADO para o do outro
        // lado.* ⇒ contar pontas de arco e chamar-lhes cantos **sobrestima**, e foi o
        // que eu fiz. As duas linhas saem juntas de propósito.
        let idx = ph2d_crossfield::vertex_index(&work, &dual, &field);
        let is_sing = |v: &u32| idx.get(*v as usize).copied().unwrap_or(0) != 0;
        let sing_verts = idx.iter().filter(|k| **k != 0).count();

        let corner_verts: BTreeSet<u32> = layout.corners.iter().flatten().copied().collect();
        let c_sing = corner_verts.iter().filter(|v| is_sing(v)).count();
        let nodes: BTreeSet<u32> = layout
            .arc_chain
            .iter()
            .filter_map(|c| Some((*c.first()?, *c.last()?)))
            .flat_map(|(a, b)| [a, b])
            .collect();
        let n_sing = nodes.iter().filter(|v| is_sing(v)).count();
        eprintln!(
            "          ⛔CANTOS (angulo) {} · em singularidade {c_sing} · fora dela {} \
             || NOS de arco {} · em singularidade {n_sing} · fora dela {} \
             || singularidades SEM no {}",
            corner_verts.len(),
            corner_verts.len() - c_sing,
            nodes.len(),
            nodes.len() - n_sing,
            sing_verts.saturating_sub(n_sing),
        );

        // ⭐⭐ **O CUSTO CONCRETO de um nó a mais: um LADO partido em vários ARCOS.**
        //
        // ⚠️ É a ligação — testável — com o defeito que o [`ph2d_quadfill::rectangle`]
        // nomeou: *dentro de UM arco a reamostragem por `τ` é proporcional, então não há
        // desvio; a divergência entre lados opostos aparece quando um lado tem VÁRIOS
        // arcos com densidades diferentes.* ⇒ quantos lados nossos são feitos de mais
        // de um arco é a medida directa de quanta discordância o traçado impõe ao F5.
        let (mut sides_total, mut sides_multi, mut worst) = (0usize, 0usize, 0usize);
        for sides in &layout.side_arcs {
            for s in sides {
                sides_total += 1;
                if s.len() > 1 {
                    sides_multi += 1;
                }
                worst = worst.max(s.len());
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let pct = 100.0 * sides_multi as f32 / sides_total.max(1) as f32;
        eprintln!(
            "          ⭐LADOS: {sides_total} · com MAIS de um arco {sides_multi} ({pct:.0}%) · pior {worst} arcos"
        );

        // ⛔⛔ **A AFIRMAÇÃO QUE FALTAVA VERIFICAR: um canto a mais custa um IRREGULAR
        // a mais na saída?**
        //
        // ⚠️ Ela foi escrita como se fosse óbvia (2026-08-23) e **não é**: o `corners`
        // do layout é per-patch, e um nó em T é canto para os patches do lado da haste e
        // **meio de lado** para o do outro. *Um vértice onde três patches se encontram
        // pode sair com valência 4 na malha final.* ⇒ pergunta-se à SAÍDA.
        let target = ph2d_quadflow::edge_for_detail_with(
            &reference,
            0.55,
            ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
        );
        // ⚠️ **A RECUSA IMPRIME-SE, e não se salta.** Uma cura a montante pode deixar
        // um layout que o F4 ou o F5 recusam — e um `if let` silencioso faria isso
        // ler-se como *«esta fixtura não foi medida»* em vez de *«a cura partiu a
        // cadeia»*. ⛔ *Um caminho de erro sem voz é um resultado a menos e um defeito
        // escondido.*
        match layout.to_layout(target) {
            Err(e) => eprintln!("          ⛔ o LAYOUT recusou: {e:?}"),
            Ok(spec) => {
                match ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512)) {
                    Err(e) => eprintln!("          ⛔ a QUANTIZACAO recusou: {e:?}"),
                    Ok((quant, _)) => {
                        match ph2d_quadfill::fill(
                            &work,
                            &reference,
                            &layout,
                            &quant,
                            ph2d_quadfill::SMOOTHING_ROUNDS,
                        ) {
                            Err(e) => eprintln!("          ⛔ a MONTAGEM recusou: {e:?}"),
                            Ok((_, r)) => {
                                #[allow(clippy::cast_precision_loss)]
                                let ours = 100.0 * r.irregular as f32 / r.verts.max(1) as f32;
                                eprintln!(
                                    "          ⭐IRREGULARES na saida: nos {} de {} ({ours:.2}%) · ele {}",
                                    r.irregular,
                                    r.verts,
                                    irregular_of(&dir, piece),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

/// **QUANTOS IRREGULARES A MALHA DE QUADS DELE TEM** — lidos da saída final.
///
/// ⚠️ **A mesma definição da nossa** (`valência ≠ 4`, ignorando bordo), para a linha
/// ficar comparável. *Duas definições de irregular dariam uma diferença que não existe.*
fn irregular_of(dir: &std::path::Path, piece: &str) -> String {
    let path = dir.join(format!("{piece}_rem_p0_123_quadrangulation_smooth.obj"));
    let Ok(text) = std::fs::read_to_string(&path) else {
        return "(sem ficheiro)".to_string();
    };
    let Some(o) = ph2d_mesh::import_obj(&text).ok().and_then(|mut v| v.pop()) else {
        return "(nao le)".to_string();
    };
    let mut deg: BTreeMap<u32, usize> = BTreeMap::new();
    for f in o.mesh.faces() {
        for &v in f.verts() {
            *deg.entry(v).or_default() += 1;
        }
    }
    let n = deg.values().filter(|d| **d != 4).count();
    #[allow(clippy::cast_precision_loss)]
    let pct = 100.0 * n as f32 / deg.len().max(1) as f32;
    format!("{n} de {} ({pct:.2}%)", deg.len())
}

/// **A CONTAGEM E A SOMA DOS ÍNDICES, com Poincaré–Hopf ao lado.**
///
/// ⚠️ **A soma é a invariante e a contagem é o produto.** Duas malhas do mesmo género
/// têm forçosamente a mesma soma, e uma pode ter **oito** singularidades e a outra
/// **duzentas**, em pares `+1/−1` que se cancelam. ⭐ *Imprimir a soma ao lado é o que
/// impede citar uma contagem sobre um campo cuja conta nem fecha.*
fn sings(mesh: &Mesh, dual: &ph2d_crossfield::Dual, field: &ph2d_crossfield::CrossField) -> String {
    let (n, sum) = ph2d_crossfield::singularities(mesh, dual, field);
    format!("{n:>3} (Σ = {sum})")
}
