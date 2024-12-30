import { For, Show, children, createSignal, onMount } from "solid-js";
import { BiRegularSun, BiSolidMoon } from "solid-icons/bi";
import {
  LazyButton,
  LazyFlex,
  LazySpace,
  LazySwitch,
  LazyToast,
  LazyTooltip,
} from "./lazy";
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
import TimeoutDialog from "./components/timeout-dialog";
import { PairStatusTips } from "./tips";
import Chat from "./components/chat";
import { AppContext } from "./context";

enum PairRole {
  Master = 1,
  Slave = 2,
}

const App = () => {
  const [isDark, setIsDark] = useDark();

  const [peers, setPeers] = createStore<PeerInfo[]>([]);
  const [discoveryEvent, setDiscoveryEvent] = createSignal<
    "Start" | "End" | null
  >(null);
  const [pairDialog, setPairDialog] = createSignal<[string, PairStatus] | null>(
    null,
  );
  const [pairStatus, setPairStatus] = createSignal<{
    status: PairStatus;
    id: string;
  }>();
  const [pairRole, setPairRole] = createSignal<PairRole>();

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
        const [id, status] = e.payload;
        if (status === "REQUEST_RECEIVED" || status === "REQUESTED") {
          setPairRole(
            status === "REQUEST_RECEIVED" ? PairRole.Master : PairRole.Slave,
          );
          setPairDialog(e.payload);
        } else {
          setPairDialog(null);
          setPairStatus({ status, id });
        }
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
          <LazySpace gap={8}>
            <span>
              <AiFillAndroid /> {item.hostname}@{item.addr}:{item.port}
            </span>
            <LazyButton
              onClick={() => {
                sendPairRequest(item.id);
              }}
              size="small"
            >
              配对
            </LazyButton>
          </LazySpace>
        );
      }}
    </For>
  ));

  return (
    <AppContext.Provider value={{ goHome: () => setPairStatus() }}>
      <Show
        when={pairStatus()?.status !== "PAIRED"}
        fallback={
          <Chat
            targetID={pairStatus()!.id}
            peerName={peers.find((p) => p.id === pairStatus()?.id)!.hostname}
          />
        }
      >
        <LazyTooltip
          text={`切换为${isDark() ? "亮" : "暗"}色`}
          placement="left"
        >
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

          <LazyButton
            onClick={async () => {
              setPeers([]);
              await clearPeers();
              discovery();
            }}
          >
            重新搜索
          </LazyButton>

          <Show when={!peers.length} fallback={<div>已搜索到设备：</div>}>
            <Show
              when={discoveryEvent() === "End"}
              fallback={<div>正在搜索其他设备...</div>}
            >
              <div>搜索结束</div>
            </Show>
          </Show>

          {peerList()}
        </LazyFlex>

        <Show when={pairDialog()}>
          <TimeoutDialog
            targetID={pairDialog()![0]}
            pairStatus={pairDialog()![1]}
            timeout={import.meta.env.MODE === "production" ? 30000 : 10000}
            onClose={() => setPairDialog(null)}
          />
        </Show>

        <LazyToast
          message={`${pairRole() === PairRole.Master ? "已" : "对方"}${PairStatusTips[pairStatus()?.status]}`}
          onClose={() => {
            setPairStatus();
          }}
          open={!!pairStatus()}
        />
      </Show>
    </AppContext.Provider>
  );
};

export default App;
