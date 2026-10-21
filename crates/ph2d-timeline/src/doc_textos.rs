//! ⭐ **Os TEXTOS das chaves de texto** (v22) — um campo `CampoInfo::texto` (a família da fonte, o
//! conteúdo de um texto) é um canal `f32` cujo valor é um índice em `ph2d_ecs::TextosDosCanais`
//! (`0` = nenhum, `k + 1` = o k-ésimo texto).
//!
//! **A lei — duas tabelas, cada uma com o seu dono:**
//! - **Na sessão** as chaves indexam o RECURSO, que só CRESCE enquanto o processo vive: um índice
//!   nunca muda de texto, então toda cópia do documento (a pilha de undo/redo, o cemitério, o
//!   clipboard) continua certa sem guardar tabela nenhuma — o campo `textos` fica VAZIO.
//! - **No ficheiro** o documento leva a SUA tabela, compactada aos textos que alguma chave usa
//!   ([`TimelineDoc::to_bytes_com_textos`]): o recurso acumula cada tecla do conteúdo, o ficheiro
//!   não. A compactação corre numa CÓPIA — remapear o documento vivo trocaria o texto das chaves
//!   que a pilha de undo guarda com os índices de antes.
//! - **Ao abrir**, o documento traz a tabela dele (pendente) até a [`TimelineDoc::adota_textos`]
//!   INTERNAR cada texto no recurso e remapear as chaves. ⛔ Nunca `TextosDosCanais::de` por cima
//!   do recurso: isso mudaria o texto de índices que alguém ainda guarda (o quadro anterior do
//!   auto-key, um documento do undo).
//!
//! Um valor de texto mora em três sítios: as chaves de cada clip, o `default` da trilha (o
//! `insert_key` copia-lhe a 1.ª chave) e o `rest` do binding (a base do fade).

use ph2d_ecs::TextosDosCanais;

use super::*;

impl TimelineDoc {
    /// A tabela que este documento trouxe do ficheiro e ainda não passou à sessão — vazia na
    /// sessão (ver o módulo).
    #[must_use]
    pub fn textos(&self) -> &[String] {
        &self.textos
    }

    /// ⭐ **Passa a tabela vinda do ficheiro para a da sessão**: interna cada texto em `sessao` e
    /// remapeia as chaves; o documento fica a indexar a sessão. Devolve `true` se havia tabela.
    /// Idempotente, e certa para qualquer documento pendente — também um que o undo devolva.
    pub fn adota_textos(&mut self, sessao: &mut TextosDosCanais) -> bool {
        if self.textos.is_empty() {
            return false;
        }
        let de = TextosDosCanais::de(std::mem::take(&mut self.textos));
        self.retabela(&de, sessao);
        true
    }

    /// ⭐ **Os bytes do ficheiro**, com a tabela COMPACTADA aos textos que as chaves usam e as
    /// chaves remapeadas para ela — numa cópia: o documento vivo e `sessao` ficam intocados.
    /// `sessao` é a tabela que as chaves indexam (ignorada se o documento ainda traz a sua).
    pub fn to_bytes_com_textos(&self, sessao: &TextosDosCanais) -> Result<Vec<u8>, String> {
        let (bytes, perdidos) = self.bytes_compactados(sessao)?;
        debug_assert_eq!(
            perdidos, 0,
            "chave de texto com um índice que a sessão não tem"
        );
        Ok(bytes)
    }

    /// A porta de [`Self::to_bytes`]: sem tabela da sessão, recusa perder o texto de uma chave.
    pub(super) fn to_bytes_sem_sessao(&self) -> Result<Vec<u8>, String> {
        let (bytes, perdidos) = self.bytes_compactados(&TextosDosCanais::default())?;
        if perdidos > 0 {
            return Err(format!(
                "{perdidos} valor(es) de texto sem tabela: grave por to_bytes_com_textos"
            ));
        }
        Ok(bytes)
    }

    fn bytes_compactados(&self, sessao: &TextosDosCanais) -> Result<(Vec<u8>, usize), String> {
        let ser = |d: &TimelineDoc| postcard::to_allocvec(d).map_err(|e| e.to_string());
        if self.textos.is_empty() && !self.bindings.iter().any(|b| b.prop.is_text()) {
            return Ok((ser(self)?, 0));
        }
        let proprio;
        let de = if self.textos.is_empty() {
            sessao
        } else {
            proprio = TextosDosCanais::de(self.textos.clone());
            &proprio
        };
        let mut copia = self.clone();
        let mut tabela = TextosDosCanais::default();
        let perdidos = copia.retabela(de, &mut tabela);
        copia.textos = tabela.textos().to_vec();
        Ok((ser(&copia)?, perdidos))
    }

    /// Um documento acabado de ler: a tabela passa a ser exactamente a dos textos que as chaves
    /// usam, e um índice que ela não tem vira «nenhum» — nunca o texto de outra tabela.
    pub(super) fn normaliza_textos(&mut self) {
        if self.textos.is_empty() && !self.bindings.iter().any(|b| b.prop.is_text()) {
            return;
        }
        let de = TextosDosCanais::de(std::mem::take(&mut self.textos));
        let mut tabela = TextosDosCanais::default();
        self.retabela(&de, &mut tabela);
        self.textos = tabela.textos().to_vec();
    }

    /// Reescreve cada valor de texto `v` como `para.indice(de.texto(v))`, numa ordem fixa (chaves
    /// por clip, o `default` da trilha, depois o `rest`) — a mesma ordem dá a mesma tabela. Devolve
    /// quantos valores não-nulos `de` não nomeava (esses viram «nenhum»).
    fn retabela(&mut self, de: &TextosDosCanais, para: &mut TextosDosCanais) -> usize {
        let mut perdidos = 0;
        let mut mapa = |v: f32| {
            let s = de.texto(v);
            if s.is_none() && v.round() != 0.0 {
                perdidos += 1;
            }
            para.indice(s)
        };
        let alvos: Vec<AnimTarget> = self
            .bindings
            .iter()
            .filter(|b| b.prop.is_text())
            .map(|b| b.target)
            .collect();
        for c in &mut self.clips {
            for &alvo in &alvos {
                let Some(track) = c.clip.track_mut(alvo) else {
                    continue;
                };
                for id in track.ids().to_vec() {
                    if let Some(AnimValue::Float(v)) = track.key(id).map(|k| k.value) {
                        track.set_value(id, AnimValue::Float(mapa(v)));
                    }
                }
                if let AnimValue::Float(d) = track.default_value() {
                    let t = std::mem::replace(track, Track::new(Vec::new()));
                    *track = t.with_default(AnimValue::Float(mapa(d)));
                }
            }
        }
        for b in self.bindings.iter_mut().filter(|b| b.prop.is_text()) {
            b.rest = b.rest.map(&mut mapa);
        }
        perdidos
    }
}
