//! ⭐⭐⭐ **OS BOTÕES DO BRILHO, no painel** (`docs/Render3d/12`, a `W7`).
//!
//! # ⭐ A tabela é DERIVADA da arrumação, como a da camada de estilo
//!
//! O [`ph2d_field::Param::Bloom`] carrega a **posição** no [`Bloom::pack`], e é isso que
//! faz uma escrita ser uma linha: *desempacota, escreve a posição, empacota*. ⛔ Uma numeração
//! própria seria a segunda resposta à tabela do `pack`, e a que envelhece no dia em que nascer um
//! botão — a lei que o módulo [`crate::estilo`] já paga.
//!
//! # ⚠️⚠️ Porque TODAS estas fileiras nascem apagadas no caminho de OMISSÃO
//!
//! O brilho corre na cauda do sombreamento de **CPU**, e o caminho de omissão deste módulo é o
//! **dispositivo** ([`crate::gpu_frame::enabled`], que é `true` sem a env var). ⇒ com o dispositivo
//! a tomar o quadro, mexer nestes números **não muda um pixel**.
//!
//! ⭐ *Isso não é razão para esconder a secção — é razão para ela DIZER.* A decisão do dono de
//! 2026-09-18 (`ParamRow::inert`) é exactamente esta: a fileira fica à vista, apagada, com a razão
//! ao lado. ⛔ Um knob que se mexe e não faz nada é o defeito que este módulo já pagou três vezes
//! noutra família (*«não vejo efeito com density»*); um knob apagado que diz porquê é uma
//! instrução.
//!
//! ⏳ **E a dívida tem nome:** o gémeo em WGSL (o `p_pinta` do `ph2d-field-gpu` a escrever o
//! cena-linear num segundo buffer, a cadeia de níveis em compute e a composição) é a metade que
//! apaga esta nota. Até lá, `PH2D_FIELD_GPU=0` acende as fileiras.

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
}

/// ⭐⭐⭐ **AS ONZE LINHAS** — o interruptor, os três números e os sete níveis.
///
/// # ⛔⛔⛔ Os tectos, MEDIDOS (a sonda `os_tectos_do_brilho` e a `a_sonda_do_joelho`)
///
/// ⚠️ **A 1.ª sonda mediu em BYTES e saturou** — as duas colunas leram *«o fundo inteiro»* e
/// *«`255`»* em toda a varredura, que é a lição que o [`docs/Render3d/12` §3.3] já escrevia sobre o
/// oráculo: *uma régua que mede a SAÍDA do produto herda o tecto da saída do produto*. A tabela
/// abaixo é medida no **halo em linear**, onde ele nasce.
///
/// | linha | tecto | de que recurso, e o número |
/// |---|---|---|
/// | **limiar** | `32` | ⚠️ o recurso é o **PICO DA CENA**, e ele não tem tecto (um material emissivo escreve-o). ⭐ **O número é DERIVADO de uma medição que já existia neste repo:** a nota do [`ph2d_field_ecs::FieldMaterial::emission`] mede que *«acima de `32` a saída é bit a bit a mesma»* ⇒ acima disso o artista já não consegue autorar uma luz mais clara pelo slider, e um limiar que não alcança a luz mais forte é um limiar que **não a pode apagar**. ⛔ **Era `16`, e o report do dono expôs o preço:** a luz de `32` da cena `=36` era inapagável, e o passo do roteiro que manda subir o limiar prometia o que o painel não deixava fazer. Medido numa cena de pico `40`: a `16` já foram `38 %` do halo, a `32` `80 %` |
/// | **joelho** | **o LIMIAR** | ⭐⭐⭐ **derivado, e é o tecto mais bem fundamentado desta secção**: a passagem mede `2k` de largura e a borda de baixo é `limiar − k`, logo em `k = limiar` ela **encosta no zero** — medido, a 1.ª luz cai de `0,876` (`k = 0,125`) para `0,002` (`k = 1`, o limiar). *Acima disso ele deixa de suavizar a entrada e passa a fazer TUDO brilhar*, que é outra pergunta |
/// | **intensidade** | `4` | o halo é **linear** nela (`0,1 → 1 141` · `1,0 → 11 412`, exactamente `10×`) e o **raio SATURA** (`51 → 68 px`, parado a partir de `4`) ⇒ acima de `4` ela só multiplica o que o byte já satura |
/// | **níveis 1..7** | `1` | ⭐ o peso é linear no halo e **NÃO muda o raio** (`27 → 30 px` sobre uma varredura de `64×`) ⇒ acima de `1` ele é a **MESMA alavanca** que a intensidade, e *duas alavancas para a mesma coisa é o que esta casa proíbe* |
///
/// ⭐⭐ **E o RAIO é dos NÍVEIS, não da intensidade** — medido um nível de cada vez, o raio a partir
/// da borda da peça dobra: `5 · 12 · 24 · 52 px`. É a geometria que a §3.4 mediu no oráculo, e é ela
/// que faz os sete níveis serem sete controlos e não um.
const LINHAS: [Linha; 4 + Bloom::LEVELS] = [
    Linha {
        slot: 0,
        key: "panel.model3d.bloom.enabled",
        teto: |_| 1.0,
        escolha: true,
    },
    Linha {
        slot: 1,
        key: "panel.model3d.bloom.threshold",
        teto: |_| 32.0,
        escolha: false,
    },
    Linha {
        slot: 2,
        key: "panel.model3d.bloom.knee",
        // ⭐ O tecto DERIVADO — ver a tabela acima.
        teto: |b| b.threshold,
        escolha: false,
    },
    Linha {
        slot: 3,
        key: "panel.model3d.bloom.intensity",
        teto: |_| 4.0,
        escolha: false,
    },
    Linha {
        slot: 4,
        key: "panel.model3d.bloom.level_1",
        teto: |_| 1.0,
        escolha: false,
    },
    Linha {
        slot: 5,
        key: "panel.model3d.bloom.level_2",
        teto: |_| 1.0,
        escolha: false,
    },
    Linha {
        slot: 6,
        key: "panel.model3d.bloom.level_3",
        teto: |_| 1.0,
        escolha: false,
    },
    Linha {
        slot: 7,
        key: "panel.model3d.bloom.level_4",
        teto: |_| 1.0,
        escolha: false,
    },
    Linha {
        slot: 8,
        key: "panel.model3d.bloom.level_5",
        teto: |_| 1.0,
        escolha: false,
    },
    Linha {
        slot: 9,
        key: "panel.model3d.bloom.level_6",
        teto: |_| 1.0,
        escolha: false,
    },
    Linha {
        slot: 10,
        key: "panel.model3d.bloom.level_7",
        teto: |_| 1.0,
        escolha: false,
    },
];

/// ⭐⭐⭐ **PORQUE É QUE ESTA FILEIRA NÃO FAZ NADA AGORA** — as três razões, da mais geral para a mais
/// específica, e a ordem é a lei.
///
/// ⚠️ **Dizer *«o brilho está desligado»* a quem também está no caminho do dispositivo é mandá-lo
/// resolver a metade errada** — é a lei que a `recusa::Entradas` do módulo da escultura já escreve,
/// e é por isso que a razão do MOTOR vem primeiro.
fn apagada(l: &Linha, b: &Bloom, no_dispositivo: bool) -> Option<&'static str> {
    if no_dispositivo {
        return Some("field.inert.bloom_runs_on_the_reference_path");
    }
    if l.slot != 0 && !b.enabled {
        return Some("field.inert.bloom_is_off");
    }
    // ⭐ O joelho com o limiar em zero: toda a luz já passa, logo não há passagem para suavizar.
    if l.slot == 2 && b.threshold == 0.0 {
        return Some("field.inert.bloom_threshold_is_zero");
    }
    None
}

/// ⭐⭐⭐ **AS FILEIRAS DO BRILHO** — vazias fora do modo Render, como as do estilo.
///
/// ⚠️ **`entity` é `0` e ninguém o lê** — ver [`ph2d_field::Param::Bloom`]. O dreno decide o sujeito
/// pela FAMÍLIA, e o sujeito do brilho é a cena.
#[must_use]
pub fn rows(bloom: Bloom, render: bool, no_dispositivo: bool) -> Vec<ph2d_panel_model3d::ParamRow> {
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
            inert: apagada(l, &bloom, no_dispositivo),
            // ⚠️ **Uma escolha também é inteira**, como a do eixo no painel do nó.
            integral: l.escolha,
            choices: if l.escolha { &LIGA } else { &[] },
            section: (i == 0).then_some(SECCAO),
            // ⛔ Nenhuma destas é uma cor.
            swatch: None,
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
