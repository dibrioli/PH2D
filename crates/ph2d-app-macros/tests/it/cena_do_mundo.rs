//! **A sonda da cena `PH2D_MACRO_SMOKE=8`** (W19, `docs/Macros/17_W19_plano.md` §7): o MESMO roteiro
//! ([`roteiro`]) e os mesmos pedidos ([`pedido`]) sobre a banca sem placa. O que a shell faz por fora
//! faz-se aqui pelas mesmas leis: o *Add Physics Body* põe `RigidBody` + a caixa da `Sprite` e o passo do
//! undo chega a [`ph2d_app_macros::Macros::mundo_mudou`]; a ponte corre um tique por quadro com Physics
//! armado e a tocar ([`ph2d_app_macros::Macros::fisica_correu`]); a Timeline escreve o `MacroTempo` e o
//! palco ([`PalcosDoMundo`]) conduz. Imprime os números que o texto ao dono cita.

use crate::gate7::{Banca, raiz};
use ph2d_app_macros::cena_da_biblioteca::{Estado, Passo, Qual, pedido, pose};
use ph2d_app_macros::cena_do_mundo::{CHAO, CHAO_EM_PX, LADO, roteiro};
use ph2d_app_macros::mundo::estado::Lida;
use ph2d_app_macros::mundo::formato::{Evento, FAMILIA, Mudanca};
use ph2d_app_macros::mundo::palco::{Contexto, PalcosDoMundo};
use ph2d_ecs::scene::{WorldSnapshot, world_to_snapshot};
use ph2d_ecs::{
    ChildOf, Entity, Macro, MacroTempo, Transform, TransformPropagationState, WorklistBuf,
};
use ph2d_editor_core::tool::PointerPhase;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, InputTape, PhysicsBridge, PhysicsSettings, RigidBody,
};
use ph2d_preview_drive::PreviewDrive;
use ph2d_tool_painter::{PaintMedia, PainterTool};

const PPM: f32 = 100.0;
const COLISOR: &str = "ph2d::physics::Collider";

fn foto(b: &mut Banca) -> WorldSnapshot {
    let mut prop = TransformPropagationState::new(b.sim.world_mut());
    let mut out = WorldSnapshot::new();
    world_to_snapshot(
        b.sim.world_mut(),
        &mut prop,
        &mut WorklistBuf::default(),
        &b.registo,
        &mut out,
    )
    .expect("captura");
    out
}

#[expect(clippy::cast_precision_loss, reason = "lados pequenos")]
fn sprite(b: &mut Banca, bits: u64, (w, h): (u32, u32)) {
    let s = ph2d_render::Sprite::atlas(0, [w as f32 / PPM, h as f32 / PPM], [1.0; 4]);
    b.sim
        .world_mut()
        .entity_mut(Entity::from_bits(bits))
        .insert(s);
}

/// O *Add Physics Body* (`ph2d_app_physics::physics_apply`): corpo dinâmico e a caixa da `Sprite`;
/// devolve a fotografia de depois (o baseline do undo).
fn corpo(b: &mut Banca, bits: u64) -> WorldSnapshot {
    let antes = foto(b);
    let e = Entity::from_bits(bits);
    let s = *b.sim.world().get::<ph2d_render::Sprite>(e).expect("Sprite");
    b.sim.world_mut().entity_mut(e).insert((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: s.size[0] * 0.5,
                half_y: s.size[1] * 0.5,
            },
            ..Collider::default()
        },
    ));
    let depois = foto(b);
    b.m.mundo_mudou(&antes, &depois);
    depois
}

fn apaga(b: &mut Banca, bits: u64) {
    let raiz = Entity::from_bits(bits);
    let filhos: Vec<Entity> = b
        .sim
        .world()
        .iter_entities()
        .filter(|e| e.get::<ChildOf>().map(ChildOf::parent) == Some(raiz))
        .map(|e| e.id())
        .collect();
    for f in filhos.into_iter().chain([raiz]) {
        b.sim.world_mut().despawn(f);
    }
}

fn filho(b: &Banca, pai: u64) -> u64 {
    let pai = Some(Entity::from_bits(pai));
    let f: Vec<u64> = b
        .sim
        .world()
        .iter_entities()
        .filter(|e| e.get::<ChildOf>().map(ChildOf::parent) == pai)
        .map(|e| e.id().to_bits())
        .collect();
    assert_eq!(f.len(), 1, "uma imagem dentro do objecto Macro: {f:?}");
    f[0]
}

fn trilha_do_mundo(b: &Banca, dono: u64) -> Lida {
    let m = b
        .sim
        .world()
        .get::<Macro>(Entity::from_bits(dono))
        .expect("Macro");
    let d = ph2d_macro::codec::read(&m.doc).expect("lê");
    let t = d
        .tracks
        .iter()
        .find(|t| t.family == FAMILIA)
        .expect("a trilha do mundo");
    Lida::de(t).expect("lê o mundo")
}

fn pose_de(b: &Banca, bits: u64) -> [f32; 3] {
    let t = b
        .sim
        .world()
        .get::<Transform>(Entity::from_bits(bits))
        .expect("pose");
    [t.translation.x, t.translation.y, t.rotation]
}

/// O palco toca `dono` em `s` segundos do clipe (a Timeline escreve o `MacroTempo`).
fn toca_com(
    b: &mut Banca,
    palcos: &mut PalcosDoMundo,
    drive: &mut PreviewDrive,
    (dono, s): (u64, f64),
    retoma: bool,
) {
    b.sim
        .world_mut()
        .entity_mut(Entity::from_bits(dono))
        .insert(MacroTempo { segundos: s });
    let mut cx = Contexto::novo(&b.registo, PhysicsSettings::default());
    if !retoma {
        cx.retoma = None;
    }
    palcos.segue(&mut b.sim, &cx, drive);
    drive.settle();
}

fn toca(b: &mut Banca, palcos: &mut PalcosDoMundo, drive: &mut PreviewDrive, dono: u64, s: f64) {
    toca_com(b, palcos, drive, (dono, s), true);
}

fn pintados(b: &Banca, img: u64) -> usize {
    let doc = b
        .painter
        .collect_documents(&std::collections::BTreeMap::from([(img, 0)]))
        .pop()
        .expect("o editor tem o documento da imagem");
    let mut t = PainterTool::default();
    t.install_document(1, doc);
    let (px, w, h) = t.baked_document_pixels(1).expect("compõe");
    assert_eq!((w, h), (LADO, LADO), "do tamanho gravado");
    px.as_chunks::<4>()
        .0
        .iter()
        .filter(|c| c[..3].iter().any(|&v| v < 240))
        .count()
}

/// O topo do chão `k` (m).
fn topo(k: usize) -> f32 {
    #[expect(clippy::cast_precision_loss, reason = "lado pequeno")]
    let meia = CHAO.1 as f32 / PPM / 2.0;
    CHAO_EM_PX[k][1] / PPM + meia
}

#[test]
fn a_cena_8_percorre_o_caminho_inteiro() {
    let _ = percorre();
}

/// (b) O FIM no destino é o da reprodução no destino: ao acabar o clipe o boneco fica no chão NOVO.
#[test]
fn aplicado_noutro_chao_o_fim_e_o_chao_novo() {
    let y = percorre();
    let alvo = topo(1) + meio();
    assert!(
        (y - alvo).abs() < 0.05,
        "o FIM no documento: y = {y} (o chão novo pede {alvo})"
    );
}

fn meio() -> f32 {
    #[expect(clippy::cast_precision_loss, reason = "lado pequeno")]
    let m = LADO as f32 / PPM / 2.0;
    m
}

/// O caminho inteiro; devolve a altura do FIM no documento do aplicado (antes de tocar).
#[expect(clippy::too_many_lines, reason = "o caminho inteiro, por ordem")]
fn percorre() -> f32 {
    let mut b = Banca::nova();
    b.painter.set_paint_media(PaintMedia::Digital);
    let mut e = Estado::em(
        &std::env::temp_dir().join(format!("ph2d-macro-smoke-8-sonda-{}", std::process::id())),
    );
    e.nivel = 8;
    b.m.com_pasta(e.biblioteca.clone());
    let (mut ponte, fita) = (PhysicsBridge::new(), InputTape::default());
    let (mut fisica, mut a_tocar, mut desde) = (false, false, 0u64);
    let (mut ultimo, mut novo, mut chaos) = ([0.0; 2], None, 0usize);
    let (mut caiu_na_gravacao, mut bit_na_cena) = (None, None);
    let (mut palcos, mut drive) = (PalcosDoMundo::default(), PreviewDrive::default());
    let (mut alturas, mut base, mut quadro) = (Vec::new(), None, 0u32);
    let mut fim_no_documento = None;
    for (ms, p) in roteiro() {
        // Os quadros até ao passo: a ponte (Physics + Play) e o palco do aplicado.
        while b.agora_ms() < ms {
            // O render a mais que a física: 2 tiques em cada 3 quadros (o 3.º sem tique, `ate == de`).
            quadro += 1;
            if fisica && a_tocar && novo.is_none() {
                let de = ponte.last_stepped();
                let ate = de + u64::from(quadro % 3 != 0);
                if ate > de {
                    ponte.dispatch(&mut b.sim, true, ate);
                }
                b.m.fisica_correu(&mut b.sim, Some((de, ate)), &fita);
            }
            if let Some(n) = novo.filter(|_| a_tocar) {
                #[expect(clippy::cast_precision_loss, reason = "ms da cena")]
                let s = (b.agora_ms() - desde) as f64 / 1e3;
                toca(&mut b, &mut palcos, &mut drive, n, s);
                let img = filho(&b, n);
                alturas.push((b.agora_ms(), pose_de(&b, img)[1]));
            }
            b.quadro();
        }
        match p {
            Passo::Chao(_) => {
                let bits = raiz(&mut b.sim, "Floor");
                sprite(&mut b, bits, CHAO);
                e.chao = Some(bits);
            }
            Passo::EscolheMacro => {
                let entry = ph2d_app_macros::object_add::MACRO;
                let bits = ph2d_app_macros::object_add::add(entry, &mut b.sim, raiz)
                    .expect("é da família")
                    .expect("nasce");
                b.escolhido = Some(bits);
            }
            Passo::NovaImagem(lado) => {
                let bits = raiz(&mut b.sim, "Canvas");
                sprite(&mut b, bits, (lado, lado));
                assert!(
                    b.m.nasceu(&mut b.sim, bits),
                    "nasce dentro do objecto Macro"
                );
                let n = usize::try_from(lado * lado * 4).unwrap();
                b.painter.bind_document(bits, vec![255; n], lado, lado);
                b.bits = bits;
                b.escolhido = Some(bits);
            }
            Passo::Cor(c) => b.painter.set_brush_color_srgb8(c),
            Passo::Desce(uv) | Passo::Move(uv) => {
                #[expect(clippy::cast_precision_loss, reason = "lado pequeno")]
                let pos = [uv[0] * LADO as f32, uv[1] * LADO as f32];
                let fase = if matches!(p, Passo::Desce(_)) {
                    PointerPhase::Down
                } else {
                    PointerPhase::Move
                };
                ultimo = pos;
                b.ponteiro(pos, fase);
            }
            Passo::Sobe => b.ponteiro(ultimo, PointerPhase::Up),
            Passo::Corpo(q) => {
                let bits = if q == Qual::Chao { e.chao } else { b.escolhido };
                base = Some(corpo(&mut b, bits.expect("quem ganha o corpo")));
            }
            Passo::Estatico => {
                let c = e.chao.expect("o chão");
                let w = b.sim.world_mut();
                w.entity_mut(Entity::from_bits(c)).insert(RigidBody {
                    kind: BodyKind::Static,
                });
                chaos += 1;
            }
            Passo::Pose { qual, graus, px } => {
                assert_eq!(qual, Qual::Chao);
                let bits = e.chao.expect("o chão");
                let w = b.sim.world_mut();
                let t = *w.get::<Transform>(Entity::from_bits(bits)).unwrap();
                w.entity_mut(Entity::from_bits(bits))
                    .insert(pose(t, graus, px, PPM));
            }
            Passo::Fisica => fisica = true,
            Passo::Toca => {
                a_tocar = true;
                desde = ms;
            }
            Passo::Pausa => {
                // O passo do undo que a corrida deixa ao parar: a pose do solver vira documento.
                a_tocar = false;
                if let Some(antes) = base.take() {
                    let agora = foto(&mut b);
                    b.m.mundo_mudou(&antes, &agora);
                }
            }
            Passo::Rebobina => a_tocar = false,
            Passo::Apaga => {
                let g = e.gravado.expect("o objecto gravado");
                // Antes de apagar: o palco toca o gravado no fim, sobre o chão em que caiu.
                let img = filho(&b, g);
                caiu_na_gravacao = Some(pose_de(&b, img));
                let mut p2 = PalcosDoMundo::default();
                let mut d2 = PreviewDrive::default();
                toca(&mut b, &mut p2, &mut d2, g, 30.0);
                bit_na_cena = Some(pose_de(&b, img));
                let mut p3 = PalcosDoMundo::default();
                let mut d3 = PreviewDrive::default();
                toca_com(&mut b, &mut p3, &mut d3, (g, 30.0), false);
                println!("(6) o palco sem retoma {:?}", pose_de(&b, img));
                apaga(&mut b, g);
                b.escolhido = None;
            }
            Passo::ApagaChao => {
                apaga(&mut b, e.chao.take().expect("o chão"));
            }
            _ => {
                if p == Passo::Grava {
                    e.gravado = b.escolhido;
                }
                let Some(r) = pedido(p, &mut b.m, &mut e) else {
                    continue;
                };
                b.m.pede(r);
                b.quadro();
                println!(
                    "{ms:>6} ms {p:?}: {:?}",
                    b.m.aviso().map(|a| (a.chave, a.detalhe.clone()))
                );
                if let Some(a) = b.m.toma_o_aplicado() {
                    novo = Some(a);
                    let img = filho(&b, a);
                    let w = ph2d_ecs::world_transform(b.sim.world(), Entity::from_bits(img));
                    fim_no_documento = w.map(|t| t.translation.y);
                }
            }
        }
    }
    assert_eq!(chaos, 2, "dois chãos Static");

    // (7) A trilha do mundo: o passo do collider e um `Simula`.
    let gravado = e.gravado.expect("gravou");
    let _ = gravado;
    let novo = novo.expect("(11) nasceu um objecto Macro");
    let lida = trilha_do_mundo(&b, novo);
    let com_colisor = lida.eventos.iter().filter(|(_, ev)| match ev {
        Evento::Passo { mudancas, .. } => mudancas.iter().any(|m| match m {
            Mudanca::Muda { componentes, .. } => componentes.iter().any(|c| c.0 == COLISOR),
            Mudanca::Nasce(l) => l.componentes.iter().any(|c| c.nome == COLISOR),
            Mudanca::Morre(_) => false,
        }),
        Evento::Simula(_) => false,
    });
    let t_colisor: Vec<f64> = com_colisor.map(|(t, _)| t.0 as f64 / 1e6).collect();
    let simulas: Vec<_> = lida
        .simulas()
        .map(|(t, s)| (t.0 as f64 / 1e6, s.clone()))
        .collect();
    println!(
        "(7) passos {} · o collider em {t_colisor:?} s · Simula {:?}",
        lida.eventos.len(),
        simulas
            .iter()
            .map(|(t, s)| (*t, s.de, s.ate, s.conferencia.clone()))
            .collect::<Vec<_>>()
    );
    assert_eq!(t_colisor.len(), 1, "um passo põe o collider");
    assert_eq!(simulas.len(), 1, "uma corrida da física");
    let (ts, s) = &simulas[0];
    assert!(*ts > t_colisor[0], "simula depois do collider");
    // 2,5 s com 2 tiques em cada 3 quadros: 100 tiques, num Simula só.
    assert_eq!((s.de, s.ate), (0, 100), "um Simula de 100 tiques");
    let conf = s.conferencia.first().expect("a conferência do corpo").1;

    // (6) No 1.º chão: a gravação caiu e assentou; o palco dá o bit da conferência.
    let caiu = caiu_na_gravacao.expect("caiu");
    let bit = bit_na_cena.expect("tocou");
    let meio = meio();
    println!(
        "(6) 1.º chão: topo {:.3} m · a gravação assentou em y = {:.4} · conferência {conf:?} · \
         o palco {bit:?}",
        topo(0),
        caiu[1]
    );
    assert!(
        (caiu[1] - (topo(0) + meio)).abs() < 0.05,
        "assenta no 1.º chão"
    );

    // (11) No 2.º chão: nasce, pinta-se, ganha o collider e cai — sobre o chão deste projecto.
    let img = filho(&b, novo);
    let fim = alturas.last().expect("tocou").1;
    let antes = alturas
        .iter()
        .take_while(|(ms, _)| *ms < 40_500 + 13_000)
        .last()
        .map(|a| a.1);
    let texels = pintados(&b, img);
    println!(
        "(11) 2.º chão: topo {:.3} m · antes de cair y = {antes:?} · no fim y = {fim:.4} · \
         texels pintados {texels} de {}",
        topo(1),
        LADO * LADO
    );
    assert!(
        (fim - (topo(1) + meio)).abs() < 0.05,
        "assenta no 2.º chão: {fim}"
    );
    assert!(texels > 600, "o boneco: {texels}");

    // Por último, para os números acima saírem sempre: a reprodução na cena da gravação é o bit.
    assert_eq!(
        bit.map(f32::to_bits),
        conf.map(f32::to_bits),
        "o palco recozido não dá o bit da conferência: {bit:?} contra {conf:?}"
    );
    println!("(11) o FIM no documento do aplicado: y = {fim_no_documento:?}");
    fim_no_documento.expect("o aplicado tem pose")
}
