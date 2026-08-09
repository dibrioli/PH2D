//! **OS VERBOS DA LISTA** — acrescentar, duplicar e apagar uma peça.
//!
//! Filho (`#[path]`) de [`super`] para alcançar os campos privados; o corte é
//! entre *o que se faz com UMA peça* (esculpir, mascarar, subdividir — no pai) e
//! *o que se faz com a LISTA delas* (aqui). São assuntos diferentes e o segundo
//! nasceu com a W8.1: até ela existir, "a cena" e "a malha" eram a mesma coisa.
//!
//! ⚠️ **Os três gravam undo, e é o que os separa de um comando de depuração.**
//! Uma peça que o artista acrescenta por engano tem de sumir com um Ctrl+Z, e
//! uma que ele apaga tem de VOLTAR — inteira, com a pilha de níveis e a máscara
//! dela. É por isso que o delete carrega a peça em vez de limpar a fila: limpar
//! seria trocar um trabalho perdido por outro.

use ph2d_mesh::{Mesh, Pose, shapes};

use super::{Multires, Sculpt3dScene, StrokeUndo};

/// O que o gesto de FUNDIR fez — ou por que ele não fez.
///
/// ⚠️ **Um enum e não `Option`, porque há DUAS recusas e elas pedem conselhos
/// opostos:** *não há o que fundir* manda o artista pôr mais peças na mesa,
/// *a pilha está montada* manda revertê-la primeiro. Colapsá-las num `None`
/// obrigaria o log a escolher uma frase e a estar errado em metade das vezes —
/// que é como um botão passa a parecer quebrado.
pub(crate) enum Merge {
    /// Fundiu: quantas peças entraram, e o tamanho do que saiu.
    Done {
        pieces: usize,
        verts: usize,
        faces: usize,
    },
    /// Menos de duas peças À VISTA: não há o que fundir.
    Nothing,
    /// Alguma peça tem a pilha de níveis montada.
    Stack,
}

/// O que o gesto de EXTRAIR fez — ou por que ele não fez.
///
/// ⚠️ **Duas recusas, e elas pedem conselhos opostos** (a mesma razão do
/// [`Merge`] acima): *não há peça* manda pôr uma na mesa, *não há máscara* manda
/// pintar uma. Um `None` obrigaria o log a escolher uma frase e a estar errado
/// em metade das vezes.
pub(crate) enum Extracted {
    /// Saiu uma peça: o tamanho dela.
    Done { verts: usize, faces: usize },
    /// A peça ativa não tem máscara pintada.
    NoMask,
    /// A cena está vazia.
    Nothing,
}

/// A identidade **DURÁVEL** de um objeto.
///
/// ⚠️ **Um ÍNDICE não serve para a fila de undo**, e o mecanismo é conhecido:
/// apagar a peça 1 de três faz a antiga 2 virar 1, e toda entrada que apontava
/// para 2 passa a nomear outra peça — em silêncio, com os índices ainda
/// válidos. É a mesma lição que a timeline pagou no `wire_id` e a física no
/// `stable_name_id`: **posição é endereço de alocação, não identidade**.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct ObjectId(pub(super) u32);

pub(crate) struct SceneObject {
    /// Quem esta peça É — ver [`ObjectId`].
    pub(crate) id: ObjectId,
    /// **A PILHA de níveis**, e não uma malha — ver [`Multires`]. O nível 0 é a
    /// base; o artista esculpe no que estiver selecionado.
    pub(crate) stack: Multires,
    /// Onde este objeto está no mundo — ver [`Pose`].
    pub(crate) pose: Pose,
    /// A malha já subiu inteira ao device? Depois disso só sobem REGIÕES.
    ///
    /// ⚠️ `pub(super)` — o MESMO alcance que ela tinha antes do split (o módulo
    /// `sculpt3d` e os irmãos dele), e não um byte a mais: quem escreve nestes
    /// dois campos é a fiação da GPU, nunca um chamador de fora.
    pub(super) uploaded: bool,
    /// Os vértices que a GPU ainda não viu — acumulados entre frames, porque
    /// vários eventos de ponteiro cabem num quadro.
    pub(super) dirty: Vec<u32>,
    /// O PREVIEW do padrão do pincel nesta peça — ver
    /// [`super::sculpt3d_preview::PreviewState`]. ⚠️ **Por objeto e não da
    /// cena**, pelo mesmo motivo que `uploaded`/`dirty`: o padrão é lido na
    /// POSIÇÃO do vértice, então cada peça tem o seu — e um par compartilhado
    /// deixaria a segunda peça tingida pelo campo da primeira.
    pub(super) preview: super::sculpt3d_preview::PreviewState,
}

impl SceneObject {
    pub(super) fn new(id: ObjectId, mesh: Mesh, pose: Pose) -> Self {
        Self {
            id,
            stack: Multires::new(mesh),
            pose,
            uploaded: false,
            dirty: Vec::new(),
            preview: super::sculpt3d_preview::PreviewState::default(),
        }
    }

    /// Uma peça vinda de um DOCUMENTO: a pilha inteira, e **nada no device**.
    ///
    /// ⚠️ Mora ao lado do [`SceneObject::new`] de propósito: os dois são *como
    /// uma peça nasce*, e a diferença entre eles é só de onde vem a pilha. Um
    /// construtor no módulo do documento teria de abrir `uploaded`/`dirty` para
    /// fora — e são justamente os dois campos que ninguém de fora deve escrever.
    pub(super) fn from_stack(id: ObjectId, stack: Multires, pose: Pose) -> Self {
        Self {
            id,
            stack,
            pose,
            uploaded: false,
            dirty: Vec::new(),
            preview: super::sculpt3d_preview::PreviewState::default(),
        }
    }
}

/// Quantas fatias uma primitiva nasce tendo.
///
/// ⚠️ **Número de SMOKE, não teto de recurso.** Ele decide a densidade INICIAL,
/// e o `K` (subdividir) é quem a leva adiante — uma primitiva que nascesse densa
/// tiraria do artista a escolha de onde gastar vértices, que é metade do que a
/// multiresolução existe para dar.
const SEGMENTS: usize = 24;

/// As quatro formas com que se blóca. ⚠️ **Quatro e não uma lista aberta:** cada
/// uma responde a uma silhueta que as outras não dão de graça (uma bola, uma
/// caixa, um tubo, um anel), e a quinta entra quando alguém disser qual
/// silhueta ela traz.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Primitive {
    Sphere,
    Cube,
    Cylinder,
    Torus,
}

impl Primitive {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Sphere => "esfera",
            Self::Cube => "cubo",
            Self::Cylinder => "cilindro",
            Self::Torus => "toro",
        }
    }

    /// A malha, em coordenadas LOCAIS de raio ~1 — a pose é quem a dimensiona.
    ///
    /// ⚠️ **Todas cabem na mesma esfera unitária**, e é isso que faz uma pose só
    /// servir para as quatro: sem essa normalização a escala da pose teria de
    /// depender da forma, e o artista veria o cubo nascer de outro tamanho que a
    /// esfera pelo mesmo gesto.
    /// ⚠️ `pub(crate)` porque o pill SCULPT ENTRA criando uma peça (`sculpt3d_mode`), e ela tem de
    /// ser a MESMA que o verbo de acrescentar cria — duas malhas iniciais seriam duas respostas a
    /// *com que forma uma escultura começa*.
    pub(crate) fn mesh(self) -> Mesh {
        match self {
            Self::Sphere => shapes::uv_sphere(SEGMENTS / 2, SEGMENTS, 1.0),
            // `size` é a ARESTA, então a diagonal de um cubo de aresta 2/√3
            // mede 2 — a mesma esfera envolvente da bola de raio 1.
            Self::Cube => shapes::cube(2.0 / 3.0_f32.sqrt()),
            Self::Cylinder => shapes::cylinder(SEGMENTS, 0.7, 2.0),
            Self::Torus => shapes::torus(SEGMENTS, SEGMENTS / 2, 0.7, 0.3),
        }
    }
}

impl Sculpt3dScene {
    /// **ACRESCENTA uma peça** onde o artista está olhando, e a torna ativa.
    ///
    /// ⚠️ **No ALVO da câmera, e não na origem.** Não há cursor 3D neste módulo,
    /// e o alvo é o único ponto que o artista de fato escolheu — ele é o centro
    /// do que está na tela. Nascer na origem poria a peça fora de quadro assim
    /// que ele desse um pan, que é a forma de *"o botão não fez nada"*.
    ///
    /// ⚠️ **E o TAMANHO sai da cena, não de uma constante.** Um quarto do maior
    /// lado do que já existe: a peça nova é sempre visível e sempre comparável
    /// com a que está ao lado, em qualquer escala de trabalho. Um número fixo
    /// seria invisível numa cena grande e dominante numa pequena.
    ///
    /// ⚠️ **Ela vira a ATIVA, e isso contradiz o `push_object` de propósito:**
    /// lá a regra é *montar uma cena é pôr peças na mesa, não pegar cada uma*,
    /// porque quem monta é uma fixture. Aqui quem pede é o artista, e ele acabou
    /// de dizer qual peça quer. A frase que autoriza a diferença é a mesma do
    /// dropdown de meio do Painter: **escolher uma coisa USA ela**.
    pub(super) fn add_primitive(&mut self, kind: Primitive) -> usize {
        let span = self.world_bounds().longest_edge();
        let scale = if span > 0.0 { span * 0.25 } else { 1.0 };
        let at = self.camera.target;
        let id = self.mint_id();
        self.objects.push(SceneObject::new(
            id,
            kind.mesh(),
            Pose::new([at.x, at.y, at.z], scale),
        ));
        self.active = self.objects.len() - 1;
        self.record(StrokeUndo::AddedObject);
        self.mesh_rebuilt();
        self.active
    }

    /// **RECORTA a região mascarada numa peça NOVA** — ver
    /// [`ph2d_mesh::extract_masked`].
    ///
    /// ⚠️ **A peça nasce com a POSE da origem**, e não deslocada como a do
    /// `duplicate_active`: a cópia precisa saltar para o lado porque é
    /// indistinguível do original, e esta precisa do contrário — ela é uma casca
    /// que só faz sentido **onde a máscara foi pintada**. Movê-la seria desfazer
    /// o gesto no instante em que ele acontece.
    ///
    /// ⚠️ **Ela NÃO recusa com a pilha de multires montada**, e a diferença para
    /// o remesh, o tapar buraco e a fusão é o que a autoriza: aqueles trocam a
    /// BASE, e todo nível acima dela é uma subdivisão que passaria a descrever
    /// uma malha que não existe mais. Extrair não toca a origem — ele lê o nível
    /// vivo e escreve noutro objeto.
    pub(super) fn extract_masked(&mut self, opts: ph2d_mesh::Extract) -> Extracted {
        let Some(src) = self.obj() else {
            return Extracted::Nothing;
        };
        let pose = src.pose;
        let Some(mesh) = ph2d_mesh::extract_masked(src.stack.mesh(), opts) else {
            return Extracted::NoMask;
        };
        let (verts, faces) = (mesh.vert_count(), mesh.face_count());
        let id = self.mint_id();
        self.objects.push(SceneObject::new(id, mesh, pose));
        // A peça recém-cortada vira a ativa: o artista acabou de dizer que é
        // nela que quer trabalhar — a mesma frase do `add_primitive`.
        self.active = self.objects.len() - 1;
        self.record(StrokeUndo::AddedObject);
        self.mesh_rebuilt();
        Extracted::Done { verts, faces }
    }

    /// **DUPLICA a peça ativa**, deslocada para o lado NA TELA.
    ///
    /// ⚠️ **Para o lado na TELA, e não num eixo de mundo**, porque "ao lado" é
    /// uma palavra sobre o que o artista vê: um deslocamento em `+X` de mundo
    /// esconde a cópia ATRÁS da original sempre que a câmera estiver olhando por
    /// ali, e o gesto lê como *"não duplicou"*. A conversão é a mesma porta que
    /// o arrasto de barro usa (`screen_delta_to_world`), então as duas
    /// respondem a *"que direção é 'para a direita'?"* com o mesmo número.
    pub(super) fn duplicate_active(&mut self) -> bool {
        // Sem peça não há o que duplicar.
        let Some(src) = self.obj() else {
            return false;
        };
        let mesh = src.stack.mesh().clone();
        let pose = src.pose;
        let world = pose.bounds_to_world(src.stack.mesh().bounds());
        let width = (world.max[0] - world.min[0])
            .max(world.max[1] - world.min[1])
            .max(world.max[2] - world.min[2])
            .max(1e-3);

        let at = pose.translation;
        let step = self
            .camera
            .screen_delta_to_world(at, 1.0, 0.0, self.viewport);
        let len = (step[0] * step[0] + step[1] * step[1] + step[2] * step[2]).sqrt();
        // Câmera degenerada (viewport zero) não tem "para a direita": a cópia
        // nasce em cima da original em vez de saltar para o infinito.
        let right = if len > 0.0 {
            [step[0] / len, step[1] / len, step[2] / len]
        } else {
            [0.0; 3]
        };
        // ⚠️ **1,15 larguras, e não 1,0:** encostadas exatamente, as duas peças
        // partilham a silhueta e o artista não vê que são duas. A folga é o que
        // torna o gesto legível.
        let gap = width * 1.15;
        let id = self.mint_id();
        self.objects.push(SceneObject::new(
            id,
            mesh,
            Pose::new(
                [
                    at[0] + right[0] * gap,
                    at[1] + right[1] * gap,
                    at[2] + right[2] * gap,
                ],
                pose.scale(),
            ),
        ));
        self.active = self.objects.len() - 1;
        self.record(StrokeUndo::AddedObject);
        self.mesh_rebuilt();
        true
    }

    /// **APAGA a peça ativa.** Devolve `false` só quando não há nenhuma.
    ///
    /// ⚠️ **A última ERA inapagável, e o Enio derrubou a cerca no smoke** (*"não
    /// consigo deletar todos os objetos da tela"*). A recusa era honesta sobre o
    /// mecanismo — a lista nunca-vazia é o que tornava o `obj()` total — mas ela
    /// defendia uma invariante nossa, não um interesse do artista: *esvaziar a
    /// cena* é um gesto legítimo, e o verbo que a nota antiga oferecia
    /// (*substituir*: acrescente a nova e apague a velha) só serve a quem quer
    /// outra peça, não a quem quer nenhuma.
    ///
    /// ⚠️ **O preço foi 54 sítios**, e a cura é a representação admitindo o que
    /// já era verdade: `obj()` devolve `Option`, porque *"qual é a peça ativa?"*
    /// honestamente pode não ter resposta. O que sobrou de `expect` é local e
    /// curto — um guard três linhas acima, na mesma função.
    ///
    /// ⚠️ **E desfazer CONTINUA trazendo a peça de volta**, inclusive a última:
    /// o `RemovedObject` é um dos dois verbos que o `apply_entry` deixa rodar
    /// numa cena VAZIA, e é isso que faz *"apaguei tudo"* ser reversível.
    ///
    /// ⚠️ **A peça sai INTEIRA para dentro da fila** — com a pilha de níveis, a
    /// máscara e a pose. Limpar a fila em vez disso (o que a W8.1 anotou como
    /// saída) trocaria um trabalho perdido por outro: o artista recuperaria a
    /// peça e perderia a história de todas as outras.
    /// **FUNDE as peças À VISTA numa só.** O passo que falta entre blocar com
    /// primitivas e esculpir uma forma.
    ///
    /// ⚠️ **Ela funde o que se VÊ, e é isso que a torna previsível.** Não há
    /// seleção múltipla neste módulo — o "ativo" é *a última peça que você
    /// tocou* —, então *fundir as selecionadas* nomearia um conjunto que o
    /// artista não montou. O conjunto que ele montou é o que está na tela, e com
    /// uma peça isolada a resposta honesta é [`Merge::Nothing`]: ele vê uma
    /// peça, e uma peça não se funde com nada.
    ///
    /// ⚠️ **A recusa com a pilha montada é a MESMA do remesh e do tapar
    /// buraco**, pelo mesmo mecanismo: todo nível acima da base é `subdivide`
    /// dela, e a fusão troca a base por outra malha — o detalhe de cima passaria
    /// a descrever uma geometria que não existe mais. Achatar a pilha em
    /// silêncio seria destruir trabalho autorado sem dizer.
    ///
    /// ⚠️ **Ela não SOLDA nada** (ver [`ph2d_mesh::merge`]): duas superfícies que
    /// se tocam continuam duas dentro da mesma malha. Quem as transforma numa
    /// casca é o remesh (`V`), e é exatamente isto que o torna possível — ele
    /// opera numa malha.
    pub(super) fn merge_visible(&mut self) -> Merge {
        let idx: Vec<usize> = self.visible_pieces().collect();
        if idx.len() < 2 {
            return Merge::Nothing;
        }
        if idx
            .iter()
            .any(|&i| self.objects[i].stack.level_count() != 1)
        {
            return Merge::Stack;
        }
        let id = self.mint_id();
        let Some(gone) = self.fuse_visible(id) else {
            return Merge::Nothing;
        };
        let pieces = gone.len();
        self.record_for(id, StrokeUndo::Merged(gone));
        let (verts, faces) = (self.mesh().vert_count(), self.mesh().face_count());
        Merge::Done {
            pieces,
            verts,
            faces,
        }
    }

    /// **FUNDE O QUE ESTÁ À VISTA, com a identidade `id`** — a porta ÚNICA das
    /// duas rotas: o gesto do artista e o REFAZER.
    ///
    /// ⚠️ **Ela existe porque as duas fazem a mesma pergunta**, e uma segunda
    /// cópia dela divergiria no caso que importa: se o refazer fundisse *todas
    /// as peças* enquanto o gesto funde *as visíveis*, um Ctrl+Shift+Z sobre uma
    /// cena isolada produziria uma peça que o gesto original nunca produziria.
    ///
    /// ⚠️ **E o `id` é PARÂMETRO por causa do refazer:** re-fundir tem de
    /// reconstruir a peça com o MESMO [`super::ObjectId`], senão a entrada que a
    /// desfaz nomeia uma peça que não existe e o Ctrl+Z seguinte não acha o que
    /// remover.
    pub(super) fn fuse_visible(&mut self, id: super::ObjectId) -> Option<Vec<SceneObject>> {
        let idx: Vec<usize> = self.visible_pieces().collect();
        self.fuse(&idx, id)
    }

    /// **A FUSÃO propriamente dita** — troca as peças `idx` por uma só, com a
    /// identidade `id`, e devolve as que saíram.
    fn fuse(&mut self, idx: &[usize], id: super::ObjectId) -> Option<Vec<SceneObject>> {
        let parts: Vec<(&Mesh, Pose)> = idx
            .iter()
            .map(|&i| (self.objects[i].stack.mesh(), self.objects[i].pose))
            .collect();
        let mesh = ph2d_mesh::merge(&parts).ok()?;

        // ⚠️ **De trás para frente**, e não é estilo: remover o índice 0 desloca
        // todos os outros, então uma varredura crescente apagaria a peça errada
        // a partir da segunda remoção. É a mesma lei que a tira do Flip pagou no
        // arrasto de seleção — *a ORDEM de emissão é o que garante que cada
        // remoção pousa onde o chamador pensou*.
        let mut gone = Vec::with_capacity(idx.len());
        for &i in idx.iter().rev() {
            gone.push(self.objects.remove(i));
        }
        gone.reverse();

        // ⚠️ **A peça nasce em [`Pose::IDENTITY`]**: a pose de cada fonte já
        // está assada nos vértices, e uma pose herdada de alguma delas moveria a
        // fusão inteira no instante em que ela aparece.
        self.objects
            .push(SceneObject::new(id, mesh, Pose::IDENTITY));
        self.active = self.objects.len() - 1;
        self.mesh_rebuilt();
        Some(gone)
    }

    /// **ISOLA a peça ativa — ou devolve a cena inteira à vista.** Devolve o
    /// estado NOVO (`true` = isolada).
    ///
    /// ⚠️ **É um toggle e não um modo com saída própria**, porque o gesto tem um
    /// inverso óbvio e um só: a mesma tecla. Um "sair do isolamento" separado
    /// seria uma segunda porta para o mesmo fato, e a que o artista não acha
    /// quando a cena some.
    ///
    /// ⚠️ **Nada aqui entra na história.** Isolar não move um vértice — ver o
    /// campo `isolated`.
    pub(super) fn toggle_isolate(&mut self) -> bool {
        if self.isolated_index().is_some() {
            self.isolated = None;
            return false;
        }
        // Sem peça ativa não há o que isolar, e isolar "nada" apagaria a cena da
        // tela sem nada para devolver.
        let Some(id) = self.obj().map(|o| o.id) else {
            return false;
        };
        self.isolated = Some(id);
        true
    }

    pub(super) fn delete_active(&mut self) -> bool {
        let Some(object) = self.obj().map(|o| o.id) else {
            return false;
        };
        let gone = self.objects.remove(self.active);
        self.active = self.active.min(self.objects.len().saturating_sub(1));
        // ⚠️ A entrada é gravada com o id da peça que SAIU, não com o da que
        // ficou ativa — é ela que o `RemovedObject` vai recolocar, e o `record`
        // carimba o ativo. Por isso a construção é explícita aqui.
        self.record_for(object, StrokeUndo::RemovedObject(Box::new(gone)));
        self.mesh_rebuilt();
        true
    }
}
