//! **§14 Platform Player** — o comportamento de personagem (W5).
//!
//! ⚠️ **A face VAZIA é a metade importante**, e é a lição que a §11 do W2a já
//! pagou: antes dela não existia gesto nenhum no editor que tornasse um sprite
//! físico. Aqui é a mesma coisa um degrau acima — um corpo Dynamic sem o
//! componente vê **um botão**, e é ele que faz o comportamento existir.
//!
//! ⚠️ **Nada aqui é oferecido a um corpo que não é Dynamic**, e é FÍSICA: a mola
//! é um impulso, e um impulso não move massa infinita. A recusa mora no
//! construtor do info (a shell), que é quem sabe o kind — o pintor decide se
//! oferece a partir da MESMA resposta.
//!
//! ⚠️ **Os `hit_index.register` são escritos com id LITERAL, um por botão**, e
//! isso não é verbosidade: é a única forma que o `architecture_panel_wiring_parity`
//! consegue ver (ele coleta `.register(ids::<LITERAL>` e pula um primeiro
//! argumento variável). Dobrá-los num laço apagaria a cobertura de paridade dos
//! botões em silêncio — a cicatriz que a §11 já carrega escrita.

use super::rows::{card_frame, num_row_unit, seg_row};
use super::*;
use ph2d_editor_core::screens::hero::InspectorPlayerInfo;
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;
use ph2d_i18n::tr_with;

/// A tabela dos cards da §14 — irmã por RESPONSABILIDADE (ver o topo dela).
///
/// ⚠️ **Este doc dizia «os nove cards» e a tabela tem doze** desde que `LEDGE`/`GLIDE`/`FALL`
/// entraram. ⛔ Não escreva o número aqui: a fonte é `PLAYER_CARDS.len()`, e uma contagem
/// copiada para um comentário envelhece sozinha (auditoria `docs/Sprite_projeto/20` §8).
///
/// ⚠️ **`table`, e não `rows`:** a `sections::rows` já existe (os primitivos
/// `card_frame`/`num_row`/`seg_row` que este pintor consome), e um segundo
/// `rows` aqui dentro a sombrearia — o pintor compilaria a chamar outra coisa.
#[path = "player_rows.rs"]
mod table;

/// A face VAZIA — ver [`door::paint_empty_face`].
#[path = "player_door.rs"]
mod door;

/// A leitura AO VIVO (postura, `facing`, velocidade) — irmã por RESPONSABILIDADE: é a única parte
/// da secção que não se edita.
#[path = "player_live.rs"]
mod live;
use live::paint_live_readout;
pub(crate) use table::{PLAYER_CARDS, player_row_count};

/// As dicas dos QUATRO BOTÕES da seção — a mesma lei das rows, num lugar onde não
/// cabe uma tupla de row.
///
/// ⚠️ **Foram QUATRO entre a F3 e 2026-09-14**: o `INSP_PLAYER_ADD` saiu com a face vazia que o
/// continha, e voltou com ela por ordem do dono — *«todas as opções aparecem com ele»*.
pub(crate) const PLAYER_BUTTON_TIPS: [(ph2d_a11y::NodeId, TextKey); 5] = [
    (
        ids::INSP_PLAYER_ADD,
        TextKey::new("panel.inspector.player.turn_this_body_into_a"),
    ),
    (
        ids::INSP_PLAYER_FIT,
        TextKey::new("panel.inspector.player.set_float_height_from_the"),
    ),
    (
        ids::INSP_PLAYER_REMOVE,
        TextKey::new("panel.inspector.player.give_the_behaviour_back_it"),
    ),
    (
        ids::INSP_PLAYER_CLEAR_RUN,
        TextKey::new("panel.inspector.player.throw_away_the_recorded_run"),
    ),
    (
        ids::INSP_PLAYER_FIT_CROUCH,
        TextKey::new("panel.inspector.player.set_crouch_height_to_the"),
    ),
];

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_player_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorPlayerInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: section header band height
    let h = TypeToken::Md.px() + Spacing::Sm.px(); // LITERAL-PX-OK: control row height
    let color_id = core_ids::INSP_LIVE_PLAYER_COLOR;
    let rgba = store
        .widget_color(color_id)
        .unwrap_or([0x88, 0x88, 0x88, 0xff]); // LITERAL-COLOR-OK: neutral default section accent
    let header = section_header(
        store,
        core_ids::INSP_LIVE_PLAYER_SECTION,
        tr("panel.inspector.player.platform_player"),
    )
    .color(rgba);
    let header_rect = Rect::new(x, y, w, header_h);
    paint_section_header(&header, header_rect, scene, text_system, theme);
    if let Some(circle_rect) = ph2d_editor_core::widget::color_circle_hit_rect(&header, header_rect)
    {
        hit_index.register(color_id, circle_rect);
    }
    // ⚠️ **A DOBRA do corpo** — ver `SectionFold`, e o `t` no lugar do `is_collapsed`.
    let Some(fold) = SectionFold::begin(
        store,
        core_ids::INSP_LIVE_PLAYER_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };

    let mut yy = y + header_h;

    // ⭐⭐ **A FACE VAZIA — a porta, no ficheiro irmão** (ver `player_door.rs` para o porquê).
    //
    // ⚠️ **A guarda continua a ser LEI, e não decoração:** sem ela um info com `has_player =
    // false` pintaria a secção inteira de knobs sobre um player que não existe — foi o que
    // aconteceu quando a face saiu, e os dois gates do `seam_player` foram quem o disse.
    if !info.has_player {
        let fim = door::paint_empty_face(scene, text_system, theme, hit_index, store, x, w, yy, h);
        return fold.finish(store, scene, hit_index, fim);
    }

    // **COMO ele é movido** (W-KinMove) — a primeira coisa da seção, porque toda
    // row abaixo dela é interpretada por este modo (a `LEG` inteira é a mola, e
    // sob Snap não há mola).
    yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        tr("panel.inspector.player.body"),
        ids::INSP_PLAYER_MODE,
        &ids::INSP_PLAYER_MODE_IDS,
        &[
            tr("panel.inspector.player.dynamic"),
            tr("panel.inspector.player.kinematic"),
            tr("panel.inspector.player.pure"),
        ],
        info.mode_tag,
    );

    // **O QUE ELE ESTÁ A FAZER** (`W-PlayerOut`, A3) — o readout que torna a
    // afinação observável sem um `println`, e o interruptor de quem fica sabendo.
    yy = paint_live_readout(scene, text_system, theme, x, w, yy, info.live);
    yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        tr("panel.inspector.player.emit_signals"),
        ids::INSP_PLAYER_EMIT,
        &ids::INSP_PLAYER_EMIT_IDS,
        &[
            tr("panel.inspector.player.off"),
            tr("panel.inspector.player.on"),
        ],
        u8::from(info.emits_signals),
    );

    yy = paint_cards(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        info.reaction_is_live,
        info.push_is_live,
        info.spring_is_live,
    );

    // **O QUE A PLATAFORMA DA AO PULO AO LARGA-LA** (`W-Leave`) — fora dos
    // cards porque um controle segmentado mede a PROPRIA altura e a moldura de
    // um card e' medida pela CONTAGEM de rows; ver `ids::INSP_PLAYER_LIFT_POLICY`.
    yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        tr("panel.inspector.player.platform_lift"),
        ids::INSP_PLAYER_LIFT_POLICY,
        &ids::INSP_PLAYER_LIFT_POLICY_IDS,
        &[
            tr("panel.inspector.player.full"),
            tr("panel.inspector.player.up_only"),
            tr("panel.inspector.player.none"),
        ],
        info.platform_lift,
    );

    // **ELE PODE ANDAR PARA FORA DE UM PATAMAR?** (`W-Brink`) — fora dos cards
    // pelo mesmo motivo da row acima.
    yy = seg_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        tr("panel.inspector.player.walk_off_ledges"),
        ids::INSP_PLAYER_WALK_OFF,
        &ids::INSP_PLAYER_WALK_OFF_IDS,
        &[
            tr("panel.inspector.player.yes"),
            tr("panel.inspector.player.stop_at_edge"),
        ],
        info.walk_off_ledges,
    );
    // ⚠️ **Só com o AGACHAR autorado** — sem ele a `walk_for` devolve a config
    // de pé e este número nunca é lido: a row seria um controle morto.
    if info.crouch_armed {
        yy = seg_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            yy,
            tr("panel.inspector.player.when_crouching"),
            ids::INSP_PLAYER_CROUCH_WALK_OFF,
            &ids::INSP_PLAYER_CROUCH_WALK_OFF_IDS,
            &[
                tr("panel.inspector.player.yes"),
                tr("panel.inspector.player.stop_at_edge"),
            ],
            info.crouch_walk_off_ledges,
        );
    }

    // **OS VERBOS da seção** — extraídos do `paint` por TETO DE LOC (o `Brake`
    // da W-Brake foi a row que o cruzou), e o corte é por responsabilidade: o pai
    // decide o que a seção MOSTRA, o filho pinta o que ela FAZ.
    let out = paint_verbs(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        yy,
        h,
        info,
    );
    fold.finish(store, scene, hit_index, out)
}

/// **Os quatro botões da §14** — os dois `Fit`, o da corrida gravada e o
/// `Remove`.
///
/// ⚠️ Eles viajam juntos porque respondem à mesma pergunta (*que verbos esta
/// seção oferece?*) e porque três dos quatro são CONDICIONAIS: cada um só é
/// pintado onde tem o que fazer, que é a lei do knob morto desta seção. Separá-los
/// espalharia essa decisão por dois arquivos.
#[allow(clippy::too_many_arguments)]
fn paint_verbs(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    h: f32,
    info: &InspectorPlayerInfo,
) -> f32 {
    let mut yy = y;
    // ⚠️ **O piso geométrico, dito em voz alta — e pelo controle que o resolve.**
    //
    // O sensor mede na VERTICAL e quem encosta na rampa é a cápsula ao longo da
    // NORMAL dela, então flutuar de verdade exige
    // `float_height > half_height + radius / cos(max_slope)`. Com o ponto de
    // partida (`0,5`) e a cápsula canônica o personagem fica **TANGENTE** ao
    // chão — ele não paira, e só uma rampa revela. Um número que o app SABE e
    // não mostra é um número que o artista descobre por acidente.
    //
    // O aviso mora no rótulo do próprio botão que o conserta: um controle, uma
    // mensagem. Um readout separado seria uma segunda superfície dizendo o mesmo
    // fato, e as duas divergiriam no dia em que a fórmula ganhasse uma forma.
    // ⚠️ **E ele segue a row que conserta** (a auditoria de 15/08): o botão
    // escreve o `float_height`, que sob a perna que POUSA é sobrescrito pela
    // geometria do corpo antes de o motor o ver. Oferecê-lo ali seria um verbo
    // cujo efeito a lei apaga no mesmo tique — a lei do knob morto que o resto
    // desta função já honra.
    if info.min_float_known && info.spring_is_live {
        let label = if info.float_height <= info.min_float_height {
            tr_with(
                "panel.inspector.player.fit_needs",
                &[("min", &format!("{:.2}", info.min_float_height))],
            )
        } else {
            tr("panel.inspector.player.fit_to_collider").to_string()
        };
        let rect = ph2d_editor_core::property_row::caixa_do_botao(text_system, x, w, yy, h, &label);
        let btn = Button::new(ids::INSP_PLAYER_FIT, &label)
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_PLAYER_FIT));
        paint_button(&btn, rect, scene, text_system, theme);
        hit_index.register(ids::INSP_PLAYER_FIT, rect);
        yy += h + ph2d_tokens::control_gap_px();
    }

    // **O MESMO piso, uma perna abaixo** (W18) — o espelho exato do botão acima,
    // e ele existe porque o card do AGACHAR não tinha controle nenhum que
    // resolvesse o número.
    //
    // ⚠️ **O defeito é medido e não é o que a nota da W15 previa.** Ela dizia *"o
    // corpo enterrado"*; ele **não enterra, ele SATURA** — o solver o segura
    // tangente com 1 mm de folga, a pose fica perfeitamente estável, e o que
    // acontece é o slider ficar **MORTO**: numa rampa de 45° (piso `0,583`)
    // autorar `0,50` dá folga `0,059` e autorar `0,30` dá `0,058`. Duzentos
    // milímetros de curso, um milímetro de resposta, e nada na tela.
    //
    // ⚠️ **Só com o agachar ARMADO:** em zero a capacidade está desligada e não há
    // defeito nenhum — e um botão que a ligasse pelas costas conflataria *dar um
    // agachar* com *consertar o que ele mede*.
    if info.min_float_known && info.crouch_height > 0.0 {
        let label = if info.crouch_height <= info.min_float_height {
            tr_with(
                "panel.inspector.player.fit_crouch_needs",
                &[("min", &format!("{:.2}", info.min_float_height))],
            )
        } else {
            tr("panel.inspector.player.fit_crouch_to_collider").to_string()
        };
        let rect = ph2d_editor_core::property_row::caixa_do_botao(text_system, x, w, yy, h, &label);
        let btn = Button::new(ids::INSP_PLAYER_FIT_CROUCH, &label)
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_PLAYER_FIT_CROUCH));
        paint_button(&btn, rect, scene, text_system, theme);
        hit_index.register(ids::INSP_PLAYER_FIT_CROUCH, rect);
        yy += h + ph2d_tokens::control_gap_px();
    }

    // **A CORRIDA GRAVADA** (W17) — o mesmo desenho do botão acima: *o aviso mora
    // no rótulo do próprio controle*, então o número de segundos viaja no texto e
    // não num readout ao lado.
    //
    // ⚠️ **A AUSÊNCIA dele é o outro readout.** Sem corrida não há o que
    // descartar, e um botão pintado sobre nada seria um controle que não faz nada
    // — a lei do knob morto que esta seção honra em toda row opt-in.
    //
    // ⚠️ **E DESCARTAR TEM VOLTA** (W24): a corrida some do documento mas fica
    // guardada na sessão, e o mesmo lugar da tela passa a oferecer o caminho de
    // volta. Os dois nunca aparecem juntos — *há corrida viva* e *há corrida
    // descartada com a fita vazia* são estados mutuamente exclusivos por
    // construção, e é isso que dispensa qualquer coordenação entre eles.
    let run_button = if info.recorded_run_seconds > 0.0 {
        Some((
            ids::INSP_PLAYER_CLEAR_RUN,
            tr_with(
                "panel.inspector.player.clear_run",
                &[("s", &format!("{:.1}", info.recorded_run_seconds))],
            ),
        ))
    } else if info.discarded_run_seconds > 0.0 {
        Some((
            ids::INSP_PLAYER_RESTORE_RUN,
            tr_with(
                "panel.inspector.player.restore_run",
                &[("s", &format!("{:.1}", info.discarded_run_seconds))],
            ),
        ))
    } else {
        None
    };
    if let Some((id, label)) = run_button {
        let rect = ph2d_editor_core::property_row::caixa_do_botao(text_system, x, w, yy, h, &label);
        let btn = Button::new(id, &label)
            .kind(ButtonKind::Default)
            .visual(store.button_visual(id));
        paint_button(&btn, rect, scene, text_system, theme);
        hit_index.register(id, rect);
        yy += h + ph2d_tokens::control_gap_px();
    }

    let rect = ph2d_editor_core::property_row::caixa_do_botao(
        text_system,
        x,
        w,
        yy,
        h,
        tr("panel.inspector.player.remove_platform_player"),
    );
    let btn = Button::new(
        ids::INSP_PLAYER_REMOVE,
        tr("panel.inspector.player.remove_platform_player"),
    )
    .kind(ButtonKind::Default)
    .visual(store.button_visual(ids::INSP_PLAYER_REMOVE));
    paint_button(&btn, rect, scene, text_system, theme);
    hit_index.register(ids::INSP_PLAYER_REMOVE, rect);
    yy + h + ph2d_tokens::control_gap_px()
}

/// **Os oito cards de números**, na ordem da tabela — extraído do `paint` por
/// TETO DE LOC, e o corte é por responsabilidade: aqui só se pinta a grade de
/// rows; quem decide o que a seção mostra fica no pai.
#[allow(clippy::too_many_arguments)]
fn paint_cards(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    mut yy: f32,
    reaction_is_live: bool,
    push_is_live: bool,
    spring_is_live: bool,
) -> f32 {
    // ⚠️ **UMA regra, e o pintor E a moldura a perguntam** — o `card_frame`
    // recebe a CONTAGEM das rows e desenha a caixa por ela, então medir com uma
    // lista e preencher com outra é como a seção seguinte pinta por cima dos
    // controles (a lição que o `segmented_row_counts` do Painter já pagou).
    // ⚠️ **As TRÊS rows da MOLA saem juntas, e o card FICA** — sob a perna que
    // pousa (`Support::Snap`) a lei sobrescreve o `float_height` pela geometria
    // do corpo e ninguém lê a rigidez nem o amortecimento, mas a `Cling
    // Distance` vira o `snap_distance` E o `step_height` do controlador: é o
    // número mais vivo da seção, e esconder o card inteiro o levaria junto.
    const SPRING_ONLY: [ph2d_a11y::NodeId; 3] = [
        ids::INSP_PLAYER_FLOAT,
        ids::INSP_PLAYER_STIFFNESS,
        ids::INSP_PLAYER_DAMPING,
    ];
    let shown = |id: ph2d_a11y::NodeId| {
        (push_is_live || id != ids::INSP_PLAYER_REACT_PUSH)
            && (spring_is_live || !SPRING_ONLY.contains(&id))
    };
    // ⭐⭐⭐ **A coluna do rótulo é a da SECÇÃO INTEIRA, medida uma vez.**
    //
    // ⛔⛔ Report do dono, 2026-09-14: *«3 pontos (…) sendo usados antes de ficar estreito»*. A
    // causa medida: o dock dele está a `220,9 px` (o `~/.ph2d/layout.txt`; a omissão é `304`), e a
    // metade da linha dá `78,4` — **16 dos 52** rótulos elidiam, enquanto o campo ao lado mostrava
    // `2` ou `65` com espaço de sobra.
    //
    // ⚠️ **Medida sobre TODOS os cards, não por card:** *«as labels alinhadas todas à direita»* é
    // uma coluna só para a secção; uma por card daria doze colunas.
    let coluna = {
        let font = TypeToken::Sm.px();
        let mut mais_largo = 0.0_f32;
        for (_, _, rows) in PLAYER_CARDS {
            for (label, _, _, _) in rows {
                mais_largo = mais_largo.max(text_system.prefix_width(label.tr(), font));
            }
        }
        Some(mais_largo)
    };
    for (title, card_id, rows) in PLAYER_CARDS {
        // ⚠️ **O card da 3ª lei some no modo que não a tem** (W-KinPure) — não é
        // arrumação, é a lei do knob-morto: sob o *puro sangue* NENHUM dos três
        // escalares é lido, e três sliders inertes ensinariam o artista a
        // desconfiar dos outros. A pergunta chega resolvida da shell (o painel
        // não sabe o que é um `PlayerMode`).
        //
        // ⚠️ O card é reconhecido pelo **id**, nunca pelo título: o título é o
        // que o artista lê e pode ser reescrito amanhã sem que ninguém pense
        // nesta linha.
        //
        // ⚠️ Os valores AUTORADOS continuam no componente: esconder não apaga, e
        // voltar ao Kinematic devolve o card com os números que lá estavam.
        if !reaction_is_live && card_id == ids::INSP_PLAYER_CARD_REACT {
            continue;
        }
        let n = rows.iter().filter(|(_, id, _, _)| shown(*id)).count();
        let (ix, iw, mut ry, next_y) =
            card_frame(scene, text_system, theme, x, w, yy, title.tr(), n);
        for (label, id, _tip, unit) in rows {
            if !shown(*id) {
                continue;
            }
            ry = num_row_unit(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                ix,
                iw,
                ry,
                label.tr(),
                *id,
                *unit,
                coluna,
            );
        }
        // ⚠️ O `ry` é DESCARTADO de propósito: quem manda no fluxo é a moldura
        // (`next_y`), medida pela MESMA régua com que as rows avançam. Somar as
        // rows aqui seria a segunda aritmética que discorda da caixa desenhada.
        let _ = ry;
        yy = next_y;
    }
    yy
}
