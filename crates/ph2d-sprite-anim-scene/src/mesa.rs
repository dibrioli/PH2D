//! **O executor do guião** da cena 5: cada [`Passo`] pelas portas do produto, que a shell dá pela [`Mesa`]. A shell só
//! compõe: diz se a cena está armada (`ph2d_app_game::smoke::armed_level`) e responde às portas.

use crate::{self as a, Passo as P};
pub use ph2d_anim_editor::doc::AnimDoc;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::object_mode::{ModeRequest, ObjectMode};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{AnimFieldEdit, ImageUseEdit, NodeId};

/// Um gesto do rato pelas portas da shell (`smoke_pointer_*`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rato {
    Abaixo(f32, f32),
    Move(f32, f32),
    Acima,
}

/// Os cartões do Inspector que as fotos abrem ou dobram: os ids são do painel, que esta folha não vê.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cartao {
    /// «More» do cartão Animation.
    Mais,
    /// O Look do Animated Sprite (o campo Pivot).
    Aparencia,
    Imagem,
    Animacao,
}

/// As portas da shell. Cada uma responde com um VALOR, como o `ph2d_app_host::AppHost`: a folha não vê a `App`.
pub trait Mesa {
    /// O mundo da cena; `None` antes da placa.
    fn mundo(&mut self) -> Option<&mut SimWorld>;
    /// O objecto da raiz com este nome.
    fn por_nome(&mut self, nome: &str) -> Option<u64>;
    /// Uma ação no bus do herói (sem herói, nada).
    fn bus(&mut self, acao: EditorAction);
    fn uso_da_imagem(&mut self, bits: u64, edit: ImageUseEdit);
    fn escolhe_imagem(&mut self, bits: u64, imagem: [u8; 32]);
    /// As fileiras da folha visível, como o «Split by rows» as lê.
    fn fileiras(&mut self, bits: u64) -> Vec<Vec<u32>>;
    fn rato(&mut self, gesto: Rato);
    /// O centro de um widget pintado neste quadro.
    fn widget(&self, id: NodeId) -> Option<(f32, f32)>;
    /// O Ctrl seguro (ou solto) pela porta do teclado real: o `sync_touch_modifiers` relê-a em todo quadro.
    fn ctrl(&mut self, seguro: bool);
    /// Um ponto do mundo no ecrã, pela câmara e pela janela da cena.
    fn ao_ecra(&self, ponto: [f32; 2]) -> Option<(f32, f32)>;
    /// O documento aberto do editor de animações de `bits`.
    fn editor<R>(&mut self, bits: u64, ler: impl FnOnce(&AnimDoc) -> R) -> Option<R>;
    /// A câmara: centro e altura do mundo à vista.
    fn camara(&mut self, vista: ([f32; 2], f32));
    fn seleciona(&mut self, bits: Option<u64>);
    fn selecionado(&self) -> Option<u64>;
    /// Preferences ▸ Image Filter ▸ Pixel Art.
    fn filtro_pixel(&mut self);
    fn cartao(&mut self, cartao: Cartao, dobrado: bool);
    /// O painel da direita pedido a zero: cai no piso dele, `DOCK_W_MIN`.
    fn painel_estreito(&mut self);
    fn desfaz(&mut self);
    /// Este quadro conta como entrada do artista.
    fn entrada(&mut self);
    /// A regra acesa do Player, com o relógio da timeline.
    fn regra_acesa(&mut self, bits: u64) -> Option<u8>;
}

/// Um por quadro, na fase das cenas, com a cena armada.
pub fn quadro(m: &mut impl Mesa) {
    let Some(q) = a::proximo_quadro() else {
        return;
    };
    for p in a::passos(q, a::foto()) {
        passo(m, p);
    }
}

fn passo(m: &mut impl Mesa, p: P) {
    let nome = match &p {
        P::Anim(n, _)
        | P::Verbo(n, _)
        | P::Heroi(n)
        | P::Imagem(n, _)
        | P::Folha(n, _)
        | P::Ferramenta(n, _)
        | P::Corta(n)
        | P::Ouve(n)
        | P::Edita(n)
        | P::Pivo(n, _)
        | P::Alinha(n, _)
        | P::Relata(_, n) => n,
        P::Foto(f) => f.map_or(a::PLAYER, a::Foto::alvo),
        P::RelataRegra => a::PLAYER,
        P::Entrada | P::Desfaz => "",
    };
    let bits = m.por_nome(nome);
    let anim = |entity_bits, edit| EditorAction::InspectorAnimEdit { entity_bits, edit };
    match (p, bits) {
        (P::Anim(_, edit), Some(b)) => m.bus(anim(b, edit)),
        (P::Verbo(_, v), Some(b)) => m.uso_da_imagem(b, v),
        (P::Ferramenta(_, tool_id), Some(entity_bits)) => m.bus(EditorAction::OneShotImageOp {
            tool_id,
            entity_bits,
        }),
        (P::Heroi(_), Some(b)) => {
            if let Some(sim) = m.mundo()
                && let Err(motivo) =
                    ph2d_render::veste_o_heroi(sim.world_mut(), Entity::from_bits(b))
            {
                eprintln!("[animado] o herói não entrou: {motivo}");
            }
        }
        (P::Imagem(_, i), Some(b)) => {
            if let Some(id) = a::imagem(i) {
                m.escolhe_imagem(b, id);
            }
        }
        (P::Folha(_, i), Some(b)) => {
            if let Some(id) = a::imagem(i) {
                m.uso_da_imagem(b, ImageUseEdit::AddSheet(id));
            }
        }
        (P::Corta(_), Some(b)) => {
            let rows = m.fileiras(b);
            m.bus(anim(b, AnimFieldEdit::SplitByRows(rows)));
        }
        (P::Ouve(_), Some(b)) => {
            if let Some(sim) = m.mundo() {
                a::ouve_o_marcador(sim, Entity::from_bits(b));
            }
        }
        (P::Foto(f), b) => foto(m, f, b),
        (P::Edita(_), Some(_)) => m.bus(EditorAction::ObjectMode(ModeRequest::Enter(
            ObjectMode::Edit,
        ))),
        (P::Pivo(_, fase), Some(b)) => pivo(m, fase, b),
        (P::Alinha(_, fase), Some(b)) => alinha(m, fase, b),
        (P::Entrada, _) => m.entrada(),
        (P::Desfaz, _) => m.desfaz(),
        (P::Relata(t, n), Some(b)) => {
            if let Some(sim) = m.mundo() {
                let e = Entity::from_bits(b);
                let folha0 = sim
                    .world()
                    .get::<ph2d_render::Sprite>(e)
                    .map(|s| (s.size, s.anchor));
                eprintln!(
                    "[animado-prova] {t}: {} folha0={folha0:?}",
                    a::relato(sim, n, e)
                );
            }
        }
        (P::RelataRegra, Some(b)) => {
            let acesa = m.regra_acesa(b);
            eprintln!("[animado-prova] regra acesa do Player: {acesa:?}");
        }
        _ => {}
    }
}

/// O gesto do Pivot com Ctrl pelas portas do rato, uma fase por quadro: um clique humano tem o botão abaixo e acima em
/// quadros separados.
fn pivo(m: &mut impl Mesa, fase: u8, bits: u64) {
    match fase {
        0 => {
            if let Some((x, y)) = achar(m, ph2d_editor_core::ids::TOOL_PIVOT, "o botão Pivot") {
                m.rato(Rato::Abaixo(x, y));
            }
        }
        1 => m.rato(Rato::Acima),
        2 => {
            eprintln!("[animado-prova] pivô antes: {}", relato_do_pivo(m, bits));
            let gizmo = ph2d_editor_core::gizmo::ids::GIZMO_PIVOT;
            if let Some((x, y)) = achar(m, gizmo, "o ponto do pivô") {
                m.ctrl(true);
                m.rato(Rato::Abaixo(x, y));
            }
        }
        3 => {
            let c = m
                .mundo()
                .and_then(|s| centro_do_quad(s.world(), Entity::from_bits(bits)));
            if let Some((x, y)) = c.and_then(|c| m.ao_ecra(c)) {
                m.rato(Rato::Move(x + 5.0, y + 4.0));
            }
        }
        _ => {
            m.rato(Rato::Acima);
            m.ctrl(false);
            eprintln!("[animado-prova] pivô depois: {}", relato_do_pivo(m, bits));
        }
    }
}

fn achar(m: &impl Mesa, id: NodeId, o_que: &str) -> Option<(f32, f32)> {
    let p = m.widget(id);
    if p.is_none() {
        eprintln!("[animado-prova] pivô: {o_que} não está na tela");
    }
    p
}

type Pega = Option<((f32, f32), (f32, f32))>;

/// A pega do canto e o Pivot da fase `2`, para o caminho das fases seguintes.
static PEGA: std::sync::Mutex<Pega> = std::sync::Mutex::new(None);

/// A prévia Align pelas portas do rato, uma fase por quadro: `0`/`1` o botão Align; `2` a pega do canto, `3..=6` o
/// caminho até `2/3` da distância ao Pivot, `7` soltar.
fn alinha(m: &mut impl Mesa, fase: u8, bits: u64) {
    let doc = m.editor(bits, |d| {
        let l = &d.layout.align;
        let alvos = (l.pivot_grid, l.by_hand, l.pivot_follow, l.preview);
        (l.toggle, l.by_feet, d.align.painted, alvos)
    });
    let Some((toggle, pes, p, (grade, mao, segue, previa))) = doc else {
        return eprintln!("[animado-prova] align: o editor não está aberto");
    };
    let c = |r: Rect| (r.x + r.w / 2.0, r.y + r.h / 2.0);
    let mut pega = PEGA
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let abaixo = |(x, y): (f32, f32)| Rato::Abaixo(x, y);
    match (fase, *pega) {
        (0, _) => m.rato(abaixo(c(toggle))),
        (8, _) => m.rato(abaixo(c(pes))),
        (1 | 7 | 9 | 11 | 13 | 17 | 19, _) => m.rato(Rato::Acima),
        (18, _) => m.rato(abaixo(c(segue))),
        // O Pivot (GEF 29): «Bottom» é a casa do meio da última fila da grade 5 × 5.
        (10, _) => m.rato(Rato::Abaixo(
            grade.x + grade.w / 2.0,
            grade.y + grade.h * 0.9,
        )),
        (12, _) => m.rato(abaixo(c(mao))),
        // A folha arrastada para o lado, o soltar noutro quadro (o Follow, smoke do dono 09/11).
        (20, _) => m.rato(abaixo(c(previa))),
        (21, _) => m.rato(Rato::Move(c(previa).0 + previa.w * 0.2, c(previa).1)),
        (22, _) => m.rato(Rato::Acima),
        (14..=16, _) => {
            let (Some((ox, oy)), Some(r)) = (p.pivot, p.all_frames) else {
                return eprintln!("[animado-prova] pivô: sem cruz ou sem caixa");
            };
            let s = f32::from(fase - 14) / 2.0;
            let alvo = (r.x + r.w, r.y + r.h / 2.0);
            let at = (ox + (alvo.0 - ox) * s, oy + (alvo.1 - oy) * s);
            m.rato(if fase == 14 {
                abaixo(at)
            } else {
                Rato::Move(at.0, at.1)
            });
        }
        (2, _) => {
            *pega = p.handles[3].map(c).zip(p.pivot);
            eprintln!(
                "[animado-prova] align: pega={:?} pivô={:?}",
                p.handles[3], p.pivot
            );
            if let Some((h, _)) = *pega {
                m.rato(abaixo(h));
            }
        }
        (_, Some(((hx, hy), (ox, oy)))) => {
            let s = 1.0 - (1.0 - a::ESCALA_DA_RUN) * f32::from(fase - 2) / 4.0;
            m.rato(Rato::Move(ox + (hx - ox) * s, oy + (hy - oy) * s));
        }
        _ => {}
    }
}

/// Selecionado? · a origem · a folha visível e o Pivot dela · o centro do quad dela no mundo.
fn relato_do_pivo(m: &mut impl Mesa, bits: u64) -> String {
    let sel = m.selecionado() == Some(bits);
    let Some(sim) = m.mundo() else {
        return String::new();
    };
    let (w, e) = (sim.world(), Entity::from_bits(bits));
    let t = w.get::<ph2d_ecs::Transform>(e).map(|t| t.translation);
    let pivo = ph2d_sprite_edit::pivot::sprite_do_pivot(w, e).map(|(k, s)| (k, s.anchor));
    let c = centro_do_quad(w, e);
    format!("selecionado={sel} origem={t:?} folha_e_pivo={pivo:?} desenho={c:?}")
}

fn foto(m: &mut impl Mesa, foto: Option<a::Foto>, bits: Option<u64>) {
    m.camara(foto.map_or(a::VISTA, a::Foto::vista));
    m.seleciona(bits);
    // `PH2D_GAME_SMOKE_FILTRO=pixel`: as fotos do filtro do app (Preferences ▸ Image Filter ▸ Pixel Art).
    if std::env::var("PH2D_GAME_SMOKE_FILTRO").is_ok_and(|v| v.trim() == "pixel") {
        m.filtro_pixel();
    }
    let abre = |entity_bits, edit| EditorAction::InspectorAnimEdit { entity_bits, edit };
    match (foto, bits) {
        (Some(a::Foto::More), _) => m.cartao(Cartao::Mais, false),
        (Some(a::Foto::Estreito), _) => m.painel_estreito(),
        (
            Some(
                a::Foto::Editor
                | a::Foto::FolhasEditor
                | a::Foto::FolhasApagada
                | a::Foto::Alinhar
                | a::Foto::AlinharEscala
                | a::Foto::AlinharDesfaz
                | a::Foto::AlinharPes
                | a::Foto::AlinharPesDesfaz
                | a::Foto::Desalinhadas
                | a::Foto::DesalinhadasAcertadas,
            ),
            Some(b),
        ) => m.bus(abre(b, AnimFieldEdit::OpenEditor)),
        // O Pivot (GEF 29): pelo campo Pivot do Look aberto, que abre o editor com o Align ligado.
        (Some(f), Some(b)) if f.pivo_no_editor() => {
            m.cartao(Cartao::Aparencia, false);
            // Na 1.ª foto o campo Pivot tem de se ver: Image e Animation dobrados.
            for c in [Cartao::Imagem, Cartao::Animacao] {
                m.cartao(c, f == a::Foto::PivoAntes);
            }
            m.bus(abre(b, AnimFieldEdit::OpenEditorAlign));
        }
        _ => {}
    }
}

/// O centro do quad da folha visível de `e` no espaço do pai (a origem mais o Pivot dela, pela pose do objeto).
fn centro_do_quad(world: &ph2d_ecs::World, e: Entity) -> Option<[f32; 2]> {
    let (_, s) = ph2d_sprite_edit::pivot::sprite_do_pivot(world, e)?;
    let t = *world.get::<ph2d_ecs::Transform>(e)?;
    let mut filho = ph2d_ecs::Transform::default();
    (filho.translation.x, filho.translation.y) = (s.anchor[0], s.anchor[1]);
    let c = ph2d_ecs::Transform::compose(t, filho).translation;
    Some([c.x, c.y])
}
