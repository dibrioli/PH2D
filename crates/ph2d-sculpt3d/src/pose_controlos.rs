//! Os controlos **próprios** do pincel de pose, e a porta que os junta aos
//! partilhados para formar a lei.
//!
//! ⚠️⚠️ **A divisão entre «próprio» e «partilhado» é load-bearing, e é a espec
//! que a fixa:** o raio, a força, a curva de atenuação, a máscara, a simetria e
//! o *Connected Only* **já existem no pincel** e este verbo lê-os como os outros
//! — inventar um segundo raio aqui daria ao artista dois números que fazem a
//! mesma coisa e discordam. O que é próprio são os **seis** que nenhum outro
//! verbo tem.

use crate::Brush;

/// Os seis controlos que só o [`crate::Verb::Pose`] lê.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PoseControlos {
    pub modo: ph2d_pose::Modo,
    /// `1..20` — quantos segmentos a cadeia tem. Com mais de um ela dobra como
    /// um braço.
    pub segmentos: u32,
    /// `0..2` — afasta o pivô do cursor, em múltiplos do raio.
    pub desvio_da_origem: f32,
    /// `0..100`.
    pub suavizacoes_do_peso: u32,
    /// Prende a extremidade distante da cadeia.
    pub ancorado: bool,
    /// No modo de escala, escala **sem rodar**.
    pub trava_rotacao: bool,
    /// O arrasto do ponteiro em **pixels de ecrã**, no eixo horizontal, desde o
    /// pen-down.
    ///
    /// ⚠️⚠️ **É o único número deste pincel que vem do ECRÃ, e por isso o único
    /// que mora aqui apesar de não ser um controlo do artista.** O modo de
    /// torção lê pixels (espec §5.2) ⇒ a saída dele depende da **resolução e do
    /// zoom**, e a conta que os produz precisa da câmera, que vive na app. ⛔ Um
    /// `Dab` não carrega nada de ecrã, e alargá-lo tocaria todos os 27 verbos
    /// para servir um.
    ///
    /// ⭐ **E isto é um candidato NOMEADO a superar o alvo:** *o mesmo gesto dá
    /// torções diferentes conforme o zoom*, que é a família de defeitos «o
    /// resultado depende de algo em que o artista não está a pensar». ⛔ Trocá-lo
    /// é um MODO com gate próprio, nunca uma correcção silenciosa — as fixturas
    /// de torção medem esta lei.
    pub arrasto_x_pixels: f32,
}

impl Default for PoseControlos {
    fn default() -> Self {
        let lei = ph2d_pose::Controlos::default();
        PoseControlos {
            modo: lei.modo,
            segmentos: lei.segmentos,
            desvio_da_origem: lei.desvio_da_origem,
            suavizacoes_do_peso: lei.suavizacoes_do_peso,
            // ⭐ Os dois defaults vêm da crate da LEI, onde a recomendação está
            // declarada **como nossa** — ⛔ e não como observação do alvo, cuja
            // proveniência era circular (o cabeçalho da fixtura é entrada do
            // harness, não uma leitura do que o artista encontra).
            ancorado: lei.ancorado,
            trava_rotacao: lei.trava_rotacao,
            arrasto_x_pixels: 0.0,
        }
    }
}

impl PoseControlos {
    /// Junta os próprios aos partilhados e devolve a lei.
    ///
    /// ⚠️ **A força entra LINEARMENTE** neste pincel (espec §1.2) — ⛔ nunca ao
    /// quadrado, que é a curva do modo de referência dos outros verbos. *Este
    /// repo já pagou essa confusão uma vez, num corpus inteiro a força cheia
    /// onde `s`, `s²` e `s⁴` coincidem.*
    #[must_use]
    pub fn lei(&self, brush: &Brush) -> ph2d_pose::Controlos {
        ph2d_pose::Controlos {
            modo: self.modo,
            segmentos: self.segmentos,
            desvio_da_origem: self.desvio_da_origem,
            suavizacoes_do_peso: self.suavizacoes_do_peso,
            ancorado: self.ancorado,
            trava_rotacao: self.trava_rotacao,
            raio: brush.radius,
            forca: brush.strength,
            // O modificador de inversão **troca de deformação**, não de sinal.
            invertido: brush.invert,
            simetria: [false; 3],
            // ⭐ O *Connected Only* do pincel **é** o «só conectado» da espec:
            // a mesma pergunta (a travessia atravessa peças desligadas?), e o
            // artista já o conhece pelo carimbo. Um segundo controlo com o
            // mesmo sentido seria a dupla fonte de verdade de sempre.
            so_conectado: brush.surface_only,
            distancia_max_entre_pecas: ph2d_pose::Controlos::default().distancia_max_entre_pecas,
        }
    }
}
