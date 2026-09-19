//! ⭐⭐⭐ **OS BOTÕES DO BRILHO, no painel** (`docs/Render3d/12`, a `W7`).
//!
//! # ⭐ A tabela é DERIVADA da arrumação, como a da camada de estilo
//!
//! O [`ph2d_field::Param::Bloom`] carrega a **posição** no [`Bloom::pack`], e é isso que
//! faz uma escrita ser uma linha: *desempacota, escreve a posição, empacota*. ⛔ Uma numeração
//! própria seria a segunda resposta à tabela do `pack`, e a que envelhece no dia em que nascer um
//! botão — a lei que o módulo [`crate::estilo`] já paga.
//!
//! # ✅ A NOTA QUE AQUI ESTAVA FOI APAGADA PELA WAVE QUE ELA ENCOMENDOU
//!
//! Até 2026-09-19 **todas** estas fileiras nasciam apagadas, com a razão
//! *«o brilho corre no caminho de referência»* — porque ele vivia só na cauda do sombreamento de
//! CPU e o caminho de omissão deste módulo é o **dispositivo**. A dívida estava nomeada aqui, com o
//! desenho por escrito (*o pintor a guardar o cena-linear, a cadeia de níveis em compute, a
//! composição*), e o gémeo existe desde então: [`ph2d_bloom::wgsl`] atravessa como texto e o
//! `ph2d-field-gpu` hospeda-o, com paridade medida contra a referência em **`1` byte**.
//!
//! ⇒ *as fileiras acendem nos DOIS caminhos, e o `PH2D_FIELD_GPU=0` deixou de ser condição para o
//! artista ver o efeito.* ⚠️ As outras três razões de uma fileira apagada (o brilho desligado, o
//! joelho com o limiar em zero, o ângulo num halo redondo) **ficam** — essas são da LEI.

// ⚠️ **Pelo RE-EXPORT do renderer e não pela crate da lei**, e é de propósito: esta crate fala com
// o motor, e o motor declara a superfície que ele aceita. *Duas importações do mesmo tipo não são
// um defeito em Rust, mas são duas respostas à pergunta «de quem é este tipo?».*
use ph2d_field::{Bound, Param};
use ph2d_field_render::Bloom;

/// A secção onde estas fileiras vivem.
const SECCAO: &str = "panel.model3d.section.bloom";

/// ⚠️ **As duas posições do interruptor, como CHAVES** (HR-15) — quem traduz é o painel.
const LIGA: [&str; 2] = ["panel.model3d.bloom.off", "panel.model3d.bloom.on"];

/// Uma linha da secção do brilho — a posição na arrumação, o nome, e de que natureza ela é.
struct Linha {
    /// A posição no [`Bloom::pack`].
    slot: u8,
    /// A chave i18n do rótulo.
    key: &'static str,
    /// ⭐ **O TECTO, ou como o derivar do próprio brilho.**
    ///
    /// ⚠️ Um **fecho** e não um número, porque o tecto do joelho **é o limiar** — ver a tabela do
    /// [`LINHAS`]. *Um tecto derivado escrito como constante seria um palpite com cara de medição no
    /// dia em que o artista mexesse no número de que ele depende.*
    teto: fn(&Bloom) -> f32,
    /// `true` ⇒ a linha é uma **escolha** (o interruptor), não um número.
    escolha: bool,
    /// `true` ⇒ a linha é uma **AMOSTRA de cor** (três canais consecutivos), não um número.
    ///
    /// ⚠️ **Uma cor escolhe-se VENDO-a** — é a lei que o [`ph2d_panel_model3d::ParamRow::swatch`]
    /// já escreve: três sliders dizem o que ela é depois de escolhida, e para a escolher obrigam o
    /// artista a resolver de cabeça o que o olho faz num gesto.
    cor: bool,
}

/// ⭐⭐⭐ **AS DEZ LINHAS** — o interruptor, a cor e sete números. **O NOSSO modelo.**
///
/// # ⛔⛔⛔ O modelo do Godot SAIU, por ordem do dono (19/09)
///
/// *«acho que ao tentar copiar a godot, ficou muito ruim. nosso bloom original é muito melhor.
/// retire essa implementação godot»*. A 1.ª redacção desta secção tinha **sete pesos por nível**
/// (`glow_levels/1..7`) e era pior por medição: sete controlos onde o nosso tem **um** (o
/// [`ph2d_bloom::BloomParams::radius`]), **quatro deles a nascer em zero** — quatro fileiras que se
/// leem como mortas —, e sem a saturação, a tinta, o tecto e a anamorfose que o nosso já tinha.
///
/// ⭐ **E o tipo é o MESMO do halo do Motion** (`ph2d_render::BloomParams` re-exporta esta folha):
/// os dois autoram-se com a mesma estrutura, logo não há duas cópias a concordar por promessa.
///
/// # ⛔⛔⛔ Os tectos, MEDIDOS (`os_tectos_do_brilho` e `quanto_o_raio_move`)
///
/// | linha | tecto | de que recurso, e o número |
/// |---|---|---|
/// | **limiar** | `32` | o recurso é o **PICO DA CENA**. ⭐ O número é DERIVADO de uma medição que já existia: a nota do [`ph2d_field_ecs::FieldMaterial::emission`] mede que *«acima de `32` a saída é bit a bit a mesma»* ⇒ acima disso o artista já não autora uma luz mais clara, e um limiar que não a alcança **não a pode apagar** |
/// | **joelho** | **o LIMIAR** | ⭐⭐⭐ **derivado**: a passagem mede `2k` de largura e a borda de baixo é `limiar − k`, logo em `k = limiar` ela **encosta no zero** (medido: a 1.ª luz cai de `0,876` para `0,002`). Acima disso ele deixa de suavizar a entrada e passa a fazer TUDO brilhar |
/// | **intensidade** | `4` | o halo é **linear** nela e o **raio SATURA** (`51 → 68 px`, parado a partir de `4`) ⇒ acima disso ela só multiplica o que o byte já satura |
/// | **raio** | `16` | ⭐⭐ **MEDIDO, e a faixa útil não começa em zero:** o centro do halo lê `9,71 · 9,58 · 9,59` nos raios `0,25 · 0,5 · 1` — **inerte** — e `11,0 · 15,4 · 25,9 · 48,1` em `2 · 4 · 8 · 16`. *Nesta cadeia quem faz o grosso do borrão é a descida por mips; a tenda alarga-o por cima*, e é ela que este knob estica |
/// | **saturação** | `1` | é uma FRACÇÃO: `0` puxa o halo ao cinzento, `1` guarda a cor da fonte. O domínio é a lei |
/// | **tinta** | — | uma cor (amostra), e o branco é o no-op |
/// | **tecto do corte** | `64` | o antídoto dos *fireflies*. ⚠️ `0` = **desligado**, que é o caminho literal; o recurso é a REPRESENTAÇÃO (`Rgba16Float` guarda até `65 504`) e o tecto do slider é o dobro da luz mais forte que a emissão autora |
/// | **estiramento** | `4` | a anamorfose. `1` = o halo redondo de sempre, **ao bit** |
/// | **ângulo** | `180` | a direcção do *streak*, em graus. ⚠️ Sem efeito com estiramento `1` (um círculo rodado é o mesmo círculo), e a fileira apaga-se lá |
const LINHAS: [Linha; 10] = [
    Linha {
        slot: 0,
        key: "panel.model3d.bloom.enabled",
        teto: |_| 1.0,
        escolha: true,
        cor: false,
    },
    Linha {
        slot: 1,
        key: "panel.model3d.bloom.threshold",
        teto: |_| 32.0,
        escolha: false,
        cor: false,
    },
    Linha {
        slot: 2,
        key: "panel.model3d.bloom.knee",
        // ⭐ O tecto DERIVADO — ver a tabela acima.
        teto: |b| b.params.threshold,
        escolha: false,
        cor: false,
    },
    Linha {
        slot: 3,
        key: "panel.model3d.bloom.intensity",
        teto: |_| 4.0,
        escolha: false,
        cor: false,
    },
    Linha {
        slot: 4,
        key: "panel.model3d.bloom.radius",
        teto: |_| 16.0,
        escolha: false,
        cor: false,
    },
    Linha {
        slot: 5,
        key: "panel.model3d.bloom.saturation",
        teto: |_| 1.0,
        escolha: false,
        cor: false,
    },
    Linha {
        slot: 6,
        key: "panel.model3d.bloom.tint",
        teto: |_| 1.0,
        escolha: false,
        cor: true,
    },
    Linha {
        slot: 9,
        key: "panel.model3d.bloom.clamp",
        teto: |_| 64.0,
        escolha: false,
        cor: false,
    },
    Linha {
        slot: 10,
        key: "panel.model3d.bloom.stretch",
        teto: |_| 4.0,
        escolha: false,
        cor: false,
    },
    Linha {
        slot: 11,
        key: "panel.model3d.bloom.angle",
        teto: |_| 180.0,
        escolha: false,
        cor: false,
    },
];

/// ⭐⭐⭐ **PORQUE É QUE ESTA FILEIRA NÃO FAZ NADA AGORA** — as três razões, da mais geral para a mais
/// específica, e a ordem é a lei.
///
/// ⚠️ **Dizer *«o brilho está desligado»* a quem também está no caminho do dispositivo é mandá-lo
/// resolver a metade errada** — é a lei que a `recusa::Entradas` do módulo da escultura já escreve,
/// e é por isso que a razão do MOTOR vem primeiro.
fn apagada(l: &Linha, b: &Bloom) -> Option<&'static str> {
    if l.slot != 0 && !b.enabled {
        return Some("field.inert.bloom_is_off");
    }
    // ⭐ O joelho com o limiar em zero: toda a luz já passa, logo não há passagem para suavizar.
    if l.slot == 2 && b.params.threshold == 0.0 {
        return Some("field.inert.bloom_threshold_is_zero");
    }
    // ⭐⭐ **O ÂNGULO com o estiramento em `1` é INERTE por GEOMETRIA** — um círculo rodado é o
    // mesmo círculo, e a lei devolve o caminho literal ali. *É a mesma razão pela qual o `ParamGate`
    // do nó `fx.glow` o esconde: não é um knob fraco, é um knob sem sujeito.*
    if l.slot == 11 && (b.params.stretch - 1.0).abs() < f32::EPSILON {
        return Some("field.inert.bloom_is_round");
    }
    None
}

/// ⭐⭐⭐ **AS FILEIRAS DO BRILHO** — vazias fora do modo Render, como as do estilo.
///
/// ⚠️ **`entity` é `0` e ninguém o lê** — ver [`ph2d_field::Param::Bloom`]. O dreno decide o sujeito
/// pela FAMÍLIA, e o sujeito do brilho é a cena.
#[must_use]
pub fn rows(bloom: Bloom, render: bool) -> Vec<ph2d_panel_model3d::ParamRow> {
    if !render {
        return Vec::new();
    }
    let v = bloom.pack();
    LINHAS
        .iter()
        .enumerate()
        .map(|(i, l)| ph2d_panel_model3d::ParamRow {
            entity: 0,
            param: Param::Bloom(l.slot),
            key: l.key,
            value: v[l.slot as usize],
            lo: 0.0,
            // ⚠️ **`Soft` e não `Hard`**: nenhum destes tectos é uma parede do documento — a lei
            // aceita qualquer número finito (e sanea-o na porta), e o que eles limitam é o GESTO.
            bound: Bound::Soft((l.teto)(&bloom)),
            inert: apagada(l, &bloom),
            // ⚠️ **Uma escolha também é inteira**, como a do eixo no painel do nó.
            integral: l.escolha,
            choices: if l.escolha { &LIGA } else { &[] },
            section: (i == 0).then_some(SECCAO),
            // ⭐ **A amostra é o que faz uma cor ser UMA linha** e não três — a mesma lei do estilo.
            swatch: l.cor.then(|| {
                crate::materials::colour_srgb8([
                    v[l.slot as usize],
                    v[l.slot as usize + 1],
                    v[l.slot as usize + 2],
                ])
            }),
            subject: None,
        })
        .collect()
}

/// ⭐⭐⭐ **A ESCRITA** — desempacota, escreve a posição, empacota, saneia.
///
/// ⚠️ **O saneamento é o da PORTA da crate** ([`Bloom::sanitized`]): a partir daqui o número viaja
/// para a thread que desenha, e ela não tem cerca. ⛔ Fora do alcance devolve o brilho intacto —
/// *uma recusa é informação, e o retrato publicado a seguir devolve o controlo ao valor que ficou.*
#[must_use]
pub fn with_colour(bloom: Bloom, anchor: u8, srgb: [u8; 3]) -> Bloom {
    let cor = crate::materials::colour_from_srgb8(srgb);
    let mut fora = bloom;
    for (k, c) in cor.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let slot = anchor + k as u8;
        fora = with_number(fora, slot, *c);
    }
    fora
}

/// ⭐⭐⭐ **A ESCRITA DE UM NÚMERO** — desempacota, escreve a posição, empacota, saneia.
#[must_use]
pub fn with_number(bloom: Bloom, slot: u8, value: f32) -> Bloom {
    let mut v = bloom.pack();
    let Some(alvo) = v.get_mut(slot as usize) else {
        return bloom;
    };
    *alvo = value;
    Bloom::unpack(&v).sanitized()
}

#[cfg(test)]
#[path = "brilho_painel_tests.rs"]
mod tests;

/// ⭐⭐⭐ **E a metade que mede PÍXEIS na cena do dono** — irmão por responsabilidade e por tecto de
/// LOC, nunca por isenção.
#[cfg(test)]
#[path = "brilho_cena_tests.rs"]
mod cena_tests;

/// ⏱️ **E as SONDAS** — as que imprimem e não afirmam. Ver o módulo.
#[cfg(test)]
#[path = "brilho_sondas_tests.rs"]
mod sondas_tests;
