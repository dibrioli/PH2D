//! A costura com a Timeline do Pixel: a família PUBLICA as linhas de som do sprite activo por
//! quadro, o painel LÊ-as e PEDE edições; a família drena os pedidos e aplica-os ao documento.
//! O painel nunca vê a família, a família nunca vê o painel.

use std::cell::RefCell;
use std::path::Path;
use std::sync::Arc;

use ph2d_pixel_core::Sprite;

use crate::faixas::Edicao;
use crate::{Pico, Sons};

/// As extensões que o «Add Audio…» mostra (as que a `ph2d-audio-decode` lê).
pub const EXTENSOES: [&str; 7] = ["wav", "flac", "ogg", "opus", "mp3", "aif", "aiff"];

/// Uma linha de som sob os quadros.
#[derive(Debug, Clone, PartialEq)]
pub struct Linha {
    /// O nome a mostrar (o último pedaço do asset).
    pub nome: String,
    pub inicio_ms: i32,
    pub mudo: bool,
    /// `None` = o asset não se leu (a linha diz «not found»).
    pub dur_ms: Option<u64>,
    /// Um pico por [`crate::PICO_MS`] da fonte.
    pub picos: Arc<[Pico]>,
}

/// O que o painel pede.
#[derive(Debug, Clone, PartialEq)]
pub enum Pedido {
    /// **Add Audio…**: a família abre o diálogo de ficheiro.
    Escolher,
    Edita(Edicao),
}

thread_local! {
    static LINHAS: RefCell<Vec<Linha>> = const { RefCell::new(Vec::new()) };
    static PEDIDOS: RefCell<Vec<Pedido>> = const { RefCell::new(Vec::new()) };
    /// Os sons decodificados da família (um por asset, fora do Play).
    static SONS: RefCell<Sons> = RefCell::new(Sons::default());
}

/// As linhas de `s` (carrega cada asset uma vez, relativo a `base`, no cache do fio), para o painel.
/// Sem sprite (`None`), nenhuma linha.
pub fn publica(s: Option<&Sprite>, base: Option<&Path>) {
    match s {
        Some(s) => SONS.with(|c| publica_com(s, &mut c.borrow_mut(), base)),
        None => limpa(),
    }
}

/// [`publica`] com um cache dado.
pub fn publica_com(s: &Sprite, sons: &mut Sons, base: Option<&Path>) {
    let v = s
        .audio()
        .iter()
        .map(|f| {
            let som = sons
                .carrega(base, &f.asset)
                .ok()
                .and_then(|()| sons.som(&f.asset));
            Linha {
                nome: f.asset.rsplit('/').next().unwrap_or(&f.asset).to_owned(),
                inicio_ms: f.inicio_ms,
                mudo: f.mudo,
                dur_ms: som.map(crate::Som::dur_ms),
                picos: som.map_or_else(|| Arc::from([]), |s| Arc::clone(&s.picos)),
            }
        })
        .collect();
    LINHAS.with(|l| *l.borrow_mut() = v);
}

/// Sem sprite activo: nenhuma linha.
pub fn limpa() {
    LINHAS.with(|l| l.borrow_mut().clear());
}

/// As linhas publicadas.
#[must_use]
pub fn linhas() -> Vec<Linha> {
    LINHAS.with(|l| l.borrow().clone())
}

/// O painel pede.
pub fn pede(p: Pedido) {
    PEDIDOS.with(|q| q.borrow_mut().push(p));
}

/// A família drena os pedidos.
#[must_use]
pub fn toma() -> Vec<Pedido> {
    PEDIDOS.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

/// A edição de um pedido: o «Add Audio…» pergunta o ficheiro a `escolhe` (o asset é o caminho; a
/// pasta do projeto, quando houver, encurta-o por [`crate::faixas::asset_de`]).
pub fn edicao(
    p: Pedido,
    projeto: Option<&Path>,
    escolhe: impl FnOnce() -> Option<std::path::PathBuf>,
) -> Option<Edicao> {
    match p {
        Pedido::Escolher => {
            escolhe().map(|f| Edicao::Adiciona(crate::faixas::asset_de(&f, projeto)))
        }
        Pedido::Edita(e) => Some(e),
    }
}

/// O `Op` de um pedido sobre `s` (um passo do desfazer); `None` = nada muda ou o diálogo cancelou.
pub fn op(
    s: &Sprite,
    p: Pedido,
    escolhe: impl FnOnce() -> Option<std::path::PathBuf>,
) -> Option<ph2d_pixel_core::Op> {
    edicao(p, None, escolhe).and_then(|e| crate::faixas::op(s, &e))
}
