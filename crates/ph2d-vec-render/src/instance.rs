//! **A camada de INSTÂNCIA de Motion** (ADR-0154) — módulo irmão pelo teto de LOC. O corte é por
//! assunto: aqui o desenho de UMA forma de instância e o LOTE que compartilha geometria; no
//! `lib.rs` fica o motor de path ([`crate::draw_path`]/[`crate::path_tess`]) que o `dispatch` da
//! cena usa. A metade cara (tesselar) é feita uma vez por geometria e reusada por instância —
//! o congelamento das 160k estrelas.

use ph2d_vec_scene::VecPath;
use ph2d_vector::scene_prepared::PreparedFill;
use ph2d_vector::{Affine, Brush, Color, Rect, VectorScene};

use crate::{PathTess, build_contours, draw_path_with, fill_rule, path_tess};

/// **Desenha UMA forma de instância de Motion** (ADR-0154) — a porta pública que o
/// passe vetorial de Motion usa. O `VecPath` é geometria **PURA** (a cor não mora
/// nele: instâncias iguais compartilham a MESMA geometria content-cached no
/// `VecPathStore`), então o preenchimento usa o `tint` da instância, não o `fill`
/// do path. Espelha o ramo de FILL do [`draw_path`](crate::draw_path) (a mesma `build_contours` +
/// `fill_rule`) e omite o traço — uma forma de Motion é uma silhueta preenchida.
/// `transform` já leva a geometria LOCAL à tela (o basis da instância ∘ câmera).
/// A `fill_rule` vem do path, então um anel (`EvenOdd`) desenha o furo.
pub fn draw_shape_instance(
    path: &VecPath,
    transform: Affine,
    tint: [f32; 4],
    target: &mut VectorScene,
) {
    let tess = tessellate_shape_instance(path);
    draw_shape_instance_tessellated(path, &tess, None, transform, tint, target);
}

/// A [`PathTess`] de uma instância de Motion — a metade CARA de [`draw_shape_instance`], separada
/// para que um lote de instâncias da MESMA geometria a construa uma vez ([`draw_shared_instances`]).
///
/// Duas espécies de vetor vivo entram por aqui. Um PRIMITIVO `source.shape` (ADR-0154) não carrega
/// tinta autorada (`fill`/`stroke` ambos `None`, ex.: `ellipse` é `..VecPath::default()`): ele SÓ
/// tem silhueta, então tessela só o preenchimento (o `tint` da instância o pinta no desenho). Um
/// vetor-DOCUMENTO `source.object` carrega o próprio fill/stroke, então tessela pela mesma
/// [`path_tess`] que o [`draw_path`](crate::draw_path) usa (o preenchimento e o traço quando difere).
pub(crate) fn tessellate_shape_instance(path: &VecPath) -> PathTess {
    if path.fill.is_some() {
        path_tess(path)
    } else {
        // Primitivo: a silhueta é do `tint` da instância, e o traço (se houver) é do próprio
        // path. (Sem `count_cook` — o caminho antigo cozia cru aqui, e os contadores do
        // `encode_cost_tests` têm de bater byte-a-byte com ele.)
        let cooked = path.cooked();
        // ⚠️ **O preenchimento leva só os contornos FECHADOS** (`Some(true)`) — é isso que
        // faz um caminho APARADO (Trim) desenhar só o traço, sem um `if` que pergunte se
        // houve trim: um trecho de contorno não tem interior, e fechá-lo implicitamente
        // desenharia a corda.
        let fill_bp = build_contours(&cooked, Some(true));
        // O traço leva TODOS os contornos. Sem contorno aberto ele é **o mesmo desenho** do
        // preenchimento ⇒ partilham uma construção só (a mesma economia do [`path_tess`]).
        let open =
            (0..cooked.contour_count()).any(|c| cooked.contour(c).is_some_and(|(_, cl)| !cl));
        let stroke_bp = (path.stroke.is_some() && open).then(|| build_contours(&cooked, None));
        let caixa_local = crate::caixa_dos_desenhos(Some(&fill_bp), stroke_bp.as_ref());
        PathTess {
            fill_bp: Some(fill_bp),
            stroke_bp,
            dash: crate::dash_of(&cooked, path.stroke.as_ref()),
            caixa_local,
            transbordo: crate::standalone::transbordo_do_caminho(path),
        }
    }
}

/// Desenha UMA instância de Motion a partir da geometria JÁ TESSELADA (`tess`) — a metade barata,
/// que só emite os comandos Vello e não constrói nada. Ramifica exatamente como
/// [`draw_shape_instance`]: um vetor-documento (com fill/stroke) honra a tinta autorada dele pela
/// [`draw_path_with`]; um primitivo (sem paint) é preenchido com o `tint` da instância.
///
/// ⚠️ Um vetor-documento vivo NÃO é re-tingido a jusante (as cores são as do desenho); a tile
/// assada era tingível — a troca nomeada de virar vivo. Fiar `tint` pelo fill/stroke do
/// [`draw_path`](crate::draw_path) é o follow-up.
pub(crate) fn draw_shape_instance_tessellated(
    path: &VecPath,
    tess: &PathTess,
    prep: Option<&PreparedFill>,
    transform: Affine,
    tint: [f32; 4],
    target: &mut VectorScene,
) {
    if path.fill.is_some() {
        // ⚠️⚠️ **UMA INSTÂNCIA DE MOTION DE UMA FORMA COM PADRÃO PINTA A `fallback`, e é DECLARADO.**
        //
        // Esta rota é alimentada pelo cozimento do Motion, que é outro oleoduto: ele não tem o mapa
        // de ladrilhos do quadro em mãos, e inventar um aqui seria adivinhar de onde ele vem. ⛔ Não
        // é «não deu» — é a fronteira desta wave, e o gate
        // `a_motion_instance_of_a_patterned_shape_paints_the_fallback` prende-a, para que o dia em
        // que alguém a mudar seja um acto deliberado e não um efeito colateral.
        // ⛔ O último `None`: uma INSTÂNCIA partilha a geometria por `geometry_id`, e a dilatação
        // de uma camada é indexada pelo id da FORMA — a mesma lei do ladrilho e do pincel.
        draw_path_with(path, tess, transform, target, crate::Derived::NONE);
    } else {
        let brush = Brush::Solid(Color::new(tint));
        if let Some(prep) = prep {
            // ⭐ O caminho já está encodado: esta cópia paga a pose, o estilo e a tinta, e o
            // caminho é um `extend_from_slice`. **Byte-idêntico** ao ramo de baixo — a prova é o
            // `o_carimbo_preparado_escreve_os_mesmos_bytes` da `ph2d-vector`, que compara os seis
            // fluxos do Vello e os dois contadores.
            target.fill_prepared(prep, transform, &brush);
        } else {
            let fill_bp = tess
                .fill_bp
                .as_ref()
                .expect("primitivo => fill_bp construido");
            target
                .inner_mut()
                .fill(fill_rule(path), transform, &brush, None, fill_bp);
        }
        // ⚠️ **O traço vem DEPOIS, e não em vez do preenchimento** (medido 2026-08-21). Um
        // primitivo com `stroke_width > 0` ia pela rota do DOCUMENTO, que só preenche quando
        // `path.fill.is_some()` — e um primitivo tem `fill: None` de propósito, porque a cor
        // dele é o `tint` da instância. A forma ficava **oca** no instante em que o artista
        // mexia na largura do traço, e o `motion.tint` a jusante deixava de pintar coisa
        // nenhuma. Pela porta ÚNICA do traço ([`crate::draw_stroke_with`]), então tracejado,
        // pontas e alinhamento são os mesmos de um caminho de documento.
        crate::draw_stroke_with(path, tess, transform, target, None, None);
    }
}

/// **Desenha um LOTE de instâncias que compartilham geometria por handle, tesselando cada handle
/// DISTINTO uma ÚNICA vez** (ADR-0154 — o congelamento das 160k estrelas). `instances` produz
/// `(handle, transform, tint)` por instância; `resolve(handle)` mapeia o handle ao `VecPath`
/// armazenado. As instâncias de um mesmo handle reusam a [`PathTess`] cacheada, então N cópias da
/// mesma estrela pagam UMA tesselação em vez de N.
///
/// É a PORTA de lote do produto (o passe vetorial de Motion), e o que a torna correta é o gate:
/// desenhar por aqui produz o MESMO encode que N [`draw_shape_instance`] independentes
/// ([[feedback_two_doors_to_the_same_question_diverge]]). O cache é POR FRAME — o handle é estável
/// enquanto o conteúdo não muda, mas re-tesselar um punhado de geometrias distintas por frame é
/// grátis, e um cache por-frame não guarda um `BezPath` velho de um handle reciclado.
///
/// ⭐⭐⭐ **E desde 2026-09-20 um PRIMITIVO também paga UM encode em vez de `N`** — a forma é
/// encodada uma vez ([`PreparedFill`]) e cada cópia carimba-a, pagando só a pose, o estilo e a
/// tinta. Medido pela porta do produto na cadeia do report (`102 400` cópias de uma estrela):
/// `5,66 → 1,76 ms` de encode (`3,2×`), e o quadro inteiro de `27 %` para `13 %` de um quadro de
/// 60 fps. ⛔ **Sem custo de nitidez, e não por promessa:** as duas rotas escrevem os MESMOS bytes,
/// e são três gates independentes a dizê-lo — o desta crate
/// (`a_shared_batch_draws_exactly_what_n_single_draws_do`, que compara o LOTE preparado contra as
/// `N` chamadas únicas NÃO preparadas), o `o_carimbo_preparado_escreve_os_mesmos_bytes` da
/// `ph2d-vector` (os seis fluxos, com tinta diferente por cópia) e o
/// `o_lote_carimba_a_forma_preparada_uma_vez_por_geometria`, que mede a CONTA — *a economia é
/// invisível a toda régua de valor, logo só a contagem a pode gatear*.
pub fn draw_shared_instances<'p>(
    instances: impl IntoIterator<Item = (u32, Affine, [f32; 4])>,
    resolve: impl Fn(u32) -> Option<&'p VecPath>,
    janela: Option<Rect>,
    target: &mut VectorScene,
) {
    draw_shared_instances_com(
        instances,
        resolve,
        target,
        carimbo_preparado(),
        recorte_ligado().then_some(janela).flatten(),
        None,
    );
}

/// ⭐⭐⭐ **O MESMO lote, com cada cópia na SUA camada de mistura** (doc 118 do Motion, o alcance
/// `Everything`/`Copies`: *cada cópia mistura-se com o que já está por baixo dela*).
///
/// ⚠️⚠️ **Ela é uma PORTA deste lote e não um laço à volta dele, por causa do CACHE:** a tesselação
/// é guardada por geometria **dentro de uma chamada**. Chamar o lote uma cópia de cada vez para lhe
/// pôr uma camada à volta re-tesselaria toda cópia — o congelamento das `160k` estrelas outra vez.
///
/// ⚠️ **A camada recorta à caixa da CÓPIA**, a mesma que o recorte por câmara já calcula: uma camada
/// do tamanho da janela por cópia misturaria o ecrã inteiro uma vez por cópia.
pub fn draw_shared_instances_em_camadas<'p>(
    instances: impl IntoIterator<Item = (u32, Affine, [f32; 4])>,
    resolve: impl Fn(u32) -> Option<&'p VecPath>,
    janela: Option<Rect>,
    mistura: ph2d_vector::VelloBlend,
    target: &mut VectorScene,
) {
    draw_shared_instances_com(
        instances,
        resolve,
        target,
        carimbo_preparado(),
        recorte_ligado().then_some(janela).flatten(),
        Some(mistura),
    );
}

/// ⭐⭐⭐ **A JANELA do alvo de desenho, com a folga de arredondamento** — o rectângulo contra o
/// qual uma cópia é julgada.
///
/// ⚠️⚠️ **O rectângulo certo é o ALVO DE RENDER e não a banda do canvas.** A cena vectorial desta
/// casa é rasterizada ao tamanho da JANELA inteira (`render_to_intermediate`, com
/// `window_size`) e o chrome é pintado POR CIMA, dentro dela — não há recorte de banda em lado
/// nenhum. ⇒ fora do alvo de render uma forma é invisível **por construção**, e recortar ali não
/// pode estar errado; recortar contra a banda dependeria de os painéis serem opacos, que
/// ninguém mediu. *O erro de um recorte tem um lado barato e um lado caro, e este fica no barato.*
///
/// ⚠️ A folga é de ARREDONDAMENTO e não de serrilha: a cobertura que o Vello calcula nunca sai da
/// caixa da geometria (um pixel pintado toca-a sempre), logo o que se cobre aqui é o último bit de
/// um `f64` que atravessou um afim. Um pixel chega, e dois são de graça.
const FOLGA_DA_JANELA: f64 = 2.0;

/// As duas caixas tocam-se?
///
/// ⚠️ **Com `>=` e `<=`, e é isso que mantém na cena a forma que ENCOSTA na borda** — a que está
/// meia dentro e meia fora é exactamente o caso que um recorte não pode comer.
fn toca(a: Rect, b: Rect) -> bool {
    a.x1 >= b.x0 && a.x0 <= b.x1 && a.y1 >= b.y0 && a.y0 <= b.y1
}

/// **A porta de bissecção do recorte** — `PH2D_RECORTE_DA_CAMARA=0` devolve o de antes (entregar
/// à placa também o que ninguém vê).
///
/// ⚠️ **Lida UMA vez, e só aqui** — a mesma lei da irmã [`carimbo_preparado`]: *um gate que lê o
/// ambiente mede a máquina*, logo os gates entram pelo [`draw_shared_instances_com`] e escolhem a
/// janela por parâmetro.
fn recorte_ligado() -> bool {
    static LIGADO: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *LIGADO.get_or_init(|| recorte_por(std::env::var("PH2D_RECORTE_DA_CAMARA").ok().as_deref()))
}

/// A LEI da porta acima, **pura** — o que a variável significa, sem a ler.
pub(crate) fn recorte_por(valor: Option<&str>) -> bool {
    !matches!(valor.map(str::trim), Some("0"))
}

/// O CORPO do lote, com a rota como **parâmetro**.
///
/// ⚠️⚠️ **Ela é um parâmetro e não uma leitura do ambiente, e a razão é uma lei desta casa:** *uma
/// lei que só é alcançável pelo ambiente não é gateável, e um gate que lê o ambiente mede a
/// máquina* (`CLAUDE.md` §5, `Cercas`). Os gates entram por aqui e escolhem a rota; o ambiente é
/// lido **uma vez**, na porta pública de cima.
pub(crate) fn draw_shared_instances_com<'p>(
    instances: impl IntoIterator<Item = (u32, Affine, [f32; 4])>,
    resolve: impl Fn(u32) -> Option<&'p VecPath>,
    target: &mut VectorScene,
    preparado: bool,
    janela: Option<Rect>,
    // ⭐ `Some(m)` = cada cópia na sua camada `m` — ver [`draw_shared_instances_em_camadas`].
    // `None` é o lote de sempre, byte a byte.
    por_copia: Option<ph2d_vector::VelloBlend>,
) {
    let janela = janela.map(|r| r.inflate(FOLGA_DA_JANELA, FOLGA_DA_JANELA));
    let mut cache: std::collections::BTreeMap<u32, (PathTess, Option<PreparedFill>)> =
        std::collections::BTreeMap::new();
    for (handle, transform, tint) in instances {
        let Some(path) = resolve(handle) else {
            continue; // handle sem geometria (um cook adiantado) desenha nada
        };
        let (tess, prep) = cache.entry(handle).or_insert_with(|| {
            let tess = tessellate_shape_instance(path);
            let prep = preparado.then(|| prepare_primitive(path, &tess)).flatten();
            (tess, prep)
        });
        // ⭐⭐⭐ **O RECORTE POR CÂMARA** (ordem do dono, 2026-09-21: *«pode implementar a cura»*).
        //
        // ⛔⛔ Medido na cena `=126` antes de uma linha ser escrita: das `90 000` cópias o artista
        // vê `8 736` — e a placa recebia as `90 000`, todos os quadros, `44,3 MB` de cena por
        // quadro. *O tecto não era a máquina no limite dela; era entregarmos-lhe dez vezes o
        // trabalho que alguém pode ver* (`CLAUDE.md` §0.0).
        //
        // ⚠️ **A decisão é por CÓPIA e a caixa é da GEOMETRIA**: a cara (`caixa_local`) sai da
        // tesselação, que este lote já paga UMA vez por geometria; o que corre por cópia é um
        // afim sobre quatro cantos mais quatro comparações.
        //
        // ⚠️ **Sem caixa, a forma SEGUE.** Uma `caixa_local` a `None` quer dizer *«não há
        // geometria que desenhe»* ou que a tesselação não a soube medir — e nos dois casos o valor
        // conservador é desenhar. *Um recorte que decide sobre o que não mediu é um buraco na
        // tela.*
        let caixa = tess
            .caixa_local
            .map(|local| crate::bounds_from_local(&tess.transbordo, local, transform));
        if let (Some(janela), Some(caixa)) = (janela, caixa)
            && !toca(caixa, janela)
        {
            #[cfg(test)]
            crate::encode_cost_tests::count_recortada();
            continue;
        }
        #[cfg(test)]
        crate::encode_cost_tests::count_stamp(prep.is_some());
        // ⚠️ **Sem caixa a camada cai para a JANELA** (o mesmo valor conservador do recorte: não
        // medir não autoriza cortar); sem as duas, a cópia desenha sem camada — e é o caso em que
        // não há geometria que pinte, logo não há mistura a perder.
        let camada = por_copia.and_then(|m| caixa.or(janela).map(|r| (m, r)));
        if let Some((m, r)) = camada {
            target.push_object_layer(&r, m, 1.0);
        }
        draw_shape_instance_tessellated(path, tess, prep.as_ref(), transform, tint, target);
        if camada.is_some() {
            target.pop_layer();
        }
    }
}

/// **A forma de um PRIMITIVO, encodada uma vez** — `None` para tudo o resto.
///
/// ⭐⭐ O que decide é a mesma pergunta que o [`draw_shape_instance_tessellated`] faz: um
/// vetor-DOCUMENTO (`path.fill.is_some()`) honra a tinta autorada dele e pode levar gradiente,
/// padrão ou dilatação — nada disso é *«uma cor por cópia»*, e por isso ele fica no ramo de sempre.
/// Um PRIMITIVO é uma silhueta preenchida com o `tint` da instância, que é exactamente a forma que
/// esta porta serve.
///
/// ⚠️ **O traço NÃO entra aqui** e continua pela porta única dele: a economia é do preenchimento,
/// que é o que `N` cópias repetem. *Um traço por cópia é raro no carimbo, e assá-lo obrigaria a
/// preparar também a expansão da caneta — outra wave, e sem medição que a peça.*
fn prepare_primitive(path: &VecPath, tess: &PathTess) -> Option<PreparedFill> {
    if path.fill.is_some() {
        return None;
    }
    tess.fill_bp
        .as_ref()
        .map(|bp| PreparedFill::new(bp, fill_rule(path)))
}

/// **A porta de bissecção do carimbo preparado** — `PH2D_CARIMBO_PREPARADO=0` devolve o caminho
/// antigo (um `Scene::fill` por cópia).
///
/// ⚠️ **Lida UMA vez, e só aqui.** A lei desta casa é que *um gate que lê o ambiente mede a
/// máquina* — por isso o gate da igualdade em bytes
/// (`ph2d-vector`, `o_carimbo_preparado_escreve_os_mesmos_bytes`) entra pela porta
/// [`ph2d_vector::VectorScene::fill_prepared`] **directamente** e não passa por aqui: ele afirma a
/// LEI, e esta variável só escolhe a ROTA.
///
/// ⭐ Ela existe porque as duas rotas são **byte-idênticas**: sem isso não haveria nada para
/// bissectar — haveria dois produtos.
fn carimbo_preparado() -> bool {
    static LIGADO: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *LIGADO.get_or_init(|| preparado_por(std::env::var("PH2D_CARIMBO_PREPARADO").ok().as_deref()))
}

/// A lei da porta, **pura** — o que a variável significa, sem a ler.
///
/// ⚠️ Ela existe separada por uma razão de instrumento: um gate que leia o ambiente mede a
/// **máquina** em que corre, e a pergunta *«qual é o caminho de OMISSÃO?»* é sobre o produto.
/// Aqui ela é uma função de um `Option<&str>`, logo o gate afirma as três células (ausente ·
/// `"0"` · outra coisa) sem tocar no processo.
pub(crate) fn preparado_por(valor: Option<&str>) -> bool {
    valor != Some("0")
}

#[cfg(test)]
#[path = "instance_tests.rs"]
mod tests;
