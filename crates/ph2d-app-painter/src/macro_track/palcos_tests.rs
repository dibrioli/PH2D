//! Os palcos do Painter em cena: só a Timeline acorda um palco, a troca vai à sprite-alvo, uma tela
//! parada não sobe outra vez, e as ranhuras de palcos que morrem ou renascem devolvem-se.

use super::{Atuais, Lido, PalcosDoPainter, RanhurasDaCena, TrocaDaSprite};
use ph2d_ecs::{Macro, MacroTempo, ModoDaMacro, StableId, World};
use ph2d_editor_core::toast::ToastQueue;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool, Tool};
use ph2d_macro::{MacroRecorder, RefRect};
use ph2d_tool_painter::{PaintMedia, PainterTool};

const QUADRO_US: u64 = 16_666;

/// Um traço Wet Paint gravado quadro a quadro sobre a sprite `alvo`, drenando como o editor.
fn gravada(alvo: u64) -> (Vec<u8>, u64) {
    let n = 48;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (n * n * 4) as usize], n, n);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(5.0);
    let mut rec = MacroRecorder::new(
        RefRect {
            w: n as f32,
            h: n as f32,
        },
        0,
    );
    let trilha = t.macro_abre(&mut rec, alvo).expect("arma no Wet Paint");
    let mut q = 0u64;
    for i in 0..=16u32 {
        if i <= 12 {
            let f = i as f32 / 12.0;
            let phase = match i {
                0 => PointerPhase::Down,
                12 => PointerPhase::Up,
                _ => PointerPhase::Move,
            };
            t.on_canvas_pointer(CanvasPointer {
                pos: [n as f32 * (0.2 + 0.6 * f), n as f32 * 0.5],
                pressure: 1.0,
                tilt: [0.0, 0.0],
                phase,
            });
        }
        drop(t.take_preview_arc());
        t.on_tick(16.0);
        for c in t.macro_passos() {
            rec.push(trilha, q * QUADRO_US, c).expect("tempo crescente");
        }
        q += 1;
    }
    for c in t.macro_para() {
        rec.push(trilha, q * QUADRO_US, c).expect("tempo crescente");
    }
    (ph2d_macro::codec::write(&rec.finish()), q)
}

fn em(q: u64) -> MacroTempo {
    MacroTempo {
        segundos: (q * QUADRO_US) as f64 / 1e6,
    }
}

#[derive(Default)]
struct Placa {
    proxima: u32,
    adquiridas: Vec<u32>,
    substituidas: usize,
    regioes: usize,
    soltas: Vec<u32>,
    /// O que cada ranhura tem: (largura, rgba premultiplicado).
    texturas: std::collections::BTreeMap<u32, (u32, Vec<u8>)>,
}

impl RanhurasDaCena for Placa {
    fn adquire(&mut self, w: u32, _: u32, rgba: &[u8]) -> Result<u32, String> {
        self.proxima += 1;
        self.adquiridas.push(self.proxima);
        self.texturas.insert(self.proxima, (w, rgba.to_vec()));
        Ok(self.proxima)
    }

    fn substitui(&mut self, id: u32, w: u32, _: u32, rgba: &[u8]) -> Result<(), String> {
        self.substituidas += 1;
        self.texturas.insert(id, (w, rgba.to_vec()));
        Ok(())
    }

    fn substitui_regiao(
        &mut self,
        id: u32,
        (x, y, w, h): (u32, u32, u32, u32),
        rgba: &[u8],
    ) -> Result<(), String> {
        self.regioes += 1;
        let (lw, t) = self.texturas.get_mut(&id).ok_or("ranhura desconhecida")?;
        let (lw, x, w) = (*lw as usize, x as usize, w as usize);
        for (j, linha) in rgba.chunks_exact(w * 4).enumerate().take(h as usize) {
            let a = ((y as usize + j) * lw + x) * 4;
            t[a..a + w * 4].copy_from_slice(linha);
        }
        Ok(())
    }

    fn solta(&mut self, id: u32) {
        self.soltas.push(id);
    }
}

fn quadro(p: &mut PalcosDoPainter, w: &mut World, placa: &mut Placa) -> Vec<TrocaDaSprite> {
    p.segue(w);
    p.sobe(placa, &mut ToastQueue::default(), None, Atuais::default())
}

#[test]
fn so_a_timeline_acorda_o_palco_e_a_troca_vai_a_sprite_alvo() {
    let (doc, quadros) = gravada(7);
    let mut w = World::new();
    let sprite = w.spawn(StableId(7)).id();
    let dono = w
        .spawn(Macro {
            doc,
            modo: ModoDaMacro::Fiel,
        })
        .id();
    let (mut p, mut placa) = (PalcosDoPainter::default(), Placa::default());

    assert!(
        quadro(&mut p, &mut w, &mut placa).is_empty(),
        "sem relógio não toca"
    );
    assert!(placa.adquiridas.is_empty());

    w.entity_mut(dono).insert(em(quadros / 2));
    let trocas = quadro(&mut p, &mut w, &mut placa);
    assert_eq!(
        trocas,
        [TrocaDaSprite {
            entity_bits: sprite.to_bits(),
            texture_id: 1
        }]
    );
    // O mesmo instante: a tela não mudou, não sobe outra vez, e a troca continua.
    assert_eq!(quadro(&mut p, &mut w, &mut placa), trocas);
    assert_eq!(
        (placa.adquiridas.len(), placa.substituidas + placa.regioes),
        (1, 0)
    );
    // Para a frente, a pincelada sobe pela MESMA ranhura (inteira ou só o que mudou).
    w.entity_mut(dono).insert(em(quadros));
    assert_eq!(quadro(&mut p, &mut w, &mut placa), trocas);
    assert_eq!(
        (placa.adquiridas.len(), placa.substituidas + placa.regioes),
        (1, 1)
    );
    // A Timeline deixa de conduzir: a sprite volta a ser a dela, e o palco fica em cache.
    w.entity_mut(dono).remove::<MacroTempo>();
    assert!(quadro(&mut p, &mut w, &mut placa).is_empty());
    assert!(placa.soltas.is_empty());
}

#[test]
fn no_tutorial_fazer_a_macro_nao_toca_e_a_sprite_e_do_aluno() {
    let (doc, quadros) = gravada(7);
    let mut w = World::new();
    let sprite = w.spawn(StableId(7)).id();
    let dono = w
        .spawn((
            Macro {
                doc,
                modo: ModoDaMacro::Fazer,
            },
            em(quadros),
        ))
        .id();
    let (mut p, mut placa) = (PalcosDoPainter::default(), Placa::default());
    assert!(
        quadro(&mut p, &mut w, &mut placa).is_empty(),
        "com o relógio a correr, o Fazer não toca"
    );
    assert!(placa.adquiridas.is_empty());
    // Controlo: o mesmo objecto em Fiel toca e troca a sprite.
    w.entity_mut(dono).get_mut::<Macro>().expect("macro").modo = ModoDaMacro::Fiel;
    assert_eq!(
        quadro(&mut p, &mut w, &mut placa),
        [TrocaDaSprite {
            entity_bits: sprite.to_bits(),
            texture_id: 1
        }]
    );
}

#[test]
fn sem_alvo_a_macro_pinta_a_sprite_que_a_tem() {
    let (doc, quadros) = gravada(0);
    let mut w = World::new();
    let dono = w
        .spawn((
            Macro {
                doc,
                modo: ModoDaMacro::Fiel,
            },
            em(quadros),
        ))
        .id();
    let (mut p, mut placa) = (PalcosDoPainter::default(), Placa::default());
    let trocas = quadro(&mut p, &mut w, &mut placa);
    assert_eq!(trocas.len(), 1);
    assert_eq!(trocas[0].entity_bits, dono.to_bits());
}

#[test]
fn o_palco_que_muda_reedita_na_mesma_ranhura_e_o_que_morre_a_devolve() {
    let (doc, quadros) = gravada(7);
    let mut w = World::new();
    w.spawn(StableId(7));
    let m = Macro {
        doc,
        modo: ModoDaMacro::Fiel,
    };
    let dono = w.spawn((m.clone(), em(quadros))).id();
    let (mut p, mut placa) = (PalcosDoPainter::default(), Placa::default());
    quadro(&mut p, &mut w, &mut placa);
    assert_eq!(placa.adquiridas, [1]);

    // O `Macro` muda (uma etapa editada): o palco REEDITA-SE — recomeça do instantâneo — e a sprite
    // continua na mesma ranhura.
    w.entity_mut(dono).insert(m.clone());
    let trocas = quadro(&mut p, &mut w, &mut placa);
    assert_eq!(
        (placa.adquiridas.as_slice(), placa.soltas.as_slice()),
        (&[1][..], &[][..])
    );
    assert_eq!(trocas[0].texture_id, 1);
    let Some(Lido {
        palco: Ok(Some(palco)),
        ..
    }) = p.lidos.get(&dono.to_bits())
    else {
        panic!("o palco continua");
    };
    assert!(
        palco.em_cena.recomecou_de().is_some(),
        "do instantâneo, não do zero"
    );

    // Um `Macro` vazio não tem trilha: o palco sai e a ranhura volta.
    w.entity_mut(dono).insert(Macro {
        doc: Vec::new(),
        modo: ModoDaMacro::Fiel,
    });
    assert!(quadro(&mut p, &mut w, &mut placa).is_empty());
    assert_eq!(placa.soltas, [1]);

    w.entity_mut(dono).insert(m);
    quadro(&mut p, &mut w, &mut placa);
    assert_eq!(placa.adquiridas, [1, 2], "o palco nasce de novo");
    w.despawn(dono);
    assert!(quadro(&mut p, &mut w, &mut placa).is_empty());
    assert_eq!(placa.soltas, [1, 2]);
}

/// W5e — a ranhura recebe só o retângulo que mudou desde a subida anterior e fica, em cada quadro, igual
/// à tela inteira premultiplicada (a tela com PAPEL, a composição parcial).
#[test]
fn a_ranhura_recebe_so_o_que_mudou_e_fica_igual_a_tela() {
    use ph2d_tool_painter::macro_track::em_cena::MacroEmCena;
    let (doc, d) = ph2d_tool_painter::macro_track::roteiro::grava_no_papel(64, 0, 2).expect("arma");
    let mut world = World::new();
    let e = world
        .spawn((
            Macro {
                doc: doc.clone(),
                modo: ModoDaMacro::Fiel,
            },
            MacroTempo { segundos: 0.0 },
        ))
        .id();
    let mut palcos = PalcosDoPainter::default();
    let mut placa = Placa::default();
    let mut verdade = MacroEmCena::de(&doc).expect("lê-se").expect("trilha");
    let mut tela = Vec::new();
    let mut q = 0u32;
    while f64::from(q) / 60.0 <= d + 0.05 {
        let t = f64::from(q) / 60.0;
        world.get_mut::<MacroTempo>(e).expect("tempo").segundos = t;
        palcos.segue(&mut world);
        let trocas = palcos.sobe(
            &mut placa,
            &mut ToastQueue::default(),
            None,
            Atuais::default(),
        );
        if q == 1 {
            // A sprite que o Painter grava mostra o traço vivo: o palco cede-a (e só ela).
            assert!(
                palcos
                    .sobe(
                        &mut placa,
                        &mut ToastQueue::default(),
                        Some(e.to_bits()),
                        Atuais::default(),
                    )
                    .is_empty(),
                "o palco não cedeu a sprite gravada"
            );
            assert_eq!(
                palcos.sobe(
                    &mut placa,
                    &mut ToastQueue::default(),
                    Some(e.to_bits() + 1),
                    Atuais::default(),
                ),
                trocas,
                "o palco cedeu uma sprite que ninguém grava"
            );
        }
        verdade.segue(t).expect("toca");
        if let Some((a, _, _)) = verdade.toma_a_tela() {
            tela = a.to_vec();
            ph2d_render::premul::premultiply_rgba8(&mut tela);
        }
        let id = trocas[0].texture_id;
        assert!(
            placa.texturas[&id].1 == tela,
            "quadro {q}: a ranhura não é a tela"
        );
        q += 1;
    }
    // Uma composição inteira (a ferramenta invalidou a cache) sobe inteira.
    assert!(
        placa.regioes > placa.substituidas,
        "controlo: {} subidas parciais, {} inteiras depois da 1.ª",
        placa.regioes,
        placa.substituidas
    );
}

/// O recorte da subida parcial: o retângulo `(x, y, w, h)` fora do canto, linha a linha.
#[test]
fn o_recorte_da_subida_e_o_retangulo_fora_do_canto() {
    // Tela 4×3: cada texel guarda (x, y, 0, 255).
    let rgba: Vec<u8> = (0..3u8)
        .flat_map(|y| (0..4u8).flat_map(move |x| [x, y, 0, 255]))
        .collect();
    let mut out = Vec::new();
    super::recorta(&rgba, 4, (1, 1, 2, 2), &mut out);
    assert_eq!(
        out,
        [1, 1, 0, 255, 2, 1, 0, 255, 1, 2, 0, 255, 2, 2, 0, 255],
        "as linhas 1–2, colunas 1–2"
    );
}

// ── W7 item 1: a imagem é do artista ─────────────────────────────────────────────────────────────

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
