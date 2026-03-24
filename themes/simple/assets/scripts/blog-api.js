function api_delete_article(article_id, callback) {
    let params = {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({ "article_id": article_id }),
    };
    fetch(`/api/article/remove`, params)
        .then((response) => {
            fetch_alert_error(response) && callback(response);
        })
        .catch((error) => {
            tips_show("tips-item-error", error);
        });
}

function api_delete_attachment(attachment_id, callback) {
    let params = {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({ "attachment_id": attachment_id }),
    };
    fetch(`/api/attachment/remove`, params)
        .then((response) => {
            fetch_alert_error(response) && callback(response);
        })
        .catch((error) => {
            tips_show("tips-item-error", error);
        });
}

async function api_upload_resource(name, file, callback) {
    const file_sha256 = await sha256(file);

    const xhr = new XMLHttpRequest();
    xhr.open('POST', '/api/resource/upload');

    xhr.setRequestHeader('x-file-size', file.size.toString());
    xhr.setRequestHeader('x-file-name', encodeURIComponent(name));
    xhr.setRequestHeader('x-file-mime-type', file.type || 'application/octet-stream');
    xhr.setRequestHeader('x-file-sha256', file_sha256);

    xhr.upload.onprogress = (event) => {
        if (event.lengthComputable) {
            callback.progress(event.loaded, event.total);
        }
    };
    xhr.onerror = (error) => {
        tips_show("tips-item-error", error);
    };
    xhr.ontimeout = () => {
        tips_show("tips-item-error", "上传超时，请重试");
    };
    xhr.onload = () => {
        xhr_alert_error(xhr) && callback.success(xhr);
    };

    xhr.send(file);
}

async function api_upload_attachment(article_id, name, file, callback) {
    const file_sha256 = await sha256(file);

    const xhr = new XMLHttpRequest();
    xhr.open('POST', '/api/attachment/upload');

    xhr.setRequestHeader('x-article-id', article_id);
    xhr.setRequestHeader('x-file-size', file.size.toString());
    xhr.setRequestHeader('x-file-name', encodeURIComponent(name));
    xhr.setRequestHeader('x-file-mime-type', file.type || 'application/octet-stream');
    xhr.setRequestHeader('x-file-sha256', file_sha256);

    xhr.upload.onprogress = (event) => {
        if (event.lengthComputable) {
            callback.progress(event.loaded, event.total);
        }
    };
    xhr.onerror = (error) => {
        tips_show("tips-item-error", error);
    };
    xhr.ontimeout = () => {
        tips_show("tips-item-error", "上传超时，请重试");
    };
    xhr.onload = () => {
        xhr_alert_error(xhr) && callback.success(xhr);
    };

    xhr.send(file);
}

function fetch_alert_error(response) {
    if (response.status >= 400 && response.status <= 599) {
        response.json().then((data) => {
            tips_show("tips-item-error", data.message);
        }).catch((error) => {
            tips_show("tips-item-error", error);
        });
        return false;
    }
    return true;
}

function xhr_alert_error(xhr) {
    if (xhr.status >= 400 && xhr.status <= 599) {
        try {
            tips_show("tips-item-error", JSON.parse(xhr.responseText).data.message);
        } catch {
            tips_show("tips-item-error", xhr.responseText);
        }
        return false;
    }
    return true;
}
