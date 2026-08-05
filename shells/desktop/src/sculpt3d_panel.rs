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
    pub(crate) fn panel_snapshot(&self) -> Sculpt3dSnapshot {
        let light = self.rig.current();
        Sculpt3dSnapshot {
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
                wireframe: self.wireframe,
                cavity: self.cavity,
                light_az_deg: f32::from(light.angle_deg),
                light_elev_deg: f32::from(light.elev_deg),
                detail: detail_index(self.dyntopo.detail),
            },
            dyntopo: self.dyntopo.armed,
            level: self.level(),
            level_count: self.level_count(),
            pieces: self.objects.len(),
            isolated: self.isolated.is_some(),
            matcaps: ph2d_mesh_render::MATCAPS.as_slice(),
            verts: self.mesh().vert_count(),
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
        self.wireframe = ui.wireframe;
        self.cavity = ui.cavity;
        let l = self.rig.current_mut();
        // Graus INTEIROS é a unidade em que o rig é autorado — o `f32` existe só
        // porque a pista de um slider é contínua.
        l.angle_deg = (ui.light_az_deg.round().max(0.0) as u16) % 360;
        l.elev_deg =
            (ui.light_elev_deg.round().max(0.0) as u16).clamp(ph2d_light::MIN_ELEV_DEG, 90);
        self.dyntopo.detail = DETAIL_STEPS[(ui.detail as usize).min(DETAIL_STEPS.len() - 1)].0;
    }

    /// **Um gesto do painel.** Devolve `true` se ele mexeu na cena de um jeito
    /// que vale uma linha de log — os mesmos desfechos que as teclas imprimem,
    /// pela MESMA porta, porque um botão e um atalho que fizessem coisas
    /// diferentes seriam duas ferramentas com um nome só.
    pub(crate) fn apply_panel_intent(&mut self, intent: Sculpt3dIntent) {
        match intent {
            Sculpt3dIntent::SetUi(ui) => self.apply_ui(&ui),
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
            Sculpt3dIntent::MaskClear => self.mask_from_panel(MaskOp::Clear),
            Sculpt3dIntent::MaskInvert => self.mask_from_panel(MaskOp::Invert),
            Sculpt3dIntent::MaskBlur => self.mask_from_panel(MaskOp::Blur),
            Sculpt3dIntent::MaskSharpen => self.mask_from_panel(MaskOp::Sharpen),
        }
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
