//! **A seção BOOLEAN** do painel — irmã de [`super::paint_sections`] pelo teto de 600 LOC daquele
//! ficheiro, e o corte é o que este directório já pratica trinta vezes: *aqui mora a seção de UM
//! assunto*, e o ficheiro de onde ela saiu volta a ser o que o nome dele diz — a CAIXA DE
//! FERRAMENTAS do corpo (as rows, os cabeçalhos, a dobra) mais a ORDEM em que as seções correm.
//!
//! ⚠️ **Ela saiu daqui em 2026-09-05 porque o vizinho precisava de UMA linha** (a seção
//! *Appearance*), e o teto não se isenta: *a cura de um teto estourado é o corte para um irmão*.
//! A escolha de qual cortar não foi o tamanho — foi a responsabilidade: era a única SEÇÃO a viver
//! no ficheiro das ferramentas.

use ph2d_editor_core::widget::ButtonKind;
use ph2d_i18n::tr;
use ph2d_tokens::Spacing;

use crate::ids;
use crate::paint_sections::BodyCtx;

impl BodyCtx<'_> {
    /// Seção **BOOLEAN** — ops N-árias sobre as regiões fechadas SELECIONADAS (a de trás
    /// é a base, a da frente doa o estilo) + a linha Compound / Release.
    pub(crate) fn boolean_section(&mut self, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_BOOLEAN,
            tr("panel.vector.section.boolean"),
            y,
        );
        if collapsed {
            return y;
        }
        // **O MODO dos oito botões abaixo** (plano UI/UX W1). Acima deles e não abaixo, porque
        // ele decide o que eles FAZEM: `Off` consome os operandos (o mundo de sempre), `On` cria
        // um grupo cujos filhos se combinam e continuam editáveis.
        let live = crate::state::bool_live_on();
        y = self.segmented(
            tr("panel.vector.bool.live"),
            &[
                (
                    ids::VECTOR_BOOL_LIVE_OFF,
                    tr("panel.vector.bool.live.off"),
                    !live,
                ),
                (
                    ids::VECTOR_BOOL_LIVE_ON,
                    tr("panel.vector.bool.live.on"),
                    live,
                ),
            ],
            y,
        );
        // **O VERBO DESTA FORMA** (Enio, 2026-08-22): *"o modo do boolean é escolhido por shape e
        // na ordem em que aparece na hierarquia atua sobre o resultante das operações pregressas"*.
        // É o compound shape vivo do Illustrator, em que cada componente guarda o seu Shape Mode.
        //
        // ⚠️ **A fileira vem ANTES dos oito botões, e não junto deles.** Estes quatro são uma
        // PROPRIEDADE da forma em mãos; os oito são AÇÕES sobre a seleção (criam ou re-miram o
        // grupo). Vizinhos, leriam-se como uma família de doze, e o artista descobriria pelo
        // efeito que quatro mudam uma forma e oito mexem no grupo inteiro.
        //
        // ⚠️ `None` faz a fileira **não existir**, e a regra inteira mora do lado da shell
        // (`vec_bool_shape`) — inclusive as duas recusas que ela carrega: a BASE não tem verbo, e
        // um grupo numa RECEITA não deixa forma nenhuma escolher.
        if let Some((code, name)) = crate::state::bool_shape_row() {
            // ⚠️ **O rótulo NOMEIA a forma de que fala.** Com o grupo inteiro aceso no canvas
            // (tocar um filho seleciona o grupo — lei do editor), um rótulo genérico não diria de
            // QUAL das formas ele fala, e o artista escolheria o verbo no escuro. Sem `Name` no
            // documento cai-se no genérico, que é o melhor que há a dizer.
            let label = if name.is_empty() {
                tr("panel.vector.bool.shape").to_string()
            } else {
                name
            };
            y = self.segmented(
                &label,
                &[
                    (
                        ids::VECTOR_BOOL_SHAPE_UNION,
                        tr("panel.vector.bool.shape.union"),
                        code == 0,
                    ),
                    (
                        ids::VECTOR_BOOL_SHAPE_SUBTRACT,
                        tr("panel.vector.bool.shape.subtract"),
                        code == 1,
                    ),
                    (
                        ids::VECTOR_BOOL_SHAPE_INTERSECT,
                        tr("panel.vector.bool.shape.intersect"),
                        code == 2,
                    ),
                    (
                        ids::VECTOR_BOOL_SHAPE_EXCLUDE,
                        tr("panel.vector.bool.shape.exclude"),
                        code == 3,
                    ),
                ],
                y,
            );
        }
        // **O Apply só existe com uma booleana viva selecionada.** Sem ela não há o que
        // consolidar, e um botão que não aplica nada é pior que botão nenhum — a mesma lei do
        // Apply da simetria e dos dois botões do corte.
        if crate::state::bool_group_selected() {
            y = self.action_button_kind(
                ids::VECTOR_BOOL_APPLY,
                tr("panel.vector.bool.apply"),
                ButtonKind::Accent,
                y,
            );
        }
        // **As OITO** (plano 25 §8): as quatro de conjunto e as quatro receitas. Duas colunas —
        // a fileira de oito botoes de largura cheia empurrava a linha Compound para fora da vista.
        let ops = [
            (ids::VECTOR_BOOL_UNION, tr("panel.vector.bool.union")),
            (ids::VECTOR_BOOL_SUBTRACT, tr("panel.vector.bool.subtract")),
            (
                ids::VECTOR_BOOL_INTERSECT,
                tr("panel.vector.bool.intersect"),
            ),
            (ids::VECTOR_BOOL_EXCLUDE, tr("panel.vector.bool.exclude")),
            (
                ids::VECTOR_BOOL_MINUS_BACK,
                tr("panel.vector.bool.minus_back"),
            ),
            (ids::VECTOR_BOOL_TRIM, tr("panel.vector.bool.trim")),
            (ids::VECTOR_BOOL_CROP, tr("panel.vector.bool.crop")),
            (ids::VECTOR_BOOL_MERGE, tr("panel.vector.bool.merge")),
        ];
        let gap = Spacing::Sm.px();
        let w = ((self.inner_w - gap) / 2.0).max(1.0);
        for pair in ops.chunks(2) {
            let [a, b] = pair else { continue };
            y = self.row2(w, gap, [(a.0, a.1), (b.0, b.1)], y);
        }
        self.compound_row(y)
    }
}
