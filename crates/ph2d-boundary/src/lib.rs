#![forbid(unsafe_code)]
//! **O pincel de CONTORNO** — clean-room sob
//! `docs/3D/cleanroom/SPEC_boundary_brush.md` (atestada à 3.ª passagem do R-pré,
//! 2026-09-13).
//!
//! O artista aponta para perto de uma **borda aberta** da malha e arrasta; a
//! borda inteira — ou um troço dela — deforma-se, com a deformação a **esmorecer
//! para dentro** da peça.
//!
//! # O que esta crate é
//!
//! **Lei pura, zero dependências.** Ela não sabe o que é uma `Mesh`, um pincel,
//! uma câmera ou um evento de ponteiro: recebe posições, faces e controlos, e
//! devolve posições novas.
//!
//! # ⚠️⚠️ A arquitectura em fases, e porque ela é o que torna o pincel testável
//!
//! ```text
//! A  censo de bordas na malha            (uma vez por malha)   → Topologia
//! B  escolher o vértice âncora           (pen-down)            → pode RECUSAR
//! C  andar a borda a partir da âncora    (pen-down)            ┐
//! D  propagar para dentro                (pen-down)            ├ Estrutura
//! E  pesos por vértice                   (pen-down)            ┘
//! --- por passo do traço ---
//! F  avanço s a partir do arrasto
//! G  a lei do modo
//! H  translação, com filtro de peso zero e de região
//! ```
//!
//! ⭐ **A–E são FOTOGRAFADAS no pen-down e não se recalculam durante o traço.**
//! É isso que torna o resultado função só do **arrasto TOTAL** e não do caminho
//! nem do número de eventos — e o corpus do oráculo mede-o de duas maneiras
//! independentes (o mesmo arrasto em `2` e em `8` eventos dá o mesmo valor ao
//! 7.º decimal, e a prova de fatiamento fecha a `0,00000000`).
//!
//! ⛔ **Uma excepção, e ela é grande: o [`Modo::Suavizar`]** lê a posição
//! **actual** e por isso **acumula com o número de eventos**. Qualquer gate de
//! paridade dele tem de fixar a contagem de passos.
//!
//! # ⭐⭐ Onde superamos o alvo — e porque é uma lista, não uma frase
//!
//! | o que o alvo faz | o que custa | o nosso lado |
//! |---|---|---|
//! | quando o alcance excede a peça, o alcance `K` fica **maior** que o anel mais fundo que existe ⇒ os dados semeados no anel `K` ficam por preencher e o `EXPAND` deixa de deformar **por completo** | defeito ABERTO; o pincel parte-se quando é maior que a geometria | ⭐ **atamos `K` ao anel mais fundo que de facto existe** — as duas leis passam a estar bem definidas e o `EXPAND` deforma onde o alvo fica mudo ([`estrutura`], divergência **declarada** e com gate) |
//! | ignora a simetria radial e deforma em direcção diferente do traço | defeito ABERTO, *«provavelmente nunca funcionou»* | ⛔ **recusamos em voz alta** em vez de copiar o silêncio |
//! | o alisar é função do **número de eventos** | o mesmo gesto dá resultados diferentes com outra taxa de amostragem | declarado, e o alisar é **Jacobi** (independente da ordem de visita) |
//!
//! ⚠️ **O que NÃO fazemos:** mudar a lei por baixo das fixtures. A dependência
//! do alisar na taxa de eventos é um defeito real de previsibilidade *e* é o
//! comportamento que o artista conhece; trocá-la é um **modo**, com o seu
//! próprio gate, nunca uma correcção silenciosa.

pub mod ancora;
pub mod estrutura;
pub mod leis;
pub mod pesos;
pub mod topologia;
pub mod vetor;

#[cfg(test)]
mod leis_tests;

pub use ancora::Recusa;
pub use estrutura::Estrutura;
pub use pesos::Fatores;
pub use topologia::Topologia;
pub use vetor::V3;

/// A curva de queda do pincel, avaliada em `p ∈ [0, 1]`.
///
/// ⚠️ **Ela é um FECHO e não um ponteiro de função**, pela razão da irmã da
/// pose: a curva **autorada** pelo artista é uma tabela, e um `fn(f32) -> f32`
/// não a sabe exprimir.
pub type Curva<'a> = &'a dyn Fn(f32) -> f32;

/// As seis deformações (§10).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Modo {
    /// Roda a coluna à volta de um eixo **por coluna**, lido do anel `K`.
    #[default]
    Dobrar,
    /// Desliza ao longo de uma direcção **por coluna**.
    ///
    /// ⭐ Ele anda ao **contrário** da componente do avanço: puxar para fora da
    /// peça encolhe a borda para dentro.
    Expandir,
    /// Anda pela **normal de repouso** do próprio vértice.
    Inflar,
    /// Translada pelo **vector** do arrasto inteiro — o único que segue a mão.
    Agarrar,
    /// Roda à volta de **um** eixo, partilhado por toda a cadeia.
    Torcer,
    /// Média só com os vizinhos do **mesmo anel** — alisa ao LONGO do contorno.
    Suavizar,
}

impl Modo {
    /// Os seis, na ordem em que a UI os lista.
    pub const ALL: [Self; 6] = [
        Self::Dobrar,
        Self::Expandir,
        Self::Inflar,
        Self::Agarrar,
        Self::Torcer,
        Self::Suavizar,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Dobrar => "Bend",
            Self::Expandir => "Expand",
            Self::Inflar => "Inflate",
            Self::Agarrar => "Grab",
            Self::Torcer => "Twist",
            Self::Suavizar => "Smooth",
        }
    }

    /// ⭐⭐ **Este modo é conduzido pelo ARRASTO?**
    ///
    /// Cinco são; o [`Self::Suavizar`] não — ele usa a força directamente e por
    /// isso actua **já no primeiro passo**, quando os outros estão parados. *A
    /// prova é mais forte do que isso: as cinco fixtures de alisar do corpus têm
    /// arrasto ZERO e mesmo assim deformam.*
    pub fn segue_o_arrasto(self) -> bool {
        self != Self::Suavizar
    }
}

/// Os quatro modos de queda **ao longo do contorno** (§8.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum QuedaNoContorno {
    /// `1` em toda a cadeia.
    #[default]
    Constante,
    /// A curva do pincel sobre a distância de cadeia.
    Raio,
    /// A curva sobre uma onda **triangular** da distância de cadeia.
    Laco,
    /// O mesmo do [`Self::Laco`], com o sinal a inverter em lóbulos alternados
    /// — **«dois SIM, dois NÃO», deslocada de um**.
    LacoInvertido,
}

impl QuedaNoContorno {
    pub const ALL: [Self; 4] = [Self::Constante, Self::Raio, Self::Laco, Self::LacoInvertido];

    pub fn label(self) -> &'static str {
        match self {
            Self::Constante => "Constant",
            Self::Raio => "Radius",
            Self::Laco => "Loop",
            Self::LacoInvertido => "Loop and Invert",
        }
    }
}

/// O que o pincel lê (§15).
///
/// ⚠️ **O painel do alvo tem EXACTAMENTE quatro controlos próprios** — alvo de
/// deformação, tipo de deformação, queda no contorno e deslocamento da origem —
/// e nada mais; o resto vem dos ajustes gerais do pincel. *Um painel nosso com
/// um quinto knob próprio está a inventar, e um com três está a esconder.*
#[derive(Clone, Copy, Debug)]
pub struct Controlos {
    pub modo: Modo,
    pub queda_no_contorno: QuedaNoContorno,
    /// ⚠️⚠️ **Ele alonga a PROPAGAÇÃO e NÃO a queda no contorno** (§8.4) — há
    /// **dois** raios em jogo e confundi-los é o erro caro:
    ///
    /// | onde | valor |
    /// |---|---|
    /// | profundidade da propagação (e portanto `K` e o ponto-origem) | `R × (1 + deslocamento)` |
    /// | queda ao longo do contorno | **`R`**, sem deslocamento |
    ///
    /// Medido: deslocamento `1` leva os anéis de `5` para `9` e o deslocamento
    /// máximo do dobrar de `0,367` para `0,661`, enquanto o expandir **mantém**
    /// o máximo em `0,1` — *a lei do expandir não tem braço de alavanca, logo só
    /// a contagem de anéis se move.*
    pub deslocamento_da_origem: f32,
    /// O raio do pen-down — o que limita a busca da âncora, a queda no contorno
    /// e (via deslocamento) a propagação.
    pub raio_inicial: f32,
    /// O raio deste evento — o que entra no ângulo das duas leis que rodam.
    /// ⚠️ Com a pressão a modular o tamanho, os dois divergem dentro do mesmo
    /// traço (§10.7).
    pub raio_dinamico: f32,
    /// ⚠️ **Entra LINEARMENTE** (§9.2) — ⛔ nunca ao quadrado.
    pub forca: f32,
    /// `1` a menos que a simetria e o esbatimento estejam os dois ligados.
    pub esbatimento_da_simetria: f32,
    /// O modificador de inversão — ⚠️ ele **não** inverte a deformação: ele
    /// **encaixa o ângulo** em décimos (§9.3).
    pub encaixar_angulo: bool,
    /// Que eixos estão espelhados **nesta passagem** — lido pelo filtro de
    /// região (§12.2).
    pub simetria: [bool; 3],
}

impl Default for Controlos {
    fn default() -> Self {
        Controlos {
            modo: Modo::default(),
            queda_no_contorno: QuedaNoContorno::default(),
            deslocamento_da_origem: 0.0,
            raio_inicial: 0.25,
            raio_dinamico: 0.25,
            forca: 1.0,
            esbatimento_da_simetria: 1.0,
            encaixar_angulo: false,
            simetria: [false; 3],
        }
    }
}

/// O que muda de um passo para o outro.
pub struct Evento {
    /// `Δ` — o arrasto **TOTAL** desde o pen-down, em espaço de objecto.
    ///
    /// ⚠️ Não é o incremento desde o evento anterior: o ponto que o editor usa
    /// para «onde está o pincel» fica **preso** no contacto inicial durante todo
    /// o traço.
    pub arrasto: V3,
}

/// Um traço de contorno, do pen-down ao fim.
#[derive(Clone, Debug)]
pub struct Contorno {
    estrutura: Estrutura,
    pesos: Vec<f32>,
    colunas: leis::PorColuna,
    /// ⚠️⚠️ **A direcção contra a qual o arrasto é projectado — e ela NÃO é a
    /// que a espec §9.1 escreve.**
    ///
    /// A espec diz *«o unitário do ponto de contacto para o ponto-origem»*.
    /// **Quatro fixturas do corpus refutam-no**, e a que o faz de forma mais
    /// limpa é a `grade_pequena_dobrar_origem0`: ali a peça é tão pequena que a
    /// propagação **atravessa o centro** e o ponto-origem sai com `y > 0`,
    /// enquanto o contacto tem `y < 0` — as duas fórmulas ficam **opostas**, e a
    /// nossa dava o deslocamento certo em módulo (`0,185410` nos dois lados) com
    /// o sentido invertido (erro de `0,353`, que é o dobro).
    ///
    /// **Resolvido do corpus** (o `s` do alvo lê-se de um vértice do anel `0`,
    /// onde o peso é `1`):
    ///
    /// | fixtura | `s` do alvo | `contacto → origem` | **`−unitário(origem)`** |
    /// |---|---|---|---|
    /// | `grade_expandir_constante` | `−0,100000` | `−0,100000` | `−0,100000` |
    /// | `grade_expandir_…_para_dentro` | `+0,100000` | `+0,100000` | `+0,100000` |
    /// | `tubo_expandir_constante` | `−0,080000` | `−0,100000` ✗ | `−0,080000` |
    /// | `cupula_expandir_constante` | `+0,038268` | `−0,098019` ✗ | `+0,038268` |
    ///
    /// ⇒ **a lei é o unitário de `−ponto_origem`**, e bate ao `1e-6` nas quatro.
    ///
    /// ⚠️⚠️ **Porque é que a espec pôde dizer outra coisa e continuar
    /// atestada:** o corpus mede o sinal do avanço com a família `sinal_*`, que
    /// vive toda na **grelha grande** — e ali o contacto e o ponto-origem estão
    /// os dois sobre o eixo `−y`, logo **as duas fórmulas coincidem**. *Um
    /// corpus é uma amostra do comportamento do alvo, nunca uma prova da lei; e
    /// uma fórmula medida numa família que não a discrimina é uma leitura, não
    /// uma medição.*
    ///
    /// ⛔⛔ **E isto é um DEFEITO do alvo, não uma sutileza:** um ponto está a
    /// ser usado onde uma direcção pertence, logo **o resultado depende de onde
    /// está a origem do objecto** — mover a peça na cena muda a deformação.
    /// Reproduzimo-lo para ter paridade, e é o sítio nomeado onde superá-lo é
    /// ganhar (§20 do handoff).
    direccao_do_avanco: V3,
    /// A média das `P₀` de **todos** os vértices da cadeia — o centro do torcer.
    centro_do_torcer: V3,
    contacto: V3,
}

impl Contorno {
    /// As fases **B–E**, de uma vez.
    ///
    /// `sob_o_cursor` é o vértice mais próximo do ponto de contacto — ⚠️ e é ele
    /// (e **não** a âncora) o sujeito das duas recusas (§5.2).
    #[allow(clippy::too_many_arguments)]
    pub fn comecar(
        topo: &Topologia,
        repouso: &[V3],
        normais: &[V3],
        escondido: &[bool],
        sob_o_cursor: u32,
        contacto: V3,
        ctrl: &Controlos,
        curva: Curva<'_>,
        fatores: Fatores<'_>,
    ) -> Result<Contorno, Recusa> {
        let ancora = ancora::escolher(topo, repouso, escondido, sob_o_cursor, ctrl.raio_inicial)?;
        let raio_prop = ctrl.raio_inicial * (1.0 + ctrl.deslocamento_da_origem);
        let estrutura = estrutura::construir(topo, repouso, escondido, ancora, raio_prop);
        let mut pesos = Vec::new();
        pesos::construir(
            &estrutura,
            ctrl.queda_no_contorno,
            ctrl.raio_inicial,
            curva,
            fatores,
            &mut pesos,
        );
        let colunas = leis::PorColuna::construir(&estrutura, repouso, normais);
        // ⚠️ Ver o campo `direccao_do_avanco`: é o unitário de **`−ponto_origem`**,
        // e não o do contacto para o ponto-origem que a espec escreve.
        let direccao_do_avanco =
            vetor::normalizar(vetor::escalar(estrutura.ponto_origem, -1.0)).unwrap_or([0.0; 3]);
        let centro_do_torcer = centro(&estrutura, repouso);
        Ok(Contorno {
            estrutura,
            pesos,
            colunas,
            direccao_do_avanco,
            centro_do_torcer,
            contacto,
        })
    }

    /// As fases **F–H**: um passo do traço. Devolve quantos vértices se moveram.
    ///
    /// ⚠️ `repouso` são as posições do **início do traço** e `posicoes` as
    /// vivas — as duas são precisas, porque cinco dos seis modos partem do
    /// repouso e o alisar parte da posição actual.
    pub fn passo(
        &self,
        topo: &Topologia,
        ctrl: &Controlos,
        ev: &Evento,
        repouso: &[V3],
        normais: &[V3],
        posicoes: &mut [V3],
    ) -> usize {
        leis::aplicar(
            &self.estrutura,
            topo,
            ctrl,
            ev,
            &self.pesos,
            &self.colunas,
            repouso,
            normais,
            self.direccao_do_avanco,
            self.centro_do_torcer,
            self.contacto,
            posicoes,
        )
    }

    pub fn estrutura(&self) -> &Estrutura {
        &self.estrutura
    }

    pub fn pesos(&self) -> &[f32] {
        &self.pesos
    }

    /// ⭐ **O que a pré-visualização desenha** (§17): a âncora e o ponto-origem.
    /// O segmento entre os dois é o que diz ao artista **até onde a deformação
    /// vai chegar** — e essa profundidade não se lê do cursor.
    pub fn linha_da_profundidade(&self, repouso: &[V3]) -> (V3, V3) {
        (
            repouso
                .get(self.estrutura.ancora as usize)
                .copied()
                .unwrap_or([0.0; 3]),
            self.estrutura.ponto_origem,
        )
    }
}

/// §10.5 — a média das `P₀` de **todos** os vértices da cadeia.
fn centro(e: &Estrutura, repouso: &[V3]) -> V3 {
    if e.cadeia.is_empty() {
        return [0.0; 3];
    }
    let mut soma = [0.0f32; 3];
    for &v in &e.cadeia {
        soma = vetor::add(soma, repouso.get(v as usize).copied().unwrap_or([0.0; 3]));
    }
    vetor::escalar(soma, 1.0 / e.cadeia.len() as f32)
}

/// A curva de omissão do corpus — a suave (`3p² − 2p³`).
pub fn suave(p: f32) -> f32 {
    p * p * (3.0 - 2.0 * p)
}
