//! A passagem guardada POR FICHEIRO: o quadro desenha o mesmo SVG a cada nível de zoom, e moldar o
//! texto custa por glifo (medido: ~3,5 µs/glifo em release, 20 ms para ~5 800 glifos — o dobro do
//! próprio desenho ao tamanho natural). A chave são os PRÓPRIOS bytes (comparados inteiros: sem hash,
//! nem colisão, nem semente do processo — HR-5); o tecto é de MEMÓRIA, o que o parser aceita para UM
//! SVG (`MAX_ARCHIVE_TEXT_BYTES`, original e reescrito somados), saindo primeiro o usado há mais tempo.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use ph2d_imageio::MAX_ARCHIVE_TEXT_BYTES;

use super::TextGaps;

/// O resultado da passagem: o SVG reescrito (`None` = fica como está) e as lacunas.
pub(crate) struct Outlined {
    pub svg: Option<Vec<u8>>,
    pub gaps: TextGaps,
}

#[derive(Default)]
struct Cache {
    entries: VecDeque<(Box<[u8]>, Arc<Outlined>)>,
    bytes: usize,
}

/// A passagem de `txt`, guardada; `make` só corre se ela ainda não estiver.
pub(crate) fn get_or(txt: &str, make: impl FnOnce() -> Outlined) -> Arc<Outlined> {
    static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();
    let key = txt.as_bytes();
    let cache = CACHE.get_or_init(Mutex::default);
    {
        let mut c = cache.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(i) = c.entries.iter().position(|(k, _)| **k == *key) {
            let e = c.entries.remove(i).expect("posição válida");
            let hit = e.1.clone();
            c.entries.push_back(e);
            return hit;
        }
    }
    let made = Arc::new(make());
    let mut c = cache.lock().unwrap_or_else(PoisonError::into_inner);
    c.bytes += cost(key, &made);
    c.entries.push_back((key.into(), made.clone()));
    while c.bytes as u64 > MAX_ARCHIVE_TEXT_BYTES && c.entries.len() > 1 {
        if let Some((k, old)) = c.entries.pop_front() {
            c.bytes -= cost(&k, &old);
        }
    }
    made
}

fn cost(key: &[u8], o: &Outlined) -> usize {
    key.len() + o.svg.as_ref().map_or(0, Vec::len)
}
