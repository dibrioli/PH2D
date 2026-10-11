//! ⭐ **Os ficheiros do Quadro** (MiroClone W8, `docs/MiroClone/06_equipa_e_saida.md` §2 E, X, K10): o
//! que a interface pede (`Documents::take_io`) servido aqui, UMA vez por quadro, pelo diálogo da casa
//! (`ph2d_app_host::modal`) — exportar PNG/SVG/PDF/`.excalidraw`, abrir um `.excalidraw` (ou um
//! `.png`/`.svg` com a cena embebida) num quadro novo — e o nome de quem escreve, lido e gravado em
//! `~/.ph2d/board_author.txt` (o idioma das `prefs`: é da MÁQUINA, não do projecto).
//!
//! A shell só chama [`service`] (composição).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use ph2d_board_export::{
    Page, board_page, frame_page, paint_page, pdf, pdf_pages, png, selection_page, svg,
};
use ph2d_board_model::{Board, ElementId};
use ph2d_board_render::EXPORT_PAPER;
use ph2d_board_route::RouteCache;
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::documents::team::{ExportChoice, Format, IoRequest, Scope};
use ph2d_editor_core::screens::hero::document_tabs;
use ph2d_gpu::GpuContext;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;

/// O ficheiro do nome de quem escreve (`None` sem `$HOME`: a persistência salta-se).
fn author_file() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".ph2d").join("board_author.txt"))
}

static AUTHOR_READ: AtomicBool = AtomicBool::new(false);

/// K10 — o nome: lido UMA vez (o ficheiro, senão a conta do sistema), gravado quando muda.
fn author(hero: &mut HeroScreen) {
    if !AUTHOR_READ.swap(true, Ordering::Relaxed) && hero.documents.author().is_empty() {
        let saved = author_file()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|s| s.lines().next().unwrap_or("").trim().to_owned())
            .filter(|s| !s.is_empty());
        let account = || {
            std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .ok()
                .filter(|s| !s.trim().is_empty())
        };
        if let Some(name) = saved.or_else(account) {
            hero.documents.set_author(name);
        }
    }
    if let Some(name) = hero.documents.take_author_change()
        && let Some(p) = author_file()
    {
        let _ = p.parent().map(std::fs::create_dir_all);
        let _ = std::fs::write(p, format!("{name}\n"));
    }
}

/// ⭐ Uma vez por quadro (a shell chama-o): o nome de quem escreve e o pedido por servir.
pub fn service(hero: &mut HeroScreen, gpu: Option<&GpuContext>, ts: &mut TextSystem) {
    author(hero);
    crate::live::service(hero);
    let Some(req) = hero.documents.take_io() else {
        return;
    };
    match req {
        IoRequest::Export {
            board,
            choice,
            selection,
        } => {
            let Some(b) = hero.documents.boards().get(board).cloned() else {
                return;
            };
            let (ext, key) = extension(choice.format);
            let dialog = rfd::FileDialog::new()
                .add_filter(tr(key), &[ext])
                .set_file_name(format!("{}.{ext}", file_stem(&b.name)));
            let Some(path) = ph2d_app_host::modal::save_file(dialog) else {
                return;
            };
            // ADR-0182 — o quadro da sessão ao vivo sai como eu o vejo (as notas que não leio, sem o
            // texto); outro quadro não tem nada escondido de mim.
            let lv = hero.documents.live_view();
            let unread = if lv.board == Some(board) {
                lv.fac.hidden_to_me.clone()
            } else {
                BTreeSet::new()
            };
            let text = match write_export(&b, choice, &selection, gpu, ts, &unread, &path) {
                Ok(files) => {
                    let names: Vec<String> = files.iter().map(|f| display(f)).collect();
                    tr_with("board.io.exported", &[("file", &names.join(", "))])
                }
                Err(e) => tr_with("board.io.export_failed", &[("error", &e)]),
            };
            hero.documents.tell(text);
        }
        IoRequest::Import => {
            // O 1.º filtro é o que o diálogo mostra: tudo o que se abre (um `.jpg` à vista logo).
            let all: Vec<&str> = ["excalidraw", "json", "png", "svg"]
                .into_iter()
                .chain(ph2d_asset::SUPPORTED_IMAGE_EXTENSIONS.iter().copied())
                .collect();
            let dialog = rfd::FileDialog::new()
                .add_filter(tr("board.io.filter_all"), &all)
                .add_filter(
                    tr("board.io.filter_excalidraw"),
                    &["excalidraw", "json", "png", "svg"],
                )
                .add_filter(
                    tr("board.io.filter_images"),
                    ph2d_asset::SUPPORTED_IMAGE_EXTENSIONS,
                );
            let Some(path) = ph2d_app_host::modal::pick_file(dialog) else {
                return;
            };
            let text = match import_file(&path) {
                Ok((name, imp)) => {
                    let n = imp.doc.live_len();
                    let skipped = skipped_list(&imp.skipped);
                    let id = hero
                        .documents
                        .import_board(name.clone(), imp.doc, imp.votes, imp.files);
                    document_tabs::register_board(&mut hero.store, id);
                    let mut t = tr_with(
                        "board.io.imported",
                        &[("name", &name), ("n", &n.to_string())],
                    );
                    if !skipped.is_empty() {
                        t.push_str(". ");
                        t.push_str(&tr_with("board.io.skipped", &[("list", &skipped)]));
                    }
                    t
                }
                // W14 — uma imagem sem cena do Excalidraw entra no quadro aberto (o «Upload» do Miro).
                Err(e) => match crate::images::insert_paths(hero, &[path.clone()], None) {
                    p if p.placed > 0 => tr_with("board.io.image_added", &[("name", &display(&path))]),
                    _ => tr_with("board.io.import_failed", &[("error", &e)]),
                },
            };
            hero.documents.tell(text);
        }
    }
}

fn display(p: &Path) -> String {
    p.file_name().map_or_else(
        || p.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// «1 image, 2 embeddable» — o que a importação saltou (X3), pelos nomes do Excalidraw.
fn skipped_list(s: &std::collections::BTreeMap<String, usize>) -> String {
    s.iter()
        .map(|(k, n)| format!("{n} {k}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A extensão e o nome do filtro de cada formato.
fn extension(f: Format) -> (&'static str, &'static str) {
    match f {
        Format::Png => ("png", "board.io.filter_png"),
        Format::Svg => ("svg", "board.io.filter_svg"),
        Format::Pdf => ("pdf", "board.io.filter_pdf"),
        Format::Excalidraw => ("excalidraw", "board.io.filter_excalidraw"),
    }
}

/// O nome de um quadro ou de um frame num nome de ficheiro (sem as barras e os dois pontos).
fn file_stem(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| {
            if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let s = s.trim();
    if s.is_empty() {
        "board".to_owned()
    } else {
        s.to_owned()
    }
}

/// As páginas (PDF) ou imagens (PNG/SVG) de uma escolha (E2/E4).
fn pages_of(b: &Board, routes: &RouteCache, c: ExportChoice, sel: &[ElementId]) -> Vec<Page> {
    match (c.format, c.scope) {
        (Format::Pdf, Scope::Board) => pdf_pages(b, routes, &[]),
        (Format::Pdf, _) => pdf_pages(b, routes, sel),
        (_, Scope::Board) => board_page(b, routes).into_iter().collect(),
        (_, Scope::Selection) => selection_page(b, routes, sel).into_iter().collect(),
        (_, Scope::Frames) => pdf_pages(b, routes, sel)
            .into_iter()
            .filter(|p| p.frame.is_some())
            .collect(),
    }
}

/// A página onde `id` está: a do frame de topo que o contém, ou (páginas sem frame) a que contém o
/// centro dele.
fn page_of(b: &Board, pages: &[Page], id: ElementId) -> Option<usize> {
    let chain = b.doc.frame_chain(id);
    let c = b.doc.get(id)?.center();
    pages.iter().position(|p| match p.frame {
        Some(f) => f == id || chain.contains(&f),
        None => (p.rect[0]..=p.rect[2]).contains(&c[0]) && (p.rect[1]..=p.rect[3]).contains(&c[1]),
    })
}

/// A caixa de `id` em px da página `p` (a escala 1 do vector).
fn on_page(b: &Board, p: &Page, id: ElementId) -> Option<[f64; 4]> {
    let [x0, y0, x1, y1] = b.doc.get(id)?.aabb();
    Some([
        x0 - p.rect[0],
        y0 - p.rect[1],
        x1 - p.rect[0],
        y1 - p.rect[1],
    ])
}

/// ⭐ E5 — as ligações da página `i`: cada elemento com link (W5) cujo alvo está numa página exportada
/// — a mesma ou outra (a auditoria W8 viu um quadro sem frames sair sem nenhuma): a caixa da origem
/// em px desta página, a página do alvo e o alvo.
fn link_pairs(b: &Board, pages: &[Page], i: usize) -> Vec<([f64; 4], usize, ElementId)> {
    b.doc
        .live()
        .filter(|el| page_of(b, pages, el.id) == Some(i))
        .filter_map(|el| {
            let to = el.link()?;
            let page = page_of(b, pages, to)?;
            Some((on_page(b, &pages[i], el.id)?, page, to))
        })
        .collect()
}

/// As ligações do PDF (o alvo pelo canto de cima à esquerda na página dele).
fn links(b: &Board, pages: &[Page], i: usize) -> Vec<pdf::Link> {
    link_pairs(b, pages, i)
        .into_iter()
        .filter_map(|(rect, page, to)| {
            let t = on_page(b, &pages[page], to)?;
            Some(pdf::Link {
                rect,
                page,
                at: [t[0], t[1]],
            })
        })
        .collect()
}

/// Os caminhos das imagens (PNG/SVG de vários frames: o nome do frame no fim, e um número quando dois
/// frames têm o mesmo nome — a auditoria W8 viu o 2.º escrever por cima do 1.º).
fn page_paths(path: &Path, b: &Board, pages: &[Page]) -> Vec<PathBuf> {
    if pages.len() <= 1 {
        return vec![path.to_path_buf()];
    }
    let stem = path
        .file_stem()
        .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
    let ext = path
        .extension()
        .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
    let mut seen = std::collections::BTreeSet::new();
    pages
        .iter()
        .map(|page| {
            let name = page
                .frame
                .and_then(|f| b.doc.get(f))
                .and_then(|f| f.as_frame().map(|x| x.name.clone()))
                .unwrap_or_default();
            let base = format!("{stem}-{}", file_stem(&name));
            let mut file = format!("{base}.{ext}");
            let mut k = 2;
            while !seen.insert(file.clone()) {
                file = format!("{base}-{k}.{ext}");
                k += 1;
            }
            path.with_file_name(file)
        })
        .collect()
}

/// ⭐ Escreve o export de `b` em `path` (e, com vários frames em PNG/SVG, um ficheiro por frame ao
/// lado). Devolve os ficheiros escritos.
///
/// # Errors
/// Nada a exportar, a placa recusou, ou o disco.
#[allow(clippy::too_many_arguments)]
pub fn write_export(
    b: &Board,
    c: ExportChoice,
    sel: &[ElementId],
    gpu: Option<&GpuContext>,
    ts: &mut TextSystem,
    unread: &BTreeSet<ElementId>,
    path: &Path,
) -> Result<Vec<PathBuf>, String> {
    if c.format == Format::Excalidraw {
        // ADR-0182/0183 — a nota que não leio sai sem texto (o `.excalidraw` não tem marcador): nunca
        // um texto que já não existe.
        let mut seen = b.clone();
        for id in unread {
            if let Some(mut el) = seen.doc.get(*id).cloned()
                && let Some(s) = el.shape_mut()
            {
                s.text = ph2d_board_model::RichText::plain("");
                let _ = ph2d_board_model::BoardOp::Put(el).apply(&mut seen.doc);
            }
        }
        std::fs::write(path, ph2d_board_excalidraw::write(&seen)).map_err(|e| e.to_string())?;
        return Ok(vec![path.to_path_buf()]);
    }
    let mut routes = RouteCache::default();
    routes.sync(&b.doc);
    let pages = pages_of(b, &routes, c, sel);
    if pages.is_empty() {
        return Err(tr("board.io.nothing").to_owned());
    }
    let alpha = c.transparent && matches!(c.format, Format::Png | Format::Svg);
    let paper = (!alpha).then_some(EXPORT_PAPER);
    let mut out = Vec::new();
    match c.format {
        Format::Pdf => {
            let recs: Vec<_> = pages
                .iter()
                .map(|p| {
                    let (mut s, size) = paint_page(b, p, 1.0, paper, true, unread, ts);
                    (s.take_record(), size)
                })
                .collect();
            let list: Vec<pdf::PdfPage<'_>> = recs
                .iter()
                .enumerate()
                .map(|(i, (ops, size))| pdf::PdfPage {
                    ops,
                    size: *size,
                    links: links(b, &pages, i),
                })
                .collect();
            std::fs::write(path, pdf::write(&list)).map_err(|e| e.to_string())?;
            out.push(path.to_path_buf());
        }
        Format::Svg => {
            for (i, (p, to)) in pages.iter().zip(page_paths(path, b, &pages)).enumerate() {
                let (mut s, size) = paint_page(b, p, 1.0, paper, true, unread, ts);
                // As ligações de um SVG são as da própria página (um ficheiro, uma página).
                let here: Vec<([f64; 4], [f64; 4])> = link_pairs(b, &pages, i)
                    .into_iter()
                    .filter(|(_, page, _)| *page == i)
                    .filter_map(|(rect, _, to)| Some((rect, on_page(b, p, to)?)))
                    .collect();
                std::fs::write(&to, svg::write_linked(&s.take_record(), size, &here))
                    .map_err(|e| e.to_string())?;
                out.push(to);
            }
        }
        Format::Png => {
            let gpu = gpu.ok_or_else(|| tr("board.io.no_gpu").to_owned())?;
            let mut pass =
                ph2d_render::VelloPass::new(gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, (64, 64))?;
            let tile = png::tile_side(gpu);
            let mp = ph2d_board_export::PNG_MEGAPIXELS[c.size.min(2)];
            for (p, to) in pages.iter().zip(page_paths(path, b, &pages)) {
                let scale = ph2d_board_export::png_scale(p, mp);
                let img = png::render(gpu, &mut pass, b, p, scale, paper, tile, unread, ts)?;
                std::fs::write(&to, png::encode(&img)).map_err(|e| e.to_string())?;
                out.push(to);
            }
        }
        Format::Excalidraw => {}
    }
    Ok(out)
}

/// ⭐ X3 — um ficheiro (`.excalidraw`, `.json`, ou `.png`/`.svg` com a cena) → o nome do quadro novo
/// e o que se importou.
///
/// # Errors
/// O disco, ou um ficheiro que não é do Excalidraw.
pub fn import_file(path: &Path) -> Result<(String, ph2d_board_excalidraw::Import), String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let scene = ph2d_board_excalidraw::scene_from_bytes(&bytes)?;
    let imp = ph2d_board_excalidraw::read_with(&scene, &crate::images::ingest)?;
    let name = path
        .file_name()
        .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let name = name
        .trim_end_matches(".excalidraw.png")
        .trim_end_matches(".excalidraw.svg")
        .trim_end_matches(".excalidraw")
        .trim_end_matches(".json")
        .trim_end_matches(".png")
        .trim_end_matches(".svg")
        .to_owned();
    Ok((name, imp))
}

/// A página de UM frame, para quem a pede pelo nome (as cenas de smoke).
#[must_use]
pub fn frame(b: &Board, f: ElementId) -> Option<Page> {
    frame_page(b, f)
}

#[cfg(test)]
#[path = "io_tests.rs"]
mod tests;
