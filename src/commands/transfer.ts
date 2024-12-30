import { invoke } from "@tauri-apps/api/core";

export const sendText = async (targetId: string, content: string) =>
  invoke<string>("send_text", { targetId, content });
