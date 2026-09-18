//! **ONDE as coisas estão** — as portas de espaço da cena.
//!
//! Filho (`#[path]`) de [`super`] para alcançar os campos privados; o corte é *o
//! que a cena É e o que a mão FAZ* (lá) contra *em que ESPAÇO cada número vive*
//! (aqui). É o assunto que a W8.1 abriu: com uma malha só, "onde" não era uma
//! pergunta — a geometria era o mundo. Com uma lista de objetos ela passa a ser,
//! e toda conversão entre os dois espaços mora neste arquivo, num lugar só.

use super::{
    Brush, Camera3d, Hit, Mesh, ObjectId, Pose, RADIUS_MIN_PX, Ray, SceneObject, Sculpt3dScene,
};

/// ⭐⭐⭐ **A LEI DE «ESTA PEÇA APARECE?», pura** — as duas metades do
/// [`Sculpt3dScene::visible_pieces`], sem cena e sem device.
///
/// ⛔⛔ **Ela é uma função livre de propósito, e a razão é um gate:** a cena
/// pede um `wgpu::Device` para nascer, logo um teste sobre ela nasceria
/// `#[ignore]` e **o CI nunca o correria** — a mesma lição que a
/// [`crate::recusa`] pagou. *Quando um gate precisa de um device para medir
/// uma decisão que não tem pixel nenhum, a lei está no sítio errado.*
///
/// ⚠️ **O `isolada` que entra é a que EXISTE**, já resolvida pelo
/// [`Sculpt3dScene::isolated_index`]: uma isolada que morreu não isola, e essa
/// cláusula não cabe aqui porque ela é sobre a LISTA e não sobre a peça.
pub(crate) fn aparece(
    id: ObjectId,
    isolada: Option<ObjectId>,
    escondidas: &std::collections::BTreeSet<ObjectId>,
) -> bool {
    isolada.is_none_or(|k| k == id) && !escondidas.contains(&id)
}

/// ⛔⛔ **O PENTE QUE O TRAÇO RECEBE — a pré-condição da espec §2.1, e a ÚNICA
/// de estado que ele tem.**
///
/// Com a topologia dinâmica **desarmada** os dois lados do controlo dão a MESMA
/// malha, byte a byte — logo o que chega ao traço é `0`. ⚠️ E ela é a única: o
/// pente **NÃO** depende de o passe de refino correr (§2.2), e sem refino nenhum
/// ele continua a agir, com efeito **MAIOR**.
///
/// ⚠️ **Zerado AQUI e não lido lá dentro**, porque o [`ph2d_sculpt3d::SculptStroke`]
/// não sabe o que é o interruptor da cena — ensinar-lho seria a segunda resposta
/// a *«a topologia dinâmica está ligada?»*, a família de defeito que este módulo
/// já pagou com os três chips do detalhe.
///
/// ⛔⛔ **Função LIVRE pela mesma razão que a [`aparece`] acima:** a cena pede um
/// `wgpu::Device` para nascer, e um gate sobre uma decisão que não tem pixel
/// nenhum nasceria `#[ignore]` — o CI nunca o correria.
///
/// ⚠️⚠️ **E a nota que aqui esteve era FALSA:** ela dizia que *«a fileira dele só
/// é oferecida com o interruptor armado»*. Não é — o `dyntopo` é um FACTO do
/// retrato do painel e não um campo do estado autorado, logo o `show` de uma
/// fileira não lhe chega; a pista fica **sempre** e o painel **diz que ela
/// dorme** (`panel.sculpt3d.pente_dormente`). *Um doc que declara a lei que o
/// código não implementa lê-se como auditado.*
pub(crate) fn pente_do_traco(dyntopo_armado: bool, pente: f32) -> f32 {
    // ⚠️ A lei mudou-se para a crate do PINCEL, onde a bancada de paridade lhe
    // chega — aqui fica só a delegação, para os chamadores e o gate desta
    // crate não mudarem de endereço.
    ph2d_sculpt3d::pente_do_traco(dyntopo_armado, pente)
}

impl Sculpt3dScene {
    /// Acrescenta um objeto à cena. Devolve o índice dele.
    ///
    /// ⚠️ **Ele NÃO vira o ativo.** Montar uma cena é pôr peças na mesa, não
    /// pegar cada uma: quem escolhe é o pincel do artista, no clique. Uma
    /// fixture que quisesse o contrário diria isso explicitamente.
    pub(crate) fn push_object(&mut self, mesh: Mesh, pose: Pose) -> usize {
        let id = self.mint_id();
        self.objects.push(SceneObject::new(id, mesh, pose));
        self.objects.len() - 1
    }

    /// Cunha um [`ObjectId`] novo. **Nunca reusa** — ver o doc do tipo.
    pub(super) fn mint_id(&mut self) -> ObjectId {
        let id = ObjectId(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        id
    }

    /// Onde na lista está a peça `id`, se ela ainda existe.
    pub(super) fn index_of(&self, id: ObjectId) -> Option<usize> {
        self.objects.iter().position(|o| o.id == id)
    }

    /// **Onde está a peça ISOLADA** — `None` quando a cena inteira está à vista.
    ///
    /// ⚠️ **Ela resolve o id para um índice a cada pergunta, e é isso que faz o
    /// isolamento sobreviver a um delete.** A peça isolada pode morrer (apagar,
    /// desfazer, fundir), e um índice guardado passaria a nomear OUTRA peça — a
    /// mesma lição que o [`ObjectId`] existe para dizer, um degrau acima. Quando
    /// ela não existe mais **não há isolamento**: a cena inteira volta à vista,
    /// que é a única resposta que não deixa o artista com uma tela preta e
    /// nenhum gesto capaz de a explicar.
    pub(super) fn isolated_index(&self) -> Option<usize> {
        self.isolated.and_then(|id| self.index_of(id))
    }

    /// **AS PEÇAS À VISTA**, na ordem da lista — a porta única de *quem aparece*.
    ///
    /// ⚠️ Ela é perguntada pelo DESENHO, pelo PICK e pela CAIXA, e é por isso que
    /// é uma porta: as três respondem a *o que está na cena agora*, e uma cópia
    /// da regra em qualquer uma delas seria a que diverge — um pick que alcança
    /// o que não se vê é esculpir às cegas, e uma caixa que inclui o invisível
    /// enquadra a câmera em volta do nada.
    /// ⭐⭐ **E desde 2026-09-15 ela tem DUAS metades:** o isolamento, e o OLHO da
    /// Hierarquia ([`Sculpt3dScene::escondidas`]). As duas escondem, e escondem
    /// pelas mesmas razões — *um pick que alcança o que não se vê é esculpir às
    /// cegas* —, logo elas pertencem à mesma porta.
    pub(super) fn visible_pieces(&self) -> impl Iterator<Item = usize> + '_ {
        // ⚠️ **O id da isolada resolve-se ANTES do laço, e pela porta** — ver
        // [`Self::isolated_index`]: uma isolada que já morreu **não isola**, e
        // passar o `self.isolated` cru para a lei pura perderia essa cláusula.
        let so = self.isolated_index().map(|k| self.objects[k].id);
        (0..self.objects.len()).filter(move |&i| aparece(self.objects[i].id, so, &self.escondidas))
    }

    /// ⭐⭐⭐ **AS OUTRAS PEÇAS QUE UM PINCEL PODE VER** — a porta única de *contra
    /// que é que este gesto mede*, com **três** consumidores: o pen-down do
    /// traço, o pen-down do filtro de tecido e a recusa em voz alta.
    ///
    /// ⛔⛔ **Ela nasceu porque o laço estava escrito DUAS vezes**, letra a
    /// letra, em dois pen-downs — e a espec §6.1 acrescentou-lhe uma cláusula
    /// (*«todo objecto que não é o activo, É malha, e NÃO está escondido»*) que
    /// um dos dois teria herdado e o outro não. *Duas cópias de um filtro são
    /// duas respostas à mesma pergunta, e a que diverge é a que ninguém
    /// relê.*
    ///
    /// ⚠️ **E a terceira consumidora é a que torna a cura honesta:** sem ela, um
    /// traço cujo único alvo está escondido moveria zero vértices **e ficaria
    /// calado** — exactamente o defeito que o [`crate::recusa`] existe para não
    /// ter. *Filtrar a lista sem filtrar a contagem troca um pincel inerte por
    /// um pincel inerte e mudo.*
    ///
    /// ⚠️ **Devolve ÍNDICES e não malhas**, e é essa a razão de ela não devolver
    /// já os pares `(Mesh, Pose)`: a recusa só precisa de os **contar**, e uma
    /// porta que clonasse as malhas para responder «quantas?» faria o pen-down
    /// pagar uma cópia da cena inteira por um `usize`.
    pub(super) fn alvos_visiveis(&self) -> impl Iterator<Item = usize> + '_ {
        let activo = self.active;
        self.visible_pieces().filter(move |&i| i != activo)
    }

    /// A caixa dos objetos **À VISTA**, em mundo.
    pub(crate) fn world_bounds(&self) -> ph2d_mesh::Aabb {
        let mut b = ph2d_mesh::Aabb::EMPTY;
        for i in self.visible_pieces() {
            let o = &self.objects[i];
            let ob = o.pose.bounds_to_world(o.stack.mesh().bounds());
            if !ob.is_empty() {
                b.expand(&ob);
            }
        }
        b
    }

    /// Reenquadra a câmera em **toda a cena**.
    ///
    /// ⚠️ Toda, e não no objeto ativo: enquadrar o ativo esconderia as outras
    /// peças no instante em que o artista clicasse numa delas, que é o oposto do
    /// que uma cena com mais de um objeto existe para permitir.
    pub(crate) fn frame_all(&mut self, aspect: f32) {
        let b = self.world_bounds();
        self.camera.frame(b, self.view_aspect(aspect));
    }

    /// ⭐⭐ **A RAZÃO DE ASPECTO DA VISTA** — a do quadrante activo, ou `fallback`
    /// antes do primeiro desenho.
    ///
    /// ⛔⛔ **Ela existe porque a wave dos viewports criou uma SEGUNDA resposta a
    /// «qual é o aspecto?»** (2026-09-08): o desenho passou a usar o do
    /// rectângulo da vista e os chamadores do enquadramento continuavam a passar
    /// o da **JANELA**. Numa janela `1920×1080` com o canvas em `1520×950` isso
    /// é `1,78` contra `1,60` — o *fit* punha a peça `11 %` mais perto do que a
    /// vista comporta, e com a divisão aberta a diferença passa a ser de
    /// **dobro** (um quadrante é quase quadrado).
    ///
    /// ⚠️ **O argumento fica como FALLBACK e não sai**: no nascimento da cena o
    /// canvas ainda não foi publicado — o quadro publica-o depois —, e ali o
    /// aspecto da janela é a melhor resposta que existe.
    pub(crate) fn view_aspect(&self, fallback: f32) -> f32 {
        let (w, h) = self.viewport();
        if w <= 1 || h <= 1 {
            return fallback;
        }
        w as f32 / h as f32
    }

    /// **O objeto que a mão está trabalhando.**
    ///
    /// Porta única: *qual malha esta cena esculpe* passou a ter uma resposta que
    /// depende de onde o artista clicou por último, e um segundo caminho até ela
    /// seria a segunda cópia dessa resposta.
    ///
    /// ⚠️ Onde o borrow checker precisa de campos disjuntos (o `self.stroke` e a
    /// malha na MESMA expressão) o mesmo acesso aparece escrito por extenso,
    /// `self.objects[self.active]`. É a mesma porta, não uma segunda — o índice
    /// é o que ela devolve.
    /// A peça ativa — **`None` numa cena VAZIA**.
    ///
    /// ⚠️ **Ela devolvia `&SceneObject` e a lista era nunca-vazia**, o que
    /// tornava *"apagar a última"* uma recusa: o Enio a reportou como *"não
    /// consigo deletar todos os objetos da tela"*. A invariante existia para
    /// esta função ser total, e trocá-la por `Option` é a representação
    /// admitindo o que já era verdade — **uma cena vazia é um estado que o
    /// artista pode querer**, e nenhum gesto tem resposta sem peça.
    pub(super) fn obj(&self) -> Option<&SceneObject> {
        self.objects.get(self.active)
    }

    /// A peça ativa, para escrever — **`None` numa cena VAZIA**.
    pub(super) fn obj_mut(&mut self) -> Option<&mut SceneObject> {
        self.objects.get_mut(self.active)
    }

    /// Onde o objeto ativo está — ver [`Pose`].
    ///
    /// ⚠️ **`IDENTITY` numa cena vazia, e é o valor CERTO:** não há peça para
    /// posicionar, e todo consumidor desta função usa a pose para transformar
    /// algo que também não existe. Devolver `Option` aqui espalharia o `?` por
    /// vinte sítios para descrever o mesmo nada.
    pub(super) fn pose(&self) -> Pose {
        self.obj().map_or(Pose::IDENTITY, |o| o.pose)
    }

    /// A malha do nível VIVO — a que o artista vê e esculpe.
    ///
    /// ⚠️ Porta, e não campo, desde que a pilha existe: *qual malha é esta cena*
    /// passou a ter uma resposta que depende do nível selecionado, e um campo
    /// paralelo seria a segunda cópia dessa resposta.
    /// ⚠️ **Numa cena VAZIA devolve a malha vazia**, e não um `Option`: os
    /// consumidores desenham, medem caixa e fazem raycast — e as três respostas
    /// certas sobre nada são *nada desenhado*, *caixa vazia* e *nenhum acerto*,
    /// que é exatamente o que uma `Mesh` sem vértices já dá. Um `Option` aqui
    /// obrigaria vinte sítios a escrever o mesmo `else` para chegar ao mesmo
    /// lugar.
    pub fn mesh(&self) -> &Mesh {
        static EMPTY: std::sync::OnceLock<Mesh> = std::sync::OnceLock::new();
        self.obj().map_or_else(
            || EMPTY.get_or_init(|| Mesh::from_parts(Vec::new(), Vec::new()).expect("vazia")),
            |o| o.stack.mesh(),
        )
    }

    /// **O nível selecionado da peça ativa** — `0` numa cena vazia.
    ///
    /// ⚠️ Porta única para a pergunta que o desfazer faz doze vezes (*em que
    /// nível esta entrada foi gravada?*). Sem ela, cada braço do `apply_entry`
    /// escreveria o próprio `else` para a cena vazia — doze cópias de uma
    /// resposta só.
    pub(super) fn level(&self) -> usize {
        self.obj().map_or(0, |o| o.stack.level())
    }

    /// **Escolhe o nível da peça ativa** — no-op numa cena vazia.
    pub(super) fn select_level(&mut self, k: usize) {
        if let Some(o) = self.obj_mut() {
            o.stack.select(k);
        }
    }

    /// Quantos níveis a peça ativa tem — `0` numa cena vazia.
    pub(super) fn level_count(&self) -> usize {
        self.obj().map_or(0, |o| o.stack.level_count())
    }

    /// A malha do nível vivo, para escrever — **`None` numa cena vazia**.
    ///
    /// ⚠️ Aqui o `Option` FICA, ao contrário da leitura: escrever exige um alvo,
    /// e um alvo-sentinela mutável aceitaria a escrita e a jogaria fora — um
    /// traço que não aparece, sem nada dizendo por quê.
    pub(super) fn mesh_mut(&mut self) -> Option<&mut Mesh> {
        Some(self.obj_mut()?.stack.mesh_mut())
    }

    /// O raio autorado, **já clampado contra a tela desta janela**.
    ///
    /// Porta única, e é ela que faz um `resize` re-clampar sozinho: o cru é o
    /// estado autorado e o limite é do viewport, então guardar o clampado seria
    /// o mesmo número em dois lugares — e o segundo fica velho no primeiro
    /// arrasto de janela.
    pub(super) fn radius_px(&self) -> f32 {
        let (w, h) = self.viewport();
        self.radius_px
            .clamp(RADIUS_MIN_PX, super::radius_ceiling_px(w, h))
    }

    /// O pincel com o raio resolvido, para um ponto em coordenadas **LOCAIS** do
    /// objeto ativo.
    ///
    /// ⚠️ **O raio é função do ACERTO, não do pincel** — o mesmo pincel cobre
    /// menos mundo perto da câmera e mais longe dela, que é o que "tamanho em
    /// pixels" significa. Guardar um raio de mundo no `Brush` seria o mesmo
    /// número em dois lugares, e o segundo ficaria velho a cada `dolly`.
    ///
    /// ⚠️ **E ele atravessa a POSE duas vezes, nos dois sentidos.** A câmera só
    /// sabe responder sobre o mundo, então o ponto sobe (`point_to_world`) para
    /// a pergunta e o raio desce (`/ scale`) para a resposta — porque quem o
    /// consome é o dab, que mede na régua da malha. Sem a segunda metade um
    /// objeto ao dobro do tamanho receberia uma pegada com METADE do diâmetro
    /// aparente, e o artista leria isso como *"o pincel encolheu"*.
    pub(super) fn armed_brush(&self, local_at: [f32; 3]) -> Brush {
        self.armed_brush_on(self.pose(), local_at)
    }

    /// **O mesmo pincel, para uma peça que não é a ACTIVA.**
    ///
    /// ⚠️ **Existe porque o INDICADOR pergunta por outra peça:** o dab corre
    /// sempre na activa (o `aim` do pen-down garante-o), mas o osso da pose
    /// desenha-se ao sobrevoar — e ali a peça sob o cursor pode ser outra, com
    /// **outra escala**. *Copiar as três linhas para lá seria a terceira cópia
    /// desta conversão, e a que ficaria para trás no dia em que ela mudar.*
    pub(super) fn armed_brush_on(&self, pose: Pose, local_at: [f32; 3]) -> Brush {
        let world = pose.point_to_world(local_at);
        let radius =
            self.camera
                .world_radius_for_screen_px(world, self.radius_px(), self.viewport());
        Brush {
            radius: (radius / pose.scale()).max(1e-6),
            alpha_stencil: Some(self.stencil_for(pose)),
            // ⭐ Todo dab da app vem de um traço ARRASTADO a passos fixos — o pincel de plano
            // enfraquece cada um para a soma não depender do passo (espec §14.4).
            traco_arrastado: true,
            pente: pente_do_traco(self.dyntopo.armed, self.brush.pente),
            ..self.brush.clone()
        }
    }

    /// **A VISTA, do ponto de vista desta peça** — o que faz de uma imagem um
    /// ESTÊNCIL preso ao viewport (ver [`ph2d_sculpt3d::AlphaStencil`]).
    ///
    /// ⚠️ **Ela é carimbada SEMPRE, e quem decide se ela governa é o PINCEL.**
    /// A pergunta *"este padrão é um carimbo?"* já tem dono lá dentro
    /// (`Brush::alpha_frame`), e repeti-la aqui seria a segunda cópia de uma
    /// regra que já mudou uma vez nesta linha.
    ///
    /// ⚠️ **Ela NÃO recebe um ponto, e a ausência é a correção.** O primeiro
    /// corte pedia *onde medir a régua da vista*, e os dois consumidores
    /// respondiam diferente — o dab no ACERTO, o preview no CENTRO da peça —, o
    /// que fazia a tinta desenhada no barro divergir da depositada em **+24,8%**
    /// de tamanho na frente do modelo (Enio, 2026-08-09). Entregando o FRUSTUM,
    /// a profundidade passa a entrar por vértice e não há mais o que escolher:
    /// duas chamadas com a mesma pose devolvem o mesmo estêncil por construção.
    ///
    /// ⚠️ **`right`/`up` continuam ortonormais em espaço local** porque a pose
    /// deste módulo escala por um ESCALAR; uma escala não-uniforme cisalharia o
    /// par e o frame deixaria de ser uma base.
    pub(super) fn stencil_for(&self, pose: Pose) -> ph2d_sculpt3d::AlphaStencil {
        stencil_of(&self.camera, self.viewport(), pose)
    }

    /// **Quem o cursor aponta, e ONDE nele** — `(objeto, acerto em coordenadas
    /// LOCAIS desse objeto)`.
    ///
    /// ⚠️ **O RAIO é levado ao espaço de cada malha, e não a geometria ao
    /// mundo.** O caminho oposto custaria uma cópia da malha por consulta e
    /// invalidaria o octree, que é construído em espaço local; assim a consulta
    /// continua sendo exatamente a que a W1 gateou e o acerto já volta nas
    /// coordenadas em que o dab escreve.
    ///
    /// ⚠️ **A comparação é em MUNDO, e é a metade que um `t` não dá.** Cada `t`
    /// mede na régua LOCAL do seu objeto (o [`Ray`] normaliza a direção na
    /// construção), então uma peça a metade do tamanho devolveria um `t` duas
    /// vezes maior para o mesmo lugar — e a peça de trás ganharia o clique.
    /// **MIRAR** — escolhe a peça que este gesto vai trabalhar. `false` se o
    /// raio não achou nada (e aí o botão vira órbita, como em todo gesto).
    ///
    /// ⚠️ **Ela roda no PEN-DOWN, ANTES de o traço começar, e é a ÚNICA coisa
    /// que move o `active`.** Um traço pertence a UMA peça, e isso não é
    /// preferência: o `SculptStroke` dimensiona os planos por-vértice na malha
    /// em que ele COMEÇOU, então trocar de objeto no meio escreve índices de uma
    /// malha noutra — e com a peça nova maior que a velha isso é um **pânico**,
    /// não um desenho errado. É a mesma lei do NÍVEL, um degrau acima.
    pub(super) fn aim(&mut self, x: f32, y: f32) -> bool {
        match self.pick(x, y) {
            Some((object, _)) => {
                self.active = object;
                true
            }
            None => false,
        }
    }

    /// Onde o raio bate **na peça ATIVA** — a pergunta de todo dab.
    ///
    /// ⚠️ Irmã do [`Self::pick`] e não a mesma: aquela responde *que peça o
    /// artista mirou* e roda uma vez por gesto; esta responde *onde na peça que
    /// ele já escolheu*, e roda a cada dab. Usar a primeira aqui faria o pincel
    /// PULAR para outra peça no meio de uma pincelada — que é o defeito acima,
    /// pela outra ponta.
    ///
    /// ⚠️ **Ela recusa uma peça ativa ESCONDIDA**, e o caso é alcançável: um
    /// Ctrl+Z leva o ativo à peça da entrada, que pode não ser a isolada. Sem a
    /// recusa o artista esculpiria uma peça que ele **não vê** — e o gesto que a
    /// desfaz é o mesmo que a criou, então nada na tela diria por quê. Não fica
    /// preso: clicar na peça à vista devolve o ativo a ela.
    pub(super) fn pick_active(&self, x: f32, y: f32) -> Option<Hit> {
        let o = self.obj()?;
        if self.isolated_index().is_some_and(|k| k != self.active) {
            return None;
        }
        o.stack
            .mesh()
            .raycast(&o.pose.ray_to_local(&self.ray_at(x, y)))
    }

    /// ⭐⭐⭐ **O PEN-DOWN FOTOGRAFA A SUPERFÍCIE** que este traço vai picar —
    /// a porta que o [`Self::pick_do_dab`] consome.
    ///
    /// ⚠️ **Porta e não três linhas dentro do `input_down`, e a razão é
    /// GATEÁVEL:** o pen-down do produto precisa de um `AppHost` e não é
    /// alcançável de um teste, logo a decisão escrita lá dentro só podia ser
    /// afirmada por um censo textual. Aqui ela é exercitada pelo mesmo caminho
    /// que o artista toma, e o que fica para o censo é só *«o pen-down chama-a»*.
    ///
    /// ⚠️ **Escreve SEMPRE**, `Some` ou `None`: uma fotografia que sobrevivesse
    /// ao traço que a tirou faria o traço seguinte picar contra uma peça que já
    /// não existe.
    pub(crate) fn fotografa_a_superficie_do_pen_down(&mut self) {
        // ⚠️⚠️ **DUAS perguntas armam a MESMA fotografia, e elas não são a
        // mesma:** uma decide de que superfície sai o **CENTRO** do dab (o
        // projectar), a outra contra que superfície se mede **QUANDO** um dab
        // sai (o pincel afiado, `passo_no_mundo`). Quem lê o centro continua a
        // perguntar ao VERBO logo abaixo — se perguntasse à PRESENÇA da
        // fotografia, armá-la para medir o passo trocaria, em silêncio, a lei
        // do centro do outro pincel.
        let arma = self.brush.verb.pica_na_superficie_do_pen_down()
            || self.brush.verb.mede_o_passo_no_mundo();
        self.superficie_do_pen_down =
            arma.then(|| Box::new(self.objects[self.active].stack.mesh().clone()));
    }

    /// ⭐⭐⭐ **ONDE ESTE DAB ATERRA** — irmã do [`Self::pick_active`], e a
    /// diferença é **contra QUE superfície** o raio é lançado.
    ///
    /// Com uma fotografia de pen-down armada
    /// ([`Self::superficie_do_pen_down`]), o raio vai contra ela; sem
    /// fotografia, contra a malha viva, que é o caminho de sempre e é
    /// **byte-idêntico** — a chamada delega.
    ///
    /// ⚠️ **Porta separada, e não um `if` dentro do [`Self::pick_active`]:** as
    /// outras quatro consultas daquela porta perguntam *onde está a superfície
    /// AGORA* — a alça do puxão, o filtro, o gizmo. Só o dab pergunta *onde o
    /// artista mandou*, e misturar as duas faria uma fotografia armada por um
    /// pincel mudar, em silêncio, a resposta dada a quatro consumidores que não
    /// a pediram.
    ///
    /// ⚠️ **O acerto vem em coordenadas LOCAIS da peça activa nos dois casos**
    /// — a fotografia é da mesma peça, na mesma pose —, logo quem o recebe não
    /// tem de saber qual dos dois caminhos correu.
    pub(super) fn pick_do_dab(&self, x: f32, y: f32) -> Option<Hit> {
        // ⚠️ **Pergunta ao VERBO e não à PRESENÇA da fotografia** (2026-09-16):
        // desde que o pincel afiado a arma para medir o PASSO, a fotografia
        // existir deixou de querer dizer que o centro sai dela.
        if self.brush.verb.o_dab_segue_o_barro() {
            // ⭐⭐⭐ **O MESMO PONTO DE BARRO** — o raio pica a superfície
            // congelada e o acerto é LEVADO pela deformação até onde ele está
            // agora (ver [`ph2d_sculpt3d::levado_pela_deformacao`], onde a
            // medição vive). ⛔ Sem fotografia — o primeiro dab do traço — isto
            // é o `pick_active` de sempre, e tem de ser: ali as duas superfícies
            // são a mesma.
            let Some(congelada) = self.superficie_do_pen_down.as_deref() else {
                return self.pick_active(x, y);
            };
            let o = self.obj()?;
            if self.isolated_index().is_some_and(|k| k != self.active) {
                return None;
            }
            let mut hit = congelada.raycast(&o.pose.ray_to_local(&self.ray_at(x, y)))?;
            hit.point = ph2d_sculpt3d::levado_pela_deformacao(congelada, o.stack.mesh(), &hit)
                .unwrap_or(hit.point);
            return Some(hit);
        }
        if !self.brush.verb.pica_na_superficie_do_pen_down() {
            return self.pick_active(x, y);
        }
        let Some(congelada) = self.superficie_do_pen_down.as_deref() else {
            return self.pick_active(x, y);
        };
        // ⚠️ **As DUAS recusas do [`Self::pick_active`] continuam a valer**, e
        // por isso são repetidas e não saltadas: sem peça não há espaço local
        // em que o acerto signifique alguma coisa, e uma peça activa ESCONDIDA
        // não pode ser esculpida por baixo do isolamento.
        let o = self.obj()?;
        if self.isolated_index().is_some_and(|k| k != self.active) {
            return None;
        }
        congelada.raycast(&o.pose.ray_to_local(&self.ray_at(x, y)))
    }

    /// ⭐ **ONDE ESTE PONTO DO CAMINHO CAI NA SUPERFÍCIE CONGELADA** — a porta
    /// que a lei do passo no mundo consome ([`ph2d_sculpt3d::CaminhoNoMundo`]).
    ///
    /// ⚠️ **Devolve `None` sem fotografia**, e quem chama cai no passo de ecrã:
    /// uma lei de mundo sem a superfície contra que a medir não existe, e
    /// inventar-lhe um recuo silencioso seria dar-lhe outro comportamento.
    pub(crate) fn pick_congelado(&self, x: f32, y: f32) -> Option<Hit> {
        let congelada = self.superficie_do_pen_down.as_deref()?;
        let o = self.obj()?;
        if self.isolated_index().is_some_and(|k| k != self.active) {
            return None;
        }
        congelada.raycast(&o.pose.ray_to_local(&self.ray_at(x, y)))
    }

    pub(super) fn pick(&self, x: f32, y: f32) -> Option<(usize, Hit)> {
        let world = self.ray_at(x, y);
        let eye = world.origin();
        let mut best: Option<(usize, Hit, f32)> = None;
        // ⚠️ **Só o que está À VISTA**: um raio que alcança o escondido faria o
        // pen-down mudar o ativo para uma peça que o artista não vê, e o traço
        // seguinte pousaria nela.
        for i in self.visible_pieces() {
            let o = &self.objects[i];
            let Some(hit) = o.stack.mesh().raycast(&o.pose.ray_to_local(&world)) else {
                continue;
            };
            let w = o.pose.point_to_world(hit.point);
            let d = [w[0] - eye[0], w[1] - eye[1], w[2] - eye[2]];
            let d2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
            if best.is_none_or(|(_, _, b)| d2 < b) {
                best = Some((i, hit, d2));
            }
        }
        best.map(|(i, hit, _)| (i, hit))
    }

    /// Uma DIREÇÃO de mundo, no espaço da malha ativa — unitária.
    ///
    /// ⚠️ **Hoje ela devolve a entrada, e existe mesmo assim.** Sem rotação a
    /// [`Pose`] só divide pelo fator uniforme, e normalizar o desfaz — então
    /// passar a direção de mundo crua daria o mesmo número. Ela é a porta onde a
    /// rotação pousa: um call site que dispensasse a conversão continuaria
    /// compilando e começaria a mentir no dia em que um objeto girar.
    pub(super) fn dir_to_local(&self, d: [f32; 3]) -> [f32; 3] {
        Self::dir_to_local_of(self.pose(), d)
    }

    /// A mesma conversão, para uma pose DADA.
    ///
    /// ⚠️ Ela existe porque o preview percorre TODAS as peças à vista, e cada uma
    /// tem a sua pose — perguntar pela pose ATIVA ali daria a mesma base para
    /// todas, e o estêncil sairia torto em qualquer peça que estivesse girada em
    /// relação à que está selecionada.
    pub(super) fn dir_to_local_of(pose: Pose, d: [f32; 3]) -> [f32; 3] {
        let v = pose.vector_to_local(d);
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if len > 0.0 {
            [v[0] / len, v[1] / len, v[2] / len]
        } else {
            d
        }
    }

    /// **O EIXO de um giro no PLANO DA TELA em torno de `about_world`** — em
    /// coordenadas locais, unitário, apontando para o OLHO.
    ///
    /// ⚠️ **Ele aponta para o olho, e o sinal é a ferramenta inteira.**
    /// Rodrigues gira no anti-horário visto da PONTA do eixo, então um eixo
    /// virado para DENTRO da tela faz a peça girar ao contrário do dedo — que é
    /// literalmente o que o smoke reportou (*"a direção da rotação do mouse está
    /// invertida em relação à rot do objeto"*). É o mesmo sinal que o kernel do
    /// Twist nega no `stroke_target`, com o mesmo parágrafo ao lado: lá o
    /// [`ph2d_sculpt3d::Dab::eye`] aponta para dentro e a negação mora no
    /// consumidor; aqui a subtração já sai virada para fora, e o consumidor
    /// ([`ph2d_sculpt3d::Gesture::Rotate`]) recebe o eixo pronto.
    ///
    /// ⚠️ **E ele passa PELO OLHO — não basta ser paralelo à vista.** Uma reta
    /// que passa pelo olho projeta num PONTO (o próprio `about_world`
    /// projetado), então girar em torno dela deixa o pivô parado na tela e a
    /// silhueta a rodar em volta dele. O raio do pixel de PEN-DOWN é outra reta:
    /// medido em `transform_tests`, ela inclina **3,2° a 50 px** do
    /// pivô e **19,2° a 340 px** — e o que o artista vê nessa inclinação é a
    /// peça **cambalhotando** para fora do plano em vez de girar.
    pub(super) fn view_axis_local(&self, about_world: [f32; 3]) -> [f32; 3] {
        let e = self.camera.eye();
        self.dir_to_local([
            e.x - about_world[0],
            e.y - about_world[1],
            e.z - about_world[2],
        ])
    }

    /// O raio que passa pelo pixel — **a porta de projeção do gesto**.
    ///
    /// ⚠️ Ela mudou-se para cá quando o pai cruzou o teto de LOC, e o corte não
    /// foi por tamanho: são CINCO chamadores em quatro módulos (o carimbo, os
    /// três grips, o transform), e este arquivo é o que já responde *onde as
    /// coisas estão* — a pose, a direção local, o raio em pixels.
    pub(super) fn ray_at(&self, x: f32, y: f32) -> Ray {
        // ⚠️⚠️ **O ponto chega em coordenadas de JANELA e a câmera quer as da
        // VISTA** (2026-09-08). Com uma vista só e ela a cobrir o ecrã, as duas
        // coincidiam; com a área do canvas — e ainda mais com quatro quadrantes
        // — a diferença é a quina do quadrante, e o sintoma é exactamente
        // *«o lugar onde o mouse toca não corresponde ao local na malha»*.
        let (vx, vy) = self.to_view(x, y);
        self.camera.ray_through(vx, vy, self.viewport())
    }
}

/// **O FRUSTUM desta câmera, no espaço desta peça** — a função PURA por trás de
/// [`Sculpt3dScene::stencil_for`].
///
/// ⚠️ **Ela é livre e não um método, e o motivo é gate:** montar uma
/// [`Sculpt3dScene`] exige um `wgpu::Device`, então nenhum teste de CPU alcança o
/// método — e a única coisa que este cálculo precisa é uma câmera, um viewport e
/// uma pose. É a mesma cirurgia que a `line/anim` fez no overlay do motion path
/// pela mesma razão.
///
/// ⚠️ **A razão do frustum NÃO é dividida pela escala da peça, e não é
/// esquecimento:** ela é adimensional (mundo por mundo), então vale igual em
/// qualquer espaço. A pose entra **uma vez só**, no olho — e um segundo lugar
/// dividindo por ela encolheria o carimbo pelo quadrado da escala.
pub(super) fn stencil_of(
    cam: &Camera3d,
    viewport: (u32, u32),
    pose: Pose,
) -> ph2d_sculpt3d::AlphaStencil {
    let (right, up) = cam.screen_basis();
    ph2d_sculpt3d::AlphaStencil {
        right: Sculpt3dScene::dir_to_local_of(pose, right.into()),
        up: Sculpt3dScene::dir_to_local_of(pose, up.into()),
        eye: pose.point_to_local(cam.eye().into()),
        // O piso existe porque um viewport degenerado (altura 0) devolveria zero,
        // e o motor divide por esta razão.
        height_per_depth: cam.view_height_per_depth(viewport).max(1e-6),
    }
}

#[cfg(test)]
#[path = "space_tests.rs"]
mod tests;

/// O tecto do raio na cena, com GPU — ver [`raio_tests`].
#[cfg(test)]
#[path = "space_raio_tests.rs"]
mod raio_tests;
