//! [`FlipDrawing`] — um desenho (conjunto de traços), refcontado pelos frames.
//!
//! Um mesmo desenho pode ser reusado por vários frames (ciclo / "duplicate as
//! instance") — o `users` conta quantos frames o referenciam. Quando cai a zero,
//! [`crate::FlipObject::remove_unused_drawings`] o reclama e remapeia os índices.
//! Espelha o `DrawingRuntime::user_count` do GP (`02_referencia §1`).
//!
//! **O `users` é um cache mantido pelas ops de frame do objeto** (`add_user`/
//! `remove_user`), com a verdade canônica reconstruível por
//! `FlipObject::recompute_users` (contagem direta a partir dos frames). A
//! compactação sempre recomputa antes de reclamar, então drift eventual do
//! cache se auto-corrige.

use crate::stroke::FlipStroke;
use serde::{Deserialize, Serialize};

/// Um desenho: os traços + a contagem de frames que o usam. `Default` = vazio,
/// `users = 0` (o refcount é acertado quando um frame o referencia).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FlipDrawing {
    /// Os traços deste desenho, em ordem de z (fundo → topo).
    pub strokes: Vec<FlipStroke>,
    /// Quantos frames referenciam este desenho (refcount). `0` = reclamável.
    users: u32,
}

impl FlipDrawing {
    /// Desenho vazio, sem usuários (o `users` é acertado quando um frame o
    /// referencia — via as ops de `FlipObject`).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Contagem atual de usuários.
    #[must_use]
    pub fn users(&self) -> u32 {
        self.users
    }

    /// Tem ao menos um frame apontando (não é lixo).
    #[must_use]
    pub fn has_users(&self) -> bool {
        self.users > 0
    }

    /// É instanciado por mais de um frame (ciclo / reuse). Editar um desenho
    /// instanciado propaga para todos os frames que o compartilham — o valor do
    /// "duplicate as instance".
    #[must_use]
    pub fn is_instanced(&self) -> bool {
        self.users > 1
    }

    /// +1 usuário (um frame passou a referenciar).
    pub fn add_user(&mut self) {
        self.users = self.users.saturating_add(1);
    }

    /// −1 usuário (um frame deixou de referenciar). Satura em `0`.
    pub fn remove_user(&mut self) {
        self.users = self.users.saturating_sub(1);
    }

    /// Define a contagem diretamente — usado por `FlipObject::recompute_users`
    /// (a reconstrução canônica a partir dos frames).
    pub(crate) fn set_users(&mut self, n: u32) {
        self.users = n;
    }

    // ── Seleção (W6 — o Edit Mode; `FlipStroke::selected`, domínio Curve) ──
    //
    // A seleção é **derivada dos traços**, nunca guardada em paralelo: perguntar
    // "quem está selecionado?" é uma varredura, e é assim que ela não pode
    // dessincronizar de uma inserção/remoção (o balde insere no meio da lista).

    /// Os índices dos traços selecionados, em ordem de z.
    #[must_use]
    pub fn selected_indices(&self) -> Vec<usize> {
        self.strokes
            .iter()
            .enumerate()
            .filter(|(_, s)| s.selected)
            .map(|(i, _)| i)
            .collect()
    }

    /// Há algum traço selecionado?
    #[must_use]
    pub fn any_selected(&self) -> bool {
        self.strokes.iter().any(|s| s.selected)
    }

    /// Desmarca tudo — o traço E os pontos (o domínio Point materializado fica, todo
    /// falso; a projeção `any()` zera o Curve junto). Devolve `true` se algo mudou (o
    /// chamador decide se isso é um passo de undo).
    pub fn clear_selection(&mut self) -> bool {
        let mut changed = false;
        for s in &mut self.strokes {
            changed |= std::mem::replace(&mut s.selected, false);
            for i in 0..s.len() {
                if s.point_selected(i) {
                    changed |= s.set_point_selected(i, false);
                }
            }
        }
        changed
    }

    /// Remove os traços selecionados. Devolve quantos saíram.
    pub fn delete_selected(&mut self) -> usize {
        let before = self.strokes.len();
        self.strokes.retain(|s| !s.selected);
        before - self.strokes.len()
    }

    // ── Domínio POINT (W8 — `02_referencia §11`): conversão explícita + agregadas ──

    /// **Entrar no domínio Point** (o toggle do painel foi para Point): **começa
    /// DESSELECIONADO** (Enio, smoke do §4.A — *"faça com que em Select: Point comece com
    /// pontos desselecionados"*).
    ///
    /// **Isto DIVERGE do GP de propósito** (o `02_referencia §11` faz broadcast: traço
    /// selecionado ⇒ todos os pontos dele acesos). O broadcast entrega o domínio Point com
    /// TUDO aceso, e o 1º gesto do artista ali é quase sempre *"quero estas duas âncoras"*
    /// — ou seja, ele começa **desmarcando**. Entrar limpo torna o gesto seguinte o pick,
    /// que é o que o modo existe para fazer. (A volta ao Stroke **promove** por `any()`:
    /// ver [`Self::enter_stroke_domain`]. A assimetria é deliberada — entrar no Point é
    /// "vou escolher âncoras"; voltar ao Stroke é "as âncoras que toquei são deste traço".)
    pub fn enter_point_domain(&mut self) {
        self.clear_selection();
    }

    /// **Entrar no domínio Stroke** (o toggle voltou para Stroke): promove `any()` por
    /// traço e desmaterializa os vetores (half-selected só existe em Point, §11).
    pub fn enter_stroke_domain(&mut self) {
        for s in &mut self.strokes {
            s.promote_points_to_stroke();
        }
    }

    /// Seleciona TODOS os pontos de todos os traços (o botão "All" no domínio Point).
    /// Devolve `true` se algo mudou.
    pub fn select_all_points(&mut self) -> bool {
        let mut changed = false;
        for s in &mut self.strokes {
            for i in 0..s.len() {
                changed |= s.set_point_selected(i, true);
            }
        }
        changed
    }

    /// Remove os pontos selecionados (dissolve, por traço) e descarta os traços que
    /// ficaram VAZIOS. Devolve quantos pontos saíram.
    pub fn delete_selected_points(&mut self) -> usize {
        let mut removed = 0;
        for s in &mut self.strokes {
            removed += s.remove_selected_points();
        }
        if removed > 0 {
            self.strokes.retain(|s| !s.is_empty());
        }
        removed
    }
}

#[cfg(test)]
mod tests {
    /// 🔴 **Entrar no domínio Point começa DESSELECIONADO** (Enio, smoke do §4.A: *"faça
    /// com que em Select: Point comece com pontos desselecionados"*).
    ///
    /// O `02 §11` do GP faz **broadcast** aqui (traço aceso ⇒ todos os pontos dele acesos),
    /// e era isso que entregava o Point com tudo selecionado — obrigando o artista a
    /// desmarcar antes de escolher as âncoras que queria. A ida limpa; a **volta** ao
    /// Stroke ainda promove por `any()`, e a assimetria é deliberada.
    ///
    /// Mutação que sangra: `enter_point_domain` voltar a fazer broadcast (ou virar no-op —
    /// a seleção de traço sobreviveria e todo ponto leria `true` pelo "ausente = broadcast").
    #[test]
    fn entering_the_point_domain_starts_deselected() {
        let mut d = FlipDrawing::new();
        let mut s = crate::FlipStroke::new();
        for i in 0..3 {
            s.push_default(ph2d_core::Vec2::new(i as f32, 0.0));
        }
        s.selected = true;
        d.strokes.push(s);
        // Antes: o traço está aceso, e sem dado de ponto TODO ponto lê como aceso.
        assert!((0..3).all(|i| d.strokes[0].point_selected(i)));

        d.enter_point_domain();

        assert!(
            (0..3).all(|i| !d.strokes[0].point_selected(i)),
            "Select: Point tem de comecar com TODOS os pontos desselecionados"
        );
        assert!(
            !d.strokes[0].selected,
            "e o traco tambem sai (any() = false)"
        );
        assert!(
            d.strokes[0].soa_is_consistent(),
            "a invariante do §11 fica de pe"
        );
        assert!(!d.any_selected());
    }

    /// **A volta ao Stroke PROMOVE** — o par do gate acima: as âncoras que o artista tocou
    /// dizem de que traço ele estava cuidando.
    #[test]
    fn entering_the_stroke_domain_promotes_by_any() {
        let mut d = FlipDrawing::new();
        let mut s = crate::FlipStroke::new();
        for i in 0..3 {
            s.push_default(ph2d_core::Vec2::new(i as f32, 0.0));
        }
        d.strokes.push(s);
        d.strokes[0].set_point_selected(1, true);
        d.enter_stroke_domain();
        assert!(d.strokes[0].selected, "uma ancora acesa promove o traco");
        assert!(
            !d.strokes[0].has_point_selection(),
            "desmaterializa na volta"
        );
    }
    use super::*;

    #[test]
    fn refcount_transitions() {
        let mut d = FlipDrawing::new();
        assert!(!d.has_users() && !d.is_instanced());
        d.add_user();
        assert!(d.has_users() && !d.is_instanced());
        d.add_user();
        assert!(d.is_instanced());
        d.remove_user();
        assert!(d.has_users() && !d.is_instanced());
        d.remove_user();
        assert!(!d.has_users());
        // Satura em 0 (não faz underflow).
        d.remove_user();
        assert_eq!(d.users(), 0);
    }
}
