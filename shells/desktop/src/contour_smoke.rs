//! **A cena pronta para o smoke do CONTOUR** (`PH2D_BUILD_SMOKE=25`) — pesquisa `20_*` item #9.
//!
//! # A cena tem DUAS metades, e a razão é uma cicatriz
//!
//! *"nothing here is armed in code … the smoke that arms state under the table skips exactly the
//! seam it was supposed to prove"* — o doc do `impasto_smoke`, pago por um smoke que validou uma
//! feature inalcançável. Mas armar NADA deixa o artista sem referência do que a coisa deve
//! parecer. A resposta é a mesma que a `line/physics` deu na cena `=33`: **uma forma PELADA para
//! autorar pela UI, e uma PRONTA para julgar o desenho**.
//!
//! - a **estrela** entra selecionada e sem efeito: é ela que prova o seam inteiro (o botão *Add
//!   Contour* → os controles → a swatch → o Expand);
//! - o **hexágono** já vem com contour armado, para o olho ter a que comparar — e para o Expand
//!   ter o que materializar sem depender de o passo anterior ter corrido.

use ph2d_ecs::{Entity, VecContour};
use ph2d_vec_scene::ShapeKind;

use crate::build_smoke::shape;

/// Os números da cena, **MEDIDOS** e não escolhidos (`probe_contour_scene`, release, hexágono de
/// raio ~1,2 com quina Round):
///
/// | accel | `d` | anéis | externo | cozimento | anéis VIVOS |
/// |---|---|---|---|---|---|
/// | 1,0 | 0,12 | 0,12 … 0,72 | 0,72 | 2,28 ms | **6 de 6** |
/// | 1,0 | 0,14 | 0,14 … 0,84 | 0,84 | 2,04 ms | 5 de 6 |
/// | 1,3 | 0,12 | 0,12 … 1,23 | 1,23 | 2,32 ms | 5 de 6 |
/// | 1,6 | 0,12 | 0,12 … 2,11 | 2,11 | 3,14 ms | 5 de 6 |
///
/// ⚠️ **A coluna da direita é um defeito do `linesweeper`, não do Contour**, e é a razão de a
/// cena usar `accel = 1`: em certas distâncias o sweep não consegue responder e o anel sai VAZIO
/// (até 2026-07-23 ele entrava em PÂNICO e derrubava o app; o `offset_path` agora isola a queda —
/// ver o § dele). A cena demonstra com 6 de 6 e o artista arrasta o `accel` sabendo do buraco.
const DEMO_STEPS: u16 = 6;
const DEMO_D: f64 = 0.12;
const DEMO_ACCEL: f32 = 1.0;

impl crate::App {
    /// Monta a cena (frame 3): a estrela pelada + o hexágono que ganha contour no frame seguinte.
    pub(crate) fn smoke_contour_build(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
        let scene = &mut gfx.vec_scene;
        scene.push_path(shape(
            ShapeKind::Star,
            [-3.6, -1.4],
            [-0.8, 1.4],
            &[5.0, 0.45, 0.0],
            [235, 175, 60],
        ));
        scene.push_path(shape(
            ShapeKind::Polygon,
            [1.0, -1.2],
            [3.4, 1.2],
            &[6.0],
            [70, 130, 200],
        ));
    }

    /// Frame 4: seleciona a estrela (a metade que se autora) e arma o contour do hexágono (a
    /// metade que se julga). O arm é direto no componente **de propósito** — é a referência
    /// visual, não a prova do seam; quem prova o seam é a estrela, com a mão do artista.
    pub(crate) fn smoke_contour_arm(&mut self) {
        let ids: Vec<ph2d_vec_scene::VecPathId> = self
            .gfx
            .as_ref()
            .expect("gfx")
            .vec_scene
            .paths()
            .iter()
            .map(|p| p.id)
            .collect();
        let Some((&star, &hex)) = ids.first().zip(ids.get(1)) else {
            return;
        };
        self.vec_pen.select(Some(star));
        let Some(&bits) = self.vec_entities.get(&hex) else {
            eprintln!("[smoke] contour: o hexágono ainda não tem entidade — o `sync` não correu");
            return;
        };
        let sim = &mut self.gfx.as_mut().expect("gfx").sim;
        if let Ok(mut em) = sim.world_mut().get_entity_mut(Entity::from_bits(bits)) {
            em.insert(VecContour {
                steps: DEMO_STEPS,
                d: DEMO_D,
                accel: DEMO_ACCEL,
                join: 1,                 // Round: a quina que faz um contour parecer um
                side: 0,                 // Outer
                to: [230, 60, 120, 255], // o alvo da rampa: azul -> magenta
            });
        }
        eprintln!(
            "[smoke] CONTOUR — a seção **Contour** no painel (abaixo de Pattern on Path).\n\
             \x20 A cena: uma ESTRELA laranja (selecionada, sem efeito) e um HEXÁGONO azul que\n\
             \x20 já vem com {DEMO_STEPS} anéis, d={DEMO_D} por passo e accel={DEMO_ACCEL}.\n\
             \x20\n\
             \x20 1) **A estrela está selecionada e a seção mostra UM botão: `Add Contour`.**\n\
             \x20    Nenhum slider, nenhuma swatch — não há para onde eles escreverem ainda.\n\
             \x20    Clique. Os anéis aparecem NA HORA (nascem em 8% do tamanho da forma por\n\
             \x20    passo, de propósito: armar no neutro seria um clique que não muda um pixel).\n\
             \x20 2) Arraste **Steps**. Os anéis que já existem NÃO se movem — só entram/saem\n\
             \x20    novos. É o desenho inteiro: o `d` é por PASSO, então acrescentar um anel\n\
             \x20    custa um offset e não N. Com o modelo de \"alcance total\" cada passo\n\
             \x20    re-cozinharia tudo.\n\
             \x20 3) Arraste **Offset** para NEGATIVO: os anéis encolhem PARA DENTRO e passam a\n\
             \x20    ser desenhados na FRENTE da forma (senão ela os taparia). É o *Inside* do\n\
             \x20    Corel — o mesmo que escolher **Side: Inner** com offset positivo.\n\
             \x20 4) Clique a swatch **To** e escolha uma cor bem diferente do preenchimento.\n\
             \x20    Olhe o MEIO da rampa: ela passa por tons saturados, não por um cinza\n\
             \x20    lamacento — a interpolação é em **Oklab**, o mesmo espaço do picker.\n\
             \x20 5) **Accel** = 1 é linear; acima de 1 os anéis espalham para longe e as cores\n\
             \x20    com eles; abaixo de 1 amontoam junto da forma. O neutro está no CENTRO do\n\
             \x20    trilho (a faixa é geométrica) — solte o slider no meio e confira que volta\n\
             \x20    a ser linear.\n\
             \x20 6) **Side** é a DIREÇÃO (modelo do Corel): **Outer** joga os anéis para fora,\n\
             \x20    **Inner** para dentro (o espelho — nada mais some, como sumia antes), e\n\
             \x20    **Both** para os dois lados ao mesmo tempo. **Corner** (Miter/Round/Bevel)\n\
             \x20    re-cozinha na hora, sempre a partir da forma original.\n\
             \x20 7) Selecione o HEXÁGONO. Os controles têm de mostrar OS NÚMEROS DELE\n\
             \x20    ({DEMO_STEPS} passos, accel {DEMO_ACCEL}), não os da estrela — e voltar aos\n\
             \x20    da estrela quando você a selecionar de novo.\n\
             \x20 8) **Expand Contour** no hexágono: os anéis viram FORMAS de verdade (entre no\n\
             \x20    modo Node e edite as âncoras de uma delas). A FONTE fica — o contour era\n\
             \x20    desenho por cima dela. Nada pode SALTAR no clique: o que estava na tela é\n\
             \x20    exatamente o que passa a existir.\n\
             \x20 9) **Remove Contour** na estrela: ela volta a desenhar-se sozinha, e nada\n\
             \x20    de geometria fica para trás.\n\
             \x20 10) Ctrl+Z desfaz cada um destes passos (o undo global é por diff de estado).\n\
             \x20\n\
             \x20 ⚠️ **Se um anel SUMIR** ao mexer em Accel/Offset, é um buraco conhecido do motor\n\
             \x20 de offset (`linesweeper`), não do Contour: em certas distâncias o sweep não\n\
             \x20 consegue responder. Medido: numa estrela com quina **Bevel**, 118 de 200\n\
             \x20 distâncias. Até hoje ele DERRUBAVA o app (e o slider da seção Expand alcança\n\
             \x20 as mesmas distâncias); agora a queda é isolada e o anel apenas falta."
        );
    }
}
