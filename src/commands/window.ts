import { invoke } from "@tauri-apps/api/core";

export const showWindow = (label = "main") =>
  invoke<void>("show_window", { label });
