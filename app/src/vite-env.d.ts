/// <reference types="vite/client" />

declare module "*.module.css" {
  const classes: { [key: string]: string };
  export default classes;
}

declare interface Window {
  __TAURI__?: {
    core?: {
      invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
      listen?: (event: string, cb: (e: { payload: unknown }) => void) => Promise<() => void>;
    };
  };
}
