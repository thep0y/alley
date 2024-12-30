import { lazy } from "solid-js";

export const LazyButton = lazy(
  () => import("alley-components/lib/components/button"),
);
export const LazyRow = lazy(
  () => import("alley-components/lib/components/row"),
);
export const LazyCol = lazy(
  () => import("alley-components/lib/components/col"),
);
export const LazyProgress = lazy(
  () => import("alley-components/lib/components/progress"),
);
export const LazyDropdown = lazy(
  () => import("alley-components/lib/components/dropdown"),
);
export const LazyTooltip = lazy(
  () => import("alley-components/lib/components/tooltip"),
);
export const LazyLink = lazy(
  () => import("alley-components/lib/components/link"),
);
export const LazyFlex = lazy(
  () => import("alley-components/lib/components/flex"),
);
export const LazyEmpty = lazy(
  () => import("alley-components/lib/components/empty"),
);
export const LazyList = lazy(
  () => import("alley-components/lib/components/list"),
);
export const LazyListItem = lazy(
  () => import("alley-components/lib/components/list/item"),
);
export const LazyFileTypeIcon = lazy(
  () => import("~/components/file-type-icon"),
);
export const LazyFloatButton = lazy(
  () => import("alley-components/lib/components/float-button/button"),
);
export const LazyFloatButtonGroup = lazy(
  () => import("alley-components/lib/components/float-button/group"),
);
export const LazySwitch = lazy(
  () => import("alley-components/lib/components/switch"),
);
export const LazySpace = lazy(
  () => import("alley-components/lib/components/space"),
);
export const LazyTypographyText = lazy(
  () => import("alley-components/lib/components/typography/text"),
);
export const LazyToast = lazy(
  () => import("alley-components/lib/components/toast"),
);
export const LazyDialog = lazy(
  () => import("alley-components/lib/components/dialog"),
);
export const LazyQrcode = lazy(() => import("~/components/qrcode"));
