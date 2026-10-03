//! As fileiras do céu: a secção só no Render por malha, cada fileira escreve o seu número, e todo o
//! texto vem do i18n.

use super::{CEUS, LINHAS, rows, with_number};
use crate::ceu_foto::Ceu;

fn escolhido() -> Ceu {
    Ceu {
        qual: 7,
        ..Ceu::default()
    }
}

#[test]
fn fora_do_render_por_malha_nao_ha_seccao() {
    assert!(rows(escolhido(), false).is_empty());
    let r = rows(escolhido(), true);
    assert_eq!(r.len(), LINHAS.len());
    assert_eq!(r[0].section, Some("panel.model3d.section.sky"));
}

/// ⭐ Cada fileira escreve a SUA posição da arrumação — e só ela.
#[test]
fn cada_fileira_escreve_no_seu_numero() {
    let c = escolhido();
    for r in rows(c, true) {
        let ph2d_field::Param::Sky(slot) = r.param else {
            panic!("fileira do céu com outra família: {:?}", r.param);
        };
        let alvo = if r.integral {
            r.lo
        } else {
            (r.lo + 0.37 * (hi(&r) - r.lo)).round()
        };
        let alvo = if r.integral && (c.pack()[slot as usize] - alvo).abs() < 0.5 {
            hi(&r)
        } else {
            alvo
        };
        let novo = with_number(c, slot, alvo);
        let (a, b) = (c.pack(), novo.pack());
        for k in 0..a.len() {
            if k == slot as usize {
                assert!(
                    (b[k] - alvo).abs() < 1.0e-4,
                    "{}: escreveu {} e ficou {}",
                    r.key,
                    alvo,
                    b[k]
                );
            } else {
                assert_eq!(a[k], b[k], "{} mexeu na posição {k}", r.key);
            }
        }
    }
    // Fora da arrumação: o céu intacto.
    assert_eq!(with_number(c, 99, 1.0), c);
}

fn hi(r: &ph2d_panel_model3d::ParamRow) -> f32 {
    match r.bound {
        ph2d_field::Bound::Hard(h) | ph2d_field::Bound::Soft(h) => h,
        _ => r.lo + 1.0,
    }
}

/// ⭐ **Os nomes seguem os céus embarcados** — o `qual = i + 1` é o `Embarcado::TODOS[i]`, e o nome
/// que o painel mostra é o daquele ficheiro (a foto de estúdio chama-se `photo_studio`).
#[test]
fn os_nomes_seguem_os_embarcados() {
    assert_eq!(CEUS.len(), ph2d_sky::Embarcado::TODOS.len() + 1);
    for (i, e) in ph2d_sky::Embarcado::TODOS.iter().enumerate() {
        let nome = if e.chave() == "studio" {
            "photo_studio"
        } else {
            e.chave()
        };
        assert_eq!(CEUS[i + 1], format!("panel.model3d.sky.{nome}"));
        let c = Ceu {
            qual: (i + 1) as u8,
            ..Ceu::default()
        };
        assert_eq!(c.embarcado(), Some(*e));
    }
    assert_eq!(
        Ceu::default().embarcado(),
        None,
        "o 0 é o estúdio de sempre"
    );
}

/// ⭐ **Todo o texto vem do i18n** (HR-15): rótulos, opções, dicas e razões de fileira apagada.
#[test]
fn todo_o_texto_tem_traducao() {
    let mut chaves: Vec<&str> = CEUS.to_vec();
    for r in rows(escolhido(), true) {
        chaves.push(r.key);
        chaves.extend_from_slice(r.choices);
        chaves.extend(r.section);
    }
    let apagadas = rows(Ceu::default(), true);
    chaves.extend(apagadas.iter().filter_map(|r| r.inert));
    let sem_fundo = rows(
        Ceu {
            fundo: false,
            ..escolhido()
        },
        true,
    );
    chaves.extend(sem_fundo.iter().filter_map(|r| r.inert));
    chaves.extend(rows(patio_pronto(), true).iter().filter_map(|r| r.inert));
    assert!(chaves.contains(&"field.inert.sky_is_studio"));
    assert!(chaves.contains(&"field.inert.sky_background_is_off"));
    assert!(chaves.contains(&"field.inert.sky_has_no_sun"));
    for k in chaves {
        assert_ne!(ph2d_i18n::tr(k), k, "a chave {k} não tem texto");
    }
}

/// O pátio (nublado) com o atlas JÁ montado — a luz-chave só se apaga quando a lei o diz.
fn patio_pronto() -> Ceu {
    let e = ph2d_sky::Embarcado::Patio;
    let inicio = std::time::Instant::now();
    while crate::ceu_foto::pronto(e).is_none() {
        assert!(
            inicio.elapsed().as_secs() < 120,
            "o atlas do pátio não chegou"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let qual = ph2d_sky::Embarcado::TODOS
        .iter()
        .position(|x| *x == e)
        .expect("embarcado")
        + 1;
    Ceu {
        qual: qual as u8,
        ..Ceu::default()
    }
}

/// ⭐ **Um céu sem sol apaga a luz-chave** (o pátio nublado), com a razão; o resto continua a valer.
#[test]
fn um_ceu_sem_sol_apaga_a_luz_chave() {
    let r = rows(patio_pronto(), true);
    for l in &r {
        let ph2d_field::Param::Sky(slot) = l.param else {
            panic!("fileira do céu com outra família");
        };
        let esperado = (slot == 3).then_some("field.inert.sky_has_no_sun");
        assert_eq!(l.inert, esperado, "{}", l.key);
    }
}

/// ⭐ **O estúdio apaga as outras fileiras** (não há foto para girar), e o fundo desligado apaga o
/// desfoque — com a razão escrita.
#[test]
fn o_estudio_apaga_as_outras_fileiras() {
    let r = rows(Ceu::default(), true);
    assert!(r[0].inert.is_none(), "a escolha do céu nunca se apaga");
    assert!(
        r[1..]
            .iter()
            .all(|l| l.inert == Some("field.inert.sky_is_studio"))
    );
    let r = rows(escolhido(), true);
    assert!(
        r.iter().all(|l| l.inert.is_none()),
        "com uma foto e fundo, tudo vale"
    );
}
