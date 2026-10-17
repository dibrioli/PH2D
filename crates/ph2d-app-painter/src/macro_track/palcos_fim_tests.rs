//! W9, `docs/Macros/07_plano_W9.md` — **a imagem é do artista pela MARCA**: onde o artista escreveu
//! depois da gravação, a ranhura mostra a imagem atual; nos outros texels, o palco. A água que corre
//! depois do Parar não marca (A1 da auditoria `06_auditoria_W8_record.md`). O oráculo de cada gate é a
//! reprodução à parte (`MacroEmCena`) e a régua do pincel/diff à parte — nunca a mistura sob teste. E o
//! fim da trilha que toca (W8 2b/4), que só a reescrita de uma etapa pede.

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, Tool};
use ph2d_tool_painter::macro_track::tocar::ErroDaMacro;
use ph2d_tool_painter::macro_track::marca::MarcaDoArtista;
use super::super::PalcoDaPintura;

const LADO: u32 = 64;

/// O leitor da imagem de um sprite que a ranhura mostra.
type LeSprite<'a> = dyn FnMut(u64, &mut Placa) -> Option<(Vec<u8>, u32, u32)> + 'a;

fn premul(rgba: &[u8]) -> Vec<u8> {
    let mut v = rgba.to_vec();
    ph2d_render::premul::premultiply_rgba8(&mut v);
    v
}

/// A lei, escrita à parte: a atual onde `marcado`, senão o palco (premultiplicadas).
fn esperado(palco: &[u8], atual: &[u8], marcado: impl Fn(usize) -> bool) -> Vec<u8> {
    palco
        .as_chunks::<4>()
        .0
        .iter()
        .zip(atual.as_chunks::<4>().0)
        .enumerate()
        .flat_map(|(i, (p, a))| if marcado(i) { a } else { p }.to_vec())
        .collect()
}

/// Os texels onde duas telas RGBA diferem.
fn diferem(a: &[u8], b: &[u8]) -> Vec<usize> {
    (0..a.len() / 4)
        .filter(|&k| a[k * 4..k * 4 + 4] != b[k * 4..k * 4 + 4])
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
        let e = world.spawn(MacroTempo { segundos: 0.0 }).id();
        Self::com(world, e, doc)
    }

    /// O clipe de `doc` posto na entidade `e` de `world` (a sprite).
    fn com(mut world: World, e: ph2d_ecs::Entity, doc: &[u8]) -> Self {
        world.entity_mut(e).insert((
            Macro {
                doc: doc.to_vec(),
                modo: ModoDaMacro::Fiel,
            },
            MacroTempo { segundos: 0.0 },
        ));
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

    /// Um quadro na ordem da shell: segue, a marca liga-se ao Painter (ou à sprite), sobe.
    fn quadro_ligado(
        &mut self,
        t: f64,
        mut painter: Option<&mut PainterTool>,
        le: &mut LeSprite<'_>,
    ) -> Vec<u8> {
        self.world
            .get_mut::<MacroTempo>(self.e)
            .expect("tempo")
            .segundos = t;
        self.palcos.segue(&mut self.world);
        self.palcos
            .liga_a_marca(painter.as_deref_mut(), &mut *le, &mut self.placa);
        let trocas = self.palcos.sobe(
            &mut self.placa,
            &mut ToastQueue::default(),
            None,
            Atuais {
                painter: painter.as_deref(),
                le_sprite: Some(le),
            },
        );
        self.verdade.segue(t).expect("toca");
        if let Some((a, _, _)) = self.verdade.toma_a_tela() {
            self.palco = premul(&a);
        }
        self.placa.texturas[&trocas[0].texture_id].1.clone()
    }

    fn palco_mut(&mut self) -> &mut PalcoDaPintura {
        match self.palcos.lidos.get_mut(&self.e.to_bits()).map(|l| &mut l.palco) {
            Some(Ok(Some(p))) => p,
            _ => panic!("o palco existe"),
        }
    }

    fn o_fim_chegou(&self) -> bool {
        match self.palcos.lidos.get(&self.e.to_bits()).map(|l| &l.palco) {
            Some(Ok(Some(p))) => p.fim.acabou(),
            _ => false,
        }
    }

    /// Repete o quadro em `t` e colhe o fio do fim (só a reescrita o pede) até ele acabar.
    fn espera_o_fim(&mut self, t: f64, painter: Option<&PainterTool>, le: &mut LeSprite<'_>) {
        let t0 = std::time::Instant::now();
        let calculo = self.palcos.calcula_o_fim;
        loop {
            self.quadro(
                t,
                Atuais {
                    painter,
                    le_sprite: Some(le),
                },
            );
            let p = self.palco_mut();
            let _ = match calculo {
                Some(f) => p.fim.colhe_com(f),
                None => p.fim.colhe(),
            };
            if self.o_fim_chegou() {
                return;
            }
            assert!(t0.elapsed().as_secs() < 120, "o fim não chegou em 120 s");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }

    /// A versão e a imagem do fim do palco (o fio já acabou).
    fn o_fim(&mut self) -> (u64, std::sync::Arc<super::super::fim::Gravacao>) {
        let p = self.palco_mut();
        let img = std::sync::Arc::clone(p.fim.colhe().expect("o fim chegou"));
        (p.fim.versao, img)
    }
}

/// Um quadro do editor (a água anda).
fn tique(t: &mut PainterTool) {
    drop(t.take_preview_arc());
    t.on_tick(16.667);
}

/// Um traço de `de` a `ate` no Painter: `fases` eventos (Down, Move…, e Up se `levanta`).
fn traco(t: &mut PainterTool, de: [f32; 2], ate: [f32; 2], fases: usize, levanta: bool) {
    for i in 0..fases {
        #[expect(clippy::cast_precision_loss, reason = "poucos passos")]
        let f = i as f32 / (fases - 1) as f32;
        let phase = match i {
            0 => PointerPhase::Down,
            i if levanta && i == fases - 1 => PointerPhase::Up,
            _ => PointerPhase::Move,
        };
        t.on_canvas_pointer(CanvasPointer {
            pos: [de[0] + (ate[0] - de[0]) * f, de[1] + (ate[1] - de[1]) * f],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        });
        tique(t);
    }
}

/// A marca do Painter (a régua dela vive nos gates do Painter) cobre toda a pintura que se vê
/// (`mudou`, o diff da imagem composta) e nada `longe` do pincel.
fn confere_a_marca(m: &MarcaDoArtista, mudou: &[usize], longe: impl Fn(usize) -> bool) {
    assert!(!mudou.is_empty(), "controlo: o traço pintou");
    let sem: Vec<&usize> = mudou.iter().filter(|&&i| !m.marcado(i)).collect();
    assert!(sem.is_empty(), "{} texels pintados sem marca", sem.len());
    let fora = (0..(LADO * LADO) as usize)
        .filter(|&i| m.marcado(i) && longe(i))
        .count();
    assert_eq!(fora, 0, "texels marcados longe do pincel");
}

/// Com o Painter ligado não se lê a sprite.
fn nunca(_: u64, _: &mut Placa) -> Option<(Vec<u8>, u32, u32)> {
    panic!("com o Painter ligado não se lê a sprite")
}

/// A1 — ⭐⭐ **com a água a correr depois do Parar, o clipe nunca mostra a imagem final**: gravado no
/// Painter LIGADO com a água no worker REAL; a cada quadro até a água parar, primeiro no Rewind (o
/// instante 0) e depois ao longo do clipe, a ranhura é o palco ao byte. ⛔ VERMELHO com a régua da W8
/// («≠ fim assentado ⇒ atual»): no Rewind a ranhura mostrava a pintura final (217 589 texels a 1024²,
/// medido na auditoria) e 80 % da animação ficava escondida. Controlo: nesses quadros o editor difere
/// do fim assentado (a régua da W8 mostrá-lo-ia).
#[test]
fn com_a_agua_a_correr_o_clipe_nunca_mostra_a_imagem_final() {
    const L: u32 = 96;
    let mut world = World::new();
    let e = world.spawn(MacroTempo { segundos: 0.0 }).id();
    let (mut t, doc, d) =
        ph2d_tool_painter::macro_track::roteiro::grava_no_papel_ligado_com_o_worker(
            L,
            e.to_bits(),
            2,
        )
        .expect("grava");
    let assentado = fim_de(&doc);
    let mut c = Clipe::com(world, e, &doc);
    let t0 = std::time::Instant::now();
    let (mut q, mut w8_esconderia) = (0u32, 0usize);
    loop {
        let parada = t.agua_parada();
        tique(&mut t);
        let instante = if q < 30 {
            0.0
        } else {
            (f64::from(q - 30) / 60.0) % (d + 0.25)
        };
        let r = c.quadro_ligado(instante, Some(&mut t), &mut nunca);
        assert!(
            r == c.palco,
            "quadro {q} (t {instante:.3} s): a ranhura não é o palco — {} texels diferem",
            diferem(&r, &c.palco).len()
        );
        if !parada {
            let editor = t.imagem_aplicada().expect("documento").0;
            w8_esconderia = w8_esconderia.max(diferem(&editor, &assentado).len());
        }
        if parada {
            break;
        }
        assert!(t0.elapsed().as_secs() < 120, "a água não parou em 120 s");
        std::thread::sleep(std::time::Duration::from_millis(1));
        q += 1;
    }
    assert!(
        w8_esconderia > 0 && q > 30,
        "controlo: a água corria depois do Parar ({q} quadros, {w8_esconderia} texels)"
    );
    let m = t.marca_do_artista().expect("a marca");
    assert!(m.vazia(), "a água marcou {} texels", m.quantos());
}

/// ⭐ **A pincelada do artista depois do Parar fica por cima do clipe, e só ela**: o traço em voo já se
/// vê (cada texel que ele mudou mostra o editor; longe do pincel, o palco); levantado, a ranhura é a lei
/// ao byte com a régua = o diff do traço (Digital); o desfazer devolve o clipe inteiro (a caixa que
/// estava misturada volta ao palco) e o refazer devolve a pincelada.
#[test]
fn a_pincelada_do_artista_fica_por_cima_do_clipe_e_o_desfazer_devolve_o_clipe() {
    let mut world = World::new();
    let e = world.spawn(MacroTempo { segundos: 0.0 }).id();
    let (mut t, doc, d) =
        ph2d_tool_painter::macro_track::roteiro::grava_no_papel_ligado(LADO, e.to_bits(), 2)
            .expect("grava");
    while !t.agua_parada() {
        tique(&mut t);
    }
    let mut c = Clipe::com(world, e, &doc);
    let meio = d / 2.0;
    let r = c.quadro_ligado(meio, Some(&mut t), &mut nunca);
    assert!(r == c.palco, "controlo: sem pintar, a ranhura é o palco");
    t.set_paint_media(ph2d_tool_painter::PaintMedia::Digital);
    t.set_brush_size_px(4.0);
    t.set_brush_color_srgb8([200, 30, 40]);
    let antes = t.imagem_aplicada().expect("documento").0;
    let (de, ate) = ([8.0, 52.0], [40.0, 52.0]);
    traco(&mut t, de, ate, 3, false);
    let r = c.quadro_ligado(meio, Some(&mut t), &mut nunca);
    let editor = premul(&t.imagem_aplicada().expect("documento").0);
    let mudou = diferem(&antes, &t.imagem_aplicada().expect("documento").0);
    assert!(!mudou.is_empty(), "controlo: o traço em voo pintou");
    for &i in &mudou {
        assert!(
            r[i * 4..i * 4 + 4] == editor[i * 4..i * 4 + 4],
            "(em voo) o texel {i} pintado não aparece por cima do clipe"
        );
    }
    let longe = |i: usize| {
        let (x, y) = ((i % LADO as usize) as f32, (i / LADO as usize) as f32);
        y < 40.0 || x > 50.0
    };
    assert!(
        (0..(LADO * LADO) as usize)
            .filter(|&i| longe(i))
            .all(|i| r[i * 4..i * 4 + 4] == c.palco[i * 4..i * 4 + 4]),
        "(em voo) longe do pincel a ranhura não é o palco"
    );
    t.on_canvas_pointer(CanvasPointer {
        pos: ate,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase: PointerPhase::Up,
    });
    tique(&mut t);
    let depois = t.imagem_aplicada().expect("documento").0;
    let m = t.marca_do_artista().expect("a marca").clone();
    confere_a_marca(&m, &diferem(&antes, &depois), longe);
    let lei = |c: &Clipe, r: &[u8]| r == esperado(&c.palco, &premul(&depois), |i| m.marcado(i));
    for q in 0..20u32 {
        let r = c.quadro_ligado(f64::from(q) / 60.0, Some(&mut t), &mut nunca);
        assert!(lei(&c, &r), "quadro {q}: a ranhura não segue a lei");
    }
    assert!(t.undo_last(), "o desfazer");
    let r = c.quadro_ligado(meio, Some(&mut t), &mut nunca);
    assert!(
        r == c.palco,
        "depois do desfazer {} texels não são o clipe",
        diferem(&r, &c.palco).len()
    );
    assert!(t.redo_last(), "o refazer");
    let r = c.quadro_ligado(meio, Some(&mut t), &mut nunca);
    assert!(lei(&c, &r), "o refazer não devolveu a pincelada");
}

/// W8 item 2a com a marca — **apagar até ao papel depois do Parar fica apagado no Play**: a borracha
/// passa sobre o que a macro pintou; em todo quadro, os texels que ela mudou mostram o editor. Controlo:
/// no fim do clipe o palco tem tinta nesses texels (o clipe reapareceria).
#[test]
fn apagar_ate_ao_papel_depois_do_parar_fica_apagado_no_play() {
    let mut world = World::new();
    let e = world.spawn(MacroTempo { segundos: 0.0 }).id();
    let (mut t, doc, d) =
        ph2d_tool_painter::macro_track::roteiro::grava_no_papel_ligado(LADO, e.to_bits(), 2)
            .expect("grava");
    while !t.agua_parada() {
        tique(&mut t);
    }
    let mut c = Clipe::com(world, e, &doc);
    c.quadro_ligado(0.0, Some(&mut t), &mut nunca);
    t.set_paint_media(ph2d_tool_painter::PaintMedia::Digital);
    t.toggle_brush_eraser();
    t.set_brush_size_px(10.0);
    let antes = t.imagem_aplicada().expect("documento").0;
    for y in [16.0, 28.0, 40.0] {
        traco(&mut t, [4.0, y], [60.0, y], 6, true);
    }
    let depois = t.imagem_aplicada().expect("documento").0;
    let apagados = diferem(&antes, &depois);
    assert!(apagados.len() > 40, "controlo: a borracha apagou");
    let mut q = 0u32;
    loop {
        let instante = f64::from(q) / 60.0;
        let r = c.quadro_ligado(instante, Some(&mut t), &mut nunca);
        let dp = premul(&depois);
        for &k in &apagados {
            assert!(
                r[k * 4..k * 4 + 4] == dp[k * 4..k * 4 + 4],
                "quadro {q}: o texel {k} apagado mostra o clipe"
            );
        }
        if instante > d + 0.05 {
            let pintados = apagados
                .iter()
                .filter(|&&k| c.palco[k * 4..k * 4 + 4] != dp[k * 4..k * 4 + 4])
                .count();
            assert!(
                pintados > 0,
                "controlo: o palco tinha tinta onde se apagou"
            );
            break;
        }
        q += 1;
    }
}

/// ⭐ **Sem o Painter, o que mudou na sprite depois de ele a largar fica por cima, e desfazer a
/// ferramenta devolve o clipe**: a 1.ª leitura é a base (nada marcado: a ranhura é o palco); os píxeis
/// mudam (uma ferramenta de imagem) e a sprite relê-se uma vez — o que mudou fica por cima; os píxeis
/// voltam (o desfazer da ferramenta) e a ranhura é o palco outra vez.
#[test]
fn sem_o_painter_o_que_mudou_na_sprite_fica_por_cima_e_desfazer_volta_ao_clipe() {
    let (doc, _, fim) = gravada_no_papel();
    let atual = pintada_depois(&fim);
    let (lidas, qual) = (std::cell::Cell::new(0), std::cell::Cell::new(0u8));
    let mut le = |_: u64, _: &mut Placa| {
        lidas.set(lidas.get() + 1);
        Some((if qual.get() == 1 { &atual } else { &fim }.clone(), LADO, LADO))
    };
    let mut c = Clipe::novo(&doc);
    let id = |b: &[u8]| ph2d_ecs::SpritePixels(ph2d_asset::AssetId::from_bytes(b));
    c.world.entity_mut(c.e).insert(id(&fim));
    let r = c.quadro_ligado(1.0 / 60.0, None, &mut le);
    assert!(r == c.palco, "a base não marca nada");
    let antes = lidas.get();
    qual.set(1);
    c.world.entity_mut(c.e).insert(id(&atual));
    let r = c.quadro_ligado(2.0 / 60.0, None, &mut le);
    assert_eq!(lidas.get(), antes + 1, "os píxeis mudaram e a sprite não se releu");
    let mudou = diferem(&fim, &atual);
    assert!(
        r == esperado(&c.palco, &premul(&atual), |i| mudou.contains(&i)),
        "o que a ferramenta mudou não aparece por cima do clipe"
    );
    c.quadro_ligado(3.0 / 60.0, None, &mut le);
    assert_eq!(lidas.get(), antes + 1, "sem mudança, a sprite releu-se");
    qual.set(0);
    c.world.entity_mut(c.e).insert(id(&fim));
    let r = c.quadro_ligado(4.0 / 60.0, None, &mut le);
    assert!(r == c.palco, "o desfazer da ferramenta não devolveu o clipe");
}

/// **Gravar de novo começa outra marca; desfazer a gravação devolve a antiga**: com uma pincelada
/// marcada, outra gravação na sprite deixa a ranhura no palco dela (nada marcado nessa época); a
/// gravação antiga de volta (o desfazer global devolve a `Macro`) mostra a pincelada outra vez.
#[test]
fn gravar_de_novo_comeca_outra_marca_e_desfazer_a_gravacao_devolve_a_antiga() {
    let mut world = World::new();
    let e = world.spawn(MacroTempo { segundos: 0.0 }).id();
    let (mut t, doc, _) =
        ph2d_tool_painter::macro_track::roteiro::grava_no_papel_ligado(LADO, e.to_bits(), 2)
            .expect("grava");
    while !t.agua_parada() {
        tique(&mut t);
    }
    let mut c = Clipe::com(world, e, &doc);
    c.quadro_ligado(0.1, Some(&mut t), &mut nunca);
    t.set_paint_media(ph2d_tool_painter::PaintMedia::Digital);
    let antes = t.imagem_aplicada().expect("documento").0;
    traco(&mut t, [8.0, 54.0], [50.0, 54.0], 4, true);
    let depois = t.imagem_aplicada().expect("documento").0;
    let m = t.marca_do_artista().expect("a marca").clone();
    confere_a_marca(&m, &diferem(&antes, &depois), |i| i < 40 * LADO as usize);
    let r = c.quadro_ligado(0.1, Some(&mut t), &mut nunca);
    assert!(r != c.palco, "controlo: a pincelada aparece por cima");
    let (outra, _) =
        ph2d_tool_painter::macro_track::roteiro::grava_no_papel(LADO, 0, 3).expect("grava");
    c.world.entity_mut(c.e).insert(Macro {
        doc: outra.clone(),
        modo: ModoDaMacro::Fiel,
    });
    c.verdade = ph2d_tool_painter::macro_track::em_cena::MacroEmCena::de(&outra)
        .expect("lê-se")
        .expect("trilha");
    let r = c.quadro_ligado(0.1, Some(&mut t), &mut nunca);
    assert!(r == c.palco, "a gravação nova herdou a marca da antiga");
    assert!(t.marca_do_artista().is_some_and(MarcaDoArtista::vazia));
    c.world.entity_mut(c.e).insert(Macro {
        doc: doc.clone(),
        modo: ModoDaMacro::Fiel,
    });
    c.verdade = ph2d_tool_painter::macro_track::em_cena::MacroEmCena::de(&doc)
        .expect("lê-se")
        .expect("trilha");
    let r = c.quadro_ligado(0.1, Some(&mut t), &mut nunca);
    assert!(
        r == esperado(&c.palco, &premul(&depois), |i| m.marcado(i)),
        "a gravação antiga de volta não devolveu a marca dela"
    );
}

/// **A marca grava-se no projeto e volta igual**: a pincelada marcada no Painter; as marcas gravadas e
/// abertas noutro palco (outro processo: a sprite com o MESMO `StableId`, sem o Painter) dão a mesma
/// mistura — a imagem atual onde a pincelada mudou, o palco no resto. ⛔ Sem gravar, abrir o projeto
/// marcava tudo como da macro e a pincelada sumia no Play.
#[test]
fn a_marca_gravada_no_projeto_volta_igual() {
    let mut world = World::new();
    let e = world
        .spawn((MacroTempo { segundos: 0.0 }, StableId(42)))
        .id();
    let (mut t, doc, _) =
        ph2d_tool_painter::macro_track::roteiro::grava_no_papel_ligado(LADO, e.to_bits(), 2)
            .expect("grava");
    while !t.agua_parada() {
        tique(&mut t);
    }
    let mut c = Clipe::com(world, e, &doc);
    c.quadro_ligado(0.1, Some(&mut t), &mut nunca);
    t.set_paint_media(ph2d_tool_painter::PaintMedia::Digital);
    let antes = t.imagem_aplicada().expect("documento").0;
    traco(&mut t, [10.0, 20.0], [50.0, 44.0], 4, true);
    let depois = t.imagem_aplicada().expect("documento").0;
    let m = t.marca_do_artista().expect("a marca").clone();
    confere_a_marca(&m, &diferem(&antes, &depois), |i| i < 8 * LADO as usize);
    c.quadro_ligado(0.1, Some(&mut t), &mut nunca);
    let bytes = c.palcos.marcas_a_gravar(Some(&t));
    assert!(!bytes.is_empty(), "nada a gravar");

    let mut w2 = World::new();
    let e2 = w2.spawn((MacroTempo { segundos: 0.0 }, StableId(42))).id();
    let mut aberto = Clipe::com(w2, e2, &doc);
    assert!(aberto.palcos.restaura_marcas(&bytes), "as marcas não se leram");
    let mut le = |_: u64, _: &mut Placa| Some((depois.clone(), LADO, LADO));
    let r = aberto.quadro_ligado(0.1, None, &mut le);
    assert!(
        r == esperado(&aberto.palco, &premul(&depois), |i| m.marcado(i)),
        "a marca aberta não é a gravada"
    );
    assert!(!aberto.palcos.restaura_marcas(&bytes[..bytes.len() - 1]));
}

/// A3 — **a água sem marca não lê a imagem nem sobe a ranhura**: com a água do editor a correr depois do
/// Parar e o clipe parado, nenhum quadro relê a imagem do Painter e nenhuma subida acontece; com uma
/// pincelada marcada, cada quadro da água sobe só uma caixa dentro da marca, nunca a ranhura inteira. ⛔
/// A W8 relia a imagem e subia a ranhura INTEIRA a cada quadro da água (6,6 ms por quadro, auditoria A3).
#[test]
fn a_agua_sem_marca_nao_le_nem_sobe_e_com_marca_sobe_so_a_caixa() {
    const L: u32 = 96;
    let mut world = World::new();
    let e = world.spawn(MacroTempo { segundos: 0.0 }).id();
    let (mut t, doc, _) =
        ph2d_tool_painter::macro_track::roteiro::grava_no_papel_ligado_com_o_worker(
            L,
            e.to_bits(),
            2,
        )
        .expect("grava");
    let mut c = Clipe::com(world, e, &doc);
    for _ in 0..3 {
        tique(&mut t);
        c.quadro_ligado(0.0, Some(&mut t), &mut nunca);
    }
    let (leituras, inteiras, regioes) = (
        c.palco_mut().versao_da_atual,
        c.placa.substituidas,
        c.placa.regioes,
    );
    let editor0 = t.imagem_aplicada().expect("documento").0;
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(2));
        tique(&mut t);
        c.quadro_ligado(0.0, Some(&mut t), &mut nunca);
    }
    assert!(
        !diferem(&editor0, &t.imagem_aplicada().expect("documento").0).is_empty(),
        "controlo: a água correu"
    );
    assert_eq!(
        (c.palco_mut().versao_da_atual, c.placa.substituidas, c.placa.regioes),
        (leituras, inteiras, regioes),
        "sem marca, a água leu a imagem ou subiu a ranhura"
    );
    // Uma pincelada (Wet Paint, a água continua) no canto.
    t.set_brush_size_px(4.0);
    traco(&mut t, [8.0, 86.0], [30.0, 86.0], 4, true);
    let caixa = t
        .marca_do_artista()
        .and_then(MarcaDoArtista::caixa)
        .expect("a pincelada marcou");
    let inteiras = c.placa.substituidas;
    c.placa.caixas.clear();
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(2));
        tique(&mut t);
        c.quadro_ligado(0.0, Some(&mut t), &mut nunca);
    }
    assert_eq!(c.placa.substituidas, inteiras, "a ranhura subiu inteira");
    assert!(!c.placa.caixas.is_empty(), "controlo: a mistura subiu");
    assert_eq!(
        t.marca_do_artista().and_then(MarcaDoArtista::caixa),
        Some(caixa),
        "a água mexeu na marca"
    );
    let dentro = |&(x, y, w, h): &(u32, u32, u32, u32)| {
        x >= caixa.0 && y >= caixa.1 && x + w <= caixa.0 + caixa.2 && y + h <= caixa.1 + caixa.3
    };
    assert!(
        c.placa.caixas.iter().all(dentro),
        "uma subida saiu da caixa da marca: {:?} × {caixa:?}",
        c.placa.caixas
    );
}

impl Clipe {
    fn com_o_fim(doc: &[u8], le: &mut LeSprite<'_>) -> Self {
        let mut c = Self::novo(doc);
        c.espera_o_fim(0.0, None, le);
        c
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
/// imagem é, ao byte, o fim da trilha editada); mudar só o nome da etapa não.
#[test]
fn o_fim_e_a_trilha_que_toca_e_editar_uma_etapa_o_recalcula() {
    let (doc, _, fim) = gravada_no_papel();
    let editada = com_a_etapa_editada(&doc);
    let fim_editado = fim_de(&editada);
    assert_ne!(fim_editado, fim, "controlo: a etapa editada muda o fim");
    let mut le = |_: u64, _: &mut Placa| Some((fim.clone(), LADO, LADO));
    let mut c = Clipe::com_o_fim(&doc, &mut le);
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
    c.espera_o_fim(2.0 / 60.0, None, &mut le);
    let (versao_depois, img) = c.o_fim();
    assert_ne!(
        versao_depois, versao,
        "editar a cor não mudou a versão do fim"
    );
    assert!(
        img.fim.rgba == premul(&fim_editado),
        "o fim não é o da trilha editada"
    );
}

/// O cálculo do fim que entra em pânico (o fio cai).
fn panica(
    _: &ph2d_macro::Track,
    _: &std::sync::atomic::AtomicBool,
) -> super::super::fim::Resultado {
    panic!("o fio do fim caiu (gate)")
}

/// O cálculo do fim cuja reprodução falha.
fn falha(_: &ph2d_macro::Track, _: &std::sync::atomic::AtomicBool) -> super::super::fim::Resultado {
    Err(ErroDaMacro::PortaRecusada("gate".into()))
}

/// W8 item 4 — **o fio do fim que cai ou falha diz-se ao artista** (o erro da macro), e a ranhura fica
/// o palco. ⛔ VERMELHO com o `join().ok()` da W7: o pânico ficava em silêncio e `erro_de` era `None`.
#[test]
fn o_fio_do_fim_que_cai_ou_falha_diz_se_ao_artista() {
    let (doc, _, fim) = gravada_no_papel();
    let mut le = |_: u64, _: &mut Placa| Some((fim.clone(), LADO, LADO));
    for (calculo, esperado_erro) in [
        (
            panica as super::super::fim::Calculo,
            ErroDaMacro::FioCaiu("macro-fim"),
        ),
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

/// (e) — o controlo: sem nada marcado, a ranhura é o palco ao byte em todos os quadros, e o palco
/// acaba na tela do Parar (o salto no fim do clipe é a água que correu depois, decidido na W8 2c).
#[test]
fn sem_marca_a_ranhura_e_o_palco() {
    let (doc, d, fim) = gravada_no_papel();
    let mut le = |_: u64, _: &mut Placa| Some((fim.clone(), LADO, LADO));
    let mut c = Clipe::novo(&doc);
    let mut q = 0u32;
    while f64::from(q) / 60.0 <= d + 0.05 {
        let r = c.quadro_ligado(f64::from(q) / 60.0, None, &mut le);
        assert!(r == c.palco, "(e) quadro {q}: a ranhura não é o palco");
        q += 1;
    }
    assert!(
        c.palco == no_parar_p(&doc),
        "controlo: o palco acaba na tela do Parar"
    );
}

/// W8 item 6b — **o filme da Máscara do Painter ligado fica por cima do clipe**: pintar a máscara não é
/// pintar a imagem (não marca); a ranhura é o palco com o filme por cima, e mudar só a cor do filme sobe.
#[test]
fn o_filme_da_mascara_do_painter_ligado_fica_por_cima_do_clipe() {
    let mut world = World::new();
    let e = world.spawn(MacroTempo { segundos: 0.0 }).id();
    let (mut t, doc, d) =
        ph2d_tool_painter::macro_track::roteiro::grava_no_papel_ligado(LADO, e.to_bits(), 2)
            .expect("grava");
    while !t.agua_parada() {
        tique(&mut t);
    }
    let mut c = Clipe::com(world, e, &doc);
    let meio = d / 2.0;
    c.quadro_ligado(meio, Some(&mut t), &mut nunca);
    t.handle_panel_event(ph2d_editor_core::tool::PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_PAINT_MODE,
        "mask".into(),
    ));
    t.handle_panel_event(ph2d_editor_core::tool::PanelEvent::Click(
        ph2d_tool_painter::ids::PAINTER_MASK_BRUSH[0],
    ));
    traco(&mut t, [6.0, 32.0], [58.0, 32.0], 4, true);
    assert!(
        t.versao_da_mascara().is_some(),
        "controlo: a máscara pintou-se"
    );
    let r = c.quadro_ligado(meio, Some(&mut t), &mut nunca);
    let filme = |c: &Clipe, t: &PainterTool| {
        let mut e = c.palco.clone();
        ph2d_render::premul::unpremultiply_rgba8(&mut e);
        t.tinge_com_a_mascara(&mut e, (0, 0, LADO, LADO));
        ph2d_render::premul::premultiply_rgba8(&mut e);
        e
    };
    let e1 = filme(&c, &t);
    assert!(e1 != c.palco, "controlo: o filme muda o que se vê");
    assert!(r == e1, "o filme da máscara não está por cima do clipe");
    t.handle_panel_event(ph2d_editor_core::tool::PanelEvent::Click(
        ph2d_tool_painter::ids::PAINTER_MASK_COLOR[2],
    ));
    let r2 = c.quadro_ligado(meio, Some(&mut t), &mut nunca);
    let e2 = filme(&c, &t);
    assert!(e2 != e1, "controlo: a cor nova muda o filme");
    assert!(r2 == e2, "a cor nova do filme não subiu à ranhura");
}

#[path = "palcos_reescrita_tests.rs"]
mod reescrita_tests;
