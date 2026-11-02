//! **Cada preset de mídia parece a SUA mídia** (prompt 17, Fase C; 05 §4b «Presets de todas as mídias»).
//!
//! A cena da bancada (`sonda_gestos::pinta_cena_ate`) com o preset à largura da foto do Rebelle da mesma
//! mídia, SECA (os 8 s da cena + 22 s: o Rebelle é fotografado depois do `DRY`), medida pela régua do
//! Python portada (`regua_midia`) no recto. A BARRA de cada métrica é o VALE (05 §3): o preset tem de
//! ficar do lado do Rebelle do ponto médio entre o Rebelle e o CONTROLO — o Wet Round à mesma largura,
//! medido aqui, que tem de ficar FORA (sem isso o portão aprovaria um Round com outro nome).
//!
//! Só entram as métricas que DISTINGUEM a mídia do Round acima do ruído do próprio Rebelle (a diferença
//! entre o recto e o rápido da mesma foto, p90 sobre as 25 fotos: cobertura 0,012 · textura 0,37 % ·
//! borda/miolo 0,031) — a borda do Liner (1,05 contra 1,00) não distingue e não conta.

use super::regua_midia::{Medida, RECTO, mede};
use super::sonda_gestos::{Relogio, pinta_cena_com, tela};
use ph2d_brush_presets::{Biblioteca, Preset};
use std::path::PathBuf;

/// Uma métrica da régua e o ruído do próprio Rebelle nela (recto × rápido da mesma foto, p90 sobre as 25
/// fotos, 11/10): só conta numa mídia se o controlo estiver a mais de 3× este ruído da foto.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Metrica {
    Cobertura,
    Textura,
    Borda,
    Orla,
    /// A largura do traço DE PÉ (D4–D6) sobre a do DEITADO (A2–F2): o pincel chato (prompt 23).
    Chato,
    /// O CRUZAMENTO vermelho × azul (18 §2): a identidade da tinta e do marcador (prompt 42).
    Cruzamento,
    /// A uniformidade ao longo do traço (`uniformidade_cv`): o pó do aerógrafo (prompt 43).
    Cv,
    /// O RECUO da ponta (`regua_midia::ponta`): a ponta reta do Flat (prompt 44).
    Ponta,
    /// A rampa da pressão (`regua_midia::rampa`): a largura na pressão ~0,46 ÷ na 1 (prompt 44).
    RampaLargura,
    /// A rampa da pressão: a COBERTURA mediana na pressão ~0,46 ÷ na 1 — o esmaecer (prompt 45).
    RampaCobertura,
    /// ⭐ As PONTAS da rampa da pressão (W6, `regua_midia::pontas`): a largura a 5 % e a 95 % ÷ a de 50 %. Além
    /// do vale, a BARRA da W6 (05 §3): onde o Rebelle as tem a ≥ 60 % do miolo, as nossas também.
    Pontas,
    /// A largura a meio do máximo (px): ± 2 px da foto — o controlo é pintado à MESMA largura, então não
    /// distingue nada; é a condição de toda a comparação (05 §4.1).
    Largura,
}

impl Metrica {
    fn de(self, (m, c): &(Medida, FamiliaC)) -> f64 {
        match self {
            Metrica::Cobertura => m.cobertura,
            Metrica::Textura => m.textura,
            Metrica::Borda => m.borda,
            Metrica::Orla => m.orla,
            Metrica::Chato => c.de_pe / c.deitado,
            Metrica::Cruzamento => c.cruz,
            Metrica::Cv => m.cv,
            Metrica::Ponta => c.ponta,
            Metrica::RampaLargura => c.rampa,
            Metrica::RampaCobertura => c.rampa_cob,
            Metrica::Pontas => c.pontas,
            Metrica::Largura => m.largura,
        }
    }
    fn ruido(self) -> f64 {
        match self {
            Metrica::Cobertura => 0.012,
            Metrica::Textura => 0.37,
            Metrica::Borda => 0.031,
            Metrica::Orla => 1.0,
            // Uma foto só tem um traço de pé: o ruído é o de 1 px de largura nos dois (≈ 0,03) e a tinta é
            // lida como a cobertura.
            Metrica::Chato => 0.05,
            // O controlo da sessão de 16/10: o Dip Pen de 11/10 refeito deu 0,283 contra 0,223.
            Metrica::Cruzamento => 0.06,
            Metrica::Cv => 0.012,
            // O rápido × o vermelho da mesma foto (doc 44 §1).
            Metrica::Ponta => 0.045,
            // Um traço só por foto: 1 px de largura nos dois pontos (≈ 0,04).
            Metrica::RampaLargura => 0.05,
            // Um traço só por foto e nenhuma repetição: não medido. 0,04 deixa o controlo a > 5× dele.
            Metrica::RampaCobertura => 0.04,
            // A tinta × o marcador do mesmo dia (68 % nos dois) e o óleo × o óleo sujo (81 %): um traço por
            // foto; 1 px de 40 nas três larguras ≈ 0,03.
            Metrica::Pontas => 0.04,
            Metrica::Largura => 1.0,
        }
    }
}

/// A régua da família C (`ferramentas/mede_familia_c.py`, as mesmas janelas): a largura do traço vermelho DE
/// PÉ em D4 e D6 (fora do cruzamento), a do azul DEITADO A2–F2 e a tinta mediana no miolo dele.
#[derive(Debug)]
pub(super) struct FamiliaC {
    de_pe: f64,
    deitado: f64,
    cruz: f64,
    ponta: f64,
    /// A rampa da pressão: a largura e a cobertura (`regua_midia::rampa`).
    rampa: f64,
    rampa_cob: f64,
    /// As pontas da rampa (`regua_midia::pontas`).
    pontas: f64,
    /// A textura no leve e no forte da rampa (`os_fios::LEVE`/`FORTE`, W2).
    pub leve: f64,
    pub forte: f64,
}

impl FamiliaC {
    fn de(px: &[u8], w: u32) -> Self {
        let w = w as usize;
        let q = |x: usize, y: usize| &px[(y * w + x) * 4..(y * w + x) * 4 + 3];
        // O canal que a tinta mais apaga, e só a tinta da cor certa.
        let azul = |x, y| {
            let c = q(x, y);
            if c[2] < c[0] {
                0.0
            } else {
                1.0 - f64::from(c[0]) / 255.0
            }
        };
        let verm = |x, y| {
            let c = q(x, y);
            if c[0] < c[2] {
                0.0
            } else {
                1.0 - f64::from(c[1]) / 255.0
            }
        };
        // Px acima de metade do pico (o p95 do perfil).
        let acima = |perfil: &[f64]| {
            let mut o = perfil.to_vec();
            o.sort_by(f64::total_cmp);
            let pico = o[((o.len() - 1) as f64 * 0.95).round() as usize];
            perfil.iter().filter(|&&v| v > 0.5 * pico).count() as f64
        };
        let deitado: Vec<f64> = (80..160)
            .map(|y| (150..500).map(|x| azul(x, y)).sum::<f64>() / 350.0)
            .collect();
        let de_pe = [(395_usize, 445_usize), (515, 565)]
            .iter()
            .map(|&(y0, y1)| {
                let p: Vec<f64> = (240..360)
                    .map(|x| (y0..y1).map(|y| verm(x, y)).sum::<f64>() / (y1 - y0) as f64)
                    .collect();
                acima(&p)
            })
            .sum::<f64>()
            / 2.0;
        let (rampa, rampa_cob) = super::regua_midia::rampa(px, w as u32, (px.len() / 4 / w) as u32);
        Self {
            de_pe,
            deitado: acima(&deitado),
            cruz: super::a_lei_da_tinta::cruzamento(px, w as u32),
            ponta: super::regua_midia::ponta(px, w as u32),
            rampa,
            rampa_cob,
            pontas: super::regua_midia::pontas(px, w as u32),
            leve: mede(px, w as u32, super::os_fios::LEVE).textura,
            forte: mede(px, w as u32, super::os_fios::FORTE).textura,
        }
    }
}

/// Os 8 s da cena + 22 s: seca, como a foto do Rebelle.
pub(super) const QUADROS_SECOS: u32 = 1800;

/// Uma mídia: o preset, o
/// raio que dá a largura dela (W2: o número é a largura) e os números da foto (`mede_traco.py`, recto) nas
/// métricas que DISTINGUEM a mídia do Round (18 §3).
pub(crate) struct Midia {
    pub preset: &'static str,
    pub raio: f32,
    pub metricas: &'static [(Metrica, f64)],
}

use Metrica::Cruzamento;

/// ⭐ As mídias de TODAS as famílias (`familias/<família>.rs::MIDIAS`, J3): cada família põe a foto dos seus pincéis no
/// próprio ficheiro. ⛔ As PONTAS da rampa do Rebelle saíram dos pincéis da tinta, do óleo, dos lápis e dos pastéis por
/// ordem do DONO (smoke 18/10: «apertar muito e pouco variam pouco a espessura; antes funcionava melhor»): o piso que
/// as dava comia a resposta da mão dele (0,24–0,86). Ficam no aerógrafo e nos Flats, que já as tinham (doc 47_medicao §2).
static MIDIAS: std::sync::LazyLock<Vec<&'static Midia>> = std::sync::LazyLock::new(|| {
    super::familias::FAMILIAS
        .iter()
        .flat_map(|f| f.midias)
        .collect()
});

/// O raio da cena da mídia `nome` (a largura da foto do Rebelle), se ela tem foto.
pub(super) fn raio_da_midia(nome: &str) -> Option<f32> {
    MIDIAS.iter().find(|m| m.preset == nome).map(|m| m.raio)
}

/// O preset de fábrica `nome` — ou, se saiu da fábrica, o seu ficheiro de candidato ([`candidato`]): o Oil Bristle
/// (smoke do dono 27/10) continua a base de muitas medidas.
pub(super) fn de_fabrica(nome: &str) -> Preset {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .find(|e| e.preset.parametros.name == nome)
        .map(|e| e.preset)
        .or_else(|| candidato(nome))
        .unwrap_or_else(|| panic!("o preset de fábrica {nome:?}"))
}

/// O ficheiro `docs/Painter/Pinceis/W2_<grupo>_candidatos/<nome>.ph2d-brush.png` de um preset que saiu (ou ainda não
/// entrou) na fábrica — `EM_ESTUDO_W2`.
pub(super) fn candidato(nome: &str) -> Option<Preset> {
    let docs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/Painter/Pinceis");
    std::fs::read_dir(&docs)
        .ok()?
        .flatten()
        .filter(|d| {
            let n = d.file_name();
            let n = n.to_string_lossy();
            n.starts_with("W2_") && n.ends_with("_candidatos")
        })
        .map(|d| d.path().join(format!("{nome}.ph2d-brush.png")))
        .find_map(|f| std::fs::read(f).ok())
        .map(|b| ph2d_brush_presets::ler(&b).expect("o ficheiro do candidato lê-se"))
}

/// As tiragens do PÓ em que se medem as pontas e a rampa ([`FamiliaC`]) de um meio com pó (a mediana): numa só a régua
/// mede a sorte — o Airbrush de fábrica lia 0,596 a 1,028 nas pontas por semente, e o de antes da regra 0,540 a 0,783
/// (doc 51 §4).
const TIRAGENS_DO_PO: u64 = 7;

pub(super) fn cena_seca(p: &Preset, raio: f32, ficheiros: &[String]) -> (Medida, FamiliaC) {
    let com_po = p.parametros.engine.get("dust").is_some_and(|&d| d > 0.0);
    let (m, mut c) = cena_seca_semente(p, raio, ficheiros, 0);
    if com_po {
        let mut v: Vec<[f64; 3]> = std::thread::scope(|s| {
            let hs: Vec<_> = (1..TIRAGENS_DO_PO)
                .map(|k| {
                    s.spawn(move || {
                        let c = cena_seca_semente(p, raio, &[], k).1;
                        [c.pontas, c.rampa, c.rampa_cob]
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        v.push([c.pontas, c.rampa, c.rampa_cob]);
        let mediana = |i: usize| {
            let mut x: Vec<f64> = v.iter().map(|t| t[i]).collect();
            x.sort_by(f64::total_cmp);
            x[x.len() / 2]
        };
        [c.pontas, c.rampa, c.rampa_cob] = [mediana(0), mediana(1), mediana(2)];
    }
    (m, c)
}

/// SONDA (doc 51 §4): as pontas de cada tiragem de `PH2D_SONDA_PRESETS` (por omissão o Airbrush) e das variantes
/// (`PH2D_SONDA_VARIANTES`, a régua de `sonda_spacing_continuo`) — `PH2D_SONDA_TIRAGENS` sementes (21).
#[test]
#[ignore = "sonda: as pontas por tiragem do pó; corre-se à mão"]
fn sonda_as_pontas_por_tiragem() {
    use super::sonda_spacing_continuo::{com, com_sub, variantes};
    let n: u64 = std::env::var("PH2D_SONDA_TIRAGENS").map_or(21, |v| v.parse().unwrap());
    let nomes = std::env::var("PH2D_SONDA_PRESETS").unwrap_or_else(|_| "Airbrush".to_owned());
    for nome in nomes.split(',') {
        let raio = raio_da_midia(nome).unwrap();
        for (vn, kv) in variantes() {
            let t: Vec<[f64; 5]> = std::thread::scope(|s| {
                let hs: Vec<_> = (0..n)
                    .map(|k| {
                        let kv = &kv;
                        s.spawn(move || {
                            com_sub(kv, || {
                                let (m, c) =
                                    cena_seca_semente(&com(de_fabrica(nome), kv), raio, &[], k);
                                [c.pontas, c.rampa, c.rampa_cob, m.textura, m.cobertura]
                            })
                        })
                    })
                    .collect();
                hs.into_iter().map(|h| h.join().unwrap()).collect()
            });
            let med = |i: usize| {
                let mut v: Vec<f64> = t.iter().map(|x| x[i]).collect();
                v.sort_by(f64::total_cmp);
                v[v.len() / 2]
            };
            println!(
                "[pontas] {nome}@{vn}: pontas {:.3} · rampa {:.3} · rampa_cob {:.3} · textura {:.2} · cobertura {:.3} (medianas de {n})",
                med(0),
                med(1),
                med(2),
                med(3),
                med(4)
            );
        }
    }
}

/// A tela da cena seca de `p` a `raio`, com a semente dos traços e o relógio `r` da água.
pub(super) fn imagem_seca(p: &Preset, raio: f32, semente: u64, r: Relogio) -> (Vec<u8>, u32, u32) {
    let mut t = tela();
    t.set_semente_dos_tracos(semente);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.set_brush_size_px(raio);
    pinta_cena_com(&mut t, QUADROS_SECOS, r)
}

/// [`cena_seca`] numa tiragem só: os traços a partir da semente `semente` (`0` = a do produto).
fn cena_seca_semente(
    p: &Preset,
    raio: f32,
    ficheiros: &[String],
    semente: u64,
) -> (Medida, FamiliaC) {
    let (px, w, h) = imagem_seca(p, raio, semente, Relogio::DeUmaVez);
    if let Ok(dir) = std::env::var("PH2D_SONDA_OUT") {
        let mut f = format!("P6\n{w} {h}\n255\n").into_bytes();
        px.as_chunks::<4>()
            .0
            .iter()
            .for_each(|q| f.extend_from_slice(&q[..3]));
        std::fs::create_dir_all(&dir).unwrap();
        for nome in ficheiros {
            std::fs::write(format!("{dir}/midia_{nome}.ppm"), &f).unwrap();
        }
    }
    (mede(&px, w, RECTO), FamiliaC::de(&px, w))
}

/// `n` do lado do Rebelle `r` do ponto médio entre ele e o controlo `c`.
fn no_vale(n: f64, r: f64, c: f64) -> bool {
    (n - r).abs() < 0.5 * (c - r).abs()
}

/// **Sonda do custo das mídias** — o CPU do processo (todas as threads) de cada cena da antiga `cada_preset_parece_a_sua_midia` (retirada 02/11: paridade com o Rebelle),
/// uma de cada vez: as mídias e o controlo (o Wet Round) a cada largura de foto.
#[test]
#[ignore = "sonda: o custo de cada cena das mídias"]
fn sonda_custo_das_midias() {
    fn cpu_s() -> f64 {
        let st = std::fs::read_to_string("/proc/self/stat").unwrap();
        let f: Vec<&str> = st.rsplit_once(')').unwrap().1.split_whitespace().collect();
        (f[11].parse::<f64>().unwrap() + f[12].parse::<f64>().unwrap()) / 100.0
    }
    let round = de_fabrica("Wet Round");
    let mut raios: Vec<f32> = MIDIAS.iter().map(|m| m.raio).collect();
    raios.sort_by(f32::total_cmp);
    raios.dedup();
    let cenas = MIDIAS
        .iter()
        .map(|m| (m.preset.to_owned(), de_fabrica(m.preset), m.raio))
        .chain(
            raios
                .iter()
                .map(|&r| ("controlo Wet Round".to_owned(), round.clone(), r)),
        );
    // O CPU de cada 120 quadros da cena seca do Wet Round a 20 (a água a correr e a secar).
    {
        let mut t = tela();
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(&round).is_empty());
        t.set_brush_size_px(20.0);
        let c0 = cpu_s();
        super::sonda_gestos::gestos_da_cena(&mut t);
        let mut linha = vec![format!("gestos {:.1}", cpu_s() - c0)];
        for _ in 0..QUADROS_SECOS / 120 {
            let c = cpu_s();
            super::sonda_gestos::passa_o_tempo(&mut t, 120, Relogio::DeUmaVez);
            linha.push(format!("{:.1}", cpu_s() - c));
        }
        let c = cpu_s();
        let _ = ph2d_editor_core::tool::RasterEditTool::run_full(&mut t);
        linha.push(format!("run_full {:.1}", cpu_s() - c));
        println!(
            "[custo-midia] Wet Round 20, CPU por 120 quadros: {}",
            linha.join(" · ")
        );
    }
    for (nome, p, r) in cenas {
        let (c0, t0) = (cpu_s(), std::time::Instant::now());
        let _ = imagem_seca(&p, r, 0, Relogio::DeUmaVez);
        println!(
            "[custo-midia] {nome} raio {r}: {:.1} s CPU, {:.1} s parede",
            cpu_s() - c0,
            t0.elapsed().as_secs_f64()
        );
    }
}

// `cada_preset_parece_a_sua_midia` saiu (02/11): cada asserção (o vale, a faixa do Cruzamento, a barra das Pontas e o
// controlo Wet Round) mede a distância às fotos do Rebelle — só paridade; o dono 31/10 tirou o Rebelle de referência.
// Era vermelho desde `0170428784` (Pigment mixing de fábrica: o Wet Round a Cruzamento 0,827 da foto pencil4bcolor 0,73).
// Registo: docs/Painter/BUGS_painter.md (réguas retiradas). Os helpers ficam (outros testes os usam).

/// ⭐ **O Dabber é uma esponja REDONDA** (L13, J2 20/10): o traço de pé tem a largura do deitado, como na foto
/// (`mede_familia_c.py` no `dabber.png`: 37 / 36 = 1,028). O Chato não entrava no vale da antiga régua das mídias
/// (o Wet Round também é redondo: não distingue a mídia), mas é o que distingue o Dabber da lâmina de pé do Oil
/// Bristle que o Jitter espalhava — o «rasgado» (bancada: rugosidade 2,97 × 1,05 do Rebelle; Chato 0,74). Mutação:
/// o Dabber de antes (`dab_flatten` 0,68) sangra.
#[test]
fn o_dabber_e_redondo() {
    const CHATO_DA_FOTO: f64 = 37.0 / 36.0;
    let m = MIDIAS.iter().find(|m| m.preset == "Dabber").unwrap();
    let medida = cena_seca(&de_fabrica("Dabber"), m.raio, &[]);
    let chato = Metrica::Chato.de(&medida);
    println!("[dabber] Chato {chato:.3} (foto {CHATO_DA_FOTO:.3})");
    assert!(
        (chato - CHATO_DA_FOTO).abs() <= 3.0 * Metrica::Chato.ruido(),
        "o Dabber de pé ÷ deitado {chato:.3}, a foto {CHATO_DA_FOTO:.3}: a ponta não é a esponja redonda"
    );
}

/// **SONDA (prompt 42b): as métricas de um preset com knobs trocados** — `PH2D_SONDA_PRESETS="Ink Brush,…"` (por
/// omissão a tinta e o marcador) × `PH2D_SONDA_KNOBS="nome:k=v,b.k=v|…"` (`k` no mapa do motor, `b.k` no do
/// pincel; `grao`/`rough`/`gx` = o Size do grão, a Roughness dele e o `paperGate` × x; a variante vazia é o preset de
/// fábrica), a cena seca medida como na antiga `cada_preset_parece_a_sua_midia` (retirada 02/11: paridade com o Rebelle),
/// com o vale de cada métrica. `PH2D_SONDA_RAIO=<r>` troca o raio da cena.
#[test]
#[ignore = "sonda: as métricas das variantes de um preset (prompt 42b); corre-se à mão"]
fn sonda_as_metricas_das_variantes() {
    let presets: Vec<String> = std::env::var("PH2D_SONDA_PRESETS").map_or_else(
        |_| {
            ["Ink Bullet", "Dip Pen", "Ink Brush", "Marker Bullet"]
                .map(str::to_owned)
                .to_vec()
        },
        |v| v.split(',').map(str::to_owned).collect(),
    );
    let variantes: Vec<(String, Vec<(String, f64)>)> = std::env::var("PH2D_SONDA_KNOBS")
        .map_or_else(
            |_| vec![(String::new(), Vec::new())],
            |v| {
                v.split('|')
                    .map(|s| {
                        let (n, kv) = s.split_once(':').unwrap_or((s, ""));
                        let kv = kv
                            .split(',')
                            .filter(|x| !x.is_empty())
                            .map(|x| {
                                let (k, v) = x.split_once('=').unwrap();
                                (k.to_owned(), v.parse().unwrap())
                            })
                            .collect();
                        (n.to_owned(), kv)
                    })
                    .collect()
            },
        );
    let raio_fixo: Option<f32> = std::env::var("PH2D_SONDA_RAIO")
        .ok()
        .and_then(|v| v.parse().ok());
    let round = &de_fabrica("Wet Round");
    std::thread::scope(|s| {
        for nome in &presets {
            let m = MIDIAS.iter().find(|m| m.preset == nome).unwrap();
            let raio = raio_fixo.unwrap_or(m.raio);
            let c = s.spawn(move || cena_seca(round, raio, &[]));
            let hs: Vec<_> = variantes
                .iter()
                .map(|(vn, kv)| {
                    let mut p = de_fabrica(nome);
                    for (k, x) in kv {
                        if k == "grao" {
                            // O Size do grão (`texture.size`, os dois eixos).
                            p.parametros.brush.get_mut("texture").unwrap()["size"] =
                                serde_json::json!([x, x]);
                        } else if k == "rough" {
                            p.parametros.brush.get_mut("texture").unwrap()["params"][3] =
                                serde_json::json!(x);
                        } else if k == "gx" {
                            *p.parametros.engine.get_mut("paperGate").unwrap() *= x;
                        } else if let Some(b) = k.strip_prefix("b.") {
                            p.parametros
                                .brush
                                .insert(b.to_owned(), serde_json::json!(x));
                        } else {
                            p.parametros.engine.insert(k.clone(), *x);
                        }
                    }
                    let f = format!("{nome}@{vn}");
                    (vn, s.spawn(move || cena_seca(&p, raio, &[f])))
                })
                .collect();
            let c = c.join().unwrap();
            for (vn, h) in hs {
                let n = h.join().unwrap();
                let linha: Vec<String> = m
                    .metricas
                    .iter()
                    .map(|&(met, r)| {
                        let (nv, cv) = (met.de(&n), met.de(&c));
                        let ok = match met {
                            Cruzamento => (nv - r).abs() < 3.0 * met.ruido(),
                            _ => no_vale(nv, r, cv),
                        };
                        format!(
                            "{met:?} {nv:.3} (R {r} c {cv:.3}){}",
                            if ok { "" } else { " ✗" }
                        )
                    })
                    .collect();
                println!("[metricas] {nome}@{vn} r {raio}: {}", linha.join(" · "));
            }
        }
    });
}
