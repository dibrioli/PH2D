//! ⭐⭐⭐ **UM SCRIPT É UM MÓDULO** — o seu próprio ambiente, as suas declarações, os seus ganchos
//! (TOP-20 #16). A lei dos valores vive em [`crate::props`]; aqui ela encontra a VM.
//!
//! # ⚠️ Um ambiente POR SCRIPT, e o porquê medido
//!
//! A VM da casa está em sandbox (HR-9), e a sandbox do Luau dá **um** ambiente local partilhado:
//! dois scripts que definam `function update` escreveriam **no mesmo** sítio, e o último a
//! carregar ganhava para os dois. ⇒ cada módulo corre com uma tabela própria cujas leituras caem
//! para o global (a biblioteca, o `ph2d`) e cujas escritas ficam nela. Gate:
//! `dois_scripts_nao_se_pisam`.
//!
//! # ⚠️ A colheita CORRE o script, e é a única forma honesta
//!
//! As declarações são a resposta **da linguagem**, não a de um *parser* nosso ao lado dela: um
//! `ph2d.property` dentro de um `if` conta se o `if` correr, e um nome calculado é o nome calculado.
//! O preço é declarado: o topo do script corre **uma vez por conteúdo** (a recarga por hash, HR-16),
//! na sandbox, e o que ele tentar escrever no mundo cai numa fila que a colheita não drena.
//!
//! ⛔ **Fora do topo, `ph2d.property` é ERRO** — uma declaração que só existisse depois de a corrida
//! começar seria uma linha de painel que aparece e desaparece. A cerca é a presença do
//! [`DeclSink`] nos dados da VM: ele só existe enquanto o topo corre.

use mlua::{Function, Lua, Table, Value};

use crate::props::{DeclError, PropDecl, PropHint, ScriptValue, check_decl};

/// **O balde das declarações** — presente nos dados da VM só enquanto o topo de um script corre.
#[derive(Default)]
pub(crate) struct DeclSink {
    decls: Vec<PropDecl>,
    /// A recusa que parou o topo — guardada à parte para a distinguir de um erro qualquer.
    error: Option<DeclError>,
}

/// **Porque um script não carregou.**
#[derive(Clone, Debug, PartialEq)]
pub enum ModuleError {
    /// O Luau não o conseguiu ler.
    Syntax(String),
    /// Uma declaração foi recusada — a mensagem diz qual.
    Decl(DeclError),
    /// O topo do script falhou ao correr.
    Runtime(String),
}

impl std::fmt::Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syntax(m) | Self::Runtime(m) => f.write_str(m),
            Self::Decl(e) => write!(f, "{e}"),
        }
    }
}

/// **Um script carregado.**
pub struct ScriptModule {
    env: Table,
    /// As declarações, na ordem das chamadas (Q7).
    pub decls: Vec<PropDecl>,
}

impl ScriptModule {
    /// O gancho `name` **deste** módulo (`init` · `update` · `on_signal`), se o script o definiu.
    ///
    /// ⚠️ **`raw_get`**: uma leitura normal cairia para o global pelo `__index`, e um gancho que
    /// outro módulo pusesse lá (impossível hoje, com a sandbox) seria emprestado a este.
    #[must_use]
    pub fn hook(&self, name: &str) -> Option<Function> {
        self.env.raw_get::<Option<Function>>(name).ok().flatten()
    }
}

/// ⭐ **Carrega `source` como um módulo novo** — corre o topo e colhe as declarações.
///
/// # Errors
/// O [`ModuleError`] que o painel mostra.
pub fn load_module(lua: &Lua, chunk_name: &str, source: &str) -> Result<ScriptModule, ModuleError> {
    let env = module_env(lua).map_err(|e| ModuleError::Runtime(e.to_string()))?;
    let func = lua
        .load(source)
        .set_name(format!("={chunk_name}"))
        .set_environment(env.clone())
        .into_function()
        .map_err(|e| match e {
            mlua::Error::SyntaxError { message, .. } => ModuleError::Syntax(message),
            other => ModuleError::Runtime(other.to_string()),
        })?;
    // ⚠️ O balde entra ANTES e sai DEPOIS, com qualquer resultado: um balde esquecido faria o
    // `ph2d.property` de um gancho posterior declarar em silêncio num módulo que já carregou.
    lua.set_app_data(DeclSink::default());
    let ran = func.call::<()>(());
    let sink = lua.remove_app_data::<DeclSink>().unwrap_or_default();
    match (ran, sink.error) {
        (Ok(()), _) => Ok(ScriptModule {
            env,
            decls: sink.decls,
        }),
        (Err(_), Some(decl)) => Err(ModuleError::Decl(decl)),
        (Err(e), None) => Err(ModuleError::Runtime(runtime_message(&e))),
    }
}

/// O ambiente de um módulo: escritas nele, leituras caem para o global.
fn module_env(lua: &Lua) -> mlua::Result<Table> {
    let env = lua.create_table()?;
    let meta = lua.create_table()?;
    meta.set("__index", lua.globals())?;
    env.set_metatable(Some(meta))?;
    Ok(env)
}

/// A mensagem de um erro de corrida SEM a pilha de chamadas do Rust — o painel mostra uma linha.
#[must_use]
pub fn runtime_message(e: &mlua::Error) -> String {
    match e {
        mlua::Error::CallbackError { cause, .. } => runtime_message(cause),
        mlua::Error::RuntimeError(m) => m.lines().next().unwrap_or_default().to_owned(),
        other => other
            .to_string()
            .lines()
            .next()
            .unwrap_or_default()
            .to_owned(),
    }
}

/// ⭐ **`ph2d.property(name, default, options?)`** — instalada pelo `ScriptHost` antes da sandbox.
pub(crate) fn property_binding(lua: &Lua) -> mlua::Result<Function> {
    lua.create_function(
        |lua, (name, default, options): (String, Value, Option<Table>)| -> mlua::Result<()> {
            let Some(mut sink) = lua.app_data_mut::<DeclSink>() else {
                return Err(mlua::Error::RuntimeError(
                    DeclError::NotAtTop(name).to_string(),
                ));
            };
            let decl = read_decl(name, &default, options.as_ref());
            let checked = decl.and_then(|d| check_decl(&sink.decls, &d).map(|()| d));
            match checked {
                Ok(d) => {
                    sink.decls.push(d);
                    Ok(())
                }
                Err(e) => {
                    let msg = e.to_string();
                    sink.error = Some(e);
                    Err(mlua::Error::RuntimeError(msg))
                }
            }
        },
    )
}

/// Traduz os três argumentos para uma declaração — a recusa de FORMA; a de conteúdo é o
/// [`check_decl`].
fn read_decl(
    name: String,
    default: &Value,
    options: Option<&Table>,
) -> Result<PropDecl, DeclError> {
    let default = match default {
        Value::Number(n) => ScriptValue::Number(*n),
        #[expect(
            clippy::cast_precision_loss,
            reason = "o Luau guarda números como f64; um inteiro vindo da VM já cabia nele"
        )]
        Value::Integer(i) => ScriptValue::Number(*i as f64),
        Value::Boolean(b) => ScriptValue::Bool(*b),
        Value::String(s) => match s.to_str() {
            Ok(s) => ScriptValue::Text(s.to_owned()),
            Err(_) => return Err(DeclError::BadDefault(name)),
        },
        _ => return Err(DeclError::BadDefault(name)),
    };
    let mut hint = PropHint::default();
    if let Some(t) = options {
        for pair in t.pairs::<Value, Value>() {
            let Ok((k, v)) = pair else {
                return Err(DeclError::BadHint(name));
            };
            let key = match &k {
                Value::String(s) => s.to_str().map(|s| s.to_owned()).unwrap_or_default(),
                _ => String::new(),
            };
            let slot = match key.as_str() {
                "min" => &mut hint.min,
                "max" => &mut hint.max,
                "step" => &mut hint.step,
                _ => {
                    return Err(DeclError::UnknownOption {
                        name,
                        option: if key.is_empty() {
                            format!("{k:?}")
                        } else {
                            key
                        },
                    });
                }
            };
            *slot = Some(match v {
                Value::Number(n) => n,
                #[expect(clippy::cast_precision_loss, reason = "o Luau guarda números como f64")]
                Value::Integer(i) => i as f64,
                _ => return Err(DeclError::BadHint(name)),
            });
        }
    }
    Ok(PropDecl {
        name,
        default,
        hint,
    })
}

#[cfg(test)]
#[path = "module_tests.rs"]
mod tests;
