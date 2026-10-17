//! I11 (Onda 2, `docs/CodeEditor/03_onda2.md`): as cores do assistente por cima das do tree-sitter.
//! O servidor é o FALSO de `assistente.rs`, a colorir como o `luau-lsp` 1.70.1 na sonda do I6/I11:
//! a biblioteca com `defaultLibrary`, os parâmetros, e NENHUM token para um global que não existe —
//! esse acusa-o o diagnóstico `1001`.
//!
//! ⛔ Toda asserção de «NÃO pergunta / NÃO pinta» dá tempo REAL a uma resposta chegar ([`nunca`]).

use crate::Mesa;
use crate::assistente::{Partilhado, Plano, acusa_pirnt, com_falso, converge, letra, tecla};
use crate::completar::{T0, quadro_em};
use crate::dicas::{ate, nunca};
use ph2d_editor_core::doc_surface::Key;
use ph2d_host::Modifiers;
use ph2d_panel_code::Ink;
use ph2d_script::ScriptId;
use std::time::{Duration, Instant};

/// `idle_parse_delay` e `idle_parse_delay_with_errors_found` do Godot 4.7.2.
const PAUSA: u128 = 1_500_000_000;
const PAUSA_COM_ERROS: u128 = 500_000_000;
const PASSO: u128 = 100_000_000;

/// Os índices da legenda do 1.70.1: `parameter` 7, `variable` 8, `function` 12; `readonly` (bit 2)
/// e `defaultLibrary` (bit 9).
const BIBLIOTECA: u64 = 4 | 512;

/// Como o `luau-lsp`: todo `ph2d` (até num comentário — o editor é que não o pinta lá), o membro
/// depois de `ph2d.`, `print`, e todo `dt`. Nunca `pirnt`.
pub(crate) fn colore(s: &str) -> crate::assistente::Tokens {
    let mut out = Vec::new();
    for (i, _) in s.match_indices("ph2d") {
        out.push((i..i + 4, 8, BIBLIOTECA));
        if s[i + 4..].starts_with('.') {
            let ini = i + 5;
            let fim = s[ini..]
                .find(|c: char| !c.is_ascii_alphanumeric())
                .map_or(s.len(), |k| ini + k);
            out.push((ini..fim, 12, BIBLIOTECA));
        }
    }
    for (i, _) in s.match_indices("print") {
        out.push((i..i + 5, 12, BIBLIOTECA));
    }
    for (i, _) in s.match_indices("dt") {
        out.push((i..i + 2, 7, 0));
    }
    out.sort_by_key(|t| t.0.start);
    out
}

/// O servidor acusa cada `pirnt` (e só isso: um texto sem ele não tem erros).
fn mesa(texto: &str, atraso: Duration) -> (Mesa, ScriptId, Partilhado) {
    let (mut m, ids, visto) = com_falso(
        &[("a.luau", texto)],
        Plano {
            atraso,
            regra: Some(acusa_pirnt),
            colorir: Some(colore),
            ..Plano::default()
        },
    );
    let id = ids[0];
    m.pede(id);
    converge(&mut m, &visto, "abrir");
    (m, id, visto)
}

fn cores(v: &Partilhado) -> usize {
    v.lock().unwrap().cores
}

fn tinta(m: &mut Mesa, id: ScriptId, at: usize) -> Ink {
    m.docs().ink_painted(id, at)
}

/// ⭐ Ao abrir, as cores chegam sem esperar a pausa (não houve escrita): `ph2d` e `ph2d.set` na cor da
/// biblioteca (o tree-sitter dá-lhes variável e função), o uso de `dt` na de parâmetro, o `pirnt` na
/// de alerta (pelo diagnóstico), o `ph2d` do comentário continua comentário. A escrever, nada se
/// pergunta: uma pergunta só depois da pausa — a de 0,5 s, porque o script tem um erro.
#[test]
fn as_cores_chegam_na_pausa_e_o_global_que_nao_existe_pinta_de_alerta() {
    let texto =
        "function update(self, dt)\n  ph2d.set(self.id, \"x\", dt)\n  pirnt(dt)\nend\n-- ph2d\n";
    let (mut m, id, visto) = mesa(texto, Duration::ZERO);
    let ph2d = texto.find("ph2d").unwrap();
    let set = ph2d + 5;
    let dt = texto.find("dt)\n  pirnt").unwrap();
    let pirnt = texto.find("pirnt").unwrap();
    let com = texto.rfind("ph2d").unwrap();
    ate(&mut m, id, T0, "as cores não chegaram", |m| {
        m.docs().semantic_count(id) > 0
    });
    assert_eq!(
        tinta(&mut m, id, ph2d),
        Ink::Builtin,
        "`ph2d` é da biblioteca"
    );
    assert_eq!(
        tinta(&mut m, id, set + 1),
        Ink::Builtin,
        "`ph2d.set` é da biblioteca"
    );
    assert_eq!(
        tinta(&mut m, id, dt),
        Ink::Parameter,
        "o uso de `dt` é o parâmetro"
    );
    assert_eq!(
        tinta(&mut m, id, com),
        Ink::Comment,
        "num comentário, a gramática manda"
    );
    ate(
        &mut m,
        id,
        T0,
        "o global que não existe não pintou de alerta",
        |m| m.docs().ink_painted(id, pirnt + 2) == Ink::Unresolved,
    );
    assert_eq!(cores(&visto), 1, "uma pergunta por versão");
    // Escrever uma letra a cada 0,1 s durante 3 s, no fim do ficheiro: nenhuma pausa, nenhuma pergunta.
    tecla(&mut m, id, Key::End, crate::ctrl(), None, "");
    let mut t = T0;
    for k in 0..30u128 {
        t = T0 + k * PASSO;
        quadro_em(&mut m, t);
        letra(&mut m, id, 'a');
        quadro_em(&mut m, t);
        std::thread::sleep(Duration::from_millis(2));
    }
    nunca(
        &mut m,
        id,
        t + PAUSA_COM_ERROS - 1,
        "perguntou antes da pausa",
        |_| cores(&visto) == 1,
    );
    ate(
        &mut m,
        id,
        t + PAUSA_COM_ERROS,
        "a pausa passou e não perguntou",
        |_| cores(&visto) == 2,
    );
    nunca(
        &mut m,
        id,
        t + PAUSA * 4,
        "perguntou duas vezes a mesma versão",
        |_| cores(&visto) == 2,
    );
}

/// Sem erros a pausa é a de 1,5 s.
#[test]
fn sem_erros_a_pausa_e_a_longa() {
    let texto = "local x = ph2d.get(1, \"x\")\n";
    let (mut m, id, visto) = mesa(texto, Duration::ZERO);
    ate(&mut m, id, T0, "as cores não chegaram", |_| {
        cores(&visto) == 1
    });
    letra(&mut m, id, 'q');
    quadro_em(&mut m, T0);
    nunca(
        &mut m,
        id,
        T0 + PAUSA - 1,
        "perguntou antes da pausa longa",
        |_| cores(&visto) == 1,
    );
    ate(
        &mut m,
        id,
        T0 + PAUSA,
        "não perguntou depois da pausa",
        |_| cores(&visto) == 2,
    );
}

/// ⭐ As cores seguem as edições: uma linha nova antes leva-as junto (na mesma tecla, sem perguntar);
/// escrever colado a um nome larga a cor dele (o nome mudou).
#[test]
fn as_cores_seguem_as_edicoes_e_largam_o_nome_tocado() {
    let texto = "local x = 1\nph2d.set(x, \"y\", 1)\n";
    let (mut m, id, visto) = mesa(texto, Duration::ZERO);
    let ph2d = texto.find("ph2d").unwrap();
    quadro_em(&mut m, T0);
    ate(&mut m, id, T0, "as cores não chegaram", |m| {
        m.docs().ink_painted(id, ph2d) == Ink::Builtin
    });
    // Ctrl+Home e uma letra no início: tudo anda um byte.
    tecla(&mut m, id, Key::Home, crate::ctrl(), None, "");
    letra(&mut m, id, 'z');
    quadro_em(&mut m, T0);
    m.pinta(id);
    assert_eq!(
        tinta(&mut m, id, ph2d + 1),
        Ink::Builtin,
        "a cor não seguiu o nome"
    );
    assert_ne!(
        tinta(&mut m, id, ph2d),
        Ink::Builtin,
        "a cor ficou no sítio velho"
    );
    // Colado ao fim de `ph2d`: o nome passa a `ph2dq` e a cor da biblioteca sai na mesma tecla; a do
    // `set`, depois do ponto, anda e fica.
    tecla(&mut m, id, Key::Down, Modifiers::default(), None, "");
    tecla(&mut m, id, Key::Home, Modifiers::default(), None, "");
    for _ in 0..4 {
        tecla(&mut m, id, Key::Right, Modifiers::default(), None, "");
    }
    letra(&mut m, id, 'q');
    quadro_em(&mut m, T0);
    m.pinta(id);
    let novo = ph2d + 1;
    assert_ne!(
        tinta(&mut m, id, novo),
        Ink::Builtin,
        "um nome que mudou guardou a cor"
    );
    assert_eq!(
        tinta(&mut m, id, novo + 6),
        Ink::Builtin,
        "o `set` perdeu a cor"
    );
    assert_eq!(cores(&visto), 1, "seguir as edições não pergunta");
}

/// ⭐ Uma resposta para um texto que já não há nunca pinta: o servidor lento responde à 1.ª pergunta
/// depois de uma tecla no início; as cores dela (um byte atrás) não entram.
#[test]
fn uma_resposta_velha_nunca_pinta_no_sitio_errado() {
    let texto = "local x = 1\nph2d.set(x, \"y\", 1)\n";
    let (mut m, id, visto) = mesa(texto, Duration::from_millis(60));
    let ph2d = texto.find("ph2d").unwrap();
    // O quadro que pergunta, e a tecla logo a seguir (o servidor só lê o pedido daqui a 60 ms).
    quadro_em(&mut m, T0);
    letra(&mut m, id, 'z');
    for _ in 0..50 {
        quadro_em(&mut m, T0);
        m.pinta(id);
        assert_eq!(m.docs().semantic_count(id), 0, "a resposta velha entrou");
        assert_ne!(
            tinta(&mut m, id, ph2d),
            Ink::Builtin,
            "pintou no sítio velho"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        cores(&visto),
        1,
        "a 1.ª pergunta (a da versão velha) não chegou a ir"
    );
    ate(&mut m, id, T0 + PAUSA, "a pergunta nova não chegou", |m| {
        m.docs().ink_painted(id, ph2d + 1) == Ink::Builtin
    });
}

/// Os tokens nunca fazem a tecla esperar: com o servidor a dormir 0,2 s por mensagem, 40 teclas e os
/// seus quadros (com as perguntas das cores pelo meio) correm muito abaixo disso.
#[test]
fn as_cores_nunca_seguram_a_tecla() {
    let texto = "local x = ph2d.get(1, \"x\")\n";
    let (mut m, id, _visto) = mesa(texto, Duration::from_millis(200));
    let t0 = Instant::now();
    for k in 0..40u128 {
        letra(&mut m, id, 'a');
        quadro_em(&mut m, T0 + k * PAUSA);
    }
    assert!(
        t0.elapsed() < Duration::from_millis(800),
        "40 teclas levaram {:?}",
        t0.elapsed()
    );
}
