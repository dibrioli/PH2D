//! ⛔ **PORTA DE MEDIÇÃO TEMPORÁRIA** (doc 121 §9.20, a prova antes do produto): a lei por colunas de
//! antes (`ph2d_contact::separate` + `impulsos`, verbatim de `36a9db77c`), atrás de uma chave POR FIO
//! ([`mede_com_a_lei_antiga`]) para a sonda `prova_do_mundo_de_contacto` correr as duas leis no MESMO
//! processo. Sai com a prova.

use ph2d_nodegraph::attr::Stream;
use std::cell::Cell;

thread_local! {
    static LEI_ANTIGA: Cell<bool> = const { Cell::new(false) };
}

/// Liga (`true`) a lei antiga NESTE fio de execução — só a sonda a usa.
pub fn mede_com_a_lei_antiga(sim: bool) {
    LEI_ANTIGA.with(|c| c.set(sim));
}

pub(crate) fn ligada() -> bool {
    LEI_ANTIGA.with(Cell::get)
}

const VARREDURAS: usize = 8;

pub(crate) fn resolve(
    state: &Stream,
    p: &mut [[f32; 2]],
    vel: &mut [[f32; 2]],
    pesos: &[f32],
    antes_do_passo: &[[f32; 2]],
    girou: &[f32],
    dt: impl Fn(usize) -> f32,
) -> (Vec<f32>, Vec<f32>) {
    let n = p.len();
    let Some(colisores) = ph2d_contact::colisores(state) else {
        return (Vec::new(), Vec::new());
    };
    if colisores.len() != n || antes_do_passo.len() != n || girou.len() != n {
        return (Vec::new(), Vec::new());
    }
    let inv_inercia = ph2d_contact::inv_inercias(state, &colisores, pesos);
    let material =
        ph2d_contact::materiais(state).unwrap_or_else(|| vec![ph2d_contact::Material::LISO; n]);
    let mut giro = vec![0.0_f32; n];
    let antes = p.to_vec();
    let pecas = ph2d_contact::Pecas {
        colisores: &colisores,
        pesos,
        inv_inercia: &inv_inercia,
        deslize: Some(ph2d_contact::Deslize {
            antes: antes_do_passo,
            girou_antes: girou,
            material: &material,
        }),
    };
    ph2d_contact::separate(
        p,
        &mut ph2d_contact::Saida { giro: &mut giro },
        &pecas,
        VARREDURAS,
    );
    if !ph2d_contact::Leis::EM_VIGOR.giro_posicional {
        giro.iter_mut().for_each(|g| *g = 0.0);
    }
    let mut spin = vec![0.0_f32; n];
    ph2d_contact::impulsos(
        &antes,
        &mut ph2d_contact::Movimento {
            vel,
            giro: &mut giro,
            spin: &mut spin,
        },
        &pecas,
        &dt,
        ph2d_contact::Leis::EM_VIGOR,
    );
    (giro, spin)
}
