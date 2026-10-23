//! A régua de registos (o `Cargo.toml` diz o porquê). Uso:
//!
//! ```text
//! regua-registos <pasta>...
//! ```
//!
//! Cada `<pasta>/<nome>.txt` é uma pipeline: `modulo <ficheiro.wgsl>` (relativo à pasta), `entrada <nome>` e
//! uma constante `override` por linha (`NOME valor`). O `medida_luz2d_regua_registos` grava-as tal e qual o
//! `pipeline_em` da C3 as monta; uma forma candidata é o mesmo `.txt` a apontar outro `.wgsl`. Sai uma linha
//! `REGISTOS · …` por pipeline, com a estatística inteira do driver por baixo.

use std::collections::BTreeMap;
use std::ffi::CString;
use std::path::Path;

use ash::vk;
use naga::back::spv;
use naga::proc::{BoundsCheckPolicies, BoundsCheckPolicy};

type Erro = Box<dyn std::error::Error>;

struct Placa {
    _entry: ash::Entry,
    instancia: ash::Instance,
    dev: ash::Device,
    exec: ash::khr::pipeline_executable_properties::Device,
    opcoes: spv::Options<'static>,
    subgrupo_variavel: bool,
    nome: String,
}

fn placa() -> Result<Placa, Erro> {
    let entry = unsafe { ash::Entry::load()? };
    let app = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_3);
    let instancia = unsafe {
        entry.create_instance(
            &vk::InstanceCreateInfo::default().application_info(&app),
            None,
        )?
    };
    let pds = unsafe { instancia.enumerate_physical_devices()? };
    let pd = *pds
        .iter()
        .find(|&&p| unsafe {
            instancia.get_physical_device_properties(p).device_type
                == vk::PhysicalDeviceType::DISCRETE_GPU
        })
        .or(pds.first())
        .ok_or("nenhuma placa Vulkan")?;
    let props = unsafe { instancia.get_physical_device_properties(pd) };
    let nome = props.device_name_as_c_str()?.to_string_lossy().into_owned();
    let exts: Vec<CString> = unsafe { instancia.enumerate_device_extension_properties(pd)? }
        .iter()
        .filter_map(|e| e.extension_name_as_c_str().ok().map(CString::from))
        .collect();
    let tem = |n: &std::ffi::CStr| exts.iter().any(|e| e.as_c_str() == n);
    let exec_nome = ash::khr::pipeline_executable_properties::NAME;
    if !tem(exec_nome) {
        return Err(format!("{nome}: sem {exec_nome:?}").into());
    }
    let r2_tem = tem(ash::ext::robustness2::NAME);
    let fila = unsafe { instancia.get_physical_device_queue_family_properties(pd) }
        .iter()
        .position(|q| q.queue_flags.contains(vk::QueueFlags::COMPUTE))
        .ok_or("sem fila de computação")? as u32;

    // Tudo o que a placa tem, ligado (o wgpu-hal liga o que o PH2D pede; o compilador só vê f16, int16,
    // subgrupos e as robustezes).
    let mut f11 = vk::PhysicalDeviceVulkan11Features::default();
    let mut f12 = vk::PhysicalDeviceVulkan12Features::default();
    let mut f13 = vk::PhysicalDeviceVulkan13Features::default();
    let mut r2 = vk::PhysicalDeviceRobustness2FeaturesEXT::default();
    let mut pe = vk::PhysicalDevicePipelineExecutablePropertiesFeaturesKHR::default();
    let core = {
        let mut f2 = vk::PhysicalDeviceFeatures2::default()
            .push_next(&mut f11)
            .push_next(&mut f12)
            .push_next(&mut f13)
            .push_next(&mut pe);
        if r2_tem {
            f2 = f2.push_next(&mut r2);
        }
        unsafe { instancia.get_physical_device_features2(pd, &mut f2) };
        f2.features
    };
    for p in [
        &mut f11.p_next,
        &mut f12.p_next,
        &mut f13.p_next,
        &mut pe.p_next,
        &mut r2.p_next,
    ] {
        *p = std::ptr::null_mut();
    }
    let robusto_buffer = r2_tem && r2.robust_buffer_access2 == vk::TRUE;
    let robusta_imagem = r2_tem && r2.robust_image_access2 == vk::TRUE;
    let subgrupo_variavel = f13.subgroup_size_control == vk::TRUE;
    let zero = f13.shader_zero_initialize_workgroup_memory == vk::TRUE;
    let io16 = f11.storage_input_output16 == vk::TRUE;

    let mut nomes = vec![exec_nome.as_ptr()];
    if r2_tem {
        nomes.push(ash::ext::robustness2::NAME.as_ptr());
    }
    let prioridade = [1.0];
    let filas = [vk::DeviceQueueCreateInfo::default()
        .queue_family_index(fila)
        .queue_priorities(&prioridade)];
    let mut f2 = vk::PhysicalDeviceFeatures2::default()
        .features(core)
        .push_next(&mut f11)
        .push_next(&mut f12)
        .push_next(&mut f13)
        .push_next(&mut pe);
    if r2_tem {
        f2 = f2.push_next(&mut r2);
    }
    let ci = vk::DeviceCreateInfo::default()
        .queue_create_infos(&filas)
        .enabled_extension_names(&nomes)
        .push_next(&mut f2);
    let dev = unsafe { instancia.create_device(pd, &ci, None)? };
    let exec = ash::khr::pipeline_executable_properties::Device::new(&instancia, &dev);

    // As opções do `Adapter` do wgpu-hal 29 (`vulkan/adapter.rs`, `spv::Options`) para uma placa 1.3.
    let mut flags = spv::WriterFlags::LABEL_VARYINGS;
    flags.set(spv::WriterFlags::FORCE_POINT_SIZE, true);
    let politica = |robusto| {
        if robusto {
            BoundsCheckPolicy::Unchecked
        } else {
            BoundsCheckPolicy::Restrict
        }
    };
    let opcoes = spv::Options {
        lang_version: (1, 6),
        flags,
        capabilities: None,
        bounds_check_policies: BoundsCheckPolicies {
            index: BoundsCheckPolicy::Restrict,
            buffer: politica(robusto_buffer),
            image_load: politica(robusta_imagem),
            binding_array: BoundsCheckPolicy::Unchecked,
        },
        zero_initialize_workgroup_memory: if zero {
            spv::ZeroInitializeWorkgroupMemoryMode::Native
        } else {
            spv::ZeroInitializeWorkgroupMemoryMode::Polyfill
        },
        force_loop_bounding: true,
        ray_query_initialization_tracking: true,
        use_storage_input_output_16: io16,
        ..spv::Options::default()
    };
    eprintln!(
        "# régua de registos · {nome} · robustness2 buffer {robusto_buffer} imagem {robusta_imagem} · subgrupo variável {subgrupo_variavel} · io16 {io16}"
    );
    Ok(Placa {
        _entry: entry,
        instancia,
        dev,
        exec,
        opcoes,
        subgrupo_variavel,
        nome,
    })
}

/// O tipo de descritor de uma variável global com ligação.
fn descritor(m: &naga::Module, g: &naga::GlobalVariable) -> Result<vk::DescriptorType, Erro> {
    Ok(match g.space {
        naga::AddressSpace::Uniform => vk::DescriptorType::UNIFORM_BUFFER,
        naga::AddressSpace::Storage { .. } => vk::DescriptorType::STORAGE_BUFFER,
        naga::AddressSpace::Handle => match m.types[g.ty].inner {
            naga::TypeInner::Image {
                class: naga::ImageClass::Storage { .. },
                ..
            } => vk::DescriptorType::STORAGE_IMAGE,
            naga::TypeInner::Image { .. } => vk::DescriptorType::SAMPLED_IMAGE,
            naga::TypeInner::Sampler { .. } => vk::DescriptorType::SAMPLER,
            ref t => return Err(format!("ligação de tipo inesperado: {t:?}").into()),
        },
        s => return Err(format!("espaço inesperado: {s:?}").into()),
    })
}

/// Compila uma pipeline e devolve a estatística do driver por executável.
fn mede(
    p: &Placa,
    fonte: &str,
    entrada: &str,
    constantes: &naga::back::PipelineConstants,
) -> Result<Vec<Vec<(String, String)>>, Erro> {
    let module = naga::front::wgsl::parse_str(fonte).map_err(|e| e.emit_to_string(fonte))?;
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)?;
    let (module, info) = naga::back::pipeline_constants::process_overrides(
        &module,
        &info,
        Some((naga::ShaderStage::Compute, entrada)),
        constantes,
    )?;
    let po = spv::PipelineOptions {
        entry_point: entrada.to_owned(),
        shader_stage: naga::ShaderStage::Compute,
    };
    let spv = spv::write_vec(&module, &info, &p.opcoes, Some(&po))?;

    let ep = module
        .entry_points
        .iter()
        .position(|e| e.name == entrada)
        .ok_or_else(|| format!("sem a entrada {entrada}"))?;
    let usos = info.get_entry_point(ep);
    let mut grupos: BTreeMap<u32, Vec<vk::DescriptorSetLayoutBinding>> = BTreeMap::new();
    for (h, g) in module.global_variables.iter() {
        let Some(rb) = &g.binding else { continue };
        if usos[h].is_empty() {
            continue;
        }
        grupos.entry(rb.group).or_default().push(
            vk::DescriptorSetLayoutBinding::default()
                .binding(rb.binding)
                .descriptor_type(descritor(&module, g)?)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        );
    }
    let n = grupos.keys().next_back().map_or(0, |g| g + 1);
    let d = &p.dev;
    unsafe {
        let layouts: Vec<vk::DescriptorSetLayout> = (0..n)
            .map(|g| {
                let b = grupos.get(&g).map_or(&[][..], Vec::as_slice);
                d.create_descriptor_set_layout(
                    &vk::DescriptorSetLayoutCreateInfo::default().bindings(b),
                    None,
                )
            })
            .collect::<Result<_, _>>()?;
        let pl = d.create_pipeline_layout(
            &vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts),
            None,
        )?;
        let sm = d.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&spv), None)?;
        let nome = CString::new(entrada)?;
        let mut estagio = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(sm)
            .name(&nome);
        if p.subgrupo_variavel {
            estagio =
                estagio.flags(vk::PipelineShaderStageCreateFlags::ALLOW_VARYING_SUBGROUP_SIZE);
        }
        let ci = vk::ComputePipelineCreateInfo::default()
            .stage(estagio)
            .layout(pl)
            .flags(vk::PipelineCreateFlags::CAPTURE_STATISTICS_KHR);
        let pipe = d
            .create_compute_pipelines(vk::PipelineCache::null(), &[ci], None)
            .map_err(|(_, e)| e)?[0];
        let execs = p
            .exec
            .get_pipeline_executable_properties(&vk::PipelineInfoKHR::default().pipeline(pipe))?;
        let mut saida = Vec::new();
        for i in 0..execs.len() as u32 {
            let st = p.exec.get_pipeline_executable_statistics(
                &vk::PipelineExecutableInfoKHR::default()
                    .pipeline(pipe)
                    .executable_index(i),
            )?;
            saida.push(
                st.iter()
                    .map(|s| {
                        let v = match s.format {
                            vk::PipelineExecutableStatisticFormatKHR::BOOL32 => {
                                (s.value.b32 != 0).to_string()
                            }
                            vk::PipelineExecutableStatisticFormatKHR::INT64 => {
                                s.value.i64.to_string()
                            }
                            vk::PipelineExecutableStatisticFormatKHR::UINT64 => {
                                s.value.u64.to_string()
                            }
                            _ => s.value.f64.to_string(),
                        };
                        let k = s
                            .name_as_c_str()
                            .map_or_else(|_| "?".into(), |c| c.to_string_lossy().into_owned());
                        let d = s
                            .description_as_c_str()
                            .map_or_else(|_| "?".into(), |c| c.to_string_lossy().into_owned());
                        (k, format!("{v} ({:?}: {d})", s.format))
                    })
                    .collect(),
            );
        }
        d.destroy_pipeline(pipe, None);
        d.destroy_shader_module(sm, None);
        d.destroy_pipeline_layout(pl, None);
        for l in layouts {
            d.destroy_descriptor_set_layout(l, None);
        }
        Ok(saida)
    }
}

/// Lê um `.txt` de pipeline: (módulo, entrada, constantes).
fn pipeline(txt: &Path) -> Result<(String, String, naga::back::PipelineConstants), Erro> {
    let mut modulo = None;
    let mut entrada = None;
    let mut c = naga::back::PipelineConstants::default();
    for l in std::fs::read_to_string(txt)?
        .lines()
        .filter(|l| !l.trim().is_empty())
    {
        let (k, v) = l
            .split_once(' ')
            .ok_or_else(|| format!("{}: linha sem valor: {l}", txt.display()))?;
        match k {
            "modulo" => modulo = Some(v.trim().to_owned()),
            "entrada" => entrada = Some(v.trim().to_owned()),
            _ => {
                c.insert(k.to_owned(), v.trim().parse()?);
            }
        }
    }
    let modulo = modulo.ok_or("sem `modulo`")?;
    let fonte = std::fs::read_to_string(txt.with_file_name(&modulo))?;
    Ok((fonte, entrada.ok_or("sem `entrada`")?, c))
}

fn main() -> Result<(), Erro> {
    let pastas: Vec<String> = std::env::args().skip(1).collect();
    if pastas.is_empty() {
        return Err("uso: regua-registos <pasta>...".into());
    }
    let p = placa()?;
    for pasta in &pastas {
        let mut txts: Vec<_> = std::fs::read_dir(pasta)?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|f| f.extension().is_some_and(|x| x == "txt"))
            .collect();
        txts.sort();
        for txt in txts {
            let nome = txt
                .file_stem()
                .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
            let (fonte, entrada, c) = pipeline(&txt)?;
            let st =
                mede(&p, &fonte, &entrada, &c).map_err(|e| format!("{}: {e}", txt.display()))?;
            for (i, e) in st.iter().enumerate() {
                let de = |k: &str| {
                    e.iter()
                        .find(|(n, _)| n == k)
                        .map_or("?", |(_, v)| v.split(' ').next().unwrap_or("?"))
                };
                println!(
                    "REGISTOS · {pasta} · {nome} · {entrada} [{i}] · registos {} · binário {} · partilhada {}",
                    de("Register Count"),
                    de("Binary Size"),
                    de("Shared Memory Size")
                );
                for (k, v) in e {
                    println!("    {k} = {v}");
                }
            }
        }
    }
    eprintln!("# {} · fim", p.nome);
    unsafe {
        p.dev.destroy_device(None);
        p.instancia.destroy_instance(None);
    }
    Ok(())
}
