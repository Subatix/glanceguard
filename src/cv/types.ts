export type CameraSelection =
  | { kind: "Index"; value: number }
  | { kind: "StableId"; value: string };

export type CameraInfo = {
  id: CameraSelection;
  name: string;
  description: string;
};

export type BoundingBox = {
  x: number;
  y: number;
  width: number;
  height: number;
};

export type DebugFace = {
  id: number;
  bbox: BoundingBox;
  label: string;
  similarity?: number | null;
  observerScore?: number | null;
};

export type FrameEvent = {
  frameWidth: number;
  frameHeight: number;
  faces: DebugFace[];
  observerScore?: number | null;
  state: string;
  image?: number[];
};

export type AlertEvent = {
  score: number;
  reason: string;
  cooldownSec: number;
};

export type ErrorEvent = {
  message: string;
};
