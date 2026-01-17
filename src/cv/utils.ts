import type { CameraSelection } from "./types";

export const cameraSelectionKey = (selection: CameraSelection): string => {
  if (selection.kind === "Index") {
    return `index:${selection.value}`;
  }
  return `stable:${selection.value}`;
};
