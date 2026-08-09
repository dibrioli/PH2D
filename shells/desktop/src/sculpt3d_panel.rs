//! **A COSTURA COM O PAINEL** — o retrato que ele pinta e o gesto que ele
//! devolve (ADR-0150, W12).
//!
//! Filho (`#[path]`) de [`super`] pelo motivo dos outros: ele alcança os campos
//! privados da cena. O corte é entre *o que a mão faz com o ponteiro e o
//! teclado* (`input`/`keys`) e *o que ela pede pelo PAINEL* — e a diferença que
//! justifica um arquivo é que aqui o gesto chega como DADO, um frame depois,
//! pela fila de intents.
//!
//! ⚠️ **Uma porta, dois vocabulários.** O painel fala `Sculpt3dUi` — o estado
//! autorado inteiro — e comandos nomeados; a cena fala campos privados e verbos
//! com consequência. A tradução mora AQUI e em lugar nenhum mais: um segundo
//! sítio que escrevesse `scene.cavity` a partir de um intent seria a segunda
//! resposta a *"quem manda na cavidade?"*, e as duas divergiriam no frame em que
//! só uma delas fosse corrigida.

use ph2d_panel_sculpt3d::{Sculpt3dIntent, Sculpt3dSnapshot, Sculpt3dUi};

use super::dyntopo::DETAIL_STEPS;
use super::{MaskOp, Primitive, Sculpt3dScene};

impl Sculpt3dScene {
    /// **O retrato deste frame.** Tudo o que o painel pinta sai daqui, e ele não
    /// guarda cópia de nada — é a mesma lei do painel de física.
    ///
    /// `has_bake_target` vem de FORA porque ele é um fato da cena **2D** — quem
    /// está selecionado no canvas não é pergunta que uma escultura responda.
    pub(crate) fn panel_snapshot(&self, has_bake_target: bool) -> Sculpt3dSnapshot {
        let light = self.rig.current();
        Sculpt3dSnapshot {
            transform: self.transform_arm(),
            ui: Sculpt3dUi {
                brush: self.brush,
                // ⚠️ **O raio publicado é o CLAMPADO**, e não o `radius_px` cru:
                // o teto real é 1/8 da altura do viewport, então numa janela
                // baixa o número que a pista mostraria seria um que o dab não
                // usa. Publicar o clampado faz a pista *voltar* ao encostar no
                // teto, que é a verdade — e é a mesma porta que o log da tecla
                // `[`/`]` imprime, para os dois não discordarem.
                radius_px: self.radius_px(),
                symmetry: self.symmetry,
                matcap: self.matcap,
                alpha_preview: self.alpha_preview,
                wireframe: self.wireframe,
                cavity: self.cavity,
                env: self.env,
                ao: self.ao,
                ssao: self.ssao,
                sss: self.sss,
                sss_scatter: self.sss_scatter,
                light_az_deg: f32::from(light.angle_deg),
                light_elev_deg: f32::from(light.elev_deg),
                detail: detail_index(self.dyntopo.detail),
                extract: self.extract,
            },
            dyntopo: self.dyntopo.armed,
            level: self.level(),
            level_count: self.level_count(),
            ao_stale: self.mesh().ao_is_stale(),
            pieces: self.objects.len(),
            isolated: self.isolated.is_some(),
            matcaps: ph2d_mesh_render::MATCAPS.as_slice(),
            verts: self.mesh().vert_count(),
            alpha_seed: ph2d_sculpt3d::recommended_scale(self.mesh()),
            model_span: {
                // O MAIOR lado, e não a diagonal — a mesma régua que o
                // `recommended_scale` usa, e ele já pagou o `√3` de escolher a
                // outra. Duas réguas para uma grandeza é a doença de sempre.
                let b = self.mesh().bounds();
                (b.max[0] - b.min[0])
                    .max(b.max[1] - b.min[1])
                    .max(b.max[2] - b.min[2])
            },
            has_bake_target,
        }
    }

    /// **O estado autorado, aplicado.** O painel manda a struct INTEIRA a cada
    /// arrasto; escrever campo a campo aqui é o que mantém um intent só.
    fn apply_ui(&mut self, ui: &Sculpt3dUi) {
        self.brush = ui.brush;
        // O raio autorado é o número cru; o clamp mora na porta `radius_px()`,
        // que é quem o dab e o retrato perguntam. Guardar o clampado aqui faria
        // o valor ENCOLHER de vez ao passar por uma janela baixa, e ele nunca
        // mais voltaria quando ela crescesse.
        self.radius_px = ui.radius_px;
        self.symmetry = ui.symmetry;
        self.matcap = ui.matcap;
        self.alpha_preview = ui.alpha_preview;
        self.wireframe = ui.wireframe;
        self.cavity = ui.cavity;
        self.env = ui.env;
        self.ao = ui.ao;
        self.ssao = ui.ssao;
        self.sss = ui.sss;
        self.sss_scatter = ui.sss_scatter;
        let l = self.rig.current_mut();
        // Graus INTEIROS é a unidade em que o rig é autorado — o `f32` existe só
        // porque a pista de um slider é contínua.
        l.angle_deg = (ui.light_az_deg.round().max(0.0) as u16) % 360;
        l.elev_deg =
            (ui.light_elev_deg.round().max(0.0) as u16).clamp(ph2d_light::MIN_ELEV_DEG, 90);
        self.dyntopo.detail = DETAIL_STEPS[(ui.detail as usize).min(DETAIL_STEPS.len() - 1)].0;
        self.extract = ui.extract;
    }

    /// **Um gesto do painel**, traduzido para a cena — os mesmos desfechos que as
    /// teclas imprimem, pela MESMA porta, porque um botão e um atalho que
    /// fizessem coisas diferentes seriam duas ferramentas com um nome só.
    ///
    /// Devolve `true` quando o gesto **não é para a cena**: ele pede o bake, que
    /// só o laço de frame consegue fazer (ver [`Sculpt3dIntent::BakeToSprite`]).
    ///
    /// ⚠️ **O `match` continua exaustivo, e é isso que este retorno compra.** A
    /// alternativa era o bridge filtrar o intent antes de chegar aqui — e aí a
    /// tradução passaria a morar em dois lugares, com o segundo sendo uma
    /// cascata de `if` que apodrece calada no dia em que nascer o próximo verbo
    /// que o frame tem de executar.
    pub(crate) fn apply_panel_intent(&mut self, intent: Sculpt3dIntent) -> bool {
        match intent {
            // ⚠️ Ele ARMA e sai, e não faz nada com a cena: o bake precisa do
            // mundo, do renderizador e do mapa de atlas, e os três só existem
            // dentro do frame. Mesmo desenho do `Shift+B`.
            Sculpt3dIntent::BakeToSprite => return true,
            Sculpt3dIntent::SetUi(ui) => self.apply_ui(&ui),
            // ⚠️ **Quem decide o desligamento é a CENA**, não o painel: o
            // intent carrega o TIPO, e `arm_transform` compara com o que já
            // está armado. Duas cópias dessa comparação divergiriam no dia em
            // que o arm ganhasse um segundo escritor (uma tecla, um smoke).
            Sculpt3dIntent::ArmTransform(kind) => {
                let on = self.arm_transform(kind);
                eprintln!(
                    "[sculpt3d] transform: {} {}",
                    kind.label(),
                    if on {
                        "ARMADO -- o botao esquerdo move a parte LIVRE (clique de novo para desarmar)"
                    } else {
                        "desarmado -- o botao esquerdo volta a esculpir"
                    }
                );
            }
            Sculpt3dIntent::ToggleDyntopo => {
                let (on, tris) = self.toggle_dyntopo();
                eprintln!(
                    "[sculpt3d] topologia dinamica {} ({tris} faces trianguladas)",
                    if on { "LIGADA" } else { "DESLIGADA" }
                );
            }
            Sculpt3dIntent::ChangeLevel(up) => {
                if self.change_level(up) {
                    eprintln!(
                        "[sculpt3d] nivel {} de {} -- {} vertices",
                        self.level(),
                        self.level_count().saturating_sub(1),
                        self.mesh().vert_count()
                    );
                } else {
                    eprintln!(
                        "[sculpt3d] ja' esta' no {}",
                        if up { "TOPO" } else { "nivel 0" }
                    );
                }
            }
            Sculpt3dIntent::Subdivide => {
                if self.subdivide() {
                    eprintln!(
                        "[sculpt3d] subdividida: nivel {} -- {} vertices / {} faces",
                        self.level(),
                        self.mesh().vert_count(),
                        self.mesh().face_count()
                    );
                } else {
                    eprintln!("[sculpt3d] so' do TOPO: suba (+) antes de subdividir");
                }
            }
            Sculpt3dIntent::ReverseLevel => {
                if self.reverse_level() {
                    eprintln!("[sculpt3d] revertida: nivel {}", self.level());
                } else {
                    eprintln!("[sculpt3d] nao' reverte: esta malha nao e' uma subdivisao");
                }
            }
            Sculpt3dIntent::Remesh => match self.remesh(ph2d_sdf::DEFAULT_RESOLUTION) {
                Some(r) => eprintln!(
                    "[sculpt3d] reconstruida: {} -> {} vertices / {} -> {} faces",
                    r.verts.0, r.verts.1, r.faces.0, r.faces.1
                ),
                None => eprintln!("[sculpt3d] nao' reconstroi com a pilha montada: reverta antes"),
            },
            Sculpt3dIntent::BakeAo => {
                let r = self.bake_ao();
                eprintln!("[sculpt3d] AO assado: {} vertices em {:.0} ms", r.0, r.1);
            }
            Sculpt3dIntent::CloseHoles => match self.close_holes() {
                Some(r) if r.is_noop() => {
                    eprintln!("[sculpt3d] nenhum buraco: a malha ja' e' fechada");
                }
                Some(r) => eprintln!("[sculpt3d] tapados {} buraco(s)", r.filled()),
                None => {
                    eprintln!("[sculpt3d] nao' tapa com a pilha montada: tape ANTES de subdividir")
                }
            },
            Sculpt3dIntent::AddSphere => self.add_from_panel(Primitive::Sphere),
            Sculpt3dIntent::AddCube => self.add_from_panel(Primitive::Cube),
            Sculpt3dIntent::AddCylinder => self.add_from_panel(Primitive::Cylinder),
            Sculpt3dIntent::AddTorus => self.add_from_panel(Primitive::Torus),
            Sculpt3dIntent::Duplicate => {
                self.duplicate_active();
                eprintln!(
                    "[sculpt3d] DUPLICOU: a cena tem {} pecas",
                    self.objects.len()
                );
            }
            Sculpt3dIntent::Delete => {
                if self.delete_active() {
                    eprintln!("[sculpt3d] APAGOU: sobram {} pecas", self.objects.len());
                } else {
                    eprintln!("[sculpt3d] a cena ja' esta' VAZIA: nao ha' peca a apagar");
                }
            }
            Sculpt3dIntent::ToggleIsolate => {
                let on = self.toggle_isolate();
                eprintln!(
                    "[sculpt3d] {}",
                    if on {
                        "ISOLADA (o pincel nao alcanca o que nao se ve)"
                    } else {
                        "a cena inteira voltou"
                    }
                );
            }
            // ⚠️ Os TRÊS desfechos são reportados, e não só o bem-sucedido: a
            // fusão **não muda a silhueta da cena** (as peças ficam onde
            // estavam), então sem a contagem o artista vê a mesma imagem e não
            // tem como saber se o botão fez alguma coisa — é a mesma razão pela
            // qual a tecla `Shift+J` imprime os três.
            Sculpt3dIntent::Merge => match self.merge_visible() {
                super::Merge::Done {
                    pieces,
                    verts,
                    faces,
                } => eprintln!(
                    "[sculpt3d] FUNDIDAS {pieces} pecas numa so' -- {verts} vertices / {faces} faces"
                ),
                super::Merge::Nothing => {
                    eprintln!(
                        "[sculpt3d] nao ha' o que fundir: e' preciso mais de UMA peca a' vista"
                    );
                }
                super::Merge::Stack => {
                    eprintln!("[sculpt3d] nao' funde com a pilha montada: reverta os niveis antes")
                }
            },
            // ⚠️ **Os TRÊS desfechos, e não só o bem-sucedido** — a mesma
            // razão do Merge acima, e aqui ela é mais afiada: a peça nova nasce
            // EXATAMENTE em cima da que a gerou (é uma casca), então a silhueta
            // da cena não muda e o artista não tem como saber se o botão fez
            // alguma coisa. A contagem de peças é a evidência.
            Sculpt3dIntent::Extract => match self.extract_masked(self.extract) {
                super::Extracted::Done { verts, faces } => eprintln!(
                    "[sculpt3d] EXTRAIU: peca nova com {verts} vertices / {faces} faces \
                     (a cena tem {}) -- ela nasce EM CIMA da origem e vira a ativa; Ctrl+Z a tira",
                    self.objects.len()
                ),
                super::Extracted::NoMask => eprintln!(
                    "[sculpt3d] nao ha' o que extrair: pinte uma mascara antes (verbo Mask, tecla M)"
                ),
                super::Extracted::Nothing => {
                    eprintln!("[sculpt3d] a cena esta' VAZIA: nao ha' peca de onde extrair")
                }
            },
            Sculpt3dIntent::MaskClear => self.mask_from_panel(MaskOp::Clear),
            Sculpt3dIntent::MaskInvert => self.mask_from_panel(MaskOp::Invert),
            Sculpt3dIntent::MaskBlur => self.mask_from_panel(MaskOp::Blur),
            Sculpt3dIntent::MaskSharpen => self.mask_from_panel(MaskOp::Sharpen),
        }
        false
    }

    fn add_from_panel(&mut self, kind: Primitive) {
        let i = self.add_primitive(kind);
        eprintln!(
            "[sculpt3d] + {} (peca {i}, a cena tem {})",
            kind.label(),
            self.objects.len()
        );
    }

    fn mask_from_panel(&mut self, op: MaskOp) {
        self.mask_op(op);
        eprintln!("[sculpt3d] mascara: {}", op.label());
    }
}

/// Em que degrau da tabela um detalhe caiu.
///
/// ⚠️ **O mais PRÓXIMO, e não uma igualdade exata.** O rótulo do log usa
/// `abs() < 1e-6` e devolve `"custom"` quando nada casa — o que é honesto num
/// texto e inútil num rádio, que tem de acender alguma coisa. Um valor fora dos
/// três degraus só existe se alguém o escrever à mão; acender o vizinho é a
/// leitura menos surpreendente.
fn detail_index(detail: f32) -> u8 {
    let mut best = 0usize;
    let mut best_d = f32::INFINITY;
    for (i, (d, _)) in DETAIL_STEPS.iter().enumerate() {
        let dist = (d - detail).abs();
        if dist < best_d {
            best_d = dist;
            best = i;
        }
    }
    u8::try_from(best).unwrap_or(0)
}
