//! **A CÂMERA do traçador** — irmã de [`super`], cortada por RESPONSABILIDADE quando o teto de LOC
//! de 700 disparou.
//!
//! ⚠️ O corte é de **arquivo**, nunca de superfície: `Orbit` continua a ser
//! `ph2d_field_render::Orbit` para todo o mundo (o `pub use` no pai), como o `sculpt3d_rulers` fez
//! com as réguas dele. Um corte que mudasse caminhos seria uma migração disfarçada de arrumação.
//!
//! ⚠️ **A aritmética de quaternion NÃO mora aqui** — ela vive em [`ph2d_field::xform`], junto do
//! tipo cuja rotação ela compõe. Havia uma cópia local dela neste arquivo até 20/08, e uma segunda
//! resposta para *"qual é a rotação resultante?"* é a forma normal de duas metades do mesmo módulo
//! divergirem sem que nada fique vermelho.

use ph2d_field::xform::{dot, quat_axis_angle, quat_mul, quat_normalize, quat_rotate};

/// ⭐ **A LENTE** — o que o olho faz com o que está longe.
///
/// ⚠️ **É uma escolha, não uma troca**: a nota que estava aqui dizia que a perspectiva *"merece a
/// sua própria comparação lado a lado, não uma troca silenciosa"*. As duas ficam, e a tecla que as
/// alterna é a comparação. O default é a convergente, que é o que um modelador espera; a paralela é
/// a vista de CAD, e é ela que deixa medir e alinhar sem que a distância minta sobre o tamanho.
///
/// ⭐ **O `half_extent` continua a querer dizer a MESMA coisa nas duas**: quantas unidades de mundo
/// cabem em meia altura de quadro **no plano do alvo**. É isso que faz a lente ser só uma lente —
/// zoom, enquadramento e o passo da grelha não mudam de lei, e as duas imagens **coincidem
/// exatamente** naquele plano.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Lens {
    /// Raios **paralelos**: o tamanho na tela não depende da distância.
    Ortho,
    /// Raios que **convergem** num olho. O número é a **meia** abertura, medida no lado **menor** do
    /// quadro — o mesmo lado de que o [`Screen::half`] toma metade, para as duas contas falarem da
    /// mesma coisa.
    Perspective { half_fov: f32 },
}

/// A meia-abertura default, **derivada da referência declarada** e não escolhida.
///
/// O Blender abre uma câmera com **50 mm** sobre um sensor de **36 mm**, e a viewport dele usa o
/// mesmo número. A abertura inteira é `2·atan(18/50) = 39,6°`; a metade é `atan(18/50)`.
///
/// ⚠️ Escrito como a **conta**, e não como o resultado: quem quiser outra distância focal muda o
/// numerador e vê logo o que está a mudar. Um `0.3456` solto seria um número sem procedência.
pub const DEFAULT_HALF_FOV: f32 = 0.345_405_2; // atan(18.0 / 50.0)

/// **Abaixo desta fração da distância do olho, um ponto não tem projeção.**
///
/// ⚠️ O recurso é a **aritmética**, não o gosto: a projeção multiplica por `dist / (dist − z)`, e
/// esse fator explode quando o ponto encosta no olho. A `1e-3` ele vale 1000 — um pixel a ~240 mil
/// de distância do centro, que ainda é um número, e já está tão fora do quadro que nada o desenha.
/// Mais perto do que isso a resposta deixa de ser um pixel e passa a ser ruído com sinal.
const NEAR_FRACTION: f32 = 1.0e-3;

/// A câmera, com a orientação num **quaternion**.
///
/// # ⭐ Por que não é `yaw`/`pitch`
///
/// Uma câmera de dois ângulos tem **polos por construção**: a elevação satura em ±90°, e a partir
/// dali arrastar na vertical não faz nada. Com o enquadramento inicial já a 30° de cima, meio
/// centímetro de rato para baixo bate na parede — e o que o artista vê é *"só roda para um lado"*
/// (Enio, 2026-08-19). A câmera da casa (`ph2d_mesh_render::camera`) tem exatamente o mesmo teto,
/// e **prende-o com um `clamp`**.
///
/// Um `clamp` é o remédio para o sintoma. A causa é a **representação**: dois ângulos não conseguem
/// exprimir uma orientação livre, então nenhum número melhor a devolve. Guardando a orientação
/// inteira, a rotação passa a ser *uma* composição de quaternions — sem polo, sem `clamp`, sem caso
/// especial, e sem o eixo vertical do mundo a decidir o que a mão pode fazer.
///
/// ⚠️ O preço é real e está aceite: **o horizonte deixa de ser fixo**. Uma câmera de dois ângulos
/// nunca inclina; esta inclina, porque é isso que *rotação livre* significa. A volta é
/// [`Orbit::from_yaw_pitch`], que é o que a tecla de repor a vista chama.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Orbit {
    /// A orientação, como quaternion `(x, y, z, w)`: leva os eixos **locais** da câmera para o
    /// mundo.
    pub rotation: [f32; 4],
    /// Quantas unidades de mundo cabem em meia altura de tela **no plano do alvo**. Menor = mais
    /// perto. ⚠️ A definição diz *"no plano do alvo"* de propósito: é o que faz [`Lens`] ser só uma
    /// lente (ver o doc dela).
    pub half_extent: f32,
    /// O ponto que fica no centro do quadro.
    pub target: [f32; 3],
    /// O que o olho faz com o que está longe. Ver [`Lens`].
    pub lens: Lens,
}

impl Default for Orbit {
    fn default() -> Self {
        // Três-quartos, ligeiramente por cima: o ângulo em que uma aresta viva e um filete se
        // distinguem sem ambiguidade (escolhido na W0, ao olhar as imagens).
        // ⚠️ Só a ROTAÇÃO vem de lá, e o resto é escrito aqui — ver a nota em `from_yaw_pitch`.
        Self {
            rotation: Self::from_yaw_pitch(0.72, 0.52).rotation,
            half_extent: 0.8,
            target: [0.0; 3],
            lens: Lens::Perspective {
                half_fov: DEFAULT_HALF_FOV,
            },
        }
    }
}

impl Orbit {
    /// A orientação que os dois ângulos de uma câmera de prato giratório dariam.
    ///
    /// Continua a existir por duas razões, e nenhuma é nostalgia: é como se escreve um
    /// **enquadramento nomeado** (o inicial, a vista de frente, a de topo), e é o que **repõe** a
    /// vista depois de a rotação livre a ter inclinado.
    #[must_use]
    pub fn from_yaw_pitch(yaw: f32, pitch: f32) -> Self {
        // `R = Ry(yaw) · Rx(−pitch)` — a composição que reproduz exatamente a base antiga
        // (`fwd = (cos p·sin y, sin p, cos p·cos y)`), verificada por gate.
        let (sy, cy) = (yaw * 0.5).sin_cos();
        let (sp, cp) = (-pitch * 0.5).sin_cos();
        // ⛔ **Nada de `..Self::default()` aqui.** O `Default` é escrito em termos DESTA função (o
        // enquadramento inicial é um yaw/pitch nomeado), então herdar dele seria recursão infinita —
        // e ela aparece como um teste a estourar a pilha, não como um erro de compilação. Custou
        // uma corrida para descobrir; o custo de a evitar é escrever os quatro campos.
        Self {
            rotation: quat_mul([0.0, sy, 0.0, cy], [sp, 0.0, 0.0, cp]),
            half_extent: 0.8,
            target: [0.0; 3],
            lens: Lens::Perspective {
                half_fov: DEFAULT_HALF_FOV,
            },
        }
    }

    /// ⭐ **A que distância do plano do alvo está o olho** — `None` na paralela, onde ele está no
    /// infinito e a pergunta não tem resposta finita.
    ///
    /// A conta é a que define a lente: `tan(meia abertura) = half_extent / dist`.
    #[must_use]
    pub fn eye_distance(&self) -> Option<f32> {
        match self.lens {
            Lens::Ortho => None,
            Lens::Perspective { half_fov } => {
                let t = half_fov.tan();
                (t > 0.0 && t.is_finite()).then(|| self.half_extent / t)
            }
        }
    }

    /// Onde o olho está, no mundo. `None` na paralela, pela mesma razão.
    #[must_use]
    pub fn eye(&self) -> Option<[f32; 3]> {
        let d = self.eye_distance()?;
        let (_, _, fwd) = self.basis();
        Some([
            self.target[0] + fwd[0] * d,
            self.target[1] + fwd[1] * d,
            self.target[2] + fwd[2] * d,
        ])
    }

    /// ⭐ **Quantos pixels mede uma unidade de mundo NAQUELE ponto.**
    ///
    /// ⚠️ Na paralela é uma constante do quadro; na convergente **depende da distância**, e é por
    /// isso que esta pergunta tem de receber o ponto. Um gizmo dimensionado pela constante do quadro
    /// encolheria e cresceria conforme a peça se afasta — e as alças deixariam de medir o que
    /// dizem medir, que é o defeito que este módulo já nomeou uma vez (`MIN_ARM_PX`).
    #[must_use]
    pub fn px_per_world_at(&self, p: [f32; 3], screen: Screen) -> f32 {
        let base = screen.px_per_world();
        let Some(dist) = self.eye_distance() else {
            return base;
        };
        let (_, _, fwd) = self.basis();
        let z = dot(
            [
                p[0] - self.target[0],
                p[1] - self.target[1],
                p[2] - self.target[2],
            ],
            fwd,
        );
        let ahead = dist - z;
        if ahead <= dist * NEAR_FRACTION {
            return base;
        }
        base * dist / ahead
    }

    /// A base ortonormal da câmera: `(direita, cima, para-o-observador)`.
    ///
    /// ⚠️ **Projeção ortográfica**, e isso é uma escolha com data: é a que a W0 validou, e é a que
    /// o matcap pressupõe (`ph2d-mesh-render::matcap` amostra pela normal de vista, com a vista em
    /// `(0,0,1)`). Perspectiva é item ABERTO — ela muda o *feel* de um modelador e merece a sua
    /// própria comparação lado a lado, não uma troca silenciosa.
    ///
    /// ⚠️ A trigonometria daqui **não** fere o HR-5: a câmera é estado de VISTA — não entra no
    /// documento salvo, não entra no undo e não entra em hash de replay nenhum.
    #[must_use]
    pub fn basis(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let q = self.rotation;
        (
            quat_rotate(q, [1.0, 0.0, 0.0]),
            quat_rotate(q, [0.0, 1.0, 0.0]),
            quat_rotate(q, [0.0, 0.0, 1.0]),
        )
    }

    /// ⭐ **Rotação LIVRE**: gira em torno de um eixo dado nas coordenadas da **própria câmera**.
    ///
    /// É a composição pela direita (`q ⊗ Δ`), e é ela que faz a rotação ser local — o eixo é o que
    /// o gesto nomeia na tela, e não um eixo do mundo. Daí não haver polo: nenhum eixo do mundo
    /// participa da conta.
    pub fn turn_local(&mut self, axis: [f32; 3], angle: f32) {
        self.rotation = quat_normalize(quat_mul(self.rotation, quat_axis_angle(axis, angle)));
    }

    /// Gira em torno de um eixo do **mundo** (composição pela esquerda) — o prato giratório.
    pub fn turn_world(&mut self, axis: [f32; 3], angle: f32) {
        self.rotation = quat_normalize(quat_mul(quat_axis_angle(axis, angle), self.rotation));
    }
}

/// **O mapeamento pixel ↔ plano da câmera**, e ele é o **único** neste módulo.
///
/// ⭐ A marcha de raios constrói os raios a partir desta conta, e o gizmo projeta as alças com ela.
/// Duas cópias seriam duas respostas para *"onde este ponto cai na tela?"* — e a divergência não
/// apareceria como erro, apareceria como uma alça que se agarra meio pixel ao lado da superfície
/// que ela diz mover. É a mesma razão pela qual o shell guarda o `last_canvas` em vez de repetir a
/// aritmética do layout.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Screen {
    w: f32,
    h: f32,
    /// Metade do lado **menor**, em pixels — é ele que fixa a escala, para o quadro não deformar.
    half: f32,
    half_extent: f32,
}

impl Screen {
    #[must_use]
    pub fn new(w: u32, h: u32, half_extent: f32) -> Self {
        Self {
            w: w as f32,
            h: h as f32,
            half: (w.min(h) as f32) * 0.5,
            half_extent,
        }
    }

    #[must_use]
    pub fn width(self) -> f32 {
        self.w
    }

    #[must_use]
    pub fn height(self) -> f32 {
        self.h
    }

    /// Pixel → plano da câmera, em unidades de **mundo**.
    #[must_use]
    pub fn plane_at(self, x: f32, y: f32) -> (f32, f32) {
        (
            (x - self.w * 0.5) / self.half * self.half_extent,
            -(y - self.h * 0.5) / self.half * self.half_extent,
        )
    }

    /// A inversa exacta de [`Screen::plane_at`]. O gate `a_pixel_survives_the_round_trip` prende as
    /// duas — uma inversa escrita à mão é onde um sinal trocado sobrevive anos.
    #[must_use]
    pub fn pixel_of(self, u: f32, v: f32) -> (f32, f32) {
        (
            u / self.half_extent * self.half + self.w * 0.5,
            -v / self.half_extent * self.half + self.h * 0.5,
        )
    }

    /// Quantos **pixels** mede uma unidade de mundo neste enquadramento. É o número que converte um
    /// tamanho de alça (que é de tela) num tamanho de gizmo (que é de mundo).
    #[must_use]
    pub fn px_per_world(self) -> f32 {
        self.half / self.half_extent
    }
}

impl Orbit {
    /// **Onde um ponto do mundo cai na tela**, e a que profundidade.
    ///
    /// Devolve `(pixel, profundidade)`, com a profundidade a crescer **na direção do observador** —
    /// é ela que resolve qual de duas alças sobrepostas está à frente.
    ///
    /// ⚠️ **`None` quando o ponto não TEM projeção** — na convergente, um ponto ao lado do olho ou
    /// atrás dele. Não há pixel honesto para ele, e devolver um seria pintar uma alça num sítio
    /// arbitrário: quem recebe `None` não desenha e não oferece (é o `live` do gizmo, que já
    /// existia por outra razão). Na paralela **nunca** é `None` — não há divisão nenhuma.
    #[must_use]
    pub fn project(&self, p: [f32; 3], screen: Screen) -> Option<([f32; 2], f32)> {
        let (right, up, fwd) = self.basis();
        let v = [
            p[0] - self.target[0],
            p[1] - self.target[1],
            p[2] - self.target[2],
        ];
        let z = dot(v, fwd);
        let (mut u, mut w) = (dot(v, right), dot(v, up));
        if let Some(dist) = self.eye_distance() {
            let ahead = dist - z;
            if ahead <= dist * NEAR_FRACTION {
                return None;
            }
            // ⭐ A convergência inteira, numa linha: o que está mais longe do olho encolhe na mesma
            // proporção. No plano do alvo (`z = 0`) o fator é 1 — é ali que as duas lentes coincidem.
            let k = dist / ahead;
            u *= k;
            w *= k;
        }
        let (x, y) = screen.pixel_of(u, w);
        Some(([x, y], z))
    }

    /// ⭐ **O raio de um pixel** — `(origem, direção unitária)`.
    ///
    /// ⚠️ **É a porta única**, e passou a sê-lo em 20/08: a marcha de raios reconstruía esta mesma
    /// aritmética com um afastamento próprio, e duas respostas para *"que raio sai daqui?"* é como
    /// uma alça se agarra meio pixel ao lado da superfície que ela diz mover — o defeito que o doc
    /// deste arquivo já prometia não ter.
    #[must_use]
    pub fn ray(&self, x: f32, y: f32, screen: Screen) -> ([f32; 3], [f32; 3]) {
        let (u, v) = screen.plane_at(x, y);
        self.ray_at_plane(u, v)
    }

    /// O mesmo raio, a partir de coordenadas **do plano do alvo** em unidades de mundo — que é o
    /// que a marcha tem em mãos (ela varre o plano, não a grelha de pixels).
    ///
    /// ⚠️ **A origem fica FORA da peça**, e não sobre o plano do alvo: na paralela ela recua
    /// [`ORTHO_START`]; na convergente ela **é o olho**, que já está recuado por construção. Uma
    /// origem sobre o plano perderia tudo o que está à frente dele — metade da peça, em silêncio.
    #[must_use]
    pub fn ray_at_plane(&self, u: f32, v: f32) -> ([f32; 3], [f32; 3]) {
        let (right, up, fwd) = self.basis();
        let on_plane = [
            self.target[0] + right[0] * u + up[0] * v,
            self.target[1] + right[1] * u + up[1] * v,
            self.target[2] + right[2] * u + up[2] * v,
        ];
        match self.eye() {
            None => (
                [
                    on_plane[0] + fwd[0] * ORTHO_START,
                    on_plane[1] + fwd[1] * ORTHO_START,
                    on_plane[2] + fwd[2] * ORTHO_START,
                ],
                [-fwd[0], -fwd[1], -fwd[2]],
            ),
            Some(eye) => {
                let d = [
                    on_plane[0] - eye[0],
                    on_plane[1] - eye[1],
                    on_plane[2] - eye[2],
                ];
                let len = dot(d, d).sqrt();
                if len <= 0.0 || !len.is_finite() {
                    return (eye, [-fwd[0], -fwd[1], -fwd[2]]);
                }
                (eye, [d[0] / len, d[1] / len, d[2] / len])
            }
        }
    }
}

/// Quanto o raio da lente **paralela** recua antes de marchar.
///
/// ⚠️ Sem olho não há afastamento natural, então este número tem de vir de algum lado: ele é o raio
/// da maior peça que o enquadramento inicial comporta com folga. Uma peça maior do que isto começaria
/// a ser marchada **por dentro**, e o que se veria era a superfície de trás.
///
/// ⚠️ Na convergente ele **não** é usado: ali o olho já está recuado `half_extent / tan(fov/2)`, e
/// somar mais afastaria a origem do sítio de onde a projeção diz que ela vê.
pub const ORTHO_START: f32 = 4.0;
