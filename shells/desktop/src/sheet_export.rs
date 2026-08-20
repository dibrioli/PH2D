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
//! ## O artista escolhe a pasta
//!
//! Enio, 2026-08-19: *"não sei onde foi parar a sprite sheet exportada. não pode ser assim. tem
//! que abrir um dialog para escolher a pasta onde salvar"*.
//!
//! ⚠️ **A 1ª versão gravava num caminho fixo, e a nota que a justificava era FALSA:** ela dizia
//! *"o app não TEM diálogo de ficheiro"*. Eu tinha medido o `io_menu` (que é mesmo um stub) e
//! generalizado dali, sem procurar — o `rfd` é dependência da shell desde o M14.4c e há **dez**
//! chamadores dele, incluindo um `save_file()` no export de malha 3D. *Uma ausência afirmada sem
//! grep é um palpite com cara de medição*, e o custo dela foi um ficheiro que o dono do produto
//! não encontrou.
//!
//! O diálogo abre no diretório do projeto (um default útil, não uma decisão) e o `.json` vai
//! **ao lado** do `.png`, com o mesmo nome — tem de ir: o formato do Aseprite refere a imagem pelo
//! nome de ficheiro nu, resolvido relativo ao próprio `.json`.

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

/// O diretório em que o diálogo ABRE — o do arquivo de projeto.
///
/// ⚠️ É um ponto de partida, não um destino: quem decide é o artista. Abrir na pasta do projeto
/// poupa-lhe a navegação no caso comum (a folha pertence ao projeto) sem lhe tirar a escolha.
fn start_dir() -> PathBuf {
    let project =
        std::env::var("PH2D_PROJECT_PATH").unwrap_or_else(|_| "ph2d_project.postcard".into());
    PathBuf::from(project)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Os dois caminhos e o nome que vai DENTRO do `.json`, a partir do que o artista escolheu.
///
/// ⚠️ **Força a extensão a `png`.** Se ele escrever `heroi` sai `heroi.png`; se escrever
/// `heroi.jpg` sai `heroi.png` na mesma — e o toast diz o caminho final, então nada fica
/// escondido. A alternativa era gravar bytes PNG num ficheiro `.jpg`, e o primeiro programa a
/// abri-lo diria que o ARQUIVO está corrompido, apontando para o lugar errado. (É a mesma lição
/// que o export de malha 3D regista, do outro lado: lá há vários formatos e a extensão CARREGA a
/// escolha, então ela não pode ser reescrita; aqui há um só e ela é só o rótulo.)
///
/// PURA de propósito: é a parte desta função que se pode testar sem abrir uma janela.
pub(crate) fn paths_for(picked: &std::path::Path) -> (PathBuf, PathBuf, String) {
    let png = picked.with_extension("png");
    let json = png.with_extension("json");
    let image_filename = png
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("sheet.png")
        .to_string();
    (png, json, image_filename)
}

/// **Grava `<nome>.png` + `<nome>.json`.** Devolve o caminho do PNG.
///
/// ⚠️ **O `.json` nomeia o `.png` pelo nome de FICHEIRO, não pelo caminho**: é assim que o formato
/// do Aseprite funciona, e é o que permite mover o par para outra pasta sem o partir. Um caminho
/// absoluto lá dentro tornaria a folha exportada intransportável — e o defeito só apareceria na
/// máquina de outra pessoa.
pub(crate) fn export(sheet: &AuthoredSheet, toasts: &mut ToastQueue) -> Option<PathBuf> {
    let stem = safe_stem(&sheet.name);
    // ⚠️ O nome da folha vai como sugestão, não como imposição — e passa pelo `safe_stem` na mesma:
    // um `/` no nome faria o diálogo abrir noutra pasta sem o artista perceber porquê.
    let picked = rfd::FileDialog::new()
        .add_filter("PNG", &["png"])
        .set_directory(start_dir())
        .set_file_name(format!("{stem}.png"))
        .save_file()?;
    // Cancelar é SILENCIOSO: o artista fechou a janela, e um toast a dizer "cancelado" é ruído
    // sobre uma decisão que ele já sabe que tomou.
    write_pair(sheet, &picked, toasts)
}

/// Grava o par no caminho escolhido. Separada do [`export`] porque **esta metade é testável** — a
/// outra abre uma janela do sistema, que nenhum teste pode responder.
pub(crate) fn write_pair(
    sheet: &AuthoredSheet,
    picked: &std::path::Path,
    toasts: &mut ToastQueue,
) -> Option<PathBuf> {
    let (png, json, image_filename) = paths_for(picked);
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
    let meta = ph2d_sprite_sheet::to_aseprite_json(sheet, &image_filename);
    if let Err(e) = std::fs::write(&json, meta) {
        // ⚠️ O PNG já está em disco, e dizê-lo importa: sem esta metade da frase o artista fica a
        // pensar que nada saiu, apaga a pasta e perde o que de facto tinha.
        toasts.push(Toast::error(format!(
            "Export Sheet: image written, but metadata failed ({}): {e}",
            json.display()
        )));
        return Some(png);
    }
    // ⚠️ O caminho COMPLETO, e não «exportado com sucesso»: foi por não o dizer que o Enio
    // perguntou *"não sei onde foi parar"*. Um toast que não diz onde não responde a pergunta que
    // o artista faz a seguir.
    toasts.push(Toast::success(format!(
        "Sheet exported: {} (+ .json)",
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

    /// **O `.json` é irmão do `.png`, com o mesmo nome** — tem de ser: o formato refere a imagem
    /// pelo nome nu, resolvido relativo ao próprio `.json`.
    #[test]
    fn the_json_lands_beside_the_png_with_the_same_stem() {
        let (png, json, image) = paths_for(std::path::Path::new("/tmp/art/hero.png"));
        assert_eq!(png, std::path::Path::new("/tmp/art/hero.png"));
        assert_eq!(json, std::path::Path::new("/tmp/art/hero.json"));
        assert_eq!(image, "hero.png");
    }

    /// ⚠️ **A extensão é FORÇADA a `png`.** Sem isto, escrever `hero.jpg` no diálogo produziria
    /// bytes PNG num ficheiro `.jpg`, e o primeiro programa a abri-lo diria que o ARQUIVO está
    /// corrompido — apontando para o lugar errado.
    #[test]
    fn a_wrong_or_missing_extension_becomes_png() {
        let (png, json, image) = paths_for(std::path::Path::new("/tmp/hero"));
        assert_eq!(png, std::path::Path::new("/tmp/hero.png"));
        assert_eq!(json, std::path::Path::new("/tmp/hero.json"));
        assert_eq!(image, "hero.png");
        let (png, _, image) = paths_for(std::path::Path::new("/tmp/hero.jpg"));
        assert_eq!(png, std::path::Path::new("/tmp/hero.png"));
        assert_eq!(image, "hero.png");
    }

    /// O nome que o artista escreveu no diálogo é o que vai no `.json` — **não** o nome da folha.
    /// Renomear no diálogo e o `.json` continuar a apontar para o nome antigo daria um par partido.
    #[test]
    fn the_metadata_names_the_file_the_artist_chose() {
        let sheet = AuthoredSheet::new(
            0,
            "My Sheet".into(),
            4,
            4,
            vec![0; 4 * 4 * 4],
            [("a".to_string(), [0, 0, 2, 2])],
        );
        let (_, _, image) = paths_for(std::path::Path::new("/tmp/renamed_by_hand.png"));
        let json = ph2d_sprite_sheet::to_aseprite_json(&sheet, &image);
        assert!(
            json.contains(r#""image": "renamed_by_hand.png""#),
            "o `.json` tem de nomear o ficheiro ESCOLHIDO, nao o nome da folha: {json}"
        );
    }

    /// O nome da folha ainda serve de **sugestão** no diálogo, e passa pelo `safe_stem`.
    #[test]
    fn the_sheet_name_is_only_the_suggested_filename() {
        let sheet = AuthoredSheet::new(
            0,
            "My Sheet".into(),
            4,
            4,
            vec![0; 4 * 4 * 4],
            [("a".to_string(), [0, 0, 2, 2])],
        );
        let stem = safe_stem(&sheet.name);
        assert_eq!(stem, "My_Sheet");
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
