//! ⭐ **Salvar → recarga → selo, e o console** (CodeEditor Onda 1, E6 + E10 — o MOTOR).
//!
//! Tudo pelo caminho do produto: o texto vive na `ProjectScripts` do mundo (o editor escreve-o lá ao
//! salvar), o `scene_sync` de cada quadro apanha-o, e o `scene_tick` corre os ganchos. Sem disco,
//! sem relógio de parede (o `dt` é o do teste).

use ph2d_ecs::{Entity, SimWorld, StableId, Transform};
use ph2d_script::{
    CONSOLE_LINES_MAX, ConsoleKind, ConsoleLine, LuauScript, ProjectScripts, ReloadOutcome,
    ScriptHost, ScriptId, ScriptInfo, ScriptReload,
};

/// `update` soma `passo` ao `x` em cada tique — o «valor de uma propriedade usada no update».
fn soma(passo: u32) -> String {
    format!(
        "ph2d.property(\"passo\", {passo})\n\
         function update(self, dt)\n\
         \x20 ph2d.set(self.id, \"x\", ph2d.get(self.id, \"x\") + self.passo)\n\
         end\n"
    )
}

/// O mesmo script com o `end` do `update` apagado — o smoke do dono.
fn sem_end(passo: u32) -> String {
    let t = soma(passo);
    let i = t.rfind("end\n").expect("tem end");
    format!("{}{}", &t[..i], &t[i + 4..])
}

struct Cena {
    sim: SimWorld,
    host: ScriptHost,
    id: ScriptId,
    objs: Vec<Entity>,
}

/// `n` objectos com pose e identidade, todos no MESMO script da biblioteca.
fn cena(texto: &str, n: u64) -> Cena {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let id = w
        .get_resource_or_init::<ProjectScripts>()
        .insert("bob.luau", texto);
    let objs = (1..=n)
        .map(|sid| {
            w.spawn((Transform::IDENTITY, StableId(sid), LuauScript::library(id)))
                .id()
        })
        .collect();
    Cena {
        sim,
        host: ScriptHost::new().expect("vm"),
        id,
        objs,
    }
}

impl Cena {
    /// Um quadro do produto: o `sync` (recarga) e um passo fixo.
    fn quadro(&mut self) {
        self.host.scene_sync(self.sim.world_mut());
        let r = self.host.scene_tick(self.sim.world_mut(), 1.0 / 60.0);
        assert!(r.failed.is_empty(), "{:?}", r.failed);
    }

    /// Um quadro sem exigir que nada falhe.
    fn quadro_livre(&mut self) -> ph2d_script::SceneReport {
        self.host.scene_sync(self.sim.world_mut());
        self.host.scene_tick(self.sim.world_mut(), 1.0 / 60.0)
    }

    /// O editor SALVA: escreve o texto na biblioteca (sem disco, sem mtime).
    fn salva(&mut self, texto: &str) {
        let id = self.id;
        assert!(
            self.sim
                .world_mut()
                .resource_mut::<ProjectScripts>()
                .set_text(id, texto)
        );
    }

    fn xs(&self) -> Vec<f32> {
        self.objs
            .iter()
            .map(|e| {
                self.sim
                    .world()
                    .get::<Transform>(*e)
                    .expect("pose")
                    .translation
                    .x
            })
            .collect()
    }
}

/// ⭐⭐ **Apagar um `end` não pára a cena**: os três continuam a correr a versão anterior, e o
/// diagnóstico com a linha chega pela MESMA porta que o cartão lê (`scene_info`).
#[test]
fn um_texto_que_nao_compila_deixa_a_versao_anterior_a_correr() {
    let mut c = cena(&soma(1), 3);
    c.quadro();
    assert_eq!(c.xs(), [1.0; 3]);
    assert!(
        c.host.drain_reloads().is_empty(),
        "a 1.ª leitura não é recarga"
    );
    c.salva(&sem_end(5));
    c.quadro();
    let [r] = c.host.drain_reloads().try_into().expect("um registo");
    let ScriptReload {
        script,
        outcome: ReloadOutcome::Refused(d),
        objects,
    } = r
    else {
        panic!("{r:?}");
    };
    assert_eq!((script, objects, d.line), (c.id, 3, Some(4)), "{d:#}");
    assert!(c.host.scene().runs_previous(c.id));
    assert_eq!(c.xs(), [2.0; 3], "a lei VELHA (+1) continua, nos três");
    c.quadro();
    assert_eq!(c.xs(), [3.0; 3], "e no passo seguinte também");
    let Some(ScriptInfo::Broken(d)) = c.host.scene_info(c.id) else {
        panic!("{:?}", c.host.scene_info(c.id));
    };
    assert_eq!(d.line, Some(4), "o <eof> depois da linha 3 — {d:#}");
    // A recusa também chega ao console, com a linha.
    let recusa = c.host.drain_console();
    assert!(
        matches!(&recusa[..], [ConsoleLine { kind: ConsoleKind::ReloadRefused(d), entity: None, .. }]
            if d.line == Some(4)),
        "{recusa:?}"
    );
    // Controlo: corrigir ⇒ recarrega, e a lei NOVA corre sobre o mesmo `self`.
    c.salva(&soma(5));
    c.quadro();
    assert_eq!(c.xs(), [8.0; 3]);
    assert!(matches!(
        c.host.scene_info(c.id),
        Some(ScriptInfo::Ready(_))
    ));
    assert!(!c.host.scene().runs_previous(c.id));
    let [r] = c.host.drain_reloads().try_into().expect("um registo");
    assert_eq!(
        (r.outcome, r.objects),
        (ReloadOutcome::Reloaded, 3),
        "o controlo volta a recarregar"
    );
}

/// ⭐ **Salvar ⇒ «Reloaded · 3 objects»**, e os três correm o valor novo no passo seguinte. O texto
/// chega pela biblioteca (sem disco nem mtime) e a ordem dos registos é a do `ScriptId`.
#[test]
fn salvar_recarrega_os_tres_e_o_relatorio_diz_tres() {
    let mut c = cena(&soma(1), 3);
    // Um 2.º script com um objecto, para a ordem.
    let outro = c
        .sim
        .world_mut()
        .resource_mut::<ProjectScripts>()
        .insert("outro.luau", soma(1));
    c.sim
        .world_mut()
        .spawn((Transform::IDENTITY, StableId(9), LuauScript::library(outro)));
    c.quadro();
    assert_eq!(c.xs(), [1.0; 3]);
    assert_eq!(
        (c.host.objects_using(c.id), c.host.objects_using(outro)),
        (3, 1)
    );
    // Gravados por ordem INVERSA da do id.
    assert!(
        c.sim
            .world_mut()
            .resource_mut::<ProjectScripts>()
            .set_text(outro, &soma(2))
    );
    c.salva(&soma(10));
    c.quadro();
    assert_eq!(c.xs(), [11.0; 3], "os três com a lei nova (+10)");
    let rs = c.host.drain_reloads();
    assert_eq!(
        rs,
        [
            ScriptReload {
                script: c.id,
                outcome: ReloadOutcome::Reloaded,
                objects: 3
            },
            ScriptReload {
                script: outro,
                outcome: ReloadOutcome::Reloaded,
                objects: 1
            },
        ]
    );
    // Nada mudou ⇒ nada a dizer; o pedido explícito recarrega o mesmo texto.
    c.quadro();
    assert!(c.host.drain_reloads().is_empty());
    c.host.request_reload(c.id);
    c.quadro();
    assert_eq!(c.host.drain_reloads().len(), 1);
    assert_eq!(c.xs(), [31.0; 3], "recarregar não renasce: o `x` segue");
}

/// Um script que NUNCA compilou não tem versão boa: ninguém corre, e o diagnóstico está lá.
#[test]
fn um_script_que_nunca_compilou_nao_corre() {
    let mut c = cena(&sem_end(1), 2);
    c.host.scene_sync(c.sim.world_mut());
    let r = c.host.scene_tick(c.sim.world_mut(), 1.0 / 60.0);
    assert_eq!((r.calls, c.xs()), (0, vec![0.0; 2]));
    assert_eq!(c.host.scene().live_count(), 0);
    assert!(matches!(
        c.host.scene_info(c.id),
        Some(ScriptInfo::Broken(d)) if d.line.is_some()
    ));
}

/// ⭐ **`print(\"oi\", 1, nil)` chega ao console** como o `print` do Luau o escreveria, com o script e
/// o objecto que falaram.
#[test]
fn o_print_vai_para_o_console_com_o_script_certo() {
    let mut c = cena("function update(self, dt) print(\"oi\", 1, nil) end\n", 1);
    c.quadro();
    assert_eq!(
        c.host.drain_console(),
        [ConsoleLine {
            script: c.id,
            entity: Some(c.objs[0]),
            text: "oi\t1\tnil".into(),
            kind: ConsoleKind::Print,
        }]
    );
    assert!(c.host.drain_console().is_empty(), "drenar esvazia");
}

/// ⭐ **Um erro de runtime num `update` entra no console com a LINHA do script.**
#[test]
fn um_erro_de_runtime_entra_no_console_com_a_linha() {
    let mut c = cena(
        "function update(self, dt)\n  local t = nil\n  print(t.x)\nend\n",
        1,
    );
    let r = c.quadro_livre();
    assert_eq!(r.failed.len(), 1);
    let linhas = c.host.drain_console();
    let [
        ConsoleLine {
            script,
            entity,
            kind: ConsoleKind::RuntimeError(d),
            text,
        },
    ] = &linhas[..]
    else {
        panic!("{linhas:?}");
    };
    assert_eq!((*script, *entity, d.line), (c.id, Some(c.objs[0]), Some(3)));
    assert!(text.starts_with("line 3: "), "{text}");
    // Parado não repete: o console não cresce com um erro por quadro.
    c.quadro_livre();
    assert!(c.host.drain_console().is_empty());
}

/// ⭐ **O tecto do console**: cheio, a linha mais antiga sai e conta — a memória não cresce com
/// um script que imprime em todo quadro e ninguém a drenar.
#[test]
fn o_console_cheio_deita_fora_a_mais_antiga_e_conta() {
    const POR_QUADRO: usize = 256;
    let mut c = cena(
        &format!(
            "local n = 0\nfunction update(self, dt)\n  for i = 1, {POR_QUADRO} do n = n + 1 ; print(n) end\nend\n"
        ),
        1,
    );
    let quadros = CONSOLE_LINES_MAX / POR_QUADRO + 2;
    for _ in 0..quadros {
        c.quadro();
    }
    let total = quadros * POR_QUADRO;
    let linhas = c.host.drain_console();
    assert_eq!(linhas.len(), CONSOLE_LINES_MAX);
    assert_eq!(c.host.console_dropped(), (total - CONSOLE_LINES_MAX) as u64);
    assert_eq!(
        linhas[0].text,
        (total - CONSOLE_LINES_MAX + 1).to_string(),
        "sai a mais antiga"
    );
    assert_eq!(linhas[CONSOLE_LINES_MAX - 1].text, total.to_string());
}

/// 📏 **A medição do tecto** (corre à mão, `--ignored --nocapture`): a cena pior do plano — 1 000
/// objectos a fazer `print` em cada passo — durante 60 passos (1 s a 60 Hz), e os bytes por linha.
#[test]
#[ignore = "medição: corre à mão com --nocapture"]
fn mede_o_tecto_do_console() {
    let mut c = cena(
        "function update(self, dt) print(\"oi\", 1, nil) end\n",
        1_000,
    );
    for _ in 0..60 {
        c.quadro();
    }
    let linhas = c.host.drain_console();
    let inline = std::mem::size_of::<ConsoleLine>();
    let heap: usize = linhas.iter().map(|l| l.text.capacity()).sum();
    let por_linha = inline as f64 + heap as f64 / linhas.len() as f64;
    eprintln!(
        "linhas em 60 passos × 1000 objectos: {} · size_of::<ConsoleLine>() = {inline} B · \
         texto médio {:.1} B · ≈ {por_linha:.0} B/linha (sem o cabeçalho do malloc) · \
         tecto {CONSOLE_LINES_MAX} ⇒ ≈ {:.1} MB",
        linhas.len(),
        heap as f64 / linhas.len() as f64,
        por_linha * CONSOLE_LINES_MAX as f64 / 1e6
    );
}

/// O marcador que o filho imprime — único o bastante para não casar com nada do libtest.
const MARCA: &str = "marca-do-print-7f3a";

/// ⚠️ **Só corre como FILHO** de [`o_print_nao_vai_para_o_stdout`]: o `print` do Luau escreve pelo
/// C (`fwrite`), por baixo da captura do libtest — só o stdout de OUTRO processo o vê.
#[test]
#[ignore = "processo filho de o_print_nao_vai_para_o_stdout"]
fn filho_que_faz_print() {
    let mut c = cena(
        &format!("function update(self, dt) print(\"{MARCA}\", 1, nil) end\n"),
        1,
    );
    c.quadro();
}

/// ⭐ **`print` não vai para o stdout** — o processo filho corre um `update` com `print` e o stdout
/// dele não tem a marca. Controlo: o filho correu UM teste (um filtro que casa zero passaria).
#[test]
fn o_print_nao_vai_para_o_stdout() {
    let out = std::process::Command::new(std::env::current_exe().expect("exe"))
        .args([
            "--exact",
            "a_recarga_e_o_console::filho_que_faz_print",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .output()
        .expect("filho");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{stdout}");
    assert!(
        stdout.contains("1 passed"),
        "o filho correu o teste: {stdout}"
    );
    assert!(
        !stdout.contains(MARCA),
        "o print chegou ao stdout: {stdout}"
    );
}

impl Cena {
    /// ⭐ **Abrir OUTRO projecto** pela porta do load (`ProjectScripts::install`): o mundo troca de
    /// objectos e os `ScriptId` recomeçam em 1 — o `#1` do projecto novo é OUTRO script.
    fn abre_outro_projecto(&mut self, texto: &str, n: u64) {
        let w = self.sim.world_mut();
        for e in std::mem::take(&mut self.objs) {
            w.despawn(e);
        }
        let mut lib = ph2d_script::ScriptLibrary::default();
        let id = lib.insert("enemy.luau", texto);
        assert_eq!(id, self.id, "o #1 do projecto novo tem o MESMO id");
        ProjectScripts::install(w, lib);
        self.objs = (1..=n)
            .map(|sid| {
                w.spawn((Transform::IDENTITY, StableId(sid), LuauScript::library(id)))
                    .id()
            })
            .collect();
    }
}

/// ⛔⛔ **O projecto novo nunca corre o código do anterior** (auditoria do fecho): o `#1` partido do
/// projecto B não pode cair na «versão anterior que continua a correr» — essa era a do projecto A.
#[test]
fn abrir_outro_projecto_esquece_a_versao_anterior_do_mesmo_id() {
    let mut c = cena(&soma(1), 2);
    c.quadro();
    assert_eq!(c.xs(), [1.0; 2]);
    let _ = c.host.drain_reloads();
    c.abre_outro_projecto(&sem_end(5), 2);
    let _ = c.quadro_livre();
    assert!(
        !c.host.scene().runs_previous(c.id),
        "o #1 partido do projecto B corre o código do projecto A"
    );
    assert_eq!(c.xs(), [0.0; 2], "os objectos do B correram o update do A");
    assert!(
        c.host.drain_reloads().is_empty(),
        "a 1.ª leitura do B não é uma recarga do A"
    );
}

/// ⛔ E um `#1` bom no projecto B é a 1.ª leitura dele — nenhum «Reloaded» espúrio.
#[test]
fn abrir_outro_projecto_nao_relata_uma_recarga() {
    let mut c = cena(&soma(1), 2);
    c.quadro();
    let _ = c.host.drain_reloads();
    c.abre_outro_projecto(&soma(5), 3);
    c.quadro();
    assert_eq!(c.xs(), [5.0; 3], "o B corre o texto dele");
    assert!(
        c.host.drain_reloads().is_empty(),
        "o selo diria «Reloaded» sobre um projecto acabado de abrir"
    );
}
