//! Public, read-only API for privacy assessment. This service cannot sign,
//! decrypt wallet seeds, construct spend authorizations, or broadcast funds.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::{HeaderValue, Request, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use leaklens_core::{
    Certainty, Finding, PaymentFacts, Policy, PoolKind, PrivacyAssessment, ProposalFacts,
    RecipientKind, Severity, StepFacts, assess,
};
use leaklens_zcash::{extract_proposal_facts, fixture};
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

const MAX_STEPS: usize = 16;
const MAX_PAYMENTS_PER_STEP: usize = 128;
const MAX_CHANGE_POOLS_PER_STEP: usize = 128;
const MAX_BODY_BYTES: usize = 32 * 1024;
const MAX_ACTIVE_ASSESSMENTS: usize = 8;

#[derive(Clone)]
struct AppState {
    allowed_origin: Option<HeaderValue>,
    assessment_permits: Arc<Semaphore>,
}

/// Configures the optional allowed web origin. Without one, this API does
/// not emit any CORS permission, so ordinary cross-origin browser calls fail.
pub fn router(allowed_origin: Option<String>) -> Router {
    router_with_capacity(allowed_origin, MAX_ACTIVE_ASSESSMENTS)
}

/// Capacity is explicit to support deterministic overload tests. Production
/// uses a conservative fixed bound of eight in-flight assessments.
pub fn router_with_capacity(allowed_origin: Option<String>, capacity: usize) -> Router {
    let state = Arc::new(AppState {
        allowed_origin: allowed_origin.and_then(|origin| {
            if origin.starts_with("https://") || origin.starts_with("http://localhost:") {
                HeaderValue::from_str(&origin).ok()
            } else {
                None
            }
        }),
        assessment_permits: Arc::new(Semaphore::new(capacity)),
    });
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/fixtures", get(fixtures))
        .route("/api/v1/assess", post(review).options(preflight))
        .route("/api/v1/review", post(review).options(preflight))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            cors_middleware,
        ))
        .with_state(state)
}

async fn cors_middleware(
    State(state): State<Arc<AppState>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let is_allowed = state
        .allowed_origin
        .as_ref()
        .is_some_and(|configured| request.headers().get(header::ORIGIN) == Some(configured));
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    response.headers_mut().insert(
        header::HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("no-referrer"),
    );
    response.headers_mut().insert(
        header::HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'"),
    );
    response.headers_mut().insert(
        header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    response
        .headers_mut()
        .insert(header::VARY, HeaderValue::from_static("Origin"));
    if is_allowed {
        let origin = state
            .allowed_origin
            .as_ref()
            .expect("allowed origin established");
        response
            .headers_mut()
            .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin.clone());
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET, POST, OPTIONS"),
        );
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("Content-Type"),
        );
    }
    response
}

async fn preflight() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status":"ok"}))
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pool {
    Transparent,
    Sapling,
    Orchard,
    Ironwood,
    Unknown,
}

impl From<Pool> for PoolKind {
    fn from(value: Pool) -> Self {
        match value {
            Pool::Transparent => Self::Transparent,
            Pool::Sapling => Self::Sapling,
            Pool::Orchard => Self::Orchard,
            Pool::Ironwood => Self::Ironwood,
            Pool::Unknown => Self::Unknown,
        }
    }
}

impl From<PoolKind> for Pool {
    fn from(value: PoolKind) -> Self {
        match value {
            PoolKind::Transparent => Self::Transparent,
            PoolKind::Sapling => Self::Sapling,
            PoolKind::Orchard => Self::Orchard,
            PoolKind::Ironwood => Self::Ironwood,
            PoolKind::Unknown => Self::Unknown,
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Recipient {
    Transparent,
    Shielded,
    Unified,
    Unknown,
}

impl From<Recipient> for RecipientKind {
    fn from(value: Recipient) -> Self {
        match value {
            Recipient::Transparent => Self::Transparent,
            Recipient::Shielded => Self::Shielded,
            Recipient::Unified => Self::Unified,
            Recipient::Unknown => Self::Unknown,
        }
    }
}

impl From<RecipientKind> for Recipient {
    fn from(value: RecipientKind) -> Self {
        match value {
            RecipientKind::Transparent => Self::Transparent,
            RecipientKind::Shielded => Self::Shielded,
            RecipientKind::Unified => Self::Unified,
            RecipientKind::Unknown => Self::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaymentDto {
    pub recipient_kind: Recipient,
    pub selected_pool: Pool,
    pub amount_zat: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepDto {
    pub payments: Vec<PaymentDto>,
    pub has_transparent_inputs: bool,
    pub has_shielded_inputs: bool,
    pub has_prior_shielded_funding: bool,
    pub change_pools: Vec<Pool>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalDto {
    pub steps: Vec<StepDto>,
}

impl From<ProposalDto> for ProposalFacts {
    fn from(dto: ProposalDto) -> Self {
        Self {
            steps: dto
                .steps
                .into_iter()
                .map(|step| StepFacts {
                    payments: step
                        .payments
                        .into_iter()
                        .map(|payment| PaymentFacts {
                            recipient_kind: payment.recipient_kind.into(),
                            selected_pool: payment.selected_pool.into(),
                            amount_zat: payment.amount_zat,
                        })
                        .collect(),
                    has_transparent_inputs: step.has_transparent_inputs,
                    has_shielded_inputs: step.has_shielded_inputs,
                    has_prior_shielded_funding: step.has_prior_shielded_funding,
                    change_pools: step.change_pools.into_iter().map(Into::into).collect(),
                })
                .collect(),
        }
    }
}

impl From<ProposalFacts> for ProposalDto {
    fn from(facts: ProposalFacts) -> Self {
        Self {
            steps: facts
                .steps
                .into_iter()
                .map(|step| StepDto {
                    payments: step
                        .payments
                        .into_iter()
                        .map(|payment| PaymentDto {
                            recipient_kind: payment.recipient_kind.into(),
                            selected_pool: payment.selected_pool.into(),
                            amount_zat: payment.amount_zat,
                        })
                        .collect(),
                    has_transparent_inputs: step.has_transparent_inputs,
                    has_shielded_inputs: step.has_shielded_inputs,
                    has_prior_shielded_funding: step.has_prior_shielded_funding,
                    change_pools: step.change_pools.into_iter().map(Into::into).collect(),
                })
                .collect(),
        }
    }
}

#[derive(Serialize)]
pub struct FindingDto {
    code: &'static str,
    step_index: usize,
    payment_index: Option<usize>,
    severity: &'static str,
    certainty: &'static str,
    message: &'static str,
}

impl From<Finding> for FindingDto {
    fn from(finding: Finding) -> Self {
        Self {
            code: finding.code,
            step_index: finding.step_index,
            payment_index: finding.payment_index,
            severity: match finding.severity {
                Severity::Warning => "warning",
                Severity::Caution => "caution",
                Severity::Incomplete => "incomplete",
            },
            certainty: match finding.certainty {
                Certainty::PublicDisclosure => "public_disclosure",
                Certainty::PossibleCorrelation => "possible_correlation",
                Certainty::Unknown => "unknown",
            },
            message: finding.message,
        }
    }
}

#[derive(Serialize)]
pub struct AssessmentDto {
    pub source: &'static str,
    pub complete: bool,
    pub policy_version: u32,
    pub findings: Vec<FindingDto>,
}

fn assessment_dto(assessment: PrivacyAssessment, source: &'static str) -> AssessmentDto {
    AssessmentDto {
        source,
        complete: assessment.complete,
        policy_version: assessment.policy_version,
        findings: assessment.findings.into_iter().map(Into::into).collect(),
    }
}

fn validate_proposal(facts: &ProposalDto) -> Result<(), &'static str> {
    if facts.steps.len() > MAX_STEPS {
        return Err("too many steps");
    }
    for step in &facts.steps {
        if step.payments.len() > MAX_PAYMENTS_PER_STEP {
            return Err("too many payments in a step");
        }
        if step.change_pools.len() > MAX_CHANGE_POOLS_PER_STEP {
            return Err("too many change outputs in a step");
        }
    }
    Ok(())
}

fn over_capacity() -> Response {
    let mut response = (
        StatusCode::TOO_MANY_REQUESTS,
        Json(serde_json::json!({"error":"assessment capacity exhausted; retry shortly"})),
    )
        .into_response();
    response
        .headers_mut()
        .insert(header::RETRY_AFTER, HeaderValue::from_static("1"));
    response
}

async fn review(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ProposalDto>,
) -> impl IntoResponse {
    let Ok(_permit) = state.assessment_permits.clone().try_acquire_owned() else {
        return over_capacity();
    };
    if let Err(message) = validate_proposal(&request) {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"error":message})),
        )
            .into_response();
    }
    let facts = ProposalFacts::from(request);
    let assessment = assess(&facts, &Policy::default());
    (
        StatusCode::OK,
        Json(assessment_dto(assessment, "user_supplied_facts")),
    )
        .into_response()
}

#[derive(Serialize)]
struct FixtureDto {
    id: &'static str,
    source: &'static str,
    facts: ProposalDto,
    assessment: AssessmentDto,
}

async fn fixtures(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let Ok(_permit) = state.assessment_permits.clone().try_acquire_owned() else {
        return over_capacity();
    };
    let cases = [
        ("shielded", fixture::sample_shielded_proposal()),
        ("transparent", fixture::sample_transparent_proposal()),
    ];
    let mut output = Vec::with_capacity(cases.len());
    for (id, sdk_proposal) in cases {
        let facts = match extract_proposal_facts(&sdk_proposal) {
            Ok(value) => value,
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error":"fixture extraction failed"})),
                )
                    .into_response();
            }
        };
        let assessment = assess(&facts, &Policy::default());
        output.push(FixtureDto {
            id,
            source: "sdk_fixture",
            facts: facts.into(),
            assessment: assessment_dto(assessment, "sdk_fixture"),
        });
    }
    (StatusCode::OK, Json(serde_json::json!({"fixtures":output}))).into_response()
}
