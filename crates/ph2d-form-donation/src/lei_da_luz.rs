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

/// ⭐⭐⭐ **O OLHAR com que a lei nova chega ao ecrã** — `3,00` stops, MEDIDO contra o lado aprovado.
///
/// # Porque ele não pode ser a identidade
///
/// Esta lei devolve **radiância** e a de sempre é **relativa** (ela divide pelo que uma superfície
/// plana do mesmo material devolveria, e é por isso que tinta plana sai byte-idêntica nela) ⇒ as
/// duas **não estão na mesma escala**. Escrita com a identidade, a lei nova sai visivelmente mais
/// escura: medido na placa sobre a mesma peça, média `59` contra `105`.
///
/// ⛔⛔ **E isso lê-se como «a feature estragou o objecto», não como «fisicamente correcto».** Um
/// dono a quem se pede um veredito sobre a APARÊNCIA e que recebe uma peça duas vezes mais escura
/// está a julgar a exposição, não a lei.
///
/// # O número, e de onde ele vem
///
/// Da sonda [`super::baked_form::prova_da_placa`], sobre a bola CINZENTA (o controlo, onde as duas
/// leis concordam na matiz e o que resta é só o NÍVEL), média do miolo contra a da lei de sempre:
///
/// ```text
///   stops   media do miolo cinzento   contra a tinta (186,1)
///    2,00                   116,8            -69,4
///    2,75                   171,9            -14,3
///    3,00                   185,7             -0,5     ←
///    3,10                   190,4             +4,3
///    4,00                   218,6            +32,4
/// ```
///
/// ⚠️ **A escada passa do candidato de propósito:** um mínimo na BORDA de uma varredura não é um
/// mínimo, é o fim da lista.
///
/// # ⚠️ O que este número NÃO é
///
/// Ele é calibrado com o **rig de omissão** e o **OpenPBR de omissão**, e iguala o NÍVEL para que a
/// comparação entre as duas leis seja sobre a LEI e não sobre o brilho. ⛔ Ele não é uma exposição
/// «certa»: a exposição é uma escolha do artista, e o dia em que ela for um controlo este valor
/// passa a ser o ponto de partida dele — não uma constante escondida.
pub const OLHAR_DA_FORMA: ph2d_view_transform::Look = ph2d_view_transform::Look {
    exposure_stops: 3.0,
    view: ph2d_view_transform::ViewTransform::Standard,
};

/// ⭐⭐⭐ **O MATERIAL da lei nova — uma porta, dois caminhos.**
///
/// ⚠️ **Ela existe porque há agora DOIS motores a acender a mesma coisa** (a referência em CPU e o
/// passe de dispositivo), e um material escrito nos dois sítios divergiria no primeiro dia em que
/// alguém mexesse num campo — com o sintoma a ser *«a placa acende diferente da régua»*, ou seja um
/// defeito de **ponte** lido como um defeito de **paridade**. Aqui é o mesmo `Surface` para os dois,
/// e quem os compara compara só **onde a aritmética corre**.
///
/// ⏳ **É o OpenPBR de omissão, e isso está DECLARADO:** um material por objecto é a coluna B1 do
/// plano (`docs/Render3d/15`). Hoje o que varia por texel é a COR, que entra como `base_color` —
/// ver [`ph2d_form_pbr::Surface::at_base_color`], que é a porta que impede o albedo de tingir o
/// destaque especular.
///
/// ⚠️ **Pela re-exportação da folha da lei e não por uma seta própria à `ph2d-material`:** uma
/// segunda aresta para a óptica seria um segundo sítio por onde a versão dela entra.
#[must_use]
pub fn material_da_forma() -> ph2d_form_pbr::Surface {
    ph2d_form_pbr::OpenPbr::default().prepare()
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

    /// ⛔⛔ **O OLHAR DA LEI NOVA NÃO É A IDENTIDADE, e isso é uma MEDIÇÃO e não um gosto.**
    ///
    /// Com `0` stops a lei nova sai a metade do nível da de sempre (média `59` contra `105`), e um
    /// dono a quem se pede um veredito sobre a APARÊNCIA e que recebe uma peça duas vezes mais
    /// escura está a julgar a exposição, não a lei. Ver o doc do [`super::OLHAR_DA_FORMA`] para a
    /// escada que escolheu o número.
    ///
    /// ⚠️ A barra é **larga de propósito** (`≥ 2` stops): o valor exacto é do rig e do material de
    /// omissão, e apertá-la aqui faria este gate reprovar no dia em que alguém mudasse o rig —
    /// medindo a CENA em vez da decisão. *O que se afirma é que a identidade está descartada.*
    #[test]
    fn o_olhar_da_lei_nova_nao_e_a_identidade() {
        let o = super::OLHAR_DA_FORMA;
        assert!(
            o.exposure_stops >= 2.0,
            "a lei nova é ABSOLUTA e sem exposição sai escura demais ({} stops)",
            o.exposure_stops
        );
        // **O CONTROLO**: a vista continua a de sempre — a exposição resolve o NÍVEL, e trocar a
        // vista é outra decisão, que ninguém tomou.
        assert_eq!(o.view, ph2d_view_transform::ViewTransform::Standard);
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
