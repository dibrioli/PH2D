//! **Bristles no Shape** (plano 27 W1): o tapete de cerdas do Wet Paint como o 30.º tipo do Shape. No
//! Digital e no Impasto ele é uma ponta de cerdas — pinta, e Count e Size mudam a ponta.

use super::super::media::PaintMedia;
use super::*;
use ph2d_painter_brush::TextureKind;

const N: u32 = 160;

fn pincel(media: PaintMedia, kind: TextureKind) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
    t.set_paint_media(media);
    t.set_brush_color_srgb8([20, 30, 120]);
    t.set_brush_shape_kind(kind.to_u8());
    t.set_wet_relogio_fixo(true);
    t
}

/// Um arco (meia volta): a tangente gira 180°, então Rake e Flow têm o que seguir.
fn arco(t: &mut PainterTool) -> Vec<u8> {
    let ponto = |k: usize| {
        let a = std::f32::consts::PI * k as f32 / 32.0;
        [80.0 - 50.0 * a.cos(), 110.0 - 50.0 * a.sin()]
    };
    t.on_canvas_pointer(cp(ponto(0), PointerPhase::Down));
    for k in 1..=32 {
        t.on_canvas_pointer(cp(ponto(k), PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp(ponto(32), PointerPhase::Up));
    for _ in 0..3 {
        t.on_tick(1000.0 / 60.0);
    }
    t.canvas_rgba.to_vec()
}

fn risca(t: &mut PainterTool) -> Vec<u8> {
    t.on_canvas_pointer(cp([20.0, 80.0], PointerPhase::Down));
    for k in 1..=28 {
        t.on_canvas_pointer(cp([20.0 + 4.0 * k as f32, 80.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([132.0, 80.0], PointerPhase::Up));
    for _ in 0..3 {
        t.on_tick(1000.0 / 60.0);
    }
    t.canvas_rgba.to_vec()
}

fn tinta(px: &[u8]) -> u64 {
    px.as_chunks::<4>()
        .0
        .iter()
        .map(|p| u64::from(255 - p[0].min(p[1]).min(p[2])))
        .sum()
}

/// No Digital e no Impasto a ponta de cerdas PINTA, não é a redonda, e os sliders Count e Size dela
/// mudam o traço. Controlo: a redonda (`None`) e o mesmo pincel sem mexer em nada pinta igual duas vezes.
#[test]
fn digital_e_impasto_pintam_com_a_ponta_de_cerdas() {
    for media in [PaintMedia::Digital, PaintMedia::Impasto] {
        let redonda = risca(&mut pincel(media, TextureKind::None));
        let cerdas = risca(&mut pincel(media, TextureKind::Bristles));
        assert_eq!(
            cerdas,
            risca(&mut pincel(media, TextureKind::Bristles)),
            "{media:?}: determinístico"
        );
        let (r, c) = (tinta(&redonda), tinta(&cerdas));
        assert!(c > 0, "{media:?}: as cerdas não pintaram");
        assert!(
            c < r,
            "{media:?}: cerdas {c} contra a redonda {r} — a ponta tem de ser porosa"
        );
        for (slot, valor) in [(0usize, 0.1f32), (2, 0.9)] {
            let mut t = pincel(media, TextureKind::Bristles);
            t.set_brush_shape_param_norm(slot, valor);
            assert_ne!(
                risca(&mut t),
                cerdas,
                "{media:?}: o slider {slot} não mudou a ponta"
            );
        }
    }
}

/// HR-15: o nome de CADA tipo do dropdown (agora 30) e os rótulos dos parâmetros dele vêm da tabela —
/// nenhuma chave crua no ecrã.
#[test]
fn cada_tipo_e_cada_parametro_tem_texto() {
    assert_eq!(TextureKind::COUNT, 30);
    assert_eq!(TextureKind::from_u8(29), TextureKind::Bristles);
    for k in (0..TextureKind::COUNT).map(TextureKind::from_u8) {
        assert_eq!(TextureKind::from_u8(k.to_u8()), k);
        let chaves = std::iter::once(k.name_key())
            .chain(ph2d_painter_brush::param_specs(k).iter().map(|s| s.label));
        for chave in chaves {
            assert_ne!(ph2d_i18n::tr(chave), chave, "{k:?}: «{chave}» sem texto");
        }
    }
}

/// ⭐ **Cada transformação do Shape age sobre a ponta** (dono 13/10: *«o primeiro Size ainda não tem efeito
/// … Rake parece que também não faz efeito. Veja com cuidado cada parâmetro da seção»*). Num arco, com o
/// Shape em Bristles e em Noise (o controlo: um procedural de sempre), nos três meios que evoluem ou herdam
/// a ponta: Size X · Size Y · Angle · Offset X · Offset Y · Follow Rake · Follow Flow · Jitter Rotate, cada
/// um sozinho, muda a tinta. Medido antes da cura: com Bristles os cinco primeiros e o Jitter Rotate
/// liam-se MORTOS nos três meios (`censo_dos_controlos::sonda_cada_controlo_com_as_cerdas_no_shape`).
#[test]
fn cada_transformacao_do_shape_muda_a_ponta_de_cerdas() {
    type Gesto = (&'static str, fn(&mut PainterTool));
    let gestos: [Gesto; 8] = [
        ("Size X", |t| t.set_brush_shape_size(0, 2.0)),
        ("Size Y", |t| t.set_brush_shape_size(1, 0.5)),
        ("Angle", |t| t.set_brush_shape_angle(37.0)),
        ("Offset X", |t| t.set_brush_shape_offset(0, 0.3)),
        ("Offset Y", |t| t.set_brush_shape_offset(1, -0.2)),
        ("Rake", |t| t.set_brush_shape_follow(1)),
        ("Flow", |t| t.set_brush_shape_follow(2)),
        ("Jitter Rotate", |t| t.set_brush_jitter_rotate(1.0)),
    ];
    let mut mortos = Vec::new();
    for media in [
        PaintMedia::Digital,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        for kind in [TextureKind::Bristles, TextureKind::Noise] {
            let base = arco(&mut pincel(media, kind));
            assert!(tinta(&base) > 0, "{media:?} {kind:?}: o arco não pintou");
            assert_eq!(
                base,
                arco(&mut pincel(media, kind)),
                "{media:?} {kind:?}: ruído"
            );
            for (nome, g) in gestos {
                let mut t = pincel(media, kind);
                g(&mut t);
                if arco(&mut t) == base {
                    mortos.push(format!("{media:?} {kind:?}: {nome}"));
                }
            }
        }
    }
    assert!(mortos.is_empty(), "sem efeito:\n{}", mortos.join("\n"));
}

/// Os nomes da seção Shape não se repetem: nenhum parâmetro de um tipo lê-se como Angle, Offset ou Size da
/// transformação, nem como outro parâmetro do mesmo tipo (o «Size» das cerdas era o segundo Size).
#[test]
fn nenhum_nome_da_secao_shape_se_repete() {
    let fixos = ["angle", "offset", "size"]
        .map(|k| ph2d_i18n::tr(&format!("panel.painter_layers.shape.{k}")));
    for k in (0..TextureKind::COUNT).map(TextureKind::from_u8) {
        let mut vistos: Vec<&str> = fixos.to_vec();
        for p in ph2d_painter_brush::param_specs(k) {
            let n = ph2d_i18n::tr(p.label);
            assert!(
                !vistos.contains(&n),
                "{k:?}: «{n}» repete-se na seção Shape"
            );
            vistos.push(n);
        }
    }
}

/// **Nada das cerdas se perde ao sair do Tuning** (dono 13/10): o Shape e o Grain mostram e aceitam os
/// quatro knobs nas MESMAS unidades do Tuning — digitar `1500` no Count, `3` na Bristle Strength, `0,5` na
/// Bristle Size e `0,05` no Felt dá EXACTAMENTE esses knobs, nos dois slots; um tipo comum continua na
/// pista `0..1` (CONTROLO: o Noise guarda o `0,3` que se digitou).
#[test]
fn as_cerdas_digitam_se_nas_unidades_do_tuning() {
    use ph2d_editor_core::tool::PanelEvent;
    let digitados = [1500.0, 3.0, 0.5, 0.05];
    for grao in [false, true] {
        let mut t = pincel(PaintMedia::Digital, TextureKind::Bristles);
        let ids = if grao {
            t.set_brush_texture_kind(TextureKind::Bristles.to_u8());
            crate::ids::PAINTER_BRUSH_TEXTURE_PARAMS
        } else {
            crate::ids::PAINTER_SHAPE_PARAMS
        };
        for (i, v) in digitados.into_iter().enumerate() {
            t.handle_panel_event(PanelEvent::SetValue(ids[i], v));
        }
        let slot = if grao {
            t.paint.brush.texture
        } else {
            t.paint.brush.shape
        };
        let k = ph2d_painter_brush::texture::bristle_knobs(&slot.params);
        assert_eq!(
            [k.bristle_count, k.bristle_strength, k.bristle_size, k.felt],
            digitados,
            "grão={grao}"
        );
        let e = ph2d_painter_brush::param_escala(TextureKind::Bristles, 0);
        assert_eq!((e.mostra(slot.params[0]), e.decimals()), (1500.0, 0));
    }
    let mut t = pincel(PaintMedia::Digital, TextureKind::Noise);
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_SHAPE_PARAMS[2],
        0.3,
    ));
    assert_eq!(t.paint.brush.shape.params[2], 0.3);
}

fn wet_round() -> ph2d_brush_presets::Preset {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    ph2d_brush_presets::Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == "Wet Round")
        .expect("o Wet Round de fábrica")
        .preset
}

/// ⭐ **O Wet Paint lê as cerdas do SHAPE** (plano 27 W2): o pincel de fábrica nasce com o Shape em Bristles
/// e as cerdas do Wet Round; o Bristle Count do Shape muda o traço; o knob `felt` do motor (agora escondido
/// do Tuning) já NÃO muda — o CONTROLO de que a fonte mudou de casa. A ponta lisa (Shape None) pinta outra
/// coisa.
#[test]
fn o_wet_paint_le_as_cerdas_do_shape() {
    use ph2d_wet_paint::tuning::Knob;
    let mut f = PainterTool::default();
    f.set_source(vec![255u8; (N * N * 4) as usize], N, N);
    f.set_paint_media(PaintMedia::WetPaint);
    let forma = f.paint.brush.shape;
    assert_eq!(
        forma.kind,
        TextureKind::Bristles,
        "o Wet Paint nasce com cerdas"
    );
    let k = ph2d_painter_brush::texture::bristle_knobs(&forma.params);
    assert_eq!(
        [k.bristle_count, k.bristle_strength, k.bristle_size, k.felt],
        [2000.0, 1.0, 2.0, 0.03]
    );
    let fresco = || {
        let mut t = pincel(PaintMedia::WetPaint, TextureKind::None);
        t.paint.brush.shape = forma;
        t
    };
    let base = arco(&mut fresco());
    assert_eq!(base, arco(&mut fresco()), "ruído");
    let mut menos = fresco();
    menos.set_brush_shape_param_norm(0, 0.1);
    assert_ne!(
        arco(&mut menos),
        base,
        "o Bristle Count do Shape não mudou o Wet Paint"
    );
    let mut felt = fresco();
    felt.paint.wetpaint.knobs.set(Knob::Felt, 0.15);
    assert_eq!(
        arco(&mut felt),
        base,
        "o felt do motor ainda manda nas cerdas"
    );
    let mut lisa = fresco();
    lisa.set_brush_shape_kind(TextureKind::None.to_u8());
    assert_ne!(arco(&mut lisa), base, "a ponta lisa pinta igual às cerdas");
}

/// ⭐ **Um Grain armado SUBSTITUI as cerdas do Shape** (plano 27 W2.4/D7, [27 §6.7]): com o grão, o
/// traço de Shape = Bristles é o MESMO, ao byte, que o de Shape = None — e o Count das cerdas não o muda.
/// Multiplicar os dois escurecia em dobro e apagava os pastéis (que pintam pelo grão).
///
/// **Mutação que sangra** (W5, a sobrevivente da prova das leis W1/W2): `grain_arg` em
/// `wetpaint/dab_route.rs` multiplicar o grão pelas cerdas.
#[test]
fn um_grain_armado_substitui_as_cerdas_do_shape() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let pastel = ph2d_brush_presets::Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == "Soft Pastel")
        .expect("o Soft Pastel de fábrica")
        .preset;
    let com_grao = |kind: TextureKind, count: Option<f32>| {
        let mut t = pincel(PaintMedia::WetPaint, TextureKind::None);
        assert!(t.wet_preset_aplicar(&pastel).is_empty());
        t.wet_preset_aplicar_volumes(&pastel.parametros.volumes);
        t.paint.wetpaint.tilt_on = false;
        assert!(
            t.paint.brush.grain_samples(false),
            "o pastel pinta pelo grão"
        );
        t.set_brush_shape_kind(kind.to_u8());
        if kind == TextureKind::Bristles {
            t.paint.brush.shape =
                crate::tool::paint::WetKnobs::DEFAULT.cerdas_no_shape(t.paint.brush.shape);
        }
        if let Some(c) = count {
            t.set_brush_shape_param_norm(0, c);
        }
        arco(&mut t)
    };
    let lisa = com_grao(TextureKind::None, None);
    assert!(
        lisa.as_chunks::<4>().0.iter().any(|p| p[0] < 250),
        "o pastel não pintou"
    );
    let diferem = |a: &[u8]| {
        a.chunks(4)
            .zip(lisa.chunks(4))
            .filter(|(x, y)| x != y)
            .count()
    };
    let cerdas = diferem(&com_grao(TextureKind::Bristles, None));
    assert_eq!(
        cerdas, 0,
        "com o Grain armado as cerdas do Shape ainda pintam ({cerdas} px)"
    );
    let count = diferem(&com_grao(TextureKind::Bristles, Some(0.1)));
    assert_eq!(
        count, 0,
        "com o Grain armado o Count das cerdas muda o traço ({count} px)"
    );
}

/// ⭐ **A migração dos presets** (plano 27 W2, D4): um preset de antes (as cerdas no mapa do motor, o Shape
/// vazio) chega com o Shape em Bristles e os MESMOS valores; gravado de novo, o mapa já não as traz e a
/// ida-e-volta guarda o Shape; um Shape None gravado depois continua None (a ponta lisa não vira cerdas).
#[test]
fn os_presets_antigos_chegam_com_as_cerdas_no_shape() {
    let p = wet_round();
    assert!(
        p.parametros.engine.contains_key("bristleCount"),
        "a fixtura é um preset de antes"
    );
    let mut t = pincel(PaintMedia::WetPaint, TextureKind::None);
    assert!(t.wet_preset_aplicar(&p).is_empty());
    let forma = t.paint.brush.shape;
    assert_eq!(forma.kind, TextureKind::Bristles);
    let k = ph2d_painter_brush::texture::bristle_knobs(&forma.params);
    assert_eq!(
        [k.bristle_count, k.bristle_strength, k.bristle_size, k.felt],
        [2000.0, 1.0, 2.0, 0.03]
    );
    let (gravado, _) = t.wet_preset_capturar(&p.parametros);
    for chave in ["felt", "bristleCount", "bristleStrength", "bristleSize"] {
        assert!(
            !gravado.engine.contains_key(chave),
            "o mapa ainda grava {chave}"
        );
    }
    let mut de_novo = p.clone();
    de_novo.parametros = gravado;
    let mut u = pincel(PaintMedia::WetPaint, TextureKind::None);
    assert!(u.wet_preset_aplicar(&de_novo).is_empty());
    assert_eq!(u.paint.brush.shape, forma, "a ida-e-volta perdeu o Shape");
    u.set_brush_shape_kind(TextureKind::None.to_u8());
    let (lisa, _) = u.wet_preset_capturar(&de_novo.parametros);
    let mut lisa_p = p;
    lisa_p.parametros = lisa;
    let mut v = pincel(PaintMedia::WetPaint, TextureKind::Bristles);
    assert!(v.wet_preset_aplicar(&lisa_p).is_empty());
    assert_eq!(
        v.paint.brush.shape.kind,
        TextureKind::None,
        "a ponta lisa virou cerdas"
    );
}

/// **O Follow existe sem textura no Shape** (dono 13/10): ele roda a pegada inteira do dab. Com o Shape vazio
/// e a ponta ACHATADA, o Rake muda o arco e a linha está viva; com a ponta redonda e nada que siga o dab, o
/// Rake não muda nada e a linha esmaece (`Dependente::FollowSemNada`) — esmaecida ⇔ inerte, nos três meios.
#[test]
fn o_follow_age_sem_textura_onde_ha_o_que_rodar() {
    use super::super::inercia::Dependente;
    for media in [
        PaintMedia::Digital,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        for achatada in [false, true] {
            let fresco = || {
                let mut t = pincel(media, TextureKind::None);
                t.set_brush_shape_kind(TextureKind::None.to_u8());
                if achatada {
                    t.set_brush_dab_flatten(0.6);
                }
                t
            };
            let base = arco(&mut fresco());
            let mut rake = fresco();
            rake.set_brush_shape_follow(1);
            let esmaecida = rake.inercias().inerte(Dependente::FollowSemNada);
            let mudou = arco(&mut rake) != base;
            assert_eq!(
                (mudou, esmaecida),
                (achatada, !achatada),
                "{media:?} achatada={achatada}: mudou={mudou} esmaecida={esmaecida}"
            );
        }
    }
}

/// O traço recto do Spacing contínuo: 20 → 140 px a meia altura, um quadro por evento, e a água assenta.
fn recta(t: &mut PainterTool) -> Vec<u8> {
    t.on_canvas_pointer(cp([20.0, 80.0], PointerPhase::Down));
    t.on_tick(1000.0 / 60.0);
    for k in 1..=24 {
        t.on_canvas_pointer(cp([20.0 + 5.0 * k as f32, 80.0], PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(cp([140.0, 80.0], PointerPhase::Up));
    for _ in 0..20 {
        t.on_tick(1000.0 / 60.0);
    }
    t.canvas_rgba.to_vec()
}

/// O CV da tinta ao longo das linhas do miolo (`|y − 80| ≤ 9`, x 50–110) — riscas ≈ 0, pontos ≫ 0.
fn cv_ao_longo(px: &[u8]) -> f64 {
    let mut cvs = Vec::new();
    for y in 71..=89usize {
        let linha: Vec<f64> = (50..110usize)
            .map(|x| {
                let p = &px[(y * N as usize + x) * 4..][..3];
                f64::from(255 - p[0].min(p[1]).min(p[2]))
            })
            .collect();
        let m = linha.iter().sum::<f64>() / linha.len() as f64;
        if m > 1.0 {
            let v = linha.iter().map(|a| (a - m) * (a - m)).sum::<f64>() / linha.len() as f64;
            cvs.push(v.sqrt() / m);
        }
    }
    cvs.iter().sum::<f64>() / cvs.len().max(1) as f64
}

/// ⭐ **Continuous Spacing é uma OPÇÃO e age nos dois meios** (dono 13/10: *«não parece funcionar»* —
/// era o Wet Paint, que não varria — e *«não pode substituir a antiga»*): desligada por omissão; ligada
/// pela caixa do Stroke, as cerdas a Spacing 0,25 riscam no Digital e no Wet Paint, sem esvaziar o traço.
/// A caixa age também na ponta lisa (a cápsula é uma lei da PONTA — dono 13/10: *«continuous deve estar
/// disponível para os outros modos»*; a igualdade com o Spacing pequeno é o portão de `tests/capsula.rs`).
#[test]
fn o_spacing_continuo_risca_no_digital_e_no_wet_paint() {
    let forma_wet = {
        let mut f = PainterTool::default();
        f.set_source(vec![255u8; (N * N * 4) as usize], N, N);
        f.set_paint_media(PaintMedia::WetPaint);
        f.paint.brush.shape
    };
    for media in [PaintMedia::Digital, PaintMedia::WetPaint] {
        let traco = |continuo: bool, kind: TextureKind| {
            let mut t = pincel(media, kind);
            if media == PaintMedia::WetPaint && kind == TextureKind::Bristles {
                t.paint.brush.shape = forma_wet;
            }
            t.set_brush_size_px(40.0);
            t.set_brush_spacing(0.25);
            // O Wet Paint nasce com ela ligada (a regra dos pincéis, prompt 51); o Digital, desligada.
            let por_omissao = media == PaintMedia::WetPaint;
            assert_eq!(
                t.paint.brush.continuous_spacing, por_omissao,
                "{media:?}: o Continuous por omissão"
            );
            if continuo != por_omissao {
                t.handle_panel_event(ph2d_editor_core::tool::PanelEvent::Click(
                    crate::ids::PAINTER_BRUSH_CONTINUOUS_SPACING,
                ));
                assert_eq!(
                    t.paint.brush.continuous_spacing, continuo,
                    "a caixa não troca"
                );
            }
            recta(&mut t)
        };
        let (off, on) = (
            traco(false, TextureKind::Bristles),
            traco(true, TextureKind::Bristles),
        );
        let (co, cn) = (cv_ao_longo(&off), cv_ao_longo(&on));
        let (to, tn) = (tinta(&off) as f64, tinta(&on) as f64);
        println!("{media:?}: CV {co:.3} → {cn:.3} · tinta {to} → {tn}");
        assert!(co > 0.3, "{media:?}: o controlo não pontilha ({co:.3})");
        assert!(cn < co / 3.0, "{media:?}: CV {co:.3} → {cn:.3}");
        assert!(tn > 0.9 * to, "{media:?}: o traço esvaziou ({to} → {tn})");
        assert_ne!(
            traco(false, TextureKind::None),
            traco(true, TextureKind::None),
            "{media:?}: a caixa não age na ponta lisa"
        );
    }
}
