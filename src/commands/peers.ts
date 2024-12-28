import { invoke } from "@tauri-apps/api/core";

export const getPeers = () => invoke("get_peers");
