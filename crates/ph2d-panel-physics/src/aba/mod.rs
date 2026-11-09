//! ⭐ **A aba «Collision Layers»** (onda 9a, versão B do dono 07/11): uma aba no centro, ao lado de
//! Scene, com o aspeto do editor de animações. À esquerda a lista «Layers» (nome e quantos objetos), em
//! cima à direita a grade «Who bumps into whom» só com as camadas visíveis, nomes nas duas bordas, e uma
//! frase FIXA no pé no lugar da dica flutuante; embaixo, a linha de título da camada aberta (o resto da
//! camada aberta é a onda 9b).
//!
//! A aba é da família que `ph2d-app-physics` instala (`DocumentSurface`): ela dá o título, o fecho e a
//! ÁREA ([`poe_area`]); quem pinta e regista os alvos é este painel, sobre a camada de cima, para as
//! células serem os MESMOS ids da matriz de sempre (despacho em `event.rs`, undo, Auto Key, arrasto que
//! pinta, losango).

use std::cell::Cell;

use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_physics_ecs::{LayerMatrix, LayerNames, MAX_LAYERS};

use crate::ids as i;

/// Onde se escreve o nome: na lista ou no rótulo da linha da grade.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Onde {
    Lista,
    Grade,
}

thread_local! {
    pub(crate) static ABERTA: Cell<bool> = const { Cell::new(false) };
    /// A camada aberta: a selecionada na lista, acesa na grade.
    pub(crate) static CAMADA: Cell<usize> = const { Cell::new(0) };
    /// A geometria do último quadro pintado (os testes medem os botões e a rolagem por ela).
    pub(crate) static GEOMETRIA: Cell<Option<Geometria>> = const { Cell::new(None) };
    /// A rolagem da grade: (primeira linha, primeira coluna) à vista.
    pub(crate) static ROLAGEM: Cell<(usize, usize)> = const { Cell::new((0, 0)) };
    /// O arrasto de uma barra da grade: (vertical?, onde o polegar foi agarrado, do começo dele).
    pub(crate) static ARRASTO: Cell<Option<(bool, f32)>> = const { Cell::new(None) };
    pub(crate) static EDITANDO: Cell<Option<(usize, Onde)>> = const { Cell::new(None) };
    /// A aba pediu para vir à frente (a superfície entrega-o à barra de abas).
    pub(crate) static PEDIDO: Cell<bool> = const { Cell::new(false) };
    /// A área da aba neste quadro; só existe enquanto a aba está à frente (`fixa` = a dos testes, que não
    /// têm a barra de abas a dizê-la a cada quadro).
    pub(crate) static AREA: Cell<Option<(Rect, bool)>> = const { Cell::new(None) };
    /// A LENTE do pintor: o que o último `paint` desenhou (a frase do pé, as linhas e colunas acesas).
    pub(crate) static LENTE: std::cell::RefCell<Lente> = std::cell::RefCell::default();
    /// A camada acabada de criar no «+ Add»: o pé diz que ela nasce batendo em tudo.
    pub(crate) static NOVA: Cell<Option<usize>> = const { Cell::new(None) };
    /// «+N more» aberto: a fila inteira dos objetos da camada aberta.
    pub(crate) static MAIS: Cell<bool> = const { Cell::new(false) };
    /// O «Remove» a perguntar para onde vão os objetos: o destino escolhido.
    pub(crate) static REMOVENDO: Cell<Option<usize>> = const { Cell::new(None) };
    /// O quadradinho de cor de cada linha da lista «Layers» no último quadro: (caixa, cor) — os testes.
    pub(crate) static QUADRADINHOS: std::cell::RefCell<std::collections::BTreeMap<usize, (Rect, [f32; 4])>> =
        const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
    /// O quadradinho de cor de cada opção aberta do destino do Remove no último quadro — os testes.
    pub(crate) static QUADRADINHOS_DO_DESTINO: std::cell::RefCell<std::collections::BTreeMap<usize, (Rect, [f32; 4])>> =
        const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}

/// A aba existe (à frente ou atrás de Scene)?
#[must_use]
pub fn aberta() -> bool {
    ABERTA.get()
}

/// A camada aberta.
#[must_use]
pub fn camada_aberta() -> usize {
    CAMADA.get()
}

/// A linha acesa na grade: a da camada aberta, com a aba aberta.
#[must_use]
pub fn linha_em_destaque() -> Option<usize> {
    aberta().then(camada_aberta)
}

/// A camada cujo nome está em edição.
#[must_use]
pub fn editando() -> Option<usize> {
    EDITANDO.get().filter(|_| aberta()).map(|(n, _)| n)
}

/// Abre a aba (e trá-la à frente); `camada` = a do objeto (Inspector), já aberta.
pub fn abre(camada: Option<usize>) {
    ABERTA.set(true);
    PEDIDO.set(true);
    if let Some(c) = camada {
        CAMADA.set(c.min(MAX_LAYERS - 1));
    }
    EDITANDO.set(None);
    NOVA.set(None);
    // A pergunta do «Remove» era da camada de antes (auditoria do fecho: remover a 2 para a 2).
    REMOVENDO.set(None);
}

/// Fecha a aba (o × da barra de abas). Um nome a meio perde-se.
pub fn fecha() {
    ABERTA.set(false);
    PEDIDO.set(false);
    EDITANDO.set(None);
    REMOVENDO.set(None);
    NOVA.set(None);
    AREA.set(None);
    ROLAGEM.set((0, 0));
    GEOMETRIA.set(None);
}

/// A superfície tira o pedido de vir à frente (uma vez).
#[must_use]
pub fn tira_pedido() -> bool {
    PEDIDO.replace(false)
}

/// A superfície diz a área da aba, a cada quadro em que ela está à frente.
pub fn poe_area(area: Rect) {
    AREA.set(Some((area, false)));
}

/// A área fixa dos testes de painel (sem a barra de abas da shell): vale até `fecha`.
pub fn poe_area_fixa(area: Rect) {
    AREA.set(Some((area, true)));
}

/// ⭐ A roda sobre a grade rola-a uma célula por passo (Shift = na horizontal); fora dela, não é tomada.
pub fn rola(x: f32, y: f32, dy: f32, shift: bool) -> bool {
    let Some(g) = GEOMETRIA.get() else {
        return false;
    };
    if !g.grade.contains(x, y) || dy == 0.0 {
        return false;
    }
    let passo = |v: usize, cabem: usize| {
        // `dy` > 0 é a roda para cima: volta ao começo.
        let novo = if dy > 0.0 { v.saturating_sub(1) } else { v + 1 };
        novo.min(g.k.saturating_sub(cabem))
    };
    let (l0, c0) = (g.l0, g.c0);
    let novo = if shift {
        (l0, passo(c0, g.cabem_c))
    } else {
        (passo(l0, g.cabem_l), c0)
    };
    ROLAGEM.set(novo);
    GEOMETRIA.set(Some(g.rolada(novo)));
    novo != (l0, c0)
}

/// ⭐ Onda 9f: carregar numa barra da grade agarra o polegar (fora dele, o polegar salta para o ponteiro, pelo
/// meio) e rola já; fora das barras não é tomado.
pub fn agarra(x: f32, y: f32) -> bool {
    let Some(g) = GEOMETRIA.get() else {
        return false;
    };
    let pega = |calha: Option<Rect>, polegar: Option<Rect>, vertical: bool| {
        let (c, p) = (calha?, polegar?);
        c.contains(x, y).then(|| {
            let (pos, ini, len) = if vertical {
                (y, p.y, p.h)
            } else {
                (x, p.x, p.w)
            };
            let dentro = (ini..ini + len).contains(&pos);
            (vertical, if dentro { pos - ini } else { len * 0.5 })
        })
    };
    let Some(a) = pega(g.barra_vertical(), g.polegar_vertical(), true)
        .or_else(|| pega(g.barra_horizontal(), g.polegar_horizontal(), false))
    else {
        return false;
    };
    ARRASTO.set(Some(a));
    let _ = arrasta(x, y);
    true
}

/// Com um polegar agarrado, o ponteiro em `(x, y)` diz a primeira linha (ou coluna) à vista.
pub fn arrasta(x: f32, y: f32) -> bool {
    let (Some((vertical, pega)), Some(g)) = (ARRASTO.get(), GEOMETRIA.get()) else {
        return false;
    };
    let novo = if vertical {
        (g.primeira_sob(true, y, pega), g.c0)
    } else {
        (g.l0, g.primeira_sob(false, x, pega))
    };
    ROLAGEM.set(novo);
    GEOMETRIA.set(Some(g.rolada(novo)));
    true
}

/// Larga o polegar; `true` se havia um agarrado.
pub fn solta() -> bool {
    ARRASTO.take().is_some()
}

/// O polegar agarrado: `Some(true)` = o vertical.
#[must_use]
pub fn arrastando() -> Option<bool> {
    ARRASTO.get().map(|(v, _)| v)
}

pub(crate) fn regista_quadradinho(n: usize, caixa: Rect, cor: [f32; 4]) {
    QUADRADINHOS.with(|m| m.borrow_mut().insert(n, (caixa, cor)));
}

/// O quadradinho de cor da linha `n` da lista «Layers» (caixa e cor), o último pintado.
#[must_use]
pub fn quadradinho_da_lista(n: usize) -> Option<(Rect, [f32; 4])> {
    QUADRADINHOS.with(|m| m.borrow().get(&n).copied())
}

pub(crate) fn regista_quadradinho_do_destino(d: usize, caixa: Rect, cor: [f32; 4]) {
    QUADRADINHOS_DO_DESTINO.with(|m| m.borrow_mut().insert(d, (caixa, cor)));
}

/// O quadradinho de cor da opção `d` da lista aberta do destino do Remove (caixa e cor), o último pintado.
#[must_use]
pub fn quadradinho_do_destino(d: usize) -> Option<(Rect, [f32; 4])> {
    QUADRADINHOS_DO_DESTINO.with(|m| m.borrow().get(&d).copied())
}

/// A geometria do último quadro pintado da aba.
#[must_use]
pub fn geometria_da_ultima() -> Option<Geometria> {
    GEOMETRIA.get()
}

/// O que o último `paint` da aba desenhou.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Lente {
    /// A frase fixa do pé da grade.
    pub pe: String,
    /// As camadas cuja linha da grade se acendeu.
    pub linhas: Vec<usize>,
    /// As camadas cuja coluna da grade se acendeu.
    pub colunas: Vec<usize>,
    /// Cada linha da lista: (camada, rótulo, contagem).
    pub lista: Vec<(usize, String, u32)>,
    /// A linha de título da camada aberta.
    pub titulo: (String, String),
    /// Projeto novo: a frase que o cartão de baixo mostra no lugar de «Bumps into» (onda 9e).
    pub frase_da_camada: Option<String>,
}

/// A lente do último `paint`.
#[must_use]
pub fn lente() -> Lente {
    LENTE.with(|l| l.borrow().clone())
}

pub(crate) fn abre_camada(n: usize) {
    CAMADA.set(n.min(MAX_LAYERS - 1));
    MAIS.set(false);
    REMOVENDO.set(None);
}

/// O destino escolhido, com a pergunta do «Remove» aberta.
#[must_use]
pub fn removendo() -> Option<usize> {
    REMOVENDO.get().filter(|_| aberta())
}

pub(crate) fn pergunta_o_destino(d: Option<usize>) {
    REMOVENDO.set(d);
}

/// «Slime, Slime 2 and Bat».
#[must_use]
pub fn lista_de_nomes(nomes: &[String]) -> String {
    match nomes {
        [] => String::new(),
        [um] => um.clone(),
        [resto @ .., ultimo] => format!(
            "{}{}{ultimo}",
            resto.join(tr("card.physics.tab_names_sep")),
            tr("card.physics.tab_names_last")
        ),
    }
}

pub(crate) fn mostra_todos() {
    MAIS.set(true);
}

/// A cor de uma camada: o token fluorescente ÚNICO dela, pelo número (onda 9c; nada se grava).
#[must_use]
pub fn cor_da_camada(n: usize) -> ph2d_tokens::ColorToken {
    ph2d_tokens::ColorToken::layer(n)
}

/// O par da frase do pé, em negrito: «Player Shots × Player:».
/// Com os nomes INTEIROS (onda 9e): o pé e o título têm a largura da grade, e o corte «…» fica para as bordas.
#[must_use]
pub fn par_do_pe(sob: Option<(usize, usize)>, nomes: &LayerNames) -> Option<String> {
    sob.map(|(a, b)| {
        tr_with(
            "card.physics.tab_pair",
            &[
                ("a", &nome_inteiro(nomes, a)),
                ("b", &nome_inteiro(nomes, b)),
            ],
        )
    })
}

pub(crate) fn nova_camada(n: usize) {
    abre_camada(n);
    NOVA.set(Some(n));
}

pub(crate) fn comeca_a_renomear(camada: usize, onde: Onde) {
    EDITANDO.set(Some((camada, onde)));
}

pub(crate) fn acaba_de_renomear() {
    EDITANDO.set(None);
}

/// O que se lê de uma camada: o nome dado; sem nome, a 0 é «Default» e as outras «Layer n». O padrão
/// nunca se grava no projeto.
#[must_use]
pub fn rotulo(nomes: &LayerNames, n: usize) -> String {
    ph2d_editor_core::nome_curto::nome_de_camada(&nome_inteiro(nomes, n))
}

/// O mesmo sem o corte: o que a DICA mostra ao pairar sobre um nome com «…» (onda 9d).
#[must_use]
pub fn nome_inteiro(nomes: &LayerNames, n: usize) -> String {
    match nomes.0.get(n).map(String::as_str) {
        Some(nome) if !nome.is_empty() => nome.to_owned(),
        _ if n == 0 => tr("card.physics.layer_zero").to_owned(),
        _ => tr_with("card.physics.layer_default", &[("n", &n.to_string())]),
    }
}

/// A dica de um rótulo de camada: com o nome inteiro quando o rótulo o cortou.
#[must_use]
pub fn dica_do_rotulo(nomes: &LayerNames, n: usize) -> String {
    let inteiro = nome_inteiro(nomes, n);
    if rotulo(nomes, n) == inteiro {
        tr("card.physics.layer_label_tip").to_owned()
    } else {
        tr_with("card.physics.layer_label_tip_named", &[("name", &inteiro)])
    }
}

/// Os pares das camadas `visiveis` e quantos atravessam (não colidem).
#[must_use]
pub fn pares(visiveis: &[usize], m: &LayerMatrix) -> (usize, usize) {
    let todos = visiveis.iter().flat_map(|&a| {
        visiveis
            .iter()
            .filter(move |&&b| b <= a)
            .map(move |&b| (a, b))
    });
    todos.fold((0, 0), |(n, off), (a, b)| {
        (n + 1, off + usize::from(!m.collides(a, b)))
    })
}

/// O resumo do cartão do painel: quantas camadas à vista e quantos pares não colidem.
#[must_use]
pub fn resumo() -> String {
    let m = LayerMatrix::from_rows(crate::state::current().vivo.layer_matrix);
    let visiveis = crate::state::visible_layers();
    let (_, desligados) = pares(&visiveis, &m);
    tr_with(
        "card.physics.matrix_summary",
        &[
            ("n", &visiveis.len().to_string()),
            ("off", &desligados.to_string()),
        ],
    )
}

/// ⭐ **A frase fixa do pé da grade**: com o ponteiro numa célula, o par por nome (linha × coluna) e o
/// que o clique fará; sem ponteiro, o resumo (ou, numa camada acabada de criar, que ela bate em tudo).
#[must_use]
pub fn frase_do_pe(
    sob: Option<(usize, usize)>,
    nomes: &LayerNames,
    m: &LayerMatrix,
    visiveis: &[usize],
    nova: Option<usize>,
) -> String {
    if let Some((a, b)) = sob {
        let chave = if m.collides(a, b) {
            "card.physics.tab_pair_bump"
        } else {
            "card.physics.tab_pair_pass"
        };
        let par = par_do_pe(Some((a, b)), nomes).unwrap_or_default();
        return format!("{par} {}", tr(chave));
    }
    if nova.is_some_and(|n| visiveis.contains(&n)) {
        return tr("card.physics.tab_new_layer").to_owned();
    }
    let (n, off) = pares(visiveis, m);
    tr_with(
        "card.physics.tab_summary",
        &[("n", &n.to_string()), ("off", &off.to_string())],
    )
}

/// A linha de título da camada aberta: «Player · 1 object» e «4 bump · 1 pass through».
#[must_use]
pub fn titulo_da_camada(
    n: usize,
    nomes: &LayerNames,
    m: &LayerMatrix,
    visiveis: &[usize],
    objetos: u32,
) -> (String, String) {
    let bate = visiveis.iter().filter(|&&b| m.collides(n, b)).count();
    let passa = visiveis.len() - bate;
    (
        tr_with(
            "card.physics.tab_layer_title",
            &[
                ("name", &nome_inteiro(nomes, n)),
                ("n", &objetos.to_string()),
            ],
        ),
        tr_with(
            "card.physics.tab_layer_pairs",
            &[("bump", &bate.to_string()), ("pass", &passa.to_string())],
        ),
    )
}

/// ⭐ A porta do painel é a FACE da matriz no painel: um controlo com as células (um valor, como a
/// grelha de bits do Inspector), não um comando a mais.
pub(crate) fn declara_a_porta() {
    ph2d_editor_core::widget::composto::grupo(
        std::iter::once(i::PHYSICS_MATRIX_OPEN).chain(i::PHYSICS_LAYER_CELL.iter().copied()),
    );
}

/// A cor da aba: em cartões, o tom da lista (como o editor de animações); `PH2D_CARDS=0`, o de antes.
/// Macro: o tipo da cor é da `ph2d-vector`, que este painel não nomeia.
macro_rules! tom {
    ($t:expr, $theme:expr $(,)?) => {{
        let t: ColorToken = $t;
        let cartao = match t {
            ColorToken::PanelBg => Some(ColorToken::KanbanList),
            t if t == ph2d_editor_core::widget::section_cards::CardDepth::Section.token() => {
                Some(ColorToken::KanbanCard)
            }
            ColorToken::Bg2 => Some(ColorToken::CardField),
            ColorToken::Bg3 => Some(ColorToken::KanbanCardRaised),
            ColorToken::Text1 => Some(ColorToken::KanbanCardFg),
            ColorToken::Text2 | ColorToken::Text3 => Some(ColorToken::CardLabel),
            _ => None,
        };
        match cartao.filter(|_| ph2d_editor_core::panel::cards::ferramenta::cards_on()) {
            Some(c) => ph2d_editor_core::widget::card_ink::CardInk::of_list($theme)
                .color(c)
                .with_alpha(1.0),
            None => ph2d_editor_core::paint::resolve(t, $theme),
        }
    }};
}

mod barras;
mod camada;
pub mod familia;
mod geometria;
mod pinta;
mod quadro;
mod remove;

pub use geometria::Geometria;
pub(crate) use pinta::paint;
pub(crate) use remove::fecha_a_lista;
pub use remove::{fundo_da_pergunta, tinta_do_destino, tinta_do_rotulo};
