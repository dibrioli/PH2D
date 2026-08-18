//! **O GESTO DO TRANSFORM** — a mão que move, gira e escala a parte LIVRE.
//!
//! Filho (`#[path]`) de [`super`] para alcançar os campos privados; o corte é o
//! dos vizinhos — a LEI mora no kernel ([`ph2d_sculpt3d::MaskTransform`]), e o
//! que mora aqui é *o que a mão na tela quer dizer*.
//!
//! # Por que não há gizmo
//!
//! A referência dirige isto por um gizmo 3D de treze alças. O gizmo deste repo é
//! o de **SPRITE** (ADR-0110/0111), 2D, e um gizmo 3D é wave própria e grande. O
//! substituto não é um compromisso: é a **outra** metade do desenho do Blender —
//! o *modal transform*, em que a ferramenta é armada e o arrasto a executa. Aqui
//! ele encaixa num `Drag` a mais, ao lado dos três que a cena já tem.
//!
//! ⚠️ **A porta de armar é o PAINEL, e a escolha foi por eliminação, não por
//! gosto:** o `sculpt3d_keys` declara, com a lista ao lado, que *"o que sobra
//! livre no app inteiro é a crase"* — os dez dígitos são verbos, `G`/`H`/`T`/`S`
//! /`A` são verbos, `C`/`I`/`B`/`N` são máscara, `K`/`J`/`V`/`O`/`P`/`U` são
//! topologia, `X`/`Y`/`Z` o espelho, `Q`/`E`/`R`/`F` a luz, `D` a doação. Um
//! `Shift+G` também não serve: o `match` dos verbos não olha o `shift`, então
//! ele já significa *pegar o Move*.
//!
//! # As três medições
//!
//! * **mover** — o delta de tela inteiro, descido à profundidade do pivô pela
//!   MESMA porta do Grab ([`Camera3d::screen_delta_to_world`]).
//! * **girar** — o ângulo VARRIDO em torno do **pivô projetado**, pela mesma
//!   porta do Twist (`swept_angle_about`), em torno da reta **olho→pivô**.
//!   ⚠️ As duas metades saem do PIVÔ, e a primeira versão tirava as duas do
//!   pixel de pen-down: o smoke reprovou com as três consequências juntas —
//!   *"a direção da rotação do mouse está invertida … e é imprecisa (não
//!   consistente)"* —, e a medição as separou em sinal, magnitude (0,45× do
//!   pedido) e proporcionalidade. Ver `sculpt3d_transform_tests`.
//! * **escalar** — a **razão de distâncias** ao pivô projetado, e não um mapa
//!   linear de pixels: assim o ponto sob o dedo continua sob o dedo, que é o que
//!   *manipulação direta* significa.
//!
//! Todas medem do **pen-down até agora** (o gesto TOTAL), porque é isso que a
//! lei do kernel consome — ver o cabeçalho de `transform.rs`.

use super::{Sculpt3dScene, StrokeUndo};
use ph2d_sculpt3d::{Gesture, MaskTransform, TransformKind};

/// A menor distância ao pivô, em pixels, que serve de referência para a escala.
///
/// ⚠️ **Não é um teto de recurso, é a régua da razão:** o gesto começa a dividir
/// por `|âncora − pivô|`, e um pen-down EM CIMA do pivô daria zero. O piso
/// converte a divisão impossível numa escala muito sensível — que o artista
/// percebe e corrige — em vez de num salto para o infinito.
const MIN_SCALE_REF_PX: f32 = 8.0;

impl Sculpt3dScene {
    /// O que o painel armou, se armou.
    pub(crate) fn transform_arm(&self) -> Option<TransformKind> {
        self.transform_arm
    }

    /// **Arma ou desarma** — clicar o que já está armado desliga, que é a lei de
    /// todo rádio deste app.
    pub(crate) fn arm_transform(&mut self, kind: TransformKind) -> bool {
        let on = self.transform_arm != Some(kind);
        self.transform_arm = if on { Some(kind) } else { None };
        if on {
            // ⚠️ **A EXCLUSÃO tem DUAS metades, e a primeira versão só tinha
            // esta ausente.** O filtro (`sculpt3d_filter`) reivindica o MESMO
            // botão esquerdo, e com a exclusão escrita só do outro lado a ORDEM
            // dos cliques decidia o que ele faz — armar o transform sobre um
            // filtro aceso deixava os dois vivos e o `pointer_down` escolhia
            // pela ordem em que os dois `if` estão escritos. Quem pegou foi o
            // gate `the_two_arms_never_claim_the_left_button_together`, que
            // afirma os dois sentidos de propósito.
            //
            // ⚠️ **Um `enum` de modo seria a resposta mais limpa** e obrigaria
            // a reescrever os cinco leitores do `transform_arm`; a exclusão por
            // porta custa duas linhas em cada uma e tem gate.
            self.filter_arm = false;
        }
        on
    }

    /// **Congela a foto e abre a sessão.** `false` quando não há o que mover — e
    /// a recusa é REPORTADA pelo chamador: um gesto que não faz nada e não diz
    /// nada é indistinguível de um botão que não chegou.
    pub(super) fn begin_transform(&mut self, x: f32, y: f32) -> bool {
        let Some(session) = MaskTransform::begin(self.mesh()) else {
            return false;
        };
        let pivot_world = self.pose().point_to_world(session.pivot());
        self.transform = Some(session);
        self.transform_from = (x, y);
        // ⚠️ O acumulador de ângulo é COMPARTILHADO com o verbo Twist, e ele é
        // do GESTO: deixá-lo vivo faria este transform começar já torcido, no
        // ponto onde o gesto anterior parou. Mesma linha do `pointer_down` — e
        // os dois braços abaixo o reescrevem, então nenhum caminho o herda.
        //
        // ⚠️ **Ele é ARMADO, não apenas limpo**, e é a diferença entre a peça
        // acompanhar a mão e ficar um passo atrás para sempre — ver
        // [`Sculpt3dScene::arm_sweep_at`]. O centro é o pivô PROJETADO, o mesmo
        // que o [`Self::transform_gesture`] usa; um pivô fora da tela não tem
        // referência a armar, e aí o gesto de girar já se recusa por conta
        // própria.
        match self.camera.project(pivot_world, self.viewport) {
            Some(c) => self.arm_sweep_at(c, x, y),
            None => self.twist = None,
        }
        true
    }

    /// O dedo andou: re-escreve a malha do `pre` com o gesto TOTAL.
    pub(super) fn transform_at(&mut self, x: f32, y: f32) {
        let Some(kind) = self.transform_arm else {
            return;
        };
        let Some(gesture) = self.transform_gesture(kind, x, y) else {
            return;
        };
        // A sessão SAI da cena para o escopo do `apply`: ela e a malha são dois
        // campos do mesmo `self`, e sem isto o empréstimo não fecha. É o mesmo
        // desenho do `take_masks` do `mask_ops`, um módulo ao lado.
        let Some(mut session) = self.transform.take() else {
            return;
        };
        session.apply(self.objects[self.active].stack.mesh_mut(), &gesture);
        let moved = session.moving().to_vec();
        Self::mesh_changed(
            &mut self.objects[self.active].dirty,
            &mut self.edits,
            &moved,
        );
        self.transform = Some(session);
    }

    /// A medição do gesto, por espécie — **em coordenadas LOCAIS da peça**.
    fn transform_gesture(&mut self, kind: TransformKind, x: f32, y: f32) -> Option<Gesture> {
        let session = self.transform.as_ref()?;
        let pivot_local = session.pivot();
        let pivot_world = self.pose().point_to_world(pivot_local);
        let from = self.transform_from;
        Some(match kind {
            TransformKind::Move => {
                let world = self.camera.screen_delta_to_world(
                    pivot_world,
                    x - from.0,
                    y - from.1,
                    self.viewport,
                );
                // ⚠️ **O deslocamento DESCE a pose e a fração NÃO** — a mesma
                // assimetria que o `turn_at` já carrega: um comprimento vive na
                // escala da peça, um ângulo e uma razão são adimensionais.
                Gesture::Move {
                    delta: self.pose().vector_to_local(world),
                }
            }
            TransformKind::Rotate => {
                // ⚠️ **As DUAS metades do giro saem do PIVÔ, e nenhuma do
                // pen-down.** O eixo é a reta olho→pivô (ver
                // [`Sculpt3dScene::view_axis_local`]) e a varredura é medida em
                // torno do pivô PROJETADO — é isso que faz o barro acompanhar a
                // mão volta por volta. Medir do pen-down entregava 0,31×–0,45×
                // do que o dedo pediu, e com o sinal trocado.
                //
                // ⚠️ **Pivô fora da tela ⇒ gesto NENHUM**, e não um neutro: o
                // que já foi girado FICA (o `pre` congelado é a entrada de todo
                // frame), enquanto um neutro devolveria a peça ao começo por o
                // artista ter passado por um pixel onde a pergunta não tem
                // resposta. É a mesma recusa do [`Self::scale_ratio`], pelo
                // mesmo motivo: não se mira um centro que não está na tela.
                let center = self.camera.project(pivot_world, self.viewport)?;
                let axis = self.view_axis_local(pivot_world);
                let radians = self.swept_angle_about(center, x, y)?;
                Gesture::Rotate { axis, radians }
            }
            TransformKind::Scale => Gesture::Scale {
                factor: self.scale_ratio(pivot_world, from, x, y),
            },
        })
    }

    /// A razão de distâncias ao pivô PROJETADO — a régua da escala.
    ///
    /// ⚠️ **Se o pivô não projeta** (ele está atrás da câmera) o gesto vira
    /// neutro em vez de inventar um número: escalar em torno de um centro que
    /// não está na tela é um gesto que o artista não consegue mirar.
    fn scale_ratio(&self, pivot_world: [f32; 3], from: (f32, f32), x: f32, y: f32) -> f32 {
        let Some(c) = self.camera.project(pivot_world, self.viewport) else {
            return 1.0;
        };
        let d0 = (from.0 - c.0).hypot(from.1 - c.1).max(MIN_SCALE_REF_PX);
        let d1 = (x - c.0).hypot(y - c.1);
        d1 / d0
    }

    /// Fecha a sessão e grava **UM** passo de undo.
    ///
    /// ⚠️ **Zero variant novo:** a entrada é a `StrokeUndo::Stroke` que todo
    /// traço já usa — a janela é [`MaskTransform::moving`] e o *antes* é
    /// [`MaskTransform::base_positions`], que é exatamente a forma que o
    /// `close_stroke` grava. Um variant próprio seria uma segunda resposta a
    /// *"como se desfaz um punhado de vértices deslocados"*.
    pub(super) fn close_transform(&mut self) {
        let Some(session) = self.transform.take() else {
            return;
        };
        if session.moving().is_empty() {
            return;
        }
        let entry = StrokeUndo::Stroke {
            level: self.level(),
            verts: session.moving().to_vec(),
            positions: session.base_positions().to_vec(),
            masks: None,
        };
        self.record(entry);
    }
}

#[cfg(test)]
#[path = "sculpt3d_transform_tests.rs"]
mod tests;
