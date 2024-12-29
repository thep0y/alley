import { invoke } from "@tauri-apps/api/core";

export const startListening = () => invoke<void>("start_listening");

export const startBroadcasting = () => invoke<void>("start_broadcasting");

export const stopListening = () => invoke<void>("stop_listening");

export const stopBroadcasting = () => invoke<void>("stop_broadcasting");
