//! **Transparency** (dono 27/10, `1 − Alpha` do Principled) sobre o papel TEXTURIZADO: o gate do produto (a tela
//! final, composite + a luz do relevo) e a sonda do oráculo (`docs/Painter/49_realshader/oraculo/README.md`
//! §Transparency).

use super::*;

/// O cartão Real Oil com a Transparency `v` (a tela recompõe-se inteira).
fn com_transparencia(t: &mut PainterTool, v: f64) {
    let mut o = t.wet_oleo_real_valores();
    o.material.transparencia = v;
    t.set_wet_oleo_real(Some(o));
}

/// A média por canal de `|a − b|` nas amostras `m`.
fn diferenca_media(a: &[u8], b: &[u8], m: &[bool]) -> f64 {
    let (mut soma, mut n) = (0.0, 0usize);
    for ((pa, pb), _) in a
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0)
        .zip(m)
        .filter(|(_, v)| **v)
    {
        for k in 0..3 {
            soma += (f64::from(pa[k]) - f64::from(pb[k])).abs();
        }
        n += 3;
    }
    soma / n as f64
}

/// As telas de Relief 0 e 0,6 com a Transparency `v` e o dente (Relief 0,6 − 0) no miolo da tinta grossa e no
/// papel nu.
fn dente_com(
    liso: &mut PainterTool,
    rugoso: &mut PainterTool,
    v: f64,
    (miolo, nu): (&[bool], &[bool]),
) -> (f64, f64, Vec<u8>) {
    com_transparencia(liso, v);
    com_transparencia(rugoso, v);
    let (a, b) = (tela_final(liso), tela_final(rugoso));
    (dente_na_tela(&b, &a, miolo), dente_na_tela(&b, &a, nu), b)
}

/// ⭐ **A Transparency deixa ver o grão e a cor do papel** (dono 27/10): o Dabber de fábrica grosso (massa > 3000)
/// sobre o papel Cold com Relief 0,6 e o Óleo Real ligado. Transparency 0 = a tinta tapa o grão (o gate do
/// §BUG-papel); 1 = o miolo mostra o dente inteiro do papel nu e a COR do papel sem tinta (a tinta não projecta
/// sombra nem se vê: o Alpha 0 do Cycles); 0,5 = o dente a meio.
#[test]
fn a_transparencia_deixa_ver_o_grao_e_a_cor_do_papel() {
    let (mut liso, cob) = cena_papel_texturizado(true, 0.0, true);
    let (mut rugoso, _) = cena_papel_texturizado(true, 0.6, true);
    let (mut so_papel, _) = cena_papel_texturizado(true, 0.6, false);
    let (mut papel_liso, _) = cena_papel_texturizado(true, 0.0, false);
    let (miolo, nu) = miolo_e_papel_nu(&cob);
    let papel = tela_final(&mut so_papel);
    // O dente do papel SEM tinta no mesmo miolo (o do papel nu é de outra região: o grão não é estacionário).
    let dp = dente_na_tela(&papel, &tela_final(&mut papel_liso), &miolo);
    let (d0, n0, _) = dente_com(&mut liso, &mut rugoso, 0.0, (&miolo, &nu));
    let (dm, nm, _) = dente_com(&mut liso, &mut rugoso, 0.5, (&miolo, &nu));
    let (d1, n1, t1) = dente_com(&mut liso, &mut rugoso, 1.0, (&miolo, &nu));
    println!(
        "[transparency] dente miolo/nu: T0 {d0:.2}/{n0:.2} · T0,5 {dm:.2}/{nm:.2} · T1 {d1:.2}/{n1:.2} · papel no miolo {dp:.2}"
    );
    assert!(n0 > 4.0, "o papel nu devia mostrar o dente: {n0}");
    assert!(d0 <= 0.1 * n0, "T 0 tapa o grão: {d0} × {n0}");
    assert!(
        d1 >= 0.95 * dp,
        "T 1 deixa ver o grão inteiro: {d1} × {dp} (nu {n1})"
    );
    assert!(
        (0.3 * dp..=0.7 * dp).contains(&dm),
        "T 0,5 deixa ver o grão a meio: {dm} × {dp} (nu {nm})"
    );
    let cor = diferenca_media(&t1, &papel, &miolo);
    assert!(cor <= 1.0, "T 1 mostra a cor do papel sem tinta: Δ {cor}");
}

fn grava_ppm(f: &std::path::Path, px: &[u8], w: u32, h: u32) {
    let mut ppm = format!("P6\n{w} {h}\n255\n").into_bytes();
    for p in px.as_chunks::<4>().0 {
        ppm.extend_from_slice(&p[..3]);
    }
    std::fs::write(f, ppm).expect("grava");
}

/// A sonda do oráculo da Transparency: `$PH2D_OLEO_PNG/` recebe as telas finais do produto `nos_t<T>.ppm` (T 0 ·
/// 0,5 · 1), o papel sem tinta `papel.ppm` e, em `entrada/`, as entradas do `cycles.py` da mesma cena (a altura, o
/// albedo e a cobertura com T 0 — a Transparency não muda nenhum deles). Régua: `oraculo/transparencia_regua.py`.
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_oleo_real_transparencia() {
    let dir =
        std::path::PathBuf::from(std::env::var("PH2D_OLEO_PNG").expect("PH2D_OLEO_PNG=<pasta>"));
    let (mut t, _) = cena_papel_texturizado(true, 0.6, true);
    let (mut so_papel, _) = cena_papel_texturizado(true, 0.6, false);
    grava_ppm(&dir.join("papel.ppm"), &tela_final(&mut so_papel), OW, OH);
    for (nome, v) in [("t0", 0.0), ("t05", 0.5), ("t1", 1.0)] {
        com_transparencia(&mut t, v);
        grava_ppm(
            &dir.join(format!("nos_{nome}.ppm")),
            &tela_final(&mut t),
            OW,
            OH,
        );
    }
    let o = ph2d_wet_paint::oleo_real::OleoReal {
        material: ph2d_wet_paint::oleo_real::MaterialCartao::NEUTRO,
        ..t.wet_oleo_real_valores()
    };
    com_transparencia(&mut t, 0.0);
    let e = dir.join("entrada");
    std::fs::create_dir_all(&e).expect("pasta");
    grava_folha_com(&mut t, &e, o);
    println!("[transparency] {}", dir.display());
}
