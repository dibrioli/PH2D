//! ⭐⭐ **O painel Physics em CARTÕES** (`docs/CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md`): as mesmas
//! tabelas ([`rows::SECTIONS`], [`interact::IROWS`]) e os MESMOS ids do corpo de antes; o despacho
//! (`event.rs`) não muda. O campo do cartão é o CHIP (ligado à barra, que continua a avisar o painel);
//! a barra deixa de se pintar: o scrub no ícone faz o que ela fazia.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::TextKey;
use ph2d_editor_core::panel::cards::corpo::letra_da_grandeza;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Seg, Widget};
use ph2d_editor_core::widget::TextInputState;
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_i18n::{tr, tr_with};
use ph2d_physics_ecs::{
    HoldMode, InteractionSettings, InteractionTool, JointTool, PhysicsSettings,
};

use crate::ids as i;
use crate::interact::{IROWS, IRow, ISection};
use crate::rows;
use crate::state::PhysicsSnapshot;

thread_local! {
    static FORCADO: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// O painel pinta-se em cartões? `PH2D_CARDS=0` = o corpo de antes.
#[must_use]
pub fn cards_on() -> bool {
    FORCADO
        .with(std::cell::Cell::get)
        .unwrap_or_else(|| std::env::var("PH2D_CARDS").as_deref() != Ok("0"))
}

/// Força o modo NESTA thread: os testes de um processo correm em paralelo, e a variável de ambiente
/// seria partilhada entre eles. `None` volta a ler o ambiente.
#[doc(hidden)]
pub fn force_cards_on_this_thread(on: Option<bool>) {
    FORCADO.with(|f| f.set(on));
}

/// O rótulo de cartão das linhas que têm unidade (D13: a unidade mora no rótulo, nunca na caixa).
const COM_UNIDADE: &[(NodeId, &str)] = &[
    (i::PHYSICS_CONTACT_HZ_NUM, "card.physics.contact_hz"),
    (i::PHYSICS_SLEEP_SPEED_NUM, "card.physics.sleep_speed"),
    (i::PHYSICS_SLEEP_DELAY_NUM, "card.physics.sleep_delay"),
    (i::PHYSICS_HOLD_SLACK_NUM, "card.physics.hold_slack"),
    (i::PHYSICS_BLAST_RADIUS_NUM, "card.physics.radius"),
    (i::PHYSICS_BLAST_FORCE_NUM, "card.physics.blast_impulse"),
    (i::PHYSICS_PULL_RADIUS_NUM, "card.physics.radius"),
    (i::PHYSICS_PULL_FORCE_NUM, "card.physics.pull_force"),
];

fn rotulo(chip: NodeId, plano: &'static str) -> &'static str {
    COM_UNIDADE
        .iter()
        .find(|(id, _)| *id == chip)
        .map_or_else(|| tr(plano), |(_, k)| tr(k))
}

/// Antes de pintar, com o store MUTÁVEL: os campos marcados, os números do mundo nas caixas e o
/// nome de cada célula muda da matriz. O campo que a mão arrasta não se semeia: sob a curva da Cena
/// o vivo puxaria o valor debaixo do gesto.
/// Cada campo mostra o nome do projeto, salvo enquanto se escreve nele.
fn semeia_os_nomes_das_camadas(store: &mut WidgetStore) {
    let nomes = crate::state::layer_names();
    for (nome, &id) in nomes.0.iter().zip(&i::PHYSICS_LAYER_NAME) {
        if let Some(InteractiveState::TextInput {
            state, text, caret, ..
        }) = store.get_mut(id)
            && *state != TextInputState::Focused
            && text != nome
        {
            *caret = nome.len();
            nome.clone_into(text);
        }
    }
}

pub(crate) fn prepare(store: &mut WidgetStore, snap: &PhysicsSnapshot) {
    store.mark_card_list_close(ph2d_editor_core::ids::PHYSICS_PANEL, i::PHYSICS_CLOSE);
    // O Debug nasce dobrado: é o que paga a catraca da altura de abertura (o cartão custa mais do que
    // a linha de antes) e o que menos se abre, com o Reset lá dentro.
    store.set_collapsed_if_unchosen(i::PHYSICS_SEC_DEBUG, true);
    semeia_os_nomes_das_camadas(store);
    let fabrica = PhysicsSettings::default();
    let arrastado = store.number_input_drag().map(|d| d.id);
    for row in rows::rows() {
        store.mark_card_field(
            row.chip,
            CardFieldSpec {
                step: row.step,
                scrub: row.step,
                default: Some(f64::from((row.get)(&fabrica))),
            },
        );
        if arrastado != Some(row.chip) {
            store.set_number_value(row.chip, f64::from((row.get)(&snap.vivo)));
        }
    }
    let fabrica = InteractionSettings::default();
    for row in IROWS {
        store.mark_card_field(
            row.chip,
            CardFieldSpec {
                step: row.step,
                scrub: row.step,
                default: Some(f64::from((row.get)(&fabrica))),
            },
        );
        if arrastado != Some(row.chip) {
            store.set_number_value(row.chip, f64::from((row.get)(&snap.interaction)));
        }
    }
}

/// O campo de uma linha da tabela; `canal` = a marca de animação (o mundo é canal da Cena; as
/// ferramentas do ponteiro são de sessão e não a levam, D17).
fn campo(chip: &'static NodeId, plano: &'static str, canal: bool) -> Cell<'static> {
    let label = rotulo(*chip, plano);
    let cell = Cell::new(Widget::Field {
        id: *chip,
        glyph: Glyph::Text(letra_da_grandeza(label)),
    })
    .label(label);
    if canal {
        cell.key(std::slice::from_ref(chip))
    } else {
        cell
    }
}

/// Duas por linha, na ordem da tabela (T56: as vizinhas de uma secção são irmãs).
fn aos_pares(c: &mut CardCtx<'_>, cells: &[Cell<'static>], y: f32) -> f32 {
    cells.chunks(2).fold(y, |y, par| c.row(par, y))
}

fn segs<'a>(ids: &[NodeId], nomes: &[&'a str], aceso: usize) -> Vec<Seg<'a>> {
    ids.iter()
        .zip(nomes)
        .enumerate()
        .map(|(n, (&id, &text))| Seg {
            id,
            icon: None,
            text,
            on: n == aceso,
        })
        .collect()
}

fn cartao(
    section: NodeId,
    title: &'static str,
    summary: Option<String>,
    body: impl Fn(&mut CardCtx<'_>, f32) -> f32 + 'static,
) -> CardDef<'static> {
    crate::state::note_painted_section_header(section);
    CardDef {
        section,
        title: tr(title),
        actions: Vec::new(),
        add: None,
        summary,
        body: Some(Box::new(body)),
    }
}

/// ⭐ Os cartões do painel, pela ordem natural de [`rows::section_header_ids`].
pub(crate) fn cards(snap: &PhysicsSnapshot) -> Vec<CardDef<'static>> {
    let snap = *snap;
    let mut out: Vec<CardDef<'static>> = rows::SECTIONS
        .iter()
        .map(|s| cartao_de_tabela(s, snap))
        .collect();
    out.push(cartao_da_interaccao(snap.interaction));
    out.push(cartao_das_juntas(snap.interaction));
    out.push(cartao_das_camadas(snap));
    out.push(cartao_de_debug(snap));
    out
}

fn cartao_de_tabela(s: &'static rows::Section, snap: PhysicsSnapshot) -> CardDef<'static> {
    let resumo = (s.id == i::PHYSICS_SEC_WORLD).then(|| {
        let n = |v: f32| ph2d_editor_core::widget::zero_sem_sinal(&format!("{v:.2}")).into_owned();
        format!("{} · {}", n(snap.vivo.gravity_x), n(snap.vivo.gravity_y))
    });
    cartao(s.id, s.title, resumo, move |c, y| {
        if s.id == i::PHYSICS_SEC_WORLD {
            return gravidade(c, y);
        }
        let mut y = y;
        if s.id == i::PHYSICS_SEC_SLEEP {
            y = c.check(
                crate::ids::PHYSICS_SLEEP_SPIN,
                tr("panel.physics.sleep_enabled"),
                snap.vivo.sleep_enabled(),
                y,
            );
        }
        let cells: Vec<Cell<'static>> = s
            .rows
            .iter()
            .map(|r| campo(&r.chip, r.label, true))
            .collect();
        aos_pares(c, &cells, y)
    })
}

/// Uma grandeza de duas componentes: um rótulo, as letras dos eixos e uma marca (T56, D17).
fn gravidade(c: &mut CardCtx<'_>, y: f32) -> f32 {
    const EIXOS: &[NodeId] = &[i::PHYSICS_GRAVITY_X_NUM, i::PHYSICS_GRAVITY_Y_NUM];
    let x = Cell::new(Widget::Field {
        id: EIXOS[0],
        glyph: Glyph::Text(tr("card.glyph.x")),
    })
    .label(tr("card.physics.gravity"));
    let y_cell = Cell::new(Widget::Field {
        id: EIXOS[1],
        glyph: Glyph::Text(tr("card.glyph.y")),
    })
    .key(EIXOS);
    c.row(&[x, y_cell], y)
}

/// As linhas numéricas da tabela de interacção vivas para a ferramenta em mãos.
fn numeros(it: &InteractionSettings, seccao: ISection) -> Vec<Cell<'static>> {
    IROWS
        .iter()
        .filter(|r: &&IRow| r.section == seccao && (r.shown)(it))
        .map(|r| campo(&r.chip, r.label, false))
        .collect()
}

fn cartao_da_interaccao(it: InteractionSettings) -> CardDef<'static> {
    let posicao = InteractionTool::ALL
        .iter()
        .position(|&t| t == it.tool)
        .unwrap_or(0);
    let ferramentas = [
        tr("panel.physics.tool.hand"),
        tr("panel.physics.tool.explode"),
        tr("panel.physics.tool.attract"),
    ];
    cartao(
        i::PHYSICS_SEC_INTERACT,
        "panel.physics.section.interact",
        Some(ferramentas[posicao].to_string()),
        move |c, y| {
            let opcoes = segs(&i::PHYSICS_INTERACT_TOOL_OPT, &ferramentas, posicao);
            let mut y = c.full(Some(tr("panel.physics.tool")), Widget::Choice(&opcoes), y);
            if it.tool == InteractionTool::Hand {
                let modo = HoldMode::ALL
                    .iter()
                    .position(|&m| m == it.hold)
                    .unwrap_or(0);
                let nomes = [
                    tr("panel.physics.hold.spring"),
                    tr("panel.physics.hold.rigid"),
                    tr("panel.physics.hold.rope"),
                ];
                let opcoes = segs(&i::PHYSICS_HOLD_MODE_OPT, &nomes, modo);
                y = c.full(Some(tr("panel.physics.hold")), Widget::Choice(&opcoes), y);
            }
            y = aos_pares(c, &numeros(&it, ISection::Sim), y);
            c.hint(tr("panel.physics.interact_hint"), y)
        },
    )
}

fn dica_da_junta(t: JointTool) -> &'static str {
    match t {
        JointTool::Body => "panel.physics.joint_hint.body",
        JointTool::Rig => "panel.physics.joint_hint.rig",
        JointTool::Links => "panel.physics.joint_hint.links",
        JointTool::Ik => "panel.physics.joint_hint.ik",
        JointTool::Fk => "panel.physics.joint_hint.fk",
    }
}

fn cartao_das_juntas(it: InteractionSettings) -> CardDef<'static> {
    let posicao = JointTool::ALL
        .iter()
        .position(|&t| t == it.joint)
        .unwrap_or(0);
    let modos = [
        tr("panel.physics.joint_tool.body"),
        tr("panel.physics.joint_tool.rig"),
        tr("panel.physics.joint_tool.links"),
        tr("panel.physics.joint_tool.ik"),
        tr("panel.physics.joint_tool.fk"),
    ];
    cartao(
        i::PHYSICS_SEC_JOINT,
        "panel.physics.section.joint",
        Some(modos[posicao].to_string()),
        move |c, y| {
            let opcoes = segs(&i::PHYSICS_JOINT_TOOL_OPT, &modos, posicao);
            let mut y = c.full(
                Some(tr("panel.physics.joint_tool")),
                Widget::Choice(&opcoes),
                y,
            );
            if it.joint == JointTool::Ik {
                let nomes = [
                    tr("panel.physics.ik_angle.free"),
                    tr("panel.physics.ik_angle.match"),
                ];
                let opcoes = segs(
                    &i::PHYSICS_IK_ANGLE_OPT,
                    &nomes,
                    usize::from(it.ik_match_angle),
                );
                y = c.full(
                    Some(tr("panel.physics.ik_angle")),
                    Widget::Choice(&opcoes),
                    y,
                );
            }
            y = aos_pares(c, &numeros(&it, ISection::Joint), y);
            let y = c.hint(tr(dica_da_junta(it.joint)), y);
            c.hint(tr("panel.physics.joint_hint.alt"), y)
        },
    )
}

fn cartao_das_camadas(_snap: PhysicsSnapshot) -> CardDef<'static> {
    // A matriz inteira mora no modal (onda 8): aqui, o resumo e a porta dele.
    let resumo = crate::modal::resumo();
    cartao(
        i::PHYSICS_SEC_LAYERS,
        "panel.physics.section.layers",
        Some(resumo.clone()),
        move |c, y| {
            let y = c.hint(&resumo, y);
            let y = c.full(
                None,
                Widget::Button {
                    id: i::PHYSICS_MATRIX_OPEN,
                    text: tr("card.insp.phys.edit_matrix"),
                    primary: false,
                },
                y,
            );
            crate::modal::declara_a_porta();
            y
        },
    )
}

fn cartao_de_debug(snap: PhysicsSnapshot) -> CardDef<'static> {
    cartao(
        i::PHYSICS_SEC_DEBUG,
        "panel.physics.section.debug",
        Some(tr_with(
            "panel.physics.bodies_count",
            &[("n", &snap.body_count.to_string())],
        )),
        move |c, y| {
            let mut y = c.check(
                crate::ids::PHYSICS_SHOW_COLLIDERS,
                Rotulo::vista(TextKey::new("panel.physics.show_colliders")),
                snap.show_colliders,
                y,
            );
            let escala = format!("{:.0}", snap.pixels_per_meter);
            let corpos = snap.body_count.to_string();
            y = c.row(
                &[
                    Cell::new(Widget::Readout { text: &escala }).label(tr("card.physics.scale")),
                    Cell::new(Widget::Readout { text: &corpos }).label(tr("card.physics.bodies")),
                ],
                y,
            );
            // Os dois verbos nunca coexistem: descartar esvazia a fita viva (W25).
            let fita = if snap.recorded_run_seconds > 0.0 {
                Some((
                    i::PHYSICS_CLEAR_RUN,
                    "panel.physics.clear_run",
                    snap.recorded_run_seconds,
                ))
            } else if snap.discarded_run_seconds > 0.0 {
                Some((
                    i::PHYSICS_RESTORE_RUN,
                    "panel.physics.restore_run",
                    snap.discarded_run_seconds,
                ))
            } else {
                None
            };
            if let Some((id, chave, segundos)) = fita {
                let text = format!("{} ({segundos:.1} s)", tr(chave));
                y = c.full(
                    None,
                    Widget::Button {
                        id,
                        text: &text,
                        primary: false,
                    },
                    y,
                );
            }
            c.full(
                None,
                Widget::Button {
                    id: i::PHYSICS_RESET_DEFAULTS,
                    text: tr("panel.physics.reset_defaults"),
                    primary: false,
                },
                y,
            )
        },
    )
}
