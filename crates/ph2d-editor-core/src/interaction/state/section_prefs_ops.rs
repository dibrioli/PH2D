//! ⭐⭐ **O TEMA, a ORDEM e o ARRASTO das secções de um painel** (ordem do dono, 2026-09-29).
//!
//! Duas decisões do artista sobre uma secção, e o gesto que muda uma delas:
//!
//! - **O TEMA** — *«com o botão direito do mouse sobre o título da seção poderemos escolher o theme
//!   da seção entre os themes disponíveis para o app»*. Ausente = o tema do app. Substitui o círculo
//!   de cor, que só pintava o próprio círculo.
//! - **A ORDEM** — *«um ícone de 10 pontos que serve para arrastar e reorganizar as seções»*. A
//!   lista guarda só o que o artista MOVEU; o resto segue a ordem natural do painel.
//! - **O ARRASTO** em curso, semeado pelo Down primário na pega.
//!
//! ⚠️ **As duas leis de ordem são PURAS e vivem aqui, e não no painel**: quem guarda a lista é o
//! store, e uma lei que o store guarda e o painel aplica tem de ter UM dono, senão o painel que a
//! pinta e o despacho que a muda discordam sobre onde uma secção fica.

use super::WidgetStore;
use ph2d_a11y::NodeId;

mod tema_custom;
use ph2d_tokens::Theme;
use std::collections::BTreeMap;

/// O estado das secções — um campo só no [`WidgetStore`] (o `mod.rs` dele está no tecto de LOC).
#[derive(Clone, Debug, Default)]
pub struct SectionPrefs {
    themes: BTreeMap<NodeId, Theme>,
    order: Vec<NodeId>,
    drag: Option<SectionDrag>,
    /// ⭐ O arrasto de uma NOTA pela pega dela (2026-09-30) — mora aqui porque é o mesmo gesto
    /// sobre o mesmo painel, e o `mod.rs` do store está no tecto de LOC. Ver [`super::notes_ops`].
    pub(super) note_drag: Option<super::notes_ops::NoteDrag>,
    /// ⭐ O TEMA CUSTOM guardado — a combinação de temas das secções que o artista gravou com
    /// *Save Custom Theme* (2026-09-30). `None` = nunca gravou; `Some(vazio)` = gravou «todas no
    /// tema do app», que é uma combinação legítima. Ver [`tema_custom`].
    custom: Option<BTreeMap<NodeId, Theme>>,
    /// Mudou algo que se GRAVA (tema ou ordem) desde a última vez que a shell perguntou.
    dirty: bool,
}

/// ⭐ **Um arrasto de secção em curso** — a secção, onde o Down aconteceu e onde o cursor está.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SectionDrag {
    pub section: NodeId,
    /// Onde a mão pegou — o ponto do cartão que o FANTASMA mantém debaixo do cursor.
    pub down_x: f32,
    pub down_y: f32,
    pub cursor_x: f32,
    pub cursor_y: f32,
    /// Vira `true` depois do limiar — antes disso um Down+Up na pega não reordena nada.
    pub active: bool,
}

impl WidgetStore {
    /// O tema que o artista escolheu para esta secção; `None` = o do app.
    #[must_use]
    pub fn section_theme(&self, section: NodeId) -> Option<Theme> {
        self.section_prefs.themes.get(&section).copied()
    }

    /// Escolhe (ou, com `None`, devolve ao app) o tema de uma secção.
    pub fn set_section_theme(&mut self, section: NodeId, theme: Option<Theme>) {
        let before = self.section_theme(section);
        match theme {
            Some(t) => {
                self.section_prefs.themes.insert(section, t);
            }
            None => {
                self.section_prefs.themes.remove(&section);
            }
        }
        if before != theme {
            self.section_prefs.dirty = true;
        }
    }

    /// Todos os temas escolhidos — o que a shell grava.
    #[must_use]
    pub fn section_themes(&self) -> &BTreeMap<NodeId, Theme> {
        &self.section_prefs.themes
    }

    /// A ordem que o artista autorou (só as secções que ele moveu, e as que estavam à vista então).
    #[must_use]
    pub fn section_order(&self) -> &[NodeId] {
        &self.section_prefs.order
    }

    /// Instala uma ordem — pelo painel (depois de uma queda) ou pela shell (ao abrir).
    pub fn set_section_order(&mut self, order: Vec<NodeId>) {
        if self.section_prefs.order != order {
            self.section_prefs.order = order;
            self.section_prefs.dirty = true;
        }
    }

    /// Instala o que a shell LEU do disco, sem marcar nada para gravar.
    pub fn restore_section_prefs(&mut self, order: Vec<NodeId>, themes: BTreeMap<NodeId, Theme>) {
        self.section_prefs.order = order;
        self.section_prefs.themes = themes;
        self.section_prefs.dirty = false;
    }

    /// `true` uma vez por mudança gravável — quem pergunta é a persistência da shell.
    pub fn take_section_prefs_dirty(&mut self) -> bool {
        std::mem::take(&mut self.section_prefs.dirty)
    }

    /// O arrasto em curso, se houver.
    #[must_use]
    pub fn section_drag(&self) -> Option<SectionDrag> {
        self.section_prefs.drag
    }

    /// Down primário na pega de `section`.
    pub fn begin_section_drag(&mut self, section: NodeId, x: f32, y: f32) {
        self.section_prefs.drag = Some(SectionDrag {
            section,
            down_x: x,
            down_y: y,
            cursor_x: x,
            cursor_y: y,
            active: false,
        });
    }

    /// Avança o cursor; vira `active` depois do limiar das abas ([`super::TAB_DRAG_THRESHOLD_PX`]
    /// — *a mesma pergunta*: quanto tem a mão de andar para um clique virar um arrasto).
    pub fn update_section_drag(&mut self, x: f32, y: f32) {
        if let Some(d) = self.section_prefs.drag.as_mut() {
            d.cursor_x = x;
            d.cursor_y = y;
            if (y - d.down_y).abs() > super::TAB_DRAG_THRESHOLD_PX {
                d.active = true;
            }
        }
    }

    /// Tira o arrasto (no Up).
    pub fn end_section_drag(&mut self) -> Option<SectionDrag> {
        self.section_prefs.drag.take()
    }
}

/// ⭐⭐ **A ordem em que as secções são PINTADAS** — as que o artista moveu na ordem dele, e cada
/// uma das outras logo a seguir à vizinha que a precede na ordem NATURAL.
///
/// `natural` é a ordem do painel (a da paleta); `stored` é a lista autorada. ⚠️ **Uma secção que
/// o artista nunca viu nasce onde nasceria** — ao lado da vizinha natural dela, e não no fim: uma
/// secção nova que aparecesse sempre por baixo de todas repetiria o defeito que pôs as Tags em
/// 38.º lugar (2026-09-21).
#[must_use]
pub fn ordena_seccoes(natural: &[NodeId], stored: &[NodeId]) -> Vec<NodeId> {
    let pos = |id: &NodeId| stored.iter().position(|s| s == id);
    let mut out: Vec<NodeId> = natural
        .iter()
        .copied()
        .filter(|id| pos(id).is_some())
        .collect();
    out.sort_by_key(|id| pos(id));
    for (i, id) in natural.iter().enumerate() {
        if pos(id).is_some() {
            continue;
        }
        let at = match i
            .checked_sub(1)
            .and_then(|p| out.iter().position(|o| *o == natural[p]))
        {
            Some(p) => p + 1,
            None => 0,
        };
        out.insert(at, *id);
    }
    out
}

/// ⭐⭐ **A lista nova depois de uma queda** — `dragged` sai de onde estava em `displayed` e entra
/// antes de `before` (ou no fim). As secções que não estavam à vista guardam a posição que tinham,
/// DEPOIS das que estavam.
///
/// ⚠️ Largar uma secção antes de si própria, ou antes da seguinte, não muda a ordem — mas grava-a
/// na mesma, e isso é inofensivo (a lista resultante pinta o mesmo).
#[must_use]
pub fn reordena_seccoes(
    displayed: &[NodeId],
    stored: &[NodeId],
    dragged: NodeId,
    before: Option<NodeId>,
) -> Vec<NodeId> {
    let mut vis: Vec<NodeId> = displayed
        .iter()
        .copied()
        .filter(|id| *id != dragged)
        .collect();
    let at = before
        .and_then(|b| vis.iter().position(|v| *v == b))
        .unwrap_or(vis.len());
    vis.insert(at, dragged);
    vis.extend(stored.iter().copied().filter(|s| !displayed.contains(s)));
    vis
}

/// ⭐⭐ **O que se GRAVA das secções** — a ordem autorada e os temas, em `chave=valor`.
///
/// ⚠️ **O formato mora aqui e o ficheiro mora na shell** (`~/.ph2d/sections.txt`): a lei de ler e
/// escrever é do dono do estado, e a shell só sabe onde fica o disco. Os ids são os `NodeId` das
/// secções — hashes de NOMES estáveis (`hash_node_id`), logo iguais entre builds.
#[must_use]
pub fn section_prefs_text(store: &WidgetStore) -> String {
    use std::fmt::Write as _;
    let mut s = String::new();
    if !store.section_order().is_empty() {
        let ids: Vec<String> = store
            .section_order()
            .iter()
            .map(|n| n.0.to_string())
            .collect();
        let _ = writeln!(s, "order={}", ids.join(","));
    }
    for (id, t) in store.section_themes() {
        let _ = writeln!(s, "theme.{}={}", id.0, t.id());
    }
    tema_custom::escreve(store, &mut s);
    s
}

/// Instala o que [`section_prefs_text`] escreveu. **Toda linha que não se entende é saltada** —
/// a mesma tolerância do `layout.txt`, que é o que dispensa uma versão no ficheiro.
pub fn restore_section_prefs_text(store: &mut WidgetStore, text: &str) {
    let mut order = Vec::new();
    let mut themes = BTreeMap::new();
    let mut custom = tema_custom::Leitura::default();
    for line in text.lines() {
        let Some((k, v)) = line.trim().split_once('=') else {
            continue;
        };
        if k == "order" {
            order = v
                .split(',')
                .filter_map(|t| t.trim().parse().ok())
                .map(NodeId)
                .collect();
            order.dedup();
        } else if let Some(id) = k.strip_prefix("theme.")
            && let (Ok(id), Some(t)) = (id.parse(), Theme::from_id(v.trim()))
        {
            themes.insert(NodeId(id), t);
        } else {
            custom.linha(k, v);
        }
    }
    store.restore_section_prefs(order, themes);
    store.section_prefs.custom = custom.fim();
}

/// ⭐⭐ **Onde uma secção arrastada cai** — antes da primeira secção cujo cabeçalho tem o meio
/// ABAIXO do cursor; `None` = no fim.
///
/// `headers` é `(secção, meio do cabeçalho em y)` pela ordem PINTADA. ⚠️ **Uma lei, dois leitores:**
/// o despacho (que aplica a queda no Up) e o painel (que desenha a marca de onde ela vai cair).
/// Duas contas acabariam com a marca num sítio e a secção noutro.
#[must_use]
pub fn alvo_da_queda(headers: &[(NodeId, f32)], dragged: NodeId, y: f32) -> Option<NodeId> {
    headers
        .iter()
        .find(|(id, meio)| *id != dragged && *meio > y)
        .map(|(id, _)| *id)
}

/// ⭐ **As secções que não se arrastam** — o Nome e a Visibilidade, as duas fileiras sem
/// cabeçalho do topo do Inspector (no Blender o nome também não é um painel: é o topo da região).
pub const SECCOES_FIXAS: [NodeId; 2] = [
    crate::ids::INSP_LIVE_NAME_SECTION,
    crate::ids::INSP_LIVE_VISIBILITY_SECTION,
];

#[cfg(test)]
mod tests {
    use super::*;

    const A: NodeId = NodeId(1);
    const B: NodeId = NodeId(2);
    const C: NodeId = NodeId(3);
    const D: NodeId = NodeId(4);

    /// Sem nada autorado a ordem é a natural. *Mutação: ordenar sempre ⇒ nada muda aqui, mas
    /// o gate seguinte sangra.*
    #[test]
    fn sem_lista_a_ordem_e_a_natural() {
        assert_eq!(ordena_seccoes(&[A, B, C], &[]), vec![A, B, C]);
    }

    /// ⭐ A ordem autorada manda, e a secção que o artista nunca viu nasce LOGO A SEGUIR à vizinha
    /// que a precede na ordem natural — não no fim. *Mutação: inserir as desconhecidas no fim ⇒
    /// `[C, A, B, D]`.*
    #[test]
    fn a_desconhecida_nasce_ao_lado_da_vizinha_natural() {
        // o artista pôs C antes de A; B e D nunca estiveram à vista: B segue A, D segue C
        assert_eq!(ordena_seccoes(&[A, B, C, D], &[C, A]), vec![C, D, A, B]);
        // e uma desconhecida na PONTA da ordem natural vai para o topo
        assert_eq!(ordena_seccoes(&[D, A, C], &[C, A]), vec![D, C, A]);
    }

    /// ⭐ Uma queda move UMA secção e as escondidas guardam o lugar depois das visíveis.
    #[test]
    fn a_queda_move_uma_seccao() {
        assert_eq!(reordena_seccoes(&[A, B, C], &[], C, Some(A)), vec![C, A, B]);
        assert_eq!(reordena_seccoes(&[A, B, C], &[], A, None), vec![B, C, A]);
        // D estava autorada e escondida: fica, depois das visíveis
        assert_eq!(
            reordena_seccoes(&[A, B], &[D, B, A], A, None),
            vec![B, A, D]
        );
        // largar antes de si própria não perde a secção
        assert_eq!(reordena_seccoes(&[A, B], &[], A, Some(A)), vec![B, A]);
    }

    /// ⭐ A queda seguida da ordenação pinta o que o artista largou.
    #[test]
    fn a_queda_e_a_ordenacao_concordam() {
        let natural = [A, B, C, D];
        let stored = reordena_seccoes(&natural, &[], D, Some(B));
        assert_eq!(ordena_seccoes(&natural, &stored), vec![A, D, B, C]);
    }

    /// O tema escolhido volta ao app com `None`, e só uma MUDANÇA pede gravação.
    #[test]
    fn o_tema_da_seccao_e_gravavel_e_reversivel() {
        let mut s = WidgetStore::with_capacity(4);
        s.set_section_theme(A, Some(Theme::Light));
        assert_eq!(s.section_theme(A), Some(Theme::Light));
        assert!(s.take_section_prefs_dirty());
        s.set_section_theme(A, Some(Theme::Light));
        assert!(
            !s.take_section_prefs_dirty(),
            "escolher o mesmo nao pede gravacao"
        );
        s.set_section_theme(A, None);
        assert_eq!(s.section_theme(A), None);
        assert!(s.take_section_prefs_dirty());
    }

    /// ⭐ A queda cai antes do primeiro cabeçalho cujo meio está abaixo do cursor, e nunca antes
    /// da própria secção. *Mutação: `<` no lugar de `>` ⇒ cai sempre no topo.*
    #[test]
    fn a_queda_cai_antes_do_primeiro_meio_abaixo_do_cursor() {
        let h = [(A, 10.0), (B, 50.0), (C, 90.0)];
        assert_eq!(alvo_da_queda(&h, C, 30.0), Some(B));
        assert_eq!(alvo_da_queda(&h, C, 5.0), Some(A));
        assert_eq!(alvo_da_queda(&h, A, 95.0), None);
        assert_eq!(
            alvo_da_queda(&h, B, 30.0),
            Some(C),
            "a propria seccao nao e alvo"
        );
    }

    /// ⭐ **O que se grava volta igual** — a ordem com a SEQUÊNCIA intacta e os temas. Uma linha
    /// ilegível é saltada, e ler não pede gravação. *Mutação: ordenar a lista ao gravar ⇒ a
    /// ordem do artista vira a ordem dos hashes.*
    #[test]
    fn o_que_se_grava_volta_igual() {
        let mut a = WidgetStore::with_capacity(4);
        a.set_section_order(vec![C, A, B]);
        a.set_section_theme(B, Some(Theme::Oled));
        let texto = section_prefs_text(&a) + "lixo\ntheme.9=nao_existe\n";
        let mut b = WidgetStore::with_capacity(4);
        restore_section_prefs_text(&mut b, &texto);
        assert_eq!(b.section_order(), &[C, A, B]);
        assert_eq!(b.section_theme(B), Some(Theme::Oled));
        assert_eq!(b.section_themes().len(), 1);
        assert!(
            !b.take_section_prefs_dirty(),
            "ler do disco nao pede gravacao"
        );
    }

    /// O arrasto só vira activo depois do limiar.
    #[test]
    fn o_arrasto_arma_depois_do_limiar() {
        let mut s = WidgetStore::with_capacity(4);
        s.begin_section_drag(A, 0.0, 100.0);
        s.update_section_drag(0.0, 101.0);
        assert!(!s.section_drag().unwrap().active);
        s.update_section_drag(0.0, 100.0 + super::super::TAB_DRAG_THRESHOLD_PX + 1.0);
        assert!(s.section_drag().unwrap().active);
        assert_eq!(s.end_section_drag().unwrap().section, A);
        assert!(s.section_drag().is_none());
    }
}
