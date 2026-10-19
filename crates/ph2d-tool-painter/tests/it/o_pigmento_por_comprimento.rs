//! ⭐ **O pigmento é por COMPRIMENTO de traço** (prompt 51, doc 51 §2): o mesmo traço a 103 px pousa a mesma tinta a
//! qualquer Spacing — o de afinação do pincel sem Continuous é a referência; a régua é a tinta (a soma de `255 − Y`)
//! sobre a dela. Medido 18/10 (`sonda_spacing_continuo`): com a lei 0,98–1,03 em todos; sem ela o Spacing 0,01
//! punha 2,2× a tinta no Wet Round, o 0,1 + Continuous 0,72× no Express Oil, e a corda entre centros (em vez do arco)
//! 1,82× no Charcoal, que espalha os dabs.

use super::midias_de_fabrica::de_fabrica;
use super::o_relevo_dos_fios::cena_seca_de;
use super::sonda_spacing_continuo::{com, diferenca_de};

const RAIO: f32 = 51.5;
/// A tinta de cada variante no máximo a 4 % da referência (o pior medido com a lei: 3,4 %).
const TOLERANCIA: f64 = 0.04;

fn tinta(nome: &str, referencia: &str, variante: &str) -> f64 {
    let a = cena_seca_de(&com(de_fabrica(nome), referencia), RAIO, false).0;
    let b = cena_seca_de(&com(de_fabrica(nome), variante), RAIO, false).0;
    diferenca_de(&a.0, &b.0)[1]
}

#[test]
fn a_tinta_nao_muda_com_o_spacing() {
    let casos: [(&str, &str, &str, bool); 7] = [
        ("Wet Round", "spacing=0.025,continuous_spacing=false", "", true),
        ("Wet Round", "spacing=0.025,continuous_spacing=false", "spacing=0.01,continuous_spacing=false", true),
        ("Wet Round", "spacing=0.025,continuous_spacing=false", "spacing=0.05", true),
        ("Charcoal", "spacing=0.025,continuous_spacing=false", "", true),
        ("Express Oil", "spacing=0.01,continuous_spacing=false", "", true),
        // Os CONTROLOS: a lei de antes (o pigmento por dab) reprova.
        ("Wet Round", "spacing=0.025,continuous_spacing=false", "spacing=0.01,continuous_spacing=false,e.pigmentSpacing=0", false),
        ("Express Oil", "spacing=0.01,continuous_spacing=false", "e.pigmentSpacing=0", false),
    ];
    let medidas: Vec<f64> = std::thread::scope(|s| {
        let h: Vec<_> = casos
            .iter()
            .map(|&(n, r, v, _)| s.spawn(move || tinta(n, r, v)))
            .collect();
        h.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut falhas = Vec::new();
    for (&(n, _, v, lei), t) in casos.iter().zip(&medidas) {
        println!("[comprimento] {n} {v:?}: tinta {t:.4}");
        if lei != ((t - 1.0).abs() <= TOLERANCIA) {
            falhas.push(format!(
                "{n} {v:?}: tinta {t:.4} ({})",
                if lei { "a lei não a guarda" } else { "o controlo não reprova" }
            ));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}
