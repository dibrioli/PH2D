//! ⭐⭐ doc 121 §9.18 (C) — **O TRACEJADO DA ROTA VELLO PELA LEI DA PLACA.** O traçador do Vello MORDE um
//! pedaço de traço curto depois de uma quina (a junta interior passa pelo pivô e o pedaço cruza-se: enrolamento
//! `0` no lado de dentro), e a placa desenha a união verdadeira. Aqui a rota Vello do Motion deixa de traçar o
//! tracejado e PREENCHE (`nonzero`) os mesmos polígonos que a placa desenha para a mesma cópia, no MESMO nível
//! de aplanamento que ela escolhe:
//!
//! - cópia **conforme** → as marcas da placa (o contorno no espaço local, [`ph2d_shape_gpu::contorno_conforme`]),
//!   uma vez por geometria e nível, pelo afim da cópia;
//! - cópia **esticada** → o eixo percorrido no ecrã pela porta CPU da lei ([`ph2d_shape_gpu::contorno_do_eixo`]).
//!
//! ⚠️ Só o caso que a placa desenha com o eixo: UM traço, sem marcadores, com o tracejado `[traço, vão]` que o
//! eixo exprime. O resto (contínuo, marcadores, padrão, pincel) continua pelo traçador da casa.

use std::collections::BTreeMap;

use ph2d_shape_gpu::{AfimDaCopia, EixoItem, LEVELS, ShapeInput};
use ph2d_vec_render::FormaParaAPlaca;
use ph2d_vec_scene::VecPath;
use ph2d_vector::{Affine, BezPath, Brush, Fill, VectorScene};

/// O traço pela lei da placa, com as geometrias deste `encode` (o cache por handle vive um quadro, como o
/// das tesselações do lote).
#[derive(Default)]
pub(crate) struct TracoDaPlaca {
    cache: BTreeMap<u32, Option<Geometria>>,
}

/// O que a placa recebe de uma forma, as tolerâncias dos níveis e, pedidos um a um, o eixo e as marcas de
/// cada nível.
struct Geometria {
    forma: FormaParaAPlaca,
    ext: f64,
    tol: [f32; LEVELS],
    eixos: [Option<Vec<EixoItem>>; LEVELS],
    conformes: [Option<BezPath>; LEVELS],
}

impl Geometria {
    fn de(path: &VecPath) -> Option<Self> {
        let forma = ph2d_vec_render::forma_para_a_placa(path)?;
        let [(_, style)] = forma.tracos.as_slice() else {
            return None;
        };
        if !forma.preenchimentos_do_traco.is_empty()
            || ph2d_shape_gpu::tracejado_do_eixo(style).is_none()
        {
            return None;
        }
        let ext = ph2d_shape_gpu::extensao(&entrada(&forma))?;
        #[expect(clippy::cast_possible_truncation, reason = "a placa lê f32")]
        let tol = std::array::from_fn(|k| ph2d_shape_gpu::tolerancia(ext, k) as f32);
        Some(Self {
            forma,
            ext,
            tol,
            eixos: Default::default(),
            conformes: Default::default(),
        })
    }

    fn eixo(&mut self, nivel: usize) -> &[EixoItem] {
        let (forma, ext) = (&self.forma, self.ext);
        self.eixos[nivel].get_or_insert_with(|| {
            ph2d_shape_gpu::eixo_do_nivel(&entrada(forma), ph2d_shape_gpu::tolerancia(ext, nivel)).0
        })
    }

    fn conforme(&mut self, nivel: usize) -> &BezPath {
        let (forma, ext) = (&self.forma, self.ext);
        self.conformes[nivel].get_or_insert_with(|| {
            ph2d_shape_gpu::contorno_conforme(
                &entrada(forma),
                ph2d_shape_gpu::tolerancia(ext, nivel),
            )
        })
    }
}

fn entrada(f: &FormaParaAPlaca) -> ShapeInput<'_> {
    crate::motion_shape_placa::entrada_de(f)
}

impl TracoDaPlaca {
    /// O traço da cópia de `handle` sob `transform`, com `pincel`; `false` ⇒ não é o caso da lei (a casa traça).
    pub(crate) fn desenha(
        &mut self,
        handle: u32,
        path: &VecPath,
        transform: Affine,
        pincel: &Brush,
        cena: &mut VectorScene,
    ) -> bool {
        let Some(g) = self
            .cache
            .entry(handle)
            .or_insert_with(|| Geometria::de(path))
        else {
            return false;
        };
        #[expect(clippy::cast_possible_truncation, reason = "a placa lê f32")]
        let [a, b, c, d, e, f] = transform.as_coeffs().map(|x| x as f32);
        let lin = [a, b, c, d];
        let n = ph2d_shape_gpu::nivel_da_copia(&g.tol, lin);
        if n.conforme {
            let contorno = g.conforme(n.nivel);
            cena.inner_mut()
                .fill(Fill::NonZero, transform, pincel, None, contorno);
        } else {
            let m = AfimDaCopia {
                lin,
                t: [e, f],
                caneta: ph2d_shape_gpu::caneta_de(lin),
            };
            let mut contorno = BezPath::new();
            ph2d_shape_gpu::contorno_do_eixo(g.eixo(n.nivel), &m, false, &mut contorno);
            cena.inner_mut()
                .fill(Fill::NonZero, Affine::IDENTITY, pincel, None, &contorno);
        }
        true
    }
}
