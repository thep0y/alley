import { For, Show } from "solid-js";
import type { Message } from "./types";

interface MessageBubbleProps {
  message: Message;
}

const MessageBubble = (props: MessageBubbleProps) => {
  return (
    <div class={`message-wrapper ${props.message.isSelf ? "self" : "other"}`}>
      <div class="message-bubble">
        <Show when={props.message.content}>
          <div class="message-text">{props.message.content}</div>
        </Show>
        <Show when={props.message.files?.length}>
          <div class="message-files">
            <For each={props.message.files}>
              {(file) => (
                <div class="file-card">
                  <div class="file-icon">📎</div>
                  <div class="file-info">
                    <div class="file-name">{file.name}</div>
                    <div class="file-size">{file.size}</div>
                  </div>
                </div>
              )}
            </For>
          </div>
        </Show>
        <div class="message-timestamp">{props.message.timestamp}</div>
      </div>
    </div>
  );
};

export default MessageBubble;
