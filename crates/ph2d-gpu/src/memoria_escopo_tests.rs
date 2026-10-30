//! **G29 — a validação que uma recusa de memória provoca não é um `[gpu-erro]` à parte** (F40): com o
//! wgpu 30 o buffer recusado do Vello voltava inválido ao bind group do MESMO `f` e a cena 41 com
//! 476 MiB livres contava 100 `[gpu-erro]`. Sem recusa, a validação continua a ir ao contador.

use super::*;

fn invalido() -> wgpu::Error {
    wgpu::Error::Validation {
        source: Box::new(std::io::Error::other("bind group sobre buffer recusado")),
        description: "Buffer with 'vello.ptcl_buf' label is invalid".into(),
    }
}

#[test]
fn a_validacao_de_uma_recusa_e_a_mesma_recusa() {
    assert!(matches!(destino(true, Some(invalido())), Destino::Nenhum));
    assert!(matches!(
        destino(false, Some(invalido())),
        Destino::Relata(_)
    ));
    assert!(matches!(destino(false, None), Destino::Nenhum));
    assert!(matches!(destino(true, None), Destino::Nenhum));
}

/// Na placa: um erro de validação PRÓPRIO dentro do escopo chega ao `[gpu-erro]`, e o escopo não o
/// toma por falta de memória.
#[test]
fn a_validacao_sem_recusa_chega_ao_contador() {
    let Ok(gpu) = crate::GpuContext::new(crate::GpuContext::default_instance(), None) else {
        return;
    };
    let antes = crate::context::erros_relatados();
    let ((), falta) = sob_escopo_de_memoria(&gpu.device, || {
        // `mapped_at_creation` exige tamanho múltiplo de 4: erro de validação.
        let _ = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("G29 inválido"),
            size: 3,
            usage: wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: true,
        });
    });
    assert!(falta.is_none(), "validação não é falta de memória");
    assert_eq!(crate::context::erros_relatados(), antes + 1);
}
