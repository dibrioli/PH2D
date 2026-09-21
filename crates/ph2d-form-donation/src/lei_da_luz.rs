//! ⭐⭐⭐ **QUAL LEI ACENDE UM OBJECTO ASSADO** — e porque hoje ela é uma PORTA e não um campo.
//!
//! A rota A (`docs/3D/02.2`) assa uma malha 3D num sprite e deixa lá os canais que a re-acendem.
//! Quem os lê, desde que a rota existe, é o passe da **TINTA** do Painter, emprestado por um
//! adaptador que neutraliza os planos dele — e esse passe é um modelo de tinta: difuso envolvido
//! mais um especular lido de uma tabela, **sem GGX e sem conservação de energia**. Ao lado, o
//! modelador acende com o OpenPBR inteiro.
//!
//! # ⭐⭐⭐⭐ Ela SHIPA LIGADA desde 2026-09-21, por ORDEM do dono e com número
//!
//! ⛔⛔ **Ela shipou DESLIGADA e a razão está aqui em baixo, intacta — o que mudou foi o veredito
//! que ela própria esperava.** O plano escrito neste módulo dizia: *«primeiro o interruptor que lhe
//! dá a imagem para julgar; o campo gravado vem com o “sim”»*. O dono julgou **três** vezes, e a
//! última sem ambiguidade: *«a malha 3d parece ter mais luz indireta que a imagem do Bake. Mas
//! precisa ser idêntica.»*
//!
//! **Medido** (`bake_light_pbr::o_visor_contra_a_sprite_que_o_produto_assa`, a mesma forma, o mesmo
//! rig, o mesmo albedo dos dois lados — o visor a mostrar a lei que assa contra a sprite que o
//! produto assava): desvio médio por canal **`0,055042`**, pior `0,159365`, contra uma barra de meio
//! código de `0,001961` ⇒ **`28×`**. *É o report dele, com número.*
//!
//! ⚠️⚠️ **E esse número já tinha sido medido uma vez e lido ao contrário.** Numa sonda anterior
//! desta linha ele apareceu como `0,055` e foi anotado como *«são duas leis diferentes»* — o que era
//! verdade e era, à letra, o defeito. *Uma explicação que dissolve a evidência é mais cara que
//! nenhuma.*
//!
//! # ⚠️ O que a razão antiga dizia, e o que sobra dela
//!
//! *«Um projecto gravado tem de continuar a abrir com a aparência com que foi gravado. O
//! `BakedForm` guarda o `rig` autorado exactamente por essa razão, e trocar a lei por baixo mudaria
//! a arte de todo objecto assado que já existe, em silêncio.»* Isso continua **verdade**, e é a
//! dívida que fica NOMEADA: a escolha certa é **por objecto e GRAVADA**, e ela custa um degrau de
//! `PROJECT_SCHEMA` (um número que SOMA entre linhas). ⛔ O que não se podia manter era o estado
//! em que *nenhuma* das duas metades do produto concorda com a outra — o dono não consegue julgar
//! uma aparência que nunca vê inteira.
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
    /// O passe do Painter — o modelo de TINTA (difuso envolvido + um especular de tabela, sem GGX
    /// e sem conservação de energia). ⛔ **Deixou de ser o valor de fábrica em 2026-09-21**: ver o
    /// cabeçalho do módulo. Alcançável por `PH2D_FORM_PBR=0`, que é o que bissecta.
    Tinta,
    /// ⭐⭐⭐ **O OpenPBR, pela [`ph2d_form_pbr`] — e é ELA que o visor mostra.** O valor de fábrica
    /// desde 2026-09-21, para que *o que se vê seja o que se assa*.
    #[default]
    Forma,
}

/// A variável que volta à lei da TINTA. ⛔ **Só o `"0"` desliga** — ausente ou qualquer outra
/// coisa ⇒ [`Lei::Forma`], que é o valor de fábrica desde 2026-09-21.
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
            Some("0") => Self::Tinta,
            _ => Self::Forma,
        }
    }
}

/// **A lei que este binário corre**, lida uma vez. Ver o cabeçalho do módulo.
#[must_use]
pub fn do_ambiente() -> Lei {
    static UMA_VEZ: std::sync::OnceLock<Lei> = std::sync::OnceLock::new();
    *UMA_VEZ.get_or_init(|| Lei::do_texto(std::env::var(ENV).ok().as_deref()))
}

/// ⭐⭐⭐ **O OLHAR com que a lei nova chega ao ecrã** — `2,10` stops, MEDIDO contra o lado aprovado.
///
/// # Porque ele não pode ser a identidade
///
/// Esta lei devolve **radiância** e a de sempre é **relativa** (ela divide pelo que uma superfície
/// plana do mesmo material devolveria, e é por isso que tinta plana sai byte-idêntica nela) ⇒ as
/// duas **não estão na mesma escala**. Escrita com a identidade, a lei nova sai visivelmente mais
/// escura.
///
/// ⛔⛔ **E isso lê-se como «a feature estragou o objecto», não como «fisicamente correcto».** Um
/// dono a quem se pede um veredito sobre a APARÊNCIA e que recebe uma peça duas vezes mais escura
/// está a julgar a exposição, não a lei.
///
/// # O número, e de onde ele vem
///
/// Da sonda `prova_da_placa::diag_a_escada_do_olhar_com_ceu`, sobre a bola CINZENTA (o controlo,
/// onde as duas leis concordam na matiz e o que resta é só o NÍVEL), média do miolo contra a da lei
/// de sempre (`186,1`):
///
/// ```text
///   stops   media do miolo cinzento   contra a tinta (186,1)
///    1,50                   133,3            -52,8
///    1,90                   170,3            -15,8
///    2,05                   183,6             -2,5
///    2,10                   187,8             +1,7     ←
///    2,15                   191,7             +5,6
///    2,50                   214,8            +28,7
///    3,00                   235,8            +49,7
/// ```
///
/// ⭐⭐ **A tabela foi RE-TIRADA quando a indirecta do OpenPBR entrou (a coluna B3, 2026-09-20) e o
/// número NÃO se mexeu.** A metade espelhada do céu soma energia — o miolo sobe `+0,4` byte em
/// `2,10` —, e isso é **um décimo** do degrau da escada (`2,05 → 2,10` vale `4,2` bytes). O `2,10`
/// continua a ser o candidato mais perto do alvo (`+1,7` contra `−2,5` do vizinho de baixo).
///
/// ⚠️ **Isto é uma medição e não uma ausência de medição:** *quem move o número que tornava outro
/// correcto tem de reconferir a nota*, e a reconferência pode devolver o mesmo número — o que não
/// pode é não acontecer.
///
/// ⚠️ **A escada passa do candidato de propósito:** um mínimo na BORDA de uma varredura não é um
/// mínimo, é o fim da lista.
///
/// # ⛔ Ele DESCEU de `3,00` para `2,10`, e a razão é o CÉU
///
/// O `3,00` foi calibrado quando o ambiente desta lei era **zero** — e nesse dia isso era o produto.
/// Com o céu derivado do rig ([`super::baked_form::ceu_do_rig`]) a peça recebe mais luz, e com o
/// número antigo ela **satura**: a mesma sonda lê `235,9` a `3,00` e `254,0` a `4,00`. *Quem move o
/// número que tornava outro correcto tem de reconferir a nota* (§0.0).
///
/// ⚠️ **A sonda mudou-se para a CPU** e deixou de viver dentro do teste de placa: o que se mede é o
/// NÍVEL que a lei entrega, e isso é a régua — não precisa de adapter. ⭐ O alvo `186,1` sobrevive à
/// correcção de sinal do `y` da fixtura, porque numa bola simétrica a média sobre a peça inteira não
/// muda ao virar a luz ao contrário.
///
/// # ⚠️ O que este número NÃO é
///
/// Ele é calibrado com o **rig de omissão** e o **OpenPBR de omissão**, e iguala o NÍVEL para que a
/// comparação entre as duas leis seja sobre a LEI e não sobre o brilho. ⛔ Ele não é uma exposição
/// «certa»: a exposição é uma escolha do artista, e o dia em que ela for um controlo este valor
/// passa a ser o ponto de partida dele — não uma constante escondida.
pub const OLHAR_DA_FORMA: ph2d_view_transform::Look = ph2d_view_transform::Look {
    exposure_stops: 2.1,
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
    openpbr_da_forma().prepare()
}

/// ⭐⭐⭐ **O MATERIAL ANTES DE PREPARADO** — a mesma porta, para quem precisa do `OpenPbr` e não do
/// [`ph2d_form_pbr::Surface`].
///
/// ⚠️ **Ela nasceu porque o VISOR passou a acender com esta lei** (o modo `Lighting::Pbr` do
/// `ph2d-mesh-render`, 2026-09-20, por ordem do dono: *«Já temos o material do módulo Modelling.
/// porque não trazer para o sculpt?»*). O `Shade` do visor guarda um `OpenPbr` e prepara-o na
/// fronteira do device; a [`material_da_forma`] entrega já preparado. ⛔ São a MESMA decisão vista
/// de dois lados, e é por isso que a segunda DELEGA na primeira em vez de escrever
/// `OpenPbr::default()` outra vez — *duas cópias divergem no primeiro dia em que alguém mexer num
/// campo, com o sintoma a ser «a sprite assa diferente do que o visor mostrava»*.
#[must_use]
pub fn openpbr_da_forma() -> ph2d_form_pbr::OpenPbr {
    ph2d_form_pbr::OpenPbr::default()
}

#[cfg(test)]
mod tests {
    use super::{ENV, Lei};

    /// ⭐⭐⭐ **A LEI DE FÁBRICA É A `Forma` — a que o visor mostra.**
    ///
    /// ⚠️⚠️ **Este gate chamava-se `a_lei_nova_shipa_desligada` e afirmava o CONTRÁRIO.** A premissa
    /// morreu em 2026-09-21 por ordem do dono (*«precisa ser idêntica»*), com `0,055042` de desvio
    /// medido entre o que ele via e o que ele assava — `28×` a barra de meio código. *Um gate cuja
    /// premissa morre é o gate a funcionar; reescrevê-lo com a morte à vista é o que impede que a
    /// razão se perca.*
    ///
    /// ⛔ **Só o `"0"` desliga**, e a exactidão é a mesma das outras bandeiras desta casa
    /// (`PH2D_SKIN_GPU=0`, `PH2D_ISO_ADAPT=0`, `PH2D_RETOPO_EXTRACT=0`): aceitar `"false"` ou
    /// `"no"` seria uma segunda ortografia que nenhuma delas tem.
    #[test]
    fn a_lei_de_fabrica_e_a_forma_e_so_o_zero_desliga() {
        assert_eq!(
            Lei::do_texto(None),
            Lei::Forma,
            "sem a variável é a lei que o visor MOSTRA"
        );
        assert_eq!(Lei::default(), Lei::Forma, "e o `Default` diz o mesmo");
        for v in ["", "1", "2", "true", "sim", " 0", "0 "] {
            assert_eq!(
                Lei::do_texto(Some(v)),
                Lei::Forma,
                "`{v}` não pode desligar a lei de fábrica"
            );
        }
        // **O CONTROLO**: o `"0"` DESLIGA — senão este gate ficaria verde sobre uma porta que nunca
        // devolve a lei da tinta, e a bissecção seria inalcançável com todos os gates verdes.
        assert_eq!(Lei::do_texto(Some("0")), Lei::Tinta);
    }

    /// ⛔⛔ **O OLHAR DA LEI NOVA NÃO É A IDENTIDADE, e isso é uma MEDIÇÃO e não um gosto.**
    ///
    /// Com `0` stops a lei nova sai a metade do nível da de sempre (média `59` contra `105`), e um
    /// dono a quem se pede um veredito sobre a APARÊNCIA e que recebe uma peça duas vezes mais
    /// escura está a julgar a exposição, não a lei. Ver o doc do [`super::OLHAR_DA_FORMA`] para a
    /// escada que escolheu o número.
    ///
    /// ⚠️ A barra é **larga de propósito** (`≥ 1` stop, ou seja *o dobro*): o valor exacto é do rig e
    /// do material de omissão, e apertá-la aqui faria este gate reprovar no dia em que alguém mudasse
    /// o rig — medindo a CENA em vez da decisão. *O que se afirma é que a identidade está
    /// descartada.*
    ///
    /// ⛔ **Ela DESCEU de `≥ 2` para `≥ 1` e isso não é afrouxar:** enquanto o ambiente desta lei foi
    /// zero, o número medido era `3,00` e a barra tinha uma folga de `1`; com o céu ele desceu para
    /// `2,10`, e uma barra a `2` passaria a estar a `5 %` do valor — *uma barra colada ao número que
    /// ela mede reprova na primeira mexida no rig, sobre produto correcto*. O número real vive no doc
    /// do [`super::OLHAR_DA_FORMA`], com a escada ao lado.
    #[test]
    fn o_olhar_da_lei_nova_nao_e_a_identidade() {
        let o = super::OLHAR_DA_FORMA;
        assert!(
            o.exposure_stops >= 1.0,
            "a lei nova é ABSOLUTA e sem exposição sai escura demais ({} stops)",
            o.exposure_stops
        );
        // **O CONTROLO**: a vista continua a de sempre — a exposição resolve o NÍVEL, e trocar a
        // vista é outra decisão, que ninguém tomou.
        assert_eq!(o.view, ph2d_view_transform::ViewTransform::Standard);
    }

    /// ⚠️ **O nome da variável é o que o roteiro do smoke escreve** — e um renome silencioso
    /// deixaria o dono a correr um comando que não desliga nada.
    ///
    /// ⚠️⚠️ **A 2.ª metade tinha a premissa INVERTIDA e ela morreu em 2026-09-21:** ela dizia
    /// *«sem a variável no ambiente, o binário corre a lei de sempre»* — verdade enquanto a lei
    /// nova shipava desligada, e falsa desde que o dono pediu que o visor e a sprite fossem
    /// idênticos. Hoje ela afirma o contrário, que é o mesmo facto do outro lado: *sem variável
    /// nenhuma, o binário corre a lei que o visor MOSTRA*.
    #[test]
    fn o_nome_da_variavel_e_o_que_o_roteiro_diz() {
        assert_eq!(ENV, "PH2D_FORM_PBR");
        assert!(
            super::do_ambiente() == Lei::Forma || std::env::var(ENV).is_ok(),
            "controlo: sem a variável no ambiente, o binário corre a lei da FORMA"
        );
    }
}
