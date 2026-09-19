//! Os gates da [`super::razao_inerte`] — a lei pura, sem mundo e sem painel.
//!
//! ⚠️ **Estes gates afirmam a FORMA da lei** (a ordem dos braços, a ausência de travamento mudo, a
//! partição a existir). ⛔ **Quem afirma que ela diz a VERDADE sobre o barro é o censo**, na
//! `ph2d-app-field3d`, que compara o que ela tranca com a medição **ao bit** da
//! [`ph2d_material::Surface::direct`] — *uma lei sobre o modelo de sombreamento não se prova numa
//! crate que não tem o modelo de sombreamento*.

use super::razao_inerte;
use crate::FieldMaterial;

/// Um material com a subsuperfície ARMADA, que é o pré-requisito da partição.
fn com_subsuperficie(parede_fina: bool) -> FieldMaterial {
    FieldMaterial {
        subsurface_weight: 0.5,
        thin_walled: if parede_fina { 1.0 } else { 0.0 },
        ..FieldMaterial::default()
    }
}

/// ⭐⭐⭐ **A PARTIÇÃO: cada caminho tranca a metade que ele não lê** (pergunta do dono, 18/09).
///
/// ⚠️ **As duas metades, e nenhuma basta:** um gate que só afirmasse os trancados passaria com uma
/// lei que tranca TUDO, e um que só afirmasse os vivos passaria com uma lei que não tranca nada.
#[test]
fn cada_caminho_tranca_a_metade_que_nao_le() {
    let macico = com_subsuperficie(false);
    let fina = com_subsuperficie(true);

    // O raio e os três canais da escala dele: vivos no maciço, trancados na parede fina.
    for k in 27..=30u8 {
        assert_eq!(
            razao_inerte(&macico, k),
            None,
            "a posição {k} devia estar VIVA no caminho maciço"
        );
        assert_eq!(
            razao_inerte(&fina, k),
            Some("field.inert.thin_wall_has_no_depth"),
            "a posição {k} devia estar trancada na parede fina, com a razão da profundidade"
        );
    }
    // A anisotropia: ao contrário.
    assert_eq!(
        razao_inerte(&macico, 31),
        Some("field.inert.solid_scatters_isotropically")
    );
    assert_eq!(razao_inerte(&fina, 31), None);

    // ⭐ **O CONTROLO que impede a leitura preguiçosa:** o peso e o primeiro canal da cor vivem nos
    // DOIS. Sem ele, «a subsuperfície é uma partição» leria-se como sendo a família inteira, e a
    // cura seria esconder o painel todo.
    for k in [23u8, 24] {
        assert_eq!(razao_inerte(&macico, k), None);
        assert_eq!(razao_inerte(&fina, k), None);
    }
}

/// ⭐⭐⭐ **O BLOQUEIO MAIS EXTERNO FALA PRIMEIRO** — ver a ordem dos braços na [`super::razao_inerte`].
///
/// ⛔ Com o peso a zero **e** a parede fina ligada, as posições `27`–`30` têm duas razões
/// verdadeiras. Dizer *«uma parede fina não tem profundidade»* a quem também tem a subsuperfície
/// desligada é mandá-lo mexer no controlo que **não** o destranca.
#[test]
fn com_a_subsuperficie_desligada_a_razao_e_essa_e_nao_a_do_caminho() {
    let desligada = FieldMaterial {
        subsurface_weight: 0.0,
        thin_walled: 1.0,
        ..FieldMaterial::default()
    };
    for k in 27..=31u8 {
        assert_eq!(
            razao_inerte(&desligada, k),
            Some("field.inert.subsurface_is_off"),
            "a posição {k} devia culpar o PESO, que é o que a destranca"
        );
    }
}

/// ⭐ **Um material de omissão não tranca a base nem o realce** — o piso que impede uma lei que
/// tranque tudo de passar nos gates acima.
///
/// ⚠️ **E ele NÃO exige que nada esteja trancado:** o material de omissão tem o verniz e o brilho
/// desligados, logo aqueles ficam trancados de fábrica — *o que se afirma é que a base, que é o que
/// todo artista mexe primeiro, chega sempre*.
#[test]
fn o_material_de_omissao_nao_tranca_a_base() {
    let m = FieldMaterial::default();
    // `0`–`3`: o peso da base e os três canais da cor dela. `5`–`10`: metalness, o realce e a
    // rugosidade.
    for k in (0..=3u8).chain(5..=10) {
        assert_eq!(
            razao_inerte(&m, k),
            None,
            "a posição {k} nasceu trancada num material de fábrica"
        );
    }
}

/// ⭐⭐⭐ **NENHUM BRAÇO TRANCA EM SILÊNCIO, e a chave é sempre do vocabulário desta família.**
///
/// ⚠️ **Um `Some("")` ou uma chave de outro prefixo passaria nos gates acima** — e o sintoma seria
/// uma fileira apagada com uma linha em branco por baixo, ou com a chave crua à vista. ⭐ A varredura
/// é sobre **todos** os materiais que a partição alcança, e não sobre uma lista escrita à mão.
#[test]
fn toda_razao_e_uma_chave_do_vocabulario_desta_familia() {
    let mut vistas = 0usize;
    for metal in [0.0f32, 1.0] {
        for coat in [0.0f32, 0.5] {
            for emission in [0.0f32, 0.5] {
                for peso in [0.0f32, 0.5] {
                    for fina in [0.0f32, 1.0] {
                        let m = FieldMaterial {
                            metalness: metal,
                            coat,
                            emission,
                            subsurface_weight: peso,
                            thin_walled: fina,
                            ..FieldMaterial::default()
                        };
                        for k in 0..ph2d_field::MATERIAL_FIELDS {
                            if let Some(razao) = razao_inerte(&m, k) {
                                assert!(
                                    razao.starts_with("field.inert."),
                                    "a posição {k} tranca com a chave `{razao}`, que não é deste \
                                     vocabulário"
                                );
                                vistas += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    // ⚠️ **O PISO**: sem ele, uma lei que devolvesse sempre `None` passaria este gate a medir o
    // nada — a mesma cegueira que o censo por prefixo de nome já custou a este repo.
    assert!(
        vistas > 100,
        "só {vistas} travamentos em 32 materiais — esta lei deixou de trancar seja o que for"
    );
}
