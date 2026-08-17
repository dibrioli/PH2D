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
//! ## O anel deita na SUPERFÍCIE, e a cerca anterior media a coisa errada
//!
//! O raio do pincel **é** medido em pixels de tela (`docs/3D/04.1` — é o
//! `computeWorldRadius2` do SculptGL), e daí esta cerca concluía que *"um
//! círculo de raio `radius_px` no plano da tela é a figura dele"*, chamando o
//! cursor conformado do Blender de resposta a um problema que não temos.
//!
//! ⚠️ **A premissa estava certa e a conclusão não, e a distinção é geométrica:**
//! a pegada é uma BOLA de mundo (`PAINT_FALLOFF_SHAPE_SPHERE`) cujo raio é
//! derivado dos pixels na profundidade do acerto. O círculo de tela é a
//! **SILHUETA dessa bola** — mas quem recebe tinta é a interseção dela com a
//! SUPERFÍCIE, e numa superfície inclinada de `θ` essa interseção projeta uma
//! ELIPSE de eixo menor `r·cos θ`, **inscrita** na silhueta. O anel de tela
//! desenhava a silhueta e o barro se movia dentro de uma elipse: o círculo
//! **superestima**, e quanto mais de perfil a superfície, mais.
//!
//! Ordem do Enio (2026-08-17): *"o gizmo da tool deve ter a direção das normais
//! onde incide (a nossa o gizmo da tool permanece na direção da tela)"*.
//!
//! ⚠️ **A NORMAL vem da mesma família que o KERNEL lê** — as normais suaves de
//! `Mesh::normals()`, que é o `base_nrm` do dab —, e **não** do
//! [`ph2d_mesh::Hit::normal`]. Não é gosto: aquele campo carrega uma lacuna
//! NOMEADA (um quad "gravata" devolve `[0,0,0]`) cujo gatilho declarado é *"o
//! primeiro leitor de produto"*, e a cura dele mexe no laço quente do raycast.
//! Ler a normal suave evita ser esse leitor **e** faz o cursor concordar com o
//! kernel por construção, em vez de por coincidência.
//!
//! ⚠️ E ele é traçado sob `Affine::IDENTITY`: no Vello o transform do `stroke`
//! **multiplica a largura**, e é isso que transformou o realce do Flip num
//! borrão (smoke, 2026-07-13).

use ph2d_mesh::Hit;
use ph2d_mesh_render::Camera3d;
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
        let landed = self.pick(x, y).and_then(|(i, hit)| {
            let pose = self.objects.get(i)?.pose;
            let at = pose.point_to_world(hit.point);
            let (cx, cy) = self.camera.project(at, self.viewport)?;
            Some((i, hit, at, cx, cy))
        });
        // ⚠️ **O anel DEITA na superfície quando ela tem orientação conhecida**,
        // e cai no círculo de tela — a silhueta — quando não tem. As duas
        // figuras COINCIDEM de frente (medido: 80,00 contra 80,00 px), então o
        // recuo não pisca; o que ele perde é justamente a informação que só
        // existe quando a superfície está de perfil.
        let path = landed.as_ref().and_then(|&(i, ref hit, at, _, _)| {
            let n = self.surface_normal(i, hit)?;
            ring_on_surface(&self.camera, self.viewport, at, n, self.radius_px())
        });
        let (cx, cy) = landed
            .as_ref()
            .map_or((x, y), |&(_, _, _, cx, cy)| (cx, cy));
        Some(CursorMark {
            path: path.unwrap_or_else(|| ring(f64::from(cx), f64::from(cy), r)),
            on_surface: landed.is_some(),
        })
    }

    /// **A NORMAL DE MUNDO DA SUPERFÍCIE NO ACERTO** — as normais SUAVES da
    /// face acertada, que são a régua do KERNEL (`base_nrm`).
    ///
    /// ⚠️ **Não é o [`Hit::normal`]**, e não é preferência: aquele campo é a
    /// normal GEOMÉTRICA da face e carrega uma lacuna nomeada (um quad
    /// "gravata" devolve `[0,0,0]`) cujo gatilho declarado é *"o primeiro
    /// leitor de produto"* — e a cura dele mexe no laço quente do raycast.
    /// Ler a suave evita ser esse leitor **e** faz o cursor concordar com o dab
    /// por construção: os dois passam a ler o mesmo array.
    ///
    /// ⚠️ **A média não é normalizada aqui** — quem normaliza é o
    /// [`ring_on_surface`], que também é quem sabe recusar. Normalizar duas
    /// vezes seria uma segunda resposta a *"esta direção existe?"*.
    fn surface_normal(&self, i: usize, hit: &Hit) -> Option<[f32; 3]> {
        let mesh = self.objects.get(i)?.stack.mesh();
        let face = mesh.faces().get(hit.face as usize)?;
        let nrm = mesh.normals();
        let mut acc = [0.0f32; 3];
        for &v in face.verts() {
            let n = nrm.get(v as usize)?;
            acc[0] += n[0];
            acc[1] += n[1];
            acc[2] += n[2];
        }
        Some(self.objects.get(i)?.pose.vector_to_world(acc))
    }
}

/// **O ANEL DEITADO NA SUPERFÍCIE** — o círculo de raio de MUNDO no plano
/// tangente ao acerto, projetado.
///
/// Função LIVRE pelo motivo do [`super::sculpt3d_space::stencil_of`]: uma
/// [`Sculpt3dScene`] exige um `wgpu::Device`, e o que esta conta precisa é uma
/// câmera, um viewport, um ponto, uma normal e um raio.
///
/// ⚠️ **`None` significa *"não sei desenhar isto"*, e o chamador cai no círculo
/// de tela** — que é a silhueta, a resposta honesta quando a orientação é
/// desconhecida. São dois casos e nenhum é hipotético: uma normal degenerada
/// (não-finita ou de comprimento ~zero) e uma amostra que **não projeta**
/// (atrás do olho), alcançável quando o pincel cobre uma fração enorme da vista.
///
/// ⚠️ **Ou o anel INTEIRO ou nenhum.** Um anel parcial — o que sai de pular a
/// amostra que falha — desenha uma figura que a pegada não tem, e o artista lê
/// a falha do projetor como forma do pincel.
pub(crate) fn ring_on_surface(
    cam: &Camera3d,
    viewport: (u32, u32),
    at: [f32; 3],
    normal: [f32; 3],
    radius_px: f32,
) -> Option<BezPath> {
    let n = unit(normal)?;
    let r = cam.world_radius_for_screen_px(at, radius_px, viewport);
    if !r.is_finite() || r <= 0.0 {
        return None;
    }
    let (u, v) = tangent_basis(n);
    let mut path = BezPath::new();
    for i in 0..=RING_SEGS {
        let a = (i as f32) * std::f32::consts::TAU / (RING_SEGS as f32);
        let (sa, ca) = (a.sin(), a.cos());
        let p = [
            at[0] + r * (ca * u[0] + sa * v[0]),
            at[1] + r * (ca * u[1] + sa * v[1]),
            at[2] + r * (ca * u[2] + sa * v[2]),
        ];
        let (sx, sy) = cam.project(p, viewport)?;
        let pt = ph2d_vector::Point::new(f64::from(sx), f64::from(sy));
        if i == 0 {
            path.move_to(pt);
        } else {
            path.line_to(pt);
        }
    }
    path.close_path();
    Some(path)
}

/// Normaliza, ou `None` se o vetor não descreve direção nenhuma.
///
/// ⚠️ O piso é sobre o comprimento ao QUADRADO e é generoso de propósito: a
/// pergunta não é *"quanto erro há?"* e sim *"existe direção aqui?"*, e uma
/// normal somada de faces que se cancelam chega com magnitude de ruído.
fn unit(v: [f32; 3]) -> Option<[f32; 3]> {
    let sq = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if !sq.is_finite() || sq <= 1e-12 {
        return None;
    }
    let inv = sq.sqrt().recip();
    Some([v[0] * inv, v[1] * inv, v[2] * inv])
}

/// Dois eixos perpendiculares a `n` (unitário).
///
/// ⚠️ **O eixo semente é o MENOS alinhado com `n`**, nunca um fixo: com um
/// `[0,0,1]` chapado o produto vetorial COLAPSA quando a superfície olha para a
/// câmera de topo — e essa é a pose mais comum que existe, não um canto.
/// Qual dos dois eixos do plano é `u` não importa (o anel é um círculo), e é
/// por isso que a escolha pode ser barata.
fn tangent_basis(n: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let a = n[0].abs();
    let b = n[1].abs();
    let c = n[2].abs();
    let seed = if a <= b && a <= c {
        [1.0, 0.0, 0.0]
    } else if b <= c {
        [0.0, 1.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    };
    let u = unit(cross(seed, n)).unwrap_or([1.0, 0.0, 0.0]);
    (u, cross(n, u))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
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

#[cfg(test)]
#[path = "sculpt3d_cursor_tests.rs"]
mod tests;
