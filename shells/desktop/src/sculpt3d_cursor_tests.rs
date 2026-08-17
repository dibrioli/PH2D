//! **O ANEL DO PINCEL** — os gates da figura que o cursor desenha.
//!
//! Módulo irmão de teste do [`super`] (`#[path]`, `cfg(test)`). ⚠️ **Nada aqui
//! pede GPU**, e é por isso que a geometria é a função LIVRE
//! [`super::ring_on_surface`]: uma [`super::Sculpt3dScene`] exige um
//! `wgpu::Device`, e o que esta conta precisa é uma câmera, um viewport, um
//! ponto, uma normal e um raio.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins sculpt3d::cursor::tests
//! ```

use super::*;

const VIEWPORT: (u32, u32) = (1920, 1080);
const R_PX: f32 = 80.0;

/// A câmera enquadrando a esfera unitária, como a cena a enquadra.
fn camera() -> Camera3d {
    let mut cam = Camera3d {
        yaw: 0.0,
        pitch: 0.0,
        ..Camera3d::default()
    };
    cam.frame(ph2d_mesh::shapes::uv_sphere(16, 24, 1.0).bounds(), {
        let (w, h) = VIEWPORT;
        w as f32 / h as f32
    });
    cam
}

/// A normal que olha para a câmera, girada de `deg` em torno do eixo Y.
fn tilted(cam: &Camera3d, deg: f32) -> [f32; 3] {
    let e = cam.eye();
    let facing = super::unit([e.x, e.y, e.z]).expect("olho na origem");
    let (s, c) = deg.to_radians().sin_cos();
    // Rotação em torno de Y: leva a normal para longe de encarar a câmera.
    [
        facing[0] * c + facing[2] * s,
        facing[1],
        -facing[0] * s + facing[2] * c,
    ]
}

/// O menor e o maior raio, em pixels, do anel desenhado — medidos contra o
/// CENTRO projetado, que é onde o círculo de tela seria desenhado.
fn extent(path: &ph2d_vector::BezPath, cam: &Camera3d, at: [f32; 3]) -> (f32, f32) {
    let (cx, cy) = cam.project(at, VIEWPORT).expect("centro atrás do olho");
    let mut lo = f32::INFINITY;
    let mut hi: f32 = 0.0;
    for el in path.elements() {
        let p = match el {
            ph2d_vector::PathEl::MoveTo(p) | ph2d_vector::PathEl::LineTo(p) => *p,
            _ => continue,
        };
        let d = ((p.x as f32 - cx).powi(2) + (p.y as f32 - cy).powi(2)).sqrt();
        lo = lo.min(d);
        hi = hi.max(d);
    }
    (lo, hi)
}

/// **QUANTO O CÍRCULO DE TELA SUPERESTIMA** — a medição que decidiu a wave.
///
/// ```text
/// cargo test -p ph2d-host-desktop --bins \
///   sculpt3d::cursor::tests::measure_what_the_screen_ring_overstates -- --ignored --nocapture
/// ```
///
/// A pegada é uma BOLA de mundo; o círculo de tela é a silhueta dela. Quem
/// recebe tinta é a interseção da bola com a SUPERFÍCIE, e numa superfície
/// inclinada de `θ` essa interseção projeta uma elipse de eixo menor `r·cos θ`.
/// A tabela é essa previsão contra a porta do produto.
#[test]
#[ignore = "sonda: imprime, não afirma"]
fn measure_what_the_screen_ring_overstates() {
    let cam = camera();
    let at = [0.0, 0.0, 0.0];
    println!("\n  tilt |  menor  |  maior  | menor/rpx | cos(tilt)");
    println!("  -----+---------+---------+-----------+----------");
    for deg in [0.0f32, 15.0, 30.0, 45.0, 60.0, 75.0, 85.0] {
        let n = tilted(&cam, deg);
        let path = super::ring_on_surface(&cam, VIEWPORT, at, n, R_PX).expect("anel");
        let (lo, hi) = extent(&path, &cam, at);
        println!(
            "  {deg:4.0}° | {lo:7.2} | {hi:7.2} |   {:5.3}   |   {:5.3}",
            lo / R_PX,
            deg.to_radians().cos()
        );
    }
}

/// **O CONTROLE: de frente, o anel deitado É o círculo de tela.**
///
/// ⚠️ É esta metade que torna a wave segura, e ela não é decorativa: a pose mais
/// comum de todas é a superfície encarando a câmera, e ali as duas figuras
/// coincidem **ao centésimo de pixel** (80,00 contra 80,00 medido). Sem ela,
/// *"o anel mudou"* e *"o anel quebrou"* seriam indistinguíveis em toda cena.
#[test]
fn the_ring_reduces_to_the_screen_circle_when_the_surface_faces_the_camera() {
    let cam = camera();
    let at = [0.0, 0.0, 0.0];
    let path = super::ring_on_surface(&cam, VIEWPORT, at, tilted(&cam, 0.0), R_PX).expect("anel");
    let (lo, hi) = extent(&path, &cam, at);
    assert!(
        (lo - R_PX).abs() < 0.5 && (hi - R_PX).abs() < 0.5,
        "de frente o anel deitado tem de SER o círculo de tela, e mediu \
         [{lo:.2}, {hi:.2}] contra um raio de {R_PX:.2} px"
    );
}

/// **A ENTREGA: numa superfície de perfil o anel ENCURTA, e o círculo não.**
///
/// A pegada é uma bola de mundo; o círculo de tela é a silhueta dela. Quem
/// recebe tinta é a interseção com a SUPERFÍCIE, que projeta uma elipse de eixo
/// menor `r·cos θ` — medido, a 60° o círculo superestimava **2,1×**.
///
/// ⚠️ **A barra é uma RAZÃO entre os dois eixos do MESMO anel**, nunca um
/// número absoluto de pixels: um limiar em px seria calibrado contra esta
/// câmera e reprovaria a wave no dia em que alguém mudasse o enquadramento.
#[test]
fn the_ring_foreshortens_on_a_tilted_surface() {
    let cam = camera();
    let at = [0.0, 0.0, 0.0];
    for (deg, want) in [(30.0f32, 0.866f32), (60.0, 0.5)] {
        let path =
            super::ring_on_surface(&cam, VIEWPORT, at, tilted(&cam, deg), R_PX).expect("anel");
        let (lo, hi) = extent(&path, &cam, at);
        assert!(
            (hi - R_PX).abs() < 1.0,
            "o eixo MAIOR não encurta (é a direção que a inclinação não toca): \
             {hi:.2} contra {R_PX:.2} px a {deg:.0}°"
        );
        let got = lo / R_PX;
        assert!(
            (got - want).abs() < 0.05,
            "a {deg:.0}° o eixo menor tinha de cair para ~cos({deg:.0}°) = {want:.3} do \
             raio, e mediu {got:.3} — o anel não deitou na superfície"
        );
    }
}

/// **UMA NORMAL QUE NÃO NOMEIA DIREÇÃO NENHUMA RECUA PARA A SILHUETA.**
///
/// ⚠️ Isto é o que mantém o cursor fora da lacuna nomeada do
/// [`ph2d_mesh::Hit::normal`]: o recuo é a resposta honesta a *"não sei a
/// orientação"*, e desenhar um anel a partir de um vetor nulo desenharia uma
/// figura inventada.
#[test]
fn a_normal_that_names_no_direction_falls_back_to_the_screen_circle() {
    let cam = camera();
    for n in [
        [0.0, 0.0, 0.0],
        [f32::NAN, 0.0, 1.0],
        [0.0, f32::INFINITY, 0.0],
    ] {
        assert!(
            super::ring_on_surface(&cam, VIEWPORT, [0.0, 0.0, 0.0], n, R_PX).is_none(),
            "uma normal degenerada ({n:?}) tem de recusar o anel, não desenhar um"
        );
    }
}

/// **OU O ANEL INTEIRO OU NENHUM** — a amostra que não projeta leva o anel toda.
///
/// ⚠️ **O caso é ALCANÇÁVEL e a fixture o mede em vez de o supor:** com a
/// superfície de perfil (89°) o anel deita num plano que CONTÉM o eixo da vista,
/// então metade dele caminha na direção do olho; com raio de mundo `6,02`
/// contra uma distância de olho de ~3,3 ele passa por trás. De frente isso
/// **nunca** acontece — o anel fica na mesma profundidade —, e é por isso que a
/// linha de controle usa um raio absurdo (100 000 px) e mesmo assim projeta.
#[test]
fn the_ring_is_whole_or_absent() {
    let cam = camera();
    let at = [0.0, 0.0, 0.0];
    assert!(
        super::ring_on_surface(&cam, VIEWPORT, at, tilted(&cam, 0.0), 100_000.0).is_some(),
        "CONTROLE: de frente o anel nunca sai da profundidade dele, por maior que seja"
    );
    let path = super::ring_on_surface(&cam, VIEWPORT, at, tilted(&cam, 89.0), 2000.0);
    assert!(
        path.is_none(),
        "de perfil e enorme, metade do anel cai atrás do olho — ele tem de recusar \
         inteiro, nunca desenhar o pedaço que sobrou"
    );
    let whole = super::ring_on_surface(&cam, VIEWPORT, at, tilted(&cam, 60.0), R_PX).expect("anel");
    let pts = whole
        .elements()
        .iter()
        .filter(|e| {
            matches!(
                e,
                ph2d_vector::PathEl::MoveTo(_) | ph2d_vector::PathEl::LineTo(_)
            )
        })
        .count();
    assert_eq!(
        pts,
        super::RING_SEGS + 1,
        "um anel aceito tem de trazer TODAS as amostras"
    );
}
