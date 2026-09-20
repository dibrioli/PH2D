//! ⭐⭐ **AS TRÊS FAMÍLIAS DE LEITURA que a UI pergunta** — irmão (`#[path]`) do
//! [`super::brush_verb_predicados`], e o corte é o SUJEITO: lá mora *o que um
//! verbo É*, e aqui *a que família de LEITURA ele pertence* — que superfície ele
//! consulta antes de decidir para onde puxar.
//!
//! São exactamente as três que o gate
//! `the_families_that_the_ui_asks_about_agree_with_the_verb_list` enumera: a
//! **máscara**, o **plano** e o **anel**.
//!
//! ⚠️ **O corte foi FORÇADO pelo tecto de LOC** (o ficheiro-mãe chegou a `725` de
//! `700` ao registar porque o pincel de plano entrou na família do plano) e é
//! melhor por isso: as três respostas que um mesmo gate compara passam a viver
//! num sítio onde se leem juntas, em vez de espalhadas por uma lista de vinte
//! predicados. ⛔ *Subir o número em vez de cortar é o que o `CLAUDE.md` §2
//! proíbe por escrito.*

use super::verb::Verb;

impl Verb {
    /// Este verbo escreve na MÁSCARA em vez da posição?
    ///
    /// Porta única: o aplicador pergunta para saber onde escrever, e a UI
    /// perguntará para saber que knobs oferecer. Duas listas divergiriam no dia
    /// em que entrar o segundo verbo de canal (Paint, na W7).
    #[must_use]
    pub fn paints_mask(self) -> bool {
        matches!(self, Self::Mask)
    }

    /// Este verbo escreve na COR em vez da posição?
    ///
    /// ⭐ **O segundo verbo de canal chegou** (2026-09-19, o [`Self::Paint`]), e
    /// o doc da irmã acima previa-o por escrito — *«duas listas divergiriam no
    /// dia em que entrar o segundo verbo de canal»*. ⇒ a pergunta que o resto
    /// do motor faz não é nenhuma das duas: é [`Self::escreve_um_canal`].
    #[must_use]
    pub fn paints_color(self) -> bool {
        matches!(self, Self::Paint | Self::Blur | Self::SmearColor)
    }

    /// ⭐⭐ **Este verbo escreve um CANAL em vez de mover barro?**
    ///
    /// ⚠️ **É esta a pergunta que o motor faz**, e não *«qual canal»*: quem
    /// decide se o dab refresca normais, se o `Ctrl+Z` fotografa geometria e se
    /// o aplicador de posições corre está a perguntar se alguma **posição**
    /// mudou — e a resposta é a mesma para os dois canais. Deixar cada sítio
    /// perguntar `paints_mask()` foi o que fez a chegada da cor tocar em cada
    /// um deles.
    #[must_use]
    pub fn escreve_um_canal(self) -> bool {
        self.paints_mask() || self.paints_color()
    }

    /// Este verbo ajusta um plano à pegada do dab? (Quem responde `true` usa o
    /// knob `plane_offset`.)
    ///
    /// # ⛔⛔⛔ O [`Self::Plane`] ficou FORA — e ele entrou e saiu no mesmo dia
    ///
    /// Ele **lê** o `plane_offset` (o [`crate::plano_da_pegada`] faz
    /// `lift = raio × plane_offset`, espec §2.4, com duas fixturas a
    /// exercitá-lo), e em 2026-09-17 eu pu-lo aqui a chamar-lhe *controlo
    /// inalcançável*. ⛔ **O smoke do dono devolveu-o na mesma hora:**
    /// *«Plane Offset com resultado completamente errado»*, com foto da peça
    /// destruída.
    ///
    /// ⭐⭐ **Medido depois, na peça da cena `=47`** (bossas de amplitude `0,09`,
    /// raio do pincel `0,35`, oito dabs, valores de fábrica do verbo — tectos
    /// `1/0`, do perfil *aparar*):
    ///
    /// | deslocamento | vértices movidos | corte máximo |
    /// |---|---|---|
    /// | `−0,50` | `739` | **`0,3552`** (`2,2×`) |
    /// | `−0,20` | `596` | `0,2732` |
    /// | `−0,10` | `456` | `0,2320` |
    /// | **`0`** | `380` | `0,1633` |
    /// | `+0,10` | `333` | `0,1216` |
    /// | `+0,20` | `216` | `0,0828` |
    /// | `+0,50` | **`2`** | **`0,0007`** — inerte |
    ///
    /// ⇒ **a lei está certa e o knob é MONÓTONO; o que não serve é a FAIXA.** A
    /// fileira herdaria o `−1 … +1` dos quatro verbos da casa, e nessa faixa
    /// **metade do curso é inerte** (a partir de `+0,5` o plano já limpou o
    /// relevo inteiro) e a outra metade **dobra o corte por passagem** — que é a
    /// foto do dono, depois de algumas passagens.
    ///
    /// ⚠️ **A assimetria é dos TECTOS e não do knob**, e isso está medido: com os
    /// tectos bilaterais (`1/1`, o perfil *achatar*) o `+0,50` move `478`
    /// vértices e corta `0,1981`. *Com um tecto de um lado só, afastar o plano
    /// do material deixa de ter material para tirar.*
    ///
    /// ⛔⛔ **E a minha leitura de «inalcançável» estava ERRADA no sentido que
    /// importa:** a ausência da fileira era **certa** e o que lhe faltava era um
    /// MOTIVO ESCRITO. *Curei a falta de documentação criando um controlo que
    /// destrói a peça* — e um censo de knobs mortos não me podia proteger disso,
    /// porque ele mede se o knob MOVE o barro e este move de mais.
    ///
    /// ⏳ **Para o reabrir falta um NÚMERO:** a faixa teria de sair de um recurso
    /// medido, e o que a limita aqui é a **altura do relevo em raios de pincel**,
    /// que é da peça e não do produto. ⛔ A faixa do alvo para este controlo
    /// **neste pincel** nunca foi medida (a §14.2 publica só o valor de fábrica,
    /// `0`) — *e escolher um número sem isso é o palpite que o §0.0 proíbe.*
    /// Enquanto isso, a capacidade continua alcançável por script e pelas
    /// fixturas, e o gate `o_deslocamento_do_plano_nao_e_oferecido_ao_pincel_de_plano`
    /// afirma as duas metades.
    pub fn uses_plane(self) -> bool {
        matches!(self, Self::Flatten | Self::Fill | Self::Scrape | Self::Clay)
    }

    /// Este verbo lê o anel de vizinhos? (Quem responde `true` custa a
    /// travessia do CSR por vértice, e é o que decide se o `vert_verts` pode um
    /// dia virar preguiçoso.)
    ///
    /// ⚠️ **O [`Self::SurfaceSmooth`] o percorre DUAS vezes** — uma para a média
    /// das posições, outra para a média dos `b` —, e ele nasceu FORA desta
    /// lista: a pergunta que ela responde é *quem precisa da adjacência*, e uma
    /// resposta falsa aqui é como um `vert_verts` preguiçoso deixaria de
    /// construí-la exatamente para o verbo que mais a usa. O gate irmão
    /// `the_families_that_the_ui_asks_about_agree_with_the_verb_list` enumera
    /// os nomes, e ficou VERDE sobre a omissão até alguém a procurar.
    #[must_use]
    pub fn uses_neighbours(self) -> bool {
        matches!(
            self,
            Self::Smooth | Self::Sharpen | Self::SurfaceSmooth | Self::SmearMultires
        )
    }

    /// **Este verbo lê a COR DA VIZINHANÇA em vez de depositar a sua?** — o
    /// [`Self::Blur`] e o [`Self::SmearColor`], contra o [`Self::Paint`].
    ///
    /// ⚠️ **Ela existe porque o conjunto estava escrito à mão em DOIS sítios
    /// como `paints_color() && self != Paint`** — a lei de acumulação
    /// ([`Self::grip_law`]) e o arnês do censo dos knobs — e o terceiro
    /// chamador chegou no mesmo dia. *Uma lei escrita em dois sítios ainda não
    /// é uma lei.*
    ///
    /// ⛔⛔ **E o consumidor que a obrigou é uma FIXTURA:** um censo que corra
    /// estes dois sobre uma peça de cor UNIFORME lê `0,000` em toda a linha —
    /// a média de branco é branco, e o transporte de branco também — e acusa
    /// de mortos os knobs de um pincel impecável. *Um corpus no ponto NEUTRO
    /// de um canal não testa esse canal*, e a porta é o que diz a quem monta a
    /// peça que ela precisa de ter cor a VARIAR.
    #[must_use]
    pub const fn le_o_anel_de_cor(self) -> bool {
        matches!(self, Self::Blur | Self::SmearColor)
    }

    /// **Este verbo deposita a COR DO PINCEL?** — o [`Self::Paint`], e só ele.
    ///
    /// ⚠️ **DERIVADA das duas portas que já existem, e não uma terceira lista:**
    /// pinta cor **e** não lê o anel. Os dois que leem o anel puxam a cor da
    /// VIZINHANÇA — o `Brush::color` não entra na lei deles em sítio nenhum —,
    /// logo oferecer-lhes a cor do pincel seria um controlo morto, que é a
    /// espécie que o censo dos knobs varre a cada wave.
    #[must_use]
    pub fn deposita_a_cor_do_pincel(self) -> bool {
        self.paints_color() && !self.le_o_anel_de_cor()
    }

    /// **Este verbo SUBTRAI a normal do puxão?** — os dois gestos TANGENCIAIS
    /// ([`crate::stroke_normal_do_gesto`]), cujo alvo é `Δ − n·(n·Δ)`.
    ///
    /// ⚠️ **Ela existe porque uma porta de PRODUTO precisou dela, e a medição é
    /// que a nomeou:** a opção [`crate::Brush::puxa_pela_normal`] é
    /// estruturalmente **inerte** nestes dois — pôr o puxão ao longo de `n` e
    /// depois subtrair a componente ao longo de `n` deixa **zero**. Medido no
    /// barro: o polegar move `6e-5` e o empurrão *perde* `0,027` (a opção
    /// desligava-o).
    ///
    /// ⛔ O [`Self::DrawSharp`] lê a mesma normal e **não** entra aqui: ele é um
    /// carimbo, não tem puxão — e é o `grip` que o separa.
    #[must_use]
    pub const fn subtrai_a_normal_do_puxao(self) -> bool {
        matches!(self, Self::Thumb | Self::Nudge)
    }
}
