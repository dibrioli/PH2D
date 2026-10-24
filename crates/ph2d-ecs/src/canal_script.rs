//! **A família SCRIPT** — o canal de uma propriedade que o SCRIPT declara (`ph2d.property`), que
//! não está no catálogo: nasce da declaração e endereça-se pelo NOME (o contrato do `own`).
//!
//! A chave: `campo = tipo << 2 | faixa`, `i:j` = o índice do NOME na [`TextosDosCanais`] (a mesma
//! tabela dos valores de texto: só CRESCE na sessão; o documento da timeline compacta-a e remapeia
//! estas chaves ao gravar e ao abrir). Zero colisões, e o nome lê-se sempre — também o de uma
//! propriedade que saiu do script. Desenho e recusas: `docs/UI_New_and_Simple/F27_E_plano.md`.

use crate::canal::familia::SCRIPT;
use crate::canal::{CampoInfo, FieldKey, TextosDosCanais};

/// O tipo de uma propriedade de script — o espelho do `ph2d_script::ScriptValue`. ⛔ Números
/// DECLARADOS (gravados na chave): só APPEND.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Tipo {
    Number = 1,
    Bool = 2,
    Text = 3,
    Vec2 = 4,
    Color = 5,
}

impl Tipo {
    const TODOS: [Tipo; 5] = [
        Tipo::Number,
        Tipo::Bool,
        Tipo::Text,
        Tipo::Vec2,
        Tipo::Color,
    ];

    fn de(campo: u16) -> Option<Self> {
        Self::TODOS.into_iter().find(|t| *t as u16 == campo >> 2)
    }

    /// Quantas faixas o tipo tem.
    #[must_use]
    pub const fn faixas(self) -> u16 {
        match self {
            Self::Vec2 => 2,
            Self::Color => 4,
            Self::Number | Self::Bool | Self::Text => 1,
        }
    }
}

/// A chave da faixa `faixa` da propriedade `nome`; o nome entra em `textos` se é novo.
pub fn chave(nome: &str, tipo: Tipo, faixa: u16, textos: &mut TextosDosCanais) -> Option<FieldKey> {
    if faixa >= tipo.faixas() {
        return None;
    }
    let n = indice(textos.indice(Some(nome)));
    #[expect(clippy::cast_possible_truncation, reason = "as duas metades do índice")]
    let (i, j) = ((n >> 16) as u16, n as u16);
    Some(FieldKey::new(SCRIPT, (tipo as u16) << 2 | faixa, i, j))
}

/// A chave da faixa 0 de `nome` se a tabela já o tem — sem a mudar (o instantâneo do painel; o
/// funil interna cada nome ao publicar).
#[must_use]
pub fn chave_lida(nome: &str, tipo: Tipo, textos: &TextosDosCanais) -> Option<FieldKey> {
    let n = indice(textos.procura(nome)?);
    #[expect(clippy::cast_possible_truncation, reason = "as duas metades do índice")]
    let (i, j) = ((n >> 16) as u16, n as u16);
    Some(FieldKey::new(SCRIPT, (tipo as u16) << 2, i, j))
}

/// As chaves do GRUPO de `k` (todas as faixas do mesmo nome e tipo).
#[must_use]
pub fn grupo(k: FieldKey) -> Vec<FieldKey> {
    let Some((t, _)) = tipo(k) else {
        return Vec::new();
    };
    let base = k.campo & !3;
    (0..t.faixas())
        .map(|f| FieldKey::new(SCRIPT, base | f, k.i, k.j))
        .collect()
}

/// As chaves de todas as faixas da propriedade.
pub fn chaves(nome: &str, tipo: Tipo, textos: &mut TextosDosCanais) -> Vec<FieldKey> {
    (0..tipo.faixas())
        .filter_map(|f| chave(nome, tipo, f, textos))
        .collect()
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "o índice da tabela, exacto em f32"
)]
fn indice(v: f32) -> u32 {
    v as u32
}

/// O valor (`f32`) da tabela que a chave nomeia — o que [`TextosDosCanais::texto`] lê.
#[must_use]
pub fn valor_do_nome(k: FieldKey) -> f32 {
    #[expect(clippy::cast_precision_loss, reason = "preso a 2^24 pela tabela")]
    let v = (u32::from(k.i) << 16 | u32::from(k.j)) as f32;
    v
}

/// O nome da propriedade que a chave endereça (`None` = outra família, ou um índice que `textos`
/// não tem).
#[must_use]
pub fn nome(k: FieldKey, textos: &TextosDosCanais) -> Option<&str> {
    (k.familia == SCRIPT).then(|| textos.texto(valor_do_nome(k)))?
}

/// A mesma chave a nomear `para` (o remapeamento da tabela ao gravar e ao abrir).
#[must_use]
pub fn com_nome(k: FieldKey, para: f32) -> FieldKey {
    let n = indice(para);
    #[expect(clippy::cast_possible_truncation, reason = "as duas metades do índice")]
    FieldKey::new(k.familia, k.campo, (n >> 16) as u16, n as u16)
}

/// O tipo e a faixa da chave.
#[must_use]
pub fn tipo(k: FieldKey) -> Option<(Tipo, u16)> {
    let t = Tipo::de(k.campo)?;
    let faixa = k.campo & 3;
    (k.familia == SCRIPT && faixa < t.faixas()).then_some((t, faixa))
}

#[must_use]
pub fn info(k: FieldKey) -> Option<CampoInfo> {
    let (t, _) = tipo(k)?;
    Some(CampoInfo {
        rotulo: "canal.script.propriedade",
        discreto: matches!(t, Tipo::Bool | Tipo::Text),
        grupo: k.j,
        por_i: true,
        texto: t == Tipo::Text,
    })
}

#[must_use]
pub fn membro(k: FieldKey) -> Option<&'static str> {
    const XY: [&str; 2] = ["canal.m.x", "canal.m.y"];
    const RGBA: [&str; 4] = ["canal.m.r", "canal.m.g", "canal.m.b", "canal.m.a"];
    let (t, faixa) = tipo(k)?;
    let faixa = usize::from(faixa);
    match t {
        Tipo::Vec2 => XY.get(faixa).copied(),
        Tipo::Color => RGBA.get(faixa).copied(),
        Tipo::Number | Tipo::Bool | Tipo::Text => Some("canal.script.propriedade"),
    }
}

#[cfg(test)]
#[path = "canal_script_tests.rs"]
mod tests;
