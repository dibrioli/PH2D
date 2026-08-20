//! **A DECORAÇÃO DA FOLHA** — a moldura hachurada e o nome, desenhados sobre o canvas.
//!
//! Pedido do Enio (2026-08-19), com exemplo: *"Falta a decoração da folha… e coloque o nome da
//! folha no topo à esquerda do objeto."*
//!
//! ## Por que é CHROME e não estilo do documento
//!
//! A folha é um retângulo vivo, então dar-lhe um preenchimento e um traço no documento seria o
//! caminho mais curto — e o errado. Três razões, em ordem de força:
//!
//! 1. **Ela diz o que a folha É, não como ela se parece.** Um preenchimento autorado é do artista:
//!    ele pode trocá-lo, e nesse dia a folha deixaria de se ler como folha. Chrome não é editável
//!    porque não é conteúdo.
//! 2. **Ela não pode entrar no bake.** O que se assa são os FILHOS; uma decoração no documento
//!    seria um convite permanente a alguém a incluir por engano.
//! 3. **Ela tem de ser legível em qualquer zoom.** As hachuras são desenhadas em **pixels de
//!    TELA** — afastar o canvas não as engrossa nem as some, como aconteceria a um traço em metros.
//!
//! ## A hachura é recortada, não calculada
//!
//! Desenhar só as porções de reta que caem na faixa exigiria intersectar cada diagonal com um anel
//! retangular — quatro casos por linha, e a aritmética de bordas é onde este tipo de desenho
//! costuma partir-se. Em vez disso a cena recebe um **clip** com a forma da faixa (regra
//! *even-odd*: retângulo externo menos interno) e as diagonais são traçadas de lado a lado. O
//! recorte é o mesmo que os painéis roláveis do editor usam.

use ph2d_ecs::{Name, SimWorld, SpriteSheetFrame, VecShape};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme, TypeToken};
use ph2d_vector::{Affine, BezPath, Brush, Color, Fill, Point, Stroke, VectorScene};

/// Largura da faixa hachurada, em **pixels de tela**.
///
/// ⚠️ Em pixels de tela, e não em metros, porque ela é uma legenda e não uma medida: a faixa diz
/// *"isto é uma folha"*, e essa frase tem de ser igualmente legível com o canvas afastado ou perto.
const BAND_PX: f64 = 14.0;

/// Distância entre hachuras, em pixels de tela.
const HATCH_STEP_PX: f64 = 8.0;

/// Espessura de cada hachura, em pixels de tela.
///
/// ⚠️ **Grossa de propósito** (Enio 2026-08-19: *"com listras como a minha imagem"*). A 1ª
/// tentativa usou `1.5` sobre fundo nenhum e o resultado leu-se como sujidade, não como padrão:
/// uma listra tem de ocupar uma fração VISÍVEL do vão para o olho a ler como listra. Aqui é
/// ~⅜ do passo, a proporção da fita de obra que este desenho cita.
const HATCH_PX: f64 = 3.0;

/// Espessura das duas linhas que fecham a faixa (a de fora e a de dentro).
const EDGE_PX: f64 = 1.0;

/// Folga entre o topo da folha e a linha de base do nome, em pixels de tela.
const LABEL_GAP_PX: f64 = 4.0;

/// Largura máxima do rótulo, em pixels de tela — o `paint_text` corta o que exceder.
///
/// ⚠️ Existe porque o nome é do artista: um nome longo sem teto atravessaria a tela e cobriria a
/// arte que ele está a tentar ver.
const LABEL_MAX_W_PX: f32 = 340.0;

/// Desenha a decoração de cada folha da cena.
///
/// `px_per_world` é a escala do afim da câmara — é ela que converte as constantes de TELA acima
/// para o espaço em que a cena é montada.
pub(crate) fn draw(
    sim: &mut SimWorld,
    cam: Affine,
    px_per_world: f64,
    theme: Theme,
    text_system: &mut TextSystem,
    scene: &mut VectorScene,
) {
    if px_per_world <= 0.0 {
        return;
    }
    for (entity, cfg) in sheets(sim) {
        let Some((w, h)) = sheet_size_world(sim, entity) else {
            continue;
        };
        // A pose de MUNDO — a folha pode ter sido movida, escalada e rodada, e a decoração tem de
        // ir com ela. O mesmo `world_transform` que o gizmo usa: uma porta só para "onde isto
        // está", senão a moldura e as alças discordam no primeiro arrasto.
        let Some(wt) = ph2d_ecs::world_transform(sim.world(), entity) else {
            continue;
        };
        let local_to_world = Affine::translate((wt.translation.x as f64, wt.translation.y as f64))
            * Affine::rotate(wt.rotation as f64)
            * Affine::scale_non_uniform(wt.scale.x as f64, wt.scale.y as f64);
        let xf = cam * local_to_world;
        // As constantes são de TELA; aqui elas voltam ao espaço local, onde a geometria vive.
        // ⚠️ A escala do objeto entra na conta: uma folha escalada a 2× encolheria a faixa pela
        // metade se só se dividisse pelo zoom da câmara.
        let sx = (wt.scale.x.abs() as f64).max(f64::EPSILON);
        let sy = (wt.scale.y.abs() as f64).max(f64::EPSILON);
        let band_x = BAND_PX / (px_per_world * sx);
        let band_y = BAND_PX / (px_per_world * sy);
        let (hw, hh) = (w * 0.5, h * 0.5);
        // ⚠️ **A faixa cresce para FORA** (Enio 2026-08-19: *"a decoração deveria ficar fora do
        // gizmo"*). Ela ocupava a borda interna, e isso custava duas coisas ao mesmo tempo: as
        // alças do gizmo — que pousam nos cantos e nos meios das arestas, ou seja, exatamente
        // sobre a borda — ficavam POR CIMA dela, e a primeira peça (o empacotador põe a maior no
        // canto superior-esquerdo) tapava-a. Fora, a faixa emoldura sem disputar pixel nenhum:
        // o interior inteiro é conteúdo, e o gizmo continua a ser a coisa mais externa que se
        // agarra.
        let outer = rect_path(-hw - band_x, -hh - band_y, hw + band_x, hh + band_y);
        let inner = rect_path(-hw, -hh, hw, hh);
        // **A MOLDURA ACUSA** (Enio 2026-08-19: *"Se houver sobreposição ou se as sprites não
        // couberem na sheet, sua moldura deve ficar vermelha e um aviso deve aparecer ao lado da
        // resolução"*).
        //
        // ⚠️ **A cor E o texto, nunca só a cor.** Vermelho sozinho diz *"algo está mal"* e deixa o
        // artista a procurar o quê — e para quem não distingue vermelho de cinzento não diz nada
        // (a lei de a cor nunca ser o único canal). O rótulo nomeia QUAL das duas condições é, e
        // as duas podem ocorrer juntas.
        //
        // ⚠️ A saúde é CONTADA aqui, a cada quadro, e não guardada em lado nenhum — vide
        // `sheet_bounds::health`. Um sinalizador guardado ficaria vermelho sobre uma folha já
        // arrumada no dia em que alguém movesse uma peça sem o atualizar.
        let health = crate::sheet_bounds::health(sim, entity);
        let edge = if health.is_ok() {
            resolve(ColorToken::BorderStrong, theme)
        } else {
            resolve(ColorToken::Danger, theme)
        };
        let hatch_color = if health.is_ok() {
            resolve(ColorToken::Text2, theme)
        } else {
            resolve(ColorToken::Danger, theme)
        };
        // A faixa = externo ⊖ interno, pela regra even-odd. É este caminho que recorta as
        // hachuras — elas são traçadas de lado a lado e o clip fica com o que interessa.
        //
        // ⚠️ **O CLIP É EM ESPAÇO DE CENA, e o `push_clip` NÃO aceita transformação** — ele usa
        // `Affine::IDENTITY` por dentro. Empurrar o caminho em coordenadas LOCAIS (aqui, ±2,5
        // unidades) recorta uma caixinha na origem da tela e apaga tudo o que se desenhar a
        // seguir. E o modo de falha é MUDO: os contornos continuam a aparecer, porque são
        // traçados depois do `pop_layer` — o que se vê é uma moldura vazia, que se lê como
        // *"esqueceram de desenhar o padrão"* e não como *"o recorte comeu o padrão"*.
        // Foi exatamente isto que fez as listras não existirem nas duas primeiras tentativas.
        let mut band = outer.clone();
        band.extend(inner.iter());
        scene.push_clip_with_rule(&(xf * band), Fill::EvenOdd);
        // ⚠️ **Um FUNDO por baixo das listras**, e é ele que as torna listras. Sem fundo, as
        // diagonais ficavam a flutuar sobre o canvas e liam-se como ruído — foi o que a 1ª
        // tentativa entregou. O par fundo-claro + traço-escuro é o que faz a faixa ser uma
        // *superfície* com um padrão, e não um punhado de riscos.
        // ⚠️ Preenche o retângulo EXTERNO, não o `band` de dois sub-caminhos: o `fill_path` usa
        // a regra *non-zero*, e os dois retângulos giram no mesmo sentido — o buraco não seria
        // buraco, e o fundo taparia o conteúdo inteiro. Quem recorta é o clip já empilhado, que
        // é *even-odd* de propósito. **Uma regra por pergunta.**
        scene.fill_path(
            &outer,
            &Brush::Solid(resolve(ColorToken::BgElev, theme)),
            xf,
        );
        hatch(
            scene,
            xf,
            hw + band_x,
            hh + band_y,
            px_per_world,
            sx,
            sy,
            hatch_color,
        );
        scene.pop_layer();
        stroke(scene, &outer, xf, edge, EDGE_PX / px_per_world);
        stroke(scene, &inner, xf, edge, EDGE_PX / px_per_world);
        // **O NOME, no topo à esquerda** — fora da folha, apoiado na quina, como o rótulo de uma
        // prancheta. Fora e não dentro porque dentro ele cobriria a primeira peça, que é
        // exatamente onde o empacotador põe a maior.
        // O nome **e a resolução** (Enio 2026-08-19). A resolução é a pergunta que o artista faz a
        // seguir a *"que folha é esta?"* — e é DERIVADA (densidade × tamanho), então mostrá-la
        // aqui é de graça e mantém-se sozinha em passo quando ele redimensiona ou muda a
        // densidade. Guardá-la seria o segundo tamanho que o `SpriteSheetFrame` recusa ter.
        //
        // ⚠️ `\u{00d7}` (×) e não a letra `x`: o gate `no_tofu_glyphs` conhece este glifo, e é o
        // mesmo que a linha `Source` do Inspector já usa — duas grafias para a mesma ideia é
        // como uma delas passa a parecer outra coisa.
        let label = format!(
            "{}  {} \u{00d7} {}{}",
            sim.world()
                .get::<Name>(entity)
                .map(|n| n.0.clone())
                .unwrap_or_else(|| "Sprite Sheet".to_string()),
            cfg.pixels_for(w as f32),
            cfg.pixels_for(h as f32),
            warning_suffix(health),
        );
        // A quina superior-esquerda da FAIXA em LOCAL (`+y` é para cima, então o topo é `+hh`).
        // ⚠️ Da faixa, e não do conteúdo: com a decoração agora por fora, ancorar no conteúdo
        // punha o nome em cima da própria faixa.
        draw_label(
            scene,
            text_system,
            &label,
            xf,
            (-hw - band_x, hh + band_y),
            if health.is_ok() {
                resolve(ColorToken::Text2, theme)
            } else {
                resolve(ColorToken::Danger, theme)
            },
        );
    }
}

/// As folhas da cena, com a configuração de cada uma.
fn sheets(sim: &mut SimWorld) -> Vec<(ph2d_ecs::Entity, SpriteSheetFrame)> {
    // ⚠️ `Option<&Visibility>` vai na QUERY: uma folha escondida não decora. Esconder é uma das
    // coisas que o Enio pediu que funcionasse, e uma moldura que continuasse a desenhar-se depois
    // de escondida diria que o botão não fez nada.
    let world = sim.world_mut();
    let mut q = world.query::<(
        ph2d_ecs::Entity,
        &SpriteSheetFrame,
        Option<&ph2d_ecs::Visibility>,
    )>();
    q.iter(world)
        .filter(|(_, _, vis)| !vis.is_some_and(|v| v.hidden))
        .map(|(e, cfg, _)| (e, *cfg))
        .collect()
}

/// O tamanho da folha em unidades locais — o `w`/`h` do `VecShape`, que É o tamanho (a folha
/// recusa ter um campo próprio; vide `SpriteSheetFrame`).
fn sheet_size_world(sim: &SimWorld, entity: ph2d_ecs::Entity) -> Option<(f64, f64)> {
    match sim.world().get::<VecShape>(entity)? {
        VecShape::Param { w, h, .. } => Some((w.abs(), h.abs())),
        VecShape::Text(_) => None,
    }
}

fn rect_path(x0: f64, y0: f64, x1: f64, y1: f64) -> BezPath {
    let mut p = BezPath::new();
    p.move_to(Point::new(x0, y0));
    p.line_to(Point::new(x1, y0));
    p.line_to(Point::new(x1, y1));
    p.line_to(Point::new(x0, y1));
    p.close_path();
    p
}

fn stroke(scene: &mut VectorScene, path: &BezPath, xf: Affine, color: Color, width: f64) {
    scene.inner_mut().stroke(
        &Stroke::new(width.max(f64::EPSILON)),
        xf,
        &Brush::Solid(color),
        None,
        path,
    );
}

/// As diagonais. Traçadas de lado a lado sobre a folha inteira — quem as recorta à faixa é o clip
/// que o chamador já empilhou.
#[allow(clippy::too_many_arguments)]
fn hatch(
    scene: &mut VectorScene,
    xf: Affine,
    hw: f64,
    hh: f64,
    px_per_world: f64,
    sx: f64,
    sy: f64,
    color: Color,
) {
    let step = HATCH_STEP_PX / (px_per_world * sx.min(sy));
    if !step.is_finite() || step <= 0.0 {
        return;
    }
    // A 45°, uma reta que entra pelo topo sai pela lateral: varrer de `-(w+h)` a `+(w+h)` em x
    // garante que toda a caixa é coberta, sem contas de canto.
    let span = (hw + hh) * 2.0;
    let mut x = -span;
    let mut path = BezPath::new();
    let mut guard = 0usize;
    while x <= span && guard < 4096 {
        path.move_to(Point::new(x, -hh));
        path.line_to(Point::new(x + hh * 2.0, hh));
        x += step;
        guard += 1;
    }
    stroke(
        scene,
        &path,
        xf,
        color,
        HATCH_PX / (px_per_world * sx.min(sy)),
    );
}

/// O nome, apoiado na quina superior-esquerda — **em espaço de TELA**.
///
/// ⚠️ Na tela, e não no espaço da folha, por duas razões que apontam para o mesmo lado: um rótulo
/// segue a leitura do humano, não a pose do objeto (uma folha rodada 30° não tem por que dar um
/// nome deitado — é a lei do rótulo de moldura no Figma), e o texto fica nítido em qualquer zoom
/// porque é rasterizado ao tamanho em que é lido. O que a pose decide é apenas ONDE ele pousa: a
/// quina superior-esquerda, projetada pelo mesmo afim que desenhou a faixa.
fn draw_label(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    label: &str,
    xf: Affine,
    corner_local: (f64, f64),
    color: Color,
) {
    let p = xf * Point::new(corner_local.0, corner_local.1);
    let size_px = TypeToken::Sm.px();
    ph2d_editor::paint_text(
        text_system,
        scene,
        label,
        p.x as f32,
        p.y as f32 - size_px - LABEL_GAP_PX as f32,
        size_px,
        LABEL_MAX_W_PX,
        color,
    );
}

/// O que se acrescenta ao rótulo quando a folha não está bem — vazio quando está.
///
/// ⚠️ **Palavras, não um glifo de aviso.** O `⚠` (U+26A0) não vem na Inter que o editor embute, e
/// sairia como um quadradinho — o defeito que o gate `no_tofu_glyphs` existe para apanhar (ele
/// varre os blocos das setas e dos símbolos técnicos; este viveria fora do alcance dele e chegaria
/// ao ecrã). O separador é o `\u{00b7}`, o único não-ASCII que aquele gate nomeia como seguro.
fn warning_suffix(health: crate::sheet_bounds::SheetHealth) -> &'static str {
    match (health.overlap, health.overflow) {
        (true, true) => "  \u{00b7} OVERLAP \u{00b7} DOESN'T FIT",
        (true, false) => "  \u{00b7} OVERLAP",
        (false, true) => "  \u{00b7} DOESN'T FIT",
        (false, false) => "",
    }
}

fn resolve(token: ColorToken, theme: Theme) -> Color {
    let c = token.resolve(theme);
    Color::from_rgba8(c.r, c.g, c.b, c.a)
}
