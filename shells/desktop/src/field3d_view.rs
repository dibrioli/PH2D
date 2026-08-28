//! ⭐ **O ESTADO DE VISTA, e a memória que o faz sobreviver a fechar o painel** (W43).
//!
//! # O ⏸️ que a W42 deixou, com as palavras dela
//!
//! > *"Fica: fechar o painel larga a **câmera** (a peça não)."*
//!
//! A W42 fez o pill desarmar de verdade — e o preço foi imediato: largar o [`Smoke`] larga o cache
//! do quadro (que é o que se quer) **e a vista** (que não é). O artista pousa a peça num ângulo,
//! pega no Vector, volta ao MODEL — e encontra a peça certa vista de outro sítio, a girar sozinha.
//!
//! # ⭐ A categoria já estava escrita — em TRÊS sítios, e ninguém lhe tinha dado nome
//!
//! Não é «a câmera». O próprio [`Smoke`] classifica os campos, um a um, e diz a mesma coisa três
//! vezes:
//!
//! | campo | o que o doc dele já dizia |
//! |---|---|
//! | `gizmo_mode` | *"É estado de **vista**, e não do documento: por isso vive aqui e não num componente"* |
//! | `gizmo_frame` | *"Estado de **vista**, como o verbo"* |
//! | `isolated` | *"Estado de VISTA, e a lei é a do módulo irmão"* |
//!
//! …mais a `cam`, que **é** a vista, e o `manual` (*"o prato para de girar assim que o artista toca
//! nele"*). São **cinco** campos com o mesmo tempo de vida, e o que faltava era dizê-lo uma vez em
//! vez de cinco. ⚠️ *Um doc-comment repetido em N campos é uma estrutura por nascer.*
//!
//! # ⚠️ Por que o `manual` viaja com a câmera, e não é um extra
//!
//! Restaurar a câmera **sem** o `manual` não restaura nada: o prato volta a girar e afasta-se do
//! ângulo restaurado a partir do quadro seguinte. O número certo estaria lá durante 16 ms, e o
//! defeito leria como *"restaurar a câmera não funciona"*. *Duas metades de um mesmo fato.*
//!
//! # ⭐⭐ Como um campo novo NÃO se perde: o destructuring sem `..`
//!
//! [`View::of`] desmonta o [`Smoke`] **nomeando os 27 campos**, sem `..`. Um campo novo no `Smoke`
//! passa a ser **erro de compilação aqui** — e o erro cai exatamente no sítio onde a pergunta tem de
//! ser respondida: *este campo é vista, ou é cache do quadro?*
//!
//! ⚠️ Esta é a lição do `Shade::default()` da `line/sculpt3d`, generalizada. Lá, uma sonda tirava a
//! vista de um `default()` e um termo novo entrou em silêncio; a cura foi escrevê-la *"por nome, os
//! 7 campos"*. Aqui a escrita por nome é a **entrada**, e não a saída — que é o lado que apanha
//! quem acrescenta, e não quem lê.
//!
//! # ⚠️ O que NÃO viaja
//!
//! Tudo o resto: o quadro pronto, o traçado em voo, o pedido em cache, a área desenhada, a âncora
//! do gizmo, o gesto em curso (`drag`, `drag_grip`, `typed`, `press_at`, `pending_pick`,
//! `pending_move`) e o `snapping`. ⚠️ **Um gesto em curso a atravessar um fecho seria pior que
//! perdê-lo**: o painel fecha *porque* outra ferramenta tomou o canvas, isto é, no meio de um gesto
//! que já não é deste módulo. Reabrir com um arrasto pendurado aplicaria à peça um movimento que
//! ninguém fez.

use super::Smoke;
use crate::field3d_gizmo::{Frame, Mode};
use ph2d_field_render::Orbit;

/// **A vista do módulo** — os cinco campos que sobrevivem a fechar o painel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct View {
    /// Onde a câmera está.
    pub(crate) cam: Orbit,
    /// O prato já foi tocado à mão? Ver a nota do módulo — ele viaja com a câmera ou nenhum dos
    /// dois vale.
    pub(crate) manual: bool,
    /// Que verbo o gizmo oferece.
    pub(crate) gizmo_mode: Mode,
    /// Em que referencial os eixos apontam.
    pub(crate) gizmo_frame: Frame,
    /// O nó isolado, se algum.
    pub(crate) isolated: Option<u64>,
    /// ⭐⭐⭐ **Como o canvas estava dividido** (W95) — ver [`crate::field3d_layout::Split`].
    ///
    /// ⛔⛔ **A W90 deixou-a de FORA com uma razão errada:** *«restaurar a divisão obrigaria a
    /// restaurar as quatro câmeras»*. É falso — **três delas são DERIVADAS** (as vistas nomeadas
    /// nascem da orientação que o nome promete, e a [`crate::field3d_smoke::ensure_viewports`] já as
    /// reconstrói a partir da câmera do artista, que é a única autorada). *Uma dependência afirmada
    /// sem a desmontar é uma feature adiada com cara de arquitectura* — a segunda desta wave, depois
    /// da que dizia que o divisor precisava do cabeçalho.
    ///
    /// ⚠️ E ela **pertence** aqui: a divisão é uma preferência de bancada, exactamente como a
    /// câmera. Um artista que trabalha em quatro vistas e pega no editor vetorial não quer voltar e
    /// encontrar uma.
    pub(crate) split: crate::field3d_layout::Split,
}

impl Default for View {
    /// ⚠️ **É esta a definição de «vista nova»**, e o [`super::boot`] usa-a — em vez de repetir os
    /// cinco valores lá. Enquanto fossem dois sítios, a primeira abertura do app e uma reabertura
    /// sem memória podiam divergir sem ninguém notar; há gate.
    fn default() -> Self {
        Self {
            split: crate::field3d_layout::Split::One,
            cam: Orbit::default(),
            manual: false,
            gizmo_mode: Mode::default(),
            gizmo_frame: Frame::default(),
            isolated: None,
        }
    }
}

impl View {
    /// ⭐ **Tira a vista de um [`Smoke`] vivo** — por destructuring e **sem `..`**. Ver a nota do
    /// módulo: é isto que faz um campo novo ser erro de compilação em vez de um estado perdido em
    /// silêncio.
    fn of(s: &Smoke) -> Self {
        let Smoke {
            // ⭐⭐ **A câmera e o prato vivem no VIEWPORT desde a W90** — e a vista que sobrevive a
            // fechar o painel é a do **activo**: guardar N câmeras exigiria também guardar a
            // DIVISÃO, e o que a W43 promete é *«a peça certa vista do sítio onde a deixei»*.
            vps: _,
            active: _,
            split,
            gizmo_mode,
            gizmo_frame,
            isolated,
            // ⚠️ Daqui para baixo, **cache do quadro ou gesto em curso** — nada disto atravessa.
            doc: _,
            seed: _,
            matcap: _,
            announced: _,
            drag: _,
            last_pointer: _,
            gizmo: _,
            gizmo_hot: _,
            pending_move: _,
            drag_grip: _,
            snapping: _,
            typed: _,
            press_at: _,
            pending_pick: _,
            lasso: _,
            pending_lasso: _,
            flight: _,
            flight_gen: _,
            flight_fresh: _,
            safe: _,
            profile_pick: _,
            nav_hot: _,
            nav_press: _,
            has_live_sculpt: _,
            // ⚠️ **Cache, e não vista** — as fitas compiladas de um quadro. Deitá-las fora ao
            // fechar não custa nada: a 1.ª mão a mexer volta a enchê-las.
        } = s;
        Self {
            split: *split,
            cam: s.vp().cam,
            manual: s.vp().manual,
            gizmo_mode: *gizmo_mode,
            gizmo_frame: *gizmo_frame,
            isolated: *isolated,
        }
    }
}

thread_local! {
    /// A vista da última vez que o módulo esteve armado.
    ///
    /// ⚠️ **Ela NÃO é um pedido, e por isso não se tira uma vez** — os irmãos em
    /// [`super::requests`] são eventos (`Cell::take`), e um evento pousado repetir-se-ia. Isto é
    /// *estado lembrado*: fechar e reabrir três vezes tem de devolver a mesma vista as três, e um
    /// `take` daria a vista à primeira e o padrão às outras duas.
    static LAST: std::cell::Cell<Option<View>> = const { std::cell::Cell::new(None) };
}

/// **Guarda a vista de um smoke que está a ser largado.** Chamada do desarme, e só de lá.
pub(super) fn remember(s: &Smoke) {
    LAST.with(|c| c.set(Some(View::of(s))));
}

/// **A vista com que um smoke novo nasce** — a lembrada, ou a padrão.
pub(crate) fn recall() -> View {
    LAST.with(std::cell::Cell::get).unwrap_or_default()
}

/// ⭐ **Um DOCUMENTO novo não herda um isolamento** — e é o único dos cinco campos com este
/// problema.
///
/// ⚠️ O `isolated` guarda os **bits de uma entidade**, e um Ctrl+O respawna o mundo inteiro: os bits
/// do projeto A vão ser realocados no projeto B, a outros nós. Um isolamento herdado ou aponta para
/// nada (o `cook_root` já o larga) ou — pior — **acerta noutro nó**, e a peça nova abre com quase
/// tudo escondido, sem uma palavra a dizer porquê. *É a lei da casa sobre bits dentro de bytes, uma
/// vez mais.*
///
/// ⚠️ **A câmera NÃO se esquece aqui**, de propósito: ela não nomeia nada do documento, e um ângulo
/// herdado é, no pior caso, um enquadramento que o artista ajusta. *Esquecer «por simetria» custaria
/// a metade que a W43 acabou de comprar.*
///
/// Quarta da família em [`crate::project_load`], ao lado de `forget_owed_poses`,
/// `forget_live_producers` e `field3d_reload::forget_tried` — todas respondem *"o que o documento
/// anterior possuía e não pode atravessar"*.
pub(crate) fn forget_isolation_across_documents() {
    LAST.with(|c| {
        if let Some(mut v) = c.get() {
            v.isolated = None;
            c.set(Some(v));
        }
    });
    super::forget_isolation();
}

/// ⚠️ Só para gates: esquece a vista lembrada, para que dois gates no mesmo processo não se
/// contaminem pela ordem em que correram.
#[cfg(test)]
pub(crate) fn forget() {
    LAST.with(|c| c.set(None));
}

#[cfg(test)]
#[path = "field3d_view_tests.rs"]
mod tests;
