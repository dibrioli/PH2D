//! A compressão da adormecida espera o silêncio, acontece na mesma, e acordar antes cancela-a: ao bit em todos.

use super::super::adormecida_tests::{borracha_viva, volta_e_apaga};
use super::super::agua_sobre_oleo_tests::com_oleo;
use super::super::borracha_de_fabrica_tests::{apaga, diferentes};
use super::{ENCOLHE_NO_SILENCIO, ULTIMO_FIO};
use crate::tool::paint::PaintMedia;
use std::time::{Duration, Instant};

type Horas = (Option<Instant>, Option<Instant>);

/// Adormece a tinta do óleo e lança o fio dela com a chave `silencio`. Devolve a hora do lançamento e as horas do fio.
fn lanca(
    t: &mut crate::tool::PainterTool,
    silencio: bool,
) -> (Instant, impl Fn() -> Horas + use<>) {
    t.set_paint_media(PaintMedia::Digital);
    ENCOLHE_NO_SILENCIO.set(silencio);
    let t0 = Instant::now();
    t.paint.wetpaint.adormecidas[0].encolhe_num_fio();
    ENCOLHE_NO_SILENCIO.set(true);
    let h = ULTIMO_FIO.take().expect("o fio nasceu");
    (t0, move || *h.lock().expect("as horas"))
}

fn espera_o_fim(horas: &impl Fn() -> Horas) -> Horas {
    let t = Instant::now();
    while horas().1.is_none() && t.elapsed() < Duration::from_secs(60) {
        std::thread::sleep(Duration::from_millis(5));
    }
    horas()
}

/// ⭐ Com a chave: o fio não começa antes de 300 ms de silêncio, comprime na mesma (controlo), e acordar dá o mesmo bit.
#[test]
fn a_compressao_espera_o_silencio_e_acontece() {
    let vivo = borracha_viva();
    let mut t = com_oleo();
    let (t0, horas) = lanca(&mut t, true);
    let (ini, fim) = espera_o_fim(&horas);
    let ini = ini.expect("comprimiu");
    assert!(fim.is_some(), "acabou");
    let atraso = ini.duration_since(t0);
    assert!(
        atraso >= Duration::from_millis(290),
        "esperou o silêncio: {atraso:?}"
    );
    let a = &t.paint.wetpaint.adormecidas[0];
    assert!(
        a.sess.lock().expect("trinco").dormente.is_some(),
        "está comprimida"
    );
    assert_eq!(diferentes(&vivo, &volta_e_apaga(&mut t)), 0);
}

/// O controlo da chave: desligada, o fio começa logo, como antes.
#[test]
fn sem_a_chave_a_compressao_comeca_logo() {
    let vivo = borracha_viva();
    let mut t = com_oleo();
    let (t0, horas) = lanca(&mut t, false);
    let (ini, _) = espera_o_fim(&horas);
    let atraso = ini.expect("comprimiu").duration_since(t0);
    assert!(
        atraso < Duration::from_millis(250),
        "começou logo: {atraso:?}"
    );
    assert_eq!(diferentes(&vivo, &volta_e_apaga(&mut t)), 0);
}

/// Acordar durante a espera não espera o fio e cancela-o: ele nunca toca a sessão, e o bit é o de quem nunca saiu.
#[test]
fn acordar_na_espera_cancela_a_compressao() {
    let vivo = borracha_viva();
    let mut t = com_oleo();
    let (t0, horas) = lanca(&mut t, true);
    // No relógio da parede: a tinta à espera conta como pronta, e a troca de modo acorda-a logo.
    t.set_wet_relogio_fixo(false);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    assert!(
        t.paint.wetpaint.session.is_some(),
        "acordou sem esperar o fio"
    );
    if t0.elapsed() < Duration::from_millis(250) {
        std::thread::sleep(Duration::from_millis(500));
        assert!(horas().0.is_none(), "o fio cancelado não comprimiu");
    }
    t.set_paint_tool_mode("eraser");
    assert_eq!(diferentes(&vivo, &apaga(&mut t)), 0);
}
