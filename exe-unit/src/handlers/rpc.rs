use crate::error::Error;
use crate::message::{GetBatchResults, GetMetrics};
use crate::runtime::Runtime;
use crate::{ExeUnit, RuntimeRef};
use actix::prelude::*;
use chrono::Utc;
use futures::channel::oneshot;
use futures::prelude::*;
use std::time::Duration;
use tokio::time::timeout;
use ya_client_model::activity::{ActivityState, ActivityUsage, ExeScriptCommandResult};
use ya_core_model::activity::sgx::CallEncryptedService;
use ya_core_model::activity::*;
use ya_service_bus::RpcEnvelope;

impl<R: Runtime> Handler<RpcEnvelope<Exec>> for ExeUnit<R> {
    type Result = <RpcEnvelope<Exec> as Message>::Result;

    fn handle(&mut self, msg: RpcEnvelope<Exec>, ctx: &mut Self::Context) -> Self::Result {
        self.ctx.verify_activity_id(&msg.activity_id)?;

        let batch_id = msg.batch_id.clone();
        if self.state.batches.contains_key(&batch_id) {
            let msg = format!("Batch {} already exists", batch_id);
            return Err(RpcMessageError::BadRequest(msg));
        }

        let (tx, rx) = oneshot::channel();
        self.state.batches.insert(batch_id.clone(), msg.clone());
        self.state.batch_control.insert(batch_id.clone(), Some(tx));

        let fut = RuntimeRef::from_ctx(&ctx)
            .exec(
                self.runtime.clone(),
                self.transfers.clone(),
                msg.into_inner(),
                rx,
            )
            .into_actor(self);
        ctx.spawn(fut);

        Ok(batch_id)
    }
}

impl<R: Runtime> Handler<RpcEnvelope<GetState>> for ExeUnit<R> {
    type Result = <RpcEnvelope<GetState> as Message>::Result;

    fn handle(&mut self, msg: RpcEnvelope<GetState>, _: &mut Self::Context) -> Self::Result {
        self.ctx.verify_activity_id(&msg.activity_id)?;

        Ok(ActivityState {
            state: self.state.inner.clone(),
            reason: None,
            error_message: None,
        })
    }
}

impl<R: Runtime> Handler<RpcEnvelope<GetUsage>> for ExeUnit<R> {
    type Result = ActorResponse<Self, ActivityUsage, RpcMessageError>;

    fn handle(&mut self, msg: RpcEnvelope<GetUsage>, _: &mut Self::Context) -> Self::Result {
        if let Err(e) = self.ctx.verify_activity_id(&msg.activity_id) {
            return ActorResponse::reply(Err(e.into()));
        }

        let metrics = self.metrics.clone();
        let fut = async move {
            let resp = match metrics.send(GetMetrics).await {
                Ok(r) => r,
                Err(e) => {
                    log::warn!("Unable to report activity usage: {:?}", e);
                    return Err(Error::from(e).into());
                }
            };

            match resp {
                Ok(data) => Ok(ActivityUsage {
                    current_usage: Some(data),
                    timestamp: Utc::now().timestamp(),
                }),
                Err(e) => Err(Error::from(e).into()),
            }
        };

        ActorResponse::r#async(fut.into_actor(self))
    }
}

impl<R: Runtime> Handler<RpcEnvelope<GetRunningCommand>> for ExeUnit<R> {
    type Result = <RpcEnvelope<GetRunningCommand> as Message>::Result;

    fn handle(
        &mut self,
        msg: RpcEnvelope<GetRunningCommand>,
        _: &mut Self::Context,
    ) -> Self::Result {
        self.ctx.verify_activity_id(&msg.activity_id)?;

        match &self.state.running_command {
            Some(command) => Ok(command.clone()),
            None => Err(RpcMessageError::NotFound(format!(
                "no command is running within activity id: {}",
                msg.activity_id
            ))),
        }
    }
}

impl<R: Runtime> Handler<RpcEnvelope<GetExecBatchResults>> for ExeUnit<R> {
    type Result = ActorResponse<Self, Vec<ExeScriptCommandResult>, RpcMessageError>;

    fn handle(
        &mut self,
        msg: RpcEnvelope<GetExecBatchResults>,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        if let Err(err) = self.ctx.verify_activity_id(&msg.activity_id) {
            return ActorResponse::reply(Err(err.into()));
        }

        let idx = match self.state.batches.get(&msg.batch_id) {
            Some(exec) => match exec.exe_script.len() {
                0 => return ActorResponse::reply(Ok(Vec::new())),
                len => msg.command_index.unwrap_or(len - 1),
            },
            None => {
                let err = RpcMessageError::NotFound(format!("batch_id = {}", msg.batch_id));
                return ActorResponse::reply(Err(err));
            }
        };

        let address = ctx.address();
        let duration = Duration::from_secs_f32(msg.timeout.unwrap_or(0.));
        let notifier = self.state.notifier(&msg.batch_id).clone();
        let fut = async move {
            if let Err(_) = timeout(duration, notifier.when(move |i| i >= idx)).await {
                if msg.command_index.is_some() {
                    return Err(RpcMessageError::Timeout);
                }
            }

            match address.send(GetBatchResults(msg.batch_id.clone())).await {
                Ok(mut results) => {
                    results.0.truncate(idx + 1);
                    Ok(results.0)
                }
                _ => Ok(Vec::new()),
            }
        };

        ActorResponse::r#async(fut.into_actor(self))
    }
}

impl<R: Runtime> Handler<RpcEnvelope<sgx::CallEncryptedService>> for ExeUnit<R> {
    type Result = ResponseFuture<Result<Vec<u8>, RpcMessageError>>;

    fn handle(
        &mut self,
        msg: RpcEnvelope<CallEncryptedService>,
        ctx: &mut Context<Self>,
    ) -> Self::Result {
        use ya_client_model::activity::encrypted::{
            Request, RequestCommand, Response, RpcMessageError as ApiError,
        };

        let me = ctx.address();
        let ctx = ya_client_model::activity::encrypted::EncryptionCtx::new(
            unimplemented!(),
            unimplemented!(),
        );

        async move {
            let bytes = msg.bytes.as_slice();
            // TODO: Decrypt message
            let body: Request = ctx
                .decrypt(msg.bytes.as_slice())
                .map_err(|e| RpcMessageError::BadRequest(e.to_string()))?;

            match body.command {
                RequestCommand::Exec { exe_script, .. } => {
                    let msg = RpcEnvelope::with_caller(
                        msg.caller(),
                        Exec {
                            activity_id: body.activity_id,
                            batch_id: body.batch_id,
                            exe_script,
                            timeout: body.timeout,
                        },
                    );
                    let response = me.send(msg).await.map_err(|_| {
                        RpcMessageError::Service("fatal: exe-unit disconnected".to_string())
                    })?;
                    response
                }
                RequestCommand::GetExecBatchResults { command_index, .. } => {
                    let msg = RpcEnvelope::with_caller(
                        msg.caller(),
                        GetExecBatchResults {
                            activity_id: body.activity_id,
                            batch_id: body.batch_id,
                            timeout: body.timeout,
                            command_index,
                        },
                    );
                    let response = me.send(msg).await.map_err(|_e| {
                        RpcMessageError::Service("fatal: exe-unit disconnected".to_string())
                    })?;
                    response
                }
            }
        }
        .then(|v| {
            let response = match v {
                Err(e) => sgx::Response::Error(e),
                Ok(v) => v,
            };
            let bytes = match serde_json::to_vec(&response) {
                Ok(bytes) => bytes,
                Err(_) => {
                    return future::err(RpcMessageError::Service("unexpected error".to_string()))
                }
            };
            // TODO encrypt bytes.

            future::ok(bytes)
        })
        .boxed_local()
    }
}
