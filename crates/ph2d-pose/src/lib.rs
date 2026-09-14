//! **O pincel de POSE** — clean-room sob `docs/3D/cleanroom/SPEC_pose_brush.md`
//! (atestada à 5.ª passagem do R-pré, 2026-09-13).
//!
//! O artista aponta o cursor a um ponto da malha e **arrasta**. O pincel acha
//! sozinho — **pela forma e pela ligação da malha**, não por um esqueleto — um
//! **pivô**, como se houvesse uma articulação enterrada ali, e uma **cadeia de
//! segmentos** encostada a ele; a região à volta **roda** em torno desse pivô
//! acompanhando o arrasto. Com mais de um segmento, a cadeia dobra como um
//! braço.
//!
//! # O que esta crate é
//!
//! **Lei pura, zero dependências.** Ela não sabe o que é uma `Mesh`, um pincel,
//! uma câmera ou um evento de ponteiro: recebe posições, faces e controlos, e
//! devolve posições. É isso que a torna gateável contra as `69` fixturas do
//! oráculo sem GPU, sem janela e sem escultura.
//!
//! # ⭐⭐ Onde superamos o alvo — e porque é que isso é uma lista e não uma frase
//!
//! Reproduzir vem primeiro, e não por burocracia: **sem paridade medida, uma
//! diferença é indistinguível de um defeito**. Com ela, cada divergência abaixo
//! é deliberada, tem número, e tem gate.
//!
//! | o que o alvo faz | o que custa | o nosso lado |
//! |---|---|---|
//! | reconstrói a cadeia inteira **a cada movimento do rato**, sem traço nenhum, só para desenhar o indicador | o editor engasga em malha densa; com peças desligadas cada zoom paga `O(V²)` — quatro relatos públicos | a cadeia constrói-se **uma vez por traço** ([`Pose::comecar`]) e é reutilizada; o indicador **não** a reconstrói |
//! | a suavização **pode** não ser reprodutível acima de uma partição da estrutura de aceleração | o mesmo gesto poderia dar resultados diferentes | Jacobi limpo ⇒ **determinístico em toda a malha** ([`pesos::suavizar`]). ⚠️ A reserva viaja com a afirmação: esse regime é um **risco do mecanismo** e **nunca foi observado** |
//! | a auto-suavização só age dentro do **raio inicial**, enquanto a deformação alcança muito mais longe | o efeito «desaparece» longe do cursor | se a oferecermos aqui, ela segue **os pesos**, não o raio |
//! | pivô em cima do cursor ⇒ **não faz nada, em silêncio** | o artista arrasta e não acontece nada | o caso é detectável ([`Pose::inerte`]) e deve ser **avisado** |
//! | vértice solto recebe a média de um **conjunto vazio** ⇒ peso indefinido | malha com vértices soltos é imprevisível | o peso **mantém-se** (§11.4, divergência declarada) |
//! | emparelhamento de peças **ganancioso por índice** ⇒ picos e deformação inconsistente | defeito público do alvo | reproduzido para paridade, e **nomeado** como o sítio onde trocar a lei ganha — ⛔ nunca em silêncio |
//!
//! ⚠️ **O que NÃO fazemos:** mudar a lei por baixo das fixturas. A dependência
//! da **taxa de eventos** (§5.1-bis) é o exemplo — ela é um defeito real de
//! previsibilidade *e* é o comportamento que o artista conhece; trocá-la é um
//! **modo**, com o seu próprio gate, nunca uma correcção silenciosa.

pub mod aplicar;
pub mod cadeia;
pub mod mapas;
pub mod pesos;
pub mod solver;
pub mod vetor;
pub mod vizinhanca;

#[cfg(test)]
mod leis_tests;

pub use aplicar::Fatores;
pub use cadeia::{Cadeia, Segmento};
pub use solver::Evento;
pub use vetor::{Rot, V3};
pub use vizinhanca::Vizinhanca;

/// Qual das três deformações o pincel faz (§0).
///
/// ⚠️ O modificador de inversão (Ctrl, ou a ponta invertida da caneta) **troca
/// de deformação** em vez de trocar o sinal da força.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Modo {
    #[default]
    GirarTorcer,
    EscalarTransladar,
    EspremerEsticar,
}

impl Modo {
    /// Os três, na ordem em que a UI os lista — ⚠️ **e é a mesma ordem em que o
    /// alvo os expõe**, porque a correspondência com as fixturas é
    /// **posicional** (o cabeçalho de cada uma traz o nome do nosso lado).
    pub const ALL: [Self; 3] = [
        Self::GirarTorcer,
        Self::EscalarTransladar,
        Self::EspremerEsticar,
    ];

    /// O nome que a UI mostra.
    ///
    /// ⚠️ **Os dois lados de cada barra são o gesto normal e o gesto com o
    /// modificador de inversão**, e não duas ferramentas: o Ctrl **troca de
    /// deformação** em vez de trocar o sinal da força. Escrever só o primeiro
    /// esconderia metade do pincel.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::GirarTorcer => "Rotate / Twist",
            Self::EscalarTransladar => "Scale / Translate",
            // ⚠️ Aqui o modificador **não muda nada**, e está medido: a fixtura
            // invertida é idêntica ao bit à normal. A barra fica porque o alvo
            // nomeia as duas metades assim.
            Self::EspremerEsticar => "Squash / Stretch",
        }
    }

    /// §3 — ⚠️ **nos modos de escala e de espremer/esticar a cadeia tem
    /// exactamente UM segmento**, seja qual for o valor do controlo.
    ///
    /// *Observado:* as fixturas que pedem `3` segmentos no cabeçalho nesses
    /// modos produzem a mesma cadeia de um segmento que as de `1`.
    pub fn cadeia_de_um_segmento(self) -> bool {
        matches!(self, Modo::EscalarTransladar | Modo::EspremerEsticar)
    }
}

/// A deformação efectiva, já resolvido o modificador de inversão.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Deformacao {
    Rodar,
    Torcer,
    Escalar,
    Transladar,
    Espremer,
}

/// A curva de atenuação, avaliada em `[0,1]`.
///
/// ⭐⭐ **Ela entra no EVENTO, não nos controlos, e isso é uma decisão de
/// desenho:** o vocabulário das curvas já vive na crate que o possui
/// (`ph2d-sculpt3d::falloff`, doze leis) e copiá-lo para aqui seria escrever a
/// mesma lei em dois sítios. ⚠️ **E um ponteiro de função não chegava:** uma
/// das doze é a curva **AUTORADA pelo artista**, que é dado e não código — ela
/// não cabe num `fn(f32) -> f32`. *Uma API que só sabe exprimir onze das doze
/// opções entrega a décima segunda como silêncio.*
///
/// ⛔ O argumento é `1 − i/n` (o **índice do segmento**, §5.2), e essa lei fica
/// aqui: quem passa a curva não tem de saber onde ela é amostrada.
pub type Curva<'a> = &'a dyn Fn(f32) -> f32;

/// A curva de omissão do pincel: `3p² − 2p³`.
///
/// ⚠️ Existe aqui **só** para os testes e para quem não tem curva própria.
/// ⛔ Não acrescente as outras onze: elas têm dono.
pub fn suave(p: f32) -> f32 {
    p * p * (3.0 - 2.0 * p)
}

/// Os controlos que o pincel lê (§1).
#[derive(Clone, Copy, Debug)]
pub struct Controlos {
    pub modo: Modo,
    /// `1..20`.
    pub segmentos: u32,
    /// `0..2` — afasta o pivô do cursor, em múltiplos do raio.
    pub desvio_da_origem: f32,
    /// `0..100`.
    pub suavizacoes_do_peso: u32,
    /// Prende a extremidade distante da cadeia (§5.1).
    pub ancorado: bool,
    /// No modo de escala, não roda antes de escalar (§5.4).
    pub trava_rotacao: bool,
    pub raio: f32,
    /// ⚠️ Entra **linearmente**, não ao quadrado.
    pub forca: f32,
    pub invertido: bool,
    pub simetria: [bool; 3],
    /// Ligado, a travessia não atravessa peças desligadas (§2.4).
    pub so_conectado: bool,
    pub distancia_max_entre_pecas: f32,
}

impl Default for Controlos {
    fn default() -> Self {
        Controlos {
            modo: Modo::default(),
            segmentos: 1,
            desvio_da_origem: 0.0,
            suavizacoes_do_peso: 4,
            // ⭐ **RECOMENDAÇÃO NOSSA, declarada como tal** (§1.4): nascer
            // ancorado, porque é a configuração em que o gesto roda em torno de
            // um pivô fixo — que é o que o nome do pincel promete.
            // ⛔ **Não é uma observação do alvo:** qual dos dois o artista lá
            // encontra **não foi medido**, e a proveniência que a espec tinha
            // era circular (o cabeçalho da fixtura é ENTRADA do harness).
            ancorado: true,
            trava_rotacao: false,
            raio: 0.25,
            forca: 1.0,
            invertido: false,
            simetria: [false; 3],
            so_conectado: true,
            distancia_max_entre_pecas: 0.1,
        }
    }
}

impl Controlos {
    pub fn deformacao(&self) -> Deformacao {
        match (self.modo, self.invertido) {
            (Modo::GirarTorcer, false) => Deformacao::Rodar,
            (Modo::GirarTorcer, true) => Deformacao::Torcer,
            (Modo::EscalarTransladar, false) => Deformacao::Escalar,
            (Modo::EscalarTransladar, true) => Deformacao::Transladar,
            // ⚠️ O modificador **não muda a saída** neste modo, e está medido:
            // a fixtura invertida é idêntica **ao bit** à normal.
            (Modo::EspremerEsticar, _) => Deformacao::Espremer,
        }
    }
}

/// Um traço de pose, do primeiro evento ao último.
#[derive(Clone, Debug)]
pub struct Pose {
    cadeia: Cadeia,
    mapas: Vec<mapas::Mapa>,
}

impl Pose {
    /// **Primeiro evento** (§10): fixa-se o ponto de aplicação e o raio em
    /// espaço de objecto, e **constrói-se a cadeia inteira** (§2–§4).
    ///
    /// ⚠️ `eleito` é o vértice que a amostragem do ponteiro elege sob o cursor.
    /// Ele é uma grandeza **distinta** do mais-próximo global (§2.1) — as duas
    /// coincidem em todas as fixturas publicadas, mas **não por construção**.
    pub fn comecar(
        viz: &Vizinhanca,
        posicoes: &[V3],
        escondido: &[bool],
        eleito: u32,
        cursor: V3,
        ctrl: &Controlos,
    ) -> Pose {
        let mut cadeia = cadeia::construir(viz, posicoes, escondido, eleito, cursor, ctrl);
        let n = cadeia.n_vertices;
        // §4 — cada segmento leva as suavizações **independentemente**.
        for i in 0..cadeia.segmentos.len() {
            pesos::suavizar(
                viz,
                &mut cadeia.pesos[i * n..(i + 1) * n],
                ctrl.suavizacoes_do_peso,
            );
        }
        Pose {
            cadeia,
            mapas: Vec::new(),
        }
    }

    /// Cada evento: actualiza-se o arrasto, resolve-se a cadeia (§5) e os mapas
    /// (§6) passam a valer para o novo deslocamento.
    pub fn evento(&mut self, ctrl: &Controlos, ev: &Evento, curva: Curva<'_>) {
        solver::resolver(&mut self.cadeia, ctrl, ev, curva);
        self.mapas = mapas::construir(&self.cadeia, ctrl);
    }

    /// Escreve a posição de cada vértice **depois** deste evento (§7).
    pub fn posicoes(&self, ctrl: &Controlos, p0: &[V3], fatores: Fatores<'_>, saida: &mut Vec<V3>) {
        aplicar::posicoes_finais(&self.cadeia, ctrl, &self.mapas, p0, fatores, saida);
    }

    pub fn cadeia(&self) -> &Cadeia {
        &self.cadeia
    }

    /// ⭐ **O que um indicador desenha** — ver [`Cadeia::ossos`].
    ///
    /// ⚠️ **A mesma porta serve o indicador ANTES de premir e o osso VIVO
    /// durante o arrasto**, e é isso que garante que os dois não divergem: em
    /// repouso a cadeia ainda não foi resolvida e os mapas são a identidade, e
    /// a mesma expressão devolve o par inicial. *Duas funções — uma «em
    /// repouso» e outra «a mexer» — são duas respostas à mesma pergunta, e a
    /// que o artista vê é a que envelhece.*
    pub fn ossos(&self, ctrl: &Controlos, saida: &mut Vec<[V3; 2]>) {
        self.cadeia.ossos(ctrl, saida);
    }

    /// ⭐ **O caso §11.1, detectável em vez de silencioso.**
    ///
    /// Se a franja fizer o pivô coincidir com o ponto de aplicação, o primeiro
    /// segmento nasce com comprimento nulo e o pincel **não move nada em toda a
    /// malha** — medido no alvo: `movidos = 0` apesar de um arrasto de `0,6`.
    ///
    /// ⚠️ **É um caso de produto, não um crash.** O alvo cala-se; nós podemos
    /// avisar, e é isso que esta porta existe para permitir.
    pub fn inerte(&self) -> bool {
        self.cadeia
            .segmentos
            .first()
            .is_none_or(|s| s.direccao_inicial().is_none())
    }
}
