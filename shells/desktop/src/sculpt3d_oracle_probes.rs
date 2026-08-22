//! ⭐⭐⭐ **A COMPARAÇÃO FASE A FASE CONTRA O ORÁCULO** — o gabarito que estava em
//! disco (`PLAN.md` §4-duotricies).
//!
//! ⛔ **A bancada comparava só o resultado FINAL** (`65–83 %` de quads contra
//! `100 %`), e as duas fases em que esta linha está encalhada há dias — o **campo**
//! e o **traçado** — nunca foram comparadas com nada. *Redescobrir às cegas com a
//! resposta no disco.*
//!
//! O binário do oráculo grava as fases intermédias dele. Por peça do corpus, em
//! `ph2d-quadbench/ref/<peça>/`:
//!
//! | ficheiro | o que é | a nossa fase |
//! |---|---|---|
//! | `*_rem.obj` | a malha remalhada dele | **F1** |
//! | ⭐ `*_rem.rosy` | **o campo cruzado dele**, uma direção por face | **F2** |
//! | ⭐ `*_rem_p0.patch` | **a decomposição dele**, o patch dono de cada face | **F3** |
//! | `*_quadrangulation.obj` | a malha final | F5 |
//!
//! ⚠️ **Ler a SAÍDA de um programa não é obra derivada** — é legal, é o padrão, e é
//! **mais forte** que ler o código: em vez de interpretar intenção, compara-se
//! número com número, face a face.
//!
//! ⚠️⚠️ **A comparação corre sobre a malha DELE**, nunca sobre a nossa. Se cada lado
//! resolvesse na sua própria malha, a diferença medida misturaria o solver com o F1
//! — e o F1 é uma fase que já sabemos que diverge.
//!
//! ⚠️ **Estas sondas dependem de um caminho FORA do repositório** e por isso são
//! `#[ignore]` e saem em silêncio quando ele não existe. ⛔ *Skip gracioso não é
//! verde* — nenhuma delas é gate.

/// Onde o oráculo mora. ⚠️ Fora da árvore de propósito (ADR-0162): o binário é GPL,
/// e o que entra aqui é a **saída** dele.
const BENCH: &str = "/home/enio/Documentos/Projetos/ph2d-quadbench/ref";

/// Lê a malha remalhada do oráculo e o campo dele. `None` quando a bancada não está
/// nesta máquina.
fn oracle(piece: &str) -> Option<(ph2d_mesh::Mesh, Vec<[f32; 3]>)> {
    let dir = std::path::Path::new(BENCH).join(piece);
    let obj = std::fs::read_to_string(dir.join(format!("{piece}_rem.obj"))).ok()?;
    let rosy = std::fs::read_to_string(dir.join(format!("{piece}_rem.rosy"))).ok()?;
    let mut mesh = ph2d_mesh::import_obj(&obj).ok()?.pop()?.mesh;
    mesh.triangulate();

    // ⚠️ **O formato: contagem, o `N` do `N`-RoSy, e depois uma direção por face.**
    // As duas primeiras linhas não são geometria — lê-las como tal desalinharia o
    // campo inteiro em duas faces, e o erro apareceria como *"o campo dele é mau"*.
    let mut it = rosy.lines();
    let count: usize = it.next()?.trim().parse().ok()?;
    let _rosy_n: usize = it.next()?.trim().parse().ok()?;
    let dirs: Vec<[f32; 3]> = it
        .filter_map(|l| {
            let v: Vec<f32> = l
                .split_whitespace()
                .filter_map(|t| t.parse().ok())
                .collect();
            (v.len() >= 3).then(|| [v[0], v[1], v[2]])
        })
        .collect();
    (dirs.len() == count && count == mesh.faces().len()).then_some((mesh, dirs))
}

/// **O desvio 4-RoSy entre duas direções, em graus** — dobrado em `[0°, 45°]`.
///
/// ⚠️ **Uma cruz tem quatro braços**, então `d` e `d` rodado de 90° dizem a mesma
/// coisa. Medir o ângulo cru daria `90°` a um acordo perfeito.
fn rosy_deg(a: [f32; 3], b: [f32; 3], n: [f32; 3]) -> f32 {
    // Projecta as duas no plano da face e mede o ângulo entre elas.
    let proj = |v: [f32; 3]| {
        let d = v[0].mul_add(n[0], v[1].mul_add(n[1], v[2] * n[2]));
        let p = [
            d.mul_add(-n[0], v[0]),
            d.mul_add(-n[1], v[1]),
            d.mul_add(-n[2], v[2]),
        ];
        let l = p[0]
            .mul_add(p[0], p[1].mul_add(p[1], p[2] * p[2]))
            .sqrt()
            .max(1.0e-9);
        [p[0] / l, p[1] / l, p[2] / l]
    };
    let (a, b) = (proj(a), proj(b));
    let c = a[0]
        .mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
        .clamp(-1.0, 1.0);
    // ⚠️ **A dobra é `mod 90` e depois o menor lado**, e a primeira versão errou-a:
    // ela fazia `45 − |45 − deg|`, que é correcto até `90°` e vai a **NEGATIVO**
    // acima disso — e o `acos` devolve até `180°`. O sintoma foi um desvio médio de
    // **`−33,9°`**, que não existe. *Uma régua que sai fora do próprio contradomínio
    // está errada antes de dizer o que quer que seja.*
    let m = c.acos().to_degrees() % 90.0;
    m.min(90.0 - m)
}

/// **QUANTO UM CAMPO OBEDECE AO RELEVO** — o desvio médio, ponderado pela
/// anisotropia, entre a cruz e a direção principal de curvatura.
///
/// ⭐⭐ **É a régua da [`ph2d_quadfill::follows_relief`] aplicada ao CAMPO em vez da
/// malha**, e é isso que a torna comparável entre os dois solvers: não passa por
/// traçado, quantização nem montagem. *A pergunta é do campo; medi-la na malha
/// final mistura-lhe quatro fases.*
///
/// ⚠️ **`22,5°` é o valor de um campo aleatório** — a média de um ângulo uniforme em
/// `[0°, 45°]`.
fn relief_of(mesh: &ph2d_mesh::Mesh, dirs: &dyn Fn(usize) -> [f32; 3]) -> (f32, f32) {
    let pd = ph2d_mesh::principal_dirs(mesh);
    let normals = mesh.face_normals();
    let (mut wsum, mut asum) = (0.0f64, 0.0f64);
    for f in 0..mesh.faces().len() {
        let a = pd[f].anisotropy;
        if a <= 0.0 {
            continue;
        }
        let d = rosy_deg(dirs(f), pd[f].dir, normals[f]);
        wsum += f64::from(d) * f64::from(a);
        asum += f64::from(a);
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    (
        if asum > 0.0 {
            (wsum / asum) as f32
        } else {
            0.0
        },
        (asum / mesh.faces().len().max(1) as f64) as f32,
    )
}

/// ⭐⭐⭐ **SONDA — O MEU CAMPO CONTRA O DELE, NA MALHA DELE.**
///
/// ⛔ **A pergunta que a bancada nunca fez.** O porte do Instant Meshes mede `13,7°`
/// de obediência ao relevo e a nossa cadeia `25,7°` — mas *"o meu é pior"* é uma
/// diferença **sem endereço**. Esta sonda dá-lhe um: em que faces os dois campos
/// discordam, e quanto.
///
/// | coluna | o que diz |
/// |---|---|
/// | **desvio médio / p95 / máx** | quanto os dois campos discordam, em graus 4-RoSy |
/// | ⭐ **≥ 30°** | a fração de faces em que eles apontam para sítios genuinamente diferentes (`45°` é o máximo possível) |
/// | **relevo** | a régua [`ph2d_quadfill::follows_relief`] aplicada aos dois, na mesma malha |
/// | **singularidades** | quantas cada um produz |
///
/// ⚠️ **Se o desvio médio for pequeno e o relevo divergir**, o problema não é o
/// campo — é o que vem depois. *É essa a bifurcação que esta sonda decide.*
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   my_field_against_the_oracle -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- o meu campo contra o do oraculo, na malha dele"]
fn my_field_against_the_oracle() {
    let pieces = [
        "sculpt_ridged",
        "sculpt_hooked",
        "sculpt_wrinkled",
        "sphere_uv_96x144",
        "torus_64x32",
    ];
    let mut ran = 0usize;
    for piece in pieces {
        let Some((mesh, theirs)) = oracle(piece) else {
            continue;
        };
        ran += 1;
        let dual = ph2d_crossfield::Dual::build(&mesh);
        let (field, rep) = ph2d_crossfield::solve_miq(&dual);
        let normals = mesh.face_normals();

        let mut degs: Vec<f32> = (0..mesh.faces().len())
            .map(|f| rosy_deg(field.direction(&dual, f), theirs[f], normals[f]))
            .collect();
        degs.sort_by(f32::total_cmp);
        let n = degs.len().max(1);
        #[allow(clippy::cast_precision_loss)]
        let mean = degs.iter().sum::<f32>() / n as f32;
        let p95 = degs[(n * 95 / 100).min(n - 1)];
        #[allow(clippy::cast_precision_loss)]
        let far = degs.iter().filter(|d| **d >= 30.0).count() as f64 / n as f64;

        let (mine_sing, mine_sum) = ph2d_crossfield::singularities(&mesh, &dual, &field);
        // ⭐⭐ **A COLUNA QUE DECIDE:** cada campo contra o relevo da MESMA malha.
        let (mine_rel, conf) = relief_of(&mesh, &|f| field.direction(&dual, f));
        let (their_rel, _) = relief_of(&mesh, &|f| theirs[f]);
        println!(
            "── {piece} ({} faces) ──\n  \
             discordancia entre os campos: media {mean:.1}° · p95 {p95:.1}° · max {:.1}° \
             | >= 30°: {:.1} %\n  \
             ⭐ RELEVO — o MEU {mine_rel:.1}° · o DELE {their_rel:.1}° \
             (aleatorio = 22,5°, confianca {conf:.2})\n  \
             singularidades: {mine_sing} (soma {mine_sum}) · {} resolucoes · \
             pior residuo {:.1e}",
            mesh.faces().len(),
            degs[n - 1],
            100.0 * far,
            rep.solves,
            rep.cg_worst_residual,
        );
    }
    if ran == 0 {
        println!("⚠️  a bancada nao esta' nesta maquina ({BENCH}) -- nada a comparar");
    }
}

/// ⭐⭐⭐ **SONDA — QUE PESO DE ALINHAMENTO APROXIMA O CAMPO DO ORÁCULO?**
///
/// ⛔ **A varredura do `ALIGN_WEIGHT` de 2026-08-22 mediu no fim da cadeia** — o
/// relevo da malha montada —, onde o campo, o traçado, a quantização e a montagem
/// estão todos misturados. É por isso que ela dava tabelas não-monótonas e
/// escolhia números que depois partiam noutro sítio.
///
/// ⭐⭐ **Agora há um alvo à saída da PRÓPRIA fase:** o campo do oráculo, na mesma
/// malha, face a face. A sonda irmã mediu-o — `12,1°` na fixtura com cristas contra
/// os `24,3°` do nosso, com `43 %` das faces a discordar em `≥ 30°`.
///
/// | coluna | o que decide |
/// |---|---|
/// | ⭐ **relevo** | quanto o nosso campo obedece à curvatura. **O alvo é o do oráculo**, não «melhor que antes» |
/// | **discordância** | quanto ele se aproxima do campo dele, face a face |
/// | **singularidades** | ⛔ o preço: um campo alinhado tem mais, e acima de certo ponto o traçado parte |
///
/// ⚠️ **Isto NÃO decide o que shipa.** Um peso que ganha aqui ainda tem de passar o
/// gate do género e o da aresta máxima — mas escolhê-lo aqui é escolhê-lo onde a
/// grandeza vive, em vez de o adivinhar quatro fases à frente.
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   which_alignment_weight_matches_the_oracle_field -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- que peso de alinhamento aproxima o campo do oraculo"]
fn which_alignment_weight_matches_the_oracle_field() {
    for piece in ["sculpt_ridged", "sculpt_hooked", "sculpt_wrinkled"] {
        let Some((mesh, theirs)) = oracle(piece) else {
            continue;
        };
        let dual = ph2d_crossfield::Dual::build(&mesh);
        let normals = mesh.face_normals();
        let (their_rel, conf) = relief_of(&mesh, &|f| theirs[f]);
        println!("── {piece}: o ORÁCULO mede {their_rel:.1}° (confianca {conf:.2}) ──");
        for w in [0.0f32, 0.01, 0.03, 0.1, 0.3, 1.0, 3.0] {
            let (field, _) =
                ph2d_crossfield::solve_miq_aligned(&dual, ph2d_crossfield::Rounding::default(), w);
            let (rel, _) = relief_of(&mesh, &|f| field.direction(&dual, f));
            #[allow(clippy::cast_precision_loss)]
            let disc = (0..mesh.faces().len())
                .map(|f| rosy_deg(field.direction(&dual, f), theirs[f], normals[f]))
                .sum::<f32>()
                / mesh.faces().len().max(1) as f32;
            let (sing, sum) = ph2d_crossfield::singularities(&mesh, &dual, &field);
            println!(
                "  peso {w:<5} | ⭐RELEVO {rel:>5.1}° (alvo {their_rel:.1}°) \
                 | discordancia {disc:>5.1}° | singularidades {sing:<4} (soma {sum})"
            );
        }
    }
}
