//! **A metade do shell do `audio.bands`** (doc 63 §3 · doc 89 folha 14 §3 item 6) —
//! decodifica o arquivo, corre a transformada, dobra os compartimentos em bandas e
//! publica o nível de cada uma no canal externo que o nó lê.
//!
//! Irmão exacto do [`motion_text_gen`](super::motion_text_gen) e do
//! [`motion_shape_gen`](super::motion_shape_gen), e pelo mesmo motivo: um nó
//! recebe params, entradas e o playhead — nada mais —, então quem alcança um
//! arquivo, um decodificador e uma FFT é o shell.
//!
//! ## A cerca, escrita no doc 63 §6 antes de existir código
//!
//! *"FFT NUNCA entra no cook."* A `ph2d-node-audio-bands` **não depende de crate
//! de áudio nenhuma** (arch-gate sobre o `Cargo.toml` dela), então a contenção é
//! estrutural: o nó não CONSEGUE analisar som. O que ele possui é a LEI —
//! [`fold`](ph2d_node_audio_bands::fold) e [`BandSpec::edges`] —, e é ela que este
//! módulo chama. Os dois lados concordam por chamarem a mesma função, não por
//! duas implementações combinarem.
//!
//! ## A ANÁLISE é cacheada; o INSTANTE não
//!
//! Analisar uma faixa de três minutos custa uma transformada sobre milhões de
//! amostras, e o resultado é **função do arquivo e dos params** — nunca do
//! playhead. Então a matriz `colunas × bandas` é construída UMA vez por
//! `(arquivo, análise)` e o trabalho por quadro é **um índice**.
//!
//! ⚠️ **É por isso que o suavizado cabe aqui e é scrub-exato:** ele corre ao longo
//! das colunas do ARQUIVO enquanto a matriz é construída, então voltar a régua dá
//! o mesmo número. Um one-pole sobre quadros da sessão daria outro (a doença de
//! estado que o `motion.emitter` recusa por desenho).
//!
//! ## Por que o espectrograma do editor de áudio, e não uma transformada própria
//!
//! [`Spectrogram`] responde *o que há neste som* e já é o que o editor PINTA.
//! Uma segunda varredura aqui seria uma segunda resposta à mesma pergunta, e as
//! barras do grafo passariam a discordar da imagem do painel sobre o mesmo
//! arquivo. O preço é conhecido e aceitável: um byte por compartimento (0,35 dB
//! por degrau) e *peak-hold* acima de ~87 s de clipe — mais fino que qualquer
//! barra que alguém desenhe.

use ph2d_audio_spectral::Spectrogram;
use ph2d_node_audio_bands::{BandSpec, FILE_KEY, MANIFEST, VALUE_COL, fold, smooth_over_columns};
use ph2d_nodegraph::attr::{Column, Stream};

use crate::motion_state::MotionState;

/// O default do manifesto para um param — o fallback que o `ctx.param` do nó toma
/// quando não há override. Ler pelo mesmo caminho dos dois lados é o que faz a
/// chave do shell e a do nó serem os mesmos bits.
/// A análise pronta de um arquivo sob uma configuração: `cols × count` níveis já
/// dobrados, ponderados, normalizados e suavizados.
pub(crate) struct BandTrack {
    /// `cols * count` valores, linha-maior por coluna.
    levels: Vec<f32>,
    count: usize,
    cols: usize,
    /// Amostras entre colunas — lido da análise, **nunca assumido** (um clipe
    /// longo é decimado e o passo cresce com o comprimento).
    hop: usize,
    sample_rate: u32,
}

impl BandTrack {
    /// Os `count` níveis no instante `seconds`. Fora do clipe devolve a coluna de
    /// borda — o silêncio de antes/depois já está gravado nela.
    pub(crate) fn at(&self, seconds: f64) -> &[f32] {
        if self.cols == 0 || self.count == 0 {
            return &[];
        }
        let frame = (seconds.max(0.0) * f64::from(self.sample_rate)) as usize;
        let col = (frame / self.hop.max(1)).min(self.cols - 1);
        &self.levels[col * self.count..(col + 1) * self.count]
    }

    /// Um leitor vazio — arquivo ausente, ilegível ou vazio.
    fn empty(count: usize) -> Self {
        Self {
            levels: Vec::new(),
            count,
            cols: 0,
            hop: 1,
            sample_rate: 48_000,
        }
    }

    /// Constrói do clipe decodificado. **É aqui que a FFT roda**, uma vez por
    /// `(arquivo, análise)`.
    fn build(data: &ph2d_audio::SampleData, spec: &BandSpec) -> Self {
        let pic = Spectrogram::build(data);
        if pic.is_empty() {
            return Self::empty(spec.count);
        }
        let (cols, hz) = (pic.columns(), pic.hz_per_bin());
        let mut levels = Vec::with_capacity(cols * spec.count);
        let (mut db, mut band) = (Vec::new(), Vec::new());
        for c in 0..cols {
            pic.column_db(c, &mut db);
            fold(&db, hz, spec, &mut band);
            levels.extend_from_slice(&band);
        }
        // O suavizado corre sobre as colunas do ARQUIVO — a metade que torna o
        // scrub exato (ver o doc do módulo).
        smooth_over_columns(&mut levels, spec.count, spec.smoothing);
        Self {
            levels,
            count: spec.count,
            cols,
            hop: pic.hop(),
            sample_rate: pic.sample_rate(),
        }
    }
}

/// As análises vivas, endereçadas pela chave de conteúdo do nó.
///
/// ⚠️ **Nada é despejado, e o recurso está NOMEADO:** a chave carrega os oito
/// params, então varrer um slider constrói uma análise por valor — mas cada uma
/// custa `cols × count` floats (a 8192 colunas e 16 bandas, **512 kB**), e o
/// conjunto vivo é limitado pelo número de nós que o artista põe na tela. Um
/// despejo por-quadro entraria no dia em que isto medisse alguma coisa.
#[derive(Default)]
pub(crate) struct BandCache {
    tracks: std::collections::BTreeMap<String, BandTrack>,
    /// Clipes decodificados, por caminho — trocar UM param não relê o disco.
    clips: std::collections::BTreeMap<String, Option<std::sync::Arc<ph2d_audio::SampleData>>>,
}

impl BandCache {
    /// O clipe daquele caminho, decodificado no máximo uma vez por sessão.
    ///
    /// ⚠️ **Um caminho ilegível é memoizado como AUSENTE** (`None`), senão todo
    /// quadro tentaria abrir o mesmo arquivo que não existe.
    fn clip(&mut self, path: &str) -> Option<std::sync::Arc<ph2d_audio::SampleData>> {
        if let Some(hit) = self.clips.get(path) {
            return hit.clone();
        }
        let decoded = std::fs::read(path)
            .ok()
            .and_then(|bytes| crate::audio::decode_any::decode(&bytes).ok())
            .map(std::sync::Arc::new);
        self.clips.insert(path.to_string(), decoded.clone());
        decoded
    }

    /// A análise para `(caminho, spec)`, construída na primeira vez.
    pub(crate) fn track(&mut self, key: &str, path: &str, spec: &BandSpec) -> &BandTrack {
        if !self.tracks.contains_key(key) {
            let built = match self.clip(path) {
                Some(data) => BandTrack::build(&data, spec),
                None => BandTrack::empty(spec.count),
            };
            self.tracks.insert(key.to_string(), built);
        }
        &self.tracks[key]
    }

    /// Quantas análises estão vivas (sonda dos gates).
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.tracks.len()
    }
}

/// Publica, para cada `audio.bands` do grafo, os níveis das bandas no instante
/// `seconds`, sob a chave de conteúdo que o nó lê.
///
/// ⚠️ **A chave NÃO carrega o instante**, e é o desenho: ela nomeia *quais bandas
/// deste arquivo*, e este `publish` reescreve o mesmo nome a cada quadro com os
/// valores do playhead. Quem diz ao cook que o valor se move é o `Effect::Temporal`
/// do nó — uma chave por instante encheria o canal externo de nomes mortos.
pub(crate) fn publish(motion: &mut MotionState, seconds: f64) {
    // Junta os trabalhos primeiro para o empréstimo do grafo morrer antes de
    // mutarmos o cache e o cook (campos disjuntos do `MotionState`). ⚠️ A resolução da
    // escada COZINHA o driver, então ela precisa do `motion` inteiro — daí o laço, e não o
    // `map` sobre um `&graph` emprestado.
    let ids: Vec<ph2d_nodegraph::graph::NodeId> = motion
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == MANIFEST.name)
        .map(|n| n.id)
        .collect();
    let mut jobs: Vec<(String, String, BandSpec)> = Vec::with_capacity(ids.len());
    for id in ids {
        // ⚠️ **A ESCADA INTEIRA** (`conduzido → override → default`), pela porta que as três
        // membranas partilham. Até 2026-08-28 esta lia só `override → default`, e o `eval` do
        // nó monta a MESMA chave por `ctx.param` — que resolve o conduzido. ⇒ conduzir
        // qualquer um dos oito params fazia as duas chaves DIVERGIREM: o nó pedia uma análise
        // que ninguém publicou, `levels` vinha vazio, e ele emitia **um campo de zeros**. Todas
        // as bandas planas, sem erro nenhum.
        let p = super::motion_externals::resolved_params(motion, id, seconds, &MANIFEST);
        let spec = BandSpec::from_params(|name| p.get(name).copied().unwrap_or(0.0));
        let file = motion
            .doc
            .graph
            .node_text_params()
            .get(&id)
            .and_then(|m| m.get(FILE_KEY))
            .cloned()
            .unwrap_or_default();
        jobs.push((spec.key(&file), file, spec));
    }
    for (key, file, spec) in jobs {
        let levels = motion.band_cache.track(&key, &file, &spec).at(seconds);
        let stream = if levels.is_empty() {
            Stream::new(0)
        } else {
            Stream::new(levels.len()).with(VALUE_COL, Column::Scalar(levels.to_vec()))
        };
        motion.pump.cook.set_external(key, stream);
    }
}

#[cfg(test)]
#[path = "motion_audio_gen_tests.rs"]
mod tests;
