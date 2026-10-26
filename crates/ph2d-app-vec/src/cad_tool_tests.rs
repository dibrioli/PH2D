//! Os gates da tool Regras (W4.6, `docs/BlenderConstraints/04_W4_cad.md` §3.7), pelo caminho da shell: o press pela
//! porta `shape_handles_live::on::press`, o botão pela fila do painel, o quadro por [`frame`], o Delete por
//! `apaga::delete` — o testkit não conduz o canvas.

use super::*;
use crate::cad_tool_plano::{Entrada, plano};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::panel::Panel;
use ph2d_text::TextSystem;
use ph2d_vec_scene::{VecPath, VecVertex, VecViewState, VertId};
use ph2d_vector::Affine;

/// 1 unidade do mundo = 100 px: os alvos ficam longe uns dos outros para a mão.
const ESCALA: f64 = 100.0;

pub(crate) struct Mesa {
    pub scene: VecScene,
    pub vec: VecState,
    pub xf: VecXforms,
    pub hero: HeroScreen,
    pub text: TextSystem,
    pub sim: SimWorld,
    pub path: VecPathId,
}

/// Um quadrilátero 2×1 com o lado esquerdo INCLINADO (de (0,1; 1) a (0; 0)), a tool Regras na mão.
pub(crate) fn mesa() -> Mesa {
    let mut scene = VecScene::new();
    let path = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [2.0, 0.0], [2.0, 1.0], [0.1, 1.0]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
        closed: true,
        ..VecPath::default()
    });
    scene.normaliza_ids();
    let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
    <ph2d_panel_vector::VectorPanel as Panel>::populate(&mut hero.store);
    let mut vec = VecState::default();
    vec.draw_config.mode = DrawMode::Rules;
    let _ = state_cad::drena();
    Mesa {
        scene,
        vec,
        xf: VecXforms::default(),
        hero,
        text: TextSystem::without_system_fonts(),
        sim: SimWorld::default(),
        path,
    }
}

impl Mesa {
    pub fn v(&self, i: u32) -> PontoCad {
        PontoCad::Vert(self.path, VertId(i))
    }
    pub fn lado(&self, i: u32, j: u32) -> SegCad {
        SegCad(self.v(i), self.v(j))
    }
    pub fn pos(&self, i: u32) -> [f64; 2] {
        self.scene
            .path(self.path)
            .and_then(|p| p.vert_by_id(VertId(i)))
            .map(|v| v.anchor)
            .expect("o vértice")
    }
    /// O plano deste quadro (a análise síncrona: a do `CadCores` corre noutro fio), guardado como o desenho guarda.
    pub fn desenha(&mut self, ativa: bool) {
        let ids: Vec<VecPathId> = vec![self.path];
        let folha = Folha::new(
            ids.iter()
                .filter_map(|&id| self.scene.path(id).map(|p| (p, xform_of(&self.xf, id)))),
        );
        let estado = ph2d_vec_constraints::estado(&folha, self.scene.regras());
        let view = VecViewState::default();
        let e = Entrada {
            scene: &self.scene,
            view: &view,
            xf: &self.xf,
            cam: Affine::scale(ESCALA),
            estado: Some(&estado),
            ativa,
            u: Unidades::of(&self.hero),
        };
        let p = plano(&e, &mut self.text);
        *self.vec.cad_tool.plano.borrow_mut() = Some(p);
    }
    /// O press no ponto do MUNDO `w`, pela porta que a shell chama.
    pub fn clica(&mut self, w: [f64; 2]) -> bool {
        crate::shape_handles_live::on::press(&mut self.vec, &mut self.sim, &mut self.scene, w, 0.01, 0)
    }
    pub fn quadro(&mut self) {
        frame(&mut self.scene, &mut self.vec, &self.xf, &mut self.hero);
    }
}

fn meio(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0]
}

#[test]
fn a_choice_offers_only_the_rules_it_allows() {
    use Possivel as P;
    let m = mesa();
    let (l, p, c) = (
        Alvo::Lado(m.lado(1, 2)),
        Alvo::Ponto(m.v(1)),
        Alvo::Circulo(m.path),
    );
    assert_eq!(possiveis(&[l]), vec![P::Horizontal, P::Vertical, P::Distancia]);
    assert_eq!(possiveis(&[l, l]), vec![P::Igual, P::Angulo]);
    assert_eq!(possiveis(&[p]), vec![P::Fixo]);
    assert_eq!(
        possiveis(&[p, p]),
        vec![P::Coincidente, P::Horizontal, P::Vertical, P::Distancia]
    );
    assert_eq!(possiveis(&[c]), vec![P::Raio]);
    assert!(possiveis(&[p, l]).is_empty(), "ponto + lado não está na lista do dono");
    assert!(possiveis(&[]).is_empty());
}

/// ⭐ O seam: clicar o lado esquerdo (inclinado) e o botão Vertical ⇒ a regra entra pela porta única e o lado FICA
/// vertical no mesmo quadro; clicar o de baixo e Distance ⇒ a cota nasce com o comprimento de agora e nada se mexe.
#[test]
fn a_rule_made_by_hand_goes_through_junta_regra_and_keeps_the_drawing() {
    let mut m = mesa();
    m.desenha(true);
    let esq = meio(m.pos(4), m.pos(1));
    assert!(m.clica(esq), "o press da tool consome");
    assert_eq!(m.vec.cad_tool.escolha, vec![Alvo::Lado(m.lado(4, 1))]);
    m.quadro();
    assert_eq!(
        state_cad::cad_vista().possiveis,
        vec![Possivel::Horizontal, Possivel::Vertical, Possivel::Distancia]
    );
    state_cad::pede(PedidoCad::Junta(Possivel::Vertical));
    m.quadro();
    assert!(
        matches!(m.scene.regras(), [r] if r.lei == Lei::Vertical(m.lado(4, 1)) && r.id != RegraId::NOVO)
    );
    assert!(
        (m.pos(4)[0] - m.pos(1)[0]).abs() < 1e-6,
        "o lado não ficou vertical: {:?} {:?}",
        m.pos(4),
        m.pos(1)
    );
    assert!(m.vec.cad_tool.escolha.is_empty(), "a escolha fica para trás");
    // A cota nasce com o que o desenho mede: juntar não mexe em nada.
    m.desenha(true);
    let antes: Vec<[f64; 2]> = (1..=4).map(|i| m.pos(i)).collect();
    m.clica(meio(m.pos(1), m.pos(2)));
    state_cad::pede(PedidoCad::Junta(Possivel::Distancia));
    m.quadro();
    let d = m.scene.regras().last().and_then(|r| r.lei.medida());
    let medido = (m.pos(2)[0] - m.pos(1)[0]).hypot(m.pos(2)[1] - m.pos(1)[1]);
    assert!(
        d.is_some_and(|d| (d - medido).abs() < 1e-12),
        "a cota nasceu com {d:?}, o desenho mede {medido}"
    );
    let depois: Vec<[f64; 2]> = (1..=4).map(|i| m.pos(i)).collect();
    assert_eq!(antes, depois, "juntar a cota mexeu no desenho");
}

/// Clicar o número de uma cota escolhe-a, semeia e foca o chip da ②; escrever estica o lado.
#[test]
fn clicking_a_dimension_number_selects_it_and_typing_moves_the_drawing() {
    let mut m = mesa();
    let (a, b) = (m.v(1), m.v(2));
    m.scene.regras_mut().extend([
        ph2d_vec_scene::cad::RegraCad::nova(Lei::Fixo(a, [0.0, 0.0])),
        ph2d_vec_scene::cad::RegraCad::nova(Lei::Horizontal(SegCad(a, b))),
        ph2d_vec_scene::cad::RegraCad::nova(Lei::Distancia(a, b, 2.0)),
    ]);
    m.scene.normaliza_ids();
    let id = m.scene.regras()[2].id;
    m.desenha(true);
    let numero = {
        let p = m.vec.cad_tool.plano.borrow();
        let c = p.as_ref().and_then(|p| p.cotas.first()).expect("a cota");
        [
            f64::from(c.rect.x + c.rect.w / 2.0) / ESCALA,
            f64::from(c.rect.y + c.rect.h / 2.0) / ESCALA,
        ]
    };
    m.clica(numero);
    assert_eq!(m.vec.cad_tool.regra, Some(id), "o número não escolheu a cota");
    m.quadro();
    let u = Unidades::of(&m.hero);
    assert_eq!(m.hero.store.focus_id(), Some(VECTOR_CAD_VALUE), "o chip não ganhou o foco");
    assert_eq!(
        m.hero.store.number_value(VECTOR_CAD_VALUE),
        Some(u.comprimento.value(2.0)),
        "o chip não foi semeado com a medida"
    );
    state_cad::pede(PedidoCad::Medida(u.comprimento.value(3.0)));
    m.quadro();
    assert!((m.pos(2)[0] - 3.0).abs() < 1e-6, "escrever 3 não esticou o lado: {:?}", m.pos(2));
    assert!(m.pos(1)[0].abs() < 1e-6 && m.pos(1)[1].abs() < 1e-6, "o ponto preso andou: {:?}", m.pos(1));
}

/// O Delete com uma regra escolhida apaga SÓ a regra (o desenho fica); sem regra escolhida, o Delete é o de sempre.
#[test]
fn delete_removes_the_chosen_rule_in_one_step() {
    let mut m = mesa();
    let s = m.lado(1, 2);
    m.scene
        .regras_mut()
        .push(ph2d_vec_scene::cad::RegraCad::nova(Lei::Horizontal(s)));
    m.scene.normaliza_ids();
    m.vec.cad_tool.regra = Some(m.scene.regras()[0].id);
    assert!(crate::apaga::delete(&mut m.vec, &mut m.scene));
    assert!(m.scene.regras().is_empty(), "a regra ficou");
    assert!(m.scene.path(m.path).is_some(), "o Delete levou a forma junto");
    assert!(m.vec.cad_tool.regra.is_none());
    // Controlo: sem regra escolhida e nada seleccionado, o Delete da tool não consome.
    assert!(!crate::apaga::delete(&mut m.vec, &mut m.scene));
}

/// Fora da tool, nada fica escolhido nem é oferecido (a ④ das regras não aparece noutra ferramenta).
#[test]
fn leaving_the_tool_drops_the_choice() {
    let mut m = mesa();
    m.desenha(true);
    m.clica(meio(m.pos(1), m.pos(2)));
    m.vec.draw_config.mode = DrawMode::Node;
    m.quadro();
    assert!(m.vec.cad_tool.escolha.is_empty());
    assert_eq!(state_cad::cad_vista(), CadVista::default());
}
