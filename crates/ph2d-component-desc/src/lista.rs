//! ⭐ **O ITEM de cada lista de componente** (F30 §A1, `docs/UI_New_and_Simple/F30_plano.md` §A1) — o
//! que a família de canais `LISTA` anima: um timer, um tween, um estado, uma seta, uma regra…
//!
//! A chave de um campo de item é `[FAMILIA, codigo << 10 | field_id << 2 | faixa, item >> 16,
//! item & 0xFFFF]` (`item` = o `ItemId`, nunca a posição). O `codigo` é DECLARADO e append-only, um
//! por lista; o `field_id` é o do item. A tabela é da folha para o painel calcular a chave da fileira
//! que pinta, sem ver o ECS.

use crate::FieldDesc;

/// A família de canais dos itens (`ph2d_ecs::canal::familia::LISTA` é ESTE número).
pub const FAMILIA: u16 = 10;

/// O maior código de lista (6 bits).
pub const MAX_CODIGO: u16 = 63;
/// O maior `field_id` de um campo de item (8 bits).
pub const MAX_CAMPO: u16 = 255;

/// ⛔ Os códigos das listas: gravados em cada chave de animação — só APPEND, nunca renumerar.
pub mod codigo {
    pub const TIMERS: u16 = 1;
    pub const TWEENS: u16 = 2;
    pub const ESTADOS: u16 = 3;
    pub const SETAS: u16 = 4;
    pub const VIGIA: u16 = 5;
    pub const ACOES: u16 = 6;
    pub const GATILHOS: u16 = 7;
    pub const ABANAO: u16 = 8;
    pub const RESISTENCIAS: u16 = 9;
    pub const ANCORAS: u16 = 10;
}

/// **Uma lista de um componente** e o item dela.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ListaDesc {
    /// [`codigo`].
    pub codigo: u16,
    /// O nome canónico do componente que a tem.
    pub componente: &'static str,
    /// A chave serde da lista no valor do componente ([`crate::WHOLE`] = o componente é a lista).
    pub caminho: &'static str,
    /// Os campos de UM item, com a chave serde de cada um (dentro do item).
    pub campos: &'static [FieldDesc],
    /// `(field_id, codigo)`: o campo guarda a POSIÇÃO de um item da lista `codigo` do MESMO
    /// componente (a seta aponta um estado). O canal dele publica o `ItemId` desse item, nunca a
    /// posição — apagar um item anterior não envelhece a chave (F39, F32 §6.3).
    pub aponta: &'static [(u16, u16)],
}

impl ListaDesc {
    /// O campo do item com este `field_id`.
    #[must_use]
    pub fn campo(&self, field_id: u16) -> Option<&'static FieldDesc> {
        self.campos.iter().find(|f| f.field_id == field_id)
    }

    /// A lista para cujo item o campo `field_id` aponta ([`Self::aponta`]).
    #[must_use]
    pub fn alvo(&self, field_id: u16) -> Option<&'static ListaDesc> {
        let &(_, c) = self.aponta.iter().find(|(f, _)| *f == field_id)?;
        lista(c)
    }
}

/// Todas as listas do catálogo, por código.
#[must_use]
pub fn todas() -> &'static [ListaDesc] {
    crate::catalog::listas::LISTAS
}

/// A lista com este código.
#[must_use]
pub fn lista(codigo: u16) -> Option<&'static ListaDesc> {
    todas().iter().find(|l| l.codigo == codigo)
}

/// As listas do componente `canonical`.
pub fn do_componente(canonical: &str) -> impl Iterator<Item = &'static ListaDesc> + '_ {
    todas().iter().filter(move |l| l.componente == canonical)
}

/// O `campo` da chave: `codigo << 10 | field_id << 2 | faixa`.
#[must_use]
pub const fn campo(codigo: u16, field_id: u16, faixa: u16) -> Option<u16> {
    if codigo == 0 || codigo > MAX_CODIGO || field_id > MAX_CAMPO || faixa > 3 {
        return None;
    }
    Some(codigo << 10 | field_id << 2 | faixa)
}

/// `(codigo, field_id, faixa)` de um `campo`.
#[must_use]
pub const fn decompor(campo: u16) -> (u16, u16, u16) {
    (campo >> 10, (campo >> 2) & MAX_CAMPO, campo & 3)
}

/// ⭐ **As chaves (todas as faixas) do campo `field_id` do item `item` da lista `codigo`** — o que a
/// fileira do Inspector regista no losango. Vazio = não é canal.
#[must_use]
pub fn chaves(codigo: u16, field_id: u16, item: u32) -> Vec<[u16; 4]> {
    let Some(f) = lista(codigo).and_then(|l| l.campo(field_id)) else {
        return Vec::new();
    };
    #[expect(clippy::cast_possible_truncation, reason = "as duas metades do id")]
    let (i, j) = ((item >> 16) as u16, item as u16);
    (0..f.faixas_de_canal())
        .filter_map(|x| Some([FAMILIA, campo(codigo, field_id, x)?, i, j]))
        .collect()
}

#[cfg(test)]
#[path = "lista_tests.rs"]
mod tests;
