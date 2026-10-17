//! W7 item 1, `docs/Macros/04_plano_W7.md` — **a imagem é do artista**: onde a imagem atual difere da
//! imagem do fim da gravação, a ranhura mostra a atual; nos outros píxeis, o palco. O fim é o da trilha
//! CRUA (editar uma etapa não o muda).

use super::*;

const LADO: u32 = 64;

fn premul(rgba: &[u8]) -> Vec<u8> {
    let mut v = rgba.to_vec();
    ph2d_render::premul::premultiply_rgba8(&mut v);
    v
}

/// A lei, escrita à parte: a atual onde difere do fim, senão o palco (as três premultiplicadas).
fn esperado(palco: &[u8], fim: &[u8], atual: &[u8]) -> Vec<u8> {
    palco
        .chunks_exact(4)
        .zip(fim.chunks_exact(4).zip(atual.chunks_exact(4)))
        .flat_map(|(p, (f, a))| if f == a { p } else { a }.to_vec())
        .collect()
}

/// A gravação no papel (duas pinceladas de Wet Paint) e a imagem ao fim dela, pela régua das etapas.
fn gravada_no_papel() -> (Vec<u8>, f64, Vec<u8>) {
    let (doc, d) =
        ph2d_tool_painter::macro_track::roteiro::grava_no_papel(LADO, 0, 2).expect("arma");
    let lido = ph2d_macro::codec::read(&doc).expect("lê-se");
    let fim = ph2d_tool_painter::macro_track::etapas::tela_no_fim(&lido.tracks[0]).expect("toca");
    (doc, d, fim)
}

/// O artista pintou à mão depois do Parar: um quadrado opaco e uma faixa a meia transparência.
fn pintada_depois(fim: &[u8]) -> Vec<u8> {
    let mut a = fim.to_vec();
    for y in 4..14usize {
        for x in 4..14usize {
            let i = (y * LADO as usize + x) * 4;
            let alfa = if y < 8 { 128 } else { 255 };
            a[i..i + 4].copy_from_slice(&[200, 30, 40, alfa]);
        }
    }
    a
}

/// O palco a seguir o clipe e a tela que ele deve mostrar (a reprodução de verdade, à parte).
struct Clipe {
    world: World,
    e: ph2d_ecs::Entity,
    palcos: PalcosDoPainter,
    placa: Placa,
    verdade: ph2d_tool_painter::macro_track::em_cena::MacroEmCena,
    palco: Vec<u8>,
}

impl Clipe {
    fn novo(doc: &[u8]) -> Self {
        let mut world = World::new();
        let e = world
            .spawn((
                Macro {
                    doc: doc.to_vec(),
                    modo: ModoDaMacro::Fiel,
                },
                MacroTempo { segundos: 0.0 },
            ))
            .id();
        let verdade = ph2d_tool_painter::macro_track::em_cena::MacroEmCena::de(doc)
            .expect("lê-se")
            .expect("trilha");
        Self {
            world,
            e,
            palcos: PalcosDoPainter::default(),
            placa: Placa::default(),
            verdade,
            palco: Vec::new(),
        }
    }

    /// Um quadro em `t`: devolve a ranhura da sprite.
    fn quadro(&mut self, t: f64, atuais: Atuais<'_, Placa>) -> Vec<u8> {
        self.world
            .get_mut::<MacroTempo>(self.e)
            .expect("tempo")
            .segundos = t;
        self.palcos.segue(&mut self.world);
        let trocas = self
            .palcos
            .sobe(&mut self.placa, &mut ToastQueue::default(), None, atuais);
        self.verdade.segue(t).expect("toca");
        if let Some((a, _, _)) = self.verdade.toma_a_tela() {
            self.palco = premul(&a);
        }
        self.placa.texturas[&trocas[0].texture_id].1.clone()
    }

    fn o_fim_chegou(&self) -> bool {
        match self.palcos.lidos.get(&self.e.to_bits()).map(|l| &l.palco) {
            Some(Ok(Some(p))) => p.fim.acabou(),
            _ => false,
        }
    }

    /// Repete o quadro em `t` até o fio do fim acabar.
    fn espera_o_fim(
        &mut self,
        t: f64,
        painter: Option<&PainterTool>,
        le: &mut dyn FnMut(u64, &mut Placa) -> Option<(Vec<u8>, u32, u32)>,
    ) {
        let t0 = std::time::Instant::now();
        while !self.o_fim_chegou() {
            assert!(t0.elapsed().as_secs() < 120, "o fim não chegou em 120 s");
            std::thread::sleep(std::time::Duration::from_millis(2));
            self.quadro(
                t,
                Atuais {
                    painter,
                    le_sprite: Some(le),
                },
            );
        }
    }
}

/// (a)(b)(c) — pintar à mão depois do Parar: antes de o fim chegar, o palco sozinho; depois, cada píxel
/// é o da imagem atual onde ela difere do fim e o do palco nos outros; no último quadro, a atual ao byte.
/// A sprite lê-se uma vez por acordar.
#[test]
fn o_que_se_pintou_depois_do_parar_aparece_por_cima_do_clipe() {
    let (doc, d, fim) = gravada_no_papel();
    let atual = pintada_depois(&fim);
    assert_ne!(atual, fim, "controlo: a pintura mudou a imagem");
    let (fim_p, atual_p) = (premul(&fim), premul(&atual));
    let lidas = std::cell::Cell::new(0);
    let mut le = |_: u64, _: &mut Placa| {
        lidas.set(lidas.get() + 1);
        Some((atual.clone(), LADO, LADO))
    };
    let mut c = Clipe::novo(&doc);
    c.palcos.segura_o_fim = true;
    for q in 0..3 {
        let r = c.quadro(
            f64::from(q) / 60.0,
            Atuais {
                painter: None,
                le_sprite: Some(&mut le),
            },
        );
        assert!(
            r == c.palco,
            "(c) quadro {q}: antes do fim, a ranhura é o palco"
        );
    }
    c.palcos.segura_o_fim = false;
    c.espera_o_fim(2.0 / 60.0, None, &mut le);
    let mut q = 3u32;
    let mut misturados = 0usize;
    loop {
        let t = f64::from(q) / 60.0;
        let r = c.quadro(
            t,
            Atuais {
                painter: None,
                le_sprite: Some(&mut le),
            },
        );
        let e = esperado(&c.palco, &fim_p, &atual_p);
        assert!(r == e, "(a) quadro {q}: a ranhura não segue a lei");
        misturados += usize::from(e != c.palco);
        if t > d + 0.05 {
            assert!(
                r == atual_p,
                "(b) no último quadro a ranhura é a imagem atual"
            );
            break;
        }
        q += 1;
    }
    assert!(misturados > 0, "controlo: a pintura mudou o que se via");
    assert_eq!(lidas.get(), 1, "a sprite lê-se uma vez por acordar");
}

impl Clipe {
    /// A versão e a imagem do fim do palco (o fio já acabou).
    fn o_fim(&mut self) -> (u64, std::sync::Arc<super::super::fim::Imagem>) {
        let Some(Lido {
            palco: Ok(Some(p)),
            ..
        }) = self.palcos.lidos.get_mut(&self.e.to_bits())
        else {
            panic!("o palco existe");
        };
        let img = std::sync::Arc::clone(p.fim.colhe().expect("o fim chegou"));
        (p.fim.versao, img)
    }
}

/// A gravação no papel com a cor da 2.ª pincelada editada (a etapa muda o clipe, não a gravação).
fn com_a_etapa_editada(doc: &[u8]) -> Vec<u8> {
    let mut lido = ph2d_macro::codec::read(doc).expect("lê-se");
    ph2d_tool_painter::macro_track::etapas::edita_cor(&mut lido.tracks[0], 1, [20, 160, 60])
        .expect("a etapa edita-se");
    ph2d_macro::codec::write(&lido)
}

/// (c) do item 1 — ⭐ **o fim é o da trilha CRUA**: com uma etapa editada, a imagem do fim é, ao byte, a
/// da trilha sem edições (o que o editor tinha no Parar), e editar uma etapa não a recalcula (a versão e
/// a imagem ficam); a pintura depois do Parar continua por cima do clipe editado, e o resto é o palco
/// (a cor nova). ⛔ VERMELHO com o fim calculado pela trilha `tocavel` (as edições aplicadas): a pincelada
/// editada difere do fim e a ranhura mostraria a cor velha da imagem atual por cima da nova.
#[test]
fn o_fim_e_a_trilha_crua_e_editar_uma_etapa_nao_o_recalcula() {
    let (doc, d, fim) = gravada_no_papel();
    let editada = com_a_etapa_editada(&doc);
    let fim_editado = ph2d_tool_painter::macro_track::etapas::tela_no_fim(
        &ph2d_macro::codec::read(&editada).expect("lê-se").tracks[0],
    )
    .expect("toca");
    assert_ne!(fim_editado, fim, "controlo: a etapa editada muda o fim do clipe");
    let atual = pintada_depois(&fim);
    let (fim_p, atual_p) = (premul(&fim), premul(&atual));
    let mut le = |_: u64, _: &mut Placa| Some((atual.clone(), LADO, LADO));

    // Editar uma etapa de um palco que já tem o fim não o recalcula.
    let mut c = Clipe::novo(&doc);
    c.espera_o_fim(0.0, None, &mut le);
    let (versao, img) = c.o_fim();
    c.world.entity_mut(c.e).insert(Macro {
        doc: editada.clone(),
        modo: ModoDaMacro::Fiel,
    });
    c.quadro(
        1.0 / 60.0,
        Atuais {
            painter: None,
            le_sprite: Some(&mut le),
        },
    );
    assert!(c.o_fim_chegou(), "editar uma etapa recomeçou o fio do fim");
    let (versao_depois, img_depois) = c.o_fim();
    assert_eq!(versao_depois, versao, "editar uma etapa mudou a versão do fim");
    assert!(
        std::sync::Arc::ptr_eq(&img_depois, &img),
        "editar uma etapa recalculou a imagem do fim"
    );

    // Um palco que nasce já com a etapa editada: a pintura depois do Parar por cima do clipe editado, e o
    // fim é o da trilha crua, ao byte.
    let mut c = Clipe::novo(&editada);
    c.espera_o_fim(0.0, None, &mut le);
    let mut q = 0u32;
    loop {
        let t = f64::from(q) / 60.0;
        let r = c.quadro(
            t,
            Atuais {
                painter: None,
                le_sprite: Some(&mut le),
            },
        );
        assert!(
            r == esperado(&c.palco, &fim_p, &atual_p),
            "quadro {q}: com a etapa editada, a ranhura não segue a lei contra o fim cru"
        );
        if t > d + 0.05 {
            assert!(
                c.palco == premul(&fim_editado),
                "controlo: o palco acaba no fim editado"
            );
            assert!(
                r != atual_p && r != c.palco,
                "no último quadro: a cor nova da etapa E a pintura depois do Parar por cima"
            );
            break;
        }
        q += 1;
    }
    let (_, img) = c.o_fim();
    assert!(
        (img.w, img.h) == (LADO, LADO) && img.rgba == fim_p,
        "o fim não é o da trilha crua (a imagem no Parar)"
    );
}

/// (e) — o controlo: sem pintura depois do Parar, a ranhura é o palco ao byte em todos os quadros.
#[test]
fn sem_pintura_depois_do_parar_a_ranhura_e_o_palco() {
    let (doc, d, fim) = gravada_no_papel();
    let mut le = |_: u64, _: &mut Placa| Some((fim.clone(), LADO, LADO));
    let mut c = Clipe::novo(&doc);
    c.espera_o_fim(0.0, None, &mut le);
    let mut q = 0u32;
    while f64::from(q) / 60.0 <= d + 0.05 {
        let r = c.quadro(
            f64::from(q) / 60.0,
            Atuais {
                painter: None,
                le_sprite: Some(&mut le),
            },
        );
        assert!(r == c.palco, "(e) quadro {q}: a ranhura não é o palco");
        q += 1;
    }
    assert!(
        c.palco == premul(&fim),
        "controlo: o palco acaba na imagem do fim"
    );
}

/// (d) — o Painter do editor ligado à sprite: o traço em voo (sem levantar) aparece por cima do clipe.
#[test]
fn o_traco_ao_vivo_do_painter_aparece_por_cima_do_clipe() {
    let (doc, d, fim) = gravada_no_papel();
    let mut c = Clipe::novo(&doc);
    let mut t = PainterTool::default();
    t.bind_document(c.e.to_bits(), fim.clone(), LADO, LADO);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_size_px(6.0);
    let mut nunca = |_: u64, _: &mut Placa| -> Option<(Vec<u8>, u32, u32)> {
        panic!("com o Painter ligado não se lê a sprite")
    };
    c.espera_o_fim(0.0, Some(&t), &mut nunca);
    let meio = d / 2.0;
    let antes = c.quadro(
        meio,
        Atuais {
            painter: Some(&t),
            le_sprite: Some(&mut nunca),
        },
    );
    assert!(
        antes == c.palco,
        "controlo: o Painter ligado sem pintar não muda o que se vê"
    );
    for (i, phase) in [PointerPhase::Down, PointerPhase::Move, PointerPhase::Move]
        .into_iter()
        .enumerate()
    {
        t.on_canvas_pointer(CanvasPointer {
            pos: [LADO as f32 * (0.3 + 0.15 * i as f32), LADO as f32 * 0.8],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        });
        t.on_tick(16.0);
    }
    // O que o editor mostra é a régua da imagem atual.
    let (editor, _, _) = t.take_preview_arc().expect("o traço mudou a tela");
    let editor_p = premul(&editor);
    assert_ne!(editor_p, premul(&fim), "controlo: o traço em voo pintou");
    let r = c.quadro(
        meio,
        Atuais {
            painter: Some(&t),
            le_sprite: Some(&mut nunca),
        },
    );
    assert!(
        r == esperado(&c.palco, &premul(&fim), &editor_p),
        "(d) o traço em voo não aparece por cima do clipe"
    );
}
