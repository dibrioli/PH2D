//! ⭐⭐⭐ **A TINTA DE UMA IMAGEM PRESA: o que vira MALHA** — irmão do [`super`] pelo tecto de LOC,
//! com o corte por RESPONSABILIDADE: ali mora *a imagem presa desenha-se como malha*, aqui **de que
//! pixels a malha nasce**.
//!
//! ⚠️ **O endereço não muda:** o [`super`] re-exporta as três portas daqui.
//!
//! ⭐ **A [`cell_alpha`] é a peça da F11** (2026-09-17): antes dela a malha de uma FOLHA de quadros
//! era traçada sobre a folha inteira e o quad espremia-a no sítio de uma célula, **em silêncio**.

use crate::skin_image::Mesh2d;

/// ⭐⭐ **A MALHA DESTA IMAGEM, a partir dos pixels dela** — a porta do gesto de prender.
///
/// ⚠️ **Só o canal ALFA entra.** A cobertura é o que decide a silhueta, e passar as três cores
/// junto seria dar ao traçador três respostas para a mesma pergunta.
///
/// ⭐⭐⭐ **`focos` são as ARTICULAÇÕES, em pixels da imagem** (report do dono, 2026-09-10:
/// *«deveria ser um quadmesh inteligente com maior densidade nas áreas das articulações»*). Elas
/// entram porque só quem prende sabe onde a dobra vai acontecer — o leaf da geometria não sabe o
/// que é um osso, e não devia saber.
#[must_use]
pub fn mesh_from_rgba(
    rgba: &[u8],
    width: u32,
    height: u32,
    focos: &[[f64; 2]],
    opts: ph2d_poly2d::GridOptions,
) -> Option<Mesh2d> {
    let alfa: Vec<u8> = rgba.iter().skip(3).step_by(4).copied().collect();
    mesh_from_alpha(&alfa, width, height, focos, opts)
}

/// A mesma lei da irmã acima, com o plano de **alfa** já extraído — a porta de quem compôs a alfa
/// em vez de a ler de uma imagem (a união dos quadros de uma folha, [`cell_alpha`]).
///
/// ⛔ *Duas chamadas ao traçador seriam duas respostas para «que malha esta tinta dá»*: a irmã
/// delega aqui, e a única diferença entre as duas é o formato dos pixels à entrada.
#[must_use]
pub fn mesh_from_alpha(
    alfa: &[u8],
    width: u32,
    height: u32,
    focos: &[[f64; 2]],
    opts: ph2d_poly2d::GridOptions,
) -> Option<Mesh2d> {
    ph2d_poly2d::grid_mesh_of(alfa, width, height, focos, opts)
}

/// ⭐⭐⭐ **A TINTA DE UMA CÉLULA É A UNIÃO DE TODOS OS QUADROS** — o plano de alfa que o traçador da
/// malha recebe quando a sprite é uma FOLHA.
///
/// ⚠️⚠️ **É a união e não o quadro VIVO, e a razão é a animação:** uma malha traçada só sobre o
/// quadro que está na tela **recorta** todos os outros — o artista prende no quadro `0`, dá play, e
/// os braços do quadro `3` desaparecem. A união cobre o que qualquer quadro possa desenhar, e onde
/// um quadro não tem tinta a alfa é `0`: *fora da tinta a malha é invisível, então cobrir a mais não
/// custa pixel nenhum — cobrir a menos custa a arte*.
///
/// ⭐ **Com uma célula só ela é a identidade BYTE-A-BYTE** (origem `[0,0]`, célula = a imagem), o
/// que é o que mantém toda sprite normal exactamente como estava.
///
/// ⚠️ **A amostragem é por pixel INTEIRO e satura na borda:** a célula pode ter lado fraccionário
/// (uma região de `41` px em `2` colunas), e ali o último pixel repete em vez de ler o vizinho —
/// que é a arte da célula do lado.
#[must_use]
pub fn cell_alpha(rgba: &[u8], src: [u32; 2], cells: &ph2d_render::SourceCells) -> Vec<u8> {
    let [cw, ch] = cells.cell_px();
    let (sw, sh) = (src[0] as usize, src[1] as usize);
    let mut out = vec![0u8; cw as usize * ch as usize];
    for k in 0..cells.count() {
        let o = cells.cell_origin(k);
        for y in 0..ch as usize {
            let sy = ((o[1] as usize).saturating_add(y)).min(sh.saturating_sub(1));
            for x in 0..cw as usize {
                let sx = ((o[0] as usize).saturating_add(x)).min(sw.saturating_sub(1));
                let Some(&a) = rgba.get((sy * sw + sx) * 4 + 3) else {
                    continue;
                };
                let d = &mut out[y * cw as usize + x];
                *d = (*d).max(a);
            }
        }
    }
    out
}
