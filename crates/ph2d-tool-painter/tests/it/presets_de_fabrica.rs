//! **Os presets de FÁBRICA** (`assets/brushes/<motor do meio>/`, W1; os quatro meios desde a J3): contados, ao byte,
//! e a miniatura de cada um é a que o motor de HOJE pinta — o corpus de regressão do motor (02 §5). As receitas
//! moram nas famílias (`familias/`, 04_BANCADA §9).
//!
//! Os valores vêm do doc 02 §6.9 (05 §1.2 diz como as colunas viram unidades do motor). Regerar depois
//! de uma mudança de lei do motor (a miniatura acusa) — o MESMO comando para o Wet e para os outros meios:
//!
//! ```text
//! PH2D_REGERA_PRESETS=1 cargo test -p ph2d-tool-painter --profile smoke --test it \
//!     gera_os_presets_de_fabrica -- --ignored --nocapture
//! ```
//!
//! `PH2D_REGERA_SO="Ink Brush,Dip Pen"` regera só esses (pelo nome; os nomes da fábrica são únicos nos 4 meios).

use super::receitas_de_fabrica::{
    DIREITA, PRESETS_DE_FABRICA, Receita, ReceitaMeio, SPACING_DA_FABRICA, grao, receitas,
    receitas_dos_meios,
};
use super::sonda_gestos::{fnv1a, largura_que_se_ve, traco_recto_alto};
use ph2d_brush_presets::{
    Biblioteca, Origem, Parametros, Preset, caminho_relativo, escrever, gravar, ler,
};
use ph2d_tool_painter::{
    LINHAS_DA_MAO, MEIO_DO_PRESET, MOTOR_DOS_PRESETS, PaintMedia, PainterTool, WetKnobs,
    campos_do_meio, meio_do_preset, miniatura_do_preset, motor_do_meio, nome_do_meio,
};
use ph2d_wet_paint::tuning::Knob;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Os raios do MOTOR da lei raio → largura (W2, 05 §4b): geométricos de 1,5 a 1 100 — a largura 600
/// (o raio 300 do portão) pede ~860 no Wet Round. O slider pára em 512 (`BRUSH_SIZE_MAX_PX`): acima, a
/// sonda pede metade e o motor recebe o dobro (`set_wet_largura_crua(2)`).
const PONTOS_DA_LEI: usize = 64;
const RAIO_MIN: f32 = 1.5;
const RAIO_MAX: f32 = 1100.0;
/// Os quadros de 60 Hz depois do traço: a largura que se promete é a que FICA, com a água assentada
/// (a 0 quadros o Wet Round a raio 30 lê 8 px; aos 8 s, 32 — 05 §4b).
const QUADROS_DA_LEI: u32 = 480;

fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes")
}

/// O pincel de fábrica do meio inteiro, com os knobs da receita. Tamanho e opacidade NÃO são fixados
/// (a tabela não os tem): fica o do artista.
///
/// ⚠️ **Parte do ficheiro de fábrica que JÁ existe** (prompt 35): desde a W2 do plano 27 as cerdas vivem no
/// Shape (`WetKnobs::cerdas_no_shape`) e gravar não as põe no motor — capturar de fresco perdia a ponta de
/// cada preset. Só um preset sem ficheiro nasce da captura.
fn parametros(r: &Receita) -> Parametros {
    let rel = caminho_relativo(MOTOR_DOS_PRESETS, r.meio, r.grupo, r.nome);
    let mut p = match std::fs::read(raiz().join(&rel)) {
        Ok(bytes) => ler(&bytes).expect("o ficheiro de fábrica lê-se").parametros,
        // As formas (prompt 44) nascem do Wet Round — a ponta, as cerdas e a lei dele; a receita muda o resto.
        Err(_) if matches!(r.nome, "Wet Flat" | "Gouache Flat") => {
            let rel = caminho_relativo(MOTOR_DOS_PRESETS, "Watercolor", "Fluid", "Wet Round");
            let mut p = ler(&std::fs::read(raiz().join(rel)).unwrap())
                .expect("o Wet Round de fábrica")
                .parametros;
            (p.name, p.group) = (r.nome.to_owned(), r.grupo.to_owned());
            p.brush
                .insert("dab_quina".to_owned(), serde_json::json!(0.0));
            p.engine.insert("pressureSize".to_owned(), 0.9);
            p.engine.insert("pressureOpacity".to_owned(), 0.0);
            p
        }
        // Os do prompt 53 nascem do ficheiro do vizinho, como a sonda do doc 52 os mediu.
        Err(_) if r.nasce_de.is_some() => {
            let base = r.nasce_de.unwrap();
            let mut p = super::midias_de_fabrica::de_fabrica(base).parametros;
            (p.name, p.group) = (r.nome.to_owned(), r.grupo.to_owned());
            p
        }
        Err(_) => {
            let mut t = PainterTool::default();
            t.set_paint_media(PaintMedia::WetPaint);
            t.wet_preset_capturar(&Parametros::novo(MEIO_DO_PRESET, r.grupo, r.nome))
                .0
        }
    };
    p.volumes.size_px = r.size_px;
    p.volumes.opacity = Some(r.opacity);
    p.volumes.water = Some(r.water);
    p.tilt = Some(r.tilt);
    // As opções da W1g não são do pincel de fábrica: aplicá-lo não mexe nelas (a regra do preset antigo).
    (p.live_water, p.heavy_edge) = (None, None);
    assert!(
        p.brush
            .insert("hardness".to_owned(), serde_json::json!(r.hardness))
            .is_some(),
        "campo hardness existe"
    );
    assert!(
        p.brush
            .insert("spacing".to_owned(), serde_json::json!(SPACING_DA_FABRICA))
            .is_some(),
        "campo spacing existe"
    );
    // Um ficheiro de antes do Continuous não o grava; o campo é do `CAMPOS_DO_PINCEL`.
    p.brush
        .insert("continuous_spacing".to_owned(), serde_json::json!(true));
    if let Some((flatten, angulo)) = r.achatado {
        for (campo, v) in [
            ("dab_flatten", serde_json::json!(flatten)),
            ("dab_angle_deg", serde_json::json!(angulo)),
        ] {
            assert!(
                p.brush.insert(campo.to_owned(), v).is_some(),
                "campo {campo} existe"
            );
        }
    }
    // Um campo que o ficheiro ainda não grava (a ponta reta do Square Hard) entra também — desde que seja do pincel.
    for (campo, json) in r.pincel {
        let v: serde_json::Value = serde_json::from_str(json).expect("o JSON do campo");
        assert!(
            ph2d_tool_painter::CAMPOS_DO_PINCEL.contains(campo),
            "campo {campo} existe"
        );
        p.brush.insert((*campo).to_owned(), v);
    }
    if let Some((kind, lado)) = r.grao {
        assert!(
            p.brush
                .insert("texture".to_owned(), grao(kind, lado))
                .is_some()
        );
    }
    let d = WetKnobs::DEFAULT;
    for (chave, v) in [
        ("pigmentPerDab", r.pigment),
        ("paperGate", r.gate),
        ("evaporation", r.evaporation),
        ("pickup", d.get(Knob::Pickup) * r.pickup),
        ("edgeDarkening", r.edge),
    ]
    .into_iter()
    .chain(r.extras.iter().copied())
    {
        // Um knob que o ficheiro ainda não grava (a pressão do pincel, prompt 44) entra também — desde que
        // seja do REGISTO.
        assert!(
            ph2d_wet_paint::tuning::KNOB_DEFS
                .iter()
                .any(|d| d.key == chave),
            "knob {chave} existe"
        );
        p.engine.insert(chave.to_owned(), v);
    }
    // As cerdas: no motor só num ficheiro de antes da W2; depois dela estão no Shape, e o ficheiro é a fonte.
    for (chave, v) in [
        ("bristleSize", r.bristle_size),
        ("bristleCount", r.bristle_count),
    ] {
        if let Some(antigo) = p.engine.get_mut(chave) {
            *antigo = v;
        }
    }
    p
}

/// O caminho de fábrica de uma receita de Digital/Watercolor/Impasto.
pub(super) fn relativo_do_meio(r: &ReceitaMeio) -> String {
    caminho_relativo(motor_do_meio(r.meio), r.pasta, r.grupo, r.nome)
}

/// **O preset de uma receita de Digital/Watercolor/Impasto** (sem a miniatura): o pincel por omissão do meio — ou o
/// ficheiro de `nasce_de` — com o que a receita fixa. Cada nome é VALIDADO: o campo é do pincel do meio
/// (`campos_do_meio`), a linha da mão existe, o meio a lê e o valor está na faixa; a Watercolor não lê a Strength.
/// ⚠️ A tabela de largura fica vazia: é a promessa do slider do Wet (`wet_preset_aplicar` lê-a; `preset_aplicar` nos
/// outros meios não a lê).
pub(super) fn parametros_do_meio(r: &ReceitaMeio) -> (Parametros, BTreeMap<String, Vec<u8>>) {
    let m = r.meio;
    assert!(m != PaintMedia::WetPaint, "{}: um preset do Wet Paint é uma `Receita`", r.nome);
    let (mut p, mut texturas) = match r.nasce_de {
        Some(base) => {
            let b = super::midias_de_fabrica::de_fabrica(base);
            assert_eq!(
                meio_do_preset(&b.parametros.media),
                Some(m),
                "{}: nasce de {base}, de outro meio",
                r.nome
            );
            (b.parametros, b.texturas)
        }
        None => {
            let mut t = PainterTool::default();
            t.set_paint_media(m);
            t.preset_capturar(&Parametros::novo(nome_do_meio(m), r.grupo, r.nome))
        }
    };
    (p.name, p.group) = (r.nome.to_owned(), r.grupo.to_owned());
    let campos = campos_do_meio(m);
    for (campo, json) in r.pincel {
        assert!(
            campos.contains(campo),
            "{}: o campo {campo} não é do pincel do meio {m:?} (`campos_do_meio`)",
            r.nome
        );
        let v: Value = serde_json::from_str(json)
            .unwrap_or_else(|e| panic!("{}: o JSON do campo {campo}: {e}", r.nome));
        p.brush.insert((*campo).to_owned(), v);
    }
    for &(chave, v) in r.mao {
        let l = LINHAS_DA_MAO
            .iter()
            .find(|l| l.chave() == chave)
            .unwrap_or_else(|| panic!("{}: a linha da mão {chave} não existe", r.nome));
        assert!(l.lida_por(m), "{}: o meio {m:?} não lê a linha da mão {chave}", r.nome);
        assert!(l.limpa(v) == v, "{}: {chave} = {v} fora da faixa {:?}", r.nome, l.faixa());
        p.engine.insert(chave.to_owned(), v);
    }
    assert!(
        !(m == PaintMedia::Watercolor && r.opacity.is_some()),
        "{}: a Watercolor não lê a Strength",
        r.nome
    );
    p.volumes.size_px = r.size_px;
    p.volumes.opacity = r.opacity;
    if let Some(modo) = r.modo {
        p.paint_mode = Some(modo.to_owned());
    }
    p.width_table.clear();
    // As imagens que a receita DESENHA (J4 `texturas`): por cima das do `nasce_de`, pelo id do ficheiro.
    for &(id, desenha) in r.texturas {
        texturas.insert(id.to_owned(), desenha());
    }
    (p, texturas)
}

/// O preset de fábrica inteiro de uma receita de outro meio (com a miniatura que o motor pinta hoje).
pub(super) fn preset_do_meio(r: &ReceitaMeio) -> Preset {
    let (parametros, texturas) = parametros_do_meio(r);
    let mut p = Preset {
        parametros,
        origem: Origem::Fabrica,
        miniatura: ph2d_brush_presets::Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![255; 4],
        },
        texturas,
    };
    p.miniatura = miniatura_do_preset(&p);
    p
}

/// `a` e `b` iguais, os números a menos de 1e-4 (o pincel guarda `f32`).
fn json_perto(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            (x.as_f64().unwrap() - y.as_f64().unwrap()).abs() <= 1e-4
        }
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(x, y)| json_perto(x, y))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| json_perto(v, w)))
        }
        _ => a == b,
    }
}

/// ⭐ **O que a receita fixa É o que o pincel fica**: o preset aplicado num pincel novo arma o meio da receita, nada
/// falha, e cada campo, linha da mão, volume e modo lidos de volta são os da receita — um valor que o motor recusa
/// ou ignora (tipo errado, fora do domínio) acusa aqui, nunca vira um preset que mente.
pub(super) fn confere_receita_do_meio(r: &ReceitaMeio, p: &Preset) {
    let mut t = PainterTool::default();
    assert_eq!(t.preset_aplicar(p), Ok(Vec::new()), "{}: o preset não se aplica inteiro", r.nome);
    assert_eq!(t.paint_media(), r.meio, "{}: o preset arma outro meio", r.nome);
    let (lido, _) = t.preset_capturar(&p.parametros);
    for (campo, json) in r.pincel {
        let v: Value = serde_json::from_str(json).unwrap();
        assert!(
            lido.brush.get(*campo).is_some_and(|w| json_perto(&v, w)),
            "{}: o campo {campo} pedia {v}, o pincel ficou com {:?}",
            r.nome,
            lido.brush.get(*campo)
        );
    }
    for &(chave, v) in r.mao {
        let w = lido.engine.get(chave).copied();
        assert!(
            w.is_some_and(|w| (w - v).abs() <= 1e-4),
            "{}: a mão {chave} pedia {v}, o pincel ficou com {w:?}",
            r.nome
        );
    }
    if let Some(d) = r.size_px {
        let w = lido.volumes.size_px.unwrap_or(f64::NAN);
        assert!((w - d).abs() <= 1e-3, "{}: o tamanho pedia {d}, ficou {w}", r.nome);
    }
    if let Some(o) = r.opacity {
        let w = lido.volumes.opacity.unwrap_or(f64::NAN);
        assert!((w - o).abs() <= 1e-4, "{}: a Strength pedia {o}, ficou {w}", r.nome);
    }
    if let Some(modo) = r.modo {
        assert_eq!(lido.paint_mode.as_deref(), Some(modo), "{}: o modo", r.nome);
    }
}

fn raios_da_lei() -> Vec<f32> {
    (0..PONTOS_DA_LEI)
        .map(|i| {
            let f = i as f32 / (PONTOS_DA_LEI - 1) as f32;
            (RAIO_MIN * (RAIO_MAX / RAIO_MIN).powf(f) * 10.0).round() / 10.0
        })
        .collect()
}

/// O preset com a tela DIREITA: a largura é a do traço; o escorrimento do Wet Drip é da tela (W1e).
fn direito(p: &Preset) -> Preset {
    let mut d = p.clone();
    d.parametros.tilt = Some(DIREITA);
    d
}

/// A largura que o pincel `p` deixa a `raio`: `crua` = o raio chega ao MOTOR sem a promessa.
pub(super) fn largura_de(p: &Preset, raio: f32, crua: bool) -> f32 {
    let escala = if crua { (raio / 500.0).max(1.0) } else { 1.0 };
    let (px, w, h) = traco_recto_alto(raio / escala, raio, QUADROS_DA_LEI, |t| {
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(&direito(p)).is_empty());
        if crua {
            t.set_wet_largura_crua(escala);
        }
    });
    largura_que_se_ve(&px, w, h)
}

/// A lei CRUA medida: raio do motor → largura, numa fila do MAIOR para o menor com `PH2D_LEI_FIOS` trabalhadores
/// (por omissão um por raio). ⚠️ A memória é a do raio: o Dabber (Jitter 0,5) a 64 de uma vez passava dos 48 GB; depois da
/// margem da janela pelo CAMINHO (`Trail::on_segment_no_caminho`) 21,9 GiB do tecto de 24 (prompt 53); com UMA lane por
/// ponta (L13) o raio 1 100 pesa 1,7 GiB (era 4,3) — mas a fatia é da LINHA e o `/tmp` (tmpfs) dos agentes conta nela
/// (14,6 G de `shmem` a 20/10): o Dabber regerou-se a `PH2D_LEI_FIOS=4`, pico 5,3 GiB, 4,8 min (03_MOTORES §4 L13).
fn tabela_de_largura(p: &Preset) -> Vec<[f64; 2]> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let raios = raios_da_lei();
    let fios = std::env::var("PH2D_LEI_FIOS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(raios.len());
    let proximo = AtomicUsize::new(0);
    let mut larguras = vec![0.0_f32; raios.len()];
    std::thread::scope(|s| {
        let h: Vec<_> = (0..fios)
            .map(|_| {
                s.spawn(|| {
                    let mut feitas = Vec::new();
                    loop {
                        let k = proximo.fetch_add(1, Ordering::Relaxed);
                        let Some(i) = raios.len().checked_sub(k + 1) else {
                            break feitas;
                        };
                        feitas.push((i, largura_de(p, raios[i], true)));
                    }
                })
            })
            .collect();
        for (i, l) in h.into_iter().flat_map(|h| h.join().unwrap()) {
            larguras[i] = l;
        }
    });
    raios
        .iter()
        .zip(larguras)
        // O raio pelo menor texto que volta ao mesmo f32 (7.8, não 7.800000190734863).
        .map(|(&r, l)| [format!("{r}").parse().unwrap(), f64::from(l)])
        .collect()
}

#[test]
#[ignore = "gerador: escreve assets/brushes/<motor do meio> (PH2D_REGERA_PRESETS=1)"]
fn gera_os_presets_de_fabrica() {
    let escreve = std::env::var("PH2D_REGERA_PRESETS").as_deref() == Ok("1");
    // `PH2D_REGERA_SO="Nome,Nome"`: só esses (cada preset do Wet custa ~2 min de lei da largura sob carga).
    let so = std::env::var("PH2D_REGERA_SO").ok();
    let pedido = |nome: &str| so.as_deref().is_none_or(|s| s.split(',').any(|n| n == nome));
    if let Some(s) = so.as_deref() {
        let nomes: Vec<&str> = receitas()
            .map(|r| r.nome)
            .chain(receitas_dos_meios().map(|r| r.nome))
            .collect();
        let sem: Vec<&str> = s.split(',').filter(|n| !nomes.contains(n)).collect();
        assert!(sem.is_empty(), "PH2D_REGERA_SO: sem receita {sem:?}");
    }
    // Os outros meios: sem tabela de largura (a promessa do slider é do Wet), a receita é a verdade inteira.
    for r in receitas_dos_meios().filter(|r| pedido(r.nome)) {
        let preset = preset_do_meio(r);
        confere_receita_do_meio(r, &preset);
        let rel = relativo_do_meio(r);
        println!(
            "[fabrica] {:<14} miniatura fnv {:#018x} · {rel}",
            r.nome,
            fnv1a(&preset.miniatura.rgba)
        );
        if escreve {
            gravar(&raiz(), &rel, &preset).unwrap();
        }
    }
    for r in receitas().filter(|r| pedido(r.nome)) {
        let mut preset = Preset {
            parametros: parametros(r),
            origem: Origem::Fabrica,
            miniatura: ph2d_brush_presets::Miniatura {
                largura: 1,
                altura: 1,
                rgba: vec![255; 4],
            },
            texturas: Default::default(),
        };
        // `PH2D_REGERA_SO_MINIATURA=1`: a tabela do ficheiro fica (só a miniatura, quando a lei da largura não mudou).
        if std::env::var("PH2D_REGERA_SO_MINIATURA").as_deref() != Ok("1") {
            preset.parametros.width_table = tabela_de_largura(&preset);
        }
        if r.nome == "Wet Round" {
            // A tabela que o motor embute (`wetpaint/largura.rs::LARGURA_DO_WET_ROUND`), já limpa.
            let mut t = PainterTool::default();
            assert!(t.wet_preset_aplicar(&preset).is_empty());
            println!(
                "[fabrica] LARGURA_DO_WET_ROUND = {:?}",
                t.wet_largura_tabela()
            );
        }
        preset.miniatura = miniatura_do_preset(&preset);
        let larguras: Vec<String> = preset
            .parametros
            .width_table
            .iter()
            .map(|[r, l]| format!("{r}→{l}"))
            .collect();
        println!(
            "[fabrica] {:<14} miniatura fnv {:#018x} · pico {:.0} MiB · largura {}",
            r.nome,
            fnv1a(&preset.miniatura.rgba),
            pico_mib(),
            larguras.join(" ")
        );
        if escreve {
            let rel = caminho_relativo(MOTOR_DOS_PRESETS, r.meio, r.grupo, r.nome);
            gravar(&raiz(), &rel, &preset).unwrap();
        }
    }
    if !escreve {
        println!("[fabrica] nada escrito (PH2D_REGERA_PRESETS=1 para gravar)");
    }
}

/// Os ficheiros de fábrica dos QUATRO meios (as leis que valem em todos: ao byte, a miniatura).
fn ficheiros_de_fabrica() -> Vec<(String, Vec<u8>)> {
    Biblioteca::carregar(&raiz(), None)
        .entradas
        .into_iter()
        .map(|e| (e.relativo, std::fs::read(&e.caminho).unwrap()))
        .collect()
}

/// Só os do Wet Paint (as leis do Wet: a largura prometida, o ruído dela).
fn ficheiros_do_wet() -> Vec<(String, Vec<u8>)> {
    ficheiros_de_fabrica()
        .into_iter()
        .filter(|(rel, _)| rel.split('/').next() == Some(MOTOR_DOS_PRESETS))
        .collect()
}

/// ⭐ **A fábrica é CONTADA das receitas**: o conjunto de ficheiros em `assets/brushes/` (as raízes dos quatro meios) é
/// EXACTAMENTE o das receitas (a base + cada família, `WET` e `MEIOS`) — nem ficheiro sem receita, nem receita sem
/// ficheiro —, cada um na pasta do seu meio e com o nome do preset, e os NOMES são únicos nos quatro meios (a bancada,
/// o `de_fabrica` e o `PH2D_REGERA_SO` procuram pelo nome).
#[test]
fn the_factory_presets_are_counted() {
    let b = Biblioteca::carregar(&raiz(), None);
    assert!(
        b.erros.is_empty(),
        "ficheiros de fábrica ilegíveis: {:?}",
        b.erros
    );
    let mut esperados: Vec<String> = receitas()
        .map(|r| caminho_relativo(MOTOR_DOS_PRESETS, r.meio, r.grupo, r.nome))
        .chain(receitas_dos_meios().map(relativo_do_meio))
        .collect();
    esperados.sort();
    let n = esperados.len();
    esperados.dedup();
    assert_eq!(esperados.len(), n, "duas receitas no mesmo caminho");
    assert_eq!(n, PRESETS_DE_FABRICA);
    let achados: Vec<String> = b.entradas.iter().map(|e| e.relativo.clone()).collect();
    let sem_ficheiro: Vec<&String> = esperados.iter().filter(|r| !achados.contains(r)).collect();
    let sem_receita: Vec<&String> = achados.iter().filter(|r| !esperados.contains(r)).collect();
    assert!(
        sem_ficheiro.is_empty() && sem_receita.is_empty(),
        "a fábrica não é a das receitas — receita sem ficheiro (regere): {sem_ficheiro:?} · ficheiro sem receita: \
         {sem_receita:?}"
    );
    let mut nomes = Vec::new();
    for e in &b.entradas {
        let m = meio_do_preset(&e.preset.parametros.media)
            .unwrap_or_else(|| panic!("{}: meio desconhecido", e.relativo));
        assert_eq!(e.partes()[0], motor_do_meio(m), "{}: fora da pasta do meio", e.relativo);
        assert_eq!(e.preset.origem, Origem::Fabrica, "{}", e.relativo);
        assert_eq!(
            e.partes()[3],
            e.preset.parametros.name,
            "o nome do ficheiro é o do preset"
        );
        assert!(
            !nomes.contains(&e.preset.parametros.name),
            "{}: o nome {:?} já existe noutro preset de fábrica",
            e.relativo,
            e.preset.parametros.name
        );
        nomes.push(e.preset.parametros.name.clone());
    }
}

/// ⭐ **Cada preset de fábrica É a sua receita**: a receita de hoje, sem pintar, dá os parâmetros do ficheiro (no Wet,
/// com a tabela de largura do ficheiro, que é medida; nos outros meios também as texturas) — mudar uma receita sem
/// regerar acusa aqui (a miniatura acusa o motor; este, a receita).
#[test]
fn cada_preset_de_fabrica_e_a_sua_receita() {
    let mut mudaram = Vec::new();
    for r in receitas() {
        let rel = caminho_relativo(MOTOR_DOS_PRESETS, r.meio, r.grupo, r.nome);
        let f = ler(&std::fs::read(raiz().join(&rel)).unwrap()).unwrap();
        let mut p = parametros(r);
        p.width_table.clone_from(&f.parametros.width_table);
        if p != f.parametros {
            mudaram.push(rel);
        }
    }
    assert!(mudaram.is_empty(), "a receita mudou e o ficheiro não — regere: {mudaram:?}");
    for r in receitas_dos_meios() {
        let rel = relativo_do_meio(r);
        let f = ler(&std::fs::read(raiz().join(&rel)).unwrap_or_else(|e| panic!("{rel}: {e}")))
            .unwrap();
        let (p, t) = parametros_do_meio(r);
        assert!(
            p == f.parametros && t == f.texturas,
            "{rel}: a receita mudou e o ficheiro não — regere (PH2D_REGERA_SO=\"{}\")",
            r.nome
        );
    }
}

/// ⭐ **A prova de ponta a ponta dos três meios** (J3), sem gravar: uma receita de cada meio (a do lápis, a da aguada e
/// a do empaste dos exemplos, `presets_dos_meios::EXEMPLOS`) vira um preset que arma o SEU meio, fixa o que a receita
/// diz (`confere_receita_do_meio`) e tem uma miniatura com tinta.
#[test]
fn uma_receita_de_cada_meio_vira_um_preset_desse_meio() {
    const PROVAS: [ReceitaMeio; 3] = [
        ReceitaMeio {
            pincel: &[("hardness", "0.3"), ("spacing", "0.05")],
            mao: &[("pressureSize", 0.6), ("pressureOpacity", 0.5), ("tiltOpacity", 0.4)],
            size_px: Some(12.0),
            opacity: Some(0.85),
            ..ReceitaMeio::digital("Pencil", "Graphite", "prova Digital")
        },
        ReceitaMeio {
            pincel: &[("granulation", "0.4"), ("wet_dilution", "0.3"), ("depth", "1.6")],
            mao: &[("pressureSize", 0.3)],
            size_px: Some(30.0),
            ..ReceitaMeio::aquarela("Watercolor", "Round", "prova Watercolor")
        },
        ReceitaMeio {
            pincel: &[("impasto_depth", "0.6"), ("impasto_body", "0.7"), ("dab_flatten", "0.4")],
            mao: &[("pressureOpacity", 0.4)],
            modo: Some("Knife"),
            ..ReceitaMeio::impasto("Oil", "Bristle", "prova Impasto")
        },
    ];
    let neutros: Vec<Vec<u8>> = [PaintMedia::Digital, PaintMedia::Watercolor, PaintMedia::Impasto]
        .into_iter()
        .map(|m| preset_do_meio(&ReceitaMeio::novo(m, "P", "G", "neutro")).miniatura.rgba)
        .collect();
    for (r, neutro) in PROVAS.iter().zip(&neutros) {
        let p = preset_do_meio(r);
        assert_eq!(p.parametros.media, nome_do_meio(r.meio));
        assert!(p.parametros.width_table.is_empty(), "{}: tabela de largura fora do Wet", r.nome);
        confere_receita_do_meio(r, &p);
        let rgba = &p.miniatura.rgba;
        let tinta = rgba.chunks(4).filter(|c| c[..3] != rgba[..3]).count();
        println!(
            "[meios] {}: {} → {} px com tinta de {} · fnv {:#018x}",
            r.nome,
            relativo_do_meio(r),
            tinta,
            rgba.len() / 4,
            fnv1a(rgba)
        );
        assert!(tinta > 100, "{}: a miniatura não tem tinta ({tinta} px)", r.nome);
        assert!(rgba != neutro, "{}: a receita pinta a miniatura do pincel neutro", r.nome);
    }
}

/// A receita recusa um campo de OUTRO meio (o da aguada num Digital) — nunca um preset com um campo que ninguém lê.
#[test]
#[should_panic(expected = "não é do pincel do meio")]
fn a_receita_recusa_um_campo_de_outro_meio() {
    let _ = parametros_do_meio(&ReceitaMeio {
        pincel: &[("granulation", "0.4")],
        ..ReceitaMeio::novo(PaintMedia::Digital, "P", "G", "errado")
    });
}

/// A receita recusa uma linha da mão que o meio não lê (a pressão → opacidade na Watercolor).
#[test]
#[should_panic(expected = "não lê a linha da mão")]
fn a_receita_recusa_uma_mao_que_o_meio_nao_le() {
    let _ = parametros_do_meio(&ReceitaMeio {
        mao: &[("pressureOpacity", 0.5)],
        ..ReceitaMeio::novo(PaintMedia::Watercolor, "P", "G", "errado")
    });
}

/// ⭐ **A REGRA do Spacing** (dono 18/10, prompt 51): cada pincel de fábrica do WET PAINT — e o que nasce SEM preset —
/// leva o Spacing [`SPACING_DA_FABRICA`] com o Continuous ligado. Os outros meios não: a regra é a da cápsula e do
/// pigmento por comprimento do Wet (o lápis do Digital vive a 0,05).
#[test]
fn a_regra_do_spacing_da_fabrica() {
    let cumpre = |b: &std::collections::BTreeMap<String, serde_json::Value>| {
        b.get("spacing")
            .and_then(serde_json::Value::as_f64)
            .is_some_and(|s| (s - SPACING_DA_FABRICA).abs() < 1e-6)
            && b.get("continuous_spacing") == Some(&serde_json::json!(true))
    };
    let fora: Vec<String> = Biblioteca::carregar(&raiz(), None)
        .entradas
        .into_iter()
        .filter(|e| e.preset.parametros.media == MEIO_DO_PRESET)
        .filter(|e| !cumpre(&e.preset.parametros.brush))
        .map(|e| e.relativo)
        .collect();
    assert!(fora.is_empty(), "fora da regra do Spacing: {fora:?}");
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    let sem_preset = t
        .wet_preset_capturar(&Parametros::novo(MEIO_DO_PRESET, "g", "sem preset"))
        .0;
    assert!(
        cumpre(&sem_preset.brush),
        "o pincel sem preset: spacing {:?} · continuous {:?}",
        sem_preset.brush.get("spacing"),
        sem_preset.brush.get("continuous_spacing")
    );
}

#[test]
fn cada_preset_de_fabrica_volta_ao_byte() {
    for (rel, b) in ficheiros_de_fabrica() {
        let p = ler(&b).unwrap();
        assert!(escrever(&p).unwrap() == b, "{rel}: escrever(ler(b)) != b");
    }
}

#[test]
fn a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje() {
    let mut vistas = Vec::new();
    for (rel, b) in ficheiros_de_fabrica() {
        let p = ler(&b).unwrap();
        let hoje = miniatura_do_preset(&p);
        let diferem = hoje
            .rgba
            .chunks(4)
            .zip(p.miniatura.rgba.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        assert!(
            hoje == p.miniatura,
            "{rel}: {diferem} px mudaram (fnv {:#018x} → {:#018x}) — o motor mudou de lei. Se foi de \
             propósito, regere a fábrica (cabeçalho deste ficheiro) e diga-o no handoff",
            fnv1a(&p.miniatura.rgba),
            fnv1a(&hoje.rgba)
        );
        vistas.push((rel, hoje.rgba));
    }
    // O leitor DECIDE: dois presets diferentes pintam miniaturas diferentes.
    for (i, (ra, a)) in vistas.iter().enumerate() {
        for (rb, b) in &vistas[i + 1..] {
            assert_ne!(
                a, b,
                "{ra} e {rb} pintam igual: um knob do preset não tem leitor"
            );
        }
    }
}

/// Os raios do portão da W2 (05 §3): fora da tabela, de 2 a 300.
const RAIOS_DA_PROMESSA: [f32; 8] = [2.0, 3.5, 7.0, 13.0, 26.0, 55.0, 120.0, 300.0];
/// A promessa do dono (05 §4.1): o número do painel é a largura que se vê ± 2 px — e, acima de 100 px,
/// ± 2 %: a lei raio → largura ONDULA de raio para raio (a franja das cerdas), e nenhuma tabela a segue
/// melhor (medido 11/10, 05 §4b W2: 64 e 128 pontos erram o mesmo, ≤ 1,8 %; cinco traços ao mesmo raio
/// variam ±1–3 px).
const TOLERANCIA_PX: f32 = 2.0;
const TOLERANCIA_REL: f32 = 0.02;
/// ⚠️ **Os pincéis com GRÃO (o slot Grain armado: grafite, carvão, os pastéis) — ± 10 %**: o grão e o
/// papel têm estrutura, e a largura a ¼ do máximo varia com ONDE o traço cai (medido 11/10,
/// `sonda_ruido_da_largura`, o traço a descer 30 px de cada vez: até 7,7 % no Graphite e 9,6 % no Charcoal
/// a 52 px; 3,8 % a 240). A cura é um grão sem estrutura grande, não a tabela (05 §4b).
const TOLERANCIA_REL_GRAO: f32 = 0.10;

/// As larguras de `p` nos [`RAIOS_DA_PROMESSA`], com a promessa (`crua = false`) ou sem ela.
fn larguras_da_promessa(p: &Preset, crua: bool) -> Vec<f32> {
    std::thread::scope(|s| {
        let h: Vec<_> = RAIOS_DA_PROMESSA
            .iter()
            .map(|&r| s.spawn(move || largura_de(p, r, crua)))
            .collect();
        h.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

/// `tecto`: o diâmetro máximo do preset (`size_max_px`, J3 decisão 8) — acima dele a promessa é o tecto.
fn fora_da_promessa(larguras: &[f32], rel: f32, tecto: f32) -> Vec<String> {
    RAIOS_DA_PROMESSA
        .iter()
        .zip(larguras)
        .filter(|(r, l)| (*l - (2.0 * *r).min(tecto)).abs() > TOLERANCIA_PX.max(rel * (2.0 * *r).min(tecto)))
        .map(|(r, l)| format!("raio {r}: {l} px (pedido {}, tecto {tecto})", 2.0 * r))
        .collect()
}

/// **A LARGURA PROMETIDA (W2)**: em cada preset de fábrica, o traço a raio `r` mede `2·r ± 2 px` (± 2 %
/// acima de 100 px), de 2 a 300 — raios fora da tabela do preset. CONTROLO dentro: a lei CRUA (o raio direto ao motor, a de antes
/// da W2) reprova no mesmo pincel.
#[test]
fn a_largura_e_a_prometida() {
    let mut falhas = Vec::new();
    // Só o Wet: a tabela de largura é a promessa do slider dele (os outros meios não a gravam nem a leem).
    let ficheiros = ficheiros_do_wet();
    // Os presets em paralelo (cada um já pinta os seus raios em paralelo): medido 11/10, 55 → 51 s.
    let medidos: Vec<(Preset, Vec<f32>, Option<Vec<f32>>)> = std::thread::scope(|s| {
        let h: Vec<_> = ficheiros
            .iter()
            .map(|(rel, b)| {
                s.spawn(move || {
                    let p = ler(b).unwrap();
                    let (larguras, crua) = std::thread::scope(|s| {
                        let crua = rel
                            .ends_with("Wet Round.ph2d-brush.png")
                            .then(|| s.spawn(|| larguras_da_promessa(&p, true)));
                        let larguras = larguras_da_promessa(&p, false);
                        (larguras, crua.map(|h| h.join().unwrap()))
                    });
                    (p, larguras, crua)
                })
            })
            .collect();
        h.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for ((rel, _), (p, larguras, crua)) in ficheiros.iter().zip(medidos) {
        println!("[largura] {rel}: {larguras:?} (pedidas {RAIOS_DA_PROMESSA:?} × 2)");
        let com_grao = p
            .parametros
            .brush
            .get("texture")
            .and_then(|t| t.get("kind"))
            .is_some_and(|k| k != "None");
        let tol = if com_grao {
            TOLERANCIA_REL_GRAO
        } else {
            TOLERANCIA_REL
        };
        let tecto = p.parametros.brush.get("size_max_px").and_then(serde_json::Value::as_f64).map_or(f32::INFINITY, |d| d as f32);
        for f in fora_da_promessa(&larguras, tol, tecto) {
            falhas.push(format!("{rel}: {f}"));
        }
        if let Some(crua) = crua {
            assert!(
                !fora_da_promessa(&crua, TOLERANCIA_REL, f32::INFINITY).is_empty(),
                "controlo: a lei crua do Wet Round já cumpria a promessa — o portão não mede nada"
            );
        }
    }
    assert!(
        falhas.is_empty(),
        "a largura não é a do painel:\n{}",
        falhas.join("\n")
    );
}

/// O pincel que nasce SEM preset tem a lei da largura do Wet Round de fábrica (o `WetKnobs::DEFAULT` é
/// o Wet Round): a tabela embutida no motor é a do ficheiro.
#[test]
fn a_lei_de_omissao_e_a_do_wet_round() {
    let b = Biblioteca::carregar(&raiz(), None);
    let e = b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Wet Round")
        .expect("o Wet Round de fábrica");
    let mut t = PainterTool::default();
    assert!(t.wet_preset_aplicar(&e.preset).is_empty());
    let mut nasce = PainterTool::default();
    nasce.set_paint_media(PaintMedia::WetPaint);
    assert_eq!(
        nasce.wet_largura_tabela(),
        t.wet_largura_tabela(),
        "a LARGURA_DO_WET_ROUND do motor não é a do ficheiro: regere-a (o gerador imprime-a)"
    );
}

/// **Sonda de custo da miniatura** — o número que decide onde o preview vivo corre (no quadro ou numa
/// thread). Mínimo de 5 corridas por preset, a tela inteira e só o traço (sem a água depois).
#[test]
#[ignore = "sonda: mede o custo de pintar a miniatura"]
fn sonda_custo_da_miniatura() {
    println!(
        "[custo] load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    for (rel, b) in ficheiros_de_fabrica() {
        let p = ler(&b).unwrap();
        let min = (0..5)
            .map(|_| {
                let t0 = std::time::Instant::now();
                let _ = miniatura_do_preset(&p);
                t0.elapsed().as_secs_f64() * 1e3
            })
            .fold(f64::INFINITY, f64::min);
        println!("[custo] {rel:<60} {min:7.1} ms");
    }
}

/// **Sonda do RUÍDO da largura** (W2): o mesmo pincel ao mesmo raio prometido, cinco traços que só
/// diferem na posição vertical (a tela cresce 60 px de cada vez; o traço desce 30) — o quanto a largura de um traço varia
/// por si. É o chão de qualquer promessa: nenhuma tabela a acerta melhor do que isto.
#[test]
#[ignore = "sonda: o ruído da largura por traço (W2)"]
fn sonda_ruido_da_largura() {
    for (rel, b) in ficheiros_do_wet() {
        let p = direito(&ler(&b).unwrap());
        for &r in &RAIOS_DA_PROMESSA {
            let ls: Vec<f32> = std::thread::scope(|s| {
                let h: Vec<_> = (0..5)
                    .map(|k| {
                        let p = &p;
                        s.spawn(move || {
                            let (px, w, h) =
                                traco_recto_alto(r, r + 10.0 * k as f32, QUADROS_DA_LEI, |t| {
                                    t.set_wet_relogio_fixo(true);
                                    assert!(t.wet_preset_aplicar(p).is_empty());
                                });
                            largura_que_se_ve(&px, w, h)
                        })
                    })
                    .collect();
                h.into_iter().map(|h| h.join().unwrap()).collect()
            });
            println!(
                "[ruido] {rel:<55} raio {r:>5} pedido {:>5} → {ls:?}",
                2.0 * r
            );
        }
    }
}

/// O pico de memória do processo (`VmHWM`), em MiB.
fn pico_mib() -> f64 {
    std::fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find_map(|l| l.strip_prefix("VmHWM:"))
        .and_then(|r| r.split_whitespace().next()?.parse::<f64>().ok())
        .unwrap_or(0.0)
        / 1024.0
}

/// **Sonda da MEMÓRIA da lei da largura** (J2 Dabber, L13): UM raio por processo (`VmHWM` é do processo), o traço
/// de [`largura_de`] com `PH2D_SONDA_PRESET` (Dabber), `PH2D_SONDA_RAIO`, `PH2D_SONDA_CRUA` (1), `PH2D_SONDA_JITTER`
/// (o do preset) e `PH2D_SONDA_QUADROS` (os da lei; `0` = só o gesto). Imprime o pico depois do gesto e depois da
/// água, as lanes e as janelas. O portão [`a_memoria_do_dabber_cabe`] corre-a num processo filho.
#[test]
#[ignore = "sonda: a memória da lei da largura, um raio por processo"]
fn sonda_memoria_da_lei() {
    use ph2d_editor_core::{Tool, tool::RasterEditTool};
    let var = |k: &str| std::env::var(k).ok();
    let nome = var("PH2D_SONDA_PRESET").unwrap_or_else(|| "Dabber".into());
    let raio: f32 = var("PH2D_SONDA_RAIO").map_or(300.0, |v| v.parse().unwrap());
    let crua = var("PH2D_SONDA_CRUA").as_deref() != Some("0");
    let quadros: u32 = var("PH2D_SONDA_QUADROS").map_or(QUADROS_DA_LEI, |v| v.parse().unwrap());
    let mut p = ficheiros_do_wet()
        .into_iter()
        .map(|(_, b)| ler(&b).unwrap())
        .find(|p| p.parametros.name == nome)
        .expect("o preset de fábrica");
    // `PH2D_SONDA_RECEITA=1`: o preset da RECEITA de hoje (antes de o regerar), com a tabela do ficheiro.
    if var("PH2D_SONDA_RECEITA").as_deref() == Some("1") {
        let r = receitas().find(|r| r.nome == nome).expect("a receita");
        let tabela = std::mem::take(&mut p.parametros.width_table);
        p.parametros = parametros(r);
        p.parametros.width_table = tabela;
    }
    if let Some(j) = var("PH2D_SONDA_JITTER") {
        p.parametros
            .brush
            .insert("jitter".into(), serde_json::json!(j.parse::<f64>().unwrap()));
    }
    let escala = if crua { (raio / 500.0).max(1.0) } else { 1.0 };
    let h = ((raio * 6.0) as u32 + 80).max(120);
    let w = super::sonda_gestos::W;
    let cy = h as f32 * 0.5;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&direito(&p)).is_empty());
    if crua {
        t.set_wet_largura_crua(escala);
    }
    t.set_brush_size_px(raio / escala);
    t.set_brush_color_srgb8(super::sonda_gestos::AZUL);
    let t0 = std::time::Instant::now();
    super::sonda_gestos::gesto(&mut t, &[[100.0, cy], [900.0, cy]], 3.0, |_| 1.0);
    let gesto = pico_mib();
    let mut janelas = Vec::new();
    t.wet_sonda_motor(|e| {
        janelas = e
            .lane_trails
            .iter()
            .map(ph2d_wet_paint::trail::Trail::window_half_for_measure)
            .collect();
    });
    for _ in 0..quadros {
        t.on_tick(1000.0 / 60.0);
    }
    let (px, pw, ph) = t.run_full();
    println!(
        "[memoria] {nome} raio {raio} crua {crua} · tela {w}×{h} · gesto {gesto:.0} MiB · água {:.0} MiB · lanes {} \
         · meias-larguras {janelas:?} · largura {:.1} · {:.1} s",
        pico_mib(),
        janelas.len(),
        largura_que_se_ve(&px, pw, ph),
        t0.elapsed().as_secs_f64()
    );
}

/// ⭐ O binário que corre AGORA, para o re-executar num filho. Nunca o CAMINHO (`current_exe`): o cargo religa o mesmo
/// `it-<hash>` (outra corrida, o `exame.sh` ao lado) gravando um ficheiro novo por cima, o inode deste processo fica
/// «(deleted)» e o filho dava `NotFound` — ou, pior, corria o código NOVO. `/proc/self/exe` abre o inode que corre.
fn o_proprio_binario() -> std::path::PathBuf {
    if cfg!(target_os = "linux") {
        "/proc/self/exe".into()
    } else {
        std::env::current_exe().unwrap()
    }
}

/// O pico do GESTO (MiB) e as lanes da [`sonda_memoria_da_lei`] num processo filho (o `VmHWM` é do processo).
fn memoria_num_filho(raio: f32) -> (f64, usize) {
    let out = std::process::Command::new(o_proprio_binario())
        .args([
            "presets_de_fabrica::sonda_memoria_da_lei",
            "--exact",
            "--ignored",
            "--nocapture",
            "--test-threads",
            "1",
        ])
        .env("PH2D_SONDA_RAIO", raio.to_string())
        .env("PH2D_SONDA_QUADROS", "0")
        .env_remove("PH2D_SONDA_JITTER")
        .env_remove("PH2D_SONDA_PRESET")
        .output()
        .expect("o processo filho da sonda");
    let txt = String::from_utf8_lossy(&out.stdout);
    let linha = txt
        .split("[memoria]")
        .nth(1)
        .unwrap_or_else(|| panic!("a sonda não respondeu a raio {raio}:\n{txt}"));
    let num = |k: &str| -> f64 {
        linha.split(k).nth(1).unwrap().split_whitespace().next().unwrap().parse().unwrap()
    };
    println!("[memoria]{}", linha.lines().next().unwrap_or(""));
    (num(" gesto "), num(" lanes ") as usize)
}

/// Os tectos do PICO do gesto do Dabber na lei crua da largura (a do gerador), em MiB, medidos 20/10 (L13, smoke,
/// um processo por raio; 03_MOTORES §4 L13):
///
/// | raio | limiar `r` (o mutante: 4 lanes) | uma ponta = uma lane | tecto |
/// |---|---|---|---|
/// | 300  | 298 | 165 | 230 |
/// | 1000 | 2 966 | 1 221 | 1 800 |
///
/// (O Dabber de antes — lâmina, Jitter 0,5 — com o limiar `r`: 467 a raio 300 e 3 955 a 1 100, 6–7 lanes.)
const TECTO_DA_MEMORIA_MIB: [(f32, f64); 2] = [(300.0, 230.0), (1000.0, 1800.0)];

/// ⭐ **A MEMÓRIA do Dabber cabe** (L13, EXAME §3.6): o Jitter espalha UMA ponta — o traço inteiro numa lane — e o
/// pico do gesto a raio 300 e 1000 fica abaixo do tecto medido. Mutação: o limiar da lane sem o Jitter
/// (`BrushSpec::limiar_da_lane` = `r`) parte o traço em lanes e passa os dois tectos.
#[test]
fn a_memoria_do_dabber_cabe() {
    for (raio, tecto) in TECTO_DA_MEMORIA_MIB {
        let (pico, lanes) = memoria_num_filho(raio);
        assert_eq!(lanes, 1, "raio {raio}: o Jitter partiu o traço em {lanes} lanes");
        assert!(pico <= tecto, "raio {raio}: pico do gesto {pico:.0} MiB > tecto {tecto:.0}");
    }
}

/// O que [`wet_com_copias`] mede: a tinta por metade (esquerda, direita), as lanes, os dabs e os que pousaram com
/// peso ZERO (a opacidade do dab no motor, `Dab::opacity`, já com o pigmento por comprimento).
struct Copias {
    tinta: [f64; 2],
    lanes: usize,
    dabs: usize,
    sem_pigmento: usize,
}

/// O Wet Round (o pincel que nasce sem preset: `pigmentSpacing` > 0, o pigmento por comprimento) com Jitter 0,5,
/// o Count e a simetria dados, um traço de `pts` a raio 16.
fn wet_com_copias(count: u32, simetria: bool, pts: &[[f32; 2]]) -> Copias {
    use ph2d_editor_core::{Tool, tool::RasterEditTool};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    let (w, h) = (super::sonda_gestos::W, super::sonda_gestos::H);
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_paint_media(PaintMedia::WetPaint);
    let mut p = t
        .wet_preset_capturar(&Parametros::novo(MEIO_DO_PRESET, "g", "copias"))
        .0;
    p.brush.insert("jitter".into(), serde_json::json!(0.5));
    p.brush.insert("spray_count".into(), serde_json::json!(count));
    let preset = Preset {
        parametros: p,
        origem: Origem::Fabrica,
        miniatura: ph2d_brush_presets::Miniatura { largura: 1, altura: 1, rgba: vec![255; 4] },
        texturas: Default::default(),
    };
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&preset).is_empty());
    if simetria {
        t.toggle_symmetry_enabled();
    }
    t.set_brush_size_px(16.0);
    t.set_brush_color_srgb8(super::sonda_gestos::AZUL);
    let (dabs, zero) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let (d2, z2) = (Arc::clone(&dabs), Arc::clone(&zero));
    t.wet_sonda_motor(|e| {
        e.on_dab = Some(Box::new(move |_b, d| {
            d2.fetch_add(1, Ordering::Relaxed);
            if d.opacity <= 1e-9 {
                z2.fetch_add(1, Ordering::Relaxed);
            }
        }));
    });
    super::sonda_gestos::gesto(&mut t, pts, 3.0, |_| 1.0);
    let mut lanes = 0;
    t.wet_sonda_motor(|e| lanes = e.lane_trails.len());
    for _ in 0..60 {
        t.on_tick(1000.0 / 60.0);
    }
    let (px, pw, _) = t.run_full();
    let mut tinta = [0.0; 2];
    for (i, c) in px.chunks_exact(4).enumerate() {
        tinta[usize::from(i as u32 % pw >= pw / 2)] += f64::from(255 - c[0]);
    }
    Copias { tinta, lanes, dabs: dabs.load(Ordering::Relaxed), sem_pigmento: zero.load(Ordering::Relaxed) }
}

/// ⭐ **Cada marca do spray é uma ponta** (L13): no Wet com Count 3 e Jitter, as três marcas de cada ponto
/// pousam pigmento — cada uma na sua lane. Com a lane só pela distância (`r + 2·alcance`) as três caíam
/// na mesma e a 2.ª e a 3.ª, sem caminho andado, pousavam com peso 0 (o mutante: 3 → 1 lane, 250 de 2 100 dabs sem
/// pigmento; a tinta do traço ×2,50 → ×2,23 da do Count 1, porque o rasto leva a tinta de uma marca à outra).
#[test]
fn cada_marca_do_spray_pousa_no_wet() {
    let pts = [[150.0, 320.0], [850.0, 320.0]];
    let (um, tres) = (wet_com_copias(1, false, &pts), wet_com_copias(3, false, &pts));
    let razao = (tres.tinta[0] + tres.tinta[1]) / (um.tinta[0] + um.tinta[1]);
    println!(
        "[copias] spray: Count 1 {} lane(s) · Count 3 {} lanes, {} de {} dabs sem pigmento · tinta ×{razao:.2}",
        um.lanes, tres.lanes, tres.sem_pigmento, tres.dabs
    );
    assert_eq!(um.lanes, 1, "Count 1 com Jitter: uma ponta, uma lane");
    assert_eq!(tres.lanes, 3, "Count 3: uma lane por marca");
    assert_eq!(tres.sem_pigmento, 0, "{} de {} marcas não pousaram pigmento", tres.sem_pigmento, tres.dabs);
    assert!(razao >= 2.0, "Count 3 pousa ×{razao:.2} a tinta do Count 1");
}

/// ⭐ **As duas cópias da simetria pintam as duas** (L13): o traço vertical a 12 px do eixo, com Jitter — as cópias
/// caem a < `r + 2·alcance` uma da outra e, só pela distância, partilhavam a lane: o rasto saltava entre elas e
/// dabs pousavam com peso 0. Mutação (a cópia ignorada): 1 lane, 71 de 790 dabs sem pigmento — sangra.
#[test]
fn as_copias_da_simetria_pintam_as_duas_no_wet() {
    let x = super::sonda_gestos::W as f32 / 2.0 + 12.0;
    let c = wet_com_copias(1, true, &[[x, 120.0], [x, 520.0]]);
    let r = c.tinta[0].min(c.tinta[1]) / c.tinta[0].max(c.tinta[1]);
    println!(
        "[copias] simetria: {} lanes, {} de {} dabs sem pigmento · tinta esquerda {:.0} · direita {:.0} (razão {r:.3})",
        c.lanes, c.sem_pigmento, c.dabs, c.tinta[0], c.tinta[1]
    );
    assert_eq!(c.lanes, 2, "a simetria: uma lane por cópia");
    assert_eq!(c.sem_pigmento, 0, "{} de {} dabs da simetria não pousaram pigmento", c.sem_pigmento, c.dabs);
    // As cópias sobrepõem-se a 12 px do eixo e a 2.ª pousa sobre a tinta molhada da 1.ª: a razão é física, não 1.
    assert!(r >= 0.85, "uma cópia da simetria pinta {r:.3} da outra");
}
