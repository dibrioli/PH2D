//! **COMO O MAP DE UM DAB É DIVIDIDO E RECOLHIDO** — o outro lado do
//! [ADR-0159](../../../docs/architecture/decisions/0159-sculpt3d-the-dab-vertex-loop-is-a-row-disjoint-map-rayon-exception.md).
//!
//! ⚠️ **A linha de corte é *como o trabalho é dividido* × *o que um dab
//! computa*.** O corpo do laço — a lei do pincel — fica no `stroke.rs`, ao lado
//! do resto do dab; aqui mora só o piso do pool e o espalhamento, que são
//! decisões sobre AGENDAMENTO e não sobre barro.
//!
//! ## Por que o laço do dab pode ser dividido
//!
//! Ele **não escreve posições** — quem escreve é o `apply_positions`, depois —,
//! as leituras são puras dentro de um passe (inclusive as `from_live`: ninguém
//! move a malha enquanto ele corre) e os slots são disjuntos, um por vértice.
//! As três escritas que ele fazia (`accum[s]`, `target[s]`, `moved.push`)
//! viraram uma saída por índice, espalhada em seguida na ORDEM dos índices.
//!
//! ⚠️ **Nada em ponto flutuante é acumulado ali**, e a exceção do ADR depende
//! disso: o `fit_plane` e o banco do `Grip` somam `f32` e ficam FORA, onde já
//! estavam. Ordem de soma entre threads move bits; ordem de *avaliação* de um
//! map não move nenhum.

use rayon::prelude::*;

use super::SculptStroke;

/// O que um índice do map devolve: o vértice, o slot dele, e os dois valores
/// que o espalhamento vai escrever.
///
/// ⚠️ **Ela existe para o laço não escrever em `self`** — cada índice devolve o
/// que ia escrever, e o espalhamento roda serial depois. Ver o ADR-0159.
pub(crate) type DabOut = (u32, usize, f32, [f32; 3]);

/// **O piso do pool para o map do dab** — abaixo dele o `rayon` custa mais que
/// o trabalho que ele divide.
///
/// ⚠️ É **medido**, não escolhido: um pincel de detalhe (2 % do modelo) toca
/// ~121 vértices na malha que o módulo abre, e ali o serial ganha. Ver
/// `tests/measure_field_cost.rs` e o padrão por-passe do
/// [ADR-0145](../../../docs/architecture/decisions/0145-wet-paint-solver-row-parallel-passes-rayon-exception.md).
pub(crate) const PAR_MIN_VERTS: usize = 4_096;

impl SculptStroke {
    /// O piso vigente do pool — o [`PAR_MIN_VERTS`], ou o que a ablação de
    /// teste pediu. **Uma porta**: o produto e os gates perguntam à mesma.
    pub(super) fn par_floor(&self) -> usize {
        #[cfg(test)]
        if let Some(f) = self.par_floor_override {
            return f;
        }
        PAR_MIN_VERTS
    }

    /// **RODA O MAP DE UM DAB e espalha o resultado** — a porta única da
    /// maquinaria do ADR-0159, para que o `stroke.rs` fique com a LEI e não com
    /// o agendamento.
    ///
    /// O `body` recebe `&SculptStroke` (leitura pura), o índice, e a saída
    /// daquele índice. ⚠️ Ele **não pode** escrever em `self`: é essa
    /// assinatura que torna o map disjunto uma propriedade do TIPO em vez de
    /// uma promessa de comentário.
    /// ⚠️ **A rota SERIAL não toca o buffer, e essa é a metade que faltava.**
    /// O `Vec<Option<DabOut>>` existe porque o `rayon` precisa de slots
    /// DISJUNTOS para escrever; quem corre num núcleo só pode espalhar cada
    /// índice na hora, que é literalmente o que o laço fazia antes do
    /// ADR-0159. A 1ª versão desta porta cobrava as três passadas do buffer
    /// (preencher de `None`, escrever, reler) das DUAS rotas — e num pincel
    /// de detalhe a pegada fica **sob** o piso do pool, então o caminho mais
    /// comum pagava a maquinaria de uma otimização que ele nunca corre.
    pub(super) fn run_dab_map<F>(&mut self, count: usize, first: bool, body: F)
    where
        F: Fn(&Self, usize, &mut Option<DabOut>) + Sync,
    {
        if count < self.par_floor() {
            for i in 0..count {
                let mut slot: Option<DabOut> = None;
                body(&*self, i, &mut slot);
                if let Some(entry) = slot {
                    self.scatter_one(entry, first);
                }
            }
            return;
        }

        let mut out = std::mem::take(&mut self.par_out);
        out.clear();
        out.resize(count, None);
        {
            let me: &Self = self;
            out.par_iter_mut()
                .enumerate()
                .for_each(|(i, o)| body(me, i, o));
        }
        self.scatter_dab_map(&out, first);
        self.par_out = out;
    }

    /// **O que UMA saída do map escreve** — a porta que as duas rotas
    /// perguntam, para não haver duas respostas a *"onde esta entrada pousa?"*.
    #[inline]
    fn scatter_one(&mut self, entry: DabOut, first: bool) {
        let (v, s, acc, tgt) = entry;
        self.accum[s] = acc;
        self.target[s] = tgt;
        if first {
            self.moved.push(v);
        }
    }

    /// **O ESPALHAMENTO — serial e na ORDEM dos índices.**
    ///
    /// ⚠️ A ordem é o que mantém o `moved` igual ao que a rota serial publica —
    /// e ela é **observável**, porque a rota serial não passa por aqui: ela
    /// espalha cada índice na hora, em ordem, por construção. Inverter este
    /// laço faz a mutação SANGRAR no `the_parallel_dab_lays_the_same_clay…`.
    ///
    /// ⚠️ Enquanto as DUAS rotas partilhavam este laço, um gate
    /// rota-contra-rota era **cego** a ele (inverter os dois lados é razão
    /// entre dois doentes) e a mutação sobrevivia por desenho. O que a tornou
    /// visível não foi um gate novo: foi a rota serial deixar de usá-lo.
    pub(super) fn scatter_dab_map(&mut self, out: &[Option<DabOut>], first: bool) {
        for entry in out.iter().flatten() {
            self.scatter_one(*entry, first);
        }
    }
}
