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
    }
}
