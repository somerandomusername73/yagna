use futures::StreamExt;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

use ya_client::model::market::{Demand as ClientDemand, Offer as ClientOffer};
use ya_service_api_web::middleware::Identity;

use crate::db::model::{Demand, DisplayVec, Offer, SubscriptionId};
use crate::protocol::discovery::builder::DiscoveryBuilder;
use crate::protocol::discovery::{
    Discovery, DiscoveryRemoteError, GetOffers, OfferIdsReceived, OfferUnsubscribed,
    OffersReceived, Propagate, Reason,
};

pub mod error;
pub(crate) mod resolver;
pub(crate) mod store;

use crate::config::Config;
use crate::identity::IdentityApi;
use error::{MatcherError, MatcherInitError, ModifyOfferError};
use resolver::Resolver;
use std::iter::FromIterator;
use store::SubscriptionStore;

/// Stores proposal generated from resolver.
#[derive(Debug)]
pub struct RawProposal {
    pub offer: Offer,
    pub demand: Demand,
}

/// Receivers for events, that can be emitted from Matcher.
pub struct EventsListeners {
    pub proposal_receiver: UnboundedReceiver<RawProposal>,
}

/// Responsible for storing Offers and matching them with demands.
#[derive(Clone)]
pub struct Matcher {
    pub store: SubscriptionStore,
    pub resolver: Resolver,
    discovery: Discovery,
    identity: Arc<dyn IdentityApi>,
    config: Arc<Config>,
}

impl Matcher {
    pub fn new(
        store: SubscriptionStore,
        identity_api: Arc<dyn IdentityApi>,
        config: Arc<Config>,
    ) -> Result<(Matcher, EventsListeners), MatcherInitError> {
        let (proposal_sender, proposal_receiver) = unbounded_channel::<RawProposal>();
        let resolver = Resolver::new(store.clone(), proposal_sender);

        let discovery = DiscoveryBuilder::default()
            .data(identity_api.clone())
            .data(store.clone())
            .data(resolver.clone())
            .add_data_handler(on_offer_ids_received)
            .add_data_handler(on_offers_received)
            .add_data_handler(on_get_offers)
            .add_data_handler(on_offer_unsubscribed)
            .build();

        let matcher = Matcher {
            store,
            resolver,
            discovery,
            config,
            identity: identity_api,
        };

        let listeners = EventsListeners { proposal_receiver };
        Ok((matcher, listeners))
    }

    pub async fn bind_gsb(
        &self,
        public_prefix: &str,
        private_prefix: &str,
    ) -> Result<(), MatcherInitError> {
        self.discovery
            .bind_gsb(public_prefix, private_prefix)
            .await?;

        // We can't spawn broadcasts, before gsb is bound.
        // That's why we don't spawn this in Matcher::new.
        tokio::task::spawn_local(random_broadcast(self.clone()));
        Ok(())
    }

    // =========================================== //
    // Offer/Demand subscription
    // =========================================== //

    pub async fn subscribe_offer(
        &self,
        offer: &ClientOffer,
        id: &Identity,
    ) -> Result<Offer, MatcherError> {
        // TODO: Handle broadcast errors. Maybe we should retry if it failed.
        let offer = self.store.create_offer(id, offer).await?;
        self.resolver.receive(&offer);

        let _ = self
            .discovery
            .broadcast_offers(vec![offer.id.clone()])
            .await
            .map_err(|e| {
                log::warn!("Failed to broadcast offer [{}]. Error: {}.", offer.id, e,);
            });
        Ok(offer)
    }

    pub async fn unsubscribe_offer(
        &self,
        offer_id: &SubscriptionId,
        id: &Identity,
    ) -> Result<(), MatcherError> {
        self.store
            .unsubscribe_offer(offer_id, true, Some(id.identity))
            .await?;

        // Broadcast only, if no Error occurred in previous step.
        // We ignore broadcast errors. Unsubscribing was finished successfully, so:
        // - We shouldn't bother agent with broadcasts
        // - Unsubscribe message probably will reach other markets, but later.
        let _ = self
            .discovery
            .broadcast_unsubscribe(id.identity.to_string(), offer_id.clone())
            .await
            .map_err(|e| {
                log::warn!(
                    "Failed to broadcast unsubscribe offer [{1}]. Error: {0}.",
                    e,
                    offer_id
                );
            });
        Ok(())
    }

    pub async fn subscribe_demand(
        &self,
        demand: &ClientDemand,
        id: &Identity,
    ) -> Result<Demand, MatcherError> {
        let demand = self.store.create_demand(id, demand).await?;

        self.resolver.receive(&demand);
        Ok(demand)
    }

    pub async fn unsubscribe_demand(
        &self,
        demand_id: &SubscriptionId,
        id: &Identity,
    ) -> Result<(), MatcherError> {
        Ok(self.store.remove_demand(demand_id, id).await?)
    }

    // TODO: Remove anyhow replace with typed error.
    pub async fn list_our_offers(&self) -> Result<Vec<Offer>, anyhow::Error> {
        let identities = self.identity.list().await?;
        let store = self.store.clone();

        let mut our_offers = vec![];
        for node_id in identities.into_iter() {
            our_offers.append(&mut store.get_offers(Some(node_id)).await?)
        }

        Ok(our_offers)
    }
}

pub(crate) async fn on_offer_ids_received(
    resolver: Resolver,
    _caller: String,
    msg: OfferIdsReceived,
) -> Result<Vec<SubscriptionId>, ()> {
    // We shouldn't propagate Offer, if we already have it in our database.
    // Note that when we broadcast our Offer, it will reach us too, so it concerns
    // not only Offers from other nodes.
    Ok(resolver
        .store
        .filter_existing(msg.offers)
        .await
        .map_err(|e| log::warn!("Error filtering Offers. Error: {}", e))?)
}

pub(crate) async fn on_offers_received(
    resolver: Resolver,
    caller: String,
    msg: OffersReceived,
) -> Result<Vec<SubscriptionId>, ()> {
    let added_offers_ids = futures::stream::iter(msg.offers.into_iter())
        .filter_map(|offer| {
            let resolver = resolver.clone();
            let offer_id = offer.id.clone();
            async move {
                resolver
                    .store
                    .save_offer(offer)
                    .await
                    .map(|offer| {
                        resolver.receive(&offer);
                        offer.id
                    })
                    .map_err(|e| {
                        log::warn!("Failed to save Offer [{}]. Error: {}", &offer_id, &e);
                        e
                    })
                    .ok()
            }
        })
        .collect::<Vec<SubscriptionId>>()
        .await;

    log::info!(
        "Received new Offers from [{}]: \n{}",
        caller,
        DisplayVec(&added_offers_ids)
    );
    Ok(added_offers_ids)
}

pub(crate) async fn on_get_offers(
    resolver: Resolver,
    _caller: String,
    msg: GetOffers,
) -> Result<Vec<Offer>, DiscoveryRemoteError> {
    match resolver.store.get_offers_batch(msg.offers).await {
        Ok(offers) => Ok(offers),
        Err(e) => {
            log::error!("Failed to get batch offers. Error: {}", e);
            // TODO: Propagate error.
            Ok(vec![])
        }
    }
}

pub(crate) async fn on_offer_unsubscribed(
    store: SubscriptionStore,
    caller: String,
    msg: OfferUnsubscribed,
) -> Result<Propagate, ()> {
    store
        .unsubscribe_offer(&msg.offer_id, false, caller.parse().ok())
        .await
        .map(|_| Propagate::Yes)
        .or_else(|e| match e {
            ModifyOfferError::UnsubscribeError(_, _)
            | ModifyOfferError::UnsubscribedNotRemoved(_)
            | ModifyOfferError::RemoveError(_, _) => {
                log::error!("Propagating Offer unsubscription, despite error: {}", e);
                // TODO: how should we handle it locally?
                Ok(Propagate::Yes)
            }
            ModifyOfferError::NotFound(_) => Ok(Propagate::No(Reason::NotFound)),
            ModifyOfferError::Unsubscribed(_) => Ok(Propagate::No(Reason::Unsubscribed)),
            ModifyOfferError::Expired(_) => Ok(Propagate::No(Reason::Expired)),
        })
}

async fn random_broadcast(matcher: Matcher) {
    let broadcast_interval = matcher
        .config
        .clone()
        .discovery
        .mean_random_broadcast_interval
        .to_std()
        .map_err(|e| format!("Failed to initialize broadcast interval: {}", e))
        .unwrap();
    loop {
        let matcher = matcher.clone();
        match async move {
            let random_interval = randomize_interval(broadcast_interval);
            tokio::time::delay_for(random_interval).await;

            // We always broadcast our own Offers.
            let our_offers = matcher
                .list_our_offers()
                .await?
                .into_iter()
                .map(|offer| offer.id)
                .collect::<Vec<SubscriptionId>>();

            // Add some random subset of Offers to broadcast.
            let num_to_broadcast =
                (matcher.config.discovery.num_broadcasted_offers - our_offers.len() as u32).min(0);

            let all_offers = matcher
                .store
                .get_offers(None)
                .await?
                .into_iter()
                .map(|offer| offer.id)
                .collect::<Vec<SubscriptionId>>();

            // Filter our Offers from set.
            let all_offers_wo_ours = all_offers
                .into_iter()
                .collect::<HashSet<SubscriptionId>>()
                .difference(&HashSet::from_iter(our_offers.clone().into_iter()))
                .cloned()
                .collect::<Vec<SubscriptionId>>();
            let mut random_offers = all_offers_wo_ours
                .choose_multiple(&mut rand::thread_rng(), num_to_broadcast as usize)
                .cloned()
                .collect::<Vec<SubscriptionId>>();
            random_offers.extend(our_offers);

            matcher.discovery.broadcast_offers(random_offers).await?;
            // TODO: Remove anyhow replace with typed error.
            Result::<(), anyhow::Error>::Ok(())
        }
        .await
        {
            Err(e) => log::warn!(
                "Failed to send random subscriptions broadcast. Error: {}",
                e
            ),
            Ok(_) => (),
        }
    }
}

fn randomize_interval(interval: std::time::Duration) -> std::time::Duration {
    interval
}
