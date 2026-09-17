//! **COM QUE MALHA CADA CENA ABRE** — irmão (`#[path]`) do
//! [`super::scenes`], cortado por ASSUNTO.
//!
//! Lá vive *qual cena está armada*; aqui vive *o que ela põe na tela*. As duas
//! perguntas são respondidas por ficheiros diferentes desde 2026-09-07, quando a
//! cena `=37` cruzou o teto de LOC do roteador — e o corte é o mesmo que a casa
//! faz sempre: **por responsabilidade, nunca por tamanho**.

use super::*;

/// ⭐⭐⭐ **A PEÇA DAS DUAS CENAS DE MULTIRRESOLUÇÃO** (`=43`, o apagador, e
/// `=44`, o esfregão) — e ela abre GROSSA, com o número do report do dono
/// (*«meio travado, até na hora de rotacionar o canvas dá uma travadinha»*,
/// 2026-09-14).
///
/// ⛔⛔ **O roteiro das duas manda apertar `K` duas vezes, e sobre o default do
/// módulo (`98 306` vértices) isso fabrica uma peça de `1 572 866`.** Medido em
/// `--release`: ali um dab de **`Draw`** custa `9,7 ms` contra o *kill* de `8`
/// — *toda* ferramenta estoura o orçamento, e rodar a câmera engasga. ⇒ *a cena
/// ensinava que o pincel é lento quando quem é pesada é a peça que ela própria
/// mandou construir*, que é a espécie que o `CLAUDE.md` §5.0 chama de **pior
/// que uma cena ausente**.
///
/// ⭐ **A escada passa a ser `386 → 1 538 → 6 146`**, e o `6 146` não é um
/// número escolhido: é **a densidade das fixturas do oráculo** destes dois
/// pincéis (espec §7), ou seja o regime em que a lei deles foi medida.
///
/// ⚠️ **É o CUBO subdividido e não uma `uv_sphere`**, pela mesma razão que fez o
/// default do módulo mudar por ordem do dono em 2026-08-10: o leque de pólo de
/// uma esfera UV dá ao mesmo pincel uma superfície por dab dez vezes menor no
/// pólo que no equador — e estas cenas tocam no pólo.
pub(crate) fn peca_de_multirresolucao() -> ph2d_mesh::Mesh {
    // ⭐⭐⭐ **AS DUAS CENAS DE MULTIRRESOLUÇÃO ABREM GROSSAS, e o número é o
    // report do dono** (*«meio travado, até na hora de rotacionar o canvas
    // dá uma travadinha»*, 2026-09-14).
    //
    // ⛔⛔ **O roteiro delas manda apertar `K` duas vezes, e sobre o default
    // do módulo (`98 306` vértices) isso fabrica uma peça de `1 572 866`.**
    // Medido: ali um dab de **`Draw`** custa `9,7 ms` contra o *kill* de
    // `8` — *toda* ferramenta estoura o orçamento, e rodar a câmera
    // engasga. ⇒ *a cena ensinava que o pincel é lento quando quem é pesada
    // é a peça que ela própria mandou construir*, que é a espécie que o
    // `CLAUDE.md` §5.0 chama de pior que uma cena ausente.
    //
    // ⭐ **A escada passa a ser `386 → 1 538 → 6 146`**, e o `6 146` não é
    // um número escolhido: é **a densidade das fixturas do oráculo** destes
    // dois pincéis (espec §7), ou seja o regime em que a lei deles foi
    // medida. Ali o dab custa uma fracção de milissegundo e a câmera roda
    // limpa.
    //
    // ⚠️ **É o cubo subdividido e não uma `uv_sphere`**, pela mesma razão
    // que fez o default do módulo mudar por ordem do dono em 2026-08-10: o
    // leque de pólo de uma esfera UV dá ao mesmo pincel uma superfície por
    // dab dez vezes menor no pólo que no equador — e uma cena que abre no
    // pólo mediria isso em vez do pincel.
    let mut m = ph2d_mesh::shapes::cube(1.0);
    for _ in 0..3 {
        m = ph2d_mesh::subdivide(&m);
    }
    // A meia-extensão da caixa a `1,0`, como a `sculpt_sphere` faz — senão a
    // câmera enquadra outra peça e os raios do pincel deixam de comparar.
    let b = m.bounds();
    let meia = (0..3)
        .map(|i| (b.max[i] - b.min[i]) * 0.5)
        .fold(0.0f32, f32::max);
    if meia > 0.0 {
        let k = 1.0 / meia;
        for p in m.positions_mut() {
            *p = [p[0] * k, p[1] * k, p[2] * k];
        }
        m.rebuild();
    }
    m
}

/// A malha com que cada cena abre.
///
/// ⚠️ **Porta única, e ela existe para o gate.** A cena `=3` só significa alguma
/// coisa se a malha dela de fato reverter, e isso é um fato sobre a GEOMETRIA
/// que nenhum arch-gate de fonte enxerga. Um gate que reconstruísse a malha por
/// conta própria estaria medindo outra malha no dia em que esta mudasse.
#[must_use]
/// A malha da `=41`, para o gate que afirma que ela não é inerte.
#[cfg(test)]
pub(crate) fn smoke_mesh_for_tests_ear() -> ph2d_mesh::Mesh {
    eared_sphere()
}

pub(crate) fn smoke_mesh() -> ph2d_mesh::Mesh {
    // ⚠️ A `=8` abre com as CRISTAS pelo motivo que o `scene_objects` explica:
    // uma esfera lisa reaberta é indistinguível de uma recém-nascida, e o smoke
    // do documento pergunta exatamente *o que eu salvei é o que eu abro?*.
    // ⚠️ A `=10` abre com as CRISTAS pelo mesmo motivo da `=8`: uma esfera lisa
    // que volta de um arquivo é indistinguível de uma recém-nascida, e o que
    // este smoke pergunta é *a FORMA atravessou?*.
    // ⚠️ A `=11` abre com as CRISTAS porque o que ela julga é a LUZ: sobre uma esfera lisa a
    // iluminação de uma normal quase constante lê como um degradê chapado, e o artista não teria
    // como separar *o objeto ficou aceso pela forma* de *alguém escureceu o sprite*.
    // ⚠️ A `=15` abre com as RUGAS EM ESCADA, e a escada é o oráculo: a cavidade
    // entrega *ver o que a luz sozinha não mostra*, então a cena tem de conter
    // sulcos que a luz já mostra e sulcos que ela quase não mostra. Com uma
    // profundidade só, ligar o canal daria *uma imagem diferente* — e diferente
    // não é a pergunta.
    // ⚠️ A `=34` abre com as CRISTAS porque a sonda `measure_sharpen_law.rs`
    // mediu que a lei do Sharpen precisa de FEIÇÃO e não de ruído: com a
    // curvatura comparável em todo vértice o `f` do pré-passe fica alto em toda
    // parte, o gather é anulado por `(1 − f)` e a lei degenera num alisador.
    // Sobre uma esfera lisa é pior — sem contraste não há o que contrastar, e os
    // dois chips novos leriam como controles mortos.
    // ⚠️ A `=35` abre na MESMA malha rugosa: uma retopologia só se distingue de
    // um voxel remesh se houver FORMA para a grade seguir.
    // ⭐ **A `=36` abre na ORELHA**, e a razão é a mesma que pôs a `=35` na
    // amassada, um degrau acima: uma retopologia só se julga sobre a feição que ela
    // tem de preservar, e um vinco côncavo fundo é a que a quebra.
    if ear::ear_scene() {
        return eared_sphere();
    }
    // ⭐⭐ **A `=39` abre com DUAS PEÇAS SOLTAS na mesma malha**, e a geometria foi
    // MEDIDA (ver [`alcance`]): duas pontas LIGADAS só mostram o defeito quando
    // são finas o bastante para o carimbo tocar `9`–`39` vértices — um alfinete,
    // não um gesto —, e num raio que o artista usa o corte cai a `6,9 %`, que não
    // se vê. Com duas peças soltas ele é `47 %` sobre centenas de vértices, e a
    // pergunta passa a ser binária: *a outra peça mexeu-se, sim ou não?*
    if alcance::alcance_scene() {
        return alcance::duas_pecas_vizinhas();
    }
    // ⭐⭐ **A `=41` abre na ORELHA, e a escolha é MEDIDA e não estética:** numa
    // esfera lisa a franja que dá o pivô é um anel **simétrico** à volta do
    // cursor ⇒ a média dela cai em cima dele, o primeiro segmento nasce com
    // comprimento nulo e o pincel de pose **não move nada** (espec §11.1). Uma
    // cena de esfera mostraria a ferramenta a parecer partida. A orelha é um
    // apêndice: a franja dela é quase toda do lado do corpo, e o pivô cai na
    // BASE — que é o que faz o gesto parecer uma articulação.
    if pose::pose_scene() {
        return eared_sphere();
    }
    // ⭐⭐ **A `=42` abre numa TIGELA, e a escolha é MEDIDA pela mesma régua:**
    // o pincel de contorno só existe onde a malha **acaba**. Numa casca fechada
    // não há aresta de borda, a busca da âncora falha e o traço **não move um
    // único vértice** — medido no corpus do oráculo. A tigela é meia esfera com
    // a boca aberta: a borda é a boca, e é de lá que tudo sai.
    if boundary::boundary_scene() {
        return boundary::tigela();
    }
    // ⭐ **A `=40` abre na MESMA enrugada, e a razão é a mesma da `=34` vista de
    // outro lado:** os dois gestos tangenciais movem o barro NO PLANO da
    // superfície, e numa esfera lisa isso não muda a silhueta nem quase a luz —
    // o dono não teria como separar *funcionou* de *não fez nada*. As cristas
    // são a marca que o gesto arrasta.
    if cavity_scene()
        || filter::filter_scene()
        || quad::quad_scene()
        || tangenciais::tangenciais_scene()
    {
        return wrinkled_sphere();
    }
    // ⚠️ **A `=16` abre DENSA, e a densidade é o que o smoke julga.** O tamanho
    // de feature que um modelo comporta é a mais grossa de duas restrições
    // (`ph2d_sculpt3d::recommended_scale`): o LOOK, ~33 features atravessando a
    // peça, e a lei das dez arestas. Numa malha grossa a segunda vence e o padrão
    // sai como CRATERA — foi o que o 1º smoke desta wave reprovou, com a esfera
    // de 38 k. Medido, o LOOK só passa a mandar a partir de ~800 segmentos:
    // 96×144 recomenda 0,327 (6 features atravessando) · 160×240 dá 0,196 (10) ·
    // 320×480 dá 0,098 (20) · **533×800 dá 0,061 (33)**, que é textura. Custa
    // 35 ms para abrir e 0,555 ms por dab — medido, e sob o kill de 8.
    // ⚠️ **A `=21` abre com a MESMA malha, e a razão é a lei das dez arestas.**
    // Um estrato picado por uma malha grossa lê como chuvisco, e o artista veria
    // o eixo girar RUÍDO — que é indistinguível de o eixo não fazer nada.
    if alpha_scene() || directional_alpha_scene() {
        return ph2d_mesh::shapes::uv_sphere(533, 800, 1.0);
    }
    // ⛔ **O braço da `=17` (o toro do AO assado) saiu com a cena, em 2026-09-11** — ver a
    // nota no lugar do `ao_scene()` em `scenes.rs`. O alcance default do AO
    // (maior lado ÷ 8) continua medido como GRATUITO (custo plano de 5,6 a 6,3 ms enquanto o
    // raio cresce 6×); o que se perdeu foi a CENA que tornava essa decisão de look visível.
    if turn_scene()
        || document_scene()
        || export_scene()
        || bake_scene()
        || reopen_scene()
        || alpha_image_scene()
    {
        return ridged_sphere();
    }
    // ⚠️ **AS CENAS DE SOMBREAMENTO SÃO DONAS DO ELENCO INTEIRO**, e esta linha é a
    // cura de um defeito que a aritmética achou: sem ela, uma cena que declarava
    // as peças no `scene_objects` abria com a esfera lisa de 96×144 por cima,
    // não convidada — e na `=19` isso ENTERRAVA uma das três bolas da escada.
    if let Some(m) = shading::primary_mesh() {
        return m;
    }
    if remesh_scene() {
        return hooked_sphere();
    }
    if masked::flatten_scene() {
        return masked::half_masked_sphere();
    }
    if holes_scene() {
        return punctured_sphere();
    }
    if dyntopo_scene() {
        // ⚠️ **GROSSA de propósito, e é a metade do smoke que o número prova.**
        // A esfera de 96×144 que o resto do módulo abre já tem arestas menores
        // que o alvo de qualquer pincel razoável — ligar a topologia dinâmica
        // sobre ela não partiria nada, e a cena ficaria verde mostrando NADA.
        // Com 10×14 as facetas são visíveis a olho nu, e é contra elas que o
        // detalhe nascendo se vê.
        return ph2d_mesh::shapes::uv_sphere(10, 14, 1.0);
    }
    if crate::scenes::plano::plano_scene() {
        // ⚠️ **Com BOSSAS, e a razão é a lição da cena** — ver o cabeçalho da
        // [`crate::scenes::plano`]: numa esfera lisa este pincel pára sozinho
        // (o auto-limite), e uma cena que abre no caso degenerado ensina o
        // contrário do que a ferramenta é.
        return crate::scenes::plano::peca();
    }
    if crate::scenes::afiado::afiado_scene() {
        // ⚠️ **Lisa e DENSA, e as duas metades são a lição** — ver o cabeçalho
        // da [`crate::scenes::afiado`]: este pincel FAZ o relevo (sobre bossas o
        // vinco perder-se-ia no meio delas), e o fundo agudo dele precisa de
        // umas três arestas na largura para existir.
        return crate::scenes::afiado::peca();
    }
    if crate::scenes::erase::erase_scene() || crate::scenes::smear::smear_scene() {
        return peca_de_multirresolucao();
    }
    if reversion_scene() {
        // ⚠️ **Ela é DUAS vezes subdividida de propósito**: um modelo denso que
        // chega pronto não tem um nível embaixo, e a cena só demonstra a
        // reversão se houver mais de um para reconstruir. A esfera UV mistura
        // quads no corpo com triângulos nos polos, que é o caso que exercita os
        // dois ramos do reconhecedor de uma vez.
        let coarse = ph2d_mesh::shapes::uv_sphere(12, 18, 1.0);
        ph2d_mesh::subdivide(&ph2d_mesh::subdivide(&coarse))
    } else {
        // ⚠️ **O DEFAULT DO MÓDULO, e ele deixou de ser uma esfera UV em
        // 2026-08-10, por ordem do Enio** — *"substitua a Sphere padrão (com
        // topologia imprópria para escultura) pela Sphere do SculptGL"*. A
        // topologia é o argumento inteiro, e ele é um número: a razão entre a
        // maior e a menor aresta é **3,9×** na subdividida contra **30,6×** na
        // `uv_sphere(96, 144)`, cujo leque de polo dá ao mesmo pincel uma
        // superfície por dab dez vezes menor lá que no equador.
        //
        // ⚠️ **Ela é 7,2× mais densa** (98 306 vértices contra 13 682) e abre em
        // **14,3 ms** contra 1,2 — o mesmo custo de abertura que a cena `=16`
        // já paga com folga (35 ms), e sob o kill de dab de 8 ms.
        peca_de_fabrica()
    }
}

/// **A PEÇA COM QUE O MÓDULO ABRE** — uma porta, para que nenhuma cena a
/// escreva outra vez.
///
/// ⚠️ Ela existe desde 2026-09-17, quando a cena `=46` deixou de abrir com uma
/// peça própria: *uma cena que escolhe a sua peça escolhe também os defeitos
/// que o dono vai ver.*
pub(crate) fn peca_de_fabrica() -> ph2d_mesh::Mesh {
    ph2d_mesh::shapes::sculpt_sphere(1.0)
}
