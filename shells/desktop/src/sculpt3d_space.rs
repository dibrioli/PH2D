//! **ONDE as coisas estão** — as portas de espaço da cena.
//!
//! Filho (`#[path]`) de [`super`] para alcançar os campos privados; o corte é *o
//! que a cena É e o que a mão FAZ* (lá) contra *em que ESPAÇO cada número vive*
//! (aqui). É o assunto que a W8.1 abriu: com uma malha só, "onde" não era uma
//! pergunta — a geometria era o mundo. Com uma lista de objetos ela passa a ser,
//! e toda conversão entre os dois espaços mora neste arquivo, num lugar só.

use super::{
    Brush, Hit, Mesh, ObjectId, Pose, RADIUS_MAX_FRAC_OF_HEIGHT, RADIUS_MIN_PX, SceneObject,
    Sculpt3dScene,
};

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

    /// A caixa de **todos** os objetos, em mundo.
    pub(crate) fn world_bounds(&self) -> ph2d_mesh::Aabb {
        let mut b = ph2d_mesh::Aabb::EMPTY;
        for o in &self.objects {
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
        self.camera.frame(b, aspect);
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
    pub(crate) fn mesh(&self) -> &Mesh {
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
        let ceiling =
            (RADIUS_MAX_FRAC_OF_HEIGHT * self.viewport.1.max(1) as f32).max(RADIUS_MIN_PX);
        self.radius_px.clamp(RADIUS_MIN_PX, ceiling)
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
        let pose = self.pose();
        let world = pose.point_to_world(local_at);
        let radius = self
            .camera
            .world_radius_for_screen_px(world, self.radius_px(), self.viewport);
        Brush {
            radius: (radius / pose.scale()).max(1e-6),
            ..self.brush
        }
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
    pub(super) fn pick_active(&self, x: f32, y: f32) -> Option<Hit> {
        let o = self.obj()?;
        o.stack
            .mesh()
            .raycast(&o.pose.ray_to_local(&self.ray_at(x, y)))
    }

    pub(super) fn pick(&self, x: f32, y: f32) -> Option<(usize, Hit)> {
        let world = self.ray_at(x, y);
        let eye = world.origin();
        let mut best: Option<(usize, Hit, f32)> = None;
        for (i, o) in self.objects.iter().enumerate() {
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
        let v = self.pose().vector_to_local(d);
        let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if len > 0.0 {
            [v[0] / len, v[1] / len, v[2] / len]
        } else {
            d
        }
    }
}
