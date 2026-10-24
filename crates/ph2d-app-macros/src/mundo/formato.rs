//! **O que vai dentro da trilha «mundo»** (`docs/Macros/17_W19_plano.md` §2.2–2.3, §2.7): o núcleo guarda-o
//! como bytes; aqui são postcard, com a versão à frente.
//!
//! Os objectos têm id LOCAL à macro (`0` = o objecto Macro; os outros pela 1.ª aparição) e o
//! `StableId` que tinham na gravação (`original`) — as referências por identidade dentro dos blobs
//! falam desse, e o aplicar remapeia-as pelo censo `RefKind::Object`.

use serde::{Deserialize, Serialize};

pub const FAMILIA: &str = ph2d_macro::codec::FAMILIA_MUNDO;
/// A versão dos bytes desta família (dentro do `exige "mundo"` do núcleo).
pub const VERSAO: u16 = 1;
/// O objecto Macro.
pub const RAIZ: u32 = 0;

/// Quem faz nascer um objecto local ao aplicar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Quem {
    /// O objecto Macro (o `spawn_root` do aplicar).
    Macro,
    /// O objecto da trilha `k` de outra família (a imagem do Painter, o objecto Motion): ela nasce-o.
    Trilha(u32),
    /// Esta família: as linhas dele bastam.
    Mundo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Objeto {
    pub id: u32,
    /// O `StableId` na gravação.
    pub original: u64,
    pub quem: Quem,
}

/// Um componente pelo nome canónico do registo e os bytes dele.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Componente {
    pub nome: String,
    pub bytes: Vec<u8>,
}

/// Um objecto inteiro: o pai (local) e os componentes que a família grava.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Linha {
    pub id: u32,
    /// `None` = o objecto Macro (a raiz da sub-árvore).
    pub pai: Option<u32>,
    pub componentes: Vec<Componente>,
}

/// Os bytes de um recurso que um componente gravado referencia (`SpritePixels`), pelo id de conteúdo.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recurso {
    pub id: [u8; 32],
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mudanca {
    Nasce(Linha),
    Morre(u32),
    Muda {
        id: u32,
        /// `Some(p)` = o pai passou a `p` (`None` dentro = o objecto Macro).
        pai: Option<Option<u32>>,
        /// `None` = tirado.
        componentes: Vec<(String, Option<Vec<u8>>)>,
    },
}

/// **Simular** (§2.3): o relógio da física correu do tique `de` ao `ate` (o da ponte: a corrida nasce no
/// tique 0 da pose autorada, ADR-0131); a reprodução recoze `0..=ate` no rapier e mostra de `de` em
/// diante. Leva a entrada do jogador e a pose que o artista viu no fim de cada corpo (conferência, nunca
/// verdade).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Simula {
    pub de: u32,
    pub ate: u32,
    /// A fita dos tiques `0..=ate`.
    pub fita: ph2d_physics_ecs::TapeWire,
    /// `(id local, [x, y, rotação])` no último tique.
    pub conferencia: Vec<(u32, [f32; 3])>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Evento {
    Passo {
        /// Os objectos que aparecem pela 1.ª vez neste passo.
        novos: Vec<Objeto>,
        mudancas: Vec<Mudanca>,
        recursos: Vec<Recurso>,
    },
    Simula(Simula),
}

/// O estado da sub-árvore ao Gravar.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inicial {
    pub objetos: Vec<Objeto>,
    pub linhas: Vec<Linha>,
    pub recursos: Vec<Recurso>,
}

/// Porque uns bytes desta família não se leem.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErroDeLeitura {
    /// Uma versão mais nova que esta.
    Versao(u16),
    Bytes,
}

fn escreve<T: Serialize>(v: &T) -> Vec<u8> {
    let mut out = VERSAO.to_le_bytes().to_vec();
    out.extend(postcard::to_allocvec(v).expect("postcard de tipos próprios não falha"));
    out
}

fn le<'a, T: Deserialize<'a>>(b: &'a [u8]) -> Result<T, ErroDeLeitura> {
    let (v, resto) = b.split_at_checked(2).ok_or(ErroDeLeitura::Bytes)?;
    let v = u16::from_le_bytes([v[0], v[1]]);
    if v > VERSAO {
        return Err(ErroDeLeitura::Versao(v));
    }
    postcard::from_bytes(resto).map_err(|_| ErroDeLeitura::Bytes)
}

impl Inicial {
    #[must_use]
    pub fn escreve(&self) -> Vec<u8> {
        escreve(self)
    }

    /// # Errors
    /// [`ErroDeLeitura`].
    pub fn le(b: &[u8]) -> Result<Self, ErroDeLeitura> {
        le(b)
    }
}

impl Evento {
    #[must_use]
    pub fn escreve(&self) -> Vec<u8> {
        escreve(self)
    }

    /// # Errors
    /// [`ErroDeLeitura`].
    pub fn le(b: &[u8]) -> Result<Self, ErroDeLeitura> {
        le(b)
    }
}
