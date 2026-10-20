//! **Família `grafite`** — J3, catálogo B (a série de grafite): Graphite 8B · 2B · HB · 2H (B-5 Mechanical 0.5 fundido no Hard Round: `exames/grafite.md`).
//! Escreva aqui as receitas e os testes da família; nada fora deste ficheiro (`docs/Painter/Pinceis/04_BANCADA.md` §9).
//! O 8B é base PARTILHADA (`receitas_de_fabrica::GRAPHITE_8B`: os lápis nascem dele): mudá-lo muda a família `lapis`.

use super::prelude::*;

pub const WET: &[Receita] = &[
    // Pencil › Graphite 8B: sem água, o grão do papel a abrir na orla. Prompt 35 (14/10), contra o Rebelle
    // «Graphite Pencil/8B» a 20 px vistos (`pastel_regua.py`, nota 4,21 → 1,15): o Noise 0,4 era a mancha de
    // ~640 px do Soft Pastel (5 px a y = 40, 8 a 450) — Noise 32; papel 0,6 e carga 125 (o miolo cheio do 8B).
    // Com o arrasto na ponta seca (o motor deixou de o zerar): o portão das mídias reprovava a Textura (7,65
    // contra 9,09 ± 0,66) e o arrasto só a BAIXA (0,16 → 3,25) — não são as estrias. Varridos 1 368 + 594 na
    // bancada: perfil Linear, jitter 0,1 (a borda ondulada do 8B), arrasto 0,05, papel 0,65 — nota 1,25 → 0,64.
    // J3 (`exames/grafite.md`): a base partilhada + a ORLA que ondula (o dual, ver [`ORLA_DO_8B`]), a carga 150 e o
    // papel 0,55 — a cobertura a 40 px 0,85 (Rebelle 0,86), a Textura da mídia 8,76 (9,09).
    // A base `GRAPHITE_8B` fica como estava (os lápis da família `lapis` leem-na).
    Receita {
        pigment: 150.0,
        gate: 0.55,
        pincel: &[("falloff", "\"Linear\""), ("jitter", "0.1"), ("dual", ORLA_DO_8B)],
        ..GRAPHITE_8B
    },
    // ⭐ Os do prompt 53 (doc 53 §8): cada um o 8B inteiro (`nasce_de`) + os overrides; o tamanho é o do preset do Rebelle.
    // J3: os quatro com a orla do dual; HB e 2H com o grão FINO e FRACO (Size 200 = 1,3 px, contraste 0,3 — o Rebelle
    // tem o fino fraco e a onda larga; um Noise só não dava as duas, 03_MOTORES §7.2); o 2H CLARO: a carga 24 dá-lhe o
    // tom da cor de fábrica dele no Rebelle (cinza 149 contra o 115 do HB: 106/140 = 0,76 da tinta) — com a MESMA cor,
    // o 2H do Rebelle é o HB (d 0,30 na família dele) e o eixo de dureza não se via (d 0,23 entre os nossos). A carga mais
    // leve pede a curva da carga 1,5 (a 2,0 a rampa da pressão saía a 0,18 da do Rebelle; a 1,5, 0,10).
    Receita {
        pincel: &[("falloff", "\"Linear\""), ("jitter", "0.1"), ("dual", ORLA_DO_LAPIS), ("texture", GRAO_FINO_DO_2H)],
        grao: None,
        ..lapis("Graphite", "Graphite 2H", 24.0, 0.18, GRAO_DURO, extras_do_lapis!(0.5, 1.5))
    },
    Receita {
        pincel: &[("falloff", "\"Linear\""), ("jitter", "0.1"), ("dual", ORLA_DO_LAPIS), ("texture", GRAO_FINO_FRACO)],
        grao: None,
        ..lapis("Graphite", "Graphite HB", 40.0, 0.18, GRAO_DURO, extras_do_lapis!(0.5, 2.0))
    },
    // O 2B: o grão de 2 px (Size 128), o papel a 0,4, a carga 46 e a curva do tamanho 0,4 (com a orla a 0,25 a rampa da
    // pressão saía a 0,19 da do Rebelle, `a_pressao_de_cada_preset_e_a_da_sua_origem`; a 0,4, 0,09).
    Receita {
        pincel: &[("falloff", "\"Linear\""), ("jitter", "0.1"), ("dual", ORLA_DO_LAPIS), ("texture", GRAO_DO_2B)],
        grao: None,
        ..lapis("Graphite", "Graphite 2B", 46.0, 0.4, GRAO_MACIO, extras_do_lapis!(0.4, 2.0))
    },
];

/// ⭐ **A ORLA do lápis** (J3): a 2.ª ponta (dual, Multiply) a 0,45 do diâmetro, espalhada 0,2 e 0,3 ATRAVÉS, 3 marcas —
/// recorta a borda numa onda do tamanho da ponta. O Rebelle tem-na (`edge_wave_jitter` 0,5 + a forma oval girada 180°
/// por marca): na borda do recto de 40 px a onda é 0,057–0,065 da largura com o comprimento de correlação 6–7 px;
/// a nossa era 0,026–0,034 com 1–2 px (só o grão) e passa a ~0,06 com 8 px; a 200 px ~0,056 (Rebelle 0,07–0,08).
/// A 0,6 com 2 marcas a onda era a mesma, mas a 5 px as marcas de 3 px correlacionavam o colar (ac 0,43; Rebelle 0,24):
/// a 0,45 com 3, ac 0,23. A revisão cega do 8B (J3) apontava a borda «aos caroços, ondulada» do original como a diferença maior.
const ORLA_DO_LAPIS: &str = r#"{"angulo": 0, "atraves": 0.3, "contagem": 3, "dureza": 0.3, "escala": 0.45, "espalhar": 0.2, "forma": {"angle_deg": 0, "flow": false, "kind": "None", "mapping": "ViewPlane", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [1.0, 1.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}, "modo": 0, "passo": 0.25, "rotacao": 0.0}"#;

/// A orla do 8B: duas marcas a 0,7 do diâmetro, 0,1 espalhadas e 0,2 através — com a dos outros (0,45 × 3) o 8B DEITADO
/// perdia o grão (textura através ÷ de pé 0,78, Rebelle 1,01 ± 0,15, `a_caneta_deitada`); assim 0,90, ao longo 1,33
/// (1,24; a 0,8 × 1, 1,38 saía do vale), a onda 0,047 e o colar a 5 px 0,076 · ac 0,50 (Rebelle 0,072 · 0,50).
const ORLA_DO_8B: &str = r#"{"angulo": 0, "atraves": 0.2, "contagem": 2, "dureza": 0.3, "escala": 0.7, "espalhar": 0.1, "forma": {"angle_deg": 0, "flow": false, "kind": "None", "mapping": "ViewPlane", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [1.0, 1.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}, "modo": 0, "passo": 0.25, "rotacao": 0.0}"#;

/// O grão do 2H: o fino e fraco ainda mais fraco (contraste 0,2) — a Textura da mídia 4,70 (Rebelle 4,98; a 0,3, 6,58).
const GRAO_FINO_DO_2H: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.2, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [200.0, 200.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

/// O grão fino e fraco dos duros: Noise `Tiled` Size 200 (1,28 px), contraste 0,3 — o colar a 5 px do HB 0,079 (Rebelle
/// 0,066; era 0,130) e a autocorrelação 0,29 (0,24; era 0,78).
const GRAO_FINO_FRACO: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.3, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [200.0, 200.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

/// O grão do 2B: Noise `Tiled` Size 128 (2 px), contraste 0,5.
const GRAO_DO_2B: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [128.0, 128.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

pub const MEIOS: &[ReceitaMeio] = &[];

// O raio é o do preset à largura SECA da foto pela lei PRÓPRIA dele (`ferramentas/avalia_53.py`).
/// ⚠️ A cobertura do 2H é a da foto (0,595, pintada com o azul da bateria) × o tom da cor de FÁBRICA dele no Rebelle
/// sobre a do HB (cinza 149 contra 115: 106 / 140) — com a mesma cor o 2H do Rebelle é o HB (o eixo de dureza do Rebelle
/// mora na cor de omissão, que os nossos presets não levam; ver a receita).
const COBERTURA_DO_2H: f64 = 0.595 * 106.0 / 140.0;
#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Graphite 8B", foto: "pencil", raio: 15.0, metricas: &[(Cobertura, 0.798), (Textura, 9.093), (Borda, 0.320), (Orla, 7.0)] },
    Midia { preset: "Graphite 2H", foto: "pencil2h", raio: 14.40, metricas: &[(Cobertura, COBERTURA_DO_2H), (Textura, 4.983), (Borda, 0.321), (Orla, 10.0), (Cruzamento, 0.599), (Largura, 24.0)] },
    Midia { preset: "Graphite HB", foto: "pencilhb", raio: 15.31, metricas: &[(Cobertura, 0.561), (Textura, 6.153), (Borda, 0.325), (Orla, 11.0), (Cruzamento, 0.583), (Largura, 24.0)] },
    Midia { preset: "Graphite 2B", foto: "pencil2b", raio: 14.40, metricas: &[(Cobertura, 0.544), (Textura, 13.901), (Borda, 0.350), (Orla, 11.0), (Cruzamento, 0.589), (Largura, 24.0)] },
];

// ---- Os portões da família (J3, `docs/Painter/Pinceis/exames/grafite.md`) ----

#[cfg(test)]
mod portoes {
    use crate::as_leis_da_escala::tamanhos;
    use crate::exame_bancada::{Fonte, Pincel, pinceis};
    use crate::o_grao_do_lapis::cv_do_recto;

    /// A cobertura (0..1) de cada píxel da tela 1000 × 640 sobre o papel branco (`leis_escala.py`).
    fn cobertura(px: &[u8]) -> Vec<f64> {
        px.chunks_exact(4)
            .map(|c| (0..3).map(|k| 255.0 - f64::from(c[k])).fold(0.0, f64::max) / 255.0)
            .collect()
    }

    /// ⭐ **A ONDA da borda** do recto horizontal em `y` (colunas 148..852, ±`h` linhas): o desvio-padrão das duas bordas
    /// (a ¼ do p90 dos picos das colunas, sem a recta de tendência) ÷ a largura mediana, e o comprimento de correlação
    /// delas (o 1.º desfasamento em que a autocorrelação cai abaixo de ½). O Rebelle nos 4 grafites: 0,057–0,065 com
    /// 6–7 px no recto de 40 px; o ruído do grão sozinho dá ~0,03 com 1–2 px.
    pub(super) fn onda(c: &[f64], y: usize, h: usize) -> (f64, usize) {
        let col = |x: usize| -> Vec<f64> { (y - h..=y + h).map(|yy| c[yy * 1000 + x]).collect() };
        let picos: Vec<f64> = (148..852).map(|x| col(x).into_iter().fold(0.0, f64::max)).collect();
        let mut p = picos.clone();
        p.sort_by(f64::total_cmp);
        let limiar = 0.25 * p[(0.9 * (p.len() - 1) as f64) as usize];
        let (mut t, mut b) = (Vec::new(), Vec::new());
        for x in 148..852 {
            let v = col(x);
            if let (Some(a), Some(z)) = (v.iter().position(|&q| q >= limiar), v.iter().rposition(|&q| q >= limiar)) {
                t.push(a as f64);
                b.push(z as f64);
            }
        }
        let mut w: Vec<f64> = t.iter().zip(&b).map(|(a, z)| z - a + 1.0).collect();
        w.sort_by(f64::total_cmp);
        let largura = w[w.len() / 2];
        let mede = |e: &[f64]| -> (f64, usize) {
            let n = e.len() as f64;
            let xm = (n - 1.0) / 2.0;
            let em = e.iter().sum::<f64>() / n;
            let sxy: f64 = e.iter().enumerate().map(|(i, v)| (i as f64 - xm) * (v - em)).sum();
            let sxx: f64 = (0..e.len()).map(|i| (i as f64 - xm).powi(2)).sum();
            let r: Vec<f64> = e.iter().enumerate().map(|(i, v)| v - em - sxy / sxx * (i as f64 - xm)).collect();
            let var = r.iter().map(|v| v * v).sum::<f64>() / n;
            let ac = |k: usize| {
                r.windows(k + 1).map(|q| q[0] * q[k]).sum::<f64>() / (n - k as f64) / var.max(1e-12)
            };
            (var.sqrt(), (1..80).find(|&k| ac(k) < 0.5).unwrap_or(80))
        };
        let ((st, lt), (sb, lb)) = (mede(&t), mede(&b));
        ((st + sb) / 2.0 / largura, (lt + lb) / 2)
    }

    /// O pincel `nome` de fábrica com o campo `campo` do pincel trocado por `valor` (JSON) — a mutação dos portões.
    fn com_campo(nome: &str, campo: &str, valor: serde_json::Value) -> Pincel {
        let mut p = pinceis(nome).remove(0);
        if let Fonte::Preset { preset, .. } = &mut p.fonte {
            preset.parametros.brush.insert(campo.to_owned(), valor);
        }
        p
    }

    /// As medidas da cena `tamanhos`: (onda e correlação a 40 px, cv e ac a 5 px, cobertura do miolo a 40 px).
    fn medidas(p: &Pincel) -> ((f64, usize), (f64, f64), f64) {
        let px = tamanhos(p);
        let c = cobertura(&px);
        let (cv5, ac5, _) = cv_do_recto(&px, 70, 40);
        (onda(&c, 190, 70), (cv5, ac5), cv_do_recto(&px, 190, 70).2)
    }

    /// A onda mínima da borda: o Rebelle 0,057–0,065; o grão sozinho (sem a orla) dá 0,022–0,034; o nosso 8B 0,045
    /// (a orla dele é mais larga: a caneta deitada) e os outros 0,058–0,060.
    const ONDA_MIN: f64 = 0.04;
    const GRAFITES: [&str; 4] = ["Graphite 2H", "Graphite HB", "Graphite 2B", "Graphite 8B"];
    /// O colar a 5 px do Rebelle (cv · ac, `fixtures/leis_escala_rebelle.json` e `o_grao_do_lapis::AC5_DO_REBELLE`).
    const FINO_DO_REBELLE: [(&str, f64, f64); 2] = [("Graphite 2H", 0.061, 0.24), ("Graphite HB", 0.066, 0.24)];
    /// O tom do 2H sobre o do HB: a tinta das cores de fábrica do Rebelle (cinza 149 e 115 → 106 / 140).
    const TOM_2H_SOBRE_HB: f64 = 106.0 / 140.0;

    /// ⭐⭐ **Os grafites são UMA família com a borda do Rebelle** (J3): (a) a borda de cada um ONDULA como a do Rebelle
    /// (onda ≥ 0,04 da largura, correlação ≥ 4 px — o dual da orla; sem ele 0,026–0,034 com 1–2 px, a «borda aos
    /// caroços» que a revisão cega viu no original e não no nosso); (b) o HB e o 2H têm o grão FINO e FRACO a 5 px (cv
    /// até 1,6× e a autocorrelação até +0,15 da do Rebelle; eram 2× e +0,54); (c) o EIXO de dureza vê-se: a cobertura a 40
    /// px sobe 2H < HB < 8B e o 2H fica no tom de fábrica dele (0,76 do HB ± 0,08).
    #[test]
    fn os_grafites_sao_uma_familia_com_a_borda_do_rebelle() {
        let m: Vec<_> = std::thread::scope(|s| {
            let hs: Vec<_> = GRAFITES.iter().map(|&n| s.spawn(move || medidas(&pinceis(n).remove(0)))).collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        let mut falhas = Vec::new();
        for (n, ((o, lag), (cv5, ac5), cob)) in GRAFITES.iter().zip(&m) {
            println!("[grafite] {n:<12} onda {o:.3} ({lag} px) · cv5 {cv5:.3} ac5 {ac5:.2} · cob40 {cob:.3}");
            if *o < ONDA_MIN || *lag < 4 {
                falhas.push(format!("{n}: a borda não ondula ({o:.3}, {lag} px)"));
            }
            if let Some(&(_, rc, ra)) = FINO_DO_REBELLE.iter().find(|f| f.0 == *n) {
                if *cv5 > 1.6 * rc || *ac5 > ra + 0.15 {
                    falhas.push(format!("{n}: o grão a 5 px não é o fino fraco (cv {cv5:.3}, ac {ac5:.2})"));
                }
            }
        }
        let cob = |n: &str| m[GRAFITES.iter().position(|&g| g == n).unwrap()].2;
        let tom = cob("Graphite 2H") / cob("Graphite HB");
        println!("[grafite] tom 2H ÷ HB {tom:.3} (alvo {TOM_2H_SOBRE_HB:.3})");
        if (tom - TOM_2H_SOBRE_HB).abs() > 0.08 || cob("Graphite HB") >= cob("Graphite 8B") {
            falhas.push(format!("o eixo de dureza: 2H ÷ HB {tom:.3}, HB {:.3} · 8B {:.3}", cob("Graphite HB"), cob("Graphite 8B")));
        }
        assert!(falhas.is_empty(), "{falhas:?}");
    }

    /// A mutação do portão de cima, por dentro: o HB SEM a orla (o dual a `null`) e com o grão de antes (Noise 64,
    /// contraste 0,5) TEM de reprovar as duas leis — se não reprovasse, o portão não as estaria a medir.
    #[test]
    fn sem_a_orla_e_com_o_grao_de_antes_o_hb_reprova() {
        let antes = serde_json::json!({"angle_deg": 0, "flow": false, "kind": "Noise", "mapping": "Tiled",
            "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [64.0, 64.0],
            "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]});
        let (o, cv5) = std::thread::scope(|s| {
            let a = s.spawn(|| medidas(&com_campo("Graphite HB", "dual", serde_json::Value::Null)).0.0);
            let b = s.spawn(|| medidas(&com_campo("Graphite HB", "texture", antes)).1.0);
            (a.join().unwrap(), b.join().unwrap())
        });
        println!("[grafite] sem a orla: onda {o:.3} · grão de antes: cv5 {cv5:.3}");
        assert!(o < ONDA_MIN, "sem o dual a borda ainda ondula ({o:.3}): o portão não mede a orla");
        assert!(cv5 > 1.6 * 0.066, "com o grão de antes o cv5 {cv5:.3} passa: o portão não mede o grão fino");
    }

    /// O preset de fábrica `nome` com as variantes `kv` (`c=` contraste do grão · `grao=` o Size · `pig=` a carga ·
    /// `gate=` o papel · `esc=`/`n=` a escala e as marcas da orla · `semdual=1`).
    fn variante(nome: &str, kv: &str) -> ph2d_brush_presets::Preset {
        let mut p = crate::midias_de_fabrica::de_fabrica(nome);
        let b = &mut p.parametros.brush;
        for x in kv.split(',').filter(|x| !x.is_empty()) {
            let (k, v) = x.split_once('=').unwrap();
            let v: f64 = v.parse().unwrap();
            match k {
                "c" => b.get_mut("texture").unwrap()["params"][0] = serde_json::json!(v),
                "grao" => b.get_mut("texture").unwrap()["size"] = serde_json::json!([v, v]),
                "esc" => b.get_mut("dual").unwrap()["escala"] = serde_json::json!(v),
                "n" => b.get_mut("dual").unwrap()["contagem"] = serde_json::json!(v as u32),
                "dur" => b.get_mut("dual").unwrap()["dureza"] = serde_json::json!(v),
                "modo" => b.get_mut("dual").unwrap()["modo"] = serde_json::json!(v as u8),
                "atr" => b.get_mut("dual").unwrap()["atraves"] = serde_json::json!(v),
                "esp" => b.get_mut("dual").unwrap()["espalhar"] = serde_json::json!(v),
                "semdual" => {
                    b.insert("dual".into(), serde_json::Value::Null);
                }
                "pig" => {
                    p.parametros.engine.insert("pigmentPerDab".into(), v);
                }
                "gate" => {
                    p.parametros.engine.insert("paperGate".into(), v);
                }
                _ => panic!("chave {k}"),
            }
        }
        p
    }

    /// A largura e a textura do traço recto do preset `pr` com a caneta a `tilt` graus (a cena de `a_caneta_deitada`).
    fn deitado(pr: &ph2d_brush_presets::Preset, tilt: [f32; 2]) -> (f64, f64) {
        use ph2d_editor_core::Tool;
        use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
        let mut t = ph2d_tool_painter::PainterTool::default();
        t.set_source(vec![255u8; 1000 * 400 * 4], 1000, 400);
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(pr).is_empty());
        t.set_brush_size_px(15.0);
        t.set_brush_color_srgb8([23, 101, 179]);
        let tilt = [tilt[0] / 90.0, tilt[1] / 90.0];
        let cp = |x: f32, phase| CanvasPointer { pos: [x, 200.0], pressure: 1.0, tilt, phase };
        t.on_canvas_pointer(cp(200.0, PointerPhase::Down));
        let mut x = 200.0;
        while x < 700.0 {
            x += 3.0;
            t.on_canvas_pointer(cp(x, PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(cp(x, PointerPhase::Up));
        for _ in 0..120 {
            t.on_tick(1000.0 / 60.0);
        }
        let (px, w, _) = t.run_full();
        let m = crate::regua_midia::mede(&px, w, [0, 80, 1000, 240]);
        (m.largura, m.textura)
    }

    /// SONDA: as métricas da mídia (a cena seca de `cada_preset_parece_a_sua_midia`, ao raio da foto) e do lápis deitado
    /// (através ÷ de pé) de cada `PH2D_SONDA_PRESETS` (`,`) com cada variante de `PH2D_SONDA_KNOBS` (`|`; ver [`variante`]).
    #[test]
    #[ignore = "sonda: as métricas da mídia e do lápis deitado por variante; corre-se à mão"]
    fn sonda_as_variantes_da_familia() {
        let nomes = std::env::var("PH2D_SONDA_PRESETS").unwrap_or_else(|_| "Graphite 8B".into());
        let kvs = std::env::var("PH2D_SONDA_KNOBS").unwrap_or_default();
        let casos: Vec<(String, String)> = nomes
            .split(',')
            .flat_map(|n| std::iter::once("").chain(kvs.split('|')).map(move |k| (n.to_owned(), k.to_owned())))
            .collect();
        let linhas: Vec<String> = std::thread::scope(|s| {
            let hs: Vec<_> = casos
                .iter()
                .map(|(n, kv)| {
                    s.spawn(move || {
                        let p = variante(n, kv);
                        let raio = crate::midias_de_fabrica::raio_da_midia(n).unwrap_or(15.0);
                        let m = crate::midias_de_fabrica::cena_seca(&p, raio, &[]).0;
                        let (pe, at, lo) = (deitado(&p, [0.0, 0.0]), deitado(&p, [0.0, 55.0]), deitado(&p, [55.0, 0.0]));
                        let pincel = Pincel {
                            nome: n.clone(),
                            fonte: Fonte::Preset { preset: Box::new(p), relativo: String::new() },
                        };
                        let ((o, lag), (cv5, ac5), cob40) = medidas(&pincel);
                        format!(
                            "[variante] {n} {kv:<22} cob {:.3} tex {:.2} borda {:.3} orla {} larg {} · deitado larg {:.2} tex {:.2} ao longo {:.2} · onda {o:.3} ({lag}) cv5 {cv5:.3} ac5 {ac5:.2} cob40 {cob40:.3}",
                            m.cobertura, m.textura, m.borda, m.orla, m.largura, at.0 / pe.0, at.1 / pe.1, lo.0 / pe.0
                        )
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        for l in linhas {
            println!("{l}");
        }
    }
}
