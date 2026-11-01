use topcoat::{
    Result,
    view::{View, component, view},
};

use crate::ui::{docs::doc_section, terminal::terminal};

#[component]
pub async fn section() -> Result<impl View> {
    Ok(view! {
        doc_section(
            id: "update-check",
            title: "The update notice",
            <p>
                "pionus makes no request by itself. To hear about new releases:"
            </p>
            terminal(
                title: "update notice",
                label: "Turn on the update notice",
                "pionus config set update.check true"
            )
            <p>
                "Once a day at most, it asks GitHub for the latest release. GitHub sees your IP address and "
                <code>"pionus/<version>"</code>
                ", nothing else. The hook never checks. "
                <code>"PIONUS_UPDATE_CHECK=false"</code>
                " turns it off for one command."
            </p>
        )
    })
}
