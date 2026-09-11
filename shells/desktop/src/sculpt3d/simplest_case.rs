//! ⭐⭐⭐ **O CASO MAIS SIMPLES, e a densidade** — as duas sondas que perguntam se o
//! defeito precisa de uma ESCULTURA para aparecer.
//!
//! ⚠️ **Irmã do [`super::quad_shape`] pelo teto de LOC da shell (HR-18, 600) e por
//! ASSUNTO:** lá a régua por-face e a barra do oráculo; aqui as duas perguntas que
//! puseram sete hipóteses de joelhos — *a densidade explica?* e *o defeito existe
//! sem relevo nenhum?*
//!
//! ⛔ **A resposta às duas é a mesma:** não e sim. À contagem do oráculo (`4 162`
//! contra `4 658`) a orelha ainda mede `22°` contra `6°`; e uma esfera **lisa** mede
//! `18°` contra `6°`, com o aspecto quase igual ao dele. *O defeito não precisa de
//! feição nenhuma para existir — e é aqui, e não na orelha, que a próxima hipótese
//! tem de ser medida primeiro.*

use ph2d_mesh::Mesh;

use super::quad_shape::Shown;

fn measure(mesh: &Mesh) -> Shown {
    Shown(ph2d_quadfill::quad_shape(mesh), mesh.faces().len())
}

/// ⭐⭐⭐ **AS DUAS SAÍDAS À MESMA CONTAGEM DE QUADS** — a comparação que nunca foi
/// feita, e a única variável que sobrou depois de cinco hipóteses mortas.
///
/// ⛔ **Todas as tabelas desta linha compararam a nossa saída a `d = 1,0` com a do
/// oráculo** — e as duas contagens não são do mesmo mundo: na orelha, `78 403` contra
/// `4 658`. ⚠️ *Uma malha 17× mais fina não é comparável com uma 17× mais grossa*: a
/// curvatura por célula, a distorção da parametrização e o piso de discretização
/// mudam todos com a densidade. **A diferença de `27°` contra `6°` pode ser
/// inteiramente um artefacto de estarmos a medir duas coisas diferentes.**
///
/// ⭐ Esta sonda varre o `detail` e imprime a linha do oráculo no fim, para se ler a
/// nossa **na contagem dele**.
///
/// | pergunta | como a resposta se lê |
/// |---|---|
/// | o enviesamento CAI quando a densidade cai? | então é a densidade, e o alvo muda |
/// | ele fica em `~27°` em toda a varredura? | então é estrutural e a densidade está ilibada |
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   at_the_same_density_as_the_oracle -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- varre a densidade e poe a linha do oraculo ao lado"]
fn at_the_same_density_as_the_oracle() {
    const BENCH: &str = "/home/enio/Documentos/Projetos/ph2d-quadbench/ref";
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
        // ⚠️ **A varredura é FINA entre `0,5` e `1,0` de propósito:** medido, a
        // contagem salta de `2 868` para `78 403` nesse intervalo — o `detail` não é
        // linear na densidade, e três pontos não descrevem a curva.
        for detail in [0.30f32, 0.45, 0.55, 0.65, 0.75, 0.85, 1.0] {
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
            let Ok((out, r)) = ph2d_quadfill::fill(
                &work,
                &reference,
                &layout,
                &quant,
                ph2d_quadfill::SMOOTHING_ROUNDS,
            ) else {
                eprintln!("  d={detail:.2} | a montagem RECUSOU");
                continue;
            };
            eprintln!(
                "  d={detail:.2} {} | dobras {}",
                measure(&out),
                r.folded_local
            );
        }
        // ⭐ A linha do oráculo, com o MESMO código de medição.
        let path = std::path::Path::new(BENCH)
            .join(piece)
            .join(format!("{piece}_rem_p0_123_quadrangulation_smooth.obj"));
        let Ok(text) = std::fs::read_to_string(&path) else {
            eprintln!("  (a bancada nao esta nesta maquina — sem controlo)");
            continue;
        };
        if let Some(o) = ph2d_mesh::import_obj(&text).ok().and_then(|mut v| v.pop()) {
            eprintln!("  ⭐ORACULO  {}", measure(&o.mesh));
        }
    }
}

/// ⛔⛔ **VERMELHO — a ESFERA LISA é a reprodução mais barata do defeito**, e ela
/// existia desde sempre sem ninguém a ter corrido.
///
/// ⭐⭐⭐ **Sete hipóteses morreram sobre ESCULTURAS** (relaxação · interior alinhado
/// ao campo · domínio ∝ segmentos · alisamento · combabilidade · densidade · mapa
/// conforme). ⚠️ *Nunca se tinha perguntado o que a cadeia faz com uma peça sem
/// relevo nenhum* — e a resposta é que o defeito está lá **inteiro**:
///
/// | esfera lisa, `d = 0,55` | nós | ⭐ oráculo |
/// |---|---|---|
/// | quads | 2 006 | 3 352 |
/// | aspecto p50 | `1,26` | **`1,22`** — quase igual |
/// | ⛔ **enviesamento p50** | **`18°`** | **`6°`** |
/// | ⛔ faces `> 60°` | **141** | **0** |
///
/// ⭐⭐ **As células têm as PROPORÇÕES certas e os ÂNGULOS errados.** Não é
/// densidade (a `d = 0,35` dá `19°` com 758 quads), não é feição (não há nenhuma),
/// não é o alisamento, não é o mapa. *É a reprodução mais limpa que esta linha tem, e
/// toda hipótese nova deve ser medida AQUI primeiro* — uma cura medida numa fixtura
/// que não isola o fenómeno lê-se como inútil.
///
/// ⚠️ **A barra é a do oráculo na MESMA peça** (`sphere_uv_96x144`, que está no
/// corpus), não uma expectativa.
///
/// ⭐ **Pista por perseguir, ainda não medida:** dos 16 patches desta esfera, **8 são
/// triângulos e 3 são pentágonos** — onze passam pelo LEQUE, cujo sector é um
/// *papagaio* no domínio, não um rectângulo. Uma grade construída dentro de um
/// papagaio nasce enviesada. ⛔ *É hipótese, não medição* — o `skew_prov` diz que a
/// `grade` está tão torta quanto o `raio`, e isso ainda não foi reconciliado.
#[test]
#[ignore = "VERMELHO -- a reproducao mais barata do enviesamento; ver o doc"]
fn a_plain_sphere_is_as_square_as_the_oracles() {
    let reference = ph2d_mesh::shapes::uv_sphere(96, 144, 1.0);
    let mut work = reference.clone();
    work.triangulate();
    ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
    work.triangulate();
    let target = ph2d_quadflow::edge_for_detail_with(
        &reference,
        0.55,
        ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
    );
    let dual = ph2d_crossfield::Dual::build(&work);
    let (field, _) = ph2d_crossfield::solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&work, &dual, &field);
    let spec = layout.to_layout(target).expect("o layout fecha");
    let (quant, _) = ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
        .expect("quantiza");
    let (out, r) = ph2d_quadfill::fill(
        &work,
        &reference,
        &layout,
        &quant,
        ph2d_quadfill::SMOOTHING_ROUNDS,
    )
    .expect("monta");
    let s = ph2d_quadfill::quad_shape(&out);
    eprintln!(
        "[f5] esfera LISA: {} quads · aspecto p50 {:.2} · enviesamento p50 {:.0}° · {} faces > 60°",
        r.quads, s.aspect_p50, s.skew_p50, s.skew_over_60
    );
    // ⚠️ **A barra é a do oráculo na mesma peça** (`6°`), com folga de arredondamento.
    // ⛔ Ela não se afrouxa: uma esfera lisa é o caso em que não há desculpa.
    assert!(
        s.skew_p50 <= 8.0,
        "enviesamento mediano {:.0}° numa esfera LISA -- o oraculo entrega 6°",
        s.skew_p50
    );
    assert_eq!(
        s.skew_over_60, 0,
        "{} faces com um canto pior que 60° numa esfera LISA -- o oraculo entrega 0",
        s.skew_over_60
    );
}
