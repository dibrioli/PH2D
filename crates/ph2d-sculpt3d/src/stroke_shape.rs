//! **QUE SILHUETA ESTE DAB TEM** — a moldura da pegada, resolvida uma vez por
//! dab.
//!
//! ⚠️ **Filho (`#[path]`) do [`super`] e não um irmão:** ele escreve
//! `self.scrape` e chama o `scrape_planes`, que são privados do traço.
//!
//! O corte é por PERGUNTA: o pai responde *o que um dab FAZ* (o ciclo, a
//! captura, o peso), e aqui mora *que FORMA ele tem* — a caixa da faixa, a
//! lâmina do V, e o disco que é a resposta de todos os outros. As duas crescem
//! por razões diferentes, e foi esta que levou o pai ao teto de LOC.

use super::*;

impl SculptStroke {
    /// A silhueta deste dab, e — de passagem — a lâmina em V que ela usa.
    ///
    /// ⚠️ **Ela ESCREVE `self.scrape` incondicionalmente**, mesmo com outro verbo
    /// em mãos: é o que garante que a moldura nunca descreve o dab anterior.
    pub(super) fn dab_footprint(
        &mut self,
        mesh: &Mesh,
        brush: &Brush,
        dab: &Dab,
        plane: &super::plane::PlaneFit,
    ) -> crate::Footprint {
        // ⚠️ **A SILHUETA é hoisted, como o `alpha_frame`** — ver
        // [`crate::Footprint`]. Ela depende do plano ajustado e da direção do
        // traço, que são fatos do DAB; construí-la por vértice pagaria duas
        // raízes quadradas em cada um.
        //
        // ⚠️ **O `None` do [`crate::Strip::new`] cai no disco**, e ele só
        // acontece sem plano em que deitar a caixa. *Sem CAMINHO* é outra coisa
        // e a faixa trata dela sozinha, nascendo redonda — a distinção custou um
        // gate de produto (ver o doc de [`crate::Strip::new`]).
        // ⚠️ **A LÂMINA EM V, resolvida ANTES da silhueta** — a moldura dela sai
        // da mesma dobradiça, e é ela que decide se este dab deposita alguma
        // coisa. Escrever o campo incondicionalmente (mesmo com outro verbo em
        // mãos) é o que garante que ele nunca descreve o dab anterior.
        self.scrape = if brush.verb == Verb::MultiplaneScrape {
            self.scrape_planes(mesh, brush, dab, plane)
        } else {
            None
        };
        if brush.verb == Verb::ClayStrips {
            // ⚠️ **O plano da faixa SOBE**, e o `plane_offset` do artista já
            // está dentro do `plane.point` — este termo soma ao dele. Ver
            // [`crate::STRIP_PLANE_FRACTION`] para o porquê de o número sair da
            // própria parábola.
            let lift = dab.radius * crate::STRIP_PLANE_FRACTION;
            crate::Strip::new(
                [
                    plane.point[0] + plane.normal[0] * lift,
                    plane.point[1] + plane.normal[1] * lift,
                    plane.point[2] + plane.normal[2] * lift,
                ],
                plane.normal,
                dab.path,
                dab.radius,
                brush.strip_length,
                brush.tip_roundness,
            )
            .map_or(crate::Footprint::Disc, crate::Footprint::Strip)
        } else if brush.verb == Verb::MultiplaneScrape {
            // ⚠️ **O eixo sai da MOLDURA que acabou de ser resolvida, não de um
            // segundo `stroke_axis`** — a lâmina tem de estar deitada na MESMA
            // dobradiça em que os planos giram, e duas derivações do mesmo eixo
            // divergiriam no dia em que o piso de degeneração de uma delas
            // mudasse. O `across` é perpendicular ao `along`, e recuperá-lo por
            // um produto vetorial com a normal é exato num frame ortonormal.
            self.scrape
                .and_then(|s| crate::Blade::new(dab.center, target::cross(s.normal, s.across)))
                .map_or(crate::Footprint::Disc, crate::Footprint::Blade)
        } else {
            crate::Footprint::Disc
        }
    }
}
