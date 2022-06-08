/**
 * @type {typeof import("@tauri-apps/api")}
 */
const tauri = window.__TAURI__;

const invoke = tauri.invoke;

export async function invokeGetClassListing() {
    return await invoke("get_class_listing");
}

export async function invokeSetCredentials(username, password) {
    return await invoke("set_credentials", { username, password });
}
