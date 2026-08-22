//! **AS SONDAS DO CAMPO E DO RELEVO** — irmãs de
//! [`super::retopo_density_probes`].
//!
//! ⭐⭐ **O corte foi forçado pela HR-18 (612 contra 600), e é de ASSUNTO:** lá as
//! sondas da **densidade** (até onde o slider vai, quanto custa a quantização);
//! aqui as do **campo** — se a peça sobrevive à cadeia, e se a grade obedece ao
//! relevo.
//!
//! ⚠️ **A régua que as une é a [`ph2d_quadfill::follows_relief`]**, e o número que
//! a torna legível é `22,5°`: a média de um ângulo uniforme em `[0°, 45°]`, ou
//! seja **uma grade que ignora o relevo por completo**. Um resultado perto disso
//! não é *"um pouco desalinhado"*: é *"não olhou"*.

/// ⭐⭐ **SONDA — A ORELHA SOBREVIVE?** (as fotos de 2026-08-22)
///
/// ⛔ **A terceira foto mostra a orelha ACHATADA**, e nenhuma régua do relatório a
/// veria: 100 % de quads, casca fechada, contagem de irregulares boa. *Uma peça
/// pode passar em toda asserção e ter perdido a forma.*
///
/// ⭐ **A régua nova é a [`ph2d_quadfill::detail_lost`], e o SENTIDO é o ponto
/// inteiro**: `referência → saída`. O contrário é tautológico — a última coisa que
/// a montagem faz é pousar cada ponto na referência.
///
/// ⭐⭐ **E a coluna que decide é a `F1`**: a mesma régua aplicada à malha
/// remalhada, **antes** de a cadeia começar. Se a orelha já morreu ali, nenhum
/// conserto a jusante a traz de volta — e a cura é outra (o `α` do F1), não o
/// traçado nem a montagem.
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   does_the_ear_survive_the_chain -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- a orelha sobrevive a' cadeia?"]
fn does_the_ear_survive_the_chain() {
    let reference = crate::sculpt3d::fixtures::eared_sphere();
    println!(
        "── orelha: {} vertices, {} faces ──",
        reference.vert_count(),
        reference.faces().len()
    );
    for alpha_mult in [1.0f32, 0.5] {
        let alpha = ph2d_remesh_iso::ALPHA * alpha_mult;
        let mut work = reference.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, alpha);
        work.triangulate();
        // ⭐⭐ **O QUE O F1 JÁ CUSTOU**, antes de a cadeia começar.
        let (f1_p95, f1_max) = ph2d_quadfill::detail_lost(&reference, &work);
        println!(
            "  α {:.4} | work {:>7} tris | ⭐ o F1 sozinho ja' perdeu p95 {:.3} % max {:.3} % \
             da diagonal",
            alpha,
            work.faces().len(),
            100.0 * f1_p95,
            100.0 * f1_max
        );
        let dual = ph2d_crossfield::Dual::build(&work);
        let (field, _) = ph2d_crossfield::solve_miq(&dual);
        let layout = ph2d_trace::trace_patches(&work, &dual, &field);
        for detail in [1.0f32] {
            let target = ph2d_quadflow::edge_for_detail_with(
                &reference,
                detail,
                ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
            );
            let Ok(spec) = layout.to_layout(target) else {
                println!("    d={detail:.2} | o layout RECUSOU");
                continue;
            };
            let Ok((quant, _)) =
                ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
            else {
                println!("    d={detail:.2} | a quantizacao RECUSOU");
                continue;
            };
            match ph2d_quadfill::fill(
                &work,
                &reference,
                &layout,
                &quant,
                ph2d_quadfill::SMOOTHING_ROUNDS,
            ) {
                #[allow(clippy::cast_precision_loss)]
                Ok((out, r)) => {
                    let (p95, mx) = ph2d_quadfill::detail_lost(&reference, &out);
                    println!(
                        "    d={detail:.2} alvo {target:.4} | {:>6} quads {:>4} irreg {:>3} bordo \
                         | dobras {:>4} ({:.2} %) | ⭐ PERDEU p95 {:.3} % max {:.3} % | patches {}",
                        r.quads,
                        r.irregular,
                        r.boundary_edges,
                        r.folded_local,
                        100.0 * r.folded_local as f64 / r.quads.max(1) as f64,
                        100.0 * p95,
                        100.0 * mx,
                        r.patches,
                    );
                }
                Err(e) => println!("    d={detail:.2} | a montagem RECUSOU {e:?}"),
            }
        }
    }
}

/// ⭐⭐ **SONDA — QUANTO PESO O ALINHAMENTO AGUENTA?**
///
/// ⛔ **A primeira tentativa pôs `1,0` e a cadeia inteira parou de fechar** —
/// patches de **39, 98 e 127 lados**, quantização a recusar em toda fixtura. *Um
/// peso de energia escolhido sem tabela é um palpite com aparência de teoria*
/// (CLAUDE.md §0.0).
///
/// As três colunas que decidem:
///
/// | coluna | o que ela diz |
/// |---|---|
/// | ⭐ **RELEVO** | o desvio, em graus, entre a grade e a direção principal. `22,5°` é o de uma grade **aleatória** |
/// | **patches / maior valência** | se o campo continua a produzir um layout que fecha — um patch de 39 lados não é *"mais detalhe"*, é traçado partido |
/// | **dobras / bordo** | se a saída ainda é uma malha |
///
/// ```text
/// \
///   cargo test -p ph2d-host-desktop --release --bins \
///   how_much_alignment_can_the_field_take -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda -- o peso do alinhamento do campo"]
fn how_much_alignment_can_the_field_take() {
    for (name, which) in [("com CRISTAS", 2u8), ("ORELHA", 3)] {
        let reference = match which {
            2 => crate::sculpt3d::fixtures::ridged_sphere(),
            3 => crate::sculpt3d::fixtures::eared_sphere(),
            _ => crate::sculpt3d::fixtures::wrinkled_sphere(),
        };
        let target = ph2d_quadflow::edge_for_detail_with(
            &reference,
            0.5,
            ph2d_quadflow::GLOBAL_FLOOR_IN_INPUT_EDGES,
        );
        let mut work = reference.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();
        let dual = ph2d_crossfield::Dual::build(&work);
        println!(
            "── {name} (alvo {target:.4}, work {} tris) ──",
            work.faces().len()
        );
        for w in [0.0f32, 0.01, 0.05, 0.2, 1.0] {
            let (field, _) =
                ph2d_crossfield::solve_miq_aligned(&dual, ph2d_crossfield::Rounding::default(), w);
            let layout = ph2d_trace::trace_patches(&work, &dual, &field);
            let worst = layout.side_arcs.iter().map(Vec::len).max().unwrap_or(0);
            let Ok(spec) = layout.to_layout(target) else {
                println!(
                    "  peso {w:<5} | patches {:<4} maior valencia {worst:<4} | o layout RECUSOU",
                    layout.side_arcs.len()
                );
                continue;
            };
            let Ok((quant, _)) =
                ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
            else {
                println!(
                    "  peso {w:<5} | patches {:<4} maior valencia {worst:<4} | a quantizacao RECUSOU",
                    layout.side_arcs.len()
                );
                continue;
            };
            match ph2d_quadfill::fill(
                &work,
                &reference,
                &layout,
                &quant,
                ph2d_quadfill::SMOOTHING_ROUNDS,
            ) {
                #[allow(clippy::cast_precision_loss)]
                Ok((out, r)) => {
                    let (deg, conf) = ph2d_quadfill::follows_relief(&reference, &out);
                    println!(
                        "  peso {w:<5} | patches {:<4} maior valencia {worst:<4} | {:>6} quads \
                         {:>4} irreg {:>3} bordo dobras {:>4} | ⭐RELEVO {deg:>5.1}° (conf {conf:.2}) \
                         | med {:.2}x max {:.2}x",
                        layout.side_arcs.len(),
                        r.quads,
                        r.irregular,
                        r.boundary_edges,
                        r.folded_local,
                        r.edge_median / target,
                        r.edge_max / target,
                    );
                }
                Err(e) => println!(
                    "  peso {w:<5} | patches {:<4} maior valencia {worst:<4} | a montagem RECUSOU {e:?}",
                    layout.side_arcs.len()
                ),
            }
        }
    }
}
