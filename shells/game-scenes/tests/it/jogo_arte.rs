/// ⭐ A arte de fábrica abre no PH2D como VETOR editável (plano 06 §4): cada `<path>` do SVG vira uma forma do
/// documento, e a importação não deixa nada para trás (sem notas de perda) — quem acrescentar ao gerador um
/// `stroke`, `clipPath` ou `opacity` que o documento não exprime fica vermelho aqui.
#[test]
fn a_arte_de_fabrica_abre_como_vetor_sem_perdas() {
    for art in &ph2d_render::FACTORY_ART {
        let d = ph2d_vec_svg::import(art.svg, &ph2d_vec_svg::Options::default())
            .unwrap_or_else(|e| panic!("{:#x}: {e:?}", art.key));
        let caminhos = std::str::from_utf8(art.svg)
            .expect("utf-8")
            .matches("<path")
            .count();
        assert!(caminhos > 0);
        assert_eq!(d.shapes.len(), caminhos, "{:#x}", art.key);
        assert!(d.notes.is_empty(), "{:#x}: {:?}", art.key, d.notes);
    }
}
