#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoolKind {
    Transparent,
    Sapling,
    Orchard,
    Ironwood,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecipientKind {
    Transparent,
    Shielded,
    Unified,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentFacts {
    pub recipient_kind: RecipientKind,
    pub selected_pool: PoolKind,
    pub amount_zat: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepFacts {
    pub payments: Vec<PaymentFacts>,
    pub has_transparent_inputs: bool,
    pub has_shielded_inputs: bool,
    /// The adapter has established that this step is funded by a shielded
    /// output from an earlier step of the same proposal.
    pub has_prior_shielded_funding: bool,
    pub change_pools: Vec<PoolKind>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProposalFacts {
    pub steps: Vec<StepFacts>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Warning,
    Caution,
    Incomplete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Certainty {
    PublicDisclosure,
    PossibleCorrelation,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    pub code: &'static str,
    pub step_index: usize,
    pub payment_index: Option<usize>,
    pub severity: Severity,
    pub certainty: Certainty,
    pub message: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Policy {
    pub version: u32,
}

impl Default for Policy {
    fn default() -> Self {
        Self { version: 1 }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivacyAssessment {
    pub findings: Vec<Finding>,
    pub complete: bool,
    pub policy_version: u32,
}
