//! **§5.0 da W4 — MEDIR se a composição já dá «nuvens que andam sozinhas»** (plano 24).
//!
//! ⚠️ **Esta sonda corre como gate e fica versionada**, porque a resposta dela é o que decide se a
//! wave existe. *Uma medição que decide uma wave e não é gateada envelhece com o produto.*

use ph2d_core::Vec2;
use ph2d_ecs::{ScrollFactor, SimWorld, Transform};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};

/// ⚠️ **O instante em que a deriva da W4 é INERTE.** Toda a bancada das W1–W3 corre aqui, e é isso
/// que mantém aquelas leis medidas SOZINHAS — `velocidade × 0` é zero seja qual for a velocidade.
const PARADO: f64 = 0.0;

use super::drive_parallax;

/// ⭐⭐⭐ **A resposta é NÃO, e o mecanismo é o LEDGER: os dois motores brigam pelo `Transform`.**
///
/// Um `Tween` de pose e um `ScrollFactor` no mesmo objecto são **dois condutores** com chaves
/// diferentes no ledger (`TweenPose` · `ParallaxPose`), e os dois escrevem o MESMO campo ⇒ quem
/// corre por último ganha, e o que o outro escreveu **entra no `authored` dele** como se tivesse
/// sido o artista a arrastar.
///
/// ⛔ E mesmo sem isso o tween não exprime a lei: ele vai de `A` a `B` num tempo, e um fundo que
/// deriva para sempre não tem `B`. *A deriva é uma VELOCIDADE, e um tween é um DESTINO.*
#[test]
fn a_composicao_nao_da_a_deriva_e_os_dois_motores_brigam() {
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        // ⚠️ **A cena não monta um `Tweens` de verdade, e é de propósito:** o que se mede é o
        // LEDGER com dois condutores sobre o mesmo campo, e ele é o mesmo seja qual for o segundo
        // motor. *Montar o tween real traria uma dependência nova a esta crate para não mudar uma
        // linha da medição.*
        .spawn((ScrollFactor { k: [0.5, 1.0] }, Transform::default()))
        .id();
    let mut drive = PreviewDrive::default();

    // A paralaxe conduz.
    drive_parallax(
        &mut sim,
        Some(([400.0, 0.0], [10.0, 10.0])),
        PARADO,
        &mut drive,
    );
    let depois_da_paralaxe = sim.world().get::<Transform>(e).expect("pose").translation;
    assert!(
        (depois_da_paralaxe.x - 200.0).abs() < 1e-3,
        "x = {} contra 200",
        depois_da_paralaxe.x
    );

    // ⚠️ Agora OUTRO motor escreve o mesmo campo — é o que um tween de pose faz todo quadro.
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        t.translation = Vec2::new(55.0, 0.0);
    }
    drive.driven(
        e,
        Driven::TweenPose(Transform {
            translation: depois_da_paralaxe,
            ..Transform::default()
        }),
        Driven::TweenPose(Transform {
            translation: Vec2::new(55.0, 0.0),
            ..Transform::default()
        }),
    );

    // ⭐ E o quadro seguinte da paralaxe lê a escrita do outro como se fosse do ARTISTA.
    drive_parallax(
        &mut sim,
        Some(([400.0, 0.0], [10.0, 10.0])),
        PARADO,
        &mut drive,
    );
    let Some(Driven::ParallaxPose(memo)) = drive.authored(e.to_bits(), Driver::ParallaxPose) else {
        panic!("a paralaxe tinha de continuar a conduzir");
    };
    assert!(
        (memo.translation.x - (55.0 - 200.0)).abs() < 1e-3,
        "o autorado da paralaxe leu {} — ele devia ter apanhado a escrita do outro motor como \
         arrasto, que e' exactamente o defeito que esta medicao existe para nomear",
        memo.translation.x
    );
    // ⇒ **a deriva tem de ser da MESMA lei da paralaxe**, somada ao deslocamento antes de o
    //   ledger o ver — nunca um segundo condutor sobre o mesmo campo.
}
