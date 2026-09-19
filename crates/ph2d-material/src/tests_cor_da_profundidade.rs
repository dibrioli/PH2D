//! ⭐⭐⭐ **OS GATES DA MATIZ QUE SEGUE A PROFUNDIDADE** — a cura medida da `docs/Render3d/10` §17.
//!
//! ⚠️ Eles vivem num módulo próprio e não no [`super::tests`] porque aquele estava a `681` linhas
//! contra um tecto de `700`: a cura de um tecto é **corte por responsabilidade**, nunca uma isenção
//! — e a responsabilidade destes é outra (a §17 é a primeira lei desta crate que **diverge** da
//! referência de propósito, em vez de a portar).

use crate::{OpenPbr, Rgb, subsurface};

/// A cor das fixturas do oráculo.
const COR: Rgb = [0.75, 0.35, 0.35];
/// O raio da peça da cena `=33`, em unidades do mundo.
const RAIO: f32 = 0.42;

/// ⭐⭐⭐ **A FIXTURA COM CABEÇALHO — quatro profundidades de um traçado CONVERGIDO.**
///
/// | | |
/// |---|---|
/// | oráculo | traçado de caminhos convergido (`4096` amostras, amostragem adaptativa **desligada**) |
/// | cena | a nossa `=33`, esfera de raio `0,42`, lâmpada pontual, céu PRETO |
/// | material | `subsurface_weight = 1` · cor `(0,75 · 0,35 · 0,35)` · especular `0` · raios **IGUAIS** nos três canais |
/// | grandeza | `R/B` **linear**, sobre a silhueta iluminada do controlo opaco (máscara fixa) |
/// | confirmação | um **segundo** traçado independente concorda a `~5 %` nos dois pontos em que está vivo |
/// | piso de ruído | `~0,1 %` — o oráculo não é bit-reprodutível entre invocações |
///
/// ⛔ **A família de raios IGUAIS é o controlo da wave**, e não uma amostra: com os três canais a
/// viajar a mesma distância, toda mudança de matiz é da PROFUNDIDADE.
const ORACULO: [(f32, f32); 4] = [
    (0.03, 2.1092),
    (0.10, 1.7007),
    (0.30, 1.3467),
    (1.00, 1.1931),
];

/// A barra: o pior erro que a calibração de duas constantes deixou, com folga.
///
/// ⛔ **Não é escolhida** — ela sai da tabela acima: a lei erra `2,8 %` no pior ponto, e `4 %` é
/// essa medição com margem. ⚠️ E a metade que a torna honesta é o **CONTROLO**: a lei de hoje erra
/// `79,6 %` no mesmo ponto, logo uma barra que a deixasse passar não estaria a afirmar nada.
const BARRA: f32 = 0.04;

/// A matiz que a lei devolve, pela porta do PRODUTO.
fn matiz(mfp: f32, peso: f32) -> f32 {
    let s = OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: false,
        subsurface_color: COR,
        base_color: COR,
        specular_weight: 0.0,
        subsurface_radius: mfp,
        subsurface_radius_scale: [1.0; 3],
        subsurface_depth_hue: peso,
        ..OpenPbr::default()
    }
    .prepare()
    .at_curvature(1.0 / RAIO);
    let c = s.direct([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.6, 0.0, 0.8], [1.0; 3]);
    c[0] / c[2]
}

/// ⭐⭐⭐ **O PONTO NEUTRO É BYTE-IDÊNTICO — e não por promessa, por ÁLGEBRA.**
///
/// A lei escreve-se `A · (α/A)^((1−f)·peso)`. Com `peso = 0` o expoente é **exactamente** `0`, e
/// `x^0` é `1` ao bit para todo `x` finito ⇒ a cor volta a ser `A` sem passar por nenhum caminho
/// especial. ⚠️ **A renormalização de luminância tem a mesma propriedade**: ela divide `lum(cor)`
/// por si próprio, e `y/y` é `1,0` exacto em IEEE-754.
///
/// ⇒ *o caminho de omissão deste produto não muda um bit*, que é a condição para a paridade da
/// §4.1 contra o renderizador de referência continuar de pé.
#[test]
fn o_ponto_neutro_da_matiz_e_byte_identico() {
    let mut casos = 0;
    for mfp in [0.0f32, 0.001, 0.03, 0.1, 0.42, 1.0, 10.0] {
        for k in [0.0f32, 0.01, 1.0 / RAIO, 100.0] {
            for cor in [COR, [0.0; 3], [1.0; 3], [0.9, 0.1, 0.5]] {
                let saiu = subsurface::cor_na_profundidade(cor, [mfp; 3], k, 0.0);
                assert_eq!(
                    saiu.map(f32::to_bits),
                    cor.map(f32::to_bits),
                    "com peso 0 a cor tem de voltar AO BIT (mfp {mfp}, κ {k}, cor {cor:?})"
                );
                casos += 1;
            }
        }
    }
    // ⚠️ Piso de população: uma varredura que encolha para zero passaria trivialmente.
    assert!(casos >= 100, "o censo mediu só {casos} casos");
}

/// ⭐⭐⭐ **OS DOIS EXTREMOS SÃO DERIVADOS — e é isso que separa esta lei de um ajuste.**
///
/// | regime | o que a física diz | o que a lei tem de dar |
/// |---|---|---|
/// | `mfp·κ → 0` | muitos eventos de espalhamento ⇒ a reflectância autorada | **a cor do painel** |
/// | `mfp·κ → ∞` | poucos; a luz atravessa ⇒ o albedo cru | a razão dos **albedos** |
///
/// ⛔ O valor do extremo fino **não é calibrado**: ele sai da inversão publicada de
/// Christensen & Burley. Se alguém trocar essa inversão, este gate reprova — que é o que se quer.
#[test]
fn os_dois_extremos_da_matiz_sao_derivados_e_nao_calibrados() {
    let autorada = COR[0] / COR[2];
    // O extremo ESPESSO: uma peça enorme contra um caminho livre minúsculo.
    let espesso = subsurface::cor_na_profundidade(COR, [1e-6; 3], 1.0 / RAIO, 1.0);
    let r_espesso = espesso[0] / espesso[2];
    assert!(
        (r_espesso / autorada - 1.0).abs() < 1e-3,
        "no extremo espesso a matiz tem de ser a AUTORADA: {r_espesso} contra {autorada}"
    );
    // O extremo FINO: o albedo cru, pela inversão publicada.
    let alvo = {
        let a = |x: f32| 1.0 - (-5.09406 * x + 2.61188 * x * x - 4.31805 * x * x * x).exp();
        a(COR[0]) / a(COR[2])
    };
    let fino = subsurface::cor_na_profundidade(COR, [1e6; 3], 1.0 / RAIO, 1.0);
    let r_fino = fino[0] / fino[2];
    assert!(
        (r_fino / alvo - 1.0).abs() < 1e-3,
        "no extremo fino a matiz tem de ser a do ALBEDO CRU: {r_fino} contra {alvo}"
    );
    // ⚠️ E os dois têm de ser DIFERENTES — senão o gate acima passaria com uma lei inerte.
    assert!(
        (alvo / autorada - 1.0).abs() > 0.3,
        "os dois extremos colapsaram ({autorada} contra {alvo}) — o gate deixou de afirmar"
    );
}

/// ⭐⭐⭐ **A LEI BATE O TRAÇADO CONVERGIDO, e a lei de HOJE não bate — o controlo é metade do gate.**
///
/// ⛔ Sem a coluna «hoje» este gate não afirmaria nada: ele mediria que a lei nova está perto do
/// oráculo sem nunca mostrar que havia alguma coisa para curar. *Uma barra sem o lado que ela
/// reprova é uma barra que ninguém pode falhar.*
#[test]
fn a_matiz_bate_o_tracado_convergido_e_a_lei_de_hoje_nao() {
    let (mut pior_cura, mut pior_hoje) = (0.0f32, 0.0f32);
    for (mfp, alvo) in ORACULO {
        pior_cura = pior_cura.max((matiz(mfp, 1.0) / alvo - 1.0).abs());
        pior_hoje = pior_hoje.max((matiz(mfp, 0.0) / alvo - 1.0).abs());
    }
    assert!(
        pior_cura <= BARRA,
        "a cura erra {:.1} % contra a barra de {:.1} %",
        100.0 * pior_cura,
        100.0 * BARRA
    );
    // ⭐ O CONTROLO: a aproximação publicada tem de REPROVAR nesta mesma barra.
    assert!(
        pior_hoje > 10.0 * BARRA,
        "a lei de HOJE erra só {:.1} % — o defeito que esta wave cura não está na fixtura",
        100.0 * pior_hoje
    );
}

/// ⭐⭐ **A cura toca na MATIZ e não no BRILHO** — a decisão medida da §17.6.
///
/// O albedo é sempre **maior** que a reflectância (o espalhamento múltiplo perde energia), logo a
/// correcção crua clarearia a peça inteira. ⛔ Mas o defeito medido é só de matiz: a magnitude da
/// lei já responde `1,76×` no terminador e `3,50×` do lado escuro. ⇒ *curar o que não está partido
/// seria trocar um defeito medido por um não medido.*
#[test]
fn a_cura_preserva_a_luminancia_e_muda_so_a_matiz() {
    let lum = |c: Rgb| 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
    let mut mexeu = false;
    for mfp in [0.03f32, 0.1, 0.3, 1.0, 3.0] {
        let antes = COR;
        let depois = subsurface::cor_na_profundidade(antes, [mfp; 3], 1.0 / RAIO, 1.0);
        assert!(
            (lum(depois) / lum(antes) - 1.0).abs() < 1e-5,
            "a luminância mexeu-se em mfp {mfp}: {} contra {}",
            lum(depois),
            lum(antes)
        );
        if ((depois[0] / depois[2]) / (antes[0] / antes[2]) - 1.0).abs() > 0.02 {
            mexeu = true;
        }
    }
    // ⚠️ A metade que impede a cura barata: devolver a cor intacta preservaria a luminância
    // **trivialmente**. A matiz tem de se ter mexido em pelo menos uma profundidade.
    assert!(mexeu, "a matiz não se mexeu em profundidade nenhuma");
}
