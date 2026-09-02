//! ⭐⭐⭐ **O NÓ SOLDADO** (plano 39) — quem mais partilha esta ponta.
//!
//! Módulo irmão de [`crate::selection`] pelo tecto de 700 LOC, e o corte é por RESPONSABILIDADE:
//! aquele responde *"o que está seleccionado"*, este responde *"quem anda junto com isto"*. São
//! perguntas diferentes, e a segunda é a que faz uma rede soldada sobreviver ao dedo.

use crate::PenTool;
use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

/// ⭐⭐⭐ **AS PONTAS DE UM CAMINHO, em índice PLANO** — uma por extremo de cada contorno ABERTO.
///
/// # Porque isto deixou de ser `[0, verts.len() - 1]`
///
/// Report do Enio (2026-09-02): *"o weld cria uma grande quantidade de path na hierarquia quando
/// na verdade deveria criar apenas 1"*. A rede soldada passou a ser **um objecto composto** — um
/// `VecPath` cujos contornos são os arcos —, e a partir daí *as duas pontas de um nó vivem no
/// MESMO caminho*. Um predicado que só olhava `verts` respondia sobre o primeiro arco e era **cego
/// aos outros**: a marca apagava-se e o arrasto rasgava a rede no primeiro dedo.
///
/// ⚠️ **O índice é o PLANO** ([`ph2d_vec_scene::VecPath::flat_vert`]) porque é o que o resto da
/// edição fala — `vert`, `vert_mut`, o hit-test e a multi-selecção. Uma segunda numeração aqui
/// poria o anel numa âncora e o arrasto noutra.
///
/// ⛔ **Um contorno FECHADO não entra**: ele não tem ponta, e o vértice `0` dele é uma emenda.
fn endpoints_flat(p: &VecPath) -> Vec<usize> {
    let mut out = Vec::new();
    for c in 0..p.contour_count() {
        let Some((verts, closed)) = p.contour(c) else {
            continue;
        };
        if closed || verts.len() < 2 {
            continue;
        }
        let n = verts.len();
        out.extend(
            [p.flat_vert(c, 0), p.flat_vert(c, n - 1)]
                .into_iter()
                .flatten(),
        );
    }
    out
}

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
    /// solda funde as pontas numa coordenada só (`weld::cluster_endpoints`), e o `Join`/o encaixe
    /// fazem o mesmo. *Uma folga generosa aqui grudaria pontas que o artista pôs perto de
    /// propósito* — o doc do `WELD_TOL` já diz que uma mão nunca chega a essa distância.
    ///
    /// ⛔ **Um contorno FECHADO não entra**: ele não tem ponta, e o vértice `0` dele é uma emenda,
    /// não uma junta. ⚠️ **Mas um caminho COMPOSTO entra, contorno a contorno** — depois de
    /// 2026-09-02 a rede soldada É um caminho composto, e as duas pontas de um nó dela vivem no
    /// mesmo objecto ([`endpoints_flat`]).
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
            let x = self.xf(p.id);
            for i in endpoints_flat(p) {
                if p.id == path && i == vert {
                    continue; // ele próprio; ⚠️ um IRMÃO no mesmo caminho composto ENTRA
                }
                let Some(v) = p.vert(i) else { continue };
                let q = x.apply(v.anchor);
                if (alvo[0] - q[0]).powi(2) + (alvo[1] - q[1]).powi(2) <= t2
                    && !out.contains(&(p.id, i))
                {
                    out.push((p.id, i));
                }
            }
        }
        out
    }

    /// `vert` é uma PONTA deste caminho? (Está num contorno ABERTO, no primeiro ou no último
    /// vértice DELE — em índice plano.)
    fn is_endpoint(p: &VecPath, vert: usize) -> bool {
        endpoints_flat(p).contains(&vert)
    }
}

impl PenTool {
    /// ⭐⭐⭐ **OS NÓS PARTILHADOS DA CENA** — um representante `(caminho, vértice)` por nó, para
    /// quem os quiser DESENHAR.
    ///
    /// # Porque isto existe ao lado do [`Self::welded_with`]
    ///
    /// Report do Enio (2026-09-01): *"as linhas não compartilham o mesmo nó"*. ⚠️ **Ele não tinha
    /// como ver**: duas pontas coincidentes e duas pontas a um pixel pintam o mesmo quadradinho, e
    /// o único instrumento era arrastar — um teste destrutivo para uma pergunta de leitura.
    ///
    /// ⚠️ **A LEI é a mesma, e é isso que importa**: o mesmo predicado de ponta (`is_endpoint`) e a
    /// mesma `WELD_TOL` exacta. *Se a marca tivesse uma segunda régua, ela acenderia onde o dedo
    /// não arrasta junto* — e o artista aprenderia uma lei falsa com o instrumento que existe para
    /// lha ensinar. O gate `the_mark_and_the_drag_answer_the_same_question` cose as duas.
    ///
    /// ⚠️ **Ordenado por `x` antes de comparar**: um passe `n²` sobre as pontas de um documento
    /// grande corre POR QUADRO. Com a lista ordenada, a varredura para no primeiro vizinho a mais
    /// de `WELD_TOL` em `x`, e a folga é tão pequena que a janela é de um punhado de pontas.
    #[must_use]
    pub fn welded_nodes(&self, scene: &VecScene) -> Vec<(VecPathId, usize)> {
        let mut pontas: Vec<(VecPathId, usize, [f64; 2])> = Vec::new();
        for p in scene.paths() {
            let x = self.xf(p.id);
            for i in endpoints_flat(p) {
                if let Some(v) = p.vert(i) {
                    pontas.push((p.id, i, x.apply(v.anchor)));
                }
            }
        }
        pontas.sort_by(|a, b| a.2[0].total_cmp(&b.2[0]));
        let tol = crate::node_ops::WELD_TOL;
        let t2 = tol * tol;
        let mut visto = vec![false; pontas.len()];
        let mut out = Vec::new();
        for a in 0..pontas.len() {
            if visto[a] {
                continue;
            }
            visto[a] = true;
            let mut irmas = 0usize;
            for b in a + 1..pontas.len() {
                if pontas[b].2[0] - pontas[a].2[0] > tol {
                    break; // ordenado: daqui para a frente ninguém alcança
                }
                if visto[b] {
                    continue;
                }
                let d = [
                    pontas[a].2[0] - pontas[b].2[0],
                    pontas[a].2[1] - pontas[b].2[1],
                ];
                if d[0].mul_add(d[0], d[1] * d[1]) <= t2 {
                    visto[b] = true;
                    irmas += 1;
                }
            }
            if irmas > 0 {
                out.push((pontas[a].0, pontas[a].1));
            }
        }
        out
    }
}
