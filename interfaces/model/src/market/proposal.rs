/*
 * Yagna Market API
 *
 *  ## Yagna Market The Yagna Market is a core component of the Yagna Network, which enables computational Offers and Demands circulation. The Market is open for all entities willing to buy computations (Demands) or monetize computational resources (Offers). ## Yagna Market API The Yagna Market API is the entry to the Yagna Market through which Requestors and Providers can publish their Demands and Offers respectively, find matching counterparty, conduct negotiations and make an agreement.  This version of Market API conforms with capability level 1 of the <a href=\"https://docs.google.com/document/d/1Zny_vfgWV-hcsKS7P-Kdr3Fb0dwfl-6T_cYKVQ9mkNg\"> Market API specification</a>.  Market API contains two roles: Requestors and Providers which are symmetrical most of the time (excluding agreement phase).
 *
 * The version of the OpenAPI document: 1.4.2
 *
 * Generated by: https://openapi-generator.tech
 */

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Proposal {
    #[serde(rename = "properties")]
    pub properties: serde_json::Value,
    #[serde(rename = "constraints")]
    pub constraints: String,
    #[serde(rename = "proposalId", skip_serializing_if = "Option::is_none")]
    pub proposal_id: Option<String>,
    #[serde(rename = "issuerId", skip_serializing_if = "Option::is_none")]
    pub issuer_id: Option<String>,
    /// * `Initial` - proposal arrived from the market as response to subscription
    /// * `Draft` - bespoke counter-proposal issued by one party directly to other party (negotiation phase)
    /// * `Rejected` by other party
    /// * `Accepted` - promoted into the Agreement draft
    /// * `Expired` - not accepted nor rejected before validity period
    #[serde(rename = "state", skip_serializing_if = "Option::is_none")]
    pub state: Option<State>,
    /// id of the proposal from other side which this proposal responds to
    #[serde(rename = "prevProposalId", skip_serializing_if = "Option::is_none")]
    pub prev_proposal_id: Option<String>,
}

impl Proposal {
    pub fn new(properties: serde_json::Value, constraints: String) -> Proposal {
        Proposal {
            properties,
            constraints,
            proposal_id: None,
            issuer_id: None,
            state: None,
            prev_proposal_id: None,
        }
    }
}

/// * `Initial` - proposal arrived from the market as response to subscription
/// * `Draft` - bespoke counter-proposal issued by one party directly to other party (negotiation phase)
/// * `Rejected` by other party
/// * `Accepted` - promoted into the Agreement draft
/// * `Expired` - not accepted nor rejected before validity period
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum State {
    #[serde(rename = "Initial")]
    Initial,
    #[serde(rename = "Draft")]
    Draft,
    #[serde(rename = "Rejected")]
    Rejected,
    #[serde(rename = "Accepted")]
    Accepted,
    #[serde(rename = "Expired")]
    Expired,
}
