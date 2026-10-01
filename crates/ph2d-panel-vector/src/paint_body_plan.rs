//! ⭐⭐⭐ **O CORPO DO PAINEL VECTOR COMO UMA LISTA** — pintado pela ordem que o artista escolheu, e
//! cada secção no tema que ele lhe deu.
//!
//! Ordem do dono, 2026-09-30, depois de o menu de temas do título e a pega de dez pontos nascerem
//! só no Inspector: *«siga com os outros painéis»*. O corpo deixou de ser uma SEQUÊNCIA de chamadas
//! (a ordem era o código) e passou a ser uma lista de [`Seccoes`]: cada entrada diz o id da secção,
//! de que ferramenta ela é ([`crate::section_scope`]) e como se pinta. A lei da ordem, do tema, da
//! marca de queda e do fantasma é a do Inspector, partilhada em
//! [`ph2d_editor_core::panel::section_plan`] — aqui só mora o laço.
//!
//! ⚠️ **O separador entre secções desapareceu do passo, e não se perdeu:** o plano fecha o cartão
//! ANTES de cada secção que se segue a uma que pintou, e no fim ([`section_plan::Corredor`]) — que é
//! exactamente o que o `separator` fazia.
//! Por isso a `path_section` deixou de ser a excepção que fechava sem separador.

use crate::paint_sections::BodyCtx;
use crate::section_scope::Scope;
use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::section_plan::{self, Faixa, Fantasma};
use ph2d_tool_vector::params::DrawMode;
use ph2d_vector::VectorScene;

/// Uma secção à espera: recebe o corpo e o `y`, e devolve o `y` seguinte.
type Tarefa<'s> = Box<dyn for<'b> FnOnce(&mut BodyCtx<'b>, f32) -> f32 + 's>;

/// ⭐⭐ **As secções deste quadro, pela ordem NATURAL em que o corpo as declara.**
pub(crate) struct Seccoes<'s> {
    mode: DrawMode,
    itens: Vec<(NodeId, Scope, Tarefa<'s>)>,
}

impl<'s> Seccoes<'s> {
    pub(crate) fn new(mode: DrawMode) -> Self {
        Self {
            mode,
            itens: Vec::with_capacity(48),
        }
    }

    /// Declara a secção `id`, de quem ela é (`scope`) e como se pinta. ⚠️ Uma secção que não se
    /// aplica ao que está seleccionado devolve o `y` intacto — o plano lê isso como «não está à
    /// vista» e ela não ocupa faixa nenhuma.
    pub(crate) fn seccao(
        &mut self,
        id: NodeId,
        scope: Scope,
        tarefa: impl for<'b> FnOnce(&mut BodyCtx<'b>, f32) -> f32 + 's,
    ) {
        self.itens.push((id, scope, Box::new(tarefa)));
    }

    /// Os ids, pela ordem natural — para os gates.
    #[cfg(test)]
    pub(crate) fn ids(&self) -> Vec<NodeId> {
        self.itens.iter().map(|(id, _, _)| *id).collect()
    }
}

impl BodyCtx<'_> {
    /// ⭐⭐ **Pinta as secções pela ordem do artista**, cada uma no tema dela, e devolve o `y`
    /// depois da última. Durante um arrasto, a secção arrastada pinta-se à parte e é reusada como o
    /// FANTASMA que segue o cursor, pintado no fim por cima do corpo.
    pub(crate) fn corre(&mut self, seccoes: Seccoes<'_>, mut y: f32) -> f32 {
        let painel = self.theme;
        let Seccoes { mode, mut itens } = seccoes;
        let natural: Vec<NodeId> = itens.iter().map(|(id, _, _)| *id).collect();
        let ordem = section_plan::ordem(&natural, self.store);
        let arrastada = self
            .store
            .section_drag()
            .filter(|d| d.active)
            .map(|d| d.section);
        let seleccao = crate::state::current_selection_count();
        let mut faixas: Vec<Faixa> = Vec::with_capacity(ordem.len());
        let mut fantasma: Option<VectorScene> = None;
        let mut corredor = section_plan::Corredor::default();
        for id in ordem {
            let Some(i) = itens.iter().position(|(t, _, _)| *t == id) else {
                continue;
            };
            let (_, scope, tarefa) = itens.swap_remove(i);
            if !scope.covers(mode, seleccao) {
                continue;
            }
            let tema = section_plan::tema_da_seccao(self.store, id, painel);
            y = corredor.antes(self.scene, painel, self.inner_x, self.inner_w, y);
            let y0 = y;
            self.theme = tema;
            if arrastada == Some(id) {
                // ⭐ A arrastada pinta-se numa cena À PARTE, pousada no sítio de sempre e reusada
                //    como fantasma. ⚠️ Os alvos dela registam-se no sítio real.
                let mut parte = VectorScene::new();
                std::mem::swap(self.scene, &mut parte);
                y = self.uma_seccao(y, tarefa);
                std::mem::swap(self.scene, &mut parte);
                self.scene.inner_mut().append(parte.inner(), None);
                fantasma = Some(parte);
            } else {
                y = self.uma_seccao(y, tarefa);
            }
            self.theme = painel;
            corredor.depois(y0, y);
            if y > y0 {
                faixas.push((id, y0, y, tema));
            }
        }
        y = corredor.antes(self.scene, painel, self.inner_x, self.inner_w, y);
        section_plan::conclui(
            self.scene,
            self.store,
            painel,
            &faixas,
            self.inner_x,
            self.inner_w,
            ph2d_editor_core::panel::rows::section_header_h(),
        );
        if let Some(f) = Fantasma::de(fantasma, &faixas, arrastada, self.inner_x, self.inner_w) {
            f.pinta(self.scene, self.store);
        }
        y
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
/// "O que estou fazendo" (modo · forma · parâmetros da forma) vem ANTES de "com que cor" (stroke ·
/// fill) — o corpo abria em Stroke/Fill e o seletor de ferramenta aparecia depois, invertido. É a
/// ordem com que o painel abre; o artista move por cima dela, e uma secção que ele nunca moveu
/// nasce ao lado da vizinha natural ([`ph2d_editor_core::interaction::ordena_seccoes`]).
///
/// ⭐ **Toda secção DECLARA de quem é** ([`crate::section_scope`], report do Enio de 2026-08-31:
/// *"deixar no painel apenas o que é útil para a ferramenta em uso"*) — o gate
/// `every_body_section_declares_which_tool_it_belongs_to` conta as duas metades.
pub(crate) fn corpo<'s>(snap: &'s ph2d_tool_vector::VectorStyleSnapshot) -> Seccoes<'s> {
    #[allow(clippy::wildcard_imports)]
    use crate::ids::*;
    use crate::section_scope as scope;
    #[allow(clippy::wildcard_imports)]
    use ph2d_tool_vector::ids::*;
    let mut p = Seccoes::new(snap.mode);
    p.seccao(VECTOR_SECTION_TOOL, scope::ALWAYS, |b, y| {
        b.tool_section(snap, y)
    });
    p.seccao(VECTOR_SECTION_SHAPE, scope::CATALOG, |b, y| {
        b.shape_catalog_section(snap, y)
    });
    // O LÁPIS entra entre a fileira TOOL e os parâmetros da forma: os três respondem "o que estou
    // desenhando, e como". Some inteira fora do modo Pencil.
    p.seccao(VECTOR_SECTION_PENCIL, scope::PENCIL, |b, y| {
        b.pencil_section(snap, y)
    });
    // A SIMETRIA é uma OPÇÃO das ferramentas de desenho, então mora ao lado do que decide o que se
    // desenha — e não entre os deformadores, que é onde ela estava quando era um efeito.
    p.seccao(VECTOR_SECTION_SYMMETRY, scope::symmetry(snap), |b, y| {
        b.symmetry_section(snap, y)
    });
    // A MOLDURA e o RECORTE são propriedades do OBJETO que se acabou de desenhar; o AUTO LAYOUT diz
    // o que o contêiner FAZ com o conteúdo, e as ÂNCORAS são a outra metade da mesma pergunta.
    p.seccao(VECTOR_SECTION_FRAME, scope::ALWAYS, |b, y| {
        b.frame_section(y)
    });
    p.seccao(VECTOR_SECTION_CLIP, scope::ALWAYS, |b, y| b.clip_section(y));
    p.seccao(VECTOR_SECTION_LAYOUT, scope::ALWAYS, |b, y| {
        b.layout_section(y)
    });
    p.seccao(VECTOR_SECTION_ANCHORS, scope::ALWAYS, |b, y| {
        b.anchors_section(y)
    });
    // OS COMPONENTES: uma instância é primeiro uma forma (pose, moldura, regra) e só depois uma
    // cópia de outra coisa.
    p.seccao(VECTOR_SECTION_COMPONENT, scope::ALWAYS, |b, y| {
        b.component_section(y)
    });
    p.seccao(VECTOR_SECTION_WIDGET, scope::ALWAYS, |b, y| {
        b.widget_skin_section(y)
    });
    p.seccao(VECTOR_SECTION_STATES, scope::ALWAYS, |b, y| {
        b.ui_states_section(y)
    });
    p.seccao(VECTOR_SECTION_SHAPE_PARAMS, scope::ALWAYS, |b, y| {
        b.shape_params_section(snap, y)
    });
    // O CONECTOR seleccionado é irmã da secção acima: as duas respondem "o que é este objeto que eu
    // seleccionei, e como o afino?".
    p.seccao(VECTOR_SECTION_CONNECTOR, scope::ALWAYS, |b, y| {
        b.connector_section(y)
    });
    p.seccao(VECTOR_SECTION_TEXT, scope::ALWAYS, |b, y| b.text_section(y));
    p.seccao(VECTOR_SECTION_FONT, scope::ALWAYS, |b, y| b.font_section(y));
    p.seccao(VECTOR_SECTION_PARAGRAPH, scope::ALWAYS, |b, y| {
        b.paragraph_section(y)
    });
    // Text on Path fica com as outras secções de TEXTO: o artista procura-a onde afina o texto.
    p.seccao(VECTOR_SECTION_TEXTPATH, scope::ALWAYS, |b, y| {
        b.textpath_section(y)
    });
    p.seccao(VECTOR_SECTION_AXES, scope::ALWAYS, |b, y| b.axes_section(y));
    p.seccao(VECTOR_SECTION_STROKE, scope::ALWAYS, |b, y| {
        b.stroke_style(snap, y)
    });
    // ⭐⭐ A PATTERN do TRAÇO (tinta 1) — IRMÃ da *Stroke*, nunca aninhada (uma dobra não aninha), e
    // a do PINCEL ao lado: as duas descrevem a mesma linha.
    p.seccao(VECTOR_SECTION_TEXPAT_STROKE, scope::ALWAYS, |b, y| {
        b.texture_pattern_section(1, y)
    });
    p.seccao(VECTOR_SECTION_BRUSH, scope::ALWAYS, |b, y| {
        b.brush_section(y)
    });
    p.seccao(VECTOR_SECTION_FILL, scope::ALWAYS, |b, y| {
        b.fill_style(snap, y)
    });
    p.seccao(VECTOR_SECTION_FILL_TYPE, scope::ALWAYS, |b, y| {
        b.fill_type_section(y)
    });
    // A lei do PADRÃO fica colada ao selector que a escolhe (a tinta 0, o preenchimento).
    p.seccao(VECTOR_SECTION_TEXPAT, scope::ALWAYS, |b, y| {
        b.texture_pattern_section(0, y)
    });
    p.seccao(VECTOR_SECTION_SNAP, scope::ALWAYS, |b, y| b.snap_section(y));
    p.seccao(VECTOR_SECTION_TRANSFORM, scope::ALWAYS, |b, y| {
        b.transform_section(y)
    });
    // A APARÊNCIA (opacidade + mistura) descreve a forma seleccionada como um todo, como o
    // Transform.
    p.seccao(VECTOR_SECTION_APPEARANCE, scope::ALWAYS, |b, y| {
        b.appearance_section(y)
    });
    p.seccao(VECTOR_SECTION_VERTEX, scope::ALWAYS, |b, y| {
        b.vertex_section(y)
    });
    p.seccao(VECTOR_SECTION_BOOLEAN, scope::WHEN_SELECTED, |b, y| {
        b.boolean_section(y)
    });
    // Expand é irmã da Boolean: comandos destrutivos sobre a seleção, mesmo motor.
    p.seccao(VECTOR_SECTION_EXPAND, scope::WHEN_SELECTED, |b, y| {
        b.expand_section(y)
    });
    p.seccao(VECTOR_SECTION_BLEND, scope::ALWAYS, |b, y| {
        b.blend_section(snap, y)
    });
    p.seccao(VECTOR_SECTION_MORPH, scope::ALWAYS, |b, y| {
        b.morph_section(y)
    });
    // A MÁQUINA do Morph — secção própria, logo abaixo da que cria o objecto.
    p.seccao(VECTOR_SECTION_MORPH_STATES, scope::ALWAYS, |b, y| {
        b.morph_states_section(y)
    });
    // O Envelope, o Pattern on Path e o Contour são a família da geometria DERIVADA de uma relação
    // viva; Effects é a generalização deles e Filters a irmã de pixels.
    p.seccao(VECTOR_SECTION_ENVELOPE, scope::WHEN_SELECTED, |b, y| {
        b.envelope_section(y)
    });
    p.seccao(VECTOR_SECTION_PATTERNPATH, scope::ALWAYS, |b, y| {
        b.patternpath_section(y)
    });
    p.seccao(VECTOR_SECTION_CONTOUR, scope::ALWAYS, |b, y| {
        b.contour_section(y)
    });
    p.seccao(VECTOR_SECTION_EFFECTS, scope::ALWAYS, |b, y| {
        b.effects_section(y)
    });
    p.seccao(VECTOR_SECTION_FILTERS, scope::ALWAYS, |b, y| {
        b.filters_section(y)
    });
    p.seccao(VECTOR_SECTION_ALIGN, scope::ALWAYS, |b, y| {
        b.align_section(y)
    });
    p.seccao(VECTOR_SECTION_ARRANGE, scope::WHEN_SELECTED, |b, y| {
        b.arrange_section(y)
    });
    // ⭐ A Path deixou de ser a excepção que fechava a dobra à mão: o plano fecha o cartão no fim.
    p.seccao(VECTOR_SECTION_PATH, scope::WHEN_SELECTED, |b, y| {
        b.path_section(y)
    });
    p
}

#[cfg(test)]
mod tests {
    use super::*;

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
