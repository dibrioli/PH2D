//! ⭐ **A água × o óleo seco** (smoke do dono J10, lei `project_watery_paint_against_relief_owner_law_2026_11_01`): a
//! água não atravessa o corpo do óleo mais depressa que escorre no papel (o atrito do leito, `solver::chao_forca`), e a
//! água parada DISSOLVE a tinta solúvel e BAIXA o relevo (`solver::remolha_parada`, Re-wet × Solubility). Cena da sonda
//! `agua_no_oleo_tests` (Dabber seco, Wet Drip d0,9 por cima). Números: `docs/Painter/Arquitetura/J11_agua_no_oleo.md`.

use super::agua_no_oleo_tests::{Variante, linha};

fn corre(v: Variante) -> String {
    std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(move || linha(v))
        .expect("a thread")
        .join()
        .expect("a sonda")
}

/// A frente média de B na amostra `k` (`| k:linha vo…`).
fn frente(l: &str, k: u32) -> f64 {
    let m = format!("| {k}:");
    let i = l
        .find(&m)
        .unwrap_or_else(|| panic!("sem a amostra {k} em {l}"));
    l[i + m.len()..]
        .split_whitespace()
        .next()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| panic!("amostra {k} ilegível em {l}"))
}

fn orla(l: &str) -> f64 {
    let i = l.find("sett orla A ").expect("a orla");
    l[i + 12..]
        .split_whitespace()
        .next()
        .and_then(|v| v.parse().ok())
        .expect("a orla lê-se")
}

/// Seis passagens: 120 tiques depois, a frente da água no corpo do óleo não vai além da do papel nu com a mesma água.
/// Medido: papel 91,5 · óleo 90,1 (sem o atrito 99,8 — a água atravessava o óleo mais depressa que escorria).
#[test]
fn a_agua_nao_atravessa_o_oleo_mais_depressa_que_escorre_no_papel() {
    let papel = corre(("papel6", false, 0.0, 0.0, 0.0, 6));
    let oleo = corre(("hoje6", true, 0.0, 0.0, 0.0, 6));
    let (fp, fo) = (frente(&papel, 8), frente(&oleo, 8));
    assert!(
        fo <= fp,
        "a água atravessou o óleo mais depressa que escorre no papel: óleo {fo:.1} × papel {fp:.1}\n{oleo}\n{papel}"
    );
}

/// Re-wet 1 sobre o Dabber de Solubility 1: a água parada dissolve a orla do óleo (resta 0,07 do assentado; só sob o
/// pincel, 0,80); com Solubility 0 nada se dissolve além do re-molhar da aquarela (0,95).
#[test]
fn a_agua_parada_dissolve_a_tinta_soluvel_da_orla() {
    let soluvel = orla(&corre(("s1rw1", true, 1.0, 1.0, 0.0, 1)));
    let insoluvel = orla(&corre(("s0rw1", true, 0.0, 1.0, 0.0, 1)));
    assert!(
        soluvel < 0.3,
        "a água parada não dissolveu a orla solúvel: resta {soluvel:.3}"
    );
    assert!(
        insoluvel > 0.9,
        "a tinta insolúvel dissolveu-se: resta {insoluvel:.3}"
    );
}

/// O relevo dissolvido deixa de segurar a água: Re-wet 0,35 em seis passagens passa por cima com MAIS água que a tinta
/// insolúvel (medido: 4 075 929 × 251 214 abaixo do óleo; sem a água parada 1 250 462).
#[test]
fn o_relevo_dissolvido_deixa_a_agua_passar() {
    let abaixo = |l: &str| -> f64 {
        let i = l.find("| abaixo ").expect("abaixo");
        l[i + 9..]
            .split_whitespace()
            .next()
            .and_then(|v| v.parse().ok())
            .expect("abaixo lê-se")
    };
    let sol = abaixo(&corre(("s1rw035x6", true, 1.0, 0.35, 0.0, 6)));
    let ins = abaixo(&corre(("hoje6", true, 0.0, 0.35, 0.0, 6)));
    assert!(
        sol > 2.0 * ins.max(1.0) && sol > 2_000_000.0,
        "o relevo dissolvido não deixou passar a água: solúvel {sol:.0} × insolúvel {ins:.0}"
    );
}
