use topcoat::{
    Result,
    view::{View, attributes, component, view},
};

use crate::{
    facts::INSTALL_CMD,
    ui::{command_line::command_line, container::container},
};

#[component]
pub async fn section() -> Result<impl View> {
    Ok(view! {
        <section>
            container(
                attrs: attributes! { class="pt-12 text-center md:pt-20" },
                <h1
                    class="mx-auto max-w-[18ch] text-[clamp(2rem,4.8vw,3.5rem)] leading-[1.06]"
                >
                    "Commit messages in "
                    <em class="text-primary not-italic">"your repo's style"</em>
                </h1>
                <p class="mx-auto mt-5 max-w-[32rem] text-base text-muted-foreground">
                    "One command writes the commit message from your staged diff and your history."
                </p>
                command_line(
                    attrs: attributes! { class="mx-auto mt-9 w-fit max-w-full" },
                    (INSTALL_CMD)
                )
            )
        </section>
    })
}
