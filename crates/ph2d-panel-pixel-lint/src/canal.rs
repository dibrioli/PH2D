//! O canal entre a família e a janela: a família entrega o documento em Draw a cada quadro
//! ([`corre`]); o lint só corre com a janela aberta, o traço solto e algo mudado (o sprite, o quadro
//! ou as regras) — 10,9 ms em 256² (`P13_plano.md` §12 «A — linter»): por edição, nunca por quadro.
//! Clicar num aviso deixa um FOCO que a família drena com [`foca`] (centra a câmera) e marca as
//! células, que a família desenha com [`desenha`].

use std::cell::RefCell;

use ph2d_pixel_core::{FrameId, Sprite};
use ph2d_pixel_lint::{Aviso, Regra, Regras, Severidade, lint};
use ph2d_vector::{Affine, Color, Point, VectorScene};

/// Os avisos de uma regra.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grupo {
    pub regra: Regra,
    pub severidade: Severidade,
    /// As células de cada aviso.
    pub avisos: Vec<Vec<[u32; 2]>>,
}

/// O que a janela mostra.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LintView {
    /// Há um Pixel Sprite em Draw.
    pub tem_documento: bool,
    /// Agrupados na ordem de [`Regra::TODAS`]; só as regras com avisos.
    pub grupos: Vec<Grupo>,
    /// O aviso escolhido (`regra`, índice no grupo), se ainda existe.
    pub escolhido: Option<(Regra, usize)>,
}

#[derive(Default)]
struct Canal {
    regras: Option<Regras>,
    aberto: bool,
    visto: Option<(u32, FrameId, u64, Regras)>,
    chave: Option<u32>,
    view: LintView,
    /// As células do aviso escolhido (a escolha sobrevive à corrida se o MESMO aviso continuar).
    marcadas: Vec<[u32; 2]>,
    foco: Option<[u32; 2]>,
    afim: Option<Affine>,
    corridas: u64,
}

thread_local! {
    static CANAL: RefCell<Canal> = RefCell::new(Canal::default());
}

fn com<R>(f: impl FnOnce(&mut Canal) -> R) -> R {
    CANAL.with(|c| f(&mut c.borrow_mut()))
}

/// As regras em vigor.
#[must_use]
pub fn regras() -> Regras {
    com(|c| c.regras.unwrap_or_default())
}

/// Troca as regras (a janela: caixa de regra, Color limit). A próxima [`corre`] repete o lint.
pub fn poe_regras(r: Regras) {
    com(|c| c.regras = Some(r));
}

/// A janela diz, a cada quadro pintado, se está à vista (fechada, o lint não corre).
pub fn poe_aberto(aberto: bool) {
    com(|c| c.aberto = aberto);
}

/// **Uma vez por quadro, pela família.** `doc` = (chave, sprite, quadro activo, carimbo do
/// documento) do Pixel Sprite em Draw; `quieto` = nenhum traço em curso.
pub fn corre(doc: Option<(u32, &Sprite, FrameId, u64)>, quieto: bool) {
    com(|c| {
        let Some((k, sprite, frame, stamp)) = doc else {
            c.view = LintView::default();
            c.visto = None;
            c.chave = None;
            c.marcadas.clear();
            return;
        };
        let regras = c.regras.unwrap_or_default();
        let chave = (k, frame, stamp, regras);
        if !c.aberto || !quieto || c.visto == Some(chave) {
            return;
        }
        c.visto = Some(chave);
        c.chave = Some(k);
        c.corridas += 1;
        c.view.tem_documento = true;
        c.view.grupos = agrupa(lint(sprite, frame, &regras));
        c.view.escolhido = c.view.grupos.iter().find_map(|g| {
            let i = g.avisos.iter().position(|a| *a == c.marcadas)?;
            Some((g.regra, i))
        });
        if c.view.escolhido.is_none() {
            c.marcadas.clear();
        }
    });
}

fn agrupa(avisos: Vec<Aviso>) -> Vec<Grupo> {
    let mut grupos: Vec<Grupo> = Vec::new();
    for a in avisos {
        match grupos.iter_mut().find(|g| g.regra == a.regra) {
            Some(g) => g.avisos.push(a.celulas),
            None => grupos.push(Grupo {
                regra: a.regra,
                severidade: a.severidade,
                avisos: vec![a.celulas],
            }),
        }
    }
    grupos
}

#[must_use]
pub fn view() -> LintView {
    com(|c| c.view.clone())
}

/// Quantas vezes o lint correu (o gate de «nunca a cada quadro»).
#[must_use]
pub fn corridas() -> u64 {
    com(|c| c.corridas)
}

/// Escolhe o aviso `i` do grupo `r`: marca as células e deixa o foco na primeira.
pub fn escolhe(r: Regra, i: usize) {
    com(|c| {
        let Some(cels) = c
            .view
            .grupos
            .iter()
            .find(|g| g.regra == r)
            .and_then(|g| g.avisos.get(i))
        else {
            return;
        };
        c.marcadas.clone_from(cels);
        c.foco = cels.first().copied();
        c.view.escolhido = Some((r, i));
    });
}

/// As células marcadas agora.
#[must_use]
pub fn marcadas() -> Vec<[u32; 2]> {
    com(|c| c.marcadas.clone())
}

/// **A família drena o foco, uma vez por quadro**: `area` = `[x, y, w, h]` da área do desenho (px de
/// ecrã), `ecra_mundo` = o `Camera2d::screen_to_world` da família. Devolve quanto somar ao centro
/// da câmera (unidades de mundo) para a primeira célula do aviso cair no meio da área; `[0, 0]` sem
/// foco. O afim célula → ecrã é o do último [`desenha`] (a câmera não mexeu desde ele).
pub fn foca(area: [f32; 4], ecra_mundo: impl Fn((f32, f32)) -> [f32; 2]) -> [f32; 2] {
    let Some(([x, y], m)) = com(|c| c.foco.take().zip(c.afim)) else {
        return [0.0, 0.0];
    };
    let p = m * Point::new(f64::from(x) + 0.5, f64::from(y) + 0.5);
    #[expect(clippy::cast_possible_truncation, reason = "px de ecrã")]
    let a = ecra_mundo((p.x as f32, p.y as f32));
    let b = ecra_mundo((area[0] + area[2] * 0.5, area[1] + area[3] * 0.5));
    [a[0] - b[0], a[1] - b[1]]
}

/// Desenha as marcas do aviso escolhido sobre o documento `key` (a família, por cima da arte) e
/// guarda o afim célula → ecrã para o [`foca`].
pub fn desenha(key: u32, m: Affine, cor: Color, width_px: f64, scene: &mut VectorScene) {
    com(|c| {
        if c.chave == Some(key) {
            c.afim = Some(m);
            ph2d_pixel_render::draw_lint_marks(scene, &c.marcadas, m, cor, width_px);
        }
    });
}
