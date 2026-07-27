//! **A MÃO** (W-Grab) — segurar um corpo enquanto a simulação corre.
//!
//! Até aqui o Play era só de leitura. A pose de um corpo dinâmico é escrita
//! pelo `readback` a cada frame, então um arrasto de gizmo durante o Play
//! escreve o `Transform` e é sobrescrito no MESMO frame: a cena roda e o artista
//! não pode cutucá-la. Todo laboratório de física 2D deixa (Algodoo, o testbed
//! do Box2D, o RUBE, o play mode da Unity), e é assim que se testa uma cena —
//! empurrar, puxar, atirar.
//!
//! # A mão é uma MOLA, e é por isso que ela não trapaceia
//!
//! Segurar não é teleportar. [`PhysicsWorld::set_body_pose`] existe e seria a
//! resposta errada: ela zera a velocidade e atravessa parede (a pose é imposta,
//! não negociada). A mão entra pelo **solver** — uma mola macia entre o ponto
//! que a mão pegou e um corpo-âncora invisível que **é o cursor** — então:
//!
//! - o corpo **colide no caminho** (você não arrasta uma caixa por dentro de uma
//!   parede; ela raspa e emperra, como uma caixa faz);
//! - **soltar não zera nada**, então o corpo sai com a velocidade que tinha —
//!   *atirar* cai de graça, e é metade da razão de existir do gesto;
//! - a mola é resolvida **junto com** as outras restrições, logo é estável na
//!   rigidez que um arrasto precisa. Um controlador PD explícito (força por tick
//!   calculada aqui) explode no ganho equivalente a 1/60 s — foi por isso que
//!   isto é um joint e não um laço de força.
//!
//! # `AccelerationBased`: a lei do MouseJoint, vinda do rapier
//!
//! O `JointKind::Spring` do artista é `ForceBased` — uma mola FÍSICA, em que um
//! corpo pesado afunda. Correto para uma mola, errado para uma mão: o artista
//! não quer lutar contra a massa para reposicionar um caixote. O Box2D resolve
//! isso com `maxForce ∝ massa`; o rapier tem a mesma ideia embutida no
//! [`MotorModel::AccelerationBased`] (*"the spring constants will be
//! automatically scaled by the attached masses"*), então a mão a usa e a mola do
//! artista não. **Duas leis para duas coisas** — e há gate medindo as duas, uma
//! contra a outra, porque é a única forma de a escolha não virar folclore.
//!
//! # Determinismo: a mão é a PRIMEIRA entrada não-reproduzível deste módulo
//!
//! Params vivos mid-play são CONFIG (reconciliados do ECS, então um replay os
//! reproduz); um corpo kinematic é respondido pelo `SceneAtTick`. Uma mão não é
//! nem uma coisa nem outra: ela não está no documento. Logo o mundo deixa de ser
//! função de `(tick, repouso, curvas)` enquanto ela existe, e a consequência é
//! do chamador — quem possui o ring de checkpoints (`bridge::grab`) o **descarta
//! ao pegar** e **não grava enquanto a mão está lá**, e um rewind **solta**.
//!
//! ⚠️ **Corolário que o `checkpoint` depende:** nenhum checkpoint jamais contém
//! a tralha da mão (o corpo-âncora e o joint). Não é uma esperança — é o
//! invariante que a ponte garante com gate próprio, e sem ele um `restore`
//! devolveria uma âncora órfã com o `grab` desta struct apontando para handles
//! de outra arena.

use rapier2d::dynamics::{
    ImpulseJointHandle, MotorModel, RigidBodyBuilder, RigidBodyHandle, SpringJointBuilder,
};
use rapier2d::na::{Isometry2, Point2, Vector2};

use super::PhysicsWorld;

/// A mão em voo: o que ela segura, por onde, e a tralha que a realiza.
#[derive(Copy, Clone, Debug)]
pub struct Grab {
    /// O corpo segurado.
    body: RigidBodyHandle,
    /// O corpo-âncora **fixo e invisível** que é o cursor. Fixo (e não
    /// kinematic) porque nada deve movê-lo a não ser a mão, e porque a mola só
    /// lê a POSIÇÃO dele — a velocidade de uma âncora não entra na conta.
    anchor: RigidBodyHandle,
    /// A mola entre os dois.
    joint: ImpulseJointHandle,
    /// **Onde no corpo** a mão pegou, no frame LOCAL dele — a mesma política de
    /// âncora dos joints do artista (ADR-0131 / W-AnchorFollow): guardar o ponto
    /// de MUNDO faria o ponto de pega caminhar pelo corpo conforme ele gira.
    local: [f32; 2],
}

impl PhysicsWorld {
    /// Rigidez da mão, em unidades **aceleração-por-metro** (o modelo é
    /// `AccelerationBased`, então isto não é N/m).
    ///
    /// ⚠️ **O teto deste número não é gosto, é a PAREDE** — e é por isso que ele
    /// tem duas tabelas. A mola e o contato são resolvidos pelo MESMO solver com
    /// um número finito de iterações, então uma mão rígida o bastante **atravessa
    /// geometria**: medido (`world::tests::sweep_the_hands_stiffness_against_a_wall`,
    /// cursor 5 m para DENTRO de uma parede estática, meio segundo), com a face
    /// da parede em `x = 1,0` e a caixa de raio 0,5 encostada nela:
    ///
    /// | k | onde a caixa parou | penetração | atravessou? |
    /// |---|---|---|---|
    /// | **400** | 0,505 | **5 mm** | não |
    /// | 1600 | 0,516 | 16 mm | não |
    /// | 6400 | 0,562 | 62 mm | não |
    /// | 12800 | 0,622 | 122 mm | não |
    /// | 25600 | 6,000 | — | **SIM** (teleporta para o cursor) |
    /// | 102400 | 6,000 | — | **SIM** |
    ///
    /// A parede **para de segurar entre 12800 e 25600**, 32-64× acima do valor
    /// que shipa; e a penetração já cresce muito antes disso — 5 mm em 400 é da
    /// ordem da tolerância de repouso do solver (1,3 mm), 122 mm em 12800 é uma
    /// caixa meio dentro do muro.
    ///
    /// A segunda tabela é o atraso (`sweep_the_hands_gains`, cursor a 4 m/s por
    /// meio segundo, depois PARADO por outro meio):
    ///
    /// | k (d = 2√k) | atraso em regime | sobressinal |
    /// |---|---|---|
    /// | 100 | 0,751 m | 0,000 m |
    /// | 200 | 0,532 m | 0,000 m |
    /// | **400** | **0,369 m** | 0,000 m |
    /// | 800 | 0,252 m | 0,000 m |
    /// | 1600 | 0,169 m | 0,000 m |
    ///
    /// O atraso em regime de um seguidor criticamente amortecido é `2v/√k`, então
    /// cai como `1/√k`: **quadruplicar a rigidez compra metade do atraso** e
    /// multiplica a penetração por 12. E ele é proporcional à velocidade do
    /// cursor — os 0,369 m são de um arrasto RÁPIDO (4 m/s, uma varrida de tela);
    /// a 1 m/s são 9 cm. Um corpo que vem um pouco atrás da mão é o que uma mola
    /// faz, e é o que o testbed do Box2D e o Algodoo mostram.
    ///
    /// **Zero sobressinal em toda a faixa** porque o amortecimento é o crítico
    /// (ver [`Self::GRAB_DAMPING`]) — sobressinal numa mão é o corpo passando do
    /// cursor, o defeito que se vê.
    pub const GRAB_STIFFNESS: f32 = 400.0;

    /// Amortecimento da mão — o **crítico** para [`Self::GRAB_STIFFNESS`]
    /// (`2·√k`), que é o que faz o corpo chegar ao cursor e PARAR ali em vez de
    /// oscilar em torno dele.
    ///
    /// Não é um segundo knob a afinar: é uma função do primeiro, escrita como
    /// constante porque `2.0 * k.sqrt()` não é `const` em ponto flutuante.
    /// `sweep_the_hands_gains` varre o par e pina esta relação.
    pub const GRAB_DAMPING: f32 = 40.0;

    /// **Pegar `handle` pelo ponto de MUNDO `world_point`.** Devolve `false` —
    /// e não pega nada — quando não há o que segurar:
    ///
    /// - handle morto;
    /// - corpo **não-dinâmico**. Um joint não move um corpo estático nem um
    ///   kinematic (massa infinita — o fato medido que a W-BakeJoint documenta),
    ///   então oferecer a mão ali seria um gesto morto. A recusa é explícita
    ///   para que o chamador possa deixar o caminho de sempre acontecer.
    ///
    /// Uma mão só: pegar com uma mão ocupada **solta** a anterior. O alternativo
    /// (duas molas) é uma feature de dois cursores que ninguém pediu, e a
    /// silenciosa (recusar) deixaria o gesto seguinte inerte sem dizer por quê.
    pub fn grab_body(&mut self, handle: RigidBodyHandle, world_point: [f32; 2]) -> bool {
        self.grab_body_tuned(
            handle,
            world_point,
            Self::GRAB_STIFFNESS,
            Self::GRAB_DAMPING,
        )
    }

    /// [`Self::grab_body`] com os ganhos na mão em vez de tomados das
    /// constantes medidas.
    ///
    /// **Existe para que a tabela em [`Self::GRAB_STIFFNESS`] seja reprodutível
    /// contra o caminho do PRODUTO**, e não contra uma segunda cópia dele escrita
    /// num arquivo de teste — exatamente o mesmo motivo (e o mesmo formato) do
    /// `spawn_joint_tuned`, que existe para as tabelas do servo. A varredura mora
    /// em `world::tests`.
    pub(super) fn grab_body_tuned(
        &mut self,
        handle: RigidBodyHandle,
        world_point: [f32; 2],
        stiffness: f32,
        damping: f32,
    ) -> bool {
        self.release_grab();
        let Some(body) = self.bodies.get(handle) else {
            return false;
        };
        if !body.is_dynamic() {
            return false;
        }
        let pose = *body.position();
        let local = Self::local_anchor_at_pose(
            [
                pose.translation.x,
                pose.translation.y,
                pose.rotation.angle(),
            ],
            world_point,
        );
        // A âncora é TRALHA, não um corpo da cena: entra direto na arena, sem
        // `insert_body` — este estampa os defaults de mundo (damping, sono), que
        // não querem dizer nada para um corpo que nunca é integrado, e a âncora
        // não tem entidade, então nada a lê de volta nem a desenha.
        let anchor = self.bodies.insert(
            RigidBodyBuilder::fixed()
                .translation(Vector2::new(world_point[0], world_point[1]))
                .build(),
        );
        // `rest_length` ZERO: o alvo é *o ponto de pega EM CIMA do cursor*. Com
        // a separação exatamente nula o erro é nulo e a mola não empurra, então
        // não há direção indefinida a produzir NaN — o que o gate de um clique
        // sem arrasto mede (ele não pode mover o corpo nem um ulp).
        let joint = SpringJointBuilder::new(0.0, stiffness, damping)
            .spring_model(MotorModel::AccelerationBased)
            .local_anchor1(Point2::origin())
            .local_anchor2(Point2::new(local[0], local[1]))
            .build();
        // ⚠️ `wake_up = true` acorda um corpo adormecido — que são a maioria dos
        // que se quer cutucar. **Medido, é a camada de FORA: mutá-lo para `false`
        // não sangra**, porque o `wake_up` do [`Self::move_grab`] cobre todo caso
        // observável (a mão não tem o que fazer entre o press e o 1º movimento —
        // o erro da mola é exatamente zero ali). Fica por ser a convenção do
        // rapier para mudança estrutural, e porque a redundância é dita em vez de
        // suposta ([[feedback_layered_defenses_need_per_layer_gates]]).
        let joint = self.impulse_joints.insert(anchor, handle, joint, true);
        self.grab = Some(Grab {
            body: handle,
            anchor,
            joint,
            local,
        });
        true
    }

    /// **A mão andou.** Move a âncora — e só ela; a mola faz o resto.
    ///
    /// No-op sem mão, para que o chamador possa chamar por frame sem perguntar.
    pub fn move_grab(&mut self, world_point: [f32; 2]) {
        let Some(g) = self.grab else {
            return;
        };
        if let Some(a) = self.bodies.get_mut(g.anchor) {
            a.set_position(
                Isometry2::new(Vector2::new(world_point[0], world_point[1]), 0.0),
                true,
            );
        }
        // ⚠️ **E acorda o corpo — isto NÃO é higiene, é o que mantém a mão viva.**
        // Um corpo na mão ADORMECE se a mão ficar parada (medido: tick 119 de mão
        // imóvel), e um corpo dormindo não é integrado: sem esta linha, segurar
        // quieto por dois segundos e voltar a arrastar move o CURSOR e mais nada
        // — a assinatura exata de *"a ferramenta parou de funcionar"*. Mover a
        // âncora com `wake_up = true` NÃO basta (ela é fixa, e a mutação medida
        // deixa o corpo em `x = 0,000` com a mão a 3 m).
        //
        // ⚠️ E eu removi esta linha uma vez, por uma medição feita no build que
        // AINDA a continha: a sonda sampleava a cada 100 ticks e imprimia
        // "acordado" porque era ela que o acordava. Gate:
        // `a_body_that_slept_in_the_hand_follows_it_again`.
        if let Some(b) = self.bodies.get_mut(g.body) {
            b.wake_up(true);
        }
    }

    /// **Soltar.** Remove a mola e a âncora, e **não toca na velocidade do
    /// corpo** — é isso que faz de soltar em movimento um ARREMESSO.
    ///
    /// No-op sem mão (o caminho comum de todo release de botão).
    pub fn release_grab(&mut self) {
        let Some(g) = self.grab.take() else {
            return;
        };
        // A mola primeiro, a âncora depois: remover o corpo já levaria os joints
        // presos a ele, mas depender disso é depender de um detalhe da biblioteca
        // para uma limpeza que se escreve em uma linha.
        self.remove_joint(g.joint);
        self.remove_body(g.anchor);
    }

    /// O corpo na mão, se houver.
    pub fn grabbed_body(&self) -> Option<RigidBodyHandle> {
        self.grab.map(|g| g.body)
    }

    /// A mão para DESENHAR: `(onde o cursor está, onde o ponto de pega está
    /// AGORA)`, ambos em mundo.
    ///
    /// ⚠️ O segundo ponto é derivado do `local` contra a pose **VIVA** do corpo,
    /// nunca memorizado: ele tem de andar (e girar) com o corpo, senão a mola
    /// desenhada aponta para onde o corpo ESTAVA quando foi pego. É a mesma
    /// razão pela qual as âncoras dos joints são locais.
    pub fn grab_marks(&self) -> Option<([f32; 2], [f32; 2])> {
        let g = self.grab?;
        let anchor = self.bodies.get(g.anchor)?.position().translation;
        let pose = self.bodies.get(g.body)?.position();
        let hold = Self::world_from_local_at_pose(
            [
                pose.translation.x,
                pose.translation.y,
                pose.rotation.angle(),
            ],
            g.local,
        );
        Some(([anchor.x, anchor.y], hold))
    }
}
