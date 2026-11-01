use topcoat::{
    Result,
    view::{View, component, view},
};

use crate::ui::{
    data_table::{cell, data_table, row},
    docs::doc_section,
};

#[component]
pub async fn section() -> Result<impl View> {
    Ok(view! {
        doc_section(
            id: "messages",
            title: "Messages and what to do",
            data_table(
                label: "Messages and what to do",
                head: &["Message", "What to do"],
                row(
                    cell(
                        label: "Message",
                        <code>"pionus is not set up yet"</code>
                    )
                    cell(
                        label: "What to do",
                        "Run "
                        <code>"pionus setup"</code>
                        "."
                    )
                )
                row(
                    cell(label: "Message", <code>"no device could run the model"</code>)
                    cell(
                        label: "What to do",
                        "Run "
                        <code>"pionus setup"</code>
                        ". The message lists what each device lacks."
                    )
                )
                row(
                    cell(
                        label: "Message",
                        <code>"no model in …"</code>
                        ", "
                        <code>"no GGUF model at …"</code>
                        ", "
                        <code>"no llama-server at …"</code>
                    )
                    cell(
                        label: "What to do",
                        "Run "
                        <code>"pionus setup"</code>
                        ", or "
                        <code>"pionus model use <id>"</code>
                        "."
                    )
                )
                row(
                    cell(
                        label: "Message",
                        <code>"… has no OpenVINO version, which npu needs"</code>
                    )
                    cell(
                        label: "What to do",
                        "This model runs on "
                        <code>"vulkan"</code>
                        " and "
                        <code>"cpu"</code>
                        " only. For the NPU: "
                        <code>"pionus model use qwen2.5-coder-1.5b"</code>
                        "."
                    )
                )
                row(
                    cell(label: "Message", <code>"OpenVINO GenAI not found"</code>)
                    cell(
                        label: "What to do",
                        "Only "
                        <code>"npu"</code>
                        ", "
                        <code>"intel-gpu"</code>
                        " and "
                        <code>"openvino-cpu"</code>
                        " need it. Use "
                        <code>"vulkan"</code>
                        " or "
                        <code>"cpu"</code>
                        "."
                    )
                )
                row(
                    cell(
                        label: "Message",
                        <code>
                            "model.device = gpu … is the old name of intel-gpu"
                        </code>
                    )
                    cell(
                        label: "What to do",
                        <code>"pionus device use intel-gpu"</code>
                        " (or "
                        <code>"auto"</code>
                        ")."
                    )
                )
                row(
                    cell(
                        label: "Message",
                        <code>"cannot load the model on the NPU"</code>
                    )
                    cell(
                        label: "What to do",
                        "Run "
                        <code>"~/.local/share/pionus/scripts/check-npu.sh"</code>
                        ". After joining the "
                        <code>"render"</code>
                        " group, log out and back in."
                    )
                )
                row(
                    cell(
                        label: "Message",
                        "NVIDIA GPU "
                        <code>"has no driver bound"</code>
                    )
                    cell(
                        label: "What to do",
                        "Install a driver ("
                        <code>"sudo ubuntu-drivers install"</code>
                        ") or use nouveau, then "
                        <code>"pionus setup"</code>
                        "."
                    )
                )
                row(
                    cell(label: "Message", "The hook does nothing")
                    cell(
                        label: "What to do",
                        "It acts on "
                        <code>"git commit"</code>
                        " without "
                        <code>"-m"</code>
                        ". Reinstall it with "
                        <code>"pionus hook install"</code>
                        "."
                    )
                )
                row(
                    cell(
                        label: "Message",
                        "Anything else, or you do not know what is wrong"
                    )
                    cell(
                        label: "What to do",
                        "Run "
                        <code>"pionus doctor"</code>
                        "."
                    )
                )
            )
        )
    })
}
