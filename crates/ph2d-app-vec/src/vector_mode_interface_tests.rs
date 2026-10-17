//! As cenas 25/26 fazem o que dizem, até onde dá dentro desta crate: as duas formas pelos MESMOS
//! estágios do smoke (o botão mais largo que alto, com a cor viva), e cada estágio que depende da
//! shell ESPERA pela resposta dela (o que ela publica ao painel), nunca por um número de quadros.
//!
//! ⚠️ Os cliques (*Create*, *Wear*, gravar, o interruptor) vão por `HeroScreen::apply_event`, que só
//! chega ao painel com o registo de painéis instalado — e ele é do processo (`OnceLock`), logo
//! instalá-lo aqui mudaria os outros testes deste binário. O clique do interruptor ⇒ aba Interface
//! é o gate `as_duas_portas_escrevem_a_mesma_aba` da `ph2d-panel-vector`; o resto de ponta a ponta
//! é a foto (`docs/UI_New_and_Simple/vetor_nova_ui/fotos/F17_cena2{5,6}_*.png`).

use super::{CREATE, FIRST, GRAVA_DEFAULT, LAST, PICK, REPOUSO, WEAR, step};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_panel_vector::state;
use ph2d_vec_scene::{Paint, Rgba8, VecPath, VecScene};

struct Cena {
    vec: crate::state::VecState,
    scene: VecScene,
    hero: HeroScreen,
    sim: ph2d_ecs::SimWorld,
}

impl Cena {
    fn new() -> Self {
        ph2d_editor_core::test_support::ensure_panel_registry();
        Self {
            vec: crate::state::VecState::default(),
            scene: VecScene::new(),
            hero: HeroScreen::new(ph2d_editor_core::NodeId(1)),
            sim: ph2d_ecs::SimWorld::default(),
        }
    }

    /// Um quadro: o passo e a porta que torna as formas vivas.
    fn passo(&mut self, s: u8, interface: bool) -> Option<u8> {
        let next = step(
            s,
            &mut self.vec,
            &mut self.scene,
            &self.sim,
            &mut self.hero,
            interface,
        );
        ph2d_vec_entities::entities::sync(&mut self.sim, &mut self.scene, &mut self.vec.entities);
        crate::shape_live::make_committed_shape_live(
            &mut self.sim,
            &mut self.scene,
            &self.vec.entities,
            &mut self.vec.shape,
            false,
        );
        next
    }
}

/// ⭐ **O botão e a estrela nascem pelos estágios, e o botão é um BOTÃO**: rectângulo arredondado
/// mais largo que alto, com a cor viva; a estrela ao lado.
#[test]
fn as_duas_formas_nascem_e_o_botao_e_mais_largo_que_alto() {
    let mut c = Cena::new();
    let mut s = FIRST;
    while s != PICK {
        s = c.passo(s, false).expect("o estágio avança");
    }
    let paths = c.scene.paths();
    assert_eq!(paths.len(), 2);
    let (w, h) = medidas(&paths[0]);
    assert!(w > 2.0 * h, "o botão não é largo: {w} × {h}");
    let (w, h) = medidas(&paths[1]);
    assert!(
        (w / h - 1.0).abs() < 0.3,
        "a estrela não é a forma comum ao lado: {w} × {h}"
    );
    let [r, g, bl] = REPOUSO;
    assert_eq!(paths[0].fill, Some(Paint::Solid(Rgba8::new(r, g, bl, 255))));
    assert_ne!(paths[1].fill, paths[0].fill, "a estrela tem a cor do botão");
}

/// Largura × altura das âncoras.
fn medidas(p: &VecPath) -> (f64, f64) {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for v in &p.verts {
        for k in 0..2 {
            lo[k] = lo[k].min(v.anchor[k]);
            hi[k] = hi[k].max(v.anchor[k]);
        }
    }
    (hi[0] - lo[0], hi[1] - lo[1])
}

/// ⭐ **O *Create* espera o botão SÓ ele seleccionado; o *Wear* espera a seleção ser interface e
/// ter seguido para a cópia** (o *Create* põe uma cópia no lugar). Sem isso, nenhum dos dois anda.
#[test]
fn o_create_e_o_wear_esperam_pela_selecao() {
    let mut c = Cena::new();
    let mut s = FIRST;
    while s != PICK {
        s = c.passo(s, false).expect("o estágio avança");
    }
    let ids: Vec<_> = c.vec.entities.keys().copied().collect();
    c.vec.pen.select_many(&ids);
    assert_eq!(
        c.passo(CREATE, false),
        None,
        "o Create andou com as duas formas"
    );
    c.vec.pen.select_many(&ids[..1]);
    assert_eq!(c.passo(CREATE, false), Some(WEAR));
    // A seleção ainda é o botão e nada foi publicado: o Wear espera.
    state::set_component_state(None);
    assert_eq!(c.passo(WEAR, false), None, "o Wear andou antes do Create");
    // A shell publica a cópia (instância) e a seleção segue para outro caminho.
    state::set_component_state(Some(state::ComponentState {
        is_instance: true,
        ..state::ComponentState::default()
    }));
    assert_eq!(
        c.passo(WEAR, false),
        None,
        "o Wear andou com o mestre na mão"
    );
    c.vec.pen.select_many(&ids[1..]);
    assert_eq!(c.passo(WEAR, false), Some(GRAVA_DEFAULT));
    state::set_component_state(None);
}

fn publica_gravados(recorded: [bool; 4]) {
    state::set_ui_states_state(Some(state::UiStatesState {
        host: Some(String::new()),
        recorded,
        role_labels: Default::default(),
        live: None,
        duration_s: 0.2,
        spring: None,
        preview: None,
        move_all: None,
        easing: ph2d_anim::Easing::LINEAR,
        bindings: Vec::new(),
    }));
}

/// ⭐ **O fim espera o Hover GRAVADO** (o que a shell publica), nas duas cenas — com só o Default,
/// ou nada, não anda.
#[test]
fn o_fim_espera_o_hover_gravado() {
    for interface in [false, true] {
        let mut c = Cena::new();
        state::set_ui_states_state(None);
        assert_eq!(c.passo(LAST, interface), None, "andou sem estados");
        publica_gravados([true, false, false, false]);
        assert_eq!(c.passo(LAST, interface), None, "andou só com o Default");
        publica_gravados([true, true, false, false]);
        assert_eq!(c.passo(LAST, interface), Some(9));
        state::set_ui_states_state(None);
    }
}
