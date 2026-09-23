//! ⭐⭐⭐⭐ **A CACHE DO CAMPO DO CHÃO** — assado uma vez por cena-luz-e-precisão, não por quadro.
//!
//! A assadura custa **`+4,98 ms` por quadro assente** e o campo **não depende de para onde a câmera
//! olha** — que é o gesto que a paga. Medido a orbitar, numa janela de calma real: `1,25×` a
//! `1,48×` no quadro assente, e até `18,1 ms` numa peça complexa.
//!
//! ⚠️ **Ele saiu do [`super::gpu_frame`] por um TECTO DE LOC** (2026-09-23, `702` contra `700`), e a
//! fronteira que o tecto forçou é a certa: *montar um pedido de quadro e guardar um campo entre
//! quadros são duas responsabilidades.*

use super::*;
use std::sync::Mutex;

/// ⭐⭐⭐⭐ **A CHAVE DO CAMPO DO CHÃO — tudo o que a assadura lê, menos a ORIENTAÇÃO da câmera.**
///
/// A assadura custa **`+4,98 ms` por quadro assente** ([`09` §6](../../../docs/Render3d/09_a_cor_que_a_peca_devolve_ao_chao.md))
/// e o campo **não depende de para onde a câmera olha** — o que é o gesto que a paga, porque orbitar
/// reassava-o a cada quadro. ⛔ **Mas ele depende da TOLERÂNCIA DE ACERTO**, e é isso que a primeira
/// redacção desta chave não sabia: ver os dois gates em
/// `ph2d-field-render/src/tests/chao_ricochete.rs` — orbitar move `6e-9` (`3` ULP, `50 000×` abaixo
/// de um byte de saída) e a tolerância move até **`0,91` de um byte** abaixo do clamp, que um zoom
/// apertado numa janela normal alcança.
#[derive(PartialEq)]
pub(crate) struct ChaveDoChao {
    /// A peça, pelo TEXTO da fita dela — a mesma lei de identidade que o cache de pipelines usa.
    pub(super) fita: String,
    /// E pelas CONSTANTES, porque duas peças com a mesma forma e números diferentes dão a mesma
    /// fita e campos diferentes (é exactamente o que arrastar um raio faz).
    pub(super) consts: Vec<f32>,
    pub(super) altura: f32,
    pub(super) lampadas: Vec<ph2d_field_render::PointLamp>,
    pub(super) materiais: Vec<ph2d_material::Surface>,
    /// ⭐⭐⭐⭐ **A PRECISÃO INTEIRA — o único eixo da câmera que chega ao campo, e ele chega.**
    ///
    /// ⛔⛔ **Até 2026-09-23 esta linha era `hit: f32`, e era METADE.** A assadura lê `sharp.hit`
    /// (o critério de acerto da marcha) **e** `sharp.normal` (o estêncil, em
    /// `ph2d_field_render::march`, dentro do `normals_into`) — e a chave levava só o primeiro.
    ///
    /// ⚠️⚠️ **Era seguro por uma relação NÃO ESCRITA entre duas constantes de outra crate:** o
    /// `normal` só se solta do clamp com `lado_px > 10 000 × half_extent` e o `hit` com
    /// `> 2 500 ×`, logo o `normal` nunca se movia sem o `hit` se mover. *Subir o `NORMAL_EPS` de
    /// `1e-4` para `1e-3` inverte a ordem, e passa a existir um regime em que o `normal` muda, o
    /// `hit` não, a chave não muda e a cache devolve um campo assado com outro `ε` — em silêncio.*
    ///
    /// ⇒ a chave leva a struct INTEIRA, e um campo novo na
    /// [`ph2d_field_render::Sharpness`] entra nela **por construção**. *Uma auditoria achou isto; a
    /// cura não é um gate sobre as duas constantes, é a chave deixar de poder ficar incompleta.*
    pub(super) sharp: ph2d_field_render::Sharpness,
    /// A grelha e as direcções, que são constantes hoje e entram porque uma delas subir sem a
    /// chave saber devolveria o campo da grelha antiga.
    pub(super) grelha: usize,
    pub(super) direccoes: u32,
}

/// ⭐⭐⭐ **O campo do chão, assado UMA vez por cena-luz-e-tolerância.**
///
/// ⚠️ **Uma entrada basta, e a razão é o gesto:** quem paga a assadura é orbitar, e orbitar não
/// mexe na chave. Trocar a luz ou a peça é uma falta, reassa, e substitui — *um mapa aqui guardaria
/// campos de peças que o artista já deixou.*
///
/// ⚠️ **Ele mora no módulo, ao lado do [`SharedTracer`]**, e pela mesma razão que ele: é estado
/// caro que atravessa quadros, e o `AppGfx` é partilhado por outra linha.
static CAMPO_DO_CHAO: Mutex<Option<(ChaveDoChao, ph2d_field_render::ground_bounce::GroundBounce)>> =
    Mutex::new(None);

/// ⭐⭐⭐⭐ **O CAMPO DO CHÃO, ASSADO UMA VEZ POR CENA-LUZ-E-TOLERÂNCIA** — ver [`ChaveDoChao`].
///
/// A assadura custa **`+4,98 ms` por quadro assente** e o campo **não depende de para onde a câmera
/// olha** — que é o gesto que a paga. ⇒ orbitar uma peça deixa de a reassar.
///
/// # ⛔⛔ As DUAS cercas, e as duas são conservadoras de propósito
///
/// 1. **Uma peça com ESCULTURA não é cacheada.** A fita da peça **não inlina** a escultura (o
///    prefixo `escultura_` dela é uma ligação, não texto), logo *duas esculturas diferentes dão a
///    MESMA fita* e a chave não as distingue. Uma cache que se contentasse com a fita entregaria o
///    campo da escultura anterior, **em silêncio**.
/// 2. **Uma peça com LEI DO DONO não é cacheada.** A lei é função das FOLHAS e da tolerância, e a
///    chave só conhece a fita COMBINADA — duas decomposições diferentes com a mesma fita são
///    improváveis e não impossíveis. ⭐ E desde a cura dos materiais (`Table::build`) a lei do dono
///    só existe com materiais **distintos**, logo esta cerca quase nunca morde no caminho de quem
///    está a modelar.
///
/// *As duas custam uma assadura a mais numa peça que as tenha, e a alternativa custa a cor errada.*
// A cena, o registo, a câmera, o chão, os materiais, as luzes, a fita, o campo, a tela e a sonda —
// dez coisas independentes, e uma struct só as renomearia (a mesma nota do [`pinta`] ao lado).
#[allow(clippy::too_many_arguments)]
pub(super) fn campo_do_chao(
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &ph2d_field_render::Orbit,
    chao: ph2d_field_render::Ground,
    surfaces: &ph2d_field_render::Surfaces<'_>,
    points: &[ph2d_field_render::PointLamp],
    fita: &ph2d_field_eval::wgsl::TapeWgsl,
    campo: &ph2d_field_eval::device::DeviceField,
    w: u32,
    h: u32,
    sonda: Sonda,
) -> ph2d_field_render::ground_bounce::GroundBounce {
    let assa = || {
        ph2d_field_render::ground_bounce::bake_ground_bounce(
            doc,
            reg,
            cam,
            chao,
            surfaces,
            points,
            ph2d_field_render::ground_bounce::GROUND_BOUNCE_GRID,
            ph2d_field_render::ground_bounce::GROUND_BOUNCE_DIRS,
            w.min(h) as usize,
        )
    };
    if !sonda.chao_em_cache || !campo.sculpts().is_empty() || surfaces.owners.is_some() {
        return assa();
    }
    let chave = ChaveDoChao {
        fita: fita.source.clone(),
        consts: fita.consts.clone(),
        altura: chao.height,
        lampadas: points.to_vec(),
        materiais: surfaces.all.to_vec(),
        sharp: ph2d_field_render::Sharpness::for_frame(cam.half_extent, w.min(h) as usize),
        grelha: ph2d_field_render::ground_bounce::GROUND_BOUNCE_GRID,
        direccoes: ph2d_field_render::ground_bounce::GROUND_BOUNCE_DIRS,
    };
    let Ok(mut guarda) = CAMPO_DO_CHAO.lock() else {
        // ⚠️ Um `Mutex` envenenado não é motivo para entregar a cor errada: assa-se.
        return assa();
    };
    if let Some((antiga, campo_guardado)) = guarda.as_ref()
        && *antiga == chave
    {
        ACERTOS_DO_CHAO.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        return campo_guardado.clone();
    }
    let novo = assa();
    *guarda = Some((chave, novo.clone()));
    novo
}

/// ⭐⭐⭐ **QUANTAS VEZES O CAMPO DO CHÃO FOI REAPROVEITADO** — o readout da cache.
///
/// ⚠️ **A economia é INVISÍVEL a toda régua de valor** (as duas rotas dão o MESMO campo, que é o
/// que a torna segura) ⇒ *o gate dela mede a CONTA*. É a mesma lei que o passe de contacto do
/// Motion já paga com o `separacoes()`.
pub static ACERTOS_DO_CHAO: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
