//! **OS GESTOS DENTRO DE UM CARTÃO** (ciclo 1 — decisão do Enio, 2026-09-05: os params vivem no
//! nó e o painel lateral sai). Irmão de [`super`], que trata dos gestos sobre o GRAFO.
//!
//! ⚠️ `super` é o `interact`: as suas `push_intent`, `View` e o estado da interacção entram por
//! `use super::*`.

use super::*;

/// ⭐⭐⭐ **ARRASTAR O VALOR DE UM PARAM NO CARTÃO** (ciclo 1 — doc 103).
///
/// A lei é a do slider que a row DESENHA: **atravessar a largura do cartão varre a faixa
/// inteira** (`min..max` do hint). É o que o preenchimento da barra mostra, então o dedo e o
/// olho concordam por construção — e não há um segundo número de sensibilidade para calibrar.
///
/// ⚠️ **O delta é contra o x de PARTIDA, nunca contra o do quadro anterior.** Somar deltas
/// acumula o arredondamento de um param inteiro: arrastar para a direita e voltar não devolveria
/// o número onde começou.
///
/// ⚠️ **A edição sai pela porta que já existe** (`GraphIntent::SetParam` → `Graph::set_param`),
/// a mesma da row do painel — o undo, o memo do cook e os limites são os mesmos nas duas
/// superfícies. Nada de um segundo caminho de escrita.
pub(super) fn apply_param_row(
    state: &mut MotionGraphPanelState,
    g: GraphGesture,
    node: u32,
    row: u16,
    rect: Rect,
    snap: &GraphViewSnapshot,
) {
    let Some(view_node) = snap.nodes.iter().find(|n| n.id == node) else {
        return;
    };
    // ⚠️ `row` é o índice de FAIXA — uma coordenada só para o pintor, o hit-test e o gesto.
    let p = match crate::geom::band_at(view_node, row as usize) {
        Some(crate::geom::BandRow::Param(k)) => &view_node.params[k],
        // ⭐ O cabeçalho DOBRA. Na PRESSÃO (Begin), não na largada: um clique que arrasta um
        // pixel é classificado End pelo dispatch, e a dobra tem de acontecer na mesma — é a
        // mesma robustez que o alt-clique num fio já usa.
        Some(crate::geom::BandRow::Header(k)) => {
            if g.phase == GesturePhase::Begin {
                push_intent(GraphIntent::ToggleParamSection {
                    node,
                    section: view_node.sections[k].title,
                });
            }
            state.interaction = Interaction::Idle;
            return;
        }
        None => return,
    };
    match g.phase {
        GesturePhase::Begin => {
            state.interaction = Interaction::ScrubParam {
                node,
                row,
                start_value: p.value,
                start_x: g.x,
            };
        }
        GesturePhase::Update => {
            let Interaction::ScrubParam {
                node: n0,
                row: r0,
                start_value,
                start_x,
            } = state.interaction
            else {
                return;
            };
            if n0 != node || r0 != row {
                return;
            }
            let view = View::new(rect, state.view);
            let largura = crate::geom::CARD_W * view.zoom;
            if largura <= 0.0 {
                return;
            }
            // A faixa **RESOLVIDA** — a mesma que a barra desenha e a mesma que o painel
            // arrasta (canal · fio · `contain`).
            let faixa = p.max - p.min;
            let bruto = start_value + (g.x - start_x) / largura * faixa;
            // O passo decide se o número é inteiro — a mesma leitura que a row usa
            // para o escrever.
            let valor = if p.step >= 1.0 { bruto.round() } else { bruto };
            // `safe_clamp` e não `clamp`: ele é tolerante a NaN e a limites TROCADOS, e um
            // hint com `min > max` existe (um param cuja faixa é decrescente). O `clamp` da
            // biblioteca entra em pânico nesse caso.
            let valor = ph2d_editor_core::math::safe_clamp(valor, p.min, p.max);
            if (valor - p.value).abs() > f32::EPSILON {
                push_intent(GraphIntent::SetParam {
                    node,
                    param: p.hint.param,
                    // ⚠️ **A volta à unidade do DOCUMENTO acontece aqui e só aqui** — a row
                    // inteira (barra, número, arrasto) trabalha na face do artista.
                    value: p.to_stored(valor),
                });
            }
        }
        // ⭐⭐ **UM CLIQUE NUM ESTADO ALTERNA-O** — um interruptor vira, um enum avança para a
        // opção seguinte (com volta ao princípio). Arrastar continua a varrer, que é como se
        // atravessa depressa um enum de 48 opções; o clique é o gesto de quem quer *a
        // seguinte*, e é o único que um dedo faz sem querer varrer.
        //
        // ⚠️ **Só o CLIQUE**, nunca o `End` de um arrasto: senão largar o dedo depois de varrer
        // dava mais um passo, e o valor saltava por cima do que o artista tinha escolhido.
        GesturePhase::Click => {
            // ⭐⭐⭐ **UM CLIQUE NUM NÚMERO ABRE-O PARA ESCRITA** (report do Enio, 2026-09-05:
            // *«vários nós não permitem clicar no número para usar o teclado para escrever»*).
            //
            // ⚠️ **Quem responde *«isto é um número?»* é o PINTOR** (`shows_a_level`), não uma
            // segunda lista de espécies aqui: a row que desenha um nível é exactamente a que
            // aceita um nível escrito.
            //
            // O arrasto continua a varrer — é o *number field* do Blender inteiro: arrastar dá
            // *«um pouco mais»*, clicar dá *«exactamente isto»*.
            match click_does(p) {
                ClickDoes::Type => {
                    crate::param_edit::arm(state, node, row, p);
                    state.interaction = Interaction::Idle;
                    return;
                }
                ClickDoes::Toggle => push_intent(GraphIntent::SetParam {
                    node,
                    param: p.hint.param,
                    value: p.to_stored(f32::from(u8::from(p.value < 0.5))),
                }),
                ClickDoes::Cycle(n) => {
                    let proxima = (p.value.round() + 1.0).rem_euclid(n as f32);
                    push_intent(GraphIntent::SetParam {
                        node,
                        param: p.hint.param,
                        value: p.to_stored(proxima),
                    });
                }
                ClickDoes::PickFile => push_intent(GraphIntent::PickFile {
                    node,
                    param: p.hint.param,
                }),
                ClickDoes::Nothing => {}
            }
            state.interaction = Interaction::Idle;
        }
        GesturePhase::End | GesturePhase::DoubleClick => {
            state.interaction = Interaction::Idle;
        }
    }
}

/// ⭐⭐⭐ **O QUE UM CLIQUE NUMA ROW DO CARTÃO FAZ — a porta única.**
///
/// ⚠️ **Ela existe porque a mesma pergunta tem DOIS leitores, e uma segunda cópia mentiria:** o
/// gesto (que executa) e o **censo** que pergunta *«que controlos o cartão ainda não alcança?»*.
/// Enquanto a lei viveu só dentro do `match` do gesto, a resposta do censo teria de ser uma
/// lista de espécies escrita à mão — e uma espécie nova entraria no produto sem entrar na lista,
/// que é exactamente a forma do knob INALCANÇÁVEL (`CLAUDE.md §5.0`).
///
/// ⛔ **`Nothing` NÃO é um defeito por si:** é o estado declarado dos editores ricos (cor, curva,
/// gradiente, paleta, texto, ficheiro, canais, fonte) enquanto o painel lateral ainda existe —
/// o cartão pinta o selo, o painel abre o editor. Ele **passa** a ser um defeito no dia em que o
/// painel sair, e é essa a conta que o censo faz.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClickDoes {
    /// Abre a caixa de escrita (a row mostra um NÍVEL).
    Type,
    /// Vira o interruptor.
    Toggle,
    /// Avança para a opção seguinte de `n`, com volta ao princípio.
    Cycle(usize),
    /// **Pede à shell que abra o diálogo de ficheiro** — o cartão nunca abre um por si.
    PickFile,
    /// **Nada** — o cartão diz que o controlo existe e não o abre.
    Nothing,
}

/// A lei, num sítio só. ⚠️ Quem responde *«isto é um número?»* continua a ser o **PINTOR**
/// (`shows_a_level`): a row que desenha um nível é exactamente a que aceita um nível escrito.
#[must_use]
pub fn click_does(p: &crate::CardParam) -> ClickDoes {
    if crate::paint::paint_card_params::shows_a_level(p) {
        return ClickDoes::Type;
    }
    match p.hint.widget {
        ph2d_node_registry::ParamWidget::Toggle => ClickDoes::Toggle,
        ph2d_node_registry::ParamWidget::Enum { labels } if !labels.is_empty() => {
            ClickDoes::Cycle(labels.len())
        }
        ph2d_node_registry::ParamWidget::File { .. } => ClickDoes::PickFile,
        _ => ClickDoes::Nothing,
    }
}
