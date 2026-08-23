//! **OS GATES DA ORIGEM DO INTERIOR** — ver [`ph2d_quadfill::Interior`].
//!
//! ⚠️ **Os dois lados, de propósito** (`reference_topic_gate_discipline`): um gate
//! que só provasse a **ausência** (o caminho que shipa é o antigo) ficaria verde
//! sobre uma canalização morta; um que só provasse a **presença** ficaria verde
//! sobre uma que já estivesse a mexer no produto sem ninguém ter decidido.
//!
//! ⛔ **O contexto (2026-08-23):** o alinhamento do interior ao campo foi
//! construído, medido e **desligado** — ele não move o enviesamento. A canalização
//! fica porque é o que faltava (o campo agora **chega** ao F5), e código que nenhum
//! teste exercita apodrece antes de o bloqueio a montante sair da frente.
//!
//! ⚠️⚠️ **A explicação que estava escrita aqui foi REFUTADA no mesmo dia, pelo
//! controlo.** Ela dizia: *«a holonomia dentro dos patches vale `29°–44°` ⇒ o campo
//! lá dentro não é combável ⇒ a dívida é do F3».* Medindo a decomposição do
//! **oráculo** com o mesmo código, ele dá `18,6°`/`38,4°` e uma distribuição
//! indistinguível da nossa (p50 `0,470°` dele contra `0,479°` nossa). *Um predicado
//! que reprova a testemunha de controlo mede a discretização, não o defeito.*
//! A holonomia fica como régua; a acusação saiu.

use ph2d_crossfield::{Dual, solve_miq};
use ph2d_mesh::{Mesh, shapes};
use ph2d_quadfill::{Interior, SMOOTHING_ROUNDS, SQUARE_ROUNDS, fill, fill_with};

/// Corre a cadeia até ao layout + quantização, devolvendo o que o F5 precisa.
fn upto_fill(
    reference: &Mesh,
    target: f32,
) -> Option<(Mesh, ph2d_trace::PatchLayout, ph2d_quantize::Quantization)> {
    let mut work = reference.clone();
    ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
    work.triangulate();
    let dual = Dual::build(&work);
    let (field, _) = solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&work, &dual, &field);
    let spec = layout.to_layout(target).ok()?;
    let (quant, _) =
        ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512)).ok()?;
    Some((work, layout, quant))
}

fn fixture() -> Mesh {
    let mut m = shapes::uv_sphere(24, 36, 1.0);
    m.triangulate();
    m
}

/// ⭐⭐ **AUSÊNCIA — o que shipa é, byte a byte, o caminho da FRONTEIRA.**
///
/// ⛔ **Sem este lado, ligar a cura por acidente passaria despercebido.** As duas
/// origens dão malhas com a **mesma topologia** e geometrias diferentes por margens
/// pequenas; nenhuma outra régua desta crate as distingue.
#[test]
fn the_shipped_interior_is_the_boundary_one() {
    let reference = fixture();
    let (work, layout, quant) = upto_fill(&reference, 0.25).expect("a cadeia fecha");
    let (a, _) = fill(&work, &reference, &layout, &quant, SMOOTHING_ROUNDS).expect("monta");
    let (b, _) = fill_with(
        &work,
        &reference,
        &layout,
        &quant,
        SMOOTHING_ROUNDS,
        SQUARE_ROUNDS,
        Interior::FromBoundary,
    )
    .expect("monta");
    assert_eq!(
        a.positions(),
        b.positions(),
        "o caminho que shipa deixou de ser o da fronteira -- foi decisao ou acidente? \
         leia o doc de `ph2d_quadfill::INTERIOR` antes de mexer aqui"
    );
}

/// ⭐⭐ **PRESENÇA — com o campo ligado o interior MUDA.**
///
/// ⛔ **Sem este lado, o [`ph2d_quadfill::aligned`] podia ser um `if` morto** e o
/// gate irmão — que é de inércia — continuaria feliz.
///
/// ⚠️ **A fronteira e a topologia têm de ficar IGUAIS**: o campo só toca o interior
/// do achatamento, e um layout partilha cada arco com o patch vizinho. *Se a
/// contagem de quads mudasse, a costura teria mudado — e aí a malha rasga.*
#[test]
fn the_field_changes_the_interior_and_nothing_else() {
    let reference = fixture();
    let (work, layout, quant) = upto_fill(&reference, 0.25).expect("a cadeia fecha");
    let mut out = Vec::new();
    for interior in [Interior::FromBoundary, Interior::AlignedToField] {
        out.push(
            fill_with(
                &work,
                &reference,
                &layout,
                &quant,
                SMOOTHING_ROUNDS,
                SQUARE_ROUNDS,
                interior,
            )
            .expect("monta"),
        );
    }
    let (b, f) = (&out[0].0, &out[1].0);
    assert_eq!(
        (b.faces().len(), b.vert_count()),
        (f.faces().len(), f.vert_count()),
        "o campo mudou a TOPOLOGIA -- ele so' pode tocar onde os pontos ficam"
    );
    assert_ne!(
        b.positions(),
        f.positions(),
        "ligar o campo nao mudou um unico ponto -- ou e' um `if` morto, ou o layout \
         chegou sem `face_dir` (o `trace_patches` e' quem o preenche)"
    );
    // ⚠️ **A holonomia é o número que EXPLICA a tabela da rejeição.** Ela mede o
    // desacordo do campo penteado dentro de um patch; grande = há singularidade lá
    // dentro, e pedir ao interior que siga um campo inconsistente não podia
    // funcionar. *Sem esta asserção, o mecanismo vira folclore no doc.*
    assert!(
        out[1].1.holonomy >= 0.0,
        "a holonomia nao foi medida no caminho alinhado"
    );
}
