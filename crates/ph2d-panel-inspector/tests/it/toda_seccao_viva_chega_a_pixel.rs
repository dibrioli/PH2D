//! ⭐⭐⭐ **TODA secção viva do Inspector chega a PIXEL** — o censo que o handoff do `#15` encomendou
//! por escrito (*«nenhuma das secções opcionais tem gate a provar que ela chega a PIXEL»*).
//!
//! # ⚠️⚠️ A primeira coisa que este ficheiro fez foi CORRIGIR o número daquela nota
//!
//! Ela dizia **26**. Medido: são **4** de `38` — as secções do `Weapon`, do `Shake`, do `Tween`, do
//! `Ray Sensor` e mais trinta ganharam gate de pintura nas waves em que nasceram, e a nota nunca
//! foi re-medida. ⛔ *Uma lista de dívida citada de cor manda reconstruir trabalho já pago, e
//! assusta com um número que ninguém contou.*
//!
//! # ⛔⛔ E a 1.ª redacção deste censo leu `0`, porque ele ENCONTRAVA-SE A SI PRÓPRIO
//!
//! A varredura lê os ficheiros de teste à procura do nome de cada semeador — e o seu PRÓPRIO
//! ficheiro contém os quatro nomes, como literais da catraca. *Um censo textual que se lê a si
//! mesmo encontra sempre o que procura*, e a catraca lia-se toda obsoleta.
//!
//! ⇒ ele **salta-se**, e isso é uma linha com nome ([`EU_PROPRIO`]). ⚠️ A cura irmã — montar a
//! agulha em runtime — já é usada em [`semeadores`] e **não chega aqui**: o que se procura não é
//! uma agulha construída, são os literais da lista de dívida.
//!
//! ⛔⛔ **E isso tem uma consequência estrutural que a wave seguinte pagou na hora: um censo que se
//! SALTA não pode conter o gate que ele CONTA.** As quatro fixturas que fecharam a catraca nasceram
//! neste ficheiro e o censo leu-as como inexistentes ⇒ elas vivem no irmão
//! [`super::as_quatro_seccoes_que_estreavam_a_catraca`]. *A régua e o que ela mede não podem morar
//! no mesmo sítio.*
//!
//! # ⭐ A régua é o SEMEADOR do instantâneo, e não o id do cabeçalho
//!
//! Uma secção opcional só é pintada se alguém semear o instantâneo dela (`set_current_inspector_*`)
//! — *é isso que a torna opcional*. ⇒ **«uma secção sem gate» é exactamente «um semeador que teste
//! nenhum chama»**, e essa pergunta responde-se sobre o texto dos testes, sem um painel montado.
//!
//! ⚠️ A régua do id do CABEÇALHO seria mais fraca e mais fácil de satisfazer: as irmãs
//! (`a_seccao_*_esta_viva`) medem os CAMPOS, que é mais forte, e nenhuma delas nomeia o id do
//! cabeçalho. *Uma régua que as declarasse em falta estaria a medir a régua, não o produto.*

use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 4000.0,
};

const ENTITY: u64 = 0x5EC0_0001;

/// Os ficheiros onde um gate pode semear uma secção. ⚠️ **Os três directórios, e não só o desta
/// crate:** a shell e o `ph2d-editor-core` também têm gates que pintam o Inspector, e limitar a
/// varredura a um deles acusaria de órfãs secções que estão cobertas noutro sítio.
const ONDE_OS_GATES_VIVEM: [&str; 3] = [
    "crates/ph2d-panel-inspector/tests",
    "crates/ph2d-editor-core/tests",
    "shells/desktop/tests",
];

/// ⛔⛔ **O ficheiro deste censo, que ele TEM de saltar** — ver o cabeçalho.
const EU_PROPRIO: &str = "toda_seccao_viva_chega_a_pixel.rs";

/// ⚠️⚠️ **A SENTINELA que torna o salto observável.**
///
/// Com a catraca VAZIA o salto é **inerte** — ele existe para o dia em que ela voltar a ter
/// entradas, e uma mutação que o apague não sangra em corpus nenhum. *Uma cerca que o produto não
/// exercita é uma cerca sem régua*, e a cura é a mesma que a `ph2d-quadflow` deu ao veto dela:
/// construir o caso que ela recusa.
///
/// ⭐ Ela é um literal que existe **só neste ficheiro** e em mais lado nenhum da árvore de testes.
const SENTINELA_DO_SALTO: &str = "set_current_inspector_ESTA_SENTINELA_NAO_EXISTE";

/// A raiz do repositório, a partir desta crate.
fn raiz() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("a raiz do repo")
}

/// Todo o texto dos ficheiros de teste que podem semear uma secção.
fn texto_dos_gates() -> String {
    let mut out = String::new();
    for dir in ONDE_OS_GATES_VIVEM {
        let mut pilha = vec![raiz().join(dir)];
        while let Some(d) = pilha.pop() {
            let Ok(rd) = std::fs::read_dir(&d) else {
                continue;
            };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    pilha.push(p);
                } else if p.extension().is_some_and(|x| x == "rs")
                    && p.file_name().is_some_and(|n| n != EU_PROPRIO)
                {
                    out.push_str(&std::fs::read_to_string(&p).unwrap_or_default());
                    out.push('\n');
                }
            }
        }
    }
    assert!(
        out.len() > 10_000,
        "piso de populacao: a varredura leu {} bytes de teste — um caminho errado devolve pouco \
         e o censo fica VERDE a medir nada",
        out.len()
    );
    out
}

/// Os semeadores que a crate publica, lidos do ficheiro que os declara.
///
/// ⚠️ **DERIVADO e não uma lista à mão:** um semeador novo entra no censo sozinho, que é a
/// diferença entre uma lista que alguém tem de se lembrar de estender e uma que não fica verde sem
/// a extensão.
fn semeadores() -> Vec<String> {
    let mut out = Vec::new();
    let mut pilha = vec![raiz().join("crates/ph2d-panel-inspector/src")];
    while let Some(d) = pilha.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                pilha.push(p);
                continue;
            }
            let Ok(t) = std::fs::read_to_string(&p) else {
                continue;
            };
            for l in t.lines() {
                if let Some(r) = l.trim().strip_prefix("pub fn set_current_inspector_") {
                    let nome: String = r
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    out.push(format!("set_current_inspector_{nome}"));
                }
            }
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// ⛔⛔ **A CATRACA — e ela só ENCOLHE. Hoje está VAZIA.**
///
/// Cada entrada seria uma secção que nenhum gate PINTA. ⚠️ **A cura é escrever o gate, nunca
/// acrescentar uma linha aqui** — e foi o que esta wave fez às quatro que a estreavam (`Factory`,
/// `Projectile`, `State Machine`, `Top-Down`), cujo preço era o mesmo: um `Info` de quinze a vinte
/// campos **sem `Default`**.
///
/// ⭐ *Uma catraca vazia é a mais apertada que existe:* já não há linha onde escrever uma secção
/// que nasça sem régua.
const SEM_GATE_QUE_PINTA: [&str; 0] = [];

/// ⭐⭐⭐ **O CENSO: nenhuma secção NOVA nasce sem gate que a pinte.**
///
/// **Mutações que devem sangrar:** apagar um `set_current_inspector_*` de um gate existente ·
/// apontar a varredura a um directório errado (o piso de população apanha-o).
#[test]
fn toda_seccao_viva_chega_a_pixel() {
    let texto = texto_dos_gates();
    let todos = semeadores();
    assert!(
        todos.len() >= 30,
        "piso de populacao: so' {} semeadores — a extraccao partiu-se",
        todos.len()
    );
    let sem: Vec<&String> = todos
        .iter()
        .filter(|s| !texto.contains(s.as_str()))
        .collect();
    let novos: Vec<&&String> = sem
        .iter()
        .filter(|s| !SEM_GATE_QUE_PINTA.contains(&s.as_str()))
        .collect();
    assert!(
        novos.is_empty(),
        "estas seccoes nao sao PINTADAS por gate nenhum, e nao estao na catraca: {novos:?}\n\
         a cura e' escrever o gate (o molde e' `a_seccao_weapon_esta_viva`), NUNCA uma linha nova \
         em `SEM_GATE_QUE_PINTA`"
    );
}

/// ⚠️ **A metade da OBSOLESCÊNCIA, sem a qual a catraca vira LICENÇA** — uma entrada que já não
/// descreve nada tem de sair.
///
/// ⭐ E ela imprime o placar, que é o que torna o `26 → 4` desta nota verificável por quem vier a
/// seguir em vez de acreditado.
#[test]
fn a_catraca_das_seccoes_sem_gate_nao_tem_entradas_obsoletas() {
    let texto = texto_dos_gates();
    let todos = semeadores();
    println!(
        "seccoes com semeador: {} · sem gate que pinte: {}",
        todos.len(),
        todos.iter().filter(|s| !texto.contains(s.as_str())).count()
    );
    for entrada in SEM_GATE_QUE_PINTA {
        assert!(
            todos.iter().any(|s| s == entrada),
            "«{entrada}» ja' nao e' um semeador desta crate — a entrada da catraca e' LIXO"
        );
        assert!(
            !texto.contains(entrada),
            "«{entrada}» JA' e' semeado por um gate — apague a linha da catraca, ela so' encolhe"
        );
    }
}

/// ⭐ **O CONTROLO da régua:** uma secção que TEM gate é mesmo pintada quando o instantâneo é
/// semeado — senão o censo acima estaria a medir texto e a afirmar pixels.
///
/// ⚠️ A `Weapon` é a testemunha porque o gate dela é o molde que a mensagem do censo aponta.
#[test]
fn o_controlo_da_regua_uma_seccao_semeada_e_de_facto_pintada() {
    use ph2d_editor_core::weapon_edits::InspectorWeaponInfo;
    use ph2d_panel_inspector::{ids, set_current_inspector_weapon};

    let mut h = MockPanelHost::with_panel::<InspectorPanel>();
    let mut st = InspectorState::default();
    set_current_inspector_weapon(Some(InspectorWeaponInfo {
        entity_bits: ENTITY,
        on_signal: "fire".into(),
        cooldown_ms: 250,
        ammo_counter: "ammo".into(),
        reload_ms: 800,
        reload_on: "reload".into(),
        on_fire: "shot".into(),
        on_empty: "click".into(),
        on_reloaded: "ready".into(),
        reserve_counter: "box".into(),
        reserva: Some(9),
        municao: Some(4),
        pente: 6,
        recarregando: false,
        clock_playing: true,
        selected_count: 1,
    }));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let achou = rects.iter().any(|(n, _)| *n == ids::INSP_WEAPON_ON_SIGNAL);
    set_current_inspector_weapon(None);
    assert!(
        achou,
        "semear o instantaneo da arma tem de PINTAR os campos dela — sem isto o censo acima mede \
         texto e afirma pixels"
    );
}

/// ⭐⭐ **O salto é observável** — a cerca do [`SENTINELA_DO_SALTO`], que a catraca vazia tornou
/// inerte.
///
/// **Mutação que deve sangrar:** tirar a condição `n != EU_PROPRIO` da varredura.
#[test]
fn o_censo_salta_se_a_si_proprio() {
    assert!(
        !texto_dos_gates().contains(SENTINELA_DO_SALTO),
        "a varredura leu o PRO'PRIO ficheiro do censo — com a catraca cheia, cada entrada dela \
         contaria como semeadura e a lista de divida leria-se toda obsoleta"
    );
}
