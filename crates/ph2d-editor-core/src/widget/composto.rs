//! ⭐⭐⭐ **O CENSO DOS CONTROLOS COMPOSTOS — e o número que ele desmente.**
//!
//! # ⛔⛔⛔ O defeito, medido em 2026-09-21
//!
//! O censo das entradas por painel ([`quantas_entradas_tem_cada_painel`]) classifica pelo
//! SUBSTRATO: um [`InteractiveState::Button`] é um **comando**, e a `D2` do dono manda triar
//! comandos por âmbito (*comando do app → barra; comando do editor → chip da fila; propriedade →
//! fica*). Ele pôs o Inspector no topo da dívida com **`314` comandos**.
//!
//! ⚠️⚠️ **`314` não são `314` comandos.** Olhados um a um, os `60` do bloco base do Inspector são:
//!
//! | o que são | quantos |
//! |---|---:|
//! | `insp_vis_layer_bit_0..31` — as **32 camadas de colisão**, que são UMA grelha de bits | `32` |
//! | `insp_phys_join_kind_*` — **uma escolha** entre 9 tipos de junta | `9` |
//! | `insp_vis_mask_*` · `insp_vis_clip_*` · `insp_order_sp_*` · `insp_render_*` — selectores | `~14` |
//! | comandos a sério (`transform_reset`, `join_draw`, `rig`, `corner_equalize`, `on_screen`) | `5` |
//!
//! ⇒ *um selector de N opções entra na dívida N vezes, e uma máscara de 32 bits entra 32.* O
//! painel que mais usa selectores lidera a lista **por causa disso** — e o Inspector, que é um
//! painel de PROPRIEDADES, é exactamente esse. **A régua mandava a wave para o sítio errado.**
//!
//! ⭐ **E o produto está CERTO**: as 32 camadas são pintadas por um widget só
//! ([`super::BitmaskGrid32`]), com o valor vindo do documento; os 32 ids são **alvos de toque** de
//! um controlo. *Quem mente é o censo, não o painel* — e a cura é a régua aprender a pergunta, não
//! o painel mudar de forma.
//!
//! # ⛔ A regra BARATA foi tentada e FALHA nos dois sentidos
//!
//! *«um id declarado dentro de um ARRAY é uma célula; um `const` escalar é um comando»* — medido:
//! `INSP_ORDER_SP_CENTER`/`_PIVOT`/`_CUSTOM` são **três escalares** que formam um selector, e
//! `INSP_INSTANCE_DROP_ORPHAN: [NodeId; N]` é um **array que é uma lista** de botões distintos.
//! ⇒ a fonte não sabe responder: **quem sabe é QUEM PINTA**.
//!
//! # ⭐⭐ O molde é o do censo de elisões
//!
//! Armado, os pintores canónicos de composto declaram o grupo; desarmado, não custa nada. ⚠️ **A
//! bandeira é da THREAD** pela mesma razão que a do [`crate::text_elide::elisao`]: sob `cargo
//! test` os testes correm em threads do mesmo processo, e uma bandeira global faria o `desarma` de
//! um gate apanhar o vizinho — que leria **zero**, *que é a cara da aprovação*.

use std::cell::{Cell, RefCell};

use ph2d_a11y::NodeId;

thread_local! {
    static ARMADO: Cell<bool> = const { Cell::new(false) };
    static GRUPOS: RefCell<Vec<Vec<NodeId>>> = const { RefCell::new(Vec::new()) };
}

/// Arma o censo e ESVAZIA o que houvesse — um gate que não esvaziasse mediria o vizinho.
pub fn arma() {
    ARMADO.set(true);
    GRUPOS.with_borrow_mut(Vec::clear);
}

/// Desarma. O par mora numa função só, a [`medindo`].
pub fn desarma() {
    ARMADO.set(false);
}

/// Os grupos declarados desde o [`arma`] — cada um é **um** controlo.
#[must_use]
pub fn grupos() -> Vec<Vec<NodeId>> {
    GRUPOS.with_borrow(Clone::clone)
}

/// ⭐⭐ **A PORTA de um gate: arma, corre, desarma, devolve.**
pub fn medindo<R>(f: impl FnOnce() -> R) -> (R, Vec<Vec<NodeId>>) {
    arma();
    let r = f();
    let out = grupos();
    desarma();
    (r, out)
}

/// ⭐ **Um pintor de composto declara aqui as células dele.**
///
/// ⚠️⚠️ **`pub` e não `pub(crate)`, e a razão é medida:** os pintores canónicos moram nesta crate,
/// mas **nem todo selector do app passa por eles**. Medido em 2026-09-21 com o
/// `diag_compostos_por_declarar`: sobram **`307` botões em fileira** por declarar em 16 painéis, e
/// no Inspector **`16` sítios** de quatro secções passam por UM helper local
/// (`sections::tween_editor::grupo`), que reflui as opções em blocos. *Uma porta que só a fundação
/// pode chamar deixa de fora exactamente os painéis que a régua existe para medir.*
///
/// ⛔ **Quem chama isto declara uma ESCOLHA** (*«uma de N»*), nunca uma fileira de comandos
/// distintos: `Add`+`Remove` lado a lado são **dois** comandos e têm de continuar a contar dois.
///
/// ⚠️ **Desarmado isto é um `Cell::get` e um `return`** — o caminho do produto não paga uma
/// alocação. É a metade que o censo de elisões já pagou para aprender: *um censo sempre ligado
/// aloca por quadro, e o vazamento é o que o `leak_key` do `ph2d-i18n` custou a esta casa.*
///
/// ⛔ Um grupo de **uma** célula não é um composto: ele entra na mesma, e quem decide o que fazer
/// com ele é o leitor — *filtrar aqui esconderia do censo a diferença entre «um selector de uma
/// opção» e «um botão solto», que é precisamente o que ele existe para ver.*
pub fn grupo(ids: impl IntoIterator<Item = NodeId>) {
    if !ARMADO.get() {
        return;
    }
    let v: Vec<NodeId> = ids.into_iter().collect();
    if !v.is_empty() {
        GRUPOS.with_borrow_mut(|g| g.push(v));
    }
}
