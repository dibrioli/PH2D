//! Os problemas do Play de uma aba ancorados ao texto que correu; a fusão com os do texto e o «On»
//! vivem em [`ph2d_code_assist::play_problems`].

pub(crate) use ph2d_code_assist::play_problems::merge;

impl crate::doc::CodeDoc {
    /// Os problemas do Play cuja linha ainda tem o texto que correu, na linha onde as edições a
    /// levaram (desfazer o conserto devolve-os: são derivados, não apagados).
    pub(crate) fn live_play(&self) -> Vec<crate::console::PlayProblem> {
        let t = self.buffer.text();
        self.play
            .1
            .iter()
            .filter_map(|g| {
                let Some(n) = g.line else {
                    return Some(g.clone());
                };
                let a = self.play_at.iter().find(|a| a.is(g, n))?;
                let line = t.line_of_byte(a.byte.min(t.len()));
                (t.line_string(line) == a.text).then(|| {
                    let mut g = g.clone();
                    g.line = Some(line as u32 + 1);
                    g
                })
            })
            .collect()
    }

    /// Os problemas do Play guardados para este script (`generation` muda quando eles mudam).
    pub(crate) fn set_play(&mut self, store: Option<&crate::console::Store>) {
        let generation = store.map_or(0, |s| s.play_generation);
        if generation != self.play.0 {
            let list: Vec<_> = store.map(|s| s.play().to_vec()).unwrap_or_default();
            self.anchor_play(&list);
            self.play = (generation, list.into());
        }
    }

    /// Ancora os problemas do Play novos; os já ancorados guardam a âncora (já seguiu as edições).
    /// O texto da linha vem do `synced` (o que o Play corre). Um que chega com a aba suja ancora-se
    /// na MESMA linha do buffer — as edições de antes dele não têm mapa; se ela mudou, não aparece.
    fn anchor_play(&mut self, list: &[crate::console::PlayProblem]) {
        let old = std::mem::take(&mut self.play_at);
        let t = self.buffer.text();
        for g in list {
            let Some(n) = g.line else { continue };
            if let Some(a) = old.iter().find(|a| a.is(g, n)) {
                self.play_at.push(a.clone());
                continue;
            }
            let line = (n as usize).saturating_sub(1);
            let Some(text) = self.synced.split('\n').nth(line) else {
                continue;
            };
            self.play_at.push(PlayAt {
                // A âncora nasce no buffer de agora; o texto, no que correu.
                kind: g.kind,
                line: n,
                message: g.message.clone(),
                byte: t.byte_of_line(line.min(t.line_count().saturating_sub(1))),
                text: text.strip_suffix('\r').unwrap_or(text).to_owned(),
            });
        }
    }
}

/// A âncora de um problema do Play ([`crate::doc::CodeDoc::play_at`]): a identidade do grupo
/// (espécie, linha do texto que correu, mensagem), o byte do início da linha no buffer (segue as
/// edições em `CodeDoc::edited`) e o texto dela.
#[derive(Clone, Debug)]
pub(crate) struct PlayAt {
    kind: crate::console::PlayKind,
    line: u32,
    message: String,
    pub(crate) byte: usize,
    text: String,
}

impl PlayAt {
    fn is(&self, g: &crate::console::PlayProblem, line: u32) -> bool {
        self.kind == g.kind && self.line == line && self.message == g.message
    }
}

#[cfg(test)]
mod tests {
    use ph2d_script::{ConsoleKind, ConsoleLine, DiagnosticKind, ScriptDiagnostic, ScriptId};

    /// O «On» de objectos sem nome conta-os («2 objects»), nunca « +2» sozinho.
    #[test]
    fn o_on_sem_nomes_conta_os_objectos() {
        let mut s = crate::console::Store::default();
        for bits in [7, 8] {
            let d = ScriptDiagnostic {
                kind: DiagnosticKind::Runtime,
                line: Some(2),
                message: "boom".to_owned(),
                traceback: None,
            };
            let l = ConsoleLine {
                script: ScriptId(1),
                entity: Some(ph2d_ecs::Entity::from_bits(bits)),
                text: String::new(),
                kind: ConsoleKind::RuntimeError(d),
                at_us: 0,
            };
            s.push(&l, None);
        }
        assert_eq!(s.play().len(), 1, "controlo: um problema agrupado");
        assert_eq!(
            ph2d_code_assist::play_problems::who(&s.play()[0]),
            "2 objects"
        );
    }
}
