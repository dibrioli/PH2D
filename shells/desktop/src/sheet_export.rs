//! **A EXPORTAÇÃO da folha** — `folha.png` + `folha.json` em disco.
//!
//! Plano [`docs/Sprite_projeto/17`] §7.3, a segunda saída do bake (W5.2). É ela que torna a
//! ferramenta **reversível**: o `.json` que sai daqui é o formato do Aseprite/TexturePacker, e o
//! par re-importa por [`crate::sheet_import`] — a mesma porta por que uma folha de fora entra.
//!
//! ⚠️ **O round-trip já tem gate, e ele é anterior a este módulo:** o teste
//! `pack → to_aseprite_json → parse_atlas_meta` do `ph2d-sprite-sheet` afirma que os retângulos
//! sobrevivem à ida e à volta, contra **o leitor que o import de facto usa**. Exportar para um
//! formato que só nós escrevemos e ninguém lê seria uma promessa sem consumidor.
//!
//! ## Onde os ficheiros caem, e por que não há diálogo
//!
//! Ao lado do arquivo de projeto (`PH2D_PROJECT_PATH`, que por omissão é o diretório de trabalho).
//! ⚠️ **Não é preguiça: o app não TEM diálogo de ficheiro** — o `io_menu` é um stub e o Ctrl+S
//! grava num caminho fixo (`CLAUDE.md` §5, "UI real de Save/Save As/Open" está em aberto). Inventar
//! um seletor só para esta ferramenta seria a segunda resposta a *"onde guardo isto?"*, e a que
//! ficaria diferente da do save no dia em que o diálogo real chegasse. O toast diz o caminho
//! completo, que é o que torna a ausência do diálogo suportável.

use std::path::PathBuf;

use ph2d_editor::{Toast, ToastQueue};
use ph2d_sprite_sheet::AuthoredSheet;

/// O nome do ficheiro a partir do nome da folha — só o que qualquer sistema de ficheiros aceita.
///
/// ⚠️ **O nome é do ARTISTA**, e ele escreve `Herói / v2`. Uma barra abriria um diretório que não
/// existe (o `save_buffer` falharia com um erro que não nomeia a causa), e os dois pontos são
/// ilegais no Windows — onde este projeto também compila. Mapear para `_` é reversível de ler e
/// não perde a ordem das palavras.
///
/// Vazio (ou só separadores) devolve `sheet`: um ficheiro chamado `.png` é invisível no Unix e
/// recusado no Windows.
pub(crate) fn safe_stem(name: &str) -> String {
    let cleaned: String = name
        .trim()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches(['_', '.']).to_string();
    if trimmed.is_empty() {
        "sheet".to_string()
    } else {
        trimmed
    }
}

/// O diretório em que os ficheiros caem — o do arquivo de projeto.
fn target_dir() -> PathBuf {
    let project =
        std::env::var("PH2D_PROJECT_PATH").unwrap_or_else(|_| "ph2d_project.postcard".into());
    PathBuf::from(project)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// **Grava `<nome>.png` + `<nome>.json`.** Devolve o caminho do PNG.
///
/// ⚠️ **O `.json` nomeia o `.png` pelo nome de FICHEIRO, não pelo caminho**: é assim que o formato
/// do Aseprite funciona, e é o que permite mover o par para outra pasta sem o partir. Um caminho
/// absoluto lá dentro tornaria a folha exportada intransportável — e o defeito só apareceria na
/// máquina de outra pessoa.
pub(crate) fn export(sheet: &AuthoredSheet, toasts: &mut ToastQueue) -> Option<PathBuf> {
    let stem = safe_stem(&sheet.name);
    let dir = target_dir();
    let png = dir.join(format!("{stem}.png"));
    let json = dir.join(format!("{stem}.json"));
    if let Err(e) = image::save_buffer(
        &png,
        &sheet.rgba,
        sheet.width,
        sheet.height,
        image::ColorType::Rgba8,
    ) {
        toasts.push(Toast::error(format!(
            "Export Sheet: could not write {}: {e}",
            png.display()
        )));
        return None;
    }
    let meta = ph2d_sprite_sheet::to_aseprite_json(sheet, &format!("{stem}.png"));
    if let Err(e) = std::fs::write(&json, meta) {
        // ⚠️ O PNG já está em disco, e dizê-lo importa: sem esta metade da frase o artista fica a
        // pensar que nada saiu, apaga a pasta e perde o que de facto tinha.
        toasts.push(Toast::error(format!(
            "Export Sheet: image written, but metadata failed ({}): {e}",
            json.display()
        )));
        return Some(png);
    }
    toasts.push(Toast::success(format!(
        "Sheet exported: {} + {stem}.json",
        png.display()
    )));
    Some(png)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_name_survives_intact() {
        assert_eq!(safe_stem("hero_idle"), "hero_idle");
        assert_eq!(safe_stem("sheet-01.v2"), "sheet-01.v2");
    }

    /// ⚠️ O caso que faz o `save_buffer` falhar com um erro que não nomeia a causa: uma barra abre
    /// um diretório que não existe.
    #[test]
    fn separators_and_illegal_chars_become_underscores() {
        assert_eq!(safe_stem("Her\u{f3}i / v2"), "Her_i___v2");
        assert_eq!(safe_stem("a:b"), "a_b");
        assert_eq!(safe_stem("..\\escape"), "escape");
    }

    /// Um ficheiro chamado `.png` é invisível no Unix e recusado no Windows.
    #[test]
    fn an_empty_or_punctuation_only_name_falls_back() {
        assert_eq!(safe_stem(""), "sheet");
        assert_eq!(safe_stem("   "), "sheet");
        assert_eq!(safe_stem("___"), "sheet");
        assert_eq!(safe_stem("..."), "sheet");
    }

    /// O nome de ficheiro que vai dentro do `.json` é o mesmo que o PNG recebeu — é o que mantém o
    /// par transportável.
    #[test]
    fn the_metadata_names_the_png_by_filename() {
        let sheet = AuthoredSheet::new(
            0,
            "My Sheet".into(),
            4,
            4,
            vec![0; 4 * 4 * 4],
            [("a".to_string(), [0, 0, 2, 2])],
        );
        let stem = safe_stem(&sheet.name);
        let json = ph2d_sprite_sheet::to_aseprite_json(&sheet, &format!("{stem}.png"));
        // ⚠️ O campo `image`, e não «o JSON não contém `/`»: a 1ª versão deste teste afirmava a
        // segunda e reprovou sobre código correto — o `meta.app` é a URL do repositório, e tem
        // duas barras. *Um assert mais largo que a afirmação apanha o inocente.*
        assert!(
            json.contains(r#""image": "My_Sheet.png""#),
            "o `.json` tem de nomear o PNG pelo nome de ficheiro que ele recebeu: {json}"
        );
    }
}
