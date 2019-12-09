/*
 * Market API
 *
 * OpenAPI spec version: 1.0.0
 *
 * Generated by: https://openapi-generator.tech
 */

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Proposal {
    #[serde(rename = "id")]
    id: Option<String>,
    #[serde(rename = "properties")]
    properties: Value,
    #[serde(rename = "constraints")]
    constraints: String,
    #[serde(rename = "prevProposalId")]
    prev_proposal_id: Option<String>,
}

impl Proposal {
    pub fn new(properties: Value, constraints: String) -> Proposal {
        Proposal {
            id: None,
            properties: properties,
            constraints: constraints,
            prev_proposal_id: None,
        }
    }

    pub fn set_id(&mut self, id: String) {
        self.id = Some(id);
    }

    pub fn with_id(mut self, id: String) -> Proposal {
        self.id = Some(id);
        self
    }

    pub fn id(&self) -> Option<&String> {
        self.id.as_ref()
    }

    pub fn reset_id(&mut self) {
        self.id = None;
    }

    pub fn set_properties(&mut self, properties: Value) {
        self.properties = properties;
    }

    pub fn with_properties(mut self, properties: Value) -> Proposal {
        self.properties = properties;
        self
    }

    pub fn properties(&self) -> &Value {
        &self.properties
    }

    pub fn set_constraints(&mut self, constraints: String) {
        self.constraints = constraints;
    }

    pub fn with_constraints(mut self, constraints: String) -> Proposal {
        self.constraints = constraints;
        self
    }

    pub fn constraints(&self) -> &String {
        &self.constraints
    }

    pub fn set_prev_proposal_id(&mut self, prev_proposal_id: String) {
        self.prev_proposal_id = Some(prev_proposal_id);
    }

    pub fn with_prev_proposal_id(mut self, prev_proposal_id: String) -> Proposal {
        self.prev_proposal_id = Some(prev_proposal_id);
        self
    }

    pub fn prev_proposal_id(&self) -> Option<&String> {
        self.prev_proposal_id.as_ref()
    }

    pub fn reset_prev_proposal_id(&mut self) {
        self.prev_proposal_id = None;
    }
}