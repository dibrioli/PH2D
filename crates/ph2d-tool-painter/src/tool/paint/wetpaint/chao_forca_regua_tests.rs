//! ⭐ **A aguada contra o relevo seco** (lei do dono 01/11, smoke da janela 9): a água que chega à tinta grossa e seca
//! EMPOÇA na borda, CONTORNA pelas laterais se houver por onde, e só passa POR CIMA com volume. Lei: o chão (o óleo
//! assente e seco no início do traço) como força no momento (`solver::chao_forca`). A cena é a da sonda
//! `escorre_contorna_tests` (Dabber seco + Wet Drip d0,9 por cima, tela inclinada de fábrica). Controlos medidos com a
//! força desligada (`CHAO_FORCA < 0`, sonda J10 `diag_o_chao_como_forca`/`diag_o_volume_transborda`):
//! `docs/Painter/Arquitetura/J10_chao_como_forca.md`.

use super::escorre_contorna_tests::{ObstaculoA, linha};

/// O número do campo `nome` na linha da sonda (`… | nome 123 | …`).
fn campo(l: &str, nome: &str) -> f64 {
    let k = l
        .find(&format!(" {nome} "))
        .unwrap_or_else(|| panic!("sem «{nome}» em {l}"));
    l[k + nome.len() + 2..]
        .split_whitespace()
        .next()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| panic!("«{nome}» ilegível em {l}"))
}

fn corre(a: ObstaculoA, vezes: u32) -> String {
    std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(move || linha(a, 1200, 0.0, 0, vezes))
        .expect("a thread")
        .join()
        .expect("a sonda")
}

/// Um traço contra o Dabber comprido: nada atravessa e a água fica em cima da borda. Controlo (sem a força): abaixo
/// 344 982, acima 279 582 — produto 0 · 441 965.
#[test]
fn a_aguada_empoca_contra_o_oleo_seco() {
    let l = corre(ObstaculoA::Dabber, 1);
    let (abaixo, acima) = (campo(&l, "abaixo"), campo(&l, "acima"));
    assert!(
        abaixo < 34_000.0,
        "a aguada atravessou o óleo seco: abaixo {abaixo:.0} — {l}"
    );
    assert!(
        acima > 360_000.0,
        "a aguada não empoçou acima do óleo: acima {acima:.0} — {l}"
    );
}

/// O Dabber curto debaixo do Wet Drip largo: a água desce pelos LADOS. Controlo: lado 249 — produto 26 614.
#[test]
fn a_aguada_contorna_o_oleo_seco_pelos_lados() {
    let l = corre(ObstaculoA::DabberCurto, 1);
    let lado = campo(&l, "lado");
    assert!(
        lado > 10_000.0,
        "a aguada não contornou o óleo: lado {lado:.0} — {l}"
    );
}

/// Seis passagens no mesmo sítio: com volume a poça passa POR CIMA. Medido: 1 passagem abaixo 0, 3 → 254 200,
/// 6 → 1 082 565; sem a água na superfície (`−G·∇chão` só) as seis davam 0 (a poça subia a `film` 497 presa à borda).
#[test]
fn com_volume_a_aguada_passa_por_cima_do_oleo_seco() {
    let l = corre(ObstaculoA::Dabber, 6);
    let abaixo = campo(&l, "abaixo");
    assert!(
        abaixo > 500_000.0,
        "com volume a aguada não passou por cima: abaixo {abaixo:.0} — {l}"
    );
    let poca = campo(&l, "poca");
    assert!(
        poca < 20.0,
        "a poça empilhou água na borda: film {poca:.2} — {l}"
    );
}

/// O Dabber diluído (Dilution 0,5, Oil) ainda fluido não é chão: o líquido dele é óleo e corre com ele. Controlo: abaixo
/// 6 636 667 — produto 6 729 472; sem o «seco» da fotografia (a tinta molhada como chão) sangra (mutação M3 da J10).
#[test]
fn a_tinta_de_oleo_ainda_fluida_nao_e_chao() {
    let l = corre(ObstaculoA::DabberDiluido, 1);
    let (abaixo, max) = (campo(&l, "abaixo"), campo(&l, "max"));
    assert!(
        abaixo > 6_000_000.0,
        "a tinta de óleo fluida travou a água: abaixo {abaixo:.0} — {l}"
    );
    assert!(
        max < 0.1,
        "a tinta de óleo fluida virou chão: max {max:.3} — {l}"
    );
}
