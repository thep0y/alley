import { invoke } from "@tauri-apps/api/core";

export const getPeers = () => invoke<PeerInfo[]>("get_peers");
