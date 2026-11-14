//! **Transparency** (dono 27/10, `1 − Alpha` do Principled) sobre o papel TEXTURIZADO: o gate do produto (a tela
//! final, composite + a luz do relevo) e a sonda do oráculo (`docs/Painter/49_realshader/oraculo/README.md`
//! §Transparency).

use super::*;

/// A tinta pousada com a Transparency `v` (a tela recompõe-se inteira).
fn com_transparencia(t: &mut PainterTool, v: f64) {
    com_material(t, 8, v);
}

/// A tinta POUSADA com o peso `k` do material da célula a `v` — 8 Transparency, 9 Transmission, 10 Relevo do papel,
/// 11 o modo da Transmission (`0` Blender · `1` Clear glass) —
/// em todas as células de todas as camadas, e a tela recomposta: o mesmo traço como se o pincel o tivesse pintado com
/// o knob (W3b ronda 3: os três saíram do cartão, são só do pincel; `material_do_pincel_tests` prova que o pincel os
/// pousa). Assim as medidas comparam a MESMA tinta.
pub(super) fn com_material(t: &mut PainterTool, k: usize, v: f64) {
    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    for l in &mut s.engine.layers {
        ph2d_wet_paint::grid::material::para_cada(&mut l.grid, |_, m| {
            let alvo = match k {
                8 => &mut m.transparencia,
                9 => &mut m.transmissao,
                10 => &mut m.relevo_papel,
                11 => &mut m.vidro_claro,
                _ => panic!("peso {k}"),
            };
            *alvo = v as f32;
        });
    }
    t.wet_recomposite_full();
}

/// O dente RELATIVO (Relief 0,6 − 0) nas amostras `m`: o desvio da luminância ÷ a luminância média — a luz do relevo
/// MULTIPLICA a cor, e sob a tinta escura o mesmo grão mexe menos níveis.
fn dente_relativo(rugoso: &[u8], liso: &[u8], m: &[bool]) -> f64 {
    let l = |p: &[u8; 4]| {
        0.2126 * f64::from(p[0]) + 0.7152 * f64::from(p[1]) + 0.0722 * f64::from(p[2])
    };
    let (s, n) = liso
        .as_chunks::<4>()
        .0
        .iter()
        .zip(m)
        .filter(|(_, v)| **v)
        .fold((0.0, 0.0), |(s, n), (p, _)| (s + l(p), n + 1.0));
    dente_na_tela(rugoso, liso, m) / (s / n).max(1.0)
}

/// A tela final de `t` SEM o dente do papel na tinta (o grão assado do vidro desligado e o ganho do Paper Relief a
/// 0) — a régua do grão que a tinta mostra: com o piso `RELIEF_NA_TINTA_MIN` (U2) a tela de Relief 0 também o tem.
fn sem_o_dente_na_tinta(t: &mut PainterTool) -> Vec<u8> {
    use crate::tool::paint::wetpaint::grao_no_filme::sem_grao;
    let o = t.wet_oleo_real_valores();
    let mut sem = o;
    sem.af.papel_na_tinta = 0.0;
    sem_grao::set(true);
    t.set_wet_oleo_real(Some(sem));
    t.wet_recomposite_full();
    let px = tela_final(t);
    sem_grao::set(false);
    t.set_wet_oleo_real(Some(o));
    t.wet_recomposite_full();
    px
}

/// ⭐ **A Transmission e o Relevo do papel mostram o grão** (W3b, dono 27/10: «a tinta era viva»): o Dabber grosso
/// sobre o papel Cold Relief 0,6. Com os dois a 0 a tinta tapa o grão (§BUG-papel); a 1, o dente RELATIVO no miolo
/// (a tela − a mesma sem o dente na tinta) é ≥ 60 % do do papel nu (Relief 0,6 − 0); a 0,5, entre os dois.
#[test]
fn a_transmissao_e_o_relevo_do_papel_mostram_o_grao() {
    let (mut liso, cob) = cena_papel_texturizado(true, 0.0, true);
    let (mut rugoso, _) = cena_papel_texturizado(true, 0.6, true);
    let (miolo, nu) = miolo_e_papel_nu(&cob);
    let (a, b) = (tela_final(&mut liso), tela_final(&mut rugoso));
    let n0 = dente_relativo(&b, &a, &nu);
    let nu_abs = dente_na_tela(&b, &a, &nu);
    let mut dente = |k: usize, v: f64| {
        com_material(&mut rugoso, k, v);
        let com = tela_final(&mut rugoso);
        let sem = sem_o_dente_na_tinta(&mut rugoso);
        (dente_relativo(&com, &sem, &miolo), com, sem)
    };
    let (d0, ..) = dente(9, 0.0);
    let (wm, ..) = dente(9, 0.5);
    let (w1, ..) = dente(9, 1.0);
    dente(9, 0.0);
    let (rm, ..) = dente(10, 0.5);
    let (r1, com, sem) = dente(10, 1.0);
    println!(
        "[transmission] dente relativo miolo ÷ nu ({n0:.4}): 0 {:.2} · Transmission 0,5 {:.2} · 1 {:.2} · Relevo do papel 0,5 {:.2} · 1 {:.2}",
        d0 / n0,
        wm / n0,
        w1 / n0,
        rm / n0,
        r1 / n0
    );
    assert!(d0 <= 0.1 * n0, "a 0 a tinta tapa o grão: {d0} × {n0}");
    for (nome, meio, um) in [("Transmission", wm, w1), ("Relevo do papel", rm, r1)] {
        assert!(um >= 0.6 * n0, "{nome} 1 mostra o grão: {um} × {n0}");
        assert!(
            meio > d0 && meio < um,
            "{nome} 0,5 entre os dois: {meio} ({d0}..{um})"
        );
    }
    // ⭐ O Relevo do papel 1 na régua do §BUG-papel (o dente na tinta grossa ÷ no papel nu, absoluto). ⚠️ A âncora é a
    // do DONO, não a do Cycles: a rampa do `relief_na_tinta` (01fb209238) dá 0,84 a Relief 0,6 e mede 2,24 contra 1,71
    // do Cycles (`oraculo/relevo_papel_cycles.py`, à profundidade 0,6). O dono provou as duas leis lado a lado (08/11,
    // «ambas me parecem boas») e ficou com a rampa — a do Cycles tirava metade do papel à tinta no Relief 0 de fábrica.
    // E o grão também nos BRILHOS da tinta.
    let abs = dente_na_tela(&com, &sem, &miolo) / nu_abs;
    let brilhos = dente_nos_brilhos(&com, &sem, &miolo) / nu_abs;
    println!(
        "[transmission] Relevo do papel 1: miolo ÷ nu {abs:.3} (dono 3,86 no ecrã, 2,24 com a luz dobrada · Cycles 1,71) · nos brilhos {brilhos:.3}"
    );
    assert!(
        // A faixa relativa de sempre (−18 % · +19 %) sobre o número aprovado. B44.3: 2,24 era a tela com a luz
        // DOBRADA (a régua reacendia o `run_full`); o ecrã do dono, a mesma lei sem repintar nada, mede 3,86.
        (3.17..=4.60).contains(&abs),
        "Relevo do papel 1 ≈ o aprovado pelo dono: {abs} × 3,86"
    );
    assert!(brilhos >= 0.5, "o grão nos brilhos: {brilhos}");
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
    // B44.3: a régua lê o ecrã (uma luz): o papel nu mede 3,08 (antes > 4 com a luz dobrada); −19 % de folga.
    assert!(n0 > 2.5, "o papel nu devia mostrar o dente: {n0}");
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
    let (mut t, cob) = cena_papel_texturizado(true, 0.6, true);
    let (mut so_papel, _) = cena_papel_texturizado(true, 0.6, false);
    // O miolo que os gates do produto medem (`miolo_e_papel_nu`): a régua dos modos lê também o Cycles nele.
    let mut pgm = format!("P5\n{OW} {OH}\n255\n").into_bytes();
    pgm.extend(
        miolo_e_papel_nu(&cob)
            .0
            .iter()
            .map(|&m| if m { 255u8 } else { 0 }),
    );
    std::fs::write(dir.join("miolo_produto.pgm"), pgm).expect("grava");
    // W3b: `PH2D_OLEO_CREME=1` = o papel creme do smoke do dono (o Cycles lê-o no `papel.ppm`).
    if std::env::var_os("PH2D_OLEO_CREME").is_some() {
        crate::tool::paint::wetpaint::oleo_no_quadro::no_evento(true);
        for x in [&mut t, &mut so_papel] {
            x.reconcile_substrate();
            poe_o_papel(x, CREME);
            x.reconcile_substrate();
        }
        crate::tool::paint::wetpaint::oleo_no_quadro::no_evento(false);
    }
    grava_ppm(&dir.join("papel.ppm"), &tela_final(&mut so_papel), OW, OH);
    // A Transparency (Alpha, 8), a Transmission (9) e o Relevo do papel (10), um de cada vez, os outros a 0. A
    // Transmission no modo Blender (`tr*`, o de omissão) e no Clear glass (`vc*`, o modo 11 a 1).
    for (nome, k, v, claro) in [
        ("t0", 8, 0.0, 0.0),
        ("t05", 8, 0.5, 0.0),
        ("t1", 8, 1.0, 0.0),
        ("tr05", 9, 0.5, 0.0),
        ("tr1", 9, 1.0, 0.0),
        ("vc05", 9, 0.5, 1.0),
        ("vc1", 9, 1.0, 1.0),
        ("rp05", 10, 0.5, 0.0),
        ("rp1", 10, 1.0, 0.0),
    ] {
        com_material(&mut t, 11, claro);
        com_material(&mut t, k, v);
        grava_ppm(
            &dir.join(format!("nos_{nome}.ppm")),
            &tela_final(&mut t),
            OW,
            OH,
        );
        com_material(&mut t, k, 0.0);
        com_material(&mut t, 11, 0.0);
    }
    // A varredura da calibração do modo Blender (`PH2D_OLEO_MODOS=0.1,0.2,…`): o modo intermédio `m` dá o filtro
    // `a^(2k)` com `k = K_BLENDER + (VIDRO_K − K_BLENDER)·m` — as variantes no MESMO processo.
    if let Ok(lista) = std::env::var("PH2D_OLEO_MODOS") {
        for m in lista
            .split(',')
            .filter_map(|x| x.trim().parse::<f64>().ok())
        {
            com_material(&mut t, 11, m);
            for (nome, v) in [("05", 0.5), ("1", 1.0)] {
                com_material(&mut t, 9, v);
                grava_ppm(
                    &dir.join(format!("nos_m{m}_{nome}.ppm")),
                    &tela_final(&mut t),
                    OW,
                    OH,
                );
            }
            com_material(&mut t, 9, 0.0);
            com_material(&mut t, 11, 0.0);
        }
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

/// O dente nos BRILHOS: o desvio (Relief 0,6 − 0) nas amostras `m` cuja luminância está no décimo mais claro da tela
/// lisa.
fn dente_nos_brilhos(rugoso: &[u8], liso: &[u8], m: &[bool]) -> f64 {
    let l = |p: &[u8; 4]| {
        0.2126 * f64::from(p[0]) + 0.7152 * f64::from(p[1]) + 0.0722 * f64::from(p[2])
    };
    let px = liso.as_chunks::<4>().0;
    let mut ls: Vec<f64> = px
        .iter()
        .zip(m)
        .filter(|(_, v)| **v)
        .map(|(p, _)| l(p))
        .collect();
    ls.sort_by(f64::total_cmp);
    let corte = ls[ls.len() * 9 / 10];
    let brilho: Vec<bool> = px.iter().zip(m).map(|(p, v)| *v && l(p) >= corte).collect();
    dente_na_tela(rugoso, liso, &brilho)
}

/// A sonda da calibração do Relevo do papel (W3b): `Afinacao::papel_na_tinta` contra a tela de ANTES de
/// `ca26a7738a` (o dente na tinta grossa ÷ no papel nu, Dabber de fábrica com o óleo ligado: **0,67**, §BUG-papel).
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_relevo_do_papel_calibra() {
    let (mut liso, cob) = cena_papel_texturizado(true, 0.0, true);
    let (mut rugoso, _) = cena_papel_texturizado(true, 0.6, true);
    let (miolo, nu) = miolo_e_papel_nu(&cob);
    for g in [0.0, 1.0, 2.0, 4.0, 8.0, 16.0] {
        for t in [&mut liso, &mut rugoso] {
            let mut o = t.wet_oleo_real_valores();
            o.af.papel_na_tinta = g;
            t.set_wet_oleo_real(Some(o));
            com_material(t, 10, 1.0);
        }
        let (a, b) = (tela_final(&mut liso), tela_final(&mut rugoso));
        let (dm, dn) = (dente_na_tela(&b, &a, &miolo), dente_na_tela(&b, &a, &nu));
        println!(
            "[relevo do papel] ganho {g}: miolo ÷ nu {:.3} (alvo 0,67) · relativo {:.3} · nos brilhos ÷ nu {:.3}",
            dm / dn,
            dente_relativo(&b, &a, &miolo) / dente_relativo(&b, &a, &nu),
            dente_nos_brilhos(&b, &a, &miolo) / dn
        );
    }
}

/// A sonda do ORÁCULO do Relevo do papel (cura 02/11c): em `$PH2D_OLEO_PNG/entrada` as entradas do `cycles.py` da
/// cena do gate com Relevo do papel 0 (`h_fina` sem o dente), o dente do papel em células `dente_papel.f32`
/// (`dente_papel × Grid::paper`, a geometria do papel nu) e as máscaras do gate `miolo.pgm`/`nu.pgm`; imprime a
/// régua do gate no produto por ganho `papel_na_tinta`. Régua: `oraculo/relevo_papel_cycles.py`.
#[test]
#[ignore = "medicao — corre-se à mão no perfil smoke"]
fn medida_relevo_do_papel_oraculo() {
    let dir =
        std::path::PathBuf::from(std::env::var("PH2D_OLEO_PNG").expect("PH2D_OLEO_PNG=<pasta>"));
    let e = dir.join("entrada");
    std::fs::create_dir_all(&e).expect("pasta");
    let (mut liso, cob) = cena_papel_texturizado(true, 0.0, true);
    let (mut rugoso, _) = cena_papel_texturizado(true, 0.6, true);
    let (miolo, nu) = miolo_e_papel_nu(&cob);
    for (nome, m) in [("miolo", &miolo), ("nu", &nu)] {
        let mut pgm = format!("P5\n{OW} {OH}\n255\n").into_bytes();
        pgm.extend(m.iter().map(|&v| if v { 255u8 } else { 0 }));
        std::fs::write(e.join(format!("{nome}.pgm")), pgm).expect("grava");
    }
    let nu_abs = dente_na_tela(&tela_final(&mut rugoso), &tela_final(&mut liso), &nu);
    let o0 = rugoso.wet_oleo_real_valores();
    for g in [0.0, 1.0, 6.0, 7.6, 8.0] {
        let mut o = o0;
        o.af.papel_na_tinta = g;
        rugoso.set_wet_oleo_real(Some(o));
        com_material(&mut rugoso, 10, 1.0);
        let com = tela_final(&mut rugoso);
        let sem = sem_o_dente_na_tinta(&mut rugoso);
        println!(
            "[relevo oraculo] ganho {g}: miolo ÷ nu {:.3} · nu {nu_abs:.3}",
            dente_na_tela(&com, &sem, &miolo) / nu_abs
        );
    }
    rugoso.set_wet_oleo_real(Some(o0));
    com_material(&mut rugoso, 10, 0.0);
    let o = ph2d_wet_paint::oleo_real::OleoReal {
        material: ph2d_wet_paint::oleo_real::MaterialCartao::NEUTRO,
        ..rugoso.wet_oleo_real_valores()
    };
    grava_folha_com(&mut rugoso, &e, o);
    // O dente do papel nu em píxeis de relevo (o que o composite põe em `OleoReal::dente_papel`).
    let dente_px = f64::from(
        crate::tool::paint::wetpaint::grao_no_filme::relief_na_tinta(rugoso.paint.substrate_depth)
            * crate::tool::paint::substrate_relief::MAX_TOOTH_PX,
    );
    let s = rugoso.paint.wetpaint.session.as_mut().expect("sessão");
    let (gw, gh) = s.grid;
    let g = s.engine.active_grid();
    let k = dente_px / f64::from(s.ratio.max(1));
    let mut dente = Vec::with_capacity(gw * gh * 4);
    for cy in 1..=gh {
        for cx in 1..=gw {
            dente
                .extend_from_slice(&((k * f64::from(g.paper[cx + cy * g.s])) as f32).to_le_bytes());
        }
    }
    std::fs::write(e.join("dente_papel.f32"), dente).expect("grava");
    grava_ppm(&e.join("nu_produto.ppm"), &tela_final(&mut liso), OW, OH);
    println!(
        "[relevo oraculo] dente_papel {dente_px} · ratio {} · {}",
        s.ratio,
        dir.display()
    );
}

/// O papel CREME do documento (o do smoke do dono, `smoke_27_10/4.png`), aplicado como o painel aplica a cor.
const CREME: [u8; 3] = [222, 208, 180];

fn poe_o_papel(t: &mut PainterTool, c: [u8; 3]) {
    t.set_paper_color_rgb8(c[0], c[1], c[2]);
    t.papel_segue_a_cor();
}

/// A média LINEAR do canal `k` nas amostras `m`.
fn canal_linear(px: &[u8], m: &[bool], k: usize) -> f64 {
    let (s, n) = px
        .as_chunks::<4>()
        .0
        .iter()
        .zip(m)
        .filter(|(_, v)| **v)
        .fold((0.0, 0.0), |(s, n), (p, _)| {
            (
                s + ph2d_wet_paint::colorops::transfer::srgb255_to_linear(f64::from(p[k])),
                n + 1.0,
            )
        });
    s / n
}

/// ⭐ **A Transmission tinge a cor REAL do papel** (o dono pinta sobre papel creme): o miolo da tinta transmissiva
/// (Transmission 1) sobre o papel creme sai, no canal da tinta (o azul), o papel × a do papel branco — e muda quando o
/// papel muda DEPOIS do traço (a sessão viva recompõe-se, `reconcilia_o_grao_do_oleo`).
#[test]
fn a_transmissao_tinge_o_papel_creme_do_documento() {
    crate::tool::paint::wetpaint::oleo_no_quadro::no_evento(true);
    let (mut t, cob) = cena_papel_texturizado(true, 0.6, true);
    let (miolo, _) = miolo_e_papel_nu(&cob);
    com_material(&mut t, 9, 1.0);
    // O quadro do produto: o reconcile corre a cada quadro (a chave fica com o papel de agora).
    t.reconcile_substrate();
    let branco = tela_final(&mut t);
    poe_o_papel(&mut t, CREME);
    t.reconcile_substrate();
    let creme = tela_final(&mut t);
    crate::tool::paint::wetpaint::oleo_no_quadro::no_evento(false);
    let p = ph2d_wet_paint::colorops::transfer::srgb255_to_linear(f64::from(CREME[2]));
    let r = canal_linear(&creme, &miolo, 2) / canal_linear(&branco, &miolo, 2);
    println!("[transmission] papel creme: azul creme ÷ branco {r:.3} × papel {p:.3}");
    // O brilho da superfície e a luz do relevo por cima não se tingem: a razão fica no papel ou acima (medido: ronda 2
    // 0,517; o vidro claro da ronda 3, 0,568 — a parte tingida encolheu face ao reflexo; × papel 0,456).
    assert!(
        (p..=1.3 * p).contains(&r),
        "a Transmission tinge pelo papel: {r} × {p}"
    );
}

/// ⭐ **O grão assado segue a lâmpada do relevo e o papel depois do traço** (sessão viva): mudar a lâmpada recompõe a
/// tinta pousada — a tela fica a da recomposição inteira, e não a de antes.
#[test]
fn o_grao_assado_segue_a_lampada_depois_do_traco() {
    crate::tool::paint::wetpaint::oleo_no_quadro::no_evento(true);
    let (mut t, _) = cena_papel_texturizado(true, 0.6, true);
    com_material(&mut t, 9, 1.0);
    t.reconcile_substrate();
    let antes = tela_final(&mut t);
    t.set_impasto_light_angle(40.0);
    t.reconcile_substrate();
    let seguiu = tela_final(&mut t);
    t.wet_recomposite_full();
    let inteira = tela_final(&mut t);
    crate::tool::paint::wetpaint::oleo_no_quadro::no_evento(false);
    assert_ne!(antes, seguiu, "a lâmpada mudou a tela");
    assert!(
        seguiu == inteira,
        "a tinta pousada seguiu a lâmpada (= a recomposição inteira)"
    );
}

/// A luminância linear ÷ a do papel e a saturação sRGB `(max − min)/max` médias nas amostras `m` (a régua do
/// `oraculo/transmissao_modos_regua.py`, sobre o miolo do produto).
fn luz_e_saturacao(px: &[u8], papel: &[u8], m: &[bool]) -> (f64, f64) {
    let lin = |p: &[u8]| {
        let c = [0, 1, 2]
            .map(|k| ph2d_wet_paint::colorops::transfer::srgb255_to_linear(f64::from(p[k])));
        0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
    };
    let (mut l, mut s, mut n) = (0.0, 0.0, 0.0);
    for ((p, q), _) in px
        .as_chunks::<4>()
        .0
        .iter()
        .zip(papel.as_chunks::<4>().0)
        .zip(m)
        .filter(|(_, v)| **v)
    {
        let mx = f64::from(*p[..3].iter().max().expect("rgb"));
        let mn = f64::from(*p[..3].iter().min().expect("rgb"));
        l += lin(p) / lin(q);
        s += (mx - mn) / mx.max(1.0);
        n += 1.0;
    }
    (l / n, s / n)
}

/// O FNV-1a de 64 bits dos bytes da tela.
fn fnv(px: &[u8]) -> u64 {
    px.iter().fold(0xcbf2_9ce4_8422_2325, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// ⭐ **Os dois modos da Transmission** (dono 28/10: «vou preferir o mais escuro do Blender, com segunda opção, sem
/// tirar a atual»), na cena do oráculo sobre o papel CREME. **Blender** casa com o Cycles (Principled Transmission
/// Weight, a mesma malha; `oraculo/README.md` §Transmission «Ronda 4»): L ÷ papel e saturação no miolo dentro da
/// tolerância MEDIDA — o Cycles lido NESTE miolo (`miolo_produto.pgm`; 0,5: 0,205 · 0,744; 1: 0,144 · 0,631).
/// **Clear glass** = a tela de `ad16f60b75` AO BYTE
/// (o hash da tela que o binário de `ad16f60b75` escreveu — `nos_tr1.ppm`/`nos_tr05.ppm` md5 iguais aos do modo 1), com o
/// contorno sub-célula de `9fba4165d8` (só o meio plano é escada; a lei do vidro ao bit continua em `ph2d-wet-paint`
/// `o_clear_glass_e_a_lei_de_antes_ao_bit`).
#[test]
fn o_modo_blender_casa_com_o_cycles_e_o_clear_glass_e_o_de_antes() {
    crate::tool::paint::wetpaint::oleo_no_quadro::no_evento(true);
    let (mut t, cob) = cena_papel_texturizado(true, 0.6, true);
    let (mut so_papel, _) = cena_papel_texturizado(true, 0.6, false);
    for x in [&mut t, &mut so_papel] {
        x.reconcile_substrate();
        poe_o_papel(x, CREME);
        x.reconcile_substrate();
    }
    crate::tool::paint::wetpaint::oleo_no_quadro::no_evento(false);
    let papel = tela_final(&mut so_papel);
    let (miolo, _) = miolo_e_papel_nu(&cob);
    let mut tela = |w: f64, claro: f64| {
        com_material(&mut t, 11, claro);
        com_material(&mut t, 9, w);
        let px = tela_final(&mut t);
        com_material(&mut t, 9, 0.0);
        com_material(&mut t, 11, 0.0);
        px
    };
    let (b05, b1) = (tela(0.5, 0.0), tela(1.0, 0.0));
    let (v05, v1) = (tela(0.5, 1.0), tela(1.0, 1.0));
    let n = |px: &[u8]| luz_e_saturacao(px, &papel, &miolo);
    let ((lb05, sb05), (lb1, sb1), (lv1, sv1)) = (n(&b05), n(&b1), n(&v1));
    println!(
        "[modos] Blender 0,5 L {lb05:.3} sat {sb05:.3} · 1 L {lb1:.3} sat {sb1:.3} (Cycles 0,124 · 0,801 | 0,076 · 0,705) · Clear glass 1 L {lv1:.3} sat {sv1:.3} · hash Clear glass 0,5 {:#x} · 1 {:#x}",
        fnv(&v05),
        fnv(&v1)
    );
    // O Cycles na malha de HOJE (re-medido 02/11c, `cura-cycles.md`: Principled Transmission, 256 amostras, este
    // miolo); o erro MEDIDO depois da cura do lobo da lâmpada (0,5: L +1 %, sat −4 %; 1: L +16 %, sat +0 %), com folga.
    // Dono 12/11 (o brilho picotado): o reflexo do ambiente passa a ter o antisserrilhado da lâmpada — o lobo mais
    // largo vê mais da janela do Estúdio: 0,5: L +8,6 %, sat −8,5 %; 1: L +27,6 %, sat −5,7 %. O produto do dono decide
    // contra o oráculo; as folgas sobem com a mesma margem (as de antes: 0,08 · 0,08 e 0,25 · 0,10).
    for (nome, l, s, (lc, sc), (tl, ts)) in [
        ("0,5", lb05, sb05, (0.124, 0.801), (0.15, 0.12)),
        ("1", lb1, sb1, (0.076, 0.705), (0.35, 0.15)),
    ] {
        assert!(
            (l / lc - 1.0).abs() <= tl,
            "Blender {nome}: L {l} × Cycles {lc}"
        );
        assert!(
            (s / sc - 1.0).abs() <= ts,
            "Blender {nome}: saturação {s} × Cycles {sc}"
        );
    }
    assert!(
        lb1 < 0.6 * lv1,
        "Blender mais escuro que o Clear glass: {lb1} × {lv1}"
    );
    assert_eq!(
        (fnv(&v05), fnv(&v1)),
        (CLEAR_GLASS_05, CLEAR_GLASS_1),
        "Clear glass ≠ a tela de ad16f60b75 (+ o contorno de 9fba4165d8)"
    );
}

/// O hash da tela de `ad16f60b75` (a lei do vidro claro) na cena de [`o_modo_blender_casa_com_o_cycles_e_o_clear_glass_e_o_de_antes`],
/// repintado em `9fba4165d8` (o contorno sub-célula só alisa a escada de um meio plano; antes `0x41f2_e6a6_379b_7fa6` ·
/// `0xaa52_ebb1_0843_041c`, bisseção 29/10: `9fba4165d8~1` dá os de antes, `9fba4165d8` e o HEAD da J6 estes).
/// Regravado 02/11 (era `0x23f9_aa15_b4a8_9263` · `0xc6ec_682c_aa7f_01a2`): a tinta da cena mudou só por lei nova —
/// degraus first-parent ea3daab14c · 5dd9f7b567+209a0e9c8e (anulam-se) · d92340b6f3 · 0170428784 · 0275929512 ·
/// 47ca63932e · 70869bf673 · 183d3d02e9 · ef3f836a86 · c77e40484c · 0f32fb6e75 · 5acb712ec6 · 7c8e08d16b (E10) ·
/// 032cda64a5; a tabela dos hashes está no commit. Regravado 02/11c (era `0xd780_ee7b_c62b_9982` ·
/// `0x65a9_16d3_3d87_ed2f`): o lobo da lâmpada passou ao microfacet físico (`oleo_real`, cura do Cycles).
/// Regravado 05/11 (era `0x92f6_8918_af19_c165` · `0xfc73_0fc0_9f11_537a`): o relevo sem degrau que o dono aprovou
/// (Bug #71); tirar o `meio_do_dente` de `height_at` devolve os de antes ao bit.
/// Regravado 09/11 (era `0xa302_b939_db67_365e` · `0x5180_1609_f93c_f777`): o papel morde a tinta pelo Relief × Tooth
/// (`60c3bfcb6f`, dono 08/11 «usar o Tooth»); bisseção: `bc6167c6f2` dá os de antes, `60c3bfcb6f` estes.
/// Regravado (B32.1 espectral) 12/11 (era `0xac49_1cf4_5ba7_e3fb` · `0x2ebc_2040_0011_d3fb`): o Wet mistura pela
/// lei espectral de 10 bandas (`00f3ca1751`, decisão 7a do dono); com `PH2D_WET_MISTURA=km` voltam os de antes.
/// Regravado 12/11 (era `0x6f35_8929_a88e_2278` · `0xb41f_d227_f619_2c77`): a luz única do traço (`7359c0095b`), o céu
/// lido inteiro com 4 fatias × 9 distâncias e o reflexo do ambiente com o antisserrilhado (o brilho picotado, dono).
/// Regravado 14/11 (B44.3, era `0x333b_fa93_94c2_2dc5` · `0xffc2_b09e_85b1_5561`): nada repintado — a régua passa a
/// ler o `run_full` (o ecrã), em vez de o acender outra vez com `apply_impasto_light`.
const CLEAR_GLASS_05: u64 = 0x66b6_7caa_0a27_e62a;
const CLEAR_GLASS_1: u64 = 0x948c_d3a3_e204_e3d3;
