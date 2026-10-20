//! **O vocabulário de canais do VETOR** ([`crate::canal::familia::VETOR`]) — os números de `campo`, o
//! sentido de `i`/`j` e o que cada um é. Quem lê e escreve o documento é `ph2d-app-vec` (`canais.rs`);
//! aqui só mora o vocabulário, para que a timeline e o dope-sheet saibam o que um endereço É sem
//! conhecerem o vetor ([plano 35 §A.1](../../../docs/UI_New_and_Simple/vetor_nova_ui/35_plano_F24_o_auto_key_grava_tudo.md)).
//!
//! ⛔ **Os números são gravados no documento da timeline: só APPEND, nunca renumerar.** O gate
//! `o_vocabulario_do_vetor_nao_renumera` fixa-os.
//!
//! - **Forma** (`i` = contorno: 0 = o principal, `k+1` = o subcaminho `k`; `j` = vértice): um grupo só
//!   para TODOS os contornos — mexer um ponto grava a forma inteira (decisão do dono, 20/10).
//! - **Tinta** (`i` = 0 a do preenchimento/traço de base, `k+1` a da entrada `k` da pilha; `j` =
//!   paragem ou ponto do gradiente): a tinta inteira é UM grupo por `i`.
//! - **Traço** (`i` como na tinta; `j` = par do tracejado).
//! - **Efeito** (`i` = entrada; `j` = parâmetro, na ordem de `PathEffect::params`).
//! - **Forma paramétrica** e **texto** vivem no `VecShape` da entidade; `j` = valor / eixo.

use crate::canal::CampoInfo;

/// Forma: vértice (`i` contorno, `j` vértice).
pub const ANCORA_X: u16 = 1;
pub const ANCORA_Y: u16 = 2;
pub const ENTRADA_X: u16 = 3;
pub const ENTRADA_Y: u16 = 4;
pub const SAIDA_X: u16 = 5;
pub const SAIDA_Y: u16 = 6;
/// O arredondamento do canto (o sinal é o estilo, como no documento).
pub const CANTO: u16 = 7;
/// `VertexKind` (Corner 0 · Smooth 1 · Symmetric 2).
pub const TIPO_VERTICE: u16 = 8;
/// Contorno fechado (`j` = 0).
pub const FECHADO: u16 = 9;
/// `FillRule`.
pub const REGRA: u16 = 10;

/// Base da tinta do PREENCHIMENTO e da do TRAÇO; o campo é `base + OFF_*`.
pub const TINTA_PREENCH: u16 = 20;
pub const TINTA_TRACO: u16 = 60;
/// Há tinta (`fill: Option` / o traço existe) — 0 ou 1.
pub const OFF_EXISTE: u16 = 0;
/// `Paint`: Solid 0 · Linear 1 · Radial 2 · MultiPoint 3 · Pattern 4.
pub const OFF_TIPO: u16 = 1;
/// Cor chapada r, g, b, a (`OFF_SOLIDA + 0..4`), `0..=255`.
pub const OFF_SOLIDA: u16 = 2;
/// Paragem `j`: offset, depois r, g, b, a (`OFF_PARAGEM + 0..5`).
pub const OFF_PARAGEM: u16 = 6;
/// Nº de paragens (a topologia do gradiente).
pub const OFF_N_PARAGENS: u16 = 11;
/// Linear: início x, y, fim x, y (`+0..4`).
pub const OFF_LINEAR: u16 = 12;
/// Radial: centro x, y, raio (`+0..3`).
pub const OFF_RADIAL: u16 = 16;
/// Ponto `j` do multiponto: x, y, r, g, b, a, influência, jitter (`+0..8`).
pub const OFF_PONTO: u16 = 19;
/// Nº de pontos do multiponto.
pub const OFF_N_PONTOS: u16 = 27;
/// Padrão: tamanho x, y, folga x, y, origem x, y, ângulo, alfa (`+0..8`).
pub const OFF_PADRAO: u16 = 28;
/// Padrão: `TileKind` e `PatternMode` (`+0`, `+1`).
pub const OFF_PADRAO_MODOS: u16 = 36;
/// O tamanho de um bloco de tinta (o próximo bloco começa depois disto).
pub const TINTA_LARGURA: u16 = 38;

/// Traço (`i` = 0 o de base, `k+1` o da entrada `k`).
pub const TRACO_LARGURA: u16 = 100;
pub const TRACO_PONTA: u16 = 101;
pub const TRACO_JUNCAO: u16 = 102;
pub const TRACO_ALINHAMENTO: u16 = 103;
/// Há tracejado — 0 ou 1.
pub const TRACO_TRACEJADO: u16 = 104;
/// Par `j` do tracejado: comprimento do traço e do vão.
pub const TRACO_TRACO: u16 = 105;
pub const TRACO_VAO: u16 = 106;
pub const TRACO_MARCA_INICIO: u16 = 107;
pub const TRACO_MARCA_FIM: u16 = 108;
pub const TRACO_ESCALA_INICIO: u16 = 109;
pub const TRACO_ESCALA_FIM: u16 = 110;
pub const TRACO_MARCA_REDONDA: u16 = 111;

/// A mistura da forma (`BlendMode`). A opacidade é o `PropKind::Opacity` de sempre.
pub const MISTURA: u16 = 140;

/// Entrada `i - 1` da pilha de pinturas.
pub const PILHA_LIGADA: u16 = 150;
pub const PILHA_OPACIDADE: u16 = 151;
pub const PILHA_MISTURA: u16 = 152;
pub const PILHA_DESLOCA_X: u16 = 153;
pub const PILHA_DESLOCA_Y: u16 = 154;
pub const PILHA_DILATA: u16 = 155;
pub const PILHA_DILATA_JUNCAO: u16 = 156;

/// Efeito `i`: ligado, e o parâmetro `j`.
pub const EFEITO_LIGADO: u16 = 170;
pub const EFEITO_PARAM: u16 = 171;

/// Forma paramétrica (`VecShape::Param`): largura, altura, valor `j`.
pub const PARAM_LARGURA: u16 = 180;
pub const PARAM_ALTURA: u16 = 181;
pub const PARAM_VALOR: u16 = 182;

/// Texto (`VecShape::Text`).
pub const TEXTO_TAMANHO: u16 = 190;
pub const TEXTO_PESO: u16 = 191;
pub const TEXTO_ENTRELINHA: u16 = 192;
pub const TEXTO_TRACKING: u16 = 193;
pub const TEXTO_ALINHAMENTO: u16 = 194;
pub const TEXTO_QUEBRA: u16 = 195;
pub const TEXTO_EIXO: u16 = 196;

const fn c(rotulo: &'static str, grupo: u16, por_i: bool) -> CampoInfo {
    CampoInfo {
        rotulo,
        discreto: false,
        grupo,
        por_i,
    }
}

const fn d(rotulo: &'static str, grupo: u16, por_i: bool) -> CampoInfo {
    CampoInfo {
        rotulo,
        discreto: true,
        grupo,
        por_i,
    }
}

/// O que o campo `campo` do vetor é. A tinta resolve-se pelo deslocamento dentro do bloco.
#[must_use]
pub fn info(campo: u16) -> Option<CampoInfo> {
    const FORMA: &str = "canal.vetor.forma";
    Some(match campo {
        ANCORA_X..=CANTO => c(FORMA, ANCORA_X, false),
        TIPO_VERTICE | FECHADO => d(FORMA, ANCORA_X, false),
        REGRA => d("canal.vetor.regra", REGRA, false),
        TINTA_PREENCH..TINTA_TRACO => tinta(
            campo - TINTA_PREENCH,
            "canal.vetor.preenchimento",
            TINTA_PREENCH,
        )?,
        TINTA_TRACO..TRACO_LARGURA => {
            tinta(campo - TINTA_TRACO, "canal.vetor.traco_cor", TINTA_TRACO)?
        }
        TRACO_LARGURA => c("canal.vetor.traco_largura", campo, true),
        TRACO_PONTA => d("canal.vetor.traco_ponta", campo, true),
        TRACO_JUNCAO => d("canal.vetor.traco_juncao", campo, true),
        TRACO_ALINHAMENTO => d("canal.vetor.traco_alinhamento", campo, true),
        TRACO_TRACEJADO => d("canal.vetor.tracejado", TRACO_TRACEJADO, true),
        TRACO_TRACO | TRACO_VAO => c("canal.vetor.tracejado", TRACO_TRACEJADO, true),
        TRACO_MARCA_INICIO => d("canal.vetor.marca_inicio", campo, true),
        TRACO_MARCA_FIM => d("canal.vetor.marca_fim", campo, true),
        TRACO_ESCALA_INICIO => c("canal.vetor.escala_inicio", campo, true),
        TRACO_ESCALA_FIM => c("canal.vetor.escala_fim", campo, true),
        TRACO_MARCA_REDONDA => c("canal.vetor.marca_redonda", campo, true),
        MISTURA => d("canal.vetor.mistura", campo, false),
        PILHA_LIGADA => d("canal.vetor.pilha_ligada", campo, true),
        PILHA_OPACIDADE => c("canal.vetor.pilha_opacidade", campo, true),
        PILHA_MISTURA => d("canal.vetor.pilha_mistura", campo, true),
        PILHA_DESLOCA_X | PILHA_DESLOCA_Y => {
            c("canal.vetor.pilha_deslocamento", PILHA_DESLOCA_X, true)
        }
        PILHA_DILATA => c("canal.vetor.pilha_dilatacao", campo, true),
        PILHA_DILATA_JUNCAO => d("canal.vetor.pilha_dilatacao_juncao", campo, true),
        EFEITO_LIGADO => d("canal.vetor.efeito_ligado", campo, true),
        EFEITO_PARAM => c("canal.vetor.efeito", campo, true),
        PARAM_LARGURA | PARAM_ALTURA => c("canal.vetor.tamanho", PARAM_LARGURA, false),
        PARAM_VALOR => c("canal.vetor.forma_param", PARAM_VALOR, false),
        TEXTO_TAMANHO => c("canal.vetor.texto_tamanho", campo, false),
        TEXTO_PESO => c("canal.vetor.texto_peso", campo, false),
        TEXTO_ENTRELINHA => c("canal.vetor.texto_entrelinha", campo, false),
        TEXTO_TRACKING => c("canal.vetor.texto_tracking", campo, false),
        TEXTO_ALINHAMENTO => d("canal.vetor.texto_alinhamento", campo, false),
        TEXTO_QUEBRA => c("canal.vetor.texto_quebra", campo, false),
        TEXTO_EIXO => c("canal.vetor.texto_eixo", campo, false),
        _ => return None,
    })
}

/// O nome do campo `campo` dentro do seu grupo (a linha do membro no dope-sheet). Um campo que é o
/// grupo inteiro devolve o rótulo do grupo.
#[must_use]
pub fn membro(campo: u16) -> Option<&'static str> {
    Some(match campo {
        ANCORA_X => "canal.m.ponto_x",
        ANCORA_Y => "canal.m.ponto_y",
        ENTRADA_X => "canal.m.entrada_x",
        ENTRADA_Y => "canal.m.entrada_y",
        SAIDA_X => "canal.m.saida_x",
        SAIDA_Y => "canal.m.saida_y",
        CANTO => "canal.m.canto",
        TIPO_VERTICE => "canal.m.tipo_ponto",
        FECHADO => "canal.m.fechado",
        TINTA_PREENCH..TINTA_TRACO => membro_tinta(campo - TINTA_PREENCH)?,
        TINTA_TRACO..TRACO_LARGURA => membro_tinta(campo - TINTA_TRACO)?,
        TRACO_TRACEJADO => "canal.m.ligado",
        TRACO_TRACO => "canal.m.traco",
        TRACO_VAO => "canal.m.vao",
        PILHA_DESLOCA_X | PARAM_LARGURA => "canal.m.x",
        PILHA_DESLOCA_Y | PARAM_ALTURA => "canal.m.y",
        PARAM_VALOR => "canal.m.valor",
        EFEITO_PARAM => "canal.m.parametro",
        _ => info(campo)?.rotulo,
    })
}

/// O nome de um campo dentro de um bloco de tinta.
fn membro_tinta(off: u16) -> Option<&'static str> {
    const RGBA: [&str; 4] = ["canal.m.r", "canal.m.g", "canal.m.b", "canal.m.a"];
    const PARAGEM: [&str; 5] = [
        "canal.m.posicao",
        "canal.m.r",
        "canal.m.g",
        "canal.m.b",
        "canal.m.a",
    ];
    const LINEAR: [&str; 4] = [
        "canal.m.inicio_x",
        "canal.m.inicio_y",
        "canal.m.fim_x",
        "canal.m.fim_y",
    ];
    const RADIAL: [&str; 3] = ["canal.m.centro_x", "canal.m.centro_y", "canal.m.raio"];
    const PONTO: [&str; 8] = [
        "canal.m.ponto_x",
        "canal.m.ponto_y",
        "canal.m.r",
        "canal.m.g",
        "canal.m.b",
        "canal.m.a",
        "canal.m.influencia",
        "canal.m.jitter",
    ];
    const PADRAO: [&str; 8] = [
        "canal.m.tamanho_x",
        "canal.m.tamanho_y",
        "canal.m.folga_x",
        "canal.m.folga_y",
        "canal.m.origem_x",
        "canal.m.origem_y",
        "canal.m.angulo",
        "canal.m.a",
    ];
    let em = |base: u16, tabela: &[&'static str]| tabela.get(usize::from(off - base)).copied();
    match off {
        OFF_EXISTE => Some("canal.m.ligado"),
        OFF_TIPO => Some("canal.m.tipo"),
        OFF_N_PARAGENS => Some("canal.m.n_paragens"),
        OFF_N_PONTOS => Some("canal.m.n_pontos"),
        OFF_PADRAO_MODOS => Some("canal.m.ladrilho"),
        o if o == OFF_PADRAO_MODOS + 1 => Some("canal.m.modo"),
        OFF_SOLIDA..OFF_PARAGEM => em(OFF_SOLIDA, &RGBA),
        OFF_PARAGEM..OFF_N_PARAGENS => em(OFF_PARAGEM, &PARAGEM),
        OFF_LINEAR..OFF_RADIAL => em(OFF_LINEAR, &LINEAR),
        OFF_RADIAL..OFF_PONTO => em(OFF_RADIAL, &RADIAL),
        OFF_PONTO..OFF_N_PONTOS => em(OFF_PONTO, &PONTO),
        OFF_PADRAO..OFF_PADRAO_MODOS => em(OFF_PADRAO, &PADRAO),
        _ => None,
    }
}

/// Um campo dentro de um bloco de tinta: a tinta inteira é um grupo (por `i`).
fn tinta(off: u16, rotulo: &'static str, grupo: u16) -> Option<CampoInfo> {
    match off {
        OFF_EXISTE | OFF_TIPO | OFF_N_PARAGENS | OFF_N_PONTOS => Some(d(rotulo, grupo, true)),
        o if (OFF_PADRAO_MODOS..OFF_PADRAO_MODOS + 2).contains(&o) => Some(d(rotulo, grupo, true)),
        o if o < TINTA_LARGURA => Some(c(rotulo, grupo, true)),
        _ => None,
    }
}
