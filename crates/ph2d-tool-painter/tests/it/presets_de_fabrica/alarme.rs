//! ⭐ **O ALARME da largura** (P4, [PLANO_ACELERACAO_2026-11](../../../../../docs/DevOps/PLANO_ACELERACAO_2026-11.md);
//! lei do dono 03/11): cada preset de fábrica do Wet a [`RAIOS_DO_ALARME`] — três tamanhos pequenos, uma semente, a
//! régua da fábrica — contra a largura APROVADA ([`ficheiro`]). Acusa QUAIS pincéis mudaram e escreve-os no relatório
//! ([`super::relatorio`]); o dono decide a olho (o cursor sobre o traço) e aprovar regrava o ficheiro
//! (`PH2D_PINCEIS_APROVA=1`). Substitui os 8 irmãos `a_largura_e_a_prometida_*` (8 raios de 2 a 300 contra `2·r`):
//! a medição A×B e a prova de mutação estão na [medição P1/P4](../../../../../docs/DevOps/MEDICAO_P1_P4_LARGURA_2026-11-03.md).

use super::memoria::{custo_da_cena, em_paralelo_pela_memoria};
use super::relatorio::{self, Figura, Parte};
use super::*;

/// Os tamanhos do alarme (raio do slider, px). Pequenos: o custo de uma medição cresce com a área da cena (CPU medido
/// 03/11, `sonda_custo_da_largura`, Blender Soft: 3,5 → 8 s · 26 → 34 s · 300 → 498 s).
const RAIOS_DO_ALARME: [f32; 3] = [2.0, 7.0, 26.0];
/// O que o alarme deixa passar (px). A régua é determinística (uma semente, o relógio fixo): isto é a folga de vírgula
/// flutuante entre máquinas, não a promessa — a promessa (± 2 px) julga-a o dono, a olho.
const TOLERANCIA_DO_ALARME_PX: f32 = 0.25;
/// A única semente do alarme: mudar o sorteio é mudar o pincel, e o relatório mostra-o.
const SEMENTE_DO_ALARME: u64 = 0;

fn ficheiro() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/it/presets_de_fabrica/larguras_aprovadas.tsv")
}

const CABECALHO: &str = "\
# A LARGURA APROVADA de cada pincel de fábrica do Wet: a largura (px) que o traço deixa ao tamanho 2·raio do slider,
# pela régua da fábrica (uma semente). Quem a compara é `a_largura_de_fabrica_e_a_aprovada`; quem a aprova é o dono,
# pelo relatório `target/relatorio-pinceis/index.html`. Aprovar regrava este ficheiro:
#   PH2D_PINCEIS_APROVA=1 cargo test -p ph2d-tool-painter --profile smoke --test it a_largura_de_fabrica_e_a_aprovada
";

/// Os raios do cabeçalho e, por preset, uma largura por raio.
fn aprovadas() -> (Vec<f32>, BTreeMap<String, Vec<f32>>) {
    let texto = std::fs::read_to_string(ficheiro()).unwrap_or_default();
    let mut raios = Vec::new();
    let mut por_nome = BTreeMap::new();
    for linha in texto.lines().filter(|l| !l.trim().is_empty()) {
        if let Some(r) = linha.strip_prefix("preset\t") {
            raios = r.split('\t').map(|v| v.parse().unwrap()).collect();
        } else if !linha.starts_with('#') {
            let mut c = linha.split('\t');
            let nome = c.next().unwrap().to_owned();
            por_nome.insert(nome, c.map(|v| v.parse().unwrap()).collect());
        }
    }
    (raios, por_nome)
}

fn grava_aprovadas(medidas: &BTreeMap<String, Vec<f32>>) {
    let mut t = String::from(CABECALHO);
    t.push_str("preset");
    for r in RAIOS_DO_ALARME {
        t.push_str(&format!("\t{r}"));
    }
    t.push('\n');
    for (nome, ls) in medidas {
        t.push_str(nome);
        for l in ls {
            t.push_str(&format!("\t{l:.2}"));
        }
        t.push('\n');
    }
    std::fs::write(ficheiro(), t).unwrap();
}

/// Uma medição: a largura de `p` a `raio` e o traço recortado com o cursor por cima.
struct Medida {
    largura: f32,
    traco: (Vec<u8>, u32, u32),
}

fn mede(p: &Preset, raio: f32, crua: bool) -> Medida {
    let (px, w, h, regua) = imagem_da_largura(p, raio, crua, Relogio::DeUmaVez, SEMENTE_DO_ALARME);
    Medida {
        largura: regua(&px, w, h),
        traco: relatorio::traco_com_cursor(&px, w, h, raio),
    }
}

#[test]
fn a_largura_de_fabrica_e_a_aprovada() {
    let presets: Vec<Preset> = ficheiros_do_wet()
        .into_iter()
        .map(|(_, b)| ler(&b).unwrap())
        .collect();
    // (preset, raio, crua): todos com a tabela do preset; o Wet Round também cru (o controlo).
    let casos: Vec<(usize, usize, bool)> = presets
        .iter()
        .enumerate()
        .flat_map(|(k, p)| {
            let crua = p.parametros.name == "Wet Round";
            (0..RAIOS_DO_ALARME.len()).flat_map(move |i| {
                [(k, i, false)]
                    .into_iter()
                    .chain(crua.then_some((k, i, true)))
            })
        })
        .collect();
    let custos: Vec<u64> = casos
        .iter()
        .map(|&(k, i, c)| custo_da_cena(&presets[k], RAIOS_DO_ALARME[i], c))
        .collect();
    let fios = std::thread::available_parallelism().map_or(8, std::num::NonZero::get);
    let medidas = em_paralelo_pela_memoria(&custos, fios, |j| {
        let (k, i, c) = casos[j];
        mede(&presets[k], RAIOS_DO_ALARME[i], c)
    });
    let mut hoje: BTreeMap<String, Vec<Medida>> = BTreeMap::new();
    let mut crua = Vec::new();
    for (&(k, _, c), m) in casos.iter().zip(medidas) {
        if c {
            crua.push(m.largura);
        } else {
            hoje.entry(presets[k].parametros.name.clone())
                .or_default()
                .push(m);
        }
    }
    let (raios, aprovadas) = aprovadas();
    let mesmos_raios = raios == RAIOS_DO_ALARME;
    let perto = |a: &[f32], b: &[f32]| {
        a.len() == b.len()
            && a.iter()
                .zip(b)
                .all(|(x, y)| (x - y).abs() <= TOLERANCIA_DO_ALARME_PX)
    };
    // O controlo: a lei CRUA do Wet Round (o raio direto ao motor) sai do aprovado — a régua vê uma mudança de largura.
    if mesmos_raios && let Some(a) = aprovadas.get("Wet Round") {
        assert!(
            !perto(&crua, a),
            "controlo: a lei crua do Wet Round {crua:?} cabe no aprovado {a:?} — o alarme não mede nada"
        );
    }
    let mut partes = Vec::new();
    let mut acusa = Vec::new();
    for (nome, ms) in &hoje {
        let agora: Vec<f32> = ms.iter().map(|m| m.largura).collect();
        let antes = aprovadas.get(nome).filter(|_| mesmos_raios);
        if antes.is_some_and(|a| perto(&agora, a)) {
            continue;
        }
        let figuras = ms
            .iter()
            .zip(RAIOS_DO_ALARME)
            .enumerate()
            .map(|(i, (m, r))| {
                let era = antes.map_or(String::from("sem aprovada"), |a| {
                    format!("aprovada {:.1}", a[i])
                });
                Figura {
                    rgba: m.traco.0.clone(),
                    w: m.traco.1,
                    h: m.traco.2,
                    escala: (96.0 / (2.0 * r)).clamp(1.0, 6.0) as u32,
                    legenda: format!("tamanho {} px · agora {:.1} px ({era})", 2.0 * r, m.largura),
                }
            })
            .collect();
        acusa.push(format!("{nome}: {agora:?} (aprovada {antes:?})"));
        partes.push(Parte {
            pincel: nome.clone(),
            texto: String::from(
                "A largura do traço mudou. O anel é o cursor no tamanho do pincel — o traço deve enchê-lo. O número lê o \
                 MIOLO do traço (área ÷ pico) e pode dar «mais fino» onde a mancha enche o anel: serve para acusar a \
                 mudança; quem julga é o olho.",
            ),
            figuras,
        });
    }
    let orfas: Vec<&String> = aprovadas
        .keys()
        .filter(|n| !hoje.contains_key(*n))
        .collect();
    let indice = relatorio::grava("2-largura", "Largura", &partes);
    if relatorio::aprova() {
        let todas = hoje
            .iter()
            .map(|(n, ms)| (n.clone(), ms.iter().map(|m| m.largura).collect()))
            .collect();
        grava_aprovadas(&todas);
        println!(
            "[alarme] {} aprovado(s) — {}",
            acusa.len(),
            indice.display()
        );
        return;
    }
    assert!(
        acusa.is_empty() && orfas.is_empty() && mesmos_raios,
        "a largura mudou (o dono decide pelo relatório {}; aprovar: {}=1):\n{}\nsem preset: {orfas:?}",
        indice.display(),
        relatorio::APROVA,
        acusa.join("\n")
    );
}
