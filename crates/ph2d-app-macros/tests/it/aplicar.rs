//! W18 (`docs/Macros/16_W18_plano.md` §2.5–2.7): guardar ▸ exportar ▸ importar ▸ aplicar — a macro
//! sai de um mundo e faz nascer o que gravou noutro; por cima, toca na imagem de baixo no tamanho
//! gravado; o que não toca até ao fim não se escreve; duas famílias por cima de um objecto recusam;
//! a receita dos assets aplica-se pela mesma porta.

use crate::gate7::Banca;
use ph2d_app_macros::biblioteca;
use ph2d_ecs::{ChildOf, Entity, Macro, MasterRoot, Name, StableId, Transform};
use ph2d_editor_core::macro_edits::{self as m, MacroRequest, OndeAplicar};
use ph2d_macro::ficheiro::{EXTENSAO, Ficheiro};
use ph2d_macro::{Event, MacroDoc, MacroTime, RefRect, Track};
use ph2d_tool_painter::PainterTool;
use ph2d_tool_painter::macro_track::{encaixe::Encaixe, fim, inicial};
use std::path::PathBuf;

const IMAGEM: u64 = 10;
const OBJETO: u64 = 20;

/// Sem Motion: a porta de transpor recusa (nenhuma trilha dele aqui).
struct SemMotion;
impl ph2d_editor_core::macro_tutorial::TranspoeDaFamilia for SemMotion {}

pub(crate) fn pasta(nome: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "ph2d-macros-aplicar-{nome}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&p);
    p
}

/// Uma banca cuja imagem tem `w×h` (branca) e vive dentro de um objecto Macro; devolve o objecto.
pub(crate) fn banca(w: u32, h: u32, lib: &PathBuf) -> (Banca, u64) {
    let mut b = Banca::nova();
    b.m.com_pasta(lib.clone());
    let wd = b.sim.world_mut();
    let dono = wd
        .spawn((
            Transform::IDENTITY,
            Name::new("Splash"),
            Macro::default(),
            StableId(OBJETO),
        ))
        .id();
    let img = wd
        .spawn((
            Transform::IDENTITY,
            Name::new("Tela"),
            StableId(IMAGEM),
            ChildOf(dono),
        ))
        .id()
        .to_bits();
    let n = usize::try_from(w * h * 4).unwrap();
    b.painter.bind_document(img, vec![255u8; n], w, h);
    b.bits = img;
    b.escolhido = Some(dono.to_bits());
    (b, dono.to_bits())
}

pub(crate) fn grava(b: &mut Banca, de: [f32; 2], para: [f32; 2]) {
    b.m.pede(MacroRequest::Grava);
    b.quadro();
    b.pincelada(de, para);
    b.m.pede(MacroRequest::Para);
    b.quadro();
}

fn aviso(b: &Banca) -> &'static str {
    b.m.aviso().map(|a| a.chave).unwrap_or("nenhum")
}

/// Os píxeis de `doc` pelo caminho do load de um projecto.
fn composto(doc: ph2d_tool_painter::PaintedDocument) -> (Vec<u8>, u32, u32) {
    let mut t = PainterTool::default();
    t.install_document(1, doc);
    t.baked_document_pixels(1).expect("compõe")
}

fn px(img: &(Vec<u8>, u32, u32), x: u32, y: u32) -> [u8; 4] {
    let i = ((y * img.1 + x) * 4) as usize;
    img.0[i..i + 4].try_into().unwrap()
}

fn macros(b: &mut Banca) -> Vec<u64> {
    let w = b.sim.world_mut();
    let mut q = w.query::<(Entity, &Macro)>();
    q.iter(w)
        .filter(|(_, m)| !m.doc.is_empty())
        .map(|(e, _)| e.to_bits())
        .collect()
}

fn doc_de(b: &Banca, bits: u64) -> MacroDoc {
    let m = b.sim.world().get::<Macro>(Entity::from_bits(bits)).unwrap();
    ph2d_macro::codec::read(&m.doc).unwrap()
}

#[test]
fn guardar_exportar_importar_e_aplicar_novo_faz_nascer_a_imagem_do_tamanho_gravado() {
    let lib = pasta("novo");
    let (mut b, dono) = banca(96, 64, &lib);
    grava(&mut b, [10.0, 30.0], [80.0, 30.0]);
    b.m.pede(MacroRequest::Guarda { entity_bits: dono });
    b.quadro();
    assert_eq!(aviso(&b), m::MACRO_LIB_GUARDADA);
    let fora = pasta("novo-fora").join("enviada");
    let escrito =
        b.m.exporta_para("Splash", &fora, &SemMotion)
            .expect("exporta");
    assert!(escrito.to_string_lossy().ends_with(EXTENSAO));

    // Outro mundo, outra biblioteca: importar e aplicar como novo.
    let lib2 = pasta("novo-2");
    let (mut b2, _) = banca(32, 32, &lib2);
    let nome = b2.m.importa_de(&escrito).expect("importa");
    assert_eq!(nome, "Splash");
    let antes = macros(&mut b2);
    b2.m.pede(MacroRequest::Aplica {
        nome: nome.clone(),
        onde: OndeAplicar::Novo {
            em: Some([3.0, 4.0]),
        },
    });
    b2.quadro();
    assert_eq!(aviso(&b2), m::MACRO_APLICADA, "{:?}", b2.m.aviso());
    let novos: Vec<u64> = macros(&mut b2)
        .into_iter()
        .filter(|x| !antes.contains(x))
        .collect();
    assert_eq!(novos.len(), 1, "nasceu UM objecto Macro");
    let raiz = Entity::from_bits(novos[0]);
    let w = b2.sim.world();
    assert_eq!(w.get::<Name>(raiz).map(|n| n.0.as_str()), Some("Splash"));
    let t = w.get::<Transform>(raiz).unwrap();
    assert_eq!(
        (t.translation.x, t.translation.y),
        (3.0, 4.0),
        "onde se pediu"
    );
    let filhos: Vec<u64> = b2
        .imagens
        .borrow()
        .keys()
        .copied()
        .filter(|&c| w.get::<ChildOf>(Entity::from_bits(c)).map(|p| p.0) == Some(raiz))
        .collect();
    assert_eq!(filhos.len(), 1, "a imagem nasceu dentro dele");
    let filho = filhos[0];
    assert_eq!(
        w.get::<Name>(Entity::from_bits(filho))
            .map(|n| n.0.as_str()),
        Some("Tela")
    );
    let nascida = b2.imagens.borrow()[&filho].clone();
    assert_eq!((nascida.1, nascida.2), (96, 64), "do tamanho gravado");

    // O editor tem o documento do FIM da reprodução, e a imagem nasceu com os píxeis dele.
    let trilha = biblioteca::Biblioteca::carregar(&lib2)
        .procura(&nome)
        .unwrap()
        .ficheiro
        .doc
        .tracks[0]
        .clone();
    let esperado = composto(fim::documento_no_fim(&trilha, None).expect("toca"));
    let editor = b2
        .painter
        .baked_document_pixels(filho)
        .expect("o editor tem-no");
    assert_eq!(editor, esperado, "os píxeis do editor = o fim gravado");
    assert_eq!(nascida, esperado, "a imagem nasceu com eles");
    assert_ne!(px(&esperado, 40, 30), [255; 4], "a pincelada está lá");

    // A Macro instalada aponta a imagem nova, e o clipe existe.
    let d = doc_de(&b2, novos[0]);
    let sid = b2
        .sim
        .world()
        .get::<StableId>(Entity::from_bits(filho))
        .unwrap()
        .0;
    assert_eq!(d.tracks[0].target, sid);
    assert!(ph2d_app_macros::instala::clipe_de(&b2.tl.doc, novos[0]).is_some());
}

#[test]
fn aplicar_por_cima_toca_na_imagem_de_baixo() {
    let lib = pasta("por-cima");
    let (mut b, dono) = banca(64, 64, &lib);
    grava(&mut b, [10.0, 30.0], [54.0, 30.0]);
    b.m.pede(MacroRequest::Guarda { entity_bits: dono });
    b.quadro();
    assert_eq!(aviso(&b), m::MACRO_LIB_GUARDADA);

    // O alvo: uma imagem de 128×128 que o Painter já tem (a da banca nova).
    let (mut b2, _) = banca(128, 128, &lib);
    let alvo = b2.bits;
    b2.sim
        .world_mut()
        .entity_mut(Entity::from_bits(alvo))
        .remove::<ChildOf>();
    b2.m.pede(MacroRequest::Aplica {
        nome: "Splash".into(),
        onde: OndeAplicar::PorCima { alvo },
    });
    b2.quadro();
    assert_eq!(aviso(&b2), m::MACRO_APLICADA, "{:?}", b2.m.aviso());
    let raiz = *macros(&mut b2).last().unwrap();
    assert_ne!(raiz, alvo, "o objecto Macro é próprio");
    let t = &doc_de(&b2, raiz).tracks[0];
    assert_eq!(t.target, IMAGEM, "a trilha toca no alvo");
    assert!(t.nasce.is_none());
    assert_eq!(
        inicial::tamanho_gravado(&t.initial),
        Some((64, 64)),
        "a etiqueta 6 guarda a tela da gravação"
    );
    // O gizmo nasce no centro do alvo: a gravação centrada nele cai 32 px para dentro.
    let e = Encaixe {
        afim: [1.0, 0.0, 32.0, 0.0, 1.0, 32.0],
    };
    let fim_ = composto(fim::documento_no_fim(t, Some(e)).expect("toca"));
    assert_eq!(
        (fim_.1, fim_.2),
        (128, 128),
        "pinta a imagem de baixo, sem a esticar"
    );
    assert_ne!(px(&fim_, 64, 62), [255; 4], "o traço cai onde o gizmo está");
    assert_eq!(px(&fim_, 64, 30), [255; 4], "e não no canto da gravação");
}

/// Uma trilha do Painter que pára no 1.º passo (bytes que não são um passo).
fn estragada(b: &Banca) -> Track {
    let bom = doc_de(b, b.escolhido.unwrap()).tracks[0].clone();
    Track {
        events: vec![Event {
            t: MacroTime(1),
            payload: vec![0xff, 0xff, 0xff],
        }],
        edicoes: Vec::new(),
        ..bom
    }
}

#[test]
fn um_ficheiro_que_nao_toca_ate_ao_fim_nao_se_exporta() {
    let lib = pasta("estragado");
    let (mut b, _) = banca(64, 64, &lib);
    grava(&mut b, [10.0, 30.0], [54.0, 30.0]);
    let mut doc = MacroDoc::new(RefRect { w: 64.0, h: 64.0 });
    doc.tracks.push(estragada(&b));
    let f = Ficheiro {
        doc,
        modo: 0,
        nome: String::new(),
    };
    biblioteca::grava(&lib, "Partida", &f).expect("grava");
    let destino = pasta("estragado-fora").join(format!("x{EXTENSAO}"));
    let r = b.m.exporta_para("Partida", &destino, &SemMotion);
    assert_eq!(r.map_err(|e| e.chave), Err(m::MACRO_LIB_NAO_TOCA));
    assert!(!destino.exists(), "nada se escreveu");
}

#[test]
fn por_cima_misto_recusa() {
    let lib = pasta("misto");
    let (mut b, dono) = banca(64, 64, &lib);
    grava(&mut b, [10.0, 30.0], [54.0, 30.0]);
    let mut doc = doc_de(&b, dono);
    let mut motion = doc.tracks[0].clone();
    motion.family = "motion".into();
    doc.tracks.push(motion);
    let f = Ficheiro {
        doc,
        modo: 0,
        nome: String::new(),
    };
    biblioteca::grava(&lib, "Misto", &f).expect("grava");
    let antes = macros(&mut b);
    b.m.pede(MacroRequest::Aplica {
        nome: "Misto".into(),
        onde: OndeAplicar::PorCima { alvo: b.bits },
    });
    b.quadro();
    assert_eq!(aviso(&b), m::MACRO_APLICAR_MISTO);
    assert_eq!(macros(&mut b), antes, "nada nasceu");
}

#[test]
fn guardar_como_asset_e_instanciar_aplica_novo() {
    let lib = pasta("asset");
    let (mut b, dono) = banca(48, 40, &lib);
    grava(&mut b, [10.0, 20.0], [38.0, 20.0]);
    b.m.pede(MacroRequest::GuardaComoAsset { entity_bits: dono });
    b.quadro();
    assert_eq!(aviso(&b), m::MACRO_ASSET_GUARDADO);
    let receita = {
        let w = b.sim.world_mut();
        let mut q = w.query::<(Entity, &MasterRoot, &Macro)>();
        let v: Vec<u64> = q.iter(w).map(|(e, _, _)| e.to_bits()).collect();
        assert_eq!(v.len(), 1, "uma receita");
        v[0]
    };
    assert!(ph2d_app_macros::e_receita(&b.sim, receita));
    assert_eq!(
        b.sim
            .world()
            .get::<Name>(Entity::from_bits(receita))
            .map(|n| n.0.as_str()),
        Some("Splash")
    );
    let antes = macros(&mut b);
    b.m.aplica_receita(receita, Some([5.0, 0.0]), None);
    b.quadro();
    assert_eq!(aviso(&b), m::MACRO_APLICADA, "{:?}", b.m.aviso());
    let novos: Vec<u64> = macros(&mut b)
        .into_iter()
        .filter(|x| !antes.contains(x))
        .collect();
    assert_eq!(novos.len(), 1);
    let raiz = Entity::from_bits(novos[0]);
    assert!(
        b.sim.world().get::<MasterRoot>(raiz).is_none(),
        "uma cópia viva, não a receita"
    );
    let filho = b
        .imagens
        .borrow()
        .iter()
        .find(|(c, _)| {
            b.sim
                .world()
                .get::<ChildOf>(Entity::from_bits(**c))
                .map(|p| p.0)
                == Some(raiz)
        })
        .map(|(_, i)| (i.1, i.2));
    assert_eq!(filho, Some((48, 40)), "a imagem gravada nasceu dentro dela");
}

/// ⭐ Aplicar escolhe o objecto Macro que nasceu (novo e por cima), como o Add: a composição lê-o
/// uma vez depois do quadro.
#[test]
fn aplicar_escolhe_o_objeto_macro_nascido() {
    let lib = pasta("escolhe");
    let (mut b, dono) = banca(64, 64, &lib);
    grava(&mut b, [10.0, 30.0], [54.0, 30.0]);
    b.m.pede(MacroRequest::Guarda { entity_bits: dono });
    b.quadro();
    assert_eq!(b.m.toma_o_aplicado(), None, "guardar não escolhe nada");
    let antes = macros(&mut b);
    b.m.pede(MacroRequest::Aplica {
        nome: "Splash".into(),
        onde: OndeAplicar::Novo { em: None },
    });
    b.quadro();
    let novo: Vec<u64> = macros(&mut b)
        .into_iter()
        .filter(|x| !antes.contains(x))
        .collect();
    assert_eq!(b.m.toma_o_aplicado(), novo.first().copied(), "o novo");
    assert_eq!(b.m.toma_o_aplicado(), None, "uma vez");
    let antes = macros(&mut b);
    b.m.pede(MacroRequest::Aplica {
        nome: "Splash".into(),
        onde: OndeAplicar::PorCima { alvo: b.bits },
    });
    b.quadro();
    assert_eq!(aviso(&b), m::MACRO_APLICADA, "{:?}", b.m.aviso());
    let raiz: Vec<u64> = macros(&mut b)
        .into_iter()
        .filter(|x| !antes.contains(x))
        .collect();
    assert_eq!(raiz.len(), 1);
    assert_ne!(raiz[0], b.bits);
    assert_eq!(
        b.m.toma_o_aplicado(),
        Some(raiz[0]),
        "o objecto Macro por cima"
    );
}

/// ⭐ Sem nada escolhido o «Aplicar por cima» pede que se escolha uma imagem ou um objecto Motion; com
/// um objecto que não serve, diz que esse não serve.
#[test]
fn sem_escolha_o_por_cima_pede_que_se_escolha() {
    let lib = pasta("sem-escolha");
    let (mut b, _) = banca(32, 32, &lib);
    let i = b.info(None);
    assert_eq!(i.por_cima_alvo, None);
    assert_eq!(i.por_cima_recusa, Some(m::MACRO_POR_CIMA_ESCOLHA));
    assert!(ph2d_i18n::tr(m::MACRO_POR_CIMA_ESCOLHA).contains("select"));
    let vazio = b.sim.world_mut().spawn(Transform::IDENTITY).id().to_bits();
    let i = b.info(Some(vazio));
    assert_eq!(i.por_cima_recusa, Some(m::MACRO_APLICAR_INCOMPATIVEL));
    let i = b.info(Some(b.bits));
    assert_eq!(
        (i.por_cima_alvo, i.por_cima_recusa),
        (Some(b.bits), None),
        "controlo"
    );
}
