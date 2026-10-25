//! W19 (`docs/Macros/17_W19_plano.md` §2.4, §2.7): aplicar uma macro com a família «mundo» — o FIM nasce
//! com `StableId` novos e as referências entre objectos apontam os nascidos; por cima, o componente vai
//! para o alvo; a continuação de uma imagem não a faz nascer outra vez; uma família desconhecida não se
//! toca como outra; exportar lê a trilha do mundo.

use crate::aplicar::{aviso, banca, composto, doc_de, grava, macros, pasta, px};
use crate::gate7::Banca;
use ph2d_app_macros::mundo::estado::Lida;
use ph2d_app_macros::mundo::formato::{
    Componente, Evento, FAMILIA, Inicial, Linha, Mudanca, Objeto, Quem,
};
use ph2d_app_macros::{aplicar, biblioteca};
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{ChildOf, Entity, Name, StableId, World};
use ph2d_editor_core::macro_edits::{self as m, MacroRequest, OndeAplicar};
use ph2d_macro::ficheiro::Ficheiro;
use ph2d_macro::{Event, MacroDoc, MacroTime, RefRect, Track};
use ph2d_physics_ecs::{Collider, PhysicsJoint, RigidBody};
use ph2d_tool_painter::macro_track::fim;
use std::collections::BTreeMap;

const RIGIDO: &str = "ph2d::physics::RigidBody";
const COLISOR: &str = "ph2d::physics::Collider";
const JUNTA: &str = "ph2d::physics::PhysicsJoint";

/// Os bytes de `c` pelo registo (o mesmo caminho do `WorldSnapshot`).
fn blob<T: ph2d_ecs::Component>(reg: &ComponentRegistry, nome: &str, c: T) -> Componente {
    let mut w = World::new();
    let e = w.spawn(c).id();
    let bytes = (reg.get_by_name(nome).expect("registado").serialize)(&w, e)
        .expect("serializa")
        .expect("tem");
    Componente {
        nome: nome.to_owned(),
        bytes,
    }
}

fn trilha_do_mundo(inicial: &Inicial, eventos: &[(u64, Evento)]) -> Track {
    Track {
        family: FAMILIA.into(),
        target: 900,
        initial: inicial.escreve(),
        events: eventos
            .iter()
            .map(|(t, e)| Event {
                t: MacroTime(*t),
                payload: e.escreve(),
            })
            .collect(),
        edicoes: Vec::new(),
        nasce: None,
        continua: None,
    }
}

fn ficheiro(tracks: Vec<Track>, nome: &str) -> Ficheiro {
    let mut doc = MacroDoc::new(RefRect { w: 64.0, h: 64.0 });
    doc.tracks = tracks;
    Ficheiro {
        doc,
        modo: 0,
        nome: nome.into(),
    }
}

fn aplica(b: &mut Banca, lib: &std::path::Path, f: &Ficheiro, onde: OndeAplicar) -> Vec<u64> {
    biblioteca::grava(lib, &f.nome, f).expect("grava");
    let antes = macros(b);
    b.m.pede(MacroRequest::Aplica {
        nome: f.nome.clone(),
        onde,
    });
    b.quadro();
    macros(b)
        .into_iter()
        .filter(|x| !antes.contains(x))
        .collect()
}

fn sid(b: &Banca, e: Entity) -> u64 {
    b.sim.world().get::<StableId>(e).expect("tem identidade").0
}

fn filhos(b: &mut Banca, raiz: Entity) -> Vec<Entity> {
    let w = b.sim.world_mut();
    let mut q = w.query::<(Entity, &ChildOf)>();
    let mut v: Vec<Entity> = q
        .iter(w)
        .filter(|(_, p)| p.0 == raiz)
        .map(|(e, _)| e)
        .collect();
    v.sort();
    v
}

fn quantos(b: &mut Banca) -> usize {
    b.sim
        .world_mut()
        .query::<Entity>()
        .iter(b.sim.world())
        .count()
}

#[test]
fn familia_desconhecida_nao_se_aplica_como_motion() {
    let lib = pasta("mundo-xpto");
    let (mut b, _) = banca(32, 32, &lib);
    b.etapas = crate::portas_de_teste::COM_MOTION;
    let mut t = trilha_do_mundo(&Inicial::default(), &[]);
    t.family = "xpto".into();
    let antes = quantos(&mut b);
    let novos = aplica(
        &mut b,
        &lib,
        &ficheiro(vec![t], "Xpto"),
        OndeAplicar::Novo { em: None },
    );
    assert_eq!(
        aviso(&b),
        m::MACRO_FAMILIA_DESCONHECIDA,
        "{:?}",
        b.m.aviso()
    );
    assert_eq!(b.m.aviso().and_then(|a| a.detalhe.as_deref()), Some("xpto"));
    assert!(novos.is_empty());
    assert_eq!(quantos(&mut b), antes, "o mundo fica intacto");
}

#[test]
fn aplicar_novo_faz_nascer_com_ids_novos_e_juntas_remapeadas() {
    let lib = pasta("mundo-novo");
    let (mut b, _) = banca(32, 32, &lib);
    let reg = crate::portas_de_teste::registo();
    // Os `StableId` da gravação colidem de propósito com os da banca (10 = a imagem, 20 = o Splash).
    let (chao, a, bb, junta) = (10, 20, 21, 22);
    let junta_v = PhysicsJoint {
        body_a: a,
        body_b: bb,
        ..PhysicsJoint::default()
    };
    let corpo = |id: u32| Linha {
        id,
        pai: None,
        componentes: vec![
            blob(&reg, RIGIDO, RigidBody::default()),
            blob(&reg, COLISOR, Collider::default()),
        ],
    };
    let inicial = Inicial {
        objetos: vec![Objeto {
            id: 0,
            original: 900,
            quem: Quem::Macro,
        }],
        ..Inicial::default()
    };
    let obj = |id, original| Objeto {
        id,
        original,
        quem: Quem::Mundo,
    };
    let passo = Evento::Passo {
        novos: vec![obj(1, chao), obj(2, a), obj(3, bb), obj(4, junta)],
        mudancas: vec![
            Mudanca::Nasce(corpo(1)),
            Mudanca::Nasce(corpo(2)),
            Mudanca::Nasce(corpo(3)),
            Mudanca::Nasce(Linha {
                id: 4,
                pai: Some(2),
                componentes: vec![blob(&reg, JUNTA, junta_v)],
            }),
        ],
        recursos: Vec::new(),
    };
    let f = ficheiro(
        vec![trilha_do_mundo(&inicial, &[(1_000_000, passo)])],
        "Corpos",
    );
    let novos = aplica(&mut b, &lib, &f, OndeAplicar::Novo { em: None });
    assert_eq!(aviso(&b), m::MACRO_APLICADA, "{:?}", b.m.aviso());
    assert_eq!(novos.len(), 1, "um objecto Macro");
    let raiz = Entity::from_bits(novos[0]);
    let dentro = filhos(&mut b, raiz);
    assert_eq!(dentro.len(), 3, "os três corpos nascem dentro da raiz");
    let w = b.sim.world();
    let sids: Vec<u64> = dentro.iter().map(|e| sid(&b, *e)).collect();
    // Novos: nenhum é o de um objecto que a cena já tinha, e cada corpo tem o seu.
    for s in &sids {
        assert!(![10, 20].contains(s), "StableId novo: {s}");
    }
    assert!(sids[0] != sids[1] && sids[1] != sids[2] && sids[0] != sids[2]);
    for e in &dentro {
        assert!(w.get::<RigidBody>(*e).is_some() && w.get::<Collider>(*e).is_some());
    }
    // A trilha instalada fala do projecto de destino: o `original` de cada objecto é o novo.
    let d = doc_de(&b, novos[0]);
    assert_eq!(d.tracks[0].target, sid(&b, raiz));
    let lida = Lida::de(&d.tracks[0]).expect("lê");
    let o: Vec<u64> = lida.objetos().map(|o| o.original).collect();
    assert_eq!(o.len(), 5);
    assert_eq!(o[0], sid(&b, raiz));
    let mut corpos = o[1..4].to_vec();
    corpos.sort_unstable();
    let mut vivos = sids.clone();
    vivos.sort_unstable();
    assert_eq!(corpos, vivos, "os três corpos são os nascidos");
    // A junta nasce dentro do corpo `a` e aponta os corpos NOVOS — no mundo e na trilha.
    let por_sid = |b: &mut Banca, s: u64| {
        ph2d_ecs::entity_of_stable_id(b.sim.world_mut(), StableId(s)).expect("vivo")
    };
    let e_a = por_sid(&mut b, o[2]);
    let j = filhos(&mut b, e_a);
    assert_eq!(j, [por_sid(&mut b, o[4])], "a junta dentro do corpo a");
    let jv = *b.sim.world().get::<PhysicsJoint>(j[0]).expect("a junta");
    assert_eq!((jv.body_a, jv.body_b), (o[2], o[3]));
    let fim = lida.no_fim();
    let gravada: PhysicsJoint = postcard::from_bytes(&fim[&4].componentes[JUNTA]).expect("junta");
    assert_eq!((gravada.body_a, gravada.body_b), (o[2], o[3]));
}

/// A macro gravada na banca (a trilha do Painter da imagem) com uma trilha do mundo que põe um
/// `Collider` na imagem (`Quem::Trilha(0)`).
fn com_colisor(b: &Banca, dono: u64, nome: &str) -> Ficheiro {
    let reg = crate::portas_de_teste::registo();
    let pintura = doc_de(b, dono).tracks[0].clone();
    let inicial = Inicial {
        objetos: vec![
            Objeto {
                id: 0,
                original: 20,
                quem: Quem::Macro,
            },
            Objeto {
                id: 1,
                original: 10,
                quem: Quem::Trilha(0),
            },
        ],
        linhas: vec![Linha {
            id: 1,
            pai: None,
            componentes: vec![Componente {
                nome: "ph2d::ecs::Name".into(),
                bytes: postcard::to_allocvec(&Name::new("Gravada")).unwrap(),
            }],
        }],
        recursos: Vec::new(),
    };
    let passo = Evento::Passo {
        novos: Vec::new(),
        mudancas: vec![Mudanca::Muda {
            id: 1,
            pai: None,
            componentes: vec![(
                COLISOR.into(),
                Some(blob(&reg, COLISOR, Collider::default()).bytes),
            )],
        }],
        recursos: Vec::new(),
    };
    ficheiro(
        vec![pintura, trilha_do_mundo(&inicial, &[(500_000, passo)])],
        nome,
    )
}

#[test]
fn aplicar_por_cima_poe_o_collider_no_alvo() {
    let lib = pasta("mundo-por-cima");
    let (mut b, dono) = banca(64, 64, &lib);
    grava(&mut b, [10.0, 30.0], [54.0, 30.0]);
    let f = com_colisor(&b, dono, "Colide");
    let (mut b2, _) = banca(64, 64, &lib);
    let alvo = b2.bits;
    let novos = aplica(&mut b2, &lib, &f, OndeAplicar::PorCima { alvo });
    assert_eq!(aviso(&b2), m::MACRO_APLICADA, "{:?}", b2.m.aviso());
    assert_eq!(novos.len(), 1);
    let w = b2.sim.world();
    let e = Entity::from_bits(alvo);
    assert!(w.get::<Collider>(e).is_some(), "o Collider foi para o alvo");
    assert_eq!(
        w.get::<Name>(e).map(|n| n.0.as_str()),
        Some("Tela"),
        "o alvo guarda o nome"
    );
    let d = doc_de(&b2, novos[0]);
    let lida = Lida::de(&d.tracks[1]).expect("lê");
    let o: Vec<u64> = lida.objetos().map(|o| o.original).collect();
    assert_eq!(o, vec![sid(&b2, Entity::from_bits(novos[0])), sid(&b2, e)]);
    // E novo: o Collider vai para a imagem que a trilha 0 faz nascer.
    let (mut b3, _) = banca(32, 32, &lib);
    let novos = aplica(
        &mut b3,
        &lib,
        &com_colisor(&b, dono, "Colide2"),
        OndeAplicar::Novo { em: None },
    );
    assert_eq!(aviso(&b3), m::MACRO_APLICADA, "{:?}", b3.m.aviso());
    let img = filhos(&mut b3, Entity::from_bits(novos[0]));
    assert_eq!(img.len(), 1);
    assert!(b3.sim.world().get::<Collider>(img[0]).is_some());
}

#[test]
fn a_continuacao_nao_faz_nascer_a_imagem_outra_vez() {
    let lib = pasta("mundo-continua");
    let (mut b, dono) = banca(64, 64, &lib);
    grava(&mut b, [10.0, 10.0], [54.0, 10.0]);
    let primeira = doc_de(&b, dono).tracks[0].clone();
    let (mut b1, dono1) = banca(64, 64, &lib);
    grava(&mut b1, [10.0, 50.0], [54.0, 50.0]);
    let mut volta = doc_de(&b1, dono1).tracks[0].clone();
    (volta.nasce, volta.continua) = (None, Some(0));
    let f = ficheiro(vec![primeira, volta.clone()], "Volta");
    let (mut b2, _) = banca(32, 32, &lib);
    let novos = aplica(&mut b2, &lib, &f, OndeAplicar::Novo { em: None });
    assert_eq!(aviso(&b2), m::MACRO_APLICADA, "{:?}", b2.m.aviso());
    let raiz = Entity::from_bits(novos[0]);
    let img = filhos(&mut b2, raiz);
    assert_eq!(img.len(), 1, "uma imagem só");
    let s = sid(&b2, img[0]);
    let d = doc_de(&b2, novos[0]);
    assert_eq!((d.tracks[0].target, d.tracks[1].target), (s, s));
    assert_eq!(d.tracks[1].continua, Some(0));
    assert_eq!(
        aplicar::imagens_que_pinta(&b2.sim, novos[0]),
        [img[0].to_bits()]
    );
    // A imagem nasce do FIM da última trilha que a pinta.
    let esperado = composto(fim::documento_no_fim(&volta, None).expect("toca"));
    let nascida = b2.imagens.borrow()[&img[0].to_bits()].clone();
    assert_eq!(nascida, esperado);
    assert_ne!(px(&nascida, 30, 50), [255; 4], "a pincelada da volta");
}

#[test]
fn exportar_confere_a_trilha_do_mundo() {
    let lib = pasta("mundo-exporta");
    let (mut b, _) = banca(32, 32, &lib);
    let boa = trilha_do_mundo(&Inicial::default(), &[]);
    let mut ma = boa.clone();
    ma.initial = vec![0xFF, 0xFF, 0xFF];
    biblioteca::grava(&lib, "Boa", &ficheiro(vec![boa], "Boa")).expect("grava");
    biblioteca::grava(&lib, "Ma", &ficheiro(vec![ma], "Ma")).expect("grava");
    let fora = pasta("mundo-exporta-fora");
    let semm = crate::aplicar::SemMotion;
    assert!(
        b.m.exporta_para("Boa", &fora.join("boa"), &semm, &[])
            .is_ok()
    );
    let r = b.m.exporta_para("Ma", &fora.join("ma"), &semm, &[]);
    assert_eq!(r.map_err(|e| e.chave), Err(m::MACRO_LIB_ILEGIVEL));
}

/// ⭐ Plano §2.4: um objecto que nasce e é apagado a gravar não existe no FIM — não nasce no destino, e
/// o `original` dele na trilha instalada é `0` (nenhum), nunca o `StableId` da gravação (aqui `10`, o da
/// imagem da banca: o palco encontrá-la-ia). A referência que lhe apontava também fica em `0`.
#[test]
fn o_apagado_na_gravacao_nao_nasce_nem_aponta_um_alheio() {
    let lib = pasta("mundo-apagado");
    let (mut b, _) = banca(32, 32, &lib);
    let reg = crate::portas_de_teste::registo();
    let (morto, vivo, junta) = (10, 21, 22);
    let inicial = Inicial {
        objetos: vec![Objeto {
            id: 0,
            original: 900,
            quem: Quem::Macro,
        }],
        ..Inicial::default()
    };
    let obj = |id, original| Objeto {
        id,
        original,
        quem: Quem::Mundo,
    };
    let corpo = |id: u32| Linha {
        id,
        pai: None,
        componentes: vec![blob(&reg, RIGIDO, RigidBody::default())],
    };
    let nasce = Evento::Passo {
        novos: vec![obj(1, morto), obj(2, vivo), obj(3, junta)],
        mudancas: vec![
            Mudanca::Nasce(corpo(1)),
            Mudanca::Nasce(corpo(2)),
            Mudanca::Nasce(Linha {
                id: 3,
                pai: Some(2),
                componentes: vec![blob(
                    &reg,
                    JUNTA,
                    PhysicsJoint {
                        body_a: morto,
                        body_b: vivo,
                        ..PhysicsJoint::default()
                    },
                )],
            }),
        ],
        recursos: Vec::new(),
    };
    let apaga = Evento::Passo {
        novos: Vec::new(),
        mudancas: vec![Mudanca::Morre(1)],
        recursos: Vec::new(),
    };
    let f = ficheiro(
        vec![trilha_do_mundo(
            &inicial,
            &[(500_000, nasce), (900_000, apaga)],
        )],
        "Apagado",
    );
    let novos = aplica(&mut b, &lib, &f, OndeAplicar::Novo { em: None });
    assert_eq!(aviso(&b), m::MACRO_APLICADA, "{:?}", b.m.aviso());
    let raiz = Entity::from_bits(novos[0]);
    let dentro = filhos(&mut b, raiz);
    assert_eq!(dentro.len(), 1, "só o vivo nasce");
    let s_vivo = sid(&b, dentro[0]);
    let d = doc_de(&b, novos[0]);
    let lida = Lida::de(&d.tracks[0]).expect("lê");
    let o: BTreeMap<u32, u64> = lida.objetos().map(|o| (o.id, o.original)).collect();
    assert_eq!(o[&1], 0, "o apagado não tem objecto no destino");
    assert_eq!(o[&2], s_vivo, "controlo: o vivo é o nascido");
    let j = filhos(&mut b, dentro[0]);
    let jv = *b.sim.world().get::<PhysicsJoint>(j[0]).expect("a junta");
    assert_eq!(
        (jv.body_a, jv.body_b),
        (0, s_vivo),
        "a junta já não aponta o 10 da cena"
    );
    let fim = lida.no_fim();
    let gravada: PhysicsJoint = postcard::from_bytes(&fim[&3].componentes[JUNTA]).expect("junta");
    assert_eq!(
        (gravada.body_a, gravada.body_b),
        (0, s_vivo),
        "nem na trilha"
    );
}

/// ⭐ Por cima com uma continuação do Painter: a 1.ª trilha pinta em cima (y = 10), a continuação em
/// baixo (y = 50). A continuação começa onde a 1.ª acaba SOBRE O ALVO — o fim dela tem as duas
/// pinceladas; antes começava da imagem do alvo e a de cima perdia-se.
#[test]
fn por_cima_a_continuacao_continua_as_pinceladas() {
    let lib = pasta("mundo-por-cima-continua");
    let (mut b, dono) = banca(64, 64, &lib);
    grava(&mut b, [10.0, 10.0], [54.0, 10.0]);
    let primeira = doc_de(&b, dono).tracks[0].clone();
    let (mut b1, dono1) = banca(64, 64, &lib);
    grava(&mut b1, [10.0, 50.0], [54.0, 50.0]);
    let mut volta = doc_de(&b1, dono1).tracks[0].clone();
    (volta.nasce, volta.continua) = (None, Some(0));
    let f = ficheiro(vec![primeira, volta], "VoltaPorCima");
    let (mut b2, _) = banca(64, 64, &lib);
    let alvo = b2.bits;
    let novos = aplica(&mut b2, &lib, &f, OndeAplicar::PorCima { alvo });
    assert_eq!(aviso(&b2), m::MACRO_APLICADA, "{:?}", b2.m.aviso());
    let d = doc_de(&b2, novos[0]);
    let s = sid(&b2, Entity::from_bits(alvo));
    assert_eq!((d.tracks[0].target, d.tracks[1].target), (s, s));
    let a_primeira = composto(fim::documento_no_fim(&d.tracks[0], None).expect("toca"));
    assert_ne!(
        px(&a_primeira, 30, 10),
        [255; 4],
        "controlo: a 1.ª pinta em cima"
    );
    let fim_da_volta = composto(fim::documento_no_fim(&d.tracks[1], None).expect("toca"));
    assert_ne!(px(&fim_da_volta, 30, 50), [255; 4], "a pincelada da volta");
    assert_eq!(
        px(&fim_da_volta, 30, 10),
        px(&a_primeira, 30, 10),
        "a volta continua as pinceladas da 1.ª"
    );
}

/// ⭐ Ida e volta de uma imagem embutida semi-transparente: o recurso (PNG) faz nascer pela porta de
/// produto os MESMOS píxeis, ao bit — e um recurso pré-multiplicado (uma textura editada) nasce em alfa
/// recto (o atlas é recto): `rgb·255/a`, não os bytes escuros tomados por rectos.
#[test]
fn a_imagem_embutida_volta_igual_ao_bit() {
    let lib = pasta("mundo-recurso");
    let (mut b, _) = banca(32, 32, &lib);
    let reg = crate::portas_de_teste::registo();
    let recto: Vec<u8> = vec![
        200, 100, 50, 128, 10, 20, 30, 255, 255, 255, 255, 0, 90, 180, 240, 7,
    ];
    let mut pre = recto.clone();
    ph2d_render::premultiply_rgba8(&mut pre);
    let png = ph2d_app_macros::mundo::aplica::png(2, 2, &recto).expect("png");
    let png_pre = ph2d_app_macros::mundo::aplica::png(2, 2, &pre).expect("png");
    let imagem = |id: u32, ativo: [u8; 32], nome: &str| Linha {
        id,
        pai: None,
        componentes: vec![
            Componente {
                nome: "ph2d::ecs::Name".into(),
                bytes: postcard::to_allocvec(&Name::new(nome)).unwrap(),
            },
            Componente {
                nome: "ph2d::ecs::SpritePixels".into(),
                bytes: postcard::to_allocvec(&ativo).unwrap(),
            },
            blob(
                &reg,
                "ph2d::render::Sprite",
                ph2d_render::Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            ),
        ],
    };
    let (ida, ida_pre) = ([1u8; 32], [2u8; 32]);
    let inicial = Inicial {
        objetos: vec![
            Objeto {
                id: 0,
                original: 900,
                quem: Quem::Macro,
            },
            Objeto {
                id: 1,
                original: 31,
                quem: Quem::Mundo,
            },
            Objeto {
                id: 2,
                original: 32,
                quem: Quem::Mundo,
            },
        ],
        linhas: vec![imagem(1, ida, "Recta"), imagem(2, ida_pre, "Pre")],
        recursos: vec![
            ph2d_app_macros::mundo::formato::Recurso {
                id: ida,
                bytes: png,
                premultiplicado: false,
            },
            ph2d_app_macros::mundo::formato::Recurso {
                id: ida_pre,
                bytes: png_pre,
                premultiplicado: true,
            },
        ],
    };
    let f = ficheiro(vec![trilha_do_mundo(&inicial, &[])], "Recurso");
    let novos = aplica(&mut b, &lib, &f, OndeAplicar::Novo { em: None });
    assert_eq!(aviso(&b), m::MACRO_APLICADA, "{:?}", b.m.aviso());
    let dentro = filhos(&mut b, Entity::from_bits(novos[0]));
    assert_eq!(dentro.len(), 2);
    let pelo_nome = |b: &Banca, n: &str| {
        let e = dentro
            .iter()
            .find(|e| {
                b.sim
                    .world()
                    .get::<Name>(**e)
                    .is_some_and(|x| x.0.starts_with(n))
            })
            .expect("nasceu");
        b.imagens.borrow()[&e.to_bits()].clone()
    };
    assert_eq!(pelo_nome(&b, "Recta"), (recto.clone(), 2, 2), "ao bit");
    let mut de_volta = pre.clone();
    ph2d_render::unpremultiply_rgba8(&mut de_volta);
    let nascida = pelo_nome(&b, "Pre");
    assert_eq!(nascida, (de_volta, 2, 2), "a pré-multiplicada nasce recta");
    assert_ne!(nascida.0, pre, "controlo: não os bytes pré-multiplicados");
}

/// A fotografia do undo global (o `world_to_snapshot`).
fn foto(sim: &mut ph2d_ecs::SimWorld, reg: &ComponentRegistry) -> ph2d_ecs::scene::WorldSnapshot {
    let mut st = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
    let mut wl = ph2d_ecs::WorklistBuf::new();
    let mut out = ph2d_ecs::scene::WorldSnapshot::new();
    ph2d_ecs::scene::world_to_snapshot(sim.world_mut(), &mut st, &mut wl, reg, &mut out)
        .expect("foto");
    out
}

/// ⭐ Uma imagem importada de 8 bits é uma célula do ATLAS (sem `SpritePixels`): gravada só com a
/// `Sprite`, noutro projecto apontaria a célula de outra imagem, calada. Ao Parar os píxeis dela
/// embutem-se; aplicada noutro projecto, nasce pela porta de produto com os MESMOS píxeis, ao bit. Se a
/// leitura falha, a `Sprite` fica de fora e o aviso nomeia-a (nunca a célula crua).
#[test]
fn a_imagem_importada_do_atlas_volta_igual_noutro_projecto() {
    use ph2d_app_macros::mundo::{self, Leitores, grava::Gravacao};
    let reg = crate::portas_de_teste::registo();
    let mut sim = ph2d_ecs::SimWorld::new();
    let t0 = ph2d_ecs::Transform::IDENTITY;
    let raiz = sim
        .world_mut()
        .spawn((
            t0,
            Name::new("Macro"),
            ph2d_ecs::Macro::default(),
            StableId(900),
        ))
        .id();
    let antes = foto(&mut sim, &reg);
    let celula = ph2d_render::Sprite::atlas(3, [1.0, 1.0], [1.0; 4]);
    sim.world_mut().spawn((
        t0,
        Name::new("Importada"),
        StableId(901),
        ChildOf(raiz),
        celula,
    ));
    let depois = foto(&mut sim, &reg);
    let rgba: Vec<u8> = vec![
        200, 100, 50, 128, 10, 20, 30, 255, 255, 255, 255, 0, 90, 180, 240, 7,
    ];
    let vazio = MacroDoc::new(RefRect { w: 1.0, h: 1.0 });
    let fecha = |sprite: &mut dyn FnMut(u64) -> Option<Option<aplicar::Rgba>>| {
        let mut g = Gravacao::new(900);
        g.mudou(&antes, &depois, 1_000);
        let l = Leitores {
            recurso: &|_| None,
            premultiplicado: &|_| false,
            sprite,
            por_assar: None,
        };
        mundo::fecha(g, &vazio, &|_| None, &reg, l).expect("gravou")
    };
    let fe = fecha(&mut |s| (s == 901).then(|| Some((rgba.clone(), 2, 2))));
    assert!(fe.avisos.recusados.is_empty(), "{:?}", fe.avisos);
    let t = Track {
        family: FAMILIA.into(),
        target: 900,
        initial: fe.inicial,
        events: fe
            .eventos
            .into_iter()
            .map(|(us, payload)| Event {
                t: MacroTime(us),
                payload,
            })
            .collect(),
        edicoes: Vec::new(),
        nasce: None,
        continua: None,
    };
    let lib = pasta("mundo-atlas");
    let (mut b, _) = banca(32, 32, &lib);
    let novos = aplica(
        &mut b,
        &lib,
        &ficheiro(vec![t], "Atlas"),
        OndeAplicar::Novo { em: None },
    );
    assert_eq!(aviso(&b), m::MACRO_APLICADA, "{:?}", b.m.aviso());
    let dentro = filhos(&mut b, Entity::from_bits(novos[0]));
    assert_eq!(dentro.len(), 1);
    let nascida = b.imagens.borrow().get(&dentro[0].to_bits()).cloned();
    assert_eq!(
        nascida,
        Some((rgba, 2, 2)),
        "a imagem nasce dos píxeis dela, ao bit"
    );

    // A leitura falha: a `Sprite` sai da gravação e o aviso nomeia-a.
    let fe = fecha(&mut |_| Some(None));
    let a = mundo::aviso(&fe.avisos).expect("o painel di-lo");
    assert!(
        a.detalhe.as_deref().is_some_and(|d| d.contains("Sprite")),
        "{a:?}"
    );
    let t = Track {
        family: FAMILIA.into(),
        target: 900,
        initial: fe.inicial,
        events: fe
            .eventos
            .into_iter()
            .map(|(us, payload)| Event {
                t: MacroTime(us),
                payload,
            })
            .collect(),
        edicoes: Vec::new(),
        nasce: None,
        continua: None,
    };
    let fim = Lida::de(&t).expect("lê").no_fim();
    assert!(
        fim.values()
            .all(|o| !o.componentes.contains_key("ph2d::render::Sprite")),
        "nunca a célula crua"
    );
}
