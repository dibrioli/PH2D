//! ⭐⭐⭐ **O CORPO DO PAINEL VECTOR COMO UMA LISTA** — pintado pela ordem que o artista escolheu, e
//! cada secção no tema que ele lhe deu.
//!
//! Ordem do dono, 2026-09-30, depois de o menu de temas do título e a pega de dez pontos nascerem
//! só no Inspector: *«siga com os outros painéis»*. O corpo deixou de ser uma SEQUÊNCIA de chamadas
//! (a ordem era o código) e passou a ser uma lista de [`Seccoes`]: cada entrada diz o id da secção,
//! de que ferramenta ela é ([`crate::section_scope`]), de que ABA ela é ([`Onde`], F17) e como se
//! pinta. A lei da ordem, do tema, da marca de queda e do fantasma é a do Inspector, partilhada em
//! [`ph2d_editor_core::panel::section_plan`] — aqui só mora o laço.
//!
//! ⚠️ **O separador entre secções desapareceu do passo, e não se perdeu:** o plano fecha o cartão
//! ANTES de cada secção que se segue a uma que pintou, e no fim ([`section_plan::Corredor`]) — que é
//! exactamente o que o `separator` fazia.
//! Por isso a `path_section` deixou de ser a excepção que fechava sem separador.

use crate::paint_sections::BodyCtx;
use crate::section_scope::Scope;
use crate::state::Aba;
use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::section_plan::{self, Faixa, Fantasma};
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// Uma secção à espera: recebe o corpo e o `y`, e devolve o `y` seguinte.
type Tarefa<'s> = Box<dyn for<'b> FnOnce(&mut BodyCtx<'b>, f32) -> f32 + 's>;

/// ⭐⭐ **De que ABA é uma secção** (F17, dono 16/10). Sem `Default`, como o [`Scope`]: escrever a
/// aba é um acto, e uma secção nova tem de dizer onde vive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Onde {
    /// A geometria, a cor, o texto e os efeitos.
    Drawing,
    /// Componente, moldura, layout, âncoras, pele e estados.
    Interface,
    /// A Posição e o Encaixe — são do OBJECTO, valem nas duas tarefas.
    Ambas,
}

impl Onde {
    fn mostra(self, aba: Aba) -> bool {
        match self {
            Self::Ambas => true,
            Self::Drawing => aba == Aba::Drawing,
            Self::Interface => aba == Aba::Interface,
        }
    }
}

/// Uma entrada da lista: uma secção, ou o CABEÇALHO de um grupo (`grupo_de == Some(id)` nos membros).
struct Item<'s> {
    id: NodeId,
    scope: Scope,
    onde: Onde,
    /// O grupo a que este membro pertence — pinta-se LOGO a seguir ao cabeçalho dele, e só aberto.
    membro_de: Option<NodeId>,
    tarefa: Tarefa<'s>,
}

/// ⭐⭐ **As secções deste quadro, pela ordem NATURAL em que o corpo as declara.**
pub(crate) struct Seccoes<'s> {
    itens: Vec<Item<'s>>,
    /// Os ids dos cabeçalhos de grupo.
    grupos: Vec<NodeId>,
    /// O grupo aberto na DECLARAÇÃO — os `seccao` dentro de [`Seccoes::grupo`] pertencem-lhe.
    a_declarar: Option<NodeId>,
}

impl<'s> Seccoes<'s> {
    pub(crate) fn new() -> Self {
        Self {
            itens: Vec::with_capacity(48),
            grupos: Vec::new(),
            a_declarar: None,
        }
    }

    /// Declara a secção `id`, de quem ela é (`scope`), de que aba (`onde`) e como se pinta. ⚠️ Uma
    /// secção que não se aplica ao que está seleccionado devolve o `y` intacto — o plano lê isso
    /// como «não está à vista» e ela não ocupa faixa nenhuma.
    pub(crate) fn seccao(
        &mut self,
        id: NodeId,
        scope: Scope,
        onde: Onde,
        tarefa: impl for<'b> FnOnce(&mut BodyCtx<'b>, f32) -> f32 + 's,
    ) {
        self.itens.push(Item {
            id,
            scope,
            onde,
            membro_de: self.a_declarar,
            tarefa: Box::new(tarefa),
        });
    }

    /// ⭐⭐ **Um GRUPO dobrável** — o cabeçalho `id` (rótulo `chave`) e as secções que `membros`
    /// declara. Dobrado, nenhum membro pinta; aberto, os membros pintam LOGO a seguir ao cabeçalho,
    /// pela ordem do artista entre eles — um arrasto nunca tira um membro do grupo.
    pub(crate) fn grupo(
        &mut self,
        id: NodeId,
        chave: &'static str,
        onde: Onde,
        membros: impl FnOnce(&mut Self),
    ) {
        self.seccao(id, crate::section_scope::ALWAYS, onde, move |b, y| {
            b.section_header(id, ph2d_i18n::tr(chave), y).0
        });
        self.grupos.push(id);
        self.a_declarar = Some(id);
        membros(self);
        self.a_declarar = None;
    }

    /// Os ids, pela ordem natural — para os gates.
    #[cfg(test)]
    pub(crate) fn ids(&self) -> Vec<NodeId> {
        self.itens.iter().map(|i| i.id).collect()
    }

    /// `(id, aba, grupo)` pela ordem natural — para os gates.
    #[cfg(test)]
    pub(crate) fn mapa(&self) -> Vec<(NodeId, Onde, Option<NodeId>)> {
        self.itens
            .iter()
            .map(|i| (i.id, i.onde, i.membro_de))
            .collect()
    }
}

/// O que o laço carrega de uma secção à seguinte.
struct Passo {
    painel: Theme,
    arrastada: Option<NodeId>,
    faixas: Vec<Faixa>,
    fantasma: Option<VectorScene>,
    corredor: section_plan::Corredor,
}

impl BodyCtx<'_> {
    /// ⭐⭐ **Pinta as secções da ABA aberta pela ordem do artista**, cada uma no tema dela, e devolve
    /// o `y` depois da última. Durante um arrasto, a secção arrastada pinta-se à parte e é reusada
    /// como o FANTASMA que segue o cursor, pintado no fim por cima do corpo.
    pub(crate) fn corre(&mut self, seccoes: Seccoes<'_>, mut y: f32) -> f32 {
        let Seccoes {
            mut itens, grupos, ..
        } = seccoes;
        let natural: Vec<NodeId> = itens.iter().map(|i| i.id).collect();
        let ordem = section_plan::ordem(&natural, self.store);
        let aba = crate::state::aba();
        let seleccao = crate::state::current_selection_count();
        itens.retain(|i| i.onde.mostra(aba) && i.scope.covers(seleccao));
        // Um grupo sem membro à vista não pinta o cabeçalho (seria um título que abre o nada).
        let com_membros: Vec<NodeId> = itens.iter().filter_map(|i| i.membro_de).collect();
        itens.retain(|i| !grupos.contains(&i.id) || com_membros.contains(&i.id));
        let mut passo = Passo {
            painel: self.theme,
            arrastada: self
                .store
                .section_drag()
                .filter(|d| d.active)
                .map(|d| d.section),
            faixas: Vec::with_capacity(ordem.len()),
            fantasma: None,
            corredor: section_plan::Corredor::default(),
        };
        for id in &ordem {
            let Some(i) = itens
                .iter()
                .position(|t| t.id == *id && t.membro_de.is_none())
            else {
                continue;
            };
            let item = itens.swap_remove(i);
            y = self.pinta(item.id, item.tarefa, &mut passo, y);
            if !grupos.contains(id) || self.store.is_collapsed(*id) {
                continue;
            }
            for m in &ordem {
                if let Some(j) = itens
                    .iter()
                    .position(|t| t.id == *m && t.membro_de == Some(*id))
                {
                    let it = itens.swap_remove(j);
                    y = self.pinta(it.id, it.tarefa, &mut passo, y);
                }
            }
        }
        let painel = passo.painel;
        y = passo
            .corredor
            .antes(self.scene, painel, self.inner_x, self.inner_w, y);
        section_plan::conclui(
            self.scene,
            self.store,
            painel,
            &passo.faixas,
            self.inner_x,
            self.inner_w,
            ph2d_editor_core::panel::rows::section_header_h(),
        );
        if let Some(f) = Fantasma::de(
            passo.fantasma,
            &passo.faixas,
            passo.arrastada,
            self.inner_x,
            self.inner_w,
        ) {
            f.pinta(self.scene, self.store);
        }
        y
    }

    /// Uma secção da lista: o cartão, o tema dela e — se é a arrastada — a cena à parte do fantasma.
    fn pinta(&mut self, id: NodeId, tarefa: Tarefa<'_>, passo: &mut Passo, mut y: f32) -> f32 {
        let painel = passo.painel;
        let tema = section_plan::tema_da_seccao(self.store, id, painel);
        y = passo
            .corredor
            .antes(self.scene, painel, self.inner_x, self.inner_w, y);
        let y0 = y;
        self.theme = tema;
        if passo.arrastada == Some(id) {
            // ⭐ A arrastada pinta-se numa cena À PARTE, pousada no sítio de sempre e reusada
            //    como fantasma. ⚠️ Os alvos dela registam-se no sítio real.
            let mut parte = VectorScene::new();
            std::mem::swap(self.scene, &mut parte);
            y = self.uma_seccao(y, tarefa);
            if y > y0 {
                y = self.cromo(id, y0, y);
            }
            std::mem::swap(self.scene, &mut parte);
            self.scene.inner_mut().append(parte.inner(), None);
            passo.fantasma = Some(parte);
        } else {
            y = self.uma_seccao(y, tarefa);
            if y > y0 {
                y = self.cromo(id, y0, y);
            }
        }
        self.theme = painel;
        passo.corredor.depois(y0, y);
        if y > y0 {
            passo.faixas.push((id, y0, y, tema));
        }
        y
    }

    /// ⭐⭐ **O cromo de uma secção que pintou** — as notas dela e o contorno à volta das duas, pela
    /// porta partilhada ([`ph2d_editor_core::widget::showcase::notes_chrome::fecha_seccao`],
    /// 2026-10-01: até aqui o contorno escolhido no menu do título não se pintava neste painel).
    fn cromo(&mut self, id: NodeId, y0: f32, y: f32) -> f32 {
        ph2d_editor_core::widget::showcase::notes_chrome::fecha_seccao(
            self.scene,
            self.text_system,
            self.hit_index,
            self.store,
            ph2d_editor_core::ids::VECTOR_PANEL,
            id,
            self.inner_x,
            self.inner_w,
            y0,
            y,
        )
    }

    /// Uma secção: abre e fecha a dobra dela pela mesma porta. ⚠️ **LIMPA ANTES, e não é
    /// higiene:** uma secção condicional sai sem chamar o `section_header`, e sem isto ela herdaria
    /// a dobra da anterior — que já foi fechada.
    fn uma_seccao(&mut self, y: f32, tarefa: Tarefa<'_>) -> f32 {
        self.open_fold = None;
        let after = tarefa(self, y);
        self.close_fold(after)
    }
}

/// ⭐⭐⭐ **A ORDEM NATURAL do corpo — e a única porta por onde uma secção entra nele.**
///
/// ⭐ **A ordem do Figma** (F17, dono 16/10 — `28_plano_F17_abas_desenho_interface.md` §3): uma só
/// para todo objecto; cada aba é um FILTRO dela, e o que não se aplica some sem ninguém mudar de
/// lugar. Componente → Posição (+ Constraints) → contêiner → Aparência → Preenchimento → Traço →
/// Texto → Efeitos → ▸ Advanced → comportamento. O artista move por cima dela, e uma secção que ele
/// nunca moveu nasce ao lado da vizinha natural ([`ph2d_editor_core::interaction::ordena_seccoes`]);
/// a época [`VECTOR_ORDER_EPOCH`] esquece UMA vez a ordem gravada antes desta.
///
/// ⭐ **Toda secção DECLARA de quem é** — o escopo ([`crate::section_scope`], report do Enio de
/// 2026-08-31: *"deixar no painel apenas o que é útil para a ferramenta em uso"*) e a aba ([`Onde`]).
pub(crate) const COLLAPSED_BY_DEFAULT: [NodeId; 5] = [
    // F16 (15/10): o Snap voltou ao painel RECOLHIDO — a catraca da altura de abertura
    // (`a_altura_de_abertura_de_um_painel_so_encolhe`) mediu +140 px com ele aberto.
    ph2d_tool_vector::ids::VECTOR_SECTION_SNAP,
    // ▸ Advanced (F17): o grupo inteiro nasce dobrado; e a Boolean dentro dele, como antes.
    crate::ids::VECTOR_SECTION_ADVANCED,
    ph2d_tool_vector::ids::VECTOR_SECTION_BOOLEAN,
    // Na aba Interface as de UI abrem (o artista foi lá para elas) — salvo as duas maiores: a
    // States tem quatro linhas `Rec` e onze curvas (foto `antes_01`), a Morph States a máquina.
    crate::ids::VECTOR_SECTION_STATES,
    crate::ids::VECTOR_SECTION_MORPH_STATES,
];

/// ⭐ **A época da ordem natural** — sobe quando a ordem MUDA DE DESENHO (não quando uma secção
/// nova entra: essa nasce ao lado da vizinha sozinha). `1` = a ordem do Figma (F17).
pub(crate) const VECTOR_ORDER_EPOCH: u32 = 1;

pub(crate) fn corpo<'s>(snap: &'s ph2d_tool_vector::VectorStyleSnapshot) -> Seccoes<'s> {
    #[allow(clippy::wildcard_imports)]
    use crate::ids::*;
    use crate::section_scope as scope;
    #[allow(clippy::wildcard_imports)]
    use ph2d_tool_vector::ids::*;
    use Onde::{Ambas, Drawing, Interface};
    let mut p = Seccoes::new();
    // ⛔ D10: `Tool` → coluna ①; o catálogo `Shape` → flyout da ①; `Pencil` e `Symmetry` →
    // fila ②; `Path` → ② (no Node) e ④. ⛔ O Alinhar e o Arrange (15/10) → barra ④.
    // ① O COMPONENTE em cima (Figma UI3: «component controls deserved top billing»).
    p.seccao(VECTOR_SECTION_COMPONENT, scope::ALWAYS, Interface, |b, y| {
        b.component_section(y)
    });
    // ② A POSIÇÃO — o Transform, as Constraints (só para o filho de uma moldura) e o encaixe.
    p.seccao(VECTOR_SECTION_TRANSFORM, scope::ALWAYS, Ambas, |b, y| {
        b.transform_section(y)
    });
    p.seccao(VECTOR_SECTION_ANCHORS, scope::ALWAYS, Interface, |b, y| {
        b.anchors_section(y)
    });
    p.seccao(VECTOR_SECTION_SNAP, scope::ALWAYS, Ambas, |b, y| b.snap_section(y));
    // ③ O CONTÊINER e o LAYOUT (Figma: «aligns with how products are built in code»).
    p.seccao(VECTOR_SECTION_FRAME, scope::ALWAYS, Interface, |b, y| {
        b.frame_section(y)
    });
    p.seccao(VECTOR_SECTION_CLIP, scope::ALWAYS, Interface, |b, y| {
        b.clip_section(y)
    });
    p.seccao(VECTOR_SECTION_LAYOUT, scope::ALWAYS, Interface, |b, y| {
        b.layout_section(y)
    });
    // ④ A APARÊNCIA, a TINTA e o TRAÇO.
    p.seccao(VECTOR_SECTION_APPEARANCE, scope::ALWAYS, Drawing, |b, y| {
        b.appearance_section(y)
    });
    p.seccao(VECTOR_SECTION_FILL, scope::ALWAYS, Drawing, |b, y| {
        b.fill_style(snap, y)
    });
    p.seccao(VECTOR_SECTION_FILL_TYPE, scope::ALWAYS, Drawing, |b, y| {
        b.fill_type_section(y)
    });
    p.seccao(VECTOR_SECTION_TEXPAT, scope::ALWAYS, Drawing, |b, y| {
        b.texture_pattern_section(0, y)
    });
    p.seccao(VECTOR_SECTION_STROKE, scope::ALWAYS, Drawing, |b, y| {
        b.stroke_style(snap, y)
    });
    p.seccao(VECTOR_SECTION_TEXPAT_STROKE, scope::ALWAYS, Drawing, |b, y| {
        b.texture_pattern_section(1, y)
    });
    p.seccao(VECTOR_SECTION_BRUSH, scope::ALWAYS, Drawing, |b, y| {
        b.brush_section(y)
    });
    p.seccao(VECTOR_SECTION_CONNECTOR, scope::ALWAYS, Drawing, |b, y| {
        b.connector_section(y)
    });
    // ⑤ O TEXTO (cada uma some sozinha quando a seleção não é texto).
    p.seccao(VECTOR_SECTION_TEXT, scope::ALWAYS, Drawing, |b, y| {
        b.text_section(y)
    });
    p.seccao(VECTOR_SECTION_FONT, scope::ALWAYS, Drawing, |b, y| {
        b.font_section(y)
    });
    p.seccao(VECTOR_SECTION_PARAGRAPH, scope::ALWAYS, Drawing, |b, y| {
        b.paragraph_section(y)
    });
    p.seccao(VECTOR_SECTION_TEXTPATH, scope::ALWAYS, Drawing, |b, y| {
        b.textpath_section(y)
    });
    p.seccao(VECTOR_SECTION_AXES, scope::ALWAYS, Drawing, |b, y| {
        b.axes_section(y)
    });
    p.seccao(VECTOR_SECTION_VERTEX, scope::ALWAYS, Drawing, |b, y| {
        b.vertex_section(y)
    });
    // ⑥ Os EFEITOS.
    p.seccao(VECTOR_SECTION_EFFECTS, scope::ALWAYS, Drawing, |b, y| {
        b.effects_section(y)
    });
    p.seccao(VECTOR_SECTION_FILTERS, scope::ALWAYS, Drawing, |b, y| {
        b.filters_section(y)
    });
    // ⑦ ▸ ADVANCED — o desenho raro, num grupo recolhido (os verbos booleanos frequentes estão na ④).
    p.grupo(
        VECTOR_SECTION_ADVANCED,
        "panel.vector.section.advanced",
        Drawing,
        |p| {
            p.seccao(VECTOR_SECTION_EXPAND, scope::WHEN_SELECTED, Drawing, |b, y| {
                b.expand_section(y)
            });
            p.seccao(VECTOR_SECTION_BLEND, scope::ALWAYS, Drawing, |b, y| {
                b.blend_section(snap, y)
            });
            p.seccao(VECTOR_SECTION_MORPH, scope::ALWAYS, Drawing, |b, y| {
                b.morph_section(y)
            });
            p.seccao(VECTOR_SECTION_ENVELOPE, scope::WHEN_SELECTED, Drawing, |b, y| {
                b.envelope_section(y)
            });
            p.seccao(VECTOR_SECTION_PATTERNPATH, scope::ALWAYS, Drawing, |b, y| {
                b.patternpath_section(y)
            });
            p.seccao(VECTOR_SECTION_CONTOUR, scope::ALWAYS, Drawing, |b, y| {
                b.contour_section(y)
            });
            p.seccao(VECTOR_SECTION_BOOLEAN, scope::WHEN_SELECTED, Drawing, |b, y| {
                b.boolean_section(y)
            });
        },
    );
    // ⑧ O COMPORTAMENTO (o Prototype do Figma): a pele e os estados.
    p.seccao(VECTOR_SECTION_WIDGET, scope::ALWAYS, Interface, |b, y| {
        b.widget_skin_section(y)
    });
    p.seccao(VECTOR_SECTION_STATES, scope::ALWAYS, Interface, |b, y| {
        b.ui_states_section(y)
    });
    p.seccao(VECTOR_SECTION_MORPH_STATES, scope::ALWAYS, Interface, |b, y| {
        b.morph_states_section(y)
    });
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐⭐ **A ordem natural é a do Figma** (F17, dono 16/10 — `28_plano_F17…` §3): Componente →
    /// Posição (+ Constraints) → contêiner → Aparência → Preenchimento → Traço → Texto → Efeitos →
    /// Advanced → comportamento. *Mutação: devolver a ordem da história ⇒ vermelho.*
    #[test]
    fn a_ordem_natural_e_a_do_figma() {
        #[allow(clippy::wildcard_imports)]
        use crate::ids::*;
        #[allow(clippy::wildcard_imports)]
        use ph2d_tool_vector::ids::*;
        let snap = ph2d_tool_vector::VectorStyleSnapshot::default();
        let esperada = vec![
            VECTOR_SECTION_COMPONENT,
            VECTOR_SECTION_TRANSFORM,
            VECTOR_SECTION_ANCHORS,
            VECTOR_SECTION_SNAP,
            VECTOR_SECTION_FRAME,
            VECTOR_SECTION_CLIP,
            VECTOR_SECTION_LAYOUT,
            VECTOR_SECTION_APPEARANCE,
            VECTOR_SECTION_FILL,
            VECTOR_SECTION_FILL_TYPE,
            VECTOR_SECTION_TEXPAT,
            VECTOR_SECTION_STROKE,
            VECTOR_SECTION_TEXPAT_STROKE,
            VECTOR_SECTION_BRUSH,
            VECTOR_SECTION_CONNECTOR,
            VECTOR_SECTION_TEXT,
            VECTOR_SECTION_FONT,
            VECTOR_SECTION_PARAGRAPH,
            VECTOR_SECTION_TEXTPATH,
            VECTOR_SECTION_AXES,
            VECTOR_SECTION_VERTEX,
            VECTOR_SECTION_EFFECTS,
            VECTOR_SECTION_FILTERS,
            VECTOR_SECTION_ADVANCED,
            VECTOR_SECTION_EXPAND,
            VECTOR_SECTION_BLEND,
            VECTOR_SECTION_MORPH,
            VECTOR_SECTION_ENVELOPE,
            VECTOR_SECTION_PATTERNPATH,
            VECTOR_SECTION_CONTOUR,
            VECTOR_SECTION_BOOLEAN,
            VECTOR_SECTION_WIDGET,
            VECTOR_SECTION_STATES,
            VECTOR_SECTION_MORPH_STATES,
        ];
        assert_eq!(corpo(&snap).ids(), esperada);
    }

    /// ⭐ **Cada secção vive na aba certa, e os sete raros no grupo Advanced** (`28_plano_F17…` §3).
    /// *Mutação: pôr a Layout em `Drawing`, ou o Blend fora do grupo ⇒ vermelho.*
    #[test]
    fn cada_seccao_vive_na_aba_certa() {
        #[allow(clippy::wildcard_imports)]
        use crate::ids::*;
        #[allow(clippy::wildcard_imports)]
        use ph2d_tool_vector::ids::*;
        let snap = ph2d_tool_vector::VectorStyleSnapshot::default();
        let mapa = corpo(&snap).mapa();
        let interface = [
            VECTOR_SECTION_COMPONENT,
            VECTOR_SECTION_ANCHORS,
            VECTOR_SECTION_FRAME,
            VECTOR_SECTION_CLIP,
            VECTOR_SECTION_LAYOUT,
            VECTOR_SECTION_WIDGET,
            VECTOR_SECTION_STATES,
            VECTOR_SECTION_MORPH_STATES,
        ];
        let ambas = [VECTOR_SECTION_TRANSFORM, VECTOR_SECTION_SNAP];
        let avancado = [
            VECTOR_SECTION_EXPAND,
            VECTOR_SECTION_BLEND,
            VECTOR_SECTION_MORPH,
            VECTOR_SECTION_ENVELOPE,
            VECTOR_SECTION_PATTERNPATH,
            VECTOR_SECTION_CONTOUR,
            VECTOR_SECTION_BOOLEAN,
        ];
        for (id, onde, grupo) in mapa {
            let esperada = if interface.contains(&id) {
                Onde::Interface
            } else if ambas.contains(&id) {
                Onde::Ambas
            } else {
                Onde::Drawing
            };
            assert_eq!(onde, esperada, "a secção {id:?} está na aba errada");
            let g = avancado
                .contains(&id)
                .then_some(VECTOR_SECTION_ADVANCED);
            assert_eq!(grupo, g, "a secção {id:?} está no grupo errado");
        }
    }

    /// ⭐⭐ **A lista do corpo é EXACTAMENTE as secções do painel** — nem uma a menos (nunca seria
    /// pintada), nem uma repetida (duas faixas com o mesmo id, uma delas sem pega), nem uma de
    /// fora (uma pega a mudar a ordem de algo que não se pinta aqui).
    /// *Mutação: apagar a linha da `Path` da lista ⇒ a secção some do painel e o gate acusa-a.*
    #[test]
    fn a_lista_do_corpo_e_toda_seccao_do_painel() {
        let snap = ph2d_tool_vector::VectorStyleSnapshot::default();
        let mut lista = corpo(&snap).ids();
        let n = lista.len();
        lista.sort_by_key(|id| id.0);
        lista.dedup();
        assert_eq!(
            lista.len(),
            n,
            "uma secção aparece duas vezes na lista do corpo"
        );
        let mut painel: Vec<NodeId> = crate::ids::VECTOR_SECTIONS.to_vec();
        painel.sort_by_key(|id| id.0);
        assert_eq!(
            lista, painel,
            "a lista do corpo e as secções do painel divergiram"
        );
    }
}
