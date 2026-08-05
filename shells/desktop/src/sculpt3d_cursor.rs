//! **O CURSOR DO PINCEL** — onde a mão está mirando, na tela (ADR-0150, W12).
//!
//! Filho (`#[path]`) de [`super`] pelo motivo dos outros: ele alcança os
//! privados da cena. O corte é o mais estreito da família — *onde o gesto vai
//! pousar*, e nada além disso.
//!
//! ## Por que ele é a peça que faltava
//!
//! Até aqui a única coisa na tela que dizia onde o pincel ia cair era **o barro
//! se mexendo**. Um artista que reporta *"o lugar onde o mouse toca não
//! corresponde ao local na malha"* está descrevendo a única evidência que tem, e
//! eu não tinha nenhuma melhor: as sondas provam que a matemática do pick fecha
//! (desvio de 0,002 sob topologia dinâmica, 0,0005 px no ida-e-volta da câmera),
//! e nenhuma delas alcança a janela viva.
//!
//! O anel **É** o instrumento: ele é desenhado no PONTO DE ACERTO reprojetado —
//! o inverso exato do raio que o dab usa —, então se ele não estiver debaixo do
//! mouse, a fiação está errada e dá para ver. E se estiver, o que sobra é
//! percepção: perto da silhueta a superfície é quase de perfil, e uma deformação
//! correta LÊ como deslocada.
//!
//! ## Pixels de tela, e é exato
//!
//! O raio do pincel **é** medido em pixels de tela (`docs/3D/04.1` — é o
//! `computeWorldRadius2` do SculptGL: aproximar é como se alcança detalhe fino),
//! então um círculo de raio `radius_px` no plano da tela não é uma aproximação
//! do que o pincel faz: é a figura dele. Isto é o oposto do cursor conformado do
//! Blender, que existe porque lá o raio é de mundo.
//!
//! ⚠️ E ele é traçado sob `Affine::IDENTITY`: no Vello o transform do `stroke`
//! **multiplica a largura**, e é isso que transformou o realce do Flip num
//! borrão (smoke, 2026-07-13).

use ph2d_vector::BezPath;

use super::Sculpt3dScene;

/// Quantos segmentos aproximam o anel. 48 porque ele é grande (até 1/8 da altura
/// da tela) e um polígono grosseiro nesse tamanho lê como polígono.
const RING_SEGS: usize = 48;

/// O anel **sobre a superfície** — a mão está no barro.
pub(crate) const ON_SURFACE_RGBA: [f32; 4] = [0.98, 0.83, 0.36, 0.95];
/// O anel **no vazio** — o raio errou a malha, e um clique aqui ORBITA em vez de
/// esculpir. Cor apagada e não ausência: sumir com o cursor no vazio esconde
/// justamente a informação de que o gesto mudou de significado.
pub(crate) const OFF_SURFACE_RGBA: [f32; 4] = [0.98, 0.83, 0.36, 0.30];

/// Onde o cursor está e o que ele vai fazer.
pub(crate) struct CursorMark {
    pub(crate) path: BezPath,
    pub(crate) on_surface: bool,
}

impl Sculpt3dScene {
    /// O anel do pincel para um cursor em `(x, y)` de tela.
    ///
    /// `None` quando o barro não está na tela (a doação em modo LUZ) — ali o
    /// ponteiro nem é da cena, e um cursor de escultura sobre a tinta prometeria
    /// um gesto que o clique não faz.
    pub(crate) fn cursor_mark(&self, x: f32, y: f32) -> Option<CursorMark> {
        if !self.shows_clay() {
            return None;
        }
        let r = f64::from(self.radius_px());
        // ⚠️ **O centro é o ACERTO REPROJETADO, não o pixel do mouse** — e é essa
        // escolha que faz do anel um instrumento em vez de um enfeite. Os dois
        // coincidem quando a fiação está certa (é a definição de `project` ser a
        // inversa de `ray_through`), então qualquer separação visível entre o
        // anel e o cursor É o defeito, desenhado.
        //
        // No vazio não há acerto a reprojetar e o anel fica no pixel cru — que é
        // onde a órbita vai começar, então ele continua descrevendo o gesto.
        let hit = self.pick(x, y).and_then(|(i, hit)| {
            let pose = self.objects.get(i)?.pose;
            self.camera
                .project(pose.point_to_world(hit.point), self.viewport)
        });
        let (cx, cy) = hit.unwrap_or((x, y));
        Some(CursorMark {
            path: ring(f64::from(cx), f64::from(cy), r),
            on_surface: hit.is_some(),
        })
    }
}

/// Um círculo em pixels de tela.
fn ring(cx: f64, cy: f64, r: f64) -> BezPath {
    let mut path = BezPath::new();
    for i in 0..=RING_SEGS {
        // Sem `libm`: este caminho é DESENHO, não o hash de determinismo — o
        // `std` basta e é o que todo o resto do chrome usa.
        let t = (i as f64) * std::f64::consts::TAU / (RING_SEGS as f64);
        let p = (r.mul_add(t.cos(), cx), r.mul_add(t.sin(), cy));
        if i == 0 {
            path.move_to(p);
        } else {
            path.line_to(p);
        }
    }
    path.close_path();
    path
}
