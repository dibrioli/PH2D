//! ⭐⭐⭐⭐ **A ATRIBUIÇÃO DO `SIGSEGV`: trabalho na placa a SOBREVIVER AO PROCESSO.**
//!
//! # ⚠️ Porque isto são SONDAS e não gates
//!
//! A metade que reproduz **estoura de propósito** — e um processo que morre com `signal 11` não
//! pode ser um teste verde. ⇒ as duas correm à mão, e o veredito é o **código de saída do
//! processo**, que é exactamente a grandeza em causa. *Um gate que precisasse de sobreviver ao
//! defeito mediria outra coisa.*
//!
//! ⚠️⚠️ **O `nextest` dá um processo por teste**, logo é ele que as separa; com `cargo test` as duas
//! partilham o processo e a primeira leva a segunda.
//!
//! # ⭐ A atribuição, em duas corridas
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo nextest run -p ph2d-app-field3d \
//!   --run-ignored all -E 'test(gpu_frame::testes::diag_)' --no-capture
//! ```
//!
//! | sonda | o que ela faz | o que se lê |
//! |---|---|---|
//! | [`diag_o_quadro_em_voo_mata_o_processo`] | desenha e **sai** | `ok` + `NVVM compilation failed: 3` + `SIGSEGV` |
//! | [`diag_esperar_pelo_quadro_cura`] | desenha, **espera** e sai | o veredito está na corrida |
//!
//! ⇒ *se o discriminador for a ESPERA e não a placa*, a lei do [`super::para_o_quadro`] é a certa e
//! a dívida de produto que sobra é **drenar os quadros em voo antes de o processo sair**.
//!
//! ⛔⛔ **E a 1.ª redacção destas duas sondas era VÁCUO:** ela drenava FORA do `armed_with`, onde o
//! módulo já largou o `Receiver` — leu `esperei por 0 quadro(s)`, estourou nas duas metades, e eu
//! quase a li como *«esperar não cura»*. ⇒ **o CONTROLO é uma asserção**, e uma corrida que não
//! deixe quadro nenhum em voo reprova **alto** em vez de mentir.

use std::sync::atomic::{AtomicBool, Ordering};

/// ⚠️ **Um átomo e não um `thread_local`:** quem lê isto é a thread que DESENHA, e ela é outra.
static AO_QUADRO: AtomicBool = AtomicBool::new(false);

/// A resposta que a [`super::para_o_quadro`] lê sob `cfg(test)`.
pub(crate) fn a_placa_vai_ao_quadro() -> bool {
    AO_QUADRO.load(Ordering::Relaxed)
}

/// ⛔ **Só as sondas desta atribuição a chamam** — ver o doc do módulo.
fn arma_a_placa_para_o_quadro() {
    AO_QUADRO.store(true, Ordering::Relaxed);
}

/// Desenha um quadro pelo caminho do produto, como os testes de `view_menu` fazem — e, se lhe
/// pedirem, **espera** por ele antes de o módulo ser desarmado.
///
/// ⛔⛔ **A espera tem de correr DENTRO do `armed_with`:** ao desarmar, o módulo larga o
/// `Receiver`, o `inflight` fica `None`, e uma espera lá fora é um no-op com cara de cura.
///
/// ⇒ devolve **quantos ficaram em voo**, lido ANTES da espera (o dreno consome-os, logo depois
/// dele a contagem é sempre `0` e o número não distinguiria nada).
fn desenha_um_quadro(esperar: bool) -> usize {
    let prazo = std::time::Duration::from_secs(10);
    let doc = crate::scene::lasso_tests::two_balls();
    crate::scene::lasso_tests::armed_with(&doc, |_| {
        let mut text = ph2d_text::TextSystem::without_system_fonts();
        let mut scene = ph2d_vector::VectorScene::new();
        crate::smoke::draw(
            crate::scene::lasso_tests::AREA,
            ph2d_tokens::Theme::default(),
            &mut text,
            &mut scene,
        );
        crate::smoke::with_smoke(|s| {
            let em_voo = s.quantos_em_voo();
            if esperar {
                let chegaram = s.espera_pelos_quadros_em_voo(prazo);
                eprintln!("[desanexado] em voo {em_voo} · esperei e chegaram {chegaram}");
            } else {
                eprintln!("[desanexado] em voo {em_voo} · NÃO se espera por eles");
            }
            em_voo
        })
        .unwrap_or(0)
    })
}

/// ⛔⛔⛔ **ESTA SONDA MATA O PROCESSO, E ISSO É O RELATÓRIO.**
#[test]
#[ignore = "sonda de atribuição: ela MATA o processo de propósito"]
fn diag_o_quadro_em_voo_mata_o_processo() {
    arma_a_placa_para_o_quadro();
    if crate::gpu_frame::shared().is_none() {
        eprintln!("[desanexado] sem adaptador — esta sonda não afirma nada aqui");
        return;
    }
    let em_voo = desenha_um_quadro(false);
    assert!(
        em_voo > 0,
        "CONTROLO: nenhum quadro ficou em voo — esta sonda não estaria a medir nada"
    );
}

/// ⭐⭐⭐ **A METADE QUE ESPERA — e é ela que faz a atribuição valer.**
///
/// O MESMO desenho, a MESMA placa, a MESMA thread desanexada, e uma espera antes de desarmar.
#[test]
#[ignore = "sonda de atribuição: o lado que espera"]
fn diag_esperar_pelo_quadro_cura() {
    arma_a_placa_para_o_quadro();
    if crate::gpu_frame::shared().is_none() {
        eprintln!("[desanexado] sem adaptador — esta sonda não afirma nada aqui");
        return;
    }
    let em_voo = desenha_um_quadro(true);
    assert!(
        em_voo > 0,
        "CONTROLO: nenhum quadro ficou em voo — a espera seria um no-op, e esta metade leria como \
         «esperar não cura» sobre uma sonda que não esperou por nada"
    );
}
