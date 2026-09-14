//! **O SOMBREAMENTO RENDER** — do G-buffer aos pixels com MATERIAL, LUZ e CÉU (`docs/Render3d/05`).
//!
//! Irmão do [`super::shade`], com a mesma fronteira: o traçador entrega máscara e normal, e isto é a
//! única coisa que sabe o que é uma cor. O que muda é a pergunta — o matcap responde *«que cor tem
//! esta normal na fotografia?»*; isto responde *«que luz esta superfície devolve ao olho?»*:
//!
//! 1. a lei do material é a da `ph2d-material` (o OpenPBR provado contra o MaterialX corrido);
//! 2. a direcção de vista de cada pixel sai do MESMO raio que o traçou ([`Orbit::ray_at_plane`]) —
//!    com a lente convergente ela muda de pixel para pixel, e uma `(0, 0, 1)` fixa poria o realce
//!    no sítio errado de toda peça vista em perspectiva;
//! 3. o olhar (exposição e vista) é o da `ph2d-view-transform`, aplicado **por amostra, antes** da
//!    média da borda — a média de duas luzes já transformadas é o que o olho vê, e transformar a
//!    média de duas luzes da cena não é.
//!
//! ⚠️ **Tudo em espaço de VISTA**, que é onde o G-buffer guarda a normal: as lâmpadas e o céu chegam
//! já nesse referencial, e quem as converte é quem sabe de onde elas vêm.

use super::*;
use ph2d_material::{Environment, Surface};
use ph2d_view_transform::Look;

/// Uma luz direcional, em espaço de VISTA.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lamp {
    /// A direcção **para** a luz, unitária.
    pub to_light: [f32; 3],
    /// A radiância que chega (`cor × intensidade`, a convenção do MaterialX).
    pub radiance: [f32; 3],
}

/// A luz de uma cena: as lâmpadas e o céu.
pub struct Lighting<'a> {
    pub lamps: &'a [Lamp],
    pub sky: &'a (dyn Environment + Sync),
}

/// ⭐⭐⭐ **OS MATERIAIS DA PEÇA, e de quem é cada pixel** (`docs/Render3d/05`).
///
/// # ⚠️ Porque são DOIS campos e não um material só
///
/// Uma peça é uma árvore de folhas, e cada folha tem o seu aspecto. O que o traçado entrega é um
/// **ponto de mundo** por pixel ([`Gbuffer::point`]); quem o traduz em *«a folha nº 3»* é a lei do
/// [`ph2d_field_eval::owners`], que é a MESMA que a selecção por clique usa.
///
/// ⚠️ **`owners: None` é o caso de UM**, e não um caso especial: uma peça com um material só não
/// precisa de perguntar de quem é o pixel, e não perguntar é exactamente o custo zero. É assim que
/// o quadro de omissão continua a ser o de sempre, byte a byte.
///
/// ⛔ **A ordem de [`Self::all`] é a das folhas do [`ph2d_field_eval::owners::Owners`]**, e quem as
/// constrói constrói as duas — uma lista com outra ordem pintaria cada peça com a cor da vizinha,
/// sem erro nenhum.
pub struct Surfaces<'a> {
    /// Um material por folha. **Nunca vazio** — ver [`Self::of`].
    pub all: &'a [Surface],
    /// De quem é cada ponto. `None` ⇒ a peça inteira usa `all[0]`.
    pub owners: Option<&'a ph2d_field_eval::owners::Owners>,
}

impl<'a> Surfaces<'a> {
    /// O material deste ponto.
    ///
    /// ⚠️ **A rede é `all[0]`**, e ela não é decorativa: o `Owners` pode devolver um índice de uma
    /// peça que já mudou entre o traçado e o sombreamento (eles correm em threads diferentes). Uma
    /// indexação crua entraria em pânico **no meio de um quadro**; pintar com o primeiro material é
    /// uma resposta que o artista lê como «ainda não actualizou», que é o que de facto aconteceu.
    ///
    /// ⛔ **O irmão `of`, que devolvia só o dono, foi APAGADO em 14/09** — o `mix_of` tomou-lhe os
    /// dois chamadores, e um método que ninguém chama é **lixo**, não um morto a ligar.
    /// ⭐⭐⭐ **OS DOIS MATERIAIS QUE DISPUTAM ESTE PONTO, e o peso do segundo** — a fronteira de cor,
    /// suavizada.
    ///
    /// # ⛔⛔ Porque ela existe: a fronteira de COR não é uma silhueta
    ///
    /// O anti-serrilhado deste ficheiro corre nas [`Gbuffer::edges`] — pixels em que umas
    /// sub-amostras acertam a peça e outras não. Uma fronteira **entre dois materiais** no meio da
    /// peça não é nenhuma dessas: ali **todas** as sub-amostras acertam, não há registo de borda, e
    /// a cor muda de um pixel para o outro **a pique**. Medido: um degrau de até `236` bytes numa
    /// banda de `1`–`2` px — *a peça fica com o contorno liso e uma escada por dentro*.
    ///
    /// ⇒ [`ph2d_field_eval::owners::Owners::mix_at`], cuja largura de transição sai da **geometria**
    /// e não de um número escolhido.
    ///
    /// ⚠️ **`t == 0` é o caminho de sempre**, e ele cobre os dois casos que dominam: uma peça de um
    /// material só (`owners: None`) e todo pixel longe de uma fronteira. *O custo desta lei mora nos
    /// `0,5 %` de pixels que estão em cima dela.*
    fn mix_of(&self, p: [f32; 3], pixel_world: f32) -> (&Surface, &Surface, f32) {
        let rede = self.all.first().unwrap_or(&self.all[0]);
        let Some(o) = self.owners else {
            return (rede, rede, 0.0);
        };
        let (a, b, t) = o.mix_at(p, pixel_world);
        (
            self.all.get(a).unwrap_or(rede),
            self.all.get(b).unwrap_or(rede),
            t,
        )
    }
}

/// A direcção **para o observador** no pixel `(x, y)`, em espaço de vista — o raio do traçado, ao
/// contrário.
///
/// ⚠️ `pub(crate)` para o gate da costura lhe poder chamar: a lei de que ela é a do RAIO, e não uma
/// constante, é exactamente o que aquele gate afirma.
pub(crate) fn view_direction(cam: &Orbit, screen: &Screen, x: usize, y: usize) -> [f32; 3] {
    let (u, v) = screen.plane_at(x as f32 + 0.5, y as f32 + 0.5);
    let (_, d) = cam.ray_at_plane(u, v);
    let (right, up, toward_eye) = cam.basis();
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    [-dot(d, right), -dot(d, up), -dot(d, toward_eye)]
}

/// A luz que a superfície devolve pela direcção `v`, já com o olhar — em linear de ECRÃ.
fn radiance(
    surface: &Surface,
    light: &Lighting<'_>,
    look: Look,
    n: [f32; 3],
    v: [f32; 3],
) -> [f32; 3] {
    let add = |a: [f32; 3], b: [f32; 3]| [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
    let mut rgb = surface.indirect(n, v, light.sky);
    for lamp in light.lamps {
        rgb = add(rgb, surface.direct(n, v, lamp.to_light, lamp.radiance));
    }
    look.apply(add(rgb, surface.emission(n, v)))
}

/// Colore o G-buffer com um material sob uma luz e devolve RGBA8 **pré-multiplicado**.
///
/// ⚠️ As mesmas duas leis do [`super::shade`]: a borda é a média em LINEAR (aqui, linear de ecrã,
/// porque o olhar já foi aplicado a cada amostra), e o pixel de fundo sai com os bytes EXACTOS que o
/// chamador pediu.
///
/// ⚠️ **As linhas correm em paralelo** e as bordas depois, em série: uma borda precisa do fundo e de
/// quatro amostras, e são poucas (`0,5–1,2 %` dos pixels, `docs/3DModeling/05`).
#[must_use]
pub fn shade_render(
    g: &Gbuffer,
    cam: &Orbit,
    surfaces: &Surfaces<'_>,
    light: &Lighting<'_>,
    look: Look,
    background: [u8; 4],
) -> Vec<u8> {
    let (w, h) = (g.width as usize, g.height as usize);
    let mut out = vec![0u8; w * h * 4];
    if w == 0 || h == 0 {
        return out;
    }
    let screen = Screen::new(g.width, g.height, cam.half_extent);
    // ⭐ **A largura da transição entre dois materiais, em MUNDO** — a única coisa que o
    // [`Surfaces::mix_of`] não pode adivinhar. Ver [`ph2d_field_eval::owners::Owners::mix_at`].
    let pixel_world = BOUNDARY_PIXELS * 2.0 * cam.half_extent / w.min(h).max(1) as f32;
    let write = |px: &mut [u8], c: [f32; 4]| {
        px[0] = ph2d_color::srgb::linear_to_srgb_byte(c[0]);
        px[1] = ph2d_color::srgb::linear_to_srgb_byte(c[1]);
        px[2] = ph2d_color::srgb::linear_to_srgb_byte(c[2]);
        px[3] = (c[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
    };

    out.par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
        for (x, px) in row.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let i = y * w + x;
            if g.hit[i] {
                // ⭐⭐⭐ **O MATERIAL sai do PONTO** — ver [`Surfaces`]. Numa peça de um material só
                // isto é uma leitura de `all[0]` e mais nada.
                let v = view_direction(cam, &screen, x, y);
                let c = mixed_radiance(
                    surfaces,
                    g.point[i],
                    pixel_world,
                    light,
                    look,
                    g.normal[i],
                    v,
                );
                write(px, [c[0], c[1], c[2], 1.0]);
            } else {
                // ⚠️ Copiado, e não passado pela conversão — a mesma cerca do `shade`.
                px.copy_from_slice(&background);
            }
        }
    });

    let bg_a = f32::from(background[3]) / 255.0;
    let bg = [
        ph2d_color::srgb::srgb_to_linear_byte(background[0]) * bg_a,
        ph2d_color::srgb::srgb_to_linear_byte(background[1]) * bg_a,
        ph2d_color::srgb::srgb_to_linear_byte(background[2]) * bg_a,
        bg_a,
    ];
    for e in &g.edges {
        let i = e.pixel as usize;
        // ⚠️ **A vista do CENTRO do pixel serve às quatro amostras**: dentro de um pixel a direcção do
        // raio muda menos do que o passo de um byte move a luz, e a borda não guarda as posições.
        let v = view_direction(cam, &screen, i % w, i / w);
        // ⚠️ **E o MATERIAL do centro serve às quatro amostras**, pela mesma razão da vista: a borda
        // não guarda os pontos das sub-amostras. ⛔ Numa silhueta entre DUAS peças de cores
        // diferentes isto pinta a borda com a cor da que o centro apanhou — declarado, e é a mesma
        // aproximação que a direcção de vista já faz.
        let mut acc = [0.0f32; 4];
        for k in 0..4 {
            let c = if e.hit[k] {
                let rgb = mixed_radiance(
                    surfaces,
                    g.point[i],
                    pixel_world,
                    light,
                    look,
                    e.normal[k],
                    v,
                );
                [rgb[0], rgb[1], rgb[2], 1.0]
            } else {
                bg
            };
            for j in 0..4 {
                acc[j] += c[j] * 0.25;
            }
        }
        write(&mut out[i * 4..i * 4 + 4], acc);
    }
    out
}

/// ⭐⭐⭐ **QUANTOS PIXELS A FRONTEIRA ENTRE DOIS MATERIAIS LEVA A MUDAR** — medido, com a tabela.
///
/// # ⚠️ Porque não é `1`, que é o que a forma fechada dá
///
/// O [`ph2d_field_eval::owners::Owners::mix_at`] converte a diferença de campos numa distância
/// supondo que se anda **perpendicular à fronteira** (`|∇d| ≈ 2`). Mas quem percorre os pixels anda
/// **ao longo da superfície visível**, e o ângulo entre as duas direcções encolhe o passo efectivo —
/// a rampa sai mais estreita do que um pixel e volta a ler-se como degrau.
///
/// ⭐ **O factor é a correcção desse ângulo, e foi VARRIDO** (duas esferas, uma vermelha e uma azul,
/// `640×360`; a coluna é quanto a COR acrescenta ao degrau, já subtraído o controlo de duas folhas
/// da mesma cor, e só sobre vizinhos que são vizinhos **na superfície**):
///
/// | factor | união dura | união suave |
/// |---:|---:|---:|
/// | `0` (sem a lei) | `+166` | `+191` |
/// | `1` (a forma fechada crua) | `+84` | `+148` |
/// | `1,5` | `+48` | `+109` |
/// | **`2`** ⬅ | **`+34`** | **`+95`** |
/// | `3` | `+16` | `+75` |
/// | `4` | `+2` | `+51` |
///
/// ⚠️ **O joelho está em `2`**, e acima dele o que se compra é **desfoque**: a fronteira deixa de ser
/// uma aresta suavizada e passa a ser um degradê de N pixels entre duas cores. *Mais suave nem sempre
/// é melhor — uma fronteira de material tem de continuar a ler-se como fronteira.*
///
/// # ⏳ E o que uma cura COMPLETA faria
///
/// A correcta é a derivada de `d` no ECRÃ (`t = ½ − d / (2·|∂d/∂pixel|)`), que dispensa este factor
/// por medir o ângulo em cada pixel. Ela pede o **gradiente** dos dois campos (seis avaliações por
/// pixel de fronteira, que são `~0,5 %` dos pixels) e fica **nomeada**, não construída.
///
/// ⛔ **E a cura de raiz é outra: sub-amostrar o DONO.** O padrão `ROOK` já re-marcha quatro
/// sub-amostras num pixel de silhueta — mas o [`EdgePixel`] guarda **normais**, não pontos, e a
/// marcha não conhece donos. Dar-lhos é o *id-buffer*, que esta linha **mediu e recusou**.
const BOUNDARY_PIXELS: f32 = 2.0;

/// ⭐⭐ **A luz de um ponto, com a fronteira entre dois materiais SUAVIZADA** — ver
/// [`Surfaces::mix_of`].
///
/// ⚠️ **Sombreia DUAS vezes e mistura o resultado**, e não os materiais: é isso que uma
/// super-amostragem convergiria a dar, e misturar os *parâmetros* de dois OpenPBR não é misturar a
/// luz que eles devolvem — um metal e um dieléctrico a meio caminho não são um meio-metal.
///
/// ⛔ **E ela só paga o dobro onde há fronteira** (`t > 0`): fora dela, e numa peça de um material
/// só, é uma chamada e mais nada.
fn mixed_radiance(
    surfaces: &Surfaces<'_>,
    p: [f32; 3],
    pixel_world: f32,
    light: &Lighting<'_>,
    look: Look,
    n: [f32; 3],
    v: [f32; 3],
) -> [f32; 3] {
    let (a, b, t) = surfaces.mix_of(p, pixel_world);
    let ca = radiance(a, light, look, n, v);
    if t <= 0.0 {
        return ca;
    }
    let cb = radiance(b, light, look, n, v);
    [0, 1, 2].map(|i| ca[i] + (cb[i] - ca[i]) * t)
}
