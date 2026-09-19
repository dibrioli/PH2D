//! ⭐⭐⭐ **O REBORDO CLARO DE UM PIXEL — a sonda** (report do dono de 2026-09-19, com foto:
//! *«toda forma apresenta uma falsa outline branca de 1 pixel»*).
//!
//! # ⭐⭐⭐ A RÉGUA É LIVRE DE MODELO: num pré-multiplicado, `rgb ≤ alfa`
//!
//! A imagem do modelador é entregue como **`Rgba8` + `AlphaPremultiplied`**, ou seja `rgb = sRGB(C)·a`
//! com `C ∈ [0,1]` ⇒ **nenhum canal pode passar do alfa**. Os dois motores gravavam `sRGB(C·a)`, e
//! como a curva sRGB é côncava isso **passa** o alfa em toda cobertura parcial: um branco a meio
//! pixel gravava `188` com alfa `128`.
//!
//! ⚠️⚠️ **A 1.ª redacção desta sonda media outra coisa e envelheceu com a cura:** ela desfazia
//! `sRGB(C·a)` para recuperar `C` e comparava com `sRGB(C)·a`. Isso *supunha a lei defeituosa*, logo
//! depois da cura ela passou a desfazer a conta errada e continuou a imprimir `+15,8` sobre uma
//! imagem correcta. *Uma régua escrita a partir da fórmula do defeito mede o defeito e mais nada.*
//! ⇒ a régua que fica é a **invariante do formato**, que não precisa de saber que lei produziu o
//! pixel.
//!
//! # ⚠️ E ela só vale onde NÃO há luz aditiva
//!
//! Um pixel pode legitimamente ter `rgb > alfa`: é a luz que a peça devolve ao chão, que **soma sem
//! tapar**. ⇒ a sonda corre **sem chão** (`ground = None`), e é isso que torna a banda da silhueta
//! uma população de cobertura pura.

use super::borda_tests::{banda, camara, quadro};
use super::device_tests::{LH, LW, cpu_ociosa_pct};

/// A cena do report: as três bolas e a barra escura.
const CENA: u32 = 36;

/// ⭐⭐⭐ **QUANTOS CANAIS PASSAM DO ALFA, E POR QUANTO** — a assinatura do rebordo, em bytes.
#[test]
#[ignore = "medição — precisa de GPU"]
fn quantos_canais_passam_do_alfa() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let doc = crate::smoke::scene(CENA);
    let reg = crate::smoke::sampled_registry();
    let Some(p) = quadro(
        t,
        &doc,
        &reg,
        &camara(0.0),
        true,
        crate::gpu_frame::Sonda::default(),
    ) else {
        println!("a placa recusa esta peça — saltado");
        return;
    };
    let (w, h) = (LW as usize, LH as usize);
    let b = banda(&p.rgba, w, h);
    let mut passam = 0usize;
    let mut canais = 0usize;
    let mut pior = 0i32;
    // ⭐ **O CONTROLO é a lei de ONTEM recalculada sobre estes mesmos pixels** — `sRGB(C)·a` é
    // invertível (`C = linear(byte/a)`), logo a sonda pode dizer o que este quadro teria gravado
    // antes da cura. *Uma sonda que só sabe dizer «zero» não distingue uma cura de uma régua cega.*
    let (mut passavam, mut pior_antes) = (0usize, 0i32);
    for &i in &b {
        let a = i32::from(p.rgba[i * 4 + 3]);
        let af = f32::from(p.rgba[i * 4 + 3]) / 255.0;
        for k in 0..3 {
            canais += 1;
            let byte = p.rgba[i * 4 + k];
            let excesso = i32::from(byte) - a;
            if excesso > 0 {
                passam += 1;
                pior = pior.max(excesso);
            }
            if af > 0.0 {
                let cor =
                    ph2d_color::srgb::srgb_to_linear_unit((f32::from(byte) / 255.0 / af).min(1.0));
                let ontem = ph2d_color::srgb::linear_to_srgb_byte(cor * af);
                let e = i32::from(ontem) - a;
                if e > 0 {
                    passavam += 1;
                    pior_antes = pior_antes.max(e);
                }
            }
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let pct = passam as f32 / canais.max(1) as f32 * 100.0;
    #[allow(clippy::cast_precision_loss)]
    let pct_antes = passavam as f32 / canais.max(1) as f32 * 100.0;
    println!(
        "\n  cena {CENA} · banda {} px · canais {canais} · load {} · ociosa {:.0} %",
        b.len(),
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim(),
        cpu_ociosa_pct()
    );
    println!(
        "  com a lei de ONTEM ... {passavam} ({pct_antes:.1} %) · pior excesso {pior_antes} bytes"
    );
    println!("  com a lei de HOJE .... {passam} ({pct:.1} %) · pior excesso {pior} bytes");
}

/// ⭐⭐⭐ **A PROVA VISUAL, COMPOSTA PELA LEI DO COMPOSITOR QUE FOI MEDIDA** — dois `.ppm` com o
/// mesmo quadro sobre o mesmo cinzento, um por lei.
///
/// ⛔⛔ **Ela existe porque a FOTO não serve de A/B aqui**, e isso foi medido: a cena `=36` tem o
/// prato a rodar, logo duas fotografias apanham a peça em sítios diferentes; e o detector de
/// rebordo mais simples apanha a **grelha do canvas** (`104` sobre `98`) e o **texto do painel**.
/// *Uma prova visual sobre uma cena que se move não é uma prova.*
///
/// ⭐ A composição usada é a que o `VelloPass` foi medido a fazer: `img + fundo·(1−a)`, em bytes.
#[test]
#[ignore = "medição — precisa de GPU"]
fn desenha_as_duas_leis_sobre_o_mesmo_cinzento() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let doc = crate::smoke::scene(CENA);
    let reg = crate::smoke::sampled_registry();
    let Some(p) = quadro(
        t,
        &doc,
        &reg,
        &camara(0.0),
        true,
        crate::gpu_frame::Sonda::default(),
    ) else {
        println!("a placa recusa esta peça — saltado");
        return;
    };
    const FUNDO: f32 = 110.0;
    let (w, h) = (LW as usize, LH as usize);
    // ⭐ O recorte é centrado onde o defeito era PIOR — o pixel em que a lei de ontem mais passava
    // do alfa. *Um recorte no meio da lista cai onde o contraste é baixo e não mostra nada.*
    let b = banda(&p.rgba, w, h);
    let centro = b
        .iter()
        .copied()
        .max_by_key(|&i| {
            let a = f32::from(p.rgba[i * 4 + 3]) / 255.0;
            if a <= 0.0 {
                return 0i32;
            }
            (0..3)
                .map(|k| {
                    let byte = f32::from(p.rgba[i * 4 + k]);
                    let cor = ph2d_color::srgb::srgb_to_linear_unit((byte / 255.0 / a).min(1.0));
                    i32::from(ph2d_color::srgb::linear_to_srgb_byte(cor * a))
                        - i32::from(p.rgba[i * 4 + 3])
                })
                .max()
                .unwrap_or(0)
        })
        .expect("a banda não é vazia");
    let (cx, cy) = (
        (centro % w).clamp(60, w - 60),
        (centro / w).clamp(60, h - 60),
    );
    let dir = std::env::var("PH2D_PREMUL_DUMP").unwrap_or_else(|_| "/tmp".to_string());
    for (nome, ontem) in [("hoje", false), ("ontem", true)] {
        let mut ppm = String::from("P3\n120 120\n255\n");
        for y in (cy - 60)..(cy + 60) {
            for x in (cx - 60)..(cx + 60) {
                let i = y * w + x;
                let a = f32::from(p.rgba[i * 4 + 3]) / 255.0;
                for k in 0..3 {
                    let mut byte = f32::from(p.rgba[i * 4 + k]);
                    if ontem && a > 0.0 {
                        // A lei de ONTEM, recalculada: `sRGB(C·a)` a partir de `sRGB(C)·a`.
                        let cor =
                            ph2d_color::srgb::srgb_to_linear_unit((byte / 255.0 / a).min(1.0));
                        byte = f32::from(ph2d_color::srgb::linear_to_srgb_byte(cor * a));
                    }
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let saida = (byte + FUNDO * (1.0 - a)).clamp(0.0, 255.0) as u8;
                    ppm.push_str(&format!("{saida} "));
                }
            }
            ppm.push('\n');
        }
        let caminho = format!("{dir}/premul_{nome}.ppm");
        std::fs::write(&caminho, ppm).expect("escreve o recorte");
        println!("  {caminho}");
    }
    println!("  recorte centrado em ({cx}, {cy}), 120x120");
}

/// ⭐⭐⭐ **NENHUM CANAL DA SILHUETA PASSA DO ALFA** — a invariante do formato, no caminho do
/// produto.
///
/// ⚠️ **O CONTROLO vem primeiro e tem de REPRODUZIR o defeito:** a lei de ontem é recalculada
/// sobre os mesmos pixels, e se ela deixar de passar do alfa então a fixtura já não contém o
/// fenómeno e a barra abaixo não afirma nada.
#[test]
#[ignore = "precisa de GPU"]
fn nenhum_canal_da_silhueta_passa_do_alfa() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let doc = crate::smoke::scene(CENA);
    let reg = crate::smoke::sampled_registry();
    let Some(p) = quadro(
        t,
        &doc,
        &reg,
        &camara(0.0),
        true,
        crate::gpu_frame::Sonda::default(),
    ) else {
        println!("a placa recusa esta peça — saltado");
        return;
    };
    let (w, h) = (LW as usize, LH as usize);
    let b = banda(&p.rgba, w, h);
    assert!(
        b.len() > 1_000,
        "a banda da silhueta tem {} pixels — a fixtura não tem contorno",
        b.len()
    );
    let (mut passam, mut passavam) = (0usize, 0usize);
    for &i in &b {
        let a = i32::from(p.rgba[i * 4 + 3]);
        let af = f32::from(p.rgba[i * 4 + 3]) / 255.0;
        for k in 0..3 {
            let byte = p.rgba[i * 4 + k];
            if i32::from(byte) > a {
                passam += 1;
            }
            if af > 0.0 {
                let cor =
                    ph2d_color::srgb::srgb_to_linear_unit((f32::from(byte) / 255.0 / af).min(1.0));
                if i32::from(ph2d_color::srgb::linear_to_srgb_byte(cor * af)) > a {
                    passavam += 1;
                }
            }
        }
    }
    assert!(
        passavam > 1_000,
        "o CONTROLO (a lei de ontem sobre estes mesmos pixels) passou do alfa em {passavam} canais \
         — a fixtura deixou de conter o rebordo, e a barra abaixo passa por vácuo"
    );
    assert_eq!(
        passam, 0,
        "{passam} canais da silhueta passam do alfa — a imagem deixou de ser um pré-multiplicado \
         válido, e o compositor pinta isso como um rebordo CLARO à volta de toda forma"
    );
}
