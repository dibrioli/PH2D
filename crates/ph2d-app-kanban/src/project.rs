//! **Os Kanbans no `.ph2dproj`** (v194, [ADR-0195]) — o campo `kanbans: Vec<u8>` do `ProjectFile` da
//! shell é o [`KanbanSet`] com versão PRÓPRIA. A shell só chama: o save pede [`to_bytes`], o load pede
//! [`from_bytes`] ANTES de mutar a sessão e instala com `Documents::replace_kanbans` depois de aceite.
//! (A leitura dos degraus anteriores — `193` e abaixo, sem o campo — é da shell: `project_load_read`.)
//!
//! [ADR-0195]: ../../../docs/architecture/decisions/0195-o-kanban-e-um-documento-do-projeto-o-quadro-referencia-o-e-a-aba-abre-o.md

use ph2d_editor_core::documents::Documents;
use ph2d_kanban_model::KanbanSet;

/// Os bytes que o save grava; sem documentos (sem hero) = sem Kanbans. `None` = não serializaram, e
/// o save é RECUSADO pela lei dos quadros: um ficheiro sem eles apagaria os do disco no próximo save.
#[must_use]
pub fn to_bytes(docs: Option<&Documents>) -> Option<Vec<u8>> {
    docs.map_or(Ok(Vec::new()), |d| d.kanbans().to_bytes())
        .inspect_err(|e| eprintln!("[proj] kanbans nao serializaram — save RECUSADO: {e}"))
        .ok()
}

/// Lê o campo. Bytes vazios = projecto sem Kanbans.
///
/// # Errors
/// Um blob que este binário não lê (versão mais nova ou corrompido): o load é RECUSADO, senão o
/// próximo Ctrl+S gravaria o projecto sem eles.
pub fn from_bytes(bytes: &[u8]) -> Result<KanbanSet, String> {
    KanbanSet::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **Os Kanbans atravessam o arquivo** — o conjunto que se gravou é o que se lê.
    #[test]
    fn os_kanbans_atravessam_o_arquivo() {
        let mut set = KanbanSet::default();
        set.create("Roadmap");
        set.create("Sprint");
        let mut docs = Documents::default();
        docs.replace_kanbans(set.clone());
        let bytes = to_bytes(Some(&docs)).expect("serializa");
        assert!(!bytes.is_empty());
        assert_eq!(from_bytes(&bytes), Ok(set));
    }

    /// ⭐ **Sem hero não há Kanbans, e vazio lê-se vazio** — o que um projecto antigo (sem o campo)
    /// e um save headless dão.
    #[test]
    fn sem_bytes_nao_ha_kanbans() {
        assert_eq!(to_bytes(None), Some(Vec::new()));
        assert_eq!(from_bytes(&[]), Ok(KanbanSet::default()));
        assert_eq!(
            to_bytes(Some(&Documents::default())).as_deref().map(from_bytes),
            Some(Ok(KanbanSet::default()))
        );
    }

    /// ⭐ **Um blob ilegível RECUSA** (o load mostra `shell.project_load.project_refused_its_kanbans`).
    #[test]
    fn um_blob_ilegivel_e_recusado() {
        assert!(from_bytes(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]).is_err());
        assert!(from_bytes(&[0x7F]).is_err());
    }
}
