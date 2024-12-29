import { invoke } from "@tauri-apps/api/core";

export const startTcpServer = () => invoke<void>("start_server");
