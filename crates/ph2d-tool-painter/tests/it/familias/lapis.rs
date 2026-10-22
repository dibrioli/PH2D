//! **Família `lapis`** — J3, catálogo B (os lápis além da série de grafite): Colored 4B, Scratch (Wet Paint), Tilted
//! Shade, Rough Sketch, Craft Flat (Digital). Os cinco REPROVADOS na revisão cega da J3 — o porquê, as receitas medidas e o
//! que falta: `docs/Painter/Pinceis/exames/lapis.md`. Escreva aqui as receitas e os testes da família; nada fora deste
//! ficheiro (`docs/Painter/Pinceis/04_BANCADA.md` §9). As bases (`GRAPHITE_8B`, `CHARCOAL`) são partilhadas: não as mude
//! daqui.

use super::prelude::*;

pub const WET: &[Receita] = &[
    // B-8 — o 4B do Rebelle (com a cor da bateria «Colored Pencil/4B Color» É «Graphite Pencil/4B» ao byte; só a cor de
    // omissão os separa): o degrau entre o 2B e o 8B. J3 (`exames/lapis.md`): era liso como o Oil Pastel (d 0,498); a
    // ORLA que ondula (a 2.ª ponta de duas marcas a 0,6), os tufos e o grão de 2 px dão-lhe a borda aos caroços do Rebelle
    // (a onda a 40 px 0,049 → 0,062; Rebelle 0,061) e o tecto o recto de 200 px dele (62).
    Receita {
        pincel: &[
            ("falloff", "\"Linear\""),
            ("jitter", "0.1"),
            ("dual", ORLA_DO_4B),
            ("texture", GRAO_DO_4B),
            ("shape", TUFOS_DO_4B),
            ("size_max_px", "62"),
        ],
        grao: None,
        ..lapis(
            "Colored",
            "Colored 4B",
            140.0,
            0.45,
            GRAO_MACIO,
            extras_do_lapis!(0.5, 3.0),
        )
    },
    // B-10 — `lapisc_053`: o carvão mais ralo, o papel a meio. ⛔ J3 (`exames/lapis.md`): REPROVADO e de volta ao de antes —
    // os riscos do Rebelle (um punhado de IMAGENS de riscos por marca) pedem a ponta de imagens, que a receita de fábrica não
    // leva; uma Stripes rodada por dab passa a régua (452/454) mas a 200 px é névoa (ou, sem a penugem, palha de 1 px) e
    // reprova o portão da mídia (a cobertura 0,307 e o cv 0,067 da foto, contra 0,255 e 0,126).
    Receita {
        meio: "Pencil",
        grupo: "Scratch",
        nome: "Scratch",
        pigment: 20.0,
        gate: 0.5,
        size_px: Some(248.0),
        extras: &[
            ("felt", 0.0),
            ("drag", 0.02),
            MAIS_PERTO,
            ("pressureSizeCurve", 0.35),
            ("pressureOpacity", 0.75),
            ("pressureOpacityCurve", 0.75),
        ],
        nasce_de: Some("Charcoal"),
        ..CHARCOAL
    },
];

/// A orla do 4B: a 2.ª ponta (dual Multiply) de DUAS marcas a 0,6 do diâmetro, espalhadas 0,25 e 0,2 através — os
/// caroços FINOS da borda do Rebelle. Com uma marca a 0,8 os caroços eram grandes e regulares e o 1.º dab saía uma
/// bolha separada (a 1.ª revisão cega); a 0,45 × 3 a onda caía abaixo da do Rebelle (0,056 contra 0,061).
const ORLA_DO_4B: &str = r#"{"angulo": 0, "atraves": 0.2, "contagem": 2, "dureza": 0.3, "escala": 0.6, "espalhar": 0.25, "forma": {"angle_deg": 0, "flow": false, "kind": "None", "mapping": "ViewPlane", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [1.0, 1.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}, "modo": 0, "passo": 0.25, "rotacao": 0.0}"#;
/// O grão do 4B: Noise `Tiled` Size 128 (2 px) a 0,5. A 2.ª revisão cega com o Size 100 a 0,55 e a carga 200 tomou o
/// nosso 8B pelo original e o Rebelle por «outro, translúcido» (o nosso «crivado de furinhos», o par do 8B a 0,43): mais
/// leve e de grão mais fino ele fica a 0,54 do 8B e a 0,73 do Oil Pastel.
const GRAO_DO_4B: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.5, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [128.0, 128.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;
/// Os tufos do 4B: cada dab uma mancha (Noise `Random` de lado 8 a 0,6).
const TUFOS_DO_4B: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "mapping": "Random", "offset": [0.0, 0.0], "params": [0.6, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [8.0, 8.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

/// O grão do carpinteiro: fraco (a Textura do Rebelle 2,0 contra 10 do 4B — a mina dura e lisa).
const GRAO_DO_CRAFT: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.1, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [128.0, 128.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

/// Nenhum lápis novo do Digital na fábrica: os três reprovaram na revisão cega (`exames/lapis.md`).
pub const MEIOS: &[ReceitaMeio] = &[];

/// Os REPROVADOS do Digital (B-6, B-9, B-7; o porquê e o que falta: `exames/lapis.md`): receitas medidas, fora da
/// fábrica (nenhum ficheiro); os portões medem-nas pela receita ([`portoes`]).
pub const EM_ESTUDO: &[ReceitaMeio] = &[
    // B-6 — o lápis DEITADO do Krita «c) Pencil-5 Tilted» (`exames/lapis.md`, a bateria `exames/lapis/bateria_deitado.json`
    // e as fotos com a caneta em `fixtures/krita_deitado/`): de pé uma linha fina de grafite; deitado, o LADO da mina.
    // A ponta é uma elipse de pé FIXA na tela (0,4 × D, o eixo longo deitado: de pé o recto horizontal fino e o vertical
    // largo, 2 × 5 px no Krita) que a caneta vira para o azimute e estica 5× (através) ou alarga 3× (ao longo); a
    // inclinação conta dos 15° (a zona morta do Rebelle vai aos 37,5°). Krita a 40 px: através 2 · 9 · 10 · 11 · 12 ·
    // 13 · 17 px (0 · 35 · 40 · 45 · 50 · 55 · 63°), ao longo 2 → 6 (portão `o_tilted_shade_deita_como_o_lapis_do_krita`).
    // A ponta é uma RAMPA (a `gradient.png` do Krita rodada pelo azimute): deitada, a ARESTA densa do lado da mina e o
    // esbatido do outro, a translucidez pela acumulação a 0,55 (a 1.ª revisão cega: «a imitação é uma faixa simétrica,
    // opaca, com as duas bordas duras»). Tamanho: o Krita pinta 17 px deitado no tamanho de fábrica (220 px × 0,18).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.8"),
            ("texture", PAPEL_DO_DEITADO),
            ("jitter", "0.04"),
            ("dab_flatten", "0.6"),
            ("dab_angle_deg", "0"),
            ("shape", RAMPA_DEITADA),
            ("accumulate", "true"),
            ("flow", "0.55"),
        ],
        mao: &[
            ("pressureSize", 0.0),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.3),
            ("dabRotation", 2.0),
            ("tiltStart", 15.0),
            ("tiltFull", 63.0),
            ("tiltWidth", 3.0),
            ("tiltLength", 5.0),
        ],
        size_px: Some(5.2),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Pencil", "Tilted", "Tilted Shade")
    },
    // B-9 — o lápis de carpinteiro do Rebelle «Graphite Pencil/Craft Flat» (`exames/lapis.md`): a mina CHATA, FIXA na tela
    // (L3: sem a caneta o eixo longo fica deitado — o recto horizontal fino, o vertical largo: 21 × 30 px a «40»), os
    // dabs RECTANGULARES soltos (espaçamento 0,2 e o espalhar do Rebelle: a borda aos degraus) e quase sem grão (a
    // Textura dele 2,0). No Digital: no Wet a cápsula de 0,025 fundia os rectângulos num traço liso (`exames/lapis.md`).
    // A caneta deitada é a do Rebelle (PEN_TILT, 37,5° → 67,5°, × 4,2). Tamanho = o do preset (14,6); o tecto = o recto
    // de 200 dele (53 px no lado fino ⇒ 85 de diâmetro). O achatado 0,38 (a 0,3 a quina e o espalhar engordavam o lado
    // fino: 0,78 contra os 0,70 do Rebelle); a dureza 0,7 dá-lhe a cobertura dele (0,75 → 0,81; Rebelle 0,84) sem
    // perder os degraus (com o espalhar a 0,06 o traço saía liso).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.7"),
            ("texture", GRAO_DO_CRAFT),
            ("jitter", "0.15"),
            ("spacing", "0.25"),
            ("flow", "1.0"),
            ("dab_flatten", "0.38"),
            ("dab_angle_deg", "0"),
            ("dab_quina", "1.0"),
            ("size_max_px", "85"),
        ],
        mao: &[
            ("pressureSize", 0.9),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 1.0),
            ("dabRotation", 2.0),
            ("tiltStart", 37.5),
            ("tiltFull", 67.5),
            ("tiltLength", 4.2),
        ],
        size_px: Some(14.6),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Pencil", "Craft", "Craft Flat")
    },
    // B-7 — o rascunho do Krita «c) Pencil-4 Soft» (o CSP Rough pencil): muito grão em FIBRAS ao comprido, a borda que
    // treme, do claro ao escuro pela pressão — na rampa 0 → 1 a pressão 0,4 pousa 0,54 da tinta e 0,25 da largura da
    // cheia (Krita 0,51 · 0,33; com o S inteiro e a tinta em recta 0,23 · 0,04: entrava tarde demais). A ponta quase
    // QUADRADA que segue o traço (a célula de grão do Krita, rodada pelo ângulo do traço): a 200 px a faixa tem a borda
    // direita dele, e não os festões de dabs redondos. Tamanho = o do preset (128 px × 0,125).
    ReceitaMeio {
        pincel: &[
            ("hardness", "0.6"),
            ("falloff", "\"Linear\""),
            ("shape", FIBRAS_DO_RASCUNHO),
            ("texture", PAPEL_DO_RASCUNHO),
            ("jitter", "0.15"),
            ("spacing", "0.2"),
            ("flow", "1.0"),
            ("dab_quina", "0.7"),
        ],
        mao: &[
            ("pressureSize", 0.9),
            ("pressureSCurve", 0.4),
            ("pressureOpacity", 1.0),
            ("pressureOpacityCurve", 0.3),
            ("dabRotation", 1.0),
        ],
        size_px: Some(16.0),
        opacity: Some(1.0),
        ..ReceitaMeio::digital("Pencil", "Sketch", "Rough Sketch")
    },
];

/// A rampa do lápis deitado: Gradient `ViewPlane` de lado 0,25 deslocado 0,5 (UMA rampa através da pegada, sem a
/// costura do `frac` no meio), contraste 0,7, brilho 0,55, virada 180° (a aresta densa do lado para onde a caneta se
/// deita, como a do Krita). Os 2 primeiros `params` são o tom (contraste, brilho): a 0 a forma apagava o dab.
const RAMPA_DEITADA: &str = r#"{"angle_deg": 180, "flow": false, "kind": "Gradient", "mapping": "ViewPlane", "offset": [0.5, 0.5], "params": [0.7, 0.55, 0.0, 0.0, 0.5, 0.5], "rake": false, "size": [0.25, 0.25], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;
/// O papel do lápis deitado: o grão preso à tela (os salpicos brancos da faixa do Krita).
const PAPEL_DO_DEITADO: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.6, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [128.0, 128.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;
/// As FIBRAS do rascunho: um Noise esticado ao longo do traço (lado 3 × 16 a 0,7, `rake` = segue a direcção) sorteado por
/// dab — as riscas ao comprido do `graphite_grain` do Krita (rodado pelo ângulo do traço). Com o espaçamento 0,2 (o do
/// Krita) elas não se desfazem na média dos dabs (a 0,1 o miolo era sal e pimenta); a 2 × 10 a 0,7 eram fios de cabelo
/// soltos e o traço saía claro (a régua 194 → 280 de 338).
const FIBRAS_DO_RASCUNHO: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "mapping": "Random", "offset": [0.0, 0.0], "params": [0.7, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": true, "size": [3.0, 16.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;
/// O papel do rascunho: fraco (a 0,6 o miolo era sal e pimenta).
const PAPEL_DO_RASCUNHO: &str = r#"{"angle_deg": 0, "flow": false, "kind": "Noise", "mapping": "Tiled", "offset": [0.0, 0.0], "params": [0.3, 0.5, 0.5, 0.5, 0.5, 0.5], "rake": false, "size": [128.0, 128.0], "stencil_angle_deg": 0, "stencil_offset": [0.0, 0.0], "stencil_size": [0.5, 0.5]}"#;

#[rustfmt::skip]
pub const MIDIAS: &[Midia] = &[
    Midia { preset: "Colored 4B", foto: "pencil4bcolor", raio: 15.00, metricas: &[(Cobertura, 0.665), (Textura, 11.826), (Borda, 0.327), (Orla, 10.0), (Cruzamento, 0.730), (Largura, 25.0)] },
    Midia { preset: "Scratch", foto: "scratch", raio: 15.71, metricas: &[(Cobertura, 0.255), (Textura, 19.780), (Borda, 0.591), (Orla, 13.0), (Cv, 0.126), (Cruzamento, 0.652), (Largura, 24.0)] },
];

// ---- Os portões da família (J3, `docs/Painter/Pinceis/exames/lapis.md`) ----

#[cfg(test)]
mod portoes {
    use crate::exame_bancada::{Fonte, Pincel, pinceis, pinta_cena};
    use serde_json::Value;

    /// A cobertura (0..1) de cada píxel da tela 1000 × 640 sobre o papel branco.
    fn cobertura(px: &[u8]) -> Vec<f64> {
        px.chunks_exact(4)
            .map(|c| (0..3).map(|k| 255.0 - f64::from(c[k])).fold(0.0, f64::max) / 255.0)
            .collect()
    }

    /// O pico e a largura (limiar ¼ do p90 dos picos) de cada coluna `x0..x1` da faixa horizontal `y ± h`.
    fn colunas(c: &[f64], y: usize, h: usize, x0: usize, x1: usize) -> (Vec<f64>, Vec<f64>) {
        let col = |x: usize| -> Vec<f64> {
            (y.saturating_sub(h)..=(y + h).min(639))
                .map(|yy| c[yy * 1000 + x])
                .collect()
        };
        let picos: Vec<f64> = (x0..x1)
            .map(|x| col(x).into_iter().fold(0.0, f64::max))
            .collect();
        let mut p = picos.clone();
        p.sort_by(f64::total_cmp);
        let lim = 0.25 * p[(0.9 * (p.len() - 1) as f64) as usize];
        let w = (x0..x1)
            .map(|x| col(x).iter().filter(|&&q| q >= lim.max(1e-3)).count() as f64)
            .collect();
        (picos, w)
    }

    fn mediana(mut v: Vec<f64>) -> f64 {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    }

    /// O pincel `nome`: o da fábrica, ou — um reprovado [`super::EM_ESTUDO`] — o preset que a receita gravaria.
    fn pincel_de(nome: &str) -> Pincel {
        let Some(r) = super::EM_ESTUDO.iter().find(|r| r.nome == nome) else {
            return pinceis(nome).remove(0);
        };
        let (parametros, texturas) = crate::presets_de_fabrica::parametros_do_meio(r);
        let preset = ph2d_brush_presets::Preset {
            parametros,
            origem: ph2d_brush_presets::Origem::Fabrica,
            miniatura: ph2d_brush_presets::Miniatura {
                largura: 1,
                altura: 1,
                rgba: vec![255; 4],
            },
            texturas,
        };
        Pincel {
            nome: nome.to_owned(),
            fonte: Fonte::Preset {
                preset: Box::new(preset),
                relativo: nome.to_owned(),
            },
        }
    }

    /// Os reprovados não estão na fábrica: nenhum ficheiro com o nome de uma receita [`super::EM_ESTUDO`].
    #[test]
    fn os_reprovados_do_lapis_nao_estao_na_fabrica() {
        let b = ph2d_brush_presets::Biblioteca::carregar(
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
            None,
        );
        let na_fabrica: Vec<&str> = b
            .entradas
            .iter()
            .map(|e| e.preset.parametros.name.as_str())
            .filter(|n| super::EM_ESTUDO.iter().any(|r| r.nome == *n))
            .collect();
        assert!(
            na_fabrica.is_empty(),
            "reprovados na fábrica: {na_fabrica:?}"
        );
    }

    /// O pincel `nome` ([`pincel_de`]) com uma linha da mão (`engine`) ou um campo do pincel (`brush.<campo>`) trocado —
    /// a mutação dos portões.
    fn com(nome: &str, chave: &str, v: Value) -> Pincel {
        let mut p = pincel_de(nome);
        if let Fonte::Preset { preset, .. } = &mut p.fonte {
            match chave.strip_prefix("brush.") {
                Some(c) => {
                    preset.parametros.brush.insert(c.to_owned(), v);
                }
                None => {
                    preset
                        .parametros
                        .engine
                        .insert(chave.to_owned(), v.as_f64().unwrap());
                }
            }
        }
        p
    }

    /// A cena `cena` da bateria DEITADA da família (`exames/lapis/bateria_deitado.json`) pintada por `p`.
    fn deitada(p: &Pincel, cena: &str) -> Vec<f64> {
        let f = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/Painter/Pinceis/exames/lapis/bateria_deitado.json");
        let bat: Value = serde_json::from_str(&std::fs::read_to_string(f).unwrap()).unwrap();
        let c = bat["cenas"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["nome"] == cena)
            .unwrap();
        cobertura(&pinta_cena(p, None, c, [1000, 640], [255; 3], 0).1)
    }

    /// A largura (px) de pé, através a 35° e a 63° e ao longo a 63° da bateria deitada.
    fn larguras_deitado(p: &Pincel) -> [f64; 4] {
        let (at, mais, lo) = (
            deitada(p, "deitado_atraves"),
            deitada(p, "deitado_mais"),
            deitada(p, "deitado_ao_longo"),
        );
        let w = |c: &[f64], y, h| mediana(colunas(c, y, h, 160, 460).1);
        [
            w(&at, 80, 79),
            w(&at, 240, 79),
            w(&mais, 400, 79),
            w(&lo, 570, 37),
        ]
    }

    /// O Krita «c) Pencil-5 Tilted» com a caneta (`fixtures/krita_deitado/`, a 40 px): de pé 2 px, através a 35° 9 e a
    /// 63° 17, ao longo a 63° 6 — as razões ao através a 63°.
    const KRITA_DEITADO: [f64; 3] = [2.0 / 17.0, 9.0 / 17.0, 6.0 / 17.0];

    /// ⭐⭐ **O Tilted Shade deita-se como o lápis do Krita** (B-6): a ponta fina de pé vira o LADO da mina — através ela
    /// alarga ~8× (de pé ÷ deitada a 63° ≤ 0,2; Krita 0,12) desde quase de pé (35° ÷ 63° 0,53 ± 0,12) e ao longo só ~3×
    /// (0,35 ± 0,08 do através). O controlo por dentro: SEM a caneta a esticar (`tiltLength` 1) a ponta deitada através é
    /// a de pé ao comprido (de pé ÷ 63° ~0,4) e o portão tem de o ver.
    #[test]
    fn o_tilted_shade_deita_como_o_lapis_do_krita() {
        let (m, sem) = std::thread::scope(|s| {
            let a = s.spawn(|| larguras_deitado(&pincel_de("Tilted Shade")));
            let b = s.spawn(|| larguras_deitado(&com("Tilted Shade", "tiltLength", 1.0.into())));
            (a.join().unwrap(), b.join().unwrap())
        });
        let r = |w: &[f64; 4]| [w[0] / w[2], w[1] / w[2], w[3] / w[2]];
        let (rm, rs) = (r(&m), r(&sem));
        println!(
            "[deitado] Tilted Shade {m:?} → {rm:.3?} (Krita {KRITA_DEITADO:.3?}) · sem esticar {sem:?} → {rs:.3?}"
        );
        assert!(
            rm[0] <= 0.2,
            "de pé ÷ deitado através {:.3} (Krita 0,12)",
            rm[0]
        );
        assert!(
            (rm[1] - KRITA_DEITADO[1]).abs() <= 0.12,
            "35° ÷ 63° {:.3} (Krita 0,53)",
            rm[1]
        );
        assert!(
            (rm[2] - KRITA_DEITADO[2]).abs() <= 0.08,
            "ao longo ÷ através {:.3} (Krita 0,35)",
            rm[2]
        );
        assert!(
            rs[0] > 0.2,
            "sem esticar a ponta deitada o portão ainda passa ({:.3}): não mede a caneta",
            rs[0]
        );
    }

    /// A largura do recto horizontal lento (y 90, x 160..460) e do vertical lento (x 700, y 460..590) da cena `gestos`.
    fn chato(p: &Pincel) -> (f64, f64) {
        let c = cobertura(&crate::as_leis_da_escala::cena(p, "gestos"));
        let h = mediana(colunas(&c, 90, 40, 160, 460).1);
        let t: Vec<f64> = c
            .chunks_exact(1000)
            .flat_map(|l| l[660..741].to_vec())
            .collect();
        // A vertical transposta: as linhas 460..590 viram colunas de uma tela 81 de largura.
        let picos: Vec<f64> = (460..590)
            .map(|y| (0..81).map(|x| t[y * 81 + x]).fold(0.0, f64::max))
            .collect();
        let mut p9 = picos.clone();
        p9.sort_by(f64::total_cmp);
        let lim = 0.25 * p9[(0.9 * (p9.len() - 1) as f64) as usize];
        let v = mediana(
            (460..590)
                .map(|y| (0..81).filter(|&x| t[y * 81 + x] >= lim).count() as f64)
                .collect(),
        );
        (h, v)
    }

    /// ⭐⭐ **O Craft Flat é CHATO e fixo na tela** (B-9): o recto horizontal é o lado fino da mina e o vertical o largo —
    /// o Rebelle 21 × 30 px a «40» (0,70; a régua). O controlo por dentro: a ponta REDONDA (`dab_flatten` 0) dá 1.
    #[test]
    fn o_craft_flat_e_chato_e_fixo_na_tela() {
        let (m, r) = std::thread::scope(|s| {
            let a = s.spawn(|| chato(&pincel_de("Craft Flat")));
            let b = s.spawn(|| chato(&com("Craft Flat", "brush.dab_flatten", 0.0.into())));
            (a.join().unwrap(), b.join().unwrap())
        });
        println!(
            "[chato] Craft Flat {m:?} → {:.3} (Rebelle 0,70) · redondo {r:?} → {:.3}",
            m.0 / m.1,
            r.0 / r.1
        );
        assert!(
            (m.0 / m.1 - 0.70).abs() <= 0.08,
            "horizontal ÷ vertical {:.3} (Rebelle 0,70)",
            m.0 / m.1
        );
        assert!(
            r.0 / r.1 > 0.85,
            "a ponta redonda ainda passa ({:.3}): o portão não mede a mina chata",
            r.0 / r.1
        );
    }

    /// A cobertura (o pico médio) e a largura na pressão ~0,4 (x 240..280) e ~0,95 (x 515..555) da rampa 0 → 1 (y 400).
    fn rampa(p: &Pincel) -> (f64, f64) {
        let c = cobertura(&crate::as_leis_da_escala::cena(p, "gestos"));
        let (pk, w) = colunas(&c, 400, 35, 60, 560);
        let media =
            |v: &[f64], a: usize, b: usize| v[a - 60..b - 60].iter().sum::<f64>() / (b - a) as f64;
        (
            media(&pk, 240, 280) / media(&pk, 515, 555),
            media(&w, 240, 280) / media(&w, 515, 555),
        )
    }

    /// ⭐⭐ **O Rough Sketch vai do claro ao escuro pela pressão** (B-7): na rampa 0 → 1 a tinta e a largura na pressão
    /// 0,4 são as do Krita «c) Pencil-4 Soft» ÷ a cheia (estas contas sobre a foto dele: 0,505 · 0,325; ± 0,08). O
    /// controlo por dentro: SEM a pressão na tinta (`pressureOpacity` 0) a cobertura sobe para fora da faixa.
    #[test]
    fn o_rough_sketch_vai_do_claro_ao_escuro_pela_pressao() {
        let (m, sem) = std::thread::scope(|s| {
            let a = s.spawn(|| rampa(&pincel_de("Rough Sketch")));
            let b = s.spawn(|| rampa(&com("Rough Sketch", "pressureOpacity", 0.0.into())));
            (a.join().unwrap(), b.join().unwrap())
        });
        println!(
            "[rampa] Rough Sketch cobertura {:.3} largura {:.3} (Krita 0,505 · 0,325) · sem a pressão na tinta {:.3}",
            m.0, m.1, sem.0
        );
        assert!(
            (m.0 - 0.505).abs() <= 0.08,
            "a cobertura a 0,4 ÷ cheia {:.3} (Krita 0,505)",
            m.0
        );
        assert!(
            (m.1 - 0.325).abs() <= 0.08,
            "a largura a 0,4 ÷ cheia {:.3} (Krita 0,325)",
            m.1
        );
        assert!(
            (sem.0 - 0.505).abs() > 0.08,
            "sem a pressão na tinta ainda passa ({:.3}): o portão não mede a tinta",
            sem.0
        );
    }

    /// A onda da borda do recto de 40 px (cena `tamanhos`, y 190 ± 70): o desvio das bordas ÷ a largura.
    fn onda(p: &Pincel) -> (f64, f64) {
        let c = cobertura(&crate::as_leis_da_escala::cena(p, "tamanhos"));
        let col = |x: usize| -> Vec<f64> { (120..=260).map(|yy| c[yy * 1000 + x]).collect() };
        let (pk, _) = colunas(&c, 190, 70, 148, 852);
        let mut p9 = pk.clone();
        p9.sort_by(f64::total_cmp);
        let lim = 0.25 * p9[(0.9 * (p9.len() - 1) as f64) as usize];
        let (mut t, mut b) = (Vec::new(), Vec::new());
        for x in 148..852 {
            let v = col(x);
            if let (Some(a), Some(z)) = (
                v.iter().position(|&q| q >= lim),
                v.iter().rposition(|&q| q >= lim),
            ) {
                t.push(a as f64);
                b.push(z as f64);
            }
        }
        let larg = mediana(t.iter().zip(&b).map(|(a, z)| z - a + 1.0).collect());
        let desvio = |e: &[f64]| {
            let n = e.len() as f64;
            let xm = (n - 1.0) / 2.0;
            let em = e.iter().sum::<f64>() / n;
            let sxy: f64 = e
                .iter()
                .enumerate()
                .map(|(i, v)| (i as f64 - xm) * (v - em))
                .sum();
            let sxx: f64 = (0..e.len()).map(|i| (i as f64 - xm).powi(2)).sum();
            (e.iter()
                .enumerate()
                .map(|(i, v)| (v - em - sxy / sxx * (i as f64 - xm)).powi(2))
                .sum::<f64>()
                / n)
                .sqrt()
        };
        let tinta = mediana(pk);
        ((desvio(&t) + desvio(&b)) / 2.0 / larg, tinta)
    }

    /// ⭐⭐ **O Colored 4B é o 4B do Rebelle: a borda aos caroços, o degrau entre o 2B e o 8B** (B-8): a onda da borda a
    /// 40 px ≥ 0,055 da largura (Rebelle 0,061; sem a orla 0,049 — a 1.ª revisão cega «liso como o Oil Pastel») e o tom
    /// do miolo entre o do 2B e o do 8B. O controlo por dentro: SEM a 2.ª ponta (`dual` nulo) a onda cai.
    #[test]
    fn o_colored_4b_tem_a_borda_aos_carocos_entre_o_2b_e_o_8b() {
        let m: Vec<_> = std::thread::scope(|s| {
            let hs: Vec<_> = [
                pinceis("Colored 4B").remove(0),
                pinceis("Graphite 2B").remove(0),
                pinceis("Graphite 8B").remove(0),
                com("Colored 4B", "brush.dual", Value::Null),
            ]
            .into_iter()
            .map(|p| s.spawn(move || onda(&p)))
            .collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        println!(
            "[4B] onda {:.3} tinta {:.3} · 2B {:.3} · 8B {:.3} · sem a orla onda {:.3}",
            m[0].0, m[0].1, m[1].1, m[2].1, m[3].0
        );
        assert!(
            m[0].0 >= 0.055,
            "a borda do 4B não ondula ({:.3}; Rebelle 0,061)",
            m[0].0
        );
        assert!(
            m[1].1 < m[0].1 && m[0].1 < m[2].1,
            "o tom: 2B {:.3} · 4B {:.3} · 8B {:.3}",
            m[1].1,
            m[0].1,
            m[2].1
        );
        assert!(
            m[3].0 < 0.055,
            "sem a orla a borda ainda ondula ({:.3}): o portão não mede a orla",
            m[3].0
        );
    }
}
