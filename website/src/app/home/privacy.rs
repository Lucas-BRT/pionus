use topcoat::{
    Result,
    icon::{IconData, icon as glyph},
    view::{Child, StaticClass, View, attributes, class, component, view},
};

use crate::{
    facts::DEFAULT_MODEL_SIZE,
    ui::{
        container::container,
        icons::{GLOBE, LAPTOP, NO_ACCOUNT, OFFLINE, PACKAGE, UNPLUG, icon_badge},
    },
};

const PANEL: StaticClass = class!(
    "grid items-center gap-10 rounded-3xl border border-border bg-card p-8",
    "md:p-14 lg:grid-cols-2 lg:gap-12",
);

/// Scene icons keep a thin line at any size, like the line between them.
const SCENE_ICON: StaticClass =
    class!("shrink-0 [&_*]:stroke-[3] [&_*]:[vector-effect:non-scaling-stroke]");

/// One thing the tool does not need, with its icon.
#[component]
async fn point(icon: IconData, title: &str, #[default] child: Child<'_>) -> Result<impl View> {
    Ok(view! {
        <li class="m-0 flex gap-4">
            icon_badge(data: icon)
            <div>
                <h3>(title)</h3>
                <p class="mt-1 mb-0 text-muted-foreground">(child)</p>
            </div>
        </li>
    })
}

#[component]
pub async fn section() -> Result<impl View> {
    Ok(view! {
        <section class="mt-24 md:mt-32" aria-labelledby="private">
            container(
                <div class=(PANEL)>
                    <div>
                        <h2 id="private">"Nothing leaves your machine"</h2>
                        <ul class="m-0 mt-8 grid max-w-none list-none gap-6 p-0">
                            point(
                                icon: NO_ACCOUNT,
                                title: "Nothing to sign up for",
                                "No account, no API key, no subscription."
                            )
                            point(
                                icon: PACKAGE,
                                title: "A small model",
                                "A "
                                (DEFAULT_MODEL_SIZE)
                                " download that runs on your hardware."
                            )
                            point(
                                icon: OFFLINE,
                                title: "Offline to write",
                                "Download once, run anywhere."
                            )
                        </ul>
                    </div>
                    <div
                        class="mx-auto flex w-full max-w-md items-center gap-3 text-muted-foreground"
                        role="img"
                        aria-label="A laptop unplugged from the internet"
                    >
                        glyph(data: LAPTOP, size: 64, attrs: attributes! { class=(class!(SCENE_ICON, "text-primary")) })
                        <span class="h-0.75 flex-1 rounded-full bg-current"></span>
                        glyph(data: UNPLUG, size: 64, attrs: attributes! { class=(class!(SCENE_ICON, "rotate-45")) })
                        <span class="h-0.75 flex-1 rounded-full bg-current"></span>
                        glyph(data: GLOBE, size: 64, attrs: attributes! { class=(SCENE_ICON) })
                    </div>
                </div>
            )
        </section>
    })
}
