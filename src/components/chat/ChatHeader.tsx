import { useAppContext } from "~/context";

interface ChatHeaderProps {
  peerName: string;
}

const ChatHeader = (props: ChatHeaderProps) => {
  const { goHome } = useAppContext();
  return (
    <div
      classList={{
        "chat-header": true,
        "chat-header-android": import.meta.env.TAURI_ENV_PLATFORM === "android",
      }}
    >
      <button type="button" onClick={goHome}>
        主页
      </button>
      {props.peerName}
      <button type="button">断开</button>
    </div>
  );
};

export default ChatHeader;
