import type { TimeoutID } from "alley-components/lib/interface";
import { children, createSignal, onMount } from "solid-js";
import { respondPairRequest } from "~/commands/peers";
import { LazyButton, LazyDialog, LazyFlex } from "~/lazy";
import { PairStatusTips } from "~/tips";

interface TimeoutDialogProps {
  targetID: string;
  pairStatus: PairStatus;
  timeout: number; // ms
  onClose: () => void;
}

const TimeoutDialog = (props: TimeoutDialogProps) => {
  const [show, setShow] = createSignal(false);
  const [remainSeconds, setRemainSeconds] = createSignal(props.timeout);

  let intervalID: TimeoutID | undefined;

  onMount(() => {
    setShow(true);
    intervalID = setInterval(() => {
      setRemainSeconds((prev) => (prev > 1000 ? prev - 1000 : 0));

      if (remainSeconds() === 0) {
        // 超时即拒绝
        onReject();
        onClose();
      }
    }, 1000);

    return () => clearInterval(intervalID);
  });

  const onClose = () => {
    clearInterval(intervalID);
    setShow(false);
    setRemainSeconds(props.timeout);
    props.onClose();
  };

  const onAccept = async () => {
    await respondPairRequest(props.targetID, true);
    onClose();
  };

  const onReject = async () => {
    await respondPairRequest(props.targetID, false);
    onClose();
  };

  const foot = children(
    () =>
      props.pairStatus === "REQUEST_RECEIVED" && (
        <LazyFlex justify="between">
          <LazyButton onClick={onReject} danger>
            拒绝
          </LazyButton>
          <LazyButton onClick={onAccept}>接受</LazyButton>
        </LazyFlex>
      ),
  );

  return (
    <>
      <LazyDialog show={show()} onClose={() => { }} foot={foot()}>
        <div>
          {PairStatusTips[props.pairStatus]}
          {props.pairStatus === "REQUEST_RECEIVED"
            ? `, 来自${props.targetID}`
            : ""}
        </div>

        {remainSeconds() / 1000}
      </LazyDialog>
    </>
  );
};

export default TimeoutDialog;
