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
use crate::ground_shade::{
    edge_ground_bounce, edge_ground_factor, ground_factors, shadowed_background,
};
use ph2d_material::{Environment, Surface};

/// ⭐⭐ O vocabulário da luz — ver [`shade_render_luz`].
#[path = "shade_render_luz.rs"]
mod shade_render_luz;
pub use shade_render_luz::{Lamp, Lighting, POINT_LAMP_MIN_DISTANCE, PointLamp};
pub(crate) use shade_render_luz::{PISO_DA_LAMPADA, chega_da_lampada};

/// ⭐ **Um ambiente que só tem a parcela DIFUSA** — a irradiância que a cena devolve a este pixel.
///
/// ⚠️ A [`Environment::radiance`] responde **zero** de propósito: ela é a pergunta do lóbulo
/// ESPECULAR (*«que luz vem daquela direcção?»*), e uma média do hemisfério não a responde.
/// Devolver a média ali poria um realce de espelho com a cor do ricochete e sem sítio nenhum.
pub(crate) struct SoIrradiancia(pub [f32; 3]);

impl Environment for SoIrradiancia {
    fn radiance(&self, _dir: [f32; 3], _alpha: f32) -> [f32; 3] {
        [0.0; 3]
    }
    fn irradiance(&self, _n: [f32; 3]) -> [f32; 3] {
        self.0
    }
}

/// A base de VISTA — o que converte uma direcção de MUNDO no referencial em que o G-buffer guarda a
/// normal.
///
/// ⚠️ Ela resolve-se **uma vez por quadro** e não por pixel: `cam.basis()` é a mesma para a imagem
/// inteira. *Uma base reconstruída dentro do laço corre onde o laço corre.*
#[derive(Clone, Copy, Debug)]
pub(crate) struct ViewBasis {
    right: [f32; 3],
    up: [f32; 3],
    toward_eye: [f32; 3],
}

impl ViewBasis {
    pub(crate) fn of(cam: &Orbit) -> Self {
        let (right, up, toward_eye) = cam.basis();
        Self {
            right,
            up,
            toward_eye,
        }
    }

    /// ⚠️ **O sinal é o do [`view_direction`]**, e não uma segunda convenção: lá a direcção do raio
    /// é negada para apontar ao olho, aqui a direcção já aponta para a luz.
    pub(crate) fn world_to_view(self, w: [f32; 3]) -> [f32; 3] {
        let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        [dot(w, self.right), dot(w, self.up), dot(w, self.toward_eye)]
    }

    /// ⭐⭐ **O caminho de volta** — a base é ortonormal, logo a transposta é a inversa.
    ///
    /// # ⛔⛔ Porque ele nasce aqui e não onde foi preciso
    ///
    /// Esta conta estava escrita **à mão em TRÊS sítios** (a normal do G-buffer vive em VISTA e o
    /// campo vive no MUNDO): o passe da sombra, o da oclusão e uma sonda dos testes — e a `W5` ia
    /// escrevê-la uma quarta vez. *Uma lei escrita em dois sítios ainda não é uma lei; só uma PORTA
    /// é.*
    ///
    /// ⚠️ **Os três diziam-no por escrito e nenhum a partilhava:** o comentário do passe da sombra
    /// promete *«é a MESMA conversão que o `shade_render` faz, senão a sombra e a luz discordariam
    /// sobre quem vê quem»* — sobre uma cópia. *Uma promessa de igualdade ao lado de uma segunda
    /// cópia é exactamente a forma que diverge no dia em que alguém corrigir uma delas.*
    ///
    /// ⚠️ **E ela não é decorativa:** a marcha guarda toda normal em espaço de VISTA (*«é nele que o
    /// matcap vive»*), logo quem lança um raio solto e usa a normal do acerto para iluminar
    /// **tem** de a trazer de volta. A 1.ª referência da caixa de Cornell não a trazia, e a
    /// irradiância mudava `45 %` só por a câmera rodar — com o SINAL do sangramento na mesma certo,
    /// porque aquela câmera olha de frente e ali os dois referenciais quase coincidem.
    pub(crate) fn view_to_world(self, v: [f32; 3]) -> [f32; 3] {
        [
            v[0] * self.right[0] + v[1] * self.up[0] + v[2] * self.toward_eye[0],
            v[0] * self.right[1] + v[1] * self.up[1] + v[2] * self.toward_eye[1],
            v[0] * self.right[2] + v[1] * self.up[2] + v[2] * self.toward_eye[2],
        ]
    }
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
    /// ⚠️⚠️ **O irmão [`Self::of`] foi APAGADO em 14/09 e VOLTOU em 17/09** — e as duas decisões
    /// estão certas, com a mesma lei: *um método que ninguém chama é lixo*. Ele saiu quando o
    /// `mix_of` lhe tomou os dois chamadores e voltou quando a `W5` trouxe o terceiro — o passe do
    /// ricochete, que pergunta pelo material de um ponto que um RAIO acertou, onde não há pixel
    /// nem largura de fronteira para suavizar.
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
    /// ⭐ **O material de um ponto, sem fronteira suavizada** — para quem não tem pixel.
    ///
    /// ⚠️ **Ele NÃO é o [`Self::mix_of`] com `t = 0`:** aquele pergunta *«que dois materiais
    /// disputam este PIXEL e com que peso»*, e a largura da transição sai do tamanho do pixel no
    /// mundo. Um raio de ricochete não tem pixel — ele acerta um ponto —, e inventar-lhe uma
    /// largura seria inventar a resposta.
    pub(crate) fn of(&self, p: [f32; 3]) -> &Surface {
        let rede = &self.all[0];
        self.owners
            .and_then(|o| o.at(p))
            .and_then(|k| self.all.get(k))
            .unwrap_or(rede)
    }

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
pub(crate) fn radiance_scene(
    surface: &Surface,
    light: &Lighting<'_>,
    pres: &Presentation,
    geom: PixelGeom,
) -> [f32; 3] {
    let PixelGeom {
        i: _,
        p,
        n,
        v,
        k,
        k_estilo,
        basis,
    } = geom;
    // ⭐⭐⭐ **O MATERIAL NESTE PONTO** — a curvatura é a única entrada geométrica que a lei do
    // OpenPBR pede, e ela chega do CAMPO (`crate::curvatura`). ⚠️ Com a subsuperfície maciça
    // desligada (a omissão) ninguém a lê, e esta linha é uma cópia de 15 floats que não muda um bit.
    // ⚠️ **O módulo é tomado AQUI**, e não dentro da [`crate::curvatura`]: a lei do OpenPBR pede um
    // comprimento (a referência estima-o por `length(fwidth(N))`), e a tinta por curvatura da `W8`
    // pede o SINAL. *Uma porta que deita fora o sinal serve o primeiro consumidor e apaga o
    // segundo* — a imagem desta é byte a byte a mesma, porque tomar o módulo antes ou depois de
    // guardar dá o mesmo `f32`.
    let surface = &surface.at_curvature(k.abs());
    let add = |a: [f32; 3], b: [f32; 3]| [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
    // ⭐⭐⭐ **A OCLUSÃO É A SOMBRA DO CÉU** — ela multiplica o que o AMBIENTE entrega, e mais nada.
    //
    // ⚠️ **Não toca nas lâmpadas** (elas têm sombra a sério, §27) nem na **emissão** (uma superfície
    // que é ela própria uma luz não se apaga por ter um vizinho). *Uma oclusão aplicada ao pixel
    // inteiro é a aparência de sujidade que este passe existe para não ter.*
    //
    // ⚠️⚠️ **APROXIMAÇÃO DECLARADA:** o `indirect` devolve o difuso **e** o especular do ambiente
    // numa chamada só, e a oclusão exacta do especular não é a do difuso (ela depende da rugosidade
    // e da direcção do lóbulo). Escalar os dois pelo mesmo número escurece reflexos a mais numa
    // superfície polida. Separá-los é mexer na fronteira do `ph2d-material`, e fica nomeado.
    let ceu = light.shadows.map_or(1.0, |s| s.ambient_at(geom.i));
    let mut rgb = surface.indirect(n, v, light.sky).map(|c| c * ceu);
    // ⭐⭐⭐ **A OUTRA METADE DO HEMISFÉRIO: a luz que a CENA devolve** (a `W5`).
    //
    // ⚠️ **Ela entra SOMADA e num ambiente próprio**, e não a multiplicar o céu: o `ambient_at`
    // responde *«que fracção do céu chega»* e este responde *«que luz mais chega»*. Somar é o que
    // as torna as duas parcelas de um integral; multiplicar faria a cena APAGAR o céu.
    //
    // ⚠️ **Só a parcela DIFUSA**, declarado: uma irradiância por pixel não tem direcção, e o lóbulo
    // especular pergunta *«que luz vem DAQUELA direcção»*. O especular indirecto continua a ser o
    // céu — ver o [`crate::bounce`].
    //
    // ⭐ **Canal vazio ⇒ o quadro de sempre, ao BIT**: sem ele o ramo não corre.
    let devolvida = light.shadows.map_or([0.0; 3], |s| s.bounce_at(geom.i));
    if devolvida != [0.0; 3] {
        rgb = add(rgb, surface.indirect(n, v, &SoIrradiancia(devolvida)));
    }
    // ⭐⭐⭐ **A SATURAÇÃO DA LUZ INDIRECTA** (`ph2d_style`, a `W8`) — aqui, sobre as DUAS parcelas
    // de ambiente e **antes das lâmpadas**: saturar no fim saturaria também o realce do sol.
    //
    // ⚠️ Com o valor de fábrica (`1`) isto é a identidade **ao bit**, por construção — e o gémeo do
    // dispositivo aplica-a exactamente no mesmo ponto da soma.
    rgb = pres.style.saturate_indirect(rgb);
    for lamp in light.lamps {
        rgb = add(rgb, surface.direct(n, v, lamp.to_light, lamp.radiance));
    }
    // ⭐⭐⭐ **AS LUZES-OBJECTO** — a direcção e a distância saem do PONTO deste pixel.
    for (l, lamp) in light.points.iter().enumerate() {
        let d = [
            lamp.world[0] - p[0],
            lamp.world[1] - p[1],
            lamp.world[2] - p[2],
        ];
        let cru = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        let piso = PISO_DA_LAMPADA;
        // ⚠️⚠️ **O piso protege DUAS grandezas, e a primeira redacção só protegia uma.** Ela coava
        // o `r²` e normalizava o vector **cru**: com a luz exactamente sobre o ponto, `d` é o vector
        // ZERO, a direcção sai `[0,0,0]`, o `N·L` dá `0` e o pixel fica **PRETO** — o mesmo sintoma
        // que a divisão por zero, por outro caminho. *Uma mutação que punha o piso a `0` sobreviveu
        // ao gate porque o produto já estava a falhar do outro lado.*
        //
        // ⇒ abaixo do piso a direcção é a **NORMAL**: a luz está em cima da superfície, logo ela
        // ilumina-a de frente. É o limite certo, e é finito.
        let to_light = if cru <= piso {
            n
        } else {
            let inv = cru.sqrt().recip();
            basis.world_to_view([d[0] * inv, d[1] * inv, d[2] * inv])
        };
        // ⭐⭐⭐ **E a sombra entra AQUI, na radiância que chega** — não no `N·L` e não no resultado.
        //
        // ⚠️ **É o sítio certo por uma razão física:** a visibilidade multiplica a LUZ INCIDENTE, e
        // o material decide sozinho o que fazer com ela. Pô-la no fim escureceria também o
        // especular do céu e a emissão — *uma peça tapada por outra continua a reflectir o
        // ambiente, e continua a brilhar se for ela própria uma luz.*
        let visivel = light.shadows.map_or(1.0, |s| s.at(l, geom.i));
        let chega = chega_da_lampada(lamp, cru, visivel);
        // ⭐⭐⭐ **E a subsuperfície lê a visibilidade MOLE, que é a da vizinhança.**
        //
        // A luz que uma peça translúcida devolve não entrou por ESTE ponto: entrou à volta dele e
        // espalhou-se por baixo da superfície ⇒ a borda de uma sombra num jade é mole, e a do
        // especular ao lado continua dura. Ver [`crate::sss_shadow`] e o report de 2026-09-18.
        //
        // ⚠️ **Sem a passagem assada o `soft_at` devolve a DURA nos três canais** ⇒ o `direct_sss`
        // sai pelo braço curto e o quadro é o de sempre, ao bit.
        let mole = light.shadows.map_or([visivel; 3], |s| s.soft_at(l, geom.i));
        let chega_mole = [0, 1, 2].map(|k| chega_da_lampada(lamp, cru, mole[k])[k]);
        rgb = add(rgb, surface.direct_sss(n, v, to_light, chega, chega_mole));
    }
    // ⭐⭐⭐ **O ESTILO ENTRA AQUI, entre a física e o olhar** (`docs/Render3d/03`, a `W8`): a luz
    // que a superfície devolve já está toda somada — céu, ricochete, lâmpadas e emissão —, e é sobre
    // ela que a direcção de arte mente. ⛔ Depois do olhar seria tinta sobre um valor já cortado, e
    // um contorno que não respira com a exposição separa-se da peça ao expor.
    //
    // ⚠️ **Com o estilo de fábrica isto é a identidade AO BIT** e a imagem é a de antes, byte a
    // byte — por construção e com gate (ver o [`ph2d_style`]).
    // ⭐⭐ **E AQUI a função parte-se em duas leituras da MESMA lei** (a `W7`): o que sai daqui é
    // **cena-linear** — a luz antes do olhar —, que é o que o BRILHO tem de ler
    // (`docs/Render3d/12` §2). ⛔ Lê-lo depois do olhar é o que o `01` §2 chama de *«sem o `1`, o
    // bloom mente»*: o tonemapper comprime justamente o que havia para colher.
    pres.style.apply(
        add(rgb, surface.emission(n, v)),
        ph2d_style::Point {
            // ⚠️ **`|N·V|`**, e o valor absoluto não é defensivo: numa silhueta o produto passa por
            // zero e muda de sinal com o ruído da normal, e um contorno que pisca não é um contorno.
            facing: (n[0] * v[0] + n[1] * v[1] + n[2] * v[2]).abs(),
            curvature: pres.styled_curvature(k_estilo),
        },
    )
}

/// A luz que este pixel manda ao olho, **depois do olhar** — o que o byte lê.
///
/// ⚠️ **Ela DELEGA na irmã** e não repete uma linha: escrita duas vezes, a cena e o ecrã divergiam
/// no dia em que alguém tocasse numa só. *Uma lei escrita em dois sítios ainda não é uma lei.*
fn radiance(
    surface: &Surface,
    light: &Lighting<'_>,
    pres: &Presentation,
    geom: PixelGeom,
) -> [f32; 3] {
    pres.look.apply(radiance_scene(surface, light, pres, geom))
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
    pres: &Presentation,
    background: [u8; 4],
) -> Vec<u8> {
    let (w, h) = (g.width as usize, g.height as usize);
    let mut out = vec![0u8; w * h * 4];
    if w == 0 || h == 0 {
        return out;
    }
    let screen = Screen::new(g.width, g.height, cam.half_extent);
    // ⭐ **Uma vez por quadro** — ver [`ViewBasis`].
    let basis = ViewBasis::of(cam);
    // ⭐ **A largura da transição entre dois materiais, em MUNDO** — a única coisa que o
    // [`Surfaces::mix_of`] não pode adivinhar. Ver [`ph2d_field_eval::owners::Owners::mix_at`].
    #[allow(clippy::cast_possible_truncation)]
    let pixel_world = boundary_world(cam.half_extent, w.min(h) as u32);
    // ⭐⭐⭐ **O ALFA É PRÉ-MULTIPLICADO EM ECRÃ** — ver [`crate::premultiplicado`], que traz a
    // medição feita NO compositor. ⚠️ A codificação usa o alfa que de facto vai para o byte, e não
    // o `f32` antes do arredondamento: o consumidor compõe com o byte.
    let write = |px: &mut [u8], c: [f32; 4], luz: [f32; 3]| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let a = (c[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        let rgb = crate::premultiplicado::para_ecra([c[0], c[1], c[2]], a, luz);
        px[0] = rgb[0];
        px[1] = rgb[1];
        px[2] = rgb[2];
        px[3] = a;
    };
    let bg_a = f32::from(background[3]) / 255.0;
    let bg = [
        ph2d_color::srgb::srgb_to_linear_byte(background[0]) * bg_a,
        ph2d_color::srgb::srgb_to_linear_byte(background[1]) * bg_a,
        ph2d_color::srgb::srgb_to_linear_byte(background[2]) * bg_a,
        bg_a,
    ];
    // ⭐⭐⭐ **O CHÃO: quanto cada pixel de fundo escurece** — `1,0` onde não há chão ou nada o tapa.
    let (fatores, postas) = ground_factors(g, cam, &screen, basis, light);

    out.par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
        for (x, px) in row.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let i = y * w + x;
            if g.hit[i] {
                // ⭐⭐⭐ **O MATERIAL sai do PONTO** — ver [`Surfaces`]. Numa peça de um material só
                // isto é uma leitura de `all[0]` e mais nada.
                let v = view_direction(cam, &screen, x, y);
                let c = mixed_radiance(
                    surfaces,
                    PixelGeom {
                        i,
                        p: g.point[i],
                        n: g.normal[i],
                        v,
                        // ⚠️ Vazio ⇒ `0` ⇒ o piso do GLSL dá o raio de `100`, que é «plano».
                        k: g.curvature.get(i).copied().unwrap_or(0.0),
                        // ⚠️ Vazio ⇒ `0` ⇒ tinta nenhuma, que é o que o estilo de fábrica quer.
                        k_estilo: g.curvature_style.get(i).copied().unwrap_or(0.0),
                        basis,
                    },
                    pixel_world,
                    light,
                    pres,
                );
                write(px, [c[0], c[1], c[2], 1.0], [0.0; 3]);
            } else {
                // ⭐⭐⭐ **A LUZ QUE A PEÇA PÕE NO CHÃO** — somada em pré-multiplicado e com alfa
                // ZERO, que é o que um compositor lê como luz acrescentada. ⛔ Ela NÃO pode
                // multiplicar o fundo: o do modelador é transparente, e a wave inteira sairia num
                // pixel que não muda um bit (ver o cabeçalho do [`crate::ground_shade`]).
                let posta = postas.get(i).copied().unwrap_or([0.0; 3]);
                match fatores.get(i) {
                    // ⭐ O chão tapado: o fundo com a sombra por cima — ver [`shadowed_background`].
                    // ⭐⭐⭐ **A LUZ ENTRA SEPARADA DA COBERTURA** (`crate::premultiplicado`): a
                    // sombra TAPA e por isso é pré-multiplicada; a luz devolvida SOMA e por isso
                    // não é. ⚠️ Enfiadas no mesmo `vec4` — que é o que a `mais_luz` fazia — o
                    // empacotamento dividia a luz pelo alfa da sombra, e o gate do chão apanhou-o.
                    Some(&f) if f < 1.0 => write(px, shadowed_background(bg, f), posta),
                    // ⚠️ Copiado, e não passado pela conversão — a mesma cerca do `shade`. É também
                    // o chão onde nada tapa: a razão é EXACTAMENTE `1`, e os bytes são os de sempre.
                    // ⭐ **Com luz devolvida ele deixa de poder ser copiado** — ela é o que há para
                    // mostrar num pixel cujo fundo é preto transparente.
                    _ if posta != [0.0; 3] => write(px, bg, posta),
                    _ => px.copy_from_slice(&background),
                }
            }
        }
    });

    /// ⭐⭐⭐ **O PONTO DE UM PIXEL DE SILHUETA — e ele TEM de estar na peça** (report do dono,
    /// 2026-09-19: *«toda forma apresenta uma falsa outline branca de 1 pixel»*).
    ///
    /// # ⛔⛔⛔ O defeito, medido
    ///
    /// O material de uma sub-amostra é escolhido pela POSIÇÃO (`Surfaces::mix_of(p, …)`, e o
    /// `dono_mix(p, …)` do WGSL faz o mesmo). O laço da borda passava `g.point[i]` — o ponto do CENTRO
    /// do pixel —, e **num pixel de silhueta o centro pode FALHAR a peça**: ali a marcha não escreve
    /// `point[i]`, que fica no valor inicial `[0,0,0]`, e o do dispositivo fica pior ainda
    /// (`origem + direcção × t` com `t < 0`, isto é **atrás da câmara**).
    ///
    /// ⇒ o material vinha de um ponto que não está na superfície. Medido na cena `=36`, sobre os
    /// **`555`** pixels de silhueta cujo centro falha: o dispositivo pintava-os **`+56,3`** bytes de
    /// verde acima da CPU, porque o ponto bogus caía numa folha **emissiva** — e uma emissiva depois da
    /// exposição é BRANCA. *É o fio branco de um pixel que o dono fotografou, e ele só se vê nas peças
    /// escuras porque a cor que ele põe é sempre a mesma.*
    ///
    /// ⭐ **A cura é o idioma que este módulo já usa:** o [`edge_ground_factor`] empresta o factor dos
    /// vizinhos de cruz que FALHAM; aqui empresta-se o ponto do primeiro vizinho de cruz que **ACERTA**,
    /// na mesma ordem (esquerda, direita, cima, baixo — a do dispositivo). ⚠️ É a mesma aproximação
    /// declarada do resto da borda (a vista e o material do centro servem às quatro amostras), agora
    /// sobre um ponto que existe.
    ///
    /// ⚠️ **Sem vizinho que acerte, devolve o que havia** — um pixel de silhueta sem um único vizinho
    /// de cruz na peça é uma peça com menos de um pixel de largura, e ali não há material a emprestar.
    fn pixel_da_borda(g: &Gbuffer, i: usize) -> usize {
        if g.hit[i] {
            return i;
        }
        let (w, h) = (g.width as usize, g.height as usize);
        let (x, y) = (i % w, i / w);
        [
            (x > 0).then(|| i - 1),
            (x + 1 < w).then(|| i + 1),
            (y > 0).then(|| i - w),
            (y + 1 < h).then(|| i + w),
        ]
        .into_iter()
        .flatten()
        .find(|&j| g.hit[j])
        .unwrap_or(i)
    }

    for e in &g.edges {
        let i = e.pixel as usize;
        // ⚠️ **A vista do CENTRO do pixel serve às quatro amostras**: dentro de um pixel a direcção do
        // raio muda menos do que o passo de um byte move a luz, e a borda não guarda as posições.
        let v = view_direction(cam, &screen, i % w, i / w);
        // ⭐⭐ **O fundo de uma sub-amostra que falha é o fundo COM o chão** — ver
        // [`edge_ground_factor`]. Sem isto a silhueta de baixo pintava um fio do fundo limpo entre a
        // peça e a sombra de contacto, que é exactamente onde a sombra é mais escura.
        let fundo = shadowed_background(bg, edge_ground_factor(g, &fatores, i));
        // ⭐ **E a luz devolvida viaja ao lado**, pela mesma razão do ramo do fundo acima.
        let luz_do_fundo = edge_ground_bounce(g, &postas, i);
        // ⭐⭐⭐ **O PIXEL DE QUEM A BORDA PEDE EMPRESTADO** — ver [`pixel_da_borda`].
        let j = pixel_da_borda(g, i);
        // ⚠️ **E o MATERIAL do centro serve às quatro amostras**, pela mesma razão da vista: a borda
        // não guarda os pontos das sub-amostras. ⛔ Numa silhueta entre DUAS peças de cores
        // diferentes isto pinta a borda com a cor da que o centro apanhou — declarado, e é a mesma
        // aproximação que a direcção de vista já faz.
        let mut acc = [0.0f32; 4];
        let mut acc_luz = [0.0f32; 3];
        for k in 0..4 {
            let c = if e.hit[k] {
                let rgb = mixed_radiance(
                    surfaces,
                    PixelGeom {
                        // ⛔⛔⛔ **O ÍNDICE fica no do pixel que se PINTA; só o PONTO é emprestado.**
                        // A 1.ª redacção emprestou também a oclusão, a sombra e o ricochete (`i: j`)
                        // — e isso NÃO move o rebordo um byte (medido: `+0,0` das duas maneiras) e
                        // parte **nove** paridades entre os motores. *Uma cura maior do que a
                        // medição pede é uma regressão com um bom argumento ao lado.*
                        i,
                        p: g.point[j],
                        n: e.normal[k],
                        v,
                        // ⚠️ Vazio ⇒ `0` ⇒ o piso do GLSL dá o raio de `100`, que é «plano».
                        k: g.curvature.get(j).copied().unwrap_or(0.0),
                        // ⚠️ Vazio ⇒ `0` ⇒ tinta nenhuma, que é o que o estilo de fábrica quer.
                        k_estilo: g.curvature_style.get(j).copied().unwrap_or(0.0),
                        basis,
                    },
                    pixel_world,
                    light,
                    pres,
                );
                [rgb[0], rgb[1], rgb[2], 1.0]
            } else {
                // ⚠️ Uma sub-amostra que FALHA traz o fundo E a luz devolvida; uma que ACERTA traz
                // só a peça. *A média é das duas grandezas, cada uma na sua.*
                for (canal, &l) in acc_luz.iter_mut().zip(&luz_do_fundo) {
                    *canal += l * 0.25;
                }
                fundo
            };
            for j in 0..4 {
                acc[j] += c[j] * 0.25;
            }
        }
        write(&mut out[i * 4..i * 4 + 4], acc, acc_luz);
    }

    // ⭐⭐⭐ **O BRILHO, por último** (`docs/Render3d/12`, a `W7`) — ele lê o quadro em CENA-linear e
    // soma o halo por cima do que já está pintado. ⚠️ **Só corre se o artista o tiver ligado**
    // ([`Presentation::blooms`]), e é isso que faz o caminho de omissão custar zero.
    if pres.blooms() {
        let cena = crate::brilho::campo_de_cena(g, cam, surfaces, light, pres);
        let halo = ph2d_bloom::halo(&cena, w, h, &pres.bloom);
        crate::brilho::soma_halo(&mut out, &halo, pres);
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

/// ⭐⭐⭐ **A largura da fronteira entre dois materiais, em MUNDO** — a PORTA do
/// [`BOUNDARY_PIXELS`].
///
/// ⚠️ **Ela existe porque o pintor do dispositivo faz a mesma pergunta** (`ph2d_field_gpu::paint`),
/// e o factor acima é **medido**: escrito duas vezes, ele diverge no dia em que a varredura for
/// refeita e só um dos motores for actualizado. *Uma lei escrita em dois sítios ainda não é uma
/// lei — só uma PORTA é.*
#[must_use]
pub fn boundary_world(half_extent: f32, lado_px: u32) -> f32 {
    BOUNDARY_PIXELS * 2.0 * half_extent / lado_px.max(1) as f32
}

/// ⭐⭐ **A luz de um ponto, com a fronteira entre dois materiais SUAVIZADA** — ver
/// [`Surfaces::mix_of`].
///
/// ⚠️ **Sombreia DUAS vezes e mistura o resultado**, e não os materiais: é isso que uma
/// super-amostragem convergiria a dar, e misturar os *parâmetros* de dois OpenPBR não é misturar a
/// luz que eles devolvem — um metal e um dieléctrico a meio caminho não são um meio-metal.
///
/// ⛔ **E ela só paga o dobro onde há fronteira** (`t > 0`): fora dela, e numa peça de um material
/// só, é uma chamada e mais nada.
/// **A GEOMETRIA de um pixel** — o que o G-buffer sabe dele, mais a base que o liga ao mundo.
///
/// ⚠️ Ela nasceu em 14/09 porque a luz-objecto trouxe o **ponto** e a **base** para dentro do
/// sombreamento, e as duas funções passaram a levar oito argumentos. *Quatro grandezas que viajam
/// sempre juntas são uma coisa só.*
#[derive(Clone, Copy)]
pub(crate) struct PixelGeom {
    /// **Qual pixel** — a chave do canal de sombra. ⚠️ Uma borda usa o do CENTRO dela, que é a
    /// mesma aproximação que a direcção de vista e o material já fazem ali.
    pub(crate) i: usize,
    /// Onde a superfície está, no MUNDO.
    pub(crate) p: [f32; 3],
    /// A normal, em espaço de VISTA.
    pub(crate) n: [f32; 3],
    /// A direcção para o observador, em espaço de VISTA.
    pub(crate) v: [f32; 3],
    /// ⭐⭐⭐ **A CURVATURA deste ponto** (`|H|`) — `0` quando ninguém a pediu, e `0` é a leitura
    /// certa de *«não sei»* (ver [`crate::Gbuffer::curvature`]).
    pub(crate) k: f32,
    /// ⭐⭐⭐ **A CURVATURA À ESCALA DO ARTISTA** — com SINAL, e é esta que o estilo lê.
    ///
    /// ⚠️ **Campo NOMEADO e não derivado do `k`**, de propósito: as duas são medidas a distâncias
    /// diferentes (ver [`crate::Gbuffer::curvature_style`]), e um `k_estilo: k` escrito num
    /// chamador por conveniência seria a borda dura de volta, sem uma linha de lei ter mudado.
    /// *Esquecê-lo é erro de compilação nos dois sítios que constroem esta struct.*
    pub(crate) k_estilo: f32,
    pub(crate) basis: ViewBasis,
}

/// ⭐⭐ **A mesma mistura, em CENA-linear** — o que o HDR do brilho guarda.
///
/// ⛔⛔ **Ela NÃO é a irmã com o olhar por cima, e a diferença é DECLARADA:** o olhar não é linear,
/// logo `look(mix(a, b)) ≠ mix(look(a), look(b))`. O BYTE continua a sair da irmã (⇒ a imagem de
/// hoje fica **ao bit**) e o HDR sai desta; as duas só discordam na faixa de fronteira entre dois
/// materiais — a população que o [`BOUNDARY_PIXELS`] descreve —, e o halo é um sinal largo e macio,
/// onde essa diferença não é observável.
pub(crate) fn mixed_radiance_scene(
    surfaces: &Surfaces<'_>,
    geom: PixelGeom,
    pixel_world: f32,
    light: &Lighting<'_>,
    pres: &Presentation,
) -> [f32; 3] {
    let (a, b, t) = surfaces.mix_of(geom.p, pixel_world);
    let ca = radiance_scene(a, light, pres, geom);
    if t <= 0.0 {
        return ca;
    }
    let cb = radiance_scene(b, light, pres, geom);
    [0, 1, 2].map(|i| ca[i] + (cb[i] - ca[i]) * t)
}

fn mixed_radiance(
    surfaces: &Surfaces<'_>,
    geom: PixelGeom,
    pixel_world: f32,
    light: &Lighting<'_>,
    pres: &Presentation,
) -> [f32; 3] {
    let (a, b, t) = surfaces.mix_of(geom.p, pixel_world);
    let ca = radiance(a, light, pres, geom);
    if t <= 0.0 {
        return ca;
    }
    let cb = radiance(b, light, pres, geom);
    [0, 1, 2].map(|i| ca[i] + (cb[i] - ca[i]) * t)
}
