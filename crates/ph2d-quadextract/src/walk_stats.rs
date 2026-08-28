//! ⭐ **AS RÉGUAS DO PASSEIO** — o que o traçado mede de si próprio.
//!
//! ⚠️ **Separado do [`super::walk`] por RESPONSABILIDADE:** lá vive o algoritmo (uma saída
//! de cada vez, e as leis de quem se liga a quem); aqui vive só o **relatório** dele, que
//! cresceu com cada pergunta que a investigação teve de fazer — e cada campo carrega o
//! porquê de existir. *O corte foi forçado pelo tecto de LOC; a linha do corte é a mesma
//! de sempre: quem lê isto, e porquê.*

/// O que o traçado mediu de si próprio.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct WalkStats {
    /// Saídas emparelhadas.
    pub linked: usize,
    /// ⚠️ Saídas que morreram no **bordo** — esperado numa malha aberta.
    pub boundary: usize,
    /// ⛔ Saídas que chegaram e **não** acharam a parceira no destino.
    pub orphan: usize,
    /// ⛔⛔ **A METADE das órfãs que chegou ao ponto de grade e não achou lá ninguém.**
    ///
    /// ⚠️ **A contagem única não distingue duas avarias diferentes**, e elas têm curas
    /// opostas: esta diz que o NÓ do outro lado não emitiu a cardinal de volta (leque
    /// colapsado, §6.4); a irmã diz que o traço nem conseguiu sair do triângulo.
    pub orphan_no_partner: usize,
    /// ⛔⛔ A outra metade: a [`exit_side`] não achou por onde sair do triângulo — o
    /// sintoma de uma carta **dobrada**, onde o segmento não cruza lado nenhum.
    pub orphan_no_exit: usize,
    /// ⛔ Destas, quantas tinham a ORIGEM já **fora** do triângulo em que estavam.
    ///
    /// ⚠️⚠️ **MEDIDO e sem valor de diagnóstico: é o caso NORMAL.** O `o` nunca é
    /// actualizado para o ponto de travessia — ele é a origem do segmento, transportada
    /// carta a carta —, logo depois do primeiro salto ele está **sempre** do lado de fora
    /// do triângulo em que o traço entrou. *Um contador que dá 100 % não separa nada, e a
    /// leitura de que «a origem estar fora é a avaria» estava errada.* Fica como controlo.
    pub orphan_no_exit_o_outside: usize,
    /// ⭐⭐⭐ **Das órfãs «sem parceira», quantas chegaram a um ponto que TEM nó** — e
    /// portanto ao qual só falta a cardinal de volta (leque colapsado, §6.4).
    ///
    /// ⚠️ A diferença para o total de «sem parceira» são as que chegaram a um ponto **sem
    /// nó nenhum**. *As duas leem-se igual no relatório antigo e pedem curas opostas:
    /// construir o nó, ou fazê-lo emitir a saída.*
    pub orphan_no_partner_node_exists: usize,
    /// ⭐⭐⭐ **Das órfãs «sem parceira», quantas caíram sobre uma ARESTA do triângulo.**
    ///
    /// ⚠️ Um nó de aresta nasce **uma vez por aresta**, no lado canónico, e fica registado
    /// com a face desse lado. *Quem chega pela outra face procura com a chave dele e não
    /// acha — o nó existe, a chave é que é de outra pessoa.*
    pub orphan_no_partner_on_edge: usize,
    /// ⭐⭐⭐ **Quantas órfãs o RESGATE salvou** — o nó existia na face gémea.
    ///
    /// ⚠️ **Zero aqui não é bom nem mau sozinho:** ele só quer dizer que nenhuma órfã caiu
    /// numa aresta nesta peça. *A régua é este número contra
    /// [`WalkStats::orphan_no_partner_on_edge`], que é quantas ficaram por salvar.*
    pub orphan_rescued_across_edge: usize,
    /// ⛔ Por que o resgate pela gémea **não** disparou: o alvo não está sobre aresta …
    pub rescue_no_side: usize,
    /// … está, mas a aresta não tem gémea (é bordo).
    pub rescue_no_twin: usize,
    /// … tem gémea, e a chave transportada **não existe** lá.
    pub rescue_no_key: usize,
    /// … dessas, quantas têm ALGUMA porta no mesmo ponto, com **outra direcção**.
    pub rescue_wrong_dir: usize,
    /// … a chave existe e é a **própria** porta.
    pub rescue_self: usize,
    /// ⭐⭐⭐ Quantas seriam resgatadas por cada convenção de direcção:
    /// `[x.dir(dir), oposta, com a troca do sinal da área, oposta dessa]`.
    /// ⚠️ *O índice `3` é o que o código usa hoje — e ele conta `0` por construção aqui,
    /// porque este ramo só corre quando ele falhou.*
    pub rescue_would: [usize; 4],
    /// ⭐⭐⭐ Qual convenção acertou, cruzada com **que faces estão dobradas** —
    /// índice `(face dobrada)·4 + (gémea dobrada)·2 + (0 = `d2`, 1 = `opposite(d2)`)`.
    pub rescue_by_fold: [usize; 8],
    /// … e quantas tinham candidata nas **duas** convenções (aí a contagem acima conta a
    /// oposta, e o desempate não pode vir daqui).
    pub rescue_ambiguous: usize,
    /// ⭐ **Das órfãs «sem parceira», quantas caíram num CANTO do triângulo.**
    ///
    /// ⚠️ Um canto é um nó de **vértice**, registado com a face canónica do leque — um
    /// terceiro dono possível, que o resgate por um lado só não alcança.
    pub orphan_on_corner: usize,
    /// ⭐⭐⭐ **Quantas órfãs o resgate pelo LEQUE salvou** — o nó era de vértice.
    pub orphan_rescued_in_fan: usize,
    /// ⭐ **O DIÂMETRO do triângulo em que a órfã morreu**, em células — a régua com que
    /// a linha de baixo se lê.
    ///
    /// ⛔⛔ **Sem ela a distância é ambígua e a 1.ª leitura foi errada:** `3,0` células de
    /// distância ao vértice mais próximo é enorme num triângulo de `0,2` células e é
    /// *estar quase lá* num de `6`. *Uma distância sem a escala do objecto a que se mede
    /// não é uma medição.*
    pub orphan_tri_cells_p50: f32,
    /// ⭐⭐⭐ **A QUE DISTÂNCIA o segmento passa do triângulo, em CÉLULAS de grade.**
    ///
    /// ⚠️ **É a régua que separa as duas curas possíveis:** se o segmento falha o triângulo
    /// por uma fracção de célula, a avaria é de **fronteira** (um `<` onde devia estar um
    /// `<=`, uma travessia por um vértice); se falha por células inteiras, o transporte
    /// levou-o para outra parte da peça e a avaria é **estrutural**. *Sem esta coluna as
    /// duas leem-se igual — «não achou saída» — e a cura errada é barata de escrever.*
    pub orphan_miss_cells_p50: f32,
    /// ⛔⛔ Destas, quantas TERIAM saída se o lado de ENTRADA fosse permitido.
    ///
    /// ⚠️ **É a hipótese de que a exclusão do lado de entrada é forte de mais numa
    /// dobra**: ali o traço tem de voltar por onde veio, e a regra que impede o
    /// ping-pong impede também isso.
    pub orphan_no_exit_entry_only: usize,
    /// ⛔⛔⛔ **Destas, quantas morreram num triângulo de ÁREA ZERO no domínio.**
    ///
    /// ⚠️ **A [`contains`] devolve `false` para SEMPRE quando a área é zero** (o `s == 0`
    /// sai logo à entrada), e nenhum lado de um triângulo achatado é cruzado por um
    /// segmento — *as duas portas fecham-se ao mesmo tempo, e o traço não tem para onde
    /// ir.* Separar esta contagem é o que distingue «o mapa dobrou» de «o mapa
    /// COLAPSOU», que são avarias diferentes com curas diferentes.
    pub orphan_no_exit_flat: usize,
    /// ⭐⭐⭐ **ONDE as órfãs morrem**, em raios normalizados pelo raio mediano da peça.
    ///
    /// ⛔ O report do artista de 2026-08-25 é sobre POSIÇÃO (*«furos nas pontas»*), e a
    /// órfã é o sintoma MAIS A MONTANTE da cadeia que produz um furo: órfã ⇒ saída
    /// pendente ⇒ célula abandonada ⇒ aresta de bordo.
    pub orphan_radius_p50: f32,
    /// O `p99` do raio normalizado de toda a peça — a régua da linha de cima.
    /// ⚠️ **É o p99, NÃO o máximo** — e o rótulo do instrumento dizia *«a peça vai até»*,
    /// que se lê como o máximo. Em 2026-08-26 isso fez uma célula colapsada a `1,54×` ser
    /// lida como *«um nó FORA da peça»* quando ela está no **1% mais externo**, que é a
    /// ponta. *Uma coluna cujo nome promete outra estatística lê-se ao contrário.*
    pub piece_radius_p99: f32,
    /// ⛔ Traços que estouraram o tecto de passos.
    pub runaway: usize,
    /// ⛔⛔ **Traços que chegaram a uma parceira JÁ EMPARELHADA com outra.**
    ///
    /// ⚠️ Acontece onde duas cartas se sobrepõem: dois talos diferentes chegam ao
    /// mesmo ponto de grade pela mesma direcção. *A primeira redacção sobrescrevia a
    /// ligação da outra e deixava o par ANTIGO a apontar para uma saída que já não
    /// apontava de volta* — uma meia-ligação assimétrica, que faz a extracção de
    /// células virar à esquerda para dentro de uma célula alheia. Era daí que saíam as
    /// quatro células de TRÊS lados que o teorema proíbe.
    pub contested: usize,
    /// Passos gastos, somados — a régua do custo.
    pub steps: usize,
    /// Quantas vezes o traço atravessou uma mudança de orientação.
    pub flips: usize,
}
