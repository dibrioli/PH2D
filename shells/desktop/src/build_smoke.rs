//! **A cena pronta para o smoke do Shape Builder** (`PH2D_BUILD_SMOKE`).
//!
//! O Enio não deve ter de montar a cena para testar a ferramenta — e o agente que só
//! *imagina* o roteiro do smoke escreve gates verdes sobre um produto quebrado (foi o que
//! aconteceu com a 1ª versão desta feature). Este hook monta a cena do print dele —
//! **pentágono + estrela + retângulo arredondado, sobrepostos** —, seleciona as três e entra
//! no modo Build. O canvas já abre como mesa de trabalho.
//!
//! - `PH2D_BUILD_SMOKE=21` — a cena da **W0** (texto + efeitos): a palavra "PATH" com um Zig Zag
//!   ATIVO, já selecionada. Mexa em qualquer knob da seção **Text** (ou escreva mais uma letra) —
//!   **a rugosidade tem de continuar lá**. Antes da W0 ela sumia neste gesto, em silêncio.
//! - `PH2D_BUILD_SMOKE=22` — a cena da **W3** (texto em CAMINHO): uma onda e dois círculos,
//!   com o texto a cavalgá-los; a 2ª linha corre paralela, o 3º está virado.
//! - `PH2D_BUILD_SMOKE=24` — a cena do **GESTO** do Pattern Along Path (plano 23, W3): um motivo
//!   (seta) e um guia (arco), os dois selecionados — clique **Pattern on Path** no painel e afine
//!   Spacing/Start/Side; as cópias giradas sobre a curva, a fonte intocada.
//! - `PH2D_BUILD_SMOKE=23` — a cena do **GESTO** (W4): um texto e uma curva já selecionados,
//!   para o artista os prender pelo painel (seção Text on Path).
//! - `PH2D_BUILD_SMOKE=19` — a cena do 17 com o **fluxo do report de 2026-07-20**: arma o
//!   Corner Round ANTES, arrasta o Offset até SATURAR (o gesto natural), solta e retuna
//!   Bevel → Miter. Com a lei da forma (±maxdim/2) o saturado é "a forma dobrada" e cada
//!   retune muda pixels visíveis — era o regime join-inerte da faixa antiga (±4 de mundo).
//! - `PH2D_BUILD_SMOKE=18` — a MESMA cena do 17, com o roteiro do **RETUNE auto-dirigido**
//!   (diagnóstico): rola o painel, arrasta o slider de Offset com o ponteiro a um d
//!   moderado, solta, clica Round, Bevel e Miter — e loga por frame o custo, o undo, a
//!   janela e os verts.
//! - `PH2D_BUILD_SMOKE=17` — a cena do **EXPAND**: um zig-zag só-traço, uma estrela com traço E
//!   preenchimento, e um donut de parede fina. Outline Stroke nos dois primeiros, Offset Path no
//!   terceiro (a borda cresce e o furo encolhe).
//! - `PH2D_BUILD_SMOKE=16` — a cena das ferramentas de **QUINA** (Fillet / Chamfer): um retângulo
//!   de quinas retas + uma elipse de âncoras suaves, no modo Fillet. Clique uma quina e ARRASTE
//!   para arredondar (Fillet) ou chanfrar (Chamfer, pelo pill). Na elipse o clique transforma a
//!   âncora suave em quina primeiro. É a consolidação da alça do Node + o toggle da seção Vertex.
//! - `PH2D_BUILD_SMOKE=15` — a cena do **CHAMFER** (ADR-0121): um quadrado de quinas arredondadas,
//!   no modo Node, com as 4 quinas selecionadas. Na seção **Vertex** clique **Chamfer** — cada
//!   quina vira uma RETA de mesmo recuo. Arraste a alça de raio e o estilo sobrevive.
//! - `PH2D_BUILD_SMOKE=14` — a cena do **APPLY / CONVERT** (ADR-0132): uma elipse com um Zig Zag
//!   ATIVO, ESTÁTICA e já selecionada. Na seção **Effects** clique **Apply Effects** (assa a
//!   pilha na geometria); ou **Convert to Curves** (que agora também assa efeitos). O card some e
//!   a borda rugosa vira curva editável.
//! - `PH2D_BUILD_SMOKE=13` — a cena da **PILHA DE EFEITOS** (ADR-0132): uma elipse que se
//!   DESENHA sozinha (o `end` do Trim sobe de 0 a 1) e uma estrela em que a janela do Trim
//!   GIRA à volta da forma. A ponta anda a velocidade constante — é a medida por ARCO.
//! - `PH2D_BUILD_SMOKE=12` — a cena do **WARP GROUP** (ADR-0129 Fatia 3): DUAS elipses sob UMA
//!   gaiola. As duas curvam pela mesma perspectiva (NODE), e o gizmo do Select abraça e move as
//!   duas juntas (SELECT). É o que separa um container de dois envelopes soltos.
//! - `PH2D_BUILD_SMOKE=11` — a cena do **ENVELOPE** (ADR-0129): uma elipse **já deformada** por
//!   uma gaiola de perspectiva. OLHE O MEIO DOS SEGMENTOS — as laterais curvam liso; é aí que
//!   o defeito ingênuo (só os cantos) apareceria, não nos cantos.
//! - `PH2D_BUILD_SMOKE=10` — a cena do **MORPH** (o `t` animável): um quadrado e uma estrela, já
//!   SELECIONADOS. Clique **Morph** no painel — nasce UMA forma no meio do caminho. Arraste
//!   **Morph t**: ela caminha entre as duas, ao vivo. As fontes ficam (mexa numa e a forma
//!   refaz-se). Para animar: com o morph selecionado, **+ Track → Morph** na timeline, ponha o
//!   playhead e aperte **K** — o `t` vira uma curva.
//! - `PH2D_BUILD_SMOKE=9` — estrela → **círculo**, 5 passos — o par que o Enio testou à mão. A
//!   transição tem de ser limpa (as pontas encolhem radialmente, sem torcer). Rotate/Reverse Match
//!   foram removidos; o ajuste é editar as formas-fonte.
//! - `PH2D_BUILD_SMOKE=8` — a cena do **GIRO**: quadrado → **círculo**, 5 passos. É o par do 2º
//!   smoke do Enio (*"o porquê da rotação?"*). As quinas têm de caminhar **reto** para fora — sem
//!   rodar 45° e voltar.
//! - `PH2D_BUILD_SMOKE=7` — a cena do **BLEND**: um quadrado e uma estrela, com 3 passos entre eles.
//!   As quinas do quadrado casam com as PONTAS da estrela (não os vales), e as arestas retas ficam
//!   retas em todo o caminho.
//! - `PH2D_BUILD_SMOKE=1` — a cena, selecionada, no modo Build. **Passe o mouse** (o realce
//!   segue o cursor), **arraste** para unir, **Alt+arraste** para apagar.
//! - `PH2D_BUILD_SMOKE=2` — idem, e o gesto é dirigido por CÓDIGO: o dedo pousa e arrasta por
//!   duas faces, sem soltar. É o harness visual do véu — a única parte da feature cujo
//!   oráculo é o pixel, e a que estava sem gate quando o Enio reprovou.

// A MÃO dos roteiros (clique/tecla/estado pelo caminho real do input) mora no irmão
// `build_smoke_drive.rs` (HR-18).
use ph2d_vec_scene::{Paint, Rgba8, ShapeKind, VecPath, cook};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

/// O frame corrente do roteiro (o hook não pode acrescentar campo em `App`).
static FRAME: AtomicU32 = AtomicU32::new(0);

/// Dois pontos de MUNDO dentro de faces diferentes: um na estrela, outro no pentágono.
const IN_STAR: [f64; 2] = [0.35, 0.15];
const IN_PENT: [f64; 2] = [-1.2, 0.0];

/// O nível pedido, lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn level() -> u32 {
    static LEVEL: OnceLock<u32> = OnceLock::new();
    *LEVEL.get_or_init(|| {
        std::env::var("PH2D_BUILD_SMOKE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    })
}

pub(crate) fn shape(kind: ShapeKind, a: [f64; 2], b: [f64; 2], v: &[f64], rgb: [u8; 3]) -> VecPath {
    let mut p = cook(kind, a, b, v);
    p.fill = Some(Paint::solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
    p
}

impl crate::App {
    /// Roda no prólogo do frame, ANTES do `build_session_upkeep`. No-op sem a env.
    pub(crate) fn build_smoke(&mut self) {
        let level = level();
        if level == 0 || self.gfx.is_none() {
            return;
        }
        let f = FRAME.fetch_add(1, Ordering::Relaxed);
        // As cenas do ENVELOPE (níveis 11 e 12) vivem no módulo irmão `envelope_smoke` — teto de
        // LOC. Elas só usam os frames 3 e 4 e nenhum braço compartilhado, então sair do `match`
        // aqui é a MESMA sequência de antes: um nível fora de 11/12 nunca entrava nesses braços, e
        // 11/12 nunca chegavam aos genéricos (os específicos vinham primeiro).
        if matches!(level, 11 | 12) {
            crate::envelope_smoke::frame(self, f, level);
            return;
        }
        // A cena da PILHA de efeitos (ADR-0132), no módulo irmão `fx_smoke` — mesma razão de
        // LOC, e mesma disciplina: os níveis 13/14 nunca tocam um braço genérico abaixo. 13 é a
        // pilha animada; 14 é a cena do Apply / Convert (estática).
        if matches!(level, 13 | 14) {
            crate::fx_smoke::frame(self, f, level);
            return;
        }
        // O UNDO da pilha de efeitos, AUTO-DIRIGIDO (o report do Enio, 3×) — irmão `fx_undo_smoke`.
        if level == 20 {
            crate::fx_undo_smoke::frame(self, f);
            return;
        }
        // A cena da W0 (texto + pilha de efeitos sobrevive ao re-cook) — irmão `text_fx_smoke`.
        if level == 21 {
            crate::text_fx_smoke::frame(self, f);
            return;
        }
        // A cena da W3 (o texto cavalga o caminho) — irmão `text_path_smoke`.
        if level == 22 {
            crate::text_path_smoke::frame(self, f);
            return;
        }
        // A cena do GESTO (o artista prende o texto pelo painel) — irmão
        // `text_path_gesture_smoke`. Irmã da 22: aquela mostra o motor, esta o caminho até ele.
        if level == 23 {
            crate::text_path_gesture_smoke::frame(self, f);
            return;
        }
        // A cena do GESTO do Pattern Along Path (plano 23, W3): motivo + guia selecionados, o
        // artista prende pelo painel; daí o `pattern_live::recook` -> `dispatch` desenha as cópias.
        if level == 24 {
            crate::pattern_path_smoke::frame(self, f);
            return;
        }
        match f {
            // A cena. A geometria entra em MUNDO com o `Transform` na identidade — é como a
            // Shape tool deixa uma forma recém-desenhada; o `settle_origins` do frame a
            // centra no local 0 e põe a pose na entidade (ADR-0111/0112).
            // A cena do BLEND: duas formas distantes, com contagens de âncora diferentes (4 e
            // 10) — é o caso em que a correspondência importa.
            3 if level == 7 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
                let scene = &mut gfx.vec_scene;
                scene.push_path(shape(
                    ShapeKind::Rectangle,
                    [-3.4, -1.0],
                    [-1.4, 1.0],
                    &[],
                    [70, 110, 190],
                ));
                scene.push_path(shape(
                    ShapeKind::Star,
                    [1.4, -1.0],
                    [3.4, 1.0],
                    &[5.0, 0.45, 0.0],
                    [200, 120, 80],
                ));
            }
            // A cena do MORPH: as duas formas do blend, mas o objetivo é UMA forma animável. Elas
            // ficam SELECIONADAS (frame 4, depois de o `sync` lhes ter dado entidade) para o smoke
            // ser um clique só — o Enio não deve ter de montar nada.
            3 if level == 10 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
                let scene = &mut gfx.vec_scene;
                scene.push_path(shape(
                    ShapeKind::Rectangle,
                    [-3.4, -1.0],
                    [-1.4, 1.0],
                    &[],
                    [70, 110, 190],
                ));
                scene.push_path(shape(
                    ShapeKind::Star,
                    [1.4, -1.0],
                    [3.4, 1.0],
                    &[5.0, 0.45, 0.0],
                    [200, 120, 80],
                ));
            }
            4 if level == 10 => {
                let ids: Vec<_> = self
                    .gfx
                    .as_ref()
                    .expect("gfx")
                    .vec_scene
                    .paths()
                    .iter()
                    .map(|p| p.id)
                    .collect();
                self.vec_pen.select_many(&ids);
                eprintln!(
                    "[smoke] morph: 2 formas selecionadas — clique **Morph** no painel, depois \
                     arraste **Morph t**"
                );
            }
            // A cena do CHAMFER (ADR-0121): um quadrado com quinas ARREDONDADAS, no modo Node,
            // centrado na origem (o `settle` não o move → local == mundo, e a seleção de vértice
            // por coordenada acerta). Só o modo Node mostra a seção Vertex + as alças de quina.
            3 if level == 15 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
                let scene = &mut gfx.vec_scene;
                let id = scene.push_path(shape(
                    ShapeKind::Rectangle,
                    [-1.2, -1.2],
                    [1.2, 1.2],
                    &[],
                    [90, 150, 220],
                ));
                // Todas as 4 quinas arredondadas (raio > 0). O toggle Chamfer as vira retas.
                if let Some(p) = scene.path_mut(id) {
                    for v in &mut p.verts {
                        v.corner_radius = 0.4;
                    }
                }
                self.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Node);
            }
            // Seleciona as 4 quinas (frame 4, pós-`settle`) — assim a seção Vertex + o toggle
            // Chamfer já aparecem no primeiro olhar; um clique no toggle chanfra as quatro.
            4 if level == 15 => {
                let corners = [[-1.2, -1.2], [1.2, -1.2], [1.2, 1.2], [-1.2, 1.2]];
                let scene = &self.gfx.as_ref().expect("gfx").vec_scene;
                for c in corners {
                    self.vec_pen.toggle_vert_at(scene, c, 0.2);
                }
                eprintln!(
                    "[smoke] chamfer: quadrado arredondado, 4 quinas selecionadas no modo Node. \
                     Na seção **Vertex** clique **Chamfer** — as quinas viram RETAS (mesmo recuo). \
                     Arraste a alça de raio: o estilo sobrevive. Clique de novo p/ voltar a arco."
                );
            }
            // A cena das ferramentas de QUINA (Fillet / Chamfer) — corpo no módulo irmão
            // `build_smoke_corner_tools` (teto de LOC). Frame 3 monta, frame 4 pré-seleciona.
            3 if level == 16 => self.smoke_corner_tools_build(),
            4 if level == 16 => self.smoke_corner_tools_select(),
            // A cena do EXPAND (Outline Stroke + Offset Path) — corpo no módulo irmão
            // `build_smoke_expand` (teto de LOC).
            3 if level == 17 => self.smoke_expand_build(),
            4 if level == 17 => self.smoke_expand_select(),
            // Nível 18 = a cena do 17 com o roteiro do RETUNE auto-dirigido (diagnóstico).
            // Nível 19 = a MESMA cena com o fluxo do report de 2026-07-20 (Round armado
            // ANTES + arrasto SATURADO — o regime que era join-inerte na faixa antiga).
            3 if level == 18 || level == 19 => self.smoke_expand_build(),
            4 if level == 18 || level == 19 => {
                // O alvo do offset é o DONUT (a 3ª forma).
                let donut = self
                    .gfx
                    .as_ref()
                    .expect("gfx")
                    .vec_scene
                    .paths()
                    .get(2)
                    .map(|p| p.id);
                self.vec_pen.select(donut);
            }
            f18 if level == 18 && f18 >= 5 => self.smoke_expand_retune_drive(f18),
            f19 if level == 19 && f19 >= 5 => self.smoke_expand_saturate_drive(f19),
            // A cena do GIRO (o 2º smoke do Enio): quadrado → CÍRCULO. Ele teve de desenhar o
            // círculo à MÃO da última vez, porque a cena não o oferecia — e é justamente o par em
            // que o defeito aparecia (as intermediárias rodavam 45° e voltavam).
            3 if level == 8 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
                let scene = &mut gfx.vec_scene;
                scene.push_path(shape(
                    ShapeKind::Rectangle,
                    [-3.4, -1.0],
                    [-1.4, 1.0],
                    &[],
                    [70, 110, 190],
                ));
                scene.push_path(shape(
                    ShapeKind::Ellipse,
                    [1.4, -1.0],
                    [3.4, 1.0],
                    &[],
                    [200, 120, 80],
                ));
            }
            // A cena estrela → CÍRCULO — o par que o Enio testou à mão. Um lado tem quinas (a
            // estrela), o outro é liso (o círculo): a transição tem de encolher radialmente, sem
            // torcer. Rotate/Reverse Match foram removidos; o ajuste é editar as formas.
            3 if level == 9 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
                let scene = &mut gfx.vec_scene;
                scene.push_path(shape(
                    ShapeKind::Star,
                    [-3.4, -1.0],
                    [-1.4, 1.0],
                    &[5.0, 0.45, 0.0],
                    [70, 110, 190],
                ));
                scene.push_path(shape(
                    ShapeKind::Ellipse,
                    [1.4, -1.0],
                    [3.4, 1.0],
                    &[],
                    [200, 120, 80],
                ));
            }
            3 => {
                let gfx = self.gfx.as_mut().expect("gfx");
                let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
                let scene = &mut gfx.vec_scene;
                scene.push_path(shape(
                    ShapeKind::RoundRect,
                    [-1.6, -1.1],
                    [1.6, 1.1],
                    &[0.4, 0.0, 0.0, 0.0, 0.0],
                    [70, 110, 190],
                ));
                scene.push_path(shape(
                    ShapeKind::Polygon,
                    [-1.9, -0.9],
                    [-0.1, 0.9],
                    &[5.0, 0.0],
                    [200, 120, 80],
                ));
                scene.push_path(shape(
                    ShapeKind::Star,
                    [-0.3, -1.0],
                    [1.7, 1.0],
                    &[5.0, 0.45, 0.0],
                    [110, 190, 130],
                ));
            }
            // BLEND, o par do GIRO: quadrado → círculo, 5 passos (com 3 o giro de 45° era fácil
            // de confundir com "a forma está virando um círculo"). O que se olha aqui é UMA coisa:
            // as quinas caminham RETO para fora, sem rodar e voltar.
            8 if level == 8 => {
                let ids: Vec<u64> = self
                    .gfx
                    .as_ref()
                    .expect("gfx")
                    .vec_scene
                    .paths()
                    .iter()
                    .map(|p| p.id)
                    .collect();
                self.vec_pen.select_many(&ids);
                self.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Select);
                let Some(gfx) = self.gfx.as_mut() else { return };
                let xf = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
                crate::vec_blend::apply(
                    &mut gfx.vec_scene,
                    &mut self.vec_history,
                    &mut self.vec_pen,
                    &xf,
                    &mut self.vec_blend,
                    5,
                    true,
                );
                self.vec_restack = self
                    .vec_blend
                    .as_ref()
                    .map(crate::vec_blend::BlendSession::stack)
                    .into_iter()
                    .collect();
                self.any_input_this_frame = true;
                eprintln!(
                    "[blend-smoke] quadrado -> CIRCULO, 5 passos (o par que girava 45 graus). \
                     As quinas tem de sair RETO, sem rodar."
                );
            }
            // estrela -> circulo, blendado. A transicao tem de ser limpa (pontas encolhem
            // radialmente, sem torcer). Rotate/Reverse Match foram removidos.
            8 if level == 9 => {
                let ids: Vec<u64> = self
                    .gfx
                    .as_ref()
                    .expect("gfx")
                    .vec_scene
                    .paths()
                    .iter()
                    .map(|p| p.id)
                    .collect();
                self.vec_pen.select_many(&ids);
                self.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Select);
                let Some(gfx) = self.gfx.as_mut() else { return };
                let xf = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
                crate::vec_blend::apply(
                    &mut gfx.vec_scene,
                    &mut self.vec_history,
                    &mut self.vec_pen,
                    &xf,
                    &mut self.vec_blend,
                    5,
                    true,
                );
                self.vec_restack = self
                    .vec_blend
                    .as_ref()
                    .map(crate::vec_blend::BlendSession::stack)
                    .into_iter()
                    .collect();
                self.any_input_this_frame = true;
                eprintln!(
                    "[blend-smoke] estrela -> CIRCULO, 5 passos. A transicao tem de encolher \
                     radialmente, sem torcer. Rotate/Reverse Match foram removidos."
                );
            }
            // BLEND: seleciona as duas e roda o blend pelo caminho REAL (a mesma função que o
            // botão do painel chama). O artista já abre o app com os passos na tela.
            8 if level == 7 => {
                let ids: Vec<u64> = self
                    .gfx
                    .as_ref()
                    .expect("gfx")
                    .vec_scene
                    .paths()
                    .iter()
                    .map(|p| p.id)
                    .collect();
                self.vec_pen.select_many(&ids);
                self.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Select);
                let Some(gfx) = self.gfx.as_mut() else { return };
                let xf = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
                crate::vec_blend::apply(
                    &mut gfx.vec_scene,
                    &mut self.vec_history,
                    &mut self.vec_pen,
                    &xf,
                    &mut self.vec_blend,
                    3,
                    true, // os passos sobem: cada um acima do anterior
                );
                self.vec_restack = self
                    .vec_blend
                    .as_ref()
                    .map(crate::vec_blend::BlendSession::stack)
                    .into_iter()
                    .collect();
                self.any_input_this_frame = true;
                eprintln!(
                    "[blend-smoke] quadrado -> estrela, 3 passos. As quinas casam com as PONTAS \
                     (nao os vales); as arestas retas ficam retas."
                );
            }
            // Seleciona as três e entra no Build — o estado em que o Enio começa a testar.
            //
            // ⚠️ Gate `level <= 6`, NÃO catch-all. Este arm é do Shape Builder (a cena default
            // `3 =>`, 3 formas, níveis 1-6). Os níveis de OBJETO vetorial (7-11) têm cena própria e
            // NÃO querem Build: 7/8/9 já têm seu `8 if level == N` acima; 10 (morph) fica no Select
            // que a cena deixou; 11 (envelope) fica no NODE que o frame 4 armou — e é o Build deste
            // arm, quando era catch-all, que engolia esse Node e sumia com a gaiola (a alça só
            // aparece no Node). Adicionar um nível novo sem cena de Build? ele cai em `_ => {}`.
            8 if level <= 6 => {
                let ids: Vec<u64> = self
                    .gfx
                    .as_ref()
                    .expect("gfx")
                    .vec_scene
                    .paths()
                    .iter()
                    .map(|p| p.id)
                    .collect();
                self.vec_pen.select_many(&ids);
                self.vec_set_draw_mode(ph2d_tool_vector::DrawMode::Build);
                eprintln!(
                    "[build-smoke] cena pronta, {} formas, modo Build",
                    ids.len()
                );
            }
            // O dedo pousa e arrasta por duas faces — e NÃO solta (o véu das pintadas fica
            // na tela para ser olhado).
            10 if level == 2 => {
                self.build_down(IN_STAR, false, false);
                self.build_move(IN_PENT);
            }
            f if f > 10 && level == 2 => {
                self.build_move(IN_STAR);
                self.build_move(IN_PENT);
            }
            // **Níveis 3 e 4 — o undo pelo CAMINHO REAL.** Nada de chamar `build_up` e
            // `undo_request` na mão: aqui entram `on_mouse_input` e `key_input`, que é por
            // onde o winit entra. É a diferença entre provar o mecanismo e provar o produto.
            //
            // O baseline precisa ser ARMADO: o hook cria as formas sem input nenhum, e o undo
            // global registra por diff **em frames com input** — sem isto o 1º clique
            // arrastaria "as 3 formas nasceram" para dentro do mesmo passo, e o Ctrl+Z
            // voltaria para a cena VAZIA. (No produto isso não acontece: desenhar é input.)
            //
            // O `>= 3` de antes VAZAVA para as cenas 7 e 8 (o Blend): elas passam do 3, então o
            // harness injetava um CLIQUE sintético em (0,35 · 0,15) quatro frames depois do blend
            // rodar — e o passo do meio nasce em cima desse ponto. No modo Select o clique PEGA a
            // forma (ADR-0112), a seleção trocava, e o Enio abria o smoke num estado que a doc
            // não descreve. O harness de undo é dos níveis 3..=6 e de mais ninguém.
            9 if (3..=6).contains(&level) => {
                self.any_input_this_frame = true;
                self.smoke_state("baseline (3 formas)");
            }
            12 if (3..=5).contains(&level) => self.smoke_click(IN_STAR),
            13 if (3..=5).contains(&level) => self.smoke_state("depois do clique"),
            14 if level == 3 || level == 4 => self.smoke_undo(false), // Ctrl+Z
            15 if level == 3 || level == 4 => self.smoke_state("depois do UNDO"),
            // Nível 3: redo direto. Nível 4: um clique no VAZIO ANTES do redo — é aí que um
            // passo espúrio apareceria, e um passo espúrio **limpa a pilha de redo**.
            16 if level == 4 => self.smoke_click([9.0, 9.0]),
            17 if level == 4 => self.smoke_state("depois de um clique no nada"),
            18 if level == 3 || level == 4 => self.smoke_undo(true), // Ctrl+Shift+Z
            19 if level == 3 || level == 4 => self.smoke_state("depois do REDO"),
            // **Nível 5 — os BOTÕES da barra**, clicados com o mouse de verdade: o ponteiro
            // acha o chip no hit-index, o widget emite o Click, o chrome despacha, o bus é
            // drenado e o shell desfaz. É o caminho inteiro, sem atalho nenhum.
            14 if level == 5 => self.smoke_rail_click(ph2d_editor::ids::TOOL_UNDO, "Undo"),
            15 if level == 5 => self.smoke_state("depois do BOTÃO Undo"),
            16 if level == 5 => self.smoke_rail_click(ph2d_editor::ids::TOOL_REDO, "Redo"),
            17 if level == 5 => self.smoke_state("depois do BOTÃO Redo"),
            // **Nível 6 — o bug do Enio: "undo só faz uma etapa".** Duas ações, depois três
            // Ctrl+Z, com o DOWN e o UP em frames SEPARADOS (é o que o winit entrega).
            12 if level == 6 => self.smoke_click(IN_STAR),
            13 if level == 6 => self.smoke_state("ação 1 (build na estrela)"),
            16 if level == 6 => self.smoke_click(IN_PENT),
            17 if level == 6 => self.smoke_state("ação 2 (build no pentágono)"),
            20 if level == 6 => self.smoke_key_z(false, true),
            21 if level == 6 => self.smoke_key_z(false, false),
            22 if level == 6 => self.smoke_state("Ctrl+Z #1"),
            24 if level == 6 => self.smoke_key_z(false, true),
            25 if level == 6 => self.smoke_key_z(false, false),
            26 if level == 6 => self.smoke_state("Ctrl+Z #2"),
            28 if level == 6 => self.smoke_key_z(false, true),
            29 if level == 6 => self.smoke_key_z(false, false),
            30 if level == 6 => self.smoke_state("Ctrl+Z #3"),
            _ => {}
        }
    }
}
