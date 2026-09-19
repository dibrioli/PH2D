//! ⭐ **As dimensões de uma forma** — o que ela mede, e o que se pode escrever nela.
//!
//! # Por que isto existe
//!
//! Até aqui a única coisa editável de uma primitiva era o **raio do filete**. Um modelador em que
//! não se consegue dizer *"este cilindro tem 20 de raio e 50 de altura"* não é um modelador de
//! precisão — é um de escala uniforme, que é o gesto que sobra quando não há números.
//!
//! # A divisão: o documento dá a PAREDE, a vista dá o CONFORTO
//!
//! Cada grandeza diz o que **admite** ([`Dim::span`]) — o `round` de uma caixa não pode chegar à
//! meia-extensão dela, porque a fonte encolhida deixaria de existir. Isso é do documento e não se
//! negoceia.
//!
//! ⛔ **O teto de um slider NÃO é isso.** A largura de uma caixa não tem limite nenhum: escrever um
//! aqui seria inventar um número que a física não pede — o que o [`CLAUDE.md §0`] proíbe. Quem
//! escolhe até onde o **gesto** vai é a vista, e a resposta natural é *o que cabe no enquadramento*
//! — uma dimensão maior do que o quadro é uma cujo efeito não se vê. O campo numérico continua sem
//! teto, porque digitar 1000 é uma afirmação sobre a peça e não sobre a janela.
//!
//! # ⚠️ Uma faixa tem DUAS pontas, e o piso não é sempre zero
//!
//! Foi o que faltou à primeira versão: [`Dim`] só dizia o **teto**, e o painel punha o piso em zero
//! para todas as linhas. Numa largura isso está certo (o documento recusa ≤ 0); numa **posição** é um
//! defeito com sintoma mudo — digitar `-0,5` era reescrito para `0` pelo espelho do controle, e a
//! peça ia para a origem. O smoke não o apanhou porque o número experimentado foi positivo.
//!
//! Daí o [`Span`]: cada grandeza diz a **forma** da sua faixa e de que recurso vem cada ponta, e
//! quem fecha as pontas abertas é a vista, num sítio só.
//!
//! # Meias-extensões não aparecem
//!
//! O documento guarda **meias**-extensões (é a forma que a distância assinada quer). Ninguém diz que
//! uma caixa tem «meia-largura 5»: [`dims`] devolve a largura **inteira** e [`set_dim`] volta a
//! dividir. A conversão mora aqui, num sítio, e não em cada painel que a mostre.
//!
//! [`CLAUDE.md §0`]: ../../../CLAUDE.md

/// ⭐ **Que número autorado de um nó** — a identidade de uma linha do painel.
///
/// ⚠️ Um `usize` cru serviria, com o painel a saber que «0..2 é a posição e o resto são dimensões».
/// Uma convenção implícita entre duas crates é o tipo de coisa que sobrevive até alguém acrescentar
/// uma linha no meio — e aí o controle passa a escrever noutro número, em silêncio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Param {
    /// A translação **local** do nó, por eixo (0 = X).
    ///
    /// ⚠️ **Local, e é a convenção da casa**: o Inspector dela mostra o `Transform.translation`, que
    /// é local, e o readout do gizmo 2D diz por extenso que o delta é local *"porque é isso que o
    /// Inspector mostra"*. Um painel que mostrasse mundo contradiria o número ao lado no dia em que
    /// alguém agrupasse.
    Pos(u8),
    /// Um dos três ângulos da rotação **local**, por eixo (0 = X), em **graus**.
    ///
    /// ⚠️ A pose guarda um **quaternion**; estes três são o nome canónico dele. Ver
    /// [`crate::xform::set_rotation_degree`], que é onde a lei (e o que ela recusou) está escrita.
    Rot(u8),
    /// A escala **uniforme** do nó. Ver a nota de [`crate::Xform::scale`].
    Scale,
    /// Uma dimensão da forma — a posição na lista de [`dims`].
    Dim(u16),
    /// Um número de um **modificador** — `slot` é a posição na pilha, `field` é qual dos números
    /// dele. Ver [`crate::mods`].
    ///
    /// ⚠️ **A posição, e não a natureza**: a pilha pode ter duas cascas, e uma chave por natureza
    /// não as distinguiria — escrever numa escreveria na outra.
    ///
    /// ⚠️ **E DOIS índices, não um**: uma matriz tem quantas cópias *e* que espaçamento. Um índice
    /// só obrigaria a achatar a pilha inteira numa lista de números, e aí inserir um modificador no
    /// meio renumeraria tudo o que vem depois — com um arrasto a meio a escrever noutro campo.
    Mod { slot: u16, field: u8 },
    /// ⭐⭐ **O NÍVEL DE RESOLUÇÃO de uma forma que ainda está ligada ao desenho** (W55).
    ///
    /// ⚠️ **Não é uma dimensão da FORMA, e é por isso que tem chave própria.** Um `Dim` diz o que a
    /// peça mede — largura, altura, filete —, e mexer nele muda a peça. Este número não muda a peça
    /// nenhuma: muda **com que finura o contorno desenhado é convertido** nela. As duas coisas vivem
    /// em sítios diferentes (a forma no nó, o vínculo ao lado dele) e sobrevivem a gestos
    /// diferentes — largar o vínculo apaga este número e deixa a forma intacta.
    ///
    /// O teto é [`crate::MAX_PROFILE_RESOLUTION`], e ele é medido.
    Resolution,
    /// ⭐⭐⭐ **O RAIO DA JUNÇÃO desta forma** (W98) — com que arredondamento ela se encontra com o
    /// resultado das anteriores.
    ///
    /// ⚠️ **Não é o [`Param::Dim`] do filete da forma, e a diferença é o SUJEITO.** O `Dim` do
    /// arredondamento é das arestas **dela própria** — as 12 de uma caixa, o aro de um cilindro — e
    /// existe mesmo numa peça de uma forma só. Este é do **encontro**, e só existe porque há alguma
    /// coisa antes. Uma caixa arredondada que corta com aresta viva precisa dos dois números ao
    /// mesmo tempo, e uma chave só não os saberia distinguir.
    ///
    /// ⭐ **Escrever aqui MATERIALIZA o verbo** quando a forma o estava a herdar: pedir um raio de
    /// junção próprio *é* pronunciar-se. O painel mostra isso na hora — o chip `Inherit` apaga-se e
    /// acende o verbo que ela agora tem por escrito.
    ///
    /// ⚠️ A **base** não tem esta chave: ela semeia o acumulado e não se junta a nada
    /// ([`crate::fold_verb`]).
    Joint,
    /// ⭐⭐⭐ **O SEGUNDO NÚMERO de uma junta** (W145) — a meia-largura de um sulco ou de um friso, o
    /// desequilíbrio de um chanfro.
    ///
    /// `0` é a mistura que o próprio nó **é** (o raio de junção padrão de uma operação, o mesmo que
    /// o [`Param::Dim`] `0` escreve); `1` é a do **verbo** dele (a mesma que o [`Param::Joint`]
    /// escreve).
    ///
    /// ⚠️ **Dois slots e não um, porque um grupo tem DUAS misturas:** a que ele oferece aos filhos
    /// calados e a com que ele próprio se dobra nos irmãos. Uma chave só escreveria numa das duas —
    /// e qual delas dependeria da ordem do `match`, que é a forma mais silenciosa de um controle
    /// escrever no sítio errado.
    ///
    /// ⚠️ **A linha só existe quando a mistura tem segundo número** ([`crate::Blend::second`]), e é
    /// a mesma lei da W34 que o resto do painel honra: *o painel oferece exactamente o que o gesto
    /// faz*.
    Seam(u8),
    /// ⭐⭐⭐ **UM NÚMERO DE UMA LUZ** — a força (`0`) e a cor (`1`..`3`), quando o objecto escolhido é
    /// uma lâmpada (ordem do dono, 2026-09-14: *«a luz deve virar objeto 3d como nos app 3d»*).
    ///
    /// # ⚠️ Porque uma luz entra pela MESMA porta que uma largura
    ///
    /// É a razão do [`Param::Material`], um nível acima: o painel deste módulo é **derivado** de
    /// [`Param`] ponta a ponta — a linha, a faixa, o despacho e a escrita. Uma superfície própria
    /// para a luz seria uma **segunda** máquina de rows ao lado de uma que funciona, e a que
    /// apodrece é sempre a segunda.
    ///
    /// ⚠️ **A POSIÇÃO não está aqui**, e é a decisão inteira: ela é a pose da entidade, logo entra
    /// pelo [`Param::Pos`] que já existe, e mover a luz é o **mesmo gesto** que mover uma forma.
    ///
    /// ⛔ **E a ROTAÇÃO não é oferecida**: um ponto não tem orientação, e uma linha que não muda um
    /// pixel é o controlo morto que a W34 proíbe.
    ///
    /// A tabela vive em [`ph2d_field_ecs::FieldLight::get`], que é quem a lê e a escreve.
    Light(u8),
    /// ⭐⭐⭐ **UM NÚMERO DO MATERIAL desta forma** — as `15` entradas do OpenPBR, **na ordem da
    /// nodedef**: a base (`0`..`5`), o realce (`6`..`11`), o verniz (`12`..`18`) e o brilho próprio
    /// (`19`..`22`). ⇒ a tabela vive em [`ph2d_field_ecs::FieldMaterial::get`], que é quem a lê e a
    /// escreve.
    ///
    /// # ⚠️ Porque um material é um `Param` como os outros, e não uma superfície à parte
    ///
    /// O painel deste módulo é **derivado** de [`crate::Param`] ponta a ponta — a linha, a faixa, o
    /// despacho e a escrita. Uma superfície própria para o material seria uma **segunda** máquina
    /// de rows ao lado de uma que já funciona, e a que apodrece é sempre a segunda. *Um material é
    /// o que a forma mede à luz, exactamente como uma largura é o que ela mede à régua.*
    ///
    /// ⚠️ **Ele NÃO entra no [`crate::FieldDoc`]**, e isso é a decisão: o documento é **geometria**
    /// (é ele que a marcha compila), e uma cor não muda uma distância. O material vive num
    /// componente ao lado do nó — logo o `FIELD_DOC_VERSION` não se mexe, e uma peça já gravada
    /// continua a ler-se.
    ///
    /// ⚠️ **Só uma FOLHA o tem.** É a folha que o traçado sabe nomear (`ph2d_field_eval::owners`),
    /// e um material num grupo seria um valor que nenhum pixel consegue ir buscar.
    Material(u8),
    /// ⭐⭐⭐ **UM NÚMERO DA CAMADA DE ESTILO DA CENA** (`docs/Render3d/03`, a `W8`) — e o índice é a
    /// **posição na arrumação** do [`ph2d_style::wgsl::pack`], nunca uma segunda numeração.
    ///
    /// # ⚠️ Porque ele é uma família NOVA e não um `Material`
    ///
    /// O sujeito é outro: um material é **daquela folha** e o estilo é da **CENA** — e é o dreno que
    /// decide o sujeito pela FAMÍLIA, como o [`Param::Light`] já obriga. *Um índice sem família é um
    /// sujeito por adivinhar, e a adivinha é silenciosa.*
    ///
    /// ⚠️⚠️ **E o `entity` de uma linha destas NÃO É LIDO** — o estilo não é de entidade nenhuma. O
    /// precedente é o material de um GRUPO, cuja linha já viaja com o id do primeiro alvo enquanto
    /// quem decide o alcance é o dreno. ⛔ Uma entidade *sentinela* aqui seria um id válido a apontar
    /// para o objecto errado, que é a forma de defeito que nunca parece um defeito.
    ///
    /// ⭐ **O índice ser o da ARRUMAÇÃO é o que faz a escrita ser uma linha:** desempacota, escreve
    /// a posição, empacota. Uma numeração própria seria a segunda resposta à tabela do `pack`.
    Style(u8),
}

/// Quantos números um material tem — ver [`Param::Material`].
///
/// ⚠️ **Derivado por quem o lê**, e não escrito em cada sítio: o `params_of` publica **até** esta
/// quantidade de linhas e o `set_param` recusa acima dela. Dois literais divergiriam no dia em que
/// um número novo entrasse, e o sintoma seria uma linha pintada que a escrita recusa.
///
/// ⚠️⚠️ **«até», e não «exactamente»:** desde o brilho próprio (`docs/Render3d/05` §20) a cor da
/// emissão (`6..=8`) **só é publicada** quando a luminância é maior que zero — ela multiplica-a,
/// logo abaixo disso é inerte —, e desde o verniz (§21) os quatro números dele (`10..=15`) seguem a
/// mesma lei sob o peso (`9`). *A porta de ESCRITA continua a aceitar as dezasseis posições*: quem
/// esconde é a apresentação, e um pedido guardado de um quadro atrás tem de continuar a poder
/// aterrar.
pub const MATERIAL_FIELDS: u8 = 33;

/// ⭐⭐⭐ **Quantos números uma LUZ tem** — a força (`0`) e os três canais da cor (`1`..`3`).
///
/// ⚠️ **Contado aqui e lido por toda a gente**, pela razão do [`MATERIAL_FIELDS`]: a tabela é o
/// `FieldLight::get`, e uma segunda contagem escrita à mão seria a que envelhece.
pub const LIGHT_FIELDS: u8 = 4;

impl Param {
    /// ⭐⭐⭐ **OS TRÊS CANAIS de uma cor, a partir da ÂNCORA** — `None` quando este param não abre
    /// uma cor.
    ///
    /// # ⚠️ Porque a lei mora AQUI e não em quem pinta
    ///
    /// Ela tem **três** leitores: o painel, que dobra os seguidores numa amostra só; o dreno, que
    /// escreve os três no mesmo quadro para a cor ser **um** passo de desfazer; e o gate que ata os
    /// dois. *Uma lei com três leitores escrita em três sítios é a definição do número que
    /// envelhece* — e o modo de falha é mudo: a cor da emissão aterraria na cor base, com a mesma
    /// escrita e sem erro nenhum.
    ///
    /// ⚠️ **Ela não sabe QUAIS âncoras existem**, e isso é de propósito: a lista de cores de um
    /// material é do documento (o `MATERIAL_KEYS`), e um segundo vocabulário aqui envelheceria na
    /// primeira cor nova. O que ela sabe é que **uma cor são três números consecutivos da mesma
    /// família**.
    #[must_use]
    pub fn colour_channels(self) -> Option<[Self; 3]> {
        match self {
            Self::Material(k) => Some([0, 1, 2].map(|i| Self::Material(k + i))),
            Self::Light(k) => Some([0, 1, 2].map(|i| Self::Light(k + i))),
            // ⚠️ **Vale a mesma lei — três números consecutivos da mesma família** —, e no estilo
            // ela é literalmente a arrumação: cada cor ocupa o `xyz` de um `vec4`.
            Self::Style(k) => Some([0, 1, 2].map(|i| Self::Style(k + i))),
            _ => None,
        }
    }
}

/// ⭐ **O que uma grandeza admite** — a forma da faixa, e de que recurso vem cada ponta.
///
/// ⚠️ Nenhuma variante escolhe um número por conforto: ou a ponta é do **documento** (a peça deixa
/// de existir acima dela), ou é da **representação** (um ângulo canónico não passa de meia volta),
/// ou está **aberta** e quem a fecha é a vista — que é a única a saber o que cabe no quadro.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Span {
    /// Positiva e sem parede: uma largura, um raio, uma escala. O documento recusa `≤ 0`; o teto é
    /// o alcance da **vista**.
    Positive,
    /// Positiva, com **parede** do documento. Acima de `wall` a forma degenera — um dente de
    /// engrenagem de largura zero, uma moldura sem espessura. ⛔ **O zero NÃO passa**, e é essa a
    /// diferença para a [`Span::WallFromZero`].
    Wall(f32),
    /// ⭐⭐⭐ **DO ZERO, com o tecto do DOCUMENTO e sem parede** (report do Enio, 2026-09-09:
    /// *«os sliders das joints vão de 0 a 16 quando só precisa de 0 a 1»*).
    ///
    /// # ⛔ O que ela cura
    ///
    /// A [`Span::Positive`] entrega o tecto à **vista**, e o alcance da vista é o da **cena inteira**
    /// (o raio da peça × 4, arredondado à potência de dois acima). Isso está certo para uma largura
    /// — que pode legitimamente crescer até ao enquadramento — e está **errado** para um raio de
    /// junta, que é um número **local**: numa fileira de seis peças o slider abria `0..16` para um
    /// número cujo valor útil vive abaixo de `0,1`. *Todo o curso do dedo passava a caber num pixel.*
    ///
    /// ⚠️ **`Soft` e não `Hard`**: uma mistura continua a ser uma distância com qualquer raio, então
    /// o campo numérico ao lado **não** tem tecto. O que este número fecha é o **curso do slider**.
    ///
    /// ⭐ Quem o calcula é quem conhece a peça (`ph2d_field_ecs::radius_bound`), e o valor é a
    /// **menor peça sob o nó** — o raio a partir do qual a mistura a engole.
    SoftFromZero(f32),
    /// ⭐⭐⭐ **Com parede E com o zero dentro** — a faixa dos DOIS RECUOS de uma aresta, o filete e o
    /// chanfro.
    ///
    /// # ⛔ Ela cura um defeito PRÉ-EXISTENTE, e a lei que o nomeia já estava neste arquivo
    ///
    /// O filete usava [`Span::Wall`], e o painel mapeia essa faixa para um slider que **começa em
    /// zero** — mas a porta de escrita recusa o zero, porque `Wall` promete «positiva». ⇒ o artista
    /// arredondava uma aresta e **não conseguia desarredondá-la**: o controle descia até ao fundo e
    /// o número parava logo acima dele, sem dizer porquê.
    ///
    /// ⚠️ É exactamente o que o doc da [`Span::Count`] já descreve, e por isso ela ganhou o `min`:
    /// *«uma faixa que oferece o que a porta recusa é uma affordance que mente»*. A cura ali foi um
    /// piso declarado; aqui é o zero declarado.
    ///
    /// ⭐ E para o **chanfro** isto não é conforto: zero é o estado de nascimento dele, e uma faixa
    /// que não o alcança faria um knob que só liga.
    ///
    /// ⛔ **A `Wall` fica como está**, e a distinção é o que ela protege: num dente de engrenagem ou
    /// numa espessura de moldura o zero é a forma a deixar de existir, não um estado que se pede.
    WallFromZero(f32),
    /// ⭐⭐⭐ **O PISO** — a irmã da [`Span::Wall`] do outro lado: abaixo de `floor` a forma
    /// degenera, e acima dela não há parede nenhuma.
    ///
    /// # ⛔⛔ O report que a obrigou (Enio, 2026-09-06)
    ///
    /// *«se reduzir muito o raio de Rounded Cylinder, todas as formas na tela somem»* — e o censo
    /// que ele provocou achou **a espécie que nenhuma faixa sabia dizer**: o `RoundCone` exige
    /// `|raio_baixo − raio_topo| < altura` e declarava as três linhas como [`Span::Positive`]. ⇒ o
    /// slider oferecia raios que o documento recusa, e **um nó inválido apaga a cena inteira**
    /// (o `FieldDoc` é validado como um todo).
    ///
    /// ⚠️ *Uma faixa que oferece o que a porta recusa é uma affordance que mente* — a mesma lei que
    /// a [`Span::Count`] pagou com o piso do prisma e a [`Span::WallFromZero`] com o zero do
    /// chanfro. **É a terceira vez**, e desta o preço era o ecrã em branco.
    Floor(f32),
    /// ⭐⭐⭐ **AS DUAS PONTAS SÃO DO DOCUMENTO, e a VISTA não tem voto** — a faixa de uma grandeza
    /// **adimensional**.
    ///
    /// # ⛔ Por que nem a [`Span::Floor`] nem a [`Span::Wall`] servem
    ///
    /// As duas deixam uma das pontas ao **alcance da vista**, que é um COMPRIMENTO. O expoente de
    /// uma superquadrática (W127) não é um comprimento: um slider que fosse de `1` até «a oitava de
    /// quatro raios da peça» daria faixas diferentes para a mesma forma em tamanhos diferentes, e
    /// *aproximar a câmera mudaria o alcance de um número que não tem unidade*.
    ///
    /// ⚠️ É a mesma lei da [`Span::Turn`] — *«as pontas são a própria representação»* — com a
    /// diferença de esta não ser periódica.
    Range { min: f32, max: f32 },
    /// Simétrica e sem parede nenhuma: uma **posição**. As duas pontas são o alcance da vista, e a
    /// de baixo é negativa — a origem não é um canto do mundo.
    Free,
    /// ⭐⭐⭐ **Uma posição AO LONGO DO EIXO DE UM DEFORMADOR** — a banda de uma torção ou de uma
    /// dobra. Simétrica como a [`Span::Free`], e **muito mais apertada**: o alcance útil dela nunca
    /// sai da peça, porque fora da peça a banda não tem matéria em que agir.
    ///
    /// # ⛔⛔ O report que a obrigou (Enio, 2026-08-31)
    ///
    /// *«resultados bizarros, veja um cubo fino e alto com Bend»*, com as setas em `From` e `To`.
    /// Aquela chapa tem `0,072` de espessura no eixo da dobra e a peça inteira dá um alcance de
    /// gesto de `4` — ⇒ a banda vivia em **`0,9 %` do curso do slider**, menos de um pixel numa
    /// barra de 100. *Um controlo cujo intervalo útil não chega a um pixel não oferece o que o
    /// gesto faz*, que é a lei que este módulo já tinha escrita.
    ///
    /// ⚠️ **`Free` fica como está, e a distinção é o que ela protege:** a posição de um objeto quer
    /// alcance PARA ALÉM da peça (afastar uma coisa da origem é um gesto legítimo); a borda de uma
    /// banda, não — além da peça ela é um no-op.
    ///
    /// ⏳ **O que ainda falta, dito com número:** o alcance certo é a extensão da peça **naquele
    /// eixo**, e o app só sabe calcular o **raio** dela. O raio majora a meia-extensão de qualquer
    /// eixo, então esta faixa é honesta e cobre sempre a peça — mas naquela chapa ela sobra `15×`.
    /// Fechar isso pede uma caixa alinhada aos eixos, que hoje não existe em `ph2d-field-eval`.
    Along,
    /// ⭐⭐⭐ **UMA ESCOLHA ENTRE N NOMES** — não um número que o artista tenha de decifrar.
    ///
    /// A linha carrega o **índice** escolhido (`0..n−1`) e esta faixa carrega as **chaves i18n** dos
    /// nomes, na mesma ordem. O painel pinta uma fileira de botões, e o índice do que for premido é
    /// o valor que a porta recebe.
    ///
    /// # ⛔ Por que não um número inteiro com um rótulo
    ///
    /// Uma [`Span::Count`] daria a mesma faixa e um slider: *«Axis: 1»*. ⚠️ *Um controlo que obriga
    /// o artista a decifrar o valor não oferece o que o gesto faz* — é a lei que este módulo já
    /// aplica ao [`Span::Locked`] e à faixa da banda.
    ///
    /// ⚠️ **As chaves, e nunca os rótulos** (HR-15): quem traduz é o painel.
    Choice(&'static [&'static str]),
    /// **Periódica**: um ângulo. As pontas são `±half` e são a própria **representação** — nem o
    /// documento nem a vista têm voto, e um número além delas não é recusado, é renomeado.
    Turn(f32),
    /// ⭐ **Não há faixa nenhuma agora**: a grandeza existe, tem valor, e **não é editável neste
    /// estado** — e o `&str` é a **chave i18n da RAZÃO**, que o painel pinta ao lado dela.
    ///
    /// ⚠️ É diferente de *"não aparece"*. O valor continua a ser um facto que o artista precisa de
    /// ler — e esconder a linha faria o painel saltar de tamanho a cada travessia. O que ela perde é
    /// o **controle**: quem a recebe pinta um facto, não um slider (*uma affordance que não pode ser
    /// honrada é pior do que nenhuma*).
    ///
    /// # ⛔⛔ Porque a razão viaja DENTRO dela, e não num campo ao lado
    ///
    /// Ela nasceu **muda** (2026-09-14) e ficou assim por quatro famílias de travamento. Medido no
    /// painel de 2026-09-18: um artista com `Thin Walled` ligado via quatro fileiras apagadas e
    /// **nada** que dissesse porquê — e a conclusão que ele tira é que a ferramenta está partida,
    /// não que falta ligar outra coisa. *Um controlo travado e mudo lê-se exactamente como um
    /// controlo morto* (`CLAUDE.md` §5.0), e é o report que a família do esculpir já pagou três
    /// vezes (*«não vejo efeito com density»*).
    ///
    /// ⛔ **Um `Locked` + um `reason: Option<..>` ao lado seriam DUAS respostas à mesma pergunta**, e
    /// a combinação `travado sem razão` — que é precisamente o defeito de hoje — continuaria
    /// exprimível. Aqui ela não é: *não há como travar uma linha sem dizer porquê*.
    ///
    /// ⚠️ **Uma CHAVE e nunca um rótulo** (HR-15), e a mesma convenção da [`Span::Choice`] logo
    /// acima: quem traduz é o painel. ⭐ Ela pode ser uma chave porque este documento **já** carrega
    /// o vocabulário dos params (o [`Dim::key`]) — ⛔ é o oposto do `ph2d_sculpt3d::CurvaInerte`,
    /// onde o MOTOR devolve um **enum** por não saber o vocabulário da interface. *A fronteira é de
    /// quem carrega os nomes, e este carrega.*
    ///
    /// ⚠️ **Escrita para o ARTISTA, e nomeando o que a destranca** — a lei do
    /// `shape_palette::why_not`: *«escolha um contorno fechado»* é acionável; *«profile_pick is
    /// none»* não é.
    ///
    /// Os casos de hoje são o terceiro ângulo na trava de cardan — ver
    /// [`crate::xform::rotation_axis_is_free`], que é a **mesma** porta que recusa a escrita — e as
    /// seis famílias do material (`ph2d_field_ecs::params_of`).
    Locked(&'static str),
    /// ⭐ **Simétrica, e fechada pelo DOCUMENTO**: `±max`, sem a vista ter voto.
    ///
    /// ⚠️ É a irmã da [`Span::Free`] com as pontas fechadas, e a diferença é de onde vem o número:
    /// numa posição não há limite nenhum e a vista escolhe o alcance; aqui o limite é um **facto**
    /// do documento — hoje, o custo de marcha que a inclinação paga
    /// ([`crate::mods::MAX_TAPER_SLOPE`]).
    Walls(f32),
    /// ⭐ **Uma CONTAGEM**: inteira, de `min` a `max`. Quantas cópias uma matriz tem, quantos lados
    /// um prisma tem.
    ///
    /// ⚠️ É uma faixa **própria** e não uma `Positive` disfarçada, porque três coisas mudam de uma
    /// vez: o passo do arrasto é **1** (e não um centésimo do curso), o número mostra-se **sem
    /// casas decimais** (não existe meia cópia), e o piso não é zero.
    ///
    /// ⚠️ **O `min` é um campo desde a W101**, e ele nasceu de um caso concreto: uma matriz começa
    /// em **1** (zero cópias é a peça a desaparecer, e apagar já tem botão) e um prisma começa em
    /// **3** (abaixo disso não há polígono). Com o piso fixo em `1`, o slider do prisma descia a 1,
    /// a escrita era recusada, e o controle **saltava para trás debaixo do dedo** — *uma recusa é
    /// informação, mas uma faixa que oferece o que a porta recusa é uma affordance que mente.*
    Count { min: u32, max: u32 },
    /// ⭐ **Positiva OU ZERO**, com o teto vindo da vista — a irmã da [`Span::Positive`] com o zero
    /// dentro.
    ///
    /// ⚠️ Ela existe por **uma** grandeza, e ela é a razão de ser da forma: o raio do TOPO de um
    /// [`crate::Primitive::Cone`], cujo zero **é o cone fechado**. Com `Positive` o documento recusa
    /// o zero e a forma que dá nome à primitiva fica indigitável; com `Free` o slider oferece
    /// negativo, que não quer dizer nada.
    FromZero,
}

/// Uma grandeza editável de um nó.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dim {
    /// A chave i18n do nome. ⚠️ Uma **chave**, nunca um rótulo pronto (HR-15).
    pub key: &'static str,
    /// O valor que o artista vê — já em unidades inteiras (ver o doc do módulo).
    pub value: f32,
    /// **O que ela admite**, e de onde vem cada ponta. Ver [`Span`].
    pub span: Span,
}

/// ⭐ A metade de **escrita** — ver [`dims_write`].
#[path = "dims_write.rs"]
mod dims_write;
/// ⭐ **As leis de COERÇÃO** — encostar a uma parede, a um piso, e achar as arestas na lista.
#[path = "dims_write_coerce.rs"]
mod dims_write_coerce;

/// ⭐ E os DOIS RECUOS de uma aresta, do lado da escrita — ver [`dims_write_edge`].
#[path = "dims_write_edge.rs"]
mod dims_write_edge;
/// ⭐ **A folga relativa com que uma aresta se encosta ao tecto dela** — exportada porque a
/// `ph2d-field-eval` precisa do MESMO número ao derivar o perfil do plano do chanfro (W142).
/// ⛔ *Uma segunda constante ali poria a lei em dois sítios.*
pub use dims_write_edge::ROUND_MARGIN;
/// ⭐ As arms das formas por FÓRMULA — ver [`dims_write_formula`].
#[path = "dims_write_formula.rs"]
mod dims_write_formula;

pub use crate::dims_scale::scale_primitive;
/// ⭐ E a porta que **repõe as invariantes** depois de cada escrita — ver [`dims_clamp`].
#[path = "dims_clamp.rs"]
mod dims_clamp;

pub use dims_clamp::{clamp_dims, set_dim};
pub use dims_write_edge::clamp_round;

/// ⭐ A tabela por-forma — ver [`dims_table`].
#[path = "dims_table.rs"]
mod dims_table;
/// ⭐ E a das duas CURVAS COM ESPESSURA (W136) — ver [`dims_table_curve`].
#[path = "dims_table_curve.rs"]
mod dims_table_curve;
/// ⭐ E a do FLUXOGRAMA — ver [`dims_table_flow`].
#[path = "dims_table_flow.rs"]
mod dims_table_flow;
/// ⭐ A metade das CHAPAS daquela tabela — ver [`dims_table_plates`].
#[path = "dims_table_plates.rs"]
mod dims_table_plates;
/// ⭐⭐ E a do POLÍGONO, a única forma cujo número de linhas depende de uma linha — ver
/// [`dims_table_polygon`].
#[path = "dims_table_polygon.rs"]
mod dims_table_polygon;
/// ⭐ Onde as coordenadas de um vértice começam na lista de um polígono — a [`crate::vertex_rows`]
/// lê-a daqui, e não de um número escrito à mão do outro lado.
pub(crate) use dims_table_polygon::ROWS_BEFORE_VERTICES;
/// ⭐ E a metade dos SINAIS — ver [`dims_table_signs`].
#[path = "dims_table_signs.rs"]
mod dims_table_signs;
pub use dims_table::dims;
pub(crate) use dims_table_flow::display_point_wall;
