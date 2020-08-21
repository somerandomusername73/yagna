use crate::agreement::Agreement;
use crate::message::*;
use actix::prelude::*;
use byte_unit::{Byte as Bytes, ByteUnit};
use std::ffi::OsString;
use std::path::PathBuf;
use ya_runtime_api::deploy::StartMode;

mod event;
pub mod process;

pub trait Runtime:
    Actor<Context = Context<Self>>
    + Handler<Shutdown>
    + Handler<RuntimeCommand>
    + Handler<SetTaskPackagePath>
    + Handler<SetRuntimeMode>
{
}

#[derive(Clone, Debug)]
pub enum RuntimeMode {
    ProcessPerCommand,
    Service,
}

impl Default for RuntimeMode {
    fn default() -> Self {
        RuntimeMode::ProcessPerCommand
    }
}

impl From<StartMode> for RuntimeMode {
    fn from(mode: StartMode) -> Self {
        match mode {
            StartMode::Empty => RuntimeMode::ProcessPerCommand,
            StartMode::Blocking => RuntimeMode::Service,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeArgs {
    workdir: PathBuf,
    task_package: Option<PathBuf>,
    cpu_cores: Option<i32>,
    mem: Option<Bytes>,
    storage: Option<Bytes>,
}

impl RuntimeArgs {
    pub fn new(work_dir: &PathBuf, agreement: &Agreement, with_inf: bool) -> Self {
        let mut cpu_cores = None;
        let mut mem = None;
        let mut storage = None;
        if with_inf {
            // TODO: Can I have 1.23 cores, please?
            cpu_cores = agreement.infrastructure.get("cpu.cores").map(|v| *v as i32);
            // TODO: we should pass bytes as an integer
            mem = agreement
                .infrastructure
                .get("mem.gib")
                .map(|v| Bytes::from_unit(*v, ByteUnit::GiB).unwrap());
            storage = agreement
                .infrastructure
                .get("storage.gib")
                .map(|v| Bytes::from_unit(*v, ByteUnit::GiB).unwrap());
        }

        RuntimeArgs {
            workdir: work_dir.clone(),
            task_package: None,
            cpu_cores,
            mem: mem,
            storage: storage,
        }
    }

    pub fn to_command_line(&self, package_path: &PathBuf) -> Vec<OsString> {
        let mut args = vec![
            OsString::from("--workdir"),
            self.workdir.clone().into_os_string(),
            OsString::from("--task-package"),
            package_path.clone().into_os_string(),
        ];
        if let Some(val) = self.cpu_cores {
            args.extend(vec![
                OsString::from("--cpu-cores"),
                OsString::from(val.to_string()),
            ]);
        }
        if let Some(val) = self.mem {
            args.extend(vec![
                OsString::from("--mem"),
                OsString::from(val.get_bytes().to_string()),
            ]);
        }
        if let Some(val) = self.storage {
            args.extend(vec![
                OsString::from("--storage"),
                OsString::from(val.get_bytes().to_string()),
            ]);
        }
        args
    }
}
