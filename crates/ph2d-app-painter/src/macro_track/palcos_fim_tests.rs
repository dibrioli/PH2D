//! W7 item 1, `docs/Macros/04_plano_W7.md`, e W8 itens 2a/2b/4, `docs/Macros/05_plano_W8.md` — **a
//! imagem é do artista**: numa imagem que recebeu a gravação, onde a atual difere do fim a ranhura mostra
//! a atual; numa que não a recebeu, só onde difere do fim E do início; nos outros píxeis, o palco. O fim
//! é o da trilha QUE TOCA (as edições das etapas), com a água assentada.

use super::*;
use ph2d_tool_painter::macro_track::tocar::ErroDaMacro;

const LADO: u32 = 64;

/// O leitor da imagem de um sprite que a ranhura mostra.
type LeSprite<'a> = dyn FnMut(u64, &mut Placa) -> Option<(Vec<u8>, u32, u32)> + 'a;

fn premul(rgba: &[u8]) -> Vec<u8> {
    let mut v = rgba.to_vec();
    ph2d_render::premul::premultiply_rgba8(&mut v);
    v
}

/// A imagem do início das gravações no papel, premultiplicada: a branca.
fn inicio_p() -> Vec<u8> {
    vec![255; (LADO * LADO * 4) as usize]
}

/// A lei, escrita à parte: a atual onde difere do fim (e do início, se a imagem não `recebeu` a gravação),
/// senão o palco (premultiplicadas).
fn esperado(palco: &[u8], fim: &[u8], atual: &[u8], recebeu: bool) -> Vec<u8> {
    let ini = inicio_p();
    palco
        .as_chunks::<4>()
        .0
        .iter()
        .zip(fim.as_chunks::<4>().0.iter().zip(atual.as_chunks::<4>().0))
        .zip(ini.as_chunks::<4>().0)
        .flat_map(|((p, (f, a)), i)| {
            if f == a || (!recebeu && i == a) {
                p
            } else {
                a
            }
            .to_vec()
        })
        .collect()
}

/// A imagem do fim da trilha do Painter de `doc` (a que toca, a água assentada), straight.
fn fim_de(doc: &[u8]) -> Vec<u8> {
    let lido = ph2d_macro::codec::read(doc).expect("lê-se");
    ph2d_tool_painter::macro_track::etapas::imagens_da_gravacao(
        &lido.tracks[0],
        &std::sync::atomic::AtomicBool::new(false),
    )
    .expect("toca")
    .expect("não parou")
    .fim
}

/// A tela no Parar (a água ainda a correr) — o último quadro do palco —, premultiplicada.
fn no_parar_p(doc: &[u8]) -> Vec<u8> {
    let lido = ph2d_macro::codec::read(doc).expect("lê-se");
    premul(&ph2d_tool_painter::macro_track::etapas::tela_no_fim(&lido.tracks[0]).expect("toca"))
}

/// A gravação no papel (duas pinceladas de Wet Paint) e a imagem ao fim dela, pela régua das etapas.
fn gravada_no_papel() -> (Vec<u8>, f64, Vec<u8>) {
    let (doc, d) =
        ph2d_tool_painter::macro_track::roteiro::grava_no_papel(LADO, 0, 2).expect("arma");
    let fim = fim_de(&doc);
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
    fn espera_o_fim(&mut self, t: f64, painter: Option<&PainterTool>, le: &mut LeSprite<'_>) {
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
        let e = esperado(&c.palco, &fim_p, &atual_p, true);
        assert!(r == e, "(a) quadro {q}: a ranhura não segue a lei");
        misturados += usize::from(e != c.palco);
        if t > d + 0.05 {
            let corria = no_parar_p(&doc);
            assert!(
                r.as_chunks::<4>()
                    .0
                    .iter()
                    .zip(atual_p.as_chunks::<4>().0)
                    .zip(corria.as_chunks::<4>().0.iter().zip(fim_p.as_chunks::<4>().0))
                    .all(|((r, a), (p, f))| r == a || p != f),
                "(b) no último quadro a ranhura é a imagem atual, salvo a água que corria no Parar"
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
    fn o_fim(&mut self) -> (u64, std::sync::Arc<super::super::fim::Gravacao>) {
        let Some(Lido {
            palco: Ok(Some(p)), ..
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

/// W8 item 2b — ⭐ **o fim é o da trilha QUE TOCA**: editar uma etapa recalcula-o (a versão muda e a
/// imagem é, ao byte, o fim da trilha editada); mudar só o nome da etapa não. ⛔ VERMELHO com o fim da
/// trilha crua (a W7): a imagem que recebeu a edição diferia do fim e escondia o clipe editado, e o
/// último quadro saltava ao parar.
#[test]
fn o_fim_e_a_trilha_que_toca_e_editar_uma_etapa_o_recalcula() {
    let (doc, _, fim) = gravada_no_papel();
    let editada = com_a_etapa_editada(&doc);
    let fim_editado = fim_de(&editada);
    assert_ne!(fim_editado, fim, "controlo: a etapa editada muda o fim");
    let mut le = |_: u64, _: &mut Placa| Some((fim.clone(), LADO, LADO));
    let mut c = Clipe::novo(&doc);
    c.espera_o_fim(0.0, None, &mut le);
    let (versao, _) = c.o_fim();

    // Só o nome: a trilha que toca não muda.
    let mut lido = ph2d_macro::codec::read(&doc).expect("lê-se");
    let ancora = ph2d_macro::LeitorDaFamilia::etapas(
        &ph2d_tool_painter::macro_track::etapas::LeitorDoPainter,
        &lido.tracks[0].events,
    )[0]
    .inicio;
    lido.tracks[0].edicao_mut(ancora).nome = Some("céu".into());
    c.world.entity_mut(c.e).insert(Macro {
        doc: ph2d_macro::codec::write(&lido),
        modo: ModoDaMacro::Fiel,
    });
    c.quadro(
        1.0 / 60.0,
        Atuais {
            painter: None,
            le_sprite: Some(&mut le),
        },
    );
    assert!(c.o_fim_chegou(), "mudar o nome recomeçou o fio do fim");
    assert_eq!(c.o_fim().0, versao, "mudar o nome mudou a versão do fim");

    c.world.entity_mut(c.e).insert(Macro {
        doc: editada.clone(),
        modo: ModoDaMacro::Fiel,
    });
    c.quadro(
        2.0 / 60.0,
        Atuais {
            painter: None,
            le_sprite: Some(&mut le),
        },
    );
    c.espera_o_fim(2.0 / 60.0, None, &mut le);
    let (versao_depois, img) = c.o_fim();
    assert_ne!(versao_depois, versao, "editar a cor não mudou a versão do fim");
    assert!(
        img.fim.rgba == premul(&fim_editado),
        "o fim não é o da trilha editada"
    );
}

/// W8 item 2a — ⭐ **apagar até ao papel numa imagem que recebeu a gravação fica apagado**: o artista
/// devolve ao branco do início um bloco do que a macro pintou; no Play esse bloco fica branco em todo
/// quadro. ⛔ VERMELHO com a lei da W7 («≠ fim E ≠ início»): a branca é a do início e o clipe reaparecia.
#[test]
fn apagar_ate_ao_papel_numa_imagem_que_recebeu_a_gravacao_fica_apagado() {
    let (doc, d, fim) = gravada_no_papel();
    let ini = vec![255u8; (LADO * LADO * 4) as usize];
    let pintados: Vec<usize> = (0..(LADO * LADO) as usize)
        .filter(|&k| fim[k * 4..k * 4 + 4] != ini[k * 4..k * 4 + 4])
        .collect();
    assert!(pintados.len() > 40, "controlo: a gravação pinta");
    let mut atual = fim.clone();
    let apagados = &pintados[..pintados.len() / 4];
    for &k in apagados {
        atual[k * 4..k * 4 + 4].copy_from_slice(&[255; 4]);
    }
    let atual_p = premul(&atual);
    let mut le = |_: u64, _: &mut Placa| Some((atual.clone(), LADO, LADO));
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
        for &k in apagados {
            assert!(
                r[k * 4..k * 4 + 4] == [255; 4],
                "quadro {q}: o texel {k} apagado até ao papel mostra o clipe"
            );
        }
        assert!(
            r == esperado(&c.palco, &premul(&fim), &atual_p, true),
            "quadro {q}: a ranhura não segue a lei"
        );
        q += 1;
    }
}

/// O cálculo do fim que entra em pânico (o fio cai).
fn panica(_: &ph2d_macro::Track, _: &std::sync::atomic::AtomicBool) -> super::super::fim::Resultado {
    panic!("o fio do fim caiu (gate)")
}

/// O cálculo do fim cuja reprodução falha.
fn falha(_: &ph2d_macro::Track, _: &std::sync::atomic::AtomicBool) -> super::super::fim::Resultado {
    Err(ErroDaMacro::PortaRecusada("gate".into()))
}

/// W8 item 4 — **o fio do fim que cai ou falha diz-se ao artista** (o erro da macro, como o fio das
/// referências diz «macro estragada»), e a ranhura fica o palco. ⛔ VERMELHO com o `join().ok()` da W7:
/// o pânico ficava em silêncio e `erro_de` era `None`.
#[test]
fn o_fio_do_fim_que_cai_ou_falha_diz_se_ao_artista() {
    let (doc, _, fim) = gravada_no_papel();
    let mut le = |_: u64, _: &mut Placa| Some((fim.clone(), LADO, LADO));
    for (calculo, esperado_erro) in [
        (panica as super::super::fim::Calculo, ErroDaMacro::FioCaiu("macro-fim")),
        (falha, ErroDaMacro::PortaRecusada("gate".into())),
    ] {
        let mut c = Clipe::novo(&doc);
        c.palcos.calcula_o_fim = Some(calculo);
        let bits = c.e.to_bits();
        assert_eq!(c.palcos.erro_de(bits), None, "controlo: sem erro antes");
        c.espera_o_fim(0.0, None, &mut le);
        let r = c.quadro(
            1.0 / 60.0,
            Atuais {
                painter: None,
                le_sprite: Some(&mut le),
            },
        );
        assert_eq!(c.palcos.erro_de(bits), Some(&esperado_erro));
        assert!(r == c.palco, "com o fim em erro, a ranhura é o palco");
    }
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
        c.palco == no_parar_p(&doc),
        "controlo: o palco acaba na tela do Parar"
    );
    assert!(
        c.palco != premul(&fim),
        "controlo: a água corria no Parar (o fim é a assentada)"
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
        r == esperado(&c.palco, &premul(&fim), &editor_p, true),
        "(d) o traço em voo não aparece por cima do clipe"
    );
}

/// (f) — ⭐ **a imagem que nunca recebeu a gravação** (a do INÍCIO dela: as cenas `PH2D_MACRO_SMOKE=1`
/// e `=5`, uma `Macro` posta noutra imagem, a gravação desfeita): o clipe anima por cima dela e o que o
/// artista pintou nela fica por cima do clipe. ⛔ VERMELHO com a lei «atual ≠ fim ⇒ atual»: a branca
/// difere do fim em todo píxel que a macro pinta, e a ranhura ficava branca o clipe inteiro (a foto da
/// `=1` a 0,633 s, 17/10).
#[test]
fn na_imagem_do_inicio_da_gravacao_o_clipe_anima_e_a_pintura_fica_por_cima() {
    let (doc, d, fim) = gravada_no_papel();
    let atual = pintada_depois(&vec![255; (LADO * LADO * 4) as usize]);
    let (ini_p, fim_p, atual_p) = (inicio_p(), premul(&fim), premul(&atual));
    let mut le = |_: u64, _: &mut Placa| Some((atual.clone(), LADO, LADO));
    let mut c = Clipe::novo(&doc);
    c.espera_o_fim(0.0, None, &mut le);
    let pintado = (8 * LADO as usize + 8) * 4;
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
            r[pintado..pintado + 4] == atual_p[pintado..pintado + 4],
            "(f) quadro {q}: a pintura na imagem do início não está por cima do clipe"
        );
        if t > d + 0.05 {
            // O palco no último quadro é a tela do Parar (a água ainda a correr).
            let da_macro = r
                .as_chunks::<4>()
                .0
                .iter()
                .zip(
                    c.palco
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .zip(ini_p.as_chunks::<4>().0),
                )
                .filter(|(px, (p, i))| p != i && px == p)
                .count();
            assert!(
                da_macro > (LADO * LADO / 20) as usize,
                "(f) no último quadro só {da_macro} píxeis mostram a macro: a imagem do início escondeu o clipe"
            );
            assert!(
                r == esperado(&c.palco, &fim_p, &atual_p, false),
                "(f) no último quadro a ranhura não segue a lei"
            );
            break;
        }
        q += 1;
    }
    assert_ne!(ini_p, fim_p, "controlo: a gravação pinta a branca");
}


/// W8 item 6a — **a sprite sem o Painter ligado relê-se quando os píxeis dela mudam** (um Trim, um
/// filtro, o Apply de outro objecto: a identidade `SpritePixels` muda), não só quando o palco acorda; sem
/// mudança, não se relê. ⛔ VERMELHO na W7: a pintura nova só aparecia no Play seguinte.
#[test]
fn a_sprite_sem_painter_rele_se_quando_os_pixeis_dela_mudam() {
    let (doc, _, fim) = gravada_no_papel();
    let atual = pintada_depois(&fim);
    let (lidas, mudou) = (std::cell::Cell::new(0), std::cell::Cell::new(false));
    let mut le = |_: u64, _: &mut Placa| {
        lidas.set(lidas.get() + 1);
        Some((if mudou.get() { &atual } else { &fim }.clone(), LADO, LADO))
    };
    let mut c = Clipe::novo(&doc);
    let id = |b: &[u8]| ph2d_ecs::SpritePixels(ph2d_asset::AssetId::from_bytes(b));
    c.world.entity_mut(c.e).insert(id(&fim));
    c.espera_o_fim(0.0, None, &mut le);
    let quadro = |c: &mut Clipe, q: u32, le: &mut LeSprite<'_>| {
        c.quadro(
            f64::from(q) / 60.0,
            Atuais {
                painter: None,
                le_sprite: Some(le),
            },
        )
    };
    let r = quadro(&mut c, 1, &mut le);
    assert!(r == c.palco, "controlo: a sprite igual ao fim mostra o palco");
    let antes = lidas.get();
    mudou.set(true);
    c.world.entity_mut(c.e).insert(id(&atual));
    let r = quadro(&mut c, 2, &mut le);
    assert_eq!(lidas.get(), antes + 1, "os píxeis da sprite mudaram e ela não se releu");
    assert!(
        r == esperado(&c.palco, &premul(&fim), &premul(&atual), true),
        "a pintura nova da sprite não aparece por cima do clipe"
    );
    quadro(&mut c, 3, &mut le);
    assert_eq!(lidas.get(), antes + 1, "sem mudança, a sprite releu-se");
}

/// W8 item 6b — **o filme da Máscara do Painter ligado fica por cima do clipe**, como o editor o mostra
/// fora do Play: a ranhura é a mistura (sem o filme na régua da imagem atual) com o filme por cima. ⛔
/// VERMELHO na W7: a ranhura do clipe tapava a pré-visualização e o filme sumia ao dar Play.
#[test]
fn o_filme_da_mascara_do_painter_ligado_fica_por_cima_do_clipe() {
    let (doc, d, fim) = gravada_no_papel();
    let mut c = Clipe::novo(&doc);
    let mut t = PainterTool::default();
    t.bind_document(c.e.to_bits(), fim.clone(), LADO, LADO);
    let mut nunca = |_: u64, _: &mut Placa| -> Option<(Vec<u8>, u32, u32)> {
        panic!("com o Painter ligado não se lê a sprite")
    };
    c.espera_o_fim(0.0, Some(&t), &mut nunca);
    let meio = d / 2.0;
    t.handle_panel_event(ph2d_editor_core::tool::PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_PAINT_MODE,
        "mask".into(),
    ));
    t.handle_panel_event(ph2d_editor_core::tool::PanelEvent::Click(ph2d_tool_painter::ids::PAINTER_MASK_BRUSH[0]));
    for (i, phase) in [
        PointerPhase::Down,
        PointerPhase::Move,
        PointerPhase::Move,
        PointerPhase::Up,
    ]
    .into_iter()
    .enumerate()
    {
        t.on_canvas_pointer(CanvasPointer {
            pos: [LADO as f32 * (0.1 + 0.25 * i as f32), LADO as f32 * 0.5],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        });
        t.on_tick(16.0);
    }
    assert!(t.versao_da_mascara().is_some(), "controlo: a máscara pintou-se");
    let r = c.quadro(
        meio,
        Atuais {
            painter: Some(&t),
            le_sprite: Some(&mut nunca),
        },
    );
    let atual = premul(&t.imagem_aplicada().expect("documento").0);
    let sem_filme = esperado(&c.palco, &premul(&fim), &atual, true);
    let mut e = sem_filme.clone();
    ph2d_render::premul::unpremultiply_rgba8(&mut e);
    t.tinge_com_a_mascara(&mut e, (0, 0, LADO, LADO));
    ph2d_render::premul::premultiply_rgba8(&mut e);
    assert!(e != sem_filme, "controlo: o filme muda o que se vê");
    assert!(r == e, "o filme da máscara não está por cima do clipe");
}

#[path = "palcos_reescrita_tests.rs"]
mod reescrita_tests;
