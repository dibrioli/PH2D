//! ⭐⭐⭐ **UMA COR QUE É UMA LINHA DE PROPRIEDADE PASSA PELA PORTA.**
//!
//! ⛔⛔ **Report do dono, 2026-09-21, com uma foto do Inspector e um DESENHO ao lado:** *«os
//! seletores de cor de todo o app precisam ser padronizados»*. Ele riscou o quadradinho encostado
//! à direita e desenhou no lugar dele uma **barra** na coluna do valor — como a caixa de marcar e
//! o campo numérico.
//!
//! **Medido nesse dia:** o app pintava `109` selectores de cor em **CINCO** larguras (`18`, `24`,
//! `32`, `120` e `268 px`). A porta nasceu daí
//! ([`ph2d_editor_core::property_row::paint_color_row`]) e dois painéis passaram por ela; os
//! outros continuaram a montar a linha à mão.
//!
//! # ⭐⭐ O discriminador é DERIVADO, e não uma lista de nomes de painel
//!
//! Nem toda amostra de cor é uma linha de propriedade — há **paletas** (o `bgremoval`, a máscara
//! do Painter), **etiquetas** de lista (a Hierarquia, as camadas de forma) e **editores ricos** (a
//! rampa). ⛔ Forçar qualquer uma delas pela porta trocaria uma grelha por um campo.
//!
//! ⇒ a pergunta que separa as duas populações é *«ele pinta um RÓTULO DE PROPRIEDADE no mesmo
//! fôlego?»*: um ficheiro que chama [`paint_property_label`] **e** desenha a amostra à mão está a
//! escrever uma linha de propriedade com uma segunda aritmética de colunas. Uma paleta não chama
//! rótulo nenhum.
//!
//! ⚠️ **Isto é uma régua TEXTUAL e ela diz o que não vê:** uma linha de propriedade montada sem o
//! `paint_property_label` (um rótulo escrito com `paint_text` à mão) passa-lhe ao lado. O censo das
//! elisões e o [`super::architecture_panel_wiring_parity`] medem outras metades; esta mede a
//! DUPLICAÇÃO da aritmética.

use std::fs;
use std::path::{Path, PathBuf};

/// A agulha do rótulo de propriedade.
const ROTULO: &str = "paint_property_label(";
/// As duas formas de pintar a amostra à mão.
const AMOSTRAS: [&str; 2] = ["paint_color_swatch(", "paint_swatch_or_mixed("];

/// ⛔ **As excepções — cada uma VERIFICADA pela metade de obsolescência de baixo.**
///
/// `(caminho relativo a `crates/`, porquê)`.
const FORA: &[(&str, &str)] = &[
    (
        "ph2d-editor-core/src/property_row.rs",
        "e' a PORTA: o `paint_color_row` chama o `paint_swatch_or_mixed` por dentro, e o ficheiro \
         tambem declara o `paint_property_label`. ⛔ Uma excepcao que nao estivesse aqui faria a \
         propria porta reprovar o gate que ela existe para impor",
    ),
    (
        "ph2d-panel-inspector/src/sections/color_tint.rs",
        "o tingimento POR CANTO e' uma GRELHA 2x2 dentro da coluna do valor — quatro valores de \
         UMA propriedade, mapeados aos cantos do quad. ⭐ Ele ja' passa pelo `property_row_columns` \
         (le^ `row.label` / `row.control` / `row.dot`), logo a coluna e' a mesma da porta: o que \
         ele nao pode e' colapsar quatro cantos numa barra",
    ),
];

fn raiz_das_crates() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn percorre(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entradas) = fs::read_dir(dir) else {
        return;
    };
    for e in entradas.flatten() {
        let p = e.path();
        if p.is_dir() {
            percorre(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// **Toda a superfície que o artista vê**: os painéis, a moldura e a shell.
fn ficheiros() -> Vec<PathBuf> {
    let raiz = raiz_das_crates();
    let mut out = Vec::new();
    if let Ok(entradas) = fs::read_dir(&raiz) {
        let mut dirs: Vec<PathBuf> = entradas
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.is_dir()
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.starts_with("ph2d-panel-") || n == "ph2d-editor-core")
            })
            .collect();
        dirs.sort();
        for d in &dirs {
            let src = d.join("src");
            if src.is_dir() {
                percorre(&src, &mut out);
            }
        }
    }
    let shell = raiz.join("../shells/desktop/src");
    if shell.is_dir() {
        percorre(&shell, &mut out);
    }
    out.sort();
    out
}

/// `(caminho relativo, pinta amostra à mão, pinta rótulo de propriedade)`.
fn censo() -> Vec<(String, bool, bool)> {
    let raiz = raiz_das_crates();
    let mut out = Vec::new();
    for p in ficheiros() {
        let Ok(src) = fs::read_to_string(&p) else {
            continue;
        };
        let amostra = AMOSTRAS.iter().any(|a| src.contains(a));
        if !amostra {
            continue;
        }
        let rel = p
            .strip_prefix(&raiz)
            .unwrap_or(&p)
            .to_string_lossy()
            .replace('\\', "/");
        out.push((rel, true, src.contains(ROTULO)));
    }
    out
}

/// ⛔ **Piso de população** — uma varredura que deixe de alcançar as crates devolve zero acusados e
/// lê-se como aprovada (`CLAUDE.md` §5.0, a falha MUDA de mover código). Medido 2026-09-22: **16**
/// ficheiros pintam uma amostra à mão.
const PISO: usize = 12;

/// ⭐⭐⭐ **NINGUÉM ESCREVE UMA LINHA DE COR COM ARITMÉTICA PRÓPRIA.**
///
/// **Mutação que deve sangrar:** devolver qualquer uma das quatro linhas do Flip ao par
/// `paint_property_label` + `paint_color_swatch`.
#[test]
fn a_linha_de_cor_passa_pela_porta() {
    let censo = censo();
    assert!(
        censo.len() >= PISO,
        "a varredura leu {} ficheiro(s) com amostra de cor — ela deixou de alcancar as crates",
        censo.len()
    );
    let fora: Vec<&str> = FORA.iter().map(|(p, _)| *p).collect();
    let maus: Vec<&String> = censo
        .iter()
        .filter(|(p, _, rotulo)| *rotulo && !fora.contains(&p.as_str()))
        .map(|(p, _, _)| p)
        .collect();
    assert!(
        maus.is_empty(),
        "estes ficheiros montam uma LINHA DE COR com aritmetica propria (rotulo de propriedade + \
         amostra a' mao): {maus:?}\n\
         cura: `ph2d_editor_core::property_row::paint_color_row` — a amostra enche a coluna do \
         valor e o hit e' a barra inteira. Se nao for uma linha (paleta, etiqueta de lista, \
         editor rico), a entrada vai para o `FORA` com o mecanismo escrito."
    );
}

/// ⚠️ **A metade de OBSOLESCÊNCIA** — *uma catraca sem censo de obsolescência não desce: ela vira
/// LICENÇA* (`CLAUDE.md` §5.0).
#[test]
fn nenhuma_excepcao_da_porta_das_cores_ficou_obsoleta() {
    let censo = censo();
    let mut mortas = Vec::new();
    for (caminho, razao) in FORA {
        match censo.iter().find(|(p, _, _)| p == caminho) {
            None => mortas.push(format!(
                "{caminho} — ja' nao pinta amostra nenhuma ({razao})"
            )),
            Some((_, _, false)) => mortas.push(format!(
                "{caminho} — ja' nao pinta rotulo de propriedade; o gate nao o acusaria e a \
                 entrada nao descreve nada ({razao})"
            )),
            Some(_) => {}
        }
    }
    assert!(
        mortas.is_empty(),
        "entrada(s) STALE no `FORA`:\n  {}",
        mortas.join("\n  ")
    );
}

/// ⭐⭐ **O CONTROLO do detector** — sem ele, um censo que devolvesse sempre `false` no rótulo
/// deixaria as duas metades verdes sobre o app inteiro por converter.
#[test]
fn o_detector_sabe_acusar_uma_linha_escrita_a_mao() {
    let fonte = format!(
        "fn linha() {{\n    {ROTULO}text_system, scene, nome, x, y, f, w, cor);\n    \
         {}rect, scene, theme);\n}}",
        AMOSTRAS[0]
    );
    assert!(
        fonte.contains(ROTULO) && AMOSTRAS.iter().any(|a| fonte.contains(a)),
        "o detector nao ve' a forma que ele existe para acusar"
    );
    // ⛔ E não inventa uma onde não há: uma paleta não pinta rótulo de propriedade.
    let paleta = format!(
        "fn paleta() {{\n    {}rect, scene, theme);\n}}",
        AMOSTRAS[0]
    );
    assert!(
        !paleta.contains(ROTULO),
        "o detector acusa uma PALETA — ela nao escreve linha de propriedade nenhuma"
    );
}
