//! **O vocabulário de canais da pilha de CONSTRAINTS** ([`crate::canal::familia::CONSTRAINT`]) — o
//! que se anima num item da pilha (plano `docs/BlenderConstraints/01_plano.md` §2.2): a força, o
//! liga/desliga e os ajustes de cada tipo, nas unidades que o cartão mostra (graus, metros, índice). Quem lê e escreve é `ph2d-app-constraints` (`canais.rs`).
//!
//! ⛔ **Os números são gravados no documento da timeline: só APPEND, nunca renumerar** (gate
//! `o_vocabulario_das_constraints_nao_renumera`).
//!
//! A entidade é a dona da pilha; **`i:j` = o `Item::id`** (32 bits: `i` os BAIXOS, `j` os altos) — o
//! endereço durável: reordenar a pilha não muda de quem é a curva. Cada campo é um grupo, partido por
//! item: o grupo parte-se por `i` ([`FieldKey::grupo`]), daí os bits baixos em `i`.

use crate::canal::{CampoInfo, Especie, FieldKey, familia::CONSTRAINT};

/// `Item::influence` (`0..=1`).
pub const FORCA: u16 = 1;
/// `Item::enabled` (0/1).
pub const LIGADA: u16 = 2;

/// Limit Rotation: o lado de baixo do arco, em graus.
pub const LIMITE_MIN: u16 = 3;
/// Limit Rotation: o lado de cima do arco, em graus.
pub const LIMITE_MAX: u16 = 4;
/// Limit Distance: a distância ao alvo, em metros (a unidade do mundo).
pub const DISTANCIA: u16 = 5;
/// Look At, IK de um osso e Follow Path: o desvio do eixo, em graus.
pub const DESVIO: u16 = 6;
/// Limit Distance: Inside · Outside · Surface ([`MODOS`]).
pub const MODO: u16 = 7;
/// Copy e Limit: World · Local ([`ESPACOS`]).
pub const ESPACO: u16 = 8;
/// Copy: Replace · Add ([`MISTURAS`]).
pub const MISTURA: u16 = 9;
/// IK: quantos ossos (0 = até à raiz).
pub const CORRENTE: u16 = 10;
/// IK: a suavidade, fracção do alcance.
pub const SUAVIDADE: u16 = 11;
/// IK: de que lado o joelho dobra ([`DOBRAS`]).
pub const DOBRA: u16 = 12;
/// IK: esticar além do alcance ([`NAO_SIM`]).
pub const ESTICA: u16 = 13;
/// Stretch To: o comprimento de repouso, em metros.
pub const REPOUSO: u16 = 14;
/// Stretch To: a largura ao esticar ([`VOLUMES`]).
pub const VOLUME: u16 = 15;
/// Follow Path: onde no caminho (fracção ou metros, conforme [`POR`]).
pub const POSICAO: u16 = 16;
/// Follow Path: a posição em fracção ou em distância ([`PORES`]).
pub const POR: u16 = 17;
/// Follow Path: rodar para a tangente ([`NAO_SIM`]).
pub const SEGUE: u16 = 18;
/// Pivot: o desvio do pivô em x e em y, em metros.
pub const DESVIO_X: u16 = 19;
pub const DESVIO_Y: u16 = 20;
/// Floor: o lado do plano ([`EIXOS`]).
pub const EIXO: u16 = 21;
/// Floor: a normal roda com o alvo ([`NAO_SIM`]).
pub const USA_ROTACAO: u16 = 22;
/// Transformation e Action: o canal do alvo que se lê ([`CANAIS`]).
pub const DE: u16 = 23;
/// Transformation: o canal do dono que se escreve ([`CANAIS`]).
pub const PARA: u16 = 24;
/// Transformation e Action: a faixa do canal do alvo (graus, metros ou %, conforme [`DE`]).
pub const DE_MIN: u16 = 25;
pub const DE_MAX: u16 = 26;
/// Transformation: a faixa do canal do dono (conforme [`PARA`]).
pub const PARA_MIN: u16 = 27;
pub const PARA_MAX: u16 = 28;
/// Transformation: fora da faixa continua a recta ([`NAO_SIM`]).
pub const EXTRAPOLA: u16 = 29;
/// Spline IK: os ossos mantêm o comprimento ou cobrem a curva ([`AJUSTES`]).
pub const AJUSTE: u16 = 30;
/// Copy Point: o índice do vértice da forma.
pub const PONTO: u16 = 31;
/// Action: a posição do clip na animação do dono.
pub const CLIP: u16 = 32;
/// Action: o instante do clip nas bordas da faixa, em segundos.
pub const INSTANTE_DE: u16 = 33;
pub const INSTANTE_ATE: u16 = 34;
/// Spring: segundos de uma oscilação.
pub const DURACAO: u16 = 35;
/// Spring: o ressalto, `0..1`.
pub const RESSALTO: u16 = 36;
/// Spring: que canais têm mola ([`MOLAS`]).
pub const MOLA: u16 = 37;
/// Follow Value: com mola ou a seguir no mesmo quadro ([`NAO_SIM`]).
pub const MOLA_LIGADA: u16 = 38;
/// *Behaviours* (W5.4): a amplitude do Tremer e do Flutuar (graus em Rotation).
pub const AMPLITUDE: u16 = 39;
/// *Behaviours*: ciclos por segundo do Tremer e do Flutuar.
pub const FREQUENCIA: u16 = 40;
/// Tremer: a linha do ruído.
pub const SEMENTE: u16 = 41;
/// Flutuar: a fase, em ciclos (o mesmo número que a semente do Tremer).
pub const FASE: u16 = 42;
/// Orbitar: graus por segundo.
pub const VELOCIDADE: u16 = 43;
/// Orbitar: a distância ao pivô sem alvo; Atrair: o alcance.
pub const RAIO: u16 = 44;
/// Atrair: a força no centro.
pub const PUXAO: u16 = 45;
/// Atrair: o travão (o `coefficient` do `force.drag`).
pub const AMORTECE: u16 = 46;
/// Tremer e Flutuar: onde mexem ([`ONDES`]).
pub const ONDE: u16 = 47;
/// Orbitar: o objecto vira com a volta ([`NAO_SIM`]).
pub const LEVA_ROTACAO: u16 = 48;
/// O peso por distância: o raio em que cai a zero.
pub const PESO_RAIO: u16 = 49;
/// O peso por distância: a curva ([`CURVAS`]).
pub const PESO_CURVA: u16 = 50;
/// O peso por distância: invertido ([`NAO_SIM`]).
pub const PESO_INVERTE: u16 = 51;

/// As palavras de [`MODO`], pelo índice.
pub const MODOS: &[&str] = &[
    "panel.inspector.constraint.mode.inside",
    "panel.inspector.constraint.mode.outside",
    "panel.inspector.constraint.mode.surface",
];
/// As palavras de [`ESPACO`], pelo índice.
pub const ESPACOS: &[&str] = &[
    "panel.inspector.constraint.space.world",
    "panel.inspector.constraint.space.local",
];
/// As palavras de [`MISTURA`], pelo índice.
pub const MISTURAS: &[&str] = &[
    "panel.inspector.constraint.mix.replace",
    "panel.inspector.constraint.mix.add",
];

/// As palavras de [`DOBRA`], pelo índice (`BendSide`: Keep · Ccw · Cw · Mixed).
pub const DOBRAS: &[&str] = &[
    "panel.inspector.constraint.bend.auto",
    "panel.inspector.constraint.bend.ccw",
    "panel.inspector.constraint.bend.cw",
    "panel.inspector.constraint.bend.mixed",
];
/// As palavras de [`ESTICA`] e [`SEGUE`].
pub const NAO_SIM: &[&str] = &[
    "panel.inspector.constraint.no",
    "panel.inspector.constraint.yes",
];
/// As palavras de [`VOLUME`] (`Volume`: Xz · Area · Nenhum).
pub const VOLUMES: &[&str] = &[
    "panel.inspector.constraint.volume.xz",
    "panel.inspector.constraint.volume.area",
    "panel.inspector.constraint.volume.none",
];
/// As palavras de [`POR`] (`PathBy`: Fraction · Distance).
pub const PORES: &[&str] = &[
    "panel.inspector.constraint.by.fraction",
    "panel.inspector.constraint.by.distance",
];
/// As palavras de [`EIXO`] (`EixoDoChao`: +Y · −Y · +X · −X).
pub const EIXOS: &[&str] = &[
    "panel.inspector.constraint.axis.y",
    "panel.inspector.constraint.axis.neg_y",
    "panel.inspector.constraint.axis.x",
    "panel.inspector.constraint.axis.neg_x",
];
/// As palavras de [`DE`] e [`PARA`] (`Canal`: X · Y · Rotation · Scale X · Scale Y).
pub const CANAIS: &[&str] = &[
    "panel.inspector.constraint.channel.x",
    "panel.inspector.constraint.channel.y",
    "panel.inspector.constraint.channel.rotation",
    "panel.inspector.constraint.channel.scale_x",
    "panel.inspector.constraint.channel.scale_y",
];
/// As palavras de [`AJUSTE`] (`Ajuste`: Nenhum · Esticar).
pub const AJUSTES: &[&str] = &[
    "panel.inspector.constraint.fit.none",
    "panel.inspector.constraint.fit.curve",
];
/// As palavras de [`MOLA`]: posição · rotação · as duas.
pub const MOLAS: &[&str] = &[
    "panel.inspector.constraint.spring.location",
    "panel.inspector.constraint.spring.rotation",
    "panel.inspector.constraint.spring.both",
];
/// As palavras de [`ONDE`] (`Onde`: XY · X · Y · Rotation).
pub const ONDES: &[&str] = &[
    "panel.inspector.constraint.where.xy",
    "panel.inspector.constraint.channel.x",
    "panel.inspector.constraint.channel.y",
    "panel.inspector.constraint.channel.rotation",
];
/// As palavras de [`PESO_CURVA`] (as do `motion.falloff`: Linear · Quad · Smooth · Smoother).
pub const CURVAS: &[&str] = &[
    "panel.inspector.constraint.curve.linear",
    "panel.inspector.constraint.curve.quad",
    "panel.inspector.constraint.curve.smooth",
    "panel.inspector.constraint.curve.smoother",
];
/// Os campos de um item, na ordem do cartão (o tipo diz quais tem).
pub const CAMPOS: [u16; 51] = [
    FORCA,
    LIGADA,
    LIMITE_MIN,
    LIMITE_MAX,
    DISTANCIA,
    DESVIO,
    MODO,
    ESPACO,
    MISTURA,
    CORRENTE,
    SUAVIDADE,
    DOBRA,
    ESTICA,
    REPOUSO,
    VOLUME,
    POSICAO,
    POR,
    SEGUE,
    DESVIO_X,
    DESVIO_Y,
    EIXO,
    USA_ROTACAO,
    DE,
    PARA,
    DE_MIN,
    DE_MAX,
    PARA_MIN,
    PARA_MAX,
    EXTRAPOLA,
    AJUSTE,
    PONTO,
    CLIP,
    INSTANTE_DE,
    INSTANTE_ATE,
    DURACAO,
    RESSALTO,
    MOLA,
    MOLA_LIGADA,
    AMPLITUDE,
    FREQUENCIA,
    SEMENTE,
    FASE,
    VELOCIDADE,
    RAIO,
    PUXAO,
    AMORTECE,
    ONDE,
    LEVA_ROTACAO,
    PESO_RAIO,
    PESO_CURVA,
    PESO_INVERTE,
];

/// A chave do campo `campo` do item `item`.
#[must_use]
pub const fn chave(campo: u16, item: u32) -> FieldKey {
    #[expect(clippy::cast_possible_truncation, reason = "as duas metades do id")]
    FieldKey::new(CONSTRAINT, campo, item as u16, (item >> 16) as u16)
}

/// O `Item::id` que a chave endereça.
#[must_use]
pub const fn item_de(k: FieldKey) -> u32 {
    (k.j as u32) << 16 | k.i as u32
}

const fn c(rotulo: &'static str, grupo: u16, especie: Especie) -> CampoInfo {
    CampoInfo {
        rotulo,
        especie,
        grupo,
        por_i: true,
    }
}

/// O que o campo é.
#[must_use]
pub fn info(campo: u16) -> Option<CampoInfo> {
    Some(match campo {
        FORCA => c("canal.constraint.forca", campo, Especie::Continuo),
        LIGADA => c("canal.constraint.ligada", campo, Especie::Ligado),
        LIMITE_MIN => c("canal.constraint.limite_min", campo, Especie::Continuo),
        LIMITE_MAX => c("canal.constraint.limite_max", campo, Especie::Continuo),
        DISTANCIA => c("canal.constraint.distancia", campo, Especie::Continuo),
        DESVIO => c("canal.constraint.desvio", campo, Especie::Continuo),
        MODO => c("canal.constraint.modo", campo, Especie::Opcoes(MODOS)),
        ESPACO => c("canal.constraint.espaco", campo, Especie::Opcoes(ESPACOS)),
        MISTURA => c("canal.constraint.mistura", campo, Especie::Opcoes(MISTURAS)),
        CORRENTE => c("canal.constraint.corrente", campo, Especie::Continuo),
        SUAVIDADE => c("canal.constraint.suavidade", campo, Especie::Continuo),
        DOBRA => c("canal.constraint.dobra", campo, Especie::Opcoes(DOBRAS)),
        ESTICA => c("canal.constraint.estica", campo, Especie::Opcoes(NAO_SIM)),
        REPOUSO => c("canal.constraint.repouso", campo, Especie::Continuo),
        VOLUME => c("canal.constraint.volume", campo, Especie::Opcoes(VOLUMES)),
        POSICAO => c("canal.constraint.posicao", campo, Especie::Continuo),
        POR => c("canal.constraint.por", campo, Especie::Opcoes(PORES)),
        SEGUE => c("canal.constraint.segue", campo, Especie::Opcoes(NAO_SIM)),
        DESVIO_X => c("canal.constraint.desvio_x", campo, Especie::Continuo),
        DESVIO_Y => c("canal.constraint.desvio_y", campo, Especie::Continuo),
        EIXO => c("canal.constraint.eixo", campo, Especie::Opcoes(EIXOS)),
        USA_ROTACAO => c(
            "canal.constraint.usa_rotacao",
            campo,
            Especie::Opcoes(NAO_SIM),
        ),
        DE => c("canal.constraint.de", campo, Especie::Opcoes(CANAIS)),
        PARA => c("canal.constraint.para", campo, Especie::Opcoes(CANAIS)),
        DE_MIN => c("canal.constraint.de_min", campo, Especie::Continuo),
        DE_MAX => c("canal.constraint.de_max", campo, Especie::Continuo),
        PARA_MIN => c("canal.constraint.para_min", campo, Especie::Continuo),
        PARA_MAX => c("canal.constraint.para_max", campo, Especie::Continuo),
        EXTRAPOLA => c(
            "canal.constraint.extrapola",
            campo,
            Especie::Opcoes(NAO_SIM),
        ),
        AJUSTE => c("canal.constraint.ajuste", campo, Especie::Opcoes(AJUSTES)),
        PONTO => c("canal.constraint.ponto", campo, Especie::Continuo),
        CLIP => c("canal.constraint.clip", campo, Especie::Continuo),
        INSTANTE_DE => c("canal.constraint.instante_de", campo, Especie::Continuo),
        INSTANTE_ATE => c("canal.constraint.instante_ate", campo, Especie::Continuo),
        DURACAO => c("canal.constraint.duracao", campo, Especie::Continuo),
        RESSALTO => c("canal.constraint.ressalto", campo, Especie::Continuo),
        MOLA => c("canal.constraint.mola", campo, Especie::Opcoes(MOLAS)),
        MOLA_LIGADA => c(
            "canal.constraint.mola_ligada",
            campo,
            Especie::Opcoes(NAO_SIM),
        ),
        AMPLITUDE => c("canal.constraint.amplitude", campo, Especie::Continuo),
        FREQUENCIA => c("canal.constraint.frequencia", campo, Especie::Continuo),
        SEMENTE => c("canal.constraint.semente", campo, Especie::Continuo),
        FASE => c("canal.constraint.fase", campo, Especie::Continuo),
        VELOCIDADE => c("canal.constraint.velocidade", campo, Especie::Continuo),
        RAIO => c("canal.constraint.raio", campo, Especie::Continuo),
        PUXAO => c("canal.constraint.puxao", campo, Especie::Continuo),
        AMORTECE => c("canal.constraint.amortece", campo, Especie::Continuo),
        ONDE => c("canal.constraint.onde", campo, Especie::Opcoes(ONDES)),
        LEVA_ROTACAO => c(
            "canal.constraint.leva_rotacao",
            campo,
            Especie::Opcoes(NAO_SIM),
        ),
        PESO_RAIO => c("canal.constraint.peso_raio", campo, Especie::Continuo),
        PESO_CURVA => c(
            "canal.constraint.peso_curva",
            campo,
            Especie::Opcoes(CURVAS),
        ),
        PESO_INVERTE => c(
            "canal.constraint.peso_inverte",
            campo,
            Especie::Opcoes(NAO_SIM),
        ),
        _ => return None,
    })
}

/// O nome do campo dentro do grupo: cada grupo é um campo só.
#[must_use]
pub fn membro(campo: u16) -> Option<&'static str> {
    Some(info(campo)?.rotulo)
}

#[cfg(test)]
#[path = "canal_constraint_tests.rs"]
mod tests;
