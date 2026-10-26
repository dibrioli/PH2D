//! **O aviso de placa sem memória** (F35 §2, `docs/UI_New_and_Simple/F35_plano.md`): quando outro
//! programa ocupa a placa, a tela do editor fica vazia — o toast é desenhado por essa mesma placa. O
//! aviso que SEMPRE se vê é o TÍTULO da janela, desenhado pelo sistema; o toast diz a volta.

use crate::toast::Toast;
use ph2d_i18n::tr;

/// O começo do título enquanto a placa está sem memória (`""` quando sã).
pub fn prefixo_do_titulo(sem_memoria: bool) -> String {
    if sem_memoria {
        format!("{} \u{2014} ", tr("shell.gpu_memory.title"))
    } else {
        String::new()
    }
}

/// O toast de uma mudança de estado: a placa ficou sem memória, ou voltou a ter.
pub fn toast(sem_memoria: bool) -> Toast {
    if sem_memoria {
        Toast::warning(tr("shell.gpu_memory.out"))
    } else {
        Toast::success(tr("shell.gpu_memory.back"))
    }
}
