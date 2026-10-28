//! **Transparency** (dono 27/10, `1 − Alpha` do Principled) sobre o papel TEXTURIZADO: o gate do produto (a tela
//! final, composite + a luz do relevo) e a sonda do oráculo (`docs/Painter/49_realshader/oraculo/README.md`
//! §Transparency).

use super::*;

/// A tinta pousada com a Transparency `v` (a tela recompõe-se inteira).
fn com_transparencia(t: &mut PainterTool, v: f64) {
    com_material(t, 8, v);
}

/// A tinta POUSADA com o peso `k` do material da célula a `v` — 8 Transparency, 9 Transmission, 10 Relevo do papel —
/// em todas as células de todas as camadas, e a tela recomposta: o mesmo traço como se o pincel o tivesse pintado com
/// o knob (W3b ronda 3: os três saíram do cartão, são só do pincel; `material_do_pincel_tests` prova que o pincel os
/// pousa). Assim as medidas comparam a MESMA tinta.
pub(super) fn com_material(t: &mut PainterTool, k: usize, v: f64) {
    let s = t.paint.wetpaint.session.as_mut().expect("sessão");
    s.bring_home();
    for l in &mut s.engine.layers {
        for m in ph2d_wet_paint::grid::material::garante(&mut l.grid) {
            let alvo = match k {
                8 => &mut m.transparencia,
                9 => &mut m.transmissao,
                10 => &mut m.relevo_papel,
                _ => panic!("peso {k}"),
            };
            *alvo = v as f32;
        }
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
    // ⭐ O Relevo do papel 1 = a tela de antes de `ca26a7738a` na régua do §BUG-papel (o dente na tinta grossa ÷ no
    // papel nu, absoluto: 0,67 com o óleo ligado) — e o grão também nos BRILHOS da tinta.
    let abs = dente_na_tela(&com, &sem, &miolo) / nu_abs;
    let brilhos = dente_nos_brilhos(&com, &sem, &miolo) / nu_abs;
    println!(
        "[transmission] Relevo do papel 1: miolo ÷ nu {abs:.3} (antes de ca26a7738a 0,67) · nos brilhos {brilhos:.3}"
    );
    assert!(
        (0.55..=0.8).contains(&abs),
        "Relevo do papel 1 ≈ a tela de antes: {abs} × 0,67"
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
    // A Transparency (Alpha, 8), a Transmission (9) e o Relevo do papel (10), um de cada vez, os outros a 0.
    for (nome, k, v) in [
        ("t0", 8, 0.0),
        ("t05", 8, 0.5),
        ("t1", 8, 1.0),
        ("tr05", 9, 0.5),
        ("tr1", 9, 1.0),
        ("rp05", 10, 0.5),
        ("rp1", 10, 1.0),
    ] {
        com_material(&mut t, k, v);
        grava_ppm(
            &dir.join(format!("nos_{nome}.ppm")),
            &tela_final(&mut t),
            OW,
            OH,
        );
        com_material(&mut t, k, 0.0);
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
