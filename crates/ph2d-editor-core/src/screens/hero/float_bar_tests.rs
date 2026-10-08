use super::*;

fn area() -> Rect {
    Rect::new(100.0, 50.0, 600.0, 400.0)
}

#[test]
fn a_coluna_separa_os_grupos_e_nao_sobrepoe() {
    let r = column_rects(area(), &[2, 3]);
    assert_eq!(r.len(), 5);
    let step = button_px() + gap();
    assert!(
        (r[1].y - r[0].y - step).abs() < 1e-3,
        "dentro do grupo: um passo"
    );
    assert!(
        (r[2].y - r[1].y - (step + group_sep() - gap())).abs() < 1e-3,
        "entre grupos: o passo mais o separador"
    );
    for w in r.windows(2) {
        assert!(w[1].y >= w[0].y + w[0].h, "botões sobrepostos: {w:?}");
    }
    assert!(
        r.iter().all(|b| b.x >= area().x),
        "a coluna sai da área pela esquerda"
    );
}

#[test]
fn a_grelha_do_flyout_nunca_passa_do_fundo_da_area() {
    let a = area();
    let anchor = Rect::new(a.x + 8.0, a.y + a.h - button_px(), button_px(), button_px());
    let g = grid_rects(a, anchor, 12, 3);
    assert_eq!(g.len(), 12);
    let bottom = g.iter().map(|r| r.y + r.h).fold(0.0_f32, f32::max);
    assert!(
        bottom <= a.y + a.h,
        "a grelha passa do fundo ({bottom} > {})",
        a.y + a.h
    );
    assert!(
        g.iter().all(|r| r.x > anchor.x + anchor.w),
        "a grelha tapa o botão que a abre"
    );
}

#[test]
fn a_barra_da_selecao_sobe_e_desce_conforme_cabe() {
    let a = area();
    let alta = Rect::new(300.0, 300.0, 80.0, 60.0);
    let r = strip_rects(a, alta, &[4, 2], strip_button_px());
    assert!(
        r.iter().all(|b| b.y + b.h <= alta.y),
        "com lugar por cima, fica por cima"
    );
    let no_topo = Rect::new(300.0, a.y + 2.0, 80.0, 60.0);
    let r = strip_rects(a, no_topo, &[4, 2], strip_button_px());
    assert!(
        r.iter().all(|b| b.y >= no_topo.y + no_topo.h),
        "sem lugar por cima, desce para baixo da seleção"
    );
    assert!(
        r.iter().all(|b| b.x >= a.x && b.x + b.w <= a.x + a.w),
        "presa à área"
    );
}

#[test]
fn a_barra_fixada_ignora_a_selecao_e_fica_no_topo() {
    let a = area();
    let r = pinned_strip_rects(a, &[4, 2], button_px());
    assert_eq!(r.len(), 6);
    let top = r.iter().map(|b| b.y).fold(f32::MAX, f32::min);
    assert!(top < a.y + button_px(), "fixada mora no topo da área");
}

#[test]
fn um_grupo_vazio_nao_abre_um_buraco() {
    let a = area();
    let sel = Rect::new(300.0, 300.0, 80.0, 60.0);
    let com = strip_rects(a, sel, &[3, 0, 2], strip_button_px());
    let sem = strip_rects(a, sel, &[3, 2], strip_button_px());
    assert_eq!(com, sem);
}
