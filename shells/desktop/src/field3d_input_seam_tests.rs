//! Os gates da **COSTURA** do gesto — o caminho inteiro: ponteiro, alça, lei, intent.
//!
//! ⭐ **A costura ponteiro → gizmo → peça**, no caminho de produção inteiro.
//!
//! ⚠️ **É este o gate que a `DIRETIVA_IMPLEMENTACAO` §1 exige**, e não os dois de cima. Ele pergunta
//! *"clicar numa seta faz a peça andar?"* — a pergunta que a lei pura e a pintura, cada uma verde no
//! seu canto, **não** respondem. A causa nº 1 da semana perdida no Painter foi exatamente esta:
//! costura não-testada, com os dois lados dela corretos.

//! ⚠️ Módulo-filho do arquivo de gates da entrada: `use super::*` traz as fixtures do pai, que
//! continuam a existir **uma vez**.

use crate::field3d_gizmo::{self, Handle};
use crate::field3d_input::{advance, begin, hot_handle};
use crate::field3d_smoke::{Drag, set_armed_by_panel, with_smoke};
use ph2d_field_render::Screen;

const AREA: ph2d_editor::zones::Rect = ph2d_editor::zones::Rect {
    x: 40.0,
    y: 24.0,
    w: 800.0,
    h: 600.0,
};

/// Arma o módulo e põe o smoke num estado de quadro: com área desenhada e com o gizmo ancorado
/// na origem. É o que a ponte com a cena publica.
fn armed<R>(f: impl FnOnce(&mut crate::field3d_smoke::Smoke) -> R) -> R {
    set_armed_by_panel(true);
    with_smoke(|s| {
        s.area = Some(AREA);
        s.gizmo = Some(field3d_gizmo::Anchor::global(7, [0.0, 0.0, 0.0]));
        s.pending_move = None;
        s.drag = None;
        s.gizmo_hot = None;
        f(s)
    })
    .expect("o módulo está armado")
}

fn translation_of(m: field3d_gizmo::Motion) -> [f32; 3] {
    match m {
        field3d_gizmo::Motion::Translate(d) => d,
        other => panic!("o modo de mover pede translação, e veio {other:?}"),
    }
}

fn screen_of(s: &crate::field3d_smoke::Smoke) -> Screen {
    Screen::new(AREA.w as u32, AREA.h as u32, s.cam.half_extent)
}

/// O ponto de janela, em pixels, do meio da haste do eixo `n`.
fn mid_of_axis(s: &crate::field3d_smoke::Smoke, n: usize) -> (f32, f32) {
    let anchor = s.gizmo.expect("ancorado");
    let handles = field3d_gizmo::project(anchor, &s.cam, screen_of(s), s.gizmo_mode);
    let h = handles
        .iter()
        .find(|h| h.handle == Handle::Axis(n))
        .expect("o eixo existe");
    let field3d_gizmo::Shape::Arrow { from, to } = h.shape else {
        panic!("um eixo é uma seta");
    };
    (
        AREA.x + (from[0] + to[0]) * 0.5,
        AREA.y + (from[1] + to[1]) * 0.5,
    )
}

/// ⭐ **Carregar numa seta agarra a seta — e não orbita a câmera.**
#[test]
fn pressing_on_an_arrow_grabs_it_instead_of_orbiting() {
    armed(|s| {
        let p = mid_of_axis(s, 0);
        let before = s.cam;
        assert!(begin(s, winit::event::MouseButton::Left, Drag::Orbit, p));
        assert_eq!(
            s.drag,
            Some(Drag::Gizmo(Handle::Axis(0))),
            "o clique sobre a seta virou gesto de câmera — a alça está pintada e morta"
        );
        assert_eq!(hot_handle(s), Some(Handle::Axis(0)), "e ela acende");

        // E arrastar move a PEÇA, não a vista.
        assert!(advance(s, p.0 + 60.0, p.1));
        assert_eq!(s.cam, before, "a câmera não pode ter-se mexido");
        let (entity, motion) = s.pending_move.expect("o arrasto tem de pedir um movimento");
        assert_eq!(entity, 7, "e tem de pedi-lo para a entidade da âncora");
        assert!(
            !motion.is_idle(),
            "o pedido saiu vazio: {motion:?} — o ponteiro não chegou à lei do arrasto"
        );
    });
}

/// **Longe do gizmo, o botão esquerdo continua a orbitar.** Sem isto o gizmo sequestraria a
/// navegação da janela inteira.
#[test]
fn pressing_away_from_the_gizmo_still_orbits() {
    armed(|s| {
        let far = (AREA.x + AREA.w - 5.0, AREA.y + 5.0);
        assert!(begin(s, winit::event::MouseButton::Left, Drag::Orbit, far));
        assert_eq!(s.drag, Some(Drag::Orbit));
        assert!(s.pending_move.is_none());
    });
}

/// ⚠️ **O botão DIREITO orbita mesmo por cima da alça** — é a saída de quem quer girar a vista
/// sem primeiro tirar o rato de cima da peça.
#[test]
fn the_right_button_orbits_even_over_a_handle() {
    armed(|s| {
        let p = mid_of_axis(s, 0);
        assert!(begin(s, winit::event::MouseButton::Right, Drag::Orbit, p));
        assert_eq!(s.drag, Some(Drag::Orbit));
    });
}

/// ⭐ **Os pedidos ACUMULAM entre quadros.**
///
/// ⚠️ Entre dois quadros chegam vários eventos de ponteiro. Guardar só o último faria a peça
/// andar menos do que a mão — devagar, e **só quando o rato vai depressa**, que é o defeito mais
/// difícil de acreditar quando alguém o reporta.
#[test]
fn pointer_events_between_two_frames_add_up() {
    armed(|s| {
        let p = mid_of_axis(s, 0);
        begin(s, winit::event::MouseButton::Left, Drag::Orbit, p);
        advance(s, p.0 + 30.0, p.1);
        let one = translation_of(s.pending_move.expect("primeiro evento").1);
        advance(s, p.0 + 60.0, p.1);
        let two = translation_of(s.pending_move.expect("segundo evento").1);
        assert!(
            (two[0] - one[0] * 2.0).abs() < one[0].abs() * 1e-3,
            "dois passos iguais têm de somar: {one:?} depois {two:?}"
        );
    });
}

/// **Sem arrasto, mover o rato acende a alça e NÃO consome o evento.**
///
/// ⚠️ As duas metades importam: sem a primeira o artista não sabe o que vai agarrar; com a
/// segunda invertida, a janela 3D engoliria todo movimento de rato do app 2D.
#[test]
fn hover_lights_the_handle_without_swallowing_the_event() {
    armed(|s| {
        let p = mid_of_axis(s, 1);
        assert!(!advance(s, p.0, p.1), "hover não é um gesto desta janela");
        assert_eq!(hot_handle(s), Some(Handle::Axis(1)));
        assert!(!advance(s, AREA.x + 2.0, AREA.y + 2.0));
        assert_eq!(hot_handle(s), None, "e apaga-se ao sair");
    });
}

/// ⭐ **Soltar sem ter arrastado é um CLIQUE**, e um clique pede uma seleção.
///
/// ⚠️ É a metade de entrada da seleção por clique. A outra (de quem é o ponto) precisa do MUNDO
/// e vive na ponte; sem esta, ela nunca é chamada — e o gizmo continuaria a só chegar pela
/// Hierarquia.
#[test]
fn a_press_and_release_without_dragging_asks_for_a_pick() {
    armed(|s| {
        let far = (AREA.x + AREA.w - 60.0, AREA.y + 40.0);
        begin(s, winit::event::MouseButton::Left, Drag::Orbit, far);
        s.last_pointer = far;
        crate::field3d_input::finish_for_test(s);
        let px = s.pending_pick.expect("um clique tem de pedir uma seleção");
        assert!(
            (px[0] - (far.0 - AREA.x)).abs() < 0.01 && (px[1] - (far.1 - AREA.y)).abs() < 0.01,
            "o pixel pedido tem de ser o do CLIQUE, no referencial da área: {px:?}"
        );
    });
}

/// ⚠️ **Um arrasto NÃO é um clique** — girar a vista não pode trocar a seleção.
#[test]
fn dragging_the_camera_is_not_a_click() {
    armed(|s| {
        let far = (AREA.x + AREA.w - 60.0, AREA.y + 40.0);
        begin(s, winit::event::MouseButton::Left, Drag::Orbit, far);
        advance(s, far.0 + 40.0, far.1 + 30.0);
        crate::field3d_input::finish_for_test(s);
        assert!(
            s.pending_pick.is_none(),
            "orbitar a vista pediu uma seleção — o limiar de clique não está a morder"
        );
    });
}

/// ⚠️ **Soltar uma ALÇA do gizmo nunca é uma seleção.**
///
/// Sem isto, mover um objeto trocaria a seleção para o que estivesse por baixo dele no fim do
/// gesto — e o artista perderia o objeto que acabou de posicionar.
#[test]
fn releasing_a_gizmo_handle_is_never_a_selection() {
    armed(|s| {
        let p = mid_of_axis(s, 0);
        begin(s, winit::event::MouseButton::Left, Drag::Orbit, p);
        assert!(matches!(s.drag, Some(Drag::Gizmo(_))));
        crate::field3d_input::finish_for_test(s);
        assert!(s.pending_pick.is_none());
    });
}

/// ⚠️ **Durante o arrasto o realce fica na alça AGARRADA**, mesmo com o cursor longe dela — que
/// é onde o cursor está, porque arrastar é isso.
#[test]
fn the_grabbed_handle_stays_lit_while_the_cursor_walks_away() {
    armed(|s| {
        let p = mid_of_axis(s, 2);
        begin(s, winit::event::MouseButton::Left, Drag::Orbit, p);
        advance(s, AREA.x + AREA.w - 3.0, AREA.y + AREA.h - 3.0);
        assert_eq!(hot_handle(s), Some(Handle::Axis(2)));
    });
}

// ───────── W49: o gizmo de NAVEGAÇÃO, pela costura inteira ─────────

/// Onde uma bola está, em coordenadas de **janela** (a área tem canto em `AREA.x/y`).
fn ball_at(s: &crate::field3d_smoke::Smoke, v: crate::field3d_views::Standard) -> (f32, f32) {
    let b = crate::field3d_navball::balls(&s.cam, AREA, crate::field3d_smoke::safe_of(s))
        .into_iter()
        .find(|b| b.view == v)
        .expect("as seis estão sempre lá");
    (AREA.x + b.at[0], AREA.y + b.at[1])
}

/// ⭐⭐ **CLICAR NUMA BOLA LEVA A CÂMERA ÀQUELA VISTA** — a costura inteira: `begin` → `finish`.
///
/// ⚠️ **É o gate que a W48 me ensinou a escrever primeiro.** Lá eu provei o tratador e entreguei
/// botões mortos; aqui o caminho é o do ponteiro, de ponta a ponta, e nenhuma intenção é empurrada
/// à mão.
#[test]
fn clicking_a_navball_takes_the_camera_to_that_view() {
    for v in crate::field3d_views::Standard::ALL {
        armed(|s| {
            // Longe da vista, de propósito: sem isto o gate passaria com um clique que não faz nada.
            s.cam.rotation = ph2d_field_render::Orbit::default().rotation;
            s.nav_press = None;
            let at = ball_at(s, v);
            assert!(
                begin(s, winit::event::MouseButton::Left, Drag::Orbit, at),
                "{v:?}: o gizmo de navegação não pegou o botão"
            );
            assert_eq!(
                s.nav_press,
                Some(v),
                "{v:?}: a pegada não guardou a bola sob o cursor"
            );
            crate::field3d_input::finish_for_test(s);
            // ⭐ **A câmera VIAJA** (W51): soltar pede a viagem, não salta. As duas metades no
            // mesmo gate, de propósito — sem a primeira, um salto voltaria a passar; sem a
            // segunda, uma viagem que nunca chegasse ao destino também.
            assert!(
                s.flight.is_some(),
                "{v:?}: soltar em cima da bola tinha de PEDIR uma viagem, não saltar"
            );
            assert_eq!(
                crate::field3d_views::named_view(&s.cam),
                None,
                "{v:?}: a câmera saltou para a vista em vez de partir para ela"
            );
        });
        // A mola da casa serve o progresso; aqui empurra-se até ao fim, que é o que ela faz.
        crate::field3d_smoke::note_flight_progress(1.0);
        armed(|s| {
            assert_eq!(
                crate::field3d_views::named_view(&s.cam),
                Some(v),
                "{v:?}: a viagem terminou fora da vista dela"
            );
            assert!(
                s.flight.is_none(),
                "{v:?}: a viagem não largou o voo ao chegar"
            );
        });
    }
}

/// ⭐⭐ **ARRASTAR A PARTIR DO GIZMO ORBITA, E NÃO ESCOLHE VISTA.**
///
/// ⚠️ É o gesto que a pesquisa da referência mede como o **rápido** (quase 2× o clique), e ele tem
/// de ganhar do clique quando a mão se mexe: sem isto, um arrasto começado por acidente em cima do
/// widget teleportaria a câmera ao soltar — e essa é a forma mais assustadora de um gizmo falhar.
#[test]
fn dragging_from_the_navball_orbits_instead_of_snapping() {
    armed(|s| {
        s.cam.rotation = ph2d_field_render::Orbit::default().rotation;
        let start = s.cam.rotation;
        let at = ball_at(s, crate::field3d_views::Standard::Front);
        assert!(begin(s, winit::event::MouseButton::Left, Drag::Orbit, at));
        s.pending_pick = None;
        advance(s, at.0 + 60.0, at.1 + 12.0);
        assert_ne!(s.cam.rotation, start, "o arrasto tinha de orbitar");
        crate::field3d_input::finish_for_test(s);
        // ⚠️ **E não pede seleção na peça.** Achado por uma mutação sobrevivente: a guarda que o
        // impede só é load-bearing neste caso — o do arrasto que COMEÇOU no widget — e nenhum gate
        // o media. Um arrasto de câmera que selecionasse o que está por baixo do gizmo ao soltar é
        // um gesto a fazer duas coisas, e a segunda invisível.
        assert_eq!(
            s.pending_pick, None,
            "o arrasto começado no gizmo pediu uma seleção na peça ao soltar"
        );
        assert_eq!(
            crate::field3d_views::named_view(&s.cam),
            None,
            "o arrasto acabou numa vista NOMEADA — ele saltou para a bola em vez de orbitar"
        );
    });
}

/// ⚠️ **Um clique no gizmo não é um clique na PEÇA.** O widget fica por cima dela; sem esta
/// precedência, escolher uma vista selecionaria também o que estivesse por baixo.
#[test]
fn a_click_on_the_navball_never_reaches_the_part() {
    armed(|s| {
        s.pending_pick = None;
        let at = ball_at(s, crate::field3d_views::Standard::Top);
        assert!(begin(s, winit::event::MouseButton::Left, Drag::Orbit, at));
        crate::field3d_input::finish_for_test(s);
        assert_eq!(
            s.pending_pick, None,
            "o clique no gizmo pediu também uma seleção na peça"
        );
    });
}

/// ⚠️ **E fora do widget o gesto continua a ser da peça** — o controle que separa *"o gizmo ganha"*
/// de *"o gizmo come tudo"*.
#[test]
fn a_click_away_from_the_navball_still_belongs_to_the_part() {
    armed(|s| {
        s.pending_pick = None;
        s.nav_press = None;
        // O canto oposto ao widget, bem longe dele.
        let at = (AREA.x + 20.0, AREA.y + AREA.h - 20.0);
        assert!(begin(s, winit::event::MouseButton::Left, Drag::Orbit, at));
        assert_eq!(
            s.nav_press, None,
            "o gizmo pegou um clique que não era dele"
        );
        crate::field3d_input::finish_for_test(s);
        assert!(
            s.pending_pick.is_some(),
            "um clique fora do gizmo tem de continuar a pedir a seleção da peça"
        );
    });
}

/// ⭐⭐ **A PARTE LIVRE PUBLICADA PELO SHELL MOVE O GIZMO DE VERDADE** (W50).
///
/// ⚠️ A lei do `safe_corner` é pura e tem os gates dela; este mede a **costura**: que o valor
/// publicado chega ao gesto. Sem ele, o shell podia deixar de publicar e a lei continuaria verde no
/// canto dela — que é exatamente a forma como a W48 me apanhou.
#[test]
fn the_published_safe_rect_moves_the_gizmo() {
    // ⚠️ **O `note_safe` corre FORA do `armed`**, e tem de correr: ele próprio entra por
    // `with_smoke`, e chamá-lo de dentro de outro `with_smoke` é um `RefCell` re-entrante — o teste
    // rebenta antes de afirmar seja o que for. *A primeira escrita fazia-o de dentro, e o arnês de
    // mutação deu-a por «apanhada» porque só exigia VERMELHO, nunca o verde antes dele.*
    let bare = armed(|s| {
        s.safe = None;
        crate::field3d_navball::centre_in(AREA, crate::field3d_smoke::safe_of(s))
    });

    // Um painel de 300 px encostado à direita, como o MODEL aberto — pela **porta do módulo**.
    let panel = ph2d_editor::zones::Rect::new(AREA.x + AREA.w - 300.0, AREA.y, 300.0, AREA.h);
    crate::field3d_smoke::note_safe(crate::field3d_navball::safe_corner(AREA, &[panel]));

    armed(|s| {
        let moved = crate::field3d_navball::centre_in(AREA, crate::field3d_smoke::safe_of(s));
        assert!(
            (bare[0] - moved[0] - 300.0).abs() < 0.01,
            "a parte livre publicada não moveu o gizmo: {bare:?} -> {moved:?}"
        );
        // …e o gesto segue-o: o clique tem de cair no sítio NOVO, não no antigo.
        let safe = crate::field3d_smoke::safe_of(s);
        assert!(
            crate::field3d_navball::hits_widget(AREA, safe, moved),
            "o gizmo desenhou-se num sítio e o clique ficou noutro"
        );
        assert!(
            !crate::field3d_navball::hits_widget(AREA, safe, bare),
            "o gizmo ainda apanha cliques no sítio ANTIGO — pintado num sítio, apontável noutro"
        );
        s.safe = None;
    });
}

/// ⭐⭐ **A MÃO CANCELA A VIAGEM** (W51).
///
/// ⚠️ É a lei que o módulo já aplica ao refinamento do preview (*"um refinamento cede à mão"*) e ao
/// prato giratório. Uma câmera a viajar por baixo de um arrasto é o app a disputar o rato com o
/// artista — e o sintoma seria a peça a fugir do cursor durante 200 ms, uma vez a cada tantas.
#[test]
fn the_hand_cancels_a_trip_in_flight() {
    armed(|s| {
        s.cam = ph2d_field_render::Orbit::default();
        crate::field3d_input::fly_to_view(s, crate::field3d_views::Standard::Top);
        assert!(s.flight.is_some(), "o controle: a viagem partiu");

        // Um arrasto: começa e move.
        let at = (AREA.x + 100.0, AREA.y + 100.0);
        crate::field3d_input::begin(s, winit::event::MouseButton::Left, Drag::Orbit, at);
        crate::field3d_input::advance(s, at.0 + 30.0, at.1);
        assert!(
            s.flight.is_none(),
            "a viagem continuou por baixo do arrasto — o app está a disputar o rato"
        );
    });
}

/// ⚠️ **Uma viagem para onde já se está NÃO parte** — senão a mola acende por nada, e um segundo
/// clique no mesmo chip daria 200 ms de imobilidade que lêem como o botão não ter funcionado.
#[test]
fn asking_for_the_view_we_are_already_in_starts_no_trip() {
    armed(|s| {
        s.flight = None;
        crate::field3d_input::fly_to_view(s, crate::field3d_views::Standard::Front);
        crate::field3d_smoke::advance_flight(s, 1.0);
        assert!(s.flight.is_none());
        crate::field3d_input::fly_to_view(s, crate::field3d_views::Standard::Front);
        assert!(
            s.flight.is_none(),
            "pedir a vista em que já se está fez partir uma viagem de zero graus"
        );
    });
}

/// ⭐ **Cada viagem tem um id NOVO** — a mola da casa lembra-se por id, e reusar um faria a segunda
/// continuar de onde a primeira parou (isto é: chegar instantaneamente).
#[test]
fn each_trip_gets_a_fresh_track_id() {
    armed(|s| {
        s.flight = None;
        s.cam = ph2d_field_render::Orbit::default();
        crate::field3d_input::fly_to_view(s, crate::field3d_views::Standard::Top);
        let first = s.flight_gen;
        crate::field3d_smoke::advance_flight(s, 1.0);
        crate::field3d_input::fly_to_view(s, crate::field3d_views::Standard::Left);
        assert_ne!(
            s.flight_gen, first,
            "a segunda viagem reusou a track da primeira"
        );
    });
}
