//! **O ARM da topologia dinâmica** — o interruptor, o detalhe e o que um dab
//! faz com eles.
//!
//! Filho (`#[path]`) de [`super`]. O motor mora na `ph2d-mesh`
//! ([`ph2d_mesh::refine_in_sphere`]); aqui fica o que é do EDITOR: quando o
//! refino roda, o que ele custa ao traço em voo e como o desfazer o carrega.
//!
//! # Ele é opt-in, e a razão é dos autores do Blender
//!
//! *"o dyntopo foi sempre explicado como uma otimização quando na verdade é bem
//! mais lento e come muito mais memória"* — por isso o default é DESLIGADO, e o
//! padrão desta casa continua sendo `remesh sob comando + multires`
//! ([`docs/3D/04-Ferramentas/04.3-Topologia.md`]).
//!
//! # As DUAS metades, e a segunda chegou depois
//!
//! Armado, um dab **refina** onde a aresta é longa demais e **COLAPSA** onde ela
//! é curta demais. O limiar do colapso é DERIVADO do alvo do refino (ver
//! [`collapse_target`]) e não é um segundo slider: a relação entre os dois é a
//! histerese que impede o par de moer, e ela é propriedade do par, não
//! preferência. É o default do Blender (*Subdivide Collapse*); o SculptGL expõe
//! os dois e ship o de colapso em ZERO.
//!
//! # As três consequências de ligar, e nenhuma é escondida
//!
//! 1. **A peça é TRIANGULADA** ([`ph2d_mesh::Mesh::triangulate`]). Não é gosto:
//!    partir a aresta de um quad devolve um triângulo e um pentágono.
//! 2. **O desfazer de um traço passa a ser a MALHA INTEIRA.** A janela por
//!    vértice (`StrokeUndo::Stroke`) descreve *quais índices se moveram*, e um
//!    traço que muda a contagem não tem essa janela — os índices de depois não
//!    descrevem os de antes. É o mesmo argumento do remesh, e a mesma entrada
//!    (`Remeshed`), que já é uma TROCA simétrica: desfazer devolve a malha de
//!    antes, refazer devolve a de depois.
//! 3. **A pilha de multiresolução recusa.** Refinar a base sob níveis que são
//!    subdivisão dela deixaria cada nível descrevendo outra malha — o mesmo
//!    motivo pelo qual o remesh recusa, e a recusa é NOMEADA no log em vez de
//!    achatar a pilha em silêncio.

use ph2d_mesh::{
    Collapse, Refine, collapse_target, edge_target_for_mesh,
    refine_in_sphere_sized,
};

use super::Sculpt3dScene;

/// O estado autorado do modo.
///
/// ⚠️ **`detail` é uma FRAÇÃO, não um comprimento** — quem o transforma em alvo
/// de aresta é o [`edge_target_for_mesh`], contra a **ÁREA DA SUPERFÍCIE** da
/// peça. Guardar um comprimento aqui daria ao artista um número que muda de
/// significado de peça para peça.
///
/// ⚠️⚠️ **A âncora era o RAIO DO PINCEL e mudou em 2026-09-14** (ordem do dono:
/// *«a densidade da malha deve ser independente do zoom»*): aquele raio é
/// derivado do raio em PIXELS **através da câmera**, logo o zoom entrava no
/// alvo — `4,9×` medido. *O pincel diz ONDE, este número diz QUÃO FINO.*
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) struct Dyntopo {
    pub(super) armed: bool,
    pub(super) detail: f32,
}

impl Default for Dyntopo {
    fn default() -> Self {
        // ⚠️ O default é DESARMADO (ver o cabeçalho) e o detalhe nasce no MEIO:
        // ligado no extremo fino, o primeiro traço num modelo grosso multiplica
        // a contagem de faces antes de o artista ter visto o que a tecla faz.
        Self {
            armed: false,
            detail: 0.5,
        }
    }
}

/// Os três degraus que a TECLA `U` percorre, e os nomes que o log usa.
///
/// ⚠️⚠️ **Eles deixaram de ser a única superfície em 2026-09-14** — report do
/// dono: *«porque não temos um slider neste pincel para definir a densidade da
/// malha»*. A nota que aqui estava dizia *«três e não um slider contínuo,
/// porque a UI aqui é o teclado»*, e a **premissa dela expirou** quando a
/// secção Topology do painel ganhou os knobs do remesh; ninguém releu a nota.
///
/// ⭐ Hoje o valor é uma **pista contínua** no painel e esta tabela é o
/// **atalho**: um toque por degrau, com nome próprio, que é o que um log
/// consegue dizer (`0,5 → 0,53 → 0,56` não é). *A mesma relação que o `[`/`]`
/// tem com a pista do raio.*
pub(super) const DETAIL_STEPS: [(f32, &str); 3] = [(0.15, "grosso"), (0.5, "medio"), (1.0, "fino")];

/// Quantas passagens cada um dos dois campos da retícula leva por carimbo.
///
/// ⛔⛔ **Número MEDIDO, e a medição refutou a hipótese natural:** o risco que a
/// pesquisa nomeou para esta wave — *«a retícula de um vizinho discorda da do
/// outro por uma célula»* — **não é convergência**. A escada `1 · 2 · 3 · 4 · 6
/// · 8 · 16` no lado de célula que shipa:
///
/// | rondas | grade | vinco p50 | vinco p90 |
/// |---|---|---|---|
/// | `1` | `61,8 %` | `1,059°` | `2,810°` |
/// | **`2`** | **`64,2 %`** | **`0,994°`** | **`2,620°`** |
/// | `3` | `64,3 %` | `0,963°` | `2,653°` |
/// | `6` | `64,4 %` | `0,953°` | `2,631°` |
/// | `16` | `65,4 %` | `0,977°` | `2,661°` |
///
/// ⇒ **o patamar é em `2`**, e o que sobra acima dele é relógio. **Medido em
/// `--release`, o mínimo de cinco** (`diag_o_relogio_da_reticula`), com a pegada
/// a crescer porque o pincel mede píxeis e a malha adensa:
///
/// | vértices | pegada | `6` rondas | **`2` rondas** |
/// |---|---|---|---|
/// | `5 276` | `21` | `0,328 ms` | **`0,099 ms`** |
/// | `21 098` | `115` | `1,490 ms` | **`0,499 ms`** |
/// | `84 386` | `539` | `7,710 ms` (**96 %** do orçamento) | **`2,435 ms`** (`30 %`) |
///
/// ⚠️ **O recurso é o orçamento do carimbo (`8 ms`)** e o custo é **linear na
/// PEGADA** (`~4,5 µs` por vértice a `2` rondas), nunca na peça — é isso que a
/// [`crate::regiao`] existe para garantir.
///
/// ⚠️⚠️ **A primeira corrida desta escada leu «sem tendência» e estava a medir
/// outro programa:** ela varreu as rondas com o [`LADO_DA_CELULA`] em `1,0`,
/// onde TODAS as leituras são más — *uma escada corrida no regime errado
/// responde sobre um produto que não existe*. Sonda: `diag_a_escada_das_rondas`.
pub(crate) const RONDAS_DA_GRELHA: usize = 2;

/// O lado da célula, em aresta média da pegada.
///
/// ⭐⭐⭐ **É ESTA a alavanca, e a escada é brutal** (peça da cena `=49`, rumo
/// `x`, com o vinco a ser o que a LUZ mostra):
///
/// | `k` | grade | vinco p50 | vinco p90 |
/// |---|---|---|---|
/// | `0,80` | **`64,9 %`** | **`0,96°`** | **`2,70°`** |
/// | `0,90` | `64,9 %` | `1,13°` | `3,27°` |
/// | `0,931` | `64,1 %` | `1,20°` | `3,49°` |
/// | `1,00` | `57,3 %` | `2,13°` | `20,64°` |
/// | `1,10` | `45,4 %` | `5,90°` | `98,49°` |
/// | `1,25` | `39,8 %` | `15,99°` | `126,42°` |
///
/// ⚠️ **O recurso é a CAPACIDADE da célula:** um quadrado de lado `e` cobre `e²`
/// por vértice e um triângulo equilátero de aresta `e` cobre `0,866 e²` ⇒ pedir
/// a uma malha de triângulos que pouse numa grelha quadrada do **mesmo** lado
/// empilha vértices, e um empilhamento é uma dobra. O ponto de empate teórico é
/// `√0,866 = 0,931`, e a medição põe o joelho **abaixo** dele — entre `0,931` e
/// `1,00` o vinco p90 salta `5,9×`.
///
/// ⛔ **Abaixo de `0,80` não se mediu ganho:** `0,80` e `0,90` leem a mesma
/// grade (`64,9 %`), logo o que `0,80` compra é só o vinco, e ele já está ao
/// nível da malha **por pentear**.
pub(crate) const LADO_DA_CELULA: f32 = 0.80;

impl Sculpt3dScene {
    /// Liga/desliga. Devolve `(ligado, faces trianguladas)` — o segundo é zero
    /// quando a malha já era de triângulos, e é o número que o log mostra
    /// porque **triangular muda a malha** e uma mudança calada é a que o artista
    /// descobre no save.
    pub(super) fn toggle_dyntopo(&mut self) -> (bool, usize) {
        let on = !self.dyntopo.armed;
        self.dyntopo.armed = on;
        if !on {
            return (false, 0);
        }
        // ⚠️ **Triangula ao LIGAR, não no primeiro dab.** No primeiro dab a
        // mudança chegaria junto com o barro e o artista não teria como separar
        // *"a ferramenta mudou minha malha"* de *"a ferramenta esculpiu"*.
        let Some(o) = self.obj_mut() else {
            return (true, 0);
        };
        let added = o.stack.mesh_mut().triangulate();
        if added > 0 {
            self.mesh_rebuilt();
        }
        (true, added)
    }

    /// O rótulo do degrau atual — **a mesma tabela que a tecla percorre**, e é
    /// isso que impede o log de dizer "médio" enquanto o motor usa outro número.
    pub(super) fn detail_label(&self) -> &'static str {
        let actual = self.detalhe_do_gesto(&self.brush);
        DETAIL_STEPS
            .iter()
            .find(|(d, _)| (d - actual).abs() < 1e-6)
            .map_or("custom", |(_, l)| l)
    }

    /// O degrau seguinte do detalhe. Devolve o rótulo para o log.
    ///
    /// ⚠️⚠️ **Ele cicla o número que o GESTO EM MÃOS lê, e não sempre o da
    /// cena** — desde que o pincel de densidade ganhou alvo próprio (ordem do
    /// dono, 14/09) há **dois** sliders, e um atalho que escrevesse sempre no da
    /// cena seria uma tecla que não mexe no controlo que está à vista. A escolha
    /// vem da MESMA porta que o passe usa ([`Sculpt3dScene::detalhe_do_gesto`]).
    pub(super) fn cycle_detail(&mut self) -> &'static str {
        let actual = self.detalhe_do_gesto(&self.brush);
        let at = DETAIL_STEPS
            .iter()
            .position(|(d, _)| (d - actual).abs() < 1e-6)
            .unwrap_or(0);
        let (d, label) = DETAIL_STEPS[(at + 1) % DETAIL_STEPS.len()];
        if self.brush.offers_density_controls() {
            self.brush.density_detail = d;
        } else {
            self.dyntopo.detail = d;
        }
        label
    }

    /// **Refina onde o dab vai cair.** Chamado por [`Sculpt3dScene::sculpt_at`]
    /// ANTES do carimbo, e devolve `true` se a topologia mudou.
    ///
    /// ⚠️ **A ordem é a wave inteira: refinar e DEPOIS carimbar.** O contrário
    /// deposita o barro na malha grossa e adensa em cima — o detalhe nasce um
    /// dab atrasado, e o traço fica com a silhueta do que a malha era, não do
    /// que ela é.
    /// ⭐⭐ **O PINCEL DIZ PORQUE NÃO FEZ NADA** — uma vez por traço.
    ///
    /// ⚠️ **Report do dono, 2026-09-14: *«não vejo efeito com density»*.** O
    /// pincel estava certo e a queixa também: um verbo cujo efeito inteiro é
    /// sobre a topologia **parece partido** sempre que o passe não corre, e há
    /// **três** razões diferentes que o artista vê **iguais** — nada acontece.
    ///
    /// ⛔ Só para quem não tem outra forma de se mostrar
    /// ([`ph2d_sculpt3d::Verb::sem_lei_por_vertice`]): num `Draw` um passe que
    /// não parte nada é o caso **normal** (a malha já tem a densidade pedida
    /// ali), e uma linha de log por dab seria um log que ninguém lê.
    fn queixa_do_passe(&mut self, verbo: ph2d_sculpt3d::Verb, motivo: &str) {
        if !verbo.sem_lei_por_vertice() || self.dyn_queixa_dita {
            return;
        }
        self.dyn_queixa_dita = true;
        eprintln!("[sculpt3d] {} nao mudou a malha: {motivo}", verbo.label());
    }

    /// **QUE DENSIDADE ESTE GESTO PEDE?** — a porta que escolhe entre os DOIS
    /// sliders.
    ///
    /// ⭐⭐⭐ **ORDEM DO DONO (14/09): *«deixe o slider Detail para o dynamic
    /// Retopology e coloque outro slider Detail exclusivo para o pincel»*.** São
    /// duas perguntas que partilhavam um número — *quão fina a malha fica
    /// debaixo de um TRAÇO* contra *quão fina eu quero esta zona AGORA* — e
    /// separá-las é a consequência directa da ordem anterior (*«Dynamic topology
    /// é para os outros pincéis»*).
    ///
    /// ⚠️ **A escolha é feita AQUI e em lugar nenhum mais.** Ela vive numa porta
    /// e não num `if` no sítio de uso porque tem um segundo consumidor: a tecla
    /// `U`, que cicla **o mesmo número que o gesto em mãos lê**. *Dois sítios a
    /// escolher entre dois sliders é como o atalho passa a mexer no slider
    /// errado.*
    ///
    /// ⚠️ **Quem responde é o PINCEL** ([`ph2d_sculpt3d::Brush::offers_density_controls`]),
    /// que é a mesma porta que o painel consulta para oferecer a pista — senão
    /// haveria um slider visível a governar outra coisa.
    pub(super) fn detalhe_do_gesto(&self, brush: &ph2d_sculpt3d::Brush) -> f32 {
        if brush.offers_density_controls() {
            brush.density_detail
        } else {
            self.dyntopo.detail
        }
    }

    pub(super) fn refine_for_dab(
        &mut self,
        brush: &ph2d_sculpt3d::Brush,
        centre: [f32; 3],
    ) -> bool {
        let verbo = brush.verb;
        let radius = brush.radius;
        let detalhe = self.detalhe_do_gesto(brush);
        // ⭐⭐⭐ **O INTERRUPTOR NÃO ALCANÇA QUEM NÃO TEM TRAÇO** — ordem do dono
        // (14/09): *«independente se Dynamic topology está ligado ou não,
        // Density faz o seu trabalho. Dynamic topology é para os outros
        // pincéis.»*
        //
        // ⚠️ **DIVERGÊNCIA DECLARADA da referência** (espec §3.2, 1.ª linha da
        // tabela-verdade: o modo de detalhe em *Manual* desarma o passe inteiro,
        // este pincel incluído). O argumento é dele e é bom: aquele interruptor
        // responde *«o meu traço também muda a topologia?»*, e este verbo **não
        // tem traço**.
        //
        // ⛔ **A queixa do modo desligado MORREU com esta linha**, e não por
        // descuido: ela só falava por quem não tem lei por-vértice
        // ([`queixa_do_passe`]) — ou seja, exactamente por quem já não passa por
        // aqui. *Uma queixa inalcançável é pior que nenhuma: ela faz o censo
        // dizer três onde a verdade é duas.*
        if !self.dyntopo.armed && !verbo.corre_sem_o_interruptor() {
            return false;
        }
        // ⭐⭐⭐ **A PERGUNTA É AO VERBO, e até 2026-09-14 ela não era feita.**
        //
        // Esta porta tem **um** chamador de produto — o braço do carimbo —, logo
        // a pergunta que o produto respondia era *«este gesto passou pelo
        // caminho do carimbo?»* e não *«este verbo cria superfície nova?»*.
        // ⚠️ É a mesma família de defeito que este módulo já pagou três vezes ao
        // contrário: *inferir uma propriedade do VERBO a partir do CAMINHO que o
        // gesto tomou.*
        //
        // ⛔ **Report do dono:** *«algumas tools que não deveriam fazer a
        // subdivisão … estão fazendo (como smooth) enquanto algumas que deveriam
        // não estão»*. A tabela inteira é pergunta de ORÁCULO
        // (`docs/3D/22_plano_quem_subdivide_no_dyntopo.md`) — **uma** célula
        // dela não é, e é a que esta linha cura: a **MÁSCARA** não move um
        // vértice, e medido nesta cena ela levava a peça de `830` para `1 331`
        // vértices.
        if !verbo.refina_no_dyntopo() && !verbo.colapsa_no_dyntopo() {
            return false;
        }
        // ⚠️ **Recusa com a pilha montada** (ver o cabeçalho). Silenciosa aqui
        // de propósito: a mensagem sai no ARM, uma vez, em vez de por dab.
        if self.level_count() > 1 {
            self.queixa_do_passe(
                verbo,
                ph2d_i18n::tr("app.sculpt3d.dyntopo.pilha_montada_j_reverte"),
            );
            return false;
        }
        // ⭐⭐⭐ **O ALVO SAI DA PEÇA, NUNCA DO PINCEL** — ordem do dono
        // (*«a densidade da malha deve ser independente do zoom»*, 14/09) e o
        // mecanismo está em [`ph2d_mesh::edge_target_for_mesh`]: o
        // `Brush::radius` é derivado do raio em PIXELS **através da câmera** a
        // cada dab, logo o zoom entrava no alvo — medido, `4,9×` de alvo só por
        // aproximar ou afastar, com o mesmo pincel e o mesmo slider.
        //
        // ⚠️ **O raio continua a decidir a REGIÃO**, e é essa a separação que a
        // cura compra: *o pincel diz ONDE, o slider diz QUÃO FINO.* Enquanto as
        // duas perguntas partilhavam um número, mexer numa mexia na outra.
        let target = edge_target_for_mesh(self.objects[self.active].stack.mesh(), detalhe);
        // A peça ativa — a MESMA que o `sculpt_at` acabou de escolher pelo
        // `pick_active`, e é por isso que o índice basta aqui.
        //
        // ⚠️ O buffer de nascimentos é SCRATCH da cena, não estado do modo: o
        // `Dyntopo` guarda o que o artista autorou (o interruptor e o detalhe) e
        // segue `Copy`. Reusá-lo entre dabs é o que mantém o refino sem alocação
        // no caminho quente.
        // ⭐⭐⭐ **O PENTE ENTRA AQUI, e é ESTE o sítio onde o alinhamento nasce.**
        //
        // A metade de DESLOCAMENTO da lei ([`ph2d_rake::pentear`]) reproduz o
        // campo do alvo (`cos 0,971` contra `0,583` da lei que ela substituiu) e
        // **não produz grade nenhuma** (`Q +0,0000` contra a barra `+0,0465`):
        // *o alinhamento não mora no deslocamento.* Quem o produz é o alvo de
        // aresta deste passe passar a depender da DIRECÇÃO da aresta — ver
        // [`ph2d_sculpt3d::campo_do_pente`] para o mecanismo e para a hipótese
        // oposta, que foi construída e refutada.
        //
        // ⚠️ **A força passa pela MESMA porta que o carimbo lê**
        // ([`crate::space::pente_do_traco`]): com a topologia desarmada ela é
        // zero, e é isso que mantém a célula `porta/c_nodyn` do corpus — onde o
        // alvo não move um vértice — a bater.
        //
        // ⚠️ **E a direcção é lida ANTES do carimbo**, do `last_center` que ainda
        // descreve o dab anterior. ⛔ No primeiro carimbo ela é NULA e o campo
        // devolve o alvo nu: a inércia da espec §4.3 cai por construção, aqui
        // como no deslocamento, porque as duas metades leem a **mesma** porta.
        let forca = crate::space::pente_do_traco(self.dyntopo.armed, self.brush.pente);
        let direccao = self.stroke.direccao_do_traco(centre);
        let pente = (forca > 0.0 && verbo.honra_o_pente()).then_some(Pente {
            direccao,
            forca,
            queda: self.brush.falloff,
        });
        let mut births = std::mem::take(&mut self.dyn_births);
        let mut remap = std::mem::take(&mut self.dyn_remap);
        let mesh = self.objects[self.active].stack.mesh_mut();
        let mut region = std::mem::take(&mut self.dyn_region);
        // ⚠️ **O COLAPSO PRIMEIRO, e a ordem é a do canal.** As duas metades
        // falam com o traço em voo por canais diferentes — o colapso por uma
        // RENUMERAÇÃO, o refino por uma lista de NASCIMENTOS —, e o segundo
        // afirma que a malha cresceu exactamente o que ele partiu. Refinar antes
        // faria a renumeração chegar depois e descrever índices que já não são os
        // que o `grow_with` acabou de instalar.
        // ⚠️ **As duas colunas são lidas em separado**, e não porque algum verbo
        // hoje as separe: elas são **leis independentes** que por acaso vivem na
        // mesma porta (um verbo pode querer relaxar densidade sem criar
        // detalhe), e o estudo pode separá-las. *Uma porta que lê uma coluna só
        // obriga quem a preenche a escolher pelas duas.*
        // ⚠️⚠️ **As duas colunas são lidas em SEPARADO, e o `&&` curto-circuita** —
        // um verbo que diga `false` não chega a chamar o motor. Elas são **leis
        // independentes** que por acaso vivem na mesma porta (um verbo pode
        // querer relaxar densidade sem criar detalhe), e o estudo pode separá-las.
        // *Uma porta que lê uma coluna só obriga quem a preenche a escolher
        // pelas duas.*
        //
        // ⛔ E o veredito é lido como **booleano** e não por um estado novo nos
        // enums da `ph2d-mesh`: `Collapse::Enough` significa *«nenhuma aresta
        // está sob o limiar»*, que é um facto sobre a MALHA — usá-lo para dizer
        // *«o verbo não pediu»* poria duas coisas diferentes no mesmo byte, que
        // é o defeito que este módulo acabou de pagar noutro sítio.
        let (cut, done, arrumou) = passe_nos_motores(
            mesh,
            verbo,
            target,
            centre,
            radius,
            Rascunho {
                remap: &mut remap,
                births: &mut births,
                region: &mut region,
            },
            pente,
        );
        self.dyn_region = region;
        if cut {
            // ⚠️ **Antes do `grow_with`, sempre.** Ele afirma que a malha cresceu
            // exactamente o número de nascimentos que chegaram, e a conta é
            // contra `slot.len()` — que ainda descreve a malha de antes do
            // colapso enquanto ninguém aplicar o remap.
            self.stroke.shrink_with(&remap);
        }
        self.dyn_remap = remap;
        if done {
            // O traço em voo sobrevive: os índices antigos não se moveram, e cada
            // vértice novo HERDA o `pre` do par que o gerou. Ver
            // `SculptStroke::grow_with` — chamar `begin` aqui jogaria fora o
            // `pre`, e tratar o novo como nunca-visto conta o deslocamento do
            // traço duas vezes (a agulha).
            let mesh = self.objects[self.active].stack.mesh();
            self.stroke.grow_with(mesh, &births);
        }
        self.dyn_births = births;
        if !done && !cut && !arrumou {
            // ⚠️ **A razão mais provável, e a mais difícil de adivinhar de
            // fora:** o alvo de aresta é `raio × f(detalhe)`, logo a malha pode
            // já estar exactamente no ponto que o slider pede.
            //
            // ⚠️⚠️ **Esta razão ENCOLHEU em 2026-09-14 e a queixa foi reescrita
            // com ela.** Enquanto a densidade só colapsava, ela disparava em
            // metade do curso do slider — *«apenas no grosso vi alguma coisa
            // acontecendo»* (report do dono) —, e o texto mandava **baixar** o
            // detalhe, que é conselho de um pincel que só afina. Hoje o ajuste
            // de omissão leva a malha ao alvo nos DOIS sentidos, logo chegar
            // aqui quer mesmo dizer *já está no ponto*; quem só afina escolheu
            // esse ajuste e a cura dele continua a ser baixar o detalhe.
            self.queixa_do_passe(
                verbo,
                ph2d_i18n::tr("app.sculpt3d.dyntopo.ja_no_ponto_que_o_detail_pede"),
            );
            return false;
        }
        // A malha tem faces novas: o upload incremental não as descreve.
        self.mesh_rebuilt();
        true
    }
}

/// **Quem muda a topologia em dyntopo** — ver [`tests`].
#[cfg(test)]
#[path = "dyntopo_tests.rs"]
mod tests;

/// ⭐⭐⭐ **OS DOIS MOTORES, SEM CENA E SEM DEVICE** — o miolo do
/// [`Sculpt3dScene::refine_for_dab`], com **dois** chamadores.
///
/// # Porque ela é uma porta e não um bloco lá dentro
///
/// ⛔⛔ **O censo dos knobs declarava o [`ph2d_sculpt3d::Verb::Density`] como
/// ADORMECIDO**, com a saída escrita na própria catraca: *«o arnês teria de
/// correr o `refine_for_dab` e comparar a CONTAGEM de vértices em vez das
/// posições»*. E não podia: a [`Sculpt3dScene`] pede um `wgpu::Device` para
/// nascer, logo o censo passaria a ser `#[ignore]` e **o CI deixaria de o
/// correr** — trocando um verbo por cobertura em todos os outros.
///
/// ⛔ **E chamar os motores soltos a partir do censo era a alternativa errada:**
/// o cabeçalho daquele ficheiro declara que *«a régua é o PRODUTO, nunca as
/// funções soltas»*, e uma segunda cópia da ordem colapso→refino divergiria da
/// primeira no dia em que o estudo separasse as duas colunas.
///
/// ⇒ **uma lei, dois chamadores.** O que fica na cena é o que precisa dela: as
/// três recusas, o alvo de aresta, a costura com o traço em voo
/// (`shrink_with`/`grow_with`), a queixa e o `mesh_rebuilt`.
///
/// # ⚠️ A ORDEM é load-bearing, e é por isso que ela viaja aqui dentro
///
/// **O colapso primeiro.** As duas metades falam com o traço em voo por canais
/// diferentes — o colapso por uma RENUMERAÇÃO, o refino por uma lista de
/// NASCIMENTOS —, e o segundo afirma que a malha cresceu exactamente o que ele
/// partiu. Refinar antes faria a renumeração chegar depois e descrever índices
/// que já não são os que o `grow_with` instalou. *Uma ordem que vive no corpo de
/// quem chama é uma ordem que o segundo chamador pode escrever ao contrário.*
///
/// # ⚠️ As duas colunas são lidas em SEPARADO, e o `&&` curto-circuita
///
/// Um verbo que diga `false` não chega a chamar o motor. Elas são **leis
/// independentes** que por acaso vivem na mesma porta (um verbo pode querer
/// relaxar densidade sem criar detalhe), e o estudo pode separá-las.
///
/// ⛔ O veredito é **booleano** e não um estado novo nos enums da `ph2d-mesh`:
/// `Collapse::Enough` é um facto sobre a MALHA (*«nenhuma aresta está sob o
/// limiar»*), e usá-lo para dizer *«o verbo não pediu»* poria duas coisas
/// diferentes no mesmo byte.
///
/// **O RASCUNHO do passe** — os três buffers que ele reusa, num só argumento.
///
/// ⚠️ **Eles são UM conceito e não três parâmetros**: os dois chamadores já os
/// seguram juntos (a cena guarda-os em campos `dyn_*` para o caminho quente não
/// alocar, o censo declara-os na mesma linha), e agrupá-los é o que mantém a
/// porta dentro do tecto de argumentos — ⛔ nunca um `allow` por cima do aviso.
pub(crate) struct Rascunho<'a> {
    pub(crate) remap: &'a mut ph2d_mesh::Remap,
    pub(crate) births: &'a mut Vec<ph2d_mesh::Birth>,
    pub(crate) region: &'a mut ph2d_mesh::RegionScratch,
}

/// Devolve `(colapsou, refinou, arrumou_na_grelha)`.
///
/// ⚠️ **A terceira é obrigatória e não é conforto:** a retícula move vértices
/// sem mudar a contagem, logo um carimbo em que só ela trabalhe não acorda nem
/// o `cut` nem o `done` — e sem a terceira o quadro sairia com as posições
/// velhas na tela **com a malha certa na CPU**.
/// **O que o passe precisa de saber sobre o PENTE** — `None` quando ele está
/// desligado, quando o verbo não o honra, ou no PRIMEIRO carimbo (sem direcção).
///
/// ⭐⭐⭐ **É por aqui que o alinhamento entra no produto**, e a razão está medida
/// em [`ph2d_sculpt3d::campo_do_pente`]: a lei de deslocamento reproduz o campo do
/// alvo (`cos 0,971`) e **não produz grade nenhuma** (`Q +0,0000`); quem produz
/// a grade é o passe de topologia a receber um alvo de aresta que depende da
/// DIRECÇÃO da aresta.
#[derive(Clone, Copy)]
pub(crate) struct Pente {
    pub(crate) direccao: [f32; 3],
    pub(crate) forca: f32,
    /// A queda do pincel — a **NUA**, sem a dureza, pela mesma razão que o
    /// deslocamento já escrevia: a dureza é do VERBO em mãos e mudaria o
    /// alcance de uma coisa que não é do verbo.
    pub(crate) queda: ph2d_sculpt3d::Falloff,
}

pub(crate) fn passe_nos_motores(
    mesh: &mut ph2d_mesh::Mesh,
    verbo: ph2d_sculpt3d::Verb,
    alvo_de_aresta: f32,
    centre: [f32; 3],
    radius: f32,
    rascunho: Rascunho<'_>,
    pente: Option<Pente>,
) -> (bool, bool, bool) {
    let Rascunho {
        remap,
        births,
        region,
    } = rascunho;
    // ⚠️ **As duas portas passam SEMPRE pela variante `_sized`**, e não há um
    // braço para «sem pente»: `Sizing = None` é **byte-idêntico** à porta nua
    // (a `collapse_in_sphere` literalmente delega nela), logo um `match` aqui
    // seria duas escritas da mesma chamada — e a que alguém esquecesse de
    // emendar era a que o pente desligado percorre, ou seja a de fábrica.
    let mut arrumou = false;
    if let Some(p) = pente {
        let mut andaram = Vec::new();
        let queda = |q: [f32; 3]| {
            let d = [q[0] - centre[0], q[1] - centre[1], q[2] - centre[2]];
            let r = d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt();
            p.forca * p.queda.weight(r / radius.max(f32::MIN_POSITIVE))
        };
        if ph2d_quadflow::regiao::arruma_na_grelha_com(
            mesh,
            centre,
            radius,
            p.direccao,
            &queda,
            RONDAS_DA_GRELHA,
            LADO_DA_CELULA,
            &mut andaram,
        ) > 0
        {
            mesh.refresh_region(&andaram, region);
            arrumou = true;
        }
    }

    let alvo_do_colapso = collapse_target(alvo_de_aresta);
    // ⛔⛔⛔ **O CAMPO DE TAMANHO POR DIRECÇÃO SAIU**, e a medição está na
    // terceira metade, lá em baixo: ele era metade da lei que o dono reprovou
    // em 19/09 (*«pior que o original, com irregularidade a 90 graus da
    // direcção do movimento»*), e o que ele faz é prescrever a anisotropia —
    // pedir arestas mais curtas numa direcção do que na outra. A retícula não
    // precisa dele: ela alinha **e** iguala o espaçamento pela mesma
    // construção. *A [`ph2d_sculpt3d::campo_do_pente`] fica, com os gates e a
    // bancada dela; o que saiu foi o CHAMADOR.*
    let campo_colapso: Option<&(dyn Fn([f32; 3], [f32; 3]) -> f32 + Sync)> = None;
    let campo_refino: Option<&(dyn Fn([f32; 3], [f32; 3]) -> f32 + Sync)> = None;
    let cut = verbo.colapsa_no_dyntopo()
        && matches!(
            // ⭐⭐⭐ **A QUINTA GUARDA é pedida AQUI, e não escrita no motor.**
            // Ela recusa um colapso que vire uma face do avesso — o defeito que
            // o dono fotografou em 19/09 (*«pior que o original, com
            // irregularidade a 90 graus da direcção do movimento»*). ⛔ Escrita
            // dentro do `plan`, ela alcançava o remalhador isotrópico e a cadeia
            // de retopologia inteira: **onze** gates vermelhos numa obra que esta
            // wave não toca. *Um motor partilhado não muda de lei por causa de um
            // consumidor* — ver [`ph2d_mesh::Guarda`].
            ph2d_mesh::collapse_in_sphere_com(
                mesh,
                centre,
                radius,
                alvo_do_colapso,
                campo_colapso,
                ph2d_mesh::Guarda::ETambemAForma,
                remap,
                region,
            ),
            Collapse::Done { .. }
        );
    let done = verbo.refina_no_dyntopo()
        && matches!(
            refine_in_sphere_sized(
                mesh,
                centre,
                radius,
                alvo_de_aresta,
                campo_refino,
                births,
                region
            ),
            Refine::Done { .. }
        );
    // ⭐⭐⭐⭐ **A TERCEIRA METADE — A RETÍCULA**, e ela substituiu as outras duas
    // leis do pente por ORDEM DO DONO (*«vamos modificar completamente esse
    // algoritmo … traga o estado da arte»*, 19/09).
    //
    // A classe é a do **campo de posição** do *Instant Field-Aligned Meshes*: em
    // vez de escolher que triângulos se ligam, dá-se a cada vértice **o ponto de
    // uma grelha quadrada** alinhada com o traço e manda-se ele para lá ⇒
    // *alinhamento e espaçamento igual são a MESMA construção*, e por isso ela
    // não compra um à custa do outro.
    //
    // ⭐ **Medido na peça da cena `=49`, os quatro rumos** (`grade` é a fracção
    // das arestas a menos de `15°` da grade do traço; `vinco` é o ângulo entre
    // normais vizinhas, que é o que a LUZ mostra):
    //
    // | lei | grade | vinco p50 | vinco p90 |
    // |---|---|---|---|
    // | por pentear | `32`–`39 %` | `0,92`–`1,38°` | `2,58`–`2,65°` |
    // | a que o dono REPROVOU | `42`–`45 %` | `1,75`–`1,89°` | `4,13`–`4,35°` |
    // | ⭐ **a retícula** | **`64`–`65 %`** | **`0,88`–`1,03°`** | `2,64`–`2,86°` |
    //
    // ⇒ ela **quebra a troca** que o §81 tinha medido (*«alinhamento e ondulação
    // são o mesmo botão»*): metade outra vez do alinhamento, com o relevo ao
    // nível da malha por pentear.
    //
    // ⛔⛔ **As outras duas metades SAEM do caminho do produto, e não do
    // código:** o campo de tamanho por direcção ([`ph2d_sculpt3d::campo_do_pente`])
    // deixa de entrar nos dois motores (`None` acima) e a troca de diagonal
    // ([`ph2d_mesh::alinha_arestas`]) deixa de ser chamada daqui — as duas
    // continuam com os gates e a bancada de paridade delas, que é o que mede a
    // lei do ALVO. *A ordem do dono foi trocar o algoritmo, não apagar a
    // medição.*
    //
    // ⭐⭐⭐⭐ **ELA CORRE ANTES DOS DOIS MOTORES, e a ordem foi MEDIDA — a minha
    // primeira redacção dizia o contrário e o gate da cena reprovou-a.**
    //
    // Com a retícula em ÚLTIMO, a faixa fica com `3` triângulos abaixo de `5°`
    // em `2 418` (o pior a `0,26°`), onde a lei que ela substituiu deixava
    // **zero**. ⚠️ E a cerca de forma que ela ganhou (`lasca`, que recusa um
    // destino que afine um triângulo do anel) **não os apanha**: ela julga um
    // vértice de cada vez contra as posições de entrada, e um empilhamento é
    // feito por DOIS vizinhos que, cada um por si, não afinam nada.
    //
    // ⭐ **Em PRIMEIRO, zero.** O mecanismo é direto: dois vértices no mesmo
    // ponto de retícula são uma **aresta curta**, e uma aresta curta é
    // exactamente o que o colapso existe para comer. *Arrumar e depois limpar;
    // limpar e depois arrumar deixa por limpar o que o último carimbo arrumou.*
    (cut, done, arrumou)
}
