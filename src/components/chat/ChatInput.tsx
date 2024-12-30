import { Show } from "solid-js";

interface ChatInputProps {
  value: string;
  isMobile: boolean;
  onInput: (e: Event) => void;
  onKeyDown: (e: KeyboardEvent) => void;
  onSend: () => Promise<void>;
  textareaRef: (el: HTMLTextAreaElement) => void;
}

export const ChatInput = (props: ChatInputProps) => {
  return (
    <div class="input-wrapper">
      <textarea
        ref={props.textareaRef}
        value={props.value}
        onInput={props.onInput}
        onKeyDown={props.onKeyDown}
        placeholder="输入消息..."
        rows={1}
      />
      <Show when={props.isMobile}>
        <button type="button" class="send-button" onClick={props.onSend}>
          发送
        </button>
      </Show>
    </div>
  );
};

export default ChatInput;
