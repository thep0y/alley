import { createContext, useContext } from "solid-js";

export const AppContext = createContext<AppContext>();

export function useAppContext() {
  const context = useContext(AppContext);
  if (!context) {
    throw new Error("useCounterContext: cannot find a AppContext");
  }
  return context;
}
