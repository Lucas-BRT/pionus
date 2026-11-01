use topcoat::{
    Result,
    view::{View, component, view},
};

use crate::ui::{
    doc_link::doc_link,
    docs::doc_section,
    terminal::{note, terminal},
};

#[component]
pub async fn section() -> Result<impl View> {
    Ok(view! {
        doc_section(
            id: "report",
            title: "Report a bad message",
            terminal(
                title: "trace",
                label: "Record a generation and replay it",
                "pionus --trace bad.json\n"
                note("or: PIONUS_TRACE=bad.json git commit")
                "pionus replay bad.json\n"
                note("checks that it reproduces")
            )
            <p>
                "A trace holds your staged diff and recent commit subjects. Review it before sharing. "
                <code>"pionus -v"</code>
                " shows what was decided; "
                <code>"pionus prompt"</code>
                " shows the prompt."
            </p>
            <p>
                "All commands: the "
                doc_link(page: "usage.md", "usage guide")
                "."
            </p>
        )
    })
}
