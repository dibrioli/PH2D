//! Seam do **TEXTURE PATTERN** (plano 33, W5) — o chip *Tile* está vivo sob o ponteiro.
//!
//! O gesto é REAL (Down+Up sobre o rectângulo que o painel pintou), e não um `WidgetEvent::Click`
//! sintético: ⚠️ o sintético prova a allowlist do painel mas **pula a checagem de focabilidade no
//! store** — é a lacuna que já deixou 36 células da matriz de física e dez chips do Painter
//! *pintados, hit-registrados e mortos sob o ponteiro*. Foi também o 2.º report do Enio sobre os
//! chips da booleana em 26/08: **um controlo nunca pintado e um morto sob o dedo dão o MESMO
//! report.**
//!
//! As duas metades de cada gate são independentes: sair do `populate_ops` mata a primeira (o
//! ponteiro não vira Click), sair do `event_clicks` mata a segunda (o Click não chega ao bus).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_vector::state::{FillKind, VectorPanelState};
use ph2d_panel_vector::{VectorPanel, ids, state};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};
const SEC: u128 = 1_000_000_000;

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

fn click_reaches_bus(id: ph2d_a11y::NodeId, what: &str) {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let r = host
        .painted_rect::<VectorPanel>(&mut panel_state, VIEWPORT, id)
        .unwrap_or_else(|| panic!("{what} nao foi PINTADO com area clicavel"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    host.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    let evs = host.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100));
    assert!(
        evs.iter()
            .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
        "o ponteiro sobre {what} nao virou Click - ele esta' desenhado e nao existe para o \
         dispatcher (falta o `register` no populate_ops)"
    );
    for ev in evs {
        host.apply_panel_event::<VectorPanel>(&mut panel_state, ev);
    }
    assert!(
        host.drained_actions().into_iter().any(|a| matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(c)) if c == id
        )),
        "o Click de {what} nao chegou ao bus - ele acende sob o mouse e nao faz nada (falta a \
         linha na allowlist do event_clicks)"
    );
}

/// **O chip `Tile` está vivo sob o ponteiro e chega ao bus.**
#[test]
fn the_pattern_chip_is_reachable_and_reaches_the_bus() {
    state::set_current_fill(Some(FillKind::Solid), None);
    click_reaches_bus(ids::VECTOR_FILL_KIND_PATTERN, "o chip Tile");
}

/// ⚠️ **A fileira inteira continua viva** — acrescentar o 5.º chip encolheu TODOS os outros (de
/// `58,5` para `45,6` px, medido), e um rectângulo mal calculado mata o vizinho sem mudar o
/// desenho de nada.
#[test]
fn adding_the_fifth_chip_left_the_other_four_alive() {
    state::set_current_fill(Some(FillKind::Solid), None);
    for (id, what) in [
        (ids::VECTOR_FILL_KIND_SOLID, "o chip Solid"),
        (ids::VECTOR_FILL_KIND_LINEAR, "o chip Linear"),
        (ids::VECTOR_FILL_KIND_RADIAL, "o chip Radial"),
        (ids::VECTOR_FILL_KIND_MULTI, "o chip Multi"),
    ] {
        click_reaches_bus(id, what);
    }
}

/// ⚠️ **Os cinco chips não se sobrepõem** — em NENHUM dos dois eixos.
///
/// ⚠️⚠️ **A 1.ª redacção deste gate comparava só o eixo X, e reprovou o produto certo.** A fileira
/// passou a usar o `paint_segmented_group_adaptive`, que **REFLUI** quando os chips não cabem: dois
/// chips em LINHAS diferentes partilham a mesma faixa de `x` por construção, e isso não é
/// sobreposição nenhuma. *Uma régua de um eixo só chama de colisão o que é apenas uma quebra de
/// linha.*
#[test]
fn the_five_chips_do_not_overlap() {
    state::set_current_fill(Some(FillKind::Solid), None);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    let mut rects: Vec<(&str, Rect)> = Vec::new();
    for (id, name) in [
        (ids::VECTOR_FILL_KIND_SOLID, "Solid"),
        (ids::VECTOR_FILL_KIND_LINEAR, "Linear"),
        (ids::VECTOR_FILL_KIND_RADIAL, "Radial"),
        (ids::VECTOR_FILL_KIND_MULTI, "Multi"),
        (ids::VECTOR_FILL_KIND_PATTERN, "Tile"),
    ] {
        let r = host
            .painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
            .expect("chip pintado");
        rects.push((name, r));
    }
    for i in 0..rects.len() {
        for j in (i + 1)..rects.len() {
            let (an, a) = rects[i];
            let (bn, b) = rects[j];
            let x = a.x < b.x + b.w - 1e-3 && b.x < a.x + a.w - 1e-3;
            let y = a.y < b.y + b.h - 1e-3 && b.y < a.y + a.h - 1e-3;
            assert!(
                !(x && y),
                "os chips `{an}` e `{bn}` sobrepoem-se: {a:?} e {b:?}"
            );
        }
    }
}

/// ⭐ **Sonda: quantas LINHAS a fileira de tipo de preenchimento ocupa.** Ela reflui quando não
/// cabe, e saber em que ponto isso acontece é o que decide se cabe um 6.º chip.
#[test]
#[ignore = "sonda: imprime a disposicao, nao afirma nada"]
fn measure_the_fill_kind_row_layout() {
    state::set_current_fill(Some(FillKind::Solid), None);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    for (id, name) in [
        (ids::VECTOR_FILL_KIND_SOLID, "Solid"),
        (ids::VECTOR_FILL_KIND_LINEAR, "Linear"),
        (ids::VECTOR_FILL_KIND_RADIAL, "Radial"),
        (ids::VECTOR_FILL_KIND_MULTI, "Multi"),
        (ids::VECTOR_FILL_KIND_PATTERN, "Tile"),
    ] {
        let r = host
            .painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
            .expect("chip pintado");
        println!(
            "{name:>7}: x {:.1} y {:.1} w {:.1} h {:.1}",
            r.x, r.y, r.w, r.h
        );
    }
}

/// A lei de referência da secção: uma grade, arte de 1 unidade, sem vão nem rotação.
fn row(kind: u8) -> ph2d_panel_vector::TexturePatternRow {
    ph2d_panel_vector::TexturePatternRow {
        kind,
        offset_denom: 2.0,
        size: [1.0, 1.0],
        lock_aspect: true,
        gap: [0.0, 0.0],
        link_gap: true,
        angle_deg: 0.0,
        shift_pct: [0.0, 0.0],
        mode: 0,
        // A referência é um ladrilho que FECHA — a dica de costura tem gate próprio.
        wrap_seam_visible: false,
        // E cuja arte EXISTE — o aviso de arte apagada tem gate próprio.
        art: ph2d_panel_vector::PatternArt::Ready,
    }
}

/// **Todo controlo da secção *Pattern* está vivo sob o ponteiro e chega ao bus.**
///
/// ⚠️ Sem esta secção o padrão nasce e **fica como nasceu** — o motor existiria, gateado e smokado,
/// e o artista teria uma imagem repetida que não consegue tocar.
#[test]
fn every_pattern_section_control_is_reachable_and_reaches_the_bus() {
    use ph2d_panel_vector::ids::TexPatKnob as K;
    state::set_current_fill(Some(FillKind::Pattern), None);
    // ⭐⭐ **AS DUAS secções** (plano 35, wave F) — a do preenchimento e a do traço. ⚠️ E percorridas
    // pela MESMA lista que as pinta e regista (`TexPatKnob::ALL`): um knob novo entra neste gate
    // sozinho. *Uma lista escrita à mão aqui seria a quarta cópia dos controlos.*
    for slot in 0..ph2d_panel_vector::ids::TEXPAT_SLOTS {
        // A secção do traço só sobe com um traço que tenha padrão — a do preenchimento não olha
        // para isto, e é por isso que a publicação é por tinta.
        state::set_stroke_present(Some(true));
        state::set_stroke_paint_kind(Some(ph2d_panel_vector::StrokePaintKind::Pattern));
        state::set_current_texture_pattern(slot, Some(row(0)));
        for k in K::ALL {
            // Só os BOTÕES atravessam o barramento por Click; os sliders têm porta própria.
            if matches!(
                k,
                K::Source | K::PickShape | K::Lock | K::Tile(_) | K::Mode(_)
            ) {
                click_reaches_bus(
                    ph2d_panel_vector::texture_pattern::kid(slot, k),
                    &format!("o controlo {k:?} da tinta {slot}"),
                );
            }
        }
        state::set_current_texture_pattern(slot, None);
    }
    state::set_stroke_paint_kind(None);
}

/// ⚠️⚠️ **A secção SOME inteira para uma forma sem padrão** — presença E ausência.
///
/// A metade da AUSÊNCIA é a que impede o ruído: uma forma sólida não tem lei de ladrilho, e um
/// cabeçalho que aparece para ela é um controlo que não se aplica.
#[test]
fn the_section_vanishes_for_a_shape_without_a_pattern() {
    state::set_current_fill(Some(FillKind::Solid), None);
    state::set_current_texture_pattern(0, None);
    state::set_current_texture_pattern(1, None);
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut st = VectorPanelState;
    assert!(
        host.painted_rect::<VectorPanel>(
            &mut st,
            VIEWPORT,
            ph2d_panel_vector::texture_pattern::kid(0, ph2d_panel_vector::ids::TexPatKnob::Width)
        )
        .is_none(),
        "a seccao Pattern subiu para uma forma sem padrao"
    );
    // Controlo: com padrão publicado, ela sobe — senão este gate passaria num painel partido.
    state::set_current_texture_pattern(0, Some(row(0)));
    let mut host2 = MockPanelHost::with_panel::<VectorPanel>();
    assert!(
        host2
            .painted_rect::<VectorPanel>(
                &mut st,
                VIEWPORT,
                ph2d_panel_vector::texture_pattern::kid(
                    0,
                    ph2d_panel_vector::ids::TexPatKnob::Width
                )
            )
            .is_some(),
        "com padrao a seccao tem de subir"
    );
    state::set_current_texture_pattern(0, None);
}

/// ⚠️ **O Offset só existe onde tem sentido.** Na GRADE não há desfasamento, e na COLMEIA ele é
/// **fixo** em meio passo — é isso que a torna colmeia. Oferecê-lo ali seria um knob que o modelo
/// ignora, que é o defeito que o [doc 90](../../../docs/Motion%20Nodes/90_caca_aos_knobs_mortos.md)
/// catalogou dezanove vezes.
#[test]
fn the_offset_row_only_shows_for_brick_and_column() {
    let shows = |kind: u8| {
        state::set_current_fill(Some(FillKind::Pattern), None);
        state::set_current_texture_pattern(0, Some(row(kind)));
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        host.painted_rect::<VectorPanel>(
            &mut st,
            VIEWPORT,
            ph2d_panel_vector::texture_pattern::kid(0, ph2d_panel_vector::ids::TexPatKnob::Offset),
        )
        .is_some()
    };
    assert!(!shows(0), "a GRADE nao tem desfasamento");
    assert!(shows(1), "o Brick tem");
    assert!(shows(2), "o Column tem");
    assert!(!shows(3), "a COLMEIA tem-no FIXO em meio passo");
    state::set_current_texture_pattern(0, None);
}

/// ⭐⭐ **UM PARÂMETRO QUE O MODO NÃO USA NÃO APARECE** (Enio, 2026-08-27) — e a pergunta é sempre
/// por PARÂMETRO, nunca por modo.
///
/// ⛔⛔ **A 1.ª redacção deste gate escondia CINCO knobs no `Clamp` e só UM deles é morto lá**
/// (auditoria de 2026-08-30). Ela afirmava, com estas palavras, que *"o reticulado, o desfasamento,
/// o tamanho e o vão não têm quem os leia"* — e quatro dessas afirmações são falsas, porque o
/// `mode` **não entra na chave do assado**: o ladrilho é assado com o reticulado inteiro também no
/// `Clamp`, e o `placement_in` consome `cells` e `tile_px`.
///
/// - **`size`** — a RAZÃO entre os eixos decide o enquadramento. Com o cadeado ligado o factor
///   cancela; com ele **desligado**, `size` é o único knob que escolhe o aspecto da cópia — e o
///   gate `clamp_frames_the_copy_over_the_shape_without_touching_the_authored_law` já o afirmava,
///   na crate ao lado. *Dois gates da mesma casa diziam coisas opostas.*
/// - **`gap` / `kind` / `offset_denom`** — entram no assado; um vão positivo rodeia a arte de
///   transparente que o `Extend::Pad` esborrata, e não havia knob para o desfazer.
///
/// ⇒ só a **FASE** é morta no `Clamp`: o `placement_in` passa o canto da caixa e nunca `origin`.
///
/// ⚠️ *Esconder um knob VIVO é o mesmo defeito de mostrar um MORTO, com o sinal trocado* — o
/// artista vê o desenho mudar e não tem o controlo que o mudou.
#[test]
fn the_clamp_mode_hides_only_the_phase_which_is_the_one_it_does_not_read() {
    let visible = |mode: u8, id| {
        state::set_current_fill(Some(FillKind::Pattern), None);
        let mut r = row(1); // Brick: com desfasamento, para o Offset ter direito a aparecer
        r.mode = mode;
        state::set_current_texture_pattern(0, Some(r));
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
            .is_some()
    };
    use ph2d_panel_vector::ids::TexPatKnob as K;
    use ph2d_panel_vector::texture_pattern::kid;

    // A FASE — e só ela — some no `Clamp`, e VOLTA fora dele (esconder não é apagar).
    for (id, what) in [
        (kid(0, K::ShiftX), "o Shift X"),
        (kid(0, K::ShiftY), "o Shift Y"),
    ] {
        assert!(
            !visible(2, id),
            "o Clamp mostra {what}, e ali a colocacao e' DERIVADA da caixa da forma"
        );
        assert!(visible(0, id), "{what} nao VOLTOU fora do Clamp");
    }

    // ⭐ Tudo o resto o `Clamp` LÊ, e por isso continua alcançável em TODOS os modos.
    for (id, what) in [
        (kid(0, K::Tile(0)), "o reticulado"),
        (kid(0, K::Offset), "o desfasamento"),
        (kid(0, K::Width), "a largura"),
        (kid(0, K::Height), "a altura"),
        (kid(0, K::Lock), "o cadeado"),
        (kid(0, K::Gap), "o vao"),
        (kid(0, K::Source), "a arte"),
        (kid(0, K::PickShape), "a forma como arte"),
        (kid(0, K::Angle), "o angulo"),
        (kid(0, K::Mode(0)), "os modos"),
    ] {
        for modo in [0u8, 1, 2] {
            assert!(
                visible(modo, id),
                "o modo {modo} escondeu {what}, que ele USA - o artista ve^ o desenho mudar e nao \
                 tem o controlo que o mudou"
            );
        }
    }
    state::set_current_texture_pattern(0, None);
}

/// ⭐⭐ **A CAIXA É ALIMENTADA PELO ESTADO PUBLICADO** — e não por um literal.
///
/// ⚠️ Este gate nasceu de uma **mutação sobrevivente**: trocar `p.lock_aspect` por `true` na pintura
/// deixava a caixa sempre marcada, e os gates de alcance (que só medem se ela é CLICÁVEL) ficavam
/// todos verdes. *Um controlo que chega ao barramento mas mente sobre o estado dá o mesmo report que
/// um controlo morto.*
///
/// ⚠️⚠️ **Ele lê o FONTE, e a tentativa behavioural foi MEDIDA e falhou:** `host.store().checkbox()`
/// devolve `None`, porque esta caixa é registada como `Button` (o molde do `VECTOR_TRANSFORM_RESIZE_BOX`)
/// e o `checkbox_row` recebe o valor **por argumento**, do estado publicado — ele nunca chega ao
/// store. Mudar o registo para `Checkbox` só para o teste o alcançar mudaria a rota do clique.
#[test]
fn the_lock_checkbox_is_fed_by_the_published_state() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("paint_texture_pattern.rs"),
    )
    .expect("o pintor da seccao");
    let code: String = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let i = code
        .find("kid(ids::TexPatKnob::Lock),")
        .expect("a caixa e' pintada");
    // ⚠️ A linha tem de ser EXACTAMENTE o campo publicado. Uma versão anterior deste gate procurava
    // a substring `p.lock_aspect` e **um `!p.lock_aspect` SOBREVIVEU** — a negação contém a agulha.
    // *Uma verificação por substring aprova o contrário do que ela quer afirmar.*
    let arg = code[i..i + 160]
        .lines()
        .map(str::trim)
        .find(|l| l.contains("lock_aspect"));
    assert_eq!(
        arg,
        Some("p.lock_aspect,"),
        "a caixa deixou de ser alimentada pelo estado publicado tal e qual - ela passa a mentir \
         sobre o cadeado, e nenhum gate de alcance o ve^"
    );
}

/// ⭐⭐⭐ **O AVISO DE COSTURA aparece, e SÓ onde ele tem sujeito** (plano 33, W10).
///
/// Um ladrilho cujo salto na volta passa o joelho medido mostra uma aresta dura em cada fronteira.
/// O app tem os bytes, mediu-o no assado, e diz-lo — com o remédio ao lado.
///
/// # ⚠️ O oráculo é a GEOMETRIA, e não um id
///
/// Uma dica é **texto**: ela não regista hit-rect nenhum (é a lei do `label_line`, e o
/// `architecture_panel_wiring_parity` está certo em não exigir nada dela — não há o que registar).
/// Logo o `painted_rect` **não a vê**. O que se afirma é o que ela **desloca**: a altura do
/// conteúdo do painel. É o mesmo oráculo que o aviso da §11 do Sprite usa.
///
/// # As três metades
///
/// - `Tile` + salto visível ⇒ **fala** (o conteúdo cresce);
/// - `Tile` sem salto ⇒ cala;
/// - `Mirror` ⇒ cala **mesmo com salto**, porque ele fecha a junta por construção — e é ele o
///   remédio que a frase aponta. ⛔ Um aviso que aparece no modo que o cura ensina a ignorá-lo.
/// - `Clamp` ⇒ cala (uma cópia só, não há junta). ⚠️ Ele esconde outras fileiras, então a linha
///   de base dele é **a dele próprio** — comparar com a do `Tile` mediria os knobs escondidos.
#[test]
fn the_seam_hint_shows_only_where_it_has_a_subject() {
    let altura = |mode: u8, visivel: bool| -> f32 {
        state::set_current_fill(Some(FillKind::Pattern), None);
        let mut r = row(0);
        r.mode = mode;
        r.wrap_seam_visible = visivel;
        state::set_current_texture_pattern(0, Some(r));
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        let _ = host.painted_rect::<VectorPanel>(
            &mut st,
            VIEWPORT,
            ph2d_panel_vector::texture_pattern::kid(0, ph2d_panel_vector::ids::TexPatKnob::Source),
        );
        ph2d_panel_vector::last_content_h()
    };
    let calado = altura(0, false);
    let falando = altura(0, true);
    assert!(
        falando > calado + 1.0,
        "o aviso de costura nao DESLOCOU nada ({falando} contra {calado}) - ele nao esta' a ser \
         pintado, e o artista fica com uma aresta dura em cada fronteira sem saber porque'"
    );
    assert!(
        (altura(1, true) - calado).abs() < 0.5,
        "o MIRROR mostrou o aviso - ele fecha a junta por construcao, e avisar no modo que cura \
         ensina o artista a ignorar o aviso"
    );
    assert!(
        (altura(2, true) - altura(2, false)).abs() < 0.5,
        "o CLAMP mostrou o aviso - ali ha' UMA copia e junta nenhuma"
    );
    state::set_current_texture_pattern(0, None);
}

/// ⭐⭐⭐ **A ARTE APAGADA tem nome no painel** (plano 33, W11).
///
/// Sem esta linha, apagar a forma que serve de arte faz a estampa voltar a **cor chapada** — que é
/// exactamente o que um preenchimento sólido correcto parece —, e a secção sobe **inteira e
/// normal** por cima de um vínculo morto: reticulado, tamanho, vão e rotação a oferecerem-se, e
/// nenhum deles com um ladrilho para arrumar.
///
/// ⚠️ **É o desenho que a `line/Vector` já usa para a instância cujo mestre sumiu** (*"main
/// missing"*, uma FRASE e não um botão) — a diferença é que a estampa não tinha nenhum, porque o
/// `PatternSource` não sabe dizer *"sem arte"*.
///
/// # ⚠️ O oráculo é a GEOMETRIA
///
/// Uma dica é texto e não regista hit-rect, logo `painted_rect` não a vê. O que se afirma é o que
/// ela **desloca**. E as duas metades importam: ela tem de **aparecer** quando a arte sumiu, e tem
/// de **calar** quando ela está lá — um aviso que fica ligado é um aviso que se aprende a ignorar.
#[test]
fn the_missing_art_hint_names_the_dead_link() {
    let altura = |sumiu: bool| -> f32 {
        state::set_current_fill(Some(FillKind::Pattern), None);
        let mut r = row(0);
        r.art = if sumiu {
            ph2d_panel_vector::PatternArt::Deleted
        } else {
            ph2d_panel_vector::PatternArt::Ready
        };
        state::set_current_texture_pattern(0, Some(r));
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        let _ = host.painted_rect::<VectorPanel>(
            &mut st,
            VIEWPORT,
            ph2d_panel_vector::texture_pattern::kid(0, ph2d_panel_vector::ids::TexPatKnob::Source),
        );
        ph2d_panel_vector::last_content_h()
    };
    let calado = altura(false);
    let falando = altura(true);
    assert!(
        falando > calado + 1.0,
        "o aviso de arte apagada nao DESLOCOU nada ({falando} contra {calado}) - a estampa volta a \
         cor chapada e o painel nao tem uma palavra a dizer sobre isso"
    );
    state::set_current_texture_pattern(0, None);
}

/// ⚠️ **O aviso fica ACIMA dos dois botões que o resolvem** (plano 33, W11).
///
/// *Source…* e *Use Shape…* são a reparação. Ler o problema imediatamente acima do gesto que o
/// resolve é o que separa um aviso útil de uma queixa — ⛔ no fim da secção, o artista teria de
/// procurar. O oráculo é a posição do primeiro botão de arte: com o aviso ligado ele **desce**.
#[test]
fn the_missing_art_hint_sits_above_the_buttons_that_fix_it() {
    let topo_do_botao = |sumiu: bool| -> f32 {
        state::set_current_fill(Some(FillKind::Pattern), None);
        let mut r = row(0);
        r.art = if sumiu {
            ph2d_panel_vector::PatternArt::Deleted
        } else {
            ph2d_panel_vector::PatternArt::Ready
        };
        state::set_current_texture_pattern(0, Some(r));
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        host.painted_rect::<VectorPanel>(
            &mut st,
            VIEWPORT,
            ph2d_panel_vector::texture_pattern::kid(0, ph2d_panel_vector::ids::TexPatKnob::Source),
        )
        .expect("o botao da arte e' pintado")
        .y
    };
    let sem = topo_do_botao(false);
    let com = topo_do_botao(true);
    assert!(
        com > sem + 1.0,
        "o botao `Source...` nao desceu ({com} contra {sem}) - o aviso nao esta' ACIMA dele, e o \
         artista le^ o problema depois de ja' ter passado pelo gesto que o resolve"
    );
    state::set_current_texture_pattern(0, None);
}

/// ⛔⛔ **A DICA QUEBRA EM VÁRIAS LINHAS, e o que vem a seguir desce por TODAS elas** (plano 33, W11).
///
/// Foi por isto que ela é `paint_text_block` e não `paint_text`: o 9-slice pagou este defeito com um
/// smoke do Enio em 2026-08-22 — *"estas dicas quebram em duas linhas num painel estreito, e avançar
/// `label_font` por elas escrevia o rótulo seguinte por cima"*.
///
/// # ⛔⛔⛔ A 1.ª redacção deste gate era INCAPAZ de apanhar isso, e a mutação provou-o
///
/// Ela comparava um viewport largo com um estreito — e **o painel não segue o viewport**: a sonda
/// `measure_where_the_hint_wraps` imprime `55,56` de deslocamento em TODAS as larguras, de `1600` a
/// `420`. Com `paint_text` o número seria outro, mas seria o mesmo nas duas pontas ⇒ a comparação
/// dava igual nos dois mundos e a mutação **sobreviveu**. *Um gate comparativo não vê um defeito
/// que afecta os dois lados da comparação por igual.*
///
/// # A régua, derivada
///
/// O oráculo é **uma FILEIRA do próprio painel**: a distância entre os dois botões de arte, que são
/// consecutivos. Se a dica desloca mais do que isso, ela ocupou mais de uma linha — que é
/// exactamente a propriedade. ⛔ Um número escrito à mão aqui envelheceria com a tipografia.
#[test]
fn the_hint_pushes_what_follows_it_through_every_line_it_wraps_to() {
    let topo = |id, sumiu: bool| -> f32 {
        state::set_current_fill(Some(FillKind::Pattern), None);
        let mut r = row(0);
        r.art = if sumiu {
            ph2d_panel_vector::PatternArt::Deleted
        } else {
            ph2d_panel_vector::PatternArt::Ready
        };
        state::set_current_texture_pattern(0, Some(r));
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        host.painted_rect::<VectorPanel>(&mut st, VIEWPORT, id)
            .expect("o botao e' pintado")
            .y
    };
    use ph2d_panel_vector::ids::TexPatKnob as K;
    use ph2d_panel_vector::texture_pattern::kid;
    // Uma FILEIRA: os dois botoes de arte sao consecutivos.
    let fileira = topo(kid(0, K::PickShape), false) - topo(kid(0, K::Source), false);
    assert!(
        fileira > 1.0,
        "as duas fileiras de arte colapsaram: {fileira}"
    );

    let deslocamento = topo(kid(0, K::Source), true) - topo(kid(0, K::Source), false);
    assert!(
        deslocamento > fileira + 0.5,
        "a dica deslocou {deslocamento}, que nao passa UMA fileira ({fileira}) - ela quebra em mais          de uma linha no painel real, entao um avanco de uma linha so' escreve a fileira seguinte          POR CIMA dela"
    );
    state::set_current_texture_pattern(0, None);
}

/// Sonda: em que largura de viewport a dica QUEBRA? Imprime o deslocamento por largura.
#[test]
#[ignore = "sonda: imprime, nao afirma"]
fn measure_where_the_hint_wraps() {
    for largura in [
        1600.0_f32, 1200.0, 1000.0, 900.0, 800.0, 700.0, 600.0, 520.0, 460.0, 420.0,
    ] {
        let viewport = Rect {
            x: 0.0,
            y: 0.0,
            w: largura,
            h: 900.0,
        };
        let topo = |sumiu: bool| -> Option<f32> {
            state::set_current_fill(Some(FillKind::Pattern), None);
            let mut r = row(0);
            r.art = if sumiu {
                ph2d_panel_vector::PatternArt::Deleted
            } else {
                ph2d_panel_vector::PatternArt::Ready
            };
            state::set_current_texture_pattern(0, Some(r));
            let mut host = MockPanelHost::with_panel::<VectorPanel>();
            let mut st = VectorPanelState;
            host.painted_rect::<VectorPanel>(
                &mut st,
                viewport,
                ph2d_panel_vector::texture_pattern::kid(
                    0,
                    ph2d_panel_vector::ids::TexPatKnob::Source,
                ),
            )
            .map(|r| r.y)
        };
        match (topo(false), topo(true)) {
            (Some(a), Some(b)) => {
                println!("viewport {largura:>6.0} -> deslocamento {:>6.2}", b - a)
            }
            _ => println!("viewport {largura:>6.0} -> o botao nao e' pintado"),
        }
    }
    state::set_current_texture_pattern(0, None);
}

/// ⭐⭐⭐ **O PAINEL É O ESCOLHEDOR DA ARTE, e ele fala DUAS frases** (report do Enio, 2026-08-30:
/// *"ao apertar pattern o usuário é obrigado a selecionar uma img no dialog. não tem a opção de
/// usar shape até que se use a img em pattern"*).
///
/// # As duas metades, e nenhuma se mede sozinha
///
/// **A 1.ª é a reachability.** Um padrão que acabou de nascer tem de mostrar os DOIS botões de arte
/// — *Source…* (imagem) e *Use Shape…* (forma do documento). Enquanto o chip escolhia pelo artista,
/// esta secção só existia **depois** de a escolha estar feita, e a forma ficava atrás da imagem.
///
/// **A 2.ª é a frase.** O estado *nunca escolhida* e o estado *foi apagada* pintam o mesmo aviso e
/// pedem palavras opostas: uma convida, a outra alarma. Se alguém colapsar os dois num bit outra
/// vez, a igualdade abaixo apanha-o — a sentença passaria a ser a mesma.
///
/// ⚠️ **O `Ready` é o CONTROLO:** sem ele, um painel que pintasse o aviso SEMPRE passaria as duas
/// primeiras afirmações.
#[test]
fn a_pattern_with_no_art_yet_offers_both_doors_and_says_a_different_sentence() {
    let botoes = |art: ph2d_panel_vector::PatternArt| -> (Option<f32>, Option<f32>) {
        state::set_current_fill(Some(FillKind::Pattern), None);
        let mut r = row(0);
        r.art = art;
        state::set_current_texture_pattern(0, Some(r));
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut st = VectorPanelState;
        let topo = |host: &mut MockPanelHost, st: &mut VectorPanelState, k| {
            host.painted_rect::<VectorPanel>(
                st,
                VIEWPORT,
                ph2d_panel_vector::texture_pattern::kid(0, k),
            )
            .map(|r| r.y)
        };
        (
            topo(
                &mut host,
                &mut st,
                ph2d_panel_vector::ids::TexPatKnob::Source,
            ),
            topo(
                &mut host,
                &mut st,
                ph2d_panel_vector::ids::TexPatKnob::PickShape,
            ),
        )
    };
    // 1. Acabou de nascer: as DUAS portas estão na tela.
    let (img, forma) = botoes(ph2d_panel_vector::PatternArt::NotChosen);
    let img = img.expect(
        "um padrao sem arte escolhida nao pinta `Source...` - o caminho da imagem fica inalcancavel",
    );
    let forma = forma.expect(
        "um padrao sem arte escolhida nao pinta `Use Shape...` - e' EXACTAMENTE o report de 30/08: \
         a arte-forma fica atras da arte-imagem",
    );
    assert!(forma > img, "as duas portas colapsaram numa posicao so'");
    // 2. E o aviso empurra-as para baixo — ele existe, e vem ANTES delas.
    let (pronto, _) = botoes(ph2d_panel_vector::PatternArt::Ready);
    let pronto = pronto.expect("o botao da arte e' sempre pintado");
    assert!(
        img > pronto + 1.0,
        "o aviso de `NotChosen` nao empurrou os botoes ({img} contra {pronto}) - ou ele nao e' \
         pintado, ou nao esta' acima do gesto que o resolve"
    );
    // 3. ⭐ E as DUAS frases são diferentes. Colapsá-las num bit acusa quem carregou no chip de ter
    //    apagado uma forma que ele nunca teve.
    assert_ne!(
        ph2d_i18n::tr("panel.vector.texpat.art_not_chosen.hint"),
        ph2d_i18n::tr("panel.vector.texpat.art_missing.hint"),
        "os dois estados da arte dizem a MESMA frase - um deles esta' a mentir ao artista"
    );
}
