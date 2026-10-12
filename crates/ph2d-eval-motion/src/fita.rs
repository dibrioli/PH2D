//! **A fita das entradas** (ciclo 19, [doc 128 §3.3](../../../docs/Motion%20Nodes/128_ciclo_19_o_jogo_manda.md)):
//! o que estava publicado em cada tique cozido para a frente, para o scrub o repetir.
//!
//! O anel de checkpoints guarda o estado do GRAFO; os externals vêm de fora (o jogo, o editor) e
//! não voltam atrás com o editor. Sem a fita, o replay coze cada tique passado com o valor de
//! AGORA (medido: doc 128 §5.1).
//!
//! Só se gravam MUDANÇAS, por revisão: uma entrada parada não custa nada por tique. Por dentro a
//! fita guarda cada mudança COMPACTA — o nome internado, uma coluna de um elemento pelos bits, as
//! maiores como a MESMA alocação do publicado — e só reconstrói o `External` na leitura (doc 131
//! §7, C.11: o `External` clonado retinha `~640 B` por mudança e a régua declarava `~36`; agora
//! `~50`, declarados a `±3 %`). Gravar o tique seguinte parte do cursor; ler um tique qualquer
//! parte da âncora mais próxima — nenhum dos dois refaz a história.

use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::external::{All, External};
use std::collections::BTreeMap;
use std::sync::Arc;

/// O que a gravação de um tique fez.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Gravacao {
    /// O tique já estava gravado com as mesmas revisões (a tocar sobre chão já gravado).
    Igual,
    /// Um tique novo, para lá do último gravado.
    Nova,
    /// O tique já estava gravado com OUTRAS entradas: o futuro velho foi esquecido a partir
    /// daqui — quem guarda estado desse futuro (o anel) tem de o esquecer também.
    Ramo,
}

/// **O orçamento da fita, em bytes de RAM** — o heap que ela retém, com o cabeçalho do `malloc`
/// (o gate `tests/fita_memoria.rs` prova `bytes()` a `±25 %` do medido). Classe: o anel da CPU
/// ([`crate::CPU_RING_BYTES`], 128 MiB), que é quem a fita serve no scrub.
///
/// A história que cabe (B/tique medido na app — `fila_c_a_fita_por_entrada`,
/// `measure_the_game_group` — e no gate; uma entrada parada não custa nada; doc 131 §7):
///
/// | entradas a mudar | B/tique | tiques | a 60 Hz |
/// |---|---:|---:|---:|
/// | o jogo do doc 128 §5.4 (3 a mexer) | 137 | 982 k | 4,5 h |
/// | 3, em todo tique | 215 | 623 k | 173 min |
/// | 30, em todo tique | 1 553 | 86 k | 24 min |
/// | 30 de duas colunas | 4 429 | 30 k | 8,4 min |
/// | 300, em todo tique | 14 933 | 9,0 k | 2,5 min |
///
/// ⚠️ Os «32 MiB» de antes declaravam `~36 B` por mudança e retinham `~640`: o jogo do doc 128
/// cabia os mesmos `~982 k` tiques, mas em `~600 MB` de RAM. Agora são 128 MiB reais.
pub const FITA_BYTES: usize = 128 * 1024 * 1024;

/// De quantas em quantas marcas a fita guarda uma ÂNCORA (o estado inteiro naquele tique): ler um
/// tique qualquer refaz no máximo esta quantidade de marcas, em vez da história desde a base. Uma
/// âncora custa um vetor de `nomes × 40 B` (as colunas são partilhadas): `~1 %` do que as marcas
/// entre duas âncoras ocupam.
const ANCORA_A_CADA: usize = 32;

/// O que o `malloc` acrescenta a cada pedido (cabeçalho e arredondamento, glibc).
const CABECALHO: usize = 16;

/// Uma árvore B que cresce pelo fim deixa as folhas (11 lugares) com ~6: cada entrada custa ~2×.
const FOLHA: usize = 2;

type Id = u32;

/// O conteúdo de uma coluna guardada.
#[derive(Clone)]
enum Corpo {
    /// Um elemento só, pelos BITS — a coluna publicada não fica presa à história (ela custava
    /// `84 B` por mudança: o `Arc<Column>` e o `Vec` de um número).
    Bits(Largura, [f32; 4]),
    /// Maior: a MESMA alocação do publicado, sem cópia.
    Partilhada(Arc<Column>),
}

#[derive(Clone)]
enum Colunas {
    /// Quase toda entrada é UM número (`v`) ou uma cor (`c`) numa coluna só: nada alocado.
    Pequena(Id, Largura, [f32; 4]),
    Uma(Id, Arc<Column>),
    /// Um gatilho (`n` e `v`), uma cor com o número (`v` e `c`), uma curva…
    Varias(Arc<[(Id, Corpo)]>),
}

#[derive(Clone, Copy)]
enum Largura {
    Um,
    Dois,
    Tres,
    Quatro,
}

/// O elemento único de `c`, por valor — `None` se a coluna não tem exactamente um.
fn pequena(c: &Column) -> Option<(Largura, [f32; 4])> {
    match c {
        Column::Scalar(v) if v.len() == 1 => Some((Largura::Um, [v[0], 0.0, 0.0, 0.0])),
        Column::Vec2(v) if v.len() == 1 => Some((Largura::Dois, [v[0][0], v[0][1], 0.0, 0.0])),
        Column::Vec3(v) if v.len() == 1 => Some((Largura::Tres, [v[0][0], v[0][1], v[0][2], 0.0])),
        Column::Vec4(v) if v.len() == 1 => Some((Largura::Quatro, v[0])),
        _ => None,
    }
}

fn coluna_de(l: Largura, x: [f32; 4]) -> Column {
    match l {
        Largura::Um => Column::Scalar(vec![x[0]]),
        Largura::Dois => Column::Vec2(vec![[x[0], x[1]]]),
        Largura::Tres => Column::Vec3(vec![[x[0], x[1], x[2]]]),
        Largura::Quatro => Column::Vec4(vec![x]),
    }
}

#[derive(Clone)]
struct Valor {
    rev: u64,
    count: usize,
    colunas: Colunas,
}

/// `valor: None` = o nome deixou de estar publicado.
struct Mudanca {
    nome: Id,
    valor: Option<Valor>,
}

/// O publicado num tique, por id de nome (`None` = ausente).
type Estado = Vec<Option<Valor>>;

/// Os nomes internados (um id por nome, na ordem em que apareceram).
#[derive(Default)]
struct Tabela {
    ids: BTreeMap<String, Id>,
    nomes: Vec<String>,
}

impl Tabela {
    /// O id de `nome`, e o que a tabela cresceu em bytes.
    fn id(&mut self, nome: &str) -> (Id, usize) {
        if let Some(&id) = self.ids.get(nome) {
            return (id, 0);
        }
        let id = Id::try_from(self.nomes.len()).expect("menos de 4 mil milhões de nomes");
        self.ids.insert(nome.to_owned(), id);
        self.nomes.push(nome.to_owned());
        let cresceu = 2 * bloco(nome.len())
            + size_of::<String>()
            + FOLHA * (size_of::<String>() + size_of::<Id>());
        (id, cresceu)
    }

    fn nome(&self, id: Id) -> &str {
        &self.nomes[id as usize]
    }
}

/// O estado à volta do último tique gravado: `antes` = o gravado ANTES de `t`, `em` = o de `t`.
struct Cursor {
    t: u64,
    antes: Estado,
    em: Estado,
}

#[derive(Default)]
pub struct Fita {
    nomes: Tabela,
    colunas: Tabela,
    /// O estado antes da primeira marca (e, depois de [`Self::dobra`], o de `base_ate`).
    base: Estado,
    base_ate: Option<u64>,
    marcas: BTreeMap<u64, Box<[Mudanca]>>,
    /// `ancoras[k]` = o estado no tique `k` (um tique com marca), a cada [`ANCORA_A_CADA`] marcas.
    ancoras: BTreeMap<u64, Estado>,
    /// `[primeiro, ultimo]` = os tiques gravados.
    primeiro: Option<u64>,
    ultimo: Option<u64>,
    /// Gravar o tique seguinte (a tocar para a frente, ou sobre chão já gravado) parte daqui, sem
    /// refazer a história (doc 131 §7, C.11).
    cursor: Option<Cursor>,
    /// Marcas, âncoras, as colunas da base e as tabelas; os vetores da base e do cursor somam-se
    /// em [`Self::bytes`].
    bytes: usize,
    orcamento: usize,
}

fn bloco(n: usize) -> usize {
    if n == 0 { 0 } else { n + CABECALHO }
}

fn peso_coluna(c: &Column) -> usize {
    bloco(2 * size_of::<usize>() + size_of::<Column>()) + bloco(c.approx_bytes())
}

fn corpo(c: &Arc<Column>) -> Corpo {
    pequena(c).map_or_else(
        || Corpo::Partilhada(Arc::clone(c)),
        |(l, x)| Corpo::Bits(l, x),
    )
}

fn coluna(c: &Corpo) -> Arc<Column> {
    match c {
        Corpo::Bits(l, x) => Arc::new(coluna_de(*l, *x)),
        Corpo::Partilhada(c) => Arc::clone(c),
    }
}

/// O que um valor segura de seu (as colunas).
fn peso_valor(v: &Valor) -> usize {
    match &v.colunas {
        Colunas::Pequena(..) => 0,
        Colunas::Uma(_, c) => peso_coluna(c),
        Colunas::Varias(cs) => {
            bloco(2 * size_of::<usize>() + size_of_val::<[(Id, Corpo)]>(&**cs))
                + cs.iter()
                    .map(|(_, c)| match c {
                        Corpo::Bits(..) => 0,
                        Corpo::Partilhada(c) => peso_coluna(c),
                    })
                    .sum::<usize>()
        }
    }
}

fn peso_estado(e: &Estado) -> usize {
    bloco(e.capacity() * size_of::<Option<Valor>>())
}

fn peso_marca(m: &[Mudanca]) -> usize {
    FOLHA * (size_of::<u64>() + size_of::<Box<[Mudanca]>>())
        + bloco(size_of_val(m))
        + m.iter()
            .filter_map(|x| x.valor.as_ref())
            .map(peso_valor)
            .sum::<usize>()
}

fn peso_ancora(e: &Estado) -> usize {
    FOLHA * (size_of::<u64>() + size_of::<Estado>()) + peso_estado(e)
}

fn peso_valores(e: &Estado) -> usize {
    e.iter().flatten().map(peso_valor).sum()
}

fn aplica(estado: &mut Estado, mudancas: &[Mudanca]) {
    for m in mudancas {
        let i = m.nome as usize;
        if i >= estado.len() {
            estado.resize(i + 1, None);
        }
        estado[i].clone_from(&m.valor);
    }
}

/// O valor compacto de `e`, as colunas partilhadas; e o que as tabelas cresceram.
fn compacta(colunas: &mut Tabela, e: &External) -> (Valor, usize) {
    let mut cresceu = 0;
    let mut id = |n: &str| {
        let (id, b) = colunas.id(n);
        cresceu += b;
        id
    };
    let mut it = e.value.shared_columns();
    let cols = match (it.next(), it.next()) {
        (Some((n, c)), None) => match pequena(c) {
            Some((l, x)) => Colunas::Pequena(id(n), l, x),
            None => Colunas::Uma(id(n), Arc::clone(c)),
        },
        _ => Colunas::Varias(
            e.value
                .shared_columns()
                .map(|(n, c)| (id(n), corpo(c)))
                .collect(),
        ),
    };
    let valor = Valor {
        rev: e.rev,
        count: e.value.count(),
        colunas: cols,
    };
    (valor, cresceu)
}

/// As mudanças de `de` para `agora`; e o que as tabelas cresceram.
fn diferenca(
    nomes: &mut Tabela,
    colunas: &mut Tabela,
    de: &Estado,
    agora: &All,
) -> (Vec<Mudanca>, usize) {
    let mut out = Vec::new();
    let mut cresceu = 0;
    for (nome, e) in agora {
        let (id, b) = nomes.id(nome);
        cresceu += b;
        if de
            .get(id as usize)
            .and_then(Option::as_ref)
            .is_none_or(|d| d.rev != e.rev)
        {
            let (valor, b) = compacta(colunas, e);
            cresceu += b;
            out.push(Mudanca {
                nome: id,
                valor: Some(valor),
            });
        }
    }
    for (i, v) in de.iter().enumerate() {
        let id = Id::try_from(i).expect("ids cabem");
        if v.is_some() && !agora.contains_key(nomes.nome(id)) {
            out.push(Mudanca {
                nome: id,
                valor: None,
            });
        }
    }
    (out, cresceu)
}

fn mesmas_revisoes(nomes: &Tabela, e: &Estado, agora: &All) -> bool {
    e.iter().flatten().count() == agora.len()
        && agora.iter().all(|(n, x)| {
            nomes
                .ids
                .get(n)
                .and_then(|&id| e.get(id as usize)?.as_ref())
                .is_some_and(|v| v.rev == x.rev)
        })
}

impl Fita {
    #[must_use]
    pub fn new() -> Self {
        Self {
            orcamento: FITA_BYTES,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn cobre(&self, t: u64) -> bool {
        matches!((self.primeiro, self.ultimo), (Some(p), Some(u)) if p <= t && t <= u)
    }

    /// O heap que a fita retém, com o cabeçalho do `malloc` — o que o orçamento compara.
    #[must_use]
    pub fn bytes(&self) -> usize {
        self.bytes
            + peso_estado(&self.base)
            + self
                .cursor
                .as_ref()
                .map_or(0, |c| peso_estado(&c.antes) + peso_estado(&c.em))
    }

    pub fn set_orcamento(&mut self, bytes: usize) {
        self.orcamento = bytes;
    }

    fn externo(&self, v: &Valor) -> External {
        let mut s = Stream::new(v.count);
        let mut col = |id: Id, c: Arc<Column>| {
            s = std::mem::take(&mut s).with_shared(self.colunas.nome(id), c);
        };
        match &v.colunas {
            Colunas::Pequena(id, l, x) => col(*id, Arc::new(coluna_de(*l, *x))),
            Colunas::Uma(id, c) => col(*id, Arc::clone(c)),
            Colunas::Varias(cs) => cs.iter().for_each(|(id, c)| col(*id, coluna(c))),
        }
        External {
            rev: v.rev,
            value: s,
        }
    }

    fn para_all(&self, e: &Estado) -> All {
        e.iter()
            .enumerate()
            .filter_map(|(i, v)| {
                let id = Id::try_from(i).ok()?;
                Some((self.nomes.nome(id).to_owned(), self.externo(v.as_ref()?)))
            })
            .collect()
    }

    /// O que estava publicado em `t`, ou `None` fora do que foi gravado (o replay usa então o
    /// publicado agora).
    #[must_use]
    pub fn estado_em(&self, t: u64) -> Option<All> {
        if !self.cobre(t) {
            return None;
        }
        Some(match &self.cursor {
            Some(c) if c.t == t => self.para_all(&c.em),
            _ => self.para_all(&self.estado_ate(t + 1)),
        })
    }

    /// Avança `estado` (o de `t − 1`) para `t`; `true` se mudou — o replay anda tique a tique sem
    /// refazer a soma desde a base.
    pub fn avanca(&self, t: u64, estado: &mut All) -> bool {
        let Some(m) = self.marcas.get(&t) else {
            return false;
        };
        for x in m.iter() {
            let nome = self.nomes.nome(x.nome);
            match &x.valor {
                Some(v) => {
                    estado.insert(nome.to_owned(), self.externo(v));
                }
                None => {
                    estado.remove(nome);
                }
            }
        }
        true
    }

    /// Grava o que está publicado AGORA como o estado de `t`.
    pub fn grava(&mut self, t: u64, agora: &All) -> Gravacao {
        let resultado = if self.cobre(t) {
            self.posiciona(t);
            let em = &self.cursor.as_ref().expect("posicionado").em;
            if mesmas_revisoes(&self.nomes, em, agora) {
                return Gravacao::Igual;
            }
            Gravacao::Ramo
        } else if self.primeiro.is_some_and(|p| t < p) {
            Gravacao::Ramo
        } else {
            Gravacao::Nova
        };
        if resultado == Gravacao::Ramo {
            self.esquece_de(t);
        }
        if self.base_ate.is_some_and(|b| t <= b) {
            // Dentro do que já foi dobrado: o tique passa a ser a nova base.
            let (m, cresceu) = diferenca(&mut self.nomes, &mut self.colunas, &Estado::new(), agora);
            let mut base = Estado::new();
            aplica(&mut base, &m);
            self.bytes = self.bytes + cresceu + peso_valores(&base) - peso_valores(&self.base);
            self.base = base;
            self.base_ate = Some(t);
            self.cursor = Some(Cursor {
                t,
                antes: self.base.clone(),
                em: self.base.clone(),
            });
        } else {
            self.posiciona(t);
            let c = self.cursor.as_mut().expect("posicionado");
            let (m, cresceu) = diferenca(&mut self.nomes, &mut self.colunas, &c.antes, agora);
            c.em.clone_from(&c.antes);
            aplica(&mut c.em, &m);
            self.bytes += cresceu;
            if !m.is_empty() {
                let m = m.into_boxed_slice();
                self.bytes += peso_marca(&m);
                self.marcas.insert(t, m);
                let desde = self.ancoras.last_key_value().map_or(0, |(k, _)| k + 1);
                if self.marcas.range(desde..).take(ANCORA_A_CADA).count() == ANCORA_A_CADA {
                    let a = c.em.clone();
                    self.bytes += peso_ancora(&a);
                    self.ancoras.insert(t, a);
                }
            }
        }
        self.primeiro = self.primeiro.or(Some(t));
        self.ultimo = Some(t);
        while self.bytes() > self.orcamento && !self.marcas.is_empty() {
            self.dobra();
        }
        resultado
    }

    /// Põe o cursor em `t` — a partir dele se nenhuma marca ficou entre os dois, senão da âncora.
    fn posiciona(&mut self, t: u64) {
        if !self.cursor.as_ref().is_some_and(|c| c.t == t) {
            let antes = match self.cursor.take() {
                Some(c) if c.t < t && self.marcas.range(c.t + 1..t).next().is_none() => c.em,
                _ => self.estado_ate(t),
            };
            let mut em = antes.clone();
            if let Some(m) = self.marcas.get(&t) {
                aplica(&mut em, m);
            }
            self.cursor = Some(Cursor { t, antes, em });
        }
    }

    /// O estado no fim do que está gravado ANTES de `t`: da âncora mais próxima, no máximo
    /// [`ANCORA_A_CADA`] marcas.
    fn estado_ate(&self, t: u64) -> Estado {
        let (desde, mut estado) = match self.ancoras.range(..t).next_back() {
            Some((k, a)) => (k + 1, a.clone()),
            None => (0, self.base.clone()),
        };
        for m in self.marcas.range(desde..t).map(|(_, m)| m) {
            aplica(&mut estado, m);
        }
        estado
    }

    /// Esquece `t` e tudo depois; antes do primeiro tique gravado, esquece tudo.
    fn esquece_de(&mut self, t: u64) {
        if self.primeiro.is_none_or(|p| t <= p) {
            self.clear_keeping_budget();
            return;
        }
        for m in self.marcas.split_off(&t).values() {
            self.bytes -= peso_marca(m);
        }
        for a in self.ancoras.split_off(&t).values() {
            self.bytes -= peso_ancora(a);
        }
        self.ultimo = Some(t - 1);
        // Em `t` o `antes` continua válido (quem chama reescreve o `em`); depois, não.
        if self.cursor.as_ref().is_some_and(|c| c.t > t) {
            self.cursor = None;
        }
    }

    /// ⚠️ **Acima do orçamento, a marca mais velha DOBRA-SE na base**: um scrub para antes dela
    /// repete o estado da base em vez do que se publicou na época — o limite nomeado (doc 128 §5).
    fn dobra(&mut self) {
        let Some((t, m)) = self.marcas.pop_first() else {
            return;
        };
        self.bytes -= peso_marca(&m);
        for x in m.into_vec() {
            let i = x.nome as usize;
            if i >= self.base.len() {
                self.base.resize(i + 1, None);
            }
            if let Some(v) = &x.valor {
                self.bytes += peso_valor(v);
            }
            if let Some(velho) = std::mem::replace(&mut self.base[i], x.valor) {
                self.bytes -= peso_valor(&velho);
            }
        }
        self.base_ate = Some(t);
        let depois = self.ancoras.split_off(&(t + 1));
        for a in std::mem::replace(&mut self.ancoras, depois).values() {
            self.bytes -= peso_ancora(a);
        }
        if self.cursor.as_ref().is_some_and(|c| c.t < t) {
            self.cursor = None;
        }
    }

    fn clear_keeping_budget(&mut self) {
        let orcamento = self.orcamento;
        *self = Self {
            orcamento,
            ..Self::default()
        };
    }

    pub fn clear(&mut self) {
        self.clear_keeping_budget();
    }
}

#[cfg(test)]
#[path = "fita_tests.rs"]
mod tests;
