//! **Como nasce uma variação.** Roll e Mutate são a mesma função em dois tamanhos.
//!
//! O sorteio de cada campo é fixado pelo número e pela camada; o tamanho só o escala. Por isso arrastar para longe muda mais e
//! arrastar de volta devolve, e travar um campo não muda o sorteio dos outros. Anda a partir do valor atual, nunca pela faixa
//! do campo: pela faixa, 6 de 64 variações saíram mudas (`docs/SfxLab/05_W4_plano.md` §6).

use ph2d_a11y::NodeId;
use ph2d_det_math::DetMath;
use ph2d_sfx_dsp::Rng;
use ph2d_sfx_lab::tabela::{CAMPOS, Campo, Grupo, OPCOES, Onde};
use ph2d_sfx_recipe::{Layer, LayerId, Recipe};
use ph2d_tool_registry::hash_node_id;

/// O Roll: até ×4,5 em cada valor. Com ele, nenhuma de 64 variações saiu muda (§6). O ouvido do dono é quem confirma.
pub const TAMANHO_ROLL: f64 = 1.5;
/// O Mutate: até ±35 %, e a maior diferença de volume numa ninhada foi 5,3 dB (§6).
pub const TAMANHO_MUTATE: f64 = 0.3;
/// A partir daqui a onda, o filtro e a lei do corpo também podem trocar. O Mutate não muda a natureza do som.
const TAMANHO_DOS_DISCRETOS: f64 = 1.0;
/// Uma em quatro: trocar sempre tiraria a família do som que se está procurando.
const CHANCE_DE_DISCRETO: u64 = 4;
const SAL_DA_CAMADA: u64 = 0xD6E8_FEB8_6659_FD93;

/// Como um campo anda.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Lei {
    /// Multiplica: altura, tempo e tudo o que se ouve em proporção. Zero continua zero.
    Fator,
    /// Soma este tanto por unidade de tamanho, na unidade da tela.
    Soma(f64),
}

/// Os três pares início/fim, nomeados UMA vez: a lista de leis e a dos pares leem os mesmos.
const LEI_NOISE_HZ: NodeId = hash_node_id("sfxlab_noise_hz");
const LEI_NOISE_HZ_END: NodeId = hash_node_id("sfxlab_noise_hz_end");
const LEI_TONE_HZ: NodeId = hash_node_id("sfxlab_tone_hz");
const LEI_TONE_HZ_END: NodeId = hash_node_id("sfxlab_tone_hz_end");
const LEI_FM_INDEX: NodeId = hash_node_id("sfxlab_fm_index");
const LEI_FM_INDEX_END: NodeId = hash_node_id("sfxlab_fm_index_end");

/// A lei de cada campo de camada. O gate `todo_campo_de_camada_tem_lei` reprova campo novo sem linha aqui.
pub const LEIS: &[(NodeId, Lei)] = &[
    (hash_node_id("sfxlab_gain"), Lei::Soma(4.0)),
    (hash_node_id("sfxlab_start"), Lei::Soma(10.0)),
    (LEI_NOISE_HZ, Lei::Fator),
    (LEI_NOISE_HZ_END, Lei::Fator),
    (hash_node_id("sfxlab_noise_q"), Lei::Fator),
    (LEI_TONE_HZ, Lei::Fator),
    (LEI_TONE_HZ_END, Lei::Fator),
    (hash_node_id("sfxlab_tone_glide"), Lei::Fator),
    (hash_node_id("sfxlab_fm_ratio"), Lei::Fator),
    (LEI_FM_INDEX, Lei::Fator),
    (LEI_FM_INDEX_END, Lei::Fator),
    (hash_node_id("sfxlab_body_stretch"), Lei::Soma(0.1)),
    (hash_node_id("sfxlab_body_base"), Lei::Fator),
    (hash_node_id("sfxlab_body_modes"), Lei::Fator),
    (hash_node_id("sfxlab_body_decay"), Lei::Fator),
    (hash_node_id("sfxlab_body_damping"), Lei::Fator),
    (hash_node_id("sfxlab_body_rolloff"), Lei::Fator),
    (hash_node_id("sfxlab_body_contact"), Lei::Fator),
    (hash_node_id("sfxlab_body_noise"), Lei::Soma(10.0)),
    (hash_node_id("sfxlab_rec_semitones"), Lei::Soma(3.0)),
    (hash_node_id("sfxlab_rec_offset"), Lei::Fator),
    (hash_node_id("sfxlab_attack"), Lei::Fator),
    (hash_node_id("sfxlab_decay"), Lei::Fator),
    (hash_node_id("sfxlab_close"), Lei::Fator),
];

/// Início e fim que nascem iguais continuam iguais: senão todo tom parado ganharia um deslize.
const PARES: &[(NodeId, NodeId)] = &[
    (LEI_NOISE_HZ, LEI_NOISE_HZ_END),
    (LEI_TONE_HZ, LEI_TONE_HZ_END),
    (LEI_FM_INDEX, LEI_FM_INDEX_END),
];

#[must_use]
pub fn lei_de(id: NodeId) -> Option<Lei> {
    LEIS.iter().find(|(i, _)| *i == id).map(|(_, l)| *l)
}

/// A variação de número `numero` da `base`. Tamanho zero devolve a base. A duração e o som aprovado nunca mudam.
#[must_use]
pub fn varia(base: &Recipe, numero: u64, tamanho: f64) -> Recipe {
    let mut r = base.clone();
    if tamanho <= 0.0 {
        return r;
    }
    let ids: Vec<LayerId> = base.layers.iter().map(|l| l.id).collect();
    for id in ids {
        varia_camada(&mut r, base, id, numero, tamanho);
    }
    r
}

fn varia_camada(r: &mut Recipe, base: &Recipe, id: LayerId, numero: u64, tamanho: f64) {
    let Some(de_antes) = base.layer(id) else {
        return;
    };
    if de_antes.travada {
        return;
    }
    let mut rng = Rng::new(numero ^ id.0.wrapping_mul(SAL_DA_CAMADA));
    // A semente da camada anda sozinha: a da receita fica, e a camada travada não ouve o sorteio.
    let variacao = rng.next_u64() | 1;
    for c in CAMPOS {
        // Sorteia antes de olhar: um campo travado não desloca o sorteio dos outros.
        let u = f64::from(rng.bipolar());
        if let Some(v) = novo_valor(c, de_antes, base, u, tamanho) {
            c.escreve(r, Some(id), v);
        }
    }
    for (inicio, fim) in PARES {
        iguala_o_fim(r, base, id, *inicio, *fim);
    }
    for grupo in [Grupo::Filter, Grupo::Wave, Grupo::Law] {
        let sorteio = rng.next_u64();
        if tamanho >= TAMANHO_DOS_DISCRETOS && sorteio.is_multiple_of(CHANCE_DE_DISCRETO) {
            troca_a_escolha(r, id, grupo, sorteio);
        }
    }
    if let Some(l) = r.layer_mut(id) {
        l.variacao = variacao;
    }
}

fn novo_valor(c: &Campo, l: &Layer, base: &Recipe, u: f64, tamanho: f64) -> Option<f64> {
    if !matches!(c.onde, Onde::Camada { .. }) || l.travas.contains(&c.id.0) {
        return None;
    }
    let v = c.le(base, Some(l))?;
    if !dentro_da_faixa(c, v) {
        return None;
    }
    Some(match lei_de(c.id)? {
        Lei::Fator => v * (u * tamanho).det_exp(),
        Lei::Soma(por_unidade) => v + u * tamanho * por_unidade,
    })
}

/// O valor que veio de fora da faixa do campo foi posto de propósito: o sorteio não o corta.
fn dentro_da_faixa(c: &Campo, v: f64) -> bool {
    (c.faixa.0..=c.faixa.1).contains(&v)
}

fn iguala_o_fim(r: &mut Recipe, base: &Recipe, id: LayerId, inicio: NodeId, fim: NodeId) {
    let (Some(ci), Some(cf)) = (campo(inicio), campo(fim)) else {
        return;
    };
    let Some(l) = base.layer(id) else {
        return;
    };
    let de_antes = ci.le(base, Some(l));
    if l.travas.contains(&fim.0)
        || de_antes != cf.le(base, Some(l))
        || !de_antes.is_some_and(|v| dentro_da_faixa(ci, v))
    {
        return;
    }
    if let Some(novo) = r.layer(id).and_then(|n| ci.le(r, Some(n))) {
        cf.escreve(r, Some(id), novo);
    }
}

fn campo(id: NodeId) -> Option<&'static Campo> {
    CAMPOS.iter().find(|c| c.id == id)
}

fn troca_a_escolha(r: &mut Recipe, id: LayerId, grupo: Grupo, sorteio: u64) {
    let Some(l) = r.layer_mut(id) else {
        return;
    };
    let dele: Vec<_> = OPCOES
        .iter()
        .filter(|o| o.grupo == grupo && (o.aceso)(l).is_some())
        .collect();
    if dele.is_empty() || dele.iter().any(|o| l.travas.contains(&o.id.0)) {
        return;
    }
    let i = ((sorteio >> 8) % dele.len() as u64) as usize;
    (dele[i].escolhe)(l);
}
