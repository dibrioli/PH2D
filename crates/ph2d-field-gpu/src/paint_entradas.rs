//! ⭐⭐⭐⭐ **QUE TEXTO CADA ENTRADA DO PINTOR LEVA — e a compilação delas em LOTE.**
//!
//! ⚠️ **Corte por RESPONSABILIDADE, forçado pelo tecto de LOC do [`super`]** (`702` contra `700`,
//! 2026-10-01): o irmão despacha os passes; este decide de que TEXTO cada pipeline nasce, que é a
//! pergunta que decide se uma forma nova recompila o pintor ou não.
//!
//! # ⭐⭐⭐⭐ A fita por ENTRADA, e não por módulo
//!
//! Report do dono (2026-10-01): *«ao acrescentar um box ele demora uns 5 segundos para aparecer»*.
//! Medido pelo censo [`crate::paint_wgsl`]`::censo_de_quem_toca_a_fita`: das entradas deste passe,
//! só o `assa_sondas` MARCHA a peça, e o `pinta`/`pinta_bordas` só a leem pela curvatura. Até aqui a
//! fita real ia para TODAS assim que o ricochete ligava ⇒ cada forma nova recompilava o
//! `pinta_bordas` (`2,0`–`3,9 s` no driver, `PH2D_PIPELINE_LOG`) e o `pinta` (`0,9 s`), que nunca a
//! liam. ⇒ quem não a lê compila com a INERTE, cujo texto não muda com a peça, e só o `assa_sondas`
//! paga a compilação de uma estrutura nova (`~140 ms`).
//!
//! ⚠️ **Os pipelines de textos diferentes partilham o MESMO grupo `0`:** o `k` é montado pelo
//! traçado a partir da fita real, e a inerte não o indexa.

use crate::paint::PaintSetup;
use ph2d_field_eval::wgsl::TapeWgsl;

/// O que o [`escolhe`] decidiu: o texto da pintura, o das sondas, e se o ricochete corre.
pub(crate) struct Fitas<'a> {
    /// A fita de TODA entrada menos o `assa_sondas` — a real só quando alguém lê a curvatura.
    pub(crate) pintura: &'a TapeWgsl,
    /// A fita do `assa_sondas` — sempre a real: ele marcha a peça.
    pub(crate) sondas: &'a TapeWgsl,
    /// As direcções do ricochete neste quadro — `0` quando o movimento não pode esperar.
    pub(crate) ao_rays: u32,
}

/// ⭐⭐⭐⭐ **Escolhe a fita de cada entrada, decide o ricochete, e compila o que falta em LOTE.**
///
/// ⭐ **O lote é o [`crate::FieldPipelines::precompila`]**, com as MESMAS condições dos pedidos
/// que o despacho faz a seguir: medido, o `pinta` e o `pinta_bordas` em fila eram `0,47 + 1,27 s`
/// na primeira entrada no Render, e em paralelo pagam o mais lento dos dois.
#[allow(clippy::too_many_arguments)]
pub(crate) fn escolhe<'a>(
    cache: &mut crate::FieldPipelines,
    device: &wgpu::Device,
    fonte: &str,
    fita: &'a TapeWgsl,
    inerte: &'a TapeWgsl,
    pintor: &PaintSetup<'_>,
    bordas: u64,
    layout: &wgpu::PipelineLayout,
    // ⏱️ Há sondas que este quadro pode ler sem as assar — ver
    // [`crate::FieldPipelines::sondas_a_mexer`]. Num quadro que não pode esperar, sem elas o
    // ricochete fica de fora (a assadura é do assente).
    sondas_servem: bool,
) -> Fitas<'a> {
    let sondas = fita;
    let pintura = if pintor.le_o_campo { fita } else { inerte };
    // ⭐⭐⭐⭐ Ver [`PaintSetup::ricochete_sem_esperar`] — os QUATRO pipelines que o ricochete pede.
    let ao_rays = if pintor.ricochete_sem_esperar
        && pintor.ao_rays > 0
        && !(sondas_servem
            && cache.tem_entrada(fonte, sondas, "assa_sondas")
            && ["pinta", "pinta_ricochete", "borra_ricochete"]
                .iter()
                .all(|e| cache.tem_entrada(fonte, pintura, e)))
    {
        0
    } else {
        pintor.ao_rays
    };
    let mut lista = vec![(fonte, pintura, "pinta")];
    if ao_rays > 0 {
        lista.extend([
            (fonte, pintura, "pinta_ricochete"),
            (fonte, sondas, "assa_sondas"),
            (fonte, pintura, "borra_ricochete"),
        ]);
    }
    if pintor.mole {
        lista.extend([
            (fonte, pintura, "borra_mole_h"),
            (fonte, pintura, "borra_mole_v"),
        ]);
    }
    if bordas > 0 {
        lista.push((fonte, pintura, "pinta_bordas"));
    }
    cache.precompila(device, &lista, Some(layout));
    Fitas {
        pintura,
        sondas,
        ao_rays,
    }
}
