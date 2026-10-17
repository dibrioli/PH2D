//! A árvore que segue o texto: parse inteiro uma vez, depois reparse INCREMENTAL por edição.
//!
//! Lei medida (`tests/it/incremental.rs`): com o texto VÁLIDO a árvore incremental é igual à do
//! parse do zero (forma e faixas). Com ERROR, a recuperação de erro do tree-sitter depende da
//! história de edições: ambas veem o erro, mas a forma dentro dele pode diferir (medido: 0 a 17
//! passos em 600 de traços aleatórios longos), e a diferença nunca sobrevive à volta a um texto
//! válido. Re-parse do zero quando há erro foi RECUSADO: 17 ms contra 1,9 ms por tecla a 7 500
//! linhas (`reparse_por_tecla`), e texto quebrado é o estado normal de quem escreve.
//!
//! **Reparse FORA da tecla** (Onda 2, I1 — `docs/CodeEditor/03_onda2.md`): [`Syntax::edit_deferred`]
//! só desloca a árvore (`Tree::edit`, sem reparse) e marca-a VELHA; um worker reparsa um
//! [`ReparseJob`] (a árvore e o texto, ambos por `Arc`) e [`Syntax::adopt`] traz a árvore nova,
//! re-aplicando as edições que chegaram entretanto. Quem precisa da árvore exacta NA tecla chama
//! [`Syntax::fresh`] (o Enter, o auto-par, o recuo de `end`), e só esse paga o reparse síncrono.
//! Medido (I0a): o reparse percorre cada declaração de topo — 1,3 µs por `ph2d.property(…)` —
//! dentro do tree-sitter; não há cura barata do nosso lado.

use crate::queries;
use ph2d_code_buffer::{Point, Text, TextEdit};
use tree_sitter::{InputEdit, Parser, Tree};

pub struct Syntax {
    parser: Parser,
    tree: Tree,
    /// A árvore da última [`Syntax::changed_since_mark`], editada junto (as faixas mudadas
    /// saem da comparação dela com a de agora). `None` = ninguém pediu.
    mark: Option<Tree>,
    /// A árvore foi deslocada sem reparse: a forma dentro das faixas editadas é a de antes.
    stale: bool,
    /// As edições depois do último [`ReparseJob`] (a re-aplicar na árvore que ele trouxer).
    pending: Vec<InputEdit>,
    /// O trabalho em voo; outro número (ou `None`) torna velha a resposta que chegar.
    job: Option<u64>,
    next_job: u64,
    /// Cresce cada vez que a árvore muda de FORMA (reparse ou adopção) — a chave de quem deriva dela.
    generation: u64,
    sync_reparses: u64,
}

/// O reparse de uma árvore deslocada, para correr noutra thread com o seu [`Parser`].
pub struct ReparseJob {
    id: u64,
    tree: Tree,
    text: Text,
}

pub struct Reparsed {
    id: u64,
    tree: Tree,
}

impl ReparseJob {
    pub fn run(self, parser: &mut Parser) -> Reparsed {
        Reparsed {
            id: self.id,
            tree: parse(parser, &self.text, Some(&self.tree)),
        }
    }
}

/// Um parser com a gramática Luau (um por thread de reparse).
pub fn luau_parser() -> Parser {
    let mut parser = Parser::new();
    parser
        .set_language(&queries::get().language)
        .expect("ABI da gramática Luau fora da faixa do tree-sitter (teste `abi_da_gramatica`)");
    parser
}

fn input_edit(e: &TextEdit) -> InputEdit {
    InputEdit {
        start_byte: e.start_byte,
        old_end_byte: e.old_end_byte,
        new_end_byte: e.new_end_byte,
        start_position: ts_point(e.start),
        old_end_position: ts_point(e.old_end),
        new_end_position: ts_point(e.new_end),
    }
}

fn ts_point(p: Point) -> tree_sitter::Point {
    tree_sitter::Point {
        row: p.row,
        column: p.col,
    }
}

/// Lê a rope por pedaços: o tree-sitter pede «o texto a partir do byte b», a rope dá o resto do
/// pedaço onde `b` cai — nunca uma `String` do texto inteiro.
fn parse(parser: &mut Parser, text: &Text, old: Option<&Tree>) -> Tree {
    let rope = text.rope();
    let len = rope.byte_len();
    let mut read = |byte: usize, _: tree_sitter::Point| -> &[u8] {
        if byte >= len {
            return &[];
        }
        // Um pedido no meio de um carácter lê a partir do início dele e salta os bytes já lidos.
        let at = text.floor_char(byte);
        rope.byte_slice(at..len)
            .chunks()
            .next()
            .map_or(&[][..], |c| &c.as_bytes()[byte - at..])
    };
    // `None` só com cancelamento ou sem linguagem; nenhum dos dois existe aqui (texto quebrado
    // dá árvore com ERROR, nunca `None`).
    parser
        .parse_with_options(&mut read, old, None)
        .expect("tree-sitter sem cancelamento devolve sempre árvore")
}

impl Syntax {
    pub fn new(text: &Text) -> Self {
        let mut parser = luau_parser();
        let tree = parse(&mut parser, text, None);
        Self {
            parser,
            tree,
            mark: None,
            stale: false,
            pending: Vec::new(),
            job: None,
            next_job: 0,
            generation: 0,
            sync_reparses: 0,
        }
    }

    /// Aplica as edições à árvore (na ordem dada: a do `Buffer`, do fim para o início) e reparse
    /// incremental contra `text_after`, o texto JÁ editado.
    pub fn edit(&mut self, edits: &[TextEdit], text_after: &Text) {
        if edits.is_empty() {
            return;
        }
        self.edit_deferred(edits);
        self.reparse_now(text_after);
    }

    /// Só desloca a árvore (sem reparse): ela fica VELHA até [`Self::adopt`] ou [`Self::fresh`].
    pub fn edit_deferred(&mut self, edits: &[TextEdit]) {
        for e in edits {
            let ie = input_edit(e);
            self.tree.edit(&ie);
            if let Some(m) = &mut self.mark {
                m.edit(&ie);
            }
            self.pending.push(ie);
            self.stale = true;
        }
    }

    fn reparse_now(&mut self, text: &Text) {
        self.tree = parse(&mut self.parser, text, Some(&self.tree));
        self.pending.clear();
        self.job = None;
        self.stale = false;
        self.generation += 1;
    }

    /// A árvore exacta de `text` (o texto de agora): reparse síncrono só se estiver velha.
    pub fn fresh(&mut self, text: &Text) {
        if self.stale {
            self.sync_reparses += 1;
            self.reparse_now(text);
        }
    }

    pub fn is_stale(&self) -> bool {
        self.stale
    }

    /// Muda quando a FORMA da árvore muda (não quando só é deslocada).
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Quantas vezes [`Self::fresh`] teve de reparsar NA hora (o gate da tecla conta-as).
    pub fn sync_reparses(&self) -> u64 {
        self.sync_reparses
    }

    /// O trabalho de reparse da árvore velha sobre `text` (o texto de agora), se não há um em voo.
    pub fn job(&mut self, text: &Text) -> Option<ReparseJob> {
        if !self.stale || self.job.is_some() {
            return None;
        }
        self.next_job += 1;
        self.job = Some(self.next_job);
        self.pending.clear();
        Some(ReparseJob {
            id: self.next_job,
            tree: self.tree.clone(),
            text: text.clone(),
        })
    }

    /// Traz a árvore de um [`ReparseJob`]: desloca-a pelas edições que chegaram depois dele. Uma
    /// resposta de um trabalho esquecido (por [`Self::fresh`]) é deitada fora. Devolve `true` se
    /// ainda falta outro trabalho (houve edições entretanto).
    pub fn adopt(&mut self, r: Reparsed) -> bool {
        if self.job != Some(r.id) {
            return self.stale && self.job.is_none();
        }
        self.job = None;
        let mut tree = r.tree;
        for ie in &self.pending {
            tree.edit(ie);
        }
        self.tree = tree;
        self.stale = !self.pending.is_empty();
        self.generation += 1;
        self.stale
    }

    /// Marca a árvore de agora (se ainda não há marca): daqui em diante as edições levam-na junto.
    pub fn arm_changes(&mut self) {
        if self.mark.is_none() {
            self.mark = Some(self.tree.clone());
        }
    }

    /// Esquece a marca: as edições deixam de a levar e ninguém paga a comparação.
    pub fn disarm_changes(&mut self) {
        self.mark = None;
    }

    /// As faixas de bytes (do texto de agora) cuja estrutura mudou desde a marca, e marca a árvore
    /// de agora para a seguinte. Sem marca só marca (`None`). Custa a comparação das duas árvores
    /// (`changed_ranges`): só se arma quando alguém precisa.
    pub fn changed_since_mark(&mut self) -> Option<Vec<std::ops::Range<usize>>> {
        let out = self.mark.as_ref().map(|m| {
            m.changed_ranges(&self.tree)
                .map(|r| r.start_byte..r.end_byte)
                .collect()
        });
        self.mark = Some(self.tree.clone());
        out
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    pub fn sexp(&self) -> String {
        self.tree.root_node().to_sexp()
    }
}
