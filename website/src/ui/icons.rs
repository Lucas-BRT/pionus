//! The few icons of the site, from the pinned Iconify lucide set (see
//! `build.rs`), drawn in the current text color.

use topcoat::{
    Result,
    icon::{IconData, icon, iconify::iconify_icon},
    view::{View, attributes, component, view},
};

pub const GPU: IconData = iconify_icon!("lucide:gpu");

/// The AI chip.
pub const NPU: IconData = iconify_icon!("lucide:brain-circuit");

pub const CPU: IconData = iconify_icon!("lucide:cpu");

/// A person who is not there: nothing to sign up for.
pub const NO_ACCOUNT: IconData = iconify_icon!("lucide:user-x");

/// A box: a model that is a file to download.
pub const PACKAGE: IconData = iconify_icon!("lucide:package");

/// A signal crossed out: no network.
pub const OFFLINE: IconData = iconify_icon!("lucide:wifi-off");

pub const COPY: IconData = iconify_icon!("lucide:copy");

/// Shown once something is copied.
pub const CHECK: IconData = iconify_icon!("lucide:check");

/// An icon in a small square of its own, for a list of features.
#[component]
pub async fn icon_badge(data: IconData) -> Result<impl View> {
    Ok(view! {
        <span class="grid size-10 shrink-0 place-items-center rounded-lg bg-background text-primary">
            icon(data: data, size: 20)
        </span>
    })
}

/// An icon for the head of a card.
#[component]
pub async fn card_icon(data: IconData) -> Result<impl View> {
    Ok(view! {
        icon(data: data, size: 28, attrs: attributes! { class="mb-2 text-primary" })
    })
}
