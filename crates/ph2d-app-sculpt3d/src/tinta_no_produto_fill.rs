//! ⭐⭐ **O `FILL` PELO CAMINHO DO PRODUTO** — irmão (`#[path]`) do
//! [`super`], com o arnês dele.
//!
//! ⚠️ Os gates PUROS da lei vivem na crate `ph2d-sculpt3d` (o valor de cada
//! amostra, a máscara interpolada, a recusa) e os da entrada de desfazer no
//! `history_tinta_fina_tests.rs`. O que ESTE afirma é o elo que nenhum deles
//! vê: a cena preenche os DOIS canais, grava UMA entrada, e a tecla desfá-la
//! inteira — com o plano de tinta fina armado, que é o caso que o dono pinta.

use super::{amostras, cena_52, gesto, tecla};
use crate::preenche::Preenchido;

const VERMELHO: [f32; 3] = [1.0, 0.0, 0.0];

fn cores(s: &crate::Sculpt3dScene) -> Option<Vec<[f32; 3]>> {
    s.mesh().colors().map(<[[f32; 3]]>::to_vec)
}

/// ⭐⭐⭐ **GATE — O `Fill` PINTA A PEÇA INTEIRA NOS DOIS CANAIS, E UM `Ctrl+Z`
/// DEVOLVE OS DOIS, AO BIT.**
///
/// ⚠️ **O CONTROLO é o plano estar ARMADO antes do gesto:** sem ele o gate
/// passaria sobre uma cena em que só a cor por vértice existe — e o `Fill`
/// que esquecesse o plano ficaria verde.
#[test]
#[ignore = "precisa de adaptador"]
fn o_fill_pinta_os_dois_canais_e_o_ctrl_z_devolve_os_dois() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    let virgem = amostras(&s);
    let cor_virgem = cores(&s);
    assert!(
        !virgem.is_empty(),
        "o controlo: o plano de tinta fina esta' armado"
    );
    assert_eq!(s.brush.color, VERMELHO, "a cena entrega o pincel vermelho");

    assert_eq!(s.fill_color(), Preenchido::Feito { fina: true });
    s.sync_mesh(&gpu.device, &gpu.queue);
    let depois = amostras(&s);
    assert!(
        depois.iter().all(|c| *c == VERMELHO),
        "sem mascara, toda amostra do plano tem de sair exactamente a cor do pincel"
    );
    assert!(
        cores(&s).is_some_and(|c| c.iter().all(|c| *c == VERMELHO)),
        "a cor por vertice tambem tem de ser a do pincel"
    );

    assert!(tecla(&mut s, false), "o Ctrl+Z tem de ser consumido");
    s.sync_mesh(&gpu.device, &gpu.queue);
    assert_eq!(
        amostras(&s),
        virgem,
        "o Ctrl+Z nao devolveu o plano de antes, ao bit"
    );
    assert_eq!(
        cores(&s),
        cor_virgem,
        "o Ctrl+Z nao devolveu a cor por vertice de antes"
    );

    assert!(tecla(&mut s, true), "o Ctrl+Shift+Z tem de ser consumido");
    s.sync_mesh(&gpu.device, &gpu.queue);
    assert_eq!(
        amostras(&s),
        depois,
        "o refazer nao devolveu o preenchido, ao bit"
    );
}

/// ⚠️ **GATE — a meio de um traço o `Fill` RECUSA**, e o traço acaba com o
/// plano intacto: o plano está emprestado ao gesto, e preencher por baixo dele
/// deixaria o traço a devolver um plano antigo por cima do preenchido.
#[test]
#[ignore = "precisa de adaptador"]
fn a_meio_de_um_traco_o_fill_recusa() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    let mut host = super::HostDeTeste {
        ponteiro: (420.0, 350.0),
    };
    assert!(
        crate::input_down::pointer_down(&mut host, &mut s, winit::event::MouseButton::Left),
        "o pen-down tem de ser da cena"
    );
    assert!(
        s.stroke.tinta_fina.is_some(),
        "o controlo: o plano esta' emprestado ao traco"
    );
    assert_eq!(s.fill_color(), Preenchido::TracoAberto);
    crate::input::pointer_up(&mut s);
    assert!(
        amostras(&s).iter().any(|c| *c != VERMELHO),
        "o Fill recusado nao pode ter pintado a peca inteira"
    );
}

/// ⭐⭐ **GATE — com a peça JÁ pintada, o `Ctrl+Z` devolve a cor de antes do
/// `Fill`, e não a peça por pintar.**
///
/// ⛔ **Ele nasceu de uma mutação SOBREVIVENTE:** o gate irmão parte de uma
/// peça que nunca teve cor, logo só o ramo *«não havia cor, e desfazer TIRA o
/// plano»* era exercido — apagar o ramo que DEVOLVE a cor de antes passava
/// verde. Aqui um traço real pinta primeiro, e o CONTROLO é a cor existir antes
/// do gesto.
#[test]
#[ignore = "precisa de adaptador"]
fn com_a_peca_ja_pintada_o_ctrl_z_devolve_a_cor_de_antes_do_fill() {
    let gpu = gpu_or_skip!();
    let mut s = cena_52(&gpu.device);
    s.sync_mesh(&gpu.device, &gpu.queue);
    assert!(
        gesto(&mut s, 380.0, 470.0, 10),
        "o pen-down nao foi da cena"
    );
    s.sync_mesh(&gpu.device, &gpu.queue);
    let pintada = cores(&s);
    assert!(
        pintada
            .as_ref()
            .is_some_and(|c| c.iter().any(|c| *c != VERMELHO)),
        "o controlo: a peca tem cor por vertice, e nao toda vermelha, antes do Fill"
    );
    let plano = amostras(&s);

    s.brush.color = [0.1, 0.8, 0.2];
    assert_eq!(s.fill_color(), Preenchido::Feito { fina: true });
    s.sync_mesh(&gpu.device, &gpu.queue);
    assert_ne!(cores(&s), pintada, "o Fill tem de ter mudado a cor");

    assert!(tecla(&mut s, false), "o Ctrl+Z tem de ser consumido");
    s.sync_mesh(&gpu.device, &gpu.queue);
    assert_eq!(
        cores(&s),
        pintada,
        "o Ctrl+Z nao devolveu a cor por vertice de antes"
    );
    assert_eq!(
        amostras(&s),
        plano,
        "o Ctrl+Z nao devolveu o plano de antes"
    );
}
