//! ⭐⭐⭐ **A MISTURA EM GRUPO das formas e dos quads do Motion** (doc 118, ordem do dono de
//! 2026-09-22: *«todas as opções possíveis sem excluir nenhuma, com seletor de modo no nó»* e
//! *«Igual, como nas formas»*).
//!
//! O [`super::encode`] parte a lista em CORRIDAS de linhas consecutivas com a mesma chave de
//! mistura; aqui cada corrida recebe o arranjo de camadas do alcance dela. As três respostas estão
//! em forma fechada no doc 118 §1, e o gate de pixel (`motion_shape_mistura_gpu_tests`) mede-as
//! contra a placa.

use ph2d_eval_motion::{BlendWith, VectorInstance};
use ph2d_vector::{Affine, VectorScene};

use super::{VecPathStore, instance_pose};

/// **A chave de uma corrida** — `None` para toda linha que desenha sem camada.
///
/// ⚠️ **O `sink` entra na chave**: dois sinks VIZINHOS com a mesma mistura são dois grupos, e
/// fundi-los faria o `Copies` de um misturar-se com as cópias do outro — o artista pôs dois nós,
/// e *um grupo é um NÓ, não um modo*.
pub(super) fn chave_de_mistura(
    m: &ph2d_eval_motion::MisturaDoSink,
) -> Option<(u8, BlendWith, u32)> {
    // ⚠️ A pergunta é a do `MisturaDoSink` — a MESMA que manda o sink inteiro ao Vello.
    m.tem_camada().then_some((m.blend, m.com, m.sink))
}

/// **O tag de mistura do sink traduzido para a PLACA** — `None` quando não há camada a fazer.
///
/// ⭐ **Uma lei só, a do vector:** o tag passa ao vocabulário do documento e a tradução é a
/// [`ph2d_vec_render::blend::vello_blend`], a mesma que uma forma desenhada à mão usa. É isso
/// que faz imagens e formas misturarem-se **no mesmo espaço** (ordem do dono: *«Igual, como nas
/// formas»*).
///
/// ⛔ **`Subtract` fica SEM camada, e é DECLARADO:** o W3C não o tem e o shader do Vello também
/// não (a mesma fronteira que o doc do `vello_blend` escreve para os três do Photoshop). Os tags
/// `0` (Mix) e `5` (alfa pré-multiplicada) não são mistura nenhuma — são o desenho de sempre.
pub(crate) fn mistura_vello(tag: u8) -> Option<ph2d_vector::VelloBlend> {
    let modo = match tag {
        1 => ph2d_vec_scene::BlendMode::Add,
        3 => ph2d_vec_scene::BlendMode::Multiply,
        4 => ph2d_vec_scene::BlendMode::Screen,
        _ => return None,
    };
    ph2d_vec_render::blend::vello_blend(modo)
}

/// **O rectângulo de um GRUPO** — a janela, ou (sem ela) um rectângulo que cobre qualquer
/// alvo real.
///
/// ⚠️ A camada de grupo não pode recortar à caixa das cópias sem as tesselar DUAS vezes; a janela
/// é o alvo de render inteiro, logo recortar a ela não pode comer nada que se veja (a mesma
/// razão do recorte por câmara).
fn rect_do_grupo(janela: Option<ph2d_vector::Rect>) -> ph2d_vector::Rect {
    janela.unwrap_or(ph2d_vector::Rect::new(-1.0e6, -1.0e6, 1.0e6, 1.0e6))
}

/// **Uma corrida da lista**, com o arranjo de camadas do alcance dela (doc 118 §1).
///
/// - **`Everything`** — cada cópia na SUA camada `m`: ela mistura-se com TUDO o que já está
///   pintado, as cópias anteriores e o cenário.
/// - **`Copies`** — um grupo ISOLADO (`Normal`) com cada cópia na camada `m` dentro dele: elas
///   misturam-se só entre si, e o grupo pousa sobre o cenário normalmente.
/// - **`Scene`** — as cópias juntam-se em `Normal` dentro de um grupo, e o GRUPO pousa em `m`.
#[allow(clippy::too_many_arguments)]
pub(super) fn desenha_corrida(
    corrida: &[VectorInstance],
    chave: Option<(u8, BlendWith, u32)>,
    store: &VecPathStore,
    art: &mut dyn FnMut(u32, [f32; 4]) -> Option<crate::motion_leaf_images::Art>,
    cam: Affine,
    janela: Option<ph2d_vector::Rect>,
    filtro: ph2d_render::ImageFilterMode,
    scene: &mut VectorScene,
) {
    let Some((tag, com, _)) = chave else {
        return desenha_linhas(corrida, None, store, art, cam, janela, filtro, scene);
    };
    let Some(m) = mistura_vello(tag) else {
        return desenha_linhas(corrida, None, store, art, cam, janela, filtro, scene);
    };
    let normal =
        ph2d_vector::VelloBlend::new(ph2d_vector::Mix::Normal, ph2d_vector::Compose::SrcOver);
    // ⭐ **Os dois alcances que se misturam com o CENÁRIO pedem-no por baixo da cena** (W2): as
    // sprites vivem noutra textura, e sem a marca a camada misturava-se com o vazio. O `Copies`
    // pousa em `Normal` e não precisa dele — ⛔ pedi-lo ali custaria a cópia do mundo por nada.
    if com != BlendWith::Copies {
        scene.pede_o_mundo_por_baixo();
    }
    match com {
        BlendWith::Everything => {
            desenha_linhas(corrida, Some(m), store, art, cam, janela, filtro, scene)
        }
        BlendWith::Copies => {
            scene.push_object_layer(&rect_do_grupo(janela), normal, 1.0);
            desenha_linhas(corrida, Some(m), store, art, cam, janela, filtro, scene);
            scene.pop_layer();
        }
        BlendWith::Scene => {
            scene.push_object_layer(&rect_do_grupo(janela), m, 1.0);
            desenha_linhas(corrida, None, store, art, cam, janela, filtro, scene);
            scene.pop_layer();
        }
    }
}

/// **As linhas de uma corrida, na ordem** — `por_copia = Some(m)` põe cada cópia na sua camada.
///
/// ⭐⭐⭐ **A TERCEIRA MÉDIA, e a ORDEM é o ponto** (report do Enio, 2026-08-30, três vezes:
/// *"Leaves in front ainda não funciona quando a folha é IMG"*). Uma instância com textura é um
/// **quad texturado desenhado NESTA cena** — a mesma em que a planta vive —, e por isso ela pode
/// ficar por cima de uma forma. O que decide é a ORDEM das linhas, como em toda esta lista.
///
/// ⚠️ **As formas continuam a ir em LOTE** (o cache de tesselação por quadro que o
/// `draw_shared_instances` guarda): só uma imagem no meio o quebra, e o lote recomeça a
/// seguir. Sem imagem nenhuma, isto é **uma** chamada com tudo — byte-idêntico ao que havia.
#[allow(clippy::too_many_arguments)]
fn desenha_linhas(
    linhas: &[VectorInstance],
    por_copia: Option<ph2d_vector::VelloBlend>,
    store: &VecPathStore,
    art: &mut dyn FnMut(u32, [f32; 4]) -> Option<crate::motion_leaf_images::Art>,
    cam: Affine,
    janela: Option<ph2d_vector::Rect>,
    filtro: ph2d_render::ImageFilterMode,
    scene: &mut VectorScene,
) {
    let mut lote: Vec<(u32, Affine, [f32; 4])> = Vec::new();
    let despeja = |lote: &mut Vec<(u32, Affine, [f32; 4])>, scene: &mut VectorScene| {
        if lote.is_empty() {
            return;
        }
        match por_copia {
            None => ph2d_vec_render::draw_shared_instances(
                lote.drain(..),
                |h| store.get(h),
                janela,
                scene,
            ),
            Some(m) => ph2d_vec_render::draw_shared_instances_em_camadas(
                lote.drain(..),
                |h| store.get(h),
                janela,
                m,
                scene,
            ),
        }
    };
    for inst in linhas {
        if inst.geometry_id > 0 {
            lote.push((inst.geometry_id, instance_pose(inst, cam), inst.tint));
            continue;
        }
        // ⚠️ **A arte pode não resolver** (a textura já não existe, o formato é recusado) — e aí
        // a linha simplesmente não desenha, que é o mesmo que a membrana faz com um nome que
        // ninguém publicou. ⛔ Desenhar um rectângulo de substituição seria inventar arte.
        let Some((w, h, rgba)) = art(inst.texture_id, inst.atlas_uv) else {
            continue;
        };
        despeja(&mut lote, scene);
        draw_quad(inst, &rgba, w, h, cam, por_copia, filtro, scene);
    }
    despeja(&mut lote, scene);
}

/// **Um quad texturado na cena vectorial** — a pose é a MESMA função das formas
/// ([`instance_pose`]), e é isso que mantém as duas médias alinhadas ao pixel.
///
/// ⚠️ **A imagem desenha-se em coordenadas de PIXEL e a pose leva-a ao mundo**, então o
/// transform compõe `pose · escala(1/w, 1/h) · translação(−w/2, −h/2)`: o quad de uma instância
/// é `[−½, ½]²` no local, como o da sprite.
///
/// ⚠️ **`camada = Some(m)` embrulha o quad numa camada recortada à caixa DELE** — a mesma lei
/// da porta das formas (`draw_shared_instances_em_camadas`): uma camada do tamanho da janela por
/// cópia misturaria o ecrã inteiro uma vez por cópia.
#[allow(clippy::too_many_arguments)]
fn draw_quad(
    inst: &VectorInstance,
    rgba: &std::sync::Arc<Vec<u8>>,
    w: u32,
    h: u32,
    cam: Affine,
    camada: Option<ph2d_vector::VelloBlend>,
    filtro: ph2d_render::ImageFilterMode,
    scene: &mut VectorScene,
) {
    if w == 0 || h == 0 {
        return;
    }
    let t = instance_pose(inst, cam)
        * Affine::scale_non_uniform(1.0 / f64::from(w), 1.0 / f64::from(h))
        * Affine::translate((-f64::from(w) / 2.0, -f64::from(h) / 2.0));
    let tinta = Tinta::de(inst.tint);
    if tinta == Tinta::Apagada {
        return;
    }
    if let Some(m) = camada {
        scene.push_object_layer(&caixa_do_quad(t, w, h), m, 1.0);
    }
    let quad = ph2d_vector::Rect::new(0.0, 0.0, f64::from(w), f64::from(h));
    tinta.abre(quad, t, scene);
    let qualidade = qualidade_da_imagem(inst.sampling, filtro);
    // ⚠️ **A alfa da FONTE decide a porta**, como no lowering de sprites: uma textura já
    // premultiplicada entra pela porta que NÃO volta a multiplicar.
    if inst.premultiplied > 0.5 {
        scene.draw_image_rgba_premultiplied_transformed(rgba, w, h, t, qualidade);
    } else {
        scene.draw_image_rgba_transformed(rgba, w, h, t, qualidade);
    }
    tinta.fecha(scene);
    if camada.is_some() {
        scene.pop_layer();
    }
}

/// ⭐⭐ **O FILTRO de um quad de imagem na cena vectorial** (doc 118 §8) — a MESMA pergunta que o
/// sampler da sprite responde, feita ao pincel de imagem do Vello.
///
/// ⚠️ **A tag `0` HERDA o projecto, como na sprite** (`fase_extract_inputs`: `PixelArt → Nearest`,
/// `Smooth → Linear`). Até 2026-09-23 esta rota desenhava `Medium` para TUDO, logo num projecto em
/// `PixelArt` uma imagem era nítida como sprite e borrada no quadro em que entrava na cena
/// vectorial — *a MESMA folha a mudar de filtro por mudar de média*. ⭐ O projecto de fábrica é
/// `Smooth` ⇒ `Medium`, que é o que já se desenhava: o caminho de omissão fica **byte-idêntico**.
///
/// ⛔ **O resto da tag NÃO tem destino, e é DECLARADO** (`StyleReach::IMAGE_ON_VECTOR`): o pincel
/// de imagem do Vello escolhe só a lei de AMPLIAÇÃO (`Low` = ponto, `Medium` = bilinear) — ele não
/// tem cadeia de mips nem anisotropia, logo as tags `3..=6` reduzem-se à metade de ampliação delas
/// ([`ph2d_render::filter_tag_magnifies_by_point`], a MESMA função que monta o sampler). ⛔ E o
/// `High` (bicúbico) não entra: nenhuma tag o pede, e a sprite nunca o faz.
#[must_use]
pub(crate) fn qualidade_da_imagem(
    sampling: u32,
    projeto: ph2d_render::ImageFilterMode,
) -> ph2d_vector::ImageQuality {
    let (tag, _repeat) = ph2d_render::RenderInstance::unpack_sampling(sampling);
    let ponto = if tag == 0 {
        projeto == ph2d_render::ImageFilterMode::PixelArt
    } else {
        ph2d_render::filter_tag_magnifies_by_point(tag)
    };
    if ponto {
        ph2d_vector::ImageQuality::Low
    } else {
        ph2d_vector::ImageQuality::Medium
    }
}

/// ⭐⭐⭐ **A TINTA de um quad de imagem na cena vectorial** (doc 118 §7 W6) — a MESMA conta que o
/// `sprite.wgsl` faz: `rgb · T.rgb` e `α · T.a`, com a saída pré-multiplicada `(rgb·α·T.rgb·T.a,
/// α·T.a)`.
///
/// ⛔ **O Vello não tem «imagem × cor».** A forma ingénua — pintar a cor por cima com `Multiply` e
/// compor `SrcAtop` — **vaza a cor da tinta nas bordas meio transparentes**: a mistura do W3C faz
/// `cs' = (1 − αb)·cs + αb·B(cb, cs)`, e com o fundo parcial o primeiro termo é a tinta pura
/// (erro até `¼·T` a `α = ½` sobre um texel escuro).
///
/// ⭐ **A ordem inversa é EXACTA:** a COR vai por baixo, opaca, recortada ao quad, e a IMAGEM entra
/// por cima numa camada `Multiply` + `SrcIn`. Com o fundo opaco (`αb = 1`) o `cs'` é `T·cs` sem
/// resto, e o `SrcIn` guarda `αs·αb = αs` — a cor só existe onde a imagem existe. A alfa da tinta é
/// a alfa da camada de fora, que multiplica o vector pré-multiplicado inteiro.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Tinta {
    /// Branca e opaca — o desenho de sempre, **sem camada nenhuma** (byte-idêntico).
    Neutra,
    /// Só a alfa muda — uma camada, sem a cor por baixo.
    SoAlfa(f32),
    /// Cor (e alfa) — as duas camadas.
    Cor([f32; 4]),
    /// `α ≤ 0` — nada a desenhar (o sprite também não pinta nada).
    Apagada,
}

impl Tinta {
    fn de(t: [f32; 4]) -> Self {
        if t[3] <= 0.0 {
            Self::Apagada
        } else if t == [1.0, 1.0, 1.0, 1.0] {
            Self::Neutra
        } else if t[0] == 1.0 && t[1] == 1.0 && t[2] == 1.0 {
            Self::SoAlfa(t[3])
        } else {
            Self::Cor(t)
        }
    }

    /// Abre as camadas da tinta à volta do quad `quad` (em píxeis da imagem) sob o afim `t`.
    fn abre(self, quad: ph2d_vector::Rect, t: Affine, scene: &mut VectorScene) {
        let normal =
            ph2d_vector::VelloBlend::new(ph2d_vector::Mix::Normal, ph2d_vector::Compose::SrcOver);
        match self {
            Self::Neutra | Self::Apagada => {}
            Self::SoAlfa(a) => scene.push_layer_shape(normal, a.min(1.0), t, &quad),
            Self::Cor([r, g, b, a]) => {
                scene.push_layer_shape(normal, a.min(1.0), t, &quad);
                let cor = ph2d_vector::Color::new([
                    r.clamp(0.0, 1.0),
                    g.clamp(0.0, 1.0),
                    b.clamp(0.0, 1.0),
                    1.0,
                ]);
                scene.fill_path(
                    &ph2d_vector::Shape::to_path(&quad, 0.1),
                    &ph2d_vector::Brush::Solid(cor),
                    t,
                );
                let multiplica = ph2d_vector::VelloBlend::new(
                    ph2d_vector::Mix::Multiply,
                    ph2d_vector::Compose::SrcIn,
                );
                scene.push_layer_shape(multiplica, 1.0, t, &quad);
            }
        }
    }

    /// Fecha o que [`Self::abre`] abriu.
    fn fecha(self, scene: &mut VectorScene) {
        match self {
            Self::Neutra | Self::Apagada => {}
            Self::SoAlfa(_) => scene.pop_layer(),
            Self::Cor(_) => {
                scene.pop_layer();
                scene.pop_layer();
            }
        }
    }
}

/// **A caixa de ecrã de um quad** — os quatro cantos da imagem pelo afim dela, com um pixel de
/// folga (o filtro `Medium` amostra meio pixel para fora da borda).
pub(crate) fn caixa_do_quad(t: Affine, w: u32, h: u32) -> ph2d_vector::Rect {
    let (w, h) = (f64::from(w), f64::from(h));
    let cantos =
        [(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)].map(|(x, y)| t * ph2d_vector::Point::new(x, y));
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for p in cantos {
        x0 = x0.min(p.x);
        y0 = y0.min(p.y);
        x1 = x1.max(p.x);
        y1 = y1.max(p.y);
    }
    ph2d_vector::Rect::new(x0 - 1.0, y0 - 1.0, x1 + 1.0, y1 + 1.0)
}

#[cfg(test)]
#[path = "motion_shape_mistura_gpu_tests.rs"]
mod gpu_tests;

#[cfg(test)]
#[path = "motion_shape_mistura_filtro_tests.rs"]
mod filtro_tests;
