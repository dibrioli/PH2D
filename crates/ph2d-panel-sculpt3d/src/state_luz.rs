//! **COM QUE LUZ o barro é mostrado** — os modos da fileira *Light*.
//!
//! ⚠️ Irmão (`#[path]`) do [`super`], cortado pelo tecto de LOC do painel e pelo
//! ASSUNTO — a mesma forma dos `state_modes` e `state_channel`.

/// ⭐⭐⭐⭐ **COM QUE LUZ o barro é mostrado** — os três modos que a fileira
/// *Light* oferece (report do dono, 2026-09-20: *«precisamos como no blender
/// modos de shaders além do matcap para pintar»*).
///
/// ⛔⛔ **É uma SEGUNDA definição do conceito, e a duplicação é DELIBERADA:**
/// este painel não conhece o renderizador (ele não depende de `ph2d-mesh-render`
/// e não vai passar a depender — é UI, e aquela crate arrasta o `wgpu`), logo o
/// tipo do device não é nomeável aqui. *O que torna as duas honestas é a PONTE
/// ter gate de ida-e-volta* (`ph2d-app-sculpt3d`, `panel.rs`), que é o mesmo
/// desenho da `Deformacao::modo_e_inversao`.
///
/// ⚠️ **O `Flat` é o modo de PINTAR:** sem luz, a cor que o artista escolheu é a
/// cor que ele vê — e é por isso que ele é o primeiro chip da fileira.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LightMode {
    /// Sem luz nenhuma: o albedo cru.
    Flat,
    /// As lâmpadas do documento, pelo modelo de ARGILA.
    #[default]
    Rig,
    /// ⭐⭐⭐⭐ **A LEI QUE ASSA** — as mesmas lâmpadas, pelo OpenPBR, com o material
    /// desta peça e o céu que o rig produz (report do dono, 2026-09-21: *«o que se
    /// vê no objecto 3d não é o que se vê na sprite cozida»*).
    Pbr,
    /// O matcap `i` — a luz do olho.
    Matcap(u8),
}

impl LightMode {
    /// ⭐⭐⭐ **QUANTAS OPÇÕES VÊM ANTES DOS MATERIAIS** — e ela é DERIVADA, não contada à mão.
    ///
    /// ⚠️⚠️ **O pintor precisa deste número para saber quantos rótulos escrever antes dos matcaps, e
    /// escrevê-lo lá é a SEGUNDA contagem da mesma coisa.** Com ela parada em `2` depois do `Pbr`
    /// entrar, a fileira pintaria `Flat · Rig · Basic Bright · …` sobre os ids
    /// `flat · rig · pbr · matcap0 · …` ⇒ **cada chip de material despacharia o VIZINHO**, e o
    /// `every_matcap_chip_arms_its_own_material` ficaria VERDE, porque ele percorre os ids e nunca
    /// lê um rótulo. *Um chip com o nome errado e um chip morto dão reports diferentes; só o
    /// primeiro parece funcionar.*
    ///
    /// ⭐ Ela sai da [`Self::from_option_index`], que é a mesma escada que o despacho lê: o primeiro
    /// índice que resolve para um [`Self::Matcap`] é, por definição, quantos fixos vêm antes.
    pub const FIXOS: usize = {
        // ⚠️ Um laço `const` e não um literal: a escada muda num sítio só, e este número segue-a.
        let mut i = 0;
        loop {
            match Self::from_option_index_const(i) {
                Some(Self::Matcap(_)) | None => break i,
                Some(_) => i += 1,
            }
        }
    };

    /// A [`Self::from_option_index`] em contexto `const`, para a [`Self::FIXOS`] a poder percorrer.
    ///
    /// ⚠️ Ela devolve `None` acima da escada para o laço parar mesmo que alguém apague o braço final
    /// — *um laço `const` que não pára é um erro de compilação com uma mensagem que não ajuda*.
    const fn from_option_index_const(i: usize) -> Option<Self> {
        match i {
            0 => Some(Self::Flat),
            1 => Some(Self::Rig),
            2 => Some(Self::Pbr),
            n if n < 64 => Some(Self::Matcap(n as u8 - 3)),
            _ => None,
        }
    }

    /// **A OPÇÃO DA FILEIRA que este modo é** — `0` plano, `1` rig, `2 + i` o
    /// matcap `i`.
    ///
    /// ⚠️ **Uma porta e não a aritmética escrita duas vezes:** o pintor precisa
    /// dela para dizer qual chip está aceso e o despacho para a inverter, e as
    /// duas cópias divergiriam no dia do quarto modo — que é exactamente o que
    /// esta wave acabou de ser.
    ///
    /// ⭐ **E o QUINTO chegou em 2026-09-21** (o `Pbr`), pela porta que esta frase
    /// abriu: a aritmética mudou num sítio só, e os matcaps desceram de `2 + i`
    /// para `3 + i` sem que ninguém fora deste ficheiro precisasse de saber.
    #[must_use]
    pub const fn option_index(self) -> usize {
        match self {
            Self::Flat => 0,
            Self::Rig => 1,
            Self::Pbr => 2,
            Self::Matcap(i) => 3 + i as usize,
        }
    }

    /// A inversa de [`Self::option_index`] — o que o clique na fileira escolheu.
    #[must_use]
    pub fn from_option_index(i: usize) -> Self {
        match i {
            0 => Self::Flat,
            1 => Self::Rig,
            2 => Self::Pbr,
            n => Self::Matcap(u8::try_from(n - 3).unwrap_or(u8::MAX)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LightMode;
    use ph2d_tool_registry::hash_node_id;

    /// ⭐ **O ORÁCULO: o nome de cada opção da fileira, ESCRITO À MÃO e em ordem.**
    ///
    /// ⚠️⚠️ **Ele é escrito à mão de propósito** — derivá-lo da [`LightMode::from_option_index`]
    /// faria o gate comparar a porta consigo mesma e passar a afirmar **nada**. Aqui a lista é a
    /// terceira testemunha, e a corrente que ela fecha é *nome → índice → id*: mover um id, trocar a
    /// escada ou renomear uma chave quebram-na cada um por seu lado.
    const NOMES: [&str; 13] = [
        "sculpt3d.matcap.flat",
        "sculpt3d.matcap.rig",
        "sculpt3d.matcap.pbr",
        "sculpt3d.matcap.0",
        "sculpt3d.matcap.1",
        "sculpt3d.matcap.2",
        "sculpt3d.matcap.3",
        "sculpt3d.matcap.4",
        "sculpt3d.matcap.5",
        "sculpt3d.matcap.6",
        "sculpt3d.matcap.7",
        "sculpt3d.matcap.8",
        "sculpt3d.matcap.9",
    ];

    /// O modo que o ORÁCULO diz que a opção `i` é.
    fn modo_esperado(i: usize) -> LightMode {
        match NOMES[i].rsplit_once('.').expect("a chave tem um ponto").1 {
            "flat" => LightMode::Flat,
            "rig" => LightMode::Rig,
            "pbr" => LightMode::Pbr,
            m => LightMode::Matcap(m.parse().expect("o sufixo de um material e' um numero")),
        }
    }

    /// ⭐⭐⭐ **A ORDEM DOS IDS DA FILEIRA É A ESCADA DA PORTA — a POSIÇÃO é a tag.**
    ///
    /// ⚠️⚠️ **O despacho resolve o chip por [`LightMode::from_option_index`] do ÍNDICE no array**,
    /// logo um id que entre no FIM (que é onde um id novo costuma entrar) liga o chip do modo novo
    /// ao último material e desloca todos os outros — *cada chip passa a armar o VIZINHO*.
    ///
    /// ⛔ **E o `every_matcap_chip_arms_its_own_material` NÃO o vê:** ele percorre o array e pergunta
    /// à mesma porta que o despacho usa, logo os dois concordam **por construção**, em qualquer
    /// ordem. O que este gate acrescenta é a ponte para o NOME, que é o que o artista lê.
    ///
    /// **Mutação que deve sangrar:** trocar dois ids do array de sítio.
    #[test]
    fn a_ordem_dos_ids_da_fileira_e_a_escada_da_porta() {
        let ids = crate::ids::SCULPT3D_MATCAP;
        assert_eq!(
            ids.len(),
            NOMES.len(),
            "o oráculo desta fileira ficou para trás do array de ids"
        );
        for (i, &id) in ids.iter().enumerate() {
            assert_eq!(
                id,
                hash_node_id(NOMES[i]),
                "a opção {i} devia ter o id de `{}` e tem outro",
                NOMES[i]
            );
            assert_eq!(
                LightMode::from_option_index(i),
                modo_esperado(i),
                "a opção {i} tem o id de `{}` e a escada resolve-a para outro modo",
                NOMES[i]
            );
        }
        // ⭐ **O CONTROLO:** o array tem de ter os fixos MAIS pelo menos um material, senão a
        // varredura acima passaria sobre uma fileira vazia.
        assert!(
            ids.len() > LightMode::FIXOS,
            "controlo: a fileira tem de oferecer materiais ({} ids)",
            ids.len()
        );
    }

    /// ⭐⭐⭐ **E O PINTOR CONTA OS FIXOS PELA PORTA, nunca por um literal.**
    ///
    /// ⚠️⚠️ **Esta é a metade que uma MUTAÇÃO SOBREVIVENTE encomendou.** Com o `FIXOS` do pintor
    /// parado em `2` depois de o `Pbr` entrar, ele escreve `[Flat, Rig]` de rótulos fixos e depois
    /// os dez materiais, sobre ids que são `[flat, rig, pbr, matcap0 … matcap8]` ⇒ **o chip que diz
    /// «Basic Bright» despacha o PBR**, e a suíte inteira fica verde: o gate dos chips percorre ids
    /// (concorda por construção) e o da involução mede só a escada.
    ///
    /// ⛔ **Por TEXTO porque a const é privada ao pintor** — e ela é privada com razão, senão seria
    /// uma terceira resposta a *«quantos fixos há»*. *Uma régua textual é o preço de a lei estar no
    /// sítio certo.*
    ///
    /// **Mutação que deve sangrar:** `const FIXOS: usize = 2;` no `paint/body.rs`.
    #[test]
    fn o_pintor_conta_os_fixos_pela_porta() {
        let fonte = include_str!("paint/body.rs");
        assert!(
            fonte.contains("const FIXOS: usize = crate::state::LightMode::FIXOS;"),
            "o pintor da fileira voltou a contar os fixos a' mão — cada chip de material passa a \
             pintar o nome do vizinho, e nenhum gate de despacho o vê"
        );
    }
}
