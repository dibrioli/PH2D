//! ⭐⭐⭐ **A EXPORTAÇÃO SAI DA THREAD QUE DESENHA** — a bancada de trabalho do módulo 3D.
//!
//! # ⚠️ O report do Enio: *"o linux fica cinza"* (2026-08-25)
//!
//! Depois da W62 a exportação caiu de 8 min 17 s para 6,4 s na esfera e **12 s** na peça dele. O
//! tempo deixou de ser absurdo e o defeito **não desapareceu**: durante esses 12 s o loop não
//! responde ao *ping* do compositor, e o KDE pinta a janela de cinza e oferece *"forçar o
//! encerramento"*. ⛔ **Isso não é um incómodo estético — é o sistema operativo a dizer ao artista
//! que o programa morreu**, e o gesto natural a seguir é matá-lo, com o trabalho não gravado dentro.
//!
//! ⭐ **A cura não é ser mais rápido: é não bloquear.** Um trabalho de 200 ms cinza-nada e um de
//! 12 s cinza tudo, e a fronteira entre os dois é do compositor, não nossa. *Enquanto o trabalho
//! correr na thread que desenha, a única defesa é ser rápido o suficiente — e "rápido o suficiente"
//! é um número que outra pessoa muda.*
//!
//! # ⚠️ Por que uma BANCADA e não uma thread solta
//!
//! O padrão da casa para «o painel pede, o quadro serve» é a caixa de correio drenada uma vez por
//! quadro ([`crate::field3d_smoke_requests`]). Esta é a mesma lei com um degrau a mais: o pedido
//! atravessa para o app, o app entrega o trabalho **à bancada**, e a **resposta** volta pela mesma
//! porta. ⚠️ A caixa dos pedidos é `thread_local` de propósito (tudo lá vive numa thread só); esta
//! **não pode ser**, porque quem escreve é o trabalhador — daí o `Mutex`.
//!
//! ⚠️ **Uma de cada vez.** Duas exportações a correr sobre o mesmo caminho de arquivo dariam um
//! arquivo com metade de cada, e a segunda mensagem apagaria a primeira. O segundo clique é
//! recusado **com aviso** — recusar em silêncio é o defeito que este módulo já pagou.
//!
//! ⚠️ **O que atravessa é a MENSAGEM PRONTA, não a malha.** Montar a frase é trabalho puro
//! (contagens, tamanho da caixa, o que o formato perde) e fazê-lo do lado de lá deixa o quadro com
//! uma coisa só a fazer: mostrar. *A fronteira mais barata é a que só carrega texto.*

use std::sync::{Mutex, OnceLock};

/// O que a bancada sabe: se há trabalho a correr e qual foi a última resposta por entregar.
#[derive(Default)]
struct Desk {
    running: bool,
    done: Option<String>,
}

fn desk() -> &'static Mutex<Desk> {
    static DESK: OnceLock<Mutex<Desk>> = OnceLock::new();
    DESK.get_or_init(|| Mutex::new(Desk::default()))
}

/// ⚠️ **Um `Mutex` envenenado não pode derrubar o quadro.** Se o trabalhador entrou em `panic!` com
/// o cadeado na mão, o desenho não tem culpa — e recusar-se a desenhar seria transformar um defeito
/// da exportação numa janela morta, que é exactamente o que esta bancada existe para evitar.
fn lock() -> std::sync::MutexGuard<'static, Desk> {
    desk()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Há uma exportação a correr?
pub(crate) fn is_running() -> bool {
    lock().running
}

/// ⭐ **A resposta, tirada UMA vez** — a mesma lei das caixas de correio do painel: um resultado que
/// ficasse pousado repetiria o toast em todo quadro seguinte.
pub(crate) fn take_finished() -> Option<String> {
    lock().done.take()
}

/// ⭐⭐ **Entrega o trabalho à bancada.** Devolve `false` — e não faz nada — se já houver um a
/// correr.
///
/// ⚠️ **O `Send + 'static` é o gate que o compilador escreve.** Ele é a razão de esta função receber
/// um fecho em vez de os argumentos da exportação: qualquer coisa que a exportação venha a precisar
/// e que não atravesse a fronteira torna-se um **erro de compilação aqui**, e não um defeito
/// descoberto no smoke.
///
/// ⚠️ **Um `panic!` do trabalhador não pode deixar a bancada ocupada para sempre** — senão o
/// primeiro estouro trancava a exportação até ao fim da sessão, e o sintoma seria *"o botão parou de
/// funcionar"*. O `running` é baixado por um sentinela cujo `Drop` corre na desmontagem.
pub(crate) fn spawn(job: impl FnOnce() -> String + Send + 'static) -> bool {
    {
        let mut d = lock();
        if d.running {
            return false;
        }
        d.running = true;
    }
    std::thread::Builder::new()
        .name("ph2d-field3d-export".into())
        .spawn(move || {
            let _guard = Guard;
            let message = job();
            let mut d = lock();
            d.done = Some(message);
        })
        .map_or_else(
            |_| {
                // A thread não nasceu (sem recursos): a bancada tem de voltar a estar livre, senão
                // o botão fica mudo para sempre.
                lock().running = false;
                false
            },
            |_| true,
        )
}

/// Baixa o `running` mesmo se o trabalho estourar — ver [`spawn`].
struct Guard;

impl Drop for Guard {
    fn drop(&mut self) {
        lock().running = false;
    }
}

#[cfg(test)]
#[path = "field3d_export_job_tests.rs"]
mod tests;
