//! ⭐ **Anexar FICHEIROS** — o que o diálogo escolheu ou o sistema largou: os bytes entram no
//! conjunto por conteúdo (`KanbanSet::put_file`, com o tamanho natural se é imagem) e o cartão ganha
//! um anexo `Link::File`. A 1.ª imagem de um cartão sem capa passa a capa (o Trello).

use super::write;
use crate::docs::KanbanDocs;
use ph2d_i18n::tr_with;
use ph2d_kanban_model::CardId;
use std::path::PathBuf;

/// O tamanho natural de uma imagem: o do CABEÇALHO (sem descodificar os píxeis); sem cabeçalho que
/// o `image` leia (TIFF, PSD, ORA…), a porta da cena descodifica. `None` = não é imagem.
#[must_use]
pub(crate) fn image_size(bytes: &[u8]) -> Option<[u32; 2]> {
    let head = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()
        .and_then(|r| r.into_dimensions().ok());
    let (w, h) = head.or_else(|| {
        let a = ph2d_asset::decode_image(bytes).ok()?;
        a.image_rgba8().map(|(w, h, _)| (w, h))
    })?;
    (w > 0 && h > 0).then_some([w, h])
}

/// O tipo de um anexo pelo nome («PNG», «PDF»); sem extensão, `None`.
#[must_use]
pub(crate) fn kind_of(name: &str) -> Option<String> {
    let (_, ext) = name.rsplit_once('.')?;
    (!ext.is_empty() && ext.len() <= 8 && ext.chars().all(|c| c.is_ascii_alphanumeric()))
        .then(|| ext.to_ascii_uppercase())
}

/// «512 B», «34 KB», «2.4 MB» — o tamanho de um anexo como o Trello o escreve (base 1000).
#[must_use]
pub(crate) fn human_size(n: usize) -> String {
    #[allow(clippy::cast_precision_loss)]
    let f = n as f64;
    if n < 1000 {
        format!("{n} B")
    } else if f < 1e6 {
        format!("{:.0} KB", f / 1e3)
    } else if f < 1e9 {
        format!("{:.1} MB", f / 1e6)
    } else {
        format!("{:.1} GB", f / 1e9)
    }
}

/// ⭐ Os ficheiros `(nome, bytes)` anexados ao cartão `card` da aba `id`, um passo de desfazer cada.
/// Devolve quantos entraram (um ficheiro vazio não entra).
pub(crate) fn attach_bytes(
    docs: &mut KanbanDocs,
    id: u64,
    card: CardId,
    files: Vec<(String, Vec<u8>)>,
) -> usize {
    let mut n = 0;
    for (name, bytes) in files {
        if bytes.is_empty() {
            continue;
        }
        let size = image_size(&bytes);
        let f = docs.set.borrow_mut().put_file(bytes, size);
        if write::add_file(docs, id, card, f, name, size.is_some()) {
            n += 1;
        }
    }
    n
}

/// ⭐ A porta dos CAMINHOS (o diálogo, o largar do sistema): lê cada um e anexa-o; o que não se lê
/// fica num aviso. Devolve quantos entraram.
pub(crate) fn attach_paths(docs: &mut KanbanDocs, id: u64, card: CardId, paths: &[PathBuf]) -> usize {
    let mut files = Vec::new();
    for p in paths {
        let name = p.file_name().map_or_else(
            || p.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        match std::fs::read(p) {
            Ok(b) if p.is_file() => files.push((name, b)),
            _ => docs
                .notices
                .push(tr_with("kanban.back.cannot_read", &[("t", &name)])),
        }
    }
    attach_bytes(docs, id, card, files)
}

#[cfg(test)]
mod tests {
    use super::{human_size, kind_of};

    #[test]
    fn sizes_and_kinds_read_like_the_trello() {
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(34_200), "34 KB");
        assert_eq!(human_size(2_400_000), "2.4 MB");
        assert_eq!(kind_of("fox.sheet.png").as_deref(), Some("PNG"));
        assert_eq!(kind_of("README"), None);
    }
}
