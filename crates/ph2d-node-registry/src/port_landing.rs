//! **ONDE UM FIO ATERRA NUM NÓ** — irmão cortado do [`super`] no tecto de LOC (700) e por
//! RESPONSABILIDADE: aquele é o CONTÉM (as tabelas de side-metadata, o registo, os
//! acessores) e este é uma LEI sobre elas — *dada a metadata, para onde vai este fio?*
//! Crescem por razões diferentes: aquele quando um canal novo nasce, este quando um gesto
//! novo pergunta.

/// ⭐⭐⭐ **ONDE UM FIO ATERRA NUM NÓ** — a porta **PRINCIPAL** se ela servir, senão a primeira
/// que sirva. A lei de [`NodeRegistry::primary_input`] com um chamador de cada gesto.
///
/// ⛔⛔⛔ **Ela é uma FUNÇÃO e não três `if`s porque já foi escrita três vezes e mordeu duas.**
/// O report do Enio de 2026-09-01 (*«automaticamente o fio do emitter entra no input errado do
/// duplicator»*) curou **um** sítio — o splice —, e o de 2026-09-19 (*«quando puxo um fio e
/// escolho Duplicator, o grid em vez de se conectar em points, está se conectando no slot de
/// Shape»*) apanhou os **outros dois**: o `smart_connect` da paleta e o fio largado sobre o CORPO
/// de um cartão. *Uma lei escrita em dois sítios ainda não é uma lei — só uma PORTA é.*
///
/// ⚠️⚠️ **Nem o tipo nem o `validate` podem acusar isto:** as duas entradas do
/// `motion.duplicator` são `INST_VEC2`, logo as duas aceitam o fio e o grafo fica **válido** com
/// o significado trocado — as posições entram no lado da APARÊNCIA e o produto cartesiano explode
/// pelo lado errado. É por isso que a desambiguação tem de ser **declarada** (side-metadata no
/// registo, nunca o contrato congelado) e não derivada.
///
/// `serve` é o predicado do chamador — *«esta porta aceita este fio?»* —, e cada gesto tem o seu:
/// o splice pergunta pelo TIPO, o corpo do cartão pergunta pelo tipo **e** por estar LIVRE.
/// ⚠️ A principal **não é imposta**: se ela não servir, um fio que caberia noutra porta continua
/// a cair lá — *uma preferência que recusa em vez de ceder transforma um acerto num bloqueio*.
#[must_use]
pub fn landing_port(
    primary: u16,
    inputs: usize,
    mut serve: impl FnMut(u16) -> bool,
) -> Option<u16> {
    let n = u16::try_from(inputs).unwrap_or(u16::MAX);
    if primary < n && serve(primary) {
        return Some(primary);
    }
    (0..n).find(|&i| serve(i))
}

#[cfg(test)]
mod tests {
    use super::landing_port;

    /// ⭐ **A PRINCIPAL ganha quando serve** — o report inteiro num caso: duas portas do mesmo
    /// tipo, as duas livres, e o fio tem de ir à `1`.
    #[test]
    fn a_principal_ganha_quando_serve() {
        assert_eq!(landing_port(1, 2, |_| true), Some(1));
    }

    /// ⚠️ **E CEDE quando não serve** — senão uma preferência vira um bloqueio.
    #[test]
    fn a_principal_cede_quando_nao_serve() {
        assert_eq!(landing_port(1, 2, |i| i == 0), Some(0));
    }

    /// ⚠️ **Ausente (`0`) é o comportamento de sempre**, e um nó sem porta nenhuma devolve
    /// `None` em vez de uma porta inventada.
    #[test]
    fn sem_declaracao_e_o_de_sempre_e_sem_portas_e_none() {
        assert_eq!(landing_port(0, 3, |i| i >= 1), Some(1));
        assert_eq!(landing_port(0, 0, |_| true), None);
        // Uma principal FORA do alcance não pode ser escolhida (um manifesto que encolheu).
        assert_eq!(landing_port(9, 2, |_| true), Some(0));
    }
}
