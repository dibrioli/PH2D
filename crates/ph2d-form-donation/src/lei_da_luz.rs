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
//! # ⭐⭐⭐⭐ A escolha é um CAMPO DO OBJECTO desde 2026-09-21 — e o ambiente é o BISSECTOR
//!
//! ⛔⛔ **A secção que aqui esteve chamava-se *«Porque a escolha é uma VARIÁVEL DE AMBIENTE e não um
//! campo do documento — hoje»*, e a palavra que a datava era o `hoje`.** Ela escreveu o plano
//! inteiro desta wave: *«a escolha certa é por objecto — é o que deixa dois objectos numa cena usar
//! leis diferentes — e o que decide se ela vale um degrau de `PROJECT_SCHEMA` é o veredito do dono
//! sobre a APARÊNCIA, que ainda não existe ⇒ primeiro o interruptor que lhe dá a imagem para
//! julgar; o campo gravado vem com o “sim”»*. **O «sim» chegou** (*«Smoke OK»*, 21/09), e com ele
//! o degrau.
//!
//! ⇒ a lei vive no [`super::baked_form::BakedForm::lei`], **ao lado do `rig` e pelo mesmo
//! argumento**: os dois são factos sobre COMO estes pixels foram acesos, e um objecto que reabre
//! sem eles reabre com outra aparência — em silêncio, e sem nada na tela a dizer porquê.
//!
//! # ⚠️ O que o ambiente passou a ser: um BISSECTOR, e NUNCA o documento
//!
//! `PH2D_FORM_PBR=0` força a TINTA em **todo** objecto e `=1` força a FORMA — é o que bissecta um
//! projecto já gravado **sem lhe tocar**. Sem variável nenhuma, **cada objecto decide**.
//!
//! ⛔⛔ **Ela nunca escreve no campo**, e a alternativa é a mais cruel que este módulo podia ter:
//! o artista bissectaria uma vez e o ficheiro ficaria com a lei do bissector para sempre. Há gate
//! (`a_sobreposicao_nao_escreve_no_documento`).
//!
//! ⚠️ **E ela é GLOBAL por construção** — é o que um bissector quer ser, e é exactamente o que um
//! documento não pode ser. *A frase que aqui esteve a dizê-lo continua verdadeira; o que mudou foi
//! deixar de ser o produto e passar a ser a ferramenta.*
//!
//! # ⚠️ Lida UMA vez, e só na porta do produto
//!
//! Um `var()` por objecto ou por quadro poria o AMBIENTE dentro de um laço, e a lei desta casa é
//! clara sobre o que isso faz a um gate: *um gate que lê o ambiente mede a máquina*. Ela lê-se uma
//! vez, e quem quer medir as duas leis chama a porta que as recebe como PARÂMETRO.

/// A lei que acende os pixels de um objecto assado.
///
/// ⭐ **Ela VIAJA NO DOCUMENTO** (`BakedFormDocument::lei`, degrau `161` do `PROJECT_SCHEMA`) — ver o
/// cabeçalho do módulo para porque ela é do objecto e não do binário.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

/// ⭐⭐ **AS CHAVES DOS RÓTULOS, na ordem do [`Lei::ALL`]** — o que o painel pinta.
///
/// ⚠️ **Ela existe porque o painel NÃO pode depender desta crate** (ela puxa o `wgpu`), e sem uma
/// lista `'static` o retrato teria de alocar por quadro ou o painel teria de escrever os dois nomes
/// à mão — *a segunda ortografia da mesma lei, no sítio onde o HR-15 já reprova*.
///
/// ⛔ Ela é **derivada** do [`Lei::label_key`] em tempo de compilação, e há gate a exigir que as
/// duas concordem termo a termo: uma lei nova que se esqueça daqui fica com o chip **sem nome**.
pub const CHAVES_DOS_ROTULOS: [&str; Lei::ALL.len()] =
    [Lei::Tinta.label_key(), Lei::Forma.label_key()];

/// A variável que **bissecta**: `0` força a TINTA, `1` força a FORMA, ausente deixa cada objecto
/// decidir. ⛔ Ela nunca escreve no documento — ver o cabeçalho.
pub const ENV: &str = "PH2D_FORM_PBR";

impl Lei {
    /// **AS DUAS, na ordem em que a fileira as pinta.**
    ///
    /// ⚠️ **A POSIÇÃO é a tag do chip** (o array de ids do painel é comparado com este pela costura),
    /// logo uma lei nova entra **no fim** — reordenar aqui trocaria a lei de toda peça já gravada,
    /// porque o `serde` de um enum sem `#[serde(tag)]` escreve o DISCRIMINANTE.
    pub const ALL: [Lei; 2] = [Lei::Tinta, Lei::Forma];

    /// A posição desta lei no [`Lei::ALL`] — o que a fileira pinta como escolhido.
    #[must_use]
    pub fn index(self) -> usize {
        match self {
            Self::Tinta => 0,
            Self::Forma => 1,
        }
    }

    /// A lei de uma posição, ou `None` — a volta do [`Lei::index`].
    ///
    /// ⚠️ **A ida-e-volta é gateada nos DOIS sentidos**: sem a volta, uma lei nova podia nascer
    /// inalcançável pelo painel e nada reprovaria.
    #[must_use]
    pub fn from_index(i: usize) -> Option<Self> {
        Self::ALL.get(i).copied()
    }

    /// A chave de i18n do rótulo — o painel chama `tr()` sobre ela.
    ///
    /// ⚠️ **O texto mora na tabela de i18n e o NOME mora aqui**, que é o idioma desta casa
    /// (`PoseDeformacao::label_key`, `Fit::label`…): um rótulo escrito no painel seria a segunda
    /// resposta a *como esta lei se chama*, e o censo do HR-15 acusa-o.
    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Tinta => "panel.sculpt3d.bake_law.paint",
            Self::Forma => "panel.sculpt3d.bake_law.form",
        }
    }

    /// ⭐⭐ **A SOBREPOSIÇÃO, a partir do texto** — pura, e é por isso que ela existe à parte.
    ///
    /// Um gate sobre a [`sobreposicao`] teria de escrever numa variável de ambiente **global ao
    /// processo**, que outra corrida em paralelo lê — e a resposta dela é memoizada, logo a ordem
    /// dos testes decidiria o veredito. *Uma lei que só é alcançável pelo ambiente não é gateável.*
    ///
    /// ⛔ **Só `"0"` e `"1"` falam**, e a exactidão é deliberada: um `"true"` ou um `"yes"` aceites
    /// aqui seriam uma segunda ortografia que a próxima variável desta casa não teria, e a lei de
    /// todas as outras (`PH2D_CONTACT_DUAS_CAMADAS`, `PH2D_SKIN_GPU`…) é esta.
    ///
    /// ⛔⛔ **E o `None` é a resposta NOVA de 2026-09-21** — antes esta porta chamava-se `do_texto`,
    /// devolvia uma `Lei` e **não tinha como dizer *«não sobreponhas nada»***; enquanto a escolha
    /// era global isso estava certo, e com o campo no objecto passou a ser a coisa errada de dizer.
    #[must_use]
    pub fn sobreposicao_do_texto(v: Option<&str>) -> Option<Self> {
        match v {
            Some("0") => Some(Self::Tinta),
            Some("1") => Some(Self::Forma),
            _ => None,
        }
    }
}

/// **O que o bissector está a forçar**, lido uma vez. Ver o cabeçalho do módulo.
#[must_use]
pub fn sobreposicao() -> Option<Lei> {
    static UMA_VEZ: std::sync::OnceLock<Option<Lei>> = std::sync::OnceLock::new();
    *UMA_VEZ.get_or_init(|| Lei::sobreposicao_do_texto(std::env::var(ENV).ok().as_deref()))
}

/// ⭐⭐⭐ **A LEI QUE ACENDE ESTES PIXELS** — a do OBJECTO, a menos que alguém esteja a bissectar.
///
/// ⚠️ **Uma porta só, e é por ela que o [`super::baked_form::light`] entra.** Com a decisão escrita
/// em dois sítios, o dia em que a sobreposição ganhar um terceiro estado deixa um deles para trás —
/// e o sintoma seria *«o bissector funciona no relight e não no bake»*.
#[must_use]
pub fn efectiva(do_objecto: Lei) -> Lei {
    efectiva_com(sobreposicao(), do_objecto)
}

/// **A MESMA composição, com a sobreposição DITA** — e é ela que se pode gatear.
///
/// ⚠️ Ela existe pela razão que esta casa já escreveu para a [`Lei::sobreposicao_do_texto`]: a
/// [`sobreposicao`] lê uma variável de ambiente **global ao processo** e memoiza a resposta, logo
/// um gate sobre a [`efectiva`] mediria a MÁQUINA e a ordem dos testes. Com a sobreposição como
/// PARÂMETRO, *«o bissector ganha»* e *«sem bissector manda o objecto»* passam a ser duas
/// afirmações.
#[must_use]
pub const fn efectiva_com(sobreposta: Option<Lei>, do_objecto: Lei) -> Lei {
    match sobreposta {
        Some(l) => l,
        None => do_objecto,
    }
}

/// ⭐⭐⭐ **O OLHAR com que a lei nova chega ao ecrã** — `1,50` stops, MEDIDO contra o lado aprovado.
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
///    1,30                   175,7            -10,4
///    1,40                   181,0             -5,1
///    1,45                   183,8             -2,3
///    1,48                   185,4             -0,7
///    1,50                   186,5             +0,4     ←
///    1,52                   187,6             +1,5
///    1,55                   189,3             +3,2
///    1,60                   192,1             +6,0
///    1,70                   197,7            +11,6
///    2,10                   218,7            +32,6     (o número ANTIGO)
/// ```
///
/// ⭐⭐ **A tabela foi RE-TIRADA quando a indirecta do OpenPBR entrou (a coluna B3, 2026-09-20) e o
/// número NÃO se mexeu** — nesse dia ele era `2,10`, e a metade espelhada do céu valia `+0,4` byte
/// contra um degrau de escada de `4,2`. ⚠️ *Uma reconferência que devolve o mesmo número não prova
/// que o número está certo — prova que aquela mudança não o move.*
///
/// # ⛔⛔⛔ **Ele DESCEU de `2,10` para `1,50`, e a causa é que ele estava a fazer o trabalho de UMA CURVA**
///
/// Até 2026-09-20 esta lei escrevia os bytes **CRUS** (`v × 255`) numa ranhura cujos bytes são
/// **códigos sRGB** — e aí a peça chega ao ecrã com `v` onde a malha põe `srgb(v)`, até **`+73`
/// códigos** de diferença no meio-tom. ⚠️ **A medição que escolheu o `3,00` e depois o `2,10` estava
/// CERTA** (a peça saía mesmo mais escura: `srgb⁻¹(0,5) = 0,214`, ou seja `128` escrito como `~55`)
/// **e o diagnóstico ao lado dela não** — o que faltava era a **curva**, não o **brilho**.
///
/// ⛔ **E as duas curas não são substítutas uma da outra:** a exposição multiplica tudo (levanta o
/// meio, **queima** o alto e não salva o escuro) e a curva levanta o escuro **preservando** o alto.
/// *É por isso que o lado assado saía ao mesmo tempo estourado em cima e esmagado em baixo.*
///
/// ⭐ Com a curva no sítio ([`ph2d_form_pbr::imagem`] e o gémeo do passe de dispositivo), esta escada
/// foi **re-tirada de raiz** e o candidato bate o alvo melhor do que o antigo alguma vez bateu
/// (`+0,4` contra `+1,7`). ⚠️ **E o `2,10` fica na tabela de propósito**: ele lê hoje `+32,6`, que
/// é o tamanho do que a curva trazia e que a exposição estava a pagar.
///
/// ⚠️ **Isto é uma medição e não uma ausência de medição:** *quem move o número que tornava outro
/// correcto tem de reconferir a nota*, e a reconferência pode devolver o mesmo número — o que não
/// pode é não acontecer.
///
/// ⚠️ **A escada passa do candidato de propósito:** um mínimo na BORDA de uma varredura não é um
/// mínimo, é o fim da lista.
///
/// # ⛔ E antes disso ele já tinha DESCIDO de `3,00` para `2,10`, por causa do CÉU
///
/// O `3,00` foi calibrado quando o ambiente desta lei era **zero** — e nesse dia isso era o produto.
/// Com o céu derivado do rig ([`super::baked_form::ceu_do_rig`]) a peça recebe mais luz, e com o
/// número antigo ela **satura**. *Quem move o número que tornava outro correcto tem de reconferir a
/// nota* (§0.0) — e esta constante já o pagou **duas** vezes, pelas duas coisas que a acompanhavam
/// sem estar nela: primeiro a luz que entra, agora a curva que sai.
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
    exposure_stops: 1.5,
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

    /// ⭐⭐⭐ **UM OBJECTO NASCE COM A LEI QUE O VISOR MOSTRA — e o ambiente SOBREPÕE-SE, não escolhe.**
    ///
    /// ⚠️⚠️ **Este gate já morreu DUAS vezes, e as duas mortes estão no nome.** Ele chamou-se
    /// `a_lei_nova_shipa_desligada` (e afirmava o contrário: a premissa caiu em 21/09, com
    /// `0,055042` de desvio medido — `28×` a barra de meio código) e depois
    /// `a_lei_de_fabrica_e_a_forma_e_so_o_zero_desliga`, que perguntava a uma porta chamada
    /// `do_texto` **qual era a lei do binário**. Essa pergunta deixou de existir no dia em que a lei
    /// passou a ser do OBJECTO: hoje o texto do ambiente responde *«sobrepõe-te a toda a gente»* ou
    /// *«não te metas»*, e quem responde «qual lei» é o campo gravado.
    ///
    /// ⛔ **Só `"0"` e `"1"` falam**, e a exactidão é a mesma das outras bandeiras desta casa
    /// (`PH2D_SKIN_GPU=0`, `PH2D_ISO_ADAPT=0`, `PH2D_RETOPO_EXTRACT=0`).
    #[test]
    fn um_objecto_nasce_na_lei_da_forma_e_o_ambiente_apenas_sobrepoe() {
        assert_eq!(
            Lei::default(),
            Lei::Forma,
            "um objecto assado HOJE nasce com a lei que o visor MOSTRA — senão o que se vê não é o \
             que se assa, que é o report do dono medido em 0,055042 por canal"
        );
        assert_eq!(
            Lei::sobreposicao_do_texto(None),
            None,
            "sem variável nenhuma, quem manda é o CAMPO DO OBJECTO"
        );
        for v in ["", "2", "true", "sim", " 0", "0 "] {
            assert_eq!(
                Lei::sobreposicao_do_texto(Some(v)),
                None,
                "`{v}` não pode sobrepor-se à lei gravada"
            );
        }
        // **OS DOIS CONTROLOS**: sem eles este gate ficava verde sobre uma porta que nunca sobrepõe
        // nada, e a bissecção seria inalcançável com todos os gates verdes.
        assert_eq!(Lei::sobreposicao_do_texto(Some("0")), Some(Lei::Tinta));
        assert_eq!(Lei::sobreposicao_do_texto(Some("1")), Some(Lei::Forma));

        // ⭐ **E a porta que o produto chama compõe as duas metades**: sem sobreposição ela é a
        // identidade sobre o campo — as DUAS leis, senão ela podia devolver `Forma` sempre.
        for lei in Lei::ALL {
            assert_eq!(
                super::efectiva(lei),
                lei,
                "sem bissector no ambiente, `efectiva` tem de devolver a lei do objecto"
            );
        }
    }

    /// ⛔⛔ **A SOBREPOSIÇÃO NUNCA ESCREVE NO DOCUMENTO** — o gate que o cabeçalho promete.
    ///
    /// # O defeito que ele impede, e porque ele é a pior forma desta família
    ///
    /// O bissector é GLOBAL por construção (é o que um bissector quer ser). Se alguém o gravasse —
    /// por exemplo copiando a lei EFECTIVA para o campo ao acender —, o artista bissectaria **uma**
    /// vez e o ficheiro ficaria com a lei do bissector **para sempre**, em toda peça da cena. ⚠️ E
    /// isso não se vê: as duas leis acendem, a imagem é plausível, e o defeito só aparece no dia em
    /// que ele tirar a variável do ambiente e a arte não voltar.
    ///
    /// ⚠️ **A régua é o TEXTO, e o âmbito é o que ela consegue provar:** esta crate é a dona da lei
    /// e do único caminho que a lê (`light`), logo ela pode afirmar duas coisas — que o ambiente
    /// entra por **uma porta só** e que **nada aqui dentro escreve o campo**. Quem escreve o campo
    /// são o gesto de assar e o chip do painel, e os dois vivem noutras crates, com gates próprios.
    ///
    /// **Mutação que deve sangrar:** pôr `bake.lei = efectiva(bake.lei);` dentro do `light`.
    #[test]
    fn a_sobreposicao_nao_escreve_no_documento() {
        let lei = include_str!("lei_da_luz.rs");
        let bake = include_str!("baked_form.rs");

        // (1) **UMA PORTA**: o ambiente é lido pela `sobreposicao`, e ela é consumida pela
        // `efectiva` e por mais ninguém. ⚠️ Conta-se no CÓDIGO e não nos docs — uma citação em
        // prosa não é um chamador —, e por isso as linhas de comentário saem primeiro.
        // ⚠️ **O PRODUTO acaba no `#[cfg(test)]`**, e este ficheiro tem os dois: sem o corte, o
        // controlo do próprio gate (que lê a `super::sobreposicao()`) contaria como um consumidor e
        // a régua mediria a si mesma. ⛔ E a linha da DEFINIÇÃO não é um chamador.
        let produto = &lei[..lei
            .find("#[cfg(test)]")
            .expect("controlo: o módulo de teste existe")];
        let chamadas = produto
            .lines()
            .filter(|l| {
                let t = l.trim_start();
                !t.starts_with("//") && !t.starts_with("pub fn sobreposicao()")
            })
            .filter(|l| l.contains("sobreposicao()"))
            .count();
        assert_eq!(
            chamadas, 1,
            "a sobreposição tem de ter UM leitor (a `efectiva`) — com dois, o dia em que ela ganhar \
             um terceiro estado deixa um deles para trás, e o sintoma é «o bissector funciona no \
             relight e não no bake»"
        );

        // (2) **NADA AQUI ESCREVE O CAMPO.** A crate lê `bake.lei` e devolve leis; escrever seria
        // gravar o bissector no documento.
        // ⚠️⚠️ **A agulha é MONTADA e o âmbito é o PRODUTO**, e as duas metades foram escritas por
        // um gate vermelho: um censo textual que CONTÉM a própria agulha encontra-se sempre a si
        // mesmo (a lição que a vassoura da parede já paga com base64), e a linha da mutação escrita
        // no doc deste teste é exactamente essa armadilha.
        let agulha = concat!(".lei", " =");
        for (nome, fonte) in [("lei_da_luz.rs", lei), ("baked_form.rs", bake)] {
            let corpo = fonte.find("#[cfg(test)]").map_or(fonte, |i| &fonte[..i]);
            let sem_prosa: String = corpo
                .lines()
                .filter(|l| !l.trim_start().starts_with("//"))
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                !sem_prosa.contains(agulha),
                "`{nome}` escreve no campo da lei — quem o escreve é o gesto de assar (que a \
                 PRESERVA) e o chip do painel (que é o artista a escolher), nunca o bissector"
            );
        }

        // ⭐ **O CONTROLO da régua**: ela tem de poder dizer NÃO. Sem esta metade, um `contains`
        // sobre um ficheiro renomeado passaria por vácuo.
        assert!(
            bake.contains("crate::lei_da_luz::efectiva(bake.lei)"),
            "controlo: a porta do produto TEM de ler o campo — senão as asserções acima são sobre \
             um ficheiro onde a lei já não vive"
        );
    }

    /// ⭐ **AS CHAVES DERIVADAS CONCORDAM COM A LEI** — senão uma lei nova entra na lista e o chip
    /// dela pinta o nome de outra.
    #[test]
    fn as_chaves_dos_rotulos_sao_as_da_lei() {
        assert_eq!(super::CHAVES_DOS_ROTULOS.len(), Lei::ALL.len());
        for (i, lei) in Lei::ALL.iter().enumerate() {
            assert_eq!(super::CHAVES_DOS_ROTULOS[i], lei.label_key());
        }
    }

    /// ⭐⭐ **A COMPOSIÇÃO: o bissector GANHA, e sem ele manda o OBJECTO.**
    ///
    /// ⚠️ **As duas metades, porque as mutações são opostas:** uma `efectiva` que devolvesse sempre
    /// o objecto deixaria o bissector morto (e ele é a única forma de comparar as duas leis num
    /// projecto já gravado); uma que devolvesse sempre a sobreposta ignoraria o campo gravado, que
    /// é o defeito que o degrau `161` existe para curar.
    #[test]
    fn o_bissector_ganha_e_sem_ele_manda_o_objecto() {
        for objecto in Lei::ALL {
            assert_eq!(
                super::efectiva_com(None, objecto),
                objecto,
                "sem bissector, quem manda é a lei GRAVADA do objecto"
            );
            for forcada in Lei::ALL {
                assert_eq!(
                    super::efectiva_com(Some(forcada), objecto),
                    forcada,
                    "com bissector, ele ganha — é isso que bissecta um projecto sem lhe tocar"
                );
            }
        }
        // ⭐ **O CONTROLO**: as duas leis TÊM de ser distinguíveis, senão as igualdades acima são
        // entre valores iguais e o gate afirma nada.
        assert_ne!(Lei::Tinta, Lei::Forma);
    }

    /// ⭐⭐ **A IDA-E-VOLTA DO CHIP, nos dois sentidos** — sem a volta, uma lei nova podia nascer
    /// inalcançável pelo painel e nada reprovaria.
    ///
    /// ⚠️ **A POSIÇÃO é a tag**: o `serde` de um enum sem `tag` escreve o discriminante, logo
    /// reordenar o [`Lei::ALL`] trocaria a lei de toda peça já gravada. O gate mede as duas coisas —
    /// que a ida e a volta fecham, e que a ORDEM é a que o documento assume.
    #[test]
    fn a_ida_e_a_volta_do_chip_fecham() {
        for (i, lei) in Lei::ALL.iter().enumerate() {
            assert_eq!(lei.index(), i, "a ida");
            assert_eq!(Lei::from_index(i), Some(*lei), "a volta");
        }
        assert_eq!(
            Lei::from_index(Lei::ALL.len()),
            None,
            "controlo: fora da lista"
        );
        // ⛔ A ordem é o CONTRATO do ficheiro, e é por isso que ela se afirma à mão.
        assert_eq!(Lei::ALL, [Lei::Tinta, Lei::Forma]);
        // ⭐ E os rótulos são chaves distintas — duas leis com a mesma chave dariam dois chips com o
        // mesmo nome, que é um selector que não selecciona.
        assert_ne!(Lei::Tinta.label_key(), Lei::Forma.label_key());
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
            super::sobreposicao().is_none() || std::env::var(ENV).is_ok(),
            "controlo: sem a variável no ambiente, ninguém se sobrepõe ao campo do objecto"
        );
    }
}
