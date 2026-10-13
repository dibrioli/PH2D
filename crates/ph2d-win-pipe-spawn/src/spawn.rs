//! `CreatePipe` ×2 + `CreateProcessW` com `PROC_THREAD_ATTRIBUTE_HANDLE_LIST`: o filho herda SÓ as duas
//! pontas dele; e um job `KILL_ON_JOB_CLOSE` leva a árvore inteira (os processos-filho do navegador)
//! quando o [`Child`] cai ou o PH2D morre. Os handles saem daqui embrulhados (`OwnedHandle`/`File`);
//! nenhum ponteiro cru atravessa.
#![allow(unsafe_code)]

use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::marker::PhantomData;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::Path;
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{
    HANDLE, HANDLE_FLAG_INHERIT, STILL_ACTIVE, SetHandleInformation, WAIT_OBJECT_0,
};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject, TerminateJobObject,
};
use windows_sys::Win32::System::Pipes::CreatePipe;
use windows_sys::Win32::System::Threading::{
    CREATE_NO_WINDOW, CREATE_SUSPENDED, CreateProcessW, DeleteProcThreadAttributeList,
    EXTENDED_STARTUPINFO_PRESENT, GetExitCodeProcess, INFINITE, InitializeProcThreadAttributeList,
    LPPROC_THREAD_ATTRIBUTE_LIST, OpenProcess, PROC_THREAD_ATTRIBUTE_HANDLE_LIST,
    PROCESS_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, ResumeThread, STARTUPINFOEXW,
    TerminateProcess, UpdateProcThreadAttribute, WaitForSingleObject,
};

/// O processo lançado. Ao cair, o job fecha e leva o que ainda viver da árvore.
pub struct Child {
    process: OwnedHandle,
    /// `None` = o SO recusou o job (p.ex. o PH2D já corre num job que não deixa aninhar).
    job: Option<OwnedHandle>,
}

impl Child {
    /// Termina o processo e a árvore dele (como o `Child::kill` do `std`, mais os filhos).
    ///
    /// # Errors
    /// O do `TerminateJobObject`/`TerminateProcess` (p.ex. o processo já saiu).
    pub fn kill(&mut self) -> io::Result<()> {
        match &self.job {
            // SAFETY: `job` é um handle de job vivo e nosso.
            Some(job) => ok(unsafe { TerminateJobObject(job.as_raw_handle(), 1) }),
            // SAFETY: `process` é um handle de processo vivo e nosso.
            None => ok(unsafe { TerminateProcess(self.process.as_raw_handle(), 1) }),
        }
    }

    /// Espera que saia; devolve o código de saída.
    ///
    /// # Errors
    /// O da espera ou do `GetExitCodeProcess`.
    pub fn wait(&mut self) -> io::Result<u32> {
        let h = self.process.as_raw_handle();
        // SAFETY: `h` é um handle de processo vivo e nosso.
        if unsafe { WaitForSingleObject(h, INFINITE) } != WAIT_OBJECT_0 {
            return Err(io::Error::last_os_error());
        }
        let mut code = 0u32;
        // SAFETY: `h` vivo; `code` é um `u32` local.
        ok(unsafe { GetExitCodeProcess(h, &raw mut code) })?;
        Ok(code)
    }
}

/// O filho e as NOSSAS pontas dos dois tubos.
pub struct Spawned {
    pub child: Child,
    /// O que se escreve aqui o filho lê no handle de leitura dele.
    pub to_child: File,
    /// O que o filho escreve no handle de escrita dele chega aqui.
    pub from_child: File,
}

/// Lança `exe` com os argumentos que `args(leitura, escrita)` devolve — os valores (no filho) dos dois
/// handles que ele herda. Sem consola nem outra herança.
///
/// # Errors
/// O do Win32 (`CreatePipe`, `CreateProcessW`…), p.ex. `exe` inexistente.
pub fn spawn(exe: &Path, args: impl FnOnce(u64, u64) -> Vec<OsString>) -> io::Result<Spawned> {
    let (child_read, to_child) = pipe()?;
    let (from_child, child_write) = pipe()?;
    // ⚠️ Herdáveis só entre aqui e o fecho abaixo: um `Command::spawn` noutro fio nesse intervalo também
    // as herdaria (o `std` herda tudo o que é herdável) — o tubo só daria EOF quando esse outro saísse.
    inheritable(&child_read)?;
    inheritable(&child_write)?;
    let handles: [HANDLE; 2] = [child_read.as_raw_handle(), child_write.as_raw_handle()];
    let wide = |s: &std::ffi::OsStr| s.encode_wide().collect::<Vec<u16>>();
    let argv: Vec<Vec<u16>> = args(handles[0] as u64, handles[1] as u64)
        .iter()
        .map(|a| wide(a))
        .collect();
    let mut cmd = crate::command_line(&wide(exe.as_os_str()), &argv);
    let mut app = wide(exe.as_os_str());
    app.push(0);

    let job = kill_on_close_job();
    let list = AttrList::new(&handles)?;
    let mut si = STARTUPINFOEXW::default();
    si.StartupInfo.cb = u32::try_from(size_of::<STARTUPINFOEXW>()).unwrap_or(u32::MAX);
    si.lpAttributeList = list.ptr();
    let mut pi = PROCESS_INFORMATION::default();
    // SAFETY: `app` e `cmd` terminam em NUL e vivem até ao fim da chamada (`cmd` é mutável, como a API
    // pede); `si` aponta para a lista de atributos viva (`list`), que aponta para `handles` (vivo).
    let r = unsafe {
        CreateProcessW(
            app.as_ptr(),
            cmd.as_mut_ptr(),
            null(),
            null(),
            1,
            EXTENDED_STARTUPINFO_PRESENT | CREATE_NO_WINDOW | CREATE_SUSPENDED,
            null(),
            null(),
            &raw const si.StartupInfo,
            &raw mut pi,
        )
    };
    drop(list);
    ok(r)?;
    // SAFETY: o `CreateProcessW` deu certo: os dois handles são novos e nossos.
    let (process, thread) = unsafe {
        (
            OwnedHandle::from_raw_handle(pi.hProcess),
            OwnedHandle::from_raw_handle(pi.hThread),
        )
    };
    // Entra no job ANTES de correr (suspenso): os filhos que ele lançar nascem lá dentro.
    // SAFETY: `job` e `process` são handles vivos e nossos.
    let job = job.filter(
        |j| unsafe { AssignProcessToJobObject(j.as_raw_handle(), process.as_raw_handle()) } != 0,
    );
    // SAFETY: `thread` é o fio principal, suspenso, vivo e nosso.
    if unsafe { ResumeThread(thread.as_raw_handle()) } == u32::MAX {
        let e = io::Error::last_os_error();
        // SAFETY: `process` vivo e nosso; o filho nunca correu.
        unsafe { TerminateProcess(process.as_raw_handle(), 1) };
        return Err(e);
    }
    drop(thread);
    // As cópias NOSSAS das pontas do filho fecham-se já: sem isto o fim do filho nunca daria EOF.
    drop((child_read, child_write));
    Ok(Spawned {
        child: Child { process, job },
        to_child: File::from(to_child),
        from_child: File::from(from_child),
    })
}

/// `true` = o processo `pid` existe e ainda corre.
#[must_use]
pub fn is_alive(pid: u32) -> bool {
    // SAFETY: só pede um handle de consulta; nulo = não há (ou não se pode ver).
    let h = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if h.is_null() {
        return false;
    }
    // SAFETY: `h` acabou de ser aberto e é nosso.
    let h = unsafe { OwnedHandle::from_raw_handle(h) };
    let mut code = 0u32;
    // SAFETY: `h` vivo; `code` é um `u32` local.
    let r = unsafe { GetExitCodeProcess(h.as_raw_handle(), &raw mut code) };
    r != 0 && code == STILL_ACTIVE.cast_unsigned()
}

/// Um job que mata os seus processos quando o último handle dele fecha. `None` se o SO o recusar.
fn kill_on_close_job() -> Option<OwnedHandle> {
    // SAFETY: sem nome nem atributos.
    let h = unsafe { CreateJobObjectW(null(), null()) };
    if h.is_null() {
        return None;
    }
    // SAFETY: `h` acabou de ser criado e é nosso.
    let job = unsafe { OwnedHandle::from_raw_handle(h) };
    let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    // SAFETY: `info` é a estrutura que a classe pede, com o tamanho dela.
    let r = unsafe {
        SetInformationJobObject(
            job.as_raw_handle(),
            JobObjectExtendedLimitInformation,
            (&raw const info).cast(),
            u32::try_from(size_of_val(&info)).unwrap_or(u32::MAX),
        )
    };
    (r != 0).then_some(job)
}

/// Um tubo anónimo, as duas pontas NÃO herdáveis: `(leitura, escrita)`.
fn pipe() -> io::Result<(OwnedHandle, OwnedHandle)> {
    let (mut r, mut w): (HANDLE, HANDLE) = (null_mut(), null_mut());
    // SAFETY: dois ponteiros de saída para `HANDLE`s locais; atributos nulos = não herdáveis.
    ok(unsafe { CreatePipe(&raw mut r, &raw mut w, null(), 0) })?;
    // SAFETY: o `CreatePipe` deu certo: dois handles novos e nossos.
    Ok(unsafe {
        (
            OwnedHandle::from_raw_handle(r),
            OwnedHandle::from_raw_handle(w),
        )
    })
}

fn inheritable(h: &OwnedHandle) -> io::Result<()> {
    // SAFETY: `h` é um handle vivo e nosso.
    ok(
        unsafe {
            SetHandleInformation(h.as_raw_handle(), HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT)
        },
    )
}

fn ok(r: i32) -> io::Result<()> {
    if r == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// A lista de atributos com o `HANDLE_LIST`; aponta para `handles`, que tem de lhe sobreviver.
struct AttrList<'a> {
    // O dono da memória; só se usa por `list` (tirado de `as_mut_ptr`: a API escreve nela).
    _buf: Vec<usize>,
    list: LPPROC_THREAD_ATTRIBUTE_LIST,
    _handles: PhantomData<&'a [HANDLE]>,
}

impl<'a> AttrList<'a> {
    fn new(handles: &'a [HANDLE]) -> io::Result<Self> {
        let mut size = 0usize;
        // SAFETY: a pergunta do tamanho (lista nula) — falha por desenho e escreve `size`.
        unsafe { InitializeProcThreadAttributeList(null_mut(), 1, 0, &raw mut size) };
        if size == 0 {
            return Err(io::Error::last_os_error());
        }
        // `usize` alinha a lista como a API pede.
        let mut buf = vec![0usize; size.div_ceil(size_of::<usize>())];
        let raw: LPPROC_THREAD_ATTRIBUTE_LIST = buf.as_mut_ptr().cast();
        // SAFETY: `raw` aponta para pelo menos `size` bytes alinhados.
        ok(unsafe { InitializeProcThreadAttributeList(raw, 1, 0, &raw mut size) })?;
        let list = Self {
            _buf: buf,
            list: raw,
            _handles: PhantomData,
        };
        // SAFETY: a lista foi inicializada; `handles` vive `'a`, mais que a lista.
        ok(unsafe {
            UpdateProcThreadAttribute(
                list.ptr(),
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                handles.as_ptr().cast(),
                size_of_val(handles),
                null_mut(),
                null(),
            )
        })?;
        Ok(list)
    }

    fn ptr(&self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        self.list
    }
}

impl Drop for AttrList<'_> {
    fn drop(&mut self) {
        // SAFETY: a lista foi inicializada no `new` (só se constrói depois disso).
        unsafe { DeleteProcThreadAttributeList(self.ptr()) };
    }
}
