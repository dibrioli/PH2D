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

mod memoria;
mod portoes;
pub(crate) mod regua_da_largura;
mod sondas;

use super::receitas_de_fabrica::{
    DIREITA, PRESETS_DE_FABRICA, Receita, ReceitaMeio, SPACING_DA_FABRICA, base_da_ferramenta,
    grao, modo_da_ferramenta, receitas, receitas_dos_meios,
};
use super::sonda_gestos::{
    Relogio, fnv1a, largura_da_nuvem, largura_que_se_ve, passa_o_tempo, traco_recto_alto,
    traco_recto_alto_com,
};
use ph2d_brush_presets::{
    Biblioteca, Origem, Parametros, Preset, caminho_relativo, escrever, gravar, ler,
};
use ph2d_tool_painter::{
    LINHAS_DA_MAO, MEIO_DO_PRESET, MOTOR_DOS_PRESETS, PaintMedia, PainterTool, WetKnobs,
    campos_do_meio, meio_do_preset, miniatura_do_preset, motor_do_meio, nome_da_ferramenta,
    nome_do_meio,
};
use ph2d_wet_paint::tuning::Knob;
use regua_da_largura::{
    imagem_da_ferramenta, largura_do_misturador, largura_do_rasto, sementes_da_largura,
    tolerancia_da_largura,
};
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

/// A caixa Real Oil no `engine` do preset (`wetpaint_preset::CHAVE_OLEO`, gravada só ligada). Que é ESTA a chave que o
/// motor lê prova-o `os_oleos_de_fabrica_vem_com_o_real_oil`, aplicando cada ficheiro.
const CHAVE_REAL_OIL: &str = "realOil";

fn raiz() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes")
}

/// O pincel de fábrica do meio inteiro, com os knobs da receita. Tamanho e opacidade NÃO são fixados
/// (a tabela não os tem): fica o do artista.
///
/// ⚠️ **Parte do ficheiro de fábrica que JÁ existe** (prompt 35): desde a W2 do plano 27 as cerdas vivem no
/// Shape (`WetKnobs::cerdas_no_shape`) e gravar não as põe no motor — capturar de fresco perdia a ponta de
/// cada preset. Só um preset sem ficheiro nasce da captura.
pub(super) fn parametros(r: &Receita) -> Parametros {
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
        // Um candidato SEM vizinho (o Oil Bristle, fora da fábrica desde o smoke do dono 27/10): o ficheiro dele.
        Err(_) if r.nasce_de.is_none() && super::midias_de_fabrica::candidato(r.nome).is_some() => {
            super::midias_de_fabrica::candidato(r.nome)
                .unwrap()
                .parametros
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
    // O dente do papel é da TELA (W3b): um ficheiro de antes dela ainda o levava, ignorado.
    p.brush.remove("paper_depth");
    if r.real_oil {
        p.engine.insert(CHAVE_REAL_OIL.to_owned(), 1.0);
    } else {
        p.engine.remove(CHAVE_REAL_OIL);
    }
    p.volumes.size_px = r.size_px;
    p.volumes.opacity = Some(r.opacity);
    p.volumes.water = Some(r.water);
    // Uma FERRAMENTA (J4 `mistura`): o modo é da receita, não do ficheiro.
    if let Some(modo) = modo_da_ferramenta(r.nome) {
        p.paint_mode = Some(modo.to_owned());
    }
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

/// O preset SEM a caixa Real Oil (J6, triagem dos portões): um portão que mede uma lei do PIGMENTO, do relevo do
/// Impasto/cerdas ou da textura usa o preset de fábrica só como instrumento — com os `Oil/*` de Real Oil (dono 28/10)
/// a luz do óleo somava-se à lei medida. A lei continua medida, sem isenção; o óleo mede-se no EXAME.
pub(super) fn sem_real_oil(mut p: Preset) -> Preset {
    p.parametros.engine.remove(CHAVE_REAL_OIL);
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
    assert!(
        m != PaintMedia::WetPaint,
        "{}: um preset do Wet Paint é uma `Receita`",
        r.nome
    );
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
            // Um preset de ferramenta nasce do slot DELA (o pincel e a mão de fábrica da Borracha, do Blur…).
            if let Some(f) = r.ferramenta {
                t.set_paint_tool_mode(&f.to_lowercase());
                assert_eq!(
                    t.ferramenta_dos_presets().and_then(nome_da_ferramenta),
                    Some(f),
                    "{}: a ferramenta {f} não se pega pelo trilho",
                    r.nome
                );
            }
            t.preset_capturar(&Parametros::novo(nome_do_meio(m), r.grupo, r.nome))
        }
    };
    assert!(
        r.ferramenta.is_none() || r.modo.is_none(),
        "{}: um preset de ferramenta não fixa o modo (a ferramenta É o modo)",
        r.nome
    );
    p.ferramenta = r.ferramenta.map(str::to_owned);
    if r.ferramenta.is_some() {
        p.paint_mode = None;
    }
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
        assert!(
            l.lida_por(m),
            "{}: o meio {m:?} não lê a linha da mão {chave}",
            r.nome
        );
        assert!(
            l.limpa(v) == v,
            "{}: {chave} = {v} fora da faixa {:?}",
            r.nome,
            l.faixa()
        );
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
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| json_perto(v, w)))
        }
        _ => a == b,
    }
}

/// ⭐ **O que a receita fixa É o que o pincel fica**: o preset aplicado num pincel novo arma o meio da receita, nada
/// falha, e cada campo, linha da mão, volume e modo lidos de volta são os da receita — um valor que o motor recusa
/// ou ignora (tipo errado, fora do domínio) acusa aqui, nunca vira um preset que mente.
pub(super) fn confere_receita_do_meio(r: &ReceitaMeio, p: &Preset) {
    let mut t = PainterTool::default();
    assert_eq!(
        t.preset_aplicar(p),
        Ok(Vec::new()),
        "{}: o preset não se aplica inteiro",
        r.nome
    );
    assert_eq!(
        t.paint_media(),
        r.meio,
        "{}: o preset arma outro meio",
        r.nome
    );
    assert_eq!(
        t.ferramenta_dos_presets().and_then(nome_da_ferramenta),
        r.ferramenta,
        "{}: o preset põe outra ferramenta na mão",
        r.nome
    );
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
        assert!(
            (w - d).abs() <= 1e-3,
            "{}: o tamanho pedia {d}, ficou {w}",
            r.nome
        );
    }
    if let Some(o) = r.opacity {
        let w = lido.volumes.opacity.unwrap_or(f64::NAN);
        assert!(
            (w - o).abs() <= 1e-4,
            "{}: a Strength pedia {o}, ficou {w}",
            r.nome
        );
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

/// O preset com a tela DIREITA: a largura é a do traço; o escorrimento do Wet Drip é da tela (W1e). E SEM o pó solto
/// (J4 `pastel_carvao`): os grãos fora da borda não são a largura do traço (com eles a tabela do Charcoal lia 1,5× e o
/// slider pintava o miolo mais fino que o prometido).
fn direito(p: &Preset) -> Preset {
    let mut d = p.clone();
    d.parametros.tilt = Some(DIREITA);
    if let Some(j) = d
        .parametros
        .brush
        .get_mut("dab_jitter")
        .and_then(|j| j.as_object_mut())
    {
        j.remove("po");
    }
    d
}

/// A largura que o pincel `p` deixa a `raio`: `crua` = o raio chega ao MOTOR sem a promessa.
pub(super) fn largura_de(p: &Preset, raio: f32, crua: bool) -> f32 {
    largura_de_com(p, raio, crua, Relogio::PorQuadro)
}

/// [`largura_de`] pelo relógio `r` da água: a mediana sobre as [`sementes_da_largura`] (as mesmas na tabela e na promessa).
fn largura_de_com(p: &Preset, raio: f32, crua: bool, r: Relogio) -> f32 {
    mediana(
        sementes_da_largura(p)
            .iter()
            .map(|&semente| {
                let (px, w, h, regua) = imagem_da_largura(p, raio, crua, r, semente);
                regua(&px, w, h)
            })
            .collect(),
    )
}

std::thread_local! {
    /// `true` = a largura medida COM o Real Oil do preset (o caminho de antes da J6; só a sonda
    /// `sonda_largura_sem_oleo` o liga, para comparar).
    static LEI_COM_OLEO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// A imagem que [`largura_de`] mede, e a régua com que a mede. ⛔ Com o Real Oil DESLIGADO (J6, REGRAS_DE_CUSTO
/// §METODOLOGIA 4): a luz não muda onde a tinta está, e com ela cada raio pagava o campo de altura a cada passo.
type Regua = fn(&[u8], u32, u32) -> f32;
fn imagem_da_largura(
    p: &Preset,
    raio: f32,
    crua: bool,
    r: Relogio,
    semente: u64,
) -> (Vec<u8>, u32, u32, Regua) {
    let sem;
    let p = if LEI_COM_OLEO.get() {
        p
    } else {
        sem = sem_real_oil(p.clone());
        &sem
    };
    if let Some(modo) = p.parametros.paint_mode.as_deref().filter(|m| *m != "Paint") {
        let (px, w, h) = imagem_da_ferramenta(p, modo, raio, crua, r, semente);
        let regua: Regua = if modo == "Smear" {
            largura_do_rasto
        } else {
            largura_do_misturador
        };
        return (px, w, h, regua);
    }
    let escala = if crua { (raio / 500.0).max(1.0) } else { 1.0 };
    let (px, w, h) = traco_recto_alto_com(raio / escala, raio, QUADROS_DA_LEI, r, |t| {
        t.set_wet_relogio_fixo(true);
        t.set_semente_dos_tracos(semente);
        assert!(t.wet_preset_aplicar(&direito(p)).is_empty());
        if crua {
            t.set_wet_largura_crua(escala);
        }
    });
    (
        px,
        w,
        h,
        if espalha(p) {
            largura_da_nuvem
        } else {
            largura_que_se_ve
        },
    )
}

/// O pincel ESPALHA marcas (o spray, `spray_count` > 1): a largura que se vê é a da nuvem
/// ([`super::sonda_gestos::largura_da_nuvem`]). Nenhum preset de fábrica do Wet espalha.
fn espalha(p: &Preset) -> bool {
    p.parametros
        .brush
        .get("spray_count")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|n| n > 1)
}

/// A mediana de `v` (`0` sem nenhuma).
fn mediana(mut v: Vec<f32>) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(f32::total_cmp);
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        0.5 * (v[n / 2 - 1] + v[n / 2])
    }
}

/// O raio do motor de `raio` pela tabela de largura de um preset — a conta de `LeiDaLargura::raio_do_motor`
/// (`wetpaint/largura.rs`: os pontos sem traço e os que não crescem saem; abaixo do 1.º e acima do último, a razão
/// da ponta). Tabela vazia = o raio pedido.
fn raio_do_motor_da_tabela(tabela: &[[f64; 2]], raio: f32) -> f32 {
    let mut pontos: Vec<(f64, f64)> = Vec::new();
    let mut sem_tinta_ate = 0.0f64;
    for &[r, l] in tabela {
        if !r.is_finite() || !l.is_finite() {
            continue;
        }
        if l <= 0.0 {
            if pontos.is_empty() {
                sem_tinta_ate = sem_tinta_ate.max(r);
            }
            continue;
        }
        if pontos.last().is_none_or(|&(r0, l0)| r > r0 && l > l0) {
            pontos.push((r, l));
        }
    }
    if pontos.len() < 2 {
        return raio;
    }
    let alvo = 2.0 * f64::from(raio);
    let (r0, l0) = pontos[0];
    if alvo <= l0 {
        let r = r0 * alvo / l0;
        return (if r < sem_tinta_ate { r0 } else { r }) as f32;
    }
    for w in pontos.windows(2) {
        let ((ra, la), (rb, lb)) = (w[0], w[1]);
        if alvo <= lb {
            return (ra + (rb - ra) * (alvo - la) / (lb - la)) as f32;
        }
    }
    let (rn, ln) = pontos[pontos.len() - 1];
    (rn * alvo / ln) as f32
}

/// A lei CRUA medida: raio do motor → largura, numa fila do MAIOR para o menor com `PH2D_LEI_FIOS` trabalhadores
/// (por omissão um por raio). ⚠️ A memória é a do raio: o Dabber (Jitter 0,5) a 64 de uma vez passava dos 48 GB; depois da
/// margem da janela pelo CAMINHO (`Trail::on_segment_no_caminho`) 21,9 GiB do tecto de 24 (prompt 53); com UMA lane por
/// ponta (L13) o raio 1 100 pesa 1,7 GiB (era 4,3) — mas a fatia é da LINHA e o `/tmp` (tmpfs) dos agentes conta nela
/// (14,6 G de `shmem` a 20/10): o Dabber regerou-se a `PH2D_LEI_FIOS=4`, pico 5,3 GiB, 4,8 min (03_MOTORES §4 L13).
pub(super) fn tabela_de_largura(p: &Preset) -> Vec<[f64; 2]> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let raios = raios_da_lei();
    // A largura mede-se sem o Real Oil ([`imagem_da_largura`]): a tela de um raio volta a ~940 MiB no Dabber a 1000
    // (com o óleo eram 1 513 e os 64 raios de uma vez passavam os 24G da fatia, J6).
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
        // O raio e a largura pelo menor texto que volta ao mesmo f32 (7.8, não 7.800000190734863): a largura equivalente
        // não é inteira, e o f64 de um f32 com 16 casas nem sempre volta igual do JSON (o gravar recusa-o).
        .map(|(&r, l)| {
            [
                format!("{r}").parse().unwrap(),
                format!("{l}").parse().unwrap(),
            ]
        })
        .collect()
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
/// a 52 px; 3,8 % a 240). A cura é um grão sem estrutura grande, não a tabela (05 §4b). A mesma classe vale para quem a
/// SEMENTE move além da tolerância fina — medido, [`regua_da_largura::a_semente_move_a_largura`] (a mediana de 3 sementes
/// ainda erra ±4 %: Smear Shattered 231,7 × 240 a raio 120, 622,4 × 600 a 300).
const TOLERANCIA_REL_GRAO: f32 = 0.10;

/// A borda que varia com ONDE o traço cai — e por isso a [`TOLERANCIA_REL_GRAO`]: o slot Grain armado. (As partículas
/// espalhadas na largura — `spray_count` > 1, `dab_jitter.across` > 0 — só o Metallic Splats as tinha no Wet; saiu da
/// fábrica no smoke do dono 27/10 e a cláusula com ele.)
fn borda_ruidosa(p: &Preset) -> bool {
    p.parametros
        .brush
        .get("texture")
        .and_then(|t| t.get("kind"))
        .is_some_and(|k| k != "None")
}

/// As larguras de `p` nos [`RAIOS_DA_PROMESSA`], com a promessa (`crua = false`) ou sem ela.
fn larguras_da_promessa(p: &Preset, crua: bool) -> Vec<f32> {
    std::thread::scope(|s| {
        let h: Vec<_> = RAIOS_DA_PROMESSA
            .iter()
            .map(|&r| s.spawn(move || largura_de_com(p, r, crua, Relogio::DeUmaVez)))
            .collect();
        h.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

/// `tecto`: o diâmetro máximo do preset (`size_max_px`, J3 decisão 8) — acima dele a promessa é o tecto.
fn fora_da_promessa(larguras: &[f32], rel: f32, tecto: f32) -> Vec<String> {
    RAIOS_DA_PROMESSA
        .iter()
        .zip(larguras)
        .filter(|(r, l)| {
            (*l - (2.0 * *r).min(tecto)).abs() > TOLERANCIA_PX.max(rel * (2.0 * *r).min(tecto))
        })
        .map(|(r, l)| format!("raio {r}: {l} px (pedido {}, tecto {tecto})", 2.0 * r))
        .collect()
}
