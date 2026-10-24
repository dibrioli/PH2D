//! **A sonda da cena `PH2D_MACRO_SMOKE=7`** (W18, `docs/Macros/16_W18_plano.md` §6): o MESMO roteiro
//! ([`roteiro`]), os mesmos pedidos ao painel ([`pedido`]) e a mesma pose ([`pose`]) sobre a banca sem
//! placa — o Add pela porta da família, a imagem nova com o documento ligado ao Painter, o ponteiro na
//! tela, o diálogo respondido pela porta do modal. Imprime os números que o texto ao dono cita.

use crate::gate7::{Banca, raiz};
use ph2d_app_macros::cena_da_biblioteca::{
    CORES, Estado, GRAUS, LADO, LINHAS, MAIOR, Passo, Qual, pedido, pose, roteiro,
};
use ph2d_ecs::{ChildOf, Entity, Macro, Name, Transform};
use ph2d_editor_core::tool::PointerPhase;
use ph2d_tool_painter::macro_track::{fim, inicial};
use ph2d_tool_painter::{PaintMedia, PainterTool};

const PPM: f32 = 100.0;

/// Os texels pintados (≠ branco) de uma imagem: quantos, a caixa e, pelo 2.º momento, o ângulo do eixo
/// principal (graus, y para baixo) e o comprimento ao longo dele.
#[derive(Debug)]
struct Traco {
    texels: usize,
    caixa: [u32; 4],
    graus: f64,
    comprimento: f64,
}

fn traco((px, w, _): &(Vec<u8>, u32, u32)) -> Traco {
    let pontos: Vec<(f64, f64)> = px
        .chunks_exact(4)
        .enumerate()
        .filter(|(_, c)| c[..3].iter().any(|&v| v < 240))
        .map(|(i, _)| {
            let i = u32::try_from(i).unwrap();
            (f64::from(i % w), f64::from(i / w))
        })
        .collect();
    #[expect(clippy::cast_precision_loss, reason = "texels de uma imagem")]
    let n = pontos.len() as f64;
    let (mx, my) = pontos
        .iter()
        .fold((0.0, 0.0), |(a, b), (x, y)| (a + x / n, b + y / n));
    let (mut xx, mut yy, mut xy) = (0.0, 0.0, 0.0);
    for (x, y) in &pontos {
        xx += (x - mx) * (x - mx) / n;
        yy += (y - my) * (y - my) / n;
        xy += (x - mx) * (y - my) / n;
    }
    let a = 0.5 * (2.0 * xy).atan2(xx - yy);
    let proj = pontos
        .iter()
        .map(|(x, y)| (x - mx) * a.cos() + (y - my) * a.sin());
    let (lo, hi) = proj.fold((f64::MAX, f64::MIN), |(l, h), v| (l.min(v), h.max(v)));
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "coordenadas de texel"
    )]
    let caixa = pontos.iter().fold([u32::MAX, u32::MAX, 0, 0], |c, (x, y)| {
        let (x, y) = (*x as u32, *y as u32);
        [c[0].min(x), c[1].min(y), c[2].max(x), c[3].max(y)]
    });
    Traco {
        texels: pontos.len(),
        caixa,
        graus: a.to_degrees(),
        comprimento: hi - lo,
    }
}

fn composto(doc: ph2d_tool_painter::PaintedDocument) -> (Vec<u8>, u32, u32) {
    let mut t = PainterTool::default();
    t.install_document(1, doc);
    t.baked_document_pixels(1).expect("compõe")
}

fn trilha(b: &Banca, bits: u64) -> ph2d_macro::Track {
    let m = b
        .sim
        .world()
        .get::<Macro>(Entity::from_bits(bits))
        .expect("Macro");
    ph2d_macro::codec::read(&m.doc)
        .expect("lê")
        .tracks
        .remove(0)
}

fn filho(b: &Banca, pai: u64) -> u64 {
    let pai = Some(Entity::from_bits(pai));
    let w = b.sim.world();
    let f: Vec<u64> = w
        .iter_entities()
        .filter(|e| e.get::<ChildOf>().map(ChildOf::parent) == pai)
        .map(|e| e.id().to_bits())
        .collect();
    assert_eq!(f.len(), 1, "uma imagem dentro do objecto Macro");
    f[0]
}

fn aviso(b: &Banca) -> String {
    format!("{:?}", b.m.aviso().map(|a| (a.chave, a.detalhe.clone())))
}

#[test]
#[expect(clippy::too_many_lines, reason = "o caminho inteiro, por ordem")]
fn a_cena_7_percorre_o_caminho_inteiro() {
    let mut b = Banca::nova();
    b.painter.set_paint_media(PaintMedia::Digital);
    let mut e = Estado::em(
        &std::env::temp_dir().join(format!("ph2d-macro-smoke-7-sonda-{}", std::process::id())),
    );
    b.m.com_pasta(e.biblioteca.clone());
    let (mut novo, mut por_cima, mut ultimo) = (None, None, [0.0; 2]);
    for (ms, p) in roteiro() {
        b.ate_ms(ms);
        match p {
            Passo::EscolheMacro => {
                let entry = ph2d_app_macros::object_add::MACRO;
                let bits = ph2d_app_macros::object_add::add(entry, &mut b.sim, raiz)
                    .expect("é da família")
                    .expect("nasce");
                b.escolhido = Some(bits);
            }
            Passo::NovaImagem(lado) => {
                let bits = raiz(&mut b.sim, "Canvas");
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
            Passo::Apaga => {
                let g = e.gravado.expect("o objecto gravado");
                b.sim.world_mut().despawn(Entity::from_bits(g));
                b.escolhido = None;
            }
            Passo::ImagemMaior => {
                let (w, h) = MAIOR;
                let bits = raiz(&mut b.sim, "Bigger image");
                #[expect(clippy::cast_precision_loss, reason = "lados pequenos")]
                let s = ph2d_render::Sprite::atlas(0, [w as f32 / PPM, h as f32 / PPM], [1.0; 4]);
                b.sim
                    .world_mut()
                    .entity_mut(Entity::from_bits(bits))
                    .insert(s);
                let px = vec![255; usize::try_from(w * h * 4).unwrap()];
                b.imagens.borrow_mut().insert(bits, (px, w, h));
                e.maior = Some(bits);
                b.escolhido = Some(bits);
            }
            Passo::Pose { qual, graus, px } => {
                let bits = match qual {
                    Qual::Maior => e.maior,
                    Qual::Escolhido => b.escolhido,
                }
                .expect("quem a pose muda");
                let w = b.sim.world_mut();
                let t = *w.get::<Transform>(Entity::from_bits(bits)).unwrap();
                w.entity_mut(Entity::from_bits(bits))
                    .insert(pose(t, graus, px, PPM));
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
                println!("{ms:>6} ms {p:?}: {}", aviso(&b));
                if let Some(a) = b.m.toma_o_aplicado() {
                    b.escolhido = Some(a);
                    *if p == Passo::AplicaNovo {
                        &mut novo
                    } else {
                        &mut por_cima
                    } = Some(a);
                }
            }
        }
    }

    // (5) e (6): a biblioteca e o ficheiro exportado; (8) a importada.
    let nomes = b.m.nomes_da_biblioteca();
    println!("biblioteca {}: {nomes:?}", e.biblioteca.display());
    assert_eq!(nomes.len(), 2, "a guardada e a importada");
    assert!(
        e.exportada.is_file(),
        "exportada para {}",
        e.exportada.display()
    );

    // (9) Criar novo: o objecto Macro e a imagem do tamanho gravado, com as pinceladas do fim.
    let novo = novo.expect("(9) nasceu um objecto Macro");
    let img = filho(&b, novo);
    let n = b
        .sim
        .world()
        .get::<Name>(Entity::from_bits(img))
        .map(|n| n.0.clone());
    let doc = b
        .painter
        .collect_documents(&std::collections::BTreeMap::from([(img, 0)]))
        .pop()
        .expect("o editor tem o documento da imagem nascida");
    let nove = composto(doc);
    let t9 = traco(&nove);
    println!("(9) {n:?} {}×{}: {t9:?}", nove.1, nove.2);
    assert_eq!((nove.1, nove.2), (LADO, LADO), "do tamanho gravado");
    assert!(t9.texels > 2_000, "as duas pinceladas: {t9:?}");
    assert!(t9.graus.abs() < 2.0, "horizontais: {t9:?}");
    let cor = |y: f32| {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "texel"
        )]
        let i = ((y * 256.0) as usize * 256 + 128) * 4;
        [nove.0[i], nove.0[i + 1], nove.0[i + 2]]
    };
    let (vermelha, azul) = (cor(LINHAS[0]), cor(LINHAS[1]));
    println!("(9) cor no meio da 1.ª {vermelha:?} e da 2.ª {azul:?} (pedidas {CORES:?})");
    assert!(
        vermelha[0] > vermelha[2] && azul[2] > azul[0],
        "vermelha e azul"
    );

    // (10) Por cima: a trilha toca na imagem maior, pelo encaixe vivo do gizmo rodado.
    let por_cima = por_cima.expect("(10) nasceu um objecto Macro por cima");
    let t = trilha(&b, por_cima);
    let maior = e.maior.unwrap();
    let gravado = inicial::tamanho_gravado(&t.initial).expect("o tamanho gravado");
    let pose_ = b
        .sim
        .world()
        .get::<Transform>(Entity::from_bits(por_cima))
        .copied();
    let enc = ph2d_app_painter::macro_track::encaixe_vivo::do_mundo(
        b.sim.world(),
        Entity::from_bits(por_cima),
        Entity::from_bits(maior),
        MAIOR,
        gravado,
    )
    .expect("o encaixe");
    let dez = composto(fim::documento_no_fim(&t, Some(enc)).expect("toca"));
    let t10 = traco(&dez);
    println!(
        "(10) gravado {gravado:?} sobre {}×{}; pose {:?}; {t10:?}",
        dez.1,
        dez.2,
        pose_.map(|p| (p.translation.x, p.translation.y, p.rotation.to_degrees()))
    );
    assert_eq!((dez.1, dez.2), MAIOR, "pinta a imagem maior, sem a esticar");
    assert!(
        (t10.graus.abs() - f64::from(GRAUS)).abs() < 3.0,
        "rodadas {GRAUS}°: {t10:?}"
    );
    let meio = (t10.caixa[0] + t10.caixa[2]) / 2;
    assert!(
        meio + 48 < MAIOR.0 / 2,
        "o objecto Macro andou para a esquerda: o meio das pinceladas em x = {meio}"
    );
    assert!(
        (t10.comprimento - t9.comprimento).abs() < 6.0,
        "no tamanho gravado: {} contra {}",
        t10.comprimento,
        t9.comprimento
    );
    assert!(
        (t10.texels as f64 / t9.texels as f64 - 1.0).abs() < 0.15,
        "os mesmos texels, rodados: {} contra {}",
        t10.texels,
        t9.texels
    );
}
