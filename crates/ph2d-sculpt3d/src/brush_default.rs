//! ⭐ **COM QUE NÚMEROS UM PINCEL NASCE** — o [`Default`] do [`Brush`], inteiro.
//!
//! Irmão (`#[path]`) do [`super::brush`], e o corte é de RESPONSABILIDADE: lá
//! moram **quais knobs existem** (a struct, com o significado e a faixa de cada
//! um), aqui **que valor cada um tem de fábrica** — a lista que cresce uma
//! linha a cada pincel novo e cujos comentários carregam a proveniência MEDIDA
//! de cada número.
//!
//! ⚠️ **O corte foi FORÇADO pelo tecto de LOC** (o ficheiro-mãe chegou a `693`
//! de `700` e o modo do pincel de esfregar não cabia) e é melhor por isso: um
//! campo novo e o valor de fábrica dele deixam de crescer no MESMO ficheiro,
//! que é o que fazia dois assuntos partilharem um tecto.
//! ⛔ *Subir o número em vez de cortar é o que o `CLAUDE.md` §2 proíbe por
//! escrito* — e uma struct não se parte em dois ficheiros, então a única linha
//! de corte honesta aqui é esta.
//!
//! ⚠️ **Nenhum número muda.** O bloco foi movido inteiro, e a prova são os
//! censos de defaults que já existiam — entre eles o
//! [`super::verb::defaults::tests::the_factory_strength_is_the_table_and_nothing_else`]
//! e o [`super::verb::defaults::tests::the_accumulate_delegation_changed_nothing`].
//!
//! ⚠️⚠️ **E esta linha já foi uma MENTIRA:** a 1.ª redacção citava aqui um gate
//! `the_factory_brush_is_the_verb_it_declares` que **nunca existiu**, e quem a
//! apanhou foi o censo `every_gate_the_sculpt_family_names_exists`, escrito em
//! 13/09 depois de esta família ter oito citações dessas. *Uma promessa de gate
//! lê-se exactamente como um gate, e a diferença só aparece no dia em que ele
//! devia sangrar.*

use super::*;

impl Default for Brush {
    fn default() -> Self {
        Self {
            verb: Verb::Draw,
            mode: crate::RefMode::S,
            // ⚠️ **DERIVADO do verbo, nunca um literal ao lado dele.** O
            // `Brush.js:16` da referência ship `_accumulate = true`, e a tool
            // `Brush` dele é a nossa Draw+Clay — nós shipávamos os dois
            // desarmados, que é o *"com accumulate checado por padrão"* do
            // pedido original. Escrever `true` aqui poria o MESMO fato em dois
            // lugares, e o dia em que a tabela do verbo mudasse este literal
            // ficaria a contradizê-la em silêncio.
            accumulate: Verb::Draw.default_accumulate(),
            // ⚠️ **E a CURVA delega pela mesma razão, que o comentário logo
            // acima já enunciava e que este literal contradizia** (corrigido em
            // 2026-08-12, plano 21 W0). Ele dizia `Falloff::Smooth`, uma curva
            // que **nenhuma referência declara** — o D1 do
            // `docs/3D/20_divergencias_tools.md`, e o achado que o artista
            // encontra sem tocar em nada: a quártica da referência é `1,08× a
            // 1,44×` mais cheia ao longo do raio.
            //
            // ⚠️ **E o literal aqui não era só *um* default errado — ele
            // TRAVAVA o arming:** a lei *"arma se o artista não mexeu"* compara
            // com o que o verbo que SAI declara, então um pincel de fábrica em
            // `Smooth` contra uma tabela que diz `Plateau` parece *mexido* e
            // nunca mais seria armado por ninguém.
            falloff: Verb::Draw.default_falloff(crate::RefMode::S),
            alpha: None,
            alpha_scale: crate::DEFAULT_ALPHA_SCALE,
            // ⚠️ **O eixo nasce em +Y — as camadas saem HORIZONTAIS**, que é a
            // leitura que um estrato tem no mundo e a que o olho resolve na
            // primeira olhada. Com `az = 0` o eixo seria +X e as camadas
            // sairiam de pé; com `elev = 90` ele apontaria para a CÂMERA (a
            // vista é `+Z`) e o artista veria uma camada só, o que é
            // indistinguível de *"o padrão não funciona"*.
            alpha_az_deg: 90,
            alpha_elev_deg: 0,
            alpha_offset: [0.0, 0.0],
            alpha_stencil: None,
            // Um quarto da altura da tela por ladrilho — quatro carimbos
            // atravessando o que se vê. ⚠️ E ele **não** é semeado do modelo: um
            // estêncil não sabe o tamanho da peça, e é essa independência que o
            // artista pediu.
            alpha_stencil_scale: 0.25,
            radius: 0.25,
            strength: 0.5,
            // O meio da faixa que a referência declara trabalhável — ver o doc
            // do campo, e a medição que recusou o `0,5` dela.
            layer_height: 0.1,
            invert: false,
            plane_offset: 0.0,
            pinch: 0.5,
            // ⚠️ **PONTA QUADRADA, e o `1.0` que eu shipei era o §0 mordendo em
            // casa.** O único número citável — a redondeza de ponta `1,0` do
            // pincel GENÉRICO da referência — não é o desta ferramenta (a tabela
            // por-ferramenta vive na rotina de reset, que já não existe, §7.1). Deixei o
            // fallback definir o produto, e o
            // preço foi a ferramenta inteira: com `1` a caixa É a distância
            // euclidiana, e a faixa saía redonda. *"parece redondo"* (Enio).
            //
            // ⚠️ **A byte-identidade nunca dependeu deste número.** Quem a
            // carrega é a [`crate::Footprint::Disc`], que é a rota dos outros
            // dezasseis verbos; a faixa é nova e não tem mundo anterior a
            // preservar.
            //
            // ⚠️ **E o número é MEDIDO, declarado como NOSSO.** A propriedade que
            // separa uma faixa de um domo é o traço ter **lados paralelos** —
            // medida a largura do depósito em sete secções ao longo do caminho:
            //
            // | roundness | larguras | ponta ÷ meio |
            // |---|---|---|
            // | 0,00 | `0,8` nas sete | **1,00** |
            // | 0,25 | `0,7` nas sete | **1,00** |
            // | 1,00 | `0,5 0,5 0,6 0,6 0,6 0,5 0,5` | **0,83** |
            //
            // `0,25` dá o mesmo lado paralelo que a quina viva, o maior platô da
            // varredura (**23,7 %** dos vértices movidos contra 16,3 % em `0`) e
            // ainda arredonda a quina o bastante para ela não virar um degrau
            // numa malha grossa.
            tip_roundness: 0.25,
            // ⚠️ **Uma pegada de LADOS IGUAIS por default, e a medição diz que
            // é o certo:** a tira nasce do TRAÇO, não de um dab esticado — é a
            // quina reta que faz os lados ficarem paralelos, e o esticão é um
            // segundo eixo de estilo. O pincel genérico da referência diz `1,0`
            // e aqui ele concorda com o que a sonda mostra.
            strip_length: 1.0,
            // ⚠️ **DELEGA à constante MEDIDA**, e não repete o número: o dia em
            // que a varredura mudar de veredito, um literal aqui ficaria a
            // contradizê-la em silêncio — e o gate do V é escrito para não
            // mencionar nenhum dos dois.
            scrape_angle_deg: DEFAULT_MULTIPLANE_ANGLE_DEG,
            // O conjunto de flags zerado do pincel genérico — ver o campo.
            scrape_dynamic: false,
            // O `_hardness` de fábrica da `Masking` do original.
            mask_hardness: 0.25,
            // O neutro da etapa de dureza — ver o campo.
            hardness: 0.0,
            // ⚠️ **`0,5` é o valor de fábrica da referência, lido do cabeçalho
            // das fixtures** (`fator_raio_da_normal`), não um palpite — e as
            // duas fixtures que o movem para `0,3` são o controlo de que ele
            // chega ao resultado.
            normal_radius_frac: 0.5,
            // ⚠️ **Desligada, como na referência** — e a diferença só se vê em
            // malha grossa (ver o doc da porta).
            grab_active_vertex: false,
            pose: PoseControlos::default(),
            boundary: crate::boundary_controlos::BoundaryControlos::default(),
            // O MEIO da faixa, o mesmo ponto em que o ajuste da cena nasce: os
            // dois medem a mesma grandeza e o artista não tem porque encontrar
            // dois valores diferentes na primeira vez que olha.
            density_detail: 0.5, // LITERAL-PX-OK: fracao do curso, nao metrica de layout
            // ⚠️ **O arrasto é o de fábrica porque é o gesto que dá NOME ao
            // pincel** — *smear* é levar o relevo com a mão. Os outros dois são
            // leis próprias (adensar e espalhar) que o artista escolhe; este é
            // o que ele espera ao pegar a ferramenta.
            smear_mode: crate::SmearMode::Drag,
            // ⭐ Os três de fábrica do corpus do oráculo: raio de vista,
            // folga `0`, um sentido — a base de `21` das `24` fixturas.
            project_mode: crate::ProjectMode::View,
            trim_forma: crate::TrimForma::Caixa,
            // ⚠️ **Nasce em ZERO, e é a lei desta casa para todo knob novo:** o
            // caminho de omissão fica byte-idêntico ao traço que a mão fez.
            trim_suavizacao: 0.0,
            // ⭐⭐ **O pincel de plano nasce a APARAR**, e os dois números dizem
            // porquê: o dono pediu-o pela palavra (*«faz o trim esfregando»*), e
            // os autores do alvo registam em público que os dois tectos no
            // máximo ao mesmo tempo deformam algumas superfícies (espec §10.2).
            // ⛔ **É decisão de PRODUTO, não um número lido do alvo** — os
            // defaults por-ferramenta dele vivem num ficheiro binário, e a espec
            // §13 declara este par como o único que ela não carrega.
            plano_altura: 1.0,
            plano_profundidade: 0.0,
            // ⚠️ **`0,5`, e é facto de INTERFACE do alvo** (espec §13) — não uma
            // escolha nossa: as duas fracções de amostragem nascem ali.
            area_radius_frac: 0.5,
            // ⚠️ **AFASTAR é o de fábrica** porque é o que a família inteira
            // desta casa já faz com o `Ctrl`; a outra lei existe para o artista
            // ter *aparar* e *encher* na mesma mão.
            plano_inversao: crate::PlanoInversao::Afastar,
            project_min_distance: 0.0,
            project_bidirectional: false,
            // ⚠️ **DERIVADO do verbo, como o `accumulate` e o `falloff` logo
            // acima** — e pela mesma razão: um literal aqui seria o MESMO fato
            // em dois lugares, e no dia em que a tabela do verbo mudasse ele
            // ficaria a contradizê-la em silêncio. Ele também TRAVARIA o arming,
            // que compara com o que o verbo que sai declara.
            front_faces_only: Verb::Draw.default_front_faces_only(),
            surface_only: true,
            // O default do Blender, e o neutro deste passe — ver o campo.
            auto_smooth: 0.0,
            // ⚠️ **DELEGA, e não repete a palavra `Tri`:** a família que shipa
            // é a que a MEDIÇÃO escolheu (o resíduo de borda, em
            // [`crate::kelvinlet::Scales`]), e escrevê-la aqui poria o mesmo
            // fato em dois lugares — no dia em que a medição mudar de veredito,
            // este literal ficaria a contradizê-la em silêncio.
            elastic_scales: crate::kelvinlet::Scales::default(),
            hc_shape: crate::HC_SHAPE_DEFAULT,
            hc_vertex: crate::HC_VERTEX_DEFAULT,
            cloth_mode: crate::ClothMode::default(),
            cloth_area: crate::ClothArea::default(),
            // ⚠️ **As sete omissões são as do CÓDIGO do alvo** (espec §8.1), e
            // não as dos presets — as dos presets já governam o `cloth_area`,
            // com o número medido ao lado dele. Com estes valores a tradução
            // `Brush → Pincel` entrega exactamente o `Pincel::default()` que a
            // bancada de paridade corre ⇒ o mundo pré-wave é byte-idêntico.
            cloth_force_falloff: crate::ClothForceFalloff::default(),
            cloth_limit: 2.5,
            cloth_falloff: 0.75,
            cloth_pin: false,
            cloth_mass: 1.0,
            cloth_damping: 0.01,
            cloth_plasticity: 0.0,
            cloth_sweeps: ph2d_cloth::verlet::VARREDURAS,
            cloth_persistent: false,
            cloth_collisions: false,
        }
    }
}
