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
    edge_ground_bounce, edge_ground_factor, ground_factors, mais_luz, shadowed_background,
};
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

/// ⭐⭐⭐ **UMA LUZ QUE É UM OBJECTO DA CENA** — um ponto no MUNDO (ordem do dono, 2026-09-14).
///
/// # ⚠️ Porque ela é um tipo À PARTE da [`Lamp`], e não uma variante dela
///
/// As duas respondem a perguntas diferentes **por pixel**: a [`Lamp`] é ancorada no ECRÃ e a
/// direcção dela é a mesma em toda a imagem (é o estúdio — ela não se mexe quando a câmera roda);
/// esta é ancorada no MUNDO, e a direcção e a distância mudam de pixel para pixel. ⇒ um `enum`
/// poria um ramo dentro do laço mais quente do sombreamento para distinguir duas listas que o
/// chamador já tem separadas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointLamp {
    /// Onde ela está, no **MUNDO** — a pose da entidade, propagada.
    pub world: [f32; 3],
    /// A radiância que ela entrega a **UMA unidade** de distância.
    ///
    /// ⚠️ A queda é `1/r²`, logo este é o numerador. A unidade é a do rig da casa (`cor ×
    /// intensidade × π`), medida a uma unidade — ver o doc do `ph2d_field_ecs::FieldLight`.
    pub radiance_at_one: [f32; 3],
}

/// **O PISO DA DISTÂNCIA** de uma [`PointLamp`], em unidades de mundo — a remoção de uma
/// singularidade.
///
/// Um ponto matemático diverge quando a superfície o alcança, e `1/0` entra no sombreamento como
/// `inf`. Abaixo deste piso a peça já está saturada **há muito** — com força `1` e o material de
/// omissão uma difusa satura a `r ≈ 0,9`, isto é a `18×` este raio —, logo o que ele corta é a
/// divisão por zero e mais nada. *É o que uma luz ESFÉRICA de raio `0,05` faria.*
///
/// # ⚠️⚠️ O VALOR dele não é observável, e a lei que o gate prende é a OUTRA metade
///
/// Uma prova de mutação pô-lo a `0` e **sobreviveu**: o byte satura, logo `1/0,0025` e `1/0` pintam
/// os mesmos `255`. *O que era observável era o defeito ao lado dele* — com a luz exactamente sobre
/// o ponto, `d` é o vector **ZERO**, a direcção normalizada sai `[0,0,0]`, o `N·L` dá `0` e o pixel
/// fica **PRETO**. ⇒ abaixo do piso a direcção passa a ser a **NORMAL**, e é essa mutação que sangra
/// (`a_light_falls_off_with_the_square_of_the_distance`).
///
/// *Um piso que protege a aritmética e deixa a geometria degenerada resolve metade de um defeito, e
/// a metade que fica tem o mesmo sintoma.*
pub const POINT_LAMP_MIN_DISTANCE: f32 = 0.05;

/// O quadrado do [`POINT_LAMP_MIN_DISTANCE`] — o piso, na grandeza em que ele é comparado.
pub(crate) const PISO_DA_LAMPADA: f32 = POINT_LAMP_MIN_DISTANCE * POINT_LAMP_MIN_DISTANCE;

/// ⭐⭐ **A radiância que uma [`PointLamp`] ENTREGA a um ponto** — a queda `1/r²`, o piso e a
/// visibilidade, numa porta.
///
/// ⚠️ **É a parte da lei que NÃO depende de referencial**, e é por isso que é ela que se partilha: a
/// DIRECÇÃO para a luz tem de sair no espaço de quem pergunta (o sombreador quer-a em VISTA, o
/// [`crate::bounce`] em MUNDO) e o braço degenerado devolve *«a normal»*, que é uma resposta
/// diferente em cada um. *Partilhar o que é comum e nomear o que não é vale mais que uma porta que
/// converte duas vezes para caber nos dois.*
///
/// ⚠️ A visibilidade entra **aqui, na luz que chega** — nunca no `N·L` e nunca no resultado. Ver o
/// comentário no laço do [`radiance`] para a razão física.
pub(crate) fn chega_da_lampada(lamp: &PointLamp, dist2: f32, visivel: f32) -> [f32; 3] {
    lamp.radiance_at_one
        .map(|c| c * visivel / dist2.max(PISO_DA_LAMPADA))
}

/// A luz de uma cena: as lâmpadas de estúdio, as luzes-objecto e o céu.
pub struct Lighting<'a> {
    /// Ancoradas no ECRÃ — o estúdio.
    pub lamps: &'a [Lamp],
    /// ⭐ Ancoradas no MUNDO — os objectos da cena. `&[]` é o caminho de sempre, ao bit.
    pub points: &'a [PointLamp],
    pub sky: &'a (dyn Environment + Sync),
    /// ⭐⭐⭐ **Quanto de cada [`PointLamp`] CHEGA a cada pixel** — ver [`crate::Shadows`].
    ///
    /// ⚠️ **`None` é o caminho de sempre, ao bit**: sem o passe, toda lâmpada chega inteira a todo
    /// lado, que é exactamente o que o produto fazia até 2026-09-14. ⛔ As luzes de ECRÃ
    /// ([`Lighting::lamps`]) NÃO têm sombra e não é omissão: elas estão ancoradas no ecrã, logo
    /// giram com a câmera — uma sombra que gira com o olhar não pousa nada, ensina o contrário.
    pub shadows: Option<&'a crate::Shadows>,
}

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
fn radiance(surface: &Surface, light: &Lighting<'_>, look: Look, geom: PixelGeom) -> [f32; 3] {
    let PixelGeom {
        i: _,
        p,
        n,
        v,
        k,
        basis,
    } = geom;
    // ⭐⭐⭐ **O MATERIAL NESTE PONTO** — a curvatura é a única entrada geométrica que a lei do
    // OpenPBR pede, e ela chega do CAMPO (`crate::curvatura`). ⚠️ Com a subsuperfície maciça
    // desligada (a omissão) ninguém a lê, e esta linha é uma cópia de 15 floats que não muda um bit.
    let surface = &surface.at_curvature(k);
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
    // ⭐ **Uma vez por quadro** — ver [`ViewBasis`].
    let basis = ViewBasis::of(cam);
    // ⭐ **A largura da transição entre dois materiais, em MUNDO** — a única coisa que o
    // [`Surfaces::mix_of`] não pode adivinhar. Ver [`ph2d_field_eval::owners::Owners::mix_at`].
    #[allow(clippy::cast_possible_truncation)]
    let pixel_world = boundary_world(cam.half_extent, w.min(h) as u32);
    let write = |px: &mut [u8], c: [f32; 4]| {
        px[0] = ph2d_color::srgb::linear_to_srgb_byte(c[0]);
        px[1] = ph2d_color::srgb::linear_to_srgb_byte(c[1]);
        px[2] = ph2d_color::srgb::linear_to_srgb_byte(c[2]);
        px[3] = (c[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
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
                        basis,
                    },
                    pixel_world,
                    light,
                    look,
                );
                write(px, [c[0], c[1], c[2], 1.0]);
            } else {
                // ⭐⭐⭐ **A LUZ QUE A PEÇA PÕE NO CHÃO** — somada em pré-multiplicado e com alfa
                // ZERO, que é o que um compositor lê como luz acrescentada. ⛔ Ela NÃO pode
                // multiplicar o fundo: o do modelador é transparente, e a wave inteira sairia num
                // pixel que não muda um bit (ver o cabeçalho do [`crate::ground_shade`]).
                let posta = postas.get(i).copied().unwrap_or([0.0; 3]);
                match fatores.get(i) {
                    // ⭐ O chão tapado: o fundo com a sombra por cima — ver [`shadowed_background`].
                    Some(&f) if f < 1.0 => write(px, mais_luz(shadowed_background(bg, f), posta)),
                    // ⚠️ Copiado, e não passado pela conversão — a mesma cerca do `shade`. É também
                    // o chão onde nada tapa: a razão é EXACTAMENTE `1`, e os bytes são os de sempre.
                    // ⭐ **Com luz devolvida ele deixa de poder ser copiado** — ela é o que há para
                    // mostrar num pixel cujo fundo é preto transparente.
                    _ if posta != [0.0; 3] => write(px, mais_luz(bg, posta)),
                    _ => px.copy_from_slice(&background),
                }
            }
        }
    });

    for e in &g.edges {
        let i = e.pixel as usize;
        // ⚠️ **A vista do CENTRO do pixel serve às quatro amostras**: dentro de um pixel a direcção do
        // raio muda menos do que o passo de um byte move a luz, e a borda não guarda as posições.
        let v = view_direction(cam, &screen, i % w, i / w);
        // ⭐⭐ **O fundo de uma sub-amostra que falha é o fundo COM o chão** — ver
        // [`edge_ground_factor`]. Sem isto a silhueta de baixo pintava um fio do fundo limpo entre a
        // peça e a sombra de contacto, que é exactamente onde a sombra é mais escura.
        let fundo = mais_luz(
            shadowed_background(bg, edge_ground_factor(g, &fatores, i)),
            edge_ground_bounce(g, &postas, i),
        );
        // ⚠️ **E o MATERIAL do centro serve às quatro amostras**, pela mesma razão da vista: a borda
        // não guarda os pontos das sub-amostras. ⛔ Numa silhueta entre DUAS peças de cores
        // diferentes isto pinta a borda com a cor da que o centro apanhou — declarado, e é a mesma
        // aproximação que a direcção de vista já faz.
        let mut acc = [0.0f32; 4];
        for k in 0..4 {
            let c = if e.hit[k] {
                let rgb = mixed_radiance(
                    surfaces,
                    PixelGeom {
                        i,
                        p: g.point[i],
                        n: e.normal[k],
                        v,
                        // ⚠️ Vazio ⇒ `0` ⇒ o piso do GLSL dá o raio de `100`, que é «plano».
                        k: g.curvature.get(i).copied().unwrap_or(0.0),
                        basis,
                    },
                    pixel_world,
                    light,
                    look,
                );
                [rgb[0], rgb[1], rgb[2], 1.0]
            } else {
                fundo
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
struct PixelGeom {
    /// **Qual pixel** — a chave do canal de sombra. ⚠️ Uma borda usa o do CENTRO dela, que é a
    /// mesma aproximação que a direcção de vista e o material já fazem ali.
    i: usize,
    /// Onde a superfície está, no MUNDO.
    p: [f32; 3],
    /// A normal, em espaço de VISTA.
    n: [f32; 3],
    /// A direcção para o observador, em espaço de VISTA.
    v: [f32; 3],
    /// ⭐⭐⭐ **A CURVATURA deste ponto** (`|H|`) — `0` quando ninguém a pediu, e `0` é a leitura
    /// certa de *«não sei»* (ver [`crate::Gbuffer::curvature`]).
    k: f32,
    basis: ViewBasis,
}

fn mixed_radiance(
    surfaces: &Surfaces<'_>,
    geom: PixelGeom,
    pixel_world: f32,
    light: &Lighting<'_>,
    look: Look,
) -> [f32; 3] {
    let (a, b, t) = surfaces.mix_of(geom.p, pixel_world);
    let ca = radiance(a, light, look, geom);
    if t <= 0.0 {
        return ca;
    }
    let cb = radiance(b, light, look, geom);
    [0, 1, 2].map(|i| ca[i] + (cb[i] - ca[i]) * t)
}
