use topcoat::{
    Result,
    view::{Child, View, component, view},
};

use crate::facts::docs_url;

/// An external link to a page of the pionus documentation.
#[component]
pub async fn doc_link(page: &str, #[default] child: Child<'_>) -> Result<impl View> {
    Ok(view! { <a href=(docs_url(page))>(child)</a> })
}
