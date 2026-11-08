//! **A entrada gravada de um documento, guardada entre cópias** (P12 S): a cópia de recuperação
//! reescreve só os documentos que mudaram.
//!
//! A chave é dupla. As células (`Cel.data`, mapas, tiles) só mudam por um `Op`, e todo `Op` que fica
//! passa pelo [`crate::History`], cuja [`crate::History::revision`] muda nele. O resto do documento
//! (próximo id, prefs, último export, user data, bandeiras de camada) pode mudar sem passo do
//! desfazer — compara-se pelos próprios bytes, que são pequenos (o registo sem células). Assim nada
//! que se grava escapa à chave, e a saída é a de [`super::write_entry`] byte a byte.
//!
//! Medido (P12_plano §4.3, `o_codec_reaproveita_os_documentos_que_nao_mudaram`, `--profile smoke`,
//! MESMO processo, intercalado, mínimo de 5; 16 sprites 512² × 8 quadros = 128 MiB, um editado):
//!
//! | antes (clona + `write_library`) | cache + junção (o Save) | só entradas (fio da janela da cópia) |
//! |---|---|---|
//! | 74–76 ms | 23–24 ms | 1,1 ms |

use std::sync::Arc;

use super::{sprite_record_with, write_entry, write_library_from_entries};
use crate::sprite::Sprite;

#[derive(Debug, Clone, Default)]
pub struct EntryCache {
    held: Option<Held>,
}

#[derive(Debug, Clone)]
struct Held {
    key: u64,
    revision: u64,
    rest: Vec<u8>,
    entry: Arc<Vec<u8>>,
}

impl EntryCache {
    /// Os bytes de [`write_entry`]`(key, sprite)`; `revision` é a do histórico DESTE sprite.
    pub fn entry(&mut self, key: u64, revision: u64, sprite: &Sprite) -> &[u8] {
        let rest = sprite_record_with(sprite, false);
        let fresh = self
            .held
            .as_ref()
            .is_some_and(|h| h.key == key && h.revision == revision && h.rest == rest);
        if !fresh {
            self.held = Some(Held {
                key,
                revision,
                rest,
                entry: Arc::new(write_entry(key, sprite)),
            });
        }
        self.bytes()
    }

    /// Os bytes da última [`Self::entry`] (vazio antes da primeira).
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.held.as_ref().map_or(&[], |h| h.entry.as_slice())
    }

    /// A entrada partilhada (conta uma referência, não copia): o que [`Entries`] leva a outro fio.
    #[must_use]
    pub fn shared(&self) -> Option<Arc<Vec<u8>>> {
        self.held.as_ref().map(|h| Arc::clone(&h.entry))
    }

    /// Larga os bytes guardados (a memória volta; a próxima cópia reescreve).
    pub fn clear(&mut self) {
        self.held = None;
    }
}

/// As entradas de uma biblioteca, em ordem de chave, partilhadas: juntá-las ([`Self::library`]) pode
/// correr noutro fio. Sem entradas = o blob vazio (projecto sem Pixel Sprites).
#[derive(Debug, Clone, Default)]
pub struct Entries(pub Vec<Arc<Vec<u8>>>);

impl Entries {
    /// O blob, byte a byte o de [`super::write_library`] sobre os mesmos documentos.
    #[must_use]
    pub fn library(&self) -> Vec<u8> {
        if self.0.is_empty() {
            return Vec::new();
        }
        write_library_from_entries(self.0.iter().map(|e| e.as_slice()))
    }
}
