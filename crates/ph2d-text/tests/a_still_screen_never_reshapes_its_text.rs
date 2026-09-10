//! ⭐⭐⭐ **UM ECRÃ PARADO NÃO VOLTA A MOLDAR TEXTO** — o gate do penhasco de 2026-09-10.
//!
//! # O defeito, medido
//!
//! A cache de layouts era **deitada fora inteira** ao transbordar. Enquanto o conjunto de
//! trabalho de um quadro coubesse no tecto, ninguém via nada; ao passá-lo, o `clear` caía **a
//! meio do quadro**, tudo o que vinha depois falhava, voltava a encher e a transbordar — e o
//! quadro seguinte moldava **tudo outra vez**. Medido no app com todos os painéis abertos:
//! `775` textos distintos ⇒ **0** re-moldagens e `4,87 ms`; `1 110` ⇒ **1 109** re-moldagens e
//! `157,91 ms`. *`+43 %` de conteúdo, `29×` de relógio.*
//!
//! # ⚠️ Por que a régua é uma CONTAGEM e não um relógio
//!
//! Um gate de tempo sobre isto seria mais um membro da família de flakes de carga do
//! `CLAUDE.md` §5.0 — e a grandeza que interessa é **contável**: quantas vezes o mesmo texto foi
//! moldado. *Uma contagem é determinística e diz o mecanismo; um relógio diz o sintoma e mente
//! sob carga.*

use ph2d_text::TextSystem;

/// O tecto de UMA geração. ⚠️ Lido do produto e não escrito aqui: um número copiado deixaria
/// este gate a medir o tecto de ontem no dia em que ele mudasse.
fn cap() -> usize {
    ph2d_text::LAYOUT_CACHE_CAP
}

fn paint_once(sys: &mut TextSystem, textos: &[String]) {
    for t in textos {
        let _ = sys.layout(t, 13.0, f32::INFINITY);
    }
}

/// ⭐⭐⭐ **Com o conjunto de trabalho ACIMA do tecto de uma geração, o segundo quadro molda ZERO.**
#[test]
fn a_working_set_over_one_generation_still_reshapes_nothing() {
    let mut sys = TextSystem::without_system_fonts();
    let n = cap() + cap() / 4;
    let textos: Vec<String> = (0..n).map(|i| format!("linha numero {i}")).collect();

    paint_once(&mut sys, &textos);
    let depois_do_primeiro = sys.shapes();
    assert!(
        depois_do_primeiro >= n as u64,
        "controlo partido: o 1.º quadro moldou {depois_do_primeiro} de {n} textos — esta fixtura \
         não enche a cache"
    );
    assert!(
        n > cap(),
        "controlo partido: {n} textos não passam o tecto de {} — sem transbordo não há o que medir",
        cap()
    );

    paint_once(&mut sys, &textos);
    let no_segundo = sys.shapes() - depois_do_primeiro;
    assert_eq!(
        no_segundo, 0,
        "um ecrã PARADO voltou a moldar {no_segundo} textos: a cache está a perder o conjunto de \
         trabalho a cada quadro, e o preço é a moldagem inteira outra vez"
    );
}

/// ⛔ **E o que sobra é LIMITADO** — a rotação guarda duas gerações, nunca mais.
///
/// ⚠️ É o recurso que o tecto mede: **memória**. Um conjunto de trabalho muito acima de `2 × CAP`
/// volta a bater, e isso é o desenho — o que não pode voltar é perder **tudo** de uma vez.
#[test]
fn the_cache_never_holds_more_than_two_generations() {
    let mut sys = TextSystem::without_system_fonts();
    let n = cap() * 4;
    let textos: Vec<String> = (0..n).map(|i| format!("texto {i}")).collect();
    paint_once(&mut sys, &textos);
    assert!(
        sys.layout_cache_len() <= cap() * 2,
        "a cache guarda {} layouts, acima das duas gerações ({} × 2)",
        sys.layout_cache_len(),
        cap()
    );
}
