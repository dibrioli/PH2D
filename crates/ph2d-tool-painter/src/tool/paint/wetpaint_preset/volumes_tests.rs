//! Os volumes do Pincel do Wet Paint (Size, Water, Dilution, Volume µm) e quem os LÊ.
//! Filho de [`super::tests`] por LOC (E10h).

use super::tests::{W, ferramenta, pinta, risca};
use super::*;
use ph2d_wet_paint::tuning::Knob;

#[test]
fn os_volumes_sao_o_que_o_painel_mostra() {
    let mut t = ferramenta();
    // E10: a Opacity saiu — os volumes vivos do Pincel são a Dilution e o Volume (µm), no `extra` pela chave.
    let mut extra = std::collections::BTreeMap::new();
    extra.insert("dilution".to_owned(), serde_json::json!(0.4));
    extra.insert("volume".to_owned(), serde_json::json!(12.5));
    t.wet_preset_aplicar_volumes(&Volumes {
        size_px: Some(40.0),
        opacity: Some(0.5),
        water: Some(0.25),
        extra,
        ..Volumes::default()
    });
    assert_eq!(t.paint.brush.radius_px, 20.0, "Size px é o DIÂMETRO");
    let k = &t.paint.wetpaint.knobs;
    assert_eq!((k.water, k.dilution(), k.volume_um()), (0.25, 0.4, 12.5));
    let v = t.wet_preset_volumes();
    assert_eq!(
        (v.size_px, v.opacity, v.water),
        (Some(40.0), None, Some(0.25)),
        "a `opacity` de antes da E10 não volta"
    );
    assert_eq!(
        (v.extra["dilution"].as_f64(), v.extra["volume"].as_f64()),
        (Some(0.4), Some(12.5))
    );
}

/// **O Volume do Wet Paint é a QUANTIDADE de pigmento** (dono 10/10; E10: a Opacity virou o Volume em µm): o traço a
/// 26 % do Volume pousa 26 % da massa do traço cheio, e fica mais claro. A régua é a MASSA (a lei, F3 §9), não o escuro
/// do Rebelle (revogado como referência a 31/10; a barra de escuro 0,18..0,36 já reprovava antes da E10 com 17/47).
/// Mutação que sangra: a receita ignorar o Volume (as duas massas iguais).
#[test]
fn a_opacity_do_wet_paint_e_a_quantidade_de_pigmento() {
    let mede = |frac: f64| {
        let mut t = ferramenta();
        let v = t.paint.wetpaint.knobs.volume_um();
        t.paint.wetpaint.knobs.set(Knob::Volume, v * frac);
        risca(&mut t, [23, 101, 179]);
        let g = &t
            .paint
            .wetpaint
            .session
            .as_ref()
            .expect("sessão viva")
            .engine
            .layers[0]
            .grid;
        let massa: f64 = g.susp.iter().chain(&g.sett).map(|&m| f64::from(m)).sum();
        let px = t.run_full().0;
        let escuro: u64 = px
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| u64::from(255 - p[0].min(p[1]).min(p[2])))
            .sum();
        (massa, escuro)
    };
    let ((m1, e1), (m26, e26)) = (mede(1.0), mede(0.26));
    eprintln!("massa {m26} / {m1} = {}; escuro {e26} / {e1}", m26 / m1);
    assert!(
        (m26 / m1 - 0.26).abs() < 0.02,
        "a 26 % do Volume a massa deu {m26} contra {m1} ({:.3})",
        m26 / m1
    );
    assert!(
        e26 < e1,
        "o traço com menos tinta não ficou mais claro: {e26} contra {e1}"
    );
}

/// O escuro do MIOLO do traço de [`pinta`] (a mediana na linha central, x 80…160) e a cor lá.
fn miolo_do_traco(px: &[u8]) -> (f64, [f64; 3]) {
    let mut v: Vec<(u32, [f64; 3])> = (80..160u32)
        .map(|x| {
            let y = (60.0 + 10.0 * (x as f32 * 0.03).sin()).round() as u32;
            let i = ((y * W + x) * 4) as usize;
            let c = [px[i], px[i + 1], px[i + 2]];
            (255 - u32::from(c[0].min(c[1]).min(c[2])), c.map(f64::from))
        })
        .collect();
    v.sort_by_key(|e| e.0);
    let m = v[v.len() / 2];
    (f64::from(m.0), m.1)
}

/// **Com a Opacity baixa a cor ACUMULA até à escolhida** (dono 10/10: «a cor nunca vai chegar à cor
/// escolhida no painel … o correto para clarear não seria usar pigment?»; escolhido por ele: como o Rebelle,
/// onde a tinta acumula até ao amarelo puro abaixo do próprio traço — `fixtures/rebelle_camadas/`). O mesmo
/// traço, Opacity 0,26, seco entre passadas (lido 1 s depois de cada uma): o miolo escurece a cada passada, passa o tecto da cor diluída
/// e fica no TOM do pincel. ⚠️ O CONTROLO está dentro: a lei recusada (a cor do pincel diluída para o papel,
/// o pigmento cheio) pára no tecto dela e reprova a mesma barra.
#[test]
fn com_a_opacity_baixa_a_cor_acumula_ate_a_escolhida() {
    const PASSADAS: usize = 12;
    let azul = [23.0, 101.0, 179.0];
    let tecto = 0.26 * (255.0 - 23.0);
    let camadas = |op: f32, cor: [u8; 3]| {
        let mut t = ferramenta();
        let v = t.paint.wetpaint.knobs.volume_um();
        t.paint.wetpaint.knobs.set(Knob::Volume, v * f64::from(op)); // E10: o Volume é a Opacity de antes
        let mut out = Vec::new();
        for _ in 0..PASSADAS {
            risca(&mut t, cor);
            out.push(miolo_do_traco(&t.canvas_rgba));
            assert!(
                t.route_brush_wetpaint_event(&ph2d_editor_core::tool::PanelEvent::Click(
                    crate::ids::PAINTER_WETPAINT_DRYCANVAS
                ))
            );
        }
        out
    };
    let barra = |v: &[(f64, [f64; 3])]| -> Result<(), String> {
        let escuros: Vec<f64> = v.iter().map(|e| e.0).collect();
        let (ultimo, tom) = (
            escuros[PASSADAS - 1],
            ph2d_wet_paint::colorops::diferenca_de_pigmento(v[PASSADAS - 1].1, azul),
        );
        let sobe = escuros.windows(2).all(|w| w[1] >= w[0] - 1.0);
        if sobe && ultimo >= 1.5 * tecto && tom <= 0.15 {
            Ok(())
        } else {
            Err(format!(
                "escuro por passada {escuros:?} (tecto da diluída {tecto:.0}, barra {:.0}); tom {tom:.3}",
                1.5 * tecto
            ))
        }
    };
    let diluida = azul.map(|c| (255.0 - 0.26 * (255.0 - c)).round() as u8);
    let c = camadas(1.0, diluida);
    eprintln!(
        "[acumula] controlo (diluída): {:?}",
        c.iter().map(|e| e.0).collect::<Vec<_>>()
    );
    assert!(
        barra(&c).is_err(),
        "CONTROLO: a cor diluída passou a barra — a régua não discrimina"
    );
    let p = camadas(0.26, [23, 101, 179]);
    eprintln!(
        "[acumula] produto: {:?}",
        p.iter().map(|e| e.0).collect::<Vec<_>>()
    );
    if let Err(e) = barra(&p) {
        panic!("a Opacity 0,26 não acumula até à cor escolhida: {e}");
    }
}

/// O painel mostra um volume só quando o traço o LÊ. A Opacity (a força) não tem leitor no Wet Paint
/// (dono, 2026-08-12: escondida pela mesma medição): quando a W5 lhe der um, este gate fica vermelho e
/// o volume Opacity entra no painel. O Water e o Size são lidos — o controlo do gesto.
#[test]
fn a_opacidade_nao_tem_leitor_no_wet_paint() {
    let traco = |f: &dyn Fn(&mut PainterTool)| {
        let mut t = ferramenta();
        f(&mut t);
        pinta(&mut t)
    };
    let fraca = traco(&|t| t.set_brush_strength(0.25));
    let forte = traco(&|t| t.set_brush_strength(0.95));
    assert_eq!(
        fraca, forte,
        "a força ganhou leitor no Wet Paint: o volume Opacity entra no painel"
    );
    // E10: no Pincel a receita lê a Dilution (o Water ficou para a ferramenta Wet).
    let seca = traco(&|t| t.paint.wetpaint.knobs.set(Knob::Dilution, 0.95));
    assert_ne!(seca, forte, "controlo: a Dilution é lida");
    let grossa = traco(&|t| t.set_brush_size_px(20.0));
    assert_ne!(grossa, forte, "controlo: o Size é lido");
}
