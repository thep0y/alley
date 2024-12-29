import { invoke } from "@tauri-apps/api/core";

export const getPeers = () => invoke<PeerInfo[]>("get_peers");

export const clearPeers = () => invoke<void>("clear_peers");

export const sendPairRequest = (targetId: string) =>
  invoke<void>("send_pair_request", { targetId });

export const respondPairRequest = (targetId: string, accepted: boolean) =>
  invoke<void>("respond_pair_request", { targetId, accepted });
