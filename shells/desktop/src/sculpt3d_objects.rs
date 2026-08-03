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
}

impl SceneObject {
    pub(super) fn new(id: ObjectId, mesh: Mesh, pose: Pose) -> Self {
        Self {
            id,
            stack: Multires::new(mesh),
            pose,
            uploaded: false,
            dirty: Vec::new(),
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
    fn mesh(self) -> Mesh {
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
