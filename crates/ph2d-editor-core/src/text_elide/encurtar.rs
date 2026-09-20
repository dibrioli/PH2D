//! ⭐⭐⭐ **A LEI DE ENCURTAR — *um nome perde a EXPLICAÇÃO antes de perder LETRAS*.**
//!
//! Ordem do dono, 2026-09-19: *«encurtar · balão ao passar o rato»*. Esta é a primeira metade; a
//! segunda vive no [`super::balao`].
//!
//! ⚠️ **Irmão por RESPONSABILIDADE do [`super`], forçado pelo tecto de 700 LOC** (2026-09-19):
//! aquele ficheiro responde *o que fazer a um texto que não cabe* — cortá-lo com reticências;
//! este responde *o que um NOME pode largar antes de chegar a esse ponto*. ⛔ Os dois crescem por
//! motivos diferentes: o primeiro por defeitos de medição de texto, o segundo por ordens de
//! produto sobre a forma dos rótulos.

use super::{coube, elide, elisao};
use crate::text_elide::balao;
use ph2d_text::{FontWeight, TextSystem};

/// ⭐⭐⭐ **O NOME SEM A EXPLICAÇÃO** — a metade «encurtar» da ordem do dono (2026-09-19).
///
/// Metade dos rótulos cortados do Inspector carrega a explicação do valor neutro DENTRO do nome:
/// *«Acceleration (0 = instant)»*, *«Max Alive (0 = no limit)»*, *«Lifetime (s, 0 = forever)»*.
/// Numa coluna de `90 px` isso sai **`"Acceleration…"`** — o artista perde a explicação **e** o
/// fim do nome, e fica com destroços.
///
/// ⭐ **A lei é *um nome perde a EXPLICAÇÃO antes de perder LETRAS*.** Ela devolve o que fica ao
/// tirar um grupo `(…)` colado ao fim; `None` quando não há nenhum, e aí nada muda — o caminho
/// sem parênteses é **byte-idêntico**.
///
/// ⛔ **Ela é do NOME de uma linha de formulário e de mais nada** ([`fit_do_nome`]): numa FRASE
/// («1 value(s) kept until the script loads again.») o parêntese é parte do que se diz, e cortá-lo
/// mudava o sentido. *Uma lei sobre a forma do texto tem de nomear de que texto ela é.*
///
/// ⚠️ **Ela atravessa a tradução** porque o parêntese é a mesma marca em todas as línguas que este
/// app pode falar, e porque a saída é um PREFIXO do que o tradutor escreveu — nunca uma palavra
/// inventada. ⚠️ E quando o parêntese É o nome inteiro (`"(none)"`), ela recusa: não há nome
/// debaixo dele.
#[must_use]
pub fn nome_sem_a_explicacao(texto: &str) -> Option<&str> {
    let t = texto.trim_end();
    if !t.ends_with(')') {
        return None;
    }
    let abre = t.rfind('(')?;
    let nome = t[..abre].trim_end();
    (!nome.is_empty()).then_some(nome)
}

/// ⭐⭐⭐ **O texto de um NOME de linha de formulário que cabe numa coluna** — a escada das três
/// respostas, da melhor para a pior.
///
/// 1. **O nome inteiro**, quando cabe.
/// 2. **O nome sem a explicação** ([`nome_sem_a_explicacao`]), quando ela existe e o que sobra
///    cabe **inteiro**. ⚠️ *Inteiro*: um nome encurtado que ainda precisasse de reticências seria
///    a pior das duas perdas de uma vez.
/// 3. **O corte com reticências**, como sempre.
///
/// ⚠️⚠️ **O degrau 2 continua a ser um CORTE para o censo** (`Medido::texto` é o nome inteiro),
/// e isso é deliberado: o artista continua sem ver a explicação, logo o
/// [`super::balao`] tem de a poder mostrar. O que muda — e é o que o dono vê — é que `Medido::pintado`
/// deixa de acabar em reticências. *Uma régua que contasse o degrau 2 como «coube» esconderia
/// exactamente a informação que ele custa.*
#[must_use]
#[track_caller]
pub fn fit_do_nome(
    text_system: &mut TextSystem,
    texto: &str,
    font_size: f32,
    col_w: f32,
) -> String {
    if coube(text_system, texto, font_size, col_w, FontWeight::MEDIUM) {
        return texto.to_string();
    }
    if let Some(nome) = nome_sem_a_explicacao(texto)
        && text_system.prefix_width_weighted(nome, font_size, FontWeight::MEDIUM) <= col_w
    {
        elisao::regista(texto, nome, col_w, font_size, FontWeight::MEDIUM);
        balao::corte(texto);
        return nome.to_string();
    }
    elide(text_system, texto, font_size, col_w, FontWeight::MEDIUM)
        .unwrap_or_else(|| texto.to_string())
}

/// ⭐⭐⭐ **A LEI DE ENCURTAR — *um nome perde a EXPLICACAO antes de perder LETRAS*.**
///
/// Ordem do dono, 2026-09-19: *«encurtar · balao ao passar o rato»*.
#[cfg(test)]
mod o_nome_perde_a_explicacao_antes_das_letras {
    use super::*;
    use crate::text_elide::fit;

    /// A forma que o Inspector tem dezenas de vezes.
    #[test]
    fn a_explicacao_do_valor_neutro_sai_e_o_nome_fica() {
        assert_eq!(
            nome_sem_a_explicacao("Acceleration (0 = instant)"),
            Some("Acceleration")
        );
        assert_eq!(
            nome_sem_a_explicacao("Gravity X (m/s\u{b2})"),
            Some("Gravity X")
        );
        assert_eq!(
            nome_sem_a_explicacao("Lifetime (s, 0 = forever)"),
            Some("Lifetime")
        );
    }

    /// ⛔ **Sem parenteses nao ha nada a tirar** — e e este braco que faz o caminho de omissao
    /// deste app ser byte-identico ao de antes da lei.
    #[test]
    fn um_nome_sem_explicacao_fica_intacto() {
        assert_eq!(nome_sem_a_explicacao("Air Acceleration"), None);
        assert_eq!(nome_sem_a_explicacao("Bounds X / Y / W / H"), None);
    }

    /// ⛔⛔ **Quando o parentese E o nome inteiro, nao ha nome debaixo dele.**
    ///
    /// ⚠️ Sem esta recusa a lei devolveria a string VAZIA, e o pintor desenharia **nada** — que e
    /// exactamente o defeito que a varredura vizinha existe para impedir. *Uma lei que encurta
    /// tem de ter um chao.*
    #[test]
    fn um_parentese_que_e_o_nome_inteiro_e_recusado() {
        assert_eq!(nome_sem_a_explicacao("(none)"), None);
        assert_eq!(nome_sem_a_explicacao("   (auto)"), None);
    }

    /// ⚠️ Um parentese a MEIO nao e a explicacao do fim — so o grupo colado ao fim sai.
    #[test]
    fn so_o_grupo_colado_ao_fim_sai() {
        assert_eq!(nome_sem_a_explicacao("A (b) c"), None);
        assert_eq!(nome_sem_a_explicacao("A (b) c (d)"), Some("A (b) c"));
    }

    /// ⭐⭐⭐ **A ESCADA INTEIRA, pelo produto** — e a 3.ª resposta prova que encurtar nao substitui
    /// a reticencia: num orcamento em que nem o nome cabe, ela volta.
    #[test]
    fn a_escada_tem_os_tres_degraus() {
        let mut ts = TextSystem::without_system_fonts();
        let texto = "Acceleration (0 = instant)";
        let size = 12.0;
        let inteiro = ts.prefix_width_weighted(texto, size, FontWeight::MEDIUM);
        let nome = ts.prefix_width_weighted("Acceleration", size, FontWeight::MEDIUM);
        assert!(nome < inteiro, "a explicacao tem de custar alguma coisa");

        // 1) cabe inteiro
        assert_eq!(fit_do_nome(&mut ts, texto, size, inteiro + 1.0), texto);
        // 2) cabe o nome, sem a explicacao — e SEM reticencias
        let meio = fit_do_nome(&mut ts, texto, size, nome + 1.0);
        assert_eq!(meio, "Acceleration");
        assert!(
            !meio.ends_with('\u{2026}'),
            "o degrau 2 nao pode ter reticencias"
        );
        // 3) nem o nome cabe ⇒ a reticencia volta
        let curto = fit_do_nome(&mut ts, texto, size, nome * 0.5);
        assert!(
            curto.ends_with('\u{2026}'),
            "num orcamento abaixo do nome a reticencia tem de voltar, e veio {curto:?}"
        );
    }

    /// ⭐⭐ **O CONTROLO: sem parenteses, a lei e o [`fit`] AO BIT.**
    ///
    /// ⚠️ Sem esta metade, uma lei que encurtasse por outra regra qualquer (cortar a ultima
    /// palavra, por exemplo) passaria os testes de cima e mudava **toda** a tela.
    #[test]
    fn sem_parenteses_e_o_fit_de_sempre() {
        let mut ts = TextSystem::without_system_fonts();
        for texto in ["Air Acceleration", "Crouch Height", "Bounds X / Y / W / H"] {
            for orcamento in [8.0_f32, 20.0, 45.0, 78.0, 300.0] {
                assert_eq!(
                    fit_do_nome(&mut ts, texto, 12.0, orcamento),
                    fit(&mut ts, texto, 12.0, orcamento),
                    "{texto:?} a {orcamento} px"
                );
            }
        }
    }

    /// ⛔⛔ **O degrau 2 continua a ser um CORTE para o censo, e isso e a lei.**
    ///
    /// ⚠️ Se ele se registasse como «coube», a varredura deixaria de o contar **e o
    /// [`balao`] nao teria o que mostrar** — o artista ficaria sem a explicacao e sem maneira
    /// nenhuma de a ler. *Encurtar nao e a mesma coisa que caber.*
    #[test]
    fn encurtar_nao_e_caber_e_o_censo_sabe_a_diferenca() {
        let mut ts = TextSystem::without_system_fonts();
        let texto = "Acceleration (0 = instant)";
        let nome = ts.prefix_width_weighted("Acceleration", 12.0, FontWeight::MEDIUM);
        let (_, medidos) = elisao::medindo(|| {
            let mut ts2 = TextSystem::without_system_fonts();
            fit_do_nome(&mut ts2, texto, 12.0, nome + 1.0)
        });
        let m = medidos
            .iter()
            .find(|m| m.texto == texto)
            .expect("o censo tem de ver o nome INTEIRO");
        assert!(!m.coube(), "o degrau 2 e um corte");
        assert_eq!(m.pintado, "Acceleration");
    }
}
