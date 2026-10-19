//! W9, `docs/Macros/07_plano_W9.md` — **a mistura pela MARCA**, com o Painter ligado e a água no worker
//! real: o Rewind e o clipe inteiro com a água a correr, a pincelada por cima e o desfazer, apagar até ao
//! papel, a sprite sem Painter, outra gravação, o tamanho que muda, a marca gravada e o custo (A3).
//! Filho de `palcos_fim_tests` (o `Clipe` e as réguas à parte são dele).

use super::*;

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
    let lei = |c: &Clipe, r: &[u8]| perto(r, &esperado_pela_marca(&c.palco, &premul(&depois), &m));
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
/// passa sobre o que a macro pintou; em todo quadro, o miolo dela (apagado de todo) mostra o editor e a
/// borda segue a lei da cobertura (W10: o papel por cima do clipe na força da borracha). Controlo: no fim
/// do clipe o palco tem tinta nesses texels (o clipe reapareceria).
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
    let m = t.marca_do_artista().expect("a marca").clone();
    let cheios = apagados
        .iter()
        .filter(|&&k| m.cobertura(k) == ph2d_tool_painter::macro_track::marca::CHEIA)
        .count();
    assert!(
        cheios > 40,
        "controlo: o miolo da borracha apagou de todo ({cheios})"
    );
    let mut q = 0u32;
    loop {
        let instante = f64::from(q) / 60.0;
        let r = c.quadro_ligado(instante, Some(&mut t), &mut nunca);
        let dp = premul(&depois);
        let m = t.marca_do_artista().expect("a marca").clone();
        let lei = esperado_pela_marca(&c.palco, &dp, &m);
        for &k in &apagados {
            assert!(
                perto(&r[k * 4..k * 4 + 4], &lei[k * 4..k * 4 + 4]),
                "quadro {q}: o texel {k} apagado não segue a lei"
            );
            if m.cobertura(k) == ph2d_tool_painter::macro_track::marca::CHEIA {
                assert!(
                    r[k * 4..k * 4 + 4] == dp[k * 4..k * 4 + 4],
                    "quadro {q}: o texel {k} apagado até ao papel mostra o clipe"
                );
            }
        }
        if instante > d + 0.05 {
            let pintados = apagados
                .iter()
                .filter(|&&k| c.palco[k * 4..k * 4 + 4] != dp[k * 4..k * 4 + 4])
                .count();
            assert!(pintados > 0, "controlo: o palco tinha tinta onde se apagou");
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
        Some((
            if qual.get() == 1 { &atual } else { &fim }.clone(),
            LADO,
            LADO,
        ))
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
    assert_eq!(
        lidas.get(),
        antes + 1,
        "os píxeis mudaram e a sprite não se releu"
    );
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
    assert!(
        r == c.palco,
        "o desfazer da ferramenta não devolveu o clipe"
    );
}

/// **Uma ferramenta que muda o tamanho não apaga a marca**: a sprite pintada (marcada), depois cortada
/// (outro tamanho: a ranhura é o palco — a gravação é de outro tamanho), depois o desfazer devolve o
/// tamanho e a pintura volta por cima do clipe. ⛔ A 2.ª lente da auditoria de 17/10: o tamanho novo
/// recriava a marca vazia e o desfazer não a devolvia.
#[test]
fn mudar_o_tamanho_e_desfazer_devolve_a_marca() {
    let (doc, _, fim) = gravada_no_papel();
    let atual = pintada_depois(&fim);
    let cortada = vec![255u8; ((LADO / 2) * (LADO / 2) * 4) as usize];
    let qual = std::cell::Cell::new(0u8);
    let mut le = |_: u64, _: &mut Placa| match qual.get() {
        0 => Some((fim.clone(), LADO, LADO)),
        1 => Some((atual.clone(), LADO, LADO)),
        _ => Some((cortada.clone(), LADO / 2, LADO / 2)),
    };
    let mut c = Clipe::novo(&doc);
    let id = |b: &[u8]| ph2d_ecs::SpritePixels(ph2d_asset::AssetId::from_bytes(b));
    let passo = |c: &mut Clipe, q: u8, img: &[u8], t: f64, le: &mut LeSprite<'_>| {
        qual.set(q);
        c.world.entity_mut(c.e).insert(id(img));
        c.quadro_ligado(t, None, le)
    };
    passo(&mut c, 0, &fim, 0.02, &mut le);
    let mudou = diferem(&fim, &atual);
    let mistura = |c: &Clipe| esperado(&c.palco, &premul(&atual), |i| mudou.contains(&i));
    let r = passo(&mut c, 1, &atual, 0.03, &mut le);
    assert!(r == mistura(&c), "controlo: a pintura aparece por cima");
    let r = passo(&mut c, 2, &cortada, 0.04, &mut le);
    assert!(r == c.palco, "noutro tamanho a ranhura não é o palco");
    let r = passo(&mut c, 1, &atual, 0.05, &mut le);
    assert!(r == mistura(&c), "o desfazer do corte não devolveu a marca");
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
        perto(&r, &esperado_pela_marca(&c.palco, &premul(&depois), &m)),
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
    assert!(
        aberto.palcos.restaura_marcas(&bytes),
        "as marcas não se leram"
    );
    let mut le = |_: u64, _: &mut Placa| Some((depois.clone(), LADO, LADO));
    let r = aberto.quadro_ligado(0.1, None, &mut le);
    assert!(
        perto(
            &r,
            &esperado_pela_marca(&aberto.palco, &premul(&depois), &m)
        ),
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
        (
            c.palco_mut().versao_da_atual,
            c.placa.substituidas,
            c.placa.regioes
        ),
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

/// A 2.ª tomada sobre uma sprite que já tem macro, na ordem da shell: o clipe toca, o Painter grava uma
/// pincelada (a sprite cede ao traço vivo; a pincelada marca-se na época ANTIGA), e no quadro do Parar a
/// subida corre ainda a ceder — a gravação nova entra depois dela (`fase_macros`, no fim do quadro).
/// Devolve o clipe com a gravação nova já instalada, o Painter e o que a ranhura tinha no Parar.
fn segunda_tomada_ate_ao_parar() -> (Clipe, PainterTool, Vec<u8>) {
    let mut world = World::new();
    let e = world.spawn(MacroTempo { segundos: 0.0 }).id();
    let (mut t, doc, _) =
        ph2d_tool_painter::macro_track::roteiro::grava_no_papel_ligado(LADO, e.to_bits(), 2)
            .expect("grava");
    while !t.agua_parada() {
        tique(&mut t);
    }
    let mut c = Clipe::com(world, e, &doc);
    // A cena `=6` abre com uma linha (a espia liga-se antes da 1.ª subida).
    let _ = c.palcos.diagnostico(e.to_bits(), Some(&t), &c.world);
    c.quadro_ligado(0.0, Some(&mut t), &mut nunca);
    t.set_paint_media(ph2d_tool_painter::PaintMedia::Digital);
    traco(&mut t, [8.0, 54.0], [50.0, 54.0], 4, true);
    let cede = Some(e.to_bits());
    let (trocou, no_parar) = c.quadro_cedido(0.0, Some(&mut t), &mut nunca, cede);
    assert!(!trocou, "a gravar, a sprite mostra o traço vivo do editor");
    let (nova, _) =
        ph2d_tool_painter::macro_track::roteiro::grava_no_papel(LADO, 0, 3).expect("grava");
    c.world.entity_mut(e).insert(Macro {
        doc: nova.clone(),
        modo: ModoDaMacro::Fiel,
    });
    c.verdade = ph2d_tool_painter::macro_track::em_cena::MacroEmCena::de(&nova)
        .expect("lê-se")
        .expect("trilha");
    (c, t, no_parar)
}

/// ⭐ **O Parar não pisca a tomada antiga** (o «piscar» aberto na W9; medido na `=6 parar` com a espia:
/// o ecrã nunca o mostrou): depois do Parar, cada quadro troca a sprite pela gravação NOVA ao byte —
/// nenhum mostra a mistura da antiga com a pincelada da tomada nova. Controlo: no quadro do Parar a
/// ranhura TEM essa mistura (a pincelada por cima do clipe antigo); só a cedência a esconde.
#[test]
fn o_parar_nunca_troca_a_sprite_pela_mistura_da_gravacao_antiga() {
    let (mut c, mut t, no_parar) = segunda_tomada_ate_ao_parar();
    for q in 0..6u32 {
        let instante = f64::from(q) / 60.0;
        let (trocou, r) = c.quadro_cedido(instante, Some(&mut t), &mut nunca, None);
        assert!(trocou, "quadro {q} depois do Parar: a sprite não trocou");
        assert!(
            r == c.palco,
            "quadro {q} depois do Parar: {} texels não são a gravação nova",
            diferem(&r, &c.palco).len()
        );
        assert!(
            q > 0 || !diferem(&no_parar, &r).is_empty(),
            "controlo: no Parar a ranhura tinha a mistura antiga"
        );
    }
}

/// **A linha da `=6` diz o que o ecrã MOSTROU** — ela corre no início do quadro, antes do `segue`: o
/// Parar já tirou a cedência e a gravação nova já está no mundo, mas o palco ainda é o do quadro
/// anterior. ⛔ VERMELHO antes da cura: a linha recalculava a troca com a cedência DESTE quadro contra
/// o palco do ANTERIOR e dizia `troca=true` com a marca antiga num quadro que não trocou — o «piscar»
/// registado na W9, que o ecrã nunca teve.
#[test]
fn a_linha_do_caminho_diz_o_que_a_subida_anterior_mostrou() {
    let (mut c, mut t, _) = segunda_tomada_ate_ao_parar();
    let bits = c.e.to_bits();
    let linha = c.palcos.diagnostico(bits, Some(&t), &c.world);
    assert!(
        !linha.contains("troca=true") && linha.contains("mostrou: troca=false"),
        "o quadro do Parar não trocou a sprite: {linha}"
    );
    c.quadro_cedido(0.0, Some(&mut t), &mut nunca, None);
    let linha = c.palcos.diagnostico(bits, Some(&t), &c.world);
    assert!(
        linha.contains("mostrou: troca=true"),
        "o quadro seguinte trocou: {linha}"
    );
}
