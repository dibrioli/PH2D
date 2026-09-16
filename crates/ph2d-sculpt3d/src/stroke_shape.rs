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
        if brush.verb == Verb::Plane {
            return self.pegada_de_plano(mesh, brush, dab);
        }
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

    /// ⭐⭐⭐ **A PEGADA DO [`Verb::Plane`]** — o elipsóide dos dois tectos, ou o
    /// **nada** quando o traço ainda não tem direcção.
    ///
    /// ⚠️⚠️ **O «nada» é uma pegada e não um `return` antecipado**, e é isso que
    /// o torna a lei da espec §1 em vez de uma optimização: um par de tectos a
    /// zero põe todo vértice **na borda** da silhueta, onde toda curva de queda
    /// vale zero ⇒ o dab corre inteiro (a máscara, a simetria, o undo, o refit) e
    /// **move `0` vértices**, que é exactamente o que o alvo mede num traço de um
    /// dab só. ⛔ Um desvio antes do laço teria de reproduzir à mão tudo o que o
    /// laço faz de resto.
    ///
    /// ⚠️ **A direcção é lida do MESMO produto vectorial que a faixa lê**
    /// (`normal × caminho`), e é ele que responde aos dois degenerados de uma vez:
    /// caminho nulo (o primeiro dab) e caminho paralelo à normal (a mão a andar
    /// «para dentro» da superfície). *Duas leituras do mesmo eixo divergiriam no
    /// dia em que o piso de degeneração de uma delas mudasse.*
    fn pegada_de_plano(&mut self, mesh: &Mesh, brush: &Brush, dab: &Dab) -> crate::Footprint {
        let inerte = |origem: [f32; 3]| {
            crate::Footprint::Tectos(crate::Tectos {
                origin: origem,
                // ⚠️ Uma normal qualquer serve: com os dois tectos a zero a
                // coordenada é `1` para todo vértice, e a curva não a lê.
                normal: [0.0, 1.0, 0.0],
                altura: 0.0,
                profundidade: 0.0,
            })
        };
        // ⭐ **UMA escrita, DOIS leitores** — ver [`super::SculptStroke::plano`]:
        // a silhueta abaixo e o alvo por-vértice saem deste mesmo valor.
        self.plano = self.plano_da_pegada(mesh, brush, dab);
        let Some(plano) = self.plano else {
            // Sem amostra nenhuma não há plano — e um pincel sem plano não tem
            // para onde puxar. Ver [`super::plano_da_pegada`].
            return inerte(dab.center);
        };
        if crate::footprint::unit(target::cross(plano.normal, dab.path)).is_some() {
            self.plano_teve_direccao = true;
        }
        if !self.plano_teve_direccao {
            return inerte(plano.centro);
        }
        // ⭐ A inversão escolhe **qual** par de tectos este dab usa, por uma porta
        // com dois chamadores — ver [`crate::PlanoInversao::tectos`].
        let invertido = brush.invert && brush.verb.honours_invert();
        let (altura, profundidade) =
            brush
                .plano_inversao
                .tectos(invertido, brush.plano_altura, brush.plano_profundidade);
        crate::Footprint::Tectos(crate::Tectos {
            origin: plano.centro,
            normal: plano.normal,
            altura,
            profundidade,
        })
    }
}
