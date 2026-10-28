//! ⭐ **A tool Regras** (W4.6, `docs/BlenderConstraints/04_W4_cad.md` §3.7): a regra cria-se, vê-se, edita-se e
//! apaga-se NO CANVAS. Clicar pontos, lados ou um círculo faz a ESCOLHA (até 2); a ④ oferece só as regras que ela
//! permite ([`possiveis`]) e o botão junta-a pela porta única `junta_regra`; clicar um selo ou o número de uma cota
//! escolhe a regra (o número foca o chip da ②); o Delete apaga-a. Os gizmos são o [`plano`](crate::cad_tool_plano),
//! e o que se pinta é o que se acerta.
//!
//! A shell paga: a linha do quadro ([`frame`]), a do desenho ([`crate::cad_tool_plano::draw`]) e o Delete (a cadeia
//! mudou-se para [`crate::apaga`]). O clique e o Esc chegam pelas portas que ela já chamava (`shape_handles_live::on`).
//! Undo: tudo o que muda o documento muda-o num quadro, e a captura do fim do quadro faz disso UM passo.

use std::cell::RefCell;
use std::collections::BTreeSet;

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::LengthDisplay;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_vector::state_cad::{
    self, CadVista, Escolhida, PedidoCad, Possivel, VECTOR_CAD_VALUE,
};
use ph2d_tool_vector::DrawMode;
use ph2d_vec_constraints::{Folha, medida_na_folha, resolve};
use ph2d_vec_scene::cad::{Lei, PontoCad, RegraId, SegCad};
use ph2d_vec_scene::{VecPathId, VecScene, VecXforms, xform_of};

use crate::cad_tool_plano::{Acerto, Plano};
use crate::state::VecState;

/// Um alvo da escolha: um vértice, um lado (dois vértices seguidos de um contorno) ou um path que é círculo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alvo {
    Ponto(PontoCad),
    Lado(SegCad),
    Circulo(VecPathId),
}

/// O estado da tool (no `VecState`, fora da `VecScene`: não entra no undo).
#[derive(Default)]
pub struct CadTool {
    /// A escolha, pela ordem do clique (máx. 2).
    pub escolha: Vec<Alvo>,
    /// A regra escolhida (o selo ou a cota clicada).
    pub regra: Option<RegraId>,
    /// O que está sob o ponteiro, pelo último plano.
    pub hover: Option<Acerto>,
    /// O clique no número pediu o foco do chip (o quadro foca-o: o press não tem o store).
    focar: bool,
    /// A regra cujo valor o chip já recebeu (a semente é one-shot: semear todo quadro brigaria com a escrita).
    semeada: Option<(RegraId, u64)>,
    /// As entidades que um pedido da barra pode mudar ([`alvos`]); `maos::publicar` leva-as ao Auto Key.
    pub alvos: Vec<u64>,
    /// O último plano pintado — o mesmo que o press e o hover perguntam.
    pub(crate) plano: RefCell<Option<Plano>>,
}

/// ⭐ **As regras que uma escolha permite** — a lista do dono (Vertical, Horizontal, Coincident, Equal, Fixed,
/// Distance, Angle, Radius), cada uma onde faz sentido.
#[must_use]
pub fn possiveis(escolha: &[Alvo]) -> Vec<Possivel> {
    use Possivel as P;
    match escolha {
        [Alvo::Lado(_)] => vec![P::Horizontal, P::Vertical, P::Distancia],
        [Alvo::Lado(_), Alvo::Lado(_)] => vec![P::Igual, P::Angulo],
        [Alvo::Ponto(_)] => vec![P::Fixo],
        [Alvo::Ponto(_), Alvo::Ponto(_)] => {
            vec![P::Coincidente, P::Horizontal, P::Vertical, P::Distancia]
        }
        [Alvo::Circulo(_)] => vec![P::Raio],
        _ => Vec::new(),
    }
}

/// O tipo de uma lei na lista do dono (`None` = uma lei que só a caneta cria, ex.: Paralelo).
#[must_use]
pub fn tipo_de(lei: &Lei) -> Option<Possivel> {
    Some(match lei {
        Lei::Vertical(..) => Possivel::Vertical,
        Lei::Horizontal(..) => Possivel::Horizontal,
        Lei::Coincidente(..) => Possivel::Coincidente,
        Lei::Igual(..) => Possivel::Igual,
        Lei::Fixo(..) => Possivel::Fixo,
        Lei::Distancia(..) => Possivel::Distancia,
        Lei::Angulo(..) => Possivel::Angulo,
        Lei::Raio(..) => Possivel::Raio,
        _ => return None,
    })
}

fn caminhos(lei: &Lei, scene: &VecScene) -> BTreeSet<VecPathId> {
    scene
        .regras()
        .iter()
        .flat_map(|r| r.lei.pontos())
        .chain(lei.pontos())
        .map(PontoCad::path)
        .collect()
}

/// A lei de `p` sobre a escolha, com a MEDIDA que o desenho tem agora (juntar não mexe no desenho) e o `Fixo` onde
/// o ponto está. `None` = a escolha não permite `p`.
#[must_use]
pub fn lei_de(p: Possivel, escolha: &[Alvo], scene: &VecScene, xf: &VecXforms) -> Option<Lei> {
    use Possivel as P;
    let lei = match (p, escolha) {
        (P::Horizontal, [Alvo::Lado(s)]) => Lei::Horizontal(*s),
        (P::Vertical, [Alvo::Lado(s)]) => Lei::Vertical(*s),
        (P::Distancia, [Alvo::Lado(s)]) => Lei::Distancia(s.0, s.1, 0.0),
        (P::Igual, [Alvo::Lado(s), Alvo::Lado(t)]) => Lei::Igual(*s, *t),
        (P::Angulo, [Alvo::Lado(s), Alvo::Lado(t)]) => Lei::Angulo(*s, *t, 0.0),
        (P::Fixo, [Alvo::Ponto(a)]) => Lei::Fixo(*a, [0.0; 2]),
        (P::Coincidente, [Alvo::Ponto(a), Alvo::Ponto(b)]) => Lei::Coincidente(*a, *b),
        (P::Horizontal, [Alvo::Ponto(a), Alvo::Ponto(b)]) => Lei::Horizontal(SegCad(*a, *b)),
        (P::Vertical, [Alvo::Ponto(a), Alvo::Ponto(b)]) => Lei::Vertical(SegCad(*a, *b)),
        (P::Distancia, [Alvo::Ponto(a), Alvo::Ponto(b)]) => Lei::Distancia(*a, *b, 0.0),
        (P::Raio, [Alvo::Circulo(c)]) => Lei::Raio(*c, 0.0),
        _ => return None,
    };
    let ids = caminhos(&lei, scene);
    let folha = Folha::new(
        ids.iter()
            .filter_map(|&id| scene.path(id).map(|p| (p, xform_of(xf, id)))),
    );
    let mut lei = lei;
    if let Lei::Fixo(a, xy) = &mut lei {
        *xy = ph2d_vec_constraints::Solucao::da_folha(&folha).ponto(*a)?;
    }
    if lei.medida().is_some() {
        let agora = medida_na_folha(&folha, &lei)?;
        *lei.medida_mut()? = agora;
    }
    Some(lei)
}

/// ⭐ **O que a tool aponta para o Auto Key**: com uma regra ou uma escolha na mão, todo path que a resolução da folha
/// pode mover (os citados por uma regra, mais os da escolha) — a medida da tampa grava-se com a caixa selecionada.
#[must_use]
pub fn alvos(
    t: &CadTool,
    scene: &VecScene,
    entidades: &ph2d_vec_entities::entities::VecEntityMap,
) -> Vec<u64> {
    if t.regra.is_none() && t.escolha.is_empty() {
        return Vec::new();
    }
    let escolha = t.escolha.iter().flat_map(|a| match *a {
        Alvo::Ponto(p) => [Some(p.path()), None],
        Alvo::Lado(s) => [Some(s.0.path()), Some(s.1.path())],
        Alvo::Circulo(id) => [Some(id), None],
    });
    let ids: BTreeSet<VecPathId> = scene
        .regras()
        .iter()
        .flat_map(|r| r.lei.pontos())
        .map(PontoCad::path)
        .chain(escolha.flatten())
        .collect();
    ids.iter()
        .filter_map(|id| entidades.get(id).copied())
        .collect()
}

/// Resolve todas as regras sobre todo path citado, a partir do desenho de AGORA, e escreve a solução de volta — o
/// que faz uma regra nova (ou uma medida escrita) mover o desenho no mesmo quadro.
pub fn resolve_a_folha(scene: &mut VecScene, xf: &VecXforms) {
    let ids: BTreeSet<VecPathId> = scene
        .regras()
        .iter()
        .flat_map(|r| r.lei.pontos())
        .map(PontoCad::path)
        .collect();
    let res = {
        let folha = Folha::new(
            ids.iter()
                .filter_map(|&id| scene.path(id).map(|p| (p, xform_of(xf, id)))),
        );
        resolve(&folha, scene.regras(), None, None)
    };
    for &id in &ids {
        if let Some(p) = scene.path_mut(id) {
            res.solucao.aplica(p, xform_of(xf, id));
        }
    }
}

/// As unidades do projecto: o comprimento e o ângulo que o artista vê.
#[derive(Clone, Copy, Debug)]
pub struct Unidades {
    pub comprimento: LengthDisplay,
    pub angulo: ph2d_editor_core::DisplayAngle,
}

impl Unidades {
    #[must_use]
    pub fn of(hero: &HeroScreen) -> Self {
        Self {
            comprimento: LengthDisplay::of(&hero.project),
            angulo: hero.project.display_angle,
        }
    }

    /// A medida de `lei` na unidade de ecrã (`None` = não é medida).
    #[must_use]
    pub fn no_ecra(self, lei: &Lei) -> Option<f64> {
        let m = lei.medida()?;
        Some(match lei {
            Lei::Angulo(..) => self.angulo.from_radians_f64(m),
            _ => self.comprimento.value(m),
        })
    }

    /// O texto da cota: o número e a unidade.
    #[must_use]
    pub fn texto(self, lei: &Lei) -> Option<String> {
        let v = self.no_ecra(lei)?;
        let sufixo = match lei {
            Lei::Angulo(..) => self.angulo.suffix(),
            _ => self.comprimento.suffix(),
        };
        let raio = if matches!(lei, Lei::Raio(..)) {
            "R "
        } else {
            ""
        };
        Some(format!("{raio}{} {sufixo}", arredonda(v)).replace(" °", "°"))
    }

    /// O valor escrito (unidade de ecrã) → o que a lei guarda (mundo; radianos num ângulo). Distância e raio não são
    /// negativos.
    #[must_use]
    pub fn do_ecra(self, lei: &Lei, v: f64) -> f64 {
        match lei {
            Lei::Angulo(..) => self.angulo.to_radians_f64(v),
            _ => self.comprimento.to_world(v).max(0.0),
        }
    }
}

/// Duas casas no máximo, sem zeros à direita.
fn arredonda(v: f64) -> String {
    let s = format!("{v:.2}");
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// Aplica um pedido da barra. Devolve se o documento mudou.
pub fn aplica(
    t: &mut CadTool,
    scene: &mut VecScene,
    xf: &VecXforms,
    p: PedidoCad,
    u: Unidades,
) -> bool {
    match p {
        PedidoCad::Junta(tipo) => {
            let Some(lei) = lei_de(tipo, &t.escolha, scene, xf) else {
                return false;
            };
            let Some(nova) = ph2d_vec_edit::junta_regra(scene, xf, lei) else {
                return false;
            };
            scene.assenta_regras();
            resolve_a_folha(scene, xf);
            t.escolha.clear();
            // A nova é a última (o assentamento deu-lhe o id); escolhida, a cota foca-se para escrever.
            t.regra = scene
                .regras()
                .last()
                .filter(|r| r.lei == nova.lei)
                .map(|r| r.id);
            t.focar = nova.lei.medida().is_some() && !nova.referencia;
            true
        }
        PedidoCad::Medida(v) => {
            let Some(r) = t
                .regra
                .and_then(|id| scene.regra_mut(id))
                .filter(|r| !r.referencia)
            else {
                return false;
            };
            let novo = u.do_ecra(&r.lei, v);
            let Some(d) = r.lei.medida_mut() else {
                return false;
            };
            *d = novo;
            resolve_a_folha(scene, xf);
            true
        }
        PedidoCad::Apaga => apaga_escolhida(t, scene),
    }
}

fn apaga_escolhida(t: &mut CadTool, scene: &mut VecScene) -> bool {
    let Some(id) = t.regra.take() else {
        return false;
    };
    let antes = scene.regras().len();
    scene.regras_mut().retain(|r| r.id != id);
    scene.regras().len() != antes
}

/// O Delete com a tool Regras na mão e uma regra escolhida. `true` = consumido.
pub fn delete(vec: &mut VecState, scene: &mut VecScene) -> bool {
    vec.draw_config.mode == DrawMode::Rules && apaga_escolhida(&mut vec.cad_tool, scene)
}

fn existe(scene: &VecScene, a: &Alvo) -> bool {
    let p = |q: PontoCad| match q {
        PontoCad::Vert(id, v) => scene.path(id).and_then(|p| p.vert_by_id(v)).is_some(),
        PontoCad::Centro(id) => scene.path(id).is_some(),
    };
    match *a {
        Alvo::Ponto(q) => p(q),
        Alvo::Lado(s) => p(s.0) && p(s.1),
        Alvo::Circulo(id) => scene.path(id).is_some(),
    }
}

thread_local! {
    /// O canal da cota escolhida (o losango do chip da ② lê-o: `marcas::Contexto::do_painel`).
    static CANAL: std::cell::Cell<Option<ph2d_timeline::PropKind>> = const { std::cell::Cell::new(None) };
}

/// O canal da medida de uma regra — a porta única da família CAD (`medida::chave_da`).
#[must_use]
pub fn canal_da_regra(lei: &Lei, id: RegraId) -> Option<ph2d_timeline::PropKind> {
    ph2d_vec_entities::medida::chave_da(lei, id).map(ph2d_timeline::PropKind::Field)
}

/// O canal da cota escolhida neste quadro (`None` = nenhuma, ou fora da tool).
#[must_use]
pub fn canal_da_escolhida() -> Option<ph2d_timeline::PropKind> {
    CANAL.with(std::cell::Cell::get)
}

/// ⭐ **Um quadro da tool**: drena os pedidos da barra, larga o que deixou de existir (undo, um ponto apagado),
/// acha o hover pelo último plano e publica a ④/② (a vista, a semente do número, o foco pedido pelo clique).
pub fn frame(
    scene: &mut VecScene,
    sim: &mut ph2d_ecs::SimWorld,
    vec: &mut VecState,
    xf: &VecXforms,
    hero: &mut HeroScreen,
) {
    crate::hud_tool::frame(sim, vec); // a irmã do HUD (W5.5) pela mesma porta
    let u = Unidades::of(hero);
    let ativa = vec.draw_config.mode == DrawMode::Rules;
    let t = &mut vec.cad_tool;
    let mut ecoa = false;
    for p in state_cad::drena() {
        ecoa |= aplica(t, scene, xf, p, u);
    }
    if !ativa {
        t.escolha.clear();
        t.regra = None;
    }
    t.alvos = alvos(t, scene, &vec.entities);
    t.escolha.retain(|a| existe(scene, a));
    if t.regra.is_some_and(|id| scene.regra(id).is_none()) {
        t.regra = None;
    }
    t.hover = hero.store.last_pointer().and_then(|p| {
        let plano = t.plano.borrow();
        plano
            .as_ref()
            .and_then(|pl| pl.acerta([f64::from(p[0]), f64::from(p[1])]))
    });
    let regra = t.regra.and_then(|id| scene.regra(id).copied());
    CANAL.with(|c| {
        c.set(
            regra
                .filter(|r| ativa && !r.referencia)
                .and_then(|r| canal_da_regra(&r.lei, r.id)),
        );
    });
    if let Some(r) = regra
        && let Some(v) = u.no_ecra(&r.lei)
    {
        let marca = (r.id, v.to_bits());
        // A medida mudou por outro caminho (scrub, undo): o chip re-semeia; o que o artista digita fica
        // (`set_number_value` só re-semeia um campo focado que ninguém tocou).
        if ecoa || t.semeada != Some(marca) {
            hero.store.set_number_value(VECTOR_CAD_VALUE, v);
            t.semeada = Some(marca);
        }
        if std::mem::take(&mut t.focar) {
            ph2d_editor_core::interaction::dispatch::focus_number_field(
                &mut hero.store,
                VECTOR_CAD_VALUE,
            );
        }
    } else {
        t.semeada = None;
        t.focar = false;
    }
    let ancora = ativa
        .then(|| {
            let plano = t.plano.borrow();
            plano.as_ref().and_then(|pl| pl.ancora(&t.escolha, t.regra))
        })
        .flatten();
    state_cad::set_cad_vista(CadVista {
        possiveis: if ativa {
            possiveis(&t.escolha)
        } else {
            Vec::new()
        },
        escolhida: regra.filter(|_| ativa).map(|r| Escolhida {
            tipo: tipo_de(&r.lei),
            referencia: r.referencia,
        }),
        ancora: ancora.map(|r: Rect| r),
    });
}

/// O press com a tool na mão (pela porta `shape_handles_live::on::press`): o número de uma cota escolhe-a e pede o
/// foco do chip; um selo escolhe a regra; um ponto/lado/círculo entra na escolha (de novo = sai; um 3.º recomeça);
/// o vazio limpa. Consome SEMPRE — um clique no vazio não pode cair na cadeia e desenhar.
pub fn press(vec: &mut VecState, w: [f64; 2]) -> bool {
    let t = &mut vec.cad_tool;
    let acerto = {
        let plano = t.plano.borrow();
        plano.as_ref().and_then(|pl| pl.acerta(pl.no_ecra(w)))
    };
    match acerto {
        Some(Acerto::Numero(id)) => {
            t.regra = Some(id);
            t.escolha.clear();
            t.focar = true;
        }
        Some(Acerto::Selo(id)) => {
            t.regra = Some(id);
            t.escolha.clear();
        }
        Some(Acerto::Alvo(a)) => {
            t.regra = None;
            if let Some(i) = t.escolha.iter().position(|x| *x == a) {
                t.escolha.remove(i);
            } else {
                if t.escolha.len() >= 2 {
                    t.escolha.clear();
                }
                t.escolha.push(a);
            }
        }
        None => {
            t.escolha.clear();
            t.regra = None;
        }
    }
    true
}

/// O Esc: larga a escolha e a regra escolhida. `true` = havia o que largar.
pub fn escape(vec: &mut VecState) -> bool {
    let t = &mut vec.cad_tool;
    let havia = !t.escolha.is_empty() || t.regra.is_some();
    t.escolha.clear();
    t.regra = None;
    havia
}

#[cfg(test)]
#[path = "cad_tool_tests.rs"]
pub(crate) mod tests;
