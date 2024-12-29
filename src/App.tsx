import { For, Show, children, createSignal, onMount } from "solid-js";
import { BiRegularSun, BiSolidMoon } from "solid-icons/bi";
import { LazyButton, LazyFlex, LazySwitch, LazyTooltip } from "./lazy";
import "~/App.scss";
import useDark from "alley-components/lib/hooks/useDark";
import RippleEffect from "./components/ripple";
import { createStore } from "solid-js/store";
import { AiFillAndroid } from "solid-icons/ai";
import { showWindow } from "./commands/window";
import { getPeers } from "./api";
import {
  startBroadcasting,
  startListening,
  stopBroadcasting,
  stopListening,
} from "./commands/discovery";
import { clearPeers, sendPairRequest } from "./commands/peers";
import { startTcpServer } from "./commands/tcp";
import { getCurrentWebview } from "@tauri-apps/api/webview";

const App = () => {
  const [isDark, setIsDark] = useDark();

  const [peers, setPeers] = createStore<PeerInfo[]>([]);
  const [discoveryEvent, setDiscoveryEvent] = createSignal<
    "Start" | "End" | null
  >(null);

  const discovery = async () => {
    await startBroadcasting();
    await startListening();
    setDiscoveryEvent("Start");

    const intervalID = setInterval(async () => {
      const peers = await getPeers();
      setPeers(peers);
    }, 1000);

    const timeoutID = setTimeout(() => {
      stopBroadcasting();
      stopListening();
      setDiscoveryEvent("End");
      clearTimeout(intervalID);
      clearTimeout(timeoutID);
    }, 10000);
  };

  onMount(async () => {
    const unlisten = await getCurrentWebview().listen<[string, PairStatus]>(
      "pair-update",
      (e) => {
        console.log(e.payload);
      },
    );

    return () => unlisten();
  });

  onMount(async () => {
    if (import.meta.env.TAURI_ENV_PLATFORM !== "android") {
      showWindow();
    }

    await startTcpServer();

    discovery();
  });

  const peerList = children(() => (
    <For each={peers}>
      {(item) => {
        return (
          <LazyButton
            onClick={() => {
              sendPairRequest(item.id);
            }}
          >
            <AiFillAndroid /> {item.addr}:{item.port}
          </LazyButton>
        );
      }}
    </For>
  ));

  return (
    <>
      <LazyTooltip text={`切换为${isDark() ? "亮" : "暗"}色`} placement="left">
        <LazySwitch
          class="dark-switch"
          checked={isDark()}
          setChecked={() => {
            setIsDark((pre) => {
              return !pre;
            });
          }}
          uncheckedChild={<BiRegularSun />}
          checkedChild={<BiSolidMoon />}
        />
      </LazyTooltip>

      <LazyFlex
        direction="vertical"
        align="center"
        justify="center"
        style={{ width: "100%" }}
        gap={16}
      >
        <Show when={discoveryEvent() !== "End"}>
          <RippleEffect />
        </Show>

        <Show when={!peers.length} fallback={<div>已搜索到设备：</div>}>
          <Show
            when={discoveryEvent() === "End"}
            fallback={<div>正在搜索其他设备...</div>}
          >
            <div>搜索结束</div>
          </Show>
        </Show>

        <LazyButton
          onClick={async () => {
            setPeers([]);
            await clearPeers();
            discovery();
          }}
        >
          重新搜索
        </LazyButton>
        {peerList()}
      </LazyFlex>
    </>
  );
};

export default App;
