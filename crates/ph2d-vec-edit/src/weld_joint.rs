//! ⭐⭐⭐ **O NÓ SOLDADO** (plano 39) — quem mais partilha esta ponta.
//!
//! Módulo irmão de [`crate::selection`] pelo tecto de 700 LOC, e o corte é por RESPONSABILIDADE:
//! aquele responde *"o que está seleccionado"*, este responde *"quem anda junto com isto"*. São
//! perguntas diferentes, e a segunda é a que faz uma rede soldada sobreviver ao dedo.

use crate::PenTool;
use ph2d_vec_scene::{VecPathId, VecScene};

impl PenTool {
    /// ⭐⭐⭐ **AS OUTRAS PONTAS QUE PARTILHAM ESTE NÓ** (plano 39) — em coordenadas de MUNDO, e
    /// **só pontas**: um vértice interior não é junta de nada.
    ///
    /// # Duas pontas no mesmo sítio são UM nó
    ///
    /// É a lei do esboço de CAD, e é o que faz uma rede soldada sobreviver ao dedo. Report do Enio
    /// (2026-08-31, com foto): *"weld dividiu e não soldou (eu que afastei os pontos)"* — os arcos
    /// eram coincidentes e independentes, e arrastar um deixava o buraco no meio.
    ///
    /// ⚠️ **A comparação é EXACTA** (`WELD_TOL`, 1e-6 de mundo), e não uma folga de clique: quem
    /// solda funde as pontas numa coordenada só (`weld::fuse_endpoints`), e o `Join`/o encaixe
    /// fazem o mesmo. *Uma folga generosa aqui grudaria pontas que o artista pôs perto de
    /// propósito* — o doc do `WELD_TOL` já diz que uma mão nunca chega a essa distância.
    ///
    /// ⛔ **Um caminho FECHADO não entra**: ele não tem ponta, e o vértice `0` dele é uma emenda,
    /// não uma junta.
    #[must_use]
    pub fn welded_with(
        &self,
        scene: &VecScene,
        path: VecPathId,
        vert: usize,
    ) -> Vec<(VecPathId, usize)> {
        let Some(alvo) = scene
            .path(path)
            .filter(|p| Self::is_endpoint(p, vert))
            .and_then(|p| p.vert(vert))
            .map(|v| self.xf(path).apply(v.anchor))
        else {
            return Vec::new();
        };
        let t2 = crate::node_ops::WELD_TOL * crate::node_ops::WELD_TOL;
        let mut out = Vec::new();
        for p in scene.paths() {
            if p.closed || !p.subpaths.is_empty() || p.verts.len() < 2 {
                continue;
            }
            let x = self.xf(p.id);
            for i in [0, p.verts.len() - 1] {
                if p.id == path && i == vert {
                    continue;
                }
                let q = x.apply(p.verts[i].anchor);
                if (alvo[0] - q[0]).powi(2) + (alvo[1] - q[1]).powi(2) <= t2
                    && !out.contains(&(p.id, i))
                {
                    out.push((p.id, i));
                }
            }
        }
        out
    }

    /// `vert` é uma PONTA deste caminho? (Aberto, sem subpaths, e no índice `0` ou no último.)
    fn is_endpoint(p: &ph2d_vec_scene::VecPath, vert: usize) -> bool {
        !p.closed
            && p.subpaths.is_empty()
            && p.verts.len() >= 2
            && (vert == 0 || vert == p.verts.len() - 1)
    }
}
