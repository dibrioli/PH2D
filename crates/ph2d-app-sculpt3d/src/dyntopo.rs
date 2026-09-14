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
    Collapse, Refine, collapse_in_sphere, collapse_target, edge_target_for_mesh, refine_in_sphere,
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
                "ha' uma pilha de multiresolucao montada -- J reverte-a",
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
        let cut = verbo.colapsa_no_dyntopo()
            && matches!(
                collapse_in_sphere(
                    mesh,
                    centre,
                    radius,
                    collapse_target(target),
                    &mut remap,
                    &mut region,
                ),
                Collapse::Done { .. }
            );
        let done = verbo.refina_no_dyntopo()
            && matches!(
                refine_in_sphere(mesh, centre, radius, target, &mut births, &mut region),
                Refine::Done { .. }
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
        if !done && !cut {
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
                "a malha aqui ja' esta' no ponto que o Detail pede -- mova o \
                 slider (ou a tecla U) para pedir outra densidade, ou aumente o \
                 pincel com ] para alcancar mais peca",
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
