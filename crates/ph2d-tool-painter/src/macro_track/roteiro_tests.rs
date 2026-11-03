use super::{grava_no_papel, grava_no_papel_com_entrada, grava_o_splash};
use crate::macro_track::em_cena::MacroEmCena;

/// Quanto a tela andou, ao longo do caminho de `p`, de `antes` para `depois` na direcção da COR de
/// `p` (o croma: a cor menos a sua média), em níveis sRGB.
fn puxa(antes: &[u8], depois: &[u8], p: &super::Pincelada) -> f32 {
    let media = |t: &[u8]| {
        let mut s = [0f32; 3];
        for i in 1..10u8 {
            let [x, y] = super::ponto(p, f32::from(i) / 10.0, 128.0);
            let o = ((y as usize) * 128 + x as usize) * 4;
            for c in 0..3 {
                s[c] += f32::from(t[o + c]) / 9.0;
            }
        }
        s
    };
    let (a, b) = (media(antes), media(depois));
    let d: Vec<f32> = (0..3).map(|c| b[c] - a[c]).collect();
    let dm = d.iter().sum::<f32>() / 3.0;
    let cor = p.cor.map(f32::from);
    let cm = cor.iter().sum::<f32>() / 3.0;
    let n = cor.iter().map(|c| (c - cm) * (c - cm)).sum::<f32>().sqrt();
    (0..3).map(|c| (d[c] - dm) * (cor[c] - cm)).sum::<f32>() / n
}

/// Texels não brancos da tela do palco de `doc` no instante `t`.
fn pintados_em(doc: &[u8], t: f64) -> usize {
    let mut m = MacroEmCena::de(doc).expect("lê-se").expect("trilha");
    m.segue(t).expect("toca");
    let (tela, _, _) = m.toma_a_tela().expect("a tela mostra-se");
    tela.as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[..3] != [255; 3])
        .count()
}

/// A cena `PH2D_MACRO_SMOKE=5` abre com o clipe parado no instante 0: a gravação com ENTRADA mostra
/// aí a tela branca e pinta depois. ⛔ O controlo: no papel sem entrada o instante 0 já tem o 1.º
/// toque (a mancha da 1.ª foto da `=5`, 17/10).
#[test]
fn com_entrada_o_instante_0_e_a_tela_branca() {
    let (sem, _) = grava_no_papel(128, 0, 1).expect("arma");
    assert!(
        pintados_em(&sem, 0.0) > 0,
        "controlo: sem entrada, o instante 0 traz o 1.º toque"
    );
    let (com, d) = grava_no_papel_com_entrada(128, 0, 1, 30).expect("arma");
    assert_eq!(
        pintados_em(&com, 0.0),
        0,
        "com entrada, o instante 0 é a tela branca"
    );
    assert!(
        pintados_em(&com, d) > 128 * 128 / 50,
        "controlo: a gravação pinta"
    );
}

/// A cena `PH2D_MACRO_SMOKE=1` ensina o que acontece: o Splash tocado pinta a tela branca, cada
/// pincelada puxa a tela para a SUA cor por onde passa, e o início é a tela branca. ⛔ Não se pede
/// texel «vermelho» no fim: com o Pigment mixing (K-M) de fábrica (dono 30/10, `0170428784`) o
/// vermelho pousa todo sobre o azul ainda molhado e o Splash acaba em cinzas e verdes de mistura —
/// a lei do pigmento, não defeito (cura 02/11c).
#[test]
fn o_splash_tocado_pinta_a_tela_e_o_inicio_e_branco() {
    let (bytes, d) = grava_o_splash(128, 0).expect("arma no Wet Paint");
    let mut m = MacroEmCena::de(&bytes)
        .expect("lê-se")
        .expect("tem trilha do Painter");
    // Antes de qualquer passo (o instante 0 já traz o 1.º toque do pincel).
    let (inicio, _, _) = m.toma_a_tela().expect("a tela inicial mostra-se");
    assert!(inicio.iter().all(|&b| b == 255), "o início é a tela branca");
    m.segue(d).expect("toca até ao fim");
    assert!(m.erro().is_none(), "{:?}", m.erro());
    let (fim, _, _) = m.toma_a_tela().expect("a tela mudou");
    let pintados = fim
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[..3] != [255; 3])
        .count();
    assert!(pintados > 128 * 128 / 5, "pintou {pintados} texels");
    // Cada cor chega: tocada de novo até ao fim de cada pincelada, a tela anda na direcção dela.
    let mut m = MacroEmCena::de(&bytes).expect("lê-se").expect("trilha");
    let mut antes = inicio.to_vec();
    for (k, p) in super::SPLASH.iter().enumerate() {
        let fim_k = (u32::try_from(k).expect("4") + 1) * (super::TRACO + 1 + super::ENTRE);
        m.segue((f64::from(fim_k) / 60.0).min(d)).expect("toca");
        let agora = m.toma_a_tela().expect("a tela mostra-se").0.to_vec();
        let anda = puxa(&antes, &agora, p);
        assert!(anda > 5.0, "a pincelada {k} ({:?}) puxou {anda:.2}", p.cor);
        antes = agora;
    }
}
