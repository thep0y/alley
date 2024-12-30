import { For, Show, createEffect, createSignal, onMount } from "solid-js";
import { createStore } from "solid-js/store";
import "./index.scss";
import type { ChatProps, FileInfo, Message } from "./types";
import { formatFileSize } from "./utils";
import ChatHeader from "./ChatHeader";
import MessageBubble from "./MessageBubble";
import FileUpload from "./FileUpload";
import SelectedFiles from "./SelectedFiles";
import ChatInput from "./ChatInput";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { sendText } from "~/commands/transfer";

const Chat = (props: ChatProps) => {
  const [isMobile, setIsMobile] = createSignal(window.innerWidth < 768);
  const [messages, setMessages] = createStore<Message[]>([
    {
      id: 1,
      type: "text",
      content: "你好！",
      isSelf: false,
      timestamp: "14:00",
    },
  ]);
  const [inputValue, setInputValue] = createStore({ text: "" });
  const [selectedFiles, setSelectedFiles] = createStore<FileInfo[]>([]);

  let textareaRef: HTMLTextAreaElement | undefined;
  let fileInputRef: HTMLInputElement | undefined;

  onMount(async () => {
    const unlisten = await getCurrentWebviewWindow().listen<TransferTask>(
      "transfer-update",
      (e) => {
        const { id, peer_id, transfer_type } = e.payload;
        if ("content" in transfer_type) {
          const newMessage: Message = {
            id: messages.length + 1,
            type: selectedFiles.length > 0 ? "mixed" : "text",
            content: transfer_type.content,
            isSelf: false,
            timestamp: new Date().toLocaleTimeString("zh-CN", {
              hour: "2-digit",
              minute: "2-digit",
            }),
          };
          setMessages([...messages, newMessage]);
        }
      },
    );

    return () => unlisten();
  });

  createEffect(() => {
    const handleResize = () => setIsMobile(window.innerWidth < 768);
    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  });

  const handleFileSelect = (e: Event) => {
    const input = e.target as HTMLInputElement;
    if (input.files) {
      const newFiles = Array.from(input.files).map((file) => ({
        id: Math.random().toString(36).substr(2, 9),
        name: file.name,
        size: formatFileSize(file.size),
        type: file.type,
      }));
      setSelectedFiles([...selectedFiles, ...newFiles]);
      if (fileInputRef) fileInputRef.value = "";
    }
  };

  const removeFile = (fileId: string) => {
    setSelectedFiles(selectedFiles.filter((file) => file.id !== fileId));
  };

  const adjustTextareaHeight = (e: Event) => {
    const textarea = e.target as HTMLTextAreaElement;
    textarea.style.height = "auto";
    textarea.style.height = `${Math.min(textarea.scrollHeight, 150)}px`;
  };

  const resetTextareaHeight = () => {
    if (textareaRef) {
      textareaRef.style.height = "auto";
      textareaRef.style.height = "24px";
    }
  };

  const handleInput = (e: Event) => {
    setInputValue({ text: (e.target as HTMLTextAreaElement).value });
    adjustTextareaHeight(e);
  };

  const sendMessage = async () => {
    const text = inputValue.text.trim();
    if (!text && selectedFiles.length === 0) return;

    const newMessage: Message = {
      id: messages.length + 1,
      type: selectedFiles.length > 0 ? "mixed" : "text",
      content: text,
      files: selectedFiles.length > 0 ? [...selectedFiles] : undefined,
      isSelf: true,
      timestamp: new Date().toLocaleTimeString("zh-CN", {
        hour: "2-digit",
        minute: "2-digit",
      }),
    };

    if (newMessage.type === "text") {
      if (text) await sendText(props.targetID, text);
    }

    setMessages([...messages, newMessage]);
    setInputValue({ text: "" });
    setSelectedFiles([]);
    resetTextareaHeight();
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (!isMobile() && e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  };

  return (
    <div class="chat-container">
      <ChatHeader peerName={props.peerName} />

      <div class="chat-messages">
        <For each={messages}>
          {(message) => <MessageBubble message={message} />}
        </For>
      </div>

      <div class="chat-input">
        <div class="input-container">
          <Show when={!isMobile()}>
            <FileUpload
              isMobile={false}
              onFileSelect={handleFileSelect}
              fileInputRef={(el) => {
                fileInputRef = el;
              }}
            />
          </Show>

          <div class="input-area">
            <Show when={selectedFiles.length > 0}>
              <SelectedFiles files={selectedFiles} onRemove={removeFile} />
            </Show>

            <ChatInput
              value={inputValue.text}
              isMobile={isMobile()}
              onInput={handleInput}
              onKeyDown={handleKeyDown}
              onSend={sendMessage}
              textareaRef={(el) => {
                textareaRef = el;
              }}
            />
          </div>

          <Show when={isMobile()}>
            <FileUpload
              isMobile={true}
              onFileSelect={handleFileSelect}
              fileInputRef={(el) => {
                fileInputRef = el;
              }}
            />
          </Show>
        </div>
      </div>
    </div>
  );
};

export default Chat;
