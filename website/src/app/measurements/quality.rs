use topcoat::{
    Result,
    view::{View, component, view},
};

use crate::{
    measures::QUALITY,
    ui::{
        data_table::{cell, data_table, row},
        doc_link::doc_link,
        docs::doc_section,
    },
};

/// How well the fine-tuned models write, from the Results of the training
/// document.
#[component]
pub async fn section() -> Result<impl View> {
    Ok(view! {
        doc_section(
            id: "quality",
            title: "Quality of the fine-tunes",
            <p>
                "Scored with "
                <code>"pionus-training score"</code>
                " on eval-v1 (500 commits of repositories never seen in training) and eval-short (young repositories). A difference under about 0.06 is noise."
            </p>
            for table in QUALITY {
                data_table(
                    label: "Scores of the fine-tuned models",
                    head: table.head,
                    for line in table.rows {
                        row(
                            for (cell_text, name) in line.iter().zip(table.head) {
                                cell(label: name, (*cell_text))
                            }
                        )
                    }
                )
            }
            <p class="mt-2">
                "How the models were trained and judged is in the "
                doc_link(page: "training.md", "training document")
                "."
            </p>
        )
    })
}
