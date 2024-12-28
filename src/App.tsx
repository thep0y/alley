import { For, Show, createSignal, onMount } from "solid-js";
import { BiRegularSun, BiSolidMoon } from "solid-icons/bi";
import { LazyFlex, LazySwitch, LazyTooltip } from "./lazy";
import "~/App.scss";
import useDark from "alley-components/lib/hooks/useDark";
import RippleEffect from "./components/ripple";
import { createStore } from "solid-js/store";
import { AiFillAndroid } from "solid-icons/ai";
import { showWindow } from "./commands/window";
import { getPeers } from "./api";

const App = () => {
  const [isDark, setIsDark] = useDark();

  const [remoteAccesses] = createStore<Remote[]>([]);
  const [receiveEvent] = createSignal<ReceiveEvent | null>(null);

  onMount(async () => {
    if (import.meta.env.TAURI_ENV_PLATFORM !== "android") {
      showWindow();
    }
    console.log("获取 peers");
    const peers = await getPeers();
    console.log("****");
    console.log(peers);
  });

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
        <Show when={receiveEvent() !== "End"}>
          <RippleEffect />
        </Show>

        <Show
          when={!remoteAccesses.length}
          fallback={<div>已搜索到设备：</div>}
        >
          <Show
            when={receiveEvent() === "End"}
            fallback={<div>正在搜索其他设备...</div>}
          >
            <div>搜索结束</div>
          </Show>
        </Show>

        <For each={remoteAccesses}>
          {(item) => {
            return (
              <div>
                <AiFillAndroid /> {item.name}
              </div>
            );
          }}
        </For>
      </LazyFlex>
    </>
  );
};

export default App;
