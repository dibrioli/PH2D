//! RENDER-AND-LOOK do arco palido da aquarela.
//!
//! # Por que uma sonda que DESENHA, e nao mais um numero
//!
//! Esta sessao produziu SEIS fixtures escalares seguidas cujo numero descrevia outra coisa que
//! nao o defeito reportado (o `docs/Painter/32` §6 as lista). A ultima delas — a janela da
//! concavidade — mede "clareamento acima da mediana local" num retangulo que **tambem contem a
//! transicao tinta/papel do vao da cruz**, entao ela reporta um contraste grande em Dilution 0,00
//! (onde a tinta e densa) e pequeno em 0,45 (onde e palida) — exatamente o INVERSO do que o Enio
//! ve. O numero nao esta errado; ele esta a responder outra pergunta.
//!
//! O repo ja tem o precedente para isto: `push_look_probe` (shell) e `fx_look` (vec-scene)
//! **desenham** e deixam o olho decidir. O escritor de PNG abaixo e o mesmo de
//! `ph2d-vec-scene/tests/look/mod.rs` — blocos deflate ARMAZENADOS, **zero dependencia**: uma
//! sonda de diagnostico nao precisa de ficheiro pequeno, e uma dep nova por causa dela seria o
//! preco errado.
//!
//! # Rodar
//!
//! ```text
//! env PH2D_WC_LOOK_DIR=/tmp/wc cargo test -p ph2d-tool-painter --release \
//!     probe_watercolor_arc -- --ignored --nocapture
//! ```
//!
//! Sem a variavel a sonda **nao escreve nada** e diz porque — um probe que escreve em sitio
//! escolhido por ele proprio e um probe que suja a arvore de quem so correu a suite.

use super::measure_watercolor_water_edge::wash_over_dry;

const SIDE: usize = 256;

/// Uma tela RGB de 8 bits. Nao ha alfa de proposito: o que se julga aqui e o que a tela MOSTRA.
struct Canvas {
    w: usize,
    h: usize,
    px: Vec<[u8; 3]>,
}

impl Canvas {
    fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            px: vec![[0, 0, 0]; w * h],
        }
    }

    fn set(&mut self, x: usize, y: usize, c: [u8; 3]) {
        if x < self.w && y < self.h {
            self.px[y * self.w + x] = c;
        }
    }
}

/// O canvas do produto (RGBA premultiplicado sobre papel branco) como RGB opaco.
///
/// ⚠️ O `canvas_rgba` do Painter e **straight alpha sobre papel**: o papel ja esta la (o fixture
/// semeia 255). Compor de novo contra branco escureceria a tinta duas vezes, entao aqui e uma
/// copia de canal, nao um `over`.
fn as_rgb(px: &[u8], x0: usize, y0: usize, w: usize, h: usize, zoom: usize) -> Canvas {
    let mut c = Canvas::new(w * zoom, h * zoom);
    for row in 0..h {
        for col in 0..w {
            let i = ((y0 + row) * SIDE + (x0 + col)) * 4;
            let rgb = [px[i], px[i + 1], px[i + 2]];
            for dy in 0..zoom {
                for dx in 0..zoom {
                    c.set(col * zoom + dx, row * zoom + dy, rgb);
                }
            }
        }
    }
    c
}

// ── PNG sem dependencias (porte verbatim de ph2d-vec-scene/tests/look/mod.rs) ───────────────

fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, e) in table.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xEDB8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
        *e = c;
    }
    let mut c = 0xFFFF_FFFFu32;
    for b in data {
        c = table[((c ^ u32::from(*b)) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], body: &[u8]) {
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    let mut full = kind.to_vec();
    full.extend_from_slice(body);
    out.extend_from_slice(&full);
    out.extend_from_slice(&crc32(&full).to_be_bytes());
}

fn write_png(path: &std::path::Path, c: &Canvas) -> std::io::Result<()> {
    let mut raw = Vec::with_capacity(c.h * (1 + c.w * 3));
    for y in 0..c.h {
        raw.push(0); // filtro None
        for x in 0..c.w {
            raw.extend_from_slice(&c.px[y * c.w + x]);
        }
    }
    let mut z = vec![0x78, 0x01];
    for (i, block) in raw.chunks(65_535).enumerate() {
        let last = u8::from((i + 1) * 65_535 >= raw.len());
        z.push(last);
        z.extend_from_slice(&(block.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(block.len() as u16)).to_le_bytes());
        z.extend_from_slice(block);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for byte in &raw {
        a = (a + u32::from(*byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());

    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(c.w as u32).to_be_bytes());
    ihdr.extend_from_slice(&(c.h as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]); // 8 bits, truecolor
    chunk(&mut png, b"IHDR", &ihdr);
    chunk(&mut png, b"IDAT", &z);
    chunk(&mut png, b"IEND", &[]);
    std::fs::write(path, png)
}

/// Desenha a cruz `wash_over_dry` e escreve o que a tela mostra.
///
/// ⚠️ **A cena e a MESMA das sondas escalares** (mesmo pincel real: `Falloff::Watercolor`,
/// raio 72, o que o modo aquarela de facto produz — o `Falloff::Constant` que as quatro
/// primeiras sondas usavam e um pincel que este modo **nao consegue** montar). O que muda e o
/// instrumento: aqui o oraculo e o olho, e o olho nao confunde "o vao da cruz e papel" com
/// "ha um arco palido acompanhando a concavidade".
#[test]
#[ignore = "sonda de diagnostico: escreve PNG, roda com PH2D_WC_LOOK_DIR"]
fn probe_watercolor_arc() {
    let Ok(dir) = std::env::var("PH2D_WC_LOOK_DIR") else {
        eprintln!(
            "probe_watercolor_arc: defina PH2D_WC_LOOK_DIR=<dir> para escrever os PNG.\n\
             Sem ela a sonda nao escreve nada — um probe nao escolhe sozinho onde sujar."
        );
        return;
    };
    std::fs::create_dir_all(&dir).expect("criar o diretorio de saida");

    for (tag, dilution) in [("d000", 0.00f32), ("d045", 0.45)] {
        for (edges, smooth) in [("smooth", true), ("hard", false)] {
            let px = wash_over_dry(dilution, smooth);

            // A cena inteira: e ela que diz se o arco e local ou se a lavagem toda mudou.
            let full = as_rgb(&px, 0, 0, SIDE, SIDE, 2);
            let p = format!("{dir}/{tag}_{edges}_full.png");
            write_png(std::path::Path::new(&p), &full).expect("escrever o PNG");

            // A concavidade INFERIOR-DIREITA da cruz, 4x. As quatro quinas sao equivalentes por
            // construcao (a faixa e horizontal, a vertical cruza no meio), entao uma basta.
            let crop = as_rgb(&px, 128, 100, 96, 96, 4);
            let p = format!("{dir}/{tag}_{edges}_quina.png");
            write_png(std::path::Path::new(&p), &crop).expect("escrever o PNG");
        }
        eprintln!("probe_watercolor_arc: Dilution {dilution:.2} escrito em {dir}");
    }
}

/// O MIOLO e a BANDA DE BORDA da lavagem, por dilucao.
///
/// # Por que um escalar e honesto AQUI
///
/// As sondas escalares desta sessao falharam por medirem janelas que continham uma borda **e**
/// papel nu, sem saber qual das duas coisas o numero descrevia. Esta mede duas grandezas de
/// significado unico ao longo de UMA coluna que atravessa UM flanco: o alfa do **centro** da
/// faixa (a dezenas de texels de qualquer contorno) e a **largura** da transicao 10%-90% desse
/// flanco. Nao ha quina, nao ha vao, nao ha segunda borda dentro da amostra.
///
/// # A TABELA, medida na coluna limpa e com o motor INTEIRO
///
/// ```text
///   dilution   flow   alfa MIOLO   y10%   y90%   BANDA (px)
///       0,00   1,00        0,880     23     31            8
///       0,15   0,85        0,880     24     33            9
///       0,30   0,70        0,880     26     37           11
///       0,45   0,55        0,876     28     42           14
///       0,60   0,40        0,733     31      -   nunca fecha
/// ```
///
/// O miolo fica PLANO ate D0,45 e so cede em 0,60; o que a dilucao move e a BANDA, monotonica,
/// ate o flanco deixar de alcancar 90% em lugar nenhum. ⚠️ Uma versao anterior desta tabela
/// dizia `6 / 7 / 13` e *"de D0,45 em diante nunca alcanca 90%"*: aqueles numeros sairam da
/// coluna `x = 64`, que esta DENTRO do alcance do traco vertical. A troca para `x = 40` e a
/// unica diferenca — o backrun e inerte aqui, e a prova e a linha D0,00, onde `water = dilution`
/// vale zero nos dois casos e o numero ainda assim se move.
///
/// ⚠️ **E a previsao aritmetica que eu escrevi aqui foi DERRUBADA pela primeira corrida.** Eu
/// previa que a dilucao empurrasse o miolo para o pe do `smoothstep(SS0, SS1, coverage x flow)`
/// e que o alfa interior caisse de 0,62 para 0,11; medido, ele fica em **0,912 em toda a faixa
/// ate D0,45** e so cede em 0,60. O modelo estava errado (a cobertura de um texel interior nao e
/// a opacidade do pincel — o envelope `max` sobre dabs sobrepostos satura muito acima dela), e o
/// que sobra e a coluna que ele nao previa: **a largura da BANDA**.
#[test]
#[ignore = "sonda de diagnostico"]
fn measure_the_edge_band_across_dilutions() {
    eprintln!("\n=== O MIOLO E A BANDA DE BORDA (pincel real, coluna x=40) ===\n");
    eprintln!("dilution   flow   alfa MIOLO   y10%   y90%   BANDA (px)");
    for dilution in [0.00f32, 0.15, 0.30, 0.45, 0.60] {
        let px = wash_over_dry(dilution, true);
        // A faixa horizontal corre em y=90 com raio 72. ⚠️ A primeira corrida desta sonda usou a
        // coluna x=64, que esta DENTRO do alcance do traco vertical (cx=128, raio 72 => 56..200):
        // o flanco medido ali carregava um piso da vertical e nao era limpo. x=40 fica fora dele
        // (|40-128| = 88 > 72) e longe das pontas da faixa, entao a coluna cruza UM flanco so.
        // ⚠️ Os limiares sao RELATIVOS ao miolo, nunca absolutos. Uma lavagem diluida e mais
        // PALIDA por desenho: com o miolo em 0,69 um limiar fixo de 0,90 nunca e cruzado, e a
        // sonda reportaria "a borda nunca fecha" sobre uma borda perfeitamente nitida. A
        // pergunta e a LARGURA DA TRANSICAO, e ela so tem sentido na escala do proprio miolo.
        let core = alpha_at(&px, 40, 90);
        let mut y10 = None;
        let mut y90 = None;
        for y in 0..90 {
            let a = alpha_at(&px, 40, y);
            if y10.is_none() && a >= 0.10 * core {
                y10 = Some(y);
            }
            if y90.is_none() && a >= 0.90 * core {
                y90 = Some(y);
            }
        }
        let banda = match (y10, y90) {
            (Some(a), Some(b)) => format!("{}", b.saturating_sub(a)),
            _ => "—".to_string(),
        };
        eprintln!(
            "{dilution:8.2} {:6.2} {:12.3} {:6} {:6} {banda:>11}",
            1.0 - dilution,
            core,
            y10.map_or("—".to_string(), |v| v.to_string()),
            y90.map_or("—".to_string(), |v| v.to_string()),
        );
    }
    eprintln!(
        "\nA janela de endurecimento `smoothstep(SS0=0,12 · SS1=0,60, cobertura)` e ABSOLUTA e a\n\
         dilucao ESCALA a cobertura que entra nela — entao o que ela move e ONDE o flanco cruza a\n\
         janela, nao o valor do miolo saturado.\n"
    );
}

/// O flanco de uma lavagem, medido na coluna LIMPA `x = 40` (fora do alcance do traco vertical) —
/// devolve `(miolo, inicio do flanco, largura da banda 10%-90%)`. Os limiares sao RELATIVOS ao
/// miolo: veja o aviso em [`measure_the_edge_band_across_dilutions`].
fn wash_flank(dilution: f32) -> (f32, usize, usize) {
    let px = wash_over_dry(dilution, true);
    let core = alpha_at(&px, 40, 90);
    let (mut y10, mut y90) = (None, None);
    for y in 0..90 {
        let a = alpha_at(&px, 40, y);
        if y10.is_none() && a >= 0.10 * core {
            y10 = Some(y);
        }
        if y90.is_none() && a >= 0.90 * core {
            y90 = Some(y);
        }
    }
    let (a, b) = (
        y10.expect("o flanco tem de cruzar 10% do miolo"),
        y90.expect("o flanco tem de cruzar 90% do miolo"),
    );
    (core, a, b.saturating_sub(a))
}

/// **A DILUICAO E QUANTA TINTA, NUNCA ONDE A LAVAGEM ESTA** — a lei que o arco palido do Enio
/// (2026-08-11) pagou, e a razao de a `flow` viver em
/// [`super::watercolor_field::style::wash_flow`] em vez de multiplicar a cobertura.
///
/// Tres afirmacoes, e nenhuma delas basta sozinha:
///
/// - **(a)** o inicio do flanco NAO anda com a dilucao. Este e o arco: a lei antiga escalava a
///   cobertura antes de um `smoothstep(SS0, SS1, ..)` de limiares ABSOLUTOS, entao a dilucao movia
///   ONDE o flanco cruza a janela em vez de quanto pigmento ha ali — a silhueta encolhia para
///   dentro (medido, `y10%` 23 -> 30 na faixa toda).
/// - **(b)** a banda de transicao nao CRESCE. A mesma causa a abria (7 -> 11 px); uma silhueta que
///   so encolhesse rigidamente passaria em (a) e falharia aqui.
/// - **(c)** o knob de facto DILUI. Sem esta metade, "nao mover a borda" seria satisfeito por uma
///   `flow` inerte — e era quase o caso na lei antiga, cujo miolo ficava plano ate `dilution` 0,4
///   porque o `smoothstep` satura em `SS1 = 0,60`.
///
/// ⚠️ **`dilution = 0` e o CONTROLE** — ele mede o mundo que ja shipava, e as tres asserções sao
/// comparacoes CONTRA ele, nunca numeros absolutos: uma barra literal aqui pinaria o desenho da
/// lavagem, que nao e o assunto deste gate.
///
/// ⚠️ **E este gate NAO basta sozinho — o irmao dele e o
/// `watercolor_clean_water_backrun_blooms_on_wet_wash`.** Medido: pondo a `flow` no `depth` em vez
/// do `fill`, as tres asserções daqui passam VERDES e o do backrun sangra (a agua pura apagava o
/// proprio anel). Um mede *onde a lavagem esta*, o outro *de quem e o pigmento* — nenhum dos dois
/// enxerga a metade do outro.
#[test]
fn dilution_thins_the_wash_without_moving_its_edge() {
    let (core_dry, start_dry, band_dry) = wash_flank(0.00);
    let (core_wet, start_wet, band_wet) = wash_flank(0.60);
    // (a) ±1 px, nao igualdade estrita: a coluna e amostrada por texel, entao a fronteira pode
    // pousar de um lado ou do outro de um pixel sem que a silhueta tenha se movido.
    assert!(
        start_wet.abs_diff(start_dry) <= 1,
        "a dilucao moveu a SILHUETA: o flanco comeca em y={start_dry} seco e y={start_wet} \
         diluido (o arco palido). A `flow` voltou para a cobertura?"
    );
    assert!(
        band_wet <= band_dry,
        "a dilucao ABRIU a borda: banda {band_dry} px seca contra {band_wet} px diluida"
    );
    assert!(
        core_wet < core_dry * 0.80,
        "a dilucao nao diluiu o miolo: {core_dry:.3} seco contra {core_wet:.3} diluido \
         (a `flow` precisa entrar na densidade, onde nada satura)"
    );
}

/// O papel e branco e a tinta e `[0.90, 0.15, 0.18]`: o canal VERDE mede a cobertura
/// (`255` = papel nu, `38` = tinta cheia), entao `alfa = (255 - g) / (255 - 38)`.
fn alpha_at(px: &[u8], x: usize, y: usize) -> f32 {
    let i = (y * SIDE + x) * 4;
    f32::from(255 - px[i + 1]) / (255.0 - 38.0)
}

/// ⚠️ **REFUTADA PELA [`measure_the_depletion_along_the_stroke`] — o oraculo desta sonda NAO
/// vale, e o numero dela nao sustenta a conclusao que eu tirei dele.** Ela compara dois pontos
/// a mesma PROFUNDIDADE na banda de borda, um perto do comeco do traco e outro perto do fim, e
/// assume que a unica diferenca entre eles e a quina. Medido: ao longo de um traco RETO, sem
/// quina em lugar nenhum, o alfa a profundidade constante varia de **0,710 a 0,000** (o miolo,
/// no mesmo traco, fica plano em 0,866-0,917). A banda de borda oscila de posicao ao longo do
/// traco — dente do papel, granulacao — e uma amostra a meio de uma rampa ingreme amplifica
/// esse deslocamento ate a escala inteira. O controle `x = 40` calhou de ser um pico e a quina
/// `x ~ 191` um vale.
///
/// A sonda FICA, com este aviso, porque o que ela mediu e real (a banda oscila) e porque a
/// conclusao errada que eu tirei dela — *"o 2o traco APAGA a tinta seca na quina"* — chegou a
/// entrar numa mensagem de commit. Para medir quina de verdade o controle tem de estar no MESMO
/// comprimento de arco, e o oraculo nao pode ser um ponto no meio da rampa.
///
/// **A RETRACAO DA QUINA CONCAVA** — a sonda que mede o *arco palido* que o Enio fotografou, com
/// DOIS controles dentro da mesma imagem.
///
/// # O oraculo, e por que ele nao depende de eu adivinhar nada
///
/// A fixture e um sinal de MAIS: a faixa horizontal (dabs em `y=90`, raio 72 => a borda de cima
/// mora em `y=18`) e a vertical que a atravessa (dabs em `x=128` => a borda da direita mora em
/// `x=200`). O vertice REFLEXO — a concavidade — fica perto de `(200, 18)`, e a bissetriz dele
/// aponta para dentro do corpo, na direcao `(-1, +1)`.
///
/// Ao longo dessa bissetriz, no ponto `(200 - k, 18 + k)`, as DUAS coberturas valem exatamente a
/// mesma coisa: a distancia radial ate a linha de dabs horizontal e `72 - k`, e ate a vertical
/// tambem e `72 - k`. Sob o envelope `max` que o deposito de fato usa, isso obriga o perfil da
/// bissetriz a ser **identico ao perfil de um flanco RETO** na mesma profundidade `k` — ou seja,
/// o contorno de uma quina concava e um angulo RETO, sem arredondamento e sem palidez.
///
/// **Toda diferenca entre as tres colunas abaixo e o defeito**, e ela e medida contra dois
/// controles tirados da MESMA imagem, no MESMO instante, com a MESMA tinta:
///
/// - `flanco H` em `(40, 18 + k)` — o 1o traco sobre papel. `|40 - 128| = 88 > 72`, entao a
///   vertical nao alcanca a coluna: e um flanco de um dono so.
/// - `flanco V` em `(200 - k, 200)` — o 2o traco sobre papel. `|200 - 90| = 110 > 72`, entao a
///   faixa horizontal nao alcanca a linha.
/// - `QUINA` em `(200 - k, 18 + k)` — a bissetriz, onde o 2o traco pinta sobre o pigmento SECO do
///   1o.
///
/// ⚠️ **Perto de `k = 0` a quina nao e um vertice exato**: a vertical comeca em `y=30`, entao a
/// borda dela ali e o ARCO da tampa, e as duas bordas se cruzam por volta de `(199, 18)`. O erro
/// e sub-pixel e nao decide nada — o que se le e a coluna inteira, nao um texel.
#[test]
#[ignore = "sonda de diagnostico"]
fn measure_the_concave_corner_retreat() {
    for dilution in [0.00f32, 0.45] {
        let px = wash_over_dry(dilution, true);
        eprintln!("\n=== A QUINA CONCAVA vs DOIS FLANCOS RETOS — Dilution {dilution:.2} ===\n");
        eprintln!("  k   flanco H   flanco V   max(H,V)     QUINA   quina-max");
        for k in 0..26usize {
            let h = alpha_at(&px, 40, 18 + k);
            let v = alpha_at(&px, 200 - k, 200);
            let q = alpha_at(&px, 200 - k, 18 + k);
            let m = h.max(v);
            eprintln!("{k:3} {h:10.3} {v:10.3} {m:10.3} {q:9.3} {:11.3}", q - m);
        }
    }

    eprintln!("\n=== ONDE CADA CONTORNO CRUZA (profundidade k em px) ===\n");
    eprintln!("dilution   sitio      k@10%   k@50%   k@90%");
    for dilution in [0.00f32, 0.15, 0.30, 0.45, 0.60] {
        let px = wash_over_dry(dilution, true);
        for (name, probe) in [("flanco H", 0u8), ("flanco V", 1), ("QUINA   ", 2)] {
            let at = |k: usize| -> f32 {
                match probe {
                    0 => alpha_at(&px, 40, 18 + k),
                    1 => alpha_at(&px, 200 - k, 200),
                    _ => alpha_at(&px, 200 - k, 18 + k),
                }
            };
            let cross = |thr: f32| -> String {
                (0..60)
                    .find(|&k| at(k) >= thr)
                    .map_or_else(|| "—".to_string(), |k| k.to_string())
            };
            eprintln!(
                "{dilution:8.2}   {name}  {:>6}  {:>6}  {:>6}",
                cross(0.10),
                cross(0.50),
                cross(0.90),
            );
        }
    }
    eprintln!(
        "\nSob o envelope `max` as tres colunas TEM de coincidir. Se a QUINA precisar de mais\n\
         profundidade para alcancar o mesmo alfa, o contorno dela RECUOU para dentro — e o recuo\n\
         em px, por limiar, e o tamanho do arco palido.\n"
    );
}

/// **O 2o TRACO CLAREIA O QUE O 1o JA TINHA DEIXADO?** — a pergunta que a sonda da quina
/// obrigou, e que e a frase do proprio report do Enio (*"pintando sobre o pigmento ja seco"*).
///
/// # Por que esta pergunta, e por que ela nao e mais uma hipotese
///
/// A [`measure_the_concave_corner_retreat`] mediu a bissetriz da concavidade contra dois flancos
/// retos da MESMA imagem. Sob o envelope `max` que o deposito usa, os tres TEM de coincidir na
/// mesma profundidade — e a quina lia **0,000 onde o flanco lia 0,152**, dez texels adentro.
/// Um envelope `max` nao consegue produzir isso: `max` nunca DEVOLVE menos do que um dos lados
/// ja tinha. Entao a diferenca nao esta no deposito — ela esta em alguma coisa que **retira**.
///
/// A fixture e a MESMA cruz, agora com o canvas capturado nos DOIS instantes
/// ([`wash_two_stages`]): a faixa horizontal sozinha, e depois dela com a vertical por cima.
/// Um texel que **perde** alfa entre as duas capturas e tinta que o 2o traco APAGOU — e o
/// conjunto desses texels e a forma do defeito.
///
/// ⚠️ **O CONTROLE mora na propria imagem:** longe da vertical (`|x - 128| > 72`) o 2o traco nao
/// alcanca nada, entao ali a diferenca TEM de ser zero. Se nao for, o que a sonda mede nao e o
/// cruzamento — e outra coisa, e o numero nao serve para nada.
#[test]
#[ignore = "sonda de diagnostico"]
fn measure_what_the_second_stroke_takes_away() {
    eprintln!("\n=== O QUE O 2o TRACO RETIRA DO 1o (cruz, canvas inteiro) ===\n");
    eprintln!("dilution   texels que PERDERAM   pior perda   |   fora do alcance (controle)");
    for dilution in [0.00f32, 0.15, 0.30, 0.45, 0.60] {
        let (one, two) = wash_over_dry_two_stages(dilution);
        let (mut lost, mut worst, mut outside) = (0usize, 0.0f32, 0usize);
        for y in 0..SIDE {
            for x in 0..SIDE {
                let d = alpha_at(&one, x, y) - alpha_at(&two, x, y);
                if d > 0.01 {
                    lost += 1;
                    worst = worst.max(d);
                    // O 2o traco corre em x=128 com raio 72: fora disso ele nao alcanca.
                    if !(56..=200).contains(&x) {
                        outside += 1;
                    }
                }
            }
        }
        eprintln!("{dilution:8.2} {lost:21} {worst:12.3}   |   {outside}");
    }

    eprintln!("\n=== O PERFIL DA PERDA (linha y=90, o MIOLO saturado da faixa seca) ===\n");
    eprintln!(
        "Colunas 176..=216: a vertical acaba em x=200, entao a metade direita e o controle.\n"
    );
    for dilution in [0.00f32, 0.45] {
        let (one, two) = wash_over_dry_two_stages(dilution);
        eprintln!("  Dilution {dilution:.2}");
        eprintln!("    x    antes   depois   perda");
        for x in (176..=216).step_by(4) {
            let a = alpha_at(&one, x, 90);
            let b = alpha_at(&two, x, 90);
            eprintln!("  {x:3} {a:8.3} {b:8.3} {:7.3}", a - b);
        }
        eprintln!();
    }
}

/// A cruz nos DOIS instantes, pela porta da fixture irma — nunca uma 2a montagem da cena.
fn wash_over_dry_two_stages(dilution: f32) -> (Vec<u8>, Vec<u8>) {
    super::measure_watercolor_water_edge::wash_two_stages(dilution, true)
}

/// **ONDE a perda mora, e ela e a QUINA?** — a sonda que amarra as duas anteriores.
///
/// A [`measure_what_the_second_stroke_takes_away`] achou ~1300 texels que PERDEM tinta quando o
/// 2o traco passa, com contagem **plana na dilucao** e pior perda a CAIR (0,452 -> 0,341). Isso
/// ja diz que a remocao nao e o que o Enio reportou (que e dirigido pela Dilution) — mas nao diz
/// **onde** ela esta, e a [`measure_the_concave_corner_retreat`] deixou uma leitura de 0,000 na
/// bissetriz que so uma remocao explica.
///
/// Esta pergunta as duas coisas de uma vez: a caixa que contem os texels perdidos, e o perfil da
/// bissetriz **antes e depois** do 2o traco. Se a quina lia zero porque o 1o traco nunca pintou
/// ali, o `antes` tambem le zero e a remocao e inocente; se o `antes` tem tinta e o `depois` nao,
/// a quina E a remocao.
#[test]
#[ignore = "sonda de diagnostico"]
fn measure_where_the_loss_lives() {
    for dilution in [0.00f32, 0.45] {
        let (one, two) = wash_over_dry_two_stages(dilution);
        let (mut x0, mut y0, mut x1, mut y1) = (usize::MAX, usize::MAX, 0usize, 0usize);
        let mut n = 0usize;
        for y in 0..SIDE {
            for x in 0..SIDE {
                if alpha_at(&one, x, y) - alpha_at(&two, x, y) > 0.01 {
                    n += 1;
                    x0 = x0.min(x);
                    y0 = y0.min(y);
                    x1 = x1.max(x);
                    y1 = y1.max(y);
                }
            }
        }
        eprintln!(
            "\n=== Dilution {dilution:.2}: {n} texels perdidos, caixa x[{x0}..{x1}] y[{y0}..{y1}] ==="
        );

        eprintln!("\n  A BISSETRIZ DA QUINA (200-k, 18+k), antes e depois do 2o traco:");
        eprintln!("    k    antes   depois   flanco H");
        for k in 0..20usize {
            eprintln!(
                "  {k:3} {:8.3} {:8.3} {:10.3}",
                alpha_at(&one, 200 - k, 18 + k),
                alpha_at(&two, 200 - k, 18 + k),
                alpha_at(&two, 40, 18 + k),
            );
        }
    }
}

/// ⚠️ **REFUTADA PELA [`measure_the_depletion_along_the_stroke`] — o oraculo desta sonda NAO
/// vale, e o numero dela nao sustenta a conclusao que eu tirei dele.** Ela compara dois pontos
/// a mesma PROFUNDIDADE na banda de borda, um perto do comeco do traco e outro perto do fim, e
/// assume que a unica diferenca entre eles e a quina. Medido: ao longo de um traco RETO, sem
/// quina em lugar nenhum, o alfa a profundidade constante varia de **0,710 a 0,000** (o miolo,
/// no mesmo traco, fica plano em 0,866-0,917). A banda de borda oscila de posicao ao longo do
/// traco — dente do papel, granulacao — e uma amostra a meio de uma rampa ingreme amplifica
/// esse deslocamento ate a escala inteira. O controle `x = 40` calhou de ser um pico e a quina
/// `x ~ 191` um vale.
///
/// A sonda FICA, com este aviso, porque o que ela mediu e real (a banda oscila) e porque a
/// conclusao errada que eu tirei dela — *"o 2o traco APAGA a tinta seca na quina"* — chegou a
/// entrar numa mensagem de commit. Para medir quina de verdade o controle tem de estar no MESMO
/// comprimento de arco, e o oraculo nao pode ser um ponto no meio da rampa.
///
/// **A QUINA PALIDA PRECISA DE DOIS TRACOS?** — o discriminador que separa as duas rotas
/// que sobraram depois de ler a escrita.
///
/// # Por que esta e a pergunta certa
///
/// A escrita final do render composita a tinta SOBRE a base em transmitancia
/// (`optical = sb*t + pigmento*(1-t)`): com densidade baixa `t -> 1` e o resultado tende a
/// `sb`, a base. **Uma composicao assim nao consegue clarear o que ja estava la** — ela so
/// escurece. Entao a perda medida pela [`measure_where_the_loss_lives`] tem de vir de uma de
/// duas rotas, e as duas SO existem com dois tracos:
///
/// - o `apply_wet_lift`, que caminha o pigmento da base de volta ao papel (o re-wet do doc 23);
/// - a troca de DONO — o 2o traco reivindica o texel, e a aparencia dele passa a ser
///   RE-DERIVADA da cobertura do 2o traco, que perto do proprio aro e fraca.
///
/// A cena aqui e um **L desenhado num traco so**: um braco horizontal em `y=90` (x 24..200) e
/// um vertical em `x=200` (y 90..222). Eles formam um vertice REFLEXO em `(128, 18)`, a mesma
/// concavidade da cruz — com **um dono so, e nenhum pigmento seco por baixo**.
///
/// ⚠️ **O CONTROLE e o flanco reto do MESMO traco** (`x=40`, onde o braco vertical nao alcanca:
/// `|40 - 200| = 160 > 72`). Se a quina de um traco unico ler igual ao flanco, a palidez precisa
/// de dois tracos e a causa esta numa das duas rotas acima. Se ela ler palida sozinha, as duas
/// rotas estao inocentes e o defeito e do proprio deposito na concavidade.
#[test]
#[ignore = "sonda de diagnostico"]
fn measure_whether_the_pale_corner_needs_two_strokes() {
    use super::measure_impasto_cost::cp;
    use crate::tool::PainterTool;
    use ph2d_editor_core::Tool;
    use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};
    use ph2d_painter_brush::{BrushSpec, Falloff};

    let elbow = |dilution: f32| -> Vec<u8> {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; SIDE * SIDE * 4], SIDE as u32, SIDE as u32);
        t.paint.brush = BrushSpec {
            radius_px: 72.0,
            hardness: 1.0,
            falloff: Falloff::Watercolor,
            color: [0.90, 0.15, 0.18],
            space_attenuation: false,
            watercolor: true,
            smooth_edges: true,
            wet_dilution: dilution,
            fill: 0.45,
            depth: 2.0,
            edge_gain: 1.2,
            edge_spread: 6.0,
            opacity: 0.4,
            ..Default::default()
        };
        for slot in &mut t.paint.brush_by_mode {
            *slot = t.paint.brush;
        }
        // UM traco: o braco horizontal ate a dobra, e dali o vertical para baixo.
        t.on_canvas_pointer(cp([24.0, 90.0], PointerPhase::Down));
        for i in 1..=11u8 {
            t.on_canvas_pointer(cp([24.0 + f32::from(i) * 16.0, 90.0], PointerPhase::Move));
            t.on_tick(16.0);
        }
        for i in 1..=11u8 {
            t.on_canvas_pointer(cp([200.0, 90.0 + f32::from(i) * 12.0], PointerPhase::Move));
            t.on_tick(16.0);
        }
        t.on_canvas_pointer(cp([200.0, 222.0], PointerPhase::Up));
        for _ in 0..8 {
            t.on_tick(16.0);
        }
        t.canvas_rgba.as_ref().clone()
    };

    eprintln!("\n=== A QUINA DE UM TRACO SO (cotovelo, vertice reflexo em 128,18) ===\n");
    eprintln!("Bissetriz (128+k, 18+k) contra o flanco reto do MESMO traco em x=40.\n");
    for dilution in [0.00f32, 0.45] {
        let px = elbow(dilution);
        eprintln!("  Dilution {dilution:.2}");
        eprintln!("    k    QUINA   flanco   quina-flanco");
        for k in 0..22usize {
            let q = alpha_at(&px, 128 + k, 18 + k);
            let f = alpha_at(&px, 40, 18 + k);
            eprintln!("  {k:3} {q:8.3} {f:8.3} {:14.3}", q - f);
        }
        eprintln!();
    }
}

/// **O CONTROLE DAS TRES SONDAS ANTERIORES ESTAVA CONFUNDIDO** — esta mede o confundidor.
///
/// A [`measure_the_concave_corner_retreat`] e a [`measure_whether_the_pale_corner_needs_two_strokes`]
/// comparam a bissetriz da quina (que cai perto do FIM do braco horizontal, `x ~ 191`) contra um
/// flanco reto em `x = 40` (o COMECO do mesmo traco). As duas afirmam medir geometria — mas o
/// render tem um termo que varia ao longo do traco e que nenhuma delas neutraliza: a reserva de
/// pigmento do pincel (`depl_buf`, MIX-1), que ESVAZIA conforme a mao anda.
///
/// Esta sonda percorre a MESMA profundidade (`y = 27`, nove texels dentro da borda de cima) ao
/// longo de `x`, com **um traco horizontal so e nenhuma quina em lugar nenhum**. Se o alfa cair
/// com `x`, as duas sondas acima estavam a medir a deplecao e a chamar-lhe quina.
#[test]
#[ignore = "sonda de diagnostico"]
fn measure_the_depletion_along_the_stroke() {
    let (one, _) = wash_over_dry_two_stages(0.00);
    eprintln!(
        "\n=== O ALFA AO LONGO DO TRACO, A PROFUNDIDADE CONSTANTE (1 traco, sem quina) ===\n"
    );
    eprintln!("O traco corre em y=90 de x=24 a x=232. Amostras em y=27 (nove texels da borda)");
    eprintln!(
        "e em y=90 (o miolo saturado), para separar 'a borda empalidece' de 'tudo empalidece'.\n"
    );
    eprintln!("    x    y=27     y=90");
    for x in (32..=224).step_by(16) {
        eprintln!(
            "  {x:3} {:8.3} {:8.3}",
            alpha_at(&one, x, 27),
            alpha_at(&one, x, 90)
        );
    }
    eprintln!(
        "\nSe a coluna y=27 cair com x, o 'deficit da quina' das sondas anteriores e DEPLECAO:\n\
         elas punham o controle no comeco do traco e a quina perto do fim.\n"
    );
}
