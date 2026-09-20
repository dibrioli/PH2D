//! ⭐⭐⭐ **QUAL LEI ACENDE UM OBJECTO ASSADO** — e porque hoje ela é uma PORTA e não um campo.
//!
//! A rota A (`docs/3D/02.2`) assa uma malha 3D num sprite e deixa lá os canais que a re-acendem.
//! Quem os lê, desde que a rota existe, é o passe da **TINTA** do Painter, emprestado por um
//! adaptador que neutraliza os planos dele — e esse passe é um modelo de tinta: difuso envolvido
//! mais um especular lido de uma tabela, **sem GGX e sem conservação de energia**. Ao lado, o
//! modelador acende com o OpenPBR inteiro.
//!
//! # ⛔ Ela SHIPA DESLIGADA, e o valor de fábrica é a lei de sempre
//!
//! É a lei desta casa para tudo o que é novo, e aqui ela tem um segundo motivo, mais duro: um
//! projecto gravado tem de continuar a abrir **com a aparência com que foi gravado**. O `BakedForm`
//! guarda o `rig` autorado exactamente por essa razão, e trocar a lei por baixo mudaria a arte de
//! todo objecto assado que já existe, em silêncio.
//!
//! # ⚠️ Porque a escolha é uma VARIÁVEL DE AMBIENTE e não um campo do documento — hoje
//!
//! A escolha certa é **por objecto** (é o que deixa dois objectos numa cena usar leis diferentes, e
//! é o que um campo gravado exprime). Ela custa um degrau de `PROJECT_SCHEMA`, que é um número que
//! SOMA entre linhas — e o que decide se ele vale a pena é o veredito do dono sobre a APARÊNCIA,
//! que ainda não existe. ⇒ *primeiro o interruptor que lhe dá a imagem para julgar; o campo
//! gravado vem com o «sim».*
//!
//! ⚠️ **Enquanto ela for de ambiente, ela é GLOBAL** — com ela ligada, todo objecto assado acende
//! pela lei nova. Isso é o que um smoke quer e é o que um documento não pode ter.
//!
//! # ⚠️ Lida UMA vez, e só na porta do produto
//!
//! Um `var()` por objecto ou por quadro poria o AMBIENTE dentro de um laço, e a lei desta casa é
//! clara sobre o que isso faz a um gate: *um gate que lê o ambiente mede a máquina*. Ela lê-se uma
//! vez, e quem quer medir as duas leis chama a porta que as recebe como PARÂMETRO.

/// A lei que acende os pixels de um objecto assado.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lei {
    /// O passe do Painter — o modelo de TINTA. ⭐ O valor de fábrica, e a aparência de todo
    /// projecto que já foi gravado.
    #[default]
    Tinta,
    /// O OpenPBR, pela [`ph2d_form_pbr`]. Hoje no caminho de REFERÊNCIA (CPU) — ver o custo medido
    /// no `diag_quanto_custa_acender_um_sprite` daquela crate.
    Forma,
}

/// A variável que a liga. ⛔ Ausente ou `0` ⇒ [`Lei::Tinta`].
pub const ENV: &str = "PH2D_FORM_PBR";

impl Lei {
    /// ⭐⭐ **A LEI, a partir do texto** — pura, e é por isso que ela existe à parte.
    ///
    /// Um gate sobre a [`do_ambiente`] teria de escrever numa variável de ambiente **global ao
    /// processo**, que outra corrida em paralelo lê — e a resposta dela é memoizada, logo a ordem
    /// dos testes decidiria o veredito. *Uma lei que só é alcançável pelo ambiente não é gateável.*
    ///
    /// ⛔ **Só o `"1"` liga**, e a exactidão é deliberada: um `"true"` ou um `"yes"` aceites aqui
    /// seriam uma segunda ortografia que a próxima variável desta casa não teria, e a lei de todas
    /// as outras (`PH2D_CONTACT_DUAS_CAMADAS`, `PH2D_SKIN_GPU`…) é esta.
    #[must_use]
    pub fn do_texto(v: Option<&str>) -> Self {
        match v {
            Some("1") => Self::Forma,
            _ => Self::Tinta,
        }
    }
}

/// **A lei que este binário corre**, lida uma vez. Ver o cabeçalho do módulo.
#[must_use]
pub fn do_ambiente() -> Lei {
    static UMA_VEZ: std::sync::OnceLock<Lei> = std::sync::OnceLock::new();
    *UMA_VEZ.get_or_init(|| Lei::do_texto(std::env::var(ENV).ok().as_deref()))
}

#[cfg(test)]
mod tests {
    use super::{ENV, Lei};

    /// ⛔⛔ **ELA SHIPA DESLIGADA** — e a metade que interessa é a lista do que NÃO liga.
    ///
    /// ⚠️ O `"0"` e o `""` estão lá porque são o que alguém escreve a tentar desligá-la, e um
    /// `Some(_) => Forma` acidental deixaria os dois a LIGÁ-LA — *o oposto exacto da intenção de
    /// quem os escreveu*.
    #[test]
    fn a_lei_nova_shipa_desligada() {
        assert_eq!(
            Lei::do_texto(None),
            Lei::Tinta,
            "sem a variável é a de sempre"
        );
        assert_eq!(Lei::default(), Lei::Tinta, "e o `Default` diz o mesmo");
        for v in ["", "0", "2", "true", "sim", " 1", "1 "] {
            assert_eq!(Lei::do_texto(Some(v)), Lei::Tinta, "`{v}` não pode ligar");
        }
        // **O CONTROLO**: o `"1"` LIGA — senão este gate ficaria verde sobre uma porta que nunca
        // devolve a lei nova, e a feature seria inalcançável com todos os gates verdes.
        assert_eq!(Lei::do_texto(Some("1")), Lei::Forma);
    }

    /// ⚠️ **O nome da variável é o que o roteiro do smoke escreve** — e um renome silencioso
    /// deixaria o dono a correr um comando que não liga nada.
    #[test]
    fn o_nome_da_variavel_e_o_que_o_roteiro_diz() {
        assert_eq!(ENV, "PH2D_FORM_PBR");
        assert!(
            super::do_ambiente() == Lei::Tinta || std::env::var(ENV).is_ok(),
            "controlo: sem a variável no ambiente, o binário corre a lei de sempre"
        );
    }
}
