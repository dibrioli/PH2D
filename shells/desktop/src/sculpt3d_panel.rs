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
use super::{MaskOp, Primitive, RemeshRefusal, Sculpt3dScene};

/// **O que o LAÇO DE FRAME tem de cumprir** por um gesto do painel.
///
/// ⚠️ **Ele existe porque a resposta deixou de ser binária.** Os dois pedidos
/// aqui têm a MESMA razão de não morarem na cena — o mundo 2D, o renderizador e
/// o mapa de atlas só existem dentro do frame —, e nomeá-los é o que impede o
/// terceiro de nascer como um segundo `bool` que alguém esquece de ler.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Sculpt3dFrameRequest {
    /// Acender o sprite selecionado com a forma esculpida (o `Shift+B`).
    Bake,
    /// Ler os pixels do sprite selecionado como PADRÃO do pincel.
    AlphaFromSprite,
}

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
            filter_armed: self.filter_arm(),
            ui: Sculpt3dUi {
                brush: self.brush.clone(),
                slots: self.verb_slots.clone(),
                ui_level: self.ui_level,
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
                // Contagem de células vira `f32` só para a pista; a fronteira de
                // volta (`apply_ui`) arredonda e clampa.
                remesh_res: self.remesh_res as f32,
            },
            dyntopo: self.dyntopo.armed,
            level: self.level(),
            level_count: self.level_count(),
            ao_stale: self.mesh().ao_is_stale(),
            alpha_image_name: self.alpha_image.as_ref().map(|(_, n)| n.clone()),
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

    /// **O SPRITE SELECIONADO VIRA O PADRÃO** — a porta única do alpha por
    /// imagem. Devolve a escala em vigor, que é o número que o readout reporta.
    ///
    /// ⚠️ **Ela SEMEIA a escala, e é a mesma lei do chip:** armar um padrão sem
    /// um tamanho que este modelo comporte é o defeito que um smoke já reprovou
    /// (*"os poros são gigantescos"*). O chip semeia pelo `alpha_seed` que o
    /// retrato publica; aqui a cena pergunta à MESMA `recommended_scale`, e a
    /// diferença é que ela tem a malha na mão em vez de um número que viajou.
    ///
    /// ⚠️ **SEMEAR não é IMPOR, e a versão anterior impunha.** Ela escrevia a
    /// escala INCONDICIONALMENTE, então todo aperto do botão descartava o
    /// `Alpha Scale` que o artista tinha autorado — o report *"as configurações
    /// da textura não foram obedecidas"* (Enio, 2026-08-09). A porta irmã (o
    /// chip, em `ph2d-panel-sculpt3d/src/event.rs`) sempre obedeceu ao
    /// SENTINELA, que é a razão de [`ph2d_sculpt3d::DEFAULT_ALPHA_SCALE`]
    /// existir — o doc dela diz, com todas as letras, que *"o trabalho real
    /// desta constante é ser sentinela: é comparando contra ela que o painel
    /// sabe «o artista ainda não escolheu»"*. Duas portas, uma lei; e o
    /// doc-comment desta AFIRMAVA a equivalência que não valia, que é o que fez
    /// a divergência passar.
    ///
    /// ⚠️ **E o EIXO é semeado junto, porque UM número respondia DUAS
    /// perguntas.** Para os nove padrões procedurais o eixo diz *para que lado
    /// os estratos empilham*, e o `elev = 0` de fábrica é o certo (o doc do
    /// `Brush::default` explica: com `elev = 90` o artista veria uma camada só).
    /// Para uma IMAGEM o eixo é a **direção de PROJEÇÃO** — o `Alpha::Image`
    /// lê `q[0]`/`q[1]`, o plano perpendicular a ele —, então o mesmo `elev = 0`
    /// projeta o carimbo **de lado**, e ele degenera em listras. Medido, num
    /// swatch de 32² com bandas diagonais:
    ///
    /// | eixo | linhas distintas | colunas distintas |
    /// |---|---|---|
    /// | `az 90 · elev 0` (fábrica) | **3** | 13 |
    /// | `az 90 · elev 90` | 27 | 29 |
    /// | `az 0 · elev 0` | 8 | **1** |
    ///
    /// A transposta (`az 0`) é a prova de que a direção segue o `t` do frame e
    /// não um acaso. Com o eixo encarando a vista (`+Z`, que é
    /// [`ph2d_sculpt3d::MAX_AXIS_ELEV_DEG`]) o `t` e o `b` ficam **os dois** no
    /// plano XY ⇒ o campo é constante ao longo de `z`, e é por isso que a mesma
    /// linha conserta o **preview do painel** e o **modelo**: a fatia `z = 0`
    /// que o swatch desenha passa a ser EXATAMENTE o que todo ponto da malha
    /// recebe. O swatch fica *verdadeiro* em vez de ganhar um caso especial.
    pub(crate) fn set_alpha_image(
        &mut self,
        img: ph2d_sculpt3d::AlphaImage,
        from: std::sync::Arc<str>,
    ) -> f32 {
        self.alpha_image = Some((std::sync::Arc::new(img), from));
        self.arm_stored_image()
    }

    /// **RE-ARMA a imagem que a cena lembra** — o chip do slot, e a segunda
    /// metade do [`Sculpt3dScene::set_alpha_image`].
    ///
    /// ⚠️ **Uma LEI, dois pedintes.** Trazer um sprite novo e voltar ao slot pelo
    /// chip têm de semear igual: se cada um tivesse a sua cópia da regra do
    /// sentinela, um deles ganharia um `if` no dia em que a regra mudasse e o
    /// outro não — e o sintoma seria *"pelo botão a escala é semeada, pelo chip
    /// não"*, invisível até alguém comparar os dois gestos.
    pub(crate) fn arm_stored_image(&mut self) -> f32 {
        let Some((img, _)) = self.alpha_image.clone() else {
            return self.brush.alpha_scale;
        };
        self.seed_alpha_placement();
        self.brush.alpha = Some(ph2d_sculpt3d::Alpha::Image(img));
        self.brush.alpha_scale
    }

    /// A metade que SEMEIA — ver o doc de [`Sculpt3dScene::set_alpha_image`].
    fn seed_alpha_placement(&mut self) {
        // ⚠️ **O que é «de fábrica» sai do PRÓPRIO `Brush::default`**, nunca de
        // literais repetidos aqui: uma segunda cópia dos ângulos passaria a
        // discordar dele no dia em que ele mudasse, e o sintoma seria uma
        // semeadura que deixa de disparar — em silêncio.
        let factory = ph2d_sculpt3d::Brush::default();
        // O `1e-6` é o do chip (`ph2d-panel-sculpt3d/src/event.rs`): a MESMA
        // pergunta — *o artista já escolheu?* — feita com a mesma tolerância.
        if (self.brush.alpha_scale - factory.alpha_scale).abs() < 1e-6 {
            self.brush.alpha_scale = ph2d_sculpt3d::recommended_scale(self.mesh());
        }
        if (self.brush.alpha_az_deg, self.brush.alpha_elev_deg)
            == (factory.alpha_az_deg, factory.alpha_elev_deg)
        {
            self.brush.alpha_elev_deg = ph2d_sculpt3d::MAX_AXIS_ELEV_DEG;
        }
    }

    /// **O estado autorado, aplicado.** O painel manda a struct INTEIRA a cada
    /// arrasto; escrever campo a campo aqui é o que mantém um intent só.
    fn apply_ui(&mut self, ui: &Sculpt3dUi) {
        self.brush = ui.brush.clone();
        self.verb_slots = ui.slots.clone();
        self.ui_level = ui.ui_level;
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
        // ⚠️ **Arredondado aqui, na fronteira**, e não guardado como `f32`: a
        // resolução é uma CONTAGEM de células, e a pista de um slider é contínua
        // só porque pistas são contínuas. O clamp usa a faixa da tabela de rows,
        // cujo teto é medido (o campo transiente a 768 come 88% do orçamento do
        // app inteiro).
        self.remesh_res = (ui.remesh_res.round().max(0.0) as u32).clamp(16, 512);
    }

    /// **Um gesto do painel**, traduzido para a cena — os mesmos desfechos que as
    /// teclas imprimem, pela MESMA porta, porque um botão e um atalho que
    /// fizessem coisas diferentes seriam duas ferramentas com um nome só.
    ///
    /// Devolve o **pedido que o FRAME tem de cumprir**, quando o gesto não é
    /// para a cena — ver [`Sculpt3dFrameRequest`].
    ///
    /// ⚠️ **Era um `bool` que significava *"quer bake"*, e ele não carregava um
    /// SEGUNDO pedido.** Quando o alpha por imagem chegou, a escolha era
    /// devolver dois booleanos (dois campos para uma pergunta que tem uma
    /// resposta por gesto) ou nomear a resposta. O enum é o que faz o terceiro
    /// pedido não caber num `if` esquecido.
    ///
    /// ⚠️ **O `match` continua exaustivo, e é isso que este retorno compra.** A
    /// alternativa era o bridge filtrar o intent antes de chegar aqui — e aí a
    /// tradução passaria a morar em dois lugares, com o segundo sendo uma
    /// cascata de `if` que apodrece calada no dia em que nascer o próximo verbo
    /// que o frame tem de executar.
    pub(crate) fn apply_panel_intent(
        &mut self,
        intent: Sculpt3dIntent,
    ) -> Option<Sculpt3dFrameRequest> {
        match intent {
            // ⚠️ Ele ARMA e sai, e não faz nada com a cena: o bake precisa do
            // mundo, do renderizador e do mapa de atlas, e os três só existem
            // dentro do frame. Mesmo desenho do `Shift+B`.
            Sculpt3dIntent::BakeToSprite => return Some(Sculpt3dFrameRequest::Bake),
            // ⚠️ Mesmo desenho, e pela mesma razão: ler os pixels de um sprite
            // precisa do mundo, do renderizador e do mapa de atlas.
            Sculpt3dIntent::AlphaFromSprite => {
                return Some(Sculpt3dFrameRequest::AlphaFromSprite);
            }
            // ⚠️ **Este NÃO precisa do frame**, ao contrário dos dois acima: a
            // imagem já está aqui: o artista a trouxe uma vez e a cena a
            // segurou. Re-armar é escolher de novo o que já foi lido.
            Sculpt3dIntent::ArmStoredImage => {
                self.arm_stored_image();
            }
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
            Sculpt3dIntent::ArmFilter => {
                let on = self.arm_filter();
                eprintln!(
                    "[sculpt3d] filtro: {} -- {}",
                    self.brush.verb.label(),
                    if on {
                        "ARMADO -- arraste na horizontal para dar a forca (clique de novo para desarmar)"
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
            Sculpt3dIntent::Flatten => match self.flatten() {
                Some(n) => eprintln!(
                    "[sculpt3d] achatada: {n} niveis -- {} vertices / {} faces",
                    self.mesh().vert_count(),
                    self.mesh().face_count()
                ),
                None => eprintln!("[sculpt3d] nada a achatar: a pilha ja' tem um nivel so'"),
            },
            Sculpt3dIntent::Remesh => match self.remesh(self.remesh_res) {
                Ok(r) => eprintln!(
                    "[sculpt3d] reconstruida: {} -> {} vertices / {} -> {} faces",
                    r.verts.0, r.verts.1, r.faces.0, r.faces.1
                ),
                Err(RemeshRefusal::MultiresStack) => {
                    eprintln!("[sculpt3d] nao' reconstroi com a pilha montada: ACHATE antes")
                }
                Err(RemeshRefusal::EmptyScene) => {
                    eprintln!("[sculpt3d] nao' reconstroi: nao ha' peca na cena")
                }
                // ⚠️ A escultura CONTINUA na tela — ver o irmão no `sculpt3d_keys`.
                Err(RemeshRefusal::Engine(e)) => eprintln!(
                    "[sculpt3d] nao' reconstroi, e a escultura fica como esta': {e} -- tente outra resolucao"
                ),
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
                    eprintln!("[sculpt3d] nao' funde com a pilha montada: ACHATE a pilha antes")
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
        None
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

#[cfg(test)]
#[path = "sculpt3d_panel_tests.rs"]
mod tests;
