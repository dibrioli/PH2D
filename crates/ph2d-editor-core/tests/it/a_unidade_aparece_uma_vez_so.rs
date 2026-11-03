//! ⭐⭐⭐ **A UNIDADE APARECE UMA VEZ SÓ POR LINHA** — no rótulo, como sufixo `(px)`/`(%)`/`(°)`, OU
//! no campo (`num_row_unit` / `NumberInput::suffix` / o `display` de um chip), **nunca nos dois**.
//!
//! Duas épocas convivem enquanto os painéis migram, e esta é a lei que as duas partilham:
//! - **2026-09-15** (spec `docs/UI_New_and_Simple/spec/03_a_linha_de_propriedade.md` §7): a unidade
//!   no CAMPO, colada ao número — com ela no rótulo, 20 de 39 rótulos do Inspector eram cortados.
//! - **2026-11-01, decisão do dono** (`project-memory/feedback_units_live_in_the_label_not_in_the_box.md`):
//!   no idioma Figma dos painéis a unidade sai da caixa e vai para o rótulo (`Length (px)`), para a
//!   caixa estreitar no iPad. A F48d escreveu `Distance (px)` assim, de propósito.
//!
//! ⛔ O que as duas proíbem: a unidade DUAS vezes (`Distance (px)` sobre `12 px`), e uma unidade no
//! rótulo fora do formato `(unidade)` no fim (`Min px`, `Regen / s`).
//!
//! # A régua
//!
//! 1. Varre as tabelas de i18n e acha os textos que acabam em `(unidade)`.
//! 2. Para cada um, acha no fonte (`crates/*/src`, `shells/*/src`) cada uso da CHAVE e lê a chamada
//!    que a recebe (a primeira chamada à volta que não é o `tr(`). Se essa chamada dá também uma
//!    unidade ao campo (`Unit::…`, `.suffix(Some`), a linha diz a unidade duas vezes.
//! 3. Um rótulo sem molde `{…}` com a unidade solta no fim ou `(unidade)` a meio está fora do formato.
//!
//! ⚠️ Limite declarado: a régua lê a chamada onde a chave ENTRA. Um rótulo guardado antes numa
//! variável e entregue a outra chamada não é visto — o piso de população abaixo prova que a régua
//! não está vazia, não que é completa.

use std::fs;
use std::path::{Path, PathBuf};

/// ⏳ **Rótulos com a unidade FORA do formato `(unidade)` — MEDIDOS em 2026-11-03, e só ENCOLHEM.**
///
/// ⛔ Não acrescente entrada: um rótulo novo com unidade escreve-a `Nome (px)` ou põe-na no campo.
const FORA_DO_FORMATO: &[(&str, &str)] = &[
    (
        "panel.bg_removal.mask.min_px",
        "«Min px» — unidade solta no fim",
    ),
    (
        "panel.painter_layers.wetpaint.vol_size",
        "«Size px» — unidade solta no fim",
    ),
    (
        "component.field.health_fields.5",
        "«Regen / s» — taxa escrita «/ s»",
    ),
    (
        "component.field.damage_fields.11",
        "«Lasting / s» — taxa escrita «/ s»",
    ),
    (
        "component.field.health_bar_fields.10",
        "«Trail Speed / s» — taxa escrita «/ s»",
    ),
    (
        "panel.inspector.vida.amount_per_second",
        "«Amount / s» — taxa escrita «/ s»",
    ),
];

/// O vocabulário de unidades que um rótulo pode escrever (ver [`unidade_no_fim`]).
const SUFIXOS: &[&str] = &[
    "m", "px", "s", "ms", "1/s", "N", "N.m", "m/s", "m/s2", "m/s^2", "%", "deg", "deg/s", "rad",
    "°", "°/s", "N.s", "seconds",
];

/// Um valor de i18n «tem unidade no fim» quando acaba em `(<sufixo>)`.
///
/// ⚠️ **A lista sai do vocabulário do produto** ([`ph2d_editor_core::widget::Unit`]) mais as formas
/// tipográficas que um rótulo usa e o campo não (`°`, `N.m` com espaço). ⛔ Derivá-la SÓ do enum
/// deixaria de fora exactamente os rótulos que ainda não têm unidade correspondente — que são os
/// que este gate existe para achar.
fn unidade_no_fim(texto: &str) -> Option<String> {
    let t = texto.trim_end();
    if !t.ends_with(')') {
        return None;
    }
    let abre = t.rfind('(')?;
    let dentro = &t[abre + 1..t.len() - 1];
    // ⛔⛔ **Um MOLDE entre parênteses é uma unidade, e foi ele que escapou.** Report do dono,
    //    2026-09-15, com foto: `"Speed (°/s)"` continuava no rótulo — e o TEXTO na tabela é
    //    `"Speed ({unit})"`, que não acaba em sufixo nenhum. ⇒ *um rótulo que CONSTRÓI a unidade em
    //    tempo de execução é o caso mais certo de todos, e era o único que o detector não via.*
    // ⚠️ **E é `{unit}`, não «um molde qualquer»** — a 1.ª redacção aceitava todo `{…}` e acusou
    //    quatro TÍTULOS de secção (`"Timers  ({n})"`), onde o molde é uma CONTAGEM. *O que diz que
    //    aquilo é uma unidade é o NOME do molde, e ele está escrito ali.*
    if dentro == "{unit}" {
        return Some(dentro.to_string());
    }
    let normal = dentro.replace("\\u{00b0}", "°");
    // ⛔⛔ **QUATRO formas CEGAS, achadas em 2026-09-15 — e a mais cara era a mais óbvia.**
    //    O censo dizia-se fechado e deixou passar `"Init Spin (deg/s)"` **ao lado** do irmão
    //    `"Init Vel X"`, já curado: a lista tinha `°/s` e **não** `deg/s`, que é a forma que o
    //    `Unit::suffix` de facto emite. As outras três: `1/s` (o *Damping* da câmera, que é uma
    //    TAXA), `ms` (o relógio de uma animação de sprite) e `seconds` **por extenso** (o
    //    *Duration* de um temporizador). *Uma dívida grande esconde os erros do instrumento; foi
    //    ao ela encolher que os quatro ficaram à vista.*
    SUFIXOS
        .iter()
        .any(|s| normal.eq_ignore_ascii_case(s))
        .then_some(normal)
}

fn tabelas() -> Vec<(String, String)> {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("crates/ph2d-i18n/src");
    let mut out = Vec::new();
    let Ok(entradas) = fs::read_dir(&raiz) else {
        panic!("a varredura nao achou {raiz:?}");
    };
    for e in entradas.flatten() {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let Ok(txt) = fs::read_to_string(&p) else {
            continue;
        };
        for linha in txt.lines() {
            let Some(seta) = linha.find("\" => \"") else {
                continue;
            };
            let Some(abre) = linha[..seta].rfind('"') else {
                continue;
            };
            let chave = &linha[abre + 1..seta];
            let resto = &linha[seta + 6..];
            let Some(fim) = resto.find('"') else { continue };
            out.push((chave.to_string(), resto[..fim].to_string()));
        }
    }
    out
}

/// O texto tem a unidade fora do formato `(unidade)` no fim? Só para RÓTULOS: um texto com molde
/// (`{px} px`, `Sees {name}, at {dist} m`) é uma leitura com o valor dentro, não um rótulo.
fn fora_do_formato(texto: &str) -> Option<String> {
    if texto.contains('{') {
        return None;
    }
    let t = texto.replace("\\u{00b0}", "°");
    let t = t.trim_end();
    let palavras: Vec<&str> = t.split_whitespace().collect();
    if palavras.len() > 1 {
        let ultima = palavras[palavras.len() - 1];
        // ⚠️ À letra, sem ignorar a caixa: «Minimap — press M» é uma TECLA, não metros (medido).
        if SUFIXOS.contains(&ultima) {
            return Some(ultima.to_string());
        }
    }
    // `(unidade)` a meio. ⚠️ Uma letra só a meio (`(s)`, `(N)`) é um plural ou um atalho de tecla
    // («piece(s)», «Sticky note (N)»), medido em 2026-11-03 — não uma unidade.
    let mut resto = t;
    while let Some(a) = resto.find('(') {
        let Some(f) = resto[a..].find(')') else { break };
        let dentro = &resto[a + 1..a + f];
        let no_fim = resto[a + f + 1..].trim().is_empty();
        if !no_fim
            && dentro.chars().count() > 1
            && SUFIXOS.iter().any(|s| dentro.eq_ignore_ascii_case(s))
        {
            return Some(dentro.to_string());
        }
        resto = &resto[a + f + 1..];
    }
    None
}

/// Os fontes de produto: `crates/*/src` e `shells/*/src`, sem as tabelas de i18n.
fn fontes() -> Vec<(PathBuf, String)> {
    fn anda(p: &Path, out: &mut Vec<(PathBuf, String)>) {
        let Ok(es) = fs::read_dir(p) else { return };
        for e in es.flatten() {
            let q = e.path();
            if q.is_dir() {
                anda(&q, out);
            } else if q.extension().and_then(|s| s.to_str()) == Some("rs")
                && let Ok(s) = fs::read_to_string(&q)
            {
                out.push((q, s));
            }
        }
    }
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let mut out = Vec::new();
    for topo in ["crates", "shells"] {
        let Ok(es) = fs::read_dir(raiz.join(topo)) else {
            panic!("a varredura nao achou {topo}/");
        };
        for e in es.flatten() {
            if e.file_name() == "ph2d-i18n" {
                continue;
            }
            anda(&e.path().join("src"), &mut out);
        }
    }
    out
}

/// A chamada que RECEBE o texto em `pos`: a primeira `(` à volta que não é a do `tr(`, até à `)`
/// que a fecha. `None` se o uso não está dentro de chamada nenhuma no mesmo enunciado.
fn janela(src: &str, pos: usize) -> Option<&str> {
    let b = src.as_bytes();
    let mut prof = 0usize;
    let mut i = pos;
    let inicio = loop {
        if i == 0 {
            return None;
        }
        i -= 1;
        match b[i] {
            b')' | b']' => prof += 1,
            b'[' if prof > 0 => prof -= 1,
            b'(' if prof > 0 => prof -= 1,
            b'(' => {
                let id: String = src[..i]
                    .chars()
                    .rev()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if id != "rt" {
                    break i;
                }
            }
            b';' | b'{' | b'}' if prof == 0 => return None,
            _ => {}
        }
    };
    let mut prof = 0usize;
    for (j, &c) in b.iter().enumerate().skip(inicio) {
        match c {
            b'(' => prof += 1,
            b')' => {
                prof -= 1;
                if prof == 0 {
                    // Com o NOME da chamada à frente (`ToolRailEntry::number(`), para a mensagem.
                    let nome = src[..inicio]
                        .chars()
                        .rev()
                        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == ':')
                        .map(char::len_utf8)
                        .sum::<usize>();
                    return Some(&src[inicio - nome..=j]);
                }
            }
            _ => {}
        }
    }
    None
}

/// A chamada dá também uma unidade ao CAMPO?
fn campo_leva_unidade(chamada: &str) -> bool {
    ["Unit::", ".suffix(Some"]
        .iter()
        .any(|m| chamada.contains(m))
}

/// Cada uso da chave `chave` no fonte, com a chamada que o recebe (as sem chamada ficam de fora).
fn usos<'a>(fontes: &'a [(PathBuf, String)], chave: &str) -> Vec<(&'a Path, &'a str)> {
    let agulha = format!("\"{chave}\"");
    let mut out = Vec::new();
    for (p, s) in fontes {
        for (i, _) in s.match_indices(&agulha) {
            if let Some(j) = janela(s, i) {
                out.push((p.as_path(), j));
            }
        }
    }
    out
}

#[test]
fn a_unidade_aparece_uma_vez_so() {
    let pares = tabelas();
    // ⚠️ **Piso de população, MEDIDO** — uma varredura partida devolve zero e lê-se como aprovada.
    //    As tabelas tinham **1 801** textos em 2026-09-15 (8 153 em 2026-11-03); o piso fica em
    //    `1 500` para acusar um parser que deixou de casar.
    assert!(
        pares.len() >= 1500,
        "a varredura leu {} textos de i18n — ela deixou de alcancar as tabelas",
        pares.len()
    );
    let src = fontes();
    let mut duas_vezes = Vec::new();
    let mut vistos = 0usize;
    for (chave, texto) in &pares {
        let Some(u) = unidade_no_fim(texto) else {
            continue;
        };
        let us = usos(&src, chave);
        if !us.is_empty() {
            vistos += 1;
        }
        for (p, chamada) in us {
            if campo_leva_unidade(chamada) {
                duas_vezes.push(format!(
                    "{chave}  =>  {texto:?}  (unidade `{u}` no rotulo E no campo, {})",
                    p.display()
                ));
            }
        }
    }
    // ⚠️ Piso da 2.ª metade, MEDIDO em 2026-11-03: dos 7 textos com `(unidade)` no fim, a régua leu
    //    a chamada de 5 (o `Distance (px)`, o `Grid Size (px)` e três avisos com plural `(s)`; os
    //    dois da timeline entram por outra porta).
    assert!(
        vistos >= 5,
        "a regua nao leu a chamada de nenhum rotulo com unidade — a janela partiu-se"
    );
    let tolerado: Vec<&str> = FORA_DO_FORMATO.iter().map(|(k, _)| *k).collect();
    let mut fora = Vec::new();
    for (chave, texto) in &pares {
        if tolerado.contains(&chave.as_str()) {
            continue;
        }
        if let Some(u) = fora_do_formato(texto) {
            fora.push(format!("{chave}  =>  {texto:?}  (unidade `{u}`)"));
        }
    }
    duas_vezes.sort();
    duas_vezes.dedup();
    fora.sort();
    assert!(
        duas_vezes.is_empty() && fora.is_empty(),
        "a unidade tem de aparecer UMA vez so por linha:\n\
         no rotulo E no campo ({}):\n  {}\n\
         fora do formato `Nome (unidade)` ({}):\n  {}\n\n\
         Escolha UM sitio: o rotulo `Nome (px)` (decisao do dono 01/11, \
         project-memory/feedback_units_live_in_the_label_not_in_the_box.md) ou o campo \
         (`num_row_unit` / `NumberInput::suffix`, spec 03 §7).",
        duas_vezes.len(),
        duas_vezes.join("\n  "),
        fora.len(),
        fora.join("\n  ")
    );
}

/// ⭐⭐ **Os dois CONTROLOS da lei** — sem eles, uma `janela` que nunca achasse a chamada deixaria a
/// 1.ª metade verde sobre o app inteiro.
#[test]
fn a_regua_reprova_a_unidade_dupla_e_aprova_a_do_rotulo() {
    // ⛔ Reprova: `X (px)` num chip cujo campo também diz `px`.
    let dupla = r#"v.push(ToolRailEntry::number(id, tr("k.x"), tr("k.x"), Some(format!("{} {}", n, Unit::Px.suffix()))));"#;
    let j = janela(dupla, dupla.find("\"k.x\"").unwrap()).expect("a janela nao achou a chamada");
    assert!(
        j.starts_with("ToolRailEntry::number(id,"),
        "a janela parou no tr( em vez da chamada: {j:?}"
    );
    assert!(unidade_no_fim("X (px)").is_some() && campo_leva_unidade(j));
    // ⛔ E pela porta do Inspector.
    let inspector = "yy = num_row_unit(scene, x, w, y, tr(\"k.y\"), id, Some(Unit::Px), col);";
    let j = janela(inspector, inspector.find("\"k.y\"").unwrap()).unwrap();
    assert!(
        campo_leva_unidade(j),
        "a janela do num_row_unit nao ve o Unit: {j:?}"
    );
    // ✓ Passa: o `Distance (px)` REAL (F48d) — rótulo com a unidade, campo sem sufixo.
    let src = fontes();
    let us = usos(&src, "panel.vector.expand.offset");
    assert!(
        us.iter().any(|(_, j)| j.contains("ToolRailEntry::number")),
        "a regua nao le a fila do Offset Path: {us:?}"
    );
    assert!(us.iter().all(|(_, j)| !campo_leva_unidade(j)));
    let distance = tabelas()
        .into_iter()
        .find(|(k, _)| k == "panel.vector.expand.offset")
        .map(|(_, t)| t)
        .expect("a chave do Distance sumiu");
    assert!(
        unidade_no_fim(&distance).is_some(),
        "o controlo positivo pede o rotulo com a unidade: {distance:?}"
    );
    // E o formato: `Nome (px)` passa, a unidade solta e a meio reprovam; plural e molde não são.
    assert!(fora_do_formato("Distance (px)").is_none());
    assert_eq!(fora_do_formato("Min px").as_deref(), Some("px"));
    assert_eq!(fora_do_formato("Regen / s").as_deref(), Some("s"));
    assert_eq!(fora_do_formato("Size (px) max").as_deref(), Some("px"));
    assert!(fora_do_formato("{k} override(s) on this piece").is_none());
    assert!(fora_do_formato("Removed piece(s) from this copy").is_none());
    assert!(fora_do_formato("{px} px").is_none());
}

/// ⚠️ **A metade de OBSOLESCÊNCIA** — uma tolerância que já não descreve nada sai da lista.
#[test]
fn a_divida_fora_do_formato_ainda_descreve_algo() {
    let pares = tabelas();
    let mut mortas = Vec::new();
    for (chave, razao) in FORA_DO_FORMATO {
        match pares.iter().find(|(k, _)| k == chave) {
            None => mortas.push(format!("{chave} — a chave sumiu ({razao})")),
            Some((_, texto)) if fora_do_formato(texto).is_none() => {
                mortas.push(format!(
                    "{chave} — ja' esta no formato: {texto:?} ({razao})"
                ));
            }
            _ => {}
        }
    }
    assert!(
        mortas.is_empty(),
        "entrada(s) STALE na divida:\n  {}",
        mortas.join("\n  ")
    );
}

/// ⭐⭐ **O CONTROLO do detector** — sem ele, um `unidade_no_fim` que respondesse sempre `None`
/// deixaria as duas metades verdes sobre o app inteiro por converter.
#[test]
fn the_detector_can_see_a_unit_in_a_label() {
    for (texto, esperado) in [
        ("Break Torque (N.m)", Some("N.m")),
        ("Radius (m)", Some("m")),
        ("Motor (\\u{00b0}/s)", Some("°/s")),
        ("Init Vel X (m/s)", Some("m/s")),
        ("Grid Size (px)", Some("px")),
        // ⛔ Os quatro buracos de 2026-09-15.
        ("Init Spin (deg/s)", Some("deg/s")),
        ("Damping (1/s)", Some("1/s")),
        ("Frame ms (0 = use)", None),
        ("Hold (ms)", Some("ms")),
        ("Duration (seconds)", Some("seconds")),
        // ⛔ O MOLDE — o caso que o report de 2026-09-15 expôs.
        ("Speed ({unit})", Some("{unit}")),
    ] {
        assert_eq!(
            unidade_no_fim(texto).as_deref(),
            esperado,
            "o detector nao ve a unidade em {texto:?}"
        );
    }
    // ⛔ E uma CONTAGEM num título não é uma unidade — o molde chama-se `{n}`.
    for texto in [
        "Float Height",
        "Corner Rays",
        "Weight on Ground",
        "Damping",
        "Timers  ({n})",
    ] {
        assert!(
            unidade_no_fim(texto).is_none(),
            "o detector inventa uma unidade em {texto:?}"
        );
    }
}
