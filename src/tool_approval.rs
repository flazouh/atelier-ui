//! beui's ToolApproval (`components/agents/tool-approval.tsx`), class for class:
//!
//! - Row `items-start gap-3 p-4`: a `size-8` status tile (`ShieldCheck`, a spinning `LoaderCircle`,
//!   `Check`, `X`, or `CircleAlert`), the title `font-medium`, the tool in mono `text-xs` at muted, and a
//!   status pill (`rounded-full px-2 py-0.5 text-[11px]`) tinted amber, blue, emerald, or rose by
//!   `ToolApprovalStatus`.
//! - An optional description at `leading-5` muted, and a "View details" toggle (`text-xs`, a `size-3.5`
//!   chevron on `SPRING_SWAP`) shown only when there are parameters.
//! - Details disclose like `AgentDisclosure`: a `rounded-xl` card of label/value rows, label column
//!   `7rem`, a parameter's value in mono, or in a mono chip for `ParamValue::Code` (beui's
//!   `ToolApprovalCode`).
//! - The action row (`Allow once`, an optional `Always allow`, `Deny`) shows only while `Pending`,
//!   fading in and rising 4px over 220ms (120ms reduced), fading out the same way, and the details close
//!   themselves on leaving `Pending`.
//! - Borderless: every `border` token becomes a `card`/`card_strong` fill.

mod helpers;
mod structs;
mod types;

pub use helpers::head_words;
pub use structs::ToolApproval;
pub use types::{ParamValue, ToolApprovalStatus};

#[cfg(test)]
use gpui_kit::div;
#[cfg(test)]
use crate::tool_preview::ToolPreview;

#[cfg(test)]
mod tests;
