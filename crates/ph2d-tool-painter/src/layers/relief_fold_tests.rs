//! A dobra do relevo, passo a passo, e quem entra nela.

use super::*;

const CAP: u32 = 4;

/// O neutro: profundidade `1` em `Add` sobre nada é o relevo da camada; profundidade `0` não mexe na
/// pilha AO BIT, nem em `Level` com cobertura nula. CONTROLO: profundidade `0,5` mexe.
#[test]
fn o_neutro_da_dobra_e_ao_bit() {
    let h = 1.234_567_f32;
    let nunca = || -> f32 { panic!("Add não lê a cobertura") };
    assert_eq!(
        fold_relief_step(0.0, 2.5, 1.0, ReliefComposite::Add, nunca),
        2.5
    );
    assert_eq!(
        fold_relief_step(h, 7.0, 0.0, ReliefComposite::Add, nunca).to_bits(),
        h.to_bits()
    );
    assert_eq!(
        fold_relief_step(h, 7.0, 1.0, ReliefComposite::Level, || 0.0).to_bits(),
        h.to_bits()
    );
    assert_ne!(
        fold_relief_step(h, 7.0, 0.5, ReliefComposite::Add, nunca),
        h
    );
}

/// `Level` enterra na proporção da cobertura: sólida = a camada, metade = a média; profundidade
/// negativa cava.
#[test]
fn level_enterra_pela_cobertura_e_a_profundidade_negativa_cava() {
    assert_eq!(
        fold_relief_step(10.0, 2.0, 1.0, ReliefComposite::Level, || 1.0),
        2.0
    );
    assert_eq!(
        fold_relief_step(10.0, 2.0, 1.0, ReliefComposite::Level, || 0.5),
        6.0
    );
    assert_eq!(
        fold_relief_step(1.0, 3.0, -1.0, ReliefComposite::Add, || 0.0),
        -2.0
    );
}

/// Entram as visíveis de baixo para cima; uma escondida, ou dentro de um grupo escondido, fica de
/// fora. CONTROLO: com o grupo visível, os filhos voltam.
#[test]
fn entram_as_visiveis_de_baixo_para_cima_e_o_grupo_escondido_apaga_os_filhos() {
    let mut s = LayerStack::new();
    let base = s.add_raster("base", CAP, CAP).unwrap();
    let meio = s.add_raster("meio", CAP, CAP).unwrap();
    let grupo = s.add_group("grupo").unwrap();
    let dentro = s.add_raster("dentro", CAP, CAP).unwrap();
    assert!(s.move_into_group(dentro, grupo));
    let topo = s.add_raster("topo", CAP, CAP).unwrap();
    let todas = s.relief_layers_bottom_up();
    assert_eq!(todas, s.z_order_bottom_up(), "tudo visível = o z-order");
    assert!(todas.iter().position(|&i| i == base) < todas.iter().position(|&i| i == topo));

    s.set_visible(meio, false);
    s.set_visible(grupo, false);
    let vis = s.relief_layers_bottom_up();
    assert!(!vis.contains(&meio) && !vis.contains(&dentro) && !vis.contains(&grupo));
    assert!(vis.contains(&base) && vis.contains(&topo));
    assert!(!s.effectively_visible(dentro));

    s.set_visible(grupo, true);
    assert!(s.relief_layers_bottom_up().contains(&dentro));
}
