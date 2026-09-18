//! ⭐⭐⭐ **NENHUM CLIQUE DESTA SHELL INVERTE A CÂMERA COM A JANELA ERRADA.**
//!
//! # A lei, e as TRÊS vezes que ela foi paga antes deste gate existir
//!
//! O doc do [`ph2d_app_motion::field_gizmo::scene_window_wh`] escreve-a desde 2026-07-25: *«todo
//! mapeamento mundo↔tela do chrome da cena TEM de usar isto»*. Sob um split do centro (a timeline
//! aberta, o Motion) a cena desenha num sub-rectângulo `[0, 0, w, h·t]` e **a projecção muda** —
//! não é um recorte.
//!
//! | data | quem foi posto na porta | quem ficou de fora |
//! |---|---|---|
//! | 2026-07-25 | a grade e o gizmo | tudo o resto |
//! | 2026-08-25 | o **pan** (a cena andava `t` vezes o que o cursor andava) | tudo o resto |
//! | 2026-09-17 | o `vec_world_at` (o botão do HUD, `~340 px` ao lado) | **50 chamadas** |
//!
//! ⛔⛔ *Três curas, três vezes UM consumidor.* Um gate é a única coisa que muda isso: a partir
//! daqui, a chamada nova que não passe pela porta **reprova**.
//!
//! # ⚠️ O que se mede, e porquê ASSIM
//!
//! Para cada `screen_to_world(…, janela)` da shell, o último argumento é **resolvido**: um literal,
//! ou uma ligação `let`, recursivamente. Se ele acabar em `surface.size()`, é a JANELA — e reprova.
//!
//! ⛔ **Os comentários são retirados ANTES**, e isso não é higiene: sem isso o censo lê as DUAS
//! linhas de prosa que explicam a lei (`field_gizmo_host.rs`, `hud_smoke.rs`) como se fossem
//! chamadas, e a régua acusa exactamente quem a documenta. *Um censo textual tem de saber todas as
//! formas do que lê.*
//!
//! ⚠️ **Um argumento que vem de um PARÂMETRO não é resolúvel aqui** — ele resolve-se no chamador.
//! Esses estão em [`RESOLVIDOS_NO_CHAMADOR`], **com o chamador nomeado**, e a metade justa exige
//! que cada entrada ainda abrigue uma chamada real.

use std::path::{Path, PathBuf};

/// Os sítios cujo argumento é um PARÂMETRO — **com o chamador que o alimenta**, que foi lido.
///
/// ⚠️ Uma entrada aqui **não** é uma licença: ela diz *«a prova está no chamador, e é este»*. A
/// metade justa abaixo reprova a entrada que já não abriga chamada nenhuma.
const RESOLVIDOS_NO_CHAMADOR: &[(&str, &str)] = &[
    (
        "forwarding_picker.rs",
        "forwarding.rs passa `scene_mapping::janela(hero.view.center_split, ...)`",
    ),
    (
        "hover_highlight.rs",
        "os TRES sitios que constroem o `PickWorld` passam a banda (hover_highlight, \
         despacho_clique_gizmo, despacho_clique_pick)",
    ),
    (
        "input_dispatch/despacho_clique_roldana.rs",
        "despacho_clique_gizmo.rs passa `scene_mapping::janela(...)` ao `select_wheel_at`",
    ),
    (
        "input_dispatch/despacho_vetor_ops.rs",
        "os DOIS chamadores do `screen_offset_world` passam a banda \
         (despacho_metodos_janela_e_vetor e fase_component_verbs)",
    ),
];

fn raiz() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Tira comentários de linha e de bloco, e o conteúdo das strings.
///
/// ⛔ **A string também sai**: a prosa de um `panic!`/`assert!` que cite a chamada leria como
/// código, que é a forma gémea do defeito dos comentários.
fn sem_prosa(s: &str) -> String {
    let b: Vec<char> = s.chars().collect();
    let (mut out, mut i) = (String::with_capacity(s.len()), 0);
    while i < b.len() {
        if b[i] == '/' && i + 1 < b.len() && b[i + 1] == '/' {
            while i < b.len() && b[i] != '\n' {
                out.push(' ');
                i += 1;
            }
        } else if b[i] == '/' && i + 1 < b.len() && b[i + 1] == '*' {
            while i < b.len() && !(b[i] == '*' && i + 1 < b.len() && b[i + 1] == '/') {
                out.push(if b[i] == '\n' { '\n' } else { ' ' });
                i += 1;
            }
            i = (i + 2).min(b.len());
            out.push_str("  ");
        } else if b[i] == '"' {
            out.push(' ');
            i += 1;
            while i < b.len() && b[i] != '"' {
                if b[i] == '\\' {
                    i += 1;
                }
                out.push(if i < b.len() && b[i] == '\n' {
                    '\n'
                } else {
                    ' '
                });
                i += 1;
            }
            i = (i + 1).min(b.len());
            out.push(' ');
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    out
}

/// Os argumentos de topo de uma chamada que abre em `i` (logo DEPOIS do `(`).
fn argumentos(s: &[char], i: usize) -> Vec<String> {
    let (mut d, mut j, mut cur, mut out) = (1_i32, i, String::new(), Vec::new());
    while j < s.len() && d > 0 {
        match s[j] {
            '(' | '[' | '{' => d += 1,
            ')' | ']' | '}' => d -= 1,
            _ => {}
        }
        if d == 0 {
            break;
        }
        if s[j] == ',' && d == 1 {
            out.push(cur.trim().to_owned());
            cur.clear();
        } else {
            cur.push(s[j]);
        }
        j += 1;
    }
    out.push(cur.trim().to_owned());
    out
}

/// Resolve a expressão da janela até ao fundo. `Ok` = banda · `Err(motivo)` = janela ou por ver.
fn resolve(expr: &str, fonte: &str, ate: usize) -> Result<(), String> {
    let e: String = expr.split_whitespace().collect::<Vec<_>>().join(" ");
    if e.contains("scene_window") || e.contains("scene_camera_window") || e.contains("janela") {
        return Ok(());
    }
    if e.contains("surface.size()") {
        return Err("a JANELA crua".to_owned());
    }
    let id = e
        .trim()
        .trim_end_matches('?')
        .trim_start_matches('&')
        .trim();
    if !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
    {
        let base = id.split('.').next().unwrap_or(id);
        let alvo = format!("let {base} =");
        let alvo_mut = format!("let mut {base} =");
        if let Some(p) = fonte[..ate]
            .rfind(&alvo)
            .into_iter()
            .chain(fonte[..ate].rfind(&alvo_mut))
            .max()
        {
            let resto = &fonte[p..ate];
            if let Some(fim) = resto.find(';') {
                let valor = &resto[resto.find('=').map_or(0, |k| k + 1)..fim];
                return resolve(valor, fonte, p);
            }
        }
        // ⚠️ **A ligação em TUPLO** (`let (sw, sh) = scene_window_wh(…)`) — sem ela a régua lê
        //    `sw` como um parâmetro e acusa os DOIS sítios que estão certos. *Uma régua tem de
        //    conhecer todas as formas de ligar um nome, e `let (a, b) =` é uma delas.*
        let mut melhor: Option<usize> = None;
        let mut i = 0;
        while let Some(k) = fonte[i..ate].find("let (") {
            let abs = i + k;
            i = abs + 5;
            let Some(fecha) = fonte[abs..ate].find(')') else {
                continue;
            };
            if fonte[abs..abs + fecha]
                .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                .any(|t| t == base)
            {
                melhor = Some(abs + fecha);
            }
        }
        if let Some(p) = melhor
            && let Some(eq) = fonte[p..ate].find('=')
        {
            let resto = &fonte[p + eq + 1..ate];
            let fim = resto.find(';').unwrap_or(resto.len());
            return resolve(&resto[..fim], fonte, p);
        }
        return Err("um PARAMETRO".to_owned());
    }
    Err(format!("nao resolvido: {e}"))
}

/// ⭐⭐⭐ **Toda inversão da câmera da shell usa a janela da CENA.**
///
/// **Mutação que deve sangrar:** trocar um `gfx.scene_window()` por `gfx.surface.size()` em
/// qualquer sítio que alimente um `screen_to_world`.
#[test]
fn todo_aponte_passa_pela_janela_da_cena() {
    let raiz = raiz();
    let mut vistos = 0_usize;
    let mut maus: Vec<String> = Vec::new();
    let mut por_parametro: Vec<String> = Vec::new();
    let mut ficheiros: Vec<PathBuf> = Vec::new();
    junta(&raiz, &mut ficheiros);
    for p in &ficheiros {
        let rel = p
            .strip_prefix(&raiz)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/");
        let fonte = sem_prosa(&std::fs::read_to_string(p).expect("ler o fonte"));
        let ch: Vec<char> = fonte.chars().collect();
        let mut de = 0;
        while let Some(k) = fonte[de..].find("screen_to_world") {
            let abs = de + k;
            de = abs + 15;
            let Some(par) = fonte[abs..].find('(') else {
                continue;
            };
            let args = argumentos(&ch, abs + par + 1);
            // ⚠️ **DUAS formas, e a segunda não tem argumento de janela:** a `GizmoCamera` leva-a
            //    DENTRO (`window_w`/`window_h`), logo um `screen_to_world((x, y))` de UM argumento
            //    não se julga aqui — julga-se onde a struct NASCE, no gate irmão. *Uma régua que
            //    não conhece as duas formas acusa a chamada que está certa* (foi o que ela fez na
            //    primeira corrida, sobre o `field_gizmo_host`).
            if args.len() < 2 {
                continue;
            }
            let Some(janela) = args.last() else { continue };
            vistos += 1;
            let linha = fonte[..abs].matches('\n').count() + 1;
            match resolve(janela, &fonte, abs) {
                Ok(()) => {}
                Err(m) if m == "um PARAMETRO" => por_parametro.push(rel.clone()),
                Err(m) => maus.push(format!("{rel}:{linha} — {m} ({janela})")),
            }
        }
    }
    // ⛔ **Piso de população**: uma régua partida acha zero chamadas e lê-se como aprovada. O
    //    número é o MEDIDO em 2026-09-17 menos folga para a shell encolher.
    assert!(
        vistos >= 60,
        "o censo achou {vistos} chamadas — está a ler o sítio errado"
    );
    assert!(
        maus.is_empty(),
        "estes sítios invertem a câmera da cena com a JANELA e não com a BANDA — sob um split do \
         centro (a timeline aberta) o dedo aponta para outro sítio do mundo:\n  {}\n\nA cura é \
         `gfx.scene_window()`, ou `crate::scene_mapping::janela(hero.view.center_split, \
         gfx.surface.size())` quando o `hero` já está emprestado.",
        maus.join("\n  ")
    );
    let sem_nome: Vec<&String> = por_parametro
        .iter()
        .filter(|r| !RESOLVIDOS_NO_CHAMADOR.iter().any(|(f, _)| r == f))
        .collect();
    assert!(
        sem_nome.is_empty(),
        "a janela destes sítios vem de um PARÂMETRO e ninguém diz qual é o chamador — a prova tem \
         de existir em algum lado: {sem_nome:?}"
    );
}

/// ⭐ **A METADE JUSTA: cada entrada da lista ainda abriga uma chamada?**
///
/// ⛔ Sem isto a lista é uma licença aberta ao próximo sítio que caia naquele ficheiro — a doença
/// que esta casa chama de *«catraca sem censo de obsolescência»*.
#[test]
fn cada_resolvido_no_chamador_ainda_abriga_alguma_coisa() {
    let raiz = raiz();
    for (f, porque) in RESOLVIDOS_NO_CHAMADOR {
        assert!(
            porque.len() > 30,
            "a entrada `{f}` não nomeia o chamador — uma lista sem mecanismo é uma licença"
        );
        let p = raiz.join(f);
        let fonte = sem_prosa(&std::fs::read_to_string(&p).unwrap_or_default());
        assert!(
            fonte.contains("screen_to_world"),
            "a entrada `{f}` já não tem chamada nenhuma — apague-a, senão fica aberta para a próxima"
        );
    }
}

/// ⭐⭐ **E a `GizmoCamera` — a forma que leva a janela DENTRO — nasce da banda.**
///
/// ⛔ Ela é o ponto cego do gate acima **por construção**: ali o `screen_to_world` recebe só o
/// ponto, e a janela entrou na struct linhas antes. Sem esta metade, seis gizmos de arrasto podiam
/// voltar à janela crua com o censo principal VERDE.
///
/// **Mutação que deve sangrar:** trocar o `gfx.scene_window()` de um `let size = …` que alimente
/// uma `GizmoCamera`.
#[test]
fn toda_gizmo_camera_nasce_da_banda() {
    let raiz = raiz();
    let mut ficheiros: Vec<PathBuf> = Vec::new();
    junta(&raiz, &mut ficheiros);
    let (mut vistas, mut maus) = (0_usize, Vec::new());
    for p in &ficheiros {
        let rel = p
            .strip_prefix(&raiz)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/");
        let fonte = sem_prosa(&std::fs::read_to_string(p).expect("ler o fonte"));
        let mut de = 0;
        while let Some(k) = fonte[de..].find("GizmoCamera {") {
            let abs = de + k;
            de = abs + 13;
            let Some(w) = fonte[abs..].find("window_w:") else {
                continue;
            };
            let campo = &fonte[abs + w + 9..];
            let Some(fim) = campo.find(',') else { continue };
            vistas += 1;
            let expr = campo[..fim].replace(" as f32", "").replace(".width", "");
            if resolve(&expr, &fonte, abs).is_err() {
                let linha = fonte[..abs].matches('\n').count() + 1;
                maus.push(format!("{rel}:{linha} — window_w vem de `{}`", expr.trim()));
            }
        }
    }
    assert!(
        vistas >= 5,
        "o censo achou {vistas} `GizmoCamera` — está a ler o sítio errado"
    );
    assert!(
        maus.is_empty(),
        "estas `GizmoCamera` nascem da JANELA e não da BANDA — o arrasto delas aponta para outro \
         sítio do mundo com a timeline aberta:\n  {}",
        maus.join("\n  ")
    );
}

/// ⭐⭐⭐ **A propriedade que torna a troca de ~90 sítios SEGURA: fora do split é a janela inteira.**
///
/// ⛔ Sem ela esta wave mudaria a imagem de toda a gente; com ela, o caminho de omissão do app
/// (centro NÃO dividido) é **byte-idêntico** ao de antes, e só a timeline/o Motion abertos veem a
/// diferença — que é precisamente onde o dedo estava a apontar para o sítio errado.
///
/// ⚠️ **O gate da identidade já existe e vive noutra crate** (`field_gizmo_tests`,
/// `scene_camera_window(CenterSplit::None, win) == win`). O que esta metade prova é que a porta
/// desta shell **HERDA** essa prova — ela DELEGA, em vez de reescrever a conta. *Uma segunda conta
/// com a mesma intenção é como as três cópias nasceram.*
///
/// A shell é um BINÁRIO (sem `[lib]`), logo um teste de integração não a pode chamar: a prova é
/// sobre o FONTE, e é por isso que ela é uma afirmação de DELEGAÇÃO e não de valor.
///
/// **Mutação que deve sangrar:** reescrever o corpo de `janela`/`scene_window` com a conta à mão.
#[test]
fn a_porta_delega_e_por_isso_herda_a_identidade_fora_do_split() {
    let fonte = std::fs::read_to_string(raiz().join("scene_mapping.rs")).expect("a porta existe");
    let corpo = sem_prosa(&fonte);
    let n = corpo.matches("scene_camera_window(").count();
    assert!(
        n >= 2,
        "a porta tem de DELEGAR nas duas formas (achei {n} chamadas a `scene_camera_window`) — \
         reescrever a conta aqui perderia a prova de identidade que vive na crate do Motion"
    );
    for proibido in ["h *", "* t", "floor()"] {
        assert!(
            !corpo.contains(proibido),
            "a porta está a FAZER a conta (`{proibido}`) em vez de a delegar — é assim que nasce a \
             quarta cópia"
        );
    }
}

/// ⭐⭐⭐ **QUANTO é que o dedo errava — o número, e a exigência de que ele EXISTA.**
///
/// Sem isto, os censos acima são afirmações sobre TEXTO. Esta metade mede a coisa: com o centro
/// partido a `55 %` (a timeline aberta, a arrumação do dono) e a superfície da foto de 17/09
/// (`1930×1012`), o MESMO pixel de ecrã resolve para dois pontos do mundo diferentes.
///
/// ⛔ **E ela exige que a divergência seja GRANDE**, não que seja pequena: um tecto que aceitasse
/// «quase igual» tornaria a cura opcional. *A régua que aprova os dois lados não separa nada.*
///
/// ⚠️ **Os DOIS eixos**, e não só o `y`: o `aspect = w/h` cresce quando o `h` encolhe, logo o
/// `half_w` cresce com ele. A leitura *«está deslocado para baixo»* é o sintoma mais visível, não
/// a conta.
#[test]
fn a_janela_errada_aponta_para_outro_sitio_do_mundo() {
    use ph2d_editor_core::screens::layout::CenterSplit;
    use ph2d_host::WindowSize;

    let janela = WindowSize::new(1930, 1012);
    let banda = ph2d_app_motion::field_gizmo::scene_camera_window(
        CenterSplit::Horizontal { t: 0.55 },
        janela,
    );
    assert_ne!(
        banda.height, janela.height,
        "a fixtura não parte o centro — este gate mediria o nada"
    );
    let cam = ph2d_render::Camera2d::new([0.0, 0.0], 10.0);
    // O centro do botão do HUD na foto de 17/09.
    let ponto = (965.0_f32, 432.0_f32);
    let certo = cam.screen_to_world(ponto, banda);
    let errado = cam.screen_to_world(ponto, janela);
    let dx = f64::from((certo[0] - errado[0]).abs());
    let dy = f64::from((certo[1] - errado[1]).abs());
    assert!(
        dy > 1.0,
        "o eixo Y tinha de divergir mais de 1 metro e divergiu {dy:.3} — se a divergência \
         desaparecer, ou a lei do split mudou, ou esta régua deixou de a medir"
    );
    // ⚠️ No centro horizontal da vista o `x` coincide por SIMETRIA (`nx = 0` anula o `half_w`).
    //    Fora dele não coincide, e é isso que se afirma — senão alguém lê «é só o y» como lei.
    let fora = (1700.0_f32, 432.0_f32);
    let dx_fora = f64::from(
        (cam.screen_to_world(fora, banda)[0] - cam.screen_to_world(fora, janela)[0]).abs(),
    );
    assert!(
        dx < 1e-6 && dx_fora > 1.0,
        "no centro o X coincide por simetria ({dx:.6}) e fora dele NÃO ({dx_fora:.3}) — se esta \
         relação se inverter, a conta do `aspect` mudou"
    );
    println!(
        "[banda] com o centro a 55%: o mesmo pixel resolve {dy:.2} m ao lado no Y e {dx_fora:.2} m no X"
    );
}

/// A janela CRUA é legítima aqui — `(ficheiro, porquê)`, e o porquê foi LIDO.
///
/// ⚠️ **Não é uma lista de dívida: é a partição.** Nem toda `surface.size()` é um defeito — há
/// consumidores cujo assunto É a janela (configurar a superfície, a câmera do JOGO). A metade justa
/// abaixo reprova a entrada que já não abriga nada.
const A_JANELA_E_O_ASSUNTO: &[(&str, &str)] = &[(
    "render_loop/fase_game_camera.rs",
    "a camera do JOGO (TOP-20 #7): o `aspect_of` mede o ecra' do jogador, nao a banda do chrome \
     — e' outro assunto, e a regua larga apanha-o como falso positivo",
)];

/// O nome e os argumentos da chamada que envolve a posição `pos`.
fn chamada_envolvente(fonte: &str, pos: usize) -> Option<(String, String)> {
    let b: Vec<char> = fonte.chars().collect();
    let (mut d, mut i) = (0_i32, pos);
    let abre = loop {
        if i == 0 {
            return None;
        }
        i -= 1;
        match b[i] {
            ')' => d += 1,
            '(' if d == 0 => break i,
            '(' => d -= 1,
            _ => {}
        }
    };
    let mut j = abre;
    while j > 0
        && (b[j - 1].is_alphanumeric() || b[j - 1] == '_' || b[j - 1] == ':' || b[j - 1] == '.')
    {
        j -= 1;
    }
    let nome: String = b[j..abre].iter().collect();
    let (mut d2, mut k) = (1_i32, abre + 1);
    while k < b.len() && d2 > 0 {
        match b[k] {
            '(' => d2 += 1,
            ')' => d2 -= 1,
            _ => {}
        }
        k += 1;
    }
    Some((nome, b[abre..k.min(b.len())].iter().collect()))
}

/// ⭐⭐⭐ **O OUTRO LADO DO PAR: quem DESENHA também usa a banda.**
///
/// O gate de cima cobre onde o dedo APONTA. Este cobre onde a coisa é PINTADA — e o par tem de
/// concordar, senão o realce da selecção fica ao lado do objecto que ele realça.
///
/// ⚠️⚠️ **Uma `surface.size()` ao lado de um `camera` NÃO é prova de defeito, e isto custou-me uma
/// acusação errada:** o handoff da manhã listou quatro sítios como dívida e **três já estavam
/// certos** — o consumidor recebe o `center_split` em SEPARADO e deriva a banda lá dentro
/// (`scene_px_per_world`, `publish_editor_inputs`, `draw_warp_gizmo`). ⇒ a régua aceita **as duas**
/// formas de estar certo: passar a banda, ou passar o split ao lado.
///
/// **Mutação que deve sangrar:** tirar o `janela(…)` de um realce do `fase_selection_highlight`.
#[test]
fn quem_desenha_no_mundo_tambem_usa_a_banda() {
    let raiz = raiz();
    let mut ficheiros: Vec<PathBuf> = Vec::new();
    junta(&raiz, &mut ficheiros);
    let (mut vistas, mut maus) = (0_usize, Vec::new());
    for p in &ficheiros {
        let rel = p
            .strip_prefix(&raiz)
            .unwrap_or(p)
            .to_string_lossy()
            .replace('\\', "/");
        if A_JANELA_E_O_ASSUNTO.iter().any(|(f, _)| rel == *f) {
            continue;
        }
        let fonte = sem_prosa(&std::fs::read_to_string(p).expect("ler o fonte"));
        let mut de = 0;
        while let Some(k) = fonte[de..].find("surface.size()") {
            let abs = de + k;
            de = abs + 14;
            let Some((nome, args)) = chamada_envolvente(&fonte, abs) else {
                continue;
            };
            // ⚠️⚠️ **O `camera` pode ser o RECEPTOR e não um argumento** — e em várias linhas
            //    (`gfx\n.camera\n.world_to_screen(…)`) o nome extraído é só `.world_to_screen`.
            //    Sem olhar o texto ANTES da chamada, a régua passava ao lado da sonda do undo do
            //    osso, que esta mesma wave tinha acabado de partir. *Uma régua que só vê argumentos
            //    é cega a metade das chamadas de método.* (Há prova de mutação sobre esta linha.)
            let antes: String = fonte[abs.saturating_sub(90)..abs]
                .split_whitespace()
                .collect::<Vec<_>>()
                .join("");
            if !antes.contains("camera")
                && !args.contains("camera")
                && !args.contains("height_world")
            {
                continue;
            }
            vistas += 1;
            let ok = nome.contains("scene_camera_window")
                || nome.contains("scene_window_wh")
                || nome.contains("janela")
                || args.contains("center_split")
                || args.contains("scene_window()")
                || antes.contains("center_split");
            if !ok {
                let linha = fonte[..abs].matches('\n').count() + 1;
                let nome = nome.trim().to_owned();
                maus.push(format!("{rel}:{linha} — `{nome}(…)`"));
            }
        }
    }
    assert!(
        vistas >= 14,
        "o censo achou {vistas} chamadas com câmera — está a ler o sítio errado"
    );
    assert!(
        maus.is_empty(),
        "estes sítios dão a JANELA a uma conta que leva a CÂMERA da cena, sem lhe dar o split — com \
         o centro partido o que se pinta fica ao lado do que se vê:\n  {}\n\nAs DUAS curas valem: \
         passar `crate::scene_mapping::janela(split, size)`, ou passar o `center_split` ao lado (o \
         consumidor deriva a banda).",
        maus.join("\n  ")
    );
}

/// ⭐ **A metade justa da partição acima.**
#[test]
fn cada_janela_que_e_o_assunto_ainda_abriga_alguma_coisa() {
    for (f, porque) in A_JANELA_E_O_ASSUNTO {
        assert!(
            porque.len() > 40,
            "a entrada `{f}` não diz o mecanismo — uma lista sem mecanismo é uma licença"
        );
        let fonte = std::fs::read_to_string(raiz().join(f)).unwrap_or_default();
        assert!(
            fonte.contains("surface.size()"),
            "a entrada `{f}` já não tem `surface.size()` — apague-a"
        );
    }
}

fn junta(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            junta(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}
