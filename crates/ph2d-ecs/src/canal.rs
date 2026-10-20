//! ⭐⭐⭐ **O canal de QUALQUER propriedade** — o endereço com que a timeline anima um campo que não é
//! um dos 19 `PropKind` de sempre (ordem do dono, 19/10: *«o auto key deve gravar absolutamente tudo»*;
//! [plano 35 §3.0](../../../docs/UI_New_and_Simple/vetor_nova_ui/35_plano_F24_o_auto_key_grava_tudo.md)).
//!
//! A timeline **não conhece família nenhuma** (ADR-0075): ela lê o que cada família PUBLICA em
//! [`CanaisLidos`] — o valor que o documento dela tem agora — e publica em [`CanaisEscritos`] o que a
//! curva manda escrever neste quadro; cada família drena o seu. Um sistema nunca chama o outro.
//!
//! Tudo é `f32`: uma cor são quatro campos do mesmo [`CampoInfo::grupo`], um vértice são oito, um
//! bool/enum/escolha é o índice com `Hold` forçado na chave ([`CampoInfo::discreto`]).

use std::collections::BTreeMap;

use bevy_ecs::resource::Resource;
use serde::{Deserialize, Serialize};

/// O endereço DURÁVEL de um campo animável: `familia` e `campo` são números DECLARADOS (fio
/// congelado só por APPEND, como os discriminantes do `PropKind`); `i`/`j` dizem QUAL — contorno e
/// vértice, entrada da pilha e paragem, efeito e parâmetro (o sentido é do vocabulário da família).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FieldKey {
    pub familia: u16,
    pub campo: u16,
    pub i: u16,
    pub j: u16,
}

impl FieldKey {
    #[must_use]
    pub const fn new(familia: u16, campo: u16, i: u16, j: u16) -> Self {
        Self {
            familia,
            campo,
            i,
            j,
        }
    }

    const MIN: FieldKey = FieldKey::new(0, 0, 0, 0);
    const MAX: FieldKey = FieldKey::new(u16::MAX, u16::MAX, u16::MAX, u16::MAX);

    /// O que este campo É — a porta única ([`info`]).
    #[must_use]
    pub fn info(self) -> Option<CampoInfo> {
        info(self)
    }

    /// O endereço do GRUPO deste campo — `(familia, grupo, i)`, com `i = 0` quando o grupo não se
    /// parte por `i`. A porta única de «que campos se gravam juntos»: o autokey ([`Self::mesmo_grupo`])
    /// e a linha de grupo do dope-sheet perguntam aqui.
    #[must_use]
    pub fn grupo(self) -> Option<(u16, u16, u16)> {
        self.info()
            .map(|c| (self.familia, c.grupo, if c.por_i { self.i } else { 0 }))
    }

    /// `true` se `outro` grava-se na MESMA chave que este (a forma inteira, a cor inteira).
    #[must_use]
    pub fn mesmo_grupo(self, outro: FieldKey) -> bool {
        match (self.grupo(), outro.grupo()) {
            (Some(a), Some(b)) => a == b,
            _ => self == outro,
        }
    }

    /// Chave `ph2d-i18n` do nome deste campo DENTRO do grupo («Point X», «R», «Dash») — a linha do
    /// membro quando o artista abre o grupo. `None` = endereço que nenhuma família declara.
    #[must_use]
    pub fn membro(self) -> Option<&'static str> {
        match self.familia {
            familia::VETOR => crate::canal_vetor::membro(self.campo),
            _ => None,
        }
    }
}

/// O que um campo é, para quem não é a família dele (a timeline, o dope-sheet, o losango).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CampoInfo {
    /// Chave `ph2d-i18n` do rótulo do GRUPO (o que o artista lê na linha do dope-sheet).
    pub rotulo: &'static str,
    /// `true` ⇒ a chave é sempre `Hold` e o escritor arredonda ao índice válido.
    pub discreto: bool,
    /// Os membros do mesmo grupo gravam-se juntos, no mesmo instante.
    pub grupo: u16,
    /// `true` ⇒ o grupo parte-se por `i` (cada entrada da pilha tem a sua tinta); `false` ⇒ todo `i`
    /// é o mesmo grupo (a forma inteira são todos os contornos).
    pub por_i: bool,
}

/// As famílias. ⛔ Números DECLARADOS e gravados no documento da timeline: só APPEND.
pub mod familia {
    /// O documento vetorial (`VecScene` e o `VecShape` da entidade) — `crate::canal_vetor`.
    pub const VETOR: u16 = 1;
}

/// ⭐ **A porta única de «o que é este campo»** — `None` = endereço que nenhuma família declara (um
/// documento gravado por uma build com uma família a mais).
#[must_use]
pub fn info(k: FieldKey) -> Option<CampoInfo> {
    match k.familia {
        familia::VETOR => crate::canal_vetor::info(k.campo),
        _ => None,
    }
}

/// **O que o documento de cada família vale AGORA**, por `(entidade, campo)` — publicado pela família
/// antes de quem o lê (o autokey, o `rest` da timeline, o losango do painel).
///
/// ⭐⭐ **O ECO** — o documento de uma família QUANTIZA (uma cor é `u8`, um parâmetro inteiro
/// arredonda, um limite prende): a curva escreve `127,5`, o documento fica `128`, e quem compara o
/// documento com a curva veria o artista a mexer em todo quadro a meio de uma interpolação (a chave
/// fantasma). Quem escreve regista [`Self::eco`] — o que a curva mandou e o que o documento ficou —
/// e [`Self::publicar`] devolve o valor da CURVA enquanto o documento ainda vale o que ficou. Só
/// uma mudança real do documento (a mão) aparece como diferença.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct CanaisLidos {
    valores: BTreeMap<(u64, FieldKey), f32>,
    ecos: BTreeMap<(u64, FieldKey), (f32, f32)>,
}

impl CanaisLidos {
    /// Publica o valor que o documento tem (`doc`), lido através do eco da última escrita.
    pub fn publicar(&mut self, entidade: u64, k: FieldKey, doc: f32) {
        let v = match self.ecos.get(&(entidade, k)) {
            Some(&(escrito, ficou)) if ficou == doc => escrito,
            _ => doc,
        };
        self.valores.insert((entidade, k), v);
    }

    /// Regista que a curva escreveu `escrito` e o documento ficou `ficou` (ver o cabeçalho).
    pub fn eco(&mut self, entidade: u64, k: FieldKey, escrito: f32, ficou: f32) {
        self.ecos.insert((entidade, k), (escrito, ficou));
    }

    /// Esquece tudo o que a entidade publicou (antes de a republicar: um vértice apagado sai).
    pub fn esquecer(&mut self, entidade: u64) {
        let dela: Vec<_> = self
            .valores
            .range((entidade, FieldKey::MIN)..=(entidade, FieldKey::MAX))
            .map(|(k, _)| *k)
            .collect();
        for k in dela {
            self.valores.remove(&k);
        }
    }

    #[must_use]
    pub fn valor(&self, entidade: u64, k: FieldKey) -> Option<f32> {
        self.valores.get(&(entidade, k)).copied()
    }

    /// Os campos que a entidade publicou, em ordem de endereço.
    pub fn campos(&self, entidade: u64) -> impl Iterator<Item = (FieldKey, f32)> + '_ {
        self.valores
            .range((entidade, FieldKey::MIN)..=(entidade, FieldKey::MAX))
            .map(|(&(_, k), &v)| (k, v))
    }

    pub fn limpar(&mut self) {
        self.valores.clear();
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.valores.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.valores.is_empty()
    }
}

/// **O que a curva manda escrever neste quadro** — publicado pelo apply da timeline, drenado pela
/// família dona do campo.
#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct CanaisEscritos {
    valores: Vec<(u64, FieldKey, f32)>,
}

impl CanaisEscritos {
    pub fn escrever(&mut self, entidade: u64, k: FieldKey, v: f32) {
        self.valores.push((entidade, k, v));
    }

    /// Tira as escritas de UMA família, deixando as outras para quem as drena.
    pub fn drenar(&mut self, familia: u16) -> Vec<(u64, FieldKey, f32)> {
        let (dela, resto) = std::mem::take(&mut self.valores)
            .into_iter()
            .partition(|(_, k, _)| k.familia == familia);
        self.valores = resto;
        dela
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.valores.is_empty()
    }
}

#[cfg(test)]
#[path = "canal_tests.rs"]
mod tests;
