//! Gates da CRT na janela do jogo ([`crate::palco`]): desligada = zero passes e o quadro do
//! compositor; ligada = um passe por quadro, ao centro, com os níveis a atravessar crus.

use crate::palco::{Logica, compoe, passes};
use crate::{CrtParams, gpu, vista::VistaCrt};
use ph2d_gpu::GpuContext;
use std::sync::OnceLock;

fn placa() -> Option<&'static GpuContext> {
    static G: OnceLock<Option<GpuContext>> = OnceLock::new();
    G.get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .as_ref()
}

const JANELA: (u32, u32) = (65, 49);
const FMT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const COR: wgpu::Color = wgpu::Color {
    r: 0.2,
    g: 0.5,
    b: 0.8,
    a: 1.0,
};

/// O «compositor» do teste: limpa o alvo que lhe dão com [`COR`].
fn limpa(g: &GpuContext, v: &wgpu::TextureView) {
    let mut enc = g
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    enc.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: v,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(COR),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    g.queue.submit([enc.finish()]);
}

/// Um quadro do jogo num ecrã `*Srgb` de [`JANELA`]; devolve os bytes do ecrã.
fn quadro(g: &GpuContext, crt: Option<(Logica, CrtParams)>) -> Vec<u8> {
    let tex = |usage| {
        g.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: JANELA.0,
                height: JANELA.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FMT,
            usage,
            view_formats: &[],
        })
    };
    let ecra = tex(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC);
    let vista = ecra.create_view(&wgpu::TextureViewDescriptor::default());
    compoe(g, FMT, &vista, JANELA, crt, |v| limpa(g, v));
    let c = gpu::Corrida {
        origem: tex(wgpu::TextureUsages::TEXTURE_BINDING),
        alvo: ecra,
        dims: (JANELA.0, JANELA.1, 1),
    };
    gpu::le_alvo(g, &c).expect("leitura")
}

/// 24 texels de altura numa janela 65×49 ⇒ 32×24 ×2 = 64×48 no canto (0, 0); a coluna 64 e a
/// linha 48 ficam pretas.
fn logica() -> Logica {
    Logica::da_camara(JANELA, 0.24, 100.0).expect("lógica")
}

fn px(b: &[u8], x: u32, y: u32) -> [u8; 4] {
    let i = ((y * JANELA.0 + x) * 4) as usize;
    [b[i], b[i + 1], b[i + 2], b[i + 3]]
}

#[test]
fn a_resolucao_logica_e_a_camara_vezes_os_pixels_por_metro() {
    assert_eq!(
        logica(),
        Logica {
            w: 32,
            h: 24,
            escala: 2
        }
    );
    // O projecto de fábrica (10 m × 100 px/m) numa janela 1080p: a grade passa a janela? não — 1000.
    let hd = Logica::da_camara((1920, 1080), 10.0, 100.0).expect("hd");
    assert_eq!((hd.w, hd.h, hd.escala), (1778, 1000, 1));
    // Pixel art a 16 px/m: 160 texels ⇒ ×6 = 1704×960, centrado.
    let pa = Logica::da_camara((1920, 1080), 10.0, 16.0).expect("pa");
    assert_eq!((pa.w, pa.h, pa.escala), (284, 160, 6));
    assert_eq!(pa.canto((1920, 1080)), (108, 60));
    // A grade nunca passa a janela.
    let fino = Logica::da_camara((640, 360), 10.0, 100.0).expect("fino");
    assert_eq!((fino.w, fino.h, fino.escala), (640, 360, 1));
    assert_eq!(Logica::da_camara((0, 10), 1.0, 1.0), None);
    assert_eq!(Logica::da_camara((10, 10), f32::NAN, 1.0), None);
}

#[test]
fn a_crt_desligada_no_jogo_nao_corre_e_o_quadro_e_o_do_compositor() {
    let Some(g) = placa() else {
        eprintln!("SKIP: sem GPU — gate saltado");
        return;
    };
    let antes = passes();
    let q = quadro(g, None);
    assert_eq!(passes(), antes, "desligada ⇒ zero passes");
    let cor = px(&q, 0, 0);
    assert!(
        q.chunks(4).all(|p| p == cor),
        "o ecrã inteiro é o do compositor"
    );
    assert_eq!(px(&q, 64, 48), cor, "sem barras pretas");
}

#[test]
fn a_crt_ligada_no_jogo_corre_um_passe_por_quadro_ao_centro() {
    let Some(g) = placa() else {
        eprintln!("SKIP: sem GPU — gate saltado");
        return;
    };
    let de_hoje = quadro(g, None);
    let cor = px(&de_hoje, 0, 0);
    let antes = passes();
    // Params neutros: os níveis atravessam a redução, a CRT e a curva sRGB sem mudar um byte.
    for _ in 0..3 {
        let q = quadro(g, Some((logica(), CrtParams::default())));
        for (x, y) in [(0, 0), (31, 23), (63, 47)] {
            assert_eq!(px(&q, x, y), cor, "({x}, {y}) dentro da saída");
        }
        assert_eq!(px(&q, 64, 10), [0, 0, 0, 255], "a coluna que sobra é preta");
        assert_eq!(px(&q, 10, 48), [0, 0, 0, 255], "a linha que sobra é preta");
    }
    assert_eq!(passes(), antes + 3, "um passe por quadro");
    // Os params de fábrica mudam a imagem: a linha ímpar escurece.
    let tv = quadro(g, Some((logica(), VistaCrt::default().params)));
    assert_ne!(px(&tv, 31, 23), px(&tv, 31, 22), "a scanline da TV");
    assert_eq!(passes(), antes + 4);
}
