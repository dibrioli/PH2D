//! **Fase do quadro: O DIAGNÓSTICO DOS SINAIS** — irmã da [`super::fase_signal_outbox`], cortada
//! dela pelo tecto de 200 LOC por função, **por responsabilidade**: aquela decide o que acontece,
//! esta só conta o que aconteceu.
//!
//! ⚠️⚠️ **O nome começa por `fase_` e isso é load-bearing:** o texto emendado do quadro
//! (`tests/it/frame_text.rs`) colhe **só** as `fn fase_*`, e uma fase-filha com outro nome
//! desaparece do oráculo de **toda** lei de ordem desta shell — em silêncio. A wave da fábrica
//! mediu-o e escreveu-o; aqui ele é honrado.
//!
//! ⚠️ **Ela corre por ÚLTIMO, depois de todo consumidor**, e é o que a torna um diagnóstico
//! honesto: o que ela imprime é o que os outros já viram.

impl crate::App {
    /// Ver o cabeçalho do módulo. No-op sem `PH2D_SIGNAL_LOG`.
    pub(super) fn fase_signal_log(&mut self) {
        if let Some(reader) = self.signal_readers.log.as_mut() {
            for sig in self.signals.read(reader) {
                match sig.origin {
                    ph2d_runtime::SignalOrigin::Timeline { t } => {
                        eprintln!("[signal] {} <- timeline @ {t:.3}s", sig.name);
                    }
                    ph2d_runtime::SignalOrigin::Contact { source, other } => {
                        eprintln!(
                            "[signal] {} <- fisica, {} tocou {}",
                            sig.name, source.0, other.0
                        );
                    }
                    ph2d_runtime::SignalOrigin::Control => {
                        eprintln!("[signal] {} <- controle autorado", sig.name);
                    }
                    ph2d_runtime::SignalOrigin::Motion { tick, rows } => {
                        eprintln!(
                            "[signal] {} <- grafo motion, tique {tick}, {rows} linha(s)",
                            sig.name
                        );
                    }
                    ph2d_runtime::SignalOrigin::Animation { source, cycles } => {
                        eprintln!(
                            "[signal] {} <- animacao da sprite {}, {cycles} ciclo(s)",
                            sig.name, source.0
                        );
                    }
                    ph2d_runtime::SignalOrigin::Timer { source, fires } => {
                        eprintln!(
                            "[signal] {} <- timer do objecto {}, {fires} periodo(s)",
                            sig.name, source.0
                        );
                    }
                    ph2d_runtime::SignalOrigin::Spawned { source, count } => {
                        eprintln!(
                            "[signal] {} <- fabrica {}, {count} copia(s) nasceram",
                            sig.name, source.0
                        );
                    }
                    ph2d_runtime::SignalOrigin::Death { source } => {
                        eprintln!("[signal] {} <- morreu a copia {}", sig.name, source.0);
                    }
                    ph2d_runtime::SignalOrigin::StateMachine { source } => {
                        eprintln!(
                            "[signal] {} <- o cerebro do objecto {} mudou de estado",
                            sig.name, source.0
                        );
                    }
                }
            }
        }
    }
}
