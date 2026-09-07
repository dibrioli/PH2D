//! ⭐⭐⭐ **A SONDA DO UNDO DA ÂNCORA** (`PH2D_BONE_UNDO_PROBE=1`) — o aparelho que o report
//! *«Undo não funciona para add IK»* (Enio, 2026-09-07) pede.
//!
//! # Por que ela existe, e por que a leitura do código não bastou
//!
//! Entre o clique e o passo há uma **máquina** com cinco motivos de supressão
//! ([`crate::App::post_frame_undo`]), e eles suprimem **igual**. Duas medições já foram feitas por
//! outro caminho e **ilibaram** a metade fácil:
//!
//! | medido | resultado |
//! |---|---|
//! | a fotografia do undo VÊ a âncora? | **sim** (`parts_that_differ` devolve `["world"]`) |
//! | o restauro leva-a embora? | **sim** (alvo despawnado, `IkGoal` fora) |
//!
//! ⇒ a máquina do snapshot está sã, e o que falta saber é se o **passo chega a nascer**. Isso
//! nenhum teste responde: `post_frame_undo` vive na `App`, que segura uma surface de janela real.
//!
//! # ⚠️ O gesto é REAL, e é o que separa esta sonda de um palpite
//!
//! Ela **acha o botão no hit-index** e carrega nele com o ponteiro, pelo roteamento do `winit`
//! (`pixel → hit → widget → chrome → barramento → shell`). ⛔ Empurrar a acção no barramento à mão
//! mediria **outro programa**: o `any_input_this_frame` e o `held_button` — dois dos cinco motivos —
//! são escritos exactamente por esse caminho, e fabricá-los seria fabricar a resposta.
//!
//! # Como se lê
//!
//! ```text
//! [probe-ik-undo] f=NN undo=<n> redo=<n> ancoras=<n> alvos=<n> held=<...>
//! ```
//!
//! - `ancoras` sobe no quadro do clique: o verbo chegou.
//! - `undo` tem de subir **uma** vez logo a seguir. Se não sobe, corra com `PH2D_UNDO_LOG=1` ao
//!   lado: a linha `⛔ o documento MUDOU … motivo: …` nomeia qual dos cinco.
//! - Depois do `Ctrl+Z`, `ancoras` e `alvos` têm de **descer uma**.
//!
//! # ⛔⛔ O QUE ELA MEDIU (2026-09-07) — e o report NÃO reproduz
//!
//! ```text
//! f=49 undo=1 ancoras=1        <- antes
//! Down em Add IK em (696, 717)
//! f=50..52  held=Some(Primary) <- o botao segurado por TRES quadros, como uma mao humana
//! f=53 Up
//! f=54 undo=2 ancoras=2        <- o passo NASCEU e a ancora foi criada
//! f=70 Ctrl+Z
//! f=71 undo=1 redo=1 ancoras=1 <- a ancora do botao FOI EMBORA
//! ```
//!
//! ⇒ pelo caminho completo do artista — **rolar o painel**, `Down` num quadro, `Up` três quadros
//! depois, `Ctrl+Z` — o *Add IK* **é** desfeito. As três metades estão ilibadas por medição: a
//! fotografia vê a âncora, o passo nasce, e o restauro leva-a embora.
//!
//! ⚠️ **O que a sonda NÃO cobre, e é onde a próxima medição começa:** a sequência exacta do dono.
//! Duas hipóteses baratas de eliminar, as duas com o mesmo sintoma (*«o Ctrl+Z não fez nada»*):
//! **(a)** ele arrastou a âncora depois de a criar — o arrasto é um passo PRÓPRIO, e desfazê-lo
//! move o losango de volta uns píxeis, o que se lê como *nada*; **(b)** o `Ctrl+Z` foi roteado para
//! outro dono (o `undo_or_redo` escolhe entre o Áudio, o Painter, o global e o image-edit).
//!
//! ⛔ **E há uma armadilha estrutural nesta cena, medida aqui:** a cena de smoke monta-se **sem
//! entrada nenhuma**, então o primeiro clique do dono — seja ele qual for — regista um passo cujo
//! *antes* é a **cena vazia**. Um `Ctrl+Z` a mais apaga o desenho inteiro, e isso não é um defeito
//! da âncora.

use std::sync::atomic::{AtomicU32, Ordering};

/// O quadro corrente do roteiro — a sonda não pode acrescentar campo à `App`.
static FRAME: AtomicU32 = AtomicU32::new(0);

impl crate::App {
    /// No prólogo do quadro, ao lado das outras sondas. No-op sem a env.
    pub(crate) fn bone_undo_probe(&mut self) {
        if std::env::var_os("PH2D_BONE_UNDO_PROBE").is_none() || self.gfx.is_none() {
            return;
        }
        let f = FRAME.fetch_add(1, Ordering::Relaxed);
        match f {
            // A cena dos ossos monta em dois tempos (o `vec_bone_smoke` precisa do `sync`), e o
            // painel precisa de mais um quadro para publicar o osso em foco.
            // ⚠️ **O pill PRIMEIRO** — medido: sem o modo Osso a secção SKELETON nem sequer é
            // pintada (o pill está no índice, os controlos dela não). *O roteiro tem de ser o
            // caminho do artista, incluindo o passo que ele nem repara que dá.*
            35 => self.probe_click(ph2d_editor::ids::VECTOR_MODE_BONE, "o pill Bone"),
            40 => self.probe_pick_a_bone_without_anchor(),
            // ⚠️ **ROLAR o painel é parte do caminho do artista.** Medido: a secção SKELETON fica
            // ABAIXO da dobra (a STROKE está em `y=258` e a FILL em `y=678`), e o índice de acerto
            // é **recortado** pela banda visível ⇒ o botão não existe para o dedo enquanto não se
            // rola. *Uma sonda que não rola mede um painel que o artista nunca vê inteiro.*
            42..=47 => self.probe_scroll_panel(),
            49 => self.probe_what_is_reachable(),
            // ⚠️⚠️ **O Down e o Up em QUADROS DIFERENTES** — é o que uma mão humana faz, e é a
            // ÚNICA diferença que sobrava contra o roteiro que passou. Dois dos cinco motivos de
            // supressão (`held_button`, `any_input_this_frame`) vivem exactamente nessa janela:
            // um `Down`+`Up` no mesmo quadro nunca vê o `held_button` preso.
            50 => self.probe_press(ph2d_editor::ids::VECTOR_BONE_IK_ADD, "Add IK"),
            53 => {
                eprintln!("[probe-ik-undo] --- Up (3 quadros depois do Down) ---");
                self.smoke_pointer_up();
            }
            70 => {
                eprintln!("[probe-ik-undo] --- Ctrl+Z ---");
                self.smoke_undo(false);
            }
            _ => {}
        }
        // ⭐⭐⭐ **A DERIVA** — o documento a mudar SEM entrada nenhuma. Medida em 2026-09-07: `913`
        // supressões em 15 s com a cena dos ossos, e **`0`** sem smoke nenhum. Isto imprime QUAL
        // linha do mundo muda entre dois quadros parados, que é a pergunta que o
        // `parts_that_differ` (seis baldes) não responde.
        if (30..=34).contains(&f) {
            self.probe_which_rows_drift();
        }
        if (38..=90).contains(&f) {
            let (ancoras, alvos) = self.probe_counts();
            eprintln!(
                "[probe-ik-undo] f={f} undo={} redo={} ancoras={ancoras} alvos={alvos} held={:?}",
                self.undo.depth(),
                self.undo.redo_depth(),
                self.held_button,
            );
        }
    }

    /// **Que LINHAS do mundo mudam entre dois quadros parados** — o nome e o que difere.
    ///
    /// ⚠️ Duas capturas seguidas, no mesmo quadro: se elas diferirem, a deriva está **dentro da
    /// própria captura** (algo que ela mesma escreve); se não, está entre quadros.
    fn probe_which_rows_drift(&mut self) {
        let Some(a) = self.capture_project() else {
            return;
        };
        let Some(b) = self.capture_project() else {
            return;
        };
        let iguais = a.world == b.world;
        eprintln!("[probe-ik-undo] duas capturas no MESMO quadro: mundo igual? {iguais}");
        if iguais {
            return;
        }
        let por_id = |s: &ph2d_ecs::scene::WorldSnapshot| {
            s.entities
                .iter()
                .map(|r| (r.id, r.clone()))
                .collect::<std::collections::BTreeMap<_, _>>()
        };
        let (ma, mb) = (por_id(&a.world), por_id(&b.world));
        for (id, ra) in &ma {
            match mb.get(id) {
                None => eprintln!("[probe-ik-undo]   linha {id:?} SUMIU"),
                Some(rb) if *ra != *rb => {
                    let quais: Vec<u64> = ra
                        .components
                        .iter()
                        .zip(rb.components.iter())
                        .filter(|(x, y)| x != y)
                        .map(|(x, _)| x.type_id)
                        .collect();
                    eprintln!(
                        "[probe-ik-undo]   linha {id:?} MUDOU: {} componente(s), difere em {quais:?}",
                        ra.components.len()
                    );
                }
                _ => {}
            }
        }
        for id in mb.keys() {
            if !ma.contains_key(id) {
                eprintln!("[probe-ik-undo]   linha {id:?} NASCEU");
            }
        }
    }

    /// Quantas âncoras e quantos alvos a cena tem — as duas metades, porque apagar uma sem a outra
    /// é um estado que o report não distingue.
    fn probe_counts(&self) -> (usize, usize) {
        let Some(gfx) = self.gfx.as_ref() else {
            return (0, 0);
        };
        let w = gfx.sim.world();
        (
            w.iter_entities()
                .filter(|e| e.contains::<ph2d_skeleton_ecs::IkGoal>())
                .count(),
            w.iter_entities()
                .filter(|e| e.contains::<ph2d_skeleton_ecs::IkTarget>())
                .count(),
        )
    }

    /// Escolhe um osso que ainda **não** tem âncora — é nele que o painel oferece o *Add IK*.
    ///
    /// ⚠️ A cena do smoke já ancora a ponta do braço, então escolher "um osso qualquer" daria o
    /// *Remove IK* metade das vezes: *uma sonda que não controla o sujeito mede outro botão*.
    fn probe_pick_a_bone_without_anchor(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let alvo = gfx
            .sim
            .world()
            .iter_entities()
            .find(|e| {
                e.contains::<ph2d_skeleton_ecs::Bone>()
                    && !e.contains::<ph2d_skeleton_ecs::IkGoal>()
            })
            .map(|e| e.id());
        match (alvo, gfx.hero_screen.as_mut()) {
            (Some(e), Some(hero)) => {
                hero.gizmo.clear_all_selection();
                hero.gizmo.add_to_selection(e.to_bits());
                eprintln!("[probe-ik-undo] escolhi o osso {e:?} (sem ancora)");
            }
            _ => eprintln!("[probe-ik-undo] ⛔ nao ha' osso sem ancora na cena"),
        }
    }

    /// Rola o painel do vector para baixo, com o cursor sobre ele.
    fn probe_scroll_panel(&mut self) {
        // O ponto é o do pill do modo, que já se sabe estar dentro do painel.
        self.smoke_pointer_move(696.0, 400.0);
        self.on_mouse_wheel(winit::event::MouseScrollDelta::LineDelta(0.0, -24.0));
    }

    /// **Quais controlos da seção estão ALCANÇÁVEIS pelo dedo, agora.**
    ///
    /// ⚠️ Ela existe porque *«o botão não está no hit-index»* tem duas leituras que o sintoma não
    /// distingue: **o painel inteiro** não está visível, ou **só este botão** não é pintado. A
    /// resposta muda tudo o que se investiga a seguir.
    fn probe_what_is_reachable(&mut self) {
        for (id, nome) in [
            (ph2d_editor::ids::VECTOR_SECTION_BONE, "a seccao SKELETON"),
            (ph2d_editor::ids::VECTOR_BONE_BIND, "Bind"),
            (ph2d_editor::ids::VECTOR_BONE_LENGTH, "Length"),
            (ph2d_editor::ids::VECTOR_BONE_IK_ADD, "Add IK"),
            (ph2d_editor::ids::VECTOR_MODE_BONE, "o pill Bone"),
            (ph2d_editor::ids::VECTOR_SECTION_STROKE, "a seccao STROKE"),
            (ph2d_editor::ids::VECTOR_SECTION_FILL, "a seccao FILL"),
        ] {
            eprintln!(
                "[probe-ik-undo] alcancavel? {nome}: {:?}",
                self.smoke_find_widget(id)
            );
        }
    }

    /// Só o **Down** sobre um widget — o `Up` vem noutro quadro, como na mão de uma pessoa.
    fn probe_press(&mut self, id: ph2d_editor::NodeId, nome: &str) {
        let Some((x, y)) = self.smoke_find_widget(id) else {
            eprintln!("[probe-ik-undo] ⛔ o botao {nome} NAO esta' no hit-index");
            return;
        };
        eprintln!("[probe-ik-undo] Down em {nome} em ({x}, {y})");
        self.smoke_pointer_down(x, y);
    }

    /// Carrega num widget do painel **com o ponteiro**, pelo caminho do artista.
    fn probe_click(&mut self, id: ph2d_editor::NodeId, nome: &str) {
        let Some((x, y)) = self.smoke_find_widget(id) else {
            eprintln!("[probe-ik-undo] ⛔ o botao {nome} NAO esta' no hit-index (nao e' clicavel)");
            return;
        };
        eprintln!("[probe-ik-undo] clicando em {nome} em ({x}, {y})");
        self.smoke_pointer_down(x, y);
        self.smoke_pointer_up();
    }
}
