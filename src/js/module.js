export function js_build_interface() {
    return {
        env: {
            log_info: window.wasmBindings.log_info,
        },
    };
}
