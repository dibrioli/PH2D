//! ⭐⭐⭐ **A ORDEM DO QUADRO do suplente #24** — a metade que FICOU na shell quando a ponte da
//! tabela de acções desceu para a família (2026-09-19).
//!
//! # ⚠️ Porque ela fica, e a irmã não
//!
//! Os gates da LEI viajaram com o código que medem ([`ph2d_app_components::signal_actions_bridge`]).
//! Este mede a **FASE** — *a leitura passa pela porta com nome · as mortes chegam ao despachante · a
//! tabela recebe DISPAROS* —, e a fase é composição desta shell. ⛔ *Um gate que CHAMA as funções
//! afirma que as peças existem, nunca que a fase as usa*: a lei que esta casa pagou quatro vezes.
//!
//! ⚠️ **`include_str!` e não um `grep` em runtime** — ele deixa de COMPILAR se o ficheiro mudar de
//! sítio, que é o modo de falha bom (HOWTO §2.2: o gémeo em runtime só falha quando o teste corre).

/// Ver o cabeçalho.
///
/// **Mutações que devem sangrar:** a fase a voltar ao `.map(|s| s.name.to_string())` · apagar o
/// `deaths.extend(r.mortes)` · o `resolve` a receber nomes outra vez.
#[test]
fn a_fase_le_pela_porta_e_manda_as_mortes_ao_despachante() {
    const FASE: &str = include_str!("fase_tabela_de_accoes.rs");
    assert!(
        FASE.contains(".map(signal_actions::lido)"),
        "a fase deixou de ler pela porta com nome — a origem volta a morrer no `map`"
    );
    assert!(
        FASE.contains("deaths.extend(r.mortes)"),
        "as mortes do `Destroy` deixaram de chegar ao despachante"
    );
    assert!(
        FASE.contains("resolve_signal_actions(sim.world_mut(), tags, &disparos)"),
        "a tabela voltou a receber so' nomes"
    );
    // ⛔ O CONTROLO da própria régua: sem isto, um `include_str!` que apontasse a um ficheiro vazio
    // (ou a um que deixasse de existir e fosse recriado) passaria as três asserções por vácuo.
    assert!(
        FASE.len() > 2_000,
        "o ficheiro da fase encolheu para {} bytes — a regua esta' a medir outra coisa",
        FASE.len()
    );
}
