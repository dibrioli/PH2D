//! **De onde vêm os bytes dos anexos.** O JSON do Trello só traz o URL de cada ficheiro carregado
//! (o download pede a sessão do trello.com); quem tem os ficheiros numa pasta dá-a aqui e eles
//! entram como ficheiros do projecto. Sem pasta, entram como links.

use std::path::{Path, PathBuf};

/// Uma fonte de bytes de anexos, pelo id do anexo no Trello e o nome do ficheiro.
pub trait FileSource {
    fn bytes(&self, attachment_id: &str, file_name: &str) -> Option<Vec<u8>>;
}

/// Uma pasta: procura `<pasta>/<id do anexo>/<nome>` e depois `<pasta>/<nome>`. Um nome com
/// separador de caminho ou `..` nunca sai da pasta (o JSON é de fora).
#[derive(Clone, Debug)]
pub struct DirFiles(pub PathBuf);

fn safe_component(s: &str) -> bool {
    !s.is_empty()
        && s != "."
        && s != ".."
        && !s.contains(['/', '\\', '\0'])
        && !Path::new(s).is_absolute()
}

impl FileSource for DirFiles {
    fn bytes(&self, attachment_id: &str, file_name: &str) -> Option<Vec<u8>> {
        if !safe_component(file_name) {
            return None;
        }
        let nested =
            safe_component(attachment_id).then(|| self.0.join(attachment_id).join(file_name));
        nested
            .into_iter()
            .chain(std::iter::once(self.0.join(file_name)))
            .find_map(|p| std::fs::read(p).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_name_never_leaves_the_folder() {
        for bad in ["", ".", "..", "../x", "a/b", "a\\b", "/etc/passwd"] {
            assert!(!safe_component(bad), "{bad:?}");
        }
        assert!(safe_component("cover image.png"));
    }
}
