function modal_upload_file_register(target, callback) {
    modal_register(target, {
        on_open: (modal, _) => {
            let file_input = modal.querySelector("input[name=file]");
            let name_input = modal.querySelector("input[name=name]");
            let path_input = modal.querySelector("input[name=path]");
            file_input.value = "";
            name_input.value = "";
            path_input.value = "";

            callback.open && callback.open(modal);
        },
        on_confirm: (modal, _) => {
            let file_input = modal.querySelector("input[name=file]");
            let name_input = modal.querySelector("input[name=name]");

            if (file_input.files.length == 0) {
                return;
            }
            let file = file_input.files[0];

            callback.confirm && callback.confirm(modal, name_input.value, file);
        },
    });
    let modals = document.querySelectorAll(target);
    modals.forEach((modal) => {
        let file_input = modal.querySelector("input[name=file]");
        file_input.addEventListener("change", () => {
            let name_input = modal.querySelector("input[name=name]");
            if (file_input.files.length == 0) {
                name_input.value = "";
            } else {
                name_input.value = file_input.files[0].name;
            }
        });
    });
}

function modal_upload_attachment_register(target, callback) {
    modal_register(target, {
        on_open: (modal, _) => {
            let file_input = modal.querySelector("input[name=file]");
            let name_input = modal.querySelector("input[name=name]");
            file_input.value = "";
            name_input.value = "";

            callback.open && callback.open(modal);
        },
        on_confirm: (modal, _) => {
            let file_input = modal.querySelector("input[name=file]");
            let name_input = modal.querySelector("input[name=name]");

            if (file_input.files.length == 0) {
                return;
            }
            let file = file_input.files[0];

            callback.confirm && callback.confirm(modal, name_input.value, file);
        },
    });
    let modals = document.querySelectorAll(target);
    modals.forEach((modal) => {
        let file_input = modal.querySelector("input[name=file]");
        file_input.addEventListener("change", () => {
            let name_input = modal.querySelector("input[name=name]");
            if (file_input.files.length == 0) {
                name_input.value = "";
            } else {
                name_input.value = file_input.files[0].name;
            }
        });
    });
}

async function modal_upload_file_copy_path(modal_class_id) {
    let input = document.querySelector(`.${modal_class_id} input[name=path]`);
    try {
        await navigator.clipboard.writeText(input.value);
        tips_show("tips-item-success", "内容已复制到剪贴板");
    } catch (err) {
        tips_show("tips-item-error", `复制失败：${err}`);
    }
}
