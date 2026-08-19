//! **O FILTRO** — o verbo aplicado à MALHA INTEIRA, dirigido por um arrasto.
//!
//! ⚠️ **NÃO HÁ KERNEL NOVO, e é isso que a wave é.** Um traço já é
//! `lerp(base, alvo, accum)`: o `pre` congela no pen-down, cada dab enche o
//! `accum` e escreve o `alvo`, e o aplicador anda. Um filtro **enche o `accum`
//! ele mesmo, uniformemente**, e chama as MESMAS leis de alvo — as quatro
//! funções do [`super::ring`] e o `base + n·f` do [`Verb::Inflate`]. É a frase
//! do irmão 2-D (o *Filter Layer* do Painter) palavra por palavra, e é a razão
//! pela qual esta wave é fiação e não geometria.
//!
//! ⚠️ **Filho (`#[path]`) de [`super`] pelo mesmo motivo dos outros nove:** ele
//! lê os planos congelados (`base_pos`, `base_nrm`, `base_mask`, `slot`,
//! `accum`, `target`) e escreve a lista `moved`. Um irmão obrigaria os seis a
//! virar `pub(crate)`, e a visibilidade viraria função do TAMANHO do arquivo.
//!
//! # A lei, e onde ela diverge da referência
//!
//! O `sculpt_filter_mesh.cc` computa `translations` a partir de
//! `orig_data.positions` e escreve `eval + t`; o `reset_translations_to_original`
//! existe para que um arrasto seja **UMA** operação a partir da pose do
//! pen-down, e não uma composição de todas as posições por que o rato passou.
//! Aqui ele é o [`SculptStroke::restore_frozen_pose`], e **é uma metade da lei,
//! não cerimónia** — ver o doc dele.
//!
//! O que sai de graça é a outra metade: `accum = 1` faz o
//! [`super::apply::toward`] devolver o alvo AO BIT (é a promessa `a = 1 → t`
//! que aquele doc mede), e com `f = 0` toda lei devolve o próprio `base` ⇒ o
//! arrasto de volta é **exacto**.
//!
//! ⚠️ **A divergência DECLARADA é o histórico.** Para o Smooth e o Relax o
//! Blender guarda os eventos do arrasto e **REPLAYA** a cadeia inteira
//! (`sculpt_mesh_filter_apply_with_history`), porque `smooth^n ≠ smooth(n·s)` —
//! `n` passos pequenos difundem mais longe que um passo grande. Nós aplicamos
//! **um** passo de magnitude `s` a partir do `pre`, e o preço está nomeado: o
//! nosso Smooth **satura** (em `f = 1` o vértice pousa exactamente na média do
//! anel, e mais que isso ele não alcança), enquanto a cadeia da referência não
//! tem esse teto. Em troca ganhamos duas propriedades que ela não tem — o
//! arrasto de volta é **exacto** e não há histórico a guardar —, e o dia em que
//! o smoke disser *"não alisa o suficiente"* a resposta é um knob de
//! ITERAÇÕES com um número medido ao lado, nunca o histórico.

use super::*;
// A aritmética do alvo vem do IRMÃO que já a tem: uma segunda cópia de
// `base + n·f` aqui seria a segunda resposta a *"como se soma um vetor a um
// ponto"*, e é o mesmo motivo pelo qual o `cross` já é re-exportado de lá.
use super::target::add;
use crate::FilterKind;

/// **QUANTO UM PIXEL DE ARRASTO VALE**, em unidades de mundo por pixel.
///
/// ⚠️ **A LEI é da referência e a CALIBRAÇÃO é nossa, e a divisão importa.** O
/// `calc_inflate_filter` desloca `orig_normals × strength` — ou seja o `strength`
/// **é** a distância, em unidades de objeto, sem raio nenhum no meio (ao
/// contrário de todo verbo de carimbo, que passa pelo [`Brush::reach`] e escala
/// com o raio do pincel). Então a única coisa que sobra para escolher é quantos
/// pixels custam uma unidade.
///
/// ⚠️ **O número é o do Blender** (`filter_strength = start · −len · 0,001 ·
/// UI_SCALE`, `sculpt_filter_mesh.cc:2301`): mil pixels de arrasto valem um
/// `strength` de `1,0`. Sobre a esfera de fábrica (raio ~1) isso é um arrasto de
/// tela cheia para inflar por um raio inteiro — que é exactamente o que a
/// referência faz, e é por isso que o número não é uma escolha nossa a defender.
pub const FILTER_DRAG_PER_PX: f32 = 0.001;

impl SculptStroke {
    /// **Congela a malha INTEIRA** — o `filter_begin` do gesto.
    ///
    /// ⚠️ **Uma vez por GESTO, não por passo do arrasto.** É o que faz o
    /// arrasto ser reversível: cada [`Self::filter`] reescreve o alvo a partir
    /// do mesmo `pre`, então voltar a força a zero devolve a pose exacta. Chamar
    /// isto a cada passo faria de um arrasto uma composição de filtros — o
    /// oposto da promessa, e ainda por cima com o resultado dependendo de quantos
    /// eventos o rato mandou.
    ///
    /// ⚠️ **O `footprint` é preenchido com a malha inteira de propósito:** ele é
    /// a lista que o [`Self::fill_hc_disp`] percorre, e enchê-lo aqui é o que
    /// faz o [`Verb::SurfaceSmooth`] chegar ao filtro **pela mesma porta** do
    /// dab, em vez de por uma segunda travessia escrita neste arquivo.
    pub fn filter_begin(&mut self, mesh: &Mesh) {
        self.begin(mesh);
        let n = mesh.vert_count();
        self.footprint.clear();
        self.footprint.reserve(n);
        for v in 0..n as u32 {
            self.capture(mesh, v);
            self.footprint.push(v);
        }
    }

    /// **DEVOLVE a malha à pose do [`Self::filter_begin`]** — o
    /// `reset_translations_to_original` da referência, e a metade da lei sem a
    /// qual o filtro deixa de ser *um passo a partir do `pre`*.
    ///
    /// ⚠️ **As leis de anel leem o mesmo `mesh` que este passe escreveu**
    /// (`neighbour_average` sobre `mesh.positions()`, `relax_normal` sobre
    /// `mesh.normals()`), e **num traço isso está CERTO de propósito** — o doc
    /// do [`Self::neighbour_average`] o diz e corrige um doc anterior que
    /// creditava a idempotência ao lugar errado: quem impede a superfície de
    /// derreter sob um pincel parado é o APLICADOR, que interpola de `base_pos`
    /// em vez de compor sobre o resultado anterior. ⚠️ **Aqui esse freio não
    /// existe:** `accum = 1` faz o aplicador escrever o alvo VERBATIM, então
    /// duas chamadas na mesma força **compõem** — e um arrasto é uma chamada
    /// por evento de ponteiro, ou seja *o desenho passaria a depender de
    /// quantos eventos o rato mandou*. É a lei que esta casa já pagou quatro
    /// vezes no relevo do Painter (**o traço é fato do CAMINHO, nunca de quão
    /// fino o motor amostrou o caminho**) e que o [`super::freeze`] declara no
    /// próprio cabeçalho: *um termo que lê estado VIVO realimenta com a
    /// deformação que ele próprio produziu*. Medido antes da cura: o vértice 0
    /// **andava** na segunda aplicação de `Smooth(0,4)`.
    ///
    /// ⚠️ **A restauração é a cura, e não um anel congelado**, porque ela deixa
    /// as três leis de anel serem chamadas **VERBATIM** — uma variante de
    /// `neighbour_average` que lesse o `base_pos` seria a segunda resposta a
    /// *"qual é a média do anel deste vértice?"*, e as duas divergiriam no dia
    /// em que uma delas ganhasse um caso especial.
    ///
    /// ⚠️ **As NORMAIS voltam junto, e não é zelo:** uma pose são as posições
    /// **e** as normais que concordam com elas. Restaurar só metade entregaria
    /// às leis de anel uma malha INCONSISTENTE, e o que a salvaria seria a
    /// premissa *"o Smooth não olha para normais"* — exactamente a espécie de
    /// premissa que apodrece no dia em que alguém acrescenta essa leitura. O
    /// [`Mesh`] não expõe escrita de normal (de propósito: ela não pode
    /// discordar das posições), então elas são **recomputadas** das posições já
    /// restauradas ⇒ o mesmo número que o `capture` congelou, pela mesma
    /// função. *Se o custo desta segunda passada aparecer num arrasto de malha
    /// grande, a cura NOMEADA é uma porta de escrita que copie o `base_nrm`,
    /// que já está na mão — mas ela abre a possibilidade de normal que mente, e
    /// só se paga com um número ao lado.*
    /// ⚠️ **`pub(in crate::stroke)` porque tem DOIS consumidores** — este
    /// filtro e o [`super::sharpen`], que é um filtro com pré-passe e portanto
    /// vive noutro arquivo. A alternativa (uma segunda restauração lá) seria a
    /// segunda resposta a *"o que um filtro faz antes de tudo?"*, divergindo no
    /// dia em que a captura ganhasse um plano.
    pub(in crate::stroke) fn restore_frozen_pose(&mut self, mesh: &mut Mesh) {
        let out = mesh.positions_mut();
        // `touched[i]` e `base_pos[i]` são paralelos POR CONSTRUÇÃO — o
        // `capture` empurra os dois no mesmo passo, e o `slot` que ele escreve
        // é o índice de ANTES do push. Percorrer por índice em vez de pelo
        // `slot` é a mesma resposta, sem a indireção.
        for (i, &v) in self.touched.iter().enumerate() {
            out[v as usize] = self.base_pos[i];
        }
        mesh.refresh_region(&self.touched, &mut self.region);
    }

    /// **Aplica o verbo do `brush` à malha inteira**, com `amount` de força
    /// COM SINAL. Devolve quantos vértices foram escritos.
    ///
    /// ⚠️ **O `amount` é o arrasto, e o [`Brush::strength`] fica de FORA.** O
    /// irmão 2-D toma a Strength do pincel como o `k` dele porque o gesto de lá
    /// é um **BOTÃO**, que não carrega número nenhum; aqui o gesto é um
    /// **ARRASTO**, que carrega. Multiplicar os dois daria duas portas para uma
    /// grandeza, e o artista giraria um slider sem entender por que o arrasto
    /// mudou de escala.
    ///
    /// ⚠️ **O que o `brush` ainda decide é a LEI** — o [`crate::RefMode`] dele
    /// governa o operador do anel e o `β` do HC, exactamente como no carimbo.
    /// Um filtro com modo próprio seria a segunda resposta a *"que referência
    /// governa este verbo?"*.
    ///
    /// ⚠️ **Sem [`Self::filter_begin`] ele devolve `0`**, e a guarda é DERIVADA
    /// (a captura cobre a malha?) em vez de um flag: um segundo campo dizendo
    /// *"estou em modo filtro"* seria um fato guardado duas vezes, e o dia em
    /// que os dois discordassem o filtro rodaria sobre um `pre` de outro gesto.
    pub fn filter(
        &mut self,
        mesh: &mut Mesh,
        brush: &Brush,
        kind: FilterKind,
        amount: f32,
    ) -> usize {
        if self.touched.len() != mesh.vert_count() {
            return 0;
        }
        // ⚠️ **A ÚNICA lei que não cabe no laço abaixo, e a bifurcação é
        // estrutural:** o alvo de um vértice do Sharpen é pesado pela curvatura
        // dos VIZINHOS, então ele precisa de um passe que meça a malha inteira
        // antes de escrever o primeiro ponto. Um braço no `match` que às vezes
        // dependesse de um pré-passe seria a porta pela qual o próximo caso
        // especial entra sem ninguém ver.
        if kind == FilterKind::Sharpen {
            return self.filter_sharpen(mesh, brush, amount);
        }
        let (lo, hi) = kind.range();
        // ⚠️ **A ORDEM é load-bearing:** o `fill_hc_disp` lê `mesh.positions()`,
        // então ele tem de ver a pose JÁ restaurada — senão o `b` do HC seria o
        // deslocamento medido contra a saída da chamada anterior.
        self.restore_frozen_pose(mesh);
        // O `b` do HC, pela MESMA porta do dab — no-op para os outros três.
        // ⚠️ **O filtro: quem lê o `hc_b` é a LEI, nunca o verbo.** Desde o
        // picker da W9b o artista escolhe a lei com o pincel que estiver em
        // mãos, e perguntar pelo verbo aqui deixava o buffer vazio debaixo do
        // leitor — o panic que o smoke da `=34` reportou.
        self.fill_hc_disp(mesh, brush, kind == FilterKind::SurfaceSmooth);

        // ⚠️ **UMA vez por gesto, nunca por vértice** — o rotor deste app é a
        // rotação de um grau ACUMULADA (O(graus)), e por vértice ele custaria
        // mais que o campo que orienta. É a mesma hoist que o `dab_core` faz, e
        // a assinatura do [`Brush::alpha_weight`] existe para a impor.
        let alpha_frame = brush.alpha_frame();

        self.moved.clear();
        self.moved.reserve(self.touched.len());
        for i in 0..self.touched.len() {
            let v = self.touched[i];
            let s = self.slot[v as usize] as usize;
            let base = self.base_pos[s];
            // ⚠️ **A ORDEM é a da referência:** `scale_factors(factors,
            // strength)` e só depois `clamp_factors` — o fator parte da máscara,
            // é escalado pelo arrasto e **então** é aparado. Clampar antes
            // deixaria um vértice meio-mascarado receber mais que um livre no
            // extremo da faixa.
            // ⚠️ **O ALPHA É MAIS UM PESO POR-VÉRTICE, exactamente como a
            // máscara** — mesmo produto, mesma posição na ordem da referência,
            // antes do clamp. Não há semântica nova: um [`crate::Alpha`] é um
            // CAMPO avaliado num ponto, e ele nunca precisou de um dab para
            // existir. O gesto de malha inteira recebia o `brush` e simplesmente
            // nunca lhe perguntava — a seção Alpha do painel não tinha efeito
            // nenhum sobre ele (report do Enio, 2026-08-19).
            //
            // ⚠️ **Avaliado no `base`, a pose CONGELADA** — e não na viva, pela
            // razão do `the_weight_is_a_fact_about_the_frozen_surface`: um campo
            // lido na pose que ele próprio deformou escorrega debaixo do gesto.
            //
            // ⚠️ **Sem alpha armado a porta devolve `1.0` EXATO**, e `x * 1.0`
            // é `x` ao bit no IEEE-754 — o mundo pré-alpha não perde um bit, e
            // há gate a medir isso (`an_unarmed_alpha_moves_no_bit_of_the_filter`).
            let f = (amount
                * crate::mask_ops::free_weight(self.base_mask[s])
                * brush.alpha_weight(base, &alpha_frame))
            .clamp(lo, hi);
            // ⚠️ **O `match` é sobre a LEI e não sobre o verbo, e é isso que o
            // torna exaustivo:** uma lei nova na tabela do
            // [`Verb::filter_kind`] que não seja implementada aqui é um erro de
            // COMPILAÇÃO, e não um chip que aparece na lista e não move um
            // vértice.
            //
            // ⚠️ **As leis leem o `mesh`, e aqui o `mesh` É a pose congelada**
            // — o [`Self::restore_frozen_pose`] acabou de a repor, então `live`
            // e `base` são o MESMO ponto e toda entrada da lei é o `pre`. É
            // isso que faz do filtro um passo a partir do pen-down em vez de
            // uma composição sobre a chamada anterior.
            //
            // ⚠️ **Consequência HONESTA e não escondida:** com `q == o`, o
            // `α` do [`Verb::SurfaceSmooth`] (`hc_shape`) fica **INERTE** neste
            // gesto — ele interpola entre *a posição de agora* e *a do
            // pen-down*, e numa operação de UM passo elas coincidem. É a
            // degeneração correcta do knob, não um controle que se perdeu: quem
            // o torna vivo é a iteração, e a iteração é a divergência que o
            // cabeçalho declara.
            let t = match kind {
                FilterKind::Inflate => add(base, self.base_nrm[s], f),
                FilterKind::Smooth => self.target_smooth(mesh, brush, v, base, f),
                FilterKind::Relax => self.target_slide_relax(mesh, brush, v, base, f),
                FilterKind::SurfaceSmooth => self.target_surface_smooth(mesh, v, s, base, f, brush),
                FilterKind::Scale => add(base, base, f),
                FilterKind::Sphere => sphere_target(base, f),
                FilterKind::Random => add(
                    base,
                    self.base_nrm[s],
                    f * random_unit(base, self.filter_seed),
                ),
                // ⚠️ **O [`Self::target_sharpen`] e nao um `target_smooth(-f)`,
                // e a escolha vale um ULP:** a expressao daquele kernel
                // (`live + (live - avg)*w`) **e'** a do
                // `calc_enhance_details_filter`, enquanto o `target_smooth`
                // escreve a mesma lei como `live*(1-w) + avg*w` — algebricamente
                // igual, e em `f32` a 1,2e-7 de distancia (medido). Rotear pela
                // expressao da referencia torna a paridade uma identidade em vez
                // de um epsilon.
                // ⚠️ **O `f.abs()` é o `-std::abs(strength)` da referência
                // (`sculpt_filter_mesh.cc:1883`, a PRIMEIRA linha do
                // `calc_enhance_details_filter`), e sem ele esta lei fazia o
                // OPOSTO do que o chip promete para metade do gesto.**
                //
                // A referência realça nos DOIS sentidos do arrasto: `t =
                // detail_directions × −|s|` é *afastar da média com |força|*,
                // e o sinal do que o artista arrasta não entra. Sem o `abs`, um
                // arrasto para trás dava `live + (live − avg)·(−2)` — o vértice
                // ATRAVESSA a média do próprio anel e sai do outro lado ao dobro
                // da distância, invertendo a curvatura local.
                //
                // ⚠️ **Medido, e o número é o pior possível:** com `f = −1` o
                // resultado era **byte-idêntico** ao `Smooth(+1)` — o chip dizia
                // *Enhance Details* e a malha ALISAVA à força total; com `−2`,
                // **os 830 vértices** da sonda atravessavam a média.
                //
                // ⚠️ **E o gesto o alcança sem esforço:** esta é a única das nove
                // leis cuja `range()` é `(f32::MIN, f32::MAX)` — o clamp que
                // apararia o negativo é, aqui, deliberadamente inerte (é a
                // entrega da W9c-a), então o arrasto chega cru ao kernel.
                //
                // ⚠️ **A máscara não estraga a identidade:** `free_weight >= 0`,
                // logo `|amount·w| == |amount|·w` e aparar aqui é exactamente o
                // que a referência faz um passo antes.
                FilterKind::EnhanceDetails => self.target_sharpen(mesh, brush, v, base, f.abs()),
                // ⚠️ **A guarda no topo já o consumiu, e este braço é a PROVA
                // de que o `match` continua exaustivo** — foi ele que apanhou a
                // lei nova em tempo de compilação, que é exactamente o que o
                // comentário acima promete. Um `_ => ` genérico aqui apagaria
                // essa propriedade para todas as leis futuras.
                FilterKind::Sharpen => unreachable!("o pré-passe corre antes do laço"),
            };
            // ⚠️ **`accum = 1`, e o alvo é a posição FINAL** — a lei que os
            // quatro gestos com âncora já usam (`GripLaw::unit_accum`), e a
            // única em que `toward(base, alvo, accum)` devolve o alvo AO BIT.
            // Sem ela o filtro precisaria de um segundo aplicador.
            self.accum[s] = 1.0;
            self.target[s] = t;
            self.moved.push(v);
        }
        self.apply_positions(mesh, false);
        mesh.refresh_region(&self.moved, &mut self.region);
        self.moved.len()
    }
}

/// **A ESFERA** — `midpoint(normalize(p), −p) · |f|`, somado à pose congelada.
///
/// ⚠️ **A lei é a do `calc_sphere_translations`, verbatim**, e o que ela faz não
/// é óbvio pelo nome: `normalize(p)` é o ponto projectado na esfera UNITÁRIA e
/// `−p` é o espelho pela origem, então o ponto médio dos dois é
/// `(unit(p) − p)/2` — *metade do caminho até a esfera de raio um*. Um vértice
/// que já está a distância 1 da origem não se move, um que está longe vem para
/// dentro, um que está perto sai. É por isso que o verbo se chama Sphere e não
/// *"projectar numa esfera"*: ele **aproxima**, e a magnitude do arrasto diz
/// quanto.
///
/// ⚠️ **O `|f|` é da referência e NÃO é um descuido a corrigir:** arrastar para
/// os dois lados esferiza igual. Escrever `f` cru daria a metade que a
/// referência não tem — *afastar-se da esfera* —, e seria uma lei nossa a
/// vestir o nome dela.
///
/// ⚠️ **A guarda da ORIGEM é NOSSA, e a única resposta finita:** um vértice
/// exactamente na origem não tem direcção para uma esfera, então ele fica onde
/// está. Sem ela, `normalize` de um vector nulo entra no alvo e a malha inteira
/// herda o `NaN` pela recomputação de normais.
fn sphere_target(base: [f32; 3], f: f32) -> [f32; 3] {
    let len = (base[0] * base[0] + base[1] * base[1] + base[2] * base[2]).sqrt();
    if len <= f32::EPSILON {
        return base;
    }
    let a = f.abs() * 0.5;
    let mut t = base;
    for i in 0..3 {
        // `unit − p` na mesma expressão: `p/len − p`, sem materializar o
        // versor, porque a soma seguinte é a única leitora dele.
        t[i] += (base[i] / len - base[i]) * a;
    }
    t
}

/// **O SORTEIO por vértice**, em `[−0,5, 0,5)` — o `randomize_factors` da
/// referência.
///
/// ⚠️ **A entrada são os BITS da POSIÇÃO, e é isso que dá a propriedade:**
/// dois vértices vizinhos têm coordenadas quase iguais e bits completamente
/// diferentes, então o campo é branco POR VÉRTICE em vez de suave no espaço —
/// o oposto de um ruído espacial, e é o que um *jitter* quer.
///
/// ⚠️ **E é isso que torna a SEMENTE quase supérflua, medido no desenho:** cada
/// gesto re-congela a pose, então um segundo Random sobre uma malha já
/// perturbada hasheia posições **outras** e sorteia de novo sozinho. A semente
/// só serve para re-sortear **sem mover nada**, e por isso nasce em `0` — o
/// crate fica determinista e a variação é uma escolha do shell.
///
/// ⚠️ **DIVERGÊNCIA DECLARADA, e ela não é afinável:** a referência mistura com
/// o `BLI_hash_int_2d`, cuja definição **não está neste clone** (medido: só o
/// arquivo que a chama). Usamos o hash desta crate ([`crate::alpha::hash4`], o
/// mesmo dos alphas procedurais) — *um crate, um hash* — e o que os gates
/// afirmam são as quatro propriedades que fazem de um ruído um ruído
/// (determinismo · a faixa · estabilidade ao longo do arrasto · vizinhos
/// descorrelacionados), nunca uma paridade bit-a-bit que não temos como medir.
fn random_unit(base: [f32; 3], seed: u32) -> f32 {
    let h = crate::alpha::hash4(
        base[0].to_bits() as i32,
        base[1].to_bits() as i32,
        base[2].to_bits() as i32,
        seed as i32,
    );
    crate::alpha::unit(h) - 0.5
}
