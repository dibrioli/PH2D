//! O fim da cadeia do `tr_ingles`, pela MESMA ordem: as famílias `ph2d-app-*`, o editor de scripts
//! e o Kanban. Mora aqui porque o `lib.rs` bate no tecto de LOC (cura por corte, nunca isenção).

use crate::{
    app_components, app_flip, app_motion, app_painter, app_physics, app_vec, code_editor,
    code_explain, code_help, kanban, kanban_back,
};

/// A tradução de uma chave destas famílias, ou `None` se ela não é de nenhuma.
pub(crate) fn tr(k: &str) -> Option<&'static str> {
    app_components::tr(k)
        .or_else(|| app_flip::tr(k))
        .or_else(|| app_painter::tr(k))
        .or_else(|| app_physics::tr(k))
        .or_else(|| app_motion::tr(k))
        .or_else(|| app_vec::tr(k))
        .or_else(|| code_editor::tr(k))
        .or_else(|| code_explain::tr(k))
        .or_else(|| code_help::tr(k))
        .or_else(|| kanban::tr(k))
        .or_else(|| kanban_back::tr(k))
        .or_else(|| kanban_front::tr(k))
        .or_else(|| kanban_views::tr(k))
        .or_else(|| planner::tr(k))
        .or_else(|| kanban_links::tr(k))
}

/// Os «Power-Ups» do Kanban (W11): as ligações do cartão e a nota ↔ cartão.
#[path = "kanban_links.rs"]
mod kanban_links;

/// A família pessoal do Kanban (W9): Calendário, Planner, «My cards», notificações.
#[path = "planner.rs"]
mod planner;

/// A frente do cartão do Kanban (W3) — declarada aqui, não no `lib.rs` (o tecto de LOC dele).
#[path = "kanban_front.rs"]
mod kanban_front;

/// As vistas do Kanban (W7): Tabela, Calendário, Linha do tempo e Painel.
#[path = "kanban_views.rs"]
mod kanban_views;
