let resized = false;

export async function js_track_resized_setup() {
    window.addEventListener("resize", () => {
        resized = true;
    });
}

export function js_poll_resized() {
    let ret = resized;
    resized = false;
    return ret;
}

export function js_bundt_api_server() {
    return globalThis.apiServer || "http://localhost:8080/api";
}

export function js_bundt_secure_api_server() {
    return globalThis.secureApiServer || "http://localhost:8080/api";
}
