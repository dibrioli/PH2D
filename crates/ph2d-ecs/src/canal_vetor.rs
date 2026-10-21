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
/// Padrão: o desfasamento do Brick/Column (`PatternFill::offset_denom`, `1..=255`, discreto).
pub const OFF_PADRAO_DESFASE: u16 = 38;
/// O tamanho de um bloco de tinta (o próximo bloco começa depois disto).
pub const TINTA_LARGURA: u16 = 39;

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
/// O pincel do traço (`StrokePaint::Brush`, `i` como no traço): membros da TINTA do traço.
pub const PINCEL_ESPACO: u16 = 112;
pub const PINCEL_DESLOCA: u16 = 113;
pub const PINCEL_ROTACAO: u16 = 114;
pub const PINCEL_ESCALA: u16 = 115;
pub const PINCEL_ESPELHO: u16 = 116;

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
/// O MODO da quebra: 0 = automática (`wrap_width: None`), 1 = largura fixa. Membro do grupo da
/// largura ([`TEXTO_QUEBRA`]): as duas gravam-se juntas.
pub const TEXTO_QUEBRA_FIXA: u16 = 197;
/// A família da fonte e o conteúdo: o índice em `canal::TextosDosCanais` (`0` = nenhum).
pub const TEXTO_FAMILIA: u16 = 198;
pub const TEXTO_CONTEUDO: u16 = 199;

/// O efeito Contour (`VecContour`, componente da entidade): passos, distância por passo,
/// aceleração, quina, lado, e a cor do último anel r, g, b, a (`CONTORNO_COR + 0..4`, `0..=255`).
/// Criar/remover o efeito é estrutura (plano 35 §8), não canal.
pub const CONTORNO_PASSOS: u16 = 200;
pub const CONTORNO_DISTANCIA: u16 = 201;
pub const CONTORNO_ACEL: u16 = 202;
pub const CONTORNO_JUNCAO: u16 = 203;
pub const CONTORNO_LADO: u16 = 204;
pub const CONTORNO_COR: u16 = 205;
const CONTORNO_COR_A: u16 = CONTORNO_COR + 3;

/// O Offset vivo (`VecOffset`, componente da entidade): a distância de MUNDO, a quina e o lado
/// (os códigos do painel).
pub const OFFSET_DISTANCIA: u16 = 210;
pub const OFFSET_JUNCAO: u16 = 211;
pub const OFFSET_LADO: u16 = 212;

/// Os FILTROS da forma (`VecFilter::ops`, `i` = filtro): o olho é um grupo, o resto do filtro
/// outro (como o efeito). As cores são `[f32; 4]` como o `FxOp` as guarda; a rampa: paragem `j`
/// = posição, r, g, b, a (`FILTRO_PARAGEM + 0..5`).
pub const FILTRO_TIPO: u16 = 220;
pub const FILTRO_LIGADO: u16 = 221;
pub const FILTRO_RAIO: u16 = 222;
pub const FILTRO_DESLOCA_X: u16 = 223;
pub const FILTRO_DESLOCA_Y: u16 = 224;
pub const FILTRO_COR: u16 = 225;
pub const FILTRO_COR_B: u16 = 229;
pub const FILTRO_OPACIDADE: u16 = 233;
pub const FILTRO_MODO: u16 = 234;
pub const FILTRO_MISTURA: u16 = 235;
pub const FILTRO_ESCALA: u16 = 236;
pub const FILTRO_DETALHE: u16 = 237;
pub const FILTRO_SEMENTE: u16 = 238;
pub const FILTRO_CRESCE: u16 = 239;
pub const FILTRO_MATIZ: u16 = 240;
pub const FILTRO_SATURACAO: u16 = 241;
pub const FILTRO_BRILHO: u16 = 242;
pub const FILTRO_N_PARAGENS: u16 = 243;
pub const FILTRO_PARAGEM: u16 = 244;
const FILTRO_ULTIMO: u16 = FILTRO_PARAGEM + 4;

/// O perfil de largura (`VecStrokeProfile`, Power Stroke): existe (o neutro é a ausência), nº de
/// paragens e a paragem `j` (posição, multiplicador). Um grupo só.
pub const PERFIL_EXISTE: u16 = 250;
pub const PERFIL_N: u16 = 251;
pub const PERFIL_POS: u16 = 252;
pub const PERFIL_MULT: u16 = 253;

/// O padrão ao longo do caminho (`VecPatternPath` + `VecPatternRotation`).
pub const PADRAO_CAMINHO_INICIO: u16 = 260;
pub const PADRAO_CAMINHO_FIM: u16 = 261;
pub const PADRAO_CAMINHO_ESPACO: u16 = 262;
pub const PADRAO_CAMINHO_DESLOCA: u16 = 263;
pub const PADRAO_CAMINHO_ESPELHO: u16 = 264;
pub const PADRAO_CAMINHO_ROTACAO: u16 = 265;

/// O texto no caminho (`VecTextPath`).
pub const TEXTO_CAMINHO_INICIO: u16 = 270;
pub const TEXTO_CAMINHO_ESPELHO: u16 = 271;

/// O Blend vivo (`VecBlend::steps`). O Morph vivo (`VecMorph::t`) é o `PropKind::Morph` de sempre.
pub const BLEND_PASSOS: u16 = 280;

/// O Envelope vivo (`VecEnvelope`) — a geometria inteira é UM grupo (a lei da forma): canto `j`
/// x/y, alça `j` (= aresta × 2 + lado) x/y, o tipo, o warp (0 = nenhum, `k+1` = estilo `k`), nº de
/// pinos, pino `j` (origem x, y, destino x, y: `ENVELOPE_PINO + 0..4`), a curvatura.
pub const ENVELOPE_CANTO_X: u16 = 290;
pub const ENVELOPE_CANTO_Y: u16 = 291;
pub const ENVELOPE_ALCA_X: u16 = 292;
pub const ENVELOPE_ALCA_Y: u16 = 293;
pub const ENVELOPE_TIPO: u16 = 294;
pub const ENVELOPE_WARP: u16 = 295;
pub const ENVELOPE_N_PINOS: u16 = 296;
pub const ENVELOPE_PINO: u16 = 297;
pub const ENVELOPE_CURVA: u16 = 301;

/// Interface: as âncoras (`VecAnchors`): existe, min x/y, max x/y, base `+0..4` — discretas e UM
/// grupo (a âncora e a base são a mesma escolha do painel).
pub const ANCORAS_EXISTE: u16 = 310;
pub const ANCORA_MIN_X: u16 = 311;
pub const ANCORA_MIN_Y: u16 = 312;
pub const ANCORA_MAX_X: u16 = 313;
pub const ANCORA_MAX_Y: u16 = 314;
pub const ANCORA_BASE: u16 = 315;
const ANCORA_ULTIMA: u16 = ANCORA_BASE + 3;

/// Interface: a moldura — recortar o conteúdo (`VecClipContent`).
pub const MOLDURA_RECORTA: u16 = 320;

/// Interface: o auto layout. Do contentor (`VecLayout`): direcção, vão x/y, margem `+0..4`,
/// alinhamento, distribuição, colunas. Do filho (`VecLayoutItem`, `VecLayoutSize`,
/// `VecLayoutAbsolute`): cresce, encolhe, base (com o «tem base»), tamanho/min/max por eixo `j`
/// (com o «tem min/max»), absoluto.
pub const LAYOUT_DIR: u16 = 330;
pub const LAYOUT_VAO_X: u16 = 331;
pub const LAYOUT_VAO_Y: u16 = 332;
pub const LAYOUT_MARGEM: u16 = 333;
pub const LAYOUT_ALINHA: u16 = 337;
pub const LAYOUT_DISTRIBUI: u16 = 338;
pub const LAYOUT_COLUNAS: u16 = 339;
pub const LAYOUT_CRESCE: u16 = 340;
pub const LAYOUT_ENCOLHE: u16 = 341;
pub const LAYOUT_TEM_BASE: u16 = 342;
pub const LAYOUT_BASE: u16 = 343;
pub const LAYOUT_TAMANHO: u16 = 344;
pub const LAYOUT_TEM_MIN: u16 = 345;
pub const LAYOUT_MIN: u16 = 346;
pub const LAYOUT_TEM_MAX: u16 = 347;
pub const LAYOUT_MAX: u16 = 348;
pub const LAYOUT_ABSOLUTO: u16 = 349;

/// Interface: a pele do widget (`VecWidget::kind`).
pub const WIDGET_PELE: u16 = 350;
/// Interface: o ícone da pele (`VecWidgetIcon::slug`, um texto: o índice em
/// `canal::TextosDosCanais`, `0` = sem ícone).
pub const WIDGET_ICONE: u16 = 351;

const fn c(rotulo: &'static str, grupo: u16, por_i: bool) -> CampoInfo {
    CampoInfo {
        rotulo,
        discreto: false,
        grupo,
        por_i,
        texto: false,
    }
}

const fn d(rotulo: &'static str, grupo: u16, por_i: bool) -> CampoInfo {
    CampoInfo {
        rotulo,
        discreto: true,
        grupo,
        por_i,
        texto: false,
    }
}

/// Um texto: discreto, e o valor é um índice em `canal::TextosDosCanais`.
const fn t(rotulo: &'static str, grupo: u16) -> CampoInfo {
    CampoInfo {
        rotulo,
        discreto: true,
        grupo,
        por_i: false,
        texto: true,
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
        PINCEL_ESPACO..PINCEL_ESPELHO => c("canal.vetor.traco_cor", TINTA_TRACO, true),
        PINCEL_ESPELHO => d("canal.vetor.traco_cor", TINTA_TRACO, true),
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
        TEXTO_QUEBRA_FIXA => d("canal.vetor.texto_quebra", TEXTO_QUEBRA, false),
        TEXTO_FAMILIA => t("canal.vetor.texto_familia", campo),
        TEXTO_CONTEUDO => t("canal.vetor.texto_conteudo", campo),
        FILTRO_LIGADO => d("canal.vetor.filtro_ligado", campo, true),
        FILTRO_TIPO | FILTRO_MODO | FILTRO_MISTURA | FILTRO_DETALHE | FILTRO_SEMENTE
        | FILTRO_N_PARAGENS => d("canal.vetor.filtro", FILTRO_TIPO, true),
        FILTRO_RAIO..=FILTRO_ULTIMO => c("canal.vetor.filtro", FILTRO_TIPO, true),
        PERFIL_EXISTE | PERFIL_N => d("canal.vetor.perfil", PERFIL_EXISTE, false),
        PERFIL_POS | PERFIL_MULT => c("canal.vetor.perfil", PERFIL_EXISTE, false),
        PADRAO_CAMINHO_INICIO => c("canal.vetor.padrao_caminho_inicio", campo, false),
        PADRAO_CAMINHO_FIM => c("canal.vetor.padrao_caminho_fim", campo, false),
        PADRAO_CAMINHO_ESPACO => c("canal.vetor.padrao_caminho_espaco", campo, false),
        PADRAO_CAMINHO_DESLOCA => c("canal.vetor.padrao_caminho_desloca", campo, false),
        PADRAO_CAMINHO_ESPELHO => d("canal.vetor.padrao_caminho_espelho", campo, false),
        PADRAO_CAMINHO_ROTACAO => c("canal.vetor.padrao_caminho_rotacao", campo, false),
        TEXTO_CAMINHO_INICIO => c("canal.vetor.texto_caminho_inicio", campo, false),
        TEXTO_CAMINHO_ESPELHO => d("canal.vetor.texto_caminho_espelho", campo, false),
        BLEND_PASSOS => d("canal.vetor.blend_passos", campo, false),
        ENVELOPE_TIPO | ENVELOPE_WARP | ENVELOPE_N_PINOS => {
            d("canal.vetor.envelope", ENVELOPE_CANTO_X, false)
        }
        ENVELOPE_CANTO_X..=ENVELOPE_CURVA => c("canal.vetor.envelope", ENVELOPE_CANTO_X, false),
        ANCORAS_EXISTE..=ANCORA_ULTIMA => d("canal.vetor.ancoras", ANCORAS_EXISTE, false),
        MOLDURA_RECORTA => d("canal.vetor.moldura_recorta", campo, false),
        LAYOUT_DIR => d("canal.vetor.layout_dir", campo, false),
        LAYOUT_VAO_X | LAYOUT_VAO_Y => c("canal.vetor.layout_vao", LAYOUT_VAO_X, false),
        LAYOUT_MARGEM..LAYOUT_ALINHA => c("canal.vetor.layout_margem", LAYOUT_MARGEM, false),
        LAYOUT_ALINHA => d("canal.vetor.layout_alinha", campo, false),
        LAYOUT_DISTRIBUI => d("canal.vetor.layout_distribui", campo, false),
        LAYOUT_COLUNAS => d("canal.vetor.layout_colunas", campo, false),
        LAYOUT_CRESCE => c("canal.vetor.layout_cresce", campo, false),
        LAYOUT_ENCOLHE => c("canal.vetor.layout_encolhe", campo, false),
        LAYOUT_TEM_BASE => d("canal.vetor.layout_base", LAYOUT_BASE, false),
        LAYOUT_BASE => c("canal.vetor.layout_base", campo, false),
        LAYOUT_TAMANHO => d("canal.vetor.layout_tamanho", campo, false),
        LAYOUT_TEM_MIN => d("canal.vetor.layout_min", LAYOUT_MIN, false),
        LAYOUT_MIN => c("canal.vetor.layout_min", campo, false),
        LAYOUT_TEM_MAX => d("canal.vetor.layout_max", LAYOUT_MAX, false),
        LAYOUT_MAX => c("canal.vetor.layout_max", campo, false),
        LAYOUT_ABSOLUTO => d("canal.vetor.layout_absoluto", campo, false),
        WIDGET_PELE => d("canal.vetor.widget_pele", campo, false),
        WIDGET_ICONE => t("canal.vetor.widget_icone", campo),
        CONTORNO_PASSOS => d("canal.vetor.contorno_passos", campo, false),
        CONTORNO_DISTANCIA => c("canal.vetor.contorno_distancia", campo, false),
        CONTORNO_ACEL => c("canal.vetor.contorno_acel", campo, false),
        CONTORNO_JUNCAO => d("canal.vetor.contorno_juncao", campo, false),
        CONTORNO_LADO => d("canal.vetor.contorno_lado", campo, false),
        CONTORNO_COR..=CONTORNO_COR_A => c("canal.vetor.contorno_cor", CONTORNO_COR, false),
        OFFSET_DISTANCIA => c("canal.vetor.offset_distancia", campo, false),
        OFFSET_JUNCAO => d("canal.vetor.offset_juncao", campo, false),
        OFFSET_LADO => d("canal.vetor.offset_lado", campo, false),
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
        CONTORNO_COR..=CONTORNO_COR_A => membro_tinta(campo - CONTORNO_COR + OFF_SOLIDA)?,
        PINCEL_ESPACO => "canal.m.espaco",
        PINCEL_DESLOCA => "canal.m.desloca",
        PINCEL_ROTACAO => "canal.m.angulo",
        PINCEL_ESCALA => "canal.m.escala",
        PINCEL_ESPELHO => "canal.m.espelho",
        TEXTO_QUEBRA_FIXA => "canal.m.fixa",
        FILTRO_TIPO.. => membro_vivo(campo)?,
        _ => info(campo)?.rotulo,
    })
}

/// O nome de um membro dos grupos dos componentes da entidade (filtro, perfil, envelope, âncoras,
/// auto layout).
fn membro_vivo(campo: u16) -> Option<&'static str> {
    const RGBA: [&str; 4] = ["canal.m.r", "canal.m.g", "canal.m.b", "canal.m.a"];
    const RGBA_B: [&str; 4] = ["canal.m.r2", "canal.m.g2", "canal.m.b2", "canal.m.a2"];
    const PARAGEM: [&str; 5] = [
        "canal.m.posicao",
        "canal.m.r",
        "canal.m.g",
        "canal.m.b",
        "canal.m.a",
    ];
    const PINO: [&str; 4] = [
        "canal.m.origem_x",
        "canal.m.origem_y",
        "canal.m.destino_x",
        "canal.m.destino_y",
    ];
    const LADOS: [&str; 4] = [
        "canal.m.topo",
        "canal.m.direita",
        "canal.m.baixo",
        "canal.m.esquerda",
    ];
    const CAIXA: [&str; 4] = [
        "canal.m.base_x0",
        "canal.m.base_y0",
        "canal.m.base_x1",
        "canal.m.base_y1",
    ];
    let em = |base: u16, tabela: &[&'static str]| tabela.get(usize::from(campo - base)).copied();
    Some(match campo {
        FILTRO_TIPO => "canal.m.tipo",
        FILTRO_LIGADO | PERFIL_EXISTE | ANCORAS_EXISTE => "canal.m.ligado",
        LAYOUT_TEM_BASE | LAYOUT_TEM_MIN | LAYOUT_TEM_MAX => "canal.m.ligado",
        FILTRO_RAIO => "canal.m.raio",
        FILTRO_DESLOCA_X | LAYOUT_VAO_X => "canal.m.x",
        FILTRO_DESLOCA_Y | LAYOUT_VAO_Y => "canal.m.y",
        FILTRO_COR..FILTRO_COR_B => em(FILTRO_COR, &RGBA)?,
        FILTRO_COR_B..FILTRO_OPACIDADE => em(FILTRO_COR_B, &RGBA_B)?,
        FILTRO_OPACIDADE => "canal.m.opacidade",
        FILTRO_MODO => "canal.m.modo",
        FILTRO_MISTURA => "canal.m.mistura",
        FILTRO_ESCALA => "canal.m.escala",
        FILTRO_DETALHE => "canal.m.detalhe",
        FILTRO_SEMENTE => "canal.m.semente",
        FILTRO_CRESCE => "canal.m.cresce",
        FILTRO_MATIZ => "canal.m.matiz",
        FILTRO_SATURACAO => "canal.m.saturacao",
        FILTRO_BRILHO => "canal.m.brilho",
        FILTRO_N_PARAGENS | PERFIL_N => "canal.m.n_paragens",
        FILTRO_PARAGEM..=FILTRO_ULTIMO => em(FILTRO_PARAGEM, &PARAGEM)?,
        PERFIL_POS => "canal.m.posicao",
        PERFIL_MULT => "canal.m.largura",
        ENVELOPE_CANTO_X => "canal.m.canto_x",
        ENVELOPE_CANTO_Y => "canal.m.canto_y",
        ENVELOPE_ALCA_X => "canal.m.alca_x",
        ENVELOPE_ALCA_Y => "canal.m.alca_y",
        ENVELOPE_TIPO => "canal.m.tipo",
        ENVELOPE_WARP => "canal.m.warp",
        ENVELOPE_N_PINOS => "canal.m.n_pinos",
        ENVELOPE_PINO..ENVELOPE_CURVA => em(ENVELOPE_PINO, &PINO)?,
        ENVELOPE_CURVA => "canal.m.curva",
        ANCORA_MIN_X => "canal.m.min_x",
        ANCORA_MIN_Y => "canal.m.min_y",
        ANCORA_MAX_X => "canal.m.max_x",
        ANCORA_MAX_Y => "canal.m.max_y",
        ANCORA_BASE..=ANCORA_ULTIMA => em(ANCORA_BASE, &CAIXA)?,
        LAYOUT_MARGEM..LAYOUT_ALINHA => em(LAYOUT_MARGEM, &LADOS)?,
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
        OFF_PADRAO_DESFASE => Some("canal.m.desfase"),
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
        o if (OFF_PADRAO_MODOS..=OFF_PADRAO_DESFASE).contains(&o) => Some(d(rotulo, grupo, true)),
        o if o < TINTA_LARGURA => Some(c(rotulo, grupo, true)),
        _ => None,
    }
}
