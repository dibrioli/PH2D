//! **O vocabulário do PAINEL vira estado da tool** — irmão de [`super::tool`] pelo teto de 700 LOC.
//!
//! O corte é por RESPONSABILIDADE, e é o par exato do [`super::adopt`]: lá está o caminho de LER o
//! documento; aqui, o de traduzir o que o painel diz (`SetValue` / `Click` sobre os `VECTOR_*`) no
//! estado AUTORADO da tool. O que a tool *é* fica no `tool.rs`; o que um clique *faz com ela*, aqui.
//!
//! ⚠️ **Módulo FILHO de `tool.rs`, não irmão** (`#[path]` + `use super::*`): os campos que este
//! router escreve são privados do módulo da tool, e alargá-los para `pub(crate)` só para acomodar
//! um split seria pagar em superfície o que o teto de LOC cobra em organização — a mesma lei que o
//! `tool_adopt.rs` já segue.

use super::*;

impl VectorTool {
    /// A porta ÚNICA por onde um evento de painel entra na tool. O `impl Tool` delega para cá.
    pub(super) fn apply_panel_event(&mut self, event: PanelEvent) {
        // Docked-panel control ids are the shared `ph2d_editor_core::ids::VECTOR_*`
        // chrome NodeIds (the panel forwards `SetValue` / `Click` over
        // `ToolPanelEvent`; the swatch colours arrive via the setters above,
        // driven by the picker read-back in `vector_bridge`).
        match event {
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_WIDTH => {
                self.stroke_width_px = slider_to_px(v as f32);
                // Also restyle the selected path (mirror of a colour change), so
                // the width slider affects the path you're looking at — not just
                // the next one drawn.
                self.apply_to_selected = true;
                // ⚠️ E diz que foi a LARGURA que mudou. O bridge precisa das duas metades: a
                // primeira abre o restyle, a segunda decide se a largura entra nele — sem a
                // segunda ele perguntava ao slider se estava em arrasto, e a caixa numérica (que
                // chega por AQUI, pelo mesmo evento) ficava muda sobre a forma selecionada.
                self.width_authored = true;
            }
            // **Os dois knobs do LÁPIS.** Nenhum deles restila a seleção (`apply_to_selected`):
            // eles descrevem como a MÃO é capturada, não como o traço é pintado — mexer neles não
            // pode reescrever uma curva que já está no documento.
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_PENCIL_FIDELITY => {
                self.pencil_fidelity_px = crate::params::slider_to_fidelity_px(v as f32);
            }
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_PENCIL_STABILIZER => {
                self.pencil_stabilizer = (v as f32).clamp(0.0, 1.0);
            }
            // **Campo de forma** — um braço só para TODAS as formas: o id carrega o
            // ÍNDICE do parâmetro no catálogo, e a forma ativa diz o que ele significa.
            // Antes era um braço por parâmetro por forma; com 25 formas seria um pântano.
            PanelEvent::SetValue(id, v) if shape_field_index(id).is_some() => {
                if let Some(i) = shape_field_index(id) {
                    self.set_shape_field(i, v);
                }
            }
            // Opacity sliders own the fill/stroke alpha (the single source). The
            // picker only sets RGB. `0 %` alpha ⇒ invisible (no fill).
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_FILL_OPACITY => {
                self.fill[3] = slider_to_opacity(v as f32);
                self.apply_to_selected = true;
            }
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_STROKE_OPACITY => {
                self.stroke[3] = slider_to_opacity(v as f32);
                self.apply_to_selected = true;
            }
            // Draw-mode segmented row: switches the canvas gesture. No recolour
            // (mode is not a Style change) — the shell reads `mode()` to route.
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_SELECT => self.mode = DrawMode::Select,
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_NODE => self.mode = DrawMode::Node,
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_PEN => self.mode = DrawMode::Pen,
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_WIDTH => {
                self.mode = DrawMode::Width;
            }
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_TRIM => {
                self.mode = DrawMode::Trim;
            }
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_BUCKET => {
                self.mode = DrawMode::Bucket;
            }
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_CUT => {
                self.mode = DrawMode::Cut;
            }
            // **Moldura** (plano UI/UX W0) — o 14º pill. O gesto é o do retângulo; o que ele
            // acrescenta ao soltar é o componente `VecFrame` (a shell o pendura).
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_FRAME => {
                self.mode = DrawMode::Frame;
            }
            // ⭐ **Osso** (estudo 42 item 5) — o 17.º pill. O gesto produz uma ENTIDADE com
            // `VecBone`, não um caminho; a shell é quem a cria (aqui só se troca de modo).
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_BONE => {
                self.mode = DrawMode::Bone;
            }
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_PENCIL => {
                self.mode = DrawMode::Pencil;
            }
            // **A fonte da largura do lápis** (W1d) — três chips exclusivos. Escolher uma NÃO
            // arma o modo Pencil: a seção só é pintada nele, então já se está lá; e trocar de
            // fonte no meio de outro modo seria arrancar o artista do que ele estava a fazer.
            PanelEvent::Click(id) if id == ids::VECTOR_PENCIL_W_UNIFORM => {
                self.pencil_width_source = ph2d_vec_edit::pencil_width::WidthSource::Uniform;
            }
            PanelEvent::Click(id) if id == ids::VECTOR_PENCIL_W_SPEED => {
                self.pencil_width_source = ph2d_vec_edit::pencil_width::WidthSource::Speed;
            }
            PanelEvent::Click(id) if id == ids::VECTOR_PENCIL_W_PRESSURE => {
                self.pencil_width_source = ph2d_vec_edit::pencil_width::WidthSource::Pressure;
            }
            // **A FORMA do marquee** — o par pegajoso. Escolher um NÃO arma o modo Node: a row só
            // é pintada nele, então já se está lá (a mesma lei da fonte de largura acima).
            PanelEvent::Click(id) if id == ids::VECTOR_MARQUEE_BOX => {
                self.marquee = crate::params::MarqueeShape::Box;
            }
            PanelEvent::Click(id) if id == ids::VECTOR_MARQUEE_LASSO => {
                self.marquee = crate::params::MarqueeShape::Lasso;
            }
            // **A SIMETRIA de desenho** (W6.3) — o par que arma o modo e os quatro tipos.
            //
            // ⚠️ O `Enable` fica no TOPO e gateia toda a seção (a lei que o Enio estabeleceu no
            // painel do impasto), então os chips abaixo só são clicáveis com ele ligado: não há
            // como escolher um tipo para um espelho que não existe.
            PanelEvent::Click(id) if id == ids::VECTOR_SYM_OFF => self.symmetry.on = false,
            // **Apply DESARMA** — *"botão apply para consolidar a forma e desativar a simetria"*
            // (Enio). A tool não materializa nada (não vê a cena); ela sabe que o modo acabou, e
            // a shell faz a geometria. Duas metades de um gesto, cada uma na camada que a pode
            // fazer — e o fato `on` continua com UM dono.
            //
            // ⚠️ A ordem dentro do frame é benigna e está medida: o espelho `vec_draw_config` só
            // é reescrito DEPOIS do bloco que materializa, então o `on` que o arm lê nesse frame
            // ainda é `true` (e o re-arm é um no-op, a spec não mudou); no frame seguinte ele lê
            // `false` e desarma uma selecção que já são as formas NOVAS, sem componente.
            PanelEvent::Click(id) if id == ids::VECTOR_SYM_APPLY => self.symmetry.on = false,
            PanelEvent::Click(id) if id == ids::VECTOR_SYM_ON => self.symmetry.on = true,
            PanelEvent::Click(id) if symmetry_kind(id).is_some() => {
                if let Some(k) = symmetry_kind(id) {
                    self.symmetry.kind = k;
                }
            }
            PanelEvent::Click(id) if id == ids::VECTOR_SYM_FUSE_OFF => self.symmetry.fuse = false,
            PanelEvent::Click(id) if id == ids::VECTOR_SYM_FUSE_ON => self.symmetry.fuse = true,
            // ⭐⭐ **QUANTAS cópias a rosácea faz** — o braço que faltava, e o controlo estava
            // **MORTO** por isso (caça de 2026-08-30). O id era declarado, registado, pintado e
            // contado pelo censo de focalizabilidade, e o valor não tinha para onde ir: o artista
            // arrastava a barra, o chip mudava, e a contagem ficava pregada no `6` do default.
            //
            // ⚠️ **Nenhum `apply_to_selected` aqui.** A simetria é um MODO de desenho: as cópias
            // são desenho enquanto ela está ligada, e o Apply é que as consolida. Restilar a
            // selecção a partir daqui reescreveria a `SymmetrySpec` de uma forma que já fechou.
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_SYM_SEGMENTS => {
                self.symmetry.segments = crate::params::segments_from_value(v);
            }
            // **Forma** — o 5º pill. Re-arma o gesto na forma ATIVA do catálogo (não a
            // troca): é o caminho de volta ao desenho depois de um desvio pelo Select,
            // e é o pill que ACENDE enquanto se desenha (antes o modo Shape não tinha
            // botão nenhum e a fileira inteira ficava apagada).
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_SHAPE => self.mode = DrawMode::Shape,
            // **Botão de forma** — idem: o id carrega o índice no catálogo. Escolher a
            // forma JÁ arma o gesto de desenho (`set_shape` põe o modo em Shape).
            PanelEvent::Click(id) if shape_index(id).is_some() => {
                if let Some(k) = shape_index(id).and_then(|i| crate::shapes::SHAPES.get(i)) {
                    self.set_shape(k.kind);
                }
            }
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_TEXT => self.mode = DrawMode::Text,
            // **Conector** — o 6º pill. Arma o gesto forma→forma; a linha resultante é
            // derivada da relação, não autorada (a shell a re-cozinha por frame).
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_CONNECT => {
                self.mode = DrawMode::Connect;
            }
            // **Shape Builder** — o 7º pill. Precisa de 2+ formas selecionadas; a shell é
            // quem sabe disso (a tool não vê a cena).
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_BUILD => {
                self.mode = DrawMode::Build;
            }
            // **Pick Shapes** — o 8º pill (Blend). Arma a coleta de formas na ordem de clique; a
            // shell junta a lista e o botão Blend a liga (ADR-0128 C2b).
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_PICKBLEND => {
                self.mode = DrawMode::PickBlend;
            }
            // **Fillet / Chamfer** — o 9º e 10º pills. Clicar-e-arrastar sobre uma quina para
            // arredondá-la (arco) ou chanfrá-la (reta). O gesto e a conversão vira-quina moram
            // no shell; a tool só troca o modo, como todos os outros.
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_FILLET => self.mode = DrawMode::Fillet,
            PanelEvent::Click(id) if id == ids::VECTOR_MODE_CHAMFER => {
                self.mode = DrawMode::Chamfer;
            }
            // Stroke cap / join segmented rows + Dash slider. These are Style →
            // restyle the selected path (mirror of colour/width).
            PanelEvent::Click(id) if id == ids::VECTOR_ALIGN_CENTRE => {
                self.set_stroke_align(StrokeAlign::Centre);
            }
            PanelEvent::Click(id) if id == ids::VECTOR_ALIGN_INNER => {
                self.set_stroke_align(StrokeAlign::Inner);
            }
            PanelEvent::Click(id) if id == ids::VECTOR_ALIGN_OUTER => {
                self.set_stroke_align(StrokeAlign::Outer);
            }
            PanelEvent::Click(id) if id == ids::VECTOR_CAP_BUTT => self.set_cap(StrokeCap::Butt),
            PanelEvent::Click(id) if id == ids::VECTOR_CAP_ROUND => self.set_cap(StrokeCap::Round),
            PanelEvent::Click(id) if id == ids::VECTOR_CAP_SQUARE => {
                self.set_cap(StrokeCap::Square)
            }
            PanelEvent::Click(id) if id == ids::VECTOR_JOIN_MITER => {
                self.set_join(StrokeJoin::Miter)
            }
            PanelEvent::Click(id) if id == ids::VECTOR_JOIN_ROUND => {
                self.set_join(StrokeJoin::Round)
            }
            PanelEvent::Click(id) if id == ids::VECTOR_JOIN_BEVEL => {
                self.set_join(StrokeJoin::Bevel)
            }
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_DASH => {
                self.dash = slider_to_dash(v as f32);
                self.apply_to_selected = true;
            }
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_GAP => {
                self.gap = slider_to_gap(v as f32);
                self.apply_to_selected = true;
            }
            // **Pontas do traço.** O popover do chip escolhe uma ponta e o painel emite o
            // DISCRIMINANTE dela (`Marker::as_u8`) no id do chip — um `SetValue` por
            // seletor, e não um `Click` por opção: as pontas são DADO (`ALL_MARKERS`), e
            // uma ponta nova não pode exigir um braço novo aqui.
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_MARKER_START_DD => {
                self.set_marker_start(marker_from_value(v));
            }
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_MARKER_END_DD => {
                self.set_marker_end(marker_from_value(v));
            }
            // **Tamanho / arredondamento da ponta** — caixas numéricas (o valor chega
            // autorado, não um track 0..1), como os campos de forma e os do conector.
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_MARKER_SCALE => {
                self.set_marker_scale(v);
            }
            PanelEvent::SetValue(id, v) if id == ids::VECTOR_MARKER_ROUND => {
                self.set_marker_round(v);
            }
            // **Dupla via** — o clique reescreve as PONTAS (o estado é derivado delas).
            PanelEvent::Click(id) if id == ids::VECTOR_MARKER_BOTH => self.toggle_both_ends(),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::ids;

    /// ⭐⭐⭐ **TODO MODO QUE TEM PILL CHEGA À FERRAMENTA — e todo modo diz se tem pill.**
    ///
    /// ⛔⛔ **O gate com este nome era um FANTASMA**: o `seam.rs` do painel cita-o num comentário
    /// (*"Os pills têm seam na tabela de modos (`every_mode_button_reaches_the_tool`)"*) e ele
    /// **não existia em ficheiro nenhum** — `grep` por ele devolvia só a citação. Entre *o clique
    /// sai do painel* (que o `seam_*` prova) e *a ferramenta muda de modo* havia um passo sem
    /// instrumento, e é nele que um `match` sem braço mora.
    ///
    /// ⚠️ **A metade que faz disto um CENSO é a segunda:** a tabela é confrontada com o
    /// [`DrawMode::ALL`], então um modo novo **tem** de aparecer aqui — com pill, ou na lista dos
    /// que deliberadamente não têm. *Uma tabela escrita à mão sem essa confrontação envelhece na
    /// primeira feature.*
    #[test]
    fn every_mode_button_reaches_the_tool() {
        let pills: &[(ph2d_a11y::NodeId, DrawMode)] = &[
            (ids::VECTOR_MODE_SELECT, DrawMode::Select),
            (ids::VECTOR_MODE_NODE, DrawMode::Node),
            (ids::VECTOR_MODE_PEN, DrawMode::Pen),
            (ids::VECTOR_MODE_PENCIL, DrawMode::Pencil),
            (ids::VECTOR_MODE_SHAPE, DrawMode::Shape),
            (ids::VECTOR_MODE_TEXT, DrawMode::Text),
            (ids::VECTOR_MODE_BUILD, DrawMode::Build),
            (ids::VECTOR_MODE_CONNECT, DrawMode::Connect),
            (ids::VECTOR_MODE_PICKBLEND, DrawMode::PickBlend),
            (ids::VECTOR_MODE_FILLET, DrawMode::Fillet),
            (ids::VECTOR_MODE_CHAMFER, DrawMode::Chamfer),
            (ids::VECTOR_MODE_WIDTH, DrawMode::Width),
            (ids::VECTOR_MODE_TRIM, DrawMode::Trim),
            (ids::VECTOR_MODE_BUCKET, DrawMode::Bucket),
            (ids::VECTOR_MODE_CUT, DrawMode::Cut),
            (ids::VECTOR_MODE_FRAME, DrawMode::Frame),
            (ids::VECTOR_MODE_BONE, DrawMode::Bone),
        ];
        for &(id, esperado) in pills {
            // ⚠️ Parte de OUTRO modo, senão o teste ficaria verde sobre um braço que não existe.
            let mut tool = VectorTool {
                mode: if esperado == DrawMode::Select {
                    DrawMode::Pen
                } else {
                    DrawMode::Select
                },
                ..VectorTool::default()
            };
            tool.handle_panel_event(PanelEvent::Click(id));
            assert_eq!(
                tool.mode, esperado,
                "o pill de {esperado:?} nao chegou a' ferramenta - ele pinta, acende sob o rato e \
                 o modo nunca muda"
            );
        }
        // ⛔ **O CENSO**: todo modo do vocabulário aparece na tabela acima. Um modo novo sem pill
        // tem de se declarar aqui, e a lista de excepções é vazia de propósito — hoje todos têm.
        for m in DrawMode::ALL {
            assert!(
                pills.iter().any(|(_, x)| x == m),
                "o modo {m:?} nao esta' na tabela: ou ele tem pill (acrescente a linha) ou nao tem \
                 (declare-o aqui, com o motivo)"
            );
        }
    }
}
