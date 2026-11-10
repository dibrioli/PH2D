//! `PH2D_COMPOSITE_SMOKE` — a cena do **Composite Brush**, montada com a pilha do dono (2026-09-23).
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && \
//!   PH2D_COMPOSITE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
//! ```
//!
//! Ordem do dono: *«monte uma cena completa com o composite pronto para testarmos até o fim das
//! melhorias»*. ⚠️ **Ao contrário das irmãs, esta cena ARMA o pincel** — e é de propósito: o que
//! ela existe para julgar não é o valor de fábrica, é a pilha que o dono montou à mão na foto de
//! 2026-09-22 (o report *«ainda está lenta e engasgando com pincel grande»*), e remontá-la à mão a
//! cada smoke é o passo que ninguém repete igual duas vezes. O meio continua o de fábrica (o
//! Digital): a pilha vive nele.
//!
//! ## A tela
//!
//! Uma IMAGEM gerada, e não papel branco: numa tela chapada o borrão e o esfregão não têm o que
//! mostrar (medido: numa tela uniforme o borrão é um no-op). Ela tem um degradê de cor, uma grelha
//! escura a cada `64 px` (o esfregão arrasta-a, o borrão amolece-a) e discos de cor.
//! `PH2D_COMPOSITE_FUNDO=branco|papel` troca-a (a régua 4K mede por fundo); `PH2D_COMPOSITE_TELA=<w>x<h>` vale
//! para os três. `PH2D_COMPOSITE_SEGUNDA=1` monta uma 2.ª tela igual ao lado (a régua mede abrir um doc).
//!
//! ## A pilha (a da foto, posição `1` = o TOPO)
//!
//! | | operação | força | tamanho | cor |
//! |---|---|---|---|---|
//! | 1 | Blur | `1` | `2.048` | — |
//! | 2 | Brush | `0.133` | `0.574` | branco |
//! | 3 | Brush | `0.204` | `1.002` | vermelho |
//! | 4 | Brush | `0.176` | `1.221` | preto |
//! | 5 | Smear | `0.596` | `1` | — |
//! | 6 | Erase | `0.104` | `1` | Image |
//!
//! e o pincel em `Size 0.4` (`82,8 px` de raio).

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::SimWorld;
use ph2d_render::SpriteRenderer;
use std::collections::BTreeMap;

/// **O maior nível a que este roteador responde** — ele é de PRESENÇA (`var_os(..).is_some()`),
/// como o do `taper_smoke`, logo o único nível com significado é o `1`.
pub const NIVEIS: u32 = 1;

/// O lado da tela, em píxeis — o mesmo `1024²` da bancada `diag_preco_da_pilha`, que mediu esta
/// pilha, para o que o dono sente e o que a sonda mede serem o mesmo regime.
pub const LADO: u32 = 1024;

/// O tamanho da tela: `LADO²`, ou `PH2D_COMPOSITE_TELA=<w>x<h>` (sonda do pen-up a 4K, dono 10/11: o tamanho que
/// trava), com qualquer fundo.
fn tela() -> (u32, u32) {
    std::env::var("PH2D_COMPOSITE_TELA")
        .ok()
        .and_then(|v| {
            let (w, h) = v.split_once('x')?;
            Some((w.parse().ok()?, h.parse().ok()?))
        })
        .unwrap_or((LADO, LADO))
}

/// O centro (metros) da 2.ª tela de `PH2D_COMPOSITE_SEGUNDA`: à direita da 1.ª (centrada na origem), sem se tocarem.
fn centro_da_segunda(w: u32, pixels_per_meter: f32) -> Vec2 {
    Vec2::new(1.5 * w as f32 / pixels_per_meter.max(1e-3), 0.0)
}

/// Se a cena está armada. Barato o bastante para ser perguntado por quadro.
pub fn enabled() -> bool {
    std::env::var_os("PH2D_COMPOSITE_SMOKE").is_some()
}

/// O fundo da tela, `PH2D_COMPOSITE_FUNDO=blocos|branco|papel` (missão 4K, B08.2: o «antes» separa o papel do
/// tamanho). Omissão: os blocos; `PH2D_COMPOSITE_PAPEL=1` continua a ser o papel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fundo {
    /// A [`imagem`] gerada (degradê, grelha, discos).
    Blocos,
    /// Branco opaco, sem papel.
    Branco,
    /// O *New Image ▸ Textured Paper* (o bind reconhece-o e serve o papel).
    Papel,
}

fn fundo() -> Fundo {
    match std::env::var("PH2D_COMPOSITE_FUNDO").as_deref() {
        Ok("branco") => Fundo::Branco,
        Ok("papel") => Fundo::Papel,
        Ok("blocos") => Fundo::Blocos,
        _ if std::env::var_os("PH2D_COMPOSITE_PAPEL").is_some() => Fundo::Papel,
        _ => Fundo::Blocos,
    }
}

/// Os píxeis RGBA8 da tela `w×h` com o `fundo` pedido.
#[must_use]
pub fn pixeis_da_tela(fundo: Fundo, w: u32, h: u32) -> Vec<u8> {
    match fundo {
        Fundo::Blocos => imagem(w, h),
        Fundo::Branco => vec![255u8; (w as usize) * (h as usize) * 4],
        Fundo::Papel => crate::paint_mode::pixeis_da_imagem_nova(
            [w, h],
            ph2d_editor_core::ids::NEW_IMAGE_BG_PAPER,
        )
        .unwrap_or_else(|| imagem(w, h)),
    }
}

/// A tela gerada `w×h`: degradê de cor, grelha escura a cada `64 px`, e discos de cor (redondos: o raio conta-se
/// no lado menor; quadrada = a imagem de sempre, ao bit).
#[must_use]
pub fn imagem(w: u32, h: u32) -> Vec<u8> {
    let (lw, lh) = (w as f32, h as f32);
    let m = lw.min(lh);
    let (ex, ey) = (lw / m, lh / m);
    let mut px = vec![0u8; (w as usize) * (h as usize) * 4];
    let discos = [
        (0.25f32, 0.30f32, 0.09f32, [230u8, 60, 60]),
        (0.70, 0.28, 0.12, [60, 120, 230]),
        (0.45, 0.70, 0.10, [250, 200, 40]),
        (0.80, 0.75, 0.07, [40, 170, 90]),
    ];
    for y in 0..h {
        for x in 0..w {
            let (u, v) = (x as f32 / lw, y as f32 / lh);
            let mut c = [
                (70.0 + 150.0 * u) as u8,
                (120.0 + 80.0 * (1.0 - v)) as u8,
                (200.0 - 120.0 * u) as u8,
            ];
            for &(cx, cy, r, cor) in &discos {
                if ((u - cx) * ex).hypot((v - cy) * ey) < r {
                    c = cor;
                }
            }
            if x % 64 < 3 || y % 64 < 3 {
                c = [30, 30, 40];
            }
            let i = ((y as usize) * (w as usize) + x as usize) * 4;
            px[i..i + 3].copy_from_slice(&c);
            px[i + 3] = 255;
        }
    }
    px
}

/// A cena prepara a mão do artista: escolhe a tela que o [`spawn_if_enabled`] montou, enquadra-a
/// e pega no Painter.
///
/// ⚠️ Esta cena abre com o Painter NA MÃO (ordem do dono: «pronto para testarmos») — pelo MODO Paint
/// da imagem, a porta do artista (spec/06 F2; o botão IMG saiu em 05/10).
///
/// ⚠️ Mora aqui e não na shell por ordem da catraca `the_shell_only_shrinks` (integração de
/// 2026-09-25): a shell só chama; a cena é da família.
pub fn arm_hero(hero: &mut ph2d_editor_core::HeroScreen, bits: u64) {
    use ph2d_editor_core::action_bus::EditorAction;
    hero.gizmo.replace_selection(Some(bits));
    hero.bus.push(EditorAction::SetViewFocus {
        kind: ph2d_editor_core::ViewFocusKind::Selected,
    });
    hero.bus.push(EditorAction::ObjectMode(
        ph2d_editor_core::object_mode::ModeRequest::Enter(
            ph2d_editor_core::object_mode::ObjectMode::Draw,
        ),
    ));
    // A foto do seletor (24/10): `PH2D_COMPOSITE_PICKER=1` abre-o FIXADO, ao lado do Inspector. O
    // alvo é o próprio seletor — nenhum painel o fecha por ser «dele».
    if std::env::var_os("PH2D_COMPOSITE_PICKER").is_some() {
        let picker = ph2d_editor_core::ids::INSP_BLENDER_PICKER;
        hero.store.set_picker_target(Some(picker));
        hero.store.set_picker_pinned(true);
    }
}

/// A foto do painel em cartões (`docs/CartoesUI/13_PAINTER_PLANO.md`): `PH2D_COMPOSITE_PAINEL=digital|water|
/// impasto|wet|layers` mantém a aba à frente nos primeiros quadros, porque entrar no Draw põe lá o Inspector.
pub(crate) fn painel_a_frente(hero: &mut ph2d_editor_core::HeroScreen) {
    use std::sync::atomic::{AtomicU32, Ordering};
    static QUADROS: AtomicU32 = AtomicU32::new(0);
    if enabled()
        && std::env::var_os("PH2D_COMPOSITE_PAINEL").is_some()
        && QUADROS.fetch_add(1, Ordering::Relaxed) < 240
    {
        hero.store
            .bump_panel_z(ph2d_editor_core::ids::PAINTER_LAYERS_PANEL);
    }
}

/// Montar a tela quando `PH2D_COMPOSITE_SMOKE=1`, UMA vez, devolvendo os bits da entidade para
/// quem chama pôr a selecção nela. Imprime O QUE montou e o roteiro.
///
/// ⚠️ O "uma vez" é deste módulo (um `AtomicBool`) e não um campo da `App`: a shell é composição,
/// e um campo lá por cada cena é o que a catraca `the_shell_only_shrinks` existe para não deixar
/// crescer.
pub fn spawn_if_enabled(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Option<u64> {
    use std::sync::atomic::{AtomicBool, Ordering};
    static FEITO: AtomicBool = AtomicBool::new(false);
    if !enabled() || FEITO.swap(true, Ordering::Relaxed) {
        return None;
    }
    let (tw, th) = tela();
    match ph2d_image_import::spawn_rgba(
        sim,
        renderer,
        asset_db,
        cell_idx,
        tw,
        th,
        pixeis_da_tela(fundo(), tw, th),
        Vec2::new(0.0, 0.0),
        pixels_per_meter,
        atlas_asset_map,
        "Composite",
    ) {
        Ok((label, bits)) => {
            println!(
                "PH2D_COMPOSITE_SMOKE: tela '{label}' montada ({tw}x{th}, uma imagem com \
                 degrade, grelha e discos). O Painter abre sozinho e arma a pilha da foto \
                 com o pincel em Size 0.4."
            );
            // ⚠️ O roteiro vive DENTRO do `println!`: o censo de texto da casa isenta o que sai por
            // terminal, e uma `const` solta perde essa isenção.
            println!(
                "PH2D_COMPOSITE_SMOKE: O ROTEIRO\n\
  1) A cena abre com o pincel JA na mao. Em cima a direita, clique na aba 'Painter' (ao lado \
     de 'Inspector'): a caixa 'Composite Brush' vem ligada, com as SEIS camadas da sua foto \
     (1 Blur em cima ... 6 Erase em baixo), e o pincel em Size 0.4.\n\
  2) Risque DEPRESSA, em curvas e em zigue-zague, por cima da grelha e dos discos. O traco tem de \
     acompanhar o rato sem engasgar. Deu errado se: ele fica para tras do cursor, ou salta aos \
     bocados.\n\
  3) Olhe o traco de perto: a grelha deve aparecer arrastada (Smear) e amolecida (Blur), com a \
     tinta branca, vermelha e preta por cima. Deu errado se: aparecem RECTANGULOS ou degraus ao \
     longo do traco, ou a borda do traco fica serrilhada em blocos.\n\
  4) Faca o MESMO gesto devagar e depressa, lado a lado. Os dois tracos devem ter o mesmo aspecto. \
     Deu errado se: o traco rapido sai diferente do lento (mais claro, mais escuro, com falhas).\n\
  5) Ctrl+Z: o traco inteiro tem de desaparecer de uma vez. Deu errado se: fica resto de tinta ou \
     um rectangulo mais claro onde ele estava.\n\
  6) Desligue a caixa 'Composite Brush', risque, e volte a ligar: sem ela e o pincel simples; com \
     ela volta a pilha, igual.\n\
  7) Com as setas da camada 5 (Smear), suba-o ate ficar ACIMA do Blur e risque. Este arranjo usa o \
     caminho antigo de proposito (pode ser mais lento); a imagem tem de continuar limpa.\n\
  8) Mude o Size do pincel para 0.2 e para 0.6 e risque: mais pequeno tem de ser mais leve, e \
     nenhum dos dois pode deixar rectangulos."
            );
            if std::env::var_os("PH2D_COMPOSITE_SEGUNDA").is_some() {
                // B08.1 «abrir doc 4K»: clicar nesta na Hierarquia liga um doc WxH novo no Painter. A célula é a
                // seguinte à da 1.ª (o contador da shell só anda 1: medição, nunca produto).
                let segunda = ph2d_image_import::spawn_rgba(
                    sim,
                    renderer,
                    asset_db,
                    cell_idx.saturating_add(1),
                    tw,
                    th,
                    pixeis_da_tela(fundo(), tw, th),
                    centro_da_segunda(tw, pixels_per_meter),
                    pixels_per_meter,
                    atlas_asset_map,
                    "Composite 2",
                );
                match segunda {
                    Ok((l, _)) => {
                        println!("PH2D_COMPOSITE_SMOKE: segunda tela '{l}' ({tw}x{th}) ao lado")
                    }
                    Err(e) => {
                        eprintln!("PH2D_COMPOSITE_SMOKE: nao consegui montar a segunda tela: {e}")
                    }
                }
            }
            Some(bits)
        }
        Err(e) => {
            eprintln!("PH2D_COMPOSITE_SMOKE: nao consegui montar a tela: {e}");
            None
        }
    }
}

/// Armar a pilha da foto e o pincel na primeira vez que o Painter liga um documento sob a cena.
/// UMA vez: depois disso tudo o que o artista mexer é dele.
static ARMADO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// `PH2D_COMPOSITE_PILHA=0` deixa o pincel sozinho, sem a pilha (o rabisco mede o pincel puro).
pub fn arm_brush_once(painter: &mut ph2d_tool_painter::PainterTool) {
    if !enabled() || ARMADO.swap(true, std::sync::atomic::Ordering::Relaxed) {
        return;
    }
    if std::env::var_os("PH2D_COMPOSITE_PILHA").is_none_or(|v| v != "0") {
        arma_a_pilha_do_dono(painter);
    }
    use ph2d_tool_painter::PaintMedia as Meio;
    match std::env::var("PH2D_COMPOSITE_PAINEL").as_deref() {
        Ok("water") => painter.set_paint_media(Meio::Watercolor),
        Ok("impasto") => painter.set_paint_media(Meio::Impasto),
        Ok("wet") => painter.set_paint_media(Meio::WetPaint),
        Ok("layers") => painter.set_dock_shows_layers(true),
        _ => {}
    }
    if let Some(v) = std::env::var("PH2D_COMPOSITE_RELEVO")
        .ok()
        .and_then(|v| v.parse().ok())
    {
        painter.set_substrate_depth(v); // o papel com Relief (arma o papel de reserva)
    }
    // A foto da prévia de um pincel de fábrica (08/11, os esbatedores): `PH2D_COMPOSITE_PRESET=<nome>`.
    #[cfg(feature = "panel-painter-layers")]
    if let Ok(nome) = std::env::var("PH2D_COMPOSITE_PRESET") {
        let raiz =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
        match ph2d_brush_presets::Biblioteca::carregar(&raiz, None)
            .entradas
            .into_iter()
            .find(|e| e.preset.parametros.name == nome)
        {
            Some(e) => {
                if let Err(err) = painter.preset_aplicar(&e.preset) {
                    eprintln!("PH2D_COMPOSITE_PRESET: {nome}: {err:?}");
                }
            }
            None => eprintln!("PH2D_COMPOSITE_PRESET: {nome} não é de fábrica"),
        }
    }
    println!(
        "PH2D_COMPOSITE_SMOKE: pilha armada — {} camadas, Composite Brush ligado, Size 0.4.",
        painter.composite_len()
    );
}

/// **O RABISCO AUTOMÁTICO** — `PH2D_COMPOSITE_RABISCO=<Size>` (com a cena ligada): a cada quadro
/// a ponte entrega à ferramenta os eventos de um risco rápido, pela MESMA porta do rato
/// (`on_canvas_pointer`), e o `PH2D_PAINT_PERF=1` mede o quadro INTEIRO da app — a pilha, a
/// subida para a placa e o desenho — sem ninguém a riscar.
///
/// ⭐ Nasceu do report do dono de 2026-10-01 (*«pincel com size 0.5 fps cai para 40»* e, depois da
/// 1.ª cura, *«não percebi melhorias»*): a sonda da crate da ferramenta mede só a PILHA, e o quadro
/// da app tem mais coisas. ⚠️ Os eventos são escritos ANTES da drenagem, logo o custo deles cai na
/// fase `preview` do relatório e não na `INPUT` (que é a do handler do rato).
///
/// O risco: `16` eventos por quadro (um rato de `1 kHz` a `60 Hz`) ao longo da MESMA curva da sonda
/// `diag_onde_vai_o_quadro_do_rabisco`, traços de `3 s` com `0,5 s` de pausa, para a medição ver
/// o traço e o descanso.
pub fn rabisca(painter: &mut ph2d_tool_painter::PainterTool) {
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
    use std::sync::atomic::{AtomicU32, Ordering};
    static QUADRO: AtomicU32 = AtomicU32::new(0);
    static SIZE: std::sync::OnceLock<Option<f32>> = std::sync::OnceLock::new();
    let Some(size) = *SIZE.get_or_init(|| {
        if !enabled() {
            return None;
        }
        std::env::var("PH2D_COMPOSITE_RABISCO")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
    }) else {
        return;
    };
    // Só depois de a cena armar a pilha (o documento ligado).
    if !ARMADO.load(Ordering::Relaxed) {
        return;
    }
    const POR_QUADRO: u32 = 16;
    const TRACO: u32 = 180;
    const PAUSA: u32 = 30;
    let q = QUADRO.fetch_add(1, Ordering::Relaxed);
    if q == 0 {
        painter.set_brush_size_norm(size);
        println!("PH2D_COMPOSITE_RABISCO: a riscar sozinho com o Size {size}.");
    }
    let fase = q % (TRACO + PAUSA);
    let ev = |pos: [f32; 2], phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let ponto = |i: u32| {
        #[allow(clippy::cast_precision_loss)]
        let s = i as f32 * 0.03;
        [
            512.0 + 300.0 * (1.3 * s).sin(),
            512.0 + 260.0 * (0.9 * s).cos(),
        ]
    };
    let base = q * POR_QUADRO;
    if fase == 0 {
        painter.on_canvas_pointer(ev(ponto(base), PointerPhase::Down));
    }
    if fase < TRACO {
        for k in 1..=POR_QUADRO {
            painter.on_canvas_pointer(ev(ponto(base + k), PointerPhase::Move));
        }
    }
    if fase + 1 == TRACO {
        painter.on_canvas_pointer(ev(ponto(base + POR_QUADRO), PointerPhase::Up));
    }
    // O PERÍODO do quadro durante o traço — o número que o dono SENTE (o `frame p50` do relatório
    // mede o trabalho dentro do quadro, não o intervalo entre dois).
    static INICIO: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
    if let Ok(mut ini) = INICIO.lock() {
        if fase == 0 {
            *ini = Some(std::time::Instant::now());
        } else if fase + 1 == TRACO
            && let Some(t) = ini.take()
        {
            #[allow(clippy::cast_precision_loss)]
            let ms = t.elapsed().as_secs_f64() * 1e3 / f64::from(TRACO - 1);
            println!(
                "PH2D_COMPOSITE_RABISCO: periodo {ms:.1} ms/quadro ({:.0} fps) durante o traco",
                1e3 / ms
            );
        }
    }
}

/// **A pilha da foto, montada pelas portas públicas do Painter** — sem ler o ambiente, para o gate
/// a poder medir (um gate que lê o ambiente mede a máquina).
///
/// ⚠️ Pelas MESMAS portas que o painel usa (`acrescenta_camada` · `set_composite_layer_*` ·
/// `set_brush_size_norm`), e não escrevendo os campos: é isso que faz a quota e o piso do tamanho
/// valerem aqui como valem para o artista.
pub fn arma_a_pilha_do_dono(painter: &mut ph2d_tool_painter::PainterTool) {
    // A pilha, posição 0 = o TOPO. As operações vão pelos números que o painel usa (`Brush 0 ·
    // Smear 1 · Blur 2 · Erase 3`), e cada camada nova entra no FUNDO — logo cria-se de cima para
    // baixo.
    const PILHA: [(u8, f32, f32, Option<[f32; 3]>); 6] = [
        (2, 1.0, 2.048, None),
        (0, 0.133, 0.574, Some([1.0, 1.0, 1.0])),
        (0, 0.204, 1.002, Some([1.0, 0.0, 0.0])),
        (0, 0.176, 1.221, Some([0.0, 0.0, 0.0])),
        (1, 0.596, 1.0, None),
        (3, 0.104, 1.0, None),
    ];
    while painter.composite_len() > 0 {
        painter.retira_camada(0);
    }
    for (pos, &(op, forca, tamanho, cor)) in PILHA.iter().enumerate() {
        painter.acrescenta_camada(op);
        painter.set_composite_layer_strength(pos, forca);
        painter.set_composite_layer_size(pos, tamanho);
        match cor {
            Some(rgb) => painter.set_composite_layer_color(pos, rgb),
            None => painter.clear_composite_layer_color(pos),
        }
    }
    if !painter.composite_enabled() {
        painter.toggle_composite();
    }
    painter.set_brush_size_norm(0.4);
}

#[cfg(test)]
mod tests {
    /// **A cena monta a pilha da FOTO do dono** — e não outra. A régua é o snapshot que o PAINEL
    /// pinta (`brush_settings`), logo o que este gate afirma é o que o dono vê no cartão.
    ///
    /// ⚠️ Cada coluna é uma decisão da foto de 2026-09-22 (operação · força · tamanho · cor), e a
    /// ordem é do TOPO para baixo: a linha `1` do painel é a posição `0`, que a pilha aplica por
    /// último. ⛔ Uma pilha invertida mediria outro produto — foi a pergunta mais importante da
    /// auditoria de 2026-09-23 sobre a sonda de preço, e esta cena não pode reabri-la.
    #[test]
    fn a_cena_monta_a_pilha_da_foto() {
        let mut p = ph2d_tool_painter::PainterTool::default();
        super::arma_a_pilha_do_dono(&mut p);
        let s = p.brush_settings();
        assert!(s.composite_enabled, "o Composite Brush tem de vir ligado");
        assert_eq!(s.composite_len, 6, "as seis camadas da foto");
        assert_eq!(
            &s.composite_ops[..6],
            &[2, 0, 0, 0, 1, 3],
            "Blur em cima, três Brush, Smear, Erase em baixo"
        );
        let forcas = [1.0, 0.133, 0.204, 0.176, 0.596, 0.104];
        let tamanhos = [2.048, 0.574, 1.002, 1.221, 1.0, 1.0];
        for i in 0..6 {
            assert!(
                (s.composite_strength[i] - forcas[i]).abs() < 1e-6,
                "camada {}: força {} contra {}",
                i + 1,
                s.composite_strength[i],
                forcas[i]
            );
            assert!(
                (s.composite_size[i] - tamanhos[i]).abs() < 1e-6,
                "camada {}: tamanho {} contra {}",
                i + 1,
                s.composite_size[i],
                tamanhos[i]
            );
        }
        assert_eq!(
            &s.composite_color_authored[..6],
            &[false, true, true, true, false, false],
            "só os três Brush têm cor própria"
        );
        assert_eq!(s.composite_color[1], [1.0, 1.0, 1.0], "a camada 2 é branca");
        assert_eq!(
            s.composite_color[2],
            [1.0, 0.0, 0.0],
            "a camada 3 é vermelha"
        );
        assert_eq!(s.composite_color[3], [0.0, 0.0, 0.0], "a camada 4 é preta");
        assert_eq!(
            s.composite_erase_scope[5], 0,
            "a borracha apaga a imagem (Image)"
        );
        // O pincel em Size 0.4 — lido pela mesma lei do slider, não por uma conta à mão.
        assert!(
            (s.size_norm - 0.4).abs() < 1e-3,
            "o pincel tem de vir em Size 0.4 (lido {})",
            s.size_norm
        );
    }

    /// Armar DUAS vezes dá a mesma pilha — a montagem começa por esvaziar o que lá estiver, senão uma
    /// sessão que já tinha camadas ficava com a quota esgotada e a pilha pela metade.
    #[test]
    fn armar_sobre_uma_pilha_existente_da_a_mesma_pilha() {
        let mut p = ph2d_tool_painter::PainterTool::default();
        p.acrescenta_camada(0);
        p.acrescenta_camada(0);
        super::arma_a_pilha_do_dono(&mut p);
        let s = p.brush_settings();
        assert_eq!(s.composite_len, 6);
        assert_eq!(&s.composite_ops[..6], &[2, 0, 0, 0, 1, 3]);
    }

    /// A tela tem textura — num campo uniforme o borrão e o esfregão não têm o que mostrar.
    #[test]
    fn a_tela_nao_e_chapada() {
        let px = super::imagem(128, 128);
        let cores: std::collections::BTreeSet<[u8; 3]> = px
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        assert!(
            cores.len() > 50,
            "a tela tem de ter variação: {} cores",
            cores.len()
        );
        assert!(
            px.as_chunks::<4>().0.iter().all(|c| c[3] == 255),
            "a tela é opaca"
        );
    }

    /// Oráculo verbatim da `imagem(lado)` quadrada de antes do B08.2.
    fn imagem_quadrada_de_antes(lado: u32) -> Vec<u8> {
        let l = lado as f32;
        let mut px = vec![0u8; (lado * lado * 4) as usize];
        let discos = [
            (0.25f32, 0.30f32, 0.09f32, [230u8, 60, 60]),
            (0.70, 0.28, 0.12, [60, 120, 230]),
            (0.45, 0.70, 0.10, [250, 200, 40]),
            (0.80, 0.75, 0.07, [40, 170, 90]),
        ];
        for y in 0..lado {
            for x in 0..lado {
                let (u, v) = (x as f32 / l, y as f32 / l);
                let mut c = [
                    (70.0 + 150.0 * u) as u8,
                    (120.0 + 80.0 * (1.0 - v)) as u8,
                    (200.0 - 120.0 * u) as u8,
                ];
                for &(cx, cy, r, cor) in &discos {
                    if (u - cx).hypot(v - cy) < r {
                        c = cor;
                    }
                }
                if x % 64 < 3 || y % 64 < 3 {
                    c = [30, 30, 40];
                }
                let i = ((y * lado + x) * 4) as usize;
                px[i..i + 3].copy_from_slice(&c);
                px[i + 3] = 255;
            }
        }
        px
    }

    /// A tela quadrada de blocos é a de sempre, ao bit (o «antes» a 1024² continua comparável).
    #[test]
    fn a_tela_de_blocos_quadrada_e_a_de_antes_ao_bit() {
        for lado in [128, 1024] {
            assert!(
                super::pixeis_da_tela(super::Fundo::Blocos, lado, lado)
                    == imagem_quadrada_de_antes(lado),
                "lado {lado}"
            );
        }
    }

    /// Blocos e branco obedecem ao `PH2D_COMPOSITE_TELA` (B08.2: antes os blocos eram sempre 1024² e não havia
    /// branco); os discos ficam redondos na tela retangular.
    #[test]
    fn os_fundos_obedecem_ao_tamanho_pedido() {
        let (w, h) = (640u32, 360u32);
        let blocos = super::pixeis_da_tela(super::Fundo::Blocos, w, h);
        assert_eq!(blocos.len(), (w * h * 4) as usize, "blocos {w}x{h}");
        let branco = super::pixeis_da_tela(super::Fundo::Branco, w, h);
        assert_eq!(branco.len(), (w * h * 4) as usize, "branco {w}x{h}");
        assert!(branco.iter().all(|&b| b == 255), "branco opaco");
        // O disco vermelho (centro 0,25·w × 0,30·h, raio 0,09 do lado menor): dentro a 0,08·h na horizontal e
        // na vertical, fora a 0,10·h.
        let px = |x: f32, y: f32| {
            let i = ((y as usize) * (w as usize) + x as usize) * 4;
            [blocos[i], blocos[i + 1], blocos[i + 2]]
        };
        let (cx, cy, m) = (0.25 * w as f32, 0.30 * h as f32, h as f32);
        for (dx, dy) in [(0.08, 0.0), (0.0, 0.08), (-0.08, 0.0), (0.0, -0.08)] {
            assert_eq!(
                px(cx + dx * m, cy + dy * m),
                [230, 60, 60],
                "dentro ({dx},{dy})"
            );
        }
        assert_ne!(px(cx + 0.10 * m, cy), [230, 60, 60], "fora na horizontal");
    }

    /// A 2.ª tela (B08.1, abrir doc) nasce fora da 1.ª: o traço da matriz só cai na 1.ª.
    #[test]
    fn a_segunda_tela_nasce_fora_da_primeira() {
        for (w, ppm) in [(3840u32, 100.0f32), (1920, 100.0), (1024, 32.0)] {
            let meia = 0.5 * w as f32 / ppm;
            let c = super::centro_da_segunda(w, ppm);
            assert!(c.x - meia >= meia, "w {w} ppm {ppm}: centro {}", c.x);
        }
    }
}
