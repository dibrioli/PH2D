//! ⭐ **Anexar FICHEIROS** — o que o diálogo escolheu ou o sistema largou: os bytes entram no
//! conjunto por conteúdo (`KanbanSet::put_file`, com o tamanho natural se é imagem) e o cartão ganha
//! um anexo `Link::File`. A 1.ª imagem de um cartão sem capa passa a capa (o Trello).

use super::write;
use crate::docs::KanbanDocs;
use ph2d_i18n::tr_with;
use ph2d_kanban_model::{CardId, FileId};
use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

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
    let files = files.into_iter().map(|(name, b)| {
        let size = image_size(&b);
        (name, Arc::from(b), size, None)
    });
    attach_ready(docs, id, card, files)
}

fn attach_ready(
    docs: &mut KanbanDocs,
    id: u64,
    card: CardId,
    files: impl IntoIterator<Item = (String, Arc<[u8]>, Option<[u32; 2]>, Option<FileId>)>,
) -> usize {
    let mut n = 0;
    for (name, bytes, size, key) in files {
        if bytes.is_empty() {
            continue;
        }
        // A chave calculada no fio entra feita; a do ficheiro lido na hora calcula-se aqui.
        let f = match key {
            Some(k) => docs.set.borrow_mut().put_file_keyed(k, bytes, size),
            None => docs.set.borrow_mut().put_file(bytes, size),
        };
        if write::add_file(docs, id, card, f, name, size.is_some()) {
            n += 1;
        }
    }
    n
}

/// Até aqui um ficheiro lê-se na hora; acima, num fio (ler + `image_size` custam ~0,2 ms/MB e o
/// sha-256 do `put_file` ~0,6 ms/MB: 8 MB ≈ meio quadro). Não é um tecto de tamanho — o tecto é a
/// RAM. Medição: `docs/TrelloClone/loop/MEDICOES.md` §W3 E.
const READ_NOW_UP_TO: u64 = 8 << 20;

/// Um ficheiro lido no fio: (aba, cartão, nome, bytes, tamanho natural e a chave sha-256, ou a falha).
type Loaded = (u64, CardId, String, Option<(Arc<[u8]>, Option<[u32; 2]>, FileId)>);

#[derive(Default)]
struct Loading {
    tx: Option<Sender<Loaded>>,
    rx: Option<Receiver<Loaded>>,
    waiting: usize,
}

thread_local! {
    static LOADING: RefCell<Loading> = RefCell::new(Loading::default());
}

/// Lê `p`, o tamanho natural (se é imagem) e a chave ([`FileId::of`]) no fio; chega por [`arrived`].
fn read_later(id: u64, card: CardId, name: String, p: PathBuf) {
    LOADING.with_borrow_mut(|l| {
        if l.tx.is_none() {
            let (tx, rx) = channel();
            (l.tx, l.rx) = (Some(tx), Some(rx));
        }
        let Some(tx) = l.tx.clone() else { return };
        l.waiting += 1;
        let _ = std::thread::Builder::new()
            .name("kanban-attach".into())
            .spawn(move || {
                let got = std::fs::read(&p).ok().map(|b| {
                    let size = image_size(&b);
                    let key = FileId::of(&b);
                    (Arc::from(b), size, key)
                });
                let _ = tx.send((id, card, name, got));
            });
    });
}

/// Ainda há ficheiros grandes a ler (quem desenha a pedido pede outro quadro).
#[must_use]
pub(crate) fn loading() -> bool {
    LOADING.with_borrow(|l| l.waiting > 0)
}

/// ⭐ Anexa o que o fio acabou de ler (chamado a cada quadro pela fila do verso); o que não se leu
/// fica num aviso. Devolve quantos entraram.
pub(crate) fn arrived(docs: &mut KanbanDocs) -> usize {
    let done: Vec<Loaded> = LOADING.with_borrow_mut(|l| {
        let got: Vec<Loaded> =
            l.rx.as_ref()
                .map(|rx| rx.try_iter().collect())
                .unwrap_or_default();
        l.waiting -= got.len().min(l.waiting);
        got
    });
    let mut n = 0;
    for (id, card, name, got) in done {
        match got {
            Some((bytes, size, key)) => {
                n += attach_ready(docs, id, card, [(name, bytes, size, Some(key))]);
            }
            None => docs
                .notices
                .push(tr_with("kanban.back.cannot_read", &[("t", &name)])),
        }
    }
    n
}

/// ⭐ A porta dos CAMINHOS (o diálogo, o largar do sistema): lê cada um e anexa-o — o pequeno já, o
/// grande no fio ([`READ_NOW_UP_TO`]); o que não se lê fica num aviso. Devolve quantos foram aceites.
pub(crate) fn attach_paths(
    docs: &mut KanbanDocs,
    id: u64,
    card: CardId,
    paths: &[PathBuf],
) -> usize {
    let mut files = Vec::new();
    let mut later = 0;
    for p in paths {
        let name = p.file_name().map_or_else(
            || p.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        let meta = std::fs::metadata(p).ok().filter(std::fs::Metadata::is_file);
        match meta.map(|m| m.len()) {
            Some(0) => {}
            Some(len) if len > READ_NOW_UP_TO => {
                read_later(id, card, name, p.clone());
                later += 1;
            }
            Some(_) => match std::fs::read(p) {
                Ok(b) => files.push((name, b)),
                Err(_) => docs
                    .notices
                    .push(tr_with("kanban.back.cannot_read", &[("t", &name)])),
            },
            None => docs
                .notices
                .push(tr_with("kanban.back.cannot_read", &[("t", &name)])),
        }
    }
    attach_bytes(docs, id, card, files) + later
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

    #[test]
    #[ignore = "sonda de medição"]
    fn sonda_tecto_do_anexo() {
        use std::time::Instant;
        let dir =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/sonda-anexo");
        std::fs::create_dir_all(&dir).unwrap();
        let sizes = [1usize, 10, 100, 500];
        for &mb in &sizes {
            let n = mb << 20;
            let mut x = 0x9e37_79b9_7f4a_7c15u64;
            let data: Vec<u8> = (0..n)
                .map(|_| {
                    x ^= x << 13;
                    x ^= x >> 7;
                    x ^= x << 17;
                    x as u8
                })
                .collect();
            std::fs::write(dir.join(format!("{mb}.bin")), &data).unwrap();
        }
        let mut best = std::collections::BTreeMap::<(usize, &str), f64>::new();
        for _round in 0..3 {
            for &mb in &sizes {
                let p = dir.join(format!("{mb}.bin"));
                let mut t = Instant::now();
                let mut lap = |k: &'static str, t: &mut Instant| {
                    let s = t.elapsed().as_secs_f64() * 1e3;
                    let e = best.entry((mb, k)).or_insert(f64::MAX);
                    *e = e.min(s);
                    *t = Instant::now();
                };
                let bytes = std::fs::read(&p).unwrap();
                lap("1 ler", &mut t);
                let size = super::image_size(&bytes);
                lap("2 image_size", &mut t);
                let mut set = ph2d_kanban_model::KanbanSet::default();
                let k = set.create("m");
                let f = set.put_file(bytes, size);
                lap("3 put_file(sha256)", &mut t);
                set.doc_mut(k).unwrap().header.background = ph2d_kanban_model::Background::Image(f);
                let blob = set.to_bytes().unwrap();
                lap("4 KanbanSet::to_bytes", &mut t);
                let mut file = Vec::with_capacity(blob.len() + 64);
                file.extend_from_slice(&blob);
                lap("5 ProjectFile copia", &mut t);
                let out = dir.join("proj.out");
                let mut w = std::fs::File::create(&out).unwrap();
                std::io::Write::write_all(&mut w, &file).unwrap();
                w.sync_all().unwrap();
                lap("6 gravar+fsync", &mut t);
                let back = std::fs::read(&out).unwrap();
                let _set = ph2d_kanban_model::KanbanSet::from_bytes(&back).unwrap();
                lap("7 abrir (ler+from_bytes)", &mut t);
            }
        }
        for ((mb, k), ms) in &best {
            println!("SONDA {mb} MB | {k} | {ms:.1} ms");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
