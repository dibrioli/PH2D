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

    /// **ESTE PINCEL ESCOLHE COMO PROJECTAR?** — a porta única dos três knobs
    /// do [`crate::Verb::SceneProject`] (`project_mode` · `project_min_distance`
    /// · `project_bidirectional`), pelo mesmo argumento da irmã acima: a
    /// população é de UM, e um predicado com um membro só seria uma lista
    /// escrita à mão com outro nome.
    #[must_use]
    pub fn offers_project_controls(&self) -> bool {
        self.verb == crate::Verb::SceneProject
    }

    /// ⭐⭐⭐ **ESTE GESTO PRECISA DE SABER O QUE MAIS HÁ NA CENA?** — a porta
    /// única que a shell pergunta antes de fotografar as outras peças
    /// ([`crate::SculptStroke::pecas_da_cena`]).
    ///
    /// ⚠️ **Ela existe porque a lista tem DOIS consumidores com predicados
    /// DIFERENTES**, e essa é exactamente a forma que produz duas respostas à
    /// mesma pergunta: o tecido só quer as peças com a colisão LIGADA (ela é
    /// cara — `2,6×` a `6,1×` o dab, medido), e a projecção quer-as sempre,
    /// porque **sem elas ela não tem lei nenhuma**. ⛔ Escrito no sítio da
    /// fotografia como um `if` de duas pernas, o terceiro consumidor herdaria a
    /// perna errada em silêncio.
    #[must_use]
    pub fn precisa_das_pecas_da_cena(&self) -> bool {
        match self.verb {
            crate::Verb::Cloth => self.cloth_collisions,
            crate::Verb::SceneProject => true,
            _ => false,
        }
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
        self.offers_pose_controls() && self.pose.deformacao == crate::PoseDeformacao::Escalar
    }

    // ⛔⛔⛔ **O `offers_pose_drag_law` SAIU (2026-09-17), com o chip que ele
    // governava.** Veredito do dono depois de o testar: *«Full drag parece ser
    // o único necessário»*. ⇒ o pincel lê o arrasto INTEIRO em toda deformação
    // que tenha a alavanca `δ`, e a escolha deixou de existir — a lei e a
    // divergência declarada estão em [`crate::PoseControlos::lei`].
    //
    // ⚠️ *A medição que o chip trouxe FICA e não se perde:* só as duas
    // deformações do quociente de escala leem esta lei (a translação já somava o
    // deslocamento inteiro, e as duas rotações resolvem uma cadeia) — e é por
    // isso que trocá-la não toca nos outros três gestos.

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

    /// **Este verbo oferece o [`Brush::puxa_pela_normal`]?** — a porta única,
    /// **derivada do GRIP** e não de uma lista de nomes.
    ///
    /// A pergunta é *«o gesto deste verbo É uma direcção de puxão?»*, e quem a
    /// responde é a tabela do [`crate::Grip`]: o [`crate::Grip::Hold`] (o Grab)
    /// e o [`crate::Grip::Hook`] (o Snake Hook) leem o `dab.pull` como um vector
    /// de mundo e escrevem o barro ao longo dele. ⛔ Os outros grips não têm
    /// puxão nenhum para redireccionar — o carimbo escreve ao longo da normal
    /// **por lei**, a torção gira, a simulação conduz um solver.
    ///
    /// ⚠️ **Derivar em vez de listar é o que faz um verbo NOVO com um destes
    /// dois grips nascer com a opção** — uma lista de nomes nasceria incompleta
    /// no dia seguinte, que é a forma que esta casa já pagou meia dúzia de vezes.
    #[must_use]
    pub fn oferece_puxar_pela_normal(&self) -> bool {
        matches!(self.verb.grip(), crate::Grip::Hold | crate::Grip::Hook)
            // ⛔⛔ **E as duas exclusões são MEDIDAS, não uma opinião** — a 1.ª
            // redacção parava no grip e o painel oferecia a caixa a SEIS verbos.
            // Correndo o mesmo gesto com e sem a opção
            // (`diag_quem_sente_a_opcao`): `Move` move `0,150` · `SnakeHook`
            // `0,297` · **`Pose` e `Boundary` `0,000`** (resolvem a própria
            // região e nunca leem o `dab.pull`) · **`Thumb` `6e-5` e `Nudge`
            // `−0,027`** (subtraem a normal do puxão, logo pô-lo ao longo dela
            // deixa zero — a opção *desligava* o empurrão).
            //
            // ⚠️ E a catraca da dobra do painel apanhou isto antes de eu medir:
            // uma fileira nova no bloco partilhado custa `+28 px` a **todos** os
            // pincéis, e o `Boundary` desceu de `1083` para `1111`.
            && !self.verb.resolve_a_propria_regiao()
            && !self.verb.subtrai_a_normal_do_puxao()
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
            // ⛔⛔ **E os que o passe NÃO ALCANÇA** — ver
            // [`crate::Verb::o_auto_smooth_chega`], onde mora a razão de cada
            // um e as duas saídas nomeadas. *Um controlo que o artista arrasta
            // e o barro não sente é pior que um ausente*, e o censo dos knobs
            // mede-o (`0,000e0` entre as duas pontas da faixa).
            || !self.verb.o_auto_smooth_chega()
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
    /// ⛔⛔ **ESTA NOTA DIZIA QUE O CLAMP ERA DESNECESSÁRIO, E AS DUAS METADES
    /// DELA ERAM FALSAS** — a redacção anterior afirmava que o ramo `t > 1` é
    /// *«inalcançável, quem monta a pegada só admite `d² < r²`»* e que *«a
    /// guarda mora onde o consumo mora»*. Medido no produto em 2026-09-20:
    /// o `t` vale **`1,0005` a `1,0108`**, e o consumo
    /// ([`crate::SculptStroke`]) não tem guarda nenhuma — ela vivia dentro de
    /// UMA das três curvas de peso, a [`crate::Falloff::weight`], e esta
    /// nasceu sem ela.
    ///
    /// ⇒ hoje as duas leem a mesma porta, [`crate::fora_da_pegada`], onde o
    /// mecanismo está escrito: a pegada sai das posições **VIVAS** e o peso do
    /// envelope sai da **CONGELADA**, e a topologia dinâmica move uma sem mexer
    /// na outra. *A premissa estava certa sobre a REFERÊNCIA e falsa sobre a
    /// nossa consulta, e ninguém a reconferiu quando o dyntopo passou a mover
    /// vértices debaixo de um traço.*
    #[must_use]
    pub fn mask_weight(&self, t: f32) -> f32 {
        self.channel_weight(t, self.mask_hardness)
    }

    /// **A CURVA DE UM CANAL, com a dureza de quem a pede** — a porta que o
    /// [`Self::mask_weight`] e o [`Self::paint_weight`] partilham.
    ///
    /// ⭐ **Ela é literalmente a mesma expressão nos dois lados da referência**
    /// (`Masking.js:66-69` e `Paint.js:124-127` escrevem `softness = 2(1−h)` e
    /// `pow(1−d, softness)`), e é por isso que ela é UMA função: duas cópias da
    /// potência divergiriam no dia em que alguém corrigisse uma delas.
    ///
    /// ⛔⛔⛔ **FORA DA PEGADA ELA DEVOLVE ZERO, e essa linha é a cura do report
    /// das manchas pretas** (20/09). Sem ela, `t > 1` faz a base ser negativa e
    /// `powf` com o expoente de fábrica (`2·(1 − 0,75) = 0,5`, uma raiz
    /// quadrada) devolve **`NaN`** — que não pinta um pixel errado, **contamina
    /// a interpolação da FACE inteira**. Ver [`crate::fora_da_pegada`] para o
    /// mecanismo e para a premissa que ela derrubou.
    ///
    /// ⚠️ **Dentro da pegada não muda um bit:** para `t ∈ [0, 1)` o caminho é o
    /// de sempre, e em `t = 1` exacto as duas leis já concordavam (`0^s = 0`).
    #[must_use]
    pub fn channel_weight(&self, t: f32, hardness: f32) -> f32 {
        if crate::fora_da_pegada(t) {
            return 0.0;
        }
        let softness = 2.0 * (1.0 - f64::from(hardness));
        (1.0 - f64::from(t)).powf(softness) as f32
    }

    /// **A CURVA DO CANAL DE COR** — `channel_weight` com a dureza do
    /// [`Verb::Paint`]. Ver [`Self::paint_hardness`].
    #[must_use]
    pub fn paint_weight(&self, t: f32) -> f32 {
        self.channel_weight(t, self.paint_hardness)
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
        // ⭐⭐ **O SINAL tem DOIS termos desde 2026-09-16, e o segundo é a
        // direcção de FÁBRICA do verbo** ([`crate::Verb::afunda_de_fabrica`]):
        // até aqui a casa só sabia *«o `Ctrl` inverte»*, e um pincel que nasce a
        // AFUNDAR não tinha onde o dizer. A lei é a da espec §2.4 do pincel
        // afiado — `σ = direcção de fábrica × (−1 se Ctrl)` —, e o `!=` é o
        // «ou exclusivo» dela: com o `Ctrl` carregado o pincel que afunda
        // levanta, e vice-versa.
        let afunda = self.verb.afunda_de_fabrica() != (self.invert && self.verb.honours_invert());
        let s = if afunda { -1.0 } else { 1.0 };
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
        // ⭐⭐ **E a ATENUAÇÃO DO TRAÇO ARRASTADO entra aqui**, pela porta única
        // [`Self::factor_do_traco`]: ela é *quanto cada dab de um traço vale
        // quando os dabs se sobrepõem*, e o deslocamento de um dab **é** este
        // alcance. `1` fora do arrasto e nos verbos que não a declaram, logo o
        // caminho de todos os outros é byte-idêntico.
        //
        // ⚠️ **O pincel de plano NÃO passa por aqui** — o alvo dele é uma
        // projecção e não um deslocamento ao longo de uma normal, então ele lê a
        // MESMA porta no sítio onde a lei dele pesa
        // ([`crate::PlanoDaPegada`]). *Uma lei, uma porta, dois consumidores —
        // e cada um multiplica-a na grandeza que a sua própria lei escala.*
        radius * f * s * self.factor_do_traco()
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
        } else if self.verb == crate::Verb::DrawSharp {
            // ⭐ **O afiado alarga a consulta PELA MESMA PORTA que o plano**, e
            // por uma razão mais simples: a normal da área dele soma sobre
            // `R_n = fracção × R`, e a fracção **pode passar de `1`** (a fixtura
            // `normal_da_area_raio_2_0` da espec usa `2` e só é reprodutível com
            // a soma sobre `2R`). ⛔ Uma segunda varredura para a normal daria
            // duas respostas a *«que vértices este dab consulta?»*.
            //
            // ⚠️ **No valor de fábrica ele é inerte** (`0,5 × R < R`), e é assim
            // que tem de ser: o `max` com `1` é o que impede a consulta de
            // ENCOLHER abaixo da pegada da própria curva de queda.
            radius * self.normal_radius_frac.max(1.0)
        } else if self.verb == crate::Verb::Plane {
            // ⚠️ **O elipsóide dos dois tectos NÃO cabe no círculo do cursor**, e
            // a razão é outra que a da faixa: ele cabe no círculo do **centro do
            // plano**, que a lei do centro e o deslocamento afastam do cursor. O
            // factor é perguntado à forma ([`crate::Footprint::tectos_query_factor`]),
            // que o deriva da MESMA barra que o gate G-6 afirma — e é este o
            // único sítio que sabe o deslocamento.
            radius
                * crate::Footprint::tectos_query_factor(
                    self.plane_offset,
                    self.normal_radius_frac,
                    self.area_radius_frac,
                )
        } else {
            radius
        }
    }
}
