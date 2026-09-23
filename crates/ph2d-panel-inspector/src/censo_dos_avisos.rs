//! ⭐⭐⭐ **O CENSO DOS AVISOS — que frase foi PINTADA, e de onde.**
//!
//! ⛔⛔ **Ele nasce de um report do dono que nenhum instrumento desta casa sabia responder**
//! (2026-09-21): *«vários componentes cheios de mensagens»*. Havia censo de texto (HR-15), censo
//! de elisões (*«coube?»*), censo de ids e gate de costura — e **nenhum** perguntava
//! ***quantas frases o painel escreve de uma vez, e quantas dizem a mesma coisa***.
//!
//! ⚠️ **O censo da elisão não o podia responder, e a razão é a LEI dele**: um aviso é uma FRASE e
//! ela QUEBRA em vez de cortar, logo nunca passa pela lei da reticência e **não deixa rasto**
//! (é exactamente o que o `um_aviso_quebra_e_o_pintor_de_rotulo_corta` afirma). *Um censo cego
//! àquilo de que o dono se queixa lê-se como um painel limpo.*
//!
//! ⚠️ **A LOCALIZAÇÃO é metade do valor.** Uma régua que diz *«esta frase saiu 21 vezes»* e não
//! diz **de onde** obriga a arqueologia — a lição que o censo das elisões já pagou, e que as três
//! réguas da ponta do Sculpt pagaram antes dela (mediam ápice a ápice e deitavam fora o ÍNDICE).
//!
//! # ⚠️ Ele nasce DESARMADO, e isso não é conforto
//!
//! O caminho do produto paga **a leitura de uma bandeira da própria thread**, e mais nada — sem
//! isso seria uma `String` por aviso **por quadro**.

use std::cell::{Cell, RefCell};

/// ⭐ **Um aviso que foi pintado** — o que dizia e quem o mandou pintar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Aviso {
    /// A frase, tal como o pintor a recebeu.
    pub texto: String,
    /// O ficheiro do CHAMADOR — a secção que a escreveu.
    pub ficheiro: &'static str,
    /// A linha do chamador.
    pub linha: u32,
}

impl Aviso {
    /// O nome da secção — o `basename` sem extensão do ficheiro que o pintou.
    #[must_use]
    pub fn seccao(&self) -> &str {
        self.ficheiro
            .rsplit('/')
            .next()
            .unwrap_or(self.ficheiro)
            .strip_suffix(".rs")
            .unwrap_or(self.ficheiro)
    }
}

thread_local! {
    /// ⚠️ **Por THREAD e nunca um átomo global**: a suíte corre os gates em paralelo no mesmo
    /// processo, e um contador partilhado mediria o vizinho. (A mesma lição que o contador de
    /// cozimentos do Motion pagou por reprovar na suíte enquanto passava sozinho.)
    static ARMADO: Cell<bool> = const { Cell::new(false) };
    static PINTADOS: RefCell<Vec<Aviso>> = const { RefCell::new(Vec::new()) };
}

/// Arma o censo e ESVAZIA o que houvesse — um gate que não esvaziasse mediria o vizinho.
pub fn arma() {
    ARMADO.set(true);
    PINTADOS.with_borrow_mut(Vec::clear);
}

/// Desarma.
pub fn desarma() {
    ARMADO.set(false);
}

/// Tudo o que foi pintado desde o [`arma`].
#[must_use]
pub fn pintados() -> Vec<Aviso> {
    PINTADOS.with_borrow(Clone::clone)
}

/// ⭐⭐ **A PORTA de um gate: arma, corre, desarma, devolve** — o que ela compra é que ninguém
/// escreva `arma` sem o `desarma`.
pub fn medindo<R>(f: impl FnOnce() -> R) -> (R, Vec<Aviso>) {
    arma();
    let r = f();
    let out = pintados();
    desarma();
    (r, out)
}

/// ⚠️ **Lê o `Location` AQUI e não dentro do fecho:** um fecho é outra função e o
/// `#[track_caller]` não atravessa a fronteira dele — a lição que o censo da elisão pagou.
#[track_caller]
pub(crate) fn regista(texto: &str) {
    if !ARMADO.get() || texto.is_empty() {
        return;
    }
    let onde = core::panic::Location::caller();
    PINTADOS.with_borrow_mut(|v| {
        v.push(Aviso {
            texto: texto.to_string(),
            ficheiro: onde.file(),
            linha: onde.line(),
        });
    });
}
