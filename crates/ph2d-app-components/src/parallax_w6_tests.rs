//! **A W6 — e ela é UM GATE, porque a §5.0 refutou as DUAS metades da premissa** (plano 24).
//!
//! O plano encomendava *«uma porta só para a pose que a vista conduz, com dois leitores»*, com o
//! gate *«`UiCanvas` e um objecto com `k = 0` produzem a MESMA pose, ao bit»*.
//!
//! # ⛔⛔ Metade 1 — «a POSE» não é partilhável; a TRANSLAÇÃO é
//!
//! Medido (o corpo deste ficheiro): a translação dos dois é **idêntica ao bit** e a escala **nunca**
//! o é. A do canvas responde *«quantos metros de mundo cabem nesta janela?»* — ela existe para o HUD
//! ser legível em qualquer resolução —, e a da paralaxe responde *«a que PROFUNDIDADE está esta
//! camada?»* (a W5), não existindo de todo com `dolly = 0`. *Duas grandezas com o mesmo nome.*
//!
//! # ⛔⛔ Metade 2 — não há LEI duplicada para unificar
//!
//! A translação do canvas é [`ph2d_hud::place`], e ali ela é **`translate: view.center`** — uma
//! ATRIBUIÇÃO, não uma lei. A da paralaxe é `autorada + centro·(1 − k)`. Elas coincidem em
//! `k = 0` com a pose autorada em zero, e isso é um **FACTO sobre as duas leis**, não uma cópia de
//! uma delas. ⇒ chamar a lei da paralaxe de dentro do canvas acrescentaria uma dependência para
//! exprimir `centro`: **cerimónia, e não unificação**.
//!
//! # ⭐ O que a wave entrega, então
//!
//! **O gate que ATA as duas**, que é o que a unificação ia comprar e é tudo o que ela ia comprar: no
//! dia em que qualquer uma das duas leis mudar, a relação é afirmada em voz alta. *Uma relação
//! medida e gateada vale o mesmo que uma porta partilhada, e não paga a dependência.*

use ph2d_ecs::{Fit, ScrollFactor, SimWorld, Transform, UiCanvas};
use ph2d_preview_drive::PreviewDrive;

use super::super::hud_bridge::{View, drive_canvases};
use super::drive_parallax;
use super::tests::{PARADO, SEM_LIMITE};

/// ⭐⭐⭐ **A premissa do plano está METADE certa, e a metade que falha é a que decide o refactor.**
///
/// | grandeza | `UiCanvas` | `ScrollFactor { k: 0 }` | iguais? |
/// |---|---|---|---|
/// | **translação** | o centro da vista | `autorada + centro·1` | ⭐ **sim, ao bit** (autorada `0`) |
/// | **escala** | `vista / referência` | intocada | ⛔ **não, e nunca** |
///
/// ⭐ **A translação é a MESMA LEI**, e é isso que a W6 pode partilhar: os dois dizem *«esta coisa
/// está colada à vista»*.
///
/// ⛔⛔ **A escala NÃO é partilhável, e não é um detalhe de implementação:** a do canvas responde
/// *«quantos metros de mundo cabem nesta janela?»* — ela existe para o HUD ser legível em qualquer
/// resolução, e o `Fit` escolhe entre confinar e esticar. A da paralaxe responde *«a que
/// PROFUNDIDADE está esta camada?»* (a W5), e com `dolly = 0` ela **não existe de todo**. *Duas
/// grandezas com o mesmo nome e com perguntas diferentes.*
///
/// ⇒ **a porta partilhada é da TRANSLAÇÃO e nunca «da pose»**, e o cabeçalho do plano diz *«a pose»*.
#[test]
fn a_premissa_da_w6_esta_metade_certa_e_a_escala_nao_e_partilhavel() {
    // ⚠️⚠️ **A vista NÃO pode ter o tamanho da referência**, e a 1.ª redacção tinha: ali a escala do
    // canvas é `1,0` e a da paralaxe também, os dois lados coincidem por ACIDENTE e o gate lê-se
    // como *«as duas grandezas são a mesma»*. *Uma fixtura no ponto neutro de uma das grandezas não
    // a distingue de nenhuma outra* — a forma que esta sessão já pagou quatro vezes.
    let vista = View {
        center: [400.0, -250.0],
        half: [32.0, 18.0],
    };

    // O canvas.
    let mut a = SimWorld::default();
    let ea = a
        .world_mut()
        .spawn((
            UiCanvas {
                ref_w: 32.0,
                ref_h: 18.0,
                fit: Fit::Keep,
            },
            Transform::default(),
        ))
        .id();
    let mut da = PreviewDrive::default();
    drive_canvases(&mut a, Some(vista), &mut da);
    let pa = *a.world().get::<Transform>(ea).expect("pose");

    // O objecto de `k = 0`, autorado na origem.
    let mut b = SimWorld::default();
    let eb = b
        .world_mut()
        .spawn((ScrollFactor { k: [0.0, 0.0] }, Transform::default()))
        .id();
    let mut db = PreviewDrive::default();
    drive_parallax(&mut b, Some((vista.center, vista.half)), PARADO, &mut db);
    let pb = *b.world().get::<Transform>(eb).expect("pose");

    assert_eq!(
        pa.translation, pb.translation,
        "a TRANSLACAO dos dois divergiu: a metade partilhavel da premissa caiu, e com ela a W6"
    );
    assert_ne!(
        pa.scale, pb.scale,
        "as ESCALAS coincidiram: ou o canvas deixou de enquadrar, ou a paralaxe passou a escalar \
         sem dolly — e a premissa de que elas sao grandezas DIFERENTES precisa de ser re-medida"
    );
    let _ = SEM_LIMITE;
}
