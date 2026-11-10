//! ⭐⭐ **O LOSANGO DE ANIMAÇÃO na coluna do decorator** (plano 34 §3) — o que fecha o «trabalho a
//! seguir» que o [`super::row::paint_decorator`] declarava: o ponto vira losango quando a linha do
//! tempo conduz o campo.
//!
//! *A shell publica, a folha lê* (o idioma do `published.rs`): a shell sabe que canal da linha do
//! tempo cada campo mostra e em que estado ele está; o pintor da linha só sabe os ids dos campos
//! que desenha. O mapa é o encontro dos dois, publicado uma vez por quadro.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ph2d_a11y::NodeId;
use ph2d_i18n::TextKey;
use ph2d_tokens::{ColorToken, Spacing, Theme};
use ph2d_vector::{Affine, BezPath, Fill, VectorScene};

use crate::paint::resolve;
use crate::zones::Rect;

/// O estado de animação de um campo, como o losango o mostra. A AUSÊNCIA no mapa é «este campo não
/// tem canal» (o ponto de sempre, inerte).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyMark {
    /// O campo tem canal e nenhuma curva o conduz — o ponto de sempre, mas CLICÁVEL: o 1.º clique
    /// grava a 1.ª chave (o cronómetro do After Effects).
    Livre,
    /// Uma curva conduz o campo, sem chave neste quadro — losango vazio.
    Animado,
    /// Há chave neste quadro — losango cheio.
    Chave,
    /// O valor saiu da curva e ninguém o gravou — losango cheio na cor de aviso: perde-se quando o
    /// tempo mudar (o laranja do Blender).
    Mudado,
}

/// O que um clique no losango de uma linha faz — a shell lê-o e escreve na linha do tempo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyMarkClick {
    /// Os campos da linha clicada (um losango por linha: `X`/`Y` partilham o seu).
    pub fields: Vec<NodeId>,
    /// O objecto do cartão preso onde o losango foi clicado (`None` = a seleção).
    pub sujeito: Option<u64>,
}

thread_local! {
    static MARCAS: RefCell<BTreeMap<NodeId, KeyMark>> = const { RefCell::new(BTreeMap::new()) };
    /// C3 (`docs/CartoesUI/09`): o mapa de cada objecto preso, e de quem é o corpo a pintar agora.
    static DOS_PRESOS: RefCell<BTreeMap<u64, BTreeMap<NodeId, KeyMark>>> = const { RefCell::new(BTreeMap::new()) };
    static PINTANDO: Cell<Option<u64>> = const { Cell::new(None) };
    static ALVOS: RefCell<Vec<(NodeId, Vec<NodeId>)>> = const { RefCell::new(Vec::new()) };
    static PEDIDO: RefCell<Option<KeyMarkClick>> = const { RefCell::new(None) };
    static ROTULOS: RefCell<BTreeMap<NodeId, &'static str>> = const { RefCell::new(BTreeMap::new()) };
    static CANAIS: RefCell<BTreeMap<NodeId, Vec<ChaveDeCanal>>> = const { RefCell::new(BTreeMap::new()) };
}

/// O prefixo das chaves de rótulo do catálogo (`ph2d-component-desc`): cada uma nomeia UM campo.
pub const ROTULO_DO_CATALOGO: &str = "component.field.";

/// ⭐ **O nome de uma linha de campo** (F27 B2): o texto que se pinta e, quando é a chave de um campo
/// do catálogo, o canal que o losango da linha mostra — a ligação controlo → campo sai do rótulo,
/// sem tabela à mão. `&str` = só texto; [`Rotulo::vista`] = estado da vista, sem coluna.
#[derive(Clone, Copy, Debug)]
pub struct Rotulo<'a> {
    pub texto: &'a str,
    chave: Option<&'static str>,
    /// Uma chave por campo da linha (a linha que junta dois campos do catálogo, cada caixa o seu).
    por_campo: &'static [TextKey],
    coluna: bool,
    /// As letras das caixas de uma linha de vários números (T29).
    pub eixos: Eixos,
}

/// ⭐ **O que as caixas de uma linha de vários números SÃO** — dá a letra de cada pega no cartão
/// (T29): um eixo (`X · Y · Z · W`), um tamanho (`W · H`, como o Figma) ou um retângulo
/// (`X · Y · W · H`). Declara-se no sítio que sabe o que guarda, nunca se adivinha pelo texto.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Eixos {
    #[default]
    Xyzw,
    Tamanho,
    Retangulo,
    /// Um campo só cuja letra o sítio DECLARA (T56: a regra da última palavra não serve).
    Letra(&'static str),
    /// As letras de uma linha de vários campos que não são eixos (`L` · `T`, `R` · `B` das bordas).
    Letras(&'static [&'static str]),
}

impl Eixos {
    /// A letra da `i`-ésima caixa.
    #[must_use]
    pub fn letra(self, i: usize) -> &'static str {
        let l: &[&str] = match self {
            Self::Xyzw => &["X", "Y", "Z", "W"],
            Self::Tamanho => &["W", "H"],
            Self::Retangulo => &["X", "Y", "W", "H"],
            Self::Letra(l) => return if i == 0 { l } else { "" },
            Self::Letras(l) => l,
        };
        l.get(i).copied().unwrap_or("")
    }
}

impl<'a> From<&'a str> for Rotulo<'a> {
    fn from(texto: &'a str) -> Self {
        Self {
            texto,
            chave: None,
            por_campo: &[],
            coluna: true,
            eixos: Eixos::Xyzw,
        }
    }
}

impl<'a> From<&'a String> for Rotulo<'a> {
    fn from(texto: &'a String) -> Self {
        Self::from(texto.as_str())
    }
}

impl From<TextKey> for Rotulo<'static> {
    fn from(k: TextKey) -> Self {
        Self {
            texto: k.tr(),
            chave: Some(k.key()),
            por_campo: &[],
            coluna: true,
            eixos: Eixos::Xyzw,
        }
    }
}

impl Rotulo<'static> {
    /// ⛔ Estado da VISTA ou da ferramenta (não documento): a coluna fica reservada e VAZIA — um
    /// ponto ali diria «isto anima».
    #[must_use]
    pub fn vista(k: TextKey) -> Self {
        Self {
            coluna: false,
            ..Self::from(k)
        }
    }
}

impl<'a> Rotulo<'a> {
    /// O mesmo rótulo (a chave e a coluna dele) com outro texto.
    #[must_use]
    pub fn com_texto<'b>(&self, texto: &'b str) -> Rotulo<'b> {
        Rotulo {
            texto,
            chave: self.chave,
            por_campo: self.por_campo,
            coluna: self.coluna,
            eixos: self.eixos,
        }
    }

    /// O espaço da marca de animação; o que é [`Rotulo::vista`] não leva marca e o nome fica com ele.
    #[must_use]
    pub fn reserva_da_marca(&self) -> f32 {
        if self.coluna {
            ph2d_card_stack::metrics::KEY_MARK_BOX
        } else {
            0.0
        }
    }

    /// [`Rotulo::vista`] com o nome já traduzido.
    #[must_use]
    pub fn vista_de(texto: &'a str) -> Self {
        Self {
            coluna: false,
            ..Self::from(texto)
        }
    }

    /// Um nome composto (`tr_with`) para o campo `campo` do catálogo.
    #[must_use]
    pub fn de_campo(texto: &'a str, campo: TextKey) -> Self {
        Self {
            chave: Some(campo.key()),
            ..Self::from(texto)
        }
    }

    /// Um nome para uma linha cujas caixas são campos DIFERENTES do catálogo, pela ordem dos ids.
    #[must_use]
    pub fn de_campos(texto: &'a str, campos: &'static [TextKey]) -> Self {
        Self {
            por_campo: campos,
            ..Self::from(texto)
        }
    }

    /// As caixas da linha são um tamanho / um retângulo (ver [`Eixos`]).
    #[must_use]
    pub fn eixos(self, eixos: Eixos) -> Self {
        Self { eixos, ..self }
    }

    /// A chave do campo do catálogo que este rótulo nomeia.
    #[must_use]
    pub fn do_catalogo(&self) -> Option<&'static str> {
        self.chave.filter(|k| k.starts_with(ROTULO_DO_CATALOGO))
    }
}

/// [`paint_decorator_for`] de uma linha com nome: regista cada campo sob o rótulo do catálogo (a
/// shell resolve-o no quadro seguinte: [`rotulos_pintados`]); [`Rotulo::vista`] não pinta nada.
#[must_use]
pub fn paint_decorator_campo(
    scene: &mut VectorScene,
    theme: Theme,
    r: Rect,
    fields: &[NodeId],
    rotulo: Rotulo<'_>,
) -> Option<NodeId> {
    if !rotulo.coluna {
        return None;
    }
    let de = |k: &'static str| Some(k).filter(|k| k.starts_with(ROTULO_DO_CATALOGO));
    ROTULOS.with(|m| {
        let mut m = m.borrow_mut();
        if rotulo.por_campo.is_empty() {
            if let Some(k) = rotulo.do_catalogo() {
                m.extend(fields.iter().map(|&f| (f, k)));
            }
        } else {
            let pares = fields.iter().zip(rotulo.por_campo);
            m.extend(pares.filter_map(|(&f, k)| Some((f, de(k.key())?))));
        }
    });
    paint_decorator_for(scene, theme, r, fields)
}

/// Os campos pintados desde o último [`set_key_marks`] com o rótulo do catálogo de cada um.
#[must_use]
pub fn rotulos_pintados() -> Vec<(NodeId, &'static str)> {
    ROTULOS.with(|m| m.borrow().iter().map(|(&f, &k)| (f, k)).collect())
}

/// O rótulo do catálogo com que o campo `id` foi pintado (o clique resolve-o assim).
#[must_use]
pub fn rotulo_de(id: NodeId) -> Option<&'static str> {
    ROTULOS.with(|m| m.borrow().get(&id).copied())
}

/// A chave de um canal (`ph2d_ecs::FieldKey`: família, campo, `i`, `j`) sem a crate dela — esta é
/// folha de UI.
pub type ChaveDeCanal = [u16; 4];

/// ⭐ **Os campos cujo canal a PINTURA conhece** (as propriedades que o script declara: dinâmicas,
/// fora do catálogo — o instantâneo traz a chave): regista-os sob o grupo `canal` até ao próximo
/// [`set_key_marks`]. Quem pinta a coluna é a porta de sempre.
pub fn regista_canal(fields: &[NodeId], canal: &[ChaveDeCanal]) {
    if canal.is_empty() {
        return;
    }
    CANAIS.with(|m| {
        let mut m = m.borrow_mut();
        m.extend(fields.iter().map(|&f| (f, canal.to_vec())));
    });
}

/// Os campos registados por [`regista_canal`] desde o último [`set_key_marks`].
#[must_use]
pub fn canais_registados() -> Vec<(NodeId, Vec<ChaveDeCanal>)> {
    CANAIS.with(|m| m.borrow().iter().map(|(&f, k)| (f, k.clone())).collect())
}

/// O sal do id do losango: um campo `f` regista o seu losango como `f ^ SAL` no índice de alvos.
const SAL: u64 = 0x6c6f_7361_6e67_6f00; // "losango"

/// O id com que o losango da linha cujo primeiro campo é `campo` entra no índice de alvos — o rato
/// acha-o pela MESMA ordem de empilhamento que os outros widgets (um menu aberto por cima ganha).
#[must_use]
pub const fn key_mark_id(campo: NodeId) -> NodeId {
    NodeId(campo.0 ^ SAL)
}

/// Publica o estado de cada campo animado neste quadro (substitui o anterior) e esvazia os alvos do
/// quadro anterior — a pintura que se segue regista os de agora. Chamado pela shell, uma vez por
/// quadro, antes de pintar os painéis.
pub fn set_key_marks(marks: BTreeMap<NodeId, KeyMark>) {
    MARCAS.with(|m| *m.borrow_mut() = marks);
    ALVOS.with(|a| a.borrow_mut().clear());
    ROTULOS.with(|r| r.borrow_mut().clear());
    CANAIS.with(|r| r.borrow_mut().clear());
}

/// Acrescenta o estado de UM campo ao quadro corrente, sem apagar os outros — a Widget Gallery
/// mostra os quatro estados da marca lado a lado (doc `CartoesUI/08` §1.1).
pub fn add_key_mark(field: NodeId, mark: KeyMark) {
    MARCAS.with(|m| {
        m.borrow_mut().insert(field, mark);
    });
}

/// Publica o mapa de cada objecto preso por um cartão solto (uma vez por quadro).
pub fn set_key_marks_dos_presos(marks: BTreeMap<u64, BTreeMap<NodeId, KeyMark>>) {
    DOS_PRESOS.with(|m| *m.borrow_mut() = marks);
}

/// `f` a pintar o corpo do cartão preso a `pin`: cada losango dentro dele é o desse objecto. Um preso
/// ainda sem mapa não tem canal: nunca mostra o da seleção.
pub fn com_marcas_de<R>(pin: u64, f: impl FnOnce() -> R) -> R {
    let antes = PINTANDO.with(|p| p.replace(Some(pin)));
    let out = f();
    PINTANDO.with(|p| p.set(antes));
    out
}

/// O losango de uma linha com estes campos: o estado mais forte entre eles (`Mudado` > `Chave` >
/// `Animado` > `Livre`); `None` = nenhum tem canal.
#[must_use]
pub fn key_mark_of(fields: &[NodeId]) -> Option<KeyMark> {
    let forte =
        |m: &BTreeMap<NodeId, KeyMark>| fields.iter().filter_map(|id| m.get(id).copied()).max();
    match PINTANDO.with(Cell::get) {
        Some(pin) => DOS_PRESOS.with(|p| p.borrow().get(&pin).and_then(forte)),
        None => MARCAS.with(|m| forte(&m.borrow())),
    }
}

/// O losango cujo id de alvo é `id`, se foi pintado neste quadro: os campos da linha dele.
#[must_use]
pub fn key_mark_target(id: NodeId) -> Option<KeyMarkClick> {
    ALVOS.with(|a| {
        a.borrow()
            .iter()
            .rev()
            .find(|(alvo, _)| *alvo == id)
            .map(|(_, f)| KeyMarkClick {
                fields: f.clone(),
                sujeito: None,
            })
    })
}

/// O despacho do rato regista o clique num losango; a shell lê-o com [`take_key_mark_click`].
/// Regista o alvo clicável da marca de `fields` (o desenho é de quem chama — o campo Figma tem o
/// seu, de 6 px) e devolve o id a registar no `HitIndex`.
pub(crate) fn register_key_mark_target(fields: &[NodeId]) -> Option<NodeId> {
    let &primeiro = fields.first()?;
    ALVOS.with(|a| {
        a.borrow_mut()
            .push((key_mark_id(primeiro), fields.to_vec()))
    });
    Some(key_mark_id(primeiro))
}

pub(crate) fn pedir(c: KeyMarkClick) {
    PEDIDO.with(|p| *p.borrow_mut() = Some(c));
}

/// O clique num losango desde a última leitura (a shell lê-o uma vez por quadro).
#[must_use]
pub fn take_key_mark_click() -> Option<KeyMarkClick> {
    PEDIDO.with(|p| p.borrow_mut().take())
}

/// Pinta a coluna do decorator de uma linha com `fields`: o losango quando algum é animado, o ponto
/// quando não. Quando algum tem canal devolve o id do alvo do clique ([`key_mark_id`]); quem pinta
/// regista-o em `r` no índice de alvos (o `widget` não fala com a `interaction`).
#[must_use]
pub fn paint_decorator_for(
    scene: &mut VectorScene,
    theme: Theme,
    r: Rect,
    fields: &[NodeId],
) -> Option<NodeId> {
    if !crate::paint::ui_is_redesign() {
        return None;
    }
    let (Some(mark), Some(&primeiro)) = (key_mark_of(fields), fields.first()) else {
        super::row::paint_decorator(scene, theme, r, false);
        return None;
    };
    paint_key_mark(scene, theme, r, mark);
    ALVOS.with(|a| {
        a.borrow_mut()
            .push((key_mark_id(primeiro), fields.to_vec()))
    });
    Some(key_mark_id(primeiro))
}

/// ⭐ **A tinta de cada estado** (maquete B1, os 3 do Rive + o laranja do Spine): `Livre` na tinta
/// `apagada` do ponto de quem pinta (`Text3` no clássico, `CardLabel` no cartão); `Animado` e `Chave`
/// na da chave da dope-sheet — o mesmo losango, o mesmo significado; `Mudado` no laranja da maquete
/// ([`cor_mudado`]). `tok` resolve um token na tinta de quem pinta.
#[must_use]
pub fn cor_do_losango(
    mark: KeyMark,
    apagada: ColorToken,
    theme: Theme,
    tok: impl Fn(ColorToken) -> ph2d_vector::Color,
) -> ph2d_vector::Color {
    match mark {
        KeyMark::Livre => tok(apagada),
        KeyMark::Animado | KeyMark::Chave => tok(ColorToken::TimelineKey),
        KeyMark::Mudado => cor_mudado(theme),
    }
}

/// O laranja do `Mudado` (losango, valor do campo): `#FF9F5A` da maquete no tema dela, o mesmo desvio
/// OKLab nos outros — piso e matiz medidos em `o_mudado_e_laranja_em_todo_tema`.
#[must_use]
pub fn cor_mudado(theme: Theme) -> ph2d_vector::Color {
    crate::paint::token_to_vello(ph2d_tokens::TimelineTinta::Mudado.resolve(theme))
}

/// O contorno do campo `Mudado` nos cartões (maquete B1).
#[must_use]
pub fn contorno_mudado(theme: Theme) -> ph2d_vector::Color {
    crate::paint::token_to_vello(ph2d_tokens::TimelineTinta::MudadoContorno.resolve(theme))
}

/// O campo `id` mudou e ninguém o gravou (o valor e o contorno vestem o laranja).
#[must_use]
pub fn mudado(id: NodeId) -> bool {
    key_mark_of(&[id]) == Some(KeyMark::Mudado)
}

/// `Livre` e `Animado` são CONTORNO; `Chave` e `Mudado`, cheios.
#[must_use]
pub const fn losango_vazio(mark: KeyMark) -> bool {
    matches!(mark, KeyMark::Livre | KeyMark::Animado)
}

/// A porta única dos dois Inspectors: o losango de meia diagonal `d` centrado em `(cx, cy)`.
pub(crate) fn pinta_losango(
    scene: &mut VectorScene,
    (cx, cy): (f64, f64),
    d: f64,
    mark: KeyMark,
    cor: ph2d_vector::Color,
) {
    let mut p = BezPath::new();
    p.move_to((cx, cy - d));
    p.line_to((cx + d, cy));
    p.line_to((cx, cy + d));
    p.line_to((cx - d, cy));
    p.close_path();
    if losango_vazio(mark) {
        let traco = ph2d_vector::Stroke::new(f64::from(ph2d_card_stack::metrics::FIELD_BORDER));
        scene
            .inner_mut()
            .stroke(&traco, Affine::IDENTITY, cor, None, &p);
    } else {
        scene
            .inner_mut()
            .fill(Fill::NonZero, Affine::IDENTITY, cor, None, &p);
    }
}

/// O losango no centro de `r` (o Inspector clássico).
pub fn paint_key_mark(scene: &mut VectorScene, theme: Theme, r: Rect, mark: KeyMark) {
    let centro = (f64::from(r.x + r.w * 0.5), f64::from(r.y + r.h * 0.5));
    let cor = cor_do_losango(mark, ColorToken::Text3, theme, |t| resolve(t, theme));
    pinta_losango(scene, centro, f64::from(Spacing::Xs.px()), mark, cor);
}

#[cfg(test)]
#[path = "key_mark_tests.rs"]
mod tests;
