//! Text-only preview for an already prepared Zcash wallet proposal.
//!
//! This module never creates, signs, or broadcasts transactions. The caller
//! must keep its actual SDK proposal immutable and pass the same `GateReview`
//! into the wallet's checked authorization path after this preview returns.

use std::io::{self, BufRead, Write};

use leaklens_core::Certainty;
use leaklens_gate::{GateDecision, GateReview};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreviewOutcome {
    /// Return to the wallet to construct a new proposal and rerun inspection.
    ReviseProposal,
    /// User finished reviewing; the wallet must still run the gate.
    ProceedToWallet,
    /// The preview did not establish user approval. No signing is authorized.
    Abort,
}

pub fn preview_review(
    review: &mut GateReview,
    mut input: impl BufRead,
    output: &mut impl Write,
) -> io::Result<PreviewOutcome> {
    writeln!(output, "LeakLens — pre-signing privacy review")?;
    writeln!(output, "This is a risk assessment, not a guarantee of anonymity.")?;

    for finding in &review.assessment().findings {
        let certainty = match finding.certainty {
            Certainty::PublicDisclosure => "PUBLIC DISCLOSURE",
            Certainty::PossibleCorrelation => "POSSIBLE CORRELATION",
            Certainty::Unknown => "INSPECTION INCOMPLETE",
        };
        let location = match finding.payment_index {
            Some(payment) => format!("step {}, payment {}", finding.step_index + 1, payment + 1),
            None => format!("step {}", finding.step_index + 1),
        };
        writeln!(output, "{} [{}] {}: {}", finding.code, certainty, location, finding.message)?;
    }

    match review.decision() {
        GateDecision::DenyIncomplete => {
            writeln!(output, "INSPECTION INCOMPLETE: authorization is unavailable; revise or abort.")?;
            writeln!(output, "Type REVISE to change the payment; any other input aborts.")?;
            let choice = read_choice(&mut input)?;
            Ok(if choice == "REVISE" { PreviewOutcome::ReviseProposal } else { PreviewOutcome::Abort })
        }
        GateDecision::RequireAcknowledgment => {
            writeln!(output, "Type ACKNOWLEDGE to accept the listed risks, or REVISE to change the payment.")?;
            let choice = read_choice(&mut input)?;
            if choice == "REVISE" {
                return Ok(PreviewOutcome::ReviseProposal);
            }
            if choice != "ACKNOWLEDGE" {
                return Ok(PreviewOutcome::Abort);
            }
            review.acknowledge().map_err(|error| io::Error::other(format!("{error:?}")))?;
            Ok(PreviewOutcome::ProceedToWallet)
        }
        GateDecision::Allow => {
            writeln!(output, "No known exposure was flagged by this policy. This is not an anonymity guarantee.")?;
            writeln!(output, "Type CONTINUE to return to the wallet authorization gate, or REVISE.")?;
            match read_choice(&mut input)?.as_str() {
                "CONTINUE" => Ok(PreviewOutcome::ProceedToWallet),
                "REVISE" => Ok(PreviewOutcome::ReviseProposal),
                _ => Ok(PreviewOutcome::Abort),
            }
        }
    }
}

fn read_choice(input: &mut impl BufRead) -> io::Result<String> {
    let mut line = String::new();
    input.read_line(&mut line)?;
    Ok(line.trim().to_owned())
}
