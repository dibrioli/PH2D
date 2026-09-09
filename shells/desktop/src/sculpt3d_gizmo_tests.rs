//! **O GIZMO DE TRANSFORMAÇÃO DA ESCULTURA, medido.**
//!
//! ⚠️ Precisam de um device (uma `Sculpt3dScene` não existe sem ele), mas **não**
//! de janela.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins sculpt3d::gizmo
//! ```

use ph2d_editor::zones::Rect;
use ph2d_mesh::shapes::uv_sphere;
use ph2d_sculpt3d::TransformKind;

const W: f32 = 900.0;
const H: f32 = 700.0;

fn cena(kind: TransformKind) -> Option<crate::sculpt3d::Sculpt3dScene> {
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()?;
    let mut s = crate::sculpt3d::Sculpt3dScene::new(&gpu.device, uv_sphere(20, 30, 1.0), 1.0);
    s.note_canvas(Rect::new(0.0, 0.0, W, H));
    assert!(
        s.arm_transform(kind),
        "a fixture nao conseguiu armar o transform"
    );
    Some(s)
}

macro_rules! cena_ou_sai {
    ($k:expr) => {
        match cena($k) {
            Some(s) => s,
            None => {
                eprintln!("no GPU adapter on this machine — nothing to assert");
                return;
            }
        }
    };
}

/// O centroide da malha — a régua de *para onde a peça foi*.
fn centroide(s: &crate::sculpt3d::Sculpt3dScene) -> [f32; 3] {
    let p = s.mesh().positions();
    let n = p.len().max(1) as f32;
    let mut c = [0.0f32; 3];
    for q in p {
        for k in 0..3 {
            c[k] += q[k] / n;
        }
    }
    c
}

/// Roda o gesto inteiro a partir de um pixel, e devolve o deslocamento do
/// centroide.
fn arrasta(s: &mut crate::sculpt3d::Sculpt3dScene, from: (f32, f32), to: (f32, f32)) -> [f32; 3] {
    let antes = centroide(s);
    // ⚠️ O pen-down do produto pergunta SEMPRE ao gizmo antes de começar a
    // sessão — ver o censo no fim deste ficheiro. Um `false` aqui é o caso
    // livre, não uma recusa.
    s.gizmo_grab(from.0, from.1);
    assert!(s.begin_transform(from.0, from.1), "o begin recusou");
    s.transform_at(to.0, to.1);
    let depois = centroide(s);
    s.close_transform();
    [
        depois[0] - antes[0],
        depois[1] - antes[1],
        depois[2] - antes[2],
    ]
}

/// ⭐⭐⭐ **AS ALÇAS EXISTEM, E SÓ COM O TRANSFORM ARMADO.**
///
/// ⚠️ **As duas metades são o gate.** Só a primeira deixaria passar um gizmo
/// permanente — alças desenhadas sobre a peça enquanto o artista esculpe, a
/// roubar-lhe o clique. Só a segunda deixaria passar um gizmo que nunca aparece.
#[test]
fn as_alcas_existem_e_so_com_o_transform_armado() {
    let mut s = cena_ou_sai!(TransformKind::Move);
    let n_movidas = s.gizmo_handles().len();
    println!("alcas no modo Move: {n_movidas}");
    assert!(
        n_movidas >= 7,
        "o modo Move devia dar o disco de vista + 3 planos + 3 setas, e deu {n_movidas}"
    );
    // Desarmar (clicar o que já está armado desliga) apaga o gizmo.
    assert!(!s.arm_transform(TransformKind::Move));
    assert!(
        s.gizmo_handles().is_empty(),
        "as alcas sobreviveram ao transform DESARMADO -- elas ficariam por cima da peca a roubar \
         o clique de quem esta' a esculpir"
    );
}

/// ⭐⭐⭐ **A SETA DE UM EIXO MOVE SÓ NAQUELE EIXO.**
///
/// ⛔⛔ **É o gesto que NÃO EXISTIA.** O transform modal move no plano da tela; o
/// kernel sempre soube aceitar qualquer vector em `Gesture::Move { delta }`, e o
/// que faltava era como pedir *«só em X»*.
///
/// ⚠️ **A régua é o deslocamento do CENTROIDE**, e o controlo é o mesmo arrasto
/// **sem** agarrar a alça: sem ele o gate não distingue *«prendeu ao eixo»* de
/// *«a câmera calhou de mover quase só em X»*.
#[test]
fn a_seta_de_um_eixo_move_so_naquele_eixo() {
    let mut s = cena_ou_sai!(TransformKind::Move);
    // A ponta da seta de X, onde a própria lei a pôs.
    let handles = s.gizmo_handles();
    let seta = handles
        .iter()
        .find(|h| h.handle == crate::field3d_gizmo::Handle::Axis(0) && h.live)
        .expect("a seta de X tem de estar viva neste enquadramento");
    let crate::field3d_gizmo::Shape::Arrow { from, to } = seta.shape else {
        panic!("a seta de um eixo devia ser uma Arrow");
    };
    // Agarra a meio da haste e arrasta na DIAGONAL — as duas componentes.
    let meio = (f32::midpoint(from[0], to[0]), f32::midpoint(from[1], to[1]));
    let destino = (meio.0 + 120.0, meio.1 + 90.0);
    let preso = arrasta(&mut s, meio, destino);

    // ⚠️ **O CONTROLO**: o MESMO arrasto, longe de qualquer alça.
    let mut t = cena_ou_sai!(TransformKind::Move);
    let livre = {
        let antes = centroide(&t);
        assert!(t.begin_transform(destino.0, destino.1));
        t.transform_at(destino.0 + 120.0, destino.1 + 90.0);
        let d = centroide(&t);
        t.close_transform();
        [d[0] - antes[0], d[1] - antes[1], d[2] - antes[2]]
    };
    println!("preso ao eixo X: {preso:?} | livre: {livre:?}");
    let fora = preso[1].hypot(preso[2]);
    assert!(
        preso[0].abs() > 1e-4,
        "a peca nao andou em X ({preso:?}) -- a alca nao chegou a prender gesto nenhum"
    );
    assert!(
        fora < preso[0].abs() * 1e-3,
        "presa a` seta de X a peca andou {fora} fora do eixo (deslocamento {preso:?})"
    );
    assert!(
        livre[1].hypot(livre[2]) > 1e-4,
        "o CONTROLO nao produziu movimento fora de X ({livre:?}) -- sem isso este gate nao \
         distingue «prendeu ao eixo» de «a camera calhou de mover so' em X»"
    );
}

/// ⭐⭐ **O QUADRADO DE UM PLANO MOVE DENTRO DELE, E NADA NA NORMAL.**
///
/// ⚠️ Complementar da seta, e as duas juntas são o par: uma projecta SOBRE o
/// eixo, a outra REMOVE o eixo. Trocá-las compila, e o sintoma é o quadrado do
/// plano `XY` a mover a peça só em `Z`.
#[test]
fn o_quadrado_de_um_plano_move_dentro_dele() {
    let mut s = cena_ou_sai!(TransformKind::Move);
    let handles = s.gizmo_handles();
    // `Plane(2)` é o quadrado do plano XY — a normal dele é Z.
    let quad = handles
        .iter()
        .find(|h| h.handle == crate::field3d_gizmo::Handle::Plane(2) && h.live)
        .expect("o quadrado do plano XY tem de estar vivo neste enquadramento");
    let crate::field3d_gizmo::Shape::Quad(c) = quad.shape else {
        panic!("um plano devia ser um Quad");
    };
    let centro = (
        (c[0][0] + c[1][0] + c[2][0] + c[3][0]) / 4.0,
        (c[0][1] + c[1][1] + c[2][1] + c[3][1]) / 4.0,
    );
    let d = arrasta(&mut s, centro, (centro.0 + 110.0, centro.1 + 80.0));
    println!("preso ao plano XY: {d:?}");
    assert!(
        d[0].hypot(d[1]) > 1e-4,
        "a peca nao andou no plano XY ({d:?})"
    );
    assert!(
        d[2].abs() < d[0].hypot(d[1]) * 1e-3,
        "presa ao plano XY a peca andou {} na NORMAL dele (deslocamento {d:?}) -- as duas \
         projeccoes estao trocadas",
        d[2].abs()
    );
}

/// ⭐⭐⭐ **A ARGOLA DE UM EIXO RODA EM TORNO DELE — e no sentido que se VÊ.**
///
/// ⚠️⚠️ **A segunda metade é a que custa.** A varredura que dá o ângulo é medida
/// no ECRÃ, então uma argola cujo eixo aponta para TRÁS rodaria ao contrário do
/// arrasto se o sinal fosse o cru — o artista lê isso como *«a argola de trás
/// roda ao contrário»*.
///
/// # ⛔ A régua é o SINAL, e a primeira redacção comparava MAGNITUDES
///
/// Ela exigia que a argola de `Z` e a de VISTA dessem o mesmo deslocamento com a
/// câmera de frente — e elas dão `[0,893, −0,686]` contra `[0,831, −0,536]`,
/// porque as duas argolas têm **raios diferentes** (a de vista fica por fora),
/// logo o mesmo arrasto em pixels varre ângulos diferentes. *A fixture media
/// duas coisas que nunca foram iguais, e o que a lei afirma é o SENTIDO.*
///
/// ⇒ mede-se o ângulo **com sinal** que um vértice-sonda varre em torno do pivô,
/// no plano da rotação. E o discriminador do «segue o observador» é a **troca de
/// lado**: o MESMO arrasto, visto de FRENTE e visto de TRÁS, tem de rodar a peça
/// em sentidos OPOSTOS no mundo — que é o que faz os dois parecerem iguais na
/// tela.
#[test]
fn a_argola_de_um_eixo_roda_no_sentido_que_se_ve() {
    /// O ângulo com sinal que a sonda varreu no plano `XY` — o da rotação em `Z`.
    fn varrido(antes: [f32; 3], depois: [f32; 3], pivo: [f32; 3]) -> f32 {
        let a = [antes[0] - pivo[0], antes[1] - pivo[1]];
        let b = [depois[0] - pivo[0], depois[1] - pivo[1]];
        (a[0] * b[1] - a[1] * b[0]).atan2(a[0] * b[0] + a[1] * b[1])
    }
    /// Arrasta na argola `handle` a partir da vista `vista`, e devolve o ângulo
    /// varrido em `XY` mais o desvio em `Z` do vértice-sonda.
    fn gira(
        handle: crate::field3d_gizmo::Handle,
        vista: crate::field3d_views::Standard,
    ) -> Option<(f32, f32)> {
        let mut s = cena(TransformKind::Rotate)?;
        s.aim_view(vista);
        let alcas = s.gizmo_handles();
        let alvo = alcas.iter().find(|h| h.handle == handle && h.live)?;
        let crate::field3d_gizmo::Shape::Arc(pts) = &alvo.shape else {
            panic!("uma argola devia ser um Arc");
        };
        let p = *pts.first()?;
        let pivo = ph2d_sculpt3d::free_pivot(s.mesh())?;
        let antes = s.mesh().positions()[7];
        assert!(s.gizmo_grab(p[0], p[1]), "a argola nao foi agarrada");
        assert!(s.begin_transform(p[0], p[1]));
        s.transform_at(p[0] + 80.0, p[1] + 60.0);
        let depois = s.mesh().positions()[7];
        s.close_transform();
        Some((varrido(antes, depois, pivo), depois[2] - antes[2]))
    }
    use crate::field3d_gizmo::Handle;
    use crate::field3d_views::Standard;
    let (Some((anel, dz)), Some((vista, _)), Some((de_tras, _))) = (
        gira(Handle::Ring(2), Standard::Front),
        gira(Handle::ViewRing, Standard::Front),
        gira(Handle::Ring(2), Standard::Back),
    ) else {
        eprintln!("no GPU adapter on this machine — nothing to assert");
        return;
    };
    println!(
        "argola de Z (frente) {anel:+.4} rad, desvio em Z {dz:+.6} | de VISTA {vista:+.4} | \
         argola de Z (de tras) {de_tras:+.4}"
    );
    assert!(
        anel.abs() > 1e-3,
        "a argola de Z nao rodou nada ({anel:+.4} rad)"
    );
    // (1) — **o EIXO é o que a argola diz**: uma rotação em `Z` não move `z`.
    assert!(
        dz.abs() < 1e-4,
        "rodar em torno de Z moveu a sonda {dz:+.6} em Z -- o eixo nao e' o que a argola diz"
    );
    // (2) — **o SENTIDO é o da vista**: de frente, a argola de `Z` e a de vista
    // descrevem o mesmo eixo, logo o mesmo arrasto tem de rodar para o mesmo
    // lado. (As magnitudes diferem: os raios das duas argolas são diferentes.)
    assert_eq!(
        anel.is_sign_positive(),
        vista.is_sign_positive(),
        "de FRENTE a argola de Z ({anel:+.4}) e a de VISTA ({vista:+.4}) rodaram para lados \
         OPOSTOS -- elas descrevem o mesmo eixo"
    );
    // (3) — **o DISCRIMINADOR**: visto de trás, o mesmo arrasto roda ao
    // contrário no mundo. É isso que o faz parecer igual na tela.
    assert_ne!(
        anel.is_sign_positive(),
        de_tras.is_sign_positive(),
        "o mesmo arrasto rodou para o MESMO lado do mundo visto de frente ({anel:+.4}) e de tras \
         ({de_tras:+.4}) -- o eixo da argola nao esta' a seguir o observador, e na tela ela roda \
         ao contrario de um dos dois lados"
    );
}

/// ⭐⭐ **SEM ALÇA AGARRADA O TRANSFORM CORRE LIVRE** — o gesto modal fica.
///
/// ⛔ *Um gizmo que tomasse conta do botão inteiro tiraria uma ferramenta que
/// funciona para dar outra.* Este gate é a cerca disso.
#[test]
fn sem_alca_agarrada_o_transform_corre_livre() {
    let mut s = cena_ou_sai!(TransformKind::Move);
    // Um canto do écran, longe de qualquer alça (elas vivem à volta do pivô,
    // que está no meio).
    assert!(
        s.gizmo_pick(8.0, 8.0).is_none(),
        "a fixture escolheu um pixel que TEM alca -- ela nao mede o caso livre"
    );
    let d = arrasta(&mut s, (8.0, 8.0), (128.0, 98.0));
    println!("livre: {d:?}");
    let fora_de_um_eixo = [d[0].abs() > 1e-5, d[1].abs() > 1e-5, d[2].abs() > 1e-5]
        .iter()
        .filter(|b| **b)
        .count();
    assert!(
        fora_de_um_eixo >= 2,
        "sem alca o movimento devia ser o do PLANO DA TELA (duas componentes ou mais) e deu \
         {d:?} -- o gizmo prendeu um gesto que ninguem agarrou"
    );
}

/// ⭐⭐ **TODO PEN-DOWN DE TRANSFORM PERGUNTA AO GIZMO PRIMEIRO.**
///
/// ⛔ **Sem isto a alça agarrada pode ficar VELHA.** O `gizmo_grip` é largado no
/// `close_transform`, que **não corre** quando o `begin_transform` recusa (a
/// peça toda protegida): o que impede uma alça de um gesto morto de prender o
/// gesto seguinte é o pen-down do produto reescrever o grip **sempre**, e não
/// só quando ele acerta numa alça.
///
/// ⚠️ **Censo de FONTE, e a razão é a de sempre**: o `App::sculpt3d_pointer_down`
/// precisa de uma surface de janela real, então esta costura não é alcançável
/// de um teste. O que se pode afirmar é a ORDEM — e ela é a lei: agarrar depois
/// de a sessão congelar a foto perguntaria a uma peça e responderia sobre outra.
#[test]
fn todo_pen_down_de_transform_pergunta_ao_gizmo_primeiro() {
    let fonte = include_str!("sculpt3d_input_down.rs");
    let grab = fonte
        .find("gizmo_grab(")
        .expect("o pen-down tem de perguntar ao gizmo -- sem isso ele esta' MORTO sob o ponteiro");
    let begin = fonte.find("begin_transform(").expect(
        "controlo positivo: o pen-down do transform mudou de ficheiro e este censo \
                 varreria o vazio",
    );
    assert!(
        grab < begin,
        "o `gizmo_grab` (byte {grab}) vem DEPOIS do `begin_transform` (byte {begin}) -- a \
         projeccao das alcas sai do pivo da malha ATUAL, e agarrar depois de a sessao congelar a \
         foto pergunta a uma peca e responde sobre outra"
    );
}

/// ⭐⭐⭐ **COM O TRANSFORM ARMADO, ARRASTAR NO VAZIO AINDA GIRA A CÂMERA.**
///
/// ⛔⛔ **REPORT DO ENIO, 2026-09-08:** *«com as ferramentas de transformação
/// ativadas anulo a rot do canvas. isso não pode acontecer.»*
///
/// O defeito é **pré-existente** — a wave do gizmo só o tornou visível: o braço
/// do transform tomava o botão esquerdo **sem perguntar se o raio acertou alguma
/// coisa**, então armar a ferramenta apagava o gesto mais comum do mundo,
/// *arrastar no vazio para girar a peça*.
///
/// ⚠️⚠️ **O `aim` já declarava a lei no próprio doc** — *«`false` se o raio não
/// achou nada (e aí o botão vira órbita, como em todo gesto)»* — e era «todo
/// gesto» menos este.
///
/// ⚠️ **TRÊS metades, e nenhuma sozinha é o gate.** Sem a (2) bastaria desarmar
/// o transform para passar; sem a (3) bastaria devolver o vazio a toda a gente,
/// e a ferramenta deixaria de funcionar.
#[test]
fn com_o_transform_armado_arrastar_no_vazio_ainda_gira_a_camera() {
    let s = cena_ou_sai!(TransformKind::Move);
    // Um pixel do canvas onde não há peça nem alça: a quina.
    let vazio = (6.0, 6.0);
    assert!(
        s.pick_active(vazio.0, vazio.1).is_none() && s.gizmo_pick(vazio.0, vazio.1).is_none(),
        "a fixture escolheu um pixel que TEM peca ou alca -- ela nao mede o vazio"
    );

    // ⚠️ **A decisão do pen-down NÃO é alcançável daqui** (ela vive no `App`,
    // que precisa de uma surface real) — quem a afirma é o censo irmão abaixo.
    // O que esta metade mede é a outra: que a ferramenta continua a funcionar
    // sobre a peça depois da cura. *Sem ela, devolver o vazio a toda a gente
    // passaria — e a ferramenta deixaria de existir.*

    // (2) — SOBRE A PEÇA a ferramenta continua a funcionar.
    let mut t = cena_ou_sai!(TransformKind::Move);
    let (cx, cy) = (450.0, 350.0);
    assert!(
        t.pick_active(cx, cy).is_some(),
        "a fixture nao acerta a peca no centro -- ela nao mede o caso vivo"
    );
    let sobre = arrasta(&mut t, (cx, cy), (cx + 120.0, cy));
    println!("sobre a peca: {sobre:?}");
    assert!(
        sobre[0].abs() + sobre[1].abs() + sobre[2].abs() > 1e-4,
        "com o transform armado, arrastar SOBRE a peca deixou de a mover ({sobre:?}) -- a cura do \
         vazio levou a ferramenta com ela"
    );
}

/// ⭐⭐⭐ **E O PEN-DOWN DECIDE ASSIM** — o censo da ordem, que é o que uma cena
/// não alcança.
///
/// ⚠️ **De FONTE porque a decisão vive no `App`**, que precisa de uma surface de
/// janela real. O que se afirma é que o braço do transform **pergunta as duas
/// coisas** (a alça e o barro) e **desvia para a órbita** quando as duas falham
/// — e que a alça vem primeiro, senão a ponta de uma seta espetada no vazio
/// seria inalcançável.
#[test]
fn o_pen_down_do_transform_desvia_para_a_orbita_quando_erra_tudo() {
    let fonte = include_str!("sculpt3d_input_down.rs");
    let i = fonte.find("if scene.transform_arm().is_some() {").expect(
        "controlo positivo: o braco do transform mudou de forma e este censo varreria o vazio",
    );
    let braco = &fonte[i..];
    let fim = braco.find("scene.brush.invert").unwrap_or(braco.len());
    let braco = &braco[..fim];
    let alca = braco
        .find("gizmo_grab(")
        .expect("o braco tem de perguntar pela ALCA");
    let barro = braco.find("scene.aim(").expect("o braco tem de MIRAR");
    assert!(
        alca < barro,
        "o `gizmo_grab` (byte {alca}) vem depois do `aim` (byte {barro}) -- a ponta de uma seta \
         espeta-se no VAZIO, e perguntar pelo barro primeiro tornaria essa alca inalcancavel"
    );
    assert!(
        braco.contains("if !na_alca && !no_barro {") && braco.contains("Drag::Orbit"),
        "o braco do transform nao desvia para a ORBITA quando erra a alca E o barro -- armar a \
         ferramenta volta a apagar o gesto de girar a peca"
    );
}
