use topcoat::{
    Result,
    icon::icon,
    view::{Attributes, Child, StaticClass, View, attributes, class, component, view},
};

use crate::{
    components::button::{ButtonSize, ButtonVariant, button},
    ui::icons::{CHECK, COPY},
};

const BAR: StaticClass = class!(
    "flex items-center gap-3 rounded-xl border border-border bg-terminal px-4 py-3.5",
    "text-left font-mono text-[0.85rem] leading-relaxed text-terminal-foreground",
    "select-all [overflow-wrap:anywhere]",
);

/// A command on its own, in a bar that selects whole when clicked, with a
/// button that copies it.
#[component]
pub async fn command_line(
    #[default] mut attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
    // The site's only script (website/AGENTS.md); without it the bar still selects whole.
    Ok(view! {
        <div class=(class!(BAR, attrs.remove("class"))) (attrs)>
            <span class="text-primary">"$"</span>
            <span>(child)</span>
            button(
                variant: ButtonVariant::Ghost,
                size: ButtonSize::Icon,
                attrs: attributes! {
                    type="button"
                    class="group ml-auto"
                    aria-label="Copy"
                    onclick="navigator.clipboard.writeText(this.previousElementSibling.textContent).then(()=>{this.classList.add('copied');this.ariaLabel='Copied'})"
                },
                icon(data: COPY, size: 16, attrs: attributes! { class="group-[.copied]:hidden" })
                icon(data: CHECK, size: 16, attrs: attributes! { class="hidden group-[.copied]:block" })
            )
        </div>
    })
}
