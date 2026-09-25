//! ⭐⭐⭐ **O PAINTER PINTA A PEÇA** — a costura entre o módulo Painter e a
//! escultura (ordem do dono, 2026-09-24: *«a integração total do módulo Painter
//! já existente para que consiga pintar com os mesmos features na malha 3d»*).
//!
//! # A corrente, e quem é dono de cada elo
//!
//! 1. **O Painter pinta uma tela do tamanho da vista** — o motor dele inteiro,
//!    sem uma linha nova (`ph2d_tool_painter::PainterTool::bind_screen_canvas`).
//! 2. **A ponta de cada gesto** chega aqui por [`entrega`], em coordenadas de
//!    janela, e sai para o Painter em coordenadas da tela (a vista activa).
//! 3. **A cada quadro e no fim do traço**, o que a tela mudou é pousado na peça
//!    ([`quadro`], [`ph2d_sculpt3d::tela_na_malha`]) — só onde a superfície está à
//!    vista, com a máscara, na tinta fina quando ela está armada.
//! 4. **O traço fecha pela porta de sempre** (`close_stroke`), logo o `Ctrl+Z`
//!    desfaz uma pincelada do Painter como desfaz uma do pincel de pintura.
//!
//! # ⚠️ A tela limpa-se a cada traço — menos com o papel MOLHADO
//!
//! A peça guarda a tinta; a tela é só o traço em voo. Deixá-la cheia faria o
//! traço seguinte compor o anterior outra vez por cima dele.
//!
//! ⭐⭐ **A excepção é a aquarela molhada** (report do dono, 24/09: *«Watercolor
//! não fica molhado»*). Limpar a tela passa pelo `set_source` do Painter, e ele
//! SECA o papel — logo cada traço nascia sobre papel seco e nunca fundia com o
//! anterior. Com o papel molhado no pen-up a tela FICA, e o traço seguinte
//! reaproveita-a se ela ainda descreve a peça ([`TelaMolhada`]).
//!
//! # ⚠️ Os limites, ditos
//!
//! Pinta-se o lado que se VÊ (o de trás espera que o artista rode a peça). A
//! pintura simples começa numa tela TRANSPARENTE; os modos que lêem a cor
//! debaixo do pincel começam com o RETRATO da peça ([`Sculpt3dScene::painter_semeia`],
//! etapa 2). E rodar a vista SECA a aquarela: a humidade vive nos píxeis do
//! ecrã, não na superfície.

use std::sync::Arc;

use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_sculpt3d::tela_na_malha::{Tela, TelaNaMalha, Vista};
use ph2d_tool_painter::{PainterTool, ScreenCanvasFrame};

use crate::Sculpt3dScene;
use crate::objects::ObjectId;

/// ⭐⭐ **A tela da aquarela que ficou MOLHADA depois de um traço** — o que a
/// peça recebeu, e a chave que diz se a tela ainda o descreve.
///
/// O traço seguinte só a reaproveita com a chave INTEIRA igual: a mesma vista
/// (câmera, tamanho, pose), a mesma peça, e nada mais a mexer nela
/// ([`Sculpt3dScene::edits`]). Qualquer diferença e o traço volta ao retrato fresco, que seca o papel:
/// ⚠️ **rodar a vista seca a aquarela**, porque a humidade vive nos píxeis do
/// ecrã e não na superfície.
///
/// ⭐ O `retrato` é a SEMENTE do traço seguinte — a lei da diferença pede
/// `c − s` com `s` = o que a peça já tem, e o que ela tem é esta tela.
///
/// ⛔ **Um contador do HISTÓRICO ao lado do `edits` foi construído e RETIRADO
/// por prova de mutação:** o desfazer e um traço de outro pincel na tinta fina
/// — as duas coisas que ele existia para ver — já sobem o `edits` (o fecho de
/// um traço de tinta fina devolve a cor grossa), e apagá-lo nas duas portas
/// deixou o gate de produto verde. *Uma linha que a mutação não consegue matar
/// não é lei, é comentário com sintaxe de código.*
pub(crate) struct TelaMolhada {
    vista: Vista,
    objeto: ObjectId,
    edits: u64,
    retrato: Arc<Vec<u8>>,
}

impl TelaMolhada {
    /// A tela ainda descreve a peça como ela está AGORA?
    fn serve(&self, vista: &Vista, objeto: ObjectId, edits: u64) -> bool {
        self.vista == *vista && self.objeto == objeto && self.edits == edits
    }
}

/// ⭐ **Um quadro da costura** — prende (ou solta) a tela do Painter conforme o
/// barro está ou não no ecrã, e pousa na peça o que a tela mudou.
///
/// `painter` é `None` quando a ferramenta em mãos não é o Painter: aí um traço
/// aberto fecha, porque ninguém o vai terminar.
pub fn quadro(scene: Option<&mut Sculpt3dScene>, painter: Option<&mut PainterTool>) {
    let Some(painter) = painter else {
        if let Some(s) = scene {
            s.painter_fecha();
            s.painter_raio_px = None;
        }
        return;
    };
    match scene {
        Some(s) if s.tela_do_painter().is_some() => {
            // ⭐ Outra mão mexeu na peça com a tinta a escorrer: o traço fecha
            // ANTES de pousar, senão a água escreveria sobre a mudança dela.
            s.painter_escorre_se_a_peca_mudou();
            // Pousa ANTES de redimensionar: a tinta em voo é da tela de antes.
            let drenou = painter.take_screen_canvas().map(|f| s.painter_pousa(&f));
            // ⭐⭐ A água parou E a última drenagem já chegou: a pincelada acaba.
            if s.painter_escorre.is_some()
                && drenou.is_none()
                && !painter.screen_canvas_is_flowing()
            {
                termina(s, painter);
            }
            // Uma tela que nasce (ou muda de tamanho) é transparente: a molhada
            // que estava guardada deixou de estar nela.
            if let Some((w, h)) = s.tela_do_painter()
                && painter.bind_screen_canvas(w, h)
            {
                s.painter_molhada = None;
                s.painter_fecha();
            }
            s.painter_raio_px = Some(painter.screen_canvas_ring_px());
        }
        Some(s) => {
            s.painter_fecha();
            s.painter_raio_px = None;
            s.painter_molhada = None;
            painter.release_screen_canvas();
        }
        None => painter.release_screen_canvas(),
    }
}

/// ⭐ **Um ponto do gesto**, em coordenadas de JANELA. Devolve se o Painter o
/// consumiu (para o gesto não descer ao pan nem à selecção).
///
/// ⚠️ Só arma com a tela presa ([`PainterTool::on_screen_canvas`]); fora disso
/// quem responde é a ponte da sprite.
pub fn entrega(
    scene: &mut Sculpt3dScene,
    painter: &mut PainterTool,
    x: f32,
    y: f32,
    pressure: f32,
    phase: PointerPhase,
) -> bool {
    if !painter.on_screen_canvas() {
        return false;
    }
    let (vx, vy) = scene.to_view(x, y);
    if phase == PointerPhase::Down {
        let (w, h) = scene.viewport();
        if vx < 0.0 || vy < 0.0 || vx >= w as f32 || vy >= h as f32 {
            return false;
        }
        // ⭐⭐ **A pincelada anterior ainda ESCORRE:** ela acaba pela MESMA porta
        // de quando a água pára — pousa o que falta e guarda a tela molhada —,
        // e o traço novo reaproveita-a. Report do dono (24/09): *«ao usar a
        // segunda cor, a primeira cor ainda seca e para»* — o `painter_abre`
        // fechava sem guardar, o traço novo re-semeava a tela, e semear a tela
        // MATA a sessão da água (o `set_source` troca o `canvas_rgba`). Daqui em
        // diante o que a água da 1.ª escorre é pousado no traço novo.
        if scene.painter_escorre.is_some() {
            if let Some(f) = painter.take_screen_canvas() {
                scene.painter_pousa(&f);
            }
            termina(scene, painter);
        }
        if !scene.painter_abre(x, y) {
            return false;
        }
        // ⭐⭐ Os modos que lêem a cor debaixo do pincel começam com o RETRATO
        // da peça na tela, e a lei passa a ser a diferença (etapa 2).
        if painter.screen_canvas_reads_the_piece() {
            let _ = scene.painter_semeia(painter);
        } else {
            scene.painter_limpa(painter);
        }
    }
    let consumed = painter.on_canvas_pointer(CanvasPointer {
        pos: [vx, vy],
        pressure,
        tilt: [0.0, 0.0],
        phase,
    });
    if phase == PointerPhase::Up {
        if let Some(f) = painter.take_screen_canvas() {
            scene.painter_pousa(&f);
        }
        // ⭐⭐ **A tinta molhada ainda escorre:** o traço FICA aberto, e cada
        // quadro continua a pousar o que a água muda ([`quadro`]). Um só passo
        // de desfazer para o traço e para o que ele escorreu.
        if scene.painter_tela.is_some() && painter.screen_canvas_is_flowing() {
            scene.painter_escorre = Some(scene.edits);
        } else {
            termina(scene, painter);
        }
    }
    consumed
}

/// ⭐ **O fim de uma pincelada do Painter** — no pen-up, ou quando a água que ela
/// deixou a escorrer parou. Fecha o traço, e decide se a tela FICA (papel
/// molhado) ou se limpa.
fn termina(scene: &mut Sculpt3dScene, painter: &mut PainterTool) {
    // A vista ANTES do fecho: é ele que larga a sessão.
    let vista = scene.painter_tela.as_ref().map(|s| *s.vista());
    let semeado = scene.painter_tela.as_ref().is_some_and(|s| s.tem_semente());
    scene.painter_fecha();
    let ultima = scene.painter_ultima.take();
    // ⭐⭐ Papel molhado ⇒ a tela FICA: limpá-la secava-o.
    match (vista, ultima) {
        (Some(vista), Some(retrato)) if semeado && painter.screen_canvas_is_wet() => {
            scene.painter_guarda(vista, retrato);
        }
        _ => {
            painter.clear_screen_canvas();
        }
    }
}

impl Sculpt3dScene {
    /// O tamanho da tela que o Painter deve segurar — a vista activa —, ou
    /// `None` quando não há barro no ecrã a receber tinta.
    pub(crate) fn tela_do_painter(&self) -> Option<(u32, u32)> {
        (self.clay_on_screen() && self.obj().is_some()).then(|| self.viewport())
    }

    /// A vista da peça ACTIVA, de espaço local para píxeis da tela.
    fn vista_do_painter(&self) -> Option<Vista> {
        let o = self.obj()?;
        let (w, h) = self.viewport();
        // ⚠️ A pose é translação + escala UNIFORME (`ph2d_mesh::Pose`), logo
        // `VP · modelo` fecha-se coluna a coluna sem uma matriz de modelo.
        let s = o.pose.scale();
        let t = o.pose.translation;
        let vp = self
            .camera
            .view_proj(w as f32 / h.max(1) as f32)
            .to_cols_array();
        let mut m = [0.0f32; 16];
        for r in 0..4 {
            for c in 0..3 {
                m[4 * c + r] = vp[4 * c + r] * s;
            }
            m[12 + r] = vp[r] * t[0] + vp[4 + r] * t[1] + vp[8 + r] * t[2] + vp[12 + r];
        }
        let olho = o.pose.point_to_local(self.camera.eye().into());
        Some(Vista::nova(m, (w, h), olho))
    }

    /// **Abre a pincelada do Painter na peça** — a mesma abertura de um traço de
    /// pintura: mira a peça sob o cursor, congela o traço e empresta o plano de
    /// tinta fina. Devolve `false` sem peça nenhuma.
    pub(crate) fn painter_abre(&mut self, x: f32, y: f32) -> bool {
        self.painter_fecha();
        // ⚠️ Errar a peça NÃO recusa o traço: o pincel pinta a tela, e a tinta
        // aterra onde a peça estiver quando o traço lá chegar (a lei do pincel
        // de cor desta casa, 21/09). A mira só muda a peça activa quando acerta.
        self.aim(x, y);
        // A vista ANTES do empréstimo: um `return` depois dele deixaria o plano
        // preso num traço que ninguém fecha.
        let Some(vista) = self.vista_do_painter() else {
            return false;
        };
        self.stroke.begin(self.objects[self.active].stack.mesh());
        let dono = self.objects[self.active].id;
        self.stroke.tinta_fina =
            crate::tinta_da_peca::empresta(&mut self.objects[self.active].tinta, dono);
        let mesh = self.objects[self.active].stack.mesh();
        let destino = self
            .stroke
            .tinta_fina
            .as_ref()
            .map_or(mesh.vert_count(), |t| t.tinta().amostras().len());
        self.painter_tela = Some(TelaNaMalha::nova(mesh, vista, destino));
        true
    }

    /// ⭐⭐ **A tela começa com o retrato da peça** — os MESMOS bytes vão para
    /// o Painter e para a sessão, senão o que o pincel não tocou deixa de se
    /// anular na diferença. Devolve `true` quando REAPROVEITOU a tela molhada
    /// do traço anterior em vez de semear um retrato novo.
    ///
    /// ⚠️ **E a drenagem do retrato deita-se FORA:** semear a tela marca-a
    /// inteira como mudada, e o retrato não é uma mudança — pousá-lo varreria a
    /// peça inteira no quadro seguinte para concluir que nada mudou.
    pub(crate) fn painter_semeia(&mut self, painter: &mut PainterTool) -> bool {
        let guardada = self.painter_molhada.take();
        let chave = self.obj().map(|o| o.id);
        let edits = self.edits;
        let Some(sessao) = self.painter_tela.as_mut() else {
            return false;
        };
        // ⭐⭐ O papel ainda molhado e a tela ainda a descrever a peça: a tela
        // FICA como está (semeá-la secava-o) e a semente é o que a peça recebeu.
        if let (Some(g), Some(objeto)) = (guardada, chave)
            && painter.screen_canvas_is_wet()
            && g.serve(sessao.vista(), objeto, edits)
        {
            sessao.com_semente(g.retrato.as_ref().clone());
            self.painter_ultima = Some(g.retrato);
            return true;
        }
        let tinta = self.stroke.tinta_fina.as_ref().map(|t| t.tinta());
        let mesh = self.objects[self.active].stack.mesh();
        let retrato = ph2d_sculpt3d::tela_semente::semente(mesh, tinta, sessao.vista());
        if painter.seed_screen_canvas(retrato.clone()) {
            let _ = painter.take_screen_canvas();
            sessao.com_semente(retrato.clone());
            self.painter_ultima = Some(Arc::new(retrato));
        }
        false
    }

    /// **A pintura simples começa numa tela TRANSPARENTE** — e ela só não o
    /// está quando um traço anterior a deixou molhada ([`TelaMolhada`]). A
    /// drenagem da limpeza deita-se fora, como a do retrato: ela não é uma
    /// mudança da peça.
    pub(crate) fn painter_limpa(&mut self, painter: &mut PainterTool) {
        if self.painter_molhada.take().is_some() {
            painter.clear_screen_canvas();
            let _ = painter.take_screen_canvas();
        }
    }

    /// **Guarda a tela molhada** no pen-up — com a chave lida DEPOIS do fecho,
    /// que é quem devolve a cor grossa e sobe o `edits`.
    fn painter_guarda(&mut self, vista: Vista, retrato: Arc<Vec<u8>>) {
        self.painter_molhada = self.obj().map(|o| TelaMolhada {
            vista,
            objeto: o.id,
            edits: self.edits,
            retrato,
        });
    }

    /// **Pousa na peça o que a tela mudou.**
    pub(crate) fn painter_pousa(&mut self, f: &ScreenCanvasFrame) {
        let Some(sessao) = self.painter_tela.as_mut() else {
            return;
        };
        let rect = f.rect.map_or([0, 0, f.w, f.h], |(x, y, w, h)| [x, y, w, h]);
        let tela = Tela {
            rgba: &f.rgba,
            largura: f.w,
            altura: f.h,
        };
        let active = self.active;
        let Some(o) = self.objects.get_mut(active) else {
            return;
        };
        let (vertices, _) = self
            .stroke
            .pousa_a_tela(o.stack.mesh_mut(), sessao, &tela, rect);
        self.painter_ultima = Some(Arc::clone(&f.rgba));
        if !vertices.is_empty() {
            Self::mesh_changed(&mut o.dirty, &mut self.edits, &vertices);
        }
        // O pouso é DESTE traço: a mudança do `edits` não é de outra mão.
        if self.painter_escorre.is_some() {
            self.painter_escorre = Some(self.edits);
        }
    }

    /// **Fecha a pincelada** — o desfazer é gravado pela porta de todo traço.
    pub(crate) fn painter_fecha(&mut self) {
        self.painter_escorre = None;
        if self.painter_tela.take().is_some() {
            self.close_stroke();
        }
    }

    /// ⭐⭐ **Fecha a pincelada que ESCORRE** — e só essa: um traço com o dedo
    /// em baixo continua do gesto que o abriu. Chamada por toda porta que mexe
    /// na peça (o desfazer, as teclas, o painel, um clique da escultura), porque
    /// a pincelada que escorre pode durar `~20 s` e o artista não espera por ela.
    ///
    /// ⚠️ A tela do Painter fica como está e a água continua a correr NELA; a
    /// peça é que deixa de a receber. O traço seguinte volta ao retrato fresco.
    pub(crate) fn painter_fecha_o_que_escorre(&mut self) {
        if self.painter_escorre.is_some() {
            self.painter_fecha();
            self.painter_molhada = None;
        }
    }

    /// A rede de segurança das portas acima: o `edits` mudou desde o último
    /// pouso DESTE traço ⇒ outra mão mexeu na peça, e o traço fecha.
    fn painter_escorre_se_a_peca_mudou(&mut self) {
        if self.painter_escorre.is_some_and(|e| e != self.edits) {
            self.painter_fecha_o_que_escorre();
        }
    }
}

/// ⭐⭐ **Este gesto do painel pode mexer na PEÇA?** — a pergunta que decide se
/// a pincelada do Painter que ainda escorre fecha antes dele.
///
/// Report do dono (24/09, sobre a etapa 3a): *«a simulação seca (para) ao trocar
/// a cor do pincel»*. Trocar a cor na caixa do painel é um `SetUi`, e a 1.ª
/// redacção fechava a pincelada em TODO intent. ⇒ o `SetUi` é um AJUSTE: o
/// `apply_ui` só copia números para a cena (pincel, luz, vista, os alvos dos
/// botões) e não escreve um vértice, uma face nem o plano — logo não há nada
/// na peça que a água possa escrever por cima.
///
/// ⚠️ **O `match` é EXAUSTIVO de propósito, sem `_`:** um intent novo é erro de
/// compilação aqui até alguém dizer se ele é um gesto ou um ajuste. Na dúvida
/// ele é um GESTO (`true`) — fechar cedo perde o resto do escorrido; não fechar
/// deixaria a água escrever sobre uma peça que mudou por baixo dela.
pub(crate) fn o_painel_mexe_na_peca(intent: &ph2d_panel_sculpt3d::Sculpt3dIntent) -> bool {
    use ph2d_panel_sculpt3d::Sculpt3dIntent as I;
    match intent {
        I::SetUi(_) => false,
        // ⚠️ Chegou na fusão com a UIUX (2026-09-25): ele só ABRE o catálogo de pincéis — escolher
        // é o *pick* da paleta, noutro quadro, e esse chega como `SetUi`. Não escreve na peça.
        I::OpenBrushPalette => false,
        I::ArmTransform(_)
        | I::ArmFilter
        | I::ArmStoredImage
        | I::ToggleDyntopo
        | I::ChangeLevel(_)
        | I::Subdivide
        | I::ReverseLevel
        | I::Flatten
        | I::Remesh
        | I::QuadRemesh
        | I::CloseHoles
        | I::SetClothPersistentBase
        | I::BakeAo
        | I::BakeToSprite
        | I::AlphaFromSprite
        | I::AddSphere
        | I::AddCube
        | I::AddCylinder
        | I::AddTorus
        | I::Duplicate
        | I::Delete
        | I::ToggleIsolate
        | I::Merge
        | I::MaskClear
        | I::MaskInvert
        | I::MaskBlur
        | I::MaskSharpen
        | I::ColorFill
        | I::Extract => true,
    }
}
