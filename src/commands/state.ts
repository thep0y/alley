import { invoke } from "@tauri-apps/api/core";

export const getSelfID = async () => invoke<string>("get_self_id");
