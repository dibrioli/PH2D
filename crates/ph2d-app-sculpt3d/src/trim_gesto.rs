//! **O GESTO do corte** — o que a mão desenha, antes de virar volume.
//!
//! Filho (`#[path]`) de [`super`]. A lei que transforma o anel num prisma vive
//! na [`ph2d_trim`]; aqui fica o que é do GESTO: o que o pen-down fotografa, que
//! pontos o arrasto guarda, e o que o pen-up entrega.
//!
//! ⚠️ **Puro de propósito — sem cena, sem `wgpu::Device`, sem câmara.** É isso
//! que o torna gateável, e é a lição que o censo dos knobs desta linha pagou:
//! uma lei que precisa de um dispositivo para ser medida nasce `#[ignore]`, e o
//! CI deixa de a correr.

/// ⭐⭐ **A FORMA vive no PINCEL desde 2026-09-15** — ela é um selector do
/// pincel, como o do esfregão e o do projectar, e o painel oferece-a na secção
/// deles ([`ph2d_sculpt3d::TrimForma`]).
///
/// ⛔ **O enum local MORREU, e a razão é o painel:** enquanto ele vivia aqui, a
/// única maneira de o alcançar era uma tecla — e *uma tecla é alcançável mas
/// não DESCOBRÍVEL*. A tabela de rows do painel lê o `Brush`, logo a forma tinha
/// de estar lá para ter chip.
pub(crate) use ph2d_sculpt3d::TrimForma as Forma;

/// A orientação do varrimento que o artista pediu (espec §5).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum Orientacao {
    /// O eixo é a direcção da vista, invertida.
    ///
    /// ⚠️ **É o valor de fábrica**, e não por gosto: sem superfície sob o cursor
    /// é o único que a espec §3 deixa de pé.
    #[default]
    Vista,
    /// O eixo é a normal da superfície onde o gesto começou.
    Superficie,
}

/// ⭐ **A DISTÂNCIA MÍNIMA entre dois pontos guardados de um laço, em pixels.**
///
/// ⚠️ **Ela nomeia o recurso, que é o CUSTO DA TAMPA:** o prisma tem `2n`
/// vértices e a triangulação da tampa é quadrática em `n`, logo um laço cru de
/// um arrasto lento (centenas de eventos) pagaria o quadrado disso por nada —
/// dois pontos a meio pixel um do outro não descrevem forma nenhuma que o
/// artista consiga ver.
const PASSO_MINIMO_PX: f32 = 4.0;

/// **O ESTADO DO CORTE na cena** — a orientação e o gesto em curso.
///
/// ⭐⭐⭐ **O `armado` SAIU daqui em 2026-09-15, e isso é a wave inteira:** o
/// corte passou a ser um VERBO ([`ph2d_sculpt3d::Verb::BoxTrim`]), logo
/// *«está armado?»* é a mesma pergunta que *«qual é o pincel na mão?»* — e
/// escrevê-la duas vezes era a segunda resposta que diverge no dia em que uma
/// delas mudar. ⭐ De graça vem a **exclusividade**: escolher um pincel desarma
/// o corte e escolher o corte larga o pincel, sem uma regra escrita à mão.
#[derive(Default)]
pub(crate) struct Trim {
    /// ⭐ **O verbo que a TECLA interrompeu** — e nada mais o lê.
    ///
    /// ⚠️ **É estado do ATALHO, não da ferramenta:** quem escolhe o corte no
    /// painel escolhe outro pincel no painel, e nunca passa por aqui. O `L`,
    /// esse, é um ciclo que tem de saber a onde voltar — *um atalho que arma e
    /// não desarma deixa o artista preso à ferramenta que ele espreitou*.
    pub(crate) verbo_anterior: Option<ph2d_sculpt3d::Verb>,
    /// A orientação que o artista pediu.
    pub(crate) orientacao: Orientacao,
    /// O gesto a decorrer, entre o pen-down e o pen-up.
    pub(crate) gesto: Option<Gesto>,
}

/// O gesto em curso.
pub(crate) struct Gesto {
    forma: Forma,
    pontos: Vec<[f32; 2]>,
    /// O que havia sob o cursor no pen-down: `(posição, normal)` em mundo.
    ///
    /// ⚠️ **Fotografado no pen-down e nunca relido** — é a mesma lei que o
    /// polegar e o projectar desta linha já pagaram: uma grandeza relida do vivo
    /// deriva enquanto o gesto acontece.
    acerto: Option<([f32; 3], [f32; 3])>,
}

impl Gesto {
    /// O pen-down: a forma, onde começou, e o que havia por baixo.
    pub(crate) fn comeca(forma: Forma, em: [f32; 2], acerto: Option<([f32; 3], [f32; 3])>) -> Self {
        Self {
            forma,
            pontos: vec![em],
            acerto,
        }
    }

    /// Um evento de movimento.
    pub(crate) fn move_para(&mut self, p: [f32; 2]) {
        match self.forma {
            // ⚠️ A caixa e o círculo guardam só DOIS pontos: os cantos, ou o
            // centro e o raio. Guardar o caminho seria descrever um rectângulo
            // por uma linha que serpenteia.
            Forma::Caixa | Forma::Circulo => {
                self.pontos.truncate(1);
                self.pontos.push(p);
            }
            Forma::Laco => {
                let ultimo = *self.pontos.last().expect("o pen-down pôs um");
                let d = (p[0] - ultimo[0]).hypot(p[1] - ultimo[1]);
                if d >= PASSO_MINIMO_PX {
                    self.pontos.push(p);
                }
            }
        }
    }

    /// Os pontos crus, para o gate da suavização — *a régua do «ao bit»
    /// precisa do lado de dentro*.
    #[cfg(test)]
    pub(crate) fn pontos_para_o_gate(&self) -> Vec<[f32; 2]> {
        self.pontos.clone()
    }

    /// **O anel de ecrã** que a lei recebe (espec §4).
    ///
    /// ⚠️ Devolve vazio quando o gesto não delimita área — quem chama recusa em
    /// voz alta, e a recusa é do gesto, não do motor.
    pub(crate) fn anel(&self, suavizacao: f32, passo_px: f32) -> Vec<[f32; 2]> {
        match self.forma {
            Forma::Caixa => {
                let [a, b] = [self.pontos[0], *self.pontos.last().expect("um")];
                if (a[0] - b[0]).abs() < 1.0 || (a[1] - b[1]).abs() < 1.0 {
                    return Vec::new();
                }
                vec![[a[0], a[1]], [b[0], a[1]], [b[0], b[1]], [a[0], b[1]]]
            }
            Forma::Circulo => {
                let c = self.pontos[0];
                let b = *self.pontos.last().expect("um");
                let r = (b[0] - c[0]).hypot(b[1] - c[1]);
                if r < 1.0 {
                    return Vec::new();
                }
                let n = segmentos_do_circulo(r, passo_px);
                (0..n)
                    .map(|i| {
                        let t = i as f32 / n as f32 * std::f32::consts::TAU;
                        [c[0] + r * t.cos(), c[1] + r * t.sin()]
                    })
                    .collect()
            }
            Forma::Laco => {
                if self.pontos.len() < 3 {
                    return Vec::new();
                }
                // ⭐⭐ **A suavização entra AQUI e só aqui** (ordem do dono,
                // 2026-09-15). ⚠️ Ela é do LAÇO por construção: a caixa e o
                // círculo saem de dois pontos, e não há traço a suavizar. E
                // `0` devolve o traço cru ao bit — a lei
                // [`ph2d_trim::suaviza`] empresta em vez de copiar.
                ph2d_trim::suaviza::suaviza(&self.pontos, suavizacao).into_owned()
            }
        }
    }

    /// ⭐⭐ **A ORIENTAÇÃO EFECTIVA, e se ela foi COAGIDA** (espec §3).
    ///
    /// Sem superfície sob o cursor no pen-down não há normal de onde tirar um
    /// eixo ⇒ a orientação é forçada a [`Orientacao::Vista`].
    ///
    /// ⚠️⚠️ **O alvo faz isto em SILÊNCIO — é a única correcção calada de um
    /// parâmetro autorado em toda a família dele.** Nós devolvemos o facto para
    /// quem chama poder **dizê-lo**, que é a divergência que a espec nomeia
    /// (**N**): *o resto dos vereditos desta casa é recusa em voz alta, e um
    /// knob que muda de valor sem avisar é a espécie de controlo que mente.*
    pub(crate) fn orientacao(&self, pedida: Orientacao) -> (Orientacao, bool) {
        match (pedida, self.acerto) {
            (Orientacao::Superficie, None) => (Orientacao::Vista, true),
            _ => (pedida, false),
        }
    }

    /// O plano da forma (espec §5): origem e eixo do varrimento.
    ///
    /// `direccao_da_vista` é para onde a câmara olha; o eixo de
    /// [`Orientacao::Vista`] é ela **invertida**.
    pub(crate) fn plano(
        &self,
        pedida: Orientacao,
        direccao_da_vista: [f32; 3],
        origem_sem_acerto: [f32; 3],
    ) -> (ph2d_trim::Plano, bool) {
        let (efectiva, coagida) = self.orientacao(pedida);
        let origem = self.acerto.map_or(origem_sem_acerto, |(p, _)| p);
        let normal = match efectiva {
            Orientacao::Superficie => self.acerto.map_or(
                [
                    -direccao_da_vista[0],
                    -direccao_da_vista[1],
                    -direccao_da_vista[2],
                ],
                |(_, n)| n,
            ),
            Orientacao::Vista => [
                -direccao_da_vista[0],
                -direccao_da_vista[1],
                -direccao_da_vista[2],
            ],
        };
        (ph2d_trim::Plano { origem, normal }, coagida)
    }
}

#[cfg(test)]
#[path = "trim_gesto_tests.rs"]
mod tests;

/// ⭐⭐ **A PRÉ-VISUALIZAÇÃO do gesto** — o que a mão já desenhou, em pixels.
///
/// ⚠️ **Sem ela o artista arrasta e não vê nada até largar**, e um gesto invisível
/// lê-se como uma ferramenta que não responde. É a mesma razão pela qual o
/// contorno e a pose desta linha ganharam indicador: *o anel do cursor descreve
/// mal um verbo cuja região não sai do cursor.*
///
/// Devolve o anel fechado em coordenadas de ECRÃ, ou `None` quando ainda não há
/// forma nenhuma.
impl Gesto {
    pub(crate) fn previa(&self, suavizacao: f32, passo_px: f32) -> Option<Vec<[f32; 2]>> {
        let anel = self.anel(suavizacao, passo_px);
        (anel.len() >= 3).then_some(anel)
    }
}

impl super::Sculpt3dScene {
    /// O anel do corte em curso, para a moldura pintar.
    ///
    /// ⚠️ **Sai do campo do GESTO e nunca de um pedido** — pintar a partir do
    /// que o produto *vai* fazer, e não do que ele *já* guardou, é como um
    /// indicador passa a mostrar outra coisa que não a ferramenta.
    pub fn trim_previa(&self) -> Option<Vec<[f32; 2]>> {
        // ⭐ **A prévia é pintada com a MESMA suavização que o corte vai usar** —
        // senão o artista vê uma forma e a ferramenta corta outra.
        let g = self.trim.gesto.as_ref()?;
        g.previa(
            self.brush.trim_suavizacao,
            self.passo_do_corte_px(g.inicio()),
        )
    }
}

/// **Quantos segmentos um círculo de raio `r` px precisa.**
///
/// # ⛔⛔ Report do dono (2026-09-15): *«circle ficou com baixa resolução nas
/// próprias linhas do círculo»*
///
/// A 1.ª lei olhava só para a **FLECHA** — a distância da corda ao arco — e
/// exigia-a abaixo de meio pixel, o que dá `n ≥ π·√(r/(2·flecha))`. Ela está
/// certa sobre o que promete e **promete a coisa errada**: a `r = 250 px` são
/// `50` lados, e cada um mede `31 px` de RETA no ecrã. *A flecha é
/// sub-pixel e o olho vê a quebra de TANGENTE em cada vértice* — que é o que
/// uma silhueta sombreada mostra.
///
/// ⭐⭐⭐ **A segunda régua é o `passo_px`, e ela é GRÁTIS:** o prisma já subdivide
/// cada segmento do anel até à aresta da peça (`ph2d_trim::Resolucao::Ate`),
/// logo os pontos vão ser criados de qualquer maneira — **só que sobre as
/// CORDAS**. Gerá-los sobre o CÍRCULO custa exactamente o mesmo e entrega a
/// forma certa. ⇒ *a resolução do círculo é a da peça, como tudo o resto desde a
/// wave da lâmina tesselada.*
///
/// ⚠️ **As duas contam, e fica a MAIOR:** o `passo_px` pode vir enorme numa peça
/// grosseira, e aí é a flecha que impede o polígono de se ver.
fn segmentos_do_circulo(r: f32, passo_px: f32) -> usize {
    let pela_flecha = std::f32::consts::PI * (r / (2.0 * FLECHA_MAX_PX)).sqrt();
    let pelo_passo = if passo_px.is_finite() && passo_px > 0.0 {
        std::f32::consts::TAU * r / passo_px
    } else {
        0.0
    };
    // ⚠️ **O piso de `12` é a forma, não a precisão:** abaixo dele um raio
    // pequeno entregaria um polígono que o artista reconhece como polígono.
    // ⛔ E o tecto nomeia o recurso: a tampa é triangulada por corte de orelha,
    // que é `O(n²)`, e o anel inteiro vira paredes.
    (pela_flecha.max(pelo_passo).ceil() as usize).clamp(12, TECTO_DE_SEGMENTOS)
}

/// O tecto de pontos de um anel gerado.
///
/// ⚠️ **Ele nomeia o recurso: a tampa é triangulada por CORTE DE ORELHA, que é
/// `O(n²)`, e cada ponto do anel é uma coluna de paredes.** `4 096` pontos são
/// `16,8 M` de operações no pior caso da tampa — ainda abaixo de um décimo de
/// segundo, e numa operação que corre **uma vez por gesto**. ⭐ O caminho do
/// produto nunca lá chega: o `passo_px` sai da peça, logo a contagem é
/// `perímetro/aresta`, que é a mesma ordem da própria malha.
const TECTO_DE_SEGMENTOS: usize = 4_096;

/// Meio-pixel de flecha: metade do que o ecrã consegue mostrar.
const FLECHA_MAX_PX: f32 = 0.5;

impl Gesto {
    /// Onde o gesto começou — a régua do [`super::Sculpt3dScene::passo_do_corte_px`].
    pub(crate) fn inicio(&self) -> [f32; 2] {
        self.pontos[0]
    }
}

impl super::Sculpt3dScene {
    /// ⭐⭐⭐ **A ARESTA DA PEÇA, medida em PIXEIS do ecrã** — quantos pixéis vale
    /// o triângulo que a peça tem, visto de onde o artista está.
    ///
    /// # Porque ela existe
    ///
    /// O gesto é de ECRÃ e a densidade é de MUNDO, e o círculo precisa das duas:
    /// ele tem de ter tantos lados quantos o prisma vai criar de qualquer
    /// maneira ao subdividir as cordas. *Sem esta conversão o anel é gerado numa
    /// unidade e consumido noutra.*
    ///
    /// ⚠️ **A distância é a do CENTRO DA PEÇA, e não a do acerto:** o acerto pode
    /// não existir (o gesto começa fora da peça, espec §3), e a escala de um
    /// pixel em mundo varia tão pouco ao longo de uma peça que medi-la no centro
    /// dela é o valor honesto.
    pub(crate) fn passo_do_corte_px(&self, em: [f32; 2]) -> f32 {
        let Some(malha) = self.obj().map(|_| self.mesh()) else {
            return f32::INFINITY;
        };
        let alvo = super::trim_aplica::alvo_da_lamina(malha);
        let b = malha.bounds();
        let centro = [
            (b.min[0] + b.max[0]) * 0.5,
            (b.min[1] + b.max[1]) * 0.5,
            (b.min[2] + b.max[2]) * 0.5,
        ];
        let (r0, r1) = (self.ray_at(em[0], em[1]), self.ray_at(em[0] + 1.0, em[1]));
        let o = r0.origin();
        let d =
            ((centro[0] - o[0]).powi(2) + (centro[1] - o[1]).powi(2) + (centro[2] - o[2]).powi(2))
                .sqrt();
        let (p0, p1) = (r0.at(d), r1.at(d));
        let mundo_por_px =
            ((p0[0] - p1[0]).powi(2) + (p0[1] - p1[1]).powi(2) + (p0[2] - p1[2]).powi(2)).sqrt();
        if mundo_por_px > 0.0 {
            alvo / mundo_por_px
        } else {
            f32::INFINITY
        }
    }
}
