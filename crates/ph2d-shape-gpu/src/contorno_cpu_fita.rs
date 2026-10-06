//! ⭐ doc 121 §9.19 (2) — **A FITA**: os pedaços de um traço que se ligam pela FAIXA num só polígono.
//!
//! Dois quadriláteros que acabam na MESMA bissectriz partilham a aresta nos dois sentidos; a placa não a
//! escreve, e a porta CPU escrevia-a duas vezes (cancelam-se no `nonzero`). Aqui a fita junta-os: a
//! esquerda pela ordem do traço, a direita fecha ao contrário — a mesma soma com menos arestas para o Vello.
//! ⚠️ Só continua um pedaço que começa nos MESMOS bits em que a fita acabou e com o mesmo sentido (um
//! quadrilátero virado somaria `−1`, e a aresta partilhada deixaria de se anular).

use super::{Saida, V, positivo};

/// Os dois lados da fita aberta.
#[derive(Default)]
pub(super) struct Fita {
    esq: Vec<V>,
    dir: Vec<V>,
}

impl Saida<'_> {
    /// O quadrilátero `(q0 + m0, q1 + m1, q1 − m1, q0 − m0)` de um pedaço, na fita.
    pub(super) fn quadrilatero_na_fita(&mut self, q0: V, q1: V, m0: V, m1: V) {
        let (a, b, c, d) = (q0 + m0, q1 + m1, q1 - m1, q0 - m0);
        if !positivo(a, b, c) {
            self.fecha_a_fita();
            self.quad(a, b, c, d);
            return;
        }
        let continua = self.fita.esq.last() == Some(&a) && self.fita.dir.last() == Some(&d);
        if !continua {
            self.fecha_a_fita();
            self.fita.esq.push(a);
            self.fita.dir.push(d);
        }
        self.fita.esq.push(b);
        self.fita.dir.push(c);
    }

    /// Fecha a fita aberta num polígono (a esquerda, e a direita ao contrário).
    pub(super) fn fecha_a_fita(&mut self) {
        if self.fita.esq.is_empty() {
            return;
        }
        let mut pts = std::mem::take(&mut self.fita.esq);
        let mut dir = std::mem::take(&mut self.fita.dir);
        pts.extend(dir.drain(..).rev());
        self.poligono(&pts);
        pts.clear();
        self.fita = Fita { esq: pts, dir };
    }
}
