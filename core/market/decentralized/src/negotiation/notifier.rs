use std::fmt::{Debug, Display};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::broadcast::{channel, Receiver, Sender};

#[derive(Error, Debug)]
pub enum NotifierError<Type>
where
    Type: Debug + PartialEq + Clone + Display,
{
    #[error("Timeout while waiting for events for subscription [{0}]")]
    Timeout(Type),
    #[error("Unsubscribed [{0}]")]
    Unsubscribed(Type),
    #[error("Channel closed while waiting for events for subscription [{0}]")]
    ChannelClosed(Type),
}

/// Allows to listen to new incoming events and notify if event was generated.
#[derive(Clone)]
pub struct EventNotifier<Type>
where
    Type: Debug + PartialEq + Clone + Display,
{
    sender: Sender<Notification<Type>>,
}

/// Thanks to EventNotifierListener we can create separate object, that already collects
/// events, before we call wait_for_event function. This way we can avoid
/// losing events.
pub struct EventNotifierListener<Type>
where
    Type: Debug + PartialEq + Clone + Display,
{
    receiver: Receiver<Notification<Type>>,
    subscription_id: Type,
}

#[derive(Clone)]
enum Notification<Type>
where
    Type: Debug + PartialEq + Clone + Display,
{
    NewEvent(Type),
    StopEvents(Type),
}

impl<Type> EventNotifier<Type>
where
    Type: Debug + PartialEq + Clone + Display,
{
    pub fn new() -> EventNotifier<Type> {
        // We will create receivers later, when someone needs it.
        let (sender, _receiver) = channel(100);
        EventNotifier { sender }
    }

    pub async fn notify(&self, subscription_id: &Type) {
        let sender = self.sender.clone();
        let to_send = Notification::<Type>::NewEvent(subscription_id.clone());
        // TODO: How to handle this error?
        let _ = sender.send(to_send);
    }

    pub async fn stop_notifying(&self, subscription_id: &Type) {
        let sender = self.sender.clone();
        let to_send = Notification::<Type>::StopEvents(subscription_id.clone());
        // TODO: How to handle this error?
        let _ = sender.send(to_send);
    }

    pub fn listen(&self, subscription_id: &Type) -> EventNotifierListener<Type> {
        EventNotifierListener::<Type> {
            receiver: self.sender.subscribe(),
            subscription_id: subscription_id.clone(),
        }
    }
}

impl<Type> EventNotifierListener<Type>
where
    Type: Debug + PartialEq + Clone + Display,
{
    pub async fn wait_for_event(&mut self) -> Result<(), NotifierError<Type>> {
        while let Ok(value) = self.receiver.recv().await {
            match value {
                Notification::<Type>::NewEvent(value) => {
                    if value == self.subscription_id {
                        return Ok(());
                    }
                }
                Notification::<Type>::StopEvents(subscription_id) => {
                    return Err(NotifierError::Unsubscribed(subscription_id));
                }
            }
        }
        Err(NotifierError::ChannelClosed(self.subscription_id.clone()))
    }

    pub async fn wait_for_event_with_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<(), NotifierError<Type>> {
        tokio::time::timeout(timeout, self.wait_for_event())
            .await
            .map_err(|_| NotifierError::Timeout(self.subscription_id.clone()))?
    }
}
