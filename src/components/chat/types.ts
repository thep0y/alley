export interface FileInfo {
  id: string;
  name: string;
  size: string;
  type: string;
}

export type MessageType = "text" | "mixed";

export interface Message {
  id: number;
  type: MessageType;
  content: string;
  files?: FileInfo[];
  isSelf: boolean;
  timestamp: string;
}

export interface ChatProps {
  peerName: string;
  targetID: string;
}
