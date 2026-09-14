//! **AS PORTAS ENTRE O NÚMERO DO ARTISTA E O QUE O KERNEL CONSOME** — irmão do
//! [`super::brush`], cortado por ASSUNTO.
//!
//! O `brush.rs` diz **o que um pincel É** (os verbos, os campos, os defaults de
//! fábrica). Aqui vive a outra pergunta: **como cada número que o artista digita
//! vira o número que o laço do dab usa** —
//!
//! - [`Brush::weight`]: o slider de força → o peso do dab (a `StrengthCurve`);
//! - [`Brush::shaped_distance`]: a distância → a distância que a curva lê (o
//!   `hardness`);
//! - [`Brush::mask_weight`]: a distância → o peso do canal de máscara;
//! - [`Brush::reach`]: o raio → o deslocamento de um dab de peso cheio.
//!
//! ⚠️ **As quatro são PORTAS ÚNICAS, e é por isso que elas viajam juntas.** Cada
//! uma é o único lugar onde aquela conversão acontece; espalhá-las é como a
//! segunda cópia nasce, e este módulo existe para que a próxima conversão tenha
//! um endereço óbvio em vez de cair no arquivo que tiver espaço.

use super::*;

impl Brush {
    /// **O PESO que este pincel deposita** — a porta única entre o número do
    /// slider e o que um dab de fato faz.
    ///
    /// ⚠️ **Ela existe porque o slider e o peso deixaram de ser a mesma coisa.**
    /// A referência eleva ao quadrado (para dar mais sensibilidade à metade
    /// BAIXA do curso) e o SculptGL não; uma força crua no
    /// sítio de uso seria a segunda resposta que ignora o modo **em silêncio**,
    /// e o `stroke.rs` tem UM consumidor — é ele que pergunta aqui.
    ///
    /// ⚠️ **E ela pergunta pelo [`RefMode::for_verb`], não pelo modo cru.** Um
    /// modo que não DECLARA o verbo devolve `profile == None`, e o `map_or` caía
    /// no slider cru — o peso do SculptGL numa ferramenta que o SculptGL **não
    /// tem**. Os outros dois consumidores da lei de referência
    /// ([`RefMode::kernel_for`] e [`RefMode::lateral_for`]) já recuavam para a
    /// referência que TEM a tool; esta era a porta que ficou de fora, e era a
    /// única das três cujo erro muda a forma do barro.
    ///
    /// ⚠️ **Medido, e o número é o dobro:** com a demão nascida em `S` (a shell
    /// escrevia `[RefMode::default(); N]` — ver [`RefMode::birth_for`]), o
    /// slider de fábrica `0,50` entregava peso **0,5000** onde o *Layer*
    /// pede **0,2500**. Numa lei que satura isso não deposita mais alto: ela
    /// fecha o platô no dobro da velocidade e **o ombro que o falloff desenha
    /// desaparece** — a parede vira um degrau que a malha só sabe desenhar como
    /// escada. Os sete verbos do Blender pagavam o mesmo.
    #[must_use]
    pub fn weight(&self) -> f32 {
        self.verb
            .profile(self.mode.for_verb(self.verb))
            .map_or(self.strength, |p| p.strength_curve.resolve(self.strength))
    }
}

impl Brush {
    /// **ESTE INTERRUPTOR FAZ ALGUMA COISA AQUI?** — a porta única do
    /// [`Brush::front_faces_only`].
    ///
    /// ⚠️ **A pergunta é do PAR, e é por isso que ela não mora no `Verb` nem no
    /// `RefMode` sozinhos:** a *lei* é do modo ([`crate::FrontFace`], que o
    /// `S` deixa `Ignored` e o `B` deixa `Continuous`) e o *interruptor* é do
    /// pincel — o Blender tem as duas metades pelo mesmo motivo, com o
    /// filtro a existir sempre e um BIT do pincel a decidir se ele corre.
    ///
    /// ⚠️ **Porta e não um `matches!` no sítio de uso:** o painel pergunta para
    /// OFERECER a caixa e o roteador pergunta para HONRAR o clique — duas
    /// cópias divergiriam num controle que aparece e não move um vértice, que
    /// é exactamente o que o irmão [`crate::Verb::accumulates`] existe para
    /// impedir. O KERNEL não a chama: ele lê o `front_faces_only` dentro do
    /// `match` sobre a lei, onde o braço `Ignored` já o torna inalcançável.
    ///
    /// ⚠️ **Ela responde *"a lei existe"*, nunca *"o flag está ligado"*** — uma
    /// caixa que se escondesse quando desmarcada seria uma caixa que ninguém
    /// consegue marcar.
    #[must_use]
    pub fn offers_front_faces(&self) -> bool {
        matches!(
            self.mode.kernel_for(self.verb).front_face,
            crate::FrontFace::Continuous
        )
    }

    /// **Este verbo oferece a ÂNCORA EM VÉRTICE?** — o [`Brush::grab_active_vertex`].
    ///
    /// ⚠️ **Só o agarrar, e a cerca é a MEDIÇÃO e não a mecânica.** O polegar
    /// tem exactamente a mesma âncora (os dois são [`crate::Grip::Hold`]) e
    /// nenhuma fixture do corpus a exerce nele — oferecer ali seria estender uma
    /// lei da referência a uma ferramenta onde ela não foi observada, que é o
    /// que a §4 do plano 21 proíbe pelo nome.
    #[must_use]
    pub fn offers_grab_anchor(&self) -> bool {
        matches!(self.verb, crate::Verb::Move)
    }

    /// **Este pincel oferece os controlos da POSE?**
    #[must_use]
    pub fn offers_pose_controls(&self) -> bool {
        self.verb == crate::Verb::Pose
    }

    /// **Este pincel oferece os controlos do CONTORNO?**
    ///
    /// ⚠️ São **três** superfícies aqui (modo, queda no contorno, deslocamento
    /// da origem) e a espec conta **quatro** controlos próprios no alvo — o
    /// quarto é o alvo de deformação, que depende do solver de pano e é outra
    /// espec. *A ausência é declarada, não esquecida.*
    #[must_use]
    pub fn offers_boundary_controls(&self) -> bool {
        self.verb == crate::Verb::Boundary
    }

    /// **ESTE PINCEL TRAZ O PRÓPRIO ALVO DE DENSIDADE?**
    ///
    /// ⭐⭐ **A porta ÚNICA, e ela tem DOIS consumidores de crates diferentes:**
    /// o painel pergunta para **oferecer** a pista
    /// ([`Brush::density_detail`]) e o passe de topologia pergunta para saber
    /// **qual dos dois números ler** — o deste pincel ou o ajuste da cena.
    /// *Duas cópias divergiriam num slider que aparece e governa outra coisa.*
    ///
    /// ⚠️ **A pergunta é ao PREDICADO do verbo**
    /// ([`crate::Verb::sem_lei_por_vertice`]), nunca ao nome: quem não tem lei
    /// por-vértice tem a densidade como **único** efeito, logo é quem tem o que
    /// pedir. Comparar com `Verb::Density` aqui seria a segunda resposta à mesma
    /// pergunta.
    ///
    /// ⚠️ Ela responde hoje o mesmo que a
    /// [`crate::Verb::corre_sem_o_interruptor`], e as duas **perguntas** são
    /// diferentes — *«tem alvo próprio?»* contra *«precisa do interruptor?»*.
    /// Elas coincidem porque a mesma propriedade responde às duas; o dia em que
    /// uma delas mudar, muda sozinha.
    #[must_use]
    pub fn offers_density_controls(&self) -> bool {
        self.verb.sem_lei_por_vertice()
    }

    /// **ESTE PINCEL ESCOLHE UMA DIRECÇÃO DE ESFREGÃO?** — a porta única do
    /// [`Brush::smear_mode`].
    ///
    /// ⚠️ **Ela compara com o VERBO, e isso é o oposto da irmã de cima** — que
    /// pergunta a um predicado de propósito. Aqui não há propriedade a
    /// perguntar: *«qual a direcção do esfregão»* é uma pergunta sobre **este**
    /// pincel e mais nenhum, e um predicado com um membro só seria uma família
    /// inventada para parecer derivada. ⛔ *Uma lei derivada de uma população
    /// de um é uma lista escrita à mão com outro nome.*
    #[must_use]
    pub fn offers_smear_controls(&self) -> bool {
        self.verb == crate::Verb::SmearMultires
    }

    /// **E a TRAVA DE ROTAÇÃO?**
    ///
    /// ⭐⭐ **Só no modo de escala, e isso é LEI e não arrumação:** ela decide se
    /// o gesto roda **antes** de escalar (espec §5.4 passo 1). No modo de
    /// girar/torcer não há escala nenhuma para travar, e no de espremer/esticar
    /// a espec diz por escrito que *«a trava de rotação não tem papel nenhum»* —
    /// ali a cadeia nem sequer é resolvida.
    ///
    /// ⚠️ *Mostrá-la nos três seria um controlo morto em dois deles* — e um
    /// interruptor que não faz nada é pior que um ausente, porque o artista
    /// conclui que já tentou.
    #[must_use]
    pub fn offers_pose_rotation_lock(&self) -> bool {
        self.offers_pose_controls() && self.pose.modo == crate::PoseModo::EscalarTransladar
    }

    /// **Este verbo lê o [`Brush::surface_only`]?** — a porta única, pelo mesmo
    /// argumento do irmão acima.
    ///
    /// ⛔⛔ **O [`crate::Verb::Cloth`] NÃO o lê, e a razão é estrutural:** ele
    /// **desvia antes do `dab_core`** ([`crate::stroke_symmetry`] — ele é dono da
    /// própria expansão de simetria, porque cada cópia tem a sua região), então a
    /// máscara de alcance nunca corre nele. *Pintar a caixa ali seria um
    /// interruptor de coisa nenhuma*, que é exactamente a espécie de controlo
    /// morto que esta casa caça.
    #[must_use]
    pub fn offers_surface_only(&self) -> bool {
        self.verb != crate::Verb::Cloth
    }

    /// **O PINCEL DO SEGUNDO PASSE**, ou `None` quando ele não corre — a porta
    /// única do [`Brush::auto_smooth`].
    ///
    /// Porte do passe que a referência corre **depois** do verbo, em cada
    /// passada de simetria: se o verbo não é um alisamento nem a máscara, e o
    /// factor de auto-alisamento é positivo, ela corre o alisamento com esse
    /// factor como força.
    ///
    /// ⚠️ **As duas exclusões são do original e cada uma tem um motivo
    /// diferente.** Alisar um alisamento é o mesmo verbo duas vezes com pesos
    /// que ninguém pediu; e o canal de MÁSCARA não é geometria — um passe que
    /// mexesse na posição enquanto o artista pinta uma máscara moveria o barro
    /// num gesto cuja razão de existir é **não** mover o barro.
    ///
    /// ⚠️ **O pincel do passe é o MESMO, com o verbo e a força trocados**, e não
    /// um pincel montado do zero: no original o passe chama o verbo *Smooth* com
    /// o pincel inteiro do artista e só a força é substituída pelo fator, então a
    /// dureza, a curva, o padrão e a simetria do artista **valem também no
    /// alisamento**. Montar
    /// um pincel neutro aqui seria uma segunda resposta a *"que forma tem este
    /// dab?"*, e ela divergiria da primeira exactamente na borda, que é onde o
    /// passe existe para agir.
    ///
    /// ⚠️ **`auto_smooth: 0.0` no filho é o que impede a recursão**, e ele é
    /// estrutural em vez de uma guarda no chamador: um passe que se pedisse a si
    /// mesmo não daria erro nenhum — daria um laço.
    #[must_use]
    pub fn auto_smooth_brush(&self) -> Option<Self> {
        // ⚠️ **`is_finite` E `<= 0` — o par, não a negação de `> 0`.** Um
        // `NaN` que chegasse por um param mal carregado passaria por
        // `<= 0.0` (toda comparação com NaN é falsa) e viraria a FORÇA de um
        // pincel de Smooth, que é a forma mais barata de envenenar a malha.
        if !self.auto_smooth.is_finite()
            || self.auto_smooth <= 0.0
            || matches!(self.verb, crate::Verb::Smooth | crate::Verb::Mask)
        {
            return None;
        }
        // ⚠️ **`strength: 1,0` e NÃO `self.auto_smooth`** — a força do
        // alisamento é um ORÇAMENTO de passadas
        // ([`crate::auto_smooth::iteration_strengths`]), não o coeficiente de
        // um lerp; quem a carrega é o `Pass::weight`, no chamador. Pô-la aqui
        // fazia duas coisas erradas de uma vez: reproduzia a referência só
        // abaixo de `0,25` e ainda passava pela curva do slider do modo, que no
        // `b-mode` a **elevaria ao quadrado**.
        Some(Self {
            verb: crate::Verb::Smooth,
            strength: 1.0,
            auto_smooth: 0.0,
            ..self.clone()
        })
    }
}

/// A dureza do canal em `1.0` dá expoente **zero**, e `x^0 == 1` em toda a
/// pegada — o disco duro. É o topo da faixa da tool do original, e o nome existe
/// para o painel não repetir o literal.
pub const MAX_MASK_HARDNESS: f32 = 1.0;

impl Brush {
    /// **A DISTÂNCIA QUE A CURVA VAI LER** — a porta única do `hardness`.
    ///
    /// Porte literal do remapeamento de DUREZA da referência,
    /// em distância NORMALIZADA (`t = d / raio`), que é a forma em que o resto
    /// deste motor fala:
    ///
    /// ```text
    /// t' = 0                        se t < hardness
    /// t' = (t − hardness)/(1 − h)   caso contrário
    /// ```
    ///
    /// Ou seja: um **platô de peso cheio** de raio `hardness · r`, e o falloff
    /// inteiro espremido na casca que sobra. Em `hardness = 1` o pincel vira um
    /// **disco duro** — e esse caso tem braço próprio no original porque a
    /// fórmula geral dividiria por zero.
    ///
    /// ⚠️ **Ela remapeia a distância de TODOS os consumidores da curva**, o
    /// canal de máscara incluído, porque é isso que o original faz: o
    /// remapeamento roda **antes** de a curva ser avaliada, e nenhuma curva sabe
    /// que ele existe.
    /// Aplicá-la só na geometria faria a máscara ler uma distância diferente da
    /// que o pincel usa no mesmo dab.
    #[must_use]
    pub fn shaped_distance(&self, t: f32) -> f32 {
        let h = self.hardness;
        if h <= 0.0 {
            // ⚠️ **O early-out É a identidade bit a bit**, e é ele que torna
            // esta wave invisível no produto: sem `hardness`, nem uma subtração
            // acontece.
            return t;
        }
        if h >= 1.0 {
            // O braço do disco duro: dentro do raio nada decai, fora dele o
            // peso é zero (que é o que a curva devolve em `t >= 1`).
            return if t < 1.0 { 0.0 } else { 1.0 };
        }
        if t < h { 0.0 } else { (t - h) / (1.0 - h) }
    }

    /// **A CURVA DO CANAL DE MÁSCARA** — `(1 − t)^{2(1 − hardness)}`, o
    /// `Masking.paint` do original (`Masking.js:66-69`).
    ///
    /// ⚠️ **A aritmética é `f64` e a arredondada é UMA**, como em todo o porte
    /// (`ref_kernels`): o `Math.pow` do JS trabalha em duplo e o
    /// `Float32Array` guarda uma vez. Computá-la em `f32` acumularia uma
    /// segunda arredondada e a paridade sairia do piso do formato.
    ///
    /// ⚠️ **`t` chega JÁ normalizado pelo raio** e não é clampado aqui: o
    /// original clampa (`if dist > 1 dist = 1`) e a nota do
    /// [`crate::ref_kernels`] mede que esse ramo é **inalcançável** — quem monta
    /// a pegada só admite `d² < r²`. A guarda contra `t > 1` mora onde o
    /// consumo mora, e duplicá-la aqui seria a segunda resposta à mesma
    /// pergunta.
    #[must_use]
    pub fn mask_weight(&self, t: f32) -> f32 {
        let softness = 2.0 * (1.0 - f64::from(self.mask_hardness));
        (1.0 - f64::from(t)).powf(softness) as f32
    }
}

impl Brush {
    /// O deslocamento, com sinal, que um dab deste pincel alcança — em unidades
    /// de mundo. Porta única: Draw, Inflate, Clay e Crease perguntam aqui.
    ///
    /// ⚠️ **O termo `honours_invert()` aqui é INERTE hoje, e fica assim mesmo.**
    /// Depois que o predicado passou a dizer a verdade, *todo* verbo que lê
    /// `reach` está na whitelist ⇒ trocá-lo por um `if self.invert` puro daria o
    /// mesmo número em todos os doze, e uma mutação que o remova **não sangra**.
    /// Ele fica porque é aqui que a pergunta pertence — *o sinal é assunto do
    /// VERBO, não do checkbox* —, e porque o dia em que entrar um verbo que
    /// consome `reach` sem ter oposto é o dia em que ele deixa de ser inerte, sem
    /// ninguém precisar lembrar. Defesa em camadas documentada em vez de gateada,
    /// pelo precedente do ADR-0145.
    ///
    /// ⚠️ **Este bloco estava ÓRFÃO** — colado acima do `alpha_weight`, descrevendo
    /// uma função que não é esta, com o `reach` sem doc nenhum. É a classe que
    /// este módulo já registrou duas vezes (*"minhas linhas `mod` orfanaram
    /// doc-comments"*), e ela não levanta erro: só uma leitura pega.
    #[must_use]
    pub fn reach(&self, radius: f32) -> f32 {
        let s = if self.invert && self.verb.honours_invert() {
            -1.0
        } else {
            1.0
        };
        // ⚠️ **A FRAÇÃO É DO MODO, e era uma constante para o catálogo todo.**
        // Aqui morava `if verb == ClayStrips { 1,0 } else { 0,1 }` — o `0,1` é
        // o `deform = intensidade · raio · 0,1` do `Brush.js:62`, do SculptGL, e
        // a exceção nasceu de um smoke que mediu a faixa **7,5× mais fraca**.
        // Medido em 2026-08-16, **sete** verbos só existem no Blender e **seis**
        // seguiam com o número do SculptGL: o `if` era a primeira linha de uma
        // enumeração, não um caso especial. Hoje quem responde é o perfil da
        // referência que o modo escolhe, então um verbo do Blender nasce com
        // `1,0` **por declaração** e a exceção dissolve.
        // ⚠️ **`for_verb`, nunca `self.mode` cru** — o recuo que o
        // [`Self::weight`] já documenta: um pincel pode carregar um modo que o
        // verbo NÃO declara, e ali `profile` devolve `None`. Medido: com o modo
        // cru o Clay Strips voltava a `0,1` (a fixture nasce em `S` e ele só
        // existe em `B`), que é a regressão exacta que esta wave cura.
        let f = self
            .verb
            .profile(self.mode.for_verb(self.verb))
            .and_then(|p| p.reach)
            .unwrap_or(REACH_FRACTION);
        radius * f * s
    }

    /// **O RAIO QUE A CONSULTA DA PEGADA USA** — a quinta porta, e a única que
    /// pode devolver MAIS que o raio do pincel.
    ///
    /// ⚠️ **Ela existe porque um campo elástico decide o próprio suporte, e o
    /// inverso não é verdade.** Um verbo de carimbo é uma curva que já vale zero
    /// em `t = 1`, então a pegada e a influência são o mesmo círculo. Um
    /// Kelvinlet não acaba em lado nenhum: o raio do pincel é o `ε` dele — a
    /// ESCALA da resposta —, e quem decide onde cortar é o resíduo que ainda
    /// sobra (ver [`crate::KELVINLET_REACH`]).
    ///
    /// ⚠️ **A leitura do anel do cursor MUDA, e é o preço nomeado:** com um
    /// campo, ele deixa de significar *o que eu toco* e passa a significar *a
    /// escala do que eu deformo* — a mesma leitura do Elastic Deform do Blender,
    /// cujo pincel deforma bem além do círculo desenhado. Um artista que quer o
    /// círculo literal tem o `s-mode` ao lado, no mesmo verbo.
    #[must_use]
    pub fn query_radius(&self, radius: f32) -> f32 {
        if self.mode.field(self.verb).is_some() {
            radius * crate::KELVINLET_REACH
        } else if self.verb == crate::Verb::ClayStrips {
            // ⚠️ **Uma CAIXA não cabe no círculo que a inscreve.** O canto de
            // uma faixa `1 × L` está a `√(1 + L²)` raios do centro, e uma
            // consulta de raio `r` devolveria a faixa com as QUINAS comidas —
            // um defeito mudo, porque a silhueta continuaria plausível. O fator
            // é perguntado à própria forma, nunca recomputado aqui.
            radius * crate::Footprint::strip_query_factor(self.strip_length)
        } else {
            radius
        }
    }
}
