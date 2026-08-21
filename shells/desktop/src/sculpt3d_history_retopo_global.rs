//! **A PORTA DA CADEIA GLOBAL** — o botão a chamar o motor do pivô (ADR-0161).
//!
//! Irmão (`#[path]`) do [`super`] e do [`crate::sculpt3d::history_remesh`], e o
//! corte é de ASSUNTO: lá moram o voxel remesh e a retopologia **local** (o porte
//! BSD do Instant Meshes, ADR-0160); aqui a cadeia **global** — remalha isotrópica
//! · campo cruzado com decisão inteira · traçado dos patches · quantização Bi-MDF
//! · quadrangulação por patch.
//!
//! # Por que ela substitui o botão, com o número ao lado
//!
//! ⭐ **A grandeza que decide é a contagem de vértices irregulares**, e uma grade
//! numa esfera admite **oito**. Medido no corpus da bancada, mesmo alvo de
//! densidade:
//!
//! | malha | motor LOCAL | **cadeia GLOBAL** | oráculo |
//! |---|---|---|---|
//! | esfera 96×144 | 68,7 % quads · ~1 800 irreg. | **100 % · 14** | 100 % · ~7 |
//! | toro 64×32 | 64,9 % · ~2 200 | **100 % · 24** | 100 % · ~5 |
//! | esfera 98 k | 82,7 % · ~1 000 | **100 % · 21** | 100 % · ~9 |
//!
//! ⚠️ **E ela é LENTA em comparação**: o motor local responde em sub-segundo e
//! este leva ~8,5 s numa escultura de 98 k. É a troca declarada do ADR-0161 — o
//! local fica como *preview*, e é por isso que ele não foi removido.
//!
//! ⛔ **`PH2D_RETOPO_LEGACY=1` volta ao motor local**, e ele existe para bissecar:
//! um resultado mau só se atribui a esta cadeia depois de se ver o que o outro faz
//! com a mesma peça.

use super::remesh::QuadRemeshReport;
use super::{RemeshRefusal, Sculpt3dScene, SculptStroke, StrokeUndo};

/// **O ORÇAMENTO da busca da quantização.**
///
/// ⚠️ **Ele é um teto de ESPERA, e não de qualidade.** A busca do F4 é exacta e
/// prova o ótimo; o que este número limita é quanto tempo o artista fica à espera
/// antes de o gesto desistir com um nome. Medido no corpus (2026-08-21): as seis
/// malhas fecham **com prova** dentro dele, e a mais cara gasta 66 das 256
/// expansões. *Um teto que nenhuma fixtura toca é folga, não política.*
const QUANTIZE_BUDGET: (usize, usize) = (256, 512);

impl Sculpt3dScene {
    /// **A RETOPOLOGIA GLOBAL** — a cadeia inteira do ADR-0161. Devolve o
    /// [`QuadRemeshReport`].
    ///
    /// ⚠️ **Ela NÃO recebe o `adaptive`, e a ausência é deliberada.** Este backend
    /// não tem densidade adaptativa: passá-lo aqui seria um knob que o painel
    /// mostra, o artista mexe e **nada consome** — o defeito que já custou uma
    /// caçada nesta base. O painel diz isso em voz alta quando ele não é zero.
    ///
    /// ⚠️ **O `detail` atravessa pela MESMA lei do irmão local**
    /// (`ph2d_quadflow::edge_for_detail`), e isso é o que faz o slider significar
    /// a mesma coisa nos dois motores. *Duas leis para o mesmo knob é como dois
    /// botões passam a precisar de duas explicações.*
    ///
    /// ⚠️ **A mesma recusa de PILHA dos irmãos**, e pelo mesmo motivo: a saída é
    /// uma malha com outra contagem de vértices, e um nível de multires é uma
    /// subdivisão da base.
    pub(in crate::sculpt3d) fn quad_remesh_global(
        &mut self,
        detail: f32,
    ) -> Result<QuadRemeshReport, RemeshRefusal> {
        if self.level_count() != 1 {
            return Err(RemeshRefusal::MultiresStack);
        }
        let t = std::time::Instant::now();
        let target = ph2d_quadflow::edge_for_detail(self.mesh(), detail);

        // ── F1. A remalha isotrópica, sobre uma CÓPIA.
        //
        // ⚠️ **Ela é o passe que torna a saída independente da entrada**, e sem
        // ela um cubo de oito vértices devolve malha vazia. A cópia existe porque
        // a peça na cena só é substituída no fim: uma recusa a meio da cadeia tem
        // de deixar o artista com o que ele tinha.
        let reference = self.mesh().clone();
        let mut work = reference.clone();
        ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
        work.triangulate();

        // ── F2. O campo cruzado com decisão inteira global.
        let dual = ph2d_crossfield::Dual::build(&work);
        let (field, _) = ph2d_crossfield::solve_miq(&dual);

        // ── F3. As paredes e os patches.
        let layout = ph2d_trace::trace_patches(&work, &dual, &field);
        let spec = layout.to_layout(target).map_err(RemeshRefusal::Layout)?;

        // ── F4. A quantização Bi-MDF.
        let budget = ph2d_quantize::Budget::new(QUANTIZE_BUDGET.0, QUANTIZE_BUDGET.1);
        let (quant, _) =
            ph2d_quantize::quantize_within(&spec, budget).map_err(RemeshRefusal::Quantize)?;

        // ── F5. A malha.
        //
        // ⚠️ **A referência da reprojeção é a malha ORIGINAL e não a remalhada.**
        // Reprojetar sobre a saída do F1 seria alisar contra uma superfície que já
        // é uma aproximação — o erro das duas somaria, e a silhueta que o artista
        // esculpiu perderia o que o F1 já tinha arredondado.
        let (out, r) =
            ph2d_quadfill::fill(&reference, &layout, &quant, ph2d_quadfill::SMOOTHING_ROUNDS)
                .map_err(RemeshRefusal::Fill)?;
        if out.faces().is_empty() {
            return Err(RemeshRefusal::TooCoarseToResolve);
        }

        let report = QuadRemeshReport {
            verts: r.verts,
            quads: r.quads,
            non_quads: r.non_quads,
            edge: target,
            ms: t.elapsed().as_secs_f64() * 1000.0,
            // ⚠️ **`boundary_edges` e não uma contagem de buracos por inundação.**
            // Uma aresta com uma face só é a assinatura da casca aberta, e é a
            // mesma grandeza que o irmão local reporta na coluna `holes`.
            holes: r.boundary_edges,
            irregular: r.irregular,
        };
        let previous = core::mem::replace(self.mesh_mut().ok_or(RemeshRefusal::EmptyScene)?, out);
        self.record(StrokeUndo::Remeshed(Box::new(previous)));
        // A malha é OUTRA: o traço em voo fala de vértices que não existem mais.
        self.stroke = SculptStroke::default();
        self.mesh_rebuilt();
        Ok(report)
    }
}

/// **O motor LOCAL foi pedido explicitamente?** — `PH2D_RETOPO_LEGACY=1`.
///
/// ⚠️ **Ele existe para BISSECAR e não para escolher qualidade.** Um resultado mau
/// só se atribui à cadeia global depois de se ver o que o porte local faz com a
/// mesma peça — e sem esta porta a comparação exigiria recompilar.
#[must_use]
pub(in crate::sculpt3d) fn legacy_requested() -> bool {
    legacy_from(std::env::var("PH2D_RETOPO_LEGACY").ok().as_deref())
}

/// **A DECISÃO, sem tocar no ambiente** — a metade que se pode gatear.
///
/// ⚠️ **A separação não é estética: é o que torna esta lei testável.** Esta crate
/// proíbe `unsafe`, e desde a edição 2024 `std::env::set_var` é `unsafe` — um gate
/// que quisesse mexer na variável não compilava. Com a decisão pura, o gate lê a
/// lei e a leitura do ambiente fica numa linha que não tem o que decidir.
///
/// ⚠️ **O `"0"` DESLIGA**, e a regra não é decoração: sem ela um
/// `PH2D_RETOPO_LEGACY=0` esquecido numa sessão ligaria o motor antigo — o oposto
/// do que a linha diz. É a mesma lei do `PH2D_GPU_COOK=0` e do
/// `PH2D_FLIP_NEW_ENGINE=0`.
#[must_use]
fn legacy_from(value: Option<&str>) -> bool {
    value.is_some_and(|v| v != "0")
}

#[cfg(test)]
mod tests {
    /// ⭐ **A PORTA DE BISSECAÇÃO, e a decisão dela é pura de propósito.**
    ///
    /// ⚠️ **O gesto em si precisa de GPU** (a cena segura buffers de device), então
    /// um gate sobre ele é `skip` gracioso na máquina sem adapter — e *skip
    /// gracioso não é verde*. A **decisão** de qual motor correr não precisa de
    /// nada disso, e é ela que este gate pina.
    #[test]
    fn the_bisect_door_only_opens_when_it_is_asked_to() {
        for (value, want) in [
            // ⭐ O caso por omissão, e é o que decide qual motor o Enio recebe
            // sem configurar nada: a cadeia GLOBAL.
            (None, false),
            // ⚠️ E o `"0"` DESLIGA — sem esta linha, um `=0` esquecido ligaria o
            // motor antigo em silêncio.
            (Some("0"), false),
            (Some("1"), true),
            (Some("sim"), true),
            (Some(""), true),
        ] {
            assert_eq!(
                super::legacy_from(value),
                want,
                "PH2D_RETOPO_LEGACY={value:?} tinha de dar {want}"
            );
        }
    }
}
