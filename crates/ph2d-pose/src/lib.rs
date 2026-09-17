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

/// ⭐⭐⭐ **QUANTO DO ARRASTO A ESCALA LÊ** — a decisão do dono de 17/09
/// (*«cada modo com opção, com um botão para mudar o modo»*).
///
/// # Quem lê isto, e quem NÃO lê
///
/// Só as duas deformações que passam pelo **quociente de escala** —
/// [`Deformacao::Escalar`] e [`Deformacao::Espremer`] (§5.4 e §5.5). ⛔ As
/// outras três **não** projectam nada: a [`Deformacao::Transladar`] soma o
/// deslocamento **inteiro** à origem de cada segmento, e as duas de rotação
/// resolvem uma cadeia contra um alvo. ⚠️⚠️ *A nota que esta linha levava dizia
/// «Scale/Translate/Squash leem só a componente axial», e a medição diz **duas
/// de três** — a translação já lia o arrasto todo.*
///
/// # A propriedade que torna isto seguro
///
/// `δ` é a alavanca do quociente `L / (L − δ)`, e as duas leis são:
///
/// | modo | `δ` |
/// |---|---|
/// | [`Arrasto::AoLongoDoOsso`] | `dot(d, n̂)` — a projecção |
/// | [`Arrasto::Completo`] | `sign(dot(d, n̂)) · ‖d‖` — a mão toda, com o sentido da projecção |
///
/// ⭐ **Com a mão a puxar AO LONGO do osso as duas coincidem**: para `d = α·n̂`
/// vale `dot = α` e `‖d‖ = |α|`, logo `sign(α)·|α| = α`. ⇒ *o modo novo não
/// abre um regime novo onde o corpus vive; ele só deixa de deitar fora o que a
/// mão fez de lado.*
///
/// ⛔⛔ **O de fábrica é a projecção, e isso NÃO é gosto:** as `69` fixturas do
/// oráculo foram gravadas com ela, e o braço dela chama o **mesmo código de
/// antes** — a paridade fica intacta por CONSTRUÇÃO e não por um argumento
/// numérico.
///
/// # ⛔⛔⛔ O BOTÃO SAIU do painel (2026-09-17) — e a lei FICA
///
/// Veredito do dono, no dia seguinte a ele pedir os dois chips: *«Full drag
/// parece ser o único necessário»*. ⇒ o produto crava
/// [`Arrasto::Completo`] em [`ph2d_sculpt3d::PoseControlos::lei`], com a
/// divergência declarada e gateada ali; a escolha deixou de existir na tela.
///
/// ⚠️⚠️ **[`Arrasto::AoLongoDoOsso`] NÃO se apaga**, e não é por inércia: ela é
/// o valor de [`Controlos::default`] e **é o que as `69` fixturas alimentam**.
/// *Apagá-la levava o corpus do oráculo junto* — a mesma razão pela qual a
/// folga simétrica do `Scene Project` ficou viva depois de recusada. E o
/// [`Arrasto::label`] fica com ela: o dia em que alguém quiser a escolha de
/// volta, ela está escrita.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Arrasto {
    /// **A projecção no osso** — a lei da espec §5.4/§5.5, e o valor de fábrica.
    ///
    /// Puxar de lado não faz nada: o que não estiver ao longo do osso é
    /// deitado fora.
    #[default]
    AoLongoDoOsso,
    /// **O deslocamento inteiro**, com o sentido dado pela projecção.
    ///
    /// ⚠️ **Divergência DECLARADA.** Nenhuma referência a descreve; ela existe
    /// porque o dono a pediu como opção, e o gate exige que as duas leis
    /// **difiram** num arrasto transversal — senão o botão é decorativo.
    Completo,
}

impl Arrasto {
    /// Os dois, na ordem em que o painel os pinta — o de fábrica primeiro.
    pub const ALL: [Self; 2] = [Self::AoLongoDoOsso, Self::Completo];

    /// O rótulo do chip (a UI da casa é inglesa).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::AoLongoDoOsso => "Along Bone",
            Self::Completo => "Full Drag",
        }
    }

    /// **A PORTA ÚNICA** — o `δ` que o quociente de escala consome.
    ///
    /// ⚠️ `deslocamento` é `alvo − cabeça_inicial` e `normal` é a direcção
    /// inicial do primeiro segmento, **já normalizada** pelo chamador.
    #[must_use]
    pub fn alavanca(self, deslocamento: [f32; 3], normal: [f32; 3]) -> f32 {
        let axial = crate::vetor::ponto(deslocamento, normal);
        match self {
            Self::AoLongoDoOsso => axial,
            Self::Completo => {
                let n = crate::vetor::comprimento(deslocamento);
                // ⚠️ O `signum` de `0,0` é `+1` e o de `−0,0` é `−1`; com `n = 0`
                // o produto é zero nos dois casos, que é a resposta certa (*a
                // mão não andou*). ⛔ Um `if axial == 0.0 { 0.0 }` à frente seria
                // uma cerca que repete o que a aritmética já garante — a espécie
                // que o `clamp` inerte do emissor de partículas pagou.
                axial.signum() * n
            }
        }
    }
}

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
    /// ⭐⭐ **A CHAVE do rótulo** — `sculpt3d.pose_deformacao.<variante>`; o texto vive na
    /// tabela de strings (`ph2d-i18n/src/sculpt_engine.rs`) e quem o resolve é a interface.
    ///
    /// ⛔ **Esta crate NÃO ganha o acessório em inglês que o `ph2d-sculpt3d` tem**, e a razão
    /// está escrita no `Cargo.toml` dela: *«a lib continua sem dependência nenhuma»*. Um
    /// `tr_em(Ingles, …)` aqui traria a tabela de strings para dentro de uma crate de LEI.
    pub fn label_key(self) -> &'static str {
        match self {
            Self::GirarTorcer => "sculpt3d.pose_deformacao.girar_torcer",
            Self::EscalarTransladar => "sculpt3d.pose_deformacao.escalar_transladar",
            // ⚠️ Aqui o modificador **não muda nada**, e está medido: a fixtura
            // invertida é idêntica ao bit à normal. A barra fica porque o alvo
            // nomeia as duas metades assim.
            Self::EspremerEsticar => "sculpt3d.pose_deformacao.espremer_esticar",
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
///
/// ⭐⭐⭐ **E desde 2026-09-15 ela é também o que o ARTISTA escolhe**, por ordem
/// do dono (*«não devem ser ativados com CTRL mas checando o botão no
/// painel»*). A espec descreve o alvo, onde o modificador **troca de
/// deformação** e o artista alcança três botões e tem de descobrir que cada um
/// tem uma segunda metade escondida; aqui as **cinco** estão à vista.
///
/// ⚠️ **A LEI não mudou, e é por isso que os `69` traços do oráculo ficam
/// intactos:** o [`Controlos`] continua a ser `(modo, invertido)` e o
/// [`Controlos::deformacao`] continua a resolvê-los — o que mudou foi quem
/// **escolhe**, e a ponte do pincel entra por [`Self::modo_e_inversao`], que é a
/// inversa exacta (há gate de ida-e-volta).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Deformacao {
    #[default]
    Rodar,
    Torcer,
    Escalar,
    Transladar,
    Espremer,
}

impl Deformacao {
    /// As cinco, **na ordem em que o painel as lista**.
    ///
    /// ⚠️ A ordem é a dos pares do alvo (`girar|torcer`, `escalar|transladar`,
    /// `espremer`) achatada — assim quem vier de lá encontra os vizinhos onde os
    /// deixou, e quem não vier lê uma lista de cinco gestos.
    pub const ALL: [Self; 5] = [
        Self::Rodar,
        Self::Torcer,
        Self::Escalar,
        Self::Transladar,
        Self::Espremer,
    ];

    /// O nome que a UI mostra.
    ///
    /// ⚠️ **`Squash / Stretch` é UM gesto e a barra é parte do nome** — ao
    /// contrário das outras quatro, aqui não há duas metades: o modificador de
    /// inversão **não muda nada** neste modo, e está medido (a fixtura invertida
    /// é idêntica **ao bit** à normal).
    #[must_use]
    /// ⭐⭐ **A CHAVE do rótulo** — `sculpt3d.pose_modo.<variante>`; o texto vive na
    /// tabela de strings (`ph2d-i18n/src/sculpt_engine.rs`) e quem o resolve é a interface.
    ///
    /// ⛔ **Esta crate NÃO ganha o acessório em inglês que o `ph2d-sculpt3d` tem**, e a razão
    /// está escrita no `Cargo.toml` dela: *«a lib continua sem dependência nenhuma»*. Um
    /// `tr_em(Ingles, …)` aqui traria a tabela de strings para dentro de uma crate de LEI.
    pub fn label_key(self) -> &'static str {
        match self {
            Self::Rodar => "sculpt3d.pose_modo.rodar",
            Self::Torcer => "sculpt3d.pose_modo.torcer",
            Self::Escalar => "sculpt3d.pose_modo.escalar",
            Self::Transladar => "sculpt3d.pose_modo.transladar",
            Self::Espremer => "sculpt3d.pose_modo.espremer",
        }
    }

    /// **A INVERSA do [`Controlos::deformacao`]** — o par que a lei lê.
    ///
    /// ⚠️ **Ela existe para a lei não ter de mudar.** A alternativa era pôr a
    /// `Deformacao` dentro de [`Controlos`] e apagar o par — e isso mudaria o
    /// que as `69` fixturas do oráculo alimentam, que é a única coisa que mede
    /// esta crate. *Quem escolhe muda; o que a lei lê, não.*
    #[must_use]
    pub fn modo_e_inversao(self) -> (Modo, bool) {
        match self {
            Self::Rodar => (Modo::GirarTorcer, false),
            Self::Torcer => (Modo::GirarTorcer, true),
            Self::Escalar => (Modo::EscalarTransladar, false),
            Self::Transladar => (Modo::EscalarTransladar, true),
            // ⚠️ **`false` e não «qualquer um»:** o modificador não muda a saída
            // aqui, mas escrever `true` faria a ida-e-volta com o
            // [`Controlos::deformacao`] deixar de ser a identidade **na
            // representação**, e o gate que a prova é o que impede as duas
            // tabelas de divergirem no dia de uma sexta deformação.
            Self::Espremer => (Modo::EspremerEsticar, false),
        }
    }
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
    /// Quantas iterações de Jacobi esbatem os pesos (§4). **A lei não tem
    /// tecto** — quem o põe é o produto, e o dele é medido
    /// (`ph2d_sculpt3d::PoseControlos::SUAVIZACOES_MAX`).
    ///
    /// ⚠️ A largura que ela compra vale **`≈ 1,79 · √N` arestas da malha** —
    /// medida constante a `2 %` sobre quatro densidades —, logo ela é uma
    /// grandeza da MALHA e não do raio do pincel. ⭐ É esse defeito que a
    /// [`Self::banda_do_peso`] existe para curar.
    pub suavizacoes_do_peso: u32,
    /// ⭐⭐⭐ **A LARGURA DA TRANSIÇÃO, EM UNIDADES DE OBJECTO** — a lei
    /// alternativa à [`Self::suavizacoes_do_peso`], e a razão de ela existir
    /// está medida.
    ///
    /// `None` (o valor de fábrica) corre a difusão de Jacobi do §4 e a saída é
    /// **byte-idêntica** à de sempre — é isso que mantém os `69` traços do
    /// oráculo a valer como régua.
    ///
    /// `Some(b)` substitui a difusão por uma **distância na superfície**: o peso
    /// de cada vértice sai de quão longe ele está da fronteira do anel do
    /// segmento, e a transição mede **exactamente `b`** no barro.
    ///
    /// # ⛔ Porque é que a difusão não servia
    ///
    /// Ela espalha o peso de vizinho em vizinho, logo a largura que compra é
    /// contada em **arestas da malha**: a mesma posição do botão dá uma
    /// transição diferente numa peça fina e numa grossa, e numa peça grossa o
    /// topo do curso **dilui o núcleo** (medido: `1,0000 → 0,5498` a `1 490`
    /// vértices). ⚠️ E as reentrâncias que o dono reportou em 2026-09-17 viviam
    /// **todas** dentro dessa faixa — `203/203`, `878/878`, `1 336/1 336`,
    /// `801/801` e `10/10` das faces viradas do avesso tinham peso estritamente
    /// entre `0` e `1`.
    ///
    /// ⇒ *a largura da transição É o botão de qualidade deste pincel, e estava
    /// na unidade errada.*
    pub banda_do_peso: Option<f32>,
    /// Prende a extremidade distante da cadeia (§5.1).
    pub ancorado: bool,
    /// No modo de escala, não roda antes de escalar (§5.4).
    pub trava_rotacao: bool,
    /// ⭐⭐ **Quanto do arrasto a ESCALA lê** — ver [`Arrasto`]. Só as duas
    /// deformações do quociente de escala o consultam.
    pub lei_do_arrasto: Arrasto,
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
            // ⛔ **`None` é a lei do ORÁCULO, e é por isso que ela é o valor de
            // fábrica:** os `69` traços foram gravados com a difusão, e cravar a
            // outra aqui re-baseava o corpus em silêncio.
            banda_do_peso: None,
            // ⭐ **RECOMENDAÇÃO NOSSA, declarada como tal** (§1.4): nascer
            // ancorado, porque é a configuração em que o gesto roda em torno de
            // um pivô fixo — que é o que o nome do pincel promete.
            // ⛔ **Não é uma observação do alvo:** qual dos dois o artista lá
            // encontra **não foi medido**, e a proveniência que a espec tinha
            // era circular (o cabeçalho da fixtura é ENTRADA do harness).
            ancorado: true,
            trava_rotacao: false,
            // ⚠️ A projeccao e' a lei da espec, e e' com ela que as `69`
            // fixturas do oraculo foram gravadas.
            lei_do_arrasto: Arrasto::AoLongoDoOsso,
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
        let segs = cadeia.segmentos.len();
        if let Some(banda) = ctrl.banda_do_peso {
            // ⭐⭐⭐ **A LEI ALTERNATIVA: a transição é uma DISTÂNCIA no barro.**
            //
            // ⚠️⚠️ **Ela esbate os campos CUMULATIVOS, não as diferenças, e isso
            // é o que mantém a §3.3 de pé:** o peso de um segmento é a diferença
            // contra o estado depois do anterior, e uma diferença de dois campos
            // binários **não é binária** — esbatê-la por «distância à fronteira»
            // mediria a fronteira errada (a de um anel, não a do conjunto). Os
            // cumulativos são os `0/1` que o §3.3 cresceu, e a soma das
            // diferenças dos esbatidos telescopa para o último, exactamente como
            // antes.
            let mut cum = vec![0.0f32; n];
            let mut anterior = vec![0.0f32; n];
            let mut suave = vec![0.0f32; n];
            for i in 0..segs {
                for (v, c) in cum.iter_mut().enumerate().take(n) {
                    *c += cadeia.pesos[i * n + v];
                }
                suave.copy_from_slice(&cum);
                pesos::por_distancia(viz, posicoes, &mut suave, banda);
                for (v, (s, a)) in suave.iter().zip(&anterior).enumerate().take(n) {
                    cadeia.pesos[i * n + v] = s - a;
                }
                anterior.copy_from_slice(&suave);
            }
        } else {
            // §4 — cada segmento leva as suavizações **independentemente**.
            for i in 0..segs {
                pesos::suavizar(
                    viz,
                    &mut cadeia.pesos[i * n..(i + 1) * n],
                    ctrl.suavizacoes_do_peso,
                );
            }
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
