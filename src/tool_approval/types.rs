use gpui_kit::SharedString;

use crate::{
    animated_badge::{BadgeStatus},
    icon::{IconName},
    };

/// beui's `ToolApprovalStatus`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolApprovalStatus {
    Pending,
    Approving,
    Approved,
    Denied,
    Running,
    Complete,
    Error,
}

impl ToolApprovalStatus {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Pending => "Approval required",
            Self::Approving => "Approving",
            Self::Approved => "Approved",
            Self::Denied => "Denied",
            Self::Running => "Running",
            Self::Complete => "Completed",
            Self::Error => "Failed",
        }
    }

    pub(super) fn busy(self) -> bool {
        matches!(self, Self::Approving | Self::Running)
    }

    pub(super) fn icon(self) -> IconName {
        if self.busy() {
            return IconName::Progress;
        }
        match self {
            Self::Error => IconName::Error,
            Self::Denied => IconName::Close,
            Self::Approved | Self::Complete => IconName::Check,
            _ => IconName::VerifiedUser,
        }
    }

    /// The badge's status: amber, a turning ring while it works, emerald and rose.
    pub(super) fn badge(self) -> BadgeStatus {
        match self {
            Self::Pending => BadgeStatus::Warning,
            Self::Approving | Self::Running => BadgeStatus::Loading,
            Self::Approved | Self::Complete => BadgeStatus::Success,
            Self::Denied | Self::Error => BadgeStatus::Danger,
        }
    }
}

/// A parameter's value: plain mono text, or a mono chip like beui's `ToolApprovalCode`.
#[derive(Clone, Debug)]
pub enum ParamValue {
    Text(SharedString),
    Code(SharedString),
}
