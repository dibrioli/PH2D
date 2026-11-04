//! ⭐ **States e Morph States em cartões** (fatia G8, `docs/CartoesUI/12_VECTOR_PLANO.md`). Mesmos ids e
//! eventos. As mesmas perguntas de antes: sem hospedeiro só a frase; com a preview ligada a autoria
//! fecha (nem verbos, nem duração, nem sinais); a mola troca a duração e a curva; a direção só onde a
//! curva a usa; Show e Clear só num papel gravado; sem máquina, só a face que a cria.

use crate::cards_fx::com_unidade;
use crate::cards_tinta::{aberto, caixa, dica, escolha, seg, seg_icone, semeia, spec};
use crate::populate::components::{
    SPRING_DAMPING_DEFAULT, SPRING_STIFFNESS_DEFAULT, STATE_DURATION_DEFAULT_S,
};
use crate::state::{self, MorphShapeRow, MorphStatesState, UiStatesState};
use ph2d_a11y::NodeId;
use ph2d_anim::{Easing, EasingFamily, EasingMode};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::paint::{folga_do_texto, paint_text_elided};
use ph2d_editor_core::panel::TextKey;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget, corpo};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_i18n::tr;
use ph2d_tokens::ColorToken;

use crate::ids as p;

const PASSO_DA_DURACAO: f64 = 0.01; // LITERAL-PX-OK: segundos
const PASSO_DA_RIGIDEZ: f64 = 0.1; // LITERAL-PX-OK: régua de física, como `MAX_STIFFNESS`
const PASSO_DO_AMORTECIMENTO: f64 = 0.01; // LITERAL-PX-OK: razão de amortecimento

/// Com o store MUTÁVEL, antes de pintar: as marcas, as dicas dos ícones e os números do documento.
pub(crate) fn prepare(store: &mut WidgetStore) {
    for (chip, scrub, fabrica) in [
        (
            p::VECTOR_STATE_DURATION_NUM,
            PASSO_DA_DURACAO,
            STATE_DURATION_DEFAULT_S,
        ),
        (
            p::VECTOR_STATE_STIFFNESS_NUM,
            PASSO_DA_RIGIDEZ,
            SPRING_STIFFNESS_DEFAULT,
        ),
        (
            p::VECTOR_STATE_DAMPING_NUM,
            PASSO_DO_AMORTECIMENTO,
            SPRING_DAMPING_DEFAULT,
        ),
    ] {
        let seta = store
            .number_range(chip)
            .map_or(scrub, |(_, _, passo)| passo);
        store.mark_card_field(chip, spec(seta, scrub, Some(f64::from(fabrica))));
    }
    for i in 0..p::MAX_SIGNAL_BINDINGS {
        dica(
            store,
            p::vector_state_signal_remove_id(i),
            "card.vector.states.remove_signal",
        );
    }
    for i in 0..p::MAX_MORPH_STATES {
        dica(store, p::morph_shape_play_id(i), "card.vector.morph.play");
        dica(
            store,
            p::morph_shape_disconnect_id(i),
            "card.vector.morph.disconnect",
        );
    }
    if let Some(s) = state::ui_states_state() {
        semeia(
            store,
            p::VECTOR_STATE_DURATION_NUM,
            f64::from(s.duration_s),
            false,
        );
        if let Some((rigidez, amortecimento)) = s.spring {
            semeia(
                store,
                p::VECTOR_STATE_STIFFNESS_NUM,
                f64::from(rigidez),
                false,
            );
            semeia(
                store,
                p::VECTOR_STATE_DAMPING_NUM,
                f64::from(amortecimento),
                false,
            );
        }
    }
}

fn botao<'a>(id: NodeId, text: &'a str, primary: bool) -> Cell<'a> {
    Cell::new(Widget::Button { id, text, primary })
}

fn numero<'a>(id: NodeId, nome: &'a str) -> Cell<'a> {
    let glyph = Glyph::Text(corpo::letra_da_grandeza(nome));
    Cell::new(Widget::Field { id, glyph }).label(nome)
}

/// ⭐ **States**: de quem são as poses, como o gesto se comporta, que poses há, como se transita e o
/// que faz transitar. Nessa ordem de leitura.
pub(crate) fn states(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let Some(s) = state::ui_states_state() else {
        return y;
    };
    // A secção não some: é onde o artista descobre que falta uma forma que governe a seleção.
    let Some(host) = s.host.as_deref() else {
        let y = c.hint(tr("panel.vector.states.nohost"), y);
        return c.hint(tr("panel.vector.states.nohost.hint"), y);
    };
    let nome = if host.is_empty() {
        tr("panel.vector.states.host.unnamed")
    } else {
        host
    };
    let y = c.hint(&format!("{} {nome}", tr("panel.vector.states.host")), y);
    let y = gesto(c, &s, y);
    let em_preview = s.preview == Some(true);
    let mut y = papeis(c, &s, !em_preview, y);
    // A cena parada num hover parece o Default: esta frase é o que o desmente.
    if let Some(i) = s.live {
        y = c.hint(
            &format!("{} {}", tr("panel.vector.states.showing"), s.role_labels[i]),
            y,
        );
    }
    // A preview escreve poses derivadas e suprime o undo: editar aqui perderia o passo.
    if em_preview {
        return y;
    }
    let y = transicao(c, &s, y);
    sinais(c, &s.bindings, &s.role_labels, y)
}

/// A preview e o mover com todos os estados dividem a linha: dizem como um GESTO se comporta. O aceso
/// é o tipo do botão; o rótulo nunca alterna.
fn gesto(c: &mut CardCtx<'_>, s: &UiStatesState, y: f32) -> f32 {
    let preview = s.preview.map(|on| {
        botao(
            crate::ids::VECTOR_STATE_PREVIEW,
            tr("panel.vector.states.preview"),
            on,
        )
    });
    let mover = s.move_all.map(|on| {
        caixa(
            crate::ids::VECTOR_STATE_MOVE_ALL,
            Rotulo::vista(TextKey::new("panel.vector.states.move_all")),
            on,
        )
    });
    let y = match (preview, mover) {
        (Some(a), Some(b)) => c.pair(a, b, y),
        (Some(a), None) | (None, Some(a)) => c.row(&[a], y),
        (None, None) => y,
    };
    // O modo toma o rato: a saída anuncia-se, senão lê-se como travado.
    if s.preview == Some(true) {
        c.hint(tr("panel.vector.states.preview.on"), y)
    } else {
        y
    }
}

/// O nome do papel diz, pela tinta, o que ele tem: à vista na cena, gravado ou vazio. Na coluna à
/// esquerda do verbo `ao_lado`, já pintado.
fn nome_do_papel(c: &mut CardCtx<'_>, nome: &str, ao_lado: NodeId, gravado: bool, vivo: bool) {
    let Some(r) = c.hit_index.rect_for(ao_lado) else {
        return;
    };
    let tinta = if vivo {
        ColorToken::Accent
    } else if gravado {
        ColorToken::Text1
    } else {
        ColorToken::Text2
    };
    paint_text_elided(
        c.text_system,
        c.scene,
        nome,
        c.x,
        r.y + folga_do_texto(r.h, m::VALUE_PX),
        m::VALUE_PX,
        (r.x - m::FIELD_GAP - c.x).max(0.0),
        c.ink.color(tinta),
    );
}

/// Uma linha por PAPEL, numa grelha: os verbos ficam na mesma coluna em todos os papéis, e um papel
/// vazio tem só Rec, porque Show e Clear nele seriam cliques mudos. Em preview, os nomes aos pares, com
/// a marca no que está à vista.
fn papeis(c: &mut CardCtx<'_>, s: &UiStatesState, verbos: bool, y: f32) -> f32 {
    let vazio = || Cell::new(Widget::Readout { text: "" });
    if !verbos {
        let nomes: Vec<String> = s
            .role_labels
            .iter()
            .enumerate()
            .map(|(i, nome)| {
                if s.live == Some(i) {
                    format!("{} {nome}", tr("panel.vector.morph.live_mark"))
                } else {
                    nome.clone()
                }
            })
            .collect();
        let celulas: Vec<Cell<'_>> = nomes
            .iter()
            .map(|text| Cell::new(Widget::Readout { text }))
            .collect();
        return c.compact(&celulas, y);
    }
    s.role_labels.iter().enumerate().fold(y, |y, (i, nome)| {
        let gravado = s.recorded[i];
        let gravar = botao(
            p::vector_state_record_id(i),
            tr("panel.vector.states.rec"),
            true,
        );
        let fim = if gravado {
            let linha = [
                vazio(),
                botao(
                    p::vector_state_apply_id(i),
                    tr("panel.vector.states.show"),
                    false,
                ),
                gravar,
                botao(
                    p::vector_state_clear_id(i),
                    tr("panel.vector.states.clear"),
                    false,
                ),
            ];
            c.row(&linha, y)
        } else {
            c.row(&[vazio(), vazio(), gravar, vazio()], y)
        };
        // O 1.º verbo da linha: Show num papel gravado, Rec num vazio, que mora mais à direita.
        let ao_lado = if gravado {
            p::vector_state_apply_id(i)
        } else {
            p::vector_state_record_id(i)
        };
        nome_do_papel(c, nome, ao_lado, gravado, s.live == Some(i));
        fim
    })
}

/// A mola é uma OPÇÃO que TROCA as linhas: duas respostas à mesma pergunta nunca ficam à vista.
fn transicao(c: &mut CardCtx<'_>, s: &UiStatesState, y: f32) -> f32 {
    let mola = caixa(
        crate::ids::VECTOR_STATE_SPRING,
        Rotulo::vista(TextKey::new("panel.vector.states.spring")),
        s.spring.is_some(),
    );
    if s.spring.is_some() {
        let y = c.row(&[mola], y);
        return c.row(
            &[
                numero(
                    p::VECTOR_STATE_STIFFNESS_NUM,
                    tr("panel.vector.states.stiffness"),
                ),
                numero(
                    p::VECTOR_STATE_DAMPING_NUM,
                    tr("panel.vector.states.damping"),
                ),
            ],
            y,
        );
    }
    let duracao = com_unidade(
        tr("panel.vector.states.duration"),
        tr("card.vector.unit.seconds"),
    );
    let y = c.pair(numero(p::VECTOR_STATE_DURATION_NUM, &duracao), mola, y);
    curva(c, s.easing, y)
}

/// As fileiras saem do vocabulário da curva, e a da direção só onde ela muda a curva.
fn curva(c: &mut CardCtx<'_>, e: Easing, y: f32) -> f32 {
    let familias: Vec<Seg<'_>> = EasingFamily::ALL
        .iter()
        .enumerate()
        .map(|(i, f)| {
            seg(
                p::vector_easing_family_id(i),
                tr(f.label_key()),
                *f == e.family,
            )
        })
        .collect();
    let y = escolha(c, tr("panel.vector.states.curve"), &[], &familias, y);
    if !e.family.uses_mode() {
        return y;
    }
    let modos: Vec<Seg<'_>> = EasingMode::ALL
        .iter()
        .enumerate()
        .map(|(i, modo)| {
            seg(
                p::vector_easing_mode_id(i),
                tr(modo.label_key()),
                *modo == e.mode,
            )
        })
        .collect();
    escolha(c, tr("panel.vector.states.curve.mode"), &[], &modos, y)
}

/// A tabela sinal → papel vem por último: é a única lista que cresce. No teto o «+» some, porque o teto
/// já se vê nas linhas.
fn sinais(c: &mut CardCtx<'_>, ligacoes: &[(String, usize)], papeis: &[String; 4], y: f32) -> f32 {
    let rotulo = tr("panel.vector.states.signals");
    let mostra = ligacoes.len().min(p::MAX_SIGNAL_BINDINGS);
    let mut y = y;
    for (i, (_, papel)) in ligacoes.iter().enumerate().take(mostra) {
        let apagar = [seg_icone(
            p::vector_state_signal_remove_id(i),
            IconId::Minus,
            false,
        )];
        let campo = Cell::new(Widget::Text {
            id: p::vector_state_signal_name_id(i),
            placeholder: tr("panel.vector.states.signals.hint"),
        });
        let campo = if i == 0 { campo.label(rotulo) } else { campo };
        y = c.row(&[campo, Cell::new(Widget::Bare(&apagar))], y);
        let opcoes: Vec<Seg<'_>> = papeis
            .iter()
            .enumerate()
            .map(|(r, nome)| seg(p::vector_state_signal_role_id(i, r), nome, r == *papel))
            .collect();
        y = c.row(&[Cell::new(Widget::Choice(&opcoes))], y);
    }
    if mostra == p::MAX_SIGNAL_BINDINGS {
        return y;
    }
    let mais = botao(
        p::VECTOR_STATE_SIGNAL_ADD,
        tr("panel.vector.states.signals.add"),
        false,
    );
    let mais = if mostra == 0 {
        mais.label(rotulo)
    } else {
        mais
    };
    c.row(&[mais], y)
}

/// ⭐ **Morph States**: a preview com o que está à vista, uma linha por FORMA (nome, tecla, tocar e
/// desligar) e, longe dos verbos por forma, o que desfaz o conjunto. Sem máquina, a face que a cria.
pub(crate) fn morph_states(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let Some(s) = state::morph_states_state() else {
        return y;
    };
    if s.rows.is_empty() {
        return face_de_criar(c, &s, y);
    }
    let preview = botao(
        crate::ids::VECTOR_MORPH_PREVIEW,
        tr("panel.vector.morph.preview"),
        s.preview,
    );
    let agora = s
        .current
        .as_ref()
        .map(|forma| format!("{} {forma}", tr("panel.vector.morph.current")));
    let mut y = match agora.as_deref() {
        Some(text) => c.pair(preview, Cell::new(Widget::Readout { text }), y),
        None => c.row(&[preview], y),
    };
    // O modo toma o teclado: quem não soubesse sair carregaria em teclas, que é o que ele consome.
    if s.preview {
        y = c.hint(tr("panel.vector.morph.preview.on"), y);
    }
    let mostra = s.rows.len().min(p::MAX_MORPH_STATES);
    for (i, linha) in s.rows.iter().enumerate().take(mostra) {
        y = forma(c, i, linha, y);
    }
    y = c.danger_button(
        p::VECTOR_MORPH_DISSOLVE,
        tr("panel.vector.morph.dissolve"),
        y,
    );
    // Acima do tecto a máquina alcança todas: a lista diz o que não mostra, nunca corta em silêncio.
    if s.rows.len() > mostra {
        y = c.hint(
            &format!(
                "{} {}",
                s.rows.len() - mostra,
                tr("panel.vector.morph.beyond")
            ),
            y,
        );
    }
    y
}

/// O nome, a tecla e os dois verbos numa linha; os rótulos das colunas só na 1.ª. Um nome que não cabe
/// na coluna sobe a rótulo da tecla, com a largura da linha, e não se corta.
fn forma(c: &mut CardCtx<'_>, i: usize, linha: &MorphShapeRow, y: f32) -> f32 {
    let nome = if linha.live {
        format!("{} {}", tr("panel.vector.morph.live_mark"), linha.to)
    } else {
        linha.to.clone()
    };
    let verbos = [
        seg_icone(p::morph_shape_play_id(i), IconId::Play, false),
        seg_icone(p::morph_shape_disconnect_id(i), IconId::Unlink, false),
    ];
    let id = p::morph_shape_key_button_id(i);
    let tecla = Cell::new(Widget::Dropdown {
        id,
        text: if linha.when.is_empty() {
            tr("panel.vector.morph.when.none")
        } else {
            linha.when.as_str()
        },
        options: &[],
    });
    let coluna = (c.w - m::FIELD_GAP * 2.0 - m::FIELD_H * verbos.len() as f32) * 0.5;
    let cabe = c.text_system.prefix_width(&nome, m::VALUE_PX) <= coluna;
    let botoes = Cell::new(Widget::Bare(&verbos));
    let y = if !cabe {
        c.row(&[tecla.label(&nome), botoes], y)
    } else if i == 0 {
        let quem =
            Cell::new(Widget::Readout { text: &nome }).label(tr("panel.vector.morph.arrows"));
        let tecla = tecla.label(tr("panel.vector.morph.reached_by"));
        c.row(&[quem, tecla, botoes], y)
    } else {
        c.row(
            &[Cell::new(Widget::Readout { text: &nome }), tecla, botoes],
            y,
        )
    };
    if let Some(caixa_da_tecla) = aberto(c, id) {
        state::set_pending_morph_key_dd(Some((i, caixa_da_tecla)));
    }
    y
}

/// Três respostas diferentes de propósito: quem escolheu doze formas não pode ler «escolha duas».
fn face_de_criar(c: &mut CardCtx<'_>, s: &MorphStatesState, y: f32) -> f32 {
    let n = s.can_make;
    if n < 2 {
        return c.hint(tr("panel.vector.morph.need_shapes"), y);
    }
    if n > p::MAX_MORPH_STATES {
        return c.hint(
            &format!(
                "{} {}.",
                tr("panel.vector.morph.too_many"),
                p::MAX_MORPH_STATES
            ),
            y,
        );
    }
    let y = c.hint(&format!("{n} {}", tr("panel.vector.morph.make.shapes")), y);
    c.full(
        None,
        Widget::Button {
            id: p::VECTOR_MORPH_STATES_MAKE,
            text: tr("panel.vector.morph.make"),
            primary: false,
        },
        y,
    )
}
