//! Gates do VÍNCULO texto ↔ caminho-guia ([`super`]).
//!
//! Os oráculos são **geométricos e de produto**: o texto sai onde o caminho está, não onde uma
//! fórmula diz. O motor já tem os gates de referencial ([`ph2d_vec_scene::text_path`]) e de
//! layout ([`crate::vec_glyph`]); aqui a pergunta é a costura — *o vínculo chega ao re-cook?*

use super::*;
use ph2d_ecs::{VecShape, VecTextParams};
use ph2d_vec_scene::{VecPath, VecPathId, VecVertex, VertexKind};

/// Um texto e um caminho na cena, já sincronizados com entidades.
struct Fix {
    sim: SimWorld,
    scene: VecScene,
    map: VecEntityMap,
    text: VecPathId,
    guide: VecPathId,
}

fn params(text: &str) -> VecTextParams {
    VecTextParams {
        text: text.to_owned(),
        origin: [0.0, 0.0],
        family: None,
        size: 0.5,
        weight: 400.0,
        line_height: 1.2,
        tracking: 0.0,
        align: 0,
        axes: Vec::new(),
    }
}

/// Um quarto de círculo de raio `r` — curvo (a tangente vira) e com tangente boa nas pontas.
fn arc_verts(r: f64) -> Vec<VecVertex> {
    const K: f64 = 0.552_284_749_830_793_4;
    vec![
        VecVertex {
            anchor: [r, 0.0],
            in_handle: [r, -K * r],
            out_handle: [r, K * r],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: [0.0, r],
            in_handle: [K * r, r],
            out_handle: [-K * r, r],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
    ]
}

fn fixture() -> Fix {
    let mut sim = SimWorld::default();
    let mut scene = VecScene::default();
    let text = scene.push_path(VecPath {
        verts: arc_verts(1.0),
        closed: false,
        ..Default::default()
    });
    let guide = scene.push_path(VecPath {
        verts: arc_verts(6.0),
        closed: false,
        ..Default::default()
    });
    let mut map = VecEntityMap::new();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let te = Entity::from_bits(map[&text]);
    sim.world_mut()
        .entity_mut(te)
        .insert(VecShape::Text(params("PATH")));
    Fix {
        sim,
        scene,
        map,
        text,
        guide,
    }
}

impl Fix {
    fn text_entity(&self) -> Entity {
        Entity::from_bits(self.map[&self.text])
    }

    fn link(&mut self, start_offset: f32, flip: bool) {
        let e = self.text_entity();
        self.sim.world_mut().entity_mut(e).insert(VecTextPath {
            path: self.guide,
            start_offset,
            flip,
        });
    }

    fn recook(&mut self) {
        let e = self.text_entity();
        crate::vec_text_object::recook_text_object(
            &mut self.sim,
            &mut self.scene,
            &self.map,
            self.text,
            e,
            params("PATH"),
        );
    }

    /// A menor e a maior distância de uma âncora do texto à origem.
    fn radial_band(&self) -> (f64, f64) {
        self.scene
            .paths()
            .iter()
            .find(|p| p.id == self.text)
            .expect("texto")
            .verts_all()
            .map(|v| v.anchor[0].hypot(v.anchor[1]))
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), d| {
                (lo.min(d), hi.max(d))
            })
    }
}

/// **Sem o componente, nada muda.** É a lei #1 desta wave, e é o que torna o vínculo um
/// componente OPCIONAL em vez de um campo: *texto reto é o que não tem o componente* é uma
/// afirmação que a ausência garante, enquanto *texto reto é `on_path: None`* é uma que cada
/// leitor tem de lembrar.
#[test]
fn a_text_with_no_link_is_cooked_exactly_as_before() {
    let mut f = fixture();
    f.recook();
    let straight: Vec<[f64; 2]> = f
        .scene
        .paths()
        .iter()
        .find(|p| p.id == f.text)
        .expect("texto")
        .verts_all()
        .map(|v| v.anchor)
        .collect();
    // O texto reto cozinha CENTRADO no local 0 — a assinatura do caminho de hoje.
    let cx = straight.iter().map(|a| a[0]).sum::<f64>() / straight.len() as f64;
    assert!(
        cx.abs() < 0.6,
        "texto reto continua centrado perto do local 0: {cx:.3}"
    );
    // E o `Transform` NÃO foi tocado (o texto reto preserva a pose do usuário).
    let t = *f.sim.world().get::<Transform>(f.text_entity()).expect("t");
    assert_eq!(t, Transform::default());
}

/// **Vinculado, o texto sai NA CURVA** — e o oráculo é o raio, não a fórmula: um texto de
/// tamanho 0,5 sobre um arco de raio 6 tem de ter TODAS as âncoras perto de 6, enquanto o
/// mesmo texto reto se espalha de 0 a ~1,4 de largura em volta da origem.
#[test]
fn a_linked_text_rides_the_guide() {
    let mut f = fixture();
    f.link(0.0, false);
    f.recook();
    let (lo, hi) = f.radial_band();
    assert!(
        lo > 5.3 && hi < 6.2,
        "as âncoras deviam cair na banda do raio 6: ({lo:.3}, {hi:.3})"
    );
}

/// **Mover o CAMINHO move o texto** — a metade visível da feature. O guia é lido cozido *e
/// assado pela pose de mundo dele*; ler a geometria local faria o texto ficar onde o caminho
/// não está, e o artista veria os dois separarem-se ao arrastar.
#[test]
fn moving_the_guide_moves_the_text() {
    let mut f = fixture();
    f.link(0.0, false);
    f.recook();
    let before = f.radial_band();
    // Empurra o CAMINHO 10 unidades para o lado.
    let ge = Entity::from_bits(f.map[&f.guide]);
    if let Some(mut t) = f.sim.world_mut().get_mut::<Transform>(ge) {
        t.translation.x = 10.0;
    }
    f.recook();
    let after = f.radial_band();
    assert!(
        after.0 > before.1,
        "o texto tem de ter ido junto: antes {before:?}, depois {after:?}"
    );
}

/// **Um texto vinculado é devolvido à IDENTIDADE, a cada re-cook.**
///
/// A geometria dele é MUNDO, então uma pose por cima a aplicaria duas vezes. O gizmo continua
/// a existir sobre a entidade — re-impor a identidade é o que torna um arrasto **inócuo** em
/// vez de errado, que é o precedente do `connector_live`.
#[test]
fn a_linked_text_is_pinned_to_the_identity() {
    let mut f = fixture();
    f.link(0.0, false);
    f.recook();
    // O artista arrasta o texto pelo gizmo…
    let te = f.text_entity();
    if let Some(mut t) = f.sim.world_mut().get_mut::<Transform>(te) {
        t.translation.x = 7.0;
        t.translation.y = -3.0;
    }
    f.recook();
    assert_eq!(
        *f.sim.world().get::<Transform>(te).expect("t"),
        Transform::IDENTITY,
        "…e o re-cook devolve a identidade"
    );
}

/// **Um guia APAGADO devolve o texto ao layout reto — ele não some.**
///
/// O vínculo guarda um id, e um id pode ficar pendurado (o artista apaga o caminho). A escolha
/// é entre *texto reto* e *nada*: um texto que desaparece porque a curva foi apagada é trabalho
/// perdido sem aviso, e o componente órfão é curado no próximo gesto.
#[test]
fn a_deleted_guide_leaves_the_text_straight_instead_of_gone() {
    let mut f = fixture();
    f.link(0.0, false);
    f.recook();
    assert!(f.radial_band().0 > 5.0, "estava na curva");
    f.scene.remove_path(f.guide);
    f.recook();
    let (lo, _) = f.radial_band();
    assert!(
        lo < 2.0,
        "com o guia apagado o texto volta a ser reto (perto da origem): {lo:.3}"
    );
    assert!(
        f.scene.paths().iter().any(|p| p.id == f.text),
        "e continua a existir"
    );
}

/// **O `start_offset` é uma FRAÇÃO do total** — meia volta do arco põe o texto no meio dele.
///
/// Fração e não distância porque é o `startOffset` do SVG *e* porque sobrevive a editar o
/// caminho. O gate mede a **posição ao longo do arco**, não o número guardado: é a conversão
/// (feita numa porta só) que ele existe para prender.
#[test]
fn the_start_offset_is_a_fraction_of_the_total_length() {
    let mut f = fixture();
    f.link(0.0, false);
    f.recook();
    let at_start = f.radial_band();
    let start_y = f
        .scene
        .paths()
        .iter()
        .find(|p| p.id == f.text)
        .expect("t")
        .verts_all()
        .map(|v| v.anchor[1])
        .fold(f64::NEG_INFINITY, f64::max);
    f.link(0.5, false);
    f.recook();
    let mid_y = f
        .scene
        .paths()
        .iter()
        .find(|p| p.id == f.text)
        .expect("t")
        .verts_all()
        .map(|v| v.anchor[1])
        .fold(f64::NEG_INFINITY, f64::max);
    // O arco vai de (6,0) a (0,6): meio caminho está bem mais alto que o começo.
    assert!(
        mid_y > start_y + 2.0,
        "meia fração do arco tem de subir: {start_y:.3} -> {mid_y:.3}"
    );
    // E continua NA curva (a fração não empurrou o texto para fora).
    assert!(f.radial_band().0 > 5.0, "banda {:?}", at_start);
}

/// **O flip põe o texto do outro lado da BASELINE.** Num arco percorrido de (r,0) para (0,r) a
/// esquerda da marcha aponta para DENTRO; virar troca o lado, e o gate mede o lado — não o
/// booleano.
///
/// ⚠️ Este gate nasceu a pedir um VÃO entre as duas bandas (`flipped.0 > plain.1`) e falhou por
/// **0,0024**. Medido: normal `(5,638 … 6,004)`, virado `(6,001 … 6,367)` — elas **encostam**,
/// e encostam no raio 6, que é a baseline. Não há vão nenhum a esperar: as duas se apoiam na
/// MESMA linha e crescem em sentidos opostos, que é precisamente o que "do outro lado"
/// significa para texto. O oráculo passou a ser esse.
#[test]
fn flipping_puts_the_text_on_the_other_side_of_the_guide() {
    const R: f64 = 6.0;
    let mut f = fixture();
    f.link(0.2, false);
    f.recook();
    let (lo_a, hi_a) = f.radial_band();
    f.link(0.2, true);
    f.recook();
    let (lo_b, hi_b) = f.radial_band();
    // As duas bandas apoiam-se na baseline (o raio do guia): uma cresce para dentro, a outra
    // para fora. A tolerância cobre a descida das letras (o `P` não desce) e a corda do arco.
    assert!(
        (hi_a - R).abs() < 0.05 && hi_a >= lo_a,
        "sem flip a banda encosta no raio por CIMA e vive por dentro: ({lo_a:.3}, {hi_a:.3})"
    );
    assert!(
        (lo_b - R).abs() < 0.05 && hi_b > R,
        "com flip ela encosta por BAIXO e vive por fora: ({lo_b:.3}, {hi_b:.3})"
    );
    assert!(
        lo_a < R - 0.2 && hi_b > R + 0.2,
        "e cada uma tem corpo do seu lado: {lo_a:.3} / {hi_b:.3}"
    );
}
