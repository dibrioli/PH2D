//! A página ao leitor de ecrã (`docs/CodeEditor/04_pagina_de_ajuda.md` §5): um `Document` com o
//! título; filhos o ‹ e o › (`Button`), o índice (`List`) e os blocos À VISTA — `Heading` (título,
//! secção, assinatura), `Paragraph` (prosa e exemplos) com os `Link` e o «Copy example» dentro. O
//! link do Tab é o `active_descendant`. Os ids derivam do bloco e do número do link: rolar troca
//! os nós e um bloco que fica à vista mantém o seu.

use super::layout::ParaKind;
use super::view::HelpView;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;

/// O bloco das ids da página, longe dos do editor (`popup`, `hint`, `console`, `status`).
const A11Y_HELP: u64 = (1 << 31) | (1 << 26);
/// Cada espécie de nó tem uma faixa de `2^20` ids dentro do bloco.
const SPAN: u64 = 1 << 20;
const BACK: u64 = 0;
const FORWARD: u64 = 1;
const LIST: u64 = 2;
const ROWS: u64 = SPAN;
const PARAS: u64 = 2 * SPAN;
const LINKS: u64 = 3 * SPAN;
const COPIES: u64 = 4 * SPAN;

fn id(base: NodeId, kind: u64, i: u64) -> NodeId {
    debug_assert!(i < SPAN, "a faixa de ids da página transborda");
    NodeId(base.0 + A11Y_HELP + kind + i)
}

fn bounds(b: NodeBuilder, r: Rect) -> NodeBuilder {
    b.bounds(
        f64::from(r.x),
        f64::from(r.y),
        f64::from(r.w),
        f64::from(r.h),
    )
}

fn button(label: &str, r: Rect, on: bool) -> Node {
    let mut n = bounds(NodeBuilder::new(Role::Button).label(label), r)
        .action(Action::Click)
        .build();
    if !on {
        n.set_disabled();
    }
    n
}

impl HelpView {
    /// O id do nó do link `n` (o `active_descendant` quando tem o Tab).
    #[must_use]
    pub fn a11y_link_id(&self, base: NodeId, n: u32) -> NodeId {
        id(base, LINKS, u64::from(n))
    }

    /// Os nós do último quadro, o `Document` (`base`) primeiro.
    #[must_use]
    pub fn a11y(&self, base: NodeId) -> Vec<(NodeId, Node)> {
        let mut out = Vec::new();
        let mut kids = vec![id(base, BACK, 0), id(base, FORWARD, 0)];
        out.push((
            kids[0],
            button(tr("code.help.back"), self.back.0, self.back.1),
        ));
        out.push((
            kids[1],
            button(tr("code.help.forward"), self.forward.0, self.forward.1),
        ));
        if !self.rows.is_empty() {
            let rows: Vec<NodeId> = (0..self.rows.len())
                .map(|i| id(base, ROWS, i as u64))
                .collect();
            let list = bounds(
                NodeBuilder::new(Role::List).label(tr("code.help.sections")),
                self.index,
            );
            out.push((
                id(base, LIST, 0),
                list.children(rows.iter().copied()).build(),
            ));
            kids.push(id(base, LIST, 0));
            for (i, ((r, label), &rid)) in self.rows.iter().zip(&self.labels).zip(&rows).enumerate()
            {
                let mut n = bounds(NodeBuilder::new(Role::ListItem).label(label.as_str()), *r)
                    .action(Action::Click)
                    .build();
                n.set_selected(self.section == Some(i));
                out.push((rid, n));
            }
        }
        let mut active = None;
        if let Some((_, l)) = &self.laid {
            let (top, bottom) = (
                self.scroll - super::view::pad(),
                self.scroll - super::view::pad() + self.viewport.h,
            );
            let mut seen: Vec<u32> = Vec::new();
            for (k, p) in l
                .paras
                .iter()
                .enumerate()
                .filter(|(_, p)| p.y + p.h > top && p.y < bottom)
            {
                let pid = id(base, PARAS, k as u64);
                let mut children = Vec::new();
                for r in &l.runs[p.runs.clone()] {
                    let Some(n) = r.link else { continue };
                    if seen.contains(&n) {
                        continue;
                    }
                    seen.push(n);
                    let text: Vec<&str> = l.runs[p.runs.clone()]
                        .iter()
                        .filter(|q| q.link == Some(n))
                        .map(|q| q.text.as_str())
                        .collect();
                    let lid = self.a11y_link_id(base, n);
                    let rect = self.screen(r.x, r.y, r.w, r.h);
                    let node = bounds(NodeBuilder::new(Role::Link).label(text.join(" ")), rect)
                        .action(Action::Click)
                        .build();
                    out.push((lid, node));
                    children.push(lid);
                    if self.focus == Some(n) {
                        active = Some(lid);
                    }
                }
                if let ParaKind::Code(n) = p.kind
                    && let Some(r) = self.target_rect(super::view::HelpHit::Copy(n))
                {
                    let cid = id(base, COPIES, u64::from(n));
                    out.push((cid, button(tr("code.help.copy"), r, true)));
                    children.push(cid);
                }
                let role = match p.kind {
                    ParaKind::Heading(_) => Role::Heading,
                    ParaKind::Text | ParaKind::Code(_) => Role::Paragraph,
                };
                let rect = self.screen(0.0, p.y, self.dims.width, p.h);
                let mut node = bounds(NodeBuilder::new(role).label(&l.plain[p.text.clone()]), rect)
                    .children(children)
                    .build();
                if let ParaKind::Heading(level) = p.kind {
                    node.set_level(usize::from(level));
                }
                out.push((pid, node));
                kids.push(pid);
            }
        }
        let mut doc = bounds(
            NodeBuilder::new(Role::Document).label(self.title.as_str()),
            self.area,
        )
        .children(kids)
        .build();
        if let Some(a) = active {
            doc.set_active_descendant(a.into());
        }
        out.insert(0, (base, doc));
        out
    }
}
