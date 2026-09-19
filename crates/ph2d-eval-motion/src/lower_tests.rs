//! ADR-0154 gates for the `geometry_id` lowering convention — the sibling of
//! `texture_id` (doc 86). A row whose `geometry_id > 0` is a crisp vector shape
//! (lowered to a [`VectorInstance`] the shell draws through `ph2d-vec-render`); a
//! row of 0, or no column at all, is a sprite. The convention is ADDITIVE: a
//! stream without the column lowers exactly as it did before shapes existed.

use crate::lower::{lower_to_instances_onto, lower_to_vector_instances_onto};
use crate::{Column, RenderInstance, Stream, VectorInstance};
use ph2d_render::SinkStyle;

const UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const SZ: [f32; 2] = [1.0, 1.0];

/// A stream with no `geometry_id` column lowers to ALL sprites and ZERO vectors —
/// byte-identical to the pre-shape world. FALSIFIED by a lowering that invents a
/// vector where the convention column is absent.
#[test]
fn a_stream_without_geometry_id_is_all_sprites_and_no_vectors() {
    let s = Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]));
    let mut sprites: Vec<RenderInstance> = Vec::new();
    lower_to_instances_onto(&s, UV, SZ, SinkStyle::PLAIN, &mut sprites);
    assert_eq!(sprites.len(), 3, "every row is a sprite");
    let mut vectors: Vec<VectorInstance> = Vec::new();
    lower_to_vector_instances_onto(&s, SinkStyle::PLAIN, &mut vectors);
    assert!(vectors.is_empty(), "no geometry_id column ⇒ no vectors");
}

/// A mixed stream SPLITS by `geometry_id`: rows of 0 are sprites, rows > 0 are
/// vectors — each side keeping its own rows, in order. FALSIFIED by an inverted
/// filter (the split is the whole convention).
#[test]
fn geometry_id_splits_sprites_from_vectors() {
    // Rows 0 & 2 are sprites (id 0); rows 1 & 3 are shapes (id 5, 3).
    let s = Stream::new(4)
        .with(
            "P",
            Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]]),
        )
        .with("geometry_id", Column::Scalar(vec![0.0, 5.0, 0.0, 3.0]));

    let mut sprites: Vec<RenderInstance> = Vec::new();
    lower_to_instances_onto(&s, UV, SZ, SinkStyle::PLAIN, &mut sprites);
    assert_eq!(sprites.len(), 2, "the two id-0 rows are sprites");
    assert_eq!(sprites[0].world_pos, [0.0, 0.0]);
    assert_eq!(sprites[1].world_pos, [2.0, 0.0]);

    let mut vectors: Vec<VectorInstance> = Vec::new();
    lower_to_vector_instances_onto(&s, SinkStyle::PLAIN, &mut vectors);
    assert_eq!(vectors.len(), 2, "the two id>0 rows are vectors");
    assert_eq!(vectors[0].geometry_id, 5);
    assert_eq!(vectors[0].world_pos, [1.0, 0.0]);
    assert_eq!(vectors[1].geometry_id, 3);
    assert_eq!(vectors[1].world_pos, [3.0, 0.0]);
}

/// The `geometry_id` and `texture_id` conventions COMPOSE: a shape row carries a
/// live `geometry_id` AND is skipped by the sprite lowering, so a shape is never
/// ALSO stamped as a shared-atlas quad (the doc-86 pattern, one axis over).
#[test]
fn a_shape_row_is_not_also_a_sprite() {
    let s = Stream::new(1)
        .with("P", Column::Vec2(vec![[7.0, 8.0]]))
        .with("geometry_id", Column::Scalar(vec![9.0]));
    let mut sprites: Vec<RenderInstance> = Vec::new();
    lower_to_instances_onto(&s, UV, SZ, SinkStyle::PLAIN, &mut sprites);
    assert!(sprites.is_empty(), "a shape row is not a sprite");
    let mut vectors: Vec<VectorInstance> = Vec::new();
    lower_to_vector_instances_onto(&s, SinkStyle::PLAIN, &mut vectors);
    assert_eq!(vectors.len(), 1);
    assert_eq!(vectors[0].geometry_id, 9);
}

/// **A COLUNA `blend` DECIDE POR LINHA, E O `0` É *O DO SINK*** (doc 89, folha 07 — o
/// *Echo Operator*).
///
/// ⚠️ **As três metades numa só, porque separá-las esconderia o defeito caro.** A escada é
/// `0 = o do sink`, `m + 1 = o modo m`; um design que guardasse o modo CRU faria a
/// identidade de junção (`0`) baixar toda linha alheia para `Normal` — em silêncio, e só
/// numa cena que compõe em `Add`.
#[test]
fn a_blend_column_overrides_per_row_and_zero_means_the_sinks_mode() {
    // O sink compõe em `Add` (tag 1). A stream traz a coluna: linha 0 sem escolha (`0`),
    // linha 1 escolhe `Normal` (tag 0 ⇒ valor 1), linha 2 escolhe `Screen` (tag 4 ⇒ 5).
    let s = Stream::new(3)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; 3]))
        .with("blend", Column::Scalar(vec![0.0, 1.0, 5.0]));
    let mut out = Vec::new();
    lower_to_instances_onto(
        &s,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        SinkStyle {
            blend: 1,
            ..SinkStyle::PLAIN
        },
        &mut out,
    );
    assert_eq!(out.len(), 3);
    let sink = RenderInstance::pack_blend_bits(1);
    assert_eq!(out[0].flip_uv, sink, "0 na coluna = o modo do SINK");
    assert_eq!(out[1].flip_uv, RenderInstance::pack_blend_bits(0), "Normal");
    assert_eq!(out[2].flip_uv, RenderInstance::pack_blend_bits(4), "Screen");
    assert_ne!(
        out[0].flip_uv, out[1].flip_uv,
        "senao a coluna nao decide nada"
    );
}

/// **SEM A COLUNA, NADA MUDA** — o default byte-idêntico, e o controle do gate acima.
#[test]
fn a_stream_without_the_column_lowers_exactly_as_before() {
    let s = Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0]; 2]));
    let mut out = Vec::new();
    lower_to_instances_onto(
        &s,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        SinkStyle {
            blend: 3,
            ..SinkStyle::PLAIN
        },
        &mut out,
    );
    let sink = RenderInstance::pack_blend_bits(3);
    assert!(out.iter().all(|i| i.flip_uv == sink));
}

/// **UM NÚMERO ABSURDO NA COLUNA NÃO ESCOLHE UM PIPELINE QUE NÃO EXISTE.**
///
/// ⚠️ A coluna é escrita por um NÓ, mas nada impede um `value.*` de a produzir — e o
/// `flip_uv` indexa um array de pipelines do renderer. O teto é lido DE LÁ.
#[test]
fn a_wild_column_value_is_clamped_to_the_renderers_pipelines() {
    let top = ph2d_render::pipeline::BLEND_PIPELINE_COUNT as f32;
    let s = Stream::new(3)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; 3]))
        .with("blend", Column::Scalar(vec![999.0, -4.0, f32::NAN]));
    let mut out = Vec::new();
    lower_to_instances_onto(
        &s,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        SinkStyle {
            blend: 2,
            ..SinkStyle::PLAIN
        },
        &mut out,
    );
    #[expect(clippy::cast_possible_truncation, reason = "o teto cabe num u8")]
    let last = RenderInstance::pack_blend_bits((top as u8) - 1);
    let sink = RenderInstance::pack_blend_bits(2);
    assert_eq!(out[0].flip_uv, last, "999 satura no ultimo modo REAL");
    assert_eq!(out[1].flip_uv, sink, "negativo = sem escolha = o do sink");
    assert_eq!(
        out[2].flip_uv, sink,
        "NaN idem — nunca um pipeline inventado"
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// ⭐⭐⭐ A LEI DO DONO: *«sem o duplicator só aparece um gizmo»* — [`SinkStyle::so_com_forma`].
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// O estilo com a lei LIGADA. ⚠️ Construído à MÃO e nunca lido do ambiente: é isso que faz estes
/// gates medirem a LEI em vez de medirem a máquina (a auditoria do doc 115 §31).
const SO_COM_FORMA: SinkStyle = SinkStyle {
    so_com_forma: true,
    ..SinkStyle::PLAIN
};

/// **UMA CORRENTE DE POSIÇÕES NÃO PRODUZ PIXEL NENHUM** — a ordem do dono, nos DOIS lowerings.
///
/// ⚠️ **As duas metades são obrigatórias.** Escrita só no lowering das sprites, o passe VECTORIAL
/// continuaria a pintar a mesma corrente — a forma de defeito que o cabeçalho de `MediaColumns` já
/// nomeia (*«a mesma linha desenhava-se duas vezes ou nenhuma»*).
#[test]
fn uma_corrente_de_posicoes_nao_desenha_quando_a_lei_esta_ligada() {
    let s = Stream::new(4).with("P", Column::Vec2(vec![[0.0, 0.0]; 4]));

    let mut sprites: Vec<RenderInstance> = Vec::new();
    lower_to_instances_onto(&s, UV, SZ, SO_COM_FORMA, &mut sprites);
    assert!(
        sprites.is_empty(),
        "posicoes sem forma nao viram sprite: {} linhas",
        sprites.len()
    );

    // ⛔⛔ **A 2.ª metade tem de trazer a TERCEIRA MÉDIA, e isto foi uma MUTAÇÃO SOBREVIVENTE.**
    //
    // Com a corrente nua acima, apagar a saída cedo do lowering vectorial **não é observável**: sem
    // `geometry_id` e sem `vector_pass` toda linha é `RowMedium::Sprite`, e aquele passe já
    // devolvia `None` para todas ⇒ a asserção ficava verde sobre a mutação. *A fixtura não
    // produzia o fenómeno.*
    //
    // Quem o produz é a corrente que o `vector_pass` marca para o passe VECTORIAL **sem trazer
    // ladrilho**: hoje ela desenha um quad com o ladrilho de omissão da shell, dentro da cena
    // Vello. É essa que a lei tem de calar.
    let marcada = Stream::new(3)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; 3]))
        .with(
            crate::lower::VECTOR_PASS_COLUMN,
            Column::Scalar(vec![1.0; 3]),
        );
    let mut vectors: Vec<VectorInstance> = Vec::new();
    lower_to_vector_instances_onto(&marcada, SinkStyle::PLAIN, &mut vectors);
    assert_eq!(
        vectors.len(),
        3,
        "o CONTROLO: hoje o passe vectorial pinta-as"
    );
    vectors.clear();
    lower_to_vector_instances_onto(&marcada, SO_COM_FORMA, &mut vectors);
    assert!(vectors.is_empty(), "com a lei, o passe vectorial cala-se");
}

/// **E O CONTROLO: desligada, a MESMA corrente desenha como sempre desenhou.**
///
/// ⚠️ Sem esta metade o gate acima passaria sobre um lowering que não desenha NADA — *uma régua
/// que não vê o fenómeno acontecer não prova que ele não aconteceu*.
#[test]
fn a_mesma_corrente_desenha_com_a_lei_desligada() {
    let s = Stream::new(4).with("P", Column::Vec2(vec![[0.0, 0.0]; 4]));
    let mut sprites: Vec<RenderInstance> = Vec::new();
    lower_to_instances_onto(&s, UV, SZ, SinkStyle::PLAIN, &mut sprites);
    assert_eq!(sprites.len(), 4, "o de sempre: o ladrilho de omissao");
}

/// **O QUE VEIO DE UM `source.object` CONTINUA A DESENHAR, AO BIT** — a lei não é «apagar tudo»:
/// ela distingue *quem recebeu uma forma* de *quem só tem posições*.
#[test]
fn o_que_traz_ladrilho_desenha_igual_com_a_lei_ligada() {
    let s = Stream::new(3)
        .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]]))
        .with("uv_rect", Column::Vec4(vec![[0.0, 0.0, 0.5, 0.5]; 3]));
    let (mut ligada, mut desligada) = (Vec::new(), Vec::new());
    lower_to_instances_onto(&s, UV, SZ, SO_COM_FORMA, &mut ligada);
    lower_to_instances_onto(&s, UV, SZ, SinkStyle::PLAIN, &mut desligada);
    assert_eq!(ligada.len(), 3);
    // ⚠️ O `Debug` derivado despeja os CAMPOS TODOS: uma comparação campo a campo escrita à mão
    // ficaria verde no dia em que a `RenderInstance` ganhasse mais um.
    assert_eq!(
        format!("{ligada:?}"),
        format!("{desligada:?}"),
        "a lei nao toca em quem tem aparencia"
    );
}

/// **E O QUE TRAZ GEOMETRIA VIVA também** — a outra origem (`source.shape`/`text`/`lsystem`), pelo
/// passe vectorial.
#[test]
fn o_que_traz_geometria_viva_desenha_com_a_lei_ligada() {
    let s = Stream::new(2)
        .with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0]]))
        .with("geometry_id", Column::Scalar(vec![7.0, 7.0]));
    let mut vectors: Vec<VectorInstance> = Vec::new();
    lower_to_vector_instances_onto(&s, SO_COM_FORMA, &mut vectors);
    assert_eq!(vectors.len(), 2, "geometria viva e' aparencia");
}

/// ⛔⛔ **UM `geometry_id` TODO A ZERO É *«não é forma»*, e não uma aparência.**
///
/// A convenção desta casa põe `0` como *«esta linha não é uma forma»* (o `> 0.5` do `RowMedium`),
/// logo uma corrente que carrega a coluna a zeros continua a ser POSIÇÕES. ⚠️ Sem este gate, a
/// porta podia perguntar `is_some()` à coluna — o que é certo para o ladrilho e **falso** aqui — e
/// toda corrente que passasse por um nó que a materializa a zeros voltaria a desenhar quadrados.
#[test]
fn geometria_a_zeros_continua_a_ser_posicoes() {
    let s = Stream::new(3)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; 3]))
        .with("geometry_id", Column::Scalar(vec![0.0, 0.0, 0.0]));
    assert!(!crate::lower::tem_aparencia(&s));
    let mut sprites: Vec<RenderInstance> = Vec::new();
    lower_to_instances_onto(&s, UV, SZ, SO_COM_FORMA, &mut sprites);
    assert!(sprites.is_empty());
}

/// **UMA CORRENTE MISTA DESENHA** — a junção de formas com pontos carrega a coluna do ladrilho,
/// logo a lei não a apaga; quem decide o destino de cada linha continua a ser o `RowMedium`.
///
/// ⚠️ É a fronteira que a decisão «por CORRENTE e não por linha» escolheu, e ela está declarada
/// nos dois doc-comments: por linha, o device teria de compactar a saída.
#[test]
fn uma_corrente_mista_continua_a_desenhar() {
    let s = Stream::new(4)
        .with("P", Column::Vec2(vec![[0.0, 0.0]; 4]))
        .with("geometry_id", Column::Scalar(vec![0.0, 7.0, 0.0, 7.0]))
        .with("uv_rect", Column::Vec4(vec![[0.0, 0.0, 0.5, 0.5]; 4]));
    let mut sprites: Vec<RenderInstance> = Vec::new();
    lower_to_instances_onto(&s, UV, SZ, SO_COM_FORMA, &mut sprites);
    let mut vectors: Vec<VectorInstance> = Vec::new();
    lower_to_vector_instances_onto(&s, SO_COM_FORMA, &mut vectors);
    assert_eq!(sprites.len(), 2, "as duas linhas sem geometria");
    assert_eq!(vectors.len(), 2, "as duas com geometria");
}

/// ⭐⭐⭐ **A LEI SHIPA DESLIGADA** — a lei da casa, e esta linha já a violou uma vez (doc 115 §31,
/// o corte em duas camadas que shipou ligado e foi invertido por auditoria).
///
/// ⚠️ **A metade CONSTANTE dela é do compilador** (`const _: () = assert!(..)`, em
/// `ph2d-render/src/sink_style.rs`): um `assert!` de teste sobre uma const é dobrado antes de
/// correr e não afirma nada. O que sobra para um teste é a metade que só existe em RUNTIME — **a
/// porta do PRODUTO**, que é a que de facto decide o que o dono vê.
///
/// ⛔⛔ **E ele percorre a ROTA, nunca chama a porta** — isto foi uma MUTAÇÃO SOBREVIVENTE: a 1.ª
/// redacção perguntava a `so_com_forma_por_ordem()` directamente, e cravar a resposta do fio
/// dentro do `sink_style` (`so_com_forma: true`) deixava-a **verde**. *Um gate que chama a função
/// em vez de percorrer a rota afirma que a peça certa existe, nunca que o produto a usa.*
#[test]
fn a_porta_do_produto_shipa_desligada() {
    use ph2d_nodegraph::graph::Graph;
    let mut g = Graph::new();
    let sink = g.add_node("motion.output");
    assert!(
        !crate::sink_style::sink_style(&g, sink).so_com_forma,
        "sem `PH2D_MOTION_SO_COM_FORMA` o estilo que o produto monta tem a lei desligada"
    );
    // E a porta pura, pelo mesmo caminho — as duas metades, porque uma porta certa ligada a nada
    // e um fio certo com a porta errada dão o mesmo sintoma.
    assert!(!crate::sink_style::so_com_forma_por_ordem());
}

/// **E A PORTA LÊ O QUE O DONO ESCREVE** — a escada das outras portas da casa.
#[test]
fn a_porta_le_o_que_o_dono_escreve() {
    use crate::sink_style::ordem_de;
    assert!(!ordem_de(None), "ausente = desligada");
    assert!(!ordem_de(Some("")), "vazia = desligada");
    assert!(!ordem_de(Some("0")), "zero = desligada");
    assert!(ordem_de(Some("1")));
    assert!(ordem_de(Some("sim")));
}
