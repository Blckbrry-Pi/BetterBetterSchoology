const invoke = window.__TAURI__.invoke

alert(window.__TAURI__)

export async function invokeGetClassListing() {
    return await invoke("get_class_listing");
}

export async function invokeSetCredentials(username, password) {
    return await invoke("set_credentials", { username, password });
}

export async function parseSingleClassInfo(classid) {
    return await invoke("parse_single_class_info", { classid });
}