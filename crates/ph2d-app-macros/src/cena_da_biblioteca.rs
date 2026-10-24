//! `PH2D_MACRO_SMOKE=7` — **a biblioteca, o caminho inteiro, feito pelo app sozinho** (W18,
//! `docs/Macros/16_W18_plano.md` §6): Add ▸ Macro, Gravar, uma imagem nova dentro dele e duas pinceladas
//! de cores diferentes, Parar, Guardar na biblioteca, Exportar, apagar tudo, Importar, Aplicar ▸ Criar
//! novo (toca), e uma imagem maior com Aplicar ▸ Por cima, o objecto Macro rodado e movido (toca).
//!
//! Só portas de produto: o «+» da Hierarchy e o clique no menu, o pedido do modal de imagem nova, o modo
//! Paint, o seletor de cor do pincel, o ponteiro da tela, os mesmos [`MacroRequest`] que o painel MACROS
//! envia (o diálogo de ficheiro responde-se pela porta [`ph2d_app_host::modal::responde_o_proximo`]), o
//! `Delete` da Hierarchy, o Inspector (a pose) e o Play da barra de cima. Cada passo diz no ecrã o que é.
//!
//! O [`roteiro`] e os [`pedido`]/[`pose`] são os mesmos na shell e na sonda sem placa
//! (`tests/it/cena_da_biblioteca.rs`, o gate `a_cena_7_percorre_o_caminho_inteiro`).
//!
//! A cena `=8` (W19, [`crate::cena_do_mundo`]) corre neste mesmo condutor, com o roteiro dela.

use crate::Macros;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::action_bus::{EditorAction, HierRequest, TransportCmd};
use ph2d_editor_core::macro_edits::{self as m, MacroRequest, OndeAplicar};
use std::path::PathBuf;

/// O lado da imagem gravada (px).
pub const LADO: u32 = 256;
/// A imagem maior do passo 10 (px).
pub const MAIOR: (u32, u32) = (512, 256);
/// As cores das duas pinceladas (sRGB).
pub const CORES: [[u8; 3]; 2] = [[210, 40, 40], [30, 70, 210]];
/// A linha `v` de cada pincelada (horizontais, de `u = 0,2` a `0,8`).
pub const LINHAS: [f32; 2] = [0.35, 0.65];
/// Quanto o objecto Macro roda sobre a imagem maior.
pub const GRAUS: f32 = 30.0;
/// Onde a imagem maior fica (px do mundo à escala do projecto, à direita da primeira).
pub const MAIOR_EM_PX: [f32; 2] = [430.0, 0.0];
/// Quanto o objecto Macro anda sobre ela depois de nascer no centro dela (px).
pub const ANDA_PX: [f32; 2] = [-96.0, 0.0];

/// Quem a pose muda.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Qual {
    /// A imagem maior do passo 10.
    Maior,
    /// O objecto escolhido (o objecto Macro que o aplicar fez nascer).
    Escolhido,
    /// O chão da cena `=8` (fora do objecto Macro).
    Chao,
}

/// Um passo da cena.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Passo {
    /// A frase do passo `k` (`1..=10`; `0` = o fim) no ecrã.
    Diz(u8),
    /// O «+» da Hierarchy (o menu Add abre).
    AbreAdd,
    /// O clique em «Macro» no menu Add.
    EscolheMacro,
    Grava,
    /// File ▸ New Image, branca, `lado`².
    NovaImagem(u32),
    /// O modo Paint da imagem escolhida.
    Pinta,
    /// De volta ao modo Object (o Inspector e o painel MACROS à vista).
    Objeto,
    /// A cor do pincel no seletor da miniatura de cor do Painter (abre-o com ela).
    Cor([u8; 3]),
    /// O seletor fecha (o clique fora dele).
    FechaCor,
    /// O ponteiro na imagem escolhida, em fracções `(u, v)`.
    Desce([f32; 2]),
    Move([f32; 2]),
    Sobe,
    Para,
    Guarda,
    Exporta,
    /// O objecto Macro escolhido e o `Delete` da Hierarchy (a imagem vai com ele).
    Apaga,
    Importa,
    AplicaNovo,
    /// A imagem maior (branca) nasce pela porta de produto do aplicar.
    ImagemMaior,
    /// A pose de `qual` pelo Inspector: rodada `graus`, deslocada `px` (à escala do projecto).
    Pose {
        qual: Qual,
        graus: f32,
        px: [f32; 2],
    },
    AplicaPorCima,
    /// O Play da barra de cima (os clipes em Autoplay tocam).
    Toca,
    /// O Reset do transporte (a cabeça volta ao 0 e pára).
    Rebobina,
    /// *Enquadrar tudo*.
    Enquadra,
    /// O chão `n` da cena `=8` nasce (uma imagem cinzenta comprida) pela porta do aplicar.
    Chao(u8),
    /// *Add Physics Body* no Inspector de `qual` (o collider à medida da imagem).
    Corpo(Qual),
    /// O tipo do corpo do chão: *Static* (o chip do Inspector).
    Estatico,
    /// O interruptor *Physics* da barra de transporte, armado.
    Fisica,
    /// O Pause do transporte.
    Pausa,
    /// O chão escolhido e o `Delete` da Hierarchy.
    ApagaChao,
}

pub(crate) const PASSO_MS: u64 = 16;

fn pincelada(r: &mut Vec<(u64, Passo)>, t0: u64, v: f32) {
    let n = 900 / PASSO_MS;
    for i in 0..=n {
        #[expect(clippy::cast_precision_loss, reason = "poucos passos")]
        let u = 0.2 + 0.6 * (i as f32 / n as f32);
        let p = if i == 0 {
            Passo::Desce([u, v])
        } else {
            Passo::Move([u, v])
        };
        r.push((t0 + i * PASSO_MS, p));
    }
    r.push((t0 + 900 + PASSO_MS, Passo::Sobe));
}

/// **O roteiro**: cada passo e o instante (ms do relógio da cena), por ordem. As pausas dão para ver;
/// a gravação é curta (a reprodução dura o mesmo).
#[must_use]
pub fn roteiro() -> Vec<(u64, Passo)> {
    use Passo as P;
    let mut r = vec![
        (500, P::Diz(1)),
        (1_500, P::AbreAdd),
        (3_000, P::EscolheMacro),
        (5_000, P::Diz(2)),
        (6_000, P::Grava),
        (6_300, P::Diz(3)),
        (6_800, P::NovaImagem(LADO)),
        (7_300, P::Pinta),
        (7_800, P::Cor(CORES[0])),
        (8_300, P::FechaCor),
    ];
    pincelada(&mut r, 8_500, LINHAS[0]);
    r.extend([(9_800, P::Cor(CORES[1])), (10_300, P::FechaCor)]);
    pincelada(&mut r, 10_500, LINHAS[1]);
    r.extend([
        (11_600, P::Diz(4)),
        (12_400, P::Para),
        (12_900, P::Objeto),
        (14_500, P::Diz(5)),
        (15_500, P::Guarda),
        (18_000, P::Diz(6)),
        (19_000, P::Exporta),
        (21_500, P::Diz(7)),
        (22_500, P::Apaga),
        (25_000, P::Diz(8)),
        (26_000, P::Importa),
        (28_500, P::Diz(9)),
        (29_500, P::AplicaNovo),
        (30_500, P::Toca),
        (38_000, P::Rebobina),
        (38_500, P::Diz(10)),
        (39_500, P::ImagemMaior),
        (
            40_500,
            P::Pose {
                qual: Qual::Maior,
                graus: 0.0,
                px: MAIOR_EM_PX,
            },
        ),
        (41_000, P::Enquadra),
        (42_000, P::AplicaPorCima),
        (
            43_500,
            P::Pose {
                qual: Qual::Escolhido,
                graus: GRAUS,
                px: ANDA_PX,
            },
        ),
        (45_000, P::Toca),
        (55_000, P::Diz(0)),
    ]);
    r
}

/// O roteiro da cena `nivel` (`7` ou `8`).
#[must_use]
pub fn roteiro_de(nivel: u8) -> Vec<(u64, Passo)> {
    if nivel == 8 {
        crate::cena_do_mundo::roteiro()
    } else {
        roteiro()
    }
}

/// O estado da cena: as pastas, o objecto Macro gravado, a imagem maior.
#[derive(Default)]
pub struct Estado {
    /// A cena (`7` ou `8`): as frases e o roteiro.
    pub nivel: u8,
    /// O chão da cena `=8` que existe agora.
    pub chao: Option<u64>,
    pub biblioteca: PathBuf,
    pub exportada: PathBuf,
    /// O objecto Macro do passo 1 (escolhido ao Gravar).
    pub gravado: Option<u64>,
    pub maior: Option<u64>,
    /// O nome com que o Importar a deixou na biblioteca (a que se aplica).
    pub importada: Option<String>,
    /// A cor que o seletor aberto escolhe (o fecho escreve-a outra vez: o pincel lê-a no fecho).
    cor: Option<ph2d_tokens::ColorValue>,
}

impl Estado {
    /// As pastas da cena numa raiz nova (`raiz/biblioteca`, `raiz/exportada.ph2dmacro`), limpas.
    #[must_use]
    pub fn em(raiz: &std::path::Path) -> Self {
        let _ = std::fs::remove_dir_all(raiz);
        Self {
            biblioteca: raiz.join("biblioteca"),
            exportada: raiz.join(format!("exportada{}", ph2d_macro::ficheiro::EXTENSAO)),
            ..Self::default()
        }
    }
}

/// **O pedido ao painel MACROS de um passo** (o que o botão dele envia), com o diálogo respondido.
/// `None` = o passo não é do painel.
pub fn pedido(p: Passo, m: &mut Macros, e: &mut Estado) -> Option<MacroRequest> {
    // A importada: a frase do Importar diz o nome com que ficou (a lista ordena por nome).
    if let Some(a) = m.aviso().filter(|a| a.chave == m::MACRO_LIB_IMPORTADA) {
        e.importada.clone_from(&a.detalhe);
    }
    Some(match p {
        Passo::Grava => MacroRequest::Grava,
        Passo::Para => MacroRequest::Para,
        Passo::Guarda => MacroRequest::Guarda {
            entity_bits: e.gravado?,
        },
        Passo::Exporta => {
            ph2d_app_host::modal::responde_o_proximo(e.exportada.clone());
            MacroRequest::Exporta {
                nome: m.nomes_da_biblioteca().into_iter().next()?,
            }
        }
        Passo::Importa => {
            ph2d_app_host::modal::responde_o_proximo(e.exportada.clone());
            MacroRequest::Importa
        }
        Passo::AplicaNovo | Passo::AplicaPorCima => MacroRequest::Aplica {
            // A `=8` não importa: aplica a guardada.
            nome: e
                .importada
                .clone()
                .or_else(|| m.nomes_da_biblioteca().into_iter().next())?,
            onde: if p == Passo::AplicaNovo {
                OndeAplicar::Novo { em: None }
            } else {
                OndeAplicar::PorCima { alvo: e.maior? }
            },
        },
        _ => return None,
    })
}

/// **A pose nova** de quem tem `atual`: rodada `graus`, deslocada `px / ppm` (o resto fica).
#[must_use]
pub fn pose(atual: Transform, graus: f32, px: [f32; 2], ppm: f32) -> Transform {
    let mut t = atual;
    t.rotation = graus.to_radians();
    t.translation.x += px[0] / ppm;
    t.translation.y += px[1] / ppm;
    t
}

/// A frase do passo `k` no ecrã.
#[must_use]
pub fn frase(k: u8, e: &Estado) -> String {
    if e.nivel == 8 {
        return crate::cena_do_mundo::frase(k);
    }
    match k {
        0 => ph2d_i18n::tr("app.macros.smoke7.done").to_owned(),
        5 => ph2d_i18n::tr_with(
            "app.macros.smoke7.step.5",
            &[("detail", &e.biblioteca.display().to_string())],
        ),
        6 => ph2d_i18n::tr_with(
            "app.macros.smoke7.step.6",
            &[("detail", &e.exportada.display().to_string())],
        ),
        k => ph2d_i18n::tr(&format!("app.macros.smoke7.step.{k}")).to_owned(),
    }
}

/// O condutor da cena no app: o relógio (nasce no 1.º quadro) e os passos já dados.
struct Condutor {
    inicio: std::time::Instant,
    dados: usize,
    roteiro: Vec<(u64, Passo)>,
    estado: Estado,
}

thread_local! {
    static CENA: std::cell::RefCell<Option<Condutor>> = const { std::cell::RefCell::new(None) };
}

/// **Os passos que vencem neste quadro** (o 1.º quadro arma a cena: a biblioteca numa pasta de teste,
/// nunca a do dono, salvo `PH2D_MACROS_DIR`).
pub fn vencidos() -> (u64, Vec<Passo>) {
    CENA.with(|c| {
        let mut c = c.borrow_mut();
        let c = c.get_or_insert_with(|| {
            let nivel = u8::from(std::env::var("PH2D_MACRO_SMOKE").is_ok_and(|n| n == "8")) + 7;
            let raiz = std::env::temp_dir().join(format!("ph2d-macro-smoke-{nivel}"));
            let mut estado = Estado::em(&raiz);
            estado.nivel = nivel;
            if let Some(p) = std::env::var_os("PH2D_MACROS_DIR") {
                estado.biblioteca = PathBuf::from(p);
            }
            crate::com(|m| m.com_pasta(estado.biblioteca.clone()));
            eprintln!(
                "[macro smoke ={nivel}] the app does the whole path by itself (~60 s); the macros \
                 folder is {}",
                estado.biblioteca.display()
            );
            Condutor {
                inicio: std::time::Instant::now(),
                dados: 0,
                roteiro: roteiro_de(nivel),
                estado,
            }
        });
        let ms = u64::try_from(c.inicio.elapsed().as_millis()).unwrap_or(u64::MAX);
        let n = c.roteiro[c.dados..]
            .iter()
            .take_while(|(t, _)| *t <= ms)
            .count();
        let v = c.roteiro[c.dados..c.dados + n]
            .iter()
            .map(|(_, p)| *p)
            .collect();
        c.dados += n;
        (ms, v)
    })
}

impl Passo {
    /// O ponto da tela de um passo do ponteiro.
    #[must_use]
    pub fn uv(self) -> Option<[f32; 2]> {
        match self {
            Self::Desce(uv) | Self::Move(uv) => Some(uv),
            _ => None,
        }
    }
}

/// **A imagem maior** (branca, [`MAIOR`]) ou **o chão** da `=8` ([`Passo::Chao`]) pela porta de produto
/// do aplicar (`nasce`: o atlas e o `AssetDb`); devolve os bits e guarda-os.
pub fn nasce_a_maior(
    p: Passo,
    sim: &mut SimWorld,
    nasce: impl FnOnce(&mut SimWorld, (u32, u32, Vec<u8>), &str) -> Result<u64, String>,
) -> Option<u64> {
    let chao = matches!(p, Passo::Chao(_));
    let (img, nome) = if chao {
        (
            crate::cena_do_mundo::chao_px()?,
            ph2d_i18n::tr("app.macros.smoke8.floor"),
        )
    } else {
        let (w, h) = MAIOR;
        let px = vec![255; usize::try_from(w * h * 4).ok()?];
        ((w, h, px), ph2d_i18n::tr("app.macros.smoke7.bigger_image"))
    };
    let b = nasce(sim, img, nome)
        .map_err(|e| eprintln!("[macro smoke] {nome} was not born: {e}"))
        .ok()?;
    CENA.with(|c| {
        if let Some(c) = c.borrow_mut().as_mut() {
            *if chao {
                &mut c.estado.chao
            } else {
                &mut c.estado.maior
            } = Some(b);
        }
    });
    Some(b)
}

/// Quem o `Delete` de `p` tira: o objecto Macro gravado ([`Passo::Apaga`]) ou o chão.
#[must_use]
pub fn a_apagar(p: Passo) -> Option<u64> {
    CENA.with(|c| {
        let c = c.borrow();
        let e = &c.as_ref()?.estado;
        match p {
            Passo::Apaga => e.gravado,
            Passo::ApagaChao => e.chao,
            _ => None,
        }
    })
}

impl Passo {
    /// O passo é um `Delete` da Hierarchy.
    #[must_use]
    pub fn apaga(self) -> bool {
        matches!(self, Self::Apaga | Self::ApagaChao)
    }
}

/// **O passo pelas portas que o `HeroScreen` tem** (o menu, o modal, o modo, o seletor, o painel, o
/// Inspector, o transporte, a frase). Devolve o passo quando é da shell (o ponteiro, o `Delete`, a
/// imagem maior).
pub fn faz(
    p: Passo,
    hero: &mut HeroScreen,
    sim: &SimWorld,
    toasts: &mut ph2d_editor_core::toast::ToastQueue,
) -> Option<Passo> {
    use ph2d_editor_core::ids;
    let ppm = hero.project.pixels_per_meter;
    match p {
        Passo::Diz(k) => {
            let (f, n) = CENA.with(|c| {
                let c = c.borrow();
                let e = &c.as_ref()?.estado;
                Some((frase(k, e), e.nivel))
            })?;
            eprintln!("[macro smoke ={n}] {f}");
            // O painel MACROS à vista: a gravação, a biblioteca e a frase de cada gesto dela.
            ph2d_editor_core::panel::PanelHostInternal::set_panel_visible(hero, "macros", true);
            toasts.push(ph2d_editor_core::toast::Toast::info(f).ttl_seconds(6.0));
        }
        Passo::AbreAdd => hero.bus.push(EditorAction::Hierarchy(HierRequest::AddRoot)),
        Passo::EscolheMacro => {
            let ev =
                ph2d_editor_core::interaction::WidgetEvent::Click(crate::object_add::MACRO.id());
            if !hero.apply_event(ev) {
                eprintln!("[macro smoke] the Add menu is not open");
            }
        }
        Passo::NovaImagem(lado) => {
            hero.store.set_new_image_size(lado);
            hero.store.set_new_image_bg(2);
            hero.store.request_new_image();
        }
        Passo::Pinta | Passo::Objeto => {
            use ph2d_editor_core::object_mode::{ModeRequest, ObjectMode};
            let modo = if p == Passo::Pinta {
                ObjectMode::Paint
            } else {
                ObjectMode::Object
            };
            hero.bus
                .push(EditorAction::ObjectMode(ModeRequest::Enter(modo)));
        }
        Passo::Cor(c) => {
            let s = &mut hero.store;
            s.set_widget_color(ids::PAINTER_COLOR_THUMB, [c[0], c[1], c[2], 0xFF]);
            s.set_picker_target(Some(ids::PAINTER_COLOR_THUMB));
            let v = ph2d_tokens::ColorValue::from_rgba8(c[0], c[1], c[2], 0xFF);
            s.set_blender_value(ids::INSP_BLENDER_PICKER, v);
            CENA.with(|x| {
                if let Some(x) = x.borrow_mut().as_mut() {
                    x.estado.cor = Some(v);
                }
            });
        }
        Passo::FechaCor => {
            if let Some(v) = CENA.with(|x| x.borrow().as_ref().and_then(|x| x.estado.cor)) {
                hero.store.set_blender_value(ids::INSP_BLENDER_PICKER, v);
            }
            hero.store.set_picker_target(None);
        }
        Passo::Pose { qual, graus, px } => {
            let bits = match qual {
                Qual::Escolhido => hero.gizmo.selection,
                q => o_chao_ou_a_maior(q),
            }?;
            let atual = sim
                .world()
                .get::<Transform>(Entity::from_bits(bits))
                .copied()?;
            let t = pose(atual, graus, px, ppm);
            let info = ph2d_editor_core::screens::hero::InspectorTransformInfo {
                entity_bits: bits,
                translation: [t.translation.x, t.translation.y],
                rotation_rad: t.rotation,
                scale: [t.scale.x, t.scale.y],
                skew_rad: [t.skew_x, t.skew_y],
            };
            hero.bus.push(EditorAction::InspectorTransformEdit(info));
        }
        Passo::Corpo(_) | Passo::Estatico => {
            let qual = if let Passo::Corpo(q) = p {
                q
            } else {
                Qual::Chao
            };
            let bits = match qual {
                Qual::Escolhido => hero.gizmo.selection,
                q => o_chao_ou_a_maior(q),
            }?;
            use ph2d_editor_core::PhysicsFieldEdit as F;
            // O chip *Static* é o 2.º do Inspector (`BodyKind::Static.tag()`).
            let edit = if p == Passo::Estatico {
                F::Kind(1)
            } else {
                F::Add
            };
            // O cartão Physics do Inspector à vista (o botão que este passo carrega).
            ph2d_editor_core::panel::PanelHostInternal::set_panel_visible(hero, "macros", false);
            ph2d_editor_core::panel::PanelHostInternal::set_panel_visible(hero, "inspector", true);
            hero.bus.push(EditorAction::InspectorPhysicsEdit {
                entity_bits: bits,
                edit,
            });
        }
        // O interruptor da barra de transporte: o `PanelEvent` que o clique levanta.
        Passo::Fisica => hero.bus.push(EditorAction::TimelinePanelEvent(
            ph2d_editor_core::tool::PanelEvent::Toggle(
                ph2d_panel_timeline::ids::TIMELINE_PHYSICS,
                true,
            ),
        )),
        Passo::Pausa => hero.bus.push(EditorAction::Transport(TransportCmd::Pause)),
        Passo::Toca => hero.bus.push(EditorAction::Transport(TransportCmd::Play)),
        Passo::Rebobina => hero.bus.push(EditorAction::Transport(TransportCmd::Reset)),
        Passo::Enquadra => hero.bus.push(EditorAction::SetViewFocus {
            kind: ph2d_editor_core::ViewFocusKind::All,
        }),
        Passo::Desce(_)
        | Passo::Move(_)
        | Passo::Sobe
        | Passo::Apaga
        | Passo::ApagaChao
        | Passo::ImagemMaior
        | Passo::Chao(_) => {
            return Some(p);
        }
        _ => {
            let r = CENA.with(|c| {
                let mut c = c.borrow_mut();
                let c = c.as_mut()?;
                if p == Passo::Grava {
                    c.estado.gravado = hero.gizmo.selection;
                }
                crate::com(|m| pedido(p, m, &mut c.estado))
            });
            match r {
                Some(r) => crate::com(|m| m.pede(r)),
                None => eprintln!("[macro smoke] {p:?}: nothing to ask the Macros panel"),
            }
        }
    }
    None
}

/// A imagem maior ou o chão (o que a cena guardou).
fn o_chao_ou_a_maior(q: Qual) -> Option<u64> {
    CENA.with(|c| {
        let c = c.borrow();
        let e = &c.as_ref()?.estado;
        if q == Qual::Chao { e.chao } else { e.maior }
    })
}
