//! **A paleta, a cor e o modo** (P2) do lado da família: a selecção (a rampa do Shading), o menu ≡
//! da paleta do Aseprite (tamanho, mover, ordenar, usadas, do sprite, ler/gravar, a de omissão), o
//! *Edit color* pelo seletor partilhado, o *Replace Color* e o diálogo *Color Mode*. Tudo o que muda
//! o documento é UM passo do desfazer do sprite; reordenar nunca muda a imagem.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::pixel_panel::{
    ColorModeChoice, NewSpriteMode, PixelDither, PixelRequest, PixelSelect, PixelSortKey,
    PixelToGray,
};
use ph2d_pixel_core::color::ToGray;
use ph2d_pixel_core::convert::{Bayer, Convert, Dither, convert};
use ph2d_pixel_core::palette::{self, SortKey};
use ph2d_pixel_core::{Cel, ColorMode, Op, Patch, Rect, Sprite, Whole, compose_rgba};
use ph2d_pixel_io::palette_file::{self, PaletteFormat};

use crate::PixelApp;

#[path = "paleta_predefinicoes.rs"]
mod predefinicoes;
pub use predefinicoes::{DEFAULT_FILE, pasta_do_utilizador};

/// **Mudar o bloco `sel` (índices crescentes) para a célula `to`** (o arrasto na paleta):
/// `perm[novo] = antigo`. O bloco fica junto, pela ordem dele, e cai como uma amostra só cairia
/// ([`palette::move_permutation`]): largado à frente do primeiro, o ÚLTIMO fica em `to`; atrás,
/// o primeiro fica em `to`.
#[must_use]
pub fn drag_permutation(size: usize, sel: &[usize], to: usize) -> Vec<usize> {
    let (Some(&first), true) = (sel.first(), to < size) else {
        return (0..size).collect();
    };
    if sel.iter().any(|&i| i >= size) {
        return (0..size).collect();
    }
    let k = sel.len();
    let inicio = if to > first { (to + 1).saturating_sub(k) } else { to }.min(size - k);
    let mut perm: Vec<usize> = (0..size).filter(|i| !sel.contains(i)).collect();
    for (j, &i) in sel.iter().enumerate() {
        perm.insert(inicio + j, i);
    }
    perm
}

/// A cor com que *Palette Size* estende a paleta (Aseprite: preto).
const PRETO: [u8; 4] = [0, 0, 0, 255];

/// O modo do documento no vocabulário do chrome.
#[must_use]
pub fn modo(m: ColorMode) -> NewSpriteMode {
    match m {
        ColorMode::Rgba => NewSpriteMode::Rgba,
        ColorMode::Gray => NewSpriteMode::Gray,
        ColorMode::Indexed => NewSpriteMode::Indexed,
    }
}

/// O valor (no modo do sprite) da amostra `i` da paleta: Indexed = o índice, RGBA = os bytes,
/// cinza = a cor levada a cinza.
#[must_use]
pub fn valor_da_amostra(s: &Sprite, i: usize) -> Option<u32> {
    let c = *s.palette().get(i)?;
    Some(match s.mode() {
        ColorMode::Indexed => u32::try_from(i).ok()?,
        ColorMode::Rgba => u32::from_le_bytes(c),
        ColorMode::Gray => s.value_of_rgba(c),
    })
}

/// A rampa do Shading: as amostras seleccionadas em ordem de paleta, como valores do sprite.
#[must_use]
pub fn rampa(s: &Sprite, sel: &BTreeSet<usize>) -> Vec<u32> {
    sel.iter().filter_map(|&i| valor_da_amostra(s, i)).collect()
}

fn sort_key(k: PixelSortKey) -> SortKey {
    match k {
        PixelSortKey::Hue => SortKey::Hue,
        PixelSortKey::Saturation => SortKey::Saturation,
        PixelSortKey::Brightness => SortKey::Brightness,
        PixelSortKey::Luminance => SortKey::Luminance,
        PixelSortKey::Red => SortKey::Red,
        PixelSortKey::Green => SortKey::Green,
        PixelSortKey::Blue => SortKey::Blue,
        PixelSortKey::Alpha => SortKey::Alpha,
    }
}

/// As escolhas do diálogo *Color Mode* no vocabulário do núcleo.
#[must_use]
pub fn conversao(c: ColorModeChoice) -> (ColorMode, Convert) {
    let mode = match c.target {
        NewSpriteMode::Rgba => ColorMode::Rgba,
        NewSpriteMode::Gray => ColorMode::Gray,
        NewSpriteMode::Indexed => ColorMode::Indexed,
    };
    let dither = match c.dither {
        PixelDither::None => Dither::None,
        PixelDither::Ordered2 => Dither::Ordered(Bayer::M2),
        PixelDither::Ordered4 => Dither::Ordered(Bayer::M4),
        PixelDither::Ordered8 => Dither::Ordered(Bayer::M8),
        PixelDither::Old2 => Dither::Old(Bayer::M2),
        PixelDither::Old4 => Dither::Old(Bayer::M4),
        PixelDither::Old8 => Dither::Old(Bayer::M8),
        PixelDither::FloydSteinberg => Dither::FloydSteinberg,
    };
    let to_gray = match c.gray {
        PixelToGray::Luminance => ToGray::Luminance,
        PixelToGray::Hsv => ToGray::Hsv,
        PixelToGray::Hsl => ToGray::Hsl,
    };
    (mode, Convert { dither, to_gray })
}

/// Os cels do sprite, por camada e frame.
fn cels(
    s: &Sprite,
) -> impl Iterator<Item = ((ph2d_pixel_core::LayerId, ph2d_pixel_core::FrameId), &Cel)> {
    s.frames().iter().flat_map(move |f| {
        s.layers()
            .iter()
            .filter_map(move |l| s.cel((l.id, f.id)).map(|c| ((l.id, f.id), c)))
    })
}

/// Os índices que alguma célula usa (Indexed), em ordem de primeira aparição.
fn usados(s: &Sprite) -> Vec<usize> {
    let mut vistos = [false; palette::MAX_COLORS];
    let mut out = Vec::new();
    for (_, c) in cels(s) {
        for &v in &c.data {
            if !std::mem::replace(&mut vistos[usize::from(v)], true) {
                out.push(usize::from(v));
            }
        }
    }
    out
}

impl PixelApp {
    /// A selecção, em ordem de paleta.
    #[must_use]
    pub fn selection(&self) -> Vec<usize> {
        self.sel.iter().copied().collect()
    }

    /// A última amostra clicada (a do *Edit color*).
    #[must_use]
    pub fn anchor(&self) -> Option<usize> {
        self.ultima
    }

    /// Põe a selecção (a cena de smoke, os testes).
    pub fn select(&mut self, sel: impl IntoIterator<Item = usize>) {
        self.sel = sel.into_iter().collect();
        self.ancora = self.sel.first().copied();
        self.ultima = self.ancora;
    }

    fn aviso(&mut self, erro: bool, texto: impl Into<String>) {
        self.avisos.push((erro, texto.into()));
    }

    /// Os avisos por mostrar (`true` = erro), e limpa-os.
    pub fn take_avisos(&mut self) -> Vec<(bool, String)> {
        std::mem::take(&mut self.avisos)
    }

    /// **Um pedido da paleta/cor sobre o documento em Draw** — `false` se não é um destes.
    pub(crate) fn aplica_paleta(&mut self, r: PixelRequest) -> bool {
        let handled = !matches!(
            r,
            PixelRequest::Tool(_)
                | PixelRequest::Fg(_)
                | PixelRequest::Bg(_)
                | PixelRequest::Swap
                | PixelRequest::Options(_)
        );
        if !handled {
            return false;
        }
        if let PixelRequest::SortKey(k) = r {
            self.sort_key = k;
            return true;
        }
        let Some(key) = self.chave.filter(|k| self.docs.contains_key(k)) else {
            // O Draw saiu entre o clique e o quadro: o painel já não oferece o gesto.
            eprintln!("[pixel] pedido da paleta sem sprite em Draw: {r:?}");
            return true;
        };
        match r {
            PixelRequest::Swatch { index, how } => self.clica(key, index, how),
            PixelRequest::PaletteSize(n) => self.tamanho(key, usize::from(n)),
            PixelRequest::Sort { descending } => self.ordena(key, descending),
            PixelRequest::MoveSelection(d) => self.move_selecao(key, d),
            PixelRequest::SelectUsed => self.seleciona_usadas(key),
            PixelRequest::PaletteFromSprite => self.do_sprite(key),
            PixelRequest::LoadPalette => self.le(key),
            PixelRequest::SavePalette => self.grava(key),
            PixelRequest::DefaultPalette => {
                let c = self.paleta_de_omissao();
                self.poe_paleta(key, c, None);
            }
            PixelRequest::DragSwatch { from, to } => self.arrasta(key, from, to),
            PixelRequest::SavePreset => self.grava_predefinicao_pelo_dialogo(key),
            PixelRequest::SaveDefault => self.grava_omissao(key),
            PixelRequest::LoadPreset(nome) => self.carrega_predefinicao(key, &nome),
            PixelRequest::ReplaceColor => self.substitui(key),
            _ => {}
        }
        true
    }

    fn clica(&mut self, key: u32, index: usize, how: PixelSelect) {
        let Some(&c) = self.docs[&key].sprite.palette().get(index) else {
            return;
        };
        self.fg = c;
        match how {
            PixelSelect::Only => {
                self.sel = BTreeSet::from([index]);
                self.ancora = Some(index);
            }
            PixelSelect::Range => {
                let a = self.ancora.unwrap_or(index);
                self.sel = (a.min(index)..=a.max(index)).collect();
                self.ancora = Some(a);
            }
            PixelSelect::Toggle => {
                if !self.sel.remove(&index) {
                    self.sel.insert(index);
                }
                self.ancora = Some(index);
            }
        }
        self.ultima = Some(index);
    }

    /// Prende a selecção ao tamanho da paleta.
    fn prende_selecao(&mut self, n: usize) {
        self.sel.retain(|&i| i < n);
        self.ancora = self.ancora.filter(|&i| i < n);
        self.ultima = self.ultima.filter(|&i| i < n);
    }

    /// Troca a paleta inteira (`transparent = None`: o de agora, preso à paleta nova).
    fn poe_paleta(&mut self, key: u32, colors: Vec<[u8; 4]>, transparent: Option<u8>) {
        if colors.is_empty() {
            return;
        }
        let s = &self.docs[&key].sprite;
        let ultimo = u8::try_from(colors.len() - 1).unwrap_or(u8::MAX);
        let transparent = transparent.unwrap_or(s.transparent_index()).min(ultimo);
        if s.palette() == colors.as_slice() && s.transparent_index() == transparent {
            return;
        }
        let n = colors.len();
        let _ = self.commit(
            key,
            Op::Palette {
                colors,
                transparent,
            },
        );
        self.prende_selecao(n);
    }

    fn tamanho(&mut self, key: u32, n: usize) {
        let n = n.clamp(1, palette::MAX_COLORS);
        let mut colors = self.docs[&key].sprite.palette().to_vec();
        colors.resize(n, PRETO);
        self.poe_paleta(key, colors, None);
    }

    fn ordena(&mut self, key: u32, descending: bool) {
        let s = &self.docs[&key].sprite;
        let n = s.palette().len();
        let range = match (self.sel.first(), self.sel.last()) {
            (Some(&a), Some(&b)) if self.sel.len() >= 2 => a..b + 1,
            _ => 0..n,
        };
        let t = (s.mode() == ColorMode::Indexed).then(|| usize::from(s.transparent_index()));
        let perm =
            palette::sort_permutation(s.palette(), range, sort_key(self.sort_key), descending, t);
        if perm.iter().enumerate().any(|(i, &o)| i != o) {
            let _ = self.commit(key, Op::Remap { perm });
        }
    }

    fn move_selecao(&mut self, key: u32, d: i8) {
        let n = self.docs[&key].sprite.palette().len();
        let sel: Vec<usize> = self.sel.iter().copied().collect();
        let (Some(&a), Some(&b)) = (sel.first(), sel.last()) else {
            return;
        };
        let mut perm: Vec<usize> = (0..n).collect();
        if d < 0 {
            if a == 0 {
                return;
            }
            for &i in &sel {
                perm.swap(i - 1, i);
            }
        } else {
            if b + 1 >= n {
                return;
            }
            for &i in sel.iter().rev() {
                perm.swap(i, i + 1);
            }
        }
        if self.commit(key, Op::Remap { perm }).is_ok() {
            let anda = |i: usize| if d < 0 { i - 1 } else { i + 1 };
            self.sel = sel.into_iter().map(anda).collect();
            self.ancora = self.ancora.map(anda);
            self.ultima = self.ultima.map(anda);
        }
    }

    /// O arrasto de `from` para `to`: a selecção (ou só `from`, se não estava nela) muda-se para lá,
    /// UM passo, e continua seleccionada no sítio novo.
    fn arrasta(&mut self, key: u32, from: usize, to: usize) {
        let n = self.docs[&key].sprite.palette().len();
        if from >= n || to >= n {
            return;
        }
        let bloco: Vec<usize> = if self.sel.contains(&from) {
            self.sel.iter().copied().collect()
        } else {
            vec![from]
        };
        let perm = drag_permutation(n, &bloco, to);
        if perm.iter().enumerate().all(|(i, &o)| i == o) {
            return;
        }
        let novo = palette::invert_permutation(&perm);
        if self.commit(key, Op::Remap { perm }).is_ok() {
            self.sel = bloco.iter().map(|&i| novo[i]).collect();
            self.ancora = self.sel.first().copied();
            self.ultima = Some(novo[from]);
            if let Some(&c) = self.docs[&key].sprite.palette().get(novo[from]) {
                self.fg = c;
            }
        }
    }

    fn seleciona_usadas(&mut self, key: u32) {
        let s = &self.docs[&key].sprite;
        if s.mode() != ColorMode::Indexed {
            self.aviso(false, ph2d_i18n::tr("pixel.palette.indexed_only"));
            return;
        }
        let usados = usados(s);
        self.sel = usados.iter().copied().collect();
        self.ancora = usados.first().copied();
        self.ultima = self.ancora;
    }

    /// *New Palette from Sprite*: as cores distintas da imagem, pela ordem em que aparecem. Em
    /// Indexed as células seguem as cores (o documento inteiro, UM passo) e a imagem não muda.
    fn do_sprite(&mut self, key: u32) {
        let s = &self.docs[&key].sprite;
        if s.mode() == ColorMode::Indexed {
            let t = usize::from(s.transparent_index());
            let transparente = s.palette().get(t).copied().unwrap_or([0; 4]);
            let mut colors = vec![transparente];
            let mut por_cor: BTreeMap<[u8; 4], u8> = BTreeMap::new();
            let mut mapa = [0u8; palette::MAX_COLORS];
            for i in usados(s).into_iter().filter(|&i| i != t) {
                let c = s.palette().get(i).copied().unwrap_or([0; 4]);
                let novo = *por_cor.entry(c).or_insert_with(|| {
                    colors.push(c);
                    u8::try_from(colors.len() - 1).unwrap_or(u8::MAX)
                });
                mapa[i] = novo;
            }
            let cels = cels(s)
                .map(|(k, c)| {
                    let data = c.data.iter().map(|&v| mapa[usize::from(v)]).collect();
                    (k, Cel { bbox: c.bbox, data })
                })
                .collect();
            let w = Whole {
                mode: ColorMode::Indexed,
                palette: colors,
                transparent: 0,
                cels,
            };
            let n = w.palette.len();
            let _ = self.commit(key, Op::Whole(Box::new(w)));
            self.prende_selecao(n);
            return;
        }
        let mut vistas = BTreeSet::new();
        let mut colors = Vec::new();
        let mut buf = Vec::new();
        for f in s.frames() {
            compose_rgba(s, f.id, None, &mut buf);
            for p in buf.as_chunks::<4>().0 {
                if p[3] > 0 && vistas.insert(*p) {
                    colors.push(*p);
                }
            }
        }
        if colors.len() > palette::MAX_COLORS {
            let n = colors.len().to_string();
            self.aviso(
                false,
                ph2d_i18n::tr_with("pixel.palette.too_many", &[("n", &n)]),
            );
            colors.truncate(palette::MAX_COLORS);
        }
        self.poe_paleta(key, colors, Some(0));
    }

    fn le(&mut self, key: u32) {
        let dialogo = rfd::FileDialog::new().add_filter(
            ph2d_i18n::tr("pixel.palette.file_kind"),
            &PaletteFormat::READABLE,
        );
        let Some(caminho) = ph2d_app_host::modal::pick_file(dialogo) else {
            return;
        };
        let ext = caminho
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_lowercase);
        let lida = std::fs::read(&caminho)
            .map_err(|e| e.to_string())
            .and_then(|b| palette_file::read(&b, ext.as_deref()).map_err(|e| format!("{e:?}")));
        match lida {
            Ok(f) => {
                let n = f.colors.len().to_string();
                self.poe_paleta(key, f.colors, f.transparent);
                self.aviso(
                    false,
                    ph2d_i18n::tr_with("pixel.palette.loaded", &[("n", &n)]),
                );
            }
            Err(e) => {
                let texto = ph2d_i18n::tr_with("pixel.palette.load_failed", &[("e", &e)]);
                self.aviso(true, texto);
            }
        }
    }

    fn grava(&mut self, key: u32) {
        let exts: Vec<&str> = PaletteFormat::WRITABLE.iter().map(|(_, e)| *e).collect();
        let dialogo = rfd::FileDialog::new()
            .add_filter(ph2d_i18n::tr("pixel.palette.file_kind"), &exts)
            .set_file_name(format!("palette.{}", exts[0]));
        let Some(caminho) = ph2d_app_host::modal::save_file(dialogo) else {
            return;
        };
        let ext = caminho
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_lowercase);
        let formato = PaletteFormat::WRITABLE
            .iter()
            .find(|(_, e)| Some(*e) == ext.as_deref())
            .map_or(PaletteFormat::Gpl, |(f, _)| *f);
        let s = &self.docs[&key].sprite;
        let nome = caminho
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let t = (s.mode() == ColorMode::Indexed).then(|| s.transparent_index());
        let bytes = palette_file::write(formato, nome, s.palette(), t);
        let caminho_txt = caminho.display().to_string();
        match std::fs::write(&caminho, bytes) {
            Ok(()) => self.aviso(
                false,
                ph2d_i18n::tr_with("pixel.palette.saved", &[("path", &caminho_txt)]),
            ),
            Err(e) => self.aviso(
                true,
                ph2d_i18n::tr_with("pixel.palette.save_failed", &[("e", &e.to_string())]),
            ),
        }
    }

    /// *Replace Color*: no cel activo, a cor de frente vira a de fundo (tolerância por canal RGBA;
    /// Indexed com tolerância 0 compara o ÍNDICE — a lei do balde).
    fn substitui(&mut self, key: u32) {
        let doc = &self.docs[&key];
        let s = &doc.sprite;
        let ck = (doc.layer, doc.frame);
        let de = s.value_of_rgba(self.fg);
        let para = s.value_of_rgba(self.bg);
        let tol = self.options.replace_tolerance;
        let por_indice = s.mode() == ColorMode::Indexed && tol == 0;
        let norm = |c: [u8; 4]| if c[3] == 0 { [0; 4] } else { c };
        let alvo = norm(s.rgba_of_value(de));
        let igual = |v: u32| {
            if por_indice {
                return v == de;
            }
            let c = norm(s.rgba_of_value(v));
            (0..4).all(|k| c[k].abs_diff(alvo[k]) <= tol)
        };
        // Fora do cel tudo é transparente: só conta se a cor de frente o é.
        let caixa = match s.cel(ck) {
            Some(c) if !igual(s.transparent_value()) => c.bbox,
            _ => s.bounds(),
        };
        let mut celulas = Vec::new();
        for y in caixa.y..caixa.y + caixa.h as i32 {
            for x in caixa.x..caixa.x + caixa.w as i32 {
                let v = s.value_at(ck, x, y);
                if v != para && igual(v) {
                    celulas.push((x, y));
                }
            }
        }
        if celulas.is_empty() {
            self.aviso(false, ph2d_i18n::tr("pixel.replace.nothing"));
            return;
        }
        let rect = celulas
            .iter()
            .fold(Rect::default(), |r, &(x, y)| r.union(Rect::new(x, y, 1, 1)));
        let bpp = s.mode().bpp();
        let mut pixels = s.read_rect(ck, rect);
        for (x, y) in celulas {
            let i = ((y - rect.y) as usize * rect.w as usize + (x - rect.x) as usize) * bpp;
            pixels[i..i + bpp].copy_from_slice(&para.to_le_bytes()[..bpp]);
        }
        let bbox = s.cel(ck).map_or(rect, |c| c.bbox.union(rect));
        let op = Op::Patch(Patch {
            layer: doc.layer,
            frame: doc.frame,
            bbox: Some(bbox),
            rect,
            pixels,
        });
        let _ = self.commit(key, op);
    }

    /// O OK do diálogo *Color Mode*: o sprite convertido, UM passo do desfazer.
    pub(crate) fn converte(&mut self, key: u32, c: ColorModeChoice) {
        let Some(doc) = self.docs.get(&key) else {
            return;
        };
        let (mode, opt) = conversao(c);
        let Some(w) = convert(&doc.sprite, mode, opt) else {
            return;
        };
        if self.commit(key, Op::Whole(Box::new(w))).is_ok() {
            let nome = crate::panel::mode_label(c.target);
            self.aviso(
                false,
                ph2d_i18n::tr_with("pixel.color_mode.done", &[("mode", &nome)]),
            );
        }
    }

    /// ⭐ **O *Edit color***: com o seletor partilhado aberto na amostra `PIXEL_PAL_EDIT`, a cor dele
    /// escreve a da última amostra clicada — e arrastar reescreve o MESMO passo do desfazer. Fechado,
    /// a amostra volta a mostrar a cor da paleta.
    pub(crate) fn sincroniza_edicao(&mut self, store: &mut WidgetStore, key: Option<u32>) {
        let id = ph2d_editor_core::ids::PIXEL_PAL_EDIT;
        let Some((key, i)) = key.zip(self.ultima) else {
            self.edicao = None;
            return;
        };
        let Some(&cor) = self.docs.get(&key).and_then(|d| d.sprite.palette().get(i)) else {
            self.edicao = None;
            return;
        };
        if store.picker_target() != Some(id) {
            self.edicao = None;
            store.set_widget_color(id, cor);
            return;
        }
        let Some(nova) = store.widget_color(id).filter(|w| *w != cor) else {
            return;
        };
        let passos = self.docs[&key].history.undo_len();
        if self.edicao == Some((key, i, passos)) {
            self.undo(key);
        }
        let s = &self.docs[&key].sprite;
        let mut colors = s.palette().to_vec();
        colors[i] = nova;
        let transparent = s.transparent_index();
        if self
            .commit(
                key,
                Op::Palette {
                    colors,
                    transparent,
                },
            )
            .is_ok()
        {
            self.edicao = Some((key, i, self.docs[&key].history.undo_len()));
        }
    }
}
